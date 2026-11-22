use crate::report::{
    self, Behavior, Context, Counts, Incident, LocalCity, LocalUnit, Report, Seat, Turn,
};
use legatum_core::{
    AttackMode, CityId, Event, Faction, GameState, Hex, Order, PlayerId, PlayerView,
    ProductionItem, UnitId, UnitKind,
    events::Defender,
    forecast_attack_mode, player_view,
    rules::{hostile, is_military, units::order_unit},
    state::Unit,
};
use legatum_wasm::session::OrderMap;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Config {
    /// Observed unfounded turns. Anchored saves do not invent the unit's earlier age.
    pub settler_turns: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self { settler_turns: 12 }
    }
}

pub(crate) struct Detector {
    report: Report,
    counts: BTreeMap<(Faction, Behavior, Option<Faction>), Counts>,
    streaks: BTreeMap<(PlayerId, Behavior, u32), (u32, u32)>,
    positions: BTreeMap<(PlayerId, UnitId), (u32, VecDeque<Hex>)>,
}

struct Frame<'a> {
    before: &'a GameState,
    after: &'a GameState,
    view: &'a PlayerView,
    orders: &'a [Order],
    valid: BTreeSet<usize>,
    events: &'a [Event],
}

impl Frame<'_> {
    fn enemies(&self) -> impl Iterator<Item = &legatum_core::view::UnitView> {
        self.view
            .units
            .iter()
            .filter(|u| u.military && hostile(self.before, self.view.me, u.owner))
    }

    fn responded(&self, unit: &Unit) -> bool {
        if self.events.iter().any(|e| match e {
            Event::UnitMoved { unit: id, .. } => *id == unit.id,
            Event::Battle {
                attacker,
                defender,
                damage_to_attacker,
                ..
            } => {
                *attacker == unit.id
                    || (*damage_to_attacker > 0
                        && matches!(defender, Defender::Unit { unit: id, .. } if *id == unit.id))
            }
            _ => false,
        }) {
            return true;
        }
        let orders: Vec<_> = self
            .orders
            .iter()
            .enumerate()
            .filter(|(i, o)| self.valid.contains(i) && order_unit(o) == Some(unit.id))
            .map(|(_, o)| o)
            .collect();
        if orders.is_empty() {
            return unit.goto.is_some_and(|h| h != unit.pos);
        }
        orders.iter().any(|o| {
            matches!(o, Order::MoveUnit { to, .. } if *to != unit.pos)
                || matches!(o, Order::RangedAttack { .. } | Order::UpgradeUnit { .. })
        })
    }

    /// A garrison holding its city under bombardment is the designed behavior (it absorbs
    /// attacks for the city). Flag it only when it had a better option this turn: a
    /// reachable ranged target, or a melee sortie it could reach with a favorable forecast.
    fn garrison_had_better_option(&self, unit: &Unit) -> bool {
        let stats = legatum_core::data::unit_stats_for(self.view.faction_of(unit.owner), unit.kind);
        if stats.ranged > 0 {
            return self.enemies().any(|e| {
                forecast_attack_mode(self.view, unit.id, e.pos, AttackMode::Ranged).is_some()
            });
        }
        let mut map = legatum_core::state::Map::new(
            self.view.width,
            self.view.height,
            legatum_core::Terrain::Grassland,
        );
        for (tile, seen) in map.tiles.iter_mut().zip(&self.view.tiles) {
            if let Some(t) = seen.terrain {
                tile.terrain = t;
            }
        }
        self.enemies().any(|e| {
            let Some(path) = legatum_core::path::find_path(&map, unit.pos, e.pos, |h| {
                self.view.tile(h)?.move_cost()
            }) else {
                return false;
            };
            // Every step but the attack itself must fit this turn, leaving a move to strike.
            let approach: i32 = path[..path.len().saturating_sub(1)]
                .iter()
                .filter_map(|h| self.view.tile(*h)?.move_cost())
                .sum();
            approach < stats.moves
                && forecast_attack_mode(self.view, unit.id, e.pos, AttackMode::Melee).is_some_and(
                    |f| {
                        f.win_chance >= 50
                            || (f.loss_chance == 0
                                && f.expected_damage_to_defender > f.expected_damage_to_attacker)
                    },
                )
        })
    }

    fn safe_step(&self, unit: &Unit) -> Option<Hex> {
        unit.pos.neighbors().into_iter().find(|h| {
            self.view
                .tile(*h)
                .is_some_and(|t| t.visible && t.terrain.is_some_and(|t| t.move_cost().is_some()))
                && self.enemies().all(|e| e.pos.distance(*h) > 1)
                && !self.view.units.iter().any(|u| {
                    u.pos == *h
                        && (hostile(self.before, unit.owner, u.owner)
                            || u.military == is_military(unit.kind))
                })
                && !self
                    .view
                    .cities
                    .iter()
                    .any(|c| c.pos == *h && hostile(self.before, unit.owner, c.owner))
        })
    }

    /// Attackers are taken from resolved events; context separately shows pre-order vision.
    fn attackers(
        &self,
        unit: Option<UnitId>,
        city: Option<CityId>,
        ranged_only: bool,
    ) -> Vec<Faction> {
        self.events
            .iter()
            .filter_map(|e| {
                let Event::Battle {
                    attacker_owner,
                    defender,
                    at,
                    ranged,
                    ..
                } = e
                else {
                    return None;
                };
                let matches_unit =
                    matches!(defender, Defender::Unit { unit: id, .. } if Some(*id) == unit);
                // Garrison absorbs fire intended for its city. Use the city's pre-turn location.
                let matches_city = city
                    .and_then(|id| self.before.cities.get(&id))
                    .is_some_and(|c| c.pos == *at);
                ((matches_unit || matches_city)
                    && (!ranged_only || *ranged)
                    && hostile(self.before, self.view.me, *attacker_owner))
                .then_some(self.before.player(*attacker_owner).faction)
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn context(&self, unit: Option<UnitId>, city: Option<CityId>) -> Context {
        let u = unit.and_then(|id| self.before.units.get(&id));
        let c = city.and_then(|id| self.before.cities.get(&id));
        let at = u.map(|u| u.pos).or_else(|| c.map(|c| c.pos));
        let mut nearby: Vec<_> = self
            .view
            .units
            .iter()
            .filter(|n| at.is_some_and(|at| at.distance(n.pos) <= 3))
            .collect();
        nearby.sort_by_key(|n| (at.map(|h| h.distance(n.pos)), n.id));
        let orders = if let Some(u) = u {
            let mut orders: Vec<_> = self
                .orders
                .iter()
                .filter(|o| order_unit(o) == Some(u.id))
                .map(|o| serde_json::to_string(o).expect("order serializes"))
                .collect();
            if orders.is_empty() {
                orders.push(format!("no new order; persistent destination {:?}", u.goto));
            }
            orders
        } else {
            self.orders
                .iter()
                .filter(|o| order_unit(o).is_none())
                .take(8)
                .map(|o| serde_json::to_string(o).expect("order serializes"))
                .collect()
        };
        let counterattacks = u.map_or_else(Vec::new, |u| self.enemies().filter_map(|e| {
            let mode = if legatum_core::data::unit_stats_for(self.view.faction_of(u.owner), u.kind).ranged > 0 { AttackMode::Ranged } else { AttackMode::Melee };
            // The forecast checks range and visibility, not route reachability for melee.
            if mode == AttackMode::Melee && u.pos.distance(e.pos) != 1 { return None; }
            forecast_attack_mode(self.view, u.id, e.pos, mode).map(|f| format!("{:?} vs u{} at {:?}: kill {}%, die {}% (forecast, not a chosen action)", mode, e.id.0, e.pos, f.win_chance, f.loss_chance))
        }).take(4).collect());
        Context {
            at,
            hp_before: u.map(|u| u.hp).or_else(|| c.map(|c| c.hp)),
            hp_after: match unit {
                Some(id) => Some(self.after.units.get(&id).map_or(0, |u| u.hp)),
                None => city.and_then(|id| self.after.cities.get(&id).map(|c| c.hp)),
            },
            orders,
            nearby_units_omitted: nearby.len().saturating_sub(8),
            nearby_visible_units: nearby
                .into_iter()
                .take(8)
                .map(|u| LocalUnit {
                    unit: u.id.0,
                    player: u.owner.0,
                    faction: self.view.faction_of(u.owner),
                    kind: u.kind,
                    at: u.pos,
                    hp: u.hp,
                    hostile: hostile(self.before, self.view.me, u.owner),
                })
                .collect(),
            nearby_own_cities: self
                .view
                .cities
                .iter()
                .filter(|c| c.owner == self.view.me && at.is_some_and(|h| h.distance(c.pos) <= 3))
                .take(4)
                .map(|c| LocalCity {
                    city: c.id.0,
                    at: c.pos,
                    hp: c.hp,
                    producing: format!("{:?}", c.detail.as_ref().and_then(|d| d.producing)),
                })
                .collect(),
            safe_step: u.and_then(|u| self.safe_step(u)),
            counterattacks,
        }
    }
}

impl Detector {
    pub fn new(game_id: &str, state: &GameState, config: Config) -> Self {
        let mut counts = BTreeMap::new();
        for p in state.players.iter().filter(|p| !p.is_human) {
            for behavior in Behavior::ALL {
                counts.entry((p.faction, behavior, None)).or_default();
            }
        }
        Self {
            report: Report {
                schema_version: 1,
                detector_version: report::VERSION,
                game_id: game_id.into(),
                engine_commit: legatum_wasm::history::ENGINE_COMMIT.into(),
                rules_fingerprint: legatum_wasm::history::RULES_FINGERPRINT.into(),
                complete_history: state.turn == 1,
                first_turn: state.turn,
                finished: false,
                has_winner: false,
                map_size: [state.setup.width, state.setup.height],
                config,
                seats: vec![],
                turns: vec![],
                incidents: vec![],
                rates: vec![],
            },
            counts,
            streaks: BTreeMap::new(),
            positions: BTreeMap::new(),
        }
    }

    fn streak(&mut self, f: &Frame<'_>, rule: Behavior, subject: u32, condition: bool) -> u32 {
        let key = (f.view.me, rule, subject);
        if !condition {
            self.streaks.remove(&key);
            return 0;
        }
        let turn = f.before.turn;
        let count = self
            .streaks
            .get(&key)
            .filter(|(last, _)| last.checked_add(1) == Some(turn))
            .map_or(1, |(_, n)| n + 1);
        self.streaks.insert(key, (turn, count));
        count
    }

    #[allow(clippy::too_many_arguments)]
    fn sample(
        &mut self,
        f: &Frame<'_>,
        behavior: Behavior,
        unit: Option<UnitId>,
        city: Option<CityId>,
        attackers: Vec<Faction>,
        consecutive: u32,
        flagged: bool,
        evidence: String,
    ) {
        let faction = f.before.player(f.view.me).faction;
        for attacker in std::iter::once(None).chain(attackers.iter().copied().map(Some)) {
            let c = self
                .counts
                .entry((faction, behavior, attacker))
                .or_default();
            c.opportunities += 1;
            c.incidents += u64::from(flagged);
        }
        if !flagged {
            return;
        }
        self.report.incidents.push(Incident {
            id: format!("I{:06}", self.report.incidents.len() + 1),
            turn: f.before.turn,
            player: f.view.me.0,
            faction,
            behavior,
            unit: unit.map(|u| u.0),
            city: city.map(|c| c.0),
            consecutive_turns: consecutive,
            evidence,
            attacker_factions: attackers,
            context: f.context(unit, city),
        });
    }

    pub fn observe(
        &mut self,
        before: &GameState,
        orders: &OrderMap,
        after: &GameState,
        events: &[Event],
    ) {
        let first = self.report.incidents.len();
        for p in before.players.iter().filter(|p| p.alive && !p.is_human) {
            let view = player_view(before, p.id);
            let orders = orders.get(&p.id).map_or(&[][..], Vec::as_slice);
            let invalid: BTreeSet<_> = legatum_core::validate_orders(before, p.id, orders)
                .iter()
                .map(|e| e.index)
                .collect();
            let f = Frame {
                before,
                after,
                view: &view,
                orders,
                valid: (0..orders.len()).filter(|i| !invalid.contains(i)).collect(),
                events,
            };
            self.units(&f);
            self.economy(&f);
        }
        self.report.turns.push(Turn {
            turn: before.turn,
            events: key_events(events),
            incidents: self.report.incidents[first..]
                .iter()
                .map(|i| i.id.clone())
                .collect(),
            players: after
                .players
                .iter()
                .filter(|p| !p.is_human)
                .map(|p| report::PlayerTurn {
                    player: p.id.0,
                    cities: after.cities_of(p.id).count(),
                    population: after.cities_of(p.id).map(|c| c.population).sum(),
                    techs: p.techs.len(),
                    vp: legatum_core::scoring::vp_total(after, p.id),
                    gold: p.gold,
                    alive: p.alive,
                    outcomes: player_outcomes(events, p.id),
                })
                .collect(),
        });
    }

    fn units(&mut self, f: &Frame<'_>) {
        for u in f.before.units.values().filter(|u| u.owner == f.view.me) {
            let garrison = is_military(u.kind)
                && f.before
                    .cities
                    .values()
                    .any(|c| c.owner == u.owner && c.pos == u.pos);
            // Holding is the garrison's job; only a missed better option is unanswered.
            let responded = f.responded(u) || (garrison && !f.garrison_had_better_option(u));
            let attackers = f.attackers(Some(u.id), None, false);
            if !attackers.is_empty() {
                let n = self.streak(f, Behavior::UnansweredAttack, u.id.0, !responded);
                self.sample(f, Behavior::UnansweredAttack, Some(u.id), None, attackers, n, n >= 2, format!("attacked {n} consecutive turns without moving, counterattacking or attempting a response; hp {} -> {}", u.hp, f.after.units.get(&u.id).map_or(0, |u| u.hp)));
            }
            if u.kind == UnitKind::Scout {
                let enemies: Vec<_> = f.enemies().filter(|e| e.pos.distance(u.pos) == 1).collect();
                if !enemies.is_empty()
                    && let Some(step) = f.safe_step(u)
                {
                    let factions = enemies
                        .iter()
                        .map(|e| f.view.faction_of(e.owner))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    self.sample(f, Behavior::ScoutNotFleeing, Some(u.id), None, factions, 1, !responded, format!("visible adjacent hostile military; known unoccupied step {:?} avoids adjacency; no escape attempted", step));
                }
            }
            let city = f
                .before
                .cities
                .values()
                .find(|c| c.owner == u.owner && c.pos == u.pos);
            if is_military(u.kind)
                && let Some(c) = city
            {
                let attackers = f.attackers(Some(u.id), Some(c.id), true);
                if !attackers.is_empty() {
                    let n = self.streak(f, Behavior::GarrisonUnderRangedFire, u.id.0, !responded);
                    self.sample(f, Behavior::GarrisonUnderRangedFire, Some(u.id), Some(c.id), attackers, n, n >= 2, format!("garrison/city tile bombarded for {n} consecutive turns while a favorable sortie or reachable ranged target was available; no sortie, ranged response, retreat or upgrade attempted"));
                }
            }
            if is_military(u.kind) && u.kind != UnitKind::Scout && u.hp >= 70 && city.is_none() {
                let threatened = f.before.cities.values().find(|c| {
                    c.owner == u.owner
                        && c.pos.distance(u.pos) <= 3
                        && f.enemies().any(|e| e.pos.distance(c.pos) <= 3)
                });
                if let Some(c) = threatened {
                    let n =
                        self.streak(f, Behavior::IdleMilitaryThreatenedCity, u.id.0, !responded);
                    self.sample(f, Behavior::IdleMilitaryThreatenedCity, Some(u.id), Some(c.id), vec![], n, n >= 2, format!("healthy military ({} HP) idle for {n} turns within 3 hexes of c{}; visible enemy military within 3 hexes of city", u.hp, c.id.0));
                }
            }
            let bounce = f
                .events
                .iter()
                .any(|e| matches!(e, Event::Bounce { units, .. } if units.contains(&u.id)));
            if bounce {
                let n = self.streak(f, Behavior::RepeatedBounce, u.id.0, true);
                self.sample(
                    f,
                    Behavior::RepeatedBounce,
                    Some(u.id),
                    None,
                    vec![],
                    n,
                    n >= 2,
                    format!("engine Bounce event in {n} consecutive turns"),
                );
            }
            if let Some(next) = f.after.units.get(&u.id).filter(|n| n.owner == u.owner) {
                let (last_turn, positions) = self
                    .positions
                    .entry((u.owner, u.id))
                    .or_insert_with(|| (f.before.turn.saturating_sub(1), VecDeque::from([u.pos])));
                if last_turn.checked_add(1) != Some(f.before.turn) {
                    *positions = VecDeque::from([u.pos]);
                }
                *last_turn = f.before.turn;
                positions.push_back(next.pos);
                while positions.len() > 4 {
                    positions.pop_front();
                }
                if positions.len() == 4 {
                    let oscillated = positions[0] == positions[2]
                        && positions[1] == positions[3]
                        && positions[0] != positions[1];
                    let path = format!(
                        "end-of-turn positions {:?} (A-B-A-B); retreats and front-line maneuvers need review",
                        positions
                    );
                    self.sample(
                        f,
                        Behavior::MovementOscillation,
                        Some(u.id),
                        None,
                        vec![],
                        3,
                        oscillated,
                        path,
                    );
                }
                if u.kind == UnitKind::Settler {
                    let n = self.streak(f, Behavior::SettlerWandering, u.id.0, true);
                    self.sample(f, Behavior::SettlerWandering, Some(u.id), None, vec![], n, n > self.report.config.settler_turns, format!("settler survives unfounded for {n} observed turns (threshold >{}); age before anchor unknown", self.report.config.settler_turns));
                }
            }
        }
    }

    fn economy(&mut self, f: &Frame<'_>) {
        let p = f.before.player(f.view.me);
        if f.view.economy.science_per_turn.total > 0 && !f.view.research.available.is_empty() {
            let idle = p.researching.is_none()
                && f.after.player(p.id).researching.is_none()
                && !f
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::TechResearched { player, .. } if *player == p.id));
            self.sample(f, Behavior::ResearchIdle, None, None, vec![], 1, idle, format!("research empty before and after turn despite {} science/turn and {} available technologies", f.view.economy.science_per_turn.total, f.view.research.available.len()));
        }
        let mut buy = None;
        let mut attacking_factions = BTreeSet::new();
        for city in f.view.cities.iter().filter(|c| c.owner == p.id) {
            let Some(detail) = &city.detail else {
                continue;
            };
            if detail.resistance == 0 && !detail.options.is_empty() {
                let idle = detail.producing.is_none() && f.after.cities.get(&city.id).is_some_and(|c| c.owner == p.id && c.producing.is_none())
                    && !f.events.iter().any(|e| matches!(e, Event::ProductionCompleted { city: id, .. } if *id == city.id));
                self.sample(f, Behavior::ProductionIdle, None, Some(city.id), vec![], 1, idle, "production empty before and after turn; city has legal build options and no resistance".into());
            }
            let attackers = f.attackers(None, Some(city.id), false);
            if attackers.is_empty() {
                continue;
            }
            attacking_factions.extend(attackers);
            for option in &detail.options {
                let defensive = matches!(option.item, ProductionItem::Unit { unit: k } if is_military(k) && k != UnitKind::Scout)
                    || matches!(
                        option.item,
                        ProductionItem::Building {
                            building: legatum_core::BuildingKind::Walls
                        }
                    );
                if defensive
                    && let Some(cost) = option.buy_cost
                    && cost <= p.gold
                    && cost > 0
                    && legatum_core::validate_orders(
                        f.before,
                        p.id,
                        &[Order::BuyItem {
                            city: city.id,
                            item: option.item,
                        }],
                    )
                    .is_empty()
                {
                    buy.get_or_insert((city.id, option.item, cost));
                }
            }
        }
        if let Some((city, item, cost)) = buy {
            let spent = f.events.iter().any(|e| matches!(e, Event::ItemBought { owner, .. } | Event::UnitUpgraded { owner, .. } if *owner == p.id));
            let n = self.streak(
                f,
                Behavior::GoldHoardedUnderAttack,
                0,
                !spent && p.gold >= 400,
            );
            self.sample(f, Behavior::GoldHoardedUnderAttack, None, Some(city), attacking_factions.into_iter().collect(), n, n >= 2, format!("city/garrison attacked {n} consecutive turns with no gold purchases/upgrades; treasury {}, legal defensive purchase {:?} costs {cost}", p.gold, item));
        }
    }

    pub fn finish(mut self, state: &GameState) -> Report {
        self.report.finished = state.finished;
        self.report.has_winner = state.winner.is_some();
        self.report.seats = state
            .players
            .iter()
            .map(|p| Seat {
                player: p.id.0,
                faction: p.faction,
                human: p.is_human,
                won: state.finished && state.winner == Some(p.team),
                difficulty: state.setup.players[p.id.0 as usize].difficulty,
            })
            .collect();
        self.report.rates = report::rates(self.counts);
        self.report
    }
}

