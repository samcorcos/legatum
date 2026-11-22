//! Units with both a charge and a bow (the Mongol Keshig, `composition::dual`). One order a turn,
//! so each turn the AI weighs its best shot against its best charge in one currency, HP (kept in
//! hundredths): what the attack takes from the enemy (the expected damage it deals, plus the
//! harm the victim would do next turn, `tactics.rs::enemy_threat`, at the kill chance; a city it
//! takes is a city's stake) less what it costs us. A shot costs nothing up front but leaves the
//! unit where it stands, in whatever fire reaches that tile; a charge takes the defender's
//! retaliation (and any city shot), and a winning charge advances into the victim's tile (core
//! `rules/movement.rs`), into the fire that reaches there. So a Keshig charges an exposed Archer
//! (the mounted bonus wins the fight) and a city it can take, and shoots a fortified Spearman (its
//! arrows draw no spear bonus and no retaliation). Gene `dual_attack_choice` (0: the bow always
//! wins when it has a target).

use crate::composition;
use crate::scripted::Planner;
use crate::tactics::{expected_damage, fight_bonuses};
use legatum_core::view::UnitView;
use legatum_core::{AttackMode, Hex, Terrain, UnitId, data, forecast_city_strike};
use std::collections::BTreeMap;

/// A city we take, in HP: twice a city's HP, since we gain the city as its owner loses it (the
/// stake `enemy_threat` puts on a rival's capture of one of ours is one city's HP).
const CAPTURE_STAKE: i64 = 2 * data::CITY_HP as i64;

