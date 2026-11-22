//! VP v2 era objectives (TI4-style public objectives).
//!
//! Every era after the first reveals three public objectives, one each from Build, Expand and
//! Fight, drawn deterministically from the era's pool at game start. Meeting any one earns that
//! era's single permanent VP, once per empire. An empire can score an era only once it has entered
//! it, and an unscored era stays open for good. The era's objectives are a pure function of the
//! game seed and the era index, so they need no stored state.
//!
//! Era objectives count only what an empire gains after it entered the era and after its last era
//! score (R1, user 2026-10-20): each open era keeps a snapshot of the empire's fight counters and
//! holdings (`EraRecord::base`, `EraRecord::levels`), and every era score takes a fresh snapshot
//! for the eras still open, so what scored one era never scores another. At most one era scores
//! per empire per turn, the oldest first (R2).
//!
//! Era-generic: the era index and count come from `data::era_index` / `data::era_count`. To add
//! eras, add a pool for each in `era_pool` (an era without one falls back to `generic_pool`).

use crate::data;
use crate::events::{Attacker, Defender, Event};
use crate::rng::{Rng, purpose};
use crate::scoring;
use crate::state::GameState;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Fights an empire has won so far. Fight objectives count from when counting began for their era
/// (`EraRecord::base`), so a capture made long ago banks nothing for a later era.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FightStats {
    /// Cities taken from another empire (captured or razed).
    pub captures: u16,
    /// ...of which from an empire ahead of the taker on VP.
    pub from_ahead: u16,
    /// ...of which from the VP leader.
    pub from_leader: u16,
    /// Enemy military units destroyed.
    pub kills: u16,
    /// A VP hold ended by taking a city from the holder the same turn.
    pub holds_broken: u16,
    /// World Ages: cities taken from an empire ahead on VP, not counting capitals and wonder cities.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub humbled: u16,
    /// World Ages: captured walled cities that were not capitals or wonder cities.
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub walled: u16,
    /// World Ages: promotion levels gained through XP by this empire's units (levels a unit starts
    /// with never count).
    #[serde(default, skip_serializing_if = "is_zero_u16")]
    pub promotions: u16,
}

