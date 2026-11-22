//! The learned value network (AI war plan step 3): a small integer MLP that scores each team
//! in `winchance`'s conditional logit in place of the fitted linear score.
//!
//! Gene `learned_value` (default 0) switches it on for the plan's estimate
//! (`WinFacts::with_relative`). Everything else about the estimate stays rule-based: the VP
//! hold overlay (a completing hold wins outright) and the VP race term.
//! Learning only replaces the uncertain part, the value of a position's public shares.
//!
//! Inputs come from `WinFacts` alone, so every projection the planner already makes ("one more
//! city, a shorter runway") is scored the same way. They are permille integers, read only from
//! this seat's `PlayerView` through `WinFacts::of`.
//!
//! Training is offline (`tools/sim/train-value.py`, PyTorch) on Hard self-play games from
//! `crates/sim/examples/value_data.rs`. The trainer quantizes the weights into
//! `crates/ai/data/value_net.json`, compiled in with `include_str!`. Inference is integer only
//! (i64 accumulators, truncating division by the weight scale), so native and WASM agree exactly.
use crate::winchance::WinFacts;
use legatum_core::TeamId;
use serde::Deserialize;

/// Bumped whenever an input changes meaning; the weights file must carry the same number.
pub const VERSION: u32 = 2;
/// Inputs per team.
pub const INPUTS: usize = 19;
/// The most teams a game has (eight seats).
pub const MAX_TEAMS: usize = 8;

pub const INPUT_NAMES: [&str; INPUTS] = [
    "VP progress",
    "VP progress late",
    "cities share",
    "population share",
    "economy share",
    "army share",
    "science share",
    "territory share",
    "runway short",
    "is me",
    "phase",
    "one over teams",
    "holding",
    "hold done",
    "hold margin",
    "progress gap",
    "cities gap",
    "economy gap",
    "army gap",
];

/// Each team's inputs, in `WinFacts::teams` order. `features` is `facts.features()`.
pub fn inputs_with(
    facts: &WinFacts,
    features: &[(TeamId, [i64; 9])],
) -> Vec<(TeamId, [i32; INPUTS])> {
    let n = features.len().max(1) as i64;
    // The best other team's value of linear feature `k`, for team `t`.
    let best_other = |t: TeamId, k: usize| -> i64 {
        features
            .iter()
            .filter(|(u, _)| *u != t)
            .map(|(_, x)| x[k])
            .max()
            .unwrap_or(x_default(k, n))
    };
    let hold_turns = facts.hold_turns.max(1) as i64;
    features
        .iter()
        .map(|(t, x)| {
            let team = facts.team(*t);
            let hold_left = team.and_then(|f| f.hold_left);
            let margin = facts.hold_margin(*t).unwrap_or(-1000).clamp(-1000, 300);
            let mut v = [0i64; INPUTS];
            v[..9].copy_from_slice(x);
            v[9] = if *t == facts.me { 1000 } else { 0 };
            v[10] = (facts.phase as i64).clamp(0, 1500);
            v[11] = 1000 / n;
            v[12] = if hold_left.is_some() { 1000 } else { 0 };
            v[13] = hold_left.map_or(0, |left| {
                (hold_turns - left as i64).clamp(0, hold_turns) * 1000 / hold_turns
            });
            v[14] = margin;
            v[15] = x[0] - best_other(*t, 0);
            v[16] = x[2] - best_other(*t, 2);
            v[17] = x[4] - best_other(*t, 4);
            v[18] = x[5] - best_other(*t, 5);
            (*t, v.map(|e| e.clamp(-30_000, 30_000) as i32))
        })
        .collect()
}

/// A lone team's "best other" is an even share (and no VP).
fn x_default(k: usize, n: i64) -> i64 {
    if k < 2 { 0 } else { 1000 / n }
}

pub fn inputs(facts: &WinFacts) -> Vec<(TeamId, [i32; INPUTS])> {
    inputs_with(facts, &facts.features())
}

/// The rule-based hold overlay `WinFacts::estimate` applies: each team's share (permille) of
/// the chance that some hold completes, and the permille left for the logit to share.
pub fn hold_overlay(facts: &WinFacts) -> (Vec<(TeamId, i64)>, i64) {
    let holds = facts.hold_chances();
    let sum: i64 = holds.iter().map(|(_, q)| *q).sum();
    let scale = sum.max(1000);
    let shares = facts
        .teams
        .iter()
        .map(|t| {
            let q = holds
                .iter()
                .find(|(h, _)| *h == t.team)
                .map_or(0, |(_, q)| *q);
            (t.team, q * 1000 / scale)
        })
        .collect();
    (shares, 1000 - sum.min(1000))
}

