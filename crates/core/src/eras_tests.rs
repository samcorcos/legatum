//! The era overhaul's rules (`crate::eras`): the tech table, the strict gate, era entry,
//! diffusion, legacy saves, Railways, the Castle and Star Fort tiers and the research forecast.
use crate::eras;
use crate::events::Event;
use crate::rules::cities::{
    apply_city_orders, process_research, validate_city_order, validate_city_orders,
};
use crate::rules::test_support::{city, game, hex, know_eras_before, unit};
use crate::{combat, data, economy, player_view, rules, *};
use std::collections::{BTreeMap, BTreeSet};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn set_research(tech: TechId) -> Order {
    Order::SetResearch { tech }
}

fn queue(techs: &[TechId]) -> Order {
    Order::SetResearchQueue {
        techs: techs.to_vec(),
    }
}

/// Gives `player` every technology except `left_out` (a tech they are about to learn).
fn know_all_but(s: &mut GameState, player: PlayerId, left_out: &[TechId]) {
    s.player_mut(player)
        .techs
        .extend(TechId::ALL.into_iter().filter(|t| !left_out.contains(t)));
}

/// Makes `player` research `tech` and puts its whole cost in the bank, so the next
/// `process_research` completes it with `science` income this turn.
fn about_to_finish(s: &mut GameState, player: PlayerId, tech: TechId) {
    let cost = economy::tech_cost(s, player, tech).total;
    let p = s.player_mut(player);
    p.researching = Some(tech);
    p.research_queue = vec![tech];
    p.science_stored = cost;
    p.science_history = vec![0];
}

// ---------- the table ----------

#[test]
fn the_table_has_nine_named_eras_and_every_era_has_at_least_five_techs() {
    assert_eq!(eras::ERA_COUNT, 9);
    assert_eq!(eras::ERA_NAMES.len(), 9);
    assert_eq!(
        eras::ERA_NAMES,
        [
            "Ancient",
            "Classical",
            "Medieval",
            "Renaissance",
            "Industrial",
            "Machine Age",
            "World War",
            "Modern",
            "Future"
        ]
    );
    assert_eq!(
        eras::ERA_NAMES.iter().collect::<BTreeSet<_>>().len(),
        9,
        "era names are unique"
    );
    assert_eq!(eras::LAST_ERA, 8);
    let mut total = 0;
    for era in 0..eras::ERA_COUNT as u8 {
        let n = eras::era_techs(era).count();
        assert!(n >= 5, "era {era} has {n} techs");
        total += n;
        assert_eq!(eras::era_name(era), eras::ERA_NAMES[usize::from(era)]);
    }
    assert_eq!(total, TechId::ALL.len());
    assert_eq!(eras::TECHS.len(), TechId::ALL.len());
    // An out-of-range era reads as the last one rather than panicking.
    assert_eq!(eras::era_name(200), "Future");
}

#[test]
fn techs_follow_tech_id_order_with_unique_names() {
    let mut names = BTreeSet::new();
    let mut previous_era = 0;
    for (i, def) in eras::TECHS.iter().enumerate() {
        assert_eq!(def.id, TechId::ALL[i], "TECHS[{i}] must follow TechId::ALL");
        assert_eq!(def.id as usize, i);
        assert!(std::ptr::eq(eras::tech_def(def.id), def));
        assert!(names.insert(def.name), "duplicate name {}", def.name);
        assert_eq!(data::tech_name(def.id), def.name);
        assert!(def.era >= previous_era, "eras run in order in the table");
        previous_era = def.era;
        assert_eq!(eras::era_of_tech(def.id), def.era);
        assert_eq!(data::tech_era(def.id), eras::era_name(def.era));
        assert_eq!(data::tech_stats(def.id).cost, def.cost);
        assert!(def.cost > 0);
    }
    assert!(
        TechId::ALL.windows(2).all(|w| w[0] < w[1]),
        "TechId derives its order from the declaration order"
    );
    // The era listing is the table filtered, in table order.
    for era in 0..eras::ERA_COUNT as u8 {
        let listed: Vec<_> = eras::era_techs(era).collect();
        let filtered: Vec<_> = TechId::ALL
            .into_iter()
            .filter(|t| eras::era_of_tech(*t) == era)
            .collect();
        assert_eq!(listed, filtered);
    }
}

#[test]
fn costs_rise_from_each_era_to_the_next() {
    for era in 1..eras::ERA_COUNT as u8 {
        let cheapest_here = eras::era_techs(era)
            .map(|t| eras::tech_def(t).cost)
            .min()
            .unwrap();
        let dearest_before = eras::era_techs(era - 1)
            .map(|t| eras::tech_def(t).cost)
            .max()
            .unwrap();
        assert!(
            cheapest_here > dearest_before,
            "era {era} must cost more than era {}",
            era - 1
        );
    }
}

// ---------- the current era ----------

#[test]
fn current_era_matches_table_scan_for_tech_subsets() {
    let old_definition = |known: &BTreeSet<TechId>| {
        (0..=eras::LAST_ERA)
            .find(|era| eras::era_techs(*era).any(|tech| !known.contains(&tech)))
            .unwrap_or(eras::LAST_ERA)
    };
    for (index, tech) in TechId::ALL.into_iter().enumerate() {
        assert_eq!(
            eras::TECHS[index].id,
            tech,
            "table must follow TechId order"
        );
        if index > 0 {
            assert!(eras::TECHS[index - 1].era <= eras::TECHS[index].era);
        }
    }
    let mut subsets = vec![BTreeSet::new(), TechId::ALL.into_iter().collect()];
    let mut seed = 0x5eed_1234_9876_abcd_u64;
    for _ in 0..2_048 {
        let mut known = BTreeSet::new();
        for tech in TechId::ALL {
            // Fixed integer RNG: reproducible subsets, including gaps before later known techs.
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            if seed & 1 != 0 {
                known.insert(tech);
            }
        }
        subsets.push(known);
    }
    for known in subsets {
        assert_eq!(
            eras::current_era(&known),
            old_definition(&known),
            "{known:?}"
        );
    }
}

#[test]
fn current_era_is_the_first_era_with_an_unknown_tech() {
    let mut known = BTreeSet::new();
    assert_eq!(eras::current_era(&known), 0);
    // Later-era techs do not move an empire out of an era it has not finished.
    known.extend(eras::era_techs(3));
    assert_eq!(eras::current_era(&known), 0);
    known.clear();
    for era in 0..eras::ERA_COUNT as u8 {
        let last = eras::era_techs(era).last().unwrap();
        // One short of finishing the era: still in it.
        known.extend(eras::era_techs(era).filter(|t| *t != last));
        assert_eq!(eras::current_era(&known), era);
        known.insert(last);
    }
    // Everything known: the last era, never past it.
    assert_eq!(eras::current_era(&known), eras::LAST_ERA);
    assert!(eras::all_researched(&known));
    known.remove(&TechId::RecursiveAi);
    assert_eq!(eras::current_era(&known), eras::LAST_ERA);
    assert!(!eras::all_researched(&known));
    // A gap in an early era pulls the empire back to it.
    known.insert(TechId::RecursiveAi);
    known.remove(&TechId::Masonry);
    assert_eq!(eras::current_era(&known), 0);
}

#[test]
fn era_of_reads_the_players_techs_and_barbarians_are_ancient() {
    let mut s = game(&[0, 1]);
    assert_eq!((s.era_of(P0), eras::era_of(&s, P0)), (0, 0));
    know_eras_before(&mut s, P0, 3);
    assert_eq!(s.era_of(P0), 3);
    assert_eq!(s.era_of(P1), 0, "per empire");
    know_all_but(&mut s, P1, &[]);
    assert_eq!(s.era_of(P1), eras::LAST_ERA);
    assert_eq!(s.era_of(PlayerId::BARBARIAN), 0);
}

