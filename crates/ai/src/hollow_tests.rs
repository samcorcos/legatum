//! Genes `hollow_read` (`threat.rs::runaway_signs`, `plan.rs::hollow_leader`) and
//! `persist_hollow` (`plan.rs::persists_against`): a rich empire with empty cities is read as a
//! runaway at a VP tie, from public numbers only (8.9.15 Swiss review rec 2).
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use crate::opponents::OpponentAssessments;
use crate::threat::{HOLLOW_ENTER, TOP_THREAT, hollow_bonus, hollowness, runaway_signs};
use legatum_core::diplomacy::{Relation, RelationState};
use legatum_core::view::TeamStanding;
use legatum_core::{AiDifficulty, PlayerId, PlayerView, TeamId};

const THIRD: PlayerId = PlayerId(3);

fn peace(a: u8, b: u8) -> Relation {
    Relation {
        a: TeamId(a),
        b: TeamId(b),
        state: RelationState::Peace,
        effective_turn: 0,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: None,
        declarers: Vec::new(),
    }
}

/// Every team at 3 of 10 VP. The enemy holds eight cities and a Power lead with an army worth 30
/// (one unit); we and the third empire hold three cities each with armies worth 300.
fn hollow_view(enemy_army: i32) -> PlayerView {
    let mut b = ViewBuilder::new(40, 20)
        .city(1, ME, off(4, 8), 3)
        .city(2, ME, off(4, 12), 3)
        .city(3, ME, off(6, 4), 3)
        .city(20, THIRD, off(34, 4), 3)
        .city(21, THIRD, off(34, 10), 3)
        .city(22, THIRD, off(34, 16), 3);
    for i in 0..8u32 {
        b = b.city(
            10 + i,
            ENEMY,
            off(14 + 2 * (i as i32 % 4), 4 + 4 * (i as i32 / 4)),
            4,
        );
    }
    let mut v = b.build();
    v.turn = 137;
    v.diplomacy.legacy = false;
    v.diplomacy.relations = vec![peace(0, 2), peace(0, 3), peace(2, 3)];
    for p in &mut v.players {
        p.cities = v.cities.iter().filter(|c| c.owner == p.id).count() as u32;
    }
    v.players[ENEMY.0 as usize].lead_percent = 64;
    crate::fixture::set_armies(
        &mut v,
        &[
            (ME, 300),
            (PlayerId(1), 0),
            (ENEMY, enemy_army),
            (THIRD, 300),
        ],
    );
    v.standings = [(0u8, ME), (2, ENEMY), (3, THIRD)]
        .map(|(team, member)| TeamStanding {
            team: TeamId(team),
            members: vec![member],
            vp: 3,
            target: 10,
            victory_hold: None,
        })
        .to_vec()
        .into();
    v
}

#[test]
fn a_rich_empty_empire_reads_hollow_at_a_vp_tie() {
    let v = hollow_view(30);
    let enemy = v.team_of(ENEMY);
    assert!(hollowness(&v, enemy) >= 90, "{}", hollowness(&v, enemy));
    assert!(hollow_bonus(&v, enemy) >= 20, "{}", hollow_bonus(&v, enemy));
    // Without the gene a VP tie reads nothing; with it the Power lead and the hollowness count.
    assert_eq!(runaway_signs(&v, enemy, 0), 0);
    assert!(runaway_signs(&v, enemy, HOLLOW_ENTER) >= 40);
    // An army in line with everyone else's is not hollow, whatever its riches.
    let armed = hollow_view(800);
    assert_eq!(hollowness(&armed, enemy), 0);
    assert_eq!(runaway_signs(&armed, enemy, HOLLOW_ENTER), 0);
    // Empty but poor (no goal, no lead, no era ahead, no bounty in sight): no read.
    let mut poor = hollow_view(30);
    poor.players[ENEMY.0 as usize].lead_percent = 0;
    assert_eq!(hollow_bonus(&poor, enemy), 0);
    // Two VP behind the race leader is not "near the top".
    let mut behind = hollow_view(30);
    behind.standings[0].vp = 5;
    assert_eq!(runaway_signs(&behind, enemy, HOLLOW_ENTER), 0);
}

#[test]
fn the_hollow_leader_becomes_the_top_threat_only_with_the_gene() {
    let v = hollow_view(30);
    let score = |hollow: i32| {
        let mut model = OpponentAssessments::default();
        model.threats.runaway_read = 1;
        model.threats.hollow_read = hollow;
        model.update(&v);
        (
            model.threats.get(ENEMY).unwrap().score,
            model.threats.top().map(|t| t.rival),
        )
    };
    let (off_score, off_top) = score(0);
    let (on_score, on_top) = score(1);
    assert!(off_score < TOP_THREAT, "off: {off_score}");
    assert_eq!(off_top, None);
    assert!(on_score >= TOP_THREAT, "on: {on_score}");
    assert_eq!(on_top, Some(ENEMY));
}

