//! Byzantium's rule break, **Buy a city** (`FactionRule::ByzantiumBuyCity`), and its drawback,
//! **the fall of Constantinople** (`FactionRule::ByzantiumCollapse`) (rules 66).
//!
//! # Buy a city
//!
//! Byzantium orders `Order::BuyCity { city, gold }` on a rival's city. The order starts an
//! *offer*: the bid (`gold`, from the price ([`price`]) up to `MAX_BID_PERCENT`% of it; 0 means
//! the price) leaves Byzantium's treasury into escrow on the offer (`Player::city_offer`, a
//! [`CityOffer`]), and the city changes hands at the end of turn `order turn + OFFER_TURNS`,
//! unless something stops it first. All of the escrow goes to the seller on a sale. A higher bid
//! takes more armed units or Walls to break ([`guards_to_stop`], [`walls_to_stop`]).
//!
//! **Who may be bought.** [`blocker`] gives the one refusal used by order validation, resolution
//! and the view (None means the order is legal now). A city is buyable when:
//! - the buyer is a living Byzantium and the rule is on;
//! - the city belongs to another empire (not barbarians), not on the buyer's team, whose team the
//!   buyer's team has met, and the two are not at War (Peace or Alliance);
//! - it is not a capital and not an original capital (`City::original_capital`), whoever holds it;
//! - one of its tiles is owned by the buyer or is next to a tile the buyer owns ([`touches`]);
//! - it is not in capture resistance;
//! - nobody else is already buying it, and the buyer has no other offer running;
//! - the buyer has the Gold for the price now.
//!
//! [`bid_blocker`] adds the bid's own checks (not below the price, not above `MAX_BID_PERCENT`%
//! of it, and the Gold for it): order validation and resolution use it.
//!
//! [`city_blocker`] is the same check without the last two buyer-side conditions (its own running
//! offer and its treasury): what the city's owner and third parties are shown.
//!
//! **Price** ([`price_breakdown`], [`price`]), integer math, rounded down at each step:
//! `BASE + PER_POP x population + BUILDING_PERCENT% of the Gold price of its buildings`
//! (the Walls family is left out: it is the next step), then
//! `+ WALLS_PERCENT% per Walls level + GUARD_PERCENT% per armed unit of the owner's team in the
//! city or next to it` (added together, applied once), then `x GOAL_MULTIPLIER` if the city holds a
//! goal's held VP ([`holds_goal_vp`]). Every line is in the breakdown, so nothing is hidden.
//!
//! **Each end of turn** while an offer runs ([`process_offers`], after diplomacy resolves, so a
//! same-turn declaration counts):
//! 1. War between the buyer and the owner, the city gone or with a new owner, the owner now on the
//!    buyer's team, or either empire eliminated: **cancelled**, escrow refunded in full.
//! 2. Else, from the turn after the order on, the offer's price now ([`offer_price`]) above the
//!    escrow (the bid): **failed**, the escrow refunded minus `FAIL_FEE_PERCENT`% (the fee goes to nobody).
//!    The offer's price keeps the first step (population and buildings) as it was when the offer
//!    was made and recomputes the rest, so only the owner's defences (Walls, armed units, a goal's
//!    VP moved in) can break an offer, never the city's own growth.
//! 3. Else, on turn `ends_turn`: **sold** ([`transfer_city`]).
//!
//! Every outcome is one public `Event::CityOfferEnded` with a one-line `reason`.
//!
//! **Sale** ([`transfer_city`]): the city changes owner with all its population, buildings, Walls,
//! banked Production and land. It is not a capture: no population loss, no resistance, no
//! occupation lock (`occupied_from` is cleared), no Greece Captive, no Swiss bounty, no Celtic
//! sack, no `CityCaptured` event. The escrow goes to the seller. Units of other teams standing in
//! the city go to their owner's nearest free city, else the nearest free tile their owner owns,
//! else the nearest free unowned land tile; one with nowhere to go is destroyed. Aircraft in its
//! hangar rebase to the seller's nearest city with room, else are lost. The current build and the
//! queue keep what Byzantium can build and drop the rest.
//!
//! # The fall of Constantinople
//!
//! When a Byzantium's current capital (`City::is_capital`) is captured, every other city of that
//! Byzantium is captured by the same captor in the same step, through the ordinary capture path,
//! in city-id order ([`capital_fall`], called from `rules::battle::capture_city`). **Refuge:** if
//! Byzantium's own Heir (owned by it) stands in another of its cities at that moment, nothing
//! collapses and that city becomes the capital (the lowest city id if, impossibly, there were two).
//! Units left with no city follow the ordinary elimination rule (`rules::units::check_elimination`).
//!
//! # API for the AI and the UI
//!
//! - Order: `Order::BuyCity { city, gold }` (JSON `{"type":"BuyCity","city":7,"gold":900}`;
//!   `gold` 0 or left out offers the price).
//! - State: `Player::city_offer: Option<CityOffer>`.
//! - View (`PlayerView`): `city_buys` ([`CityBuyView`]: price, breakdown, refusal, the machine
//!   fields the AI reads, and for the buyer the bid choices with what stops each),
//!   `city_offers` ([`CityOfferView`]: every running offer the viewer may know of, with the
//!   armed units or Walls levels that would stop it) and
//!   `byzantine_capitals` ([`CapitalFallView`]: each Byzantium's capital and Heir refuge).
//! - Events: `CityOfferMade`, `CityOfferEnded` (outcome [`OfferOutcome`]), `CapitalFell`
//!   (the collapse) and `HeirRefuge` (the Heir saved the empire).
//! - Functions: [`blocker`], [`bid_blocker`], [`city_blocker`], [`price`], [`price_breakdown`],
//!   [`defended_price`], [`guards_to_stop`], [`walls_to_stop`], [`max_bid`], [`bid_gold`],
//!   [`bid_choices`],
//!   [`base_breakdown`], [`offer_price`] (the number a running offer must stay under), [`offer_on`],
//!   [`touches`], [`bordering`], [`holds_goal_vp`], [`guards`], [`refuge_city`], [`capital_fall`],
//!   [`is_buyer`], [`buying_on`], [`collapse_on`].
use crate::data;
use crate::events::Event;
use crate::experiments::FactionRule;
use crate::hex::Hex;
use crate::orders::{Order, OrderError};
use crate::state::{City, GameState};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Gold every city costs before anything else.
pub const BASE: i32 = 75;
/// Gold per population.
pub const PER_POP: i32 = 25;
/// Share of the Gold price (`data::BUY_COST_PERCENT` of Production cost) of each building.
pub const BUILDING_PERCENT: i32 = 20;
/// Added per Walls level, in percent of the base price.
pub const WALLS_PERCENT: i32 = 25;
/// Added per armed unit of the owner's team in the city or next to it, in percent.
pub const GUARD_PERCENT: i32 = 20;
/// The price is multiplied by this when the city holds a goal's held VP.
pub const GOAL_MULTIPLIER: i32 = 2;
/// The city changes hands at the end of turn `order turn + OFFER_TURNS`.
pub const OFFER_TURNS: u32 = 3;
/// Share of the escrow lost when an offer fails because the price rose.
pub const FAIL_FEE_PERCENT: i32 = 10;
/// The most Byzantium may offer, in percent of the price (an overbid needs more guards to break).
pub const MAX_BID_PERCENT: i32 = 300;
/// The bid choices shown to the buyer, in percent of the price (`CityBuyView::bids`).
pub const BID_STEPS: [i32; 4] = [100, 150, 200, 300];

