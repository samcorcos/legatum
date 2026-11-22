//! Threat assessment: which rival is on the war path, from public events and our own sight.
//!
//! A human watching the scoreboard sees empires die, city counts move between rivals at war,
//! and who declared which war. Combined with the armies we can see near our borders, that is
//! enough to say "this player is snowballing". Nothing here reads hidden state: city counts,
//! life and relations are public in `PlayerView.players` and `diplomacy.relations`; armies
//! and cities come from our own fogged view. The tracker keeps the previous turn's public
//! snapshot so transfers can be attributed, and seeds itself from a single view (relations
//! with eliminated empires persist) so a loaded game is not blind to earlier conquests.
use crate::opponents::OpponentAssessment;
#[cfg(test)]
use legatum_core::Faction;
use legatum_core::diplomacy::RelationState;
use legatum_core::view::{PlayerView, UnitView};
use legatum_core::{CityId, Hex, PlayerId, TeamId, data};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A rival at or above this score is a top threat: the empire to wall against, ally against
/// and, when it is safe, fight on a second front.
pub const TOP_THREAT: i32 = 60;
/// A rival worth watching: cheap precautions (Walls' tech) only.
pub const WATCH_THREAT: i32 = 30;
/// Standard turns over which
/// captures and city growth count as recent momentum.
const RECENT_TURNS: i32 = 40;
const GROWTH_TURNS: i32 = 20;
/// Army sightings near our cities kept for the trend.
const ARMY_HISTORY: usize = 6;
/// Hex radius around our cities for "army seen near our borders".
pub const NEAR_RADIUS: i32 = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConquestRecord {
    /// Turns of inferred city captures (public city-count transfers between empires at war,
    /// or known cities seen changing owner).
    pub captures: Vec<u32>,
    /// Empires this rival eliminated (or most plausibly eliminated).
    pub eliminations: Vec<PlayerId>,
    /// (turn, public city count), oldest first, bounded.
    pub cities: VecDeque<(u32, u32)>,
    /// Visible military power near our cities, newest last, bounded.
    pub near_army: VecDeque<i32>,
}

/// Why a rival scores as a threat, for logs, telemetry and tests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RivalThreat {
    pub rival: PlayerId,
    pub score: i32,
    pub eliminations: u32,
    pub recent_captures: u32,
    pub captures: u32,
    pub wars_declared: u32,
    pub cities: u32,
    pub city_growth: i32,
    pub near_army: i32,
    pub near_army_trend: i32,
    /// Percent weight from distance and reachability to our cities.
    pub proximity: i32,
    /// Fighting someone other than us right now (a war it declared or is in).
    pub busy_elsewhere: bool,
    /// Public VP-race component of the score (absent in older saves).
    #[serde(default)]
    pub race: i32,
}

