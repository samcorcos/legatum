//! Unit phases other than movement and combat: turn reset, unit orders (Sleep, Skip, Disband,
//! Upgrade, MoveUnit gotos, FoundCity/RangedAttack collection), ruins, found-city conflicts,
//! elimination, healing and fortification.

use super::battle;
use super::is_military;
use crate::bitset::BitSet;
use crate::data::{self, FORTIFY_MAX, FORTIFY_PER_TURN, UNIT_HP, UPGRADE_COST_PERCENT};
use crate::events::{Event, RuinBuy, RuinReward};
use crate::hex::Hex;
use crate::orders::Order;
use crate::path;
use crate::rng::{Rng, purpose};
use crate::state::GameState;
use crate::types::*;
use crate::view::PlayerView;
use std::collections::{BTreeMap, BTreeSet};

/// Gold ruins give.
pub const RUIN_GOLD: i32 = 40;
/// Stored science ruins give.
pub const RUIN_SCIENCE: i32 = 25;

const NO_SUCH_UNIT: &str = "No such unit of yours";

/// Orders collected in phase 1 for later phases.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitPlan {
    pub automation_paths: BTreeMap<UnitId, Vec<Hex>>,
    /// Phase 2, in player then submission order.
    pub ranged: Vec<(PlayerId, UnitId, Hex)>,
    /// Phase 5.
    pub found: Vec<(PlayerId, UnitId)>,
    /// Attackers ordered onto a unit or city of a team their team declares War on this turn.
    /// War begins next turn, so they hold until then instead of bouncing off a peaceful unit
    /// or closed borders.
    pub awaiting_war: BTreeSet<UnitId>,
}

impl UnitPlan {
    /// An attack on a unit, or a military move onto a city, of a team this order list declares
    /// War on waits for that War.
    fn note_attack(
        &mut self,
        state: &GameState,
        unit: UnitId,
        to: Hex,
        target: Option<UnitId>,
        declared: &BTreeSet<(TeamId, TeamId)>,
    ) {
        let u = &state.units[&unit];
        let waits = match target {
            Some(t) => !super::hostile(state, u.owner, state.units[&t].owner),
            None => {
                is_military(u.kind)
                    && state.cities.values().any(|c| {
                        c.pos == to
                            && !state.same_team(u.owner, c.owner)
                            && !super::hostile(state, u.owner, c.owner)
                            && declared.contains(&(state.team_of(u.owner), state.team_of(c.owner)))
                    })
            }
        };
        if waits {
            self.awaiting_war.insert(unit);
        }
    }
}

/// (declaring team, target team) for every valid DeclareWar order this turn.
pub(super) fn declared_wars(
    state: &GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
) -> BTreeSet<(TeamId, TeamId)> {
    let mut wars = BTreeSet::new();
    for (&player, os) in orders {
        if !os.iter().any(|o| matches!(o, Order::DeclareWar { .. })) {
            continue;
        }
        let errors = crate::diplomacy::validate_orders(state, player, os);
        for (i, o) in os.iter().enumerate() {
            if let Order::DeclareWar { against, .. } = o
                && !errors.iter().any(|e| e.index == i)
            {
                wars.insert((state.team_of(player), *against));
            }
        }
    }
    wars
}

/// The unit an explicit melee attack order on `to` targets: a foreign unit the player's team
/// sees there (military first) whose team is hostile, or is being declared War on this turn. A
/// foreign city on the tile makes the city the target, so no unit is pursued. Anything else is a
/// plain move and never pursues into combat.
fn explicit_attack_target(
    state: &GameState,
    player: PlayerId,
    kind: UnitKind,
    to: Hex,
    declared: &BTreeSet<(TeamId, TeamId)>,
) -> Option<UnitId> {
    let foreign = |owner: PlayerId| !state.same_team(player, owner);
    if !is_military(kind)
        || state
            .cities
            .values()
            .any(|c| c.pos == to && foreign(c.owner))
    {
        return None;
    }
    let target = state
        .units
        .values()
        .filter(|u| u.pos == to && foreign(u.owner))
        .filter(|u| {
            super::hostile(state, player, u.owner)
                || declared.contains(&(state.team_of(player), state.team_of(u.owner)))
        })
        .min_by_key(|u| (!is_military(u.kind), u.id))?;
    let seen = crate::visibility::visible_tiles(state, state.team_of(player));
    state
        .map
        .index(to)
        .is_some_and(|i| seen.get(i))
        .then_some(target.id)
}

/// The unit a unit-action order is for (Move/Sleep/Skip/Found/Disband/RangedAttack/Upgrade).
pub fn order_unit(order: &Order) -> Option<UnitId> {
    match order {
        Order::SetAutomation { unit, .. }
        | Order::SetAutoFire { unit, .. }
        | Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::SleepUnit { unit, .. }
        | Order::SkipUnit { unit }
        | Order::FoundCity { unit }
        | Order::DisbandUnit { unit }
        | Order::RangedAttack { unit, .. }
        | Order::UpgradeUnit { unit }
        | Order::UpgradeSiege { unit } => Some(*unit),
        _ => None,
    }
}

/// Gold to upgrade a unit of this kind, if it has an upgrade.
pub fn upgrade_cost(kind: UnitKind) -> Option<i32> {
    upgrade_cost_for(Faction::India, kind)
}

pub fn upgrade_cost_for(faction: Faction, kind: UnitKind) -> Option<i32> {
    Some(step_cost_for(
        faction,
        kind,
        data::unit_stats(kind).upgrades_to?,
    ))
}

/// Gold for one upgrade step from `kind` to `to`.
fn step_cost_for(faction: Faction, kind: UnitKind, to: UnitKind) -> i32 {
    // Never negative: a faction piece dearer than what it upgrades to (a Keshig is close to a
    // Cataphract's price) upgrades for nothing rather than for a refund.
    let diff =
        (data::unit_stats_for(faction, to).cost - data::unit_stats_for(faction, kind).cost).max(0);
    UPGRADE_COST_PERCENT * diff / 100
}

/// The same price breakdown is used by quotes, validation and resolution.
pub fn upgrade_price(
    state: &GameState,
    player: PlayerId,
    kind: UnitKind,
    assume_logistics: bool,
) -> Option<Breakdown> {
    let to = data::unit_stats(kind).upgrades_to?;
    Some(step_price(state, player, kind, to, assume_logistics))
}

/// The price breakdown of one step from `kind` to `to` (one of `data::upgrade_choices(kind)`).
fn step_price(
    state: &GameState,
    player: PlayerId,
    kind: UnitKind,
    to: UnitKind,
    assume_logistics: bool,
) -> Breakdown {
    let base = step_cost_for(Faction::India, kind, to);
    let modified = step_cost_for(state.player(player).faction, kind, to);
    let mut b = Breakdown::default();
    b.add("Base upgrade Gold", base);
    b.add("Faction", modified - base);
    if assume_logistics
        || state
            .player(player)
            .techs
            .contains(&TechId::MilitaryEngineering)
    {
        b.add("Military Engineering (50%)", b.total / 2 - b.total);
    }
    b
}

/// The newest researched form and every paid step to reach it, by the default line.
pub fn upgrade_plan(
    state: &GameState,
    player: PlayerId,
    from: UnitKind,
) -> Option<(UnitKind, Vec<UnitKind>, Breakdown)> {
    upgrade_plan_via(state, player, from, None)
}

/// `upgrade_plan` with the first step chosen (`via`, one of `data::upgrade_choices(from)`; None:
/// the default): a Biplane goes on to a Fighter or a Bomber, then on along that line.
pub fn upgrade_plan_via(
    state: &GameState,
    player: PlayerId,
    from: UnitKind,
    via: Option<UnitKind>,
) -> Option<(UnitKind, Vec<UnitKind>, Breakdown)> {
    let first = match via {
        Some(choice) => data::upgrade_choices(from)
            .contains(&choice)
            .then_some(choice)?,
        None => data::unit_stats(from).upgrades_to?,
    };
    let mut kind = from;
    let mut next = Some(first);
    let mut steps = Vec::new();
    let mut price = Breakdown::default();
    while let Some(to) = next {
        if data::unit_stats(to)
            .requires
            .is_some_and(|tech| !state.player(player).techs.contains(&tech))
        {
            break;
        }
        for (label, value) in step_price(state, player, kind, to, false).parts {
            price.add(label, value);
        }
        steps.push(to);
        kind = to;
        next = data::unit_stats(kind).upgrades_to;
    }
    (!steps.is_empty()).then_some((kind, steps, price))
}

pub(crate) fn lowercase_first(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_lowercase().chain(chars).collect())
}

/// Why this unit cannot upgrade now with `gold`, or (new kind, cost).
pub fn check_upgrade(
    state: &GameState,
    player: PlayerId,
    unit: UnitId,
    gold: i32,
) -> Result<(UnitKind, i32), String> {
    let u = &state.units[&unit];
    if crate::company::is_company(u) {
        return Err("a hired Company cannot be upgraded: hire the next one instead".into());
    }
    let Some(first) = data::unit_stats(u.kind).upgrades_to else {
        return Err(format!("{} has no upgrade", data::unit_name(u.kind)));
    };
    if let Some(t) = data::unit_stats(first).requires
        && !state.player(player).techs.contains(&t)
    {
        return Err(format!(
            "upgrading to {} needs {}",
            data::unit_name(first),
            data::tech_name(t)
        ));
    }
    if state.map.get(u.pos).and_then(|t| t.owner) != Some(player) {
        return Err("units upgrade only in own territory".into());
    }
    let (to, _, price) = upgrade_plan(state, player, u.kind).expect("researched upgrade");
    let cost = price.total;
    if gold < cost {
        return Err(format!("upgrade costs {cost} Gold, you have {gold}"));
    }
    if let Some(reason) = crate::resources::upgrade_blocker(state, player, u, to, 0) {
        return Err(format!(
            "upgrading to {}: {}",
            data::unit_name(to),
            lowercase_first(&reason)
        ));
    }
    Ok((to, cost))
}

/// A heal order needs something to heal and somewhere to do it: a unit at full HP has nothing to
/// wait for, and one in closed foreign land heals nothing there and would never wake.
pub fn check_heal(state: &GameState, u: &crate::state::Unit) -> Result<(), String> {
    if u.hp >= UNIT_HP {
        return Err("already at full health: there is nothing to heal".into());
    }
    if resting_heal(state, u).total <= 0 {
        return Err("it does not heal here: closed foreign territory".into());
    }
    Ok(())
}

/// Occupation lock: the refusal when `order` would take a locked unit out of its city, or
/// disband it to dodge the lock (`rules::occupation`). Staying put, fortifying, healing and
/// shooting are never refused.
fn occupation_refusal(state: &GameState, u: &crate::state::Unit, order: &Order) -> Option<String> {
    let leaves = match order {
        Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => *to != u.pos,
        Order::SetAutomation { automation, .. } => automation.is_some(),
        Order::DisbandUnit { .. } => true,
        _ => false,
    };
    leaves
        .then(|| super::occupation::refusal(state, u))
        .flatten()
}

