//! Mercenary Companies (Byzantium, the Byzantium): the rule break that replaced bought peace.
//!
//! Byzantium's own army is capped by thin levies (`data::levy_cap`), so its field army is hired.
//! It may order `HireCompany { city, unit, seller }` in a city of its own: any armed unit that a
//! met empire it is not at War with (and is not on a team with) can build right now. The Gold
//! goes to that seller in full. The unit appears in the city next turn as a **Company**:
//!
//! * it does not count toward the levy cap (`economy::levy_count`) and uses none of Byzantium's
//!   strategic supply (`resources::current_users`): the seller needs an improved deposit of
//!   the unit's resource, but the contract reserves no supply slot;
//! * it starts at promotion level 1, fights, earns XP and captures like any unit, and cannot be
//!   upgraded (hire the next era's Company instead);
//! * its contract runs [`COMPANY_TURNS`] turns, public on the unit, and it goes home at the end
//!   of the last one whatever it is doing ([`expire`]);
//! * it costs [`UPKEEP`] Gold a turn, outside the two free units; when the treasury cannot pay,
//!   a Company is the first thing lost and it turns barbarian where it stands
//!   (`rules::cities::pay_gold`, the Catalan Grand Company).
//!
//! Each empire sells only its own special unit (`data::special_unit`: the Spartan Hoplite, the
//! Keshig, the Eagle Warrior, ...; user, 2026-11-14), and the Company is that empire's version of
//! it: the unit's own stat line, name and rules (the phalanx, the Keshig's bow) follow the seller
//! (`Unit::origin`, `GameState::made_by`). The seller's faction-wide rules (the Mongol move, a
//! Spartan's ignored wounds) do not travel.
//!
//! The price is [`HIRE_PERCENT`] of the seller's version's Production cost in Gold (half the
//! ordinary purchase price).
use crate::economy;
use crate::state::{City, GameState, Unit};
use crate::types::*;
use crate::{Event, Faction, ProductionItem, data};
use serde::{Deserialize, Serialize};

/// Turns a contract lasts: the unit goes home at the end of the `COMPANY_TURNS`th turn after the
/// order's turn. About one campaign; the user's call (2026-10-17: "20 turns feels short").
pub const COMPANY_TURNS: u32 = 30;
/// Gold per turn per Company, outside the free allowance (an ordinary armed unit pays 1).
pub const UPKEEP: i32 = 2;
/// The hire price: this percent of the unit's Production cost, in Gold (everyone else buys at
/// `data::BUY_COST_PERCENT`).
pub const HIRE_PERCENT: i32 = 100;

/// One Company in the shop: who sells it and what it costs now. `unavailable` says why the
/// seller cannot field its special unit yet (the shop shows it muted); such a row is never hired.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub seller: PlayerId,
    pub unit: UnitKind,
    pub price: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<String>,
}

/// Whether this unit is a hired Company.
pub fn is_company(u: &Unit) -> bool {
    u.contract_until.is_some()
}

/// Companies `p` fields now.
pub fn count(state: &GameState, p: PlayerId) -> usize {
    state.units_of(p).filter(|u| is_company(u)).count()
}

/// Turns of contract left at the start of planning turn `turn` (0 on the last turn).
pub fn turns_left(u: &Unit, turn: u32) -> Option<u32> {
    u.contract_until.map(|end| end.saturating_sub(turn))
}

/// Whether Byzantium's Mercenary Companies are in the game (`byzantium_companies` not switched off).
fn hiring_on(state: &GameState) -> bool {
    !state
        .setup
        .rule_experiments
        .rule_off(crate::experiments::FactionRule::ByzantiumCompanies)
}

/// Empires `buyer` may hire from: alive, on another team, met and not at War.
pub fn sellers(state: &GameState, buyer: PlayerId) -> Vec<PlayerId> {
    let Some(d) = state.diplomacy.as_ref() else {
        return Vec::new();
    };
    let team = state.team_of(buyer);
    state
        .players
        .iter()
        .filter(|p| p.alive && p.id != buyer && p.team != team)
        .filter(|p| d.met(team, p.team) && !d.at_war(team, p.team))
        .map(|p| p.id)
        .collect()
}