#[test]
fn researchable_means_unknown_and_in_the_current_era() {
    let mut s = game(&[0]);
    know_eras_before(&mut s, P0, 2);
    s.player_mut(P0).techs.insert(TechId::Feudalism);
    let p = s.player(P0);
    assert!(!eras::researchable(p, TechId::Feudalism), "known");
    assert!(!eras::researchable(p, TechId::Masonry), "known and earlier");
    assert!(eras::researchable(p, TechId::Machinery));
    assert!(!eras::researchable(p, TechId::Gunpowder), "next era");
    for tech in TechId::ALL {
        assert_eq!(
            economy::tech_available(&s, P0, tech),
            eras::researchable(p, tech)
        );
    }
}

// ---------- the strict gate ----------

#[test]
fn the_gate_offers_exactly_the_unknown_techs_of_the_current_era() {
    for era in 0..eras::ERA_COUNT as u8 {
        let mut s = game(&[0]);
        know_eras_before(&mut s, P0, era);
        let available: Vec<TechId> = TechId::ALL
            .into_iter()
            .filter(|t| economy::tech_available(&s, P0, *t))
            .collect();
        assert_eq!(
            available,
            eras::era_techs(era).collect::<Vec<_>>(),
            "era {era}"
        );
        // There are no prerequisites inside an era: any one can be researched first.
        for tech in &available {
            assert!(validate_city_order(&s, P0, &set_research(*tech)).is_ok());
            assert!(validate_city_order(&s, P0, &queue(&[*tech])).is_ok());
        }
        // The whole era may be queued, in any order.
        let mut reversed = available.clone();
        reversed.reverse();
        assert!(validate_city_order(&s, P0, &queue(&reversed)).is_ok());
    }
}

#[test]
fn set_research_and_queue_reject_other_eras_known_techs_and_duplicates() {
    let mut s = game(&[0, 1]);
    know_eras_before(&mut s, P0, 1);
    s.player_mut(P0).techs.insert(TechId::Mathematics);
    // Later era, earlier era (known), known in the current era, duplicates, mixed plans.
    let bad: Vec<(Order, &str)> = vec![
        (
            set_research(TechId::Feudalism),
            "finish the Classical era first",
        ),
        (set_research(TechId::Masonry), "already researched"),
        (set_research(TechId::Mathematics), "already researched"),
        (
            queue(&[TechId::Feudalism]),
            "finish the Classical era first",
        ),
        (queue(&[TechId::Masonry]), "already researched"),
        (
            queue(&[TechId::Engineering, TechId::Engineering]),
            "more than once",
        ),
        (
            queue(&[TechId::Engineering, TechId::Feudalism]),
            "finish the Classical era first",
        ),
        (
            queue(&[TechId::Mathematics, TechId::Engineering]),
            "already researched",
        ),
    ];
    for (order, expect) in bad {
        let message = validate_city_order(&s, P0, &order).unwrap_err();
        assert!(message.contains(expect), "{order:?}: {message}");
        // An invalid order changes nothing and reports one failure.
        let before = s.clone();
        let mut events = Vec::new();
        apply_city_orders(&mut s, P0, std::slice::from_ref(&order), &mut events);
        assert_eq!(s, before);
        assert!(matches!(
            events.as_slice(),
            [Event::OrderFailed { player: P0, .. }]
        ));
    }
    // Valid: the rest of the Classical era, once each.
    let ok = queue(&[
        TechId::CodeOfLaws,
        TechId::Engineering,
        TechId::Philosophy,
        TechId::IronWorking,
    ]);
    assert!(validate_city_order(&s, P0, &ok).is_ok());
    apply_city_orders(&mut s, P0, &[ok], &mut Vec::new());
    assert_eq!(s.player(P0).researching, Some(TechId::CodeOfLaws));
    assert_eq!(
        s.player(P0).research_queue,
        [
            TechId::CodeOfLaws,
            TechId::Engineering,
            TechId::Philosophy,
            TechId::IronWorking
        ]
    );
    // A batch with one bad order fails only that order.
    let orders = [
        set_research(TechId::Gunpowder),
        set_research(TechId::Engineering),
    ];
    assert_eq!(validate_city_orders(&s, P0, &orders).len(), 1);
    // The same tech is fine for an empire that is further along.
    know_eras_before(&mut s, P1, 3);
    assert!(validate_city_order(&s, P1, &set_research(TechId::Gunpowder)).is_ok());
    assert!(validate_city_order(&s, P1, &set_research(TechId::Engineering)).is_err());
}

// ---------- entering an era ----------

#[test]
fn finishing_the_last_tech_of_an_era_enters_the_next_that_turn() {
    let mut s = game(&[0, 1]);
    // P0 knows the Ancient era except Writing.
    s.player_mut(P0).techs.extend([
        TechId::Irrigation,
        TechId::BronzeWorking,
        TechId::Masonry,
        TechId::Currency,
    ]);
    assert_eq!(s.era_of(P0), 0);
    about_to_finish(&mut s, P0, TechId::Writing);
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    // Writing was the last Ancient tech: P0 is in the Classical era this very turn.
    assert_eq!(s.era_of(P0), 1);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::EraEntered { .. }))
            .collect::<Vec<_>>(),
        [&Event::EraEntered {
            player: P0,
            era: 2,
            first: true
        }]
    );
    let finished = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::TechResearched {
                    tech: TechId::Writing,
                    ..
                }
            )
        })
        .unwrap();
    let entered = events
        .iter()
        .position(|e| matches!(e, Event::EraEntered { .. }))
        .unwrap();
    assert!(finished < entered, "the era starts after its last tech");
    assert_eq!(s.era_firsts.get(&1), Some(&P0));
    assert_eq!(s.player(P0).researching, None, "a new plan is needed");
    assert!(s.player(P0).research_queue.is_empty());
    // And only the new era's unknown techs are researchable now.
    assert!(economy::tech_available(&s, P0, TechId::Mathematics));
    assert!(!economy::tech_available(&s, P0, TechId::Feudalism));
}

#[test]
fn only_the_first_empire_into_an_era_is_first() {
    let mut s = game(&[0, 1]);
    for p in [P0, P1] {
        know_eras_before(&mut s, p, 1);
        s.player_mut(p).techs.remove(&TechId::Writing);
        s.player_mut(p).techs.remove(&TechId::Masonry);
    }
    // P1 gets there first, on an earlier turn.
    s.player_mut(P1).techs.insert(TechId::Masonry);
    about_to_finish(&mut s, P1, TechId::Writing);
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    assert!(events.contains(&Event::EraEntered {
        player: P1,
        era: 2,
        first: true
    }));
    assert_eq!(s.era_firsts.get(&1), Some(&P1));
    // P0 follows later and is not first, and the record does not change.
    s.player_mut(P0).techs.insert(TechId::Masonry);
    about_to_finish(&mut s, P0, TechId::Writing);
    s.turn += 5;
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    assert!(events.contains(&Event::EraEntered {
        player: P0,
        era: 2,
        first: false
    }));
    assert_eq!(s.era_firsts.get(&1), Some(&P1));
    assert_eq!(s.era_firsts.len(), 1);
}

#[test]
fn two_empires_finishing_the_same_turn_resolve_in_id_order() {
    let mut s = game(&[0, 1, 2]);
    for p in [P0, P1, P2] {
        know_eras_before(&mut s, p, 1);
        s.player_mut(p).techs.remove(&TechId::Writing);
    }
    // P2 and P1 finish together; P0 does not finish.
    about_to_finish(&mut s, P2, TechId::Writing);
    about_to_finish(&mut s, P1, TechId::Writing);
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    let entered: Vec<(PlayerId, bool)> = events
        .iter()
        .filter_map(|e| match e {
            Event::EraEntered {
                player,
                era: 2,
                first,
            } => Some((*player, *first)),
            _ => None,
        })
        .collect();
    assert_eq!(
        entered,
        [(P1, true), (P2, false)],
        "research runs in id order"
    );
    assert_eq!(s.era_firsts.get(&1), Some(&P1));
    assert_eq!(s.era_of(P0), 0);
}

