//! Search over the engine, step 1: rollouts for the big decisions (Hard+Search, opt-in).
//!
//! At each plan review point (`plan.rs::choose_plan`) the plan layer still enumerates and prices
//! its candidates analytically. Search takes the top K of them (plus the plan already running),
//! builds D game states from this seat's own `PlayerView` (`determinize`, never the hidden
//! state), and plays each candidate H turns forward on the real engine: every seat runs the
//! scripted Hard brain, this seat with the candidate's plan forced. Each rollout is scored by
//! the calibrated win-chance estimate at the horizon (`winchance.rs`), averaged over the D
//! samples, and the best candidate becomes the plan. The D samples are shared by every
//! candidate (common random numbers), and everything is seeded, so a game replays exactly.
//!
//! An omniscient mode starts the rollouts from the true state instead. It reads hidden state
//! and exists only as a flagged diagnostic upper bound: no shipped seat may use it.

mod determinize;
pub mod tactics;

pub use determinize::{Prior, prior_map};
pub use determinize::{calibrate_economy, chance_in, determinize};

use crate::Brain;
use crate::ScriptedBrain;
use crate::openings::{Opening, OpeningBrain};
use crate::plan::PlanKind;
use legatum_core::rng::Rng;
use legatum_core::{AiDifficulty, GameState, PlayerId, PlayerView};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// RNG stream for rollout seeds.
const STREAM: u64 = 0x5EA4_C402;
/// Rollouts "to the end" stop here at the latest (Standard turns past the start).
const END_CAP: u32 = 400;

/// How hard a Hard+Search seat looks ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Candidates rolled out (K): the top analytic ones, plus the running plan.
    pub candidates: u8,
    /// Turns each plan rollout plays (H), in Standard turns; 0 = to the
    /// end of the game.
    pub horizon: u32,
    /// Determinization samples per candidate (D).
    pub samples: u8,
    /// Diagnostic upper bound only: roll out from the true state (reads hidden state).
    pub omniscient: bool,
    /// The turn-1 opening is searched too (Default, Rush, GreedyExpand, Snowball, Economic).
    #[serde(default)]
    pub openings: bool,
    /// Opening rollout length in Standard turns; 0 = to the end of the game.
    #[serde(default)]
    pub opening_horizon: u32,
    /// Rival policies sampled in rollouts, cycled by sample: 1 = scripted Hard only; up to 4
    /// adds a Rush, a GreedyExpand and a Turtle-biased opponent.
    #[serde(default = "one")]
    pub opponents: u8,
    /// Information triggers reopen the plan between reviews: a rival met, enemy military
    /// closing on our cities, a rival losing cities, a new war.
    #[serde(default)]
    pub triggers: bool,
    /// Determinized rival cities get economy buildings up to their public economy title.
    #[serde(default)]
    pub economy: bool,
    /// After each search (and at review points with a single plan), roll out the knob
    /// postures (Military, Expand, Economy, or the personality) under the chosen plan.
    #[serde(default)]
    pub postures: bool,
    /// `w`: the plan layer prices its candidates with the win-chance weights fitted for this
    /// player count (`WEIGHTS_DUEL` in 1v1). With `k=0` this is plain Hard plus the weights.
    #[serde(default)]
    pub fitted_plan: bool,
    /// `g`: Hold plans (garrison the most threatened city) join the searched candidates.
    #[serde(default)]
    pub holds: bool,
    /// `f`: every enemy city a campaign could open on (soonest three) is searched.
    #[serde(default)]
    pub fronts: bool,
    /// `s`: tactical stance search on engaged turns (Aggressive / default / Cautious),
    /// scored by `material` after `stance_horizon` Standard turns (`sh=`, default 4).
    #[serde(default)]
    pub stances: bool,
    #[serde(default)]
    pub stance_horizon: u32,
    /// `x`: move-level tactical search on city attacks (`tactics.rs`), resolved this turn
    /// and scored by `material` after `tactic_horizon` Standard turns (`xh=`, default 3).
    #[serde(default)]
    pub tactics: bool,
    #[serde(default)]
    pub tactic_horizon: u32,
    /// `xc`: tactical rollouts scored by `material_captures` instead of `material`.
    #[serde(default)]
    pub tactic_captures: bool,
    /// `z`: the first plan's review is delayed by the seat number (modulo the review interval),
    /// so several search seats don't all search on the same turn (the app, pillar 7).
    #[serde(default)]
    pub stagger: bool,
    /// `r=`: the policy every seat plays inside rollouts (never in the real game).
    /// 0 = scripted Hard; 1 = Hard without prospective war planners or strike forecasts;
    /// 2 = level 1, and rivals re-plan only every other turn (orders persist between);
    /// 3 = rivals every third turn; 4 = rivals every third turn and this seat every other
    /// turn after the decision turn.
    #[serde(default)]
    pub rollout_policy: u8,
    /// `a=`: this many of the D samples (the last ones) model every rival as the human-playbook
    /// proxy (Proxy v6) instead of scripted Hard: an opponent that masses and attacks.
    #[serde(default)]
    pub proxy_rivals: u8,
    /// `c=`: search this many (plan, posture) candidates together instead of plans alone:
    /// every plan with the current posture first, then each plan with Military, Expand,
    /// Economy, Ranged and Melee in turn, until the count is reached.
    #[serde(default)]
    pub composites: u8,
    /// `y`: a win chance a third below the last search's reopens the plan decision.
    #[serde(default)]
    pub collapse: bool,
}

fn one() -> u8 {
    1
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            candidates: 3,
            horizon: 20,
            // 4 determinizations: +1.6 points over 2 in 1,024 paired 1v1 games (2026-10-06).
            samples: 4,
            omniscient: false,
            openings: false,
            opening_horizon: 0,
            opponents: 1,
            triggers: false,
            economy: false,
            postures: false,
            fitted_plan: false,
            holds: false,
            fronts: false,
            stances: false,
            stance_horizon: 4,
            tactics: false,
            tactic_horizon: 3,
            tactic_captures: false,
            stagger: false,
            rollout_policy: 0,
            proxy_rivals: 0,
            composites: 0,
            collapse: false,
        }
    }
}

impl SearchConfig {
    /// The Hard+Search difficulty in the app, whose engine runs on one thread (WASM): the plan
    /// layer priced by the win-chance weights fitted for this player count, and a budget that
    /// keeps a search turn near a second (pillar 7). The budget depends on the player count
    /// only, never on the platform, so native and WASM play the same game.
    ///   - 2 players: the confirmed 1v1 config, k3 h20 d4.
    ///   - 3-4 players: k3 h20 d2.
    ///   - 5-8 players: k3 h10 d2.
    ///
    /// Several search seats' first reviews are staggered, so they don't all search on one turn.
    /// Measured on one native thread (`examples/app_search_time.rs`, CPU ms of a turn's planning,
    /// max): 1v1 870, 4 players 810, 6 players 710, 8 players 690; the WASM build runs about
    /// 1.4x slower. Without `z`, 8 players at k3 h10 d2 reached 2,600 ms (6 seats searching on
    /// one turn), and at the 1v1 budget 8,800 ms.
    pub fn in_app(players: usize) -> Self {
        Self {
            fitted_plan: true,
            stagger: true,
            horizon: if players <= 4 { 20 } else { 10 },
            samples: if players <= 2 { 4 } else { 2 },
            ..Self::default()
        }
    }