/// Whether `seller`'s faction sells `kind`: only its own special unit.
pub fn sold_by(faction: Faction, kind: UnitKind) -> bool {
    kind == data::special_unit(faction)
}

/// Whether `seller` can field `kind` right now: it knows the tech (through its own faction's
/// roster) and has an improved deposit of its strategic resource. Other users of the
/// seller's supply do not prevent a contract.
pub fn seller_can_field(state: &GameState, seller: PlayerId, kind: UnitKind) -> bool {
    seller_refusal(state, seller, kind).is_none()
}

/// What hiring `seller`'s version of `kind` costs `buyer` now.
pub fn price(state: &GameState, buyer: PlayerId, seller: PlayerId, kind: UnitKind) -> i32 {
    economy::version_unit_cost(state, buyer, state.player(seller).faction, kind) * HIRE_PERCENT
        / 100
}

/// Why `seller` cannot field `kind` now, if it cannot: the technology it lacks, or the strategic
/// deposit it has not improved.
pub fn seller_refusal(state: &GameState, seller: PlayerId, kind: UnitKind) -> Option<String> {
    let s = state.player(seller);
    let name = data::unit_name_for(s.faction, kind);
    let item = ProductionItem::Unit { unit: kind };
    if !state
        .setup
        .rule_experiments
        .available_to_player(s.faction, state.era_of(seller), item)
    {
        return Some(format!("{name} is not on offer yet"));
    }
    if let Some(tech) = data::item_requires_for_rules(state.setup.rule_experiments, item)
        && !s.techs.contains(&tech)
    {
        return Some(format!("{name} needs {}", data::tech_name(tech)));
    }
    let need = crate::resources::unit_need(kind)?;
    let held = state.map.tiles.iter().enumerate().any(|(i, tile)| {
        tile.owner == Some(seller)
            && tile.resource == Some(need)
            && (tile.improved
                || state
                    .city_at(state.map.hex_at(i))
                    .is_some_and(|city| city.owner == seller))
    });
    (!held).then(|| {
        format!(
            "{name} needs {}: no improved deposit yet",
            crate::resources::name(need)
        )
    })
}

/// The shop of `buyer`: one row per seller, its own special unit, priced as its version. A
/// seller that cannot field it yet carries the reason in `unavailable`. Ordered by seller. Empty
/// for any faction but Byzantium.
pub fn offers(state: &GameState, buyer: PlayerId) -> Vec<Offer> {
    if state.player(buyer).faction != Faction::Byzantium || !hiring_on(state) {
        return Vec::new();
    }
    sellers(state, buyer)
        .into_iter()
        .map(|seller| {
            let unit = data::special_unit(state.player(seller).faction);
            Offer {
                seller,
                unit,
                price: price(state, buyer, seller, unit),
                unavailable: seller_refusal(state, seller, unit),
            }
        })
        .collect()
}

