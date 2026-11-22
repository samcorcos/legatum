use crate::Config;
use legatum_core::{AiDifficulty, Faction, Hex, UnitKind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Write, fs, path::Path};

pub const VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Behavior {
    UnansweredAttack,
    ScoutNotFleeing,
    GarrisonUnderRangedFire,
    IdleMilitaryThreatenedCity,
    RepeatedBounce,
    MovementOscillation,
    GoldHoardedUnderAttack,
    SettlerWandering,
    ResearchIdle,
    ProductionIdle,
}

impl Behavior {
    pub const ALL: [Self; 10] = [
        Self::UnansweredAttack,
        Self::ScoutNotFleeing,
        Self::GarrisonUnderRangedFire,
        Self::IdleMilitaryThreatenedCity,
        Self::RepeatedBounce,
        Self::MovementOscillation,
        Self::GoldHoardedUnderAttack,
        Self::SettlerWandering,
        Self::ResearchIdle,
        Self::ProductionIdle,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::UnansweredAttack => "unanswered_attack",
            Self::ScoutNotFleeing => "scout_not_fleeing",
            Self::GarrisonUnderRangedFire => "garrison_under_ranged_fire",
            Self::IdleMilitaryThreatenedCity => "idle_military_threatened_city",
            Self::RepeatedBounce => "repeated_bounce",
            Self::MovementOscillation => "movement_oscillation",
            Self::GoldHoardedUnderAttack => "gold_hoarded_under_attack",
            Self::SettlerWandering => "settler_wandering",
            Self::ResearchIdle => "research_idle",
            Self::ProductionIdle => "production_idle",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LocalUnit {
    pub unit: u32,
    pub player: u8,
    pub faction: Faction,
    pub kind: UnitKind,
    pub at: Hex,
    pub hp: i32,
    pub hostile: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LocalCity {
    pub city: u32,
    pub at: Hex,
    pub hp: i32,
    pub producing: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Context {
    pub at: Option<Hex>,
    pub hp_before: Option<i32>,
    pub hp_after: Option<i32>,
    pub orders: Vec<String>,
    pub nearby_visible_units: Vec<LocalUnit>,
    pub nearby_units_omitted: usize,
    pub nearby_own_cities: Vec<LocalCity>,
    pub safe_step: Option<Hex>,
    pub counterattacks: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Incident {
    pub id: String,
    pub turn: u32,
    pub player: u8,
    pub faction: Faction,
    pub behavior: Behavior,
    pub unit: Option<u32>,
    pub city: Option<u32>,
    pub consecutive_turns: u32,
    pub evidence: String,
    /// Outcome attribution, not a claim that the AI saw these attackers before choosing orders.
    pub attacker_factions: Vec<Faction>,
    pub context: Context,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Counts {
    pub opportunities: u64,
    pub incidents: u64,
}
impl Counts {
    pub fn percent(&self) -> Option<f64> {
        (self.opportunities > 0).then(|| self.incidents as f64 * 100.0 / self.opportunities as f64)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Rate {
    pub faction: Faction,
    pub behavior: Behavior,
    /// Some = exposures involving this opposing faction; None = all exposures.
    pub attacker_faction: Option<Faction>,
    #[serde(flatten)]
    pub counts: Counts,
    pub incident_percent: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Turn {
    pub turn: u32,
    pub events: String,
    pub incidents: Vec<String>,
    /// Reconstructed from the verified replay, including turns without incident flags.
    pub players: Vec<PlayerTurn>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerTurn {
    pub player: u8,
    pub cities: usize,
    pub population: i32,
    pub techs: usize,
    pub vp: u32,
    pub gold: i32,
    pub alive: bool,
    /// Whitelisted resolved events, with no names or free-form archive text.
    pub outcomes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Seat {
    pub player: u8,
    pub faction: Faction,
    pub human: bool,
    pub won: bool,
    pub difficulty: AiDifficulty,
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub detector_version: u32,
    pub game_id: String,
    pub engine_commit: String,
    pub rules_fingerprint: String,
    pub complete_history: bool,
    pub first_turn: u32,
    pub finished: bool,
    pub has_winner: bool,
    pub map_size: [i32; 2],
    pub config: Config,
    pub seats: Vec<Seat>,
    pub turns: Vec<Turn>,
    pub incidents: Vec<Incident>,
    pub rates: Vec<Rate>,
}

impl Report {
    /// Full facts remain in JSON. At most two local examples per turn and a strict byte budget;
    /// skipped turns/examples are explicit so a long game never silently becomes an empty review.
    pub fn narrative(&self, budget: usize) -> String {
        let mut out = format!(
            "Game {} | detector v{} | verified {} turns from T{} | complete history: {}\nRules {} | engine {}\nObservational flags, not proof of bad decisions. Positions and alternatives use pre-order player vision.\n",
            self.game_id,
            self.detector_version,
            self.turns.len(),
            self.first_turn,
            self.complete_history,
            self.rules_fingerprint,
            self.engine_commit
        );
        let _ = writeln!(
            out,
            "Seats: {}",
            self.seats
                .iter()
                .map(|p| format!(
                    "P{} {:?} {}{}",
                    p.player,
                    p.faction,
                    if p.human { "human" } else { "AI" },
                    if p.won { " winner" } else { "" }
                ))
                .collect::<Vec<_>>()
                .join("; ")
        );
        let by_id: BTreeMap<_, _> = self.incidents.iter().map(|i| (i.id.as_str(), i)).collect();
        let mut omitted_turns = 0;
        for turn in &self.turns {
            let mut line = format!(
                "T{}: {} | {} incident(s)\n",
                turn.turn,
                turn.events,
                turn.incidents.len()
            );
            for id in turn.incidents.iter().take(2) {
                let i = by_id[id.as_str()];
                let nearby = i
                    .context
                    .nearby_visible_units
                    .iter()
                    .take(3)
                    .map(|u| {
                        format!(
                            "P{} {:?}#{} {:?} hp{}",
                            u.player, u.kind, u.unit, u.at, u.hp
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let _ = writeln!(
                    line,
                    "  {} P{} {} u{:?}/c{:?}: {} | orders {} | nearby {}",
                    i.id,
                    i.player,
                    i.behavior.name(),
                    i.unit,
                    i.city,
                    i.evidence,
                    i.context.orders.join(","),
                    nearby
                );
            }
            for player in &turn.players {
                if !player.outcomes.is_empty() {
                    let _ = writeln!(
                        line,
                        "  P{}: {} | cities {}, population {}, techs {}, VP {}, gold {}",
                        player.player,
                        player.outcomes.join("; "),
                        player.cities,
                        player.population,
                        player.techs,
                        player.vp,
                        player.gold
                    );
                }
            }
            if out.len() + line.len() + 160 > budget {
                omitted_turns += 1;
            } else {
                out.push_str(&line);
            }
        }
        let _ = writeln!(
            out,
            "Total incidents: {}. Omitted turn lines: {omitted_turns}; all turns/incidents and local context are in incidents JSON.",
            self.incidents.len()
        );
        out
    }
}

/// Rate denominators are rule-specific entity-turn opportunities (see docs/ai-analysis.md).
/// Sum counts across games before dividing; never average game percentages.
#[derive(Default, Serialize)]
pub struct Aggregate {
    pub games: usize,
    pub seats_by_faction: BTreeMap<Faction, u64>,
    pub wins_by_faction: BTreeMap<Faction, u64>,
    pub rates: Vec<Rate>,
}
impl Aggregate {
    pub fn add(&mut self, report: &Report) {
        self.games += 1;
        for seat in report.seats.iter().filter(|s| !s.human) {
            *self.seats_by_faction.entry(seat.faction).or_default() += 1;
            *self.wins_by_faction.entry(seat.faction).or_default() += u64::from(seat.won);
        }
        let mut counts: BTreeMap<_, Counts> = self
            .rates
            .iter()
            .map(|r| {
                (
                    (r.faction, r.behavior, r.attacker_faction),
                    r.counts.clone(),
                )
            })
            .collect();
        for r in &report.rates {
            let c = counts
                .entry((r.faction, r.behavior, r.attacker_faction))
                .or_default();
            c.opportunities += r.counts.opportunities;
            c.incidents += r.counts.incidents;
        }
        self.rates = rates(counts);
    }
    pub fn markdown(&self) -> String {
        let mut out = format!(
            "\n## AI behavior analysis ({} verified games)\n\nIncidents / eligible entity-turns. Overlapping rules are not additive; these rates do not establish causes of wins.\n\n| AI faction | Behavior | Attacker faction | Incidents | Opportunities | Rate |\n| --- | --- | --- | ---: | ---: | ---: |\n",
            self.games
        );
        for r in &self.rates {
            let pct = r
                .incident_percent
                .map_or_else(|| "n/a".into(), |v| format!("{v:.2}%"));
            let _ = writeln!(
                out,
                "| {:?} | {} | {} | {} | {} | {} |",
                r.faction,
                r.behavior.name(),
                r.attacker_faction
                    .map_or_else(|| "all".into(), |f| format!("{f:?}")),
                r.counts.incidents,
                r.counts.opportunities,
                pct
            );
        }
        out
    }
}

pub(crate) fn rates(counts: BTreeMap<(Faction, Behavior, Option<Faction>), Counts>) -> Vec<Rate> {
    counts
        .into_iter()
        .map(|((faction, behavior, attacker_faction), counts)| Rate {
            faction,
            behavior,
            attacker_faction,
            incident_percent: counts.percent(),
            counts,
        })
        .collect()
}

pub fn write_report(report: &Report, out: &Path) -> Result<serde_json::Value, String> {
    if report.game_id.is_empty()
        || !report
            .game_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid analysis game id".into());
    }
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let incidents = out.join(format!("{}.incidents.json", report.game_id));
    let narrative = out.join(format!("{}.narrative.md", report.game_id));
    for (path, content) in [
        (
            &incidents,
            serde_json::to_string_pretty(report).map_err(|e| e.to_string())?,
        ),
        (&narrative, report.narrative(64_000)),
    ] {
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temp, content).map_err(|e| e.to_string())?;
        fs::rename(temp, path).map_err(|e| e.to_string())?;
    }
    Ok(
        serde_json::json!({ "game_id": report.game_id, "incidents": incidents, "narrative": narrative, "incident_count": report.incidents.len() }),
    )
}
