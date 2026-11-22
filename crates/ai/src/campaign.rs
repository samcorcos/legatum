//! A war objective survives individual planning passes. Only public, fogged PlayerView facts
//! may create, revise or end it; saves serialize the same commitment as a live session.

use crate::analysis::{offense_power, threat_power};
use crate::scripted::Planner;
use legatum_core::diplomacy::RelationState;
use legatum_core::{AiDifficulty, CityId, Faction, Order, PlayerView, TeamId, UnitId, UnitKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Campaign {
    pub target: CityId,
    pub start_turn: u32,
    pub committed: Vec<UnitId>,
    /// First turn whose unchanged economy forecast ends with negative Gold.
    pub deadline: Option<u32>,
    /// Expected VP swing in hundredths, net of expected unit losses at Production value.
    pub expected_value: i32,
    pub target_was_unwalled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignEnd {
    Captured,
    NegativeValue,
    InsufficientArmy,
    WallsBeforeArrival,
    TargetGone,
    Peace,
    Deadline,
    /// The goal-driven plan chose another objective (`plan.rs`).
    PlanChanged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignEvent {
    pub turn: u32,
    pub target: CityId,
    /// None means started; otherwise the explicit end reason.
    pub end: Option<CampaignEnd>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PausedCampaign {
    pub target: CityId,
    pub force_power: i32,
    pub city_hp: i32,
    pub walls: bool,
    pub deadline: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FundDelivery {
    pub ordered_turn: u32,
    pub bought_in: Option<CityId>,
    pub kind: Option<legatum_core::UnitKind>,
    pub unit: Option<UnitId>,
    pub destination: CityId,
    pub offensive: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CampaignMemory {
    pub active: Option<Campaign>,
    /// A Gold-fund recruit or refit keeps the objective it was paid for until it reaches
    /// the front. The purchase resolves after planning, so its unit id is bound next turn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fund_deliveries: Vec<FundDelivery>,
    /// A failed plan cannot restart on the same observation; new strength, damage,
    /// a cleared wall, or a funded clock must improve it first.
    pub paused: Option<PausedCampaign>,
    /// Captors still hold a damaged city while resistance and the war persist.
    pub held_capture: Option<(CityId, Vec<UnitId>)>,
    pub events: Vec<CampaignEvent>,
    /// Last turn this player owned each city. Own cities are always present in PlayerView;
    /// keep this in saved AI memory so a recently lost city can be recognized without
    /// reading the engine's hidden history.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub last_owned: std::collections::BTreeMap<CityId, u32>,
    /// Hard's goal-driven plan (`plan.rs`).
    pub plan: crate::plan::PlanMemory,
    /// The turn each rival city was last in sight (gene `stale_intel`; filled only while it
    /// is on). A city first noted out of sight starts its clock on that turn.
    #[serde(default)]
    pub city_seen: std::collections::BTreeMap<CityId, u32>,
    /// Gene `found_here`: the found orders given last turn on a tile the site search left out
    /// (Settler, tile, turn). A Settler still standing there failed to found (an unseen city
    /// within spacing), so it rescouts instead of ordering the same found again.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub found_tried: Vec<(UnitId, legatum_core::Hex, u32)>,
    /// Plain Hard's Settler guard (`hard_defense.rs`): the escort kept beside each Settler, by
    /// Settler, until it founds its city or is back in one of ours. Absent when no Settler
    /// has an escort, so saves without one keep their shape.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub settler_escorts: std::collections::BTreeMap<UnitId, UnitId>,
    /// A committed operation (gene `operations`, `operation.rs`). Kept after it ends so its
    /// log can be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<crate::operation::Operation>,
    /// Gene `short_objective` (`short_objective.rs`): the short-horizon objective held.
    #[serde(
        default,
        skip_serializing_if = "crate::short_objective::ShortMemory::is_empty"
    )]
    pub short: crate::short_objective::ShortMemory,
}

/// Observer-only version of the campaign's visible capture gate. It uses exactly the
/// same public combat rollout and value score for both recorded master and branch games.
pub fn favorable_visible_capture(view: &PlayerView, against: TeamId) -> bool {
    let faction = view.faction_of(view.me);
    let mut planner = Planner::new(view, crate::Knobs::for_faction(faction), faction, 0);
    planner.difficulty = AiDifficulty::Hard;
    view.cities.iter().any(|city| {
        if !city.visible || view.team_of(city.owner) != against || !view.at_war(view.me, city.owner)
        {
            return false;
        }
        let committed = planner.campaign_force(city);
        planner
            .campaign_siege_estimate(city, &committed)
            .is_some_and(|(turns, _)| {
                planner.campaign_value(city, &committed) > 0
                    && planner.campaign_deadline().is_none_or(|deadline| {
                        deadline > view.turn.saturating_add(turns.max(0) as u32)
                    })
            })
    })
}

impl Planner<'_> {
    /// Strategic value of pressing a leader's capital or point-holding city with
    /// a coalition. Public army totals are discounted by the distance of visible
    /// units from this city; an absent front contributes nothing. This is a plan
    /// price, not a combat override: the siege forecast still decides the assault.
    pub(crate) fn coalition_city_push(&self, city: &legatum_core::view::CityView) -> i32 {
        if self.decisions.coalition_push == 0 {
            return 0;
        }
        let v = self.view();
        let target = v.team_of(city.owner);
        if !v.teams_at_war(v.team, target)
            || (self.strict_leader(70) != Some(target)
                && crate::hold::rival_hold_of(v, target).is_none())
        {
            return 0;
        }
        let points = self.hold_city_points(city) as i32;
        let prize = 800 * i32::from(city.is_capital) + 500 * points;
        if prize == 0 {
            return 0;
        }
        let own_distance = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.hp >= 30)
            .map(|u| u.pos.distance(city.pos))
            .min()
            .unwrap_or(99);
        let own_reach = (12 - own_distance).clamp(0, 10);
        if own_reach == 0 {
            return 0;
        }
        let coalition: i32 = v
            .standings
            .iter()
            .filter(|s| s.team != target && v.teams_at_war(s.team, target))
            .map(|s| {
                let distance = v
                    .units
                    .iter()
                    .filter(|u| {
                        u.military && u.kind != UnitKind::Scout && v.team_of(u.owner) == s.team
                    })
                    .map(|u| u.pos.distance(city.pos))
                    .min()
                    .unwrap_or(99);
                crate::diplomacy::strength(v, s.team) * (12 - distance).clamp(0, 10) / 10
            })
            .sum();
        let defender = crate::diplomacy::strength(v, target).max(1);
        let odds = (coalition * 100 / defender).clamp(0, 100);
        let own_force = crate::diplomacy::strength(v, v.team) * own_reach / 10;
        let contribution = (own_force * 100 / coalition.max(1)).clamp(0, 100);
        let fronts = self.coalition_fronts(target).min(2);
        (prize as i64 * odds as i64 * own_reach as i64 * contribution as i64 * (4 + fronts) as i64
            / 400_000) as i32
    }

    /// Gene `stale_intel`: note which rival cities are in sight this turn. A city first noted
    /// out of sight starts its clock now (nothing says how old that picture is).
    pub(crate) fn note_city_sightings(&mut self) {
        if self.decisions.stale_intel == 0 {
            return;
        }
        let v = self.view();
        let turn = v.turn;
        for c in v.cities.iter().filter(|c| !v.same_team(v.me, c.owner)) {
            if c.visible {
                self.campaigns.city_seen.insert(c.id, turn);
            } else {
                self.campaigns.city_seen.entry(c.id).or_insert(turn);
            }
        }
    }

    /// Gene `stale_intel`: turns since `city` was last in sight (0 while it is).
    pub(crate) fn city_age(&self, city: &legatum_core::view::CityView) -> i32 {
        if city.visible {
            return 0;
        }
        let v = self.view();
        let seen = self
            .campaigns
            .city_seen
            .get(&city.id)
            .copied()
            .unwrap_or(v.turn);
        v.turn.saturating_sub(seen) as i32
    }

    /// Gene `stale_intel`: the picture of `city` is older than the gene's turns.
    pub(crate) fn stale_city(&self, city: &legatum_core::view::CityView) -> bool {
        let limit = self.decisions.stale_intel;
        limit > 0 && !city.owner.is_barbarian() && self.city_age(city) > limit
    }

    /// Gene `stale_intel`: what a stale rival city has probably become since it was seen.
    /// Public signals only: its owner past the first era has researched Masonry (every era's
    /// technologies come first under the strict gate) and walls its cities once they are
    /// threatened; a city grows about a person every 16 turns (Lugano 3 to 7 in 72 turns);
    /// and it has healed. `None` when the picture is not stale.
    pub(crate) fn aged_city(
        &self,
        city: &legatum_core::view::CityView,
    ) -> Option<legatum_core::view::CityView> {
        if !self.stale_city(city) {
            return None;
        }
        let v = self.view();
        let mut aged = city.clone();
        let owner_era = v
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .map_or(1, |p| p.era);
        if owner_era >= 2 && v.faction_of(city.owner) != Faction::Mongols {
            aged.walls = true;
        }
        aged.population = city.population + (self.city_age(city) / 16).min(5);
        aged.hp = aged.max_hp;
        aged.strength = legatum_core::combat::city_defense_for_rules(
            v.rule_experiments,
            v.faction_of(city.owner),
            aged.population,
            aged.walls,
            aged.fort_tier(),
            aged.hp,
            v.era_of(city.owner),
        )
        .total;
        Some(aged)
    }

    /// A field wing can press a newly founded open city while one regiment
    /// holds the last capture. This uses only observed city and unit state.
    pub(crate) fn sparta_chain_target(&self, city: &legatum_core::view::CityView) -> bool {
        let v = self.view();
        if self.decisions.chain_press == 0
            || self.ctx.own_cities.len() < 3
            || city.owner.is_barbarian()
            || city.walls
            || (!v.at_war(v.me, city.owner)
                && !crate::opportunity::legal_declaration(v, v.team_of(city.owner)))
            || v.turn > self.decisions.chain_press_turn as u32
        {
            return false;
        }
        let recent = self.campaigns.events.iter().rev().find(|e| {
            e.end == Some(CampaignEnd::Captured)
                && self.ctx.own_cities.iter().any(|home| home.id == e.target)
        });
        let Some(home) = recent
            .filter(|e| v.turn <= e.turn + 24)
            .and_then(|e| self.ctx.own_cities.iter().find(|home| home.id == e.target))
        else {
            return false;
        };
        home.hp >= 60
            && home.pos.distance(city.pos) <= 8
            && self
                .ctx
                .own_units
                .iter()
                .any(|u| u.military && u.pos == home.pos)
            && !self
                .ctx
                .threat_units
                .iter()
                .any(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(home.pos) <= 3)
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.military
                        && u.kind == UnitKind::Soldier
                        && u.hp >= self.decisions.sparta_expedition_min_hp
                        && u.pos != home.pos
                        && u.pos.distance(city.pos) <= 5
                })
                .count()
                >= 3
    }

    /// An Sparta army released by its first capture can catch the next nearby
    /// open city before its defender finishes Walls. The siege rollout is too
    /// pessimistic while only the vanguard is in reach, so keep the rest of the
    /// field force in the decision instead of waiting for the wall to finish.
    pub(crate) fn sparta_open_followup(&self, city: &legatum_core::view::CityView) -> bool {
        let v = self.view();
        self.decisions.open_followup > 0
            && self.ctx.own_cities.len() == 2
            && self.campaigns.events.iter().any(|e| {
                e.end == Some(CampaignEnd::Captured)
                    && self.ctx.own_cities.iter().any(|home| home.id == e.target)
            })
            && v.turn <= self.decisions.open_followup_turn as u32
            && !city.owner.is_barbarian()
            && (v.faction_of(city.owner) == Faction::India
                || self.decisions.sparta_conquest_bundle > 0)
            && !city.walls
            && self
                .ctx
                .own_cities
                .iter()
                .any(|home| home.pos.distance(city.pos) <= self.decisions.open_prize_radius)
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.military
                        && u.kind == UnitKind::Soldier
                        && u.hp >= self.decisions.sparta_expedition_min_hp
                })
                .count()
                >= 5
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.military
                        && u.kind == UnitKind::Soldier
                        && u.hp >= self.decisions.sparta_expedition_min_hp
                        && u.pos.distance(city.pos) <= 6
                })
                .count()
                >= 2
    }

    /// Gene `assault_wing_forecast`: the early Sparta prize the assault wing is trained for,
    /// the valued conquest nomination when it is an open city within ten hexes, else the
    /// nearest such city.
    pub(crate) fn wing_target(&self) -> Option<&legatum_core::view::CityView> {
        if self.decisions.assault_wing == 0
            || self.decisions.assault_wing_forecast == 0
            || !self.hard_defense()
        {
            return None;
        }
        let v = self.view();
        let near = |c: &legatum_core::view::CityView| {
            self.ctx
                .own_cities
                .iter()
                .map(|h| h.pos.distance(c.pos))
                .min()
                .filter(|d| *d <= self.decisions.open_prize_radius)
        };
        self.conquest_city()
            .filter(|c| !c.walls && near(c).is_some())
            .or_else(|| {
                v.cities
                    .iter()
                    .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner) && !c.walls)
                    .filter_map(|c| near(c).map(|d| (d, c.id, c)))
                    .min_by_key(|(d, id, _)| (*d, *id))
                    .map(|(_, _, c)| c)
            })
    }

    /// Our healthy melee regiments, placed on the city's ring as if they had arrived, take it
    /// within three rounds by the siege forecast; eight are always enough.
    /// With `within`, only regiments that close range count (the staged wing).
    pub(crate) fn assault_wing_ready(
        &self,
        city: &legatum_core::view::CityView,
        within: Option<i32>,
    ) -> bool {
        let v = self.view();
        let melee: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.ranged == 0 && u.hp >= 50)
            .filter(|u| within.is_none_or(|d| u.pos.distance(city.pos) <= d))
            .collect();
        if melee.len() >= 8 {
            return true;
        }
        let ring: Vec<_> = city
            .pos
            .ring(1)
            .into_iter()
            .filter(|h| {
                v.tile(*h)
                    .is_some_and(|t| t.terrain.and_then(|x| x.move_cost()).is_some())
            })
            .collect();
        if melee.is_empty() || ring.is_empty() {
            return false;
        }
        let placed: Vec<_> = melee
            .iter()
            .enumerate()
            .map(|(i, u)| (*u, ring[i % ring.len()], 0))
            .collect();
        let mut prospective = v.clone();
        let team = v.team_of(city.owner);
        if let Some(r) = prospective
            .diplomacy
            .relations
            .iter_mut()
            .find(|r| r.between(v.team, team))
        {
            r.state = legatum_core::diplomacy::RelationState::War;
        }
        self.forecast_siege(&prospective, city, &placed)
            .is_some_and(|s| s.turns <= 3)
    }

    pub(crate) fn sparta_opening_byzantium(&self, city: &legatum_core::view::CityView) -> bool {
        let v = self.view();
        self.decisions.lone_city_strike > 0
            && self.ctx.own_cities.len() == 1
            && !city.owner.is_barbarian()
            && (v.faction_of(city.owner) == Faction::Byzantium
                || self.decisions.sparta_conquest_bundle > 0)
            && !city.walls
            && v.turn <= self.decisions.opening_strike_turn as u32
            && self.ctx.own_cities[0].pos.distance(city.pos)
                <= if self.value_first_target() {
                    self.decisions.open_prize_radius
                } else {
                    self.decisions.lone_city_strike_radius
                }
    }

    /// A wall is worth paying through only for a city whose capture can change the race.
    /// Population is public and stands in for the private yields that would compound after
    /// capture; the last city also removes an opponent's whole future economy.
    pub(crate) fn sparta_walled_prize(&self, city: &legatum_core::view::CityView) -> bool {
        self.decisions.walled_prize > 0
            && self.hard_defense()
            && city.walls
            && (city.is_capital
                || city.population >= 3
                || self.conquest_victory_value(city) > 0
                || self
                    .view()
                    .players
                    .iter()
                    .any(|p| p.id == city.owner && p.cities == 1))
    }

    /// The first turn the reserve ledger's runway runs out (`funding.rs`).
    pub(crate) fn campaign_deadline(&self) -> Option<u32> {
        self.gold_runway()
            .map(|runway| self.view().turn.saturating_add(runway))
    }

    /// A city is a VP asset even if taking it scores no milestone immediately: it supplies
    /// population, land and future production. Denial matters more against a race leader.
    pub(crate) fn campaign_value(
        &self,
        city: &legatum_core::view::CityView,
        units: &[UnitId],
    ) -> i32 {
        if self.decisions.late_capture_value > 0 {
            let forecast = self
                .campaign_siege_estimate(city, units)
                .map(|(turns, losses)| crate::plan::Forecast {
                    turns,
                    losses,
                    force: units.to_vec(),
                    march: 0,
                })
                .or_else(|| self.grind_forecast(city, units));
            if let Some(forecast) = forecast {
                let base = crate::winchance::WinFacts::of(self.view())
                    .with_relative(&self.decisions)
                    .refined();
                return self
                    .price_campaign(&base, base.chance(), city, &forecast)
                    .value;
            }
        }
        let v = self.view();
        let progress = v
            .standings
            .iter()
            .find(|s| s.team == v.team_of(city.owner))
            .map_or(0, |s| 100 * s.vp as i32 / s.target.max(1) as i32);
        let own_vp = 100 + 100 * self.conquest_victory_value(city);
        // A team counting down to victory: a capture that takes its VP can break the hold.
        let denied_vp = 50 + progress / 2 + crate::hold::denial_priority(v, city.owner, || 0) / 2;
        // Gene `campaign_persist`: a siege grinding on past a forecast miss loses a city strike a
        // turn (`plan.rs::grind_forecast`), not a share of every unit standing there.
        let grind = || self.grind_forecast(city, units).map(|f| f.losses);
        let expected_losses: i32 = self
            .campaign_siege_estimate(city, units)
            .map(|(_, losses)| losses)
            .or_else(grind)
            .unwrap_or_else(|| {
                units
                    .iter()
                    .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
                    .map(|u| {
                        let cost = crate::tuned(
                            self.view()
                                .rule_experiments
                                .unit_stats_for(self.faction, u.kind)
                                .cost,
                        );
                        cost * (100 - u.hp.clamp(0, 100)) / 100
                            + if city.walls { cost / 3 } else { cost / 8 }
                    })
                    .sum()
            });
        own_vp + denied_vp + self.coalition_city_push(city) - expected_losses
    }

    /// Gene `finish_kill`: a siege that has the city nearly down (HP at most this percent of
    /// its maximum) and still has a shooter on it is not ended for want of a capturer while
    /// one can arrive within the march window: the nearest melee-capable units join the
    /// force, and it goes on while the campaign's value with them stays positive (8.9.24:
    /// the Mongols had unwalled Lugano at 61 of 200 HP on T61 when two bought archers killed
    /// their two melee units; the campaign ended for an insufficient army, the city healed
    /// and built Walls before the next one). Returns whether the campaign continues.
    pub(crate) fn finish_kill(
        &self,
        city: &legatum_core::view::CityView,
        committed: &mut Vec<UnitId>,
    ) -> bool {
        match self.finish_kill_force(city, committed) {
            Some(force) => {
                *committed = force;
                true
            }
            None => false,
        }
    }

    /// Gene `finish_kill`: this unit is committed to a campaign whose city is nearly down,
    /// so it marches even from a garrison.
    pub(crate) fn finishing(&self, unit: UnitId) -> bool {
        let pct = self.decisions.finish_kill;
        pct > 0
            && self.campaigns.active.as_ref().is_some_and(|c| {
                c.committed.contains(&unit)
                    && self.view().cities.iter().any(|city| {
                        city.id == c.target && city.hp * 100 <= pct * city.max_hp.max(1)
                    })
            })
    }

    /// The force that finishes a nearly-down city (`finish_kill`), if the kill is on.
    pub(crate) fn finish_kill_force(
        &self,
        city: &legatum_core::view::CityView,
        committed: &[UnitId],
    ) -> Option<Vec<UnitId>> {
        let pct = self.decisions.finish_kill;
        if pct == 0
            || !self.hard_defense()
            || !city.visible
            || city.hp * 100 > pct * city.max_hp.max(1)
        {
            return None;
        }
        let at = self.view().index(city.pos)?;
        // Something of ours is still on the city: a shooter within a step of its range or
        // a troop beside it.
        let holds = self.ctx.own_units.iter().any(|u| {
            u.military
                && u.hp > 0
                && (committed.contains(&u.id) || u.pos.distance(city.pos) <= 3)
                && if u.ranged > 0 {
                    u.pos.distance(city.pos) <= u.range.max(1) + 1
                } else {
                    u.pos.distance(city.pos) <= 2
                }
        });
        if !holds {
            return None;
        }
        let capturers: Vec<UnitId> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.hp >= 50
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .position(|c| c.pos == u.pos)
                        .is_none_or(|i| self.quiet_city(i))
                    && self.ctx.distance_field(u.pos, false)[at]
                        <= self.turn_moves(u) * self.decisions.march_turns
            })
            .map(|u| u.id)
            .collect();
        if !capturers.iter().any(|id| {
            self.ctx
                .own_units
                .iter()
                .any(|u| u.id == *id && crate::composition::charges(u.kind))
        }) {
            return None;
        }
        let mut force = committed.to_vec();
        force.extend(capturers);
        force.sort_unstable();
        force.dedup();
        (self.campaign_value(city, &force) > 0).then_some(force)
    }

    pub(crate) fn campaign_force(&self, city: &legatum_core::view::CityView) -> Vec<UnitId> {
        if self.sparta_walled_prize(city) && city.visible
            || self.sparta_opening_byzantium(city)
            || self.sparta_open_followup(city)
            || self.sparta_chain_target(city)
        {
            // Commit the same reachable force that the siege forecast priced, including
            // reinforcements within the six-turn march. A fixed four-hex circle could omit the units
            // that make a costly wall assault possible.
            let v = self.view();
            let mut prospective;
            let siege_view = if v.at_war(v.me, city.owner) {
                v
            } else {
                prospective = v.clone();
                if let Some(relation) = prospective
                    .diplomacy
                    .relations
                    .iter_mut()
                    .find(|r| r.between(v.team, v.team_of(city.owner)))
                {
                    relation.state = RelationState::War;
                }
                &prospective
            };
            let mut ids: Vec<_> = self
                .strike_force_within(siege_view, city, self.decisions.march_turns)
                .into_iter()
                .map(|(u, _, _)| u.id)
                .collect();
            ids.sort_unstable();
            return ids;
        }
        let mut units: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.pos.distance(city.pos) <= 4
                    && (self.rushing() || !self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
            })
            .map(|u| u.id)
            .collect();
        units.sort_unstable();
        units
    }

    pub(crate) fn campaign_can_take(
        &self,
        city: &legatum_core::view::CityView,
        ids: &[UnitId],
    ) -> bool {
        if city.visible {
            return self.campaign_siege_estimate(city, ids).is_some();
        }
        let army: Vec<_> = ids
            .iter()
            .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id).copied())
            .filter(|u| u.hp >= 30)
            .collect();
        if !army.iter().any(|u| crate::composition::charges(u.kind)) {
            return false;
        }
        let mine: i32 = army.iter().map(|u| offense_power(self.view(), u)).sum();
        let garrison: i32 = self
            .view()
            .units
            .iter()
            .filter(|u| u.military && u.pos == city.pos && u.owner == city.owner)
            .map(|u| threat_power(self.view(), u))
            .sum();
        let need = city.strength + garrison + if city.walls { city.strength / 2 } else { 0 };
        mine * 100 >= need.max(1) * self.attack_threshold()
    }

    pub(crate) fn end_campaign(&mut self, campaign: Campaign, reason: CampaignEnd) {
        let v = self.view();
        if reason == CampaignEnd::Captured {
            self.campaigns.held_capture = Some((campaign.target, campaign.committed.clone()));
        }
        if matches!(
            reason,
            CampaignEnd::Deadline
                | CampaignEnd::NegativeValue
                | CampaignEnd::InsufficientArmy
                | CampaignEnd::WallsBeforeArrival
        ) {
            if let Some(city) = v.cities.iter().find(|c| c.id == campaign.target) {
                self.campaigns.paused = Some(PausedCampaign {
                    target: campaign.target,
                    force_power: campaign
                        .committed
                        .iter()
                        .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
                        .map(|u| offense_power(v, u))
                        .sum(),
                    city_hp: city.hp,
                    walls: city.walls,
                    deadline: campaign.deadline,
                });
            }
            self.campaign_withdraw = campaign.committed;
        }
        self.campaigns.events.push(CampaignEvent {
            turn: v.turn,
            target: campaign.target,
            end: Some(reason),
        });
    }

    /// The Celts sack rather than keep cities, and without gene `celtic_campaign` they run no
    /// city campaign, siege or assault at all. With it they do, priced by the sack's Gold and
    /// the city the owner loses (`plan.rs::price_campaign`; idea #25, the 8.9.4 Swiss review:
    /// the second-strongest army held a war on the leader for 76 turns with no campaign).
    pub(crate) fn takes_no_cities(&self) -> bool {
        self.faction == Faction::Celts
            && self.decisions.celtic_campaign == 0
            && self.rule_on(legatum_core::experiments::FactionRule::CeltsNoKeep)
    }

    pub(crate) fn reconcile_campaign(&mut self) {
        if self.takes_no_cities() {
            return;
        }
        let v = self.view();
        if self.decisions.retake_hold > 0 {
            for city in v.cities.iter().filter(|city| city.owner == v.me) {
                self.campaigns.last_owned.insert(city.id, v.turn);
            }
            self.campaigns
                .last_owned
                .retain(|_, turn| v.turn.saturating_sub(*turn) <= 20);
        }
        if let Some((id, _)) = &self.campaigns.held_capture {
            let hold = v.cities.iter().any(|c| {
                c.id == *id
                    && c.owner == v.me
                    && (c.detail.as_ref().is_some_and(|d| d.resistance > 0) || c.hp * 2 < c.max_hp)
                    && v.players
                        .iter()
                        .any(|p| p.alive && p.team != v.team && v.teams_at_war(v.team, p.team))
            });
            if !hold {
                self.campaigns.held_capture = None;
            }
        }
        // A running operation's strike is its own campaign: only its abort conditions end it.
        if self.operation_strike().is_some_and(|(_, target)| {
            self.campaigns
                .active
                .as_ref()
                .is_some_and(|c| c.target == target)
        }) {
            return;
        }
        if let Some(mut active) = self.campaigns.active.take() {
            let city = v.cities.iter().find(|c| c.id == active.target);
            let reason = match city {
                None => Some(CampaignEnd::TargetGone),
                Some(c) if c.owner == v.me => Some(CampaignEnd::Captured),
                Some(c) if !v.at_war(v.me, c.owner) => Some(CampaignEnd::Peace),
                Some(c) => {
                    active
                        .committed
                        .retain(|id| self.ctx.own_units.iter().any(|u| u.id == *id));
                    // Gene `campaign_persist`: reinforcements that reach the city join the
                    // siege (the Golden Horde: a campaign begun with the two units in reach
                    // ended the next turn with six beside the city, counted as two).
                    if self.sparta_walled_prize(c)
                        || self.sparta_opening_byzantium(c)
                        || self.sparta_open_followup(c)
                        || self.sparta_chain_target(c)
                        || (self.decisions.campaign_persist > 0 && self.hard_defense())
                    {
                        active.committed.extend(self.campaign_force(c));
                        active.committed.sort_unstable();
                        active.committed.dedup();
                    }
                    active.deadline = self.campaign_deadline();
                    active.expected_value = self.campaign_value(c, &active.committed);
                    let wall_rose_early = active.target_was_unwalled
                        && c.walls
                        && active.committed.iter().all(|id| {
                            self.ctx
                                .own_units
                                .iter()
                                .find(|u| u.id == *id)
                                .is_none_or(|u| u.pos.distance(c.pos) > 1)
                        });
                    // A single new defender can make the one-turn siege forecast fail
                    // just after our opening volley. Do not discard a funded, already
                    // engaged Sparta assault while its capture force is still present.
                    let engaged_open_assault = self.decisions.engaged_assault_keep > 0
                        && (v.faction_of(c.owner) == Faction::India
                            || self.sparta_opening_byzantium(c)
                            || self.decisions.sparta_conquest_bundle > 0)
                        && !c.walls
                        && c.visible
                        && c.hp < c.max_hp
                        && v.turn < active.start_turn + 6
                        && active
                            .committed
                            .iter()
                            .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
                            .filter(|u| {
                                u.hp >= self.decisions.sparta_expedition_min_hp.min(50)
                                    && u.pos.distance(c.pos) <= 3
                            })
                            .count()
                            >= 3
                        && active.committed.iter().any(|id| {
                            self.ctx.own_units.iter().any(|u| {
                                u.id == *id
                                    && crate::composition::charges(u.kind)
                                    && u.hp >= self.decisions.sparta_expedition_min_hp.min(50)
                                    && u.pos.distance(c.pos) <= 3
                            })
                        });
                    let staging_open_assault = (self.sparta_opening_byzantium(c)
                        || self.sparta_open_followup(c)
                        || self.sparta_chain_target(c))
                        && v.turn
                            <= active.start_turn
                                + if self.sparta_open_followup(c) { 16 } else { 6 } as u32
                        && active
                            .committed
                            .iter()
                            .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
                            .filter(|u| {
                                u.hp >= self.decisions.sparta_expedition_min_hp
                                    && u.pos.distance(c.pos) <= 6
                            })
                            .count()
                            >= 4;
                    // With Siege to cover the force (gene `siege_for_walls`) a rising wall is a
                    // price in the forecast, not the end of the campaign.
                    let covered = self.walls_are_a_price();
                    if wall_rose_early
                        && ((!self.sparta_walled_prize(c) && !covered)
                            || !self.campaign_can_take(c, &active.committed))
                    {
                        Some(CampaignEnd::WallsBeforeArrival)
                    } else if active.expected_value < 0 {
                        Some(CampaignEnd::NegativeValue)
                    } else if !engaged_open_assault
                        && !staging_open_assault
                        && !self.campaign_can_take(c, &active.committed)
                        // Gene `campaign_persist`: a siege that still outweighs the defence
                        // grinds on (`plan.rs::grind_forecast`).
                        && self.grind_forecast(c, &active.committed).is_none()
                        && !self.retake_hold(c, &active)
                        && !self.finish_kill(c, &mut active.committed)
                    {
                        Some(CampaignEnd::InsufficientArmy)
                    } else if active.deadline.is_some_and(|d| {
                        // Funding before cutting: a short runway carries the campaign only
                        // if its capture comes first (assault now); otherwise it is ended
                        // deliberately, its army withdraws and peace is offered, before the
                        // treasury runs dry and the engine picks a unit to disband.
                        d < v.turn + self.runway_floor()
                            && self
                                .campaign_siege_estimate(c, &active.committed)
                                .is_none_or(|(turns, _)| turns as u32 > d.saturating_sub(v.turn))
                    }) {
                        Some(CampaignEnd::Deadline)
                    } else {
                        None
                    }
                }
            };
            if let Some(reason) = reason {
                self.end_campaign(active, reason);
            } else {
                self.campaigns.active = Some(active);
            }
        }
        if self.campaigns.active.is_some()
            || self.campaigns.held_capture.is_some()
            || !self.campaign_withdraw.is_empty()
        {
            return;
        }
        // Under a goal-driven plan, new campaigns start only from the plan (`plan.rs`).
        if self.plan_routing {
            return;
        }
        // A prospective strike may nominate a city before diplomacy has actually opened
        // the war. Commitment begins only from the next observed wartime view.
        let target = self
            .opening_strike_city()
            .filter(|c| v.at_war(v.me, c.owner))
            .or_else(|| {
                self.opportunity_strike()
                    .and_then(|s| v.cities.iter().find(|c| c.id == s.city))
                    .filter(|c| v.at_war(v.me, c.owner))
            })
            .or_else(|| self.conquest_city().filter(|c| v.at_war(v.me, c.owner)));
        let Some(id) = target.filter(|c| !v.same_team(v.me, c.owner)).map(|c| c.id) else {
            return;
        };
        if let Some(city) = v.cities.iter().find(|c| c.id == id) {
            self.begin_campaign(city);
        }
    }

    /// Keep a recent retake objective long enough for its committed reinforcements to arrive.
    /// The floor is deliberately independent of the optimistic capture forecast.
    pub(crate) fn retake_hold(
        &self,
        city: &legatum_core::view::CityView,
        active: &Campaign,
    ) -> bool {
        let v = self.view();
        if self.decisions.retake_hold <= 0
            || v.turn.saturating_sub(active.start_turn) >= self.decisions.retake_hold as u32
            || !self
                .campaigns
                .last_owned
                .get(&city.id)
                .is_some_and(|turn| *turn < v.turn && v.turn.saturating_sub(*turn) <= 20)
        {
            return false;
        }
        let ours: i32 = active
            .committed
            .iter()
            .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
            .filter(|u| u.pos.distance(city.pos) <= 4)
            .map(|u| offense_power(v, u))
            .sum();
        let garrison: i32 = v
            .units
            .iter()
            .filter(|u| u.military && u.owner == city.owner && u.pos == city.pos)
            .map(|u| threat_power(v, u))
            .sum();
        let defenders = city.strength + garrison + if city.walls { city.strength / 2 } else { 0 };
        ours * 2 >= defenders.max(1)
    }

    /// Commit the local force to `city`, unless the same failed plan would just restart or
    /// the force can't take it, fund it, or gain by it.
    pub(crate) fn begin_campaign(&mut self, city: &legatum_core::view::CityView) {
        let v = self.view();
        let committed = self.campaign_force(city);
        if self.campaigns.paused.as_ref().is_some_and(|paused| {
            paused.target == city.id
                && city.hp >= paused.city_hp
                && (!paused.walls || city.walls)
                && match (self.campaign_deadline(), paused.deadline) {
                    (None, None) => true,
                    (None, Some(_)) => false,
                    (Some(_), None) => true,
                    (Some(now), Some(then)) => now <= then,
                }
                && committed
                    .iter()
                    .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
                    .map(|u| offense_power(v, u))
                    .sum::<i32>()
                    <= paused.force_power
        }) {
            return;
        }
        if !self.campaign_can_take(city, &committed) || !self.campaign_funded(city, &committed) {
            return;
        }
        let expected_value = self.campaign_value(city, &committed);
        if expected_value <= 0 {
            return;
        }
        let campaign = Campaign {
            target: city.id,
            start_turn: v.turn,
            committed,
            deadline: self.campaign_deadline(),
            expected_value,
            target_was_unwalled: !city.walls,
        };
        self.campaigns.events.push(CampaignEvent {
            turn: v.turn,
            target: city.id,
            end: None,
        });
        self.campaigns.paused = None;
        self.campaigns.active = Some(campaign);
    }

    /// No new campaign on a runway under the floor unless its forecast capture comes first.
    fn campaign_funded(&self, city: &legatum_core::view::CityView, committed: &[UnitId]) -> bool {
        self.gold_runway()
            .filter(|runway| *runway < self.runway_floor())
            .is_none_or(|runway| {
                self.campaign_siege_estimate(city, committed)
                    .is_some_and(|(turns, _)| turns.max(0) as u32 <= runway)
            })
    }

    pub(crate) fn campaign_protects(&self, id: UnitId) -> bool {
        self.campaigns
            .active
            .as_ref()
            .is_some_and(|c| c.expected_value > 0 && c.committed.contains(&id))
            || self
                .campaigns
                .held_capture
                .as_ref()
                .is_some_and(|(_, units)| units.contains(&id))
    }

    /// The tactical planner can begin a city attack that the strategic nomination did not
    /// foresee. Own the actual attack as a campaign so next turn either pursues it or records
    /// a reasoned withdrawal. Only the original, observed war state can authorize this.
    pub(crate) fn adopt_started_siege(&mut self, observed: &PlayerView, orders: &[Order]) {
        if self.takes_no_cities()
            || self.campaigns.active.is_some()
            || self.campaigns.held_capture.is_some()
            || !self.campaign_withdraw.is_empty()
        {
            return;
        }
        let city = observed
            .cities
            .iter()
            .filter(|c| observed.at_war(observed.me, c.owner))
            .find(|c| {
                orders.iter().any(|order| match order {
                    Order::RangedAttack { target, .. } | Order::MoveUnit { to: target, .. } => {
                        *target == c.pos
                    }
                    _ => false,
                })
            });
        let Some(city) = city else { return };
        if self.plan_routing
            && self.routed_plan().and_then(|p| p.kind.war_target())
                != Some(observed.team_of(city.owner))
        {
            return;
        }
        let committed = self.campaign_force(city);
        if !self.campaign_can_take(city, &committed) || !self.campaign_funded(city, &committed) {
            return;
        }
        let expected_value = self.campaign_value(city, &committed);
        if expected_value <= 0
            || self.campaign_deadline().is_some_and(|deadline| {
                self.campaign_siege_estimate(city, &committed)
                    .is_none_or(|(turns, _)| {
                        deadline <= observed.turn.saturating_add(turns.max(0) as u32)
                    })
            })
        {
            return;
        }
        let campaign = Campaign {
            target: city.id,
            start_turn: observed.turn,
            expected_value,
            committed,
            deadline: self.campaign_deadline(),
            target_was_unwalled: !city.walls,
        };
        self.campaigns.events.push(CampaignEvent {
            turn: observed.turn,
            target: city.id,
            end: None,
        });
        self.campaigns.paused = None;
        self.campaigns.active = Some(campaign);
    }
}