    /// Parse `k=3,h=20,d=2,o,oh=0,v=3,t,e,p,w,g,omniscient` (`k=0`: no plan rollouts) (any subset; unknown keys are errors).
    /// `h=end` / `oh=end` roll to the end of the game.
    pub fn parse(spec: &str) -> Result<Self, String> {
        let mut c = Self::default();
        for part in spec.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            let num = || {
                if value == "end" {
                    Ok(0)
                } else {
                    value.parse::<u32>().map_err(|e| format!("{part}: {e}"))
                }
            };
            match key {
                "k" => c.candidates = num()?.min(16) as u8,
                "h" => c.horizon = num()?.min(END_CAP),
                "d" => c.samples = num()?.clamp(1, 16) as u8,
                "o" => c.openings = true,
                "oh" => c.opening_horizon = num()?.min(END_CAP),
                "v" => c.opponents = num()?.clamp(1, 4) as u8,
                "t" => c.triggers = true,
                "e" => c.economy = true,
                "p" => c.postures = true,
                "w" => c.fitted_plan = true,
                "g" => c.holds = true,
                "f" => c.fronts = true,
                "s" => c.stances = true,
                "sh" => c.stance_horizon = num()?.clamp(1, 40),
                "x" => c.tactics = true,
                "xh" => c.tactic_horizon = num()?.clamp(1, 40),
                "xc" => c.tactic_captures = true,
                "z" => c.stagger = true,
                "r" => c.rollout_policy = num()?.min(4) as u8,
                "a" => c.proxy_rivals = num()?.min(16) as u8,
                "c" => c.composites = num()?.min(64) as u8,
                "y" => c.collapse = true,
                "omniscient" => c.omniscient = true,
                _ => {
                    return Err(format!(
                        "unknown search option '{part}' (k, h, d, o, oh, v, t, e, p, w, g, f, s, sh, x, xh, xc, z, r, a, c, y, omniscient)"
                    ));
                }
            }
        }
        Ok(c)
    }

    pub fn label(&self) -> String {
        let h = |x: u32| {
            if x == 0 {
                "end".to_string()
            } else {
                x.to_string()
            }
        };
        let mut s = format!("k{}h{}d{}", self.candidates, h(self.horizon), self.samples);
        if self.openings {
            s += &format!("-o{}", h(self.opening_horizon));
        }
        if self.opponents > 1 {
            s += &format!("-v{}", self.opponents);
        }
        if self.triggers {
            s += "-t";
        }
        if self.economy {
            s += "-e";
        }
        if self.postures {
            s += "-p";
        }
        if self.fitted_plan {
            s += "-w";
        }
        if self.holds {
            s += "-g";
        }
        if self.fronts {
            s += "-f";
        }
        if self.stances {
            s += &format!("-s{}", self.stance_horizon);
        }
        if self.tactics {
            s += &format!("-x{}", self.tactic_horizon);
            if self.tactic_captures {
                s += "c";
            }
        }
        if self.stagger {
            s += "-z";
        }
        if self.rollout_policy > 0 {
            s += &format!("-r{}", self.rollout_policy);
        }
        if self.proxy_rivals > 0 {
            s += &format!("-a{}", self.proxy_rivals);
        }
        if self.composites > 0 {
            s += &format!("-c{}", self.composites);
        }
        if self.collapse {
            s += "-y";
        }
        if self.omniscient {
            s += "-omniscient";
        }
        s
    }

    /// Rollout length in turns (`to_end` caps at END_CAP).
    fn turns(horizon: u32) -> u32 {
        let h = if horizon == 0 { END_CAP } else { horizon };
        h.max(1)
    }
}

/// One search: what was rolled out and what it found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchEvent {
    pub turn: u32,
    /// Win chance now (0..=10,000), from the seat's own view.
    pub chance: i32,
    pub options: Vec<SearchOption>,
    /// Index into `options` of the plan chosen.
    pub chosen: usize,
    /// Index of the analytic favourite (what plain Hard would have picked).
    pub analytic: usize,
    pub omniscient: bool,
    /// Why the search ran: start, ended:<reason>, review, challenger, signal:<what>, opening.
    #[serde(default)]
    pub trigger: String,
    /// Rollout length in turns.
    #[serde(default)]
    pub horizon: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchOption {
    pub kind: PlanKind,
    /// For an opening search: the opening rolled out (`kind` is then Economy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<Opening>,
    /// For a posture search: the posture rolled out under the plan `kind`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posture: Option<String>,
    /// The plan layer's analytic value (hundredths of a point of win chance).
    pub value: i32,
    /// Mean win chance at the horizon over the samples.
    pub mean: i32,
    pub samples: Vec<i32>,
    /// Worth beyond the rollout horizon added to `mean` when choosing: a containment war's
    /// leader's race (gene `search_race`). Zero for every other option.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub beyond: i32,
}

fn is_zero(x: &i32) -> bool {
    *x == 0
}

/// What a planning pass needs to search: the configuration, the brain to copy into rollouts
/// and, for the diagnostic only, the true state.
#[derive(Clone, Copy)]
pub(crate) struct SearchHook<'a> {
    pub config: SearchConfig,
    pub brain: &'a ScriptedBrain,
    pub oracle: Option<&'a GameState>,
}

/// A plan a rollout seat keeps until it ends on its own or the horizon passes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ForcedPlan {
    pub kind: PlanKind,
    pub until: u32,
}

/// The D starting states (determinized, or the truth for the diagnostic) and their seeds.
fn starts(
    config: &SearchConfig,
    brain: &ScriptedBrain,
    oracle: Option<&GameState>,
    view: &PlayerView,
) -> Vec<(GameState, u64, u8)> {
    let d = config.samples.max(1) as u64;
    let indices: Vec<u64> = (0..d).collect();
    map_jobs(&indices, |i| {
        let seed = Rng::derive(
            brain.seed,
            &[STREAM, view.turn as u64, view.me.0 as u64, *i],
        )
        .next_u64();
        let state = match oracle {
            Some(truth) => {
                let mut s = truth.clone();
                s.setup.seed = seed;
                s
            }
            None => {
                let mut s = determinize::determinize_with(
                    view,
                    &brain.prior_map(view, *i),
                    seed,
                    brain.decisions.fog_army_forward_percent,
                    brain.opponents.threats.top().map(|t| t.rival),
                );
                if config.economy {
                    calibrate_economy(&mut s, view);
                }
                s
            }
        };
        // Low bits: the rival variant; high bits: the rollout policy level (`rival_brain`).
        (
            state,
            seed,
            if *i + config.proxy_rivals as u64 >= d {
                PROXY_RIVAL
            } else {
                (*i % config.opponents.max(1) as u64) as u8
            } | (config.rollout_policy << 4),
        )
    })
}

