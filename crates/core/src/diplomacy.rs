//! Team diplomacy: Peace, Alliance or War. Orders are checked against P(t), then applied only
//! after R(t)'s scoring.
//!
//! Calls to arms: allies defend each other at once
//! and join each other's wars by choice. When a team declares War on your ally, you are at War
//! with it too, the same turn (`start_war_effects`); when your ally declares War, you get a
//! `WarCall` to join or decline, and declining keeps the Alliance and the Peace
//! (`check_call_answer`, `resolve_boundary`). Alliances cost Gold per turn
//! (`economy::treaty_upkeep`). Peace needs only its two teams and ends only their war.
//! An Alliance that would put a member at war
//! with its own ally is refused (`alliance_conflict`).

pub mod coalition;

use crate::{Event, GameState, Order, OrderError, PlayerId, TeamId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProposalId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationState {
    Peace,
    War,
    Alliance,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TreatyKind {
    #[default]
    Peace,
    Alliance,
}
impl TreatyKind {
    /// Gold each empire pays per turn. An Alliance adds one per owned city;
    /// `upkeep_for_cities` is the actual price shown in forecasts and income.
    pub fn upkeep(self) -> i32 {
        match self {
            Self::Alliance => 1,
            Self::Peace => 0,
        }
    }
    pub fn upkeep_for_cities(self, cities: usize) -> i32 {
        match self {
            Self::Alliance => self.upkeep() * cities.max(1) as i32,
            Self::Peace => 0,
        }
    }
    /// Gold the leaving empire pays to break the treaty inside its protected window (0 for
    /// Peace).
    pub fn breach_cost(self) -> i32 {
        match self {
            Self::Alliance => 120,
            Self::Peace => 0,
        }
    }
}
/// Turns of guaranteed peace after a war ends in a Peace treaty.
pub const PEACE_TRUCE_TURNS: u32 = 10;
/// Turns after an Alliance activates during which leaving it (or declaring war on the ally)
/// costs the breach Gold and counts as a betrayal.
pub const ALLIANCE_PROTECTED_TURNS: u32 = 10;
/// Turns a betrayed team refuses Alliance offers from its betrayer.
pub const BETRAYAL_TURNS: u32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agreement {
    Security,
}

/// The terms of an active Alliance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityTreaty {
    pub id: u32,
    pub started_turn: u32,
    /// First planning turn in which leaving costs nothing and is no betrayal.
    pub protected_until: u32,
    /// Public reason for this alliance, not a separate coalition or military obligation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coalition_target: Option<TeamId>,
}

/// A public record: `by` left an Alliance with `against` inside its protected window, or
/// declared war on it while allied. `against` refuses Alliance offers from `by` until `until`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Betrayal {
    pub by: TeamId,
    pub against: TeamId,
    pub turn: u32,
    pub until: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub a: TeamId,
    pub b: TeamId,
    pub state: RelationState,
    pub effective_turn: u32,
    /// First planning turn in which War may take effect; protection ends before its resolution.
    pub guarantee_until: u32,
    pub version: u32,
    #[serde(default)]
    pub security: Option<SecurityTreaty>,
    pub war_id: Option<u32>,
    /// Both teams for mutual declarations (there is then no defender).
    pub declarers: Vec<TeamId>,
}

impl Relation {
    pub fn between(&self, a: TeamId, b: TeamId) -> bool {
        (self.a, self.b) == pair(a, b)
    }
}

/// Whether `team`'s war with `enemy` is one an ally brought it into: an ally on its side is at war
/// with `enemy` in the same war, and `team` did not declare it.
pub fn joined_war(relations: &[Relation], team: TeamId, enemy: TeamId) -> bool {
    let Some(war) = relations
        .iter()
        .find(|r| r.between(team, enemy) && r.state == RelationState::War && r.war_id.is_some())
    else {
        return false;
    };
    !war.declarers.contains(&team)
        && side_of(relations, team).iter().any(|x| {
            *x != team
                && relations.iter().any(|r| {
                    r.between(*x, enemy) && r.state == RelationState::War && r.war_id == war.war_id
                })
        })
}

/// Whether `team` is the defender in the war relation `r`: its enemy declared it (and `team` did
/// not). A war `team` declared or joined by a call is not one it defends.
pub fn defender_in(r: &Relation, team: TeamId) -> bool {
    let enemy = if r.a == team { r.b } else { r.a };
    r.declarers.contains(&enemy) && !r.declarers.contains(&team)
}

/// A war that begins for `team` because its side meets `enemy`'s side (see `NewWar` users).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NewWar {
    pub team: TeamId,
    pub enemy: TeamId,
}

/// The teams joined with `team` by Alliances, directly or through other Alliances (itself
/// included): one side in every war.
pub fn side_of(relations: &[Relation], team: TeamId) -> BTreeSet<TeamId> {
    side_without(relations, team, None)
}

/// `side_of` as if the Alliance between the `cut` pair did not exist.
fn side_without(
    relations: &[Relation],
    team: TeamId,
    cut: Option<(TeamId, TeamId)>,
) -> BTreeSet<TeamId> {
    let mut side = BTreeSet::from([team]);
    let mut frontier = vec![team];
    while let Some(t) = frontier.pop() {
        for r in relations.iter().filter(|r| {
            r.state == RelationState::Alliance
                && (r.a == t || r.b == t)
                && !cut.is_some_and(|(a, b)| r.between(a, b))
        }) {
            let other = if r.a == t { r.b } else { r.a };
            if side.insert(other) {
                frontier.push(other);
            }
        }
    }
    side
}

/// Every side of two or more teams, each ascending, in ascending order of its first team.
pub fn alliance_sides(relations: &[Relation]) -> Vec<Vec<TeamId>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for r in relations
        .iter()
        .filter(|r| r.state == RelationState::Alliance)
    {
        for t in [r.a, r.b] {
            if seen.contains(&t) {
                continue;
            }
            let side = side_of(relations, t);
            seen.extend(side.iter().copied());
            out.push(side.into_iter().collect::<Vec<_>>());
        }
    }
    out.sort();
    out
}

