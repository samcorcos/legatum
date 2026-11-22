//! Replay the playtest-6 Mongols rush under current rules. The archived assault eliminated the
//! Byzantium and Athens; current rules still eliminate Byzantium but Athens survives. Provenance
//! are in fixtures/threat-t35.md.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::diplomacy::TreatyKind;
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use serde::Deserialize;
use std::collections::BTreeMap;

const HUMAN: PlayerId = PlayerId(0);
const HUMAN_TEAM: TeamId = TeamId(0);
const SPARTA: PlayerId = PlayerId(3);
const ATHENS: PlayerId = PlayerId(4);
const SWITZERLAND: PlayerId = PlayerId(5);
const INDIA: PlayerId = PlayerId(6);
const BYZANTIUM: PlayerId = PlayerId(7);

fn original() -> GameState {
    use std::io::Read;
    let mut json = String::new();
    flate2::read::GzDecoder::new(&include_bytes!("fixtures/threat-t35.json.gz")[..])
        .read_to_string(&mut json)
        .unwrap();
    let state: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(state.turn, 35);
    state
}

fn recorded() -> Vec<(u32, BTreeMap<PlayerId, Vec<Order>>)> {
    let turns: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/threat-t35-orders.json")).unwrap();
    turns
        .into_iter()
        .map(|t| {
            (
                t["turn"].as_u64().unwrap() as u32,
                serde_json::from_value(t["orders"].clone()).unwrap(),
            )
        })
        .collect()
}

#[derive(Deserialize)]
struct UnitAnchor {
    owner: u8,
    archived_id: u32,
    kind: UnitKind,
    pos: Hex,
}

/// The archive's numeric IDs are only keys into this recorded identity table. Bind each
/// survivor by owner, kind and position when it first appears in the current-rule replay;
/// thereafter follow the bound unit through movement, even if its position diverges.
struct ReplayUnits {
    anchors: BTreeMap<(PlayerId, UnitId), (UnitKind, Hex)>,
    bound: BTreeMap<(PlayerId, UnitId), UnitId>,
}

impl ReplayUnits {
    fn new() -> Self {
        let refs: Vec<UnitAnchor> =
            serde_json::from_str(include_str!("fixtures/threat-t35-unit-refs.json")).unwrap();
        let anchors = refs
            .into_iter()
            .map(|r| ((PlayerId(r.owner), UnitId(r.archived_id)), (r.kind, r.pos)))
            .collect();
        Self {
            anchors,
            bound: BTreeMap::new(),
        }
    }

    fn orders(
        &mut self,
        state: &GameState,
        recorded: BTreeMap<PlayerId, Vec<Order>>,
    ) -> BTreeMap<PlayerId, Vec<Order>> {
        recorded
            .into_iter()
            .map(|(owner, orders)| {
                let orders = orders
                    .into_iter()
                    .filter_map(|order| {
                        let mut value = serde_json::to_value(&order).unwrap();
                        let Some(old_id) = value.get("unit").and_then(serde_json::Value::as_u64)
                        else {
                            return Some(order);
                        };
                        let key = (owner, UnitId(old_id as u32));
                        let (kind, origin) = self
                            .anchors
                            .get(&key)
                            .expect("recorded unit has a stable identity");
                        let current = self
                            .bound
                            .get(&key)
                            .and_then(|id| state.units.get(id))
                            .filter(|u| u.owner == owner && u.kind == *kind)
                            .or_else(|| {
                                state.units.values().find(|u| {
                                    u.owner == owner
                                        && u.kind == *kind
                                        && u.pos == *origin
                                        && !self.bound.values().any(|id| *id == u.id)
                                })
                            })?;
                        self.bound.insert(key, current.id);
                        value["unit"] = current.id.0.into();
                        Some(serde_json::from_value(value).unwrap())
                    })
                    .collect();
                (owner, orders)
            })
            .collect()
    }
}

/// The state before `turn` resolves, from the recorded orders.
fn at(turn: u32) -> GameState {
    let mut state = original();
    let mut units = ReplayUnits::new();
    for (t, orders) in recorded().into_iter().filter(|(t, _)| *t < turn) {
        state = resolve_archived(&state, &units.orders(&state, orders));
        assert_eq!(state.turn, t + 1);
    }
    state
}

/// The archive is a rush played before the occupation lock (rules 31 made it the rule): replay it
/// as recorded, with every captured city released at once, or the locked garrisons stall the conquest.
fn resolve_archived(state: &GameState, orders: &BTreeMap<PlayerId, Vec<Order>>) -> GameState {
    let mut next = resolve_turn(state, orders).0;
    for city in next.cities.values_mut() {
        city.occupied_from = None;
    }
    next
}

fn hard(state: &GameState, p: PlayerId) -> ScriptedBrain {
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, p.0 as u64]).next_u64();
    ScriptedBrain::with_difficulty(state.player(p).faction, seed, p, AiDifficulty::Hard)
}

fn top_threat(brain: &ScriptedBrain) -> Option<PlayerId> {
    brain.opponents.threats.top().map(|t| t.rival)
}

/// The fixture is the user's state before turn 35. Since cities shield their garrisons
/// (`combat::city_tile_defender`), the same recorded orders now leave Athens alive.
#[test]
fn the_fixture_is_the_users_game_and_its_orders_still_resolve() {
    let mut state = original();
    let mut units = ReplayUnits::new();
    for (turn, orders) in recorded() {
        assert_eq!(turn, state.turn);
        state = resolve_archived(&state, &units.orders(&state, orders));
        assert_eq!(state.turn, turn + 1);
    }
    assert!(state.player(ATHENS).alive);
    assert!(!state.player(BYZANTIUM).alive);
    assert_eq!(
        state.cities.values().filter(|c| c.owner == HUMAN).count(),
        4
    );
}