/// Mean of each candidate's samples.
fn means(results: &[Vec<i32>]) -> Vec<i32> {
    results
        .iter()
        .map(|r| r.iter().sum::<i32>() / r.len().max(1) as i32)
        .collect()
}

impl SearchHook<'_> {
    /// Horizon win chance per sample for each candidate plan, in order.
    pub(crate) fn evaluate(&self, view: &PlayerView, pool: &[PlanKind]) -> (Vec<Vec<i32>>, u32) {
        let horizon = SearchConfig::turns(self.config.horizon);
        let starts = starts(&self.config, self.brain, self.oracle, view);
        let jobs: Vec<(usize, usize)> = (0..pool.len())
            .flat_map(|k| (0..starts.len()).map(move |s| (k, s)))
            .collect();
        let results = map_jobs(&jobs, |(k, s)| {
            let mut brain = self.brain.rollout_copy_with(self.config.rollout_policy);
            brain.campaigns.plan.forced = Some(ForcedPlan {
                kind: pool[*k].clone(),
                until: view.turn + horizon,
            });
            let (state, seed, variant) = &starts[*s];
            rollout_relative(
                state.clone(),
                view.me,
                rollout_seat(brain, self.config.rollout_policy, view.turn),
                horizon,
                *seed,
                *variant,
                self.brain.decisions,
            )
        });
        let mut out = vec![Vec::new(); pool.len()];
        for ((k, _), r) in jobs.iter().zip(results) {
            out[*k].push(r);
        }
        (out, horizon)
    }
}

impl SearchHook<'_> {
    /// Horizon win chance per sample for each posture under one forced plan.
    /// Horizon win chance per sample for each (plan, posture) candidate, in order.
    pub(crate) fn evaluate_composites(
        &self,
        view: &PlayerView,
        pool: &[(PlanKind, Option<crate::plan::Posture>)],
    ) -> (Vec<Vec<i32>>, u32) {
        let horizon = SearchConfig::turns(self.config.horizon);
        let starts = starts(&self.config, self.brain, self.oracle, view);
        let jobs: Vec<(usize, usize)> = (0..pool.len())
            .flat_map(|k| (0..starts.len()).map(move |s| (k, s)))
            .collect();
        let results = map_jobs(&jobs, |(k, s)| {
            let mut brain = self.brain.rollout_copy_with(self.config.rollout_policy);
            brain.campaigns.plan.forced = Some(ForcedPlan {
                kind: pool[*k].0.clone(),
                until: view.turn + horizon,
            });
            brain.campaigns.plan.posture = pool[*k].1;
            let (state, seed, variant) = &starts[*s];
            rollout_relative(
                state.clone(),
                view.me,
                rollout_seat(brain, self.config.rollout_policy, view.turn),
                horizon,
                *seed,
                *variant,
                self.brain.decisions,
            )
        });
        let mut out = vec![Vec::new(); pool.len()];
        for ((k, _), r) in jobs.iter().zip(results) {
            out[*k].push(r);
        }
        (out, horizon)
    }

    pub(crate) fn evaluate_postures(
        &self,
        view: &PlayerView,
        plan: &PlanKind,
        postures: &[Option<crate::plan::Posture>],
    ) -> (Vec<Vec<i32>>, u32) {
        let horizon = SearchConfig::turns(self.config.horizon);
        let starts = starts(&self.config, self.brain, self.oracle, view);
        let jobs: Vec<(usize, usize)> = (0..postures.len())
            .flat_map(|k| (0..starts.len()).map(move |s| (k, s)))
            .collect();
        let results = map_jobs(&jobs, |(k, s)| {
            let mut brain = self.brain.rollout_copy_with(self.config.rollout_policy);
            brain.campaigns.plan.forced = Some(ForcedPlan {
                kind: plan.clone(),
                until: view.turn + horizon,
            });
            brain.campaigns.plan.posture = postures[*k];
            let (state, seed, variant) = &starts[*s];
            rollout_relative(
                state.clone(),
                view.me,
                rollout_seat(brain, self.config.rollout_policy, view.turn),
                horizon,
                *seed,
                *variant,
                self.brain.decisions,
            )
        });
        let mut out = vec![Vec::new(); postures.len()];
        for ((k, _), r) in jobs.iter().zip(results) {
            out[*k].push(r);
        }
        (out, horizon)
    }
}

/// A rival seat's policy in a rollout: variant 0 is scripted Hard; 1 Rush, 2 GreedyExpand
/// (full-game opening policies on Hard), 3 Hard with the Turtle bias.
/// Rival variant: the human-playbook proxy (Proxy v6), starting fresh from the rollout's view.
const PROXY_RIVAL: u8 = 4;

fn rival_brain(
    faction: legatum_core::Faction,
    seed: u64,
    id: PlayerId,
    variant: u8,
    start: u32,
) -> Box<dyn Brain> {
    let fast = variant >> 4;
    let hard = || {
        let mut b = ScriptedBrain::with_difficulty(faction, seed, id, AiDifficulty::Hard);
        b.fast = fast;
        b
    };
    let opening = |o: Opening| {
        let mut b = OpeningBrain::new(hard(), crate::hard_v2::policy(o));
        b.primitives_enabled = true;
        Box::new(b) as Box<dyn Brain>
    };
    let brain: Box<dyn Brain> = match variant & 0xF {
        1 => opening(Opening::Rush),
        2 => opening(Opening::GreedyExpand),
        PROXY_RIVAL => Box::new(crate::proxy_v6::HumanPlaybookV6Brain::default()),
        3 => {
            let mut b = ScriptedBrain::with_strategy(faction, seed, id, crate::Strategy::Turtle);
            b.difficulty = AiDifficulty::Hard;
            b.fast = fast;
            Box::new(b)
        }
        _ => Box::new(hard()),
    };
    match fast {
        0 | 1 => brain,
        2 => Box::new(Lazy::new(brain, 2, start)),
        _ => Box::new(Lazy::new(brain, 3, start)),
    }
}

/// A rollout seat that plans only every `every` turns from `start`; between, it submits no
/// orders (the rollout skips its view), so units keep their routes, cities their builds and
/// research its queue.
pub(crate) struct Lazy {
    inner: Box<dyn Brain>,
    every: u32,
    start: u32,
}

impl Lazy {
    pub(crate) fn new(inner: Box<dyn Brain>, every: u32, start: u32) -> Self {
        Self {
            inner,
            every,
            start,
        }
    }
}

