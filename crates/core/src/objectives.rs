//! World Ages: one public card per Age (owner, 2026-11-22: "world ages and then we'll iterate on
//! world ages from there"; the three-objectives-per-era system is gone).
//!
//! When the first empire reaches a new era, a new Age begins for the whole world and one card,
//! drawn at random from the Age's pool (`age_pool`), is revealed to everyone. Any empire may
//! complete it for 1 permanent VP; the card never expires. Only what an empire does after the
//! reveal counts: each open Age keeps a snapshot of the empire's fight counters and holdings
//! (`EraRecord::base`, `EraRecord::levels`), and every Age score takes a fresh snapshot for the
//! Ages still open, so what scored one Age never scores another (R1). At most one Age scores per
//! empire per turn, the oldest first (R2). The cards are a pure function of the game seed and the
//! era index, so they need no stored state.

use crate::data;
use crate::events::Event;
use crate::rng::{Rng, purpose};
use crate::scoring;
use crate::state::GameState;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Fights an empire has won so far. Fight cards count from when counting began for their Age
/// (`EraRecord::base`), so a capture made long ago banks nothing for a later Age.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FightStats {
    /// Cities taken from an empire ahead on VP, not counting capitals and wonder cities.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub humbled: u16,
    /// Captured walled cities that were not capitals or wonder cities.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub walled: u16,
    /// Promotion levels gained through XP by this empire's units (levels a unit starts with never
    /// count).
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub promotions: u16,
}

impl FightStats {
    pub const ZERO: FightStats = FightStats {
        humbled: 0,
        walled: 0,
        promotions: 0,
    };

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    fn since(self, base: FightStats) -> FightStats {
        FightStats {
            humbled: self.humbled.saturating_sub(base.humbled),
            walled: self.walled.saturating_sub(base.walled),
            promotions: self.promotions.saturating_sub(base.promotions),
        }
    }
}

fn is_zero_u16(n: &u16) -> bool {
    *n == 0
}

/// What an empire holds at one moment: the counters the Build and Expand objectives count gains
/// of. An era's record keeps the holdings from when counting began (`EraRecord::levels`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Levels {
    pub cities: u16,
    pub population: u32,
    pub tiles: u32,
    /// Cities holding each building (or a later step of its family), by kind; zero counts omitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buildings: Vec<(BuildingKind, u16)>,
}

impl Levels {
    pub fn is_zero(&self) -> bool {
        *self == Self::default()
    }

    /// What empire `p` holds now.
    pub fn of(state: &GameState, p: PlayerId) -> Levels {
        Levels {
            cities: Self::cities_now(state, p),
            population: Self::population_now(state, p),
            tiles: Self::tiles_now(state, p),
            buildings: BuildingKind::ALL
                .iter()
                .filter_map(|b| {
                    let n = Self::building_now(state, p, *b);
                    (n > 0).then_some((*b, n))
                })
                .collect(),
        }
    }

    fn cities_now(state: &GameState, p: PlayerId) -> u16 {
        state.cities_of(p).count() as u16
    }

    fn population_now(state: &GameState, p: PlayerId) -> u32 {
        state.cities_of(p).map(|c| c.population.max(0) as u32).sum()
    }

    fn tiles_now(state: &GameState, p: PlayerId) -> u32 {
        state
            .map
            .tiles
            .iter()
            .filter(|t| t.owner == Some(p))
            .count() as u32
    }

    fn building_now(state: &GameState, p: PlayerId, building: BuildingKind) -> u16 {
        // An idle Forge or Factory (no supply) doesn't count: it produces nothing.
        state
            .cities_of(p)
            .filter(|c| {
                data::has_building(&c.buildings, building)
                    && crate::resources::idle_tier(state.player(p), c.id, building).is_none()
            })
            .count() as u16
    }

    fn building_at(&self, building: BuildingKind) -> u16 {
        self.buildings
            .iter()
            .find(|(b, _)| *b == building)
            .map_or(0, |(_, n)| *n)
    }
}

/// An era an empire has entered, and whether it has scored it. `base` and `levels` are what the
/// empire had when counting began for this era: when it entered, or at its last era score if
/// later. The era's objectives count what it gained since.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraRecord {
    pub entered: u32,
    /// The turn of the latest snapshot, initially the era-entry turn.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub counting_since: u32,
    #[serde(default, skip_serializing_if = "FightStats::is_zero")]
    pub base: FightStats,
    #[serde(default, skip_serializing_if = "Levels::is_zero")]
    pub levels: Levels,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scored: Option<EraScore>,
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

