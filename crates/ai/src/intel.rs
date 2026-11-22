//! Value of information from a fogged PlayerView: public army strength, sighting freshness and
//! likely routes. None of these estimates reveals an unseen current unit position.

use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{PlayerId, UnitClass, UnitKind, data};

impl Planner<'_> {
    /// Give a reachable, high-value stale conquest nominee a current sighting. This uses only
    /// the remembered city position and explored, legal ground in this player's view.
    pub(crate) fn refresh_stale_nominee(&mut self) {
        if !self.hard_defense() || self.decisions.stale_nominee == 0 {
            return;
        }
        let Some(id) = self.choose_conquest_city(false) else {
            return;
        };
        let Some(city) = self.view().cities.iter().find(|c| c.id == id) else {
            return;
        };
        if !self.stale_city(city) {
            return;
        }
        let pos = city.pos;
        let mut units: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| {
                u.military
                    && !self.has_acted(u.id)
                    && (u.kind == UnitKind::Scout
                        || u.pos.distance(pos) <= self.decisions.conquest_home_distance)
                    && (u.kind == UnitKind::Scout || u.max_moves >= 2)
                    && (u.kind == UnitKind::Scout
                        || !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
            })
            .collect();
        units.sort_by_key(|u| {
            (
                u.kind != UnitKind::Scout,
                -u.max_moves,
                data::unit_stats_for(self.faction, u.kind).cost,
                u.id,
            )
        });
        for unit in units {
            let mut best = None;
            for h in pos.within(data::UNIT_SIGHT) {
                if h == unit.pos {
                    continue;
                }
                let Some(tile) = self.view().tile(h) else {
                    continue;
                };
                if !tile.explored
                    || tile.move_cost().is_none()
                    || !self
                        .view()
                        .passage_allowed(self.view().me, unit.kind, tile.owner)
                    || !self.free_tile_for(unit, h)
                    || !(if unit.kind == UnitKind::Scout {
                        self.safe_scout_tile(unit, h)
                    } else {
                        self.safe_campaign_tile(h)
                    })
                {
                    continue;
                }
                let Some(step) = self.campaign_approach(unit, h) else {
                    continue;
                };
                let candidate = (unit.pos.distance(h), h.distance(pos), h, step);
                if best.is_none_or(|old| candidate < old) {
                    best = Some(candidate);
                }
            }
            if let Some((_, _, _, step)) = best {
                self.move_or_hold(unit, step);
                return;
            }
        }
    }
}

