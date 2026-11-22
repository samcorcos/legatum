//! Training mode (designer, 2026-11-07; `docs/design/training-mode/fable.md` and its Addendum).
//!
//! A training seat plays Hard, except in its battles. For every battle in scope (a city it
//! attacks, a city of its own under attack, a field battle within reach of either) the searcher
//! cuts the local battle out of the true state into a small arena (the engine's own `blank_game`
//! and `resolve_tactical_turn`, as the combat lab does), with every unit of either side within
//! reach, the cities there with their HP and Walls, and the terrain. The teacher may see
//! everything: it is training, not the shipped AI.
//!
//! **Candidates.** Every legal order per searched unit: hold, fortify or heal, each move tile in
//! reach this turn, each ranged target, each melee target. Only the plainly bad are pruned: a
//! melee attack forecast to win less than 10% of the time, and moves away from the fight, which
//! collapse to the one retreat tile farthest from the enemy.
//!
//! **Search.** Coordinate descent: one unit at a time with the others fixed, repeated until no
//! single change improves the score (or `max_passes`). Each candidate is screened on
//! `screen` rollouts; the ones that beat the current plan there get the full `rollouts`, with
//! the same dice for every candidate (common random numbers).
//!
//! **Rollouts.** Each plays the battle out for up to `horizon` turns, ending early at a capture,
//! a loss or when one side has no army left. Hard's tactical planner continues for the seat;
//! the enemy alternates between Hard and a sortie policy (every unit strikes the weakest enemy
//! it can reach). The score is one integer: capture or hold, earlier is better, units and HP
//! lost and killed at their Production cost, and city HP.
//!
//! The run is a resumable state machine (`TrainingRun::step`), so the browser can run it in WASM
//! in short slices (the simulator runs it natively), and the caller enforces the time cap: when it hits,
//! the best orders found so far are played. Deterministic for a given state and budget.

use crate::ScriptedBrain;
use legatum_core::data;
use legatum_core::forecast::{AttackMode, forecast_attack_mode};
use legatum_core::game::blank_game;
use legatum_core::rng::Rng;
use legatum_core::rules::{is_military, resolve_tactical_turn};
use legatum_core::state::Map;
use legatum_core::view::PlayerView;
use legatum_core::{
    AiDifficulty, CityId, GameState, Hex, Order, PlayerId, Terrain, UnitId, UnitKind,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A seat unit this close to an enemy city or unit (or an enemy unit this close to a seat city)
/// starts a battle.
const ENGAGE: i32 = 2;
/// The seat's units this close to the battle's centre are searched.
const REACH: i32 = 3;
/// The arena: every tile within this many columns and rows of the centre.
const WINDOW: i32 = 7;
/// A melee attack forecast to win less often than this (percent) is a suicide, never searched.
const SUICIDE: u32 = 10;
/// Score weights, in Production (a Spearman costs 80).
const TURN_WEIGHT: i64 = 15;
const CITY_HP_WEIGHT: i64 = 1;
/// The RNG stream for training searches.
const STREAM: u64 = 0x7261_696e;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TrainingConfig {
    /// Rollouts per candidate that survives screening.
    pub rollouts: u32,
    /// Rollouts every candidate gets first.
    pub screen: u32,
    /// Turns each rollout may play.
    pub horizon: u32,
    /// Coordinate-descent passes over the units.
    pub max_passes: u32,
    /// Battles searched per seat and turn.
    pub max_battles: usize,
    /// Wall-clock cap for the whole turn, ms (0: none). The caller's clock enforces it.
    pub cap_ms: u64,
    /// Deterministic cap per battle (0: none): the headless simulator's budget.
    pub max_rollouts_per_battle: u64,
    /// Logged with every line: `app`, `selfplay`, `lab`, `scenario`.
    pub source: String,
    /// Round 2 (a): full-game rollouts per confirmed candidate (0: off, the arena decides).
    pub confirm: u32,
    /// Turns each full-game rollout plays.
    pub confirm_turns: u32,
    /// Candidates confirmed in the full game, Hard's own orders included.
    pub top_k: u32,
    /// Round 2 (c): search a continuation stance and keep a winning plan across turns.
    pub persist: bool,
    /// Round 2 (b): leave battles with aircraft near them to Hard (the arena has no aircraft).
    pub skip_air: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            rollouts: 64,
            screen: 16,
            horizon: 20,
            max_passes: 3,
            max_battles: 3,
            cap_ms: 15_000,
            max_rollouts_per_battle: 0,
            source: "app".into(),
            confirm: 0,
            confirm_turns: 6,
            top_k: 3,
            persist: false,
            skip_air: false,
        }
    }
}

impl TrainingConfig {
    /// These settings with `confirm` full-game rollouts per candidate.
    pub fn with_confirm(self, confirm: u32) -> Self {
        Self { confirm, ..self }
    }

    /// Round 2's fidelity settings (`docs/sim-results/training/round2-1108/`): the arena
    /// screens, the full game confirms, and a winning stance is kept across turns.
    pub fn round2(self) -> Self {
        Self {
            confirm: 16,
            confirm_turns: 6,
            top_k: 3,
            persist: true,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleKind {
    /// The seat attacks an enemy city.
    Attack,
    /// An enemy army is at a seat city.
    Defend,
    /// Armies in reach of each other away from a city.
    Field,
}

/// One battle a seat fights this turn, from the true state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battle {
    pub seat: PlayerId,
    pub enemy: PlayerId,
    pub kind: BattleKind,
    /// The centre: the city, or the enemy unit of a field battle.
    pub at: Hex,
    pub city: Option<CityId>,
    /// The city's name, or "near <city>" for a field battle.
    pub name: String,
    /// The seat's searched units.
    pub units: Vec<UnitId>,
}

fn fighting(kind: UnitKind) -> bool {
    is_military(kind) && kind != UnitKind::Scout && !data::is_aircraft(kind)
}

fn enemy_of(state: &GameState, seat: PlayerId, p: PlayerId) -> bool {
    p != seat && !p.is_barbarian() && state.player(p).alive && state.at_war(seat, p)
}

/// The battles `seat` fights this turn, most engaged first, at most `max`. Each seat unit is
/// searched in at most one battle (the nearest centre).
pub fn find_battles(state: &GameState, seat: PlayerId, max: usize) -> Vec<Battle> {
    find_battles_where(state, seat, max, &|_: &Battle| true)
}

/// As `find_battles`, among the battles `keep` accepts.
pub fn find_battles_where(
    state: &GameState,
    seat: PlayerId,
    max: usize,
    keep: &dyn Fn(&Battle) -> bool,
) -> Vec<Battle> {
    if max == 0 || seat.is_barbarian() || !state.player(seat).alive {
        return Vec::new();
    }
    let mine: Vec<_> = state
        .units
        .values()
        .filter(|u| u.owner == seat && fighting(u.kind))
        .collect();
    if mine.is_empty() {
        return Vec::new();
    }
    let foes: Vec<_> = state
        .units
        .values()
        .filter(|u| enemy_of(state, seat, u.owner) && fighting(u.kind))
        .collect();
    let engaged = |at: Hex| {
        mine.iter().filter(|u| u.pos.distance(at) <= REACH).count()
            + foes.iter().filter(|u| u.pos.distance(at) <= REACH).count()
    };
    // (priority, battle) candidates.
    let mut found: Vec<(usize, Battle)> = Vec::new();
    for c in state.cities.values() {
        if enemy_of(state, seat, c.owner) && mine.iter().any(|u| u.pos.distance(c.pos) <= ENGAGE) {
            found.push((
                engaged(c.pos) * 4 + 1,
                Battle {
                    seat,
                    enemy: c.owner,
                    kind: BattleKind::Attack,
                    at: c.pos,
                    city: Some(c.id),
                    name: c.name.clone(),
                    units: Vec::new(),
                },
            ));
        } else if c.owner == seat {
            let near: Vec<_> = foes
                .iter()
                .filter(|u| u.pos.distance(c.pos) <= ENGAGE)
                .collect();
            if let Some(enemy) = most_common(near.iter().map(|u| u.owner)) {
                found.push((
                    engaged(c.pos) * 4 + 2,
                    Battle {
                        seat,
                        enemy,
                        kind: BattleKind::Defend,
                        at: c.pos,
                        city: Some(c.id),
                        name: c.name.clone(),
                        units: Vec::new(),
                    },
                ));
            }
        }
    }
    for f in &foes {
        if !mine.iter().any(|u| u.pos.distance(f.pos) <= ENGAGE)
            || found.iter().any(|(_, b)| b.at.distance(f.pos) <= REACH)
        {
            continue;
        }
        let name = state
            .cities
            .values()
            .filter(|c| c.pos.distance(f.pos) <= 6)
            .min_by_key(|c| (c.pos.distance(f.pos), c.id))
            .map_or_else(|| "open field".to_string(), |c| format!("near {}", c.name));
        found.push((
            engaged(f.pos) * 4,
            Battle {
                seat,
                enemy: f.owner,
                kind: BattleKind::Field,
                at: f.pos,
                city: None,
                name,
                units: Vec::new(),
            },
        ));
    }
    found.retain(|(_, b)| keep(b));
    found.sort_by_key(|(p, b)| (std::cmp::Reverse(*p), b.at));
    found.truncate(max);
    let mut battles: Vec<Battle> = found.into_iter().map(|(_, b)| b).collect();
    for u in &mine {
        if let Some(b) = battles
            .iter_mut()
            .filter(|b| b.at.distance(u.pos) <= REACH)
            .min_by_key(|b| b.at.distance(u.pos))
        {
            b.units.push(u.id);
        }
    }
    battles.retain(|b| !b.units.is_empty());
    battles
}

fn most_common(owners: impl Iterator<Item = PlayerId>) -> Option<PlayerId> {
    let mut counts: BTreeMap<PlayerId, usize> = BTreeMap::new();
    for o in owners {
        *counts.entry(o).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(id, n)| (*n, std::cmp::Reverse(*id)))
        .map(|(id, _)| id)
}

/// The battle cut out of the true state. Arena hexes map to real ones by an offset of whole
/// columns and an even number of rows, which keeps the odd-r layout (and every adjacency).
#[derive(Clone, Debug)]
pub struct Arena {
    pub state: GameState,
    c0: i32,
    r0: i32,
}

impl Arena {
    pub fn to_arena(&self, h: Hex) -> Option<Hex> {
        let (c, r) = h.to_offset();
        let at = Hex::from_offset(c - self.c0, r - self.r0);
        self.state.map.contains(at).then_some(at)
    }

    pub fn to_real(&self, h: Hex) -> Hex {
        let (c, r) = h.to_offset();
        Hex::from_offset(c + self.c0, r + self.r0)
    }

    /// Cut `battle` out of `real`: the window's terrain, the two sides' units and cities there.
    pub fn cut(real: &GameState, battle: &Battle) -> Arena {
        let (cc, cr) = battle.at.to_offset();
        let span = 2 * WINDOW + 1;
        let w = span.min(real.map.width);
        let c0 = (cc - WINDOW).clamp(0, real.map.width - w);
        let mut r0 = (cr - WINDOW).clamp(0, (real.map.height - span).max(0));
        r0 -= r0 & 1;
        let h = (span + 1).min(real.map.height - r0);
        let sides = [battle.seat, battle.enemy];
        let inside = |p: Hex| {
            let (c, r) = p.to_offset();
            c >= c0 && c < c0 + w && r >= r0 && r < r0 + h
        };
        let shift = |p: Hex| {
            let (c, r) = p.to_offset();
            Hex::from_offset(c - c0, r - r0)
        };
        let cities: Vec<_> = real
            .cities
            .values()
            .filter(|c| sides.contains(&c.owner) && inside(c.pos))
            .collect();
        let kept: BTreeSet<CityId> = cities.iter().map(|c| c.id).collect();
        let mut map = Map::new(w, h, Terrain::Plains);
        for r in 0..h {
            for c in 0..w {
                let mut tile = real
                    .map
                    .get(Hex::from_offset(c + c0, r + r0))
                    .cloned()
                    .expect("window inside the map");
                tile.city = tile.city.filter(|id| kept.contains(id));
                if tile.city.is_none() && !tile.owner.is_some_and(|o| sides.contains(&o)) {
                    tile.owner = None;
                }
                map.tiles[(r * w + c) as usize] = tile;
            }
        }
        map.roaded = map.tiles.iter().any(|t| t.road != 0);
        let mut setup = real.setup.clone();
        setup.legacy_all_war = true;
        setup.barbarians = false;
        setup.width = w;
        setup.height = h;
        let mut s = blank_game(&setup, map);
        let fresh = std::mem::replace(&mut s.players, real.players.clone());
        let tiles = s.map.len();
        for (p, f) in s.players.iter_mut().zip(fresh) {
            p.explored = f.explored;
            for i in 0..tiles {
                p.explored.set(i);
            }
            p.tile_memory = f.tile_memory;
            p.observed_threats.clear();
            p.failed_city_sites = Default::default();
        }
        for city in cities {
            let mut city = city.clone();
            city.pos = shift(city.pos);
            city.grown_tiles = city
                .grown_tiles
                .iter()
                .filter(|t| inside(**t))
                .map(|t| shift(*t))
                .collect();
            s.cities.insert(city.id, city);
        }
        let units: BTreeSet<UnitId> = real
            .units
            .values()
            .filter(|u| sides.contains(&u.owner) && inside(u.pos) && !data::is_aircraft(u.kind))
            .map(|u| u.id)
            .collect();
        for id in &units {
            let mut u = real.units[id].clone();
            u.pos = shift(u.pos);
            u.goto = u.goto.filter(|g| inside(*g)).map(shift);
            if u.path.iter().all(|p| inside(*p)) {
                u.path = u.path.iter().map(|p| shift(*p)).collect();
            } else {
                u.path.clear();
                u.path_stop_on_block = false;
            }
            u.automation = None;
            u.attack_target = u.attack_target.filter(|t| units.contains(t));
            s.units.insert(u.id, u);
        }
        s.next_unit_id = real.next_unit_id;
        s.next_city_id = real.next_city_id;
        s.turn = real.turn;
        Arena { state: s, c0, r0 }
    }

    /// A real order in arena hexes; None for orders the arena can't play (a target outside it,
    /// automation, anything that is not a unit's move or fight).
    fn order_in(&self, o: &Order) -> Option<Order> {
        Some(match o {
            // A move out of the window heads for the passable arena tile nearest its real
            // destination, so Hard's "leave the fight" stays a move away rather than a hold.
            Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                if self.to_arena(*to).is_none() =>
            {
                Order::MoveUnit {
                    unit: *unit,
                    to: self.toward(*to)?,
                }
            }
            Order::MoveUnit { unit, to } => Order::MoveUnit {
                unit: *unit,
                to: self.to_arena(*to)?,
            },
            Order::MoveUnitPath {
                unit,
                to,
                path,
                stop_on_block,
            } => match path
                .iter()
                .map(|h| self.to_arena(*h))
                .collect::<Option<_>>()
            {
                Some(path) => Order::MoveUnitPath {
                    unit: *unit,
                    to: self.to_arena(*to)?,
                    path,
                    stop_on_block: *stop_on_block,
                },
                None => Order::MoveUnit {
                    unit: *unit,
                    to: self.to_arena(*to)?,
                },
            },
            Order::RangedAttack { unit, target } => Order::RangedAttack {
                unit: *unit,
                target: self.to_arena(*target)?,
            },
            Order::SleepUnit { .. } | Order::SkipUnit { .. } | Order::UpgradeUnit { .. } => {
                o.clone()
            }
            _ => return None,
        })
        .filter(|o| {
            legatum_core::rules::units::order_unit(o)
                .is_some_and(|u| self.state.units.contains_key(&u))
        })
    }

    /// The passable, empty arena tile nearest a real hex outside the window.
    fn toward(&self, real: Hex) -> Option<Hex> {
        self.state
            .map
            .hexes()
            .filter(|h| self.state.map.move_cost(*h).is_some())
            .filter(|h| !self.state.units.values().any(|u| u.pos == *h))
            .min_by_key(|h| (self.to_real(*h).distance(real), *h))
    }

    /// A searched order (move, shoot, hold, rest) back in real hexes.
    fn order_out(&self, o: &Order) -> Order {
        match o {
            Order::MoveUnit { unit, to } => Order::MoveUnit {
                unit: *unit,
                to: self.to_real(*to),
            },
            Order::RangedAttack { unit, target } => Order::RangedAttack {
                unit: *unit,
                target: self.to_real(*target),
            },
            other => other.clone(),
        }
    }
}

/// How the enemy plays in a rollout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Foe {
    /// Hard's own tactical planner.
    Hard,
    /// Every unit strikes the weakest enemy it can reach; a garrison stays in its city.
    Sortie,
}

fn sortie_orders(view: &PlayerView) -> Vec<Order> {
    let side = view.me;
    let foes: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.military && u.owner != side && view.at_war(side, u.owner))
        .collect();
    let mut out = Vec::new();
    for u in view
        .units
        .iter()
        .filter(|u| u.owner == side && u.military && fighting(u.kind))
    {
        if u.ranged > 0 {
            if let Some(t) = foes
                .iter()
                .filter(|e| e.pos.distance(u.pos) <= u.range)
                .filter(|e| forecast_attack_mode(view, u.id, e.pos, AttackMode::Ranged).is_some())
                .min_by_key(|e| (e.hp, e.id))
            {
                out.push(Order::RangedAttack {
                    unit: u.id,
                    target: t.pos,
                });
            }
            continue;
        }
        if view
            .cities
            .iter()
            .any(|c| c.pos == u.pos && c.owner == side)
        {
            continue;
        }
        if let Some(t) = foes
            .iter()
            .filter(|e| e.pos.distance(u.pos) <= 2)
            .min_by_key(|e| (e.hp, e.pos.distance(u.pos), e.id))
        {
            out.push(Order::MoveUnit {
                unit: u.id,
                to: t.pos,
            });
        }
    }
    out
}

