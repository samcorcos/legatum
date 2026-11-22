//! Replays one tape and prints how every empire's VP changed: each gain or loss of a VP source
//! (era objective, goal, Great Work), every VP hold started or broken, and a standing
//! line every N turns (VP by team, hold progress, cities, alive). The method of the title stall
//! diagnosis: who holds the target, and who breaks the hold, and how.
//! stall_trace <tape.json> [standing-every=50] [from-turn=0] [detail-turn detail-player]
//! The detail turn prints that player's cities: buildings, build queue, options, blockers.
use legatum_core::{Event, resolve_turn, scoring};
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let tape =
        legatum_wasm::tape::Tape::from_json(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let every: u32 = a.get(2).map_or(50, |x| x.parse().unwrap());
    let from: u32 = a.get(3).map_or(0, |x| x.parse().unwrap());
    let detail: Option<(u32, u8)> = a
        .get(4)
        .map(|t| (t.parse().unwrap(), a[5].parse().unwrap()));
    let mut state = tape.initial_state();
    let mut prev: BTreeMap<u8, BTreeSet<String>> = BTreeMap::new();
    let mut lost: BTreeMap<String, u32> = BTreeMap::new();
    let mut gained: BTreeMap<String, u32> = BTreeMap::new();
    for t in &tape.turns {
        let (next, events) = resolve_turn(&state, &t.orders);
        state = next;
        let shown = t.turn >= from;
        for p in &state.players {
            let now: BTreeSet<String> = scoring::vp(&state, p.id)
                .into_iter()
                .map(|(s, v)| format!("{s:?}={v}"))
                .collect();
            let old = prev.entry(p.id.0).or_default();
            for s in now.difference(old) {
                *gained
                    .entry(s.split_whitespace().next().unwrap().to_string())
                    .or_default() += 1;
                if shown {
                    println!("T{} p{} {:?} +{s}", t.turn, p.id.0, p.faction);
                }
            }
            for s in old.difference(&now) {
                *lost
                    .entry(s.split_whitespace().next().unwrap().to_string())
                    .or_default() += 1;
                if shown {
                    println!("T{} p{} {:?} -{s}", t.turn, p.id.0, p.faction);
                }
            }
            *old = now;
        }
        for e in &events {
            match e {
                Event::VictoryHoldStarted {
                    team, vp, target, ..
                } if shown => {
                    println!("T{} HOLD START team{} {vp}/{target}", t.turn, team.0)
                }
                Event::VictoryHoldBroken {
                    team,
                    turns_held,
                    vp,
                    target,
                } if shown => {
                    println!(
                        "T{} HOLD BROKEN team{} after {turns_held} at {vp}/{target}",
                        t.turn, team.0
                    )
                }
                Event::CityCaptured { .. } if shown => println!("T{} {e:?}", t.turn),
                _ => {}
            }
        }
        if let Some((dt, dp)) = detail
            && t.turn == dt
        {
            let v = legatum_core::player_view(&state, legatum_core::PlayerId(dp));
            println!("=== detail T{dt} p{dp} gold={}", v.economy.gold);
            for c in v.cities.iter().filter(|c| c.owner.0 == dp) {
                let Some(d) = &c.detail else { continue };
                println!(
                    "  city {:?} pop {} producing {:?} stored {} queue {:?} buildings {:?} buildable {:?} unavailable {:?}",
                    c.id,
                    c.population,
                    d.producing,
                    d.production_stored,
                    d.production_queue,
                    d.buildings,
                    d.options
                        .iter()
                        .filter(|o| matches!(o.item, legatum_core::ProductionItem::Building { .. }))
                        .map(|o| format!("{:?}", o.item))
                        .collect::<Vec<_>>(),
                    d.unavailable
                        .iter()
                        .map(|o| format!("{o:?}"))
                        .collect::<Vec<_>>()
                );
            }
        }
        if shown && every > 0 && t.turn % every == 0 {
            println!("--- standing T{}", t.turn);
            let (mut wars, mut allies) = (Vec::new(), Vec::new());
            for x in &state.players {
                for y in state.players.iter().filter(|y| y.id.0 > x.id.0) {
                    if state.at_war(x.id, y.id) {
                        wars.push(format!("{}-{}", x.id.0, y.id.0));
                    } else if state.allied(x.id, y.id) {
                        allies.push(format!("{}-{}", x.id.0, y.id.0));
                    }
                }
            }
            println!("  wars {wars:?} alliances {allies:?}");
            for p in &state.players {
                println!(
                    "  p{} {:?} alive={} vp={} (target {}) cities={} units={} {:?}",
                    p.id.0,
                    p.faction,
                    p.alive,
                    scoring::vp_total(&state, p.id),
                    scoring::team_target(&state, p.team),
                    state.cities_of(p.id).count(),
                    state.units.values().filter(|u| u.owner == p.id).count(),
                    scoring::vp(&state, p.id)
                        .iter()
                        .map(|(s, v)| format!("{s:?}={v}"))
                        .collect::<Vec<_>>()
                );
                if p.alive {
                    let v = legatum_core::player_view(&state, p.id);
                    for o in v
                        .objective_progress
                        .iter()
                        .filter(|o| o.entered && !o.era_scored)
                    {
                        let what = v
                            .eras
                            .iter()
                            .find(|e| e.era == o.era)
                            .and_then(|e| e.objectives.iter().find(|x| x.slot == o.slot))
                            .map(|x| format!("{:?}", x.objective))
                            .unwrap_or_default();
                        println!(
                            "      open era {} slot {}: {}/{} {what}",
                            o.era, o.slot, o.current, o.target
                        );
                    }
                }
            }
        }
    }
    println!("--- source changes over the game (gained / lost)");
    for (k, g) in &gained {
        println!("  {k}: +{g} / -{}", lost.get(k).copied().unwrap_or(0));
    }
    println!("final T{} finished={}", state.turn, state.finished);
}
