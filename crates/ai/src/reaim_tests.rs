//! Genes `war_reaim` (`plan.rs::reaim_candidates`) and `peace_own_danger`
//! (`diplomacy.rs::wants_peace`): a war on an enemy survives the end of one campaign, and Peace
//! with the enemy the plan fights follows that enemy's danger, not a third war's.
use crate::campaign::{CampaignEnd, CampaignEvent};
use crate::fixture::{ENEMY, ME, ViewBuilder};
use crate::plan::{Plan, PlanKind};
use crate::scripted::Planner;
use legatum_core::diplomacy::{Relation, RelationState};
use legatum_core::view::TeamStanding;
use legatum_core::{AiDifficulty, CityId, Hex, PlayerId, PlayerView, TeamId, UnitKind};

const THIRD: PlayerId = PlayerId(3);
const HOME: Hex = Hex { q: 3, r: 8 };
/// The campaign's city: walled, garrisoned, out of our army's reach.
const FIRST: Hex = Hex { q: 30, r: 8 };
/// The same enemy's empty city, about four turns' march from our army.
const SOFT: Hex = Hex { q: 14, r: 13 };

fn war(v: &mut PlayerView, team: TeamId) {
    v.diplomacy.relations.push(Relation {
        a: v.team,
        b: team,
        state: RelationState::War,
        effective_turn: 5,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: Some(team.0 as u32),
        declarers: vec![v.team],
    });
}

fn standings(v: &mut PlayerView) {
    v.standings = [
        (v.team, ME),
        (v.team_of(ENEMY), ENEMY),
        (v.team_of(THIRD), THIRD),
    ]
    .map(|(team, member)| TeamStanding {
        team,
        members: vec![member],
        vp: 2,
        target: 10,
        victory_hold: None,
    })
    .to_vec()
    .into();
}

/// At war with the enemy (two cities) and with a third empire. Our army of six has fallen back
/// from the enemy's walled first city, which a Trooper holds, and stands about four turns'
/// march from its second, empty one.
fn reaim_view() -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, HOME, 4)
        .city(9, ENEMY, FIRST, 4)
        .walled(9)
        .city(10, ENEMY, SOFT, 3)
        .city(20, THIRD, Hex::new(36, 2), 3)
        .unit(50, ENEMY, UnitKind::Trooper, FIRST)
        .unit(1, ME, UnitKind::Soldier, Hex::new(11, 9))
        .unit(2, ME, UnitKind::Soldier, Hex::new(10, 10))
        .unit(3, ME, UnitKind::Soldier, Hex::new(12, 8))
        .unit(4, ME, UnitKind::Ranger, Hex::new(11, 8))
        .unit(5, ME, UnitKind::Ranger, Hex::new(12, 7))
        .unit(6, ME, UnitKind::Soldier, Hex::new(13, 7))
        .build();
    v.diplomacy.legacy = false;
    v.players[ME.0 as usize].cities = 1;
    v.players[ENEMY.0 as usize].cities = 2;
    v.players[THIRD.0 as usize].cities = 1;
    let (enemy, third) = (v.team_of(ENEMY), v.team_of(THIRD));
    war(&mut v, enemy);
    war(&mut v, third);
    standings(&mut v);
    v
}

/// The plan was a Campaign on the first city, which ended this turn for want of an army.
fn campaign_ended(v: &PlayerView, reaim: i32) -> Planner<'_> {
    let mut p = crate::fixture::planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.plan_routing = true;
    p.decisions.war_reaim = reaim;
    // The gene itself, against any enemy (`persist_hollow` 0; `hollow_tests` covers the gate).
    p.decisions.persist_hollow = 0;
    p.campaigns.plan.active = Some(Plan {
        kind: PlanKind::Campaign {
            against: v.team_of(ENEMY),
            target: CityId(9),
        },
        start_turn: v.turn - 2,
        review_turn: v.turn + 8,
        reserve: 0,
        value: 500,
        terms: Vec::new(),
        breach: false,
        misses: 0,
    });
    p.campaigns.events.push(CampaignEvent {
        turn: v.turn,
        target: CityId(9),
        end: Some(CampaignEnd::InsufficientArmy),
    });
    p
}

fn soft_candidate(p: &Planner<'_>) -> Option<crate::plan::Candidate> {
    p.plan_candidates().into_iter().find(|c| {
        matches!(
            c.kind,
            PlanKind::Campaign {
                target: CityId(10),
                ..
            }
        )
    })
}

