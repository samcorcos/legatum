//! Playing one AI-only game, the determinism re-play, and tape replay.

use crate::stats::{
    DeterminismCheck, Elimination, FinalFacts, GameRecord, PlayerRecord, WonderCompletion,
    WonderStats, classify,
};
use crate::telemetry::{Collector, GameTelemetry};
use legatum_ai::{AiParams, Brain, ScriptedBrain, Strategy};
use legatum_core::{AiDifficulty, Event, GameSetup, GameState, scoring};
use legatum_wasm::panic_message;
use legatum_wasm::session::{BrainKind, Session};
use legatum_wasm::tape::{Tape, TapeTurn};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum BrainMode {
    Scripted,
    Random,
    /// Even seats scripted, odd seats random.
    Mixed,
}

impl BrainMode {
    pub fn brain_for(self, player: legatum_core::PlayerId) -> BrainKind {
        match self {
            BrainMode::Scripted => BrainKind::Scripted,
            BrainMode::Random => BrainKind::Random,
            BrainMode::Mixed if player.0.is_multiple_of(2) => BrainKind::Scripted,
            BrainMode::Mixed => BrainKind::Random,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BrainMode::Scripted => "scripted",
            BrainMode::Random => "random",
            BrainMode::Mixed => "mixed",
        }
    }
}

/// `--training DIR`: every scripted Hard AI seat plays Training mode (its battles searched,
/// `legatum_ai::training`), and each game writes its log to `DIR/game-<seed>.jsonl`, in the
/// same format as the app's.
pub struct TrainingSim {
    pub dir: std::path::PathBuf,
    pub cfg: legatum_ai::training::TrainingConfig,
}

pub static TRAINING: std::sync::OnceLock<TrainingSim> = std::sync::OnceLock::new();

/// Search this turn's battles for every Hard AI seat and log them (Training mode).
fn train_turn(
    s: &legatum_wasm::session::Session,
    orders: &mut BTreeMap<legatum_core::PlayerId, Vec<legatum_core::Order>>,
    tracker: &mut legatum_ai::training::OutcomeTracker,
    out: &mut impl std::io::Write,
) {
    let Some(t) = TRAINING.get() else { return };
    let hard: BTreeMap<_, _> = orders
        .iter()
        .filter(|(id, _)| {
            s.is_ai(**id) && s.state.setup.players[id.0 as usize].difficulty == AiDifficulty::Hard
        })
        .map(|(id, o)| (*id, o.clone()))
        .collect();
    if hard.is_empty() {
        return;
    }
    let result = legatum_ai::training::search_turn(&s.state, hard, &t.cfg, || 0, |_| {});
    tracker.open(&result.records);
    for r in result.records {
        let _ = writeln!(
            out,
            "{}",
            legatum_ai::training::LogLine::Decision(Box::new(r)).json()
        );
    }
    for (id, o) in result.orders {
        orders.insert(id, o);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RunOpts {
    pub lobby: crate::lobby::Config,
    pub rule_experiments: legatum_core::experiments::RuleExperiments,
    /// Barbarian camps (`GameSetup::barbarians`).
    pub barbarians: bool,
    pub safety_cap: Option<u32>,
    pub turn_limit: Option<u32>,
    pub brain: BrainMode,
    pub difficulty_rotation: Option<u8>,
    pub proxy: Option<crate::proxy_benchmark::Config>,
    pub strategies: [Option<Strategy>; 8],
    pub strategy_matchups: bool,
    /// Record per-turn hashes (and the tape).
    pub record: bool,
    pub determinism: bool,
    /// Observer-only counterfactual games; never changes submitted orders or state.
    pub tuning: Tuning,
}

/// Decision parameters for scripted seats (`--ai-params`, `--opponent-params`) and the
/// lobby difficulty outside Proxy runs. Parameters change only how brains decide (pillar 6).
#[derive(Clone, Copy, Debug, Default)]
pub struct Tuning {
    pub openings: crate::openings::Config,
    /// Every scripted seat, or the non-opponent half when `opponent` is set.
    pub params: Option<&'static AiParams>,
    /// Half the seats, alternating by game: seat s of game i when (s + i) is odd.
    pub opponent: Option<&'static AiParams>,
    /// Difficulty of every seat (non-Proxy lobbies; Proxy runs set their own).
    pub difficulty: Option<AiDifficulty>,
    /// Hard+Search (`--search`): scripted Hard seats decide plan reviews by rollouts.
    pub search: Option<legatum_ai::search::SearchConfig>,
    /// Only one search seat per game (seat = game index mod players), against plain Hard.
    pub search_rotation: bool,
    /// 1v1 schedule over every ordered faction pair (`--duel-matrix`): game i plays cell
    /// (i / 2) mod 64 on seed first + i / 2, side A in seat i mod 2.
    pub duel_matrix: bool,
    /// With `duel_matrix`: side B plays this exploiting opponent (a full-game opening policy
    /// on Hard, or a human-playbook proxy) instead of plain Hard.
    pub duel_opponent: Option<DuelOpponent>,
}

/// An exploiting opponent for side B of a duel (`--duel-opponent`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuelOpponent {
    /// A full-game opening policy on Hard.
    Opening(legatum_ai::openings::Opening),
    /// The human-playbook proxy of this version (5, 6, 7, 8 Sparta or 9 Mongols),
    /// playing from its own view only.
    Proxy(u32),
}

impl std::str::FromStr for DuelOpponent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().strip_prefix("proxy") {
            Some("5") => Ok(Self::Proxy(5)),
            Some("6") => Ok(Self::Proxy(6)),
            Some("7") => Ok(Self::Proxy(7)),
            Some("8o") => Ok(Self::Proxy(8)),
            Some("8g") => Ok(Self::Proxy(9)),
            Some(_) => Err("choose proxy5, proxy6, proxy7, proxy8o or proxy8g".into()),
            None => s.parse().map(Self::Opening),
        }
    }
}

/// The duel matrix cell of game `index`: (side A's faction, side B's faction, side A's seat).
pub fn duel_cell(index: usize) -> (legatum_core::Faction, legatum_core::Faction, usize) {
    let all = legatum_core::Faction::ALL;
    let cell = (index / 2) % (all.len() * all.len());
    (all[cell / all.len()], all[cell % all.len()], index % 2)
}

