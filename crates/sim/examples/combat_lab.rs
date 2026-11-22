//! The combat lab: mass city-capture arenas on the real engine, to calibrate the AI's siege
//! forecast and compare tactical policies. Stage 2 adds a live defending economy, an attacker
//! scout/fog dimension, and four forced tactical hooks alongside the AI's own policy and storm.
//!
//! Every cell is one situation: attacker era, faction, army mix and size, the city's Walls,
//! garrison and HP, the terrain around it and the march distance. Each cell is played `--repeats`
//! times with a different dice seed (the seed changes the engine's attack rolls, not just the
//! AI's tie-breaks), once per policy with the same dice seed across all five:
//!   - `ai`: Hard's own tactical planner (`ScriptedBrain::tactical_orders`) on the attacker side;
//!   - `storm`: every melee and mounted unit walks into the city every turn, whatever its
//!     health; shooters fire at the city tile from the nearest firing position.
//!   - `wait`, `surround`, and `focus`: small simulator-only policy hooks.
//!
//! The defender uses Hard's real city planner and tactical AI. The city begins with 300 Gold;
//! normal city orders, production, income, upkeep, and healing run each turn. Pass
//! `--static-defender` to disable city orders and city processing for a stage-1 comparison.
//!
//! Per cell it also records what the AI itself expects from the starting position
//! (`ScriptedBrain::probe_city_assault`): the `forecast_siege` result (capture turn and losses
//! in Production), whether the strike forecast pays for the war, and whether the assault check
//! commits orders on turn 1.
//!
//!   cargo run --release -p legatum-sim --example combat_lab -- --out DIR [--repeats 24]
//!       [--threads N] [--eras ancient,classical,renaissance] [--factions generic,sparta,mongols]
//!       [--ground flat,hills,river] [--hp 200,100] [--stride K] [--subset fixed]
//!       [--static-defender]
//!       [--check BASELINE.json] [--write-baseline FILE] [--trace CELL_ID SEED POLICY]
//!
//! Output: `cells.csv` (one row per cell), `summary.json` (runtime and run counts).
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{
    AiDifficulty, BuildingKind, Event, Faction, GameSetup, GameState, Hex, Order, PlayerId, TeamId,
    TechId, Terrain, UnitId, UnitKind,
    data::{self, FORTIFY_MAX},
    events::Defender,
    game::{blank_game, spawn_unit},
    los,
    rules::{is_military, resolve_tactical_turn},
    state::{City, Map},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fmt::Write as _, fs, path::PathBuf, time::Instant};

