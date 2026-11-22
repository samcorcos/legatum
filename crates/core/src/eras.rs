//! Eras and the technology table (era overhaul, phase A1).
//!
//! The rules in one place:
//! - Nine eras: Ancient, Classical, Medieval, Renaissance, Industrial, Machine Age, World War,
//!   Modern, Future (the Machine Age added 2026-11-14 between Industrial and the era then called
//!   Modern, which became World War when the new Modern, 1950 to 2010, was added after it). Each
//!   empire is in the first era that still has a technology it does not know (`current_era`).
//! - The strict gate (user, 2026-10-10): an empire researches only technologies of its current
//!   era, with no prerequisites inside an era, and enters the next era the turn it finishes the
//!   era's last technology (`rules::cities::process_research` emits `Event::EraEntered`).
//! - No boosts (user, 2026-10-17): a technology's cost depends only on the table, tech diffusion
//!   and the game's rule experiments (`economy::tech_cost`), never on what the empire has done.
//!
//! Stable API for other systems (victory points, AI, UI): `ERA_COUNT`, `era_name`, `current_era`,
//! `era_of` (a player in a game) and `GameState::era_of`.
//!
//! ## Updating the Future era (a living data set)
//! The Future era mirrors technology that is still unfolding, so it is meant to change. To add
//! or replace a Future technology: (1) add a variant at the end of `TechId` in `types.rs` and to
//! `TechId::ALL` and `TechId::from_name`; (2) add a row with `era: LAST_ERA` at the bottom of `TECHS`
//! below, in the same order; (3) add its card effects in `data::tech_card_effect_list` and, if
//! it unlocks an item, set that item's `requires`. The Future era may hold more than five
//! technologies; nothing assumes five. Nothing else needs to change: the gate, costs,
//! the AI and the research screen read this table.

use crate::state::{GameState, Player};
use crate::types::*;
use std::collections::BTreeSet;

pub const ERA_COUNT: usize = 9;
/// The era an empire starts in.
pub const FIRST_ERA: u8 = 0;
/// The last era; it is never "finished" into a further one.
pub const LAST_ERA: u8 = (ERA_COUNT - 1) as u8;
pub const ERA_NAMES: [&str; ERA_COUNT] = [
    "Ancient",
    "Classical",
    "Medieval",
    "Renaissance",
    "Industrial",
    "Machine Age",
    "World War",
    "Modern",
    "Future",
];

pub fn era_name(era: u8) -> &'static str {
    ERA_NAMES[usize::from(era).min(ERA_COUNT - 1)]
}

/// One technology. Costs are Science (`economy::tech_cost` applies diffusion).
pub struct TechDef {
    pub id: TechId,
    pub name: &'static str,
    pub era: u8,
    pub cost: i32,
}

const fn t(id: TechId, name: &'static str, era: u8, cost: i32) -> TechDef {
    TechDef {
        id,
        name,
        era,
        cost,
    }
}

/// Every technology in `TechId` order (`TECHS[tech as usize]`). Five per era today; the Future
/// rows at the bottom are the living part (see the module note on updating them).
pub static TECHS: [TechDef; TechId::ALL.len()] = {
    use TechId::*;
    [
        // Ancient
        t(Irrigation, "Irrigation", 0, 24),
        t(BronzeWorking, "Bronze Working", 0, 26),
        t(Masonry, "Masonry", 0, 30),
        t(Currency, "Currency", 0, 32),
        t(Writing, "Writing", 0, 38),
        // Classical
        t(Mathematics, "Mathematics", 1, 48),
        t(Engineering, "Engineering", 1, 48),
        t(IronWorking, "Iron Working", 1, 56),
        t(CodeOfLaws, "Code of Laws", 1, 62),
        t(Philosophy, "Philosophy", 1, 68),
        // Medieval
        t(Feudalism, "Feudalism", 2, 120),
        t(Machinery, "Machinery", 2, 126),
        t(Chivalry, "Chivalry", 2, 132),
        t(Education, "Education", 2, 136),
        t(CropRotation, "Crop Rotation", 2, 140),
        // Renaissance
        t(Gunpowder, "Gunpowder", 3, 216),
        t(Metallurgy, "Metallurgy", 3, 224),
        t(MilitaryEngineering, "Military Engineering", 3, 230),
        t(Banking, "Banking", 3, 238),
        t(PrintingPress, "Printing Press", 3, 248),
        // Industrial
        t(Rifling, "Rifling", 4, 344),
        t(MilitaryScience, "Military Science", 4, 352),
        t(Cannonry, "Ballistics", 4, 356),
        t(Industrialization, "Industrialization", 4, 368),
        t(Railways, "Railways", 4, 376),
        // Machine Age (user, 2026-11-14): the five technologies moved down from Modern, at their
        // costs before rules 31 (the +30% came back off when this era took over its job).
        t(ReplaceableParts, "Replaceable Parts", 5, 388),
        t(Combustion, "Combustion", 5, 394),
        t(Steel, "Steel", 5, 402),
        t(Flight, "Flight", 5, 414),
        t(Electricity, "Electricity", 5, 422),
        // World War (the era first called Modern): each cost is the geometric midpoint of the
        // Machine Age and Modern technology in the same slot (sqrt(388 * 654) = 504, ...).
        t(MassMobilisation, "Mass Mobilisation", 6, 504),
        t(Rocketry, "Rocketry", 6, 514),
        t(ArmouredWarfare, "Armoured Warfare", 6, 525),
        t(AirPower, "Air Power", 6, 539),
        t(Electronics, "Electronics", 6, 551),
        // Modern (user, 2026-11-14, 1950 to 2010): the costs the Future had before it, 650 to 720.
        t(MechanisedWarfare, "Mechanised Warfare", 7, 654),
        t(CompositeArmour, "Composite Armour", 7, 670),
        t(GuidedMissiles, "Guided Missiles", 7, 686),
        t(JetEngines, "Jet Engines", 7, 702),
        t(GreenRevolution, "Green Revolution", 7, 720),
        // Future: the living era. Update here (see the module note). One step up the same cost
        // curve (about +30% per era) from where it stood before the Modern era was added.
        t(AutonomousSystems, "Autonomous Systems", 8, 840),
        t(Robotics, "Robotics", 8, 862),
        t(FusionPower, "Fusion Power", 8, 884),
        t(StealthTechnology, "Stealth Technology", 8, 906),
        t(RecursiveAi, "Recursive AI", 8, 930),
    ]
};