/// These are outcomes to review, not automatic claims of good strategy. In particular,
/// a completion or a favorable battle can still have been a poor choice or a lucky roll.
fn player_outcomes(events: &[Event], player: PlayerId) -> Vec<String> {
    events.iter().filter_map(|event| match event {
        Event::CityFounded { city, owner, at, .. } if *owner == player =>
            Some(format!("founded city {} at {at:?}", city.0)),
        Event::CityCaptured { city, from, to, .. } if *from == player || *to == player =>
            Some(format!("city {} captured P{} -> P{}", city.0, from.0, to.0)),
        Event::ProductionCompleted { city, owner, item } if *owner == player =>
            Some(format!("city {} completed {item:?}", city.0)),
        Event::ItemBought { city, owner, item, gold } if *owner == player =>
            Some(format!("city {} bought {item:?} for {gold}", city.0)),
        Event::TechResearched { player: owner, tech } if *owner == player =>
            Some(format!("researched {tech:?}")),
        Event::FactionGoalCompleted { player: owner, vp, .. } if *owner == player =>
            Some(format!("completed faction goal for {vp} VP")),
        Event::UnitDestroyed { unit, owner, unit_kind, .. } if *owner == player =>
            Some(format!("lost {unit_kind:?} {}", unit.0)),
        Event::Battle { attacker, attacker_owner, defender, damage_to_attacker, damage_to_defender,
            attacker_hp_after, defender_hp_after, ranged, .. }
            if *attacker_owner == player || match defender {
                Defender::Unit { owner, .. } | Defender::City { owner, .. } => *owner == player,
            } => Some(format!("{} battle: P{} {attacker} vs {defender:?}; damage {damage_to_attacker}/{damage_to_defender}, remaining HP {attacker_hp_after}/{defender_hp_after}",
                if *ranged { "ranged" } else { "melee" }, attacker_owner.0)),
        _ => None,
    }).collect()
}

fn key_events(events: &[Event]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for e in events {
        let label = match e {
            Event::Battle { ranged: true, .. } => "ranged battles",
            Event::Battle { .. } => "melee battles",
            Event::UnitDestroyed { .. } => "units killed",
            Event::Bounce { .. } => "bounces",
            Event::CityFounded { .. } => "cities founded",
            Event::CityCaptured { .. } => "cities captured",
            Event::ProductionCompleted { .. } => "builds completed",
            Event::TechResearched { .. } => "techs completed",
            Event::OrderFailed { .. } => "failed orders",
            Event::VictoryHoldStarted { .. } => "VP holds started",
            Event::VictoryHoldBroken { .. } => "VP holds broken",
            Event::GameWon { .. } | Event::GameDrawn { .. } => "game finished",
            _ => continue,
        };
        *counts.entry(label).or_default() += 1;
    }
    if counts.is_empty() {
        "no key events".into()
    } else {
        counts
            .into_iter()
            .map(|(k, n)| format!("{n} {k}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