impl Brain for Lazy {
    fn rests(&self, turn: u32) -> bool {
        !(turn.saturating_sub(self.start)).is_multiple_of(self.every)
    }
    fn plan(&mut self, view: &PlayerView) -> Vec<legatum_core::Order> {
        if self.rests(view.turn) {
            Vec::new()
        } else {
            self.inner.plan(view)
        }
    }
}

/// The seat's own brain in a rollout at policy level `fast`: level 4 plans every other turn.
pub fn rollout_seat(brain: ScriptedBrain, fast: u8, start: u32) -> Box<dyn Brain> {
    if fast >= 4 {
        Box::new(Lazy::new(Box::new(brain), 2, start))
    } else {
        Box::new(brain)
    }
}

thread_local! {
    /// This thread's city yields cache for rollout views. `ViewCache` compares every input of
    /// a cached result by value, so one cache serves every seat, turn and rollout (and any
    /// state) without going stale; it never changes a view, only how often yields are computed.
    static ROLLOUT_VIEWS: std::cell::RefCell<legatum_core::ViewCache> =
        std::cell::RefCell::new(legatum_core::ViewCache::default());
}

/// Run `f` with this thread's rollout view cache (a fresh one if a rollout is already using it).
pub(crate) fn with_rollout_views<R>(f: impl FnOnce(&mut legatum_core::ViewCache) -> R) -> R {
    let mut cache = ROLLOUT_VIEWS.with(|c| c.take());
    let result = f(&mut cache);
    ROLLOUT_VIEWS.with(|c| *c.borrow_mut() = cache);
    result
}

/// Play `horizon` turns: this seat with `seat`, every other seat its rival policy
/// (`variant`, see `rival_brain`). Returns the seat's win chance at the end.
pub fn rollout(
    state: GameState,
    me: PlayerId,
    seat: Box<dyn Brain>,
    horizon: u32,
    seed: u64,
    variant: u8,
) -> i32 {
    rollout_with(state, me, seat, horizon, seed, variant, chance_in)
}

fn rollout_relative(
    state: GameState,
    me: PlayerId,
    seat: Box<dyn Brain>,
    horizon: u32,
    seed: u64,
    variant: u8,
    decisions: crate::DecisionParams,
) -> i32 {
    rollout_with(state, me, seat, horizon, seed, variant, move |state, me| {
        determinize::chance_in_with(state, me, &decisions)
    })
}

/// `rollout`, scored by `eval` at the end.
pub fn rollout_with(
    mut state: GameState,
    me: PlayerId,
    seat: Box<dyn Brain>,
    horizon: u32,
    seed: u64,
    variant: u8,
    eval: impl Fn(&GameState, PlayerId) -> i32,
) -> i32 {
    let start = state.turn;
    let mut brains: BTreeMap<PlayerId, Box<dyn Brain>> = state
        .players
        .iter()
        .filter(|p| p.alive && p.id != me)
        .map(|p| {
            let s = Rng::derive(seed, &[STREAM, p.id.0 as u64]).next_u64();
            (p.id, rival_brain(p.faction, s, p.id, variant, start))
        })
        .collect();
    brains.insert(me, seat);
    let end = state.turn + horizon;
    with_rollout_views(|cache| {
        while !state.finished && state.turn < end {
            // Viewer-independent parts once per state; decision-only views (what AI seats read).
            let public = legatum_core::public_view(&state);
            let orders: BTreeMap<PlayerId, Vec<legatum_core::Order>> = brains
                .iter_mut()
                .filter(|(id, _)| state.player(**id).alive)
                .map(|(id, b)| {
                    if b.rests(state.turn) {
                        return (*id, Vec::new());
                    }
                    let view = legatum_core::player_view_cached(
                        &state,
                        *id,
                        &public,
                        legatum_core::ViewDetail::Decisions,
                        cache,
                    );
                    let o = b.orders(&view);

                    (*id, o)
                })
                .collect();
            state = legatum_core::resolve_turn(&state, &orders).0;
        }
    });
    eval(&state, me)
}

/// Short-horizon combat score for tactical search: our military strength, cities and city HP
/// minus the same for every team at war with us (a lost game is far below any material).
pub fn material(state: &GameState, me: PlayerId) -> i32 {
    material_weighted(state, me, 200, 1)
}

/// `material` with cities and city HP weighted three and four times as much: a capture or a
/// battered city outweighs the units it cost (tactical option `xc`).
pub fn material_captures(state: &GameState, me: PlayerId) -> i32 {
    material_weighted(state, me, 600, 4)
}

/// Material with `city` points per city and `hp_x2` half-points per city HP.
fn material_weighted(state: &GameState, me: PlayerId, city: i32, hp_x2: i32) -> i32 {
    let team = state.team_of(me);
    if state.finished {
        return if state.winner == Some(team) {
            1_000_000
        } else {
            -1_000_000
        };
    }
    let side = |p: PlayerId| -> i32 {
        let faction = state.player(p).faction;
        let army: i32 = state
            .units
            .values()
            .filter(|u| u.owner == p)
            .map(|u| {
                let s = legatum_core::data::unit_stats_for(faction, u.kind);
                if s.military {
                    s.strength.max(s.ranged) * u.hp / 100
                } else {
                    0
                }
            })
            .sum();
        let cities: i32 = state
            .cities_of(p)
            .map(|c| city + c.hp * hp_x2 / 2 + 20 * c.population)
            .sum();
        army + cities
    };
    let ours: i32 = state.team_members(team).into_iter().map(side).sum();
    let theirs: i32 = state
        .players
        .iter()
        .filter(|p| p.alive && p.team != team && state.teams_at_war(team, p.team))
        .map(|p| side(p.id))
        .sum();
    ours - theirs
}

/// How aggressively the seat fights this turn (Hard+Search tactics, `SearchConfig::stances`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    /// Attack at worse odds: threshold x0.8, accepted loss chance +20.
    Aggressive,
    /// Attack only at better odds: threshold x1.25, accepted loss chance -15.
    Cautious,
}

impl Stance {
    pub const ALL: [Stance; 2] = [Stance::Aggressive, Stance::Cautious];

    pub fn threshold(stance: Option<Self>, base: i32) -> i32 {
        match stance {
            Some(Self::Aggressive) => base * 4 / 5,
            Some(Self::Cautious) => base * 5 / 4,
            None => base,
        }
    }

    pub fn max_loss(stance: Option<Self>, base: i32) -> i32 {
        match stance {
            Some(Self::Aggressive) => base + 20,
            Some(Self::Cautious) => base - 15,
            None => base,
        }
    }
}