#[test]
fn finishing_a_tech_that_does_not_end_the_era_enters_nothing() {
    let mut s = game(&[0, 1]);
    about_to_finish(&mut s, P0, TechId::Irrigation);
    let mut events = Vec::new();
    process_research(&mut s, &mut events);
    assert!(s.player(P0).techs.contains(&TechId::Irrigation));
    assert!(!events.iter().any(|e| matches!(e, Event::EraEntered { .. })));
    assert!(s.era_firsts.is_empty());
    assert_eq!(s.era_of(P0), 0);
}

#[test]
fn learning_the_last_future_tech_stays_in_the_last_era() {
    let mut s = game(&[0, 1]);
    know_all_but(&mut s, P0, &[TechId::RecursiveAi]);
    s.era_firsts.insert(6, P1);
    let mut events = Vec::new();
    eras::learn(&mut s, P0, TechId::RecursiveAi, &mut events);
    assert_eq!(
        events,
        [Event::TechResearched {
            player: P0,
            tech: TechId::RecursiveAi
        }]
    );
    assert_eq!(s.era_of(P0), eras::LAST_ERA);
    assert!(eras::all_researched(&s.player(P0).techs));
}

#[test]
fn one_tech_can_open_several_eras_at_once() {
    // An empire holding every later tech (a free pick, a legacy save) and then learning the one
    // missing Ancient tech passes through every era in turn.
    let mut s = game(&[0, 1]);
    know_all_but(&mut s, P0, &[TechId::Writing]);
    s.era_firsts.insert(2, P1);
    let mut events = Vec::new();
    eras::learn(&mut s, P0, TechId::Writing, &mut events);
    let entered: Vec<(u8, bool)> = events
        .iter()
        .filter_map(|e| match e {
            Event::EraEntered {
                player: P0,
                era,
                first,
            } => Some((*era, *first)),
            _ => None,
        })
        .collect();
    assert_eq!(
        entered,
        [
            (2, true),
            (3, false),
            (4, true),
            (5, true),
            (6, true),
            (7, true),
            (8, true),
            (9, true)
        ]
    );
    assert_eq!(s.era_firsts.get(&2), Some(&P1), "an earlier first stands");
    assert_eq!(s.era_firsts.get(&7), Some(&P0));
}

#[test]
fn every_seats_view_shows_each_empires_era_and_the_research_view_the_firsts() {
    let mut s = game(&[0, 1]);
    city(&mut s, 0, hex(2, 2), 3);
    city(&mut s, 1, hex(8, 6), 3);
    let eras_in = |s: &GameState, seat| {
        player_view(s, seat)
            .players
            .iter()
            .map(|p| p.era)
            .collect::<Vec<_>>()
    };
    assert_eq!(eras_in(&s, P0), [1, 1]);
    know_eras_before(&mut s, P1, 4);
    s.era_firsts.insert(4, P1);
    // Public: both seats see both empires' eras (`PlayerPublic.era` is 1-based, the research
    // view's `era` and `era_firsts` keys are 0-based).
    assert_eq!(eras_in(&s, P0), [1, 5]);
    assert_eq!(eras_in(&s, P1), [1, 5]);
    let v0 = player_view(&s, P0);
    assert_eq!(v0.research.era, 0);
    assert_eq!(v0.research.era_name, "Ancient");
    assert_eq!(v0.research.era_firsts, [(4, P1)]);
    let v1 = player_view(&s, P1);
    assert_eq!(
        (v1.research.era, v1.research.era_name.as_str()),
        (4, "Industrial")
    );
    assert_eq!((v1.research.era_total, v1.research.era_known), (5, 0));
    // The era survives the wire format.
    let json = serde_json::to_string(&v0.players[1]).unwrap();
    assert!(json.contains("\"era\":5"), "{json}");
}

#[test]
fn era_state_round_trips_through_a_save() {
    let mut s = game(&[0, 1]);
    know_eras_before(&mut s, P0, 2);
    s.era_firsts.insert(1, P0);
    s.era_firsts.insert(2, P0);
    let json = serde_json::to_string(&s).unwrap();
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded, s);
    assert_eq!(loaded.hash(), s.hash());
}

// ---------- diffusion ----------

