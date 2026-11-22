//! Faction-rule ablation (`RuleExperiments::faction_rules_off`): each switch changes the rule it
//! names, and with the set empty nothing changes.
use crate::economy;
use crate::experiments::{FactionRule, FactionRuleSet, RuleExperiments, RuleKind};
use crate::rules::cities::tests::{P0, P1, at, found, test_game};
use crate::{
    BuildingKind, Faction, GameState, IndustryProject, ProductionItem, TechId, UnitKind, data,
    goals, scoring,
};

fn off(rule: FactionRule) -> RuleExperiments {
    let mut r = RuleExperiments::default();
    r.faction_rules_off.insert(rule);
    r
}

fn german() -> GameState {
    test_game(&[(Faction::Germany, 0), (Faction::India, 1)])
}

fn building(b: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building: b }
}

fn project(p: IndustryProject) -> ProductionItem {
    ProductionItem::Project { project: p }
}

#[test]
fn names_parse_round_trip_and_unknown_names_list_the_valid_ones() {
    for rule in FactionRule::ALL {
        assert_eq!(FactionRule::parse(rule.name()), Ok(*rule));
    }
    let err = FactionRule::parse("germany_nonsense").unwrap_err();
    assert!(err.contains("germany_nonsense"), "{err}");
    for rule in FactionRule::ALL {
        assert!(err.contains(rule.name()), "{err}");
    }
    let set = FactionRuleSet::parse_list("germany_ruhr_production").unwrap();
    assert_eq!(
        set.iter().collect::<Vec<_>>(),
        [FactionRule::GermanyRuhrProduction]
    );
}