/// How and when an era was scored: the objective's slot (its family's index) and the turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraScore {
    pub turn: u32,
    pub slot: u8,
}

/// What kind of play a card asks for: building, growing or fighting. Shown on the card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Family {
    Build,
    Expand,
    Fight,
}

impl Family {
    pub fn name(self) -> &'static str {
        match self {
            Family::Build => "Build",
            Family::Expand => "Expand",
            Family::Fight => "Fight",
        }
    }
}

/// One World Age card: one thing an empire can do to score an Age.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Objective {
    /// Veterans: your units earn `n` promotions (levels gained through XP) after the reveal.
    Veterans { n: u16 },
    /// Humble the mighty: capture a city from an empire ahead of you on VP. A capital or wonder
    /// city never counts.
    HumbleTheMighty,
    /// Breach: capture a walled city. A capital or wonder city never counts.
    Breach,
    /// Gain `n` cities (founded or captured) after the reveal.
    Cities { n: u16 },
    /// Prosperity: grow total population by `n`.
    Population { n: u16 },
    /// Frontier: gain `n` tiles of territory.
    Territory { n: u16 },
    /// Patrons: `cities` more cities hold the building (or a later step of its family).
    Buildings { building: BuildingKind, cities: u16 },
}

impl Objective {
    /// Baseline and current total behind a gain-count objective, in the same units as its text.
    pub fn count_accounting(self, state: &GameState, p: PlayerId, era: u8) -> Option<(u32, u32)> {
        let r = state.player(p).eras.get(&era)?;
        let f = state.player(p).fight;
        let pair = match self {
            Objective::Cities { .. } => (
                u32::from(r.levels.cities),
                u32::from(Levels::cities_now(state, p)),
            ),
            Objective::Population { .. } => (r.levels.population, Levels::population_now(state, p)),
            Objective::Territory { .. } => (r.levels.tiles, Levels::tiles_now(state, p)),
            Objective::Buildings { building, .. } => (
                u32::from(r.levels.building_at(building)),
                u32::from(Levels::building_now(state, p, building)),
            ),
            Objective::HumbleTheMighty => (u32::from(r.base.humbled), u32::from(f.humbled)),
            Objective::Breach => (u32::from(r.base.walled), u32::from(f.walled)),
            Objective::Veterans { .. } => (u32::from(r.base.promotions), u32::from(f.promotions)),
        };
        Some(pair)
    }

    pub fn family(self) -> Family {
        match self {
            Objective::Cities { .. }
            | Objective::Population { .. }
            | Objective::Territory { .. } => Family::Expand,
            Objective::Buildings { .. } => Family::Build,
            Objective::Veterans { .. } | Objective::HumbleTheMighty | Objective::Breach => {
                Family::Fight
            }
        }
    }

    pub fn target(self) -> u32 {
        match self {
            Objective::HumbleTheMighty | Objective::Breach => 1,
            Objective::Veterans { n }
            | Objective::Cities { n }
            | Objective::Population { n }
            | Objective::Territory { n } => u32::from(n),
            Objective::Buildings { cities, .. } => u32::from(cities),
        }
    }

    /// Plain-language rule, e.g. "Frontier: gain 80 tiles of territory".
    pub fn text(self) -> String {
        let n = self.target();
        let cities = if n == 1 { "city" } else { "cities" };
        match self {
            Objective::Veterans { n } => format!(
                "Veterans: earn {n} promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)"
            ),
            Objective::HumbleTheMighty => {
                "Humble the mighty: capture a city from an empire ahead of you on VP".into()
            }
            Objective::Breach => "Breach: capture a walled city".into(),
            Objective::Cities { .. } => format!("Gain {n} {cities}"),
            Objective::Population { .. } => format!("Prosperity: grow your population by {n}"),
            Objective::Territory { .. } => format!("Frontier: gain {n} tiles of territory"),
            Objective::Buildings { building, .. } => format!(
                "Patrons: add a {} to {n} more {cities}",
                data::building_name(building)
            ),
        }
    }