#[test]
fn diffusion_is_ten_percent_per_other_empire_up_to_half() {
    assert_eq!(data::TECH_DIFFUSION_PER_EMPIRE, 10);
    assert_eq!(data::TECH_DIFFUSION_MAX, 50);
    let mut s = game(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let tech = TechId::Writing;
    let base = data::tech_stats(tech).cost;
    // Knowing it oneself discounts nothing; each other empire takes 10% off.
    s.player_mut(P0).techs.insert(tech);
    assert_eq!(economy::tech_cost(&s, P0, tech).total, base);
    for n in 1..=7u8 {
        s.player_mut(PlayerId(n)).techs.insert(tech);
        let percent = (10 * i32::from(n)).min(50);
        let quote = economy::tech_cost(&s, P0, tech);
        assert_eq!(quote.total, base * (100 - percent) / 100, "{n} others");
        assert_eq!(quote.parts[0], ("Listed cost".into(), base));
        assert!(quote.parts[1].0.contains(&format!("{n} empires know it")));
    }
    assert_eq!(economy::tech_known_by(&s, P0, tech), 7);
}

#[test]
fn diffusion_ignores_eliminated_empires() {
    let mut s = game(&[0, 1, 2]);
    let tech = TechId::Currency;
    let base = data::tech_stats(tech).cost;
    s.player_mut(P1).techs.insert(tech);
    s.player_mut(P2).techs.insert(tech);
    assert_eq!(economy::tech_cost(&s, P0, tech).total, base * 80 / 100);
    s.player_mut(P2).alive = false;
    assert_eq!(economy::tech_cost(&s, P0, tech).total, base * 90 / 100);
}

#[test]
fn byzantium_library_keeps_ordinary_diffusion() {
    let mut s = game(&[0, 1]);
    s.player_mut(P0).faction = Faction::Byzantium;
    let home = city(&mut s, 0, hex(2, 2), 3);
    let tech = TechId::Writing;
    s.player_mut(P1).techs.insert(tech);
    let before = economy::tech_cost(&s, P0, tech);
    s.cities.get_mut(&home).unwrap().buildings = [BuildingKind::Laboratory].into();
    let after = economy::tech_cost(&s, P0, tech);
    assert_eq!(before, after);
    assert_eq!(after.parts.len(), 2);
    assert_eq!(
        data::building_name_for(Faction::Byzantium, BuildingKind::Laboratory),
        "Library"
    );
}

#[test]
fn every_legacy_tech_name_loads_as_its_nearest_new_tech() {
    // (old name, the technology it loads as)
    let legacy: [(&str, TechId); 32] = [
        ("SoilReclamation", TechId::Irrigation),
        ("Fabrication", TechId::BronzeWorking),
        ("Fortification", TechId::Masonry),
        ("Exchange", TechId::Currency),
        ("SignalTheory", TechId::Writing),
        ("Ballistics", TechId::Feudalism),
        ("DeepSurvey", TechId::Philosophy),
        ("CivicNetworks", TechId::CodeOfLaws),
        ("RelayRestoration", TechId::Philosophy),
        ("SalvageProtocols", TechId::Irrigation),
        ("IntensiveCultivation", TechId::CropRotation),
        ("MachineTools", TechId::IronWorking),
        ("PublicAdministration", TechId::CodeOfLaws),
        ("DiplomaticProtocols", TechId::CodeOfLaws),
        ("FieldLogistics", TechId::Engineering),
        ("Academies", TechId::Education),
        ("CommercialLaw", TechId::Banking),
        ("ClosedHabitats", TechId::CropRotation),
        ("IndustrialAutomation", TechId::Industrialization),
        ("FederatedMarkets", TechId::Banking),
        ("ResearchNetworks", TechId::PrintingPress),
        ("MutualDefense", TechId::Feudalism),
        ("CombinedArms", TechId::Machinery),
        ("SiegeEngineering", TechId::Mathematics),
        ("RecoveredPower", TechId::Electricity),
        ("NutrientSynthesis", TechId::CropRotation),
        ("PrecisionIndustry", TechId::Industrialization),
        ("PredictiveComputation", TechId::Electricity),
        ("StrategicLogistics", TechId::MilitaryEngineering),
        ("PoweredArmor", TechId::Gunpowder),
        ("PlanetaryAccord", TechId::Banking),
        ("LegacyIntegration", TechId::PrintingPress),
    ];
    for (old, new) in legacy {
        assert_eq!(TechId::from_name(old), Some(new), "{old}");
        let parsed: TechId = serde_json::from_str(&format!("\"{old}\"")).unwrap();
        assert_eq!(parsed, new, "{old} through serde");
        // A mapped technology remains legal in recorded tapes.
        assert!(TechId::ALL.contains(&new));
    }
    assert_eq!(
        legacy
            .iter()
            .map(|(old, _)| *old)
            .collect::<BTreeSet<_>>()
            .len(),
        32,
        "the 32 retired names are all distinct"
    );
}

#[test]
fn current_tech_names_round_trip_and_unknown_names_are_rejected() {
    for tech in TechId::ALL {
        let json = serde_json::to_string(&tech).unwrap();
        let name = json.trim_matches('"');
        assert_eq!(TechId::from_name(name), Some(tech), "{name}");
        assert_eq!(serde_json::from_str::<TechId>(&json).unwrap(), tech);
    }
    // Cannonry is displayed "Ballistics", but the wire name "Ballistics" is the old Trooper tech.
    assert_eq!(data::tech_name(TechId::Cannonry), "Ballistics");
    assert_eq!(
        serde_json::to_string(&TechId::Cannonry).unwrap(),
        "\"Cannonry\""
    );
    assert_eq!(TechId::from_name("Ballistics"), Some(TechId::Feudalism));
    for bad in [
        "",
        "Nonexistent",
        "irrigation",
        "Irrigation ",
        "DELETED_SalvageProtocols",
    ] {
        assert_eq!(TechId::from_name(bad), None, "{bad:?}");
        let err = serde_json::from_str::<TechId>(&format!("\"{bad}\"")).unwrap_err();
        assert!(err.to_string().contains("unknown technology"), "{err}");
    }
    assert!(serde_json::from_str::<TechId>("7").is_err());
    // A whole save naming an unknown tech does not load.
    let s = game(&[0, 1]);
    let mut saved = serde_json::to_value(&s).unwrap();
    saved["players"][0]["techs"] = serde_json::json!(["Teleportation"]);
    assert!(serde_json::from_value::<GameState>(saved).is_err());
}

#[test]
fn legacy_building_names_load_as_their_replacements() {
    assert_eq!(
        serde_json::from_str::<BuildingKind>("\"Citadel\"").unwrap(),
        BuildingKind::Castle
    );
    assert_eq!(
        serde_json::from_str::<BuildingKind>("\"Arcology\"").unwrap(),
        BuildingKind::Hydroponics
    );
    assert_eq!(
        serde_json::from_str::<BuildingKind>("\"Monument\"").unwrap(),
        BuildingKind::BroodHall
    );
    assert!(serde_json::from_str::<BuildingKind>("\"Ziggurat\"").is_err());
    // New saves write the new names.
    assert_eq!(
        serde_json::to_string(&BuildingKind::Castle).unwrap(),
        "\"Castle\""
    );
    assert_eq!(
        serde_json::to_string(&BuildingKind::StarFort).unwrap(),
        "\"StarFort\""
    );
    for kind in BuildingKind::ALL {
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(serde_json::from_str::<BuildingKind>(&json).unwrap(), kind);
    }
    // Inside a city of an old save.
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 4);
    let mut saved = serde_json::to_value(&s).unwrap();
    saved["cities"][id.0.to_string()]["buildings"] = serde_json::json!(["Citadel", "Arcology"]);
    let loaded: GameState = serde_json::from_value(saved).unwrap();
    assert_eq!(
        loaded.cities[&id].buildings,
        BTreeSet::from([BuildingKind::Castle, BuildingKind::Hydroponics])
    );
    assert_eq!(data::fort_tier(&loaded.cities[&id].buildings), 1);
}

#[test]
fn buildings_have_historical_display_names() {
    for (kind, name) in [
        (BuildingKind::Laboratory, "Library"),
        (BuildingKind::College, "University"),
        (BuildingKind::Clearinghouse, "Bank"),
        (BuildingKind::Hydroponics, "Mill"),
        (BuildingKind::MachineShop, "Forge"),
        (BuildingKind::Autoworks, "Factory"),
        (BuildingKind::InferenceCore, "Research Lab"),
        (BuildingKind::RelayUplink, "Great Library"),
        (BuildingKind::PlanetaryFoundry, "Royal Arsenal"),
        (BuildingKind::WorldExchange, "Grand Bazaar"),
        (BuildingKind::Castle, "Castle"),
        (BuildingKind::StarFort, "Star Fort"),
    ] {
        assert_eq!(data::building_name(kind), name);
    }
    let names: BTreeSet<_> = BuildingKind::ALL
        .iter()
        .map(|k| data::building_name(*k))
        .collect();
    assert_eq!(
        names.len(),
        BuildingKind::ALL.len(),
        "display names are unique"
    );
}

// ---------- what techs unlock ----------

#[test]
fn the_buildings_and_their_unlock_techs_follow_the_new_tree() {
    use BuildingKind as B;
    for (building, tech) in [
        (B::Workshop, TechId::BronzeWorking),
        (B::Market, TechId::Currency),
        (B::Laboratory, TechId::Writing),
        (B::Walls, TechId::Masonry),
        (B::Roadworks, TechId::Engineering),
        (B::MachineShop, TechId::IronWorking),
        (B::RelayUplink, TechId::Philosophy),
        (B::HagiaSophia, TechId::Machinery),
        (B::Castle, TechId::Chivalry),
        (B::WarShrine, TechId::Chivalry),
        (B::College, TechId::Education),
        (B::Hydroponics, TechId::Feudalism),
        (B::StarFort, TechId::Machinery),
        (B::Clearinghouse, TechId::Banking),
        (B::WorldExchange, TechId::Banking),
        (B::Autoworks, TechId::Industrialization),
        (B::PlanetaryFoundry, TechId::Cannonry),
        (B::StockExchange, TechId::Electricity),
        (B::InferenceCore, TechId::Electronics),
    ] {
        assert_eq!(
            data::building_stats(building).requires,
            Some(tech),
            "{building:?}"
        );
    }
    assert_eq!(data::building_stats(B::Granary).requires, None);
    for (unit, tech) in [
        (UnitKind::Trooper, TechId::Feudalism),
        (UnitKind::Marksman, TechId::Machinery),
        (UnitKind::AssaultTrooper, TechId::Gunpowder),
        (UnitKind::TrenchInfantry, TechId::ReplaceableParts),
        (UnitKind::Landship, TechId::Combustion),
        (UnitKind::Howitzer, TechId::Steel),
        (UnitKind::Biplane, TechId::Flight),
        (UnitKind::Infantry, TechId::MassMobilisation),
        (UnitKind::Artillery, TechId::Rocketry),
        (UnitKind::Tank, TechId::ArmouredWarfare),
        (UnitKind::Fighter, TechId::AirPower),
        (UnitKind::Bomber, TechId::AirPower),
    ] {
        assert_eq!(data::unit_stats(unit).requires, Some(tech), "{unit:?}");
    }
}

