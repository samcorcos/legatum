//! Game data tables and tuning constants. Starting values; balance comes from simulation.

use crate::experiments::{FactionRule, FactionRuleSet};
use crate::types::*;

pub const UNIT_HP: i32 = 100;
pub const CITY_HP: i32 = 200;
pub const CITY_BASE_STRENGTH: i32 = 10;
pub const CITY_STRENGTH_PER_POP: i32 = 2;

/// How much stronger each era's melee line is than the one before, in percent (user, 2026-10-19:
/// the curve accelerates, each step 5 points bigger than the last: 25, 30, 35, 40, 45, 50). The
/// melee line is `ANCIENT_MELEE_STRENGTH` grown by these steps, so the early steps are small
/// (+5 for a 20-strength Spearman) and the late ones huge (+45 from Modern to Future), and a unit
/// two or more eras ahead is decisive. The numbers to tune. Index 0 is Ancient to Classical.
/// The Machine Age (user, 2026-11-14) splits the old 45% Industrial-to-Modern step in two,
/// 21% then 20% (1.21 x 1.20 = 1.45), so every World War and Future strength was unchanged. The
/// Modern era (user, 2026-11-14) adds a 20% step after the World War era (the era first called
/// Modern): its units stand about 20% above the World War unit of the same line, and the Future,
/// one step on, keeps its 50% over the era before it.
pub const ERA_STEP_PERCENT: [i32; CURVE_ERAS - 1] = [25, 30, 35, 40, 21, 20, 20, 50];
/// The Ancient melee line, the anchor the curve grows from.
pub const ANCIENT_MELEE_STRENGTH: i32 = 20;
/// Number of eras (`eras::ERA_COUNT`), for the curve tables.
const CURVE_ERAS: usize = crate::eras::ERA_COUNT;

/// The melee line's strength in each era: `ANCIENT_MELEE_STRENGTH` grown by `ERA_STEP_PERCENT`
/// step by step from the exact (unrounded) product, then rounded to the nearest whole number
/// (a .5 rounds down): 20, 25, 32, 44, 61, 74, 89, 107, 161.
pub const MELEE_LINE: [i32; CURVE_ERAS] = melee_line();

const fn melee_line() -> [i32; CURVE_ERAS] {
    let mut line = [0; CURVE_ERAS];
    let (mut num, mut den) = (ANCIENT_MELEE_STRENGTH as i64, 1_i64);
    let mut era = 0;
    while era < CURVE_ERAS {
        line[era] = ((2 * num + den - 1) / (2 * den)) as i32;
        if era < CURVE_ERAS - 1 {
            num *= (100 + ERA_STEP_PERCENT[era]) as i64;
            den *= 100;
        }
        era += 1;
    }
    line
}

/// The old straight-line melee table (20 + 6 per era). Every design number in the unit and
/// building tables (a Knight's 32, a Walls tier's 10) was balanced against its era's value of this
/// line, which fixes how a class compares with the melee line of its own era. `era_scaled` keeps
/// those relations and puts them on the new curve, so same-era balance is unchanged and only the
/// gap between eras grows. The Machine Age, inserted later, takes 49: between its neighbours, and
/// the one value that puts its units on the proposal's numbers (Trench Infantry 74, Howitzer
/// 62/80, EarlyTank 86, Biplane 60). The Modern era takes 50 like the World War era before it, so
/// each Modern unit keeps its World War line's design number and stands one 20% step above it.
pub const DESIGN_MELEE_LINE: [i32; CURVE_ERAS] = [20, 26, 32, 38, 44, 49, 50, 50, 56];

/// Pre-Modern mounted units stand this percent of their era's melee line (user, 2026-10-31, the
/// Civ 6 way: its Horseman is 36 against the Spearman's 25, and the spear's bonus only evens the
/// fight). They were at or below the line (Horseman 18 against the Spearman's 20), so the class
/// bonus made every charge into spears a massacre and nobody built them. The Tank and the
/// Autonomous Armour, past the triangle, keep their own numbers.
pub const MOUNTED_STRENGTH_PERCENT: i32 = 130;

/// The design strength of the pre-Modern mounted unit of `era`: `MOUNTED_STRENGTH_PERCENT` of
/// that era's design melee line, rounded half up.
const fn mounted_design_strength(era: usize) -> i32 {
    (DESIGN_MELEE_LINE[era] * MOUNTED_STRENGTH_PERCENT + 50) / 100
}

/// A design strength of a unit or building in `era` (0 Ancient .. 8 Future) on the era curve:
/// `design * MELEE_LINE[era] / DESIGN_MELEE_LINE[era]`, rounded half up (half away from zero for
/// a negative penalty).
pub fn era_scaled(design: i32, era: u8) -> i32 {
    let e = usize::from(era).min(CURVE_ERAS - 1);
    let (num, den) = (i64::from(MELEE_LINE[e]), i64::from(DESIGN_MELEE_LINE[e]));
    let scaled = i64::from(design.abs()) * num;
    (((2 * scaled + den) / (2 * den)) as i32) * design.signum()
}
pub const HEAL_FIELD: i32 = 10;
pub const HEAL_TERRITORY: i32 = 20;
/// HP a city regains at the end of a turn, unless it took damage that turn or an at-war
/// military unit is adjacent (`rules::cities::heal_cities`). Empty to full takes 10 turns.
pub const CITY_HEAL: i32 = 20;
/// Fortification: +3 defense per full idle turn, up to +6. Cleared by moving or attacking.
pub const FORTIFY_PER_TURN: i32 = 3;
pub const FORTIFY_MAX: i32 = 6;
/// A captured city produces nothing for this many turns.
pub const CAPTURE_RESISTANCE_TURNS: i32 = 6;

// Promotions (Civ 6's experience model, without promotion choices). See rules/promotion.rs.
/// Cumulative XP for promotion levels 1..=5 (Civ 6's 15/45/90/150/225).
pub const PROMOTION_XP: [i32; 5] = [16, 46, 90, 150, 226];
/// Combat strength per promotion level, as a percent of the unit's base strength.
pub const PROMOTION_STRENGTH_PERCENT: i32 = 10;
/// Unit-vs-unit XP: enemy base strength / own base strength (doubled if the enemy died),
/// plus these bonuses, rounded half up and capped (Civ 6).
pub const XP_MELEE_BONUS: i32 = 2;
pub const XP_RANGED_BONUS: i32 = 1;
pub const XP_INITIATOR_BONUS: i32 = 1;
pub const XP_UNIT_FIGHT_CAP: i32 = 8;
/// Attacking a city; the attack that drops it to 0 HP earns `XP_CITY_DEFEATED` instead.
/// A city already at 0 HP gives nothing (Civ 6).
pub const XP_CITY_ATTACK: i32 = 3;
pub const XP_CITY_DEFEATED: i32 = 10;

/// The game's expected length in turns: the horizon forecasts and AI pacing measure against.
pub const EXPECTED_GAME_TURNS: u32 = 300;
/// The first third of the expected game: Sparta's peace grace and the AI's opening phase.
pub const FIRST_THIRD_TURNS: u32 = EXPECTED_GAME_TURNS / 3;

/// A city may not be founded within this many hexes of another (distance must be >= this + 1).
pub const MIN_CITY_GAP: i32 = 3;
/// India may found cities at distance 3; other factions need distance 4.
pub const INDIA_MIN_CITY_GAP: i32 = 2;
pub const fn min_city_gap(faction: Faction) -> i32 {
    min_city_gap_rules(FactionRuleSet::EMPTY, faction)
}

/// `min_city_gap` with the faction rules in `off` switched off (ablation).
pub const fn min_city_gap_rules(off: FactionRuleSet, faction: Faction) -> i32 {
    if matches!(faction, Faction::India) && !off.contains(FactionRule::IndiaCloseCities) {
        INDIA_MIN_CITY_GAP
    } else {
        MIN_CITY_GAP
    }
}
/// Population is territory: a city works every tile it owns within this many hexes. Tiles it
/// claims farther out are owned but unworked (they only block rivals).
pub const CITY_WORK_RADIUS: i32 = 3;
/// Gold price of a tile bought for a city: base plus a step per tile the city has gained since
/// founding (grown or bought), times the ring percent below.
pub const TILE_GOLD_BASE: i32 = 40;
pub const TILE_GOLD_PER_TILE: i32 = 20;
/// A bought tile costs this much more per ring beyond the first (ring = hex distance from the
/// city, 1 to `CITY_WORK_RADIUS`), as a percent of the base price: 100%, 125%, 150%.
pub const TILE_RING_STEP_PERCENT: i32 = 25;
pub const CITY_CENTER_YIELDS: Yields = Yields::new(2, 1, 1, 0);
/// Minimum production from the center and worked tiles, before buildings and faction penalties.
pub const CITY_MIN_PRODUCTION: i32 = 3;
/// Irrigation: extra Food in every city (it does not unlock the Granary, which anyone can build
/// from turn 1).
pub const IRRIGATION_FOOD: i32 = 1;
pub const CAPITAL_BASE_SCIENCE: i32 = 2;

pub const GROWTH_BASE: i32 = 30;
pub const GROWTH_PER_POP: i32 = 16;
/// Food for the first growth (population 1 to 2); see `growth_threshold`.
/// Population is territory, so ring one is worked from the start (about +10 Food a turn): 65 Food
/// puts population 2, and the first claimed tile, around turn 7 (65 Food pairs with the full-price
/// first Settler to hold the first Settler near turn 18 at Standard).
pub const FIRST_GROWTH: i32 = 65;
pub const FOOD_PER_POP: i32 = 2;
pub const SETTLER_MIN_POP: i32 = 2;

/// Gold every empire starts with.
pub const START_GOLD: i32 = 60;
/// Military units per empire (not per city) with no gold upkeep. Scouts never count
/// (`pays_upkeep`).
pub const FREE_MILITARY_UNITS: i32 = 2;
pub const UPKEEP_PER_UNIT: i32 = 1;
pub const BUY_COST_PERCENT: i32 = 200;
pub const SWISS_BUY_PERCENT: i32 = 175;
/// Upgrading costs this percent of the production cost difference, in gold, in own territory.
pub const UPGRADE_COST_PERCENT: i32 = 200;
/// Settlers cost this plus SETTLER_COST_STEP per settler the empire already trained.
pub const SETTLER_BASE_COST: i32 = 70;
/// An empire's first Settler costs this share of its price.
/// 100% since population became territory: a new city works its whole first ring, so the first
/// discount that kept the first Settler near turn 18 at Standard is no longer needed.
pub const FIRST_SETTLER_PERCENT: i32 = 100;
pub const SETTLER_COST_STEP: i32 = 50;
/// "Most Advanced" compares science generated over this many recent turns.
pub const SCIENCE_HISTORY_TURNS: usize = 10;
/// The Heir's Production cost (provisional, to calibrate): expensive on purpose, and the Byzantium
/// can keep only one (`goals`).
pub const HEIR_COST: i32 = 800;
/// The Courier Hall's Production cost (provisional, to calibrate): pricey, one per Mongols
/// empire, and it opens the holder's whole land to every empire.
pub const YAM_RELAY_STATION_COST: i32 = 600;
/// Roadworks: Production cost, and the range in hexes of the cities its city is linked to.
pub const ROADWORKS_COST: i32 = 140;
/// Siege (user decision 2026-10-22): a Gold upgrade that makes
/// one land military unit ignore enemy Walls when it attacks a city, for good. The price is this
/// many Gold for an Ancient unit (the Engineer's old 80 Production, at the upgrade rate of 2 Gold
/// a Production), scaled by the unit's era like the units themselves (`siege_gold`).
pub const SIEGE_BASE_GOLD: i32 = 160;
/// Each Siege unit uses 1 Iron supply while it exists, as the Pikeman does (user, 2026-10-22:
/// "not sure" about a resource, so this is the one switch: `false` makes Siege cost Gold only).
pub const SIEGE_USES_IRON: bool = true;
pub const ROADWORKS_RANGE: i32 = 8;
pub const RAILWAYS_ROADWORKS_RANGE: i32 = 12;
/// Ruhr Foundries Germany must hold in its cities for its live goal.
pub const GERMANY_GOAL_CITIES: i32 = 5;