pub fn pair(a: TeamId, b: TeamId) -> (TeamId, TeamId) {
    (a.min(b), a.max(b))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceProposal {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<DecisionExplanation>,
    pub id: ProposalId,
    #[serde(default)]
    pub kind: TreatyKind,
    pub from: TeamId,
    pub to: TeamId,
    pub proposer: PlayerId,
    pub created_turn: u32,
    pub respond_by: u32,
    pub relation_version: u32,
    /// Peace only: the teams that have agreed so far, the proposer first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consents: Vec<TeamId>,
    /// Filled in views only: the teams whose answer is still awaited (always empty in the saved
    /// state, where it is derived from the two parties and their consents).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<TeamId>,
    /// Filled in views and Peace events only: the two teams bound by this Peace.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_teams: Vec<TeamId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupersededOrder {
    pub team: TeamId,
    pub player: PlayerId,
    pub order: Order,
    pub winner: PlayerId,
    pub winner_order: Order,
    pub reported_turn: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceCooldown {
    pub from: TeamId,
    pub to: TeamId,
    pub retry_turn: u32,
    #[serde(default)]
    pub kind: TreatyKind,
}

/// Planning turns an ally's call to arms stays open: the turn it appears and the next. A call
/// nobody answers lapses: the team stays allied and at Peace with the target.
pub const CALL_WINDOW_TURNS: u32 = 2;

/// A call to arms: `ally` declared war on `target` (or was already at war with it when the
/// Alliance was signed), and `team`, its ally, may join that war or decline. Joining is an order
/// (`Order::RespondToProposal` with the call's id) that resolves at the turn boundary like a
/// declaration and puts `team` at War with `target`. Declining, or letting the window lapse,
/// keeps the Alliance and the Peace with `target`, with no betrayal and no penalty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarCall {
    /// Shares the proposal id counter, so an id names one offer or one call.
    pub id: ProposalId,
    pub team: TeamId,
    pub ally: TeamId,
    pub target: TeamId,
    pub created_turn: u32,
    pub respond_by: u32,
    /// Versions of the Alliance (`team`, `ally`) and of the war (`ally`, `target`) when the
    /// call was made: a change in either voids it.
    pub alliance_version: u32,
    pub war_version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<DiplomaticReply>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub betrayals: Vec<Betrayal>,
    #[serde(default)]
    pub next_treaty_id: u32,
    pub relations: Vec<Relation>,
    pub proposals: Vec<PeaceProposal>,
    pub next_proposal_id: u32,
    pub next_war_id: u32,
    pub superseded: Vec<SupersededOrder>,
    pub peace_cooldowns: Vec<PeaceCooldown>,
    /// Team pairs (lower id first) that have met: one side's own units or cities saw a unit,
    /// city or owned tile of the other (`visibility::contact_pairs`, recorded by
    /// `visibility::update_explored`). Contact is permanent and mutual. An Alliance needs it;
    /// Peace and War don't. `None` only in saves from before the contact rule:
    /// there every pair counts as met, so those games continue unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contacts: Option<BTreeSet<(TeamId, TeamId)>>,
    /// Open calls to arms.
    #[serde(default)]
    pub calls: Vec<WarCall>,
}

/// Why a pending offer ended: one of the two teams declared War on the other.
pub const WAR_ENDS_OFFERS: &str = "War was declared";

/// Why an Alliance can't be offered or accepted yet. Clients name the empire.
pub const NOT_MET: &str = "You haven't met this empire yet: offers need contact first (see one of their units, cities or borders)";

impl DiplomacyState {
    pub fn new(teams: impl Iterator<Item = TeamId>) -> Self {
        let teams: Vec<_> = teams.collect::<BTreeSet<_>>().into_iter().collect();
        let mut relations = Vec::new();
        for (i, &a) in teams.iter().enumerate() {
            for &b in &teams[i + 1..] {
                relations.push(Relation {
                    a,
                    b,
                    state: RelationState::Peace,
                    effective_turn: 1,
                    guarantee_until: 0,
                    version: 0,
                    security: None,
                    war_id: None,
                    declarers: Vec::new(),
                });
            }
        }
        Self {
            replies: Vec::new(),
            betrayals: Vec::new(),
            next_treaty_id: 0,
            relations,
            proposals: Vec::new(),
            next_proposal_id: 0,
            next_war_id: 0,
            superseded: Vec::new(),
            peace_cooldowns: Vec::new(),
            contacts: Some(BTreeSet::new()),
            calls: Vec::new(),
        }
    }

    /// Record that two teams have met (normally done by `visibility::update_explored`).
    pub fn meet(&mut self, a: TeamId, b: TeamId) {
        if let Some(contacts) = self.contacts.as_mut().filter(|_| a != b) {
            contacts.insert(pair(a, b));
        }
    }

    /// Every pair of these teams has met: for fixtures and scenarios that start in contact.
    pub fn meet_all(&mut self, teams: &[TeamId]) {
        for (i, a) in teams.iter().enumerate() {
            for b in &teams[i + 1..] {
                self.meet(*a, *b);
            }
        }
    }

    /// Whether two teams have met. In saves from before the contact rule every pair has.
    pub fn met(&self, a: TeamId, b: TeamId) -> bool {
        a == b
            || self
                .contacts
                .as_ref()
                .is_none_or(|c| c.contains(&pair(a, b)))
    }

    pub fn relation(&self, a: TeamId, b: TeamId) -> Option<&Relation> {
        self.relations.iter().find(|r| r.between(a, b))
    }

    /// The side `team` fights on: itself and everyone it is allied with.
    pub fn side(&self, team: TeamId) -> BTreeSet<TeamId> {
        side_of(&self.relations, team)
    }

    /// Whether the two teams are at War.
    pub fn at_war(&self, a: TeamId, b: TeamId) -> bool {
        self.relation(a, b)
            .is_some_and(|r| r.state == RelationState::War)
    }

    /// Both teams must agree to either kind of proposal.
    pub fn required_teams(&self, p: &PeaceProposal) -> BTreeSet<TeamId> {
        self.peace_parties(p.from, p.to)
    }

    /// A Peace binds exactly the two teams named in its proposal.
    pub fn peace_parties(&self, a: TeamId, b: TeamId) -> BTreeSet<TeamId> {
        BTreeSet::from([a, b])
    }

    /// The teams `team` is directly allied with.
    pub fn partners(&self, team: TeamId) -> Vec<TeamId> {
        self.relations
            .iter()
            .filter(|r| r.state == RelationState::Alliance && (r.a == team || r.b == team))
            .map(|r| if r.a == team { r.b } else { r.a })
            .collect()
    }

    /// The open call to arms with this id, if any.
    pub fn call(&self, id: ProposalId) -> Option<&WarCall> {
        self.calls.iter().find(|c| c.id == id)
    }

    /// Calls-to-arms rules: the wars an Alliance between `a` and `b` carries each into at once
    /// (those where its partner is the defender: the enemy declared them) and the calls it
    /// raises (wars its partner declared or joined). Both lists are sorted.
    pub fn alliance_wars(
        &self,
        alive: impl Fn(TeamId) -> bool,
        a: TeamId,
        b: TeamId,
    ) -> (Vec<NewWar>, Vec<NewWar>) {
        let (mut joins, mut calls) = (BTreeSet::new(), BTreeSet::new());
        for (partner, new) in [(a, b), (b, a)] {
            for r in self
                .relations
                .iter()
                .filter(|r| r.state == RelationState::War && (r.a == partner || r.b == partner))
            {
                let enemy = if r.a == partner { r.b } else { r.a };
                if !alive(enemy)
                    || enemy == new
                    || self.at_war(new, enemy)
                    || self.side(new).contains(&enemy)
                {
                    continue;
                }
                let war = NewWar { team: new, enemy };
                if defender_in(r, partner) {
                    joins.insert(war);
                } else {
                    calls.insert(war);
                }
            }
        }
        (joins.into_iter().collect(), calls.into_iter().collect())
    }

    /// Calls-to-arms rules: what a declaration by `a` against `t` brings besides the pair
    /// itself: the allies of `t` who join at once (against `a`), and the calls `a`'s allies
    /// get (against `t`). `breach` is a war on a direct ally, whose Alliance ends first.
    pub fn declare_wars(
        &self,
        alive: impl Fn(TeamId) -> bool,
        a: TeamId,
        t: TeamId,
        breach: bool,
    ) -> (Vec<NewWar>, Vec<NewWar>) {
        let cut = breach.then_some((a, t));
        let direct = |team: TeamId| -> Vec<TeamId> {
            self.relations
                .iter()
                .filter(|r| {
                    r.state == RelationState::Alliance
                        && (r.a == team || r.b == team)
                        && !cut.is_some_and(|(x, y)| r.between(x, y))
                })
                .map(|r| if r.a == team { r.b } else { r.a })
                .collect()
        };
        let side_a = side_without(&self.relations, a, cut);
        let mut joins: Vec<NewWar> = direct(t)
            .into_iter()
            .filter(|x| alive(*x) && *x != a && !side_a.contains(x) && !self.at_war(*x, a))
            .map(|x| NewWar { team: x, enemy: a })
            .collect();
        let mut calls: Vec<NewWar> = direct(a)
            .into_iter()
            .filter(|y| alive(*y) && *y != t && !self.at_war(*y, t))
            .map(|y| NewWar { team: y, enemy: t })
            .collect();
        joins.sort();
        calls.sort();
        (joins, calls)
    }

    /// The teams whose answer a proposal still waits for (never the proposer).
    pub fn awaiting(&self, p: &PeaceProposal) -> Vec<TeamId> {
        self.required_teams(p)
            .into_iter()
            .filter(|t| *t != p.from && !p.consents.contains(t))
            .collect()
    }

    /// Whom `team` is dealing with in this proposal: the other named team.
    pub fn counterparty(&self, p: &PeaceProposal, team: TeamId) -> TeamId {
        if team == p.from { p.to } else { p.from }
    }

    /// Why a war between `a` and `t` can't begin: an Alliance chain still binds them (declaring
    /// war on a direct ally ends that one Alliance first).
    pub fn declare_conflict(
        &self,
        state: &GameState,
        a: TeamId,
        t: TeamId,
        breach: bool,
    ) -> Option<String> {
        let cut = breach.then_some((a, t));
        side_without(&self.relations, a, cut).contains(&t).then(|| {
            format!(
                "{} is also allied to you through another Alliance: leave that Alliance first",
                crate::scoring::team_label(state, t)
            )
        })
    }

    /// Why an Alliance between `a` and `b` is refused: it would make some empire fight beside a
    /// team it is at war with.
    pub fn alliance_conflict(&self, state: &GameState, a: TeamId, b: TeamId) -> Option<String> {
        let (sa, sb) = (self.side(a), self.side(b));
        sa.iter().find_map(|&x| {
            sb.iter().find(|&&y| self.at_war(x, y)).map(|&y| {
                format!(
                    "{} is at War with {}, and this Alliance would make them allies: end that War first",
                    crate::scoring::team_label(state, x),
                    crate::scoring::team_label(state, y)
                )
            })
        })
    }

    /// Why a set of new wars can't begin in planning turn `effective`: a team it would drag in
    /// holds a guaranteed peace with its new enemy.
    pub fn join_blocked(
        &self,
        state: &GameState,
        effective: u32,
        wars: &[NewWar],
    ) -> Option<String> {
        wars.iter().find_map(|w| {
            let r = self.relation(w.team, w.enemy)?;
            (effective < r.guarantee_until).then(|| {
                format!(
                    "{} would have to join the war on {}, but their peace is guaranteed through turn {}",
                    empire_label(state, w.team),
                    empire_label(state, w.enemy),
                    r.guarantee_until - 1
                )
            })
        })
    }

    /// The betrayal that makes `against` refuse Alliance offers from `by` in planning turn
    /// `turn`, if any.
    pub fn refusal(&self, by: TeamId, against: TeamId, turn: u32) -> Option<&Betrayal> {
        self.betrayals
            .iter()
            .rev()
            .find(|b| b.by == by && b.against == against && turn < b.until)
    }
}

/// The team that waives the Alliance between `a` and `b` its upkeep this turn, if any: a team
/// either ally is at War with that is close to winning (`scoring::near_victory`). The lowest such
/// team id is named. Sam, 2026-11-22 (via the Fable review).
pub fn upkeep_waived_by(state: &GameState, a: TeamId, b: TeamId) -> Option<TeamId> {
    state.teams().into_iter().find(|&t| {
        t != a
            && t != b
            && (state.teams_at_war(a, t) || state.teams_at_war(b, t))
            && crate::scoring::near_victory(state, t)
    })
}

impl GameState {
    pub fn at_war(&self, a: PlayerId, b: PlayerId) -> bool {
        self.teams_at_war(self.team_of(a), self.team_of(b))
    }

    pub fn teams_at_war(&self, a: TeamId, b: TeamId) -> bool {
        if a == TeamId::BARBARIAN || b == TeamId::BARBARIAN {
            return a != b;
        }
        a != b
            && self.diplomacy.as_ref().is_none_or(|d| {
                d.relation(a, b)
                    .is_some_and(|r| r.state == RelationState::War)
            })
    }

    pub fn allied(&self, a: PlayerId, b: PlayerId) -> bool {
        self.diplomacy.as_ref().is_some_and(|d| {
            d.relation(self.team_of(a), self.team_of(b))
                .is_some_and(|r| r.state == RelationState::Alliance)
        })
    }

    /// Whether `mover`'s unit of `kind` may stand in land owned by `land_owner`: unowned land,
    /// its own team's, an enemy's (War), an ally's, or land its faction crosses at Peace
    /// or the
    /// land of a Yam Relay Station's holder, which is open to every empire's units, military
    /// included (`goals::holds_open_road`). This is the single passage rule.
    pub fn passage_allowed(
        &self,
        mover: PlayerId,
        _kind: crate::UnitKind,
        land_owner: Option<PlayerId>,
    ) -> bool {
        land_owner.is_none_or(|o| {
            self.same_team(mover, o)
                || self.at_war(mover, o)
                // A closed country (Sakoku) lets nobody in at Peace: no Alliance passage, no
                // Mongol couriers, no open road. Its own team still crosses (above).
                || (!mover.is_barbarian()
                    && !crate::goals::is_closed(self, o)
                    && (self.allied(mover, o)
                        || crate::goals::holds_open_road(self, o)))
        })
    }

    pub fn shares_vision(&self, a: PlayerId, b: PlayerId) -> bool {
        self.same_team(a, b) || self.allied(a, b)
    }

    /// `passage_allowed` for an existing unit, wherever it stands.
    pub fn unit_in_open_land(&self, u: &crate::state::Unit) -> bool {
        self.passage_allowed(u.owner, u.kind, self.map.get(u.pos).and_then(|t| t.owner))
    }

    pub fn team_alive(&self, team: TeamId) -> bool {
        self.players.iter().any(|p| p.team == team && p.alive)
    }
}

pub fn is_diplomatic(order: &Order) -> bool {
    matches!(
        order,
        Order::DeclareWar { .. }
            | Order::ProposeTreaty { .. }
            | Order::RespondToProposal { .. }
            | Order::WithdrawProposal { .. }
            | Order::BreakTreaty { .. }
    )
}

/// Never reveal whether a guessed private proposal ID exists for another pair.
fn other_team(state: &GameState, team: TeamId, order: &Order) -> Result<TeamId, String> {
    match order {
        Order::DeclareWar { against, .. } => Ok(*against),
        Order::BreakTreaty { with, .. } => Ok(*with),
        Order::ProposeTreaty { to, .. } => Ok(*to),
        Order::RespondToProposal { proposal, .. } | Order::WithdrawProposal { proposal } => {
            let d = state.diplomacy.as_ref();
            if let Some(call) = d
                .and_then(|d| d.call(*proposal))
                .filter(|c| c.team == team && matches!(order, Order::RespondToProposal { .. }))
            {
                return Ok(call.ally);
            }
            let p = d
                .and_then(|d| {
                    d.proposals
                        .iter()
                        .find(|p| p.id == *proposal && d.required_teams(p).contains(&team))
                })
                .ok_or("No such proposal for your team")?;
            Ok(d.expect("found above").counterparty(p, team))
        }
        _ => Err("Not a diplomatic order".into()),
    }
}

/// The teams `team` is allied with.
pub fn allied_teams(state: &GameState, team: TeamId) -> Vec<TeamId> {
    state.diplomacy.as_ref().map_or_else(Vec::new, |d| {
        d.relations
            .iter()
            .filter(|r| r.state == RelationState::Alliance && (r.a == team || r.b == team))
            .map(|r| if r.a == team { r.b } else { r.a })
            .collect()
    })
}

/// Why a Sakoku vow can't ride with these orders: it resolves after the turn's declarations and
/// Alliance signatures were checked against an open country, so it may not be ordered with one
/// (accepting Peace, or any other treaty, is fine).
pub fn vow_conflict(state: &GameState, orders: &[Order]) -> Option<String> {
    let alliance = |proposal: &crate::diplomacy::ProposalId| {
        state.diplomacy.as_ref().is_some_and(|d| {
            d.proposals
                .iter()
                .any(|p| p.id == *proposal && p.kind == TreatyKind::Alliance)
        })
    };
    orders.iter().find_map(|o| match o {
        Order::DeclareWar { .. } => Some(
            "A closed country can't declare war: close it on a turn without a declaration"
                .to_string(),
        ),
        Order::ProposeTreaty {
            kind: TreatyKind::Alliance,
            ..
        } => Some(
            "A closed country can't join an Alliance: close it on a turn without signing one"
                .to_string(),
        ),
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } if alliance(proposal) => Some(
            "A closed country can't join an Alliance: close it on a turn without signing one"
                .to_string(),
        ),
        _ => None,
    })
}