#[test]
fn the_hollow_leader_is_containable_and_wars_on_it_persist() {
    let v = hollow_view(30);
    let enemy = v.team_of(ENEMY);
    let mut p = crate::fixture::planner(&v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.hollow_read = 0;
    assert_eq!(p.contain_leader(), None, "a VP tie has no strict leader");
    p.decisions.hollow_read = 1;
    assert_eq!(p.contain_leader(), Some(enemy));
    // Persistence follows the read only with `persist_hollow`.
    let third = v.team_of(THIRD);
    p.decisions.persist_hollow = 0;
    assert!(p.persists_against(enemy) && p.persists_against(third));
    p.decisions.persist_hollow = 1;
    assert!(p.persists_against(enemy));
    assert!(!p.persists_against(third));
}

#[test]
fn one_bought_defender_does_not_end_the_hollow_read() {
    let mut model = OpponentAssessments::default();
    model.threats.runaway_read = 1;
    model.threats.hollow_read = 1;
    let mut v = hollow_view(30);
    let enemy = v.team_of(ENEMY);
    model.update(&v);
    assert!(model.threats.hollow_teams.contains_key(&enemy));
    // A few bought defenders: 35 hollow, under the entry floor but above the keep floor.
    v = hollow_view(520);
    v.turn += 1;
    assert_eq!(hollow_bonus(&v, enemy), 0, "a fresh read would not see it");
    model.update(&v);
    assert!(model.threats.hollow_teams.contains_key(&enemy));
    assert!(model.threats.get(ENEMY).unwrap().race > 0);
    // A real garrison ends it.
    v = hollow_view(720);
    v.turn += 2;
    model.update(&v);
    assert!(!model.threats.hollow_teams.contains_key(&enemy));
}

/// Gene `hollow_march` (`hollow_march.rs::garrison_spared`): at war with the hollow leader under
/// a Contain plan, a city with no foreign army in sight keeps no standing garrison; the capital
/// keeps its own unless part 2 is set, and a visible enemy unit nearby keeps it too.
#[test]
fn a_war_on_the_hollow_leader_spares_garrisons_no_army_threatens() {
    use crate::hollow_march::{SPARE, SPARE_CAPITAL};
    use crate::plan::{Plan, PlanKind};
    let spared = |enemy_army: i32, mask: i32, enemy_near: bool| {
        let mut v = hollow_view(enemy_army);
        let enemy = v.team_of(ENEMY);
        v.diplomacy.relations.retain(|r| !r.between(v.team, enemy));
        v.diplomacy.relations.push(Relation {
            state: RelationState::War,
            war_id: Some(1),
            declarers: vec![v.team],
            ..peace(0, 2)
        });
        if enemy_near {
            v.units.push(legatum_core::view::UnitView {
                pos: off(6, 12),
                ..crate::fixture::ViewBuilder::new(40, 20)
                    .unit(90, ENEMY, legatum_core::UnitKind::Horseman, off(6, 12))
                    .build()
                    .units[0]
                    .clone()
            });
        }
        let mut p = crate::fixture::planner(&v, 7);
        p.difficulty = AiDifficulty::Hard;
        p.plan_routing = true;
        p.decisions.hollow_march = mask;
        p.campaigns.plan.active = Some(Plan {
            kind: PlanKind::Contain { against: enemy },
            start_turn: 130,
            review_turn: 140,
            reserve: 0,
            value: 300,
            terms: Vec::new(),
            breach: false,
            misses: 0,
        });
        let capital = v
            .cities
            .iter()
            .find(|c| c.owner == ME && c.is_capital)
            .unwrap();
        let town = v
            .cities
            .iter()
            .find(|c| c.owner == ME && !c.is_capital)
            .unwrap();
        (p.garrison_spared(capital), p.garrison_spared(town))
    };
    assert_eq!(spared(30, 0, false), (false, false), "gene off");
    assert_eq!(spared(30, SPARE, false), (false, true));
    assert_eq!(spared(30, SPARE | SPARE_CAPITAL, false), (true, true));
    assert!(!spared(30, SPARE, true).1, "a Horseman two hexes away");
    assert_eq!(spared(720, SPARE, false), (false, false), "not hollow");
}