#[derive(Clone, Debug, Deserialize)]
struct Layer {
    #[serde(rename = "in")]
    inputs: usize,
    #[serde(rename = "out")]
    outputs: usize,
    /// `outputs` rows of `inputs` weights, times `weight_scale`.
    w: Vec<Vec<i32>>,
    /// Biases in milli-units (1000 = 1.0).
    b: Vec<i32>,
    relu: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Net {
    version: u32,
    weight_scale: i64,
    layers: Vec<Layer>,
    /// The hand formula's nine `WEIGHTS` refitted on the same games (milli-logits per full
    /// share), for gene value `learned_value = 2`: the recalibration alone, without the network.
    #[serde(default)]
    refit_weights: Option<[i32; 9]>,
}

/// Activations are milli-units; clamping keeps every product far inside i64.
const ACTIVATION_LIMIT: i64 = 1_000_000;

impl Net {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let net: Net = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if net.version != VERSION {
            return Err(format!(
                "value net inputs v{} but the AI reads v{VERSION}",
                net.version
            ));
        }
        if net.weight_scale <= 0 || net.layers.is_empty() {
            return Err("value net has no layers".into());
        }
        let mut width = INPUTS;
        for (i, l) in net.layers.iter().enumerate() {
            if l.inputs != width
                || l.w.len() != l.outputs
                || l.b.len() != l.outputs
                || l.w.iter().any(|row| row.len() != l.inputs)
            {
                return Err(format!("value net layer {i} has the wrong shape"));
            }
            width = l.outputs;
        }
        if width != 1 {
            return Err("value net must end in one output".into());
        }
        Ok(net)
    }

    /// One team's score in milli-logits.
    pub fn score(&self, x: &[i32; INPUTS]) -> i64 {
        let mut a: Vec<i64> = x.iter().map(|v| *v as i64).collect();
        for l in &self.layers {
            a =
                l.w.iter()
                    .zip(&l.b)
                    .map(|(row, b)| {
                        let dot: i64 = row.iter().zip(&a).map(|(w, x)| *w as i64 * x).sum();
                        let z = dot / self.weight_scale + *b as i64;
                        let z = if l.relu { z.max(0) } else { z };
                        z.clamp(-ACTIVATION_LIMIT, ACTIVATION_LIMIT)
                    })
                    .collect();
        }
        a[0]
    }
}

/// The trained network compiled into the AI, or None when the file is missing or stale (the
/// estimate then falls back to the hand formula).
pub fn net() -> Option<&'static Net> {
    static NET: std::sync::OnceLock<Option<Net>> = std::sync::OnceLock::new();
    NET.get_or_init(|| Net::from_json(include_str!("../data/value_net.json")).ok())
        .as_ref()
}

/// The refitted linear weights compiled in with the network, if the file carries them.
pub fn refit_weights() -> Option<&'static [i32; 9]> {
    net()?.refit_weights.as_ref()
}

/// Every team's learned score (milli-logits) in `features` order, or None without a network.
pub fn scores(facts: &WinFacts, features: &[(TeamId, [i64; 9])]) -> Option<Vec<i64>> {
    let net = net()?;
    Some(
        inputs_with(facts, features)
            .iter()
            .map(|(_, x)| net.score(x))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Golden {
        x: Vec<i32>,
        score: i64,
    }

    #[derive(Deserialize)]
    struct File {
        golden: Vec<Golden>,
    }

    #[test]
    fn compiled_network_loads_and_matches_the_trainers_integer_scores() {
        let json = include_str!("../data/value_net.json");
        let net = Net::from_json(json).expect("value_net.json loads");
        let file: File = serde_json::from_str(json).expect("golden vectors");
        assert!(!file.golden.is_empty());
        for g in &file.golden {
            let x: [i32; INPUTS] = g.x.clone().try_into().expect("input width");
            assert_eq!(net.score(&x), g.score, "{:?}", g.x);
        }
    }

    #[test]
    fn learned_estimate_is_a_probability_and_rewards_a_lead() {
        let view = crate::fixture::ViewBuilder::new(24, 16)
            .city(1, crate::fixture::ME, crate::fixture::off(3, 5), 4)
            .city(2, crate::fixture::ENEMY, crate::fixture::off(12, 5), 2)
            .build();
        let decisions = crate::DecisionParams {
            learned_value: 1,
            ..Default::default()
        };
        let facts = WinFacts::of(&view).with_relative(&decisions);
        let c = facts.chance();
        assert!((0..=10_000).contains(&c));
        let mut ahead = facts.clone();
        ahead.team_mut(view.team).unwrap().cities += 3;
        assert!(ahead.chance() >= c, "{} {c}", ahead.chance());
        let refit = WinFacts::of(&view).with_relative(&crate::DecisionParams {
            learned_value: 2,
            ..Default::default()
        });
        assert!(
            refit_weights().is_some(),
            "value_net.json carries refit_weights"
        );
        assert!((0..=10_000).contains(&refit.chance()));
    }

    #[test]
    fn inputs_are_bounded_and_mark_this_seat() {
        let view = crate::fixture::ViewBuilder::new(24, 16)
            .city(1, crate::fixture::ME, crate::fixture::off(3, 5), 4)
            .city(2, crate::fixture::ENEMY, crate::fixture::off(12, 5), 2)
            .build();
        let facts = WinFacts::of(&view);
        let x = inputs(&facts);
        assert_eq!(x.len(), facts.teams.len());
        assert_eq!(x.iter().filter(|(_, v)| v[9] == 1000).count(), 1);
        assert!(x.iter().all(|(_, v)| v.iter().all(|e| e.abs() <= 30_000)));
    }
}