#[test]
fn printing_press_and_the_future_techs_add_their_city_effects() {
    let mut s = game(&[0, 1]);
    s.player_mut(P0).faction = Faction::Switzerland;
    let id = city(&mut s, 0, hex(3, 3), 4);
    let science = |s: &GameState| economy::city_yields(s, &s.cities[&id]).total.science;
    let production = |s: &GameState| economy::city_yields(s, &s.cities[&id]).total.production;
    s.cities.get_mut(&id).unwrap().buildings = [BuildingKind::Laboratory].into();
    let base = science(&s);
    // Printing Press adds Science to the Library family; Recursive AI boosts Production.
    s.player_mut(P0).techs.insert(TechId::PrintingPress);
    assert_eq!(science(&s), base + 1);
    s.player_mut(P0).techs.insert(TechId::RecursiveAi);
    assert_eq!(science(&s), base + 1);
    // The Printing Press bonus rides with the Library family through a University.
    s.cities.get_mut(&id).unwrap().buildings = [BuildingKind::College].into();
    let with_college = science(&s);
    s.player_mut(P0).techs.remove(&TechId::PrintingPress);
    s.player_mut(P0).techs.remove(&TechId::RecursiveAi);
    assert_eq!(with_college, science(&s) + 1);
    // Stealth Technology unlocks aircraft, with no Science bonus.
    s.cities.get_mut(&id).unwrap().buildings = [BuildingKind::InferenceCore].into();
    let lab = science(&s);
    s.player_mut(P0).techs.insert(TechId::StealthTechnology);
    assert_eq!(science(&s), lab);
    // Fusion Power helps every city; Recursive AI adds 20% after its flat bonus.
    s.cities.get_mut(&id).unwrap().buildings.clear();
    let bare = (science(&s), production(&s));
    s.player_mut(P0).techs.extend([
        TechId::FusionPower,
        TechId::StealthTechnology,
        TechId::RecursiveAi,
    ]);
    assert_eq!(
        (science(&s), production(&s)),
        (bare.0, bare.1 + 5 + (bare.1 + 5) * 20 / 100)
    );
}

#[test]
fn future_names_load_from_old_saves() {
    assert_eq!(
        TechId::from_name("QuantumComputing"),
        Some(TechId::StealthTechnology)
    );
    assert_eq!(
        serde_json::from_str::<UnitKind>("\"HumanoidRobot\"").unwrap(),
        UnitKind::HunterKiller
    );
}

#[test]
fn recursive_ai_reveals_public_map_but_not_enemy_units() {
    let mut s = game(&[0, 1]);
    let enemy = city(&mut s, 1, hex(8, 6), 3);
    let i = s.map.index(s.cities[&enemy].pos).unwrap();
    assert!(!s.player(P0).explored.get(i));
    eras::learn(&mut s, P0, TechId::RecursiveAi, &mut Vec::new());
    assert!(s.player(P0).explored.get(i));
    assert_eq!(
        s.player(P0).tile_memory[i]
            .as_ref()
            .unwrap()
            .city
            .as_ref()
            .unwrap()
            .id,
        enemy
    );
    assert!(!crate::visibility::visible_tiles(&s, s.team_of(P0)).get(i));
}

#[test]
fn earlier_tech_effects_change_city_yields_and_unit_costs() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 3);
    let yields = |s: &GameState| economy::city_yields(s, &s.cities[&id]).total;
    let before = yields(&s);
    s.player_mut(P0).techs.insert(TechId::Philosophy);
    assert_eq!(yields(&s).science, before.science + 1);
    s.cities.get_mut(&id).unwrap().buildings = [
        BuildingKind::Granary,
        BuildingKind::Workshop,
        BuildingKind::Market,
    ]
    .into();
    let before = yields(&s);
    s.player_mut(P0).techs.extend([
        TechId::CropRotation,
        TechId::Machinery,
        TechId::PrintingPress,
    ]);
    let after = yields(&s);
    assert_eq!(after.food, before.food + 1);
    assert_eq!(after.production, before.production + 1);
    assert_eq!(after.gold, before.gold + 1);
    s.player_mut(P0).techs.insert(TechId::FusionPower);
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Autoworks);
    let factory = yields(&s).production;
    s.player_mut(P0).techs.insert(TechId::ReplaceableParts);
    assert_eq!(yields(&s).production, factory + 2);
    let soldier = ProductionItem::Unit {
        unit: UnitKind::Soldier,
    };
    let cost = economy::production_cost(&s, P0, soldier);
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    let civilian_cost = economy::production_cost(&s, P0, settler);
    s.player_mut(P0).techs.insert(TechId::MilitaryScience);
    assert_eq!(economy::production_cost(&s, P0, soldier), cost * 90 / 100);
    assert_eq!(
        economy::production_cost(&s, P0, settler),
        civilian_cost * 90 / 100
    );
    let fighter = ProductionItem::Unit {
        unit: UnitKind::Fighter,
    };
    let air_cost = economy::production_cost(&s, P0, fighter);
    s.player_mut(P0).techs.remove(&TechId::MilitaryScience);
    assert_eq!(economy::production_cost(&s, P0, fighter), air_cost);
}

#[test]
fn improvement_steps_follow_their_techs() {
    let knows = |known: &[TechId], resource| {
        crate::improvements::resource_yields(resource, |tech| known.contains(&tech))
    };
    assert_eq!(knows(&[], Resource::Stone).production, 1);
    assert_eq!(knows(&[TechId::Masonry], Resource::Stone).production, 2);
    assert_eq!(knows(&[TechId::IronWorking], Resource::Stone).production, 1);
    let iron = [TechId::IronWorking];
    assert_eq!(
        knows(&iron, Resource::Iron).production + 1,
        knows(&[TechId::IronWorking, TechId::Railways], Resource::Iron).production
    );
    // Combustion reveals Oil and upgrades its well in one step: hidden, then the +1 Gold step.
    assert_eq!(knows(&[], Resource::Oil), crate::Yields::ZERO);
    assert_eq!(
        knows(&[TechId::Combustion], Resource::Oil).gold,
        crate::resources::stats(Resource::Oil).bonus.gold + 1
    );
}

#[test]
fn no_world_wonder_shares_a_resource_reveal() {
    let reveals: BTreeSet<_> = Resource::ALL
        .into_iter()
        .filter_map(|r| crate::resources::stats(r).revealed_by)
        .collect();
    for wonder in data::GREAT_WORKS {
        let tech = data::building_stats(wonder).requires.unwrap();
        assert!(
            !reveals.contains(&tech),
            "{wonder:?} shares {tech:?} with a resource reveal"
        );
    }
}

#[test]
fn completed_research_converts_science_at_each_chosen_destination() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 3);
    s.player_mut(P0).techs.extend(TechId::ALL);
    let base = economy::city_yields(&s, &s.cities[&id]).total;
    let gold = economy::player_income(&s, P0).gold;
    assert!(gold.parts.iter().any(|(label, amount)| {
        label.starts_with("Research after the last technology") && *amount == base.science / 2
    }));
    s.player_mut(P0).science_conversion = ScienceConversion::Food;
    assert_eq!(
        economy::city_yields(&s, &s.cities[&id]).total.food,
        base.food + base.science / 2
    );
    assert!(
        !economy::player_income(&s, P0)
            .gold
            .parts
            .iter()
            .any(|(label, _)| label.starts_with("Research after the last technology"))
    );
    s.player_mut(P0).science_conversion = ScienceConversion::Production;
    assert_eq!(
        economy::city_yields(&s, &s.cities[&id]).total.production,
        base.production + base.science / 2
    );
    let order = Order::SetScienceConversion {
        to: ScienceConversion::Gold,
    };
    assert!(validate_city_order(&s, P0, &order).is_ok());
}