impl Tuning {
    /// Parameters for each seat of game `index`; None means the built-in AI.
    fn seats(&self, index: usize, players: usize) -> SeatPolicies {
        let mut openings = self.openings;
        if openings.hard_v2_rotation {
            openings.hard_v2 = std::array::from_fn(|seat| seat == index % players);
            openings.legacy_hard = std::array::from_fn(|seat| seat != index % players);
        }
        if let Some(challenger) = openings.challenger {
            openings.seats = [legatum_ai::openings::Opening::Default; 8];
            openings.seats[index % players] = challenger;
            openings.legacy_hard[index % players] = false;
        }
        if self.duel_matrix
            && let Some(DuelOpponent::Opening(opponent)) = self.duel_opponent
        {
            openings.seats[1 - index % 2] = opponent;
        }
        SeatPolicies {
            openings,
            params: std::array::from_fn(|seat| match self.opponent {
                Some(opp) if (seat + index) % 2 == 1 => Some(opp),
                _ => self.params,
            }),
            search: std::array::from_fn(|seat| {
                self.search
                    .filter(|_| !self.search_rotation || seat == index % players)
            }),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct SeatPolicies {
    params: [Option<&'static AiParams>; 8],
    openings: crate::openings::Config,
    search: [Option<legatum_ai::search::SearchConfig>; 8],
}

impl RunOpts {
    pub fn safety_cap(&self) -> u32 {
        self.safety_cap
            .unwrap_or(legatum_core::data::EXPECTED_GAME_TURNS * 4)
    }
}

pub fn setup_for(seed: u64, opts: &RunOpts) -> GameSetup {
    setup_for_game(seed, 0, opts)
}

fn setup_for_game(seed: u64, index: usize, opts: &RunOpts) -> GameSetup {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(opts.lobby.players);
    (setup.width, setup.height) = opts.lobby.map_size.dimensions();
    // One seat per faction, randomly assigned independently of map generation.
    // Modulo in u64/i64 before narrowing keeps this identical on native and wasm32.
    // Preserve established seeded sim rosters and their order tapes. Egypt and Rome join
    // through an explicit lobby roster until the next deliberate roster repin.
    let mut factions: [legatum_core::Faction; 9] = legatum_core::Faction::ALL[..9]
        .try_into()
        .expect("the original nine factions");
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    let mut factions = opts
        .lobby
        .factions
        .unwrap_or_else(|| std::array::from_fn(|i| factions[i]));
    if opts.lobby.rotate_factions {
        factions[..opts.lobby.players].rotate_right(index % opts.lobby.players);
    }
    for (seat, (player, faction)) in setup.players.iter_mut().zip(factions).enumerate() {
        player.faction = faction;
        if let Some(offset) = opts.difficulty_rotation {
            player.difficulty = [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard]
                [(seat + usize::from(offset)) % 3];
        }
    }
    if let Some(difficulty) = opts.tuning.difficulty {
        for player in &mut setup.players {
            player.difficulty = difficulty;
        }
    }
    if let Some(proxy) = opts.proxy {
        proxy.configure(&mut setup, index);
    }
    setup.rule_experiments = opts.rule_experiments;
    setup.barbarians = opts.barbarians;
    setup.turn_limit = opts.turn_limit;
    setup
}

/// One full play of a game.
pub struct Played {
    pub opening_records: Vec<crate::openings::SeatRecord>,
    pub state: GameState,
    pub tape: Tape,
    pub eliminations: Vec<Elimination>,
    pub brains: Vec<String>,
    pub strategies: [Option<Strategy>; 8],
    /// Turns resolved.
    pub turns: u32,
    pub city_captures: u32,
    pub athenian_city_captures: u32,
    pub athenian_free_techs_granted: u32,
    pub early_war_captures: u32,
    pub workable_claims_before: u32,
    pub workable_claims_after: u32,
    pub team_vp_turn_75: BTreeMap<u8, u32>,
    pub team_vp_turn_100: BTreeMap<u8, u32>,
    pub telemetry: GameTelemetry,
    pub archive: Option<legatum_wasm::history::Recording>,
    pub panic: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn observe_events(
    state: &GameState,
    turn: u32,
    events: &[Event],
    telemetry: &mut Collector,
    city_captures: &mut u32,
    athenian_city_captures: &mut u32,
    athenian_free_techs_granted: &mut u32,
    early_war_captures: &mut u32,
    eliminations: &mut Vec<Elimination>,
) {
    for (index, event) in events.iter().enumerate() {
        telemetry.observe_event(turn, event);
        match event {
            Event::CityCaptured { from, .. }
                if !from.is_barbarian()
                    && state.player(*from).faction == legatum_core::Faction::Athens =>
            {
                *athenian_city_captures += 1
            }
            Event::CityRazed {
                from,
                ordered: false,
                ..
            } if !from.is_barbarian()
                && state.player(*from).faction == legatum_core::Faction::Athens =>
            {
                *athenian_city_captures += 1
            }
            Event::AthensCityCaptured { .. } => *athenian_free_techs_granted += 1,
            _ => {}
        }
        if matches!(event, Event::CityCaptured { .. }) {
            *city_captures += 1;
            if turn < 60 {
                *early_war_captures += 1;
            }
        }
        if let Event::PlayerEliminated { player } = event {
            let by = events[..index].iter().rev().find_map(|event| match event {
                Event::CityCaptured { from, to, .. } if from == player => Some(to.0),
                Event::CityRazed {
                    from,
                    by,
                    ordered: false,
                    ..
                } if from == player => Some(by.0),
                _ => None,
            });
            eliminations.push(Elimination {
                player: player.0,
                turn,
                by,
            });
        }
    }
}

/// Play until victory or the simulation safety cap. A cap stop leaves the core unfinished
/// and is reported as stalled, never as a turn-limit victory.
#[cfg(test)]
pub fn play(setup: &GameSetup, mode: BrainMode, record: bool, safety_cap: u32) -> Played {
    play_with_brains(
        setup,
        mode,
        None,
        record,
        safety_cap,
        [None; 8],
        SeatPolicies::default(),
        None,
    )
}

/// The brain playing a seat: the runner's own when it built one, else the session's.
fn seat_brain<'a>(
    biased: &'a BTreeMap<legatum_core::PlayerId, Box<dyn Brain>>,
    s: &'a Session,
    seat: usize,
) -> Option<&'a dyn Brain> {
    let id = legatum_core::PlayerId(seat as u8);
    biased.get(&id).map(|b| b.as_ref()).or_else(|| s.brain(id))
}

#[allow(clippy::too_many_arguments)]
fn play_with_brains(
    setup: &GameSetup,
    mode: BrainMode,
    proxy: Option<(u8, u32, Option<legatum_ai::openings::Opening>)>,
    record: bool,
    safety_cap: u32,
    mut strategies: [Option<Strategy>; 8],
    policies: SeatPolicies,
    decision_trace: Option<&Path>,
) -> Played {
    let SeatPolicies {
        params,
        openings,
        search,
    } = policies;
    // A Proxy seat keeps its own policy even if a caller assigns experimental biases.
    if let Some((seat, _, _)) = proxy {
        strategies[usize::from(seat)] = None;
    }
    let brain_kind = |p: legatum_core::PlayerId| {
        if proxy.is_some_and(|(seat, _, _)| seat == p.0) {
            if proxy.is_some_and(|(_, version, _)| version == 2) {
                BrainKind::ProxyV2
            } else {
                BrainKind::Proxy
            }
        } else {
            mode.brain_for(p)
        }
    };
    // Tuned parameters apply to scripted seats only; a strategy bias takes precedence.
    let tuned = |p: legatum_core::PlayerId| {
        params[p.0 as usize]
            .filter(|_| brain_kind(p) == BrainKind::Scripted && strategies[p.0 as usize].is_none())
    };
    let mut s = Session::new(setup, brain_kind);
    let custom = decision_trace.is_some()
        || proxy.is_some_and(|(_, version, opening)| matches!(version, 5..=9) || opening.is_some())
        || openings.enabled()
        || strategies.iter().any(Option::is_some)
        || search.iter().any(Option::is_some)
        || s.state.players.iter().any(|p| tuned(p.id).is_some());
    // Hard+Search applies to scripted Hard seats, including parameterized ones.
    let search_seat = |p: legatum_core::PlayerId| {
        search[p.0 as usize].filter(|_| {
            brain_kind(p) == BrainKind::Scripted
                && strategies[p.0 as usize].is_none()
                && openings.seats[p.0 as usize] == legatum_ai::openings::Opening::Default
                && setup.players[p.0 as usize].difficulty == AiDifficulty::Hard
        })
    };
    let mut biased_brains: BTreeMap<_, Box<dyn Brain>> = if custom {
        s.state
            .players
            .iter()
            .filter(|p| !p.is_human)
            .map(|p| {
                let seed = legatum_wasm::session::brain_seed(setup.seed, p.id);
                let brain: Box<dyn Brain> = match (strategies[p.id.0 as usize], tuned(p.id)) {
                    _ if search_seat(p.id).is_some() => {
                        let base = match tuned(p.id) {
                            Some(params) => ScriptedBrain::with_params(
                                p.faction,
                                seed,
                                p.id,
                                AiDifficulty::Hard,
                                params,
                            ),
                            None => ScriptedBrain::with_difficulty(
                                p.faction,
                                seed,
                                p.id,
                                AiDifficulty::Hard,
                            ),
                        };
                        Box::new(legatum_ai::search::SearchBrain::new(
                            base,
                            search_seat(p.id).expect("search seat"),
                        ))
                    }
                    (Some(strategy), _) => Box::new(ScriptedBrain::with_strategy(
                        p.faction, seed, p.id, strategy,
                    )),
                    (None, Some(params)) => Box::new(ScriptedBrain::with_params(
                        p.faction,
                        seed,
                        p.id,
                        setup
                            .players
                            .get(p.id.0 as usize)
                            .map(|x| x.difficulty)
                            .unwrap_or_default(),
                        params,
                    )),
                    (None, None)
                        if brain_kind(p.id) == BrainKind::Scripted
                            && (openings.hard_v2[p.id.0 as usize] || openings.hard_v2_handoff)
                            && !openings.legacy_hard[p.id.0 as usize]
                            && setup.players[p.id.0 as usize].difficulty == AiDifficulty::Hard =>
                    {
                        if openings.hard_v2_handoff {
                            legatum_ai::hard_v2::handoff_brain(p.faction, seed, p.id)
                        } else {
                            legatum_ai::hard_v2::brain(p.faction, seed, p.id)
                        }
                    }
                    // League runs and --legacy-hard seats keep the scripted Hard as their base.
                    (None, None)
                        if brain_kind(p.id) == BrainKind::Scripted
                            && (openings.legacy_hard[p.id.0 as usize] || openings.league()) =>
                    {
                        Box::new(ScriptedBrain::with_difficulty(
                            p.faction,
                            seed,
                            p.id,
                            setup
                                .players
                                .get(p.id.0 as usize)
                                .map(|x| x.difficulty)
                                .unwrap_or_default(),
                        ))
                    }
                    (None, None)
                        if proxy.is_some_and(|(seat, _, opening)| {
                            seat == p.id.0 && opening.is_some()
                        }) =>
                    {
                        let opening = proxy
                            .and_then(|(_, _, opening)| opening)
                            .expect("selected opening");
                        let base = ScriptedBrain::with_difficulty(
                            p.faction,
                            seed,
                            p.id,
                            setup.players[p.id.0 as usize].difficulty,
                        );
                        let mut brain = legatum_ai::openings::OpeningBrain::new(
                            base,
                            legatum_ai::hard_v2::policy(opening),
                        );
                        brain.primitives_enabled = true;
                        Box::new(brain)
                    }
                    (None, None) if proxy == Some((p.id.0, 6, None)) => {
                        Box::new(legatum_ai::HumanPlaybookV6Brain::default())
                    }
                    (None, None) if proxy == Some((p.id.0, 7, None)) => {
                        Box::new(legatum_ai::HumanPlaybookV7Brain::default())
                    }
                    (None, None) if proxy == Some((p.id.0, 8, None)) => {
                        Box::new(legatum_ai::HumanSpartaBrain::default())
                    }
                    (None, None) if proxy == Some((p.id.0, 9, None)) => {
                        Box::new(legatum_ai::HumanMongolsBrain::default())
                    }
                    (None, None) if proxy == Some((p.id.0, 5, None)) => {
                        Box::new(legatum_ai::HumanPlaybookBrain::default())
                    }
                    (None, None) => {
                        legatum_wasm::session::make_brain(brain_kind(p.id), &s.state, p.id)
                    }
                };
                let opening = openings.seats[p.id.0 as usize];
                let brain = if opening != legatum_ai::openings::Opening::Default
                    || openings.ai_primitives && brain_kind(p.id) == BrainKind::Scripted
                {
                    let difficulty = setup.players[p.id.0 as usize].difficulty;
                    let base = match params[p.id.0 as usize] {
                        Some(params) => {
                            ScriptedBrain::with_params(p.faction, seed, p.id, difficulty, params)
                        }
                        None => ScriptedBrain::with_difficulty(p.faction, seed, p.id, difficulty),
                    };
                    let mut brain = legatum_ai::openings::OpeningBrain::new(
                        base,
                        openings.policy(p.id.0 as usize),
                    );
                    brain.primitives_enabled = true;
                    Box::new(brain) as Box<dyn Brain>
                } else {
                    brain
                };
                (p.id, brain)
            })
            .collect()
    } else {
        BTreeMap::new()
    };
    if !record {
        s.history = None;
    }
    let mut brains: Vec<String> = s
        .state
        .players
        .iter()
        .map(|p| match (strategies[p.id.0 as usize], tuned(p.id)) {
            _ if search_seat(p.id).is_some() => {
                format!(
                    "scripted+search:{}",
                    search_seat(p.id).expect("seat").label()
                )
            }
            (Some(strategy), _) => format!("scripted:{strategy}"),
            (None, Some(params)) => format!("scripted@{}", params.name),
            (None, None) => s.brain_kind(p.id).map_or("human", |k| k.name()).to_string(),
        })
        .collect();
    for (seat, name) in brains.iter_mut().enumerate() {
        if openings.seats[seat] != legatum_ai::openings::Opening::Default {
            *name = format!("opening:{}", openings.seats[seat].name());
        } else if let Some(o) = seat_brain(&biased_brains, &s, seat).and_then(|b| b.opening()) {
            *name = format!(
                "{}:{}",
                if openings.hard_v2_handoff {
                    "hard-v2-handoff"
                } else {
                    "hard-v2"
                },
                o.name()
            );
        }
    }
    let mut tape = Tape {
        version: legatum_wasm::tape::TAPE_VERSION,
        setup: setup.clone(),
        initial_hash: if record { s.state.hash() } else { 0 },
        turns: Vec::new(),
    };
    let mut opening_records: Vec<_> = openings
        .seats
        .iter()
        .take(setup.players.len())
        .enumerate()
        .map(|(seat, opening)| crate::openings::SeatRecord {
            ai_primitives: *opening != legatum_ai::openings::Opening::Default
                || openings.ai_primitives
                    && brain_kind(legatum_core::PlayerId(seat as u8)) == BrainKind::Scripted,
            opening: *opening,
            ..Default::default()
        })
        .collect();
    // Hard v2 seats record the opening they drew.
    for (seat, record) in opening_records.iter_mut().enumerate() {
        if let Some(o) = seat_brain(&biased_brains, &s, seat).and_then(|b| b.opening())
            && record.opening == legatum_ai::openings::Opening::Default
        {
            record.opening = o;
            record.ai_primitives = true;
        }
    }
    let mut eliminations = Vec::new();
    let mut played = 0u32;
    let mut city_captures = 0;
    let mut athenian_city_captures = 0;
    let mut athenian_free_techs_granted = 0;
    let mut early_war_captures = 0;
    let mut team_vp_turn_75 = BTreeMap::new();
    let mut team_vp_turn_100 = BTreeMap::new();
    let mut telemetry = Collector::new();
    telemetry.observe_initial(&s.state);
    telemetry.proxy = proxy.map(|(seat, version, opening)| {
        let mut game = crate::proxy_benchmark::Game::new(seat, &s.state, version);
        if let Some(opening) = opening {
            game.proxy_version = None;
            game.benchmark_brain = Some(format!("opening:{}", opening.name()));
        }
        game
    });
    let mut vp_race = crate::vp_race::Observer::default();
    let mut panic = None;
    let mut workable_claims_before = 0;
    let mut workable_claims_after = 0;
    let mut primitive_observers: Vec<_> = (0..s.state.players.len())
        .map(|_| crate::primitive_telemetry::Observer::default())
        .collect();
    // Decision time per seat and turn (ms), recorded only in games with a search seat.
    let timed = search.iter().any(Option::is_some);
    let mut decision_ms: Vec<Vec<u32>> = vec![Vec::new(); setup.players.len()];
    // City yields reused across turns by the runner's own view builds (as in `Session`).
    let mut view_cache = legatum_core::ViewCache::default();
    let mut decision_out = decision_trace.map(|path| {
        std::io::BufWriter::new(std::fs::File::create(path).expect("create decision trace"))
    });
    let mut training_out = TRAINING.get().map(|t| {
        std::fs::create_dir_all(&t.dir).expect("create training dir");
        std::io::BufWriter::new(
            std::fs::File::create(t.dir.join(format!("game-{}.jsonl", setup.seed)))
                .expect("create training log"),
        )
    });
    let mut training_tracker = legatum_ai::training::OutcomeTracker::default();
    while !s.state.finished && played < safety_cap {
        let orders = match caught(|| {
            if biased_brains.is_empty() {
                s.ai_orders()
            } else {
                let truth = biased_brains
                    .values()
                    .any(|b| b.wants_true_state())
                    .then(|| std::sync::Arc::new(s.state.clone()));
                // The viewer-independent part of every seat's view, built once per turn.
                let public = legatum_core::public_view(&s.state);
                biased_brains
                    .iter_mut()
                    .filter(|(id, _)| s.state.player(**id).alive)
                    .map(|(id, brain)| {
                        if let Some(truth) = &truth {
                            brain.observe_true_state(truth.clone());
                        }
                        let start = Instant::now();
                        let view = legatum_core::player_view_cached(
                            &s.state,
                            *id,
                            &public,
                            legatum_core::ViewDetail::Decisions,
                            &mut view_cache,
                        );
                        let (orders, diagnosis) = if decision_out.is_some() {
                            brain.orders_with_decision_trace(&view)
                        } else {
                            (brain.orders(&view), None)
                        };
                        if let (Some(out), Some(diagnosis)) = (&mut decision_out, diagnosis) {
                            writeln!(
                                out,
                                "{}",
                                serde_json::json!({
                                    "turn": s.state.turn, "seat": id.0,
                                    "faction": format!("{:?}", s.state.player(*id).faction),
                                    "decisions": diagnosis.decisions,
                                    "passes": diagnosis.passes,
                                    "city_passes": diagnosis.city_passes,
                                })
                            )
                            .expect("write decision trace");
                        }
                        if timed {
                            decision_ms[id.0 as usize].push(start.elapsed().as_millis() as u32);
                        }
                        (*id, orders)
                    })
                    .collect()
            }
        }) {
            Ok(orders) => orders,
            Err(error) => {
                panic = Some(error);
                break;
            }
        };
        let mut orders = orders;
        if let Some(out) = &mut training_out {
            train_turn(&s, &mut orders, &mut training_tracker, out);
        }
        telemetry.observe_decisions(&s.state, &orders);
        for (seat, observer) in primitive_observers.iter_mut().enumerate() {
            observer.before(
                &mut opening_records[seat].primitives,
                &s.state,
                legatum_core::PlayerId(seat as u8),
            );
        }
        let turn = s.state.turn;
        let events = match caught(|| s.step(&orders)) {
            Ok(events) => events,
            Err(error) => {
                panic = Some(error);
                break;
            }
        };
        if let Some(out) = &mut training_out {
            for o in training_tracker.close(&s.state) {
                let _ = writeln!(out, "{}", legatum_ai::training::LogLine::Outcome(o).json());
            }
        }
        workable_claims_before += events
            .iter()
            .filter_map(|event| match event {
                Event::TilesReassigned { tiles, .. } => Some(tiles.len() as u32),
                _ => None,
            })
            .sum::<u32>();
        workable_claims_after += s
            .state
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(i, tile)| {
                let Some(source) = tile.city.and_then(|id| s.state.cities.get(&id)) else {
                    return false;
                };
                let at = s.state.map.hex_at(*i);
                tile.owner == Some(source.owner)
                    && at.distance(source.pos) > legatum_core::data::CITY_WORK_RADIUS
                    && s.state.cities.values().any(|other| {
                        other.owner == source.owner
                            && other.id != source.id
                            && at.distance(other.pos) <= legatum_core::data::CITY_WORK_RADIUS
                    })
            })
            .count() as u32;
        observe_events(
            &s.state,
            turn,
            &events,
            &mut telemetry,
            &mut city_captures,
            &mut athenian_city_captures,
            &mut athenian_free_techs_granted,
            &mut early_war_captures,
            &mut eliminations,
        );
        if let Some(metrics) = &mut telemetry.proxy {
            metrics.observe(turn, &s.state, &events);
        }
        for (seat, metrics) in opening_records.iter_mut().enumerate() {
            metrics.observe(legatum_core::PlayerId(seat as u8), turn, &s.state, &events);
            let id = legatum_core::PlayerId(seat as u8);
            primitive_observers[seat].after(
                &mut metrics.primitives,
                &s.state,
                id,
                turn,
                &events,
                seat_brain(&biased_brains, &s, seat).and_then(|b| b.primitive_coalition_march()),
            );
        }
        played += 1;
        telemetry.observe_resolved_turn(turn, &s.state);
        vp_race.observe_resolved_turn(turn, &s.state);
        if turn == 75 {
            team_vp_turn_75 = s
                .state
                .teams()
                .into_iter()
                .map(|t| (t.0, scoring::team_vp(&s.state, t)))
                .collect();
        }
        if turn == 100 {
            team_vp_turn_100 = s
                .state
                .teams()
                .into_iter()
                .map(|t| (t.0, scoring::team_vp(&s.state, t)))
                .collect();
        }
        if record {
            tape.turns.push(TapeTurn {
                turn,
                orders,
                hash: s.state.hash().to_string(),
            });
        }
    }
    for (seat, metrics) in opening_records.iter_mut().enumerate() {
        metrics.decision_ms = std::mem::take(&mut decision_ms[seat]);
        let id = legatum_core::PlayerId(seat as u8);
        let brain = seat_brain(&biased_brains, &s, seat);
        let phases = brain.map_or(&[][..], |b| b.opening_phase_switches());
        metrics.finish(id, &s.state, phases);
        metrics.opening_handoff = brain.and_then(|b| b.opening_handoff());
        if let Some(brain) = brain {
            metrics.strategy_switches = brain.strategy_switches().to_vec();
            metrics.threat_log = brain.threat_log().to_vec();
            metrics.campaign_events = brain.campaign_events().to_vec();
            metrics.plan_events = brain.plan_events().to_vec();
            metrics.blocked_expenses = brain.blocked_expenses();
            metrics.search_events = brain.search_events().to_vec();
        }
    }
    let mut game_telemetry = telemetry.finish(&s.state, eliminations.len());
    game_telemetry.vp_race = Some(vp_race.finish(&s.state));
    Played {
        opening_records,
        telemetry: game_telemetry,
        state: s.state,
        tape,
        eliminations,
        brains,
        strategies,
        turns: played,
        city_captures,
        athenian_city_captures,
        athenian_free_techs_granted,
        early_war_captures,
        workable_claims_before,
        workable_claims_after,
        team_vp_turn_75,
        team_vp_turn_100,
        archive: s.history,
        panic,
    }
}

/// Build the record from a finished play.
pub fn record(index: usize, p: &Played, wall_ms: u64, keep_hashes: bool) -> GameRecord {
    let st = &p.state;
    let alive_teams: BTreeSet<_> = st
        .players
        .iter()
        .filter(|x| x.alive)
        .map(|x| x.team)
        .collect();
    let outcome = if p.panic.is_some() {
        crate::stats::Outcome::Panic
    } else {
        classify(FinalFacts {
            finished: st.finished,
            winner: st.winner.map(|t| t.0),
            alive_teams: alive_teams.len(),
            winner_hold_complete: crate::stats::winner_hold_complete(st),
            winner_holds_last_capital: crate::stats::winner_holds_last_capital(st),
        })
    };
    let winners: Vec<_> = st
        .players
        .iter()
        .filter(|x| Some(x.team) == st.winner)
        .collect();
    let wonder_sources = |id| {
        scoring::vp(st, id)
            .iter()
            .filter(|(source, _)| matches!(source, legatum_core::VpSource::GreatWork { .. }))
            .map(|(_, vp)| *vp)
            .collect::<Vec<_>>()
    };
    let wonder_stats = WonderStats {
        completions: st
            .history
            .chronicle
            .iter()
            .filter_map(|entry| match entry {
                legatum_core::history::Chronicle::GreatWork {
                    turn,
                    player,
                    building,
                    ..
                } => Some(WonderCompletion {
                    turn: *turn,
                    building: *building,
                    player: player.0,
                    faction: st.player(*player).faction,
                }),
                _ => None,
            })
            .collect(),
        held_by_empire: st
            .players
            .iter()
            .map(|x| wonder_sources(x.id).len() as u32)
            .collect(),
        winner_vp: winners
            .iter()
            .map(|x| wonder_sources(x.id).into_iter().sum::<u32>())
            .sum(),
    };
    GameRecord {
        index,
        seed: st.setup.seed.to_string(),
        outcome,
        victory_type: outcome.victory_type(),
        mirrored: false,
        winner_team: st.winner.map(|t| t.0),
        winner_players: winners.iter().map(|x| x.id.0).collect(),
        winner_factions: winners.iter().map(|x| x.faction).collect(),
        turns: p.turns,
        players: st
            .players
            .iter()
            .map(|x| PlayerRecord {
                opening: p.opening_records[x.id.0 as usize].clone(),
                strategy: p.strategies[x.id.0 as usize],
                archetype: p
                    .telemetry
                    .strategy
                    .as_ref()
                    .and_then(|s| s.empires.get(&x.id.0))
                    .map(legatum_sim::diversity::classify),
                id: x.id.0,
                faction: x.faction,
                team: x.team.0,
                brain: p.brains[x.id.0 as usize].clone(),
                difficulty: st.setup.players[x.id.0 as usize].difficulty,
                alive: x.alive,
                vp: scoring::vp_total(st, x.id),
                cities: st.cities_of(x.id).count() as u32,
                techs: x.techs.len() as u32,
                population: st.cities_of(x.id).map(|c| c.population).sum(),
                gold: Some(x.gold),
            })
            .collect(),
        eliminations: p.eliminations.clone(),
        city_captures: p.city_captures,
        athenian_city_captures: p.athenian_city_captures,
        athenian_free_techs_granted: p.athenian_free_techs_granted,
        early_war_captures: p.early_war_captures,
        wonder_stats: Some(wonder_stats),
        workable_claims_before: p.workable_claims_before,
        workable_claims_after: p.workable_claims_after,
        team_vp_turn_75: p.team_vp_turn_75.clone(),
        team_vp_turn_100: p.team_vp_turn_100.clone(),
        max_era: (p.panic.is_none()).then(|| {
            st.players
                .iter()
                .map(|x| legatum_core::eras::era_of(st, x.id))
                .max()
                .unwrap_or(0)
        }),
        telemetry: Some(p.telemetry.clone()),
        wall_ms,
        hashes: if keep_hashes {
            p.tape.turns.iter().map(|t| t.hash.clone()).collect()
        } else {
            vec![]
        },
        panic: p.panic.clone(),
        determinism: None,
    }
}

/// First divergence between two hash sequences: 0 = initial state, n = after turn n.
pub fn first_divergence(a: &Tape, b: &Tape) -> Option<u32> {
    if a.initial_hash != b.initial_hash {
        return Some(0);
    }
    let n = a.turns.len().max(b.turns.len());
    (0..n)
        .find(|&i| a.turns.get(i).map(|t| &t.hash) != b.turns.get(i).map(|t| &t.hash))
        .map(|i| i as u32 + 1)
}

fn caught<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|p| panic_message(&*p))
}