/// A running offer (`Player::city_offer`). Public: every player who knows the city sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityOffer {
    pub city: CityId,
    /// The city's owner when the offer was made; the Gold goes to it.
    pub seller: PlayerId,
    /// The Gold held for the sale (the bid: the price when the offer was made, or more).
    pub escrow: i32,
    /// The first step of that price (`base_breakdown`: base, population, buildings). The failure
    /// check keeps it and recomputes only the defences (`offer_price`).
    pub base: i32,
    /// The turn the offer was made.
    pub started: u32,
    /// The city changes hands at the end of this turn.
    pub ends_turn: u32,
}

/// How an offer ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferOutcome {
    /// The city changed hands; the escrow went to the seller.
    Sold,
    /// Called off (War, the city lost or gone, ...): the escrow came back in full.
    Cancelled,
    /// The price rose above the escrow: the escrow came back minus the fee.
    Failed,
}

/// One bid choice on a city (`CityBuyView::bids`): the Gold, and what it takes to break it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BidView {
    /// Percent of the price (`BID_STEPS`, capped at `MAX_BID_PERCENT`).
    pub percent: i32,
    pub gold: i32,
    /// More armed units in or next to the city that would lift the price above this bid.
    pub guards_to_stop: i32,
    /// More Walls levels that would, with today's units; None when Walls alone cannot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walls_to_stop: Option<i32>,
}

/// One city's price for one Byzantium, as the viewer may see it (`PlayerView::city_buys`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityBuyView {
    pub city: CityId,
    /// The Byzantium that would buy it.
    pub buyer: PlayerId,
    /// The price now (`price`).
    pub price: i32,
    /// Every line of the price; the parts sum to `price`.
    pub breakdown: Breakdown,
    /// Why the buyer cannot make the offer now (`blocker` for the buyer's own view,
    /// `city_blocker` for everyone else); None when it can.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker: Option<String>,
    /// Machine fields (the AI reads these, never the text): the price's first step (`BASE`,
    /// population and buildings), which an offer keeps; the defences multiply it
    /// ([`defended_price`]).
    #[serde(default)]
    pub base_price: i32,
    /// The full Gold price of its buildings
    /// (the Walls family left out; the price charges `BUILDING_PERCENT`% of it).
    #[serde(default)]
    pub buildings_gold: i32,
    /// It holds a goal's held VP (the price is multiplied by `GOAL_MULTIPLIER`).
    #[serde(default)]
    pub holds_goal: bool,
    /// Armed units of the owner's team in or next to it now.
    #[serde(default)]
    pub guards: i32,
    /// The buyer's own view only: the Gold it lacks for the price, when only its treasury blocks
    /// the offer (0 otherwise).
    #[serde(default, skip_serializing_if = "crate::orders::is_zero")]
    pub short_of_gold: i32,
    /// The most the buyer may offer (`MAX_BID_PERCENT`% of the price).
    #[serde(default)]
    pub max_bid: i32,
    /// The buyer's own view only: the bid choices (`BID_STEPS`), each with what stops it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bids: Vec<BidView>,
}

