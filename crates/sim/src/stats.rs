//! Per-game records, outcome classification and run-level aggregation. Floats are fine here:
//! this is reporting, not game logic.

use crate::hold_telemetry::{HoldSummary, TurnStats};
use crate::telemetry::{self, GameTelemetry, Sample, TelemetrySummary};
use legatum_core::{AiDifficulty, Faction, GameSetup};
use legatum_sim::diversity::{
    self, Archetype, DiversitySummary, EmpireResult, Guardrail, GuardrailConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// A team held its VP target for the full hold (`data::VP_HOLD_TURNS` turn ends).
    Vp,
    /// Last team standing.
    Conquest,
    /// The only team still holding an original capital of its own (rules 37).
    Capitals,
    /// Turn limit reached; highest VP won.
    TurnLimit,
    /// Finished with no winner.
    Draw,
    /// The runner stopped it: the core never marked it finished.
    #[serde(alias = "unfinished")]
    Stalled,
    /// The game panicked.
    Panic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VictoryType {
    Vp,
    Elimination,
    Capitals,
    TurnLimit,
}

impl Outcome {
    pub fn victory_type(self) -> Option<VictoryType> {
        match self {
            Self::Vp => Some(VictoryType::Vp),
            Self::Conquest => Some(VictoryType::Elimination),
            Self::Capitals => Some(VictoryType::Capitals),
            Self::TurnLimit => Some(VictoryType::TurnLimit),
            Self::Draw | Self::Stalled | Self::Panic => None,
        }
    }
}

fn is_false(value: &bool) -> bool {
    !value
}
fn is_zero(value: &u32) -> bool {
    *value == 0
}

impl Outcome {
    pub const ALL: [Outcome; 7] = [
        Outcome::Vp,
        Outcome::Conquest,
        Outcome::Capitals,
        Outcome::TurnLimit,
        Outcome::Draw,
        Outcome::Stalled,
        Outcome::Panic,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Outcome::Vp => "VP hold",
            Outcome::Conquest => "Conquest",
            Outcome::Capitals => "Capitals",
            Outcome::TurnLimit => "Turn limit",
            Outcome::Draw => "Draw",
            Outcome::Stalled => "Stalled (safety cap)",
            Outcome::Panic => "Panic",
        }
    }
}

/// The final-state facts that decide how a game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalFacts {
    pub finished: bool,
    pub winner: Option<u8>,
    /// Teams with at least one alive empire.
    pub alive_teams: usize,
    /// The winner's VP hold reached the full count. A team at or above its target at the turn
    /// limit without a completed hold wins on the turn limit, not by VP.
    pub winner_hold_complete: bool,
    /// The winner is the only team still holding an original capital of its own, with rivals
    /// still alive (`scoring::holds_own_capital`).
    pub winner_holds_last_capital: bool,
}

/// The winner is the only living team that still holds its own original capital.
pub fn winner_holds_last_capital(state: &legatum_core::GameState) -> bool {
    let alive: Vec<_> = state
        .teams()
        .into_iter()
        .filter(|t| {
            state
                .team_members(*t)
                .iter()
                .any(|p| state.player(*p).alive)
        })
        .collect();
    alive.len() > 1
        && state.winner.is_some_and(|w| {
            alive
                .iter()
                .all(|t| (*t == w) == legatum_core::scoring::holds_own_capital(state, *t))
        })
}

/// The winner finished a VP hold (it stood at its target for `VP_HOLD_TURNS` turn ends).
pub fn winner_hold_complete(state: &legatum_core::GameState) -> bool {
    state.winner.is_some_and(|t| {
        state
            .vp_holds
            .get(&t)
            .is_some_and(|held| *held >= legatum_core::data::VP_HOLD_TURNS)
    })
}

