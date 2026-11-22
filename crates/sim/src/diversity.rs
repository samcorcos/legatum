//! Strategy labels describe observed behavior, independently of a brain's assigned bias.
//! Only reporting uses floating point. Observations and classification are integer-only.
use legatum_ai::Strategy;
use legatum_core::{
    BuildingKind, Event, Faction, GameState, ProductionItem, UnitKind, data, economy,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub const CLASSIFIER_VERSION: u32 = 2;
pub const DEFAULT_MIN_ENTROPY: f64 = 0.80;
pub const DEFAULT_MIN_MATCHUP_GAMES: u64 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    Rush,
    Wide,
    Science,
    Turtle,
    Economic,
    Generalist,
    Diplomatic,
}

impl Archetype {
    pub const ACTIVE: [Self; 6] = [
        Self::Rush,
        Self::Wide,
        Self::Science,
        Self::Turtle,
        Self::Economic,
        Self::Generalist,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Rush => "Rush",
            Self::Wide => "Wide",
            Self::Science => "Science",
            Self::Turtle => "Turtle/Wonder",
            Self::Economic => "Economic",
            Self::Generalist => "Generalist",
            Self::Diplomatic => "Diplomatic",
        }
    }
}

impl From<Strategy> for Archetype {
    fn from(s: Strategy) -> Self {
        match s {
            Strategy::Rush => Self::Rush,
            Strategy::Wide => Self::Wide,
            Strategy::Science => Self::Science,
            Strategy::Turtle => Self::Turtle,
            Strategy::Economic => Self::Economic,
        }
    }
}

/// Hook for real diplomacy events, not embassy buildings or faction names. Unavailable
/// until an observer can measure both fields from the diplomacy implementation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyEvidence {
    pub allied_turns: u32,
    pub coalitions_led: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub early_attacks: u32,
    pub early_founded: u32,
    pub early_peak_cities: u32,
    pub early_military_cost: u64,
    pub early_total_cost: u64,
    pub development_total_cost: u64,
    pub science_cost: u64,
    pub market_cost: u64,
    pub defense_cost: u64,
    pub great_works: u32,
    /// Last alive observation up to the development horizon, preserved after elimination.
    pub development_turn: u32,
    pub cities: u32,
    pub population: i32,
    pub production: i32,
    pub science: i32,
    /// Gross city gold yield, before empire upkeep.
    pub gold: i32,
    pub laboratories: u32,
    pub markets: u32,
    pub walls: u32,
    pub diplomacy: Option<DiplomacyEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyTelemetry {
    pub classifier_version: u32,
    pub early_end_turn: u32,
    pub development_end_turn: u32,
    pub empires: BTreeMap<u8, Evidence>,
}