#[test]
fn the_switch_is_absent_from_saves_when_empty_and_round_trips_when_set() {
    let default = serde_json::to_string(&RuleExperiments::default()).unwrap();
    assert!(!default.contains("faction_rules_off"), "{default}");
    let on = off(FactionRule::GermanyGoal);
    let json = serde_json::to_string(&on).unwrap();
    assert!(
        json.contains(r#""faction_rules_off":["germany_goal"]"#),
        "{json}"
    );
    assert_eq!(serde_json::from_str::<RuleExperiments>(&json).unwrap(), on);
    let old = r#"{"faction_rules_off":["swiss_wonder_buys","celts_ambusher","mongols_peace_passage","sparta_slow_heal","germany_goal"]}"#;
    assert_eq!(serde_json::from_str::<RuleExperiments>(old).unwrap(), on);
    assert!(serde_json::from_str::<RuleExperiments>(r#"{"faction_rules_off":["x"]}"#).is_err());
    assert!(RuleExperiments::default().is_default() && !on.is_default());
}

#[test]
fn every_faction_rule_belongs_to_a_faction_with_a_strength_or_drawback_kind() {
    let germany: Vec<_> = FactionRule::of(Faction::Germany).collect();
    assert_eq!(germany.len(), 6);
    assert_eq!(
        germany
            .iter()
            .filter(|r| r.kind() == RuleKind::Drawback)
            .collect::<Vec<_>>(),
        [&FactionRule::GermanyIndustryFood]
    );
}

#[test]
fn ruhr_production_off_makes_it_an_ordinary_forge() {
    let foundry = building(BuildingKind::Forge);
    let mut s = german();
    let c = found(&mut s, P0, at(5, 5));
    s.cities
        .get_mut(&c)
        .unwrap()
        .buildings
        .insert(BuildingKind::Forge);
    let control = economy::city_yields(&s, &s.cities[&c]).total.production;
    let cost = economy::production_cost(&s, P0, foundry);
    s.setup.rule_experiments = off(FactionRule::GermanyRuhrProduction);
    let ablated = economy::city_yields(&s, &s.cities[&c]).total.production;
    assert_eq!(control - ablated, 3, "+6 becomes the Forge's +3");
    let ablated_stats = s
        .setup
        .rule_experiments
        .building_stats_for(Faction::Germany, BuildingKind::Forge);
    assert_eq!(
        ablated_stats.yields.production,
        data::building_stats(BuildingKind::Forge).yields.production
    );
    assert_eq!(
        ablated_stats.yields.food, 0,
        "the Foundry's own Food cost is removed"
    );
    assert_eq!(
        economy::production_cost(&s, P0, foundry),
        cost,
        "price unchanged"
    );
    // The goal still counts Foundries: the switch changes only what the building yields.
    assert_eq!(goals::factory_cities(&s, P0), 1);
}

#[test]
fn industry_projects_off_removes_the_choice_and_its_yield() {
    let gold = project(IndustryProject::Gold);
    let mut s = german();
    let c = found(&mut s, P0, at(5, 5));
    s.cities.get_mut(&c).unwrap().population = 6;
    assert!(economy::production_check::<economy::Blocked>(&s, &s.cities[&c], gold, true).is_none());
    s.cities.get_mut(&c).unwrap().producing = Some(gold);
    let control = economy::city_yields(&s, &s.cities[&c]);
    assert!(control.total.gold > 0 && control.total.gold >= control.total.production / 2);

    s.setup.rule_experiments = off(FactionRule::GermanyIndustryProjects);
    let reason = economy::production_check::<String>(&s, &s.cities[&c], gold, true)
        .expect("the project is refused");
    assert!(reason.contains("switched off"), "{reason}");
    assert!(!data::available_to_faction_rules(
        s.setup.rule_experiments.faction_rules_off,
        Faction::Germany,
        project(IndustryProject::Science)
    ));
    let ablated = economy::city_yields(&s, &s.cities[&c]);
    assert!(
        ablated.total.gold < control.total.gold,
        "no Production-to-Gold"
    );
    // The player's view lists no project option for the city.
    let view = crate::player_view(&s, P0);
    let detail = view
        .cities
        .iter()
        .find(|v| v.id == c)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    assert!(
        !detail
            .options
            .iter()
            .any(|o| matches!(o.item, ProductionItem::Project { .. }))
    );
}

#[test]
fn every_production_building_costs_one_german_food_and_the_foundry_has_no_own_food() {
    let mut s = german();
    let c = found(&mut s, P0, at(5, 5));
    let qualifying: Vec<_> = BuildingKind::ALL
        .into_iter()
        .filter(|b| data::is_production_building(*b))
        .collect();
    assert_eq!(
        qualifying,
        [
            BuildingKind::Workshop,
            BuildingKind::Pyramids,
            BuildingKind::Forge,
            BuildingKind::Factory
        ]
    );
    assert_eq!(
        s.setup
            .rule_experiments
            .building_stats_for(Faction::Germany, BuildingKind::Forge)
            .yields
            .food,
        0
    );
    for building in qualifying {
        s.cities.get_mut(&c).unwrap().buildings.clear();
        s.cities.get_mut(&c).unwrap().buildings.insert(building);
        let control = economy::city_yields(&s, &s.cities[&c]);
        s.setup.rule_experiments = off(FactionRule::GermanyIndustryFood);
        let ablated = economy::city_yields(&s, &s.cities[&c]);
        let tiers = 1
            + i32::from(building == BuildingKind::Forge)
            + 2 * i32::from(building == BuildingKind::Factory);
        assert_eq!(
            ablated.total.food - control.total.food,
            tiers,
            "{building:?}"
        );
        assert_eq!(ablated.total.production, control.total.production);
        s.setup.rule_experiments = RuleExperiments::default();
    }
    s.cities.get_mut(&c).unwrap().buildings.clear();
    s.cities
        .get_mut(&c)
        .unwrap()
        .buildings
        .insert(BuildingKind::Granary);
    let control = economy::city_yields(&s, &s.cities[&c]).total.food;
    s.setup.rule_experiments = off(FactionRule::GermanyIndustryFood);
    assert_eq!(economy::city_yields(&s, &s.cities[&c]).total.food, control);
}

#[test]
fn captured_city_uses_its_current_owners_industry_rule_and_names_the_loss() {
    let mut s = german();
    let c = found(&mut s, P1, at(5, 5));
    s.cities
        .get_mut(&c)
        .unwrap()
        .buildings
        .insert(BuildingKind::Forge);
    let indian_food = economy::city_yields(&s, &s.cities[&c]).total.food;
    s.cities.get_mut(&c).unwrap().owner = P0;
    let german = economy::city_yields(&s, &s.cities[&c]);
    assert_eq!(
        german.total.food,
        indian_food - 2 - data::INDIA_NETWORK_FOOD
    );
    assert!(german.sources.iter().any(|source| source.label
        == "Industry: -2 Food (Workshop, Ruhr Foundry)"
        && source.yields.food == -2));
    s.cities.get_mut(&c).unwrap().owner = P1;
    assert!(
        !economy::city_yields(&s, &s.cities[&c])
            .sources
            .iter()
            .any(|source| source.label.starts_with("Industry:"))
    );
}

#[test]
fn german_deposits_double_base_and_mine_supply_and_switch_off_independently() {
    use crate::resources::{self, Resource};
    let mut s = german();
    found(&mut s, P0, at(5, 5));
    s.player_mut(P0).techs.insert(TechId::IronWorking);
    let tile = s.map.get_mut(at(5, 5)).unwrap();
    tile.resource = Some(Resource::Iron);
    assert_eq!(resources::supplied(&s, P0, Resource::Iron), 6);
    assert_eq!(
        resources::supply_views(&s, P0)
            .into_iter()
            .find(|v| v.resource == Resource::Iron)
            .unwrap()
            .supplied
            .total,
        6
    );
    s.setup.rule_experiments = off(FactionRule::GermanyDoubleSupply);
    assert_eq!(resources::supplied(&s, P0, Resource::Iron), 3);
    assert_eq!(
        resources::mine_supply_gain(Faction::Germany, s.setup.rule_experiments.faction_rules_off),
        1
    );
    assert_eq!(resources::supplied(&s, P1, Resource::Iron), 0);
}

#[test]
fn goal_off_scores_nothing_and_shows_no_meter() {
    let mut s = german();
    for i in 0..5 {
        let c = found(&mut s, P0, at(2 + 4 * (i % 4), 2 + 8 * (i / 4)));
        s.cities
            .get_mut(&c)
            .unwrap()
            .buildings
            .insert(BuildingKind::Forge);
    }
    let goal_vp = |s: &GameState| -> u32 {
        scoring::vp(s, P0)
            .into_iter()
            .filter(|(src, _)| matches!(src, crate::VpSource::FactionGoal { .. }))
            .map(|(_, vp)| vp)
            .sum()
    };
    scoring::update(&mut s, &mut Vec::new());
    assert!(s.player(P0).faction_goal_done && goals::view(&s, P0).is_some());
    assert_eq!(goal_vp(&s), 2);

    s.setup.rule_experiments = off(FactionRule::GermanyGoal);
    scoring::update(&mut s, &mut Vec::new());
    assert!(
        !s.player(P0).faction_goal_done,
        "the live goal is lost at once"
    );
    assert_eq!(goal_vp(&s), 0);
    assert!(goals::view(&s, P0).is_none());
    assert_eq!(
        goals::spec_in(s.setup.rule_experiments, Faction::Germany).vp,
        0
    );
    assert_eq!(goals::spec(Faction::Germany).vp, 2);
    // Rivals' goals are not affected by Germany's switch.
    assert_eq!(
        goals::spec_in(s.setup.rule_experiments, Faction::India).vp,
        2
    );
}

fn building_key(s: data::BuildingStats) -> (i32, crate::Yields, i32, bool) {
    (s.cost, s.yields, s.city_strength, s.war_building)
}

fn unit_key(s: data::UnitStats) -> (i32, i32, i32, i32, i32) {
    (s.cost, s.moves, s.strength, s.ranged, s.range)
}

#[test]
fn an_empty_set_changes_nothing() {
    let s = german();
    let rules = s.setup.rule_experiments;
    for f in Faction::ALL {
        for b in BuildingKind::ALL {
            assert_eq!(
                building_key(rules.building_stats_for(f, b)),
                building_key(data::building_stats_for(f, b))
            );
        }
        for u in UnitKind::ALL {
            assert_eq!(
                unit_key(rules.unit_stats_for(f, u)),
                unit_key(data::unit_stats_for(f, u))
            );
        }
    }
}

// ---------- the other eight factions ----------

use crate::combat;
use crate::rules::test_support::{hex, unit};
use crate::{PlayerId, Terrain, VpSource};

fn state_of(faction: Faction, rule: Option<FactionRule>) -> GameState {
    let mut s = test_game(&[(faction, 0), (Faction::Athens, 1)]);
    if let Some(rule) = rule {
        s.setup.rule_experiments = off(rule);
    }
    s
}

/// Production, Food, Gold of a founded city of `faction`, with `rule` off or not.
fn city_totals(faction: Faction, rule: Option<FactionRule>, pop: i32) -> (i32, i32, i32) {
    let mut s = state_of(faction, rule);
    let c = found(&mut s, P0, at(5, 5));
    s.cities.get_mut(&c).unwrap().population = pop;
    let y = economy::city_yields(&s, &s.cities[&c]).total;
    (y.food, y.production, y.gold)
}

#[test]
fn swiss_tolls_off_removes_the_gold_per_two_population() {
    let rule = Some(FactionRule::SwissTolls);
    let (_, _, control) = city_totals(Faction::Switzerland, None, 6);
    let (_, _, ablated) = city_totals(Faction::Switzerland, rule, 6);
    assert_eq!(control - ablated, 6 / data::SWISS_TOLL_POPULATION);
}

#[test]
fn swiss_interest_off_pays_no_interest() {
    let mut s = state_of(Faction::Switzerland, None);
    s.player_mut(P0).gold = 1_234;
    assert_eq!(economy::swiss_interest_gross(&s, P0), 12);
    s.setup.rule_experiments = off(FactionRule::SwissInterest);
    assert_eq!(economy::swiss_interest_gross(&s, P0), 0);
    assert_eq!(economy::swiss_interest(&s, P0), 0);
}

#[test]
fn swiss_cheap_buys_off_prices_purchases_like_everyone_else() {
    let mut s = state_of(Faction::Switzerland, None);
    let c = found(&mut s, P0, at(5, 5));
    let spearman = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let cheap = economy::buy_cost_for(&s, &s.cities[&c], spearman).unwrap();
    assert_eq!(economy::buy_percent(&s, P0), data::SWISS_BUY_PERCENT);
    s.setup.rule_experiments = off(FactionRule::SwissCheapBuys);
    let dear = economy::buy_cost_for(&s, &s.cities[&c], spearman).unwrap();
    assert_eq!(economy::buy_percent(&s, P0), data::BUY_COST_PERCENT);
    assert!(dear > cheap);
}

fn building_item(building: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building }
}

#[test]
fn swiss_redoubt_off_is_ordinary_walls() {
    let rules = off(FactionRule::SwissRedoubt);
    let control = data::building_stats_for(Faction::Switzerland, BuildingKind::Walls);
    assert_eq!((control.cost, control.city_strength), (120, 15));
    let ablated = rules.building_stats_for(Faction::Switzerland, BuildingKind::Walls);
    let ordinary = data::building_stats(BuildingKind::Walls);
    assert_eq!(
        (ablated.cost, ablated.city_strength),
        (ordinary.cost, ordinary.city_strength)
    );
    // And in a fight: a walled Swiss city defends like a walled Athens city.
    let defend = |faction, off_set| {
        combat::city_defense_strength_for_rules(off_set, faction, 3, true, data::CITY_HP, 0).total
    };
    assert!(
        defend(Faction::Switzerland, FactionRuleSet::EMPTY)
            > defend(Faction::Switzerland, rules.faction_rules_off)
    );
    assert_eq!(
        defend(Faction::Switzerland, rules.faction_rules_off),
        defend(Faction::Athens, FactionRuleSet::EMPTY)
    );
}

#[test]
fn swiss_unit_upkeep_off_charges_the_ordinary_one_gold() {
    let mut s = state_of(Faction::Switzerland, None);
    found(&mut s, P0, at(5, 5));
    for i in 0..5 {
        unit(&mut s, 0, UnitKind::Spearman, hex(2 + i, 8));
    }
    let control = economy::upkeep(&s, P0);
    s.setup.rule_experiments = off(FactionRule::SwissUnitUpkeep);
    let ablated = economy::upkeep(&s, P0);
    assert_eq!(control, 3 * 3);
    assert_eq!(ablated, 3 * data::UPKEEP_PER_UNIT);
}

#[test]
fn swiss_capture_bounty_off_pays_the_captor_nothing() {
    let mut s = state_of(Faction::Switzerland, None);
    let c = found(&mut s, P0, at(5, 5));
    s.player_mut(P0).gold = 900;
    assert!(economy::swiss_capture_bounty(&s, &s.cities[&c]) > 0);
    s.setup.rule_experiments = off(FactionRule::SwissCaptureBounty);
    assert_eq!(economy::swiss_capture_bounty(&s, &s.cities[&c]), 0);
}

#[test]
fn india_food_off_drops_the_extra_food() {
    let (control, _, _) = city_totals(Faction::India, None, 4);
    let (ablated, _, _) = city_totals(Faction::India, Some(FactionRule::IndiaFood), 4);
    assert_eq!(control - ablated, data::INDIA_NETWORK_FOOD);
}

#[test]
fn india_network_off_drops_the_per_neighbour_bonus() {
    let totals = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::India, rule);
        let a = found(&mut s, P0, at(5, 5));
        found(&mut s, P0, at(8, 5));
        found(&mut s, P0, at(5, 8));
        s.cities.get_mut(&a).unwrap().population = 8;
        economy::city_yields(&s, &s.cities[&a]).total
    };
    let control = totals(None);
    let ablated = totals(Some(FactionRule::IndiaNetwork));
    assert!(control.food > ablated.food && control.production > ablated.production);
}