/// How the seat's battle units play after the searched first turn (round 2 (c), the stance
/// mixture of `learning-loop.md` §5). Each overrides Hard's order only where it has something
/// to say for a unit; elsewhere Hard's order stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    /// Hard's own orders.
    #[default]
    Hard,
    /// Every unit hits the city: shooters shoot it, melee charges it.
    Storm,
    /// Shooters shoot the city (its walls); melee holds within 2 tiles until the city's HP is
    /// 0, then storms. The designer's siege.
    Siege,
    /// Every unit strikes the weakest enemy unit it can reach.
    Focus,
    /// Nobody moves except to attack.
    Hold,
}

impl Stance {
    /// The stances a battle of this kind may continue with.
    fn options(kind: BattleKind) -> &'static [Stance] {
        match kind {
            BattleKind::Attack => &[Stance::Hard, Stance::Storm, Stance::Siege, Stance::Focus],
            BattleKind::Defend | BattleKind::Field => &[Stance::Hard, Stance::Focus, Stance::Hold],
        }
    }
}

/// A searched plan kept on the seat for its next turn (round 2 (c)): the battle and the
/// stance its rollouts continued with. Next turn the search continues it unless the full-game
/// check says to abandon it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPlan {
    pub seat: PlayerId,
    pub kind: BattleKind,
    pub city: Option<CityId>,
    pub at: Hex,
    pub stance: Stance,
    /// The turn the plan was first played.
    pub since: u32,
}

impl StoredPlan {
    fn matches(&self, b: &Battle) -> bool {
        self.seat == b.seat
            && self.kind == b.kind
            && match (self.city, b.city) {
                (Some(a), Some(c)) => a == c,
                (None, None) => self.at.distance(b.at) <= ENGAGE,
                _ => false,
            }
    }
}

/// The order of an attack: a shot, or a move onto an enemy unit or city.
fn is_attack(view: &PlayerView, o: &Order) -> bool {
    match o {
        Order::RangedAttack { .. } => true,
        Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => {
            view.units
                .iter()
                .any(|e| e.pos == *to && e.owner != view.me && view.at_war(view.me, e.owner))
                || view
                    .cities
                    .iter()
                    .any(|c| c.pos == *to && view.at_war(view.me, c.owner))
        }
        _ => false,
    }
}

/// The order's aircraft, for the air orders `order_unit` doesn't cover.
fn order_plane(o: &Order) -> Option<UnitId> {
    match o {
        Order::AirStrike { plane, .. } | Order::Rebase { plane, .. } => Some(*plane),
        _ => None,
    }
}

/// The designer's breach (8.9.27 Germany: Xochimilco and Chalco in one turn, Khanbaliq with
/// one Cavalry): every shooter and aircraft in range hits `city`, and when their expected
/// damage takes it to 0 HP, the cheapest melee unit that can reach it walks in the same turn
/// (ranged fire resolves before movement). `hard` with those units' orders replaced, or None
/// when the shots can't zero the city or nobody can walk in.
pub fn breach_orders(view: &PlayerView, hard: &[Order], city: CityId) -> Option<Vec<Order>> {
    let me = view.me;
    let c = view
        .cities
        .iter()
        .find(|c| c.id == city && view.at_war(me, c.owner))?;
    let faction = view.players.iter().find(|p| p.id == me)?.faction;
    let unit_of = legatum_core::rules::units::order_unit;
    let mut out = hard.to_vec();
    let mut damage = 0;
    let mut shooters = BTreeSet::new();
    if c.hp > 0 {
        for u in view
            .units
            .iter()
            .filter(|u| u.owner == me && u.military && u.ranged > 0 && fighting(u.kind))
        {
            if u.pos.distance(c.pos) > u.range {
                continue;
            }
            if let Some(f) = forecast_attack_mode(view, u.id, c.pos, AttackMode::Ranged) {
                damage += f.expected_damage_to_defender;
                shooters.insert(u.id);
                out.retain(|o| unit_of(o) != Some(u.id));
                out.push(Order::RangedAttack {
                    unit: u.id,
                    target: c.pos,
                });
            }
        }
        for a in view.aircraft.iter().filter(|a| a.owner == me) {
            let Some(base) = view.cities.iter().find(|b| b.id == a.city) else {
                continue;
            };
            let range = data::unit_stats(a.kind).range;
            if base.pos.distance(c.pos) > range {
                continue;
            }
            if let Some(f) = forecast_attack_mode(view, a.id, c.pos, AttackMode::Ranged) {
                damage += f.expected_damage_to_defender;
                out.retain(|o| order_plane(o) != Some(a.id));
                out.push(Order::AirStrike {
                    plane: a.id,
                    target: c.pos,
                });
            }
        }
        if damage < c.hp {
            return None;
        }
    }
    let walker = view
        .units
        .iter()
        .filter(|u| u.owner == me && u.military && fighting(u.kind) && !shooters.contains(&u.id))
        .filter(|u| u.ranged == 0 || crate::composition::charges(u.kind))
        .filter(|u| u.pos.distance(c.pos) <= data::unit_moves_for(faction, u.kind).max(1))
        .filter(|u| forecast_attack_mode(view, u.id, c.pos, AttackMode::Melee).is_some())
        .min_by_key(|u| {
            (
                data::unit_stats_for(faction, u.kind).cost,
                u.pos.distance(c.pos),
                u.id,
            )
        })?;
    out.retain(|o| unit_of(o) != Some(walker.id));
    out.push(Order::MoveUnit {
        unit: walker.id,
        to: c.pos,
    });
    Some(out)
}

/// `hard` (the seat's orders) with `stance` applied to the seat's fighting units within
/// `reach` of `centre`. `city`: the battle's city.
fn stance_orders(
    view: &PlayerView,
    stance: Stance,
    mut hard: Vec<Order>,
    centre: Hex,
    reach: i32,
    city: Option<CityId>,
) -> Vec<Order> {
    if stance == Stance::Hard {
        return hard;
    }
    let me = view.me;
    let unit_of = legatum_core::rules::units::order_unit;
    let city = city
        .and_then(|id| view.cities.iter().find(|c| c.id == id))
        .filter(|c| view.at_war(me, c.owner));
    let foes: Vec<_> = view
        .units
        .iter()
        .filter(|e| e.military && e.owner != me && view.at_war(me, e.owner))
        .collect();
    let Some(faction) = view.players.iter().find(|p| p.id == me).map(|p| p.faction) else {
        return hard;
    };
    // Storm and Siege breach when the shots this turn can zero the city: shooters and
    // aircraft hit it and the cheapest melee walks in (`breach_orders`). Their orders stand.
    let mut breached = BTreeSet::new();
    if matches!(stance, Stance::Storm | Stance::Siege)
        && let Some(c) = city
        && let Some(b) = breach_orders(view, &hard, c.id)
    {
        breached = b
            .iter()
            .filter(|o| !hard.contains(o))
            .filter_map(unit_of)
            .collect();
        hard = b;
    }
    for u in view.units.iter().filter(|u| {
        u.owner == me
            && u.military
            && fighting(u.kind)
            && u.pos.distance(centre) <= reach
            && !breached.contains(&u.id)
    }) {
        let moves = data::unit_moves_for(faction, u.kind).max(1);
        let charges = u.ranged == 0 || crate::composition::charges(u.kind);
        let shoot = |at: Hex| {
            (u.ranged > 0
                && at.distance(u.pos) <= u.range
                && forecast_attack_mode(view, u.id, at, AttackMode::Ranged).is_some())
            .then_some(Order::RangedAttack {
                unit: u.id,
                target: at,
            })
        };
        let charge = |at: Hex| {
            (charges
                && at.distance(u.pos) <= moves
                && forecast_attack_mode(view, u.id, at, AttackMode::Melee)
                    .is_some_and(|f| f.win_chance >= SUICIDE))
            .then_some(Order::MoveUnit { unit: u.id, to: at })
        };
        let attacking = hard
            .iter()
            .any(|o| unit_of(o) == Some(u.id) && is_attack(view, o));
        let pick = match stance {
            Stance::Hard => None,
            Stance::Storm => city.and_then(|c| shoot(c.pos).or_else(|| charge(c.pos))),
            Stance::Siege => city.and_then(|c| {
                if u.ranged > 0 {
                    shoot(c.pos)
                } else if c.hp == 0 {
                    charge(c.pos)
                } else if attacking {
                    None
                } else if u.pos.distance(c.pos) <= 2 {
                    Some(Order::SkipUnit { unit: u.id })
                } else {
                    None
                }
            }),
            Stance::Focus => foes
                .iter()
                .filter_map(|e| {
                    shoot(e.pos)
                        .or_else(|| charge(e.pos))
                        .map(|o| (e.hp, e.id, o))
                })
                .min_by_key(|(hp, id, _)| (*hp, *id))
                .map(|(_, _, o)| o),
            Stance::Hold => (!attacking).then_some(Order::SkipUnit { unit: u.id }),
        };
        if let Some(o) = pick {
            hard.retain(|x| unit_of(x) != Some(u.id));
            hard.push(o);
        }
    }
    hard
}

/// Every legal order for `unit` this turn, minus the plainly bad.
fn unit_options(arena: &GameState, view: &PlayerView, unit: UnitId, centre: Hex) -> Vec<Order> {
    let Some(u) = view.units.iter().find(|u| u.id == unit) else {
        return Vec::new();
    };
    let faction = arena.player(u.owner).faction;
    let mut out = vec![
        Order::SkipUnit { unit },
        Order::SleepUnit {
            unit,
            until_healed: u.hp < data::UNIT_HP,
        },
    ];
    let enemy_at = |h: Hex| {
        view.units
            .iter()
            .any(|e| e.pos == h && e.owner != u.owner && view.at_war(u.owner, e.owner))
            || view
                .cities
                .iter()
                .any(|c| c.pos == h && view.at_war(u.owner, c.owner))
    };
    let targets: BTreeSet<Hex> = view
        .units
        .iter()
        .filter(|e| e.owner != u.owner && view.at_war(u.owner, e.owner))
        .map(|e| e.pos)
        .chain(
            view.cities
                .iter()
                .filter(|c| view.at_war(u.owner, c.owner))
                .map(|c| c.pos),
        )
        .collect();
    if u.ranged > 0 {
        for t in &targets {
            if t.distance(u.pos) <= u.range
                && forecast_attack_mode(view, unit, *t, AttackMode::Ranged).is_some()
            {
                out.push(Order::RangedAttack { unit, target: *t });
            }
        }
    }
    let moves = data::unit_moves_for(faction, u.kind).max(1);
    let charges = u.ranged == 0 || crate::composition::charges(u.kind);
    let city_at = |h: Hex| {
        view.cities
            .iter()
            .any(|c| c.pos == h && view.at_war(u.owner, c.owner))
    };
    if charges {
        for t in &targets {
            // A charge on a city is never pruned: shots this turn may take it to 0 HP first,
            // and then the charge walks in (the designer's 8.9.27 captures).
            if t.distance(u.pos) <= moves
                && forecast_attack_mode(view, unit, *t, AttackMode::Melee)
                    .is_some_and(|f| f.win_chance >= SUICIDE || city_at(*t))
            {
                out.push(Order::MoveUnit { unit, to: *t });
            }
        }
    }
    // Tiles reachable this turn: a unit may always enter a tile while it has moves left.
    let occupied: BTreeSet<Hex> = view.units.iter().map(|o| o.pos).collect();
    let mut cost: BTreeMap<Hex, i32> = BTreeMap::from([(u.pos, 0)]);
    let mut frontier = vec![u.pos];
    while let Some(at) = frontier.pop() {
        let spent = cost[&at];
        if spent >= moves {
            continue;
        }
        for n in at.neighbors() {
            let Some(step) = arena.map.move_cost(n) else {
                continue;
            };
            if occupied.contains(&n) || enemy_at(n) {
                continue;
            }
            let next = spent + step;
            if cost.get(&n).is_none_or(|c| next < *c) {
                cost.insert(n, next);
                frontier.push(n);
            }
        }
    }
    let foes: Vec<Hex> = view
        .units
        .iter()
        .filter(|e| e.military && e.owner != u.owner && view.at_war(u.owner, e.owner))
        .map(|e| e.pos)
        .collect();
    let here = u.pos.distance(centre);
    let safety = |h: Hex| foes.iter().map(|f| f.distance(h)).min().unwrap_or(99);
    let retreat = cost
        .keys()
        .filter(|h| **h != u.pos && h.distance(centre) > here)
        .max_by_key(|h| (safety(**h), h.distance(centre), std::cmp::Reverse(**h)))
        .copied();
    for h in cost.keys() {
        if *h == u.pos {
            continue;
        }
        if h.distance(centre) > here && Some(*h) != retreat {
            continue;
        }
        out.push(Order::MoveUnit { unit, to: *h });
    }
    out
}