impl RivalThreat {
    /// Short human-readable reasons, strongest first.
    pub fn reasons(&self) -> Vec<String> {
        let mut r = Vec::new();
        if self.eliminations > 0 {
            r.push(format!("eliminated {}", self.eliminations));
        }
        if self.race > 0 {
            r.push(format!("VP race +{}", self.race));
        }
        if self.recent_captures > 0 {
            r.push(format!("captured {} recently", self.recent_captures));
        }
        if self.wars_declared > 0 {
            r.push(format!("declared {} wars", self.wars_declared));
        }
        if self.city_growth > 0 {
            r.push(format!("+{} cities", self.city_growth));
        }
        if self.near_army > 0 {
            r.push(format!(
                "army {} near us (trend {:+})",
                self.near_army, self.near_army_trend
            ));
        }
        r.push(format!("proximity {}%", self.proximity));
        r
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThreatTracker {
    observed_turn: Option<u32>,
    /// A fresh brain can see public eliminations but cannot date them. Its
    /// seeded conquest record is not a newly observed neighbour fall.
    pub(crate) seeded_turn: Option<u32>,
    last_cities: BTreeMap<PlayerId, u32>,
    last_alive: BTreeSet<PlayerId>,
    last_wars: BTreeSet<(TeamId, TeamId)>,
    /// Owner of each city when this brain last read its view.
    city_owner: BTreeMap<CityId, PlayerId>,
    pub records: BTreeMap<PlayerId, ConquestRecord>,
    /// Cities we saw change hands: (captor, turn). Freshly taken cities are weakly held.
    pub captured_cities: BTreeMap<CityId, (PlayerId, u32)>,
    /// Publicly observed falls of cities near our own, including the last city of an empire.
    /// Positions are remembered from our view, never from the full game state.
    pub fallen_fronts: Vec<(PlayerId, Hex, u32)>,
    /// Last known owner and position, for a neighbour eliminated outside current sight.
    known_city_fronts: BTreeMap<CityId, (PlayerId, Hex)>,
    /// First recent sighting of a rival's approach, with its latest sighting turn. Later
    /// movement toward us does not turn its scouted homeward frontier into a target west of us.
    pub(crate) approach_sightings: BTreeMap<PlayerId, (Hex, u32)>,
    /// Gene `snipe_read`: rivals that took one of our Settlers, for the rest of the game.
    /// Only our own view is read: a Settler we had last turn that is now a rival's unit in sight.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub settler_snipers: BTreeSet<PlayerId>,
    /// Our Settlers when this brain last read its view.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    own_settlers: BTreeSet<legatum_core::UnitId>,
    pub scores: BTreeMap<PlayerId, RivalThreat>,
    /// Gene `hollow_read`: teams read as hollow leaders, with the turn the read began. Once read
    /// a team stays hollow down to `HOLLOW_KEEP` (one bought defender must not end the read:
    /// the 8.9.15 Swiss went from 66 to 35 hollow the turn after the Mongols declared).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hollow_teams: BTreeMap<TeamId, u32>,
    /// The current top threat. Once named it stays until its score falls 20 below
    /// `TOP_THREAT`, so one lost sighting does not flip alliances and wars on and off.
    top_rival: Option<PlayerId>,
    /// Gene `runaway_read`, set by the brain before each observation (never saved).
    #[serde(skip)]
    pub runaway_read: i32,
    /// Gene `hollow_read`, set by the brain before each observation (never saved).
    #[serde(skip)]
    pub hollow_read: i32,
    /// Gene set by the brain before an observation; 0 restores the capped score.
    #[serde(skip)]
    pub army_threat_weight: i32,
    /// Reuse the city-readiness arrival horizon for border-army sightings.
    #[serde(skip)]
    pub threat_horizon_turns: i32,
}

/// Public signs that the strict VP leader `team` is running away economically (gene
/// `runaway_read`; playtest 8.9.2 Switzerland: 4 VP to 2 from T78, the 1,000-Gold goal live and
/// a Power lead that set decadence at 12 to 18%, yet no AI named him a threat until T132).
/// A live faction goal and the public Power lead over the average empire; 0 for anyone who is
/// not the strict leader.
///
/// Gene `hollow_read` (`hollow_floor`, 0 off): a rich empire whose cities stand empty is read
/// without a strict VP lead, as long as nobody is more than a tenth of the target ahead of it,
/// and its hollowness adds to the signs (`hollow_bonus_from` at that floor; 8.9.15 Swiss review
/// rec 2: one unit for eight cities and 3,600 Gold scored 30 to 44 at a VP tie).
pub(crate) fn runaway_signs(view: &PlayerView, team: TeamId, hollow_floor: i32) -> i32 {
    let progress = |t: TeamId| {
        view.standings
            .iter()
            .find(|s| s.team == t)
            .map_or(0, |s| (100 * s.vp / s.target.max(1)) as i32)
    };
    let lead = progress(team);
    let best_other = view
        .standings
        .iter()
        .filter(|s| s.team != team && s.target > 0)
        .map(|s| progress(s.team))
        .max()
        .unwrap_or(0);
    let strict = best_other < lead;
    let hollow = if hollow_floor > 0 && best_other <= lead + 10 {
        hollow_bonus_from(view, team, hollow_floor)
    } else {
        0
    };
    if (!strict || lead == 0) && hollow == 0 {
        return 0;
    }
    economic_signs(view, team) + hollow
}

/// A live faction goal and a quarter of the public Power lead.
fn economic_signs(view: &PlayerView, team: TeamId) -> i32 {
    view.players
        .iter()
        .filter(|p| p.alive && p.team == team)
        .map(|p| {
            let goal = p
                .vp_sources
                .iter()
                .any(|(s, vp)| *vp > 0 && *s == legatum_core::VpSource::FactionGoal { of: p.id });
            i32::from(goal) * 12 + p.lead_percent.clamp(0, 100) / 4
        })
        .max()
        .unwrap_or(0)
}

/// How empty `team`'s cities are, 0..100: its public army strength per city against the median of every other living empire's, 100 for no army at all, 0 at
/// the median or above. Public numbers only (pillar 6).
pub fn hollowness(view: &PlayerView, team: TeamId) -> i32 {
    let value = |id: PlayerId| i64::from(view.army_of(id).max(0));
    let (mut own, mut cities) = (0i64, 0i64);
    let mut others: Vec<i64> = Vec::new();
    for p in view.players.iter().filter(|p| p.alive && p.cities > 0) {
        if p.team == team {
            own += value(p.id);
            cities += i64::from(p.cities);
        } else {
            others.push(value(p.id) * 100 / i64::from(p.cities));
        }
    }
    if cities == 0 || others.is_empty() {
        return 0;
    }
    others.sort_unstable();
    let median = others[others.len() / 2];
    if median <= 0 {
        return 0;
    }
    (100 - own * 100 * 100 / cities / median).clamp(0, 100) as i32
}

/// Gene `hollow_read`: the hollow-leader term. A team whose army per city is at most half the
/// median empire's and that shows riches in public (a live goal, a Power lead, an era
/// ahead of the median empire, or a Swiss capture bounty on a city we see) scores its
/// hollowness weighted by its cities (up to ten): one unit for eight cities reads about 25.
pub fn hollow_bonus(view: &PlayerView, team: TeamId) -> i32 {
    hollow_bonus_from(view, team, HOLLOW_ENTER)
}

/// Hollowness at which a team starts to read hollow, and down to which it keeps reading so.
pub const HOLLOW_ENTER: i32 = 50;
pub const HOLLOW_KEEP: i32 = 25;

/// `hollow_bonus` with the hollowness floor `floor` (`HOLLOW_KEEP` for a team already read).
pub fn hollow_bonus_from(view: &PlayerView, team: TeamId, floor: i32) -> i32 {
    let hollow = hollowness(view, team);
    if hollow < floor {
        return 0;
    }
    let members: Vec<_> = view
        .players
        .iter()
        .filter(|p| p.alive && p.team == team)
        .collect();
    let cities: i32 = members.iter().map(|p| p.cities as i32).sum();
    if cities < 2 {
        return 0;
    }
    let mut eras: Vec<u8> = view
        .players
        .iter()
        .filter(|p| p.alive && p.team != team)
        .map(|p| p.era)
        .collect();
    eras.sort_unstable();
    let median_era = eras.get(eras.len() / 2).copied().unwrap_or(0);
    let era_lead = members.iter().any(|p| p.era > median_era);
    let bounty: i32 = view
        .cities
        .iter()
        .filter(|c| view.team_of(c.owner) == team)
        .filter_map(|c| c.capture_bounty)
        .sum();
    let rich = economic_signs(view, team) >= 8 || era_lead || bounty >= 500;
    if !rich {
        return 0;
    }
    hollow * cities.min(10) / 30
}

fn pair(a: TeamId, b: TeamId) -> (TeamId, TeamId) {
    if a <= b { (a, b) } else { (b, a) }
}

fn wars(view: &PlayerView) -> BTreeSet<(TeamId, TeamId)> {
    view.diplomacy
        .relations
        .iter()
        .filter(|r| r.state == RelationState::War)
        .map(|r| pair(r.a, r.b))
        .collect()
}

pub(crate) fn unit_power(unit: &UnitView) -> i32 {
    crate::supply::power(unit)
}

impl ThreatTracker {
    /// Idempotent per turn. `rivals` supplies the slower assessment's known land distances
    /// and declared-war counts.
    pub fn observe(&mut self, view: &PlayerView, rivals: &BTreeMap<PlayerId, OpponentAssessment>) {
        if self.observed_turn.is_some_and(|t| t >= view.turn) {
            return;
        }
        for id in &self.own_settlers {
            if let Some(taker) = view.units.iter().find(|u| {
                u.id == *id
                    && u.owner != view.me
                    && !u.owner.is_barbarian()
                    && !view.same_team(view.me, u.owner)
            }) {
                self.settler_snipers.insert(taker.owner);
            }
        }
        self.own_settlers = view
            .units
            .iter()
            .filter(|u| u.owner == view.me && u.kind == legatum_core::UnitKind::Settler)
            .map(|u| u.id)
            .collect();
        let first = self.observed_turn.is_none();
        let now_wars = wars(view);
        let alive: BTreeSet<_> = view
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.id)
            .collect();
        if first {
            self.seeded_turn = Some(view.turn);
            self.seed(view, &now_wars);
        } else {
            self.attribute_transfers(view, &now_wars, &alive);
        }
        let own_positions: Vec<_> = view
            .cities
            .iter()
            .filter(|c| c.owner == view.me)
            .map(|c| c.pos)
            .collect();
        let distance = |at: Hex| {
            own_positions
                .iter()
                .map(|h| h.distance(at))
                .min()
                .unwrap_or(99)
        };
        let mut sightings: BTreeMap<PlayerId, Hex> = BTreeMap::new();
        for u in view
            .units
            .iter()
            .filter(|u| u.owner != view.me && !u.owner.is_barbarian() && u.military)
        {
            sightings
                .entry(u.owner)
                .and_modify(|at| {
                    if distance(u.pos) < distance(*at) {
                        *at = u.pos;
                    }
                })
                .or_insert(u.pos);
        }
        for (id, at) in sightings {
            let sighting = self.approach_sightings.entry(id).or_insert((at, view.turn));
            if view.turn.saturating_sub(sighting.1) > 20 {
                sighting.0 = at;
            }
            sighting.1 = view.turn;
        }
        self.approach_sightings
            .retain(|_, (_, turn)| view.turn.saturating_sub(*turn) <= 20);
        let near_ours = |at: Hex| own_positions.iter().any(|h| h.distance(at) <= 18);
        // Known cities seen changing owner: the strongest evidence of a capture.
        for city in &view.cities {
            if let Some(old) = self.city_owner.get(&city.id).copied()
                && old != city.owner
                && !city.owner.is_barbarian()
            {
                self.captured_cities
                    .insert(city.id, (city.owner, view.turn));
                if old != view.me && near_ours(city.pos) {
                    self.fallen_fronts.push((city.owner, city.pos, view.turn));
                }
            }
            self.city_owner.insert(city.id, city.owner);
        }
        if !first {
            for dead in view
                .players
                .iter()
                .filter(|p| self.last_alive.contains(&p.id) && !p.alive && p.id != view.me)
            {
                for killer in Self::war_partners(view, &now_wars, dead.team) {
                    for (owner, at) in self.known_city_fronts.values() {
                        if *owner == dead.id
                            && near_ours(*at)
                            && !self.fallen_fronts.iter().any(|(p, pos, turn)| {
                                *p == killer && *pos == *at && *turn == view.turn
                            })
                        {
                            self.fallen_fronts.push((killer, *at, view.turn));
                        }
                    }
                }
            }
        }
        self.known_city_fronts = view
            .cities
            .iter()
            .map(|c| (c.id, (c.owner, c.pos)))
            .collect();
        self.fallen_fronts
            .retain(|(_, _, turn)| view.turn.saturating_sub(*turn) <= RECENT_TURNS as u32);
        let horizon = RECENT_TURNS as u32;
        self.captured_cities
            .retain(|_, (_, turn)| view.turn.saturating_sub(*turn) <= horizon);
        for p in view.players.iter().filter(|p| p.alive) {
            let record = self.records.entry(p.id).or_default();
            if record.cities.back().is_none_or(|(t, _)| *t < view.turn) {
                record.cities.push_back((view.turn, p.cities));
            }
            let keep = GROWTH_TURNS as u32;
            while record
                .cities
                .front()
                .is_some_and(|(t, _)| view.turn.saturating_sub(*t) > keep)
            {
                record.cities.pop_front();
            }
        }
        // Visible armies near our cities, every turn (the slower assessment is every 5).
        let own: Vec<_> = view
            .cities
            .iter()
            .filter(|c| c.owner == view.me)
            .map(|c| c.pos)
            .collect();
        for p in view
            .players
            .iter()
            .filter(|p| p.alive && p.team != view.team)
        {
            let power: i32 = view
                .units
                .iter()
                .filter(|u| {
                    u.owner == p.id
                        && u.military
                        && own.iter().any(|h| {
                            if self.threat_horizon_turns > 0 {
                                crate::hard_defense::declaration_arrival_turns(
                                    view,
                                    u,
                                    *h,
                                    self.threat_horizon_turns as u32,
                                )
                                .is_some()
                            } else {
                                h.distance(u.pos) <= NEAR_RADIUS
                            }
                        })
                })
                .map(unit_power)
                .sum();
            let record = self.records.entry(p.id).or_default();
            record.near_army.push_back(power);
            while record.near_army.len() > ARMY_HISTORY {
                record.near_army.pop_front();
            }
        }
        self.last_cities = view.players.iter().map(|p| (p.id, p.cities)).collect();
        self.last_alive = alive;
        self.last_wars = now_wars;
        self.observed_turn = Some(view.turn);
        if self.hollow_read > 0 {
            self.read_hollow(view);
        }
        self.score(view, rivals);
    }

    /// A single view still shows dead empires and the wars that killed them.
    fn seed(&mut self, view: &PlayerView, wars: &BTreeSet<(TeamId, TeamId)>) {
        for dead in view.players.iter().filter(|p| !p.alive) {
            let killers = Self::war_partners(view, wars, dead.team);
            for killer in killers {
                let record = self.records.entry(killer).or_default();
                record.eliminations.push(dead.id);
                record.captures.push(view.turn);
            }
        }
    }

    /// Alive non-barbarian players at war with `team`, preferring those that declared it.
    fn war_partners(
        view: &PlayerView,
        wars: &BTreeSet<(TeamId, TeamId)>,
        team: TeamId,
    ) -> Vec<PlayerId> {
        let partners: Vec<TeamId> = wars
            .iter()
            .filter_map(|(a, b)| {
                if *a == team {
                    Some(*b)
                } else if *b == team {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect();
        let declarers: Vec<TeamId> = view
            .diplomacy
            .relations
            .iter()
            .filter(|r| r.state == RelationState::War && (r.a == team || r.b == team))
            .flat_map(|r| r.declarers.iter().copied())
            .filter(|t| *t != team)
            .collect();
        let chosen: Vec<TeamId> = if partners.iter().any(|t| declarers.contains(t)) {
            partners
                .into_iter()
                .filter(|t| declarers.contains(t))
                .collect()
        } else {
            partners
        };
        view.players
            .iter()
            .filter(|p| p.alive && chosen.contains(&p.team))
            .map(|p| p.id)
            .collect()
    }

    /// Public city counts move between empires at war: credit the gainers.
    fn attribute_transfers(
        &mut self,
        view: &PlayerView,
        now_wars: &BTreeSet<(TeamId, TeamId)>,
        alive: &BTreeSet<PlayerId>,
    ) {
        let delta = |p: &legatum_core::view::PlayerPublic| {
            p.cities as i32 - self.last_cities.get(&p.id).copied().unwrap_or(p.cities) as i32
        };
        let mut gains: BTreeMap<PlayerId, i32> = view
            .players
            .iter()
            .filter(|p| p.alive && delta(p) > 0)
            .map(|p| (p.id, delta(p)))
            .collect();
        let at_war = |a: TeamId, b: TeamId| {
            self.last_wars.contains(&pair(a, b)) || now_wars.contains(&pair(a, b))
        };
        for loser in view.players.iter() {
            let lost = -delta(loser);
            let died = self.last_alive.contains(&loser.id) && !alive.contains(&loser.id);
            if lost <= 0 && !died {
                continue;
            }
            let mut remaining = lost.max(0);
            let mut credited = Vec::new();
            for (gainer, gain) in gains.iter_mut() {
                if remaining == 0 {
                    break;
                }
                let g = view.team_of(*gainer);
                if g == loser.team || !at_war(g, loser.team) || *gain <= 0 {
                    continue;
                }
                let n = remaining.min(*gain);
                *gain -= n;
                remaining -= n;
                credited.push(*gainer);
                let record = self.records.entry(*gainer).or_default();
                record
                    .captures
                    .extend(std::iter::repeat_n(view.turn, n as usize));
            }
            if died {
                // A razing (Celts) captor gains no city; fall back to the war partners.
                let killer = credited.first().copied().or_else(|| {
                    Self::war_partners(view, now_wars, loser.team)
                        .first()
                        .copied()
                });
                if let Some(killer) = killer {
                    let record = self.records.entry(killer).or_default();
                    record.eliminations.push(loser.id);
                    if credited.is_empty() {
                        record.captures.push(view.turn);
                    }
                }
            } else if remaining > 0 && credited.is_empty() {
                // Lost cities with no gainer (razed): credit a lone war partner that declared.
                let partners = Self::war_partners(view, now_wars, loser.team);
                if let [only] = partners.as_slice() {
                    let record = self.records.entry(*only).or_default();
                    record
                        .captures
                        .extend(std::iter::repeat_n(view.turn, remaining as usize));
                }
            }
        }
    }

    /// Gene `hollow_read`: enter the hollow read at `HOLLOW_ENTER`, leave it below `HOLLOW_KEEP`.
    fn read_hollow(&mut self, view: &PlayerView) {
        let teams: BTreeSet<TeamId> = view
            .players
            .iter()
            .filter(|p| p.alive && p.team != view.team)
            .map(|p| p.team)
            .collect();
        self.hollow_teams.retain(|t, _| teams.contains(t));
        for team in teams {
            if hollow_bonus_from(view, team, self.hollow_floor(team)) > 0 {
                self.hollow_teams.entry(team).or_insert(view.turn);
            } else {
                self.hollow_teams.remove(&team);
            }
        }
    }

    /// The hollowness `team` must show to read hollow: lower once it already does.
    pub fn hollow_floor(&self, team: TeamId) -> i32 {
        if self.hollow_teams.contains_key(&team) {
            HOLLOW_KEEP
        } else {
            HOLLOW_ENTER
        }
    }

    fn score(&mut self, view: &PlayerView, rivals: &BTreeMap<PlayerId, OpponentAssessment>) {
        self.scores.clear();
        let previous = self.top_rival.take();
        let mut counts: Vec<u32> = view
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.cities)
            .collect();
        counts.sort_unstable();
        let median = counts.get(counts.len() / 2).copied().unwrap_or(0);
        let recent = RECENT_TURNS as u32;
        let own: Vec<_> = view
            .cities
            .iter()
            .filter(|c| c.owner == view.me)
            .map(|c| c.pos)
            .collect();
        let own_army: i32 = view
            .units
            .iter()
            .filter(|u| u.owner == view.me && u.military)
            .map(unit_power)
            .sum();
        let wars_now = wars(view);
        // Two passes: fighting the conqueror everyone fears is not aggression toward us, so
        // wars declared on the first pass's top threat do not count on the second.
        for pass in 0..2 {
            let fought = if pass == 0 {
                None
            } else {
                self.top_rival = self.pick_top(previous);
                match self.top().map(|t| view.team_of(t.rival)) {
                    Some(team) => Some(team),
                    None => break,
                }
            };
            let against_fought = |r: &legatum_core::diplomacy::Relation, team: TeamId| {
                fought.is_some_and(|f| f != team && (r.a == f || r.b == f))
            };
            for p in view
                .players
                .iter()
                .filter(|p| p.alive && p.team != view.team)
            {
                let record = self.records.get(&p.id).cloned().unwrap_or_default();
                let recent_captures = record
                    .captures
                    .iter()
                    .filter(|t| view.turn.saturating_sub(**t) <= recent)
                    .count() as u32;
                let captures = record.captures.len() as u32;
                let eliminations = record.eliminations.len() as u32;
                let assessed = rivals.get(&p.id);
                let declared_now = |count_fought: bool| {
                    view.diplomacy
                        .relations
                        .iter()
                        .filter(|r| {
                            r.state == RelationState::War
                                && r.declarers == vec![p.team]
                                && (count_fought || !against_fought(r, p.team))
                        })
                        .count() as u32
                };
                let wars_declared = (assessed
                    .map_or(0, |a| a.declared_wars)
                    .max(declared_now(true)))
                .saturating_sub(declared_now(true) - declared_now(false));
                let city_growth = record
                    .cities
                    .front()
                    .map_or(0, |(_, c)| p.cities as i32 - *c as i32);
                let near_army = record.near_army.back().copied().unwrap_or(0);
                let near_army_trend =
                    near_army - record.near_army.front().copied().unwrap_or(near_army);
                // Distance: known land route from the slower assessment, else the nearest known
                // city or unit in straight hexes; never seen means far away.
                let straight = view
                    .cities
                    .iter()
                    .filter(|c| c.owner == p.id)
                    .map(|c| c.pos)
                    .chain(
                        view.units
                            .iter()
                            .filter(|u| u.owner == p.id && u.military)
                            .map(|u| u.pos),
                    )
                    .flat_map(|h| own.iter().map(move |o| o.distance(h)))
                    .min();
                // Its front: known cities of empires it is fighting. A siege four hexes
                // from our border is close even before we have seen its own cities.
                let front = view
                    .cities
                    .iter()
                    .filter(|c| {
                        c.owner != view.me
                            && !c.owner.is_barbarian()
                            && view.team_of(c.owner) != p.team
                            && wars_now.contains(&pair(p.team, view.team_of(c.owner)))
                    })
                    .flat_map(|c| own.iter().map(move |o| o.distance(c.pos) + 2))
                    .min();
                let distance = [
                    assessed.and_then(|a| a.nearest_distance).or(straight),
                    front,
                ]
                .into_iter()
                .flatten()
                .min();
                let signs = if self.runaway_read > 0 {
                    runaway_signs(
                        view,
                        p.team,
                        if self.hollow_read > 0 {
                            self.hollow_floor(p.team)
                        } else {
                            0
                        },
                    )
                } else {
                    0
                };
                // Gene `runaway_read`: the strict VP leader's race is everyone's problem, so it
                // is not discounted by distance (the Swiss leader stood 22 hexes from Sparta).
                let proximity = match distance {
                    _ if signs > 0 => 100,
                    Some(d) if d <= 8 => 100,
                    Some(d) if d <= 12 => 85,
                    Some(d) if d <= 18 => 65,
                    _ => 45,
                };
                let busy_elsewhere = wars_now.iter().any(|(a, b)| {
                    (*a == p.team && *b != view.team || *b == p.team && *a != view.team)
                        && view
                            .players
                            .iter()
                            .any(|o| o.alive && o.team != p.team && (o.team == *a || o.team == *b))
                });
                let progress = view
                    .standings
                    .iter()
                    .find(|s| s.team == p.team)
                    .map_or(0, |s| (100 * s.vp / s.target.max(1)) as i32);
                // Wars it started and is still fighting against living empires.
                let offensive = view
                    .diplomacy
                    .relations
                    .iter()
                    .filter(|r| {
                        r.state == RelationState::War
                            && r.declarers == vec![p.team]
                            && !against_fought(r, p.team)
                            && view.players.iter().any(|o| {
                                o.alive && o.team != p.team && (o.team == r.a || o.team == r.b)
                            })
                    })
                    .count() as i32;
                let conquest = 35 * eliminations.min(3) as i32
                    + 15 * recent_captures.min(6) as i32
                    + 5 * captures.saturating_sub(recent_captures).min(4) as i32
                    + 10 * wars_declared.min(4) as i32
                    + 10 * offensive.min(2);
                let momentum =
                    5 * city_growth.clamp(0, 5) + 4 * (p.cities as i32 - median as i32).clamp(0, 5);
                // The public VP race is global, not weighted by distance: a leader pulling
                // away is everyone's problem well before the coalition warning (60% with a
                // 15-point lead, or 80%).
                let runner_up = view
                    .standings
                    .iter()
                    .filter(|s| s.team != p.team && s.target > 0)
                    .map(|s| (100 * s.vp / s.target) as i32)
                    .max()
                    .unwrap_or(0);
                // Gene `runaway_read` starts the race at 30% of the target and adds the public
                // signs of an economic runaway.
                let start = if self.runaway_read > 0 { 30 } else { 40 };
                let race =
                    (progress - start).max(0) + (progress - runner_up).clamp(0, 30) / 2 + signs;
                // A team counting down to victory (public) is everyone's top threat, more so
                // as its clock runs (`hold.rs`).
                let race = race
                    + crate::hold::rival_hold_of(view, p.team)
                        .map_or(0, |h| 100 + h.elapsed_permille() / 10);
                // A visible army at our border matters relative to what we field ourselves.
                let army = if near_army > 0 && self.army_threat_weight > 0 {
                    let public_value = |id| view.army_of(id);
                    let own_army_value: i32 = view
                        .units
                        .iter()
                        .filter(|u| u.owner == view.me)
                        .map(crate::hold::army_value)
                        .sum();
                    let public_excess =
                        ((i64::from((public_value(p.id) - public_value(view.me)).max(0))
                            * i64::from(own_army.max(1)))
                            / i64::from(own_army_value.max(1)))
                        .min(i64::from(i32::MAX)) as i32;
                    let numerator = (near_army as i64 * 40
                        + near_army_trend.max(0) as i64 * 10
                        + public_excess as i64 * 10)
                        * self.army_threat_weight as i64;
                    (numerator / (100 * own_army.max(15) as i64)).clamp(0, i32::MAX as i64) as i32
                } else if near_army > 0 {
                    (near_army * 20 / own_army.max(1)).min(20)
                        + if near_army_trend > 0 { 5 } else { 0 }
                } else {
                    0
                };
                let score = (conquest + momentum) * proximity / 100 + army + race;
                self.scores.insert(
                    p.id,
                    RivalThreat {
                        rival: p.id,
                        score,
                        eliminations,
                        recent_captures,
                        captures,
                        wars_declared,
                        cities: p.cities,
                        city_growth,
                        near_army,
                        near_army_trend,
                        proximity,
                        busy_elsewhere,
                        race,
                    },
                );
            }
        }
        self.top_rival = self.pick_top(previous);
    }

    /// The highest-scoring rival at or above `TOP_THREAT` (or the previous top threat while it
    /// stays within 20 of it), ties to the lower id.
    pub fn top(&self) -> Option<&RivalThreat> {
        self.top_rival.and_then(|r| self.scores.get(&r))
    }

    fn pick_top(&self, previous: Option<PlayerId>) -> Option<PlayerId> {
        self.scores
            .values()
            .filter(|t| {
                t.score >= TOP_THREAT || Some(t.rival) == previous && t.score >= TOP_THREAT - 20
            })
            .max_by_key(|t| (t.score, std::cmp::Reverse(t.rival)))
            .map(|t| t.rival)
    }

    pub fn get(&self, rival: PlayerId) -> Option<&RivalThreat> {
        self.scores.get(&rival)
    }
}

/// Faction temperament toward opening a second front (TI4-style identity, not numbers
/// alone): Sparta relish the war, Switzerland and Germany prefer walls and allies.
#[cfg(test)]
pub fn second_front_temperament(faction: Faction) -> Option<i32> {
    Some(crate::DecisionParams::builtin_for(faction).second_front_temperament)
}

use crate::scripted::Planner;
use legatum_core::view::CityView;

impl Planner<'_> {
    /// Threat responses are Hard decisions: harder by deciding better, never by seeing more.
    pub(crate) fn threat_aware(&self) -> bool {
        self.difficulty == legatum_core::AiDifficulty::Hard
    }

    pub(crate) fn top_threat(&self) -> Option<&RivalThreat> {
        if !self.threat_aware() {
            return None;
        }
        self.opponents?.threats.top()
    }

    pub(crate) fn top_threat_team(&self) -> Option<TeamId> {
        self.top_threat().map(|t| self.view().team_of(t.rival))
    }

    /// A fallen neighbour is a warning before the conqueror's units enter our sight.
    /// Public army, the distance from that fall and its age set the size of the warning.
    /// A peace with every living opponent ends it; an army no longer near us lets it fade.
    pub(crate) fn neighbour_fall_power(&self, city: &CityView) -> i32 {
        self.fall_warning(city, false)
    }

    /// Gene `neighbour_alarm`: the warning production answers (unit positioning, Gold
    /// reserves and diplomacy keep `neighbour_fall_power`). It also starts from a public
    /// capture plus a sighting, grows with his pace, and does not hand over to live
    /// forecasts when the conqueror's army reaches our border:
    /// one-turn hold forecasts read a walled city under a multi-turn siege as safe until the
    /// turn it falls (Sparta 8.9.57, Gergovia T29 to T32), too late to build a defender.
    pub(crate) fn conqueror_alarm(&self, city: &CityView) -> i32 {
        if self.decisions.neighbour_alarm == 0 {
            return self.fall_warning(city, false);
        }
        self.fall_warning(city, true)
    }

    fn fall_warning(&self, city: &CityView, alarm: bool) -> i32 {
        if !self.hard_defense() || self.decisions.neighbour_fall == 0 {
            return 0;
        }
        let Some(model) = self.opponents else {
            return 0;
        };
        let v = self.view();
        // When the fallen city itself is unexplored, public city counts still identify a
        // capture between known war partners. Repeated captures (or an elimination) plus
        // a unit we actually saw near home give an approach without inventing the hidden
        // victim's city coordinate after unrelated public captures.
        let seen_fronts = model.threats.records.iter().filter_map(|(rival, record)| {
            let (at, seen) = model.threats.approach_sightings.get(rival)?;
            let turn = *record.captures.last()?;
            // Gene `neighbour_alarm`: a public capture with his army seen near us is enough;
            // waiting for an elimination missed Rome 8.9.71 (Kabul fell T26 and Egypt, whose
            // capital stands six hexes from it, never knew the city).
            ((!record.eliminations.is_empty() || alarm)
                && turn > model.threats.seeded_turn.unwrap_or(0)
                && v.turn.saturating_sub(*seen) <= 20
                && v.turn.saturating_sub(turn) <= 30)
                .then_some((*rival, *at, turn))
        });
        model
            .threats
            .fallen_fronts
            .iter()
            .copied()
            .chain(seen_fronts)
            .filter_map(|(rival, front, turn)| {
                if v.same_team(v.me, rival) || v.allied(v.me, rival) {
                    return None;
                }
                if self.top_threat_team() != Some(v.team_of(rival)) {
                    return None;
                }
                let t = model.threats.get(rival)?;
                let public = v.army_of(rival).max(0);
                let ours = v.army_of(v.me).max(0);
                if public.saturating_mul(2) < ours.saturating_mul(3) {
                    return None;
                }
                if front.distance(city.pos) > 18 {
                    return None;
                }
                // Once the conqueror's army reaches any of our cities, live hold
                // forecasts and tactical defence have better evidence than this
                // remembered approach. Do not pin the rest of our army at home.
                if !alarm
                    && v.units.iter().any(|u| {
                        v.team_of(u.owner) == v.team_of(rival)
                            && u.military
                            && u.kind != legatum_core::UnitKind::Scout
                            && self
                                .ctx
                                .own_cities
                                .iter()
                                .any(|home| u.pos.distance(home.pos) <= 6)
                    })
                {
                    return None;
                }
                // When most of the public army is actually visible marching away,
                // that observation supersedes a remembered capture near our border.
                let sighted: Vec<_> = v
                    .units
                    .iter()
                    .filter(|u| {
                        v.team_of(u.owner) == v.team_of(rival)
                            && u.military
                            && u.kind != legatum_core::UnitKind::Scout
                    })
                    .collect();
                let visible: i32 = sighted
                    .iter()
                    .map(|u| crate::runaway::unit_army_value(u))
                    .sum();
                if visible.saturating_mul(2) >= public
                    && sighted.iter().all(|u| {
                        self.ctx
                            .own_cities
                            .iter()
                            .all(|home| u.pos.distance(home.pos) > 10)
                    })
                {
                    return None;
                }
                // The last city can fall before any living war opponent remains. The public
                // declaration is still evidence until a peace is signed or the warning ages out.
                let fighting = v.diplomacy.relations.iter().any(|r| {
                    r.state == RelationState::War
                        && (r.a == v.team_of(rival) || r.b == v.team_of(rival))
                });
                if !fighting {
                    return None;
                }
                let age = v.turn.saturating_sub(turn) as i32;
                let near = t.near_army > 0;
                let lifetime = if near {
                    RECENT_TURNS
                } else {
                    RECENT_TURNS * 3 / 4
                };
                if age >= lifetime {
                    return None;
                }
                let distance = front.distance(city.pos);
                // Gene `neighbour_alarm`: the pace of his conquest is the evidence a person
                // reads first ("after athens, and then after celts... surely India knew").
                let strength = if alarm {
                    let pace = model
                        .threats
                        .records
                        .get(&rival)
                        .map_or(0, |r| {
                            r.captures
                                .iter()
                                .filter(|c| v.turn.saturating_sub(**c) <= 20)
                                .count()
                        })
                        .min(4) as i32;
                    (public / 2 * (2 + pace) / 2).clamp(0, 240)
                } else {
                    (public / 2).clamp(0, 120)
                };
                Some(
                    strength * (lifetime - age) / lifetime * (24 - distance) / 24
                        * self.decisions.neighbour_fall
                        / 100,
                )
            })
            .max()
            .unwrap_or(0)
    }

    /// How exposed one of our cities is to the top threat: 0 not, 1 exposed (its cities
    /// within ten reachable hexes, its army within six, or we are at war), 2 acute (its army
    /// is here and we are at war, or a dangerous conqueror's army stands at the border).
    pub(crate) fn threat_exposure(&self, c: &CityView) -> u8 {
        self.top_threat().map_or(0, |t| self.exposure_to(t, c, 10))
    }

    fn exposure_to(&self, t: &RivalThreat, c: &CityView, city_radius: i32) -> u8 {
        let v = self.view();
        let at_war = v.at_war(v.me, t.rival);
        let army_near = v.units.iter().any(|u| {
            u.owner == t.rival
                && u.military
                && u.pos.distance(c.pos) <= NEAR_RADIUS
                && self.ctx.reachable(u.pos, c.pos)
        });
        if army_near && (at_war || t.score >= TOP_THREAT + 30) {
            return 2;
        }
        // A known city of the threat this close is a danger even across unexplored land:
        // fog between us is not evidence of a sea (a human would prepare too).
        let city_near = v
            .cities
            .iter()
            .any(|e| e.owner == t.rival && e.pos.distance(c.pos) <= city_radius);
        u8::from(at_war || army_near || city_near)
    }

    /// A rival worth watching (`WATCH_THREAT`) within twelve reachable hexes of this city:
    /// Walls become a priority investment before it is the top threat.
    pub(crate) fn watched_exposure(&self, c: &CityView) -> bool {
        self.threat_aware()
            && self.opponents.is_some_and(|m| {
                m.threats
                    .scores
                    .values()
                    .filter(|t| t.score >= WATCH_THREAT)
                    .any(|t| self.exposure_to(t, c, 12) > 0)
            })
    }

    /// Extra army investment while a top threat can reach us, 0..=100 like conquest pressure.
    pub(crate) fn threat_pressure(&self) -> i32 {
        let Some(t) = self.top_threat() else {
            return 0;
        };
        if !self
            .ctx
            .own_cities
            .iter()
            .any(|c| self.threat_exposure(c) > 0)
        {
            return 0;
        }
        (t.score - TOP_THREAT / 2).clamp(0, 100)
    }

    /// A city the top threat took recently is weakly held (zero output, a thin garrison,
    /// resentful population): the natural objective of a second front.
    pub(crate) fn recently_captured_by_threat(&self, city: &CityView) -> bool {
        let Some(t) = self.top_threat() else {
            return false;
        };
        self.opponents.is_some_and(|m| {
            m.threats
                .captured_cities
                .get(&city.id)
                .is_some_and(|(captor, _)| *captor == t.rival && city.owner == t.rival)
        })
    }

    /// Expected value of opening a second front against the top threat now. Zero or more
    /// means worth declaring if a winnable campaign exists (diplomacy checks that too). Rewards a conqueror busy elsewhere,
    /// allies already fighting it and weakly held objectives; penalises its army at our
    /// door, our own thin defence and faction temperament (Celts never march out).
    pub(crate) fn second_front_value(&self, against: TeamId) -> Option<i32> {
        let t = self.top_threat()?;
        let v = self.view();
        if v.team_of(t.rival) != against {
            return None;
        }
        let temperament = self.decisions.second_front_temperament;
        // A war we cannot pay for ends in disbanded units and a gift of cities: the
        // treasury must cover the current deficit for ten Standard turns.
        let deficit = (-v.economy.gold_per_turn.total).max(0);
        if v.economy.bankruptcy.is_some() || v.economy.gold < deficit * 10 {
            return None;
        }
        let strength = |team| crate::diplomacy::strength(v, team);
        let ours = strength(v.team);
        let theirs = strength(against).max(1);
        let allies: i32 = v
            .standings
            .iter()
            .filter(|s| {
                s.team != v.team
                    && s.team != against
                    && v.teams_at_war(s.team, against)
                    && v.players.iter().any(|p| p.team == s.team && p.alive)
            })
            .map(|s| strength(s.team))
            .sum();
        let ratio = (ours + allies) * 100 / theirs;
        // Declaring war opens our borders to it. Together with those already fighting it we
        // must not be badly outweighed, and alone we must not be a pushover unless allies or
        // its other wars hold its attention. Its distraction and our allies raise the value.
        let engaged = t.busy_elsewhere || allies > 0;
        if ratio < 80 || ours * 2 < theirs && !engaged {
            return None;
        }
        // Its visible army near our cities, against our own home defence.
        let threat_here: i32 = self
            .ctx
            .enemy_units
            .iter()
            .filter(|u| {
                u.owner == t.rival
                    && u.military
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|c| c.pos.distance(u.pos) <= NEAR_RADIUS + 2)
            })
            .map(|u| crate::analysis::threat_power(v, u))
            .sum();
        let home: i32 = self
            .ctx
            .own_cities
            .iter()
            .map(|c| self.local_defense(c) + crate::analysis::city_power(v, c) / 2)
            .sum();
        if threat_here * 100 > home * 120 {
            // Declaring now invites its main army straight into us.
            return None;
        }
        // Only its cities we could march on count: known, within twelve hexes of one of ours
        // and on the same landmass. A distant VP leader with no city in reach is not a front.
        let in_reach: Vec<&CityView> = v
            .cities
            .iter()
            .filter(|c| c.owner == t.rival)
            .filter(|c| {
                self.ctx
                    .own_cities
                    .iter()
                    .any(|o| o.pos.distance(c.pos) <= 12 && self.ctx.reachable(o.pos, c.pos))
            })
            .collect();
        if in_reach.is_empty() {
            return None;
        }
        let soft = in_reach
            .iter()
            .filter(|c| {
                self.recently_captured_by_threat(c) || c.visible && !c.walls && c.hp < c.max_hp
            })
            .count() as i32;
        // The old soft-city test made a fully walled leader immune to a second front.
        // Keep it as an A/B baseline, but normally price the least costly reachable siege.
        if self.decisions.second_front_siege_price == 0
            && soft == 0
            && allies == 0
            && temperament < 25
        {
            return None;
        }
        let siege_cost = if self.decisions.second_front_siege_price > 0 && soft == 0 {
            in_reach
                .iter()
                .map(|c| {
                    let outlook = self.campaign_outlook(c);
                    // The forecast can be unavailable before the army is built. Its absence
                    // raises the price; it must not erase the leader-denial opportunity.
                    let preparation = outlook.map_or(15, |o| 15 - o.odds / 5);
                    i32::from(c.walls) * 10 + c.hp * 10 / c.max_hp.max(1) + preparation.max(0)
                })
                .min()
                .unwrap_or(0)
        } else {
            0
        };
        let leader_denial = if self.decisions.second_front_siege_price > 0 {
            v.standings
                .iter()
                .find(|s| s.team == against)
                .map_or(0, |s| {
                    2 * (s.vp * 100 / s.target.max(1)).saturating_sub(30) as i32
                })
        } else {
            0
        };
        Some(
            temperament
                + (t.score - TOP_THREAT).min(60) / 2
                + if t.busy_elsewhere { 25 } else { -15 }
                + if allies > 0 { 20 } else { 0 }
                + 10 * soft.min(3)
                + (ratio - 100).clamp(-60, 60) / 2
                - threat_here * 30 / home.max(1)
                + leader_denial
                - siege_cost,
        )
    }
}

impl<'a> Planner<'a> {
    /// Switch research to Walls' tech when a conqueror can reach an unwalled city. Stored
    /// science is one pool, so switching loses nothing. Returns whether research is set.
    pub(crate) fn threat_research(&mut self) -> bool {
        if self.faction == legatum_core::Faction::Mongols {
            return false;
        }
        // Walls' tech is cheap: research it once any rival worth watching can reach an
        // unwalled city, before it becomes the top threat.
        let watched = self
            .ctx
            .own_cities
            .iter()
            .any(|c| !c.walls && self.watched_exposure(c));
        if self
            .orders
            .iter()
            .any(|o| matches!(o, legatum_core::Order::SetResearch { .. }))
            || !watched
        {
            return false;
        }
        let walls = data::building_stats(legatum_core::BuildingKind::Walls);
        let Some(tech) = walls.requires else {
            return false;
        };
        let r = &self.view().research;
        if r.known.contains(&tech) {
            return false;
        }
        let mut needed = BTreeSet::new();
        crate::investment::prerequisites(tech, &r.known, &mut needed);
        needed.insert(tech);
        if r.current.is_some_and(|t| needed.contains(&t)) {
            return true;
        }
        let Some(next) = r
            .available
            .iter()
            .map(|o| o.tech)
            .find(|t| needed.contains(t))
        else {
            return false;
        };
        self.orders
            .push(legatum_core::Order::SetResearch { tech: next });
        true
    }