/// Play game `index` (seed = first_seed + index), catching panics; with determinism, play it
/// a second time and compare. Returns the record and, when recording, the tape.
#[cfg(test)]
pub fn run_game(index: usize, first_seed: u64, opts: &RunOpts) -> (GameRecord, Option<Tape>) {
    let (record, tape, _) = run_game_archived(index, first_seed, opts, None);
    (record, tape)
}

pub fn run_game_archived(
    index: usize,
    first_seed: u64,
    opts: &RunOpts,
    archive: Option<&legatum_archive::Archive>,
) -> (GameRecord, Option<Tape>, Option<String>) {
    run_game_archived_with_trace(index, first_seed, opts, archive, None)
}

pub fn run_game_archived_with_trace(
    index: usize,
    first_seed: u64,
    opts: &RunOpts,
    archive: Option<&legatum_archive::Archive>,
    trace_dir: Option<&Path>,
) -> (GameRecord, Option<Tape>, Option<String>) {
    let (seed, opts) = scheduled_game(index, first_seed, *opts);
    let policy_index = if opts.lobby.mirror { index / 2 } else { index };
    let setup = setup_for_game(seed, policy_index, &opts);
    let mut policies = opts.tuning.seats(policy_index, opts.lobby.players);
    let mirrored = opts.lobby.mirror && index % 2 == 1;
    if mirrored {
        policies.params[..opts.lobby.players].reverse();
        policies.openings.seats[..opts.lobby.players].reverse();
        policies.openings.legacy_hard[..opts.lobby.players].reverse();
        policies.openings.hard_v2[..opts.lobby.players].reverse();
    }
    let proxy = opts
        .proxy
        .map(|p| (p.seat_for(index), p.version, p.opening))
        .or(match opts.tuning.duel_opponent {
            Some(DuelOpponent::Proxy(version)) if opts.tuning.duel_matrix => {
                Some(((1 - index % 2) as u8, version, None))
            }
            _ => None,
        });
    let start = Instant::now();
    let trace_path = trace_dir.map(|dir| dir.join(format!("game-{index:05}-seed-{seed}.jsonl")));
    let first = caught(|| {
        play_with_brains(
            &setup,
            opts.brain,
            proxy,
            opts.record || opts.determinism,
            opts.safety_cap(),
            opts.strategies,
            policies,
            trace_path.as_deref(),
        )
    });
    let wall_ms = start.elapsed().as_millis() as u64;
    let mut first = match first {
        Ok(p) => p,
        Err(msg) => {
            return (
                GameRecord::panicked(index, seed, msg, wall_ms),
                None,
                archive.map(|_| {
                    "game initialization panicked before a history could be recorded".into()
                }),
            );
        }
    };
    let mut rec = record(index, &first, wall_ms, opts.record);
    rec.mirrored = mirrored;
    if opts.determinism {
        rec.determinism = Some(check_determinism(&setup, &opts, proxy, policies, &first));
    }
    let archive_error = archive.and_then(|store| {
        let history = first.archive.as_mut()?;
        history.stop(if first.panic.is_some() {
            "panic"
        } else {
            "stalled"
        });
        store.write(history).err()
    });
    (rec, opts.record.then_some(first.tape), archive_error)
}

