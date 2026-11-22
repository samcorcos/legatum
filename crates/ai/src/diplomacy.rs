//! Diplomatic objectives use public standings and the same fogged view as the army.
use crate::scripted::Planner;
use legatum_core::diplomacy::coalition::pressure_view as coalition_pressure;
use legatum_core::diplomacy::{RelationState, TreatyKind};
use legatum_core::{Hex, Order, PlayerView, TeamId, UnitKind};
use std::collections::BTreeSet;

/// Turns an AI must know a team before offering it an Alliance. Every team is public on the
/// victory race from turn 1, so this is game turns since the start: no AI opens with an
/// Alliance.
pub const ALLIANCE_ACQUAINTANCE_TURNS: u32 = 15;

/// Whether we have known the other teams long enough to offer an Alliance.
pub fn alliance_acquainted(view: &PlayerView) -> bool {
    view.turn > ALLIANCE_ACQUAINTANCE_TURNS
}

/// An active betrayal record: `by` broke an Alliance with `against`.
fn betrayed(view: &PlayerView, by: TeamId, against: TeamId) -> bool {
    view.diplomacy
        .betrayals
        .iter()
        .any(|b| b.by == by && b.against == against)
}

/// Offers to a team with a human player are staggered so at most one new AI offer reaches
/// it per turn. AI empires plan independently from the same public state, so each alive
/// AI-controlled team gets its own turn slot (turn mod count, in team order).
pub fn offer_slot_open(view: &PlayerView, to: TeamId) -> bool {
    if !slot_limited(view, to) {
        return true;
    }
    let mut senders: Vec<TeamId> = view
        .players
        .iter()
        .filter(|p| p.alive && !p.is_human && p.team != to)
        .map(|p| p.team)
        .collect();
    senders.sort_unstable();
    senders.dedup();
    senders
        .iter()
        .position(|t| *t == view.team)
        .is_none_or(|rank| view.turn as usize % senders.len() == rank)
}

fn slot_limited(view: &PlayerView, to: TeamId) -> bool {
    view.players
        .iter()
        .any(|p| p.team == to && p.alive && p.is_human)
}

/// Score lost per Alliance we already hold, have accepted this turn or have offered. Each
/// one forecloses another war and costs upkeep; restraint comes from the sum.
pub const TREATY_HELD_PENALTY: i32 = 6;

/// How much of a rival's army a treaty keeps off us, by how close it is (hexes between the
/// nearest known cities or units). Distant empires can't threaten us, so an Alliance with them
/// buys no security.
pub fn treaty_reach(distance: Option<i32>) -> i32 {
    match distance {
        Some(d) if d <= 6 => 100,
        Some(d) if d <= 10 => 60,
        Some(d) if d <= 14 => 30,
        _ => 0,
    }
}

/// What an Alliance protects us from: their public army against ours (0..200%), scaled by
/// reach. An equal neighbour is worth 20; a twice-as-strong neighbour 40; anyone far away 0.
pub fn alliance_security(ours: i32, theirs: i32, distance: Option<i32>) -> i32 {
    (theirs * 100 / ours.max(1)).clamp(0, 200) * treaty_reach(distance) / 500
}

/// The wars an Alliance forecloses: our aggression and our army edge against a reachable rival.
pub fn alliance_opportunity(aggression: i32, ours: i32, theirs: i32, distance: Option<i32>) -> i32 {
    let edge = (ours * 100 / theirs.max(1) - 100).clamp(0, 50);
    (aggression.clamp(0, 100) / 2 + edge / 2) * treaty_reach(distance) / 100
}

/// Alliance score for fighting a shared leader at `progress` percent of its target: nothing
/// below half, then up to +30 for a team about to win.
fn leader_war_value(progress: u32) -> i32 {
    (progress as i32 - 40).clamp(0, 60) / 2
}

/// A nearby rival's land and public victory progress raise the value of a feasible war.
/// Distance comes only from discovered cities and units; a distant unknown empire adds nothing.
fn war_land_value(distance: Option<i32>, progress: u32) -> i32 {
    let frontier = match distance {
        Some(d) if d <= 6 => 20,
        Some(d) if d <= 10 => 10,
        _ => 0,
    };
    if distance.is_none() || distance.is_some_and(|d| d > 14) {
        0
    } else {
        frontier + (progress as i32 / 5).min(20)
    }
}

/// Turns of upkeep the AI weighs.
const TREATY_COST_HORIZON: i32 = 10;
/// Score for each team of the target's side we already fight, when answering a call to arms.
const CALL_SHARED_ENEMY_BONUS: i32 = 10;
/// Gold of upkeep that costs the AI one point of treaty score.
const TREATY_GOLD_PER_POINT: i32 = 10;
/// Turns of any Gold deficit after upkeep that the treasury must cover.
const TREATY_RUNWAY_TURNS: i32 = 20;

/// Diagnostics (`ScriptedBrain::peace_diagnosis`): how this AI would answer a Peace with one
/// enemy this turn, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeaceDiagnosis {
    /// `wants_peace` and the gate that decided it.
    pub wants: bool,
    pub gate: &'static str,
    /// The Peace score (`treaty_score`); None when a veto refuses outright. Accepted at 0+.
    pub score: Option<i32>,
    pub terms: Vec<(&'static str, i32)>,
    pub vp_cost: i32,
    /// Turns since our units last attacked this enemy (or since the war began).
    pub idle_turns: Option<u32>,
    pub plan: String,
    pub endangered: bool,
    /// Gene `dead_war_peace`: turns past the gene's age of a war nobody fights.
    pub dead_war: Option<i32>,
    /// Every own city with visible enemy pressure within 5: (city, HP, max HP, pressure, the
    /// local defence plus half the city's power, the pressure's share from `against`, whether
    /// the city is in resistance). `endangered` fires on HP <= 100 or pressure above defence.
    pub danger: Vec<(u32, i32, i32, i32, i32, i32, bool)>,
}

/// The Gold side of an Alliance for `v.me`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreatyCost {
    /// Current treasury.
    pub treasury: i32,
    /// Score penalty: one point per `TREATY_GOLD_PER_POINT` Gold over `TREATY_COST_HORIZON`
    /// turns of upkeep, plus a heavy penalty when the treasury cannot cover
    /// `TREATY_RUNWAY_TURNS` of any deficit the upkeep leaves (a bankrupt treasury disbands
    /// units).
    pub penalty: i32,
    pub affordable: bool,
}

/// Turn an Alliance's upkeep into a score penalty for `v.me`. An Alliance with `partner` costs
/// nothing while the waiver applies (either side at War with a team close to winning,
/// `PlayerView::alliance_upkeep_waived_by`), so it is priced at what it really costs now.
pub fn treaty_cost(v: &PlayerView, kind: TreatyKind, partner: TeamId) -> TreatyCost {
    let upkeep = treaty_upkeep(v, kind, partner);
    let treasury = v.economy.gold;
    let net = v.economy.gold_per_turn.total - upkeep;
    if upkeep == 0 {
        return TreatyCost {
            treasury,
            penalty: 0,
            affordable: true,
        };
    }
    let affordable =
        v.economy.bankruptcy.is_none() && treasury + TREATY_RUNWAY_TURNS * net.min(0) >= 0;
    let mut penalty = TREATY_COST_HORIZON * upkeep / TREATY_GOLD_PER_POINT;
    if !affordable {
        penalty += 50;
    }
    TreatyCost {
        treasury,
        penalty,
        affordable,
    }
}

/// The Gold per turn an Alliance with `partner` would cost `v.me` now: nothing while the waiver
/// applies, otherwise the ordinary upkeep for its cities.
pub fn treaty_upkeep(v: &PlayerView, kind: TreatyKind, partner: TeamId) -> i32 {
    if kind == TreatyKind::Alliance && v.alliance_upkeep_waived_by(v.team, partner).is_some() {
        return 0;
    }
    kind.upkeep_for_cities(v.cities.iter().filter(|c| c.owner == v.me).count())
}

/// Hexes from our nearest city (or unit, before we have a city) to their nearest known city or
/// visible unit. None when we know of nothing of theirs.
pub fn nearest_known(v: &PlayerView, other: TeamId) -> Option<i32> {
    let ours: Vec<Hex> = {
        let cities: Vec<Hex> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == v.team)
            .map(|c| c.pos)
            .collect();
        if cities.is_empty() {
            v.units
                .iter()
                .filter(|u| v.team_of(u.owner) == v.team)
                .map(|u| u.pos)
                .collect()
        } else {
            cities
        }
    };
    v.cities
        .iter()
        .filter(|c| v.team_of(c.owner) == other)
        .map(|c| c.pos)
        .chain(
            v.units
                .iter()
                .filter(|u| !u.owner.is_barbarian() && v.team_of(u.owner) == other)
                .map(|u| u.pos),
        )
        .flat_map(|h| ours.iter().map(move |o| o.distance(h)))
        .min()
}

/// Alliances this plan already holds, has offered, or has accepted this turn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TreatyBudget {
    pub held: i32,
}

impl TreatyBudget {
    pub fn of(v: &PlayerView) -> Self {
        let mut b = Self {
            held: v
                .diplomacy
                .relations
                .iter()
                .filter(|r| (r.a == v.team || r.b == v.team) && r.security.is_some())
                .count() as i32,
        };
        for p in v.diplomacy.proposals.iter().filter(|p| p.from == v.team) {
            if p.kind == TreatyKind::Alliance {
                b.commit();
            }
        }
        b
    }

    /// Count a treaty as signed.
    pub fn commit(&mut self) {
        self.held += 1;
    }
}

pub(crate) fn strength(view: &PlayerView, team: TeamId) -> i32 {
    view.players
        .iter()
        .filter(|p| p.team == team)
        .map(|p| p.army)
        .sum()
}

/// Public army of a team and every team allied with it: what fights on its side in a war.
pub(crate) fn side_strength(view: &PlayerView, team: TeamId) -> i32 {
    view.diplomacy
        .side(team)
        .into_iter()
        .map(|t| strength(view, t))
        .sum()
}

/// The force that can legally fight this war. Direct allies defend automatically; an
/// offensive call is optional, and chains do not enter unless already at war.
pub(crate) fn participating_strength(
    view: &PlayerView,
    team: TeamId,
    enemy: TeamId,
    offensive: bool,
    optional_percent: i32,
) -> i32 {
    let existing = view.teams_at_war(team, enemy);
    let mut total = strength(view, team);
    for s in view.diplomacy.side(team) {
        if s == team {
            continue;
        }
        let weight = if view.teams_at_war(s, enemy) {
            100
        } else if existing {
            0
        } else if view
            .diplomacy
            .relations
            .iter()
            .any(|r| r.state == RelationState::Alliance && r.between(team, s))
        {
            if offensive { optional_percent } else { 100 }
        } else {
            0
        };
        total += strength(view, s) * weight / 100;
    }
    total
}

fn coalition_target(view: &PlayerView) -> Option<TeamId> {
    legatum_core::diplomacy::coalition::target_view(view)
}

fn progress(v: &PlayerView, team: TeamId) -> u32 {
    v.standings
        .iter()
        .find(|s| s.team == team)
        .map_or(0, |s| 100 * s.vp / s.target.max(1))
}

/// A coalition's war has a public objective. Once that target falls behind, let it
/// recover instead of using a favorable army ratio to prolong yesterday's dogpile.
fn coalition_disengage(v: &PlayerView, other: TeamId) -> bool {
    let Some(war) = v
        .diplomacy
        .relations
        .iter()
        .find(|r| r.between(v.team, other) && r.state == RelationState::War)
    else {
        return false;
    };
    if v.turn.saturating_sub(war.effective_turn) < 8 {
        return false;
    }
    let best = v
        .standings
        .iter()
        .filter(|s| v.players.iter().any(|p| p.team == s.team && p.alive))
        .map(|s| coalition_pressure(v, s.team))
        .max()
        .unwrap_or(0);
    // A war an ally brought us into, once its target falls behind (or someone else leads).
    legatum_core::diplomacy::joined_war(&v.diplomacy.relations, v.team, other)
        && (coalition_pressure(v, other) < 60 || best >= coalition_pressure(v, other) + 10)
}

/// Gene `coalition_own_read`: threat-score points within which the formal coalition target
/// wins a near-tie against our own top threat.
const COALITION_TIE: i32 = 10;

