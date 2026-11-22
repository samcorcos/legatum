#![recursion_limit = "512"]
//! The offline guide can open before a game/WASM client exists. Export its numbers from
//! the rules, and fail CI if the checked-in snapshot drifts. Refresh with:
//! UPDATE_UI_RULES=1 cargo test -p legatum-ai --test ui_rules
use legatum_core::{
    BuildingKind, Faction, ImprovementKind, Resource, TechId, Terrain, UnitKind, company, data,
    diplomacy::{ALLIANCE_PROTECTED_TURNS, BETRAYAL_TURNS, PEACE_TRUCE_TURNS, TreatyKind},
    goals, resources,
    rules::promotion,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

/// The display calendar's labels are exported for turns 0..=this; the UI clamps beyond it.
const CALENDAR_LABEL_TURNS: u32 = 1000;

#[test]
fn ui_rules_match_core() {
    let wound_immunity = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                UnitKind::ALL
                    .into_iter()
                    .filter(|kind| data::ignores_wounds(faction, *kind))
                    .collect::<Vec<_>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let resources = Resource::ALL
        .map(|r| {
            let stats = resources::stats(r);
            (
                format!("{r:?}"),
                json!({
                    "name": resources::name(r),
                    "class": stats.class,
                    "terrains": stats.terrains,
                    "bonus": stats.bonus,
                    "revealedBy": stats.revealed_by,
                }),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let improvement_kinds = ImprovementKind::ALL
        .map(|kind| {
            (
                format!("{kind:?}"),
                json!({
                    "name": legatum_core::improvements::name(kind, false),
                    "upgradedName": legatum_core::improvements::name(kind, true),
                    "cost": legatum_core::improvements::base_cost(kind),
                    "upgradeTech": legatum_core::improvements::upgrade_tech(kind),
                }),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let improvement_for_resource = Resource::ALL
        .map(|r| {
            (
                format!("{r:?}"),
                json!({
                    "kind": legatum_core::improvements::kind_for(r),
                    "upgradeYields": legatum_core::improvements::upgrade_yields(r),
                }),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    // Per faction: effects that do nothing under a faction's rules are dropped (pillar 5).
    let tech_card_effects = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                TechId::ALL
                    .map(|tech| {
                        (
                            format!("{tech:?}"),
                            data::tech_card_effects_for(faction, tech),
                        )
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let tech_benefit = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                TechId::ALL
                    .map(|tech| {
                        (
                            format!("{tech:?}"),
                            data::tech_has_benefit(Default::default(), faction, tech),
                        )
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    // Faction replacement names (only where they differ from the generic item).
    let faction_names = Faction::ALL
        .map(|faction| {
            let buildings = BuildingKind::ALL.into_iter().filter_map(|b| {
                let name = data::building_name_for(faction, b);
                (name != data::building_name(b)).then(|| (format!("{b:?}"), name))
            });
            let units = UnitKind::ALL.into_iter().filter_map(|u| {
                let name = data::unit_name_for(faction, u);
                (name != data::unit_name(u)).then(|| (format!("{u:?}"), name))
            });
            (
                format!("{faction:?}"),
                buildings.chain(units).collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    // The technology table, so the app does not hand-mirror it (era overhaul).
    let techs = TechId::ALL
        .into_iter()
        .map(|tech| {
            let def = legatum_core::eras::tech_def(tech);
            (
                format!("{tech:?}"),
                json!({
                    "name": def.name,
                    "era": def.era,
                    "cost": def.cost,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let yields_json = |y: legatum_core::Yields| {
        json!({
            "food": y.food,
            "production": y.production,
            "gold": y.gold,
            "science": y.science,
        })
    };
    let building_names = BuildingKind::ALL
        .into_iter()
        .map(|b| (format!("{b:?}"), data::building_name(b)))
        .collect::<BTreeMap<_, _>>();
    let building_stats = BuildingKind::ALL
        .into_iter()
        .map(|b| {
            let stats = data::building_stats(b);
            (
                format!("{b:?}"),
                json!({
                    "cost": stats.cost,
                    "yields": yields_json(stats.yields),
                    "cityStrength": stats.city_strength,
                    "requires": stats.requires,
                    "warBuilding": stats.war_building,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let unit_stats = UnitKind::ALL
        .into_iter()
        .map(|u| {
            let stats = data::unit_stats(u);
            (
                format!("{u:?}"),
                json!({
                    "cost": stats.cost,
                    "moves": stats.moves,
                    "strength": stats.strength,
                    "ranged": stats.ranged,
                    "designStrength": data::design_stats(u).strength,
                    "designRanged": data::design_stats(u).ranged,
                    "siegeBonus": data::siege_city_bonus(u),
                    "range": stats.range,
                    "military": stats.military,
                    "requires": stats.requires,
                    "upgradesTo": stats.upgrades_to,
                    "class": data::unit_class(u),
                    "eraTier": data::unit_era_tier(u),
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let unit_stats_json = |stats: data::UnitStats| {
        json!({
            "cost": stats.cost, "moves": stats.moves, "strength": stats.strength,
            "ranged": stats.ranged, "range": stats.range, "military": stats.military,
            "requires": stats.requires, "upgradesTo": stats.upgrades_to,
        })
    };
    let building_stats_json = |stats: data::BuildingStats| {
        json!({
            "cost": stats.cost, "yields": yields_json(stats.yields),
            "cityStrength": stats.city_strength, "requires": stats.requires,
            "warBuilding": stats.war_building,
        })
    };
    let faction_unit_stats = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                UnitKind::ALL
                    .map(|kind| {
                        (
                            format!("{kind:?}"),
                            unit_stats_json(data::unit_stats_for(faction, kind)),
                        )
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let standard_unit_stats = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                UnitKind::ALL
                    .map(|kind| {
                        (
                            format!("{kind:?}"),
                            unit_stats_json(data::standard_unit_stats_for(faction, kind)),
                        )
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let faction_building_stats = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                BuildingKind::ALL
                    .map(|kind| {
                        (
                            format!("{kind:?}"),
                            building_stats_json(data::building_stats_for(faction, kind)),
                        )
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let standard_building_stats = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                BuildingKind::ALL
                    .map(|kind| {
                        (
                            format!("{kind:?}"),
                            building_stats_json(data::standard_building_stats_for(faction, kind)),
                        )
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let terrain_yields = Terrain::ALL
        .map(|terrain| {
            (
                format!("{terrain:?}"),
                yields_json(data::terrain_yields(Faction::India, terrain)),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let city_gaps = Faction::ALL
        .map(|faction| (format!("{faction:?}"), data::min_city_gap(faction)))
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let phalanx_units = Faction::ALL
        .map(|faction| {
            (
                format!("{faction:?}"),
                UnitKind::ALL
                    .map(|kind| {
                        let mut breakdown = legatum_core::Breakdown::default();
                        legatum_core::combat::add_phalanx(&mut breakdown, faction, kind, true);
                        (format!("{kind:?}"), breakdown.total)
                    })
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let unit_needs = UnitKind::ALL
        .into_iter()
        .filter_map(|unit| resources::unit_need(unit).map(|r| (format!("{unit:?}"), json!(r))))
        .collect::<BTreeMap<_, _>>();
    let building_needs = BuildingKind::ALL
        .into_iter()
        .filter_map(|b| resources::building_need(b).map(|r| (format!("{b:?}"), json!(r))))
        .collect::<BTreeMap<_, _>>();
    let expected = json!({
        "saveVersion": legatum_core::RULES_VERSION,
        "factionGoals": Faction::ALL.map(|faction| {
            let spec = goals::spec(faction);
            (
                format!("{faction:?}"),
                json!({
                    "rule": format!("{:?}", spec.rule),
                    "kind": format!("{:?}", spec.kind),
                    "vp": spec.vp,
                    "target": spec.target,
                    "label": spec.label,
                }),
            )
        }).into_iter().collect::<BTreeMap<_, _>>(),
        "signalFreeTechs": goals::SIGNAL_FREE_TECHS,
        "aztecNoHorses": data::AZTEC_NO_HORSES,
        "horseMounted": UnitKind::ALL.into_iter().filter(|u| data::is_horse_mounted(*u)).collect::<Vec<_>>(),
        "swarmPopulation": goals::AZTEC_NEW_FIRE_POPULATION,
        "celtsSackGoldPerPopulation": legatum_core::economy::SACK_GOLD_PER_POPULATION,
        "spartaRetireLevel": goals::SPARTA_RETIRE_LEVEL,
        "firstGrowth": data::FIRST_GROWTH,
        "growthBase": data::GROWTH_BASE,
        "growthPerPop": data::GROWTH_PER_POP,
        "settlerCostStep": data::SETTLER_COST_STEP,
        "firstSettlerPercent": data::FIRST_SETTLER_PERCENT,
        "germanyProjectGoldPercent": data::GERMANY_GOLD_PROJECT_PERCENT,
        "egyptNileFoodPercent": legatum_core::experiments::EGYPT_NILE_FOOD_PERCENT,
        "egyptNileBuildPercent": legatum_core::experiments::EGYPT_NILE_BUILD_PERCENT,
        "germanyProjectSciencePercent": data::GERMANY_SCIENCE_PROJECT_PERCENT,
        "projectGoldPercent": data::PROJECT_GOLD_PERCENT,
        "projectSciencePercent": data::PROJECT_SCIENCE_PERCENT,
        "keshig": {
            "kind": format!("{:?}", data::KESHIG_KIND),
            "cost": data::KESHIG_COST,
            "ranged": data::KESHIG_RANGED,
            "range": data::KESHIG_RANGE,
        },
        "specialUnits": Faction::ALL
            .map(|f| (format!("{f:?}"), format!("{:?}", data::special_unit(f))))
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        // What each special unit changes against the unit it replaces, as the stats and the
        // breakdown lines carry it (era-scaled), so the app never mirrors the arithmetic.
        "specialUnitNumbers": {
            "eagleWarriorStrength": data::unit_stats_for(Faction::Aztecs, UnitKind::Soldier).strength - data::unit_stats(UnitKind::Soldier).strength,
            "eagleWarriorCost": data::EAGLE_WARRIOR_COST,
            "gaesataeAttack": legatum_core::combat::gaesatae_attack(Default::default(), Faction::Celts, UnitKind::Phalangite),
            "gaesataeDefense": legatum_core::combat::gaesatae_defense(Default::default(), Faction::Celts, UnitKind::Phalangite),
            "peltastMoves": data::unit_stats_for(Faction::Athens, UnitKind::CompositeArcher).moves,
            "warElephantStrength": data::unit_stats_for(Faction::India, UnitKind::Knight).strength - data::unit_stats(UnitKind::Knight).strength,
            "warElephantMoves": data::unit_stats_for(Faction::India, UnitKind::Knight).moves,
            "reislauferMountedPercent": data::REISLAUFER_MOUNTED_PERCENT,
            "panzerMoves": data::unit_stats_for(Faction::Germany, UnitKind::Tank).moves,
        },
        "indiaCapitalRadius": data::INDIA_CAPITAL_RADIUS,
        "indiaNetworkRadius": data::INDIA_NETWORK_RADIUS,
        "indiaNetworkPercentPerCity": data::INDIA_NETWORK_PERCENT_PER_CITY,
        "indiaYieldPercentPerHex": data::INDIA_YIELD_PERCENT_PER_HEX,
        "swissInterestThreshold": data::SWISS_INTEREST_THRESHOLD,
        "swissTollPopulation": data::SWISS_TOLL_POPULATION,
        "startGold": data::START_GOLD,
        "resources": resources,
        "improvements": {
            "kinds": improvement_kinds,
            "forResource": improvement_for_resource,
        },
        "eraNames": legatum_core::eras::ERA_NAMES,
        "calendarLabels": (0..=CALENDAR_LABEL_TURNS).map(legatum_core::calendar::calendar_label).collect::<Vec<_>>(),
        "techDiffusionPerEmpire": data::TECH_DIFFUSION_PER_EMPIRE,
        "techDiffusionMax": data::TECH_DIFFUSION_MAX,
        "hoplitePhalanxStrength": data::HOPLITE_PHALANX_STRENGTH,
        "railwayMoves": data::RAILWAY_MOVES,
        "techOrder": TechId::ALL.iter().map(|t| format!("{t:?}")).collect::<Vec<_>>(),
        "techs": techs,
        "buildingNames": building_names,
        "buildingStats": building_stats,
        "factionBuildingStats": faction_building_stats,
        "standardBuildingStats": standard_building_stats,
        "terrainYields": terrain_yields,
        "cityGaps": city_gaps,
        "unitStats": unit_stats,
        "factionUnitStats": faction_unit_stats,
        "standardUnitStats": standard_unit_stats,
        "unitOrder": UnitKind::ALL.iter().map(|u| format!("{u:?}")).collect::<Vec<_>>(),
        "unitNames": UnitKind::ALL
            .iter()
            .map(|u| (format!("{u:?}"), data::unit_name(*u)))
            .collect::<BTreeMap<_, _>>(),
        "armyStrengthReference": legatum_core::army::STRENGTH_REFERENCE,
        "classBonusPercent": legatum_core::combat::CLASS_BONUS_PERCENT,
        "mountedStrengthPercent": data::MOUNTED_STRENGTH_PERCENT,
        "mountedCityPenaltyPercent": legatum_core::combat::MOUNTED_CITY_PENALTY_PERCENT,
        "meleeVsMountedLabel": legatum_core::combat::MELEE_VS_MOUNTED_LABEL,
        "mountedVsRangedLabel": legatum_core::combat::MOUNTED_VS_RANGED_LABEL,
        "mountedCityLabel": legatum_core::combat::MOUNTED_CITY_LABEL,
        "rollMin": legatum_core::combat::ROLL_MIN,
        "rollMax": legatum_core::combat::ROLL_MAX,
        "unitHp": data::UNIT_HP,
        "cityHp": data::CITY_HP,
        "cityHeal": data::CITY_HEAL,
        "maxWallTier": data::MAX_WALL_TIER,
        "wallTiers": [
            [0, 0, 0],
            [
                data::wall_tier([BuildingKind::Walls].iter()),
                data::wall_tier([BuildingKind::Walls, BuildingKind::Castle].iter()),
                data::wall_tier([BuildingKind::Walls, BuildingKind::Castle, BuildingKind::StarFort].iter()),
            ],
        ],
        "phalanxLabel": legatum_core::combat::PHALANX_LABEL,
        "phalanxUnits": phalanx_units,
        "eraCurve": {
            "stepPercent": data::ERA_STEP_PERCENT,
            "meleeLine": data::MELEE_LINE,
            "designMeleeLine": data::DESIGN_MELEE_LINE,
        },
        "walls": {
            "damagePercent": legatum_core::combat::WALLS_DAMAGE_PERCENT,
            "siegePartialPercent": legatum_core::combat::SIEGE_PARTIAL_PERCENT,
        },
        "siege": {
            "baseGold": data::SIEGE_BASE_GOLD,
            "usesIron": data::SIEGE_USES_IRON,
            "builtInEra": data::BUILT_IN_SIEGE_ERA,
            "builtInNote": data::BUILT_IN_SIEGE_NOTE,
        },
        "mountedMoves": data::MOUNTED_MOVES,
        "mongolWallsEra": data::MONGOL_WALLS_ERA,
        "mongolWallsReason": data::MONGOL_WALLS_REASON,
        "mongolSiegeReason": data::MONGOL_SIEGE_REASON,
        "tankMoves": data::TANK_MOVES,
        "landshipMoves": data::LANDSHIP_MOVES,
        "indirectFire": UnitKind::ALL
            .into_iter()
            .filter(|kind| data::indirect_fire(*kind, true))
            .collect::<Vec<_>>(),
        "autonomousArmourMoves": data::AUTONOMOUS_ARMOUR_MOVES,
        "triangleEndsAtEraTier": legatum_core::combat::TRIANGLE_END_ERA_TIER,
        "airBaseCap": data::AIR_BASE_CAP,
        "fighterRange": data::FIGHTER_RANGE,
        "fighterStrikeRange": data::FIGHTER_STRIKE_RANGE,
        "bomberRange": data::BOMBER_RANGE,
        "jetFighterStrikeRange": data::JET_FIGHTER_STRIKE_RANGE,
        "jetBomberRange": data::JET_BOMBER_RANGE,
        "rocketArtilleryRange": data::ROCKET_ARTILLERY_RANGE,
        "mechanisedInfantryMoves": data::MECHANISED_INFANTRY_MOVES,
        "biplaneStrikeRange": data::BIPLANE_STRIKE_RANGE,
        "biplaneGuardRange": data::BIPLANE_GUARD_RANGE,
        "aircraftSight": data::AIRCRAFT_SIGHT,
        "reconRadius": data::RECON_RADIUS,
        "biplaneCityDamagePercent": data::BIPLANE_CITY_DAMAGE_PERCENT,
        "techCardEffects": tech_card_effects,
        "techBenefit": tech_benefit,
        "factionNames": faction_names,
        "supply": {
            "perDeposit": resources::SUPPLY_PER_DEPOSIT,
            "perMine": resources::SUPPLY_PER_MINE,
            "unsuppliedStrengthPercent": resources::UNSUPPLIED_STRENGTH_PERCENT,
            "strategicSiteValue": resources::STRATEGIC_SITE_VALUE,
            "unitNeeds": unit_needs,
            "buildingNeeds": building_needs,
            "eraResources": resources::ERA_RESOURCES,
            "strategic": Resource::STRATEGIC,
        },
        "captureResistanceTurns": data::CAPTURE_RESISTANCE_TURNS,
        "aiAllianceAfterTurn": legatum_ai::ALLIANCE_ACQUAINTANCE_TURNS,
        "campClearGold": legatum_core::rules::barbarians::CAMP_CLEAR_GOLD,
        "firstCampTurn": legatum_core::rules::barbarians::FIRST_CAMP_TURN,
        "woundImmunity": wound_immunity,
        "promotion": {
            "maxLevel": promotion::MAX_LEVEL,
            "xp": (1..=promotion::MAX_LEVEL)
                .map(|level| promotion::threshold(level).unwrap())
                .collect::<Vec<_>>(),
            "ironGuardXp": promotion::threshold_for(Faction::Sparta, promotion::SPARTA_MAX_LEVEL).unwrap(),
            "spartaMaxLevel": promotion::SPARTA_MAX_LEVEL,
            "spartaXpPercent": 150,
            "strengthPercent": data::PROMOTION_STRENGTH_PERCENT,
            "meleeXp": data::XP_MELEE_BONUS,
            "rangedXp": data::XP_RANGED_BONUS,
            "initiatorXp": data::XP_INITIATOR_BONUS,
            "unitFightCap": data::XP_UNIT_FIGHT_CAP,
            "cityAttackXp": data::XP_CITY_ATTACK,
            "cityDefeatedXp": data::XP_CITY_DEFEATED,
        },
        "company": {
            "turns": company::COMPANY_TURNS,
            "upkeep": company::UPKEEP,
            "hirePercent": company::HIRE_PERCENT,
        },
        "treaty": {
            "guaranteeTurns": PEACE_TRUCE_TURNS,
            "Alliance": {
                "upkeep": TreatyKind::Alliance.upkeep(),
                "breachCost": TreatyKind::Alliance.breach_cost(),
                "protectedTurns": ALLIANCE_PROTECTED_TURNS,
                "betrayalTurns": BETRAYAL_TURNS,
            },
        },
        "decadence": {
            "powerWindowTurns": legatum_core::experiments::POWER_WINDOW_TURNS,
        },
        "siteBands": {
            "floors": legatum_core::settlement::SITE_BAND_FLOORS,
            "names": legatum_core::settlement::SiteBand::ALL.map(|band| band.name()),
        },
        "territory": {
            "workRadius": data::CITY_WORK_RADIUS,
            "tileGoldBase": data::TILE_GOLD_BASE,
            "tileGoldPerTile": data::TILE_GOLD_PER_TILE,
            "tileRingStepPercent": data::TILE_RING_STEP_PERCENT,
            "irrigationFood": data::IRRIGATION_FOOD,
            "indiaNetworkFood": data::INDIA_NETWORK_FOOD,
            "claimPullWeight": legatum_core::rules::cities::CLAIM_PULL_WEIGHT,
        },
    });
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src/rules.generated.json");
    if std::env::var_os("UPDATE_UI_RULES").is_some() {
        fs::write(
            &file,
            format!("{}\n", serde_json::to_string_pretty(&expected).unwrap()),
        )
        .unwrap();
    }
    let actual: Value =
        serde_json::from_str(&fs::read_to_string(file).expect("generate UI rules first")).unwrap();
    assert_eq!(
        actual, expected,
        "Refresh with UPDATE_UI_RULES=1 cargo test -p legatum-ai --test ui_rules"
    );
}
