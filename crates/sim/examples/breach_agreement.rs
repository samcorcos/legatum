//! How often plain Hard picks the Training search's tactical action (increment 3 of the AI
//! plan: distil the round-2 battle search into Hard's genes).
//!
//!   breach_agreement LOG.jsonl [--params FILE] [--max 100] [--out FILE]
//!
//! LOG is a round-2 training log (`combat_lab --training-compare OUT --training-round2` writes
//! `OUT/training-lab.jsonl`). A position is a decision whose full-game check played a searched
//! plan over Hard's (`confirm.reason == "clear"`, `played > 0`): the search's plan beat Hard's
//! with the interval clear of zero. Its lesson is the played candidate's orders (the searched
//! units' first-turn orders that differ from Hard's). Hard (the built-in genes, then the genes in
//! FILE) plans the logged arena again (`tactical_orders` on the battle's city, as the lab does),
//! and each lesson unit's order is compared by action: shoot or strike the same tile, attack the
//! same tile, move, or hold. A position agrees when every lesson unit's action matches.

use legatum_ai::training::{Arena, LogLine, TrainingRecord};
use legatum_ai::{AiParams, ScriptedBrain};
use legatum_core::{AiDifficulty, GameState, Hex, Order, UnitId, player_view};
use std::collections::BTreeMap;
use std::{env, fs};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Shoot(Hex),
    Attack(Hex),
    Move,
    Hold,
}

fn unit_of(o: &Order) -> Option<UnitId> {
    match o {
        Order::AirStrike { plane, .. } => Some(*plane),
        _ => legatum_core::rules::units::order_unit(o),
    }
}

fn action(s: &GameState, me: legatum_core::PlayerId, o: &Order) -> Action {
    let hostile = |h: Hex| {
        s.units.values().any(|u| u.pos == h && u.owner != me)
            || s.cities.values().any(|c| c.pos == h && c.owner != me)
    };
    match o {
        Order::RangedAttack { target, .. } | Order::AirStrike { target, .. } => {
            Action::Shoot(*target)
        }
        Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } if hostile(*to) => {
            Action::Attack(*to)
        }
        Order::MoveUnit { .. } | Order::MoveUnitPath { .. } => Action::Move,
        _ => Action::Hold,
    }
}