impl Planner<'_> {
    /// Opportunity cost of ending a war that can remove a rival's next point. Compare it
    /// with the same city's retaliation loss used by the plan model; a collapsing defence
    /// can still rationally seek Peace.
    pub(crate) fn vp_peace_cost(&self, against: TeamId) -> i32 {
        if self.decisions.vp_peace == 0 {
            return 0;
        }
        let v = self.view();
        let Some(s) = v.standings.iter().find(|s| s.team == against) else {
            return 0;
        };
        if against == v.team || s.vp + 1 < s.target {
            return 0;
        }
        let base = crate::winchance::WinFacts::of(v)
            .with_relative(&self.decisions)
            .refined();
        let mut broken = base.clone();
        let needed = s.vp.saturating_sub(s.target) as i32 + 1;
        let unit = broken.vp_unit;
        if let Some(team) = broken.team_mut(against) {
            team.vp -= needed * unit;
        }
        let swing = (broken.chance() - base.chance()).max(0);
        let reach = nearest_known(v, against).map_or(0, |d| (15 - d).clamp(0, 10));
        let ratio = (strength(v, v.team) * 100 / strength(v, against).max(1)).min(100);
        // Even a distant war occupies a front and can join a coalition; it has a small
        // option value until a force comes into range.
        let pressure = (ratio * reach / 10).max(10);
        swing * pressure * self.decisions.vp_peace / 10_000
    }

    fn vp_war_loss(&self, against: TeamId) -> i32 {
        let base = crate::winchance::WinFacts::of(self.view())
            .with_relative(&self.decisions)
            .refined();
        let lost = self.city_lost(&base, against);
        self.retaliation_permille(against, &[]) * (base.chance() - lost.chance()).max(0) / 1000
            + if self.endangered() { 200 } else { 0 }
    }

    pub(crate) fn war_side_strength(&self, team: TeamId, enemy: TeamId, offensive: bool) -> i32 {
        let v = self.view();
        if self.decisions.ally_commitment_model == 0 {
            side_strength(v, team)
        } else {
            participating_strength(
                v,
                team,
                enemy,
                offensive,
                self.decisions.ally_offensive_join_percent,
            )
        }
    }
    /// Public city counts plus a recent declared war identify a locally expanding
    /// aggressor before VP alone reaches the global coalition threshold. This is
    /// an inference of conquest risk, not knowledge of fogged captures or armies.
    fn containment_target(&self) -> Option<TeamId> {
        if !self.hard_defense() {
            return None;
        }
        // The threat model (captures, eliminations, wars, armies at our border, momentum)
        // names a conqueror long before VP or the city-count rule below would.
        if let Some(team) = self.top_threat_team() {
            return Some(team);
        }
        let v = self.view();
        let own_members = v.players.iter().filter(|p| p.team == v.team).count().max(1) as u32;
        let own_cities: u32 = v
            .players
            .iter()
            .filter(|p| p.team == v.team)
            .map(|p| p.cities)
            .sum();
        v.standings
            .iter()
            .filter_map(|s| {
                if s.team == v.team || !v.players.iter().any(|p| p.team == s.team && p.alive) {
                    return None;
                }
                let members = v.players.iter().filter(|p| p.team == s.team).count().max(1) as u32;
                let cities: u32 = v
                    .players
                    .iter()
                    .filter(|p| p.team == s.team)
                    .map(|p| p.cities)
                    .sum();
                if cities < 5 * members
                    || cities * own_members < (own_cities + 2 * own_members) * members
                {
                    return None;
                }
                if !v.diplomacy.relations.iter().any(|r| {
                    r.state == RelationState::War
                        && r.declarers.contains(&s.team)
                        && v.turn.saturating_sub(r.effective_turn) <= 80
                }) {
                    return None;
                }
                let distance = v
                    .cities
                    .iter()
                    .filter(|c| v.team_of(c.owner) == s.team)
                    .flat_map(|c| {
                        self.ctx
                            .own_cities
                            .iter()
                            .map(move |home| home.pos.distance(c.pos))
                    })
                    .min()?;
                (distance <= 12).then_some((
                    std::cmp::Reverse(cities * 100 / members),
                    std::cmp::Reverse(progress(v, s.team)),
                    distance,
                    s.team,
                ))
            })
            .min()
            .map(|(_, _, _, team)| team)
    }

    pub(crate) fn diplomatic_target(&self) -> Option<TeamId> {
        let public = coalition_target(self.view());
        // Gene `coalition_own_read`: each member follows its own threat read (captures, wars,
        // armies at its border, the runaway signs), and the formal coalition target decides
        // only a near-tie (8.9.4 Swiss: at T193 the AIs turned on the formal target, Byzantium at
        // 9 of 10, and left the user, the runaway every threat read named, free to win).
        if self.decisions.coalition_own_read > 0
            && self.hard_defense()
            && let Some(top) = self.top_threat()
        {
            let v = self.view();
            let top_team = v.team_of(top.rival);
            let tie = public.filter(|t| {
                *t != top_team
                    && self.opponents.is_some_and(|m| {
                        m.threats.scores.values().any(|r| {
                            v.team_of(r.rival) == *t && r.score + COALITION_TIE >= top.score
                        })
                    })
            });
            return tie.or(Some(top_team));
        }
        if public.is_some_and(|team| progress(self.view(), team) >= 80) {
            public
        } else {
            self.containment_target().or(public)
        }
    }

    /// Hard (gene `coalition_alliance`): the leader both we and `other` are at war with, as
    /// its victory progress (100 for a team counting down to victory). A leader is a team
    /// counting down, our diplomatic target, or the strict public VP leader at half its target
    /// or more. Playtest 8.4: two AIs at war with the same 6-VP leader refused each other an
    /// Alliance eight times on the 20-turn treasury test and allied 49 turns later.
    pub(crate) fn shared_leader_war(&self, other: TeamId) -> Option<u32> {
        if self.difficulty != legatum_core::AiDifficulty::Hard
            || self.decisions.coalition_alliance == 0
        {
            return None;
        }
        let v = self.view();
        let alive = |team: TeamId| v.players.iter().any(|p| p.team == team && p.alive);
        let target = self.diplomatic_target();
        v.standings
            .iter()
            .filter(|s| {
                s.team != v.team
                    && s.team != other
                    && alive(s.team)
                    && v.teams_at_war(v.team, s.team)
                    && v.teams_at_war(other, s.team)
            })
            .filter_map(|s| {
                if crate::hold::rival_hold_of(v, s.team).is_some() {
                    return Some(100);
                }
                let leads = v
                    .standings
                    .iter()
                    .all(|o| o.team == s.team || !alive(o.team) || o.vp < s.vp);
                let p = progress(v, s.team);
                (target == Some(s.team) || (leads && p >= 50)).then_some(p)
            })
            .max()
    }

    pub fn plan_diplomacy(&mut self) {
        let v = self.view();
        if v.diplomacy.legacy {
            return;
        }
        let alive = |team| v.players.iter().any(|p| p.team == team && p.alive);
        let wars: Vec<_> = v
            .standings
            .iter()
            .filter(|s| alive(s.team) && v.teams_at_war(v.team, s.team))
            .map(|s| s.team)
            .collect();
        let target = self.diplomatic_target();
        let oath_target = self.sparta_expedition_target();
        let legal = |order: &Order| {
            v.diplomacy
                .actions
                .iter()
                .find(|a| legatum_core::diplomacy::same_action(&a.order, order))
                .is_some_and(|a| a.blocked_reason.is_none())
        };
        let mut handled = BTreeSet::new();
        let mut decisions = Vec::new();
        // Answer the most valuable treaty offers first, accounting for the
        // treaty count earlier acceptances leave (no cap; just the sum).
        let mut budget = TreatyBudget::of(v);
        let open = budget;
        let mut incoming: Vec<_> = v
            .diplomacy
            .proposals
            .iter()
            .filter(|p| p.needs.contains(&v.team))
            .map(|p| {
                let other = v.diplomacy.counterparty(p, v.team);
                let accept = Order::RespondToProposal {
                    proposal: p.id,
                    accept: true,
                    explanation: None,
                };
                let first = self.treaty_score(other, p.kind, &accept, &open);
                (std::cmp::Reverse(first.unwrap_or(i32::MIN)), p.id, p)
            })
            .collect();
        incoming.sort_by_key(|(first, id, _)| (*first, *id));
        for (_, _, p) in incoming {
            let other = v.diplomacy.counterparty(p, v.team);
            let accept = Order::RespondToProposal {
                proposal: p.id,
                accept: true,
                explanation: None,
            };
            let score = if (oath_target == Some(other) && p.kind == TreatyKind::Alliance)
                || (p.kind == TreatyKind::Peace && !self.wants_peace(other))
            {
                Some(-100)
            } else {
                self.treaty_score(other, p.kind, &accept, &budget)
            };
            let order = Order::RespondToProposal {
                proposal: p.id,
                accept: legal(&accept) && score.is_some_and(|s| s >= 0),
                explanation: (p.kind == TreatyKind::Alliance)
                    .then(|| self.explain_response(p, score, &accept, &budget)),
            };
            if legal(&order) {
                if matches!(order, Order::RespondToProposal { accept: true, .. })
                    && p.kind == TreatyKind::Alliance
                {
                    budget.commit();
                }
                decisions.push(order);
                handled.insert(other);
            }
        }
        // An ally's war is a call we may decline: join what is worth fighting, stay allied
        // and at Peace with the rest.
        for call in &v.diplomacy.calls {
            // One diplomatic order per other team per turn: an answer to an offer from the same
            // ally already used it, and the call lapses as a decline.
            if handled.contains(&call.ally) {
                continue;
            }
            let join = Order::RespondToProposal {
                proposal: call.id,
                accept: true,
                explanation: None,
            };
            let order = Order::RespondToProposal {
                proposal: call.id,
                accept: legal(&join) && self.call_value(call.target) >= 0,
                explanation: None,
            };
            if legal(&order) {
                decisions.push(order);
                handled.insert(call.ally);
            }
        }
        let mut offers = Vec::new();
        let mut declarations = Vec::new();
        for a in &v.diplomacy.actions {
            if a.blocked_reason.is_some() {
                continue;
            }
            match a.order {
                // Leave an Alliance only when it is free (outside its protected window): a
                // partner nearing victory, a growing aggressor, an Alliance whose coalition
                // reason has lapsed, or one we can no longer afford.
                Order::BreakTreaty { with, .. }
                    if !handled.contains(&with)
                        && a.cost == 0
                        && (progress(v, with) >= (self.decisions.ally_progress_limit as u32)
                            || v.economy.bankruptcy.is_some()
                            || self.containment_target() == Some(with)
                            || oath_target == Some(with)
                            || (!self.endangered() && self.alliance_war_opportunity(with))
                            || v.diplomacy.relations.iter().any(|r| {
                                r.between(v.team, with)
                                    && r.state == RelationState::Alliance
                                    && r.security.as_ref().is_some_and(|t| {
                                        t.coalition_target.is_some_and(|old| {
                                            !alive(old)
                                                // A conqueror is still a reason to stand
                                                // together when its VP dip (threat.rs).
                                                || (coalition_pressure(v, old) < 60
                                                    && self.top_threat_team() != Some(old))
                                                || v.standings.iter().any(|s| {
                                                    s.team != old
                                                        && alive(s.team)
                                                        && coalition_pressure(v, s.team)
                                                            > coalition_pressure(v, old)
                                                })
                                                || target == Some(v.team)
                                                || target == Some(with)
                                        })
                                    })
                            })) =>
                {
                    decisions.push(a.order.clone());
                    handled.insert(with);
                }
                Order::ProposeTreaty { to, kind } if !handled.contains(&to) => {
                    if oath_target == Some(to) && kind == TreatyKind::Alliance {
                        continue;
                    }
                    if (kind == TreatyKind::Alliance && !alliance_acquainted(v))
                        || !offer_slot_open(v, to)
                    {
                        continue;
                    }
                    if v.diplomacy.peace_cooldowns.iter().any(|c| {
                        c.from == v.team && c.to == to && c.kind == kind && v.turn < c.retry_turn
                    }) {
                        continue;
                    }
                    if kind == TreatyKind::Peace && !self.wants_peace(to) {
                        continue;
                    }
                    let score = self.treaty_score(to, kind, &a.order, &budget);
                    if let Some(score) = score.filter(|s| *s >= 0) {
                        offers.push((std::cmp::Reverse(score), to, kind, a.order.clone()));
                    }
                }
                Order::DeclareWar { against, breach } if !handled.contains(&against) => {
                    // An open city can wall itself before a five-regiment field army
                    // finishes assembling. After a first capture, start this local
                    // follow-up while the army is already on its border.
                    if !breach
                        && v.cities.iter().any(|city| {
                            v.team_of(city.owner) == against
                                && (self.sparta_open_followup(city)
                                    || self.sparta_chain_target(city))
                        })
                        && !self.ctx.enemy_units.iter().any(|u| {
                            u.military
                                && u.kind != UnitKind::Scout
                                && self
                                    .ctx
                                    .own_cities
                                    .iter()
                                    .any(|home| home.pos.distance(u.pos) <= 5)
                        })
                        && !v.players.iter().any(|p| {
                            p.alive
                                && p.cities > 0
                                && p.team != v.team
                                && p.team != against
                                && v.teams_at_war(v.team, p.team)
                        })
                    {
                        declarations.push((
                            std::cmp::Reverse(true),
                            0,
                            std::cmp::Reverse(i32::MAX),
                            against,
                            a.order.clone(),
                        ));
                        continue;
                    }
                    // A goal-driven plan declares only the war it names, already priced
                    // against every alternative (`plan.rs`); legality was checked above.
                    if let Some(plan) = self.routed_plan() {
                        if plan.kind.war_target() == Some(against) && plan.breach == breach {
                            // Opening fire while the Sparta army is eight hexes away
                            // gives a defender time to complete Walls. A selected early
                            // expedition first stages at Peace, then declares when the
                            // same funded local forecast used by campaigns is ready.
                            if self.musters_campaigns()
                                && v.turn <= legatum_core::data::FIRST_THIRD_TURNS
                                && let crate::plan::PlanKind::Campaign { target, .. } = &plan.kind
                                && let Some(city) = v.cities.iter().find(|c| c.id == *target)
                                && !city.walls
                                && !self.sparta_open_followup(city)
                                && !self.campaign_outlook(city).is_some_and(|f| f.march == 0)
                                // Gene `assault_wing_forecast`: a wing staged at the border (its
                                // land is closed until war opens it) that the forecast says takes
                                // the city declares; the march through opened land is short.
                                && !(self.decisions.assault_wing_forecast > 0
                                    && self.assault_wing_ready(city, Some(6)))
                            {
                                continue;
                            }
                            declarations.push((
                                std::cmp::Reverse(true),
                                0,
                                std::cmp::Reverse(i32::MAX),
                                against,
                                a.order.clone(),
                            ));
                        }
                        continue;
                    }
                    let coalition = target == Some(against) && against != v.team;
                    // A team counting down to victory: declare whenever our army can reach it.
                    let holder = crate::hold::rival_hold_of(v, against).is_some()
                        && self.reachable_war_target(against);
                    // The veteran objective starts one sustainable war; it does not
                    // open a second front on its own.
                    let expedition = !breach && wars.is_empty() && oath_target == Some(against);
                    // A local, forecast-profitable capture is already priced for retaliation.
                    // It need not satisfy the broader army ratio used for diffuse coalitions.
                    let opportunity = !breach && self.strike_against(against).is_some();
                    // Public victory containment can justify a second front under the
                    // ordinary legality, strength and affordability checks.
                    if !wars.is_empty() && (!coalition || wars.len() >= 2) && !holder {
                        continue;
                    }
                    let relation = v
                        .diplomacy
                        .relations
                        .iter()
                        .find(|r| r.between(v.team, against))
                        .unwrap();
                    // War on an ally is always a betrayal. It is reserved for an imminent
                    // public victory, never mere opportunism; otherwise leave the Alliance
                    // honourably first.
                    if breach
                        && (relation.state != RelationState::Alliance
                            || !coalition
                            || progress(v, against) < 90)
                    {
                        continue;
                    }
                    let theirs = self.war_side_strength(against, v.team, false);
                    let ratio = self.war_side_strength(v.team, against, true) * 100 / theirs.max(1);
                    if coalition {
                        let own_side = v.diplomacy.side(v.team);
                        let combined: i32 = self.war_side_strength(v.team, against, true)
                            + v.standings
                                .iter()
                                .filter(|s| {
                                    alive(s.team)
                                        && !own_side.contains(&s.team)
                                        && v.teams_at_war(s.team, against)
                                })
                                .map(|s| strength(v, s.team))
                                .sum::<i32>();
                        if combined * 100 < theirs * 80 && !opportunity && !holder {
                            continue;
                        }
                    }
                    let bounty = v
                        .cities
                        .iter()
                        .filter(|city| v.team_of(city.owner) == against)
                        .filter_map(|city| city.capture_bounty)
                        .max()
                        .unwrap_or(0);
                    let denial = v
                        .cities
                        .iter()
                        .filter(|city| v.team_of(city.owner) == against)
                        .map(|city| {
                            self.hoard_denial_value(city)
                                + self.greece_captive_tech_value(city)
                                + self.swarm_weakness_value(city)
                        })
                        .max()
                        .unwrap_or(0)
                        .max(self.held_goal_denial(against));
                    let hubs = self.opponents.map_or(0, |model| {
                        model
                            .rivals
                            .values()
                            .filter(|r| v.team_of(r.id) == against)
                            .flat_map(|r| &r.cities)
                            .map(|city| 15 * city.india_links as i32)
                            .max()
                            .unwrap_or(0)
                    });
                    let expansion = self.opponents.map_or(0, |model| {
                        model
                            .rivals
                            .values()
                            .filter(|r| v.team_of(r.id) == against)
                            .map(|r| r.public_city_count_change.max(0))
                            .sum::<i32>()
                    });
                    let score = self.knobs.aggression
                        + if coalition { 5 * expansion.min(3) } else { 0 }
                        + (hubs / 5).min(20)
                        + (bounty / 20).min(40)
                        + denial / 5
                        + (ratio - 100).clamp(-50, 50) / 2
                        + war_land_value(nearest_known(v, against), progress(v, against))
                        + 30 * i32::from(coalition)
                        + 80 * i32::from(holder) * self.decisions.vp_hold_break / 100
                        - 20 * wars.len() as i32;
                    // A feasible second front against a growing aggressor is useful
                    // even to a peaceful personality; legality, combined strength and
                    // a real spare campaign force remain mandatory below.
                    let score = if coalition && self.containment_target() == Some(against) {
                        if self.top_threat_team() == Some(against) {
                            // A second front against a conqueror must pay: expected gain
                            // from its distraction, our allies and weakly held cities,
                            // minus the risk of its army at our door (threat.rs).
                            match self.second_front_value(against) {
                                Some(value) if value >= 0 => score.max(70),
                                _ if opportunity || holder => score,
                                _ => continue,
                            }
                        } else {
                            score.max(70)
                        }
                    } else {
                        score
                    };
                    let campaign = self.diplomatic_campaign(against);
                    let prepared = self.conquest_ready(against);
                    let raid = !breach && wars.is_empty() && self.border_raid_opportunity(against);
                    // A local force that the forecast says takes a city within a few turns,
                    // for more than it risks (`opportunity.rs`): strike now, while it lasts.
                    if ((score >= 70 || prepared) && (campaign.is_some() || raid))
                        || expedition
                        || opportunity
                        || holder
                    {
                        let rush_cost = if self.strategy == Some(crate::Strategy::Rush)
                            && let Some(target) = campaign
                        {
                            let city = v
                                .cities
                                .iter()
                                .find(|c| c.pos == target)
                                .expect("campaign city");
                            let distance = self
                                .ctx
                                .own_cities
                                .iter()
                                .map(|c| c.pos.distance(target))
                                .min()
                                .unwrap_or(99);
                            crate::analysis::city_power(v, city) * (8 + distance)
                        } else {
                            0
                        };
                        declarations.push((
                            std::cmp::Reverse(coalition || holder),
                            rush_cost,
                            std::cmp::Reverse(score),
                            against,
                            a.order.clone(),
                        ));
                    }
                }
                _ => {}
            }
        }
        declarations
            .sort_by_key(|(coalition, cost, score, team, _)| (*coalition, *cost, *score, *team));
        if let Some((_, _, _, team, order)) = declarations.into_iter().next() {
            handled.insert(team);
            decisions.retain(|o| !matches!(o, Order::BreakTreaty { with, .. } if *with == team));
            decisions.push(order);
        }
        // All AI teammates answer for themselves; one deterministic team spokesperson
        // originates offers, so distinct preferred pairs cannot create several offers.
        if v.players
            .iter()
            .filter(|p| p.alive && !p.is_human && p.team == v.team)
            .map(|p| p.id)
            .min()
            == Some(v.me)
        {
            // An offer to a human team can only go out in this AI's slot for that team (one
            // turn in N); offers to AIs can go next turn. Spend a scarce slot first, or a
            // human ally is never the one called when an AI ally has a lower TeamId.
            offers.sort_by_key(|(score, team, kind, _)| {
                (
                    std::cmp::Reverse(slot_limited(v, *team)),
                    *score,
                    *team,
                    *kind,
                )
            });
            if let Some((_, _, _, order)) =
                offers.into_iter().find(|(_, t, _, _)| !handled.contains(t))
            {
                decisions.push(order);
            }
        }
        self.orders.extend(decisions);
    }

    /// After the protected window, leave an Alliance when a reachable campaign's
    /// territorial and victory value exceeds the security that Alliance buys us.
    /// The ordinary war legality and force check still apply on the following turn.
    fn alliance_war_opportunity(&self, other: TeamId) -> bool {
        let v = self.view();
        if v.standings.iter().any(|s| v.teams_at_war(v.team, s.team)) {
            return false;
        }
        let ours = strength(v, v.team);
        let theirs = strength(v, other);
        let distance = nearest_known(v, other);
        let ratio = ours * 100 / theirs.max(1);
        let bounty = v
            .cities
            .iter()
            .filter(|city| v.team_of(city.owner) == other)
            .filter_map(|city| city.capture_bounty)
            .max()
            .unwrap_or(0);
        let value = self.knobs.aggression
            + (bounty / 20).min(40)
            + (ratio - 100).clamp(-50, 50) / 2
            + war_land_value(distance, progress(v, other));
        value >= 70 + alliance_security(ours, theirs, distance) / 2
            && self.diplomatic_campaign(other).is_some()
    }

    /// Germany defends rather than sieges. Its optional wars need a valuable local field
    /// target instead of a city campaign.
    /// Count only visible troops that could fight around one of our cities.
    fn border_raid_opportunity(&self, against: TeamId) -> bool {
        if self.decisions.border_raid == 0 || self.endangered() {
            return false;
        }
        let v = self.view();
        v.units.iter().any(|enemy| {
            enemy.military
                && v.team_of(enemy.owner) == against
                && v.tile(enemy.pos).is_some_and(|t| t.visible)
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|home| home.pos.distance(enemy.pos) <= 4)
                && {
                    let ours: i32 = self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| {
                            u.military
                                && u.kind != legatum_core::UnitKind::Scout
                                && u.hp >= 60
                                && u.pos.distance(enemy.pos) <= 4
                        })
                        .map(|u| crate::analysis::threat_power(v, u))
                        .sum();
                    let theirs: i32 = v
                        .units
                        .iter()
                        .filter(|u| {
                            u.military
                                && v.team_of(u.owner) == against
                                && u.pos.distance(enemy.pos) <= 4
                        })
                        .map(|u| crate::analysis::threat_power(v, u))
                        .sum();
                    ours * 10 >= theirs * 18 && theirs * 3 >= ours && ours > 0
                }
        })
    }

    fn explain_response(
        &self,
        p: &legatum_core::diplomacy::PeaceProposal,
        score: Option<i32>,
        order: &Order,
        budget: &TreatyBudget,
    ) -> legatum_core::diplomacy::DecisionExplanation {
        let v = self.view();
        let target = self.diplomatic_target();
        let mut reasons = vec![
            "Base score: -10".into(),
            format!("Proposer victory progress: {}%", progress(v, p.from)),
        ];
        if score == Some(-100) && self.sparta_expedition_target() == Some(p.from) {
            reasons.push("The Iron Guard war needs this nearby border; honor existing notice periods before war".into());
        }
        if let Some(reason) = v
            .diplomacy
            .actions
            .iter()
            .find(|a| legatum_core::diplomacy::same_action(&a.order, order))
            .and_then(|a| a.blocked_reason.clone())
        {
            reasons.push(reason);
        }
        if p.kind == TreatyKind::Alliance {
            if progress(v, p.from) >= (self.decisions.ally_offer_progress as u32)
                && self.shared_leader_war(p.from).is_none_or(|lead| {
                    lead < progress(v, p.from) + 10
                        || progress(v, p.from) >= (self.decisions.ally_progress_limit as u32)
                })
            {
                reasons.push("A prospective ally has reached 60% victory progress".into());
            }
            if betrayed(v, p.from, v.team) {
                reasons.push("They broke an alliance with us; we won't ally yet".into());
            }
            if target.is_some_and(|x| x != v.team && x != p.from) {
                reasons.push("We face the same coalition target: +25".into());
            }
            let shared_enemy = v
                .standings
                .iter()
                .any(|s| v.teams_at_war(v.team, s.team) && v.teams_at_war(p.from, s.team));
            if shared_enemy {
                reasons.push("Shared enemy: +25".into());
            }
            if !shared_enemy && !target.is_some_and(|x| x != v.team && x != p.from) {
                reasons.push(
                    "No shared enemy or coalition target: +0 (a coalition forms against a team \
                     strictly ahead at 60% with a 15-point lead, or at 80%)"
                        .into(),
                );
            }
            let distance = nearest_known(v, p.from);
            let (ours, theirs) = (strength(v, v.team), strength(v, p.from));
            reasons.extend(self.alliance_war_value(order).1);
            reasons.push(format!(
                "Their army it keeps off us: +{}; wars it forecloses: -{}",
                alliance_security(ours, theirs, distance),
                alliance_opportunity(self.knobs.aggression, ours, theirs, distance)
            ));
            let c = treaty_cost(v, p.kind, p.from);
            reasons.push(format!(
                "{} Gold/turn upkeep: score -{}",
                treaty_upkeep(v, p.kind, p.from),
                c.penalty
            ));
            if let Some(lead) = self.shared_leader_war(p.from) {
                reasons.push(format!(
                    "We both fight the leader ({lead}% of its target): its win costs more than \
                     the upkeep, +{}",
                    leader_war_value(lead)
                ));
            } else if !c.affordable {
                reasons.push("The upkeep would empty our treasury within 20 turns".into());
            }
            reasons.push(format!(
                "Alliances held or committed: {} (score -{})",
                budget.held,
                TREATY_HELD_PENALTY * budget.held
            ));
        }
        if self.endangered() {
            reasons.push("Endangered: alliance security +15".into());
        }
        if score.is_none() {
            reasons.push(
                "Victory progress, a betrayal or the Gold cost prevents this agreement".into(),
            );
        }
        reasons.truncate(11);
        if let Some(score) = score {
            reasons.push(format!("Final score {score}; accept at zero or higher"));
        }
        legatum_core::diplomacy::DecisionExplanation { score, reasons }
    }

    /// What answering an ally's call to arms against `target` with Join is worth to us: the
    /// price of the war itself (`join_war_value`) and a small bonus for each team on the
    /// target's side we already fight (it is largely our war already).
    pub(crate) fn call_value(&self, target: TeamId) -> i32 {
        let v = self.view();
        let shared = v
            .diplomacy
            .side(target)
            .into_iter()
            .filter(|t| v.teams_at_war(v.team, *t))
            .count()
            .min(3) as i32;
        self.join_war_value(target) + CALL_SHARED_ENEMY_BONUS * shared
    }

    /// What an Alliance does to our wars (allies defend each other at once, and a war an ally
    /// declared is a call to arms we may decline): the wars we would
    /// join, each priced by whether it is worth fighting, and the allies who would join ours.
    /// Returns the score and the reasons that make it up.
    fn alliance_war_value(&self, order: &Order) -> (i32, Vec<String>) {
        let v = self.view();
        let Some(action) = v
            .diplomacy
            .actions
            .iter()
            .find(|a| legatum_core::diplomacy::same_action(&a.order, order))
        else {
            return (0, Vec::new());
        };
        let (mut cost, mut joined) = (0, 0);
        let (mut gain, mut helpers) = (0, 0);
        for war in &action.new_wars {
            if war.team == v.team {
                cost += self.join_war_value(war.enemy);
                joined += 1;
            } else if v.teams_at_war(v.team, war.enemy) {
                let edge = strength(v, war.team) * 100 / strength(v, war.enemy).max(1);
                let raw = 10 + (edge / 5).min(20) + 10 * i32::from(self.endangered());
                gain += raw * self.helper_reach_percent(war.team, war.enemy) / 100;
                helpers += 1;
            }
        }
        let mut notes = Vec::new();
        if !action.calls.is_empty() {
            notes.push(format!(
                "Wars we would be asked to join (we may decline): {}",
                action.calls.len()
            ));
        }
        if joined > 0 {
            notes.push(format!("Wars we would join: {joined} (score {cost:+})"));
        }
        if helpers > 0 {
            notes.push(format!(
                "Allies who would join our wars: {helpers} (score {gain:+})"
            ));
        }
        (cost + gain, notes)
    }

    /// What a bigger bloc costs us: every empire in an Alliance side is a war we can never fight
    /// and an enemy list we share, so each team the Alliance adds beyond a pair weighs on its value.
    fn bloc_size_cost(&self, other: TeamId) -> i32 {
        let v = self.view();
        let alive = |t: &TeamId| v.players.iter().any(|p| p.team == *t && p.alive);
        let merged: std::collections::BTreeSet<TeamId> = v
            .diplomacy
            .side(v.team)
            .into_iter()
            .chain(v.diplomacy.side(other))
            .filter(alive)
            .collect();
        let merged: Vec<TeamId> = merged.into_iter().collect();
        // Gene `runaway_coalition` bit 4: a bloc of teams all within reach of the runaway is free.
        if self.runaway_bloc_waives_size(&merged) {
            return 0;
        }
        self.decisions.bloc_size_penalty * (merged.len() as i32 - 2).max(0)
    }

    /// How much of an ally's help the war it would join can really give us, in percent: all of
    /// it when the ally is within 8 hexes of the enemy, none past 16 (an ally that cannot reach
    /// the war only costs us its enemies). Public information: the ally's visible cities and
    /// units, the enemy's known cities.
    pub(crate) fn helper_reach_percent(&self, helper: TeamId, enemy: TeamId) -> i32 {
        let v = self.view();
        let mine: Vec<_> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == helper)
            .map(|c| c.pos)
            .chain(
                v.units
                    .iter()
                    .filter(|u| u.military && v.team_of(u.owner) == helper)
                    .map(|u| u.pos),
            )
            .collect();
        let theirs: Vec<_> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == enemy)
            .map(|c| c.pos)
            .collect();
        let distance = mine
            .iter()
            .flat_map(|a| theirs.iter().map(move |b| a.distance(*b)))
            .min();
        match distance {
            Some(d) if d <= 8 => 100,
            Some(d) if d <= 16 => 100 * (16 - d) / 8,
            _ => 0,
        }
    }

    /// What joining an ally's war on `enemy` is worth to us: nothing is free (-20), a war on the
    /// public leader or a team counting down to victory is what we want, and the odds, our other
    /// wars and danger at home decide the rest; a war our armies cannot win or reach is a pure cost.
    fn join_war_value(&self, enemy: TeamId) -> i32 {
        let v = self.view();
        let holder = crate::hold::rival_hold_of(v, enemy).is_some();
        let own_side = v.diplomacy.side(v.team);
        let combined: i32 = self.war_side_strength(v.team, enemy, true)
            + v.standings
                .iter()
                .filter(|s| {
                    v.players.iter().any(|p| p.team == s.team && p.alive)
                        && !own_side.contains(&s.team)
                        && v.teams_at_war(s.team, enemy)
                })
                .map(|s| strength(v, s.team))
                .sum::<i32>();
        let theirs = self.war_side_strength(enemy, v.team, false);
        let advantage = (combined * 100 / theirs.max(1) - 100).clamp(-50, 50);
        // A war nobody can win, or reach, is a pure cost; a team counting down to victory is
        // worth fighting even outnumbered.
        let hopeless =
            !holder && (combined * 100 < theirs * 80 || !self.reachable_war_target(enemy));
        let wars = v
            .standings
            .iter()
            .filter(|s| s.team != enemy && v.teams_at_war(v.team, s.team))
            .count() as i32;
        -20 + 40 * i32::from(self.diplomatic_target() == Some(enemy))
            + 60 * i32::from(holder)
            + 60 * i32::from(holder) * self.decisions.vp_hold_break / 100
            + advantage / 2
            - 20 * wars
            - 30 * i32::from(self.endangered())
            - 80 * i32::from(hopeless)
            - self.inherited_front_cost(enemy)
            + self.runaway_join_bonus(enemy)
    }

    /// What a war on `enemy` (and its allies) takes from the war we already fight: the share of
    /// the army near our cities that would be theirs (a front at home), and, with a campaign
    /// under way against someone else, the odds it loses when the army has a second enemy to
    /// face. Public information only: visible units and the Strongest Army figures.
    fn inherited_front_cost(&self, enemy: TeamId) -> i32 {
        let v = self.view();
        let side = v.diplomacy.side(enemy);
        let direct_defender = |team: TeamId| {
            v.diplomacy
                .relations
                .iter()
                .any(|r| r.state == RelationState::Alliance && r.between(enemy, team))
        };
        let near = |team_in: &dyn Fn(TeamId) -> bool| -> i32 {
            v.units
                .iter()
                .filter(|u| u.military && !u.owner.is_barbarian() && team_in(v.team_of(u.owner)))
                .filter(|u| {
                    v.cities
                        .iter()
                        .any(|c| v.team_of(c.owner) == v.team && c.pos.distance(u.pos) <= 6)
                })
                .map(|u| crate::analysis::threat_power(v, u))
                .sum()
        };
        let theirs = near(&|t| {
            if self.decisions.ally_commitment_model == 0 {
                side.contains(&t)
            } else {
                t == enemy
                    || v.teams_at_war(t, v.team)
                    || (!v.teams_at_war(enemy, v.team) && direct_defender(t))
            }
        });
        let ours = near(&|t| t == v.team);
        let front = 40 * theirs / (theirs + ours + 1);
        let fighting = self
            .campaigns
            .active
            .as_ref()
            .is_some_and(|c| c.expected_value > 0)
            || self
                .campaigns
                .plan
                .active
                .as_ref()
                .and_then(|p| p.kind.war_target())
                .is_some_and(|t| t != enemy && !side.contains(&t));
        let campaign = if fighting {
            let odds =
                self.war_side_strength(enemy, v.team, false) * 100 / strength(v, v.team).max(1);
            20 + (odds / 5).clamp(0, 40)
        } else {
            0
        };
        front + campaign
    }

    pub(crate) fn treaty_score(
        &self,
        other: TeamId,
        kind: TreatyKind,
        order: &Order,
        budget: &TreatyBudget,
    ) -> Option<i32> {
        // Apply the same campaign assessment to incoming peace as to our own offers.
        // Otherwise an opponent can end a live siege by proposing peace, only for us
        // to declare again once the same border opportunity reappears.
        if (kind == TreatyKind::Peace && !self.wants_peace(other))
            || (kind == TreatyKind::Alliance
                && self
                    .conquest_city()
                    .is_some_and(|c| self.view().team_of(c.owner) == other))
        {
            return None;
        }
        // A search chose Peace with this team (it rolled out the alternatives and found nothing
        // better to do with the war): the older heuristics below must not refuse what the
        // search wants (gene `peace_follows_plan`; playtest 8.4: a Athens whose search
        // chose Peace refused two offers because the offerer was its containment target).
        let plan_peace = kind == TreatyKind::Peace
            && self.decisions.peace_follows_plan > 0
            && (!self.campaigns.plan.searched.is_empty() || self.decisions.peace_follows_plan >= 2)
            && self.routed_plan().is_some_and(
                |plan| matches!(plan.kind, crate::plan::PlanKind::Peace { with } if with == other),
            );
        // A winning strike is not traded for a treaty with its target.
        if !plan_peace
            && self.strike_against(other).is_some()
            && (kind != TreatyKind::Peace || !self.endangered())
        {
            return None;
        }
        if kind == TreatyKind::Peace
            && !plan_peace
            && self.hard_defense()
            && self.diplomatic_target() == Some(other)
            && !self.endangered()
        {
            return None;
        }
        if kind == TreatyKind::Alliance && self.containment_target() == Some(other) {
            return None;
        }
        if kind == TreatyKind::Peace && self.primitive_persist_siege(other) {
            return None;
        }
        let v = self.view();
        if self.strategy == Some(crate::Strategy::Rush) {
            // Preserve the opening attack window: no Alliance with an observed neighbour.
            // Distant partners remain useful.
            if kind == TreatyKind::Alliance
                && v.turn <= self.decisions.rush_window_turn as u32
                && v.cities.iter().any(|c| {
                    v.team_of(c.owner) == other
                        && self
                            .ctx
                            .own_cities
                            .iter()
                            .any(|h| h.pos.distance(c.pos) <= 14)
                })
            {
                return None;
            }
        }
        let leader = self.diplomatic_target();
        let coalition = leader.is_some_and(|l| l != v.team && l != other);
        let shared_enemy = v
            .standings
            .iter()
            .any(|s| v.teams_at_war(v.team, s.team) && v.teams_at_war(other, s.team));
        let age = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, other))
            .map_or(0, |r| v.turn.saturating_sub(r.effective_turn));
        let advantage =
            (strength(v, v.team) * 100 / strength(v, other).max(1) - 100).clamp(-50, 50);
        let disengage_coalition = coalition_disengage(v, other);
        // Both of us stand in the way of the same conqueror. Selective: the partner must be
        // fighting it or have a known city within twelve hexes of one of its known cities, so
        // this isn't a bonus for every distant stranger.
        let shared_threat = self.top_threat_team().is_some_and(|threat| {
            threat != other
                && threat != v.team
                && (v.teams_at_war(other, threat)
                    || v.cities.iter().any(|c| {
                        v.team_of(c.owner) == other
                            && v.cities.iter().any(|t| {
                                v.team_of(t.owner) == threat && t.pos.distance(c.pos) <= 12
                            })
                    }))
        });
        let score = match kind {
            TreatyKind::Alliance => {
                // Value: an ally against a shared enemy or the public leader, the army it keeps
                // off us (a strong reachable neighbour), and security when we are losing a war;
                // against the wars it forecloses. Price: upkeep and every
                // Alliance we already carry. A betrayer is refused while its record lasts.
                let cost = treaty_cost(v, kind, other);
                // Against a shared leader, the leader's win can outweigh the upkeep deficit.
                let shared_leader = self.shared_leader_war(other);
                let affordable = cost.affordable
                    || (shared_leader.is_some()
                        && cost.treasury >= 0
                        && v.economy.bankruptcy.is_none());
                // A partner past 60% is still a fair ally against a leader ten points further
                // ahead, until the partner itself is near winning (the leave threshold).
                let near_winner = progress(v, other) >= (self.decisions.ally_offer_progress as u32)
                    && shared_leader.is_none_or(|lead| {
                        lead < progress(v, other) + 10
                            || progress(v, other) >= (self.decisions.ally_progress_limit as u32)
                    });
                if near_winner || betrayed(v, other, v.team) || !affordable {
                    return None;
                }
                let penalty = if cost.affordable {
                    cost.penalty
                } else {
                    cost.penalty - 50
                };
                let distance = nearest_known(v, other);
                let (ours, theirs) = (strength(v, v.team), strength(v, other));
                let (war_value, _) = self.alliance_war_value(order);
                // A pact made while a conqueror approaches can draft us into its
                // next war. Price that commitment until every city has a defender
                // and one field unit can answer the call.
                let warning = self
                    .ctx
                    .own_cities
                    .iter()
                    .map(|c| self.neighbour_fall_power(c))
                    .max()
                    .unwrap_or(0);
                let field = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military && u.kind != UnitKind::Scout)
                    .count();
                let unready = (self.ctx.own_cities.len() + 1).saturating_sub(field) as i32;
                -10 + 25 * i32::from(shared_enemy)
                    + war_value
                    + shared_leader.map_or(0, leader_war_value)
                    + 25 * i32::from(coalition)
                    + 20 * i32::from(shared_threat)
                    + 15 * i32::from(self.endangered())
                    + alliance_security(ours, theirs, distance)
                        * (100 - self.decisions.ally_exit_discount_percent)
                        / 100
                    - alliance_opportunity(self.knobs.aggression, ours, theirs, distance)
                    - self.decisions.ally_vision_cost * self.ctx.own_cities.len() as i32 / 2
                    - self.decisions.ally_passage_cost * treaty_reach(distance) / 100
                    - 80 * i32::from(leader == Some(other))
                    - penalty
                    - TREATY_HELD_PENALTY * budget.held
                    - self.bloc_size_cost(other)
                    + self.runaway_alliance_value(other)
                    + self.runaway_bloc_value(other)
                    - warning.min(100) * unready
            }
            TreatyKind::Peace => {
                // A plan that chose Peace with this team accepts it whatever the score says.
                let floor = if plan_peace { 0 } else { i32::MIN };
                let sum: i32 = self
                    .peace_terms(other, age, advantage, disengage_coalition)
                    .iter()
                    .map(|(_, x)| x)
                    .sum();
                floor.max(sum)
                    - self.vp_peace_cost(other)
                    - self
                        .peace_siege_cost(other)
                        .max(self.peace_prize_cost(other))
            }
        };
        Some(score)
    }

    /// Gene `dead_war_peace`: a war nobody fights. The war with `team` is at least the gene's
    /// age, the plan fights someone else (or nobody), no campaign or strike is aimed at `team`,
    /// none of our units stands within two hexes of its cities and its army does not endanger
    /// us. Never the leader the plan contains, a team counting down, our diplomatic target
    /// (`leader_war`: even an idle war costs the leader) or a hollow leader (W2.3: that war is
    /// worth keeping). Returns the turns past the gene's age, up to 30; None when not dead.
    pub(crate) fn dead_war(&self, team: TeamId) -> Option<i32> {
        let grace = self.decisions.dead_war_peace;
        if grace <= 0 {
            return None;
        }
        let v = self.view();
        let plan = self.routed_plan()?;
        if plan.kind.war_target() == Some(team) || !v.teams_at_war(v.team, team) {
            return None;
        }
        // A co-belligerent: `team` is itself at war with the enemy our plan fights or with the
        // VP leader, so its army is already on the front that matters and a Peace frees
        // nothing of ours. Offering one only spends its diplomacy (one order per team a turn):
        // Chur T137, the Mongols' early offer to Japan turned Japan's own Peace offer to the
        // Swiss, which accepted in 24 of 24 runs (5 of 24 without the gene).
        let front = [
            plan.kind.war_target(),
            self.strict_leader(30),
            self.diplomatic_target(),
        ];
        if front
            .into_iter()
            .flatten()
            .any(|t| t != team && t != v.team && v.teams_at_war(team, t))
        {
            return None;
        }
        let age = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, team))
            .map_or(0, |r| v.turn.saturating_sub(r.effective_turn)) as i32;
        if age < grace
            || self.leader_war(team)
            || crate::threat::hollow_bonus_from(v, team, self.hollow_floor(team)) > 0
            || self.strike_against(team).is_some()
            || self.campaigns.active.as_ref().is_some_and(|c| {
                v.cities
                    .iter()
                    .any(|city| city.id == c.target && v.team_of(city.owner) == team)
            })
            || self.endangered_by(team)
            || self.collapsing()
        {
            return None;
        }
        let besieging = v.cities.iter().any(|c| {
            !c.owner.is_barbarian()
                && v.team_of(c.owner) == team
                && self
                    .ctx
                    .own_units
                    .iter()
                    .any(|u| u.military && u.pos.distance(c.pos) <= 2)
        });
        (!besieging).then_some((age - grace).min(30))
    }

    /// Gene `dead_war_peace`: the war that matters is elsewhere. A rival leader other than
    /// `enemy` (the strict VP leader from 30% of its target, our diplomatic target, or a hollow
    /// leader), not our ally: 20 when one of its cities is within 12 hexes of ours, 10 if
    /// farther.
    fn rival_leader_pull(&self, enemy: TeamId) -> i32 {
        let v = self.view();
        let candidate = |t: TeamId| {
            t != enemy
                && t != v.team
                && v.players.iter().any(|p| p.alive && p.team == t)
                && !v
                    .diplomacy
                    .relations
                    .iter()
                    .any(|r| r.between(v.team, t) && r.state == RelationState::Alliance)
        };
        let hollow = v
            .standings
            .iter()
            .map(|s| s.team)
            .filter(|t| {
                *t != v.team && crate::threat::hollow_bonus_from(v, *t, self.hollow_floor(*t)) > 0
            })
            .collect::<Vec<_>>();
        [self.strict_leader(30), self.diplomatic_target()]
            .into_iter()
            .flatten()
            .chain(hollow)
            .filter(|t| candidate(*t))
            .map(|t| match nearest_known(v, t) {
                Some(d) if d <= 12 => 20,
                _ => 10,
            })
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn peace_diagnosis(&self, against: TeamId) -> PeaceDiagnosis {
        let v = self.view();
        let (wants, gate) = self.wants_peace_why(against);
        let order = Order::ProposeTreaty {
            to: against,
            kind: TreatyKind::Peace,
        };
        let age = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, against))
            .map_or(0, |r| v.turn.saturating_sub(r.effective_turn));
        let advantage =
            (strength(v, v.team) * 100 / strength(v, against).max(1) - 100).clamp(-50, 50);
        PeaceDiagnosis {
            wants,
            gate,
            score: self.treaty_score(against, TreatyKind::Peace, &order, &TreatyBudget::of(v)),
            terms: self.peace_terms(against, age, advantage, coalition_disengage(v, against)),
            vp_cost: self.vp_peace_cost(against),
            idle_turns: self.idle_war_turns(against),
            plan: format!("{:?}", self.routed_plan().map(|p| &p.kind)),
            endangered: self.endangered(),
            danger: self.danger_cities(against),
            dead_war: self.dead_war(against),
        }
    }

    /// The terms of a Peace's score with `other` (before the plan's floor and the VP cost of
    /// peace with a holder): `treaty_score` sums them, `peace_diagnosis` prints them.
    pub(crate) fn peace_terms(
        &self,
        other: TeamId,
        age: u32,
        advantage: i32,
        disengage_coalition: bool,
    ) -> Vec<(&'static str, i32)> {
        let v = self.view();
        let horizon = self.plan_horizon();
        let truce = legatum_core::diplomacy::PEACE_TRUCE_TURNS as i32;
        let dead = self.dead_war(other);
        let danger_from_party = if self.decisions.peace_pair_scope > 0 {
            self.endangered_by(other)
        } else {
            self.endangered()
        };
        let mut terms = vec![
            ("base", 10),
            ("war age", 2 * age.min(10) as i32),
            // Gene `peace_strong`: an army well ahead of the proposer's answers the danger.
            (
                "endangered",
                30 * i32::from(
                    danger_from_party
                        && !(self.decisions.peace_strong > 0
                            && advantage >= self.decisions.peace_strong),
                ),
            ),
            // Holding our own target: a war can only cost us VP before the clock ends.
            (
                "own hold",
                40 * i32::from(crate::hold::own_hold(v).is_some()),
            ),
            ("vow at stake", 60 * i32::from(self.vow_at_stake())),
            ("coalition disengage", 30 * i32::from(disengage_coalition)),
            // Gene `war_pressure`: a war our units have not fought for a while is
            // easier to end, one point per idle turn past the grace, up to 30.
            (
                "idle war",
                if self.decisions.war_pressure > 0 && !self.leader_war(other) {
                    self.idle_war_excess(other).min(30)
                } else {
                    0
                },
            ),
            (
                "rival would redeclare",
                -(self.rival_redeclare_permille(other) * (horizon - truce).max(0)
                    / horizon
                    / if danger_from_party { 40 } else { 10 }),
            ),
            ("army advantage", -(advantage / 2)),
            // A stronger army loses its chance to press this war for the truce window,
            // even if a Peace plan was not selected. A coalition's former target is no
            // longer an offensive goal.
            (
                "truce forgoes pressing",
                -(if disengage_coalition || dead.is_some() {
                    0
                } else {
                    advantage.max(0)
                } * truce.min(horizon)
                    / horizon
                    / 2),
            ),
            ("aggression", -(self.knobs.aggression / 5)),
            (
                "campaign withdrawn",
                if self.campaign_withdraw.is_empty() {
                    0
                } else if self.decisions.peace_withdraw_price > 0 {
                    self.ctx
                        .own_units
                        .iter()
                        .filter(|u| u.military)
                        .map(|u| (100 - u.hp).max(0) / 4)
                        .sum::<i32>()
                } else {
                    100
                },
            ),
        ];
        // A separate Peace removes one army from a coalition without ending the remaining war.
        // Keep the value small: the surviving ally still threatens us.
        if self.decisions.peace_pair_scope > 0
            && v.diplomacy.relations.iter().any(|r| {
                r.state == RelationState::Alliance
                    && (r.a == other || r.b == other)
                    && v.teams_at_war(v.team, if r.a == other { r.b } else { r.a })
            })
        {
            terms.push(("separate alliance peace", 15));
        }
        // Gene `dead_war_peace`: a war nobody fights is worth ending, the more so while a
        // rival leader is the war that matters.
        if let Some(turns) = dead {
            terms.push(("dead war", turns));
            terms.push(("rival leader", self.rival_leader_pull(other)));
        }
        terms.retain(|(_, x)| *x != 0);
        terms
    }

    /// Sparta policy: the Iron Guard war (`sparta_iron_target`). It never licenses a
    /// betrayal, hidden path, or abandoned home defense.
    pub(crate) fn sparta_expedition_target(&self) -> Option<TeamId> {
        if self.faction != legatum_core::Faction::Sparta {
            return None;
        }
        if self.decisions.sparta_valued_expedition > 0 {
            return self.routed_plan().and_then(|plan| {
                (plan.value > 0 && matches!(plan.kind, crate::plan::PlanKind::Campaign { .. }))
                    .then(|| plan.kind.war_target())
                    .flatten()
            });
        }
        self.sparta_iron_target()
    }

    pub(crate) fn sparta_iron_target(&self) -> Option<TeamId> {
        let v = self.view();
        if self.faction != legatum_core::Faction::Sparta
            || self.endangered()
            || self.ctx.own_cities.is_empty()
        {
            return None;
        }
        let army: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != legatum_core::UnitKind::Scout
                    && u.hp >= self.decisions.sparta_expedition_min_hp
            })
            .collect();
        if army.len() < 2 || !army.iter().any(|u| crate::composition::charges(u.kind)) {
            return None;
        }
        let early = v.turn <= legatum_core::data::FIRST_THIRD_TURNS;
        v.cities
            .iter()
            .filter(|c| {
                v.team_of(c.owner) != v.team
                    && v.players
                        .iter()
                        .any(|p| p.team == v.team_of(c.owner) && p.alive)
                    && (!early || !legatum_core::city_fires(c))
            })
            .filter_map(|city| {
                let team = v.team_of(city.owner);
                let walled = legatum_core::city_fires(city);
                if walled && (army.len() < 3 || strength(v, v.team) * 100 < strength(v, team) * 110)
                {
                    return None;
                }
                if !v.teams_at_war(v.team, team)
                    && (!v.diplomacy.actions.iter().any(|a| {
                        a.order
                            == Order::DeclareWar {
                                against: team,
                                breach: false,
                            }
                            && a.blocked_reason.is_none()
                    }) || strength(v, v.team) * 100 < strength(v, team) * 75)
                {
                    return None;
                }
                let distance = self
                    .ctx
                    .own_cities
                    .iter()
                    .map(|c| c.pos.distance(city.pos))
                    .min()?;
                let mut projected = v.clone();
                let relation = projected
                    .diplomacy
                    .relations
                    .iter_mut()
                    .find(|r| r.between(v.team, team))?;
                relation.state = RelationState::War;
                (distance <= 14
                    && army
                        .iter()
                        .filter(|u| {
                            projected
                                .movement_path(u.owner, u.kind, u.pos, city.pos)
                                .is_some()
                        })
                        .count()
                        >= 2)
                    .then_some((!v.teams_at_war(v.team, team), walled, distance, team))
            })
            .min()
            .map(|(_, _, _, team)| team)
    }

    pub(crate) fn reachable_war_target(&self, enemy: TeamId) -> bool {
        let mut projected = self.view().clone();
        let own = projected.team;
        if let Some(r) = projected
            .diplomacy
            .relations
            .iter_mut()
            .find(|r| r.between(own, enemy))
        {
            r.state = RelationState::War;
        }
        // Reachability uses observed terrain, closed neutral borders, and our actual units.
        projected
            .units
            .iter()
            .filter(|u| projected.team_of(u.owner) == own && u.military)
            .any(|u| {
                projected
                    .cities
                    .iter()
                    .filter(|c| projected.team_of(c.owner) == enemy)
                    .any(|c| {
                        projected
                            .movement_path(u.owner, u.kind, u.pos, c.pos)
                            .is_some()
                    })
            })
    }

    /// Gene `peace_own_danger`: the danger a Peace with `team` would actually remove. Level 1:
    /// Hard's local test of `endangered`, counting only `team`'s units (its army within five
    /// hexes of a city outweighs our units within two and half the city). Level 2: that army must outweigh everything of ours within the same five hexes and
    /// half the city: one shooter beside an empty town, while our army fights that same enemy
    /// nearby, is the war we chose, not a reason to end it. A city's low HP is no danger by
    /// itself: it already weakens `city_power`, and a city we just took and wore down ourselves
    /// is safe while the army that holds it outweighs the enemy beside it.
    pub(crate) fn endangered_by(&self, team: TeamId) -> bool {
        let v = self.view();
        let wide = self.decisions.peace_own_danger >= 2;
        v.teams_at_war(v.team, team)
            && self.ctx.own_cities.iter().any(|c| {
                let near = |e: &legatum_core::view::UnitView| {
                    e.military
                        && !e.owner.is_barbarian()
                        && v.team_of(e.owner) == team
                        && e.pos.distance(c.pos) <= 5
                        && self.ctx.reachable(e.pos, c.pos)
                };
                let pressure: i32 = self
                    .ctx
                    .enemy_units
                    .iter()
                    .filter(|e| near(e))
                    .map(|e| crate::analysis::threat_power(v, e))
                    .sum();
                if pressure == 0 {
                    return false;
                }
                if !wide {
                    return pressure
                        > self.local_defense(c) + crate::analysis::city_power(v, c) / 2;
                }
                let ours: i32 = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 5
                    })
                    .map(|u| {
                        crate::analysis::defense_power(v, u)
                            .max(crate::analysis::attack_power(v, u))
                    })
                    .sum();
                pressure > ours + crate::analysis::city_power(v, c) / 2
            })
    }

    /// The army of `against` stands within three hexes of our original capital: Peace with it is
    /// real relief, and no siege elsewhere may refuse it.
    fn capital_pressed_by(&self, against: TeamId) -> bool {
        let v = self.view();
        self.ctx.own_cities.iter().any(|home| {
            self.own_original_capital(home)
                && self.ctx.enemy_units.iter().any(|unit| {
                    unit.military
                        && v.team_of(unit.owner) == against
                        && unit.pos.distance(home.pos) <= 3
                })
        })
    }

    /// Gene `peace_prize`: the prize in reach, priced against Peace. The best enemy city (a
    /// remembered sighting counts, as in `plan.rs::campaign_forecast`) that the forces already
    /// in place take within the ten-turn truce, at the campaign planner's own price. Unlike
    /// `peace_siege_cost` it needs no visible city and no committed siege.
    fn peace_prize_cost(&self, against: TeamId) -> i32 {
        let weight = self.decisions.peace_prize;
        let v = self.view();
        if weight == 0 || !v.teams_at_war(v.team, against) || self.capital_pressed_by(against) {
            return 0;
        }
        let base = crate::winchance::WinFacts::of(v)
            .with_relative(&self.decisions)
            .refined();
        let c0 = base.chance();
        v.cities
            .iter()
            .filter(|city| v.team_of(city.owner) == against)
            .filter_map(|city| {
                let forecast = self.campaign_forecast(city, None)?;
                (forecast.march == 0
                    && forecast.turns >= 0
                    && forecast.turns <= legatum_core::diplomacy::PEACE_TRUCE_TURNS as i32)
                    .then(|| self.price_campaign(&base, c0, city, &forecast).value.max(0))
            })
            .max()
            .map_or(0, |value| (value * weight / 250).min(200))
    }

    /// The opportunity that the truce would close. Use only visible cities and our own
    /// observed force. A local siege needs a favorable capture forecast within the ten-turn
    /// truce; a badly damaged original capital also has option value while an attacker is in
    /// reach, even if its current force lacks the melee unit needed for a capture forecast.
    fn peace_siege_cost(&self, against: TeamId) -> i32 {
        let weight = self.decisions.peace_siege_value;
        if weight == 0 || !self.view().teams_at_war(self.view().team, against) {
            return 0;
        }
        let v = self.view();
        if self.capital_pressed_by(against) {
            return 0;
        }
        self.ctx
            .enemy_cities
            .iter()
            .filter(|city| city.visible && v.team_of(city.owner) == against)
            .filter_map(|city| {
                let force = self.campaign_force(city);
                let siege = self
                    .campaign_siege_estimate(city, &force)
                    .is_some_and(|(turns, _)| {
                        turns >= 0
                            && turns <= legatum_core::diplomacy::PEACE_TRUCE_TURNS as i32
                            && self.campaign_deadline().is_none_or(|deadline| {
                                deadline > v.turn.saturating_add(turns as u32)
                            })
                    });
                let capital_attack = self.capital_capture_vp(city) > 0
                    && city.hp * 4 <= city.max_hp
                    && self.ctx.own_units.iter().any(|unit| {
                        unit.military && unit.hp >= 30 && unit.pos.distance(city.pos) <= 4
                    });
                if !siege && !capital_attack {
                    return None;
                }
                let value = self.campaign_value(city, &force).max(0);
                (value > 0).then_some((value * weight / 250).min(200))
            })
            .max()
            .unwrap_or(0)
    }

    /// Gene `peace_own_danger`: the escape valve. A city of ours at half HP or less with enemies
    /// at war beside it that outweigh what holds it is about to fall, whoever's war it is. A
    /// city we just took and wore down ourselves, held by an army that outweighs them, is not.
    pub(crate) fn collapsing(&self) -> bool {
        let v = self.view();
        self.ctx.own_cities.iter().any(|c| {
            c.hp * 2 <= c.max_hp
                && self
                    .ctx
                    .enemy_units
                    .iter()
                    .filter(|e| {
                        e.military
                            && !e.owner.is_barbarian()
                            && v.at_war(v.me, e.owner)
                            && e.pos.distance(c.pos) <= 2
                    })
                    .map(|e| crate::analysis::threat_power(v, e))
                    .sum::<i32>()
                    > self.local_defense(c)
        })
    }

    /// Diagnostics (`PeaceDiagnosis::danger`): the cities `endangered` weighs.
    fn danger_cities(&self, against: TeamId) -> Vec<(u32, i32, i32, i32, i32, i32, bool)> {
        let v = self.view();
        self.ctx
            .own_cities
            .iter()
            .filter_map(|c| {
                let near = |e: &legatum_core::view::UnitView| {
                    e.military
                        && !e.owner.is_barbarian()
                        && e.pos.distance(c.pos) <= 5
                        && self.ctx.reachable(e.pos, c.pos)
                };
                let pressure: i32 = self
                    .ctx
                    .enemy_units
                    .iter()
                    .filter(|e| near(e))
                    .map(|e| crate::analysis::threat_power(v, e))
                    .sum();
                let theirs: i32 = self
                    .ctx
                    .enemy_units
                    .iter()
                    .filter(|e| near(e) && v.team_of(e.owner) == against)
                    .map(|e| crate::analysis::threat_power(v, e))
                    .sum();
                (pressure > 0).then(|| {
                    (
                        c.id.0,
                        c.hp,
                        c.max_hp,
                        pressure,
                        self.local_defense(c) + crate::analysis::city_power(v, c) / 2,
                        theirs,
                        c.detail.as_ref().is_some_and(|d| d.resistance > 0),
                    )
                })
            })
            .collect()
    }

    pub(crate) fn endangered(&self) -> bool {
        let v = self.view();
        let enemies: Vec<_> = v
            .standings
            .iter()
            .filter(|s| {
                v.teams_at_war(v.team, s.team)
                    && v.players.iter().any(|p| p.team == s.team && p.alive)
            })
            .collect();
        if self.hard_defense() {
            // A stronger rival on another front is the reason to cooperate, not
            // evidence that our own cities are about to fall. Use visible local
            // pressure when deciding whether survival requires a separate peace.
            return !enemies.is_empty()
                && self.ctx.own_cities.iter().any(|c| {
                    let pressure: i32 = self
                        .ctx
                        .enemy_units
                        .iter()
                        .filter(|e| {
                            e.military
                                && !e.owner.is_barbarian()
                                && e.pos.distance(c.pos) <= 5
                                && self.ctx.reachable(e.pos, c.pos)
                        })
                        .map(|e| crate::analysis::threat_power(v, e))
                        .sum();
                    pressure > 0
                        && (c.hp <= 100
                            || pressure
                                > self.local_defense(c) + crate::analysis::city_power(v, c) / 2)
                });
        }
        !enemies.is_empty()
            && (v
                .cities
                .iter()
                .any(|c| v.team_of(c.owner) == v.team && c.hp <= 100)
                || strength(v, v.team) * 100
                    < enemies
                        .iter()
                        .map(|s| strength(v, s.team))
                        .max()
                        .unwrap_or(0)
                        * 80)
    }

    /// Project permissions for this pair only. Reserve home defenders exactly as the
    /// military planner does; an army committed to garrisons is not an invasion force.
    pub(crate) fn diplomatic_campaign(&self, against: TeamId) -> Option<Hex> {
        // Fast rollout policy: no prospective war planners.
        if self.fast >= 1 {
            return None;
        }
        // Pure for the pass (view, knobs and memoized conquest city): computed once per team.
        if let Some(hit) = self.diplomatic_campaigns.borrow().get(&against) {
            return *hit;
        }
        let out = self.diplomatic_campaign_uncached(against);
        self.diplomatic_campaigns.borrow_mut().insert(against, out);
        out
    }

    fn diplomatic_campaign_uncached(&self, against: TeamId) -> Option<Hex> {
        // Cases where the prospective planner's `campaign_target_for` must return None,
        // checked before building it: a faction that never sieges, no known city of theirs,
        // no city of ours, or no possible army (it marches only healthy military units, at
        // least two, one of them melee: `available_campaign`).
        let v = self.view();
        let marching = || {
            self.ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout && u.hp >= self.march_min_hp())
        };
        if self.decisions.campaign_contain_only > 0
            || self.ctx.own_cities.is_empty()
            || !v
                .cities
                .iter()
                .any(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == against)
            || marching().count() < 2
            || !marching().any(|u| crate::composition::charges(u.kind))
        {
            return None;
        }
        if self.conquest_city().is_some()
            && !self.view().teams_at_war(self.view().team, against)
            && self.diplomatic_target() != Some(against)
            && !self.conquest_ready(against)
        {
            return None;
        }
        let mut prospective = self.view().clone();
        prospective
            .diplomacy
            .relations
            .iter_mut()
            .find(|r| r.between(self.view().team, against))?
            .state = RelationState::War;
        let mut planner =
            Planner::prospective(&prospective, &self.ctx, self.knobs, self.faction, self.seed);
        // Rivals other than `against` keep their relation with us, so their reach is ours.
        planner.shared_routes = Some(self.shared_routes.unwrap_or(&self.declaration_routes));
        planner.opponents = self.opponents;
        planner.strategy = self.strategy;
        if self.hard_defense() {
            planner.difficulty = self.difficulty;
            planner.prepare_hard_defense();
        }
        let target = planner.available_campaign(against)?;
        if self.hard_defense()
            && self
                .view()
                .cities
                .iter()
                .any(|c| c.pos == target && legatum_core::city_fires(c))
            && self.diplomatic_target() != Some(against)
            && !self.view().teams_at_war(self.view().team, against)
        {
            // A nominal strength advantage does not fund a siege. Starting an
            // optional war with only infantry pins that army outside city fire
            // and leaves another neighbour free to attack home. Retaliation,
            // containment and the separately planned Sparta expedition keep
            // their own urgency and legality checks.
            let shooters = planner
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    crate::composition::shooter_only(u.kind, u.ranged)
                        && u.hp >= self.decisions.recover_hp
                        && !planner.has_acted(u.id)
                        && planner.ctx.reachable(u.pos, target)
                })
                .count();
            if shooters < 2 {
                return None;
            }
        }
        Some(target)
    }

    pub(crate) fn wants_peace(&self, against: TeamId) -> bool {
        self.wants_peace_why(against).0
    }

    /// `wants_peace` with the gate that decided it (diagnostics: `peace_diagnosis`).
    pub(crate) fn wants_peace_why(&self, against: TeamId) -> (bool, &'static str) {
        let danger_from_party = if self.decisions.peace_pair_scope > 0 {
            self.endangered_by(against)
        } else {
            self.endangered()
        };
        // Gene `operations`: a committed strike on this team refuses Peace while it runs.
        if self.operation_refuses_peace(against) {
            return (false, "committed operation");
        }
        // India playbook bit 4: keep the war against the strict public VP leader.
        if self.faction == legatum_core::Faction::India
            && self.decisions.india_playbook & 4 != 0
            && self.strict_leader(60) == Some(against)
            && self.view().teams_at_war(self.view().team, against)
        {
            return (false, "India leader war");
        }
        // Gene `runaway_coalition` bit 1: no Peace with a runaway leader (`runaway.rs`).
        if self.runaway_refuses_peace(against) {
            return (false, "runaway leader");
        }
        if self.decisions.vp_peace == 0 {
            if crate::hold::rival_hold_of(self.view(), against).is_some() {
                return (false, "rival holds");
            }
        } else if self.vp_peace_cost(against) > self.vp_war_loss(against) {
            return (false, "VP cost of peace");
        }
        // Gene `vow_hold`: a closed country whose vow holds or scores soon wants every war over,
        // whatever the plan: it cannot start wars of its own and can protect its cities.
        if self.vow_at_stake() {
            return (true, "vow at stake");
        }
        let siege_cost = self
            .peace_siege_cost(against)
            .max(self.peace_prize_cost(against));
        if siege_cost > 0 {
            let v = self.view();
            let age = v
                .diplomacy
                .relations
                .iter()
                .find(|r| r.between(v.team, against))
                .map_or(0, |r| v.turn.saturating_sub(r.effective_turn));
            let advantage =
                (strength(v, v.team) * 100 / strength(v, against).max(1) - 100).clamp(-50, 50);
            let relief: i32 = self
                .peace_terms(against, age, advantage, coalition_disengage(v, against))
                .iter()
                .map(|(_, value)| value)
                .sum();
            if siege_cost > relief.max(0) {
                return (false, "siege worth more than peace");
            }
        }
        if self.view().cities.iter().any(|city| {
            self.view().team_of(city.owner) == against
                && (self.sparta_open_followup(city) || self.sparta_chain_target(city))
        }) {
            return (false, "Sparta follow-up city");
        }
        // Gene `peace_redeclare`: not a Peace the next campaign would break (`plan.rs`).
        // Gene `dead_war_peace`: no plan would declare a dead war again.
        if self.routed_plan().is_some()
            && self.peace_would_be_broken(against)
            && self.dead_war(against).is_none()
        {
            return (false, "peace would be broken");
        }
        if !danger_from_party {
            let continuation = self.leader_peace_continuation_value(against);
            let recovery: i32 = 25
                + self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military)
                    .map(|u| (100 - u.hp).max(0) / 4)
                    .sum::<i32>();
            if continuation > recovery {
                return (false, "leader campaign continues");
            }
        }
        if self.decisions.rival_redeclare_percent > 0 && !danger_from_party {
            let risk = self.rival_redeclare_permille(against)
                * (self.plan_horizon() - legatum_core::diplomacy::PEACE_TRUCE_TURNS as i32).max(0)
                / self.plan_horizon()
                / 10;
            let healing: i32 = self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military)
                .map(|u| (100 - u.hp).max(0) / 4)
                .sum();
            // A rival already at our city makes the current war costly too. Price that
            // pressure against its chance of using Peace as a healing window.
            let home_pressure = self
                .ctx
                .enemy_units
                .iter()
                .filter(|u| {
                    u.military
                        && self.view().team_of(u.owner) == against
                        && self
                            .ctx
                            .own_cities
                            .iter()
                            .any(|c| c.pos.distance(u.pos) <= 3)
                })
                .count() as i32
                * 15
                + self.ctx.own_cities.iter().filter(|c| c.hp <= 100).count() as i32 * 20;
            if risk > 25 + healing + home_pressure {
                return (false, "rival would redeclare");
            }
        }
        // Hard's plan decides: it ends every war except the one it fights (`plan.rs`).
        if let Some(plan) = self.routed_plan() {
            // Gene `peace_own_danger`: toward the enemy the plan fights, pressure from a third
            // war is no reason for this Peace (it would not remove that pressure).
            if self.decisions.peace_own_danger > 0
                && self.hard_defense()
                && plan.kind.war_target() == Some(against)
                && self.persists_against(against)
            {
                return (
                    self.endangered_by(against) || self.collapsing(),
                    "plan fights this enemy (own danger)",
                );
            }
            return (
                danger_from_party || plan.kind.war_target() != Some(against),
                if plan.kind.war_target() == Some(against) {
                    "plan fights this enemy"
                } else {
                    "plan fights elsewhere"
                },
            );
        }
        if self.campaigns.active.as_ref().is_some_and(|campaign| {
            self.view().cities.iter().any(|c| {
                c.id == campaign.target
                    && self.view().team_of(c.owner) == against
                    && campaign.expected_value > 0
            })
        }) {
            return (false, "active campaign");
        }
        if !self.campaign_withdraw.is_empty() {
            return (true, "campaign withdrawn");
        }
        // A locally forecastable city capture is a live war objective even when an
        // opening or tactical pass has not yet recorded its campaign. Do not let a
        // different diplomacy score end that favorable siege.
        if !danger_from_party
            && self.ctx.enemy_cities.iter().any(|city| {
                if !city.visible || self.view().team_of(city.owner) != against {
                    return false;
                }
                let committed = self.campaign_force(city);
                self.campaign_siege_estimate(city, &committed)
                    .is_some_and(|(turns, _)| {
                        self.campaign_value(city, &committed) > 0
                            && self.campaign_deadline().is_none_or(|deadline| {
                                deadline > self.view().turn.saturating_add(turns.max(0) as u32)
                            })
                    })
            })
        {
            return (false, "forecast capture");
        }
        // A conqueror's damaged new city is a counterattack opportunity. Do not let a
        // routine peace offer restore its city HP while our local force can retake it.
        if self.difficulty == legatum_core::AiDifficulty::Hard
            && self.ctx.enemy_cities.iter().any(|city| {
                city.visible
                    && self.view().team_of(city.owner) == against
                    && city.hp * 2 <= city.max_hp
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(city.pos) <= 6)
                    && self.ctx.own_units.iter().any(|u| {
                        u.military
                            && crate::composition::charges(u.kind)
                            && u.hp >= 50
                            && u.pos.distance(city.pos) <= 3
                    })
                    && self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| u.military && u.hp >= 50 && u.pos.distance(city.pos) <= 3)
                        .count()
                        >= 2
            })
        {
            return (false, "counterattack on damaged city");
        }
        if (self.persist_conquest(against) || self.strike_against(against).is_some())
            && !danger_from_party
        {
            return (false, "persist conquest or strike");
        }
        if self.hard_defense() && self.diplomatic_target() == Some(against) && !danger_from_party {
            return (false, "diplomatic target");
        }
        if self.primitive_persist_siege(against) {
            return (false, "persist siege");
        }
        let v = self.view();
        let age = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, against))
            .map_or(0, |r| v.turn.saturating_sub(r.effective_turn));
        (
            danger_from_party
                || coalition_disengage(v, against)
                || (age
                    >= if self.strategy == Some(crate::Strategy::Rush) {
                        48
                    } else {
                        8
                    }
                    && strength(v, v.team) * 100 / strength(v, against).max(1) < 130),
            "no plan: age and strength",
        )
    }

    /// Public evidence that a signed Peace will be used as a short healing window.
    pub(crate) fn rival_redeclare_permille(&self, other: TeamId) -> i32 {
        let v = self.view();
        let declared_us = v
            .diplomacy
            .relations
            .iter()
            .any(|r| r.between(v.team, other) && r.declarers.contains(&other));
        let past: u32 = self.opponents.map_or(0, |model| {
            model
                .rivals
                .values()
                .filter(|r| v.team_of(r.id) == other)
                .map(|r| r.declared_wars)
                .sum()
        });
        let at_border = v
            .units
            .iter()
            .filter(|u| {
                u.military
                    && v.team_of(u.owner) == other
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|c| c.pos.distance(u.pos) <= 6)
            })
            .count() as i32;
        let evidence =
            (150 * past.min(3) as i32 + 250 * i32::from(declared_us) + 150 * at_border.min(3))
                .min(900);
        evidence * self.decisions.rival_redeclare_percent / 100
    }

    pub(crate) fn leader_peace_continuation_value(&self, other: TeamId) -> i32 {
        if self.decisions.leader_peace_horizon == 0
            || self.strict_leader(90) != Some(other)
            || !self.view().teams_at_war(self.view().team, other)
        {
            return 0;
        }
        let invested = self
            .campaigns
            .active
            .as_ref()
            .filter(|c| {
                self.view()
                    .cities
                    .iter()
                    .any(|city| city.id == c.target && self.view().team_of(city.owner) == other)
            })
            .map_or(0, |c| c.expected_value.max(0));
        let forecast = self
            .campaign_against(other)
            .ok()
            .filter(|o| o.turns + o.march <= 10)
            .map_or(0, |o| o.odds);
        invested.max(forecast)
    }

    /// Units in newly closed borders take a legal step toward an exit before other planning.
    pub fn plan_egress(&mut self) {
        let v = self.view();
        let trapped: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| !v.passage_allowed(u.owner, u.kind, v.tile(u.pos).and_then(|t| t.owner)))
            .collect();
        if trapped.is_empty() {
            return;
        }
        let mut map =
            legatum_core::state::Map::new(v.width, v.height, legatum_core::Terrain::Mountains);
        for (i, t) in v.tiles.iter().enumerate() {
            if let Some(terrain) = t.terrain {
                map.tiles[i].terrain = terrain;
            }
        }
        for u in trapped {
            let open = |h| {
                v.tile(h)
                    .is_some_and(|t| v.passage_allowed(v.me, u.kind, t.owner))
            };
            let exits = v.exit_distances(&map, open);
            let destination = u
                .pos
                .neighbors()
                .into_iter()
                .filter(|h| map.move_cost(*h).is_some())
                .filter(|h| legatum_core::path::border_step(&map, u.pos, *h, open, Some(&exits)))
                .filter(|h| {
                    !v.units.iter().any(|other| {
                        other.pos == *h
                            && other.id != u.id
                            && (!v.same_team(u.owner, other.owner) || other.military == u.military)
                    })
                })
                .min_by_key(|h| (exits[map.index(*h).expect("on map")], *h));
            self.unit_order(
                u.id,
                destination.map_or(Order::SkipUnit { unit: u.id }, |to| Order::MoveUnit {
                    unit: u.id,
                    to,
                }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::view::TeamStanding;
    use legatum_core::{Faction, PlayerId, UnitKind};

    #[test]
    fn a_damaged_capital_attack_outweighs_peace_but_own_capital_danger_allows_relief() {
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 6), 4)
            .city(2, ENEMY, off(10, 6), 4)
            .unit(10, ME, UnitKind::Ranger, off(8, 6))
            .build();
        v.turn = 40;
        v.diplomacy.legacy = false;
        let enemy = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: enemy,
                state: RelationState::War,
                effective_turn: 20,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![v.team],
            });
        let target = v
            .cities
            .iter_mut()
            .find(|city| city.id == legatum_core::CityId(2))
            .unwrap();
        target.hp = target.max_hp / 8;
        target.original_capital = true;
        target.original_capital_of = Some(ENEMY);
        let answer = |view: &PlayerView, weight| {
            let mut planner = Planner::new(
                view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            planner.difficulty = legatum_core::AiDifficulty::Hard;
            planner.plan_routing = true;
            planner.decisions.peace_siege_value = weight;
            planner.decisions.vp_peace = 0;
            planner.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Peace { with: enemy },
                start_turn: 39,
                review_turn: 45,
                reserve: 0,
                value: 0,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            (
                planner.peace_siege_cost(enemy),
                planner.wants_peace_why(enemy),
            )
        };
        assert!(answer(&v, 0).1.0, "gene off follows the Peace plan");
        let (cost, decision) = answer(&v, 200);
        assert!(cost > 0, "visible damaged capital is worth pressing");
        assert!(!decision.0, "{decision:?}, cost {cost}");

        let home = v
            .cities
            .iter_mut()
            .find(|city| city.id == legatum_core::CityId(1))
            .unwrap();
        home.original_capital = true;
        home.original_capital_of = Some(ME);
        home.hp = home.max_hp / 2;
        v.units.push(
            ViewBuilder::new(24, 16)
                .unit(50, ENEMY, UnitKind::Soldier, off(5, 6))
                .build()
                .units
                .pop()
                .unwrap(),
        );
        let (cost, decision) = answer(&v, 200);
        assert_eq!(cost, 0);
        assert!(
            decision.0,
            "dangerous war should still accept relief: {decision:?}"
        );
    }

    #[test]
    fn a_remembered_city_in_reach_of_our_army_is_a_prize_worth_more_than_peace() {
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 6), 4)
            .city(2, ENEMY, off(10, 6), 3)
            .unit(10, ME, UnitKind::Soldier, off(8, 6))
            .unit(11, ME, UnitKind::Soldier, off(8, 7))
            .unit(12, ME, UnitKind::Ranger, off(8, 5))
            .unit(13, ME, UnitKind::Ranger, off(7, 6))
            .unit(14, ME, UnitKind::Soldier, off(8, 8))
            .build();
        v.turn = 40;
        v.diplomacy.legacy = false;
        let enemy = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: enemy,
                state: RelationState::War,
                effective_turn: 20,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![v.team],
            });
        // Seen once and fogged since: a sighting, not a visible siege.
        v.cities
            .iter_mut()
            .find(|city| city.id == legatum_core::CityId(2))
            .unwrap()
            .visible = false;
        let answer = |weight| {
            let mut planner = Planner::new(
                &v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            planner.difficulty = legatum_core::AiDifficulty::Hard;
            planner.plan_routing = true;
            planner.decisions.peace_prize = weight;
            planner.decisions.vp_peace = 0;
            planner.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Peace { with: enemy },
                start_turn: 39,
                review_turn: 45,
                reserve: 0,
                value: 0,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            (planner.peace_prize_cost(enemy), planner.wants_peace(enemy))
        };
        assert_eq!(answer(0), (0, true), "gene off follows the Peace plan");
        let (cost, wants) = answer(100);
        assert!(cost > 0, "the army in place takes the remembered city");
        assert!(!wants, "cost {cost} outweighs the Peace terms");
    }

    #[test]
    fn pre_hold_leader_cannot_buy_off_an_active_valuable_siege() {
        use crate::campaign::Campaign;
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 6), 4)
            .city(2, ENEMY, off(10, 6), 4)
            .unit(10, ME, UnitKind::Soldier, off(8, 6))
            .unit(11, ME, UnitKind::Soldier, off(8, 7))
            .build();
        v.turn = 40;
        let rival = v.team_of(ENEMY);
        v.diplomacy.legacy = false;
        v.standings = [(v.team, 3), (rival, 9)]
            .map(|(team, vp)| TeamStanding {
                team,
                members: vec![if team == v.team { ME } else { ENEMY }],
                vp,
                target: 10,
                victory_hold: None,
            })
            .to_vec()
            .into();
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: rival,
                state: RelationState::War,
                effective_turn: 20,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![v.team],
            });
        let wants_peace = |gene| {
            let mut p = Planner::new(
                &v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.leader_peace_horizon = gene;
            p.decisions.rival_redeclare_percent = 0;
            // This test prices `leader_peace_horizon` alone.
            p.decisions.runaway_coalition = 0;
            p.decisions.vp_peace = 0;
            p.campaigns.active = Some(Campaign {
                target: legatum_core::CityId(2),
                start_turn: 35,
                committed: vec![legatum_core::UnitId(10), legatum_core::UnitId(11)],
                deadline: None,
                expected_value: 100,
                target_was_unwalled: true,
            });
            p.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Peace { with: rival },
                start_turn: 39,
                review_turn: 45,
                reserve: 0,
                value: 0,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            assert!(!p.endangered());
            p.wants_peace(rival)
        };
        assert!(wants_peace(0));
        assert!(!wants_peace(1));
    }

    #[test]
    fn a_capture_hold_keeps_the_war_that_declaring_again_would_cost_more_than_it_risks() {
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::Mongols)
            .city(1, ME, off(4, 6), 6)
            .city(2, ENEMY, off(14, 6), 3)
            .build();
        v.turn = 120;
        v.diplomacy.legacy = false;
        crate::fixture::set_armies(&mut v, &[(ME, 300), (ENEMY, 40)]);
        let against = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: against,
                state: RelationState::War,
                effective_turn: 100,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![v.team],
            });
        let wants_peace = |gene: i32, hold: bool| {
            let mut p = Planner::new(
                &v,
                Knobs::for_faction(Faction::Mongols),
                Faction::Mongols,
                1,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.peace_redeclare = gene;
            p.decisions.dead_war_peace = 0;
            p.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Peace { with: against },
                start_turn: 119,
                review_turn: 130,
                reserve: 0,
                value: 0,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            if hold {
                p.campaigns.held_capture = Some((legatum_core::CityId(2), Vec::new()));
            }
            assert!(!p.endangered());
            p.wants_peace(against)
        };
        // No hold: the quiet plan's Peace goes through, with the gene on or off.
        assert!(wants_peace(0, false) && wants_peace(2, false));
        // Holding a captured city: the gene keeps the war; off, the Peace is signed.
        assert!(wants_peace(0, true));
        assert!(!wants_peace(2, true));
    }

    #[test]
    fn a_closed_country_with_its_vow_at_stake_accepts_peace_whatever_its_plan_fights() {
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(4, 6), 6)
            .city(2, ENEMY, off(14, 6), 3)
            .build();
        v.turn = 190;
        v.diplomacy.legacy = false;
        crate::fixture::set_armies(&mut v, &[(ME, 300), (ENEMY, 40)]);
        v.actions.sakoku = Some(legatum_core::rules::actions::SakokuView {
            closed: true,
            turns: 27,
            target: 30,
            holding: false,
            blocker: None,
        });
        let against = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: against,
                state: RelationState::War,
                effective_turn: 150,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![against],
            });
        let wants_peace = |gene: i32| {
            let mut p = Planner::new(&v, Knobs::for_faction(Faction::India), Faction::India, 1);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.vow_hold = gene;
            p.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Campaign {
                    against,
                    target: legatum_core::CityId(2),
                },
                start_turn: 189,
                review_turn: 200,
                reserve: 0,
                value: 500,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            p.wants_peace(against)
        };
        assert!(!wants_peace(0), "the plan fights this war");
        assert!(wants_peace(1), "three turns from the vow's VP: end it");
    }

    #[test]
    fn peace_prices_a_rival_who_declared_and_kept_an_army_at_our_border() {
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 6), 4)
            .city(2, ENEMY, off(13, 6), 4)
            .unit(10, ME, UnitKind::Soldier, off(4, 6))
            .unit(11, ME, UnitKind::Soldier, off(5, 6))
            .unit(12, ME, UnitKind::Soldier, off(5, 7))
            .unit(20, ENEMY, UnitKind::Soldier, off(8, 6))
            .unit(21, ENEMY, UnitKind::Soldier, off(8, 7))
            .build();
        v.turn = 40;
        let rival = v.team_of(ENEMY);
        v.diplomacy.legacy = false;
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: rival,
                state: RelationState::War,
                effective_turn: 20,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![rival],
            });
        let wants_peace = |gene| {
            let mut p = Planner::new(
                &v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.rival_redeclare_percent = gene;
            p.campaigns.plan.active = Some(Plan {
                kind: PlanKind::Peace { with: rival },
                start_turn: 39,
                review_turn: 45,
                reserve: 0,
                value: 0,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            assert!(!p.endangered());
            p.wants_peace(rival)
        };
        assert!(wants_peace(0));
        assert!(
            wants_peace(100),
            "a ten-turn truce removes the rival's re-declaration risk from this plan"
        );
        let mut p = Planner::new(
            &v,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            1,
        );
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.decisions.rival_redeclare_percent = 0;
        p.campaign_withdraw = vec![legatum_core::UnitId(10)];
        let order = Order::ProposeTreaty {
            to: rival,
            kind: TreatyKind::Peace,
        };
        p.decisions.peace_withdraw_price = 0;
        let flat = p
            .treaty_score(rival, TreatyKind::Peace, &order, &TreatyBudget::of(&v))
            .unwrap();
        p.decisions.peace_withdraw_price = 1;
        let healed = p
            .treaty_score(rival, TreatyKind::Peace, &order, &TreatyBudget::of(&v))
            .unwrap();
        assert_eq!(
            flat - healed,
            100,
            "a healthy withdrawn army needs no flat Peace prize"
        );
    }

    #[test]
    fn a_peace_plan_accepts_the_peace_the_score_would_refuse() {
        use crate::plan::{Plan, PlanKind};
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(14, 6), 3)
            .build();
        crate::fixture::set_armies(&mut v, &[(ME, 200), (ENEMY, 20)]);
        let enemy = v.team_of(ENEMY);
        let peace = Order::ProposeTreaty {
            to: enemy,
            kind: TreatyKind::Peace,
        };
        let score = |plan: Option<PlanKind>, gene: i32| {
            let mut p = Planner::new(&v, Knobs::new(50, 50, 50, 50, 100), Faction::Celts, 1);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.peace_follows_plan = gene;
            p.campaigns.plan.searched = vec![PlanKind::Economy];
            p.campaigns.plan.active = plan.map(|kind| Plan {
                kind,
                start_turn: 1,
                review_turn: 10,
                reserve: 0,
                value: 0,
                terms: Vec::new(),
                breach: false,
                misses: 0,
            });
            p.treaty_score(enemy, TreatyKind::Peace, &peace, &TreatyBudget::of(&v))
        };
        let negative = score(Some(PlanKind::Economy), 1).expect("economy plan wants peace");
        assert!(
            negative < 0,
            "a far stronger army has little use for peace: {negative}"
        );
        assert_eq!(score(Some(PlanKind::Peace { with: enemy }), 1), Some(0));
        assert_eq!(
            score(Some(PlanKind::Peace { with: enemy }), 0),
            Some(negative),
            "the gene off restores the old score"
        );
        assert_eq!(score(Some(PlanKind::Contain { against: enemy }), 1), None);
    }

    #[test]
    fn the_border_raid_gene_requires_a_favorable_local_field_target() {
        let v = ViewBuilder::new(24, 16)
            .faction(Faction::Germany)
            .city(1, ME, off(4, 6), 3)
            .unit(1, ME, UnitKind::Soldier, off(6, 6))
            .unit(2, ME, UnitKind::Soldier, off(6, 7))
            .unit(3, ENEMY, UnitKind::Soldier, off(8, 6))
            .build();
        let raid = |v: &PlayerView| {
            let mut planner =
                Planner::new(v, Knobs::for_faction(Faction::Germany), Faction::Germany, 1);
            planner.decisions.border_raid = 1;
            planner.border_raid_opportunity(TeamId(2))
        };
        assert!(raid(&v));
        let mut remote = v.clone();
        remote.units[2].pos = off(20, 12);
        assert!(!raid(&remote));
        remote.units[2].pos = off(8, 6);
        remote.units[1].pos = off(1, 14);
        assert!(!raid(&remote));
    }

    #[test]
    fn sparta_can_choose_a_visible_unwalled_rival_before_war() {
        let mut v = ViewBuilder::new(28, 20)
            .faction(Faction::Sparta)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(11, 6), 3)
            .unit(1, ME, UnitKind::Soldier, off(4, 6))
            .unit(2, ME, UnitKind::Soldier, off(5, 6))
            .build();
        v.diplomacy.legacy = false;
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: TeamId(2),
                state: RelationState::Peace,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: None,
                declarers: Vec::new(),
            });
        v.diplomacy
            .actions
            .push(legatum_core::diplomacy::DiplomaticAction {
                new_wars: Vec::new(),
                calls: Vec::new(),
                also_agree: Vec::new(),
                order: Order::DeclareWar {
                    against: TeamId(2),
                    breach: false,
                },
                effective_turn: v.turn,
                cost: 0,
                blocked_reason: None,
                upkeep: 0,
                betrayal: false,
            });
        crate::fixture::set_armies(&mut v, &[(ME, 80), (ENEMY, 20)]);
        let target = |v: &PlayerView| {
            Planner::new(v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1)
                .sparta_iron_target()
        };
        assert_eq!(target(&v), Some(TeamId(2)));
        v.cities[1].visible = false;
        assert_eq!(
            target(&v),
            Some(TeamId(2)),
            "known city remains a target in fog"
        );
        let walled = ViewBuilder { view: v }.walled(2).build();
        assert_eq!(target(&walled), None, "early push avoids city fire");
    }

    #[test]
    fn optional_hard_war_requires_spare_ranged_support_but_containment_does_not() {
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(11, 6), 3)
            .walled(2)
            .unit(1, ME, UnitKind::Trooper, off(4, 6))
            .unit(2, ME, UnitKind::Trooper, off(8, 6))
            .unit(3, ME, UnitKind::Trooper, off(8, 5))
            .unit(4, ME, UnitKind::Trooper, off(8, 4))
            .build();
        v.diplomacy.legacy = false;
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: TeamId(2),
                state: RelationState::Peace,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: None,
                declarers: vec![],
            });
        let target = |v: &PlayerView, difficulty| {
            let mut p = Planner::new(v, Knobs::new(50, 80, 50, 50, 100), v.faction_of(ME), 1);
            p.difficulty = difficulty;
            p.diplomatic_campaign(TeamId(2))
        };
        assert!(target(&v, legatum_core::AiDifficulty::Normal).is_some());
        assert_eq!(target(&v, legatum_core::AiDifficulty::Hard), None);
        // An unwalled city cannot fire, so infantry alone may besiege it.
        let mut open = v.clone();
        open.cities[1].walls = false;
        assert!(target(&open, legatum_core::AiDifficulty::Hard).is_some());
        let with_support = ViewBuilder { view: v.clone() }
            .unit(5, ME, UnitKind::Ranger, off(8, 7))
            .unit(6, ME, UnitKind::Ranger, off(9, 7))
            .build();
        assert!(target(&with_support, legatum_core::AiDifficulty::Hard).is_some());
        v.standings = vec![
            TeamStanding {
                team: v.team,
                members: vec![ME, PlayerId(1)],
                vp: 4,
                target: 20,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(2),
                members: vec![ENEMY],
                vp: 9,
                target: 10,
                victory_hold: None,
            },
        ]
        .into();
        assert!(
            target(&v, legatum_core::AiDifficulty::Hard).is_some(),
            "an imminent leader still warrants containment"
        );
    }

    #[test]
    fn hard_coalition_stays_against_distant_strength_but_can_save_an_overrun_home() {
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, off(4, 6), 3)
            .unit(1, ME, UnitKind::Soldier, off(4, 6))
            .build();
        v.diplomacy.legacy = false;
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: TeamId(2),
                state: RelationState::War,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![TeamId(2)],
            });
        v.standings = vec![
            TeamStanding {
                team: v.team,
                members: vec![ME, PlayerId(1)],
                vp: 4,
                target: 20,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(2),
                members: vec![ENEMY],
                vp: 9,
                target: 10,
                victory_hold: None,
            },
        ]
        .into();
        crate::fixture::set_armies(&mut v, &[(ME, 20), (ENEMY, 200)]);
        let peace = |v: &PlayerView, difficulty| {
            let mut p = Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 1);
            p.difficulty = difficulty;
            // The overrun-home escape of the ordinary gates; `runaway_coalition` refuses Peace
            // with a runaway even then (`runaway.rs` tests).
            p.decisions.runaway_coalition = 0;
            p.wants_peace(TeamId(2))
        };
        assert!(peace(&v, legatum_core::AiDifficulty::Normal));
        assert!(
            !peace(&v, legatum_core::AiDifficulty::Hard),
            "a distant army is not an overrun home"
        );
        let threatened = ViewBuilder { view: v }
            .unit(50, ENEMY, UnitKind::Trooper, off(7, 6))
            .unit(51, ENEMY, UnitKind::Trooper, off(7, 5))
            .unit(52, ENEMY, UnitKind::Trooper, off(7, 7))
            .build();
        assert!(peace(&threatened, legatum_core::AiDifficulty::Hard));
    }

    #[test]
    fn containment_uses_public_growth_and_recent_aggression_not_peaceful_expansion() {
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(10, 6), 3)
            .build();
        v.turn = 100;
        v.diplomacy.legacy = false;
        for p in &mut v.players {
            p.cities = if p.team == v.team {
                3
            } else if p.id == ENEMY {
                5
            } else {
                1
            };
        }
        v.standings = vec![
            TeamStanding {
                team: v.team,
                members: vec![ME, PlayerId(1)],
                vp: 6,
                target: 20,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(2),
                members: vec![ENEMY],
                vp: 3,
                target: 10,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(3),
                members: vec![PlayerId(3)],
                vp: 3,
                target: 10,
                victory_hold: None,
            },
        ]
        .into();
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: TeamId(2),
                b: TeamId(3),
                state: RelationState::War,
                effective_turn: 90,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: Some(1),
                declarers: vec![TeamId(2)],
            });
        let target = |v: &PlayerView| {
            let mut p = Planner::new(
                v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                1,
            );
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.diplomatic_target()
        };
        assert_eq!(coalition_target(&v), None, "no global VP alarm yet");
        assert_eq!(target(&v), Some(TeamId(2)));
        v.diplomacy.relations[0].declarers = vec![TeamId(3)];
        assert_eq!(
            target(&v),
            None,
            "growth by the defender is not evidence of aggression"
        );
        v.diplomacy.relations[0].declarers = vec![TeamId(2)];
        v.turn = 200;
        assert_eq!(target(&v), None, "old wars do not create a permanent enemy");
    }

    #[test]
    fn coalition_ties_have_no_target_and_can_select_self_without_attacking_self() {
        let mut v = ViewBuilder::new(4, 4).build();
        v.standings = vec![
            TeamStanding {
                team: TeamId(3),
                members: vec![PlayerId(3)],
                vp: 8,
                target: 10,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(2),
                members: vec![ENEMY],
                vp: 8,
                target: 10,
                victory_hold: None,
            },
            TeamStanding {
                team: TeamId(0),
                members: vec![ME],
                vp: 7,
                target: 10,
                victory_hold: None,
            },
        ]
        .into();
        // Two teams tied at 80%: a race, not a leader, so nobody is singled out by id.
        assert_eq!(coalition_target(&v), None);
        v.standings[1].vp = 9;
        assert_eq!(coalition_target(&v), Some(TeamId(2)));
        v.standings[2].vp = 9;
        assert_eq!(coalition_target(&v), None);
        v.standings[2].vp = 10;
        assert_eq!(coalition_target(&v), Some(TeamId(0)));
    }

    fn war(a: u8, b: u8) -> legatum_core::diplomacy::Relation {
        legatum_core::diplomacy::Relation {
            a: TeamId(a),
            b: TeamId(b),
            state: RelationState::War,
            effective_turn: 40,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(u32::from(a) * 10 + u32::from(b)),
            declarers: vec![TeamId(b)],
        }
    }

    /// Team 2 leads (8 of 10 VP); we are team 0 and team 3 is the potential partner.
    fn coalition_view(gold: i32, wars: &[(u8, u8)]) -> PlayerView {
        let mut v = ViewBuilder::new(4, 4).gold(gold).build();
        v.standings = [(0, 1), (2, 8), (3, 1)]
            .map(|(team, vp)| TeamStanding {
                team: TeamId(team),
                members: vec![PlayerId(team)],
                vp,
                target: 10,
                victory_hold: None,
            })
            .to_vec()
            .into();
        v.diplomacy.legacy = false;
        v.diplomacy.relations = wars.iter().map(|(a, b)| war(*a, *b)).collect();
        v
    }

    fn planner_for(v: &PlayerView) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(Faction::India), Faction::India, 1);
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p
    }

    #[test]
    fn separate_peace_prices_only_the_named_enemy_alliance_member() {
        let mut v = coalition_view(500, &[(0, 2), (0, 3)]);
        let mut alliance = war(2, 3);
        alliance.state = RelationState::Alliance;
        alliance.war_id = None;
        v.diplomacy.relations.push(alliance);
        let mut planner = planner_for(&v);
        let terms = planner.peace_terms(TeamId(2), 10, 0, false);
        assert!(terms.contains(&("separate alliance peace", 15)));
        planner.decisions.peace_pair_scope = 0;
        let legacy_terms = planner.peace_terms(TeamId(2), 10, 0, false);
        assert!(
            !legacy_terms
                .iter()
                .any(|(name, _)| *name == "separate alliance peace")
        );
    }

    #[test]
    fn indian_leader_peace_bit_only_blocks_a_war_with_the_strict_leader() {
        let mut v = coalition_view(500, &[(0, 2)]);
        let gate = |v: &PlayerView| {
            let mut p = planner_for(v);
            p.decisions.india_playbook = 4;
            p.wants_peace_why(TeamId(2)).1
        };
        assert_eq!(gate(&v), "India leader war");
        v.standings
            .iter_mut()
            .find(|s| s.team == TeamId(2))
            .unwrap()
            .vp = 5;
        assert_ne!(gate(&v), "India leader war");
        v.standings
            .iter_mut()
            .find(|s| s.team == TeamId(2))
            .unwrap()
            .vp = 8;
        v.standings
            .iter_mut()
            .find(|s| s.team == TeamId(3))
            .unwrap()
            .vp = 8;
        assert_ne!(gate(&v), "India leader war");
        v.standings
            .iter_mut()
            .find(|s| s.team == TeamId(3))
            .unwrap()
            .vp = 1;
        v.diplomacy.relations.clear();
        assert_ne!(gate(&v), "India leader war");
    }

    /// An ally's help in our war counts for what it can reach: all of it within 8 hexes of the
    /// enemy, none past 16.
    #[test]
    fn an_ally_that_cannot_reach_the_enemy_gives_little_help() {
        let reach = |ally_col: i32| {
            let v = ViewBuilder::new(30, 16)
                .city(1, ME, off(2, 6), 3)
                .city(2, ENEMY, off(20, 6), 3)
                .city(3, PlayerId(3), off(ally_col, 6), 3)
                .build();
            planner_for(&v).helper_reach_percent(TeamId(3), TeamId(2))
        };
        assert_eq!(reach(14), 100);
        assert!((1..100).contains(&reach(7)), "partial reach: {}", reach(7));
        assert_eq!(reach(2), 0);
    }

    /// Joining a war costs more with the new enemy's army at our gates, and with a campaign
    /// already under way against someone else.
    #[test]
    fn offensive_allies_are_optional_and_defensive_chains_do_not_join() {
        let mut v = ViewBuilder::new(48, 16)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(12, 6), 3)
            .city(3, PlayerId(3), off(18, 6), 3)
            .city(4, PlayerId(4), off(26, 6), 3)
            .city(5, PlayerId(5), off(32, 6), 3)
            .city(6, PlayerId(6), off(38, 6), 3)
            .build();
        let mut players = v.players.to_vec();
        for id in 4..=6 {
            let mut player = players[3].clone();
            player.id = PlayerId(id);
            player.team = TeamId(id);
            player.cities = 1;
            players.push(player);
        }
        v.players = players.into();
        v.diplomacy.legacy = false;
        let alliance = |a, b| legatum_core::diplomacy::Relation {
            a,
            b,
            state: RelationState::Alliance,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: Some(legatum_core::diplomacy::SecurityTreaty {
                id: a.0 as u32 * 10 + b.0 as u32,
                started_turn: 1,
                protected_until: 11,
                coalition_target: None,
            }),
            war_id: None,
            declarers: Vec::new(),
        };
        v.diplomacy.relations.extend([
            alliance(v.team, TeamId(2)),
            alliance(TeamId(2), TeamId(3)),
            alliance(TeamId(4), TeamId(5)),
            alliance(TeamId(5), TeamId(6)),
        ]);
        crate::fixture::set_armies(
            &mut v,
            &[
                (ME, 20),
                (ENEMY, 100),
                (PlayerId(3), 200),
                (PlayerId(4), 40),
                (PlayerId(5), 100),
                (PlayerId(6), 200),
            ],
        );
        assert_eq!(side_strength(&v, v.team), 320);
        assert_eq!(participating_strength(&v, v.team, TeamId(4), true, 50), 70);
        assert_eq!(
            participating_strength(&v, TeamId(4), v.team, false, 50),
            140
        );
    }

    #[test]
    fn an_inherited_war_costs_its_army_at_home_and_the_campaign_it_splits() {
        let cost = |enemy_units: bool, campaign: bool| {
            let mut b = ViewBuilder::new(30, 16)
                .city(1, ME, off(4, 6), 3)
                .city(2, ENEMY, off(24, 6), 3)
                .city(3, PlayerId(3), off(4, 12), 3)
                .unit(1, ME, UnitKind::Soldier, off(4, 7));
            if enemy_units {
                b = b.unit(2, ENEMY, UnitKind::Soldier, off(6, 6)).unit(
                    3,
                    ENEMY,
                    UnitKind::Soldier,
                    off(6, 7),
                );
            }
            let mut v = b.build();
            v.diplomacy.legacy = false;
            crate::fixture::set_armies(&mut v, &[(ME, 100), (ENEMY, 80), (PlayerId(3), 50)]);
            let mut p = planner_for(&v);
            if campaign {
                p.campaigns.active = Some(crate::campaign::Campaign {
                    target: legatum_core::CityId(9),
                    start_turn: 1,
                    committed: vec![legatum_core::UnitId(1)],
                    deadline: None,
                    expected_value: 100,
                    target_was_unwalled: true,
                });
            }
            p.inherited_front_cost(TeamId(2))
        };
        let quiet = cost(false, false);
        assert_eq!(quiet, 0);
        assert!(cost(true, false) > quiet);
        assert!(cost(false, true) >= 20);
        assert!(cost(true, true) > cost(true, false));
    }

    /// Each team an Alliance adds to our side beyond a pair costs the `bloc_size_penalty` gene.
    #[test]
    fn alliance_prices_shared_vision_passage_and_free_exit() {
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(4, 6), 4)
            .city(2, ENEMY, off(10, 6), 4)
            .build();
        v.turn = 40;
        v.economy.gold = 1000;
        let rival = v.team_of(ENEMY);
        v.diplomacy.legacy = false;
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: rival,
                state: RelationState::Peace,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: None,
                declarers: Vec::new(),
            });
        let order = Order::ProposeTreaty {
            to: rival,
            kind: TreatyKind::Alliance,
        };
        let score = |price: bool| {
            let mut p = Planner::new(&v, Knobs::for_faction(Faction::India), Faction::India, 1);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            if !price {
                p.decisions.ally_vision_cost = 0;
                p.decisions.ally_passage_cost = 0;
                p.decisions.ally_exit_discount_percent = 0;
            }
            p.treaty_score(rival, TreatyKind::Alliance, &order, &TreatyBudget::of(&v))
                .expect("affordable alliance")
        };
        assert!(score(true) < score(false));
    }

    #[test]
    fn a_bigger_bloc_costs_an_alliance_score() {
        let v = ViewBuilder::new(30, 16)
            .city(1, ME, off(2, 6), 3)
            .city(2, ENEMY, off(20, 6), 3)
            .city(3, PlayerId(3), off(10, 6), 3)
            .build();
        let mut v = v;
        v.diplomacy.legacy = false;
        let p = planner_for(&v);
        assert_eq!(p.bloc_size_cost(TeamId(3)), 0);
        let mut bigger = v.clone();
        bigger
            .diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: TeamId(2),
                b: TeamId(3),
                state: RelationState::Alliance,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: None,
                declarers: Vec::new(),
            });
        let p = planner_for(&bigger);
        assert_eq!(p.bloc_size_cost(TeamId(3)), p.decisions.bloc_size_penalty);
    }

    /// A partner past 60% is still an ally against a leader ten points further ahead.
    #[test]
    fn a_runner_up_is_an_ally_against_a_leader_far_ahead() {
        let mut v = coalition_view(500, &[(0, 2), (2, 3)]);
        v.standings = [(0, 1), (2, 9), (3, 6)]
            .map(|(team, vp)| TeamStanding {
                team: TeamId(team),
                members: vec![PlayerId(team)],
                vp,
                target: 10,
                victory_hold: None,
            })
            .to_vec()
            .into();
        let order = Order::ProposeTreaty {
            to: TeamId(3),
            kind: TreatyKind::Alliance,
        };
        let score = |v: &PlayerView, gene| {
            let mut p = Planner::new(v, Knobs::for_faction(Faction::India), Faction::India, 1);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.decisions.coalition_alliance = gene;
            p.treaty_score(
                TeamId(3),
                TreatyKind::Alliance,
                &order,
                &TreatyBudget::of(v),
            )
        };
        assert!(score(&v, 1).is_some_and(|s| s >= 0));
        assert_eq!(score(&v, 0), None);
        // A partner at 70% (the leave threshold) is too close to winning itself.
        let mut close = v.clone();
        close.standings = [(0, 1), (2, 9), (3, 7)]
            .map(|(team, vp)| TeamStanding {
                team: TeamId(team),
                members: vec![PlayerId(team)],
                vp,
                target: 10,
                victory_hold: None,
            })
            .to_vec()
            .into();
        assert_eq!(score(&close, 1), None);
    }
}

