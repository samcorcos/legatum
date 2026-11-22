//! What the AI trains (2026-10-31, the mounted fix: "i basically never buy things from that class
//! until there are tanks"). Eight-player Standard all-Hard games, solo teams, factions shuffled
//! per seed as in `supply_census`, played to the end (safety cap 4x the expected length).
//!
//! unit_census <games> [first_seed=9700001]
//!
//! Counts every land military unit completed (trained or bought: `Event::ProductionCompleted`)
//! by class, before the Modern era (where the triangle applies) and from it, as units and as
//! Production. It also counts how the units are used before the Modern era: attacks by class on
//! units and on cities (`Event::Battle`) and units lost (`Event::UnitDestroyed`). Prints one JSON
//! object.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::events::{Attacker, Defender};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, Faction, GameSetup, PlayerId, ProductionItem, TeamId, UnitClass, UnitKind,
    combat, data, player_view, resolve_turn,
};
use rayon::prelude::*;
use serde_json::json;
use std::collections::BTreeMap;

/// (class, pre-Modern) -> (units, Production).
type Tally = BTreeMap<(String, bool), (u64, u64)>;

/// (class, "attacks_unit" | "attacks_city" | "lost") -> count, pre-Modern units only.
type Uses = BTreeMap<(String, &'static str), u64>;

fn class_name(k: UnitKind) -> Option<&'static str> {
    match data::unit_class(k)? {
        UnitClass::Melee => Some("melee"),
        UnitClass::Ranged => Some("ranged"),
        UnitClass::Mounted => Some("mounted"),
        UnitClass::Air => None,
    }
}

fn play(seed: u64) -> (Tally, Uses, u32) {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(8);
    (setup.width, setup.height) = (data::MAP_WIDTH, data::MAP_HEIGHT);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (seat, (p, f)) in setup.players.iter_mut().zip(factions).enumerate() {
        p.faction = f;
        p.team = TeamId(seat as u8);
        p.difficulty = AiDifficulty::Hard;
    }
    setup.turn_limit = None;
    let cap = data::EXPECTED_GAME_TURNS * 4;
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<(PlayerId, ScriptedBrain)> = (0..8u8)
        .map(|i| {
            let id = PlayerId(i);
            let b = ScriptedBrain::with_difficulty(
                state.player(id).faction,
                legatum_wasm::session::brain_seed(setup.seed, id),
                id,
                AiDifficulty::Hard,
            );
            (id, b)
        })
        .collect();
    let mut tally = Tally::new();
    let mut uses = Uses::new();
    while !state.finished && state.turn < cap {
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                orders.insert(*id, b.orders(&player_view(&state, *id)));
            }
        }
        let (next, events) = resolve_turn(&state, &orders);
        let pre_class = |kind: UnitKind| {
            (data::unit_era_tier(kind) < combat::TRIANGLE_END_ERA_TIER)
                .then(|| class_name(kind))
                .flatten()
        };
        for e in &events {
            match e {
                Event::Battle {
                    attacker: Attacker::Unit(u),
                    attacker_owner,
                    defender,
                    ..
                } if !attacker_owner.is_barbarian() => {
                    if let Some(kind) = state.units.get(u).map(|x| x.kind)
                        && let Some(class) = pre_class(kind)
                    {
                        let what = match defender {
                            Defender::Unit { .. } => "attacks_unit",
                            Defender::City { .. } => "attacks_city",
                        };
                        *uses.entry((class.into(), what)).or_default() += 1;
                    }
                }
                Event::UnitDestroyed {
                    owner, unit_kind, ..
                } if !owner.is_barbarian() => {
                    if let Some(class) = pre_class(*unit_kind) {
                        *uses.entry((class.into(), "lost")).or_default() += 1;
                    }
                }
                _ => {}
            }
        }
        for e in &events {
            if let Event::ProductionCompleted {
                owner,
                item: ProductionItem::Unit { unit },
                ..
            } = e
                && let Some(class) = class_name(*unit)
                && !owner.is_barbarian()
            {
                let pre = data::unit_era_tier(*unit) < combat::TRIANGLE_END_ERA_TIER;
                let cost = data::unit_stats_for(next.player(*owner).faction, *unit).cost;
                let t = tally.entry((class.into(), pre)).or_default();
                t.0 += 1;
                t.1 += cost as u64;
            }
        }
        state = next;
    }
    (tally, uses, state.turn)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a[1].parse().unwrap();
    let first: u64 = a.get(2).map_or(9_700_001, |s| s.parse().unwrap());
    let outs: Vec<(Tally, Uses, u32)> = (0..games)
        .into_par_iter()
        .map(|i| play(first + i))
        .collect();
    let mut total = Tally::new();
    let mut uses_total = Uses::new();
    for (_, u, _) in &outs {
        for (k, v) in u {
            *uses_total.entry(k.clone()).or_default() += v;
        }
    }
    for (t, _, _) in &outs {
        for (k, v) in t {
            let e = total.entry(k.clone()).or_default();
            e.0 += v.0;
            e.1 += v.1;
        }
    }
    let mut eras = serde_json::Map::new();
    for (pre, name) in [(true, "pre_modern"), (false, "modern_on")] {
        let rows: Vec<_> = ["melee", "ranged", "mounted"]
            .iter()
            .map(|c| {
                (
                    *c,
                    total
                        .get(&(c.to_string(), pre))
                        .copied()
                        .unwrap_or_default(),
                )
            })
            .collect();
        let (units, prod): (u64, u64) =
            rows.iter().fold((0, 0), |s, (_, v)| (s.0 + v.0, s.1 + v.1));
        let mut per = serde_json::Map::new();
        for (c, v) in rows {
            per.insert(
                c.into(),
                json!({
                    "units": v.0,
                    "unit_share_percent": (v.0 as f64 * 1000.0 / units.max(1) as f64).round() / 10.0,
                    "production_share_percent": (v.1 as f64 * 1000.0 / prod.max(1) as f64).round() / 10.0,
                }),
            );
        }
        per.insert("units".into(), json!(units));
        eras.insert(name.into(), serde_json::Value::Object(per));
    }
    let mut use_rows = serde_json::Map::new();
    for c in ["melee", "ranged", "mounted"] {
        let n = |w: &'static str| uses_total.get(&(c.to_string(), w)).copied().unwrap_or(0);
        use_rows.insert(
            c.into(),
            json!({ "attacks_on_units": n("attacks_unit"), "attacks_on_cities": n("attacks_city"), "lost": n("lost") }),
        );
    }
    let turns: Vec<u32> = outs.iter().map(|o| o.2).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "games": games,
            "first_seed": first,
            "mean_turns": turns.iter().map(|&t| t as f64).sum::<f64>() / turns.len().max(1) as f64,
            "land_units_completed": eras,
            "pre_modern_use": use_rows,
        }))
        .unwrap()
    );
}
