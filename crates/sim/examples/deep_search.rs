//! Offline oracle on the two human playtest maps. The executable ignores the cloud launcher's
//! scheduling flags; one invocation runs the requested position and mode and writes JSON.
//! Local: cargo run --profile sim -p legatum-sim --example deep_search -- \
//!   --position 85-oath --mode sweep --samples 16 --out-dir /tmp/deep-search
//! Cloud: tools/cloud/run-batch.sh --max-hours 2 --bin deep_search --ai-params BUNDLE \
//!   -- --games 1 --position 85-oath --mode sweep --samples 16 --label deep-search
//! BUNDLE is JSON {"p85": <Recording>, "p84": <Recording>}; it is transferred as an opaque
//! input by the batch launcher, not interpreted as live AI params.
use legatum_ai::learned_macro;
use legatum_ai::openings::Opening;
use legatum_ai::plan::PlanKind;
use legatum_ai::search::{OracleMetrics, OracleNode, SearchBrain, SearchConfig, SearchEvent};
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, GameState, PlayerId, player_view, resolve_turn};
use legatum_wasm::history::Recording;
use legatum_wasm::session::brain_seed;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct SweepRow {
    horizon: u32,
    event: SearchEvent,
}

struct Branch {
    node: OracleNode,
    line: Vec<String>,
    actions: Vec<MacroAction>,
    milestones: Vec<OracleMetrics>,
    score: i64,
}

#[derive(Clone)]
enum MacroAction {
    Opening(Opening),
    Continue,
    Plan(PlanKind),
}

#[derive(Clone, Copy)]
struct TreeConfig<'a> {
    position: &'a str,
    depth: usize,
    width: usize,
    sample: u64,
    search_rivals: bool,
}

fn get_arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn load_recordings(bundle: Option<&str>) -> Result<(Recording, Recording), String> {
    if let Some(path) = bundle {
        let data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let p85 = serde_json::from_value(data["p85"].clone()).map_err(|e| e.to_string())?;
        let p84 = serde_json::from_value(data["p84"].clone()).map_err(|e| e.to_string())?;
        return Ok((p85, p84));
    }
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let bug = Path::new(&home).join("LegatumRuns/bug-saves");
    let p85 = legatum_archive::read(&bug.join("playtest-8.5-mongols-win.tape.json.zst"))?;
    let raw = std::fs::read_to_string(bug.join("playtest-8.4-sparta-win.legatum"))
        .map_err(|e| e.to_string())?;
    let save: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let game: serde_json::Value =
        serde_json::from_str(save["game"].as_str().ok_or("8.4 save has no game JSON")?)
            .map_err(|e| e.to_string())?;
    let p84 = serde_json::from_value(game["archive"].clone()).map_err(|e| e.to_string())?;
    Ok((p85, p84))
}

fn position(recording: &Recording, turn: u32) -> Result<(GameState, usize), String> {
    let tape = &recording.tape;
    let mut state = match &recording.initial_state {
        Some(s) => legatum_wasm::json::parse_state(s)?,
        None => tape.initial_state(),
    };
    let mut mismatches = usize::from(state.hash() != tape.initial_hash);
    for t in tape.turns.iter().filter(|t| t.turn < turn) {
        if state.turn != t.turn {
            return Err(format!("missing turn {}", state.turn));
        }
        state = resolve_turn(&state, &t.orders).0;
        mismatches += usize::from(state.hash().to_string() != t.hash);
    }
    if state.turn != turn {
        return Err(format!("tape stops before turn {turn}"));
    }
    Ok((state, mismatches))
}

fn brain(state: &GameState, seat: PlayerId) -> ScriptedBrain {
    ScriptedBrain::with_difficulty(
        state.player(seat).faction,
        brain_seed(state.setup.seed, seat),
        seat,
        AiDifficulty::Hard,
    )
}