#[cfg(test)]
mod peace_strong_tests {
    //! Gene `peace_strong` (decision audit round 5, 2026-10-31).
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::Faction;

    #[test]
    fn an_army_well_ahead_of_the_proposer_answers_the_danger() {
        let mut v = ViewBuilder::new(20, 16)
            .city(1, ME, off(5, 8), 3)
            .unit(2, ENEMY, UnitKind::Soldier, off(7, 8))
            .build();
        v.cities[0].hp = 80;
        let against = v.team_of(ENEMY);
        v.standings = [(v.team, ME), (against, ENEMY)]
            .map(|(team, member)| legatum_core::view::TeamStanding {
                team,
                members: vec![member],
                vp: 2,
                target: 10,
                victory_hold: None,
            })
            .to_vec()
            .into();
        let endangered = |gene, advantage| {
            let mut p = Planner::new(&v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.decisions.peace_strong = gene;
            assert!(p.endangered(), "a damaged city with an enemy beside it");
            p.peace_terms(against, 20, advantage, false)
                .iter()
                .any(|(name, _)| *name == "endangered")
        };
        assert!(endangered(0, 50), "gene off");
        assert!(!endangered(50, 50), "an army half again the proposer's");
        assert!(endangered(50, 20), "an army only a fifth ahead");
    }
}
