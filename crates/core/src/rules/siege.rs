//! Siege (user decision 2026-10-22): a permanent upgrade, bought
//! with Gold in the owner's own territory like a unit upgrade (`Order::UpgradeSiege`), that makes
//! one land military unit ignore enemy Walls when it attacks a city.
//!
//! - **Who.** Any land military unit except a Scout (`data::can_have_siege`), once its owner knows
//!   Mathematics (`data::siege_tier` > 0). The Mongols only from the Medieval era.
//! - **What.** `battle::siege_cover`: the unit's attacks on a city fight as if the city had no
//!   Walls up to the tier of its owner's technology (Mathematics Walls, Machinery also Castles,
//!   Metallurgy also Star Forts). The tier follows the technology, so nothing is bought twice.
//! - **Price.** `data::siege_gold`, 160 Gold for an Ancient unit and scaled by era after that,
//!   halved by Military Engineering like the other upgrades.
//! - **Iron.** Each Siege unit uses 1 Iron supply while it exists (`data::SIEGE_USES_IRON`): the
//!   purchase needs 1 spare, and short of supply the newest users go idle and fight 25% weaker
//!   (`resources::SupplyUser::Siege`).
//! - **Kept** through unit upgrades; the unit cannot act on the turn it is bought.
//! - **Built in from the Industrial era** (designer, 2026-10-28; `data::built_in_siege`): an
//!   Industrial or later land unit ignores every Walls tier without buying anything, so it is not
//!   offered Siege, and a Siege unit that upgrades into one drops the bought flag (and its Iron).

use crate::data;
use crate::state::GameState;
use crate::types::*;

/// The Gold price of Siege for a unit of `kind`, with its breakdown.
pub fn price(state: &GameState, player: PlayerId, kind: UnitKind) -> Breakdown {
    let mut b = Breakdown::default();
    b.add("Siege, by the unit's era", data::siege_gold(kind));
    if state
        .player(player)
        .techs
        .contains(&TechId::MilitaryEngineering)
    {
        b.add("Military Engineering (50%)", b.total / 2 - b.total);
    }
    b
}

/// Why `unit` cannot be given Siege now, or its price. `gold` is the treasury left;
/// `earlier_iron` the Siege purchases and Iron-using upgrades ahead of this order in the same
/// list (`rules::validate_orders`).
pub fn check(
    state: &GameState,
    player: PlayerId,
    unit: UnitId,
    gold: i32,
    earlier_iron: i32,
) -> Result<Breakdown, String> {
    let u = state.units.get(&unit).ok_or("no such unit")?;
    let name = data::unit_name_for(state.player(player).faction, u.kind);
    if data::built_in_siege(u.kind) {
        return Err(format!(
            "{name} needs no Siege: it {}",
            data::BUILT_IN_SIEGE_NOTE
        ));
    }
    if !data::can_have_siege(u.kind) {
        return Err(format!("{name} cannot have Siege"));
    }
    if u.siege {
        return Err(format!("{name} already has Siege"));
    }
    if data::siege_tier(&state.player(player).techs) == 0 {
        return Err(format!(
            "Siege needs {}",
            data::tech_name(TechId::Mathematics)
        ));
    }
    let p = state.player(player);
    if !data::siege_available_rules(
        state.setup.rule_experiments.faction_rules_off,
        p.faction,
        state.era_of(player),
    ) {
        return Err(data::MONGOL_SIEGE_REASON.into());
    }
    if state.map.get(u.pos).and_then(|t| t.owner) != Some(player) {
        return Err("Siege is bought only in your own territory".into());
    }
    let price = price(state, player, u.kind);
    if gold < price.total {
        return Err(format!("Siege costs {} Gold, you have {gold}", price.total));
    }
    if let Some(reason) = iron_blocker(state, player, earlier_iron) {
        return Err(format!(
            "Siege: {}",
            crate::rules::units::lowercase_first(&reason)
        ));
    }
    Ok(price)
}

/// Why no Iron can be spared for one more Siege unit (`None` when Siege uses none, or one is
/// spare after `earlier` others in the same order list).
pub fn iron_blocker(state: &GameState, player: PlayerId, earlier: i32) -> Option<String> {
    let need = crate::resources::siege_need()?;
    crate::resources::shortage_after_orders(state, player, need, earlier)
}