impl FightStats {
    pub const ZERO: FightStats = FightStats {
        captures: 0,
        from_ahead: 0,
        from_leader: 0,
        kills: 0,
        holds_broken: 0,
        humbled: 0,
        walled: 0,
        promotions: 0,
    };

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    fn since(self, base: FightStats) -> FightStats {
        FightStats {
            captures: self.captures.saturating_sub(base.captures),
            from_ahead: self.from_ahead.saturating_sub(base.from_ahead),
            from_leader: self.from_leader.saturating_sub(base.from_leader),
            kills: self.kills.saturating_sub(base.kills),
            holds_broken: self.holds_broken.saturating_sub(base.holds_broken),
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
    pub techs: u16,
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
            techs: state.player(p).techs.len() as u16,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Family {
    Build,
    Expand,
    Fight,
}

impl Family {
    /// The three families of the era objectives.
    pub const ALL: [Family; 3] = [Family::Build, Family::Expand, Family::Fight];

    pub fn name(self) -> &'static str {
        match self {
            Family::Build => "Build",
            Family::Expand => "Expand",
            Family::Fight => "Fight",
        }
    }
}

/// One thing an empire can do to score an era.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Objective {
    /// World Age Veterans: your units earn `n` promotions (levels gained through XP) after the
    /// reveal.
    Veterans { n: u16 },
    /// World Age Humble the mighty: capture a city from an empire ahead of you on VP. A capital or
    /// wonder city never counts.
    HumbleTheMighty,
    /// World Age Breach: capture a walled city. A capital or wonder city never counts.
    Breach,
    /// Expand: gain `n` cities (founded or captured) after counting began.
    Cities { n: u16 },
    /// Expand: grow total population by `n`.
    Population { n: u16 },
    /// Expand: gain `n` tiles of territory.
    Territory { n: u16 },
    /// Build: `cities` more cities hold the building (or a later step of its family).
    Buildings { building: BuildingKind, cities: u16 },
    /// Build: learn `n` technologies.
    Techs { n: u16 },
    /// Fight: take `n` cities from other empires after entering the era.
    Captures { n: u16 },
    /// Fight: take `n` cities from empires ahead of you on VP.
    CapturesFromAhead { n: u16 },
    /// Fight: take `n` cities from the VP leader.
    CapturesFromLeader { n: u16 },
    /// Fight: destroy `n` enemy military units.
    Kills { n: u16 },
    /// Fight: take a city from a team holding its VP target, ending the hold.
    BreakLeaderHold,
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
            Objective::Techs { .. } => (
                u32::from(r.levels.techs),
                state.player(p).techs.len() as u32,
            ),
            Objective::Captures { .. } => (u32::from(r.base.captures), u32::from(f.captures)),
            Objective::CapturesFromAhead { .. } => {
                (u32::from(r.base.from_ahead), u32::from(f.from_ahead))
            }
            Objective::CapturesFromLeader { .. } => {
                (u32::from(r.base.from_leader), u32::from(f.from_leader))
            }
            Objective::Kills { .. } => (u32::from(r.base.kills), u32::from(f.kills)),
            Objective::HumbleTheMighty => (u32::from(r.base.humbled), u32::from(f.humbled)),
            Objective::Breach => (u32::from(r.base.walled), u32::from(f.walled)),
            Objective::Veterans { .. } => (u32::from(r.base.promotions), u32::from(f.promotions)),
            _ => return None,
        };
        Some(pair)
    }

    pub fn family(self) -> Family {
        match self {
            Objective::Cities { .. }
            | Objective::Population { .. }
            | Objective::Territory { .. } => Family::Expand,
            Objective::Buildings { .. } | Objective::Techs { .. } => Family::Build,
            _ => Family::Fight,
        }
    }

    pub fn target(self) -> u32 {
        match self {
            Objective::HumbleTheMighty | Objective::Breach => 1,
            Objective::Veterans { n }
            | Objective::Cities { n }
            | Objective::Population { n }
            | Objective::Territory { n }
            | Objective::Techs { n }
            | Objective::Captures { n }
            | Objective::CapturesFromAhead { n }
            | Objective::CapturesFromLeader { n }
            | Objective::Kills { n } => u32::from(n),
            Objective::Buildings { cities, .. } => u32::from(cities),
            Objective::BreakLeaderHold => 1,
        }
    }

    /// Plain-language rule, e.g. "Gain 5 cities".
    pub fn text(self) -> String {
        let n = self.target();
        let s = if n == 1 { "" } else { "s" };
        match self {
            Objective::Veterans { n } => format!(
                "Veterans: earn {n} promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)"
            ),
            Objective::HumbleTheMighty => {
                "Humble the mighty: capture a city from an empire ahead of you on VP".into()
            }
            Objective::Breach => "Breach: capture a walled city".into(),
            Objective::Cities { .. } => format!("Gain {n} cit{}", if n == 1 { "y" } else { "ies" }),
            Objective::Population { .. } => format!("Grow your population by {n}"),
            Objective::Territory { .. } => format!("Gain {n} tiles of territory"),
            Objective::Buildings { building, .. } => format!(
                "Add a {} to {n} more cit{}",
                data::building_name(building),
                if n == 1 { "y" } else { "ies" }
            ),
            Objective::Techs { .. } => format!("Learn {n} technologies"),
            Objective::Captures { .. } => {
                format!("Capture {n} enemy cit{}", if n == 1 { "y" } else { "ies" })
            }
            Objective::CapturesFromAhead { .. } => format!(
                "Capture {n} cit{} from an empire ahead of you on VP",
                if n == 1 { "y" } else { "ies" }
            ),
            Objective::CapturesFromLeader { .. } => format!(
                "Take {n} cit{} from the VP leader",
                if n == 1 { "y" } else { "ies" }
            ),
            Objective::Kills { .. } => format!("Destroy {n} enemy military unit{s}"),
            Objective::BreakLeaderHold => {
                "End a rival's victory hold by taking one of its cities".into()
            }
        }
    }

    /// How far empire `p` is towards it (not capped at the target), counting only what it gained
    /// since counting began for `era`: when it entered, or at its last era score if later. An era
    /// it has not entered counts nothing.
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
            Objective::Techs { .. } => gain(player.techs.len() as u32, &|l| u32::from(l.techs)),
            Objective::Captures { .. } => u32::from(fights().captures),
            Objective::CapturesFromAhead { .. } => u32::from(fights().from_ahead),
            Objective::CapturesFromLeader { .. } => u32::from(fights().from_leader),
            Objective::Kills { .. } => u32::from(fights().kills),
            Objective::BreakLeaderHold => u32::from(fights().holds_broken),
        }
    }

    /// Whether the turn-end check scores it.
    pub fn met(self, state: &GameState, p: PlayerId, era: u8) -> bool {
        self.progress(state, p, era) >= self.target()
    }
}

use Objective as O;