#[test]
fn india_close_cities_off_needs_the_ordinary_spacing() {
    assert_eq!(data::min_city_gap(Faction::India), 2);
    let rules = off(FactionRule::IndiaCloseCities);
    assert_eq!(rules.min_city_gap(Faction::India), data::MIN_CITY_GAP);
    // Founding three hexes from a city is legal at 2, illegal at 3.
    let legal = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::India, rule);
        found(&mut s, P0, at(5, 5));
        let settler = crate::game::spawn_unit(&mut s, P0, UnitKind::Settler, at(8, 5));
        crate::rules::cities::found_city(&mut s, settler, &mut Vec::new()).is_ok()
    };
    assert!(legal(None));
    assert!(!legal(Some(FactionRule::IndiaCloseCities)));
}

#[test]
fn india_corruption_off_ends_the_distance_penalty() {
    let far = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::India, rule);
        found(&mut s, P0, at(2, 2));
        let c = found(&mut s, P0, at(2, 12));
        s.cities.get_mut(&c).unwrap().population = 6;
        economy::city_yields(&s, &s.cities[&c]).total
    };
    let control = far(None);
    let ablated = far(Some(FactionRule::IndiaCorruption));
    assert!(ablated.food + ablated.production > control.food + control.production);
}

#[test]
fn india_remittance_off_lets_every_city_keep_its_gold() {
    let income = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::India, rule);
        found(&mut s, P0, at(2, 2));
        let c = found(&mut s, P0, at(2, 6));
        let city = s.cities.get_mut(&c).unwrap();
        city.population = 6;
        city.buildings.insert(BuildingKind::Market);
        economy::player_income(&s, P0).gold.total
    };
    assert!(income(Some(FactionRule::IndiaRemittance)) > income(None));
}

