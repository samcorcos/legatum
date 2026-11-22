//! Scenarios: preset training positions with one measurable objective, played by a human and
//! by the AI from the same state (repo `scenarios/`, `legatum-sim scenario`).
//!
//! Everything here reads game state only. The objective is judged from the scenario's start
//! state and the current state, so the app, the simulator and any later replay score a run the
//! same way. `validate_state` checks that an edited position is one the rules could reach.

use crate::data::{self, CITY_HP, UNIT_HP};
use crate::hex::Hex;
use crate::rules::is_military;
use crate::state::{City, GameState, Unit};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Something that is true or false of a position, from the scenario seat's point of view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// The seat owns every city it owned at the start.
    HoldStartingCities,
    /// The seat owns at least this many cities.
    MinCities(u32),
    /// The seat still owns at least this many of the cities it owned at the start. Unlike
    /// `MinCities`, a city founded or captured since does not make up for a lost one.
    MinStartingCities(u32),
    /// The seat has at least this many combat units (military units other than Scouts).
    MinCombatUnits(u32),
    /// The seat has this many units of one kind within the given radius of a city.
    MinUnitsNearCity {
        kind: UnitKind,
        city: CityId,
        radius: i32,
        count: u32,
    },
    /// Exactly these owned cities, and no other owned cities, produce this unit kind.
    ExactUnitProduction { kind: UnitKind, cities: Vec<CityId> },
    /// The seat owns this city.
    OwnCity(CityId),
    /// A rival owns at most this many cities.
    RivalMaxCities { rival: PlayerId, max: u32 },
    /// The seat owns at least `min` cities that `rival` founded (so cities taken from the rival,
    /// including ones it founds after the start; founding a city of its own does not count).
    CapturedFrom { rival: PlayerId, min: u32 },
    /// None of these units exists any more (killed in battle or disbanded).
    UnitsGone(Vec<UnitId>),
    /// This rival's team has not won the game (for a window that ends when its victory hold
    /// would complete: breaking the hold is the objective).
    StopWin { rival: PlayerId },
}