pub fn tech_def(tech: TechId) -> &'static TechDef {
    let def = &TECHS[tech as usize];
    debug_assert_eq!(def.id, tech, "TECHS must follow TechId order");
    def
}

/// The era (0-based) a technology belongs to.
pub fn era_of_tech(tech: TechId) -> u8 {
    tech_def(tech).era
}

/// The technologies of one era, in table order.
pub fn era_techs(era: u8) -> impl Iterator<Item = TechId> {
    TECHS.iter().filter(move |d| d.era == era).map(|d| d.id)
}

/// The era an empire knowing exactly `techs` is in: the first with a technology still unknown;
/// the last era once everything is known.
#[inline(never)]
pub fn current_era(techs: &BTreeSet<TechId>) -> u8 {
    // TechId order matches TECHS, and the eras are contiguous in that order. The first gap
    // in the known prefix is therefore the first era with an unknown technology. Walking the
    // set once avoids scanning TECHS for every era and looking up each technology again.
    let mut next = 0;
    for tech in techs {
        if *tech as usize != next {
            break;
        }
        next += 1;
    }
    TECHS.get(next).map_or(LAST_ERA, |def| def.era)
}

/// Whether the empire has researched every technology of every era.
pub fn all_researched(techs: &BTreeSet<TechId>) -> bool {
    TechId::ALL.iter().all(|t| techs.contains(t))
}

/// A player's current era in a game. Barbarians are in the first era.
pub fn era_of(state: &GameState, player: PlayerId) -> u8 {
    if player.is_barbarian() {
        return FIRST_ERA;
    }
    current_era(&state.player(player).techs)
}

impl GameState {
    /// This player's current era, 0 (Ancient) to `ERA_COUNT - 1` (Future).
    pub fn era_of(&self, player: PlayerId) -> u8 {
        era_of(self, player)
    }
}

/// Settle "first into an era" between empires that entered it on the same turn. `learn` runs one
/// empire at a time, so the lower id would always be first; after the turn's research this picks
/// the first among every empire that entered an era this turn, when that era had no first
/// before the turn, by a seeded draw (`purpose::ERA_FIRST`), and rewrites `era_firsts` and the
/// `first` flags of this turn's `EraEntered` events to match.
pub fn draw_same_turn_firsts(state: &mut GameState, events: &mut [crate::events::Event]) {
    use crate::events::Event;
    // 1-based eras first claimed this turn.
    let claimed: Vec<u8> = events
        .iter()
        .filter_map(|e| match e {
            Event::EraEntered {
                era, first: true, ..
            } => Some(*era),
            _ => None,
        })
        .collect();
    for era in claimed {
        let mut entrants: Vec<PlayerId> = events
            .iter()
            .filter_map(|e| match e {
                Event::EraEntered { player, era: e, .. } if *e == era => Some(*player),
                _ => None,
            })
            .collect();
        entrants.sort();
        entrants.dedup();
        if entrants.len() < 2 {
            continue;
        }
        let first = crate::rng::seeded_order(
            state.dice_seed(),
            &[
                u64::from(state.turn),
                crate::rng::purpose::ERA_FIRST,
                u64::from(era),
            ],
            &entrants,
        )[0];
        state.era_firsts.insert(era - 1, first);
        for e in events.iter_mut() {
            if let Event::EraEntered {
                player,
                era: e_era,
                first: f,
            } = e
                && *e_era == era
            {
                *f = *player == first;
            }
        }
    }
}

/// Whether the empire may research `tech` now: unknown and in its current era.
pub fn researchable(p: &Player, tech: TechId) -> bool {
    !p.techs.contains(&tech) && era_of_tech(tech) == current_era(&p.techs)
}

/// Gives `player` a technology: records it, emits `TechResearched`, and emits `EraEntered` for
/// each era the empire enters because of it. The one way a technology is learned.
pub fn learn(
    state: &mut GameState,
    player: PlayerId,
    tech: TechId,
    events: &mut Vec<crate::events::Event>,
) {
    use crate::events::Event;
    let before = current_era(&state.player(player).techs);
    let first_time = state.player_mut(player).techs.insert(tech);
    events.push(Event::TechResearched { player, tech });
    if first_time && tech == TechId::CodeOfLaws {
        let cities: Vec<_> = state
            .cities
            .values()
            .filter(|city| city.owner == player)
            .map(|city| city.id)
            .collect();
        for city in cities {
            crate::rules::cities::claim_code_of_laws_tile(state, city, events);
        }
    }
    if first_time && tech == TechId::RecursiveAi {
        crate::visibility::reveal_all_to_team(state, player);
    }
    let after = current_era(&state.player(player).techs);
    for era in before + 1..=after {
        let first = !state.era_firsts.contains_key(&era);
        state.era_firsts.entry(era).or_insert(player);
        events.push(Event::EraEntered {
            player,
            era: era + 1,
            first,
        });
    }
}