/// The pool each era draws from (1-based era: 2 Classical .. 9 Future; Ancient has none): about
/// eight objectives, three Build, three Expand and two or three Fight (each family must be
/// present, or the era has no objective in that family). Counts are for an 8-empire lobby
/// (`scaled_for` raises them for fewer) and count what an empire gains after entering the era and
/// after its last era score (R1), so Build and Expand counts are gains, not holdings. Eras stay
/// open for good. Each threshold sits near the 70th percentile of the best gain entrants made
/// (Fight measures count from entry too): about 60% of the empires that enter Classical score it,
/// falling to about 35% in the Future era (user, 2026-10-10; TI4 public objectives are the
/// baseline). Recalibrate with `vp_race.rs` peaks and `tools/sim/era-objectives.sh`:
/// `docs/sim-results/era-objectives/step1.md`. An era without a pool falls back to
/// `generic_pool`.
pub fn era_pool(era: u8) -> Vec<Objective> {
    match era {
        2 => vec![
            O::Buildings {
                building: BuildingKind::Laboratory,
                cities: 4,
            },
            O::Buildings {
                building: BuildingKind::Market,
                cities: 4,
            },
            O::Buildings {
                building: BuildingKind::Workshop,
                cities: 4,
            },
            O::Cities { n: 5 },
            O::Population { n: 50 },
            O::Territory { n: 80 },
            O::Captures { n: 2 },
            O::Kills { n: 30 },
        ],
        3 => vec![
            O::Buildings {
                building: BuildingKind::Laboratory,
                cities: 4,
            },
            O::Buildings {
                building: BuildingKind::Walls,
                cities: 4,
            },
            O::Buildings {
                building: BuildingKind::Castle,
                cities: 2,
            },
            O::Cities { n: 4 },
            O::Population { n: 42 },
            O::Territory { n: 62 },
            O::CapturesFromAhead { n: 1 },
            O::Kills { n: 38 },
        ],
        4 => vec![
            O::Buildings {
                building: BuildingKind::College,
                cities: 4,
            },
            O::Buildings {
                building: BuildingKind::Clearinghouse,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::Hydroponics,
                cities: 2,
            },
            O::Cities { n: 3 },
            O::Population { n: 34 },
            O::Territory { n: 49 },
            O::CapturesFromAhead { n: 1 },
            O::Kills { n: 38 },
        ],
        5 => vec![
            O::Buildings {
                building: BuildingKind::MachineShop,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::Autoworks,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::College,
                cities: 3,
            },
            O::Cities { n: 2 },
            O::Population { n: 25 },
            O::Territory { n: 35 },
            O::CapturesFromLeader { n: 1 },
            O::Kills { n: 29 },
        ],
        // Machine Age: a placeholder until its own pool (increment 2 of the Machine Age): the
        // World War pool (then called Modern) with the era's Stock Exchange in place of the Research
        // Lab.
        6 => vec![
            O::Buildings {
                building: BuildingKind::Autoworks,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::StockExchange,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::Clearinghouse,
                cities: 3,
            },
            O::Cities { n: 2 },
            O::Population { n: 23 },
            O::Territory { n: 31 },
            O::Captures { n: 1 },
            O::CapturesFromLeader { n: 1 },
            O::Kills { n: 30 },
        ],
        7 => vec![
            O::Buildings {
                building: BuildingKind::Autoworks,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::InferenceCore,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::Clearinghouse,
                cities: 3,
            },
            O::Cities { n: 2 },
            O::Population { n: 23 },
            O::Territory { n: 31 },
            O::Captures { n: 1 },
            O::CapturesFromLeader { n: 1 },
            O::Kills { n: 30 },
        ],
        // Modern (user, 2026-11-14): a placeholder until its own pool (increment 2 of the Modern
        // era): the Future's pool.
        8 | 9 => vec![
            O::Buildings {
                building: BuildingKind::InferenceCore,
                cities: 3,
            },
            O::Buildings {
                building: BuildingKind::StarFort,
                cities: 2,
            },
            O::Buildings {
                building: BuildingKind::Autoworks,
                cities: 2,
            },
            O::Cities { n: 2 },
            O::Population { n: 25 },
            O::Territory { n: 33 },
            O::Captures { n: 1 },
            O::CapturesFromLeader { n: 1 },
            O::Kills { n: 26 },
        ],
        _ => generic_pool(era),
    }
}

/// Promotions a Veterans card asks for in every Age.
const VETERANS_PROMOTIONS: u16 = 4;
/// Cities gained by a Gain N cities card. The card is only in the pool from the Renaissance on,
/// when free land is gone.
const CITIES_GAINED: u16 = 3;
/// The first era whose pool has a Gain N cities card (Renaissance).
const CITIES_FROM_ERA: u8 = 4;