/// A running offer (`PlayerView::city_offers`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityOfferView {
    pub city: CityId,
    pub buyer: PlayerId,
    pub seller: PlayerId,
    pub escrow: i32,
    pub ends_turn: u32,
    /// Turn ends left before the sale, counting this one (1 on the last turn).
    pub turns_left: u32,
    /// The offer's price now (`offer_price`: its base when made, with today's Walls, armed units
    /// and goal VP); the offer fails at the end of a turn on which it is above `escrow`.
    pub price_now: i32,
    /// More armed units in or next to the city that would lift `price_now` above `escrow`
    /// (0 when it is already above).
    #[serde(default)]
    pub guards_to_stop: i32,
    /// More Walls levels that would, with today's units; None when Walls alone cannot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walls_to_stop: Option<i32>,
}

/// A Byzantium's capital and Heir refuge, public to every player (`PlayerView::byzantine_capitals`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapitalFallView {
    pub player: PlayerId,
    /// The current capital: take it and you take every Byzantine city (unless `refuge`).
    pub capital: Option<CityId>,
    /// The other Byzantine city the Heir stands in now, which would become the capital.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refuge: Option<CityId>,
}

/// Whether Buy a city is in this game.
pub fn buying_on(state: &GameState) -> bool {
    !state
        .setup
        .rule_experiments
        .rule_off(FactionRule::ByzantiumBuyCity)
}

/// Whether the fall of Constantinople is in this game.
pub fn collapse_on(state: &GameState) -> bool {
    !state
        .setup
        .rule_experiments
        .rule_off(FactionRule::ByzantiumCollapse)
}

/// Whether `p` buys cities: a living Byzantium, with the rule on.
pub fn is_buyer(state: &GameState, p: PlayerId) -> bool {
    buying_on(state)
        && !p.is_barbarian()
        && state
            .players
            .get(p.0 as usize)
            .is_some_and(|pl| pl.alive && pl.faction == Faction::Byzantium)
}

/// The armed units of the owner's team in the city or on a tile next to it.
pub fn guards(state: &GameState, c: &City) -> i32 {
    state
        .units
        .values()
        .filter(|u| {
            u.pos.distance(c.pos) <= 1
                && crate::rules::is_military(u.kind)
                && !u.owner.is_barbarian()
                && state.same_team(u.owner, c.owner)
        })
        .count() as i32
}

/// Whether the city holds a goal's held VP: a landmark (a Great Work or goal building) on its
/// land, the city of a held goal (Imperial Diwan, Yam Relay Station, Colonia Victrix, royal
/// tomb), or a Heir standing in it.
pub fn holds_goal_vp(state: &GameState, c: &City) -> bool {
    if state.map.tiles.iter().any(|t| {
        t.landmark.is_some_and(|l| l.city == c.id) || (t.city == Some(c.id) && t.landmark.is_some())
    }) {
        return true;
    }
    if state
        .units_at(c.pos)
        .any(|u| u.kind == UnitKind::HeirToTheThrone)
    {
        return true;
    }
    state.players.iter().any(|p| {
        !crate::goals::switched_off(state, p.id)
            && (crate::goals::hall_city(state, p.id).is_some_and(|h| h.id == c.id)
                || crate::goals::colonia_city(state, p.id).is_some_and(|h| h.id == c.id)
                || crate::goals::tomb_city(state, p.id).is_some_and(|h| h.id == c.id)
                || (p.diwan_city == Some(c.id)
                    && data::has_building(&c.buildings, BuildingKind::ImperialDiwan)))
    })
}

/// The price of `c` now, line by line (parts sum to the total).
pub fn price_breakdown(state: &GameState, c: &City) -> Breakdown {
    with_defences(state, c, base_breakdown(state, c))
}

/// The first step of the price: `BASE`, population and buildings.
pub fn base_breakdown(state: &GameState, c: &City) -> Breakdown {
    let mut b = Breakdown::default();
    b.add("Starting price", BASE);
    b.add(
        format!("Population {} x {PER_POP}", c.population),
        c.population.max(0).saturating_mul(PER_POP),
    );
    b.add(
        "Buildings".to_string(),
        clamp(building_gold(state, c) * i64::from(BUILDING_PERCENT) / 100),
    );
    b
}

/// The full Gold price of `c`'s buildings, the Walls family left out.
fn building_gold(state: &GameState, c: &City) -> i64 {
    let rules = state.setup.rule_experiments;
    let faction = state.player(c.owner).faction;
    c.buildings
        .iter()
        .filter(|k| data::building_family(**k) != BuildingKind::Walls)
        .map(|k| {
            i64::from(
                rules.base_production_cost(faction, ProductionItem::Building { building: *k }),
            ) * i64::from(data::BUY_COST_PERCENT)
                / 100
        })
        .sum()
}

/// The later steps on top of `base`: Walls and armed units (added together), then the goal
/// doubling. These are what the owner controls while an offer runs.
fn with_defences(state: &GameState, c: &City, mut b: Breakdown) -> Breakdown {
    let base = i64::from(b.total);
    let walls = i64::from(c.walls_level) * i64::from(WALLS_PERCENT);
    let guards = i64::from(guards(state, c));
    let guard_percent = guards * i64::from(GUARD_PERCENT);
    let walls_gold = base * walls / 100;
    let both_gold = base * (walls + guard_percent) / 100;
    b.add(
        format!("Walls level {}: +{walls}%", c.walls_level),
        clamp(walls_gold),
    );
    b.add(
        format!(
            "{guards} military unit{} in or next to it: +{guard_percent}%",
            if guards == 1 { "" } else { "s" }
        ),
        clamp(both_gold - walls_gold),
    );
    if holds_goal_vp(state, c) {
        let before = i64::from(b.total);
        b.add(
            format!("Holds a goal's VP: x{GOAL_MULTIPLIER}"),
            clamp(before * i64::from(GOAL_MULTIPLIER - 1)),
        );
    }
    b
}