fn warmed_brain(recording: &Recording, state: &GameState, seat: PlayerId) -> ScriptedBrain {
    let mut base = brain(state, seat);
    if state.turn == 1 {
        return base;
    }
    let mut past = recording.tape.initial_state();
    let mut replay = SearchBrain::new(brain(&past, seat), SearchConfig::in_app(past.players.len()));
    for turn in recording.tape.turns.iter().filter(|t| t.turn < state.turn) {
        if past.player(seat).alive {
            replay.orders(&player_view(&past, seat));
        }
        past = resolve_turn(&past, &turn.orders).0;
    }
    if let Some(memory) = replay.opponent_memory() {
        base.restore_opponent_memory(memory);
    }
    base
}

fn sweep(
    state: &GameState,
    seat: PlayerId,
    base: &ScriptedBrain,
    samples: u8,
    horizons: &[u32],
) -> Vec<SweepRow> {
    let view = player_view(state, seat);
    let opening = state.turn == 1;
    horizons
        .iter()
        .copied()
        .map(|h| {
            let config = SearchConfig {
                horizon: h,
                opening_horizon: h,
                samples,
                candidates: 16,
                openings: opening,
                fronts: true,
                ..SearchConfig::default()
            };
            let mut search = SearchBrain::new(base.rollout_copy(), config);
            search.orders(&view);
            let event = search
                .search_events()
                .first()
                .cloned()
                .expect("a turn-one opening or turn-14 plan search");
            SweepRow { horizon: h, event }
        })
        .collect()
}

fn proxy(m: &OracleMetrics, target: u32) -> i64 {
    // A late proxy only ranks beam survivors. The final report keeps raw milestones and
    // distinguishes a real win from this estimate.
    if m.won {
        return 1_000_000 - i64::from(m.turn);
    }
    if m.finished {
        return -100_000;
    }
    i64::from(m.chance)
        + i64::from(m.vp) * 800
        + m.cities as i64 * 650
        + i64::from(m.population) * 25
        + i64::from(m.production + m.gold + m.science) * 10
        - m.rival_cities as i64 * 300
        + if m.turn >= target {
            0
        } else {
            i64::from(target - m.turn)
        }
}

fn milestone_bonus(node: &OracleNode, position: &str) -> i64 {
    let at = |turn| node.checkpoints().iter().find(|m| m.turn == turn);
    match position {
        "84-oath" => at(62).map_or(0, |m| m.cities.min(7) as i64 * 2_000),
        _ => {
            at(50).map_or(0, |m| m.cities.min(5) as i64 * 1_500)
                + at(60).map_or(0, |m| i64::from(m.vp.min(7)) * 1_500)
        }
    }
}