/// Each other living empire that knows a technology makes it this percent cheaper.
pub const TECH_DIFFUSION_PER_EMPIRE: i32 = 10;
/// The most diffusion can take off (era overhaul: raised from 30 to 50, start value).
pub const TECH_DIFFUSION_MAX: i32 = 50;
/// Railways: extra moves for a unit that starts its turn on a road (1 without them).
pub const ROAD_MOVES: i32 = 1;
pub const RAILWAY_MOVES: i32 = 2;

pub const UNIT_SIGHT: i32 = 2;
pub const CITY_SIGHT: i32 = 3;

/// Extra Food in every India city from its city network.
pub const INDIA_NETWORK_FOOD: i32 = 1;
pub const INDIA_CAPITAL_RADIUS: i32 = 4;
pub const INDIA_NETWORK_RADIUS: i32 = 5;
/// Linear loss of city yields per hex beyond `INDIA_CAPITAL_RADIUS`.
pub const INDIA_YIELD_PERCENT_PER_HEX: i32 = 10;

/// Ten percentage points per extra hex, capped at 100%.
/// Clamping before multiplication keeps even an arbitrarily large distance safe.
pub fn india_distance_loss_percent(beyond: i32) -> i32 {
    INDIA_YIELD_PERCENT_PER_HEX * beyond.clamp(0, 10)
}
pub const INDIA_NETWORK_PERCENT_PER_CITY: i32 = 7;
pub const IMPERIAL_DIWAN_COST: i32 = 300;
/// Gold in the treasury that earns a Swiss city set +1 Gold a turn in interest. There is no maximum
/// (user, 2026-10-17); there is no maximum.
pub const SWISS_INTEREST_THRESHOLD: i32 = 100;
/// Population that earns a Swiss city +1 Gold a turn ("tolls"): one Gold for every this many people.
pub const SWISS_TOLL_POPULATION: i32 = 2;

/// Consecutive turn ends a team must hold its VP target to win by VP.
pub const VP_HOLD_TURNS: u32 = 10;
pub const MAP_WIDTH: i32 = 48;
pub const MAP_HEIGHT: i32 = 32;
/// Every faction goal is worth exactly this (user rule, 2026-10-06): goals are tuned by their
/// threshold or cost, never their VP (`goals::spec`).
pub const FACTION_GOAL_VP: u32 = 2;

pub struct UnitStats {
    pub cost: i32,
    pub moves: i32,
    /// Melee strength (attack and defense).
    pub strength: i32,
    /// Ranged attack strength; 0 = no ranged attack.
    pub ranged: i32,
    pub range: i32,
    pub military: bool,
    pub requires: Option<TechId>,
    /// The unit this one can upgrade to.
    pub upgrades_to: Option<UnitKind>,
}

/// A unit's stats on the era curve: `design_stats` with its strengths scaled to its era.
pub fn unit_stats(kind: UnitKind) -> UnitStats {
    scale_to_era(design_stats(kind))
}

fn scale_to_era(mut s: UnitStats) -> UnitStats {
    let era = s
        .requires
        .map_or(crate::eras::FIRST_ERA, crate::eras::era_of_tech);
    s.strength = era_scaled(s.strength, era);
    s.ranged = era_scaled(s.ranged, era);
    s
}

/// The stats as designed, before the era curve: strengths on the old straight line, which sets
/// each unit against the melee line of its own era (`DESIGN_MELEE_LINE`).
pub fn design_stats(kind: UnitKind) -> UnitStats {
    let (cost, moves, strength, ranged, range, military, requires, upgrades_to) = match kind {
        UnitKind::Scout => (60, 3, 5, 0, 0, true, None, None),
        // Settler cost here is the base; see economy::production_cost for the rising cost.
        UnitKind::Settler => (SETTLER_BASE_COST, 2, 0, 0, 0, false, None, None),
        UnitKind::HeirToTheThrone => (HEIR_COST, 2, 0, 0, 0, false, None, None),
        // One unit per class in every era (user, 2026-10-10). The melee line (the spear line):
        // Spearman, Phalangite, Pikeman, Musketman, Rifleman, Infantry, Hunter-Killer.
        UnitKind::Spearman => (80, 2, 20, 0, 0, true, None, Some(UnitKind::Phalangite)),
        UnitKind::Phalangite => (
            110,
            2,
            26,
            0,
            0,
            true,
            Some(TechId::Mathematics),
            Some(UnitKind::Pikeman),
        ),
        UnitKind::Pikeman => (
            152,
            2,
            32,
            0,
            0,
            true,
            Some(TechId::Feudalism),
            Some(UnitKind::Musketman),
        ),
        UnitKind::Musketman => (
            210,
            2,
            38,
            0,
            0,
            true,
            Some(TechId::Gunpowder),
            Some(UnitKind::Rifleman),
        ),
        UnitKind::Rifleman => (
            290,
            2,
            44,
            0,
            0,
            true,
            Some(TechId::Rifling),
            Some(UnitKind::TrenchInfantry),
        ),
        UnitKind::TrenchInfantry => (
            340,
            2,
            49,
            0,
            0,
            true,
            Some(TechId::ReplaceableParts),
            Some(UnitKind::Infantry),
        ),
        UnitKind::Infantry => (
            400,
            2,
            50,
            0,
            0,
            true,
            Some(TechId::MassMobilisation),
            Some(UnitKind::MechanisedInfantry),
        ),
        // The Modern era's Infantry: the same design strength one era step up, and the melee
        // line's only unit that keeps pace with armour (`MECHANISED_INFANTRY_MOVES`).
        UnitKind::MechanisedInfantry => (
            470,
            MECHANISED_INFANTRY_MOVES,
            50,
            0,
            0,
            true,
            Some(TechId::MechanisedWarfare),
            Some(UnitKind::HunterKiller),
        ),
        UnitKind::HunterKiller => (552, 2, 56, 0, 0, true, Some(TechId::Robotics), None),
        // The ranged line: Archer, Composite Archer, Crossbowman, Arquebusier, Field Cannon,
        // Artillery, Combat Drone.
        UnitKind::Archer => (
            132,
            2,
            8,
            15,
            2,
            true,
            None,
            Some(UnitKind::CompositeArcher),
        ),
        UnitKind::CompositeArcher => (
            168,
            2,
            15,
            23,
            2,
            true,
            Some(TechId::Engineering),
            Some(UnitKind::Crossbowman),
        ),
        UnitKind::Crossbowman => (
            214,
            2,
            22,
            31,
            2,
            true,
            Some(TechId::Machinery),
            Some(UnitKind::Arquebusier),
        ),
        UnitKind::Arquebusier => (
            272,
            2,
            29,
            39,
            2,
            true,
            Some(TechId::Gunpowder),
            Some(UnitKind::FieldCannon),
        ),
        UnitKind::FieldCannon => (
            346,
            2,
            36,
            47,
            2,
            true,
            Some(TechId::Cannonry),
            Some(UnitKind::Howitzer),
        ),
        UnitKind::Howitzer => (
            390,
            2,
            41,
            53,
            2,
            true,
            Some(TechId::Steel),
            Some(UnitKind::Artillery),
        ),
        UnitKind::Artillery => (
            440,
            2,
            43,
            55,
            2,
            true,
            Some(TechId::Rocketry),
            Some(UnitKind::RocketArtillery),
        ),
        // Rocket Artillery: the Artillery's design numbers one era step up, and one more tile
        // of range (`ROCKET_ARTILLERY_RANGE`). It fires indirectly like the guns before it.
        UnitKind::RocketArtillery => (
            495,
            2,
            43,
            55,
            ROCKET_ARTILLERY_RANGE,
            true,
            Some(TechId::GuidedMissiles),
            Some(UnitKind::CombatDrone),
        ),
        UnitKind::CombatDrone => (
            560,
            3,
            50,
            63,
            2,
            true,
            Some(TechId::AutonomousSystems),
            None,
        ),
        // The mounted line, four moves: Horseman, Cataphract, Knight, Cuirassier, Cavalry,
        // Tank, Autonomous Armour. Before the Modern era a rider stands well above its era's
        // melee line (`MOUNTED_STRENGTH_PERCENT`) and costs more.
        UnitKind::Horseman => (
            110,
            MOUNTED_MOVES,
            mounted_design_strength(0),
            0,
            0,
            true,
            None,
            Some(UnitKind::Cataphract),
        ),
        UnitKind::Cataphract => (
            150,
            MOUNTED_MOVES,
            mounted_design_strength(1),
            0,
            0,
            true,
            Some(TechId::IronWorking),
            Some(UnitKind::Knight),
        ),
        UnitKind::Knight => (
            204,
            MOUNTED_MOVES,
            mounted_design_strength(2),
            0,
            0,
            true,
            Some(TechId::Chivalry),
            Some(UnitKind::Cuirassier),
        ),
        UnitKind::Cuirassier => (
            278,
            MOUNTED_MOVES,
            mounted_design_strength(3),
            0,
            0,
            true,
            Some(TechId::Metallurgy),
            Some(UnitKind::Cavalry),
        ),
        UnitKind::Cavalry => (
            378,
            MOUNTED_MOVES,
            mounted_design_strength(4),
            0,
            0,
            true,
            Some(TechId::MilitaryScience),
            Some(UnitKind::EarlyTank),
        ),
        // The EarlyTank: the first armour, slow (`EARLY_TANK_MOVES`) and below the 130% rider rule,
        // since it is a machine, not a horse.
        UnitKind::EarlyTank => (
            440,
            EARLY_TANK_MOVES,
            57,
            0,
            0,
            true,
            Some(TechId::Combustion),
            Some(UnitKind::Tank),
        ),
        UnitKind::Tank => (
            514,
            TANK_MOVES,
            53,
            0,
            0,
            true,
            Some(TechId::ArmouredWarfare),
            Some(UnitKind::MainBattleTank),
        ),
        // The Main Battle Tank: the Tank's design numbers one era step up, the same 5 moves.
        UnitKind::MainBattleTank => (
            600,
            TANK_MOVES,
            53,
            0,
            0,
            true,
            Some(TechId::CompositeArmour),
            Some(UnitKind::AutonomousArmour),
        ),
        UnitKind::AutonomousArmour => (
            700,
            AUTONOMOUS_ARMOUR_MOVES,
            60,
            0,
            0,
            true,
            Some(TechId::AutonomousSystems),
            None,
        ),
        // The air layer (Machine Age on): based in a city, never on a map tile. `range` is the
        // strike range from the city: a Biplane's is the shortest, a Fighter's short (it hits
        // units with its own strength, no city bonus), a Bomber's long. A Fighter's interception
        // radius is `FIGHTER_RANGE`, a Biplane's `BIPLANE_GUARD_RANGE`.
        UnitKind::Biplane => (
            340,
            1,
            40,
            40,
            BIPLANE_STRIKE_RANGE,
            true,
            Some(TechId::Flight),
            Some(UnitKind::Fighter),
        ),
        UnitKind::Fighter => (
            400,
            1,
            58,
            58,
            FIGHTER_STRIKE_RANGE,
            true,
            Some(TechId::AirPower),
            Some(UnitKind::JetFighter),
        ),
        // The jets: the Fighter's and Bomber's design numbers one era step up, with one more
        // tile of reach each (`JET_FIGHTER_STRIKE_RANGE`, `JET_BOMBER_RANGE`).
        UnitKind::JetFighter => (
            470,
            1,
            58,
            58,
            JET_FIGHTER_STRIKE_RANGE,
            true,
            Some(TechId::JetEngines),
            Some(UnitKind::StealthFighter),
        ),
        UnitKind::JetBomber => (
            540,
            1,
            36,
            64,
            JET_BOMBER_RANGE,
            true,
            Some(TechId::JetEngines),
            Some(UnitKind::StealthBomber),
        ),
        UnitKind::Bomber => (
            460,
            1,
            36,
            64,
            BOMBER_RANGE,
            true,
            Some(TechId::AirPower),
            Some(UnitKind::JetBomber),
        ),
        UnitKind::StealthFighter => (
            552,
            1,
            58,
            58,
            7,
            true,
            Some(TechId::StealthTechnology),
            None,
        ),
        UnitKind::StealthBomber => (
            636,
            1,
            36,
            64,
            10,
            true,
            Some(TechId::StealthTechnology),
            None,
        ),
    };
    UnitStats {
        cost,
        moves,
        strength,
        ranged,
        range,
        military,
        requires,
        upgrades_to,
    }
}

