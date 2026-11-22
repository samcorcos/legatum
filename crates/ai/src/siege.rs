//! Gene `war_siege`: the war our plan fights always has an active objective (user, 2026-10-18,
//! after EU4: "factions don't just declare war and chill, they pretty much always attack with
//! whatever they have"). A containment war used to stage on a ring no enemy could reach and
//! wait there for a capture forecast that never came against a walled, garrisoned city.
//!
//! The objective is chosen by value from what we see: the enemy city whose local defence our
//! nearby force best outweighs, sieged by shooters in range (a city hit this turn does not heal,
//! so steady fire wears it down until the city-assault coordinator can take it), or failing
//! that, the enemy's field units near our army. The test is a strength ratio, never a fixed
//! "always attack": a force that doesn't outweigh the defence keeps the old safe staging.

use super::*;
use legatum_core::{TeamId, UnitId};

/// Gene `siege_parts` bits: the early pass on the campaign's city, the late pass for every
/// plan war, the field-army raid, and keeping a failed campaign's army at the front.
const SIEGE_EARLY: i32 = 1;
const SIEGE_LATE: i32 = 2;
const SIEGE_RAID: i32 = 4;
const SIEGE_HOLD_FRONT: i32 = 8;

impl Planner<'_> {
    /// The enemy our plan's war fights, for a war under way: Contain or Campaign only. Gene
    /// `siege_scope` 1 narrows it to the wars on the leader (the strict VP leader, a team
    /// counting down, our diplomatic target), where every coalition member's front counts.
    fn siege_enemy(&self) -> Option<TeamId> {
        // Gene `siege_exempt` 1: the former Germany, Sparta and Celtic exclusion (they storm,
        // run no field campaigns or raze rather than besiege), kept as a comparison arm.
        if self.decisions.siege_exempt > 0 {
            return None;
        }
        let v = self.view();
        let against = match self.routed_plan().map(|p| &p.kind) {
            Some(crate::plan::PlanKind::Contain { against })
            | Some(crate::plan::PlanKind::Campaign { against, .. })
                if v.teams_at_war(v.team, *against) =>
            {
                *against
            }
            _ => return None,
        };
        (self.decisions.siege_scope == 0 || self.leader_war(against)).then_some(against)
    }

    /// Whether a failed campaign's army should stay at the front (`war_siege`): our plan
    /// still fights the city's owner.
    pub(super) fn siege_keeps_front(&self) -> bool {
        self.decisions.war_siege > 0
            && self.siege_part(SIEGE_HOLD_FRONT)
            && self.hard_defense()
            && self.siege_enemy().is_some()
    }

    /// Gene `siege_parts`: which parts of the siege objective run (a bit mask).
    fn siege_part(&self, bit: i32) -> bool {
        self.decisions.siege_parts & bit != 0
    }

    /// The percent of the local defence our force must reach: the gene's value, two thirds of
    /// it while another coalition member's army stands at the enemy's cities (a second front
    /// splits their defence, so ours may be the weaker one).
    fn siege_ratio(&self, against: TeamId) -> i32 {
        let base = self.decisions.war_siege;
        if self.decisions.coalition_front > 0 && self.coalition_fronts(against) > 0 {
            base * 2 / 3
        } else {
            base
        }
    }

    /// Gene `campaign_persist`: our committed units still at `city` (within three hexes and
    /// healthy enough to march), as a percent of the city's local defence, when at least two
    /// stand there and one of them can take the city. None when the siege has broken up.
    pub(crate) fn siege_local_margin(&self, city: &CityView, ids: &[UnitId]) -> Option<i32> {
        let near: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| ids.contains(&u.id) && u.pos.distance(city.pos) <= 3 && u.hp >= 30)
            .collect();
        if near.len() < 2 || !near.iter().any(|u| crate::composition::charges(u.kind)) {
            return None;
        }
        // The defence directly, not through `targets`: its Mongol gate drops a walled city
        // whose capture forecast failed, which is exactly the siege this keeps going.
        let their = self.city_defence_weight(city);
        Some(self.attack_force(&near) * 100 / their.max(1))
    }

    /// `early`: the pass right after the city-assault coordinator, before the retreats from
    /// city fire and free damage. It sieges only the running campaign's city, whose capture
    /// the plan has priced, so a siege there holds its ground under the city's strike. The
    /// later pass, after home defence, sieges for every war the plan fights (or raids) with
    /// the units those retreats left in place. Sparta runs it after its campaign pass so
    /// the melee assault wing is not consumed by siege screening first.
    pub(super) fn siege_war(&mut self, army: &[&UnitView], early: bool) {
        if self.decisions.war_siege == 0 || !self.hard_defense() {
            return;
        }
        let Some(against) = self.siege_enemy() else {
            return;
        };
        let campaign_city = self
            .campaigns
            .active
            .as_ref()
            .and_then(|c| {
                self.ctx
                    .enemy_cities
                    .iter()
                    .find(|city| city.id == c.target)
            })
            .map(|c| c.pos);
        if early && campaign_city.is_none() {
            return;
        }
        if !self.siege_part(if early { SIEGE_EARLY } else { SIEGE_LATE }) {
            return;
        }
        // A home under real threat (more than half its own strength) keeps the units standing
        // at it; the defence passes after this one call in the rest they need.
        let threatened: Vec<Hex> = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(i, c)| self.ctx.threat[*i] > city_power(self.view(), c) / 2)
            .map(|(_, c)| c.pos)
            .collect();
        let free: Vec<&UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| u.hp >= self.march_min_hp() && !self.last_garrison_under_attack(u))
            .filter(|u| {
                threatened
                    .iter()
                    .all(|c| c.distance(u.pos) > self.decisions.siege_home_radius)
            })
            .collect();
        if free.len() < 2 {
            return;
        }
        let need = self.siege_ratio(against);
        let v = self.view();
        // Known cities too: the safe staging ring is often out of sight of the city, and a
        // remembered city's defence carries the cautious fog allowance (`targets`).
        let targets = self.targets(true);
        // The city we can siege at the best margin, then the nearest.
        let mut best: Option<(i32, i32, Hex)> = None;
        for c in self
            .ctx
            .enemy_cities
            .iter()
            .filter(|c| c.hp > 0 && v.team_of(c.owner) == against)
            .filter(|c| !early || Some(c.pos) == campaign_city)
        {
            let near: Vec<&UnitView> = free
                .iter()
                .copied()
                .filter(|u| {
                    u.pos.distance(c.pos) <= self.decisions.siege_reach
                        && self.ctx.reachable(u.pos, c.pos)
                })
                .collect();
            // Gene `siege_shooters`: at least this many shooters to make a siege of it.
            let shooters = near.iter().filter(|u| u.ranged > 0 && u.range > 0).count() as i32;
            if near.len() < 2 || shooters < self.decisions.siege_shooters.max(1) {
                continue;
            }
            // Gene `siege_walled_only`: an open city is stormed (the assault coordinator), not
            // sieged; the combat lab found a storm the better policy in most open-city cases.
            if self.decisions.siege_walled_only > 0 && !c.walls {
                continue;
            }
            // The defence directly: `targets` drops a Mongol-walled city whose capture
            // forecast failed, and a siege is how such a city is taken.
            let their = self.city_defence_weight(c);
            let mine = self.attack_force(&near);
            let margin = mine * 100 / their.max(1);
            if margin < need {
                continue;
            }
            let d = near
                .iter()
                .map(|u| u.pos.distance(c.pos))
                .min()
                .unwrap_or(99);
            if best.is_none_or(|(m, bd, _)| (margin, -d) > (m, -bd)) {
                best = Some((margin, d, c.pos));
            }
        }
        if let Some((_, _, city)) = best {
            self.siege_city(&free, city, early);
            return;
        }
        if !early && self.siege_part(SIEGE_RAID) {
            self.raid_field_army(&free, against, need, &targets);
        }
    }

    /// Shooters take firing positions in range of `city` (line of sight, accepting the city's
    /// strike as long as they survive its strongest roll, and no more enemy power able to reach
    /// the tile than half our force there); melee screens them, beside the city when it has no
    /// Walls to strike with (an adjacent enemy also stops the city healing).
    pub(super) fn siege_city(&mut self, free: &[&UnitView], city: Hex, early: bool) {
        let mut near: Vec<&UnitView> = free
            .iter()
            .copied()
            .filter(|u| {
                u.pos.distance(city) <= self.decisions.siege_reach
                    && self.ctx.reachable(u.pos, city)
            })
            .collect();
        let mine = self.attack_force(&near);
        let walls = self
            .ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos == city && c.walls);
        // Shooters first, so the screen forms around their tiles.
        near.sort_by_key(|u| {
            (
                !(crate::composition::shooter_only(u.kind, u.ranged)
                    || self.mongol_horde()
                        && self.decisions.mongol_horde >= 3
                        && crate::composition::dual(u.kind, u.ranged)),
                u.pos.distance(city),
                u.id,
            )
        });
        let mut shooter_spots: Vec<Hex> = Vec::new();
        let mut gun_spots: Vec<Hex> = Vec::new();
        // Gene `joint_assault`: against a city that strikes (Walls, HP left), plan every tile
        // first and send them only together (`joint_entry`); `planned` holds the tiles taken.
        let firing = self
            .ctx
            .enemy_cities
            .iter()
            .copied()
            .find(|c| c.pos == city && c.walls && c.hp > 0)
            .filter(|_| self.decisions.joint_assault > 0);
        let mut plans: Vec<(&UnitView, Hex)> = Vec::new();
        let mut planned = crate::analysis::HexSet::new(self.view().width, self.view().height);
        for u in near {
            if self.has_acted(u.id) {
                continue;
            }
            let shooter = u.ranged > 0
                && u.range > 0
                && (!crate::composition::charges(u.kind)
                    || self.mongol_horde()
                        && self.decisions.mongol_horde >= 3
                        && walls
                        && crate::composition::dual(u.kind, u.ranged));
            // Gene `siege_melee` 0: the melee stay with the storm and campaign passes (the
            // assault coordinator, Sparta's sorties and expedition), which take the city once
            // the shooters have broken it.
            if !shooter
                && (self.decisions.siege_melee == 0
                    || (early
                        && self.faction == Faction::Sparta
                        && self.decisions.siege_early_shooters > 0))
            {
                continue;
            }
            let rings: Vec<i32> = if shooter {
                (1..=u.range).rev().collect()
            } else if walls {
                vec![2]
            } else {
                vec![1, 2]
            };
            let spot = rings
                .iter()
                .flat_map(|r| city.ring(*r))
                .filter(|h| {
                    self.free_tile_for(u, *h)
                        && !planned.contains(h)
                        && (*h == u.pos || self.ctx.reachable(u.pos, *h))
                        && (!shooter || self.ranged_unit_clear(u, *h, city))
                        // It survives the city's strongest strike (gene `siege_floor` 0), or
                        // keeps that much HP after an average one; one unit takes the strike
                        // each turn and goes to heal (`recover_wounded`) while the rest fire.
                        && if self.decisions.siege_floor == 0 {
                            u.hp - self.city_damage_now(u) - self.max_city_damage_at(u, *h) > 0
                        } else {
                            u.hp - self.city_damage_now(u) - self.city_damage_at(u, *h)
                                >= self.decisions.siege_floor
                        }
                        && self.enemy_exposure_key(*h).0 * 2 <= mine
                        // Gene `joint_assault`: not in another walled city's range on the way.
                        && (firing.is_none() || !self.in_other_city_fire(*h, city))
                })
                .min_by_key(|h| {
                    let terrain = self.view().tile(*h).and_then(|t| t.terrain);
                    (
                        // Gene `joint_assault`: a tile one turn's move away, so the force massed
                        // outside the range steps in on one turn.
                        firing.is_some() && h.distance(u.pos) > u.max_moves.max(1),
                        // Shooters stand as far out as their range allows; the screen stands
                        // next to a shooter.
                        if shooter {
                            u.range - h.distance(city)
                        } else {
                            i32::from(!shooter_spots.iter().any(|s| s.distance(*h) <= 1))
                                + i32::from(
                                    !gun_spots.is_empty()
                                        && !gun_spots.iter().any(|s| s.distance(*h) <= 1),
                                )
                        },
                        self.enemy_exposure_key(*h),
                        self.city_damage_at(u, *h),
                        std::cmp::Reverse(terrain == Some(Terrain::Hills)),
                        std::cmp::Reverse(terrain == Some(Terrain::Forest)),
                        h.distance(u.pos),
                        *h,
                    )
                });
            let Some(to) = spot else { continue };
            if shooter {
                shooter_spots.push(to);
                if matches!(
                    u.kind,
                    UnitKind::Artillery | UnitKind::RocketArtillery | UnitKind::CombatDrone
                ) {
                    gun_spots.push(to);
                }
            }
            // A shooter already in place fires (`fire_from_holds` turns its hold into the best
            // shot, the city when nothing better is in range).
            if firing.is_some() {
                planned.insert(to);
                plans.push((u, to));
            } else {
                self.go_to(u, to);
            }
        }
        if let Some(target) = firing {
            self.joint_entry(target, &plans);
        }
    }

    /// Gene `joint_assault`: the siege tiles of a city that strikes are all within its two-hex
    /// range. The planned units that can be in range this turn (within one move of it) step in
    /// only when the siege forecast of that group, on its tiles, takes the city: units arriving
    /// later would walk in one at a time and each take the city's strike (and its garrison's
    /// shots) alone. Otherwise every planned unit stages just outside the range, where the
    /// next turn's group forms.
    pub(super) fn joint_entry(&mut self, city: &CityView, plans: &[(&UnitView, Hex)]) {
        let entering: Vec<(&UnitView, Hex, i32)> = plans
            .iter()
            .filter(|(u, to)| u.pos == *to || u.pos.distance(city.pos) <= 2 + u.max_moves.max(1))
            .map(|&(u, to)| (u, to, i32::from(u.pos != to)))
            .collect();
        // Value 2: a force that all steps in on this turn goes in whatever the forecast says, as
        // the siege always did (waiting brings nobody else); only a part of it waits.
        let whole = self.decisions.joint_assault >= 2 && entering.len() == plans.len();
        let takes = whole
            || entering
                .iter()
                .any(|(u, _, _)| crate::composition::charges(u.kind))
                && self
                    .forecast_siege_rounds(
                        self.view(),
                        city,
                        &entering,
                        // Our Bombers that can strike it break the city with the force.
                        &self.planes_in_reach(city),
                        (self.decisions.joint_window > 0).then_some(self.decisions.joint_window),
                    )
                    .is_some();
        for &(u, to) in plans {
            if takes {
                self.go_to(u, to);
                continue;
            }
            let d = u.pos.distance(city.pos);
            let staging = (3..=4)
                .find_map(|r| self.campaign_staging_spot(u, city.pos, r))
                .or_else(|| {
                    // No safe staging tile: the nearest free one outside the range will do.
                    (d > 4)
                        .then(|| {
                            (3..=4)
                                .flat_map(|r| city.pos.ring(r))
                                .filter(|h| {
                                    self.free_tile_for(u, *h)
                                        && self.ctx.reachable(u.pos, *h)
                                        && !self.in_other_city_fire(*h, city.pos)
                                })
                                .min_by_key(|h| {
                                    (self.enemy_exposure_key(*h), h.distance(u.pos), *h)
                                })
                        })
                        .flatten()
                });
            match staging {
                Some(h) => self.go_to(u, h),
                None if d > 2 => self.hold_or_upgrade(u),
                None => match self.out_of_city_range(u, Some(city)) {
                    Some(h) => {
                        self.go_to(u, h);
                        self.retreats.insert(u.id);
                    }
                    None => self.hold_or_upgrade(u),
                },
            }
        }
    }

    /// Whether `h` is within the two-hex strike of an enemy city other than `target` that can
    /// fire (Walls and HP left).
    pub(super) fn in_other_city_fire(&self, h: Hex, target: Hex) -> bool {
        self.ctx
            .enemy_cities
            .iter()
            .any(|c| c.pos != target && c.walls && c.hp > 0 && c.pos.distance(h) <= 2)
    }

    /// No city to siege: strike the enemy's field units near our army, on the same strength
    /// test against each target's defence and support.
    fn raid_field_army(
        &mut self,
        free: &[&UnitView],
        against: TeamId,
        need: i32,
        targets: &[Target],
    ) {
        let v = self.view();
        let mut best: Option<(i32, Hex)> = None;
        let mut best_arrivals = Vec::new();
        for t in targets {
            let Some(e) = self
                .ctx
                .enemy_units
                .iter()
                .find(|e| e.pos == t.pos && e.military && v.team_of(e.owner) == against)
            else {
                continue;
            };
            if self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos) {
                continue;
            }
            let near: Vec<&UnitView> = free
                .iter()
                .copied()
                .filter(|u| {
                    u.pos.distance(t.pos) <= self.decisions.raid_reach
                        && self.ctx.reachable(u.pos, t.pos)
                })
                .collect();
            if near.len() < 2 {
                continue;
            }
            let nominal = self.attack_force(&near);
            let arrivals = if self.decisions.raid_arrival_weight > 0 {
                self.raid_arrivals(&near, t.pos)
            } else {
                Vec::new()
            };
            let force = if self.decisions.raid_arrival_weight > 0 {
                if arrivals.len() < 2 {
                    continue;
                }
                let arrived: Vec<_> = arrivals.iter().map(|(u, _)| *u).collect();
                let arriving = self.attack_force(&arrived);
                let surviving = self.assault_force_after_exposure(&arrivals);
                arriving - (arriving - surviving).max(0) * self.decisions.raid_arrival_weight / 100
            } else {
                nominal
            };
            let margin = force * 100 / t.their.max(1);
            if margin >= need && best.is_none_or(|(m, _)| margin > m) {
                best = Some((margin, t.pos));
                best_arrivals = arrivals;
            }
        }
        let Some((_, target)) = best else {
            return;
        };
        if self.decisions.raid_arrival_weight > 0 {
            // Reconciliation cancels the entire arrival group if traffic delays a member.
            self.assault_groups.push(
                best_arrivals
                    .iter()
                    .filter(|(u, to)| u.pos != *to)
                    .map(|(u, _)| u.id)
                    .collect(),
            );
            for (u, to) in best_arrivals {
                self.go_to(u, to);
            }
            return;
        }
        let near: Vec<&UnitView> = free
            .iter()
            .copied()
            .filter(|u| {
                !self.has_acted(u.id)
                    && u.pos.distance(target) <= self.decisions.raid_reach
                    && self.ctx.reachable(u.pos, target)
            })
            .collect();
        for u in near {
            let shooter = u.ranged > 0 && u.range > 0 && !crate::composition::charges(u.kind);
            let ring = if shooter { u.range } else { 1 };
            let spot = target
                .ring(ring)
                .into_iter()
                .filter(|h| {
                    self.free_tile_for(u, *h)
                        && self.ctx.reachable(u.pos, *h)
                        && (!shooter || self.ranged_unit_clear(u, *h, target))
                        && u.hp - self.max_city_damage_at(u, *h) > 0
                })
                .min_by_key(|h| (self.enemy_exposure_key(*h), h.distance(u.pos), *h));
            if let Some(to) = spot {
                self.go_to(u, to);
            }
        }
    }

    /// Count a field raid's actual arrivals, reserving distinct firing/screen tiles. A
    /// nearby unit behind a forest or another unit cannot absorb this turn's enemy fire.
    fn raid_arrivals<'a>(&self, near: &[&'a UnitView], target: Hex) -> Vec<(&'a UnitView, Hex)> {
        let mut arrivals = Vec::new();
        let mut reserved = self.mil_dest.clone();
        let mut sorted = near.to_vec();
        sorted.sort_by_key(|u| (u.pos.distance(target), u.id));
        for u in sorted {
            let shooter = u.ranged > 0 && u.range > 0 && !crate::composition::charges(u.kind);
            let range = if shooter { u.range } else { 1 };
            let spot = (1..=range)
                .rev()
                .flat_map(|r| target.ring(r))
                .filter(|h| {
                    !reserved.contains(h)
                        && self.free_tile_for(u, *h)
                        && self.reaches_this_turn(u, *h)
                        && self
                            .known_path_this_turn(u, *h)
                            .is_some_and(|path| path.iter().all(|step| !reserved.contains(step)))
                        && (!shooter || self.ranged_unit_clear(u, *h, target))
                })
                .min_by_key(|h| (self.enemy_exposure_key(*h), h.distance(u.pos), *h));
            if let Some(to) = spot {
                reserved.insert(to);
                arrivals.push((u, to));
            }
        }
        arrivals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use crate::plan::{Plan, PlanKind};
    use legatum_core::diplomacy::{Relation, RelationState};
    use legatum_core::{AiDifficulty, PlayerView, UnitId};

    const CITY: Hex = Hex::new(14, 8);

    /// Two Archers and a Spearman five or six hexes from a walled enemy city held by one
    /// Spearman, at war under a Contain plan.
    fn front() -> PlayerView {
        let mut view = ViewBuilder::new(30, 16)
            .city(1, ME, Hex::new(3, 8), 3)
            .city(9, ENEMY, CITY, 2)
            .walled(9)
            .unit(1, ME, UnitKind::Archer, Hex::new(9, 8))
            .unit(2, ME, UnitKind::Archer, Hex::new(9, 9))
            .unit(3, ME, UnitKind::Spearman, Hex::new(8, 8))
            .unit(50, ENEMY, UnitKind::Spearman, CITY)
            .build();
        view.diplomacy.legacy = false;
        let enemy = view.team_of(ENEMY);
        view.diplomacy.relations.push(Relation {
            a: view.team,
            b: enemy,
            state: RelationState::War,
            effective_turn: 5,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(1),
            declarers: vec![view.team],
        });
        view
    }

    fn planner(view: &PlayerView, siege: i32) -> Planner<'_> {
        let mut p = Planner::new(
            view,
            Knobs::new(50, 50, 50, 50, 50),
            Faction::Switzerland,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.plan_routing = true;
        p.decisions.war_siege = siege;
        p.campaigns.plan.active = Some(Plan {
            kind: PlanKind::Contain {
                against: view.team_of(ENEMY),
            },
            start_turn: 5,
            review_turn: 15,
            reserve: 0,
            value: 100,
            terms: Vec::new(),
            breach: false,
            misses: 0,
        });
        p
    }

    fn moves(p: &Planner<'_>) -> Vec<(UnitId, Hex)> {
        p.orders
            .iter()
            .filter_map(|o| match *o {
                Order::MoveUnit { unit, to } => Some((unit, to)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn sparta_142_raid_counts_arrivals_before_sending_the_first_phalangite() {
        use crate::Brain;
        let view: PlayerView = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("fixtures/sparta-1025/t142-japan.view.json.gz")[..],
        ))
        .unwrap();
        let memory: crate::OpponentMemory = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("fixtures/sparta-1025/t142-japan.memory.json.gz")[..],
        ))
        .unwrap();
        let seed =
            legatum_core::rng::Rng::derive(7732777812994077, &[legatum_core::rng::purpose::AI, 2])
                .next_u64();
        for weight in [0, 100] {
            let mut brain = crate::ScriptedBrain::with_difficulty(
                Faction::India,
                seed,
                view.me,
                AiDifficulty::Hard,
            );
            brain.restore_opponent_memory(memory.clone());
            // This fixture compares the whole historical plan, before queue revaluation.
            brain.decisions.modernization_value = 0;
            // The recorded plan predates the battle-tactics genes; this test is about the arrival price.
            brain.decisions.focus_fire = 0;
            brain.decisions.gather_hold = 0;
            // Nor the army-commitment genes (default 1 since 2026-10-31).
            brain.decisions.capture_hold = 0;
            brain.decisions.strike_kill = 0;
            // The recorded plan also predates stale-target re-scouting.
            brain.decisions.stale_nominee = 0;
            brain.decisions.raid_arrival_weight = weight;
            let orders = brain.orders(&view);

            let suicide = Order::MoveUnit {
                unit: UnitId(97),
                to: Hex::new(14, 10),
            };
            if weight == 0 {
                assert!(orders.contains(&suicide), "historical order: {orders:?}");
            } else {
                assert!(
                    !orders.contains(&suicide),
                    "no isolated arrival: {orders:?}"
                );
                assert!(
                    orders.contains(&Order::MoveUnit {
                        unit: UnitId(97),
                        to: Hex::new(12, 11)
                    }),
                    "advances on the safe side of the front: {orders:?}"
                );
            }
            // Same historical human orders for two turns, including the killing shot at
            // (14,10). This bounds the intervention; it is not adaptive human counterplay.
            let mut state_json: serde_json::Value =
                serde_json::from_reader(flate2::read::GzDecoder::new(
                    &include_bytes!("fixtures/sparta-1025/t142.state.json.gz")[..],
                ))
                .unwrap();
            state_json["setup"]["seed"] = serde_json::json!(7732777812994077u64);
            let state: legatum_core::GameState = serde_json::from_value(state_json).unwrap();
            let errors = legatum_core::validate_orders(&state, view.me, &orders);
            assert!(errors.is_empty(), "legal planner orders: {errors:?}");
            let mut tape: Vec<std::collections::BTreeMap<legatum_core::PlayerId, Vec<Order>>> =
                serde_json::from_reader(flate2::read::GzDecoder::new(
                    &include_bytes!("fixtures/sparta-1025/t142-143.orders.json.gz")[..],
                ))
                .unwrap();
            if weight == 0 {
                // Compare only the unit orders. City builds and the research queue respond to
                // the new tech effects, while the recorded tactical intervention stays fixed.
                let units = |o: &Vec<Order>| -> Vec<Order> {
                    o.iter()
                        .filter(|o| {
                            !matches!(
                                o,
                                Order::SetProduction { .. } | Order::SetResearchQueue { .. }
                            )
                        })
                        .cloned()
                        .collect()
                };
                assert_eq!(
                    units(&tape[0][&view.me]),
                    units(&orders),
                    "entire original plan at zero weight"
                );
            }
            tape[0].insert(view.me, orders);
            let (next, _) = legatum_core::resolve_turn(&state, &tape[0]);
            if weight > 0 {
                let moved = &next.units[&UnitId(97)];
                assert_eq!(moved.pos, Hex::new(12, 11));
                assert_eq!(moved.hp, 100);
            }
            let (next, _) = legatum_core::resolve_turn(&next, &tape[1]);
            assert_eq!(
                next.units.contains_key(&UnitId(97)),
                weight > 0,
                "the Phalangite survives the recorded reply only with the arrival price"
            );
        }
    }

    #[test]
    fn raid_weights_never_credit_troops_that_cannot_arrive_but_allow_a_ready_force() {
        for ready in [false, true] {
            let mut view = ViewBuilder::new(30, 16)
                .city(1, ME, Hex::new(3, 8), 3)
                .unit(1, ME, UnitKind::Infantry, Hex::new(12, 8))
                .unit(
                    2,
                    ME,
                    UnitKind::Infantry,
                    if ready {
                        Hex::new(12, 9)
                    } else {
                        Hex::new(9, 8)
                    },
                )
                .unit(
                    3,
                    ME,
                    UnitKind::Infantry,
                    if ready {
                        Hex::new(13, 6)
                    } else {
                        Hex::new(9, 9)
                    },
                )
                .unit(50, ENEMY, UnitKind::Archer, CITY)
                .build();
            view.diplomacy = front().diplomacy;
            for weight in [1, 50, 100] {
                let mut p = planner(&view, 100);
                p.decisions.raid_arrival_weight = weight;
                let army = p.ctx.own_units.clone();
                let targets = p.targets(false);
                p.raid_field_army(&army, view.team_of(ENEMY), 100, &targets);
                if ready {
                    assert!(
                        p.orders.len() >= 2,
                        "viable ready force at {weight}: {:?}",
                        p.orders
                    );
                    assert!(p.assault_groups.iter().any(|g| g.len() >= 2));
                } else {
                    assert!(
                        p.orders.is_empty(),
                        "distant units cannot fund a lone arrival at {weight}: {:?}",
                        p.orders
                    );
                }
            }
        }
    }

    #[test]
    fn a_contain_war_sieges_the_city_its_force_outweighs() {
        let view = front();
        let mut off = planner(&view, 0);
        let army = off.ctx.own_units.clone();
        off.siege_war(&army, false);
        assert!(off.orders.is_empty(), "gene off: nothing new");

        let mut p = planner(&view, 100);
        let army = p.ctx.own_units.clone();
        p.siege_war(&army, false);
        let moves = moves(&p);
        // The units are ordered to their siege tiles (the core paths them over the turns).
        for archer in [UnitId(1), UnitId(2)] {
            let (_, to) = moves
                .iter()
                .find(|(u, _)| *u == archer)
                .unwrap_or_else(|| panic!("{archer:?} sieges: {:?}", p.orders));
            assert!((1..=2).contains(&to.distance(CITY)), "{archer:?} to {to:?}");
        }
        let (_, screen) = moves
            .iter()
            .find(|(u, _)| *u == UnitId(3))
            .expect("the Spearman screens");
        assert_eq!(
            screen.distance(CITY),
            2,
            "Walls strike: the screen keeps off the walls"
        );
    }

    /// Gene `joint_assault` (8.9.25 Celts: Aztec units walked into Lugdunum's strike range one
    /// at a time): a force that cannot reach its tiles in range this turn stages outside it.
    #[test]
    fn joint_assault_stages_outside_the_strike_range_until_the_force_can_step_in() {
        let view = front();
        let mut p = planner(&view, 100);
        p.decisions.joint_assault = 1;
        let army = p.ctx.own_units.clone();
        p.siege_war(&army, false);
        let moves = moves(&p);
        assert_eq!(moves.len(), 3, "{:?}", p.orders);
        for (unit, to) in moves {
            assert!(
                to.distance(CITY) >= 3,
                "{unit:?} waits outside the walls' range, not at {to:?}"
            );
        }
    }

    /// The same front three hexes out, against a broken city: the group in reach takes it, so it
    /// steps into range together.
    #[test]
    fn joint_assault_steps_in_together_when_the_group_in_reach_takes_the_city() {
        let mut view = front();
        for (id, pos) in [
            (1, Hex::new(11, 9)),
            (2, Hex::new(11, 10)),
            (3, Hex::new(11, 8)),
        ] {
            view.units
                .iter_mut()
                .find(|u| u.id == UnitId(id))
                .unwrap()
                .pos = pos;
        }
        view.cities.iter_mut().find(|c| c.pos == CITY).unwrap().hp = 20;
        view.units
            .iter_mut()
            .find(|u| u.id == UnitId(50))
            .unwrap()
            .hp = 20;
        let mut p = planner(&view, 100);
        p.decisions.joint_assault = 1;
        let army = p.ctx.own_units.clone();
        p.siege_war(&army, false);
        let moves = moves(&p);
        assert_eq!(moves.len(), 3, "{:?}", p.orders);
        assert!(
            moves.iter().all(|(_, to)| to.distance(CITY) <= 2),
            "{moves:?}"
        );
    }

    #[test]
    fn steel_artillery_can_take_a_screened_firing_position_behind_forest() {
        let mut view = front();
        let gun = view.units.iter_mut().find(|u| u.id == UnitId(1)).unwrap();
        gun.kind = UnitKind::Artillery;
        gun.pos = Hex::new(12, 8);
        let screen = view.units.iter_mut().find(|u| u.id == UnitId(3)).unwrap();
        screen.pos = Hex::new(11, 8);
        view.research.known.push(legatum_core::TechId::Steel);
        let block = Hex::new(13, 8);
        let i = view.index(block).unwrap();
        view.tiles[i].terrain = Some(Terrain::Forest);
        let mut p = planner(&view, 100);
        let gun = p.ctx.own_units.iter().find(|u| u.id == UnitId(1)).unwrap();
        assert!(p.ranged_unit_clear(gun, gun.pos, CITY));
        let army = p.ctx.own_units.clone();
        p.siege_war(&army, false);
        assert!(
            p.has_acted(UnitId(1)),
            "artillery joins the siege: {:?}",
            p.orders
        );
        assert!(p.has_acted(UnitId(3)), "melee screens it: {:?}", p.orders);
        let moves = moves(&p);
        let gun_to = moves
            .iter()
            .find(|(u, _)| *u == UnitId(1))
            .map(|(_, h)| *h)
            .unwrap_or(Hex::new(12, 8));
        let screen_to = moves
            .iter()
            .find(|(u, _)| *u == UnitId(3))
            .map(|(_, h)| *h)
            .unwrap_or(Hex::new(11, 8));
        assert_eq!(gun_to.distance(CITY), 2);
        assert!(
            screen_to.distance(gun_to) <= 1,
            "screen {screen_to:?} must cover gun {gun_to:?}"
        );
    }

    #[test]
    fn a_force_that_does_not_outweigh_the_defence_keeps_the_safe_staging() {
        let view = front();
        let mut p = planner(&view, 300);
        let army = p.ctx.own_units.clone();
        p.siege_war(&army, false);
        assert!(p.orders.is_empty(), "{:?}", p.orders);
    }

    #[test]
    fn a_siege_at_the_city_grinds_on_past_a_forecast_miss() {
        // Two Archers and a Spearman two hexes from the walled city: the siege outweighs one
        // Spearman's defence, so the campaign is priced as a grind instead of ending.
        let mut view = front();
        for (id, pos) in [
            (1, Hex::new(12, 8)),
            (2, Hex::new(12, 9)),
            (3, Hex::new(12, 7)),
        ] {
            view.units
                .iter_mut()
                .find(|u| u.id == UnitId(id))
                .unwrap()
                .pos = pos;
        }
        let city = view.cities.iter().find(|c| c.pos == CITY).unwrap().clone();
        let ids = [UnitId(1), UnitId(2), UnitId(3)];
        let mut p = planner(&view, 100);
        p.decisions.campaign_persist = 0;
        assert!(p.grind_forecast(&city, &ids).is_none(), "gene off");
        p.decisions.campaign_persist = 100;
        let grind = p.grind_forecast(&city, &ids).expect("the siege grinds on");
        assert!((2..=12).contains(&grind.turns), "{}", grind.turns);
        assert!(grind.losses > 0);
        // A force that no longer outweighs the defence ends the campaign as before.
        p.decisions.campaign_persist = 300;
        assert!(p.grind_forecast(&city, &ids).is_none());
    }

    #[test]
    fn idle_war_turns_count_from_our_last_attack() {
        let view = front();
        let mut p = planner(&view, 100);
        p.decisions.war_pressure = 10;
        let enemy = view.team_of(ENEMY);
        // At war since turn 5, never attacked: idle for the whole war.
        assert_eq!(p.idle_war_turns(enemy), Some(view.turn - 5));
        // A shot at the enemy city counts as fighting this turn.
        p.note_war_fights(&[Order::RangedAttack {
            unit: UnitId(1),
            target: CITY,
        }]);
        assert_eq!(p.idle_war_turns(enemy), Some(0));
        assert_eq!(p.idle_war_excess(enemy), 0);
    }
}
