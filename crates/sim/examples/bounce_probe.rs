//! Movement-failure telemetry over recorded tapes (`legatum-sim --tape-out DIR`): how often
//! a unit's step failed because a friend held the tile, and how many attack orders reached a
//! battle. Replays with this build's rules, so pair it with tapes from the same build.
//! Usage: bounce_probe TAPE_DIR
use legatum_core::events::Attacker;
use legatum_core::rules::hostile;
use legatum_core::{Event, Order, resolve_turn};
use rayon::prelude::*;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Clone, Copy)]
struct Counts {
    games: u64,
    turns: u64,
    /// MoveBlocked: a friend held the tile and did not leave (or stayed to attack).
    friend_blocks: u64,
    /// Units in friendly Bounce events (two friends claimed one empty tile).
    friend_bounces: u64,
    /// Attack orders (a move whose destination held a hostile unit or city).
    attack_orders: u64,
    attacks_resolved: u64,
    attacks_failed: u64,
    /// Failed attack orders whose unit was stopped by a friend this turn.
    attacks_failed_by_friend: u64,
    /// Friend blocks on the unit's own destination tile (a destination choice, not routing).
    blocks_at_destination: u64,
    /// Friend blocks by a friend that stayed to attack.
    blocks_by_attacker: u64,
}

fn friend_block(reason: &str) -> bool {
    // Old wording: "blocked by a unit that did not move". New: "blocked by P0's Soldier,
    // which stayed to attack" / "..., which did not move".
    reason == "blocked by a unit that did not move"
        || (reason.starts_with("blocked by ") && reason.contains(", which "))
}

fn probe(path: &std::path::Path) -> Result<Counts, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tape = legatum_wasm::tape::Tape::from_json(&text)?;
    let mut state = tape.initial_state();
    let mut c = Counts {
        games: 1,
        ..Default::default()
    };
    for t in &tape.turns {
        let mut attacks: BTreeSet<legatum_core::UnitId> = BTreeSet::new();
        for (player, orders) in &t.orders {
            for o in orders {
                let (unit, to) = match o {
                    Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                        (*unit, *to)
                    }
                    _ => continue,
                };
                let Some(u) = state.units.get(&unit).filter(|u| u.owner == *player) else {
                    continue;
                };
                if !legatum_core::data::unit_stats(u.kind).military {
                    continue;
                }
                let target = state
                    .units
                    .values()
                    .any(|v| v.pos == to && hostile(&state, *player, v.owner))
                    || state
                        .cities
                        .values()
                        .any(|ct| ct.pos == to && hostile(&state, *player, ct.owner));
                if target {
                    attacks.insert(unit);
                }
            }
        }
        let mut dests: BTreeMap<legatum_core::UnitId, legatum_core::Hex> = state
            .units
            .values()
            .filter_map(|u| u.goto.map(|g| (u.id, g)))
            .collect();
        for orders in t.orders.values() {
            for o in orders {
                if let Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } = o {
                    dests.insert(*unit, *to);
                }
            }
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        let mut fought = BTreeSet::new();
        let mut friend_stopped = BTreeSet::new();
        for e in &events {
            match e {
                Event::Battle {
                    attacker: Attacker::Unit(a),
                    ..
                } => {
                    fought.insert(*a);
                }
                Event::MoveBlocked {
                    unit,
                    reason,
                    toward,
                    ..
                } if friend_block(reason) => {
                    c.friend_blocks += 1;
                    if dests.get(unit) == Some(toward) {
                        c.blocks_at_destination += 1;
                    }
                    if reason.ends_with("stayed to attack") {
                        c.blocks_by_attacker += 1;
                    }
                    friend_stopped.insert(*unit);
                }
                Event::Bounce { units, .. } => {
                    c.friend_bounces += units.len() as u64;
                    friend_stopped.extend(units.iter().copied());
                }
                _ => {}
            }
        }
        c.attack_orders += attacks.len() as u64;
        for a in &attacks {
            if fought.contains(a) {
                c.attacks_resolved += 1;
            } else {
                c.attacks_failed += 1;
                if friend_stopped.contains(a) {
                    c.attacks_failed_by_friend += 1;
                }
            }
        }
        c.turns += 1;
        state = next;
        if state.hash().to_string() != t.hash {
            return Err(format!(
                "{}: hash mismatch at turn {}",
                path.display(),
                t.turn
            ));
        }
    }
    Ok(c)
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: bounce_probe TAPE_DIR");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("tape dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let results: Vec<_> = paths.par_iter().map(|p| probe(p)).collect();
    let mut total = Counts::default();
    let mut errors = BTreeMap::new();
    for (p, r) in paths.iter().zip(results) {
        match r {
            Ok(c) => {
                total.games += c.games;
                total.turns += c.turns;
                total.friend_blocks += c.friend_blocks;
                total.friend_bounces += c.friend_bounces;
                total.attack_orders += c.attack_orders;
                total.attacks_resolved += c.attacks_resolved;
                total.attacks_failed += c.attacks_failed;
                total.attacks_failed_by_friend += c.attacks_failed_by_friend;
                total.blocks_at_destination += c.blocks_at_destination;
                total.blocks_by_attacker += c.blocks_by_attacker;
            }
            Err(e) => {
                errors.insert(p.display().to_string(), e);
            }
        }
    }
    let per_game = |v: u64| v as f64 / total.games.max(1) as f64;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "games": total.games,
            "turns": total.turns,
            "friend_blocks": total.friend_blocks,
            "friend_blocks_per_game": per_game(total.friend_blocks),
            "blocks_at_destination": total.blocks_at_destination,
            "blocks_by_attacker": total.blocks_by_attacker,
            "friend_bounces": total.friend_bounces,
            "friend_bounces_per_game": per_game(total.friend_bounces),
            "friend_failures_per_game": per_game(total.friend_blocks + total.friend_bounces),
            "attack_orders": total.attack_orders,
            "attacks_resolved": total.attacks_resolved,
            "attacks_failed": total.attacks_failed,
            "attacks_failed_by_friend": total.attacks_failed_by_friend,
            "attack_resolved_pct": 100.0 * total.attacks_resolved as f64 / total.attack_orders.max(1) as f64,
            "errors": errors,
        }))
        .unwrap()
    );
}