#[test]
fn cached_city_view_changes_when_science_conversion_changes() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 3);
    s.player_mut(P0).techs.extend(TechId::ALL);
    let mut cache = crate::yields_cache::ViewCache::default();
    let city_yields = |s: &GameState, cache: &mut crate::yields_cache::ViewCache| {
        let public = crate::view::public_view(s);
        crate::view::player_view_cached(s, P0, &public, crate::view::ViewDetail::Full, cache)
            .cities
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .yields
            .total
    };
    let gold = city_yields(&s, &mut cache);
    s.player_mut(P0).science_conversion = ScienceConversion::Production;
    let production = city_yields(&s, &mut cache);
    assert_eq!(
        production,
        city_yields(&s, &mut crate::yields_cache::ViewCache::default())
    );
    assert!(production.production > gold.production);
    s.player_mut(P0).science_conversion = ScienceConversion::Food;
    let food = city_yields(&s, &mut cache);
    assert_eq!(
        food,
        city_yields(&s, &mut crate::yields_cache::ViewCache::default())
    );
    assert!(food.food > gold.food);
    s.player_mut(P0).science_conversion = ScienceConversion::Gold;
    assert_eq!(city_yields(&s, &mut cache), gold);
}

#[test]
fn irrigation_gives_a_food() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 3);
    let food = |s: &GameState| economy::city_yields(s, &s.cities[&id]).total.food;
    let before = food(&s);
    s.player_mut(P0).techs.insert(TechId::Irrigation);
    assert_eq!(food(&s), before + data::IRRIGATION_FOOD);
}

// ---------- Railways ----------

#[test]
fn a_unit_starting_on_a_road_gets_one_extra_move_and_two_with_railways() {
    let mut s = game(&[0, 1]);
    let on_road = unit(&mut s, 0, UnitKind::Soldier, hex(3, 3));
    let off_road = unit(&mut s, 0, UnitKind::Soldier, hex(5, 3));
    let rival = unit(&mut s, 1, UnitKind::Soldier, hex(3, 3));
    s.map.get_mut(hex(3, 3)).unwrap().road = crate::roads::NODE;
    let moves = data::unit_stats_for(Faction::Switzerland, UnitKind::Soldier).moves;
    let reset = |s: &GameState| {
        let mut s = s.clone();
        rules::units::reset_turn(&mut s);
        (
            s.units[&on_road].moves_left,
            s.units[&off_road].moves_left,
            s.units[&rival].moves_left,
        )
    };
    assert_eq!(data::ROAD_MOVES, 1);
    assert_eq!(data::RAILWAY_MOVES, 2);
    assert_eq!(reset(&s), (moves + 1, moves, moves + 1));
    // Railways is per empire: P0 gets +2 on a road, the rival still +1.
    s.player_mut(P0).techs.insert(TechId::Railways);
    assert_eq!(reset(&s), (moves + 2, moves, moves + 1));
    // Off the road, Railways add nothing.
    s.units.get_mut(&off_road).unwrap().pos = hex(3, 3);
    assert_eq!(reset(&s).1, moves + 2);
    s.units.get_mut(&off_road).unwrap().pos = hex(6, 6);
    assert_eq!(reset(&s).1, moves);
}

// ---------- Castle and Star Fort ----------

#[test]
fn steel_guns_fire_over_terrain_only_at_seen_targets() {
    let mut s = game(&[0, 1]);
    let from = hex(3, 3);
    let target = hex(5, 3);
    s.map.get_mut(hex(4, 3)).unwrap().terrain = Terrain::Forest;
    let gun = unit(&mut s, 0, UnitKind::Artillery, from);
    unit(&mut s, 1, UnitKind::Soldier, target);
    let spotter = unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
    let shot = |s: &GameState| rules::battle::ranged_target(s, P0, gun, target, &mut None);
    assert!(shot(&s).unwrap_err().contains("No line of sight"));
    s.player_mut(P0).techs.insert(TechId::Steel);
    assert!(shot(&s).is_ok());
    assert!(
        crate::forecast_attack_mode(&player_view(&s, P0), gun, target, AttackMode::Ranged)
            .is_some()
    );
    s.units.remove(&spotter);
    assert!(shot(&s).unwrap_err().contains("not visible"));
}

#[test]
fn walled_garrison_shoots_through_forest_but_unwalled_garrison_and_field_unit_do_not() {
    let mut s = game(&[0, 1]);
    let from = hex(3, 3);
    let target = hex(5, 3);
    s.map.get_mut(hex(4, 3)).unwrap().terrain = Terrain::Forest;
    let home = city(&mut s, 0, from, 3);
    let archer = unit(&mut s, 0, UnitKind::Ranger, from);
    unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
    unit(&mut s, 1, UnitKind::Soldier, target);
    let shot = |s: &GameState| rules::battle::ranged_target(s, P0, archer, target, &mut None);
    let forecast = |s: &GameState| {
        crate::forecast_attack_mode(&player_view(s, P0), archer, target, AttackMode::Ranged)
    };
    let error = shot(&s).unwrap_err();
    assert!(error.contains("No line of sight"), "{error}");
    assert!(forecast(&s).is_none());
    for tier in [
        BuildingKind::Walls,
        BuildingKind::Castle,
        BuildingKind::StarFort,
    ] {
        s.cities.get_mut(&home).unwrap().buildings = [tier].into();
        assert!(shot(&s).is_ok(), "{tier:?}");
        assert!(forecast(&s).is_some(), "{tier:?}");
    }
    s.units.get_mut(&archer).unwrap().pos = hex(4, 4);
    s.units
        .values_mut()
        .find(|u| u.kind == UnitKind::Scout)
        .unwrap()
        .pos = hex(6, 3);
    s.map.get_mut(hex(4, 4)).unwrap().terrain = Terrain::Hills;
    s.map.get_mut(hex(4, 3)).unwrap().terrain = Terrain::Mountains;
    s.map.get_mut(hex(5, 4)).unwrap().terrain = Terrain::Mountains;
    let error = shot(&s).unwrap_err();
    assert!(error.contains("No line of sight"), "{error}");
    assert!(forecast(&s).is_none());
}

#[test]
fn artillery_and_drones_keep_full_builtin_siege() {
    let mut s = game(&[0, 1]);
    let target = hex(5, 3);
    let c = city(&mut s, 1, target, 4);
    s.cities.get_mut(&c).unwrap().buildings.extend([
        BuildingKind::Walls,
        BuildingKind::Castle,
        BuildingKind::StarFort,
    ]);
    s.player_mut(P0).techs.insert(TechId::Steel);
    for kind in [UnitKind::Artillery, UnitKind::Drone] {
        let id = unit(&mut s, 0, kind, hex(3, 3));
        let f = crate::forecast_attack_mode(&player_view(&s, P0), id, target, AttackMode::Ranged)
            .unwrap();
        let walls = f.walls.as_ref().unwrap();
        assert_eq!((walls.covered, walls.percent), (3, 100));
        assert_eq!(rules::battle::siege_cover(&s, &s.units[&id]), 3);
        assert!(f.city_strike.is_some());
        s.units.remove(&id);
    }
}

#[test]
fn castles_and_star_forts_add_their_strength_cumulatively() {
    let faction = Faction::Switzerland;
    let castle = data::building_stats(BuildingKind::Castle).city_strength;
    let star = data::building_stats(BuildingKind::StarFort).city_strength;
    assert!(castle > 0 && star > 0);
    let at = |walls: bool, fort: u8| {
        combat::city_defense_with_fort(faction, 3, walls, fort, data::CITY_HP, 0).total
    };
    let none = at(false, 0);
    let walls = at(true, 0);
    assert!(walls > none);
    assert_eq!(at(true, 1), walls + castle, "a Castle adds to Walls");
    assert_eq!(
        at(true, 2),
        walls + castle + star,
        "a Star Fort adds to both"
    );
    // The breakdown names each step.
    let b = combat::city_defense_with_fort(faction, 3, true, 2, data::CITY_HP, 0);
    let labels: Vec<&str> = b.parts.iter().map(|p| p.0.as_str()).collect();
    assert!(labels.contains(&"Castle upgrade") && labels.contains(&"Star Fort upgrade"));
    assert_eq!(b.parts.iter().map(|p| p.1).sum::<i32>(), b.total);
    // The rules wrapper agrees with the default rules.
    assert_eq!(
        combat::city_defense_for_rules(Default::default(), faction, 3, true, 2, data::CITY_HP, 0),
        b
    );
    // Wounded cities scale all of it down.
    assert!(
        combat::city_defense_with_fort(faction, 3, true, 2, data::CITY_HP / 2, 0).total < b.total
    );
}

