//! Fixed-input Large/Standard benchmark, using the same Session and JSON API as WASM.
//! Run `cargo run --release -p legatum-wasm --example perf_full_tree -- record DIR`,
//! then `... -- measure DIR [samples=31] [warmup=5] > native.csv`.
//! Record before production edits: measurement checks AI orders, full saves, reports and
//! all eight serialized views byte for byte before timing individual (unbatched) calls.

use legatum_core::{
    AiDifficulty, GameSetup, GameState, PlayerId, TechId, player_view, resolve_turn, visibility,
};
use legatum_wasm::{
    Game, json,
    session::{BrainKind, OrderMap, Session, make_brain},
    tape::{Tape, TapeTurn},
};
use serde::{Deserialize, Serialize};
use std::{fs, hint::black_box, io::Write, path::Path, time::Instant};

#[derive(Serialize, Deserialize)]
struct Case {
    name: String,
    seed: String,
    difficulty: AiDifficulty,
    turn: u32,
    units: usize,
    cities: usize,
    techs: Vec<usize>,
    synthetic: bool,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    technologies: usize,
    cases: Vec<Case>,
}

fn write(path: &Path, bytes: impl AsRef<[u8]>) {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap()
        .write_all(bytes.as_ref())
        .unwrap();
}

fn read(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join(name)).unwrap()
}

fn human_orders(state: &GameState) -> OrderMap {
    // Seat zero develops normally, but its orders are supplied as a human's would be.
    // This planning is outside every measured interval; seven other brains are timed.
    let mut brain = make_brain(BrainKind::Scripted, state, PlayerId(0));
    OrderMap::from([(PlayerId(0), brain.orders(&player_view(state, PlayerId(0))))])
}

fn record_case(root: &Path, state: &GameState, synthetic: bool) -> Case {
    assert!(!state.finished);
    assert!(state.players.iter().all(|p| p.alive));
    let difficulty = state.setup.players[1].difficulty;
    let name = format!(
        "{:?}-seed{}-turn{}{}",
        difficulty,
        state.setup.seed,
        state.turn,
        if synthetic { "-all-techs" } else { "" }
    );
    let dir = root.join(&name);
    fs::create_dir(&dir).unwrap();
    let save = json::state_to_json(state);
    let human = human_orders(state);
    let human_json = serde_json::to_string(&human).unwrap();
    let mut session = Session::from_state(state.clone(), |_| BrainKind::Scripted);
    let mut orders = session.ai_orders();
    assert_eq!(orders.len(), 7);
    orders.extend(human);
    let next = resolve_turn(state, &orders).0;
    let mut game = Game::from_save_json(&save).unwrap();
    let report = game.end_turn_json(&human_json).unwrap();
    assert_eq!(game.save_json(), json::state_to_json(&next));
    write(&dir.join("state.json"), save);
    write(&dir.join("human.json"), human_json);
    write(
        &dir.join("orders.json"),
        serde_json::to_string(&orders).unwrap(),
    );
    write(&dir.join("next.json"), game.save_json());
    write(&dir.join("report.json"), report);
    for id in 0..8 {
        write(
            &dir.join(format!("view-{id}.json")),
            serde_json::to_string(&player_view(state, PlayerId(id))).unwrap(),
        );
    }
    eprintln!(
        "recorded {name}: {} units, {} cities, techs {:?}",
        state.units.len(),
        state.cities.len(),
        state
            .players
            .iter()
            .map(|p| p.techs.len())
            .collect::<Vec<_>>()
    );
    Case {
        name,
        seed: state.setup.seed.to_string(),
        difficulty,
        turn: state.turn,
        units: state.units.len(),
        cities: state.cities.len(),
        techs: state.players.iter().map(|p| p.techs.len()).collect(),
        synthetic,
    }
}

fn record(root: &Path) {
    assert_eq!(TechId::ALL.len(), 32);
    fs::create_dir(root).expect("record into a new directory; never overwrite the baseline");
    fs::create_dir(root.join("tapes")).unwrap();
    let mut cases = Vec::new();
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        for seed in [1, 7, 42] {
            let mut setup = GameSetup::all_ai(seed);
            setup.width = 60;
            setup.height = 40;
            setup.players[0].is_human = true;
            for player in &mut setup.players {
                player.difficulty = difficulty;
            }
            let mut session = Session::new(&setup, |_| BrainKind::Scripted);
            let mut tape = Tape {
                version: legatum_wasm::tape::TAPE_VERSION,
                setup,
                initial_hash: session.state.hash(),
                turns: Vec::new(),
            };
            let mut last_full = None;
            while !session.state.finished && session.state.turn <= 1200 {
                let state = &session.state;
                if state.turn >= 300 && state.players.iter().all(|p| p.alive) {
                    if state.turn.is_multiple_of(150) {
                        cases.push(record_case(root, state, false));
                    }
                    last_full = Some(state.clone());
                }
                let human = human_orders(state);
                let mut orders = session.ai_orders();
                orders.extend(human);
                let turn = session.state.turn;
                session.step(&orders);
                tape.turns.push(TapeTurn {
                    turn,
                    orders,
                    hash: session.state.hash().to_string(),
                });
                if turn.is_multiple_of(150) {
                    eprintln!("{difficulty:?} seed {seed}: resolved {turn}");
                }
            }
            let mut last_full = last_full.expect("game must reach turn 300 with eight live seats");
            if !last_full.turn.is_multiple_of(150) {
                cases.push(record_case(root, &last_full, false));
            }
            // Explicit supplemental stress case: retain the last natural eight-seat world,
            // grant all 32 technologies, and refresh sight. This is not a natural playthrough.
            for player in &mut last_full.players {
                player.techs = TechId::ALL.into_iter().collect();
                player.researching = None;
            }
            visibility::update_explored(&mut last_full);
            cases.push(record_case(root, &last_full, true));
            write(
                &root
                    .join("tapes")
                    .join(format!("{difficulty:?}-{seed}.json")),
                tape.to_json(),
            );
            eprintln!(
                "finished {difficulty:?} seed {seed}: {} turns, finished={}, last eight-seat turn={}",
                tape.turns.len(),
                session.state.finished,
                last_full.turn
            );
        }
    }
    write(
        &root.join("manifest.json"),
        serde_json::to_string_pretty(&Manifest {
            version: 1,
            technologies: 32,
            cases,
        })
        .unwrap(),
    );
}