/// Why `buyer` cannot hire `kind` from `seller` in `city` now, if anything blocks it.
pub fn blocker(state: &GameState, city: &City, kind: UnitKind, seller: PlayerId) -> Option<String> {
    let buyer = city.owner;
    if state.player(buyer).faction != Faction::Byzantium {
        return Some("Only Byzantium can hire Mercenary Companies".into());
    }
    if !hiring_on(state) {
        return Some("Mercenary Companies are switched off".into());
    }
    if !sellers(state, buyer).contains(&seller) {
        return Some(
            "That empire sells no Companies to you: it must be met, and not at War".into(),
        );
    }
    let seller_faction = state.player(seller).faction;
    if !sold_by(seller_faction, kind) {
        return Some(format!(
            "{} sells only its own {}",
            data::faction_name(seller_faction),
            data::unit_name_for(seller_faction, data::special_unit(seller_faction))
        ));
    }
    if let Some(reason) = seller_refusal(state, seller, kind) {
        return Some(format!(
            "{} cannot field it now: {reason}",
            data::faction_name(seller_faction)
        ));
    }
    if city.last_purchase_turn == Some(state.turn) {
        return Some(format!("{} already bought something this turn", city.name));
    }
    if let Some(reason) = crate::rules::occupation::city_refusal(state, city) {
        return Some(reason);
    }
    if city.resistance > 0 {
        return Some(format!("{} is resisting occupation", city.name));
    }
    if crate::rules::cities::spawn_tile(state, buyer, city.pos, kind).is_none() {
        return Some("No free spawn tile in or beside the city; move a blocking unit first".into());
    }
    let cost = price(state, buyer, seller, kind);
    let gold = state.player(buyer).gold;
    (gold < cost).then(|| {
        format!(
            "Not enough Gold: the company costs {cost} Gold; you have {gold} (need {} more)",
            cost - gold
        )
    })
}

/// Carry out a checked hire: the Gold moves to the seller, the Company appears next to the city
/// with its contract, and the city has bought for this turn. Returns false (and changes nothing)
/// when no tile is free.
pub fn hire(
    state: &mut GameState,
    city: CityId,
    kind: UnitKind,
    seller: PlayerId,
    events: &mut Vec<Event>,
) -> bool {
    let (owner, pos) = {
        let c = &state.cities[&city];
        (c.owner, c.pos)
    };
    let Some(tile) = crate::rules::cities::spawn_tile(state, owner, pos, kind) else {
        return false;
    };
    let gold = price(state, owner, seller, kind);
    state.player_mut(owner).gold -= gold;
    state.player_mut(seller).gold = crate::validation::credit_gold(state.player(seller).gold, gold);
    let until = state.turn + COMPANY_TURNS;
    let origin = state.player(seller).faction;
    let uid = crate::game::spawn_unit(state, owner, kind, tile);
    let u = state.units.get_mut(&uid).expect("just spawned");
    u.contract_until = Some(until);
    u.origin = Some(origin);
    u.xp = crate::rules::promotion::threshold(1).unwrap_or(0);
    u.moves_left = 0;
    state
        .cities
        .get_mut(&city)
        .expect("city exists")
        .last_purchase_turn = Some(state.turn);
    events.push(Event::CompanyHired {
        city,
        owner,
        seller,
        unit: kind,
        gold,
        until,
    });
    true
}

/// End of a turn's resolution: every Company whose contract ends this turn goes home.
pub fn expire(state: &mut GameState, events: &mut Vec<Event>) {
    let turn = state.turn;
    let gone: Vec<crate::types::UnitId> = state
        .units
        .values()
        .filter(|u| u.contract_until.is_some_and(|end| end <= turn))
        .map(|u| u.id)
        .collect();
    for id in gone {
        let u = state.units.remove(&id).expect("listed above");
        events.push(Event::CompanyDeparted {
            unit: u.kind,
            owner: u.owner,
            origin: u.origin,
            pos: u.pos,
        });
    }
}

/// An unpaid Company turns on its employer: it becomes a barbarian unit where it stands.
pub fn turn_barbarian(state: &mut GameState, id: crate::types::UnitId, events: &mut Vec<Event>) {
    // Barbarians Off omits the controller until a special source creates a barbarian.
    state.barbarians.get_or_insert_with(Default::default);
    let u = state.units.get_mut(&id).expect("company exists");
    let (kind, owner, origin, pos) = (u.kind, u.owner, u.origin, u.pos);
    u.owner = PlayerId::BARBARIAN;
    u.contract_until = None;
    u.origin = None;
    u.goto = None;
    u.path.clear();
    u.automation = None;
    u.standing_order = None;
    u.attack_target = None;
    events.push(Event::CompanyTurned {
        unit: kind,
        owner,
        origin,
        pos,
    });
}