/// The price from its first step `base` with the defences (the same math as the breakdown):
/// `base + base x (WALLS_PERCENT x walls + GUARD_PERCENT x guards) / 100`, then
/// `x GOAL_MULTIPLIER` when `goal`.
pub fn defended_price(base: i32, walls_level: i32, guards: i32, goal: bool) -> i32 {
    let base = i64::from(base);
    let percent = i64::from(walls_level) * i64::from(WALLS_PERCENT)
        + i64::from(guards) * i64::from(GUARD_PERCENT);
    let total = base + base * percent / 100;
    clamp(if goal {
        total * i64::from(GOAL_MULTIPLIER)
    } else {
        total
    })
}

/// More armed units the owner must add (in or next to the city) so the price from `base` passes
/// `escrow`: 0 when it is already above. Capped at 99.
pub fn guards_to_stop(base: i32, walls_level: i32, guards: i32, goal: bool, escrow: i32) -> i32 {
    (0..=99)
        .find(|k| defended_price(base, walls_level, guards + k, goal) > escrow)
        .unwrap_or(99)
}

/// More Walls levels the owner must add (with today's units) so the price from `base` passes
/// `escrow`: Some(0) when it is already above, None when even the top level does not.
pub fn walls_to_stop(
    base: i32,
    walls_level: i32,
    guards: i32,
    goal: bool,
    escrow: i32,
) -> Option<i32> {
    (0..=(i32::from(data::MAX_WALL_TIER) - walls_level).max(0))
        .find(|k| defended_price(base, walls_level + k, guards, goal) > escrow)
}

/// The most a buyer may offer for a city priced `price`.
pub fn max_bid(price: i32) -> i32 {
    clamp(i64::from(price) * i64::from(MAX_BID_PERCENT) / 100)
}

/// The Gold an order's `gold` stands for: 0 (or less) offers the price.
pub fn bid_gold(price: i32, gold: i32) -> i32 {
    if gold <= 0 { price } else { gold }
}

/// The bid choices for a city priced `price`: (percent, Gold) per `BID_STEPS`, capped at
/// [`max_bid`], without repeats.
pub fn bid_choices(price: i32) -> Vec<(i32, i32)> {
    let mut out: Vec<(i32, i32)> = Vec::new();
    for step in BID_STEPS {
        let percent = step.min(MAX_BID_PERCENT);
        let gold = clamp(i64::from(price) * i64::from(percent) / 100);
        if out.last().is_none_or(|(_, g)| *g < gold) {
            out.push((percent, gold));
        }
    }
    out
}

/// What a running offer's city costs now for the failure check: the offer's `base` (population
/// and buildings when it was made, so growth or a finished building never breaks an offer) with
/// today's Walls, armed units and goal VP. 0 when the city is gone.
pub fn offer_price(state: &GameState, offer: &CityOffer) -> i32 {
    let Some(c) = state.cities.get(&offer.city) else {
        return 0;
    };
    let mut base = Breakdown::default();
    base.add(
        "Population and buildings when the offer was made",
        offer.base,
    );
    with_defences(state, c, base).total
}

fn clamp(v: i64) -> i32 {
    v.clamp(0, i64::from(i32::MAX)) as i32
}

/// The price of `city` now; 0 for a city that does not exist.
pub fn price(state: &GameState, city: CityId) -> i32 {
    state
        .cities
        .get(&city)
        .map_or(0, |c| price_breakdown(state, c).total)
}

/// Cities with a tile owned by `buyer` or next to one, in id order.
pub fn bordering(state: &GameState, buyer: PlayerId) -> BTreeSet<CityId> {
    let mut out = BTreeSet::new();
    for (i, t) in state.map.tiles.iter().enumerate() {
        if t.owner != Some(buyer) {
            continue;
        }
        let h = state.map.hex_at(i);
        if let Some(c) = t.city {
            out.insert(c);
        }
        for n in h.neighbors() {
            if let Some(c) = state.map.get(n).and_then(|t| t.city) {
                out.insert(c);
            }
        }
    }
    out
}

/// Whether one of `c`'s tiles is owned by `buyer` or is next to a tile `buyer` owns.
pub fn touches(state: &GameState, buyer: PlayerId, c: &City) -> bool {
    state.map.tiles.iter().enumerate().any(|(i, t)| {
        t.city == Some(c.id)
            && (t.owner == Some(buyer)
                || state
                    .map
                    .hex_at(i)
                    .neighbors()
                    .into_iter()
                    .any(|n| state.map.get(n).is_some_and(|t| t.owner == Some(buyer))))
    })
}

/// The offer running on `city`, by whichever Byzantium.
pub fn offer_on(state: &GameState, city: CityId) -> Option<(PlayerId, &CityOffer)> {
    state.players.iter().find_map(|p| {
        p.city_offer
            .as_ref()
            .filter(|o| o.city == city)
            .map(|o| (p.id, o))
    })
}

