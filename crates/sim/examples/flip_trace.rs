//! flip_trace <tape.json> [max-age=10] [params.json]
//! Replays an all-AI tape with fresh brains and prints, for every war declared within `max-age`
//! turns of a Peace with the same target: who proposed the Peace, and each side's plan and
//! held-capture state when the Peace took effect and when the war was declared, with the
//! declarer's plan switches in between. Diagnostic only; reads the tape, changes nothing.
use legatum_core::{Event, PlayerId, TeamId, player_view, resolve_turn};
use legatum_wasm::session::{BrainKind, make_brain};
use legatum_wasm::tape::Tape;
use serde_json::Value;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let max_age: u32 = a.get(2).map_or(10, |s| s.parse().unwrap());
    let params: Option<legatum_ai::AiParams> = a
        .get(3)
        .map(|p| serde_json::from_str(&std::fs::read_to_string(p).expect("read params")))
        .transpose()?;
    let mut state = tape.initial_state();
    let mut brains: Vec<Box<dyn legatum_ai::Brain>> = state
        .players
        .iter()
        .map(|p| match &params {
            Some(params) => Box::new(legatum_ai::ScriptedBrain::with_params(
                p.faction,
                legatum_wasm::session::brain_seed(state.setup.seed, p.id),
                p.id,
                state.setup.players[p.id.0 as usize].difficulty,
                params,
            )) as Box<dyn legatum_ai::Brain>,
            None => make_brain(BrainKind::Scripted, &state, p.id),
        })
        .collect();
    let factions: Vec<String> = state
        .players
        .iter()
        .map(|p| format!("{:?}", p.faction))
        .collect();
    let teams: Vec<TeamId> = state.players.iter().map(|p| p.team).collect();
    let seat_of = |t: TeamId| teams.iter().position(|x| *x == t).unwrap_or(0);
    // plan history per seat: (turn, kind string, reason, held_capture)
    let mut held: BTreeMap<(u32, usize), bool> = BTreeMap::new();
    let mut proposals: Vec<(u32, TeamId, TeamId)> = Vec::new();
    let mut peaces: Vec<(u32, TeamId, TeamId)> = Vec::new();
    let mut wars: Vec<(u32, TeamId, TeamId)> = Vec::new();
    let mut plans: Vec<Vec<Value>> = vec![Vec::new(); state.players.len()];
    for turn in &tape.turns {
        let t = turn.turn;
        for p in state.players.clone() {
            if p.is_human || !p.alive {
                continue;
            }
            let v = player_view(&state, p.id);
            let _ = brains[p.id.0 as usize].orders(&v);
            if let Some(m) = brains[p.id.0 as usize].opponent_memory() {
                let j = serde_json::to_value(&m)?;
                held.insert(
                    (t, p.id.0 as usize),
                    !j["campaigns"]["held_capture"].is_null(),
                );
                if let Some(ev) = j["campaigns"]["plan"]["events"].as_array() {
                    plans[p.id.0 as usize] = ev.clone();
                }
            }
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        for e in &events {
            match e {
                Event::ProposalCreated { proposal }
                    if format!("{:?}", proposal.kind) == "Peace" =>
                {
                    proposals.push((t, proposal.from, proposal.to));
                }
                Event::PeaceEstablished { relation } => {
                    peaces.push((relation.effective_turn, relation.a, relation.b));
                }
                Event::WarDeclared { relation } => {
                    for d in &relation.declarers {
                        let tgt = if *d == relation.a {
                            relation.b
                        } else {
                            relation.a
                        };
                        wars.push((relation.effective_turn, *d, tgt));
                    }
                }
                _ => {}
            }
        }
        state = next;
    }
    let kind = |ev: &Value| -> String {
        let to = &ev["to"];
        to.to_string()
    };
    let plan_at = |seat: usize, turn: u32| -> String {
        plans[seat]
            .iter()
            .rfind(|e| e["turn"].as_u64().unwrap_or(0) as u32 <= turn)
            .map_or("-".into(), |e| {
                format!("{} ({})", kind(e), e["reason"].as_str().unwrap_or("?"))
            })
    };
    for &(wt, d, tgt) in &wars {
        let last = peaces
            .iter()
            .filter(|(pt, x, y)| *pt <= wt && ((*x == d && *y == tgt) || (*x == tgt && *y == d)))
            .map(|p| p.0)
            .max();
        let Some(pt) = last else { continue };
        if wt - pt > max_age {
            continue;
        }
        let initiator = proposals
            .iter()
            .rfind(|(t0, f, to)| *t0 <= pt && ((*f == d && *to == tgt) || (*f == tgt && *to == d)))
            .map(|p| p.1);
        let (ds, ts) = (seat_of(d), seat_of(tgt));
        let who = match initiator {
            Some(i) if i == d => "declarer proposed",
            Some(_) => "target proposed",
            None => "?",
        };
        println!(
            "T{wt} {} declares on {} after Peace T{pt} ({who}); declarer plan at peace: {} held={}; target plan at peace: {}; declarer plan now: {} held={}",
            factions[ds],
            factions[ts],
            plan_at(ds, pt.saturating_sub(1)),
            held.get(&(pt.saturating_sub(1), ds))
                .copied()
                .unwrap_or(false),
            plan_at(ts, pt.saturating_sub(1)),
            plan_at(ds, wt),
            held.get(&(wt, ds)).copied().unwrap_or(false),
        );
        let _ = PlayerId(0);
    }
    Ok(())
}
