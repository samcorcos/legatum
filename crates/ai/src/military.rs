//! The army: ranged fire, defense of threatened cities, garrisons, attacks when the local
//! strength ratio beats the aggression threshold, settler escorts, campaigns on enemy cities,
//! and resting.

use crate::analysis::{
    UNREACHABLE, attack_power, city_power, defense_power, forecast_moves, offense_power,
    threat_power,
};
use crate::scripted::Planner;
use legatum_core::data;
use legatum_core::path::find_path;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{
    AttackMode, Faction, Hex, Order, TeamId, Terrain, UnitKind, forecast_attack_mode,
    forecast_city_strike,
};

/// Army units this far from a threatened city are called back to defend it.
pub(crate) const DEFEND_RADIUS: i32 = 6;
/// Units within this distance of a target count toward the local strength ratio.
const STRIKE_RADIUS: i32 = 3;
/// Idle units this far from a city come back to it.
const HOME_RADIUS: i32 = 2;
/// Wounded units below this HP don't march on enemy cities.
pub(crate) const MARCH_MIN_HP: i32 = 60;
/// Empires below this aggression never start campaigns (they still strike nearby targets).
pub(crate) const CAMPAIGN_MIN_AGGRESSION: i32 = 30;
struct Target {
    pos: Hex,
    /// Defensive weight at the target, including nearby support.
    their: i32,
    /// A lone civilian: capture it with one melee unit.
    civilian: bool,
    /// A visible Heir held by someone else: capturing him swings Byzantium's goal VP, so he
    /// is struck first and on thinner margins than any other civilian.
    heir: bool,
}

#[cfg(test)]
mod fund_delivery_tests {
    use super::*;
    use crate::{
        Knobs,
        campaign::FundDelivery,
        fixture::{ENEMY, ME, ViewBuilder},
    };
    use legatum_core::{AiDifficulty, CityId, PlayerId, UnitId};

    #[test]
    fn newly_bought_emergency_unit_heads_for_the_named_city() {
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(4, 6), 3)
            .city(2, ME, Hex::new(10, 6), 3)
            .unit(20, ME, UnitKind::Pikeman, Hex::new(4, 6))
            .build();
        view.turn = 11;
        view.units
            .iter_mut()
            .find(|u| u.id == UnitId(20))
            .unwrap()
            .created_turn = Some(10);
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(PlayerId(0))),
            view.faction_of(PlayerId(0)),
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.campaigns.fund_deliveries.push(FundDelivery {
            ordered_turn: 10,
            bought_in: Some(CityId(1)),
            kind: Some(UnitKind::Pikeman),
            unit: None,
            destination: CityId(2),
            offensive: false,
        });
        p.deliver_fund_units();
        assert_eq!(p.campaigns.fund_deliveries[0].unit, Some(UnitId(20)));
        assert!(p.orders.iter().any(|o| matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } if *unit == UnitId(20))));
    }

    #[test]
    fn newly_bought_campaign_unit_marches_to_its_target() {
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(4, 6), 3)
            .city(3, ENEMY, Hex::new(12, 6), 3)
            .unit(20, ME, UnitKind::Pikeman, Hex::new(4, 6))
            .build();
        view.turn = 11;
        view.units
            .iter_mut()
            .find(|u| u.id == UnitId(20))
            .unwrap()
            .created_turn = Some(10);
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(ME)),
            view.faction_of(ME),
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.campaigns.fund_deliveries.push(FundDelivery {
            ordered_turn: 10,
            bought_in: Some(CityId(1)),
            kind: Some(UnitKind::Pikeman),
            unit: None,
            destination: CityId(3),
            offensive: true,
        });
        p.deliver_fund_units();
        assert_eq!(p.campaigns.fund_deliveries[0].unit, Some(UnitId(20)));
        assert!(p.orders.iter().any(|o| matches!(o, Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. } if *unit == UnitId(20))));
    }
}