/// One rollout's outcome for the seat, by component, in Production.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parts {
    /// Capture (attack) or loss (defence) of the city, with the earliness bonus.
    pub city: i64,
    /// Production of the seat's units lost (dead units whole, wounds pro rata).
    pub own_lost: i64,
    /// Production of the enemy's units killed or wounded.
    pub enemy_lost: i64,
    /// City HP damage, signed for the seat (attack: dealt; defence: taken, negative).
    pub city_hp: i64,
    /// The turn the battle was decided (the horizon if it was not).
    pub turns: i64,
}

impl Parts {
    pub fn total(&self) -> i64 {
        self.city + self.enemy_lost - self.own_lost + self.city_hp
    }

    fn mean(xs: &[Parts]) -> Parts {
        let n = xs.len().max(1) as i64;
        let sum = |f: fn(&Parts) -> i64| xs.iter().map(f).sum::<i64>() / n;
        Parts {
            city: sum(|p| p.city),
            own_lost: sum(|p| p.own_lost),
            enemy_lost: sum(|p| p.enemy_lost),
            city_hp: sum(|p| p.city_hp),
            turns: sum(|p| p.turns),
        }
    }
}

/// A city's value in the score, in Production.
fn city_value(population: i32) -> i64 {
    400 + 80 * i64::from(population)
}

/// One rollout's result for the seat. In the full game (round 2 (a)) every map unit of the
/// two sides counts (aircraft, in hangars, don't), and so does every other city the seat gains
/// or loses.
fn score(
    start: &GameState,
    end: &GameState,
    b: &Battle,
    decided: Option<u32>,
    horizon: u32,
) -> Parts {
    let mut parts = Parts {
        turns: i64::from(decided.unwrap_or(horizon)),
        ..Parts::default()
    };
    for c in start.cities.values().filter(|c| Some(c.id) != b.city) {
        let before = c.owner == b.seat;
        let after = end.cities.get(&c.id).is_some_and(|e| e.owner == b.seat);
        if before != after {
            let v = city_value(c.population);
            parts.city += if after { v } else { -v };
        }
    }
    for u in start
        .units
        .values()
        .filter(|u| is_military(u.kind) && u.kind != UnitKind::Scout)
    {
        let cost = i64::from(data::unit_stats_for(start.player(u.owner).faction, u.kind).cost);
        let lost = end
            .units
            .get(&u.id)
            .filter(|e| e.owner == u.owner)
            .map_or(i64::from(u.hp), |e| i64::from(u.hp - e.hp));
        let value = cost * lost / i64::from(data::UNIT_HP);
        if u.owner == b.seat {
            parts.own_lost += value;
        } else if u.owner == b.enemy {
            parts.enemy_lost += value;
        }
    }
    if let Some(city) = b.city.and_then(|id| start.cities.get(&id)) {
        let value = city_value(city.population);
        let after = end.cities.get(&city.id);
        let owner = after.map(|c| c.owner);
        let early = i64::from(horizon - decided.unwrap_or(horizon).min(horizon)) * TURN_WEIGHT;
        let hp_lost = i64::from(city.hp - after.map_or(0, |c| c.hp)) * CITY_HP_WEIGHT;
        match b.kind {
            BattleKind::Attack if owner == Some(b.seat) || owner.is_none() => {
                parts.city += value + early
            }
            BattleKind::Attack => parts.city_hp = hp_lost,
            BattleKind::Defend if owner != Some(b.seat) => parts.city -= value + early,
            BattleKind::Defend => parts.city_hp = -hp_lost,
            BattleKind::Field => {}
        }
    }
    parts
}

fn army(s: &GameState, p: PlayerId) -> bool {
    s.units.values().any(|u| u.owner == p && fighting(u.kind))
}

/// Play the battle from `arena` with the seat's first-turn orders fixed to `first`; later
/// turns are Hard's with `stance` applied to the seat's units near `centre` (arena hexes).
#[allow(clippy::too_many_arguments)]
fn rollout(
    arena: &GameState,
    b: &Battle,
    first: &[Order],
    seed: u64,
    foe: Foe,
    horizon: u32,
    stance: Stance,
    centre: Hex,
) -> Parts {
    let mut s = arena.clone();
    s.dice_seed = Some(Rng::derive(seed, &[1]).next_u64());
    let brain = |p: PlayerId, k: u64| {
        ScriptedBrain::with_difficulty(
            s.player(p).faction,
            Rng::derive(seed, &[k]).next_u64(),
            p,
            AiDifficulty::Hard,
        )
    };
    let seat_brain = brain(b.seat, 2);
    let foe_brain = brain(b.enemy, 3);
    let (seat_target, foe_target) = match b.kind {
        BattleKind::Attack => (b.city, None),
        BattleKind::Defend => (None, b.city),
        BattleKind::Field => (None, None),
    };
    let mut decided = None;
    for t in 1..=horizon {
        let mut orders = BTreeMap::new();
        let seat_orders = if t == 1 {
            first.to_vec()
        } else {
            let view = legatum_core::player_view(&s, b.seat);
            let hard = seat_brain.tactical_orders(&view, seat_target);
            stance_orders(&view, stance, hard, centre, REACH + 1, b.city)
        };
        orders.insert(b.seat, seat_orders);
        let foe_view = legatum_core::player_view(&s, b.enemy);
        orders.insert(
            b.enemy,
            match foe {
                Foe::Hard => foe_brain.tactical_orders(&foe_view, foe_target),
                Foe::Sortie => sortie_orders(&foe_view),
            },
        );
        let (mut next, _) = resolve_tactical_turn(&s, &orders);
        // City healing, production and upkeep run as in the game, on the turn just resolved.
        next.turn -= 1;
        legatum_core::rules::cities::process_cities(&mut next, &mut Vec::new());
        next.turn += 1;
        s = next;
        let city_owner = b.city.map(|id| s.cities.get(&id).map(|c| c.owner));
        let over = match b.kind {
            BattleKind::Attack => city_owner != Some(Some(b.enemy)) || !army(&s, b.seat),
            BattleKind::Defend => city_owner != Some(Some(b.seat)) || !army(&s, b.enemy),
            BattleKind::Field => !army(&s, b.seat) || !army(&s, b.enemy),
        };
        if over {
            decided = Some(t);
            break;
        }
    }
    score(arena, &s, b, decided, horizon)
}

/// Round 2 (a): play the whole game from `real` for `turns` turns, aircraft, production and
/// purchases included. The two sides and every empire at war with either play Hard (fresh
/// brains, the same for every candidate of rollout `seed`); empires at peace with both give no
/// orders for these few turns (their cities still produce), which keeps the check affordable
/// in 8-empire games. The seat's first-turn orders are `first`; on later turns its units near
/// the battle continue with `stance`. Scored like the arena, over the whole map.
fn full_rollout(
    real: &GameState,
    b: &Battle,
    first: &[Order],
    stance: Stance,
    seed: u64,
    turns: u32,
) -> Parts {
    let mut s = real.clone();
    s.dice_seed = Some(Rng::derive(seed, &[1]).next_u64());
    let acts = |p: PlayerId| {
        p == b.seat
            || p == b.enemy
            || real.at_war(p, b.seat)
            || (real.player(b.enemy).alive && real.at_war(p, b.enemy))
    };
    let mut brains: Vec<ScriptedBrain> = s
        .players
        .iter()
        .filter(|p| !p.id.is_barbarian() && acts(p.id))
        .map(|p| {
            ScriptedBrain::with_difficulty(
                p.faction,
                Rng::derive(seed, &[2, u64::from(p.id.0)]).next_u64(),
                p.id,
                AiDifficulty::Hard,
            )
        })
        .collect();
    let mut decided = None;
    for t in 1..=turns.max(1) {
        let mut orders = BTreeMap::new();
        for brain in &mut brains {
            let p = brain.player;
            if !s.player(p).alive {
                continue;
            }
            if p == b.seat && t == 1 {
                orders.insert(p, first.to_vec());
                continue;
            }
            let view = legatum_core::player_view(&s, p);
            let mut o = crate::Brain::orders(brain, &view);
            if p == b.seat {
                o = stance_orders(&view, stance, o, b.at, REACH + 1, b.city);
            }
            orders.insert(p, o);
        }
        let (next, _) = legatum_core::resolve_turn(&s, &orders);
        s = next;
        let city_owner = b.city.map(|id| s.cities.get(&id).map(|c| c.owner));
        let over = match b.kind {
            BattleKind::Attack => city_owner != Some(Some(b.enemy)),
            BattleKind::Defend => city_owner != Some(Some(b.seat)),
            BattleKind::Field => false,
        };
        if over || s.finished {
            decided = Some(t);
            break;
        }
    }
    score(real, &s, b, decided, turns.max(1))
}

/// The restarts the descent may start from: Hard's orders, then the combat lab's policies.
pub const RESTARTS: [&str; 4] = ["hard", "storm", "focus", "wait"];

/// A unit of search work: one arena rollout of a trial, or full-game rollout `k` of every
/// confirmed candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Job {
    Arena(usize, u32),
    Full(u32),
}

/// Where the search of one battle is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// The restarts, at full rollouts.
    Baseline,
    /// The continuation stances on the best restart, at full rollouts (round 2 (c)).
    Stance,
    /// Coordinate descent on this unit.
    Unit(usize),
    /// The full-game confirmation (round 2 (a)).
    Confirm,
    Done,
}

/// The full-game confirmation's candidates and results.
struct Confirm {
    /// (plan, stance) per candidate; 0 is Hard's orders with Hard's continuation.
    cands: Vec<(Vec<usize>, Stance)>,
    /// The seat's whole first-turn orders where a candidate isn't a plan of the arena's
    /// options (the breach, which orders aircraft the arena doesn't have).
    firsts: Vec<Option<Vec<Order>>>,
    /// Per full-game rollout: each candidate's result.
    samples: Vec<Option<Vec<Parts>>>,
    queue: std::collections::VecDeque<u32>,
}

/// One candidate in the full-game check, real hexes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmCandidate {
    /// The searched units' first-turn orders that differ from Hard's.
    pub orders: Vec<Order>,
    pub stance: Stance,
    /// The breach (`breach_orders`): shots and aircraft zero the city, a melee unit walks in.
    #[serde(default)]
    pub breach: bool,
    /// Mean full-game score, and its paired gain over Hard's with the 95% half-width.
    pub score: i64,
    pub gain: i64,
    pub gain_ci: i64,
    /// Its mean arena score (None for Hard's own with Hard's continuation when not scored).
    pub arena: Option<i64>,
}

/// The full-game check of a battle (round 2 (a)).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmRecord {
    pub rollouts: u32,
    pub turns: u32,
    pub candidates: Vec<ConfirmCandidate>,
    /// The candidate played (0: Hard's).
    pub played: usize,
    /// `clear`: it beat Hard's with the interval clear of zero; `continued`: a kept plan not
    /// significantly worse than Hard's; `hard`: neither; `cap`: the time cap hit first.
    pub reason: String,
}

/// The coordinate-descent search of one battle.
struct BattleSearch {
    battle: Battle,
    arena: Arena,
    seed: u64,
    cfg: TrainingConfig,
    /// The battle's centre in arena hexes.
    centre: Hex,
    /// The seat's arena orders for its units that aren't searched (Hard's).
    fixed: Vec<Order>,
    /// Per searched unit: its options; option 0 is Hard's own orders (possibly none).
    options: Vec<Vec<Vec<Order>>>,
    choice: Vec<usize>,
    /// The continuation stance of the current plan.
    stance: Stance,
    /// The stance came from a plan kept from last turn.
    persisted: bool,
    /// Per-rollout results of the current plan, and of Hard's (empty until scored).
    current: Vec<Parts>,
    hard: Vec<Parts>,
    /// Which restart the descent started from (index into `RESTARTS`).
    restart: usize,
    /// The restarts' plans; the baseline stage scores each distinct one.
    starts: Vec<(usize, Vec<usize>)>,
    /// Per unit and option: the latest mean score and how many rollouts it rests on.
    profile: Vec<Vec<Option<(i64, u32)>>>,
    stage: Stage,
    pass: u32,
    improved: bool,
    /// Each trial's full plan (one option per searched unit) and continuation stance.
    trials: Vec<Vec<usize>>,
    trial_stance: Vec<Stance>,
    samples: Vec<Vec<Option<Parts>>>,
    alive: Vec<bool>,
    queue: std::collections::VecDeque<(usize, u32)>,
    /// Rollouts the current stage gives each live trial.
    stage_end: u32,
    /// Every plan scored at full rollouts: the full-game check takes the best of these.
    evaluated: Vec<(Vec<usize>, Stance, Vec<Parts>)>,
    /// Round 2 (a): the real state and the seat's Hard orders there.
    real: Option<std::sync::Arc<GameState>>,
    seat_hard: Vec<Order>,
    confirm: Option<Confirm>,
    confirm_record: Option<ConfirmRecord>,
    /// The seat's first-turn orders when the played candidate is the breach.
    played_first: Option<Vec<Order>>,
    /// Why the battle was left to Hard without a search (`air`).
    skipped: Option<String>,
    done: bool,
    complete: bool,
    rollouts: u64,
    candidates: u32,
    started_ms: Option<u64>,
    /// When the arena stages must stop (the full-game check runs after it), and the end.
    arena_deadline_ms: u64,
    deadline_ms: u64,
    ended_ms: u64,
    /// Enemy units in the arena the seat can't see in the real game.
    hidden_units: u32,
}

fn mean(xs: &[Parts]) -> i64 {
    if xs.is_empty() {
        return i64::MIN;
    }
    xs.iter().map(Parts::total).sum::<i64>() / xs.len() as i64
}

/// The option a lab policy gives a unit: storm charges or shoots the city, focus strikes the
/// weakest enemy, wait holds. Hard's own option where the policy has nothing to say.
fn policy_option(
    view: &PlayerView,
    unit: UnitId,
    opts: &[Vec<Order>],
    city: Option<Hex>,
    policy: &str,
) -> usize {
    let hp_at = |h: Hex| {
        view.units
            .iter()
            .filter(|e| e.pos == h && e.owner != view.me)
            .map(|e| e.hp)
            .min()
            .unwrap_or(i32::MAX)
    };
    let find = |f: &dyn Fn(&Order) -> Option<i64>| {
        opts.iter()
            .enumerate()
            .skip(1)
            .filter_map(|(i, o)| (o.len() == 1).then(|| f(&o[0]).map(|k| (k, i))).flatten())
            .min()
            .map(|(_, i)| i)
    };
    let pick = match policy {
        "storm" => find(&|o| match o {
            Order::MoveUnit { to, .. } | Order::RangedAttack { target: to, .. }
                if Some(*to) == city =>
            {
                Some(if matches!(o, Order::MoveUnit { .. }) {
                    0
                } else {
                    1
                })
            }
            _ => None,
        }),
        "focus" => find(&|o| match o {
            Order::RangedAttack { target, .. } => Some(i64::from(hp_at(*target))),
            Order::MoveUnit { to, .. } if hp_at(*to) < i32::MAX => Some(i64::from(hp_at(*to))),
            _ => None,
        }),
        "wait" => find(&|o| matches!(o, Order::SkipUnit { unit: u } if *u == unit).then_some(0)),
        _ => None,
    };
    pick.unwrap_or(0)
}

/// Aircraft of either side this close to a battle put it beyond the arena (round 2 (b)).
const AIR_REACH: i32 = 12;