impl SearchHook<'_> {
    /// Material after a short horizon for each stance, the running plan kept.
    pub(crate) fn evaluate_stances(
        &self,
        view: &PlayerView,
        stances: &[Option<Stance>],
        plan: Option<&PlanKind>,
    ) -> (Vec<Vec<i32>>, u32) {
        let horizon = SearchConfig::turns(self.config.stance_horizon.max(1));
        let starts = starts(&self.config, self.brain, self.oracle, view);
        let jobs: Vec<(usize, usize)> = (0..stances.len())
            .flat_map(|k| (0..starts.len()).map(move |s| (k, s)))
            .collect();
        let results = map_jobs(&jobs, |(k, s)| {
            let mut brain = self.brain.rollout_copy_with(self.config.rollout_policy);
            if let Some(kind) = plan {
                brain.campaigns.plan.forced = Some(ForcedPlan {
                    kind: kind.clone(),
                    until: view.turn + horizon,
                });
            }
            brain.campaigns.plan.stance = stances[*k];
            let (state, seed, variant) = &starts[*s];
            rollout_with(
                state.clone(),
                view.me,
                rollout_seat(brain, self.config.rollout_policy, view.turn),
                horizon,
                *seed,
                *variant,
                material,
            )
        });
        let mut out = vec![Vec::new(); stances.len()];
        for ((k, _), r) in jobs.iter().zip(results) {
            out[*k].push(r);
        }
        (out, horizon)
    }
}

/// Hard+Search with a searched opening: on its first turn the seat rolls out every opening
/// policy (each handing off to Hard when it completes) and plays the best; afterwards the
/// Hard plan layer searches its review points as usual.
pub struct SearchBrain {
    inner: Inner,
    config: SearchConfig,
    log: Vec<SearchEvent>,
    /// Plan-layer events already copied into `log`.
    copied: usize,
    decided: bool,
}

enum Inner {
    Hard(Box<ScriptedBrain>),
    Opening(Box<OpeningBrain>),
    /// Only while the opening is being chosen.
    Empty,
}

impl SearchBrain {
    pub fn new(hard: ScriptedBrain, mut config: SearchConfig) -> Self {
        if matches!(hard.decisions.learned_macro, 1 | 3) {
            config.openings = true;
        }
        Self {
            inner: Inner::Hard(Box::new(hard.with_search(config))),
            config,
            log: Vec::new(),
            copied: 0,
            decided: !config.openings,
        }
    }

    fn hard(&self) -> &ScriptedBrain {
        match &self.inner {
            Inner::Hard(b) => b,
            Inner::Opening(b) => &b.base,
            Inner::Empty => unreachable!("opening being chosen"),
        }
    }

    fn hard_mut(&mut self) -> &mut ScriptedBrain {
        match &mut self.inner {
            Inner::Hard(b) => b,
            Inner::Opening(b) => &mut b.base,
            Inner::Empty => unreachable!("opening being chosen"),
        }
    }

    fn dyn_brain(&self) -> &dyn Brain {
        match &self.inner {
            Inner::Hard(b) => b.as_ref(),
            Inner::Opening(b) => b.as_ref(),
            Inner::Empty => unreachable!("opening being chosen"),
        }
    }

    fn opening_seat(base: ScriptedBrain, opening: Opening) -> Box<dyn Brain> {
        if opening == Opening::Default {
            return Box::new(base);
        }
        let mut b = OpeningBrain::new(base, crate::hard_v2::policy(opening)).with_handoff();
        b.primitives_enabled = true;
        Box::new(b)
    }

    fn choose_opening(&mut self, view: &PlayerView) {
        self.decided = true;
        // The oracle training set covers four- and eight-player Standard games. The few
        // two-player benchmark openings are not enough to replace the duel policy.
        if view.players.len() < 4 && matches!(self.hard().decisions.learned_macro, 1 | 3) {
            return;
        }
        let hard = self.hard();
        if matches!(hard.decisions.learned_macro, 1 | 3) {
            let per: Vec<i32> = Opening::ALL
                .iter()
                .map(|o| {
                    let f = crate::learned_macro::features(view, &PlanKind::Economy, Some(*o));
                    crate::learned_macro::value(&f, &PlanKind::Economy, Some(*o))
                })
                .collect();
            let chosen = (0..per.len())
                .max_by_key(|i| (per[*i], std::cmp::Reverse(*i)))
                .expect("openings");
            self.log.push(SearchEvent {
                turn: view.turn,
                chance: crate::winchance::WinFacts::of(view).chance(),
                options: Opening::ALL
                    .iter()
                    .zip(&per)
                    .map(|(o, mean)| SearchOption {
                        kind: PlanKind::Economy,
                        opening: Some(*o),
                        posture: None,
                        value: 0,
                        mean: *mean,
                        samples: Vec::new(),
                        beyond: 0,
                    })
                    .collect(),
                chosen,
                analytic: 0,
                omniscient: false,
                trigger: "learned:opening".into(),
                horizon: 0,
            });
            if Opening::ALL[chosen] != Opening::Default
                && let Inner::Hard(base) = std::mem::replace(&mut self.inner, Inner::Empty)
            {
                let mut b = OpeningBrain::new(*base, crate::hard_v2::policy(Opening::ALL[chosen]))
                    .with_handoff();
                b.primitives_enabled = true;
                self.inner = Inner::Opening(Box::new(b));
            }
            return;
        }
        let oracle = hard.oracle.as_deref().filter(|_| self.config.omniscient);
        let horizon = SearchConfig::turns(self.config.opening_horizon);
        let starts = starts(&self.config, hard, oracle, view);
        let jobs: Vec<(usize, usize)> = (0..Opening::ALL.len())
            .flat_map(|k| (0..starts.len()).map(move |s| (k, s)))
            .collect();
        let results = map_jobs(&jobs, |(k, s)| {
            let (state, seed, variant) = &starts[*s];
            rollout_relative(
                state.clone(),
                view.me,
                Self::opening_seat(
                    hard.rollout_copy_with(self.config.rollout_policy),
                    Opening::ALL[*k],
                ),
                horizon,
                *seed,
                *variant,
                hard.decisions,
            )
        });
        let mut per = vec![Vec::new(); Opening::ALL.len()];
        for ((k, _), r) in jobs.iter().zip(results) {
            per[*k].push(r);
        }
        let means = means(&per);
        let default = Opening::ALL
            .iter()
            .position(|o| *o == Opening::Default)
            .unwrap_or(0);
        // Highest mean; ties keep Default, then the list order.
        let chosen = (0..per.len())
            .max_by_key(|i| (means[*i], *i == default, std::cmp::Reverse(*i)))
            .expect("openings");
        self.log.push(SearchEvent {
            turn: view.turn,
            chance: crate::winchance::WinFacts::of(view)
                .with_relative(&hard.decisions)
                .chance(),
            options: Opening::ALL
                .iter()
                .zip(&per)
                .zip(&means)
                .map(|((o, r), m)| SearchOption {
                    kind: PlanKind::Economy,
                    opening: Some(*o),
                    posture: None,
                    value: 0,
                    mean: *m,
                    samples: r.clone(),
                    beyond: 0,
                })
                .collect(),
            chosen,
            analytic: default,
            omniscient: oracle.is_some(),
            trigger: "opening".into(),
            horizon,
        });
        let opening = Opening::ALL[chosen];
        if opening != Opening::Default
            && let Inner::Hard(base) = std::mem::replace(&mut self.inner, Inner::Empty)
        {
            let mut b = OpeningBrain::new(*base, crate::hard_v2::policy(opening)).with_handoff();
            b.primitives_enabled = true;
            self.inner = Inner::Opening(Box::new(b));
        }
    }
}