#[test]
fn city_defense_in_a_real_city_uses_its_fort_tier() {
    let mut s = game(&[0, 1]);
    let id = city(&mut s, 1, hex(5, 3), 4);
    let strength = |s: &GameState| {
        let v = player_view(s, P0);
        v.cities
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.strength, c.fort_tier(), c.citadel, c.star_fort))
    };
    unit(&mut s, 0, UnitKind::Soldier, hex(4, 3));
    crate::visibility::update_explored(&mut s);
    let (plain, tier, castle_flag, star_flag) = strength(&s).unwrap();
    assert_eq!((tier, castle_flag, star_flag), (0, false, false));
    s.cities.get_mut(&id).unwrap().buildings = [BuildingKind::Walls].into();
    let (walls, ..) = strength(&s).unwrap();
    assert!(walls > plain);
    s.cities.get_mut(&id).unwrap().buildings = [BuildingKind::Castle].into();
    let (castle, tier, castle_flag, star_flag) = strength(&s).unwrap();
    assert_eq!((tier, castle_flag, star_flag), (1, true, false));
    assert_eq!(
        castle,
        walls + data::building_stats(BuildingKind::Castle).city_strength
    );
    s.cities.get_mut(&id).unwrap().buildings = [BuildingKind::StarFort].into();
    let (star, tier, castle_flag, star_flag) = strength(&s).unwrap();
    assert_eq!((tier, castle_flag, star_flag), (2, true, true));
    assert_eq!(
        star,
        castle + data::building_stats(BuildingKind::StarFort).city_strength
    );
    // The memory of a seen city keeps the tier too.
    let memory = crate::state::CityMemory::from(&s.cities[&id]);
    assert_eq!(memory.fort_tier(), 2);
    assert!(memory.walls && memory.citadel && memory.star_fort);
}

#[test]
fn a_star_fort_needs_a_castle_which_needs_walls() {
    use BuildingKind::{Castle, StarFort, Walls};
    assert_eq!(data::building_predecessor(Castle), Some(Walls));
    assert_eq!(data::building_predecessor(StarFort), Some(Castle));
    assert_eq!(data::building_predecessor(Walls), None);
    for kind in [Walls, Castle, StarFort] {
        assert_eq!(data::building_family(kind), Walls);
    }
    // A higher tier satisfies the lower ones' requirements, not the other way round.
    assert!(data::building_satisfies(StarFort, Castle));
    assert!(data::building_satisfies(StarFort, Walls));
    assert!(data::building_satisfies(Castle, Walls));
    assert!(!data::building_satisfies(Walls, Castle));
    assert!(!data::building_satisfies(Castle, StarFort));
    assert_eq!(data::fort_tier(&BTreeSet::from([Walls])), 0);
    assert_eq!(data::fort_tier(&BTreeSet::from([Castle])), 1);
    assert_eq!(data::fort_tier(&BTreeSet::from([StarFort])), 2);
    assert_eq!(data::fort_tier(&BTreeSet::new()), 0);

    let mut s = game(&[0, 1]);
    let id = city(&mut s, 0, hex(3, 3), 3);
    s.player_mut(P0)
        .techs
        .extend([TechId::Masonry, TechId::Chivalry, TechId::Machinery]);
    let can = |s: &GameState, b| {
        economy::can_produce(s, &s.cities[&id], ProductionItem::Building { building: b })
    };
    let blocker = |s: &GameState, b| {
        economy::production_blocker(
            s,
            &s.cities[&id],
            ProductionItem::Building { building: b },
            true,
        )
        .unwrap_or_default()
    };
    assert!(can(&s, Walls));
    assert!(!can(&s, Castle));
    assert!(!can(&s, StarFort));
    let walls_name = data::building_name_for(Faction::Switzerland, Walls);
    assert_eq!(blocker(&s, Castle), format!("Requires {walls_name}"));
    assert_eq!(blocker(&s, StarFort), "Requires Castle");
    economy::install_building(s.cities.get_mut(&id).unwrap(), Walls);
    assert!(!can(&s, Walls), "already built");
    assert!(can(&s, Castle));
    assert!(
        !can(&s, StarFort),
        "a Star Fort needs the Castle, not just Walls"
    );
    economy::install_building(s.cities.get_mut(&id).unwrap(), Castle);
    assert_eq!(
        s.cities[&id].buildings.iter().copied().collect::<Vec<_>>(),
        [Castle],
        "each tier replaces the one below"
    );
    assert!(!can(&s, Walls) && !can(&s, Castle));
    assert!(can(&s, StarFort));
    // Techs gate the tiers.
    s.player_mut(P0).techs.remove(&TechId::Machinery);
    assert!(!can(&s, StarFort));
    assert_eq!(blocker(&s, StarFort), "Requires Machinery");
    s.player_mut(P0).techs.remove(&TechId::Chivalry);
    s.cities.get_mut(&id).unwrap().buildings = [Walls].into();
    assert!(!can(&s, Castle));
    assert_eq!(blocker(&s, Castle), "Requires Chivalry");
}

#[test]
fn mongols_raises_no_walls_tier_before_the_medieval_era_and_every_tier_after() {
    // The Mongols may raise Walls from the Medieval era.
    use BuildingKind::{Castle, StarFort, Walls};
    let mut s = game(&[0, 1]);
    s.player_mut(P0).faction = Faction::Mongols;
    s.player_mut(P1).faction = Faction::India;
    let id = city(&mut s, 0, hex(3, 3), 3);
    let other = city(&mut s, 1, hex(8, 6), 3);
    s.player_mut(P1).techs.extend(TechId::ALL);
    for building in [Walls, Castle, StarFort] {
        let item = ProductionItem::Building { building };
        assert!(
            data::available_to_faction(Faction::Mongols, item),
            "{building:?}"
        );
        assert!(
            !data::available_to_player(Faction::Mongols, 1, item),
            "{building:?}"
        );
        assert!(data::available_to_player(
            Faction::Mongols,
            data::MONGOL_WALLS_ERA,
            item
        ));
        assert_eq!(
            data::faction_excluded_building_at(Faction::Mongols, building, 1),
            Some("The Mongols raise Walls from the Medieval era")
        );
        assert_eq!(
            data::faction_excluded_building_at(Faction::Mongols, building, 2),
            None
        );
        assert_eq!(
            data::player_unavailable_reason(item, Faction::Mongols, 1).as_deref(),
            Some("The Mongols raise Walls from the Medieval era")
        );
    }
    // In a game: with every Ancient and Classical technology and the Walls techs it is the
    // Medieval era and the line opens; with fewer it stays shut, whatever else is known.
    let walls_techs = [TechId::Masonry, TechId::Feudalism, TechId::Machinery];
    s.player_mut(P0).techs.extend(walls_techs);
    assert!(s.era_of(P0) < 2);
    for building in [Walls, Castle, StarFort] {
        let item = ProductionItem::Building { building };
        assert!(
            !economy::can_produce(&s, &s.cities[&id], item),
            "{building:?}"
        );
        assert!(validate_city_order(&s, P0, &Order::SetProduction { city: id, item }).is_err());
    }
    let earlier: Vec<TechId> = (0..2).flat_map(crate::eras::era_techs).collect();
    s.player_mut(P0).techs.extend(earlier);
    assert_eq!(s.era_of(P0), 2);
    s.cities.get_mut(&id).unwrap().buildings.insert(Walls);
    s.cities.get_mut(&id).unwrap().buildings.insert(Castle);
    for building in [Walls, Castle, StarFort] {
        let item = ProductionItem::Building { building };
        // Already built tiers are not offered again; the rest are.
        let built = s.cities[&id].buildings.contains(&building);
        assert_eq!(
            economy::can_produce(&s, &s.cities[&id], item),
            !built,
            "{building:?}"
        );
    }
    assert!(economy::can_produce(
        &s,
        &s.cities[&other],
        ProductionItem::Building { building: Walls }
    ));
}

