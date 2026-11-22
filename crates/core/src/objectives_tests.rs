use super::*;
use crate::experiments::RuleExperiments;
use crate::rules::cities::tests::{P0, P1, at, found, test_game, two_players};
use crate::rules::test_support::know_eras_before;
use crate::{Event, TechId, economy};

fn setup(seed: u64) -> crate::state::GameSetup {
    let mut s = two_players().setup;
    s.seed = seed;
    s
}

fn run(state: &mut GameState) -> Vec<Event> {
    let mut ev = Vec::new();
    scoring::update(state, &mut ev);
    ev
}

/// A World Age setup whose era-`era` draw satisfies `want`.
fn age_setup(era: u8, want: impl Fn(&[Objective]) -> bool) -> crate::state::GameSetup {
    (1..400)
        .map(setup)
        .find(|s| want(&era_objectives(s, era)))
        .expect("some seed draws it")
}

/// A setup whose draws satisfy `want` (seeds 1..400).
fn setup_where(want: impl Fn(&crate::state::GameSetup) -> bool) -> crate::state::GameSetup {
    (1..400)
        .map(setup)
        .find(|s| want(s))
        .expect("some seed draws it")
}

fn age_game(setup: crate::state::GameSetup) -> GameState {
    let mut s = two_players();
    s.setup = setup;
    s
}

fn card_slot(s: &GameState, era: u8, pred: impl Fn(&Objective) -> bool) -> u8 {
    era_objectives(&s.setup, era)
        .iter()
        .position(pred)
        .expect("drawn") as u8
}

fn player_view_eras(s: &GameState, p: PlayerId) -> Vec<u8> {
    crate::player_view(s, p)
        .eras
        .iter()
        .map(|e| e.era)
        .collect()
}

#[test]
fn every_age_draws_one_card_from_its_pool() {
    let mut s = setup(71);
    for era in 2..=7 {
        let pool = age_pool(era);
        assert_eq!(pool.len(), if era >= 4 { 7 } else { 6 });
        for seed in 1..60 {
            s.seed = seed;
            let cards = era_objectives(&s, era);
            assert_eq!(cards.len(), 1);
            assert_eq!(cards, era_objectives(&s, era), "same seed, same draw");
            assert!(scaled_pool(&s, era).contains(&cards[0]));
        }
    }
    assert!(era_objectives(&s, 1).is_empty());
    assert!(
        age_pool(2)
            .iter()
            .all(|c| !matches!(c, Objective::Cities { .. }))
    );
}

/// The peaceful cards (owner, 2026-11-22, via the review: Ages leaned on war): every Age offers
/// Patrons, Prosperity and Frontier, and Patrons asks for a building every faction can raise
/// (the Library line or the Bank, never the Workshop line Egypt lacks or Walls the Mongols lack).
#[test]
fn every_age_offers_three_peaceful_cards_every_faction_can_reach() {
    for era in 2..=data::era_count() {
        let pool = age_pool(era);
        let peaceful: Vec<&Objective> = pool
            .iter()
            .filter(|o| o.family() != Family::Fight && !matches!(o, Objective::Cities { .. }))
            .collect();
        assert_eq!(peaceful.len(), 3, "Age {era}: {pool:?}");
        assert!(
            pool.iter()
                .any(|o| matches!(o, Objective::Population { .. }))
        );
        assert!(
            pool.iter()
                .any(|o| matches!(o, Objective::Territory { .. }))
        );
        let Some(Objective::Buildings { building, .. }) = pool
            .iter()
            .find(|o| matches!(o, Objective::Buildings { .. }))
        else {
            panic!("Age {era} has a Patrons card");
        };
        assert!(
            matches!(
                data::building_family(*building),
                BuildingKind::Laboratory | BuildingKind::Clearinghouse | BuildingKind::Market
            ),
            "Age {era}: {building:?}"
        );
        assert!(pool.iter().filter(|o| o.family() == Family::Fight).count() >= 3);
    }
    assert!(age_pool(1).is_empty());
    assert!(age_pool(data::era_count() + 1).is_empty());
}

#[test]
fn the_first_empire_to_enter_an_era_opens_it_for_everyone_and_future_ages_stay_hidden() {
    let mut s = age_game(age_setup(2, |_| true));
    run(&mut s);
    assert!(player_view_eras(&s, P0).is_empty());
    assert!(player_view_eras(&s, P1).is_empty());
    know_eras_before(&mut s, P0, 1);
    run(&mut s);
    // P1 is still in the Ancient era but sees and holds the Classical Age.
    assert_eq!(s.player(P1).techs.len(), 0);
    assert!(s.player(P1).eras.contains_key(&2));
    assert_eq!(s.player(P1).eras[&2].entered, s.turn);
    assert_eq!(player_view_eras(&s, P1), vec![2]);
    assert_eq!(player_view_eras(&s, P0), vec![2]);
    let laggard = crate::player_view(&s, P1);
    assert!(
        laggard
            .objective_progress
            .iter()
            .all(|p| p.era == 2 && p.entered)
    );
    assert_eq!(laggard.objective_progress.len(), 1);
    // Nothing of the Medieval Age, in either view or on the wire.
    let medieval: Vec<String> = era_objectives(&s.setup, 3)
        .iter()
        .map(|o| o.text())
        .collect();
    for who in [P0, P1] {
        let json = serde_json::to_string(&crate::player_view(&s, who)).unwrap();
        for text in &medieval {
            if !era_objectives(&s.setup, 2)
                .iter()
                .any(|o| o.text() == *text)
            {
                assert!(!json.contains(text.as_str()), "{text} leaked to {who:?}");
            }
        }
    }
    know_eras_before(&mut s, P0, 2);
    run(&mut s);
    assert_eq!(player_view_eras(&s, P1), vec![2, 3]);
}

