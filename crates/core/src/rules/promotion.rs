//! Experience and promotions: Civ 6's experience model without promotion choices.
//!
//! A military unit that fights and survives earns XP. Aircraft do too: a plane acting this turn
//! stands in as a temporary map unit (`rules::air`) carrying its `Aircraft::xp`, so everything
//! below applies to it unchanged. Promotion is automatic and instant when
//! its cumulative XP reaches a threshold, and each level adds
//! [`data::PROMOTION_STRENGTH_PERCENT`]% of the unit's base strength (melee and ranged) as its
//! own breakdown line. There are no skills to choose.
//!
//! XP per fight (Civ 6, scored per fight, so a unit attacked twice in one stage earns twice):
//! - **Unit vs unit:** enemy base strength / own base strength, doubled if the enemy died, plus
//!   [`data::XP_MELEE_BONUS`] for melee and meeting fights or [`data::XP_RANGED_BONUS`] for a
//!   ranged shot, plus [`data::XP_INITIATOR_BONUS`] for the side that started it (both sides of
//!   a meeting engagement). Rounded half up, at most [`data::XP_UNIT_FIGHT_CAP`]. A side's base
//!   strength is its stat for its role in the fight (ranged strength for the shooter, melee
//!   strength otherwise), without promotions, terrain, fortification or wounds.
//! - **Attacking a city:** [`data::XP_CITY_ATTACK`]; the attack that drops it to 0 HP earns
//!   [`data::XP_CITY_DEFEATED`] instead (every attacker in that stage); a city already at 0 HP
//!   gives nothing. Cities do not attack, so no unit earns XP for defending against one.
//! - Civilians never fight, so they neither give nor earn XP.
//!
//! Levels: level N needs `data::PROMOTION_XP[N - 1]` XP (Civ 6's 15/45/90/150/225), up to [`MAX_LEVEL`] for other factions and level 6 for Sparta. XP is kept through upgrades (the unit changes kind in
//! place, as in Civ 6). Only civilians can be captured and they never carry XP, so captures
//! need no rule. Civ 6's 50 HP heal on promotion is left out: a promotion's effect stays one
//! visible number.

use super::battle::{Fight, FightKind, Target};
use crate::data;
use crate::events::Event;
use crate::experiments::{FactionRule, FactionRuleSet};
use crate::state::GameState;
use crate::types::*;
use std::collections::BTreeMap;

pub const MAX_LEVEL: u8 = data::PROMOTION_XP.len() as u8;
pub const SPARTA_MAX_LEVEL: u8 = MAX_LEVEL + 1;
const IRON_GUARD_XP: i32 = 318;

/// Cumulative XP to reach `level` (1..=MAX_LEVEL).
pub fn threshold(level: u8) -> Option<u32> {
    let i = usize::from(level).checked_sub(1)?;
    data::PROMOTION_XP.get(i).map(|&xp| xp.max(1) as u32)
}

pub fn threshold_for(faction: Faction, level: u8) -> Option<u32> {
    threshold_for_rules(FactionRuleSet::EMPTY, faction, level)
}

/// Whether `faction` climbs the Spartan ladder: one level higher, XP +50% (`sparta_veterans`).
pub fn spartan_ladder(off: FactionRuleSet, faction: Faction) -> bool {
    faction == Faction::Sparta && !off.contains(FactionRule::SpartaVeterans)
}

/// `threshold_for` with the faction rules in `off` switched off (ablation).
pub fn threshold_for_rules(off: FactionRuleSet, faction: Faction, level: u8) -> Option<u32> {
    if spartan_ladder(off, faction) && level == SPARTA_MAX_LEVEL {
        Some(IRON_GUARD_XP as u32)
    } else {
        threshold(level)
    }
}

pub fn level_for(faction: Faction, xp: u32) -> u8 {
    level_for_rules(FactionRuleSet::EMPTY, faction, xp)
}

/// `level_for` with the faction rules in `off` switched off (ablation).
pub fn level_for_rules(off: FactionRuleSet, faction: Faction, xp: u32) -> u8 {
    (1..=if spartan_ladder(off, faction) {
        SPARTA_MAX_LEVEL
    } else {
        MAX_LEVEL
    })
        .take_while(|&l| threshold_for_rules(off, faction, l).is_some_and(|t| xp >= t))
        .last()
        .unwrap_or(0)
}

pub fn next_level_xp_for(faction: Faction, xp: u32) -> Option<u32> {
    next_level_xp_for_rules(FactionRuleSet::EMPTY, faction, xp)
}