/// Japan's Sakoku vow (`goals::GoalRule::Sakoku`) on a diplomatic order: a closed country can't
/// declare war, and no Alliance can be proposed or signed by a team with a closed member, or
/// with a team that has one. Peace, leaving an Alliance and the teammate's own war are not
/// touched: the vow binds the closed empire, and teams are not Alliances.
fn closed_refusal(
    state: &GameState,
    proposals: &[PeaceProposal],
    player: PlayerId,
    team: TeamId,
    other: TeamId,
    order: &Order,
) -> Option<String> {
    let alliance = match order {
        Order::DeclareWar { .. } => {
            return crate::goals::is_closed(state, player).then(|| {
                "You have closed the country (Sakoku): it can't declare war. Reopen it first"
                    .to_string()
            });
        }
        // Joining an ally's war is a declaration of one's own.
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } if state
            .diplomacy
            .as_ref()
            .is_some_and(|d| d.call(*proposal).is_some()) =>
        {
            return crate::goals::is_closed(state, player).then(|| {
                "You have closed the country (Sakoku): it can't join a war. Reopen it first"
                    .to_string()
            });
        }
        Order::ProposeTreaty { kind, .. } => *kind == TreatyKind::Alliance,
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } => proposals
            .iter()
            .find(|p| p.id == *proposal)
            .is_some_and(|p| p.kind == TreatyKind::Alliance),
        _ => false,
    };
    if !alliance {
        return None;
    }
    [team, other]
        .into_iter()
        .find(|t| crate::goals::team_closed(state, *t))
        .map(|t| {
            format!(
                "{} has closed the country (Sakoku): a closed country can't join an Alliance",
                crate::scoring::team_label(state, t)
            )
        })
}

pub fn check_order(state: &GameState, player: PlayerId, order: &Order) -> Result<(), String> {
    let d = state
        .diplomacy
        .as_ref()
        .ok_or("Legacy game: permanent war")?;
    let team = state.team_of(player);
    let other = other_team(state, team, order)?;
    if other == team {
        return Err("Permanent teammates cannot negotiate or declare war".into());
    }
    if let Some(reason) = closed_refusal(state, &d.proposals, player, team, other, order) {
        return Err(reason);
    }
    if !state.team_alive(team) || !state.team_alive(other) {
        return Err("Both teams must survive to complete diplomacy".into());
    }
    let r = d.relation(team, other).ok_or("No such team")?;
    let effective = state.turn + 1;
    let alive = |t| state.team_alive(t);
    match order {
        Order::DeclareWar { breach, .. } => {
            if r.state == RelationState::War {
                return Err("Already at War".into());
            }
            check_guarantee(state.turn, r, state, other)?;
            if r.state == RelationState::Alliance {
                if !breach {
                    return Err(format!(
                        "You are allied with {}: leave the Alliance first, or declare war on your ally as a betrayal",
                        crate::scoring::team_label(state, other)
                    ));
                }
                afford(
                    state,
                    player,
                    leave_cost(r, effective),
                    "Breaking the Alliance",
                )?;
            }
            if let Some(reason) = d.declare_conflict(state, team, other, *breach) {
                return Err(reason);
            }
            let (wars, _) = d.declare_wars(alive, team, other, *breach);
            // Plain-Peace declarations are surprise wars: defensive allies join now, not
            // at the next boundary. The last protected resolution still belongs to Peace.
            let join_turn = if surprise_war(r, state.turn) {
                state.turn
            } else {
                effective
            };
            if let Some(reason) = d.join_blocked(state, join_turn, &wars) {
                return Err(reason);
            }
        }
        Order::BreakTreaty { .. } => {
            if r.state != RelationState::Alliance {
                return Err("No Alliance to leave".into());
            }
            afford(
                state,
                player,
                leave_cost(r, effective),
                "Breaking the Alliance",
            )?;
        }
        Order::ProposeTreaty { kind, .. } => {
            check_kind(r, *kind)?;
            if *kind == TreatyKind::Alliance {
                check_alliance(d, state, team, other, state.turn)?;
                if let Some(b) = d.refusal(team, other, effective) {
                    return Err(refusal_reason(state, b));
                }
            }
            if d.proposals
                .iter()
                .any(|p| pair(p.from, p.to) == pair(team, other))
            {
                return Err("This pair already has an outstanding proposal".into());
            }
        }
        Order::RespondToProposal {
            proposal, accept, ..
        } if d.call(*proposal).is_some() => {
            let call = d.call(*proposal).expect("matched above");
            check_call_answer(state, d, team, call, *accept)?;
        }
        Order::RespondToProposal { proposal, .. } | Order::WithdrawProposal { proposal } => {
            let p = d
                .proposals
                .iter()
                .find(|p| p.id == *proposal)
                .expect("checked above");
            if let Order::RespondToProposal {
                explanation: Some(e),
                ..
            } = order
                && (e.reasons.len() > 12 || e.reasons.iter().any(|r| r.len() > 160))
            {
                return Err("Diplomatic explanation is too long".into());
            }
            let withdraw = matches!(order, Order::WithdrawProposal { .. });
            if (withdraw && p.from != team) || (!withdraw && !d.awaiting(p).contains(&team)) {
                return Err("Your team cannot take that action on this proposal".into());
            }
            if state.turn < p.created_turn || state.turn > p.respond_by {
                return Err("Proposal is outside its response window".into());
            }
            let rp = d.relation(p.from, p.to).ok_or("No such team")?;
            if p.relation_version != rp.version {
                return Err("Proposal no longer matches the relationship".into());
            }
            check_kind(rp, p.kind)?;
            if matches!(order, Order::RespondToProposal { accept: true, .. })
                && p.kind == TreatyKind::Alliance
            {
                check_alliance(d, state, p.from, p.to, state.turn)?;
            }
        }
        _ => return Err("Not a diplomatic order".into()),
    }
    Ok(())
}