/// The cards a World Age (era 2..9) can draw from. None is a stock the empire already holds, so
/// each counts only what is done after the reveal (cards never expire): Veterans counts promotions earned in battle; Humble the mighty and Breach are harder captures than a normal war;
/// Gain N cities counts from the Renaissance on.
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
    pool
}

/// A scaled pool for an era with no hand-written one.
fn generic_pool(era: u8) -> Vec<Objective> {
    let e = u16::from(era);
    vec![
        O::Buildings {
            building: BuildingKind::Workshop,
            cities: 2 + e,
        },
        O::Buildings {
            building: BuildingKind::Market,
            cities: 2 + e,
        },
        O::Techs { n: 4 * e + 2 },
        O::Cities { n: 2 + e },
        O::Population { n: 10 * e },
        O::Territory { n: 15 * e },
        O::CapturesFromAhead { n: 1 },
        O::Kills { n: 3 * e },
    ]
}

impl Objective {
    /// Count targets are written for an 8-empire lobby; fewer empires share the land and each
    /// grows bigger, so they rise 12% per missing empire (a 4-empire lobby asks 48% more).
    fn scaled_for(self, empires: usize, objective_scale: u16) -> Objective {
        let lobby = 100 + 12 * 8usize.saturating_sub(empires.max(2)) as u32;
        let percent = (lobby * u32::from(objective_scale) + 50) / 100;
        let scale = |n: u16| ((u32::from(n) * percent + 50) / 100) as u16;
        match self {
            Objective::Cities { n } => Objective::Cities { n: scale(n) },
            Objective::Population { n } => Objective::Population { n: scale(n) },
            Objective::Territory { n } => Objective::Territory { n: scale(n) },
            Objective::Buildings { building, cities } => Objective::Buildings {
                building,
                cities: scale(cities),
            },
            Objective::Kills { n } => Objective::Kills { n: scale(n) },
            other => other,
        }
    }
}

/// The era's whole pool at the game's counts (`scaled_for`), in `era_pool` order.
pub fn scaled_pool(setup: &crate::state::GameSetup, era: u8) -> Vec<Objective> {
    if setup.rule_experiments.objectives_deck {
        return age_pool(era);
    }
    let empires = setup.players.len();
    let objective_scale = setup.rule_experiments.objective_scale_percent();
    era_pool(era)
        .into_iter()
        .map(|o| o.scaled_for(empires, objective_scale))
        .collect()
}

/// The public objectives of an era (one each from Build, Expand and Fight; with World Ages, the single card drawn for the Age), identical for every empire in a
/// game, each with its index in `era_pool`. Empty for era 1 and for eras beyond the tree.
pub fn drawn_from_pool(setup: &crate::state::GameSetup, era: u8) -> Vec<(usize, Objective)> {
    if era < 2 || era > data::era_count() {
        return Vec::new();
    }
    let seed = setup.seed;
    let pool = scaled_pool(setup, era);
    if setup.rule_experiments.objectives_deck {
        // One card per Age, drawn at random (user, 2026-11-11: "only one, randomly decided option per age").
        let mut rng = Rng::derive(seed, &[u64::from(era), purpose::OBJECTIVES, 97]);
        let i = rng.range(0, pool.len() as i64 - 1) as usize;
        return vec![(i, pool[i])];
    }
    Family::ALL
        .iter()
        .enumerate()
        .filter_map(|(slot, family)| {
            let candidates: Vec<usize> = (0..pool.len())
                .filter(|i| pool[*i].family() == *family)
                .collect();
            if candidates.is_empty() {
                return None;
            }
            let mut rng = Rng::derive(seed, &[u64::from(era), purpose::OBJECTIVES, slot as u64]);
            let i = candidates[rng.range(0, candidates.len() as i64 - 1) as usize];
            Some((i, pool[i]))
        })
        .collect()
}

/// The public objectives of an era (one each from Build, Expand and Fight; with World Ages, the single card drawn for the Age), identical for every empire in a
/// game. Empty for era 1 and for eras beyond the tree.
pub fn era_objectives(setup: &crate::state::GameSetup, era: u8) -> Vec<Objective> {
    drawn_from_pool(setup, era)
        .into_iter()
        .map(|(_, o)| o)
        .collect()
}