fn measure(root: &Path, samples: usize, warmup: usize) {
    let manifest: Manifest = serde_json::from_str(&read(root, "manifest.json")).unwrap();
    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.technologies, TechId::ALL.len());
    assert!(samples > 0 && warmup > 0);
    println!("case,turn,units,cities,metric,sample,ns");
    for case in manifest.cases {
        let dir = root.join(&case.name);
        let save = read(&dir, "state.json");
        let state = json::parse_state(&save).unwrap();
        let human_json = read(&dir, "human.json");
        let human: OrderMap = serde_json::from_str(&human_json).unwrap();
        let orders: OrderMap = serde_json::from_str(&read(&dir, "orders.json")).unwrap();
        assert_eq!((state.map.width, state.map.height), (60, 40));
        assert!(state.turn >= 300 && !state.finished);
        assert_eq!(
            state
                .players
                .iter()
                .filter(|p| p.alive && !p.is_human)
                .count(),
            7
        );
        let mut session = Session::from_state(state.clone(), |_| BrainKind::Scripted);
        let mut actual = session.ai_orders();
        actual.extend(human.clone());
        assert_eq!(
            serde_json::to_string(&actual).unwrap(),
            read(&dir, "orders.json")
        );
        assert_eq!(
            json::state_to_json(&resolve_turn(&state, &orders).0),
            read(&dir, "next.json")
        );
        let mut game = Game::from_save_json(&save).unwrap();
        for id in 0..8 {
            assert_eq!(
                game.view_json(id).unwrap(),
                read(&dir, &format!("view-{id}.json"))
            );
        }
        assert_eq!(
            game.end_turn_json(&human_json).unwrap(),
            read(&dir, "report.json")
        );
        assert_eq!(game.save_json(), read(&dir, "next.json"));
        eprintln!(
            "verified {}: AI orders, state, report, eight views",
            case.name
        );

        let mut metrics: Vec<_> = ["resolve", "ai_orders", "total", "end_turn"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        for id in 0..8 {
            metrics.push(format!("player_view_{id}"));
        }
        for sample in 0..warmup + samples {
            // Rotate to avoid permanently giving one operation the warmest caches.
            for offset in 0..metrics.len() {
                let metric = metrics[(sample + offset) % metrics.len()].as_str();
                // Reset outside the timer; include the ordinary production internal clones.
                let mut game = (metric == "end_turn").then(|| Game::from_save_json(&save).unwrap());
                let viewer = metric
                    .strip_prefix("player_view_")
                    .map(|id| PlayerId(id.parse().unwrap()));
                let start = Instant::now();
                match metric {
                    "resolve" => {
                        black_box(resolve_turn(black_box(&state), black_box(&orders)));
                    }
                    "ai_orders" => {
                        black_box(session.ai_orders());
                    }
                    "total" => {
                        let mut all = session.ai_orders();
                        all.extend(human.clone());
                        black_box(session.step(&all));
                    }
                    "end_turn" => {
                        black_box(
                            game.as_mut()
                                .unwrap()
                                .end_turn_json(black_box(&human_json))
                                .unwrap(),
                        );
                    }
                    _ => {
                        black_box(player_view(black_box(&state), viewer.unwrap()));
                    }
                }
                let elapsed = start.elapsed().as_nanos();
                if sample >= warmup {
                    println!(
                        "{},{},{},{},{},{},{}",
                        case.name,
                        case.turn,
                        case.units,
                        case.cities,
                        metric,
                        sample - warmup,
                        elapsed
                    );
                }
                // total advanced this scratch session; restore it before any later AI timer.
                if metric == "total" {
                    session = Session::from_state(state.clone(), |_| BrainKind::Scripted);
                }
            }
        }
    }
}

fn main() {
    if cfg!(debug_assertions) {
        panic!("use --release for this benchmark");
    }
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 3,
        "perf_full_tree <record|measure> DIR [samples=31] [warmup=5]"
    );
    let root = Path::new(&args[2]);
    match args[1].as_str() {
        "record" => record(root),
        "measure" => measure(
            root,
            args.get(3).map_or(31, |v| v.parse().unwrap()),
            args.get(4).map_or(5, |v| v.parse().unwrap()),
        ),
        _ => panic!("unknown mode"),
    }
}
