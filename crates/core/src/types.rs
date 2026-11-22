use crate::hex::Hex;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TeamId(pub u8);

impl PlayerId {
    /// The barbarians: a reserved owner outside `GameState::players`, never an empire.
    /// See `rules::barbarians` for why they are a unit owner rather than a player.
    pub const BARBARIAN: PlayerId = PlayerId(u8::MAX);

    pub fn is_barbarian(self) -> bool {
        self == Self::BARBARIAN
    }
}

impl TeamId {
    /// The barbarians' team: at war with every other team, with no diplomatic relation.
    pub const BARBARIAN: TeamId = TeamId(u8::MAX);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CityId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Ocean,
    Coast,
    Grassland,
    Plains,
    Forest,
    Hills,
    Desert,
    Tundra,
    Mountains,
}

impl Terrain {
    pub const ALL: [Terrain; 9] = [
        Terrain::Ocean,
        Terrain::Coast,
        Terrain::Grassland,
        Terrain::Plains,
        Terrain::Forest,
        Terrain::Hills,
        Terrain::Desert,
        Terrain::Tundra,
        Terrain::Mountains,
    ];

    pub fn yields(self) -> Yields {
        match self {
            Terrain::Ocean => Yields::new(1, 0, 0, 0),
            Terrain::Coast => Yields::new(1, 0, 1, 0),
            Terrain::Grassland => Yields::new(2, 0, 0, 0),
            Terrain::Plains => Yields::new(1, 1, 0, 0),
            Terrain::Forest => Yields::new(1, 2, 0, 0),
            Terrain::Hills => Yields::new(0, 2, 0, 0),
            Terrain::Desert => Yields::new(0, 0, 1, 0),
            Terrain::Tundra => Yields::new(1, 0, 0, 0),
            Terrain::Mountains => Yields::ZERO,
        }
    }

    /// Movement points to enter this terrain for a land unit; `None` = impassable.
    pub fn move_cost(self) -> Option<i32> {
        match self {
            Terrain::Ocean | Terrain::Coast | Terrain::Mountains => None,
            Terrain::Forest | Terrain::Hills => Some(2),
            _ => Some(1),
        }
    }

    pub fn is_land(self) -> bool {
        !matches!(self, Terrain::Ocean | Terrain::Coast)
    }

    /// Flat defense bonus for units standing here.
    pub fn defense_bonus(self) -> i32 {
        match self {
            Terrain::Hills => 5,
            Terrain::Forest => 3,
            _ => 0,
        }
    }
}

/// The four yields. Plain integers; percentages elsewhere are integers out of 100.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Yields {
    pub food: i32,
    pub production: i32,
    pub gold: i32,
    pub science: i32,
}

impl Yields {
    pub const ZERO: Yields = Yields::new(0, 0, 0, 0);

    pub const fn new(food: i32, production: i32, gold: i32, science: i32) -> Self {
        Self {
            food,
            production,
            gold,
            science,
        }
    }

    /// Each yield scaled by `pct`/100, truncated toward zero.
    pub fn percent(self, pct: i32) -> Yields {
        let scaled = |value: i32| {
            (i64::from(value) * i64::from(pct) / 100)
                .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
        };
        Yields::new(
            scaled(self.food),
            scaled(self.production),
            scaled(self.gold),
            scaled(self.science),
        )
    }
}

impl Add for Yields {
    type Output = Yields;
    fn add(self, o: Yields) -> Yields {
        Yields::new(
            self.food.saturating_add(o.food),
            self.production.saturating_add(o.production),
            self.gold.saturating_add(o.gold),
            self.science.saturating_add(o.science),
        )
    }
}

impl AddAssign for Yields {
    fn add_assign(&mut self, o: Yields) {
        *self = *self + o;
    }
}