impl Brain for SearchBrain {
    fn add_air_orders(&self, view: &PlayerView, orders: &mut Vec<legatum_core::Order>) {
        if self.hard().decisions.air_wrapper_orders > 0 {
            self.hard().add_air_orders(view, orders);
        }
    }

    fn opponent_memory(&self) -> Option<crate::OpponentMemory> {
        self.dyn_brain().opponent_memory()
    }
    fn restore_opponent_memory(&mut self, memory: crate::OpponentMemory) {
        self.hard_mut().restore_opponent_memory(memory);
    }
    fn strategy_switches(&self) -> &[crate::opponents::StrategySwitch] {
        self.dyn_brain().strategy_switches()
    }
    fn threat_log(&self) -> &[crate::threat::ThreatEvent] {
        self.dyn_brain().threat_log()
    }
    fn campaign_events(&self) -> &[crate::campaign::CampaignEvent] {
        self.dyn_brain().campaign_events()
    }
    fn plan_events(&self) -> &[crate::plan::PlanEvent] {
        self.hard().plan_events()
    }
    fn blocked_expenses(&self) -> u32 {
        self.hard().blocked_expenses()
    }
    fn search_events(&self) -> &[SearchEvent] {
        &self.log
    }
    fn opening_handoff(&self) -> Option<crate::openings::OpeningHandoff> {
        self.dyn_brain().opening_handoff()
    }
    fn opening_phase_switches(&self) -> &[crate::openings::PhaseSwitch] {
        self.dyn_brain().opening_phase_switches()
    }
    fn opening(&self) -> Option<Opening> {
        self.dyn_brain().opening()
    }
    fn wants_true_state(&self) -> bool {
        self.config.omniscient
    }
    fn observe_true_state(&mut self, state: std::sync::Arc<GameState>) {
        self.hard_mut().observe_true_state(state);
    }
    fn plan(&mut self, view: &PlayerView) -> Vec<legatum_core::Order> {
        if !self.decided && !view.finished {
            self.choose_opening(view);
        }
        let orders = match &mut self.inner {
            Inner::Hard(b) => b.plan(view),
            Inner::Opening(b) => b.plan(view),
            Inner::Empty => unreachable!("opening chosen"),
        };
        let events = &self.hard().search_log;
        if events.len() > self.copied {
            let new = events[self.copied..].to_vec();
            self.copied = events.len();
            self.log.extend(new);
        }
        orders
    }
}

/// Rollouts run on their own pool (all cores, or `LEGATUM_ROLLOUT_THREADS`), so a search
/// waiting on its rollouts never steals unrelated work (a whole simulator game) from the
/// caller's pool; decision times then measure the decision.
#[cfg(feature = "parallel-search")]
fn rollout_pool() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        let threads = std::env::var("LEGATUM_ROLLOUT_THREADS")
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|i| format!("rollout-{i}"))
            .build()
            .expect("rollout pool")
    })
}

/// Run independent jobs, in parallel where the build allows, keeping input order.
#[cfg(feature = "parallel-search")]
pub(crate) fn map_jobs<T: Sync, R: Send>(jobs: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    use rayon::prelude::*;
    let run = || rollout_pool().install(|| jobs.par_iter().map(&f).collect());
    if rayon::current_thread_index().is_none() {
        return run();
    }
    // A rayon worker waiting on another pool keeps stealing its own pool's jobs (here whole
    // simulator games, nested on its stack until it overflows). Wait from a plain thread,
    // whose join blocks without stealing.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(8 << 20)
            .spawn_scoped(scope, run)
            .expect("rollout dispatcher")
            .join()
            .expect("rollouts")
    })
}

#[cfg(not(feature = "parallel-search"))]
pub(crate) fn map_jobs<T: Sync, R: Send>(jobs: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    jobs.iter().map(f).collect()
}

#[cfg(test)]
mod tests;

/// The seat's orders this turn with its committed strike's battle searched (`StrikeTactics`).
fn strike_tactics(
    t: &mut StrikeTactics,
    brain: &ScriptedBrain,
    state: &GameState,
    me: PlayerId,
    orders: &mut BTreeMap<PlayerId, Vec<legatum_core::Order>>,
) {
    use crate::operation::{OperationKind, Phase};
    use crate::training::{BattleKind, Stance};
    let Some(op) = brain
        .campaigns
        .operation
        .as_ref()
        .filter(|op| op.running() && brain.decisions.operations > 0)
    else {
        return;
    };
    let OperationKind::Strike { target, .. } = op.kind else {
        return;
    };
    if !matches!(op.phase, Phase::Strike | Phase::Storm) {
        return;
    }
    let Some(hard) = orders.get(&me).cloned() else {
        return;
    };
    let r = crate::training::search_turn_with(
        state,
        BTreeMap::from([(me, hard)]),
        &t.cfg,
        &t.plans,
        |b| b.kind == BattleKind::Attack && b.city == Some(target),
        || 0,
        |_| {},
    );
    t.plans = r.plans;
    for rec in &r.records {
        t.stats.turns += 1;
        t.stats.rollouts += rec.rollouts;
        if rec.changed_units > 0 || rec.stance != Stance::Hard {
            t.stats.changed += 1;
        }
        let verdict = rec
            .confirm
            .as_ref()
            .map_or_else(|| "arena".to_string(), |c| c.reason.clone());
        *t.stats.verdicts.entry(verdict).or_default() += 1;
    }
    if let Some(o) = r.orders.get(&me) {
        orders.insert(me, o.clone());
    }
}

/// Offline-only branching state for experiments with sequences of the existing Hard plans.
/// Its initial state is sampled solely from the searching seat's view. Every branch retains
/// the simulated brains' observation memory; rivals use the same scripted Hard policy as the
/// ordinary plan rollouts. This API is intentionally unused by the shipped AI.
pub struct OracleNode {
    state: GameState,
    brains: Vec<ScriptedBrain>,
    me: PlayerId,
    checkpoints: Vec<OracleMetrics>,
    search_rivals: bool,
    opening: Option<OpeningBrain>,
    settlers_stolen: u32,
    settlers_lost: u32,
    cities_captured: u32,
    cities_lost: u32,
    strike_tactics: Option<StrikeTactics>,
}

/// Training round 2: a committed strike (`operation.rs`) fights its battle at the target
/// with the Training search (`training.rs`) instead of plain Hard tactics, once the strike
/// has begun. Offline only.
#[derive(Clone, Debug)]
struct StrikeTactics {
    cfg: crate::training::TrainingConfig,
    plans: Vec<crate::training::StoredPlan>,
    stats: TacticsStats,
}

