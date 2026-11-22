//! The coalition's answer to a runaway leader (gene `runaway_coalition`, Hard only).
//!
//! The designer, 2026-11-07, after the Byzantium "Horde Looks East" scenario: "the mongols are
//! clearly running away with this game and none of the other civs are willing to ally with me
//! to counter them. And also, sparta, my one ally, has a big army just sitting off to the side
//! when the mongols leave to attack me and they don't counter attack." The 8.9.26 reviews found
//! the same: the Celts ran a Contain plan on him for 78 turns with no attack, and three AIs
//! offered Peace during his hold.
//!
//! Earlier coalition genes (`coalition_push`, `leader_commit`, `conquest_alarm`,
//! `see-the-leader`) priced the war on the leader and then handed it to the plan layer and the
//! campaign executors, which re-judged it every turn and mostly never marched. What is
//! different here:
//!
//! - **The read** (`runaway`) is one public score: VP lead, an army lead, cities captured
//!   recently, empires eliminated, a hold. Nothing fogged.
//! - **Bit 1, diplomacy**: Alliances with anyone at war with the runaway (the human included)
//!   are valued by how much of its army the pair can bring; calls to arms against it are
//!   answered; no Peace with it while it runs away.
//! - **Bit 2, second front**: when the runaway's main army is visibly committed far from its
//!   own cities, an empire at war with it (or allied with one that is) commits an operation on
//!   its nearest weakly held city. The operation executor (`operation.rs`) carries it out; the
//!   plan layer and the campaign reconciler cannot cancel it.

use crate::diplomacy::{nearest_known, strength};
use crate::operation::{Operation, OperationKind};
use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{AiDifficulty, CityId, PlayerView, TeamId, UnitKind, data};

/// A runaway's army this far from every one of its known cities is committed elsewhere.
pub(crate) const AWAY_DISTANCE: i32 = 8;
/// The second front takes only own units this close to the target (the operation's radius).
pub(crate) const FRONT_RADIUS: i32 = 12;
/// The radius when the strike is all in (bits 8 and 16): armies fighting elsewhere join too.
pub(crate) const ALL_IN_RADIUS: i32 = 20;
/// Turn budget of a second-front strike.
pub(crate) const FRONT_TURNS: u32 = 15;
/// Turns after an operation ends before another second front starts.
const FRONT_COOLDOWN: u32 = 5;
/// Score weight of the share of the runaway's army an Alliance can bring against it.
const ALLY_SLOW_WEIGHT: i32 = 50;

/// The public read of a runaway leader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Runaway {
    pub team: TeamId,
    pub score: i32,
    /// Its VP minus the best other living team's.
    pub vp_lead: i32,
    /// Its public army as a percent of the next strongest team's.
    pub army_percent: i32,
    pub recent_captures: u32,
    pub eliminations: u32,
    /// It is counting down to victory.
    pub hold: bool,
}

/// A team's public army: the sum of its members' Strongest Army values.
fn army(v: &PlayerView, team: TeamId) -> i32 {
    strength(v, team)
}

fn alive(v: &PlayerView, team: TeamId) -> bool {
    v.players.iter().any(|p| p.team == team && p.alive)
}

/// A visible unit's army value (supply unknown, so counted as supplied).
pub(crate) fn unit_army_value(u: &UnitView) -> i32 {
    if !u.military || u.kind == UnitKind::Scout || data::is_aircraft(u.kind) {
        return 0;
    }
    legatum_core::army::combat_value(u.strength, u.ranged, u.hp, u.level, false)
}

/// The runaway score of `team` from public numbers, before the strict-leader test.
pub(crate) fn runaway_score(
    vp_lead: i32,
    army_percent: i32,
    recent_captures: u32,
    eliminations: u32,
    hold: bool,
) -> i32 {
    15 * vp_lead.clamp(0, 4)
        + (army_percent - 100).clamp(0, 100) / 4
        + 10 * recent_captures.min(4) as i32
        + 20 * eliminations.min(2) as i32
        + 60 * i32::from(hold)
}

