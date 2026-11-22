//! Fit and calibrate the AI's chance-to-win estimate (`legatum_ai::winchance`).
//!
//!   winchance_fit <train-tapes> [--holdout <name>=<tapes-dir>]... [--every N]
//!
//! Replays completed games, reads every alive seat's own PlayerView every N turns, and fits the
//! conditional-logit weights on the training tapes by Newton's method (float, offline). It
//! then reports calibration for the fitted float weights, the rounded integer weights the AI
//! actually uses (`WEIGHTS`) and the candidate integer weights, on the training set and on every
//! held-out set: log loss and Brier score against the uniform baseline, reliability bins, and
//! breakdowns by game phase and faction. Games without a winner are skipped.
use legatum_ai::winchance::{FEATURES, WEIGHTS, WEIGHTS_DUEL, WinFacts};
use legatum_core::{TeamId, player_view, resolve_turn};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;

const K: usize = 9;

struct Row {
    phase: i32,
    faction: String,
    me: TeamId,
    winner: TeamId,
    facts: WinFacts,
    /// Feature vectors (as floats, shares in 0..1) per team, in team order.
    x: Vec<(TeamId, [f64; K])>,
}

fn load(dir: &str, every: u32) -> Vec<Row> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .expect("tapes dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
        .par_iter()
        .flat_map(|file| {
            let tape =
                Tape::from_json(&std::fs::read_to_string(file).expect("tape")).expect("tape json");
            let mut state = tape.initial_state();
            let mut samples = Vec::new();
            for t in &tape.turns {
                if state.turn.is_multiple_of(every) {
                    for p in state.players.iter().filter(|p| p.alive) {
                        let view = player_view(&state, p.id);
                        let facts = WinFacts::of(&view);
                        samples.push((format!("{:?}", p.faction), facts));
                    }
                }
                let (next, _) = resolve_turn(&state, &t.orders);
                state = next;
            }
            let Some(winner) = state.winner else {
                return Vec::new();
            };
            samples
                .into_iter()
                .map(|(faction, facts)| {
                    let x = facts
                        .features()
                        .into_iter()
                        .map(|(t, f)| (t, f.map(|v| v as f64 / 1000.0)))
                        .collect();
                    Row {
                        phase: facts.phase,
                        faction,
                        me: facts.me,
                        winner,
                        facts,
                        x,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn probs(row: &Row, w: &[f64; K]) -> Vec<f64> {
    let s: Vec<f64> = row
        .x
        .iter()
        .map(|(_, x)| x.iter().zip(w).map(|(a, b)| a * b).sum())
        .collect();
    let top = s.iter().cloned().fold(f64::MIN, f64::max);
    let e: Vec<f64> = s.iter().map(|v| (v - top).exp()).collect();
    let z: f64 = e.iter().sum();
    e.iter().map(|v| v / z).collect()
}

fn fit(rows: &[Row]) -> [f64; K] {
    let mut w = [0.0; K];
    let lambda = 1e-3 * rows.len() as f64;
    for _ in 0..30 {
        let (g, h) = rows
            .par_iter()
            .map(|row| {
                let p = probs(row, &w);
                let mut mean = [0.0; K];
                for (pi, (_, x)) in p.iter().zip(&row.x) {
                    for k in 0..K {
                        mean[k] += pi * x[k];
                    }
                }
                let win = row.x.iter().position(|(t, _)| *t == row.winner).unwrap();
                let mut g = [0.0; K];
                let mut h = [[0.0; K]; K];
                for k in 0..K {
                    g[k] = row.x[win].1[k] - mean[k];
                }
                for (pi, (_, x)) in p.iter().zip(&row.x) {
                    for a in 0..K {
                        for b in 0..K {
                            h[a][b] += pi * (x[a] - mean[a]) * (x[b] - mean[b]);
                        }
                    }
                }
                (g, h)
            })
            .reduce(
                || ([0.0; K], [[0.0; K]; K]),
                |(mut g1, mut h1), (g2, h2)| {
                    for a in 0..K {
                        g1[a] += g2[a];
                        for b in 0..K {
                            h1[a][b] += h2[a][b];
                        }
                    }
                    (g1, h1)
                },
            );
        // Newton step on the L2-penalized log likelihood: (H + λI) Δ = g − λw.
        let mut m = [[0.0; K + 1]; K];
        for a in 0..K {
            for b in 0..K {
                m[a][b] = h[a][b] + if a == b { lambda } else { 0.0 };
            }
            m[a][K] = g[a] - lambda * w[a];
        }
        let delta = solve(m);
        let mut step = 0.0;
        for k in 0..K {
            w[k] += delta[k];
            step += delta[k].abs();
        }
        if step < 1e-7 {
            break;
        }
    }
    w
}

fn solve(mut m: [[f64; K + 1]; K]) -> [f64; K] {
    for c in 0..K {
        let p = (c..K)
            .max_by(|a, b| m[*a][c].abs().total_cmp(&m[*b][c].abs()))
            .unwrap();
        m.swap(c, p);
        let d = m[c][c];
        if d.abs() < 1e-12 {
            continue;
        }
        for x in &mut m[c][c..=K] {
            *x /= d;
        }
        let pivot = m[c];
        for (r, row) in m.iter_mut().enumerate() {
            if r != c {
                let f = row[c];
                for (x, p) in row[c..=K].iter_mut().zip(&pivot[c..=K]) {
                    *x -= f * p;
                }
            }
        }
    }
    std::array::from_fn(|k| m[k][K])
}

/// (predicted P(me wins), won?) for each row.
fn predictions(rows: &[Row], pred: &dyn Fn(&Row) -> f64) -> Vec<(f64, bool)> {
    rows.iter().map(|r| (pred(r), r.me == r.winner)).collect()
}

fn summary(label: &str, rows: &[Row], pred: &dyn Fn(&Row) -> f64) -> String {
    let ps = predictions(rows, pred);
    let n = ps.len().max(1) as f64;
    let brier: f64 = ps
        .iter()
        .map(|(p, y)| (p - f64::from(*y as u8)).powi(2))
        .sum::<f64>()
        / n;
    let base: f64 = rows
        .iter()
        .map(|r| {
            let p = 1.0 / r.x.len() as f64;
            (p - f64::from((r.me == r.winner) as u8)).powi(2)
        })
        .sum::<f64>()
        / n;
    let logloss: f64 = ps
        .iter()
        .map(|(p, y)| -(if *y { p.max(1e-6) } else { (1.0 - p).max(1e-6) }).ln())
        .sum::<f64>()
        / n;
    let mut out = format!(
        "{label}: {} samples, Brier {:.4} (uniform baseline {:.4}, skill {:+.1}%), log loss {:.4}\n",
        ps.len(),
        brier,
        base,
        100.0 * (1.0 - brier / base),
        logloss
    );
    out += "  reliability (predicted bin: samples, mean predicted, actual win rate):\n";
    for (lo, hi) in [
        (0.0, 0.05),
        (0.05, 0.1),
        (0.1, 0.2),
        (0.2, 0.35),
        (0.35, 0.5),
        (0.5, 0.7),
        (0.7, 0.9),
        (0.9, 1.01),
    ] {
        let bin: Vec<_> = ps.iter().filter(|(p, _)| *p >= lo && *p < hi).collect();
        if bin.is_empty() {
            continue;
        }
        let m = bin.iter().map(|(p, _)| p).sum::<f64>() / bin.len() as f64;
        let a = bin.iter().filter(|(_, y)| *y).count() as f64 / bin.len() as f64;
        out += &format!(
            "    {:>3.0}-{:>3.0}%: {:>6}  {:>5.1}%  {:>5.1}%\n",
            lo * 100.0,
            hi.min(1.0) * 100.0,
            bin.len(),
            m * 100.0,
            a * 100.0
        );
    }
    out += "  by phase (turn / expected length: samples, Brier, uniform):\n";
    for (lo, hi, name) in [
        (0, 250, "0-25%"),
        (250, 500, "25-50%"),
        (500, 750, "50-75%"),
        (750, 1000, "75-100%"),
        (1000, 9999, ">100%"),
    ] {
        let idx: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.phase >= lo && r.phase < hi)
            .map(|(i, _)| i)
            .collect();
        if idx.is_empty() {
            continue;
        }
        let b = idx
            .iter()
            .map(|i| (ps[*i].0 - f64::from(ps[*i].1 as u8)).powi(2))
            .sum::<f64>()
            / idx.len() as f64;
        let u = idx
            .iter()
            .map(|i| {
                let p = 1.0 / rows[*i].x.len() as f64;
                (p - f64::from(ps[*i].1 as u8)).powi(2)
            })
            .sum::<f64>()
            / idx.len() as f64;
        out += &format!("    {name:>8}: {:>6}  {b:.4}  {u:.4}\n", idx.len());
    }
    out += "  by faction (samples, mean predicted, actual win share of samples, Brier):\n";
    let mut factions: Vec<_> = rows.iter().map(|r| r.faction.clone()).collect();
    factions.sort();
    factions.dedup();
    for f in factions {
        let idx: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.faction == f)
            .map(|(i, _)| i)
            .collect();
        let m = idx.iter().map(|i| ps[*i].0).sum::<f64>() / idx.len() as f64;
        let a = idx.iter().filter(|i| ps[**i].1).count() as f64 / idx.len() as f64;
        let b = idx
            .iter()
            .map(|i| (ps[*i].0 - f64::from(ps[*i].1 as u8)).powi(2))
            .sum::<f64>()
            / idx.len() as f64;
        out += &format!(
            "    {f:>13}: {:>6}  {:>5.1}%  {:>5.1}%  {b:.4}\n",
            idx.len(),
            m * 100.0,
            a * 100.0
        );
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let train_dir = args.get(1).expect("train tapes dir");
    let mut holdouts = Vec::new();
    let mut every = 3;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--holdout" => {
                let (name, dir) = args[i + 1].split_once('=').expect("name=dir");
                holdouts.push((name.to_string(), dir.to_string()));
                i += 2;
            }
            "--every" => {
                every = args[i + 1].parse().expect("--every N");
                i += 2;
            }
            other => panic!("unknown argument {other}"),
        }
    }
    let train = load(train_dir, every);
    eprintln!("loaded {} training samples", train.len());
    let w = fit(&train);
    let candidate: [i32; K] = std::array::from_fn(|k| (w[k] * 1000.0).round() as i32);
    println!(
        "# Chance-to-win calibration (winchance v{})\n",
        legatum_ai::winchance::VERSION
    );
    println!("Training tapes: {train_dir} (every {every} turns, every alive seat's own view)\n");
    println!("Fitted weights (logits per full share):");
    for k in 0..K {
        println!("  {:>18}: {:+.3}", FEATURES[k], w[k]);
    }
    println!("Candidate integer WEIGHTS: {candidate:?}");
    println!("Current integer WEIGHTS:   {WEIGHTS:?}");
    println!("Current WEIGHTS_DUEL:      {WEIGHTS_DUEL:?}\n");
    let float = |r: &Row| {
        let p = probs(r, &w);
        p[r.x.iter().position(|(t, _)| *t == r.me).unwrap()]
    };
    let current = |r: &Row| r.facts.estimate_with(&WEIGHTS).chance as f64 / 10_000.0;
    let cand = |r: &Row| r.facts.estimate_with(&candidate).chance as f64 / 10_000.0;
    let duel = |r: &Row| r.facts.estimate_with(&WEIGHTS_DUEL).chance as f64 / 10_000.0;
    println!("```");
    print!("{}", summary("train / fitted float", &train, &float));
    print!("{}", summary("train / candidate integer", &train, &cand));
    print!("{}", summary("train / current integer", &train, &current));
    for (name, dir) in &holdouts {
        let rows = load(dir, every);
        print!(
            "{}",
            summary(&format!("{name} / candidate integer"), &rows, &cand)
        );
        print!(
            "{}",
            summary(&format!("{name} / current integer"), &rows, &current)
        );
        print!(
            "{}",
            summary(&format!("{name} / current WEIGHTS_DUEL"), &rows, &duel)
        );
    }
    println!("```");
}