pub fn classify(f: FinalFacts) -> Outcome {
    if !f.finished {
        Outcome::Stalled
    } else if f.winner.is_none() {
        Outcome::Draw
    } else if f.alive_teams <= 1 {
        Outcome::Conquest
    } else if f.winner_holds_last_capital {
        Outcome::Capitals
    } else if f.winner_hold_complete {
        Outcome::Vp
    } else {
        Outcome::TurnLimit
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRecord {
    #[serde(flatten)]
    pub opening: crate::openings::SeatRecord,
    pub id: u8,
    pub faction: Faction,
    pub team: u8,
    pub brain: String,
    /// Assigned experimental bias; never substituted for the observed classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<legatum_ai::Strategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype: Option<Archetype>,
    #[serde(default)]
    pub difficulty: AiDifficulty,
    pub alive: bool,
    pub vp: u32,
    pub cities: u32,
    pub techs: u32,
    pub population: i32,
    /// Treasury at the final observed state, including eliminated empires. Older reports
    /// did not retain this observation; absence must not be interpreted as zero gold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gold: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Elimination {
    pub player: u8,
    pub turn: u32,
    /// Who took or razed its last city (absent in older reports and for other causes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismCheck {
    pub ok: bool,
    /// 0 = the initial state differed; n = the state after turn n differed.
    pub first_divergent_turn: Option<u32>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRecord {
    pub index: usize,
    /// Decimal string (64-bit).
    pub seed: String,
    pub outcome: Outcome,
    #[serde(default)]
    pub victory_type: Option<VictoryType>,
    /// Second orientation of the same seed; policies reversed, map and factions unchanged.
    #[serde(default, skip_serializing_if = "is_false")]
    pub mirrored: bool,
    pub winner_team: Option<u8>,
    pub winner_players: Vec<u8>,
    pub winner_factions: Vec<Faction>,
    /// Turns resolved.
    pub turns: u32,
    pub players: Vec<PlayerRecord>,
    pub eliminations: Vec<Elimination>,
    #[serde(default)]
    pub city_captures: u32,
    #[serde(default)]
    pub athenian_city_captures: u32,
    #[serde(default)]
    pub athenian_free_techs_granted: u32,
    /// City captures resolved before turn 60, including recaptures.
    #[serde(default)]
    pub early_war_captures: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wonder_stats: Option<WonderStats>,
    /// Tile-turns eligible for a same-empire handoff at the border phase, then left after it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub workable_claims_before: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub workable_claims_after: u32,
    /// Team VP after resolving turn 75, used to identify late comebacks.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub team_vp_turn_75: BTreeMap<u8, u32>,
    /// Team VP after resolving turn 100 (the census's runaway-leader read, pillar 11).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub team_vp_turn_100: BTreeMap<u8, u32>,
    /// The furthest era any empire reached by the end (0 = Ancient). None on panics and on
    /// records written before this field existed: unknown, not Ancient.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_era: Option<u8>,
    /// None on panics and legacy records: unavailable observations are not zero counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<GameTelemetry>,
    pub wall_ms: u64,
    /// State hash after each turn (decimal), only with --determinism or tapes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hashes: Vec<String>,
    pub panic: Option<String>,
    pub determinism: Option<DeterminismCheck>,
}

impl GameRecord {
    /// A record for a game that panicked before producing a final state.
    pub fn panicked(index: usize, seed: u64, msg: String, wall_ms: u64) -> Self {
        GameRecord {
            index,
            seed: seed.to_string(),
            outcome: Outcome::Panic,
            victory_type: None,
            mirrored: false,
            winner_team: None,
            winner_players: vec![],
            winner_factions: vec![],
            turns: 0,
            players: vec![],
            eliminations: vec![],
            city_captures: 0,
            athenian_city_captures: 0,
            athenian_free_techs_granted: 0,
            early_war_captures: 0,
            wonder_stats: None,
            workable_claims_before: 0,
            workable_claims_after: 0,
            team_vp_turn_75: BTreeMap::new(),
            team_vp_turn_100: BTreeMap::new(),
            max_era: None,
            telemetry: None,
            wall_ms,
            hashes: vec![],
            panic: Some(msg),
            determinism: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderStats {
    /// Named completions persist in the chronicle after capture or destruction.
    pub completions: Vec<WonderCompletion>,
    /// Number of held wonder sources for each empire, in player-id order at game end.
    pub held_by_empire: Vec<u32>,
    /// Current wonder VP scored by the winning team at game end.
    pub winner_vp: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonderCompletion {
    pub turn: u32,
    pub building: legatum_core::BuildingKind,
    pub player: u8,
    pub faction: Faction,
}

/// Nearest-rank percentile (0-100) of a sorted slice.
pub fn percentile(sorted: &[u32], p: usize) -> Option<u32> {
    if sorted.is_empty() {
        return None;
    }
    let idx = (p * (sorted.len() - 1) + 50) / 100;
    Some(sorted[idx.min(sorted.len() - 1)])
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct WinRate {
    /// Player seats with this key across all non-panicked games.
    pub seats: u32,
    /// Seats whose team won.
    pub wins: u32,
    pub rate: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Dist {
    pub min: u32,
    pub p10: u32,
    pub median: u32,
    pub p90: u32,
    pub max: u32,
}

impl Dist {
    fn of(mut v: Vec<u32>) -> Option<Dist> {
        v.sort_unstable();
        Some(Dist {
            min: *v.first()?,
            p10: percentile(&v, 10)?,
            median: percentile(&v, 50)?,
            p90: percentile(&v, 90)?,
            max: *v.last()?,
        })
    }
}

/// How often a faction's permanent goal was reached, and when (`faction_goal_turns`).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct GoalProgress {
    /// Seats of this faction in non-panicked games whose record carries telemetry,
    /// including eliminated empires and games that stopped at a cap.
    pub seats: u32,
    /// Seats that were awarded the goal's VP before the game ended.
    pub reached: u32,
    pub rate: f64,
    /// The turn the goal was first awarded, over the seats that reached it.
    pub turns: Option<Dist>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OpeningMetric {
    pub reached_empires: usize,
    /// Not reached by the end of observed play, including safety-cap stops.
    pub never_empires: usize,
    /// Only empires that reached this milestone enter the turn distribution.
    pub turns: Option<Dist>,
}

impl OpeningMetric {
    fn of(observations: impl Iterator<Item = Option<u32>>) -> Self {
        let mut turns = Vec::new();
        let mut never_empires = 0;
        for turn in observations {
            if let Some(turn) = turn {
                turns.push(turn);
            } else {
                never_empires += 1;
            }
        }
        Self {
            reached_empires: turns.len(),
            never_empires,
            turns: Dist::of(turns),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OpeningSummary {
    pub observed_games: usize,
    pub observed_empires: usize,
    pub excluded_games_without_openings: usize,
    pub first_soldier: OpeningMetric,
    pub first_settler: OpeningMetric,
    pub first_city: OpeningMetric,
    pub second_city: OpeningMetric,
}

impl OpeningSummary {
    fn of(records: &[GameRecord]) -> Self {
        let games: Vec<_> = records
            .iter()
            .filter_map(|r| r.telemetry.as_ref()?.openings.as_ref())
            .collect();
        let empires: Vec<_> = games.iter().flat_map(|g| g.values()).collect();
        Self {
            observed_games: games.len(),
            observed_empires: empires.len(),
            excluded_games_without_openings: records.len() - games.len(),
            first_soldier: OpeningMetric::of(empires.iter().map(|e| e.first_soldier_turn)),
            first_settler: OpeningMetric::of(empires.iter().map(|e| e.first_settler_turn)),
            first_city: OpeningMetric::of(empires.iter().map(|e| e.first_city_turn)),
            second_city: OpeningMetric::of(empires.iter().map(|e| e.second_city_turn)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Summary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<crate::proxy_benchmark::Summary>,
    pub diversity: StrategyReport,
    pub games: usize,
    pub panics: usize,
    pub outcomes: BTreeMap<Outcome, usize>,
    pub by_faction: BTreeMap<Faction, WinRate>,
    /// Each faction's wins / all winning seats; sums to 1 when any game had a winner.
    pub win_share_by_faction: BTreeMap<Faction, f64>,
    pub by_brain: BTreeMap<String, WinRate>,
    pub by_difficulty: BTreeMap<AiDifficulty, WinRate>,
    pub by_faction_difficulty: BTreeMap<Faction, BTreeMap<AiDifficulty, WinRate>>,
    pub goal_progress: BTreeMap<Faction, GoalProgress>,
    pub winner_vp: Option<Dist>,
    /// Final VP of every player: vp -> count.
    pub vp_histogram: BTreeMap<u32, u32>,
    pub length: Option<Dist>,
    /// Games (non-panic) whose end state records an era, and how many had any empire in the
    /// World War era or later. The share is over the games that recorded an era.
    pub era_observed_games: usize,
    pub world_war_games: usize,
    pub world_war_share: f64,
    /// Share of non-panic games by how they ended, as fractions of non-panic games:
    /// capitals victory, VP hold, turn limit, everything else (conquest, draw, stalled).
    pub ending_shares: EndingShares,
    /// Turns resolved per outcome kind (VP hold, conquest, turn limit, draw, stalled).
    pub length_by_outcome: BTreeMap<Outcome, TurnStats>,
    /// VP-hold telemetry: holds started/broken/completed, first reach to win, comebacks.
    pub victory_holds: HoldSummary,
    pub avg_turns: f64,
    pub avg_cities: f64,
    pub avg_techs: f64,
    pub avg_eliminations: f64,
    pub total_turns: u64,
    pub total_game_ms: u64,
    pub determinism_checked: usize,
    pub determinism_failures: usize,
    pub telemetry: TelemetrySummary,
    pub openings: OpeningSummary,
}

/// The era index of "World War" in `legatum_core::eras::ERA_NAMES`.
pub fn world_war_era() -> u8 {
    legatum_core::eras::ERA_NAMES
        .iter()
        .position(|n| *n == "World War")
        .expect("the era table names a World War era") as u8
}

/// How non-panic games ended, as shares of those games (they sum to 1 when any game ended).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct EndingShares {
    pub capitals: f64,
    pub vp_hold: f64,
    pub turn_limit: f64,
    pub other: f64,
}

impl EndingShares {
    pub fn of(outcomes: &BTreeMap<Outcome, usize>) -> Self {
        let n = |o: Outcome| outcomes.get(&o).copied().unwrap_or(0);
        let played: usize = Outcome::ALL
            .iter()
            .filter(|o| **o != Outcome::Panic)
            .map(|o| n(*o))
            .sum();
        if played == 0 {
            return Self::default();
        }
        let share = |c: usize| c as f64 / played as f64;
        let (capitals, vp_hold, turn_limit) =
            (n(Outcome::Capitals), n(Outcome::Vp), n(Outcome::TurnLimit));
        Self {
            capitals: share(capitals),
            vp_hold: share(vp_hold),
            turn_limit: share(turn_limit),
            other: share(played - capitals - vp_hold - turn_limit),
        }
    }
}

fn rate(w: &mut WinRate) {
    w.rate = if w.seats == 0 {
        0.0
    } else {
        w.wins as f64 / w.seats as f64
    };
}

pub fn summarize(records: &[GameRecord]) -> Summary {
    let mut s = Summary {
        diversity: StrategyReport::of(records),
        games: records.len(),
        proxy: crate::proxy_benchmark::summarize(records),
        openings: OpeningSummary::of(records),
        victory_holds: HoldSummary::of(records),
        ..Default::default()
    };
    for o in Outcome::ALL {
        s.outcomes.insert(o, 0);
    }
    let mut winner_vp = Vec::new();
    let mut lengths = Vec::new();
    let mut lengths_by_outcome: BTreeMap<Outcome, Vec<u32>> = BTreeMap::new();
    let mut goal_turns: BTreeMap<Faction, Vec<u32>> = BTreeMap::new();
    let (mut seats, mut cities, mut techs, mut elims) = (0u64, 0u64, 0u64, 0u64);
    for r in records {
        s.telemetry.observe(Sample {
            telemetry: r.telemetry.as_ref(),
            city_captures: r.city_captures,
            eliminations: r.eliminations.len(),
            finished: !matches!(r.outcome, Outcome::Panic | Outcome::Stalled),
            winner: r.winner_team,
            winner_players: &r.winner_players,
        });
        *s.outcomes.entry(r.outcome).or_default() += 1;
        if let Some(d) = &r.determinism {
            s.determinism_checked += 1;
            if !d.ok {
                s.determinism_failures += 1;
            }
        }
        if r.outcome == Outcome::Panic {
            s.panics += 1;
            continue;
        }
        lengths.push(r.turns);
        if let Some(era) = r.max_era {
            s.era_observed_games += 1;
            s.world_war_games += (era >= world_war_era()) as usize;
        }
        lengths_by_outcome
            .entry(r.outcome)
            .or_default()
            .push(r.turns);
        s.total_turns += r.turns as u64;
        s.total_game_ms += r.wall_ms;
        elims += r.eliminations.len() as u64;
        let mut team_vp = 0;
        for p in &r.players {
            let won = r.winner_team == Some(p.team);
            let f = s.by_faction.entry(p.faction).or_default();
            f.seats += 1;
            f.wins += won as u32;
            if let Some(t) = &r.telemetry {
                let goal = s.goal_progress.entry(p.faction).or_default();
                goal.seats += 1;
                if let Some(turn) = t.faction_goal_turns.get(&p.id) {
                    goal.reached += 1;
                    goal_turns.entry(p.faction).or_default().push(*turn);
                }
            }
            let b = s.by_brain.entry(p.brain.clone()).or_default();
            b.seats += 1;
            b.wins += won as u32;
            // The Proxy has no difficulty policy; its seat uses the lobby setting only
            // to keep setup/rules identical. Exclude it from AI difficulty denominators.
            if p.brain != "proxy" {
                let d = s.by_difficulty.entry(p.difficulty).or_default();
                d.seats += 1;
                d.wins += won as u32;
                let fd = s
                    .by_faction_difficulty
                    .entry(p.faction)
                    .or_default()
                    .entry(p.difficulty)
                    .or_default();
                fd.seats += 1;
                fd.wins += won as u32;
            }
            *s.vp_histogram.entry(p.vp).or_default() += 1;
            seats += 1;
            cities += p.cities as u64;
            techs += p.techs as u64;
            if won {
                team_vp += p.vp;
            }
        }
        if r.winner_team.is_some() {
            winner_vp.push(team_vp);
        }
    }
    for (f, goal) in &mut s.goal_progress {
        goal.rate = if goal.seats == 0 {
            0.0
        } else {
            f64::from(goal.reached) / f64::from(goal.seats)
        };
        goal.turns = Dist::of(goal_turns.remove(f).unwrap_or_default());
    }
    s.by_faction.values_mut().for_each(rate);
    let winning_seats: u32 = s.by_faction.values().map(|w| w.wins).sum();
    if winning_seats > 0 {
        s.win_share_by_faction = s
            .by_faction
            .iter()
            .map(|(f, w)| (*f, f64::from(w.wins) / f64::from(winning_seats)))
            .collect();
    }
    s.length_by_outcome = lengths_by_outcome
        .into_iter()
        .filter_map(|(o, v)| Some((o, TurnStats::of(v)?)))
        .collect();
    s.by_brain.values_mut().for_each(rate);
    s.by_difficulty.values_mut().for_each(rate);
    for difficulties in s.by_faction_difficulty.values_mut() {
        difficulties.values_mut().for_each(rate);
    }
    let played = lengths.len();
    if played > 0 {
        s.avg_turns = s.total_turns as f64 / played as f64;
        s.avg_eliminations = elims as f64 / played as f64;
    }
    if seats > 0 {
        s.avg_cities = cities as f64 / seats as f64;
        s.avg_techs = techs as f64 / seats as f64;
    }
    s.winner_vp = Dist::of(winner_vp);
    s.length = Dist::of(lengths);
    if s.era_observed_games > 0 {
        s.world_war_share = s.world_war_games as f64 / s.era_observed_games as f64;
    }
    s.ending_shares = EndingShares::of(&s.outcomes);
    s
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct StrategyReport {
    pub observed: DiversitySummary,
    pub assigned: Option<DiversitySummary>,
    pub observed_check: Option<Guardrail>,
    pub assigned_check: Option<Guardrail>,
    /// Finished-game seats: assigned bias -> observed label -> count.
    pub assigned_vs_observed: BTreeMap<legatum_ai::Strategy, BTreeMap<Archetype, u64>>,
}

impl StrategyReport {
    fn of(records: &[GameRecord]) -> Self {
        let mut universe = Archetype::ACTIVE.to_vec();
        if records
            .iter()
            .flat_map(|g| &g.players)
            .any(|p| p.archetype == Some(Archetype::Diplomatic))
        {
            universe.push(Archetype::Diplomatic);
        }
        let mut r = Self {
            observed: DiversitySummary::new(universe),
            ..Default::default()
        };
        if records
            .iter()
            .flat_map(|g| &g.players)
            .any(|p| p.strategy.is_some())
        {
            r.assigned = Some(DiversitySummary::new(
                legatum_ai::Strategy::ALL
                    .into_iter()
                    .map(Archetype::from)
                    .collect(),
            ));
        }
        for game in records {
            let finished =
                !matches!(game.outcome, Outcome::Panic | Outcome::Stalled) && game.panic.is_none();
            let mut players: Vec<_> = game
                .players
                .iter()
                .map(|p| EmpireResult {
                    team: p.team,
                    faction: p.faction,
                    archetype: p.archetype,
                    won: game.winner_team == Some(p.team),
                    alive: p.alive,
                    vp: p.vp,
                    eliminated_turn: game
                        .eliminations
                        .iter()
                        .find(|e| e.player == p.id)
                        .map(|e| e.turn),
                })
                .collect();
            r.observed.observe(finished, &players);
            if let Some(assigned) = &mut r.assigned {
                for (p, recorded) in players.iter_mut().zip(&game.players) {
                    p.archetype = recorded.strategy.map(Archetype::from);
                    if finished
                        && let (Some(bias), Some(label)) = (recorded.strategy, recorded.archetype)
                    {
                        *r.assigned_vs_observed
                            .entry(bias)
                            .or_default()
                            .entry(label)
                            .or_default() += 1;
                    }
                }
                assigned.observe(finished, &players);
            }
        }
        r.observed.finish();
        if let Some(assigned) = &mut r.assigned {
            assigned.finish();
        }
        r.check(GuardrailConfig::default());
        r
    }

    pub fn check(&mut self, config: GuardrailConfig) {
        self.observed_check = Some(diversity::check_diversity(&self.observed, config));
        self.assigned_check = self
            .assigned
            .as_ref()
            .map(|s| diversity::check_diversity(s, config));
    }

    fn markdown(&self) -> String {
        let mut out = diversity::render_markdown(
            &self.observed,
            self.observed_check.as_ref().unwrap(),
            "observed behavior (classifier v2)",
        );
        if let (Some(s), Some(check)) = (&self.assigned, &self.assigned_check) {
            out += &diversity::render_markdown(s, check, "assigned scripted biases");
            out += "\n### Assigned bias versus observed behavior\n\nA bias is an attempted strategy, not proof that play met its classifier thresholds. Counts include finished-game seats only.\n\n| Bias |";
            for a in &self.observed.archetypes {
                out += &format!(" {} |", a.name());
            }
            out += "\n| --- |";
            for _ in &self.observed.archetypes {
                out += " ---: |";
            }
            out += "\n";
            for (bias, row) in &self.assigned_vs_observed {
                out += &format!("| {bias} |");
                for a in &self.observed.archetypes {
                    out += &format!(" {} |", row.get(a).copied().unwrap_or(0));
                }
                out += "\n";
            }
        }
        out
    }
}

/// Runtime checkout provenance. Missing Git metadata is reported as unknown, never clean.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SourceRevision {
    /// Checkout observed at run start, not a compile-time embedded revision.
    pub commit: Option<String>,
    pub worktree_dirty: Option<bool>,
}

impl SourceRevision {
    pub fn current() -> Self {
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                .output()
                .ok()?;
            output
                .status
                .success()
                .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        };
        Self {
            commit: git(&["rev-parse", "HEAD"]),
            worktree_dirty: git(&["status", "--porcelain", "--untracked-files=normal"])
                .map(|s| !s.is_empty()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RunMeta {
    pub label: String,
    pub date: String,
    pub started_at: String,
    pub games: usize,
    pub first_seed: String,
    pub threads: usize,
    pub brain: String,
    pub difficulty_rotation: Option<u8>,
    pub proxy: Option<crate::proxy_benchmark::Config>,
    pub proxy_version: Option<u32>,
    pub strategies: Vec<legatum_ai::Strategy>,
    pub strategy_matchups: bool,
    pub safety_cap: u32,
    pub turn_limit: Option<u32>,
    pub determinism: bool,
    pub wall_ms: u64,
    pub source: SourceRevision,
    pub command: Vec<String>,
    /// See runner::scheduled_game for ordinary versus mirrored-tournament seed schedules.
    #[serde(serialize_with = "serialize_setup")]
    pub first_setup: GameSetup,
}

fn serialize_setup<S: serde::Serializer>(
    setup: &GameSetup,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    legatum_wasm::json::setup_to_value(setup).serialize(serializer)
}

fn pct(n: usize, d: usize) -> String {
    if d == 0 {
        "-".into()
    } else {
        format!("{:.1}%", 100.0 * n as f64 / d as f64)
    }
}

fn dist_row(name: &str, d: &Option<Dist>) -> String {
    match d {
        Some(d) => format!(
            "| {name} | {} | {} | {} | {} | {} |\n",
            d.min, d.p10, d.median, d.p90, d.max
        ),
        None => format!("| {name} | - | - | - | - | - |\n"),
    }
}

fn promotion_markdown(records: &[GameRecord]) -> String {
    let mut promoted = BTreeMap::<u8, u64>::new();
    let mut last = BTreeMap::<u8, u64>::new();
    let mut games = 0;
    for p in records
        .iter()
        .filter_map(|r| r.telemetry.as_ref()?.promotions.as_ref())
    {
        games += 1;
        for (l, n) in &p.promoted_to {
            *promoted.entry(*l).or_default() += n;
        }
        for (l, n) in &p.final_military_by_level {
            *last.entry(*l).or_default() += n;
        }
    }
    let mut o = format!(
        "\n## Promotions\n\n{games} games with promotion telemetry. Promotions are UnitPromoted events by the level reached; \
         final units are military units alive when the game ended.\n\n| Level | Promotions / game | Final units / game |\n| --- | --- | --- |\n"
    );
    for level in 0..=legatum_core::rules::promotion::MAX_LEVEL {
        let per = |m: &BTreeMap<u8, u64>| {
            if games == 0 {
                "n/a".to_string()
            } else {
                format!("{:.2}", *m.get(&level).unwrap_or(&0) as f64 / games as f64)
            }
        };
        o += &format!("| {level} | {} | {} |\n", per(&promoted), per(&last));
    }
    o
}

fn turn_stats_cell(t: &Option<TurnStats>) -> String {
    t.as_ref().map_or_else(
        || "-".into(),
        |t| format!("{:.1} / {} / {}-{}", t.mean, t.median, t.min, t.max),
    )
}

/// The compact run overview: lengths, win reasons, VP-hold telemetry and winner share.
fn overview_markdown(s: &Summary) -> String {
    let h = &s.victory_holds;
    let mut o = String::from("## Overview\n\n| Metric | Value |\n| --- | --- |\n");
    o += &format!(
        "| Games (non-panic) | {} of {} |\n| Game length, turns (mean / median / min-max) | {} |\n",
        s.games - s.panics,
        s.games,
        s.length.as_ref().map_or_else(
            || "-".into(),
            |d| format!("{:.1} / {} / {}-{}", s.avg_turns, d.median, d.min, d.max)
        ),
    );
    let mix = Outcome::ALL
        .iter()
        .filter(|k| s.outcomes.get(k).is_some_and(|n| *n > 0))
        .map(|k| format!("{} {}", k.label(), pct(s.outcomes[k], s.games)))
        .collect::<Vec<_>>()
        .join(", ");
    o += &format!("| Win reason mix | {mix} |\n");
    // Appended rows (keep existing rows stable for parsers): the length and ending read the
    // review asked for on every census.
    let e = &s.ending_shares;
    o += &format!(
        "| Median end turn | {} |\n| Ending shares of played games (capitals / VP hold / turn limit / other) | {:.1}% / {:.1}% / {:.1}% / {:.1}% |\n",
        s.length
            .as_ref()
            .map_or_else(|| "-".into(), |d| d.median.to_string()),
        100.0 * e.capitals,
        100.0 * e.vp_hold,
        100.0 * e.turn_limit,
        100.0 * e.other,
    );
    if s.era_observed_games == 0 {
        o += "| Games reaching the World War era | not observed (no era in these records) |\n";
    } else {
        o += &format!(
            "| Games reaching the World War era (any empire) | {} of {} ({:.1}%) |\n",
            s.world_war_games,
            s.era_observed_games,
            100.0 * s.world_war_share,
        );
    }
    if h.observed_games == 0 {
        o += "| VP holds | not observed (no hold telemetry in these records) |\n";
    } else {
        o += &format!(
            "| VP hold length | {} turn ends (the reaching turn counts) |\n\
             | Holds started / broken / completed / unfinished | {} / {} / {} / {} |\n\
             | Hold break rate | {:.1}% of started; {:.1}% of resolved (broken / (broken + completed)) |\n\
             | Hold wins | {} |\n\
             | First reach to win, turns (mean / median / min-max) | {} |\n\
             | Final hold start to win, turns | {} |\n\
             | Comebacks (winner broke an earlier hold of its own) | {} |\n\
             | Wins by a team another team beat to its target | {} |\n",
            h.turns_needed.unwrap_or(0),
            h.started,
            h.broken,
            h.completed,
            h.unfinished,
            100.0 * h.break_rate,
            100.0 * h.break_rate_resolved,
            h.hold_wins,
            turn_stats_cell(&h.first_reach_to_win),
            turn_stats_cell(&h.final_hold_to_win),
            h.comebacks,
            h.wins_not_first_to_reach,
        );
    }
    o += "\nGame length by how the game ended (turns resolved; a stall is a safety-cap stop, never a win):\n\n\
          | Ended by | Games | Length mean / median / min-max |\n| --- | --- | --- |\n";
    for (k, t) in &s.length_by_outcome {
        o += &format!(
            "| {} | {} | {} |\n",
            k.label(),
            t.n,
            turn_stats_cell(&Some(t.clone()))
        );
    }
    o += "\nWinner share by faction (wins of that faction's seats / all winning seats):\n\n\
          | Faction | Wins | Share of wins |\n| --- | --- | --- |\n";
    for (f, w) in &s.by_faction {
        let share = s.win_share_by_faction.get(f).copied().unwrap_or(0.0);
        o += &format!("| {f:?} | {} | {:.1}% |\n", w.wins, 100.0 * share);
    }
    o += "\n";
    o
}

pub fn render_markdown(meta: &RunMeta, s: &Summary, records: &[GameRecord]) -> String {
    let mut o = String::new();
    o += &format!("# Simulation: {} ({})\n\n", meta.label, meta.date);
    o += &format!(
        "{} games, first seed {}, brain `{}`, turn limit {}, safety cap {}, {} threads, started {}. \
         Wall time {:.1}s.\n\n",
        meta.games,
        meta.first_seed,
        meta.brain,
        meta.turn_limit
            .map_or_else(|| "off".into(), |n| n.to_string()),
        meta.safety_cap,
        meta.threads,
        meta.started_at,
        meta.wall_ms as f64 / 1000.0
    );
    o += &format!(
        "Source checkout at run start: `{}`; dirty: {}. Map {}×{}, {} initial seats. \
         `first_setup` and exact binary argv are recorded in JSON. Ordinary games use seed + i \
         (wrapping); the strategy tournament schedule is specified below. All games use a \
         deterministic shuffle of the full faction pool, taking eight distinct factions, with any Proxy rotation described below.\n\nCommand argv: `{}`\n\n",
        meta.source.commit.as_deref().unwrap_or("unavailable"),
        meta.source
            .worktree_dirty
            .map_or("unknown", |dirty| if dirty { "yes" } else { "no" }),
        meta.first_setup.width,
        meta.first_setup.height,
        meta.first_setup.players.len(),
        serde_json::to_string(&meta.command)
            .expect("argv serializes")
            .replace('`', "\\u0060")
            .replace('|', "\\u007c")
    );

    if let Some(offset) = meta.difficulty_rotation {
        o += &format!(
            "Difficulty rotation offset {offset}: seat `s` uses `[Easy, Normal, Hard][(s + {offset}) % 3]`. \
             Repeat the same seeds at offsets 0, 1 and 2 for equal exposure per faction and start.\n\n"
        );
    }
    if let Some(proxy) = &s.proxy {
        o += &crate::proxy_benchmark::render(proxy, meta.proxy);
    }
    if meta.strategy_matchups {
        o += "Strategy tournament: 10 unordered pairs, two mirrored eight-seat games per pair. Game i uses seed first_seed + floor(i/2), wrapping in u64; pair floor(i/2) modulo 10, in Strategy::ALL order. Alternating seats use each bias; the second game swaps every seat, holding map and faction assignment fixed. These are free-for-all games, not two allied teams.\n\n";
    } else if !meta.strategies.is_empty() {
        o += &format!(
            "Scripted seat biases (repeat to fill eight seats): {:?}.\n\n",
            meta.strategies
        );
    }
    o += &overview_markdown(s);
    o += "## Outcomes\n\n| Outcome | Games | Share |\n| --- | --- | --- |\n";
    for (k, n) in &s.outcomes {
        o += &format!("| {} | {n} | {} |\n", k.label(), pct(*n, s.games));
    }

    o += "\n## Win rate by faction\n\nPer seat: wins / seats of that faction. \
          With 8 single-empire teams, an even game is 12.5% per seat.\n\n\
          | Faction | Seats | Wins | Win rate |\n| --- | --- | --- | --- |\n";
    for (f, w) in &s.by_faction {
        o += &format!(
            "| {f:?} | {} | {} | {:.1}% |\n",
            w.seats,
            w.wins,
            100.0 * w.rate
        );
    }
    if s.by_brain.len() > 1 {
        o += "\n## Win rate by brain\n\n| Brain | Seats | Wins | Win rate |\n| --- | --- | --- | --- |\n";
        for (b, w) in &s.by_brain {
            o += &format!(
                "| {b} | {} | {} | {:.1}% |\n",
                w.seats,
                w.wins,
                100.0 * w.rate
            );
        }
    }

    o += "\n## Win rate by difficulty\n\nAll initial seats in non-panicked games, including eliminated empires and cap stops. \
          Determinism replays are not additional samples.\n\n\
          | Difficulty | Seats | Wins | Win rate |\n| --- | --- | --- | --- |\n";
    for (d, w) in &s.by_difficulty {
        o += &format!(
            "| {d:?} | {} | {} | {:.1}% |\n",
            w.seats,
            w.wins,
            100.0 * w.rate
        );
    }
    o += "\n## Win rate by faction and difficulty\n\n\
          | Faction | Difficulty | Seats | Wins | Win rate |\n| --- | --- | --- | --- | --- |\n";
    for (f, difficulties) in &s.by_faction_difficulty {
        for (d, w) in difficulties {
            o += &format!(
                "| {f:?} | {d:?} | {} | {} | {:.1}% |\n",
                w.seats,
                w.wins,
                100.0 * w.rate
            );
        }
    }

    o += "\n## Faction goal progress\n\nPer seat: how often the faction's permanent goal was reached before the game ended, \
          and the turn its VP was first awarded (over the seats that reached it). Seats are all initial \
          empires of non-panicked games with telemetry, including eliminated empires and cap stops.\n\n\
          | Faction | Seats | Reached | Goal rate | Min | P10 | Median turn | P90 | Max |\n\
          | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n";
    for (f, g) in &s.goal_progress {
        let turns = match &g.turns {
            Some(d) => format!(
                "{} | {} | {} | {} | {}",
                d.min, d.p10, d.median, d.p90, d.max
            ),
            None => "- | - | - | - | -".into(),
        };
        o += &format!(
            "| {f:?} | {} | {} | {:.1}% | {turns} |\n",
            g.seats,
            g.reached,
            100.0 * g.rate
        );
    }

    o += "\n## Distributions\n\n| Metric | Min | P10 | Median | P90 | Max |\n\
          | --- | --- | --- | --- | --- | --- |\n";
    o += &dist_row(
        "Observed game length (turns, including cap stops)",
        &s.length,
    );
    o += &dist_row("Winning team VP", &s.winner_vp);
    for (name, metric) in [
        (
            "First Soldier completed (turn, per empire)",
            &s.openings.first_soldier,
        ),
        (
            "First Settler completed (turn, per empire)",
            &s.openings.first_settler,
        ),
        (
            "First city founded (turn, per empire)",
            &s.openings.first_city,
        ),
        (
            "Second city founded (turn, per empire)",
            &s.openings.second_city,
        ),
    ] {
        o += &dist_row(name, &metric.turns);
    }
    o += "\nSafety-cap stops are censored observations, not completed game lengths.\n";
    o += &format!(
        "\nOpenings: {} observed empires in {} first plays; {} games without opening telemetry excluded. \
         Turn distributions include only empires that reached the milestone. \
         Never means not reached by the end of observed play, including eliminated empires and safety-cap stops. \
         Per-empire turns are in JSON (`games[].telemetry.openings`, keyed by player ID).\n\n\
         | Opening milestone | Reached empires | Never empires |\n| --- | --- | --- |\n",
        s.openings.observed_empires,
        s.openings.observed_games,
        s.openings.excluded_games_without_openings,
    );
    for (name, metric) in [
        ("First Soldier completed", &s.openings.first_soldier),
        ("First Settler completed", &s.openings.first_settler),
        ("First city founded", &s.openings.first_city),
        ("Second city founded", &s.openings.second_city),
    ] {
        o += &format!(
            "| {name} | {} | {} |\n",
            metric.reached_empires, metric.never_empires
        );
    }
    o += "\nFinal VP per empire (all seats):\n\n| VP | Empires |\n| --- | --- |\n";
    for (vp, n) in &s.vp_histogram {
        o += &format!("| {vp} | {n} |\n");
    }

    o += &format!(
        "\n## End state\n\n- Average game length: {:.1} turns\n- Average cities per empire: {:.2}\n\
         - Average techs per empire: {:.2}\n- Average eliminations per game: {:.2}\n",
        s.avg_turns, s.avg_cities, s.avg_techs, s.avg_eliminations
    );

    if records.iter().any(|r| r.outcome == Outcome::Stalled) {
        o += "\n## Stalled game VP standings\n\nSafety cap reached without victory. Final scores (including eliminated empires):\n\n| Seed | Turns | Faction / team: VP |\n| --- | --- | --- |\n";
        for r in records.iter().filter(|r| r.outcome == Outcome::Stalled) {
            let mut players: Vec<_> = r.players.iter().collect();
            players.sort_by_key(|p| (std::cmp::Reverse(p.vp), p.id));
            let scores = players
                .iter()
                .map(|p| {
                    format!(
                        "{:?} / {}: {}{}",
                        p.faction,
                        p.team,
                        p.vp,
                        if p.alive { "" } else { " (eliminated)" }
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            o += &format!("| {} | {} | {} |\n", r.seed, r.turns, scores);
        }
    }

    o += &s.diversity.markdown();
    o += &telemetry::render_markdown(&s.telemetry);
    o += &promotion_markdown(records);
    o += "\n## Per-game events\n\nEach row is one observed first play; unavailable telemetry is shown as n/a. \
          Full per-player/team score sources and turn-50 standings are in JSON.\n\n\
          | Game | Seed | Resolved turns | Battles | City captures | Settler captures | Eliminations |\n\
          | --- | --- | --- | --- | --- | --- | --- |\n";
    for r in records {
        let counts = r.telemetry.as_ref().map_or_else(
            || "n/a | n/a | n/a | n/a".to_owned(),
            |m| {
                format!(
                    "{} | {} | {} | {}",
                    m.battles,
                    r.city_captures,
                    m.settler_captures,
                    r.eliminations.len()
                )
            },
        );
        o += &format!("| {} | {} | {} | {counts} |\n", r.index, r.seed, r.turns);
    }

    let secs = meta.wall_ms as f64 / 1000.0;
    o += &format!(
        "\n## Throughput\n\n- Turns simulated: {} (first play of each game)\n\
         - Aggregate: {:.1} turns/s over {:.1}s wall\n- Per game thread: {:.1} turns/s\n",
        s.total_turns,
        if secs > 0.0 {
            s.total_turns as f64 / secs
        } else {
            0.0
        },
        secs,
        if s.total_game_ms > 0 {
            s.total_turns as f64 * 1000.0 / s.total_game_ms as f64
        } else {
            0.0
        }
    );

    o += "\n## Determinism\n\n";
    if !meta.determinism {
        o += "Not checked (run with `--determinism`).\n";
    } else if s.determinism_checked == 0 {
        o += "Not checked: no game completed its first play.\n";
    } else if s.determinism_failures == 0 {
        o += &format!(
            "PASS: {} games played twice with identical per-turn hashes (and save/load keeps the \
             final hash).\n",
            s.determinism_checked
        );
    } else {
        o += &format!(
            "FAIL: {} of {} games diverged.\n\n| Game | Seed | First divergent turn | Detail |\n\
             | --- | --- | --- | --- |\n",
            s.determinism_failures, s.determinism_checked
        );
        for r in records {
            if let Some(d) = r.determinism.as_ref().filter(|d| !d.ok) {
                o += &format!(
                    "| {} | {} | {} | {} |\n",
                    r.index,
                    r.seed,
                    d.first_divergent_turn
                        .map_or("-".to_string(), |t| t.to_string()),
                    d.detail.as_deref().unwrap_or("")
                );
            }
        }
    }

    o += "\n## Panics\n\n";
    if s.panics == 0 {
        o += "None.\n";
    } else {
        o += &format!(
            "{} games panicked.\n\n| Game | Seed | Message |\n| --- | --- | --- |\n",
            s.panics
        );
        for r in records.iter().filter(|r| r.outcome == Outcome::Panic) {
            let msg = r.panic.as_deref().unwrap_or("").replace(['\n', '|'], " ");
            o += &format!("| {} | {} | {msg} |\n", r.index, r.seed);
        }
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> FinalFacts {
        FinalFacts {
            finished: true,
            winner: Some(2),
            alive_teams: 5,
            winner_hold_complete: true,
            winner_holds_last_capital: false,
        }
    }

    #[test]
    fn classify_outcomes() {
        assert_eq!(classify(facts()), Outcome::Vp);
        assert_eq!(
            classify(FinalFacts {
                alive_teams: 1,
                ..facts()
            }),
            Outcome::Conquest
        );
        assert_eq!(
            classify(FinalFacts {
                winner_hold_complete: false,
                ..facts()
            }),
            Outcome::TurnLimit
        );
        // The capitals victory is its own outcome, even when a VP hold completed the same turn
        // (the core checks capitals first).
        for hold in [false, true] {
            assert_eq!(
                classify(FinalFacts {
                    winner_hold_complete: hold,
                    winner_holds_last_capital: true,
                    ..facts()
                }),
                Outcome::Capitals
            );
        }
        assert_eq!(
            classify(FinalFacts {
                alive_teams: 1,
                winner_holds_last_capital: true,
                ..facts()
            }),
            Outcome::Conquest,
            "last team standing keeps its own label"
        );
        assert_eq!(
            Outcome::Capitals.victory_type(),
            Some(VictoryType::Capitals)
        );
        assert_eq!(
            classify(FinalFacts {
                winner: None,
                ..facts()
            }),
            Outcome::Draw
        );
        assert_eq!(
            classify(FinalFacts {
                finished: false,
                ..facts()
            }),
            Outcome::Stalled
        );
    }

    fn player(id: u8, faction: Faction, vp: u32, cities: u32, techs: u32) -> PlayerRecord {
        PlayerRecord {
            opening: Default::default(),
            id,
            faction,
            difficulty: AiDifficulty::Normal,
            strategy: None,
            archetype: None,
            team: id,
            brain: if id.is_multiple_of(2) {
                "scripted"
            } else {
                "random"
            }
            .into(),
            alive: true,
            vp,
            cities,
            techs,
            population: 3,
            gold: Some(0),
        }
    }

    fn game(index: usize, winner: Option<u8>, outcome: Outcome, turns: u32) -> GameRecord {
        GameRecord {
            index,
            seed: index.to_string(),
            outcome,
            victory_type: outcome.victory_type(),
            mirrored: false,
            winner_team: winner,
            winner_players: winner.into_iter().collect(),
            winner_factions: vec![],
            turns,
            players: vec![
                player(0, Faction::India, 10, 4, 6),
                player(1, Faction::Byzantium, 4, 2, 2),
            ],
            eliminations: vec![Elimination {
                player: 1,
                turn: 50,
                by: None,
            }],
            wall_ms: 1000,
            city_captures: 0,
            athenian_city_captures: 0,
            athenian_free_techs_granted: 0,
            early_war_captures: 0,
            wonder_stats: None,
            workable_claims_before: 0,
            workable_claims_after: 0,
            team_vp_turn_75: BTreeMap::new(),
            team_vp_turn_100: BTreeMap::new(),
            max_era: None,
            telemetry: None,
            hashes: vec![],
            panic: None,
            determinism: None,
        }
    }

    #[test]
    fn percentiles_nearest_rank() {
        let v: Vec<u32> = (1..=10).collect();
        assert_eq!(percentile(&v, 0), Some(1));
        assert_eq!(percentile(&v, 50), Some(6));
        assert_eq!(percentile(&v, 90), Some(9));
        assert_eq!(percentile(&v, 100), Some(10));
        assert_eq!(percentile(&[], 50), None);
    }

    #[test]
    fn goal_progress_counts_seats_and_turns_per_faction() {
        let metrics = |goal: Option<u32>| {
            let state = legatum_core::new_game(&GameSetup::all_ai(1));
            let mut t = crate::telemetry::Collector::default().finish(&state, 0);
            t.faction_goal_turns.extend(goal.map(|turn| (0u8, turn)));
            t
        };
        let mut recs = vec![
            game(0, Some(0), Outcome::Vp, 100),
            game(1, Some(0), Outcome::Vp, 120),
            game(2, Some(1), Outcome::Vp, 150),
            game(3, None, Outcome::Draw, 150),
            GameRecord::panicked(4, 99, "boom".into(), 5),
        ];
        // India (player 0) reaches its goal on turns 80 and 120 and misses in the third game;
        // the fourth record carries no telemetry and the fifth panicked.
        recs[0].telemetry = Some(metrics(Some(80)));
        recs[1].telemetry = Some(metrics(Some(120)));
        recs[2].telemetry = Some(metrics(None));
        let s = summarize(&recs);
        let india = &s.goal_progress[&Faction::India];
        assert_eq!((india.seats, india.reached), (3, 2));
        assert!((india.rate - 2.0 / 3.0).abs() < 1e-9);
        let d = india.turns.clone().unwrap();
        assert_eq!((d.min, d.max), (80, 120));
        let byzantium = &s.goal_progress[&Faction::Byzantium];
        assert_eq!(
            (byzantium.seats, byzantium.reached, byzantium.turns.clone()),
            (3, 0, None)
        );
        let meta = RunMeta {
            label: "t".into(),
            date: "2026-10-06".into(),
            started_at: "2026-10-06T00:00:00Z".into(),
            games: 5,
            first_seed: "0".into(),
            threads: 2,
            brain: "mixed".into(),
            strategies: vec![],
            strategy_matchups: false,
            difficulty_rotation: None,
            proxy: None,
            proxy_version: None,
            safety_cap: 600,
            turn_limit: Some(150),
            determinism: true,
            wall_ms: 2000,
            source: SourceRevision::default(),
            command: vec![],
            first_setup: GameSetup::all_ai(0),
        };
        let md = render_markdown(&meta, &s, &recs);
        assert!(md.contains("## Faction goal progress"));
        assert!(md.contains("| India | 3 | 2 | 66.7% | 80 |"), "{md}");
        assert!(
            md.contains("| Byzantium | 3 | 0 | 0.0% | - | - | - | - | - |"),
            "{md}"
        );
    }

    #[test]
    fn summarize_aggregates_fabricated_records() {
        let mut recs = vec![
            game(0, Some(0), Outcome::Vp, 100),
            game(1, Some(0), Outcome::Vp, 120),
            game(2, Some(1), Outcome::TurnLimit, 150),
            game(3, None, Outcome::Draw, 150),
            GameRecord::panicked(4, 99, "boom".into(), 5),
        ];
        recs[1].determinism = Some(DeterminismCheck {
            ok: false,
            first_divergent_turn: Some(7),
            detail: None,
        });
        let s = summarize(&recs);
        assert_eq!(s.games, 5);
        assert_eq!(s.panics, 1);
        assert_eq!(s.outcomes[&Outcome::Vp], 2);
        assert_eq!(s.outcomes[&Outcome::TurnLimit], 1);
        assert_eq!(s.outcomes[&Outcome::Draw], 1);
        assert_eq!(s.outcomes[&Outcome::Conquest], 0);
        let india = &s.by_faction[&Faction::India];
        assert_eq!((india.seats, india.wins), (4, 2));
        assert!((india.rate - 0.5).abs() < 1e-9);
        let byzantium = &s.by_faction[&Faction::Byzantium];
        assert_eq!((byzantium.seats, byzantium.wins), (4, 1));
        assert_eq!(s.by_brain["scripted"].wins, 2);
        assert!((s.win_share_by_faction[&Faction::India] - 2.0 / 3.0).abs() < 1e-9);
        assert!((s.win_share_by_faction[&Faction::Byzantium] - 1.0 / 3.0).abs() < 1e-9);
        // Winning team VP: 10, 10, 4.
        let w = s.winner_vp.clone().unwrap();
        assert_eq!((w.min, w.median, w.max), (4, 10, 10));
        assert_eq!(s.vp_histogram[&10], 4);
        assert_eq!(s.vp_histogram[&4], 4);
        let l = s.length.clone().unwrap();
        assert_eq!((l.min, l.median, l.max), (100, 150, 150));
        assert!((s.avg_turns - 130.0).abs() < 1e-9);
        assert!((s.avg_cities - 3.0).abs() < 1e-9);
        assert!((s.avg_techs - 4.0).abs() < 1e-9);
        assert!((s.avg_eliminations - 1.0).abs() < 1e-9);
        assert_eq!(s.total_turns, 520);
        assert_eq!((s.determinism_checked, s.determinism_failures), (1, 1));

        let meta = RunMeta {
            label: "t".into(),
            date: "2026-09-26".into(),
            started_at: "2026-09-26T00:00:00Z".into(),
            games: 5,
            first_seed: "0".into(),
            threads: 2,
            brain: "mixed".into(),
            strategies: vec![],
            strategy_matchups: false,
            difficulty_rotation: None,
            proxy: None,
            proxy_version: None,
            safety_cap: 600,
            turn_limit: Some(150),
            determinism: true,
            wall_ms: 2000,
            source: SourceRevision::default(),
            command: vec![],
            first_setup: GameSetup::all_ai(0),
        };
        let md = render_markdown(&meta, &s, &recs);
        assert!(md.contains("| VP hold | 2 | 40.0% |"), "{md}");
        assert!(md.contains("| India | 4 | 2 | 50.0% |"), "{md}");
        assert!(md.contains("## Win rate by brain"));
        assert!(md.contains("FAIL: 1 of 1 games diverged"));
        assert!(md.contains("| 1 | 1 | 7 |"));
        assert!(md.contains("| 4 | 99 | boom |"));
    }

    #[test]
    fn empty_run_summarizes() {
        let s = summarize(&[]);
        assert_eq!(s.games, 0);
        assert!(s.length.is_none());
        assert_eq!(s.avg_turns, 0.0);
    }

    #[test]
    fn legacy_player_records_default_to_normal_difficulty() {
        let mut value = serde_json::to_value(player(0, Faction::India, 10, 4, 6)).unwrap();
        value.as_object_mut().unwrap().remove("difficulty");
        let legacy: PlayerRecord = serde_json::from_value(value).unwrap();
        assert_eq!(legacy.difficulty, AiDifficulty::Normal);
    }

    #[test]
    fn record_json_omits_empty_hashes_and_keeps_seed_string() {
        let r = GameRecord::panicked(0, u64::MAX, "x".into(), 1);
        let v = serde_json::to_value(&r).unwrap();
        assert!(v.get("hashes").is_none());
        assert_eq!(v["seed"], serde_json::Value::from(u64::MAX.to_string()));
        assert_eq!(v["outcome"], serde_json::Value::from("panic"));
    }

    #[test]
    fn legacy_records_do_not_become_zero_telemetry_samples() {
        let mut value = serde_json::to_value(game(0, None, Outcome::Draw, 150)).unwrap();
        for field in ["telemetry", "city_captures", "team_vp_turn_75"] {
            value.as_object_mut().unwrap().remove(field);
        }
        let record: GameRecord = serde_json::from_value(value).unwrap();
        assert!(record.telemetry.is_none());
        let summary = summarize(&[record]);
        assert_eq!(summary.games, 1);
        assert_eq!(summary.telemetry.sample_games, 0);
        assert_eq!(summary.telemetry.excluded_games_without_telemetry, 1);
        assert_eq!(summary.telemetry.turn_50.any_leader.eligible_games, 0);
        assert_eq!(summary.openings.observed_games, 0);
        assert_eq!(summary.openings.excluded_games_without_openings, 1);
    }

    #[test]
    fn older_telemetry_without_openings_is_excluded_instead_of_counted_as_never() {
        let mut record = game(0, None, Outcome::Draw, 150);
        let state = legatum_core::new_game(&GameSetup::all_ai(1));
        record.telemetry = Some(telemetry::Collector::default().finish(&state, 0));
        let mut value = serde_json::to_value(record).unwrap();
        value["telemetry"]
            .as_object_mut()
            .unwrap()
            .remove("openings");
        let legacy: GameRecord = serde_json::from_value(value).unwrap();
        assert!(legacy.telemetry.as_ref().unwrap().openings.is_none());
        let summary = summarize(&[legacy, GameRecord::panicked(1, 2, "boom".into(), 0)]);
        assert_eq!(summary.telemetry.sample_games, 1);
        assert_eq!(summary.openings.observed_games, 0);
        assert_eq!(summary.openings.observed_empires, 0);
        assert_eq!(summary.openings.excluded_games_without_openings, 2);
        for metric in [
            summary.openings.first_soldier,
            summary.openings.first_settler,
            summary.openings.second_city,
        ] {
            assert_eq!(metric.reached_empires, 0);
            assert_eq!(metric.never_empires, 0);
            assert!(metric.turns.is_none());
        }
    }

    #[test]
    fn world_war_is_era_six_and_shares_count_only_games_that_recorded_an_era() {
        assert_eq!(world_war_era(), 6);
        let mut a = game(0, Some(0), Outcome::Vp, 120);
        a.max_era = Some(6);
        let mut b = game(1, Some(0), Outcome::Capitals, 140);
        b.max_era = Some(5);
        let mut c = game(2, Some(1), Outcome::TurnLimit, 160);
        c.max_era = Some(8);
        let legacy = game(3, Some(1), Outcome::Conquest, 180);
        let s = summarize(&[a, b, c, legacy, GameRecord::panicked(4, 4, "x".into(), 0)]);
        assert_eq!(s.era_observed_games, 3);
        assert_eq!(s.world_war_games, 2);
        assert!((s.world_war_share - 2.0 / 3.0).abs() < 1e-9);
        // Median end turn is over the four played games (nearest rank, upper median).
        assert_eq!(s.length.as_ref().unwrap().median, 160);
        let e = s.ending_shares;
        assert!((e.vp_hold - 0.25).abs() < 1e-9);
        assert!((e.capitals - 0.25).abs() < 1e-9);
        assert!((e.turn_limit - 0.25).abs() < 1e-9);
        assert!((e.other - 0.25).abs() < 1e-9);
    }

    #[test]
    fn overview_appends_the_length_and_era_rows_after_the_stable_ones() {
        let mut a = game(0, Some(0), Outcome::Vp, 100);
        a.max_era = Some(7);
        let md = overview_markdown(&summarize(&[a]));
        let mix = md.find("| Win reason mix").unwrap();
        let median = md.find("| Median end turn | 100 |").unwrap();
        assert!(mix < median);
        assert!(md.contains("| Games reaching the World War era (any empire) | 1 of 1 (100.0%) |"));
        assert!(md.contains("100.0% / 0.0% / 0.0%") || md.contains("0.0% / 100.0% / 0.0% / 0.0%"));
        let legacy = overview_markdown(&summarize(&[game(0, Some(0), Outcome::Vp, 100)]));
        assert!(legacy.contains("not observed (no era in these records)"));
    }

    #[test]
    fn a_record_without_max_era_reads_as_unknown() {
        let mut value = serde_json::to_value(game(0, None, Outcome::Draw, 10)).unwrap();
        assert!(value.get("max_era").is_none());
        value["max_era"] = serde_json::json!(6);
        let r: GameRecord = serde_json::from_value(value).unwrap();
        assert_eq!(r.max_era, Some(6));
    }
}