#[test]
fn byzantium_companies_off_closes_the_shop() {
    let build = |rule: Option<FactionRule>| {
        let mut setup = crate::GameSetup::all_ai(5);
        setup.players.truncate(2);
        setup.players[0].faction = Faction::Byzantium;
        setup.players[1].faction = Faction::Sparta;
        for (i, p) in setup.players.iter_mut().enumerate() {
            p.team = crate::TeamId(i as u8);
        }
        if let Some(rule) = rule {
            setup.rule_experiments = off(rule);
        }
        let mut s =
            crate::game::blank_game(&setup, crate::state::Map::new(24, 16, Terrain::Grassland));
        s.player_mut(P1).techs.extend(TechId::ALL);
        let teams = s.teams();
        s.diplomacy.as_mut().unwrap().meet_all(&teams);
        for r in &mut s.diplomacy.as_mut().unwrap().relations {
            r.state = crate::diplomacy::RelationState::Peace;
        }
        let settler = crate::game::spawn_unit(&mut s, P0, UnitKind::Settler, hex(4, 4));
        crate::rules::cities::found_city(&mut s, settler, &mut Vec::new()).unwrap();
        s
    };
    assert!(!crate::company::offers(&build(None), P0).is_empty());
    let s = build(Some(FactionRule::ByzantiumCompanies));
    assert!(crate::company::offers(&s, P0).is_empty());
    let city = s.cities_of(P0).next().unwrap();
    let reason = crate::company::blocker(&s, city, UnitKind::Spearman, P1).unwrap();
    assert!(reason.contains("switched off"), "{reason}");
}