impl BattleSearch {
    #[allow(clippy::too_many_arguments)]
    fn new(
        real: &GameState,
        battle: Battle,
        hard: &[Order],
        seed: u64,
        cfg: &TrainingConfig,
        visible: &BTreeSet<UnitId>,
        kept: Option<Stance>,
        shared: Option<std::sync::Arc<GameState>>,
    ) -> Self {
        let arena = Arena::cut(real, &battle);
        let hidden = arena
            .state
            .units
            .values()
            .filter(|u| u.owner == battle.enemy && !visible.contains(&u.id))
            .count() as u32;
        // Aircraft live in city hangars (`GameState::aircraft`), never on the map.
        let air = cfg.skip_air
            && real.aircraft.iter().any(|a| {
                real.cities.get(&a.city).is_some_and(|c| {
                    (c.owner == battle.seat || c.owner == battle.enemy)
                        && c.pos.distance(battle.at) <= AIR_REACH
                })
            });
        let mut s = Self::in_arena(arena, battle, hard, seed, cfg, hidden, kept);
        if cfg.confirm > 0 {
            s.real = shared;
        }
        if air {
            s.skipped = Some("air".into());
            s.stage = Stage::Done;
            s.done = true;
            s.queue.clear();
        }
        s
    }

    /// The search of a battle already cut into its arena (a live turn, or a logged battle
    /// searched again offline). `kept`: the stance of a plan kept from last turn.
    fn in_arena(
        arena: Arena,
        battle: Battle,
        hard: &[Order],
        seed: u64,
        cfg: &TrainingConfig,
        hidden_units: u32,
        kept: Option<Stance>,
    ) -> Self {
        let view = legatum_core::player_view(&arena.state, battle.seat);
        let centre = arena.to_arena(battle.at).unwrap_or(battle.at);
        let searched: BTreeSet<UnitId> = battle.units.iter().copied().collect();
        let in_arena: Vec<Order> = hard.iter().filter_map(|o| arena.order_in(o)).collect();
        let unit_of = legatum_core::rules::units::order_unit;
        let fixed = in_arena
            .iter()
            .filter(|o| unit_of(o).is_none_or(|u| !searched.contains(&u)))
            .cloned()
            .collect();
        let options = battle
            .units
            .iter()
            .map(|id| {
                let own: Vec<Order> = in_arena
                    .iter()
                    .filter(|o| unit_of(o) == Some(*id))
                    .cloned()
                    .collect();
                let mut opts = vec![own.clone()];
                for o in unit_options(&arena.state, &view, *id, centre) {
                    if own.len() != 1 || own[0] != o {
                        opts.push(vec![o]);
                    }
                }
                opts
            })
            .collect::<Vec<_>>();
        let city_at = battle
            .city
            .and_then(|c| arena.state.cities.get(&c))
            .map(|c| c.pos);
        let mut starts: Vec<(usize, Vec<usize>)> = Vec::new();
        for (r, name) in RESTARTS.iter().enumerate() {
            let plan: Vec<usize> = battle
                .units
                .iter()
                .zip(&options)
                .map(|(id, opts)| policy_option(&view, *id, opts, city_at, name))
                .collect();
            if !starts.iter().any(|(_, p)| *p == plan) {
                starts.push((r, plan));
            }
        }
        let n = battle.units.len();
        let profile = options.iter().map(|o| vec![None; o.len()]).collect();
        let stance = if cfg.persist {
            kept.unwrap_or_default()
        } else {
            Stance::Hard
        };
        let mut s = BattleSearch {
            battle,
            arena,
            seed,
            cfg: cfg.clone(),
            centre,
            fixed,
            options,
            choice: vec![0; n],
            stance,
            persisted: stance != Stance::Hard,
            current: Vec::new(),
            hard: Vec::new(),
            restart: 0,
            starts: starts.clone(),
            profile,
            stage: Stage::Baseline,
            pass: 0,
            improved: false,
            trials: Vec::new(),
            trial_stance: Vec::new(),
            samples: Vec::new(),
            alive: Vec::new(),
            queue: Default::default(),
            stage_end: 0,
            evaluated: Vec::new(),
            real: None,
            seat_hard: hard.to_vec(),
            confirm: None,
            confirm_record: None,
            played_first: None,
            skipped: None,
            done: false,
            complete: false,
            rollouts: 0,
            candidates: 0,
            started_ms: None,
            arena_deadline_ms: u64::MAX,
            deadline_ms: u64::MAX,
            ended_ms: 0,
            hidden_units,
        };
        let plans: Vec<Vec<usize>> = starts.into_iter().map(|(_, p)| p).collect();
        let stances = vec![stance; plans.len()];
        s.start_stage(plans, stances, cfg.rollouts.max(1));
        s
    }

    fn start_stage(&mut self, trials: Vec<Vec<usize>>, stances: Vec<Stance>, end: u32) {
        let n = trials.len();
        self.samples = vec![vec![None; self.cfg.rollouts.max(1) as usize]; n];
        self.alive = vec![true; n];
        self.trials = trials;
        self.trial_stance = stances;
        self.queue.clear();
        self.stage_end = 0;
        self.extend_stage(end);
    }

    fn extend_stage(&mut self, end: u32) {
        for k in self.stage_end..end {
            for t in 0..self.trials.len() {
                if self.alive[t] {
                    self.queue.push_back((t, k));
                }
            }
        }
        self.stage_end = end;
    }

    /// The arena stages (the restarts, the stances, the descent) are running.
    fn in_arena_stage(&self) -> bool {
        matches!(self.stage, Stage::Baseline | Stage::Stance | Stage::Unit(_))
    }

    /// The seat's first-turn arena orders for a plan.
    fn joint(&self, plan: &[usize]) -> Vec<Order> {
        let mut out = self.fixed.clone();
        for (opts, pick) in self.options.iter().zip(plan) {
            out.extend(opts[*pick].iter().cloned());
        }
        out
    }

    /// The seat's first-turn orders for a plan in the real game: Hard's, with the searched
    /// units that the plan changes replaced.
    fn seat_first(&self, plan: &[usize]) -> Vec<Order> {
        let unit_of = legatum_core::rules::units::order_unit;
        let ids: BTreeSet<UnitId> = self
            .battle
            .units
            .iter()
            .zip(plan)
            .filter(|(_, c)| **c != 0)
            .map(|(id, _)| *id)
            .collect();
        let mut out: Vec<Order> = self
            .seat_hard
            .iter()
            .filter(|o| unit_of(o).is_none_or(|u| !ids.contains(&u)))
            .cloned()
            .collect();
        for (u, c) in plan.iter().enumerate() {
            if *c != 0 {
                out.extend(self.options[u][*c].iter().map(|o| self.arena.order_out(o)));
            }
        }
        out
    }

    fn eval(&self, job: Job) -> Vec<Parts> {
        match job {
            Job::Arena(trial, k) => {
                // Common random numbers: rollout k has the same dice and defender for every
                // candidate.
                let seed = Rng::derive(self.seed, &[u64::from(k)]).next_u64();
                vec![rollout(
                    &self.arena.state,
                    &self.battle,
                    &self.joint(&self.trials[trial]),
                    seed,
                    foe_of(k),
                    self.cfg.horizon.max(1),
                    self.trial_stance[trial],
                    self.centre,
                )]
            }
            Job::Full(k) => {
                let (Some(real), Some(c)) = (self.real.as_deref(), self.confirm.as_ref()) else {
                    return Vec::new();
                };
                let seed = Rng::derive(self.seed, &[0xf011, u64::from(k)]).next_u64();
                c.cands
                    .iter()
                    .zip(&c.firsts)
                    .map(|((plan, stance), first)| {
                        let first = first.clone().unwrap_or_else(|| self.seat_first(plan));
                        full_rollout(
                            real,
                            &self.battle,
                            &first,
                            *stance,
                            seed,
                            self.cfg.confirm_turns,
                        )
                    })
                    .collect()
            }
        }
    }

    fn take(&mut self, n: usize) -> Vec<Job> {
        if let Some(c) = self
            .confirm
            .as_mut()
            .filter(|_| self.stage == Stage::Confirm)
        {
            // A full-game rollout costs about a hundred arena rollouts.
            let n = (n / 64).max(1).min(c.queue.len());
            return c.queue.drain(..n).map(Job::Full).collect();
        }
        let n = n.min(self.queue.len());
        self.queue
            .drain(..n)
            .map(|(t, k)| Job::Arena(t, k))
            .collect()
    }

    fn submit(&mut self, results: Vec<(Job, Vec<Parts>)>) {
        for (job, r) in results {
            match job {
                Job::Arena(t, k) => {
                    if let Some(p) = r.first() {
                        self.samples[t][k as usize] = Some(*p);
                    }
                    self.rollouts += 1;
                }
                Job::Full(k) => {
                    if let Some(c) = self.confirm.as_mut() {
                        self.rollouts += r.len() as u64;
                        c.samples[k as usize] = Some(r);
                    }
                }
            }
        }
        match self.stage {
            Stage::Confirm => {
                if self
                    .confirm
                    .as_ref()
                    .is_some_and(|c| c.queue.is_empty() && c.samples.iter().all(Option::is_some))
                {
                    self.decide();
                }
            }
            Stage::Done => {}
            _ => {
                if self.queue.is_empty() {
                    self.advance();
                }
            }
        }
    }

    fn trial_samples(&self, t: usize, upto: u32) -> Vec<Parts> {
        self.samples[t][..upto as usize]
            .iter()
            .flatten()
            .copied()
            .collect()
    }

    /// Keep every live trial scored on all `upto` rollouts for the full-game check.
    fn note_evaluated(&mut self, upto: u32) {
        for t in 0..self.trials.len() {
            if !self.alive[t] {
                continue;
            }
            let xs = self.trial_samples(t, upto);
            if xs.len() < upto as usize {
                continue;
            }
            let (plan, stance) = (&self.trials[t], self.trial_stance[t]);
            if !self
                .evaluated
                .iter()
                .any(|(p, s, _)| p == plan && *s == stance)
            {
                self.evaluated.push((plan.clone(), stance, xs));
            }
        }
    }

    /// Record each trial's latest score in the per-unit profile.
    fn note_profile(&mut self, u: usize, upto: u32) {
        for t in 0..self.trials.len() {
            let xs = self.trial_samples(t, upto);
            if !xs.is_empty() {
                let opt = self.trials[t][u];
                self.profile[u][opt] = Some((mean(&xs), xs.len() as u32));
            }
        }
        let chosen = self.choice[u];
        self.profile[u][chosen] = Some((mean(&self.current), self.current.len() as u32));
    }

    /// The stage is scored: decide, then queue the next one.
    fn advance(&mut self) {
        let full = self.cfg.rollouts.max(1);
        let screen = self.cfg.screen.clamp(1, full);
        match self.stage {
            Stage::Baseline => {
                self.hard = self.trial_samples(0, full);
                self.candidates += self.trials.len() as u32;
                self.note_evaluated(full);
                // The best restart (ties keep Hard's, the first).
                let best = (0..self.trials.len())
                    .max_by_key(|t| (mean(&self.trial_samples(*t, full)), std::cmp::Reverse(*t)))
                    .unwrap_or(0);
                self.restart = self.starts[best].0;
                self.choice = self.trials[best].clone();
                self.current = self.trial_samples(best, full);
                self.start_stances();
            }
            Stage::Stance => {
                self.candidates += self.trials.len() as u32;
                self.note_evaluated(full);
                let bar = mean(&self.current);
                let best = (0..self.trials.len())
                    .map(|t| (mean(&self.trial_samples(t, full)), t))
                    .max_by_key(|(m, t)| (*m, std::cmp::Reverse(*t)));
                if let Some((m, t)) = best
                    && m > bar
                {
                    self.stance = self.trial_stance[t];
                    self.current = self.trial_samples(t, full);
                }
                self.next_unit(0);
            }
            Stage::Unit(u) if self.stage_end < full => {
                self.note_profile(u, screen);
                // Screening done: keep the trials that beat the current plan on the same dice.
                let bar = mean(&self.current[..screen as usize]);
                let mut ranked: Vec<(i64, usize)> = (0..self.trials.len())
                    .map(|t| (mean(&self.trial_samples(t, screen)), t))
                    .filter(|(m, _)| *m > bar)
                    .collect();
                ranked.sort_by_key(|(m, t)| (std::cmp::Reverse(*m), *t));
                ranked.truncate((self.trials.len() / 4).max(2));
                let keep: BTreeSet<usize> = ranked.iter().map(|(_, t)| *t).collect();
                for t in 0..self.trials.len() {
                    self.alive[t] = keep.contains(&t);
                }
                if keep.is_empty() {
                    self.next_unit(u + 1);
                } else {
                    self.extend_stage(full);
                }
            }
            Stage::Unit(u) => {
                self.note_profile(u, full);
                self.note_evaluated(full);
                let bar = mean(&self.current);
                let best = (0..self.trials.len())
                    .filter(|t| self.alive[*t])
                    .map(|t| (mean(&self.trial_samples(t, full)), t))
                    .max_by_key(|(m, t)| (*m, std::cmp::Reverse(*t)));
                if let Some((m, t)) = best
                    && m > bar
                {
                    self.choice[u] = self.trials[t][u];
                    self.current = self.trial_samples(t, full);
                    self.improved = true;
                    self.note_profile(u, full);
                }
                self.next_unit(u + 1);
            }
            Stage::Confirm | Stage::Done => {}
        }
    }

    /// Round 2 (c): score the best restart under every other continuation stance.
    fn start_stances(&mut self) {
        if !self.cfg.persist {
            self.next_unit(0);
            return;
        }
        let stances: Vec<Stance> = Stance::options(self.battle.kind)
            .iter()
            .copied()
            .filter(|s| *s != self.stance)
            .collect();
        if stances.is_empty() {
            self.next_unit(0);
            return;
        }
        let plans = vec![self.choice.clone(); stances.len()];
        self.stage = Stage::Stance;
        self.start_stage(plans, stances, self.cfg.rollouts.max(1));
    }

    /// Queue the screening stage of the next unit with options, or end the pass.
    fn next_unit(&mut self, mut u: usize) {
        loop {
            while u < self.options.len() && self.options[u].len() <= 1 {
                u += 1;
            }
            if u < self.options.len() {
                let trials: Vec<Vec<usize>> = (0..self.options[u].len())
                    .filter(|o| *o != self.choice[u])
                    .map(|o| {
                        let mut plan = self.choice.clone();
                        plan[u] = o;
                        plan
                    })
                    .collect();
                self.candidates += trials.len() as u32;
                self.stage = Stage::Unit(u);
                let screen = self.cfg.screen.clamp(1, self.cfg.rollouts.max(1));
                let stances = vec![self.stance; trials.len()];
                self.start_stage(trials, stances, screen);
                return;
            }
            self.pass += 1;
            if self.improved && self.pass < self.cfg.max_passes.max(1) {
                self.improved = false;
                u = 0;
                continue;
            }
            self.end_arena(true);
            return;
        }
    }

    /// The arena stages are over (converged, or out of budget): confirm in the full game, or
    /// stop with the arena's best.
    fn end_arena(&mut self, complete: bool) {
        self.complete = complete;
        self.queue.clear();
        if self.cfg.confirm > 0 && self.real.is_some() {
            self.start_confirm();
        } else {
            self.stage = Stage::Done;
            self.done = true;
        }
    }