    /// How far empire `p` is towards it (not capped at the target), counting only what it gained
    /// since counting began for the Age: its reveal, or its last Age score if later. An Age that
    /// has not begun counts nothing.
    pub fn progress(self, state: &GameState, p: PlayerId, era: u8) -> u32 {
        let player = state.player(p);
        let record = player.eras.get(&era);
        let fights = || {
            let base = record.map_or(player.fight, |r| r.base);
            player.fight.since(base)
        };
        let gain = |now: u32, at_base: &dyn Fn(&Levels) -> u32| {
            record.map_or(0, |r| now.saturating_sub(at_base(&r.levels)))
        };
        match self {
            Objective::Veterans { .. } => u32::from(fights().promotions),
            Objective::HumbleTheMighty => u32::from(fights().humbled),
            Objective::Breach => u32::from(fights().walled),
            Objective::Cities { .. } => gain(u32::from(Levels::cities_now(state, p)), &|l| {
                u32::from(l.cities)
            }),
            Objective::Population { .. } => {
                gain(Levels::population_now(state, p), &|l| l.population)
            }
            Objective::Territory { .. } => gain(Levels::tiles_now(state, p), &|l| l.tiles),
            Objective::Buildings { building, .. } => {
                gain(u32::from(Levels::building_now(state, p, building)), &|l| {
                    u32::from(l.building_at(building))
                })
            }
        }
    }

    /// Whether the turn-end check scores it.
    pub fn met(self, state: &GameState, p: PlayerId, era: u8) -> bool {
        self.progress(state, p, era) >= self.target()
    }
}

use Objective as O;

/// Promotions a Veterans card asks for in every Age.
const VETERANS_PROMOTIONS: u16 = 4;
/// Cities gained by a Gain N cities card. The card is only in the pool from the Renaissance on,
/// when free land is gone.
const CITIES_GAINED: u16 = 3;
/// The first era whose pool has a Gain N cities card (Renaissance).
const CITIES_FROM_ERA: u8 = 4;

/// The peaceful cards of an Age (era 2..9): Patrons (a building that every faction can raise:
/// the Library line, or the Bank in the Machine Age and World War), Prosperity (population) and
/// Frontier (territory), so an empire at peace always has a card it can work towards. Counts are
/// gains after the reveal for an 8-empire lobby (`scaled_for` raises them for fewer), carried
/// over from the old era objectives, whose counts sat near the 70th percentile of the best gain
/// entrants made (`docs/sim-results/era-objectives/step1.md`).
fn peaceful_cards(era: u8) -> [Objective; 3] {
    let (building, cities, population, tiles) = match era {
        2 => (BuildingKind::Library, 4, 50, 80),
        3 => (BuildingKind::Library, 4, 42, 62),
        4 => (BuildingKind::University, 4, 34, 49),
        5 => (BuildingKind::University, 3, 25, 35),
        6 => (BuildingKind::Bank, 3, 23, 31),
        7 => (BuildingKind::Bank, 3, 23, 31),
        _ => (BuildingKind::ResearchLab, 3, 25, 33),
    };
    [
        O::Buildings { building, cities },
        O::Population { n: population },
        O::Territory { n: tiles },
    ]
}

/// The cards a World Age (era 2..9) can draw from, as written (`scaled_pool` applies the lobby
/// size). None is a stock the empire already holds, so each counts only what is done after the
/// reveal (cards never expire): Veterans counts promotions earned in battle; Humble the mighty
/// and Breach are harder captures than a normal war; Gain N cities counts from the Renaissance
/// on; Patrons, Prosperity and Frontier are the peaceful cards.
pub fn age_pool(era: u8) -> Vec<Objective> {
    if !(2..=data::era_count()).contains(&era) {
        return Vec::new();
    }
    let mut pool = vec![
        O::Veterans {
            n: VETERANS_PROMOTIONS,
        },
        O::HumbleTheMighty,
        O::Breach,
    ];
    if era >= CITIES_FROM_ERA {
        pool.push(O::Cities { n: CITIES_GAINED });
    }
    pool.extend(peaceful_cards(era));
    pool
}

impl Objective {
    /// The peaceful cards' counts are written for an 8-empire lobby; fewer empires share the land
    /// and each grows bigger, so they rise 12% per missing empire (a 4-empire lobby asks 48% more).
    /// The fight cards and Gain N cities are not scaled.
    fn scaled_for(self, empires: usize, objective_scale: u16) -> Objective {
        let lobby = 100 + 12 * 8usize.saturating_sub(empires.max(2)) as u32;
        let percent = (lobby * u32::from(objective_scale) + 50) / 100;
        let scale = |n: u16| ((u32::from(n) * percent + 50) / 100) as u16;
        match self {
            Objective::Population { n } => Objective::Population { n: scale(n) },
            Objective::Territory { n } => Objective::Territory { n: scale(n) },
            Objective::Buildings { building, cities } => Objective::Buildings {
                building,
                cities: scale(cities),
            },
            other => other,
        }
    }
}