#[test]
fn an_age_begins_once_when_the_first_empire_enters_its_era() {
    let last = TechId::Writing;
    let enter = |s: &mut GameState, p: PlayerId| {
        know_eras_before(s, p, 1);
        s.player_mut(p).techs.remove(&last);
        let mut ev = Vec::new();
        crate::eras::learn(s, p, last, &mut ev);
        scoring::update(s, &mut ev);
        ev
    };
    let began = |ev: &[Event]| -> Vec<Event> {
        ev.iter()
            .filter(|e| matches!(e, Event::AgeBegan { .. }))
            .cloned()
            .collect()
    };
    let mut s = age_game(age_setup(2, |_| true));
    assert!(began(&run(&mut s)).is_empty());
    let ev = enter(&mut s, P0);
    assert_eq!(began(&ev), [Event::AgeBegan { era: 2, first: P0 }]);
    // Once only: a later turn and a second empire entering the same era add nothing.
    assert!(began(&run(&mut s)).is_empty());
    assert!(began(&enter(&mut s, P1)).is_empty());
    // The next era is a new Age.
    know_eras_before(&mut s, P0, 2);
    s.player_mut(P0).techs.remove(&TechId::Mathematics);
    let mut ev = Vec::new();
    crate::eras::learn(&mut s, P0, TechId::Mathematics, &mut ev);
    scoring::update(&mut s, &mut ev);
    assert_eq!(began(&ev), [Event::AgeBegan { era: 3, first: P0 }]);
}

fn veterans_age() -> GameState {
    let mut s = age_game(age_setup(2, |c| {
        c.iter().any(|o| matches!(o, Objective::Veterans { .. }))
    }));
    know_eras_before(&mut s, P0, 1);
    run(&mut s);
    s
}

/// Give `unit` enough XP to stand at `level` (one grant, so one promotion event of many levels).
fn promote_to(s: &mut GameState, unit: UnitId, level: u8) {
    let xp = crate::rules::promotion::threshold(level).unwrap() - s.units[&unit].xp;
    crate::rules::promotion::grant_xp(s, &[(unit, xp)], &mut Vec::new());
}

#[test]
fn veterans_count_levels_gained_in_battle_and_score_at_turn_end() {
    let mut s = veterans_age();
    let slot = card_slot(&s, 2, |o| matches!(o, Objective::Veterans { .. }));
    let card = era_objectives(&s.setup, 2)[usize::from(slot)];
    let Objective::Veterans { n } = card else {
        unreachable!()
    };
    assert_eq!(card.progress(&s, P1, 2), 0);
    let a = crate::game::spawn_unit(&mut s, P1, UnitKind::Soldier, at(5, 7));
    let b = crate::game::spawn_unit(&mut s, P1, UnitKind::Soldier, at(6, 7));
    promote_to(&mut s, a, 1);
    assert_eq!(card.progress(&s, P1, 2), 1);
    assert_eq!(
        card.progress(&s, P0, 2),
        0,
        "another empire's units count for it alone"
    );
    // Levels gained across one grant count each.
    promote_to(&mut s, b, u8::try_from(n).unwrap() - 1);
    assert_eq!(card.progress(&s, P1, 2), u32::from(n));
    run(&mut s);
    assert_eq!(s.player(P1).eras[&2].scored.unwrap().slot, slot);
    assert!(s.player(P0).eras[&2].scored.is_none());
}

#[test]
fn levels_a_unit_already_has_or_starts_with_count_nothing() {
    let mut s = veterans_age();
    let card = Objective::Veterans { n: 4 };
    // A unit that has the XP from before the reveal, or a bought unit that starts with it.
    let a = crate::game::spawn_unit(&mut s, P1, UnitKind::Soldier, at(5, 7));
    s.units.get_mut(&a).unwrap().xp = crate::rules::promotion::threshold(3).unwrap();
    assert_eq!(card.progress(&s, P1, 2), 0);
    // Only the next level counts.
    promote_to(&mut s, a, 4);
    assert_eq!(card.progress(&s, P1, 2), 1);
    // XP that stays inside a level adds nothing.
    crate::rules::promotion::grant_xp(&mut s, &[(a, 1)], &mut Vec::new());
    assert_eq!(card.progress(&s, P1, 2), 1);
}

#[test]
fn cards_do_not_expire() {
    let mut s = veterans_age();
    s.turn += 400;
    let a = crate::game::spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 7));
    promote_to(&mut s, a, 4);
    run(&mut s);
    assert!(s.player(P0).eras[&2].scored.is_some());
}

#[test]
fn only_what_is_done_after_the_reveal_counts() {
    let mut s = age_game(age_setup(2, |_| true));
    // Banked before the reveal: a hard capture, a walled capture, a city.
    found(&mut s, P1, at(3, 3));
    let f = &mut s.player_mut(P1).fight;
    f.humbled = 2;
    f.walled = 2;
    know_eras_before(&mut s, P0, 1);
    run(&mut s);
    for card in [
        Objective::HumbleTheMighty,
        Objective::Breach,
        Objective::Cities { n: 1 },
    ] {
        assert_eq!(card.progress(&s, P1, 2), 0, "{}", card.text());
    }
    found(&mut s, P1, at(9, 3));
    assert_eq!(Objective::Cities { n: 1 }.progress(&s, P1, 2), 1);
    s.player_mut(P1).fight.walled += 1;
    assert_eq!(Objective::Breach.progress(&s, P1, 2), 1);
}