/// `next_level_xp_for` with the faction rules in `off` switched off (ablation).
pub fn next_level_xp_for_rules(off: FactionRuleSet, faction: Faction, xp: u32) -> Option<u32> {
    threshold_for_rules(off, faction, level_for_rules(off, faction, xp) + 1)
}

/// Promotion level (0..=MAX_LEVEL) for this much XP.
pub fn level(xp: u32) -> u8 {
    (1..=MAX_LEVEL)
        .take_while(|&l| threshold(l).is_some_and(|t| xp >= t))
        .last()
        .unwrap_or(0)
}

/// Cumulative XP for the next level, or None at the maximum level.
pub fn next_level_xp(xp: u32) -> Option<u32> {
    threshold(level(xp) + 1)
}

/// Strength a promotion adds to a base strength: `base * (100 + 10 * level) / 100 - base`,
/// rounded down like the rest of the combat math.
pub fn strength_bonus(base: i32, level: u8) -> i32 {
    let percent = data::PROMOTION_STRENGTH_PERCENT * i32::from(level.min(SPARTA_MAX_LEVEL));
    base * (100 + percent) / 100 - base
}

pub fn strength_label(level: u8) -> String {
    format!(
        "Level {level}: +{}%",
        data::PROMOTION_STRENGTH_PERCENT * i32::from(level.min(SPARTA_MAX_LEVEL))
    )
}

/// XP from one unit-vs-unit fight for the side with base strength `own`.
pub fn unit_fight_xp(own: i32, enemy: i32, enemy_died: bool, ranged: bool, initiator: bool) -> u32 {
    if own <= 0 || enemy <= 0 {
        return 0;
    }
    // Tenths of an XP, so the Civ 6 ratio rounds half up exactly once.
    let mut tenths = i64::from(enemy) * 10 * if enemy_died { 2 } else { 1 } / i64::from(own);
    tenths += 10
        * i64::from(if ranged {
            data::XP_RANGED_BONUS
        } else {
            data::XP_MELEE_BONUS
        });
    if initiator {
        tenths += 10 * i64::from(data::XP_INITIATOR_BONUS);
    }
    ((tenths + 5) / 10).min(i64::from(data::XP_UNIT_FIGHT_CAP)) as u32
}

/// XP for attacking a city from `hp_before` to `hp_after`.
pub fn city_attack_xp(hp_before: i32, hp_after: i32) -> u32 {
    if hp_before <= 0 {
        0
    } else if hp_after <= 0 {
        data::XP_CITY_DEFEATED as u32
    } else {
        data::XP_CITY_ATTACK as u32
    }
}

/// The XP a new armed unit or aircraft of `owner` starts with: the first level for the Spartan
/// ladder, nothing otherwise.
pub fn starting_xp(state: &GameState, owner: PlayerId, kind: UnitKind) -> u32 {
    if spartan_ladder(
        state.setup.rule_experiments.faction_rules_off,
        state.player(owner).faction,
    ) && data::unit_stats(kind).military
    {
        threshold(1).unwrap()
    } else {
        0
    }
}

/// A unit's base strength for its role in a fight.
fn base(state: &GameState, id: UnitId, shooter: bool) -> i32 {
    let u = &state.units[&id];
    if !data::unit_stats(u.kind).military {
        return 0;
    }
    let stats = state.unit_stats_of(u);
    if shooter {
        stats.ranged
    } else {
        stats.strength
    }
}