/// The Age's whole pool at the game's counts (`scaled_for`), in `age_pool` order.
pub fn scaled_pool(setup: &crate::state::GameSetup, era: u8) -> Vec<Objective> {
    let empires = setup.players.len();
    let objective_scale = setup.rule_experiments.objective_scale_percent();
    age_pool(era)
        .into_iter()
        .map(|o| o.scaled_for(empires, objective_scale))
        .collect()
}

/// The single card drawn for an Age, identical for every empire in a game, with its index in
/// `age_pool`. Empty for era 1 and for eras beyond the tree.
pub fn drawn_from_pool(setup: &crate::state::GameSetup, era: u8) -> Vec<(usize, Objective)> {
    if era < 2 || era > data::era_count() {
        return Vec::new();
    }
    let pool = scaled_pool(setup, era);
    // One card per Age, drawn at random (user, 2026-11-11: "only one, randomly decided option per age").
    let mut rng = Rng::derive(setup.seed, &[u64::from(era), purpose::OBJECTIVES, 97]);
    let i = rng.range(0, pool.len() as i64 - 1) as usize;
    vec![(i, pool[i])]
}

/// The card of an Age (one per Age), identical for every empire in a game. Empty for era 1 and
/// for eras beyond the tree.
pub fn era_objectives(setup: &crate::state::GameSetup, era: u8) -> Vec<Objective> {
    drawn_from_pool(setup, era)
        .into_iter()
        .map(|(_, o)| o)
        .collect()
}

/// The slot a card takes in its Age's list: always 0 (one card per Age).
pub fn slot_in(setup: &crate::state::GameSetup, era: u8, objective: Objective) -> u8 {
    era_objectives(setup, era)
        .iter()
        .position(|o| *o == objective)
        .unwrap_or(0) as u8
}

/// The era an empire is in, 1-based (1 Ancient .. 9 Future): `eras::current_era` + 1. It rises
/// the turn the previous era's last technology lands (`Event::EraEntered`). Only ever rises.
pub fn era_of(state: &GameState, p: PlayerId) -> u8 {
    crate::eras::current_era(&state.player(p).techs) + 1
}

/// Each living team's VP at the start of the turn, so every capture this turn is judged against
/// the same standings.
#[derive(Clone, Debug)]
pub struct CaptureVpSnapshot {
    vp: BTreeMap<TeamId, u32>,
    /// Who held each scoring rival original capital at the start of the turn
    /// (`scoring::capital_holders`), so end-of-turn scoring reports its change of hands.
    pub(crate) capitals: BTreeMap<CityId, (PlayerId, String)>,
}

impl CaptureVpSnapshot {
    pub fn new(state: &GameState) -> Self {
        let vp: BTreeMap<TeamId, u32> = state
            .teams()
            .into_iter()
            .filter(|t| {
                state
                    .team_members(*t)
                    .iter()
                    .any(|p| state.player(*p).alive)
            })
            .map(|t| (t, scoring::team_vp(state, t)))
            .collect();
        Self {
            vp,
            capitals: scoring::capital_holders(state),
        }
    }

    fn ahead(&self, victim: TeamId, taker: TeamId) -> bool {
        self.vp.get(&victim).copied().unwrap_or(0) > self.vp.get(&taker).copied().unwrap_or(0)
    }
}

/// A capital or a wonder city: capturing it already pays VP, so a World Age Fight card never
/// counts it.
pub fn prize_city(city: &crate::state::City) -> bool {
    city.original_capital
        || city
            .buildings
            .iter()
            .any(|b| data::is_great_work(*b) || crate::landmarks::is_landmark(*b))
}

/// Whether a city has Walls of any tier.
pub fn walled_city(city: &crate::state::City) -> bool {
    city.buildings
        .iter()
        .any(|b| data::building_family(*b) == BuildingKind::Walls)
}