impl Planner<'_> {
    /// A city's only unit is the last occupation barrier when an enemy is at its door.
    /// Offensive planners price that unit as unavailable until the adjacent threat clears.
    pub(crate) fn last_garrison_under_attack(&self, u: &UnitView) -> bool {
        self.ctx.own_cities.iter().any(|c| {
            c.pos == u.pos
                && self.ctx.threat_units.iter().any(|e| {
                    e.military
                        && e.pos.distance(c.pos) <= 1
                        && self.view().at_war(self.view().me, e.owner)
                })
        })
    }

    /// Sparta regiments never heal by resting, so a wounded one marches and fights down to
    /// the Sparta recovery floor instead of waiting at home for HP that never comes.
    pub(crate) fn march_min_hp(&self) -> i32 {
        if self.faction == Faction::Sparta {
            self.decisions.sparta_march_min_hp
        } else {
            MARCH_MIN_HP
        }
    }

    pub fn plan_military(&mut self) {
        let army: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .collect();
        // Gene `operations`: a committed operation orders its roster before any other pass.
        self.pass("execute_operation");
        self.execute_operation();
        // The capitals victory: a rival in reach of our capital keeps its garrison home.
        self.pass("secure_capital");
        self.secure_capital(&army);
        self.pass("gather_veterans");
        self.gather_veterans();
        self.pass("protect_heirs");
        self.protect_heirs();
        self.pass("plan_primitive_escorts");
        let primitive_escorts = if self.settler_safety() {
            false
        } else {
            self.plan_primitive_escorts()
        };
        self.pass("plan_primitive_snipes");
        self.plan_primitive_snipes();
        self.pass("plan_primitive_coalition_march");
        self.plan_primitive_coalition_march();
        self.pass("plan_primitive_barbarian_guards");
        self.plan_primitive_barbarian_guards();
        // Gene `contest_hold`: removing a rival's public VP source that breaks its hold goes
        // first, priced against what the units it takes would do otherwise (`contest.rs`).
        self.pass("contest_vp_sources");
        self.contest_vp_sources(&army);
        self.pass("hold_captures");
        self.hold_captures(&army);
        self.pass("withdraw_failed_campaign");
        self.withdraw_failed_campaign(&army);
        // Gene `door_defense`: strikes on the attackers at our cities, garrisons for empty
        // threatened cities and retaking a weakened lost city come before any withdrawal.
        self.pass("door_military");
        self.door_military(&army);
        // A Roman recruit adjacent to a favourable fight can earn its first
        // promotion now. Leave distant marches to the later field pass.
        if self.faction == Faction::Rome {
            self.pass("drill_nearby_recruits");
            self.drill_veterans(&army, false);
        }
        // Gene `breach`: a city our shots can zero this turn is entered the same turn.
        self.pass("breach_cities");
        self.breach_cities(&army);
        self.pass("leave_strike_range");
        self.leave_strike_range(&army);
        self.pass("press_strike");
        self.press_strike(&army);
        self.pass("retake_weakened_city");
        self.retake_weakened_city(&army);
        self.pass("scout_threat_front");
        self.scout_threat_front(&army);
        self.pass("sparta_nearby_sortie");
        self.sparta_nearby_sortie(&army);
        self.pass("sparta_followup_sortie");
        self.sparta_followup_sortie(&army);
        self.pass("reserve_front_guards");
        self.reserve_front_guards(&army);
        self.pass("guard_exposed_settlers");
        self.guard_exposed_settlers(&army);
        self.pass("sortie_garrisons");
        self.sortie_garrisons(&army);
        self.pass("occupy_rush_city");
        self.occupy_rush_city(&army);
        if self.decisions.mongol_horde >= 2 && self.mongol_horde() {
            self.pass("mongol_veteran_recovery");
            for u in self.free(&army) {
                if u.kind == UnitKind::Cataphract
                    && u.ranged > 0
                    && u.hp < 80
                    && self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.military && e.pos.distance(u.pos) == 1)
                    && self.exposure_at(u.pos) > 0
                    && let Some(to) = self.safer_tile(u)
                {
                    self.go_to(u, to);
                    self.retreats.insert(u.id);
                }
            }
        }
        self.pass("coordinate_city_assaults");
        self.coordinate_city_assaults(&army);
        // Gene `siege_stance`: shoot a city down that the shooters can zero within a few turns.
        self.pass("siege_stance_cities");
        self.siege_stance_cities(&army);
        if self.faction != Faction::Sparta || self.decisions.siege_storm_first == 0 {
            self.pass("siege_war");
            self.siege_war(&army, true);
        }
        self.pass("recover_city_fire");
        self.recover_city_fire(&army);
        self.pass("recover_wounded");
        self.recover_wounded(&army);
        // Gene `focus_fire`: joint kill plans take their units before single-unit attacks.
        self.pass("focus_fire");
        self.focus_fire(&army);
        self.pass("ranged_fire");
        self.ranged_fire(&army);
        // Coordinated fire takes favourable kills first; surviving Roman recruits
        // then seek their own first promotion before routine garrisons park them.
        self.pass("drill_veterans");
        self.drill_veterans(&army, true);
        self.pass("rush_siege");
        self.rush_siege(&army);
        self.pass("counterattack");
        self.counterattack(&army);
        self.pass("attack_targets");
        self.attack_targets(&army, true);
        if self.rushing()
            || self.orders.iter().any(|o| {
                matches!(o, Order::RangedAttack { target, .. }
            if self.ctx.enemy_cities.iter().any(|c| c.pos == *target))
            })
        {
            // Committed fire supports a coordinated approach for every policy. Otherwise
            // exposure avoidance sends the melee wing home immediately after its volley.
            // Wounded recovery still has first priority.
            self.pass("attack_targets#2");
            self.attack_targets(&army, false);
        }
        // Let Sparta's sorties, rush siege and coordinated assault take their units first.
        // The early siege then moves only unused shooters toward a running campaign.
        if self.faction == Faction::Sparta && self.decisions.siege_storm_first > 0 {
            self.pass("siege_war#2");
            self.siege_war(&army, true);
        }
        self.pass("answer_bombardment");
        self.answer_bombardment(&army);
        self.pass("avoid_free_damage");
        self.avoid_free_damage(&army);
        if !primitive_escorts && self.opening.is_some() && !self.settler_safety() {
            self.pass("plan_opening_escorts");
            self.plan_opening_escorts();
        }
        self.pass("rush_reconnaissance");
        self.rush_reconnaissance(&army);
        self.pass("deliver_fund_units");
        self.deliver_fund_units();
        self.pass("defend_cities");
        self.defend_cities(&army);
        self.pass("seize_open_city");
        self.seize_open_city(&army);
        // Gene `war_siege`: after home defence, every war the plan fights sieges a city or
        // strikes field units before the campaign's safe staging parks the army.
        if self.faction != Faction::Sparta || self.decisions.siege_storm_first == 0 {
            self.pass("siege_war#3");
            self.siege_war(&army, false);
        }
        // Funded campaigns and sworn regiments leave after emergency home defense.
        // A routine empty-city garrison must not consume the assault wing.
        if (self.stages_campaigns()
            && matches!(
                self.routed_plan().map(|p| &p.kind),
                Some(crate::plan::PlanKind::Campaign { .. })
            ))
            || self.conquest_city().is_some()
            || self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| self.sparta_open_followup(c))
            || self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| self.sparta_chain_target(c))
            || (self.faction == Faction::Sparta
                && self.view().turn <= legatum_core::data::FIRST_THIRD_TURNS
                && self
                    .sparta_iron_target()
                    .is_some_and(|team| self.view().teams_at_war(self.view().team, team)))
        {
            self.pass("campaign");
            self.campaign(&army);
        }
        // A storm or staged expedition owns its melee wing first. Unused troops still
        // answer the war's siege objective after the campaign pass.
        if self.faction == Faction::Sparta && self.decisions.siege_storm_first > 0 {
            self.pass("siege_war#3");
            self.siege_war(&army, false);
        }
        self.pass("hold_captures_late");
        self.hold_captures_late(&army);
        self.pass("collect_ruins");
        self.collect_ruins(&army);
        self.pass("commit_idle_defenders");
        self.commit_idle_defenders(&army);
        self.pass("garrison_cities#2");
        self.garrison_cities(&army);
        self.pass("attack_targets#3");
        self.attack_targets(&army, false);
        if !primitive_escorts && self.opening.is_none() && !self.hard_defense() {
            self.pass("escort_settlers");
            self.escort_settlers(&army);
        }
        if self.rushing() {
            self.pass("campaign#2");
            self.campaign(&army);
        }
        self.pass("clear_barbarian_camps");
        self.clear_barbarian_camps(&army);
        // A closed India's held vow is broken by one unit that stays alive inside its land.
        self.pass("campaign#3");
        self.campaign(&army);
        self.pass("military_scout");
        self.military_scout(&army);
        self.pass("scout_front");
        self.scout_front(&army);
        self.pass("dual_fire");
        self.dual_fire(&army);
        self.pass("rest");
        self.rest(&army);
        // Gene `gather_hold`: stragglers stop short of the enemy's reach.
        self.pass("hold_stragglers");
        self.hold_stragglers(&army);
        // Holds come from many phases (an early Mongols garrison pass, defense, rest);
        // none of them is better than a shot from the same tile.
        // Gene `shot_sweep`: a shooter ordered to move kills from where it stands instead.
        self.pass("shot_sweep");
        self.shot_sweep();
        self.pass("fire_from_holds");
        self.fire_from_holds();
        self.pass("return_for_modernization");
        self.return_for_modernization(&army);
    }

    /// A bought unit does not exist while its purchase is planned. Bind it on the
    /// following view, then march it to the fund's named front before routine
    /// garrison and idle passes can park it at the purchase city.
    fn deliver_fund_units(&mut self) {
        let pending = std::mem::take(&mut self.campaigns.fund_deliveries);
        let mut kept = Vec::new();
        for mut delivery in pending {
            if self.view().turn > delivery.ordered_turn.saturating_add(20) {
                continue;
            }
            let Some(target) = self
                .view()
                .cities
                .iter()
                .find(|c| c.id == delivery.destination)
            else {
                kept.push(delivery);
                continue;
            };
            if delivery.offensive == self.view().same_team(self.view().me, target.owner) {
                continue;
            }
            if delivery.unit.is_none()
                && let (Some(city), Some(kind)) = (delivery.bought_in, delivery.kind)
                && let Some(home) = self.ctx.own_cities.iter().find(|c| c.id == city)
            {
                delivery.unit = self
                    .ctx
                    .own_units
                    .iter()
                    .copied()
                    .find(|u| {
                        u.kind == kind
                            && u.created_turn == Some(delivery.ordered_turn)
                            && u.pos.distance(home.pos) <= 1
                    })
                    .map(|u| u.id);
            }
            let Some(unit) = delivery.unit else {
                if self.view().turn <= delivery.ordered_turn.saturating_add(1) {
                    kept.push(delivery);
                }
                continue;
            };
            let Some(u) = self.ctx.own_units.iter().copied().find(|u| u.id == unit) else {
                continue;
            };
            let close = if delivery.offensive { 3 } else { 1 };
            if u.pos.distance(target.pos) <= close {
                continue;
            }
            if !self.has_acted(unit) {
                let to = if delivery.offensive {
                    self.campaign_staging_spot(u, target.pos, 3)
                        .or_else(|| self.campaign_staging_spot(u, target.pos, 2))
                } else if self.free_tile_for(u, target.pos) {
                    Some(target.pos)
                } else {
                    target
                        .pos
                        .ring(1)
                        .into_iter()
                        .filter(|h| self.free_tile_for(u, *h))
                        .min_by_key(|h| (h.distance(u.pos), *h))
                };
                if let Some(to) = to {
                    self.go_to(u, to);
                }
            }
            kept.push(delivery);
        }
        self.campaigns.fund_deliveries = kept;
    }

    /// A city we just took (still in resistance), or a damaged one with enemies about, is the
    /// front. Keep a defender in it before anything marches elsewhere, preferring a ranged
    /// unit that reaches it this turn: from the city it shoots every turn without
    /// retaliation. (rush-defense, user run 2026-10-04: Mongols took Hymnal on T23, sent
    /// every unit to stage against a neutral city, and lost the empty city back on T29.)
    pub(crate) fn hold_captures(&mut self, army: &[&UnitView]) {
        self.hold_captures_at(army, false);
    }

    /// Gene `capture_hold`: the front cities the first pass left for later take their holder
    /// from the units the attack passes did not use (`army_commit.rs`).
    pub(crate) fn hold_captures_late(&mut self, army: &[&UnitView]) {
        if self.decisions.capture_hold > 0 {
            self.capture_deferred.clear();
            self.hold_captures_at(army, true);
        }
    }

    fn hold_captures_at(&mut self, army: &[&UnitView], late: bool) {
        let early_gate = !late && self.decisions.capture_hold > 0;
        let fronts: Vec<&CityView> = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(i, c)| {
                let resistance = c.detail.as_ref().is_some_and(|d| d.resistance > 0);
                let contested = self.ctx.threat[*i] > 0
                    || self.ctx.enemy_units.iter().any(|e| {
                        e.military && !e.owner.is_barbarian() && e.pos.distance(c.pos) <= 6
                    })
                    || self
                        .ctx
                        .remembered_threats
                        .iter()
                        .any(|o| !o.owner.is_barbarian() && o.pos.distance(c.pos) <= 6);
                resistance || (c.hp < c.max_hp && contested)
            })
            .map(|(_, c)| *c)
            .collect();
        for city in fronts {
            if self.mil_dest.contains(&city.pos) {
                continue;
            }
            if let Some(u) = self.free(army).into_iter().find(|u| u.pos == city.pos) {
                if !early_gate || self.capture_hold_now(city, u) {
                    self.hold_or_upgrade(u);
                } else {
                    self.capture_deferred.push(city.pos);
                }
                continue;
            }
            if self
                .view()
                .index(city.pos)
                .is_some_and(|i| self.ctx.friendly_military_at[i])
            {
                continue;
            }
            let holder = self
                .free(army)
                .into_iter()
                .filter(|u| !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
                .filter(|u| u.pos.distance(city.pos) <= 3 && self.can_defend_at(u, city.pos))
                .min_by_key(|u| {
                    (
                        !self.reaches_this_turn(u, city.pos),
                        u.ranged == 0,
                        u.pos.distance(city.pos),
                        u.id,
                    )
                });
            if let Some(u) = holder {
                if !early_gate || self.capture_hold_now(city, u) {
                    self.go_to(u, city.pos);
                } else {
                    self.capture_deferred.push(city.pos);
                }
            }
        }
    }

    /// An enemy city with nobody guarding it is an opening any war should take, whatever
    /// the plan (playtest 8.2: four AIs at war with the human for 12-25 turns never moved on
    /// its five empty, damaged cities). Hard only, after home defence has its units: the
    /// free field units within reach of the most open city at war, the runaway's first, go
    /// at it together.
    pub(crate) fn seize_open_city(&mut self, army: &[&UnitView]) {
        if !self.hard_defense() || self.takes_no_cities() {
            return;
        }
        let v = self.view();
        let min_hp = self.march_min_hp();
        let free: Vec<&UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| u.hp >= min_hp && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
            .collect();
        let runaway = self.top_threat_team();
        // A containment war is fought against its target: march farther for its cities.
        let contained = match self.routed_plan().map(|p| &p.kind) {
            Some(crate::plan::PlanKind::Contain { against }) => Some(*against),
            _ => None,
        };
        // (not the runaway's, city HP, nearest distance, tile) and the units that would go.
        type Opening<'u> = ((bool, i32, i32, Hex), Vec<&'u UnitView>);
        let mut best: Option<Opening> = None;
        let mut capture_rows: Vec<(legatum_core::CityId, Option<i32>, Option<&'static str>)> =
            Vec::new();
        for c in &self.ctx.enemy_cities {
            if !c.visible || c.owner.is_barbarian() || !v.at_war(v.me, c.owner) {
                if self.decision_trace.is_some() {
                    capture_rows.push((
                        c.id,
                        None,
                        Some(if !c.visible {
                            "not currently visible"
                        } else if c.owner.is_barbarian() {
                            "barbarian city"
                        } else {
                            "not at war with owner"
                        }),
                    ));
                }
                continue;
            }
            // Open: no soldier inside or beside it, and either it doesn't fire (no Walls) or
            // it's already half broken.
            let guarded = self.ctx.enemy_units.iter().any(|e| {
                e.military
                    && (e.kind != UnitKind::Scout || self.decisions.scout_threat > 0)
                    && e.pos.distance(c.pos) <= 1
            });
            // Gene `siege_stance`: walls that still fire are a siege for the shooters near
            // them when those (and our aircraft) can zero the city within the gene's turns.
            let walls_up = legatum_core::city_fires(c)
                && c.hp * 2 > c.max_hp
                && (self.decisions.siege_stance == 0
                    || !self.siege_within_reach(
                        c,
                        &free
                            .iter()
                            .copied()
                            .filter(|u| {
                                u.pos.distance(c.pos) <= self.decisions.open_city_radius
                                    && self.ctx.reachable(u.pos, c.pos)
                            })
                            .collect::<Vec<_>>(),
                    ));
            if guarded || walls_up {
                if self.decision_trace.is_some() {
                    capture_rows.push((
                        c.id,
                        None,
                        Some(if guarded {
                            "adjacent garrison"
                        } else {
                            "walls still fire at more than half HP"
                        }),
                    ));
                }
                continue;
            }
            let near: Vec<&UnitView> = free
                .iter()
                .copied()
                .filter(|u| {
                    let radius = if contained == Some(v.team_of(c.owner)) {
                        self.decisions.contain_march_radius
                    } else {
                        self.decisions.open_city_radius
                    };
                    u.pos.distance(c.pos) <= radius && self.ctx.reachable(u.pos, c.pos)
                })
                .collect();
            if !near.iter().any(|u| crate::composition::charges(u.kind)) {
                if self.decision_trace.is_some() {
                    capture_rows.push((
                        c.id,
                        None,
                        Some("no free charger within open-city radius"),
                    ));
                }
                continue;
            }
            // Enough to break it: our attack weight against the city's (it has no garrison).
            if self.attack_force(&near) < city_power(v, c) {
                if self.decision_trace.is_some() {
                    capture_rows.push((c.id, None, Some("attack force below city power")));
                }
                continue;
            }
            let key = (
                runaway != Some(v.team_of(c.owner)),
                c.hp,
                near.iter()
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(0),
                c.pos,
            );
            if self.decision_trace.is_some() {
                // The comparator is lexicographic. This numeric projection preserves its first
                // three fields; position is the deterministic final tie-breaker.
                capture_rows.push((
                    c.id,
                    Some(-i32::from(key.0) * 1_000_000 - key.1 * 1_000 - key.2),
                    None,
                ));
            }
            if best.as_ref().is_none_or(|(k, _)| key < *k) {
                best = Some((key, near));
            }
        }
        if self.decision_trace.is_some() {
            let winner_id = best
                .as_ref()
                .and_then(|(_, near)| near.first())
                .and_then(|_| {
                    self.ctx
                        .enemy_cities
                        .iter()
                        .find(|c| Some(c.pos) == best.as_ref().map(|(k, _)| k.3))
                        .map(|c| c.id)
                });
            let mut scores: Vec<i32> = capture_rows
                .iter()
                .filter_map(|(_, score, _)| *score)
                .collect();
            scores.sort_unstable_by(|a, b| b.cmp(a));
            for (id, score, gate) in capture_rows {
                let mut row = crate::DecisionOption::new(
                    "capture",
                    format!("city={id:?}"),
                    if gate.is_some() {
                        "gated_out"
                    } else if Some(id) == winner_id {
                        "selected"
                    } else {
                        "considered_rejected"
                    },
                );
                row.score = score;
                row.winner_score = scores.first().copied();
                row.runner_up_score = scores.get(1).copied();
                row.gate = gate.map(str::to_owned);
                self.note_option(row);
            }
        }
        let Some(((_, _, _, target), near)) = best else {
            return;
        };
        // Gene `siege_stance`: against walls that still fire, the melee holds out of their strike.
        let siege = self.decisions.siege_stance > 0
            && self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| c.pos == target && legatum_core::city_fires(c) && c.hp > 0);
        let before = self.orders.len();
        for u in near {
            if self.has_acted(u.id) {
                continue;
            }
            if siege && !(u.ranged > 0 && u.range > 0) {
                self.siege_hold(u, target);
            } else {
                self.strike(u, target, false);
            }
        }
        if let Some(trace) = &self.decision_trace
            && let Some(row) = trace
                .borrow_mut()
                .iter_mut()
                .rev()
                .find(|r| r.domain == "capture" && r.outcome == "selected")
        {
            row.selected_order = self.orders.get(before).cloned();
        }
    }

    /// Sparta with Honour the Fallen pending (gene `veteran_drill`): the units nearest the
    /// retire level are the ones that fight. Each candidate (the most experienced, below the
    /// level) attacks an enemy in reach when the forecast favours it, after this turn's volleys
    /// so it takes the kill; with nothing in reach and no threat at home it marches on the
    /// nearest barbarian unit or camp within 8 hexes (user, 2026-10-07: "the AI just needs to
    /// be better at getting units more experience").
    fn drill_veterans(&mut self, army: &[&UnitView], seek: bool) {
        let roman = self.faction == Faction::Rome && self.decisions.roman_founders > 0;
        let count = if roman {
            4
        } else {
            self.decisions.veteran_drill
        };
        if count <= 0 || !(self.faction == Faction::Sparta || roman) || !self.hard_defense() {
            return;
        }
        let v = self.view();
        if v.faction_goal.as_ref().is_some_and(|g| g.done) {
            return;
        }
        let level = if roman {
            1
        } else {
            legatum_core::goals::SPARTA_RETIRE_LEVEL
        };
        let mut candidates: Vec<&UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| {
                crate::composition::is_melee_line(u.kind)
                    && u.level < level
                    && (roman || u.level >= 1)
                    && (!roman || self.roman_unit_can_leave(u))
            })
            .collect();
        candidates.sort_by_key(|u| (std::cmp::Reverse(u.xp.unwrap_or(0)), u.id));
        candidates.truncate(count as usize);
        let quiet = self.ctx.threat.iter().all(|t| *t == 0);
        // Gene `roman_hunt_radius`: an XP source far from home is a recruit lost to the next
        // strike and a founding site far from support; only nearby land is worth walking to.
        let hunt_radius = if roman {
            self.decisions.roman_hunt_radius
        } else {
            0
        };
        let homes: Vec<Hex> = self.ctx.own_cities.iter().map(|c| c.pos).collect();
        let home_distance = |h: Hex| homes.iter().map(|c| c.distance(h)).min().unwrap_or(0);
        let campaign_planned = matches!(
            self.routed_plan().map(|p| &p.kind),
            Some(crate::plan::PlanKind::Campaign { .. })
        ) || self.conquest_city().is_some();
        for u in candidates {
            let target = self
                .ctx
                .enemy_units
                .iter()
                .filter(|e| e.military && !self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos))
                .map(|e| e.pos)
                .chain(
                    self.ctx
                        .enemy_cities
                        .iter()
                        .filter(|c| roman && v.at_war(v.me, c.owner))
                        .map(|c| c.pos),
                )
                .filter(|h| u.pos.distance(*h) <= u.max_moves.max(1))
                .filter_map(|h| {
                    let f = self.melee_forecast(u, h)?;
                    (f.win_chance >= (if roman { 50 } else { 60 })
                        && f.loss_chance <= (if roman { 35 } else { 20 }))
                    .then_some((f.win_chance, h))
                })
                .max_by_key(|(w, h)| (*w, std::cmp::Reverse(*h)))
                .map(|(_, h)| h);
            if let Some(to) = target
                && self.reaches_this_turn(u, to)
            {
                self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
                continue;
            }
            if !seek {
                continue;
            }
            // Gene `roman_hunt_radius`: a planned campaign is Rome's surest promotion (city
            // attacks and the capture pay XP) and needs the army together, so the hunt leaves
            // these recruits to it.
            if roman && hunt_radius > 0 && campaign_planned {
                continue;
            }
            // A barbarian on the map makes the shared threat field nonempty. Rome still
            // needs to seek that fight to earn its first founder, unless a nearby city
            // needs this unit for defence.
            if !quiet && (!roman || self.defending(u.pos)) {
                continue;
            }
            let prey = self
                .ctx
                .enemy_units
                .iter()
                .filter(|e| {
                    e.military && (e.owner.is_barbarian() || self.view().at_war(v.me, e.owner))
                })
                .map(|e| e.pos)
                .chain(self.view().barbarian_camps.iter().map(|c| c.pos))
                .chain(
                    self.ctx
                        .enemy_cities
                        .iter()
                        .filter(|c| roman && v.at_war(v.me, c.owner))
                        .map(|c| c.pos),
                )
                .filter(|h| {
                    u.pos.distance(*h) <= if roman { 10 } else { 8 }
                        && (!roman || hunt_radius == 0 || home_distance(*h) <= hunt_radius)
                        && self.ctx.reachable(u.pos, *h)
                })
                .min_by_key(|h| (u.pos.distance(*h), *h));
            // Gene `roman_hunt_radius`: with nothing worth walking to near home, the recruit
            // stays with the army (a strike or campaign needs it together) instead of
            // reconnoitring the frontier, which near home only shuttles it between fog edges.
            if let Some(h) = prey
                && let Some(to) = self.campaign_staging_spot(u, h, 1)
                && (!roman || !self.mil_dest.contains(&to))
            {
                self.go_to(u, to);
            } else if roman && !self.defending(u.pos) && hunt_radius == 0 {
                // A visible barbarian is not guaranteed in the opening sight radius.
                // Reconnoitre a reachable frontier with a spare soldier; later turns
                // can choose an actual camp or safe melee attack from its own view.
                if let Some(h) = self
                    .ctx
                    .frontier
                    .iter()
                    .copied()
                    .filter(|h| {
                        (2..=8).contains(&u.pos.distance(*h))
                            && (hunt_radius == 0 || home_distance(*h) <= hunt_radius)
                            && self.ctx.reachable(u.pos, *h)
                            && self.ctx.unexplored_near(*h) > 0
                            && !self.mil_dest.contains(h)
                    })
                    // A nearest-edge choice sends the same recruit back and forth
                    // across its newly explored ring. Keep pushing away from home
                    // toward land that can still reveal a camp or vulnerable scout.
                    .max_by_key(|h| {
                        let away = self
                            .ctx
                            .own_cities
                            .iter()
                            .map(|c| c.pos.distance(*h))
                            .min()
                            .unwrap_or(0);
                        (
                            away + self.ctx.unexplored_near(*h) * 4 - u.pos.distance(*h) * 2,
                            *h,
                        )
                    })
                {
                    self.go_to(u, h);
                }
            }
        }
    }

    /// A damaged capture near our border may be taken back before resistance and city
    /// healing make it defensible. The shared assault planner still checks the actual
    /// garrison, city shield and local force before committing an order.
    /// The assault planner on one visible enemy city (`door.rs` retakes).
    /// The coordinated assault on the city at `pos` with `army` only (`operation.rs`).
    pub(crate) fn operation_assault(&mut self, army: &[&UnitView], pos: Hex) {
        self.coordinate_city_assaults_on(army, Some(pos), true);
    }

    pub(crate) fn assault_city_at(&mut self, army: &[&UnitView], pos: Hex) {
        self.coordinate_city_assaults_at(army, Some(pos));
    }

    fn retake_weakened_city(&mut self, army: &[&UnitView]) {
        let target = self
            .ctx
            .enemy_cities
            .iter()
            .filter(|city| {
                city.visible
                    && city.hp * 2 <= city.max_hp
                    && self.attack_view().at_war(self.view().me, city.owner)
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(city.pos) <= 6)
                    && army.iter().any(|u| {
                        crate::composition::charges(u.kind)
                            && u.hp >= 50
                            && u.pos.distance(city.pos) <= 3
                    })
            })
            .min_by_key(|city| (city.hp, city.pos))
            .map(|city| city.pos);
        if let Some(pos) = target {
            self.coordinate_city_assaults_at(army, Some(pos));
        }
    }

    /// A runaway can take a city we last saw under another flag. Until we know one
    /// of its current cities, refresh the nearest fogged border city with a field unit.
    /// This uses only remembered geography, and keeps the last garrisons at home.
    fn scout_threat_front(&mut self, army: &[&UnitView]) {
        let Some(threat) = self.top_threat_team() else {
            return;
        };
        if self
            .view()
            .cities
            .iter()
            .any(|c| self.view().team_of(c.owner) == threat)
        {
            return;
        }
        if self.ctx.own_units.iter().filter(|u| u.military).count() < 3 {
            return;
        }
        let mut best = None;
        for unit in self.free(army) {
            if unit.hp < 70 || self.ctx.own_cities.iter().any(|c| c.pos == unit.pos) {
                continue;
            }
            for &city in &self.ctx.enemy_cities {
                if city.visible
                    || !self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(city.pos) <= 10)
                    || !self.ctx.reachable(unit.pos, city.pos)
                {
                    continue;
                }
                let key = (unit.pos.distance(city.pos), city.pos, unit.id);
                if best.as_ref().is_none_or(|(old, _, _)| key < *old) {
                    best = Some((key, unit, city.pos));
                }
            }
        }
        let Some((_, unit, city_pos)) = best else {
            return;
        };
        if let Some(spot) = city_pos
            .ring(legatum_core::data::UNIT_SIGHT)
            .into_iter()
            .filter(|h| {
                self.free_tile_for(unit, *h)
                    && self.safe_campaign_tile(*h)
                    && self.view().tile(*h).is_some_and(|t| {
                        self.view().passage_allowed(
                            self.view().me,
                            legatum_core::UnitKind::Spearman,
                            t.owner,
                        )
                    })
            })
            .min_by_key(|h| (unit.pos.distance(*h), *h))
            && let Some(to) = self.campaign_approach(unit, spot)
        {
            self.go_to(unit, to);
        }
    }

    /// Carry out the strike the forecast approved (`opportunity.rs`), from the turn war is
    /// declared: committed units move to their assault tiles, Archers in range shoot the city
    /// (its garrison first), and melee attacks when the whole-city assault forecast says the
    /// capture holds. Runs before routine garrisons and recovery, which would otherwise
    /// scatter a winning force (rush-offense, 2026-10-04: an Archer walked home to garrison
    /// against the one Mongols Spearman that was busy holding its own city).
    fn press_strike(&mut self, army: &[&UnitView]) {
        let Some(strike) = self.opportunity_strike().cloned() else {
            return;
        };
        let v = self.view();
        let declared = self
            .orders
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { against, .. } if *against == strike.team));
        if !v.teams_at_war(v.team, strike.team) && !declared {
            return;
        }
        if self
            .ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos == strike.pos && c.visible)
        {
            self.coordinate_city_assaults_at(army, Some(strike.pos));
        }
        let simultaneous_melee = strike
            .force
            .iter()
            .filter_map(|(id, _)| army.iter().copied().find(|u| u.id == *id))
            .filter(|u| {
                crate::composition::charges(u.kind) && self.reaches_this_turn(u, strike.pos)
            })
            .count();
        // Gene `joint_assault`: against a city that strikes, the force steps into its range only
        // together (`siege.rs::joint_entry`), not each unit as it arrives.
        let joint_city = self
            .ctx
            .enemy_cities
            .iter()
            .copied()
            .find(|c| c.pos == strike.pos && c.visible && c.walls && c.hp > 0)
            .filter(|_| self.decisions.joint_assault > 0 && self.hard_defense());
        let mut joint_plans: Vec<(&UnitView, Hex)> = Vec::new();
        for (id, to) in &strike.force {
            let Some(u) = army.iter().copied().find(|u| u.id == *id) else {
                continue;
            };
            if self.has_acted(u.id) {
                continue;
            }
            // Gene `strike_kill`: a kill on the way is worth more than the staging tile.
            if self.decisions.strike_kill > 0 && self.kill_beside(u) {
                continue;
            }
            // The opportunity forecast can route a distant wing into staging positions.
            // A nearer striker must still pay for its own duel and the response at the
            // occupied city before it commits ahead of that wing.
            if self.decisions.melee_response_weight > 0
                && simultaneous_melee < 2
                && crate::composition::charges(u.kind)
                && self.reaches_this_turn(u, strike.pos)
                && let Some(e) = self
                    .ctx
                    .enemy_units
                    .iter()
                    .find(|e| e.pos == strike.pos && e.military)
                && self
                    .melee_forecast(u, strike.pos)
                    .is_some_and(|f| !self.melee_reply_pays(u, e, &f))
            {
                continue;
            }
            let fires_now = crate::composition::shooter_only(u.kind, u.ranged)
                && u.pos.distance(strike.pos) <= u.range
                && self.ranged_unit_clear(u, u.pos, strike.pos);
            if let Some(city) = joint_city {
                let tile = if fires_now || self.mil_dest.contains(to) {
                    u.pos
                } else {
                    *to
                };
                if tile.distance(city.pos) <= 2 || self.in_other_city_fire(tile, city.pos) {
                    joint_plans.push((u, tile));
                    continue;
                }
            }
            if fires_now {
                self.mil_dest.insert(u.pos);
                self.unit_order(
                    u.id,
                    Order::RangedAttack {
                        unit: u.id,
                        target: strike.pos,
                    },
                );
            } else if self.mil_dest.contains(to) || u.pos == *to {
                self.hold_or_upgrade(u);
            } else {
                // The forecast already routed this move under war borders; on the
                // declaration turn the same submission opens them (a surprise war takes
                // effect before movement), which this turn's peace-time map cannot see.
                self.mil_dest.insert(*to);
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: *to,
                    },
                );
            }
        }
        if let Some(city) = joint_city
            && !joint_plans.is_empty()
        {
            self.joint_entry(city, &joint_plans);
        }
    }

    /// Once four Spearmen are ready against a nearby open Byzantium city, bring the
    /// capital guard to the peace-side staging ring. A lone guard stays home, and
    /// this sortie stops if the capital is threatened or the city gains Walls.
    fn sparta_nearby_sortie(&mut self, army: &[&UnitView]) {
        let v = self.view();
        if self.decisions.lone_city_strike == 0
            || self.ctx.own_cities.len() != 1
            || v.turn > self.decisions.lone_city_sortie_turn as u32
            || army
                .iter()
                .filter(|u| u.kind == UnitKind::Spearman && u.hp >= 70)
                .count()
                < 4
        {
            return;
        }
        let home = self.ctx.own_cities[0];
        if self
            .ctx
            .threat_units
            .iter()
            .any(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(home.pos) <= 4)
        {
            return;
        }
        // Gene `value_first_target`: the valued conquest nomination, not a faction name.
        let target = if self.value_first_target() {
            self.conquest_city().filter(|c| {
                !c.walls && home.pos.distance(c.pos) <= self.decisions.open_prize_radius
            })
        } else {
            v.cities.iter().find(|c| {
                !c.owner.is_barbarian()
                    && !v.same_team(v.me, c.owner)
                    && v.faction_of(c.owner) == Faction::Byzantium
                    && !c.walls
                    && home.pos.distance(c.pos) <= self.decisions.lone_city_strike_radius
            })
        };
        let Some(target) = target else {
            return;
        };
        let Some(guard) = self
            .free(army)
            .into_iter()
            .find(|u| u.kind == UnitKind::Spearman && u.pos == home.pos && u.hp >= 70)
        else {
            return;
        };
        let spot = target
            .pos
            .ring(3)
            .into_iter()
            .filter(|h| {
                *h != guard.pos
                    && v.tile(*h).is_some_and(|t| {
                        t.explored
                            && t.terrain.and_then(|terrain| terrain.move_cost()).is_some()
                            && v.passage_allowed(v.me, guard.kind, t.owner)
                    })
                    && self.safe_campaign_tile(*h)
                    && self.free_tile_for(guard, *h)
                    && self.ctx.reachable(guard.pos, *h)
            })
            .min_by_key(|h| (h.distance(home.pos), *h));
        if let Some(to) = spot.and_then(|spot| self.campaign_approach(guard, spot)) {
            self.go_to(guard, to);
        }
    }

    /// After an occupied Byzantium capital has recovered, its veteran rejoins the
    /// field army for a nearby open India war. A real military threat keeps it home.
    fn sparta_followup_sortie(&mut self, army: &[&UnitView]) {
        let v = self.view();
        if self.decisions.open_followup == 0 || self.ctx.own_cities.len() != 2 {
            return;
        }
        let valued = self
            .value_first_target()
            .then(|| self.conquest_city().filter(|c| !c.walls))
            .flatten();
        let Some(target) = valued.or_else(|| {
            if self.value_first_target() {
                return None;
            }
            v.cities
                .iter()
                .filter(|c| {
                    !c.owner.is_barbarian() && v.faction_of(c.owner) == Faction::India && !c.walls
                })
                .min_by_key(|c| {
                    (
                        !c.is_capital,
                        self.ctx
                            .own_cities
                            .iter()
                            .map(|home| home.pos.distance(c.pos))
                            .min()
                            .unwrap_or(i32::MAX),
                    )
                })
        }) else {
            return;
        };
        let Some(city) =
            self.ctx.own_cities.iter().skip(1).find(|c| {
                c.hp * 2 >= c.max_hp
                    && c.detail.as_ref().is_some_and(|d| d.resistance == 0)
                    && c.pos.distance(target.pos) <= 14
                    && !self.ctx.threat_units.iter().any(|u| {
                        u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 4
                    })
            })
        else {
            return;
        };
        let Some(veteran) = self
            .free(army)
            .into_iter()
            .filter(|u| u.pos == city.pos && crate::composition::charges(u.kind) && u.hp >= 70)
            .max_by_key(|u| (u.xp, u.hp, std::cmp::Reverse(u.id)))
        else {
            return;
        };
        let spot = target
            .pos
            .ring(3)
            .into_iter()
            .filter(|h| {
                v.tile(*h).is_some_and(|t| {
                    t.explored
                        && t.terrain.and_then(|terrain| terrain.move_cost()).is_some()
                        && v.passage_allowed(v.me, veteran.kind, t.owner)
                }) && self.safe_campaign_tile(*h)
                    && self.free_tile_for(veteran, *h)
                    && self.ctx.reachable(veteran.pos, *h)
            })
            .min_by_key(|h| (h.distance(veteran.pos), *h))
            .or_else(|| {
                self.ctx
                    .frontier
                    .iter()
                    .copied()
                    .filter(|h| {
                        self.safe_campaign_tile(*h)
                            && self.free_tile_for(veteran, *h)
                            && self.ctx.reachable(veteran.pos, *h)
                    })
                    .min_by_key(|h| (h.distance(target.pos), h.distance(veteran.pos), *h))
            });
        if let Some(to) = spot.and_then(|spot| self.campaign_approach(veteran, spot)) {
            self.go_to(veteran, to);
        }
    }

    /// A neighbour whose land we have seen near home but none of whose nearby cities we
    /// know, first one we are at war with: send one healthy idle unit to look, so a strike
    /// or a defence has a target. (A player reads the map for the neighbour's cities; a
    /// stateless AI only knows what its units have seen.)
    fn scout_front(&mut self, army: &[&UnitView]) {
        if self.decisions.scout_front == 0 {
            return;
        }
        let v = self.view();
        let mut leads: Vec<(bool, i32, Hex)> = Vec::new();
        for p in v
            .players
            .iter()
            .filter(|p| p.alive && !v.same_team(v.me, p.id))
        {
            let war = v.at_war(v.me, p.id);
            let known_city = v.cities.iter().any(|c| {
                c.owner == p.id
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(c.pos) <= 8)
            });
            if known_city {
                continue;
            }
            for home in &self.ctx.own_cities {
                for h in home.pos.within(6) {
                    if v.tile(h)
                        .is_some_and(|t| t.explored && t.owner == Some(p.id))
                    {
                        leads.push((!war, home.pos.distance(h), h));
                    }
                }
            }
        }
        leads.sort();
        let Some(&(_, _, lead)) = leads.first() else {
            return;
        };
        // Look from the nearest tile we may stand on; at peace that is our side of the border.
        let Some(spot) = lead
            .within(2)
            .into_iter()
            .filter(|h| {
                v.tile(*h).is_some_and(|t| {
                    t.terrain.and_then(|t| t.move_cost()).is_some()
                        && v.passage_allowed(v.me, legatum_core::UnitKind::Spearman, t.owner)
                }) && self.safe_campaign_tile(*h)
            })
            .min_by_key(|h| (h.distance(lead), *h))
        else {
            return;
        };
        let scout = self
            .free(army)
            .into_iter()
            .filter(|u| {
                u.hp >= MARCH_MIN_HP
                    && u.pos.distance(spot) > 0
                    && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
            })
            .filter(|u| self.ctx.reachable(u.pos, spot) && self.free_tile_for(u, spot))
            .min_by_key(|u| (u.pos.distance(spot), std::cmp::Reverse(u.max_moves), u.id));
        if let Some(u) = scout
            && let Some(to) = self.campaign_approach(u, spot)
        {
            self.go_to(u, to);
        }
    }

    /// The starting infantry helps find a target while the capital trains its army.
    /// Use the same known frontiers and safe routes as scouts, on separate headings.
    fn rush_reconnaissance(&mut self, army: &[&UnitView]) {
        if !self.rushing() || self.opening_strike_city().is_some() {
            return;
        }
        let mut taken: Vec<_> = self
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => Some(*to),
                _ => None,
            })
            .collect();
        let march_min_hp = self.march_min_hp();
        for u in self
            .free(army)
            .into_iter()
            .filter(|u| crate::composition::charges(u.kind) && u.hp >= march_min_hp)
            .take(2)
        {
            if let Some(target) = self.choose_frontier(u, &taken)
                && let Some(to) = self.campaign_approach(u, target)
            {
                taken.push(target);
                self.go_to(u, to);
            }
        }
    }

    fn occupy_rush_city(&mut self, army: &[&UnitView]) {
        if self.takes_no_cities() {
            return;
        }
        if !self.rushing() {
            return;
        }
        let Some(pos) = self
            .opening_strike_city()
            .filter(|c| {
                c.visible
                    && c.hp == 0
                    && self.attack_view().at_war(self.view().me, c.owner)
                    && !self
                        .ctx
                        .enemy_units
                        .iter()
                        .any(|e| e.military && e.pos == c.pos)
            })
            .map(|c| c.pos)
        else {
            return;
        };
        let capturer = self
            .free(army)
            .into_iter()
            .filter(|u| {
                crate::composition::charges(u.kind)
                    && self.reaches_this_turn(u, pos)
                    && (u.hp >= self.march_min_hp() || self.exposure_at(u.pos) == 0)
            })
            .min_by_key(|u| (u.pos.distance(pos), std::cmp::Reverse(u.hp), u.id));
        if let Some(u) = capturer {
            self.unit_order(
                u.id,
                Order::MoveUnit {
                    unit: u.id,
                    to: pos,
                },
            );
            self.mil_dest.insert(pos);
        }
    }

    /// Where `u` can stand this turn outside every enemy city's two-hex strike range, the
    /// least exposed first; failing that, a step away from `city` (rough terrain can make
    /// leaving the whole ring impossible in one turn: step toward its edge instead of dying
    /// in place).
    fn out_of_city_range(&self, u: &UnitView, city: Option<&CityView>) -> Option<Hex> {
        u.pos
            .within(u.max_moves)
            .into_iter()
            .filter(|h| {
                self.free_tile_for(u, *h)
                    && self.reaches_this_turn(u, *h)
                    && self
                        .ctx
                        .enemy_cities
                        .iter()
                        .all(|c| c.hp == 0 || c.pos.distance(*h) > 2)
            })
            .min_by_key(|h| {
                (
                    self.exposure_key(*h),
                    !self.outside_remembered_reach(*h),
                    h.distance(u.pos),
                    self.recovery_key(u, *h),
                )
            })
            .or_else(|| {
                let c = city?;
                u.pos
                    .within(u.max_moves)
                    .into_iter()
                    .filter(|h| {
                        h.distance(c.pos) > u.pos.distance(c.pos)
                            && self.free_tile_for(u, *h)
                            && self.reaches_this_turn(u, *h)
                    })
                    .min_by_key(|h| (self.exposure_key(*h), -h.distance(c.pos), *h))
            })
    }

    /// Gene `strike_range_exit`: no plan attacks a firing enemy city any more (the Campaign
    /// went to Peace, or was abandoned after its first strike), so units standing within its
    /// two-hex strike range leave it at once (the 8.9.4 Mongols stood in Lugano's range for
    /// three turns after the plan changed and lost three units to its strikes). A city a
    /// plan, a running campaign or an opportunistic assault goes for keeps its units, and so
    /// does one already half broken; units in our own cities stay.
    fn leave_strike_range(&mut self, army: &[&UnitView]) {
        if self.decisions.strike_range_exit == 0
            || matches!(self.faction, Faction::Sparta | Faction::Celts)
            || self.conquest_city().is_some()
            || self.opportunity_strike().is_some()
        {
            return;
        }
        let v = self.view();
        let mut fighting: Vec<TeamId> = Vec::new();
        match self.routed_plan().map(|p| &p.kind) {
            Some(crate::plan::PlanKind::Campaign { against, .. })
            | Some(crate::plan::PlanKind::Contain { against }) => fighting.push(*against),
            _ => {}
        }
        if let Some(active) = &self.campaigns.active
            && let Some(c) = v.cities.iter().find(|c| c.id == active.target)
        {
            fighting.push(v.team_of(c.owner));
        }
        for u in self.free(army) {
            if self.ctx.own_cities.iter().any(|c| c.pos == u.pos) {
                continue;
            }
            let Some(city) = self.ctx.enemy_cities.iter().copied().find(|c| {
                !fighting.contains(&v.team_of(c.owner))
                    && c.hp * 2 > c.max_hp
                    && forecast_city_strike(v, c, u, u.pos).is_some()
            }) else {
                continue;
            };
            if let Some(to) = self.out_of_city_range(u, Some(city)) {
                self.go_to(u, to);
                self.retreats.insert(u.id);
            }
        }
    }

    /// A city volley precedes movement, so rotate before the next shot is fatal.
    /// Recovery outside the firing ring must not be interrupted at half health.
    fn recover_city_fire(&mut self, army: &[&UnitView]) {
        for u in self.free(army) {
            let city = self
                .ctx
                .enemy_cities
                .iter()
                .copied()
                .find(|c| forecast_city_strike(self.view(), c, u, u.pos).is_some());
            let threatened = city.is_some() && {
                let first = self.city_damage_now(u);
                let mut after = u.clone();
                after.hp = (u.hp - first).max(0);
                // Ranged fire precedes the escape. Leave while that first shot is
                // survivable, rather than waiting until next turn's volley kills us.
                u.hp < self.decisions.recover_hp
                    || first > 0
                        && (after.hp == 0 || self.max_city_damage_at(&after, u.pos) >= after.hp)
            };
            if threatened {
                let destination = self.out_of_city_range(u, city);
                if let Some(to) = destination {
                    self.go_to(u, to);
                    self.retreats.insert(u.id);
                }
            } else if self.faction != Faction::Sparta
                && self.rushing()
                && city.is_none()
                && u.hp
                    < if crate::composition::shooter_only(u.kind, u.ranged) {
                        100
                    } else {
                        80
                    }
                && self.safe_campaign_tile(u.pos)
            {
                self.hold_or_upgrade(u);
            }
        }
    }

    fn rush_melee_screen_at(&self, target: Hex) -> bool {
        self.ctx.own_units.iter().any(|u| {
            if !u.military
                || u.kind == UnitKind::Scout
                || crate::composition::shooter_only(u.kind, u.ranged)
                || u.hp < 80
                || self.retreats.contains(&u.id)
            {
                return false;
            }
            let arrival = self.orders.iter().find_map(|o| match o {
                Order::MoveUnit { unit, to } if *unit == u.id => Some(*to),
                _ => None,
            });
            match arrival {
                Some(to) => {
                    if to.distance(target) > 2 || u.pos.distance(to) > u.max_moves {
                        return false;
                    }
                    // find_path reads only the map's geometry.
                    let Some(path) = find_path(self.movement_map(), u.pos, to, |h| {
                        let i = self.view().index(h)?;
                        self.ctx.move_cost(h).filter(|_| {
                            !self.ctx.friendly_military_at[i]
                                && (h == to
                                    || !self.mil_dest.contains(&h) && !self.ctx.hostile_at[i])
                        })
                    }) else {
                        return false;
                    };
                    // The destination was reserved by this unit's own movement order.
                    let mut moves = u.max_moves;
                    path.into_iter().all(|h| {
                        let cost = self.ctx.move_cost(h).expect("known staging route");
                        let reached =
                            legatum_core::rules::movement::can_step(moves, u.max_moves, cost);
                        moves = (moves - cost).max(0);
                        reached
                    })
                }
                None => u.pos.distance(target) <= 2,
            }
        })
    }

    /// Coordinate the whole local force, including archers that already fired. A
    /// siege must not dissolve when the general planner reserves half its support.
    fn rush_siege(&mut self, army: &[&UnitView]) {
        if self.takes_no_cities() {
            return;
        }
        if !self.rushing() {
            return;
        }
        let Some(city) = self
            .opening_strike_city()
            .filter(|c| c.visible && self.attack_view().at_war(self.view().me, c.owner))
        else {
            return;
        };
        let pos = city.pos;
        // The capture planner already reserved the melee wing. In particular, a
        // guaranteed volley must have just one occupier, even in the Rush fallback.
        if self.orders.iter().any(|o| {
            matches!(o, Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } if *to == pos)
        }) {
            return;
        }
        let city_hp = city.hp;
        if city_hp == 0
            && !self
                .ctx
                .enemy_units
                .iter()
                .any(|u| u.military && u.pos == pos)
        {
            // The occupation passes assign a melee capturer and leave the rest free
            // for recovery or reinforcement.
            return;
        }
        let Some(target) = self.targets(false).into_iter().find(|t| t.pos == pos) else {
            return;
        };
        let near: Vec<_> = army
            .iter()
            .copied()
            .filter(|u| {
                u.hp >= if crate::composition::shooter_only(u.kind, u.ranged) {
                    80
                } else {
                    self.march_min_hp()
                } && u.pos.distance(pos) <= 4
                    && self.ctx.reachable(u.pos, pos)
                    && !self.retreats.contains(&u.id)
            })
            .collect();
        let force: i32 = near.iter().map(|u| offense_power(self.view(), u)).sum();
        if near.len() < 2 || force * 100 < self.attack_threshold() * target.their.max(1) {
            return;
        }
        // A city with HP absorbs fire even when garrisoned (core `combat::city_tile_defender`),
        // so archers can wear it down; its garrison must then be beaten in melee.
        let garrisoned = self
            .ctx
            .enemy_units
            .iter()
            .any(|u| u.military && u.pos == pos);
        let bombard = near
            .iter()
            .filter(|u| crate::composition::shooter_only(u.kind, u.ranged))
            .count()
            >= 2
            && city_hp > 0;
        let volley: i32 = near.iter().filter_map(|u| {
            self.orders.iter().any(|o| matches!(o, Order::RangedAttack { unit, target } if *unit == u.id && *target == pos))
                .then(|| forecast_attack_mode(&self.attack_view(), u.id, pos, AttackMode::Ranged))
                .flatten().map(|f| f.expected_damage_to_defender)
        }).sum();
        for u in self.free(&near) {
            if crate::composition::shooter_only(u.kind, u.ranged) {
                if u.pos.distance(pos) <= u.range || self.rush_melee_screen_at(pos) {
                    self.strike(u, pos, false);
                } else {
                    self.hold_or_upgrade(u);
                }
                continue;
            }
            if bombard {
                let finish = self.melee_forecast(u, pos).is_some_and(|f| {
                    f.loss_chance < 25
                        && if garrisoned {
                            // The volley breaks the city first; this attack then fights
                            // the exposed garrison and captures only if it wins.
                            f.garrison.as_ref().is_some_and(|g| !g.shielded) && f.win_chance >= 50
                        } else {
                            city_hp <= volley + f.expected_damage_to_defender
                        }
                });
                if !finish || self.mil_dest.contains(&pos) {
                    // Preserve the capturer while archers reduce the city. A melee
                    // screen at range two can rotate out in one step after a volley;
                    // charging early pays both the city shot and melee retaliation.
                    if let Some(to) = self.counterattack_target(u, None) {
                        self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
                        continue;
                    }
                    let spot = pos
                        .ring(1)
                        .into_iter()
                        .chain(pos.ring(2))
                        .filter(|h| {
                            self.free_tile_for(u, *h)
                                && (*h == u.pos || self.reaches_this_turn(u, *h))
                                && u.hp - self.max_city_damage_at(u, *h) >= MARCH_MIN_HP
                        })
                        .min_by_key(|h| {
                            let cover =
                                self.view().tile(*h).and_then(|t| t.terrain).map_or(0, |t| {
                                    data::terrain_defense_rules(
                                        self.view().rule_experiments.faction_rules_off,
                                        self.faction,
                                        t,
                                    )
                                });
                            (
                                self.exposure_key(*h),
                                // A survivor next to the city stops its 20-HP end-of-turn regeneration.
                                i32::from(h.distance(pos) != 1),
                                -cover,
                                i32::from(*h != u.pos),
                                h.distance(u.pos),
                                *h,
                            )
                        });
                    match spot {
                        Some(to) => self.go_to(u, to),
                        None => {
                            if let Some(to) = self.campaign_staging_spot(u, pos, 2) {
                                self.go_to(u, to);
                            }
                        }
                    }
                    continue;
                }
                if self.reaches_this_turn(u, pos) {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: pos,
                        },
                    );
                    self.mil_dest.insert(pos);
                    continue;
                }
            }
            if self.reaches_this_turn(u, pos) && !self.too_risky(u, pos) {
                let tank = self.melee_forecast(u, pos).is_some_and(|f| {
                    f.city_strike.is_some_and(|s| {
                        army.iter()
                            .filter(|ally| ally.pos.distance(pos) <= 2)
                            .min_by_key(|ally| (ally.hp, ally.id))
                            .is_some_and(|ally| ally.id == u.id)
                            && u.hp - s.damage - f.expected_damage_to_attacker < 60
                    })
                });
                if tank {
                    self.hold_or_upgrade(u);
                } else {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: pos,
                        },
                    );
                }
            } else {
                // Choose distinct, reachable arrivals; the group strength check above
                // pays for entering city range instead of treating it as routine travel.
                let spot = pos
                    .neighbors()
                    .into_iter()
                    .filter(|h| {
                        self.free_tile_for(u, *h)
                            && self.reaches_this_turn(u, *h)
                            && self.outside_remembered_reach(*h)
                    })
                    .min_by_key(|h| (self.exposure_key(*h), h.distance(u.pos), *h));
                if let Some(to) = spot {
                    self.go_to(u, to);
                }
            }
        }
    }

    pub(crate) fn free<'u>(&self, army: &[&'u UnitView]) -> Vec<&'u UnitView> {
        army.iter()
            .copied()
            .filter(|u| !self.has_acted(u.id))
            .collect()
    }

    /// A tile the unit may stand on: not reserved, and not held by another friendly military
    /// unit or anything hostile.
    pub(crate) fn free_tile_for(&self, u: &UnitView, h: Hex) -> bool {
        if h == u.pos {
            return !self.mil_dest.contains(&h);
        }
        let Some(i) = self.ctx.view.index(h) else {
            return false;
        };
        !self.ctx.occupied[i]
            && self.ctx.land_region[i].is_some_and(|r| self.ctx.region(u.pos) == Some(r))
            && !self.mil_dest.contains(&h)
    }

    pub(crate) fn hold_or_upgrade(&mut self, u: &UnitView) {
        self.mil_dest.insert(u.pos);
        if self.named_fund().is_some() {
            self.unit_order(u.id, Order::SkipUnit { unit: u.id });
            return;
        }
        let reserve = self.gold_reserve();
        if let Some(cost) = u.upgrade_cost
            && (self.mongol_keshig_upgrade(u) && self.gold_left >= cost
                || self.gold_left - cost >= reserve)
            && (self.mongol_keshig_upgrade(u) || self.affordable_beyond_runway(cost))
            && (self.mongol_keshig_upgrade(u) || self.modernization_upgrade_worth(u))
            && self.upgrade_gains(u)
            && self.upgrade_supply_worth(u)
        {
            self.gold_left -= cost;
            self.unit_order(u.id, Order::UpgradeUnit { unit: u.id });
        } else {
            self.unit_order(u.id, Order::SkipUnit { unit: u.id });
        }
    }

    pub(crate) fn mongol_keshig_upgrade(&self, u: &UnitView) -> bool {
        self.mongol_horde()
            && self.decisions.mongol_horde >= 2
            && u.kind == UnitKind::Horseman
            && u.upgrade_need == Some(legatum_core::resources::Resource::Iron)
    }

    /// An upgrade that uses supply (Phalangite to Pikeman, 1 Iron) is worth the supply it takes
    /// from the Forges waiting for it (`supply_value::unit_outbids_industry`).
    fn upgrade_supply_worth(&self, u: &UnitView) -> bool {
        let (Some(iron), Some(to)) = (u.upgrade_need, data::unit_stats(u.kind).upgrades_to) else {
            return true;
        };
        crate::supply_value::unit_outbids_industry(
            self.view(),
            u.kind,
            to,
            iron,
            self.decisions.supply_war_percent,
            self.mongol_keshig_upgrade(u),
        )
    }

    /// Whether the upgrade is a better unit. Always along an ordinary line; the Keshig's upgrade
    /// (an ordinary Knight) gives up its bow, so it must gain more than that is worth
    /// (`composition::worth`).
    fn upgrade_gains(&self, u: &UnitView) -> bool {
        if !crate::composition::dual(u.kind, u.ranged) {
            return true;
        }
        let Some(to) = data::unit_stats(u.kind).upgrades_to else {
            return false;
        };
        let to_stats = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, to);
        // Never stranded: once the unit it upgrades to is itself outdated (its own upgrade is
        // known), take the step, so the line can climb on to current units.
        let outdated = to_stats
            .upgrades_to
            .and_then(|next| data::unit_stats(next).requires)
            .is_some_and(|t| self.view().research.known.contains(&t));
        outdated
            || crate::composition::worth(to, to_stats.strength, to_stats.ranged)
                > crate::composition::worth(u.kind, u.strength, u.ranged)
    }

    pub(crate) fn go_to(&mut self, u: &UnitView, to: Hex) {
        if to == u.pos {
            self.hold_or_upgrade(u);
        } else {
            self.mil_dest.insert(to);
            self.move_or_hold(u, to);
        }
    }

    /// Archers shoot the best visible target in range (`best_shot`). A unit that can also
    /// charge (the Keshig) charges instead when its best charge is worth more (`dual.rs`).
    fn ranged_fire(&mut self, army: &[&UnitView]) {
        self.fire_units(army, false);
    }

    /// The bow of a unit that also charges (the Keshig) is for what is left after the plan's
    /// moves: it skips the early volley (`ranged_fire`), so a campaign's march, staging and
    /// assault orders take it first, and only a unit still free here picks its best shot or
    /// charge (Golden Horde scenario, 256 runs: firing first, 44% to 33%; here, 39%).
    pub(crate) fn dual_fire(&mut self, army: &[&UnitView]) {
        self.fire_units(army, true);
    }

    fn fire_units(&mut self, army: &[&UnitView], late: bool) {
        let mut committed = self.committed_fire();
        for u in self.free(army) {
            if crate::composition::dual(u.kind, u.ranged) != late {
                continue;
            }
            if self.weighs_dual(u)
                && let Some((shot, _)) = self.dual_shot_net(u, &committed)
                && let Some((charge, to)) = self.best_charge(u)
                && !(self.mongol_horde()
                    && self.decisions.mongol_horde >= 3
                    && self
                        .ctx
                        .enemy_cities
                        .iter()
                        .any(|c| c.pos == to && c.walls && c.hp > 0))
                && charge > shot
            {
                if self.ctx.enemy_cities.iter().any(|c| c.pos == to) {
                    // One occupier per fallen city (`attack_targets`).
                    self.mil_dest.insert(to);
                }
                self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
                continue;
            }
            if let Some(target) = self.best_shot(u, &committed) {
                if self.decisions.ranged_survival_weight > 0 {
                    let shot = self.ranged_action_value(u, target);
                    if self.retreat_action_value(u) > shot {
                        continue;
                    }
                    if self.rest_action_value(u) > shot {
                        self.unit_order(u.id, Order::SkipUnit { unit: u.id });
                        continue;
                    }
                }
                if let Some(f) =
                    forecast_attack_mode(&self.attack_view(), u.id, target, AttackMode::Ranged)
                {
                    *committed.entry(target).or_default() += f.expected_damage_to_defender;
                }
                self.mil_dest.insert(u.pos);
                self.unit_order(u.id, Order::RangedAttack { unit: u.id, target });
            }
        }
    }

    fn ranged_action_value(&self, u: &UnitView, target: Hex) -> i32 {
        let Some(f) = forecast_attack_mode(self.view(), u.id, target, AttackMode::Ranged) else {
            return 0;
        };
        if let Some(e) = self.ctx.enemy_units.iter().find(|e| e.pos == target)
            && !self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| c.pos == target && c.hp > 0)
        {
            let denial = crate::hold::denial_priority(self.view(), e.owner, || 0);
            let cost = self
                .view()
                .rule_experiments
                .unit_stats_for(self.view().faction_of(e.owner), e.kind)
                .cost;
            let material = cost * f.expected_damage_to_defender.min(e.hp) / e.hp.max(1);
            return ((self.shot_value(e, e.hp, &f) / 1000) as i32 + material)
                .saturating_mul(120 + denial)
                / 100;
        }
        let Some(c) = self.ctx.enemy_cities.iter().find(|c| c.pos == target) else {
            return 0;
        };
        (self.city_shot_value(c.pos, c.hp, &f) / 1000) as i32
            + 400 * f.expected_damage_to_defender.min(c.hp) / c.hp.max(1)
    }

    fn rest_action_value(&self, u: &UnitView) -> i32 {
        let heal = if self
            .view()
            .tile(u.pos)
            .is_some_and(|t| t.owner == Some(u.owner))
        {
            data::HEAL_TERRITORY
        } else {
            data::HEAL_FIELD
        };
        let regained = heal.min(data::UNIT_HP - u.hp).max(0);
        let cost = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, u.kind)
            .cost;
        let healing = regained * cost * self.decisions.ranged_survival_weight / 10_000;
        let exposed = self.city_shield(u.pos) != crate::tactics::CityShield::Full;
        let incoming = if exposed {
            self.incoming_at(u, u.pos, u.hp, None)
        } else {
            0
        };
        let preservation = regained * cost * self.decisions.ranged_survival_weight * incoming
            / (u.hp.max(1) * u.hp.max(1) * 100);
        healing + preservation
    }

    fn retreat_action_value(&self, u: &UnitView) -> i32 {
        let Some(to) = self.safer_tile(u) else {
            return 0;
        };
        let now = self.incoming_at(u, u.pos, u.hp, None);
        let there = if self.ctx.own_cities.iter().any(|c| c.pos == to && c.hp > 0)
            && self.can_defend_at(u, to)
        {
            0
        } else {
            self.incoming_at(u, to, u.hp, None)
        };
        let cost = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, u.kind)
            .cost;
        let avoided = (now - there).max(0) * cost * self.decisions.ranged_survival_weight / 30_000;
        let saved_unit = if now >= u.hp && there < u.hp {
            cost * self.decisions.ranged_survival_weight / 60
        } else {
            0
        };
        avoided + saved_unit
    }

    /// A held ranged unit can still fire from its reserved tile. A wounded garrison keeps its
    /// rest when that recovery is worth more than the available shot.
    fn fire_from_holds(&mut self) {
        let mut committed = self.committed_fire();
        for i in 0..self.orders.len() {
            let Order::SkipUnit { unit } = self.orders[i] else {
                continue;
            };
            let Some(u) = self.ctx.own_units.iter().copied().find(|u| u.id == unit) else {
                continue;
            };
            let Some(target) = self.best_shot(u, &committed) else {
                continue;
            };
            if self.decisions.ranged_survival_weight > 0
                && self.rest_action_value(u) > self.ranged_action_value(u, target)
            {
                continue;
            }
            if let Some(f) =
                forecast_attack_mode(&self.attack_view(), u.id, target, AttackMode::Ranged)
            {
                *committed.entry(target).or_default() += f.expected_damage_to_defender;
            }
            self.orders[i] = Order::RangedAttack { unit, target };
        }
    }

    /// Expected damage already ordered onto each tile by this turn's shots.
    pub(crate) fn committed_fire(&self) -> std::collections::BTreeMap<Hex, i32> {
        let mut committed = std::collections::BTreeMap::<Hex, i32>::new();
        for o in &self.orders {
            if let Order::RangedAttack { unit, target } = *o
                && let Some(f) =
                    forecast_attack_mode(&self.attack_view(), unit, target, AttackMode::Ranged)
            {
                *committed.entry(target).or_default() += f.expected_damage_to_defender;
            }
        }
        committed
    }

    /// The best visible target in range, by value: a unit by the threat a shot removes
    /// (`shot_value`), a city with HP by the capture progress it makes (`city_shot_value`),
    /// then kill chance and damage relative to HP. A city with HP shields its garrison (core
    /// `combat::city_tile_defender`), so every shot at its tile is scored as city damage, and
    /// shots in one phase share a snapshot: damage past the city's 0 HP is wasted.
    pub(crate) fn best_shot(
        &self,
        u: &UnitView,
        committed: &std::collections::BTreeMap<Hex, i32>,
    ) -> Option<Hex> {
        self.best_shot_choosing(u, committed, false)
    }

    /// `best_shot` for a standing order the player set (a garrison's auto-fire): never a
    /// barbarian when they spare us (`PlayerView::is_threat`), since the Celts shoot those only by
    /// an explicit order.
    pub(crate) fn best_shot_automatic(
        &self,
        u: &UnitView,
        committed: &std::collections::BTreeMap<Hex, i32>,
    ) -> Option<Hex> {
        self.best_shot_choosing(u, committed, true)
    }

    fn best_shot_choosing(
        &self,
        u: &UnitView,
        committed: &std::collections::BTreeMap<Hex, i32>,
        automatic: bool,
    ) -> Option<Hex> {
        if !u.military || u.ranged <= 0 || u.range <= 0 {
            return None;
        }
        let mut best: Option<((i64, i64, i64), Hex)> = None;
        let mut consider = |key: (i64, i64, i64), pos: Hex| {
            if best.is_none_or(|(b, bp)| key > b || (key == b && pos < bp)) {
                best = Some((key, pos));
            }
        };
        // Ranged attacks hit military units and cities only (not civilians).
        for e in &self.ctx.enemy_units {
            let d = u.pos.distance(e.pos);
            let remaining = e.hp - committed.get(&e.pos).copied().unwrap_or(0);
            if !e.military
                || automatic && !self.view().is_threat(u.owner, e.owner)
                || d < 1
                || d > u.range
                || remaining <= 0
                || !self.ranged_unit_clear(u, u.pos, e.pos)
                || legatum_core::city_attack_target(self.view(), e.pos)
                    == Some(legatum_core::combat::CityTileDefender::City)
            {
                continue;
            }
            let Some(f) =
                forecast_attack_mode(&self.attack_view(), u.id, e.pos, AttackMode::Ranged)
            else {
                continue;
            };
            if f.garrison.as_ref().is_some_and(|g| g.shielded) {
                continue;
            }
            let captive_opportunity = if self.faction == Faction::Aztecs
                && self.rule_on(legatum_core::experiments::FactionRule::AztecsFlowerWars)
                && f.win_chance >= 80
            {
                self.ctx
                    .own_units
                    .iter()
                    .filter(|m| crate::composition::is_melee_line(m.kind) && !self.has_acted(m.id))
                    .filter_map(|m| self.melee_forecast(m, e.pos).map(|forecast| (m, forecast)))
                    .filter(|(m, forecast)| {
                        forecast.win_chance >= 80
                            && forecast.loss_chance <= 10
                            && self.reaches_this_turn(m, e.pos)
                            && self.melee_reply_pays(m, e, forecast)
                    })
                    .map(|(m, forecast)| i64::from(self.flower_wars_value(m, e, &forecast)) * 1000)
                    .max()
                    .unwrap_or(0)
            } else {
                0
            };
            let denial = i64::from(crate::hold::denial_priority(self.view(), e.owner, || 0));
            consider(
                (
                    (self.shot_value(e, remaining, &f) + self.sparta_kill_heal_value(u, &f)
                        - captive_opportunity)
                        * (100 + denial)
                        / 100,
                    i64::from(f.win_chance),
                    i64::from(f.expected_damage_to_defender) * 100 / i64::from(remaining),
                ),
                e.pos,
            );
        }
        for c in &self.ctx.enemy_cities {
            if self
                .campaigns
                .active
                .as_ref()
                .is_some_and(|campaign| c.id != campaign.target)
            {
                continue;
            }
            let d = u.pos.distance(c.pos);
            let remaining = c.hp - committed.get(&c.pos).copied().unwrap_or(0);
            // At zero HP a city needs occupation (or its garrison shot, above).
            if !c.visible
                || d < 1
                || d > u.range
                || c.hp <= 0
                || remaining <= 0
                || !self.ranged_unit_clear(u, u.pos, c.pos)
            {
                continue;
            }
            let Some(f) =
                forecast_attack_mode(&self.attack_view(), u.id, c.pos, AttackMode::Ranged)
            else {
                continue;
            };
            let denial = i64::from(self.hold_denial_city(c));
            consider(
                (
                    self.city_shot_value(c.pos, remaining, &f) * (100 + denial) / 100,
                    i64::from(f.win_chance),
                    i64::from(f.expected_damage_to_defender) * 100 / i64::from(remaining),
                ),
                c.pos,
            );
        }
        best.map(|(_, target)| target)
    }

    /// The best adjacent enemy this unit can profitably attack on its own, if any.
    pub(crate) fn best_strike(&self, u: &UnitView) -> Option<Hex> {
        let threshold = self.attack_threshold();
        let mine = attack_power(self.view(), u);
        let mut best: Option<(i32, Hex)> = None;
        for e in &self.ctx.enemy_units {
            if u.pos.distance(e.pos) != 1 || self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos) {
                continue;
            }
            let their = if e.military {
                defense_power(self.ctx.view, e)
            } else {
                0
            };
            let ratio = mine * 100 / their.max(1);
            if ratio >= threshold
                && (!e.military || !self.too_risky(u, e.pos))
                && (!e.military
                    || self
                        .melee_forecast(u, e.pos)
                        .is_some_and(|f| self.melee_reply_pays(u, e, &f)))
                && best.is_none_or(|(b, bp)| ratio > b || (ratio == b && e.pos < bp))
            {
                best = Some((ratio, e.pos));
            }
        }
        best.map(|(_, h)| h)
    }

    fn rush_ignores_home_barbarians(&self, city: usize) -> bool {
        // Public rule: barbarians raid civilians but never assault cities. A rush
        // with no settlers need not keep its entire field force guarding an empty home.
        self.rushing()
            && self.ctx.threat[city] == self.ctx.barbarian_threat[city]
            && !self
                .ctx
                .own_units
                .iter()
                .any(|u| u.kind == UnitKind::Settler)
    }

    /// Threatened cities and valuable supply on their land pull in free defenders. A deposit
    /// exposed beyond the ordinary city-threat radius still matters when losing its supply
    /// would displace useful users.
    fn defend_cities(&mut self, army: &[&UnitView]) {
        let mut order: Vec<(i32, i32, &CityView)> = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if self.rush_ignores_home_barbarians(i) {
                    return None;
                }
                let (stake, deposit_threat) = self
                    .view()
                    .supply
                    .iter()
                    .flat_map(|s| s.deposits.iter().map(move |d| (s.resource, d)))
                    .filter(|(_, d)| d.city == Some(c.id))
                    .map(|(resource, d)| {
                        let stake = crate::supply_value::own_loss(self.view(), resource, d.supply);
                        let threat = if stake > 0 {
                            self.ctx
                                .threat_units
                                .iter()
                                .filter(|e| e.military && e.pos.distance(d.hex) <= 3)
                                .map(|e| offense_power(self.view(), e))
                                .sum()
                        } else {
                            0
                        };
                        (stake, threat)
                    })
                    .max_by_key(|(stake, threat)| (i32::from(*threat > 0), *stake))
                    .unwrap_or((0, 0));
                let threat = self.ctx.threat[i].max(deposit_threat);
                (threat > 0).then_some((threat, stake, *c))
            })
            .collect();
        // A Hall city (its land is open to every empire) is answered first, and widest.
        order.sort_by(|a, b| {
            self.holds_hall_city(b.2)
                .cmp(&self.holds_hall_city(a.2))
                .then(b.0.cmp(&a.0))
                .then(b.1.cmp(&a.1))
                .then(a.2.id.cmp(&b.2.id))
        });
        for (threat, _, city) in order {
            // An attacker needs local superiority; Hard answers with a margin over the
            // visible attack instead of merely matching it.
            let threat = if self.hard_defense() {
                threat * self.decisions.defend_margin_percent / 100
            } else {
                threat
            };
            let threat = threat * self.hall_defense_percent(city) / 100;
            // Walls buy time and fire back, but do not stop a siege. Reserve actual troops against ranged
            // pressure, and do not count a unit that already retreated or marched away.
            let bombardment = self
                .ctx
                .threat_units
                .iter()
                .any(|e| e.ranged > 0 && e.pos.distance(city.pos) <= e.range + 1);
            let mut have = if bombardment {
                0
            } else {
                city_power(self.view(), city) / 2
            } + army
                .iter()
                .filter(|u| {
                    self.has_acted(u.id)
                        && self
                            .orders
                            .iter()
                            .find_map(|o| match o {
                                Order::MoveUnit { unit, to } if *unit == u.id => Some(*to),
                                _ => None,
                            })
                            .unwrap_or(u.pos)
                            .distance(city.pos)
                            <= 2
                })
                .map(|u| defense_power(self.ctx.view, u))
                .sum::<i32>();
            let mut near: Vec<&UnitView> = self
                .free(army)
                .into_iter()
                .filter(|u| {
                    u.pos.distance(city.pos) <= DEFEND_RADIUS && self.ctx.reachable(u.pos, city.pos)
                })
                .collect();
            // Civ 6: a healthy city absorbs attacks for its garrison, so a ranged garrison
            // fires every turn in safety while melee units stay free to strike.
            let ranged_garrison = Self::prefers_ranged_garrison(city)
                && !self
                    .ctx
                    .view
                    .index(city.pos)
                    .is_some_and(|i| self.ctx.friendly_military_at[i]);
            near.sort_by_key(|u| {
                (
                    ranged_garrison && u.ranged == 0,
                    u.pos.distance(city.pos),
                    u.id,
                )
            });
            for u in near {
                if have >= threat {
                    break;
                }
                if self.defend(u, city) {
                    have += defense_power(self.ctx.view, u).max(attack_power(self.view(), u));
                }
            }
        }
    }

    /// Two full held turns are already recorded by fortification. Surplus defenders
    /// still choose a useful position even after the ordinary defense quota is filled.
    fn commit_idle_defenders(&mut self, army: &[&UnitView]) {
        if self.rushing() {
            return;
        }
        for u in self.free(army) {
            if u.hp < self.decisions.recover_hp
                || u.fortify < 2 * data::FORTIFY_PER_TURN
                || self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
            {
                continue;
            }
            let city = self
                .ctx
                .own_cities
                .iter()
                .copied()
                .filter(|c| {
                    u.pos.distance(c.pos) <= 3
                        && self
                            .ctx
                            .threat_units
                            .iter()
                            .any(|e| e.military && e.pos.distance(c.pos) <= 3)
                })
                .min_by_key(|c| (u.pos.distance(c.pos), c.id));
            let Some(city) = city else { continue };
            // Ranged fire and immediate favorable melee already had priority above.
            // A ranged field defender can also finish a favorable melee exchange.
            if let Some(to) = self.counterattack_target(u, None) {
                self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
                continue;
            }
            let spot = city
                .pos
                .within(HOME_RADIUS)
                .into_iter()
                .filter(|h| {
                    self.free_tile_for(u, *h)
                        && self.can_defend_at(u, *h)
                        && self.outside_remembered_reach(*h)
                        && (*h == u.pos || self.reaches_this_turn(u, *h))
                })
                .min_by_key(|h| {
                    let cover = self.view().tile(*h).and_then(|t| t.terrain).map_or(0, |t| {
                        data::terrain_defense_rules(
                            self.view().rule_experiments.faction_rules_off,
                            self.faction,
                            t,
                        )
                    });
                    let approach = self
                        .ctx
                        .threat_units
                        .iter()
                        .filter(|e| e.military)
                        .map(|e| e.pos.distance(*h))
                        .min()
                        .unwrap_or(0);
                    (
                        i32::from(*h != city.pos),
                        -cover,
                        h.distance(city.pos),
                        approach,
                        h.distance(u.pos),
                        *h,
                    )
                });
            if let Some(to) = spot {
                self.go_to(u, to);
            }
        }
    }

    fn defend(&mut self, u: &UnitView, city: &CityView) -> bool {
        if let Some(target) = self
            .best_strike(u)
            .filter(|h| h.distance(city.pos) <= HOME_RADIUS)
        {
            self.unit_order(
                u.id,
                Order::MoveUnit {
                    unit: u.id,
                    to: target,
                },
            );
            return true;
        }
        if self.free_tile_for(u, city.pos) && self.can_defend_at(u, city.pos) {
            self.go_to(u, city.pos);
            return true;
        }
        if u.pos.distance(city.pos) <= HOME_RADIUS
            && self.free_tile_for(u, u.pos)
            && self.can_defend_at(u, u.pos)
        {
            self.hold_or_upgrade(u);
            return true;
        }
        let mut spots: Vec<Hex> = city
            .pos
            .within(HOME_RADIUS)
            .into_iter()
            .filter(|h| self.free_tile_for(u, *h) && self.can_defend_at(u, *h))
            .collect();
        spots.sort_by_key(|h| {
            let bonus = self
                .ctx
                .view
                .tile(*h)
                .and_then(|t| t.terrain)
                .map_or(0, |t| {
                    legatum_core::data::terrain_defense_rules(
                        self.view().rule_experiments.faction_rules_off,
                        self.faction,
                        t,
                    )
                });
            (h.distance(city.pos), -bonus, h.distance(u.pos), *h)
        });
        if spots.is_empty() && self.relief.contains(&city.pos) {
            // Every safe tile beside the city is taken or under fire. Holding out of reach
            // while the city falls helps nobody: close in to strike the attackers next turn.
            let closer = city
                .pos
                .within(HOME_RADIUS + 1)
                .into_iter()
                .filter(|h| {
                    h.distance(city.pos) < u.pos.distance(city.pos)
                        && self.free_tile_for(u, *h)
                        && self.may_close_into(u, city, *h)
                })
                .min_by_key(|h| (h.distance(city.pos), h.distance(u.pos), *h));
            if let Some(h) = closer {
                self.go_to(u, h);
                return true;
            }
        }
        match spots.first() {
            Some(&h) => {
                self.go_to(u, h);
                true
            }
            None => {
                self.hold_or_upgrade(u);
                false
            }
        }
    }

    /// Gene `held_shot_stepin`: closing in on a city under siege never steps a unit into the
    /// fallen city itself when it cannot survive there. At 0 HP the unit meets the charge and
    /// the city's attackers' fire; under the held-shot rule of PR #4 (rules 47, not yet landed)
    /// the volley kills the unit before the charge lands. Either way the
    /// city falls (trace: the Swiss Horseman stepped into Geneva on T67 with `can_defend_at`
    /// false, M51-geneva, 4 of 4 salts).
    pub(crate) fn may_close_into(&self, u: &UnitView, city: &CityView, to: Hex) -> bool {
        self.decisions.held_shot_stepin == 0
            || city.hp > 0
            || to != city.pos
            || self.can_defend_at(u, to)
    }

    /// Keep an eligible defender on each city, without undoing a tactical retreat.
    fn garrison_cities(&mut self, army: &[&UnitView]) {
        // A one-city rush commits its healthy force. Threatened homes have already
        // reserved defenders; a ceremonial garrison must not strand the first soldier.
        if self.rushing() {
            if self.decision_trace.is_some() {
                for city in &self.ctx.own_cities {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?}", city.id),
                        "gated_out",
                    );
                    row.gate = Some("one-city rush releases ceremonial garrisons".into());
                    self.note_option(row);
                }
            }
            return;
        }
        let mut cities: Vec<&CityView> = self.ctx.own_cities.clone();
        cities.sort_by_key(|c| {
            (
                std::cmp::Reverse(
                    self.hall_guard_city(c)
                        || self.diwan_guard_city(c)
                        || self.wonder_guard_city(c)
                        || self.own_original_capital(c),
                ),
                std::cmp::Reverse(self.threat_exposure(c)),
                std::cmp::Reverse(self.exposed_city(c)),
                c.hp,
                c.id,
            )
        });
        // Gene `war_garrison`: while the plan fights a war, a city no rival can reach soon (no
        // threat on it, no enemy city within eight hexes, no enemy unit within five) keeps no
        // standing garrison, so the army goes to the front (user, 2026-10-18: defence is held
        // back by the threat, not by default).
        let interior_free = self.decisions.war_garrison > 0
            && self.hard_defense()
            && self.decisions.war_siege > 0
            && self
                .routed_plan()
                .and_then(|p| p.kind.war_target())
                .is_some_and(|t| self.view().teams_at_war(self.view().team, t));
        for city in cities {
            if self.mil_dest.contains(&city.pos) || self.capture_deferred.contains(&city.pos) {
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?}", city.id),
                        "gated_out",
                    );
                    row.gate = Some("destination reserved or captured-city hold deferred".into());
                    self.note_option(row);
                }
                continue;
            }
            if interior_free
                && !self.wonder_guard_city(city)
                && !self.own_original_capital(city)
                && self.threat_exposure(city) == 0
                && !self.exposed_city(city)
            {
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?}", city.id),
                        "gated_out",
                    );
                    row.gate = Some("war_garrison interior_free".into());
                    self.note_option(row);
                }
                continue;
            }
            // Gene `hollow_march`: at war with a hollow leader, a city no foreign army can be
            // seen near keeps no standing garrison.
            if self.garrison_spared(city) && !self.capital_pressed(city) {
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?}", city.id),
                        "gated_out",
                    );
                    row.gate = Some("hollow_march garrison_spared".into());
                    self.note_option(row);
                }
                continue;
            }
            // Gene `celts_playbook` part 1: barbarians never attack the Celts, so a city no
            // empire's army can come to soon keeps no standing garrison.
            if self.sheltered_city(city) && !self.capital_pressed(city) {
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?}", city.id),
                        "gated_out",
                    );
                    row.gate = Some("sheltered city".into());
                    self.note_option(row);
                }
                continue;
            }
            if let Some(u) = self.free(army).into_iter().find(|u| u.pos == city.pos) {
                let before = self.orders.len();
                self.hold_or_upgrade(u);
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?} unit={:?}", city.id, u.id),
                        "selected",
                    );
                    row.selected_order = self.orders.get(before).cloned();
                    self.note_option(row);
                }
                continue;
            }
            let occupied = self
                .ctx
                .view
                .index(city.pos)
                .is_some_and(|i| self.ctx.friendly_military_at[i]);
            if occupied {
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?}", city.id),
                        "gated_out",
                    );
                    row.gate = Some("already occupied by friendly military".into());
                    self.note_option(row);
                }
                continue;
            }
            let nearest = self
                .free(army)
                .into_iter()
                .filter(|u| {
                    // Never strip another city's only garrison to fill this one.
                    !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                })
                .filter(|u| u.pos.distance(city.pos) <= 5 && self.can_defend_at(u, city.pos))
                // Before Walls, a garrisoned shooter is what holds a city a conqueror can
                // reach: it fires every turn and the city shields it while it has HP.
                .min_by_key(|u| {
                    (
                        (self.exposed_city(city) || self.threat_exposure(city) > 0)
                            && (u.ranged == 0) == Self::prefers_ranged_garrison(city),
                        u.pos.distance(city.pos),
                        u.id,
                    )
                });
            if let Some(u) = nearest {
                let before = self.orders.len();
                self.go_to(u, city.pos);
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "garrison",
                        format!("city={:?} unit={:?}", city.id, u.id),
                        "selected",
                    );
                    row.selected_order = self.orders.get(before).cloned();
                    self.note_option(row);
                }
            } else if self.decision_trace.is_some() {
                let mut row = crate::DecisionOption::new(
                    "garrison",
                    format!("city={:?}", city.id),
                    "gated_out",
                );
                row.gate = Some("no free defender within radius 5 that can defend city".into());
                self.note_option(row);
            }
        }
    }

    /// A known ruin is a one-time economic gain for any unit that enters it. Compare that gain
    /// with travel, lost fortification and the defensive value of the unit's current post.
    /// Emergency defence and attacks have already reserved their units; this competes with
    /// routine garrisons and resting, where nearby safe ruins are usually cheap to take.
    fn collect_ruins(&mut self, army: &[&UnitView]) {
        if self.decisions.ruin_collect == 0 {
            return;
        }
        let ruins: Vec<Hex> = self
            .ctx
            .frontier
            .iter()
            .copied()
            .filter(|h| {
                self.view().tile(*h).is_some_and(|t| {
                    t.ruin && (t.owner == Some(self.view().me) || t.owner.is_none())
                }) && self.safe_campaign_tile(*h)
                    && !self.orders.iter().any(|order| {
                        matches!(order, Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } if to == h)
                    })
            })
            .collect();
        for ruin in ruins {
            let best = self
                .free(army)
                .into_iter()
                .filter(|u| self.free_tile_for(u, ruin))
                .filter_map(|u| {
                    let distance = u.pos.distance(ruin);
                    if distance == 0 {
                        return None;
                    }
                    let post_value =
                        self.ctx
                            .own_cities
                            .iter()
                            .find(|c| c.pos == u.pos)
                            .map_or(0, |c| {
                                let approaching_army = self
                                    .ctx
                                    .threat_units
                                    .iter()
                                    .filter(|e| e.military)
                                    .map(|e| e.pos.distance(c.pos))
                                    .min()
                                    .map_or(0, |d| match d {
                                        0..=3 => 100,
                                        4 => 20,
                                        5 => 5,
                                        _ => 0,
                                    });
                                let hostile_city = self
                                    .ctx
                                    .enemy_cities
                                    .iter()
                                    .map(|e| e.pos.distance(c.pos))
                                    .min()
                                    .map_or(0, |d| match d {
                                        0..=4 => 100,
                                        5..=8 => 10,
                                        _ => 0,
                                    });
                                let watched = match self.threat_exposure(c) {
                                    2 => 100,
                                    1 => 5,
                                    _ => 0,
                                };
                                5 + approaching_army.max(hostile_city).max(watched)
                            });
                    let value = self.decisions.ruin_bonus
                        - distance * 5
                        - u.fortify
                        - post_value
                        - (100 - u.hp).max(0) / 3;
                    if value <= 0 {
                        return None;
                    }
                    let to = self.campaign_approach(u, ruin)?;
                    (to != u.pos).then_some((value, u, to))
                })
                .max_by_key(|(value, u, _)| (*value, std::cmp::Reverse(u.id)));
            if let Some((_, unit, to)) = best {
                self.go_to(unit, to);
            }
        }
    }

    /// Spend a genuinely unused field unit on information or a ruin. Combat, emergency
    /// defence, ordinary garrisons and every campaign pass have already claimed their units.
    fn military_scout(&mut self, army: &[&UnitView]) {
        if !self.hard_defense() || self.decisions.military_scout == 0 {
            return;
        }
        let mut taken = Vec::new();
        for u in self.free(army) {
            if u.hp < self.march_min_hp()
                || !self.safe_campaign_tile(u.pos)
                || self.campaign_withdraw.contains(&u.id)
            {
                continue;
            }
            let blocked: Vec<bool> = (0..self.view().tiles.len())
                .map(|i| {
                    let h = self.ctx.hex_at(i);
                    !self.safe_campaign_tile(h) || !self.free_tile_for(u, h)
                })
                .collect();
            let field = self
                .ctx
                .distance_field_avoiding(u.pos, true, Some(&blocked));
            let mut candidates = Vec::new();
            for &h in &self.ctx.frontier {
                if h == u.pos || taken.iter().any(|t: &Hex| t.distance(h) <= 3) {
                    continue;
                }
                let Some(i) = self.view().index(h) else {
                    continue;
                };
                let distance = field[i];
                if distance == UNREACHABLE {
                    continue;
                }
                let ruin = self.view().tile(h).is_some_and(|t| {
                    t.ruin && (t.owner.is_none() || t.owner == Some(self.view().me))
                });
                let information = self.ctx.unexplored_near(h) * 3;
                let value = (information + if ruin { self.decisions.ruin_bonus } else { 0 })
                    * self.decisions.military_scout
                    / 100;
                let home = self
                    .ctx
                    .nearest_own_city(h)
                    .map_or(0, |c| c.pos.distance(h));
                let cost = distance * 10 + home * 2 + u.fortify + (100 - u.hp).max(0) / 3;
                if value > cost {
                    candidates.push((value - cost, h));
                }
            }
            candidates.sort_by_key(|(value, h)| (std::cmp::Reverse(*value), *h));
            for (_, target) in candidates {
                if let Some(to) = self.campaign_approach(u, target)
                    && to != u.pos
                {
                    taken.push(target);
                    self.go_to(u, to);
                    break;
                }
            }
        }
    }

    /// While a city keeps most of its HP it shields its garrison (core
    /// `combat::city_tile_defender`), so a shooter there is the stronger defence. A badly
    /// damaged city may fall this turn, and then the garrison must survive melee.
    fn prefers_ranged_garrison(city: &CityView) -> bool {
        city.hp * 2 > data::CITY_HP
    }

    fn targets(&self, include_fogged_cities: bool) -> Vec<Target> {
        let ctx = &self.ctx;
        let support = |pos: Hex| -> i32 {
            ctx.enemy_units
                .iter()
                .filter(|o| o.military && o.pos != pos && o.pos.distance(pos) <= 2)
                .map(|o| threat_power(self.view(), o) / 2)
                .sum::<i32>()
                + ctx
                    .remembered_threats
                    .iter()
                    .filter(|o| o.pos != pos && o.pos.distance(pos) <= 2)
                    .map(|o| {
                        let stats = self
                            .view()
                            .rule_experiments
                            .unit_stats_for(self.view().faction_of(o.owner), o.kind);
                        stats.strength.max(stats.ranged) / 2
                    })
                    .sum::<i32>()
        };
        let mut out = Vec::new();
        for e in &ctx.enemy_units {
            if ctx.enemy_cities.iter().any(|c| c.pos == e.pos) {
                continue;
            }
            let own = if e.military {
                defense_power(ctx.view, e)
            } else {
                0
            };
            let civilian = !e.military
                && ctx
                    .enemy_units
                    .iter()
                    .all(|o| o.pos != e.pos || !o.military);
            out.push(Target {
                pos: e.pos,
                their: own + support(e.pos),
                civilian,
                heir: civilian && self.heir_capture_value(e) > 0,
            });
        }
        for c in &ctx.enemy_cities {
            if self.takes_no_cities() || !self.assessed_campaign_allowed(c) {
                continue;
            }
            if !c.visible && !include_fogged_cities {
                continue;
            }
            out.push(Target {
                pos: c.pos,
                their: self.city_defence_weight(c),
                civilian: false,
                heir: false,
            });
        }
        out
    }

    /// A force heuristic over both layers an attacker must beat at a city (the city, then its
    /// garrison: core `combat::city_tile_defender`), plus nearby support; not a combat forecast.
    pub(crate) fn city_defence_weight(&self, c: &CityView) -> i32 {
        let ctx = &self.ctx;
        let garrison = ctx
            .enemy_units
            .iter()
            .find(|e| e.pos == c.pos && e.military);
        let mut own = match garrison {
            Some(g) => defense_power(ctx.view, g) + city_power(self.view(), c) / 4,
            None => city_power(self.view(), c),
        };
        // Sightings contain no HP or fortification. Use nominal public unit stats as
        // a bounded planning allowance, never synthesize units for combat forecasts.
        for o in ctx.remembered_threats.iter().filter(|o| o.pos == c.pos) {
            let stats = self
                .view()
                .rule_experiments
                .unit_stats_for(self.view().faction_of(o.owner), o.kind);
            own = own.max(stats.strength + city_power(self.view(), c) / 4);
        }
        let support: i32 = ctx
            .enemy_units
            .iter()
            .filter(|o| o.military && o.pos != c.pos && o.pos.distance(c.pos) <= 2)
            .map(|o| threat_power(self.view(), o) / 2)
            .sum::<i32>()
            + ctx
                .remembered_threats
                .iter()
                .filter(|o| o.pos != c.pos && o.pos.distance(c.pos) <= 2)
                .map(|o| {
                    let stats = self
                        .view()
                        .rule_experiments
                        .unit_stats_for(self.view().faction_of(o.owner), o.kind);
                    stats.strength.max(stats.ranged) / 2
                })
                .sum::<i32>();
        // A remembered garrison may have changed. This is a cautious estimate for marching,
        // never a combat forecast or permission to attack through fog.
        own + support + if c.visible { 0 } else { own / 2 }
    }

    fn attackers<'u>(
        &self,
        army: &[&'u UnitView],
        target: Hex,
        immediate_group: bool,
    ) -> Vec<&'u UnitView> {
        self.free(army)
            .into_iter()
            .filter(|u| u.pos.distance(target) <= STRIKE_RADIUS)
            .filter(|u| self.ctx.reachable(u.pos, target))
            .filter(|u| {
                !immediate_group
                    || (crate::composition::charges(u.kind)
                        && !self.too_risky(u, target)
                        && self.reaches_this_turn(u, target))
            })
            .collect()
    }

    /// Ranged troops that have already fired still support this turn's siege. The
    /// ordinary free-unit accounting would otherwise discard the entire ranged wing
    /// immediately before deciding whether its melee partners may close in.
    fn committed_fire_support(&self, army: &[&UnitView], target: Hex) -> i32 {
        if !self.ctx.enemy_cities.iter().any(|c| c.pos == target) {
            return 0;
        }
        army.iter()
            .filter(|u| {
                u.hp >= self.march_min_hp() && self.orders.iter().any(|o| {
            matches!(o, Order::RangedAttack { unit, target: h } if *unit == u.id && *h == target)
        })
            })
            .map(|u| offense_power(self.view(), u))
            .sum()
    }

    pub(crate) fn attack_force(&self, units: &[&UnitView]) -> i32 {
        units
            .iter()
            .map(|u| {
                let mut after = (*u).clone();
                after.hp = (after.hp - self.city_damage_now(u)).max(0);
                offense_power(self.view(), &after)
            })
            .sum()
    }

    /// Reserve favorable groups that can attack now before retreat takes their orders.
    /// The later pass also permits longer approaches once home defense is assigned.
    fn attack_targets(&mut self, army: &[&UnitView], immediate_group: bool) {
        let threshold = self.attack_threshold();
        let mut candidates: Vec<(i32, Target)> = Vec::new();
        for t in self.targets(false) {
            // Couriers raid exposed civilians but do not take field battles far from home.
            if self.decisions.courier_raid_only > 0
                && !t.civilian
                && self
                    .ctx
                    .enemy_units
                    .iter()
                    .any(|u| u.military && u.pos == t.pos)
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .all(|c| c.pos.distance(t.pos) > 3)
            {
                continue;
            }
            // Live cities use the group capture forecast before recovery. A failed
            // assault must not become a stream of individually approved charges.
            if self.ctx.enemy_cities.iter().any(|c| c.pos == t.pos) {
                continue;
            }
            let near = self.attackers(army, t.pos, immediate_group);
            if immediate_group && near.len() < 2 {
                continue;
            }
            let mine: i32 = self.attack_force(&near) + self.committed_fire_support(army, t.pos);
            if mine == 0 {
                continue;
            }
            let ratio = mine * 100 / t.their.max(1);
            if ratio >= if t.heir { threshold / 2 } else { threshold } {
                candidates.push((ratio + if t.heir { 10_000 } else { 0 }, t));
            }
        }
        // A target that denies a team counting down to victory comes first (`hold.rs`).
        let denial = |t: &Target| self.hold_denial_at(t.pos);
        candidates.sort_by(|a, b| {
            denial(&b.1)
                .cmp(&denial(&a.1))
                .then(b.0.cmp(&a.0))
                .then(a.1.pos.cmp(&b.1.pos))
        });
        for (_, t) in candidates {
            let mut near = self.attackers(army, t.pos, immediate_group);
            if immediate_group && near.len() < 2 {
                continue;
            }
            near.sort_by_key(|u| (u.pos.distance(t.pos), u.id));
            // Several melee orders on a defeated city would all claim its empty tile and
            // bounce. Give occupation to one survivor; the rest remain available elsewhere.
            let occupation = self
                .ctx
                .enemy_cities
                .iter()
                .any(|c| c.pos == t.pos && c.hp == 0)
                && !self
                    .ctx
                    .enemy_units
                    .iter()
                    .any(|e| e.pos == t.pos && e.military);
            if occupation {
                if self.mil_dest.contains(&t.pos) {
                    continue;
                }
                if let Some(u) = near.iter().find(|u| {
                    (!self.rushing() || crate::composition::charges(u.kind))
                        && self
                            .melee_forecast(u, t.pos)
                            .is_some_and(|f| f.loss_chance < 25)
                }) {
                    // The early and late attack passes share this one occupation slot.
                    self.strike(u, t.pos, immediate_group);
                    self.mil_dest.insert(t.pos);
                }
                continue;
            }
            let mine: i32 = self.attack_force(&near) + self.committed_fire_support(army, t.pos);
            if mine == 0
                || mine * 100 / t.their.max(1) < if t.heir { threshold / 2 } else { threshold }
            {
                continue;
            }
            if t.civilian {
                if let Some(u) = near
                    .into_iter()
                    .find(|u| crate::composition::charges(u.kind))
                {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: t.pos,
                        },
                    );
                }
                continue;
            }
            if !immediate_group && self.stage_assault(&near, &t) {
                near.retain(|u| !self.has_acted(u.id));
            }
            // The distant wing contributes to a campaign's eventual strength, but it cannot
            // absorb the defender's reply this turn. Price an immediate city assault from the
            // units that can actually reach its garrison before issuing any charge.
            if !immediate_group
                && self.decisions.melee_response_weight > 0
                && self.ctx.enemy_cities.iter().any(|c| c.pos == t.pos)
                && self
                    .ctx
                    .enemy_units
                    .iter()
                    .any(|e| e.pos == t.pos && e.military)
            {
                let arriving: Vec<_> = near
                    .iter()
                    .copied()
                    .filter(|u| self.reaches_this_turn(u, t.pos))
                    .collect();
                let current =
                    self.attack_force(&arriving) + self.committed_fire_support(army, t.pos);
                if current * 100 < threshold * t.their.max(1) {
                    continue;
                }
            }
            for u in near {
                self.strike(u, t.pos, immediate_group);
            }
        }
    }

    /// An approved visible assault may spend its strength margin on one exposed staging
    /// turn. Count only healthy melee units that can arrive next to the city together.
    /// Routine approaches and reconnaissance still use `campaign_staging_spot` below.
    fn stage_assault(&mut self, near: &[&UnitView], target: &Target) -> bool {
        if !self
            .ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos == target.pos && c.visible)
            || !self.view().tile(target.pos).is_some_and(|t| t.visible)
            || near.iter().all(|u| self.reaches_this_turn(u, target.pos))
        {
            return false;
        }
        let mut staged = Vec::new();
        let mut reserved = self.mil_dest.clone();
        // find_path reads only the map's geometry.
        let map = self.movement_map();
        for &u in near {
            if crate::composition::shooter_only(u.kind, u.ranged)
                || u.hp < self.march_min_hp()
                || self.too_risky(u, target.pos)
            {
                continue;
            }
            let mut spots: Vec<_> = target
                .pos
                .neighbors()
                .into_iter()
                .filter(|h| {
                    !reserved.contains(h)
                        && self.free_tile_for(u, *h)
                        && self.reaches_this_turn(u, *h)
                })
                .collect();
            spots.sort_by_key(|h| {
                (
                    self.exposure_key(*h),
                    h.distance(u.pos),
                    std::cmp::Reverse(self.phalanx_spot_among(u, *h, &reserved)),
                    *h,
                )
            });
            let spot = spots.into_iter().find(|h| {
                find_path(map, u.pos, *h, |step| {
                    self.ctx
                        .move_cost(step)
                        .filter(|_| self.free_tile_for(u, step))
                })
                .is_some_and(|path| path.iter().all(|step| self.outside_remembered_reach(*step)))
            });
            if let Some(to) = spot {
                reserved.insert(to);
                staged.push((u, to));
            }
        }
        let remaining = self.assault_force_after_exposure(&staged);
        if staged.len() < 2
            || remaining == 0
            || (remaining + self.committed_fire_support(&self.ctx.own_units, target.pos)) * 100
                < self.attack_threshold() * target.their.max(1)
        {
            return false;
        }
        // Final movement reconciliation must cancel the whole group if traffic leaves
        // even one member short of its forecast arrival.
        self.assault_groups.push(
            staged
                .iter()
                .filter(|(u, to)| u.pos != *to)
                .map(|(u, _)| u.id)
                .collect(),
        );
        for (u, to) in staged {
            self.go_to(u, to);
        }
        true
    }

    /// Forecast visible defenders attacking the proposed arrivals, using only our view.
    /// Each enemy gets one attack, choosing its largest expected reduction of our force.
    /// Charge successive attacks at the resulting HP; take no credit for retaliation.
    fn assault_force_after_exposure(&self, staged: &[(&UnitView, Hex)]) -> i32 {
        let mut projected = self.view().clone();
        for (u, to) in staged {
            let arrival = projected.units.iter_mut().find(|v| v.id == u.id).unwrap();
            arrival.hp = (arrival.hp - self.city_damage_now(u)).max(0);
            arrival.pos = *to;
            if *to != u.pos {
                arrival.fortify = 0;
            }
        }
        let mut city_damage = std::collections::BTreeMap::<legatum_core::UnitId, i32>::new();
        for city in &self.ctx.enemy_cities {
            // Only committed participants can be relied on to absorb staging fire.
            let target = staged
                .iter()
                .filter_map(|(u, _)| projected.units.iter().find(|v| v.id == u.id))
                .filter(|u| u.hp > 0)
                .filter_map(|u| forecast_city_strike(&projected, city, u, u.pos).map(|s| (u, s)))
                .min_by_key(|(u, _)| (u.hp, u.id));
            if let Some((u, shot)) = target {
                *city_damage.entry(u.id).or_default() += shot.expected_damage(u.hp);
            }
        }
        for (id, damage) in city_damage {
            let arrival = projected.units.iter_mut().find(|u| u.id == id).unwrap();
            arrival.hp -= damage * self.decisions.staging_damage_percent / 100;
        }
        for enemy in self.ctx.threat_units.iter().filter(|e| e.military) {
            // The forecast API requires an owned attacker. Changing perspective on this
            // copy exposes no hidden information: it still contains only our observations.
            projected.me = enemy.owner;
            projected.team = projected.team_of(enemy.owner);
            let mut worst = None;
            for (u, to) in staged {
                let arrival = projected.units.iter().find(|v| v.id == u.id).unwrap();
                if arrival.hp < self.march_min_hp() {
                    continue;
                }
                for mode in [AttackMode::Melee, AttackMode::Ranged] {
                    // Melee forecasts assume arrival; bound enemy reach conservatively.
                    if mode == AttackMode::Melee
                        && (enemy.pos.distance(*to) > enemy.max_moves
                            || !self.ctx.reachable(enemy.pos, *to))
                    {
                        continue;
                    }
                    let Some(f) = forecast_attack_mode(&projected, enemy.id, *to, mode) else {
                        continue;
                    };
                    let mut damaged = arrival.clone();
                    damaged.hp -=
                        f.expected_damage_to_defender * self.decisions.staging_damage_percent / 100;
                    // Troops that would recover instead of attacking cannot support the
                    // next turn's assault, even if they survive the forecast damage.
                    let remaining = if damaged.hp >= self.march_min_hp() {
                        offense_power(&projected, &damaged)
                    } else {
                        0
                    };
                    let loss = offense_power(&projected, arrival) - remaining;
                    let key = (loss, f.expected_damage_to_defender, std::cmp::Reverse(u.id));
                    if worst.as_ref().is_none_or(|(best, _)| key > *best) {
                        worst = Some((key, damaged));
                    }
                }
            }
            if let Some((_, damaged)) = worst {
                let arrival = projected
                    .units
                    .iter_mut()
                    .find(|v| v.id == damaged.id)
                    .unwrap();
                *arrival = damaged;
            }
        }
        let ready: Vec<_> = staged
            .iter()
            .filter_map(|(u, _)| projected.units.iter().find(|v| v.id == u.id))
            .filter(|u| u.hp >= self.march_min_hp())
            .collect();
        if ready.len() < 2 {
            return 0;
        }
        ready.iter().map(|u| offense_power(&projected, u)).sum()
    }

    pub(crate) fn melee_forecast(
        &self,
        u: &UnitView,
        target: Hex,
    ) -> Option<legatum_core::BattleForecast> {
        let forecast = forecast_attack_mode(&self.attack_view(), u.id, target, AttackMode::Melee)?;
        let damage = self.attack_city_damage(u, &forecast);
        // Volleys share the original snapshot. Credit only the low-roll damage; excess
        // city-scale damage reaches a garrison through damage_tile_defender.
        let volley: i32 = self
            .orders
            .iter()
            .filter_map(|order| {
                let Order::RangedAttack { unit, target: at } = order else {
                    return None;
                };
                if *at != target {
                    return None;
                }
                let shot =
                    forecast_attack_mode(&self.attack_view(), *unit, target, AttackMode::Ranged)?;
                Some(legatum_core::combat::damage(
                    shot.attacker_strength.total,
                    shot.defender_strength.total,
                    legatum_core::combat::ROLL_MIN,
                ))
            })
            .sum();
        if damage == 0 && volley == 0 {
            return Some(self.queued_split_forecast(u, target, forecast));
        }
        if damage >= u.hp {
            return None;
        }
        let mut projected = self.view().clone();
        projected.units.iter_mut().find(|v| v.id == u.id)?.hp -= damage;
        crate::analysis::damage_tile_defender(&mut projected, target, volley);
        let mut after = forecast_attack_mode(&projected, u.id, target, AttackMode::Melee)?;
        after.city_strike = forecast.city_strike;
        Some(self.queued_split_forecast(u, target, after))
    }

    /// Orders already aimed at this defender share its one counterattack. Approximate a common
    /// movement stage for tactical scoring; the resolver remains the authority on arrival rounds.
    fn queued_split_forecast(
        &self,
        u: &UnitView,
        target: Hex,
        mut f: legatum_core::BattleForecast,
    ) -> legatum_core::BattleForecast {
        let others = self
            .orders
            .iter()
            .filter(|order| match order {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                    *unit != u.id && *to == target
                }
                _ => false,
            })
            .count();
        if others == 0 {
            return f;
        }
        let count = (others + 1) as i32;
        f.expected_damage_to_attacker /= count;
        let losses = (legatum_core::combat::ROLL_MIN..=legatum_core::combat::ROLL_MAX)
            .filter(|&roll| {
                (legatum_core::combat::damage(
                    f.defender_strength.total,
                    f.attacker_strength.total,
                    roll,
                ) + count
                    - 1)
                    / count
                    >= f.attacker_hp
            })
            .count() as u32;
        f.loss_chance = losses * 100
            / (legatum_core::combat::ROLL_MAX - legatum_core::combat::ROLL_MIN + 1) as u32;
        f
    }

    /// Whether this unit attacking `target` alone would likely just die (exact forecast).
    pub(crate) fn too_risky(&self, u: &UnitView, target: Hex) -> bool {
        let max_loss = crate::search::Stance::max_loss(
            self.campaigns.plan.stance,
            40 + self.knobs.aggression / 3,
        );
        if !self.view().tile(target).is_some_and(|t| t.visible) {
            return true;
        }
        self.melee_forecast(u, target)
            .is_none_or(|f| f.loss_chance as i32 > max_loss && f.win_chance < 50)
    }

    pub(crate) fn strike(&mut self, u: &UnitView, target: Hex, immediate: bool) {
        let occupy = self
            .ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos == target && c.visible && c.hp == 0)
            && !self
                .ctx
                .enemy_units
                .iter()
                .any(|e| e.pos == target && e.military);
        if crate::composition::charges(u.kind) || occupy {
            let d = u.pos.distance(target);
            // Immediate groups already checked the full movement budget and route.
            // Fast infantry counted in that group must attack, not stop at staging.
            if immediate || (d <= 2 && self.reaches_this_turn(u, target)) {
                let reply_unprofitable = !immediate
                    && self.decisions.melee_response_weight > 0
                    && self
                        .ctx
                        .enemy_units
                        .iter()
                        .find(|e| e.pos == target && e.military)
                        .and_then(|e| self.melee_forecast(u, target).map(|f| (e, f)))
                        .is_some_and(|(e, f)| !self.melee_reply_pays(u, e, &f));
                if self.too_risky(u, target) || reply_unprofitable {
                    self.hold_or_upgrade(u);
                } else {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: target,
                        },
                    );
                }
                return;
            }
            // Close in on a free tile next to the target rather than all converging on the
            // target's own tile (friendly units stepping into one tile all bounce).
            match self
                .campaign_staging_spot(u, target, 1)
                .or_else(|| self.storm_ring_spot(u, target, &self.mil_dest))
            {
                Some(to) => self.go_to(u, to),
                None => self.hold_or_upgrade(u),
            }
            return;
        }
        let d = u.pos.distance(target);
        let visible = self.ctx.view.tile(target).is_some_and(|t| t.visible);
        if d >= 1
            && d <= u.range
            && visible
            && self.ranged_unit_clear(u, u.pos, target)
            && forecast_attack_mode(&self.attack_view(), u.id, target, AttackMode::Ranged).is_some()
        {
            self.mil_dest.insert(u.pos);
            self.unit_order(u.id, Order::RangedAttack { unit: u.id, target });
            return;
        }
        // Get in range without walking into melee.
        let spot = target
            .ring(u.range)
            .into_iter()
            .filter(|h| {
                self.free_tile_for(u, *h)
                    && self.ranged_unit_clear(u, *h, target)
                    && u.hp - self.city_damage_now(u) - self.city_damage_at(u, *h)
                        >= self.decisions.recover_hp
            })
            .min_by_key(|h| {
                (
                    self.city_damage_at(u, *h),
                    std::cmp::Reverse(
                        self.view().tile(*h).and_then(|t| t.terrain) == Some(Terrain::Hills),
                    ),
                    std::cmp::Reverse(
                        self.view().tile(*h).and_then(|t| t.terrain) == Some(Terrain::Forest),
                    ),
                    h.distance(u.pos),
                    *h,
                )
            });
        match spot {
            Some(h) => self.go_to(u, h),
            None => self.hold_or_upgrade(u),
        }
    }

    /// A free tile next to a fallen (0 HP) enemy city for a unit whose own charge at it passes
    /// `too_risky`. `safe_campaign_tile` rejects every tile beside a garrisoned city, so without
    /// this a healthy unit can never join the ring that storms it (his 8.9.36 Celts game: a
    /// 100-HP Cuirassier sat two tiles out for seven turns while three wounded units fought).
    /// The garrison it would be exposed to is the one its own forecast already accepts fighting;
    /// a fallen city does not fire. Nearest by route (through friends), then least exposed.
    pub(crate) fn storm_ring_spot(
        &self,
        u: &UnitView,
        target: Hex,
        taken: &crate::analysis::HexSet,
    ) -> Option<Hex> {
        let fallen = self
            .ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos == target && c.visible && c.hp == 0);
        if !fallen || self.too_risky(u, target) {
            return None;
        }
        let map = self.movement_map();
        // Units pass through their own (`path::route_around_friends`): only foreign units
        // block the way, so the ring's far side counts even when friends hold the near side.
        let cost = |h| {
            let i = self.view().index(h)?;
            self.ctx
                .move_cost(h)
                .filter(|_| !self.ctx.hostile_at[i] && !self.ctx.neutral_at[i])
        };
        target
            .ring(1)
            .into_iter()
            .filter(|h| self.free_tile_for(u, *h) && !taken.contains(h))
            .filter_map(|h| {
                let path = find_path(map, u.pos, h, cost)?;
                let steps: i32 = path.iter().filter_map(|s| self.ctx.move_cost(*s)).sum();
                Some(((steps, self.exposure_key(h), h), h))
            })
            .min_by_key(|(key, _)| *key)
            .map(|(_, h)| h)
    }

    /// Keep an arrived escort through the founding turn; recruit for settlers still walking.
    fn escort_settlers(&mut self, army: &[&UnitView]) {
        let sites = self.settle_targets.clone();
        for site in sites {
            let far = self
                .ctx
                .nearest_own_city(site)
                .is_none_or(|c| c.pos.distance(site) > self.decisions.escort_home_distance);
            if !far {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("escort", format!("site={site:?}"), "gated_out");
                    row.gate = Some(format!(
                        "within escort_home_distance {}",
                        self.decisions.escort_home_distance
                    ));
                    self.note_option(row);
                }
                continue;
            }
            // Arrival clears goto. Reassign from the active site and our visible units,
            // before treating the occupant as a blocker to a replacement escort.
            if let Some(u) = self
                .free(army)
                .into_iter()
                .find(|u| u.pos == site && self.free_tile_for(u, site))
            {
                let before = self.orders.len();
                self.hold_or_upgrade(u);
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "escort",
                        format!("site={site:?} unit={:?}", u.id),
                        "selected",
                    );
                    row.selected_order = self.orders.get(before).cloned();
                    self.note_option(row);
                }
                continue;
            }
            if !self.escort_sites.contains(&site) || !self.free_tile_for_any(site) {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("escort", format!("site={site:?}"), "gated_out");
                    row.gate = Some(
                        if !self.escort_sites.contains(&site) {
                            "not an active walking site"
                        } else {
                            "site tile occupied or reserved"
                        }
                        .into(),
                    );
                    self.note_option(row);
                }
                continue;
            }
            let escort = self
                .free(army)
                .into_iter()
                .filter(|u| u.pos.distance(site) <= self.decisions.escort_search_distance)
                .min_by_key(|u| (u.pos.distance(site), u.id));
            if let Some(u) = escort {
                let before = self.orders.len();
                self.go_to(u, site);
                if self.decision_trace.is_some() {
                    let mut row = crate::DecisionOption::new(
                        "escort",
                        format!("site={site:?} unit={:?}", u.id),
                        "selected",
                    );
                    row.selected_order = self.orders.get(before).cloned();
                    self.note_option(row);
                }
            } else if self.decision_trace.is_some() {
                let mut row =
                    crate::DecisionOption::new("escort", format!("site={site:?}"), "gated_out");
                row.gate = Some(format!(
                    "no free escort within radius {}",
                    self.decisions.escort_search_distance
                ));
                self.note_option(row);
            }
        }
    }

    fn free_tile_for_any(&self, h: Hex) -> bool {
        self.ctx.view.index(h).is_some_and(|i| {
            !self.mil_dest.contains(&h)
                && !self.ctx.friendly_military_at[i]
                && !self.ctx.hostile_at[i]
                && !self.ctx.neutral_at[i]
        })
    }

    pub(crate) fn has_reachable_enemy_city(&self) -> bool {
        self.ctx.enemy_cities.iter().any(|e| {
            self.ctx
                .own_cities
                .iter()
                .any(|c| self.ctx.reachable(c.pos, e.pos))
        })
    }

    /// Routine scouting and army approaches avoid positions our retreat policy would reject
    /// if a recent sighting became visible again. Reach is deliberately conservative, just
    /// like visible exposure; it does not predict an unseen unit's movement.
    pub(crate) fn safe_campaign_tile(&self, pos: Hex) -> bool {
        let Some(i) = self.view().index(pos) else {
            return self.compute_safe_campaign_tile(pos);
        };
        if let Some(safe) = self.safe_memo.borrow().get(i).copied().flatten() {
            return safe;
        }
        let safe = self.compute_safe_campaign_tile(pos);
        let mut memo = self.safe_memo.borrow_mut();
        if memo.is_empty() {
            memo.resize(self.view().tiles.len(), None);
        }
        memo[i] = Some(safe);
        safe
    }

    fn compute_safe_campaign_tile(&self, pos: Hex) -> bool {
        self.exposure_at(pos) == 0 && self.outside_remembered_reach(pos)
    }

    pub(crate) fn outside_remembered_reach(&self, pos: Hex) -> bool {
        self.ctx.remembered_threats.iter().all(|o| {
            let stats = self
                .view()
                .rule_experiments
                .unit_stats_for(self.view().faction_of(o.owner), o.kind);
            let distance = o.pos.distance(pos);
            !(stats.ranged > 0 && distance <= stats.range
                || distance <= forecast_moves(self.view(), o.owner, o.kind)
                    && self.ctx.reachable(o.pos, pos))
        })
    }

    /// A safe distant destination is insufficient: terrain or changing friendly occupancy
    /// can leave the unit exposed en route. Near a threat, issue an adjacent safe waypoint
    /// on a safe route, so the engine cannot repath a distant order through the hazard.
    /// The next step toward `to` avoiding danger, or `to` itself when the whole way is safe.
    ///
    /// Once a search for a unit fails, the tiles it can reach at all (with or without the
    /// safety filter) are flood-filled and kept for the rest of the planning pass, so later
    /// searches that must fail are skipped. Same answers: a path exists exactly when the goal
    /// is in the flood fill of the same step rule, and reach only shrinks during a pass
    /// (`mil_dest` only grows; the occupancy tables and safety memo are fixed).
    pub(crate) fn campaign_approach(&self, u: &UnitView, to: Hex) -> Option<Hex> {
        let reach = self
            .approach_reach
            .borrow_mut()
            .entry(u.id)
            .or_default()
            .clone();
        let reach = &*reach;
        let safe = |h| {
            if u.kind == UnitKind::Scout {
                self.safe_scout_tile(u, h)
            } else {
                self.safe_campaign_tile(h)
            }
        };
        if to == u.pos {
            return safe(to).then_some(to);
        }
        // find_path reads only the map's geometry; costs come from the closures.
        let map = self.movement_map();
        let cost = |h| self.ctx.move_cost(h).filter(|_| self.free_tile_for(u, h));
        let index = map.index(to);
        let blocked = |set: &std::cell::OnceCell<Vec<bool>>| {
            set.get().is_some_and(|r| index.is_none_or(|i| !r[i]))
        };
        if blocked(&reach.any) {
            return None;
        }
        // A goal outside the safe flood has no all-safe path, so both branches below end in
        // None: skip the unfiltered search too.
        if blocked(&reach.safe) {
            debug_assert!(
                find_path(map, u.pos, to, |h| cost(h).filter(|_| safe(h))).is_none(),
                "safe flood excludes a goal with a safe path"
            );
            return None;
        }
        let safe_cost = |h| cost(h).filter(|_| safe(h));
        let safe_step = || match find_path(map, u.pos, to, safe_cost) {
            Some(path) => path.first().copied(),
            None => {
                reach.safe.get_or_init(|| flood(map, u.pos, safe_cost));
                None
            }
        };
        let danger_nearby = u
            .pos
            .within(u.max_moves)
            .into_iter()
            .any(|h| self.view().index(h).is_some() && !safe(h));
        // With danger nearby the answer is the safe route's first step whatever the
        // unfiltered route is (no unfiltered route means no safe one either).
        if danger_nearby {
            return safe_step();
        }
        let Some(path) = find_path(map, u.pos, to, cost) else {
            reach.any.get_or_init(|| flood(map, u.pos, cost));
            return None;
        };
        if path.iter().all(|h| safe(*h)) {
            Some(to)
        } else {
            safe_step()
        }
    }

    /// Whether standing on `h` puts a Spartan Hoplite next to another one (the phalanx,
    /// `data::HOPLITE_PHALANX_STRENGTH`): a neighbour that holds another of our Hoplites now or
    /// is in `reserved` (tiles taken this turn). Gene `phalanx_formation`; false for any other unit.
    fn phalanx_spot_among(&self, u: &UnitView, h: Hex, reserved: &crate::analysis::HexSet) -> bool {
        self.decisions.phalanx_formation != 0
            && legatum_core::data::is_hoplite_rules(
                self.view().rule_experiments.faction_rules_off,
                self.faction,
                u.kind,
            )
            && h.neighbors().into_iter().any(|n| {
                n != u.pos
                    && (reserved.contains(&n)
                        || self
                            .ctx
                            .own_units
                            .iter()
                            .any(|o| o.id != u.id && o.pos == n && o.kind == u.kind))
            })
    }

    pub(crate) fn phalanx_spot(&self, u: &UnitView, h: Hex) -> bool {
        self.phalanx_spot_among(u, h, &self.mil_dest)
    }

    pub(crate) fn campaign_staging_spot(
        &self,
        u: &UnitView,
        target: Hex,
        radius: i32,
    ) -> Option<Hex> {
        let mut spots: Vec<_> = target
            .ring(radius)
            .into_iter()
            .filter(|h| {
                self.free_tile_for(u, *h)
                    && self.safe_campaign_tile(*h)
                    && (crate::composition::charges(u.kind)
                        || self.ranged_unit_clear(u, *h, target))
            })
            .collect();
        spots.sort_by_key(|h| {
            let terrain = self.view().tile(*h).and_then(|t| t.terrain);
            (
                std::cmp::Reverse(
                    crate::composition::shooter_only(u.kind, u.ranged)
                        && terrain == Some(Terrain::Hills),
                ),
                std::cmp::Reverse(terrain == Some(Terrain::Forest)),
                h.distance(u.pos),
                // A tie-break only: formation never costs a unit distance, cover or safety.
                std::cmp::Reverse(self.phalanx_spot(u, *h)),
                *h,
            )
        });
        spots.into_iter().find_map(|h| self.campaign_approach(u, h))
    }

    /// Extend v1's weakest-city campaign: count only reachable troops, and in the late game
    /// prefer a reachable VP leader or the owner threatening home. Safety ratios stay intact.
    pub(crate) fn campaign_target(&self, army: &[&UnitView]) -> Option<Hex> {
        self.campaign_target_for(army, self.opening.and(self.strike_target), false)
    }

    /// Hard (gene `contain_campaign`): the city a containment war marches on. Playtest 8.4:
    /// two AIs at war with the human leader under Contain made 0 campaigns and 4 attacks in 94
    /// turns, all on a Scout, while their stacks sat at home. The pick is the target's city
    /// with the most VP at stake (titles its capture takes, a held goal, a hoard) for the
    /// least defence and distance, within a wider range than an ordinary campaign. The march
    /// stages outside the city; the assault itself keeps every forecast gate.
    pub(crate) fn contain_target(&self, army: &[&UnitView]) -> Option<Hex> {
        let against = self.contain_war()?;
        self.campaign_target_for(army, Some(against), true)
            .or_else(|| self.open_front(army, against))
    }

    /// Gene `coalition_front`: a containment war with no city in marching range still opens a
    /// front. The army stages on the target's known city nearest to it, so the target has to
    /// split its defence; the assault itself keeps every forecast gate (user, 2026-10-17: "at a
    /// minimum, if part of a coalition, open up another front").
    fn open_front(&self, army: &[&UnitView], against: TeamId) -> Option<Hex> {
        if self.decisions.coalition_front == 0 || self.takes_no_cities() {
            return None;
        }
        let v = self.view();
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| v.team_of(c.owner) == against)
            .filter_map(|c| {
                let reach: Vec<_> = army
                    .iter()
                    .filter(|u| self.ctx.reachable(u.pos, c.pos))
                    .collect();
                (reach.len() >= 2).then(|| {
                    let d = reach
                        .iter()
                        .map(|u| u.pos.distance(c.pos))
                        .min()
                        .unwrap_or(99);
                    (d, c.walls, c.pos)
                })
            })
            .min()
            .map(|(_, _, pos)| pos)
    }

    pub(crate) fn available_campaign(&mut self, against: TeamId) -> Option<Hex> {
        let army: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .collect();
        self.defend_cities(&army);
        self.garrison_cities(&army);
        let free: Vec<_> = self
            .free(&army)
            .into_iter()
            .filter(|u| u.hp >= self.march_min_hp())
            .collect();
        self.campaign_target_for(&free, Some(against), false)
    }

    /// `contain`: a containment march (`contain_target`), which stages before starting a
    /// siege, reaches farther, ignores the early Sparta restraint and ranks cities by
    /// the VP their capture takes from the target. Sparta keeps only funded forecasts.
    fn campaign_target_for(
        &self,
        army: &[&UnitView],
        against: Option<TeamId>,
        contain: bool,
    ) -> Option<Hex> {
        if self.takes_no_cities() || (self.faction == Faction::Germany && !contain) {
            return None;
        }
        if self.rushing()
            && against == self.strike_target
            && let Some(city) = self.opening_strike_city()
            && self.assessed_campaign_allowed(city)
            && self.attack_view().at_war(self.view().me, city.owner)
            && army
                .iter()
                .filter(|u| self.ctx.reachable(u.pos, city.pos))
                .count()
                >= 2
            && army
                .iter()
                .any(|u| crate::composition::charges(u.kind) && self.ctx.reachable(u.pos, city.pos))
        {
            // The opening already committed to this war. Keep bringing reinforcements
            // to the front; tactical forecasts still decide whether to enter combat.
            return Some(city.pos);
        }
        let pressure = self.conquest_pressure();
        let hard = self.difficulty == legatum_core::AiDifficulty::Hard;
        if (self.view().diplomacy.legacy
            && !self.rushing()
            && !contain
            && self.knobs.aggression < CAMPAIGN_MIN_AGGRESSION
            && pressure < if hard { 20 } else { 30 })
            || self.ctx.own_cities.is_empty()
        {
            return None;
        }
        let threshold = self.attack_threshold() * self.decisions.war_commit_ratio / 100;
        let cities: Vec<Hex> = self.ctx.enemy_cities.iter().map(|c| c.pos).collect();
        let leader = self
            .view()
            .standings
            .iter()
            .filter(|s| s.team != self.view().team && s.target > 0)
            .map(|s| s.vp * 100 / s.target)
            .max()
            .unwrap_or(0);
        let hollow_leader = self.hollow_leader();
        self.targets(true)
            .into_iter()
            .filter(|t| cities.contains(&t.pos))
            .filter_map(|t| {
                let city = self.ctx.enemy_cities.iter().find(|c| c.pos == t.pos)?;
                if self.strike_city.is_some_and(|id| city.id != id) {
                    return None;
                }
                if against.is_some_and(|team| self.view().team_of(city.owner) != team) {
                    return None;
                }
                // Containment uses the same funded siege forecast as a campaign plan.
                // A high-VP but untakeable walled city cannot pull the army into a war
                // that the campaign layer will abandon on the next turn.
                if contain
                    && self.decisions.contain_march_outlook > 0
                    && self.decisions.contain_commit > 0
                    && self.campaign_outlook(city).is_none()
                {
                    return None;
                }
                if self.decisions.early_walled_skip > 0
                    && !contain
                    && self.view().turn <= legatum_core::data::FIRST_THIRD_TURNS
                    && city.walls
                {
                    return None;
                }
                let reachable: Vec<_> = army
                    .iter()
                    .filter(|u| self.ctx.reachable(u.pos, t.pos))
                    .collect();
                // A containment march may be all shooters (a Germany line of Archers): it
                // stages and fires; capture still needs melee through the assault forecast.
                if reachable.len() < 2
                    || (!contain
                        && !reachable
                            .iter()
                            .any(|u| crate::composition::charges(u.kind)))
                {
                    return None;
                }
                let force: i32 = reachable
                    .iter()
                    .map(|u| offense_power(self.view(), u))
                    .sum();
                // A containment march is measured from the army that marches, too.
                let d = self
                    .ctx
                    .own_cities
                    .iter()
                    .filter(|c| self.ctx.reachable(c.pos, t.pos))
                    .map(|c| c.pos.distance(t.pos))
                    .chain(
                        reachable
                            .iter()
                            .filter(|_| contain)
                            .map(|u| u.pos.distance(t.pos)),
                    )
                    .min()?;
                let slow_core = self.view().turn <= 30
                    && self
                        .view()
                        .players
                        .iter()
                        .any(|p| p.id == city.owner && p.faction == Faction::Germany)
                    && d <= 6;
                let winnable =
                    force * 100 >= (threshold - if slow_core { 10 } else { 0 }) * t.their.max(1);
                // Hard accepts a slightly riskier siege of the public VP leader's city:
                // stopping a runaway empire is worth more than an even trade elsewhere.
                // A team counting down to victory: its titled cities first (`hold.rs`).
                let denial = self.hold_denial_city(city);
                let strain = crate::objective_pursuit::strain_value(
                    self.view(),
                    city.owner,
                    &self.decisions,
                );
                // Gene `hollow_read`: the rich, empty empire near the top of the race is the
                // best target on the map, VP tie or not (8.9.15 Swiss review rec 2).
                let hollow = hollow_leader.filter(|t| *t == self.view().team_of(city.owner));
                let contained = denial > 0
                    || strain > 0
                    || hollow.is_some()
                    || self.top_threat_team() == Some(self.view().team_of(city.owner));
                let winnable = winnable
                    || (self.sparta_walled_prize(city) && self.campaign_outlook(city).is_some())
                    || (contained && force * 100 >= (threshold - 20) * t.their.max(1))
                    || (hard
                        && leader >= self.decisions.leader_siege_percent as u32
                        && self.view().standings.iter().any(|s| {
                            s.target > 0
                                && s.vp * 100 / s.target == leader
                                && self
                                    .view()
                                    .players
                                    .iter()
                                    .any(|p| p.id == city.owner && p.team == s.team)
                        })
                        && force * 100 >= (threshold - 20) * t.their.max(1));
                let team = self
                    .view()
                    .players
                    .iter()
                    .find(|p| p.id == city.owner)?
                    .team;
                let leading = contained
                    || (leader > 0
                        && self.view().standings.iter().any(|s| {
                            s.team == team && s.target > 0 && s.vp * 100 / s.target == leader
                        }));
                let threatens_home = self.ctx.enemy_units.iter().any(|e| {
                    e.owner == city.owner
                        && e.military
                        && self
                            .ctx
                            .own_cities
                            .iter()
                            .any(|c| c.pos.distance(e.pos) <= DEFEND_RADIUS)
                });
                let priority = 100
                    + self.assessed_target_value(city)
                    + if slow_core { 35 } else { 0 }
                    + pressure
                        * (i32::from(leading) * if hard { 4 } else { 2 }
                            + i32::from(threatens_home))
                    + 400 * self.conquest_victory_value(city)
                    + 4 * denial
                    + strain
                    + city.capture_bounty.unwrap_or(0) / 4
                    + hollow.map_or(0, |t| {
                        4 * crate::threat::hollow_bonus_from(self.view(), t, self.hollow_floor(t))
                    })
                    + self.hoard_denial_value(city)
                    + self.greece_captive_tech_value(city)
                    + 100 * (self.capital_capture_vp(city) + self.capital_race_vp(city))
                    + self.swarm_weakness_value(city)
                    + self.flower_wars_campaign_value(city)
                    + 15 * self
                        .view()
                        .cities
                        .iter()
                        .filter(|other| {
                            other.owner == city.owner
                                && other.id != city.id
                                && other.pos.distance(city.pos) <= 5
                        })
                        .count() as i32;
                // A containment march: the VP the capture takes from the target, before a hold
                // starts too (the Heir it shelters).
                let priority = priority
                    + if contain {
                        self.heir_in_city_value(city)
                            + if self.decisions.vp_denial > 0 {
                                (self.hall_in_city_value(city) + self.forge_denial_value(city))
                                    * self.decisions.vp_denial
                                    / 100
                            } else {
                                0
                            }
                            + self.rival_objective_denial(city)
                            + self.coalition_city_push(city)
                    } else {
                        0
                    };
                let priority = priority
                    + self.decisions.contain_stack_weight
                        * self
                            .ctx
                            .own_units
                            .iter()
                            .filter(|u| u.military && u.pos.distance(city.pos) <= 2)
                            .count() as i32;
                // Walled cities fire back every turn of the siege: prefer unwalled ones.
                let fire = if legatum_core::city_fires(city) { 3 } else { 2 };
                let range = self.decisions.campaign_range
                    + pressure / 10
                    + if contain {
                        self.decisions.contain_extra_range
                    } else {
                        0
                    };
                (d <= range && (winnable || contain)).then_some((
                    t.their.max(1) * (8 + d) * 50 * fire / priority.max(25),
                    t.pos,
                ))
            })
            .min()
            .map(|(_, h)| h)
    }

    /// With enough aggression and no city left undefended, healthy free units gather on
    /// distinct tiles just outside strike range of the weakest nearby enemy city they could
    /// take together; `attack_targets` storms it once enough of them have arrived.
    fn campaign(&mut self, army: &[&UnitView]) {
        if self.takes_no_cities() {
            return;
        }
        let undefended = self.ctx.own_cities.iter().enumerate().any(|(i, c)| {
            if self.rush_ignores_home_barbarians(i) {
                return false;
            }
            // Defense has already reserved units. A defended front must not veto every
            // other campaign; count only assigned nearby troops, never the free force
            // about to march away.
            let reserved: i32 = army
                .iter()
                .filter(|u| self.has_acted(u.id) && u.pos.distance(c.pos) <= 2)
                .map(|u| defense_power(self.view(), u).max(attack_power(self.view(), u)))
                .sum();
            self.ctx.threat[i] > city_power(self.view(), c) / 2 + reserved
        });
        if undefended {
            return;
        }
        let free: Vec<&UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| u.hp >= self.march_min_hp() && !self.last_garrison_under_attack(u))
            .collect();
        // Units holding this turn only to break a bounce still belong to the group.
        let yielding: Vec<&UnitView> = army
            .iter()
            .copied()
            .filter(|u| {
                self.has_acted(u.id)
                    && Self::stuck(u)
                    && u.hp >= self.march_min_hp()
                    && !self.last_garrison_under_attack(u)
            })
            .collect();
        let planned_march = self.stages_campaigns()
            && matches!(
                self.routed_plan().map(|p| &p.kind),
                Some(crate::plan::PlanKind::Campaign { .. })
            );
        if free.is_empty() || (!self.rushing() && !planned_march && free.len() + yielding.len() < 2)
        {
            return;
        }
        let force: Vec<_> = if self.rushing() {
            army.iter()
                .copied()
                .filter(|u| u.hp >= self.march_min_hp() && !self.last_garrison_under_attack(u))
                .collect()
        } else {
            free.iter().chain(&yielding).copied().collect()
        };
        // Neutral cities are absent from enemy_cities. Approach the known target at
        // peace; the opening declares only after the force has assembled here.
        let planned_city = self
            .stages_campaigns()
            .then(|| self.routed_plan())
            .flatten()
            .and_then(|p| match &p.kind {
                crate::plan::PlanKind::Campaign { target, .. } => {
                    self.view().cities.iter().find(|city| city.id == *target)
                }
                _ => None,
            });
        let muster = planned_city
            .filter(|c| !self.view().at_war(self.view().me, c.owner))
            .map(|c| c.pos)
            .or_else(|| {
                self.conquest_city()
                    .filter(|c| !self.view().at_war(self.view().me, c.owner))
                    .map(|c| c.pos)
            })
            .or_else(|| {
                self.rushing()
                    .then(|| self.opening_strike_city())
                    .flatten()
                    .filter(|c| !self.view().at_war(self.view().me, c.owner))
                    .map(|c| c.pos)
            });
        let committed = self.campaigns.active.as_ref().and_then(|c| {
            self.view()
                .cities
                .iter()
                .find(|city| city.id == c.target)
                .map(|city| city.pos)
        });
        // A searched Campaign can be chosen while its army is still marching. Stage on
        // that city from the first turn; the active siege begins only after the normal
        // local forecast admits it. Otherwise a distant plan can declare war while the
        // routine conquest nomination sends no one toward its named objective.
        let planned = planned_city.map(|city| city.pos);
        // A containment war marches on its target before any neutral muster.
        let contain = committed
            .is_none()
            .then(|| self.contain_target(&force))
            .flatten();
        let muster = if contain.is_some() { None } else { muster };
        let Some(target) = committed
            .or(planned)
            .or(contain)
            .or(muster)
            .or_else(|| {
                self.ctx
                    .enemy_cities
                    .iter()
                    .filter(|c| self.sparta_open_followup(c) || self.sparta_chain_target(c))
                    .min_by_key(|c| (c.pos.distance(self.ctx.own_cities[0].pos), c.id))
                    .map(|c| c.pos)
            })
            .or_else(|| self.conquest_city().map(|c| c.pos))
            .or_else(|| self.campaign_target(&force))
        else {
            return;
        };
        // Infantry sees two tiles, one fewer than the safe muster ring. Once the
        // army is assembled, buy one sighting with a survivable city shot so the
        // next turn can use an actual capture forecast rather than wait forever.
        if muster.is_none()
            && !self.view().tile(target).is_some_and(|t| t.visible)
            && force
                .iter()
                .filter(|u| u.pos.distance(target) <= STRIKE_RADIUS)
                .count()
                >= 2
            && let Some((u, to)) = free
                .iter()
                .filter(|u| crate::composition::charges(u.kind))
                .find_map(|&u| {
                    target
                        .ring(2)
                        .into_iter()
                        .filter(|h| self.free_tile_for(u, *h) && self.reaches_this_turn(u, *h))
                        .filter(|h| {
                            self.enemy_exposure_key(*h).0 == 0 && self.outside_remembered_reach(*h)
                        })
                        .filter(|h| {
                            u.hp - self.city_damage_now(u) - self.max_city_damage_at(u, *h)
                                >= self.march_min_hp()
                        })
                        .min_by_key(|h| (self.city_damage_at(u, *h), h.distance(u.pos), *h))
                        .map(|to| (u, to))
                })
        {
            self.go_to(u, to);
        }
        // Sight is only two tiles. Staging three away from a remembered city would never
        // refresh it; approach into sight, then use the ordinary visible-target checks.
        let staging_radius =
            if muster.is_some() || self.view().tile(target).is_some_and(|t| t.visible) {
                STRIKE_RADIUS
            } else {
                legatum_core::data::UNIT_SIGHT
            };
        for u in free {
            if self.has_acted(u.id) {
                continue;
            }
            if self.conquest_city().is_some()
                && !planned_march
                && crate::composition::charges(u.kind)
                && self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                && !self.finishing(u.id)
            {
                continue;
            }
            if muster.is_some() && (planned_march || self.conquest_city().is_some()) {
                let spot = (3..=6).find_map(|radius| self.campaign_staging_spot(u, target, radius));
                if let Some(to) = spot {
                    self.go_to(u, to);
                }
                continue;
            }
            if !self.ctx.reachable(u.pos, target)
                && !(muster.is_some() && self.opening_reachable(u.pos, target))
            {
                continue;
            }
            if u.pos.distance(target) <= staging_radius && self.safe_campaign_tile(u.pos) {
                if self.free_tile_for(u, u.pos) {
                    self.hold_or_upgrade(u);
                }
                continue;
            }
            if self.rushing()
                && !self.view().tile(target).is_some_and(|t| t.visible)
                && let Some(to) = target
                    .ring(staging_radius)
                    .into_iter()
                    .filter(|h| self.free_tile_for(u, *h) && self.exposure_at(*h) == 0)
                    .min_by_key(|h| (h.distance(u.pos), *h))
            {
                // Reconnaissance may accept a remembered threat with a committed army.
                // It never attacks an unseen unit or assumes a remembered city is empty.
                if muster.is_some()
                    || crate::composition::charges(u.kind)
                    || self.rush_melee_screen_at(target)
                {
                    self.go_to(u, to);
                } else {
                    self.hold_or_upgrade(u);
                }
                continue;
            }
            // If sight range is covered by a recent defender, stage outside its reach.
            // Fresh reconnaissance, expiry or a different target can reopen the approach.
            let spot = (staging_radius..=staging_radius + STRIKE_RADIUS)
                .find_map(|radius| self.campaign_staging_spot(u, target, radius));
            match spot {
                Some(to) => self.go_to(u, to),
                None => self.hold_or_upgrade(u),
            }
        }
    }

    /// A campaign that failed an explicit gate exits the front as a group. It does not
    /// leave its old siege units idle to be reassigned by a later routine pass.
    fn withdraw_failed_campaign(&mut self, army: &[&UnitView]) {
        if self.decisions.finish_fallen > 0 && self.hard_defense() {
            self.pass("finish_fallen_cities");
            self.finish_fallen_cities(army);
            self.pass("withdraw_failed_campaign");
        }
        // Gene `war_siege`: while the plan still fights the enemy, the army stays at the front
        // to siege it rather than walking home for a turn and marching back.
        if self.campaign_withdraw.is_empty() || self.siege_keeps_front() {
            return;
        }
        let homes: Vec<Hex> = self.ctx.own_cities.iter().map(|c| c.pos).collect();
        let withdrawing = self.campaign_withdraw.clone();
        for u in army.iter().copied().filter(|u| withdrawing.contains(&u.id)) {
            if self.has_acted(u.id) {
                continue;
            }
            let Some(home) = homes.iter().min_by_key(|h| (h.distance(u.pos), *h)) else {
                continue;
            };
            if u.pos.distance(*home) <= 1 {
                self.hold_or_upgrade(u);
            } else if let Some(to) = self.campaign_approach(u, *home) {
                self.go_to(u, to);
            } else {
                self.hold_or_upgrade(u);
            }
        }
    }

    /// Gene `finish_fallen`: a broken enemy city is the cheapest capture there is. Before any
    /// withdrawal takes the units near it, the city-assault coordinator prices taking it with
    /// every free unit in reach (its multi-turn forecast of the volley on the exposed garrison
    /// and the capturer's charge) and commits them when the capture succeeds.
    fn finish_fallen_cities(&mut self, army: &[&UnitView]) {
        let me = self.view().me;
        let mut fallen: Vec<(i32, Hex)> = self
            .ctx
            .enemy_cities
            .iter()
            .filter(|c| {
                c.visible
                    && c.hp * 100 <= self.decisions.finish_fallen * c.max_hp
                    && self.attack_view().at_war(me, c.owner)
                    && army.iter().any(|u| {
                        !self.has_acted(u.id) && u.pos.distance(c.pos) <= STRIKE_RADIUS + 1
                    })
            })
            .map(|c| (c.hp, c.pos))
            .collect();
        fallen.sort();
        for (_, pos) in fallen {
            self.coordinate_city_assaults_on(army, Some(pos), true);
        }
    }

    /// Everything left rests near home (healing and fortifying).
    fn rest(&mut self, army: &[&UnitView]) {
        for u in self.free(army) {
            let Some(city) = self.ctx.nearest_own_city(u.pos) else {
                self.hold_or_upgrade(u);
                continue;
            };
            // Keep room to build and buy: a city whose tile and ring are all occupied can't
            // place a new unit (core `cities::spawn_tile`). Hard moves a resting unit off the
            // ring when fewer than two spawn tiles are left (goals-3 stalls: a rich 9-VP
            // empire couldn't buy the Army title for 600 turns because its idle army filled
            // every city's ring).
            if self.hard_defense()
                && self.decisions.endgame_push > 0
                && u.pos.distance(city.pos) == 1
            {
                let free = std::iter::once(city.pos)
                    .chain(city.pos.neighbors())
                    .filter(|h| {
                        self.view().tile(*h).is_some_and(|t| {
                            t.terrain.and_then(|terrain| terrain.move_cost()).is_some()
                        }) && self.free_tile_for_any(*h)
                    })
                    .count();
                if free < 2
                    && let Some(to) = city
                        .pos
                        .within(3)
                        .into_iter()
                        .filter(|h| {
                            h.distance(city.pos) >= 2
                                && self.free_tile_for(u, *h)
                                && self.can_defend_at(u, *h)
                                && self.reaches_this_turn(u, *h)
                        })
                        .min_by_key(|h| (h.distance(city.pos), h.distance(u.pos), *h))
                {
                    self.go_to(u, to);
                    continue;
                }
            }
            if u.pos.distance(city.pos) <= HOME_RADIUS && self.free_tile_for(u, u.pos) {
                self.hold_or_upgrade(u);
                continue;
            }
            let spot = city
                .pos
                .within(1)
                .into_iter()
                .filter(|h| self.free_tile_for(u, *h) && self.can_defend_at(u, *h))
                .min_by_key(|h| (h.distance(u.pos), *h));
            match spot {
                Some(h) => self.go_to(u, h),
                None => self.hold_or_upgrade(u),
            }
        }
    }
}

