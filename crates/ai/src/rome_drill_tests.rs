use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, planner};
use legatum_core::view::PlayerView;
use legatum_core::{AiDifficulty, UnitId, UnitKind};

const HOME: Hex = Hex::new(10, 8);
const RIVAL: Hex = Hex::new(18, 8);

/// Hard Rome at war beside an unwalled rival city with `city_hp` HP; one Spearman per entry
/// of `xp` stands beside the city (units 1, 2, ...), the last entry's level given by `level`.
fn siege(city_hp: i32, xp: &[u32], last_level: u8) -> PlayerView {
    let mut b = ViewBuilder::new(40, 20)
        .faction(Faction::Rome)
        .city(1, ME, HOME, 3)
        .city(5, ENEMY, RIVAL, 3);
    let ring = RIVAL.ring(1);
    for (i, _) in xp.iter().enumerate() {
        b = b.unit(i as u32 + 1, ME, UnitKind::Spearman, ring[i]);
    }
    let mut v = b.build();
    for (i, x) in xp.iter().enumerate() {
        let u = v
            .units
            .iter_mut()
            .find(|u| u.id == UnitId(i as u32 + 1))
            .unwrap();
        u.xp = Some(*x);
        if !(i + 1 == xp.len()) {
        } else {
            u.level = last_level;
        }
    }
    let city = v.cities.iter_mut().find(|c| c.pos == RIVAL).unwrap();
    city.hp = city_hp;
    city.visible = true;
    assert!(v.at_war(ME, ENEMY));
    v
}

fn hard(v: &PlayerView, gene: i32) -> Planner<'_> {
    let mut p = planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.roman_founders = 1;
    p.decisions.rome_drill_rotation = gene;
    p
}

fn drill(p: &mut Planner<'_>, v: &PlayerView) {
    let army: Vec<_> = v.units.iter().filter(|u| u.owner == ME).collect();
    p.rome_drill_sieges(&army);
}

fn hits(p: &Planner<'_>, unit: u32) -> bool {
    p.orders.contains(&Order::MoveUnit {
        unit: UnitId(unit),
        to: RIVAL,
    })
}

fn holds(p: &Planner<'_>, unit: u32) -> bool {
    !(!(p.has_acted(UnitId(unit))) || !(!hits(p, unit)))
}

#[test]
fn fresh_recruits_take_the_early_hits_and_the_gene_off_orders_nothing() {
    let v = siege(200, &[0, 0], 0);
    let mut p = hard(&v, 1);
    drill(&mut p, &v);
    assert!(hits(&p, 1) && hits(&p, 2), "{:?}", p.orders);
    let mut off = hard(&v, 0);
    drill(&mut off, &v);
    assert!(off.orders.is_empty(), "{:?}", off.orders);
}

#[test]
fn a_ready_recruit_and_a_veteran_hold_while_an_early_hit_is_made() {
    // Unit 1 needs its early hits; unit 2 has the 6 XP the break completes; unit 3 is a
    // veteran. Only unit 1 spends the city's HP.
    let v = siege(200, &[0, 6, 20], 1);
    let mut p = hard(&v, 1);
    drill(&mut p, &v);
    assert!(hits(&p, 1), "{:?}", p.orders);
    assert!(holds(&p, 2) && holds(&p, 3), "{:?}", p.orders);
}

#[test]
fn when_no_early_hit_is_safe_every_recruit_beside_the_city_charges_together() {
    // 20 HP left: any hit breaks the city, so both recruits take the breaking stage's 10 XP
    // and the veteran stays out of it.
    let v = siege(20, &[0, 6, 20], 1);
    let mut p = hard(&v, 1);
    drill(&mut p, &v);
    assert!(hits(&p, 1) && hits(&p, 2), "{:?}", p.orders);
    assert!(holds(&p, 3), "{:?}", p.orders);
}

#[test]
fn early_hits_never_add_up_to_a_break() {
    // 60 HP: one Spearman hit is safe, two could break it. The recruit nearer 6 XP hits.
    let v = siege(60, &[0, 3], 0);
    let mut p = hard(&v, 1);
    drill(&mut p, &v);
    assert!(hits(&p, 2) && holds(&p, 1), "{:?}", p.orders);
}
