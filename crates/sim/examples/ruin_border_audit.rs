//! Count ruins that remain inside a living AI's borders for at least 20 turns.
//! The batch launcher appends its own --out/--status-file flags; this study ignores them.
use legatum_core::{AiDifficulty, GameSetup, PlayerId};
use legatum_wasm::session::{BrainKind, Session};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |key: &str, default: u64| {
        args.windows(2)
            .find(|w| w[0] == key)
            .and_then(|w| w[1].parse::<u64>().ok())
            .unwrap_or(default)
    };
    let games = arg("--games", 200);
    let seed = arg("--seed", 9_810_000);
    let mut long_lived = 0u64;
    let mut ever_owned = 0u64;
    let mut game_hits = 0u64;
    for game in 0..games {
        let mut setup = GameSetup::all_ai(seed + game);
        for p in &mut setup.players {
            p.difficulty = AiDifficulty::Hard;
        }
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        let mut ages: BTreeMap<usize, (PlayerId, u32)> = BTreeMap::new();
        let mut hit = false;
        while !session.state.finished && session.state.turn <= 120 {
            let mut next = BTreeMap::new();
            for (i, tile) in session.state.map.tiles.iter().enumerate() {
                let Some(owner) = tile.owner.filter(|_| tile.ruin) else {
                    continue;
                };
                if !session.state.player(owner).alive {
                    continue;
                }
                let age = match ages.get(&i) {
                    Some((previous, age)) if *previous == owner => age + 1,
                    _ => {
                        ever_owned += 1;
                        1
                    }
                };
                if age == 20 {
                    long_lived += 1;
                    hit = true;
                }
                next.insert(i, (owner, age));
            }
            ages = next;
            let orders = session.ai_orders();
            session.step(&orders);
        }
        game_hits += u64::from(hit);
    }
    let report = format!(
        "games={games} seed={seed} through_turn=120 owned_ruin_episodes={ever_owned} unclaimed_20_turns={long_lived} games_with_one={game_hits}"
    );
    println!("{report}");
    if let Some(dir) = args.windows(2).find(|w| w[0] == "--out-dir").map(|w| &w[1]) {
        let path = std::path::Path::new(dir).join("ruin-border-audit.txt");
        std::fs::create_dir_all(dir).expect("create report directory");
        std::fs::write(path, format!("{report}\n")).expect("write report");
    }
}