fn arena_order(arena: &Arena, o: &Order) -> Option<Order> {
    let m = |h: Hex| arena.to_arena(h);
    Some(match o {
        Order::RangedAttack { unit, target } => Order::RangedAttack {
            unit: *unit,
            target: m(*target)?,
        },
        Order::AirStrike { plane, target } => Order::AirStrike {
            plane: *plane,
            target: m(*target)?,
        },
        Order::MoveUnit { unit, to } => Order::MoveUnit {
            unit: *unit,
            to: m(*to)?,
        },
        Order::MoveUnitPath { unit, to, .. } => Order::MoveUnit {
            unit: *unit,
            to: m(*to)?,
        },
        other => other.clone(),
    })
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let log = args.get(1).expect("LOG.jsonl");
    let max: usize = arg(&args, "--max").map_or(100, |v| v.parse().expect("max"));
    let params: Option<AiParams> = arg(&args, "--params").map(|p| {
        let params: AiParams =
            serde_json::from_str(&fs::read_to_string(p).expect("params")).expect("params");
        params.validate().expect("params");
        params
    });
    let mut records: Vec<TrainingRecord> = fs::read_to_string(log)
        .expect("log")
        .lines()
        .filter_map(|l| match serde_json::from_str::<LogLine>(l) {
            Ok(LogLine::Decision(r)) => Some(*r),
            _ => None,
        })
        .filter(|r| {
            r.confirm
                .as_ref()
                .is_some_and(|c| c.reason == "clear" && c.played > 0)
        })
        .collect();
    // Deterministic pick: the largest full-game gains first.
    records.sort_by_key(|r| {
        let c = r.confirm.as_ref().expect("confirm");
        (
            std::cmp::Reverse(c.candidates[c.played].gain),
            r.engagement.clone(),
            r.turn,
        )
    });
    records.truncate(max);
    let mut rows = Vec::new();
    let (mut agree_base, mut agree_cand, mut units, mut unit_base, mut unit_cand) = (0, 0, 0, 0, 0);
    let mut kinds: BTreeMap<String, (u32, u32, u32)> = BTreeMap::new();
    for r in &records {
        let arena = Arena::restore(&r.arena).expect("arena");
        let s = &arena.state;
        let view = player_view(s, r.seat);
        let faction = s.player(r.seat).faction;
        let base = ScriptedBrain::with_difficulty(faction, 7, r.seat, AiDifficulty::Hard)
            .tactical_orders(&view, r.city);
        let cand = match &params {
            Some(p) => ScriptedBrain::with_params(faction, 7, r.seat, AiDifficulty::Hard, p)
                .tactical_orders(&view, r.city),
            None => base.clone(),
        };
        let c = r.confirm.as_ref().expect("confirm");
        let played = &c.candidates[c.played];
        let lesson: Vec<Order> = played
            .orders
            .iter()
            .filter_map(|o| arena_order(&arena, o))
            .collect();
        let pick = |orders: &[Order], id: UnitId| {
            orders
                .iter()
                .find(|o| unit_of(o) == Some(id))
                .map_or(Action::Hold, |o| action(s, r.seat, o))
        };
        let (mut b_all, mut c_all) = (true, true);
        for o in &lesson {
            let Some(id) = unit_of(o) else { continue };
            let want = action(s, r.seat, o);
            let b = pick(&base, id) == want;
            let k = pick(&cand, id) == want;
            units += 1;
            unit_base += u32::from(b);
            unit_cand += u32::from(k);
            b_all &= b;
            c_all &= k;
        }
        agree_base += u32::from(b_all);
        agree_cand += u32::from(c_all);
        let kind = format!(
            "{:?}{}",
            played.stance,
            if played.breach { "+breach" } else { "" }
        );
        let e = kinds.entry(kind).or_default();
        e.0 += 1;
        e.1 += u32::from(b_all);
        e.2 += u32::from(c_all);
        rows.push(serde_json::json!({
            "engagement": r.engagement, "turn": r.turn, "kind": format!("{:?}", r.kind),
            "stance": format!("{:?}", played.stance), "breach": played.breach,
            "gain": played.gain, "lesson_units": lesson.len(),
            "base_agrees": b_all, "candidate_agrees": c_all,
        }));
    }
    let n = records.len().max(1) as f64;
    let summary = serde_json::json!({
        "log": log, "params": arg(&args, "--params"), "positions": records.len(),
        "base_agree": agree_base, "candidate_agree": agree_cand,
        "base_agree_percent": f64::from(agree_base) * 100.0 / n,
        "candidate_agree_percent": f64::from(agree_cand) * 100.0 / n,
        "lesson_units": units, "base_unit_agree": unit_base, "candidate_unit_agree": unit_cand,
        "by_played": kinds.iter().map(|(k, (n, b, c))| serde_json::json!({
            "played": k, "positions": n, "base": b, "candidate": c})).collect::<Vec<_>>(),
        "rows": rows,
    });
    let text = serde_json::to_string_pretty(&summary).expect("json");
    if let Some(out) = arg(&args, "--out") {
        fs::write(out, &text).expect("write");
    }
    println!(
        "positions {} | whole-position agreement: Hard {} ({:.1}%), candidate {} ({:.1}%) | lesson units {}: Hard {}, candidate {}",
        records.len(),
        agree_base,
        f64::from(agree_base) * 100.0 / n,
        agree_cand,
        f64::from(agree_cand) * 100.0 / n,
        units,
        unit_base,
        unit_cand
    );
    for (k, (n, b, c)) in &kinds {
        println!("  {k}: {n} positions, Hard {b}, candidate {c}");
    }
}