#[cfg(test)]
thread_local! {
    /// Tests: every draw with a chance above zero comes up (set by the `planner` fixture).
    static FORCE_DRAWS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The wariness arithmetic (see `Planner::wariness`): recent captures, eliminations, the
/// target's army as a percent of the observer's, its VP lead and its nearness (0 to 100).
pub(crate) fn wariness_score(
    captures: u32,
    eliminations: u32,
    army_percent: i32,
    vp_lead: i32,
    near: i32,
) -> i32 {
    let taken = [0, 3, 8, 30, 45, 55][captures.min(5) as usize];
    let base = taken
        + 20 * eliminations.min(2) as i32
        + ((army_percent - 100) / 5).clamp(0, 30)
        + (6 * vp_lead).clamp(0, 24);
    base * (25 + near * 75 / 100) / 100
}

impl Planner<'_> {
    /// A public leader fighting another empire has a visible army away from a reachable
    /// city, or a recent public capture on that front. This opens a second front before
    /// a funded siege exists, provided no enemy army is threatening our own cities.
    pub(crate) fn busy_leader_front(&self, target: TeamId) -> bool {
        if self.difficulty != AiDifficulty::Hard
            || self.decisions.busy_leader_front == 0
            || self.runaway_read().is_none_or(|r| {
                r.team != target || (r.recent_captures == 0 && r.eliminations == 0 && r.vp_lead < 3)
            })
            || self
                .top_threat_team()
                .is_some_and(|threat| threat != target)
            || self.view().teams_at_war(self.view().team, target)
            || !self.reachable_war_target(target)
            || self.collapsing()
        {
            return false;
        }
        // Keep a defender near every own city while leaving a force that can threaten
        // a reachable leader city. Ordinary war costs still price a thin offensive.
        let field = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .count();
        if field < self.ctx.own_cities.len() {
            return false;
        }
        if self
            .ctx
            .own_cities
            .iter()
            .any(|c| self.local_defense(c) == 0)
        {
            return false;
        }
        let v = self.view();
        if !v.players.iter().any(|p| {
            p.alive && p.team != v.team && p.team != target && v.teams_at_war(target, p.team)
        }) {
            return false;
        }
        if v.units.iter().any(|u| {
            v.team_of(u.owner) == target
                && u.military
                && u.kind != UnitKind::Scout
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|c| u.pos.distance(c.pos) <= 6)
        }) {
            return false;
        }
        if self.ctx.own_cities.iter().any(|city| {
            self.ctx
                .enemy_units
                .iter()
                .filter(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && !u.owner.is_barbarian()
                        && u.pos.distance(city.pos) <= 4
                })
                .map(|u| crate::analysis::threat_power(v, u))
                .sum::<i32>()
                > self.local_defense(city)
        }) {
            return false;
        }
        let total = army(v, target);
        total > 0
            && v.cities
                .iter()
                .filter(|c| v.team_of(c.owner) == target)
                .filter(|c| {
                    self.ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(c.pos) <= 12)
                })
                .any(|city| {
                    let attackers: Vec<_> = self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| {
                            u.military && u.kind != UnitKind::Scout && u.pos.distance(city.pos) <= 8
                        })
                        .collect();
                    if attackers.is_empty() {
                        return false;
                    }
                    let local: i32 = attackers.iter().map(|u| unit_army_value(u)).sum();
                    let guards: i32 = v
                        .units
                        .iter()
                        .filter(|u| {
                            v.team_of(u.owner) == target
                                && u.military
                                && u.pos.distance(city.pos) <= 4
                        })
                        .map(unit_army_value)
                        .sum();
                    if local <= guards {
                        return false;
                    }
                    let seen_away: i32 = v
                        .units
                        .iter()
                        .filter(|u| {
                            v.team_of(u.owner) == target
                                && u.pos.distance(city.pos) >= AWAY_DISTANCE
                        })
                        .map(unit_army_value)
                        .sum();
                    let captured_front = self.opponents.is_some_and(|model| {
                        model
                            .threats
                            .fallen_fronts
                            .iter()
                            .any(|(captor, at, turn)| {
                                v.team_of(*captor) == target
                                    && v.turn.saturating_sub(*turn) <= 8
                                    && at.distance(city.pos) >= 6
                            })
                    });
                    // Public city-count transfers during its other war show an active campaign
                    // even when that army has left our sight. The local-danger gates above still
                    // refuse a declaration if it has come to our door.
                    let recent_campaign = self.opponents.is_some_and(|model| {
                        model.threats.records.iter().any(|(id, record)| {
                            v.team_of(*id) == target
                                && record.captures.last().is_some_and(|turn| {
                                    *turn > model.threats.seeded_turn.unwrap_or(0)
                                        && v.turn.saturating_sub(*turn) <= 12
                                })
                        })
                    });
                    seen_away * 2 >= total || captured_front || recent_campaign
                })
    }

    /// A reachable city left thinly held while the leader fights elsewhere is a
    /// concrete prize, even before a funded siege forecast exists. Price only our
    /// nearby field force and visible local defenders; the ordinary war costs still
    /// apply to the Contain candidate.
    pub(crate) fn busy_front_prize(&self, target: TeamId) -> i32 {
        if !self.busy_leader_front(target) {
            return 0;
        }
        let v = self.view();
        v.cities
            .iter()
            .filter(|city| v.team_of(city.owner) == target)
            .filter_map(|city| {
                let force: Vec<_> = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military && u.kind != UnitKind::Scout && u.pos.distance(city.pos) <= 8
                    })
                    .collect();
                let ours: i32 = force.iter().map(|u| unit_army_value(u)).sum();
                let visible: i32 = v
                    .units
                    .iter()
                    .filter(|u| {
                        v.team_of(u.owner) == target && u.military && u.pos.distance(city.pos) <= 4
                    })
                    .map(unit_army_value)
                    .sum();
                (!force.is_empty() && ours > visible)
                    .then_some(crate::production::city_keep_value(city) * force.len().min(4) as i32)
            })
            .max()
            .unwrap_or(0)
    }
    /// A new war needs a local army, including the defence its public treasury can
    /// buy in one turn (one purchase per public city). No unseen unit or private tech is read.
    pub(crate) fn reachable_war_force(&self, target: TeamId) -> bool {
        self.reachable_war_force_by(target, 0)
    }

    /// For a race-ending war, count units that can march to the known city before
    /// the race closes. Zero keeps the ordinary, near-the-border declaration gate.
    pub(crate) fn reachable_war_force_by(&self, target: TeamId, turns: i32) -> bool {
        if self.decisions.reachable_wars == 0 || self.difficulty != AiDifficulty::Hard {
            return true;
        }
        let v = self.view();
        let cities: Vec<_> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == target)
            .collect();
        if cities.is_empty() {
            return false;
        }
        let defenders: Vec<_> = v
            .players
            .iter()
            .filter(|p| p.alive && p.team == target)
            .collect();
        let purchases: Vec<_> = defenders
            .iter()
            .map(|p| {
                let era = p.era.saturating_sub(1);
                let seen = v
                    .units
                    .iter()
                    .filter(|u| {
                        u.owner == p.id
                            && u.military
                            && !data::is_aircraft(u.kind)
                            && data::unit_era_tier(u.kind) == era
                    })
                    .map(|u| u.kind)
                    .max_by_key(|k| data::unit_stats_for(p.faction, *k).strength);
                let kind = seen.or_else(|| {
                    UnitKind::ALL
                        .into_iter()
                        .filter(|k| {
                            crate::composition::is_melee_line(*k) && data::unit_era_tier(*k) <= era
                        })
                        .max_by_key(|k| {
                            (
                                data::unit_era_tier(*k),
                                data::unit_stats_for(p.faction, *k).strength,
                            )
                        })
                });
                let Some(kind) = kind else {
                    return (p.id, 0, 0, 0, false, 1);
                };
                let stats = v.rule_experiments.unit_stats_for(p.faction, kind);
                let cost = (crate::tuned(stats.cost)
                    * if p.faction == legatum_core::Faction::Switzerland {
                        data::SWISS_BUY_PERCENT
                    } else {
                        data::BUY_COST_PERCENT
                    }
                    / 100)
                    .max(1);
                let count = (p.gold.max(0) / cost).min(p.cities as i32);
                let value =
                    legatum_core::army::combat_value(stats.strength, stats.ranged, 100, 0, false);
                let known = cities.iter().filter(|c| c.owner == p.id).count() as u32;
                (
                    p.id,
                    count,
                    value,
                    2 * stats.moves.max(1),
                    p.cities > known,
                    cost,
                )
            })
            .collect();
        let seen_army: i32 = v
            .units
            .iter()
            .filter(|u| v.team_of(u.owner) == target)
            .map(unit_army_value)
            .sum();
        let public_cities = defenders
            .iter()
            .map(|p| p.cities as i32)
            .sum::<i32>()
            .max(1);
        let unseen_share =
            ((strength(v, target) - seen_army).max(0) + public_cities - 1) / public_cities;
        // At Peace, the live context cannot route through the target's territory. Judge
        // the route under the proposed declaration, without revealing any new terrain.
        let war_view = if v.teams_at_war(v.team, target) {
            None
        } else {
            let mut war = v.clone();
            let Some(relation) = war
                .diplomacy
                .relations
                .iter_mut()
                .find(|r| r.between(v.team, target))
            else {
                return false;
            };
            relation.state = legatum_core::diplomacy::RelationState::War;
            Some(war)
        };
        let war_ctx = war_view
            .as_ref()
            .map(|war| crate::analysis::Ctx::prospective(war, &self.ctx));
        let route = war_ctx.as_ref().unwrap_or(&self.ctx);
        cities.iter().any(|city| {
            let ours: Vec<_> = if turns > 0 {
                self.strike_force_within(war_view.as_ref().unwrap_or(v), city, turns)
                    .into_iter()
                    .map(|(u, _, _)| u)
                    .collect()
            } else {
                self.ctx
                    .own_units
                    .iter()
                    .copied()
                    .filter(|u| {
                        u.military
                            && u.kind != UnitKind::Scout
                            && !data::is_aircraft(u.kind)
                            && u.pos.distance(city.pos) <= 8
                            && (route.reachable(u.pos, city.pos)
                                || (2..=4).any(|ring| {
                                    city.pos
                                        .ring(ring)
                                        .into_iter()
                                        .any(|h| route.reachable(u.pos, h))
                                }))
                    })
                    .collect()
            };
            let local: i32 = ours.iter().map(|u| unit_army_value(u)).sum();
            let visible: i32 = v
                .units
                .iter()
                .filter(|u| {
                    v.team_of(u.owner) == target
                        && u.pos.distance(city.pos) <= 2 * u.max_moves.max(1)
                })
                .map(unit_army_value)
                .sum();
            let prospective: i32 = purchases
                .iter()
                .map(|(owner, count, value, radius, unknown, cost)| {
                    if turns > 0 {
                        // Nearby cities can each buy one unit this turn and march it in.
                        // Swiss Gold saved toward the public Reserve is itself two VP:
                        // spending it to meet this war denies the imminent finish.
                        let spendable = defenders.iter().find(|p| p.id == *owner).map_or(0, |p| {
                            if p.faction == legatum_core::Faction::Switzerland
                                && p.gold >= legatum_core::goals::SWISS_HOARD_GOLD - *cost
                            {
                                (p.gold - legatum_core::goals::SWISS_HOARD_GOLD).max(0)
                            } else {
                                p.gold
                            }
                        });
                        let nearby = cities
                            .iter()
                            .filter(|c| c.owner == *owner && c.pos.distance(city.pos) <= *radius)
                            .count() as i32;
                        return (spendable / *cost).min(nearby + i32::from(*unknown)) * *value;
                    }
                    let nearby = cities
                        .iter()
                        .filter(|c| c.owner == *owner && c.pos.distance(city.pos) <= *radius)
                        .count() as i32;
                    (*count).min((nearby + i32::from(*unknown)).max(1)) * *value
                })
                .sum();
            ours.len() >= 2 && local > visible + unseen_share + prospective
        })
    }

    fn runaway_gene(&self, bit: i32) -> bool {
        self.difficulty == AiDifficulty::Hard && self.decisions.runaway_coalition & bit != 0
    }

    /// The runaway leader as the coalition acts on it: the public read below, passed through a
    /// seeded draw whose percent is its score (no cutoff: a leader at 30 is acted on a third of
    /// the time, at 60 three times in five, and can't sit just under a line).
    pub(crate) fn runaway(&self) -> Option<Runaway> {
        let read = self.runaway_read()?;
        self.wary_draw(3, u64::from(read.team.0), read.score)
            .then_some(read)
    }

    /// The public read of the leader, if any: the strict VP leader (by VP) or a team counting
    /// down to victory, with its score on public signs. Never our own team.
    pub(crate) fn runaway_read(&self) -> Option<Runaway> {
        let v = self.view();
        let living: Vec<_> = v
            .standings
            .iter()
            .filter(|s| alive(v, s.team) && s.target > 0)
            .collect();
        let lead = living
            .iter()
            .max_by_key(|s| (s.vp, std::cmp::Reverse(s.team)))?;
        let team = lead.team;
        let best_other = living
            .iter()
            .filter(|s| s.team != team)
            .map(|s| s.vp)
            .max()
            .unwrap_or(0);
        let hold = lead.victory_hold.is_some();
        if team == v.team || (lead.vp <= best_other && !hold) {
            return None;
        }
        let vp_lead = lead.vp as i32 - best_other as i32;
        let next_army = living
            .iter()
            .filter(|s| s.team != team)
            .map(|s| army(v, s.team))
            .max()
            .unwrap_or(0);
        let army_percent = army(v, team) * 100 / next_army.max(1);
        let (recent_captures, eliminations) = self.opponents.map_or((0, 0), |m| {
            m.threats
                .scores
                .values()
                .filter(|t| v.team_of(t.rival) == team)
                .fold((0, 0), |(c, e), t| {
                    (c + t.recent_captures, e + t.eliminations)
                })
        });
        let score = runaway_score(vp_lead, army_percent, recent_captures, eliminations, hold);
        (score > 0).then_some(Runaway {
            team,
            score,
            vp_lead,
            army_percent,
            recent_captures,
            eliminations,
            hold,
        })
    }

    fn runaway_team(&self, bit: i32) -> Option<TeamId> {
        if !self.runaway_gene(bit) {
            return None;
        }
        self.runaway().map(|r| r.team)
    }

    /// Bit 1: no Peace with the runaway while it runs away, unless one of our cities is about
    /// to fall (`collapsing`). Stricter than the other Peace gates, which let any endangered
    /// home sue: in the Horde scenario that escape let India and the Celts sign Peace with the
    /// Mongols and freed the horde (Byzantine cities held 3.12 with this gate, 2.39 with it).
    pub(crate) fn runaway_refuses_peace(&self, against: TeamId) -> bool {
        self.runaway_team(1) == Some(against) && !self.collapsing()
    }

    /// Bit 1: what an Alliance with `other` is worth against the runaway: nothing unless
    /// `other` is at war with it; then 15 plus up to `ALLY_SLOW_WEIGHT` for the share of the
    /// runaway's public army the pair can bring (each side's army by its reach to the runaway's
    /// cities, `helper_reach_percent`).
    pub(crate) fn runaway_alliance_value(&self, other: TeamId) -> i32 {
        let Some(leader) = self.runaway_team(1) else {
            return 0;
        };
        let v = self.view();
        if other == leader
            || !v.teams_at_war(other, leader)
            || (!v.teams_at_war(v.team, leader)
                && !self.reachable_war_force(leader)
                && !self.busy_leader_front(leader))
        {
            return 0;
        }
        let brought = army(v, v.team) * self.helper_reach_percent(v.team, leader) / 100
            + army(v, other) * self.helper_reach_percent(other, leader) / 100;
        let slow = (brought * 100 / army(v, leader).max(1)).clamp(0, 100);
        15 + slow * ALLY_SLOW_WEIGHT / 100
    }

    /// Wariness (gene `runaway_coalition` bit 4; the designer, 2026-11-08): how much `observer`
    /// should fear `target` as the next conqueror, from public information only and hidden from
    /// the player. Cities it captured recently (1 to 2 barely register, 3 to 5 are a roll-over),
    /// empires it eliminated, a visible army against the observer's, and its VP lead, all scaled
    /// by how near it is to the observer (the next target is the one in its path).
    pub(crate) fn wariness(&self, observer: TeamId, target: TeamId) -> i32 {
        let v = self.view();
        if observer == target || !alive(v, observer) || !alive(v, target) {
            return 0;
        }
        let (captures, eliminations) = self.opponents.map_or((0, 0), |m| {
            m.threats
                .scores
                .values()
                .filter(|t| v.team_of(t.rival) == target)
                .fold((0u32, 0u32), |(c, e), t| {
                    (c + t.recent_captures, e + t.eliminations)
                })
        });
        let vp = |t: TeamId| {
            v.standings
                .iter()
                .find(|s| s.team == t)
                .map_or(0, |s| s.vp as i32)
        };
        wariness_score(
            captures,
            eliminations,
            army(v, target) * 100 / army(v, observer).max(1),
            vp(target) - vp(observer),
            self.helper_reach_percent(observer, target),
        )
    }

    /// The empire we are most wary of, with how much (any wariness above 0).
    pub(crate) fn top_wariness(&self) -> Option<(TeamId, i32)> {
        let v = self.view();
        v.players
            .iter()
            .filter(|p| p.alive && p.team != v.team)
            .map(|p| p.team)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|t| (t, self.wariness(v.team, t)))
            .filter(|(_, w)| *w > 0)
            .max_by_key(|(t, w)| (*w, std::cmp::Reverse(*t)))
    }

    /// A yes/no choice for one purpose, drawn from a seeded stream that holds for `HOLD` turns
    /// (no flicker, and nothing to game by sitting one point under a line): true with
    /// probability `percent`.
    fn wary_draw(&self, purpose: u64, key: u64, percent: i32) -> bool {
        #[cfg(test)]
        if FORCE_DRAWS.with(std::cell::Cell::get) {
            return percent > 0;
        }
        let v = self.view();
        let mut r = legatum_core::rng::Rng::derive(
            self.seed,
            &[
                0x3A21_0B10,
                purpose,
                u64::from(v.team.0),
                key,
                u64::from(v.turn / 8),
            ],
        );
        (r.range(0, 99) as i32) < percent
    }

    /// Bit 4 (a standing bloc before any war): what an Alliance with `other` is worth against
    /// the empire we are both wary of: the weaker of the two wariness scores, the share of that
    /// empire's army the pair can bring, and 10 for a partner next door. The choice to seek or
    /// accept it is a seeded draw at the weaker wariness as a percent. The Alliance is the pact:
    /// the threat attacks one member and the Alliance brings the rest in at once.
    pub(crate) fn runaway_bloc_value(&self, other: TeamId) -> i32 {
        if !self.runaway_gene(4) {
            return 0;
        }
        let v = self.view();
        let Some((threat, strength_w)) = self.bloc_threat(other) else {
            return 0;
        };
        let ready = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .count()
            > self.ctx.own_cities.len();
        let fall_warning =
            if ready && !v.teams_at_war(other, threat) && self.top_threat_team() == Some(threat) {
                self.ctx
                    .own_cities
                    .iter()
                    .map(|c| self.neighbour_fall_power(c))
                    .max()
                    .unwrap_or(0)
            } else {
                0
            };
        if !v.teams_at_war(v.team, threat)
            && !self.reachable_war_force(threat)
            && !self.busy_leader_front(threat)
            && fall_warning == 0
        {
            return 0;
        }
        if !self.wary_draw(1, u64::from(other.0) << 8 | u64::from(threat.0), strength_w) {
            return 0;
        }
        let brought = army(v, v.team) * self.helper_reach_percent(v.team, threat) / 100
            + army(v, other) * self.helper_reach_percent(other, threat) / 100;
        let slow = (brought * 100 / army(v, threat).max(1)).clamp(0, 100);
        let near = nearest_known(v, other).is_some_and(|d| d <= 8);
        strength_w * 40 / 100 + slow * 30 / 100 + 10 * i32::from(near) + fall_warning.min(50) / 2
    }

    /// The empire both we and `other` are wary of, and the weaker of the two scores.
    fn bloc_threat(&self, other: TeamId) -> Option<(TeamId, i32)> {
        let v = self.view();
        v.players
            .iter()
            .filter(|p| p.alive && p.team != v.team && p.team != other)
            .map(|p| p.team)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|t| (t, self.wariness(v.team, t).min(self.wariness(other, t))))
            .filter(|(_, w)| *w > 0)
            .max_by_key(|(t, w)| (*w, std::cmp::Reverse(*t)))
    }

    /// Bit 4: the bloc-size cost is waived for a bloc whose every member is wary of the same
    /// empire (no guard against many members: the designer, 2026-11-08, "dogpiles are fine").
    pub(crate) fn runaway_bloc_waives_size(&self, merged: &[TeamId]) -> bool {
        if !self.runaway_gene(4) {
            return false;
        }
        let v = self.view();
        v.players
            .iter()
            .filter(|p| p.alive && !merged.contains(&p.team))
            .map(|p| p.team)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|t| {
                (
                    t,
                    merged
                        .iter()
                        .map(|m| self.wariness(*m, t))
                        .min()
                        .unwrap_or(0),
                )
            })
            .filter(|(_, w)| *w > 0)
            .max_by_key(|(t, w)| (*w, std::cmp::Reverse(*t)))
            .is_some_and(|(t, w)| self.wary_draw(4, u64::from(t.0), w))
    }

    /// Bit 32: more home troops while wary, in proportion: one extra defender per city of ours
    /// within 10 tiles of the empire we fear most, drawn by wariness (seeded).
    pub(crate) fn wary_guard(&self) -> i32 {
        if !self.runaway_gene(32) {
            return 0;
        }
        let Some((threat, w)) = self.top_wariness() else {
            return 0;
        };
        if !self.wary_draw(2, u64::from(threat.0), w) {
            return 0;
        }
        let v = self.view();
        let theirs: Vec<_> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == threat)
            .map(|c| c.pos)
            .collect();
        self.ctx
            .own_cities
            .iter()
            .filter(|c| theirs.iter().any(|t| t.distance(c.pos) <= 10))
            .count() as i32
    }

    /// Bit 16 (coalition wars fight at once): the empire we are at war with, wary of, and that
    /// another empire not on its side is at war with too. Its war is a coalition's.
    fn coalition_enemy(&self) -> Option<TeamId> {
        if !self.runaway_gene(16) {
            return None;
        }
        let v = self.view();
        v.players
            .iter()
            .filter(|p| p.alive && p.team != v.team && v.teams_at_war(v.team, p.team))
            .map(|p| p.team)
            .filter(|t| {
                // All in at a chance that grows with the square of wariness: next to nothing
                // for a mild worry (the seat gate measured every war all-in at -3.6 pp).
                let w = self.wariness(v.team, *t);
                self.wary_draw(5, u64::from(t.0), w * w / 100)
            })
            .find(|t| {
                v.players.iter().any(|q| {
                    q.alive && q.team != *t && q.team != v.team && v.teams_at_war(q.team, *t)
                })
            })
    }

    /// Bit 1: joining a war on the runaway (an ally's call to arms, or the war an Alliance
    /// brings) is worth this much more.
    pub(crate) fn runaway_join_bonus(&self, enemy: TeamId) -> i32 {
        if self.runaway_team(1) == Some(enemy)
            && (self.view().teams_at_war(self.view().team, enemy)
                || self.reachable_war_force(enemy)
                || self.busy_leader_front(enemy))
        {
            40 + 80 * i32::from(self.busy_leader_front(enemy))
        } else {
            0
        }
    }

    /// Bit 2: start a second-front operation on the runaway when its main army is committed
    /// away from its cities. Called before the operation review, so the roster fills this turn.
    pub(crate) fn start_second_front(&mut self) {
        if !(self.runaway_gene(2) || self.runaway_gene(8) || self.runaway_gene(16))
            || self.opening.is_some()
        {
            return;
        }
        let v = self.view();
        if let Some(op) = &self.campaigns.operation {
            if op.running() {
                return;
            }
            if op
                .log
                .end_turn
                .is_some_and(|t| v.turn < t.saturating_add(FRONT_COOLDOWN))
            {
                return;
            }
        }
        let Some((against, target)) = self.second_front_target() else {
            return;
        };
        self.campaigns.operation = Some(Operation::new(
            OperationKind::Strike {
                against,
                target,
                buy: false,
            },
            v.turn,
            FRONT_TURNS,
            self.front_radius(),
        ));
    }

    /// How far from the target own units join the strike.
    pub(crate) fn front_radius(&self) -> i32 {
        if self.runaway_gene(8) || self.runaway_gene(16) {
            ALL_IN_RADIUS
        } else {
            FRONT_RADIUS
        }
    }

    /// The runaway's city a second front strikes now, if the moment is right: we are at war
    /// with the runaway (or allied with a team that is, and may declare), its main army is
    /// visibly committed `AWAY_DISTANCE`+ tiles from its known cities (at least half its public
    /// army), none of it stands near our cities, and one of its known cities within reach is
    /// weakly held against the force we have near it.
    pub(crate) fn second_front_target(&self) -> Option<(TeamId, CityId)> {
        self.second_front_why().ok()
    }

    /// `second_front_target` with the gate that refused it (diagnostics).
    ///
    /// Bit 8 (strike a hollow runaway): a runaway that counts down to victory or reads as
    /// hollow (a rich empire whose cities stand empty) needs no army to be away and no war
    /// already under way: any reachable weakly held city of it is a target, and the strike
    /// declares when assembled. Its army at our door and our own collapse still stop it.
    pub(crate) fn second_front_why(&self) -> Result<(TeamId, CityId), &'static str> {
        let v = self.view();
        let radius = self.front_radius();
        let runaway = self.runaway();
        let coalition = self.coalition_enemy();
        let leader = runaway
            .as_ref()
            .map(|r| r.team)
            .or(coalition)
            .ok_or("no runaway")?;
        // Strikes that need no army away: a hollow or counting-down runaway (bit 8), or any
        // empire a coalition is at war with (bit 16: no token wars, all in at once).
        let all_in = coalition == Some(leader);
        let hollow_strike = all_in
            || (self.runaway_gene(8)
                && runaway.as_ref().is_some_and(|r| {
                    r.hold
                        || crate::threat::hollow_bonus_from(v, leader, self.hollow_floor(leader))
                            > 0
                }));
        if !hollow_strike && !self.runaway_gene(2) {
            return Err("bit 2 off");
        }
        let at_war = v.teams_at_war(v.team, leader);
        let allied_front = !at_war
            && v.diplomacy
                .side(v.team)
                .into_iter()
                .any(|t| t != v.team && alive(v, t) && v.teams_at_war(t, leader))
            && crate::opportunity::legal_declaration(v, leader);
        let hollow_front = hollow_strike
            && !all_in
            && !at_war
            && !allied_front
            && crate::opportunity::legal_declaration(v, leader);
        if !(at_war || allied_front || hollow_front) {
            return Err("not at war with it, no allied front");
        }
        if !at_war && !self.reachable_war_force(leader) {
            return Err("unready leader war");
        }
        if self.collapsing() {
            return Err("collapsing");
        }
        let cities: Vec<&CityView> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == leader)
            .collect();
        if cities.is_empty() {
            return Err("no known city");
        }
        let units: Vec<&UnitView> = v
            .units
            .iter()
            .filter(|u| v.team_of(u.owner) == leader && unit_army_value(u) > 0)
            .collect();
        let total = army(v, leader);
        if total <= 0 {
            return Err("no public army");
        }
        // Its main army, measured from each candidate city: at least half its public army is
        // visibly `AWAY_DISTANCE`+ tiles from that city (busy elsewhere, often at a third
        // empire's cities, which may be near cities it captured there).
        let away_from = |at: legatum_core::Hex| -> i32 {
            units
                .iter()
                .filter(|u| u.pos.distance(at) >= AWAY_DISTANCE)
                .map(|u| unit_army_value(u))
                .sum()
        };
        if !hollow_strike && !cities.iter().any(|c| away_from(c.pos) * 2 >= total) {
            return Err("army not away");
        }
        // Its army at our door is no second front: it is the first.
        if units.iter().any(|u| {
            self.ctx
                .own_cities
                .iter()
                .any(|c| c.pos.distance(u.pos) <= 6)
        }) {
            return Err("army at our door");
        }
        let ours: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout && !data::is_aircraft(u.kind))
            .collect();
        cities
            .iter()
            .filter_map(|c| {
                let away = away_from(c.pos);
                if !hollow_strike && away * 2 < total {
                    return None;
                }
                // What the runaway can have here: its public army less what we see away from
                // this city, shared by its cities we cannot see into.
                let home = (total - away).max(0);
                // Reachable: the city itself, or (at Peace, its land closed to us until War)
                // a tile of the staging ring the strike assembles on.
                let reach = |o: &CityView| {
                    self.ctx.reachable(o.pos, c.pos)
                        || c.pos
                            .ring(3)
                            .into_iter()
                            .any(|h| self.ctx.reachable(o.pos, h))
                };
                if !self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|o| o.pos.distance(c.pos) <= radius && reach(o))
                {
                    return None;
                }
                let force: Vec<&&UnitView> = ours
                    .iter()
                    .filter(|u| u.pos.distance(c.pos) <= radius)
                    .collect();
                if force.len() < 2 {
                    return None;
                }
                let power: i32 = force
                    .iter()
                    .map(|u| crate::analysis::threat_power(v, u))
                    .sum();
                let defenders: i32 = if c.visible {
                    units
                        .iter()
                        .filter(|u| u.pos.distance(c.pos) <= 2)
                        .map(|u| crate::analysis::threat_power(v, u))
                        .sum()
                } else {
                    // Unseen: its share of the army at home, in the same units as threat power
                    // (a Strongest Army point is a twentieth of a strength-20 unit squared).
                    let unseen = cities.iter().filter(|c| !c.visible).count().max(1) as i32;
                    let share = home / unseen;
                    // value = s^2 / 20, so strength = sqrt(20 * value); a cheap integer root.
                    isqrt(20 * share)
                };
                let city = crate::analysis::city_power(v, c);
                let need = city * if c.walls { 250 } else { 150 } / 100 + defenders * 3 / 2;
                if power < need
                    || (!at_war
                        && self.decisions.reachable_wars > 0
                        && self.campaign_outlook(c).is_none())
                {
                    return None;
                }
                Some({
                    let distance = self
                        .ctx
                        .own_cities
                        .iter()
                        .map(|o| o.pos.distance(c.pos))
                        .min()
                        .unwrap_or(99);
                    ((need * 100 / power.max(1), distance, c.id), c.id)
                })
            })
            .min_by_key(|(key, _)| *key)
            .map(|(_, id)| (leader, id))
            .ok_or("no weak city in reach")
    }
}