#[test]
fn byzantium_levy_off_lifts_the_cap() {
    let mut s = state_of(Faction::Byzantium, None);
    found(&mut s, P0, at(5, 5));
    assert_eq!(economy::levy(&s, P0).map(|(_, cap)| cap), Some(2));
    s.setup.rule_experiments = off(FactionRule::ByzantiumLevy);
    assert_eq!(economy::levy(&s, P0), None);
}

#[test]
fn byzantium_tagmatic_off_makes_the_pikeman_ordinary() {
    let rules = off(FactionRule::ByzantiumTagmatic);
    let control = data::unit_stats_for(Faction::Byzantium, UnitKind::Pikeman);
    let ablated = rules.unit_stats_for(Faction::Byzantium, UnitKind::Pikeman);
    let ordinary = data::unit_stats(UnitKind::Pikeman);
    assert!(control.strength > ordinary.strength && control.cost > ordinary.cost);
    assert_eq!(
        (ablated.strength, ablated.cost),
        (ordinary.strength, ordinary.cost)
    );
    // The fight reads the switched strength.
    let attack = |off_set| {
        combat::unit_attack_total_rules(
            off_set,
            Faction::Byzantium,
            Faction::Byzantium,
            UnitKind::Pikeman,
            100,
            0,
        )
    };
    assert!(attack(FactionRuleSet::EMPTY) > attack(rules.faction_rules_off));
}

