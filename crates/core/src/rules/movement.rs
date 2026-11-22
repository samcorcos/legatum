//! Movement in rounds with simultaneous intent resolution: the heart of simultaneous turns.
//!
//! Every player's units move at the same time. Movement runs in rounds until nobody declares a
//! step. Hostile contacts are simultaneous; same-owner traffic takes priority by lowest unit id.
//!
//! # One round
//!
//! 1. **Snapshot and declare.** Every unit with a goto, moves left and not stopped for the turn
//!    declares one step: the first tile of its path to the goto. The path comes from the land
//!    pathfinder, routing around tiles held by hostile units or cities (unless that tile is the
//!    goto: stepping there is an attack). A lower-id friend's reserved step can cost one
//!    movement round of waiting; it is routed around only when that arrives sooner in turns.
//!    **Friends that stay put are passed through, never ended on** (Civ 6, user
//!    2026-10-21): a same-class friend of the unit's team that is parked, out of moves or
//!    attacking in place is a tile the route may cross. A route that starts across such
//!    friends is one atomic **hop**: the unit lands on the first tile beyond them and pays
//!    for every tile it crossed, so no round ever ends with it on a friend. A hop it cannot
//!    pay for this round waits for next turn on the last free tile it passed; with no
//!    landing (an enemy beyond, the goto is the friend's tile, a run of friends too dear to
//!    cross) the step into the friend fails as a block. A route avoiding friends that move
//!    out wins whenever it arrives in as few turns (`path::route_around_friends`, the
//!    costing shared with the AI and the client; `path::hop`). A confirmed step is used only
//!    while it passes these same checks; otherwise the
//!    confirmation is cleared and repathed, unless the order requires stopping on a block.
//!    A constrained route keeps its destination and waits for a new order. If no such
//!    route exists the plain terrain path is used. A unit may step if it
//!    has moves ≥ the step cost, or always at full moves (so every unit can take one step per
//!    turn). A unit that cannot step stops for the turn. No path at all: goto cleared.
//! 2. **Classify each step** against the snapshot:
//!    - a civilian stepping into a tile with any hostile unit or city is blocked;
//!    - a military unit stepping into a hostile military unit's tile attacks it (a garrison
//!      absorbs the attack on its city), or, with no hostile military there, a hostile city's
//!      tile attacks the city. If the defender is simultaneously stepping into the attacker's
//!      tile (a swap), the pair fights one **meeting engagement** instead;
//!    - a military unit stepping onto a lone hostile civilian moves in and captures it if it
//!      advances;
//!    - anything else is a plain move.
//! 3. **Engagements.** Melee attacks (defender gets terrain and fortify), swap meetings, and a
//!    meeting for every hostile military pair claiming the same empty tile or an ungarrisoned
//!    0-HP city (three-way contests: every hostile pair fights once). The ruined city does not
//!    absorb these attacks. Two enemies of the city's owner never meet there: force decides
//!    which of them enters (`settle_by_force`). Meetings use `combat::unit_meeting_strength` for both sides.
//!    **Strike, then retreat** (G21): a unit attacked in melee while stepping away takes the
//!    hit where it stands, without fortification (terrain still counts), and if it survives it
//!    completes its step. Outrider is the exception when melee began beyond adjacency: a
//!    pre-Modern horse that steps away evades the attack. Its attacker stays put unless it kills
//!    it. A civilian that a hostile
//!    military unit steps onto is still pinned and captured: a capture is its hit.
//! 4. **Damage.** Every fight is computed from the snapshot and all damage is applied together,
//!    so mutual kills happen. Every unit that attacked (including meeting participants) has its
//!    movement ended for the turn and its fortification cleared. **One attack per order**
//!    (G18, Civ 6): a fighter whose opponent survived ends its order when movement is over. It
//!    stays where it fought with no destination and needs orders; it never re-attacks next turn.
//!    **Peaceful contests** (Sam, 2026-11-22). Units of different teams that are not at war
//!    with each other stepping onto the same tile (no city there) never fight: the team with
//!    the most combined strength among its claimants (meeting strength after damage; civilians
//!    0) keeps its claim, as force decides a fallen city (`settle_by_force`), and every other
//!    team's claimant stays put with a MoveBlocked naming the winner and both totals. Equal
//!    best totals are drawn by seed, turn and tile (`settle_peaceful_contest`).
//! 5. **Advance, military first.** A military claimant of tile T advances iff it survived, is
//!    not pinned, wins the surviving claims to T, no hostile military unit is left
//!    alive on T, any hostile city on T is at 0 HP (capture), and a
//!    friendly military unit on T advances out of it. The last rule makes friendly chains and
//!    cycles resolve together: every candidate starts as succeeding and failures propagate
//!    along "depends on the unit leaving my target" links to a fixpoint, so a blocked link
//!    blocks everything behind it while a closed cycle all moves. Two or more friendly
//!    claimants of one tile take turns by unit id if they share an owner. Other friendly
//!    claimants bounce except at a defeated enemy city: the healthiest eligible
//!    teammate captures, with ties drawn from seed, turn and tile after sorting by approach.
//!    The owner's units stepping into their own assaulted city are not claimants but its
//!    supporters (designer, 2026-11-15): they share with the garrison what gets past the
//!    city's HP, each adds its counterattack, and if the garrison fell the one that can soak
//!    the most enters as garrison (`reserves`, `battle::carried_damage`, `battle::soak`).
//! 6. **Advance, civilians** by the same rules, with military final positions known: a
//!    civilian cannot end on a tile a hostile military unit holds or enters.
//! 7. **Apply.** Advancing units move and pay the step cost (0 moves left if they fought);
//!    a non-ranged military unit entering a 0 HP city captures it; a military unit ending on a hostile
//!    civilian captures it (0 moves); the first unit to enter a ruin explores it. Units that did
//!    not advance stop for the turn, except waiting own traffic when their owner made progress.
//!    Newly seen tiles are revealed.
//!
//! Waiting same-owner traffic retries only after a round with an advance by their owner. Each such round
//! spends at least one move; a round without progress stops every claimant, so movement terminates.

use super::battle::{self, Fight, FightKind, Target};
use super::units::explore_ruin;
use super::{hostile, is_military};
use crate::combat;
use crate::data;
use crate::events::{Event, PursuitOutcome};
use crate::hex::Hex;
use crate::path;
use crate::rng::{Rng, purpose};
use crate::state::{GameState, Unit};
use crate::types::*;
use crate::visibility;
use std::collections::{BTreeMap, BTreeSet};

/// `path::exit_distances` for `open` tiles on the real map, with every city centre a wall: a
/// unit stranded in closed land never routes out through a foreign city, since it could not
/// stop there (game-rules "Passage and closed borders").
fn exits_from(state: &GameState, open: impl Fn(Hex) -> bool) -> Vec<i32> {
    let cities: BTreeSet<Hex> = state.cities.values().map(|c| c.pos).collect();
    path::exit_distances_around(&state.map, open, |h| cities.contains(&h))
}

/// A unit may step if it can pay for it, or always when it still has full moves.
pub fn can_step(moves_left: i32, max_moves: i32, cost: i32) -> bool {
    moves_left > 0 && (moves_left >= cost || moves_left == max_moves)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Move,
    Attack(Target),
    /// Swap with a hostile military unit: one meeting engagement.
    Meet(UnitId),
}

#[derive(Clone, Copy, Debug)]
struct Intent {
    unit: UnitId,
    owner: PlayerId,
    military: bool,
    from: Hex,
    to: Hex,
    cost: i32,
    step: Step,
}

#[derive(Clone, Copy, Debug, Default)]
struct Occ {
    military: Option<UnitId>,
    civilian: Option<UnitId>,
    city: Option<CityId>,
}

fn occupancy(state: &GameState) -> BTreeMap<Hex, Occ> {
    let mut occ: BTreeMap<Hex, Occ> = BTreeMap::new();
    for u in state.units.values() {
        let o = occ.entry(u.pos).or_default();
        let slot = if is_military(u.kind) {
            &mut o.military
        } else {
            &mut o.civilian
        };
        slot.get_or_insert(u.id);
    }
    for c in state.cities.values() {
        occ.entry(c.pos).or_default().city.get_or_insert(c.id);
    }
    occ
}

/// Phase 3: movement rounds until nobody steps.
pub fn resolve_movement(state: &mut GameState, events: &mut Vec<Event>) {
    let mut threats = super::units::sleeping_threats(state);
    resolve_movement_with_paths(
        state,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &mut threats,
        None,
        Vec::new(),
        events,
    );
}

pub fn resolve_movement_with_paths(
    state: &mut GameState,
    paths: &BTreeMap<UnitId, Vec<Hex>>,
    awaiting_war: &BTreeSet<UnitId>,
    threats: &mut super::units::SleepThreats,
    turn_start: Option<&BTreeMap<UnitId, Hex>>,
    mut held: Vec<battle::HeldShot>,
    events: &mut Vec<Event>,
) {
    let fallback: BTreeMap<UnitId, Hex> = state.units.values().map(|u| (u.id, u.pos)).collect();
    let turn_start = turn_start.unwrap_or(&fallback);
    let mut stopped = BTreeSet::new();
    let mut round = 0u64;
    let mut pursuit = start_pursuits(state, awaiting_war, turn_start, &mut stopped, events);
    // A move onto a city of a team declared on this turn holds until War opens its borders.
    for id in awaiting_war {
        if state
            .units
            .get(id)
            .is_some_and(|u| u.attack_target.is_none() && u.goto.is_some())
        {
            stopped.insert(*id);
        }
    }
    // Manual moves, including constrained routes: checked for no progress below.
    let starts: BTreeMap<UnitId, Hex> = state
        .units
        .values()
        .filter(|u| u.goto.is_some_and(|g| g != u.pos) && !paths.contains_key(&u.id))
        .map(|u| (u.id, u.pos))
        .collect();
    let first_event = events.len();
    super::units::wake_threatened_sleepers(state, threats, events);
    while {
        aim_pursuits(state, &pursuit, &mut stopped);
        movement_round(
            state,
            round,
            paths,
            &mut stopped,
            &mut pursuit,
            &mut held,
            events,
        )
    } {
        super::units::wake_threatened_sleepers(state, threats, events);
        round += 1;
    }
    // Held shots whose city nobody came in to defend land on it as before.
    battle::release_held_shots(state, &held, events);
    finish_pursuits(state, &pursuit, first_event, events);
    end_engaged_orders(state, &pursuit.engaged);
    stop_blocked_moves(state, &starts, first_event, events);
    for (id, path) in paths {
        if let Some(u) = state.units.get_mut(id) {
            if path.last().is_some_and(|h| *h != u.pos) && u.automation.take().is_some() {
                events.push(Event::AutomationStopped {
                    unit: *id,
                    owner: u.owner,
                    at: u.pos,
                    reason: ExploreStopReason::Blocked,
                });
            }
            u.goto = None;
        }
    }
}

/// This turn's melee attacks on units (`Unit::attack_target`). Normally the attack follows the
/// unit: each round a pursuer's goto is re-aimed at its target. A distant melee attack on a
/// pre-Modern horse keeps its original aim; if the horse moves, the attack ends. Every round
/// still resolves from one
/// snapshot, so an attacker that starts a round adjacent pins its target (contact pins) and
/// nobody gains from unit ids or seats. The order lasts this turn only (G18): whether it
/// attacked, lost its target or could not reach it, it ends when movement is over, except
/// while it waits for a War declared this turn.
#[derive(Default)]
struct Pursuit {
    /// Pursuer -> (target, the target's position when movement began).
    chasing: BTreeMap<UnitId, (UnitId, Hex)>,
    /// Distant melee attacks on pre-Modern horses keep aiming at the horse's turn-start tile.
    outrider: BTreeSet<UnitId>,
    /// Pursuers holding for a War declared this turn.
    awaiting: BTreeSet<UnitId>,
    /// Pursuer -> the tile where it fought its target.
    fought: BTreeMap<UnitId, Hex>,
    /// Units whose melee fight or meeting this turn left the other side standing. Their order
    /// ends after movement: one attack per order, never a standing re-attack (G18).
    engaged: BTreeSet<UnitId>,
}

/// One attack per order (G18): a unit that fought and left its opponent standing keeps its
/// tile, drops its destination, route and attack target, and needs orders. A winner that could
/// not advance keeps its move onto the now empty tile.
fn end_engaged_orders(state: &mut GameState, engaged: &BTreeSet<UnitId>) {
    for id in engaged {
        if let Some(u) = state.units.get_mut(id) {
            u.goto = None;
            u.path.clear();
            u.path_stop_on_block = false;
            u.attack_target = None;
        }
    }
}

fn team_sees(
    state: &GameState,
    cache: &mut BTreeMap<TeamId, crate::bitset::BitSet>,
    player: PlayerId,
    h: Hex,
) -> bool {
    let team = state.team_of(player);
    let seen = cache
        .entry(team)
        .or_insert_with(|| visibility::visible_tiles(state, team));
    state.map.index(h).is_some_and(|i| seen.get(i))
}

fn end_pursuit(state: &mut GameState, unit: UnitId) {
    let u = state.units.get_mut(&unit).expect("pursuer");
    u.attack_target = None;
    u.goto = None;
    u.path.clear();
    u.path_stop_on_block = false;
}

/// The target is gone (destroyed, or captured by our side): the order is void, not merely
/// finished, so the unit asks for new orders.
fn void_pursuit(state: &mut GameState, unit: UnitId) {
    end_pursuit(state, unit);
    state.units.get_mut(&unit).expect("pursuer").needs_orders = true;
}

fn aim_at(state: &mut GameState, unit: UnitId, at: Hex) {
    let u = state.units.get_mut(&unit).expect("pursuer");
    if u.goto != Some(at) {
        u.goto = Some(at);
        u.path.clear();
        u.path_stop_on_block = false;
    }
}

/// MoveBlocked reason for an attack whose target was destroyed before movement, with nothing
/// hostile left on its tile.
pub(crate) const TARGET_GONE: &str = "its target was destroyed before it could attack";

/// The enemy an attack order on `at` goes for when its first target is gone: a visible hostile
/// unit there, military before civilian, then the lowest id (as `units::explicit_attack_target`
/// picks). None when the tile holds a city (that is an attack on the city) or nothing hostile.
fn next_target_on(state: &GameState, player: PlayerId, at: Hex) -> Option<UnitId> {
    if state
        .cities
        .values()
        .any(|c| c.pos == at && !state.same_team(player, c.owner))
    {
        return None;
    }
    let seen = visibility::visible_tiles(state, state.team_of(player));
    if !state.map.index(at).is_some_and(|i| seen.get(i)) {
        return None;
    }
    state
        .units
        .values()
        .filter(|u| u.pos == at && hostile(state, player, u.owner))
        .min_by_key(|u| (!is_military(u.kind), u.id))
        .map(|u| u.id)
}