/// First-match precedence is intentional and versioned. Never reads faction, assigned
/// strategy, final rank, or the winner. See docs/strategy-diversity.md for the thresholds.
pub fn classify(e: &Evidence) -> Archetype {
    let share = |part: u64, whole: u64, pct: u64| {
        whole > 0 && u128::from(part) * 100 >= u128::from(whole) * u128::from(pct)
    };
    let cities = i64::from(e.cities);
    if e.diplomacy
        .as_ref()
        .is_some_and(|d| d.allied_turns >= 20 && d.coalitions_led >= 1)
    {
        Archetype::Diplomatic
    } else if e.early_attacks >= 3
        && e.early_military_cost >= 120
        && share(e.early_military_cost, e.early_total_cost, 50)
    {
        Archetype::Rush
    } else if e.early_founded >= 4 {
        Archetype::Wide
    } else if e.laboratories >= 2 && share(e.science_cost, e.development_total_cost, 25) {
        Archetype::Science
    } else if e.walls >= 2
        && (e.great_works >= 1 || share(e.defense_cost, e.development_total_cost, 10))
    {
        Archetype::Turtle
    } else if e.markets >= 2
        && (share(e.market_cost, e.development_total_cost, 20) || i64::from(e.gold) >= 6 * cities)
    {
        Archetype::Economic
    } else {
        Archetype::Generalist
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collector {
    pub telemetry: StrategyTelemetry,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            telemetry: StrategyTelemetry {
                classifier_version: CLASSIFIER_VERSION,
                early_end_turn: 120,
                development_end_turn: 240,
                empires: BTreeMap::new(),
            },
        }
    }

    pub fn observe_event(&mut self, turn: u32, event: &Event) {
        let early = turn <= self.telemetry.early_end_turn;
        if turn > self.telemetry.development_end_turn {
            return;
        }
        match event {
            Event::Battle {
                attacker_owner,
                meeting: false,
                ..
            } if early => {
                self.telemetry
                    .empires
                    .entry(attacker_owner.0)
                    .or_default()
                    .early_attacks += 1;
            }
            Event::CityFounded { owner, .. } if early => {
                self.telemetry
                    .empires
                    .entry(owner.0)
                    .or_default()
                    .early_founded += 1;
            }
            Event::ProductionCompleted { owner, item, .. } => {
                let e = self.telemetry.empires.entry(owner.0).or_default();
                // Listed costs deliberately avoid changing price/state/tape plumbing.
                // Includes purchases once, on completion; excludes starting/captured units.
                let cost = match item {
                    ProductionItem::Unit { unit } => data::unit_stats(*unit).cost,
                    ProductionItem::Building { building }
                    | ProductionItem::Wonder { building, .. } => {
                        data::building_stats(*building).cost
                    }
                    ProductionItem::Project { .. } => 0,
                    ProductionItem::Improvement { improvement, .. } => {
                        legatum_core::improvements::base_cost(*improvement)
                    }
                }
                .max(0) as u64;
                e.development_total_cost += cost;
                if early {
                    e.early_total_cost += cost;
                }
                match item {
                    ProductionItem::Unit { unit }
                        if !matches!(unit, UnitKind::Scout | UnitKind::Settler) =>
                    {
                        if early {
                            e.early_military_cost += cost;
                        }
                    }
                    ProductionItem::Building { building } => {
                        match data::building_family(*building) {
                            BuildingKind::Laboratory => e.science_cost += cost,
                            BuildingKind::Market => e.market_cost += cost,
                            BuildingKind::Walls => e.defense_cost += cost,
                            _ => {}
                        }
                        if data::is_great_work(*building) {
                            e.great_works += 1;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn observe_resolved_turn(&mut self, turn: u32, state: &GameState) {
        for p in &state.players {
            let e = self.telemetry.empires.entry(p.id.0).or_default();
            if turn > self.telemetry.development_end_turn || !p.alive {
                continue;
            }
            e.development_turn = turn;
            e.cities = 0;
            e.population = 0;
            e.production = 0;
            e.science = 0;
            e.gold = 0;
            e.laboratories = 0;
            e.markets = 0;
            e.walls = 0;
            for city in state.cities_of(p.id) {
                e.cities += 1;
                e.population += city.population;
                let y = economy::city_yields(state, city).total;
                e.production += y.production;
                e.science += y.science;
                e.gold += y.gold;
                e.laboratories += u32::from(data::has_building(
                    &city.buildings,
                    BuildingKind::Laboratory,
                ));
                e.markets += u32::from(data::has_building(&city.buildings, BuildingKind::Market));
                e.walls += u32::from(data::has_building(&city.buildings, BuildingKind::Walls));
            }
            if turn <= self.telemetry.early_end_turn {
                e.early_peak_cities = e.early_peak_cities.max(e.cities);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmpireResult {
    pub team: u8,
    pub faction: Faction,
    pub archetype: Option<Archetype>,
    pub won: bool,
    pub alive: bool,
    pub vp: u32,
    pub eliminated_turn: Option<u32>,
}

fn finish_cmp(a: &EmpireResult, b: &EmpireResult) -> Ordering {
    let rank = |p: &EmpireResult| {
        (
            p.won,
            p.alive,
            if p.alive {
                p.vp
            } else {
                p.eliminated_turn.unwrap_or(0)
            },
        )
    };
    rank(a).cmp(&rank(b))
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Frequency {
    pub seats: u64,
    pub wins: u64,
    pub win_rate: Option<f64>,
    pub player_frequency: Option<f64>,
    pub winner_frequency: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Matchup {
    pub games: u64,
    pub player_pairs: u64,
    pub ahead_pairs: u64,
    pub tied_pairs: u64,
    pub behind_pairs: u64,
    /// Each co-occurrence game has weight one, even with unequal numbers of each archetype.
    pub ahead_rate: Option<f64>,
    pub tie_rate: Option<f64>,
    pub score: Option<f64>,
}

impl Matchup {
    fn observe(&mut self, ahead: u64, tied: u64, behind: u64) {
        let pairs = ahead + tied + behind;
        if pairs == 0 {
            return;
        }
        let average = |old: Option<f64>, value: f64| {
            Some((old.unwrap_or(0.0) * self.games as f64 + value) / (self.games + 1) as f64)
        };
        self.ahead_rate = average(self.ahead_rate, ahead as f64 / pairs as f64);
        self.tie_rate = average(self.tie_rate, tied as f64 / pairs as f64);
        self.score = average(
            self.score,
            (ahead as f64 + tied as f64 / 2.0) / pairs as f64,
        );
        self.games += 1;
        self.player_pairs += pairs;
        self.ahead_pairs += ahead;
        self.tied_pairs += tied;
        self.behind_pairs += behind;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DiversitySummary {
    pub archetypes: Vec<Archetype>,
    pub games: u64,
    pub excluded_unfinished_games: u64,
    pub excluded_missing_labels: u64,
    pub seats: u64,
    pub winners: u64,
    pub by_archetype: BTreeMap<Archetype, Frequency>,
    pub by_faction: BTreeMap<Faction, BTreeMap<Archetype, Frequency>>,
    pub matchups: BTreeMap<Archetype, BTreeMap<Archetype, Matchup>>,
    pub winner_entropy: Option<f64>,
}

impl DiversitySummary {
    pub fn new(archetypes: Vec<Archetype>) -> Self {
        Self {
            by_archetype: archetypes
                .iter()
                .map(|a| (*a, Frequency::default()))
                .collect(),
            matchups: archetypes
                .iter()
                .map(|a| {
                    (
                        *a,
                        archetypes
                            .iter()
                            .filter(|b| *b != a)
                            .map(|b| (*b, Matchup::default()))
                            .collect(),
                    )
                })
                .collect(),
            archetypes,
            ..Default::default()
        }
    }

    pub fn observe(&mut self, finished: bool, players: &[EmpireResult]) {
        if !finished {
            self.excluded_unfinished_games += 1;
            return;
        }
        if players.is_empty()
            || players
                .iter()
                .any(|p| p.archetype.is_none_or(|a| !self.archetypes.contains(&a)))
        {
            self.excluded_missing_labels += 1;
            return;
        }
        self.games += 1;
        for p in players {
            let a = p.archetype.expect("checked labels");
            self.seats += 1;
            self.winners += u64::from(p.won);
            for f in [
                self.by_archetype.get_mut(&a).unwrap(),
                self.by_faction
                    .entry(p.faction)
                    .or_default()
                    .entry(a)
                    .or_default(),
            ] {
                f.seats += 1;
                f.wins += u64::from(p.won);
            }
        }
        for (&a, row) in &mut self.matchups {
            for (&b, cell) in row {
                let (mut ahead, mut tied, mut behind) = (0, 0, 0);
                for x in players.iter().filter(|p| p.archetype == Some(a)) {
                    for y in players
                        .iter()
                        .filter(|p| p.archetype == Some(b) && p.team != x.team)
                    {
                        match finish_cmp(x, y) {
                            Ordering::Greater => ahead += 1,
                            Ordering::Equal => tied += 1,
                            Ordering::Less => behind += 1,
                        }
                    }
                }
                cell.observe(ahead, tied, behind);
            }
        }
    }

    pub fn finish(&mut self) {
        let set_rates = |f: &mut Frequency, seats: u64, wins: u64| {
            f.win_rate = ratio(f.wins, f.seats);
            f.player_frequency = ratio(f.seats, seats);
            f.winner_frequency = ratio(f.wins, wins);
        };
        for f in self.by_archetype.values_mut() {
            set_rates(f, self.seats, self.winners);
        }
        for row in self.by_faction.values_mut() {
            let seats = row.values().map(|f| f.seats).sum();
            let wins = row.values().map(|f| f.wins).sum();
            for f in row.values_mut() {
                set_rates(f, seats, wins);
            }
        }
        self.winner_entropy = normalized_entropy(
            &self
                .archetypes
                .iter()
                .map(|a| self.by_archetype[a].wins)
                .collect::<Vec<_>>(),
        );
    }
}

fn ratio(n: u64, d: u64) -> Option<f64> {
    (d > 0).then(|| n as f64 / d as f64)
}

/// Fixed supported vocabulary (including zero-win categories), not just observed winners.
/// No winning seats => unavailable; one possible category => zero diversity.
pub fn normalized_entropy(counts: &[u64]) -> Option<f64> {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return None;
    }
    if counts.len() <= 1 {
        return Some(0.0);
    }
    Some(
        (counts
            .iter()
            .filter(|n| **n > 0)
            .map(|n| {
                let p = *n as f64 / total as f64;
                -p * p.ln()
            })
            .sum::<f64>()
            / (counts.len() as f64).ln())
        .clamp(0.0, 1.0),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub min_entropy: f64,
    pub min_matchup_games: u64,
}
impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            min_entropy: DEFAULT_MIN_ENTROPY,
            min_matchup_games: DEFAULT_MIN_MATCHUP_GAMES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
    InsufficientData,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guardrail {
    pub status: CheckStatus,
    pub config: GuardrailConfig,
    pub reasons: Vec<String>,
    pub dominant: Vec<Archetype>,
    pub without_observed_counter: Vec<Archetype>,
    pub never_won: Vec<Archetype>,
    pub unobserved: Vec<Archetype>,
    pub missing_matchups: Vec<(Archetype, Archetype)>,
}

/// Descriptive alarm, not a causal/statistical proof. Ties count one half for superiority;
/// the matrix separately exposes strict ahead rates. Every category in this summary is
/// assessed, including Generalist in observed reports and zero-win categories.
pub fn check_diversity(s: &DiversitySummary, config: GuardrailConfig) -> Guardrail {
    let mut r = Guardrail {
        status: CheckStatus::Pass,
        config,
        reasons: vec![],
        dominant: vec![],
        without_observed_counter: vec![],
        never_won: vec![],
        unobserved: vec![],
        missing_matchups: vec![],
    };
    if !config.min_entropy.is_finite()
        || !(0.0..=1.0).contains(&config.min_entropy)
        || config.min_matchup_games == 0
    {
        r.status = CheckStatus::Fail;
        r.reasons.push("invalid guardrail configuration".into());
        return r;
    }
    let active = &s.archetypes;
    for &a in active {
        let f = &s.by_archetype[&a];
        if f.seats == 0 {
            r.unobserved.push(a);
        } else if f.wins == 0 {
            r.never_won.push(a);
        }
        let mut scores = vec![];
        for &b in active.iter().filter(|b| **b != a) {
            let m = s.matchups.get(&a).and_then(|row| row.get(&b));
            if m.is_none_or(|m| m.games < config.min_matchup_games || m.score.is_none()) {
                if a < b {
                    r.missing_matchups.push((a, b));
                }
            } else {
                scores.push(m.unwrap().score.unwrap());
            }
        }
        if active.len() > 1 && scores.len() == active.len() - 1 {
            if scores.iter().all(|score| *score > 0.5 + 1e-12) {
                r.dominant.push(a);
            }
            if scores.iter().all(|score| *score >= 0.5 - 1e-12) {
                r.without_observed_counter.push(a);
            }
        }
    }
    if !r.missing_matchups.is_empty() || active.len() < 2 || s.winner_entropy.is_none() {
        r.status = CheckStatus::InsufficientData;
        r.reasons.push(format!(
            "insufficient evidence: {} under-sampled matchups; {} winning seats",
            r.missing_matchups.len(),
            s.winners
        ));
    }
    if !r.dominant.is_empty() {
        r.status = CheckStatus::Fail;
        r.reasons
            .push("an archetype scores above 50% against every other sampled strategy".into());
    }
    if s.winner_entropy.is_some_and(|h| h < config.min_entropy) {
        r.status = CheckStatus::Fail;
        r.reasons
            .push(format!("winner entropy below {:.2}", config.min_entropy));
    }
    r
}

fn percent(v: Option<f64>) -> String {
    v.map_or_else(|| "n/a".into(), |v| format!("{:.1}%", v * 100.0))
}

pub fn render_markdown(s: &DiversitySummary, check: &Guardrail, label: &str) -> String {
    let mut o = format!(
        "\n## Strategy diversity: {label}\n\n{} finished games; {} player seats; {} winning seats. Excluded {} unfinished/panicked games and {} games with missing labels. Winner entropy: **{}** (normalized over {} categories, including zero-win categories).\n\n",
        s.games,
        s.seats,
        s.winners,
        s.excluded_unfinished_games,
        s.excluded_missing_labels,
        s.winner_entropy
            .map_or_else(|| "n/a".into(), |h| format!("{h:.4}")),
        s.archetypes.len()
    );
    o += "Thresholds, biases and denominators: [strategy-diversity.md](../strategy-diversity.md).\n\n| Archetype | Players | Wins | Win rate | All-player frequency | Winner frequency |\n| --- | ---: | ---: | ---: | ---: | ---: |\n";
    for a in &s.archetypes {
        let f = &s.by_archetype[a];
        o += &format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            a.name(),
            f.seats,
            f.wins,
            percent(f.win_rate),
            percent(f.player_frequency),
            percent(f.winner_frequency)
        );
    }
    o += "\n### Matchup matrix\n\nCell = strict ahead % / tied % (co-occurrence games). Each game has equal weight; all cross-team seat pairs within it share that weight. Rank: winning team first, then surviving empires by VP, then eliminated empires by later elimination turn. Equal ranks tie; no ID tiebreak. This is a multiplayer association, not a duel win rate.\n\n| Row vs column |";
    for a in &s.archetypes {
        o += &format!(" {} |", a.name());
    }
    o += "\n| --- |";
    for _ in &s.archetypes {
        o += " ---: |";
    }
    o += "\n";
    for a in &s.archetypes {
        o += &format!("| {} |", a.name());
        for b in &s.archetypes {
            if a == b {
                o += " — |";
            } else {
                let m = &s.matchups[a][b];
                o += &format!(
                    " {} / {} ({}) |",
                    percent(m.ahead_rate),
                    percent(m.tie_rate),
                    m.games
                );
            }
        }
        o += "\n";
    }
    o += "\n### Faction wins by archetype\n\nCell = wins / seats; win rate in JSON.\n\n| Faction |";
    for a in &s.archetypes {
        o += &format!(" {} |", a.name());
    }
    o += "\n| --- |";
    for _ in &s.archetypes {
        o += " ---: |";
    }
    o += "\n";
    for (f, row) in &s.by_faction {
        o += &format!("| {f:?} |");
        for a in &s.archetypes {
            let v = row.get(a).cloned().unwrap_or_default();
            o += &format!(" {} / {} |", v.wins, v.seats);
        }
        o += "\n";
    }
    let names = |a: &[Archetype]| {
        if a.is_empty() {
            "none".into()
        } else {
            a.iter().map(|a| a.name()).collect::<Vec<_>>().join(", ")
        }
    };
    o += &format!(
        "\nGuardrail: **{:?}**. Entropy minimum {:.2}; at least {} games per strategy pair. Dominant by observed score: {}. Without an observed counter (all pairs covered): {}. Sampled but never won: {}. Unobserved: {}. Under-sampled pairs: {}.\n",
        check.status,
        check.config.min_entropy,
        check.config.min_matchup_games,
        names(&check.dominant),
        names(&check.without_observed_counter),
        names(&check.never_won),
        names(&check.unobserved),
        check.missing_matchups.len()
    );
    for reason in &check.reasons {
        o += &format!("\n- {reason}\n");
    }
    o
}
