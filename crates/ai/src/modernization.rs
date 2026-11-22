//! Refit and replacement investments, using the Strongest Army square and only known foes.
use crate::{composition, scripted::Planner};
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, UnitKind, army, combat, data, resources};

/// Gene `strike_upgrade_wait`: a unit this close to a home city with an at-war enemy unit this
/// close to it is defending, so it may still upgrade.
const HOME_DOOR: i32 = 4;

impl Planner<'_> {
    /// Current opponents near this front; peaceful border armies are potential opponents too.
    fn modernization_foes(&self, pos: Hex) -> Vec<&UnitView> {
        self.view()
            .units
            .iter()
            .filter(|e| {
                !self.view().same_team(self.view().me, e.owner)
                    // Barbarians never attack the Celts, so they are no opponent to upgrade for.
                    && !(e.owner.is_barbarian() && self.view().barbarians_spare_me())
                    && composition::is_combat(e.kind)
                    && (e.pos.distance(pos) <= 12
                        || self
                            .ctx
                            .own_cities
                            .iter()
                            .any(|c| c.pos.distance(e.pos) <= 6))
            })
            .collect()
    }

    /// Square effective strength, including the actual class counter and local phalanx.
    /// HP measures endurance, promotions survive upgrades, and supply changes strength.
    pub(crate) fn modernization_power(
        &self,
        kind: UnitKind,
        level: u8,
        hp: i32,
        unsupplied: bool,
        pos: Hex,
    ) -> i32 {
        let stats = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, kind);
        let base = army::effective_strength(stats.strength, stats.ranged, level, unsupplied);
        let off = self.view().rule_experiments.faction_rules_off;
        let phalanx = if data::is_hoplite_rules(off, self.faction, kind)
            && self.ctx.own_units.iter().any(|u| {
                data::is_hoplite_rules(off, self.faction, u.kind) && u.pos.distance(pos) == 1
            }) {
            data::HOPLITE_PHALANX_STRENGTH
        } else {
            0
        };
        let foes = self.modernization_foes(pos);
        // The triangle both ways: its own bonus against the foe, less the foe's against it (a
        // rider is weak against a spear line, 2026-10-31).
        let value = |enemy: Option<UnitKind>| {
            let bonus = enemy.map_or(0, |enemy| {
                combat::class_bonus_for_rules(
                    self.view().rule_experiments.faction_rules_off,
                    self.faction,
                    kind,
                    enemy,
                )
                .1 - combat::class_bonus(enemy, kind).1
            });
            army::full_health_value(base + phalanx + bonus)
        };
        let full = if foes.is_empty() {
            i64::from(value(None))
        } else {
            foes.iter()
                .map(|e| i64::from(value(Some(e.kind))))
                .sum::<i64>()
                / foes.len() as i64
        };
        (full * i64::from(hp.clamp(0, data::UNIT_HP)) / i64::from(data::UNIT_HP)) as i32
    }

    pub(crate) fn modernization_percent(&self, kind: UnitKind, pos: Hex) -> i32 {
        if let Some(value) = self.modernization_cache.borrow().get(&(pos, kind)) {
            return *value;
        }
        let foes = self.modernization_foes(pos);
        let reference = if foes.is_empty() {
            self.ctx
                .own_cities
                .iter()
                .filter_map(|c| c.detail.as_ref())
                .flat_map(|d| &d.options)
                .filter(|o| o.resource_blocker.is_none())
                .filter_map(|o| match o.item {
                    legatum_core::ProductionItem::Unit { unit } if composition::is_combat(unit) => {
                        Some(army::fresh_value(self.faction, unit))
                    }
                    _ => None,
                })
                .max()
                .unwrap_or(20)
        } else {
            (foes
                .iter()
                .map(|e| {
                    i64::from(army::combat_value(
                        e.strength,
                        e.ranged,
                        data::UNIT_HP,
                        e.level,
                        e.unsupplied.is_some(),
                    ))
                })
                .sum::<i64>()
                / foes.len() as i64) as i32
        };
        let power = self.modernization_power(
            kind,
            u8::from(self.faction == Faction::Sparta),
            data::UNIT_HP,
            false,
            pos,
        );
        let value = (i64::from(power) * 100 / i64::from(reference.max(1))).clamp(1, 400) as i32;
        self.modernization_cache
            .borrow_mut()
            .insert((pos, kind), value);
        value
    }

    /// Linear strength of a fresh `kind` at `pos` with the triangle both ways, averaged over the
    /// foes seen there (its bonus against each, less each one's against it): since the stronger
    /// riders (2026-10-31) a Horseman is the strongest Ancient unit on paper but a poor answer
    /// to a spear line.
    pub(crate) fn matchup_strength(&self, kind: UnitKind, pos: Hex) -> i32 {
        let stats = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, kind);
        let base = stats.strength.max(stats.ranged);
        let foes = self.modernization_foes(pos);
        if foes.is_empty() {
            return base;
        }
        let edge: i32 = foes
            .iter()
            .map(|e| {
                combat::class_bonus_for_rules(
                    self.view().rule_experiments.faction_rules_off,
                    self.faction,
                    kind,
                    e.kind,
                )
                .1 - combat::class_bonus(e.kind, kind).1
            })
            .sum();
        base + edge / foes.len() as i32
    }

    /// Existing buy paths historically compare linear strength/Gold. Keep their scale at zero.
    pub(crate) fn purchase_combat_strength(&self, kind: UnitKind, pos: Hex) -> i32 {
        let stats = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, kind);
        let old = stats.strength.max(stats.ranged);
        if self.decisions.modernization_value == 0 {
            return old;
        }
        let new = self.modernization_power(
            kind,
            u8::from(self.faction == Faction::Sparta),
            data::UNIT_HP,
            false,
            pos,
        );
        old + (new - old) * self.decisions.modernization_value / 100
    }

    /// An upgrade order reaches the newest researched form in one turn and pays every step.
    fn refit(&self, u: &UnitView) -> Option<(i32, i32, i32)> {
        if !composition::is_combat(u.kind)
            || u.contract_until.is_some()
            || u.upgrade_supply_blocker.is_some()
            || self.view().actions.retire.as_ref().is_some_and(|r| {
                r.veterans
                    .iter()
                    .any(|v| v.unit == u.id && v.blocker.is_none())
            })
        {
            return None;
        }
        let to = u.upgrade_to?;
        let cost = u.upgrade_breakdown.as_ref()?.total;
        let before = self.modernization_power(u.kind, u.level, u.hp, u.unsupplied.is_some(), u.pos);
        let still_unsupplied =
            u.unsupplied.is_some() && resources::unit_need(to) == resources::unit_need(u.kind);
        let after = self.modernization_power(to, u.level, u.hp, still_unsupplied, u.pos);
        let gain = after - before;
        (gain > 0).then_some((gain, cost, 1))
    }

    pub(crate) fn modernization_reserve(&self) -> i32 {
        if self.decisions.modernization_upgrade == 0 {
            return 0;
        }
        self.ctx
            .own_units
            .iter()
            .filter(|u| {
                self.decisions.modernization_return > 0
                    || self
                        .view()
                        .tile(u.pos)
                        .is_some_and(|t| t.owner == Some(self.view().me))
            })
            .filter_map(|u| {
                let (gain, cost, _) = self.refit(u)?;
                (gain > 0
                    && cost <= self.gold_left + self.view().economy.gold_per_turn.total.max(0) * 12)
                    .then_some(cost)
            })
            .min()
            .unwrap_or(0)
    }

    pub(crate) fn modernization_upgrade_worth(&self, u: &UnitView) -> bool {
        self.decisions.modernization_upgrade == 0
            || self.refit(u).is_some_and(|(gain, cost, _)| {
                self.gold_left - cost >= self.gold_reserve()
                    && i64::from(gain) * 1000 * i64::from(self.decisions.modernization_upgrade)
                        / (i64::from(cost.max(1)) * 100)
                        > self.modernization_alternative()
            })
    }

    /// Gene `upgrade_wait`: an upgrade takes all the unit's moves, so a unit that has an
    /// attack the military passes would take this turn (a profitable shot, strike or
    /// counterattack on a visible enemy) attacks first and upgrades on a later turn.
    pub(crate) fn upgrade_waits_for_attack(&self, u: &UnitView) -> bool {
        // Gene `strike_upgrade_wait`: a unit the running strike counts on keeps its moves.
        if self.decisions.strike_upgrade_wait > 0 && u.military && self.in_strike(u) {
            return true;
        }
        if self.decisions.upgrade_wait == 0 || !u.military {
            return false;
        }
        // Both read only hostile units and cities (`ctx.enemy_units`, `ctx.enemy_cities`).
        if u.ranged > 0 {
            return self.best_shot(u, &Default::default()).is_some();
        }
        self.best_strike(u).is_some() || self.counterattack_target(u, None).is_some()
    }

    /// Gene `strike_upgrade_wait`: whether `u` is part of the strike the plan runs: the active
    /// campaign's committed force, a running operation's roster, or a unit that reaches an
    /// assault tile of the routed Campaign's city within the march window (the force its
    /// forecast priced). An upgrade takes all the unit's moves, so it would arrive a turn late.
    pub(crate) fn in_strike(&self, u: &UnitView) -> bool {
        if self
            .campaigns
            .active
            .as_ref()
            .is_some_and(|c| c.committed.contains(&u.id))
            || self.operation().is_some_and(|op| op.roster.contains(&u.id))
        {
            return true;
        }
        let v = self.view();
        // A unit near a home city with an enemy army at its door defends, not strikes: its
        // upgrade is a better defender, not a late arrival.
        if self.ctx.own_cities.iter().any(|c| {
            c.pos.distance(u.pos) <= HOME_DOOR
                && self.ctx.enemy_units.iter().any(|e| {
                    e.military
                        && !e.owner.is_barbarian()
                        && v.at_war(v.me, e.owner)
                        && e.pos.distance(c.pos) <= HOME_DOOR
                })
        }) {
            return false;
        }
        let Some(crate::plan::PlanKind::Campaign { target, .. }) =
            self.routed_plan().map(|p| &p.kind)
        else {
            return false;
        };
        let Some(city) = v
            .cities
            .iter()
            .find(|c| c.id == *target && !v.same_team(v.me, c.owner))
        else {
            return false;
        };
        let mut prospective;
        let view = if v.at_war(v.me, city.owner) {
            v
        } else {
            prospective = v.clone();
            if let Some(r) = prospective
                .diplomacy
                .relations
                .iter_mut()
                .find(|r| r.between(v.team, v.team_of(city.owner)))
            {
                r.state = legatum_core::diplomacy::RelationState::War;
            }
            &prospective
        };
        self.strike_force_within(view, city, self.decisions.march_turns)
            .iter()
            .any(|(f, _, _)| f.id == u.id)
    }

    /// Gene `capturer_upgrade`: a melee unit that can take a city (it charges) within the strike
    /// radius of a visible enemy city we are at war with: the unit a capture rests on.
    fn capturer(&self, u: &UnitView) -> bool {
        let v = self.view();
        self.decisions.capturer_upgrade > 0
            && u.military
            && composition::charges(u.kind)
            && self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| c.visible && v.at_war(v.me, c.owner) && c.pos.distance(u.pos) <= 4)
    }

    /// Spend before optional buildings only when the upgrade's combat gain per Gold wins.
    /// Keep the ordinary solvency reserve, account for supply in the emitted order list.
    pub(crate) fn plan_modernization(&mut self) {
        if self.decisions.modernization_upgrade == 0 {
            return;
        }
        let wave = if let Some(fund) = self.named_fund() {
            match fund.kind {
                crate::gold_hold::FundKind::Upgrade(tech)
                    if fund.ready && self.gold_left >= fund.cost =>
                {
                    Some(tech)
                }
                _ => return,
            }
        } else {
            None
        };
        // The mounted veteran is already paid for and keeps its promotions. Iron that can
        // equip it takes precedence over an idle Forge's future production.
        if wave.is_none() && self.mongol_horde() && self.decisions.mongol_horde >= 2 {
            let veterans: Vec<_> = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| self.mongol_keshig_upgrade(u))
                .collect();
            for u in veterans {
                let Some(cost) = u.upgrade_cost else { continue };
                if self.has_acted(u.id) || self.gold_left < cost || self.upgrade_waits_for_attack(u)
                {
                    continue;
                }
                let order = Order::UpgradeUnit { unit: u.id };
                let mut orders = self.orders.clone();
                orders.push(order.clone());
                if !crate::supply::within_budget(self.view(), orders).contains(&order) {
                    continue;
                }
                self.gold_left -= cost;
                self.mil_dest.insert(u.pos);
                self.unit_order(u.id, order);
            }
        }
        let alternative = self.modernization_alternative();
        let mut candidates: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter_map(|u| {
                let (gain, cost, _) = self.refit(u)?;
                if wave.is_some_and(|tech| {
                    data::unit_stats(u.kind)
                        .upgrades_to
                        .and_then(|to| data::unit_stats(to).requires)
                        != Some(tech)
                }) {
                    return None;
                }
                // The view prices only an upgrade affordable now; the captor's is saved for.
                let immediate = u.upgrade_cost.or(self.capturer(u).then_some(cost))?;
                let score =
                    i64::from(gain) * 1000 * i64::from(self.decisions.modernization_upgrade)
                        / (i64::from(cost.max(1)) * 100);
                let score = if self.capturer(u) {
                    score * i64::from(100 + self.decisions.capturer_upgrade) / 100
                } else {
                    score
                };
                (wave.is_some() || score > alternative).then_some((score, u, immediate, cost))
            })
            .collect();
        candidates.sort_by_key(|(score, u, _, _)| (std::cmp::Reverse(*score), u.id));
        // Gene `capturer_upgrade`: the best refit that can be ordered is the captor's and Gold
        // comes in within the march window: keep its cost in hand rather than spend it on a
        // lesser refit or a later purchase (they respect `gold_reserve`).
        if wave.is_none() && self.decisions.capturer_upgrade > 0 {
            let income = self.view().economy.gold_per_turn.total.max(0);
            let orderable = |u: &UnitView| {
                let order = Order::UpgradeUnit { unit: u.id };
                let mut orders = self.orders.clone();
                orders.push(order.clone());
                !self.has_acted(u.id)
                    && !self.upgrade_waits_for_attack(u)
                    && crate::supply::within_budget(self.view(), orders).contains(&order)
            };
            if let Some((_, u, _, total)) = candidates.iter().find(|(_, u, _, _)| orderable(u))
                && self.capturer(u)
                && self.gold_left - total < self.gold_reserve()
                && self.gold_left + income * self.decisions.march_turns - total
                    >= self.gold_reserve()
            {
                self.modernization_savings += *total;
                return;
            }
        }
        for (_, u, immediate, total) in candidates {
            if self.has_acted(u.id)
                || (wave.is_some() && self.gold_left < total)
                || (wave.is_none()
                    && (self.gold_left - total < self.gold_reserve()
                        || !self.affordable_beyond_runway(total)))
                || self.upgrade_waits_for_attack(u)
            {
                continue;
            }
            let order = Order::UpgradeUnit { unit: u.id };
            let mut orders = self.orders.clone();
            orders.push(order.clone());
            if !crate::supply::within_budget(self.view(), orders).contains(&order) {
                continue;
            }
            self.gold_left -= immediate;
            self.modernization_savings += total - immediate;
            self.mil_dest.insert(u.pos);
            self.unit_order(u.id, order);
        }
    }

    /// After immediate attacks/defense, an obsolete staging unit can spend travel time on refit.
    pub(crate) fn return_for_modernization(&mut self, army: &[&UnitView]) {
        if self.decisions.modernization_return == 0 {
            return;
        }
        let mut budget = self.gold_left - self.gold_reserve();
        for u in army
            .iter()
            .copied()
            .filter(|u| {
                !self.has_acted(u.id)
                    || self
                        .orders
                        .iter()
                        .any(|o| matches!(o, Order::SkipUnit { unit } if *unit == u.id))
            })
            .collect::<Vec<_>>()
        {
            if self
                .view()
                .tile(u.pos)
                .is_some_and(|t| t.owner == Some(self.view().me))
            {
                continue;
            }
            let Some((gain, cost, steps)) = self.refit(u) else {
                continue;
            };
            if budget < cost || !self.affordable_beyond_runway(cost) {
                continue;
            }
            let before =
                self.modernization_power(u.kind, u.level, u.hp, u.unsupplied.is_some(), u.pos);
            let mut destinations: Vec<_> = self
                .view()
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.owner == Some(self.view().me))
                .map(|(i, _)| {
                    Hex::from_offset(i as i32 % self.view().width, i as i32 / self.view().width)
                })
                .filter(|to| self.free_tile_for(u, *to))
                .collect();
            destinations.sort_by_key(|to| (u.pos.distance(*to), *to));
            let target = destinations
                .into_iter()
                .take(8)
                .filter_map(|to| {
                    let path =
                        legatum_core::path::find_path(self.movement_map(), u.pos, to, |h| {
                            let i = self.view().index(h)?;
                            self.ctx.move_cost(h).filter(|_| {
                                !self.ctx.hostile_at[i]
                                    && !self.ctx.friendly_military_at[i]
                                    && !self.mil_dest.contains(&h)
                                    && self.unit_exposure_key(u, h).0
                                        <= self.unit_exposure_key(u, u.pos).0
                            })
                        })?;
                    let distance: i32 = path
                        .iter()
                        .map(|h| self.ctx.move_cost(*h).unwrap_or(1))
                        .sum();
                    let travel = (distance + u.max_moves.max(1) - 1) / u.max_moves.max(1);
                    let value = gain * self.decisions.modernization_return / 100 * 30
                        / (30 + travel * 2)
                        - before * (travel * 2 + steps) / 30;
                    (value > 0
                        && self.unit_exposure_key(u, to).0 <= self.unit_exposure_key(u, u.pos).0)
                        .then_some((value, std::cmp::Reverse(path.len()), to, path))
                })
                .max();
            if let Some((_, _, to, path)) = target {
                budget -= cost;
                self.release_modernization_hold(u.id);
                self.mil_dest.insert(to);
                self.unit_order(
                    u.id,
                    Order::MoveUnitPath {
                        unit: u.id,
                        to,
                        path,
                        stop_on_block: true,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::fixture;
    use std::io::Read;

    pub(crate) fn saved(player: u8) -> legatum_core::PlayerView {
        let mut state = saved_state();
        // The archived views predate direct-to-newest upgrade quotes. Rebuild the view from
        // the archived state with enough Gold to expose those quotes for this counterfactual.
        state.player_mut(legatum_core::PlayerId(player)).gold = 10_000;
        // The archived state predates the same-era reveals (rules version 22): an empire that knew
        // a resource under the old reveal techs keeps knowing it.
        use legatum_core::TechId::*;
        let techs = &mut state.player_mut(legatum_core::PlayerId(player)).techs;
        for (old, new) in [
            (Education, Gunpowder),
            (Metallurgy, Industrialization),
            (Industrialization, Combustion),
            (Electricity, AutonomousSystems),
        ] {
            if techs.contains(&old) {
                techs.insert(new);
            }
        }
        legatum_core::player_view(&state, legatum_core::PlayerId(player))
    }

    fn saved_state() -> legatum_core::GameState {
        let bytes = include_bytes!("../tests/fixtures/modernization-1026/t142-state.json.gz");
        let mut json: serde_json::Value =
            serde_json::from_reader(flate2::read::GzDecoder::new(&bytes[..])).unwrap();
        json["setup"]["seed"] = serde_json::json!(
            json["setup"]["seed"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        serde_json::from_value(json).unwrap()
    }

    pub(crate) fn saved_on(turn: u32, player: u8) -> legatum_core::PlayerView {
        let name = format!(
            "{}/tests/fixtures/modernization-1026/t{turn}-p{player}.json.gz",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut text = String::new();
        flate2::read::GzDecoder::new(std::fs::File::open(name).unwrap())
            .read_to_string(&mut text)
            .unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn modernization_save_sparta_is_resource_blocked_not_ancient_in_science() {
        let v = saved(4);
        let mut p = fixture::planner(&v, 1);
        p.decisions.modernization_upgrade = 100;
        assert!(v.research.known.contains(&legatum_core::TechId::Gunpowder));
        assert!(v.supply.iter().all(|s| s.spare <= 0));
        p.plan_modernization();
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::UpgradeUnit { .. }))
        );
    }

    #[test]
    fn modernization_square_makes_archer_tiny_against_field_cannon() {
        let mut v = saved(4);
        let pos = v.units.iter().find(|u| u.owner == v.me).unwrap().pos;
        let enemy = v
            .units
            .iter_mut()
            .find(|u| u.owner != v.me && composition::is_combat(u.kind))
            .unwrap();
        enemy.pos = pos;
        enemy.kind = UnitKind::FieldCannon;
        let stats = data::unit_stats(UnitKind::FieldCannon);
        enemy.strength = stats.strength;
        enemy.ranged = stats.ranged;
        enemy.level = 0;
        let id = enemy.id;
        v.units.retain(|u| u.owner == v.me || u.id == id);
        let p = fixture::planner(&v, 1);
        assert!(p.modernization_percent(UnitKind::Archer, pos) < 10);
        assert!(
            p.modernization_power(UnitKind::FieldCannon, 1, 100, false, pos)
                > 10 * p.modernization_power(UnitKind::Archer, 1, 100, false, pos)
        );
    }

    #[test]
    fn modernization_save_byzantium_funds_upgrades_before_optional_spending() {
        let v = saved(3);
        let mut p = fixture::planner(&v, 1);
        p.decisions.modernization_upgrade = 100;
        p.plan_modernization();
        let upgraded: Vec<_> = p
            .orders
            .iter()
            .filter_map(|o| {
                if let Order::UpgradeUnit { unit } = o {
                    Some(*unit)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            !upgraded.is_empty(),
            "Gold {}, reserve {}, alternative {}",
            p.gold_left,
            p.gold_reserve(),
            p.modernization_alternative()
        );
        assert!(upgraded.iter().all(|id| {
            v.units
                .iter()
                .any(|u| u.id == *id && u.upgrade_cost.is_some())
        }));
        assert_eq!(crate::supply::within_budget(&v, p.orders.clone()), p.orders);
    }

    #[test]
    fn modernization_zero_keeps_orders_and_gold_untouched() {
        for seat in [2, 3, 4, 5] {
            let v = saved(seat);
            let mut p = fixture::planner(&v, 1);
            p.decisions.modernization_upgrade = 0;
            p.decisions.modernization_return = 0;
            let gold = p.gold_left;
            p.plan_modernization();
            let army = p.ctx.own_units.clone();
            p.return_for_modernization(&army);
            assert!(p.orders.is_empty());
            assert_eq!(gold, p.gold_left);
        }
    }

    #[test]
    fn modernization_phalanx_can_make_first_upgrade_a_loss_but_keeps_veteran_chain() {
        let mut v = saved(4);
        let unit = v
            .units
            .iter()
            .find(|u| u.owner == v.me && u.kind == UnitKind::Spearman)
            .unwrap()
            .clone();
        let neighbor = v
            .units
            .iter_mut()
            .find(|u| u.owner == v.me && u.kind == UnitKind::Spearman && u.id != unit.id)
            .unwrap();
        neighbor.pos = unit.pos.neighbors()[0];
        let p = fixture::planner(&v, 1);
        let hoplite = p.modernization_power(UnitKind::Spearman, 0, 100, false, unit.pos);
        let phalangite = p.modernization_power(UnitKind::Phalangite, 0, 100, false, unit.pos);
        // With the class bonus a percentage of strength (2026-10-31) the Phalangite's larger bonus
        // against riders edges it ahead, but the phalanx keeps a Hoplite close.
        assert!(
            hoplite * 100 >= phalangite * 85,
            "{hoplite} vs {phalangite}"
        );
        assert!(
            p.modernization_power(UnitKind::Musketman, unit.level, 100, false, unit.pos) > hoplite
        );
    }
    #[test]
    fn modernization_return_replaces_staging_hold_with_legal_home_route() {
        let mut v = saved(3);
        v.economy.gold = 1500;
        let id = v
            .units
            .iter()
            .find(|u| u.owner == v.me && u.kind == UnitKind::Pikeman)
            .unwrap()
            .id;
        let home = v.units.iter().find(|u| u.id == id).unwrap().pos;
        let away = home
            .neighbors()
            .into_iter()
            .find(|h| v.index(*h).is_some() && !v.units.iter().any(|u| u.pos == *h))
            .unwrap();
        let index = v.index(away).unwrap();
        v.tiles[index].owner = None;
        v.units.iter_mut().find(|u| u.id == id).unwrap().pos = away;
        v.units.retain(|u| u.owner == v.me);
        let mut p = fixture::planner(&v, 1);
        p.decisions.modernization_return = 100;
        p.unit_order(id, Order::SkipUnit { unit: id });
        let u = p
            .ctx
            .own_units
            .iter()
            .find(|u| u.id == id)
            .copied()
            .unwrap();
        p.return_for_modernization(&[u]);
        assert!(
            p.orders.iter().any(
                |o| matches!(o,Order::MoveUnit{unit,..}|Order::MoveUnitPath{unit,..} if *unit==id)
            ),
            "{:?}",
            p.orders
        );
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o,Order::SkipUnit{unit} if *unit==id))
        );
    }

    #[test]
    fn modernization_save_full_brains_emit_legal_orders_at_both_difficulties() {
        use crate::Brain;
        let state = saved_state();
        for difficulty in [
            legatum_core::AiDifficulty::Normal,
            legatum_core::AiDifficulty::Hard,
        ] {
            for seat in [2, 3, 4, 5] {
                let view = legatum_core::player_view(&state, legatum_core::PlayerId(seat));
                for enabled in [0, 100] {
                    let mut brain = crate::ScriptedBrain::with_difficulty(
                        view.faction_of(view.me),
                        1,
                        view.me,
                        difficulty,
                    );
                    brain.decisions.modernization_value = enabled;
                    brain.decisions.modernization_upgrade = enabled;
                    brain.decisions.modernization_return = enabled;
                    let orders = brain.orders(&view);
                    let errors = legatum_core::validate_orders(&state, view.me, &orders);
                    assert!(
                        errors.is_empty(),
                        "seat {seat}, {difficulty:?}, gene {enabled}: {errors:?}"
                    );
                }
            }
        }
    }

    /// Gene `upgrade_wait`: a unit with a strike this turn attacks first and upgrades later.
    #[test]
    fn upgrade_wait_leaves_a_unit_with_a_strike_to_the_attack() {
        let upgraded = |v: &legatum_core::PlayerView, gene: i32| -> Vec<legatum_core::UnitId> {
            let mut p = fixture::planner(v, 1);
            p.decisions.modernization_upgrade = 100;
            p.decisions.upgrade_wait = gene;
            p.plan_modernization();
            p.orders
                .iter()
                .filter_map(|o| match o {
                    Order::UpgradeUnit { unit } => Some(*unit),
                    _ => None,
                })
                .collect()
        };
        let mut v = saved(3);
        let first = upgraded(&v, 0);
        let p = fixture::planner(&v, 1);
        let (unit, pos) = first
            .iter()
            .find_map(|id| {
                let u = p
                    .ctx
                    .own_units
                    .iter()
                    .find(|u| u.id == *id && u.ranged == 0)?;
                Some((u.id, u.pos))
            })
            .expect("a melee upgrade");
        let enemy = p.ctx.enemy_units.first().expect("a hostile unit").id;
        let spot = pos
            .neighbors()
            .into_iter()
            .find(|h| {
                v.tile(*h).is_some_and(|t| t.move_cost().is_some())
                    && !v.units.iter().any(|u| u.pos == *h)
                    && !v.cities.iter().any(|c| c.pos == *h)
            })
            .expect("a free land tile beside it");
        drop(p);
        let e = v.units.iter_mut().find(|u| u.id == enemy).unwrap();
        e.pos = spot;
        e.hp = 5;
        assert!(
            upgraded(&v, 0).contains(&unit),
            "gene off: upgrades as before"
        );
        assert!(
            !upgraded(&v, 1).contains(&unit),
            "gene on: the strike comes first"
        );
    }
}