    /// Gene `walls_before_shooter`: the city's Walls would finish before the nearest hostile
    /// army could reach it, so they come before the shooter (user, 2026-10-17: seeing Sparta
    /// next door, "the correct reaction is '[uh oh]' and then build walls and archers to defend
    /// yourself as soon as possible"; playtest 8.9.2: walled Tara cost him four Hoplites in two
    /// turns, while every unwalled city with an Archer fell). Public information only: visible
    /// armies by their moves, else the rival's nearest known city plus time to train a unit.
    /// Gene `aztecs_playbook` bit `AZTEC_WALLS` (the designer's Aztecs walled every new city
    /// first, opening included): only a visible army sets the deadline, from the first turn.
    pub(crate) fn walls_beat_arrival(&self, c: &CityView) -> bool {
        use legatum_core::{BuildingKind, ProductionItem};
        let aztec_walls = self.aztec_walls_playbook();
        if !(self.hard_defense() || aztec_walls) || self.decisions.walls_before_shooter == 0 {
            return false;
        }
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let Some(turns) = c
            .detail
            .as_ref()
            .and_then(|d| d.options.iter().find(|o| o.item == walls))
            .map(|o| o.turns)
            .filter(|t| *t > 0)
        else {
            return false;
        };
        if self.hard_defense()
            && self.decisions.siege_survival_clock > 0
            && let Some(fall) = self.defense_risk(c, None).production_deadline()
        {
            return turns < fall;
        }
        let v = self.view();
        let hostile = |owner: legatum_core::PlayerId| {
            !owner.is_barbarian()
                && !v.same_team(v.me, owner)
                && !v.diplomacy.relations.iter().any(|r| {
                    r.between(v.team, v.team_of(owner))
                        && r.state == legatum_core::diplomacy::RelationState::Alliance
                        && (self.decisions.ally_defense_percent == 0
                            || crate::diplomacy::strength(v, v.team_of(owner))
                                * self.decisions.ally_defense_percent
                                < crate::diplomacy::strength(v, v.team).max(1) * 150)
                })
        };
        let army = v
            .units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != legatum_core::UnitKind::Scout
                    && hostile(u.owner)
                    && u.pos.distance(c.pos) <= 12
            })
            .map(|u| (u.pos.distance(c.pos) + u.max_moves.max(1) - 1) / u.max_moves.max(1))
            .min();
        let eta = army.unwrap_or_else(|| {
            if aztec_walls {
                return 99;
            }
            v.cities
                .iter()
                .filter(|e| hostile(e.owner) && e.pos.distance(c.pos) <= 12)
                .map(|e| e.pos.distance(c.pos) / 2 + 3)
                .min()
                .unwrap_or(99)
        });
        turns <= eta
    }

    /// Every city a conqueror can reach gets Walls first when they can finish before arrival;
    /// otherwise it gets a garrisoned shooter before Walls. Stored production carries over
    /// when a build is switched, and the Walls-line always finishes with Production.
    pub(crate) fn plan_threat_defense(&mut self) {
        use legatum_core::{BuildingKind, Order, ProductionItem};
        if self.threat_aware() && self.decisions.army_door_production > 0 {
            let mut assigned = BTreeSet::new();
            for front in self.door_fronts() {
                let Some(at) = self
                    .ctx
                    .own_cities
                    .iter()
                    .copied()
                    .find(|c| c.id == front.city)
                else {
                    continue;
                };
                let mut gap = front.gap().max(0);
                if gap == 0 && self.defense_risk(at, None).falls {
                    gap = 1;
                }
                if gap == 0
                    || !self.view().units.iter().any(|u| {
                        u.military
                            && self.view().at_war(self.view().me, u.owner)
                            && u.pos.distance(at.pos) <= 6
                    })
                {
                    continue;
                }
                for order in &self.orders {
                    if let Order::BuyItem {
                        city,
                        item: ProductionItem::Unit { unit },
                    } = order
                        && *city == front.city
                    {
                        gap -= self.modernization_power(*unit, 0, data::UNIT_HP, false, at.pos);
                    }
                }
                let mut nearby: Vec<_> = self
                    .ctx
                    .own_cities
                    .iter()
                    .copied()
                    .filter(|c| c.pos.distance(at.pos) <= 4 && self.ctx.reachable(c.pos, at.pos))
                    .collect();
                nearby.sort_by_key(|c| (c.pos.distance(at.pos), c.id));
                for c in nearby {
                    if gap <= 0 {
                        break;
                    }
                    if assigned.contains(&c.id) {
                        continue;
                    }
                    let Some(d) = c.detail.as_ref().filter(|d| d.resistance == 0) else {
                        continue;
                    };
                    let best = d
                        .options
                        .iter()
                        .filter(|o| {
                            o.turns > 0
                                && o.resource_blocker.is_none()
                                && matches!(o.item, ProductionItem::Unit { unit }
                                    if crate::composition::is_melee_line(unit)
                                        || crate::composition::is_ranged_line(unit))
                        })
                        .max_by_key(|o| {
                            let ProductionItem::Unit { unit } = o.item else {
                                unreachable!()
                            };
                            (
                                data::unit_stats(unit)
                                    .requires
                                    .map(legatum_core::eras::era_of_tech)
                                    .unwrap_or(0),
                                self.modernization_power(unit, 0, data::UNIT_HP, false, c.pos),
                                -o.turns,
                            )
                        });
                    let Some(option) = best else {
                        continue;
                    };
                    let ProductionItem::Unit { unit } = option.item else {
                        unreachable!()
                    };
                    let power = self.modernization_power(unit, 0, data::UNIT_HP, false, c.pos);
                    let already = self
                        .orders
                        .iter()
                        .find_map(|o| match o {
                            Order::SetProduction { city, item } if *city == c.id => Some(*item),
                            _ => None,
                        })
                        .or(d.producing);
                    let keep = matches!(already, Some(ProductionItem::Unit { unit: old })
                    if (crate::composition::is_melee_line(old)
                        || crate::composition::is_ranged_line(old))
                        && ((data::unit_stats(old)
                            .requires
                            .map(legatum_core::eras::era_of_tech)
                            >= data::unit_stats(unit)
                                .requires
                                .map(legatum_core::eras::era_of_tech)
                            && self.modernization_power(old, 0, data::UNIT_HP, false, c.pos) * 4
                                >= power * 3)
                            || d.options.iter().any(|o| {
                                o.item == ProductionItem::Unit { unit: old }
                                    && o.turns > 0
                                    && o.turns <= front.arrival + 2
                                    && o.turns < option.turns
                            })));
                    if !keep {
                        self.orders.retain(
                            |o| !matches!(o, Order::SetProduction { city, .. } if *city == c.id),
                        );
                        self.orders.push(Order::SetProduction {
                            city: c.id,
                            item: option.item,
                        });
                    }
                    self.door_cities.push(c.id);
                    assigned.insert(c.id);
                    gap -= if let Some(ProductionItem::Unit { unit: old }) =
                        already.filter(|_| keep)
                    {
                        self.modernization_power(old, 0, data::UNIT_HP, false, c.pos)
                    } else {
                        power
                    };
                }
            }
        }
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let reserve = self.ordinary_gold_reserve();
        let exposed: Vec<(&'a CityView, u8)> = self
            .ctx
            .own_cities
            .iter()
            .copied()
            .map(|c| {
                (
                    c,
                    self.threat_exposure(c)
                        .max(u8::from(self.early_warning_power(c) > 0)),
                )
            })
            .filter(|(c, e)| {
                *e > 0
                    || (c.walls
                        && self.decisions.walled_defense_percent > 0
                        && self.defense_risk(c, None).falls)
            })
            .collect();
        for (c, exposure) in exposed {
            let Some(d) = c.detail.as_ref() else {
                continue;
            };
            let busy = self.orders.iter().any(|o| match o {
                Order::SetProduction { city, .. } | Order::BuyItem { city, .. } => *city == c.id,
                _ => false,
            });
            if busy || d.resistance > 0 || self.keep_committed_settler(c) {
                continue;
            }
            let defending = matches!(
                d.producing,
                Some(ProductionItem::Unit { unit }) if data::unit_stats(unit).military
            );
            // At the door the treasury is for defence; otherwise keep the usual reserve.
            let keep = if exposure >= 2 { 0 } else { reserve };
            let affordable = |o: &legatum_core::view::ProductionOption| {
                o.buy_block.is_none()
                    && o.turns > 1
                    && o.buy_cost
                        .is_some_and(|cost| cost > 0 && self.gold_left - cost >= keep)
            };
            // Gene `walls_under_way`: Walls in progress are the durable answer, so finish
            // them with Production; a shooter two hexes out still covers the city.
            let walls_kept = self.hard_defense() && self.decisions.walls_under_way > 0;
            if walls_kept
                && !c.walls
                && let Some(ProductionItem::Building { building }) = d.producing
                && data::building_family(building) == BuildingKind::Walls
                && (self.decisions.siege_survival_clock == 0
                    || self
                        .defense_risk(c, None)
                        .production_deadline()
                        .is_none_or(|fall| d.turns_to_complete.is_some_and(|turns| turns < fall)))
            {
                continue;
            }
            let reach = if walls_kept { 2 } else { 1 };
            let shooter_here = self
                .ctx
                .own_units
                .iter()
                .any(|u| u.ranged > 0 && u.pos.distance(c.pos) <= reach);
            if !shooter_here && !c.walls && !self.walls_beat_arrival(c) {
                let Some(option) = crate::composition::RANGED_LADDER
                    .into_iter()
                    .map(|unit| ProductionItem::Unit { unit })
                    .filter_map(|item| d.options.iter().find(|o| o.item == item))
                    .max_by_key(|o| {
                        if self.decisions.hard_purchase_scorer == 0
                            || self.difficulty != legatum_core::AiDifficulty::Hard
                        {
                            return -(crate::composition::RANGED_LADDER
                                .iter()
                                .position(|unit| o.item == ProductionItem::Unit { unit: *unit })
                                .unwrap_or(0) as i32);
                        }
                        let ProductionItem::Unit { unit } = o.item else {
                            unreachable!()
                        };
                        self.military_purchase_score(unit, o.buy_cost.unwrap_or(o.cost * 2), c.pos)
                    })
                else {
                    continue;
                };
                if exposure >= 2 && affordable(option) && !self.military_upgrade_hold(c) {
                    self.gold_left -= option.buy_cost.unwrap_or(0);
                    self.orders.push(Order::BuyItem {
                        city: c.id,
                        item: option.item,
                    });
                } else if !defending {
                    self.orders.push(Order::SetProduction {
                        city: c.id,
                        item: option.item,
                    });
                }
                continue;
            }
            if c.walls {
                let warning = self.conqueror_alarm(c);
                if (self.decisions.walled_defense_percent > 0 && self.defense_risk(c, None).falls)
                    || warning > self.local_defense(c)
                {
                    let stake = crate::production::city_keep_value(c)
                        * self.decisions.walled_defense_percent.max(warning)
                        / 100;
                    let option = d
                        .options
                        .iter()
                        .filter(|o| {
                            matches!(o.item, ProductionItem::Unit { unit }
                            if crate::composition::is_melee_line(unit)
                                || crate::composition::is_ranged_line(unit))
                        })
                        .filter(|o| o.turns > 0)
                        .max_by_key(|o| {
                            let ProductionItem::Unit { unit } = o.item else {
                                unreachable!()
                            };
                            let after = self.defense_risk(
                                c,
                                Some(crate::purchase_defense::purchased_defender(
                                    self.view(),
                                    c,
                                    unit,
                                )),
                            );
                            let stats = self
                                .view()
                                .rule_experiments
                                .unit_stats_for(self.faction, unit);
                            // A counterfactual garrison that survives the forecast saves the
                            // city's stake. Defence strength still ranks imperfect answers.
                            let saved = if after.falls { 0 } else { stake };
                            let strength = stats.strength.max(stats.ranged);
                            (saved + strength * 2) * 100 / o.turns.max(1) - o.cost
                        });
                    if let Some(option) = option
                        && d.producing != Some(option.item)
                    {
                        self.orders.push(Order::SetProduction {
                            city: c.id,
                            item: option.item,
                        });
                    }
                }
                continue;
            }
            if self.faction == legatum_core::Faction::Mongols {
                // Open cities buy a second mobile defender at the door instead of
                // waiting for Walls that this faction cannot build.
                if exposure >= 2 {
                    let local = self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| u.military && u.pos.distance(c.pos) <= 1)
                        .count();
                    if local < 2
                        && !self.military_upgrade_hold(c)
                        && let Some(option) = crate::composition::MELEE_LADDER
                            .into_iter()
                            .map(|unit| ProductionItem::Unit { unit })
                            .filter_map(|item| {
                                d.options.iter().find(|o| o.item == item && affordable(o))
                            })
                            .max_by_key(|o| {
                                if self.decisions.hard_purchase_scorer == 0
                                    || self.difficulty != legatum_core::AiDifficulty::Hard
                                {
                                    return -(crate::composition::MELEE_LADDER
                                        .iter()
                                        .position(|unit| {
                                            o.item == ProductionItem::Unit { unit: *unit }
                                        })
                                        .unwrap_or(0)
                                        as i32);
                                }
                                let ProductionItem::Unit { unit } = o.item else {
                                    unreachable!()
                                };
                                self.military_purchase_score(
                                    unit,
                                    o.buy_cost.unwrap_or(o.cost * 2),
                                    c.pos,
                                )
                            })
                    {
                        self.gold_left -= option.buy_cost.unwrap_or(0);
                        self.orders.push(Order::BuyItem {
                            city: c.id,
                            item: option.item,
                        });
                    }
                }
                continue;
            }
            if !d.options.iter().any(|o| o.item == walls) {
                continue;
            }
            if d.producing != Some(walls) && (!defending || self.walls_beat_arrival(c)) {
                // A scouted threat starts Walls with Production when they finish before
                // arrival; an in-progress shooter remains the emergency answer otherwise.
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: walls,
                });
            }
        }
    }
}