/// The highest era any empire has reached: the newest World Age. Era 1 until one begins. Only
/// reads the records, so an eliminated leader does not hide an Age that has begun.
pub fn revealed_era(state: &GameState) -> u8 {
    state
        .players
        .iter()
        .flat_map(|p| p.eras.keys().copied())
        .max()
        .unwrap_or(1)
}

/// A capture this turn: who took it, from whom, and whether it was a capital or wonder city, and
/// whether it had Walls.
struct Taken {
    taker: PlayerId,
    victim: PlayerId,
    prize: bool,
    walled: bool,
}

/// Credit this turn's fights to the empires that won them.
fn record_fights(state: &mut GameState, events: &[Event], vp: &CaptureVpSnapshot) {
    let mut taken: Vec<Taken> = Vec::new();
    for e in events {
        match e {
            Event::CityCaptured { from, to, city, .. } => {
                let c = state.cities.get(city);
                taken.push(Taken {
                    taker: *to,
                    victim: *from,
                    prize: c.is_some_and(prize_city),
                    walled: c.is_some_and(walled_city),
                });
            }
            Event::CityRazed {
                from,
                by,
                prize,
                walled,
                ordered: false,
                ..
            } => taken.push(Taken {
                taker: *by,
                victim: *from,
                prize: *prize,
                walled: *walled,
            }),
            _ => {}
        }
    }
    for t in taken {
        let (taker, victim) = (t.taker, t.victim);
        if taker.is_barbarian() || victim.is_barbarian() || taker == victim {
            continue;
        }
        let ahead = vp.ahead(state.team_of(victim), state.team_of(taker));
        let f = &mut state.player_mut(taker).fight;
        if ahead && !t.prize {
            f.humbled = f.humbled.saturating_add(1);
        }
        if t.walled && !t.prize {
            f.walled = f.walled.saturating_add(1);
        }
    }
}

/// End-of-turn: credit fights, open each Age the turn the first empire enters its era (for every
/// living empire at once), and score any Age whose card is met.
pub fn update(state: &mut GameState, events: &mut Vec<Event>, vp: &CaptureVpSnapshot) {
    let this_turn = events.clone();
    record_fights(state, &this_turn, vp);
    let setup = state.setup.clone();
    let turn = state.turn;
    let ids: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let newest = ids.iter().map(|p| era_of(state, *p)).max().unwrap_or(1);
    let opened_before = revealed_era(state);
    for era in opened_before + 1..=newest {
        if let Some(first) = state.era_firsts.get(&(era - 1)) {
            events.push(Event::AgeBegan { era, first: *first });
        }
    }
    for p in ids {
        let reached = newest.max(revealed_era(state));
        for era in 2..=reached {
            if !state.player(p).eras.contains_key(&era) {
                let base = state.player(p).fight;
                let levels = Levels::of(state, p);
                state.player_mut(p).eras.insert(
                    era,
                    EraRecord {
                        entered: turn,
                        counting_since: turn,
                        base,
                        levels,
                        scored: None,
                    },
                );
            }
        }
        // R2: at most one era scores per empire per turn, including an earlier claim action.
        if events
            .iter()
            .any(|e| matches!(e, Event::EraObjectiveScored { player, .. } if *player == p))
        {
            continue;
        }
        for era in 2..=reached {
            if state.player(p).eras[&era].scored.is_some() {
                continue;
            }
            let hit = era_objectives(&setup, era)
                .into_iter()
                .find(|o| o.met(state, p, era));
            if let Some(objective) = hit {
                let slot = slot_in(&setup, era, objective);
                if let Some(r) = state.player_mut(p).eras.get_mut(&era) {
                    r.scored = Some(EraScore { turn, slot });
                }
                // R1: counting starts again for every era still open.
                restart_counting(state, p);
                events.push(Event::EraObjectiveScored {
                    player: p,
                    era,
                    slot,
                    objective: objective.text(),
                });
                break;
            }
        }
    }
}

/// Take a fresh snapshot of what `p` holds for every era it has entered and not scored.
fn restart_counting(state: &mut GameState, p: PlayerId) {
    let base = state.player(p).fight;
    let levels = Levels::of(state, p);
    let turn = state.turn;
    for r in state.player_mut(p).eras.values_mut() {
        if r.scored.is_none() {
            r.base = base;
            r.levels = levels.clone();
            r.counting_since = turn;
        }
    }
}

#[cfg(test)]
#[path = "objectives_tests.rs"]
mod tests;
