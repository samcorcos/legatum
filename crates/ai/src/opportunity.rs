//! Opportunistic strikes: declare war on a neighbour and take its city when the force nearby
//! can capture it within a few turns and the gain clearly beats the expected losses and the
//! risk of retaliation.
//!
//! The user's rush scenarios (2026-10-04) found the gap: four Sparta units next to an
//! unwalled one-city neighbour at Peace waited six turns for a second Archer, because
//! readiness was a fixed recipe (two ranged and two melee units). Here readiness is a
//! forecast: the same combat forecasts the engine shows a player, played out over a few
//! rounds of fire and assault from the assault tiles the force can reach. Value decides,
//! never a recipe, and the evidence is the PlayerView only (a fogged city is assumed to hold
//! one fortified Spearman).

use crate::analysis::{city_power, defense_power, threat_power};
use crate::production::city_keep_value;
use crate::scripted::Planner;
use legatum_core::data;
use legatum_core::diplomacy::RelationState;
use legatum_core::rules::movement::can_step;
use legatum_core::view::{CityView, PlayerView, UnitView};
use legatum_core::{
    AttackMode, CityId, Faction, Hex, Order, TeamId, UnitId, UnitKind, forecast_attack_mode,
};
use std::collections::{BTreeMap, BTreeSet};

/// Gold the Celtic AI expects a capture to bring: 60 per population, the exact amount the
/// forecast shows (`economy::celts_sack_gold`). Other factions keep cities and take nothing here.
pub(crate) fn sack_gold_value(
    rules: legatum_core::experiments::RuleExperiments,
    city: &CityView,
    captor: Faction,
    me: legatum_core::PlayerId,
) -> i32 {
    use legatum_core::experiments::FactionRule;
    // A city we founded comes home intact: no sack (designer, 2026-11-14).
    if captor != Faction::Celts
        || city.founder == Some(me)
        || rules.rule_off(FactionRule::CeltsNoKeep)
        || rules.rule_off(FactionRule::CeltsSack)
    {
        return 0;
    }
    legatum_core::economy::celts_sack_gold(city.population)
}

/// Units below this health don't join a strike.
const STRIKE_MIN_HP: i32 = 50;
/// A stand-in for an unseen garrison.
const PHANTOM: UnitId = UnitId(u32::MAX);

#[derive(Clone, Debug)]
pub(crate) struct Strike {
    pub team: TeamId,
    pub city: CityId,
    pub pos: Hex,
    /// The units the forecast used, each with its assault tile.
    pub force: Vec<(UnitId, Hex)>,
    /// Forecast turns to capture, approach included.
    pub turns: i32,
    pub gain: i32,
    pub cost: i32,
}

pub(crate) struct Siege {
    pub turns: i32,
    /// Expected Production lost: whole units for the dead; health heals, so damage counts
    /// half its share of the unit's cost (the turns spent healing).
    pub losses: i32,
}

pub(crate) fn legal_declaration(v: &PlayerView, team: TeamId) -> bool {
    v.diplomacy.actions.iter().any(|a| {
        a.order
            == Order::DeclareWar {
                against: team,
                breach: false,
            }
            && a.blocked_reason.is_none()
    })
}

fn garrison_at(v: &PlayerView, at: Hex) -> Option<UnitId> {
    v.units
        .iter()
        .filter(|u| u.pos == at && u.military && v.at_war(v.me, u.owner))
        .min_by_key(|u| u.id)
        .map(|u| u.id)
}

/// Gene `relief_forecast`: the city owner's visible melee units that can step into the city at
/// `at` this turn, healthiest first (core `rules/movement.rs::reserves`: an owner's plain move
/// into its own attacked city goes first, and each relief unit is fought in turn).
pub(crate) fn relief_units(v: &PlayerView, at: Hex) -> Vec<UnitId> {
    let Some(owner) = v.cities.iter().find(|c| c.pos == at).map(|c| c.owner) else {
        return Vec::new();
    };
    let mut relief: Vec<(i32, UnitId)> = v
        .units
        .iter()
        .filter(|e| {
            e.military
                && e.owner == owner
                && e.pos != at
                && e.ranged == 0
                && e.pos.distance(at) <= e.max_moves.max(1)
        })
        .map(|e| (-e.hp, e.id))
        .collect();
    relief.sort();
    relief.into_iter().map(|(_, id)| id).collect()
}

/// Gene `relief_forecast`: the next relief unit still alive steps into the cleared city at
/// `at`; false when none is left.
pub(crate) fn relieve(v: &mut PlayerView, relief: &mut Vec<UnitId>, at: Hex) -> bool {
    while !relief.is_empty() {
        let id = relief.remove(0);
        if let Some(u) = v.units.iter_mut().find(|u| u.id == id) {
            u.pos = at;
            u.fortify = 0;
            return true;
        }
    }
    false
}