/// Rules 31 cut the two paid cards, Tribute (Gold) and Monument (a Triumphal Arch). Every Age
/// keeps a drawable pool and no draw, in any game, ever offers either.
#[test]
fn the_tribute_and_monument_cards_are_gone_and_every_age_still_has_cards() {
    for era in 2..=7 {
        let pool = age_pool(era);
        assert!(!pool.is_empty(), "Age {era} has a pool");
        assert!(
            pool.iter()
                .all(|o| !o.text().starts_with("Tribute") && !o.text().starts_with("Monument")),
            "Age {era}: {pool:?}"
        );
        for seed in 1..200 {
            let setup = setup(seed);
            let drawn = era_objectives(&setup, era);
            assert_eq!(drawn.len(), 1, "Age {era}, seed {seed}");
            assert!(scaled_pool(&setup, era).contains(&drawn[0]));
        }
    }
    assert!(
        BuildingKind::ALL
            .iter()
            .all(|b| !serde_json::to_string(b).unwrap().contains("Triumphal"))
    );
    assert!(serde_json::from_str::<Objective>(r#"{"kind":"SpendGold","gold":70}"#).is_err());
    assert!(serde_json::from_str::<Objective>(r#"{"kind":"Monument","production":80}"#).is_err());
}

/// The freeze names itself on a refused purchase (not "is resisting occupation").
#[test]
fn an_occupied_city_refuses_with_the_occupied_text() {
    let mut s = veterans_age();
    let city = found(&mut s, P0, at(3, 3));
    s.cities.get_mut(&city).unwrap().occupied_from = Some(P1);
    s.player_mut(P0).gold = 10_000;
    let item = ProductionItem::Unit {
        unit: crate::types::UnitKind::Soldier,
    };
    let reason = economy::buy_blocker(&s, &s.cities[&city], item).expect("refused");
    assert!(reason.starts_with("Occupied: "), "{reason}");
    assert!(
        reason.contains("builds and buys nothing until Peace"),
        "{reason}"
    );
}

#[test]
fn humble_the_mighty_and_breach_skip_capitals_and_wonder_cities() {
    let mut s = age_game(age_setup(2, |_| true));
    give_era_points(&mut s, P1, 4);
    let capital = found(&mut s, P1, at(10, 10));
    let walled = found(&mut s, P1, at(15, 10));
    let plain = found(&mut s, P1, at(10, 15));
    let wonder = found(&mut s, P1, at(15, 15));
    for city in [walled, capital, wonder] {
        s.cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
    }
    s.cities
        .get_mut(&wonder)
        .unwrap()
        .buildings
        .insert(BuildingKind::Pyramids);
    know_eras_before(&mut s, P0, 1);
    run(&mut s);
    let take = |s: &mut GameState, city: CityId| {
        let vp = CaptureVpSnapshot::new(s);
        let mut events = Vec::new();
        crate::rules::battle::capture_city(s, city, P0, &mut events);
        record_fights(s, &events, &vp);
    };
    take(&mut s, capital);
    take(&mut s, wonder);
    let f = s.player(P0).fight;
    assert_eq!(
        (f.humbled, f.walled),
        (0, 0),
        "capital and wonder city never count"
    );
    take(&mut s, walled);
    let f = s.player(P0).fight;
    assert_eq!((f.humbled, f.walled), (1, 1));
    take(&mut s, plain);
    let f = s.player(P0).fight;
    assert_eq!((f.humbled, f.walled), (2, 1));
}

#[test]
fn a_celtic_sack_of_a_walled_city_counts_as_a_breach() {
    let mut s = age_game(age_setup(2, |_| true));
    s.player_mut(P0).faction = Faction::Celts;
    found(&mut s, P1, at(10, 15));
    let target = found(&mut s, P1, at(10, 10));
    s.cities
        .get_mut(&target)
        .unwrap()
        .buildings
        .insert(BuildingKind::Walls);
    know_eras_before(&mut s, P0, 1);
    run(&mut s);
    let vp = CaptureVpSnapshot::new(&s);
    let mut events = Vec::new();
    crate::rules::battle::capture_city(&mut s, target, P0, &mut events);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::CityRazed {
            walled: true,
            ordered: false,
            ..
        }
    )));
    record_fights(&mut s, &events, &vp);
    assert_eq!(s.player(P0).fight.walled, 1);
}

#[test]
fn deck_vp_hold_needs_no_city_capture() {
    let mut s = two_players();
    s.setup.rule_experiments.vp_target = Some(1);
    s.player_mut(P0).faction_goal_done = true;
    assert!(scoring::team_vp(&s, s.team_of(P0)) >= 1);
    run(&mut s);
    assert_eq!(s.vp_holds.get(&s.team_of(P0)), Some(&1));
}

#[test]
fn every_age_card_is_seed_deterministic_and_varies_by_seed() {
    assert!(era_objectives(&setup(7), 1).is_empty());
    assert!(era_objectives(&setup(7), data::era_count() + 1).is_empty());
    for era in 2..=data::era_count() {
        assert_eq!(
            era_objectives(&setup(7), era),
            era_objectives(&setup(7), era)
        );
    }
    // Different seeds must be able to draw different cards somewhere.
    let varied = (0..40u64).any(|seed| {
        (2..=data::era_count())
            .any(|era| era_objectives(&setup(seed), era) != era_objectives(&setup(7), era))
    });
    assert!(varied);
}

#[test]
fn rule_flags_parse_and_default_serialization_is_unchanged() {
    let mut r = RuleExperiments::default();
    assert!(r.is_default());
    r.apply("vp-target=8").unwrap();
    r.apply("decadence-slope=25").unwrap();
    assert_eq!(r.solo_vp_target(), 8);
    let back: RuleExperiments = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
    assert_eq!(back, r);
    assert_eq!(
        serde_json::to_string(&RuleExperiments::default()).unwrap(),
        r#"{"frontier_cities":false,"market_plunder":false,"wall_masonry":false}"#
    );
}

#[test]
fn great_works_score_directly() {
    let mut s = two_players();
    for i in 0..5 {
        found(&mut s, P0, at(1 + 4 * (i % 4), 1 + 5 * (i / 4)));
    }
    for c in s.cities.values_mut() {
        c.population = 8;
    }
    run(&mut s);
    // A built Great Work pays its holder directly.
    let city = s.cities_of(P0).next().unwrap().id;
    let ProductionItem::Wonder { tile, .. } =
        crate::landmarks::candidates(&s, &s.cities[&city], BuildingKind::RelayUplink)[0]
    else {
        unreachable!()
    };
    crate::landmarks::complete(&mut s, city, BuildingKind::RelayUplink, tile);
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::RelayUplink);
    s.player_mut(P0).great_work_completed = Some(BuildingKind::RelayUplink);
    run(&mut s);
    assert_eq!(s.player(P0).great_work_holder, Some(P0));
    assert!(crate::scoring::vp(&s, P0).contains(&(VpSource::GreatWork { of: P0 }, 2)));
}