/// The class of a combat unit; None for Scouts and civilians. The triangle (user, 2026-10-10):
/// the melee line beats mounted, ranged beats melee, mounted beats ranged; siege supports.
pub fn unit_class(kind: UnitKind) -> Option<UnitClass> {
    use UnitKind::*;
    match kind {
        Spearman | Phalangite | Pikeman | Musketman | Rifleman | TrenchInfantry | Infantry
        | MechanisedInfantry | HunterKiller => Some(UnitClass::Melee),
        Archer | CompositeArcher | Crossbowman | Arquebusier | FieldCannon | Howitzer
        | Artillery | RocketArtillery | CombatDrone => Some(UnitClass::Ranged),
        Horseman | Cataphract | Knight | Cuirassier | Cavalry | EarlyTank | Tank
        | MainBattleTank | AutonomousArmour => Some(UnitClass::Mounted),
        Biplane | Fighter | Bomber | JetFighter | JetBomber | StealthFighter | StealthBomber => {
            Some(UnitClass::Air)
        }
        Scout | Settler | HeirToTheThrone => None,
    }
}

/// Aircraft each city's hangar holds.
pub const AIR_BASE_CAP: usize = 2;
pub const STEALTH_AIR_BASE_CAP: usize = 3;
pub fn air_base_cap(techs: &std::collections::BTreeSet<TechId>) -> usize {
    if techs.contains(&TechId::StealthTechnology) {
        STEALTH_AIR_BASE_CAP
    } else {
        AIR_BASE_CAP
    }
}
pub fn fighter_guard_radius(kind: UnitKind) -> i32 {
    match kind {
        UnitKind::StealthFighter => 5,
        UnitKind::Biplane => BIPLANE_GUARD_RANGE,
        _ => FIGHTER_RANGE,
    }
}
/// The forms a unit of this kind may upgrade into next: its `upgrades_to`, and for the Biplane
/// the Bomber as well (user, 2026-11-14: the player chooses Fighter or Bomber, the one branching
/// upgrade in the game). The first is the default.
pub fn upgrade_choices(kind: UnitKind) -> Vec<UnitKind> {
    let mut choices: Vec<UnitKind> = unit_stats(kind).upgrades_to.into_iter().collect();
    if kind == UnitKind::Biplane {
        choices.push(UnitKind::Bomber);
    }
    choices
}

/// Whether an aircraft guards: it intercepts enemy strikes near its city unless it strikes itself.
/// The Biplane guards too, within `BIPLANE_GUARD_RANGE`.
pub fn is_fighter(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::Biplane | UnitKind::Fighter | UnitKind::JetFighter | UnitKind::StealthFighter
    )
}
pub fn is_bomber(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::Bomber | UnitKind::JetBomber | UnitKind::StealthBomber
    )
}
/// A Bomber strikes a tile this many hexes from its base, a Fighter (shorter, user 2026-10-17)
/// this many; a Fighter on guard intercepts strikes on tiles `FIGHTER_RANGE` hexes from its base.
pub const BOMBER_RANGE: i32 = 8;
pub const FIGHTER_STRIKE_RANGE: i32 = 5;
pub const FIGHTER_RANGE: i32 = 4;
/// The Modern jets (Jet Engines, user, 2026-11-14): the Jet Fighter strikes one tile farther than
/// the Fighter and guards the same `FIGHTER_RANGE`; the Jet Bomber reaches one tile farther than
/// the Bomber.
pub const JET_FIGHTER_STRIKE_RANGE: i32 = 6;
pub const JET_BOMBER_RANGE: i32 = 9;
/// Rocket Artillery (Guided Missiles): a shot one tile farther than the Artillery's 2.
pub const ROCKET_ARTILLERY_RANGE: i32 = 3;
/// Mechanised Infantry (Mechanised Warfare): the infantry that keeps up with armour.
pub const MECHANISED_INFANTRY_MOVES: i32 = 3;
/// The Biplane (Flight, Machine Age; user, 2026-11-14: "smaller radius and not overwhelming
/// damage"): it strikes this many hexes from its city and guards this many.
pub const BIPLANE_STRIKE_RANGE: i32 = 3;
pub const BIPLANE_GUARD_RANGE: i32 = 2;
/// A Biplane's strike on a city deals this percent of its normal damage to the city's HP: its
/// light bombs did little to a city before the Second World War.
pub const BIPLANE_CITY_DAMAGE_PERCENT: i32 = 50;
/// An aircraft in a hangar lifts the fog this many tiles around its city (the Biplane's spotting
/// job, the Howitzer fires at what its team sees; user, 2026-11-15: every aircraft that succeeds
/// it sees as far, so a Bomber's strike target can be seen).
pub const AIRCRAFT_SIGHT: i32 = 4;

/// An aircraft's Recon (`Order::AirRecon`, user, 2026-11-15) shows the tiles within this many of
/// its target tile, terrain ignored, for the next turn: less than its base `AIRCRAFT_SIGHT`.
pub const RECON_RADIUS: i32 = 2;

/// Percent of its normal damage this kind deals to a city's HP, before Walls
/// (`combat::city_damage_percent`): 50 for the Biplane, 100 for everything else.
pub fn city_hp_damage_percent(kind: UnitKind) -> i32 {
    if kind == UnitKind::Biplane {
        BIPLANE_CITY_DAMAGE_PERCENT
    } else {
        100
    }
}

/// Whether a unit kind is an aircraft: it lives in a city's hangar (`GameState::aircraft`).
pub fn is_aircraft(kind: UnitKind) -> bool {
    unit_class(kind) == Some(UnitClass::Air)
}

/// Moves of every mounted unit (user, 2026-10-10: "mounted; 4 moves").
pub const MOUNTED_MOVES: i32 = 4;
/// The Tank outruns the rest of the mounted class (user, 2026-10-19: Tanks get 5 moves), and the
/// Autonomous Armour that replaces it outruns the Tank (6; the coordinator's correction of the same day).
pub const TANK_MOVES: i32 = 5;
/// The EarlyTank, the Machine Age's first armour, is slow (proposal 2026-11-14: "armour is slow").
pub const EARLY_TANK_MOVES: i32 = 3;
pub const AUTONOMOUS_ARMOUR_MOVES: i32 = 6;

/// The Keshig, the Mongol horse archer (`unit_stats_for`): the Mongols' Classical mounted unit
/// (user, 2026-10-17: "the Mongol horde was in the 1200s... premature to give it right at the
/// start"), so the Mongols build ordinary Horsemen in the Ancient era. Its Production price, and
/// the ranged attack it has on top of the Cataphract's melee strength and moves. Set with the
/// combat scenario simulator (`combat_scenarios --keshig`, `docs/sim-results/unit-classes/keshig.txt`).
pub const KESHIG_KIND: UnitKind = UnitKind::Cataphract;
pub const KESHIG_COST: i32 = 164;
pub const KESHIG_RANGED: i32 = 18;
pub const KESHIG_RANGE: i32 = 2;

/// Whether a unit fights by closing in (the melee line, mounted units, Scouts): it can capture a
/// city and its threat is a charge. A shooter's only move onto an enemy is a desperate one. The
/// Keshig (a mounted unit with a bow) both charges and shoots; for every other unit this is
/// "has no ranged attack". Base stats, so the answer is the same for every faction.
pub fn attacks_in_melee(kind: UnitKind) -> bool {
    unit_stats(kind).ranged == 0 || unit_class(kind) == Some(UnitClass::Mounted)
}

pub fn class_name(class: UnitClass) -> &'static str {
    match class {
        UnitClass::Melee => "Melee line",
        UnitClass::Ranged => "Ranged",
        UnitClass::Mounted => "Mounted",
        UnitClass::Air => "Air",
    }
}

/// The era (0-based, `crate::eras`) a unit belongs to: the era of the technology that unlocks it,
/// or Ancient for units available from the start.
pub fn unit_era_tier(kind: UnitKind) -> u8 {
    design_stats(kind)
        .requires
        .map_or(crate::eras::FIRST_ERA, crate::eras::era_of_tech)
}

/// Air Defence: a Star Fort's price, for a city building with no yield.
pub const AIR_DEFENCE_COST: i32 = 320;

pub struct BuildingStats {
    pub cost: i32,
    pub yields: Yields,
    pub city_strength: i32,
    pub requires: Option<TechId>,
    pub war_building: bool,
}

