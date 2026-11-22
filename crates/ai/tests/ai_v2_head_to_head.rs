//! Same-engine, paired policy comparison. See the generated report for the full protocol.
//! AI_V2_PAIRS=32 cargo test --release -p legatum-ai --test ai_v2_head_to_head paired_evaluation -- --ignored --exact --nocapture

mod ai_v1_baseline;

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::Rng;
use legatum_core::*;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::time::Instant;

const BASELINE_SHA: &str = "b638adfb0a72990cd78b59c4177c171d9aa39855";
const V1: usize = 0;
const V2: usize = 1;

fn policy(seat: u8, swap: bool) -> usize {
    (seat.is_multiple_of(2) != swap) as usize
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Counts {
    decisions: u64,
    orders: u64,
    invalid: u64,
    failed: u64,
    battles: u64,
    late_battles: u64,
    captures: u64,
    late_captures: u64,
    eliminated: u64,
    bounces: u64,
    repeat_bounces: u64,
    max_bounce_repeats: u64,
    max_bounce_streak: u64,
    blocked: u64,
    cities: u64,
    techs: u64,
    vp: u64,
}

impl Counts {
    fn add(&mut self, other: &Self) {
        self.decisions += other.decisions;
        self.orders += other.orders;
        self.invalid += other.invalid;
        self.failed += other.failed;
        self.battles += other.battles;
        self.late_battles += other.late_battles;
        self.captures += other.captures;
        self.late_captures += other.late_captures;
        self.eliminated += other.eliminated;
        self.bounces += other.bounces;
        self.repeat_bounces += other.repeat_bounces;
        self.max_bounce_repeats = self.max_bounce_repeats.max(other.max_bounce_repeats);
        self.max_bounce_streak = self.max_bounce_streak.max(other.max_bounce_streak);
        self.blocked += other.blocked;
        self.cities += other.cities;
        self.techs += other.techs;
        self.vp += other.vp;
    }

    fn json(&self) -> String {
        format!(
            concat!(
                "{{\"decisions\":{},\"orders\":{},\"invalid_orders\":{},",
                "\"resolution_failures\":{},\"initiated_battles\":{},\"late_initiated_battles\":{},",
                "\"captures\":{},\"late_captures\":{},\"eliminated_seats\":{},",
                "\"friendly_bounces\":{},\"repeated_bounces\":{},\"max_bounces_same_units_tile\":{},\"max_consecutive_bounce_streak\":{},",
                "\"move_blocked\":{},\"final_cities\":{},\"final_techs\":{},\"final_vp\":{}}}"
            ),
            self.decisions,
            self.orders,
            self.invalid,
            self.failed,
            self.battles,
            self.late_battles,
            self.captures,
            self.late_captures,
            self.eliminated,
            self.bounces,
            self.repeat_bounces,
            self.max_bounce_repeats,
            self.max_bounce_streak,
            self.blocked,
            self.cities,
            self.techs,
            self.vp
        )
    }
}

#[derive(Default)]
struct GameResult {
    seed: u64,
    swap: bool,
    turns: u32,
    winner: Option<usize>,
    winner_seat: Option<u8>,
    outcome: String,
    final_hash: u64,
    counts: [Counts; 2],
    times_ns: [Vec<u64>; 2],
    issues: [Vec<String>; 2],
    factions: Vec<Faction>,
}

fn play(seed: u64, swap: bool, limit: u32) -> GameResult {
    let mut setup = GameSetup::all_ai(seed);
    setup.turn_limit = Some(limit);
    let mut state = new_game(&setup);
    let mut brains: Vec<Box<dyn Brain>> = state
        .players
        .iter()
        .map(|p| {
            let brain_seed = seed ^ u64::from(p.id.0);
            if policy(p.id.0, swap) == V2 {
                Box::new(ScriptedBrain::new(p.faction, brain_seed, p.id)) as Box<dyn Brain>
            } else {
                Box::new(ai_v1_baseline::ScriptedBrain::new(
                    p.faction, brain_seed, p.id,
                )) as Box<dyn Brain>
            }
        })
        .collect();
    let mut result = GameResult {
        seed,
        swap,
        factions: state.players.iter().map(|p| p.faction).collect(),
        ..Default::default()
    };
    // Same sorted unit ids and tile define a bounce. Record both total repeats and consecutive
    // turn streaks; a later unrelated bounce is not described as a continuous loop.
    let mut bounces: BTreeMap<(Vec<UnitId>, Hex), (u32, u64, u64)> = BTreeMap::new();
    while !state.finished && result.turns < limit {
        let mut all = BTreeMap::new();
        for p in &state.players {
            if !p.alive {
                continue;
            }
            let version = policy(p.id.0, swap);
            let view = player_view(&state, p.id);
            let start = Instant::now();
            let orders = brains[p.id.0 as usize].orders(&view);
            result.times_ns[version].push(start.elapsed().as_nanos() as u64);
            // Determinism is checked on EVERY measured decision, outside the timing interval.
            assert_eq!(
                orders,
                brains[p.id.0 as usize].orders(&view),
                "seed {seed} turn {} seat {}",
                state.turn,
                p.id.0
            );
            let c = &mut result.counts[version];
            c.decisions += 1;
            c.orders += orders.len() as u64;
            for error in validate_orders(&state, p.id, &orders) {
                c.invalid += 1;
                if result.issues[version].len() < 12 {
                    result.issues[version].push(format!(
                        "turn {} seat {} invalid {:?}: {}",
                        state.turn, p.id.0, orders[error.index], error.message
                    ));
                }
            }
            all.insert(p.id, orders);
        }
        let (next, events) = resolve_turn(&state, &all);
        let late = state.turn * 3 >= limit * 2;
        for event in events {
            match event {
                Event::Battle { attacker_owner, .. } => {
                    let c = &mut result.counts[policy(attacker_owner.0, swap)];
                    c.battles += 1;
                    c.late_battles += u64::from(late);
                }
                Event::CityCaptured { to, .. } => {
                    let c = &mut result.counts[policy(to.0, swap)];
                    c.captures += 1;
                    c.late_captures += u64::from(late);
                }
                Event::PlayerEliminated { player } => {
                    result.counts[policy(player.0, swap)].eliminated += 1
                }
                Event::MoveBlocked { owner, .. } => {
                    result.counts[policy(owner.0, swap)].blocked += 1
                }
                Event::OrderFailed { player, reason } => {
                    let v = policy(player.0, swap);
                    result.counts[v].failed += 1;
                    if result.issues[v].len() < 12 {
                        result.issues[v].push(format!(
                            "turn {} seat {} failed: {reason}",
                            state.turn, player.0
                        ));
                    }
                }
                Event::Bounce { mut units, tile } => {
                    units.sort();
                    let owners: Vec<_> = units
                        .iter()
                        .filter_map(|u| state.units.get(u).map(|u| u.owner))
                        .collect();
                    if let Some(owner) = owners.first()
                        && owners.iter().all(|o| o == owner)
                    {
                        let c = &mut result.counts[policy(owner.0, swap)];
                        c.bounces += 1;
                        let (last, streak, count) =
                            bounces.entry((units.clone(), tile)).or_default();
                        c.repeat_bounces += u64::from(*count > 0);
                        *streak = if *count > 0 && *last + 1 == state.turn {
                            *streak + 1
                        } else {
                            1
                        };
                        *last = state.turn;
                        *count += 1;
                        c.max_bounce_repeats = c.max_bounce_repeats.max(*count);
                        if *count > 1 && std::env::var_os("AI_V2_TRACE_BOUNCES").is_some() {
                            println!(
                                "repeat {count} seed {seed} swap {swap} turn {} tile {tile:?}",
                                state.turn
                            );
                            for id in &units {
                                println!("  {:?} orders {:?}", state.units.get(id), all.get(owner));
                            }
                        }
                        c.max_bounce_streak = c.max_bounce_streak.max(*streak);
                    }
                }
                Event::GameWon { reason, .. } | Event::GameDrawn { reason } => {
                    result.outcome = reason
                }
                _ => {}
            }
        }
        result.turns += 1;
        state = next;
    }
    assert!(state.finished, "runner must not score an unfinished game");
    result.winner_seat = state.winner.map(|t| t.0); // all_ai uses one seat per team
    result.winner = result.winner_seat.map(|p| policy(p, swap));
    result.final_hash = state.hash();
    for p in &state.players {
        let c = &mut result.counts[policy(p.id.0, swap)];
        c.cities += state.cities_of(p.id).count() as u64;
        c.techs += p.techs.len() as u64;
        c.vp += u64::from(legatum_core::scoring::vp_total(&state, p.id));
    }
    result
}

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn timing(values: &[u64]) -> (u64, u64, u64) {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    if n == 0 {
        return (0, 0, 0);
    }
    (
        sorted.iter().sum::<u64>() / n as u64,
        sorted[n / 2],
        sorted[(n - 1) * 95 / 100],
    )
}

fn timing_json(values: &[u64]) -> String {
    let (mean, median, p95) = timing(values);
    format!("{{\"mean_ns\":{mean},\"median_ns\":{median},\"p95_ns\":{p95}}}")
}

impl GameResult {
    fn json(&self) -> String {
        let winner = self
            .winner
            .map_or("null".into(), |v| quote(if v == V2 { "v2" } else { "v1" }));
        let issues = self
            .issues
            .iter()
            .map(|a| {
                format!(
                    "[{}]",
                    a.iter().map(|s| quote(s)).collect::<Vec<_>>().join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{\"seed\":{},\"swap\":{},\"turns\":{},\"winner\":{},\"outcome\":{},",
                "\"final_hash\":\"{}\",\"v1\":{},\"v2\":{},\"v1_timing\":{},\"v2_timing\":{},\"issues_v1_v2\":[{}]}}"
            ),
            self.seed,
            self.swap,
            self.turns,
            winner,
            quote(&self.outcome),
            self.final_hash,
            self.counts[V1].json(),
            self.counts[V2].json(),
            timing_json(&self.times_ns[V1]),
            timing_json(&self.times_ns[V2]),
            issues
        )
    }
}

// Sampling uncertainty is over seeds, not over 512 correlated seats or 64 independent games.
// Each pair contributes two game scores: v2 win=1, draw=0.5, v1 win=0.
fn paired_interval(games: &[GameResult]) -> (f64, f64) {
    let scores: Vec<u64> = games
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            pair.iter()
                .map(|g| match g.winner {
                    Some(V2) => 2,
                    None => 1,
                    _ => 0,
                })
                .sum()
        })
        .collect();
    let mut rng = Rng::derive(0xB9, &[0x4349]);
    let mut samples = Vec::with_capacity(20_000);
    for _ in 0..20_000 {
        let sum: u64 = (0..scores.len())
            .map(|_| scores[rng.range(0, scores.len() as i64 - 1) as usize])
            .sum();
        samples.push(sum);
    }
    samples.sort_unstable();
    let denominator = (4 * scores.len()) as f64;
    (
        samples[500] as f64 / denominator,
        samples[19_499] as f64 / denominator,
    )
}

fn git(args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args(args)
        .output()
        .expect("git installed");
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn report(games: &[GameResult]) {
    let n = games.len();
    let mut counts: [Counts; 2] = Default::default();
    let mut times: [Vec<u64>; 2] = Default::default();
    let mut wins = [0; 2];
    let mut faction: BTreeMap<String, [[u64; 2]; 2]> = BTreeMap::new();
    for game in games {
        if let Some(v) = game.winner {
            wins[v] += 1;
        }
        for v in [V1, V2] {
            counts[v].add(&game.counts[v]);
            times[v].extend(&game.times_ns[v]);
        }
        for (seat, f) in game.factions.iter().enumerate() {
            let entry =
                &mut faction.entry(format!("{f:?}")).or_default()[policy(seat as u8, game.swap)];
            entry[1] += 1;
            entry[0] += u64::from(game.winner_seat == Some(seat as u8));
        }
    }
    let draws = n - wins[V1] - wins[V2];
    let (lo, hi) = paired_interval(games);
    let score = (wins[V2] as f64 + draws as f64 / 2.0) / n as f64;
    let head = git(&["rev-parse", "HEAD"]);
    let core_tree = git(&["rev-parse", "HEAD:crates/core"]);
    let policy_diff = git(&["diff", "HEAD", "--", "crates/ai/src"]);
    assert!(
        git(&["diff", "HEAD", "--", "crates/core"]).is_empty(),
        "the paired engine must match its recorded tree"
    );
    let mut paths: Vec<String> = [
        "analysis",
        "civilians",
        "military",
        "movement",
        "production",
        "research",
        "scripted",
        "knobs",
        "tests",
    ]
    .iter()
    .map(|name| format!("crates/ai/src/{name}.rs"))
    .collect();
    paths.push("crates/ai/tests/ai_v2_head_to_head.rs".into());
    let source_blobs = paths
        .iter()
        .map(|path| format!("{}:{}", quote(path), quote(&git(&["hash-object", path]))))
        .collect::<Vec<_>>()
        .join(",");
    let base = std::env::var("AI_V2_REPORT")
        .unwrap_or_else(|_| "docs/sim-results/2026-09-26-ai-v2-head-to-head".into());
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(base);
    let command = format!(
        "CARGO_NET_OFFLINE=true AI_V2_PAIRS={} cargo test --release -p legatum-ai --test ai_v2_head_to_head paired_evaluation -- --ignored --exact --nocapture",
        n / 2
    );
    let adoption = if lo > 0.5 && counts[V2].invalid == 0 && counts[V2].max_bounce_repeats <= 2 {
        "The paired interval favors v2 in this sample. Adoption still needs review of failures, runtime, and generalization to other seed blocks and team games."
    } else {
        "Strength improvement is NOT established for adoption: the paired interval and operational regressions must be reviewed. Keep adoption contingent on further evidence."
    };
    let mut md = format!(
        "# AI v2 versus frozen v1, same engine\n\nCommand:\n\n```sh\n{command}\n```\n\nBaseline source `{BASELINE_SHA}` (identical AI policy to merge `94f9404`); internal import paths were adapted and obsolete focus orders were removed to match the current engine. Both brains link one engine in one binary. Engine/starting HEAD `{head}`, core tree `{core_tree}`. Policy has uncommitted changes at run: {}. JSON records the evaluated working-tree source blob hashes; HEAD identifies the engine/base revision when the policy is dirty.\n\n{} paired seeds, fixed contiguous seeds 1..={}, {} completed games on the standard 48x32 map, 8 independent empires, 150-turn limit. Four v2 and four v1 seats alternate in the first game; all policies exchange seats in the second, inheriting each seat's faction and identical brain RNG seed. Map, faction schedule, seat RNG, and engine are unchanged within each pair. This is an eight-way free-for-all comparison, not a faction balance probe or a two-player duel. No seeds are discarded.\n\nEvery decision is repeated and compared exactly. Only the first call is timed; PlayerView creation, validation, repeated checks, and resolution are excluded. Runs are serial in release mode. Timing is machine/load dependent.\n\n## Outcomes and uncertainty\n\nv2 wins {}/{n} ({:.1}%), v1 wins {}/{n} ({:.1}%), draws {draws}/{n}. v2 draw-adjusted game score {:.1}%; 95% paired seed bootstrap interval [{:.1}%, {:.1}%] (20,000 fixed-RNG resamples of seed pairs). Resampling keeps the two games together; seats and paired games are not assumed independent.\n\n{adoption}\n\n| Metric | v1 | v2 |\n| --- | ---: | ---: |\n",
        !policy_diff.is_empty(),
        n / 2,
        n / 2,
        n,
        wins[V2],
        wins[V2] as f64 * 100.0 / n as f64,
        wins[V1],
        wins[V1] as f64 * 100.0 / n as f64,
        score * 100.0,
        lo * 100.0,
        hi * 100.0
    );
    for (label, a, b) in [
        ("Decisions", counts[0].decisions, counts[1].decisions),
        ("Submitted orders", counts[0].orders, counts[1].orders),
        (
            "Invalid orders (validation)",
            counts[0].invalid,
            counts[1].invalid,
        ),
        (
            "Resolution OrderFailed events",
            counts[0].failed,
            counts[1].failed,
        ),
        ("Initiated battles", counts[0].battles, counts[1].battles),
        (
            "Initiated battles, turn >= 100",
            counts[0].late_battles,
            counts[1].late_battles,
        ),
        ("City captures", counts[0].captures, counts[1].captures),
        (
            "City captures, turn >= 100",
            counts[0].late_captures,
            counts[1].late_captures,
        ),
        (
            "Own seats eliminated",
            counts[0].eliminated,
            counts[1].eliminated,
        ),
        (
            "Friendly bounce events",
            counts[0].bounces,
            counts[1].bounces,
        ),
        (
            "Repeat bounce events (same sorted units + tile)",
            counts[0].repeat_bounces,
            counts[1].repeat_bounces,
        ),
        (
            "Longest consecutive-turn bounce streak",
            counts[0].max_bounce_streak,
            counts[1].max_bounce_streak,
        ),
        ("MoveBlocked events", counts[0].blocked, counts[1].blocked),
        (
            "Most bounces of the same units + tile (any turns)",
            counts[0].max_bounce_repeats,
            counts[1].max_bounce_repeats,
        ),
        (
            "Final cities (all seats)",
            counts[0].cities,
            counts[1].cities,
        ),
        ("Final techs (all seats)", counts[0].techs, counts[1].techs),
    ] {
        writeln!(md, "| {label} | {a} | {b} |").unwrap();
    }
    let t1 = timing(&times[0]);
    let t2 = timing(&times[1]);
    for (label, a, b) in [
        ("Mean decision ns", t1.0, t2.0),
        ("Median decision ns", t1.1, t2.1),
        ("P95 decision ns", t1.2, t2.2),
    ] {
        writeln!(md, "| {label} | {a} | {b} |").unwrap();
    }
    md.push_str("\nBattles count initiating orders, captures count transfers (including recaptures), and eliminations count victims. Resolution failures may include legitimate simultaneous founding conflicts or units captured before acting; they are not silently removed. MoveBlocked includes combat/occupancy outcomes. Bounce repeats include nonconsecutive recurrence; the separate streak column identifies consecutive loops. Late-event totals depend on how many games reach turn 100.\n\n| Faction | v1 wins/seats | v2 wins/seats |\n| --- | ---: | ---: |\n");
    for (f, entry) in &faction {
        writeln!(
            md,
            "| {f} | {}/{} | {}/{} |",
            entry[0][0], entry[0][1], entry[1][0], entry[1][1]
        )
        .unwrap();
    }
    md.push_str("\n## Limits and handoff\n\nThe older ai-v1-war report is context, not a control: its self-play used a different engine revision. This experiment isolates policy in the current engine. The frozen v1 includes the two reviewed AI defects. A mixed-policy free-for-all can behave differently from self-play or human team games; confidence intervals do not cover that distribution shift. More combat is not automatically stronger play. Reachability uses explored land, so v2 can delay routes that require an unobserved shortcut. The AI cannot predict opponents' simultaneous orders. See JSON for every seed, final hash, and bounded examples of validation/resolution failures.\n");
    let json = format!(
        concat!(
            "{{\"baseline_source_sha\":{},\"engine_head\":{},\"core_tree\":{},\"dirty_policy\":{},",
            "\"source_blobs\":{{{}}},\"command\":{},\"pairs\":{},\"games_count\":{},\"v1_wins\":{},\"v2_wins\":{},\"draws\":{},",
            "\"v2_score\":{},\"paired_bootstrap_95\":[{},{}],\"v1\":{},\"v2\":{},\"v1_timing\":{},\"v2_timing\":{},\"games\":[{}]}}\n"
        ),
        quote(BASELINE_SHA),
        quote(&head),
        quote(&core_tree),
        !policy_diff.is_empty(),
        source_blobs,
        quote(&command),
        n / 2,
        n,
        wins[V1],
        wins[V2],
        draws,
        score,
        lo,
        hi,
        counts[V1].json(),
        counts[V2].json(),
        timing_json(&times[V1]),
        timing_json(&times[V2]),
        games
            .iter()
            .map(GameResult::json)
            .collect::<Vec<_>>()
            .join(",\n")
    );
    std::fs::write(base.with_extension("md"), md).unwrap();
    std::fs::write(base.with_extension("json"), json).unwrap();
    println!(
        "{} pairs: v2 {}/{n}, v1 {}/{n}, draws {draws}; paired 95% score interval [{:.1}%, {:.1}%]. Reports: {}",
        n / 2,
        wins[V2],
        wins[V1],
        lo * 100.0,
        hi * 100.0,
        base.display()
    );
}

#[test]
fn paired_layout_exchanges_every_seat_and_faction() {
    for seed in 1..=32 {
        for seat in 0..8 {
            assert_ne!(policy(seat, false), policy(seat, true));
            let setup = GameSetup::all_ai(seed);
            assert_eq!(setup.players[seat as usize].team, TeamId(seat));
        }
    }
}

#[test]
fn paired_smoke_replays_the_same_engine() {
    let first = play(250, false, 12);
    let repeat = play(250, false, 12);
    assert_eq!(first.final_hash, repeat.final_hash);
    assert_eq!(first.counts, repeat.counts);
    assert_eq!(first.counts[V2].invalid, 0);
    let swapped = play(250, true, 12);
    assert_eq!(swapped.counts[V2].invalid, 0);
}

#[test]
fn repeated_bounce_regression_seed_26() {
    let game = play(26, false, 150);
    assert_eq!(game.counts[V2].invalid, 0, "{:?}", game.issues[V2]);
    assert!(
        game.counts[V2].max_bounce_repeats <= 2,
        "{:?}",
        game.counts[V2]
    );
}

#[test]
#[ignore = "release evaluation: at least 32 paired seeds / 64 complete games"]
fn paired_evaluation() {
    if cfg!(debug_assertions) {
        panic!("run this evaluation with --release");
    }
    let pairs: u64 = std::env::var("AI_V2_PAIRS").map_or(32, |s| s.parse().expect("integer pairs"));
    assert!(
        pairs >= 32,
        "do not report strength from fewer than 32 matched seeds"
    );
    let mut games = Vec::new();
    for seed in 1..=pairs {
        for swap in [false, true] {
            let game = play(seed, swap, 150);
            println!(
                "seed {seed} swap {swap}: winner {:?}; v1/v2 invalid {}/{}",
                game.winner, game.counts[0].invalid, game.counts[1].invalid
            );
            games.push(game);
        }
    }
    report(&games);
    assert!(
        games.iter().all(|g| g.counts[V2].invalid == 0),
        "v2 invalid orders: inspect report"
    );
    assert!(
        games.iter().all(|g| g.counts[V2].max_bounce_repeats <= 2),
        "v2 repeated bounce loop: inspect report"
    );
}