/// Why the call can no longer be answered, as of planning turn `turn` (a call is void when a
/// team died, the Alliance or the war changed, or the team is already at war with the target).
fn call_void_reason(state: &GameState, d: &DiplomacyState, call: &WarCall) -> Option<String> {
    let label = |t| crate::scoring::team_label(state, t);
    if ![call.team, call.ally, call.target]
        .into_iter()
        .all(|t| state.team_alive(t))
    {
        return Some("A team was eliminated".into());
    }
    if !d
        .relation(call.team, call.ally)
        .is_some_and(|r| r.state == RelationState::Alliance && r.version == call.alliance_version)
    {
        return Some(format!("The Alliance with {} ended", label(call.ally)));
    }
    if !d
        .relation(call.ally, call.target)
        .is_some_and(|r| r.state == RelationState::War && r.version == call.war_version)
    {
        return Some(format!(
            "{}'s war on {} ended",
            label(call.ally),
            label(call.target)
        ));
    }
    if d.at_war(call.team, call.target) {
        return Some(format!("Already at war with {}", label(call.target)));
    }
    None
}

/// Why `team` cannot join `call` in the checked turn: the war on its ally's enemy
/// would break any guaranteed Peace, or the target is also its ally (leave that Alliance first).
fn call_join_blocked(
    state: &GameState,
    d: &DiplomacyState,
    call: &WarCall,
    effective: u32,
) -> Option<String> {
    let label = |t| empire_label(state, t);
    let r = d.relation(call.team, call.target)?;
    if d.side(call.team).contains(&call.target) {
        return Some(format!(
            "{} is also your ally: leave that Alliance first, or decline",
            label(call.target)
        ));
    }
    if effective < r.guarantee_until {
        return Some(format!(
            "Your peace with {} is guaranteed through turn {}",
            label(call.target),
            r.guarantee_until - 1
        ));
    }
    let (wars, _) = d.declare_wars(|team| state.team_alive(team), call.team, call.target, false);
    d.join_blocked(state, effective, &wars)
}

/// A team's answer to a call to arms is valid: it is the team called, the window is open, the call
/// still stands, and a Join breaks no guaranteed Peace or Alliance.
fn check_call_answer(
    state: &GameState,
    d: &DiplomacyState,
    team: TeamId,
    call: &WarCall,
    accept: bool,
) -> Result<(), String> {
    if call.team != team {
        return Err("Your team cannot take that action on this call".into());
    }
    if state.turn < call.created_turn || state.turn > call.respond_by {
        return Err("The call to arms is outside its response window".into());
    }
    if let Some(reason) = call_void_reason(state, d, call) {
        return Err(reason);
    }
    if accept && let Some(reason) = call_join_blocked(state, d, call, state.turn) {
        return Err(reason);
    }
    Ok(())
}

/// The rules an Alliance between `a` and `b` must meet: they have met, nobody on one side is at
/// war with anybody on the other, and no team it would carry into a war holds a guaranteed
/// peace with the new enemy.
fn check_alliance(
    d: &DiplomacyState,
    state: &GameState,
    a: TeamId,
    b: TeamId,
    effective: u32,
) -> Result<(), String> {
    if !d.met(a, b) {
        return Err(NOT_MET.into());
    }
    if d.side(a).contains(&b) {
        return Err("You are already on the same side through an ally".into());
    }
    if let Some(reason) = d.alliance_conflict(state, a, b) {
        return Err(reason);
    }
    let (wars, _) = d.alliance_wars(|t| state.team_alive(t), a, b);
    match d.join_blocked(state, effective, &wars) {
        Some(reason) => Err(reason),
        None => Ok(()),
    }
}

pub fn validate_orders(state: &GameState, player: PlayerId, orders: &[Order]) -> Vec<OrderError> {
    let mut seen = BTreeSet::new();
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| is_diplomatic(o))
        .filter_map(|(index, o)| {
            let result = check_order(state, player, o).and_then(|()| {
                let other = other_team(state, state.team_of(player), o)?;
                if seen.insert(other) {
                    Ok(())
                } else {
                    Err("One diplomatic order per other team per turn".into())
                }
            });
            result.err().map(|message| OrderError { index, message })
        })
        .collect()
}