impl Sub for Yields {
    type Output = Yields;
    fn sub(self, o: Yields) -> Yields {
        self + o.percent(-100)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Faction {
    Switzerland,
    #[serde(alias = "Japan", alias = "japan")]
    India,
    Byzantium,
    Germany,
    Celts,
    Mongols,
    Sparta,
    Athens,
    Aztecs,
    Egypt,
    Rome,
}

impl Faction {
    pub const ALL: [Faction; 11] = [
        Self::Switzerland,
        Self::India,
        Self::Byzantium,
        Self::Germany,
        Self::Celts,
        Self::Mongols,
        Self::Sparta,
        Self::Athens,
        Self::Aztecs,
        Self::Egypt,
        Self::Rome,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UnitKind {
    Scout,
    Settler,
    #[serde(alias = "Soldier")]
    Spearman,
    #[serde(alias = "Ranger")]
    Archer,
    #[serde(alias = "Trooper")]
    Pikeman,
    #[serde(alias = "Marksman")]
    Crossbowman,
    #[serde(alias = "AssaultTrooper")]
    Musketman,
    /// The Byzantium's Heir: an expensive, unbuyable civilian whose holder scores Byzantium's
    /// goal (`goals`). Only the Byzantium can train him.
    #[serde(alias = "Heir")]
    HeirToTheThrone,
    /// Mounted line (era overhaul phase A2): Horseman, Cataphract, Knight, Cuirassier, Cavalry, Tank,
    /// Autonomous Armour. 4 moves.
    Horseman,
    Knight,
    Cavalry,
    Tank,
    /// Future mounted unit (Autonomous Systems).
    AutonomousArmour,
    /// Melee line, Industrial to Future: Rifleman, Infantry, Hunter-Killer.
    Rifleman,
    Infantry,
    #[serde(alias = "HumanoidRobot")]
    HunterKiller,
    /// Ranged line, Industrial to Future: Field Cannon, Artillery, Combat Drone.
    FieldCannon,
    Artillery,
    #[serde(alias = "Drone")]
    CombatDrone,
    /// The Modern air layer (`UnitClass::Air`, `rules::air`): based in a city's hangar, never
    /// standing on a map tile. The Fighter guards its city's surroundings (it intercepts air
    /// strikes unless it strikes itself, a short-range hit on units); the Bomber strikes far.
    Fighter,
    Bomber,
    StealthFighter,
    StealthBomber,
    /// One unit per class in every era (user, 2026-10-10): the Classical and Renaissance
    /// forms the first lineage lacked. Phalangite (melee), Composite Archer (ranged) and
    /// Cataphract (mounted) are Classical; Arquebusier (ranged) and Cuirassier (mounted)
    /// are Renaissance.
    Phalangite,
    CompositeArcher,
    Cataphract,
    Arquebusier,
    Cuirassier,
    /// The Machine Age (user, 2026-11-14): one unit per class between the Industrial and Modern
    /// eras, and the first aircraft. Trench Infantry (melee), Howitzer (ranged, indirect fire),
    /// EarlyTank (slow early armour) and Biplane (air: short strikes, a small guard, sight).
    TrenchInfantry,
    Howitzer,
    #[serde(alias = "Landship")]
    EarlyTank,
    Biplane,
    /// The Modern era (1950 to 2010, user, 2026-11-14): one upgrade of the World War unit of
    /// each class, and the jets. Mechanised Infantry (3 moves), Main Battle Tank, Rocket
    /// Artillery (range 3), Jet Fighter and Jet Bomber.
    MechanisedInfantry,
    MainBattleTank,
    RocketArtillery,
    JetFighter,
    JetBomber,
}

impl UnitKind {
    pub const ALL: [UnitKind; 37] = [
        UnitKind::Scout,
        UnitKind::Settler,
        UnitKind::Spearman,
        UnitKind::Archer,
        UnitKind::Pikeman,
        UnitKind::Crossbowman,
        UnitKind::Musketman,
        UnitKind::HeirToTheThrone,
        UnitKind::Horseman,
        UnitKind::Knight,
        UnitKind::Cavalry,
        UnitKind::Tank,
        UnitKind::AutonomousArmour,
        UnitKind::Rifleman,
        UnitKind::Infantry,
        UnitKind::HunterKiller,
        UnitKind::FieldCannon,
        UnitKind::Artillery,
        UnitKind::CombatDrone,
        UnitKind::Fighter,
        UnitKind::Bomber,
        UnitKind::StealthFighter,
        UnitKind::StealthBomber,
        UnitKind::Phalangite,
        UnitKind::CompositeArcher,
        UnitKind::Cataphract,
        UnitKind::Arquebusier,
        UnitKind::Cuirassier,
        UnitKind::TrenchInfantry,
        UnitKind::Howitzer,
        UnitKind::EarlyTank,
        UnitKind::Biplane,
        UnitKind::MechanisedInfantry,
        UnitKind::MainBattleTank,
        UnitKind::RocketArtillery,
        UnitKind::JetFighter,
        UnitKind::JetBomber,
    ];
}

/// A military unit's class (the phase A2 unit lineage hooks into this): the melee line beats
/// mounted, ranged beats melee, mounted beats ranged; air is the Modern-era layer.
/// No combat rule reads it yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UnitClass {
    Melee,
    Ranged,
    Mounted,
    Air,
}

/// Where an empire directs Science after its last technology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScienceConversion {
    #[default]
    Gold,
    Food,
    Production,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    /// Retired Japan Rice Storehouse, unavailable to India. `Monument` remains in
    /// tracked fixture tests.
    #[serde(alias = "Monument", alias = "BroodHall")]
    RiceStorehouse,
    Granary,
    Workshop,
    Market,
    #[serde(alias = "Laboratory")]
    Library,
    Walls,
    #[serde(alias = "RelayUplink")]
    GreatLibrary,
    Pyramids,
    HagiaSophia,
    EiffelTower,
    #[serde(alias = "IterReactor")]
    IterFusionReactor,
    /// Mill (Crop Rotation). `Arcology` remains in tracked fixture tests.
    #[serde(alias = "Arcology", alias = "Hydroponics")]
    Mill,
    #[serde(alias = "MachineShop")]
    Forge,
    #[serde(alias = "WarShrine")]
    Agoge,
    #[serde(alias = "College")]
    University,
    #[serde(alias = "Clearinghouse")]
    Bank,
    #[serde(alias = "Autoworks")]
    Factory,
    /// Walls tier 2 (Feudalism). `Citadel` remains in tracked fixture tests.
    #[serde(alias = "Citadel")]
    Castle,
    /// Walls tier 3 (Military Engineering).
    StarFort,
    #[serde(alias = "PlanetaryFoundry")]
    RoyalArsenal,
    #[serde(alias = "WorldExchange")]
    GrandBazaar,
    #[serde(alias = "InferenceCore")]
    ResearchLab,
    /// Mongols's Courier Hall: a pricey landmark, one per Mongols empire. While it stands,
    /// its holder (whoever owns its city) scores the Mongols goal and every empire crosses
    /// the holder's land at Peace (`goals`, `GameState::passage_allowed`). Only Mongols can
    /// build it.
    #[serde(alias = "CourierHall")]
    YamRelayStation,
    /// Akbar's Imperial Diwan; its founding city carries the held goal.
    ImperialDiwan,
    /// Roadworks: any empire's one-time road building. It links its city to every city within
    /// `data::ROADWORKS_RANGE` hexes (`roads::extend_roadworks`) and yields 1 Gold.
    Roadworks,
    /// Stock Exchange (Electricity, Machine Age): the Gold line's third building after the
    /// Market and the Bank.
    StockExchange,
    /// Supermarket (Green Revolution, Modern): the Food line's third building after the Granary
    /// and the Mill.
    Supermarket,
    /// Air Defence (Guided Missiles, Modern): intercepts air strikes on its own city like a
    /// Fighter on guard there, without using a hangar slot.
    AirDefence,
}

impl BuildingKind {
    pub const ALL: [BuildingKind; 28] = [
        BuildingKind::RiceStorehouse,
        BuildingKind::Granary,
        BuildingKind::Workshop,
        BuildingKind::Market,
        BuildingKind::Library,
        BuildingKind::Walls,
        BuildingKind::GreatLibrary,
        BuildingKind::Pyramids,
        BuildingKind::HagiaSophia,
        BuildingKind::EiffelTower,
        BuildingKind::IterFusionReactor,
        BuildingKind::Mill,
        BuildingKind::Forge,
        BuildingKind::Agoge,
        BuildingKind::University,
        BuildingKind::Bank,
        BuildingKind::Factory,
        BuildingKind::Castle,
        BuildingKind::StarFort,
        BuildingKind::RoyalArsenal,
        BuildingKind::GrandBazaar,
        BuildingKind::ResearchLab,
        BuildingKind::YamRelayStation,
        BuildingKind::ImperialDiwan,
        BuildingKind::Roadworks,
        BuildingKind::StockExchange,
        BuildingKind::Supermarket,
        BuildingKind::AirDefence,
    ];
}

/// A technology. Declaration order is research order: five per era, nine eras
/// (`crate::eras`). Add a Future-era technology by following the note in `crate::eras`.
///
/// Saves written before the era overhaul name the old technologies; they load as the nearest new
/// one (`TechId::from_name`), so a game in progress continues.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum TechId {
    Irrigation,
    BronzeWorking,
    Masonry,
    Currency,
    Writing,
    Mathematics,
    Engineering,
    IronWorking,
    CodeOfLaws,
    Philosophy,
    Feudalism,
    Machinery,
    Chivalry,
    Education,
    CropRotation,
    Gunpowder,
    Metallurgy,
    MilitaryEngineering,
    Banking,
    PrintingPress,
    Rifling,
    MilitaryScience,
    /// Displayed as "Ballistics"; the old Pikeman technology of that name is `Feudalism` now, so
    /// the identifier differs to keep old saves unambiguous.
    Cannonry,
    Industrialization,
    Railways,
    ReplaceableParts,
    Combustion,
    Steel,
    Flight,
    Electricity,
    MassMobilisation,
    Rocketry,
    ArmouredWarfare,
    AirPower,
    Electronics,
    MechanisedWarfare,
    CompositeArmour,
    GuidedMissiles,
    JetEngines,
    GreenRevolution,
    AutonomousSystems,
    Robotics,
    FusionPower,
    StealthTechnology,
    RecursiveAi,
}

impl TechId {
    pub const ALL: [TechId; 45] = [
        TechId::Irrigation,
        TechId::BronzeWorking,
        TechId::Masonry,
        TechId::Currency,
        TechId::Writing,
        TechId::Mathematics,
        TechId::Engineering,
        TechId::IronWorking,
        TechId::CodeOfLaws,
        TechId::Philosophy,
        TechId::Feudalism,
        TechId::Machinery,
        TechId::Chivalry,
        TechId::Education,
        TechId::CropRotation,
        TechId::Gunpowder,
        TechId::Metallurgy,
        TechId::MilitaryEngineering,
        TechId::Banking,
        TechId::PrintingPress,
        TechId::Rifling,
        TechId::MilitaryScience,
        TechId::Cannonry,
        TechId::Industrialization,
        TechId::Railways,
        TechId::ReplaceableParts,
        TechId::Combustion,
        TechId::Steel,
        TechId::Flight,
        TechId::Electricity,
        TechId::MassMobilisation,
        TechId::Rocketry,
        TechId::ArmouredWarfare,
        TechId::AirPower,
        TechId::Electronics,
        TechId::MechanisedWarfare,
        TechId::CompositeArmour,
        TechId::GuidedMissiles,
        TechId::JetEngines,
        TechId::GreenRevolution,
        TechId::AutonomousSystems,
        TechId::Robotics,
        TechId::FusionPower,
        TechId::StealthTechnology,
        TechId::RecursiveAi,
    ];