fn tree(
    recording: &Recording,
    state: &GameState,
    seat: PlayerId,
    base: &ScriptedBrain,
    config: TreeConfig<'_>,
) -> serde_json::Value {
    let TreeConfig {
        position,
        depth,
        width,
        sample,
        search_rivals,
    } = config;
    let view = player_view(state, seat);
    let mut start = OracleNode::new(&view, base, sample);
    start.set_search_rivals(search_rivals);
    let first = start.metrics();
    let mut beam = vec![Branch {
        node: start,
        line: Vec::new(),
        actions: Vec::new(),
        milestones: vec![first],
        score: 0,
    }];
    let mut expanded = 0usize;
    let mut best_by_depth = Vec::new();
    for step in 0..depth {
        let mut next = Vec::new();
        for mut branch in std::mem::take(&mut beam) {
            if step == 0 && state.turn == 1 {
                for opening in Opening::ALL {
                    let mut node = branch.node.fork();
                    node.advance_opening(opening, 10);
                    expanded += 1;
                    let m = node.metrics();
                    let mut line = branch.line.clone();
                    line.push(format!("T1 opening {opening:?}"));
                    let mut milestones = branch.milestones.clone();
                    milestones.push(m.clone());
                    let mut actions = branch.actions.clone();
                    actions.push(MacroAction::Opening(opening));
                    next.push(Branch {
                        score: proxy(&m, state.turn + (depth * 10) as u32)
                            + milestone_bonus(&node, position),
                        node,
                        line,
                        actions,
                        milestones,
                    });
                }
                continue;
            }
            if let Some(opening) = branch.node.active_opening() {
                let mut node = branch.node.fork();
                node.continue_opening(10);
                expanded += 1;
                let m = node.metrics();
                let mut line = branch.line.clone();
                line.push(format!(
                    "T{} continue {opening:?}",
                    branch.node.metrics().turn
                ));
                let mut milestones = branch.milestones.clone();
                milestones.push(m.clone());
                let mut actions = branch.actions.clone();
                actions.push(MacroAction::Continue);
                next.push(Branch {
                    score: proxy(&m, state.turn + (depth * 10) as u32)
                        + milestone_bonus(&node, position),
                    node,
                    line,
                    actions,
                    milestones,
                });
            }
            let mut choices = branch.node.choices();
            choices.sort_by_key(|(_, value)| std::cmp::Reverse(*value));
            // Progressive widening: all distinct current plans up to eight, then keep the
            // best beam survivors by the late proxy. This includes every reachable city
            // candidate on these four-seat maps.
            choices.truncate(8);
            if choices.is_empty() {
                next.push(branch);
                continue;
            }
            for (plan, _) in choices {
                let mut node = branch.node.fork();
                node.advance(&plan, 10);
                expanded += 1;
                let m = node.metrics();
                let mut line = branch.line.clone();
                line.push(format!("T{} {plan:?}", branch.node.metrics().turn));
                let mut milestones = branch.milestones.clone();
                milestones.push(m.clone());
                let mut actions = branch.actions.clone();
                actions.push(MacroAction::Plan(plan));
                next.push(Branch {
                    score: proxy(&m, state.turn + (depth * 10) as u32)
                        + milestone_bonus(&node, position),
                    node,
                    line,
                    actions,
                    milestones,
                });
            }
        }
        next.sort_by_key(|b| std::cmp::Reverse(b.score));
        next.truncate(width);
        if next.is_empty() {
            break;
        }
        let best = &next[0];
        best_by_depth.push(serde_json::json!({
            "step": step + 1,
            "expanded": expanded,
            "score": best.score,
            "metrics": best.milestones.last(),
            "line": best.line,
        }));
        beam = next;
        if beam.iter().all(|b| b.node.metrics().finished) {
            break;
        }
    }
    let validation = beam.first().map(|best| {
        let brains: Vec<_> = state
            .players
            .iter()
            .map(|p| warmed_brain(recording, state, p.id))
            .collect();
        let mut actual = OracleNode::from_recorded_state(state, &brains, seat);
        actual.set_search_rivals(search_rivals);
        let mut milestones = vec![actual.metrics()];
        for action in &best.actions {
            match action {
                MacroAction::Opening(opening) => actual.advance_opening(*opening, 10),
                MacroAction::Continue => actual.continue_opening(10),
                MacroAction::Plan(plan) => actual.advance(plan, 10),
            }
            milestones.push(actual.metrics());
        }
        serde_json::json!({"milestones": milestones, "checkpoints": actual.checkpoints()})
    });
    serde_json::json!({"expanded": expanded, "beam": width, "sample": sample,
        "search_rivals": search_rivals,
        "depth": depth, "best_by_depth": best_by_depth,
        "recorded_map_validation": validation,
        "best": beam.first().map(|b| serde_json::json!({"line": b.line,
            "milestones": b.milestones, "checkpoints": b.node.checkpoints(),
            "score": b.score}))})
}