/// Exact Gold costs and earliest boundary. `cost` is the one-off Gold payment from the ordering
/// empire (the cost of breaking an Alliance inside its protected window); `upkeep` is the Gold per turn each empire on both
/// teams pays while a signed Alliance lasts. `betrayal` marks orders that leave an Alliance
/// inside its protected window or declare war on an ally.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomaticAction {
    pub order: Order,
    pub effective_turn: u32,
    pub cost: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub upkeep: i32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub betrayal: bool,
    /// The wars this action starts besides the one it names: for an Alliance, every war either
    /// side brings the other into; for a declaration, the allies on both sides who join.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub new_wars: Vec<NewWar>,
    /// The calls to arms this action raises (calls-to-arms rules): for a declaration, the
    /// allies asked to join; for an Alliance, the wars either partner already fights as the
    /// aggressor, each asked of the other.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<NewWar>,
    /// Peace: the other teams whose agreement the whole Peace still needs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_agree: Vec<TeamId>,
    pub blocked_reason: Option<String>,
}
fn is_zero(n: &i32) -> bool {
    *n == 0
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedDiplomaticOrder {
    pub player: PlayerId,
    pub order: Order,
}

/// Planning orders are supplied by the host, not saved in canonical state. Share only this
/// team's diplomacy, never military orders or another team's unannounced declaration.
pub fn with_pending<'a>(
    view: &mut crate::PlayerView,
    orders: impl Iterator<Item = (PlayerId, &'a [Order])>,
) {
    view.diplomacy.queued = orders
        .filter(|(p, _)| view.team_of(*p) == view.team)
        .flat_map(|(player, os)| {
            os.iter()
                .filter(|o| is_diplomatic(o))
                .map(move |order| QueuedDiplomaticOrder {
                    player,
                    order: order.clone(),
                })
        })
        .collect();
    view.diplomacy.queued.sort_by_key(|q| q.player);
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyView {
    #[serde(default)]
    pub replies: Vec<DiplomaticReply>,
    #[serde(default)]
    pub coalition: Option<coalition::CoalitionView>,
    #[serde(default)]
    pub exploration: coalition::ExplorationView,
    /// Public: every betrayal still refusing Alliances, for every team's row.
    #[serde(default)]
    pub betrayals: Vec<Betrayal>,
    pub queued: Vec<QueuedDiplomaticOrder>,
    pub legacy: bool,
    pub relations: Vec<Relation>,
    /// Every Alliance side of two or more teams (public, derived from `relations`).
    #[serde(default)]
    pub alliances: Vec<Vec<TeamId>>,
    pub proposals: Vec<PeaceProposal>,
    /// Calls to arms waiting for this team's answer.
    #[serde(default)]
    pub calls: Vec<WarCall>,
    /// The calls-to-arms rules apply (Alliances cost Gold; allies defend at once; an ally's own
    /// war is a call). False in older saves.
    pub superseded: Vec<SupersededOrder>,
    pub peace_cooldowns: Vec<PeaceCooldown>,
    /// Other teams this team has met (see `DiplomacyState::contacts`), ascending.
    #[serde(default)]
    pub met: Vec<TeamId>,
    pub actions: Vec<DiplomaticAction>,
}

impl DiplomacyView {
    /// The side `team` fights on: itself and everyone it is allied with.
    pub fn side(&self, team: TeamId) -> BTreeSet<TeamId> {
        side_of(&self.relations, team)
    }

    /// Whom `team` is dealing with in proposal `p`.
    pub fn counterparty(&self, p: &PeaceProposal, team: TeamId) -> TeamId {
        if team == p.from { p.to } else { p.from }
    }
}

pub fn view(state: &GameState, me: PlayerId) -> DiplomacyView {
    let team = state.team_of(me);
    let Some(d) = &state.diplomacy else {
        return DiplomacyView {
            replies: Vec::new(),
            coalition: None,
            exploration: coalition::exploration(state, me),
            betrayals: Vec::new(),
            queued: Vec::new(),
            legacy: true,
            relations: Vec::new(),
            alliances: Vec::new(),
            proposals: Vec::new(),
            calls: Vec::new(),
            superseded: Vec::new(),
            peace_cooldowns: Vec::new(),
            met: Vec::new(),
            actions: Vec::new(),
        };
    };
    let proposals: Vec<_> = d
        .proposals
        .iter()
        .filter(|p| d.required_teams(p).contains(&team))
        .map(|p| PeaceProposal {
            needs: d.awaiting(p),
            required_teams: if p.kind == TreatyKind::Peace {
                d.required_teams(p).into_iter().collect()
            } else {
                Vec::new()
            },
            ..p.clone()
        })
        .collect();
    let mut orders = Vec::new();
    for other in state
        .teams()
        .into_iter()
        .filter(|t| *t != team && state.team_alive(*t))
    {
        orders.push(Order::DeclareWar {
            against: other,
            breach: false,
        });
        orders.push(Order::DeclareWar {
            against: other,
            breach: true,
        });
        orders.push(Order::BreakTreaty {
            with: other,
            agreement: Agreement::Security,
        });
        for kind in [TreatyKind::Peace, TreatyKind::Alliance] {
            orders.push(Order::ProposeTreaty { to: other, kind });
        }
    }
    for p in &proposals {
        if p.needs.contains(&team) {
            for accept in [true, false] {
                orders.push(Order::RespondToProposal {
                    proposal: p.id,
                    accept,
                    explanation: None,
                });
            }
        } else if p.from == team {
            orders.push(Order::WithdrawProposal { proposal: p.id });
        }
    }
    let calls: Vec<WarCall> = d.calls.iter().filter(|c| c.team == team).cloned().collect();
    for c in &calls {
        for accept in [true, false] {
            orders.push(Order::RespondToProposal {
                proposal: c.id,
                accept,
                explanation: None,
            });
        }
    }
    let effective = state.turn + 1;
    let actions = orders
        .into_iter()
        .map(|order| {
            let effective_turn = match &order {
                Order::DeclareWar { against, .. } => {
                    d.relation(team, *against).map_or(effective, |r| {
                        if surprise_war(r, state.turn) {
                            state.turn
                        } else {
                            effective.max(r.guarantee_until)
                        }
                    })
                }
                Order::ProposeTreaty { .. } => state.turn + 2,
                _ => effective,
            };
            let blocked_reason = if state.finished {
                Some("The game has ended".into())
            } else if !state.player(me).alive {
                Some("Your empire has been eliminated".into())
            } else {
                check_order(state, me, &order).err()
            };
            let cost = order_cost(state, d, team, &order, effective);
            let (new_wars, calls, also_agree) = action_effects(state, d, team, &order);
            let upkeep = signed_kind(d, &order).upkeep_for_cities(state.cities_of(me).count());
            let betrayal = betrays(d, team, &order, effective);
            DiplomaticAction {
                order,
                effective_turn,
                cost,
                upkeep,
                betrayal,
                new_wars,
                calls,
                also_agree,
                blocked_reason,
            }
        })
        .collect();
    DiplomacyView {
        replies: d
            .replies
            .iter()
            .filter(|r| r.proposal.from == team || r.proposal.to == team)
            .cloned()
            .collect(),
        coalition: coalition::from_state(state),
        exploration: coalition::exploration(state, me),
        betrayals: d
            .betrayals
            .iter()
            .filter(|b| effective < b.until)
            .cloned()
            .collect(),
        queued: Vec::new(),
        legacy: false,
        relations: d.relations.clone(),
        alliances: alliance_sides(&d.relations),
        proposals,
        calls,
        superseded: d
            .superseded
            .iter()
            .filter(|s| s.team == team)
            .cloned()
            .collect(),
        peace_cooldowns: d
            .peace_cooldowns
            .iter()
            .filter(|c| c.from == team || c.to == team)
            .cloned()
            .collect(),
        met: state
            .teams()
            .into_iter()
            .filter(|t| *t != team && d.met(team, *t))
            .collect(),
        actions,
    }
}

/// What an action brings besides its own effect: the wars it starts (`DiplomaticAction::new_wars`),
/// the calls to arms it raises (`DiplomaticAction::calls`) and, for a Peace, the other teams that
/// still have to agree.
fn action_effects(
    state: &GameState,
    d: &DiplomacyState,
    team: TeamId,
    order: &Order,
) -> (Vec<NewWar>, Vec<NewWar>, Vec<TeamId>) {
    let alive = |t| state.team_alive(t);
    match order {
        Order::DeclareWar { against, breach } => {
            let (joins, calls) = d.declare_wars(alive, team, *against, *breach);
            (joins, calls, Vec::new())
        }
        Order::ProposeTreaty {
            to,
            kind: TreatyKind::Alliance,
        } => {
            let (joins, calls) = d.alliance_wars(alive, team, *to);
            (joins, calls, Vec::new())
        }
        Order::ProposeTreaty {
            to,
            kind: TreatyKind::Peace,
        } => (
            Vec::new(),
            Vec::new(),
            d.peace_parties(team, *to)
                .into_iter()
                .filter(|t| *t != team && *t != *to && alive(*t))
                .collect(),
        ),
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } => {
            if let Some(call) = d.call(*proposal) {
                let (joins, _) = d.declare_wars(alive, team, call.target, false);
                return (joins, Vec::new(), Vec::new());
            }
            d.proposals.iter().find(|p| p.id == *proposal).map_or(
                (Vec::new(), Vec::new(), Vec::new()),
                |p| match p.kind {
                    TreatyKind::Alliance => {
                        let (joins, calls) = d.alliance_wars(alive, p.from, p.to);
                        (joins, calls, Vec::new())
                    }
                    TreatyKind::Peace => (
                        Vec::new(),
                        Vec::new(),
                        d.awaiting(p)
                            .into_iter()
                            .filter(|t| *t != team && alive(*t))
                            .collect(),
                    ),
                },
            )
        }
        _ => (Vec::new(), Vec::new(), Vec::new()),
    }
}

/// Surprise war (G20, Civ 6): a declaration against a team at plain Peace (past any guaranteed
/// Peace turns) takes effect in the turn it is ordered. War on an ally takes effect next turn.
pub fn surprise_war(r: &Relation, turn: u32) -> bool {
    r.state == RelationState::Peace && turn >= r.guarantee_until
}

/// The winning diplomatic order of each (team, other team) pair, as `resolve_boundary` picks
/// it: lower PlayerId first, then submission order.
fn chosen_orders(
    state: &GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
) -> BTreeMap<(TeamId, TeamId), (PlayerId, Order)> {
    let mut chosen = BTreeMap::new();
    for (&player, os) in orders {
        let team = state.team_of(player);
        for order in os.iter().filter(|o| is_diplomatic(o)) {
            if let Ok(other) = other_team(state, team, order) {
                chosen
                    .entry((team, other))
                    .or_insert_with(|| (player, order.clone()));
            }
        }
    }
    chosen
}

/// Start of resolution, before ranged fire and movement: every valid surprise declaration
/// (`surprise_war`) begins War now, so attacks in the same order lists resolve this turn.
/// Returns the pairs now at War; the boundary skips their declarations.
pub fn declare_surprise_wars(
    state: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    events: &mut Vec<Event>,
) -> BTreeSet<(TeamId, TeamId)> {
    let mut declarations: BTreeMap<(TeamId, TeamId), Vec<TeamId>> = BTreeMap::new();
    let Some(d) = state.diplomacy.as_ref() else {
        return BTreeSet::new();
    };
    if state.finished {
        return BTreeSet::new();
    }
    for ((team, other), (player, order)) in chosen_orders(state, orders) {
        if matches!(order, Order::DeclareWar { .. })
            && check_order(state, player, &order).is_ok()
            && d.relation(team, other)
                .is_some_and(|r| surprise_war(r, state.turn))
        {
            declarations
                .entry(pair(team, other))
                .or_default()
                .push(team);
        }
    }
    let turn = state.turn;
    let mut d = state.diplomacy.take().expect("checked above");
    for (&(a, b), declarers) in &mut declarations {
        declarers.sort();
        declarers.dedup();
        d.next_war_id += 1;
        let r = d
            .relations
            .iter_mut()
            .find(|r| r.between(a, b))
            .expect("validated pair");
        r.state = RelationState::War;
        r.effective_turn = turn;
        r.guarantee_until = 0;
        r.version += 1;
        r.war_id = Some(d.next_war_id);
        r.declarers = declarers.clone();
        events.push(Event::WarDeclared {
            relation: r.clone(),
        });
    }
    for (&(a, b), declarers) in &declarations {
        for &declarer in declarers {
            let victim = if declarer == a { b } else { a };
            start_war_effects(
                &mut d,
                state,
                turn,
                turn + 1,
                declarer,
                victim,
                true,
                events,
            );
        }
    }
    end_warring_alliances(&mut d, state, turn, events);
    state.diplomacy = Some(d);
    declarations.into_keys().collect()
}

/// Put `x` and `y` at War in war `war_id`, begun by `declarers`.
fn set_war(
    d: &mut DiplomacyState,
    x: TeamId,
    y: TeamId,
    turn: u32,
    war_id: Option<u32>,
    declarers: Vec<TeamId>,
) -> Option<Relation> {
    let r = d.relations.iter_mut().find(|r| r.between(x, y))?;
    r.security = None;
    r.state = RelationState::War;
    r.effective_turn = turn;
    r.guarantee_until = 0;
    r.version += 1;
    r.war_id = war_id;
    r.declarers = declarers;
    Some(r.clone())
}

/// Calls-to-arms rules: a war has begun between `aggressor` and `victim` in turn `turn`. The
/// victim's allies are at War with the aggressor at once, in the same war. With `with_calls`, the
/// aggressor's allies get a call to arms against the victim (`created_turn` is the planning turn
/// they first see it in); a team that joins by answering a call raises none of its own.
#[allow(clippy::too_many_arguments)]
fn start_war_effects(
    d: &mut DiplomacyState,
    state: &GameState,
    turn: u32,
    created_turn: u32,
    aggressor: TeamId,
    victim: TeamId,
    with_calls: bool,
    events: &mut Vec<Event>,
) {
    let (joins, calls) = d.declare_wars(|t| state.team_alive(t), aggressor, victim, false);
    let Some(war_id) = d.relation(aggressor, victim).and_then(|r| r.war_id) else {
        return;
    };
    for NewWar { team, enemy } in joins {
        if let Some(relation) = set_war(d, team, enemy, turn, Some(war_id), vec![aggressor]) {
            events.push(Event::WarJoined {
                relation,
                team,
                ally: victim,
            });
        }
    }
    if with_calls {
        for NewWar { team, enemy } in calls {
            raise_call(d, team, aggressor, enemy, created_turn, events);
        }
    }
}

/// Ask `team` to join `ally`'s war on `target`, unless that call is already open.
fn raise_call(
    d: &mut DiplomacyState,
    team: TeamId,
    ally: TeamId,
    target: TeamId,
    created_turn: u32,
    events: &mut Vec<Event>,
) {
    if d.calls
        .iter()
        .any(|c| (c.team, c.ally, c.target) == (team, ally, target))
    {
        return;
    }
    let (Some(alliance), Some(war)) = (d.relation(team, ally), d.relation(ally, target)) else {
        return;
    };
    let (alliance_version, war_version) = (alliance.version, war.version);
    d.next_proposal_id += 1;
    let call = WarCall {
        id: ProposalId(d.next_proposal_id),
        team,
        ally,
        target,
        created_turn,
        respond_by: created_turn + CALL_WINDOW_TURNS - 1,
        alliance_version,
        war_version,
    };
    events.push(Event::CallToArms { call: call.clone() });
    d.calls.push(call);
}

/// Calls-to-arms rules: an Alliance between `a` and `b` was just signed. Each partner's wars
/// where the other defends bring the new ally in at once; the wars the partner declared or joined
/// become calls.
fn alliance_effects(
    d: &mut DiplomacyState,
    state: &GameState,
    turn: u32,
    created_turn: u32,
    a: TeamId,
    b: TeamId,
    events: &mut Vec<Event>,
) {
    let (joins, calls) = d.alliance_wars(|t| state.team_alive(t), a, b);
    for NewWar { team, enemy } in joins {
        let partner = if team == a { b } else { a };
        let Some(src) = d.relation(partner, enemy).cloned() else {
            continue;
        };
        if let Some(relation) = set_war(d, team, enemy, turn, src.war_id, src.declarers) {
            events.push(Event::WarJoined {
                relation,
                team,
                ally: partner,
            });
        }
    }
    for NewWar { team, enemy } in calls {
        let partner = if team == a { b } else { a };
        raise_call(d, team, partner, enemy, created_turn, events);
    }
}

/// A side whose members are at war with each other cannot stand: its Alliances end (only saves
/// and simultaneous orders the checks could not see get here).
fn end_warring_alliances(
    d: &mut DiplomacyState,
    state: &GameState,
    turn: u32,
    events: &mut Vec<Event>,
) {
    for side in alliance_sides(&d.relations) {
        let members: Vec<TeamId> = side.into_iter().filter(|t| state.team_alive(*t)).collect();
        if !members
            .iter()
            .any(|x| members.iter().any(|y| x < y && d.at_war(*x, *y)))
        {
            continue;
        }
        for r in d.relations.iter_mut().filter(|r| {
            r.state == RelationState::Alliance && members.contains(&r.a) && members.contains(&r.b)
        }) {
            if let Some(t) = r.security.take() {
                events.push(Event::TreatyEnded {
                    treaty_id: t.id,
                    a: r.a,
                    b: r.b,
                    effective_turn: turn,
                    reason: "Its members went to war with each other".into(),
                });
            }
            r.state = RelationState::Peace;
            r.effective_turn = turn;
            // Ending an Alliance cannot erase a still-running Peace truce.
            r.version += 1;
        }
    }
}

/// End only the named pair's war, with its own ten-turn guarantee.
fn make_peace(d: &mut DiplomacyState, a: TeamId, b: TeamId, turn: u32) -> Vec<Relation> {
    let mut changed = Vec::new();
    for r in d
        .relations
        .iter_mut()
        .filter(|r| r.between(a, b) && r.state == RelationState::War)
    {
        r.state = RelationState::Peace;
        r.security = None;
        r.effective_turn = turn;
        r.guarantee_until = turn + PEACE_TRUCE_TURNS;
        r.version += 1;
        r.war_id = None;
        r.declarers.clear();
        changed.push(r.clone());
    }
    changed
}

/// Attach the core's current treaty parties to a report event, without storing a derived list
/// in the save. For Peace this lets every participant see who the offer would have bound.
fn proposal_for_event(d: &DiplomacyState, mut p: PeaceProposal) -> PeaceProposal {
    if p.kind == TreatyKind::Peace {
        p.required_teams = d.required_teams(&p).into_iter().collect();
    }
    p
}

/// Called with the untouched planning state and the fully scored resolution state. Neither
/// validation nor teammate priority depends on military outcomes; survival is rechecked here.
pub fn resolve_boundary(
    before: &GameState,
    after: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    events: &mut Vec<Event>,
) {
    let Some(mut d) = after.diplomacy.take() else {
        return;
    };
    d.superseded.clear();
    let effective_turn = before.turn + 1;
    let mut chosen: BTreeMap<(TeamId, TeamId), (PlayerId, Order)> = BTreeMap::new();
    for (&player, os) in orders {
        let team = before.team_of(player);
        for order in os.iter().filter(|o| is_diplomatic(o)) {
            let result = other_team(before, team, order);
            let other = match result {
                Ok(other) => other,
                Err(reason) => {
                    events.push(Event::OrderFailed { player, reason });
                    continue;
                }
            };
            if let Some((winner, winner_order)) = chosen.get(&(team, other)) {
                let s = SupersededOrder {
                    team,
                    player,
                    order: order.clone(),
                    winner: *winner,
                    winner_order: winner_order.clone(),
                    reported_turn: effective_turn,
                };
                events.push(Event::OrderSuperseded {
                    decision: s.clone(),
                });
                d.superseded.push(s);
            } else {
                chosen.insert((team, other), (player, order.clone()));
            }
        }
    }
    let mut valid = Vec::new();
    for ((team, other), (player, order)) in chosen {
        let result = check_order(before, player, &order).and_then(|()| {
            // A vow taken this turn closes the country before a treaty could be signed.
            if let Some(reason) = closed_refusal(after, &d.proposals, player, team, other, &order) {
                Err(reason)
            } else if after.finished {
                Err("The game ended before diplomacy could activate".into())
            } else if !after.team_alive(team) || !after.team_alive(other) {
                Err("A team was eliminated before diplomacy could activate".into())
            } else {
                Ok(())
            }
        });
        match result {
            Ok(()) => valid.push((player, team, other, order)),
            Err(reason) => events.push(Event::OrderFailed { player, reason }),
        }
    }
    end_dead_alliances(&mut d, after, effective_turn, events);
    // Withdrawals and rejections, then declarations, acceptances by ID, new proposals.
    valid.sort_by_key(|(player, _, _, o)| match o {
        Order::WithdrawProposal { proposal }
        | Order::RespondToProposal {
            proposal,
            accept: false,
            ..
        } => (0, proposal.0, *player),
        Order::DeclareWar { .. } | Order::BreakTreaty { .. } => (1, 0, *player),
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } => (2, proposal.0, *player),
        _ => (3, 0, *player),
    });
    // Close proposals before deciding which peace acceptances can veto a call.
    let closing = valid.partition_point(|(_, _, _, order)| {
        matches!(
            order,
            Order::WithdrawProposal { .. } | Order::RespondToProposal { accept: false, .. }
        )
    });
    for (player, _, _, order) in valid.drain(..closing) {
        let proposal = match &order {
            Order::WithdrawProposal { proposal } | Order::RespondToProposal { proposal, .. } => {
                *proposal
            }
            _ => unreachable!("closing proposals sorted first"),
        };
        if let Some(i) = d
            .calls
            .iter()
            .position(|c| c.id == proposal)
            .filter(|_| matches!(order, Order::RespondToProposal { .. }))
        {
            let call = d.calls.remove(i);
            events.push(Event::CallAnswered {
                call,
                joined: false,
                effective_turn,
            });
        } else if let Some(i) = d.proposals.iter().position(|p| p.id == proposal) {
            let mut p = d.proposals.remove(i);
            if let Order::RespondToProposal { explanation, .. } = &order {
                p.explanation = explanation.clone();
            }
            if matches!(order, Order::WithdrawProposal { .. }) {
                events.push(Event::ProposalWithdrawn {
                    proposal: proposal_for_event(&d, p),
                    effective_turn,
                });
            } else {
                cooldown(&mut d, &p, effective_turn);
                events.push(Event::ProposalRejected {
                    proposal: proposal_for_event(&d, p),
                    effective_turn,
                });
            }
        } else {
            events.push(Event::OrderFailed {
                player,
                reason: "Proposal already closed at this boundary".into(),
            });
        }
    }
    // Wars begun and Alliances signed at this boundary: under the calls-to-arms rules their
    // consequences (defenders, calls) are applied once all orders have been.
    let mut began: Vec<(TeamId, TeamId)> = Vec::new();
    let mut signed: Vec<(TeamId, TeamId)> = Vec::new();
    let mut declarations: BTreeMap<(TeamId, TeamId), Vec<TeamId>> = BTreeMap::new();
    for (_, team, other, o) in &valid {
        if matches!(o, Order::DeclareWar { .. }) {
            declarations
                .entry(pair(*team, *other))
                .or_default()
                .push(*team);
        }
    }
    let unilateral: BTreeSet<_> = valid
        .iter()
        .filter(|(_, _, _, o)| matches!(o, Order::DeclareWar { .. } | Order::BreakTreaty { .. }))
        .map(|(_, a, b, _)| pair(*a, *b))
        .collect();
    // Leaving an Alliance (or declaring war on the ally): breakers inside the protected window
    // pay and are recorded as betrayers; war on an ally is always a betrayal.
    let exit_relations = d.clone();
    let mut failed_exits = BTreeSet::new();
    for (player, team, other, o) in &valid {
        if !matches!(o, Order::DeclareWar { .. } | Order::BreakTreaty { .. }) {
            continue;
        }
        let Some(relation) = exit_relations
            .relation(*team, *other)
            .filter(|r| r.state == RelationState::Alliance)
        else {
            continue;
        };
        let treaty = relation.security.clone().expect("alliance terms");
        let cost = leave_cost(relation, effective_turn);
        let betrayal = betrays(&exit_relations, *team, o, effective_turn);
        if after.player(*player).gold < cost {
            failed_exits.insert((*team, *other));
            events.push(Event::OrderFailed {
                player: *player,
                reason: format!("Breaking the Alliance costs {cost} Gold; the treasury ran short"),
            });
            continue;
        }
        pay(
            after,
            *player,
            cost,
            "Alliance breach",
            Some(treaty.id),
            effective_turn,
            events,
        );
        if cost > 0 {
            events.push(Event::TreatyBreached {
                treaty_id: treaty.id,
                by: *team,
                with: *other,
                player: *player,
                cost,
                effective_turn,
            });
        }
        if betrayal {
            let b = Betrayal {
                by: *team,
                against: *other,
                turn: effective_turn,
                until: effective_turn + BETRAYAL_TURNS,
            };
            d.betrayals.push(b.clone());
            events.push(Event::AllianceBetrayed { betrayal: b });
        }
        let r = d
            .relations
            .iter_mut()
            .find(|r| r.between(*team, *other))
            .unwrap();
        if r.state == RelationState::Alliance {
            r.security = None;
            r.state = RelationState::Peace;
            // Leaving an Alliance does not cancel an earlier Peace truce.
            r.effective_turn = effective_turn;
            r.version += 1;
            events.push(Event::TreatyEnded {
                treaty_id: treaty.id,
                a: r.a,
                b: r.b,
                effective_turn,
                reason: format!(
                    "{} left the Alliance",
                    crate::scoring::team_label(after, *team)
                ),
            });
        }
    }
    for (player, team, other, order) in valid {
        match order {
            Order::BreakTreaty { .. } => {} // Ended with the exits above.
            Order::DeclareWar { breach, .. } => {
                if failed_exits.contains(&(team, other)) {
                    continue;
                }
                let Some(mut declarers) = declarations.remove(&pair(team, other)) else {
                    continue;
                };
                declarers.retain(|declarer| {
                    !failed_exits
                        .contains(&(*declarer, if *declarer == team { other } else { team }))
                });
                // A surprise war already began this turn (`declare_surprise_wars`).
                if d.relation(team, other)
                    .is_some_and(|r| r.state == RelationState::War)
                {
                    continue;
                }
                // An Alliance signed or a war joined earlier at this boundary can tie the two
                // together after all.
                if let Some(reason) = d.declare_conflict(after, team, other, breach) {
                    events.push(Event::OrderFailed { player, reason });
                    continue;
                }
                let relation = d.relation(team, other).expect("validated pair");
                if let Err(reason) = check_guarantee(effective_turn, relation, after, other) {
                    events.push(Event::OrderFailed { player, reason });
                    continue;
                }
                let (wars, _) = d.declare_wars(|t| after.team_alive(t), team, other, breach);
                if let Some(reason) = d.join_blocked(after, effective_turn, &wars) {
                    events.push(Event::OrderFailed { player, reason });
                    continue;
                }
                declarers.sort();
                d.next_war_id += 1;
                let r = d
                    .relations
                    .iter_mut()
                    .find(|r| r.between(team, other))
                    .expect("validated pair");
                r.security = None;
                r.state = RelationState::War;
                r.effective_turn = effective_turn;
                r.guarantee_until = 0;
                r.version += 1;
                r.war_id = Some(d.next_war_id);
                r.declarers = declarers.clone();
                events.push(Event::WarDeclared {
                    relation: r.clone(),
                });
                began.extend(
                    declarers
                        .into_iter()
                        .map(|t| (t, if t == team { other } else { team })),
                );
            }
            Order::RespondToProposal {
                proposal,
                accept: true,
                ..
            } => {
                if let Some(i) = d.calls.iter().position(|c| c.id == proposal) {
                    let call = d.calls.remove(i);
                    let blocked = call_void_reason(after, &d, &call)
                        .or_else(|| call_join_blocked(after, &d, &call, effective_turn));
                    if let Some(reason) = blocked {
                        events.push(Event::CallLapsed {
                            call,
                            effective_turn,
                            reason,
                        });
                        continue;
                    }
                    let source = d
                        .relation(call.ally, call.target)
                        .cloned()
                        .expect("validated call");
                    if let Some(relation) = set_war(
                        &mut d,
                        call.team,
                        call.target,
                        effective_turn,
                        source.war_id,
                        vec![call.team],
                    ) {
                        events.push(Event::CallAnswered {
                            call: call.clone(),
                            joined: true,
                            effective_turn,
                        });
                        events.push(Event::WarJoined {
                            relation,
                            team: call.team,
                            ally: call.ally,
                        });
                        start_war_effects(
                            &mut d,
                            after,
                            effective_turn,
                            effective_turn,
                            call.team,
                            call.target,
                            false,
                            events,
                        );
                    }
                    continue;
                }
                let Some(i) = d.proposals.iter().position(|p| p.id == proposal) else {
                    events.push(Event::OrderFailed {
                        player,
                        reason: "Proposal withdrawn or rejected at this boundary".into(),
                    });
                    continue;
                };
                let mut p = d.proposals[i].clone();
                let (p_from, p_to) = (p.from, p.to);
                if let Order::RespondToProposal { explanation, .. } = &order {
                    p.explanation = explanation.clone();
                }
                let r = d.relation(p.from, p.to).expect("validated pair");
                let overtaken = unilateral.contains(&pair(p.from, p.to));
                let reason = if overtaken || r.version != p.relation_version {
                    Some("Relationship changed or a unilateral exit was ordered".to_string())
                } else if !after.player(p.proposer).alive {
                    Some(format!("{} was eliminated", after.player(p.proposer).name))
                } else if p.kind == TreatyKind::Alliance {
                    check_alliance(&d, after, p.from, p.to, effective_turn).err()
                } else {
                    None
                };
                if let Some(reason) = reason {
                    d.proposals.remove(i);
                    events.push(Event::ProposalInvalidated {
                        proposal: proposal_for_event(&d, p),
                        effective_turn,
                        reason,
                    });
                    continue;
                }
                if p.kind == TreatyKind::Peace {
                    // Only the two teams named in the offer must agree.
                    if !p.consents.contains(&team) {
                        p.consents.push(team);
                    }
                    d.proposals[i] = p.clone();
                    if d.awaiting(&p).iter().any(|t| after.team_alive(*t)) {
                        continue;
                    }
                    d.proposals.remove(i);
                    p = proposal_for_event(&d, p);
                    let ended = make_peace(&mut d, p.from, p.to, effective_turn);
                    events.push(Event::ProposalAccepted {
                        proposal: p,
                        effective_turn,
                    });
                    for relation in ended {
                        events.push(Event::PeaceEstablished { relation });
                    }
                    continue;
                }
                d.proposals.remove(i);
                d.next_treaty_id += 1;
                let treaty_id = d.next_treaty_id;
                let security = SecurityTreaty {
                    id: treaty_id,
                    started_turn: effective_turn,
                    protected_until: effective_turn + ALLIANCE_PROTECTED_TURNS,
                    coalition_target: coalition::target_state(before)
                        .filter(|target| *target != p.from && *target != p.to),
                };
                let r = d
                    .relations
                    .iter_mut()
                    .find(|r| r.between(p.from, p.to))
                    .unwrap();
                r.state = RelationState::Alliance;
                r.effective_turn = effective_turn;
                // An Alliance does not cancel an earlier Peace truce.
                r.version += 1;
                r.war_id = None;
                r.declarers.clear();
                r.security = Some(security);
                let relation = r.clone();
                events.push(Event::ProposalAccepted {
                    proposal: p,
                    effective_turn,
                });
                events.push(Event::TreatyActivated { relation });
                signed.push((p_from, p_to));
            }
            Order::ProposeTreaty { kind, .. } => {
                if d.proposals
                    .iter()
                    .any(|p| pair(p.from, p.to) == pair(team, other))
                {
                    events.push(Event::OrderFailed {
                        player,
                        reason: "Another proposal won canonical proposal-ID order for this pair"
                            .into(),
                    });
                    continue;
                }
                let r = d.relation(team, other).expect("validated pair");
                if check_kind(r, kind).is_err() || unilateral.contains(&pair(team, other)) {
                    events.push(Event::OrderFailed {
                        player,
                        reason: "Relationship changed before proposal creation".into(),
                    });
                    continue;
                }
                let version = r.version;
                d.next_proposal_id += 1;
                let p = PeaceProposal {
                    explanation: None,
                    id: ProposalId(d.next_proposal_id),
                    kind,
                    from: team,
                    to: other,
                    proposer: player,
                    created_turn: effective_turn,
                    respond_by: before.turn + 3,
                    relation_version: version,
                    consents: if kind == TreatyKind::Peace {
                        vec![team]
                    } else {
                        Vec::new()
                    },
                    needs: Vec::new(),
                    required_teams: Vec::new(),
                };
                events.push(Event::ProposalCreated {
                    proposal: proposal_for_event(&d, p.clone()),
                });
                d.proposals.push(p);
            }
            _ => unreachable!(),
        }
    }
    // Defenders join at once; the aggressors' allies are called, and a new Alliance's
    // partners join or are called into each other's wars.
    for (aggressor, victim) in began {
        start_war_effects(
            &mut d,
            after,
            effective_turn,
            effective_turn,
            aggressor,
            victim,
            true,
            events,
        );
    }
    for (a, b) in signed {
        alliance_effects(&mut d, after, effective_turn, effective_turn, a, b, events);
    }
    end_warring_alliances(&mut d, after, effective_turn, events);
    // Every offer between two teams whose relation changed this turn is withdrawn: a war
    // declaration (surprise or at the boundary), an Alliance left or bought peace ends them all,
    // so an answered or overtaken offer never waits in the inbox (playtest 8.9.2 alliance loop).
    for p in std::mem::take(&mut d.proposals) {
        let changed = d
            .relation(p.from, p.to)
            .filter(|r| r.version != p.relation_version);
        let reason = if !after.team_alive(p.from) || !after.team_alive(p.to) {
            Some("A team was eliminated")
        } else if after.finished {
            Some("The game ended")
        } else if changed.is_some_and(|r| r.state == RelationState::War) {
            Some(WAR_ENDS_OFFERS)
        } else if changed.is_some() || d.relation(p.from, p.to).is_none() {
            Some("Relationship changed")
        } else {
            None
        };
        if let Some(reason) = reason {
            events.push(Event::ProposalInvalidated {
                proposal: proposal_for_event(&d, p),
                effective_turn,
                reason: reason.into(),
            });
        } else if effective_turn > p.respond_by {
            cooldown(&mut d, &p, effective_turn);
            events.push(Event::ProposalExpired {
                proposal: proposal_for_event(&d, p),
                effective_turn,
            });
        } else {
            d.proposals.push(p);
        }
    }
    {
        let calls = std::mem::take(&mut d.calls);
        let mut open = Vec::new();
        for call in calls {
            if let Some(reason) = call_void_reason(after, &d, &call) {
                events.push(Event::CallLapsed {
                    call,
                    effective_turn,
                    reason,
                });
            } else if effective_turn > call.respond_by {
                events.push(Event::CallLapsed {
                    call,
                    effective_turn,
                    reason: "No answer: the Alliance and the Peace stand".into(),
                });
            } else {
                open.push(call);
            }
        }
        d.calls = open;
    }
    for event in events.iter() {
        let (proposal, accepted) = match event {
            Event::ProposalAccepted { proposal, .. } => (proposal, true),
            Event::ProposalRejected { proposal, .. } => (proposal, false),
            _ => continue,
        };
        if proposal.explanation.is_none() {
            continue;
        }
        d.replies.push(DiplomaticReply {
            proposal: proposal.clone(),
            accepted,
            effective_turn,
        });
    }
    // Retain a useful recent inbox history; betrayal records expire on their own.
    if d.replies.len() > 128 {
        d.replies.drain(..d.replies.len() - 128);
    }
    d.peace_cooldowns.retain(|c| c.retry_turn > effective_turn);
    d.betrayals.retain(|b| effective_turn < b.until);
    after.diplomacy = Some(d);
}