fn start_pursuits(
    state: &mut GameState,
    awaiting_war: &BTreeSet<UnitId>,
    turn_start: &BTreeMap<UnitId, Hex>,
    stopped: &mut BTreeSet<UnitId>,
    events: &mut Vec<Event>,
) -> Pursuit {
    let mut p = Pursuit::default();
    let ids: Vec<UnitId> = state
        .units
        .values()
        .filter(|u| u.attack_target.is_some())
        .map(|u| u.id)
        .collect();
    let mut sight = BTreeMap::new();
    for id in ids {
        let u = &state.units[&id];
        let (owner, target) = (u.owner, u.attack_target.expect("target"));
        let target = if state.units.contains_key(&target) {
            target
        } else if let Some(next) = u.goto.and_then(|at| next_target_on(state, owner, at)) {
            // The target died before movement (ranged fire): the attack goes on against the next
            // enemy on that tile, the one the order would have picked had it stood alone (Sam,
            // 2026-11-21, turn 75: a Keshig's Horseman target fell to a shot and the Settler
            // beside it walked away).
            state.units.get_mut(&id).expect("pursuer").attack_target = Some(next);
            next
        } else {
            target
        };
        let u = &state.units[&id];
        let Some(t) = state.units.get(&target).filter(|_| u.goto.is_some()) else {
            // The target died before movement (ranged fire, disbanding) and nothing hostile is
            // left on its tile: the order ends, and says so.
            if let Some(toward) = u.goto {
                events.push(Event::MoveBlocked {
                    unit: id,
                    owner,
                    at: u.pos,
                    toward,
                    reason: TARGET_GONE.into(),
                });
            }
            void_pursuit(state, id);
            continue;
        };
        let escapes = if let (Some(a), Some(d)) = (turn_start.get(&id), turn_start.get(&target)) {
            crate::combat::outrider_blocks(u.kind, t.kind, a.distance(*d))
        } else {
            false
        };
        let (at, target_owner) = (t.pos, t.owner);
        if escapes {
            p.outrider.insert(id);
        }
        if !hostile(state, owner, target_owner) {
            if awaiting_war.contains(&id) {
                p.awaiting.insert(id);
                stopped.insert(id);
                aim_at(state, id, at);
            } else {
                // No War after all: an ordinary move, which never attacks a peaceful unit.
                state.units.get_mut(&id).expect("unit").attack_target = None;
            }
            continue;
        }
        if team_sees(state, &mut sight, owner, at) {
            aim_at(state, id, at);
        } else {
            stopped.insert(id);
        }
        p.chasing.insert(id, (target, at));
    }
    p
}

/// Before each round: aim pursuers at their target's current tile. A pursuer whose target
/// left its team's sight stops (reported at the end), never walking on into the unknown.
fn aim_pursuits(state: &mut GameState, p: &Pursuit, stopped: &mut BTreeSet<UnitId>) {
    let mut sight = BTreeMap::new();
    for (&id, &(target, _)) in &p.chasing {
        let Some(u) = state.units.get(&id) else {
            continue;
        };
        if stopped.contains(&id) || u.attack_target != Some(target) {
            continue;
        }
        let owner = u.owner;
        if p.outrider.contains(&id) {
            if state
                .units
                .get(&target)
                .is_some_and(|t| t.pos != p.chasing[&id].1)
            {
                stopped.insert(id);
            }
            continue;
        }
        // Destroyed, or a civilian target already captured by our side: the order is done.
        match state
            .units
            .get(&target)
            .filter(|t| !state.same_team(owner, t.owner))
            .map(|t| t.pos)
        {
            None => void_pursuit(state, id),
            Some(at) if team_sees(state, &mut sight, owner, at) => aim_at(state, id, at),
            Some(_) => {
                stopped.insert(id);
            }
        }
    }
}

/// After movement: report every pursuit and end it. Only a pursuer waiting for War stays
/// aimed at its target, to attack on War's first turn.
fn finish_pursuits(
    state: &mut GameState,
    p: &Pursuit,
    first_event: usize,
    events: &mut Vec<Event>,
) {
    let mut sight = BTreeMap::new();
    let mut entries: Vec<(UnitId, Option<(UnitId, Hex)>)> =
        p.chasing.iter().map(|(u, c)| (*u, Some(*c))).collect();
    entries.extend(p.awaiting.iter().map(|u| (*u, None)));
    for (id, chase) in entries {
        let Some(u) = state.units.get(&id) else {
            continue;
        };
        let (owner, at) = (u.owner, u.pos);
        let Some(target) = chase.map(|c| c.0).or(u.attack_target) else {
            continue;
        };
        let start = chase.map(|c| c.1);
        let target_now = state.units.get(&target).map(|t| (t.owner, t.kind, t.pos));
        // A target destroyed this turn is named from its last battle.
        let who = target_now.map(|t| (t.0, t.1)).or_else(|| {
            events[first_event..].iter().rev().find_map(|e| match e {
                Event::Battle {
                    defender:
                        crate::events::Defender::Unit {
                            unit,
                            owner,
                            unit_kind,
                        },
                    ..
                } if *unit == target => Some((*owner, *unit_kind)),
                _ => None,
            })
        });
        let mut report = |outcome, target_at| {
            if let Some((target_owner, target_kind)) = who {
                events.push(Event::AttackPursuit {
                    unit: id,
                    owner,
                    at,
                    target,
                    target_owner,
                    target_kind,
                    target_at,
                    outcome,
                });
            }
        };
        let fought = p.fought.get(&id).copied();
        if fought.is_some() && fought != start {
            report(PursuitOutcome::Attacked, fought);
        }
        let Some((_, _, target_pos)) = target_now.filter(|t| !state.same_team(owner, t.0)) else {
            // Destroyed or captured: the order is complete.
            void_pursuit(state, id);
            continue;
        };
        if state.units[&id].attack_target != Some(target) {
            // Ended by movement itself (no route left): its MoveBlocked explains.
            continue;
        }
        if p.outrider.contains(&id) && start.is_some_and(|h| h != target_pos) {
            let (target_owner, target_kind, _) = target_now.expect("escaping target");
            let target_faction = state.player(target_owner).faction;
            let name = crate::data::unit_name_for(target_faction, target_kind);
            let adjective = crate::data::faction_adjective(target_faction);
            let attacker_name =
                crate::data::unit_name_for(state.player(owner).faction, state.units[&id].kind);
            end_pursuit(state, id);
            events.push(Event::MoveBlocked {
                unit: id,
                owner,
                at,
                toward: start.expect("Outrider target tile"),
                reason: format!(
                    "The {adjective} {name} rode away before your {attacker_name} reached it."
                ),
            });
            continue;
        }
        if !team_sees(state, &mut sight, owner, target_pos) {
            end_pursuit(state, id);
            report(PursuitOutcome::LostSight, None);
            continue;
        }
        if p.awaiting.contains(&id) {
            aim_at(state, id, target_pos);
            report(PursuitOutcome::AwaitingWar, Some(target_pos));
            continue;
        }
        // One turn, one attack (G18): the order never carries over to another turn.
        end_pursuit(state, id);
        if fought.is_none() {
            let outcome = if Some(target_pos) != start {
                PursuitOutcome::Followed
            } else {
                PursuitOutcome::OutOfReach
            };
            report(outcome, Some(target_pos));
        }
    }
}

/// Turns a move may make no progress before it stops and the unit asks for new orders.
pub const MOVE_BLOCKED_STOP_TURNS: u8 = 2;

/// A move whose next step stayed blocked (a peaceful foreign unit, a parked unit, anything it
/// cannot enter) for `MOVE_BLOCKED_STOP_TURNS` consecutive turns stops, says what blocks it,
/// and returns the unit to its owner's decision cycle instead of waiting silently forever.
fn stop_blocked_moves(
    state: &mut GameState,
    starts: &BTreeMap<UnitId, Hex>,
    first_event: usize,
    events: &mut Vec<Event>,
) {
    let mut blocked: BTreeMap<UnitId, Hex> = BTreeMap::new();
    for e in &events[first_event..] {
        if let Event::MoveBlocked { unit, toward, .. } = e {
            blocked.insert(*unit, *toward);
        }
    }
    let ids: Vec<UnitId> = state.units.keys().copied().collect();
    for id in ids {
        let u = &state.units[&id];
        let stuck = starts.get(&id) == Some(&u.pos) && u.goto.is_some();
        let Some(&toward) = blocked.get(&id).filter(|_| stuck) else {
            if u.blocked_turns != 0 {
                state.units.get_mut(&id).expect("unit").blocked_turns = 0;
            }
            continue;
        };
        let turns = u.blocked_turns.saturating_add(1);
        if turns < MOVE_BLOCKED_STOP_TURNS {
            state.units.get_mut(&id).expect("unit").blocked_turns = turns;
            continue;
        }
        let (owner, at) = (u.owner, u.pos);
        let (col, row) = toward.to_offset();
        let blocker = state
            .units
            .values()
            .filter(|b| b.pos == toward && b.id != id)
            .min_by_key(|b| (b.owner == owner, b.id));
        let what = match blocker {
            Some(b) => {
                let p = state.player(b.owner);
                format!(
                    "blocked by {}'s {} at {col},{row}",
                    p.name,
                    data::unit_name_for(p.faction, b.kind)
                )
            }
            None => format!("blocked at {col},{row}"),
        };
        let u = state.units.get_mut(&id).expect("unit");
        u.goto = None;
        u.path.clear();
        u.path_stop_on_block = false;
        u.blocked_turns = 0;
        u.attack_target = None;
        u.needs_orders = true;
        events.push(Event::MoveBlocked {
            unit: id,
            owner,
            at,
            toward,
            reason: format!("{MOVE_STOPPED_PREFIX}{what}"),
        });
    }
}

/// Reason prefix of a MoveBlocked event for a tile held by a unit of a team at Peace with the
/// mover; the rest names it ("AI 5's Scout"). The client headlines it, never hides it.
pub const PEACE_BLOCK_PREFIX: &str = "at Peace: ";

/// MoveBlocked reason for a unit refused by its city's capture rule.
const RANGED_CAPTURE: &str = "Ranged units can't capture cities; move a melee unit in";

/// Why a step into a friend's tile failed, naming the friend ("P0's Spearman"); the client
/// says "your Spearman" to its owner. A friend that stays is crossed in a hop (`path::hop`),
/// so this reports a move that would end on one (the goto is its tile) and a friend that
/// failed to move out of a chain.
fn stayed_reason(state: &GameState, friend: UnitId, fought: bool) -> String {
    let b = &state.units[&friend];
    let p = state.player(b.owner);
    let why = if fought {
        "stayed to attack"
    } else {
        "did not move"
    };
    format!(
        "{FRIEND_BLOCK_PREFIX}{}'s {}, which {why}",
        p.name,
        data::unit_name_for(p.faction, b.kind)
    )
}

/// Reason prefix of a MoveBlocked event for a step into a friend's tile that the friend did
/// not leave; the rest names it ("P0's Spearman, which stayed to attack").
pub const FRIEND_BLOCK_PREFIX: &str = "blocked by ";

/// Reason prefix of the MoveBlocked event that stops a move (the client headlines it).
pub const MOVE_STOPPED_PREFIX: &str = "move stopped after 2 blocked turns: ";

/// Movers that will not leave their tile this round: a melee unit whose goto is an adjacent
/// hostile unit or city attacks from where it stands (it advances only if the defender dies,
/// which nobody can count on), and a civilian whose goto is an adjacent hostile tile is
/// blocked. Their tiles can be passed through but not ended on (`path::hop`), and a route
/// never lands on a hostile goal from beyond one (`path::crossing_faults`).
fn attacking_in_place(
    state: &GameState,
    occ: &BTreeMap<Hex, Occ>,
    movers: &BTreeSet<UnitId>,
) -> BTreeSet<UnitId> {
    movers
        .iter()
        .copied()
        .filter(|id| {
            let u = &state.units[id];
            let Some(goto) = u.goto.filter(|g| g.distance(u.pos) == 1) else {
                return false;
            };
            let Some(o) = occ.get(&goto) else {
                return false;
            };
            let hostile_unit = |v: &UnitId| hostile(state, u.owner, state.units[v].owner);
            let hostile_city = o
                .city
                .is_some_and(|c| hostile(state, u.owner, state.cities[&c].owner));
            hostile_city
                || o.military.iter().any(hostile_unit)
                || (!is_military(u.kind) && o.civilian.iter().any(hostile_unit))
        })
        .collect()
}

/// `occupancy` by tile index (None where empty): the same answers as the map, without a tree
/// lookup per route edge.
fn dense_occupancy(state: &GameState, occ: &BTreeMap<Hex, Occ>) -> Vec<Option<Occ>> {
    let mut dense = vec![None; state.map.len()];
    for (h, o) in occ {
        if let Some(i) = state.map.index(*h) {
            dense[i] = Some(*o);
        }
    }
    dense
}

/// Whether `h` holds a friend of `unit`'s class that stays put this round (parked, out of
/// moves, or attacking from where it stands): a tile `unit` may pass through but not end on
/// (`path::route_around_friends`, `path::hop`). A friend with a move out is not one: it is
/// followed as a chain.
fn stationary_friend(
    state: &GameState,
    occ: &[Option<Occ>],
    movers: &BTreeSet<UnitId>,
    staying: &BTreeSet<UnitId>,
    unit: &Unit,
    h: Hex,
) -> bool {
    let military = is_military(unit.kind);
    state
        .map
        .index(h)
        .and_then(|i| occ[i])
        .and_then(|o| if military { o.military } else { o.civilian })
        .is_some_and(|v| {
            let friend = &state.units[&v];
            friend.id != unit.id
                && (state.allied(unit.owner, friend.owner)
                    || !movers.contains(&v)
                    || staying.contains(&v))
                && (state.same_team(unit.owner, friend.owner)
                    || state.allied(unit.owner, friend.owner))
        })
}