/// Brains that watch from turn 35 name the human the top threat as the assault unfolds, and no
/// watcher ever names a different top threat.
#[test]
fn watching_ais_mark_the_conqueror_within_a_few_turns() {
    let mut state = original();
    let mut units = ReplayUnits::new();
    let ais: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| !p.is_human)
        .map(|p| p.id)
        .collect();
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> =
        ais.iter().map(|p| (*p, hard(&state, *p))).collect();
    let mut first: BTreeMap<PlayerId, u32> = BTreeMap::new();
    for (turn, orders) in recorded() {
        for (p, brain) in brains.iter_mut() {
            if !state.player(*p).alive {
                continue;
            }
            brain.orders(&player_view(&state, *p));
            if let Some(top) = top_threat(brain) {
                assert_eq!(top, HUMAN, "P{} named P{} at turn {turn}", p.0, top.0);
                first.entry(*p).or_insert(turn);
            }
        }
        state = resolve_archived(&state, &units.orders(&state, orders));
    }
    assert_eq!(
        first,
        BTreeMap::from([
            (PlayerId(2), 41),
            (SPARTA, 38),
            (ATHENS, 35),
            (SWITZERLAND, 36),
            (PlayerId(6), 38),
            (BYZANTIUM, 35)
        ])
    );
    assert!(state.player(ATHENS).alive);
    assert!(!state.player(BYZANTIUM).alive);
}

/// A save loaded at turn 45 has no AI memory, but one view still shows the conquest, so every
/// fresh brain names the human the top threat, refuses to sign with it, and the exposed ones turn
/// to ranged garrisons, Walls and Walls' tech.
#[test]
fn fresh_brains_on_the_turn_45_save_respond_immediately() {
    let state = at(45);
    let mut prepared = 0;
    for p in [SPARTA, ATHENS, SWITZERLAND, INDIA] {
        let mut brain = hard(&state, p);
        let view = player_view(&state, p);
        let orders = brain.orders(&view);
        assert!(validate_orders(&state, p, &orders).is_empty());
        assert_eq!(top_threat(&brain), Some(HUMAN), "P{}", p.0);
        assert!(!orders.iter().any(|o| matches!(
            o,
            Order::ProposeTreaty {
                to: HUMAN_TEAM,
                kind: TreatyKind::Alliance
            }
        )));
        // A garrisoned shooter first, then Walls (and Walls' tech).
        let defence = |item: &ProductionItem| {
            matches!(
                item,
                ProductionItem::Building {
                    building: BuildingKind::Walls
                } | ProductionItem::Unit {
                    unit: UnitKind::Archer | UnitKind::Crossbowman
                }
            )
        };
        prepared += usize::from(
            orders.iter().any(|o| match o {
                Order::SetResearch { tech } => *tech == TechId::Masonry,
                Order::SetResearchQueue { techs } => techs.first() == Some(&TechId::Masonry),
                Order::SetProduction { item, .. } | Order::BuyItem { item, .. } => defence(item),
                _ => false,
            }) || view.research.current == Some(TechId::Masonry),
        );
    }
    assert!(prepared >= 3, "{prepared}");
}

/// Resuming at turn 45 with the human holding still: nobody signs with the raider.
///
/// This archive predates Influence/Pact removal, so its old coalition-formation count no longer
/// describes this replay. The treaty boundary stays, and the neighbours name the conqueror.
#[test]
fn resumed_ais_do_not_sign_with_the_conqueror() {
    let mut state = at(45);
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| !p.is_human)
        .map(|p| (p.id, hard(&state, p.id)))
        .collect();
    for _ in 0..10 {
        let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for (p, brain) in brains.iter_mut() {
            if state.player(*p).alive {
                orders.insert(*p, brain.orders(&player_view(&state, *p)));
            }
        }
        let (next, events) = resolve_turn(&state, &orders);
        for e in &events {
            if let Event::TreatyActivated { relation, .. } = e {
                assert!(relation.a != HUMAN_TEAM && relation.b != HUMAN_TEAM);
            }
        }
        state = next;
    }
    let log = brains[&SPARTA].threat_log();
    assert!(log.iter().any(|e| matches!(
        e,
        legatum_ai::threat::ThreatEvent::TopThreat { rival, .. } if *rival == HUMAN
    )));
}

#[test]
fn hidden_human_units_cannot_change_threat_decisions() {
    let state = at(45);
    let view = player_view(&state, INDIA);
    let mut changed = state.clone();
    let destination = (0..state.map.len())
        .map(|i| state.map.hex_at(i))
        .find(|h| {
            view.tile(*h).is_some_and(|t| !t.visible)
                && state.map.move_cost(*h).is_some()
                && state.units.values().all(|u| u.pos != *h)
                && state.cities.values().all(|c| c.pos != *h)
        })
        .unwrap();
    let hidden = changed
        .units
        .values_mut()
        .find(|u| {
            u.owner == HUMAN
                && legatum_core::data::unit_stats(u.kind).military
                && view.tile(u.pos).is_some_and(|t| !t.visible)
        })
        .expect("a fogged human army unit");
    // Position is hidden; its kind or HP would change the public army standings.
    hidden.pos = destination;
    assert_eq!(
        serde_json::to_value(&view).unwrap(),
        serde_json::to_value(player_view(&changed, INDIA)).unwrap(),
        "hidden relocation must leave India's information unchanged"
    );
    let (mut a, mut b) = (hard(&state, INDIA), hard(&state, INDIA));
    assert_eq!(a.orders(&view), b.orders(&player_view(&changed, INDIA)));
    assert_eq!(a.opponents, b.opponents);
}