/// Why `buyer` cannot buy `city`, leaving out its own treasury and running offer: what the city's
/// owner and third parties see. None when the city itself is buyable by `buyer`.
pub fn city_blocker(state: &GameState, buyer: PlayerId, city: CityId) -> Option<String> {
    city_blocker_with(state, buyer, city, None)
}

fn city_blocker_with(
    state: &GameState,
    buyer: PlayerId,
    city: CityId,
    border: Option<&BTreeSet<CityId>>,
) -> Option<String> {
    if !buying_on(state) {
        return Some("Buying cities is not in this game".into());
    }
    if !is_buyer(state, buyer) {
        return Some("Only Byzantium can buy cities".into());
    }
    let Some(c) = state.cities.get(&city) else {
        return Some("No such city".into());
    };
    if c.owner == buyer {
        return Some(format!("{} is already yours", c.name));
    }
    if c.owner.is_barbarian() {
        return Some(format!("{} is not for sale", c.name));
    }
    let owner = state.player(c.owner);
    let (mine, theirs) = (state.team_of(buyer), state.team_of(c.owner));
    if mine == theirs {
        return Some(format!("{} is on your team", owner.name));
    }
    if !state
        .diplomacy
        .as_ref()
        .is_some_and(|d| d.met(mine, theirs))
    {
        return Some(format!("You have not met {}", owner.name));
    }
    if state.at_war(buyer, c.owner) {
        return Some(format!("You are at War with {}", owner.name));
    }
    if c.is_capital {
        return Some(format!("{} is a capital", c.name));
    }
    if c.original_capital {
        return Some(format!("{} was a capital: it is never for sale", c.name));
    }
    let touching = match border {
        Some(set) => set.contains(&city),
        None => touches(state, buyer, c),
    };
    if !touching {
        return Some(format!("{} does not touch your land", c.name));
    }
    if c.resistance > 0 {
        return Some(format!("{} is still resisting its capture", c.name));
    }
    if let Some((other, _)) = offer_on(state, city).filter(|(p, _)| *p != buyer) {
        return Some(format!(
            "{} is already buying {}",
            state.player(other).name,
            c.name
        ));
    }
    None
}

/// The one refusal for `Order::BuyCity`: order validation, resolution and the buyer's view all use
/// it. None when `buyer` may make the offer now.
pub fn blocker(state: &GameState, buyer: PlayerId, city: CityId) -> Option<String> {
    blocker_with(state, buyer, city, None)
}

fn blocker_with(
    state: &GameState,
    buyer: PlayerId,
    city: CityId,
    border: Option<&BTreeSet<CityId>>,
) -> Option<String> {
    if let Some(reason) = city_blocker_with(state, buyer, city, border) {
        return Some(reason);
    }
    if let Some(offer) = &state.player(buyer).city_offer {
        let name = state
            .cities
            .get(&offer.city)
            .map_or("another city", |c| c.name.as_str());
        return Some(format!(
            "You are already buying {name}: one offer at a time"
        ));
    }
    let cost = price(state, city);
    let gold = state.player(buyer).gold;
    if gold < cost {
        return Some(format!("Needs {cost} Gold: you have {}", gold.max(0)));
    }
    None
}

/// The refusal for an offer of `gold` (0: the price): [`blocker`], then the bid below the price,
/// above `MAX_BID_PERCENT`% of it, or above the treasury. None when the offer may be made.
pub fn bid_blocker(state: &GameState, buyer: PlayerId, city: CityId, gold: i32) -> Option<String> {
    if let Some(reason) = blocker(state, buyer, city) {
        return Some(reason);
    }
    let cost = price(state, city);
    let bid = bid_gold(cost, gold);
    if bid < cost {
        return Some(format!("The offer is below the price of {cost} Gold"));
    }
    let most = max_bid(cost);
    if bid > most {
        return Some(format!(
            "The most you can offer is {most} Gold ({MAX_BID_PERCENT}% of the price)"
        ));
    }
    let have = state.player(buyer).gold;
    if have < bid {
        return Some(format!("Needs {bid} Gold: you have {}", have.max(0)));
    }
    None
}

/// Check `player`'s `BuyCity` orders against `state` (the state after this list's city orders, so
/// earlier purchases have been paid). One offer at a time: a second order in the list is refused.
pub fn validate_orders(state: &GameState, player: PlayerId, orders: &[Order]) -> Vec<OrderError> {
    let mut errors = Vec::new();
    let mut made = false;
    for (index, order) in orders.iter().enumerate() {
        let Order::BuyCity { city, gold } = order else {
            continue;
        };
        let refusal = if made {
            let name = state
                .cities
                .get(city)
                .map_or("another city", |c| c.name.as_str());
            Some(format!(
                "You are already buying {name}: one offer at a time"
            ))
        } else {
            bid_blocker(state, player, *city, *gold)
        };
        match refusal {
            Some(message) => errors.push(OrderError { index, message }),
            None => made = true,
        }
    }
    errors
}