fn cooldown(d: &mut DiplomacyState, p: &PeaceProposal, turn: u32) {
    d.peace_cooldowns
        .retain(|c| (c.from, c.to, c.kind) != (p.from, p.to, p.kind));
    d.peace_cooldowns.push(PeaceCooldown {
        from: p.from,
        to: p.to,
        retry_turn: turn + 5,
        kind: p.kind,
    });
    d.peace_cooldowns.sort_by_key(|c| (c.from, c.to, c.kind));
}

/// Refuse a diplomatic payment the empire cannot make from its current treasury.
fn afford(state: &GameState, player: PlayerId, cost: i32, what: &str) -> Result<(), String> {
    let gold = state.player(player).gold;
    if gold < cost {
        return Err(format!(
            "{what} costs {cost} Gold; you have {gold} (need {} more)",
            cost - gold
        ));
    }
    Ok(())
}
/// Gold for leaving an Alliance with the exit taking effect in `effective`: the breach cost
/// inside the protected window, nothing after it.
pub fn leave_cost(r: &Relation, effective: u32) -> i32 {
    r.security
        .as_ref()
        .filter(|t| effective < t.protected_until)
        .map_or(0, |_| TreatyKind::Alliance.breach_cost())
}
/// Whether this order, taking effect in `effective`, betrays an Alliance: leaving inside the
/// protected window, or declaring war on the ally at any time.
pub fn betrays(d: &DiplomacyState, team: TeamId, o: &Order, effective: u32) -> bool {
    let other = match o {
        Order::BreakTreaty { with, .. } => *with,
        Order::DeclareWar { against, .. } => *against,
        _ => return false,
    };
    d.relation(team, other)
        .filter(|r| r.state == RelationState::Alliance)
        .and_then(|r| r.security.as_ref())
        .is_some_and(|t| matches!(o, Order::DeclareWar { .. }) || effective < t.protected_until)
}
/// "Celts won't ally with Epoch until turn 72: betrayed on turn 52".
pub fn refusal_reason(state: &GameState, b: &Betrayal) -> String {
    format!(
        "{} won't ally with {} until turn {}: betrayed on turn {}",
        crate::scoring::team_label(state, b.against),
        crate::scoring::team_label(state, b.by),
        b.until,
        b.turn
    )
}
/// The treaty an order would sign (Peace when it signs none), for its per-turn upkeep.
fn signed_kind(d: &DiplomacyState, o: &Order) -> TreatyKind {
    match o {
        Order::ProposeTreaty { kind, .. } => *kind,
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } => d
            .proposals
            .iter()
            .find(|p| p.id == *proposal)
            .map_or(TreatyKind::Peace, |p| p.kind),
        _ => TreatyKind::Peace,
    }
}
fn check_guarantee(
    planning_turn: u32,
    r: &Relation,
    state: &GameState,
    other: TeamId,
) -> Result<(), String> {
    if planning_turn < r.guarantee_until {
        Err(format!(
            "Peace with {} is guaranteed through turn {}; earliest War: turn {}",
            empire_label(state, other),
            r.guarantee_until - 1,
            r.guarantee_until
        ))
    } else {
        Ok(())
    }
}
/// The faction-first empire name used by the UI; personal names distinguish duplicate factions.
fn empire_label(state: &GameState, team: TeamId) -> String {
    state
        .team_members(team)
        .into_iter()
        .map(|id| {
            let p = state.player(id);
            let faction = crate::data::faction_name(p.faction);
            if state
                .players
                .iter()
                .any(|o| o.team != team && o.faction == p.faction)
            {
                format!("{faction} · {}", p.name)
            } else {
                faction.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" & ")
}
fn check_kind(r: &Relation, kind: TreatyKind) -> Result<(), String> {
    match (kind, r.state) {
        (TreatyKind::Alliance, RelationState::Peace) | (TreatyKind::Peace, RelationState::War) => {
            Ok(())
        }
        (TreatyKind::Alliance, RelationState::War) => Err(
            "You are at war with them, or an ally of yours is: make peace before an Alliance"
                .into(),
        ),
        (TreatyKind::Alliance, RelationState::Alliance) => {
            Err("You are already allied with them".into())
        }
        (TreatyKind::Peace, _) => Err("Peace offers require War".into()),
    }
}
/// The Gold an order costs the ordering empire: breaking an Alliance inside its protected window.
pub fn order_cost(
    _state: &GameState,
    d: &DiplomacyState,
    team: TeamId,
    o: &Order,
    effective: u32,
) -> i32 {
    match o {
        Order::BreakTreaty { with, .. }
        | Order::DeclareWar {
            against: with,
            breach: true,
        } => d
            .relation(team, *with)
            .filter(|r| r.state == RelationState::Alliance)
            .map_or(0, |r| leave_cost(r, effective)),
        _ => 0,
    }
}
/// Pay a diplomatic Gold cost for breaking an Alliance early. Orders and
/// activations are refused when unaffordable, so the treasury never goes negative here.
pub(crate) fn pay(
    state: &mut GameState,
    player: PlayerId,
    gold: i32,
    cause: &str,
    treaty_id: Option<u32>,
    effective_turn: u32,
    events: &mut Vec<Event>,
) {
    if gold == 0 {
        return;
    }
    state.player_mut(player).gold -= gold;
    events.push(Event::TreatyPayment {
        player,
        gold,
        cause: cause.into(),
        treaty_id,
        effective_turn,
    });
}
/// Alliances end when either team is eliminated.
fn end_dead_alliances(
    d: &mut DiplomacyState,
    state: &GameState,
    turn: u32,
    events: &mut Vec<Event>,
) {
    for r in &mut d.relations {
        if r.state != RelationState::Alliance || (state.team_alive(r.a) && state.team_alive(r.b)) {
            continue;
        }
        let Some(t) = r.security.take() else {
            continue;
        };
        r.state = RelationState::Peace;
        r.effective_turn = turn;
        // Elimination does not change the recorded Peace truce.
        r.version += 1;
        events.push(Event::TreatyEnded {
            treaty_id: t.id,
            a: r.a,
            b: r.b,
            effective_turn: turn,
            reason: "Team eliminated".into(),
        });
    }
}

pub fn shares_sight(r: &Relation) -> bool {
    r.state == RelationState::Alliance
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionExplanation {
    pub score: Option<i32>,
    pub reasons: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomaticReply {
    pub proposal: PeaceProposal,
    pub accepted: bool,
    pub effective_turn: u32,
}
/// Explanations are a receipt, not part of a diplomatic action's legality or cost.
pub fn same_action(a: &Order, b: &Order) -> bool {
    match (a, b) {
        (
            Order::RespondToProposal {
                proposal: a,
                accept: aa,
                ..
            },
            Order::RespondToProposal {
                proposal: b,
                accept: ba,
                ..
            },
        ) => a == b && aa == ba,
        _ => a == b,
    }
}