pub fn building_stats(kind: BuildingKind) -> BuildingStats {
    use BuildingKind::*;
    let (cost, yields, city_strength, requires) = match kind {
        // The former Japan building is reserved for its future redesign.
        RiceStorehouse => (50, Yields::new(2, 0, 0, 0), 0, None),
        // Buildable from turn 1 (Irrigation does not unlock it).
        Granary => (50, Yields::new(2, 0, 0, 0), 0, None),
        Workshop => (60, Yields::new(0, 2, 0, 0), 0, Some(TechId::BronzeWorking)),
        Market => (60, Yields::new(0, 0, 3, 0), 0, Some(TechId::Currency)),
        // Library
        Library => (80, Yields::new(0, 0, 0, 3), 0, Some(TechId::Writing)),
        Walls => (100, Yields::ZERO, 10, Some(TechId::Masonry)),
        // Mill
        Mill => (200, Yields::new(3, 0, 0, 0), 0, Some(TechId::Feudalism)),
        // Forge
        Forge => (
            // A production ladder must leave time to punish its construction window.
            320,
            Yields::new(0, 3, 0, 0),
            0,
            Some(TechId::IronWorking),
        ),
        Agoge => (160, Yields::ZERO, 0, Some(TechId::Chivalry)),
        // University
        University => (240, Yields::new(0, 0, 0, 4), 0, Some(TechId::Education)),
        // Research Lab
        ResearchLab => (360, Yields::new(0, 0, 0, 6), 0, Some(TechId::Electronics)),
        // Stock Exchange: the Gold line's third tier (Market, Bank), priced like the Research Lab.
        StockExchange => (360, Yields::new(0, 0, 6, 0), 0, Some(TechId::Electricity)),
        // Supermarket: the Food line's third tier (Granary, Mill), priced like the Stock Exchange.
        Supermarket => (
            360,
            Yields::new(4, 0, 0, 0),
            0,
            Some(TechId::GreenRevolution),
        ),
        // Air Defence: no yields; it intercepts air strikes on its own city (`rules::air`).
        AirDefence => (
            AIR_DEFENCE_COST,
            Yields::ZERO,
            0,
            Some(TechId::GuidedMissiles),
        ),
        // Bank
        Bank => (200, Yields::new(0, 0, 4, 0), 0, Some(TechId::Banking)),
        // Factory
        Factory => (
            440,
            Yields::new(0, 5, 0, 0),
            0,
            Some(TechId::Industrialization),
        ),
        // Walls tier 2: each tier adds to the one below (`combat::city_defense_with_fort`).
        Castle => (240, Yields::new(0, 0, 0, 0), 5, Some(TechId::Chivalry)),
        // Walls tier 3. Machinery, the Medieval tech whose Siege reaches Castles, also brings the
        // answer (rules 50; was Military Engineering, one era later, when Industrial units with
        // built-in Siege followed one era after it and left the Star Fort little time to matter).
        StarFort => (320, Yields::new(0, 0, 0, 0), 5, Some(TechId::Machinery)),
        // World wonders: each is worth the same 2 held VP, so prices follow one era curve (rules 50,
        // a first pass the owner may veto: Pyramids 240 -> 360 and Great Library 600 -> 400 were
        // the two outliers more than 25% off it; the rest are within it).
        // Great Library
        GreatLibrary => (400, Yields::ZERO, 0, Some(TechId::Philosophy)),
        Pyramids => (360, Yields::new(0, 1, 0, 0), 0, Some(TechId::Masonry)),
        HagiaSophia => (400, Yields::new(0, 0, 0, 2), 0, Some(TechId::Machinery)),
        EiffelTower => (720, Yields::new(0, 0, 3, 0), 0, Some(TechId::Steel)),
        IterFusionReactor => (900, Yields::new(0, 0, 0, 4), 0, Some(TechId::FusionPower)),
        // Royal Arsenal
        RoyalArsenal => (600, Yields::ZERO, 0, Some(TechId::Cannonry)),
        // Grand Bazaar
        GrandBazaar => (600, Yields::ZERO, 0, Some(TechId::Banking)),
        YamRelayStation => (YAM_RELAY_STATION_COST, Yields::ZERO, 0, None),
        ImperialDiwan => (IMPERIAL_DIWAN_COST, Yields::ZERO, 0, Some(TechId::Currency)),
        Roadworks => (
            ROADWORKS_COST,
            Yields::new(0, 0, 1, 0),
            0,
            Some(TechId::Engineering),
        ),
    };
    BuildingStats {
        cost,
        yields,
        city_strength,
        requires,
        war_building: matches!(kind, Walls | Castle | StarFort | Agoge),
    }
}

pub struct TechStats {
    /// Listed Science (`economy::tech_cost` applies diffusion).
    pub cost: i32,
}

/// The technology table lives in `crate::eras`.
pub fn tech_stats(tech: TechId) -> TechStats {
    TechStats {
        cost: crate::eras::tech_def(tech).cost,
    }
}

/// What an immediate research effect changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardEffect {
    /// Adds only ordinary city Science.
    CityScience,
    /// Upgrades a horse-mounted unit, unavailable to the Aztecs.
    HorseUpgrade,
    /// Concerns Siege (the Gold upgrade that ignores Walls).
    Siege,
    /// Adds only Production, which the Nile takes from Egypt.
    Production,
    Other,
}

/// Short descriptions of effects granted immediately by research. Paid unit
/// and building unlocks come from `research::tech_unlocks` instead. Kept with
/// the core rules so both clients render the same technology card metadata.
pub fn tech_card_effect_list(tech: TechId) -> &'static [(&'static str, CardEffect)] {
    use CardEffect::*;
    use TechId::*;
    match tech {
        Irrigation => &[("+1 Food in every city", Other)],
        CodeOfLaws => &[("Every city claims its best available ring-2 tile", Other)],
        Mathematics => &[
            ("Spearmen can upgrade to Phalangites", Other),
            ("Units can buy Siege: they ignore enemy Walls", Siege),
        ],
        Engineering => &[("Archers can upgrade to Composite Archers", Other)],
        IronWorking => &[
            (
                "Mines upgrade: Deep Mine (+1 Gold on Copper, +1 Production on Iron and Coal)",
                Other,
            ),
            ("Horsemen can upgrade to Cataphracts", HorseUpgrade),
        ],
        Feudalism => &[("Phalangites can upgrade to Pikemen", Other)],
        Machinery => &[
            ("Composite Archers can upgrade to Crossbowmen", Other),
            ("Siege units now ignore Castles", Siege),
            ("+1 Production per Workshop family", Production),
        ],
        CropRotation => &[
            (
                "Farms and Pastures upgrade: Irrigated Farm, Ranch (+1 Food)",
                Other,
            ),
            ("+1 Food per Granary family", Other),
        ],
        Gunpowder => &[
            ("Pikemen can upgrade to Musketmen", Other),
            ("Crossbowmen can upgrade to Arquebusiers", Other),
        ],
        Metallurgy => &[
            ("Knights can upgrade to Cuirassiers", HorseUpgrade),
            ("Siege units now ignore Star Forts", Siege),
        ],
        MilitaryEngineering => &[("Upgrade Gold costs -50%", Other)],
        PrintingPress => &[
            ("+1 Science per Library family", CityScience),
            ("+1 Gold per Market family", Other),
        ],
        Chivalry => &[("Cataphracts can upgrade to Knights", HorseUpgrade)],
        Rifling => &[("Musketmen can upgrade to Riflemen", Other)],
        MilitaryScience => &[
            ("Cuirassiers can upgrade to Cavalry", HorseUpgrade),
            ("Land units cost 10% less Production", Other),
        ],
        Cannonry => &[("Arquebusiers can upgrade to Field Cannons", Other)],
        Railways => &[
            ("Units that start a turn on a road get +2 moves", Other),
            ("Roadworks link cities within 12 tiles", Other),
            ("Mines +1 Production", Production),
        ],
        ReplaceableParts => &[
            ("Riflemen can upgrade to Trench Infantry", Other),
            ("Factories +2 Production", Production),
        ],
        Combustion => &[
            ("Cavalry can upgrade to Early Tanks", HorseUpgrade),
            ("Oil Wells +1 Gold", Other),
        ],
        Steel => &[("Field Cannons can upgrade to Howitzers", Other)],
        MassMobilisation => &[("Trench Infantry can upgrade to Infantry", Other)],
        Rocketry => &[("Howitzers can upgrade to Artillery", Other)],
        ArmouredWarfare => &[("Early Tanks can upgrade to Tanks", Other)],
        AirPower => &[("Each Biplane can upgrade to either one, your choice", Other)],
        MechanisedWarfare => &[("Infantry can upgrade to Mechanised Infantry", Other)],
        CompositeArmour => &[("Tanks can upgrade to Main Battle Tanks", Other)],
        GuidedMissiles => &[("Artillery can upgrade to Rocket Artillery", Other)],
        JetEngines => &[
            ("Fighters can upgrade to Jet Fighters", Other),
            ("Bombers can upgrade to Jet Bombers", Other),
        ],
        AutonomousSystems => &[
            ("Rocket Artillery can upgrade to Combat Drones", Other),
            ("Main Battle Tanks can upgrade to Autonomous Armour", Other),
        ],
        Robotics => &[("Mechanised Infantry can upgrade to Hunter-Killers", Other)],
        FusionPower => &[
            ("+5 Production in every city", Production),
            ("Factories and Research Labs need no Coal or Oil", Other),
        ],
        StealthTechnology => &[
            (
                "Jet Fighters can upgrade to Stealth Fighters: strike up to 7 tiles away; guard 5 tiles around their city",
                Other,
            ),
            (
                "Jet Bombers can upgrade to Stealth Bombers: strike up to 10 tiles away; only Stealth Fighters intercept them",
                Other,
            ),
            (
                "Stealth aircraft use 1 Silicon each; earlier aircraft keep Oil",
                Other,
            ),
            ("Hangars hold 3 aircraft", Other),
        ],
        RecursiveAi => &[
            (
                "Reveal the whole map to your team; units still need sight",
                Other,
            ),
            ("+20% Production in every city", Production),
        ],
        Masonry => &[("Quarries upgrade: Deep Quarry (+1 Production)", Production)],
        Philosophy => &[("+1 Science in every city", CityScience)],
        BronzeWorking | Currency | Writing | Education | Banking | Industrialization | Flight
        | Electricity | Electronics | GreenRevolution => &[],
    }
}

/// Every immediate effect of a technology, for any faction.
pub fn tech_card_effects(tech: TechId) -> Vec<&'static str> {
    tech_card_effect_list(tech)
        .iter()
        .map(|(label, _)| *label)
        .collect()
}

/// The immediate effects this faction can use: Aztecs have no horse units to upgrade, and Egypt
/// (the Nile) has no Production to add to.
pub fn tech_card_effects_for(faction: Faction, tech: TechId) -> Vec<&'static str> {
    tech_card_effects_for_rules(FactionRuleSet::EMPTY, faction, tech)
}

/// `tech_card_effects_for` with the faction rules in `off` switched off (ablation).
pub fn tech_card_effects_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    tech: TechId,
) -> Vec<&'static str> {
    let no_horses = faction == Faction::Aztecs && !off.contains(FactionRule::AztecsNoHorses);
    let no_production = faction == Faction::Egypt && !off.contains(FactionRule::EgyptNile);
    let effects: Vec<&'static str> = tech_card_effect_list(tech)
        .iter()
        .filter(|(_, effect)| !no_horses || *effect != CardEffect::HorseUpgrade)
        .filter(|(_, effect)| !no_production || *effect != CardEffect::Production)
        .map(|(label, _)| *label)
        .collect();
    effects
}

