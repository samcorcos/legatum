//! p881_trace <tape.json> <threat-player>
//! Diagnostic for the 8.8.1 Sparta playtest: replays the tape with its recorded orders and,
//! for every AI seat and turn, prints its threat model of <threat-player>, its active plan,
//! new threat/campaign/plan events, its production and diplomacy orders, and whether the
//! freshly planned strategic orders match the tape (fidelity). Then prints the turn's
//! battles, captures and diplomacy events. Set LEGATUM_PLAN_TRACE=1 for every candidate.
use legatum_core::{Event, Order, PlayerId, ProductionItem, player_view, resolve_turn};
use legatum_wasm::session::{BrainKind, make_brain};
use legatum_wasm::tape::Tape;

fn strategic(o: &Order) -> Option<String> {
    match o {
        Order::DeclareWar { .. }
        | Order::ProposeTreaty { .. }
        | Order::RespondToProposal { .. }
        | Order::SetProduction { .. }
        | Order::BuyItem { .. } => {
            let s = format!("{o:?}");
            // Acceptance explanations differ in wording only; compare the head.
            Some(s.split(", explanation").next().unwrap_or(&s).to_string())
        }
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let threat = PlayerId(a[2].parse()?);
    let mut state = tape.initial_state();
    println!("initial hash ok: {}", state.hash() == tape.initial_hash);
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| make_brain(BrainKind::Scripted, &state, p.id))
        .collect();
    let mut seen_threat = vec![0usize; state.players.len()];
    let mut seen_camp = vec![0usize; state.players.len()];
    let mut seen_plan = vec![0usize; state.players.len()];
    let mut mismatches = 0;
    for turn in &tape.turns {
        let t = turn.turn;
        for p in state.players.clone() {
            if p.is_human || !p.alive {
                continue;
            }
            let i = p.id.0 as usize;
            let v = player_view(&state, p.id);
            let orders = brains[i].orders(&v);
            let mine: Vec<String> = orders.iter().filter_map(strategic).collect();
            let taped: Vec<String> = turn
                .orders
                .get(&p.id)
                .map(|os| os.iter().filter_map(strategic).collect())
                .unwrap_or_default();
            let fidelity = if mine == taped {
                "match".to_string()
            } else {
                mismatches += 1;
                format!("MISMATCH tape={taped:?}")
            };
            let mem = brains[i].opponent_memory();
            let (threat_line, plan_line) = mem.as_ref().map_or(("-".into(), "-".into()), |m| {
                let top = m.assessments.threats.top().map(|x| (x.rival.0, x.score));
                let me = m
                    .assessments
                    .threats
                    .get(threat)
                    .map(|x| format!("{} [{}]", x.score, x.reasons().join("; ")));
                let plan = m.campaigns.plan.active.as_ref().map(|pl| {
                    format!(
                        "{:?} v{} since T{} terms {:?}",
                        pl.kind, pl.value, pl.start_turn, pl.terms
                    )
                });
                (
                    format!("top {top:?} | P{}: {}", threat.0, me.unwrap_or("-".into())),
                    plan.unwrap_or("-".into()),
                )
            });
            let own: Vec<_> = v.cities.iter().filter(|c| c.owner == v.me).collect();
            let walls = own.iter().filter(|c| c.walls).count();
            let mil: Vec<_> = v
                .units
                .iter()
                .filter(|u| u.owner == v.me && u.military)
                .collect();
            let seen: Vec<_> = v
                .units
                .iter()
                .filter(|u| u.owner == threat && u.military)
                .collect();
            let nearest = seen
                .iter()
                .flat_map(|u| own.iter().map(move |c| c.pos.distance(u.pos)))
                .min();
            let prod: Vec<String> = orders
                .iter()
                .filter_map(|o| match o {
                    Order::SetProduction { city, item } => Some(format!(
                        "c{}:{}",
                        city.0,
                        match item {
                            ProductionItem::Unit { unit } => format!("{unit:?}"),
                            ProductionItem::Building { building } => format!("{building:?}"),
                            other => format!("{other:?}"),
                        }
                    )),
                    Order::BuyItem { item, .. } => Some(format!("BUY {item:?}")),
                    _ => None,
                })
                .collect();
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
                    s[..s.len().min(400)].to_string()
                })
                .collect();
            let builds: Vec<String> = own
                .iter()
                .map(|c| {
                    format!(
                        "{}:{:?}",
                        c.name,
                        c.detail.as_ref().and_then(|d| d.producing)
                    )
                })
                .collect();
            println!(
                "T{t} P{i} {:?} gold {} | {threat_line}\n    plan: {plan_line}\n    own cities {} walls {} mil {} (str {}) | seen P{} mil {} (str {}) nearest {:?} | builds {:?} | prod {:?} | dip {:?} | {fidelity}",
                p.faction,
                v.economy.gold,
                own.len(),
                walls,
                mil.len(),
                mil.iter().map(|u| u.strength.max(u.ranged)).sum::<i32>(),
                threat.0,
                seen.len(),
                seen.iter().map(|u| u.strength.max(u.ranged)).sum::<i32>(),
                nearest,
                builds,
                prod,
                dip
            );
            let tl = brains[i].threat_log();
            for e in &tl[seen_threat[i]..] {
                println!("    THREAT-EVENT {e:?}");
            }
            seen_threat[i] = tl.len();
            let ce = brains[i].campaign_events();
            for e in &ce[seen_camp[i]..] {
                println!("    CAMPAIGN-EVENT {e:?}");
            }
            seen_camp[i] = ce.len();
            let pe = brains[i].plan_events();
            for e in &pe[seen_plan[i].min(pe.len())..] {
                println!("    PLAN-EVENT {e:?}");
            }
            seen_plan[i] = pe.len();
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        for e in &events {
            let keep = matches!(
                e,
                Event::Battle { .. }
                    | Event::UnitDestroyed { .. }
                    | Event::CityCaptured { .. }
                    | Event::PlayerEliminated { .. }
                    | Event::WarDeclared { .. }
                    | Event::PeaceEstablished { .. }
                    | Event::TreatyActivated { .. }
                    | Event::ProposalRejected { .. }
                    | Event::ProposalAccepted { .. }
                    | Event::SettlerCaptured { .. }
                    | Event::UnitDisbanded { .. }
                    | Event::SpartaConquest { .. }
                    | Event::SpartaKillHeal { .. }
                    | Event::ItemBought { .. }
                    | Event::ProductionCompleted { .. }
                    | Event::CompanyHired { .. }
            );
            if keep {
                let s = format!("{e:?}");
                println!("  EVENT T{t}: {}", &s[..s.len().min(300)]);
            }
        }
        if next.hash().to_string() != turn.hash {
            println!("  HASH MISMATCH after T{t}");
        }
        state = next;
    }
    println!("strategic-order mismatches: {mismatches}");
    Ok(())
}