    /// Round 2 (a): Hard's orders, the search's plan and the next best plans that beat Hard's
    /// in the arena, re-scored in the full game on the same rollouts.
    fn start_confirm(&mut self) {
        let hard = (vec![0; self.options.len()], Stance::Hard);
        let mut cands = vec![hard.clone()];
        let hard_bar = self
            .evaluated
            .iter()
            .find(|(p, s, _)| *p == hard.0 && *s == hard.1)
            .map_or(mean(&self.hard), |(_, _, xs)| mean(xs));
        if (self.choice.clone(), self.stance) != hard {
            cands.push((self.choice.clone(), self.stance));
        }
        let mut rest: Vec<&(Vec<usize>, Stance, Vec<Parts>)> = self
            .evaluated
            .iter()
            .filter(|(_, _, xs)| mean(xs) > hard_bar)
            .collect();
        rest.sort_by_key(|(_, s, xs)| (std::cmp::Reverse(mean(xs)), *s));
        for (p, s, _) in rest {
            if cands.len() >= self.cfg.top_k.max(2) as usize {
                break;
            }
            let c = (p.clone(), *s);
            if !cands.contains(&c) {
                cands.push(c);
            }
        }
        let mut firsts = vec![None; cands.len()];
        // The breach, from the real game (its aircraft included): always checked when the
        // shots can zero the city this turn.
        if let (Some(real), Some(city), BattleKind::Attack) =
            (self.real.as_deref(), self.battle.city, self.battle.kind)
        {
            let view = legatum_core::player_view(real, self.battle.seat);
            if let Some(b) = breach_orders(&view, &self.seat_hard, city) {
                cands.push((hard.0.clone(), Stance::Storm));
                firsts.push(Some(b));
            }
        }
        if cands.len() == 1 {
            // Nothing beats Hard's orders in the arena: no check needed.
            self.choice = hard.0;
            self.stance = Stance::Hard;
            self.stage = Stage::Done;
            self.done = true;
            return;
        }
        let n = self.cfg.confirm;
        self.confirm = Some(Confirm {
            cands,
            firsts,
            samples: vec![None; n as usize],
            queue: (0..n).collect(),
        });
        self.stage = Stage::Confirm;
    }

    /// Play a candidate only if it beats Hard's orders in the full game with its interval
    /// clear of zero; a kept plan continues unless Hard's beats it the same way.
    fn decide(&mut self) {
        let Some(c) = self.confirm.as_ref() else {
            return;
        };
        let per: Vec<Vec<Parts>> = (0..c.cands.len())
            .map(|i| c.samples.iter().flatten().map(|v| v[i]).collect())
            .collect();
        let gains: Vec<(i64, i64)> = per.iter().map(|xs| paired_strict(xs, &per[0])).collect();
        let clear = (1..c.cands.len())
            .filter(|i| gains[*i].0 - gains[*i].1 > 0)
            .max_by_key(|i| (gains[*i].0, std::cmp::Reverse(*i)));
        let kept = c
            .cands
            .iter()
            .position(|x| x.0 == self.choice && x.1 == self.stance)
            .filter(|i| *i > 0 && self.persisted && gains[*i].0 + gains[*i].1 >= 0);
        let (played, reason) = match (clear, kept) {
            (Some(i), _) => (i, "clear"),
            (None, Some(i)) => (i, "continued"),
            _ => (0, "hard"),
        };
        let arena_of = |plan: &Vec<usize>, stance: Stance| {
            self.evaluated
                .iter()
                .find(|(p, s, _)| p == plan && *s == stance)
                .map(|(_, _, xs)| mean(xs))
        };
        let candidates = c
            .cands
            .iter()
            .zip(&c.firsts)
            .zip(&per)
            .zip(&gains)
            .map(
                |((((plan, stance), first), xs), (g, ci))| ConfirmCandidate {
                    orders: match first {
                        Some(f) => f
                            .iter()
                            .filter(|o| !self.seat_hard.contains(o))
                            .cloned()
                            .collect(),
                        None => plan
                            .iter()
                            .enumerate()
                            .filter(|(_, o)| **o != 0)
                            .flat_map(|(u, o)| {
                                self.options[u][*o].iter().map(|x| self.arena.order_out(x))
                            })
                            .collect(),
                    },
                    breach: first.is_some(),
                    stance: *stance,
                    score: mean(xs),
                    gain: *g,
                    gain_ci: *ci,
                    arena: arena_of(plan, *stance),
                },
            )
            .collect();
        let record = ConfirmRecord {
            rollouts: per[0].len() as u32,
            turns: self.cfg.confirm_turns,
            candidates,
            played,
            reason: reason.into(),
        };
        let (plan, stance) = c.cands[played].clone();
        self.played_first = c.firsts[played].clone();
        self.current = if played == 0 {
            self.hard.clone()
        } else {
            self.evaluated
                .iter()
                .find(|(p, s, _)| *p == plan && *s == stance)
                .map_or_else(|| self.current.clone(), |(_, _, xs)| xs.clone())
        };
        self.choice = plan;
        self.stance = stance;
        self.confirm_record = Some(record);
        self.stage = Stage::Done;
        self.done = true;
    }

    fn stop(&mut self, now_ms: u64) {
        if !self.done {
            if self.cfg.confirm > 0 && self.real.is_some() && self.skipped.is_none() {
                // The full-game check did not finish: Hard's orders.
                self.choice = vec![0; self.options.len()];
                self.stance = Stance::Hard;
                self.confirm_record = Some(ConfirmRecord {
                    rollouts: 0,
                    turns: self.cfg.confirm_turns,
                    candidates: Vec::new(),
                    played: 0,
                    reason: "cap".into(),
                });
            }
            self.done = true;
            self.stage = Stage::Done;
            self.queue.clear();
            if let Some(c) = self.confirm.as_mut() {
                c.queue.clear();
            }
        }
        self.ended_ms = self.ended_ms.max(now_ms);
    }

    /// The searched units' orders, real hexes: Hard's, or the search's (`searched`).
    fn orders(&self, hard: &[Order], searched: bool) -> Vec<Order> {
        let unit_of = legatum_core::rules::units::order_unit;
        let mut out = Vec::new();
        for (u, id) in self.battle.units.iter().enumerate() {
            if searched && self.choice[u] != 0 {
                out.extend(
                    self.options[u][self.choice[u]]
                        .iter()
                        .map(|o| self.arena.order_out(o)),
                );
            } else {
                out.extend(hard.iter().filter(|o| unit_of(o) == Some(*id)).cloned());
            }
        }
        out
    }

    /// The units and aircraft whose orders the breach changes.
    fn breach_changed(&self) -> BTreeSet<UnitId> {
        let unit_of = |o: &Order| legatum_core::rules::units::order_unit(o).or(order_plane(o));
        self.played_first.as_ref().map_or_else(BTreeSet::new, |f| {
            f.iter()
                .filter(|o| !self.seat_hard.contains(o))
                .filter_map(unit_of)
                .collect()
        })
    }

    fn changed(&self) -> usize {
        if self.played_first.is_some() {
            return self.breach_changed().len();
        }
        if self.current.is_empty() {
            return 0;
        }
        self.choice.iter().filter(|c| **c != 0).count()
    }
}

/// The defender policy of rollout `k`: even rollouts Hard, odd ones the sortie.
fn foe_of(k: u32) -> Foe {
    if k.is_multiple_of(2) {
        Foe::Hard
    } else {
        Foe::Sortie
    }
}

/// A unit in a logged battle, real hexes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitRecord {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub hp: i32,
    pub pos: Hex,
}

/// How much each of a unit's orders matters: every option's score with the others fixed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitProfile {
    pub unit: UnitId,
    pub kind: UnitKind,
    /// Index of Hard's option (always 0) and of the chosen one.
    pub chosen: usize,
    pub options: Vec<OptionScore>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionScore {
    pub orders: Vec<Order>,
    /// Mean score and the rollouts it rests on (None: never scored, the cap hit first).
    pub mean: Option<i64>,
    pub n: u32,
}

/// Scores of one plan against each defender policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByDefender {
    pub hard: i64,
    pub sortie: i64,
}

fn by_defender(xs: &[Parts]) -> ByDefender {
    let side = |even: bool| {
        let v: Vec<Parts> = xs
            .iter()
            .enumerate()
            .filter(|(k, _)| k.is_multiple_of(2) == even)
            .map(|(_, p)| *p)
            .collect();
        if v.is_empty() { 0 } else { mean(&v) }
    };
    ByDefender {
        hard: side(true),
        sortie: side(false),
    }
}

/// Paired difference (searched minus Hard) on the same dice: the mean and the half-width of
/// its 95% interval.
fn paired(best: &[Parts], hard: &[Parts]) -> (i64, i64) {
    let d: Vec<i64> = best
        .iter()
        .zip(hard)
        .map(|(b, h)| b.total() - h.total())
        .collect();
    if d.is_empty() {
        return (0, 0);
    }
    let n = d.len() as i64;
    let m = d.iter().sum::<i64>() / n;
    if n < 2 {
        return (m, 0);
    }
    let var = d.iter().map(|x| (x - m) * (x - m)).sum::<i64>() / (n - 1);
    let se = isqrt(var / n);
    (m, se * 196 / 100)
}

/// `paired`, with the half-width widened for few rollouts (Student's t, 95%, in hundredths):
/// the full-game check plays a candidate only when this interval clears zero.
fn paired_strict(best: &[Parts], hard: &[Parts]) -> (i64, i64) {
    let (m, ci) = paired(best, hard);
    let t = match best.len().min(hard.len()) {
        0..=4 => 318,
        5..=8 => 236,
        9..=16 => 213,
        17..=32 => 204,
        _ => 196,
    };
    (m, ci * t / 196)
}

fn isqrt(x: i64) -> i64 {
    if x <= 0 {
        return 0;
    }
    x.isqrt()
}

/// The log's schema version.
pub const LOG_SCHEMA: u32 = 1;

/// One searched decision: a line of the training log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub schema: u32,
    pub rules_version: u32,
    /// Where the line came from: `app`, `selfplay`, `lab`, `scenario`.
    pub source: String,
    /// Joins the decision to its realised outcome.
    pub engagement: String,
    pub turn: u32,
    pub seat: PlayerId,
    pub faction: legatum_core::Faction,
    pub enemy: PlayerId,
    pub kind: BattleKind,
    pub name: String,
    pub at: Hex,
    pub city: Option<CityId>,
    pub city_hp: Option<i32>,
    pub walls: bool,
    pub units: Vec<UnitRecord>,
    pub hard_orders: Vec<Order>,
    pub searched_orders: Vec<Order>,
    pub hard_score: i64,
    pub searched_score: i64,
    /// Searched minus Hard on the same dice, and the half-width of its 95% interval.
    pub gain: i64,
    pub gain_ci: i64,
    pub hard_parts: Parts,
    pub searched_parts: Parts,
    pub hard_by_defender: ByDefender,
    pub searched_by_defender: ByDefender,
    /// Mean turns the searched plan's rollouts played before the battle was decided.
    pub horizon_reached: i64,
    pub changed_units: u32,
    /// The restart the descent started from (`RESTARTS`).
    pub restart: String,
    /// Enemy units in the arena the seat can't see in the real game, and whether the searched
    /// plan changed anything with them there (a lesson Hard might not be able to use).
    pub hidden_units: u32,
    pub hidden_dependent: bool,
    pub profile: Vec<UnitProfile>,
    pub rollouts: u64,
    pub candidates: u32,
    pub passes: u32,
    /// The descent converged (false: the time or rollout cap stopped it).
    pub complete: bool,
    pub ms: u64,
    /// The seat's searched units (the battle's `units`).
    pub searched_units: Vec<UnitId>,
    /// The battle's arena, so the decision can be searched again offline (`retrain`).
    pub arena: ArenaSnapshot,
    /// Offline re-search only: which perturbation of the logged battle this is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Round 2: the continuation stance played, and whether it was a plan kept from last turn.
    #[serde(default)]
    pub stance: Stance,
    #[serde(default)]
    pub persisted: bool,
    /// Round 2 (a): the full-game check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmRecord>,
    /// Why the battle was left to Hard unsearched (`air`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
}

/// An arena, gzipped JSON in base64 (about the size of a few kilobytes), with its offset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArenaSnapshot {
    pub c0: i32,
    pub r0: i32,
    pub state: String,
}

impl Arena {
    pub fn snapshot(&self) -> ArenaSnapshot {
        use base64::Engine;
        use std::io::Write;
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gz.write_all(&serde_json::to_vec(&self.state).expect("arena state"))
            .expect("gzip");
        ArenaSnapshot {
            c0: self.c0,
            r0: self.r0,
            state: base64::engine::general_purpose::STANDARD.encode(gz.finish().expect("gzip")),
        }
    }

    pub fn restore(snap: &ArenaSnapshot) -> Result<Arena, String> {
        use base64::Engine;
        use std::io::Read;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&snap.state)
            .map_err(|e| e.to_string())?;
        let mut json = Vec::new();
        flate2::read::GzDecoder::new(&bytes[..])
            .read_to_end(&mut json)
            .map_err(|e| e.to_string())?;
        Ok(Arena {
            state: serde_json::from_slice(&json).map_err(|e| e.to_string())?,
            c0: snap.c0,
            r0: snap.r0,
        })
    }
}

/// What the app shows while the AI studies its battles.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleProgress {
    pub seat: PlayerId,
    pub name: String,
    pub at: Hex,
    pub kind: BattleKind,
    pub done: bool,
    /// The search changed Hard's orders here (known once done).
    pub changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingProgress {
    pub turn: u32,
    pub battles: Vec<BattleProgress>,
    /// Index of the battle being studied.
    pub current: Option<usize>,
    pub done: bool,
    pub rollouts: u64,
}

/// The finished turn: every training seat's orders, and the log lines.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingResult {
    pub turn: u32,
    pub orders: BTreeMap<PlayerId, Vec<Order>>,
    pub records: Vec<TrainingRecord>,
    pub progress: TrainingProgress,
    /// Round 2 (c): the plans to continue next turn.
    #[serde(default)]
    pub plans: Vec<StoredPlan>,
}

/// One turn's training search over every training seat's battles.
pub struct TrainingRun {
    turn: u32,
    cfg: TrainingConfig,
    hard: BTreeMap<PlayerId, Vec<Order>>,
    searches: Vec<BattleSearch>,
    real_units: BTreeMap<UnitId, UnitRecord>,
    cities: BTreeMap<CityId, (i32, bool)>,
    factions: BTreeMap<PlayerId, legatum_core::Faction>,
    game: u64,
    /// Per search: the turn its kept plan was first played.
    since: Vec<Option<u32>>,
}

impl TrainingRun {
    /// `hard`: each training seat's orders from Hard this turn.
    pub fn new(
        state: &GameState,
        hard: BTreeMap<PlayerId, Vec<Order>>,
        cfg: &TrainingConfig,
    ) -> Self {
        Self::with_plans(state, hard, cfg, &[], |_| true)
    }