/// Every unit and building, in the order research cards list them.
pub fn production_items() -> impl Iterator<Item = ProductionItem> {
    BuildingKind::ALL
        .into_iter()
        .map(|building| ProductionItem::Building { building })
        .chain(
            UnitKind::ALL
                .into_iter()
                .map(|unit| ProductionItem::Unit { unit }),
        )
}

/// Whether researching `tech` directly gives this faction anything: an item it can build that
/// does something for it, an immediate effect, or a strategic resource reveal. A faction skips a
/// tech without a benefit (`skipped_techs`): it never researches a card that does nothing.
pub fn tech_has_benefit(
    rules: crate::experiments::RuleExperiments,
    faction: Faction,
    tech: TechId,
) -> bool {
    !tech_card_effects_for_rules(rules.faction_rules_off, faction, tech).is_empty()
        || tech_reveals_resource(tech)
        || production_items().any(|item| {
            item_requires_for_rules(rules, item) == Some(tech)
                && available_to_faction_rules(rules.faction_rules_off, faction, item)
        })
}

/// Technologies that give this faction nothing (`tech_has_benefit` is false). Every empire starts
/// knowing them, so no faction ever researches a card that does nothing for it (user,
/// 2026-11-22: "there shouldn't be a tech that you get just to advance the era"). Egypt skips
/// Bronze Working: the Nile leaves it no Production, so the Workshop is unavailable to it.
pub fn skipped_techs(
    rules: crate::experiments::RuleExperiments,
    faction: Faction,
) -> impl Iterator<Item = TechId> {
    TechId::ALL
        .into_iter()
        .filter(move |&tech| !tech_has_benefit(rules, faction, tech))
}

/// Whether this faction skips `tech` (`skipped_techs`).
pub fn tech_skipped(
    rules: crate::experiments::RuleExperiments,
    faction: Faction,
    tech: TechId,
) -> bool {
    !tech_has_benefit(rules, faction, tech)
}

/// Whether `tech` reveals a strategic resource. Every faction's later buildings and units draw
/// on each era's resource, so a reveal is a benefit on its own (Egypt's Industrialization: Coal).
fn tech_reveals_resource(tech: TechId) -> bool {
    crate::resources::Resource::ALL
        .into_iter()
        .any(|r| crate::resources::stats(r).revealed_by == Some(tech))
}

/// State-aware technology benefit test for factions with unlocks earned during play.
pub fn tech_has_player_benefit(
    rules: crate::experiments::RuleExperiments,
    faction: Faction,
    era: u8,
    tech: TechId,
) -> bool {
    !tech_card_effects_for_rules(rules.faction_rules_off, faction, tech).is_empty()
        || tech_reveals_resource(tech)
        || production_items().any(|item| {
            item_requires_for_rules(rules, item) == Some(tech)
                && available_to_player_rules(rules.faction_rules_off, faction, era, item)
        })
}

/// The era (0 Ancient, 1 Classical, 2 Medieval) from which the Mongols raise Walls, Castles and
/// Star Forts and buy Siege (user, 2026-10-19 and 2026-10-31). Before it, neither is available.
pub const MONGOL_WALLS_ERA: u8 = 2;

/// Whether `item` is one of the Mongols' era-gated buildings: the Walls line.
fn is_mongol_gated(item: ProductionItem) -> bool {
    matches!(
        item,
        ProductionItem::Building {
            building: BuildingKind::Walls | BuildingKind::Castle | BuildingKind::StarFort
        }
    )
}

/// Whether a player can build an item now, including the Mongols' Medieval-era start for the
/// Walls line. `era` is the player's era.
pub fn available_to_player(faction: Faction, era: u8, item: ProductionItem) -> bool {
    available_to_player_rules(FactionRuleSet::EMPTY, faction, era, item)
}

/// `available_to_player` with the faction rules in `off` switched off (ablation).
pub fn available_to_player_rules(
    off: FactionRuleSet,
    faction: Faction,
    era: u8,
    item: ProductionItem,
) -> bool {
    if !available_to_faction_rules(off, faction, item) {
        return false;
    }
    faction != Faction::Mongols
        || era >= MONGOL_WALLS_ERA
        || !is_mongol_gated(item)
        || off.contains(FactionRule::MongolsNoWalls)
}

/// Whether a player may buy Siege now: the Mongols only from the Medieval era.
pub fn siege_available(faction: Faction, era: u8) -> bool {
    siege_available_rules(FactionRuleSet::EMPTY, faction, era)
}

/// `siege_available` with the faction rules in `off` switched off (ablation).
pub fn siege_available_rules(off: FactionRuleSet, faction: Faction, era: u8) -> bool {
    faction != Faction::Mongols
        || era >= MONGOL_WALLS_ERA
        || off.contains(FactionRule::MongolsNoWalls)
}

/// Why the Mongols cannot buy Siege yet.
pub const MONGOL_SIEGE_REASON: &str = "No Siege before the Medieval era.";

/// The first era (0-based, `unit_era_tier`) whose land military units have Siege built in:
/// Industrial (designer, 2026-10-28). Rifleman, Cavalry, Field Cannon and every later land unit
/// fight walled cities as if Siege covered every Walls tier, whatever their owner knows.
pub const BUILT_IN_SIEGE_ERA: u8 = 4;

/// The highest Walls tier (`wall_tier`): a Star Fort.
pub const MAX_WALL_TIER: u8 = 3;

/// What built-in Siege does, after the unit's name, in one wording for the order refusal, the
/// rules reference and the app (`rules.generated.json` `siege.builtInNote`).
pub const BUILT_IN_SIEGE_NOTE: &str = "ignores Walls, Castles and Star Forts (Industrial-era guns)";

/// Land military units: not Scouts, not aircraft.
fn land_military(kind: UnitKind) -> bool {
    unit_stats(kind).military && kind != UnitKind::Scout && !is_aircraft(kind)
}

/// Whether the auto-explore order (`Automation::Explore`) accepts this kind: Scouts and every
/// land military unit (designer, 2026-11-14). Civilians never explore on their own, and
/// aircraft stay in a hangar. There are no ships.
pub fn can_auto_explore(kind: UnitKind) -> bool {
    kind == UnitKind::Scout || land_military(kind)
}

/// Whether a unit of this kind has Siege built in: a land military unit of the Industrial era
/// or later (`BUILT_IN_SIEGE_ERA`). It is never bought, uses no Iron, and covers every tier.
pub fn built_in_siege(kind: UnitKind) -> bool {
    land_military(kind) && unit_era_tier(kind) >= BUILT_IN_SIEGE_ERA
}

/// Steel's gun line can fire over terrain when its team sees the target.
pub fn indirect_fire(kind: UnitKind, knows_steel: bool) -> bool {
    knows_steel
        && matches!(
            kind,
            UnitKind::Howitzer
                | UnitKind::Artillery
                | UnitKind::RocketArtillery
                | UnitKind::CombatDrone
        )
}

/// Units that can be given Siege: land military units (not Scouts, not aircraft) before the
/// Industrial era. Later ones have it built in (`built_in_siege`).
pub fn can_have_siege(kind: UnitKind) -> bool {
    land_military(kind) && !built_in_siege(kind)
}

/// Whether a unit fights walled cities with Siege: built in, or bought (`bought`, `Unit::siege`).
pub fn has_siege(kind: UnitKind, bought: bool) -> bool {
    built_in_siege(kind) || (bought && !is_aircraft(kind))
}

/// The Walls tier a unit's attacks on a city ignore, the one source for the rule
/// (`rules::battle::siege_cover`, `forecast::siege_cover`, the AI): every tier for built-in
/// Siege, else the owner's siege tier (`siege_tier`) when the unit bought Siege (`bought_with`
/// holds the owner's technologies then, `None` when it has none), else 0. Aircraft get 0.
pub fn siege_cover(kind: UnitKind, bought_with: Option<&std::collections::BTreeSet<TechId>>) -> u8 {
    if built_in_siege(kind) {
        MAX_WALL_TIER
    } else {
        bought_with
            .filter(|_| !is_aircraft(kind))
            .map_or(0, siege_tier)
    }
}

/// Gold for Siege on a unit of this kind: `SIEGE_BASE_GOLD`, scaled by the unit's era like the
/// units themselves (`era_scaled`), so Siege stays about as dear next to an army as it was in the
/// Ancient era.
pub fn siege_gold(kind: UnitKind) -> i32 {
    era_scaled(SIEGE_BASE_GOLD, unit_era_tier(kind))
}

/// The Spartan Hoplite's phalanx (user, 2026-10-17: a unique piece must differ from what it
/// replaces): strength a Hoplite gains, attacking or defending, while another Hoplite of its empire
/// stands on an adjacent tile at the fight's stage. One neighbour is enough (a line does not
/// stack). Only the Spearman kind is a Hoplite: an upgraded unit is a Phalangite and loses it.
pub const HOPLITE_PHALANX_STRENGTH: i32 = 5;

/// Whether this faction's unit of this kind is a Spartan Hoplite.
pub fn is_hoplite(faction: Faction, kind: UnitKind) -> bool {
    is_hoplite_rules(FactionRuleSet::EMPTY, faction, kind)
}

/// `is_hoplite` with the faction rules in `off` switched off: without the phalanx rule the Spartan
/// Spearman is an ordinary Spearman (ablation).
pub fn is_hoplite_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Sparta
        && kind == UnitKind::Spearman
        && !off.contains(FactionRule::SpartaPhalanx)
}

/// The six faction special units (user, 2026-11-14: every faction fields one, as in Civ 6). Each
/// replaces one plain unit for its faction only; the numbers are design numbers, so they ride the
/// era curve (`era_scaled`) like every other faction edge. `unit_stats_made_by_rules` holds the
/// stat changes, `combat` the conditional strength lines.
///
/// Aztec Eagle Warrior (replaces the Spearman): this much stronger, this much more Production.
pub const EAGLE_WARRIOR_STRENGTH: i32 = 4;
pub const EAGLE_WARRIOR_COST: i32 = 10;
/// Celtic Gaesatae (replaces the Phalangite): strength gained attacking (and in a meeting
/// engagement), and lost defending (a negative number): naked, furious charge, no shield wall.
pub const GAESATAE_ATTACK_STRENGTH: i32 = 8;
pub const GAESATAE_DEFENSE_STRENGTH: i32 = -5;
/// Athenian Peltast (replaces the Composite Archer): moves instead of 2, the same shot.
pub const PELTAST_MOVES: i32 = 3;
/// Indian War Elephant (replaces the Knight): this much stronger, with these moves instead of
/// the Knight's 4, and no mounted penalty charging a city.
pub const WAR_ELEPHANT_STRENGTH: i32 = 8;
pub const WAR_ELEPHANT_MOVES: i32 = 3;
/// Swiss Reisläufer (replaces the Pikeman): percent of its strength gained against mounted
/// units, on top of the melee line's own bonus against them.
pub const REISLAUFER_MOUNTED_PERCENT: i32 = 50;
/// German Panzer (replaces the Tank): moves gained.
pub const PANZER_MOVES: i32 = 1;