/// Gain what the Age's Expand card asks for (cities, population or territory).
fn gain_expand(s: &mut GameState, era: u8) {
    let expand = era_objectives(&s.setup, era)
        .into_iter()
        .find(|o| o.family() == Family::Expand)
        .unwrap();
    match expand {
        Objective::Cities { n } => {
            for i in 0..i32::from(n) {
                found(s, P0, at(1 + (i % 4) * 4, 1 + (i / 4) * 5));
            }
        }
        _ => {
            found(s, P0, at(3, 3));
            found(s, P0, at(9, 3));
            for c in s.cities.values_mut() {
                c.population = 60;
            }
            for t in s.map.tiles.iter_mut() {
                t.owner = Some(P0);
            }
        }
    }
}

fn expand_card(cards: &[Objective]) -> bool {
    cards[0].family() == Family::Expand
}

#[test]
fn an_age_is_scored_once_by_meeting_its_card_and_stays_open_otherwise() {
    let mut s = age_game(age_setup(2, expand_card));
    // Not entered yet: nothing is recorded.
    run(&mut s);
    assert_eq!(s.player(P0).eras.len(), 0);
    // Entering era 2 (finishing the Ancient era) scores nothing: it counts only what comes after.
    let last = TechId::Writing;
    s.player_mut(P0)
        .techs
        .extend(crate::eras::era_techs(0).filter(|t| *t != last));
    run(&mut s);
    assert_eq!(
        s.player(P0).eras.len(),
        0,
        "four of five Ancient techs is still era 1"
    );
    let mut ev = Vec::new();
    crate::eras::learn(&mut s, P0, last, &mut ev);
    scoring::update(&mut s, &mut ev);
    assert!(ev.contains(&Event::EraEntered {
        player: P0,
        era: 2,
        first: true
    }));
    assert!(
        !ev.iter()
            .any(|e| matches!(e, Event::EraObjectiveScored { .. })),
        "an era is never scored by what the empire already held when it entered"
    );
    // Gaining what the Expand objective asks for after entering scores it.
    gain_expand(&mut s, 2);
    let ev = run(&mut s);
    assert!(ev.iter().any(
        |e| matches!(e, Event::EraObjectiveScored { player, era: 2, slot: 0, .. } if *player == P0)
    ));
    assert!(scoring::vp(&s, P0).contains(&(VpSource::EraObjective { era: 2 }, 1)),);
    // Only once.
    let ev = run(&mut s);
    assert!(
        !ev.iter()
            .any(|e| matches!(e, Event::EraObjectiveScored { .. }))
    );
    // The other empire never met anything: the Age stays open for it.
    assert!(s.player(P1).eras[&2].scored.is_none());
}

#[test]
fn what_an_empire_held_when_an_age_began_does_not_count_for_it() {
    let mut s = age_game(age_setup(2, expand_card));
    gain_expand(&mut s, 2);
    know_eras_before(&mut s, P0, 1);
    let ev = run(&mut s);
    assert!(
        !ev.iter()
            .any(|e| matches!(e, Event::EraObjectiveScored { .. }))
    );
    for o in era_objectives(&s.setup, 2) {
        assert_eq!(o.progress(&s, P0, 2), 0, "{}", o.text());
    }
    // Before an Age begins, nothing counts for it either.
    for o in era_objectives(&s.setup, 3) {
        assert_eq!(o.progress(&s, P0, 3), 0, "{}", o.text());
    }
}

#[test]
fn the_view_reports_the_count_baseline_total_turn_and_other_open_ages() {
    let mut s = age_game(setup_where(|s| {
        expand_card(&era_objectives(s, 3)) && fight_card(&era_objectives(s, 2))
    }));
    found(&mut s, P0, at(3, 3));
    know_eras_before(&mut s, P0, 2);
    s.turn = 112;
    run(&mut s);
    let o = era_objectives(&s.setup, 3)[0];
    let (base, total) = o
        .count_accounting(&s, P0, 3)
        .expect("Expand is a gain count");
    let accounting = crate::player_view(&s, P0).objective_accounting;
    let line = accounting
        .iter()
        .find(|p| p.era == 3 && p.slot == 0)
        .unwrap();
    assert_eq!(
        (line.baseline, line.total, line.counting_since_turn),
        (base, total, 112)
    );
    assert_eq!(line.restart_eras, vec![2]);
    s.turn = 113;
    win_fights(&mut s, P0);
    run(&mut s);
    assert!(s.player(P0).eras[&2].scored.is_some());
    assert_eq!(s.player(P0).eras[&3].counting_since, 113);
}

/// Pile up enough fights to meet any fight card.
fn win_fights(s: &mut GameState, p: PlayerId) {
    let f = &mut s.player_mut(p).fight;
    f.humbled += 100;
    f.walled += 100;
    f.promotions += 100;
}

fn fight_card(cards: &[Objective]) -> bool {
    cards[0].family() == Family::Fight
}

#[test]
fn one_event_scores_one_age_and_an_older_open_age_needs_fresh_work() {
    let mut s = age_game(setup_where(|s| {
        fight_card(&era_objectives(s, 2)) && fight_card(&era_objectives(s, 3))
    }));
    // Entering eras 2 and 3 on the same turn.
    know_eras_before(&mut s, P0, 2);
    run(&mut s);
    assert_eq!(s.player(P0).eras.len(), 2);
    win_fights(&mut s, P0);
    let ev = run(&mut s);
    let scored: Vec<u8> = ev
        .iter()
        .filter_map(|e| match e {
            Event::EraObjectiveScored { player, era, .. } if *player == P0 => Some(*era),
            _ => None,
        })
        .collect();
    assert_eq!(scored, vec![2], "R2: one era a turn, the oldest first");
    assert!(s.player(P0).eras[&3].scored.is_none());
    // R1: the fights that scored era 2 start nothing for era 3.
    for o in era_objectives(&s.setup, 3) {
        assert_eq!(o.progress(&s, P0, 3), 0, "{}", o.text());
    }
    assert!(
        run(&mut s)
            .iter()
            .all(|e| !matches!(e, Event::EraObjectiveScored { .. }))
    );
    // Fresh fights score it.
    win_fights(&mut s, P0);
    let ev = run(&mut s);
    assert!(
        ev.iter().any(
            |e| matches!(e, Event::EraObjectiveScored { player, era: 3, .. } if *player == P0)
        )
    );
}