fn check_determinism(
    setup: &GameSetup,
    opts: &RunOpts,
    proxy: Option<(u8, u32, Option<legatum_ai::openings::Opening>)>,
    policies: SeatPolicies,
    first: &Played,
) -> DeterminismCheck {
    let fail = |turn, detail: String| DeterminismCheck {
        ok: false,
        first_divergent_turn: turn,
        detail: Some(detail),
    };
    let second = match caught(|| {
        play_with_brains(
            setup,
            opts.brain,
            proxy,
            true,
            opts.safety_cap(),
            opts.strategies,
            policies,
            None,
        )
    }) {
        Ok(p) => p,
        Err(msg) => return fail(None, format!("second play panicked: {msg}")),
    };
    if first.panic.is_some() || second.panic.is_some() {
        return fail(
            None,
            "a play panicked before completing determinism verification".into(),
        );
    }
    if let Some(t) = first_divergence(&first.tape, &second.tape) {
        return fail(Some(t), "per-turn hashes differ between two plays".into());
    }
    // Save/load must keep the hash.
    let saved = legatum_wasm::json::state_to_json(&first.state);
    match legatum_wasm::json::parse_state(&saved) {
        Ok(loaded) if loaded.hash() == first.state.hash() => DeterminismCheck {
            ok: true,
            first_divergent_turn: None,
            detail: None,
        },
        Ok(_) => fail(None, "save/load changed the final hash".into()),
        Err(e) => fail(None, format!("save/load failed: {e}")),
    }
}

/// Every unordered pair plays on the same map/factions twice, with all seat assignments
/// reversed. Twenty games complete one block (10 pairs x 2 orientations). Same-strategy
/// seats remain opponents on their own teams, as in the ordinary eight-empire lobby.
pub fn scheduled_game(index: usize, first_seed: u64, mut opts: RunOpts) -> (u64, RunOpts) {
    if opts.tuning.duel_matrix {
        let (a, b, seat_a) = duel_cell(index);
        let mut factions = std::array::from_fn::<_, 8, _>(|i| legatum_core::Faction::ALL[i]);
        factions[seat_a] = a;
        factions[1 - seat_a] = b;
        opts.lobby.factions = Some(factions);
        return (first_seed.wrapping_add((index / 2) as u64), opts);
    }
    if opts.lobby.mirror {
        if index % 2 == 1 {
            opts.strategies[..opts.lobby.players].reverse();
        }
        return (first_seed.wrapping_add((index / 2) as u64), opts);
    }
    if !opts.strategy_matchups {
        return (first_seed.wrapping_add(index as u64), opts);
    }
    let pairs: Vec<_> = Strategy::ALL
        .iter()
        .enumerate()
        .flat_map(|(i, a)| Strategy::ALL.iter().skip(i + 1).map(move |b| (*a, *b)))
        .collect();
    let (a, b) = pairs[(index / 2) % pairs.len()];
    for (seat, bias) in opts.strategies.iter_mut().enumerate() {
        *bias = Some(if (seat + index).is_multiple_of(2) {
            a
        } else {
            b
        });
    }
    (first_seed.wrapping_add((index / 2) as u64), opts)
}