    /// As `new`, continuing last turn's `plans` (round 2 (c)) and searching only the battles
    /// `keep` accepts (the operation executor's strike: its target city).
    pub fn with_plans(
        state: &GameState,
        hard: BTreeMap<PlayerId, Vec<Order>>,
        cfg: &TrainingConfig,
        plans: &[StoredPlan],
        keep: impl Fn(&Battle) -> bool,
    ) -> Self {
        let mut searches = Vec::new();
        let mut since = Vec::new();
        let shared = (cfg.confirm > 0).then(|| std::sync::Arc::new(state.clone()));
        for (seat, orders) in &hard {
            let battles = find_battles_where(state, *seat, cfg.max_battles, &keep);
            if battles.is_empty() {
                continue;
            }
            let visible: BTreeSet<UnitId> = legatum_core::player_view(state, *seat)
                .units
                .iter()
                .map(|u| u.id)
                .collect();
            for (i, b) in battles.into_iter().enumerate() {
                let seed = Rng::derive(
                    state.dice_seed.unwrap_or(state.setup.seed),
                    &[STREAM, u64::from(state.turn), u64::from(seat.0), i as u64],
                )
                .next_u64();
                let kept = plans.iter().find(|p| p.matches(&b));
                since.push(kept.map(|p| p.since));
                searches.push(BattleSearch::new(
                    state,
                    b,
                    orders,
                    seed,
                    cfg,
                    &visible,
                    kept.map(|p| p.stance),
                    shared.clone(),
                ));
            }
        }
        let involved: BTreeSet<UnitId> = searches
            .iter()
            .flat_map(|s| s.arena.state.units.keys().copied())
            .collect();
        TrainingRun {
            turn: state.turn,
            cfg: cfg.clone(),
            real_units: state
                .units
                .values()
                .filter(|u| involved.contains(&u.id))
                .map(|u| {
                    (
                        u.id,
                        UnitRecord {
                            id: u.id,
                            owner: u.owner,
                            kind: u.kind,
                            hp: u.hp,
                            pos: u.pos,
                        },
                    )
                })
                .collect(),
            cities: state
                .cities
                .values()
                .map(|c| {
                    (
                        c.id,
                        (
                            c.hp,
                            c.buildings.contains(&legatum_core::BuildingKind::Walls),
                        ),
                    )
                })
                .collect(),
            factions: state.players.iter().map(|p| (p.id, p.faction)).collect(),
            game: state.setup.seed,
            hard,
            searches,
            since,
        }
    }

    pub fn turn(&self) -> u32 {
        self.turn
    }

    pub fn is_done(&self) -> bool {
        self.searches.iter().all(|s| s.done)
    }

    /// Run up to `max_rollouts` rollouts of the battle being studied (in parallel where the
    /// build allows). `elapsed_ms`: the caller's clock since the run began. Returns whether
    /// every battle is done.
    pub fn step(&mut self, max_rollouts: usize, elapsed_ms: u64) -> bool {
        let cap = self.cfg.cap_ms;
        if cap > 0 && elapsed_ms >= cap {
            for s in &mut self.searches {
                s.stop(elapsed_ms);
            }
            return true;
        }
        let remaining = self.searches.iter().filter(|s| !s.done).count() as u64;
        let Some(s) = self.searches.iter_mut().find(|s| !s.done) else {
            return true;
        };
        if s.started_ms.is_none() {
            s.started_ms = Some(elapsed_ms);
            if cap > 0 {
                // Each battle gets an even share of the time left; a fast one passes its
                // unused time on. The arena gets three fifths of it when the full game
                // confirms.
                s.deadline_ms = elapsed_ms + (cap - elapsed_ms) / remaining.max(1);
                s.arena_deadline_ms = if s.cfg.confirm > 0 && s.real.is_some() {
                    elapsed_ms + (s.deadline_ms - elapsed_ms) * 3 / 5
                } else {
                    s.deadline_ms
                };
            }
        }
        // The rollout budget is the arena's; the full-game check runs after it.
        let over_budget =
            self.cfg.max_rollouts_per_battle > 0 && s.rollouts >= self.cfg.max_rollouts_per_battle;
        if s.in_arena_stage() && (elapsed_ms >= s.arena_deadline_ms || over_budget) {
            s.end_arena(false);
        }
        if !s.done && elapsed_ms >= s.deadline_ms {
            s.stop(elapsed_ms);
        }
        if s.done {
            s.ended_ms = s.ended_ms.max(elapsed_ms);
            return self.is_done();
        }
        let jobs = s.take(max_rollouts.max(1));
        let results = {
            let search = &*s;
            crate::search::map_jobs(&jobs, |j| search.eval(*j))
        };
        s.submit(jobs.into_iter().zip(results).collect());
        if s.done {
            s.ended_ms = elapsed_ms;
        }
        self.is_done()
    }

    pub fn progress(&self) -> TrainingProgress {
        TrainingProgress {
            turn: self.turn,
            battles: self
                .searches
                .iter()
                .map(|s| BattleProgress {
                    seat: s.battle.seat,
                    name: s.battle.name.clone(),
                    at: s.battle.at,
                    kind: s.battle.kind,
                    done: s.done,
                    changed: s.done && s.changed() > 0,
                })
                .collect(),
            current: self.searches.iter().position(|s| !s.done),
            done: self.is_done(),
            rollouts: self.searches.iter().map(|s| s.rollouts).sum(),
        }
    }

    /// Stop wherever the search is and return the best orders found, with the log lines.
    pub fn finish(mut self, elapsed_ms: u64) -> TrainingResult {
        for s in &mut self.searches {
            s.stop(elapsed_ms);
        }
        let progress = self.progress();
        let unit_of = legatum_core::rules::units::order_unit;
        let mut orders = self.hard.clone();
        let mut records = Vec::new();
        for s in &self.searches {
            let hard = &self.hard[&s.battle.seat];
            let mut searched = s.orders(hard, true);
            let changed = s.changed();
            if let Some(first) = &s.played_first {
                // The breach: its shooters, aircraft and walker take their new orders.
                let ids = s.breach_changed();
                let any = |o: &Order| unit_of(o).or(order_plane(o));
                let seat = orders.entry(s.battle.seat).or_default();
                seat.retain(|o| any(o).is_none_or(|u| !ids.contains(&u)));
                let new: Vec<Order> = first
                    .iter()
                    .filter(|o| any(o).is_some_and(|u| ids.contains(&u)))
                    .cloned()
                    .collect();
                seat.extend(new.iter().cloned());
                searched.retain(|o| any(o).is_none_or(|u| !ids.contains(&u)));
                searched.extend(new);
            } else if changed > 0 {
                let ids: BTreeSet<UnitId> = s
                    .battle
                    .units
                    .iter()
                    .zip(&s.choice)
                    .filter(|(_, c)| **c != 0)
                    .map(|(id, _)| *id)
                    .collect();
                let seat = orders.entry(s.battle.seat).or_default();
                seat.retain(|o| unit_of(o).is_none_or(|u| !ids.contains(&u)));
                seat.extend(
                    searched
                        .iter()
                        .filter(|o| unit_of(o).is_some_and(|u| ids.contains(&u)))
                        .cloned(),
                );
            }
            let (city_hp, walls) = s
                .battle
                .city
                .and_then(|c| self.cities.get(&c))
                .map_or((None, false), |(hp, w)| (Some(*hp), *w));
            let (gain, gain_ci) = paired(&s.current, &s.hard);
            let profile = s
                .battle
                .units
                .iter()
                .enumerate()
                .map(|(u, id)| UnitProfile {
                    unit: *id,
                    kind: s.arena.state.units[id].kind,
                    chosen: if s.current.is_empty() { 0 } else { s.choice[u] },
                    options: s.options[u]
                        .iter()
                        .zip(&s.profile[u])
                        .map(|(o, p)| OptionScore {
                            orders: o.iter().map(|o| s.arena.order_out(o)).collect(),
                            mean: p.map(|(m, _)| m),
                            n: p.map_or(0, |(_, n)| n),
                        })
                        .collect(),
                })
                .collect();
            let b = &s.battle;
            records.push(TrainingRecord {
                schema: LOG_SCHEMA,
                rules_version: legatum_core::RULES_VERSION,
                source: self.cfg.source.clone(),
                engagement: format!(
                    "{:x}-t{}-p{}-{},{}",
                    self.game, self.turn, b.seat.0, b.at.q, b.at.r
                ),
                turn: self.turn,
                seat: b.seat,
                faction: self.factions[&b.seat],
                enemy: b.enemy,
                kind: b.kind,
                name: b.name.clone(),
                at: b.at,
                city: b.city,
                city_hp,
                walls,
                units: s
                    .arena
                    .state
                    .units
                    .keys()
                    .filter_map(|id| self.real_units.get(id).cloned())
                    .collect(),
                hard_orders: s.orders(hard, false),
                searched_orders: searched,
                hard_score: if s.hard.is_empty() { 0 } else { mean(&s.hard) },
                searched_score: if s.current.is_empty() {
                    0
                } else {
                    mean(&s.current)
                },
                gain,
                gain_ci,
                hard_parts: Parts::mean(&s.hard),
                searched_parts: Parts::mean(&s.current),
                hard_by_defender: by_defender(&s.hard),
                searched_by_defender: by_defender(&s.current),
                horizon_reached: Parts::mean(&s.current).turns,
                changed_units: changed as u32,
                restart: RESTARTS[s.restart].to_string(),
                hidden_units: s.hidden_units,
                hidden_dependent: changed > 0 && s.hidden_units > 0,
                profile,
                rollouts: s.rollouts,
                candidates: s.candidates,
                passes: s.pass,
                complete: s.complete,
                ms: s
                    .ended_ms
                    .saturating_sub(s.started_ms.unwrap_or(s.ended_ms)),
                searched_units: b.units.clone(),
                arena: s.arena.snapshot(),
                variant: None,
                stance: s.stance,
                persisted: s.persisted,
                confirm: s.confirm_record.clone(),
                skipped: s.skipped.clone(),
            });
        }
        let plans = self
            .searches
            .iter()
            .zip(&self.since)
            .filter(|(s, _)| s.cfg.persist && s.stance != Stance::Hard && s.skipped.is_none())
            .map(|(s, since)| StoredPlan {
                seat: s.battle.seat,
                kind: s.battle.kind,
                city: s.battle.city,
                at: s.battle.at,
                stance: s.stance,
                since: since.unwrap_or(self.turn),
            })
            .collect();
        TrainingResult {
            turn: self.turn,
            orders,
            records,
            progress,
            plans,
        }
    }
}

/// Search a whole turn at once (the headless simulator).
/// `clock` returns ms since the run began; pass `|| 0` with `max_rollouts_per_battle` for a
/// deterministic budget.
pub fn search_turn(
    state: &GameState,
    hard: BTreeMap<PlayerId, Vec<Order>>,
    cfg: &TrainingConfig,
    clock: impl Fn() -> u64,
    on_progress: impl FnMut(&TrainingProgress),
) -> TrainingResult {
    search_turn_with(state, hard, cfg, &[], |_| true, clock, on_progress)
}

/// `search_turn`, continuing last turn's `plans` and searching only the battles `keep`
/// accepts (`TrainingRun::with_plans`).
pub fn search_turn_with(
    state: &GameState,
    hard: BTreeMap<PlayerId, Vec<Order>>,
    cfg: &TrainingConfig,
    plans: &[StoredPlan],
    keep: impl Fn(&Battle) -> bool,
    clock: impl Fn() -> u64,
    mut on_progress: impl FnMut(&TrainingProgress),
) -> TrainingResult {
    let mut run = TrainingRun::with_plans(state, hard, cfg, plans, keep);
    let batch = 256;
    loop {
        let done = run.step(batch, clock());
        on_progress(&run.progress());
        if done {
            break;
        }
    }
    run.finish(clock())
}

/// The realised outcome of a searched engagement, closed when the battle ends or after
/// `OUTCOME_TURNS`: the delayed label the rollouts are checked against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub schema: u32,
    pub engagement: String,
    pub seat: PlayerId,
    pub enemy: PlayerId,
    pub kind: BattleKind,
    pub city: Option<CityId>,
    pub opened: u32,
    pub closed: u32,
    /// Attack: the seat took the city. Defence: the seat kept it. None for a field battle.
    pub city_won: Option<bool>,
    /// Why it closed: `city`, `army_gone`, `broke_off`, `horizon`, `game_over`.
    pub reason: String,
    /// Units of each side from the decision that are dead or captured.
    pub seat_lost: u32,
    pub enemy_lost: u32,
}

pub const OUTCOME_TURNS: u32 = 20;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OpenEngagement {
    engagement: String,
    seat: PlayerId,
    enemy: PlayerId,
    kind: BattleKind,
    city: Option<CityId>,
    at: Hex,
    opened: u32,
    seat_units: Vec<UnitId>,
    enemy_units: Vec<UnitId>,
}

/// Engagements waiting for their realised outcome.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OutcomeTracker {
    open: Vec<OpenEngagement>,
}

impl OutcomeTracker {
    pub fn open(&mut self, records: &[TrainingRecord]) {
        for r in records {
            self.open_one(
                &r.engagement,
                r.seat,
                r.enemy,
                r.kind,
                r.city,
                r.at,
                r.turn,
                &r.units,
            );
        }
    }