/// Every XP award from one stage of fights, planned against the state before its damage (the
/// same snapshot the fights were planned on). Deaths use the stage's summed damage, like
/// [`super::battle::apply_fights`]. Awards to units that die are dropped when granted.
pub(crate) fn combat_xp(
    state: &GameState,
    fights: &[Fight],
    extra_damage: &[(UnitId, i32)],
) -> Vec<(UnitId, u32)> {
    let mut unit_damage: BTreeMap<UnitId, i32> = BTreeMap::new();
    for (unit, damage) in extra_damage {
        *unit_damage.entry(*unit).or_default() += damage;
    }
    let mut city_damage: BTreeMap<CityId, i32> = BTreeMap::new();
    for f in fights {
        *unit_damage.entry(f.attacker).or_default() += f.damage_to_attacker;
        match f.target {
            Target::Unit(u) => *unit_damage.entry(u).or_default() += f.damage_to_defender,
            Target::City(c) => *city_damage.entry(c).or_default() += f.damage_to_defender,
        }
    }
    let dies = |id: UnitId| state.units[&id].hp - unit_damage.get(&id).copied().unwrap_or(0) <= 0;
    // (unit, XP, whether the opponent was a barbarian)
    let mut raw = Vec::new();
    for f in fights {
        match f.target {
            Target::Unit(d) => {
                let ranged = f.kind == FightKind::Ranged;
                let meeting = f.kind == FightKind::Meeting;
                let a_base = base(state, f.attacker, ranged);
                let d_base = base(state, d, false);
                let barbarian = |id: UnitId| state.units[&id].owner.is_barbarian();
                raw.push((
                    f.attacker,
                    unit_fight_xp(a_base, d_base, dies(d), ranged, true),
                    barbarian(d),
                ));
                raw.push((
                    d,
                    unit_fight_xp(d_base, a_base, dies(f.attacker), ranged, meeting),
                    barbarian(f.attacker),
                ));
            }
            Target::City(c) => {
                let hp = state.cities[&c].hp;
                let after = hp - city_damage[&c];
                raw.push((f.attacker, city_attack_xp(hp, after), false));
            }
        }
    }
    // Barbarians never promote, and fighting them can earn a unit its first promotion only
    // (Civ 6 EXPERIENCE_MAX_BARB_LEVEL; see rules::barbarians::cap_barbarian_xp). Awards are
    // capped in fight order against the unit's running XP within the stage.
    let first = threshold(super::barbarians::BARBARIAN_XP_MAX_LEVEL).unwrap_or(u32::MAX);
    let mut running: BTreeMap<UnitId, u32> = BTreeMap::new();
    let mut awards = Vec::new();
    for (id, xp, vs_barbarian) in raw {
        if state.units[&id].owner.is_barbarian() {
            continue;
        }
        let so_far = running.entry(id).or_insert(state.units[&id].xp);
        let xp = if vs_barbarian {
            super::barbarians::cap_barbarian_xp(*so_far, xp, first)
        } else {
            xp
        };
        let xp = if spartan_ladder(
            state.setup.rule_experiments.faction_rules_off,
            state.player(state.units[&id].owner).faction,
        ) {
            (xp * 3).div_ceil(2)
        } else {
            xp
        };
        *so_far = so_far.saturating_add(xp);
        if xp > 0 {
            awards.push((id, xp));
        }
    }
    awards
}