const WIDTH: i32 = 24;
const HEIGHT: i32 = 15;
/// Turns an assault gets. The AI's own forecast looks at most `approach + 4` rounds ahead.
const LIMIT: u32 = 12;
const CITY_COL: i32 = 12;
const CITY_ROW: i32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Era {
    Ancient,
    Classical,
    Renaissance,
}
impl Era {
    const ALL: [Era; 3] = [Era::Ancient, Era::Classical, Era::Renaissance];
    /// The era (0 Ancient .. 7 Future, `legatum_core::eras`) the cell stands in.
    fn tier(self) -> u8 {
        match self {
            Era::Ancient => 0,
            Era::Classical => 1,
            Era::Renaissance => 3,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Era::Ancient => "ancient",
            Era::Classical => "classical",
            Era::Renaissance => "renaissance",
        }
    }
    fn melee(self) -> UnitKind {
        match self {
            Era::Ancient => UnitKind::Soldier,
            Era::Classical => UnitKind::Phalangite,
            Era::Renaissance => UnitKind::AssaultTrooper,
        }
    }
    fn ranged(self) -> UnitKind {
        match self {
            Era::Ancient => UnitKind::Ranger,
            Era::Classical => UnitKind::CompositeArcher,
            Era::Renaissance => UnitKind::Arquebusier,
        }
    }
    fn mounted(self) -> UnitKind {
        match self {
            Era::Ancient => UnitKind::Horseman,
            Era::Classical => UnitKind::Cataphract,
            Era::Renaissance => UnitKind::Cuirassier,
        }
    }
    /// The Military Engineer's siege tier an attacker of this era has by default
    /// (`data::siege_tier`): none in the Ancient era, the Catapult tier in the Classical era
    /// and the Bombard tier in the Renaissance.
    fn engineer_tier(self) -> u8 {
        match self {
            Era::Ancient => 0,
            Era::Classical => 1,
            Era::Renaissance => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mix {
    Melee,
    Mixed,
    Shooters,
    Mounted,
}
impl Mix {
    fn label(self) -> &'static str {
        match self {
            Mix::Melee => "melee",
            Mix::Mixed => "mixed",
            Mix::Shooters => "shooters",
            Mix::Mounted => "mounted",
        }
    }
    fn army(self, era: Era, n: usize) -> Vec<UnitKind> {
        let (m, r) = (era.melee(), era.ranged());
        match self {
            Mix::Melee => vec![m; n],
            Mix::Mounted => vec![era.mounted(); n],
            Mix::Mixed => {
                let melee = n.div_ceil(2);
                let mut a = vec![m; melee];
                a.extend(vec![r; n - melee]);
                a
            }
            Mix::Shooters => {
                let mut a = vec![m];
                a.extend(vec![r; n - 1]);
                a
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Ground {
    Flat,
    /// The city and the ring around it stand on Hills.
    Hills,
    /// A river runs one column west of the city; it can be crossed only at a two-tile ford.
    River,
}
impl Ground {
    fn label(self) -> &'static str {
        match self {
            Ground::Flat => "flat",
            Ground::Hills => "hills",
            Ground::River => "river",
        }
    }
}

fn faction_of(label: &str) -> Faction {
    match label {
        "generic" => Faction::India,
        "sparta" => Faction::Sparta,
        "mongols" => Faction::Mongols,
        other => panic!("unknown faction label {other}"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Cell {
    era: Era,
    /// generic (India), sparta (Sparta) or mongols (Mongols).
    faction: String,
    mix: Mix,
    n: usize,
    walls: bool,
    /// Walls tiers above plain Walls: 1 a Castle, 2 a Castle and a Star Fort.
    #[serde(default, skip_serializing_if = "is_zero")]
    fort: u8,
    /// Garrison size: the first unit stands in the city, the rest on the tiles behind it.
    garrison: usize,
    garrison_ranged: bool,
    city_hp: i32,
    ground: Ground,
    /// Hex distance from the city to the nearest attacker at the start.
    dist: i32,
    /// The attacking force has a Scout, revealing a city three hexes away.
    scout: bool,
    /// Siege research rule variant (`VARIANTS`); empty is the live rules with no Engineer.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    variant: String,
    /// The attacker's Military Engineer siege tier (1 to 3); 0 is the era's own (`Era::engineer_tier`).
    #[serde(default, skip_serializing_if = "is_zero")]
    eng_tier: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

/// How the attacker's Military Engineers are fielded and moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Eng {
    /// No Engineer: walls halve every attack.
    None,
    /// One Engineer per fighting unit, stacked on its tile (the rule as decided).
    Stacked,
}
impl Cell {
    fn id(&self) -> String {
        let suffix = if self.variant.is_empty() {
            String::new()
        } else {
            format!("-v{}", self.variant)
        };
        let tier = if self.eng_tier == 0 {
            String::new()
        } else {
            format!("-t{}", self.eng_tier)
        };
        let fort = if self.fort == 0 {
            String::new()
        } else {
            format!("f{}", self.fort)
        };
        format!(
            "{}-{}-{}{}-w{}{fort}-g{}{}-hp{}-{}-d{}-scout{}{tier}{suffix}",
            self.era.label(),
            self.faction,
            self.mix.label(),
            self.n,
            u8::from(self.walls),
            self.garrison,
            if self.garrison == 0 {
                ""
            } else if self.garrison_ranged {
                "r"
            } else {
                "m"
            },
            self.city_hp,
            self.ground.label(),
            self.dist,
            u8::from(self.scout)
        )
    }
    fn army(&self) -> Vec<UnitKind> {
        self.mix.army(self.era, self.n)
    }
    fn eng(&self) -> Eng {
        variant_eng(&self.variant)
    }
    /// Military Engineers fielded with the attacking force.
    fn eng_n(&self) -> usize {
        match self.eng() {
            Eng::None => 0,
            Eng::Stacked => self.n,
        }
    }
    /// The Walls tier of the city: 0 none, 1 Walls, 2 a Castle, 3 a Star Fort.
    fn wall_tier(&self) -> u8 {
        if self.walls { 1 + self.fort } else { 0 }
    }
    /// The attacker's siege tier: the cell's own, else the era's.
    fn tier(&self) -> u8 {
        if self.eng_tier == 0 {
            self.era.engineer_tier()
        } else {
            self.eng_tier
        }
    }
    /// Production cost of the attacking force with its Engineers at the current price.
    fn army_cost(&self) -> i32 {
        let faction = faction_of(&self.faction);
        self.army()
            .iter()
            .map(|k| data::unit_stats_for(faction, *k).cost)
            .sum::<i32>()
            // Siege is bought with Gold; at the upgrade rate of 2 Gold a Production.
            + self.eng_n() as i32 * data::SIEGE_BASE_GOLD / 2
    }
}

/// Military Engineer study variants (`docs/design/research/siege-rework-2026-10-18.md`): named
/// sets of opt-in rule experiments and how the attacker fields its Engineers. Every variant plays
/// under the real rules (Walls halve uncovered attacks); they differ in the Engineers.
const VARIANTS: &[(&str, &[&str], Eng)] =
    &[("base", &[], Eng::None), ("stacked", &[], Eng::Stacked)];

fn variant_eng(name: &str) -> Eng {
    if name.is_empty() {
        return Eng::None;
    }
    VARIANTS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, _, e)| *e)
        .unwrap_or_else(|| panic!("unknown siege variant {name}"))
}

fn variant_flags(name: &str) -> &'static [&'static str] {
    if name.is_empty() {
        return &[];
    }
    VARIANTS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, f, _)| *f)
        .unwrap_or_else(|| panic!("unknown siege variant {name}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Policy {
    Ai,
    /// Training mode (`legatum_ai::training`): Hard's orders with the battle searched.
    Training,
    Storm,
    WaitShooters,
    Surround,
    FocusFire,
}

fn passable(map: &Map, h: Hex) -> bool {
    map.move_cost(h).is_some()
}

/// The arena: a city at a fixed spot with the attackers `dist` hexes west of it.
fn arena(c: &Cell, seed: u64) -> (GameState, legatum_core::CityId) {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(2);
    setup.width = WIDTH;
    setup.height = HEIGHT;
    setup.legacy_all_war = true;
    setup.barbarians = false;
    for flag in variant_flags(&c.variant) {
        setup.rule_experiments.apply(flag).expect("variant flag");
    }
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = if i == 0 {
            faction_of(&c.faction)
        } else {
            Faction::India
        };
        p.team = TeamId(i as u8);
        p.difficulty = AiDifficulty::Hard;
    }
    let mut map = Map::new(WIDTH, HEIGHT, Terrain::Plains);
    let centre = Hex::from_offset(CITY_COL, CITY_ROW);
    if c.ground == Ground::Hills {
        for h in centre.within(1) {
            map.get_mut(h).unwrap().terrain = Terrain::Hills;
        }
    }
    if c.ground == Ground::River {
        for row in 0..HEIGHT {
            map.get_mut(Hex::from_offset(CITY_COL - 1, row))
                .unwrap()
                .terrain = Terrain::Coast;
        }
        for h in centre.neighbors() {
            if h.to_offset().0 == CITY_COL - 1 {
                map.get_mut(h).unwrap().terrain = Terrain::Plains;
            }
        }
    }
    let mut s = blank_game(&setup, map);
    // The arena city starts with a meaningful treasury; later Gold comes from real city income
    // and pays the same purchase and upkeep rules as an ordinary player.
    s.player_mut(PlayerId(1)).gold = 300;
    s.player_mut(PlayerId(1)).techs.insert(TechId::Masonry);
    // The defender stands in the attacker's era (every technology of the eras before it): its
    // city lines ride the era curve with its owner's era, as in a real game.
    for era in 0..c.era.tier() {
        s.player_mut(PlayerId(1))
            .techs
            .extend(legatum_core::eras::era_techs(era));
    }
    for p in &mut s.players {
        for i in 0..s.map.len() {
            p.explored.set(i);
        }
    }
    let id = s.alloc_city_id();
    let mut buildings = Default::default();
    for (tier, building) in [
        (1, BuildingKind::Walls),
        (2, BuildingKind::Castle),
        (3, BuildingKind::StarFort),
    ] {
        if c.wall_tier() >= tier {
            std::collections::BTreeSet::insert(&mut buildings, building);
        }
    }
    s.cities.insert(
        id,
        City {
            last_attacked_turn: None,
            id,
            owner: PlayerId(1),
            founder: PlayerId(1),
            name: "Arena".into(),
            pos: centre,
            population: 3,
            food_stored: 0,
            production_stored: 0,
            producing: None,
            production_banked: Vec::new(),
            production_queue: Vec::new(),
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_queue_notified: Default::default(),
            wonder_lost: None,
            last_purchase_turn: None,
            bought: None,
            buildings,
            hp: c.city_hp,
            tiles_claimed: 0,
            grown_tiles: Vec::new(),
            is_capital: false,
            original_capital: false,
            greece_captive_paid: Default::default(),
            occupied_from: None,
            founded_turn: 1,
            resistance: 0,
        },
    );
    for h in centre.within(1) {
        if let Some(t) = s.map.get_mut(h) {
            t.owner = Some(PlayerId(1));
            t.city = Some(id);
        }
    }
    // Each side also owns a far-off capital with one garrison unit, as in a real war: Hard's
    // strike forecast needs a home to be strategically sane, and the defender is not on its
    // last city. The two capitals are about ten hexes from the fight and never matter to it.
    for (owner, col) in [(PlayerId(0), 2), (PlayerId(1), WIDTH - 2)] {
        let home = s.alloc_city_id();
        let pos = Hex::from_offset(col, CITY_ROW);
        let mut city = s.cities[&id].clone();
        city.id = home;
        city.owner = owner;
        city.founder = owner;
        city.name = "Home".into();
        city.pos = pos;
        city.hp = data::CITY_HP;
        city.buildings = Default::default();
        city.is_capital = true;
        s.cities.insert(home, city);
        for h in pos.within(1) {
            if let Some(t) = s.map.get_mut(h) {
                t.owner = Some(owner);
                t.city = Some(home);
            }
        }
        let guard = spawn_unit(&mut s, owner, c.era.melee(), pos);
        s.units.get_mut(&guard).unwrap().fortify = FORTIFY_MAX;
    }
    // Attackers: fill the distance ring from the due-west point, then the rings behind it
    // (a river leaves only a thin strip on the near bank).
    let west = Hex::from_offset(CITY_COL - c.dist, CITY_ROW);
    let mut ring: Vec<Hex> = centre
        .within(c.dist + 3)
        .into_iter()
        .filter(|h| h.distance(centre) >= c.dist)
        .filter(|h| passable(&s.map, *h))
        .filter(|h| c.ground != Ground::River || h.to_offset().0 <= CITY_COL - 2)
        .collect();
    ring.sort_by_key(|h| (h.distance(centre), h.distance(west), h.q, h.r));
    let army = c.army();
    for (k, kind) in army.iter().enumerate() {
        spawn_unit(&mut s, PlayerId(0), *kind, ring[k]);
    }
    let tier = c.tier();
    if c.eng_n() > 0 {
        for tech in [TechId::Mathematics, TechId::Machinery, TechId::Metallurgy]
            .into_iter()
            .take(usize::from(tier))
        {
            s.player_mut(PlayerId(0)).techs.insert(tech);
        }
    }
    for at in ring.iter().take(c.eng_n()) {
        // The unit on this tile has bought Siege.
        let id = s
            .units_at(*at)
            .find(|u| u.owner == PlayerId(0) && is_military(u.kind))
            .map(|u| u.id);
        if let Some(id) = id {
            s.units.get_mut(&id).expect("unit").siege = true;
        }
    }
    if c.scout {
        // Put the distance-3 scout on an open city-ring tile. A city on hills blocks sight
        // from the next ring, so one tile ahead of the main force is not enough on that map.
        let scout_pos = centre
            .neighbors()
            .into_iter()
            .filter(|h| passable(&s.map, *h))
            .min_by_key(|h| (h.distance(west), h.q, h.r))
            .expect("city ring has a passable scout tile");
        spawn_unit(&mut s, PlayerId(0), UnitKind::Scout, scout_pos);
    }
    // Defenders: fortified, the first in the city, the rest on the far side of it.
    let kind = if c.garrison_ranged {
        c.era.ranged()
    } else {
        c.era.melee()
    };
    let mut behind: Vec<Hex> = centre
        .ring(1)
        .into_iter()
        .filter(|h| passable(&s.map, *h))
        .collect();
    behind.sort_by_key(|h| (std::cmp::Reverse(h.distance(west)), h.q, h.r));
    for k in 0..c.garrison {
        let pos = if k == 0 { centre } else { behind[k - 1] };
        let uid = spawn_unit(&mut s, PlayerId(1), kind, pos);
        s.units.get_mut(&uid).unwrap().fortify = FORTIFY_MAX;
    }
    (s, id)
}

/// "Storm now": melee and mounted units walk into the city every turn whatever their health;
/// shooters and siege fire at the city tile (or the nearest enemy unit) from where they can.
fn storm_orders(s: &GameState, side: PlayerId, city: legatum_core::CityId) -> Vec<Order> {
    let view = legatum_core::player_view(s, side);
    let city_pos = s.cities[&city].pos;
    // The garrison of the home capital stays home; it is not part of the assault.
    let at_home = |pos: Hex| {
        s.cities
            .values()
            .any(|c| c.owner == side && c.is_capital && c.pos == pos)
    };
    let own: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner == side && u.military && u.kind != UnitKind::Scout)
        .filter(|u| !at_home(u.pos))
        .collect();
    let foes: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner != side && u.military)
        .collect();
    let clear = |from: Hex, to: Hex| los::visible(from, to, |h| s.map.get(h).map(|t| t.terrain));
    let mut orders = Vec::new();
    for u in &own {
        if u.ranged == 0 {
            orders.push(Order::MoveUnit {
                unit: u.id,
                to: city_pos,
            });
            continue;
        }
        if u.pos.distance(city_pos) <= u.range && clear(u.pos, city_pos) {
            orders.push(Order::RangedAttack {
                unit: u.id,
                target: city_pos,
            });
            continue;
        }
        if let Some(e) = foes
            .iter()
            .filter(|e| u.pos.distance(e.pos) <= u.range && clear(u.pos, e.pos))
            .min_by_key(|e| (e.hp, e.id))
        {
            orders.push(Order::RangedAttack {
                unit: u.id,
                target: e.pos,
            });
            continue;
        }
        let spot = (1..=u.range)
            .flat_map(|r| city_pos.ring(r))
            .filter(|h| {
                passable(&s.map, *h)
                    && clear(*h, city_pos)
                    && !own.iter().any(|f| f.id != u.id && f.pos == *h)
                    && !foes.iter().any(|f| f.pos == *h)
            })
            .min_by_key(|h| (h.distance(u.pos), h.q, h.r));
        if let Some(to) = spot
            && to != u.pos
        {
            orders.push(Order::MoveUnit { unit: u.id, to });
        }
    }
    orders
}

fn policy_orders(
    s: &GameState,
    side: PlayerId,
    city: legatum_core::CityId,
    policy: Policy,
    ranged_city_damage: i32,
    started_city_hp: i32,
    attacker_brain: &ScriptedBrain,
) -> Vec<Order> {
    if policy == Policy::Ai {
        return attacker_brain.tactical_orders(&legatum_core::player_view(s, side), Some(city));
    }
    if policy == Policy::Training {
        let hard = attacker_brain.tactical_orders(&legatum_core::player_view(s, side), Some(city));
        let cfg = TRAINING_CFG.get().cloned().unwrap_or_default();
        let plans = TRAINING_PLANS.with(|p| p.borrow().clone());
        let mut result = legatum_ai::training::search_turn_with(
            s,
            BTreeMap::from([(side, hard)]),
            &cfg,
            &plans,
            |_| true,
            || 0,
            |_| {},
        );
        TRAINING_PLANS.with(|p| *p.borrow_mut() = std::mem::take(&mut result.plans));
        if let Some(out) = TRAINING_LOG.get() {
            use std::io::Write as _;
            let mut out = out.lock().expect("training log");
            for r in &result.records {
                let _ = writeln!(
                    out,
                    "{}",
                    legatum_ai::training::LogLine::Decision(Box::new(r.clone())).json()
                );
            }
        }
        return result.orders.remove(&side).unwrap_or_default();
    }
    if policy == Policy::Storm {
        return storm_orders(s, side, city);
    }
    let view = legatum_core::player_view(s, side);
    let pos = s.cities[&city].pos;
    let own: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner == side && u.military && u.kind != UnitKind::Scout)
        .filter(|u| {
            !s.cities
                .values()
                .any(|c| c.owner == side && c.is_capital && c.pos == u.pos)
        })
        .collect();
    let foes: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner != side && u.military)
        .collect();
    let commit = match policy {
        Policy::WaitShooters => ranged_city_damage >= started_city_hp / 2,
        Policy::Surround => {
            own.iter()
                .filter(|u| u.ranged == 0 && u.pos.distance(pos) == 1)
                .count()
                >= 3
        }
        Policy::FocusFire => true,
        _ => true,
    };
    if policy == Policy::WaitShooters && !commit {
        let mut orders = Vec::new();
        let clear =
            |from: Hex, to: Hex| los::visible(from, to, |h| s.map.get(h).map(|t| t.terrain));
        for u in own.iter().copied() {
            if u.ranged > 0 && u.pos.distance(pos) <= u.range && clear(u.pos, pos) {
                orders.push(Order::RangedAttack {
                    unit: u.id,
                    target: pos,
                });
            } else if u.ranged > 0 {
                let spot = (1..=u.range)
                    .flat_map(|r| pos.ring(r))
                    .filter(|h| {
                        passable(&s.map, *h)
                            && clear(*h, pos)
                            && !own.iter().any(|f| f.id != u.id && f.pos == *h)
                            && !foes.iter().any(|f| f.pos == *h)
                    })
                    .min_by_key(|h| (h.distance(u.pos), h.q, h.r));
                if let Some(to) = spot
                    && to != u.pos
                {
                    orders.push(Order::MoveUnit { unit: u.id, to });
                }
            } else if u.pos.distance(pos) > 2 {
                let spot = pos
                    .ring(2)
                    .into_iter()
                    .filter(|h| {
                        passable(&s.map, *h)
                            && !own.iter().any(|f| f.id != u.id && f.pos == *h)
                            && !foes.iter().any(|f| f.pos == *h)
                    })
                    .min_by_key(|h| (h.distance(u.pos), h.q, h.r));
                if let Some(to) = spot
                    && to != u.pos
                {
                    orders.push(Order::MoveUnit { unit: u.id, to });
                }
            }
        }
        return orders;
    }
    if policy == Policy::Surround && !commit {
        let mut orders = Vec::new();
        for u in own.iter().copied() {
            if u.ranged > 0 && u.pos.distance(pos) <= u.range {
                orders.push(Order::RangedAttack {
                    unit: u.id,
                    target: pos,
                });
            } else if u.ranged == 0 && u.pos.distance(pos) > 1 {
                let to = pos
                    .neighbors()
                    .into_iter()
                    .filter(|h| {
                        passable(&s.map, *h)
                            && !own.iter().any(|f| f.id != u.id && f.pos == *h)
                            && !foes.iter().any(|f| f.pos == *h)
                    })
                    .min_by_key(|h| (h.distance(u.pos), h.q, h.r));
                if let Some(to) = to {
                    orders.push(Order::MoveUnit { unit: u.id, to });
                }
            }
        }
        return orders;
    }
    let focus = (policy == Policy::FocusFire)
        .then(|| foes.iter().min_by_key(|u| (u.hp, u.id)).map(|u| u.pos))
        .flatten();
    let mut orders = Vec::new();
    for u in own {
        if u.ranged > 0 {
            let target = focus.unwrap_or(pos);
            if u.pos.distance(target) <= u.range {
                orders.push(Order::RangedAttack { unit: u.id, target });
            }
        } else {
            orders.push(Order::MoveUnit {
                unit: u.id,
                to: if policy == Policy::FocusFire {
                    focus.unwrap_or(pos)
                } else {
                    pos
                },
            });
        }
    }
    orders
}

