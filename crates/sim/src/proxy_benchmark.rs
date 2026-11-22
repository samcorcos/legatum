//! Observer-only competitiveness metrics. None of this state reaches a brain.

use crate::stats::{GameRecord, Outcome};
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, GameState, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DEFINITIONS: &str = "Proxy win rate uses all requested games (panic/stall/draw are not wins). Winning-turn median uses wins only, averaging the central pair for even samples. City/tech snapshots are after resolving the named turn; games ending earlier are excluded, never filled with zeros or carried forward. AI cities alive is the total across the seven original opponents. AI empires alive counts surviving original opponents. Proxy tech lead subtracts the largest retained tech count among those seven seats, not their average. AI techs at 150 includes those opponents already eliminated (their retained knowledge). Foundings count CityFounded, including capitals but excluding captures, per original AI seat across observed play. Gold is the eliminated AI's treasury after its elimination turn resolves; survivors are excluded. Proxy captures count CityCaptured to the Proxy, including recaptures. Event counts use first plays only. All snapshot and distribution denominators are reported. AI incidents use the master ai-game-analysis detector, default configuration (settler threshold 12 turns), after hash-verified tape replay; exclude the Proxy. Rules overlap; incident categories must not be added as independent failures. Proxy incidents are reported separately, with their own coverage. Per-game diagnostics record first attack/capture, founding and research turns, completed production, destroyed units, disbands and cities lost. Snapshot diagnostics use the resolved state. Null means analysis was unavailable or failed, not zero incidents. Missing policy versions remain unversioned; they are never assigned the current version.";

/// How Proxy benchmark games begin. Proxy v2 has no diplomacy and was calibrated when
/// every game was at war; under Peace starts it can only fight AIs that declare on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ProxyStart {
    /// Legacy all-war ruleset (no diplomacy), as in the Proxy v2 calibration.
    War,
    /// The current rules: every team starts at Peace.
    Peace,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Config {
    pub seat: u8,
    pub rotate: bool,
    pub difficulty: AiDifficulty,
    pub start: ProxyStart,
    pub version: u32,
    /// Run a tuned opening through the identical seat/faction schedule and observer.
    pub opening: Option<legatum_ai::openings::Opening>,
}

impl Config {
    pub fn seat_for(self, index: usize) -> u8 {
        if self.rotate {
            (usize::from(self.seat) + index % 8) as u8 % 8
        } else {
            self.seat
        }
    }