/// Validation for one unit-action order. `acted` tracks units already ordered in this list,
/// `gold` the treasury left after earlier upgrades in the list. `declared` holds the teams this
/// list validly declares War on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_unit_order(
    state: &GameState,
    player: PlayerId,
    order: &Order,
    declared: &BTreeSet<TeamId>,
    acted: &mut BTreeSet<UnitId>,
    gold: &mut i32,
    vision: &mut Option<BitSet>,
    view: &mut Option<PlayerView>,
) -> Result<(), String> {
    let id = order_unit(order).expect("unit order");
    let u = state
        .units
        .get(&id)
        .filter(|u| u.owner == player)
        .ok_or_else(|| NO_SUCH_UNIT.to_string())?;
    if !acted.insert(id) {
        return Err(format!(
            "Your {} already has an order this turn (one per unit)",
            data::unit_name_for(state.player(player).faction, u.kind)
        ));
    }
    if let Some(city) = state
        .city_at(u.pos)
        .filter(|c| state.allied(c.owner, u.owner))
        && !matches!(order, Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } if *to != u.pos)
    {
        return Err(format!(
            "{} is in allied city {}; it must move out this turn",
            data::unit_name_for(state.made_by(u), u.kind),
            city.name
        ));
    }
    if let Some(reason) = occupation_refusal(state, u, order) {
        return Err(reason);
    }
    match order {
        Order::MoveUnitPath { to, path, .. } => {
            let view = view.get_or_insert_with(|| crate::view::order_check_view(state, player));
            check_known_path(view, u.kind, u.pos, *to, path)?;
            allied_city_destination(view, player, *to)
        }
        Order::SetAutomation {
            automation,
            path,
            stop_reason,
            ..
        } => {
            let view = view.get_or_insert_with(|| crate::view::order_check_view(state, player));
            check_automation(view, id, *automation, path, *stop_reason)
        }
        Order::SetAutoFire { enabled, .. } => {
            if *enabled
                && (data::unit_stats(u.kind).ranged <= 0
                    || !state
                        .cities
                        .values()
                        .any(|c| c.owner == player && c.pos == u.pos))
            {
                Err("automatic fire needs a ranged unit in one of your cities".into())
            } else {
                Ok(())
            }
        }
        Order::MoveUnit { to, .. } => {
            let view = view.get_or_insert_with(|| crate::view::order_check_view(state, player));
            check_known_move(view, u.kind, u.pos, *to).or_else(|err| {
                if declared.is_empty() {
                    return Err(err);
                }
                // Declared this turn: War begins next turn, when the move needs these borders.
                let mut at_war = view.clone();
                for r in &mut at_war.diplomacy.relations {
                    if (r.a == view.team && declared.contains(&r.b))
                        || (r.b == view.team && declared.contains(&r.a))
                    {
                        r.state = crate::diplomacy::RelationState::War;
                    }
                }
                check_known_move(&at_war, u.kind, u.pos, *to).map_err(|_| err)
            })?;
            allied_city_destination(view, player, *to)
        }
        Order::FoundCity { .. }
            if u.kind != UnitKind::Settler
                && !(state.player(player).faction == Faction::Rome
                    && data::unit_class(u.kind) == Some(UnitClass::Melee)
                    && !state
                        .setup
                        .rule_experiments
                        .rule_off(crate::experiments::FactionRule::RomeVeteranCities)) =>
        {
            Err(if state.player(player).faction == Faction::Rome {
                "Only Settlers or Roman melee veterans found cities"
            } else {
                "Only Settlers found cities"
            }
            .into())
        }
        Order::FoundCity { .. } => super::cities::check_found_city(state, id),
        Order::RangedAttack { target, .. } => {
            battle::ranged_target(state, player, id, *target, vision).map(|_| ())
        }
        Order::SleepUnit {
            until_healed: true, ..
        } => check_heal(state, u),
        Order::UpgradeUnit { .. } => {
            let (_, cost) = check_upgrade(state, player, id, *gold)?;
            *gold -= cost;
            Ok(())
        }
        Order::UpgradeSiege { .. } => {
            let price = super::siege::check(state, player, id, *gold, 0)?;
            *gold -= price.total;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn allied_city_destination(view: &PlayerView, player: PlayerId, to: Hex) -> Result<(), String> {
    if let Some(city) = view.cities.iter().find(|city| city.pos == to)
        && !view.same_team(player, city.owner)
        && view.allied(player, city.owner)
    {
        return Err(format!(
            "Allied units can pass through {} but can't stop there.",
            city.name
        ));
    }
    Ok(())
}

/// The reason a route is closed, naming a closed country (Sakoku) when `at` lies in one.
fn closed_borders(view: &PlayerView, what: &str, at: Hex) -> String {
    if let Some(owner) = view
        .tile(at)
        .and_then(|t| t.owner)
        .filter(|o| view.is_closed(*o))
    {
        let name = view
            .players
            .iter()
            .find(|p| p.id == owner)
            .map_or("A rival", |p| p.name.as_str());
        return format!(
            "{what}: {name} has closed the country (Sakoku): no foreign unit may enter its land at Peace"
        );
    }
    what.to_string()
}

fn check_known_path(
    view: &PlayerView,
    kind: UnitKind,
    from: Hex,
    to: Hex,
    path: &[Hex],
) -> Result<(), String> {
    if path.is_empty() || path.len() > view.tiles.len() || path.last() != Some(&to) {
        return Err("route must end at the destination".into());
    }
    let mut prev = from;
    let mut seen = BTreeSet::from([from]);
    for &h in path {
        if prev.distance(h) != 1 || !seen.insert(h) {
            return Err("route must use adjacent tiles without loops".into());
        }
        if view
            .tile(h)
            .and_then(|t| t.terrain)
            .and_then(Terrain::move_cost)
            .is_none()
        {
            return Err("route must use explored, passable land".into());
        }
        if view.movement_path(view.me, kind, prev, h).as_deref() != Some(&[h]) {
            return Err(closed_borders(view, "route crosses closed borders", h));
        }
        let enemy = |owner: PlayerId| view.at_war(view.me, owner);
        if view.faction_of(view.me) != Faction::Mongols
            && h != to
            && (view.units.iter().any(|u| u.pos == h && enemy(u.owner))
                || view.cities.iter().any(|c| c.pos == h && enemy(c.owner)))
        {
            return Err("route must not pass through a known enemy".into());
        }
        prev = h;
    }
    Ok(())
}

fn check_known_move(view: &PlayerView, kind: UnitKind, from: Hex, to: Hex) -> Result<(), String> {
    let tile = view.tile(to).ok_or("destination is off the map")?;
    let Some(terrain) = tile.terrain else {
        // An unknown destination is an exploration order, never a terrain probe.
        return Ok(());
    };
    if terrain.move_cost().is_none() {
        return Err("destination is impassable".into());
    }
    // find_path uses Map only for coordinates/dimensions. Every movement cost comes
    // from PlayerView, with unknown intermediate tiles treated as potentially passable.
    if view.movement_path(view.me, kind, from, to).is_none() {
        return Err(closed_borders(
            view,
            "destination cannot be reached over land with the current borders",
            to,
        ));
    }
    Ok(())
}

/// Routes must be adjacent, known, and affordable this turn. No hidden terrain probes.
fn check_automation(
    view: &PlayerView,
    id: UnitId,
    automation: Option<Automation>,
    path: &[Hex],
    stop_reason: Option<ExploreStopReason>,
) -> Result<(), String> {
    let u = view.units.iter().find(|u| u.id == id).ok_or(NO_SUCH_UNIT)?;
    if !data::can_auto_explore(u.kind) {
        return Err("only Scouts and military units can auto-explore".into());
    }
    if automation.is_some() && stop_reason.is_some() {
        return Err("stopping automation must clear its flag".into());
    }
    let mut from = u.pos;
    let mut moves = u.max_moves;
    let mut visited = BTreeSet::from([from]);
    for &to in path {
        let cost = view
            .tile(to)
            .filter(|t| t.explored)
            .and_then(crate::view::TileView::move_cost);
        let Some(cost) = cost else {
            return Err("auto-explore route must use known land".into());
        };
        if from.distance(to) != 1
            || !visited.insert(to)
            || !super::movement::can_step(moves, u.max_moves, cost)
        {
            return Err("auto-explore route must be adjacent and fit this turn's moves".into());
        }
        // Adjacent routes must use the mover's same border/egress rule. Checking
        // can_enter alone would forbid legal steps out of newly closed territory.
        if view.movement_path(u.owner, u.kind, from, to).as_deref() != Some(&[to]) {
            return Err("auto-explore route crosses closed borders".into());
        }
        moves = (moves - cost).max(0);
        from = to;
    }
    Ok(())
}

fn check_move(state: &GameState, from: Hex, to: Hex) -> Result<(), String> {
    if !state.map.contains(to) {
        return Err("destination is off the map".into());
    }
    if state.map.move_cost(to).is_none() {
        return Err("destination is impassable".into());
    }
    if path::land_path(&state.map, from, to).is_none() {
        return Err("destination cannot be reached over land".into());
    }
    Ok(())
}

/// Start of resolution: full moves for everyone, nobody has acted yet.
pub fn reset_turn(state: &mut GameState) {
    let railways: Vec<bool> = state
        .players
        .iter()
        .map(|p| p.techs.contains(&TechId::Railways))
        .collect();
    let rules = state.setup.rule_experiments;
    for u in state.units.values_mut() {
        // Barbarians (a reserved owner outside `players`) use the neutral India stats.
        let faction = state
            .players
            .get(u.owner.0 as usize)
            .map_or(Faction::India, |p| p.faction);
        // A unit that starts its turn on a road gets extra moves: +2 with Railways, else +1.
        let road = if !state.map.is_road(u.pos) {
            0
        } else if railways.get(u.owner.0 as usize).copied().unwrap_or(false) {
            data::RAILWAY_MOVES
        } else {
            data::ROAD_MOVES
        };
        u.moves_left =
            rules.unit_moves_made_by(faction, u.origin.unwrap_or(faction), u.kind) + road;
        u.acted = false;
        if u.automation.is_some() {
            u.goto = None;
        }
    }
}

/// Phase 1 for units, per player in id order, orders in submission order. Invalid orders emit
/// OrderFailed and are skipped. Ranged and found-city orders are collected for later phases.
pub fn apply_unit_orders(
    state: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    events: &mut Vec<Event>,
) -> UnitPlan {
    let mut plan = UnitPlan::default();
    let declared = declared_wars(state, orders);
    for (&player, os) in orders {
        let mut acted = BTreeSet::new();
        let mut view = None;
        for order in os.iter() {
            let Some(id) = order_unit(order) else {
                continue;
            };
            let fail = |reason: String| Event::OrderFailed { player, reason };
            if !acted.insert(id) {
                events.push(fail(format!(
                    "Unit {} already had an order this turn; extra order ignored",
                    id.0
                )));
                continue;
            }
            let Some(u) = state.units.get(&id).filter(|u| u.owner == player) else {
                events.push(fail(NO_SUCH_UNIT.into()));
                continue;
            };
            if let Some(reason) = occupation_refusal(state, u, order) {
                events.push(fail(reason));
                continue;
            }
            let (pos, kind) = (u.pos, u.kind);
            // Every new unit order replaces its standing intent, even if it later fails.
            let u = state.units.get_mut(&id).expect("unit");
            let keep_auto_fire = matches!(order, Order::RangedAttack { .. })
                && u.standing_order == Some(StandingOrder::AutoFire);
            if !keep_auto_fire {
                u.standing_order = None;
            }
            u.attack_target = None;
            if !matches!(order, Order::SetAutomation { .. }) {
                state.units.get_mut(&id).expect("unit").automation = None;
            }
            match order {
                Order::SetAutoFire {
                    enabled, target, ..
                } => {
                    if *enabled
                        && (data::unit_stats(kind).ranged <= 0
                            || !state
                                .cities
                                .values()
                                .any(|c| c.owner == player && c.pos == pos))
                    {
                        events.push(fail(
                            "automatic fire needs a ranged unit in one of your cities".into(),
                        ));
                        continue;
                    }
                    let u = state.units.get_mut(&id).expect("unit");
                    u.goto = None;
                    u.path.clear();
                    u.path_stop_on_block = false;
                    u.needs_orders = false;
                    u.standing_order = enabled.then_some(StandingOrder::AutoFire);
                    if *enabled && let Some(target) = target {
                        plan.ranged.push((player, id, *target));
                    }
                }
                Order::MoveUnitPath {
                    to,
                    path,
                    stop_on_block,
                    ..
                } => {
                    match check_known_path(
                        view.get_or_insert_with(|| crate::view::order_check_view(state, player)),
                        kind,
                        pos,
                        *to,
                        path,
                    ) {
                        Ok(()) => {
                            let target =
                                explicit_attack_target(state, player, kind, *to, &declared);
                            plan.note_attack(state, id, *to, target, &declared);
                            let u = state.units.get_mut(&id).expect("unit");
                            if u.goto != Some(*to) {
                                u.blocked_turns = 0;
                            }
                            u.goto = Some(*to);
                            u.path = path.clone();
                            u.path_stop_on_block = *stop_on_block;
                            u.needs_orders = false;
                            u.attack_target = target;
                        }
                        Err(reason) => events.push(fail(reason)),
                    }
                }
                Order::SetAutomation {
                    automation,
                    path,
                    stop_reason,
                    ..
                } => {
                    let view =
                        view.get_or_insert_with(|| crate::view::order_check_view(state, player));
                    if let Err(reason) = check_automation(view, id, *automation, path, *stop_reason)
                    {
                        events.push(fail(reason));
                        continue;
                    }
                    let u = state.units.get_mut(&id).expect("unit");
                    u.path.clear();
                    u.path_stop_on_block = false;
                    u.automation = *automation;
                    u.needs_orders = false;
                    u.goto = path.last().copied();
                    plan.automation_paths.insert(id, path.clone());
                    if let Some(reason) = stop_reason {
                        events.push(Event::AutomationStopped {
                            unit: id,
                            owner: player,
                            at: pos,
                            reason: *reason,
                        });
                    }
                }
                Order::MoveUnit { to, .. } => match check_move(state, pos, *to) {
                    Ok(()) => {
                        let target = explicit_attack_target(state, player, kind, *to, &declared);
                        plan.note_attack(state, id, *to, target, &declared);
                        let u = state.units.get_mut(&id).expect("unit");
                        u.attack_target = target;
                        // Re-confirming the same destination keeps counting blocked turns.
                        if u.goto != (*to != pos).then_some(*to) {
                            u.blocked_turns = 0;
                        }
                        u.goto = (*to != pos).then_some(*to);
                        u.path.clear();
                        u.path_stop_on_block = false;
                        u.needs_orders = false;
                    }
                    // The true obstruction may still be hidden. Do not reveal its terrain
                    // or connectivity in a client-facing resolution event either.
                    Err(_) => events.push(fail(format!("Move of unit {} failed", id.0))),
                },
                Order::SleepUnit { .. } | Order::SkipUnit { .. } => {
                    let heal = matches!(
                        order,
                        Order::SleepUnit {
                            until_healed: true,
                            ..
                        }
                    );
                    if heal && let Err(reason) = check_heal(state, &state.units[&id]) {
                        events.push(fail(format!("Unit {} cannot heal: {reason}", id.0)));
                        continue;
                    }
                    let u = state.units.get_mut(&id).expect("unit");
                    u.goto = None;
                    u.path.clear();
                    u.path_stop_on_block = false;
                    u.needs_orders = false;
                    u.standing_order = match order {
                        Order::SleepUnit { .. } if heal => Some(StandingOrder::Heal),
                        Order::SleepUnit { .. } => Some(StandingOrder::Sleep),
                        _ => None,
                    };
                }
                Order::FoundCity { .. } => {
                    if kind == UnitKind::Settler
                        || (state.player(player).faction == Faction::Rome
                            && data::unit_class(kind) == Some(UnitClass::Melee)
                            && !state
                                .setup
                                .rule_experiments
                                .rule_off(crate::experiments::FactionRule::RomeVeteranCities))
                    {
                        state.units.get_mut(&id).expect("unit").goto = None;
                        plan.found.push((player, id));
                    } else {
                        events.push(fail(format!("Unit {} cannot found a city", id.0)));
                    }
                }
                Order::DisbandUnit { .. } => {
                    state.units.remove(&id);
                    events.push(Event::UnitDisbanded {
                        unit: id,
                        owner: player,
                        reason: "Disbanded by order".into(),
                    });
                }
                Order::RangedAttack { target, .. } => plan.ranged.push((player, id, *target)),
                Order::UpgradeSiege { .. } => {
                    let gold = state.player(player).gold;
                    match super::siege::check(state, player, id, gold, 0) {
                        Ok(price) => {
                            state.player_mut(player).gold -= price.total;
                            let u = state.units.get_mut(&id).expect("unit");
                            u.siege = true;
                            u.moves_left = 0;
                            events.push(Event::SiegeBought {
                                unit: id,
                                owner: player,
                                gold: price.total,
                            });
                        }
                        Err(r) => events.push(fail(format!("Siege for unit {} failed: {r}", id.0))),
                    }
                }
                Order::UpgradeUnit { .. } => {
                    let gold = state.player(player).gold;
                    match check_upgrade(state, player, id, gold) {
                        Ok((to, cost)) => {
                            let castrum = state.player(player).faction == Faction::Rome
                                && data::unit_class(kind) == Some(UnitClass::Melee)
                                && !state
                                    .setup
                                    .rule_experiments
                                    .rule_off(crate::experiments::FactionRule::RomeCastrum)
                                && state.city_at(pos).is_some_and(|c| {
                                    c.owner == player && c.buildings.contains(&BuildingKind::Walls)
                                });
                            state.player_mut(player).gold -= cost;
                            let u = state.units.get_mut(&id).expect("unit");
                            u.kind = to;
                            u.moves_left = 0;
                            if castrum {
                                u.hp = (u.hp + 50).min(data::UNIT_HP);
                            }

                            // An Industrial or later unit has Siege built in: the bought one
                            // (and the Iron it used) goes.
                            if data::built_in_siege(to) {
                                u.siege = false;
                            }
                            events.push(Event::UnitUpgraded {
                                unit: id,
                                owner: player,
                                to,
                                gold: cost,
                            });
                        }
                        Err(r) => {
                            events.push(fail(format!("Upgrade of unit {} failed: {r}", id.0)))
                        }
                    }
                }
                _ => {}
            }
        }
    }
    plan
}

/// Explore the ruin under `unit`; Athens also gain 20 Science.
pub fn explore_ruin(state: &mut GameState, unit: UnitId, events: &mut Vec<Event>) {
    let (owner, pos) = {
        let u = &state.units[&unit];
        (u.owner, u.pos)
    };
    // Barbarians walk past ruins; only empires explore them.
    if owner.is_barbarian() {
        return;
    }
    match state.map.get_mut(pos) {
        Some(t) if t.ruin => t.ruin = false,
        _ => return,
    }
    let roll = Rng::derive(
        state.dice_seed(),
        &[state.turn as u64, purpose::RUINS, unit.0 as u64],
    )
    .range(0, 23);
    let nearest_city = state
        .cities_of(owner)
        .filter(|c| c.population < crate::validation::MAX_POPULATION)
        .min_by_key(|c| (c.pos.distance(pos), c.id))
        .map(|c| c.id);
    let scout_spawn = if roll >= 20 {
        super::cities::spawn_neighbour_tile(state, owner, pos, UnitKind::Scout)
    } else {
        None
    };
    // Redistribute the removed map reward's three slots proportionally: Gold 9,
    // Science 9, population 2, Scout 4. Fallbacks still pay Gold.
    let reward = match (roll, nearest_city, scout_spawn) {
        (9..=17, _, _) => {
            state.player_mut(owner).science_stored =
                crate::validation::add_stored(state.player(owner).science_stored, RUIN_SCIENCE);
            RuinReward::Science {
                amount: RUIN_SCIENCE,
                research: None,
            }
        }
        (18..=19, Some(c), _) => {
            let before = crate::economy::city_totals(state, &state.cities[&c]);
            let city = state.cities.get_mut(&c).expect("city");
            city.population += 1;
            let population = city.population;
            // Population is territory: the new population claims its tile at once.
            crate::rules::cities::settle_territory(state, c, events);
            let after = crate::economy::city_totals(state, &state.cities[&c]);
            RuinReward::Population {
                city: c,
                amount: 1,
                population,
                yields: Yields::new(
                    after.food - before.food,
                    after.production - before.production,
                    after.gold - before.gold,
                    after.science - before.science,
                ),
            }
        }
        (20..=23, _, Some(at)) => {
            let scout = crate::game::spawn_unit(state, owner, UnitKind::Scout, at);
            // No extra movement rounds, chained finds, free order or rest (fortification) on its
            // birth turn.
            let u = state.units.get_mut(&scout).expect("new Scout");
            u.moves_left = 0;
            u.acted = true;
            RuinReward::FreeScout { unit: scout }
        }
        _ => {
            let amount = RUIN_GOLD;
            let before = state.player(owner).gold;
            let balance = crate::validation::credit_gold(before, amount);
            let affordable = state
                .cities_of(owner)
                .filter(|c| c.resistance == 0)
                .filter_map(|c| {
                    let cost = crate::economy::buy_cost(state, c)?;
                    (cost > before && cost <= balance).then_some(RuinBuy {
                        city: c.id,
                        item: c.producing?,
                        cost,
                    })
                })
                .min_by_key(|b| (b.cost, b.city));
            state.player_mut(owner).gold = balance;
            RuinReward::Gold {
                amount,
                balance,
                affordable,
            }
        }
    };
    if state.player(owner).faction == Faction::Athens
        && !state
            .setup
            .rule_experiments
            .rule_off(crate::experiments::FactionRule::AthensRuins)
    {
        state.player_mut(owner).science_stored =
            crate::validation::add_stored(state.player(owner).science_stored, 20);
    }
    state.player_mut(owner).ruins_explored += 1;
    events.push(Event::RuinExplored {
        unit,
        owner,
        at: pos,
        reward,
    });
}

/// Phase 5, first half: founders whose found-city orders are still possible, minus every order
/// with another eligible found-city order inside either founder's spacing limit (all fail, units kept).
/// Returns the founders to pass to `cities::found_city`, in id order.
pub fn found_city_candidates(
    state: &GameState,
    found: &[(PlayerId, UnitId)],
    events: &mut Vec<Event>,
) -> Vec<UnitId> {
    let mut live: BTreeMap<UnitId, (PlayerId, Hex)> = BTreeMap::new();
    for (player, id) in found {
        match state.units.get(id) {
            Some(u)
                if u.owner == *player
                    && (u.kind == UnitKind::Settler
                        || (state.player(*player).faction == Faction::Rome
                            && data::unit_class(u.kind) == Some(UnitClass::Melee))) =>
            {
                match super::cities::check_found_city(state, *id) {
                    Ok(()) => {
                        live.insert(*id, (*player, u.pos));
                    }
                    Err(reason) => events.push(Event::OrderFailed {
                        player: *player,
                        reason: format!("Unit {} could not found a city: {reason}", id.0),
                    }),
                }
            }
            _ => events.push(Event::OrderFailed {
                player: *player,
                reason: format!("Unit {} is no longer available to found a city", id.0),
            }),
        }
    }
    let mut out = Vec::new();
    for (id, (player, pos)) in &live {
        let conflict = live.iter().any(|(o, (other_player, p))| {
            o != id
                && p.distance(*pos)
                    <= state
                        .setup
                        .rule_experiments
                        .min_city_gap(state.player(*player).faction)
                        .max(
                            state
                                .setup
                                .rule_experiments
                                .min_city_gap(state.player(*other_player).faction),
                        )
        });
        if conflict {
            events.push(Event::OrderFailed {
                player: *player,
                reason: format!(
                    "Settler {} could not found a city: another settler tried to found too close this turn",
                    id.0
                ),
            });
        } else {
            out.push(*id);
        }
    }
    out
}

/// An empire with no cities and no settlers is eliminated and its units removed.
pub fn check_elimination(state: &mut GameState, events: &mut Vec<Event>) {
    for i in 0..state.players.len() {
        let p = state.players[i].id;
        if !state.players[i].alive {
            continue;
        }
        let has_city = state.cities_of(p).next().is_some();
        let has_settler = state.units_of(p).any(|u| u.kind == UnitKind::Settler);
        if !has_city && !has_settler {
            state.player_mut(p).alive = false;
            state.units.retain(|_, u| u.owner != p);
            events.push(Event::PlayerEliminated { player: p });
        }
    }
}

/// The same breakdown drives resting resolution and the owning player's forecast.
pub fn resting_heal(state: &GameState, u: &crate::state::Unit) -> Breakdown {
    if u.owner.is_barbarian() {
        return super::barbarians::resting_heal();
    }
    if !state.unit_in_open_land(u) {
        return Breakdown {
            total: 0,
            parts: vec![("Closed foreign territory".into(), 0)],
        };
    }
    let mut result = Breakdown::default();
    let home = state
        .map
        .get(u.pos)
        .is_some_and(|t| t.owner == Some(u.owner));
    result.add(
        if home { "Home territory" } else { "Field" },
        if home {
            data::HEAL_TERRITORY
        } else {
            data::HEAL_FIELD
        },
    );
    result
}

fn is_sleeping(u: &crate::state::Unit) -> bool {
    matches!(
        u.standing_order,
        Some(StandingOrder::Sleep | StandingOrder::Heal)
    )
}

/// Visible military units of teams at War with the sleeper, within its maximum movement radius (team sight
/// counts; remembered fog does not). The baseline is taken after unit orders, before
/// purchases and movement, then refreshed each round. It needs no saved history: re-slept or
/// loaded units accept the contacts already in that radius.
pub type SleepThreats = BTreeMap<UnitId, BTreeSet<UnitId>>;

pub fn sleeping_threats(state: &GameState) -> SleepThreats {
    let teams: BTreeSet<_> = state
        .units
        .values()
        .filter(|u| is_sleeping(u))
        .map(|u| state.team_of(u.owner))
        .collect();
    let mut contacts = SleepThreats::new();
    for team in teams {
        let seen = crate::visibility::visible_tiles(state, team);
        let threats: Vec<_> = state
            .units
            .values()
            .filter(|u| {
                state.team_of(u.owner) != team
                    && is_military(u.kind)
                    && state.map.index(u.pos).is_some_and(|i| seen.get(i))
            })
            .collect();
        for u in state.units.values().filter(|u| {
            is_sleeping(u)
                && state.team_of(u.owner) == team
                && super::occupation::lock(state, u).is_none()
        }) {
            let range = state.unit_moves_of(u);
            // Only armies of a team at War are threats; Peace and allied neighbours never wake,
            // and barbarians never wake a Celtic sleeper (they never attack the Celts).
            let near = threats.iter().filter(|enemy| {
                u.pos.distance(enemy.pos) <= range && super::is_threat(state, u.owner, enemy.owner)
            });
            contacts.insert(u.id, near.map(|enemy| enemy.id).collect());
        }
    }
    contacts
}

/// A sleeper wakes once per enemy that newly comes into view within its movement radius
/// (arriving in sight, approaching from farther away, or becoming hostile). Check every
/// movement round, including enemies that pass out of sight again, and after diplomacy
/// resolves.
pub fn wake_threatened_sleepers(
    state: &mut GameState,
    previous: &mut SleepThreats,
    events: &mut Vec<Event>,
) {
    let current = sleeping_threats(state);
    let waking: Vec<_> = current
        .iter()
        .filter(|(id, contacts)| {
            previous
                .get(id)
                .is_some_and(|old| contacts.difference(old).next().is_some())
        })
        .map(|(id, _)| *id)
        .collect();
    *previous = current;
    for id in waking {
        let u = state.units.get_mut(&id).expect("sleeping unit");
        u.standing_order = None;
        u.needs_orders = true;
        events.push(Event::UnitAwakened {
            unit: id,
            owner: u.owner,
            unit_kind: u.kind,
            at: u.pos,
        });
    }
}

/// Units that neither moved nor fought heal (more in own territory) and military units
/// earn resting defense, asleep or not. This never removes idle units from the order cycle.
pub fn heal_and_fortify(state: &mut GameState) {
    let heals: BTreeMap<_, _> = state
        .units
        .values()
        .map(|u| (u.id, resting_heal(state, u).total))
        .collect();
    let closed: BTreeSet<_> = state
        .units
        .values()
        .filter(|u| !state.unit_in_open_land(u))
        .map(|u| u.id)
        .collect();
    for u in state.units.values_mut() {
        if closed.contains(&u.id) {
            u.fortify = 0;
            continue;
        }
        if u.acted {
            continue;
        }
        let heal = heals[&u.id];
        u.hp = (u.hp + heal).min(UNIT_HP);
        if is_military(u.kind) {
            u.fortify = (u.fortify + FORTIFY_PER_TURN).min(FORTIFY_MAX);
        }
    }
}

/// A unit on a heal order wakes the turn its HP is full: the order ends, it asks for orders
/// again and its owner's team is told. Runs right after `heal_and_fortify`.
pub fn wake_healed(state: &mut GameState, events: &mut Vec<Event>) {
    for u in state.units.values_mut() {
        if u.standing_order == Some(StandingOrder::Heal) && u.hp >= UNIT_HP {
            u.standing_order = None;
            u.needs_orders = true;
            events.push(Event::UnitHealed {
                unit: u.id,
                owner: u.owner,
                unit_kind: u.kind,
                at: u.pos,
            });
        }
    }
}

/// Explicit stops and blocked routes need a new decision even if the scout rested.
pub fn finish_automation(state: &mut GameState, plan: &UnitPlan) {
    for id in plan.automation_paths.keys() {
        if let Some(u) = state.units.get_mut(id)
            && u.automation.is_none()
        {
            u.needs_orders = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::data::HEAL_TERRITORY;
    use crate::view::player_view;

    #[test]
    fn resting_defense_never_creates_a_standing_order_but_sleep_persists() {
        let mut s = game(&[0, 1]);
        let idle = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let skip = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let sleep_soldier = unit(&mut s, 0, UnitKind::Spearman, hex(5, 4));
        let sleep = unit(&mut s, 0, UnitKind::Settler, hex(6, 4));
        let (s, _) = resolve_units(
            &s,
            &orders(
                0,
                vec![
                    Order::SkipUnit { unit: skip },
                    Order::SleepUnit {
                        unit: sleep_soldier,
                        until_healed: false,
                    },
                    Order::SleepUnit {
                        unit: sleep,
                        until_healed: false,
                    },
                ],
            ),
        );
        let (s, _) = resolve_units(&s, &no_orders());
        for id in [idle, skip] {
            assert_eq!(s.units[&id].standing_order, None);
            assert_eq!(s.units[&id].fortify, FORTIFY_MAX);
        }
        for id in [sleep_soldier, sleep] {
            assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Sleep));
        }
        assert_eq!(s.units[&sleep_soldier].fortify, FORTIFY_MAX);
        // Skip ends the sleep and keeps the resting defense; the unit asks for orders again.
        let (s, _) = resolve_units(
            &s,
            &orders(
                0,
                vec![
                    Order::SkipUnit {
                        unit: sleep_soldier,
                    },
                    Order::MoveUnit {
                        unit: sleep,
                        to: hex(6, 5),
                    },
                ],
            ),
        );
        assert_eq!(s.units[&sleep_soldier].standing_order, None);
        assert_eq!(s.units[&sleep_soldier].fortify, FORTIFY_MAX);
        assert!(!s.units[&sleep_soldier].needs_orders);
        assert_eq!(s.units[&sleep].standing_order, None, "any order wakes");
    }

    #[test]
    fn sleep_wakes_when_enemy_moves_into_range_once_even_after_load() {
        for kind in [UnitKind::Settler, UnitKind::Spearman] {
            let mut s = game(&[0, 1]);
            let sleeper = unit(&mut s, 0, kind, hex(4, 4));
            let enemy = unit(&mut s, 1, UnitKind::Spearman, hex(7, 4));
            let (s, _) = resolve_units(
                &s,
                &orders(
                    0,
                    vec![Order::SleepUnit {
                        unit: sleeper,
                        until_healed: false,
                    }],
                ),
            );
            assert_eq!(s.units[&sleeper].standing_order, Some(StandingOrder::Sleep));
            let saved = serde_json::to_string(&s).unwrap();
            let restored: GameState = serde_json::from_str(&saved).unwrap();
            let (s, events) = resolve_units(
                &restored,
                &orders(
                    1,
                    vec![Order::MoveUnit {
                        unit: enemy,
                        to: hex(6, 4),
                    }],
                ),
            );
            assert_eq!(s.units[&sleeper].standing_order, None);
            assert!(s.units[&sleeper].needs_orders);
            assert!(events.contains(&Event::UnitAwakened {
                unit: sleeper,
                owner: PlayerId(0),
                unit_kind: kind,
                at: hex(4, 4),
            }));
            let (_, events) = resolve_units(&s, &no_orders());
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, Event::UnitAwakened { .. }))
            );
        }
    }

    #[test]
    fn re_sleep_accepts_visible_enemies_until_a_new_enemy_arrives() {
        for kind in [UnitKind::Settler, UnitKind::Spearman] {
            let mut s = game(&[0, 1]);
            let sleeper = unit(&mut s, 0, kind, hex(4, 4));
            let enemy = unit(&mut s, 1, UnitKind::Spearman, hex(7, 4));
            let new_enemy = unit(&mut s, 1, UnitKind::Spearman, hex(4, 7));
            let mut plan = orders(
                0,
                vec![Order::SleepUnit {
                    unit: sleeper,
                    until_healed: false,
                }],
            );
            plan.insert(
                PlayerId(1),
                vec![Order::MoveUnit {
                    unit: enemy,
                    to: hex(6, 4),
                }],
            );
            let (mut s, events) = resolve_units(&s, &plan);
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::UnitAwakened { .. }))
                    .count(),
                1
            );
            for turn in 0..3 {
                let plan = if turn == 0 {
                    orders(
                        0,
                        vec![Order::SleepUnit {
                            unit: sleeper,
                            until_healed: false,
                        }],
                    )
                } else {
                    no_orders()
                };
                let (next, events) = resolve_units(&s, &plan);
                assert_eq!(
                    next.units[&sleeper].standing_order,
                    Some(StandingOrder::Sleep)
                );
                assert!(!next.units[&sleeper].needs_orders);
                assert!(
                    !events
                        .iter()
                        .any(|e| matches!(e, Event::UnitAwakened { .. }))
                );
                s = serde_json::from_str(&serde_json::to_string(&next).unwrap()).unwrap();
            }
            let (s, events) = resolve_units(
                &s,
                &orders(
                    1,
                    vec![Order::MoveUnit {
                        unit: new_enemy,
                        to: hex(4, 6),
                    }],
                ),
            );
            assert_eq!(s.units[&sleeper].standing_order, None);
            assert!(s.units[&sleeper].needs_orders);
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::UnitAwakened { .. }))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn a_visible_enemy_approaching_into_range_wakes_sleep_once() {
        let mut s = game(&[0, 0, 1]);
        let sleeper = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        unit(&mut s, 1, UnitKind::Scout, hex(8, 4));
        let enemy = unit(&mut s, 2, UnitKind::Spearman, hex(7, 4));
        assert!(
            player_view(&s, PlayerId(0))
                .units
                .iter()
                .any(|u| u.id == enemy)
        );
        // Asleep with the enemy in sight but out of reach: nothing to report.
        let (s, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::SleepUnit {
                    unit: sleeper,
                    until_healed: false,
                }],
            ),
        );
        assert_eq!(s.units[&sleeper].standing_order, Some(StandingOrder::Sleep));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::UnitAwakened { .. }))
        );
        // Walking up to the sleeper wakes it, once.
        let (s, events) = resolve_units(
            &s,
            &orders(
                2,
                vec![Order::MoveUnit {
                    unit: enemy,
                    to: hex(6, 4),
                }],
            ),
        );
        assert_eq!(s.units[&enemy].pos, hex(6, 4));
        assert_eq!(s.units[&sleeper].standing_order, None);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::UnitAwakened { .. }))
                .count(),
            1
        );
        // Re-sleeping beside it holds while it stays.
        let (s, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::SleepUnit {
                    unit: sleeper,
                    until_healed: false,
                }],
            ),
        );
        let (s, more) = resolve_units(&s, &no_orders());
        assert_eq!(s.units[&sleeper].standing_order, Some(StandingOrder::Sleep));
        assert!(
            !events
                .iter()
                .chain(&more)
                .any(|e| matches!(e, Event::UnitAwakened { .. }))
        );
    }

    #[test]
    fn sleeping_with_a_visible_enemy_waits_for_it_to_leave_and_reenter_sight() {
        let mut s = game(&[0, 1]);
        let sleeper = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let enemy = unit(&mut s, 1, UnitKind::Spearman, hex(6, 4));
        let (s, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::SleepUnit {
                    unit: sleeper,
                    until_healed: false,
                }],
            ),
        );
        assert_eq!(s.units[&sleeper].standing_order, Some(StandingOrder::Sleep));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::UnitAwakened { .. }))
        );
        let (s, events) = resolve_units(
            &s,
            &orders(
                1,
                vec![Order::MoveUnit {
                    unit: enemy,
                    to: hex(8, 4),
                }],
            ),
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::UnitAwakened { .. }))
        );
        let (s, events) = resolve_units(
            &s,
            &orders(
                1,
                vec![Order::MoveUnit {
                    unit: enemy,
                    to: hex(6, 4),
                }],
            ),
        );
        assert_eq!(s.units[&sleeper].standing_order, None);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::UnitAwakened { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn waking_uses_current_shared_sight_and_the_sleepers_movement_radius() {
        let mut s = game(&[0, 0, 1]);
        let sleeper = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        let enemy = unit(&mut s, 2, UnitKind::Spearman, hex(7, 4));
        s.units.get_mut(&sleeper).unwrap().standing_order = Some(StandingOrder::Sleep);
        // Scout movement is 3, but its sight is 2. Explored fog is not live sight.
        for i in 0..s.map.len() {
            s.player_mut(PlayerId(0)).explored.set(i);
        }
        let mut events = Vec::new();
        let mut previous = sleeping_threats(&s);
        wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert!(events.is_empty());
        let ally = unit(&mut s, 1, UnitKind::Spearman, hex(8, 4));
        // Shared sight now reveals the enemy exactly three hexes away.
        wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert_eq!(events.len(), 1);
        assert!(s.units[&sleeper].needs_orders);
        // A Spearman's radius is only 2 even though the same enemy remains visible.
        let u = s.units.get_mut(&sleeper).unwrap();
        u.kind = UnitKind::Spearman;
        u.standing_order = Some(StandingOrder::Sleep);
        events.clear();
        wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert!(events.is_empty());
        s.units.get_mut(&enemy).unwrap().pos = hex(6, 4);
        s.units.get_mut(&enemy).unwrap().kind = UnitKind::Settler;
        s.units.get_mut(&ally).unwrap().pos = hex(5, 4);
        wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert!(
            events.is_empty(),
            "civilian enemies and friendly armies do not wake sleepers"
        );
    }

    #[test]
    fn only_armies_of_teams_at_war_wake_sleepers() {
        let mut s = game(&[0, 1]);
        let sleeper = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let army = unit(&mut s, 1, UnitKind::Spearman, hex(8, 4));
        s.units.get_mut(&sleeper).unwrap().standing_order = Some(StandingOrder::Sleep);
        s.diplomacy.as_mut().unwrap().relations[0].state = crate::diplomacy::RelationState::Peace;
        let mut previous = sleeping_threats(&s);
        s.units.get_mut(&army).unwrap().pos = hex(5, 4);
        let mut events = Vec::new();
        wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert!(events.is_empty(), "a visible army at Peace is not a threat");
        assert_eq!(s.units[&sleeper].standing_order, Some(StandingOrder::Sleep));
        // A declaration turns the army already in range into a new threat.
        s.diplomacy.as_mut().unwrap().relations[0].state = crate::diplomacy::RelationState::War;
        wake_threatened_sleepers(&mut s, &mut previous, &mut events);
        assert_eq!(events.len(), 1);
        assert!(s.units[&sleeper].needs_orders);
    }

    #[test]
    fn passing_enemy_wakes_a_sleeper_even_if_it_ends_out_of_sight() {
        let mut s = game(&[0, 1]);
        let at = Hex::new(4, 4);
        let sleeper = unit(&mut s, 0, UnitKind::Spearman, at);
        let enemy = unit(&mut s, 1, UnitKind::Scout, Hex::new(7, 1));
        s.units.get_mut(&sleeper).unwrap().standing_order = Some(StandingOrder::Sleep);
        let path = vec![Hex::new(6, 2), Hex::new(7, 2), Hex::new(8, 2)];
        let (s, events) = resolve_units(
            &s,
            &orders(
                1,
                vec![Order::MoveUnitPath {
                    unit: enemy,
                    to: Hex::new(8, 2),
                    path,
                    stop_on_block: false,
                }],
            ),
        );
        assert_eq!(s.units[&enemy].pos, Hex::new(8, 2));
        assert!(s.units[&enemy].pos.distance(at) > 2);
        assert_eq!(s.units[&sleeper].standing_order, None);
        assert!(s.units[&sleeper].needs_orders);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::UnitAwakened { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn sparta_regiments_and_scouts_rest_heal_at_the_normal_rate() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let soldier = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let scout = unit(&mut s, 0, UnitKind::Scout, hex(4, 5));
        for id in [soldier, scout] {
            s.units.get_mut(&id).unwrap().hp = 75;
        }
        let (s, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![
                    Order::SleepUnit {
                        unit: soldier,
                        until_healed: false,
                    },
                    Order::SleepUnit {
                        unit: scout,
                        until_healed: false,
                    },
                ],
            ),
        );
        // Both stand in the field and recover at the normal rate.
        assert_eq!(s.units[&soldier].hp, 75 + data::HEAL_FIELD);
        assert_eq!(s.units[&scout].hp, 75 + data::HEAL_FIELD);
        for id in [soldier, scout] {
            assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Sleep));
        }
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
    }

    #[test]
    fn sleep_heals_through_saves_and_stays_asleep_at_full_hp() {
        let mut s = game(&[0, 1]);
        let id = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&id).unwrap().hp = 75;
        goto(&mut s, id, hex(8, 4));
        let (s, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::SleepUnit {
                    unit: id,
                    until_healed: false,
                }],
            ),
        );
        assert_eq!(s.units[&id].hp, 85);
        assert_eq!(s.units[&id].pos, hex(4, 4));
        assert_eq!(s.units[&id].goto, None);
        assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Sleep));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::UnitAwakened { .. }))
        );
        let saved = serde_json::to_string(&s).unwrap();
        let restored: GameState = serde_json::from_str(&saved).unwrap();
        assert_eq!(s, restored);
        let (s, _) = resolve_units(&restored, &no_orders());
        assert_eq!(s.units[&id].hp, 95);
        let (s, events) = resolve_units(&s, &no_orders());
        let (s, more) = resolve_units(&s, &no_orders());
        assert_eq!(s.units[&id].hp, UNIT_HP);
        assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Sleep));
        assert!(!s.units[&id].needs_orders);
        assert!(
            !events
                .iter()
                .chain(&more)
                .any(|e| matches!(e, Event::UnitAwakened { .. }))
        );
    }

    #[test]
    fn combat_delays_the_healing_of_a_sleeper() {
        let mut s = game(&[0, 1]);
        let id = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let u = s.units.get_mut(&id).unwrap();
        u.hp = 95;
        u.acted = true;
        u.standing_order = Some(StandingOrder::Sleep);
        heal_and_fortify(&mut s);
        assert_eq!(s.units[&id].hp, 95);
        assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Sleep));
    }

    fn heal_order(unit: UnitId) -> Order {
        Order::SleepUnit {
            unit,
            until_healed: true,
        }
    }

    #[test]
    fn a_heal_order_rests_then_wakes_once_at_full_hp_with_a_notice() {
        let mut s = game(&[0, 1]);
        let id = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&id).unwrap().hp = 75;
        goto(&mut s, id, hex(8, 4));
        let (s, events) = resolve_units(&s, &orders(0, vec![heal_order(id)]));
        assert!(!events.iter().any(|e| matches!(e, Event::UnitHealed { .. })));
        assert_eq!(s.units[&id].hp, 85, "rests exactly like Sleep");
        assert_eq!(s.units[&id].pos, hex(4, 4));
        assert_eq!(s.units[&id].goto, None);
        assert_eq!(s.units[&id].fortify, FORTIFY_PER_TURN);
        assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Heal));
        assert!(!s.units[&id].needs_orders);
        // The order survives a save, and the unit stays out of the order cycle while it heals.
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(s, restored);
        let (s, events) = resolve_units(&restored, &no_orders());
        assert_eq!(s.units[&id].hp, 95);
        assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Heal));
        assert!(!s.units[&id].needs_orders);
        assert!(!events.iter().any(|e| matches!(e, Event::UnitHealed { .. })));
        // The turn it is full, the order ends and the unit asks for orders, once.
        let (s, events) = resolve_units(&s, &no_orders());
        assert_eq!(s.units[&id].hp, UNIT_HP);
        assert_eq!(s.units[&id].standing_order, None);
        assert!(s.units[&id].needs_orders);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::UnitHealed { .. }))
                .collect::<Vec<_>>(),
            vec![&Event::UnitHealed {
                unit: id,
                owner: PlayerId(0),
                unit_kind: UnitKind::Spearman,
                at: hex(4, 4),
            }]
        );
        assert_eq!(s.units[&id].fortify, FORTIFY_MAX);
        let (_, events) = resolve_units(&s, &no_orders());
        assert!(!events.iter().any(|e| matches!(e, Event::UnitHealed { .. })));
    }

    #[test]
    fn a_healing_unit_wakes_early_for_a_new_enemy_like_a_sleeper() {
        let mut s = game(&[0, 1]);
        let id = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&id).unwrap().hp = 40;
        let enemy = unit(&mut s, 1, UnitKind::Spearman, hex(7, 4));
        let (s, _) = resolve_units(&s, &orders(0, vec![heal_order(id)]));
        assert_eq!(s.units[&id].standing_order, Some(StandingOrder::Heal));
        let (s, events) = resolve_units(
            &s,
            &orders(
                1,
                vec![Order::MoveUnit {
                    unit: enemy,
                    to: hex(6, 4),
                }],
            ),
        );
        assert_eq!(s.units[&id].standing_order, None);
        assert!(s.units[&id].needs_orders);
        assert!(s.units[&id].hp < UNIT_HP, "woken early, not healed");
        assert!(events.contains(&Event::UnitAwakened {
            unit: id,
            owner: PlayerId(0),
            unit_kind: UnitKind::Spearman,
            at: hex(4, 4),
        }));
        assert!(!events.iter().any(|e| matches!(e, Event::UnitHealed { .. })));
    }

    #[test]
    fn a_heal_order_at_full_hp_is_refused_with_a_reason() {
        let mut s = game(&[0, 1]);
        let full = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        let hurt = unit(&mut s, 0, UnitKind::Spearman, hex(6, 4));
        s.units.get_mut(&hurt).unwrap().hp = UNIT_HP - 1;
        let errs =
            super::super::validate_orders(&s, PlayerId(0), &[heal_order(full), heal_order(hurt)]);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].index, 0);
        assert!(
            errs[0].message.contains("full health"),
            "{}",
            errs[0].message
        );
        // A refused order that reaches resolution changes nothing and does not stand.
        let (next, events) = resolve_units(&s, &orders(0, vec![heal_order(full)]));
        assert_eq!(next.units[&full].standing_order, None);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
    }

    #[test]
    fn heal_orders_leave_tapes_and_saves_unchanged() {
        // The flag is absent from plain Sleep, so recorded tapes serialize byte for byte as before.
        let sleep = Order::SleepUnit {
            unit: UnitId(7),
            until_healed: false,
        };
        assert_eq!(
            serde_json::to_string(&sleep).unwrap(),
            r#"{"type":"SleepUnit","unit":7}"#
        );
        let heal = serde_json::to_string(&heal_order(UnitId(7))).unwrap();
        assert_eq!(heal, r#"{"type":"SleepUnit","unit":7,"until_healed":true}"#);
        assert_eq!(
            serde_json::from_str::<Order>(&heal).unwrap(),
            heal_order(UnitId(7))
        );
        // An old tape's HealUnit still reads as a plain Sleep, never as the new heal.
        let old: Order = serde_json::from_str(r#"{"type":"HealUnit","unit":7}"#).unwrap();
        assert_eq!(old, sleep);
        // A unit on Sleep or Heal round-trips through the state's JSON.
        let mut s = game(&[0, 1]);
        let id = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        s.units.get_mut(&id).unwrap().standing_order = Some(StandingOrder::Heal);
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(
            restored.units[&id].standing_order,
            Some(StandingOrder::Heal)
        );
    }

    #[test]
    fn a_sleeping_ranged_unit_holds_fire() {
        let mut s = game(&[0, 1]);
        let archer = unit(&mut s, 0, UnitKind::Archer, hex(4, 4));
        let enemy = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
        let (s, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::SleepUnit {
                    unit: archer,
                    until_healed: false,
                }],
            ),
        );
        assert_eq!(s.units[&enemy].hp, UNIT_HP);
        assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
    }

    #[test]
    fn recorded_idle_order_names_still_load() {
        let parse = |name: &str| -> Order {
            serde_json::from_str(&format!(r#"{{"type":"{name}","unit":7}}"#)).unwrap()
        };
        for name in ["HoldUnit", "SleepUnit", "HealUnit"] {
            assert_eq!(
                parse(name),
                Order::SleepUnit {
                    unit: UnitId(7),
                    until_healed: false
                },
                "{name}"
            );
        }
        assert_eq!(parse("SkipUnit"), Order::SkipUnit { unit: UnitId(7) });
        for name in ["Fortify", "Sleep"] {
            let o: StandingOrder = serde_json::from_str(&format!("\"{name}\"")).unwrap();
            assert_eq!(o, StandingOrder::Sleep, "{name}");
        }
        let heal: StandingOrder = serde_json::from_str("\"Heal\"").unwrap();
        assert_eq!(heal, StandingOrder::Heal);
        assert_eq!(
            serde_json::to_string(&Order::SleepUnit {
                unit: UnitId(7),
                until_healed: false
            })
            .unwrap(),
            r#"{"type":"SleepUnit","unit":7}"#
        );
    }

    #[test]
    fn other_orders_cancel_sleep_and_new_state_is_private_and_defaulted() {
        let mut s = game(&[0, 0, 1]);
        let id = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        let legacy = serde_json::to_value(&s.units[&id]).unwrap();
        assert!(legacy.get("standing_order").is_none());
        let restored: crate::state::Unit = serde_json::from_value(legacy).unwrap();
        assert_eq!(restored.standing_order, None);
        let u = s.units.get_mut(&id).unwrap();
        u.hp = 50;
        u.standing_order = Some(StandingOrder::Sleep);
        crate::visibility::update_explored(&mut s);
        assert_eq!(
            player_view(&s, PlayerId(0)).units[0].standing_order,
            Some(StandingOrder::Sleep)
        );
        assert_eq!(player_view(&s, PlayerId(1)).units[0].standing_order, None);
        for order in [
            Order::MoveUnit {
                unit: id,
                to: hex(5, 4),
            },
            Order::MoveUnitPath {
                unit: id,
                to: hex(5, 4),
                path: vec![hex(5, 4)],
                stop_on_block: false,
            },
            Order::SkipUnit { unit: id },
            Order::FoundCity { unit: id },
            Order::DisbandUnit { unit: id },
            Order::UpgradeUnit { unit: id },
            Order::RangedAttack {
                unit: id,
                target: hex(5, 4),
            },
            auto(id, vec![]),
        ] {
            let (s, _) = resolve_units(&s, &orders(0, vec![order]));
            assert!(
                s.units
                    .get(&id)
                    .is_none_or(|u| u.standing_order != Some(StandingOrder::Sleep))
            );
        }
    }

    fn auto(unit: UnitId, path: Vec<Hex>) -> Order {
        Order::SetAutomation {
            unit,
            automation: Some(Automation::Explore),
            path,
            stop_reason: None,
        }
    }

    #[test]
    fn automation_persists_but_never_repeats_a_route_without_client_orders() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        crate::visibility::update_explored(&mut s);
        let to = hex(5, 4);
        let os = orders(0, vec![auto(u, vec![to])]);
        assert!(super::super::validate_orders(&s, PlayerId(0), &os[&PlayerId(0)]).is_empty());
        let (s, _) = resolve_units(&s, &os);
        assert_eq!(s.units[&u].automation, Some(Automation::Explore));
        assert_eq!(s.units[&u].pos, to);
        assert_eq!(s.units[&u].goto, None);
        let (s, _) = resolve_units(&s, &BTreeMap::new());
        assert_eq!(s.units[&u].pos, to);
        assert_eq!(s.units[&u].automation, Some(Automation::Explore));
    }

    #[test]
    fn every_land_military_unit_and_scout_accepts_auto_explore_but_civilians_do_not() {
        for kind in UnitKind::ALL {
            let mut s = game(&[0, 1]);
            let u = unit(&mut s, 0, kind, hex(4, 4));
            crate::visibility::update_explored(&mut s);
            let os = orders(0, vec![auto(u, vec![])]);
            let errs = super::super::validate_orders(&s, PlayerId(0), &os[&PlayerId(0)]);
            let accepts = data::unit_stats(kind).military && !data::is_aircraft(kind)
                || kind == UnitKind::Scout;
            assert_eq!(errs.is_empty(), accepts, "{kind:?}: {errs:?}");
            assert_eq!(data::can_auto_explore(kind), accepts, "{kind:?}");
        }
        let mut s = game(&[0, 1]);
        let soldier = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        crate::visibility::update_explored(&mut s);
        let to = hex(5, 4);
        let (s, _) = resolve_units(&s, &orders(0, vec![auto(soldier, vec![to])]));
        assert_eq!(s.units[&soldier].automation, Some(Automation::Explore));
        assert_eq!(s.units[&soldier].pos, to);
    }

    #[test]
    fn automated_rest_preserves_normal_healing_and_fortification() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        let scout = s.units.get_mut(&u).unwrap();
        scout.hp = 70;
        scout.fortify = FORTIFY_PER_TURN;
        scout.automation = Some(Automation::Explore);
        let (s, _) = resolve_units(&s, &orders(0, vec![auto(u, vec![])]));
        assert!(s.units[&u].hp > 70);
        assert_eq!(s.units[&u].fortify, FORTIFY_MAX);
        assert_eq!(s.units[&u].automation, Some(Automation::Explore));
    }

    #[test]
    fn every_other_unit_order_clears_automation() {
        let mut base = game(&[0, 1]);
        let u = unit(&mut base, 0, UnitKind::Scout, hex(4, 4));
        base.units.get_mut(&u).unwrap().automation = Some(Automation::Explore);
        for order in [
            Order::MoveUnit {
                unit: u,
                to: hex(5, 4),
            },
            Order::SleepUnit {
                unit: u,
                until_healed: false,
            },
            Order::SkipUnit { unit: u },
            Order::FoundCity { unit: u },
            Order::DisbandUnit { unit: u },
            Order::RangedAttack {
                unit: u,
                target: hex(5, 4),
            },
            Order::UpgradeUnit { unit: u },
        ] {
            let (s, _) = resolve_units(&base, &orders(0, vec![order]));
            assert!(s.units.get(&u).is_none_or(|u| u.automation.is_none()));
        }
    }

    #[test]
    fn automation_serde_default_and_owner_only_view() {
        let mut s = game(&[0, 0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        let old = serde_json::to_value(&s.units[&u]).unwrap();
        assert!(old.get("automation").is_none(), "legacy hashes stay stable");
        assert!(
            old.get("needs_orders").is_none(),
            "legacy hashes stay stable"
        );
        let restored: crate::state::Unit = serde_json::from_value(old).unwrap();
        assert_eq!(restored.automation, None);
        assert!(!restored.needs_orders);
        s.units.get_mut(&u).unwrap().automation = Some(Automation::Explore);
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<GameState>(&json).unwrap(), s);
        crate::visibility::update_explored(&mut s);
        assert_eq!(
            player_view(&s, PlayerId(0)).units[0].automation,
            Some(Automation::Explore)
        );
        assert_eq!(player_view(&s, PlayerId(1)).units[0].automation, None);
        s.units.get_mut(&u).unwrap().needs_orders = true;
        assert!(player_view(&s, PlayerId(0)).units[0].needs_orders);
        assert!(!player_view(&s, PlayerId(1)).units[0].needs_orders);
    }

    #[test]
    fn empty_automation_stop_preserves_fortification_and_requests_orders() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        s.units.get_mut(&u).unwrap().automation = Some(Automation::Explore);
        s.units.get_mut(&u).unwrap().fortify = FORTIFY_PER_TURN;
        goto(&mut s, u, hex(8, 4));
        let stop = Order::SetAutomation {
            unit: u,
            automation: None,
            path: vec![],
            stop_reason: Some(ExploreStopReason::Complete),
        };
        let (s, events) = resolve_units(&s, &orders(0, vec![stop]));
        assert_eq!(s.units[&u].automation, None);
        assert_eq!(s.units[&u].goto, None);
        assert_eq!(s.units[&u].pos, hex(4, 4));
        assert!(!s.units[&u].acted);
        assert_eq!(s.units[&u].fortify, FORTIFY_MAX);
        assert!(s.units[&u].needs_orders);
        assert!(player_view(&s, PlayerId(0)).units[0].needs_orders);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::AutomationStopped {
                reason: ExploreStopReason::Complete,
                ..
            }
        )));

        let saved = serde_json::to_string(&s).unwrap();
        let restored = serde_json::from_str::<GameState>(&saved).unwrap();
        assert_eq!(restored, s);
        let (s, _) = resolve_units(&restored, &no_orders());
        assert_eq!(s.units[&u].fortify, FORTIFY_MAX);
        assert!(s.units[&u].needs_orders, "stays awake until ordered again");
        let (rejected, _) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::MoveUnit {
                    unit: u,
                    to: hex(-1, -1),
                }],
            ),
        );
        assert!(
            rejected.units[&u].needs_orders,
            "a rejected order is not a decision"
        );
        for order in [
            Order::SleepUnit {
                unit: u,
                until_healed: false,
            },
            Order::MoveUnit {
                unit: u,
                to: hex(5, 4),
            },
            auto(u, vec![]),
        ] {
            let (next, _) = resolve_units(&s, &orders(0, vec![order]));
            assert!(!next.units[&u].needs_orders);
        }
    }

    #[test]
    fn blocked_automated_route_stops_instead_of_becoming_a_persistent_goto() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        unit(&mut s, 0, UnitKind::Spearman, hex(5, 4));
        crate::visibility::update_explored(&mut s);
        let (s, events) = resolve_units(&s, &orders(0, vec![auto(u, vec![hex(5, 4)])]));
        assert_eq!(s.units[&u].automation, None);
        assert_eq!(s.units[&u].goto, None);
        assert_eq!(s.units[&u].fortify, FORTIFY_PER_TURN);
        assert!(s.units[&u].needs_orders);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::AutomationStopped {
                reason: ExploreStopReason::Blocked,
                ..
            }
        )));
    }

    #[test]
    fn automation_routes_are_validated_from_knowledge_and_cannot_teleport() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(4, 4));
        crate::visibility::update_explored(&mut s);
        for path in [
            vec![hex(7, 4)],
            vec![hex(5, 4), hex(4, 4)],
            vec![hex(5, 4), hex(6, 4), hex(7, 4), hex(8, 4)],
        ] {
            assert!(!super::super::validate_orders(&s, PlayerId(0), &[auto(u, path)]).is_empty());
        }
        assert!(!super::super::validate_orders(&s, PlayerId(1), &[auto(u, vec![])]).is_empty());
    }

    #[test]
    fn automatic_city_fire_is_a_saved_order_and_never_moves_or_spends_gold() {
        let mut s = game(&[0, 1]);
        for relation in &mut s.diplomacy.as_mut().unwrap().relations {
            relation.state = crate::diplomacy::RelationState::War;
        }
        let pos = hex(4, 4);
        city(&mut s, 0, pos, 2);
        let bow = unit(&mut s, 0, UnitKind::Archer, pos);
        let target = hex(5, 4);
        unit(&mut s, 1, UnitKind::Spearman, target);
        s.player_mut(PlayerId(0)).gold = 100;
        let order = Order::SetAutoFire {
            unit: bow,
            enabled: true,
            target: Some(target),
        };
        assert!(crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&order)).is_empty());
        let (next, events) = resolve_units(&s, &orders(0, vec![order]));
        assert_eq!(next.units[&bow].pos, pos);
        assert_eq!(next.player(PlayerId(0)).gold, 100);
        assert_eq!(
            next.units[&bow].standing_order,
            Some(StandingOrder::AutoFire)
        );
        assert!(!next.units[&bow].needs_orders);
        assert!(events.iter().any(|e| matches!(e, Event::Battle { attacker, ranged: true, .. } if attacker.unit() == Some(bow))));
        let (held, _) = resolve_units(&next, &no_orders());
        assert_eq!(
            held.units[&bow].standing_order,
            Some(StandingOrder::AutoFire)
        );
        assert!(!held.units[&bow].needs_orders);
    }

    #[test]
    fn upgrade_order_swaps_kind_charges_gold_and_pins_unit() {
        let mut s = game(&[0, 1]);
        city(&mut s, 0, hex(4, 4), 2);
        let u = unit(&mut s, 0, UnitKind::Phalangite, hex(4, 5));
        s.units.get_mut(&u).unwrap().hp = 70;
        goto(&mut s, u, hex(9, 5));
        s.player_mut(PlayerId(0)).techs.insert(TechId::Feudalism);
        s.player_mut(PlayerId(0)).gold = 120;
        // Phalangite→Pikeman also uses 1 Iron supply: an Iron deposit inside the border.
        s.player_mut(PlayerId(0))
            .techs
            .extend([TechId::BronzeWorking, TechId::IronWorking]);
        s.map.get_mut(hex(4, 4).neighbors()[0]).unwrap().resource = Some(crate::Resource::Iron);
        let (next, events) = resolve_units(&s, &orders(0, vec![Order::UpgradeUnit { unit: u }]));
        let up = &next.units[&u];
        assert_eq!(up.kind, UnitKind::Pikeman);
        assert_eq!(up.pos, hex(4, 5), "acts next turn: no movement");
        assert_eq!(next.player(PlayerId(0)).gold, 120 - 84);
        assert_eq!(
            up.hp,
            70 + HEAL_TERRITORY,
            "keeps HP (then heals: it did not act)"
        );
        assert!(events.iter().any(|e| matches!(
            e,
            Event::UnitUpgraded {
                gold: 84,
                to: UnitKind::Pikeman,
                ..
            }
        )));
        assert_eq!(upgrade_cost(UnitKind::Phalangite), Some(84));
        assert_eq!(upgrade_cost(UnitKind::Spearman), Some(60));
        assert_eq!(upgrade_cost(UnitKind::Scout), None);
    }

    #[test]
    fn one_upgrade_reaches_newest_researched_form_for_sum_of_step_costs() {
        let mut s = game(&[0, 1]);
        city(&mut s, 0, hex(4, 4), 2);
        let u = unit(&mut s, 0, UnitKind::Spearman, hex(4, 5));
        s.player_mut(PlayerId(0)).techs.extend([
            TechId::Mathematics,
            TechId::Feudalism,
            TechId::IronWorking,
        ]);
        s.player_mut(PlayerId(0)).gold = 200;
        s.map.get_mut(hex(4, 4).neighbors()[0]).unwrap().resource = Some(crate::Resource::Iron);
        let (to, steps, price) = upgrade_plan(&s, PlayerId(0), UnitKind::Spearman).unwrap();
        assert_eq!(to, UnitKind::Pikeman);
        assert_eq!(steps, [UnitKind::Phalangite, UnitKind::Pikeman]);
        assert_eq!(price.total, 60 + 84);
        assert_eq!(
            check_upgrade(&s, PlayerId(0), u, 200),
            Ok((to, price.total))
        );
        let (next, events) = resolve_units(&s, &orders(0, vec![Order::UpgradeUnit { unit: u }]));
        assert_eq!(next.units[&u].kind, UnitKind::Pikeman);
        assert_eq!(next.player(PlayerId(0)).gold, 200 - 144);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::UnitUpgraded {
                to: UnitKind::Pikeman,
                gold: 144,
                ..
            }
        )));
    }

    #[test]
    fn upgrade_without_gold_fails_at_resolution() {
        let mut s = game(&[0, 1]);
        city(&mut s, 0, hex(4, 4), 2);
        let u = unit(&mut s, 0, UnitKind::Phalangite, hex(4, 5));
        s.player_mut(PlayerId(0)).techs.insert(TechId::Feudalism);
        let (next, events) = resolve_units(&s, &orders(0, vec![Order::UpgradeUnit { unit: u }]));
        assert_eq!(next.units[&u].kind, UnitKind::Phalangite);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
    }

    #[test]
    fn sleep_disband_and_duplicate_orders() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(2, 2));
        let b = unit(&mut s, 0, UnitKind::Scout, hex(5, 5));
        let foreign = unit(&mut s, 1, UnitKind::Scout, hex(9, 9));
        goto(&mut s, a, hex(8, 2));
        let os = orders(
            0,
            vec![
                Order::SleepUnit {
                    unit: a,
                    until_healed: false,
                },
                Order::MoveUnit {
                    unit: a,
                    to: hex(8, 2),
                },
                Order::DisbandUnit { unit: b },
                Order::DisbandUnit { unit: foreign },
            ],
        );
        let (next, events) = resolve_units(&s, &os);
        assert_eq!(next.units[&a].pos, hex(2, 2));
        assert_eq!(next.units[&a].goto, None);
        assert!(!next.units.contains_key(&b));
        assert!(next.units.contains_key(&foreign));
        let failed = events
            .iter()
            .filter(|e| matches!(e, Event::OrderFailed { .. }))
            .count();
        assert_eq!(failed, 2, "duplicate order and foreign unit");
    }

    fn ruin_game(faction: Faction) -> (GameState, UnitId) {
        let mut s = game(&[0, 1]);
        s.players[0].faction = faction;
        city(&mut s, 0, hex(1, 1), 2);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(5, 5));
        s.map.get_mut(hex(6, 5)).unwrap().ruin = true;
        goto(&mut s, u, hex(6, 5));
        (s, u)
    }

    fn wealth(s: &GameState) -> (i32, i32, i32) {
        let p = s.player(PlayerId(0));
        let pop = s.cities_of(PlayerId(0)).map(|c| c.population).sum();
        (p.gold, p.science_stored, pop)
    }

    #[test]
    fn ruins_reward_first_entrant_and_the_byzantium_gets_the_ordinary_reward() {
        for turn in 1..12 {
            let (mut s, _) = ruin_game(Faction::Switzerland);
            s.turn = turn;
            let (mut r, _) = ruin_game(Faction::Byzantium);
            r.turn = turn;
            let before = wealth(&s);
            let (ns, es) = resolve_units(&s, &no_orders());
            let (nr, _) = resolve_units(&r, &no_orders());
            assert!(!ns.map.get(hex(6, 5)).unwrap().ruin, "ruin cleared");
            assert_eq!(ns.player(PlayerId(0)).ruins_explored, 1);
            assert!(es.iter().any(|e| matches!(e, Event::RuinExplored { .. })));
            let gain = |w: (i32, i32, i32)| (w.0 - before.0, w.1 - before.1, w.2 - before.2);
            let (g, sc, pop) = gain(wealth(&ns));
            assert_eq!(gain(wealth(&nr)), (g, sc, pop));
            assert!(
                [
                    (RUIN_GOLD, 0, 0),
                    (0, RUIN_SCIENCE, 0),
                    (0, 0, 1),
                    (0, 0, 0)
                ]
                .contains(&(g, sc, pop)),
                "{g} {sc} {pop}"
            );
        }
    }

    #[test]
    fn athens_ruins_off_adds_no_extra_science() {
        let science = |off: bool| {
            let (mut s, _) = ruin_game(Faction::Athens);
            if off {
                s.setup
                    .rule_experiments
                    .faction_rules_off
                    .insert(crate::experiments::FactionRule::AthensRuins);
            }
            let (next, _) = resolve_units(&s, &no_orders());
            next.player(PlayerId(0)).science_stored
        };
        assert_eq!(science(false) - science(true), 20);
    }

    #[test]
    fn invalid_founding_attempt_does_not_block_a_legal_settlement() {
        let mut s = game(&[0, 1]);
        city(&mut s, 0, Hex::new(4, 4), 1);
        let invalid = unit(&mut s, 1, UnitKind::Settler, Hex::new(7, 4));
        let legal = unit(&mut s, 0, UnitKind::Settler, Hex::new(8, 4));
        let mut submitted = orders(0, vec![Order::FoundCity { unit: legal }]);
        let (mut alone, _) = crate::rules::resolve_turn(&s, &submitted);
        assert!(alone.city_at(Hex::new(8, 4)).is_some());

        submitted.insert(PlayerId(1), vec![Order::FoundCity { unit: invalid }]);
        let (together, events) = crate::rules::resolve_turn(&s, &submitted);
        // The only extra state is the rejected player's memory of its own failed site.
        alone
            .player_mut(PlayerId(1))
            .failed_city_sites
            .insert(Hex::new(7, 4));
        assert_eq!(
            together.hash(),
            alone.hash(),
            "an illegal attempt cannot cancel a legal one"
        );
        assert!(together.units.contains_key(&invalid));
        assert!(!together.units.contains_key(&legal));
        let failures: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::OrderFailed { player, reason } => Some((*player, reason)),
                _ => None,
            })
            .collect();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, PlayerId(1));
        assert!(failures[0].1.contains("Too close to"));
        assert!(
            failures[0]
                .1
                .ends_with("(3 tiles away): cities must be at least 4 tiles apart"),
            "{}",
            failures[0].1
        );
        // One hex is singular.
        let mut s = game(&[0]);
        city(&mut s, 0, Hex::new(4, 4), 1);
        let near = unit(&mut s, 0, UnitKind::Settler, Hex::new(5, 4));
        let reason = crate::rules::cities::check_found_city(&s, near).unwrap_err();
        assert!(
            reason.ends_with("(1 tile away): cities must be at least 4 tiles apart"),
            "{reason}"
        );
    }

    #[test]
    fn found_city_conflicts_within_three_hexes_all_fail() {
        let mut s = game(&[0, 1, 2]);
        let a = unit(&mut s, 0, UnitKind::Settler, hex(2, 2));
        let b = unit(&mut s, 1, UnitKind::Settler, hex(4, 3));
        let c = unit(&mut s, 2, UnitKind::Settler, hex(10, 8));
        let d = unit(&mut s, 2, UnitKind::Spearman, hex(10, 2));
        let mut events = Vec::new();
        let found = [
            (PlayerId(0), a),
            (PlayerId(1), b),
            (PlayerId(2), c),
            (PlayerId(2), d),
        ];
        let ok = found_city_candidates(&s, &found, &mut events);
        assert_eq!(ok, vec![c]);
        assert_eq!(events.len(), 3, "a and b conflict, d is no settler");
        // Settlers are not consumed.
        assert!(s.units.contains_key(&a) && s.units.contains_key(&b));
    }

    #[test]
    fn found_order_stops_the_settler() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Settler, hex(2, 2));
        goto(&mut s, a, hex(8, 2));
        let (next, _) = resolve_units(&s, &orders(0, vec![Order::FoundCity { unit: a }]));
        assert_eq!(next.units[&a].pos, hex(2, 2));
    }

    #[test]
    fn elimination_removes_empires_without_cities_or_settlers() {
        let mut s = game(&[0, 1, 2]);
        city(&mut s, 0, hex(2, 2), 1);
        unit(&mut s, 1, UnitKind::Settler, hex(6, 6));
        let doomed = unit(&mut s, 2, UnitKind::Spearman, hex(9, 9));
        let mut events = Vec::new();
        check_elimination(&mut s, &mut events);
        assert!(s.player(PlayerId(0)).alive && s.player(PlayerId(1)).alive);
        assert!(!s.player(PlayerId(2)).alive);
        assert!(!s.units.contains_key(&doomed));
        assert_eq!(
            events,
            vec![Event::PlayerEliminated {
                player: PlayerId(2)
            }]
        );
    }

    #[test]
    fn healing_depends_on_territory_and_activity() {
        let mut s = game(&[0, 1]);
        city(&mut s, 0, hex(2, 2), 1);
        let home = unit(&mut s, 0, UnitKind::Spearman, hex(2, 3));
        let field = unit(&mut s, 0, UnitKind::Spearman, hex(8, 8));
        let full = unit(&mut s, 0, UnitKind::Scout, hex(6, 6));
        let mover = unit(&mut s, 0, UnitKind::Scout, hex(9, 2));
        for u in [home, field, mover] {
            s.units.get_mut(&u).unwrap().hp = 50;
        }
        s.units.get_mut(&full).unwrap().hp = 95;
        goto(&mut s, mover, hex(9, 5));
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!(next.units[&home].hp, 70);
        assert_eq!(next.units[&field].hp, 60);
        assert_eq!(next.units[&full].hp, 100);
        assert_eq!(next.units[&mover].hp, 50);
    }

    #[test]
    fn fortify_accrues_caps_and_clears_on_move() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Spearman, hex(2, 2));
        let civ = unit(&mut s, 0, UnitKind::Settler, hex(5, 5));
        let (s1, _) = resolve_units(&s, &no_orders());
        assert_eq!(s1.units[&u].fortify, 3);
        let (s2, _) = resolve_units(&s1, &no_orders());
        assert_eq!(s2.units[&u].fortify, 6);
        let (s3, _) = resolve_units(&s2, &no_orders());
        assert_eq!(s3.units[&u].fortify, 6);
        assert_eq!(s3.units[&civ].fortify, 0, "civilians do not fortify");
        let (s4, _) = resolve_units(
            &s3,
            &orders(
                0,
                vec![Order::MoveUnit {
                    unit: u,
                    to: hex(3, 2),
                }],
            ),
        );
        assert_eq!(s4.units[&u].fortify, 0);
    }
}

#[cfg(test)]
#[path = "ruins_tests.rs"]
mod ruins_tests;