#[test]
fn gains_count_from_the_last_age_score_for_cities_population_and_territory() {
    let mut s = age_game(age_setup(2, fight_card));
    know_eras_before(&mut s, P0, 2);
    run(&mut s);
    found(&mut s, P0, at(3, 3));
    found(&mut s, P0, at(9, 3));
    for c in s.cities.values_mut() {
        c.population = 12;
    }
    assert_eq!(Objective::Cities { n: 2 }.progress(&s, P0, 2), 2);
    assert_eq!(Objective::Cities { n: 2 }.progress(&s, P0, 3), 2);
    assert_eq!(Objective::Population { n: 24 }.progress(&s, P0, 3), 24);
    // Scoring era 2 restarts the count for era 3, and losing a city is not a gain.
    win_fights(&mut s, P0);
    run(&mut s);
    assert!(s.player(P0).eras[&2].scored.is_some());
    assert_eq!(Objective::Cities { n: 2 }.progress(&s, P0, 3), 0);
    assert_eq!(Objective::Population { n: 24 }.progress(&s, P0, 3), 0);
    assert_eq!(Objective::Territory { n: 1 }.progress(&s, P0, 3), 0);
    found(&mut s, P0, at(3, 12));
    assert_eq!(Objective::Cities { n: 2 }.progress(&s, P0, 3), 1);
    let gone = *s.cities.keys().next().unwrap();
    s.cities.remove(&gone);
    assert_eq!(Objective::Cities { n: 2 }.progress(&s, P0, 3), 0);
}

#[test]
fn a_buildings_objective_counts_cities_that_gained_the_building() {
    let mut s = two_players();
    let a = found(&mut s, P0, at(3, 3));
    found(&mut s, P0, at(9, 3));
    s.cities
        .get_mut(&a)
        .unwrap()
        .buildings
        .insert(BuildingKind::Market);
    know_eras_before(&mut s, P0, 1);
    run(&mut s);
    let o = Objective::Buildings {
        building: BuildingKind::Market,
        cities: 2,
    };
    assert_eq!(
        o.progress(&s, P0, 2),
        0,
        "the Market held at entry is not new"
    );
    let b = *s.cities.keys().find(|c| **c != a).unwrap();
    s.cities
        .get_mut(&b)
        .unwrap()
        .buildings
        .insert(BuildingKind::Market);
    assert_eq!(o.progress(&s, P0, 2), 1);
}

fn give_era_points(state: &mut GameState, player: PlayerId, points: u8) {
    for era in 2..2 + points {
        state.player_mut(player).eras.insert(
            era,
            EraRecord {
                scored: Some(EraScore { turn: 1, slot: 0 }),
                ..Default::default()
            },
        );
    }
}

/// A capital capture that levels the score in the same turn still leaves the victim ahead for the
/// turn's other captures: every capture is judged on the turn-start VP.
#[test]
fn a_five_four_capital_capture_keeps_its_pre_resolution_credit() {
    let mut s = two_players();
    let capital = found(&mut s, P1, at(10, 10));
    let plain = found(&mut s, P1, at(15, 10));
    give_era_points(&mut s, P0, 4);
    give_era_points(&mut s, P1, 5);
    let vp = CaptureVpSnapshot::new(&s);
    let mut events = Vec::new();
    crate::rules::battle::capture_city(&mut s, capital, P0, &mut events);
    assert_eq!(
        (
            scoring::team_vp(&s, s.team_of(P0)),
            scoring::team_vp(&s, s.team_of(P1))
        ),
        (5, 5)
    );
    crate::rules::battle::capture_city(&mut s, plain, P0, &mut events);
    record_fights(&mut s, &events, &vp);
    assert_eq!(
        s.player(P0).fight.humbled,
        1,
        "the capital never counts; the plain city does, against the turn-start lead"
    );
}

#[test]
fn a_wonder_transfer_keeps_the_victims_pre_resolution_lead() {
    let mut s = two_players();
    found(&mut s, P1, at(5, 15));
    let city = found(&mut s, P1, at(10, 10));
    let plain = found(&mut s, P1, at(15, 10));
    give_era_points(&mut s, P0, 4);
    give_era_points(&mut s, P1, 3);
    s.player_mut(P1).great_work_completed = Some(BuildingKind::RelayUplink);
    let site = s
        .map
        .tiles
        .iter_mut()
        .find(|t| t.city == Some(city))
        .unwrap();
    site.landmark = Some(crate::landmarks::Landmark {
        building: BuildingKind::RelayUplink,
        city,
        of: P1,
    });
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::RelayUplink);
    assert_eq!(scoring::team_vp(&s, s.team_of(P1)), 5);
    let vp = CaptureVpSnapshot::new(&s);
    let mut events = Vec::new();
    crate::rules::battle::capture_city(&mut s, city, P0, &mut events);
    assert!(scoring::team_vp(&s, s.team_of(P0)) >= scoring::team_vp(&s, s.team_of(P1)));
    crate::rules::battle::capture_city(&mut s, plain, P0, &mut events);
    record_fights(&mut s, &events, &vp);
    assert_eq!(
        s.player(P0).fight.humbled,
        1,
        "the wonder city never counts"
    );
}