/// The unit kind a faction's special unit replaces (`unit_name_for` names it): every faction has
/// exactly one (user, 2026-11-14). Mercenary Companies sell nothing else (`company`).
pub fn special_unit(faction: Faction) -> UnitKind {
    match faction {
        Faction::Switzerland => UnitKind::Pikeman,
        Faction::Mongols => KESHIG_KIND,
        Faction::Sparta | Faction::Aztecs => UnitKind::Spearman,
        Faction::Germany => UnitKind::Tank,
        Faction::India => UnitKind::Knight,
        Faction::Byzantium => UnitKind::Pikeman,
        Faction::Celts => UnitKind::Phalangite,
        Faction::Athens => UnitKind::CompositeArcher,
        Faction::Egypt => UnitKind::Spearman,
        Faction::Rome => UnitKind::Phalangite,
    }
}

/// Whether this faction's unit of this kind is an Aztec Eagle Warrior.
pub fn is_eagle_warrior_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Aztecs
        && kind == UnitKind::Spearman
        && !off.contains(FactionRule::AztecsEagleWarrior)
}

/// Whether this faction's unit of this kind is a Celtic Gaesatae.
pub fn is_gaesatae_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Celts
        && kind == UnitKind::Phalangite
        && !off.contains(FactionRule::CeltsGaesatae)
}

/// Whether this faction's unit of this kind is an Athenian Peltast.
pub fn is_peltast_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Athens
        && kind == UnitKind::CompositeArcher
        && !off.contains(FactionRule::AthensPeltast)
}

/// The Medjay replaces Egypt's Spearman; its sight is applied at the shared sight source.
pub fn is_medjay_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Egypt
        && kind == UnitKind::Spearman
        && !off.contains(FactionRule::EgyptMedjay)
}

/// Whether this faction's unit of this kind is an Indian War Elephant.
pub fn is_war_elephant_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::India
        && kind == UnitKind::Knight
        && !off.contains(FactionRule::IndiaWarElephant)
}

/// Whether this faction's unit of this kind is a Swiss Reisläufer.
pub fn is_reislaufer_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Switzerland
        && kind == UnitKind::Pikeman
        && !off.contains(FactionRule::SwissReislaufer)
}

/// Whether this faction's unit of this kind is a German Panzer.
pub fn is_panzer_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Germany
        && kind == UnitKind::Tank
        && !off.contains(FactionRule::GermanyPanzer)
}

/// Buildings a faction can never construct, with the reason shown in choices and validation.
/// The Sparta alone can build the War Shrine.
pub fn faction_excluded_building(faction: Faction, building: BuildingKind) -> Option<&'static str> {
    match (faction, building) {
        (f, BuildingKind::Agoge) if f != Faction::Sparta => Some("Only Sparta can build an Agoge"),
        (_, BuildingKind::RiceStorehouse) => Some("No people can build the Rice Storehouse yet"),
        (f, BuildingKind::YamRelayStation) if f != Faction::Mongols => {
            Some("Only Mongols can build the Yam Relay Station")
        }
        (f, BuildingKind::ImperialDiwan) if f != Faction::India => {
            Some("Only India can build the Imperial Diwan")
        }
        _ => None,
    }
}

/// The reason the Mongols cannot raise the Walls line yet.
pub const MONGOL_WALLS_REASON: &str = "The Mongols raise Walls from the Medieval era";

/// `faction_excluded_building`, with the Mongols' Walls line shut until the Medieval era.
pub fn faction_excluded_building_at(
    faction: Faction,
    building: BuildingKind,
    era: u8,
) -> Option<&'static str> {
    faction_excluded_building_at_rules(FactionRuleSet::EMPTY, faction, building, era)
}

/// `faction_excluded_building_at` with the faction rules in `off` switched off (ablation).
pub fn faction_excluded_building_at_rules(
    off: FactionRuleSet,
    faction: Faction,
    building: BuildingKind,
    era: u8,
) -> Option<&'static str> {
    if faction == Faction::Mongols
        && !off.contains(FactionRule::MongolsNoWalls)
        && era < MONGOL_WALLS_ERA
        && matches!(
            building,
            BuildingKind::Walls | BuildingKind::Castle | BuildingKind::StarFort
        )
    {
        return Some(MONGOL_WALLS_REASON);
    }
    faction_excluded_building(faction, building)
}

/// Whether this faction can build `item` and it does something for the faction under its own
/// rules. Faction replacements (Alpine Redoubt, Tagmatic Soldier...) are the same
/// item kinds, so they count; their names come from `building_name_for` / `unit_name_for`.
pub fn available_to_faction(faction: Faction, item: ProductionItem) -> bool {
    available_to_faction_rules(FactionRuleSet::EMPTY, faction, item)
}

/// `available_to_faction` with the faction rules in `off` switched off (ablation).
pub fn available_to_faction_rules(
    off: FactionRuleSet,
    faction: Faction,
    item: ProductionItem,
) -> bool {
    match item {
        ProductionItem::Project {
            project: crate::IndustryProject::Grow,
        } => faction == Faction::Egypt && !off.contains(FactionRule::EgyptNile),
        ProductionItem::Project { .. } => {
            faction == Faction::Germany && !off.contains(FactionRule::GermanyIndustryProjects)
        }
        ProductionItem::Unit {
            unit: UnitKind::HeirToTheThrone,
        } => faction == Faction::Byzantium,
        ProductionItem::Unit { unit } => {
            (faction != Faction::Rome
                || unit != UnitKind::Settler
                || off.contains(FactionRule::RomeNoSettlers))
                && (faction != Faction::Aztecs
                    || off.contains(FactionRule::AztecsNoHorses)
                    || !is_horse_mounted(unit))
        }
        ProductionItem::Improvement { .. } => true,
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            if faction_excluded_building(faction, building).is_some() {
                return false;
            }
            // The Nile: Egypt has no Production, so the Workshop line would do nothing.
            if faction == Faction::Egypt
                && !off.contains(FactionRule::EgyptNile)
                && matches!(item, ProductionItem::Building { .. })
                && is_workshop_line_building(building)
            {
                return false;
            }
            is_great_work(building)
                || building == BuildingKind::Agoge
                || building == BuildingKind::YamRelayStation
                || building == BuildingKind::ImperialDiwan
                || building == BuildingKind::AirDefence
                || building_stats_for_rules(off, faction, building).city_strength > 0
                || building_stats_for_rules(off, faction, building).yields != Yields::ZERO
        }
    }
}

pub fn item_cost(item: ProductionItem) -> i32 {
    match item {
        ProductionItem::Unit { unit } => unit_stats(unit).cost,
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            building_stats(building).cost
        }
        ProductionItem::Project { .. } => 0,
        ProductionItem::Improvement { improvement, .. } => {
            crate::improvements::base_cost(improvement)
        }
    }
}

pub fn item_requires(item: ProductionItem) -> Option<TechId> {
    match item {
        ProductionItem::Unit { unit } => unit_stats(unit).requires,
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            building_stats(building).requires
        }
        ProductionItem::Project { .. } | ProductionItem::Improvement { .. } => None,
    }
}

/// Rule-aware unlock shared by legal production and public research previews.
pub fn item_requires_for_rules(
    _rules: crate::experiments::RuleExperiments,
    item: ProductionItem,
) -> Option<TechId> {
    item_requires(item)
}

/// Food to grow from population `pop` to the next. The first growth (population 1 to 2) costs
/// `FIRST_GROWTH`: a Settler needs population 2 to be chosen (as in Civ 6), so a new city
/// reaches Settler size in about four turns, keeping the opening near the G8 pacing.
pub fn growth_threshold(pop: i32) -> i32 {
    if pop <= 1 {
        FIRST_GROWTH
    } else {
        GROWTH_BASE + GROWTH_PER_POP * (pop - 1)
    }
}

/// Display names. Kept here so the UI and events agree.
pub fn tech_name(id: TechId) -> &'static str {
    crate::eras::tech_def(id).name
}

pub fn unit_name(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::Scout => "Scout",
        UnitKind::Settler => "Settler",
        UnitKind::Spearman => "Spearman",
        UnitKind::Archer => "Archer",
        UnitKind::Pikeman => "Pikeman",
        UnitKind::Crossbowman => "Crossbowman",
        UnitKind::Musketman => "Musketman",
        UnitKind::HeirToTheThrone => "Heir to the Throne",
        UnitKind::Horseman => "Horseman",
        UnitKind::Knight => "Knight",
        UnitKind::Cavalry => "Cavalry",
        UnitKind::Tank => "Tank",
        UnitKind::AutonomousArmour => "Autonomous Armour",
        UnitKind::Rifleman => "Rifleman",
        UnitKind::Infantry => "Infantry",
        UnitKind::HunterKiller => "Hunter-Killer",
        UnitKind::FieldCannon => "Field Cannon",
        UnitKind::Artillery => "Artillery",
        UnitKind::CombatDrone => "Combat Drone",
        UnitKind::Fighter => "Fighter",
        UnitKind::Bomber => "Bomber",
        UnitKind::StealthFighter => "Stealth Fighter",
        UnitKind::StealthBomber => "Stealth Bomber",
        UnitKind::Phalangite => "Phalangite",
        UnitKind::CompositeArcher => "Composite Archer",
        UnitKind::Cataphract => "Cataphract",
        UnitKind::Arquebusier => "Arquebusier",
        UnitKind::Cuirassier => "Cuirassier",
        UnitKind::TrenchInfantry => "Trench Infantry",
        UnitKind::Howitzer => "Howitzer",
        UnitKind::EarlyTank => "Early Tank",
        UnitKind::Biplane => "Biplane",
        UnitKind::MechanisedInfantry => "Mechanised Infantry",
        UnitKind::MainBattleTank => "Main Battle Tank",
        UnitKind::RocketArtillery => "Rocket Artillery",
        UnitKind::JetFighter => "Jet Fighter",
        UnitKind::JetBomber => "Jet Bomber",
    }
}

/// Display names. Each identifier is the PascalCase of its display name; names used before the
/// 2026-11-22 rename (`Laboratory`, `RelayUplink`, ...) still load through the serde aliases in `types.rs`.
/// Castle and StarFort are Walls tiers 2 and 3.
pub fn building_name(kind: BuildingKind) -> &'static str {
    match kind {
        BuildingKind::RiceStorehouse => "Rice Storehouse",
        BuildingKind::Granary => "Granary",
        BuildingKind::Workshop => "Workshop",
        BuildingKind::Market => "Market",
        BuildingKind::Library => "Library",
        BuildingKind::Walls => "Walls",
        BuildingKind::GreatLibrary => "Great Library",
        BuildingKind::Pyramids => "Pyramids",
        BuildingKind::HagiaSophia => "Hagia Sophia",
        BuildingKind::EiffelTower => "Eiffel Tower",
        BuildingKind::IterFusionReactor => "ITER Fusion Reactor",
        BuildingKind::RoyalArsenal => "Royal Arsenal",
        BuildingKind::GrandBazaar => "Grand Bazaar",
        BuildingKind::Mill => "Mill",
        BuildingKind::Forge => "Forge",
        BuildingKind::Agoge => "Agoge",
        BuildingKind::University => "University",
        BuildingKind::ResearchLab => "Research Lab",
        BuildingKind::Bank => "Bank",
        BuildingKind::Factory => "Factory",
        BuildingKind::Castle => "Castle",
        BuildingKind::StarFort => "Star Fort",
        BuildingKind::YamRelayStation => "Yam Relay Station",
        BuildingKind::ImperialDiwan => "Imperial Diwan",
        BuildingKind::Roadworks => "Roadworks",
        BuildingKind::StockExchange => "Stock Exchange",
        BuildingKind::Supermarket => "Supermarket",
        BuildingKind::AirDefence => "Air Defence",
    }
}