/// Add XP to surviving units and announce each promotion.
pub(crate) fn grant_xp(state: &mut GameState, awards: &[(UnitId, u32)], events: &mut Vec<Event>) {
    let mut totals: BTreeMap<UnitId, u32> = BTreeMap::new();
    for (id, xp) in awards {
        *totals.entry(*id).or_default() += xp;
    }
    for (id, xp) in totals {
        let Some(u) = state.units.get_mut(&id) else {
            continue;
        };
        let faction = state.players[u.owner.0 as usize].faction;
        let off = state.setup.rule_experiments.faction_rules_off;
        let before = level_for_rules(off, faction, u.xp);
        u.xp = u.xp.saturating_add(xp);
        let after = level_for_rules(off, faction, u.xp);
        if after > before {
            let fight = &mut state.players[u.owner.0 as usize].fight;
            fight.promotions = fight.promotions.saturating_add(u16::from(after - before));
            events.push(Event::UnitPromoted {
                unit: u.id,
                owner: u.owner,
                unit_kind: u.kind,
                level: after,
                at: u.pos,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::combat;
    use crate::orders::Order;

    #[test]
    fn thresholds_match_civ_at_standard() {
        let t = (1..=MAX_LEVEL)
            .map(|l| threshold(l).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(t, vec![16, 46, 90, 150, 226]);
        assert_eq!(threshold(0), None);
        assert_eq!(threshold(MAX_LEVEL + 1), None);
    }

    #[test]
    fn level_is_reached_exactly_at_each_threshold_and_capped() {
        assert_eq!(level(0), 0);
        assert_eq!(level(15), 0);
        assert_eq!(level(16), 1);
        assert_eq!(level(45), 1);
        assert_eq!(level(46), 2);
        assert_eq!(level(226), 5);
        assert_eq!(level(10_000), MAX_LEVEL);
        assert_eq!(next_level_xp(17), Some(46));
        assert_eq!(next_level_xp(500), None);
    }

    #[test]
    fn strength_multiplier_is_ten_percent_of_base_per_level_rounded_down() {
        assert_eq!(strength_bonus(20, 0), 0);
        assert_eq!(strength_bonus(20, 1), 2);
        assert_eq!(strength_bonus(20, 3), 6);
        assert_eq!(strength_bonus(15, 1), 1); // 16.5 -> 16
        assert_eq!(strength_bonus(5, 1), 0);
        assert_eq!(strength_bonus(5, 2), 1);
        assert_eq!(
            strength_bonus(20, MAX_LEVEL + 3),
            12,
            "capped at Spartan Veteran level"
        );
        for level in 0..=MAX_LEVEL {
            for base in [5, 8, 15, 20, 26, 44] {
                assert_eq!(
                    base + strength_bonus(base, level),
                    base * (100 + 10 * i32::from(level)) / 100
                );
            }
        }
    }

    #[test]
    fn promotion_is_its_own_breakdown_line_in_every_strength() {
        let f = Faction::India;
        let a = combat::unit_attack_strength_for(f, UnitKind::Soldier, 100, 2);
        assert_eq!(a.parts[1], ("Level 2: +20%".into(), 4));
        assert_eq!(a.total, 24);
        let d = combat::unit_defense_strength_for(f, UnitKind::Soldier, 100, Terrain::Plains, 3, 1);
        assert_eq!(d.parts[1], ("Level 1: +10%".into(), 2));
        let r = combat::unit_ranged_strength_for(f, UnitKind::Ranger, 100, 3);
        assert_eq!(r.parts[1], ("Level 3: +30%".into(), 4));
        let m = combat::unit_meeting_strength_for(f, UnitKind::Soldier, 100, 0);
        assert!(m.parts.iter().all(|(l, _)| !l.starts_with("Promotion")));
    }

    #[test]
    fn xp_per_fight_type_follows_civ6() {
        // Civ 6's example: a Warrior (20) attacks a Spearman (25).
        assert_eq!(unit_fight_xp(20, 25, true, false, true), 6); // 2.5 + 2 + 1 = 5.5
        assert_eq!(unit_fight_xp(25, 20, true, false, false), 4); // 1.6 + 2 = 3.6
        // Equal melee, nobody dies: attacker 1 + 2 + 1, defender 1 + 2.
        assert_eq!(unit_fight_xp(20, 20, false, false, true), 4);
        assert_eq!(unit_fight_xp(20, 20, false, false, false), 3);
        // Ranged gives less: +1 instead of +2.
        assert_eq!(unit_fight_xp(15, 20, false, true, true), 3); // 1.33 + 1 + 1
        assert_eq!(unit_fight_xp(20, 15, false, true, false), 2); // 0.75 + 1
        // More against stronger units, capped at 8.
        assert!(
            unit_fight_xp(20, 32, false, false, true) > unit_fight_xp(20, 20, false, false, true)
        );
        assert_eq!(unit_fight_xp(5, 44, true, false, true), 8);
        // Civilians neither earn nor give XP.
        assert_eq!(unit_fight_xp(0, 20, false, false, true), 0);
        assert_eq!(unit_fight_xp(20, 0, false, false, true), 0);
        // Cities.
        assert_eq!(city_attack_xp(200, 150), 3);
        assert_eq!(city_attack_xp(20, 0), 10);
        assert_eq!(city_attack_xp(0, 0), 0);
    }

    fn xp_of(s: &crate::state::GameState, id: UnitId) -> u32 {
        s.units[&id].xp
    }

    #[test]
    fn melee_fight_gives_both_survivors_xp_and_ranged_gives_less() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Soldier, hex(3, 3));
        let d = unit(&mut s, 1, UnitKind::Soldier, hex(4, 3));
        let os = orders(
            0,
            vec![Order::MoveUnit {
                unit: a,
                to: hex(4, 3),
            }],
        );
        let (next, _) = resolve_units(&s, &os);
        assert_eq!(xp_of(&next, a), 4, "1 + 2 melee + 1 attacking");
        assert_eq!(xp_of(&next, d), 3, "1 + 2 melee");

        let mut s = game(&[0, 1]);
        let r = unit(&mut s, 0, UnitKind::Ranger, hex(3, 3));
        let d = unit(&mut s, 1, UnitKind::Soldier, hex(5, 3));
        let os = orders(
            0,
            vec![Order::RangedAttack {
                unit: r,
                target: hex(5, 3),
            }],
        );
        let (next, _) = resolve_units(&s, &os);
        assert_eq!(xp_of(&next, r), 3, "20/15 + 1 ranged + 1 attacking");
        assert_eq!(xp_of(&next, d), 2, "15/20 + 1 ranged");
    }

    #[test]
    fn attacking_a_city_gives_xp_and_the_defeating_blow_more() {
        let mut s = game(&[0, 1]);
        let r = unit(&mut s, 0, UnitKind::Ranger, hex(3, 3));
        let c = city(&mut s, 1, hex(5, 3), 2);
        let shoot = orders(
            0,
            vec![Order::RangedAttack {
                unit: r,
                target: hex(5, 3),
            }],
        );
        let (next, _) = resolve_units(&s, &shoot);
        assert_eq!(xp_of(&next, r), 3);
        s.cities.get_mut(&c).unwrap().hp = 5;
        let (next, _) = resolve_units(&s, &shoot);
        assert_eq!(next.cities[&c].hp, 0);
        assert_eq!(xp_of(&next, r), 10);
        s.cities.get_mut(&c).unwrap().hp = 0;
        let (next, _) = resolve_units(&s, &shoot);
        assert_eq!(xp_of(&next, r), 0, "a defeated city gives nothing");
    }

    #[test]
    fn reaching_a_threshold_promotes_at_once_and_reports_it() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Soldier, hex(3, 3));
        let d = unit(&mut s, 1, UnitKind::Soldier, hex(4, 3));
        s.units.get_mut(&a).unwrap().xp = 12;
        let os = orders(
            0,
            vec![Order::MoveUnit {
                unit: a,
                to: hex(4, 3),
            }],
        );
        let (next, events) = resolve_units(&s, &os);
        assert_eq!(xp_of(&next, a), 16);
        assert_eq!(level(16), 1);
        let promoted: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::UnitPromoted { unit, level, .. } => Some((*unit, *level)),
                _ => None,
            })
            .collect();
        assert_eq!(
            promoted,
            vec![(a, 1)],
            "the defender (3 XP) stays at level 0"
        );
        assert_eq!(xp_of(&next, d), 3);
        // The next fight uses the new level.
        let b = crate::rules::battle::plan_fight(
            &next,
            1,
            a,
            crate::rules::battle::Target::Unit(d),
            FightKind::Melee,
        );
        assert!(
            b.attacker_strength
                .parts
                .iter()
                .any(|(l, v)| l == "Level 1: +10%" && *v == 2)
        );
    }

    #[test]
    fn dead_units_earn_nothing_and_killers_earn_double() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Soldier, hex(3, 3));
        let d = unit(&mut s, 1, UnitKind::Soldier, hex(4, 3));
        s.units.get_mut(&d).unwrap().hp = 1;
        let os = orders(
            0,
            vec![Order::MoveUnit {
                unit: a,
                to: hex(4, 3),
            }],
        );
        let (next, _) = resolve_units(&s, &os);
        assert!(!next.units.contains_key(&d));
        assert_eq!(xp_of(&next, a), 5, "2 (doubled) + 2 melee + 1 attacking");
    }

    #[test]
    fn upgrade_keeps_xp_and_level() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Phalangite, hex(3, 3));
        s.units.get_mut(&a).unwrap().xp = 46;
        s.player_mut(PlayerId(0)).gold = 500;
        s.player_mut(PlayerId(0)).techs.insert(TechId::Feudalism);
        // Phalangite→Pikeman also uses 1 Iron supply: an Iron deposit inside the border.
        s.player_mut(PlayerId(0))
            .techs
            .extend([TechId::BronzeWorking, TechId::IronWorking]);
        for h in hex(3, 3).within(1) {
            if let Some(t) = s.map.get_mut(h) {
                t.owner = Some(PlayerId(0));
            }
        }
        s.map.get_mut(hex(3, 4)).unwrap().resource = Some(crate::Resource::Iron);
        let os = orders(0, vec![Order::UpgradeUnit { unit: a }]);
        let (next, events) = resolve_units(&s, &os);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::UnitUpgraded { .. })),
            "{events:?}"
        );
        let u = &next.units[&a];
        assert_eq!(u.kind, UnitKind::Trooper);
        assert_eq!(u.xp, 46);
        assert_eq!(level(u.xp), 2);
    }

    #[test]
    fn xp_is_omitted_from_saves_at_zero() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Soldier, hex(3, 3));
        let json = serde_json::to_string(&s.units[&a]).unwrap();
        assert!(!json.contains("\"xp\""));
        s.units.get_mut(&a).unwrap().xp = 12;
        let json = serde_json::to_string(&s.units[&a]).unwrap();
        assert!(json.contains("\"xp\":12"));
        let back: crate::state::Unit = serde_json::from_str(&json).unwrap();
        assert_eq!(back.xp, 12);
    }
}