#[test]
fn team_target_is_ten_per_member() {
    let mut s = test_game(&[
        (Faction::Byzantium, 0),
        (Faction::Byzantium, 0),
        (Faction::Byzantium, 1),
    ]);
    let solo = (u32::from(data::era_count()) + 4).min(10);
    assert_eq!(scoring::team_target(&s, TeamId(0)), 2 * solo);
    assert_eq!(scoring::team_target(&s, TeamId(1)), solo);
    s.setup.rule_experiments.vp_target = Some(10);
    assert_eq!(scoring::team_target(&s, TeamId(0)), 20);
    s.setup.rule_experiments.vp_target = Some(8);
    assert_eq!(scoring::team_target(&s, TeamId(0)), 16);
}

#[test]
fn decadence_slope_defaults_to_zero_and_can_be_opted_in() {
    let mut r = RuleExperiments::default();
    assert_eq!(r.decadence_slope(), 0);
    r.apply("decadence-slope=45").unwrap();
    assert_eq!(r.decadence_slope(), 45);
    assert!(r.apply("unknown").is_err());
}

/// Three empires, one city each, with the given Power histories.
fn three_with_power(powers: [i32; 3]) -> GameState {
    let mut s = test_game(&[
        (Faction::Byzantium, 0),
        (Faction::Byzantium, 1),
        (Faction::Byzantium, 2),
    ]);
    let window = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    for (i, power) in powers.iter().enumerate() {
        found(&mut s, PlayerId(i as u8), at(3 + 8 * i as i32, 3));
        s.players[i].power_history = vec![*power; window];
    }
    // Markets and size give each city real Gold, so a share of it rounds to something.
    for c in s.cities.values_mut() {
        c.population = 6;
        c.buildings.insert(BuildingKind::Market);
        c.buildings.insert(BuildingKind::Clearinghouse);
    }
    s
}

#[test]
fn power_is_a_trailing_average_and_dead_empires_have_no_lead() {
    let mut s = three_with_power([100, 100, 100]);
    s.players[2].power_history = vec![1; 4]; // being conquered
    let w = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    s.players[0].power_history = [vec![100; w], vec![220; 3]].concat();
    assert_eq!(
        scoring::power(&s, P0),
        (100 * (w as i32 - 3) + 220 * 3) / w as i32
    );
    // Dead empires pay nothing.
    s.players[2].alive = false;
    assert_eq!(scoring::lead_percent(&s, PlayerId(2)), 0);
}

#[test]
fn power_is_recorded_each_turn_and_the_window_is_bounded() {
    let mut s = three_with_power([0, 0, 0]);
    for p in &mut s.players {
        p.power_history.clear();
    }
    let w = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    for _ in 0..w + 3 {
        s = crate::resolve_turn(&s, &Default::default()).0;
    }
    for p in &s.players {
        assert_eq!(p.power_history.len(), w);
        assert!(p.power_history.iter().all(|v| *v > 0));
    }
}

#[test]
fn decadence_is_proportional_to_the_lead() {
    let r = RuleExperiments {
        decadence_slope: Some(45),
        ..Default::default()
    };
    // L * c / 100 with c 45, rounded to the nearest percent: nothing at or below the mean.
    assert_eq!([-50, 0].map(|l| r.decadence_percent(l)), [0, 0]);
    assert_eq!(
        [1, 2, 10, 30, 45, 60, 100, 200, 250].map(|l| r.decadence_percent(l)),
        [0, 1, 5, 14, 20, 27, 45, 90, 113]
    );
    // Twice as far ahead pays twice the rate (to the nearest percent), with no maximum.
    for l in (2..=2_000).step_by(2) {
        let (one, two) = (r.decadence_percent(l), r.decadence_percent(2 * l));
        assert!((two - 2 * one).abs() <= 1, "{l}: {one} {two}");
    }
    assert_eq!(r.decadence_percent(10_000), 4_500);
    // A leader twice as far ahead pays twice the rate.
    assert!((2 * r.decadence_percent(30) - r.decadence_percent(60)).abs() <= 1);
    // The slope is a knob; 0 turns decadence off.
    let steep = RuleExperiments {
        decadence_slope: Some(100),
        ..r
    };
    assert_eq!(steep.decadence_percent(37), 37);
    let none = RuleExperiments {
        decadence_slope: Some(0),
        ..r
    };
    assert_eq!(none.decadence_percent(500), 0);
}

/// The decadence of the three empires after `refresh_decadence` from the given Power histories.
fn decadent(powers: [i32; 3]) -> GameState {
    let mut s = three_with_power(powers);
    s.setup.rule_experiments.decadence_slope = Some(45);
    scoring::refresh_decadence(&mut s);
    s
}

#[test]
fn one_rate_cuts_every_yield_of_every_empire_above_the_mean() {
    let plain = three_with_power([160, 100, 100]);
    let s = decadent([160, 100, 100]);
    // Powers 160, 100, 100: the mean is 120, so P0 leads it by 33% (the median, 100, by 60%).
    assert_eq!(scoring::mean_power(&s), 120);
    assert_eq!(scoring::lead_percent(&s, P0), 33);
    let rate = s.setup.rule_experiments.decadence_percent(33);
    assert!(rate > 0);
    assert_eq!(scoring::decadence_percent(&s, P0), rate);
    assert_eq!(scoring::decadence_percent(&s, P1), 0);
    assert_eq!(scoring::decadence_percent(&s, PlayerId(2)), 0);
    let city = |s: &GameState, p: PlayerId| s.cities_of(p).next().unwrap().clone();
    let (before, after) = (
        economy::city_totals(&plain, &city(&plain, P0)),
        economy::city_totals(&s, &city(&s, P0)),
    );
    let cut = |v: i32| (v * rate + 50) / 100;
    assert!(before.production > 0 && before.gold > 0 && before.science > 0);
    assert_eq!(before.production - after.production, cut(before.production));
    assert_eq!(before.gold - after.gold, cut(before.gold));
    assert_eq!(before.science - after.science, cut(before.science));
    assert_eq!(
        before.food, after.food,
        "Food is slowed through the growth cost"
    );
    // The growth cost rises so that the same surplus grows the city `rate`% slower.
    let (g0, g1) = (
        economy::growth_threshold(&plain, &city(&plain, P0)),
        economy::growth_threshold(&s, &city(&s, P0)),
    );
    assert_eq!(g1.total, economy::decadence_growth(g0.total, rate));
    assert!(g1.total > g0.total);
    assert!(
        g1.parts.iter().any(
            |(l, v)| l.contains(&format!("Decadence \u{2212}{rate}% growth"))
                && *v == g1.total - g0.total
        ),
        "{:?}",
        g1
    );
    assert_eq!(economy::growth_threshold_total(&s, &city(&s, P0)), g1.total);
    // An empire at or below the average keeps its yields and growth exactly.
    assert_eq!(
        economy::city_totals(&plain, &city(&plain, P1)),
        economy::city_totals(&s, &city(&s, P1))
    );
    assert_eq!(
        economy::growth_threshold(&plain, &city(&plain, P1)),
        economy::growth_threshold(&s, &city(&s, P1))
    );
}