/// Replacements retain the base item IDs in orders, buildings and saves. The owner's
/// faction selects the piece; modifiers are applied exactly once, here.
pub const GERMANY_GOLD_PROJECT_PERCENT: i32 = 50;
pub const GERMANY_SCIENCE_PROJECT_PERCENT: i32 = 25;

pub fn unit_moves_for(faction: Faction, kind: UnitKind) -> i32 {
    unit_moves_for_rules(FactionRuleSet::EMPTY, faction, kind)
}

/// `unit_moves_for` with the faction rules in `off` switched off (ablation).
pub fn unit_moves_for_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> i32 {
    unit_stats_for_rules(off, faction, kind).moves
}

pub fn unit_stats_for(faction: Faction, kind: UnitKind) -> UnitStats {
    unit_stats_for_rules(FactionRuleSet::EMPTY, faction, kind)
}

/// `unit_stats_for` with the faction rules in `off` switched off (ablation).
pub fn unit_stats_for_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> UnitStats {
    unit_stats_made_by_rules(off, faction, faction, kind)
}

/// `unit_stats_for_rules` for a unit whose owner is of `faction` but which `made_by` made: a
/// hired Company is its seller's own version of the unit (`company`). The unit's own stat line
/// follows `made_by`; faction-wide rules (the Mongol move) follow the owner `faction`.
pub fn unit_stats_made_by_rules(
    off: FactionRuleSet,
    faction: Faction,
    made_by: Faction,
    kind: UnitKind,
) -> UnitStats {
    // Faction edges and penalties are design numbers too, so they ride the era curve with the unit.
    let mut s = design_stats(kind);
    match (made_by, kind) {
        (Faction::Byzantium, k)
            if is_veteran_infantry(k) && !off.contains(FactionRule::ByzantiumTagmatic) =>
        {
            s.cost += 10;
            s.strength += 4;
        }
        (Faction::Mongols, KESHIG_KIND) if !off.contains(FactionRule::MongolsKeshig) => {
            // The Keshig, the Mongol horse archer (user, 2026-10-17): a Cataphract that can also
            // shoot. Each turn it charges (a melee order) or shoots (a RangedAttack), never both.
            s.cost = KESHIG_COST;
            s.ranged = KESHIG_RANGED;
            s.range = KESHIG_RANGE;
        }
        (f, k) if is_eagle_warrior_rules(off, f, k) => {
            s.cost += EAGLE_WARRIOR_COST;
            s.strength += EAGLE_WARRIOR_STRENGTH;
        }
        (f, k) if is_peltast_rules(off, f, k) => s.moves = PELTAST_MOVES,
        (f, k) if is_war_elephant_rules(off, f, k) => {
            s.strength += WAR_ELEPHANT_STRENGTH;
            s.moves = WAR_ELEPHANT_MOVES;
        }
        (f, k) if is_panzer_rules(off, f, k) => s.moves += PANZER_MOVES,
        _ => {}
    }
    apply_faction_wide_unit_stats(off, faction, &mut s);
    scale_to_era(s)
}

fn apply_faction_wide_unit_stats(off: FactionRuleSet, faction: Faction, stats: &mut UnitStats) {
    if faction == Faction::Mongols && stats.military && !off.contains(FactionRule::MongolsMoves) {
        stats.moves += 1;
    }
}

/// A generic piece under only its owner's faction-wide rules, for the UI comparison line.
pub fn standard_unit_stats_for(faction: Faction, kind: UnitKind) -> UnitStats {
    let mut stats = design_stats(kind);
    apply_faction_wide_unit_stats(FactionRuleSet::EMPTY, faction, &mut stats);
    scale_to_era(stats)
}

pub fn building_stats_for(faction: Faction, kind: BuildingKind) -> BuildingStats {
    building_stats_for_rules(FactionRuleSet::EMPTY, faction, kind)
}

/// `building_stats_for` with the faction rules in `off` switched off (ablation).
pub fn building_stats_for_rules(
    off: FactionRuleSet,
    faction: Faction,
    kind: BuildingKind,
) -> BuildingStats {
    let mut s = building_stats(kind);
    match (faction, kind) {
        (Faction::Switzerland, BuildingKind::Walls) if !off.contains(FactionRule::SwissRedoubt) => {
            s.cost += 20;
            s.city_strength = 15;
        }
        (Faction::Germany, BuildingKind::Forge) => {
            if !off.contains(FactionRule::GermanyRuhrProduction) {
                s.yields.production = 6;
            }
        }
        (Faction::Athens, BuildingKind::Library) if !off.contains(FactionRule::AthensLyceum) => {
            s.cost = 70;
            // Keep the science lead in the base family: a population-gated University
            // should not be the prerequisite for a scientist's useful research window.
            s.yields.science = 6;
        }
        (Faction::Egypt, BuildingKind::Granary)
            if !off.contains(FactionRule::EgyptRoyalGranary) =>
        {
            s.yields.food = 3;
        }
        (Faction::Athens, BuildingKind::Market)
            if !off.contains(FactionRule::AthensWeakMarkets) =>
        {
            s.yields.gold = 1;
        }
        (Faction::India, BuildingKind::RiceStorehouse) => {
            // Retired Japan building in an old save: its Food bonus does not follow India.
            s.yields = Yields::ZERO;
        }
        _ => {}
    }
    // The Hall's price is its own goal cost.
    s
}

/// A Production building is identified by its own stat line, including world wonders.
pub fn is_production_building(kind: BuildingKind) -> bool {
    building_stats(kind).yields.production > 0
}

/// A generic building under only its owner's faction-wide rules, for the UI comparison line.
pub fn standard_building_stats_for(_faction: Faction, kind: BuildingKind) -> BuildingStats {
    building_stats(kind)
}

pub fn terrain_yields(_faction: Faction, terrain: Terrain) -> Yields {
    terrain.yields()
}

pub fn ignores_wounds(faction: Faction, kind: UnitKind) -> bool {
    ignores_wounds_rules(FactionRuleSet::EMPTY, faction, kind)
}

/// `ignores_wounds` with the faction rules in `off` switched off (ablation).
pub fn ignores_wounds_rules(off: FactionRuleSet, faction: Faction, kind: UnitKind) -> bool {
    faction == Faction::Sparta
        && !off.contains(FactionRule::SpartaNoWounds)
        && unit_class(kind) == Some(UnitClass::Melee)
}

pub fn terrain_defense(faction: Faction, terrain: Terrain) -> i32 {
    terrain_defense_rules(FactionRuleSet::EMPTY, faction, terrain)
}

/// `terrain_defense` with the faction rules in `off` switched off (ablation).
pub fn terrain_defense_rules(off: FactionRuleSet, faction: Faction, terrain: Terrain) -> i32 {
    if faction == Faction::Mongols && !off.contains(FactionRule::MongolsNoTerrain) {
        0
    } else {
        terrain.defense_bonus()
    }
}

/// Thin levies (Byzantium, 2026-10-10): at most one armed unit per city, plus this
/// many. Scouts and civilians never count (`pays_upkeep`).
pub const LEVY_EXTRA: usize = 1;

/// The most armed units a faction may field with `cities` cities, if it is capped at all.
pub fn levy_cap(faction: Faction, cities: usize) -> Option<usize> {
    levy_cap_rules(FactionRuleSet::EMPTY, faction, cities)
}

/// `levy_cap` with the faction rules in `off` switched off (ablation).
pub fn levy_cap_rules(off: FactionRuleSet, faction: Faction, cities: usize) -> Option<usize> {
    (faction == Faction::Byzantium && !off.contains(FactionRule::ByzantiumLevy))
        .then_some(cities + LEVY_EXTRA)
}

/// Whether this kind counts toward the empire's unit upkeep and its free allowance: armed
/// military units do; Scouts and civilians (Settlers) never do, in any faction.
pub fn pays_upkeep(kind: UnitKind) -> bool {
    kind != UnitKind::Scout && !is_aircraft(kind) && unit_stats(kind).military
}

pub fn military_upkeep(faction: Faction) -> i32 {
    military_upkeep_rules(FactionRuleSet::EMPTY, faction)
}

/// `military_upkeep` with the faction rules in `off` switched off (ablation).
pub fn military_upkeep_rules(off: FactionRuleSet, faction: Faction) -> i32 {
    if faction == Faction::Switzerland && !off.contains(FactionRule::SwissUnitUpkeep) {
        3
    } else {
        UPKEEP_PER_UNIT
    }
}

/// Total upkeep before technology discounts, after the empire's free unit allowance.
pub fn paid_military_upkeep(faction: Faction, paid: i32) -> i32 {
    paid_military_upkeep_rules(FactionRuleSet::EMPTY, faction, paid)
}

/// `paid_military_upkeep` with the faction rules in `off` switched off (ablation).
pub fn paid_military_upkeep_rules(off: FactionRuleSet, faction: Faction, paid: i32) -> i32 {
    paid * military_upkeep_rules(off, faction)
}

pub fn unit_name_for(faction: Faction, kind: UnitKind) -> &'static str {
    match (faction, kind) {
        (Faction::Byzantium, UnitKind::Pikeman) => "Tagmatic Soldier",
        (Faction::Mongols, KESHIG_KIND) => "Keshig",
        (Faction::Sparta, UnitKind::Spearman) => "Spartan Hoplite",
        (Faction::Aztecs, UnitKind::Spearman) => "Eagle Warrior",
        (Faction::Celts, UnitKind::Phalangite) => "Gaesatae",
        (Faction::Athens, UnitKind::CompositeArcher) => "Peltast",
        (Faction::Rome, UnitKind::Phalangite) => "Legionary",
        (Faction::India, UnitKind::Knight) => "War Elephant",
        (Faction::Switzerland, UnitKind::Pikeman) => "Reisläufer",
        (Faction::Germany, UnitKind::Tank) => "Panzer",
        (Faction::Egypt, UnitKind::Spearman) => "Medjay",
        _ => unit_name(kind),
    }
}

/// Player-facing names; enum variants and serialized IDs remain stable.
pub fn faction_name(faction: Faction) -> &'static str {
    match faction {
        Faction::Switzerland => "Switzerland",
        Faction::Mongols => "Mongols",
        Faction::Sparta => "Sparta",
        Faction::Germany => "Germany",
        Faction::India => "India",
        Faction::Byzantium => "Byzantium",
        Faction::Celts => "Celts",
        Faction::Athens => "Athens",
        Faction::Aztecs => "Aztecs",
        Faction::Egypt => "Egypt",
        Faction::Rome => "Rome",
    }
}

/// Adjective used when a report names a unit's people.
pub fn faction_adjective(faction: Faction) -> &'static str {
    match faction {
        Faction::Switzerland => "Swiss",
        Faction::Mongols => "Mongol",
        Faction::Sparta => "Spartan",
        Faction::Germany => "German",
        Faction::India => "Indian",
        Faction::Byzantium => "Byzantine",
        Faction::Celts => "Celtic",
        Faction::Egypt => "Egyptian",
        Faction::Athens => "Athenian",
        Faction::Aztecs => "Aztec",
        Faction::Rome => "Roman",
    }
}

