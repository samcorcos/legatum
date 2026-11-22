//! Training games for the learned value network (`legatum_ai::value_net`, AI war plan step 3).
//!
//! Plays all-AI Hard games (the app's plain Hard brain on every seat, or `--ai-params` on every
//! seat) and every `--every` turns logs, for each alive seat, the network's inputs read from that
//! seat's own `PlayerView` (through `WinFacts`), the rule-based hold overlay, the hand formula's
//! estimates and, once the game ends, the winning team. Games without a winner are counted and
//! dropped. Rows are fixed-width little-endian records described by `value-schema.json`;
//! `tools/sim/train-value.py` reads them with numpy.
//!
//!   value_data --games 200 --seed 8800001 [--players 8,4] [--every 10] [--ai-params FILE]
//!              [--turn-cap 1000] [--label L] [--out-dir D]
//!
//! Cloud (arguments match `tools/cloud/run-batch.sh --bin`; `--out` and `--status-file` are
//! accepted and ignored):
//!   LEGATUM_CLOUD_EXAMPLES=value_data tools/cloud/build-linux.sh
//!   tools/cloud/run-batch.sh --bin value_data --max-hours 2 -- --games 20000 --seed 8800001 \
//!       --players 8,8,8,4 --label value-v1
use legatum_ai::value_net::{self, INPUTS, MAX_TEAMS};
use legatum_ai::winchance::WinFacts;
use legatum_ai::{AiParams, DecisionParams, ScriptedBrain};
use legatum_core::{AiDifficulty, Faction, GameSetup, player_view};
use legatum_wasm::session::{BrainKind, Session, brain_seed};
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Bumped when the row layout changes.
const ROW_VERSION: u32 = 1;