/// Four empires, one city each, with the given Power histories (a full window each).
fn four_with_power(powers: [i32; 4]) -> GameState {
    let mut s = test_game(&[
        (Faction::Byzantium, 0),
        (Faction::Byzantium, 1),
        (Faction::Byzantium, 2),
        (Faction::Byzantium, 3),
    ]);
    let window = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    for (i, power) in powers.iter().enumerate() {
        let site = [at(3, 3), at(11, 3), at(19, 3), at(7, 9)][i];
        found(&mut s, PlayerId(i as u8), site);
        s.players[i].power_history = vec![*power; window];
    }
    s.setup.rule_experiments.decadence_slope = Some(45);
    scoring::refresh_decadence(&mut s);
    s
}

#[test]
fn decadence_measures_the_lead_against_the_mean_so_a_runaways_lead_stays_bounded() {
    // A conqueror at 1,000 against three empires at 100: the mean is 325 (a lead of 207%)
    // because the leader is included.
    let s = four_with_power([1_000, 100, 100, 100]);
    assert_eq!(scoring::mean_power(&s), 325);
    assert_eq!(scoring::lead_percent(&s, P0), 207);
    let r = s.setup.rule_experiments;
    assert_eq!(scoring::decadence_percent(&s, P0), r.decadence_percent(207));
    // Nobody at or below the mean pays: three equal empires and a leader leave the others at 0.
    for p in [P1, PlayerId(2), PlayerId(3)] {
        assert_eq!(scoring::decadence_percent(&s, p), 0);
        assert!(scoring::lead_percent(&s, p) < 0);
    }
    // Equal empires: no lead, no cut.
    let even = four_with_power([100, 100, 100, 100]);
    assert!(even.players.iter().all(|p| p.decadence_percent == 0));
}

#[test]
fn decadence_uses_the_mean_power_lead() {
    // Powers 100, 110, 120, 500: the mean is 207. P2 (120) is below it.
    let s = four_with_power([100, 110, 120, 500]);
    let p2 = PlayerId(2);
    assert!(scoring::lead_percent(&s, p2) < 0);
    assert!(scoring::lead_percent(&s, PlayerId(3)) > 0);
    assert_eq!(scoring::decadence_percent(&s, p2), 0);
}

#[test]
fn the_power_window_is_fifty_turns_so_a_conquest_raises_the_rate_gradually() {
    let mut s = four_with_power([100, 100, 100, 100]);
    assert_eq!(usize::from(crate::experiments::POWER_WINDOW_TURNS), 50);
    // Decadence needs a full window of history: 49 turns say nothing.
    for p in &mut s.players {
        p.power_history.truncate(49);
    }
    s.players[0].power_history = vec![400; 49];
    assert_eq!(scoring::decadence_percent_now(&s, P0), 0);
    // A conquest: one empire's yields jump from 100 to 400 after a long peace. Power is the
    // trailing 50-turn average, so one turn at 400 moves it by 6 (300 / 50) and the rate follows
    // slowly.
    let mut s = four_with_power([100, 100, 100, 100]);
    s.players[0].power_history.push(400);
    s.players[0].power_history.remove(0);
    assert_eq!(scoring::power(&s, P0), 106);
    let one_turn = scoring::decadence_percent_now(&s, P0);
    for _ in 0..24 {
        s.players[0].power_history.push(400);
        s.players[0].power_history.remove(0);
    }
    assert_eq!(scoring::power(&s, P0), 250);
    let half = scoring::decadence_percent_now(&s, P0);
    assert!(one_turn < half, "{one_turn} then {half}");
}

#[test]
fn decadence_growth_scales_the_cost_so_growth_is_slower_by_the_rate() {
    assert_eq!(economy::decadence_growth(100, 0), 100);
    assert_eq!(economy::decadence_growth(90, 10), 100);
    assert_eq!(economy::decadence_growth(100, 20), 125);
    assert_eq!(economy::decadence_growth(40, 11), 45);
    // Never a division by zero or a cheaper city, whatever the rate.
    assert!(economy::decadence_growth(100, 100) >= 100);
    assert_eq!(economy::decadence_growth(100, -5), 100);
}