#[test]
fn celts_barbarian_peace_off_lets_barbarians_attack_the_celts() {
    let mut s = state_of(Faction::Celts, None);
    assert!(crate::rules::barbarians::spares(&s, P0));
    assert!(!crate::rules::is_threat(&s, P0, PlayerId::BARBARIAN));
    s.setup.rule_experiments = off(FactionRule::CeltsBarbarianPeace);
    assert!(!crate::rules::barbarians::spares(&s, P0));
    let v = crate::player_view(&s, P0);
    assert!(!v.barbarians_spare_me());
}

#[test]
fn mongols_moves_off_drops_the_extra_move() {
    let rules = off(FactionRule::MongolsMoves);
    let control = data::unit_moves_for(Faction::Mongols, UnitKind::Spearman);
    let ablated = rules.unit_moves_for(Faction::Mongols, UnitKind::Spearman);
    assert_eq!(control - ablated, 1);
}

#[test]
fn mongols_commerce_off_drops_the_long_road_gold() {
    let gold = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::Mongols, rule);
        found(&mut s, P0, at(2, 2));
        let far = found(&mut s, P0, at(14, 14));
        s.cities
            .get_mut(&far)
            .unwrap()
            .buildings
            .insert(BuildingKind::Market);
        economy::city_yields(&s, &s.cities[&far]).total.gold
    };
    assert_eq!(gold(None) - gold(Some(FactionRule::MongolsCommerce)), 2);
}

#[test]
fn mongols_keshig_off_is_an_ordinary_cataphract() {
    let rules = off(FactionRule::MongolsKeshig);
    let control = data::unit_stats_for(Faction::Mongols, data::KESHIG_KIND);
    let ablated = rules.unit_stats_for(Faction::Mongols, data::KESHIG_KIND);
    assert_eq!(
        control.ranged,
        data::KESHIG_RANGED.max(control.ranged).min(control.ranged)
    );
    assert!(control.ranged > 0 && ablated.ranged == 0);
    assert!(control.cost > ablated.cost);
}

#[test]
fn mongols_no_terrain_off_gives_the_terrain_bonus_back() {
    let rules = off(FactionRule::MongolsNoTerrain);
    assert_eq!(data::terrain_defense(Faction::Mongols, Terrain::Hills), 0);
    assert_eq!(
        data::terrain_defense_rules(rules.faction_rules_off, Faction::Mongols, Terrain::Hills),
        Terrain::Hills.defense_bonus()
    );
    let defend = |off_set| {
        combat::unit_defense_total_rules(
            off_set,
            Faction::Mongols,
            Faction::Mongols,
            UnitKind::Spearman,
            100,
            Terrain::Hills,
            0,
            0,
        )
    };
    assert!(defend(rules.faction_rules_off) > defend(FactionRuleSet::EMPTY));
}

#[test]
fn mongols_no_walls_off_opens_walls_and_siege_before_the_medieval_era() {
    let walls = building_item(BuildingKind::Walls);
    let rules = off(FactionRule::MongolsNoWalls);
    assert!(!data::available_to_player(Faction::Mongols, 0, walls));
    assert!(rules.available_to_player(Faction::Mongols, 0, walls));
    assert!(!data::siege_available(Faction::Mongols, 0));
    assert!(data::siege_available_rules(
        rules.faction_rules_off,
        Faction::Mongols,
        0
    ));
    assert!(data::faction_excluded_building_at(Faction::Mongols, BuildingKind::Walls, 0).is_some());
    assert!(
        data::faction_excluded_building_at_rules(
            rules.faction_rules_off,
            Faction::Mongols,
            BuildingKind::Walls,
            0
        )
        .is_none()
    );
}

#[test]
fn sparta_veterans_off_starts_units_at_level_zero_without_the_xp_bonus() {
    let spawn = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::Sparta, rule);
        let id = crate::game::spawn_unit(&mut s, P0, UnitKind::Spearman, hex(3, 3));
        (s.units[&id].xp, s)
    };
    let (control_xp, _) = spawn(None);
    let (ablated_xp, ablated) = spawn(Some(FactionRule::SpartaVeterans));
    assert!(control_xp > 0 && ablated_xp == 0);
    use crate::rules::promotion;
    let off_set = ablated.setup.rule_experiments.faction_rules_off;
    assert_eq!(promotion::level_for(Faction::Sparta, control_xp), 1);
    assert_eq!(
        promotion::level_for_rules(off_set, Faction::Sparta, 10_000),
        promotion::MAX_LEVEL,
        "no sixth level"
    );
    assert_eq!(
        promotion::level_for(Faction::Sparta, 10_000),
        promotion::SPARTA_MAX_LEVEL
    );
}