impl Planner<'_> {
    /// Whether this unit picks between its charge and its bow by value this turn.
    pub(crate) fn weighs_dual(&self, u: &UnitView) -> bool {
        self.decisions.dual_attack_choice != 0 && composition::dual(u.kind, u.ranged)
    }

    /// Whether a unit should charge `target` rather than shoot. Always for a unit without a bow,
    /// or a dual unit with nothing to shoot. Otherwise the charge must be worth more than its
    /// best shot (gene off: the bow wins).
    pub(crate) fn charge_over_shot(&self, u: &UnitView, target: Hex) -> bool {
        if !composition::dual(u.kind, u.ranged) {
            return true;
        }
        let Some((shot, _)) = self.dual_shot_net(u, &self.committed_fire()) else {
            return true;
        };
        self.decisions.dual_attack_choice != 0
            && self
                .charge_worth(u, target)
                .is_some_and(|charge| charge > shot)
    }

    /// The unit's best shot this turn (`best_shot`) and what it is worth, net of the fire the
    /// unit then stands in where it is. The target's own reply is part of the harm the shot
    /// takes away, so it is left out of that fire.
    pub(crate) fn dual_shot_net(
        &self,
        u: &UnitView,
        committed: &BTreeMap<Hex, i32>,
    ) -> Option<(i64, Hex)> {
        let target = self.best_shot(u, committed)?;
        let f = legatum_core::forecast_attack_mode(self.view(), u.id, target, AttackMode::Ranged)?;
        let spent = committed.get(&target).copied().unwrap_or(0);
        let view = self.view();
        let (gain, denial, skip) = match self.ctx.enemy_cities.iter().find(|c| c.pos == target) {
            Some(c) if c.hp > 0 => (
                // Capture progress, as `best_shot` scores it, in HP.
                self.city_shot_value(c.pos, c.hp - spent, &f) / 100,
                crate::hold::denial_priority(view, c.owner, || 0),
                None,
            ),
            _ => {
                let e = self.enemy_military_at(target)?;
                (
                    self.taken_from(e, e.hp - spent, &f),
                    crate::hold::denial_priority(view, e.owner, || 0),
                    Some(e.id),
                )
            }
        };
        let stay = 100 * i64::from(self.incoming_at(u, u.pos, u.hp, skip));
        Some((gain * (100 + i64::from(denial)) / 100 - stay, target))
    }

    /// The best charge this unit can make this turn, by `charge_worth`.
    pub(crate) fn best_charge(&self, u: &UnitView) -> Option<(i64, Hex)> {
        let reach = u.max_moves.max(1);
        let targets = self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| e.military)
            .map(|e| e.pos)
            .chain(
                self.ctx
                    .enemy_cities
                    .iter()
                    .filter(|c| c.hp == 0)
                    .map(|c| c.pos),
            )
            .filter(|h| u.pos.distance(*h) <= reach)
            .collect::<std::collections::BTreeSet<_>>();
        let mut best: Option<(i64, Hex)> = None;
        for h in targets {
            if let Some(value) = self.charge_worth(u, h)
                && best.is_none_or(|(b, _)| value > b)
            {
                best = Some((value, h));
            }
        }
        best
    }

    /// What charging `target` this turn is worth (`taken_from` the victim; a whole capture stake
    /// for a fallen city it takes), raised by a victory denial as a shot is, less the HP it costs:
    /// expected retaliation and city fire now, then the expected fire where it ends the turn
    /// (the victim's tile when it wins, its own when it doesn't). None when it can't get there
    /// this turn, the forecast says it would likely just die (`too_risky`), or the target is a
    /// city with HP (a siege is the city assault's call).
    pub(crate) fn charge_worth(&self, u: &UnitView, target: Hex) -> Option<i64> {
        if !composition::charges(u.kind)
            || u.pos.distance(target) > u.max_moves.max(1)
            || !self.view().tile(target).is_some_and(|t| t.visible)
        {
            return None;
        }
        let city = self.ctx.enemy_cities.iter().find(|c| c.pos == target);
        if city.is_some_and(|c| c.hp > 0) {
            return None;
        }
        let defender = self.enemy_military_at(target);
        if city.is_none() && defender.is_none() {
            return None;
        }
        if !self.reaches_this_turn(u, target) || self.too_risky(u, target) {
            return None;
        }
        let f = self.melee_forecast(u, target)?;
        let win = i64::from(f.win_chance.min(100));
        let view = self.view();
        let (gain, denial) = match (city, defender) {
            (Some(c), garrison) => (
                garrison.map_or(100 * CAPTURE_STAKE, |g| {
                    self.taken_from(g, g.hp, &f) + win * CAPTURE_STAKE
                }),
                crate::hold::denial_priority(view, c.owner, || 0),
            ),
            (None, Some(e)) => (
                self.taken_from(e, e.hp, &f),
                crate::hold::denial_priority(view, e.owner, || 0),
            ),
            (None, None) => return None,
        };
        let taken = (f.expected_damage_to_attacker + self.attack_city_damage(u, &f)).min(u.hp);
        let hp = (u.hp - taken).max(1);
        let skip = defender.map(|e| e.id);
        let after = win * i64::from(self.incoming_at(u, target, hp, skip))
            + (100 - win) * i64::from(self.incoming_at(u, u.pos, hp, skip));
        Some(gain * (100 + i64::from(denial)) / 100 - 100 * i64::from(taken) - after)
    }

    /// What one attack takes from an enemy with `remaining` HP, in hundredths of HP: the damage
    /// it deals (capped at what is left), plus, at its kill chance, the harm the victim would
    /// have done next turn.
    fn taken_from(&self, e: &UnitView, remaining: i32, f: &legatum_core::BattleForecast) -> i64 {
        let remaining = remaining.max(1);
        100 * i64::from(f.expected_damage_to_defender.min(remaining))
            + i64::from(f.win_chance.min(100)) * i64::from(self.enemy_threat(e, remaining))
    }

    /// The visible enemy military unit on a tile.
    fn enemy_military_at(&self, h: Hex) -> Option<&UnitView> {
        self.ctx
            .enemy_units
            .iter()
            .copied()
            .find(|e| e.military && e.pos == h)
    }

    /// Expected damage `u` takes next turn standing unfortified on `pos` with `hp` left: every
    /// visible enemy's most damaging attack that reaches it (a charge it can make, a shot in
    /// range, with the class bonus of that fight), plus walled cities' shots. `skip` leaves one
    /// enemy out. Capped at `hp`. An estimate for weighing, not a forecast.
    pub(crate) fn incoming_at(&self, u: &UnitView, pos: Hex, hp: i32, skip: Option<UnitId>) -> i32 {
        let view = self.view();
        let terrain = view
            .tile(pos)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense = crate::supply::defense_total(self.faction, u, hp, terrain, 0);
        let mut incoming = 0;
        for e in self.ctx.threat_units.iter().filter(|e| e.military) {
            if Some(e.id) == skip || e.pos == pos {
                continue;
            }
            let faction = view.faction_of(e.owner);
            let charge = (composition::charges(e.kind) && self.threat_reaches_this_turn(e, pos))
                .then(|| (crate::supply::attack_total(faction, e, e.hp), false));
            let shot = (e.ranged > 0
                && e.pos.distance(pos) <= e.range
                && self.ranged_unit_clear(e, e.pos, pos))
            .then(|| (crate::supply::ranged_total(faction, e, e.hp), true));
            incoming += [charge, shot]
                .into_iter()
                .flatten()
                .map(|(attack, shot)| {
                    let (bonus, against) = fight_bonuses(e.kind, u.kind, shot);
                    expected_damage(attack + bonus, defense + against, hp)
                })
                .max()
                .unwrap_or(0);
        }
        let mut there = u.clone();
        there.hp = hp;
        for c in &self.ctx.enemy_cities {
            if let Some(shot) = forecast_city_strike(view, c, &there, pos) {
                incoming += shot.expected_damage(hp);
            }
        }
        incoming.min(hp)
    }
}
