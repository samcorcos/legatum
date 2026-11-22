//! peace_trace <tape.json> [memory-turn memory.json]
//!
//! Why AI wars do not end. Replays a playtest tape with every seat's Hard brain shadowing it
//! (seeds as the app derives them; optionally restoring the save's `ai_opponent_memory` at the
//! turn an archive restarts) and, for every AI seat at war with another empire, prints each turn
//! how it would answer a Peace (`ScriptedBrain::peace_diagnosis`): the `wants_peace` gate that
//! decided, the Peace score and its terms, turns since its units last attacked that enemy, and
//! its plan. Then the seat's Peace offers and answers from the tape, and the turn's Peace events.
//! Set PEACE_TRACE_PAIRS=2-3,3-2 to limit the output to (seat-enemy team) pairs.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::diplomacy::{RelationState, TreatyKind};
use legatum_core::{AiDifficulty, Event, Order, PlayerId, player_view, resolve_turn};
use legatum_wasm::tape::Tape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let memory: Option<(u32, String)> = a
        .get(2)
        .zip(a.get(3))
        .map(|(t, f)| (t.parse().expect("memory turn"), f.clone()));
    let pairs: Option<Vec<(u8, u8)>> = std::env::var("PEACE_TRACE_PAIRS").ok().map(|s| {
        s.split(',')
            .map(|p| {
                let (x, y) = p.split_once('-').expect("pair a-b");
                (x.parse().unwrap(), y.parse().unwrap())
            })
            .collect()
    });
    let mut state = tape.initial_state();
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_difficulty(
                p.faction,
                legatum_wasm::session::brain_seed(state.setup.seed, p.id),
                p.id,
                AiDifficulty::Hard,
            )
        })
        .collect();
    let (mut mismatches, mut hash_bad) = (0, 0);
    for t in &tape.turns {
        if let Some((turn, file)) = &memory
            && *turn == t.turn
        {
            let mems: std::collections::BTreeMap<PlayerId, legatum_ai::OpponentMemory> =
                serde_json::from_str(&std::fs::read_to_string(file)?)?;
            for (id, m) in mems {
                brains[id.0 as usize].restore_opponent_memory(m);
            }
            println!("restored AI memory at T{}", t.turn);
        }
        let ids: Vec<PlayerId> = state
            .players
            .iter()
            .filter(|p| p.alive && !p.is_human)
            .map(|p| p.id)
            .collect();
        for id in state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.id)
            .collect::<Vec<_>>()
        {
            let view = player_view(&state, id);
            if ids.contains(&id) {
                let me = view.team;
                for r in &view.diplomacy.relations {
                    if r.state != RelationState::War || !(r.a == me || r.b == me) {
                        continue;
                    }
                    let other = if r.a == me { r.b } else { r.a };
                    if pairs
                        .as_ref()
                        .is_some_and(|ps| !ps.contains(&(me.0, other.0)))
                    {
                        continue;
                    }
                    let d = brains[id.0 as usize].peace_diagnosis(&view, other);
                    let declared = if r.declarers.contains(&me) {
                        "we declared"
                    } else if r.declarers.contains(&other) {
                        "they declared"
                    } else {
                        "?"
                    };
                    println!(
                        "T{} P{} {:?} vs T{} war since T{} ({declared}) idle {:?} | wants {} [{}] score {:?} vp_cost {} endangered {} | terms {:?} | plan {}",
                        t.turn,
                        id.0,
                        state.player(id).faction,
                        other.0,
                        r.effective_turn,
                        d.idle_turns,
                        d.wants,
                        d.gate,
                        d.score,
                        d.vp_cost,
                        d.endangered,
                        d.terms,
                        d.plan
                    );
                }
            }
            let planned = brains[id.0 as usize].orders(&view);
            if ids.contains(&id) {
                let strategic = |o: &Order| match o {
                    Order::DeclareWar { .. }
                    | Order::ProposeTreaty { .. }
                    | Order::RespondToProposal { .. } => {
                        let s = format!("{o:?}");
                        Some(s.split(", explanation").next().unwrap_or(&s).to_string())
                    }
                    _ => None,
                };
                let mine: Vec<_> = planned.iter().filter_map(strategic).collect();
                let taped: Vec<_> = t
                    .orders
                    .get(&id)
                    .map(|os| os.iter().filter_map(strategic).collect())
                    .unwrap_or_default();
                if mine != taped {
                    mismatches += 1;
                    println!(
                        "  MISMATCH T{} P{}: planned {mine:?} taped {taped:?}",
                        t.turn, id.0
                    );
                }
                for o in t.orders.get(&id).into_iter().flatten() {
                    match o {
                        Order::ProposeTreaty {
                            to,
                            kind: TreatyKind::Peace,
                        } => println!("  T{} P{} proposes Peace to T{}", t.turn, id.0, to.0),
                        Order::RespondToProposal {
                            proposal, accept, ..
                        } => {
                            if let Some(p) = view
                                .diplomacy
                                .proposals
                                .iter()
                                .find(|p| p.id == *proposal && p.kind == TreatyKind::Peace)
                            {
                                println!(
                                    "  T{} P{} answers Peace from T{}: {}",
                                    t.turn,
                                    id.0,
                                    p.from.0,
                                    if *accept { "accept" } else { "refuse" }
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            if let Event::PeaceEstablished { relation } = e {
                println!(
                    "  EVENT T{} Peace T{}-T{}",
                    t.turn, relation.a.0, relation.b.0
                );
            }
            if let Event::Battle {
                attacker_owner,
                defender,
                ..
            } = e
            {
                let (d, city) = match defender {
                    legatum_core::events::Defender::Unit { owner, .. } => (*owner, false),
                    legatum_core::events::Defender::City { owner, .. } => (*owner, true),
                };
                if !attacker_owner.is_barbarian() && !d.is_barbarian() {
                    println!(
                        "  BATTLE T{} P{} -> P{}{}",
                        t.turn,
                        attacker_owner.0,
                        d.0,
                        if city { " (city)" } else { "" }
                    );
                }
            }
            if let Event::CityCaptured { from, to, .. } = e {
                println!("  CAPTURE T{} P{} -> P{}", t.turn, from.0, to.0);
            }
            if let Event::WarDeclared { .. } = e {
                println!("  EVENT T{} {e:?}", t.turn);
            }
        }
        state = next;
        if state.hash().to_string() != t.hash {
            hash_bad += 1;
        }
    }
    println!("strategic order mismatches {mismatches}, hash mismatches {hash_bad}");
    Ok(())
}