    /// Track the human seat's logged positions too.
    pub fn open_positions(&mut self, records: &[PositionRecord]) {
        for r in records {
            self.open_one(
                &r.engagement,
                r.seat,
                r.enemy,
                r.kind,
                r.city,
                r.at,
                r.turn,
                &r.units,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn open_one(
        &mut self,
        engagement: &str,
        seat: PlayerId,
        enemy: PlayerId,
        kind: BattleKind,
        city: Option<CityId>,
        at: Hex,
        turn: u32,
        units: &[UnitRecord],
    ) {
        let side = |p: PlayerId| {
            units
                .iter()
                .filter(|u| u.owner == p)
                .map(|u| u.id)
                .collect()
        };
        self.open.push(OpenEngagement {
            engagement: engagement.to_string(),
            seat,
            enemy,
            kind,
            city,
            at,
            opened: turn,
            seat_units: side(seat),
            enemy_units: side(enemy),
        });
    }

    /// Close every engagement that has ended by `state` (all of them when the game is over).
    pub fn close(&mut self, state: &GameState) -> Vec<OutcomeRecord> {
        let mut out = Vec::new();
        self.open.retain(|e| {
            let lost = |ids: &[UnitId], owner: PlayerId| {
                ids.iter()
                    .filter(|id| state.units.get(id).is_none_or(|u| u.owner != owner))
                    .count() as u32
            };
            let seat_lost = lost(&e.seat_units, e.seat);
            let enemy_lost = lost(&e.enemy_units, e.enemy);
            let city_owner = e.city.map(|c| state.cities.get(&c).map(|c| c.owner));
            let near = |p: PlayerId| {
                state
                    .units
                    .values()
                    .any(|u| u.owner == p && fighting(u.kind) && u.pos.distance(e.at) <= REACH + 1)
            };
            let reason = if state.finished {
                Some("game_over")
            } else {
                match e.kind {
                    BattleKind::Attack if city_owner != Some(Some(e.enemy)) => Some("city"),
                    BattleKind::Defend if city_owner != Some(Some(e.seat)) => Some("city"),
                    _ if seat_lost as usize == e.seat_units.len()
                        || enemy_lost as usize == e.enemy_units.len() =>
                    {
                        Some("army_gone")
                    }
                    _ if !near(e.seat) || !near(e.enemy) => Some("broke_off"),
                    _ if state.turn >= e.opened + OUTCOME_TURNS => Some("horizon"),
                    _ => None,
                }
            };
            let Some(reason) = reason else {
                return true;
            };
            out.push(OutcomeRecord {
                schema: LOG_SCHEMA,
                engagement: e.engagement.clone(),
                seat: e.seat,
                enemy: e.enemy,
                kind: e.kind,
                city: e.city,
                opened: e.opened,
                closed: state.turn,
                city_won: match e.kind {
                    BattleKind::Attack => Some(city_owner == Some(Some(e.seat))),
                    BattleKind::Defend => Some(city_owner == Some(Some(e.seat))),
                    BattleKind::Field => None,
                },
                reason: reason.into(),
                seat_lost,
                enemy_lost,
            });
            false
        });
        out
    }
}

/// A battle position of a seat that is not searched live (the human's; designer, 2026-11-07:
/// his Training games are training data too). Logged with the orders he gave, so the position
/// can be searched and labelled offline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionRecord {
    pub schema: u32,
    pub rules_version: u32,
    pub source: String,
    pub engagement: String,
    pub turn: u32,
    pub seat: PlayerId,
    pub faction: legatum_core::Faction,
    pub human: bool,
    pub enemy: PlayerId,
    pub kind: BattleKind,
    pub name: String,
    pub at: Hex,
    pub city: Option<CityId>,
    pub city_hp: Option<i32>,
    pub walls: bool,
    /// Every unit of either side within the arena window.
    pub units: Vec<UnitRecord>,
    /// The seat's orders for its units in the battle.
    pub orders: Vec<Order>,
    pub searched_units: Vec<UnitId>,
    pub arena: ArenaSnapshot,
}

/// The battles `seat` fights this turn with its `orders` for them, unsearched.
pub fn positions(
    state: &GameState,
    seat: PlayerId,
    orders: &[Order],
    source: &str,
) -> Vec<PositionRecord> {
    let unit_of = legatum_core::rules::units::order_unit;
    find_battles(state, seat, usize::MAX)
        .into_iter()
        .map(|b| {
            let arena = Arena::cut(state, &b);
            let units = arena
                .state
                .units
                .keys()
                .filter_map(|id| state.units.get(id))
                .map(|u| UnitRecord {
                    id: u.id,
                    owner: u.owner,
                    kind: u.kind,
                    hp: u.hp,
                    pos: u.pos,
                })
                .collect();
            let city = b.city.and_then(|c| state.cities.get(&c));
            PositionRecord {
                schema: LOG_SCHEMA,
                rules_version: legatum_core::RULES_VERSION,
                source: source.to_string(),
                engagement: format!(
                    "{:x}-t{}-p{}-{},{}",
                    state.setup.seed, state.turn, seat.0, b.at.q, b.at.r
                ),
                turn: state.turn,
                seat,
                faction: state.player(seat).faction,
                human: state.player(seat).is_human,
                enemy: b.enemy,
                kind: b.kind,
                name: b.name.clone(),
                at: b.at,
                city: b.city,
                city_hp: city.map(|c| c.hp),
                walls: city
                    .is_some_and(|c| c.buildings.contains(&legatum_core::BuildingKind::Walls)),
                units,
                searched_units: b.units.clone(),
                arena: arena.snapshot(),
                orders: orders
                    .iter()
                    .filter(|o| unit_of(o).is_some_and(|u| b.units.contains(&u)))
                    .cloned()
                    .collect(),
            }
        })
        .collect()
}

/// A line of the training log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogLine {
    Decision(Box<TrainingRecord>),
    Position(Box<PositionRecord>),
    Outcome(OutcomeRecord),
    /// The turn's search failed or overran its cap: the training seats played Hard's orders.
    Fallback {
        turn: u32,
        reason: String,
    },
}

impl LogLine {
    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("log line")
    }
}

/// Search a logged battle again offline (designer, 2026-11-07: overnight re-search on his Mac):
/// at full width, and with `variants` also on perturbed copies of it: a searched unit moved a
/// tile, the seat's army at -20% and +20% HP, one enemy unit fewer, one more. Returns lines in
/// the log's own schema (source `retrain`, `variant` set). Outcome lines give nothing.
pub fn retrain_line(
    line: &str,
    cfg: &TrainingConfig,
    variants: bool,
) -> Result<Vec<TrainingRecord>, String> {
    let line: LogLine = serde_json::from_str(line).map_err(|e| e.to_string())?;
    let (battle, snap, hard, turn, hidden) = match &line {
        LogLine::Decision(r) => (
            Battle {
                seat: r.seat,
                enemy: r.enemy,
                kind: r.kind,
                at: r.at,
                city: r.city,
                name: r.name.clone(),
                units: r.searched_units.clone(),
            },
            &r.arena,
            r.hard_orders.clone(),
            r.turn,
            r.hidden_units,
        ),
        LogLine::Position(r) => (
            Battle {
                seat: r.seat,
                enemy: r.enemy,
                kind: r.kind,
                at: r.at,
                city: r.city,
                name: r.name.clone(),
                units: r.searched_units.clone(),
            },
            &r.arena,
            r.orders.clone(),
            r.turn,
            0,
        ),
        LogLine::Outcome(_) | LogLine::Fallback { .. } => return Ok(Vec::new()),
    };
    let arena = Arena::restore(snap)?;
    let mut out = vec![search_arena(
        arena.clone(),
        battle.clone(),
        &hard,
        cfg,
        turn,
        hidden,
        "logged",
    )];
    if variants {
        for (label, a) in perturb(&arena, &battle) {
            out.push(search_arena(
                a,
                battle.clone(),
                &hard,
                cfg,
                turn,
                hidden,
                label,
            ));
        }
    }
    Ok(out)
}

fn search_arena(
    arena: Arena,
    battle: Battle,
    hard: &[Order],
    cfg: &TrainingConfig,
    turn: u32,
    hidden: u32,
    variant: &str,
) -> TrainingRecord {
    let seed = Rng::derive(
        arena.state.setup.seed,
        &[
            STREAM,
            u64::from(turn),
            u64::from(battle.seat.0),
            variant
                .bytes()
                .fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(u64::from(b))),
        ],
    )
    .next_u64();
    let real_units = arena
        .state
        .units
        .values()
        .map(|u| {
            (
                u.id,
                UnitRecord {
                    id: u.id,
                    owner: u.owner,
                    kind: u.kind,
                    hp: u.hp,
                    pos: arena.to_real(u.pos),
                },
            )
        })
        .collect();
    let cities = arena
        .state
        .cities
        .values()
        .map(|c| {
            (
                c.id,
                (
                    c.hp,
                    c.buildings.contains(&legatum_core::BuildingKind::Walls),
                ),
            )
        })
        .collect();
    let factions = arena
        .state
        .players
        .iter()
        .map(|p| (p.id, p.faction))
        .collect();
    let game = arena.state.setup.seed;
    let seat = battle.seat;
    let search = BattleSearch::in_arena(arena, battle, hard, seed, cfg, hidden, None);
    let mut run = TrainingRun {
        turn,
        cfg: TrainingConfig {
            cap_ms: 0,
            ..cfg.clone()
        },
        hard: BTreeMap::from([(seat, hard.to_vec())]),
        searches: vec![search],
        real_units,
        cities,
        factions,
        game,
        since: vec![None],
    };
    while !run.step(256, 0) {}
    let mut r = run.finish(0).records.remove(0);
    r.source = "retrain".into();
    r.variant = Some(variant.into());
    r
}