#[test]
fn an_ended_campaign_re_aims_at_the_same_enemys_empty_city() {
    let v = reaim_view();
    // Off: the march to the empty city is a second front while the third war is on, so the
    // plan has no campaign there and falls back to Peace.
    let off = campaign_ended(&v, 0);
    assert!(
        soft_candidate(&off).is_none(),
        "gene off: no re-aim candidate"
    );
    let on = campaign_ended(&v, 1);
    let soft = soft_candidate(&on).expect("gene on: the empty city is a campaign candidate");
    assert!(
        soft.value > 0,
        "a winnable re-aim is worth taking: {soft:?}"
    );
    assert!(
        soft.terms
            .iter()
            .any(|(label, _)| label.starts_with("re-aimed")),
        "{soft:?}"
    );
    // The plan follows: the war goes on against the same enemy rather than ending in Peace.
    let mut on = campaign_ended(&v, 1);
    on.choose_plan();
    assert_eq!(
        on.campaigns.plan.active.as_ref().map(|p| p.kind.clone()),
        Some(PlanKind::Campaign {
            against: v.team_of(ENEMY),
            target: CityId(10),
        })
    );
    let mut off = campaign_ended(&v, 0);
    off.choose_plan();
    assert!(
        !matches!(
            off.campaigns.plan.active.as_ref().map(|p| &p.kind),
            Some(PlanKind::Campaign {
                target: CityId(10),
                ..
            })
        ),
        "gene off keeps today's fallback"
    );
}

#[test]
fn re_aim_stands_down_when_no_city_in_reach_is_winnable() {
    // The enemy's other city is walled and held, and far beyond the march window: nothing to
    // re-aim at, so the gene adds no candidate and the old fallback stands.
    let mut v = reaim_view();
    let soft = v.cities.iter_mut().find(|c| c.id == CityId(10)).unwrap();
    soft.pos = Hex::new(38, 22);
    let p = campaign_ended(&v, 1);
    assert!(soft_candidate(&p).is_none());
    // Nor does it act while the campaign is still running on its own target.
    let v = reaim_view();
    let mut p = campaign_ended(&v, 1);
    p.campaigns.events.clear();
    p.campaigns.plan.active.as_mut().unwrap().kind = PlanKind::Campaign {
        against: v.team_of(ENEMY),
        target: CityId(10),
    };
    assert!(
        !p.plan_candidates().iter().any(|c| c
            .terms
            .iter()
            .any(|(label, _)| label.starts_with("re-aimed"))),
        "a campaign whose target is still a candidate is not re-aimed"
    );
}

#[test]
fn war_reaim_two_prices_an_empty_city_as_holding_one_bought_defender() {
    let v = reaim_view();
    let p = campaign_ended(&v, 2);
    let soft = v.cities.iter().find(|c| c.id == CityId(10)).unwrap();
    let defended = p
        .with_bought_defender(soft)
        .expect("an empty city gets the defender its owner could buy");
    let bought: Vec<_> = defended.units.iter().filter(|u| u.pos == SOFT).collect();
    assert_eq!(bought.len(), 1);
    assert_eq!(bought[0].owner, ENEMY);
    assert_eq!(
        bought[0].kind,
        UnitKind::Trooper,
        "the strongest kind its owner is seen fielding"
    );
    let first = v.cities.iter().find(|c| c.id == CityId(9)).unwrap();
    assert!(
        p.with_bought_defender(first).is_none(),
        "a garrisoned city is priced as seen"
    );
}

/// At war with the enemy our plan fights and with a third empire whose army stands at our city.
fn peace_view(enemy_near: bool) -> PlayerView {
    let mut b = ViewBuilder::new(30, 16)
        .city(1, ME, HOME, 4)
        .city(9, ENEMY, Hex::new(20, 8), 3)
        .city(20, THIRD, Hex::new(3, 14), 3)
        .unit(1, ME, UnitKind::Soldier, Hex::new(12, 8))
        .unit(2, ME, UnitKind::Soldier, Hex::new(12, 9))
        .unit(60, THIRD, UnitKind::Soldier, Hex::new(3, 10))
        .unit(61, THIRD, UnitKind::Soldier, Hex::new(4, 10))
        .unit(62, THIRD, UnitKind::Ranger, Hex::new(4, 11));
    if enemy_near {
        b = b
            .unit(70, ENEMY, UnitKind::Soldier, Hex::new(5, 8))
            .unit(71, ENEMY, UnitKind::Soldier, Hex::new(5, 9))
            .unit(72, ENEMY, UnitKind::Ranger, Hex::new(6, 8));
    }
    let mut v = b.build();
    v.diplomacy.legacy = false;
    let (enemy, third) = (v.team_of(ENEMY), v.team_of(THIRD));
    war(&mut v, enemy);
    war(&mut v, third);
    standings(&mut v);
    for c in v.cities.iter_mut().filter(|c| c.owner == ME) {
        c.hp = 150;
    }
    v
}

fn fights_enemy(v: &PlayerView, gene: i32) -> Planner<'_> {
    let mut p = crate::fixture::planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.plan_routing = true;
    p.decisions.peace_own_danger = gene;
    p.decisions.persist_hollow = 0;
    p.campaigns.plan.active = Some(Plan {
        kind: PlanKind::Contain {
            against: v.team_of(ENEMY),
        },
        start_turn: v.turn - 1,
        review_turn: v.turn + 9,
        reserve: 0,
        value: 600,
        terms: vec![("contest the hold".into(), 600)],
        breach: false,
        misses: 0,
    });
    p
}

