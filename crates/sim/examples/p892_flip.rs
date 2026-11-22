//! p892_flip <tape.json> <player> <from-turn> <to-turn> [params.json]
//! Counterfactual for the 8.9.2 Mongol peace flip. Replays the tape, every seat on its recorded
//! orders, except <player> from <from-turn> on: its brain (built with the optional AiParams)
//! orders for itself, so a gene's effect on one empire shows turn by turn without the rest of
//! the game moving. The brain sees every turn from turn 1, so its memory is complete.
//! Prints the player's plan, plan events, diplomacy orders and the relation with each rival.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{Order, PlayerId, player_view, resolve_turn};
use legatum_wasm::session::brain_seed;
use legatum_wasm::tape::Tape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let me = PlayerId(a[2].parse()?);
    let from: u32 = a[3].parse()?;
    let to: u32 = a[4].parse()?;
    let params: Option<AiParams> = a
        .get(5)
        .map(|p| serde_json::from_str(&std::fs::read_to_string(p).expect("read params")))
        .transpose()?;
    let mut state = tape.initial_state();
    let p = state.player(me);
    let (faction, difficulty) = (p.faction, state.setup.players[me.0 as usize].difficulty);
    let seed = brain_seed(state.setup.seed, me);
    let mut brain: Box<dyn Brain> = Box::new(match &params {
        Some(params) => ScriptedBrain::with_params(faction, seed, me, difficulty, params),
        None => ScriptedBrain::with_difficulty(faction, seed, me, difficulty),
    });
    let mut seen_plan = 0usize;
    for turn in &tape.turns {
        let t = turn.turn;
        if t > to {
            break;
        }
        let view = player_view(&state, me);
        let orders = brain.orders(&view);
        let mut submitted = turn.orders.clone();
        if t >= from {
            submitted.insert(me, orders.clone());
            let dip: Vec<String> = orders
                .iter()
                .filter(|o| {
                    matches!(
                        o,
                        Order::DeclareWar { .. }
                            | Order::ProposeTreaty { .. }
                            | Order::RespondToProposal { .. }
                    )
                })
                .map(|o| {
                    let s = format!("{o:?}");
                    s[..s.len().min(160)].to_string()
                })
                .collect();
            let mem = brain.opponent_memory();
            let plan = mem
                .as_ref()
                .and_then(|m| {
                    m.campaigns
                        .plan
                        .active
                        .as_ref()
                        .map(|pl| format!("{:?} v{}", pl.kind, pl.value))
                })
                .unwrap_or("-".into());
            let rel: Vec<String> = state
                .diplomacy
                .iter()
                .flat_map(|d| d.relations.iter())
                .filter(|r| r.a.0 == me.0 || r.b.0 == me.0)
                .filter(|r| format!("{:?}", r.state) != "Peace" || r.effective_turn > 1)
                .map(|r| {
                    let other = if r.a.0 == me.0 { r.b.0 } else { r.a.0 };
                    format!("P{other}:{:?}@{}", r.state, r.effective_turn)
                })
                .collect();
            println!("T{t} P{} plan {plan} | rel {rel:?} | dip {dip:?}", me.0);
            if let Some(m) = mem {
                let pe = &m.campaigns.plan.events;
                for e in &pe[seen_plan.min(pe.len())..] {
                    println!("    PLAN-EVENT {e:?}");
                }
                seen_plan = pe.len();
            }
        }
        let (next, _events) = resolve_turn(&state, &submitted);
        state = next;
    }
    Ok(())
}