    /// The technology with this serialized name, including the names saves used before the
    /// era overhaul.
    pub fn from_name(name: &str) -> Option<TechId> {
        Some(match name {
            "Irrigation" => TechId::Irrigation,
            "BronzeWorking" => TechId::BronzeWorking,
            "Masonry" => TechId::Masonry,
            "Currency" => TechId::Currency,
            "Writing" => TechId::Writing,
            "Mathematics" => TechId::Mathematics,
            "Engineering" => TechId::Engineering,
            "IronWorking" => TechId::IronWorking,
            "CodeOfLaws" => TechId::CodeOfLaws,
            "Philosophy" => TechId::Philosophy,
            "Feudalism" => TechId::Feudalism,
            "Machinery" => TechId::Machinery,
            "Chivalry" => TechId::Chivalry,
            "Education" => TechId::Education,
            "CropRotation" => TechId::CropRotation,
            "Gunpowder" => TechId::Gunpowder,
            "Metallurgy" => TechId::Metallurgy,
            "MilitaryEngineering" => TechId::MilitaryEngineering,
            "Banking" => TechId::Banking,
            "PrintingPress" => TechId::PrintingPress,
            "Rifling" => TechId::Rifling,
            "MilitaryScience" => TechId::MilitaryScience,
            "Cannonry" => TechId::Cannonry,
            "Ballistics" => TechId::Feudalism,
            "Industrialization" => TechId::Industrialization,
            "Railways" => TechId::Railways,
            "ReplaceableParts" => TechId::ReplaceableParts,
            "Combustion" => TechId::Combustion,
            "Steel" => TechId::Steel,
            "Flight" => TechId::Flight,
            "Electricity" => TechId::Electricity,
            "MassMobilisation" => TechId::MassMobilisation,
            "Rocketry" => TechId::Rocketry,
            "ArmouredWarfare" => TechId::ArmouredWarfare,
            "AirPower" => TechId::AirPower,
            "Electronics" => TechId::Electronics,
            "MechanisedWarfare" => TechId::MechanisedWarfare,
            "CompositeArmour" => TechId::CompositeArmour,
            "GuidedMissiles" => TechId::GuidedMissiles,
            "JetEngines" => TechId::JetEngines,
            "GreenRevolution" => TechId::GreenRevolution,
            "AutonomousSystems" => TechId::AutonomousSystems,
            "Robotics" => TechId::Robotics,
            "FusionPower" => TechId::FusionPower,
            "QuantumComputing" | "StealthTechnology" => TechId::StealthTechnology,
            "RecursiveAi" => TechId::RecursiveAi,
            "SoilReclamation" => TechId::Irrigation,
            "Fabrication" => TechId::BronzeWorking,
            "Fortification" => TechId::Masonry,
            "Exchange" => TechId::Currency,
            "SignalTheory" => TechId::Writing,
            "DeepSurvey" => TechId::Philosophy,
            "CivicNetworks" => TechId::CodeOfLaws,
            "RelayRestoration" => TechId::Philosophy,
            "SalvageProtocols" => TechId::Irrigation,
            "IntensiveCultivation" => TechId::CropRotation,
            "MachineTools" => TechId::IronWorking,
            "PublicAdministration" => TechId::CodeOfLaws,
            "DiplomaticProtocols" => TechId::CodeOfLaws,
            "FieldLogistics" => TechId::Engineering,
            "Academies" => TechId::Education,
            "CommercialLaw" => TechId::Banking,
            "ClosedHabitats" => TechId::CropRotation,
            "IndustrialAutomation" => TechId::Industrialization,
            "FederatedMarkets" => TechId::Banking,
            "ResearchNetworks" => TechId::PrintingPress,
            "MutualDefense" => TechId::Feudalism,
            "CombinedArms" => TechId::Machinery,
            "SiegeEngineering" => TechId::Mathematics,
            "RecoveredPower" => TechId::Electricity,
            "NutrientSynthesis" => TechId::CropRotation,
            "PrecisionIndustry" => TechId::Industrialization,
            "PredictiveComputation" => TechId::Electricity,
            "StrategicLogistics" => TechId::MilitaryEngineering,
            "PoweredArmor" => TechId::Gunpowder,
            "PlanetaryAccord" => TechId::Banking,
            "LegacyIntegration" => TechId::PrintingPress,
            _ => return None,
        })
    }
}

impl<'de> Deserialize<'de> for TechId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        TechId::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown technology {name}")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ProductionItem {
    Unit {
        unit: UnitKind,
    },
    Building {
        building: BuildingKind,
    },
    /// A Great Work or capturable goal building, with its chosen worked tile.
    Wonder {
        building: BuildingKind,
        tile: Hex,
    },
    Project {
        project: IndustryProject,
    },
    /// Improve one owned, worked tile that holds a matching resource (`crate::improvements`).
    /// `improvement` is always the one `improvements::kind_for` names for that tile's resource.
    Improvement {
        improvement: ImprovementKind,
        tile: Hex,
    },
}