/// Start every legal offer, in player-id order then list order, after the turn's city orders and
/// actions (so purchases are paid first) and before Gold gifts. A refused order is an
/// `OrderFailed`.
pub fn apply_orders(
    state: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    events: &mut Vec<Event>,
) {
    for (player, os) in orders {
        for order in os.iter() {
            let Order::BuyCity { city, gold } = order else {
                continue;
            };
            if let Some(reason) = bid_blocker(state, *player, *city, *gold) {
                events.push(Event::OrderFailed {
                    player: *player,
                    reason: format!("City offer not made: {reason}"),
                });
                continue;
            }
            let c = &state.cities[city];
            let base = base_breakdown(state, c).total;
            let price = price_breakdown(state, c).total;
            let escrow = bid_gold(price, *gold);
            let guards_to_stop = guards_to_stop(
                base,
                i32::from(c.walls_level),
                guards(state, c),
                holds_goal_vp(state, c),
                escrow,
            );
            let seller = c.owner;
            let started = state.turn;
            let ends_turn = started + OFFER_TURNS;
            let p = state.player_mut(*player);
            p.gold -= escrow;
            p.city_offer = Some(CityOffer {
                city: *city,
                seller,
                escrow,
                base,
                started,
                ends_turn,
            });
            events.push(Event::CityOfferMade {
                city: *city,
                buyer: *player,
                seller,
                escrow,
                price,
                guards_to_stop,
                ends_turn,
            });
        }
    }
}

/// Why a running offer is called off now, if it is.
fn cancel_reason(state: &GameState, buyer: PlayerId, offer: &CityOffer) -> Option<String> {
    let seller = state.player(offer.seller);
    if !state.player(buyer).alive {
        return Some("the buyer was eliminated".into());
    }
    let Some(c) = state.cities.get(&offer.city) else {
        return Some("the city is gone".into());
    };
    if c.owner != offer.seller {
        return Some(format!("{} changed hands", c.name));
    }
    if !seller.alive {
        return Some(format!("{} was eliminated", seller.name));
    }
    if state.same_team(buyer, offer.seller) {
        return Some(format!("{} joined the buyer's team", seller.name));
    }
    if state.at_war(buyer, offer.seller) {
        return Some(format!(
            "War between {} and {}",
            state.player(buyer).name,
            seller.name
        ));
    }
    None
}

/// The end-of-turn step for every running offer, in buyer-id order: cancel, fail or sell (module
/// doc). Runs after diplomacy resolves.
pub fn process_offers(state: &mut GameState, events: &mut Vec<Event>) {
    if state.finished {
        return;
    }
    let buyers: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.city_offer.is_some())
        .map(|p| p.id)
        .collect();
    for buyer in buyers {
        let offer = state
            .player(buyer)
            .city_offer
            .clone()
            .expect("listed above");
        let name = state
            .cities
            .get(&offer.city)
            .map_or_else(|| "the city".to_string(), |c| c.name.clone());
        let end = |outcome, paid, refund, fee, reason: String| Event::CityOfferEnded {
            city: offer.city,
            buyer,
            seller: offer.seller,
            outcome,
            paid,
            refund,
            fee,
            reason,
        };
        if let Some(reason) = cancel_reason(state, buyer, &offer) {
            refund(state, buyer, offer.escrow);
            events.push(end(
                OfferOutcome::Cancelled,
                0,
                offer.escrow,
                0,
                format!("Offer for {name} called off: {reason}. The Gold went back."),
            ));
            continue;
        }
        let now = offer_price(state, &offer);
        if state.turn > offer.started && now > offer.escrow {
            let fee = offer.escrow * FAIL_FEE_PERCENT / 100;
            refund(state, buyer, offer.escrow - fee);
            events.push(end(
                OfferOutcome::Failed,
                0,
                offer.escrow - fee,
                fee,
                format!(
                    "Offer for {name} failed: the price rose to {now} Gold. {fee} Gold was lost."
                ),
            ));
            continue;
        }
        if state.turn >= offer.ends_turn {
            state.player_mut(buyer).city_offer = None;
            let seller = offer.seller;
            state.player_mut(seller).gold =
                crate::validation::credit_gold(state.player(seller).gold, offer.escrow);
            transfer_city(state, offer.city, buyer, events);
            events.push(end(
                OfferOutcome::Sold,
                offer.escrow,
                0,
                0,
                format!(
                    "{} bought {name} from {} for {} Gold.",
                    state.player(buyer).name,
                    state.player(seller).name,
                    offer.escrow
                ),
            ));
        }
    }
}

fn refund(state: &mut GameState, buyer: PlayerId, gold: i32) {
    let p = state.player_mut(buyer);
    p.city_offer = None;
    p.gold = crate::validation::credit_gold(p.gold, gold);
}

/// Whether a unit of `owner` of the given class (armed or not) may be put on `h`: passable land,
/// no city of another team, no unit of another team and none of the same class.
fn free_for(state: &GameState, h: Hex, owner: PlayerId, military: bool) -> bool {
    if state.map.move_cost(h).is_none() {
        return false;
    }
    if state
        .city_at(h)
        .is_some_and(|c| !state.same_team(c.owner, owner))
    {
        return false;
    }
    !state
        .units_at(h)
        .any(|u| !state.same_team(u.owner, owner) || crate::rules::is_military(u.kind) == military)
}