impl Planner<'_> {
    /// Production-equivalent value of finding (or continuing to watch) this rival's army.
    /// A strong army last seen many turns ago is more worth searching for than a small one
    /// currently in sight. Fresh knowledge still has a maintenance value.
    pub(crate) fn information_value(&self, rival: PlayerId) -> i32 {
        if !self.hard_defense() || self.decisions.intel_value == 0 {
            return 0;
        }
        let v = self.view();
        let public = v.army_of(rival);
        let visible: i32 = v
            .units
            .iter()
            .filter(|u| u.owner == rival && u.military)
            .map(crate::hold::army_value)
            .sum();
        let last = if visible > 0 {
            Some(v.turn)
        } else {
            v.observed_threats
                .iter()
                .filter(|o| o.owner == rival)
                .map(|o| o.seen_turn)
                .max()
        };
        let age = last.map_or(10, |turn| v.turn.saturating_sub(turn).min(10)) as i32;
        let strength = public.max(visible).max(10);
        ((i64::from(strength) * i64::from(2 + age) * i64::from(self.decisions.intel_value) / 1000)
            .clamp(0, 1200)) as i32
    }

    /// Visible force on an approach four movement turns away. This adds preparation time to
    /// the two-turn emergency horizon; it does not treat old sightings as live attackers.
    pub(crate) fn early_warning_power(&self, city: &CityView) -> i32 {
        if !self.hard_defense()
            || self.decisions.intel_value == 0
            || self.decisions.threat_horizon_turns == 0
        {
            return 0;
        }
        let v = self.view();
        v.units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && !u.owner.is_barbarian()
                    && !v.same_team(v.me, u.owner)
                    && (v.at_war(v.me, u.owner)
                        || crate::opportunity::legal_declaration(v, v.team_of(u.owner)))
                    && v.tile(u.pos).is_some_and(|t| t.visible)
                    && crate::hard_defense::declaration_arrival_turns(v, u, city.pos, 4).is_some()
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .all(|other| u.pos.distance(city.pos) <= u.pos.distance(other.pos))
            })
            .map(|u| {
                let current = crate::analysis::threat_power(v, u);
                let declare = if v.at_war(v.me, u.owner) {
                    100
                } else {
                    self.peace_declaration_percent(u.owner, u.pos)
                        * self.decisions.peace_exposure_percent
                        / 100
                };
                current * declare * (100 + self.decisions.intel_value) / 10_000
            })
            .sum()
    }

    /// Extra Production value for a unit whose class answers the approaching visible army.
    /// The class triangle already lives in `composition_bonus`; this term prices the city
    /// stake while a known army is on its way and fades with stale sightings.
    pub(crate) fn early_counter_value(&self, city: &CityView, unit: UnitKind) -> i32 {
        if !self.hard_defense() || self.decisions.counter_composition == 0 {
            return 0;
        }
        let Some(class) = crate::composition::class(unit) else {
            return 0;
        };
        let countered = match class {
            UnitClass::Melee => UnitClass::Mounted,
            UnitClass::Mounted => UnitClass::Ranged,
            UnitClass::Ranged => UnitClass::Melee,
            _ => return 0,
        };
        let v = self.view();
        let mut total = 0;
        let mut match_power = 0;
        for enemy in v.units.iter().filter(|u| {
            u.military
                && !v.same_team(v.me, u.owner)
                && !u.owner.is_barbarian()
                && crate::hard_defense::declaration_arrival_turns(v, u, city.pos, 4).is_some()
        }) {
            let power = crate::analysis::threat_power(v, enemy);
            total += power;
            if crate::composition::class(enemy.kind) == Some(countered) {
                match_power += power;
            }
        }
        if total == 0 {
            return 0;
        }
        let stake = crate::production::city_keep_value(city);
        stake * match_power * self.decisions.counter_composition / total / 10
    }
}

#[cfg(test)]
mod athens_8947_tests {
    use crate::{Brain, ScriptedBrain};
    use flate2::read::GzDecoder;
    use legatum_core::{AiDifficulty, Faction, Order, PlayerView, UnitId};

    #[test]
    fn celtic_t61_sends_an_idle_unit_to_refresh_eleusis() {
        let view: PlayerView = serde_json::from_reader(GzDecoder::new(
            include_bytes!("fixtures/athens-8947/T61-p4.view.json.gz").as_slice(),
        ))
        .unwrap();
        let memory: crate::OpponentMemory = serde_json::from_reader(GzDecoder::new(
            include_bytes!("fixtures/athens-8947/T61-p4.memory.json.gz").as_slice(),
        ))
        .unwrap();
        let mut old =
            ScriptedBrain::with_difficulty(Faction::Celts, 7, view.me, AiDifficulty::Hard);
        old.decisions.stale_nominee = 0;
        // Supported defence also moves these idle units; off in both arms so the re-scout is what is tested.
        old.decisions.supported_defence = 0;
        old.restore_opponent_memory(memory.clone());
        let old_orders = old.orders(&view);
        assert!(
            old_orders
                .iter()
                .any(|o| matches!(o, Order::SkipUnit { unit } if *unit == UnitId(14)))
        );
        assert!(
            old_orders
                .iter()
                .any(|o| matches!(o, Order::SkipUnit { unit } if *unit == UnitId(77)))
        );

        let mut new =
            ScriptedBrain::with_difficulty(Faction::Celts, 7, view.me, AiDifficulty::Hard);
        new.decisions.supported_defence = 0;
        new.restore_opponent_memory(memory);
        let orders = new.orders(&view);
        assert!(orders.iter().any(|o| matches!(o, Order::MoveUnit { unit, .. } if *unit == UnitId(14) || *unit == UnitId(77))), "orders: {orders:?}");
    }
}