struct Row {
    turn: i16,
    seat: i16,
    faction: i16,
    n_teams: i16,
    me: i16,
    team_ids: Vec<legatum_core::TeamId>,
    /// The estimate Hard's plan uses today (shared weights, the faction's VP race genes).
    hand: i16,
    /// The weights fitted for this seat count, without race terms (search rollouts).
    fitted: i16,
    /// This seat's VP race term under its genes (milli-logits), added to its own score.
    race: i16,
    rest: i16,
    hold: [i16; MAX_TEAMS],
    x: [[i16; INPUTS]; MAX_TEAMS],
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn sample(view: &legatum_core::PlayerView, faction: Faction) -> Option<Row> {
    let facts = WinFacts::of(view);
    let n = facts.teams.len();
    if n == 0 || n > MAX_TEAMS {
        return None;
    }
    let me = facts.teams.iter().position(|t| t.team == facts.me)?;
    let planned = facts
        .clone()
        .with_relative(&DecisionParams::builtin_for(faction));
    let mut planned_hand = planned.clone();
    planned_hand.learned = 0;
    let (holds, rest) = value_net::hold_overlay(&facts);
    let inputs = value_net::inputs(&facts);
    let mut x = [[0i16; INPUTS]; MAX_TEAMS];
    let mut hold = [0i16; MAX_TEAMS];
    for (i, (_, v)) in inputs.iter().enumerate() {
        x[i] = v.map(|e| e as i16);
    }
    for (i, (_, q)) in holds.iter().enumerate() {
        hold[i] = *q as i16;
    }
    Some(Row {
        turn: view.turn as i16,
        seat: view.me.0 as i16,
        faction: Faction::ALL.iter().position(|f| *f == faction).unwrap_or(0) as i16,
        n_teams: n as i16,
        me: me as i16,
        team_ids: facts.teams.iter().map(|t| t.team).collect(),
        hand: planned_hand.chance() as i16,
        fitted: facts.search_chance() as i16,
        race: planned.race_term().clamp(-30_000, 30_000) as i16,
        rest: rest as i16,
        hold,
        x,
    })
}

struct Game {
    index: usize,
    seed: u64,
    players: usize,
    finished: bool,
    winner: Option<u8>,
    turns: u32,
    wall_ms: u64,
    rows: Vec<Row>,
}

fn play(
    index: usize,
    seed: u64,
    players: usize,
    every: u32,
    cap: u32,
    params: Option<&AiParams>,
) -> Game {
    let start = Instant::now();
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    (setup.width, setup.height) = (48, 32);
    // One seat per faction, shuffled from the seed as the simulator's runner does.
    let mut factions = Faction::ALL;
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (player, faction) in setup.players.iter_mut().zip(factions) {
        player.faction = faction;
        player.difficulty = AiDifficulty::Hard;
    }
    let mut s = Session::new(&setup, |_| BrainKind::Scripted);
    if let Some(params) = params {
        for p in s.state.players.clone() {
            let brain = ScriptedBrain::with_params(
                p.faction,
                brain_seed(setup.seed, p.id),
                p.id,
                AiDifficulty::Hard,
                params,
            );
            s.replace_brain(p.id, Box::new(brain));
        }
    }
    let mut rows = Vec::new();
    while !s.state.finished && s.state.turn <= cap {
        if s.state.turn.is_multiple_of(every) {
            for p in s.state.players.iter().filter(|p| p.alive) {
                if let Some(row) = sample(&player_view(&s.state, p.id), p.faction) {
                    rows.push(row);
                }
            }
        }
        let orders = s.ai_orders();
        s.step(&orders);
    }
    let winner = s.state.winner;
    // A row is labelled by the winner's slot in that row's team list; a winner not alive at
    // the sample (impossible today) drops the row.
    let rows = match winner {
        Some(w) => rows
            .into_iter()
            .filter(|r| r.team_ids.contains(&w))
            .collect(),
        None => Vec::new(),
    };
    Game {
        index,
        seed,
        players,
        finished: s.state.finished,
        winner: winner.map(|t| t.0),
        turns: s.state.turn.saturating_sub(1),
        wall_ms: start.elapsed().as_millis() as u64,
        rows,
    }
}

fn encode(g: &Game, out: &mut Vec<u8>) {
    let Some(w) = g.winner else { return };
    for r in &g.rows {
        let winner = r.team_ids.iter().position(|t| t.0 == w).unwrap_or(0) as i16;
        out.extend_from_slice(&(g.index as i32).to_le_bytes());
        out.extend_from_slice(&g.seed.to_le_bytes());
        for v in [
            r.turn,
            g.players as i16,
            r.seat,
            r.faction,
            r.n_teams,
            r.me,
            winner,
            r.hand,
            r.fitted,
            r.race,
            r.rest,
        ] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for v in r.hold {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for team in &r.x {
            for v in team {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "row_version": ROW_VERSION,
        "inputs_version": value_net::VERSION,
        "inputs": INPUTS,
        "max_teams": MAX_TEAMS,
        "input_names": value_net::INPUT_NAMES,
        "factions": Faction::ALL.iter().map(|f| format!("{f:?}")).collect::<Vec<_>>(),
        "dtype": [
            ["game", "<i4"], ["seed", "<u8"], ["turn", "<i2"], ["players", "<i2"], ["seat", "<i2"],
            ["faction", "<i2"], ["n_teams", "<i2"], ["me", "<i2"], ["winner", "<i2"],
            ["hand", "<i2"], ["fitted", "<i2"], ["race", "<i2"], ["rest", "<i2"],
            ["hold", "<i2", [MAX_TEAMS]], ["x", "<i2", [MAX_TEAMS, INPUTS]]
        ],
        "notes": "x: per-team inputs in WinFacts team order (rows past n_teams are zero); \
                  hold: each team's share (permille) of a completing VP hold; rest: permille \
                  left for the logit; hand/fitted: chance x10000; race: milli-logits on me's score",
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = arg(&args, "--games").map_or(16, |v| v.parse().expect("--games"));
    let first_seed: u64 = arg(&args, "--seed").map_or(8_800_001, |v| v.parse().expect("--seed"));
    let every: u32 = arg(&args, "--every").map_or(10, |v| v.parse().expect("--every"));
    let cap: u32 = arg(&args, "--turn-cap").map_or(1000, |v| v.parse().expect("--turn-cap"));
    let label = arg(&args, "--label").unwrap_or_else(|| "value".into());
    let players: Vec<usize> = arg(&args, "--players")
        .unwrap_or_else(|| "8".into())
        .split(',')
        .map(|p| p.parse().expect("--players"))
        .collect();
    assert!(
        players.iter().all(|p| (2..=8).contains(p)),
        "--players 2..8"
    );
    let params: Option<&'static AiParams> = arg(&args, "--ai-params").map(|path| {
        let p: AiParams =
            serde_json::from_slice(&std::fs::read(&path).expect("read --ai-params")).expect("json");
        p.validate().expect("valid --ai-params");
        &*Box::leak(Box::new(p))
    });
    let dir = PathBuf::from(arg(&args, "--out-dir").unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir).expect("out dir");
    std::fs::write(
        dir.join("value-schema.json"),
        serde_json::to_vec_pretty(&schema()).unwrap(),
    )
    .expect("schema");
    let rows_file = Mutex::new(std::io::BufWriter::new(
        std::fs::File::create(dir.join(format!("value-rows-{label}.bin"))).expect("rows file"),
    ));
    let games_file = Mutex::new(
        std::fs::File::create(dir.join(format!("value-games-{label}.jsonl"))).expect("games file"),
    );
    let done = AtomicUsize::new(0);
    let start = Instant::now();
    (0..games).into_par_iter().for_each(|i| {
        let n = players[i % players.len()];
        let g = play(i, first_seed + i as u64, n, every, cap, params);
        let mut bytes = Vec::new();
        encode(&g, &mut bytes);
        rows_file
            .lock()
            .unwrap()
            .write_all(&bytes)
            .expect("write rows");
        let line = serde_json::json!({
            "game": g.index, "seed": g.seed, "players": g.players, "finished": g.finished,
            "winner": g.winner, "turns": g.turns, "rows": g.rows.len(), "wall_ms": g.wall_ms,
        });
        writeln!(games_file.lock().unwrap(), "{line}").expect("write games");
        let k = done.fetch_add(1, Ordering::Relaxed) + 1;
        if k == 1 || k.is_multiple_of(64) || k == games {
            eprintln!(
                "{label}: {k}/{games} games ({:.0} s)",
                start.elapsed().as_secs_f64()
            );
        }
    });
    rows_file.lock().unwrap().flush().expect("flush rows");
}
