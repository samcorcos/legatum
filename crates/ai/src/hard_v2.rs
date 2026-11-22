//! Hard v2: each Hard AI seat plays one opening tuned by the opening league. Opt-in only
//! (`RuleExperiments::hard_v2_ai`); the app's Hard is the scripted brain, because validation
//! found Hard v2 easier for a strong player (docs/sim-results/2026-09-30-hard-v2-validation.md).
//!
//! At game start a seat draws uniformly from Default, Rush, GreedyExpand, Snowball
//! and Economic (`tuned_openings.rs`, exported from duel coevolution). The draw is
//! seeded by the seat's brain seed, so the same game always gives a seat the same opening,
//! including after a save and load. The league is not faction-conditioned, so the draw is
//! global. A drawn opening plays its tuned genome with the behaviour primitives on and uses
//! the opening's phase machine (Expand, Conquer, TurtleTech, and Defend when enemies close
//! in). Drawing `Default` plays the pre-v2 scripted Hard. Easy and Normal never come here.
use crate::openings::{Opening, OpeningBrain, OpeningPolicy};
use crate::tuned_openings::TUNED;
use crate::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, Faction, PlayerId};

#[cfg(test)]
mod handoff_tests;

/// Separates the opening draw from every other stream derived from a brain seed.
const DRAW: u64 = 0x4841_5244_5632; // "HARDV2"

/// The opening a Hard seat with this brain seed plays.
pub fn draw_opening(brain_seed: u64) -> Opening {
    let total: u32 = TUNED.iter().map(|t| t.weight).sum();
    let mut r = Rng::derive(brain_seed, &[purpose::AI, DRAW]).range(0, i64::from(total) - 1) as u32;
    for t in TUNED {
        if r < t.weight {
            return t.opening;
        }
        r -= t.weight;
    }
    Opening::Default
}

/// The tuned policy for an opening (the built-in one when the league kept it untuned).
pub fn policy(opening: Opening) -> OpeningPolicy {
    TUNED
        .iter()
        .find(|t| t.opening == opening)
        .and_then(|t| t.genome)
        .map(|g| OpeningPolicy::from_genome(opening, g).expect("tuned genome fits the schema"))
        .unwrap_or_else(|| OpeningPolicy::builtin(opening))
}

/// A Hard v2 brain for one seat.
pub fn brain(faction: Faction, seed: u64, player: PlayerId) -> Box<dyn Brain> {
    make_brain(faction, seed, player, false)
}

/// The same draw and opening, then a permanent hand-off to scripted Hard. Simulator opt-in;
/// neither the app factory nor the existing full-game Hard v2 behavior changes.
pub fn handoff_brain(faction: Faction, seed: u64, player: PlayerId) -> Box<dyn Brain> {
    make_brain(faction, seed, player, true)
}

fn make_brain(faction: Faction, seed: u64, player: PlayerId, handoff: bool) -> Box<dyn Brain> {
    let base = ScriptedBrain::with_difficulty(faction, seed, player, AiDifficulty::Hard);
    match draw_opening(seed) {
        Opening::Default => Box::new(base),
        opening => {
            let mut brain = OpeningBrain::new(base, policy(opening));
            brain.primitives_enabled = true;
            if handoff {
                brain = brain.with_handoff();
            }
            Box::new(brain)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuned_openings::{GENE_NAMES, OPENING_SCHEMA};

    #[test]
    fn tuned_table_fits_the_code_and_draws_uniformly() {
        assert_eq!(OPENING_SCHEMA, crate::openings::OPENING_SCHEMA);
        let names: Vec<_> = OpeningPolicy::genes().into_iter().map(|g| g.name).collect();
        assert_eq!(
            names, GENE_NAMES,
            "regenerate with tools/league/export_tuned.py"
        );
        assert_eq!(TUNED.iter().map(|t| t.weight).sum::<u32>(), 5);
        for o in Opening::ALL {
            let t = TUNED.iter().filter(|t| t.opening == o).collect::<Vec<_>>();
            assert_eq!(t.len(), 1, "{o:?} listed once");
            assert_eq!(t[0].weight, 1);
            if o == Opening::Default {
                assert!(t[0].genome.is_none());
            } else if let Some(g) = t[0].genome {
                OpeningPolicy::from_genome(o, g).unwrap();
            }
        }
    }

    #[test]
    fn draw_is_deterministic_and_follows_the_weights() {
        let mut counts = std::collections::BTreeMap::<Opening, u32>::new();
        for seed in 0..20_000u64 {
            let s = Rng::derive(seed, &[7]).next_u64();
            assert_eq!(draw_opening(s), draw_opening(s));
            *counts.entry(draw_opening(s)).or_default() += 1;
        }
        for t in TUNED {
            let got = counts.get(&t.opening).copied().unwrap_or(0);
            if t.weight == 0 {
                assert_eq!(got, 0, "{:?} must never be drawn", t.opening);
                continue;
            }
            let total: u32 = TUNED.iter().map(|t| t.weight).sum();
            let want = t.weight * 20_000 / total;
            assert!(
                got.abs_diff(want) <= 600,
                "{:?}: drew {got} of 20000, expected about {want}",
                t.opening
            );
        }
    }

    #[test]
    fn hard_v2_seats_play_their_opening_with_primitives() {
        let setup = legatum_core::GameSetup::all_ai(5);
        let state = legatum_core::new_game(&setup);
        for p in &state.players {
            let seed = Rng::derive(setup.seed, &[purpose::AI, u64::from(p.id.0)]).next_u64();
            let brain = brain(p.faction, seed, p.id);
            let opening = draw_opening(seed);
            if opening == Opening::Default {
                assert_eq!(brain.opening(), None);
            } else {
                assert_eq!(brain.opening(), Some(opening));
            }
        }
    }
}