/// The perturbed copies of a battle searched offline beside it.
fn perturb(arena: &Arena, b: &Battle) -> Vec<(&'static str, Arena)> {
    let mut out = Vec::new();
    let s = &arena.state;
    let free = |s: &GameState, h: Hex| {
        s.map.move_cost(h).is_some()
            && !s.units.values().any(|u| u.pos == h)
            && !s.cities.values().any(|c| c.pos == h)
    };
    // A searched unit a tile away.
    if let Some((id, to)) = b.units.iter().find_map(|id| {
        let u = s.units.get(id)?;
        u.pos
            .neighbors()
            .into_iter()
            .find(|h| free(s, *h))
            .map(|h| (*id, h))
    }) {
        let mut a = arena.clone();
        a.state.units.get_mut(&id).expect("unit").pos = to;
        out.push(("moved", a));
    }
    for (label, pct) in [("hp-20", 80), ("hp+20", 120)] {
        let mut a = arena.clone();
        for id in &b.units {
            if let Some(u) = a.state.units.get_mut(id) {
                u.hp = (u.hp * pct / 100).clamp(1, data::UNIT_HP);
            }
        }
        out.push((label, a));
    }
    let centre = arena.to_arena(b.at).unwrap_or(b.at);
    let foes: Vec<_> = s
        .units
        .values()
        .filter(|u| u.owner == b.enemy && fighting(u.kind))
        .collect();
    if foes.len() > 1
        && let Some(far) = foes
            .iter()
            .filter(|u| !s.cities.values().any(|c| c.pos == u.pos))
            .max_by_key(|u| (u.pos.distance(centre), u.id))
    {
        let mut a = arena.clone();
        a.state.units.remove(&far.id);
        out.push(("enemy-1", a));
    }
    if let Some((kind, at)) = foes.iter().max_by_key(|u| (u.hp, u.id)).and_then(|u| {
        u.pos
            .neighbors()
            .into_iter()
            .find(|h| free(s, *h))
            .map(|h| (u.kind, h))
    }) {
        let mut a = arena.clone();
        legatum_core::game::spawn_unit(&mut a.state, b.enemy, kind, at);
        out.push(("enemy+1", a));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::game::spawn_unit;
    use legatum_core::{GameSetup, TeamId};

    /// Two empires at war on a plain map: an enemy city with a Spearman garrison and three
    /// of the seat's units two hexes west of it, one Ranger among them.
    fn siege() -> (GameState, CityId) {
        let mut setup = GameSetup::all_ai(7);
        setup.players.truncate(2);
        setup.width = 30;
        setup.height = 20;
        setup.legacy_all_war = true;
        setup.barbarians = false;
        for (i, p) in setup.players.iter_mut().enumerate() {
            p.faction = legatum_core::Faction::India;
            p.team = TeamId(i as u8);
        }
        let mut s = blank_game(&setup, Map::new(30, 20, Terrain::Plains));
        let centre = Hex::from_offset(16, 9);
        let id = s.alloc_city_id();
        let city = legatum_core::state::City {
            last_attacked_turn: None,
            id,
            owner: PlayerId(1),
            founder: PlayerId(1),
            name: "Lugano".into(),
            pos: centre,
            population: 3,
            food_stored: 0,
            production_stored: 0,
            producing: None,
            production_banked: Vec::new(),
            production_queue: Vec::new(),
            wonder_lost: None,
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_queue_notified: Default::default(),
            last_purchase_turn: None,
            bought: None,
            buildings: Default::default(),
            hp: data::CITY_HP,
            tiles_claimed: 0,
            grown_tiles: Vec::new(),
            is_capital: true,
            original_capital: false,
            greece_captive_paid: Default::default(),
            occupied_from: None,
            founded_turn: 1,
            resistance: 0,
        };
        s.cities.insert(id, city);
        for h in centre.within(1) {
            let t = s.map.get_mut(h).unwrap();
            t.owner = Some(PlayerId(1));
            t.city = Some(id);
        }
        spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, centre);
        for (k, kind) in [UnitKind::Soldier, UnitKind::Ranger, UnitKind::Soldier]
            .into_iter()
            .enumerate()
        {
            spawn_unit(
                &mut s,
                PlayerId(0),
                kind,
                Hex::from_offset(14, 8 + k as i32),
            );
        }
        (s, id)
    }

    fn quick() -> TrainingConfig {
        TrainingConfig {
            rollouts: 4,
            screen: 2,
            horizon: 4,
            max_passes: 1,
            cap_ms: 0,
            ..TrainingConfig::default()
        }
    }

    #[test]
    fn an_army_two_hexes_from_an_enemy_city_is_an_attack_battle_with_every_unit_searched() {
        let (s, city) = siege();
        let battles = find_battles(&s, PlayerId(0), 3);
        assert_eq!(battles.len(), 1);
        assert_eq!(battles[0].kind, BattleKind::Attack);
        assert_eq!(battles[0].city, Some(city));
        assert_eq!(battles[0].name, "Lugano");
        assert_eq!(battles[0].units.len(), 3);
        let defend = find_battles(&s, PlayerId(1), 3);
        assert_eq!(defend.len(), 1);
        assert_eq!(defend[0].kind, BattleKind::Defend);
    }

    #[test]
    fn the_arena_keeps_hexes_adjacent_and_maps_back_to_the_real_ones() {
        let (s, _) = siege();
        let b = find_battles(&s, PlayerId(0), 3).remove(0);
        let a = Arena::cut(&s, &b);
        for u in s.units.values() {
            let at = a.to_arena(u.pos).expect("inside");
            assert_eq!(a.to_real(at), u.pos);
            assert_eq!(a.state.units[&u.id].pos, at);
            for n in u.pos.neighbors() {
                assert_eq!(a.to_arena(n).map(|h| h.distance(at)), Some(1));
            }
        }
    }

    #[test]
    fn every_unit_gets_hold_rest_moves_and_attacks_and_suicides_are_pruned() {
        let (s, _) = siege();
        let b = find_battles(&s, PlayerId(0), 3).remove(0);
        let a = Arena::cut(&s, &b);
        let view = legatum_core::player_view(&a.state, PlayerId(0));
        let ranger = b
            .units
            .iter()
            .copied()
            .find(|id| s.units[id].kind == UnitKind::Ranger)
            .unwrap();
        let opts = unit_options(&a.state, &view, ranger, a.to_arena(b.at).unwrap());
        assert!(opts.iter().any(|o| matches!(o, Order::SkipUnit { .. })));
        assert!(opts.iter().any(|o| matches!(o, Order::RangedAttack { .. })));
        assert!(
            opts.iter()
                .filter(|o| matches!(o, Order::MoveUnit { .. }))
                .count()
                >= 4
        );
    }

    #[test]
    fn a_turn_search_is_deterministic_and_only_changes_the_searched_units_orders() {
        let (s, _) = siege();
        let hard = BTreeMap::from([(
            PlayerId(0),
            s.units_of(PlayerId(0))
                .map(|u| Order::SkipUnit { unit: u.id })
                .chain([Order::SetResearch {
                    tech: legatum_core::TechId::Masonry,
                }])
                .collect::<Vec<_>>(),
        )]);
        let a = search_turn(&s, hard.clone(), &quick(), || 0, |_| {});
        let b = search_turn(&s, hard.clone(), &quick(), || 0, |_| {});
        assert_eq!(a, b);
        assert_eq!(a.records.len(), 1);
        let r = &a.records[0];
        assert!(r.complete);
        assert!(r.searched_score >= r.hard_score);
        assert!(a.orders[&PlayerId(0)].contains(&Order::SetResearch {
            tech: legatum_core::TechId::Masonry
        }));
        assert_eq!(
            a.orders[&PlayerId(0)]
                .iter()
                .filter(|o| legatum_core::rules::units::order_unit(o).is_some())
                .count(),
            3
        );
    }

    #[test]
    fn the_time_cap_stops_the_search_and_keeps_hards_orders() {
        let (s, _) = siege();
        let hard = BTreeMap::from([(
            PlayerId(0),
            s.units_of(PlayerId(0))
                .map(|u| Order::SkipUnit { unit: u.id })
                .collect::<Vec<_>>(),
        )]);
        let cfg = TrainingConfig {
            cap_ms: 10,
            ..quick()
        };
        let mut run = TrainingRun::new(&s, hard.clone(), &cfg);
        assert!(run.step(1, 10));
        let r = run.finish(10);
        assert_eq!(r.orders, hard);
        assert!(!r.records[0].complete);
    }

    #[test]
    fn a_decision_logs_components_profile_and_defender_split_and_its_outcome_closes_on_capture() {
        let (s, city) = siege();
        let hard = BTreeMap::from([(
            PlayerId(0),
            s.units_of(PlayerId(0))
                .map(|u| Order::SkipUnit { unit: u.id })
                .collect::<Vec<_>>(),
        )]);
        let r = search_turn(&s, hard, &quick(), || 0, |_| {});
        let d = &r.records[0];
        assert_eq!(d.profile.len(), 3);
        assert!(d.profile.iter().all(|p| p.options.len() > 2));
        assert!(d.profile.iter().all(|p| p.options[0].mean.is_some()));
        assert_eq!(d.hard_score, d.hard_parts.total());
        assert!(d.horizon_reached >= 1 && d.horizon_reached <= 4);
        assert!(RESTARTS.contains(&d.restart.as_str()));
        assert!(!d.hidden_dependent, "the seat sees the whole siege");
        let line = LogLine::Decision(Box::new(d.clone())).json();
        assert!(line.starts_with("{\"type\":\"decision\""));
        let mut tracker = OutcomeTracker::default();
        tracker.open(&r.records);
        assert!(tracker.close(&s).is_empty(), "still being fought");
        let mut taken = s.clone();
        taken.cities.get_mut(&city).unwrap().owner = PlayerId(0);
        let out = tracker.close(&taken);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].engagement, d.engagement);
        assert_eq!(out[0].city_won, Some(true));
        assert_eq!(out[0].reason, "city");
    }

    #[test]
    fn the_paired_gain_is_the_mean_difference_with_its_interval() {
        let p = |x: i64| Parts {
            city: x,
            ..Parts::default()
        };
        let (m, ci) = paired(&[p(10), p(20), p(30), p(40)], &[p(0), p(10), p(20), p(30)]);
        assert_eq!((m, ci), (10, 0));
        let (m, ci) = paired(&[p(10), p(30)], &[p(0), p(0)]);
        assert_eq!(m, 20);
        assert!(ci > 0);
    }

    #[test]
    fn a_logged_battle_rebuilds_from_its_line_and_is_searched_again_with_variants() {
        let (s, _) = siege();
        let hard = BTreeMap::from([(
            PlayerId(0),
            s.units_of(PlayerId(0))
                .map(|u| Order::SkipUnit { unit: u.id })
                .collect::<Vec<_>>(),
        )]);
        let r = search_turn(&s, hard, &quick(), || 0, |_| {});
        let line = LogLine::Decision(Box::new(r.records[0].clone())).json();
        let again = retrain_line(&line, &quick(), true).unwrap();
        assert_eq!(
            again.len(),
            5,
            "logged, moved, hp-20, hp+20, enemy+1 (no spare enemy to drop)"
        );
        assert_eq!(again[0].variant.as_deref(), Some("logged"));
        assert_eq!(again[0].source, "retrain");
        assert_eq!(
            again[0].units, r.records[0].units,
            "the same battle, in real hexes"
        );
        assert_eq!(again[0].hard_orders, r.records[0].hard_orders);
    }

    fn hard_orders(s: &GameState, seat: PlayerId) -> Vec<Order> {
        let mut b =
            ScriptedBrain::with_difficulty(s.player(seat).faction, 11, seat, AiDifficulty::Hard);
        crate::Brain::orders(&mut b, &legatum_core::player_view(s, seat))
    }

    #[test]
    fn a_siege_stance_shoots_the_walls_and_holds_the_melee_until_the_city_is_at_zero() {
        let (mut s, city) = siege();
        let at = s.cities[&city].pos;
        // The two Soldiers two tiles from the city.
        let soldiers: Vec<UnitId> = s
            .units_of(PlayerId(0))
            .filter(|u| u.kind == UnitKind::Soldier)
            .map(|u| u.id)
            .collect();
        let taken: Vec<Hex> = s.units.values().map(|u| u.pos).collect();
        let free: Vec<Hex> = at
            .ring(2)
            .into_iter()
            .filter(|h| !taken.contains(h))
            .collect();
        for (id, h) in soldiers.iter().zip(free) {
            s.units.get_mut(id).unwrap().pos = h;
        }
        let view = legatum_core::player_view(&s, PlayerId(0));
        let hard: Vec<Order> = s
            .units_of(PlayerId(0))
            .map(|u| Order::MoveUnit {
                unit: u.id,
                to: Hex::from_offset(10, 9),
            })
            .collect();
        let o = stance_orders(&view, Stance::Siege, hard.clone(), at, 4, Some(city));
        let unit_of = legatum_core::rules::units::order_unit;
        for u in s.units_of(PlayerId(0)) {
            let mine: Vec<_> = o.iter().filter(|x| unit_of(x) == Some(u.id)).collect();
            assert_eq!(mine.len(), 1);
            if u.kind == UnitKind::Ranger {
                assert_eq!(
                    mine[0],
                    &Order::RangedAttack {
                        unit: u.id,
                        target: at
                    }
                );
            } else {
                assert_eq!(mine[0], &Order::SkipUnit { unit: u.id });
            }
        }
        assert_eq!(
            stance_orders(&view, Stance::Hard, hard.clone(), at, 4, Some(city)),
            hard
        );
        // At 0 HP (here with its garrison gone) the melee storms.
        s.cities.get_mut(&city).unwrap().hp = 0;
        s.units.retain(|_, u| u.owner == PlayerId(0));
        let soldier = soldiers[0];
        let next = at
            .neighbors()
            .into_iter()
            .find(|h| !s.units.values().any(|u| u.pos == *h))
            .unwrap();
        s.units.get_mut(&soldier).unwrap().pos = next;
        let view = legatum_core::player_view(&s, PlayerId(0));
        let o = stance_orders(&view, Stance::Siege, hard, at, 4, Some(city));
        assert!(o.contains(&Order::MoveUnit {
            unit: soldier,
            to: at
        }));
    }

    fn round2() -> TrainingConfig {
        TrainingConfig {
            confirm_turns: 3,
            ..quick().round2().with_confirm(6)
        }
    }

    /// The fixed positions of the guarantee test: the siege from each side, a battered city,
    /// and a relief army arriving.
    fn positions() -> Vec<(&'static str, GameState, PlayerId)> {
        let (s, city) = siege();
        let mut battered = s.clone();
        battered.cities.get_mut(&city).unwrap().hp = data::CITY_HP / 4;
        let mut relief = s.clone();
        for r in [8, 10] {
            spawn_unit(
                &mut relief,
                PlayerId(1),
                UnitKind::Soldier,
                Hex::from_offset(19, r),
            );
        }
        vec![
            ("attack", s.clone(), PlayerId(0)),
            ("defend", s, PlayerId(1)),
            ("battered", battered, PlayerId(0)),
            ("relief", relief, PlayerId(0)),
        ]
    }

    #[test]
    fn round_two_confirms_in_the_full_game_and_keeps_a_played_stance() {
        let (s, _) = siege();
        let hard = BTreeMap::from([(PlayerId(0), hard_orders(&s, PlayerId(0)))]);
        let a = search_turn(&s, hard.clone(), &round2(), || 0, |_| {});
        let b = search_turn(&s, hard.clone(), &round2(), || 0, |_| {});
        assert_eq!(a, b, "deterministic");
        let r = &a.records[0];
        if let Some(c) = &r.confirm {
            assert_eq!(c.rollouts, 6);
            assert_eq!(c.candidates[0].stance, Stance::Hard);
            assert!(c.candidates[0].orders.is_empty(), "Hard's own orders");
            assert!(c.candidates.len() >= 2 && c.candidates.len() <= 3);
            let played = &c.candidates[c.played];
            assert_eq!(played.stance, r.stance);
            if c.played > 0 {
                assert_eq!(c.reason, "clear");
                assert!(played.gain - played.gain_ci > 0);
            } else {
                assert_eq!(r.changed_units, 0);
                assert_eq!(a.orders, hard);
            }
        } else {
            assert_eq!(a.orders, hard, "nothing beat Hard in the arena");
        }
        assert_eq!(a.plans.is_empty(), r.stance == Stance::Hard);
        // A kept stance is the next turn's continuation.
        let kept = StoredPlan {
            seat: PlayerId(0),
            kind: BattleKind::Attack,
            city: r.city,
            at: r.at,
            stance: Stance::Siege,
            since: 3,
        };
        let c = search_turn_with(&s, hard, &round2(), &[kept], |_| true, || 0, |_| {});
        assert!(c.records[0].persisted);
        if c.records[0].stance != Stance::Hard {
            assert_eq!(c.plans[0].since, 3);
        }
    }

    #[test]
    fn with_the_time_cap_hit_before_the_full_game_check_the_seat_plays_hards_orders() {
        let (s, _) = siege();
        let hard = BTreeMap::from([(PlayerId(0), hard_orders(&s, PlayerId(0)))]);
        let mut run = TrainingRun::new(
            &s,
            hard.clone(),
            &TrainingConfig {
                cap_ms: 1000,
                ..round2()
            },
        );
        assert!(!run.step(64, 0));
        let r = run.finish(1000);
        assert_eq!(r.orders, hard);
        assert_eq!(
            r.records[0].confirm.as_ref().map(|c| c.reason.as_str()),
            Some("cap")
        );
    }

    /// Round 2 (d), the guarantee: on fixed positions the played orders' full-game score is
    /// never significantly below Hard's, on fresh rollouts the search never saw.
    #[test]
    fn the_played_orders_are_never_significantly_worse_than_hards_in_the_full_game() {
        for (name, s, seat) in positions() {
            let hard = hard_orders(&s, seat);
            let r = search_turn(
                &s,
                BTreeMap::from([(seat, hard.clone())]),
                &round2(),
                || 0,
                |_| {},
            );
            let Some(b) = find_battles(&s, seat, 3).into_iter().next() else {
                panic!("{name}: no battle");
            };
            let played = &r.orders[&seat];
            let stance = r.records[0].stance;
            let fresh = |first: &[Order], stance: Stance| -> Vec<Parts> {
                (0..16)
                    .map(|k| full_rollout(&s, &b, first, stance, 0x9e37_0000 + k, 3))
                    .collect()
            };
            let (gain, ci) = paired(&fresh(played, stance), &fresh(&hard, Stance::Hard));
            assert!(
                gain + ci >= 0,
                "{name}: played {gain} +- {ci} below Hard's orders"
            );
        }
    }

    /// The siege with the city battered to `hp`, a Soldier next to it, and a Bomber in the
    /// hangar of a seat city 6 tiles west.
    fn battered_with_bomber(hp: i32) -> (GameState, CityId, UnitId, UnitId) {
        let (mut s, city) = siege();
        s.cities.get_mut(&city).unwrap().hp = hp;
        let at = s.cities[&city].pos;
        let soldier = s
            .units_of(PlayerId(0))
            .find(|u| u.kind == UnitKind::Soldier)
            .unwrap()
            .id;
        let next = at
            .neighbors()
            .into_iter()
            .find(|h| !s.units.values().any(|u| u.pos == *h))
            .unwrap();
        s.units.get_mut(&soldier).unwrap().pos = next;
        let base = s.alloc_city_id();
        let mut home = s.cities[&city].clone();
        home.id = base;
        home.owner = PlayerId(0);
        home.founder = PlayerId(0);
        home.name = "Bern".into();
        home.pos = Hex::from_offset(10, 9);
        home.hp = data::CITY_HP;
        s.cities.insert(base, home);
        let t = s.map.get_mut(Hex::from_offset(10, 9)).unwrap();
        t.owner = Some(PlayerId(0));
        t.city = Some(base);
        let plane = UnitId(
            s.units
                .keys()
                .map(|u| u.0)
                .max()
                .unwrap_or(0)
                .max(s.next_unit_id)
                + 1,
        );
        s.next_unit_id = plane.0 + 1;
        s.aircraft.push(legatum_core::state::Aircraft {
            id: plane,
            kind: UnitKind::Bomber,
            city: base,
            hp: data::UNIT_HP,
            xp: 0,
        });
        (s, city, soldier, plane)
    }

    #[test]
    fn the_breach_bombs_the_city_to_zero_and_the_cheapest_melee_walks_in_the_same_turn() {
        let (s, city, soldier, plane) = battered_with_bomber(10);
        let at = s.cities[&city].pos;
        let view = legatum_core::player_view(&s, PlayerId(0));
        let hard: Vec<Order> = s
            .units_of(PlayerId(0))
            .map(|u| Order::SkipUnit { unit: u.id })
            .collect();
        let b = breach_orders(&view, &hard, city).expect("the shots zero a 10 HP city");
        assert!(b.contains(&Order::AirStrike { plane, target: at }));
        assert!(b.contains(&Order::MoveUnit {
            unit: soldier,
            to: at
        }));
        // A full-HP city the Ranger alone can't zero: no breach.
        let (mut s, city, _, _) = battered_with_bomber(data::CITY_HP);
        s.aircraft.clear();
        let view = legatum_core::player_view(&s, PlayerId(0));
        assert!(breach_orders(&view, &hard, city).is_none());
    }

    /// Gene `breach` in plain Hard, end to end through the live order path: the Bomber's
    /// strike is the planner's (the air planner leaves that plane alone) and the Soldier
    /// beside the city walks in the same turn.
    #[test]
    fn plain_hard_with_the_breach_gene_bombs_and_walks_in() {
        use crate::Brain;
        let (s, city, soldier, plane) = battered_with_bomber(60);
        let at = s.cities[&city].pos;
        let view = legatum_core::player_view(&s, PlayerId(0));
        let faction = s.player(PlayerId(0)).faction;
        let mut params = crate::AiParams::default();
        params.decisions.breach = 100;
        let mut brain = crate::ScriptedBrain::with_params(
            faction,
            7,
            PlayerId(0),
            legatum_core::AiDifficulty::Hard,
            &params,
        );
        let orders = brain.orders(&view);
        assert!(
            orders.contains(&Order::AirStrike { plane, target: at }),
            "{orders:?}"
        );
        assert!(
            orders.contains(&Order::MoveUnit {
                unit: soldier,
                to: at
            }),
            "{orders:?}"
        );
        assert_eq!(
            orders
                .iter()
                .filter(|o| matches!(o, Order::AirStrike { plane: p, .. } if *p == plane))
                .count(),
            1
        );
    }

    #[test]
    fn the_full_game_check_always_weighs_the_breach_and_plays_it_when_it_takes_the_city() {
        let (s, city, soldier, plane) = battered_with_bomber(10);
        let hard: Vec<Order> = s
            .units_of(PlayerId(0))
            .map(|u| Order::SkipUnit { unit: u.id })
            .collect();
        let r = search_turn(
            &s,
            BTreeMap::from([(PlayerId(0), hard)]),
            &round2(),
            || 0,
            |_| {},
        );
        let c = r.records[0]
            .confirm
            .as_ref()
            .expect("checked in the full game");
        assert!(c.candidates.iter().any(|x| x.breach));
        let played = &c.candidates[c.played];
        assert!(played.gain - played.gain_ci > 0, "{c:?}");
        let at = s.cities[&city].pos;
        let o = &r.orders[&PlayerId(0)];
        if played.breach {
            assert!(o.contains(&Order::AirStrike { plane, target: at }));
            assert!(o.contains(&Order::MoveUnit {
                unit: soldier,
                to: at
            }));
        }
    }
}