// ---------- the research forecast ----------

#[test]
fn the_schedule_forecasts_cumulative_arrival_in_queue_order() {
    let mut s = game(&[0, 1]);
    city(&mut s, 0, hex(2, 2), 4);
    city(&mut s, 1, hex(8, 6), 3);
    let science = player_view(&s, P0).economy.science_per_turn.total;
    assert!(science > 0);
    let order = [TechId::Writing, TechId::Irrigation, TechId::Currency];
    {
        let p = s.player_mut(P0);
        p.researching = Some(order[0]);
        p.research_queue = order.to_vec();
        p.science_stored = 7;
    }
    s.player_mut(P1).techs.insert(TechId::Irrigation); // diffusion shows in the slot's cost
    let v = player_view(&s, P0);
    let slots = &v.research.schedule;
    assert_eq!(
        slots.iter().map(|slot| slot.tech).collect::<Vec<_>>(),
        order,
        "queue order, not cost order"
    );
    let mut cumulative = 0;
    for slot in slots {
        let cost = economy::tech_cost(&s, P0, slot.tech).total;
        assert_eq!(slot.cost, cost);
        cumulative += cost;
        let turns = ((cumulative - 7 + science - 1) / science).max(1);
        assert_eq!(slot.turns, Some(turns), "{:?}", slot.tech);
        assert_eq!(slot.arrival_turn, Some(s.turn + turns as u32));
    }
    assert!(
        slots.windows(2).all(|w| w[0].turns <= w[1].turns),
        "later in the queue never arrives sooner"
    );
    assert!(slots[1].cost < data::tech_stats(TechId::Irrigation).cost);
    // The headline numbers describe the head of the queue.
    assert_eq!(v.research.current, Some(order[0]));
    assert_eq!(v.research.turns_left, slots[0].turns);
    assert_eq!(v.research.queue, order);
    // An empty plan has an empty schedule.
    s.player_mut(P0).researching = None;
    s.player_mut(P0).research_queue.clear();
    assert!(player_view(&s, P0).research.schedule.is_empty());
}

#[test]
fn the_schedule_has_no_dates_without_science() {
    let mut s = game(&[0, 1]);
    city(&mut s, 1, hex(8, 6), 3);
    // P0 has no city, so no Science.
    {
        let p = s.player_mut(P0);
        p.researching = Some(TechId::Masonry);
        p.research_queue = vec![TechId::Masonry, TechId::Writing];
    }
    let v = player_view(&s, P0);
    assert_eq!(v.economy.science_per_turn.total, 0);
    assert!(
        v.research
            .schedule
            .iter()
            .all(|slot| slot.turns.is_none() && slot.arrival_turn.is_none())
    );
    assert_eq!(v.research.schedule.len(), 2);
    // Science already banked covers a slot even without income: it resolves next turn.
    s.player_mut(P0).science_stored = 1000;
    let v = player_view(&s, P0);
    assert_eq!(v.research.schedule[0].turns, Some(1));
}

// ---------- unit hooks ----------

#[test]
fn units_have_a_class_and_an_era_tier_for_the_coming_lineage() {
    use UnitKind::*;
    for (kind, class) in [
        (Soldier, Some(UnitClass::Melee)),
        (Trooper, Some(UnitClass::Melee)),
        (AssaultTrooper, Some(UnitClass::Melee)),
        (Ranger, Some(UnitClass::Ranged)),
        (Marksman, Some(UnitClass::Ranged)),
        (Scout, None),
        (Settler, None),
        (Heir, None),
    ] {
        assert_eq!(data::unit_class(kind), class, "{kind:?}");
    }
    // The tier is the era of the unlocking tech, Ancient when nothing is needed.
    for kind in UnitKind::ALL {
        let expected = data::unit_stats(kind).requires.map_or(0, eras::era_of_tech);
        assert_eq!(data::unit_era_tier(kind), expected, "{kind:?}");
    }
    assert_eq!(data::unit_era_tier(Soldier), 0);
    assert_eq!(data::unit_era_tier(Trooper), 2);
    assert_eq!(data::unit_era_tier(Marksman), 2);
    assert_eq!(data::unit_era_tier(AssaultTrooper), 3);
    assert!(
        UnitKind::ALL
            .into_iter()
            .all(|k| data::unit_era_tier(k) < eras::ERA_COUNT as u8)
    );
}

// ---------- the order of research with a real turn ----------

#[test]
fn a_full_turn_learns_the_tech_enters_the_era_and_keeps_the_event_order() {
    let mut s = game(&[0, 1]);
    city(&mut s, 0, hex(2, 2), 3);
    city(&mut s, 1, hex(8, 6), 3);
    s.player_mut(P0).techs.extend([
        TechId::Irrigation,
        TechId::BronzeWorking,
        TechId::Masonry,
        TechId::Currency,
    ]);
    about_to_finish(&mut s, P0, TechId::Writing);
    s.player_mut(P0).science_stored = 1000;
    let orders = BTreeMap::from([(P0, vec![set_research(TechId::Writing)])]);
    let (next, events) = rules::resolve_turn(&s, &orders);
    assert!(next.player(P0).techs.contains(&TechId::Writing));
    assert_eq!(next.era_of(P0), 1);
    let ours: Vec<&Event> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Event::TechResearched { player: P0, .. }
                    | Event::EraEntered { player: P0, .. }
                    | Event::ResearchQueueAdvanced { player: P0, .. }
            )
        })
        .collect();
    assert_eq!(
        ours,
        [
            &Event::TechResearched {
                player: P0,
                tech: TechId::Writing
            },
            &Event::EraEntered {
                player: P0,
                era: 2,
                first: true
            },
            &Event::ResearchQueueAdvanced {
                player: P0,
                next: None
            },
        ]
    );
    // Everyone's view carries it.
    assert_eq!(player_view(&next, P1).players[0].era, 2);
    assert_eq!(player_view(&next, P1).research.era_firsts, [(1, P0)]);
}

#[test]
fn two_empires_entering_an_era_the_same_turn_have_the_first_drawn_by_seed() {
    let setup = |seed: u64, earlier: Option<PlayerId>| {
        let mut s = game(&[0, 1, 2]);
        s.dice_seed = Some(seed);
        for p in [P0, P1, P2] {
            know_eras_before(&mut s, p, 1);
            s.player_mut(p).techs.remove(&TechId::Writing);
        }
        if let Some(p) = earlier {
            s.era_firsts.insert(1, p);
        }
        about_to_finish(&mut s, P2, TechId::Writing);
        about_to_finish(&mut s, P1, TechId::Writing);
        let mut events = Vec::new();
        process_research(&mut s, &mut events);
        crate::eras::draw_same_turn_firsts(&mut s, &mut events);
        (s, events)
    };
    let mut firsts = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let (s, events) = setup(seed, None);
        let first = *s.era_firsts.get(&1).unwrap();
        assert!(first == P1 || first == P2);
        firsts.insert(first);
        // Exactly one entrant is flagged first, and it is the recorded one.
        let flagged: Vec<PlayerId> = events
            .iter()
            .filter_map(|e| match e {
                Event::EraEntered {
                    player,
                    era: 2,
                    first: true,
                } => Some(*player),
                _ => None,
            })
            .collect();
        assert_eq!(flagged, [first]);
    }
    assert_eq!(firsts.len(), 2, "either seat can be first");
    // An era with a first from an earlier turn is never redrawn.
    let (s, _) = setup(5, Some(P0));
    assert_eq!(s.era_firsts.get(&1), Some(&P0));
}
