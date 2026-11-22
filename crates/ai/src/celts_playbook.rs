//! Gene `celts_playbook`: the Hard Celts play their kit the way the designer won with it in
//! Playtest 8.9.25 (review `docs/playtests/reviews/2026-11-03-8.9.25-celts.md`, his account in
//! `docs/playtests/human-reasoning.md`). Barbarians never attack the Celts, so his cities stood
//! almost undefended and their production went to Settlers and the army; he called the Wild when
//! its 2 VP left one point to the target, and again whenever a new city or a cleared camp lapsed
//! it; and he chose his war by what he could see of the victim's defence (Walls, garrison,
//! army). Public information and the seat's own view only.
use crate::scripted::Planner;
use legatum_core::{Faction, PlayerView, view::CityView};

/// Parts of gene `celts_playbook` (a bit mask; Celts only).
/// 1: light garrisons where no rival army can come; 2: the Wild as a valued, repeatable call;
/// 4: war targets by their visible defence.
pub(crate) const LIGHT_GARRISONS: i32 = 1;
pub(crate) const WILD: i32 = 2;
pub(crate) const SOFT_TARGETS: i32 = 4;

/// A sheltered city has no foreign military unit (barbarians aside) seen this close (hexes,
/// reachable)...
const SHELTER_UNIT_RADIUS: i32 = 6;
/// ...and no city of an empire at War with us this close (`exposed_city`'s radius).
const SHELTER_WAR_CITY_RADIUS: i32 = 8;

/// Whether part `bit` of the playbook is on for the Celts in `view` (the gene is Celts only).
pub(crate) fn playbook(view: &PlayerView, gene: i32, bit: i32) -> bool {
    gene & bit != 0 && view.faction_of(view.me) == Faction::Celts
}

impl Planner<'_> {
    pub(crate) fn celts_playbook(&self, bit: i32) -> bool {
        self.faction == Faction::Celts && self.decisions.celts_playbook & bit != 0
    }

    /// Part 1: a Celtic city that needs no standing garrison. Barbarians never attack the Celts,
    /// so only an empire can take it, and none can come soon: no threat on it, no city of an
    /// empire at War within 8 reachable hexes, and no foreign military unit seen within 6. The
    /// capital (a capture is a VP), a Wonder, Hall or Heir city keep theirs. A garrison here
    /// would guard against nothing until an army comes into view, and then door defense and the
    /// emergency buys answer it, as the designer's purchases did.
    pub(crate) fn sheltered_city(&self, c: &CityView) -> bool {
        if !self.celts_playbook(LIGHT_GARRISONS)
            || c.is_capital
            || self.own_original_capital(c)
            || self.wonder_guard_city(c)
            || self.hall_guard_city(c)
            || self.heir_unguarded(c)
        {
            return false;
        }
        let threatened = self
            .ctx
            .own_cities
            .iter()
            .position(|o| o.id == c.id)
            .is_some_and(|i| self.ctx.threat.get(i).is_some_and(|t| *t > 0));
        if threatened {
            return false;
        }
        let v = self.view();
        let near = |h: legatum_core::Hex, radius: i32| {
            c.pos.distance(h) <= radius && self.ctx.reachable(c.pos, h)
        };
        let war_city = self
            .ctx
            .enemy_cities
            .iter()
            .any(|e| !e.owner.is_barbarian() && near(e.pos, SHELTER_WAR_CITY_RADIUS));
        let army_seen = v.units.iter().any(|u| {
            u.military
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && u.kind != legatum_core::UnitKind::Scout
                && near(u.pos, SHELTER_UNIT_RADIUS)
        });
        !war_city && !army_seen
    }

    /// The own cities the garrison quotas count: every city, less the sheltered ones (part 1).
    pub(crate) fn garrisoned_city_count(&self) -> i32 {
        let cities = self.ctx.own_cities.len() as i32;
        if !self.celts_playbook(LIGHT_GARRISONS) {
            return cities;
        }
        let sheltered = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| self.sheltered_city(c))
            .count() as i32;
        cities - sheltered
    }
}

impl Planner<'_> {
    /// Part 4: what the designer read off a war target in 8.9.25 ("because it was undefended
    /// and they had a weak army... the mongols too because they didn't have walls yet"): no
    /// Walls, no garrison seen in the city, and an owner whose public army (the Strongest Army
    /// title's values) is weaker than ours. Public information only; 0 when the part is off.
    pub(crate) fn celtic_soft_target(&self, c: &CityView) -> i32 {
        if !self.celts_playbook(SOFT_TARGETS) {
            return 0;
        }
        let v = self.view();
        let garrisoned = v
            .units
            .iter()
            .any(|u| u.military && u.pos == c.pos && v.same_team(u.owner, c.owner));
        let ours = crate::diplomacy::strength(v, v.team);
        let theirs = crate::diplomacy::strength(v, v.team_of(c.owner));
        let weak_army = (ours - theirs).max(0) * SOFT_ARMY_VALUE / ours.max(1);
        i32::from(!c.walls) * SOFT_UNWALLED_VALUE
            + i32::from(c.visible && !garrisoned) * SOFT_EMPTY_VALUE
            + weak_army
    }
}

