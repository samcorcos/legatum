//! Reproducible read-only peace-duration audit for the Standard scripted simulator cohort.
//! `cargo run --release -p legatum-ai --example diplomacy_audit -- 700001 500 8 > audit.csv`
//! Setup shuffling and brain seeds match legatum-sim/session. Compare every final hash and
//! event count with the simulator JSON before using this audit as evidence. Only PlayerView
//! reaches a brain; the observer's state and history never participate in its decisions.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    Event, Faction, GameSetup, PlayerId, TeamId, new_game, player_view, resolve_turn,
};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Default)]
struct Peace {
    alive_turns: u32,
    peaceful_turns: u32,
    current: u32,
    longest: u32,
    first_war: Option<u32>,
    max_wars: usize,
}

fn audit(seed: u64) -> String {
    let mut setup = GameSetup::all_ai(seed);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        factions.swap(i, rng.range(0, i as i64) as usize);
    }
    for (p, faction) in setup.players.iter_mut().zip(factions) {
        p.faction = faction;
    }
    let mut state = new_game(&setup);
    let mut peace = vec![Peace::default(); setup.players.len()];
    let mut declarations = 0;
    let mut leader_declarations = 0;
    let mut agreements = 0;
    let mut captures = 0;
    let mut settlers = 0;
    let mut resolved = 0;
    while !state.finished && resolved < legatum_core::data::EXPECTED_GAME_TURNS * 4 {
        let mut orders = BTreeMap::new();
        let mut leaders = BTreeMap::<TeamId, Option<TeamId>>::new();
        for p in state.players.iter().filter(|p| p.alive) {
            let observation = &mut peace[p.id.0 as usize];
            observation.alive_turns += 1;
            let wars = state
                .teams()
                .into_iter()
                .filter(|&other| state.team_alive(other) && state.teams_at_war(p.team, other))
                .count();
            observation.max_wars = observation.max_wars.max(wars);
            if wars == 0 {
                observation.peaceful_turns += 1;
                observation.current += 1;
                observation.longest = observation.longest.max(observation.current);
            } else {
                observation.first_war.get_or_insert(state.turn);
                observation.current = 0;
            }
            let view = player_view(&state, p.id);
            let mut progress: Vec<_> = view
                .standings
                .iter()
                .filter(|s| s.target > 0 && state.team_alive(s.team))
                .map(|s| (100 * s.vp / s.target, s.team))
                .collect();
            progress.sort_by_key(|(vp, team)| (std::cmp::Reverse(*vp), *team));
            let leader = progress.first().and_then(|&(vp, team)| {
                let second = progress.get(1).map_or(0, |(vp, _)| *vp);
                (vp > second && (vp >= 80 || (vp >= 60 && vp - second >= 15))).then_some(team)
            });
            leaders.insert(p.team, leader);
            let brain_seed = Rng::derive(seed, &[purpose::AI, u64::from(p.id.0)]).next_u64();
            // Recreate each turn too: no observer memory can accidentally become brain memory.
            let mut brain = ScriptedBrain::with_difficulty(
                p.faction,
                brain_seed,
                p.id,
                setup.players[p.id.0 as usize].difficulty,
            );
            orders.insert(p.id, brain.orders(&view));
        }
        let (next, events) = resolve_turn(&state, &orders);
        for event in events {
            match event {
                Event::WarDeclared { relation } => {
                    declarations += relation.declarers.len();
                    for team in relation.declarers {
                        let against = if team == relation.a {
                            relation.b
                        } else {
                            relation.a
                        };
                        leader_declarations +=
                            usize::from(leaders.get(&team) == Some(&Some(against)));
                    }
                }
                Event::PeaceEstablished { .. } => agreements += 1,
                Event::CityCaptured { .. } => captures += 1,
                Event::SettlerCaptured { .. } => settlers += 1,
                _ => {}
            }
        }
        state = next;
        resolved += 1;
    }
    let mut rows = String::new();
    for (id, p) in peace.iter().enumerate() {
        let faction = state.player(PlayerId(id as u8)).faction;
        rows += &format!(
            "{seed},{id},{faction:?},{resolved},{},{},{},{},{},{},{},{declarations},{leader_declarations},{agreements},{captures},{settlers}\n",
            state.hash(),
            p.alive_turns,
            p.peaceful_turns,
            p.longest,
            p.first_war.map_or(String::new(), |t| t.to_string()),
            p.max_wars,
            u8::from(p.first_war.is_none())
        );
    }
    rows
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let seed: u64 = args.get(1).map_or(700001, |s| s.parse().expect("seed"));
    let games: usize = args.get(2).map_or(5, |s| s.parse().expect("games"));
    let threads: usize = args.get(3).map_or(1, |s| s.parse().expect("threads"));
    assert!(games > 0 && threads > 0);
    let next = AtomicUsize::new(0);
    let rows = Mutex::new(BTreeMap::new());
    std::thread::scope(|scope| {
        for _ in 0..threads.min(games) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= games {
                        break;
                    }
                    let row = audit(seed.wrapping_add(i as u64));
                    rows.lock().unwrap().insert(i, row);
                }
            });
        }
    });
    println!(
        "seed,player,faction,resolved_turns,final_hash,alive_turns,peaceful_turns,longest_peace,first_war,max_wars,never_at_war,declarations,leader_declarations,peace_agreements,city_captures,settler_captures"
    );
    for row in rows.into_inner().unwrap().into_values() {
        print!("{row}");
    }
}