/// A tile improvement. Upgrades with technology are automatic (`improvements::upgraded`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ImprovementKind {
    Farm,
    Pasture,
    Quarry,
    Mine,
    /// Oil's improvement (Modern era resource).
    OilWell,
    /// Niter's improvement (Renaissance era resource).
    #[serde(alias = "SaltpeterWorks")]
    SaltpetreWorks,
    /// Silicon's improvement (Future era resource).
    #[serde(alias = "LithiumMine")]
    QuartzMine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IndustryProject {
    Gold,
    Science,
    /// Egypt's Nile: the city grows instead of building (`economy::nile`).
    Grow,
}

/// Where a victory point comes from: a stable, typed id every surface can key on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum VpSource {
    /// The permanent point for completing a World Age's card (`era` is the Age, 1-based).
    EraObjective { era: u8 },
    /// 1 VP while this empire holds a rival's original capital (`scoring::capitals_held`).
    CapitalTaken { capital: CityId },
    /// 1 VP kept for good: a rival's original capital this Celtic empire sacked
    /// (`Player::capitals_sacked`).
    CapitalSacked { capital: CityId },
    /// The Great Work built by `of`, scored by the current holder of its tile.
    GreatWork { of: PlayerId },
    /// The faction goal of empire `of`. Usually scored by `of` itself; a held goal (a
    /// capturable item) scores for whoever holds the item.
    FactionGoal { of: PlayerId },
}

/// Derived citizen priorities for the current build; never a player setting.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum CityFocus {
    #[default]
    Balanced,
    Growth,
    Production,
}

/// A number with the parts that make it up, for "no hidden math" tooltips.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Breakdown {
    pub total: i32,
    pub parts: Vec<(String, i32)>,
}

impl Breakdown {
    pub fn add(&mut self, label: impl Into<String>, amount: i32) {
        if amount != 0 {
            self.parts.push((label.into(), amount));
            self.total = self.total.saturating_add(amount);
        }
    }
}
/// Persisted intent; all exploration decisions are made from PlayerView by the client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Automation {
    Explore,
}

/// A human city's saved production delegation. This does not change tile claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CityBuildFocus {
    Balanced,
    Science,
    Gold,
    Military,
}

/// Explicit standing orders, separate from the defense earned by any resting unit.
/// Recorded tapes use the former `Fortify` name for `Sleep`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandingOrder {
    #[serde(alias = "Fortify")]
    Sleep,
    /// Sleep that ends by itself, with a notice, when the unit's HP is full.
    Heal,
    AutoFire,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExploreStopReason {
    Complete,
    Threatened,
    Damaged,
    Blocked,
}