/// The route `u` declares this round (its first tile is this round's step), going round
/// hostiles, passing through friends that stay put and preferring free tiles over friends
/// that should leave (`path::route_around_friends`).
#[allow(clippy::too_many_arguments)]
fn next_step(
    state: &mut GameState,
    occ: &[Option<Occ>],
    movers: &BTreeSet<UnitId>,
    staying: &BTreeSet<UnitId>,
    reserved: &BTreeMap<Hex, bool>,
    unit: UnitId,
    goto: Hex,
) -> Option<Vec<Hex>> {
    let u = &state.units[&unit];
    let military = is_military(u.kind);
    let occ_at = |h: Hex| state.map.index(h).and_then(|i| occ[i]);
    let same_class =
        |h: Hex| occ_at(h).and_then(|o| if military { o.military } else { o.civilian });
    // Tiles the route may not enter. Answers for the goal too (an attack cannot start from
    // beyond a friend); the searches and the confirmed step exempt the goal themselves.
    let avoid = |h: Hex| -> bool {
        // A lower-id friend's claim that will not clear this turn is still a wall for
        // this adjacent step. A pass-through claim is priced below as a round's wait.
        if u.pos.distance(h) == 1 && reserved.get(&h) == Some(&false) {
            return true;
        }
        let Some(o) = occ_at(h) else {
            return false;
        };
        let hostile_unit =
            |id: Option<UnitId>| id.is_some_and(|v| hostile(state, u.owner, state.units[&v].owner));
        let hostile_city = o
            .city
            .is_some_and(|c| hostile(state, u.owner, state.cities[&c].owner));
        let peaceful_city = o.city.is_some_and(|c| {
            let owner = state.cities[&c].owner;
            !state.same_team(u.owner, owner)
                && !state.allied(owner, u.owner)
                && !hostile(state, u.owner, owner)
        });
        let neutral = o.military.iter().chain(o.civilian.iter()).any(|id| {
            let owner = state.units[id].owner;
            !state.same_team(u.owner, owner)
                && !state.allied(u.owner, owner)
                && !hostile(state, u.owner, owner)
        });
        neutral
            || hostile_city
            || peaceful_city
            || hostile_unit(o.military)
            || hostile_unit(o.civilian)
    };
    // A friend that stays: crossed, never ended on (`path::hop`).
    let passable = |h: Hex| {
        stationary_friend(state, occ, movers, staying, u, h)
            || state
                .city_at(h)
                .is_some_and(|c| state.allied(c.owner, u.owner))
    };
    let open = |h| {
        state
            .map
            .get(h)
            .is_some_and(|t| state.passage_allowed(u.owner, u.kind, t.owner))
    };
    let exits = (!open(u.pos)).then(|| exits_from(state, open));
    let cost = |a, b| {
        state
            .map
            .move_cost(b)
            .filter(|_| path::border_step(&state.map, a, b, open, exits.as_deref()))
    };
    let max_moves = state.unit_moves_of(u);
    if u.path.last() == Some(&goto)
        && let Some(&step) = u.path.first()
        && u.pos.distance(step) == 1
        && cost(u.pos, step).is_some()
        && (step == goto || !avoid(step))
        // A confirmed step across friends needs a landing it can pay for; else it is repathed.
        && (step == goto
            || !passable(step)
            || path::crossing_faults(&u.path, u.pos, cost, avoid, passable, max_moves)
                .is_empty())
    {
        return Some(if step != goto && passable(step) {
            u.path.clone()
        } else {
            vec![step]
        });
    }
    if u.path_stop_on_block {
        return None;
    }
    // A same-team friend with a move out this round: followable, but its step may fail.
    let leaving_friend = |h: Hex| {
        same_class(h).is_some_and(|v| {
            v != unit
                && movers.contains(&v)
                && !staying.contains(&v)
                && state.same_team(u.owner, state.units[&v].owner)
        })
    };
    // A same-team friend out of moves for now (or stopped this turn) that still has a
    // destination: it stays this round but clears on a later turn.
    let waiting_friend = |h: Hex| {
        same_class(h).is_some_and(|v| {
            let friend = &state.units[&v];
            !movers.contains(&v) && friend.goto.is_some() && state.same_team(u.owner, friend.owner)
        })
    };
    let route = path::route_around_friends(
        &state.map,
        u.pos,
        goto,
        cost,
        avoid,
        passable,
        waiting_friend,
        leaving_friend,
        (u.moves_left, max_moves),
        |h| u.pos.distance(h) == 1 && reserved.get(&h) == Some(&true),
    );
    // A blocked confirmation becomes an ordinary goto, including across save/load.
    state.units.get_mut(&unit).expect("mover").path.clear();
    route
}

/// The relief of each city tile this round (user, 2026-10-04 and 2026-11-11): every non-ranged
/// military unit of the city's owner stepping in (a plain move) while an enemy attacks that
/// tile this round, in the order they take over as its garrison: the one that can soak the most
/// of the attackers' hits first (`battle::soak`: HP weighed by its defence against them, not HP
/// alone; designer, 2026-11-15), then the lowest unit id. Owner only: allies get no priority. A unit under attack where it
/// stands takes the normal path. Order-independent: intents are keyed by unit id.
fn reserves(
    state: &GameState,
    intents: &BTreeMap<UnitId, Intent>,
    occ: &BTreeMap<Hex, Occ>,
    held: &[battle::HeldShot],
) -> BTreeMap<Hex, Vec<UnitId>> {
    let mut out: BTreeMap<Hex, Vec<UnitId>> = BTreeMap::new();
    for i in intents.values() {
        let o = occ.get(&i.to).copied().unwrap_or_default();
        if i.step != Step::Move
            || !i.military
            || !data::attacks_in_melee(state.units[&i.unit].kind)
            || o.military.is_some_and(|g| state.units[&g].owner != i.owner)
            || o.civilian
                .is_some_and(|c| hostile(state, i.owner, state.units[&c].owner))
            || !o.city.is_some_and(|c| state.cities[&c].owner == i.owner)
        {
            continue;
        }
        // A shot held for this fallen city besieges it too: whoever steps in is its target.
        let besieged = intents.values().any(|j| {
            j.to == i.to && hostile(state, j.owner, i.owner) && matches!(j.step, Step::Attack(_))
        }) || held.iter().any(|h| {
            // The city a shot was held for can be gone by now (a Celtic capture razes it).
            state.cities.get(&h.city).is_some_and(|c| c.pos == i.to)
                && hostile(state, h.owner, i.owner)
        });
        let caught = intents
            .values()
            .any(|j| j.step == Step::Attack(Target::Unit(i.unit)));
        if besieged && !caught {
            out.entry(i.to).or_default().push(i.unit);
        }
    }
    for (tile, queue) in out.iter_mut() {
        let attackers: Vec<UnitId> = intents
            .values()
            .filter(|j| {
                j.to == *tile
                    && hostile(state, j.owner, state.units[&queue[0]].owner)
                    && matches!(j.step, Step::Attack(_))
            })
            .map(|j| j.unit)
            .collect();
        let terrain = state.map.get(*tile).map_or(Terrain::Plains, |t| t.terrain);
        queue.sort_by_key(|u| {
            (
                std::cmp::Reverse(battle::soak(state, *u, &attackers, terrain)),
                *u,
            )
        });
    }
    out
}

/// A relief unit steps into its own city and becomes its garrison: it pays its step and emits
/// `UnitMoved` then `CityReinforced`.
fn enter_own_city(state: &mut GameState, i: Intent, city: CityId, events: &mut Vec<Event>) {
    let u = state.units.get_mut(&i.unit).expect("relief is alive");
    u.pos = i.to;
    if let Some(k) = u.path.iter().position(|h| *h == i.to) {
        // A hop across friends lands several tiles along the confirmed route.
        u.path.drain(..=k);
    } else {
        u.path.clear();
        u.path_stop_on_block = false;
    }
    u.moves_left = (u.moves_left - i.cost).max(0);
    u.fortify = 0;
    u.acted = true;
    if u.goto == Some(i.to) {
        u.goto = None;
    }
    let unit_kind = u.kind;
    events.push(Event::UnitMoved {
        unit: i.unit,
        owner: i.owner,
        from: i.from,
        to: i.to,
    });
    events.push(Event::CityReinforced {
        unit: i.unit,
        owner: i.owner,
        unit_kind,
        city,
        at: i.to,
    });
}

/// MoveBlocked reason for an enemy claimant of a city whose relief survived the assault.
const CITY_REINFORCED: &str = "the city was reinforced";
/// MoveBlocked reason for a relief unit behind the one that holds its city.
const CITY_HELD: &str = "another of your units holds the city";

