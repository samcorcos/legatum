//! Kyivan Rus' (proposal): marriage alliances, the steppe, the Druzhina, the Lavra and the
//! father-in-law of Europe.
//!
//! - **Marriage alliances** (`FactionRule::RusMarriage`): a unit a Rus' city completes is an allied
//!   people's version of it when that people's special unit replaces that kind
//!   (`data::special_unit`), at that version's price. Units record the people that made them
//!   (`Unit::origin`, as hired Companies do), so the unit keeps the version for life, whatever
//!   becomes of the Alliance. The Knight is the Rus' own Druzhina and is never an ally's. With
//!   two allies whose special units replace the same kind, the older Alliance's version wins.
//! - **The steppe** (`FactionRule::RusSteppe`): a camp whose raid on a Rus' city ends waits
//!   `STEPPE_RAID_COOLDOWN` turns before it raids again, not `barbarians::RAID_COOLDOWN`.
//! - **The Lavra** (`FactionRule::RusLavra`): the Rus' Library adds `LAVRA_SCIENCE_PER_ALLIANCE`
//!   Science for each Alliance its owner's team holds (`economy`).
//! - **The father-in-law of Europe** (`goals::GoalRule::Marriages`, Live, 2 VP): hold Alliances
//!   with `GOAL_ALLIANCES` living teams at once.
//!
//! Every rule reads public facts (factions, Alliances and the turns they began), so the player
//! view answers the same questions as the state (`known_*`).

use crate::data;
use crate::diplomacy::{Relation, RelationState};
use crate::experiments::{FactionRule, RuleExperiments};
use crate::state::GameState;
use crate::types::*;
use crate::view::PlayerView;

/// The father-in-law of Europe: Alliances the Rus' must hold at once.
pub const GOAL_ALLIANCES: i32 = 3;
/// The Lavra: Science each Alliance adds to it.
pub const LAVRA_SCIENCE_PER_ALLIANCE: i32 = 2;
/// The steppe: turns a camp waits after a raid on a Rus' city (every other city: 30).
pub const STEPPE_RAID_COOLDOWN: u32 = 15;

/// Whether `faction` trains its allies' special units under `rules`.
pub fn marries(rules: RuleExperiments, faction: Faction) -> bool {
    faction == Faction::Rus && !rules.rule_off(FactionRule::RusMarriage)
}

/// The teams `team` holds an Alliance with that pass `alive`, oldest Alliance first (the turn
/// it began, then team id).
fn ordered_allies(
    relations: &[Relation],
    team: TeamId,
    alive: impl Fn(TeamId) -> bool,
) -> Vec<TeamId> {
    let mut allies: Vec<(u32, TeamId)> = relations
        .iter()
        .filter(|r| r.state == RelationState::Alliance && (r.a == team || r.b == team))
        .map(|r| {
            let other = if r.a == team { r.b } else { r.a };
            let began = r
                .security
                .as_ref()
                .map_or(r.effective_turn, |s| s.started_turn);
            (began, other)
        })
        .filter(|&(_, t)| t != team && t != TeamId::BARBARIAN && alive(t))
        .collect();
    allies.sort();
    allies.into_iter().map(|(_, t)| t).collect()
}

/// The first ally (in `allies` order) with a member, in `members` order, whose special unit
/// replaces `kind`. The Knight stays the Rus' own Druzhina.
fn pick(
    allies: &[TeamId],
    members: impl Fn(TeamId) -> Vec<Faction>,
    kind: UnitKind,
) -> Option<Faction> {
    if kind == data::special_unit(Faction::Rus) {
        return None;
    }
    allies.iter().find_map(|&t| {
        members(t)
            .into_iter()
            .find(|f| *f != Faction::Rus && data::special_unit(*f) == kind)
    })
}

/// Living teams `p`'s team is allied with, oldest Alliance first.
pub fn allies(state: &GameState, p: PlayerId) -> Vec<TeamId> {
    if p.is_barbarian() {
        return Vec::new();
    }
    let team = state.team_of(p);
    state.diplomacy.as_ref().map_or_else(Vec::new, |d| {
        ordered_allies(&d.relations, team, |t| state.team_alive(t))
    })
}