/// Replay a tape through the core, uninterrupted and across save/load at every turn boundary.
/// `Ok(turns)` when the initial state and every recorded hash match.
pub fn check_tape(json: &str) -> Result<usize, String> {
    let tape = Tape::from_json(json)?;
    caught(|| legatum_wasm::tape::replay_checked(&tape))??;
    Ok(tape.turns.len())
}

#[cfg(test)]
mod tests {
    #[test]
    fn decision_trace_preserves_hard_order_tape() {
        use super::*;
        let mut opts = opening_opts();
        opts.lobby.players = 2;
        opts.lobby.map_size = crate::lobby::MapSize::Small;
        opts.tuning.openings.seats = [legatum_ai::openings::Opening::Default; 8];
        opts.tuning.difficulty = Some(AiDifficulty::Hard);
        opts.safety_cap = Some(8);
        opts.record = true;
        let dir = std::env::temp_dir().join(format!(
            "legatum-decision-trace-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let (off, off_tape, _) = run_game_archived(0, 91917, &opts, None);
        let (on, on_tape, _) = run_game_archived_with_trace(0, 91917, &opts, None, Some(&dir));
        assert!(off.panic.is_none() && on.panic.is_none());
        assert_eq!(off_tape.unwrap().to_json(), on_tape.unwrap().to_json());
        let path = dir.join("game-00000-seed-91917.jsonl");
        let rows = std::fs::read_to_string(&path).unwrap();
        assert!(rows.lines().any(|line| line.contains("\"decisions\"")));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct RunnerPins {
        default_ai_tapes: [u64; 3],
        scripted_hard_city_fire_tape: u64,
    }

    fn runner_pins() -> RunnerPins {
        serde_json::from_str(include_str!("../tests/pins.json"))
            .expect("parse tests/pins.json; run tools/sim/repin.sh")
    }

    #[test]
    fn duel_mirrors_policies_with_fixed_setup_and_replays_both_games() {
        use super::*;
        use legatum_ai::openings::Opening;
        let mut opts = opening_opts();
        opts.lobby = crate::lobby::Config {
            players: 2,
            map_size: crate::lobby::MapSize::Small,
            mirror: true,
            ..Default::default()
        };
        opts.tuning.openings.seats = [Opening::Default; 8];
        opts.tuning.openings.seats[0] = Opening::Rush;
        opts.tuning.openings.seats[1] = Opening::GreedyExpand;
        opts.record = true;
        opts.determinism = true;
        opts.safety_cap = Some(30);
        let (a, at) = run_game(0, u64::MAX, &opts);
        let (b, bt) = run_game(1, u64::MAX, &opts);
        let (at, bt) = (at.unwrap(), bt.unwrap());
        assert_eq!(at.setup, bt.setup);
        assert_eq!(at.initial_hash, bt.initial_hash);
        assert_eq!((at.setup.width, at.setup.height), (40, 26));
        assert_eq!(a.players.len(), 2);
        assert_eq!(b.players.len(), 2);
        assert!(!a.mirrored && b.mirrored);
        for (game, expected) in [
            (&a, [Opening::Rush, Opening::GreedyExpand]),
            (&b, [Opening::GreedyExpand, Opening::Rush]),
        ] {
            assert_eq!(game.panic, None);
            assert!(game.determinism.as_ref().unwrap().ok);
            assert_eq!(game.outcome, crate::stats::Outcome::Stalled);
            assert_eq!(game.victory_type, None);
            for (seat, opening) in game.players.iter().zip(expected) {
                assert_eq!(seat.opening.opening, opening);
                assert!(!seat.opening.won);
                assert!(seat.opening.cities_at_turns["50"].is_none());
            }
        }
        for tape in [at, bt] {
            assert_eq!(check_tape(&tape.to_json()), Ok(30));
        }
        assert_eq!(scheduled_game(2, u64::MAX, opts).0, 0);
    }

    #[test]
    fn lobby_presets_and_seeded_factions_keep_the_default_setup() {
        use super::*;
        use legatum_core::Faction;
        let mut opts = opening_opts();
        let original = setup_for(193, &opts);
        for players in 2..=8 {
            for size in [
                crate::lobby::MapSize::Small,
                crate::lobby::MapSize::Medium,
                crate::lobby::MapSize::Large,
            ] {
                opts.lobby.players = players;
                opts.lobby.map_size = size;
                let setup = setup_for(193, &opts);
                assert_eq!(setup.players, original.players[..players]);
                assert_eq!((setup.width, setup.height), size.dimensions());
                assert_eq!(setup, setup_for(193, &opts));
            }
        }
        opts.lobby = Default::default();
        assert_eq!(original, setup_for(193, &opts));
        opts.lobby.players = 2;
        opts.lobby.factions = Some([Faction::India; 8]);
        assert!(
            setup_for(193, &opts)
                .players
                .iter()
                .all(|p| p.faction == Faction::India)
        );
    }

    fn opening_opts() -> super::RunOpts {
        use legatum_ai::openings::Opening;
        super::RunOpts {
            lobby: Default::default(),
            rule_experiments: Default::default(),
            barbarians: true,
            safety_cap: None,
            turn_limit: None,
            brain: super::BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            record: false,
            determinism: false,
            tuning: super::Tuning {
                openings: crate::openings::Config {
                    ai_primitives: false,
                    seats: [
                        Opening::Default,
                        Opening::Rush,
                        Opening::GreedyExpand,
                        Opening::Snowball,
                        Opening::Snowball,
                        Opening::Economic,
                        Opening::Economic,
                        Opening::Snowball,
                    ],
                    policies: None,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    #[test]
    fn explicit_rome_roster_rotates_through_every_seat() {
        use legatum_core::Faction;
        let mut opts = opening_opts();
        opts.lobby.factions = Some([
            Faction::Rome,
            Faction::Switzerland,
            Faction::India,
            Faction::Byzantium,
            Faction::Germany,
            Faction::Celts,
            Faction::Mongols,
            Faction::Sparta,
        ]);
        opts.lobby.rotate_factions = true;
        for seat in 0..8 {
            let setup = setup_for_game(42, seat, &opts);
            assert_eq!(setup.players[seat].faction, Faction::Rome);
        }
    }

    #[test]
    fn hard_v2_rotation_plays_one_v2_seat_against_legacy_hard() {
        let mut opts = opening_opts();
        opts.tuning.openings = Default::default();
        opts.tuning.openings.hard_v2_rotation = true;
        opts.tuning.difficulty = Some(legatum_core::AiDifficulty::Hard);
        opts.turn_limit = Some(30);
        for i in 0..3 {
            let (r, _) = super::run_game(i, 9_600_001, &opts);
            for p in &r.players {
                let v2 = legatum_ai::hard_v2::draw_opening(legatum_wasm::session::brain_seed(
                    r.seed.parse().unwrap(),
                    legatum_core::PlayerId(p.id),
                ));
                if p.id as usize == i % 8 && v2 != legatum_ai::openings::Opening::Default {
                    assert_eq!(p.brain, format!("hard-v2:{}", v2.name()));
                    assert_eq!(p.opening.opening, v2);
                    assert!(p.opening.ai_primitives);
                } else {
                    assert_eq!(p.brain, "scripted", "seat {} of game {i}", p.id);
                    assert_eq!(p.opening.opening, legatum_ai::openings::Opening::Default);
                }
            }
        }
        // Without options every Hard seat is the scripted Hard, as in the app.
        opts.tuning.openings = Default::default();
        let (r, _) = super::run_game(0, 9_600_001, &opts);
        for p in &r.players {
            assert_eq!(p.brain, "scripted");
            assert_eq!(p.opening.opening, legatum_ai::openings::Opening::Default);
        }
        // The hard-v2-ai setup switch (--hard-v2) puts every Hard seat on Hard v2, except
        // --legacy-hard seats.
        opts.rule_experiments.hard_v2_ai = true;
        opts.tuning.openings.legacy_hard[3] = true;
        let (r, _) = super::run_game(0, 9_600_001, &opts);
        for p in &r.players {
            let v2 = legatum_ai::hard_v2::draw_opening(legatum_wasm::session::brain_seed(
                r.seed.parse().unwrap(),
                legatum_core::PlayerId(p.id),
            ));
            if p.id == 3 {
                assert_eq!(p.brain, "scripted");
            } else {
                assert_eq!(p.opening.opening, v2);
            }
        }
    }

    #[test]
    fn handoff_is_deterministic_and_keeps_challenger_proxy_and_legacy_seats() {
        use legatum_ai::openings::Opening;
        let mut opts = opening_opts();
        opts.tuning.openings = crate::openings::Config {
            hard_v2_handoff: true,
            challenger: Some(Opening::Snowball),
            ..Default::default()
        };
        opts.tuning.openings.legacy_hard[3] = true;
        opts.tuning.difficulty = Some(AiDifficulty::Hard);
        opts.turn_limit = Some(85);
        opts.determinism = true;
        for proxy in [false, true] {
            if proxy {
                opts.tuning.openings.challenger = None;
                opts.proxy = Some(crate::proxy_benchmark::Config {
                    seat: 0,
                    rotate: true,
                    difficulty: AiDifficulty::Hard,
                    start: crate::proxy_benchmark::ProxyStart::Peace,
                    version: legatum_ai::PROXY_VERSION,
                    opening: None,
                });
            }
            let (r, _) = super::run_game(0, 3_500_001, &opts);
            assert_eq!(r.panic, None);
            assert!(r.determinism.as_ref().unwrap().ok);
            let mut handed_off = 0;
            for p in &r.players {
                if p.id == 0 {
                    assert_eq!(p.brain, if proxy { "proxy" } else { "opening:Snowball" });
                    assert_eq!(p.opening.opening_handoff, None);
                } else if p.id == 3 || p.opening.opening == Opening::Default {
                    assert_eq!(p.brain, "scripted");
                    assert_eq!(p.opening.opening_handoff, None);
                } else {
                    assert_eq!(
                        p.brain,
                        format!("hard-v2-handoff:{}", p.opening.opening.name())
                    );
                    // An empire eliminated before the turn cap never hands off.
                    if let Some(h) = p.opening.opening_handoff {
                        assert!((2..=80).contains(&h.turn));
                        handed_off += 1;
                    } else {
                        assert!(
                            !p.alive,
                            "a surviving hand-off seat must hand off by turn 80"
                        );
                    }
                }
            }
            assert!(handed_off > 0);
        }
    }

    #[test]
    fn challenger_rotation_keeps_the_tuned_policy_against_both_tables() {
        use legatum_ai::openings::Opening;
        for challenger in [Opening::GreedyExpand, Opening::Snowball, Opening::Rush] {
            let mut opts = opening_opts();
            opts.tuning.openings = crate::openings::Config {
                challenger: Some(challenger),
                ..Default::default()
            };
            opts.tuning.difficulty = Some(legatum_core::AiDifficulty::Hard);
            opts.turn_limit = Some(1);
            for i in 0..8 {
                let seats = opts.tuning.seats(i, 8);
                assert_eq!(
                    seats.openings.policy(i).genome(),
                    legatum_ai::hard_v2::policy(challenger).genome()
                );
                for v2 in [false, true] {
                    opts.rule_experiments.hard_v2_ai = v2;
                    opts.tuning.openings.legacy_hard = [!v2; 8];
                    let (r, _) = super::run_game(i, 3_500_001, &opts);
                    assert_eq!(r.panic, None);
                    assert_eq!(r.players.len(), 8);
                    for p in &r.players {
                        assert_eq!(p.difficulty, legatum_core::AiDifficulty::Hard);
                        let expected = if p.id as usize == i {
                            challenger
                        } else if v2 {
                            legatum_ai::hard_v2::draw_opening(legatum_wasm::session::brain_seed(
                                r.seed.parse().unwrap(),
                                legatum_core::PlayerId(p.id),
                            ))
                        } else {
                            Opening::Default
                        };
                        assert_eq!(p.opening.opening, expected);
                        assert_eq!(p.opening.ai_primitives, expected != Opening::Default);
                        let brain = if p.id as usize == i {
                            format!("opening:{}", challenger.name())
                        } else if expected == Opening::Default {
                            "scripted".into()
                        } else {
                            format!("hard-v2:{}", expected.name())
                        };
                        assert_eq!(p.brain, brain);
                    }
                }
            }
        }
    }

    #[test]
    fn opening_league_twenty_standard_games_finish_without_panics_or_stalls() {
        let opts = opening_opts();
        // The twenty games are independent and seeded by index, so they play side by side (this one test was the
        // whole critical path of the gate's test step: about 280 s run one after another). The checks below run
        // over the finished records in index order, so a failure reads the same as before.
        let workers = std::thread::available_parallelism()
            .map_or(4, |n| n.get())
            .clamp(2, 10);
        let mut games: Vec<(usize, super::GameRecord)> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|w| {
                    let opts = &opts;
                    scope.spawn(move || {
                        (w..20)
                            .step_by(workers)
                            .map(|i| (i, super::run_game(i, 8_120_001, opts).0))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect()
        });
        games.sort_by_key(|(i, _)| *i);
        let mut stalls = Vec::new();
        for (_, r) in games {
            assert_eq!(r.panic, None, "seed {}", r.seed);
            assert!(
                !matches!(r.outcome, crate::stats::Outcome::Panic),
                "seed {}",
                r.seed
            );
            // The Defend hysteresis (2026-09-30) moved seed 8120020 into the known late-game
            // title deadlock (600-turn cap with all eight empires alive; Standard validation
            // shows no more stalls than old Hard). One capped game in twenty is tolerated;
            // two means a real regression.
            if matches!(r.outcome, crate::stats::Outcome::Stalled) {
                assert!(r.players.iter().all(|p| p.alive), "seed {}", r.seed);
                stalls.push(r.seed.clone());
            }
            for p in &r.players {
                assert_eq!(p.opening.opening, opts.tuning.openings.seats[p.id as usize]);
                assert!((1..=8).contains(&p.opening.rank));
                assert_eq!(p.opening.won, r.winner_players.contains(&p.id));
                if r.turns >= 25 {
                    assert!(p.opening.cities_at_turns["25"].is_some());
                }
            }
        }
        assert!(stalls.len() <= 1, "stalled seeds: {stalls:?}");
    }

    #[test]
    fn opening_flags_genomes_and_every_save_boundary_repeat_exactly() {
        use legatum_ai::openings::{Opening, OpeningPolicy};
        let mut opts = opening_opts();
        opts.tuning.openings.ai_primitives = true;
        opts.rule_experiments.apply("start-settlers=1").unwrap();
        let mut genome = OpeningPolicy::builtin(Opening::Rush).genome();
        genome[0] = 1;
        let i = OpeningPolicy::genes()
            .iter()
            .position(|g| g.name == "primitives.scout_upkeep_minimum")
            .unwrap();
        genome[i] = 2;
        opts.tuning.openings.policies =
            Some(Box::leak(Box::new(std::collections::BTreeMap::from([(
                Opening::Rush,
                OpeningPolicy::from_genome(Opening::Rush, &genome).unwrap(),
            )]))));
        opts.record = true;
        opts.determinism = true;
        opts.turn_limit = Some(100);
        let (record, tape) = super::run_game(0, 8_220_001, &opts);
        assert_eq!(record.panic, None);
        assert!(
            record.determinism.as_ref().is_some_and(|d| d.ok),
            "{:?}",
            record.determinism
        );
        assert_eq!(
            super::check_tape(&tape.unwrap().to_json()),
            Ok(record.turns as usize)
        );
    }

    /// Pins default AI orders as well as state hashes; fixtures also pin core replay.
    /// Repinned for rules 45: the already-founded capital starts production on turn 1.
    #[test]
    fn default_ai_tapes_are_pinned() {
        let expected = runner_pins().default_ai_tapes;
        let hashes = default_ai_tape_hashes();
        assert_eq!(
            hashes, expected,
            "runner tape pins differ; run tools/sim/repin.sh"
        );
    }

    fn default_ai_tape_hashes() -> [u64; 3] {
        let mut opts = opening_opts();
        opts.tuning = Default::default();
        opts.brain = super::BrainMode::Mixed;
        opts.turn_limit = Some(80);
        opts.record = true;
        std::array::from_fn(|i| {
            let (_, tape) = super::run_game(i, u64::MAX - 2, &opts);
            legatum_core::state::fnv1a(tape.unwrap().to_json().as_bytes())
        })
    }

    #[test]
    fn scripted_hard_city_fire_tape_is_pinned() {
        // Rules 45 starts every capital's output one turn earlier; repin with the opening.
        let expected = runner_pins().scripted_hard_city_fire_tape;
        let hash = scripted_hard_city_fire_tape_hash();
        assert_eq!(
            hash, expected,
            "runner tape pin differs; run tools/sim/repin.sh"
        );
    }

    fn scripted_hard_city_fire_tape_hash() -> u64 {
        let mut opts = opening_opts();
        opts.tuning = super::Tuning {
            difficulty: Some(legatum_core::AiDifficulty::Hard),
            ..Default::default()
        };
        opts.turn_limit = Some(160);
        opts.record = true;
        let (record, tape) = super::run_game(0, 3_200_001, &opts);
        let tape = tape.unwrap().to_json();
        assert_eq!(super::check_tape(&tape), Ok(record.turns as usize));
        legatum_core::state::fnv1a(tape.as_bytes())
    }

    #[test]
    fn search_duel_seat_replays_exactly_and_logs_its_searches() {
        let mut opts = opening_opts();
        opts.lobby.players = 2;
        opts.lobby.map_size = crate::lobby::MapSize::Small;
        opts.tuning = super::Tuning {
            difficulty: Some(legatum_core::AiDifficulty::Hard),
            search: Some(legatum_ai::search::SearchConfig::parse("k=2,h=6,d=2,t,p").unwrap()),
            search_rotation: true,
            duel_matrix: true,
            ..Default::default()
        };
        opts.turn_limit = Some(60);
        opts.determinism = true;
        // Game 3 is pair (Switzerland, India) with side A (the search seat) in seat 1.
        let (record, _) = super::run_game(3, 7_100_001, &opts);
        assert_eq!(record.panic, None);
        assert!(
            record.determinism.as_ref().unwrap().ok,
            "{:?}",
            record.determinism
        );
        let seat = record.players.iter().find(|p| p.id == 1).unwrap();
        assert!(seat.brain.starts_with("scripted+search"), "{}", seat.brain);
        assert_eq!(
            (record.players[1].faction, record.players[0].faction),
            (legatum_core::Faction::ALL[0], legatum_core::Faction::ALL[1])
        );
    }

    #[test]
    fn parameterized_search_seat_keeps_search_and_uses_macro_gene() {
        for (mode, learned_opening) in [(1, true), (2, false), (3, true)] {
            for players in [2, 4] {
                let mut opts = opening_opts();
                opts.lobby.players = players;
                opts.lobby.map_size = crate::lobby::MapSize::Small;
                opts.turn_limit = Some(4);
                let mut params = legatum_ai::AiParams::builtin();
                params.decisions.learned_macro = mode;
                let params = Box::leak(Box::new(params));
                opts.tuning = super::Tuning {
                    difficulty: Some(legatum_core::AiDifficulty::Hard),
                    params: Some(params),
                    search: Some(legatum_ai::search::SearchConfig::parse("k=2,h=6,d=2").unwrap()),
                    search_rotation: true,
                    duel_matrix: players == 2,
                    ..Default::default()
                };
                let (record, _) = super::run_game(0, 7_100_001, &opts);
                let seat = record.players.iter().find(|p| p.id == 0).unwrap();
                assert!(seat.brain.starts_with("scripted+search"), "{}", seat.brain);
                assert_eq!(
                    seat.opening
                        .search_events
                        .iter()
                        .any(|e| e.trigger == "learned:opening"),
                    learned_opening && players >= 4,
                    "mode {mode}, players {players}"
                );
            }
        }
    }

    #[test]
    fn duel_opponent_names_an_opening_or_a_proxy_version() {
        use super::DuelOpponent;
        assert_eq!(
            "rush".parse(),
            Ok(DuelOpponent::Opening(legatum_ai::openings::Opening::Rush))
        );
        assert_eq!("proxy6".parse(), Ok(DuelOpponent::Proxy(6)));
        assert_eq!("Proxy5".parse(), Ok(DuelOpponent::Proxy(5)));
        assert_eq!("proxy7".parse(), Ok(DuelOpponent::Proxy(7)));
        assert_eq!("proxy8o".parse(), Ok(DuelOpponent::Proxy(8)));
        assert_eq!("proxy8g".parse(), Ok(DuelOpponent::Proxy(9)));
        assert!("proxy4".parse::<DuelOpponent>().is_err());
        assert!("nonsense".parse::<DuelOpponent>().is_err());
    }

    #[test]
    fn duel_proxy_plays_side_b_and_replays_exactly() {
        let mut opts = opening_opts();
        opts.lobby.players = 2;
        opts.lobby.map_size = crate::lobby::MapSize::Small;
        opts.tuning = super::Tuning {
            difficulty: Some(legatum_core::AiDifficulty::Hard),
            duel_matrix: true,
            duel_opponent: Some(super::DuelOpponent::Proxy(6)),
            ..Default::default()
        };
        opts.turn_limit = Some(60);
        opts.determinism = true;
        // Game 3: side A in seat 1, so the proxy takes seat 0; game 2 the other way round.
        for (index, proxy_seat) in [(3, 0u8), (2, 1u8)] {
            let (record, _) = super::run_game(index, 7_100_001, &opts);
            assert_eq!(record.panic, None);
            assert!(
                record.determinism.as_ref().unwrap().ok,
                "{:?}",
                record.determinism
            );
            for p in &record.players {
                assert_eq!(p.brain == "proxy", p.id == proxy_seat, "{}", p.brain);
            }
            let proxy = record.telemetry.as_ref().unwrap().proxy.as_ref().unwrap();
            assert_eq!(proxy.seat, proxy_seat);
        }
    }

    /// Run only through tools/sim/repin.sh after reviewing the intended engine change.
    #[test]
    #[ignore]
    fn repin_runner_tapes() {
        let pins = RunnerPins {
            default_ai_tapes: default_ai_tape_hashes(),
            scripted_hard_city_fire_tape: scripted_hard_city_fire_tape_hash(),
        };
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/pins.json");
        let json = format!("{}\n", serde_json::to_string_pretty(&pins).unwrap());
        if std::fs::read_to_string(path).ok().as_deref() != Some(json.as_str()) {
            std::fs::write(path, json).expect("write runner tape pins");
        }
    }

    use super::*;

    #[test]
    fn constructed_resolution_events_are_counted_once_and_separately() {
        use legatum_core::events::Defender;
        use legatum_core::{Breakdown, CityId, Hex, PlayerId, UnitId, UnitKind};
        let battle = |ranged, meeting| Event::Battle {
            attacker: UnitId(0).into(),
            attacker_owner: PlayerId(0),
            attacker_at: None,
            defender: Defender::Unit {
                unit: UnitId(1),
                owner: PlayerId(1),
                unit_kind: UnitKind::Scout,
            },
            at: Hex::new(0, 0),
            attacker_strength: Breakdown::default(),
            defender_strength: Breakdown::default(),
            damage_to_attacker: 0,
            damage_to_defender: 20,
            attacker_roll: 100,
            defender_roll: 100,
            attacker_hp_after: 100,
            defender_hp_after: 80,
            ranged,
            meeting,
            air_strike: false,
            defender_leaving: false,
        };
        let events = vec![
            battle(false, false),
            battle(true, false),
            battle(false, true),
            Event::CityCaptured {
                city: CityId(2),
                from: PlayerId(1),
                to: PlayerId(0),
                bounty: None,
            },
            Event::CityCaptured {
                city: CityId(2),
                from: PlayerId(0),
                to: PlayerId(1),
                bounty: None,
            },
            Event::SettlerCaptured {
                unit: UnitId(3),
                from: PlayerId(1),
                to: PlayerId(0),
                at: Hex::new(0, 0),
            },
            Event::PlayerEliminated {
                player: PlayerId(1),
            },
            Event::UnitDestroyed {
                unit: UnitId(1),
                owner: PlayerId(1),
                unit_kind: UnitKind::Scout,
                at: Hex::new(0, 0),
            },
            Event::OrderFailed {
                player: PlayerId(0),
                reason: "attack failed".into(),
            },
            Event::OrderFailed {
                player: PlayerId(0),
                reason: "Ranged attack by unit 1 failed: No line of sight: mountain".into(),
            },
        ];
        let (mut metrics, mut cities, mut eliminations) = (Collector::default(), 0, Vec::new());
        let mut early = 0;
        let mut athenian = 0;
        let mut free_techs = 0;
        let state = legatum_core::new_game(&GameSetup::all_ai(1));
        observe_events(
            &state,
            12,
            &events,
            &mut metrics,
            &mut cities,
            &mut athenian,
            &mut free_techs,
            &mut early,
            &mut eliminations,
        );
        assert_eq!(early, 2);
        let metrics = metrics.finish(&state, eliminations.len());
        assert_eq!(metrics.investments[&0].blocked_ranged_attacks, 1);
        assert_eq!(
            (metrics.battles, cities, metrics.settler_captures),
            (3, 2, 1)
        );
        assert_eq!(
            eliminations,
            vec![Elimination {
                player: 1,
                turn: 12,
                by: Some(0),
            }]
        );
        let mut summary = crate::telemetry::TelemetrySummary::default();
        summary.observe(crate::telemetry::Sample {
            telemetry: Some(&metrics),
            city_captures: cities,
            eliminations: eliminations.len(),
            finished: true,
            winner: None,
            winner_players: &[],
        });
        assert_eq!(summary.events.battles.total, 3);
        assert_eq!(summary.events.city_captures.total, 2);
        assert_eq!(summary.events.settler_captures.total, 1);
        assert_eq!(metrics.elimination_count, 1);
        assert_eq!(summary.events.eliminations.total, 1);
        assert_eq!(summary.events.city_captures.observed_games, 1);
        assert_eq!(summary.events.city_captures.games_with_events, 1);
    }

    fn tape(hashes: &[&str]) -> Tape {
        Tape {
            version: legatum_wasm::tape::TAPE_VERSION,
            setup: GameSetup::all_ai(1),
            initial_hash: 5,
            turns: hashes
                .iter()
                .enumerate()
                .map(|(i, h)| TapeTurn {
                    turn: i as u32 + 1,
                    orders: Default::default(),
                    hash: h.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn divergence_detection() {
        assert_eq!(
            first_divergence(&tape(&["1", "2"]), &tape(&["1", "2"])),
            None
        );
        assert_eq!(
            first_divergence(&tape(&["1", "2"]), &tape(&["1", "3"])),
            Some(2)
        );
        assert_eq!(first_divergence(&tape(&["1", "2"]), &tape(&["1"])), Some(2));
        let mut b = tape(&["1"]);
        b.initial_hash = 6;
        assert_eq!(first_divergence(&tape(&["1"]), &b), Some(0));
    }

    #[test]
    fn mixed_brains_alternate() {
        use legatum_core::PlayerId;
        assert_eq!(BrainMode::Mixed.brain_for(PlayerId(0)), BrainKind::Scripted);
        assert_eq!(BrainMode::Mixed.brain_for(PlayerId(3)), BrainKind::Random);
        assert_eq!(BrainMode::Random.brain_for(PlayerId(0)), BrainKind::Random);
    }

    #[test]
    fn turn_limit_override() {
        let opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: Some(20),
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            rule_experiments: Default::default(),
            barbarians: false,
            record: false,
            determinism: false,
            tuning: Tuning::default(),
        };
        assert_eq!(setup_for(9, &opts).turn_limit, Some(20));
        assert_eq!(setup_for(9, &opts).seed, 9);
    }

    #[test]
    fn tournament_balances_every_pair_faction_and_start_with_wrapping_seeds() {
        let opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: None,
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: true,
            rule_experiments: Default::default(),
            barbarians: false,
            record: false,
            determinism: false,
            tuning: Tuning::default(),
        };
        let mut pairs = BTreeSet::new();
        for i in (0..30).step_by(2) {
            let (seed_a, a) = scheduled_game(i, u64::MAX - 1, opts);
            let (seed_b, b) = scheduled_game(i + 1, u64::MAX - 1, opts);
            assert_eq!(seed_a, seed_b);
            assert_eq!(seed_a, (u64::MAX - 1).wrapping_add((i / 2) as u64));
            assert_eq!(setup_for(seed_a, &a), setup_for(seed_b, &b));
            pairs.insert((a.strategies[0].unwrap(), a.strategies[1].unwrap()));
            for seat in 0..8 {
                assert_ne!(a.strategies[seat], b.strategies[seat]);
                assert_eq!(a.strategies[seat], b.strategies[(seat + 1) % 8]);
            }
        }
        assert_eq!(pairs.len(), 10);
    }

    #[test]
    fn difficulty_rotation_balances_each_faction_and_preserves_starts() {
        let mut opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: None,
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            rule_experiments: Default::default(),
            barbarians: false,
            record: false,
            determinism: false,
            tuning: Tuning::default(),
        };
        for seed in [930001, 930002, u64::MAX] {
            opts.difficulty_rotation = None;
            let normal = setup_for(seed, &opts);
            assert!(
                normal
                    .players
                    .iter()
                    .all(|p| p.difficulty == AiDifficulty::Normal)
            );
            let mut exposure = BTreeMap::<_, BTreeSet<_>>::new();
            for offset in 0..3 {
                opts.difficulty_rotation = Some(offset);
                let mut rotated = setup_for(seed, &opts);
                assert_eq!(
                    legatum_core::mapgen::generate(&rotated),
                    legatum_core::mapgen::generate(&normal)
                );
                for (seat, p) in rotated.players.iter_mut().enumerate() {
                    assert_eq!(
                        p.difficulty,
                        [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard]
                            [(seat + usize::from(offset)) % 3]
                    );
                    exposure.entry(p.faction).or_default().insert(p.difficulty);
                    p.difficulty = AiDifficulty::Normal;
                }
                assert_eq!(rotated, normal, "only difficulty may change");
            }
            assert_eq!(exposure.len(), 8);
            assert!(exposure.values().all(|ds| ds.len() == 3));
        }
    }

    #[test]
    fn safety_cap_stalls_without_awarding_victory_and_replays_deterministically() {
        let mut opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: None,
            brain: BrainMode::Mixed,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            rule_experiments: Default::default(),
            barbarians: false,
            record: true,
            determinism: true,
            tuning: Tuning::default(),
        };
        assert_eq!(opts.safety_cap(), 1200);
        let setup = setup_for(1, &opts);
        assert_eq!(setup.turn_limit, None);
        opts.safety_cap = Some(2);
        let (record, tape) = run_game(0, 1, &opts);
        assert_eq!(record.outcome, crate::stats::Outcome::Stalled);
        assert_eq!((record.turns, record.winner_team), (2, None));
        assert!(record.determinism.unwrap().ok);
        let tape = tape.unwrap();
        assert_eq!(check_tape(&tape.to_json()), Ok(2));
        let mut final_state = tape.initial_state();
        for turn in &tape.turns {
            final_state = legatum_core::resolve_turn(&final_state, &turn.orders).0;
        }
        for player in &record.players {
            assert_eq!(
                player.gold,
                Some(final_state.player(legatum_core::PlayerId(player.id)).gold)
            );
        }

        opts.turn_limit = Some(1);
        let (record, _) = run_game(0, 1, &opts);
        assert_eq!(record.turns, 1);
        assert!(matches!(
            record.outcome,
            crate::stats::Outcome::TurnLimit | crate::stats::Outcome::Draw
        ));
    }

    #[test]
    fn empty_params_reproduce_stock_tapes() {
        let empty = Box::leak(Box::new(
            serde_json::from_str::<legatum_ai::AiParams>(r#"{"schema":1,"name":"empty"}"#).unwrap(),
        ));
        let builtin = Box::leak(Box::new(legatum_ai::AiParams::builtin()));
        for difficulty in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
            let mut opts = RunOpts {
                lobby: Default::default(),
                rule_experiments: Default::default(),
                barbarians: false,
                safety_cap: None,
                turn_limit: Some(80),
                brain: BrainMode::Scripted,
                difficulty_rotation: None,
                proxy: None,
                strategies: [None; 8],
                strategy_matchups: false,
                record: true,
                determinism: false,
                tuning: Tuning {
                    difficulty: Some(difficulty),
                    ..Tuning::default()
                },
            };
            // --ai-params tunes the scripted brain, so Hard compares with pre-v2 Hard.
            opts.tuning.openings.legacy_hard = [true; 8];
            let (_, tape) = run_game(0, 720001, &opts);
            let stock = tape.unwrap().to_json();
            for params in [&*empty, &*builtin] {
                opts.tuning.params = Some(params);
                let (_, tape) = run_game(0, 720001, &opts);
                assert_eq!(stock, tape.unwrap().to_json(), "{difficulty:?}");
            }
        }
    }

    #[test]
    fn decision_genes_change_tapes_without_personality_changes() {
        let mut params = legatum_ai::AiParams::builtin();
        params.decisions.production_settler_percent = 40;
        params.decisions.research_military_percent = 200;
        params.decisions.research_economy_percent = 40;
        params.validate().unwrap();
        let mut opts = RunOpts {
            lobby: Default::default(),
            rule_experiments: Default::default(),
            barbarians: false,
            safety_cap: None,
            turn_limit: Some(60),
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            record: true,
            determinism: true,
            tuning: Tuning {
                difficulty: Some(AiDifficulty::Hard),
                ..Tuning::default()
            },
        };
        let (_, stock) = run_game(0, 720002, &opts);
        opts.tuning.params = Some(Box::leak(Box::new(params)));
        let (record, tuned) = run_game(0, 720002, &opts);
        assert!(record.determinism.unwrap().ok);
        let tape = tuned.unwrap();
        assert_ne!(stock.unwrap().turns, tape.turns);
        assert_eq!(check_tape(&tape.to_json()), Ok(tape.turns.len()));
    }

    #[test]
    fn tuned_seats_alternate_by_game_and_replay_deterministically() {
        use legatum_ai::{AiParams, KnobValues};
        let knobs = |v| KnobValues {
            expansion: v,
            military: v,
            science: v,
            economy: v,
            aggression: v,
        };
        let params = |name: &str, v| -> &'static AiParams {
            Box::leak(Box::new(AiParams {
                schema: 1,
                name: name.into(),
                factions: legatum_core::Faction::ALL
                    .into_iter()
                    .map(|f| (f, knobs(v)))
                    .collect(),
                ..Default::default()
            }))
        };
        let opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: Some(25),
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            rule_experiments: Default::default(),
            barbarians: false,
            record: true,
            determinism: true,
            tuning: Tuning {
                params: Some(params("cand", 90)),
                opponent: Some(params("league", 10)),
                openings: Default::default(),
                difficulty: Some(AiDifficulty::Hard),
                ..Default::default()
            },
        };
        let (rec, tape) = run_game(1, 4242, &opts);
        assert_eq!(rec.panic, None);
        assert!(
            rec.determinism.as_ref().unwrap().ok,
            "{:?}",
            rec.determinism
        );
        for p in &rec.players {
            let want = if (p.id as usize + 1) % 2 == 1 {
                "scripted@league"
            } else {
                "scripted@cand"
            };
            assert_eq!(p.brain, want);
            assert_eq!(p.difficulty, AiDifficulty::Hard);
        }
        let tape = tape.unwrap();
        assert_eq!(check_tape(&tape.to_json()), Ok(tape.turns.len()));
        // Different decisions from the stock AI on the same map: the knobs really reach the brains.
        let stock = RunOpts {
            tuning: Tuning {
                difficulty: Some(AiDifficulty::Hard),
                ..Tuning::default()
            },
            ..opts
        };
        let (_, stock_tape) = run_game(1, 4242, &stock);
        assert_ne!(stock_tape.unwrap().turns, tape.turns);
    }

    #[test]
    fn panics_become_records() {
        // new_game is unimplemented on this branch; after the merge this plays a real game.
        let opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: Some(3),
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: None,
            strategies: [None; 8],
            strategy_matchups: false,
            rule_experiments: Default::default(),
            barbarians: false,
            record: true,
            determinism: true,
            tuning: Tuning::default(),
        };
        let (rec, tape) = run_game(2, 10, &opts);
        assert_eq!(rec.seed, "12");
        if rec.outcome == crate::stats::Outcome::Panic {
            assert!(rec.panic.is_some());
            assert!(tape.is_none());
        } else {
            assert!(rec.turns <= 4);
            assert!(rec.determinism.unwrap().ok);
            assert_eq!(tape.unwrap().turns.len(), rec.hashes.len());
        }
    }

    #[test]
    fn short_game_is_deterministic_and_tape_replays() {
        for rule_experiments in [
            Default::default(),
            legatum_core::experiments::RuleExperiments {
                settler_escalation: true,
                city_research: true,
                city_upkeep: true,
                expansion_coalitions: true,
                no_purchase_under_threat: true,
                frontier_cities: true,
                market_plunder: true,
                wall_masonry: true,
                ..Default::default()
            },
        ] {
            let opts = RunOpts {
                lobby: Default::default(),
                barbarians: false,
                safety_cap: None,
                turn_limit: Some(30),
                brain: BrainMode::Mixed,
                difficulty_rotation: None,
                proxy: None,
                strategies: [None; 8],
                strategy_matchups: false,
                rule_experiments,
                record: true,
                determinism: true,
                tuning: Tuning::default(),
            };
            let (rec, tape) = run_game(0, 12345, &opts);
            assert_eq!(rec.panic, None);
            assert!(
                rec.determinism.as_ref().unwrap().ok,
                "{:?}",
                rec.determinism
            );
            let tape = tape.unwrap();
            let decoded = Tape::from_json(&tape.to_json()).unwrap();
            assert_eq!(decoded.setup.rule_experiments, rule_experiments);
            assert_eq!(check_tape(&tape.to_json()), Ok(tape.turns.len()));
        }
    }

    #[test]
    fn proxy_plays_complete_games_without_panics_or_stalls_and_repeats_hashes() {
        let opts = RunOpts {
            lobby: Default::default(),
            safety_cap: None,
            turn_limit: None,
            brain: BrainMode::Scripted,
            difficulty_rotation: None,
            proxy: Some(crate::proxy_benchmark::Config {
                seat: 0,
                rotate: true,
                difficulty: AiDifficulty::Hard,
                start: crate::proxy_benchmark::ProxyStart::War,
                version: legatum_ai::PROXY_VERSION,
                opening: None,
            }),
            strategies: [None; 8],
            strategy_matchups: false,
            rule_experiments: Default::default(),
            barbarians: false,
            record: true,
            determinism: true,
            tuning: Tuning::default(),
        };
        let (rec, _) = run_game(0, 700_001, &opts);
        assert_eq!(rec.panic, None);
        assert!(!matches!(
            rec.outcome,
            crate::stats::Outcome::Stalled | crate::stats::Outcome::Panic
        ));
        assert!(rec.determinism.as_ref().is_some_and(|d| d.ok));
        let proxy = rec.telemetry.as_ref().unwrap().proxy.as_ref().unwrap();
        assert_eq!(proxy.seat, 0);
        assert_eq!(rec.players.iter().filter(|p| p.brain == "proxy").count(), 1);
        // The seven Hard opponents are Hard v2 (or pre-v2 Hard when one draws Default).
        assert_eq!(
            rec.players
                .iter()
                .filter(|p| p.brain == "scripted" || p.brain.starts_with("hard-v2:"))
                .count(),
            7
        );
    }
}