/// Turns for `u` to walk `path` (excluding its start), with moves refreshed each turn.
fn walk_turns(v: &PlayerView, u: &UnitView, path: &[Hex]) -> Option<i32> {
    let mut turns = if path.is_empty() { 0 } else { 1 };
    let mut moves = u.max_moves;
    for &h in path {
        let cost = v.tile(h)?.move_cost()?;
        if !can_step(moves, u.max_moves, cost) {
            turns += 1;
            moves = u.max_moves;
        }
        moves = (moves - cost).max(0);
    }
    Some(turns)
}

impl Planner<'_> {
    /// Capture clock for an existing, visible wartime objective. The same combat rollout
    /// used for a new strike determines whether waiting past the treasury deadline works.
    pub(crate) fn campaign_siege_estimate(
        &self,
        city: &CityView,
        committed: &[UnitId],
    ) -> Option<(i32, i32)> {
        if !city.visible || !self.view().at_war(self.view().me, city.owner) {
            return None;
        }
        let force: Vec<_> = self
            .strike_force(self.view(), city)
            .into_iter()
            .filter(|(u, _, _)| committed.contains(&u.id))
            .collect();
        if let Some(siege) = self.forecast_siege(self.view(), city, &force) {
            return Some((siege.turns, siege.losses));
        }
        // The local assault can fail while a prized wall is still capturable with
        // committed reinforcements. Use the same full-force forecast for value,
        // funding and continuation as the one that justified the campaign.
        (self.sparta_walled_prize(city) || self.sparta_opening_byzantium(city))
            .then(|| self.campaign_forecast(city, Some(committed)))
            .flatten()
            .map(|f| (f.turns, f.losses))
    }

    /// The best strike open to us this turn, against a team we are at war with or may
    /// legally declare on. Computed once per planning pass.
    pub(crate) fn opportunity_strike(&self) -> Option<&Strike> {
        self.strike_plan
            .get_or_init(|| self.choose_strike())
            .as_ref()
            .filter(|strike| {
                self.campaigns
                    .active
                    .as_ref()
                    .is_none_or(|c| c.target == strike.city)
            })
    }

    /// A strike against `team` that is worth declaring for (or pressing, at war).
    pub(crate) fn strike_against(&self, team: TeamId) -> Option<&Strike> {
        self.opportunity_strike().filter(|s| s.team == team)
    }

    fn choose_strike(&self) -> Option<Strike> {
        // Fast rollout policy: no strike forecasts.
        if self.fast >= 1 {
            return None;
        }
        let v = self.view();
        // Gene `strike_eagerness`: how much a faction's story makes it want an early strike, as a
        // percent on the gain (Sparta and Mongols favour early strikes).
        let identity = self.decisions.strike_eagerness;
        if v.diplomacy.legacy || self.ctx.own_cities.is_empty() {
            return None;
        }
        let eagerness = 60 + self.knobs.aggression + identity;
        let mut teams = BTreeSet::new();
        for c in &v.cities {
            if !c.owner.is_barbarian()
                && !v.same_team(v.me, c.owner)
                && v.players.iter().any(|p| p.id == c.owner && p.alive)
            {
                teams.insert(v.team_of(c.owner));
            }
        }
        let mut best: Option<Strike> = None;
        for team in teams {
            let at_war = v.teams_at_war(v.team, team);
            if !at_war && !legal_declaration(v, team) {
                continue;
            }
            // No unit of ours could join a strike on any of its cities: nothing to evaluate
            // (every force would be empty), so skip building its prospective view.
            if !v.cities.iter().any(|c| {
                v.team_of(c.owner) == team
                    && self
                        .ctx
                        .own_units
                        .iter()
                        .any(|u| self.could_strike(u, c.pos, self.decisions.approach_turns))
            }) {
                continue;
            }
            let mut prospective = v.clone();
            if !at_war {
                let Some(relation) = prospective
                    .diplomacy
                    .relations
                    .iter_mut()
                    .find(|r| r.between(v.team, team))
                else {
                    continue;
                };
                relation.state = RelationState::War;
            }
            for city in v.cities.iter().filter(|c| v.team_of(c.owner) == team) {
                let Some(strike) = self.evaluate_strike(&prospective, team, city, eagerness) else {
                    continue;
                };
                let better = best.as_ref().is_none_or(|b| {
                    (
                        strike.gain - strike.cost,
                        -strike.turns,
                        std::cmp::Reverse(strike.city),
                    ) > (b.gain - b.cost, -b.turns, std::cmp::Reverse(b.city))
                });
                if better {
                    best = Some(strike);
                }
            }
        }
        best
    }

    fn evaluate_strike(
        &self,
        prospective: &PlayerView,
        team: TeamId,
        city: &CityView,
        eagerness: i32,
    ) -> Option<Strike> {
        let v = self.view();
        let force = self.strike_force(prospective, city);
        if !force
            .iter()
            .any(|(u, _, _)| crate::composition::charges(u.kind))
        {
            return None;
        }
        let siege = self.forecast_siege(prospective, city, &force)?;
        // Do not open a new war when the public treasury forecast runs out before
        // the capture. Post-capture resistance can be funded by changing city
        // production or trimming uncommitted forces as the campaign unfolds.
        // Existing wars still get the choice to assault now or withdraw.
        if !v.teams_at_war(v.team, team)
            && self.campaign_deadline().is_some_and(|deadline| {
                deadline <= v.turn.saturating_add(siege.turns.max(0) as u32)
            })
        {
            return None;
        }
        let base = city_keep_value(city);
        let last_city = v
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .is_some_and(|p| p.cities <= 1);
        // A city taken from the public VP leader slows the race for everyone (pillar 11):
        // worth up to a second city as the leader goes from 30% to 55% of its target.
        let leader = v
            .standings
            .iter()
            .filter(|s| s.team != v.team && s.target > 0)
            .map(|s| (s.vp * 100 / s.target) as i32)
            .max()
            .unwrap_or(0);
        let progress = v
            .standings
            .iter()
            .find(|s| s.team == team && s.target > 0)
            .map_or(0, |s| (s.vp * 100 / s.target) as i32);
        let race = if progress == leader && progress > 30 {
            base * (progress - 30).min(25) / 25
        } else {
            0
        };
        // A team counting down to victory: more again for a capture that hurts its hold.
        let hold = base * crate::hold::denial_priority(v, city.owner, || 0) / 200;
        // A kept city is worth its keep value to its captor. A sacked one is gone: what the
        // capture is worth is the harm to its owner, a share of that value (gene
        // `sack_loss_percent`), beside the Gold taken (below).
        let kept = if self.sacks_city(city) {
            base * self.decisions.sack_loss_percent / 100
        } else {
            base
        };
        let gain = (kept
            + race
            + hold
            + if last_city { base } else { 0 }
            + city.capture_bounty.unwrap_or(0) / 4
            + sack_gold_value(
                self.view().rule_experiments,
                city,
                self.faction,
                self.ctx.me,
            ) / 4
            + 400 * self.sack_camp_vp(city)
            + self.hoard_denial_value(city)
            + self.greece_captive_tech_value(city)
            + 100 * (self.capital_capture_vp(city) + self.capital_race_vp(city))
            + self.swarm_weakness_value(city)
            + self.flower_wars_campaign_value(city)
            + self.roman_strike_promotion_value(&force, siege.turns)
            + 400 * self.conquest_victory_value(city))
            * eagerness
            / 100;
        let committed: BTreeSet<UnitId> = force.iter().map(|(u, _, _)| u.id).collect();
        let retaliation: i32 = self
            .ctx
            .own_cities
            .iter()
            .map(|home| {
                let their: i32 = prospective
                    .units
                    .iter()
                    .filter(|e| {
                        e.military
                            && e.kind != UnitKind::Scout
                            && v.team_of(e.owner) == team
                            && e.pos.distance(city.pos) > 1
                            && e.pos.distance(home.pos) <= self.decisions.retaliation_radius
                    })
                    .map(|e| threat_power(prospective, e))
                    .sum();
                if their == 0 {
                    return 0;
                }
                let ours = city_power(v, home) / 2
                    + self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| {
                            u.military
                                && !committed.contains(&u.id)
                                && u.pos.distance(home.pos) <= 2
                        })
                        .map(|u| defense_power(v, u))
                        .sum::<i32>();
                (their - ours).max(0) * city_keep_value(home) / their
            })
            .sum();
        // Their army is split across their wars: only a share of it can answer us.
        let their_wars = v
            .standings
            .iter()
            .filter(|s| s.team != team && s.team != v.team && v.teams_at_war(team, s.team))
            .filter(|s| v.players.iter().any(|p| p.team == s.team && p.alive))
            .count() as i32;
        let theirs = crate::diplomacy::strength(v, team) / (1 + their_wars);
        let ours = crate::diplomacy::strength(v, v.team).max(1);
        let outmatched = ((theirs - ours).max(0) * 100 / ours).min(100);
        let other_wars = v
            .standings
            .iter()
            .filter(|s| s.team != team && v.teams_at_war(v.team, s.team))
            .filter(|s| v.players.iter().any(|p| p.team == s.team && p.alive))
            .count() as i32;
        // Retaliation near home and a stronger army overall are the same enemy army
        // answering now or later: count the larger, not both.
        let counterattack = retaliation.max(base * outmatched / 200);
        // A fresh peace has strategic value: armies have disengaged and border plans
        // have changed. Reopening the same front immediately spends that value. The
        // cost fades with time, so a genuinely decisive chance can still justify it.
        let peace_reversal = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, team))
            .filter(|r| r.state == RelationState::Peace && r.version >= 2)
            .map_or(0, |r| {
                let horizon = 30_u32;
                base * horizon.saturating_sub(v.turn.saturating_sub(r.effective_turn)) as i32
                    / horizon as i32
            });
        let cost = siege.losses + counterattack + base * other_wars / 4 + peace_reversal;
        // The margin covers what the forecast can't see: a fogged garrison, a force still
        // marching. A visible city with the force in place (one turn at most) needs less.
        let in_position = city.visible && force.iter().all(|(_, _, t)| *t <= 1);
        let margin = if in_position { 115 } else { 125 };
        (gain - cost > 0 && gain * 100 >= cost * margin).then(|| Strike {
            team,
            city: city.id,
            pos: city.pos,
            force: force.iter().map(|(u, to, _)| (u.id, *to)).collect(),
            turns: siege.turns,
            gain,
            cost,
        })
    }

    /// Gene `roman_promotion_value`: the cities Rome's veterans would found from the
    /// promotions this strike earns, at a new city's keep value.
    fn roman_strike_promotion_value(&self, force: &[(&UnitView, Hex, i32)], turns: i32) -> i32 {
        let ids: Vec<UnitId> = force.iter().map(|(u, _, _)| u.id).collect();
        let approach = force.iter().map(|(_, _, t)| *t).min().unwrap_or(0);
        self.roman_siege_promotions(&ids, turns - approach)
            * crate::production::new_city_keep_value(1)
            * self.decisions.roman_promotion_value
            / 100
    }

    /// Healthy units that can reach an assault tile within the approach window, each with
    /// its tile and approach turns. Garrisons stay home.
    pub(crate) fn strike_force<'u>(
        &'u self,
        prospective: &PlayerView,
        city: &CityView,
    ) -> Vec<(&'u UnitView, Hex, i32)> {
        self.strike_force_within(prospective, city, self.decisions.approach_turns)
    }

    /// A unit `strike_force_within` considers for a city at `target`.
    fn could_strike(&self, u: &UnitView, target: Hex, approach: i32) -> bool {
        let min_hp = if self.faction == Faction::Sparta {
            self.decisions.sparta_strike_min_hp
        } else {
            STRIKE_MIN_HP
        };
        u.military
            && u.kind != UnitKind::Scout
            && u.hp >= min_hp
            && !self
                .ctx
                .own_cities
                .iter()
                .any(|c| c.pos == u.pos && !self.garrison_spared(c))
            && u.pos.distance(target) <= approach * u.max_moves.max(1) + 2
    }

    /// As `strike_force`, for units that reach an assault tile within `approach` turns.
    pub(crate) fn strike_force_within<'u>(
        &'u self,
        prospective: &PlayerView,
        city: &CityView,
        approach: i32,
    ) -> Vec<(&'u UnitView, Hex, i32)> {
        // Every caller passes this planner's view with the city's owner at war with us
        // (the view itself when already at war), so the force depends only on the city, the
        // window and that war state: computed once per pass.
        let key = (
            city.id,
            approach,
            prospective.at_war(prospective.me, city.owner),
        );
        let cached = self.strike_forces.borrow().get(&key).cloned();
        let ids = match cached {
            Some(ids) => ids,
            None => {
                let ids: std::rc::Rc<Vec<(UnitId, Hex, i32)>> = std::rc::Rc::new(
                    self.strike_force_uncached(prospective, city, approach)
                        .into_iter()
                        .map(|(u, h, t)| (u.id, h, t))
                        .collect(),
                );
                self.strike_forces.borrow_mut().insert(key, ids.clone());
                ids
            }
        };
        ids.iter()
            .filter_map(|(id, h, t)| {
                let u = *self.ctx.own_units.iter().find(|u| u.id == *id)?;
                Some((u, *h, *t))
            })
            .collect()
    }

    fn strike_force_uncached<'u>(
        &'u self,
        prospective: &PlayerView,
        city: &CityView,
        approach: i32,
    ) -> Vec<(&'u UnitView, Hex, i32)> {
        let v = self.view();
        let mut taken: BTreeSet<Hex> = BTreeSet::new();
        let mut units: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| self.could_strike(u, city.pos, approach))
            .collect();
        units.sort_by_key(|u| (u.pos.distance(city.pos), u.id));
        // Own units (one owner) on one view: one entry rule for every route.
        let entry = prospective.entry_table(v.me, legatum_core::UnitKind::Spearman);
        let mut force = Vec::new();
        for u in units {
            let rings: Vec<i32> = if crate::composition::shooter_only(u.kind, u.ranged) {
                (1..=u.range.max(1)).rev().collect()
            } else {
                vec![1]
            };
            let moves = u.max_moves.max(1);
            // Each turn walks at most max_moves steps (every step costs at least 1), so
            // ceil(distance / moves) bounds a tile's turns from below.
            let bound = |h: Hex| {
                if h == u.pos {
                    0
                } else {
                    (h.distance(u.pos) + moves - 1) / moves
                }
            };
            let mut candidates: Vec<(i32, i32, Hex)> = rings
                .iter()
                .flat_map(|r| city.pos.ring(*r))
                .filter(|h| {
                    !taken.contains(h)
                        && (*h == u.pos
                            || !v.units.iter().any(|o| o.pos == *h)
                                && !v.cities.iter().any(|c| c.pos == *h))
                        && v.tile(*h)
                            .and_then(|t| t.terrain)
                            .and_then(|t| t.move_cost())
                            .is_some()
                })
                .map(|h| (bound(h), h.distance(u.pos), h))
                .filter(|(b, _, _)| *b <= approach)
                .collect();
            // Best (turns, distance, tile): in bound order, a candidate whose bound tuple is
            // already worse than the best found can't win, nor can any after it.
            candidates.sort();
            let mut tile: Option<(i32, i32, Hex)> = None;
            for (b, d, h) in candidates {
                if tile.is_some_and(|best| (b, d, h) > best) {
                    break;
                }
                let turns = if h == u.pos {
                    0
                } else {
                    let path = if u.owner == v.me {
                        prospective.movement_path_with(&entry, u.pos, h)
                    } else {
                        prospective.movement_path(u.owner, u.kind, u.pos, h)
                    };
                    match path.and_then(|path| walk_turns(v, u, &path)) {
                        Some(t) => t,
                        None => continue,
                    }
                };
                if turns <= approach && tile.is_none_or(|best| (turns, d, h) < best) {
                    tile = Some((turns, d, h));
                }
            }
            if let Some((turns, _, h)) = tile {
                taken.insert(h);
                force.push((u, h, turns));
            }
        }
        force
    }

    /// Play the strike out on the prospective view: each round enemy fire, then the volley
    /// and melee of the units already in position, with the engine's own
    /// expected damage. Returns the capture turn and the expected losses, or None if the
    /// force cannot take the city within the window.
    pub(crate) fn forecast_siege(
        &self,
        prospective: &PlayerView,
        city: &CityView,
        force: &[(&UnitView, Hex, i32)],
    ) -> Option<Siege> {
        self.forecast_siege_air(prospective, city, force, &[])
    }

    /// `forecast_siege` with aircraft striking the city in the ranged volley: each plane with
    /// the turn it can first strike (a plane bought this turn strikes from the next), added to
    /// the view's hangars if it is not there (gene `decisive_buy`).
    pub(crate) fn forecast_siege_air(
        &self,
        prospective: &PlayerView,
        city: &CityView,
        force: &[(&UnitView, Hex, i32)],
        planes: &[(legatum_core::view::AircraftView, i32)],
    ) -> Option<Siege> {
        self.forecast_siege_rounds(prospective, city, force, planes, None)
    }

    /// `forecast_siege_air` playing `rounds` past the arrival instead of the siege window.
    pub(crate) fn forecast_siege_rounds(
        &self,
        prospective: &PlayerView,
        city: &CityView,
        force: &[(&UnitView, Hex, i32)],
        planes: &[(legatum_core::view::AircraftView, i32)],
        rounds: Option<i32>,
    ) -> Option<Siege> {
        if force.is_empty() {
            return None;
        }
        let mut sim = prospective.clone();
        for (plane, _) in planes {
            if !sim.aircraft.iter().any(|a| a.id == plane.id) {
                sim.aircraft.push(plane.clone());
            }
        }
        let ids: Vec<UnitId> = force.iter().map(|(u, _, _)| u.id).collect();
        sim.units
            .retain(|u| u.owner != sim.me || ids.contains(&u.id));
        for (u, to, _) in force {
            if let Some(s) = sim.units.iter_mut().find(|s| s.id == u.id) {
                if s.pos != *to {
                    s.fortify = 0;
                }
                s.pos = *to;
            }
        }
        // Walls halve every attack the force makes unless its unit bought Siege: count the units
        // that have it, and (gene `siege_for_walls`) the ones we would buy it for, priced below.
        let cover = if self.hard_defense() && self.decisions.siege_for_walls > 0 {
            let ids: Vec<UnitId> = force.iter().map(|(u, _, _)| u.id).collect();
            self.add_siege_cover(&mut sim, city, &ids, true)
        } else {
            crate::siege_buy::CoverUse::default()
        };
        // The Siege we would still have to buy is Gold the capture costs, counted in Production at
        // the upgrade rate (2 Gold a Production); buying consumes the unit's action turn.
        let planned_cost = cover.gold / 2;
        let build_delay = i32::from(self.decisions.siege_timeline > 0 && cover.planned > 0);
        if !city.visible && garrison_at(&sim, city.pos).is_none() {
            // Fog hides the garrison: assume one fortified Spearman holds the city.
            let mut phantom = force[0].0.clone();
            let stats = data::unit_stats_for(sim.faction_of(city.owner), UnitKind::Spearman);
            phantom.id = PHANTOM;
            phantom.owner = city.owner;
            phantom.kind = UnitKind::Spearman;
            phantom.pos = city.pos;
            phantom.hp = data::UNIT_HP;
            phantom.strength = stats.strength;
            phantom.ranged = 0;
            phantom.range = 0;
            phantom.fortify = data::FORTIFY_MAX;
            phantom.level = 0;
            phantom.goto = None;
            phantom.path.clear();
            sim.units.push(phantom);
        }
        let approach = force.iter().map(|(_, _, t)| *t).max().unwrap_or(0);
        // A unit that needs `t` turns to reach its tile acts from round t + 1; the enemy
        // shoots at every unit of the force from round 1, approach included (rush-offense
        // replays: an Archer two turns out was shot to 15 HP before it could fire).
        let arrives: BTreeMap<UnitId, i32> = force.iter().map(|(u, _, t)| (u.id, *t)).collect();
        let ready = |id: &UnitId, round: i32| arrives.get(id).is_some_and(|t| *t < round);
        let start_hp: BTreeMap<UnitId, i32> = force.iter().map(|(u, _, _)| (u.id, u.hp)).collect();
        let me = sim.me;
        let my_team = sim.team;
        // Gene `siege_window`: how many rounds past the arrival the forecast plays (combat lab
        // 2026-10-17: at four rounds it called 33% of winnable cities hopeless; a melee storm on
        // a garrisoned city often takes five or six).
        let window = if let Some(rounds) = rounds {
            rounds
        } else if self.hard_defense() && self.decisions.siege_window > 0 {
            self.decisions.siege_window
        } else {
            self.decisions.siege_rounds
        };
        // Gene `relief_forecast`: the owner's melee units that can step in relieve the city. They
        // still count as attackers in the fire step: the defender's orders are unknown, and
        // sparing our units their blows made the storm charge too early (2026-11-13).
        let mut relief = if self.decisions.relief_forecast > 0 {
            relief_units(&sim, city.pos)
        } else {
            Vec::new()
        };
        for round in 1..=approach + window {
            // Enemy fire first: the city's strike (walls only) and any ranged defender.
            let mut incoming = BTreeMap::<UnitId, i32>::new();
            if let Some(c) = sim.cities.iter().find(|c| c.pos == city.pos && c.hp > 0) {
                let shot = sim
                    .units
                    .iter()
                    .filter(|u| ids.contains(&u.id) && u.pos.distance(c.pos) <= 2)
                    .filter_map(|u| {
                        forecast_attack_mode(&sim, u.id, c.pos, AttackMode::Melee)
                            .and_then(|f| f.city_strike)
                            .map(|s| (u.hp, u.id, s.expected_damage(u.hp)))
                    })
                    .min_by_key(|(hp, id, _)| (*hp, *id));
                if let Some((_, id, damage)) = shot {
                    *incoming.entry(id).or_default() += damage;
                }
            }
            let enemies: Vec<UnitView> = sim
                .units
                .iter()
                .filter(|e| e.military && e.id != PHANTOM && sim.at_war(me, e.owner))
                .filter(|e| e.kind != UnitKind::Scout)
                .cloned()
                .collect();
            for enemy in &enemies {
                // On a surprise declaration the defender chose this turn's orders at
                // Peace. Its standing city shot still resolves, but its units cannot
                // retroactively order attacks against a war they had not seen.
                if round == 1 && !self.view().teams_at_war(my_team, sim.team_of(city.owner)) {
                    continue;
                }
                sim.me = enemy.owner;
                sim.team = sim.team_of(enemy.owner);
                let mode = if enemy.ranged > 0 {
                    AttackMode::Ranged
                } else {
                    AttackMode::Melee
                };
                // A unit with both (the Keshig) takes whichever attack hurts more.
                let modes: &[AttackMode] = if crate::composition::dual(enemy.kind, enemy.ranged) {
                    &[AttackMode::Ranged, AttackMode::Melee]
                } else {
                    std::slice::from_ref(&mode)
                };
                // Gene `forecast_garrison_holds`: the besieged city's garrison holds it and
                // only shoots (combat lab, 2026-11-08: Hard garrisons charged out 0 times in
                // 403 siege turns; the forecast had them hitting the weakest attacker every round).
                // 1: only in the siege stance's storm test; 2: in every siege forecast.
                let holds = match self.decisions.forecast_garrison_holds {
                    1 => self.stance_forecast.get(),
                    2 => true,
                    _ => false,
                };
                let modes: &[AttackMode] = if holds && enemy.pos == city.pos {
                    if enemy.ranged > 0 {
                        &[AttackMode::Ranged]
                    } else {
                        &[]
                    }
                } else {
                    modes
                };
                let hit = sim
                    .units
                    .iter()
                    .filter(|u| ids.contains(&u.id))
                    .flat_map(|u| modes.iter().map(move |m| (u, *m)))
                    .filter(|(u, mode)| {
                        *mode == AttackMode::Ranged
                            || u.pos.distance(enemy.pos) <= enemy.max_moves.max(1)
                    })
                    .filter_map(|(u, mode)| {
                        forecast_attack_mode(&sim, enemy.id, u.pos, mode)
                            .map(|f| (f.expected_damage_to_defender, std::cmp::Reverse(u.id)))
                    })
                    .max();
                if let Some((damage, std::cmp::Reverse(id))) = hit {
                    *incoming.entry(id).or_default() += damage;
                }
            }
            sim.me = me;
            sim.team = my_team;
            for u in &mut sim.units {
                u.hp -= incoming.get(&u.id).copied().unwrap_or(0);
            }
            sim.units.retain(|u| u.hp > 0);
            // Our volley shares one snapshot.
            let volley: i32 = sim
                .units
                .iter()
                .filter(|u| {
                    ids.contains(&u.id)
                        && crate::composition::shooter_only(u.kind, u.ranged)
                        && ready(&u.id, round)
                })
                .filter_map(|u| forecast_attack_mode(&sim, u.id, city.pos, AttackMode::Ranged))
                .map(|f| f.expected_damage_to_defender)
                .sum::<i32>()
                + planes
                    .iter()
                    .filter(|(_, arrives)| *arrives < round)
                    .filter_map(|(plane, _)| {
                        forecast_attack_mode(&sim, plane.id, city.pos, AttackMode::Ranged)
                    })
                    .map(|f| f.expected_damage_to_defender)
                    .sum::<i32>();
            crate::analysis::damage_tile_defender(&mut sim, city.pos, volley);
            // Melee, healthiest first; a 0-HP city with no garrison falls to the next one in.
            let mut melee: Vec<(i32, UnitId)> = sim
                .units
                .iter()
                .filter(|u| {
                    ids.contains(&u.id)
                        && crate::composition::charges(u.kind)
                        && ready(&u.id, round)
                })
                .map(|u| (-u.hp, u.id))
                .collect();
            melee.sort();
            // The Hard Sparta wall forecast must price its concentrated melee assault
            // by the engine's split counterattack. Keep other strike forecasts on their
            // established policy while this siege-specific change is measured.
            // Gene `storm_forecast` prices the same split on unwalled cities (1: Sparta,
            // 2: every faction) and the Sparta kill heal: up to 100 HP shared by the
            // attackers whose blows kill the garrison (playtest 8.9.2: the user stormed seven
            // unwalled cities with 2 to 4 Hoplites a turn; the AI waited for shooters).
            let storm = self.hard_defense()
                && match self.decisions.storm_forecast {
                    1 => self.faction == Faction::Sparta,
                    2 => true,
                    _ => false,
                };
            let split = if self.sparta_walled_prize(city)
                || storm
                || self.decisions.split_counterattack > 0
            {
                melee.len().max(1) as i32
            } else {
                1
            };
            // The resolver takes the counter from the strongest attack in the simultaneous
            // snapshot, divides it evenly, and gives its remainder to that attacker.
            let counters = if self.decisions.split_counterattack > 0 {
                let strongest = melee
                    .iter()
                    .filter_map(|(_, id)| {
                        forecast_attack_mode(&sim, *id, city.pos, AttackMode::Melee).map(|f| {
                            (
                                *id,
                                f.attacker_strength.total,
                                f.expected_damage_to_attacker,
                            )
                        })
                    })
                    .max_by_key(|(id, strength, _)| (*strength, std::cmp::Reverse(*id)));
                let mut shares = BTreeMap::new();
                if let Some((strongest_id, _, total)) = strongest {
                    for (_, id) in &melee {
                        shares.insert(
                            *id,
                            total / split + i32::from(*id == strongest_id) * (total % split),
                        );
                    }
                }
                shares
            } else {
                BTreeMap::new()
            };
            let mut struck: Vec<UnitId> = Vec::new();
            for (_, id) in melee {
                let open = garrison_at(&sim, city.pos).is_none()
                    && sim.cities.iter().any(|c| c.pos == city.pos && c.hp == 0);
                if open && !relieve(&mut sim, &mut relief, city.pos) {
                    return Some(self.siege_result(
                        &sim,
                        &start_hp,
                        city.pos,
                        round,
                        planned_cost,
                        build_delay,
                    ));
                }
                let Some(f) = forecast_attack_mode(&sim, id, city.pos, AttackMode::Melee) else {
                    continue;
                };
                let hp = sim.units.iter().find(|u| u.id == id).map_or(0, |u| u.hp);
                let counter = counters
                    .get(&id)
                    .copied()
                    .unwrap_or(f.expected_damage_to_attacker / split);
                if counter >= hp {
                    continue;
                }
                let garrison_before = garrison_at(&sim, city.pos);
                crate::analysis::damage_tile_defender(
                    &mut sim,
                    city.pos,
                    f.expected_damage_to_defender,
                );
                if let Some(u) = sim.units.iter_mut().find(|u| u.id == id) {
                    u.hp -= counter;
                    u.fortify = 0;
                }
                struck.push(id);
                if storm
                    && self.faction == Faction::Sparta
                    && garrison_before.is_some()
                    && garrison_at(&sim, city.pos) != garrison_before
                {
                    let killers: Vec<UnitId> = struck
                        .iter()
                        .copied()
                        .filter(|k| sim.units.iter().any(|u| u.id == *k && u.hp > 0))
                        .collect();
                    let share = data::UNIT_HP / killers.len().max(1) as i32;
                    for u in sim.units.iter_mut().filter(|u| killers.contains(&u.id)) {
                        u.hp = (u.hp + share).min(data::UNIT_HP);
                    }
                }
                if garrison_at(&sim, city.pos).is_none()
                    && sim.cities.iter().any(|c| c.pos == city.pos && c.hp == 0)
                    && !relieve(&mut sim, &mut relief, city.pos)
                {
                    return Some(self.siege_result(
                        &sim,
                        &start_hp,
                        city.pos,
                        round,
                        planned_cost,
                        build_delay,
                    ));
                }
            }
            if !sim
                .units
                .iter()
                .any(|u| ids.contains(&u.id) && crate::composition::charges(u.kind))
            {
                return None;
            }
        }
        None
    }

    fn siege_result(
        &self,
        sim: &PlayerView,
        start: &BTreeMap<UnitId, i32>,
        captured_at: Hex,
        turns: i32,
        planned_cost: i32,
        build_delay: i32,
    ) -> Siege {
        let losses: i32 = self
            .ctx
            .own_units
            .iter()
            .filter_map(|u| {
                let before = *start.get(&u.id)?;
                let after = sim.units.iter().find(|s| s.id == u.id).map_or(0, |s| {
                    if self.faction == Faction::Sparta
                        && self.decisions.sparta_capture_heal_forecast > 0
                        && s.pos.distance(captured_at) <= 2
                    {
                        data::UNIT_HP
                    } else {
                        s.hp
                    }
                });
                let cost = crate::tuned(
                    self.view()
                        .rule_experiments
                        .unit_stats_for(self.faction, u.kind)
                        .cost,
                );
                Some(if after <= 0 {
                    cost
                } else if self.faction == Faction::Sparta
                    && self.decisions.sparta_capture_heal_forecast >= 2
                    && sim
                        .units
                        .iter()
                        .any(|s| s.id == u.id && s.pos.distance(captured_at) <= 2)
                {
                    cost * (before - after) / data::UNIT_HP / 2
                } else {
                    cost * (before - after).max(0) / data::UNIT_HP / 2
                })
            })
            .sum();
        Siege {
            turns: turns + build_delay,
            losses: losses + planned_cost,
        }
    }
}