fn isqrt(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use crate::operation::OperationOutcome;
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState, TreatyKind};
    use legatum_core::view::TeamStanding;
    use legatum_core::{Faction, Order, PlayerId};

    /// Fixture teams: we are team 0 (players 0 and 1), ENEMY is team 2, player 3 is team 3.
    const THIRD: PlayerId = PlayerId(3);

    fn relation(a: TeamId, b: TeamId, state: RelationState) -> Relation {
        // Relations are stored with the lower team first.
        Relation {
            a: a.min(b),
            b: a.max(b),
            state,
            effective_turn: 10,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: (state == RelationState::War).then_some(7),
            declarers: vec![],
        }
    }

    fn standings(v: &mut PlayerView, vps: [(u8, u32); 3]) {
        v.standings = vps
            .map(|(team, vp)| TeamStanding {
                team: TeamId(team),
                members: vec![PlayerId(team)],
                vp,
                target: 12,
                victory_hold: None,
            })
            .to_vec()
            .into();
    }

    fn armies(v: &mut PlayerView, values: [(PlayerId, i32); 3]) {
        crate::fixture::set_armies(v, &values);
    }

    /// ENEMY (team 2) leads 6 VP to 2 with the strongest army and twice our army. Its city is 8
    /// tiles east of ours; its army of four Spearmen is 10+ tiles from it, far to the south.
    fn runaway_view(at_war: bool) -> PlayerView {
        let mut v = ViewBuilder::new(40, 30)
            .faction(Faction::Sparta)
            .gold(1000)
            .city(1, ME, off(4, 5), 3)
            .city(2, ENEMY, off(12, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(5, 5))
            .unit(2, ME, UnitKind::Spearman, off(5, 6))
            .unit(3, ME, UnitKind::Spearman, off(4, 6))
            .unit(10, ENEMY, UnitKind::Spearman, off(12, 20))
            .unit(11, ENEMY, UnitKind::Spearman, off(13, 20))
            .unit(12, ENEMY, UnitKind::Spearman, off(12, 21))
            .unit(13, ENEMY, UnitKind::Spearman, off(13, 21))
            .build();
        v.diplomacy.legacy = false;
        standings(&mut v, [(0, 2), (2, 6), (3, 1)]);
        let soldier = unit_army_value(&v.units.iter().find(|u| u.id.0 == 10).unwrap().clone());
        armies(
            &mut v,
            [(ME, 3 * soldier), (ENEMY, 4 * soldier), (THIRD, soldier)],
        );
        let me = v.team;
        let leader = v.team_of(ENEMY);
        let third = v.team_of(THIRD);
        v.diplomacy.relations = vec![
            relation(
                me,
                leader,
                if at_war {
                    RelationState::War
                } else {
                    RelationState::Peace
                },
            ),
            relation(third, leader, RelationState::War),
            relation(me, third, RelationState::Peace),
        ];
        if !at_war {
            v.diplomacy.actions.push(DiplomaticAction {
                new_wars: Vec::new(),
                calls: Vec::new(),
                also_agree: Vec::new(),
                order: Order::DeclareWar {
                    against: leader,
                    breach: false,
                },
                effective_turn: v.turn,
                cost: 0,
                blocked_reason: None,
                upkeep: 0,
                betrayal: false,
            });
        }
        v
    }

    fn planner(v: &PlayerView, gene: i32) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
        p.difficulty = AiDifficulty::Hard;
        p.plan_routing = true;
        p.decisions.runaway_coalition = gene;
        FORCE_DRAWS.with(|f| f.set(true));
        p
    }

    #[test]
    fn public_gold_and_local_force_gate_a_new_leader_war() {
        let mut v = runaway_view(false);
        let leader = v.team_of(ENEMY);
        let enemy_city = v.cities.iter().find(|c| c.owner == ENEMY).unwrap().clone();
        for id in [90, 91] {
            let mut city = enemy_city.clone();
            city.id = CityId(id);
            v.cities.push(city);
        }
        v.players[ENEMY.0 as usize].cities = 3;
        v.players[ENEMY.0 as usize].gold = 0;
        assert!(
            planner(&v, 1).reachable_war_force(leader),
            "own={:?} enemy={:?} era={:?}",
            v.units
                .iter()
                .filter(|u| u.owner == ME)
                .map(|u| (u.pos, u.military, u.strength))
                .collect::<Vec<_>>(),
            v.cities
                .iter()
                .filter(|c| c.owner == ENEMY)
                .map(|c| c.pos)
                .collect::<Vec<_>>(),
            v.players[ENEMY.0 as usize].era
        );
        v.players[ENEMY.0 as usize].gold = 2_097;
        assert!(!planner(&v, 1).reachable_war_force(leader));
        assert_eq!(planner(&v, 1).runaway_join_bonus(leader), 120);
        let mut no_busy_front = planner(&v, 1);
        no_busy_front.decisions.busy_leader_front = 0;
        assert_eq!(no_busy_front.runaway_join_bonus(leader), 0);
        let me = v.team;
        let third = v.team_of(THIRD);
        for relation in &mut v.diplomacy.relations {
            if relation.between(me, third) {
                relation.state = RelationState::Alliance;
            }
        }
        assert_eq!(planner(&v, 3).second_front_why(), Err("unready leader war"));
        let mut off = planner(&v, 1);
        off.decisions.reachable_wars = 0;
        off.decisions.busy_leader_front = 0;
        assert!(off.reachable_war_force(leader));
        assert_eq!(off.runaway_join_bonus(leader), 40);
        v.players[ENEMY.0 as usize].gold = 0;
        assert_eq!(planner(&v, 1).runaway_join_bonus(leader), 120);
        let mut fog = v.clone();
        fog.cities
            .retain(|c| c.owner != ENEMY || c.id == enemy_city.id);
        fog.players[ENEMY.0 as usize].cities = 1;
        fog.units.retain(|u| u.owner != ENEMY);
        assert!(!planner(&fog, 1).reachable_war_force(leader));
        for unit in v.units.iter_mut().filter(|u| u.owner == ME) {
            unit.pos = legatum_core::Hex::new(0, 20);
        }
        assert!(!planner(&v, 1).reachable_war_force(leader));
    }

    #[test]
    fn marching_force_can_reach_a_race_city_without_opening_ordinary_wars() {
        let mut v = runaway_view(false);
        let leader = v.team_of(ENEMY);
        v.players[ENEMY.0 as usize].gold = 0;
        for unit in v.units.iter_mut().filter(|u| u.owner == ME) {
            unit.pos = legatum_core::Hex::new(0, 20);
        }
        let mut on = planner(&v, 1);
        on.decisions.reachable_wars = 1;
        on.decisions.leader_war = 1;
        on.decisions.busy_leader_front = 0;
        assert!(!on.reachable_war_force(leader));
        assert!(on.reachable_war_force_by(leader, 10));

        let mut off = planner(&v, 1);
        off.decisions.reachable_wars = 1;
        off.decisions.leader_war = 0;
        off.decisions.busy_leader_front = 0;
        assert!(!off.reachable_war_force(leader));
    }

    #[test]
    fn the_score_reads_public_signs() {
        assert_eq!(runaway_score(4, 100, 0, 0, false), 60);
        assert_eq!(runaway_score(2, 160, 0, 0, false), 45);
        assert_eq!(runaway_score(1, 100, 1, 0, false), 25);
        assert_eq!(runaway_score(1, 100, 0, 0, true), 75);
        assert_eq!(runaway_score(9, 900, 9, 9, false), 60 + 25 + 40 + 40);
    }

    #[test]
    fn a_vp_and_army_leader_is_a_runaway_and_a_close_race_is_not() {
        let v = runaway_view(true);
        let r = planner(&v, 3).runaway().expect("runaway");
        assert_eq!(r.team, v.team_of(ENEMY));
        assert_eq!(r.vp_lead, 4);
        assert!(r.army_percent > 100);
        let mut close = v.clone();
        standings(&mut close, [(0, 5), (2, 6), (3, 1)]);
        // No cutoff: a close race is the same read with a lower score (a lower chance).
        let lesser = planner(&close, 3).runaway_read().expect("read");
        assert!(lesser.score < r.score, "{} {}", lesser.score, r.score);
        // We are never our own runaway.
        let mut ours = v.clone();
        standings(&mut ours, [(0, 9), (2, 6), (3, 1)]);
        assert_eq!(planner(&ours, 3).runaway(), None);
    }

    #[test]
    fn no_peace_with_the_runaway_under_bit_one() {
        let v = runaway_view(true);
        let leader = v.team_of(ENEMY);
        assert!(planner(&v, 1).runaway_refuses_peace(leader));
        assert!(!planner(&v, 0).runaway_refuses_peace(leader));
        assert_eq!(
            planner(&v, 1).wants_peace_why(leader),
            (false, "runaway leader")
        );
    }

    #[test]
    fn an_alliance_with_the_runaways_enemy_is_worth_its_army() {
        let mut v = runaway_view(true);
        let third = v.team_of(THIRD);
        let order = Order::ProposeTreaty {
            to: third,
            kind: TreatyKind::Alliance,
        };
        let budget = crate::diplomacy::TreatyBudget::of(&v);
        let value = planner(&v, 1).runaway_alliance_value(third);
        assert!(value >= 15, "value {value}");
        assert_eq!(planner(&v, 0).runaway_alliance_value(third), 0);
        let on = planner(&v, 1).treaty_score(third, TreatyKind::Alliance, &order, &budget);
        let off = planner(&v, 0).treaty_score(third, TreatyKind::Alliance, &order, &budget);
        assert_eq!(on.unwrap() - off.unwrap(), value);
        // Not at war with the runaway: no coalition value.
        let leader = v.team_of(ENEMY);
        for r in &mut v.diplomacy.relations {
            if r.between(third, leader) {
                r.state = RelationState::Peace;
            }
        }
        assert_eq!(planner(&v, 1).runaway_alliance_value(third), 0);
    }

    #[test]
    fn a_busy_leader_opens_a_reachable_front_without_a_funded_siege() {
        let mut v = ViewBuilder {
            view: runaway_view(false),
        }
        .unit(4, ME, UnitKind::Spearman, off(5, 7))
        .build();
        let target = v.team_of(ENEMY);
        let p = planner(&v, 1);
        assert!(p.busy_leader_front(target));
        assert!(p.busy_front_prize(target) > 0);
        assert_eq!(p.runaway_join_bonus(target), 120);
        let mut thin = runaway_view(false);
        thin.units.retain(|u| u.owner != ME || u.id.0 == 1);
        thin.units.iter_mut().find(|u| u.id.0 == 1).unwrap().pos = off(9, 5);
        assert!(!planner(&thin, 1).busy_leader_front(target));
        let mut marginal = v.clone();
        standings(&mut marginal, [(0, 2), (2, 3), (3, 1)]);
        assert!(!planner(&marginal, 1).busy_leader_front(target));
        let mut off = planner(&v, 1);
        off.decisions.busy_leader_front = 0;
        assert!(!off.busy_leader_front(target));
        assert_eq!(off.busy_front_prize(target), 0);
        let third = v.team_of(THIRD);
        for relation in &mut v.diplomacy.relations {
            if relation.between(target, third) {
                relation.state = RelationState::Peace;
            }
        }
        assert!(!planner(&v, 1).busy_leader_front(target));
    }

    #[test]
    fn the_second_front_strikes_the_empty_city_behind_the_runaways_army() {
        let v = runaway_view(true);
        let mut p = planner(&v, 2);
        assert_eq!(p.second_front_target(), Some((v.team_of(ENEMY), CityId(2))));
        p.start_second_front();
        p.review_operation();
        p.update_roster();
        p.reconcile_campaign();
        p.choose_plan();
        p.note_operation_took();
        let op = p.campaigns.operation.clone().expect("operation");
        assert!(op.log.took, "{op:?}");
        assert_eq!(op.roster.len(), 3);
        p.plan_military();
        let target = off(12, 5);
        let marched = p
            .orders
            .iter()
            .filter(|o| match o {
                Order::MoveUnit { unit, to } => v
                    .units
                    .iter()
                    .find(|u| u.id == *unit)
                    .is_some_and(|u| to.distance(target) < u.pos.distance(target)),
                _ => false,
            })
            .count();
        assert_eq!(marched, 3);
        // Without bit 2 nothing starts.
        let mut off_p = planner(&v, 1);
        off_p.start_second_front();
        assert!(off_p.campaigns.operation.is_none());
    }

    #[test]
    fn an_ally_of_the_runaways_enemy_opens_the_front_by_declaring() {
        let mut v = runaway_view(false);
        let me = v.team;
        let third = v.team_of(THIRD);
        for r in &mut v.diplomacy.relations {
            if r.between(me, third) {
                r.state = RelationState::Alliance;
            }
        }
        let mut old = planner(&v, 2);
        old.decisions.reachable_wars = 0;
        assert_eq!(old.second_front_target().map(|(_, c)| c), Some(CityId(2)));
        assert_eq!(planner(&v, 2).second_front_target(), None);
        // At Peace with the runaway and allied with nobody at war with it: no front.
        let lone = runaway_view(false);
        assert_eq!(planner(&lone, 2).second_front_target(), None);
    }

    #[test]
    fn no_second_front_while_the_runaways_army_is_at_home_or_at_our_door() {
        let mut home = runaway_view(true);
        for u in home.units.iter_mut().filter(|u| u.owner == ENEMY) {
            u.pos = off(13 + (u.id.0 as i32 % 2), 6 + (u.id.0 as i32 / 12));
        }
        assert_eq!(planner(&home, 2).second_front_why(), Err("army not away"));
        let mut door = runaway_view(true);
        for u in door.units.iter_mut().filter(|u| u.owner == ENEMY) {
            u.pos = off(4, 10 + (u.id.0 as i32 % 3));
        }
        assert_eq!(
            planner(&door, 2).second_front_why(),
            Err("army at our door")
        );
    }

    #[test]
    fn a_second_front_ends_by_its_own_conditions_and_waits_before_the_next() {
        let v = runaway_view(true);
        let mut p = planner(&v, 2);
        let mut ended = Operation::new(
            OperationKind::Strike {
                against: v.team_of(ENEMY),
                target: CityId(2),
                buy: false,
            },
            v.turn - 6,
            15,
            FRONT_RADIUS,
        );
        ended.outcome = OperationOutcome::RosterLost;
        ended.log.end_turn = Some(v.turn - 2);
        p.campaigns.operation = Some(ended.clone());
        p.start_second_front();
        assert_eq!(p.campaigns.operation.as_ref(), Some(&ended));
        ended.log.end_turn = Some(v.turn - FRONT_COOLDOWN);
        p.campaigns.operation = Some(ended);
        p.start_second_front();
        assert!(p.campaigns.operation.as_ref().unwrap().running());
    }

    /// A third empire's soldier beside the runaway's city: it is "within reach".
    fn with_third_near(mut v: PlayerView) -> PlayerView {
        let mut u = v.units.iter().find(|u| u.id.0 == 10).unwrap().clone();
        u.id = legatum_core::UnitId(20);
        u.owner = THIRD;
        u.pos = off(14, 5);
        v.units.push(u);
        v
    }

    fn planner_seeded(v: &PlayerView, gene: i32, seed: u64) -> Planner<'_> {
        let mut p = planner(v, gene);
        FORCE_DRAWS.with(|f| f.set(false));
        p.seed = seed;
        p
    }

    #[test]
    fn wariness_reads_public_signs_scaled_by_nearness() {
        let v = with_third_near(runaway_view(false));
        let (me, third, leader) = (v.team, v.team_of(THIRD), v.team_of(ENEMY));
        let p = planner(&v, 4);
        assert!(p.wariness(me, leader) >= 20, "{}", p.wariness(me, leader));
        assert!(p.wariness(third, leader) >= 20);
        // Nobody fears the weakest, and nobody fears themselves.
        assert_eq!(p.wariness(leader, third), 0);
        assert_eq!(p.wariness(me, me), 0);
        // Far from the threat, the same signs count for less.
        let mut far = runaway_view(false);
        for c in far.cities.iter_mut().filter(|c| c.owner == ENEMY) {
            c.pos = off(12, 29);
        }
        assert!(planner(&far, 4).wariness(me, leader) < p.wariness(me, leader));
    }

    #[test]
    fn one_or_two_captures_barely_register_but_a_roll_over_does() {
        // Next door (near 100), equal armies, level on VP: the captures alone, as the percent
        // chance of any response (no cutoff).
        assert_eq!(wariness_score(0, 0, 100, 0, 100), 0);
        assert!(wariness_score(1, 0, 100, 0, 100) <= 3);
        assert!(wariness_score(2, 0, 100, 0, 100) <= 8);
        assert!(wariness_score(3, 0, 100, 0, 100) >= 30);
        assert!(
            wariness_score(3, 0, 100, 0, 0) <= 8,
            "far away it is not our worry"
        );
        // All-in chance is the square: two captures and an army twice ours, next door, is a
        // few percent; a roll-over with eliminations is most of the time.
        let chance = |w: i32| w * w / 100;
        assert!(chance(wariness_score(2, 0, 200, 0, 100)) < 10);
        assert!(chance(wariness_score(4, 1, 150, 2, 100)) > 50);
    }

    #[test]
    fn a_standing_bloc_is_a_seeded_draw_scaled_by_wariness() {
        let v = with_third_near(runaway_view(false));
        let third = v.team_of(THIRD);
        assert_eq!(planner(&v, 3).runaway_bloc_value(third), 0);
        let hits = (0..60)
            .filter(|seed| planner_seeded(&v, 4, *seed).runaway_bloc_value(third) > 0)
            .count();
        assert!(
            hits > 5 && hits < 55,
            "a smooth chance, not a switch: {hits}"
        );
        // No partner wary of the same empire: nothing to bloc against.
        let leader = v.team_of(ENEMY);
        assert_eq!(planner(&v, 4).runaway_bloc_value(leader), 0);
    }

    #[test]
    fn the_bloc_size_cost_is_waived_for_a_bloc_wary_of_the_same_empire() {
        let v = with_third_near(runaway_view(false));
        let (me, third, leader) = (v.team, v.team_of(THIRD), v.team_of(ENEMY));
        assert!(planner(&v, 4).runaway_bloc_waives_size(&[me, third]));
        assert!(!planner(&v, 3).runaway_bloc_waives_size(&[me, third]));
        // The leader itself is wary of nobody in the bloc.
        assert!(!planner(&v, 4).runaway_bloc_waives_size(&[me, third, leader]));
    }

    #[test]
    fn a_coalition_war_strikes_at_once_with_the_army_at_home() {
        let mut v = runaway_view(true);
        // Wary enough (a VP lead and an army four times ours) for the all-in line.
        let soldier = unit_army_value(&v.units.iter().find(|u| u.id.0 == 10).unwrap().clone());
        armies(
            &mut v,
            [(ME, 3 * soldier), (ENEMY, 12 * soldier), (THIRD, soldier)],
        );
        for u in v.units.iter_mut().filter(|u| u.owner == ENEMY) {
            u.pos = off(13 + (u.id.0 as i32 % 2), 6 + (u.id.0 as i32 / 12));
        }
        assert_eq!(planner(&v, 2).second_front_why(), Err("army not away"));
        let why = planner(&v, 16).second_front_why();
        assert_ne!(why, Err("army not away"));
        assert_ne!(why, Err("bit 2 off"));
        assert_ne!(why, Err("no runaway"));
    }

    #[test]
    fn a_runaway_counting_down_is_struck_with_its_army_home_and_no_war() {
        let mut v = runaway_view(false);
        let leader = v.team_of(ENEMY);
        // Its army stands at home, and nobody (the third empire included) is at war with it.
        for u in v.units.iter_mut().filter(|u| u.owner == ENEMY) {
            u.pos = off(13 + (u.id.0 as i32 % 2), 6 + (u.id.0 as i32 / 12));
        }
        let third = v.team_of(THIRD);
        for r in &mut v.diplomacy.relations {
            if r.between(third, leader) {
                r.state = RelationState::Peace;
            }
        }
        // Not counting down, not hollow: bit 8 leaves it alone.
        assert!(planner(&v, 8).second_front_target().is_none());
        let mut hold = v.clone();
        for s in hold.standings.iter_mut().filter(|s| s.team == leader) {
            s.victory_hold = Some(legatum_core::view::VictoryHold {
                turns_held: 1,
                turns_needed: 10,
            });
        }
        // Weakly held against our three Spearmen? The runaway's four stand beside the city, so
        // this is a gate on the defenders, not on the army-away test.
        let why = planner(&hold, 10).second_front_why();
        assert_ne!(why, Err("army not away"));
        assert_ne!(why, Err("not at war with it, no allied front"));
        // Bit 8 off: the ordinary refusal.
        assert_eq!(
            planner(&hold, 2).second_front_why(),
            Err("not at war with it, no allied front")
        );
        // Emptied of defenders, the legacy draw strikes; the new gate still requires a
        // reachable local army against the public army and treasury.
        hold.units.retain(|u| u.owner != ENEMY);
        assert!(planner(&hold, 8).second_front_target().is_none());
        let mut legacy = planner(&hold, 8);
        legacy.decisions.reachable_wars = 0;
        assert_eq!(
            legacy.second_front_target().map(|(_, c)| c),
            Some(CityId(2))
        );
    }
}