#[test]
fn peace_with_the_planned_enemy_ignores_a_third_wars_pressure() {
    let v = peace_view(false);
    let mut off = fights_enemy(&v, 0);
    off.decisions.peace_pair_scope = 0;
    assert!(
        off.endangered(),
        "the third empire's army at our city counts as danger"
    );
    assert!(
        off.wants_peace(v.team_of(ENEMY)),
        "legacy pair scope: the third war's pressure ends the war the plan fights"
    );
    let on = fights_enemy(&v, 1);
    assert!(!on.endangered_by(v.team_of(ENEMY)));
    assert!(
        !on.wants_peace(v.team_of(ENEMY)),
        "gene on: Peace with the planned enemy would not remove that pressure"
    );
    // The war the plan does not fight still ends as before.
    assert!(on.wants_peace(v.team_of(THIRD)));
}

#[test]
fn peace_with_the_planned_enemy_still_follows_its_own_danger_and_a_collapse() {
    // The planned enemy's own army at our city: Peace with it removes real danger.
    let v = peace_view(true);
    let on = fights_enemy(&v, 1);
    assert!(on.endangered_by(v.team_of(ENEMY)));
    assert!(on.wants_peace(v.team_of(ENEMY)));
    // A city at half HP with a third empire's unit beside it is collapsing: the escape valve.
    let mut v = peace_view(false);
    for c in v.cities.iter_mut().filter(|c| c.owner == ME) {
        c.hp = c.max_hp / 2;
    }
    let on = fights_enemy(&v, 1);
    assert!(on.collapsing());
    assert!(on.wants_peace(v.team_of(ENEMY)));
}

#[test]
fn level_two_counts_our_whole_army_beside_the_town() {
    // One enemy shooter four hexes from our empty town, our army of three between them: the
    // local test (level 1) calls it danger, level 2 weighs the army we have there.
    let mut v = ViewBuilder::new(30, 16)
        .city(1, ME, HOME, 4)
        .city(9, ENEMY, Hex::new(20, 8), 3)
        .city(20, THIRD, Hex::new(3, 14), 3)
        .unit(1, ME, UnitKind::Soldier, Hex::new(6, 8))
        .unit(2, ME, UnitKind::Soldier, Hex::new(6, 7))
        .unit(3, ME, UnitKind::Ranger, Hex::new(7, 7))
        .unit(70, ENEMY, UnitKind::Ranger, Hex::new(7, 8))
        .build();
    v.diplomacy.legacy = false;
    let (enemy, third) = (v.team_of(ENEMY), v.team_of(THIRD));
    war(&mut v, enemy);
    war(&mut v, third);
    standings(&mut v);
    for c in v.cities.iter_mut().filter(|c| c.owner == ME) {
        c.hp = 150;
    }
    let one = fights_enemy(&v, 1);
    assert!(
        one.endangered_by(enemy),
        "level 1: the local test sees danger"
    );
    let two = fights_enemy(&v, 2);
    assert!(
        !two.endangered_by(enemy),
        "level 2: our army beside the town outweighs it"
    );
    assert!(!two.wants_peace(enemy));
}

#[test]
fn a_city_we_wore_down_is_endangered_only_by_an_enemy_that_outweighs_its_garrison() {
    // Our city just taken from the enemy, at 0 HP from our own siege. One enemy Soldier stands
    // beside it. Held by two Soldiers it is safe: its low HP is no danger by itself. Empty, the
    // same enemy would take it.
    let build = |garrison: u32| {
        let mut b = ViewBuilder::new(30, 16)
            .city(1, ME, HOME, 4)
            .city(2, ME, Hex::new(12, 8), 3)
            .city(9, ENEMY, Hex::new(20, 8), 3)
            .unit(70, ENEMY, UnitKind::Soldier, Hex::new(13, 8));
        for i in 0..garrison {
            b = b.unit(1 + i, ME, UnitKind::Soldier, Hex::new(12, 8));
        }
        let mut v = b.build();
        v.diplomacy.legacy = false;
        let enemy = v.team_of(ENEMY);
        war(&mut v, enemy);
        for c in v.cities.iter_mut().filter(|c| c.id == CityId(2)) {
            c.hp = 0;
        }
        v
    };
    for level in [1, 2] {
        let held = build(2);
        let p = fights_enemy(&held, level);
        assert!(
            !p.endangered_by(held.team_of(ENEMY)),
            "level {level}: a 0-HP city held by two Soldiers is not endangered by one"
        );
        assert!(!p.collapsing(), "level {level}: nor is it collapsing");
        let empty = build(0);
        let p = fights_enemy(&empty, level);
        assert!(
            p.endangered_by(empty.team_of(ENEMY)),
            "level {level}: an empty 0-HP city with the enemy beside it is"
        );
        assert!(p.collapsing(), "level {level}: and it is collapsing");
    }
}