#[cfg(test)]
mod sparta_wound_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder, off};

    #[test]
    fn an_empty_force_cannot_siege_a_fogged_city() {
        let mut v = ViewBuilder::new(20, 12)
            .city(1, ME, off(3, 5), 3)
            .city(2, crate::fixture::ENEMY, off(10, 5), 3)
            .build();
        let city = v.cities.iter_mut().find(|c| c.owner != ME).unwrap();
        city.visible = false;
        let planner = crate::fixture::planner(&v, 7);
        let city = v.cities.iter().find(|c| c.owner != ME).unwrap();
        assert!(
            planner
                .forecast_siege_rounds(&v, city, &[], &[], None)
                .is_none()
        );
    }

    #[test]
    fn three_non_spartan_attackers_share_one_city_counterattack() {
        let v = ViewBuilder::new(20, 12)
            .faction(Faction::India)
            .city(1, ME, off(3, 5), 3)
            .city(2, crate::fixture::ENEMY, off(10, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(9, 5))
            .unit(2, ME, UnitKind::Spearman, off(10, 4))
            .unit(3, ME, UnitKind::Spearman, off(11, 5))
            .build();
        let city = v.cities.iter().find(|c| c.owner != ME).unwrap();
        let force: Vec<_> = v.units.iter().map(|u| (u, u.pos, 0)).collect();
        let mut old = crate::fixture::planner(&v, 7);
        old.decisions.split_counterattack = 0;
        let old = old.forecast_siege(&v, city, &force).unwrap();
        let default = crate::fixture::planner(&v, 7)
            .forecast_siege(&v, city, &force)
            .unwrap();
        assert!(
            default.losses < old.losses,
            "the default forecast must charge one split counter, not three: {} vs {}",
            default.losses,
            old.losses
        );
    }

    #[test]
    fn a_wounded_sparta_soldier_can_join_a_city_strike_forecast() {
        let v = ViewBuilder::new(20, 12)
            .faction(Faction::Sparta)
            .city(1, ME, off(3, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(6, 5))
            .build();
        let mut wounded = v.units[0].clone();
        wounded.hp = 35;
        let target = off(9, 5);
        let mut sparta = Planner::new(&v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
        assert!(!sparta.could_strike(&wounded, target, 3));
        sparta.decisions.sparta_strike_min_hp = 30;
        assert!(sparta.could_strike(&wounded, target, 3));
        let mut other = Planner::new(&v, Knobs::for_faction(Faction::India), Faction::India, 1);
        other.decisions.sparta_strike_min_hp = 30;
        assert!(!other.could_strike(&wounded, target, 3));
    }

    #[test]
    fn sparta_capture_removes_surviving_nearby_wounds_from_siege_loss_price() {
        let v = ViewBuilder::new(20, 12)
            .faction(Faction::Sparta)
            .city(1, ME, off(3, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(8, 5))
            .build();
        let mut after = v.clone();
        after.units.iter_mut().next().unwrap().hp = 60;
        let start = BTreeMap::from([(after.units[0].id, 100)]);
        let mut p = Planner::new(&v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
        p.decisions.sparta_capture_heal_forecast = 0;
        let old = p.siege_result(&after, &start, off(9, 5), 2, 0, 0).losses;
        assert!(old > 0);
        p.decisions.sparta_capture_heal_forecast = 1;
        assert_eq!(p.siege_result(&after, &start, off(9, 5), 2, 0, 0).losses, 0);
        assert_eq!(
            p.siege_result(&after, &start, off(15, 5), 2, 0, 0).losses,
            old
        );
        let already_wounded = BTreeMap::from([(after.units[0].id, 60)]);
        p.decisions.sparta_capture_heal_forecast = 2;
        assert!(
            p.siege_result(&after, &already_wounded, off(9, 5), 2, 0, 0)
                .losses
                < 0
        );
    }
}
