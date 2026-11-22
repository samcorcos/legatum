//! war_trace <scenario.json> <run> [seat-brain: hard|hard+search]
//! Plays one seeded run of a war scenario (the same seeding as `war_suite`) and prints, per
//! turn, the seat's plan changes, its units near the fighting, and every diplomatic event and
//! battle that involves the seat, so a success or failure can be read turn by turn.
//! Opponents that played Hard + Search in the user's game keep it, as in `war_suite`, and
//! `WAR_TRACE_PARAMS=<AiParams.json>` gives the seat candidate parameters.
mod war_script;

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::events::{Attacker, Defender};
use legatum_core::rng::Rng;
use legatum_core::{AiDifficulty, Event, PlayerId, rules::is_military};
use legatum_wasm::scenario::ScenarioFile;
use legatum_wasm::session::brain_seed;
use std::collections::BTreeMap;

fn main() -> Result<(), String> {
    let a: Vec<String> = std::env::args().collect();
    let file = ScenarioFile::parse(&std::fs::read_to_string(&a[1]).map_err(|e| e.to_string())?)?;
    let run: u32 = a[2].parse().map_err(|_| "run must be a number")?;
    let mut start = file.prepare(false)?;
    start.setup.seed = if run == 0 {
        start.setup.seed
    } else {
        Rng::derive(start.setup.seed, &[0x5CE4_A210, u64::from(run)]).next_u64()
    };
    let seat = file.seat;
    start.setup.players[seat.0 as usize].difficulty = AiDifficulty::Hard;
    let script = war_script::load(&file.id)?;
    let mut state = start.clone();
    let memories = file.ai_memories()?;
    let params: Option<AiParams> = match std::env::var("WAR_TRACE_PARAMS") {
        Ok(path) => Some(
            serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("params: {e}"))?,
        ),
        Err(_) => None,
    };
    let search_seat = a.get(3).is_some_and(|b| b == "hard+search");
    let mut brains: BTreeMap<PlayerId, Box<dyn Brain>> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            let setup = &start.setup.players[p.id.0 as usize];
            let seed = brain_seed(start.setup.seed, p.id);
            let mut brain = match params.as_ref().filter(|_| p.id == seat) {
                Some(params) => {
                    ScriptedBrain::with_params(p.faction, seed, p.id, setup.difficulty, params)
                }
                None => ScriptedBrain::with_difficulty(p.faction, seed, p.id, setup.difficulty),
            };
            if let Some(memory) = memories.get(&p.id) {
                brain.restore_opponent_memory(memory.clone());
            }
            // As in `war_suite`: opponents keep the brains they had in the user's game.
            let search = if p.id == seat {
                search_seat
            } else {
                setup.search && setup.difficulty == AiDifficulty::Hard
            };
            let brain: Box<dyn Brain> = if search {
                Box::new(legatum_ai::search::SearchBrain::new(
                    brain,
                    legatum_ai::search::SearchConfig::in_app(start.players.len()),
                ))
            } else {
                Box::new(brain)
            };
            (p.id, brain)
        })
        .collect();
    let mut seen_plan = 0usize;
    let mut seen_threat: BTreeMap<PlayerId, usize> = BTreeMap::new();
    // The seat's planning time per turn (its view plus its brain), native.
    let mut seat_ms: Vec<f64> = Vec::new();
    for _ in 0..=file.objective.turns {
        let turn = state.turn;
        // `WAR_TRACE_DIAG=1,4`: those seats' war gates before they plan (`Brain::war_diagnosis`).
        if let Ok(list) = std::env::var("WAR_TRACE_DIAG") {
            for id in list.split(',').filter_map(|s| s.parse::<u8>().ok()) {
                let id = PlayerId(id);
                if let Some(b) = brains.get(&id).filter(|_| state.player(id).alive) {
                    for line in b.war_diagnosis(&legatum_core::player_view(&state, id)) {
                        println!("   DIAG P{}: {line}", id.0);
                    }
                }
            }
        }
        let mut orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .filter_map(|p| {
                let started = std::time::Instant::now();
                let planned = brains
                    .get_mut(&p.id)
                    .map(|b| (p.id, b.orders(&legatum_core::player_view(&state, p.id))));
                if p.id == seat {
                    seat_ms.push(started.elapsed().as_secs_f64() * 1000.0);
                }
                planned
            })
            .collect();
        if let Some(script) = &script {
            script.apply(turn, &mut orders);
        }
        let mil = |who: PlayerId| {
            state
                .units
                .values()
                .filter(|u| u.owner == who && is_military(u.kind))
                .count()
        };
        let engineers = state
            .units
            .values()
            .filter(|u| u.owner == seat && u.siege)
            .count();
        let eng_at: Vec<String> = state
            .units
            .values()
            .filter(|u| u.owner == seat && u.siege)
            .map(|u| format!("({},{})", u.pos.q, u.pos.r))
            .collect();
        let mil_at: Vec<String> = state
            .units
            .values()
            .filter(|u| u.owner == seat && is_military(u.kind))
            .map(|u| format!("({},{})", u.pos.q, u.pos.r))
            .collect();
        println!("   engineers at {eng_at:?} military at {mil_at:?}");
        println!(
            "== T{turn} seat P{} military {} siege {engineers} gold {} ({:+}) | at war with {:?}",
            seat.0,
            mil(seat),
            state.player(seat).gold,
            legatum_core::player_view(&state, seat)
                .economy
                .gold_per_turn
                .total,
            state
                .players
                .iter()
                .filter(|p| p.alive && p.id != seat && state.at_war(seat, p.id))
                .map(|p| p.id.0)
                .collect::<Vec<_>>()
        );
        // The seat's public read of each rival: hollowness (army per city against the median)
        // and the hollow-leader bonus (gene `hollow_read`).
        {
            let v = legatum_core::player_view(&state, seat);
            let reads: Vec<String> = v
                .players
                .iter()
                .filter(|p| p.alive && p.team != v.team && p.cities > 0)
                .map(|p| {
                    format!(
                        "P{} hollow {} bonus {}",
                        p.id.0,
                        legatum_ai::threat::hollowness(&v, p.team),
                        legatum_ai::threat::hollow_bonus(&v, p.team)
                    )
                })
                .collect();
            println!("   READ {}", reads.join(" | "));
        }
        // Every AI's threat log: who it named its top threat, and when.
        for (id, b) in &brains {
            let log = b.threat_log();
            let seen = seen_threat.entry(*id).or_insert(0);
            for e in &log[(*seen).min(log.len())..] {
                println!("   THREAT P{}: {e:?}", id.0);
            }
            *seen = log.len();
        }
        if let Some(b) = brains.get(&seat) {
            let pe = b.plan_events();
            for e in &pe[seen_plan.min(pe.len())..] {
                println!("   PLAN {e:?}");
            }
            seen_plan = pe.len();
        }
        for (p, os) in &orders {
            for o in os {
                let s = format!("{o:?}");
                let diplomatic = s.starts_with("DeclareWar")
                    || s.starts_with("ProposeTreaty")
                    || s.starts_with("RespondToProposal");
                if diplomatic && (*p == seat || s.contains(&format!("TeamId({})", seat.0))) {
                    println!("   ORDER P{}: {}", p.0, &s[..s.len().min(160)]);
                }
                // The seat's aircraft: what it builds, buys, rebases and strikes with.
                let air = s.starts_with("AirStrike")
                    || s.starts_with("Rebase")
                    || ((s.starts_with("SetProduction") || s.starts_with("BuyItem"))
                        && (s.contains("Fighter") || s.contains("Bomber")));
                let refit = s.starts_with("UpgradeUnit") || s.starts_with("BuyItem");
                if (air || refit) && *p == seat {
                    println!("   ORDER P{}: {}", p.0, &s[..s.len().min(160)]);
                }
            }
        }
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        for e in &events {
            match e {
                Event::Battle {
                    attacker,
                    attacker_owner,
                    defender,
                    at,
                    damage_to_defender,
                    defender_hp_after,
                    ..
                } => {
                    let (dow, dk) = match defender {
                        Defender::Unit {
                            owner, unit_kind, ..
                        } => (*owner, format!("{unit_kind:?}")),
                        Defender::City { owner, city } => (*owner, format!("City{}", city.0)),
                    };
                    if *attacker_owner == seat || dow == seat {
                        let ak = match attacker {
                            Attacker::Unit(id) => state
                                .units
                                .get(id)
                                .map_or("?".into(), |u| format!("{:?}", u.kind)),
                            Attacker::City { city } => format!("City{}", city.0),
                        };
                        println!(
                            "   BATTLE P{}:{ak} -> P{}:{dk} at ({},{}) dmg {damage_to_defender} hp {defender_hp_after}",
                            attacker_owner.0, dow.0, at.q, at.r
                        );
                    }
                }
                // Orders the engine skipped (a replayed script that no longer fits shows up here).
                Event::OrderFailed { player, reason }
                    if std::env::var_os("WAR_TRACE_FAILED").is_some() =>
                {
                    println!(
                        "   FAILED P{}: {}",
                        player.0,
                        &reason[..reason.len().min(160)]
                    );
                }
                Event::CityCaptured { .. }
                | Event::WarDeclared { .. }
                | Event::PeaceEstablished { .. }
                | Event::ProposalAccepted { .. }
                | Event::ProposalRejected { .. }
                | Event::ItemBought { .. } => {
                    let s = format!("{e:?}");
                    println!("   EVENT {}", &s[..s.len().min(260)]);
                }
                _ => {}
            }
        }
        // The suite's illegal-position flags, per turn (stranded units are legal and left out).
        for problem in legatum_core::scenario::validate_state(&next)
            .into_iter()
            .filter(|p| !p.contains("territory without being at war"))
        {
            println!("   ILLEGAL after T{turn}: {problem}");
        }
        state = next;
    }
    if !seat_ms.is_empty() {
        let mut sorted = seat_ms.clone();
        sorted.sort_by(f64::total_cmp);
        println!(
            "seat planning ms over {} turns: mean {:.2}, median {:.2}, max {:.2}",
            sorted.len(),
            sorted.iter().sum::<f64>() / sorted.len() as f64,
            sorted[sorted.len() / 2],
            sorted[sorted.len() - 1]
        );
    }
    Ok(())
}