/// The reason `available_to_faction` rejects `item` for `faction`, for validation errors and
/// greyed-out text; such items are left out of production choices.
pub const AZTEC_NO_HORSES: &str =
    "The Aztecs cannot train horse-mounted units; Early Tanks, Tanks and later armour are allowed";

pub fn is_horse_mounted(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::Horseman
            | UnitKind::Cataphract
            | UnitKind::Knight
            | UnitKind::Cuirassier
            | UnitKind::Cavalry
    )
}

pub fn faction_unavailable_reason(item: ProductionItem, faction: Faction) -> Option<String> {
    if faction == Faction::Rome
        && item
            == (ProductionItem::Unit {
                unit: UnitKind::Settler,
            })
    {
        return Some(
            "Rome cannot train or buy Settlers; full-health melee veterans can found cities".into(),
        );
    }
    if faction == Faction::Aztecs
        && matches!(item, ProductionItem::Unit { unit } if is_horse_mounted(unit))
    {
        return Some(AZTEC_NO_HORSES.into());
    }
    if available_to_faction(faction, item) {
        return None;
    }
    Some(match item {
        ProductionItem::Project {
            project: crate::IndustryProject::Grow,
        } => "Only Egyptian cities can choose Grow".into(),
        ProductionItem::Building { building }
            if faction == Faction::Egypt && is_workshop_line_building(building) =>
        {
            "Workshops, Forges and Factories do nothing for Egypt, which has no Production".into()
        }
        ProductionItem::Project { .. } => "Only German cities can run industry projects".into(),
        ProductionItem::Building {
            building: BuildingKind::RiceStorehouse,
        } => "No people can build the Rice Storehouse yet".into(),
        ProductionItem::Building {
            building: BuildingKind::ImperialDiwan,
        } => "Only India can build the Imperial Diwan".into(),
        ProductionItem::Building {
            building: BuildingKind::YamRelayStation,
        }
        | ProductionItem::Wonder {
            building: BuildingKind::YamRelayStation,
            ..
        } => "Only Mongols can build the Yam Relay Station".into(),
        ProductionItem::Unit {
            unit: UnitKind::HeirToTheThrone,
        } => "Only Byzantium can train a Heir to the Throne".into(),
        _ => format!("Does nothing for {}", faction_name(faction)),
    })
}

/// State-aware version of `faction_unavailable_reason` for changing faction rules.
pub fn player_unavailable_reason(
    item: ProductionItem,
    faction: Faction,
    era: u8,
) -> Option<String> {
    player_unavailable_reason_rules(FactionRuleSet::EMPTY, item, faction, era)
}

/// `player_unavailable_reason` with the faction rules in `off` switched off (ablation).
pub fn player_unavailable_reason_rules(
    off: FactionRuleSet,
    item: ProductionItem,
    faction: Faction,
    era: u8,
) -> Option<String> {
    if available_to_player_rules(off, faction, era, item) {
        return None;
    }
    if matches!(item, ProductionItem::Project { project } if project != crate::IndustryProject::Grow)
        && faction == Faction::Germany
        && off.contains(FactionRule::GermanyIndustryProjects)
    {
        return Some("Industry projects are switched off".into());
    }
    if faction == Faction::Mongols && is_mongol_gated(item) {
        return Some(MONGOL_WALLS_REASON.into());
    }
    faction_unavailable_reason(item, faction)
}

pub fn building_name_for(faction: Faction, kind: BuildingKind) -> &'static str {
    match (faction, kind) {
        (Faction::Switzerland, BuildingKind::Walls) => "Alpine Redoubt",
        (Faction::Germany, BuildingKind::Forge) => "Ruhr Foundry",
        (Faction::Athens, BuildingKind::Library) => "Lyceum",
        (Faction::India, BuildingKind::Market) => "Revenue Office",
        (Faction::Egypt, BuildingKind::Granary) => "Royal Granary",
        (Faction::Rome, BuildingKind::Walls) => "Castrum",
        _ => building_name(kind),
    }
}

pub fn building_predecessor(kind: BuildingKind) -> Option<BuildingKind> {
    match kind {
        BuildingKind::Mill => Some(BuildingKind::Granary),
        BuildingKind::Forge => Some(BuildingKind::Workshop),
        BuildingKind::University => Some(BuildingKind::Library),
        BuildingKind::ResearchLab => Some(BuildingKind::University),
        BuildingKind::Bank => Some(BuildingKind::Market),
        BuildingKind::StockExchange => Some(BuildingKind::Bank),
        BuildingKind::Supermarket => Some(BuildingKind::Mill),
        BuildingKind::Factory => Some(BuildingKind::Forge),
        BuildingKind::Castle => Some(BuildingKind::Walls),
        BuildingKind::StarFort => Some(BuildingKind::Castle),
        _ => None,
    }
}

pub fn building_min_population(kind: BuildingKind) -> i32 {
    match kind {
        BuildingKind::Forge => 4,
        BuildingKind::University => 5,
        BuildingKind::ResearchLab => 8,
        BuildingKind::StockExchange => 8,
        BuildingKind::Bank => 5,
        BuildingKind::Factory => 6,
        _ => 0,
    }
}

/// The Walls tier above plain Walls that these buildings give: 0 none, 1 a Castle, 2 a Star Fort.
pub fn fort_tier<'a>(buildings: impl IntoIterator<Item = &'a BuildingKind> + Clone) -> u8 {
    u8::from(has_building(buildings.clone(), BuildingKind::Castle))
        + u8::from(has_building(buildings, BuildingKind::StarFort))
}

pub fn building_family(mut kind: BuildingKind) -> BuildingKind {
    while let Some(previous) = building_predecessor(kind) {
        kind = previous;
    }
    kind
}

/// A completed higher tier also satisfies requirements for its predecessors.
pub fn building_satisfies(mut built: BuildingKind, required: BuildingKind) -> bool {
    loop {
        if built == required {
            return true;
        }
        match building_predecessor(built) {
            Some(previous) => built = previous,
            None => return false,
        }
    }
}

/// Whether a building is part of the Workshop Production line.
pub fn is_workshop_line_building(building: BuildingKind) -> bool {
    building_satisfies(building, BuildingKind::Workshop)
}

pub fn has_building<'a>(
    buildings: impl IntoIterator<Item = &'a BuildingKind>,
    required: BuildingKind,
) -> bool {
    buildings
        .into_iter()
        .any(|built| building_satisfies(*built, required))
}

/// Additional yields/cost belong to the step; the occupied family retains earlier yields.
pub fn building_family_yields(faction: Faction, kind: BuildingKind) -> Yields {
    building_stats_for(faction, kind).yields
        + building_predecessor(kind).map_or(Yields::ZERO, |p| building_family_yields(faction, p))
}

/// Era names in order (`crate::eras`). The era index in victory points (the era
/// objectives) is **1-based** into this list: 1 Ancient .. 8 Future.
pub const ERA_NAMES: [&str; crate::eras::ERA_COUNT] = crate::eras::ERA_NAMES;

/// Number of eras in the tech tree.
pub fn era_count() -> u8 {
    crate::eras::ERA_COUNT as u8
}

/// A tech's era as a 1-based index into `ERA_NAMES`.
pub fn era_index(tech: TechId) -> u8 {
    crate::eras::era_of_tech(tech) + 1
}

/// The name of era `era` (1-based).
pub fn era_name(era: u8) -> &'static str {
    ERA_NAMES
        .get(usize::from(era).wrapping_sub(1))
        .copied()
        .unwrap_or("Unknown era")
}

/// The name of the era a technology belongs to.
pub fn tech_era(tech: TechId) -> &'static str {
    crate::eras::era_name(crate::eras::era_of_tech(tech))
}

/// A Walls tier above plain Walls counts one more: 0 no Walls, 1 Walls, 2 a Castle, 3 a Star Fort.
pub fn wall_tier<'a>(buildings: impl IntoIterator<Item = &'a BuildingKind> + Clone) -> u8 {
    u8::from(has_building(buildings.clone(), BuildingKind::Walls)) + fort_tier(buildings)
}

/// The siege tier of an empire's Siege units: the highest siege technology it knows. Mathematics
/// ignores Walls (the old Catapult), Machinery Castles (the Trebuchet) and Metallurgy Star Forts
/// (the Bombard). 0: none of them, so Siege cannot be bought and covers nothing.
pub fn siege_tier(techs: &std::collections::BTreeSet<TechId>) -> u8 {
    if techs.contains(&TechId::Metallurgy) {
        3
    } else if techs.contains(&TechId::Machinery) {
        2
    } else {
        u8::from(techs.contains(&TechId::Mathematics))
    }
}

pub fn siege_city_bonus(kind: UnitKind) -> i32 {
    let design = match kind {
        UnitKind::Bomber | UnitKind::JetBomber | UnitKind::StealthBomber => 20,
        _ => 0,
    };
    era_scaled(design, unit_era_tier(kind))
}

/// The seven world wonders, at most one unlocked per era (the World War and Modern eras
/// have none); each empire may complete at most one.
pub const GREAT_WORKS: [BuildingKind; 7] = [
    BuildingKind::Pyramids,
    BuildingKind::GreatLibrary,
    BuildingKind::HagiaSophia,
    BuildingKind::GrandBazaar,
    BuildingKind::RoyalArsenal,
    BuildingKind::EiffelTower,
    BuildingKind::IterFusionReactor,
];

/// Shared construction slots. These limits are per empire, never per team or world.
pub struct EmpireBuildingLimit {
    pub buildings: &'static [BuildingKind],
    pub max: usize,
    /// World wonders have a permanent player completion flag, checked separately from construction slots.
    pub includes_built: bool,
}

pub fn building_empire_limit(building: BuildingKind) -> Option<EmpireBuildingLimit> {
    if is_great_work(building) {
        Some(EmpireBuildingLimit {
            buildings: &GREAT_WORKS,
            max: 1,
            includes_built: false,
        })
    } else {
        None
    }
}

pub fn is_great_work(building: BuildingKind) -> bool {
    GREAT_WORKS.contains(&building)
}
pub fn is_great_work_item(item: ProductionItem) -> bool {
    matches!(item, ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } if is_great_work(building))
}
pub fn great_work_requirement(building: BuildingKind) -> Option<(BuildingKind, usize)> {
    match building {
        BuildingKind::RoyalArsenal => Some((BuildingKind::Factory, 1)),
        BuildingKind::GrandBazaar => Some((BuildingKind::Bank, 2)),
        BuildingKind::GreatLibrary => Some((BuildingKind::Library, 1)),
        _ => None,
    }
}
/// Infantry may defend Sparta homes; the expedition still requires a veteran.
pub fn is_infantry(kind: UnitKind) -> bool {
    unit_class(kind) == Some(UnitClass::Melee)
}

pub fn sparta_home_guard(kind: UnitKind) -> bool {
    is_infantry(kind)
}

/// Faction goals count the full veteran infantry line.
pub fn is_veteran_infantry(kind: UnitKind) -> bool {
    is_infantry(kind) && kind != UnitKind::Spearman
}