/// One threat-driven decision, for telemetry (`Brain::threat_log`) and saved AI memory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum ThreatEvent {
    /// The first turn this rival became our top threat, and why.
    TopThreat {
        turn: u32,
        rival: PlayerId,
        score: i32,
        reasons: Vec<String>,
    },
    /// Walls selected for Production in a city exposed to the top threat; first per city.
    Walls {
        turn: u32,
        city: CityId,
        rival: PlayerId,
    },
    /// A Pact or Alliance we proposed or accepted while facing a top threat (not with it).
    Treaty {
        turn: u32,
        with: TeamId,
        kind: legatum_core::diplomacy::TreatyKind,
        rival: PlayerId,
        accepted: bool,
    },
    /// A war we declared on the top threat (a second front).
    SecondFront {
        turn: u32,
        against: TeamId,
        rival: PlayerId,
        busy_elsewhere: bool,
        /// `Planner::second_front_value` when declared (absent in older logs).
        #[serde(default)]
        value: Option<i32>,
    },
}

/// Record this turn's threat-driven decisions. Called after planning with the final orders.
pub(crate) fn log_decisions(
    p: &Planner<'_>,
    orders: &[legatum_core::Order],
    log: &mut Vec<ThreatEvent>,
) {
    use legatum_core::diplomacy::TreatyKind;
    use legatum_core::{BuildingKind, Order, ProductionItem};
    let Some(top) = p.top_threat() else {
        return;
    };
    let v = p.view();
    let turn = v.turn;
    if !log
        .iter()
        .any(|e| matches!(e, ThreatEvent::TopThreat { rival, .. } if *rival == top.rival))
    {
        log.push(ThreatEvent::TopThreat {
            turn,
            rival: top.rival,
            score: top.score,
            reasons: top.reasons(),
        });
    }
    let threat_team = v.team_of(top.rival);
    let walls = ProductionItem::Building {
        building: BuildingKind::Walls,
    };
    for order in orders {
        match order {
            Order::SetProduction { city, item }
                if *item == walls
                    && p.ctx
                        .own_cities
                        .iter()
                        .any(|c| c.id == *city && p.threat_exposure(c) > 0)
                    && !log
                        .iter()
                        .any(|e| matches!(e, ThreatEvent::Walls { city: c, .. } if c == city)) =>
            {
                log.push(ThreatEvent::Walls {
                    turn,
                    city: *city,
                    rival: top.rival,
                });
            }
            Order::ProposeTreaty { to, kind }
                if *kind == TreatyKind::Alliance && *to != threat_team =>
            {
                log.push(ThreatEvent::Treaty {
                    turn,
                    with: *to,
                    kind: *kind,
                    rival: top.rival,
                    accepted: false,
                });
            }
            Order::RespondToProposal {
                proposal,
                accept: true,
                ..
            } => {
                if let Some(pr) = v.diplomacy.proposals.iter().find(|x| x.id == *proposal)
                    && pr.kind == TreatyKind::Alliance
                    && pr.from != threat_team
                {
                    log.push(ThreatEvent::Treaty {
                        turn,
                        with: pr.from,
                        kind: pr.kind,
                        rival: top.rival,
                        accepted: true,
                    });
                }
            }
            Order::DeclareWar { against, .. } if *against == threat_team => {
                log.push(ThreatEvent::SecondFront {
                    turn,
                    against: *against,
                    rival: top.rival,
                    busy_elsewhere: top.busy_elsewhere,
                    value: p.second_front_value(*against),
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use crate::opponents::OpponentAssessments;
    use crate::{Brain, Knobs};
    use legatum_core::diplomacy::Relation;
    use legatum_core::{AiDifficulty, BuildingKind, Order, ProductionItem, TechId, UnitKind};

    const THIRD: PlayerId = PlayerId(3);

    fn relation(a: u8, b: u8, state: RelationState, declarers: &[u8]) -> Relation {
        Relation {
            a: TeamId(a),
            b: TeamId(b),
            state,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: (state == RelationState::War).then_some(1),
            declarers: declarers.iter().map(|t| TeamId(*t)).collect(),
        }
    }

    /// ENEMY (team 2) at war with THIRD (team 3), which it declared; we are at peace.
    fn world() -> PlayerView {
        world_with(&[])
    }

    fn world_with(techs: &[TechId]) -> PlayerView {
        let mut v = ViewBuilder::new(30, 16)
            .known_techs(techs)
            .city(1, ME, off(4, 8), 3)
            .city(2, ENEMY, off(12, 8), 3)
            .city(3, ENEMY, off(14, 4), 2)
            .city(4, THIRD, off(20, 8), 2)
            .build();
        v.diplomacy.legacy = false;
        v.diplomacy.relations = vec![
            relation(0, 2, RelationState::Peace, &[]),
            relation(0, 3, RelationState::Peace, &[]),
            relation(2, 3, RelationState::War, &[2]),
        ];
        for p in &mut v.players {
            p.cities = v.cities.iter().filter(|c| c.owner == p.id).count() as u32;
        }
        v.turn = 30;
        v
    }

    #[test]
    fn quiet_border_army_is_a_top_threat_relative_to_ours() {
        let mut v = world();
        v.diplomacy.relations[2].state = RelationState::Peace;
        v.diplomacy.relations[2].declarers.clear();
        let v = ViewBuilder { view: v }
            .unit(1, ME, UnitKind::Spearman, off(3, 8))
            .unit(50, ENEMY, UnitKind::Horseman, off(9, 8))
            .unit(51, ENEMY, UnitKind::Horseman, off(10, 8))
            .build();
        let mut brain = crate::fixture::brain(&v, 7, AiDifficulty::Hard);
        brain.plan(&v);
        let threat = brain.opponents.threats.get(ENEMY).unwrap();
        assert!(threat.score >= TOP_THREAT, "quiet massing army: {threat:?}");
        assert_eq!(brain.opponents.threats.top().map(|t| t.rival), Some(ENEMY));
        let mut legacy = crate::fixture::brain(&v, 7, AiDifficulty::Hard);
        legacy.decisions.army_threat_weight = 0;
        legacy.plan(&v);
        assert!(legacy.opponents.threats.get(ENEMY).unwrap().score < TOP_THREAT);

        let mut approach = v.clone();
        approach
            .units
            .iter_mut()
            .find(|u| u.id.0 == 50)
            .unwrap()
            .pos = off(11, 8);
        approach
            .units
            .iter_mut()
            .find(|u| u.id.0 == 51)
            .unwrap()
            .pos = off(11, 7);
        let mut warned = crate::fixture::brain(&approach, 7, AiDifficulty::Hard);
        warned.plan(&approach);
        assert!(
            warned.opponents.threats.get(ENEMY).unwrap().score >= TOP_THREAT,
            "a two-turn mounted approach is still a border buildup"
        );
    }

    fn capture(v: &mut PlayerView, city: u32, to: PlayerId) {
        let c = v.cities.iter_mut().find(|c| c.id.0 == city).unwrap();
        let from = c.owner;
        c.owner = to;
        for p in &mut v.players {
            if p.id == from {
                p.cities -= 1;
                p.alive = p.cities > 0;
            }
            if p.id == to {
                p.cities += 1;
            }
        }
        v.turn += 1;
    }

    #[test]
    fn transfers_between_empires_at_war_are_captures_and_eliminations() {
        let mut v = world();
        let mut model = OpponentAssessments::default();
        model.update(&v);
        assert_eq!(model.threats.get(ENEMY).unwrap().captures, 0);
        assert!(model.threats.top().is_none());
        capture(&mut v, 4, ENEMY);
        model.update(&v);
        let t = model.threats.get(ENEMY).unwrap();
        assert_eq!((t.captures, t.eliminations), (1, 1));
        assert_eq!(model.threats.records[&ENEMY].eliminations, vec![THIRD]);
        assert_eq!(model.threats.captured_cities[&CityId(4)], (ENEMY, v.turn));
        assert_eq!(model.threats.top().map(|t| t.rival), Some(ENEMY));
        // The same turn observed twice changes nothing.
        let before = model.clone();
        model.update(&v);
        assert_eq!(before, model);
    }

    #[test]
    fn the_conqueror_alarm_grows_with_his_pace_and_stays_on_at_the_border() {
        let mut v = world_with(&[TechId::Masonry]);
        crate::fixture::set_army(&mut v, ENEMY, 100);
        v.cities.iter_mut().find(|c| c.id == CityId(4)).unwrap().pos = off(13, 10);
        let mut model = OpponentAssessments::default();
        model.update(&v);
        capture(&mut v, 4, ENEMY);
        model.update(&v);
        let genes = |p: &mut Planner, alarm: i32| {
            p.decisions.neighbour_fall = 100;
            p.decisions.neighbour_alarm = alarm;
        };
        let mut on = planner(&v, &model, Faction::India);
        genes(&mut on, 1);
        let mut off_gene = planner(&v, &model, Faction::India);
        genes(&mut off_gene, 0);
        let city = on.ctx.own_cities[0];
        // One capture in the last 20 turns: half as much again as the former warning.
        let former = off_gene.neighbour_fall_power(off_gene.ctx.own_cities[0]);
        assert!(former > 0);
        assert!(on.conqueror_alarm(city) > former);
        assert_eq!(off_gene.conqueror_alarm(off_gene.ctx.own_cities[0]), former);
        // His army at our border: unit positioning hands over to live forecasts, production
        // keeps the alarm.
        let door_view = ViewBuilder { view: v.clone() }
            .unit(70, ENEMY, UnitKind::Spearman, off(5, 8))
            .build();
        let mut door = planner(&door_view, &model, Faction::India);
        genes(&mut door, 1);
        let home = door.ctx.own_cities[0];
        assert_eq!(door.neighbour_fall_power(home), 0);
        assert!(door.conqueror_alarm(home) > 0);
        let mut door_off = planner(&door_view, &model, Faction::India);
        genes(&mut door_off, 0);
        assert_eq!(door_off.conqueror_alarm(door_off.ctx.own_cities[0]), 0);
    }

    #[test]
    fn a_neighbour_fall_warns_a_peaceful_city_and_fades_after_peace() {
        let mut v = world_with(&[TechId::Masonry]);
        crate::fixture::set_army(&mut v, ENEMY, 100);
        v.cities.iter_mut().find(|c| c.id == CityId(4)).unwrap().pos = off(13, 10);
        let mut model = OpponentAssessments::default();
        model.update(&v);
        capture(&mut v, 4, ENEMY);
        model.update(&v);
        let mut p = planner(&v, &model, Faction::India);
        let city = p.ctx.own_cities[0];
        assert!(p.neighbour_fall_power(city) > 0);
        assert_eq!(p.threat_exposure(city), 1);
        p.prepare_hard_defense();
        assert!(p.ctx.threat[0] > 0);
        let mut peer_army = v.clone();
        crate::fixture::set_army(&mut peer_army, ME, 80);
        let peer = planner(&peer_army, &model, Faction::India);
        assert_eq!(peer.neighbour_fall_power(peer.ctx.own_cities[0]), 0);
        let door_view = ViewBuilder { view: v.clone() }
            .unit(70, ENEMY, UnitKind::Spearman, off(5, 8))
            .build();
        let door = planner(&door_view, &model, Faction::India);
        assert_eq!(door.neighbour_fall_power(door.ctx.own_cities[0]), 0);
        let away_view = ViewBuilder { view: v.clone() }
            .unit(71, ENEMY, UnitKind::Spearman, off(25, 2))
            .unit(72, ENEMY, UnitKind::Spearman, off(26, 2))
            .unit(73, ENEMY, UnitKind::Spearman, off(27, 2))
            .unit(74, ENEMY, UnitKind::Spearman, off(28, 2))
            .build();
        let away = planner(&away_view, &model, Faction::India);
        assert_eq!(away.neighbour_fall_power(away.ctx.own_cities[0]), 0);
        let mut old = planner(&v, &model, Faction::India);
        old.decisions.neighbour_fall = 0;
        assert_eq!(old.neighbour_fall_power(old.ctx.own_cities[0]), 0);
        for relation in &mut v.diplomacy.relations {
            relation.state = RelationState::Peace;
        }
        let peaceful = planner(&v, &model, Faction::India);
        assert_eq!(peaceful.neighbour_fall_power(peaceful.ctx.own_cities[0]), 0);
    }

    #[test]
    fn public_conquest_and_a_seen_nearby_conqueror_warn_without_revealing_the_city() {
        let mut v = ViewBuilder { view: world() }
            .city(5, THIRD, off(26, 8), 2)
            .unit(50, ENEMY, UnitKind::Spearman, off(12, 8))
            .build();
        v.players[THIRD.0 as usize].cities = 2;
        v.cities.iter_mut().find(|c| c.id == CityId(4)).unwrap().pos = off(24, 8);
        crate::fixture::set_army(&mut v, ENEMY, 100);
        let mut model = OpponentAssessments::default();
        model.update(&v);
        capture(&mut v, 4, ENEMY);
        v.cities.retain(|c| c.id != CityId(4));
        model.update(&v);
        assert!(model.threats.fallen_fronts.is_empty());
        let one = planner(&v, &model, Faction::India);
        assert_eq!(one.neighbour_fall_power(one.ctx.own_cities[0]), 0);
        capture(&mut v, 5, ENEMY);
        v.cities.retain(|c| c.id != CityId(5));
        model.update(&v);
        let p = planner(&v, &model, Faction::India);
        assert!(p.neighbour_fall_power(p.ctx.own_cities[0]) > 0);
    }

    #[test]
    fn a_loaded_game_does_not_recast_an_old_elimination_as_a_new_capture() {
        let mut v = world();
        capture(&mut v, 4, ENEMY);
        v = ViewBuilder { view: v }
            .unit(50, ENEMY, UnitKind::Spearman, off(12, 8))
            .build();
        crate::fixture::set_army(&mut v, ENEMY, 100);
        let mut model = OpponentAssessments::default();
        model.update(&v);
        let p = planner(&v, &model, Faction::India);
        assert_eq!(p.neighbour_fall_power(p.ctx.own_cities[0]), 0);
    }

    #[test]
    fn a_loaded_game_sees_a_dead_empire_and_the_war_that_killed_it() {
        let mut v = world();
        capture(&mut v, 4, ENEMY);
        let mut model = OpponentAssessments::default();
        model.update(&v);
        let t = model.threats.get(ENEMY).unwrap();
        assert_eq!(t.eliminations, 1);
        assert!(t.score >= TOP_THREAT, "{t:?}");
    }

    #[test]
    fn peaceful_growth_is_momentum_not_conquest() {
        let mut v = world();
        v.diplomacy.relations[2].state = RelationState::Peace;
        v.diplomacy.relations[2].declarers.clear();
        let mut model = OpponentAssessments::default();
        model.update(&v);
        // A new city founded at peace: the public count rises with no loser.
        v.turn += 1;
        v.players.iter_mut().find(|p| p.id == ENEMY).unwrap().cities += 1;
        model.update(&v);
        let t = model.threats.get(ENEMY).unwrap();
        assert_eq!(t.captures, 0);
        assert!(t.city_growth > 0);
        assert!(t.score < WATCH_THREAT, "{t:?}");
    }

    #[test]
    fn a_runaway_vp_leader_is_watched_wherever_it_is() {
        use legatum_core::view::TeamStanding;
        let mut v = world();
        v.diplomacy.relations[2].state = RelationState::Peace;
        v.diplomacy.relations[2].declarers.clear();
        let standing = |team: u8, vp| TeamStanding {
            team: TeamId(team),
            members: vec![PlayerId(team)],
            vp,
            target: 10,
            victory_hold: None,
        };
        v.standings = vec![standing(0, 3), standing(2, 8), standing(3, 4)].into();
        let mut model = OpponentAssessments::default();
        model.update(&v);
        let t = model.threats.get(ENEMY).unwrap();
        // 80% with a 40-point lead: 40 + 15, not scaled down by distance.
        assert_eq!(t.race, 55);
        assert!(t.score >= WATCH_THREAT, "{t:?}");
        assert_eq!(model.threats.get(THIRD).unwrap().race, 0);
    }

    #[test]
    fn identical_observations_give_identical_scores_under_reordering() {
        let mut v = world();
        capture(&mut v, 4, ENEMY);
        let mut a = OpponentAssessments::default();
        a.update(&v);
        let mut r = v.clone();
        r.players.reverse();
        r.cities.reverse();
        r.diplomacy.relations.reverse();
        let mut b = OpponentAssessments::default();
        b.update(&r);
        assert_eq!(a.threats.scores, b.threats.scores);
    }

    fn planner<'a>(
        v: &'a PlayerView,
        model: &'a OpponentAssessments,
        faction: Faction,
    ) -> Planner<'a> {
        let mut p = Planner::new(v, Knobs::for_faction(faction), faction, 9);
        p.difficulty = AiDifficulty::Hard;
        p.opponents = Some(model);
        p
    }

    fn conqueror_next_door() -> (PlayerView, OpponentAssessments) {
        conqueror_next_door_with(&[])
    }

    fn conqueror_next_door_with(techs: &[TechId]) -> (PlayerView, OpponentAssessments) {
        let mut v = world_with(techs);
        // Move the conqueror's city within reach of ours, then let it take THIRD.
        v.cities[1].pos = off(10, 8);
        capture(&mut v, 4, ENEMY);
        let mut model = OpponentAssessments::default();
        model.update(&v);
        (v, model)
    }

    #[test]
    fn an_exposed_city_garrisons_a_shooter_then_walls() {
        let (v, model) = conqueror_next_door();
        let mut p = planner(&v, &model, Faction::India);
        assert_eq!(p.threat_exposure(p.ctx.own_cities[0]), 1);
        assert!(p.threat_research());
        assert!(p.orders.contains(&Order::SetResearch {
            tech: TechId::Masonry
        }));
        // Before Walls, the first answer is a ranged unit for the city.
        let mut p = planner(&v, &model, Faction::India);
        p.plan_threat_defense();
        assert_eq!(
            p.orders,
            vec![Order::SetProduction {
                city: CityId(1),
                item: ProductionItem::Unit {
                    unit: UnitKind::Archer
                }
            }]
        );
        // With a shooter in place and the tech known, Walls come next.
        let (walled, model) = conqueror_next_door_with(&[TechId::Masonry]);
        let walled = ViewBuilder { view: walled }
            .unit(9, ME, UnitKind::Archer, off(4, 8))
            .build();
        let mut p = planner(&walled, &model, Faction::India);
        p.plan_threat_defense();
        assert_eq!(
            p.orders,
            vec![Order::SetProduction {
                city: CityId(1),
                item: ProductionItem::Building {
                    building: BuildingKind::Walls
                }
            }]
        );
        // Normal does not read the threat model at all.
        let mut normal = planner(&walled, &model, Faction::India);
        normal.difficulty = AiDifficulty::Normal;
        assert_eq!(normal.threat_exposure(normal.ctx.own_cities[0]), 0);
        normal.plan_threat_defense();
        assert!(normal.orders.is_empty());
    }

    #[test]
    fn a_scouted_threat_starts_walls_with_production_when_they_beat_arrival() {
        let (mut v, model) = conqueror_next_door_with(&[TechId::Masonry]);
        let city = v.cities.iter_mut().find(|c| c.id == CityId(1)).unwrap();
        city.detail
            .as_mut()
            .unwrap()
            .options
            .iter_mut()
            .find(|o| {
                o.item
                    == (ProductionItem::Building {
                        building: BuildingKind::Walls,
                    })
            })
            .unwrap()
            .turns = 1;
        let mut p = planner(&v, &model, Faction::India);
        assert!(p.walls_beat_arrival(p.ctx.own_cities[0]));
        p.plan_threat_defense();
        assert!(
            p.orders.contains(&Order::SetProduction {
                city: CityId(1),
                item: ProductionItem::Building {
                    building: BuildingKind::Walls
                }
            }),
            "{:?}",
            p.orders
        );
        assert!(!p.orders.iter().any(|o| matches!(
            o,
            Order::BuyItem {
                item: ProductionItem::Building {
                    building: BuildingKind::Walls | BuildingKind::Castle | BuildingKind::StarFort
                },
                ..
            }
        )));
    }

    #[test]
    fn mongols_defends_with_units_and_does_not_research_walls_for_a_threat() {
        let (mut v, model) = conqueror_next_door();
        v.players.iter_mut().find(|p| p.id == ME).unwrap().faction = Faction::Mongols;
        let mut p = planner(&v, &model, Faction::Mongols);
        assert!(!p.threat_research());
        p.plan_threat_defense();
        assert!(p.orders.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                item: ProductionItem::Unit {
                    unit: UnitKind::Archer | UnitKind::Crossbowman
                },
                ..
            } | Order::BuyItem {
                item: ProductionItem::Unit {
                    unit: UnitKind::Archer | UnitKind::Crossbowman
                },
                ..
            }
        )));
        assert!(!p.orders.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                item: ProductionItem::Building {
                    building: BuildingKind::Walls
                },
                ..
            } | Order::BuyItem {
                item: ProductionItem::Building {
                    building: BuildingKind::Walls
                },
                ..
            }
        )));
    }

    #[test]
    fn second_fronts_weigh_strength_and_economy_and_temperament() {
        let (mut v, model) = conqueror_next_door();
        let team = TeamId(2);
        // THIRD is dead, so nobody else holds its attention: a weak empire stays out.
        crate::fixture::set_army(&mut v, ME, 40);
        crate::fixture::set_army(&mut v, ENEMY, 100);
        assert_eq!(
            planner(&v, &model, Faction::India).second_front_value(team),
            None
        );
        // Sparta relish the war when clearly stronger.
        crate::fixture::set_army(&mut v, ME, 130);
        let sparta = planner(&v, &model, Faction::Sparta).second_front_value(team);
        assert!(sparta.is_some_and(|value| value > 0), "{sparta:?}");
        // Faction temperament: Switzerland values the same war less. India's court
        // funding now makes it more reluctant than the neutral Celts to open a second front.
        assert!(planner(&v, &model, Faction::Switzerland).second_front_value(team) < sparta);
        assert!(
            second_front_temperament(Faction::India) < second_front_temperament(Faction::Celts)
        );
        // A war we cannot pay for is never started.
        v.economy.gold = 0;
        v.economy.gold_per_turn.total = -2;
        assert_eq!(
            planner(&v, &model, Faction::Sparta).second_front_value(team),
            None
        );
    }

    #[test]
    fn walled_leader_still_has_a_priced_second_front() {
        use legatum_core::view::TeamStanding;
        let (mut v, model) = conqueror_next_door();
        v.cities[2].pos = off(10, 4);
        v.cities[3].pos = off(10, 12);
        for city in v.cities.iter_mut().filter(|c| c.owner == ENEMY) {
            city.walls = true;
            city.hp = city.max_hp;
        }
        crate::fixture::set_army(&mut v, ME, 120);
        crate::fixture::set_army(&mut v, ENEMY, 100);
        v.standings = vec![
            TeamStanding {
                team: v.team,
                members: vec![ME],
                vp: 3,
                target: 10,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(2),
                members: vec![ENEMY],
                vp: 7,
                target: 10,
                victory_hold: None,
            },
        ]
        .into();
        let mut p = planner(&v, &model, Faction::Switzerland);
        p.decisions.second_front_siege_price = 0;
        assert_eq!(p.second_front_value(TeamId(2)), None);
        p.decisions.second_front_siege_price = 1;
        let value = p.second_front_value(TeamId(2));
        assert!(value.is_some_and(|x| x >= 0), "{value:?}");
    }

    #[test]
    fn walls_under_way_are_finished_not_swapped_for_a_shooter() {
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        let (v, model) = conqueror_next_door_with(&[TechId::Masonry]);
        let v = ViewBuilder { view: v }.producing(1, walls, 9999).build();
        // Off (playtest 8.9.2's behaviour): no shooter next to the city, so Walls under way
        // turn into an Archer.
        let mut p = planner(&v, &model, Faction::India);
        p.decisions.walls_under_way = 0;
        p.plan_threat_defense();
        assert_eq!(
            p.orders,
            vec![Order::SetProduction {
                city: CityId(1),
                item: ProductionItem::Unit {
                    unit: UnitKind::Archer
                }
            }]
        );
        // On: the Walls keep their place in the queue.
        let mut p = planner(&v, &model, Faction::India);
        p.decisions.walls_under_way = 1;
        p.plan_threat_defense();
        assert!(p.orders.is_empty(), "{:?}", p.orders);
        assert!(!p.orders.iter().any(|o| matches!(
            o,
            Order::BuyItem {
                item: ProductionItem::Building {
                    building: BuildingKind::Walls | BuildingKind::Castle | BuildingKind::StarFort
                },
                ..
            }
        )));
        // A shooter two hexes out counts as covering a city without Walls under way.
        let (open, model) = conqueror_next_door_with(&[TechId::Masonry]);
        let open = ViewBuilder { view: open }
            .unit(9, ME, UnitKind::Archer, off(6, 8))
            .build();
        let mut p = planner(&open, &model, Faction::India);
        p.decisions.walls_under_way = 1;
        p.plan_threat_defense();
        assert_eq!(
            p.orders,
            vec![Order::SetProduction {
                city: CityId(1),
                item: walls
            }]
        );
    }

    #[test]
    fn a_conquerors_army_at_the_door_is_acute() {
        let (v, _) = conqueror_next_door();
        let v = ViewBuilder { view: v }
            .unit(7, ENEMY, UnitKind::Spearman, off(6, 8))
            .unit(8, ENEMY, UnitKind::Spearman, off(6, 9))
            .build();
        let mut model = OpponentAssessments::default();
        model.update(&v);
        let p = planner(&v, &model, Faction::India);
        let top = model.threats.top().unwrap();
        assert!(top.near_army > 0);
        assert_eq!(
            p.threat_exposure(p.ctx.own_cities[0]),
            if top.score >= TOP_THREAT + 30 { 2 } else { 1 }
        );
    }
}