#[derive(Default, Clone, Debug)]
struct Probe {
    city_visible: bool,
    force: usize,
    siege: Option<(i32, i32)>,
    strike: bool,
    assault_orders: usize,
}

struct Fight {
    captured: bool,
    /// Turns resolved when the city fell (1 = fell in the first turn), else `LIMIT`.
    turns: u32,
    attacker_lost: usize,
    defender_lost: usize,
    /// Production value of the attackers lost: dead units whole, survivors half their wounds.
    loss_cost: f64,
    first_attack: Option<u32>,
    probe: Probe,
    trace: Vec<String>,
    /// Siege research: the city reached 0 HP at some point (a breach, whether or not it fell).
    breach_turn: Option<u32>,
    /// City HP damage the attackers dealt.
    city_dmg: i32,
    /// Military Engineers captured or destroyed.
    engineer_lost: usize,
    /// Production the defender spent holding: units lost whole, plus units it added mid-fight.
    defender_cost: f64,
}

/// `--ai-params FILE`: candidate AI parameters for the attacker (the defender keeps the built-in
/// genes), so a forecast or assault change is measured against the same forced policies.
static ATTACKER_PARAMS: std::sync::OnceLock<legatum_ai::AiParams> = std::sync::OnceLock::new();
/// `--training-compare`: the training search's settings, and its log.
static TRAINING_CFG: std::sync::OnceLock<legatum_ai::training::TrainingConfig> =
    std::sync::OnceLock::new();