/// What the searched strike tactics did over a branch.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TacticsStats {
    /// Turns with a searched battle at the strike's target.
    pub turns: u32,
    /// Turns whose played orders differ from Hard's (units changed, or a non-Hard stance).
    pub changed: u32,
    /// The full-game check's verdicts (`clear`, `continued`, `hard`, `cap`), and battles the
    /// arena alone decided (`arena`).
    pub verdicts: BTreeMap<String, u32>,
    pub rollouts: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct OracleMetrics {
    pub turn: u32,
    pub finished: bool,
    pub won: bool,
    pub chance: i32,
    pub vp: u32,
    pub cities: usize,
    pub population: u32,
    pub production: i32,
    pub gold: i32,
    pub science: i32,
    pub rival_cities: usize,
    pub rival_production: i32,
    pub city_detail: Vec<OracleCity>,
    pub settlers_stolen: u32,
    pub settlers_lost: u32,
    pub cities_captured: u32,
    pub cities_lost: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct OracleCity {
    pub id: u32,
    pub owner: u8,
    pub population: u32,
    pub production: i32,
    pub gold: i32,
    pub science: i32,
    pub walls: bool,
}

impl OracleNode {
    pub fn new(view: &PlayerView, brain: &ScriptedBrain, sample: u64) -> Self {
        let seed = Rng::derive(
            brain.seed,
            &[STREAM, view.turn as u64, view.me.0 as u64, sample],
        )
        .next_u64();
        let state = determinize::determinize_with(
            view,
            &brain.prior_map(view, sample),
            seed,
            brain.decisions.fog_army_forward_percent,
            brain.opponents.threats.top().map(|t| t.rival),
        );
        let brains = state
            .players
            .iter()
            .map(|p| {
                if p.id == view.me {
                    brain.rollout_copy_with(0)
                } else {
                    let rival_seed = Rng::derive(seed, &[STREAM, p.id.0 as u64]).next_u64();
                    ScriptedBrain::with_difficulty(p.faction, rival_seed, p.id, AiDifficulty::Hard)
                }
            })
            .collect();
        Self {
            state,
            brains,
            me: view.me,
            checkpoints: Vec::new(),
            search_rivals: false,
            opening: None,
            settlers_stolen: 0,
            settlers_lost: 0,
            cities_captured: 0,
            cities_lost: 0,
            strike_tactics: None,
        }
    }

    /// Replay a line selected on determinized samples on the recorded map. The state is used
    /// only as the environment for this held-out validation; every brain still reads its
    /// ordinary PlayerView before submitting orders.
    pub fn from_recorded_state(state: &GameState, brains: &[ScriptedBrain], me: PlayerId) -> Self {
        Self {
            state: state.clone(),
            brains: brains.iter().map(ScriptedBrain::rollout_copy).collect(),
            me,
            checkpoints: Vec::new(),
            search_rivals: false,
            opening: None,
            settlers_stolen: 0,
            settlers_lost: 0,
            cities_captured: 0,
            cities_lost: 0,
            strike_tactics: None,
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            state: self.state.clone(),
            brains: self
                .brains
                .iter()
                .map(ScriptedBrain::rollout_copy)
                .collect(),
            me: self.me,
            checkpoints: self.checkpoints.clone(),
            search_rivals: self.search_rivals,
            opening: self.opening.as_ref().map(OpeningBrain::rollout_copy),
            settlers_stolen: self.settlers_stolen,
            settlers_lost: self.settlers_lost,
            cities_captured: self.cities_captured,
            cities_lost: self.cities_lost,
            strike_tactics: self.strike_tactics.clone(),
        }
    }

    /// Training round 2: fight a committed strike's battle with the Training search under
    /// `cfg` (None: plain Hard tactics, the default).
    pub fn set_strike_tactics(&mut self, cfg: Option<crate::training::TrainingConfig>) {
        self.strike_tactics = cfg.map(|cfg| StrikeTactics {
            cfg,
            plans: Vec::new(),
            stats: TacticsStats::default(),
        });
    }

    /// What the searched strike tactics did on this branch so far.
    pub fn tactics_stats(&self) -> Option<&TacticsStats> {
        self.strike_tactics.as_ref().map(|t| &t.stats)
    }

    pub fn active_opening(&self) -> Option<Opening> {
        self.opening
            .as_ref()
            .filter(|b| b.opening_handoff().is_none())
            .map(|b| b.policy.opening)
    }

    pub fn end_opening(&mut self) {
        if let Some(opening) = self.opening.take() {
            self.brains[self.me.0 as usize] = opening.base;
        }
    }

    /// Use the recorded opponents' Hard + Search brain at each simulated turn. The per-turn
    /// wrapper is rebuilt from durable observation memory so branching remains cheap to copy.
    pub fn set_search_rivals(&mut self, enabled: bool) {
        self.search_rivals = enabled;
    }

    pub fn checkpoints(&self) -> &[OracleMetrics] {
        &self.checkpoints
    }

    fn record_checkpoint(&mut self) {
        if matches!(self.state.turn, 50 | 60 | 62 | 100 | 123 | 150) {
            self.checkpoints.push(self.metrics());
        }
    }

    fn record_events(&mut self, events: &[legatum_core::Event]) {
        for event in events {
            match event {
                legatum_core::Event::SettlerCaptured { from, to, .. } => {
                    if *to == self.me {
                        self.settlers_stolen += 1;
                    }
                    if *from == self.me {
                        self.settlers_lost += 1;
                    }
                }
                legatum_core::Event::CityCaptured { from, to, .. } => {
                    if *to == self.me {
                        self.cities_captured += 1;
                    }
                    if *from == self.me {
                        self.cities_lost += 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Begin a named opening. It persists across macro intervals until it hands off or a
    /// later branch explicitly chooses a plan, preserving its phase and observed investments.
    pub fn advance_opening(&mut self, opening: Opening, turns: u32) {
        if opening == Opening::Default {
            self.advance_plain(turns);
            return;
        }
        let index = self.me.0 as usize;
        let base = self.brains[index].rollout_copy();
        let mut opener = OpeningBrain::new(base, crate::hard_v2::policy(opening)).with_handoff();
        opener.primitives_enabled = true;
        self.opening = Some(opener);
        self.advance_plain(turns);
    }

    /// All currently feasible plan candidates, including each visible reachable campaign.
    /// Values are the live analytic plan's values, before rollout evaluation.
    pub fn choices(&mut self) -> Vec<(PlanKind, i32)> {
        if self.state.finished || !self.state.player(self.me).alive {
            return Vec::new();
        }
        let view = legatum_core::player_view(&self.state, self.me);
        let brain = if let Some(opening) = self.opening.as_mut() {
            &mut opening.base
        } else {
            &mut self.brains[self.me.0 as usize]
        };
        brain.assess_opponents(&view);
        let planner = brain.planner(&view);
        planner
            .plan_candidates()
            .into_iter()
            .map(|c| (c.kind, c.value))
            .collect()
    }

    /// Force a candidate while it remains feasible, then let every seat submit orders through
    /// the normal engine. An infeasible plan falls back to Hard's ordinary plan choice.
    pub fn advance(&mut self, plan: &PlanKind, turns: u32) {
        self.end_opening();
        let end = self.state.turn.saturating_add(turns);
        self.brains[self.me.0 as usize].campaigns.plan.forced = Some(ForcedPlan {
            kind: plan.clone(),
            until: end,
        });
        self.advance_plain(turns);
    }

    pub fn continue_opening(&mut self, turns: u32) {
        self.advance_plain(turns);
    }

    /// Offline operation search (`operation_search`): play `turns` with this seat's decision
    /// genes replaced by `decisions` and, when given, `plan` forced while it stays on offer.
    /// The seat's own genes return afterwards. Returns whether the forced plan was the seat's
    /// active plan after the first turn (always true with no plan).
    pub fn advance_operation(
        &mut self,
        plan: Option<&PlanKind>,
        decisions: Option<&crate::DecisionParams>,
        turns: u32,
    ) -> bool {
        self.end_opening();
        let index = self.me.0 as usize;
        let saved = self.brains[index].decisions;
        if let Some(d) = decisions {
            self.brains[index].decisions = *d;
        }
        if let Some(kind) = plan {
            self.brains[index].campaigns.plan.forced = Some(ForcedPlan {
                kind: kind.clone(),
                until: self.state.turn.saturating_add(turns),
            });
        }
        self.advance_plain(turns.min(1));
        let took =
            plan.is_none_or(|kind| self.active_plan().is_some_and(|p| p.same_objective(kind)));
        self.advance_plain(turns.saturating_sub(1));
        self.brains[index].decisions = saved;
        took
    }

    /// Offline operation search with the committed executor (gene `operations`,
    /// `operation.rs`): play `turns` with this seat's genes replaced by `decisions` (the gene
    /// switched on) and `operation` committed, then restore the genes. Returns the operation as
    /// it ended (its outcome and log); one still running when the turns are up ends `Expired`.
    pub fn advance_committed(
        &mut self,
        operation: crate::operation::Operation,
        decisions: &crate::DecisionParams,
        turns: u32,
    ) -> crate::operation::Operation {
        self.end_opening();
        let index = self.me.0 as usize;
        let saved = self.brains[index].decisions;
        let mut d = *decisions;
        d.operations = 1;
        self.brains[index].decisions = d;
        self.brains[index].campaigns.plan.forced = None;
        self.brains[index].set_operation(operation);
        self.advance_plain(turns);
        self.brains[index].decisions = saved;
        let turn = self.state.turn;
        let op = self.brains[index]
            .campaigns
            .operation
            .as_mut()
            .expect("the operation stays on the seat");
        // Its budget is these turns: one still running here has used it up.
        if op.running() {
            op.outcome = crate::operation::OperationOutcome::Expired;
            op.log.end_turn = Some(turn);
        }
        op.clone()
    }

    /// The seat's own decision genes.
    pub fn seat_decisions(&self) -> crate::DecisionParams {
        self.brains[self.me.0 as usize].decisions
    }

    /// The seat's active plan, if any.
    pub fn active_plan(&self) -> Option<PlanKind> {
        let brain = match self.opening.as_ref() {
            Some(opening) => &opening.base,
            None => &self.brains[self.me.0 as usize],
        };
        brain.campaigns.plan.active.as_ref().map(|p| p.kind.clone())
    }

    /// The current game state (offline analysis only).
    pub fn state(&self) -> &GameState {
        &self.state
    }

    fn advance_plain(&mut self, turns: u32) {
        let end = self.state.turn.saturating_add(turns);
        let search_rivals = self.search_rivals;
        let players = self.state.players.len();
        with_rollout_views(|cache| {
            while self.state.turn < end && !self.state.finished {
                let public = legatum_core::public_view(&self.state);
                let orders = self
                    .brains
                    .iter_mut()
                    .filter(|b| self.state.player(b.player).alive)
                    .map(|b| {
                        let view = legatum_core::player_view_cached(
                            &self.state,
                            b.player,
                            &public,
                            legatum_core::ViewDetail::Decisions,
                            cache,
                        );
                        let orders = if b.player == self.me {
                            if let Some(opening) = self.opening.as_mut() {
                                opening.orders(&view)
                            } else {
                                b.orders(&view)
                            }
                        } else if search_rivals {
                            let mut search =
                                SearchBrain::new(b.rollout_copy(), SearchConfig::in_app(players));
                            let orders = search.orders(&view);
                            if let Some(memory) = search.opponent_memory() {
                                b.restore_opponent_memory(memory);
                            }
                            orders
                        } else {
                            b.orders(&view)
                        };

                        (b.player, orders)
                    })
                    .collect();
                let mut orders: BTreeMap<PlayerId, Vec<legatum_core::Order>> = orders;
                if let Some(t) = self.strike_tactics.as_mut() {
                    let brain = match self.opening.as_ref() {
                        Some(o) => &o.base,
                        None => &self.brains[self.me.0 as usize],
                    };
                    strike_tactics(t, brain, &self.state, self.me, &mut orders);
                }
                let (state, events) = legatum_core::resolve_turn(&self.state, &orders);
                self.state = state;
                self.record_events(&events);
                self.record_checkpoint();
            }
        });
    }

    pub fn metrics(&self) -> OracleMetrics {
        let me = self.me;
        let team = self.state.player(me).team;
        let mut production = 0;
        let mut gold = 0;
        let mut science = 0;
        let mut population = 0;
        let mut rival_production = 0;
        let mut cities = 0;
        let mut rival_cities = 0;
        let mut city_detail = Vec::new();
        for city in self.state.cities.values() {
            let yields = legatum_core::economy::city_yields(&self.state, city).total;
            city_detail.push(OracleCity {
                id: city.id.0,
                owner: city.owner.0,
                population: city.population as u32,
                production: yields.production,
                gold: yields.gold,
                science: yields.science,
                walls: city.buildings.contains(&legatum_core::BuildingKind::Walls),
            });
            if self.state.player(city.owner).team == team {
                cities += 1;
                population += city.population as u32;
                production += yields.production;
                gold += yields.gold;
                science += yields.science;
            } else {
                rival_cities += 1;
                rival_production += yields.production;
            }
        }
        OracleMetrics {
            turn: self.state.turn,
            finished: self.state.finished,
            won: self.state.finished && self.state.winner == Some(team),
            chance: chance_in(&self.state, me),
            vp: legatum_core::scoring::team_vp(&self.state, team),
            cities,
            population,
            production,
            gold,
            science,
            rival_cities,
            rival_production,
            city_detail,
            settlers_stolen: self.settlers_stolen,
            settlers_lost: self.settlers_lost,
            cities_captured: self.cities_captured,
            cities_lost: self.cities_lost,
        }
    }
}