/// Alliances `p`'s team holds with living teams.
pub fn alliances(state: &GameState, p: PlayerId) -> i32 {
    allies(state, p).len() as i32
}

/// The people whose version of `kind` a unit `p`'s cities complete now is, when it is not
/// `p`'s own (marriage alliances).
pub fn version_for(state: &GameState, p: PlayerId, kind: UnitKind) -> Option<Faction> {
    if p.is_barbarian() || !marries(state.setup.rule_experiments, state.player(p).faction) {
        return None;
    }
    let members = |t: TeamId| {
        state
            .players
            .iter()
            .filter(|q| q.team == t && q.alive)
            .map(|q| q.faction)
            .collect()
    };
    pick(&allies(state, p), members, kind)
}

/// The people whose version of `kind` the price and stat line of an item `p` trains follow.
pub fn cost_faction(state: &GameState, p: PlayerId, item: ProductionItem) -> Faction {
    match item {
        ProductionItem::Unit { unit } => version_for(state, p, unit),
        _ => None,
    }
    .unwrap_or_else(|| state.player(p).faction)
}

/// `allies` from viewer `view.me`'s view.
pub fn known_allies(view: &PlayerView) -> Vec<TeamId> {
    ordered_allies(&view.diplomacy.relations, view.team, |t| {
        view.players.iter().any(|q| q.team == t && q.alive)
    })
}

/// `alliances` from viewer `view.me`'s view.
pub fn known_alliances(view: &PlayerView) -> i32 {
    known_allies(view).len() as i32
}

/// `version_for` for the viewer's own cities, from its view.
pub fn known_version_for(view: &PlayerView, kind: UnitKind) -> Option<Faction> {
    if !marries(view.rule_experiments, view.faction_of(view.me)) {
        return None;
    }
    let members = |t: TeamId| {
        view.players
            .iter()
            .filter(|q| q.team == t && q.alive)
            .map(|q| q.faction)
            .collect()
    };
    pick(&known_allies(view), members, kind)
}

/// The peoples `team` would add to the viewer's allied versions if it became an ally: the unit
/// kinds its members' special units replace that no current ally already gives and that are not
/// the Druzhina's. Empty unless the viewer marries.
pub fn known_new_versions(view: &PlayerView, team: TeamId) -> Vec<UnitKind> {
    if !marries(view.rule_experiments, view.faction_of(view.me)) {
        return Vec::new();
    }
    let mut kinds: Vec<UnitKind> = view
        .players
        .iter()
        .filter(|q| q.team == team && q.alive && q.faction != Faction::Rus)
        .map(|q| data::special_unit(q.faction))
        .filter(|&k| k != data::special_unit(Faction::Rus) && known_version_for(view, k).is_none())
        .collect();
    kinds.sort();
    kinds.dedup();
    kinds
}

/// The Lavra's Science from Alliances in a city of `p` that has a Library.
pub fn lavra_science(state: &GameState, p: PlayerId) -> i32 {
    if p.is_barbarian()
        || state.player(p).faction != Faction::Rus
        || state.setup.rule_experiments.rule_off(FactionRule::RusLavra)
    {
        return 0;
    }
    LAVRA_SCIENCE_PER_ALLIANCE * alliances(state, p)
}

/// The steppe: whether raids on `owner`'s cities come again sooner.
pub fn steppe(state: &GameState, owner: PlayerId) -> bool {
    !owner.is_barbarian()
        && state.player(owner).faction == Faction::Rus
        && !state
            .setup
            .rule_experiments
            .rule_off(FactionRule::RusSteppe)
}

/// The turns a camp waits after its raid on `city` ends (`barbarians::muster`): shorter for a
/// Rus' city, the ordinary cooldown otherwise (and for a city that no longer stands).
pub fn raid_cooldown(state: &GameState, city: CityId, ordinary: u32) -> u32 {
    match state.cities.get(&city) {
        Some(c) if steppe(state, c.owner) => STEPPE_RAID_COOLDOWN.min(ordinary),
        _ => ordinary,
    }
}

#[cfg(test)]
#[path = "rus_tests.rs"]
mod tests;