/// Where a unit of `owner` standing at `from` goes when the city it is in is sold: its owner's
/// nearest free city, else the nearest free tile it owns, else the nearest free unowned land
/// (ties: lowest city id, then hex order).
fn relocation(state: &GameState, from: Hex, owner: PlayerId, military: bool) -> Option<Hex> {
    let city = state
        .cities
        .values()
        .filter(|c| c.owner == owner && free_for(state, c.pos, owner, military))
        .min_by_key(|c| (c.pos.distance(from), c.id))
        .map(|c| c.pos);
    if city.is_some() {
        return city;
    }
    let tile = |owned: Option<PlayerId>| {
        state
            .map
            .hexes()
            .filter(|h| {
                state.map.get(*h).is_some_and(|t| t.owner == owned)
                    && free_for(state, *h, owner, military)
            })
            .min_by_key(|h| (h.distance(from), *h))
    };
    tile(Some(owner)).or_else(|| tile(None))
}

/// Hand `city` to `to` peacefully (a sale): no capture effects (module doc). Units of other teams
/// in the city leave for their owner's nearest free city or tile; aircraft rebase or are lost.
pub fn transfer_city(state: &mut GameState, city: CityId, to: PlayerId, events: &mut Vec<Event>) {
    let Some(c) = state.cities.get(&city) else {
        return;
    };
    let (from, pos) = (c.owner, c.pos);
    // Aircraft belong to the seller: they rebase to its nearest city with room, else are lost.
    let planes: Vec<UnitId> = crate::rules::air::hangar(state, city)
        .map(|a| a.id)
        .collect();
    for plane in planes {
        let target = state
            .cities
            .values()
            .filter(|d| {
                d.owner == from
                    && d.id != city
                    && crate::rules::air::hangar_blocker(state, d.id).is_none()
            })
            .min_by_key(|d| (d.pos.distance(pos), d.id))
            .map(|d| d.id);
        let i = state
            .aircraft
            .iter()
            .position(|a| a.id == plane)
            .expect("listed above");
        match target {
            Some(dest) => {
                state.aircraft[i].city = dest;
                let kind = state.aircraft[i].kind;
                events.push(Event::AircraftRebased {
                    plane,
                    owner: from,
                    kind,
                    from: city,
                    to: dest,
                });
            }
            None => {
                let a = state.aircraft.remove(i);
                events.push(Event::UnitDestroyed {
                    unit: a.id,
                    owner: from,
                    unit_kind: a.kind,
                    at: pos,
                });
            }
        }
    }
    let c = state.cities.get_mut(&city).expect("checked");
    c.owner = to;
    c.occupied_from = None;
    c.build_automation = None;
    c.last_auto_pick = None;
    c.auto_queued = None;
    c.wonder_lost = None;
    c.production_queue_notified.clear();
    for t in &mut state.map.tiles {
        if t.city == Some(city) {
            t.owner = Some(to);
        }
    }
    // Keep what the buyer can build; drop the rest (its progress stays banked).
    let snapshot = state.cities[&city].clone();
    let keep_current = snapshot.producing.is_some_and(|item| {
        crate::economy::production_blocker(state, &snapshot, item, false).is_none()
    });
    let queue: Vec<ProductionItem> = snapshot
        .production_queue
        .iter()
        .copied()
        .filter(|item| crate::economy::production_blocker(state, &snapshot, *item, true).is_none())
        .collect();
    let c = state.cities.get_mut(&city).expect("checked");
    if !keep_current {
        c.switch_production(None);
    }
    c.production_queue = queue;
    // Units of other teams leave the city.
    let movers: Vec<UnitId> = state
        .units_at(pos)
        .filter(|u| !state.same_team(u.owner, to))
        .map(|u| u.id)
        .collect();
    for id in movers {
        let u = &state.units[&id];
        let (owner, military) = (u.owner, crate::rules::is_military(u.kind));
        match relocation(state, pos, owner, military) {
            Some(dest) => {
                let u = state.units.get_mut(&id).expect("listed above");
                u.pos = dest;
                u.goto = None;
                u.path.clear();
                u.path_stop_on_block = false;
                u.blocked_turns = 0;
                u.attack_target = None;
                u.standing_order = None;
                u.fortify = 0;
                u.needs_orders = true;
                events.push(Event::UnitMoved {
                    unit: id,
                    owner,
                    from: pos,
                    to: dest,
                });
            }
            None => crate::rules::battle::destroy_unit(state, id, events),
        }
    }
}

/// The other city `byz`'s own Heir stands in now (lowest id), which would become its capital if
/// `except` (its capital) fell.
pub fn refuge_city(state: &GameState, byz: PlayerId, except: CityId) -> Option<CityId> {
    let heir = crate::goals::heir(state, byz).filter(|u| u.owner == byz)?;
    state
        .cities
        .values()
        .find(|c| c.owner == byz && c.id != except && c.pos == heir.pos)
        .map(|c| c.id)
}

/// What happens when `captor` takes `city`, decided before the capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapitalFall {
    /// The Heir shelters in `refuge`, which becomes the capital.
    Refuge {
        byz: PlayerId,
        capital: CityId,
        refuge: CityId,
    },
    /// Every other city of `byz` falls to the captor, in id order.
    Collapse {
        byz: PlayerId,
        capital: CityId,
        cities: Vec<CityId>,
    },
}

