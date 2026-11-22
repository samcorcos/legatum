//! Occupation lock (user proposal 2026-11-09; a rule in every game since rules 31).
//!
//! The lock belongs to the city. `battle::capture_city` records the civ the city was taken from
//! (`City::occupied_from`). A military unit inside a city its owner captured cannot leave it while
//! its owner is at war with that civ; Peace, or that civ's elimination, releases it. The unit can
//! still fortify, heal and shoot. Peace or the civ's elimination also clears the record for good
//! (`clear_released`): a later war with that civ neither locks nor freezes the city (user, 2026-11-11). A locked unit is held automatically (`hold_locked_units`): it is
//! never in the order cycle, because nothing it could be asked is a move. If it dies another unit may move in, and that unit is locked too.
//! A city retaken by the civ (or team) it was taken from carries no record.
//!
//! The same condition freezes the city itself (`city_freeze`): no Production progress and no
//! purchases or hires until Peace with that civ or its elimination. Food, Gold, Science and
//! growth are untouched (unlike capture resistance, which zeroes every yield).

use crate::state::{City, GameState, Unit};
use crate::types::{PlayerId, StandingOrder, UnitId};

/// The city that holds `unit` and the civ it was taken from, while the lock applies.
pub fn lock<'a>(state: &'a GameState, unit: &Unit) -> Option<(&'a City, PlayerId)> {
    if !super::is_military(unit.kind) {
        return None;
    }
    let city = state.city_at(unit.pos).filter(|c| c.owner == unit.owner)?;
    let from = city.occupied_from?;
    (state.player(from).alive && state.at_war(unit.owner, from)).then_some((city, from))
}

/// End of turn, after diplomacy has resolved: drop the record on every city whose owner is not at
/// war with the recorded civ, or whose recorded civ is dead. Runs every turn boundary, so a Peace
/// is always seen before any later declaration can take effect. A fresh capture is at war, so it
/// is never cleared the turn it is set.
pub fn clear_released(state: &mut GameState) {
    let released: Vec<_> = state
        .cities
        .values()
        .filter(|c| {
            c.occupied_from
                .is_some_and(|from| !(state.player(from).alive && state.at_war(c.owner, from)))
        })
        .map(|c| c.id)
        .collect();
    for id in released {
        state.cities.get_mut(&id).expect("city").occupied_from = None;
    }
}

/// End of turn: every locked unit is put on a standing order and leaves the order cycle, so the lock
/// never shows up as a decision (user, 2026-11-11: "they're just automatically fortified"). A melee
/// unit fortifies (`Sleep`); a ranged unit takes `AutoFire`, which the session turns into a shot at
/// the best visible target each turn, so being locked never costs a shot. A unit already healing or
/// firing keeps its order. A route or attack target left over from before the capture is dropped.
pub fn hold_locked_units(state: &mut GameState) {
    let locked: Vec<UnitId> = state
        .units
        .values()
        .filter(|u| lock(state, u).is_some())
        .map(|u| u.id)
        .collect();
    for id in locked {
        let u = state.units.get_mut(&id).expect("locked unit");
        if u.standing_order.is_none() {
            u.standing_order = Some(if crate::data::unit_stats(u.kind).ranged > 0 {
                StandingOrder::AutoFire
            } else {
                StandingOrder::Sleep
            });
        }
        u.goto = None;
        u.path.clear();
        u.path_stop_on_block = false;
        u.attack_target = None;
        u.needs_orders = false;
    }
}

/// The civ whose war freezes `city`'s Production, purchases and hires.
pub fn city_freeze(state: &GameState, city: &City) -> Option<PlayerId> {
    let from = city.occupied_from?;
    (state.player(from).alive && state.at_war(city.owner, from)).then_some(from)
}

/// The refusal for a purchase or hire in a frozen city.
pub fn city_refusal(state: &GameState, city: &City) -> Option<String> {
    city_freeze(state, city).map(|from| {
        format!(
            "Occupied: {} builds and buys nothing until Peace with {}",
            city.name,
            crate::data::faction_name(state.player(from).faction)
        )
    })
}

/// The refusal for an order that would take a locked unit out of its city.
pub fn refusal(state: &GameState, unit: &Unit) -> Option<String> {
    lock(state, unit).map(|(city, from)| {
        format!(
            "{} is holding {} down and cannot leave until Peace with {}",
            crate::data::unit_name_for(state.made_by(unit), unit.kind),
            city.name,
            crate::data::faction_name(state.player(from).faction)
        )
    })
}

/// The civ a military unit of `me` would be locked against by ending a move in `city`: its own
/// captured city at war with the recorded civ, or a hostile city it would capture (none for a
/// city its own team is retaking, as in `battle::capture_city`). For the views and forecasts.
pub fn entering_lock(state: &GameState, me: PlayerId, city: &City) -> Option<PlayerId> {
    if city.owner == me {
        let from = city.occupied_from?;
        return (state.player(from).alive && state.at_war(me, from)).then_some(from);
    }
    (!city.owner.is_barbarian()
        && state.at_war(me, city.owner)
        && city
            .occupied_from
            .is_none_or(|prev| state.team_of(me) != state.team_of(prev)))
    .then_some(city.owner)
}