#[test]
fn every_decadence_cut_is_a_labelled_line_with_its_plain_reason() {
    // Larger yields so each cut rounds to at least one point.
    let mut s = three_with_power([160, 100, 100]);
    for c in s.cities.values_mut() {
        c.population = 12;
        c.buildings.insert(BuildingKind::Laboratory);
        c.buildings.insert(BuildingKind::Workshop);
    }
    s.setup.rule_experiments.decadence_slope = Some(100);
    let plain = s.clone();
    scoring::refresh_decadence(&mut s);
    let c = s.cities_of(P0).next().unwrap().clone();
    let y = economy::city_yields(&s, &c);
    assert_eq!(y.total, economy::city_totals(&s, &c), "sources add up");
    let lines: Vec<(&str, Yields)> = y
        .sources
        .iter()
        .filter(|l| l.label.starts_with("Decadence"))
        .map(|l| (l.label.as_str(), l.yields))
        .collect();
    for reason in ["sloth", "luxury", "complacency"] {
        let (label, yields) = lines
            .iter()
            .find(|(l, _)| l.contains(reason))
            .unwrap_or_else(|| panic!("no {reason} line in {lines:?}"));
        assert!(
            label.contains(&format!("\u{2212}{}%", scoring::decadence_percent(&s, P0))),
            "{label}"
        );
        assert!(yields.production + yields.gold + yields.science < 0);
    }
    // The breakdown sums to the totals: the lines carry exactly the cut.
    let cut: Yields = lines.iter().fold(Yields::ZERO, |a, (_, y)| a + *y);
    let before = economy::city_totals(&plain, &plain.cities_of(P0).next().unwrap().clone());
    assert_eq!(before + cut, y.total);
    // The empire at the median has no such line.
    let other = s.cities_of(P1).next().unwrap().clone();
    assert!(
        economy::city_yields(&s, &other)
            .sources
            .iter()
            .all(|l| !l.label.starts_with("Decadence"))
    );
}

#[test]
fn decadence_is_recorded_after_power_each_turn_and_follows_the_lead() {
    let mut s = three_with_power([100, 100, 100]);
    s.setup.rule_experiments.decadence_slope = Some(45);
    for p in &mut s.players {
        p.power_history.clear();
    }
    // P0 starts a lot richer: its Power history is far above the others'. The window is one turn
    // short of full, so the turn that fills it is the first with a verdict.
    let w = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    s.players[0].power_history = vec![400; w - 1];
    s.players[1].power_history = vec![100; w - 1];
    s.players[2].power_history = vec![100; w - 1];
    assert_eq!(scoring::decadence_percent(&s, P0), 0, "not yet refreshed");
    let next = crate::resolve_turn(&s, &Default::default()).0;
    assert!(
        scoring::decadence_percent(&next, P0) > 0,
        "the turn's end refreshes the rate from the lead"
    );
    assert_eq!(scoring::decadence_percent(&next, P1), 0);
    assert_eq!(
        scoring::decadence_percent(&next, P0),
        scoring::decadence_percent_now(&next, P0)
    );
    // A lead measured over fewer turns than the window is an opening, not a verdict.
    let mut early = s.clone();
    for p in &mut early.players {
        p.power_history.truncate(w - 2);
    }
    let early = crate::resolve_turn(&early, &Default::default()).0;
    assert!(early.players.iter().all(|p| p.decadence_percent == 0));
    assert!(
        scoring::lead_percent(&early, P0) > 0,
        "the lead is already public"
    );
}

#[test]
fn a_treasury_that_decadence_cannot_pay_makes_a_unit_desert_through_the_normal_path() {
    let mut s = decadent([400, 100, 100]);
    s.setup.rule_experiments.decadence_slope = Some(300);
    scoring::refresh_decadence(&mut s);
    assert!(scoring::decadence_percent(&s, P0) >= 90);
    for x in 4..9 {
        crate::game::spawn_unit(&mut s, P0, UnitKind::Soldier, at(x, 3));
    }
    s.player_mut(P0).gold = 0;
    let income = economy::player_income(&s, P0);
    assert!(income.gold.total < 0, "{:?}", income.gold);
    let v = crate::player_view(&s, P0);
    assert!(
        v.economy.bankruptcy.is_some(),
        "next turn's disband is forecast"
    );
    let (next, events) = crate::resolve_turn(&s, &Default::default());
    assert_eq!(next.player(P0).gold, 0);
    let reason = events
        .iter()
        .find_map(|e| match e {
            Event::UnitDisbanded { owner, reason, .. } if *owner == P0 => Some(reason.clone()),
            _ => None,
        })
        .expect("a unit disbanded");
    assert!(reason.contains("Treasury ran dry"), "{reason}");
    // Only the unit count changed: nothing else disbands units.
    assert_eq!(next.units_of(P0).count(), s.units_of(P0).count() - 1);
}

#[test]
fn the_view_publishes_opted_in_decadence() {
    let mut s = decadent([300, 100, 100]);
    for x in 4..9 {
        crate::game::spawn_unit(&mut s, P0, UnitKind::Soldier, at(x, 3));
    }
    s.player_mut(P0).gold = 3;
    let v = crate::player_view(&s, P0);
    let me = v.players.iter().find(|p| p.id == P0).unwrap();
    assert_eq!(me.decadence_percent, scoring::decadence_percent(&s, P0));
    assert!(me.decadence_percent > 0);
    let other = v.players.iter().find(|p| p.id == P1).unwrap();
    assert_eq!(other.decadence_percent, 0);
}

#[test]
fn card_counts_are_the_written_counts_unless_the_knob_scales_them() {
    let mut counted = 0;
    for seed in 1..40 {
        let mut big = setup(seed);
        big.rule_experiments.apply("objective-scale=150").unwrap();
        assert_eq!(setup(seed).rule_experiments.objective_scale_percent(), 100);
        for era in 2..=data::era_count() {
            let (base, scaled) = (era_objectives(&setup(seed), era), era_objectives(&big, era));
            assert_eq!(base.len(), scaled.len());
            for (b, n) in base.iter().zip(&scaled) {
                assert_eq!(n.family(), b.family(), "the draw does not move");
                let counted_kind = matches!(
                    b,
                    Objective::Cities { .. }
                        | Objective::Population { .. }
                        | Objective::Territory { .. }
                        | Objective::Buildings { .. }
                ) && !matches!(b, Objective::Cities { .. });
                if counted_kind {
                    counted += 1;
                    assert!(
                        n.target().abs_diff((b.target() * 150 + 50) / 100) <= 1,
                        "{b:?} -> {n:?}"
                    );
                    assert!(n.target() > b.target(), "{b:?} -> {n:?}");
                } else {
                    assert_eq!(n.target(), b.target(), "other cards are not scaled");
                }
            }
        }
    }
    assert!(counted > 0);
}