/// A number measured from the position; the score is a weighted sum of these.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Metric {
    /// Cities the seat owns.
    Cities,
    /// Cities owned at the start that the seat still owns.
    StartingCitiesHeld,
    /// Cities founded by the seat since the start.
    CitiesFounded,
    /// Cities the seat owns now that someone else owned at the start (or founded since).
    CitiesCaptured,
    /// Total population of the seat's cities.
    Population,
    /// Military units other than Scouts.
    CombatUnits,
    /// Victory points now.
    Vp,
    /// Victory points gained since the start.
    VpGained,
    /// 1 if the seat owns this city.
    OwnCity(CityId),
    /// Hit points this city has lost since the start, while someone else owns it.
    CityDamage(CityId),
    /// Turns left in the window when the objective was met early (0 otherwise).
    TurnsLeft,
    /// 1 when every win condition holds.
    ObjectiveMet,
    /// Cities a rival owns.
    RivalCities(PlayerId),
    /// Cities a rival gained since the start (never negative).
    RivalCitiesGained(PlayerId),
    /// Cities a rival lost since the start (never negative).
    RivalCitiesLost(PlayerId),
    /// A rival's military units other than Scouts.
    RivalCombatUnits(PlayerId),
    /// Victory points a rival gained since the start.
    RivalVpGained(PlayerId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreTerm {
    pub metric: Metric,
    pub weight: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objective {
    /// One sentence for the player, e.g. "Hold all 3 cities and keep 2 combat units".
    pub summary: String,
    /// Turns in the window. The scenario ends when the game reaches start turn + `turns`.
    pub turns: u32,
    /// All must hold at the end (or as soon as they all hold, with `end_on_win`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub win: Vec<Condition>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub end_on_win: bool,
    /// Must hold after every turn; breaking one ends the scenario as a loss.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keep: Vec<Condition>,
    pub score: Vec<ScoreTerm>,
}

impl Objective {
    pub fn end_turn(&self, start: &GameState) -> u32 {
        start.turn + self.turns
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Playing,
    /// Every win condition held when the scenario ended.
    Won,
    /// A keep condition broke, the seat was eliminated, or time ran out with a win condition unmet.
    Lost,
    /// A score-only scenario reached its last turn.
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionStatus {
    pub label: String,
    /// Current measurement, e.g. "2 of 3".
    pub progress: String,
    pub met: bool,
    /// A keep condition (must hold every turn) rather than a win condition.
    pub keep: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreLine {
    pub label: String,
    pub value: i32,
    pub weight: i32,
    pub points: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// The turn now being planned.
    pub turn: u32,
    pub start_turn: u32,
    pub end_turn: u32,
    pub turns_left: u32,
    pub outcome: Outcome,
    /// Why the scenario ended, or what is still needed.
    pub reason: String,
    pub score: i32,
    pub conditions: Vec<ConditionStatus>,
    pub score_lines: Vec<ScoreLine>,
    /// Where the objective is on the map (`objective_targets`).
    #[serde(default)]
    pub targets: Vec<Target>,
}

/// What the objective asks of a city.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    /// Capture it.
    Take,
    /// Keep it.
    Hold,
    /// One of the cities of an empire the objective contains.
    Contain,
}

/// A city the objective is about, marked for the player from the first turn even in fog:
/// knowing where the objective is was the difference between the user's first and second
/// runs of "The Courier King" (2026-10-04).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub kind: TargetKind,
    pub city: CityId,
    pub name: String,
    pub owner: PlayerId,
    pub pos: Hex,
    /// Rough bearing from the seat's nearest city, e.g. "6 tiles north-east of Tessel
    /// Market"; empty for the seat's own cities.
    pub bearing: String,
    /// Tiles from the seat's nearest city (0 for its own cities).
    pub distance: u32,
}

/// Compass point from `from` to `to` on the odd-r, pointy-top map (north is up).
fn compass(from: Hex, to: Hex) -> &'static str {
    let (dq, dr) = ((to.q - from.q) as i64, (to.r - from.r) as i64);
    // Pixel offsets times 1000: x = sqrt(3)/2 (2q + r), y = 3/2 r.
    let x = (2 * dq + dr) * 866;
    let y = dr * 1500;
    let (ax, ay) = (x.abs(), y.abs());
    let ns = if y < 0 { "north" } else { "south" };
    let ew = if x < 0 { "west" } else { "east" };
    if ay * 1000 >= ax * 2414 {
        ns
    } else if ax * 1000 >= ay * 2414 {
        ew
    } else {
        match (y < 0, x < 0) {
            (true, false) => "north-east",
            (true, true) => "north-west",
            (false, false) => "south-east",
            (false, true) => "south-west",
        }
    }
}

/// The cities the objective is about: cities to capture (`OwnCity`, `CityDamage`), the
/// seat's starting cities to hold, and the current cities of each rival it must contain.
/// Read from the full state, so a target in fog is still marked (the scenario reveals it).
pub fn objective_targets(
    objective: &Objective,
    seat: PlayerId,
    start: &GameState,
    now: &GameState,
) -> Vec<Target> {
    let mut take = BTreeSet::new();
    let mut hold = false;
    let mut rivals = BTreeSet::new();
    for c in objective.win.iter().chain(&objective.keep) {
        match c {
            Condition::OwnCity(id) => {
                take.insert(*id);
            }
            Condition::MinUnitsNearCity { city, .. } => {
                take.insert(*city);
            }
            Condition::HoldStartingCities | Condition::MinStartingCities(_) => hold = true,
            Condition::RivalMaxCities { rival, .. }
            | Condition::CapturedFrom { rival, .. }
            | Condition::StopWin { rival } => {
                rivals.insert(*rival);
            }
            _ => {}
        }
    }
    for t in &objective.score {
        match t.metric {
            Metric::OwnCity(id) | Metric::CityDamage(id) if t.weight > 0 => {
                take.insert(id);
            }
            Metric::StartingCitiesHeld if t.weight > 0 => hold = true,
            Metric::RivalCities(r) | Metric::RivalCitiesGained(r) | Metric::RivalCitiesLost(r) => {
                rivals.insert(r);
            }
            _ => {}
        }
    }
    let city_of = |id: &CityId| now.cities.get(id).or_else(|| start.cities.get(id));
    let mut out: Vec<Target> = Vec::new();
    let mut push = |kind, c: &City| {
        if out.iter().any(|t| t.city == c.id) {
            return;
        }
        out.push(Target {
            kind,
            city: c.id,
            name: c.name.clone(),
            owner: c.owner,
            pos: c.pos,
            bearing: String::new(),
            distance: 0,
        });
    };
    for id in &take {
        if let Some(c) = city_of(id) {
            push(TargetKind::Take, c);
        }
    }
    if hold {
        for id in starting_cities(start, seat) {
            if let Some(c) = city_of(&id) {
                push(TargetKind::Hold, c);
            }
        }
    }
    for r in &rivals {
        for c in now.cities_of(*r) {
            push(TargetKind::Contain, c);
        }
    }
    let homes: Vec<&City> = now.cities_of(seat).collect();
    for t in &mut out {
        if t.owner == seat {
            continue;
        }
        if let Some(home) = homes.iter().min_by_key(|h| (h.pos.distance(t.pos), h.id)) {
            let d = home.pos.distance(t.pos);
            t.distance = d.max(0) as u32;
            t.bearing = format!(
                "{d} tile{} {} of {}",
                if d == 1 { "" } else { "s" },
                compass(home.pos, t.pos),
                home.name
            );
        }
    }
    out
}

/// How a rival is named to the player: by faction, with the in-game name when two share one.
pub fn empire_name(state: &GameState, p: PlayerId) -> String {
    let player = state.player(p);
    let faction = crate::data::faction_name(player.faction);
    if state
        .players
        .iter()
        .filter(|q| q.faction == player.faction)
        .count()
        > 1
    {
        format!("{faction} ({})", player.name)
    } else {
        faction.to_string()
    }
}

fn city_name(start: &GameState, now: &GameState, id: CityId) -> String {
    now.cities
        .get(&id)
        .or_else(|| start.cities.get(&id))
        .map_or_else(|| format!("city {}", id.0), |c| c.name.clone())
}

fn combat_units(state: &GameState, p: PlayerId) -> i32 {
    state
        .units_of(p)
        .filter(|u| is_military(u.kind) && u.kind != UnitKind::Scout)
        .count() as i32
}

fn cities(state: &GameState, p: PlayerId) -> i32 {
    state.cities_of(p).count() as i32
}

fn starting_cities(start: &GameState, seat: PlayerId) -> BTreeSet<CityId> {
    start.cities_of(seat).map(|c| c.id).collect()
}

fn starting_held(start: &GameState, now: &GameState, seat: PlayerId) -> i32 {
    starting_cities(start, seat)
        .iter()
        .filter(|id| now.cities.get(id).is_some_and(|c| c.owner == seat))
        .count() as i32
}

fn vp(state: &GameState, p: PlayerId) -> i32 {
    crate::scoring::vp_total(state, p) as i32
}

/// The current value of a metric. `won` and `turns_left` come from the evaluation.
pub fn metric_value(
    metric: &Metric,
    seat: PlayerId,
    start: &GameState,
    now: &GameState,
    won: bool,
    turns_left: u32,
) -> i32 {
    match metric {
        Metric::Cities => cities(now, seat),
        Metric::StartingCitiesHeld => starting_held(start, now, seat),
        Metric::CitiesFounded => now
            .cities_of(seat)
            .filter(|c| c.founder == seat && !start.cities.contains_key(&c.id))
            .count() as i32,
        Metric::CitiesCaptured => now
            .cities_of(seat)
            .filter(|c| {
                start
                    .cities
                    .get(&c.id)
                    .map_or(c.founder != seat, |s| s.owner != seat)
            })
            .count() as i32,
        Metric::Population => now.cities_of(seat).map(|c| c.population).sum(),
        Metric::CombatUnits => combat_units(now, seat),
        Metric::Vp => vp(now, seat),
        Metric::VpGained => vp(now, seat) - vp(start, seat),
        Metric::OwnCity(id) => i32::from(now.cities.get(id).is_some_and(|c| c.owner == seat)),
        Metric::CityDamage(id) => match (start.cities.get(id), now.cities.get(id)) {
            (Some(_), Some(c)) if c.owner == seat => 0,
            (Some(s), Some(c)) => (s.hp - c.hp).max(0),
            // Razed (Celts) or never existed: nothing left to measure.
            _ => 0,
        },
        Metric::TurnsLeft => {
            if won {
                turns_left as i32
            } else {
                0
            }
        }
        Metric::ObjectiveMet => i32::from(won),
        Metric::RivalCities(r) => cities(now, *r),
        Metric::RivalCitiesGained(r) => (cities(now, *r) - cities(start, *r)).max(0),
        Metric::RivalCitiesLost(r) => (cities(start, *r) - cities(now, *r)).max(0),
        Metric::RivalCombatUnits(r) => combat_units(now, *r),
        Metric::RivalVpGained(r) => vp(now, *r) - vp(start, *r),
    }
}

pub fn metric_label(metric: &Metric, start: &GameState, now: &GameState) -> String {
    match metric {
        Metric::Cities => "Cities you own".into(),
        Metric::StartingCitiesHeld => "Starting cities held".into(),
        Metric::CitiesFounded => "Cities founded".into(),
        Metric::CitiesCaptured => "Cities captured".into(),
        Metric::Population => "Population".into(),
        Metric::CombatUnits => "Military units".into(),
        Metric::Vp => "Victory points".into(),
        Metric::VpGained => "Victory points gained".into(),
        Metric::OwnCity(id) => format!("{} is yours", city_name(start, now, *id)),
        Metric::CityDamage(id) => format!("Damage to {}", city_name(start, now, *id)),
        Metric::TurnsLeft => "Turns to spare".into(),
        Metric::ObjectiveMet => "Objective met".into(),
        Metric::RivalCities(r) => format!("{} cities", empire_name(now, *r)),
        Metric::RivalCitiesGained(r) => format!("Cities {} gained", empire_name(now, *r)),
        Metric::RivalCitiesLost(r) => format!("Cities {} lost", empire_name(now, *r)),
        Metric::RivalCombatUnits(r) => format!("{} military units", empire_name(now, *r)),
        Metric::RivalVpGained(r) => format!("VP {} gained", empire_name(now, *r)),
    }
}

fn condition_status(
    c: &Condition,
    seat: PlayerId,
    start: &GameState,
    now: &GameState,
    keep: bool,
) -> ConditionStatus {
    let (label, progress, met) = match c {
        Condition::HoldStartingCities => {
            let total = starting_cities(start, seat).len() as i32;
            let held = starting_held(start, now, seat);
            (
                format!(
                    "Hold your {total} {}",
                    if total == 1 { "city" } else { "cities" }
                ),
                format!("{held} of {total}"),
                held == total,
            )
        }
        Condition::MinCities(n) => {
            let have = cities(now, seat);
            (
                format!("Own at least {n} cities"),
                format!("{have} of {n}"),
                have >= *n as i32,
            )
        }
        Condition::MinStartingCities(n) => {
            let total = starting_cities(start, seat).len();
            let held = starting_held(start, now, seat);
            (
                format!("Keep at least {n} of your {total} starting cities"),
                format!("{held} of {n}"),
                held >= *n as i32,
            )
        }
        Condition::MinCombatUnits(n) => {
            let have = combat_units(now, seat);
            (
                format!("Keep at least {n} military units (Scouts don't count)"),
                format!("{have} of {n}"),
                have >= *n as i32,
            )
        }
        Condition::MinUnitsNearCity {
            kind,
            city,
            radius,
            count,
        } => {
            let center = now.cities.get(city).or_else(|| start.cities.get(city));
            let have = center.map_or(0, |center| {
                now.units_of(seat)
                    .filter(|unit| unit.kind == *kind && unit.pos.distance(center.pos) <= *radius)
                    .count()
            });
            (
                format!(
                    "Have {count} {} within {radius} tiles of {}",
                    data::unit_name(*kind),
                    city_name(start, now, *city)
                ),
                format!("{have} of {count}"),
                have >= *count as usize,
            )
        }
        Condition::ExactUnitProduction { kind, cities } => {
            let building: BTreeSet<CityId> = now
                .cities_of(seat)
                .filter(|city| city.producing == Some(ProductionItem::Unit { unit: *kind }))
                .map(|city| city.id)
                .collect();
            let expected: BTreeSet<CityId> = cities.iter().copied().collect();
            (
                format!(
                    "Produce {} in the named cities only",
                    data::unit_name(*kind)
                ),
                format!(
                    "{} of {} cities",
                    building.intersection(&expected).count(),
                    cities.len()
                ),
                building == expected,
            )
        }
        Condition::OwnCity(id) => {
            let owner = now.cities.get(id).map(|c| c.owner);
            (
                format!("Take {}", city_name(start, now, *id)),
                match owner {
                    Some(o) if o == seat => "Yours".into(),
                    Some(o) => format!("Held by {}", empire_name(now, o)),
                    None => "Razed".into(),
                },
                owner == Some(seat),
            )
        }
        Condition::RivalMaxCities { rival, max } => {
            let have = cities(now, *rival);
            (
                format!("{} holds at most {max} cities", empire_name(now, *rival)),
                format!("{have} now"),
                have <= *max as i32,
            )
        }
        Condition::CapturedFrom { rival, min } => {
            let have = now.cities_of(seat).filter(|c| c.founder == *rival).count() as i32;
            (
                format!("Capture {min} of {}'s cities", empire_name(now, *rival)),
                format!("{have} of {min}"),
                have >= *min as i32,
            )
        }
        Condition::UnitsGone(ids) => {
            let left = ids.iter().filter(|id| now.units.contains_key(id)).count();
            (
                format!("Destroy {} of their units", ids.len()),
                format!("{} of {}", ids.len() - left, ids.len()),
                left == 0,
            )
        }
        Condition::StopWin { rival } => {
            let team = now.team_of(*rival);
            let won = now.finished && now.winner == Some(team);
            let progress = match now.vp_holds.get(&team) {
                _ if won => "Won".to_string(),
                Some(held) => format!(
                    "Holding its target, {held} of {} turns",
                    crate::data::VP_HOLD_TURNS
                ),
                None => "Not holding its target".to_string(),
            };
            (
                format!("Stop {} winning", empire_name(now, *rival)),
                progress,
                !won,
            )
        }
    };
    ConditionStatus {
        label,
        progress,
        met,
        keep,
    }
}

/// Judge the seat's position. `start` is the scenario's start state; `now` the current one.
pub fn evaluate(
    objective: &Objective,
    seat: PlayerId,
    start: &GameState,
    now: &GameState,
) -> Status {
    let end_turn = objective.end_turn(start);
    let turns_left = end_turn.saturating_sub(now.turn);
    let mut conditions: Vec<ConditionStatus> = objective
        .keep
        .iter()
        .map(|c| condition_status(c, seat, start, now, true))
        .collect();
    let wins: Vec<ConditionStatus> = objective
        .win
        .iter()
        .map(|c| condition_status(c, seat, start, now, false))
        .collect();
    let all_won = !wins.is_empty() && wins.iter().all(|c| c.met);
    let broken = conditions.iter().find(|c| !c.met).map(|c| c.label.clone());
    let unmet: Vec<String> = wins
        .iter()
        .filter(|c| !c.met)
        .map(|c| c.label.clone())
        .collect();
    conditions.extend(wins);
    let over = now.turn >= end_turn || now.finished;
    let (outcome, reason) = if !now.player(seat).alive {
        (Outcome::Lost, "Your empire was eliminated.".to_string())
    } else if let Some(label) = broken {
        (Outcome::Lost, format!("Failed: {label}."))
    } else if all_won && (objective.end_on_win || over) {
        (
            Outcome::Won,
            if over {
                "Objective met at the deadline.".to_string()
            } else {
                format!(
                    "Objective met with {turns_left} turn{} to spare.",
                    if turns_left == 1 { "" } else { "s" }
                )
            },
        )
    } else if over && objective.win.is_empty() {
        (Outcome::Complete, "The window is over.".to_string())
    } else if over {
        (
            Outcome::Lost,
            format!("Time ran out: {}.", unmet.join("; ")),
        )
    } else if objective.win.is_empty() {
        (Outcome::Playing, String::new())
    } else {
        (
            Outcome::Playing,
            format!("Still needed: {}.", unmet.join("; ")),
        )
    };
    let won = outcome == Outcome::Won;
    let score_lines: Vec<ScoreLine> = objective
        .score
        .iter()
        .map(|t| {
            let value = metric_value(&t.metric, seat, start, now, won, turns_left);
            ScoreLine {
                label: metric_label(&t.metric, start, now),
                value,
                weight: t.weight,
                points: value.saturating_mul(t.weight),
            }
        })
        .collect();
    Status {
        turn: now.turn,
        start_turn: start.turn,
        end_turn,
        turns_left,
        outcome,
        reason,
        score: score_lines.iter().map(|l| l.points).sum(),
        conditions,
        score_lines,
        targets: objective_targets(objective, seat, start, now),
    }
}

/// Problems with an objective for this start state (unknown cities or rivals, empty window).
pub fn check_objective(objective: &Objective, seat: PlayerId, start: &GameState) -> Vec<String> {
    let mut errors = Vec::new();
    let player_ok = |p: PlayerId| (p.0 as usize) < start.players.len();
    if !player_ok(seat) {
        errors.push(format!("seat {} is not a player", seat.0));
        return errors;
    }
    if !start.player(seat).alive {
        errors.push(format!("seat {} is eliminated", seat.0));
    }
    if objective.turns == 0 {
        errors.push("the objective window has no turns".into());
    }
    if objective.score.is_empty() {
        errors.push("the objective has no score terms".into());
    }
    let city = |id: &CityId, errors: &mut Vec<String>| {
        if !start.cities.contains_key(id) {
            errors.push(format!("city {} is not in the start state", id.0));
        }
    };
    let rival = |r: &PlayerId, errors: &mut Vec<String>| {
        if !player_ok(*r) || *r == seat {
            errors.push(format!("rival {} is not another player", r.0));
        } else if !start.player(*r).alive {
            errors.push(format!("rival {} is eliminated", r.0));
        }
    };
    for c in objective.win.iter().chain(&objective.keep) {
        match c {
            Condition::OwnCity(id) | Condition::MinUnitsNearCity { city: id, .. } => {
                city(id, &mut errors)
            }
            Condition::ExactUnitProduction { cities, .. } => {
                for id in cities {
                    city(id, &mut errors);
                    if start.cities.get(id).is_some_and(|c| c.owner != seat) {
                        errors.push(format!("city {} is not owned by the seat", id.0));
                    }
                }
            }
            Condition::RivalMaxCities { rival: r, .. }
            | Condition::CapturedFrom { rival: r, .. }
            | Condition::StopWin { rival: r } => rival(r, &mut errors),
            _ => {}
        }
    }
    for condition in objective.win.iter().chain(&objective.keep) {
        if let Condition::MinUnitsNearCity { radius, .. } = condition
            && *radius < 0
        {
            errors.push("MinUnitsNearCity radius must be nonnegative".into());
        }
    }
    for t in &objective.score {
        match &t.metric {
            Metric::OwnCity(id) | Metric::CityDamage(id) => city(id, &mut errors),
            Metric::RivalCities(r)
            | Metric::RivalCitiesGained(r)
            | Metric::RivalCitiesLost(r)
            | Metric::RivalCombatUnits(r)
            | Metric::RivalVpGained(r) => rival(r, &mut errors),
            _ => {}
        }
    }
    if objective.keep.contains(&Condition::HoldStartingCities)
        && start.cities_of(seat).next().is_none()
    {
        errors.push("HoldStartingCities needs the seat to own a city".into());
    }
    errors
}

/// Rule violations in a (possibly hand-edited) state. Empty means the position is legal: the
/// structure is consistent, units stand where the rules allow and cities keep their spacing.
pub fn validate_state(state: &GameState) -> Vec<String> {
    let mut errors = Vec::new();
    let s = &state.setup;
    if state.turn == 0 {
        errors.push("turn must be at least 1".into());
    }
    if state.map.width != s.width
        || state.map.height != s.height
        || state.map.tiles.len() != (s.width * s.height) as usize
    {
        errors.push("map size does not match the setup".into());
        return errors;
    }
    if state.players.len() != s.players.len() {
        errors.push("player count does not match the setup".into());
        return errors;
    }
    for (i, p) in state.players.iter().enumerate() {
        let ps = &s.players[i];
        if p.id != PlayerId(i as u8) {
            errors.push(format!("player {i} has id {}", p.id.0));
        }
        if p.is_human != ps.is_human || p.team != ps.team || p.faction != ps.faction {
            errors.push(format!("player {i} does not match its setup"));
        }
        if p.explored.len() != state.map.len() || !p.explored.has_valid_storage() {
            errors.push(format!("player {i} has a malformed explored set"));
        }
        if p.alive
            && state.units_of(p.id).next().is_none()
            && state.cities_of(p.id).next().is_none()
        {
            errors.push(format!("player {i} is alive with no units or cities"));
        }
        if state.cities_of(p.id).filter(|c| c.is_capital).count() > 1 {
            errors.push(format!("player {i} has more than one capital"));
        }
    }
    let valid_owner = |p: PlayerId| p.is_barbarian() || (p.0 as usize) < state.players.len();
    // Each tile holds at most one military unit and one civilian, and both only within a team.
    let mut on_tile: BTreeMap<Hex, Vec<&Unit>> = BTreeMap::new();
    for (id, u) in &state.units {
        let name = format!("unit {} ({:?})", id.0, u.kind);
        if *id != u.id || id.0 > state.next_unit_id {
            errors.push(format!("{name} has an inconsistent id"));
        }
        if !valid_owner(u.owner) {
            errors.push(format!("{name} has no valid owner"));
            continue;
        }
        if !u.owner.is_barbarian() && !state.player(u.owner).alive {
            errors.push(format!("{name} belongs to an eliminated empire"));
        }
        if !(1..=UNIT_HP).contains(&u.hp) {
            errors.push(format!("{name} has {} HP", u.hp));
        }
        match state.map.get(u.pos) {
            None => errors.push(format!("{name} is off the map")),
            Some(t) if t.terrain.move_cost().is_none() => {
                errors.push(format!("{name} stands on impassable {:?}", t.terrain))
            }
            Some(t) => {
                if !state.passage_allowed(u.owner, u.kind, t.owner) {
                    errors.push(format!(
                        "{name} stands in {} territory without being at war",
                        empire_name(state, t.owner.expect("owned"))
                    ));
                }
            }
        }
        if let Some(c) = state.city_at(u.pos)
            && !state.same_team(c.owner, u.owner)
            && !state.allied(c.owner, u.owner)
        {
            errors.push(format!("{name} stands in the foreign city {}", c.name));
        }
        on_tile.entry(u.pos).or_default().push(u);
    }
    for (pos, units) in &on_tile {
        let military = units.iter().filter(|u| is_military(u.kind)).count();
        let civilians = units.len() - military;
        if military > 1 || civilians > 1 {
            errors.push(format!(
                "{} units stack on ({}, {})",
                units.len(),
                pos.q,
                pos.r
            ));
        } else if units.len() == 2 && !state.same_team(units[0].owner, units[1].owner) {
            errors.push(format!(
                "units of different teams share ({}, {})",
                pos.q, pos.r
            ));
        }
    }
    let cities: Vec<&City> = state.cities.values().collect();
    for (i, c) in cities.iter().enumerate() {
        let name = format!("city {} ({})", c.id.0, c.name);
        if state.cities.get(&c.id).map(|x| x.id) != Some(c.id) || c.id.0 > state.next_city_id {
            errors.push(format!("{name} has an inconsistent id"));
        }
        if (c.owner.0 as usize) >= state.players.len() || !state.player(c.owner).alive {
            errors.push(format!("{name} has no living owner"));
        }
        match state.map.get(c.pos) {
            Some(t) if t.terrain.is_land() => {
                if t.owner != Some(c.owner) || t.city != Some(c.id) {
                    errors.push(format!("{name} does not own its own tile"));
                }
            }
            _ => errors.push(format!("{name} is not on land")),
        }
        if !(0..=CITY_HP).contains(&c.hp) || c.population < 1 {
            errors.push(format!(
                "{name} has {} HP and {} population",
                c.hp, c.population
            ));
        }
        for other in &cities[i + 1..] {
            let rules = state.setup.rule_experiments;
            let gap = rules
                .min_city_gap(state.player(c.founder).faction)
                .min(rules.min_city_gap(state.player(other.founder).faction));
            if c.pos.distance(other.pos) <= gap {
                errors.push(format!("{name} is within {gap} hexes of {}", other.name));
            }
        }
        if data::has_building(&c.buildings, BuildingKind::Walls)
            && !state.player(c.owner).techs.contains(&TechId::Masonry)
            && c.founder == c.owner
        {
            // Walls need Masonry; a captured walled city keeps its walls.
            errors.push(format!("{name} has Walls its owner cannot build"));
        }
    }
    errors
}

#[cfg(test)]
mod tests;