#[cfg(test)]
#[path = "campaign_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "defense_tests.rs"]
mod defense_tests;

#[cfg(test)]
#[path = "escort_tests.rs"]
mod escort_tests;

#[cfg(test)]
mod military_scout_tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off, planner};
    use legatum_core::AiDifficulty;

    #[test]
    fn idle_soldier_collects_safe_ruin_only_with_gene() {
        let ruin = off(6, 5);
        let view = ViewBuilder::new(12, 12)
            .city(1, ME, off(3, 5), 3)
            .ruin(ruin)
            .unit(1, ME, UnitKind::Spearman, off(5, 5))
            .build();
        let mut p = planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        let army = vec![&view.units[0]];
        p.military_scout(&army);
        assert!(p.orders.is_empty());
        p.decisions.military_scout = 100;
        p.military_scout(&army);
        assert!(p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } if *to == ruin)));
    }

    #[test]
    fn military_scout_leaves_committed_unit_alone() {
        let view = ViewBuilder::new(12, 12)
            .city(1, ME, off(3, 5), 3)
            .ruin(off(6, 5))
            .unit(1, ME, UnitKind::Spearman, off(5, 5))
            .build();
        let mut p = planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.military_scout = 100;
        p.unit_order(
            view.units[0].id,
            Order::SkipUnit {
                unit: view.units[0].id,
            },
        );
        p.military_scout(&[&view.units[0]]);
        assert_eq!(
            p.orders,
            vec![Order::SkipUnit {
                unit: view.units[0].id
            }]
        );
    }

    #[test]
    fn idle_soldier_moves_toward_nearby_unknown_land() {
        let view = ViewBuilder::new(12, 12)
            .city(1, ME, off(3, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(6, 5))
            .unexplored_from_col(8)
            .build();
        let mut p = planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.military_scout = 200;
        p.military_scout(&[&view.units[0]]);
        assert!(
            p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { .. } | Order::MoveUnitPath { .. }))
        );
    }
}