/// Part 4's weights, in the conquest nominator's value units (a base city is worth 100 there):
/// an unwalled city, a city seen without a garrison, and an owner with no army against ours.
const SOFT_UNWALLED_VALUE: i32 = 60;
const SOFT_EMPTY_VALUE: i32 = 60;
const SOFT_ARMY_VALUE: i32 = 120;

impl Planner<'_> {
    /// Part 2: the guards of the camps in our own land are the goal, not targets. Barbarians are
    /// at war with everyone, so every attack pass saw a camp guard (at 10 HP from Celtic attrition)
    /// as a free kill and walked into the camp: in 256 games the Celts cleared their own camps
    /// the turn after almost every call and held the goal about one turn per game. Dropping them
    /// from the targets leaves `clear_barbarian_camps` as the one pass that decides about them.
    pub(crate) fn spare_own_camp_guards(&mut self) {
        if !self.celts_playbook(WILD) {
            return;
        }
        let v = self.view();
        let camps: Vec<legatum_core::Hex> = v
            .barbarian_camps
            .iter()
            .filter(|c| v.tile(c.pos).and_then(|t| t.owner) == Some(v.me))
            .map(|c| c.pos)
            .collect();
        self.ctx
            .enemy_units
            .retain(|u| !(u.owner.is_barbarian() && camps.contains(&u.pos)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off, planner};
    use legatum_core::{PlayerId, UnitKind};

    /// A Celtic capital, a second city 9 rows away and a rival city far to the east.
    fn celts() -> legatum_core::PlayerView {
        ViewBuilder::new(30, 24)
            .faction(Faction::Celts)
            .city(1, ME, off(3, 3), 4)
            .city(2, ME, off(3, 12), 4)
            .city(3, ENEMY, off(26, 12), 4)
            .build()
    }

    fn with_gene(v: &legatum_core::PlayerView, gene: i32) -> Planner<'_> {
        let mut p = planner(v, 7);
        p.decisions.celts_playbook = gene;
        p
    }

    #[test]
    fn a_celtic_city_no_army_can_reach_keeps_no_garrison_but_the_capital_does() {
        let v = celts();
        let p = with_gene(&v, LIGHT_GARRISONS);
        assert!(p.sheltered_city(&v.cities[1]));
        assert!(
            !p.sheltered_city(&v.cities[0]),
            "the capital keeps its guard"
        );
        assert_eq!(p.garrisoned_city_count(), 1);
        // Off, or another faction: every city counts.
        assert!(!with_gene(&v, 0).sheltered_city(&v.cities[1]));
        assert_eq!(with_gene(&v, 0).garrisoned_city_count(), 2);
        let mut germany = v.clone();
        germany.players[0].faction = Faction::Germany;
        assert!(!with_gene(&germany, LIGHT_GARRISONS).sheltered_city(&germany.cities[1]));
    }

    #[test]
    fn a_rival_soldier_in_sight_ends_the_shelter_but_a_barbarian_does_not() {
        let rival = ViewBuilder { view: celts() }
            .unit(50, ENEMY, UnitKind::Spearman, off(3, 16))
            .build();
        assert!(!with_gene(&rival, LIGHT_GARRISONS).sheltered_city(&rival.cities[1]));
        let barbarian = ViewBuilder { view: celts() }
            .unit(51, PlayerId::BARBARIAN, UnitKind::Spearman, off(3, 14))
            .build();
        assert!(with_gene(&barbarian, LIGHT_GARRISONS).sheltered_city(&barbarian.cities[1]));
    }

    #[test]
    fn the_guard_of_a_camp_in_our_land_is_no_target_once_the_wild_is_played() {
        let mut v = ViewBuilder { view: celts() }
            .owner(off(3, 6), ME)
            .unit(52, PlayerId::BARBARIAN, UnitKind::Spearman, off(3, 6))
            .unit(53, PlayerId::BARBARIAN, UnitKind::Spearman, off(12, 6))
            .build();
        v.barbarian_camps = [off(3, 6), off(12, 6)]
            .into_iter()
            .map(|pos| legatum_core::view::BarbarianCampView { pos, visible: true })
            .collect();
        let targets = |gene| {
            let mut p = with_gene(&v, gene);
            p.spare_own_camp_guards();
            p.ctx.enemy_units.iter().map(|u| u.id.0).collect::<Vec<_>>()
        };
        assert_eq!(targets(0), vec![52, 53]);
        assert_eq!(
            targets(WILD),
            vec![53],
            "the camp outside our land stays a target"
        );
    }

    #[test]
    fn a_soft_target_has_no_walls_no_garrison_and_a_weaker_owner() {
        let v = celts();
        let p = with_gene(&v, SOFT_TARGETS);
        let open = p.celtic_soft_target(&v.cities[2]);
        assert!(open >= SOFT_UNWALLED_VALUE, "{open}");
        let walled = ViewBuilder { view: celts() }
            .walled(3)
            .unit(54, ENEMY, UnitKind::Spearman, off(26, 12))
            .build();
        let p = with_gene(&walled, SOFT_TARGETS);
        assert!(p.celtic_soft_target(&walled.cities[2]) < open);
        assert_eq!(with_gene(&v, 0).celtic_soft_target(&v.cities[2]), 0);
    }
}