    pub fn configure(self, setup: &mut GameSetup, index: usize) {
        setup.legacy_all_war = self.start == ProxyStart::War;
        for player in &mut setup.players {
            player.difficulty = self.difficulty;
        }
        if self.rotate {
            // Each 72-game block covers every (seat, faction) exactly once. The diagonal
            // cycle also balances both marginal distributions in a partial block.
            let faction = Faction::ALL[(index % 8 + index / 8) % Faction::ALL.len()];
            let source = setup.players.iter().position(|p| p.faction == faction);
            let seat = usize::from(self.seat_for(index));
            let old = setup.players[seat].faction;
            setup.players[seat].faction = faction;
            if let Some(source) = source {
                setup.players[source].faction = old;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub ai_cities_alive: u32,
    pub proxy_cities: u32,
    pub proxy_techs: u32,
    pub ai_techs: BTreeMap<u8, u32>,
    #[serde(default)]
    pub diagnostics: Option<SnapshotDiagnostics>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDiagnostics {
    pub ai_empires_alive: u32,
    pub proxy_gold: i32,
    pub proxy_science_per_turn: i32,
    pub proxy_gold_per_turn: i32,
    pub proxy_units: BTreeMap<legatum_core::UnitKind, u32>,
    pub proxy_cities_founded: u32,
    pub proxy_captures: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    /// Absent in the original, unversioned reports; never infer a historical policy version.
    #[serde(default)]
    pub proxy_version: Option<u32>,
    #[serde(default)]
    pub benchmark_brain: Option<String>,
    pub seat: u8,
    pub faction: Faction,
    pub snapshots: BTreeMap<u32, Snapshot>,
    pub ai_cities_founded: BTreeMap<u8, u32>,
    pub ai_gold_at_elimination: BTreeMap<u8, i32>,
    pub proxy_cities_founded: u32,
    pub proxy_captures: u32,
    pub ai_incidents: Option<BTreeMap<String, u32>>,
    pub detector_version: Option<u32>,
    #[serde(default)]
    pub diagnostics: Option<Diagnostics>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Diagnostics {
    pub first_attack: Option<u32>,
    pub first_capture: Option<u32>,
    pub capture_turns: Vec<u32>,
    pub settlers_captured: u32,
    pub settlers_lost: u32,
    pub military_purchase_gold: i32,
    pub military_purchases: BTreeMap<u32, u32>,
    pub first_war: Option<u32>,
    pub peak_gold: i32,
    pub failed_orders: BTreeMap<String, u32>,
    pub city_fire_hits: u32,
    pub city_fire_kills: u32,
    /// Rolled damage, including overkill; do not interpret as net HP removed.
    pub city_fire_damage: i32,
    /// Distinct city/turn pairs reduced to zero by this seat, including repeated sieges.
    pub cities_zeroed_turns: u32,
    pub zeroed_without_capture_turns: u32,
    pub first_research: BTreeMap<legatum_core::TechId, u32>,
    pub foundings: Vec<u32>,
    pub units_lost: BTreeMap<legatum_core::UnitKind, u32>,
    pub disbands: BTreeMap<String, u32>,
    pub cities_lost: u32,
    pub production: BTreeMap<String, u32>,
    pub incidents: BTreeMap<String, u32>,
    /// Resolved diplomatic events involving the Proxy, not merely attempted orders.
    pub diplomacy: Option<BTreeMap<String, u32>>,
}

impl Game {
    pub fn new(seat: u8, state: &GameState, version: u32) -> Self {
        Self {
            proxy_version: Some(version),
            benchmark_brain: Some(format!("proxy-v{version}")),
            seat,
            faction: state.player(PlayerId(seat)).faction,
            snapshots: BTreeMap::new(),
            ai_cities_founded: state
                .players
                .iter()
                .filter(|p| p.id.0 != seat)
                .map(|p| (p.id.0, 0))
                .collect(),
            ai_gold_at_elimination: BTreeMap::new(),
            proxy_cities_founded: 0,
            proxy_captures: 0,
            ai_incidents: None,
            detector_version: None,
            diagnostics: Some(Diagnostics {
                diplomacy: Some(BTreeMap::new()),
                ..Default::default()
            }),
        }
    }

    pub fn observe_incidents(&mut self, report: &legatum_analysis::Report) {
        if let Some(d) = &mut self.diagnostics {
            for incident in report.incidents.iter().filter(|i| i.player == self.seat) {
                *d.incidents
                    .entry(incident.behavior.name().into())
                    .or_default() += 1;
            }
        }
        let mut counts: BTreeMap<String, u32> = legatum_analysis::Behavior::ALL
            .into_iter()
            .map(|b| (b.name().into(), 0))
            .collect();
        for incident in report.incidents.iter().filter(|i| i.player != self.seat) {
            *counts.entry(incident.behavior.name().into()).or_default() += 1;
        }
        self.ai_incidents = Some(counts);
        self.detector_version = Some(report.detector_version);
    }

    pub fn observe(&mut self, turn: u32, state: &GameState, events: &[Event]) {
        if let Some(d) = &mut self.diagnostics {
            d.peak_gold = d.peak_gold.max(state.player(PlayerId(self.seat)).gold);
            let zeroed: std::collections::BTreeSet<_> = events
                .iter()
                .filter_map(|e| match e {
                    Event::Battle {
                        attacker_owner,
                        defender: legatum_core::events::Defender::City { city, .. },
                        defender_hp_after,
                        ..
                    } if attacker_owner.0 == self.seat && *defender_hp_after <= 0 => Some(*city),
                    _ => None,
                })
                .collect();
            d.cities_zeroed_turns += zeroed.len() as u32;
            d.zeroed_without_capture_turns += zeroed
                .iter()
                .filter(|id| state.cities.get(id).is_some_and(|c| c.owner.0 != self.seat))
                .count() as u32;
        }
        for event in events {
            if let Some(d) = &mut self.diagnostics {
                let team = state.team_of(PlayerId(self.seat));
                let diplomatic_event = match event {
                    Event::WarDeclared { relation } if relation.a == team || relation.b == team => {
                        Some(if relation.declarers.contains(&team) {
                            "wars_declared"
                        } else {
                            "wars_received"
                        })
                    }
                    Event::PeaceEstablished { relation }
                        if relation.a == team || relation.b == team =>
                    {
                        Some("peace_established")
                    }
                    Event::TreatyActivated { relation, .. }
                        if relation.a == team || relation.b == team =>
                    {
                        Some("alliances_activated")
                    }
                    Event::WarJoined { relation, ally, .. }
                        if (relation.a == team || relation.b == team) && *ally != team =>
                    {
                        Some("wars_joined")
                    }
                    Event::WarJoined { ally, .. } if *ally == team => Some("allies_joined"),
                    Event::AllianceBetrayed { betrayal } if betrayal.by == team => {
                        Some("alliances_betrayed")
                    }
                    Event::TreatyBreached { by, .. } if *by == team => Some("treaties_breached"),
                    Event::ProposalCreated { proposal } if proposal.to == team => {
                        match proposal.kind {
                            legatum_core::diplomacy::TreatyKind::Alliance => {
                                Some("alliances_received")
                            }
                            _ => None,
                        }
                    }
                    Event::ProposalRejected { proposal, .. } if proposal.to == team => {
                        match proposal.kind {
                            legatum_core::diplomacy::TreatyKind::Alliance => {
                                Some("alliances_declined")
                            }
                            _ => None,
                        }
                    }
                    Event::ProposalCreated { proposal } if proposal.from == team => {
                        Some(match proposal.kind {
                            legatum_core::diplomacy::TreatyKind::Peace => "peace_proposed",
                            legatum_core::diplomacy::TreatyKind::Alliance => "alliances_proposed",
                        })
                    }
                    _ => None,
                };
                if let Some(key) = diplomatic_event
                    && let Some(counts) = &mut d.diplomacy
                {
                    *counts.entry(key.into()).or_default() += 1;
                }
                match event {
                    Event::OrderFailed { player, reason } if player.0 == self.seat => {
                        *d.failed_orders.entry(reason.clone()).or_default() += 1;
                    }
                    Event::Battle {
                        attacker: legatum_core::events::Attacker::City { .. },
                        defender: legatum_core::events::Defender::Unit { owner, .. },
                        damage_to_defender,
                        defender_hp_after,
                        ..
                    } if owner.0 == self.seat => {
                        d.city_fire_hits += 1;
                        d.city_fire_kills += u32::from(*defender_hp_after <= 0);
                        d.city_fire_damage += damage_to_defender;
                    }
                    Event::Battle { attacker_owner, .. } if attacker_owner.0 == self.seat => {
                        d.first_attack.get_or_insert(turn);
                    }
                    Event::CityCaptured { to, .. } if to.0 == self.seat => {
                        d.first_capture.get_or_insert(turn);
                        d.capture_turns.push(turn);
                    }
                    Event::CityCaptured { from, .. } if from.0 == self.seat => {
                        d.cities_lost += 1;
                    }
                    Event::CityFounded { owner, .. } if owner.0 == self.seat => {
                        d.foundings.push(turn)
                    }
                    Event::TechResearched { player, tech } if player.0 == self.seat => {
                        d.first_research.insert(*tech, turn);
                    }
                    Event::UnitDestroyed {
                        owner, unit_kind, ..
                    } if owner.0 == self.seat => {
                        *d.units_lost.entry(*unit_kind).or_default() += 1;
                    }
                    Event::UnitDisbanded { owner, reason, .. } if owner.0 == self.seat => {
                        *d.disbands.entry(reason.clone()).or_default() += 1;
                    }
                    Event::SettlerCaptured { to, .. } if to.0 == self.seat => {
                        d.settlers_captured += 1
                    }
                    Event::SettlerCaptured { from, .. } if from.0 == self.seat => {
                        d.settlers_lost += 1
                    }
                    Event::ItemBought {
                        owner,
                        item: legatum_core::ProductionItem::Unit { unit },
                        gold,
                        ..
                    } if owner.0 == self.seat && legatum_core::data::unit_stats(*unit).military => {
                        d.military_purchase_gold += *gold;
                        *d.military_purchases.entry(turn).or_default() += 1;
                    }
                    Event::WarDeclared { relation } if relation.declarers.contains(&team) => {
                        d.first_war.get_or_insert(turn);
                    }
                    Event::ProductionCompleted { owner, item, .. } if owner.0 == self.seat => {
                        *d.production.entry(format!("{item:?}")).or_default() += 1;
                    }
                    _ => {}
                }
            }
            match event {
                Event::CityFounded { owner, .. } if owner.0 == self.seat => {
                    self.proxy_cities_founded += 1
                }
                Event::CityFounded { owner, .. } => {
                    *self.ai_cities_founded.entry(owner.0).or_default() += 1
                }
                Event::CityCaptured { to, .. } if to.0 == self.seat => self.proxy_captures += 1,
                Event::PlayerEliminated { player } if player.0 != self.seat => {
                    self.ai_gold_at_elimination
                        .insert(player.0, state.player(*player).gold);
                }
                _ => {}
            }
        }
        if let Some(counts) = self.diagnostics.as_mut().and_then(|d| d.diplomacy.as_mut()) {
            let team = state.team_of(PlayerId(self.seat));
            if let Some(coalition) = legatum_core::diplomacy::coalition::from_state(state)
                && coalition.target == team
            {
                *counts.entry("coalition_target_turns".into()).or_default() += 1;
                if !coalition.at_war.is_empty() {
                    *counts.entry("coalition_war_turns".into()).or_default() += 1;
                }
            }
        }
        if [100, 136, 150, 200].contains(&turn) {
            let proxy = PlayerId(self.seat);
            let economy = legatum_core::economy::player_income(state, proxy);
            let mut proxy_units = BTreeMap::new();
            for unit in state.units_of(proxy) {
                *proxy_units.entry(unit.kind).or_default() += 1;
            }
            self.snapshots.insert(
                turn,
                Snapshot {
                    ai_cities_alive: state.cities.values().filter(|c| c.owner != proxy).count()
                        as u32,
                    proxy_cities: state.cities_of(proxy).count() as u32,
                    proxy_techs: state.player(proxy).techs.len() as u32,
                    ai_techs: state
                        .players
                        .iter()
                        .filter(|p| p.id != proxy)
                        .map(|p| (p.id.0, p.techs.len() as u32))
                        .collect(),
                    diagnostics: Some(SnapshotDiagnostics {
                        ai_empires_alive: state
                            .players
                            .iter()
                            .filter(|p| p.id != proxy && p.alive)
                            .count() as u32,
                        proxy_gold: state.player(proxy).gold,
                        proxy_science_per_turn: economy.science.total,
                        proxy_gold_per_turn: economy.gold.total,
                        proxy_units,
                        proxy_cities_founded: self.proxy_cities_founded,
                        proxy_captures: self.proxy_captures,
                    }),
                },
            );
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Metric {
    pub observations: usize,
    pub total: i64,
    pub mean: Option<f64>,
    pub median: Option<f64>,
}

impl Metric {
    fn of(values: impl Iterator<Item = i64>) -> Self {
        let mut v: Vec<_> = values.collect();
        v.sort();
        let n = v.len();
        let total = v.iter().sum();
        Self {
            observations: n,
            total,
            mean: (n > 0).then(|| total as f64 / n as f64),
            median: (n > 0).then(|| (v[(n - 1) / 2] + v[n / 2]) as f64 / 2.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Summary {
    pub proxy_versions: BTreeMap<u32, usize>,
    pub unversioned_games: usize,
    pub games: usize,
    pub observed_games: usize,
    pub wins: usize,
    pub win_rate: f64,
    pub win_rate_wilson_95: [f64; 2],
    pub winning_turn: Metric,
    pub ai_cities_alive: BTreeMap<u32, Metric>,
    pub proxy_cities: BTreeMap<u32, Metric>,
    pub proxy_techs: BTreeMap<u32, Metric>,
    pub proxy_tech_lead: BTreeMap<u32, Metric>,
    pub ai_empires_alive: BTreeMap<u32, Metric>,
    pub ai_cities_founded: Metric,
    pub ai_techs_turn_150: Metric,
    pub ai_gold_at_elimination: Metric,
    pub proxy_cities_founded: Metric,
    pub proxy_captures: Metric,
    pub seats: BTreeMap<u8, usize>,
    pub factions: BTreeMap<Faction, usize>,
    pub ai_incidents: Option<BTreeMap<String, u64>>,
    pub proxy_incidents: Option<BTreeMap<String, u64>>,
    pub proxy_incident_observed_games: usize,
    pub incident_observed_games: usize,
}

/// Two-sided Wilson score interval, all requested games in the denominator.
fn wilson_95(wins: usize, games: usize) -> [f64; 2] {
    if games == 0 {
        return [0.0, 1.0];
    }
    let n = games as f64;
    let p = wins as f64 / n;
    let z = 1.959963984540054;
    let denom = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denom;
    let half = z * ((p * (1.0 - p) + z * z / (4.0 * n)) / n).sqrt() / denom;
    [(center - half).max(0.0), (center + half).min(1.0)]
}

pub fn summarize(records: &[GameRecord]) -> Option<Summary> {
    let observed: Vec<_> = records
        .iter()
        .filter_map(|r| r.telemetry.as_ref()?.proxy.as_ref().map(|p| (r, p)))
        .collect();
    if observed.is_empty() {
        return None;
    }
    let wins: Vec<_> = observed
        .iter()
        .filter(|(r, p)| {
            !matches!(r.outcome, Outcome::Panic | Outcome::Stalled)
                && r.winner_players.contains(&p.seat)
        })
        .collect();
    let mut s = Summary {
        games: records.len(),
        observed_games: observed.len(),
        wins: wins.len(),
        win_rate: wins.len() as f64 / records.len() as f64,
        win_rate_wilson_95: wilson_95(wins.len(), records.len()),
        winning_turn: Metric::of(wins.iter().map(|(r, _)| i64::from(r.turns))),
        ai_cities_founded: Metric::of(
            observed
                .iter()
                .flat_map(|(_, p)| p.ai_cities_founded.values().map(|n| i64::from(*n))),
        ),
        ai_techs_turn_150: Metric::of(
            observed
                .iter()
                .filter_map(|(_, p)| p.snapshots.get(&150))
                .flat_map(|s| s.ai_techs.values().map(|n| i64::from(*n))),
        ),
        ai_gold_at_elimination: Metric::of(
            observed
                .iter()
                .flat_map(|(_, p)| p.ai_gold_at_elimination.values().map(|n| i64::from(*n))),
        ),
        proxy_cities_founded: Metric::of(
            observed
                .iter()
                .map(|(_, p)| i64::from(p.proxy_cities_founded)),
        ),
        proxy_captures: Metric::of(observed.iter().map(|(_, p)| i64::from(p.proxy_captures))),
        ..Default::default()
    };
    for turn in [100, 136, 150, 200] {
        s.ai_cities_alive.insert(
            turn,
            Metric::of(
                observed
                    .iter()
                    .filter_map(|(_, p)| p.snapshots.get(&turn))
                    .map(|s| i64::from(s.ai_cities_alive)),
            ),
        );
        s.proxy_cities.insert(
            turn,
            Metric::of(
                observed
                    .iter()
                    .filter_map(|(_, p)| p.snapshots.get(&turn))
                    .map(|s| i64::from(s.proxy_cities)),
            ),
        );
        s.proxy_techs.insert(
            turn,
            Metric::of(
                observed
                    .iter()
                    .filter_map(|(_, p)| p.snapshots.get(&turn))
                    .map(|s| i64::from(s.proxy_techs)),
            ),
        );
        s.proxy_tech_lead.insert(
            turn,
            Metric::of(
                observed
                    .iter()
                    .filter_map(|(_, p)| p.snapshots.get(&turn))
                    .filter_map(|s| {
                        s.ai_techs
                            .values()
                            .max()
                            .map(|n| i64::from(s.proxy_techs) - i64::from(*n))
                    }),
            ),
        );
        s.ai_empires_alive.insert(
            turn,
            Metric::of(
                observed
                    .iter()
                    .filter_map(|(_, p)| p.snapshots.get(&turn)?.diagnostics.as_ref())
                    .map(|s| i64::from(s.ai_empires_alive)),
            ),
        );
    }
    for (_, p) in &observed {
        if let Some(v) = p.proxy_version {
            *s.proxy_versions.entry(v).or_default() += 1;
        } else {
            s.unversioned_games += 1;
        }
        if let Some(counts) = &p.ai_incidents {
            s.incident_observed_games += 1;
            if let Some(d) = &p.diagnostics {
                s.proxy_incident_observed_games += 1;
                let counts = s.proxy_incidents.get_or_insert_with(|| {
                    legatum_analysis::Behavior::ALL
                        .into_iter()
                        .map(|b| (b.name().into(), 0))
                        .collect()
                });
                for (kind, n) in &d.incidents {
                    *counts.entry(kind.clone()).or_default() += u64::from(*n);
                }
            }
            for (kind, n) in counts {
                *s.ai_incidents
                    .get_or_insert_with(BTreeMap::new)
                    .entry(kind.clone())
                    .or_default() += u64::from(*n);
            }
        }
        *s.seats.entry(p.seat).or_default() += 1;
        *s.factions.entry(p.faction).or_default() += 1;
    }
    Some(s)
}

pub fn render(s: &Summary, config: Option<Config>) -> String {
    let mut out = format!(
        "## Human-proxy competitiveness\n\nProxy wins: **{}/{} ({:.1}%)**. Median winning turn: **{}**.\n\n",
        s.wins,
        s.games,
        s.win_rate * 100.0,
        number(s.winning_turn.median)
    );
    out += &format!(
        "95% Wilson win-rate interval: {:.1}%–{:.1}%.\n\n",
        s.win_rate_wilson_95[0] * 100.0,
        s.win_rate_wilson_95[1] * 100.0
    );
    if let Some(opening) = config.and_then(|c| c.opening) {
        out += &format!(
            "Benchmark seat uses tuned {} (no Proxy policy version).\n\n",
            opening.name()
        );
    }
    out += &format!(
        "Policy versions (version: games): `{:?}`; unversioned or named-opening games: {}.\n\n",
        s.proxy_versions, s.unversioned_games
    );
    if let Some(c) = config {
        out += &format!(
            "One Proxy vs seven {:?} scripted AIs, eight independent teams. Start: {}. {}\n\n",
            c.difficulty,
            match c.start {
                ProxyStart::War => "legacy all-war (no diplomacy), as calibrated",
                ProxyStart::Peace => "Peace (current diplomacy rules)",
            },
            if c.rotate {
                "Game i: seat = (initial seat + i) % 8; faction = Faction::ALL[(i % 8 + i / 8) % faction_count], swapped into the seeded shuffle (or replacing the seat when absent). Every 8 * faction_count games covers all seat/faction pairs."
            } else {
                "Fixed Proxy seat; faction follows the seeded shuffle."
            }
        );
    }
    out +=
        "| Metric | Observations | Mean | Median | Total |\n| --- | ---: | ---: | ---: | ---: |\n";
    let mut row = |name: &str, m: &Metric| {
        out += &format!(
            "| {name} | {} | {} | {} | {} |\n",
            m.observations,
            number(m.mean),
            number(m.median),
            m.total
        );
    };
    for (turn, m) in &s.ai_cities_alive {
        row(&format!("AI cities alive at {turn} (per lobby)"), m);
    }
    for (turn, m) in &s.ai_empires_alive {
        row(&format!("AI empires alive at {turn} (per lobby)"), m);
    }
    row("AI cities founded (per AI seat)", &s.ai_cities_founded);
    row("AI techs at 150 (per AI seat)", &s.ai_techs_turn_150);
    row(
        "AI gold unspent at elimination (per eliminated AI)",
        &s.ai_gold_at_elimination,
    );
    row("Captures by Proxy (per game)", &s.proxy_captures);
    row("Proxy cities founded (per game)", &s.proxy_cities_founded);
    for (turn, m) in &s.proxy_cities {
        row(&format!("Proxy cities at {turn} (per game)"), m);
    }
    for (turn, m) in &s.proxy_techs {
        row(&format!("Proxy techs at {turn} (per game)"), m);
    }
    for (turn, m) in &s.proxy_tech_lead {
        row(&format!("Proxy techs minus strongest AI at {turn}"), m);
    }
    out += &format!(
        "\n{DEFINITIONS}\n\nObserver coverage: {}/{} games.\n\n",
        s.observed_games, s.games
    );
    out += &format!(
        "AI incident coverage: {}/{} games.\n\n",
        s.incident_observed_games, s.games
    );
    if let Some(counts) = &s.ai_incidents {
        out += "| AI incident | Count |\n| --- | ---: |\n";
        for (kind, n) in counts {
            out += &format!("| {kind} | {n} |\n");
        }
        out += "\n";
    } else {
        out += "AI incident counts unavailable.\n\n";
    }
    out += &format!(
        "Proxy incident coverage: {}/{} games.\n\n",
        s.proxy_incident_observed_games, s.games
    );
    if let Some(counts) = &s.proxy_incidents {
        out += "| Proxy incident | Count |\n| --- | ---: |\n";
        for (kind, n) in counts {
            out += &format!("| {kind} | {n} |\n");
        }
        out += "\n";
    }
    out += &format!(
        "Proxy seat exposure: `{:?}`. Faction exposure: `{:?}`.\n\n",
        s.seats, s.factions
    );
    out
}

fn number(n: Option<f64>) -> String {
    n.map_or_else(|| "n/a".into(), |v| format!("{v:.2}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_interval_handles_zero_all_and_half_wins() {
        let zero = wilson_95(0, 400);
        let all = wilson_95(400, 400);
        let half = wilson_95(200, 400);
        assert!(zero[0].abs() < 1e-12 && (zero[1] - 0.0095123).abs() < 1e-6);
        assert!((all[0] - 0.9904877).abs() < 1e-6 && (all[1] - 1.0).abs() < 1e-12);
        assert!((half[0] - 0.4512345).abs() < 1e-7);
        assert!((half[0] + half[1] - 1.0).abs() < 1e-12);
        assert_eq!(wilson_95(0, 0), [0.0, 1.0]);
    }

    #[test]
    fn opening_comparator_has_identical_rotation_and_a_turn_136_snapshot() {
        let config = Config {
            seat: 0,
            rotate: true,
            difficulty: AiDifficulty::Hard,
            start: ProxyStart::Peace,
            version: 5,
            opening: None,
        };
        for index in 0..64 {
            let mut proxy = GameSetup::all_ai(3600001 + index as u64);
            let mut opening = proxy.clone();
            config.configure(&mut proxy, index);
            Config {
                opening: Some(legatum_ai::openings::Opening::GreedyExpand),
                ..config
            }
            .configure(&mut opening, index);
            assert_eq!(proxy, opening);
        }
        let state = legatum_core::new_game(&GameSetup::all_ai(3600001));
        let mut game = Game::new(0, &state, 5);
        game.observe(136, &state, &[]);
        assert!(game.snapshots.contains_key(&136));
    }

    #[test]
    fn proxy_start_selects_legacy_all_war_or_current_peace() {
        for (start, legacy) in [(ProxyStart::War, true), (ProxyStart::Peace, false)] {
            let c = Config {
                seat: 0,
                rotate: true,
                difficulty: AiDifficulty::Hard,
                start,
                version: legatum_ai::PROXY_VERSION,
                opening: None,
            };
            let mut setup = GameSetup::all_ai(700001);
            c.configure(&mut setup, 5);
            assert_eq!(setup.legacy_all_war, legacy);
            let state = legatum_core::new_game(&setup);
            // Legacy all-war means no diplomacy state: every other team is hostile.
            assert_eq!(state.diplomacy.is_none(), legacy);
            assert_eq!(state.at_war(PlayerId(0), PlayerId(1)), legacy);
            // The flag is part of the setup, so tapes replay into the same start.
            let json = serde_json::to_string(&setup).unwrap();
            assert_eq!(json.contains("legacy_all_war"), legacy);
            let back: GameSetup = serde_json::from_str(&json).unwrap();
            assert_eq!(legatum_core::new_game(&back).hash(), state.hash());
            assert!(render(&Summary::default(), Some(c)).contains(match start {
                ProxyStart::War => "legacy all-war",
                ProxyStart::Peace => "Start: Peace",
            }));
        }
    }

    #[test]
    fn rotation_covers_all_pairs_and_balances_full_cycles() {
        let c = Config {
            seat: 3,
            rotate: true,
            difficulty: AiDifficulty::Hard,
            start: ProxyStart::War,
            version: legatum_ai::PROXY_VERSION,
            opening: None,
        };
        let mut pairs = BTreeMap::new();
        let mut seats = BTreeMap::new();
        let mut factions = BTreeMap::new();
        for i in 0..(32 * Faction::ALL.len()) {
            let mut setup = GameSetup::all_ai(700001 + i as u64);
            c.configure(&mut setup, i);
            let seat = c.seat_for(i);
            let faction = setup.players[usize::from(seat)].faction;
            if i < 8 * Faction::ALL.len() {
                *pairs.entry((seat, faction)).or_insert(0) += 1;
            }
            *seats.entry(seat).or_insert(0) += 1;
            *factions.entry(faction).or_insert(0) += 1;
            assert!(
                setup
                    .players
                    .iter()
                    .all(|p| p.difficulty == AiDifficulty::Hard)
            );
            assert_eq!(
                setup
                    .players
                    .iter()
                    .map(|p| p.faction)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                8
            );
        }
        assert_eq!(pairs.len(), 8 * Faction::ALL.len());
        assert!(pairs.values().all(|n| *n == 1));
        assert!(seats.values().all(|n| *n == 4 * Faction::ALL.len()));
        assert!(factions.values().all(|n| *n == 32));
    }

    #[test]
    fn founding_capture_gold_and_snapshot_use_observed_events_only() {
        let mut state = legatum_core::new_game(&GameSetup::all_ai(12));
        let mut p = Game::new(0, &state, legatum_ai::PROXY_VERSION);
        state.player_mut(PlayerId(1)).gold = 181;
        let before = state.hash();
        p.observe(
            99,
            &state,
            &[
                Event::CityFounded {
                    city: legatum_core::CityId(7),
                    name: "Test".into(),
                    owner: PlayerId(1),
                    at: legatum_core::Hex::new(0, 0),
                    veteran: false,
                },
                Event::CityCaptured {
                    city: legatum_core::CityId(7),
                    from: PlayerId(1),
                    to: PlayerId(0),
                    bounty: None,
                },
                Event::PlayerEliminated {
                    player: PlayerId(1),
                },
            ],
        );
        assert!(p.snapshots.is_empty());
        assert_eq!(p.ai_cities_founded[&1], 1);
        assert_eq!(p.proxy_captures, 1);
        assert_eq!(p.diagnostics.as_ref().unwrap().first_capture, Some(99));
        assert_eq!(p.proxy_version, Some(legatum_ai::PROXY_VERSION));
        assert_eq!(p.proxy_cities_founded, 0);
        assert_eq!(p.ai_gold_at_elimination, BTreeMap::from([(1, 181)]));
        p.observe(150, &state, &[]);
        assert_eq!(p.snapshots[&150].ai_techs.len(), 7);
        assert_eq!(state.hash(), before);
        assert_eq!(Metric::of([2, 7].into_iter()).median, Some(4.5));
        assert_eq!(Metric::of([].into_iter()).median, None);
    }

    #[test]
    fn early_finishes_and_panics_do_not_fabricate_snapshots_or_wins() {
        let state = legatum_core::new_game(&GameSetup::all_ai(12));
        let mut r = GameRecord::panicked(0, 12, "placeholder".into(), 0);
        r.outcome = Outcome::Conquest;
        r.panic = None;
        r.turns = 99;
        r.winner_players = vec![0];
        let mut telemetry = crate::telemetry::Collector::default().finish(&state, 0);
        telemetry.proxy = Some(Game::new(0, &state, legatum_ai::PROXY_VERSION));
        r.telemetry = Some(telemetry);
        let mut later = r.clone();
        later.turns = 150;
        later
            .telemetry
            .as_mut()
            .unwrap()
            .proxy
            .as_mut()
            .unwrap()
            .observe(150, &state, &[]);
        let records = [r, later, GameRecord::panicked(2, 14, "panic".into(), 0)];
        let s = summarize(&records).unwrap();
        assert_eq!((s.games, s.observed_games, s.wins), (3, 2, 2));
        assert_eq!(s.winning_turn.median, Some(124.5));
        assert_eq!(s.ai_cities_alive[&100].observations, 0);
        assert_eq!(s.ai_cities_alive[&150].observations, 1);
        assert_eq!(s.ai_techs_turn_150.observations, 7);
        assert_eq!(s.ai_cities_founded.observations, 14);
        assert_eq!(s.ai_gold_at_elimination.observations, 0);
        assert_eq!(s.ai_incidents, None);
        assert_eq!(
            s.proxy_versions,
            BTreeMap::from([(legatum_ai::PROXY_VERSION, 2)])
        );
        assert_eq!(s.unversioned_games, 0);
        assert_eq!(s.proxy_tech_lead[&150].observations, 1);
        assert_eq!(s.proxy_incidents, None);
    }

    #[test]
    fn historical_reports_stay_unversioned_and_missing_diagnostics_stay_missing() {
        let state = legatum_core::new_game(&GameSetup::all_ai(12));
        let mut game = Game::new(0, &state, legatum_ai::PROXY_VERSION);
        game.observe(150, &state, &[]);
        let mut value = serde_json::to_value(game).unwrap();
        let fields = value.as_object_mut().unwrap();
        fields.remove("proxy_version");
        fields.remove("diagnostics");
        fields["snapshots"]["150"]
            .as_object_mut()
            .unwrap()
            .remove("diagnostics");
        let historical: Game = serde_json::from_value(value).unwrap();
        assert_eq!(historical.proxy_version, None);
        assert_eq!(historical.diagnostics, None);
        assert_eq!(historical.snapshots[&150].diagnostics, None);
    }

    #[test]
    fn diplomacy_observer_counts_alliances_betrayals_and_joined_wars() {
        use legatum_core::TeamId;
        use legatum_core::diplomacy::{Betrayal, RelationState};
        let state = legatum_core::new_game(&GameSetup::all_ai(12));
        let mut game = Game::new(0, &state, legatum_ai::PROXY_VERSION);
        let relation_of = |a: u8, b: u8| {
            state
                .diplomacy
                .as_ref()
                .unwrap()
                .relation(TeamId(a), TeamId(b))
                .unwrap()
                .clone()
        };
        let mut relation = relation_of(0, 1);
        relation.state = RelationState::Alliance;
        let alliance = Event::TreatyActivated { relation };
        // Team 0 joins ally 1's war on team 2, and ally 1 joins team 0's war on team 3.
        let joined_war = Event::WarJoined {
            relation: relation_of(0, 2),
            team: TeamId(0),
            ally: TeamId(1),
        };
        let ally_joined = Event::WarJoined {
            relation: relation_of(1, 3),
            team: TeamId(1),
            ally: TeamId(0),
        };
        game.observe(
            50,
            &state,
            &[
                alliance,
                joined_war,
                ally_joined,
                Event::AllianceBetrayed {
                    betrayal: Betrayal {
                        by: TeamId(0),
                        against: TeamId(1),
                        turn: 50,
                        until: 70,
                    },
                },
            ],
        );
        let counts = game.diagnostics.unwrap().diplomacy.unwrap();
        assert_eq!(counts["alliances_activated"], 1);
        assert_eq!(counts["wars_joined"], 1);
        assert_eq!(counts["allies_joined"], 1);
        assert_eq!(counts["alliances_betrayed"], 1);
    }

    #[test]
    fn tech_lead_compares_the_best_opponent_including_eliminated_seats() {
        let mut state = legatum_core::new_game(&GameSetup::all_ai(12));
        let techs = [
            legatum_core::TechId::Irrigation,
            legatum_core::TechId::BronzeWorking,
            legatum_core::TechId::Writing,
        ];
        state.player_mut(PlayerId(0)).techs.extend(techs);
        state.player_mut(PlayerId(1)).techs.extend(&techs[..2]);
        state.player_mut(PlayerId(1)).alive = false;
        let mut telemetry = crate::telemetry::Collector::default().finish(&state, 0);
        let mut game = Game::new(0, &state, legatum_ai::PROXY_VERSION);
        game.observe(150, &state, &[]);
        telemetry.proxy = Some(game);
        let mut record = GameRecord::panicked(0, 12, "unused outcome".into(), 0);
        record.telemetry = Some(telemetry);
        let summary = summarize(&[record]).unwrap();
        assert_eq!(summary.proxy_techs[&150].median, Some(3.0));
        assert_eq!(summary.proxy_tech_lead[&150].median, Some(1.0));
        assert_eq!(summary.ai_empires_alive[&150].median, Some(6.0));
    }
}