#[cfg(test)]
#[path = "city_fire_tests.rs"]
mod city_fire_tests;

#[path = "city_assault.rs"]
mod city_assault;

#[cfg(test)]
mod mongol_horde_tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder};

    #[test]
    fn funded_horseman_upgrade_keeps_the_veteran_even_below_the_gold_reserve() {
        let mut view = ViewBuilder::new(20, 16)
            .faction(Faction::Mongols)
            .city(1, ME, Hex::new(5, 5), 3)
            .unit(1, ME, UnitKind::Horseman, Hex::new(6, 5))
            .gold(108)
            .build();
        view.units[0].upgrade_cost = Some(108);
        view.units[0].upgrade_need = Some(legatum_core::resources::Resource::Iron);
        let mut p = crate::fixture::planner(&view, 7);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.decisions.mongol_horde = 2;
        p.hold_or_upgrade(&view.units[0]);
        assert!(
            p.orders.iter().any(|o| matches!(
                o,
                Order::UpgradeUnit {
                    unit: legatum_core::UnitId(1)
                }
            )),
            "{:?}",
            p.orders
        );
    }
}

#[path = "siege.rs"]
mod siege;

#[path = "breach.rs"]
mod breach;

#[path = "combat_probe.rs"]
mod combat_probe;
pub use combat_probe::AssaultProbe;

/// Tiles one unit can reach this planning step, filled after a search for it fails: under
/// the step rule alone, and with the safety filter too.
#[derive(Default)]
pub(crate) struct ApproachReach {
    any: std::cell::OnceCell<Vec<bool>>,
    safe: std::cell::OnceCell<Vec<bool>>,
}

/// Every tile reachable from `from` stepping only onto tiles `cost` allows (the start tile
/// itself needs no cost): exactly the goals `find_path` can reach with the same rule.
fn flood(
    map: &legatum_core::state::Map,
    from: Hex,
    cost: impl Fn(Hex) -> Option<i32>,
) -> Vec<bool> {
    let mut seen = vec![false; map.len()];
    let Some(start) = map.index(from) else {
        return seen;
    };
    seen[start] = true;
    let mut stack = vec![from];
    while let Some(cur) = stack.pop() {
        for nb in cur.neighbors() {
            let Some(i) = map.index(nb) else { continue };
            if !seen[i] && cost(nb).is_some() {
                seen[i] = true;
                stack.push(nb);
            }
        }
    }
    seen
}