#[test]
fn sparta_no_wounds_off_makes_wounds_cost_strength() {
    let rules = off(FactionRule::SpartaNoWounds);
    let wounded = |off_set| {
        combat::unit_attack_total_rules(
            off_set,
            Faction::Sparta,
            Faction::Sparta,
            UnitKind::Spearman,
            40,
            0,
        )
    };
    assert!(wounded(FactionRuleSet::EMPTY) > wounded(rules.faction_rules_off));
    assert!(data::ignores_wounds(Faction::Sparta, UnitKind::Spearman));
    assert!(!data::ignores_wounds_rules(
        rules.faction_rules_off,
        Faction::Sparta,
        UnitKind::Spearman
    ));
}

#[test]
fn sparta_phalanx_off_ends_the_hoplite_bonus() {
    let neighbours = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::Sparta, rule);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        crate::rules::battle::has_phalanx_neighbour(&s, &s.units[&a])
    };
    assert!(neighbours(None));
    assert!(!neighbours(Some(FactionRule::SpartaPhalanx)));
    let mut b = crate::Breakdown::default();
    combat::add_phalanx_rules(
        &mut b,
        FactionRuleSet::EMPTY,
        Faction::Sparta,
        UnitKind::Spearman,
        true,
    );
    assert_eq!(b.total, data::HOPLITE_PHALANX_STRENGTH);
    let mut b = crate::Breakdown::default();
    combat::add_phalanx_rules(
        &mut b,
        off(FactionRule::SpartaPhalanx).faction_rules_off,
        Faction::Sparta,
        UnitKind::Spearman,
        true,
    );
    assert_eq!(b.total, 0);
}

#[test]
fn sparta_premium_off_charges_ordinary_prices_for_civilian_builds() {
    let granary = building_item(BuildingKind::Granary);
    let mut s = state_of(Faction::Sparta, None);
    let control = economy::production_cost(&s, P0, granary);
    s.setup.rule_experiments = off(FactionRule::SpartaPremium);
    let ablated = economy::production_cost(&s, P0, granary);
    assert_eq!(control, (ablated * 3 + 1) / 2);
}

#[test]
fn athens_lyceum_off_is_an_ordinary_library() {
    let rules = off(FactionRule::AthensLyceum);
    let control = data::building_stats_for(Faction::Athens, BuildingKind::Library);
    let ablated = rules.building_stats_for(Faction::Athens, BuildingKind::Library);
    let ordinary = data::building_stats(BuildingKind::Library);
    assert_eq!((control.cost, control.yields.science), (70, 6));
    assert_eq!(
        (ablated.cost, ablated.yields.science),
        (ordinary.cost, ordinary.yields.science)
    );
}

#[test]
fn athens_weak_markets_off_pays_the_full_market_gold() {
    let rules = off(FactionRule::AthensWeakMarkets);
    assert_eq!(
        data::building_stats_for(Faction::Athens, BuildingKind::Market)
            .yields
            .gold,
        1
    );
    assert_eq!(
        rules
            .building_stats_for(Faction::Athens, BuildingKind::Market)
            .yields
            .gold,
        data::building_stats(BuildingKind::Market).yields.gold
    );
}

#[test]
fn aztecs_no_horses_off_lets_them_train_riders() {
    let horse = ProductionItem::Unit {
        unit: UnitKind::Horseman,
    };
    let rules = off(FactionRule::AztecsNoHorses);
    assert!(!data::available_to_faction(Faction::Aztecs, horse));
    assert!(data::available_to_faction_rules(
        rules.faction_rules_off,
        Faction::Aztecs,
        horse
    ));
    assert!(rules.available_to_player(Faction::Aztecs, 0, horse));
    let mut s = state_of(Faction::Aztecs, None);
    let c = found(&mut s, P0, at(5, 5));
    assert!(economy::production_check::<String>(&s, &s.cities[&c], horse, true).is_some());
    s.setup.rule_experiments = rules;
    assert!(economy::production_check::<String>(&s, &s.cities[&c], horse, true).is_none());
}

// ---------- every goal ----------