/// The slot an objective takes in its era's list: 0 for the single World Age card,
/// or the index of its family in the original era objectives.
pub fn slot_in(setup: &crate::state::GameSetup, era: u8, objective: Objective) -> u8 {
    if setup.rule_experiments.objectives_deck {
        return era_objectives(setup, era)
            .iter()
            .position(|o| *o == objective)
            .unwrap_or(0) as u8;
    }
    Family::ALL
        .iter()
        .position(|f| *f == objective.family())
        .unwrap_or(0) as u8
}

/// The era an empire is in, 1-based (1 Ancient .. 9 Future): `eras::current_era` + 1. It rises
/// the turn the previous era's last technology lands (`Event::EraEntered`). Only ever rises.
pub fn era_of(state: &GameState, p: PlayerId) -> u8 {
    crate::eras::current_era(&state.player(p).techs) + 1
}

/// The team strictly ahead on VP, if exactly one is.
#[derive(Clone, Debug)]
pub struct CaptureVpSnapshot {
    vp: BTreeMap<TeamId, u32>,
    leader: Option<TeamId>,
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
        let best = vp.values().copied().max();
        let mut top = vp.iter().filter(|(_, v)| Some(**v) == best);
        let leader = top
            .next()
            .and_then(|(team, _)| top.next().is_none().then_some(*team));
        Self { vp, leader }
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
    let mut kills: Vec<PlayerId> = Vec::new();
    for e in events {
        if let Event::Battle {
            attacker,
            attacker_owner,
            defender,
            defender_hp_after,
            attacker_hp_after,
            ..
        } = e
        {
            let defender_owner = match defender {
                Defender::Unit { owner, .. } | Defender::City { owner, .. } => *owner,
            };
            if *defender_hp_after == 0
                && matches!(defender, Defender::Unit { .. })
                && !attacker_owner.is_barbarian()
                && !defender_owner.is_barbarian()
            {
                kills.push(*attacker_owner);
            }
            if *attacker_hp_after == 0
                && matches!(attacker, Attacker::Unit(_))
                && !attacker_owner.is_barbarian()
                && !defender_owner.is_barbarian()
            {
                kills.push(defender_owner);
            }
        }
    }
    // VictoryHoldBroken is emitted later in scoring::check_victory. At this point the
    // post-combat VP is already known, while vp_holds still contains last turn's hold.
    let broken: Vec<TeamId> = events
        .iter()
        .filter_map(|e| match e {
            Event::VictoryHoldBroken { team, .. } => Some(*team),
            _ => None,
        })
        .collect();
    for t in taken {
        let (taker, victim) = (t.taker, t.victim);
        if taker.is_barbarian() || victim.is_barbarian() || taker == victim {
            continue;
        }
        let victim_team = state.team_of(victim);
        let ahead = vp.ahead(victim_team, state.team_of(taker));
        let from_leader = vp.leader == Some(victim_team) && state.team_of(taker) != victim_team;
        let hold_broken = broken.contains(&victim_team);
        let f = &mut state.player_mut(taker).fight;
        f.captures = f.captures.saturating_add(1);
        if ahead {
            f.from_ahead = f.from_ahead.saturating_add(1);
            if !t.prize {
                f.humbled = f.humbled.saturating_add(1);
            }
        }
        if t.walled && !t.prize {
            f.walled = f.walled.saturating_add(1);
        }
        if from_leader {
            f.from_leader = f.from_leader.saturating_add(1);
        }
        if hold_broken {
            f.holds_broken = f.holds_broken.saturating_add(1);
        }
    }
    for owner in kills {
        let f = &mut state.player_mut(owner).fight;
        f.kills = f.kills.saturating_add(1);
    }
}

/// End-of-turn: credit fights, open the eras each empire has entered (World Ages: when the first
/// empire enters an era it opens for every living empire at once), and score any era whose
/// objective is met.
pub fn update(state: &mut GameState, events: &mut Vec<Event>, vp: &CaptureVpSnapshot) {
    let this_turn = events.clone();
    record_fights(state, &this_turn, vp);
    let setup = state.setup.clone();
    let world_ages = setup.rule_experiments.objectives_deck;
    let turn = state.turn;
    let ids: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    let newest = ids.iter().map(|p| era_of(state, *p)).max().unwrap_or(1);
    let opened_before = revealed_era(state);
    if world_ages {
        for era in opened_before + 1..=newest {
            if let Some(first) = state.era_firsts.get(&(era - 1)) {
                events.push(Event::AgeBegan { era, first: *first });
            }
        }
    }
    for p in ids {
        let reached = if world_ages {
            newest.max(revealed_era(state))
        } else {
            era_of(state, p)
        };
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