static TRAINING_LOG: std::sync::OnceLock<std::sync::Mutex<std::io::BufWriter<fs::File>>> =
    std::sync::OnceLock::new();

thread_local! {
    /// Training round 2 (c): the attacker's kept plans within one fight (`play` runs a fight
    /// start to finish on one thread and clears this first).
    static TRAINING_PLANS: std::cell::RefCell<Vec<legatum_ai::training::StoredPlan>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {
    /// `--params-compare`: the candidate's genes for the fight running on this thread.
    static CANDIDATE_PARAMS: std::cell::RefCell<Option<legatum_ai::AiParams>> =
        const { std::cell::RefCell::new(None) };
}

/// `--params-compare PARAMS OUT`: plain Hard against Hard with the genes in PARAMS on the fixed
/// cells (or `--stride` of them), same dice per repeat, in the shape of `--training-compare`
/// (the candidate in the `training_*` fields). Writes `OUT/params-compare.json`.
fn params_compare(args: &[String], path: &str, out: &str, static_defender: bool) {
    let repeats: u32 = arg(args, "--repeats").map_or(4, |v| v.parse().expect("repeats"));
    let stride: usize = arg(args, "--stride").map_or(1, |v| v.parse().expect("stride"));
    let seed0: u64 = arg(args, "--seed0").map_or(1000, |v| v.parse().expect("seed0"));
    let params: legatum_ai::AiParams =
        serde_json::from_str(&fs::read_to_string(path).expect("params file")).expect("params");
    params.validate().expect("params");
    fs::create_dir_all(out).expect("out dir");
    let cells: Vec<Cell> = fixed_cells().into_iter().step_by(stride.max(1)).collect();
    let start = Instant::now();
    let rows: Vec<(String, Vec<(Fight, Fight)>)> = cells
        .par_iter()
        .map(|c| {
            let fights = (0..repeats)
                .map(|k| {
                    let seed = seed0 + u64::from(k);
                    let base = play(c, seed, Policy::Ai, false, static_defender);
                    CANDIDATE_PARAMS.with(|p| *p.borrow_mut() = Some(params.clone()));
                    let cand = play(c, seed, Policy::Ai, false, static_defender);
                    CANDIDATE_PARAMS.with(|p| *p.borrow_mut() = None);
                    (base, cand)
                })
                .collect();
            (c.id(), fights)
        })
        .collect();
    let all: Vec<&(Fight, Fight)> = rows.iter().flat_map(|(_, f)| f).collect();
    let n = all.len().max(1) as f64;
    let rate = |pick: &dyn Fn(&(Fight, Fight)) -> &Fight| {
        all.iter().filter(|f| pick(f).captured).count() as f64 / n
    };
    let mean = |pick: &dyn Fn(&(Fight, Fight)) -> f64| all.iter().map(|f| pick(f)).sum::<f64>() / n;
    let summary = serde_json::json!({
        "params": path, "cells": cells.len(), "repeats": repeats, "fights": all.len(),
        "hard_capture": rate(&|f| &f.0), "training_capture": rate(&|f| &f.1),
        "both_captured": all.iter().filter(|f| f.0.captured && f.1.captured).count(),
        "only_hard": all.iter().filter(|f| f.0.captured && !f.1.captured).count(),
        "only_training": all.iter().filter(|f| !f.0.captured && f.1.captured).count(),
        "hard_loss_cost": mean(&|f| f.0.loss_cost), "training_loss_cost": mean(&|f| f.1.loss_cost),
        "hard_turns": mean(&|f| f64::from(f.0.turns)), "training_turns": mean(&|f| f64::from(f.1.turns)),
        "per_cell": rows.iter().map(|(id, f)| serde_json::json!({
            "cell": id,
            "hard": f.iter().filter(|x| x.0.captured).count(),
            "training": f.iter().filter(|x| x.1.captured).count(),
        })).collect::<Vec<_>>(),
        "seconds": start.elapsed().as_secs(),
    });
    fs::write(
        PathBuf::from(out).join("params-compare.json"),
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    println!("{}", serde_json::to_string(&summary).expect("json"));
}

/// `--training-compare OUT`: plain Hard against Training mode on the fixed cells (or `--stride`
/// of them), same dice per repeat. Writes `OUT/training-compare.json` and the training log.
fn training_compare(args: &[String], out: &str, static_defender: bool) {
    let repeats: u32 = arg(args, "--repeats").map_or(4, |v| v.parse().expect("repeats"));
    let stride: usize = arg(args, "--stride").map_or(1, |v| v.parse().expect("stride"));
    let seed0: u64 = arg(args, "--seed0").map_or(1000, |v| v.parse().expect("seed0"));
    let budget: u64 = arg(args, "--training-budget").map_or(2048, |v| v.parse().expect("budget"));
    fs::create_dir_all(out).expect("out dir");
    let mut cfg = legatum_ai::training::TrainingConfig {
        cap_ms: 0,
        max_rollouts_per_battle: budget,
        source: "lab".into(),
        ..Default::default()
    };
    // `--training-round2 [--training-confirm N]`: round 2's full-game check and kept plans.
    if args.iter().any(|a| a == "--training-round2") {
        cfg = cfg.round2();
    }
    if let Some(n) = arg(args, "--training-confirm") {
        cfg.confirm = n.parse().expect("confirm");
    }
    TRAINING_CFG.set(cfg).expect("cfg once");
    let _ = TRAINING_LOG.set(std::sync::Mutex::new(std::io::BufWriter::new(
        fs::File::create(PathBuf::from(out).join("training-lab.jsonl")).expect("log"),
    )));
    let cells: Vec<Cell> = fixed_cells().into_iter().step_by(stride.max(1)).collect();
    let start = Instant::now();
    let rows: Vec<(String, Vec<(Fight, Fight)>)> = cells
        .par_iter()
        .map(|c| {
            let fights = (0..repeats)
                .map(|k| {
                    let seed = seed0 + u64::from(k);
                    (
                        play(c, seed, Policy::Ai, false, static_defender),
                        play(c, seed, Policy::Training, false, static_defender),
                    )
                })
                .collect();
            (c.id(), fights)
        })
        .collect();
    let all: Vec<&(Fight, Fight)> = rows.iter().flat_map(|(_, f)| f).collect();
    let n = all.len().max(1) as f64;
    let rate = |pick: &dyn Fn(&(Fight, Fight)) -> &Fight| {
        all.iter().filter(|f| pick(f).captured).count() as f64 / n
    };
    let mean = |pick: &dyn Fn(&(Fight, Fight)) -> f64| all.iter().map(|f| pick(f)).sum::<f64>() / n;
    let both = all.iter().filter(|f| f.0.captured && f.1.captured).count();
    let only_hard = all.iter().filter(|f| f.0.captured && !f.1.captured).count();
    let only_training = all.iter().filter(|f| !f.0.captured && f.1.captured).count();
    let summary = serde_json::json!({
        "cells": cells.len(), "repeats": repeats, "fights": all.len(), "budget": budget,
        "hard_capture": rate(&|f| &f.0), "training_capture": rate(&|f| &f.1),
        "both_captured": both, "only_hard": only_hard, "only_training": only_training,
        "hard_loss_cost": mean(&|f| f.0.loss_cost), "training_loss_cost": mean(&|f| f.1.loss_cost),
        "hard_turns": mean(&|f| f64::from(f.0.turns)), "training_turns": mean(&|f| f64::from(f.1.turns)),
        "per_cell": rows.iter().map(|(id, f)| serde_json::json!({
            "cell": id,
            "hard": f.iter().filter(|x| x.0.captured).count(),
            "training": f.iter().filter(|x| x.1.captured).count(),
        })).collect::<Vec<_>>(),
        "seconds": start.elapsed().as_secs(),
    });
    if let Some(log) = TRAINING_LOG.get() {
        use std::io::Write as _;
        log.lock().expect("log").flush().expect("flush");
    }
    fs::write(
        PathBuf::from(out).join("training-compare.json"),
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    println!("{}", serde_json::to_string(&summary).expect("json"));
}

fn play(c: &Cell, seed: u64, policy: Policy, keep_trace: bool, static_defender: bool) -> Fight {
    TRAINING_PLANS.with(|p| p.borrow_mut().clear());
    let (mut s, city) = arena(c, seed);
    let attacker_faction = faction_of(&c.faction);
    let mut brains = [
        match CANDIDATE_PARAMS
            .with(|c| c.borrow().clone())
            .as_ref()
            .or(ATTACKER_PARAMS.get())
        {
            Some(params) => ScriptedBrain::with_params(
                attacker_faction,
                seed ^ 11,
                PlayerId(0),
                AiDifficulty::Hard,
                params,
            ),
            None => ScriptedBrain::with_difficulty(
                attacker_faction,
                seed ^ 11,
                PlayerId(0),
                AiDifficulty::Hard,
            ),
        },
        ScriptedBrain::with_difficulty(Faction::India, seed ^ 29, PlayerId(1), AiDifficulty::Hard),
    ];
    let home = |owner: PlayerId| {
        s.cities
            .values()
            .find(|c| c.owner == owner && c.is_capital)
            .map(|c| c.pos)
    };
    let initial_att: Vec<_> = s
        .units_of(PlayerId(0))
        .filter(|u| Some(u.pos) != home(PlayerId(0)))
        .map(|u| (u.id, data::unit_stats_for(attacker_faction, u.kind).cost))
        .collect();
    let initial_def: Vec<_> = s
        .units_of(PlayerId(1))
        .filter(|u| Some(u.pos) != home(PlayerId(1)))
        .map(|u| u.id)
        .collect();
    let def_cost_of = |s: &GameState, id: UnitId| {
        s.units
            .get(&id)
            .map_or(0, |u| data::unit_stats_for(Faction::India, u.kind).cost)
    };
    let initial_def_cost: Vec<i32> = initial_def.iter().map(|id| def_cost_of(&s, *id)).collect();
    let def_home = home(PlayerId(1));
    // The attacker's Siege units: how many of them the assault loses.
    let engineers: Vec<UnitId> = s
        .units_of(PlayerId(0))
        .filter(|u| u.siege)
        .map(|u| u.id)
        .collect();
    let mut breach_turn = None;
    let mut city_dmg = 0;
    let mut probe = Probe::default();
    if policy == Policy::Ai {
        let view = legatum_core::player_view(&s, PlayerId(0));
        if let Some(p) = brains[0].probe_city_assault(&view, city) {
            probe = Probe {
                city_visible: p.city_visible,
                force: p.force,
                siege: p.siege,
                strike: p.strike,
                assault_orders: p.assault_orders,
            };
        }
    }
    let mut trace = Vec::new();
    let mut turns = 0;
    let mut first_attack = None;
    let started_city_hp = s.cities[&city].hp;
    let mut ranged_city_damage = 0;
    while turns < LIMIT {
        let d_view = legatum_core::player_view(&s, PlayerId(1));
        let d_city_orders = if static_defender {
            Vec::new()
        } else {
            brains[1]
                .orders(&d_view)
                .into_iter()
                .filter(|o| {
                    matches!(o,
                Order::SetProduction { city: id, .. } | Order::SetProductionQueue { city: id, .. }
                    | Order::BuyItem { city: id, .. } if *id == city)
                })
                .collect()
        };
        if !d_city_orders.is_empty() {
            let mut events = Vec::new();
            legatum_core::rules::cities::apply_city_orders(
                &mut s,
                PlayerId(1),
                &d_city_orders,
                &mut events,
            );
        }
        let mut orders = BTreeMap::new();
        orders.insert(
            PlayerId(0),
            policy_orders(
                &s,
                PlayerId(0),
                city,
                policy,
                ranged_city_damage,
                started_city_hp,
                &brains[0],
            ),
        );
        orders.insert(PlayerId(1), brains[1].tactical_orders(&d_view, None));
        let (next, events) = resolve_tactical_turn(&s, &orders);
        turns += 1;
        if first_attack.is_none()
            && events.iter().any(|e| {
                matches!(
                    e,
                    Event::Battle {
                        attacker_owner: PlayerId(0),
                        defender: Defender::City { .. }
                            | Defender::Unit {
                                owner: PlayerId(1),
                                ..
                            },
                        ..
                    }
                )
            })
        {
            first_attack = Some(turns);
        }
        if keep_trace {
            let mut line = format!("turn {turns}: city hp {}", s.cities[&city].hp);
            for u in s.units.values() {
                let _ = write!(
                    line,
                    " | P{} {:?} hp{} @{},{}",
                    u.owner.0, u.kind, u.hp, u.pos.q, u.pos.r
                );
            }
            let _ = write!(line, "\n  orders {orders:?}");
            for e in &events {
                if matches!(
                    e,
                    Event::Battle { .. } | Event::CityCaptured { .. } | Event::UnitDestroyed { .. }
                ) {
                    let _ = write!(line, "\n  {e:?}");
                }
            }
            trace.push(line);
        }
        ranged_city_damage += events
            .iter()
            .map(|event| match event {
                Event::Battle {
                    attacker_owner: PlayerId(0),
                    defender: Defender::City { city: target, .. },
                    damage_to_defender,
                    ranged: true,
                    ..
                } if *target == city => *damage_to_defender,
                _ => 0,
            })
            .sum::<i32>();
        for event in &events {
            if let Event::Battle {
                attacker,
                attacker_owner: PlayerId(0),
                defender: Defender::City { city: target, .. },
                damage_to_defender,
                ..
            } = event
                && *target == city
            {
                let _ = attacker;
                city_dmg += *damage_to_defender;
            }
        }
        if breach_turn.is_none()
            && (next.cities[&city].hp <= 0 || next.cities[&city].owner == PlayerId(0))
        {
            breach_turn = Some(turns);
        }
        s = next;
        if !static_defender {
            // `resolve_tactical_turn` advances the turn after tactical phases. In the full
            // resolver city processing happens before that increment, so keep the just-resolved
            // turn while applying city healing, production, income and upkeep.
            s.turn -= 1;
            let mut economy_events = Vec::new();
            legatum_core::rules::cities::process_cities(&mut s, &mut economy_events);
            s.turn += 1;
        }
        let captured = s.cities[&city].owner == PlayerId(0);
        let attackers_left = initial_att
            .iter()
            .any(|(id, _)| s.units.get(id).is_some_and(|u| is_military(u.kind)));
        if captured || !attackers_left {
            break;
        }
    }
    let captured = s.cities[&city].owner == PlayerId(0);
    let mut loss_cost = 0.0;
    let mut attacker_lost = 0;
    for (id, cost) in &initial_att {
        match s.units.get(id) {
            Some(u) => loss_cost += f64::from(*cost) * f64::from(100 - u.hp) / 200.0,
            None => {
                attacker_lost += 1;
                loss_cost += f64::from(*cost);
            }
        }
    }
    let defender_lost = initial_def
        .iter()
        .filter(|id| !s.units.contains_key(id))
        .count();
    let mut defender_cost = 0.0;
    for (id, cost) in initial_def.iter().zip(&initial_def_cost) {
        if !s.units.contains_key(id) {
            defender_cost += f64::from(*cost);
        }
    }
    for u in s.units_of(PlayerId(1)) {
        if is_military(u.kind) && !initial_def.contains(&u.id) && Some(u.pos) != def_home {
            defender_cost += f64::from(data::unit_stats_for(Faction::India, u.kind).cost);
        }
    }
    let engineer_lost = engineers
        .iter()
        .filter(|id| s.units.get(id).is_none_or(|u| u.owner != PlayerId(0)))
        .count();
    Fight {
        captured,
        turns: if captured { turns } else { LIMIT },
        attacker_lost,
        defender_lost,
        loss_cost,
        first_attack,
        probe,
        trace,
        breach_turn,
        city_dmg,
        engineer_lost,
        defender_cost,
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Stats {
    runs: u32,
    captures: u32,
    capture_turns: u32,
    attacker_lost: u32,
    defender_lost: u32,
    loss_cost: f64,
    engaged: u32,
    breaches: u32,
    breach_turns: u32,
    city_dmg: i64,
    engineer_lost: u32,
    defender_cost: f64,
}
impl Stats {
    fn add(&mut self, f: &Fight) {
        self.runs += 1;
        self.captures += u32::from(f.captured);
        if f.captured {
            self.capture_turns += f.turns;
        }
        self.attacker_lost += f.attacker_lost as u32;
        self.defender_lost += f.defender_lost as u32;
        self.loss_cost += f.loss_cost;
        self.engaged += u32::from(f.first_attack.is_some());
        if let Some(t) = f.breach_turn {
            self.breaches += 1;
            self.breach_turns += t;
        }
        self.city_dmg += i64::from(f.city_dmg);
        self.engineer_lost += f.engineer_lost as u32;
        self.defender_cost += f.defender_cost;
    }
    fn breach_rate(&self) -> f64 {
        f64::from(self.breaches) / f64::from(self.runs.max(1))
    }
    fn mean_breach_turns(&self) -> Option<f64> {
        (self.breaches > 0).then(|| f64::from(self.breach_turns) / f64::from(self.breaches))
    }
    /// The siege research columns of one policy.
    fn siege_csv(&self) -> String {
        let n = f64::from(self.runs.max(1));
        format!(
            "{:.4},{},{:.1},{:.2},{:.1}",
            self.breach_rate(),
            self.mean_breach_turns()
                .map_or(String::new(), |x| format!("{x:.2}")),
            self.city_dmg as f64 / n,
            f64::from(self.engineer_lost) / n,
            self.defender_cost / n,
        )
    }
    fn rate(&self) -> f64 {
        f64::from(self.captures) / f64::from(self.runs.max(1))
    }
    fn mean_turns(&self) -> Option<f64> {
        (self.captures > 0).then(|| f64::from(self.capture_turns) / f64::from(self.captures))
    }
    fn mean(&self, v: f64) -> f64 {
        v / f64::from(self.runs.max(1))
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CellResult {
    cell: Cell,
    ai: Stats,
    storm: Stats,
    wait_shooters: Stats,
    surround: Stats,
    focus_fire: Stats,
    /// The AI's own expectation at the start (identical across seeds apart from lapses).
    fc_capture: bool,
    fc_turns: Option<i32>,
    fc_losses: Option<i32>,
    fc_force: usize,
    fc_visible: bool,
    /// Share of AI runs where the strike forecast pays for the war, and where the assault
    /// check issued orders on turn 1.
    strike_rate: f64,
    commit_rate: f64,
}

fn run_cell(cell: &Cell, repeats: u32, seed0: u64, static_defender: bool) -> CellResult {
    let mut ai = Stats::default();
    let mut storm = Stats::default();
    let mut wait_shooters = Stats::default();
    let mut surround = Stats::default();
    let mut focus_fire = Stats::default();
    let mut strike = 0;
    let mut commit = 0;
    let mut first: Option<Probe> = None;
    for k in 0..repeats {
        // Dice vary with every repeat; both policies see the same dice seed.
        let seed = seed0 + u64::from(k);
        let a = play(cell, seed, Policy::Ai, false, static_defender);
        ai.add(&a);
        strike += u32::from(a.probe.strike);
        commit += u32::from(a.probe.assault_orders > 0);
        if first.is_none() {
            first = Some(a.probe.clone());
        }
        storm.add(&play(cell, seed, Policy::Storm, false, static_defender));
        wait_shooters.add(&play(
            cell,
            seed,
            Policy::WaitShooters,
            false,
            static_defender,
        ));
        surround.add(&play(cell, seed, Policy::Surround, false, static_defender));
        focus_fire.add(&play(cell, seed, Policy::FocusFire, false, static_defender));
    }
    let p = first.unwrap_or_default();
    CellResult {
        cell: cell.clone(),
        ai,
        storm,
        wait_shooters,
        surround,
        focus_fire,
        fc_capture: p.siege.is_some(),
        fc_turns: p.siege.map(|s| s.0),
        fc_losses: p.siege.map(|s| s.1),
        fc_force: p.force,
        fc_visible: p.city_visible,
        strike_rate: f64::from(strike) / f64::from(repeats.max(1)),
        commit_rate: f64::from(commit) / f64::from(repeats.max(1)),
    }
}

struct Filters {
    eras: Vec<Era>,
    factions: Vec<String>,
    grounds: Vec<Ground>,
    hps: Vec<i32>,
    ns: Vec<usize>,
}

fn grid(f: &Filters) -> Vec<Cell> {
    let mut out = Vec::new();
    for &era in &f.eras {
        for faction in &f.factions {
            for &n in &f.ns {
                for mix in [Mix::Melee, Mix::Mixed, Mix::Shooters, Mix::Mounted] {
                    // A 2-unit force has no distinct shooter-heavy mix.
                    if n == 2 && mix == Mix::Shooters {
                        continue;
                    }
                    for walls in [false, true] {
                        for (garrison, garrison_ranged) in [
                            (0, false),
                            (1, false),
                            (2, false),
                            (3, false),
                            (1, true),
                            (2, true),
                            (3, true),
                        ] {
                            for &city_hp in &f.hps {
                                for &ground in &f.grounds {
                                    for dist in 1..=3 {
                                        if ground == Ground::River && dist == 1 {
                                            continue;
                                        }
                                        let scouts: &[bool] =
                                            if dist == 3 { &[false, true] } else { &[false] };
                                        for &scout in scouts {
                                            out.push(Cell {
                                                era,
                                                faction: faction.clone(),
                                                mix,
                                                n,
                                                walls,
                                                fort: 0,
                                                garrison,
                                                garrison_ranged,
                                                city_hp,
                                                ground,
                                                dist,
                                                scout,
                                                variant: String::new(),
                                                eng_tier: 0,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// The fixed regression subset: about 20 cells that cover the policy-relevant corners,
/// including the known Sparta melee case (2 to 4 Hoplites against an unwalled city).
fn fixed_cells() -> Vec<Cell> {
    let c =
        |era, faction: &str, mix, n, walls, garrison, garrison_ranged, city_hp, ground, dist| {
            Cell {
                era,
                faction: faction.into(),
                mix,
                n,
                walls,
                fort: 0,
                garrison,
                garrison_ranged,
                city_hp,
                ground,
                dist,
                scout: dist == 3,
                variant: String::new(),
                eng_tier: 0,
            }
        };
    use Era::*;
    use Ground::*;
    use Mix::*;
    vec![
        // The Sparta storm: Hoplites against unwalled cities with 0, 1 and 2 defenders.
        c(Ancient, "sparta", Melee, 2, false, 1, false, 200, Flat, 2),
        c(Ancient, "sparta", Melee, 3, false, 1, false, 200, Flat, 2),
        c(Ancient, "sparta", Melee, 4, false, 1, false, 200, Flat, 2),
        c(Ancient, "sparta", Melee, 4, false, 2, true, 200, Flat, 2),
        c(Ancient, "sparta", Melee, 4, true, 2, true, 200, Flat, 2),
        c(Ancient, "sparta", Melee, 6, true, 3, true, 200, Flat, 2),
        // The same armies without Sparta's wound immunity.
        c(Ancient, "generic", Melee, 3, false, 1, false, 200, Flat, 2),
        c(Ancient, "generic", Melee, 4, false, 2, true, 200, Flat, 2),
        c(Ancient, "generic", Melee, 4, true, 2, true, 200, Flat, 2),
        // Mixed and shooter-heavy forces.
        c(Ancient, "generic", Mixed, 4, true, 2, false, 200, Flat, 3),
        c(
            Ancient, "generic", Shooters, 4, false, 1, false, 200, Flat, 3,
        ),
        c(Ancient, "generic", Shooters, 6, true, 2, true, 200, Flat, 3),
        // Mongols, mounted and otherwise.
        c(Ancient, "mongols", Mounted, 4, false, 1, true, 200, Flat, 3),
        c(Classical, "mongols", Mixed, 4, true, 2, true, 200, Flat, 3),
        // Terrain and a damaged city.
        c(Ancient, "generic", Melee, 4, false, 1, false, 200, Hills, 2),
        c(Ancient, "generic", Melee, 4, false, 1, false, 200, River, 3),
        c(Ancient, "sparta", Melee, 3, true, 1, false, 100, Flat, 1),
        // Later eras.
        c(Classical, "sparta", Melee, 4, false, 2, true, 200, Flat, 2),
        c(Classical, "generic", Mixed, 6, true, 3, false, 200, Flat, 2),
        c(
            Renaissance,
            "generic",
            Mixed,
            4,
            true,
            2,
            true,
            200,
            Flat,
            3,
        ),
        c(
            Renaissance,
            "sparta",
            Melee,
            3,
            false,
            1,
            false,
            200,
            Flat,
            1,
        ),
    ]
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1).cloned())
}
fn list(args: &[String], key: &str, default: &[&str]) -> Vec<String> {
    arg(args, key)
        .map(|v| v.split(',').map(str::to_owned).collect())
        .unwrap_or_else(|| default.iter().map(|s| (*s).to_owned()).collect())
}

const CSV_HEAD: &str = "cell,era,faction,mix,n,walls,fort,garrison,garrison_type,city_hp,ground,dist,scout,\
variant,eng_n,eng_tier,army_cost,\
runs,fc_capture,fc_turns,fc_losses,fc_force,fc_visible,strike_rate,commit_rate,\
ai_capture,ai_turns,ai_att_lost,ai_def_lost,ai_loss_cost,ai_engaged,\
storm_capture,storm_turns,storm_att_lost,storm_def_lost,storm_loss_cost,\
wait_capture,wait_turns,wait_att_lost,wait_def_lost,wait_loss_cost,\
surround_capture,surround_turns,surround_att_lost,surround_def_lost,surround_loss_cost,\
focus_capture,focus_turns,focus_att_lost,focus_def_lost,focus_loss_cost,\
ai_breach,ai_breach_turns,ai_city_dmg,ai_eng_lost,ai_def_cost,\
storm_breach,storm_breach_turns,storm_city_dmg,storm_eng_lost,storm_def_cost,\
wait_breach,wait_breach_turns,wait_city_dmg,wait_eng_lost,wait_def_cost,\
surround_breach,surround_breach_turns,surround_city_dmg,surround_eng_lost,surround_def_cost,\
focus_breach,focus_breach_turns,focus_city_dmg,focus_eng_lost,focus_def_cost";

fn csv_row(r: &CellResult) -> String {
    let c = &r.cell;
    let opt = |v: Option<f64>| v.map_or(String::new(), |x| format!("{x:.2}"));
    let optn = |v: Option<i32>| v.map_or(String::new(), |x| x.to_string());
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.3},{:.3},{:.4},{},{:.2},{:.2},{:.1},{:.3},{:.4},{},{:.2},{:.2},{:.1},{:.4},{},{:.2},{:.2},{:.1},{:.4},{},{:.2},{:.2},{:.1},{:.4},{},{:.2},{:.2},{:.1},{},{},{},{},{}",
        c.id(),
        c.era.label(),
        c.faction,
        c.mix.label(),
        c.n,
        c.wall_tier(),
        c.fort,
        c.garrison,
        if c.garrison == 0 {
            "none"
        } else if c.garrison_ranged {
            "ranged"
        } else {
            "melee"
        },
        c.city_hp,
        c.ground.label(),
        c.dist,
        u8::from(c.scout),
        if c.variant.is_empty() {
            "base"
        } else {
            &c.variant
        },
        c.eng_n(),
        c.tier(),
        c.army_cost(),
        r.ai.runs,
        u8::from(r.fc_capture),
        optn(r.fc_turns),
        optn(r.fc_losses),
        r.fc_force,
        u8::from(r.fc_visible),
        r.strike_rate,
        r.commit_rate,
        r.ai.rate(),
        opt(r.ai.mean_turns()),
        r.ai.mean(f64::from(r.ai.attacker_lost)),
        r.ai.mean(f64::from(r.ai.defender_lost)),
        r.ai.mean(r.ai.loss_cost),
        f64::from(r.ai.engaged) / f64::from(r.ai.runs.max(1)),
        r.storm.rate(),
        opt(r.storm.mean_turns()),
        r.storm.mean(f64::from(r.storm.attacker_lost)),
        r.storm.mean(f64::from(r.storm.defender_lost)),
        r.storm.mean(r.storm.loss_cost),
        r.wait_shooters.rate(),
        opt(r.wait_shooters.mean_turns()),
        r.wait_shooters
            .mean(f64::from(r.wait_shooters.attacker_lost)),
        r.wait_shooters
            .mean(f64::from(r.wait_shooters.defender_lost)),
        r.wait_shooters.mean(r.wait_shooters.loss_cost),
        r.surround.rate(),
        opt(r.surround.mean_turns()),
        r.surround.mean(f64::from(r.surround.attacker_lost)),
        r.surround.mean(f64::from(r.surround.defender_lost)),
        r.surround.mean(r.surround.loss_cost),
        r.focus_fire.rate(),
        opt(r.focus_fire.mean_turns()),
        r.focus_fire.mean(f64::from(r.focus_fire.attacker_lost)),
        r.focus_fire.mean(f64::from(r.focus_fire.defender_lost)),
        r.focus_fire.mean(r.focus_fire.loss_cost),
        r.ai.siege_csv(),
        r.storm.siege_csv(),
        r.wait_shooters.siege_csv(),
        r.surround.siege_csv(),
        r.focus_fire.siege_csv(),
    )
}

/// The Military Engineer grid (`--siege`): cities two hexes out on flat ground at 200 HP with no
/// Walls, Walls, a Castle or a Star Fort, against armies with and without Engineers, under every
/// rule variant. Engineers need Mathematics, so the Ancient era plays the variant `base` only;
/// the Castle and Star Fort cities are Renaissance cells, played at each Engineer siege tier.
/// Mounted units outrun an Engineer's two moves and are not played. Open cities are played with
/// `base` alone (nothing in an Engineer variant touches them).
fn siege_grid(f: &Filters, variants: &[String]) -> Vec<Cell> {
    let mut out = Vec::new();
    for &era in &f.eras {
        for faction in &f.factions {
            for &n in &f.ns {
                for mix in [Mix::Melee, Mix::Mixed, Mix::Shooters] {
                    if n == 2 && mix == Mix::Shooters {
                        continue;
                    }
                    for variant in variants {
                        let eng = variant_eng(variant);
                        if eng != Eng::None && era == Era::Ancient {
                            continue;
                        }
                        // (Walls tier above plain Walls, Engineer tier) pairs to play.
                        let setups: Vec<(bool, u8, u8)> = {
                            let mut v = Vec::new();
                            if eng == Eng::None {
                                v.push((false, 0, 0));
                            }
                            v.push((true, 0, 0));
                            if era == Era::Renaissance {
                                for fort in 1..=2u8 {
                                    if eng == Eng::None {
                                        v.push((true, fort, 0));
                                    } else {
                                        for tier in 1..=3u8 {
                                            v.push((true, fort, tier));
                                        }
                                    }
                                }
                            }
                            v
                        };
                        for (walls, fort, eng_tier) in setups {
                            for (garrison, garrison_ranged) in
                                [(0, false), (1, false), (1, true), (2, true)]
                            {
                                for &ground in &f.grounds {
                                    out.push(Cell {
                                        era,
                                        faction: faction.clone(),
                                        mix,
                                        n,
                                        walls,
                                        fort,
                                        garrison,
                                        garrison_ranged,
                                        city_hp: 200,
                                        ground,
                                        dist: 2,
                                        scout: false,
                                        variant: if variant == "base" {
                                            String::new()
                                        } else {
                                            variant.clone()
                                        },
                                        eng_tier,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// The regression check: compare a fresh fixed-subset run with the committed baseline.
fn check(results: &[CellResult], baseline: &[CellResult]) -> Result<(), String> {
    let mut problems = Vec::new();
    let by_id: BTreeMap<String, &CellResult> = baseline.iter().map(|r| (r.cell.id(), r)).collect();
    let (mut now_sum, mut base_sum, mut n) = (0.0, 0.0, 0.0);
    for r in results {
        let Some(b) = by_id.get(&r.cell.id()) else {
            problems.push(format!("{}: not in the baseline", r.cell.id()));
            continue;
        };
        n += 1.0;
        now_sum += r.ai.rate();
        base_sum += b.ai.rate();
        let drop = b.ai.rate() - r.ai.rate();
        if drop > 0.34 {
            problems.push(format!(
                "{}: AI capture rate fell from {:.0}% to {:.0}%",
                r.cell.id(),
                100.0 * b.ai.rate(),
                100.0 * r.ai.rate()
            ));
        }
        for (name, before, now) in [
            ("storm", b.storm.rate(), r.storm.rate()),
            ("wait", b.wait_shooters.rate(), r.wait_shooters.rate()),
            ("surround", b.surround.rate(), r.surround.rate()),
            ("focus", b.focus_fire.rate(), r.focus_fire.rate()),
        ] {
            if (before - now).abs() > 0.34 {
                problems.push(format!(
                    "{}: {name} capture rate moved from {:.0}% to {:.0}% (a rules change?)",
                    r.cell.id(),
                    100.0 * before,
                    100.0 * now
                ));
            }
        }
    }
    if n > 0.0 && (base_sum - now_sum) / n > 0.05 {
        problems.push(format!(
            "mean AI capture rate fell from {:.1}% to {:.1}%",
            100.0 * base_sum / n,
            100.0 * now_sum / n
        ));
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let static_defender = args.iter().any(|a| a == "--static-defender");
    if let Some(path) = arg(&args, "--ai-params") {
        let text = fs::read_to_string(&path).expect("--ai-params file");
        let params: legatum_ai::AiParams = serde_json::from_str(&text).expect("--ai-params");
        params.validate().expect("--ai-params");
        ATTACKER_PARAMS.set(params).expect("params set once");
    }
    if let Some(i) = args.iter().position(|a| a == "--params-compare") {
        params_compare(&args, &args[i + 1], &args[i + 2], static_defender);
        return;
    }
    if let Some(out) = arg(&args, "--training-compare") {
        training_compare(&args, &out, static_defender);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--trace") {
        let id = &args[i + 1];
        let seed: u64 = args[i + 2].parse().expect("seed");
        let policy = match args[i + 3].as_str() {
            "ai" => Policy::Ai,
            "storm" => Policy::Storm,
            "wait" => Policy::WaitShooters,
            "surround" => Policy::Surround,
            "focus" => Policy::FocusFire,
            other => panic!("policy {other}"),
        };
        let all = Filters {
            eras: Era::ALL.to_vec(),
            factions: vec!["generic".into(), "sparta".into(), "mongols".into()],
            grounds: vec![Ground::Flat, Ground::Hills, Ground::River],
            hps: vec![200, 100],
            ns: vec![2, 3, 4, 6],
        };
        let every_variant: Vec<String> = VARIANTS.iter().map(|(n, _, _)| (*n).to_owned()).collect();
        let cell = grid(&all)
            .into_iter()
            .chain(fixed_cells())
            .chain(siege_grid(&all, &every_variant))
            .find(|c| &c.id() == id)
            .expect("unknown cell id");
        let f = play(&cell, seed, policy, true, static_defender);
        for line in &f.trace {
            println!("{line}");
        }
        println!(
            "captured {} turns {} attacker_lost {} defender_lost {} probe {:?}",
            f.captured, f.turns, f.attacker_lost, f.defender_lost, f.probe
        );
        return;
    }
    let fixed = arg(&args, "--subset").as_deref() == Some("fixed");
    let siege = args.iter().any(|a| a == "--siege");
    let variants: Vec<String> = list(
        &args,
        "--variants",
        &VARIANTS.iter().map(|(n, _, _)| *n).collect::<Vec<_>>(),
    );
    let repeats: u32 = arg(&args, "--repeats")
        .map(|v| v.parse().expect("repeats"))
        .unwrap_or(if fixed { 12 } else { 24 });
    let seed0: u64 = arg(&args, "--seed0")
        .map(|v| v.parse().expect("seed0"))
        .unwrap_or(1000);
    let threads: usize = arg(&args, "--threads")
        .map(|v| v.parse().expect("threads"))
        .unwrap_or(0);
    let stride: usize = arg(&args, "--stride")
        .map(|v| v.parse().expect("stride"))
        .unwrap_or(1);
    let filters = Filters {
        eras: list(&args, "--eras", &["ancient", "classical", "renaissance"])
            .iter()
            .map(|e| match e.as_str() {
                "ancient" => Era::Ancient,
                "classical" => Era::Classical,
                "renaissance" => Era::Renaissance,
                other => panic!("era {other}"),
            })
            .collect(),
        factions: list(
            &args,
            "--factions",
            if siege {
                &["generic", "sparta"]
            } else {
                &["generic", "sparta", "mongols"]
            },
        ),
        grounds: list(
            &args,
            "--ground",
            if siege {
                &["flat"]
            } else {
                &["flat", "hills", "river"]
            },
        )
        .iter()
        .map(|g| match g.as_str() {
            "flat" => Ground::Flat,
            "hills" => Ground::Hills,
            "river" => Ground::River,
            other => panic!("ground {other}"),
        })
        .collect(),
        hps: list(&args, "--hp", &["200", "100"])
            .iter()
            .map(|h| h.parse().expect("hp"))
            .collect(),
        ns: list(
            &args,
            "--n",
            if siege {
                &["3", "4", "6"]
            } else {
                &["2", "3", "4", "6"]
            },
        )
        .iter()
        .map(|h| h.parse().expect("n"))
        .collect(),
    };
    let cells: Vec<Cell> = if fixed {
        fixed_cells()
    } else if siege {
        siege_grid(&filters, &variants)
            .into_iter()
            .step_by(stride.max(1))
            .collect()
    } else {
        grid(&filters).into_iter().step_by(stride.max(1)).collect()
    };
    if threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("thread pool");
    }
    eprintln!(
        "combat lab: {} cells x {repeats} repeats x 5 policies = {} fights",
        cells.len(),
        cells.len() as u64 * u64::from(repeats) * 5
    );
    let started = Instant::now();
    let results: Vec<CellResult> = cells
        .par_iter()
        .map(|cell| run_cell(cell, repeats, seed0, static_defender))
        .collect();
    let wall = started.elapsed().as_secs_f64();
    let fights = results.len() as u64 * u64::from(repeats) * 5;
    let used = rayon::current_num_threads();
    eprintln!(
        "done: {fights} fights in {wall:.1}s on {used} threads ({:.2}s of wall per thousand fights, {:.0} ms of CPU per fight)",
        wall * 1000.0 / fights as f64,
        wall * used as f64 * 1000.0 / fights as f64
    );
    if let Some(path) = arg(&args, "--write-baseline") {
        fs::write(&path, serde_json::to_string_pretty(&results).unwrap()).expect("baseline");
        eprintln!("baseline written to {path}");
    }
    if let Some(dir) = arg(&args, "--out") {
        let dir = PathBuf::from(dir);
        fs::create_dir_all(&dir).expect("out dir");
        let mut csv = String::from(CSV_HEAD);
        csv.push('\n');
        for r in &results {
            csv.push_str(&csv_row(r));
            csv.push('\n');
        }
        fs::write(dir.join("cells.csv"), csv).expect("cells.csv");
        let summary = serde_json::json!({
            "cells": results.len(),
            "repeats": repeats,
            "policies": ["ai", "storm", "wait_shooters", "surround", "focus_fire"],
            "static_defender": static_defender,
            "fights": fights,
            "wall_seconds": wall,
            "threads": used,
            "wall_seconds_per_thousand_fights": wall * 1000.0 / fights as f64,
            "cpu_ms_per_fight": wall * used as f64 * 1000.0 / fights as f64,
            "seed0": seed0,
            "turn_limit": LIMIT,
            "siege_grid": siege,
            "variants": if siege { variants.clone() } else { Vec::new() },
        });
        fs::write(
            dir.join("summary.json"),
            serde_json::to_string_pretty(&summary).unwrap(),
        )
        .expect("summary.json");
    }
    if let Some(path) = arg(&args, "--check") {
        let baseline: Vec<CellResult> =
            serde_json::from_str(&fs::read_to_string(&path).expect("baseline file"))
                .expect("baseline json");
        match check(&results, &baseline) {
            Ok(()) => eprintln!("combat lab check: ok ({} cells)", results.len()),
            Err(why) => {
                eprintln!("combat lab check FAILED:\n{why}");
                std::process::exit(1);
            }
        }
    }
}