/// The fall of Constantinople: whether taking `city` takes the whole empire. None unless `city` is
/// a Byzantium's current capital, the rule is on and the captor is not barbarian.
pub fn capital_fall(state: &GameState, city: CityId, captor: PlayerId) -> Option<CapitalFall> {
    if !collapse_on(state) || captor.is_barbarian() {
        return None;
    }
    let c = state.cities.get(&city)?;
    let byz = c.owner;
    if !c.is_capital
        || byz.is_barbarian()
        || byz == captor
        || state.player(byz).faction != Faction::Byzantium
    {
        return None;
    }
    if let Some(refuge) = refuge_city(state, byz, city) {
        return Some(CapitalFall::Refuge {
            byz,
            capital: city,
            refuge,
        });
    }
    let cities: Vec<CityId> = state
        .cities
        .values()
        .filter(|o| o.owner == byz && o.id != city)
        .map(|o| o.id)
        .collect();
    Some(CapitalFall::Collapse {
        byz,
        capital: city,
        cities,
    })
}

/// The public capital and refuge of every living Byzantium, with the rule on.
pub fn capital_views(state: &GameState) -> Vec<CapitalFallView> {
    if !collapse_on(state) {
        return Vec::new();
    }
    state
        .players
        .iter()
        .filter(|p| p.alive && p.faction == Faction::Byzantium)
        .map(|p| {
            let capital = state.capital_of(p.id).map(|c| c.id);
            CapitalFallView {
                player: p.id,
                capital,
                refuge: capital.and_then(|cap| refuge_city(state, p.id, cap)),
            }
        })
        .collect()
}

/// The prices `me` may see, for the cities in `known` (the cities in its view): a Byzantium
/// viewer sees every known city of another empire with its own `blocker`; any other viewer sees
/// the cities each Byzantium it has met could buy (`city_blocker` None), and its own cities that
/// touch a Byzantium's land with their `city_blocker`. Empty with the rule off.
pub fn buy_views(state: &GameState, me: PlayerId, known: &BTreeSet<CityId>) -> Vec<CityBuyView> {
    if !buying_on(state) {
        return Vec::new();
    }
    let my_team = state.team_of(me);
    let mut out = Vec::new();
    for buyer in state.players.iter().filter(|p| is_buyer(state, p.id)) {
        let buyer = buyer.id;
        let mine = buyer == me;
        let buyer_team = state.team_of(buyer);
        if !mine
            && buyer_team != my_team
            && !state
                .diplomacy
                .as_ref()
                .is_some_and(|d| d.met(my_team, buyer_team))
        {
            continue;
        }
        let border = bordering(state, buyer);
        for c in state.cities.values() {
            if !known.contains(&c.id) || c.owner.is_barbarian() || state.same_team(c.owner, buyer) {
                continue;
            }
            let blocker = if mine {
                blocker_with(state, buyer, c.id, Some(&border))
            } else {
                let b = city_blocker_with(state, buyer, c.id, Some(&border));
                let own_bordering = c.owner == me && border.contains(&c.id);
                if b.is_some() && !own_bordering {
                    continue;
                }
                b
            };
            let base = base_breakdown(state, c);
            let base_total = base.total;
            let breakdown = with_defences(state, c, base);
            let price = breakdown.total;
            let holds_goal = holds_goal_vp(state, c);
            let guards = guards(state, c);
            let walls = i32::from(c.walls_level);
            let treasury = state.player(buyer).gold;
            let short_of_gold = if mine
                && treasury < price
                && blocker.as_deref().is_some_and(|b| b.starts_with("Needs "))
            {
                price - treasury.max(0)
            } else {
                0
            };
            let bids = if mine {
                bid_choices(price)
                    .into_iter()
                    .map(|(percent, gold)| BidView {
                        percent,
                        gold,
                        guards_to_stop: guards_to_stop(base_total, walls, guards, holds_goal, gold),
                        walls_to_stop: walls_to_stop(base_total, walls, guards, holds_goal, gold),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            out.push(CityBuyView {
                city: c.id,
                buyer,
                price,
                breakdown,
                blocker,
                base_price: base_total,
                buildings_gold: clamp(building_gold(state, c)),
                holds_goal,
                guards,
                short_of_gold,
                max_bid: max_bid(price),
                bids,
            });
        }
    }
    out
}

/// Every running offer `me` may know of: on a city in `known`, or where `me`'s team buys or sells.
pub fn offer_views(
    state: &GameState,
    me: PlayerId,
    known: &BTreeSet<CityId>,
) -> Vec<CityOfferView> {
    state
        .players
        .iter()
        .filter_map(|p| p.city_offer.as_ref().map(|o| (p.id, o)))
        .filter(|(buyer, o)| {
            known.contains(&o.city) || state.same_team(*buyer, me) || state.same_team(o.seller, me)
        })
        .map(|(buyer, o)| {
            let (walls, guards, goal) = state.cities.get(&o.city).map_or((0, 0, false), |c| {
                (
                    i32::from(c.walls_level),
                    self::guards(state, c),
                    holds_goal_vp(state, c),
                )
            });
            CityOfferView {
                city: o.city,
                buyer,
                seller: o.seller,
                escrow: o.escrow,
                ends_turn: o.ends_turn,
                turns_left: o.ends_turn.saturating_sub(state.turn) + 1,
                price_now: offer_price(state, o),
                guards_to_stop: guards_to_stop(o.base, walls, guards, goal, o.escrow),
                walls_to_stop: walls_to_stop(o.base, walls, guards, goal, o.escrow),
            }
        })
        .collect()
}
