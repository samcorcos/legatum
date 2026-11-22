//! The capitals victory (rules 37): a team wins at the end of a turn when it is the only team
//! still holding an original capital of its own. Everything here reads the public
//! `PlayerPublic::capital_stands` flags and the fogged view, never hidden state.
//!
//! Own capital: its fall ends our chance of winning that way and, when only one other team
//! still holds one, hands it the game, so a known rival army in reach makes it a stake
//! (`capital_pressed`: garrison, Walls and a ranged garrison come first, `hard_defense.rs`,
//! `military.rs`, `production.rs`). Rival capitals: `capital_race_vp` raises the worth of a
//! capital as fewer rivals still hold one, and the last one wins the game.

use crate::scripted::Planner;
use crate::winchance::WinFacts;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{Hex, TeamId, UnitKind};

/// VP-equivalents of taking the last standing rival capital: the game ends in our favour.
pub(crate) const WIN_VP: i32 = 10;
/// VP-equivalents of retaking our own team's only capital, or of denying a rival the win.
pub(crate) const RETAKE_VP: i32 = 10;
/// Hexes a rival unit can be from our capital and still count as coming for it (a few turns'
/// walk), seen now or remembered.
pub(crate) const CAPITAL_REACH: i32 = 6;

impl Planner<'_> {
    /// Alive teams that still hold an original capital of their own (public).
    pub(crate) fn capital_holders(&self) -> Vec<TeamId> {
        let mut teams: Vec<TeamId> = self
            .view()
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian() && p.capital_stands)
            .map(|p| p.team)
            .collect();
        teams.sort();
        teams.dedup();
        teams
    }

    fn alive_teams(&self) -> usize {
        let mut teams: Vec<TeamId> = self
            .view()
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian())
            .map(|p| p.team)
            .collect();
        teams.sort();
        teams.dedup();
        teams.len()
    }

    /// Whether `team` would still hold a capital with `founder`'s one gone.
    fn holds_another_capital(&self, team: TeamId, founder: legatum_core::PlayerId) -> bool {
        self.view()
            .players
            .iter()
            .any(|p| p.alive && p.team == team && p.id != founder && p.capital_stands)
    }

    /// VP-equivalents the capitals victory adds to taking (or, for our own team's capital,
    /// retaking) the original capital `city`, on top of its +1 VP (`capital_capture_vp`):
    /// taking the last standing rival capital while ours stands wins the game; otherwise the
    /// bonus grows as fewer rivals still hold one. Retaking our team's fallen capital is worth
    /// a win's worth when we have none left. With our own capital gone and two teams left
    /// holding one, restoring any third team's capital stops the leader's win.
    pub(crate) fn capital_race_vp(&self, city: &CityView) -> i32 {
        if self.decisions.capitals == 0 {
            return 0;
        }
        let v = self.view();
        let Some(founder) = city.original_capital_of else {
            return 0;
        };
        if !city.original_capital || city.owner.is_barbarian() || self.alive_teams() < 2 {
            return 0;
        }
        let (mine, founder_team, owner_team) = (v.team, v.team_of(founder), v.team_of(city.owner));
        if owner_team == mine {
            return 0;
        }
        let holders = self.capital_holders();
        let i_stand = holders.contains(&mine);
        if founder_team == mine {
            return if i_stand { 2 } else { RETAKE_VP };
        }
        if owner_team == founder_team {
            if !holders.contains(&founder_team) || self.holds_another_capital(founder_team, founder)
            {
                return 0;
            }
            if !i_stand {
                return 0;
            }
            let rivals_left = holders.len() as i32 - 2;
            return if rivals_left <= 0 {
                WIN_VP
            } else {
                (8 / (rivals_left + 1)).min(4)
            };
        }
        // A fallen rival capital held by a third team: taking it back restores its founder.
        if !i_stand && holders.len() == 2 && !holders.contains(&founder_team) {
            RETAKE_VP / 2
        } else {
            0
        }
    }

    /// The capitals-victory price of keeping a rival capital we captured (`keep`) or razing it
    /// (not `keep`): both end the game at once when it was the last one, and a razed capital can
    /// never be retaken by its founder.
    pub(crate) fn capital_restore_vp(&self, city: &CityView) -> i32 {
        if self.decisions.capitals == 0 {
            return 0;
        }
        let v = self.view();
        let Some(founder) = city.original_capital_of else {
            return 0;
        };
        if !city.original_capital || self.alive_teams() < 2 {
            return 0;
        }
        let founder_team = v.team_of(founder);
        if founder_team == v.team || self.holds_another_capital(founder_team, founder) {
            return 0;
        }
        let holders = self.capital_holders();
        if !holders.contains(&v.team) {
            return 0;
        }
        // Rivals still holding a capital if the founder got it back.
        let rivals = holders.iter().filter(|t| **t != v.team).count() as i32 + 1;
        (8 / (rivals + 1)).min(4)
    }

    /// A known rival unit (seen now at war or under a legal declaration, or remembered) within
    /// `CAPITAL_REACH` of `pos` that can walk there.
    pub(crate) fn rival_unit_in_reach(&self, pos: Hex) -> bool {
        let v = self.view();
        v.units.iter().any(|u| {
            u.military
                && u.kind != UnitKind::Scout
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && (v.at_war(v.me, u.owner)
                    || crate::opportunity::legal_declaration(v, v.team_of(u.owner)))
                && u.pos.distance(pos) <= CAPITAL_REACH
                && self.ctx.reachable(u.pos, pos)
        }) || self.ctx.remembered_threats.iter().any(|o| {
            !o.owner.is_barbarian()
                && o.kind != UnitKind::Scout
                && o.pos.distance(pos) <= CAPITAL_REACH
        })
    }

    /// Our team's original capital stands in our hands, there is a rival to lose it to, and a
    /// known rival military unit is within a few turns' walk of it.
    pub(crate) fn capital_pressed(&self, city: &CityView) -> bool {
        self.decisions.capitals > 0
            && city.original_capital
            && self.own_original_capital(city)
            && city.owner == self.ctx.me
            && self.alive_teams() > 1
            && self.rival_unit_in_reach(city.pos)
    }

    /// Fold the capitals victory into a capture projection: taking a standing rival capital
    /// drops its team's flag, retaking our team's capital restores ours.
    pub(crate) fn project_capital_capture(&self, facts: &mut WinFacts, city: &CityView) {
        let v = self.view();
        let (Some(founder), true) = (city.original_capital_of, city.original_capital) else {
            return;
        };
        let founder_team = v.team_of(founder);
        if founder_team == v.team {
            if let Some(t) = facts.team_mut(founder_team) {
                t.capital = true;
            }
        } else if v.team_of(city.owner) == founder_team
            && !self.holds_another_capital(founder_team, founder)
            && let Some(t) = facts.team_mut(founder_team)
        {
            t.capital = false;
        }
    }

    /// While a known rival unit is in reach of our standing original capital but not yet at its
    /// door, its garrison is not free for any other pass: one troop (a shooter if there is one)
    /// stays and holds, and an empty capital calls the nearest troop that can get there and
    /// defend it. Troops alone in another city are not stripped.
    pub(crate) fn secure_capital(&mut self, army: &[&UnitView]) {
        let Some(capital) = self
            .ctx
            .own_cities
            .iter()
            .copied()
            .find(|c| self.capital_pressed(c))
        else {
            return;
        };
        let pos = capital.pos;
        // Inside the threat radius the door-defence and sortie passes decide (a garrison may
        // strike an attacker it forecasts to kill); the pin covers the approach.
        let v = self.view();
        if v.units.iter().any(|u| {
            u.military
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && u.pos.distance(pos) <= crate::analysis::THREAT_RADIUS
        }) {
            return;
        }
        let inside = self
            .free(army)
            .into_iter()
            .filter(|u| u.pos == pos)
            .min_by_key(|u| (u.ranged == 0, u.id));
        if let Some(u) = inside {
            self.hold_or_upgrade(u);
            return;
        }
        if army.iter().any(|u| u.pos == pos) {
            return;
        }
        let alone = |u: &UnitView| {
            self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                && army.iter().filter(|o| o.pos == u.pos).count() < 2
        };
        let nearest = self
            .free(army)
            .into_iter()
            .filter(|u| {
                !alone(u) && u.pos.distance(pos) <= CAPITAL_REACH && self.can_defend_at(u, pos)
            })
            .min_by_key(|u| (u.pos.distance(pos), u.id));
        if let Some(u) = nearest {
            self.go_to(u, pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use crate::winchance::WinFacts;
    use legatum_core::view::PlayerView;
    use legatum_core::{CityId, Order, PlayerId, UnitId};

    const THIRD: PlayerId = PlayerId(3);

    /// We (team 0, capital city 1), ENEMY (team 2, capital city 2, 5 hexes away) and a third
    /// empire (team 3, capital city 3 far away). Every capital stands.
    fn world() -> PlayerView {
        let mut v = ViewBuilder::new(30, 16)
            .city(1, ME, off(6, 6), 3)
            .city(2, ENEMY, off(12, 6), 3)
            .city(3, THIRD, off(26, 14), 3)
            .build();
        for c in &mut v.cities {
            c.original_capital = true;
            c.original_capital_of = Some(c.owner);
        }
        v
    }

    /// `who`'s whole team loses its capitals.
    fn fall(v: &mut PlayerView, who: PlayerId) {
        let team = v.team_of(who);
        for p in v.players.iter_mut().filter(|p| p.team == team) {
            p.capital_stands = false;
        }
    }

    fn planner(v: &PlayerView, seed: u64) -> crate::scripted::Planner<'_> {
        let mut p = crate::fixture::planner(v, seed);
        p.decisions.capitals = 1;
        p
    }

    fn city(v: &PlayerView, id: u32) -> legatum_core::view::CityView {
        v.cities
            .iter()
            .find(|c| c.id == CityId(id))
            .unwrap()
            .clone()
    }

    #[test]
    fn a_rival_capital_is_worth_more_as_fewer_rivals_still_hold_one() {
        let mut v = world();
        let enemy = city(&v, 2);
        let many = planner(&v, 1).capital_race_vp(&enemy);
        assert_eq!(many, 4, "two rivals standing: taking one leaves one");
        fall(&mut v, THIRD);
        assert_eq!(
            planner(&v, 1).capital_race_vp(&enemy),
            WIN_VP,
            "the last standing rival capital wins the game"
        );
        let mut broke = v.clone();
        fall(&mut broke, ME);
        assert_eq!(
            planner(&broke, 1).capital_race_vp(&enemy),
            0,
            "without a capital of our own, taking theirs does not win"
        );
        // Our own fallen capital held by the enemy: retaking it is a win's worth.
        let mut lost = v.clone();
        fall(&mut lost, ME);
        lost.cities[0].owner = ENEMY;
        assert_eq!(
            planner(&lost, 1).capital_race_vp(&city(&lost, 1)),
            RETAKE_VP
        );
    }

    #[test]
    fn the_last_standing_rival_capital_beats_a_richer_ordinary_city() {
        let mut v = ViewBuilder::new(40, 24)
            .city(1, ME, off(4, 6), 4)
            .city(2, ME, off(4, 14), 4)
            .city(3, ENEMY, off(14, 6), 3)
            .city(4, ENEMY, off(14, 14), 6)
            .city(5, THIRD, off(36, 20), 3)
            .build();
        for c in &mut v.cities {
            let capital = matches!(c.id, CityId(1) | CityId(3) | CityId(5));
            c.original_capital = capital;
            c.original_capital_of = capital.then_some(c.owner);
        }
        // The ordinary city is a hair easier and richer: it is the target while two rivals
        // still hold their capitals.
        v.cities[2].strength = 200;
        v.cities[3].strength = 10;
        let target = |v: &PlayerView| {
            let mut p = planner(v, 1);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.decisions.peace_exposure_percent = 100;
            p.decisions.threat_horizon_turns = 2;
            p.conquest_city().map(|c| c.id)
        };
        assert_eq!(target(&v), Some(CityId(4)), "the easier, richer city first");
        fall(&mut v, THIRD);
        assert_eq!(
            target(&v),
            Some(CityId(3)),
            "the last standing rival capital is the win"
        );
    }

    #[test]
    fn keeping_or_razing_the_last_rival_capital_is_the_same_win() {
        let mut v = world();
        fall(&mut v, THIRD);
        // We took it: the city is ours, or it is gone (razed). Either way the enemy has fallen.
        let mut kept = v.clone();
        fall(&mut kept, ENEMY);
        kept.cities[1].owner = ME;
        let mut razed = kept.clone();
        razed.cities.remove(1);
        for view in [&kept, &razed] {
            assert_eq!(WinFacts::of(view).chance(), 10_000);
        }
        // Projecting the capture of the standing capital reaches the same win.
        let p = planner(&v, 1);
        let mut facts = WinFacts::of(&v);
        assert!(facts.chance() < 10_000);
        p.project_capital_capture(&mut facts, &city(&v, 2));
        assert_eq!(facts.chance(), 10_000);
    }

    #[test]
    fn the_capital_standing_is_priced_in_the_win_chance() {
        let v = world();
        let held = WinFacts::of(&v);
        let mut lost = held.clone();
        lost.team_mut(v.team).unwrap().capital = false;
        assert!(
            lost.chance() < held.chance(),
            "{} < {}",
            lost.chance(),
            held.chance()
        );
        let mut last = held.clone();
        for t in &mut last.teams {
            t.capital = t.team == v.team;
        }
        assert_eq!(last.chance(), 10_000);
        last.team_mut(v.team).unwrap().capital = false;
        last.team_mut(legatum_core::TeamId(2)).unwrap().capital = true;
        assert_eq!(last.chance(), 0, "a rival holding the last capital has won");
    }

    fn pressed_world(enemy: legatum_core::Hex) -> PlayerView {
        let mut v = ViewBuilder::new(30, 16)
            .city(1, ME, off(6, 6), 3)
            .unit(1, ME, UnitKind::Soldier, off(9, 6))
            .unit(50, ENEMY, UnitKind::Soldier, enemy)
            .city(2, ENEMY, off(26, 6), 3)
            .build();
        v.cities[0].original_capital = true;
        v.cities[0].original_capital_of = Some(ME);
        v.cities[1].original_capital = true;
        v.cities[1].original_capital_of = Some(ENEMY);
        v
    }

    #[test]
    fn a_lone_rival_unit_walking_to_the_empty_capital_finds_it_garrisoned() {
        let v = pressed_world(off(12, 6));
        let mut p = planner(&v, 1);
        assert!(p.capital_pressed(&city(&v, 1)));
        p.plan_military();
        assert!(
            p.orders.iter().any(|o| matches!(o,
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                    if *unit == UnitId(1) && *to == off(6, 6))),
            "orders: {:?}",
            p.orders
        );
    }

    #[test]
    fn a_rival_far_from_the_capital_does_not_press_it() {
        let v = pressed_world(off(26, 12));
        assert!(!planner(&v, 1).capital_pressed(&city(&v, 1)));
    }

    #[test]
    fn the_capitals_only_garrison_stays_home_while_a_rival_is_in_reach() {
        let mut v = pressed_world(off(11, 6));
        v.units.iter_mut().find(|u| u.id == UnitId(1)).unwrap().pos = off(6, 6);
        let mut p = planner(&v, 1);
        p.plan_military();
        assert_eq!(
            p.orders
                .iter()
                .find(|o| legatum_core::rules::units::order_unit(o) == Some(UnitId(1))),
            Some(&Order::SkipUnit { unit: UnitId(1) })
        );
    }
}