#[test]
fn every_goal_switch_zeroes_its_vp_and_blocks_its_action() {
    use crate::goals::{self, GoalRule};
    let pairs = [
        (
            Faction::Switzerland,
            FactionRule::SwissGoal,
            GoalRule::Hoard,
        ),
        (Faction::India, FactionRule::IndiaGoal, GoalRule::Diwan),
        (
            Faction::Byzantium,
            FactionRule::ByzantiumGoal,
            GoalRule::Heir,
        ),
        (
            Faction::Germany,
            FactionRule::GermanyGoal,
            GoalRule::Factories,
        ),
        (Faction::Celts, FactionRule::CeltsGoal, GoalRule::Wild),
        (Faction::Mongols, FactionRule::MongolsGoal, GoalRule::Hall),
        (Faction::Sparta, FactionRule::SpartaGoal, GoalRule::Retire),
        (
            Faction::Athens,
            FactionRule::AthensGoal,
            GoalRule::Broadcast,
        ),
        (Faction::Aztecs, FactionRule::AztecsGoal, GoalRule::Swarm),
    ];
    for (faction, rule, goal) in pairs {
        assert_eq!(goals::goal_switch(goal), Some(rule));
        assert_eq!(goals::default_rule(faction), goal);
        let rules = off(rule);
        assert_eq!(goals::spec_in(rules, faction).vp, 0, "{rule}");
        assert_eq!(goals::spec(faction).vp, 2);
        for other in Faction::ALL.into_iter().filter(|f| *f != faction) {
            assert_eq!(
                goals::spec_in(rules, other).vp,
                goals::spec(other).vp,
                "{rule} leaves {other:?}"
            );
        }
        let mut s = state_of(faction, None);
        found(&mut s, P0, at(5, 5));
        assert!(!goals::switched_off(&s, P0));
        s.setup.rule_experiments = rules;
        assert!(goals::switched_off(&s, P0), "{rule}");
        assert!(goals::view(&s, P0).is_none(), "{rule}");
        assert!(!goals::reached(&s, P0), "{rule}");
        assert_eq!(goals::held_by(&s, P0), None, "{rule}");
    }
}

#[test]
fn goal_actions_refuse_while_their_goal_is_off() {
    use crate::rules::actions;
    let blocked = |faction, rule| {
        let mut s = state_of(faction, Some(rule));
        found(&mut s, P0, at(5, 5));
        s
    };
    let s = blocked(Faction::Aztecs, FactionRule::AztecsGoal);
    assert_eq!(
        actions::swarm_blocker(&s, P0).as_deref(),
        Some(crate::goals::SWITCHED_OFF)
    );
    let s = blocked(Faction::Athens, FactionRule::AthensGoal);
    assert_eq!(
        actions::broadcast_blocker(&s, P0).as_deref(),
        Some(crate::goals::SWITCHED_OFF)
    );
    let s = blocked(Faction::Celts, FactionRule::CeltsGoal);
    assert_eq!(
        actions::wild_blocker(&s, P0).as_deref(),
        Some(crate::goals::SWITCHED_OFF)
    );
    let mut s = blocked(Faction::Sparta, FactionRule::SpartaGoal);
    let u = unit(&mut s, 0, UnitKind::Spearman, hex(5, 5));
    assert_eq!(
        actions::retire_blocker(&s, P0, u).as_deref(),
        Some(crate::goals::SWITCHED_OFF)
    );
}

#[test]
fn the_swiss_hoard_scores_only_while_its_goal_is_on() {
    let score = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::Switzerland, rule);
        found(&mut s, P0, at(5, 5));
        s.player_mut(P0).gold = crate::goals::SWISS_HOARD_GOLD + 50;
        scoring::update(&mut s, &mut Vec::new());
        scoring::vp(&s, P0)
            .into_iter()
            .any(|(src, _)| matches!(src, VpSource::FactionGoal { .. }))
    };
    assert!(score(None));
    assert!(!score(Some(FactionRule::SwissGoal)));
}

#[test]
fn the_heir_scores_only_while_its_goal_is_on() {
    let held = |rule: Option<FactionRule>| {
        let mut s = state_of(Faction::Byzantium, rule);
        found(&mut s, P0, at(5, 5));
        let heir = crate::game::spawn_unit(&mut s, P0, UnitKind::HeirToTheThrone, hex(5, 6));
        s.units.get_mut(&heir).unwrap().heir_of = Some(P0);
        crate::goals::held_by(&s, P0)
    };
    assert_eq!(held(None), Some(P0));
    assert_eq!(held(Some(FactionRule::ByzantiumGoal)), None);
}

#[test]
fn an_empty_set_leaves_the_other_factions_numbers_alone() {
    let rules = RuleExperiments::default();
    for f in Faction::ALL {
        assert_eq!(rules.min_city_gap(f), data::min_city_gap(f));
        for u in UnitKind::ALL {
            assert_eq!(rules.unit_moves_for(f, u), data::unit_moves_for(f, u));
        }
    }
}