fn components(
    state: &GameState,
    seat: PlayerId,
    base: &ScriptedBrain,
    samples: u64,
) -> serde_json::Value {
    let view = player_view(state, seat);
    let mut first = OracleNode::new(&view, base, 0);
    let options = first.choices();
    let mut rows = Vec::new();
    for horizon in [20, 40, 80, 150] {
        for (plan, analytic) in &options {
            let outcomes: Vec<_> = (0..samples)
                .map(|sample| {
                    let mut node = OracleNode::new(&view, base, sample);
                    node.advance(plan, horizon);
                    node.metrics()
                })
                .collect();
            rows.push(serde_json::json!({"horizon": horizon, "plan": plan,
                "analytic": analytic, "outcomes": outcomes}));
        }
    }
    serde_json::json!({"samples": samples, "rows": rows})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let name = get_arg(&args, "--position").unwrap_or_else(|| "85-oath".into());
    let mode = get_arg(&args, "--mode").unwrap_or_else(|| "sweep".into());
    let samples: u8 = get_arg(&args, "--samples")
        .unwrap_or_else(|| "16".into())
        .parse()?;
    let depth: usize = get_arg(&args, "--depth")
        .unwrap_or_else(|| "15".into())
        .parse()?;
    let width: usize = get_arg(&args, "--width")
        .unwrap_or_else(|| "4".into())
        .parse()?;
    let sample: u64 = get_arg(&args, "--sample")
        .unwrap_or_else(|| "0".into())
        .parse()?;
    let search_rivals = args.iter().any(|a| a == "--search-rivals");
    let horizons: Vec<u32> = get_arg(&args, "--horizons")
        .unwrap_or_else(|| "20,40,80,150,0".into())
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    let out = PathBuf::from(get_arg(&args, "--out-dir").unwrap_or_else(|| "/tmp".into()));
    std::fs::create_dir_all(&out)?;
    let (p85, p84) = load_recordings(get_arg(&args, "--ai-params").as_deref())?;
    let (recording, turn, seat) = match name.as_str() {
        "85-oath" => (&p85, 14, PlayerId(2)),
        "85-glass" => (&p85, 1, PlayerId(0)),
        "84-oath" => (&p84, 1, PlayerId(0)),
        _ => return Err(format!("unknown position {name}").into()),
    };
    let (state, mismatches) = position(recording, turn)?;
    eprintln!(
        "{name}: T{turn} seat {} tape hash mismatches {mismatches}",
        seat.0
    );
    let base = warmed_brain(recording, &state, seat);
    let result = match mode.as_str() {
        "model" => {
            let view = player_view(&state, seat);
            let options = OracleNode::new(&view, &base, 0).choices();
            serde_json::json!({"position": name, "turn": turn, "seat": seat.0,
                "hash_mismatches": mismatches,
                "options": options.iter().map(|(plan, analytic)| {
                    let f = learned_macro::features(&view, plan, None);
                    serde_json::json!({"plan": plan, "analytic": analytic,
                        "model_value": learned_macro::value(&f, plan, None), "features": f})
                }).collect::<Vec<_>>()})
        }
        "sweep" => serde_json::json!({"position": name, "turn": turn, "seat": seat.0,
            "hash_mismatches": mismatches, "samples": samples,
            "sweep": sweep(&state, seat, &base, samples, &horizons)}),
        "tree" => serde_json::json!({"position": name, "turn": turn, "seat": seat.0,
            "hash_mismatches": mismatches, "tree": tree(recording, &state, seat, &base,
                TreeConfig { position: &name, depth, width, sample, search_rivals })}),
        "components" => serde_json::json!({"position": name, "turn": turn, "seat": seat.0,
            "hash_mismatches": mismatches,
            "components": components(&state, seat, &base, samples as u64)}),
        _ => return Err(format!("unknown mode {mode}").into()),
    };
    let file = out.join(format!("deep-search-{name}-{mode}.json"));
    std::fs::write(&file, serde_json::to_vec_pretty(&result)?)?;
    println!("{}", file.display());
    Ok(())
}