/// Rival assaults on one city (user, 2026-11-11, Lucerne; any rivals since 2026-11-14, Tara):
/// when military units of different permanent teams, at war with each other or not, all
/// claim a city tile whose defenders are gone, the side with the most force
/// (`battle::city_force`, summed over every unit of the side that moved onto the tile or
/// shot at it this turn; a side is one permanent team, so allies of other teams do not add up)
/// walks in, if it has a melee unit that can.
/// Sides without one count no less but cannot win. Returns the units of the winning side, or
/// the MoveBlocked reason for every claimant: for the losers, the winner and both forces. An
/// exact tie between the strongest sides is drawn by seed, turn and tile (Sam, 2026-11-22, rules
/// 51; it was a bounce for everyone), as on open ground (`settle_peaceful_contest`).
fn settle_by_force(
    state: &GameState,
    tile: Hex,
    force: &BTreeMap<UnitId, i32>,
    rivals: &[UnitId],
    pinned: &BTreeSet<UnitId>,
) -> (Vec<UnitId>, BTreeMap<UnitId, String>) {
    let owner = |u: &UnitId| state.units.get(u).map(|u| u.owner);
    let owners: BTreeSet<PlayerId> = force.keys().chain(rivals).filter_map(owner).collect();
    // Sides: players on one permanent team, merged to a fixpoint; keyed by the lowest id. Allies
    // and attackers at Peace with each other are separate sides.
    let mut side: BTreeMap<PlayerId, PlayerId> = owners.iter().map(|o| (*o, *o)).collect();
    loop {
        let mut changed = false;
        for a in &owners {
            for b in &owners {
                let m = side[a].min(side[b]);
                if state.same_team(*a, *b) && (side[a] != m || side[b] != m) {
                    side.insert(*a, m);
                    side.insert(*b, m);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut total: BTreeMap<PlayerId, i32> = BTreeMap::new();
    // Each side is named after the owner of its strongest unit (ties: lowest unit id).
    let mut strongest: BTreeMap<PlayerId, (i32, std::cmp::Reverse<UnitId>, PlayerId)> =
        BTreeMap::new();
    for (u, f) in force {
        let Some(o) = owner(u) else { continue };
        *total.entry(side[&o]).or_default() += f;
        let e = strongest
            .entry(side[&o])
            .or_insert((*f, std::cmp::Reverse(*u), o));
        if (*f, std::cmp::Reverse(*u)) > (e.0, e.1) {
            *e = (*f, std::cmp::Reverse(*u), o);
        }
    }
    let force_of = |s: &PlayerId| total.get(s).copied().unwrap_or(0);
    let name = |s: &PlayerId| {
        let o = strongest.get(s).map_or(*s, |e| e.2);
        state.player(o).name.clone()
    };
    let candidates: Vec<PlayerId> = rivals
        .iter()
        .filter(|r| !pinned.contains(r) && data::attacks_in_melee(state.units[*r].kind))
        .map(|r| side[&state.units[r].owner])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut reasons = BTreeMap::new();
    let Some(top) = candidates.iter().map(force_of).max() else {
        return (Vec::new(), reasons);
    };
    let tied: Vec<PlayerId> = candidates
        .iter()
        .copied()
        .filter(|s| force_of(s) == top)
        .collect();
    let best = crate::rng::seeded_order(
        state.dice_seed(),
        &[
            state.turn as u64,
            purpose::UNIT_CONTEST,
            tile.q as u64,
            tile.r as u64,
        ],
        &tied,
    )[0];
    let mut winners = Vec::new();
    for r in rivals {
        let mine = side[&state.units[r].owner];
        if mine == best {
            winners.push(*r);
        } else if tied.contains(&mine) {
            reasons.insert(
                *r,
                format!(
                    "{} brought equal force ({top} each) and won the draw for the city",
                    name(&best)
                ),
            );
        } else {
            reasons.insert(
                *r,
                format!(
                    "{} took the city with force {} against your {}",
                    name(&best),
                    force_of(&best),
                    force_of(&mine)
                ),
            );
        }
    }
    (winners, reasons)
}

/// Reason prefix of a MoveBlocked event for a unit that lost a peaceful contest for a tile
/// (`settle_peaceful_contest`); the rest names the winner and both strengths ("Sparta's
/// Hoplite is stronger (25 against 15) and moved in"). The client headlines it.
pub const OUTMATCHED_PREFIX: &str = "outmatched: ";

/// A unit's strength in a peaceful contest for a tile: its meeting strength (combat strength
/// after damage, with no terrain or fortification). Civilians have none.
fn contest_strength(state: &GameState, u: &Unit) -> i32 {
    if !is_military(u.kind) {
        return 0;
    }
    let off = state.setup.rule_experiments.faction_rules_off;
    let faction = state.player(u.owner).faction;
    combat::unit_meeting_strength_for_rules(
        off,
        faction,
        state.made_by(u),
        u.kind,
        u.hp,
        super::promotion::level_for_rules(off, faction, u.xp),
    )
    .total
}

/// Units of empires that are not at war stepping onto the same tile in the same round (Sam,
/// 2026-11-22, a ruin both a Spartan and a weaker unit went for): nobody attacks, and the side
/// with the most combined strength moves in while every other side stays where it is. This is
/// the open-ground twin of `settle_by_force` at a fallen city (user, 2026-11-11 and 2026-11-14:
/// "the one with the most force walks in"): a side is one permanent team, so co-op partners add
/// up, and its force is the sum of `contest_strength` over its claimants of the tile. Inside the
/// winning side the usual claim rules apply. A side needs an unpinned claimant to win; pinned
/// units still add their strength. Equal best forces are drawn by seed, turn and tile, so no
/// seat is favoured (rules 50's seeded ties). Applies only when no two claimants are at war (a
/// war meeting is fought as before) and the tile has no city. Returns the MoveBlocked reason
/// for each unit that stays out.
fn settle_peaceful_contest(
    state: &GameState,
    tile: Hex,
    units: &[UnitId],
    pinned: &BTreeSet<UnitId>,
) -> BTreeMap<UnitId, String> {
    let mut out = BTreeMap::new();
    let owner = |u: &UnitId| state.units[u].owner;
    let team = |u: &UnitId| state.team_of(owner(u));
    let teams: BTreeSet<TeamId> = units.iter().map(team).collect();
    if teams.len() < 2
        || units
            .iter()
            .any(|a| units.iter().any(|b| hostile(state, owner(a), owner(b))))
    {
        return out;
    }
    let mut force: BTreeMap<TeamId, i32> = BTreeMap::new();
    let mut members: BTreeMap<TeamId, Vec<UnitId>> = BTreeMap::new();
    for u in units {
        *force.entry(team(u)).or_default() += contest_strength(state, &state.units[u]);
        members.entry(team(u)).or_default().push(*u);
    }
    let candidates: Vec<TeamId> = members
        .iter()
        .filter(|(_, us)| us.iter().any(|u| !pinned.contains(u)))
        .map(|(t, _)| *t)
        .collect();
    let Some(top) = candidates.iter().map(|t| force[t]).max() else {
        return out;
    };
    let tied: Vec<TeamId> = candidates
        .iter()
        .copied()
        .filter(|t| force[t] == top)
        .collect();
    let drawn = tied.len() > 1;
    let side = crate::rng::seeded_order(
        state.dice_seed(),
        &[
            state.turn as u64,
            purpose::UNIT_CONTEST,
            tile.q as u64,
            tile.r as u64,
        ],
        &tied,
    )[0];
    // Name the winning side by its unit ("Sparta's Spartan Hoplite"), or by its strongest
    // unit's owner when several of its units add up ("Sparta's units").
    let winners = &members[&side];
    let lead = winners
        .iter()
        .max_by_key(|u| {
            (
                contest_strength(state, &state.units[*u]),
                std::cmp::Reverse(**u),
            )
        })
        .expect("a side has a claimant");
    let p = state.player(owner(lead));
    let (name, verb) = if winners.len() == 1 {
        (
            format!(
                "{}'s {}",
                p.name,
                data::unit_name_for(p.faction, state.units[lead].kind)
            ),
            "is",
        )
    } else {
        (format!("{}'s units", p.name), "are")
    };
    for u in units {
        if team(u) == side {
            continue;
        }
        let mine = force[&team(u)];
        let why = if mine < top {
            format!("{name} {verb} stronger ({top} against {mine}) and moved in")
        } else if drawn && candidates.contains(&team(u)) {
            format!("{name} {verb} just as strong ({top} each) and won the draw to move in")
        } else {
            // A side as strong as the winner whose every claimant was pinned by an attack.
            format!("it was attacked, and {name} moved in")
        };
        out.insert(*u, format!("{OUTMATCHED_PREFIX}{why}"));
    }
    out
}

/// A MoveBlocked reason worth reporting for a unit that also fought this round: why its
/// fight did not take the city.
fn reported_after_a_fight(reason: &str) -> bool {
    reason == RANGED_CAPTURE
        || reason == CITY_REINFORCED
        || reason.contains(" took the city with force ")
        || reason.ends_with("and won the draw for the city")
}

/// One round. Returns false when nobody had a step to declare.
fn movement_round(
    state: &mut GameState,
    round: u64,
    paths: &BTreeMap<UnitId, Vec<Hex>>,
    stopped: &mut BTreeSet<UnitId>,
    pursuit: &mut Pursuit,
    held: &mut Vec<battle::HeldShot>,
    events: &mut Vec<Event>,
) -> bool {
    let stage = 1 + round;
    // Occupation lock: a locked unit never leaves its city; its route is dropped.
    let locked: Vec<UnitId> = state
        .units
        .values()
        .filter(|u| u.goto.is_some() && super::occupation::lock(state, u).is_some())
        .map(|u| u.id)
        .collect();
    for id in locked {
        state.units.get_mut(&id).expect("unit").goto = None;
    }
    for u in state.units.values_mut() {
        if u.goto == Some(u.pos) {
            u.goto = None;
        }
        if u.goto.is_none() {
            u.path.clear();
            u.path_stop_on_block = false;
            u.attack_target = None;
        }
    }
    let movers: BTreeSet<UnitId> = state
        .units
        .values()
        .filter(|u| u.goto.is_some() && u.moves_left > 0 && !stopped.contains(&u.id))
        .map(|u| u.id)
        .collect();
    if movers.is_empty() {
        return false;
    }
    let mut occ = occupancy(state);
    let at = |h: Hex| occ.get(&h).copied().unwrap_or_default();
    let staying = attacking_in_place(state, &occ, &movers);
    let dense = dense_occupancy(state, &occ);

    // 1. Declare.
    let mut intents: BTreeMap<UnitId, Intent> = BTreeMap::new();
    let mut reservations: BTreeMap<(PlayerId, bool), BTreeMap<Hex, bool>> = BTreeMap::new();
    for &id in &movers {
        let u = &state.units[&id];
        let goto = u.goto.expect("mover has a goto");
        let reserved = reservations
            .entry((u.owner, is_military(u.kind)))
            .or_default();
        let route = if let Some(path) = paths.get(&id) {
            // Own traffic can wait for a round without moving. Advance the route cursor
            // by position, not round number, so waiting cannot skip an exploration step.
            let cursor = path.iter().position(|h| *h == u.pos).map_or(0, |i| i + 1);
            path.get(cursor)
                .copied()
                .filter(|h| u.pos.distance(*h) == 1 && state.map.move_cost(*h).is_some())
                .filter(|h| {
                    // Planning validates remembered borders; resolution must also honor
                    // borders revealed along the route, without probing them in validate.
                    let open = |at| {
                        state
                            .map
                            .get(at)
                            .is_some_and(|t| state.passage_allowed(u.owner, u.kind, t.owner))
                    };
                    let exits = (!open(u.pos)).then(|| exits_from(state, open));
                    path::border_step(&state.map, u.pos, *h, open, exits.as_deref())
                })
                .map(|h| vec![h])
        } else {
            next_step(state, &dense, &movers, &staying, reserved, id, goto)
        };
        let u = &state.units[&id];
        let Some(mut to) = route.as_ref().and_then(|r| r.first().copied()) else {
            events.push(Event::MoveBlocked {
                unit: id,
                owner: u.owner,
                at: u.pos,
                toward: u.path.first().copied().unwrap_or(goto),
                reason: if u.path_stop_on_block {
                    "confirmed route is blocked"
                } else {
                    "no route to the destination"
                }
                .into(),
            });
            let u = state.units.get_mut(&id).expect("unit");
            if u.path_stop_on_block {
                u.needs_orders = true;
            } else {
                u.goto = None;
            }
            stopped.insert(id);
            continue;
        };
        // A route that starts across friends that stay put hops over them in one move and
        // lands on the first tile beyond, paying every tile it crossed (Civ 6, user
        // 2026-10-21): a unit never ends a move on a friend. A hop it cannot pay for this
        // round waits for the next turn on the last free tile it passed; one that has no
        // landing (an enemy beyond, or the route ends on the friend) is the plain step
        // into the friend, which fails as it always did.
        let mut hop_cost = None;
        let max_moves = state.unit_moves_of(u);
        if to != goto
            && let Some(route) = route.as_deref().filter(|r| r.len() > 1)
            && let Some((landing, total)) = path::hop(
                route,
                u.pos,
                |_, h| state.map.move_cost(h),
                |h| {
                    stationary_friend(state, &dense, &movers, &staying, u, h)
                        || state
                            .city_at(h)
                            .is_some_and(|c| state.allied(c.owner, u.owner))
                },
            )
        {
            let o = at(landing);
            let foe = o
                .city
                .is_some_and(|c| hostile(state, u.owner, state.cities[&c].owner))
                || [o.military, o.civilian].iter().flatten().any(|v| {
                    !state.same_team(u.owner, state.units[v].owner)
                        && !state.allied(u.owner, state.units[v].owner)
                });
            if !foe && total <= u.moves_left {
                to = landing;
                hop_cost = Some(total);
            } else if !foe && total <= max_moves {
                stopped.insert(id);
                continue;
            }
        }
        if to == goto
            && let Some(city) = state.city_at(to).filter(|c| state.allied(c.owner, u.owner))
        {
            events.push(Event::MoveBlocked {
                unit: id,
                owner: u.owner,
                at: u.pos,
                toward: to,
                reason: format!(
                    "Allied units can pass through {} but can't stop there.",
                    city.name
                ),
            });
            let u = state.units.get_mut(&id).expect("unit");
            u.goto = None;
            u.path.clear();
            u.path_stop_on_block = false;
            u.needs_orders = true;
            stopped.insert(id);
            continue;
        }
        let cost =
            hop_cost.unwrap_or_else(|| state.map.move_cost(to).expect("path steps are passable"));
        if !can_step(u.moves_left, state.unit_moves_of(u), cost) {
            stopped.insert(id);
            continue;
        }
        // 2. Classify against the snapshot.
        let o = at(to);
        let military = is_military(u.kind);
        let hostile_unit = |v: &UnitId| hostile(state, u.owner, state.units[v].owner);
        let hostile_city = |c: &CityId| hostile(state, u.owner, state.cities[c].owner);
        let peaceful_city = o.city.is_some_and(|c| {
            let owner = state.cities[&c].owner;
            !state.same_team(u.owner, owner)
                && !state.allied(owner, u.owner)
                && !hostile(state, u.owner, owner)
        });
        let neutral_unit = o.military.iter().chain(o.civilian.iter()).find(|v| {
            let owner = state.units[v].owner;
            !state.same_team(u.owner, owner)
                && !state.allied(u.owner, owner)
                && !hostile(state, u.owner, owner)
        });
        // Barbarians never attack, shoot or capture anything of the Celts (`barbarians::spares`).
        let spared = u.owner.is_barbarian()
            && (o
                .military
                .iter()
                .chain(o.civilian.iter())
                .any(|v| super::barbarians::spares(state, state.units[v].owner))
                || o.city
                    .is_some_and(|c| super::barbarians::spares(state, state.cities[&c].owner)));
        let closed = !state.unit_in_open_land(u);
        let attacks_from_closed = closed
            && (o.military.iter().any(hostile_unit)
                || o.civilian.iter().any(hostile_unit)
                || o.city.iter().any(hostile_city));
        let enemy_on_route = to != goto
            && (o.military.iter().any(hostile_unit)
                || o.civilian.iter().any(hostile_unit)
                || o.city.iter().any(hostile_city));
        if neutral_unit.is_some()
            || peaceful_city
            || attacks_from_closed
            || enemy_on_route
            || spared
        {
            let reason = if spared {
                "barbarians don't attack the Celts".into()
            } else if let Some(b) = neutral_unit {
                // Name who is there: a move never attacks a unit of a team at Peace.
                let b = &state.units[b];
                let p = state.player(b.owner);
                format!(
                    "{PEACE_BLOCK_PREFIX}{}'s {}",
                    p.name,
                    data::unit_name_for(p.faction, b.kind)
                )
            } else if peaceful_city {
                "at Peace: foreign city occupied".into()
            } else if attacks_from_closed {
                "cannot attack from closed foreign territory".into()
            } else {
                "enemy in the way".into()
            };
            events.push(Event::MoveBlocked {
                unit: id,
                owner: u.owner,
                at: u.pos,
                toward: to,
                reason,
            });
            stopped.insert(id);
            continue;
        }
        // A pursuit attacks only the unit it was ordered to attack: if the target now stands
        // under another defender (an escort, its city), the pursuer stops beside it.
        if let Some(&(target, _)) = pursuit.chasing.get(&id)
            && to == goto
            && u.attack_target == Some(target)
        {
            // A city with HP shields its garrison, so the pursuer would only hit the city.
            let shielded = o.city.filter(hostile_city).is_some_and(|c| {
                combat::city_tile_defender(state.cities[&c].hp, true)
                    == combat::CityTileDefender::City
            });
            let hits_target = if o.military.is_some() {
                o.military == Some(target) && !shielded
            } else {
                o.civilian == Some(target) && !o.city.iter().any(hostile_city)
            };
            if !hits_target {
                stopped.insert(id);
                continue;
            }
        }
        let step = if !military {
            if o.military.iter().any(hostile_unit)
                || o.civilian.iter().any(hostile_unit)
                || o.city.iter().any(hostile_city)
            {
                events.push(Event::MoveBlocked {
                    unit: id,
                    owner: u.owner,
                    at: u.pos,
                    toward: to,
                    reason: "enemy in the way".into(),
                });
                stopped.insert(id);
                continue;
            }
            Step::Move
        } else if let Some(c) = o.city.filter(hostile_city) {
            // Civ 6: the city takes the attack while it has HP; its garrison only at 0 HP.
            let garrison = o.military.filter(hostile_unit);
            match (
                combat::city_tile_defender(state.cities[&c].hp, garrison.is_some()),
                garrison,
            ) {
                (combat::CityTileDefender::Garrison, Some(g)) => Step::Attack(Target::Unit(g)),
                _ => Step::Attack(Target::City(c)),
            }
        } else if let Some(v) = o.military.filter(hostile_unit) {
            Step::Attack(Target::Unit(v))
        } else {
            Step::Move
        };
        intents.insert(
            id,
            Intent {
                unit: id,
                owner: u.owner,
                military,
                from: u.pos,
                to,
                cost,
                step,
            },
        );
        let target = at(goto);
        let attacks_from_next = to.distance(goto) == 1
            && (target
                .city
                .is_some_and(|c| hostile(state, u.owner, state.cities[&c].owner))
                || target
                    .military
                    .is_some_and(|v| hostile(state, u.owner, state.units[&v].owner))
                || !military
                    && target
                        .civilian
                        .is_some_and(|v| hostile(state, u.owner, state.units[&v].owner)));
        reserved
            .entry(to)
            .or_insert(to != goto && u.moves_left > cost && !attacks_from_next);
    }
    // A barbarian never enters a tile a Celtic unit steps onto this round: that would be a
    // meeting the barbarians started, and they never fight the Celts.
    let celtic_steps: BTreeSet<Hex> = intents
        .values()
        .filter(|i| super::barbarians::spares(state, i.owner))
        .map(|i| i.to)
        .collect();
    let clashing: Vec<UnitId> = intents
        .values()
        .filter(|i| i.owner.is_barbarian() && celtic_steps.contains(&i.to))
        .map(|i| i.unit)
        .collect();
    for id in clashing {
        if let Some(i) = intents.remove(&id) {
            events.push(Event::MoveBlocked {
                unit: id,
                owner: i.owner,
                at: i.from,
                toward: i.to,
                reason: "barbarians don't attack the Celts".into(),
            });
            stopped.insert(id);
        }
    }
    // Swaps between hostile military units become one meeting.
    let swaps: Vec<(UnitId, UnitId)> = intents
        .values()
        .filter_map(|i| match i.step {
            Step::Attack(Target::Unit(v)) => intents
                .get(&v)
                .filter(|j| j.step == Step::Attack(Target::Unit(i.unit)))
                .map(|_| (i.unit, v)),
            _ => None,
        })
        .collect();
    for (a, b) in swaps {
        intents.get_mut(&a).expect("intent").step = Step::Meet(b);
    }

    // A distant melee attack does not pin a horse that declares a move in the same round.
    // The order stays aimed at the horse's turn-start tile and ends after the horse rides away.
    let escaping: Vec<UnitId> = intents
        .values()
        .filter_map(|i| match i.step {
            Step::Attack(Target::Unit(target))
                if pursuit.outrider.contains(&i.unit)
                    && intents.get(&target).is_some_and(|j| j.step == Step::Move) =>
            {
                Some(i.unit)
            }
            _ => None,
        })
        .collect();
    for id in escaping {
        intents.remove(&id);
        stopped.insert(id);
    }

    // Reinforcement goes first (user, 2026-10-04): a unit stepping into its owner's own empty
    // city, which an enemy is attacking this round, arrives before that attack and is the
    // garrison it meets. The owner's other units stepping in are its relief (user,
    // 2026-11-11): they wait outside the contest for the tile, take over in order as each
    // garrison falls or steps out, and the city falls only when all of them are dead.
    let mut relief: BTreeMap<Hex, Vec<Intent>> = BTreeMap::new();
    for (tile, queue) in reserves(state, &intents, &occ, held) {
        let mut queue: Vec<Intent> = queue
            .into_iter()
            .map(|id| intents.remove(&id).expect("reserve has an intent"))
            .collect();
        if occ[&tile].military.is_some() {
            relief.insert(tile, queue);
            continue;
        }
        let i = queue.remove(0);
        let id = i.unit;
        if !queue.is_empty() {
            relief.insert(tile, queue);
        }
        let city = occ[&tile].city.expect("reinforced tile is a city");
        enter_own_city(state, i, city, events);
        if let Some(o) = occ.get_mut(&i.from)
            && o.military == Some(id)
        {
            o.military = None;
        }
        occ.entry(tile).or_default().military = Some(id);
        // The city's HP still absorbs first; at 0 HP the attack now fights the garrison.
        if combat::city_tile_defender(state.cities[&city].hp, true)
            == combat::CityTileDefender::Garrison
        {
            for j in intents.values_mut() {
                if j.to == tile && matches!(j.step, Step::Attack(Target::City(_))) {
                    j.step = Step::Attack(Target::Unit(id));
                }
            }
        }
    }
    let at = |h: Hex| occ.get(&h).copied().unwrap_or_default();

    // Capture pins: civilians a hostile military unit steps onto stay and are captured. An
    // attacked unit is never pinned: it takes the hit and completes its step (G21).
    let mut pinned: BTreeSet<UnitId> = BTreeSet::new();
    for i in intents.values() {
        match i.step {
            Step::Move if i.military => {
                if let Some(c) = at(i.to)
                    .civilian
                    .filter(|c| hostile(state, i.owner, state.units[c].owner))
                {
                    pinned.insert(c);
                }
            }
            _ => {}
        }
    }

    // 3. Engagements, all planned from the snapshot.
    let mut claimants: BTreeMap<(Hex, bool), Vec<UnitId>> = BTreeMap::new();
    for i in intents.values() {
        claimants
            .entry((i.to, i.military))
            .or_default()
            .push(i.unit);
    }
    let mut fights: Vec<Fight> = Vec::new();
    let mut pairs: BTreeSet<(UnitId, UnitId)> = BTreeSet::new();
    // An ungarrisoned zero-HP city cannot fight. A defender-side entrant meets the attackers;
    // attackers arbitrate capture after combat (by force between rival sides) instead of
    // attacking ruins.
    let contested_ruins: BTreeSet<Hex> = claimants
        .iter()
        .filter(|((tile, military), _)| {
            *military
                && at(*tile).military.is_none()
                && at(*tile).city.is_some_and(|c| state.cities[&c].hp == 0)
        })
        .map(|((tile, _), _)| *tile)
        .collect();
    for i in intents.values() {
        match i.step {
            Step::Attack(Target::City(_)) if contested_ruins.contains(&i.to) => {}
            Step::Attack(t) => fights.push(battle::plan_fight_on(
                state,
                stage,
                i.unit,
                t,
                FightKind::Melee,
                // Caught moving away: a parting hit (G21).
                matches!(t, Target::Unit(v)
                    if intents.get(&v).is_some_and(|j| j.step == Step::Move)),
            )),
            Step::Meet(v) => {
                let pair = (i.unit.min(v), i.unit.max(v));
                if pairs.insert(pair) {
                    fights.push(battle::plan_fight(
                        state,
                        stage,
                        pair.0,
                        Target::Unit(pair.1),
                        FightKind::Meeting,
                    ));
                }
            }
            Step::Move => {}
        }
    }
    for ((tile, military), units) in &claimants {
        // A lone hostile civilian does not defend its tile. The military claimants
        // must settle their meeting before the survivor can capture it.
        if !*military
            || (at(*tile).military.is_some() || at(*tile).city.is_some())
                && !contested_ruins.contains(tile)
        {
            continue;
        }
        // Two rivals both assaulting the city do not fight each other: force decides.
        let city_owner = at(*tile).city.map(|c| state.cities[&c].owner);
        for (k, &a) in units.iter().enumerate() {
            for &b in &units[k + 1..] {
                let owners = (state.units[&a].owner, state.units[&b].owner);
                if city_owner
                    .is_some_and(|c| hostile(state, owners.0, c) && hostile(state, owners.1, c))
                {
                    continue;
                }
                if hostile(state, owners.0, owners.1)
                    && [a, b].iter().all(|id| {
                        let u = &state.units[id];
                        state.unit_in_open_land(u)
                    })
                    && pairs.insert((a.min(b), a.max(b)))
                {
                    let mut fight = battle::plan_fight(
                        state,
                        stage,
                        a.min(b),
                        Target::Unit(a.max(b)),
                        FightKind::Meeting,
                    );
                    fight.at = *tile;
                    fights.push(fight);
                }
            }
        }
    }
    // Held shots (Sam, 2026-11-21): once a defender stands on or steps into the fallen city
    // they were aimed at, they strike it now, after it slipped in and before this round's melee
    // (first in the stage, so supporters share them and the melee meets what is left).
    let mut volleys = Vec::new();
    let mut fired = Vec::new();
    held.retain(|h| {
        let Some(city) = state.cities.get(&h.city) else {
            return false;
        };
        let defender = at(city.pos).military.filter(|g| {
            state
                .units
                .get(g)
                .is_some_and(|g| hostile(state, h.owner, g.owner))
        });
        match defender {
            Some(g) if state.units.contains_key(&h.attacker) => {
                let mut shot = battle::plan_fight(
                    state,
                    stage,
                    h.attacker,
                    Target::Unit(g),
                    FightKind::Ranged,
                );
                shot.at = city.pos;
                volleys.push(shot);
                fired.push(*h);
                false
            }
            _ => state.units.contains_key(&h.attacker),
        }
    });
    if !volleys.is_empty() {
        volleys.append(&mut fights);
        fights = volleys;
    }
    for f in &fights {
        if let Target::Unit(d) = f.target {
            for (pursuer, prey) in [(f.attacker, d), (d, f.attacker)] {
                if pursuit.chasing.get(&pursuer).is_some_and(|c| c.0 == prey) {
                    pursuit.fought.insert(pursuer, f.at);
                }
            }
        }
    }
    let attackers: BTreeSet<UnitId> = fights
        .iter()
        .flat_map(|f| match (f.kind, f.target) {
            (FightKind::Meeting, Target::Unit(d)) => vec![f.attacker, d],
            _ => vec![f.attacker],
        })
        .collect();

    // Rival assaults: each claimant's and shooter's force, from the snapshot.
    let mut forces: BTreeMap<Hex, BTreeMap<UnitId, i32>> = BTreeMap::new();
    for ((tile, military), units) in &claimants {
        let Some(c) = at(*tile).city else { continue };
        let city_owner = state.cities[&c].owner;
        let owner = |u: &UnitId| state.units[u].owner;
        if !*military
            || !units.iter().all(|u| hostile(state, owner(u), city_owner))
            || units
                .iter()
                .all(|a| units.iter().all(|b| state.same_team(owner(a), owner(b))))
        {
            continue;
        }
        let mut force: BTreeMap<UnitId, i32> = units
            .iter()
            .map(|u| (*u, battle::city_force(state, *u, c, false)))
            .collect();
        for h in held.iter().chain(&fired).filter(|h| h.city == c) {
            if hostile(state, h.owner, city_owner) && !force.contains_key(&h.attacker) {
                force.insert(h.attacker, battle::city_force(state, h.attacker, c, true));
            }
        }
        for e in events.iter() {
            if let Event::Battle {
                attacker: crate::events::Attacker::Unit(a),
                attacker_owner,
                at: hit,
                ranged: true,
                ..
            } = e
                && hit == tile
                && state.units.contains_key(a)
                && hostile(state, *attacker_owner, city_owner)
                && !force.contains_key(a)
            {
                force.insert(*a, battle::city_force(state, *a, c, true));
            }
        }
        forces.insert(*tile, force);
    }

    // 4. Damage, all at once. A garrison stepping out of its city is not reached by overflow.
    let leaving: BTreeSet<UnitId> = intents
        .values()
        .filter(|i| i.step == Step::Move)
        .chain(relief.values().flatten())
        .map(|i| i.unit)
        .collect();
    // Supporters share what gets past the city's HP from where they stand (designer,
    // 2026-11-15); the healthiest one left enters as garrison afterwards if the garrison fell.
    let relief_cities: BTreeMap<CityId, Vec<UnitId>> = relief
        .iter()
        .map(|(tile, queue)| {
            (
                occ[tile].city.expect("relief steps into a city"),
                queue.iter().map(|i| i.unit).collect(),
            )
        })
        .collect();
    let chain = battle::carried_damage(state, &fights, &leaving, &relief_cities);
    let at = |h: Hex| occ.get(&h).copied().unwrap_or_default();
    battle::apply_movement_fights(state, &fights, &chain, events);
    // One attack per order: a fighter whose opponent still stands ends its order (G18).
    for f in &fights {
        let target_stands = match f.target {
            Target::Unit(d) => state.units.contains_key(&d),
            Target::City(c) => state.cities.get(&c).is_some_and(|c| c.hp > 0),
        };
        pursuit.engaged.extend(
            [
                Some((f.attacker, target_stands)),
                match (f.kind, f.target) {
                    (FightKind::Meeting, Target::Unit(d)) => {
                        Some((d, state.units.contains_key(&f.attacker)))
                    }
                    _ => None,
                },
            ]
            .into_iter()
            .flatten()
            .filter(|(u, other_stands)| *other_stands && state.units.contains_key(u))
            .map(|(u, _)| u),
        );
    }
    for id in &attackers {
        if let Some(u) = state.units.get_mut(id) {
            u.moves_left = 0;
            u.fortify = 0;
            stopped.insert(*id);
        }
    }

    // 5-6. Who advances.
    let alive = |s: &GameState, id: UnitId| s.units.contains_key(&id);
    let alive_claimants = |s: &GameState, to: Hex, military: bool| -> Vec<UnitId> {
        claimants
            .get(&(to, military))
            .map(|v| v.iter().copied().filter(|u| alive(s, *u)).collect())
            .unwrap_or_default()
    };
    let mut reasons: BTreeMap<UnitId, String> = BTreeMap::new();
    // Each supported city's new garrison: the supporter alive that can soak the most
    // (`battle::soak`, ties to the lowest id), unless the garrison stayed and lived; it enters once a surviving garrison steps
    // out. Supporters rejoin the advance pass, but never as claimants: they neither block nor
    // are blocked by the enemy's claim.
    let mut holders: BTreeMap<Hex, (UnitId, Option<UnitId>)> = BTreeMap::new();
    for (tile, queue) in &relief {
        let garrison = at(*tile).military.filter(|g| alive(state, *g));
        if garrison.is_some_and(|g| intents.get(&g).is_none_or(|j| j.step != Step::Move)) {
            continue;
        }
        let assault: Vec<UnitId> = fights
            .iter()
            .filter(|f| f.at == *tile && f.kind != FightKind::Ranged)
            .map(|f| f.attacker)
            .collect();
        let terrain = state.map.get(*tile).map_or(Terrain::Plains, |t| t.terrain);
        let strongest = queue
            .iter()
            .filter(|i| alive(state, i.unit))
            .max_by_key(|i| {
                (
                    battle::soak(state, i.unit, &assault, terrain),
                    std::cmp::Reverse(i.unit),
                )
            });
        if let Some(n) = strongest {
            holders.insert(*tile, (n.unit, garrison));
        }
    }
    let relief_units: BTreeSet<UnitId> = relief.values().flatten().map(|i| i.unit).collect();
    for i in relief.values().flatten() {
        if alive(state, i.unit) {
            intents.insert(i.unit, *i);
        }
    }
    // All teammates contesting a defeated enemy city select one captor. Sort by
    // approach tile so neither allocation order nor player seat can break an HP tie.
    let mut team_captors = BTreeMap::new();
    let mut decided: BTreeMap<UnitId, String> = BTreeMap::new();
    for (tile, military) in claimants.keys() {
        let mut rivals = alive_claimants(state, *tile, *military);
        if !*military
            || rivals.is_empty()
            || holders.contains_key(tile)
            || !at(*tile).city.is_some_and(|c| {
                state.cities[&c].hp == 0
                    && hostile(state, state.units[&rivals[0]].owner, state.cities[&c].owner)
            })
        {
            continue;
        }
        if rivals
            .iter()
            .any(|r| !state.same_team(state.units[&rivals[0]].owner, state.units[r].owner))
        {
            // Rival teams, at war with each other or not: the one with the most force walks in
            // (user, 2026-11-11, Lucerne; 2026-11-14, Tara).
            // A claimant on the city owner's side keeps the old contest.
            let Some(force) = forces.get(tile) else {
                continue;
            };
            let (winners, reasons) = settle_by_force(state, *tile, force, &rivals, &pinned);
            decided.extend(reasons);
            if winners.is_empty() {
                continue;
            }
            rivals = winners;
        }
        // If this very stage broke the city and killed its garrison, the strongest
        // surviving melee participant in the combined assault takes it.
        if at(*tile)
            .military
            .is_some_and(|g| !state.units.contains_key(&g))
        {
            let chosen = fights
                .iter()
                .filter(|f| {
                    f.kind == FightKind::Melee
                        && f.at == *tile
                        && matches!(f.target, Target::City(_))
                        && state.units.contains_key(&f.attacker)
                        && !pinned.contains(&f.attacker)
                        && rivals.contains(&f.attacker)
                        && data::attacks_in_melee(state.units[&f.attacker].kind)
                })
                .max_by_key(|f| (f.attacker_strength.total, std::cmp::Reverse(f.attacker)))
                .map(|f| f.attacker);
            if let Some(chosen) = chosen {
                team_captors.insert(*tile, chosen);
                continue;
            }
        }
        let mut eligible: Vec<_> = rivals
            .into_iter()
            .filter(|r| !pinned.contains(r) && data::attacks_in_melee(state.units[r].kind))
            .collect();
        let best_hp = eligible.iter().map(|r| state.units[r].hp).max();
        eligible.retain(|r| Some(state.units[r].hp) == best_hp);
        eligible.sort_by_key(|r| intents[r].from);
        if !eligible.is_empty() {
            let mut rng = Rng::derive(
                state.dice_seed(),
                &[
                    state.turn as u64,
                    purpose::CITY_CAPTURE,
                    tile.q as u64,
                    tile.r as u64,
                ],
            );
            let chosen = rng.range(0, eligible.len() as i64 - 1) as usize;
            team_captors.insert(*tile, eligible[chosen]);
        }
    }
    // Units of teams not at war contesting one tile: the strongest side moves in, the others
    // stay put; nobody attacks (Sam, 2026-11-22).
    let mut peaceful: BTreeMap<Hex, Vec<UnitId>> = BTreeMap::new();
    for ((tile, _), units) in &claimants {
        if at(*tile).city.is_some() || holders.contains_key(tile) {
            continue;
        }
        peaceful.entry(*tile).or_default().extend(
            units
                .iter()
                .copied()
                .filter(|u| alive(state, *u) && !decided.contains_key(u)),
        );
    }
    let mut outmatched: BTreeSet<UnitId> = BTreeSet::new();
    for (tile, units) in &peaceful {
        let lost = settle_peaceful_contest(state, *tile, units, &pinned);
        outmatched.extend(lost.keys().copied());
        decided.extend(lost);
    }
    let mut waiting = BTreeSet::new();
    let mut succeeded: BTreeSet<UnitId> = BTreeSet::new();
    let mut captures_city: BTreeMap<UnitId, CityId> = BTreeMap::new();
    for military in [true, false] {
        // Final military positions, known once the military pass is done.
        let military_final: BTreeMap<Hex, Vec<(UnitId, PlayerId)>> = if military {
            BTreeMap::new()
        } else {
            let mut m: BTreeMap<Hex, Vec<(UnitId, PlayerId)>> = BTreeMap::new();
            for u in state.units.values().filter(|u| is_military(u.kind)) {
                let pos = if succeeded.contains(&u.id) {
                    intents[&u.id].to
                } else {
                    u.pos
                };
                m.entry(pos).or_default().push((u.id, u.owner));
            }
            m
        };
        // Candidate -> the friendly unit it depends on leaving its target.
        let mut ok: BTreeMap<UnitId, Option<UnitId>> = BTreeMap::new();
        for i in intents.values().filter(|i| i.military == military) {
            if !alive(state, i.unit) {
                continue;
            }
            if pinned.contains(&i.unit) {
                reasons.insert(i.unit, "pinned by an enemy attack".into());
                continue;
            }
            if let Some(r) = decided.get(&i.unit) {
                reasons.insert(i.unit, r.clone());
                continue;
            }
            // Different classes may stack only within a permanent team. Neutral claimants
            // left after `settle_peaceful_contest` (a tile with a city, or a war among the
            // claimants) bounce even when the other claimant is a civilian; neither enters
            // combat.
            let neutral_rival = intents.values().any(|j| {
                j.to == i.to
                    && alive(state, j.unit)
                    && !decided.contains_key(&j.unit)
                    && !state.same_team(i.owner, j.owner)
                    && !hostile(state, i.owner, j.owner)
            });
            if neutral_rival {
                reasons.insert(i.unit, "bounced: peaceful teams contested the tile".into());
                continue;
            }
            if at(i.to)
                .city
                .is_some_and(|c| hostile(state, i.owner, state.cities[&c].owner))
                && !data::attacks_in_melee(state.units[&i.unit].kind)
            {
                reasons.insert(i.unit, RANGED_CAPTURE.into());
                continue;
            }
            if military && let Some(&(holder, dep)) = holders.get(&i.to) {
                if i.unit == holder {
                    ok.insert(i.unit, dep);
                    continue;
                }
                if relief_units.contains(&i.unit) {
                    reasons.insert(i.unit, CITY_HELD.into());
                    continue;
                }
                if at(i.to)
                    .city
                    .is_some_and(|c| hostile(state, i.owner, state.cities[&c].owner))
                {
                    reasons.insert(i.unit, CITY_REINFORCED.into());
                    continue;
                }
            } else if relief_units.contains(&i.unit) {
                reasons.insert(i.unit, CITY_HELD.into());
                continue;
            }
            let mut rivals = alive_claimants(state, i.to, military);
            rivals.retain(|r| !outmatched.contains(r));
            if rivals.len() > 1 {
                if team_captors.contains_key(&i.to) {
                    // Wait until dependencies resolve before choosing the city captor.
                } else if rivals.iter().all(|r| state.units[r].owner == i.owner) {
                    if rivals.iter().copied().find(|r| !pinned.contains(r)) != Some(i.unit) {
                        waiting.insert(i.unit);
                        reasons.insert(i.unit, "waiting for another of your units".into());
                        continue;
                    }
                } else {
                    let friendly = rivals
                        .iter()
                        .all(|r| !hostile(state, i.owner, state.units[r].owner));
                    reasons.insert(
                        i.unit,
                        if friendly {
                            "bounced: friendly units contested the tile"
                        } else {
                            "contested by an enemy"
                        }
                        .into(),
                    );
                    continue;
                }
            }
            let o = at(i.to);
            let holder = if military { o.military } else { o.civilian };
            let mut dep = None;
            if let Some(h) = holder.filter(|h| alive(state, *h)) {
                let departing_zero_hp_garrison = military
                    && o.city.is_some_and(|c| {
                        state.cities[&c].hp == 0 && hostile(state, i.owner, state.cities[&c].owner)
                    })
                    && data::attacks_in_melee(state.units[&i.unit].kind)
                    && intents.get(&h).is_some_and(|j| j.step == Step::Move);
                if hostile(state, i.owner, state.units[&h].owner) && !departing_zero_hp_garrison {
                    // A departing enemy garrison in a defeated city is a dependency;
                    // ordinary retreating enemies still hold their old tile this step.
                    continue;
                }
                dep = Some(h);
            }
            if let Some(c) = o
                .city
                .filter(|c| hostile(state, i.owner, state.cities[c].owner))
            {
                // Barbarians besiege cities but never take them (rules::barbarians).
                if military
                    && data::attacks_in_melee(state.units[&i.unit].kind)
                    && state.cities[&c].hp == 0
                    && !i.owner.is_barbarian()
                {
                    captures_city.insert(i.unit, c);
                } else {
                    continue;
                }
            }
            if !military
                && let Some(blockers) = military_final.get(&i.to)
                && blockers.iter().any(|(_, o)| !state.same_team(i.owner, *o))
            {
                reasons.insert(i.unit, "enemy in the way".into());
                continue;
            }
            ok.insert(i.unit, dep);
        }
        // Failures propagate along dependency links; cycles of candidates all succeed.
        loop {
            let failed: Vec<UnitId> = ok
                .iter()
                .filter(|(_, dep)| dep.is_some_and(|d| !ok.contains_key(&d)))
                .map(|(u, _)| *u)
                .collect();
            if failed.is_empty() {
                break;
            }
            for u in failed {
                let dep = ok.remove(&u).flatten().expect("failed on a dependency");
                reasons.insert(u, stayed_reason(state, dep, attackers.contains(&dep)));
            }
        }
        if military {
            // Select only among claimants whose path to the city actually clears.
            // If the preferred captor's dependency failed, another eligible teammate
            // takes the city in this same round.
            for (tile, preferred) in &team_captors {
                let mut eligible: Vec<_> = ok
                    .keys()
                    .copied()
                    .filter(|u| intents[u].to == *tile)
                    .collect();
                if eligible.is_empty() {
                    continue;
                }
                eligible.sort_by_key(|u| (std::cmp::Reverse(state.units[u].hp), intents[u].from));
                let chosen = if ok.contains_key(preferred) {
                    *preferred
                } else {
                    eligible[0]
                };
                for u in eligible.into_iter().filter(|u| *u != chosen) {
                    ok.remove(&u);
                    captures_city.remove(&u);
                    reasons.insert(u, "another teammate is capturing the city".into());
                }
            }
            // A follower of a claimant removed above must also stay behind.
            loop {
                let failed: Vec<_> = ok
                    .iter()
                    .filter(|(_, dep)| dep.is_some_and(|d| !ok.contains_key(&d)))
                    .map(|(u, _)| *u)
                    .collect();
                if failed.is_empty() {
                    break;
                }
                for u in failed {
                    let dep = ok.remove(&u).flatten().expect("failed on a dependency");
                    captures_city.remove(&u);
                    reasons.insert(u, stayed_reason(state, dep, attackers.contains(&dep)));
                }
            }
        }
        succeeded.extend(ok.keys().copied());
    }
    for c in captures_city.keys().copied().collect::<Vec<_>>() {
        if !succeeded.contains(&c) {
            captures_city.remove(&c);
        }
    }

    // 7. Apply: moves, captures, ruins, stops.
    for id in &succeeded {
        let i = intents[id];
        let u = state.units.get_mut(id).expect("advancing unit is alive");
        u.pos = i.to;
        if let Some(k) = u.path.iter().position(|h| *h == i.to) {
            // A hop across friends lands several tiles along the confirmed route.
            u.path.drain(..=k);
        } else {
            u.path.clear();
            u.path_stop_on_block = false;
        }
        u.moves_left = if attackers.contains(id) {
            0
        } else {
            (u.moves_left - i.cost).max(0)
        };
        u.fortify = 0;
        u.acted = true;
        if u.goto == Some(i.to) {
            u.goto = None;
        }
        events.push(Event::UnitMoved {
            unit: *id,
            owner: i.owner,
            from: i.from,
            to: i.to,
        });
        if relief_units.contains(id)
            && let Some(city) = at(i.to).city
        {
            events.push(Event::CityReinforced {
                unit: *id,
                owner: i.owner,
                unit_kind: u.kind,
                city,
                at: i.to,
            });
        }
    }
    for (unit, city) in &captures_city {
        let owner = state.units[unit].owner;
        battle::capture_city(state, *city, owner, events);
    }
    // Hostile civilians left under an arriving military unit are captured.
    for id in &succeeded {
        let i = intents[id];
        if !i.military {
            continue;
        }
        let captives: Vec<UnitId> = state
            .units
            .values()
            .filter(|c| c.pos == i.to && !is_military(c.kind) && hostile(state, i.owner, c.owner))
            .map(|c| c.id)
            .collect();
        if !captives.is_empty() {
            state.units.get_mut(id).expect("captor").moves_left = 0;
            stopped.insert(*id);
        }
        for c in captives {
            let cu = state.units.get_mut(&c).expect("captive");
            let from = cu.owner;
            cu.owner = i.owner;
            cu.moves_left = 0;
            cu.goto = None;
            cu.path.clear();
            cu.path_stop_on_block = false;
            cu.standing_order = None;
            cu.fortify = 0;
            stopped.insert(c);
            events.push(Event::SettlerCaptured {
                unit: c,
                from,
                to: i.owner,
                at: i.to,
            });
        }
    }
    for id in &succeeded {
        if state.units.contains_key(id) {
            explore_ruin(state, *id, events);
        }
    }
    let mut bounces: BTreeMap<Hex, Vec<UnitId>> = BTreeMap::new();
    for i in intents.values() {
        if succeeded.contains(&i.unit) {
            continue;
        }
        if waiting.contains(&i.unit)
            && succeeded.iter().any(|id| intents[id].owner == i.owner)
            && !attackers.contains(&i.unit)
        {
            continue;
        }
        stopped.insert(i.unit);
        if !state.units.contains_key(&i.unit)
            || (attackers.contains(&i.unit)
                && !reasons
                    .get(&i.unit)
                    .is_some_and(|r| reported_after_a_fight(r)))
        {
            continue;
        }
        let u = state.units.get_mut(&i.unit).expect("blocked unit is alive");
        if u.path_stop_on_block {
            u.needs_orders = true;
        }
        match reasons.get(&i.unit) {
            Some(r) if r.starts_with("bounced") => bounces.entry(i.to).or_default().push(i.unit),
            r => events.push(Event::MoveBlocked {
                unit: i.unit,
                owner: i.owner,
                at: i.from,
                toward: i.to,
                reason: r.cloned().unwrap_or_else(|| "tile occupied".into()),
            }),
        }
    }
    for (tile, units) in bounces {
        events.push(Event::Bounce { units, tile });
    }
    visibility::update_explored(state);
    true
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::combat;
    use crate::orders::Order;
    use crate::types::Terrain;

    fn pos(s: &GameState, u: UnitId) -> Hex {
        s.units[&u].pos
    }

    fn battles(events: &[Event]) -> Vec<Event> {
        events
            .iter()
            .filter(|e| matches!(e, Event::Battle { .. }))
            .cloned()
            .collect()
    }

    /// Battles a unit started (city strike shots excluded).
    fn unit_battles(events: &[Event]) -> Vec<Event> {
        battles(events)
            .into_iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::Battle {
                        attacker: crate::events::Attacker::Unit(_),
                        ..
                    }
                )
            })
            .collect()
    }

    #[test]
    fn always_one_step_rule() {
        assert!(can_step(2, 2, 2));
        assert!(can_step(2, 2, 5), "full moves: any single step");
        assert!(!can_step(1, 2, 2));
        assert!(can_step(1, 3, 1));
        assert!(!can_step(0, 2, 1));
    }

    #[test]
    fn outrider_allows_a_distant_spear_to_hit_a_stationary_horse() {
        let mut s = game(&[0, 1]);
        let spear = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let horse = unit(&mut s, 1, UnitKind::Horseman, hex(5, 4));
        let order = Order::MoveUnit {
            unit: spear,
            to: hex(5, 4),
        };
        let errors = crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&order));
        assert!(errors.is_empty());
        let (_, events) = resolve_units(&s, &orders(0, vec![order]));
        assert!(!unit_battles(&events).is_empty());

        s.units.get_mut(&spear).unwrap().pos = hex(4, 4);
        let adjacent = Order::MoveUnit {
            unit: spear,
            to: hex(5, 4),
        };
        assert!(
            crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&adjacent)).is_empty()
        );
        let (_, events) = resolve_units(&s, &orders(0, vec![adjacent]));
        assert!(!unit_battles(&events).is_empty());

        s.units.get_mut(&spear).unwrap().pos = hex(3, 4);
        s.units.get_mut(&horse).unwrap().kind = UnitKind::Tank;
        assert!(
            crate::validate_orders(
                &s,
                PlayerId(0),
                &[Order::MoveUnit {
                    unit: spear,
                    to: hex(5, 4)
                }]
            )
            .is_empty()
        );
    }

    #[test]
    fn mongols_route_around_a_foreign_spearman() {
        let mut s = game(&[0, 1]);
        s.players[0].faction = Faction::Mongols;
        let courier = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let guard = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, courier, hex(6, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, guard), hex(4, 4));
        assert_ne!(pos(&next, courier), hex(4, 4));
        assert_ne!(pos(&next, courier), hex(3, 4));
        assert!(unit_battles(&events).is_empty());
    }

    #[test]
    fn movement_costs_follow_terrain() {
        let mut s = game(&[0, 1]);
        // Spearman (2 moves): grass (1) then forest (2) is not affordable with 1 left.
        let u = unit(&mut s, 0, UnitKind::Spearman, hex(2, 2));
        set_terrain(&mut s, hex(4, 2), Terrain::Forest);
        goto(&mut s, u, hex(5, 2));
        // Force the route through the forest by walling the rows around it.
        for c in 2..7 {
            for r in [1, 3] {
                set_terrain(&mut s, hex(c, r), Terrain::Mountains);
            }
        }
        let (s1, _) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&s1, u), hex(3, 2));
        assert_eq!(s1.units[&u].moves_left, 1);
        let (s2, _) = resolve_units(&s1, &no_orders());
        assert_eq!(pos(&s2, u), hex(4, 2), "forest costs all 2 moves");
        assert_eq!(s2.units[&u].moves_left, 0);
        let (s3, _) = resolve_units(&s2, &no_orders());
        assert_eq!(pos(&s3, u), hex(5, 2));
        assert_eq!(s3.units[&u].goto, None, "arrived: goto cleared");
    }

    #[test]
    fn multi_turn_goto() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Spearman, hex(1, 4));
        let mut st = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::MoveUnit {
                    unit: u,
                    to: hex(6, 4),
                }],
            ),
        )
        .0;
        assert_eq!(st.units[&u].pos.distance(hex(1, 4)), 2);
        st = resolve_units(&st, &no_orders()).0;
        assert_eq!(st.units[&u].pos.distance(hex(1, 4)), 4);
        st = resolve_units(&st, &no_orders()).0;
        assert_eq!(pos(&st, u), hex(6, 4));
        assert_eq!(st.units[&u].goto, None);
        s = st;
        let explored = s.player(PlayerId(0)).explored.count();
        assert!(explored > 0, "tiles are revealed while moving");
    }

    #[test]
    fn friendly_units_contesting_a_tile_bounce() {
        let mut s = game(&[0, 0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, b, hex(4, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, a), hex(3, 4));
        assert_eq!(pos(&next, b), hex(5, 4));
        assert_eq!(next.units[&a].goto, Some(hex(4, 4)), "goto kept");
        assert!(events.contains(&Event::Bounce {
            units: vec![a, b],
            tile: hex(4, 4)
        }));
        assert!(battles(&events).is_empty(), "teammates never fight");
    }

    #[test]
    fn military_and_civilian_share_a_tile() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 0, UnitKind::Settler, hex(5, 4));
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, b, hex(4, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, a), hex(4, 4));
        assert_eq!(pos(&next, b), hex(4, 4));
    }

    #[test]
    fn friendly_chain_moves_together_and_blocked_link_blocks_it() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(2, 4));
        let b = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        goto(&mut s, a, hex(3, 4));
        goto(&mut s, b, hex(4, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, a), hex(3, 4));
        assert_eq!(pos(&next, b), hex(4, 4));

        // A parked unit at the head blocks the whole chain.
        let c = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, a), hex(2, 4));
        assert_eq!(pos(&next, b), hex(3, 4));
        assert_eq!(pos(&next, c), hex(4, 4));
        let blocked = events
            .iter()
            .filter(|e| matches!(e, Event::MoveBlocked { .. }))
            .count();
        assert_eq!(blocked, 2);
    }

    #[test]
    fn friendly_cycles_rotate() {
        let mut s = game(&[0, 1]);
        // Three mutually adjacent hexes.
        let h0 = hex(4, 4);
        let h1 = h0.neighbor(0);
        let h2 = h0.neighbor(1);
        let a = unit(&mut s, 0, UnitKind::Spearman, h0);
        let b = unit(&mut s, 0, UnitKind::Spearman, h1);
        let c = unit(&mut s, 0, UnitKind::Spearman, h2);
        goto(&mut s, a, h1);
        goto(&mut s, b, h2);
        goto(&mut s, c, h0);
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!((pos(&next, a), pos(&next, b), pos(&next, c)), (h1, h2, h0));

        // A friendly swap is a two-cycle.
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, h0);
        let b = unit(&mut s, 0, UnitKind::Spearman, h1);
        goto(&mut s, a, h1);
        goto(&mut s, b, h0);
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!((pos(&next, a), pos(&next, b)), (h1, h0));
    }

    #[test]
    fn hostile_units_meeting_on_an_empty_tile_fight_without_bonuses() {
        let mut s = game(&[0, 1]);
        set_terrain(&mut s, hex(4, 4), Terrain::Hills);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        s.units.get_mut(&b).unwrap().fortify = 6;
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, b, hex(4, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        let bs = battles(&events);
        assert_eq!(bs.len(), 1);
        let Event::Battle {
            meeting,
            attacker_strength,
            defender_strength,
            damage_to_attacker,
            damage_to_defender,
            attacker_roll,
            defender_roll,
            ..
        } = &bs[0]
        else {
            unreachable!()
        };
        assert!(*meeting);
        assert_eq!((attacker_strength.total, defender_strength.total), (20, 20));
        assert_eq!(*damage_to_defender, combat::damage(20, 20, *attacker_roll));
        assert_eq!(*damage_to_attacker, combat::damage(20, 20, *defender_roll));
        // Both survive: nobody advances; fought units lose fortification.
        assert_eq!(pos(&next, a), hex(3, 4));
        assert_eq!(pos(&next, b), hex(5, 4));
        assert_eq!(next.units[&b].fortify, 0);
        assert_eq!(next.units[&a].hp, 100 - damage_to_attacker);
    }

    #[test]
    fn hostile_claimants_meet_over_a_third_partys_settler() {
        for rival in [1, 255] {
            let mut s = game(&[0, 1, 2]);
            let target = hex(5, 4);
            let settler = unit(&mut s, 2, UnitKind::Settler, target);
            let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
            let b = unit(&mut s, rival, UnitKind::Spearman, hex(6, 4));
            s.units.get_mut(&b).unwrap().hp = 1;
            goto(&mut s, a, target);
            goto(&mut s, b, target);
            let (next, events) = resolve_units(&s, &no_orders());
            assert_eq!(unit_battles(&events).len(), 1, "rival {rival}");
            assert!(matches!(
                unit_battles(&events)[0],
                Event::Battle { meeting: true, .. }
            ));
            assert_eq!(pos(&next, a), target);
            assert_eq!(next.units[&settler].owner, PlayerId(0));
            assert_eq!(
                events
                    .iter()
                    .filter(
                        |e| matches!(e, Event::SettlerCaptured { unit, .. } if *unit == settler)
                    )
                    .count(),
                1
            );
        }
    }

    #[test]
    fn zero_hp_city_is_captured_when_garrison_leaves_three_claimants() {
        for (garrison_hp, mongol_garrison) in [(28, false), (100, false), (28, true), (100, true)] {
            let mut s = game(&[0, 1]);
            if mongol_garrison {
                // The designer's 8.9.21 game, turn 44: Shangdu at 0 HP, its Mongol garrison walking out.
                s.player_mut(PlayerId(1)).faction = Faction::Mongols;
            }
            let target = hex(5, 4);
            let c = city(&mut s, 1, target, 3);
            s.cities.get_mut(&c).unwrap().hp = 0;
            let g = unit(&mut s, 1, UnitKind::Spearman, target);
            s.units.get_mut(&g).unwrap().hp = garrison_hp;
            goto(&mut s, g, hex(6, 4));
            let claimants: Vec<_> = [2, 3, 4]
                .map(|dir| {
                    let u = unit(&mut s, 0, UnitKind::Spearman, target.neighbor(dir));
                    goto(&mut s, u, target);
                    u
                })
                .to_vec();
            let (next, events) = resolve_units(&s, &no_orders());
            assert_eq!(
                next.cities[&c].owner,
                PlayerId(0),
                "garrison HP {garrison_hp}, Mongols {mongol_garrison}: {events:?}"
            );
            assert_eq!(next.units.contains_key(&g), garrison_hp == 100);
            if garrison_hp == 100 {
                assert_eq!(pos(&next, g), hex(6, 4));
            }
            assert_eq!(
                claimants
                    .iter()
                    .filter(|u| pos(&next, **u) == target)
                    .count(),
                1
            );
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::CityCaptured { city, .. } if *city == c))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn blocked_reserve_does_not_fight_through_an_occupied_tile() {
        let mut s = game(&[0, 1]);
        let target = hex(4, 4);
        let garrison = unit(&mut s, 0, UnitKind::Spearman, target);
        let reserve = unit(&mut s, 0, UnitKind::Spearman, hex(5, 4));
        let attacker = unit(&mut s, 1, UnitKind::Spearman, hex(3, 4));
        goto(&mut s, reserve, target);
        goto(&mut s, attacker, target);

        let (next, events) = resolve_units(&s, &no_orders());
        let fights = battles(&events);
        assert_eq!(
            fights.len(),
            1,
            "only the garrison defends the occupied tile"
        );
        assert!(matches!(fights[0], Event::Battle { meeting: false, .. }));
        // "From where?": the melee attacker struck from its own tile, next to the target.
        assert!(
            matches!(fights[0], Event::Battle { at, attacker_at: Some(from), .. }
            if at == target && from == hex(3, 4))
        );
        assert!(next.units[&garrison].hp < 100);
        assert_eq!(next.units[&reserve].hp, 100);
        assert_eq!(pos(&next, reserve), hex(5, 4));
    }

    #[test]
    fn hostile_swap_is_one_meeting_and_survivor_advances() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&b).unwrap().hp = 10;
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, b, hex(3, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        let bs = battles(&events);
        assert_eq!(bs.len(), 1);
        assert!(matches!(bs[0], Event::Battle { meeting: true, .. }));
        assert!(!next.units.contains_key(&b));
        assert_eq!(pos(&next, a), hex(4, 4), "unique survivor advances");
        assert_eq!(next.units[&a].moves_left, 0, "attacking ends movement");
    }

    #[test]
    fn g21_a_unit_attacked_while_leaving_takes_a_parting_hit_then_completes_its_step() {
        let mut s = game(&[0, 1]);
        set_terrain(&mut s, hex(4, 4), Terrain::Hills);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&b).unwrap().fortify = 6;
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, b, hex(6, 4));
        // b's route east is free; a attacks b's tile as b steps away.
        let (next, events) = resolve_units(&s, &no_orders());
        let bs = battles(&events);
        assert_eq!(bs.len(), 1);
        let Event::Battle {
            meeting,
            defender_strength,
            attacker_roll,
            damage_to_defender,
            defender_leaving,
            at,
            ..
        } = &bs[0]
        else {
            unreachable!()
        };
        assert!(!meeting && *defender_leaving);
        assert_eq!(*at, hex(4, 4), "hit where it stood");
        // Caught leaving: terrain (+5 hills) still counts, fortification (+6) does not.
        assert_eq!(defender_strength.total, 25);
        assert_eq!(*damage_to_defender, combat::damage(20, 25, *attacker_roll));
        assert_ne!(pos(&next, b), hex(4, 4), "not pinned: it got away");
        assert!(pos(&next, b).distance(hex(6, 4)) < hex(4, 4).distance(hex(6, 4)));
        assert_eq!(pos(&next, a), hex(3, 4), "the attacker stays put");
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::MoveBlocked { unit, .. } if *unit == b
        )));
        assert_eq!(next.units[&a].goto, None, "one attack per order (G18)");
    }

    #[test]
    fn g21_a_parting_hit_that_kills_lets_the_attacker_overrun_the_tile() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Archer, hex(4, 4));
        s.units.get_mut(&b).unwrap().hp = 5;
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, b, hex(6, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert!(!next.units.contains_key(&b));
        assert_eq!(pos(&next, a), hex(4, 4), "overran the vacated tile");
    }

    #[test]
    fn g21_a_stationary_defender_keeps_fortification() {
        let mut s = game(&[0, 1]);
        set_terrain(&mut s, hex(4, 4), Terrain::Hills);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&b).unwrap().fortify = 6;
        goto(&mut s, a, hex(4, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        let Event::Battle {
            defender_strength,
            defender_leaving,
            ..
        } = &battles(&events)[0]
        else {
            unreachable!()
        };
        assert!(!defender_leaving);
        assert_eq!(defender_strength.total, 31, "terrain +5 and fortify +6");
        assert_eq!(pos(&next, b), hex(4, 4));
        assert_eq!(
            next.units[&b].fortify, 6,
            "defending does not clear fortify"
        );
    }

    #[test]
    fn three_way_contest_every_hostile_pair_fights_once() {
        let mut s = game(&[0, 1, 2]);
        let t = hex(5, 5);
        let a = unit(&mut s, 0, UnitKind::Spearman, t.neighbor(0));
        let b = unit(&mut s, 1, UnitKind::Spearman, t.neighbor(2));
        let c = unit(&mut s, 2, UnitKind::Spearman, t.neighbor(4));
        for u in [a, b, c] {
            goto(&mut s, u, t);
        }
        let (next, events) = resolve_units(&s, &no_orders());
        let bs = battles(&events);
        assert_eq!(bs.len(), 3);
        assert!(
            bs.iter()
                .all(|e| matches!(e, Event::Battle { meeting: true, .. }))
        );
        for u in [a, b, c] {
            assert_ne!(pos(&next, u), t, "several survivors: nobody advances");
        }
    }

    #[test]
    fn attack_kills_defender_and_attacker_advances() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Scout, hex(4, 4));
        s.units.get_mut(&b).unwrap().hp = 5;
        goto(&mut s, a, hex(4, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(!next.units.contains_key(&b));
        assert_eq!(pos(&next, a), hex(4, 4));
        assert_eq!(next.units[&a].moves_left, 0, "attacking ends movement");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::UnitDestroyed { unit, .. } if *unit == b))
        );
    }

    #[test]
    fn simultaneous_mutual_kill() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let b = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&a).unwrap().hp = 1;
        s.units.get_mut(&b).unwrap().hp = 1;
        goto(&mut s, a, hex(4, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert!(!next.units.contains_key(&a) && !next.units.contains_key(&b));
    }

    fn city_defender(e: &Event) -> Option<CityId> {
        match e {
            Event::Battle {
                defender: crate::events::Defender::City { city, .. },
                ..
            } => Some(*city),
            _ => None,
        }
    }

    #[test]
    fn a_city_with_hp_takes_the_melee_then_overflows_into_its_garrison() {
        // The city defends the hit, then excess damage reaches the garrison. It still holds
        // if the overflow does not kill it.
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        s.units.get_mut(&g).unwrap().fortify = 6;
        s.cities.get_mut(&c).unwrap().hp = 5;
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, a, hex(5, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        // An unwalled city does not fire (city strike needs Walls): only the melee happens.
        let bs = unit_battles(&events);
        assert_eq!(battles(&events).len(), 1);
        assert_eq!(city_defender(&bs[0]), Some(c));
        let Event::Battle {
            defender_strength,
            damage_to_attacker,
            ..
        } = &bs[0]
        else {
            unreachable!()
        };
        assert_eq!(
            defender_strength.total, 7,
            "city 10 + 2 x 3 population - 9 wounded; the garrison adds nothing"
        );
        assert!(*damage_to_attacker > 0, "the city retaliates");
        assert!(next.units[&g].hp > 0 && next.units[&g].hp < 100);
        assert_eq!(next.units[&g].fortify, 6, "the garrison did not fight");
        assert_eq!(next.cities[&c].hp, 0);
        assert_eq!(
            next.cities[&c].owner,
            PlayerId(1),
            "the garrison still holds"
        );
        assert_eq!(pos(&next, a), hex(4, 4));
    }

    #[test]
    fn a_fallen_city_s_garrison_fights_and_its_defeat_captures_the_city() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        s.cities.get_mut(&c).unwrap().hp = 0;
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        let civ = unit(&mut s, 1, UnitKind::Settler, hex(5, 4));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, a, hex(5, 4));
        // A healthy garrison survives one attack: no capture.
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(matches!(&unit_battles(&events)[0],
            Event::Battle { defender: crate::events::Defender::Unit { unit, .. }, .. } if *unit == g));
        assert!(next.units[&g].hp < 100);
        assert_eq!(next.cities[&c].owner, PlayerId(1));
        assert_eq!(pos(&next, a), hex(4, 4));
        // A garrison the attack destroys: the attacker enters, captures the city and the
        // civilian, and the Swiss bounty is paid as for any capture.
        s.units.get_mut(&g).unwrap().hp = 1;
        s.players[1].gold = 90;
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(!next.units.contains_key(&g));
        assert_eq!(next.cities[&c].owner, PlayerId(0));
        assert_eq!(pos(&next, a), hex(5, 4));
        assert_eq!(next.units[&civ].owner, PlayerId(0));
        assert_eq!(
            next.player(PlayerId(0)).gold,
            90,
            "the sole city's whole treasury"
        );
        assert!(events.iter().any(|e| matches!(
            e,
            Event::CityCaptured {
                bounty: Some(90),
                ..
            }
        )));
        assert_eq!(
            next.cities[&c].resistance,
            data::CAPTURE_RESISTANCE_TURNS,
            "resistance as for any capture"
        );
        // A Celts attacker razes it instead.
        s.players[0].faction = Faction::Celts;
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(!next.cities.contains_key(&c));
        assert!(events.iter().any(|e| matches!(e, Event::CityRazed { .. })));
    }

    #[test]
    fn ranged_fire_breaks_the_city_and_kills_garrison_then_melee_captures() {
        // Phase order: ranged overflow kills the garrison, then the first movement round's
        // melee attacker enters the emptied city.
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        s.cities.get_mut(&c).unwrap().hp = 10;
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        s.units.get_mut(&g).unwrap().hp = 1;
        let r = unit(&mut s, 0, UnitKind::Archer, hex(3, 4));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, a, hex(5, 4));
        let os = orders(
            0,
            vec![Order::RangedAttack {
                unit: r,
                target: hex(5, 4),
            }],
        );
        let (next, events) = resolve_units(&s, &os);
        let bs = unit_battles(&events);
        assert_eq!(city_defender(&bs[0]), Some(c), "the shot hits the city");
        assert_eq!(bs.len(), 1);
        assert!(!next.units.contains_key(&g));
        assert_eq!(next.cities[&c].owner, PlayerId(0));
        assert_eq!(pos(&next, a), hex(5, 4));
    }

    #[test]
    fn a_garrison_leaving_a_city_with_hp_is_not_hit_on_the_way_out() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, g, hex(6, 4));
        goto(&mut s, a, hex(5, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(city_defender(&unit_battles(&events)[0]), Some(c));
        assert_eq!(next.units[&g].hp, 100);
        assert_eq!(pos(&next, g), hex(6, 4));
        assert!(next.cities[&c].hp < data::CITY_HP);
        // At 0 HP the leaving garrison is the defender instead and takes a parting hit (G21).
        s.cities.get_mut(&c).unwrap().hp = 0;
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(matches!(&unit_battles(&events)[0],
            Event::Battle { defender_leaving: true, defender: crate::events::Defender::Unit { unit, .. }, .. } if *unit == g));
        assert!(next.units[&g].hp < 100);
    }

    #[test]
    fn a_garrison_leaving_a_breaking_city_takes_no_overflow() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        s.cities.get_mut(&c).unwrap().hp = 1;
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, g, hex(6, 4));
        goto(&mut s, a, hex(5, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!(next.cities[&c].hp, 0);
        assert_eq!(
            next.units[&g].hp, 100,
            "the garrison stepped out before overflow"
        );
        assert_eq!(pos(&next, g), hex(6, 4));
        // Staying put, the same hit overflows into it.
        s.units.get_mut(&g).unwrap().goto = None;
        let (next, _) = resolve_units(&s, &no_orders());
        assert!(next.units[&g].hp < 100);
    }

    #[test]
    fn a_pursuer_stops_beside_a_city_that_shields_its_quarry() {
        let mut s = game(&[0, 1]);
        city(&mut s, 1, hex(5, 4), 3);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, a, hex(5, 4));
        s.units.get_mut(&a).unwrap().attack_target = Some(g);
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(unit_battles(&events).is_empty());
        assert_eq!(next.units[&g].hp, 100);
        assert_eq!(pos(&next, a), hex(4, 4));
    }

    #[test]
    fn a_barbarian_at_an_empty_zero_hp_city_besieges_it_and_never_takes_it() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        s.cities.get_mut(&c).unwrap().hp = 0;
        for kind in [UnitKind::Spearman, UnitKind::Pikeman] {
            let mut s = s.clone();
            let b = unit(&mut s, PlayerId::BARBARIAN.0, kind, hex(4, 4));
            goto(&mut s, b, hex(5, 4));
            let (next, events) = resolve_units(&s, &no_orders());
            assert_eq!(next.cities[&c].owner, PlayerId(1));
            assert_ne!(pos(&next, b), hex(5, 4));
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, Event::CityCaptured { .. }))
            );
        }
        // The capture itself refuses a barbarian captor rather than panicking on its missing
        // player record.
        let mut events = Vec::new();
        battle::capture_city(&mut s, c, PlayerId::BARBARIAN, &mut events);
        assert_eq!(s.cities[&c].owner, PlayerId(1));
        assert!(events.is_empty());
    }

    #[test]
    fn melee_attacker_captures_city_at_zero_hp() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        s.cities.get_mut(&c).unwrap().hp = 5;
        let civ = unit(&mut s, 1, UnitKind::Settler, hex(5, 4));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, a, hex(5, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        let city = &next.cities[&c];
        assert_eq!(city.owner, PlayerId(0));
        assert_eq!(city.hp, 0);
        assert_eq!(city.population, 1);
        assert_eq!(city.resistance, data::CAPTURE_RESISTANCE_TURNS);
        assert!(!city.is_capital);
        assert_eq!(pos(&next, a), hex(5, 4));
        let captive = next
            .units
            .get(&civ)
            .expect("the city's civilian is captured");
        assert_eq!(captive.owner, PlayerId(0));
        assert_eq!(captive.moves_left, 0);
        assert_eq!(captive.pos, hex(5, 4));
        assert!(events.iter().any(|e| matches!(
            e, Event::SettlerCaptured { unit, from: PlayerId(1), to: PlayerId(0), .. }
                if *unit == civ
        )));
        assert_eq!(next.map.get(hex(5, 5)).unwrap().owner, Some(PlayerId(0)));
        // City melee defense damages the attacker.
        let Event::Battle {
            damage_to_attacker, ..
        } = &unit_battles(&events)[0]
        else {
            unreachable!()
        };
        assert!(*damage_to_attacker > 0);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::CityCaptured { .. }))
        );
    }

    #[test]
    fn city_not_at_zero_is_not_entered() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 4), 3);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, a, hex(5, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert!(next.cities[&c].hp < data::CITY_HP);
        assert_eq!(next.cities[&c].owner, PlayerId(1));
        assert_eq!(pos(&next, a), hex(4, 4));
    }

    #[test]
    fn military_entering_a_settler_tile_captures_it() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Scout, hex(3, 4));
        let st = unit(&mut s, 1, UnitKind::Settler, hex(4, 4));
        goto(&mut s, a, hex(4, 4));
        goto(&mut s, st, hex(8, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        let captured = &next.units[&st];
        assert_eq!(captured.owner, PlayerId(0));
        assert_eq!(captured.pos, hex(4, 4), "pinned, then captured");
        assert_eq!(captured.moves_left, 0);
        assert_eq!(captured.goto, None);
        assert_eq!(pos(&next, a), hex(4, 4));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::SettlerCaptured { .. }))
        );
        assert!(battles(&events).is_empty());
    }

    #[test]
    fn military_captures_a_moving_hostile_mongol_settler() {
        let mut s = game(&[0, 1]);
        s.players[1].faction = Faction::Mongols;
        let attacker = unit(&mut s, 0, UnitKind::Scout, hex(3, 4));
        let settler = unit(&mut s, 1, UnitKind::Settler, hex(4, 4));
        goto(&mut s, attacker, hex(4, 4));
        goto(&mut s, settler, hex(8, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, attacker), hex(4, 4));
        assert_eq!(next.units[&settler].owner, PlayerId(0));
        assert_eq!(pos(&next, settler), hex(4, 4));
        assert_eq!(next.units[&settler].goto, None);
        assert!(events.iter().any(|e| matches!(e,
            Event::SettlerCaptured { unit, from: PlayerId(1), to: PlayerId(0), .. } if *unit == settler
        )));
    }

    #[test]
    fn explicit_settler_capture_spends_a_scouts_remaining_movement() {
        let mut s = game(&[0, 1]);
        let scout = unit(&mut s, 0, UnitKind::Scout, hex(3, 4));
        let first = unit(&mut s, 1, UnitKind::Settler, hex(4, 4));
        let second = unit(&mut s, 1, UnitKind::Settler, hex(5, 4));
        goto(&mut s, scout, hex(4, 4));

        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, scout), hex(4, 4));
        assert_eq!(next.units[&scout].moves_left, 0);
        assert_eq!(next.units[&scout].goto, None);
        assert_eq!(next.units[&first].owner, PlayerId(0));
        assert_eq!(next.units[&first].moves_left, 0);
        assert_eq!(next.units[&second].owner, PlayerId(1));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::SettlerCaptured { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn civilians_are_blocked_by_enemies() {
        let mut s = game(&[0, 1]);
        let st = unit(&mut s, 0, UnitKind::Settler, hex(3, 4));
        let _e = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
        goto(&mut s, st, hex(4, 4));
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&next, st), hex(3, 4));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::MoveBlocked { .. }))
        );
    }

    #[test]
    fn pathing_routes_around_parked_friends() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(2, 4));
        let _parked = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        goto(&mut s, a, hex(4, 4));
        let (next, _) = resolve_units(&s, &no_orders());
        assert_ne!(pos(&next, a), hex(2, 4));
        assert_ne!(pos(&next, a), hex(3, 4));
    }

    #[test]
    fn a_move_blocked_two_turns_by_a_peaceful_unit_stops_and_asks_for_orders() {
        let mut s = game(&[0, 1]);
        s.diplomacy.as_mut().unwrap().relations[0].state = crate::diplomacy::RelationState::Peace;
        let a = unit(&mut s, 0, UnitKind::Pikeman, hex(2, 4));
        let _scout = unit(&mut s, 1, UnitKind::Scout, hex(3, 4));
        goto(&mut s, a, hex(3, 4));
        let (s1, e1) = resolve_units(&s, &no_orders());
        assert_eq!(pos(&s1, a), hex(2, 4));
        assert_eq!(
            s1.units[&a].goto,
            Some(hex(3, 4)),
            "one blocked turn keeps the order"
        );
        assert!(!s1.units[&a].needs_orders);
        assert!(!e1.iter().any(|e| matches!(e, Event::MoveBlocked { reason, .. } if reason.starts_with(MOVE_STOPPED_PREFIX))));
        // Save/load between the turns keeps the count.
        let s1: GameState = serde_json::from_str(&serde_json::to_string(&s1).unwrap()).unwrap();
        let (s2, e2) = resolve_units(&s1, &no_orders());
        let u = &s2.units[&a];
        assert_eq!(
            (u.goto, u.needs_orders, u.blocked_turns),
            (None, true, 0),
            "the move stops and asks for orders"
        );
        let (col, row) = hex(3, 4).to_offset();
        let want = format!("{MOVE_STOPPED_PREFIX}blocked by P1's Scout at {col},{row}");
        assert!(
            e2.iter().any(|e| matches!(e, Event::MoveBlocked { unit, reason, .. } if *unit == a && *reason == want)),
            "{e2:?}"
        );
        // A unit that makes progress never accumulates blocked turns.
        let mut t = game(&[0, 1]);
        let b = unit(&mut t, 0, UnitKind::Pikeman, hex(1, 4));
        goto(&mut t, b, hex(8, 4));
        let (t1, _) = resolve_units(&t, &no_orders());
        assert_eq!(
            (t1.units[&b].blocked_turns, t1.units[&b].needs_orders),
            (0, false)
        );
        assert!(
            !serde_json::to_string(&t1.units[&b])
                .unwrap()
                .contains("blocked_turns")
        );
    }
}

#[cfg(test)]
#[path = "movement_contention_tests.rs"]
mod contention_tests;

#[cfg(test)]
#[path = "pursuit_tests.rs"]
mod pursuit_tests;

#[cfg(test)]
#[path = "friendly_route_tests.rs"]
mod friendly_route_tests;

#[cfg(test)]
#[path = "reinforcement_tests.rs"]
mod reinforcement_tests;
#[cfg(test)]
#[path = "rival_assault_tests.rs"]
mod rival_assault_tests;

#[cfg(test)]
#[path = "capture_orders_tests.rs"]
mod capture_orders_tests;

#[cfg(test)]
#[path = "held_shot_tests.rs"]
mod held_shot_tests;

#[cfg(test)]
#[path = "peaceful_contest_tests.rs"]
mod peaceful_contest_tests;
