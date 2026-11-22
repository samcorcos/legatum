//! City, research, border and economy phases. Owned by the "rules-cities" work (story S3/S4).
//! `rules::resolve_turn` (mod.rs) calls these in the fixed phase order:
//! `apply_city_orders` (phase 1), `found_city` (phase 5), `process_cities` (phases 6, 7 and 10),
//! `process_research` (phase 8), `process_borders` (phase 9), then `scoring::update`.

use crate::data::{self, *};
use crate::economy::{self, PlayerIncome};
use crate::events::Event;
use crate::game;
use crate::hex::Hex;
use crate::orders::{Order, OrderError};
use crate::rng::{self, Rng};
use crate::state::{City, GameState, WonderLoss};
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

/// Production stored with an empty production plan stops accumulating here.
pub const IDLE_PRODUCTION_CAP: i32 = 60;

/// Ordinary buildings alone use Egypt's Food construction rule.
pub fn egypt_building(state: &GameState, c: &City, item: ProductionItem) -> bool {
    matches!(item, ProductionItem::Building { .. })
        && state.player(c.owner).faction == Faction::Egypt
        && state.setup.rule_experiments.egypt_construction()
}

/// Most Production a city banks while it builds nothing: the idle cap with an
/// empty plan, or the first queued item's cost while the queue waits (so a blocked Settler
/// can't bank toward a Great Work).
pub fn bank_cap(state: &GameState, c: &City) -> i32 {
    match c.production_queue.first() {
        Some(&next) => economy::city_production_cost(state, c, next),
        None => IDLE_PRODUCTION_CAP,
    }
}
/// Strategic-deposit pull (`resources::border_pull`) in claim-score points: one pull point is
/// worth one Food under the Balanced focus.
pub const CLAIM_PULL_WEIGHT: i32 = 4;

pub fn item_name(item: ProductionItem) -> &'static str {
    match item {
        ProductionItem::Unit { unit } => data::unit_name(unit),
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            data::building_name(building)
        }
        ProductionItem::Project {
            project: IndustryProject::Gold,
        } => "Industry: Gold",
        ProductionItem::Project {
            project: IndustryProject::Science,
        } => "Industry: Science",
        ProductionItem::Project {
            project: IndustryProject::Grow,
        } => "Grow",
        ProductionItem::Improvement { improvement, .. } => {
            crate::improvements::name(improvement, false)
        }
    }
}

fn is_city_order(order: &Order) -> bool {
    matches!(
        order,
        Order::SetProduction { .. }
            | Order::SetCityAutomation { .. }
            | Order::Raze { .. }
            | Order::SetProductionQueue { .. }
            | Order::BuyItem { .. }
            | Order::HireCompany { .. }
            | Order::BuyTile { .. }
            | Order::SetResearch { .. }
            | Order::SetResearchQueue { .. }
            | Order::SetScienceConversion { .. }
    )
}

fn city_of_order(order: &Order) -> Option<CityId> {
    match *order {
        Order::Raze { city }
        | Order::SetProduction { city, .. }
        | Order::SetCityAutomation { city, .. }
        | Order::SetProductionQueue { city, .. }
        | Order::BuyItem { city, .. }
        | Order::HireCompany { city, .. }
        | Order::BuyTile { city, .. } => Some(city),
        _ => None,
    }
}

fn owned_city(state: &GameState, player: PlayerId, city: CityId) -> Result<&City, String> {
    state
        .cities
        .get(&city)
        .filter(|c| c.owner == player)
        .ok_or_else(|| "No such city of yours".into())
}

/// Share the exact refusal with the owner's view, AI and order validation.
pub fn raze_blocker(state: &GameState, player: PlayerId, city: CityId) -> Option<String> {
    validate_city_order(state, player, &Order::Raze { city }).err()
}

/// The last captor's victim, if the chronicle reaches that capture; otherwise the founder. No private information is exposed: the razer's empire already holds the city.
pub fn former_owner(state: &GameState, c: &City) -> PlayerId {
    state
        .history
        .chronicle
        .iter()
        .rev()
        .find_map(|entry| match entry {
            crate::history::Chronicle::Captured { from, by, city, .. }
                if *by == c.owner && city == &c.name =>
            {
                Some(*from)
            }
            _ => None,
        })
        .unwrap_or(c.founder)
}

/// A captured capital loses `is_capital`; its first founding remains in the chronicle.
pub fn is_original_capital(state: &GameState, c: &City) -> bool {
    if c.original_capital || c.is_capital {
        return true;
    }
    let first_founding = state
        .history
        .chronicle
        .iter()
        .find_map(|entry| match entry {
            crate::history::Chronicle::Founded {
                player: founder,
                city: name,
                turn,
            } if *founder == c.founder => Some((name.as_str(), *turn)),
            _ => None,
        });
    first_founding == Some((c.name.as_str(), c.founded_turn))
}

/// Check one city order against `state`. `bought` holds cities that already bought this turn.
fn check_order(
    state: &GameState,
    player: PlayerId,
    order: &Order,
    bought: &BTreeSet<CityId>,
) -> Result<(), String> {
    if !is_city_order(order) {
        return Ok(());
    }
    let Some(p) = state.players.get(player.0 as usize) else {
        return Err(format!("Unknown player #{}", player.0));
    };
    if !p.alive {
        return Err(format!("{} has been eliminated", p.name));
    }
    match *order {
        Order::SetCityAutomation { city, picked, .. } => {
            owned_city(state, player, city)?;
            // The pick is kept on the city as it is, so a tile off the map would make the next save
            // unloadable (`validation::check_state`). In an online game the order comes from a guest.
            if let Some(
                ProductionItem::Wonder { tile, .. } | ProductionItem::Improvement { tile, .. },
            ) = picked
                && !state.map.contains(tile)
            {
                return Err("that tile is off the map".to_string());
            }
        }
        Order::Raze { city } => {
            let c = owned_city(state, player, city)?;
            if c.founder == player {
                return Err(format!("{} was founded by you", c.name));
            }
            if c.resistance >= data::CAPTURE_RESISTANCE_TURNS {
                return Err(format!(
                    "{} can be razed from the turn after its capture",
                    c.name
                ));
            }
        }
        Order::SetProduction { city, item } => {
            let c = owned_city(state, player, city)?;
            if let Some(reason) = economy::production_blocker(state, c, item, true) {
                return Err(format!(
                    "{} cannot produce {}: {reason}",
                    c.name,
                    item_name(item)
                ));
            }
            if c.producing != Some(item)
                && let Some(reason) = crate::resources::start_blocker(state, c, item)
            {
                return Err(format!(
                    "{} cannot start {}: {reason}",
                    c.name,
                    item_name(item)
                ));
            }
        }
        Order::SetProductionQueue { city, ref queue } => {
            let c = owned_city(state, player, city)?;
            if queue.len() > 50 {
                return Err("A production queue can contain at most 50 items".into());
            }
            let mut projected = c.clone();
            projected.production_queue.clear();
            projected.producing = None;
            let mut work = false;
            let mut wonder_tiles = BTreeSet::new();
            for (index, item) in queue.iter().copied().enumerate() {
                let continuing = index == 0 && c.producing == Some(item);
                if let Some(reason) =
                    economy::production_blocker(state, &projected, item, !continuing)
                {
                    return Err(format!(
                        "Queue item {}: {} cannot produce {}: {reason}",
                        index + 1,
                        c.name,
                        item_name(item)
                    ));
                }
                // Only the head starts now; later items pay when they start.
                if index == 0
                    && !continuing
                    && let Some(reason) = crate::resources::start_blocker(state, c, item)
                {
                    return Err(format!(
                        "{} cannot start {}: {reason}",
                        c.name,
                        item_name(item)
                    ));
                }
                if data::is_great_work_item(item) {
                    if work {
                        return Err("Only one world wonder can be queued per empire".into());
                    }
                    work = true;
                }
                if let ProductionItem::Wonder { tile, .. } = item
                    && !wonder_tiles.insert(tile)
                {
                    return Err("A wonder tile can only be queued once".into());
                }
                if let ProductionItem::Building { building }
                | ProductionItem::Wonder { building, .. } = item
                {
                    economy::install_building(&mut projected, building);
                }
            }
        }
        Order::BuyItem { city, item } => {
            let c = owned_city(state, player, city)?;
            if bought.contains(&city) {
                return Err(format!("{} already bought something this turn", c.name));
            }
            if let Some(reason) = economy::buy_blocker(state, c, item) {
                return Err(reason);
            }
        }
        Order::HireCompany { city, unit, seller } => {
            let c = owned_city(state, player, city)?;
            if bought.contains(&city) {
                return Err(format!("{} already bought something this turn", c.name));
            }
            if let Some(reason) = crate::company::blocker(state, c, unit, seller) {
                return Err(reason);
            }
        }
        Order::BuyTile { city, tile } => {
            let c = owned_city(state, player, city)?;
            if bought.contains(&city) {
                return Err(format!("{} already bought something this turn", c.name));
            }
            if let Some(reason) = buy_tile_blocker(state, c, tile) {
                return Err(reason);
            }
        }
        Order::SetResearch { tech } if !economy::tech_available(state, player, tech) => {
            return Err(if p.techs.contains(&tech) {
                format!("{} is already researched", data::tech_name(tech))
            } else {
                format!(
                    "{} is in the {} era: finish the {} era first",
                    data::tech_name(tech),
                    crate::eras::era_name(crate::eras::era_of_tech(tech)),
                    crate::eras::era_name(crate::eras::current_era(&p.techs))
                )
            });
        }
        Order::SetResearchQueue { ref techs } => {
            let mut seen = BTreeSet::new();
            for tech in techs {
                let name = data::tech_name(*tech);
                if p.techs.contains(tech) {
                    return Err(format!("{name} is already researched"));
                }
                if !seen.insert(*tech) {
                    return Err(format!(
                        "{name} appears more than once in the research queue"
                    ));
                }
                if !crate::eras::researchable(p, *tech) {
                    return Err(format!(
                        "{name} is in the {} era: finish the {} era first",
                        crate::eras::era_name(crate::eras::era_of_tech(*tech)),
                        crate::eras::era_name(crate::eras::current_era(&p.techs))
                    ));
                }
            }
        }
        Order::SetScienceConversion { .. } if !crate::eras::all_researched(&p.techs) => {
            return Err("Finish the technology tree before choosing a Science yield".into());
        }
        _ => {}
    }
    Ok(())
}

/// Apply one already-checked city order.
fn apply_order(
    state: &mut GameState,
    player: PlayerId,
    order: &Order,
    bought: &mut BTreeSet<CityId>,
    events: &mut Vec<Event>,
) {
    match *order {
        Order::Raze { city } => {
            super::air::lose_city_aircraft(state, city, events);
            let from = former_owner(state, &state.cities[&city]);
            let c = state.cities.remove(&city).expect("checked");
            let gold = c.population.max(0).saturating_mul(10);
            state.player_mut(player).gold =
                crate::validation::credit_gold(state.player(player).gold, gold);
            for i in 0..state.map.tiles.len() {
                if state.map.tiles[i].city == Some(city) {
                    let t = &mut state.map.tiles[i];
                    t.city = None;
                    t.owner = None;
                    t.improved = false;
                    t.landmark = None;
                }
            }
            events.push(Event::CityRazed {
                city,
                from,
                by: player,
                at: c.pos,
                original_capital: c.original_capital,
                sack_gold: Some(gold),
                camp: None,
                ordered: true,
                prize: false,
                walled: false,
            });
        }
        Order::SetProduction { city, item } => {
            let egypt = egypt_building(state, &state.cities[&city], item);
            let c = state.cities.get_mut(&city).expect("checked");
            c.wonder_lost = None;
            c.auto_queued = None;
            c.last_auto_pick = None;
            c.production_queue.clear();
            c.production_queue_notified.clear();
            if c.producing != Some(item) {
                // The old build's progress stays banked with it; the new one resumes its own.
                c.switch_production_for(Some(item), egypt);
                if let ProductionItem::Building { building }
                | ProductionItem::Wonder { building, .. } = item
                    && data::is_great_work(building)
                {
                    events.push(Event::MegaprojectStarted {
                        city,
                        owner: player,
                        building,
                    });
                }
            }
        }
        Order::SetCityAutomation {
            city,
            focus,
            picked,
        } => {
            let c = state.cities.get_mut(&city).expect("checked");
            if focus.is_some() {
                c.wonder_lost = None;
            }
            if c.build_automation != focus {
                if c.auto_queued.is_some() && c.production_queue.last().copied() == c.auto_queued {
                    c.production_queue.pop();
                }
                c.auto_queued = None;
                c.last_auto_pick = None;
            }
            c.build_automation = focus;
            if focus.is_none() {
                c.last_auto_pick = None;
            }
            if let Some(item) = picked.filter(|_| focus.is_some()) {
                c.last_auto_pick = Some(item);
                c.auto_queued = (c.production_queue.last() == Some(&item)).then_some(item);
            }
        }
        Order::SetProductionQueue { city, ref queue } => {
            let egypt = queue
                .first()
                .is_some_and(|item| egypt_building(state, &state.cities[&city], *item));
            let c = state.cities.get_mut(&city).expect("checked");
            c.wonder_lost = None;
            c.auto_queued = None;
            c.last_auto_pick = None;
            let next = queue.first().copied();
            let started = c.producing != next;
            c.switch_production_for(next, egypt);
            c.production_queue = queue.iter().skip(1).copied().collect();
            c.production_queue_notified
                .retain(|item| queue.contains(item));
            if started {
                announce_start(city, player, next, events);
            }
        }
        Order::BuyItem { city, item } => {
            let gold = economy::buy_cost_for(state, &state.cities[&city], item).expect("checked");
            state.player_mut(player).gold -= gold;
            if matches!(item, ProductionItem::Unit { unit } if data::unit_stats(unit).military) {
                state.player_mut(player).military_purchases += 1;
            }
            state
                .cities
                .get_mut(&city)
                .expect("checked")
                .last_purchase_turn = Some(state.turn);
            bought.insert(city);
            events.push(Event::ItemBought {
                city,
                owner: player,
                item,
                gold,
            });
            let completed = complete_item(state, city, item, events);
            debug_assert!(
                completed,
                "purchase was checked immediately before completion"
            );
            // A purchased building needs no second construction. Production already earned
            // toward it carries on like a completion's overflow: into the current build, or
            // into the next one when it was the current build.
            if matches!(
                item,
                ProductionItem::Building { .. }
                    | ProductionItem::Wonder { .. }
                    | ProductionItem::Improvement { .. }
            ) {
                if state.cities[&city].producing == Some(item) {
                    let c = state.cities.get_mut(&city).expect("checked");
                    c.forfeit_purchase();
                    c.producing = None;
                    advance_queue(state, city, events);
                } else {
                    let c = state.cities.get_mut(&city).expect("checked");
                    let banked = c.take_banked(item);
                    c.production_stored =
                        crate::validation::add_stored(c.production_stored, banked);
                }
                state
                    .cities
                    .get_mut(&city)
                    .expect("checked")
                    .production_queue
                    .retain(|queued| *queued != item);
            }
        }
        Order::HireCompany { city, unit, seller } => {
            let hired = crate::company::hire(state, city, unit, seller, events);
            debug_assert!(
                hired,
                "hire was checked immediately before it was carried out"
            );
            bought.insert(city);
        }
        Order::BuyTile { city, tile } => {
            let gold = tile_price(&state.cities[&city], tile);
            state.player_mut(player).gold -= gold;
            state
                .cities
                .get_mut(&city)
                .expect("checked")
                .last_purchase_turn = Some(state.turn);
            bought.insert(city);
            claim_tile(state, city, tile);
            events.push(Event::TileBought {
                city,
                owner: player,
                tile,
                gold,
            });
        }
        Order::SetResearch { tech } => {
            // Stored science is one pool, so switching keeps it.
            state.player_mut(player).researching = Some(tech);
            state.player_mut(player).research_queue = vec![tech];
        }
        Order::SetResearchQueue { ref techs } => {
            let p = state.player_mut(player);
            p.researching = techs.first().copied();
            p.research_queue.clone_from(techs);
        }
        Order::SetScienceConversion { to } => {
            state.player_mut(player).science_conversion = to;
        }
        _ => {}
    }
}

/// Validate a SetProduction, BuyItem or SetResearch order. Other orders: Ok(()).
/// Checks against `state` as it is; see `validate_city_orders` for a whole order list.
pub fn validate_city_order(
    state: &GameState,
    player: PlayerId,
    order: &Order,
) -> Result<(), String> {
    check_order(state, player, order, &BTreeSet::new())
}

/// Validate a player's whole order list the way `apply_city_orders` will apply it: in order,
/// each city order seeing the effects of the earlier ones (a SetProduction followed by a
/// BuyItem, gold spent by earlier buys, one buy per city). Non-city orders are ignored.
pub fn validate_city_orders(
    state: &GameState,
    player: PlayerId,
    orders: &[Order],
) -> Vec<OrderError> {
    validate_city_orders_projected(state, player, orders).0
}

/// `validate_city_orders`, also returning the state after the valid city orders are applied: what
/// the later phases of the same turn (aircraft upgrades) see of the treasury and supply.
pub fn validate_city_orders_projected(
    state: &GameState,
    player: PlayerId,
    orders: &[Order],
) -> (Vec<OrderError>, GameState) {
    let mut sim = state.clone();
    let mut bought = BTreeSet::new();
    let mut touched = BTreeSet::new();
    let mut errors = Vec::new();
    for (index, order) in orders.iter().enumerate() {
        let result = if matches!(order, Order::Raze { city } if touched.contains(city)) {
            Err("A city can receive only its Raze order this turn".into())
        } else {
            check_order(&sim, player, order, &bought)
        };
        match result {
            Ok(()) => {
                if let Some(city) = city_of_order(order) {
                    touched.insert(city);
                }
                apply_order(&mut sim, player, order, &mut bought, &mut Vec::new());
            }
            Err(message) => errors.push(OrderError { index, message }),
        }
    }
    (errors, sim)
}

/// Orders split by [`defer_contested_tile_buys`]: each empire's kept list, then the deferred
/// losing tile buys in drawn order.
pub(crate) type ContestedTileBuys = (BTreeMap<PlayerId, Vec<Order>>, Vec<(PlayerId, Order)>);

/// City orders are applied one empire at a time, so without this the lower id would win every
/// tile two empires buy in the same turn. For each tile that several empires order `BuyTile` on,
/// a seeded draw (`purpose::TILE_CONTEST`) picks the order they are tried in: the winner's order
/// keeps its place, each loser's (its first order for that tile) is taken out of its list and
/// returned to be tried after every empire's city orders, in drawn order. A loser then fails the
/// ordinary way ("already owned") unless the winner's own order failed. None when nothing is
/// contested.
pub(crate) fn defer_contested_tile_buys(
    state: &GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
) -> Option<ContestedTileBuys> {
    let mut bids: BTreeMap<Hex, Vec<(PlayerId, usize)>> = BTreeMap::new();
    for (p, os) in orders {
        for (i, order) in os.iter().enumerate() {
            if let Order::BuyTile { tile, .. } = order {
                let tile_bids = bids.entry(*tile).or_default();
                if !tile_bids.iter().any(|(q, _)| q == p) {
                    tile_bids.push((*p, i));
                }
            }
        }
    }
    let mut removed: BTreeSet<(PlayerId, usize)> = BTreeSet::new();
    let mut deferred = Vec::new();
    for (tile, tile_bids) in bids.iter().filter(|(_, b)| b.len() > 1) {
        let players: Vec<PlayerId> = tile_bids.iter().map(|(p, _)| *p).collect();
        let ranked = rng::seeded_order(
            state.dice_seed(),
            &[
                u64::from(state.turn),
                rng::purpose::TILE_CONTEST,
                tile.q as u64,
                tile.r as u64,
            ],
            &players,
        );
        for loser in &ranked[1..] {
            let (_, i) = *tile_bids.iter().find(|(p, _)| p == loser).expect("bidder");
            removed.insert((*loser, i));
            deferred.push((*loser, orders[loser][i].clone()));
        }
    }
    if deferred.is_empty() {
        return None;
    }
    let kept = orders
        .iter()
        .map(|(p, os)| {
            let list = os
                .iter()
                .enumerate()
                .filter(|(i, _)| !removed.contains(&(*p, *i)))
                .map(|(_, o)| o.clone())
                .collect();
            (*p, list)
        })
        .collect();
    Some((kept, deferred))
}

/// Apply a player's SetProduction, SetResearch and BuyItem orders in list order
/// (other orders are ignored). Invalid orders are skipped and reported as `OrderFailed`.
/// Runs before movement.
pub fn apply_city_orders(
    state: &mut GameState,
    player: PlayerId,
    orders: &[Order],
    events: &mut Vec<Event>,
) {
    let mut bought = BTreeSet::new();
    let mut touched = BTreeSet::new();
    for order in orders {
        let result = if matches!(order, Order::Raze { city } if touched.contains(city)) {
            Err("A city can receive only its Raze order this turn".into())
        } else {
            check_order(state, player, order, &bought)
        };
        match result {
            Ok(()) => {
                if let Some(city) = city_of_order(order) {
                    touched.insert(city);
                }
                apply_order(state, player, order, &mut bought, events);
            }
            Err(reason) => events.push(Event::OrderFailed { player, reason }),
        }
    }
}

const SWISS_NAMES: [&str; 20] = [
    "Bern",
    "Zürich",
    "Lucerne",
    "Basel",
    "Geneva",
    "Lausanne",
    "Fribourg",
    "Solothurn",
    "Schaffhausen",
    "St. Gallen",
    "Chur",
    "Zug",
    "Sion",
    "Neuchâtel",
    "Bellinzona",
    "Lugano",
    "Winterthur",
    "Thun",
    "Biel",
    "Aarau",
];

const INDIA_NAMES: [&str; 20] = [
    "Agra",
    "Delhi",
    "Lahore",
    "Fatehpur Sikri",
    "Ajmer",
    "Allahabad",
    "Jaunpur",
    "Varanasi",
    "Patna",
    "Ahmedabad",
    "Surat",
    "Burhanpur",
    "Gwalior",
    "Ujjain",
    "Kabul",
    "Multan",
    "Srinagar",
    "Ranthambore",
    "Bikaner",
    "Chittor",
];

const BYZANTIUM_NAMES: [&str; 20] = [
    "Constantinople",
    "Thessalonica",
    "Nicaea",
    "Smyrna",
    "Trebizond",
    "Adrianople",
    "Antioch",
    "Ephesus",
    "Corinth",
    "Mystras",
    "Ravenna",
    "Dyrrachium",
    "Nicomedia",
    "Caesarea",
    "Edessa",
    "Syracuse",
    "Cherson",
    "Philadelphia",
    "Pergamon",
    "Heraclea",
];

pub fn city_names(faction: Faction) -> &'static [&'static str] {
    match faction {
        Faction::Aztecs => &[
            "Tenochtitlan",
            "Texcoco",
            "Tlacopan",
            "Tlatelolco",
            "Chalco",
            "Xochimilco",
        ],
        Faction::Switzerland => &SWISS_NAMES,
        Faction::India => &INDIA_NAMES,
        Faction::Byzantium => &BYZANTIUM_NAMES,
        Faction::Germany => &["Berlin", "Cologne", "Hamburg", "Munich", "Leipzig", "Essen"],
        Faction::Celts => &[
            "Alesia",
            "Bibracte",
            "Lugdunum",
            "Gergovia",
            "Tara",
            "Emain Macha",
        ],
        Faction::Mongols => &[
            "Karakorum",
            "Sarai",
            "Khanbaliq",
            "Shangdu",
            "Almaliq",
            "Bukhara",
        ],
        Faction::Sparta => &[
            "Sparta", "Amyclae", "Therapne", "Sellasia", "Gytheio", "Pylos",
        ],
        Faction::Athens => &[
            "Athens", "Piraeus", "Eleusis", "Marathon", "Phaleron", "Laurion",
        ],
        Faction::Egypt => &["Memphis", "Thebes", "Heliopolis", "Abydos", "Aswan", "Sais"],
        Faction::Rome => &[
            "Rome",
            "Ostia",
            "Capua",
            "Ariminum",
            "Aquileia",
            "Mediolanum",
        ],
    }
}

/// The owner's next city name: walk the faction list from a per-empire seeded start and take the
/// first name no city in the game uses; after that, "<Faction> Colony N".
pub fn next_city_name(state: &GameState, owner: PlayerId) -> String {
    let faction = state.player(owner).faction;
    let names = city_names(faction);
    let used: BTreeSet<&str> = state.cities.values().map(|c| c.name.as_str()).collect();
    let start = Rng::derive(state.setup.seed, &[rng::purpose::NAMES, owner.0 as u64])
        .range(0, names.len() as i64 - 1) as usize;
    for i in 0..names.len() {
        let name = names[(start + i) % names.len()];
        if !used.contains(name) {
            return name.to_string();
        }
    }
    let mut n = state.cities.values().filter(|c| c.founder == owner).count() + 1;
    loop {
        let name = format!("{} Colony {n}", data::faction_name(faction));
        if !used.contains(name.as_str()) {
            return name;
        }
        n += 1;
    }
}

/// Check founder eligibility without changing state, before simultaneous conflicts are resolved.
pub(crate) fn check_found_city(state: &GameState, settler: UnitId) -> Result<(), String> {
    let unit = state
        .units
        .get(&settler)
        .ok_or_else(|| "No such unit of yours".to_string())?;
    let roman = state.player(unit.owner).faction == Faction::Rome
        && !state
            .setup
            .rule_experiments
            .rule_off(crate::experiments::FactionRule::RomeVeteranCities)
        && data::unit_class(unit.kind) == Some(UnitClass::Melee);
    if unit.kind != UnitKind::Settler && !roman {
        return Err(format!(
            "{}, not {}",
            if state.player(unit.owner).faction == Faction::Rome {
                "Only Settlers or Roman melee veterans found cities"
            } else {
                "Only Settlers found cities"
            },
            unit_name(unit.kind)
        ));
    }
    if roman && unit.kind != UnitKind::Settler {
        if crate::rules::promotion::level_for(Faction::Rome, unit.xp) == 0 {
            return Err("Roman founder needs at least one promotion".into());
        }
        if unit.hp < data::UNIT_HP {
            return Err(format!(
                "Roman founder needs full health ({} of {} HP)",
                unit.hp,
                data::UNIT_HP
            ));
        }
    }
    let (owner, pos) = (unit.owner, unit.pos);
    let tile = state
        .map
        .get(pos)
        .ok_or_else(|| "The settler is off the map".to_string())?;
    if tile.wonder.is_none()
        && crate::settlement::legal_site(
            owner,
            state
                .setup
                .rule_experiments
                .min_city_gap(state.player(owner).faction),
            tile.terrain,
            tile.owner,
            pos,
            state.cities.values().map(|c| c.pos),
        )
    {
        return Ok(());
    }
    // Diagnostics only; the shared predicate above is the sole acceptance path.
    if let Some(wonder) = tile.wonder {
        return Err(format!("Cannot found a city on {}", wonder.name()));
    }
    if !tile.terrain.is_land() || tile.terrain.move_cost().is_none() {
        return Err(format!("Cannot found a city on {:?}", tile.terrain));
    }
    if let Some(o) = tile.owner
        && o != owner
    {
        return Err(format!(
            "This tile belongs to {}",
            state.player(o).name.clone()
        ));
    }
    if let Some(near) = state
        .cities
        .values()
        .filter(|c| {
            c.pos.distance(pos)
                <= state
                    .setup
                    .rule_experiments
                    .min_city_gap(state.player(owner).faction)
        })
        .min_by_key(|c| (c.pos.distance(pos), c.id))
    {
        let d = near.pos.distance(pos);
        return Err(format!(
            "Too close to {} ({d} {} away): cities must be at least {} tiles apart",
            near.name,
            if d == 1 { "tile" } else { "tiles" },
            state
                .setup
                .rule_experiments
                .min_city_gap(state.player(owner).faction)
                + 1,
        ));
    }

    Err("Cannot found a city here".into())
}

/// Found a city with this unit if legal (consumes the unit). The first city is the capital.
pub fn found_city(
    state: &mut GameState,
    settler: UnitId,
    events: &mut Vec<Event>,
) -> Result<CityId, String> {
    check_found_city(state, settler)?;
    let unit = &state.units[&settler];
    let (owner, pos) = (unit.owner, unit.pos);
    let military_founder = unit.kind != UnitKind::Settler;
    let legionary = military_founder
        && unit.kind == UnitKind::Phalangite
        && !state
            .setup
            .rule_experiments
            .rule_off(crate::experiments::FactionRule::RomeLegionary);
    state.units.remove(&settler);
    Ok(found_city_at(
        state,
        owner,
        pos,
        military_founder,
        legionary,
        events,
    ))
}

/// The shared founding effects for an ordered founder and a map-generated starting capital.
pub(crate) fn found_city_at(
    state: &mut GameState,
    owner: PlayerId,
    pos: Hex,
    military_founder: bool,
    legionary: bool,
    events: &mut Vec<Event>,
) -> CityId {
    let name = next_city_name(state, owner);
    let is_capital = state.capital_of(owner).is_none();
    let original_capital = is_capital
        && !state.cities.values().any(|c| c.founder == owner)
        && !state.history.chronicle.iter().any(|entry| {
            matches!(entry,
            crate::history::Chronicle::Founded { player, .. } if *player == owner)
        });
    let id = state.alloc_city_id();
    state.cities.insert(
        id,
        City {
            last_attacked_turn: None,
            id,
            owner,
            founder: owner,
            name: name.clone(),
            pos,
            population: if legionary { 2 } else { 1 },
            food_stored: 0,
            production_stored: 0,
            producing: None,
            bought: None,
            production_queue: Vec::new(),
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_banked: Vec::new(),
            production_queue_notified: Default::default(),
            wonder_lost: None,
            last_purchase_turn: None,
            buildings: BTreeSet::new(),
            hp: CITY_HP,
            tiles_claimed: 0,
            grown_tiles: Vec::new(),
            is_capital,
            original_capital,
            greece_captive_paid: BTreeSet::new(),
            occupied_from: None,
            founded_turn: state.turn,
            resistance: 0,
        },
    );
    // The centre is always the new city's (it may have been another own city's grown tile);
    // ring 1 takes only unowned tiles.
    if let Some(previous) = state.map.get(pos).and_then(|t| t.city)
        && let Some(other) = state.cities.get_mut(&previous)
        && other.grown_tiles.contains(&pos)
    {
        other.grown_tiles.retain(|h| *h != pos);
        other.tiles_claimed = (other.tiles_claimed - 1).max(0);
    }
    for h in pos.within(1) {
        if let Some(t) = state.map.get_mut(h)
            && (h == pos || t.owner.is_none())
        {
            t.owner = Some(owner);
            t.city = Some(id);
            // A city on an improved tile replaces the improvement.
            t.improved &= h != pos;
        }
    }
    events.push(Event::CityFounded {
        city: id,
        owner,
        name,
        at: pos,
        veteran: military_founder,
    });
    if legionary {
        settle_territory(state, id, events);
    }
    if state.player(owner).techs.contains(&TechId::CodeOfLaws) {
        claim_code_of_laws_tile(state, id, events);
    }
    id
}

/// Where a newly completed unit appears: the city tile, else the first free adjacent passable
/// tile in hex order. A tile is free when it holds no enemy city, no unit of another team and no
/// unit of the same class (military / civilian) as the new one.
pub fn spawn_tile(state: &GameState, owner: PlayerId, center: Hex, kind: UnitKind) -> Option<Hex> {
    // Aircraft never stand on a tile: they need room in the city's hangar (`rules::air`).
    if data::is_aircraft(kind) {
        return state
            .city_at(center)
            .filter(|c| super::air::hangar_blocker(state, c.id).is_none())
            .map(|_| center);
    }
    if can_spawn_at(state, owner, center, kind) {
        return Some(center);
    }
    spawn_neighbour_tile(state, owner, center, kind)
}

/// Ruin gifts always step out beside the finder, even when the finder is a civilian.
pub fn spawn_neighbour_tile(
    state: &GameState,
    owner: PlayerId,
    center: Hex,
    kind: UnitKind,
) -> Option<Hex> {
    let mut adjacent = center.neighbors().to_vec();
    adjacent.sort();
    adjacent
        .into_iter()
        .find(|h| can_spawn_at(state, owner, *h, kind))
}

fn can_spawn_at(state: &GameState, owner: PlayerId, at: Hex, kind: UnitKind) -> bool {
    let military = data::unit_stats(kind).military;
    state.map.move_cost(at).is_some()
        && state.passage_allowed(owner, kind, state.map.get(at).and_then(|t| t.owner))
        && state
            .city_at(at)
            .is_none_or(|c| state.same_team(c.owner, owner))
        && state.units_at(at).all(|u| {
            state.same_team(u.owner, owner) && data::unit_stats(u.kind).military != military
        })
}

/// Growth, starvation, production (spawning units), gold income and upkeep, city healing.
/// City yields are taken from the start of the phase; upkeep includes newly produced units.
/// A city in resistance produces nothing; its counter ticks down at the end of
/// `process_borders`.
pub fn process_cities(state: &mut GameState, events: &mut Vec<Event>) {
    let incomes: Vec<(PlayerId, PlayerIncome)> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| (p.id, economy::player_income(state, p.id)))
        .collect();
    let yields: BTreeMap<CityId, Yields> = state
        .cities
        .values()
        .map(|c| (c.id, economy::city_totals(state, c)))
        .collect();

    for id in &wonder_race_order(state, &yields) {
        let y = &yields[id];
        if state.cities[id].resistance > 0 {
            continue;
        }
        // All cities use the same pre-growth yield snapshot. Egypt diverts only this turn's
        // positive surplus, so stored Food cannot buy a building and deficits still starve.
        let egypt = state.cities[id]
            .producing
            .is_some_and(|item| egypt_building(state, &state.cities[id], item));
        let gate_open = state.cities[id].producing.is_some_and(|item| {
            !egypt || economy::production_blocker(state, &state.cities[id], item, false).is_none()
        });
        let surplus = (y.food - FOOD_PER_POP * state.cities[id].population).max(0);
        if egypt && gate_open && y.food < FOOD_PER_POP * state.cities[id].population {
            // Deficits still consume reserves and starve, but never grow a city on a
            // construction turn even if its old Food reserve passed a threshold.
            let c = state.cities.get_mut(id).expect("city exists");
            let balance = c
                .food_stored
                .saturating_add(y.food - FOOD_PER_POP * c.population);
            c.food_stored = balance.max(0);
            if balance < 0 && c.population > 1 {
                c.population -= 1;
                events.push(Event::CityStarved {
                    city: *id,
                    owner: c.owner,
                    population: c.population,
                });
            }
        } else if !egypt || !gate_open {
            grow(state, *id, y.food, events);
        }
        // Occupation lock: a frozen city's Production is lost, not banked.
        if super::occupation::city_freeze(state, &state.cities[id]).is_none()
            && (!egypt || (gate_open && surplus > 0))
        {
            produce(
                state,
                *id,
                if egypt {
                    surplus.saturating_mul(2)
                } else {
                    y.production
                },
                events,
            );
        }
    }

    // Power is recorded after everyone has paid, so no empire's bill depends on turn order.
    let powers: Vec<(PlayerId, i32)> = incomes
        .iter()
        .map(|(p, income)| {
            let y = income.cities;
            (
                *p,
                y.food
                    .saturating_add(y.production)
                    .saturating_add(y.gold)
                    .saturating_add(y.science),
            )
        })
        .collect();
    for (p, income) in incomes {
        // City's remittance is fixed by its pre-growth yields. Newly completed troops still
        // pay upkeep this turn, after production has added them to the army.
        let new_upkeep = i64::from(economy::upkeep(state, p)) - i64::from(income.upkeep);
        pay_gold(state, p, i64::from(income.gold.total) - new_upkeep, events);
        let history_turns = crate::data::SCIENCE_HISTORY_TURNS;
        let history = &mut state.player_mut(p).science_history;
        history.push(income.science.total.clamp(0, crate::validation::MAX_STORED));
        let excess = history.len().saturating_sub(history_turns);
        history.drain(..excess);
    }
    let window = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    // Power remains public for AI economic threat estimates and optional decadence.
    for (p, total) in powers {
        let history = &mut state.player_mut(p).power_history;
        history.push(total.clamp(0, crate::validation::MAX_STORED));
        let excess = history.len().saturating_sub(window);
        history.drain(..excess);
    }
    crate::scoring::refresh_decadence(state);

    heal_cities(state);
}

/// The order cities produce in this phase: by city id, except that cities about to finish the
/// same world wonder this turn go in the order that settles the race (otherwise the lower city
/// id would always win it). Most overflow Production first; an exact tie is drawn
/// (`purpose::WONDER_RACE`). Only the racing cities swap places among themselves.
fn wonder_race_order(state: &GameState, yields: &BTreeMap<CityId, Yields>) -> Vec<CityId> {
    let mut order: Vec<CityId> = yields.keys().copied().collect();
    let mut races: BTreeMap<BuildingKind, Vec<(CityId, i32)>> = BTreeMap::new();
    for (id, y) in yields {
        let c = &state.cities[id];
        if c.resistance > 0 || super::occupation::city_freeze(state, c).is_some() {
            continue;
        }
        let Some(
            item
            @ (ProductionItem::Building { building } | ProductionItem::Wonder { building, .. }),
        ) = c.producing
        else {
            continue;
        };
        if !data::is_great_work(building) {
            continue;
        }
        // The Production `process_cities` will add (Egypt builds from surplus Food).
        let production = if egypt_building(state, c, item) {
            let surplus = (y.food - FOOD_PER_POP * c.population).max(0);
            if surplus == 0 || economy::production_blocker(state, c, item, false).is_some() {
                continue;
            }
            surplus.saturating_mul(2)
        } else {
            y.production
        };
        let overflow = crate::validation::add_stored(c.production_stored, production)
            - economy::city_production_cost(state, c, item);
        if overflow >= 0 {
            races.entry(building).or_default().push((*id, overflow));
        }
    }
    for (building, racers) in races.into_iter().filter(|(_, r)| r.len() > 1) {
        let ids: Vec<CityId> = racers.iter().map(|(id, _)| *id).collect();
        let drawn = rng::seeded_order(
            state.dice_seed(),
            &[
                u64::from(state.turn),
                rng::purpose::WONDER_RACE,
                building as u64,
            ],
            &ids,
        );
        let mut ranked: Vec<(CityId, i32)> = drawn
            .iter()
            .map(|id| *racers.iter().find(|(r, _)| r == id).expect("racer"))
            .collect();
        // Stable: equal overflow keeps the drawn order.
        ranked.sort_by_key(|(_, overflow)| std::cmp::Reverse(*overflow));
        let mut slots: Vec<usize> = ids
            .iter()
            .map(|id| order.iter().position(|o| o == id).expect("city in order"))
            .collect();
        slots.sort_unstable();
        for (slot, (id, _)) in slots.into_iter().zip(ranked) {
            order[slot] = id;
        }
    }
    order
}

pub(crate) fn grow(state: &mut GameState, id: CityId, food: i32, events: &mut Vec<Event>) {
    let threshold = economy::growth_threshold(state, &state.cities[&id]).total;
    let c = state.cities.get_mut(&id).expect("city exists");
    events.extend(grow_city(c, food, threshold));
}

/// One active city's growth phase, shared with forecasts. Callers skip resisting cities.
pub(crate) fn grow_city(c: &mut City, food: i32, threshold: i32) -> Option<Event> {
    let food_total = i64::from(c.food_stored) + i64::from(food)
        - i64::from(FOOD_PER_POP) * i64::from(c.population);
    c.food_stored = food_total.clamp(0, i64::from(crate::validation::MAX_STORED)) as i32;
    if c.food_stored >= threshold && c.population < crate::validation::MAX_POPULATION {
        c.food_stored -= threshold;
        c.population += 1;
        return Some(Event::CityGrew {
            city: c.id,
            owner: c.owner,
            population: c.population,
        });
    } else if food_total < 0 && c.population > 1 {
        c.population -= 1;
        return Some(Event::CityStarved {
            city: c.id,
            owner: c.owner,
            population: c.population,
        });
    }
    None
}

fn announce_start(
    city: CityId,
    owner: PlayerId,
    item: Option<ProductionItem>,
    events: &mut Vec<Event>,
) {
    if let Some(ProductionItem::Building { building } | ProductionItem::Wonder { building, .. }) =
        item
        && data::is_great_work(building)
    {
        events.push(Event::MegaprojectStarted {
            city,
            owner,
            building,
        });
    }
}

/// Visit each queued item once. Temporary placement failures retain their relative order.
fn advance_queue(state: &mut GameState, id: CityId, events: &mut Vec<Event>) {
    let mut skipped = Vec::new();
    let count = state.cities[&id].production_queue.len();
    for _ in 0..count {
        if state.cities[&id].producing.is_some() {
            break;
        }
        let item = state
            .cities
            .get_mut(&id)
            .expect("city exists")
            .production_queue
            .remove(0);
        let automated = state.cities[&id].auto_queued == Some(item);
        let c = &state.cities[&id];
        let invalid = economy::production_blocker(state, c, item, true);
        let temporary = economy::placement_blocker(state, c, item)
            .or_else(|| crate::resources::start_blocker(state, c, item));
        if let Some(reason) = invalid.as_ref().or(temporary.as_ref()) {
            if invalid.is_some() || !c.production_queue_notified.contains(&item) {
                events.push(Event::OrderFailed {
                    player: c.owner,
                    reason: format!(
                        "{} skipped queued {}: {}{}",
                        c.name,
                        item_name(item),
                        reason.trim_end_matches('.'),
                        if invalid.is_none() {
                            "; kept in queue"
                        } else {
                            ""
                        }
                    ),
                });
            }
            if invalid.is_none() {
                skipped.push(item);
                state
                    .cities
                    .get_mut(&id)
                    .expect("city exists")
                    .production_queue_notified
                    .insert(item);
            } else if automated {
                state.cities.get_mut(&id).expect("city exists").auto_queued = None;
            }
        } else {
            let egypt = egypt_building(state, c, item);
            announce_start(id, c.owner, Some(item), events);
            if automated {
                state.cities.get_mut(&id).expect("city exists").auto_queued = None;
            }
            // Unassigned Production (overflow) joins the item's own banked progress.
            state
                .cities
                .get_mut(&id)
                .expect("city exists")
                .switch_production_for(Some(item), egypt);
        }
    }
    let c = state.cities.get_mut(&id).expect("city exists");
    skipped.append(&mut c.production_queue);
    c.production_queue = skipped;
    c.production_queue_notified
        .retain(|item| c.production_queue.contains(item));
}

/// A manually selected build may wait for population; with a queue it must not starve later work.
fn defer_blocked_head(state: &mut GameState, id: CityId, events: &mut Vec<Event>) {
    let c = &state.cities[&id];
    let Some(item) = c.producing else {
        return;
    };
    if c.production_queue.is_empty() || economy::placement_blocker(state, c, item).is_none() {
        return;
    }
    let c = state.cities.get_mut(&id).expect("city exists");
    c.switch_production(None);
    c.production_queue.insert(0, item);
    advance_queue(state, id, events);
}

/// Complete a separate item, without consuming or replacing the city's active production.
fn complete_item(
    state: &mut GameState,
    id: CityId,
    item: ProductionItem,
    events: &mut Vec<Event>,
) -> bool {
    let c = &state.cities[&id];
    let owner = c.owner;
    let war_shrine = c.buildings.contains(&BuildingKind::WarShrine);
    match item {
        ProductionItem::Unit { unit } => {
            if unit == UnitKind::Settler && c.population < SETTLER_MIN_POP {
                return false;
            }
            if unit == UnitKind::Heir && crate::goals::heir(state, owner).is_some() {
                return false;
            }
            if data::is_aircraft(unit) {
                if super::air::hangar_blocker(state, id).is_some() {
                    return false;
                }
                super::air::base_new(state, id, unit);
                events.push(Event::ProductionCompleted {
                    city: id,
                    owner,
                    item,
                });
                return true;
            }
            let Some(tile) = spawn_tile(state, owner, c.pos, unit) else {
                return false;
            };
            let uid = game::spawn_unit(state, owner, unit, tile);
            if unit == UnitKind::Heir {
                state.units.get_mut(&uid).expect("just spawned").heir_of = Some(owner);
                events.push(Event::HeirBorn {
                    city: id,
                    owner,
                    city_name: state.cities[&id].name.clone(),
                    at: tile,
                });
            }
            if data::unit_stats(unit).military
                && state.player(owner).faction == Faction::Sparta
                && war_shrine
            {
                let xp = crate::rules::promotion::threshold(2).unwrap();
                let u = state.units.get_mut(&uid).expect("just spawned");
                u.xp = u.xp.max(xp);
            }
            state.units.get_mut(&uid).expect("just spawned").moves_left = 0;
            if unit == UnitKind::Settler {
                state.cities.get_mut(&id).expect("city exists").population -= 1;
                state.player_mut(owner).settlers_trained += 1;
            }
        }
        ProductionItem::Project { .. } => return false,
        ProductionItem::Improvement { improvement, tile } => {
            if crate::improvements::blocker(state, &state.cities[&id], improvement, tile).is_some()
            {
                return false;
            }
            crate::improvements::complete(state, tile);
        }
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            economy::install_building(state.cities.get_mut(&id).expect("city exists"), building);
            if let ProductionItem::Wonder { tile, .. } = item {
                crate::landmarks::complete(state, id, building, tile);
            }
            if building == BuildingKind::CourierHall {
                state.player_mut(owner).hall_city = Some(id);
            }
            if building == BuildingKind::ImperialDiwan {
                state.player_mut(owner).diwan_city = Some(id);
            }
            if data::is_great_work(building) {
                state.player_mut(owner).great_work_completed = Some(building);
                resolve_wonder_race(state, id, owner, building, events);
            }
        }
    }
    events.push(Event::ProductionCompleted {
        city: id,
        owner,
        item,
    });
    true
}

fn resolve_wonder_race(
    state: &mut GameState,
    winning_city: CityId,
    winner: PlayerId,
    building: BuildingKind,
    events: &mut Vec<Event>,
) {
    let mut refunds = Vec::new();
    let turn = state.turn;
    for city in state.cities.values_mut().filter(|c| c.id != winning_city) {
        let matches = |item: ProductionItem| {
            matches!(item,
            ProductionItem::Building { building: b } | ProductionItem::Wonder { building: b, .. }
            if b == building)
        };
        let active = city.producing.is_some_and(matches);
        let queued = city.production_queue.iter().any(|item| matches(*item));
        let banked = city
            .production_banked
            .iter()
            .any(|entry| matches(entry.item));
        if !active && !queued && !banked {
            continue;
        }
        if active {
            city.switch_production(None);
        }
        city.production_queue.retain(|item| !matches(*item));
        city.production_queue_notified
            .retain(|item| !matches(*item));
        let mut production = 0;
        city.production_banked.retain(|entry| {
            if matches(entry.item) {
                production += entry.progress;
                false
            } else {
                true
            }
        });
        let gold = production / 2;
        let owner = city.owner;
        let city_id = city.id;
        // The city that was building it stops and asks its owner, whatever the resolution order.
        if active {
            city.wonder_lost = Some(WonderLoss {
                turn,
                winner,
                building,
                production,
                gold,
            });
        }
        refunds.push((owner, gold));
        events.push(Event::WonderRaceLost {
            city: city_id,
            owner,
            winner,
            building,
            production,
            gold,
            active,
        });
    }
    for (owner, gold) in refunds {
        let player = state.player_mut(owner);
        player.gold =
            (player.gold as i64 + gold as i64).min(crate::validation::MAX_GOLD as i64) as i32;
    }
}

/// Banked progress for something this city can never build again: an item useless to its
/// owner's faction (`data::available_to_faction`, e.g. after capture or from older saves), a
/// building already standing (or a higher tier of its family), or a Great Work once the
/// empire completed its one. Units never become obsolete (`production_blocker` checks only their tech and the
/// Settler's population), so a unit's bank never goes stale.
pub fn bank_is_stale(state: &GameState, c: &City, item: ProductionItem) -> bool {
    let owner = state.player(c.owner);
    if !state
        .setup
        .rule_experiments
        .available_to_player(owner.faction, state.era_of(c.owner), item)
    {
        return true;
    }
    match item {
        ProductionItem::Building { building } => {
            data::has_building(&c.buildings, building)
                || (data::is_great_work(building) && economy::great_work_done(state, c.owner))
        }
        ProductionItem::Wonder { building, tile } => {
            crate::landmarks::check::<economy::Blocked>(state, c, building, tile).is_some()
                || data::has_building(&c.buildings, building)
                || (data::is_great_work(building) && economy::great_work_done(state, c.owner))
        }
        ProductionItem::Unit { .. } => false,
        ProductionItem::Project { .. } => true,
        ProductionItem::Improvement { improvement, tile } => {
            crate::improvements::blocker(state, c, improvement, tile).is_some()
        }
    }
}

/// Drop banked progress that can never be used (see `bank_is_stale`).
fn prune_stale_banks(state: &mut GameState, id: CityId) {
    let c = &state.cities[&id];
    let stale: Vec<ProductionItem> = c
        .production_banked
        .iter()
        .map(|b| b.item)
        .filter(|item| bank_is_stale(state, c, *item))
        .collect();
    let c = state.cities.get_mut(&id).expect("city exists");
    for item in stale {
        c.take_banked(item);
    }
}

pub(crate) fn produce(state: &mut GameState, id: CityId, production: i32, events: &mut Vec<Event>) {
    // A city that lost a wonder race this turn holds its Production for its owner's choice
    // instead of starting its queue; from its next production phase the queue resumes.
    let stopped = match state.cities[&id].wonder_lost {
        Some(loss) => loss.turn == state.turn,
        None => false,
    };
    if !stopped {
        state.cities.get_mut(&id).expect("city exists").wonder_lost = None;
    }
    prune_stale_banks(state, id);
    if !stopped {
        advance_queue(state, id, events);
    }
    // The current build can become impossible (a Settler's city dropped below population 2).
    // Its progress stays banked with it, and the city asks for a new choice.
    while let Some(item) = state.cities[&id].producing {
        let c = &state.cities[&id];
        let Some(reason) = economy::production_blocker(state, c, item, false) else {
            break;
        };
        let kept = if c.production_stored > 0 && !bank_is_stale(state, c, item) {
            format!("; its {} Production stays banked", c.production_stored)
        } else {
            String::new()
        };
        events.push(Event::OrderFailed {
            player: c.owner,
            reason: format!(
                "{} can no longer produce {}: {reason}{kept}",
                c.name,
                item_name(item)
            ),
        });
        let c = state.cities.get_mut(&id).expect("city exists");
        c.switch_production(None);
        prune_stale_banks(state, id);
        advance_queue(state, id, events);
    }
    defer_blocked_head(state, id, events);
    let cap = bank_cap(state, &state.cities[&id]);
    let c = state.cities.get_mut(&id).expect("city exists");
    let Some(item) = c.producing else {
        if c.production_stored < cap {
            c.production_stored =
                crate::validation::add_stored(c.production_stored, production).min(cap);
        }
        return;
    };
    if matches!(item, ProductionItem::Project { .. }) {
        return;
    }
    c.production_stored = crate::validation::add_stored(c.production_stored, production);
    let cost = economy::city_production_cost(state, &state.cities[&id], item);
    if state.cities[&id].production_stored < cost || !complete_item(state, id, item, events) {
        return;
    }
    let egypt = egypt_building(state, &state.cities[&id], item);
    let c = state.cities.get_mut(&id).expect("city exists");
    if egypt {
        c.production_stored = 0; // Food construction never overflows to the next item.
    } else {
        c.production_stored -= cost;
    }
    c.producing = None;
    c.bought = None;
    // Existing overflow rule: at most one natural completion per city per turn.
    advance_queue(state, id, events);
}

/// Add the turn's net gold. A negative treasury disbands the costliest military unit (ties: the
/// highest id) and resets gold to 0.
fn pay_gold(state: &mut GameState, p: PlayerId, net: i64, events: &mut Vec<Event>) {
    let player = state.player_mut(p);
    let total = i64::from(player.gold) + net;
    if total >= 0 {
        player.gold = total.min(i64::from(crate::validation::MAX_GOLD)) as i32;
        return;
    }
    let debt = total;
    player.gold = 0;
    if let Some(unit) = bankruptcy_victim(state, p) {
        if crate::company::is_company(&state.units[&unit]) {
            crate::company::turn_barbarian(state, unit, events);
            return;
        }
        state.units.remove(&unit);
        events.push(Event::UnitDisbanded {
            unit,
            owner: p,
            reason: format!("Treasury ran dry ({debt} Gold): could not pay unit upkeep"),
        });
    }
}

/// The same cost and id ordering is used by the forecast and by resolution.
pub(crate) fn bankruptcy_victim(state: &GameState, p: PlayerId) -> Option<UnitId> {
    // An unpaid Company goes first (the Catalan Grand Company): the dearest, then the highest id.
    state
        .units_of(p)
        .filter(|u| data::pays_upkeep(u.kind))
        .max_by_key(|u| {
            (
                crate::company::is_company(u),
                state.unit_stats_of(u).cost,
                u.id,
            )
        })
        .map(|u| u.id)
}

/// True when an at-war military unit stands next to the city: it regains no HP while one does.
/// Barbarians never count against the Celts (`rules::is_threat`): they cannot besiege them.
/// Shared by [`heal_cities`] and the city card's forecast.
pub(crate) fn heal_blocked_by_adjacent_enemy(state: &GameState, c: &City) -> bool {
    state.units.values().any(|u| {
        u.pos.distance(c.pos) == 1
            && data::unit_stats(u.kind).military
            && super::is_threat(state, c.owner, u.owner)
    })
}

/// A city regains `CITY_HEAL` HP at the end of a turn (up to `CITY_HP`) unless it took damage
/// this turn (`last_attacked_turn`, set when the battle phase hits it) or an at-war military
/// unit is adjacent. Battles resolve before this runs, so `state.turn` is still the turn they
/// happened on.
fn heal_cities(state: &mut GameState) {
    let ids: Vec<CityId> = state.cities.keys().copied().collect();
    for id in ids {
        let c = &state.cities[&id];
        if c.hp >= CITY_HP
            || c.last_attacked_turn == Some(state.turn)
            || heal_blocked_by_adjacent_enemy(state, c)
        {
            continue;
        }
        let c = state.cities.get_mut(&id).expect("city exists");
        c.hp = (c.hp + CITY_HEAL).min(CITY_HP);
    }
}

/// Science accumulation and tech completion. This turn's science is the value `process_cities`
/// recorded in `science_history`. Costs (with diffusion) are fixed before anyone completes a tech
/// this turn, so the result doesn't depend on player order.
pub fn process_research(state: &mut GameState, events: &mut Vec<Event>) {
    // (player, science this turn, researching tech and its cost)
    type Plan = (PlayerId, i32, Option<(TechId, i32)>);
    let plans: Vec<Plan> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            let science = p.science_history.last().copied().unwrap_or(0);
            let target = p
                .researching
                .map(|t| (t, economy::tech_cost(state, p.id, t).total));
            (p.id, science, target)
        })
        .collect();
    for (id, science, target) in plans {
        let player = state.player_mut(id);
        if crate::eras::all_researched(&player.techs) {
            continue;
        }
        player.science_stored = crate::validation::add_stored(player.science_stored, science);
        let Some((tech, cost)) = target else { continue };
        let already_known = player.techs.contains(&tech);
        if !already_known {
            let ruins: i32 = events
                .iter()
                .filter_map(|e| match e {
                    Event::RuinExplored {
                        owner,
                        reward: crate::events::RuinReward::Science { amount, .. },
                        ..
                    } if *owner == id => Some(*amount),
                    _ => None,
                })
                .sum();
            let mut progress = player.science_stored - ruins;
            let turns = |progress: i32| {
                let left = (cost - progress).max(0);
                if left == 0 {
                    Some(0)
                } else if science > 0 {
                    Some((left + science - 1) / science)
                } else {
                    None
                }
            };
            for event in events.iter_mut() {
                if let Event::RuinExplored {
                    owner,
                    reward: crate::events::RuinReward::Science { amount, research },
                    ..
                } = event
                    && *owner == id
                {
                    *research = Some(crate::events::RuinResearch {
                        tech,
                        cost,
                        progress_before: progress,
                        income: science,
                        turns_saved: turns(progress)
                            .zip(turns(progress + *amount))
                            .map(|(a, b)| a - b),
                        completes: progress < cost && progress + *amount >= cost,
                    });
                    progress += *amount;
                }
            }
        }
        if !already_known && player.science_stored >= cost {
            player.science_stored -= cost;
            crate::eras::learn(state, id, tech, events);
        }
        let player = state.player_mut(id);
        if player.techs.contains(&tech) {
            player.research_queue.retain(|t| !player.techs.contains(t));
            player.researching = player.research_queue.first().copied();
            events.push(Event::ResearchQueueAdvanced {
                player: id,
                next: player.researching,
            });
        }
    }
    let ids: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect();
    for id in ids {
        normalize_research(state, id);
    }
}

/// Keeps a plan legal after a completed technology advances the era gate.
fn normalize_research(state: &mut GameState, id: PlayerId) {
    let player = state.player_mut(id);
    let techs = player.techs.clone();
    let era = crate::eras::current_era(&techs);
    let legal = |t: &TechId| !techs.contains(t) && crate::eras::era_of_tech(*t) == era;
    let mut seen = BTreeSet::new();
    player
        .research_queue
        .retain(|t| legal(t) && seen.insert(*t));
    if player.researching.is_some_and(|t| !legal(&t)) {
        player.researching = player.research_queue.first().copied();
    }
}

/// Gold a bought tile costs before its ring surcharge: `TILE_GOLD_BASE` plus `TILE_GOLD_PER_TILE`
/// per tile this city has gained since founding (grown or bought).
pub fn tile_base_price(city: &City) -> i32 {
    TILE_GOLD_BASE + TILE_GOLD_PER_TILE * city.tiles_claimed
}

/// Percent of the base price a tile costs by its ring (hex distance from the city): 100 at ring
/// 1, plus `TILE_RING_STEP_PERCENT` per ring beyond.
pub fn tile_ring_percent(ring: i32) -> i32 {
    100 + TILE_RING_STEP_PERCENT * (ring - 1).max(0)
}

/// Gold price of buying `tile` for this city: the base price (`tile_base_price`) times the ring
/// percent (`tile_ring_percent`), one integer division rounding down.
pub fn tile_price(city: &City, tile: Hex) -> i32 {
    tile_base_price(city) * tile_ring_percent(tile.distance(city.pos)) / 100
}

/// Unowned tiles this city may buy: claim candidates within its work radius (tiles beyond it
/// only come from growth). Any ring order; farther rings cost more.
pub fn buy_candidates(state: &GameState, city: &City) -> Vec<Hex> {
    buy_candidates_among(city, &claim_candidates(state, city))
}

/// `buy_candidates` from the city's `claim_candidates`.
pub fn buy_candidates_among(city: &City, claims: &[Hex]) -> Vec<Hex> {
    claims
        .iter()
        .copied()
        .filter(|h| h.distance(city.pos) <= CITY_WORK_RADIUS)
        .collect()
}

/// Unowned tiles this city could claim: anything touching its territory (no reach cap), in hex
/// order.
pub fn claim_candidates(state: &GameState, city: &City) -> Vec<Hex> {
    let mut out = BTreeSet::new();
    for (i, t) in state.map.tiles.iter().enumerate() {
        if t.city != Some(city.id) {
            continue;
        }
        for n in state.map.hex_at(i).neighbors() {
            if state.map.get(n).is_some_and(|nt| nt.owner.is_none()) {
                out.insert(n);
            }
        }
    }
    out.into_iter().collect()
}

/// `claim_candidates` from the city's territory already gathered (every tile whose `city`
/// is this city, any order).
pub fn claim_candidates_from(state: &GameState, city: &City, territory: &[Hex]) -> Vec<Hex> {
    let mut out = BTreeSet::new();
    for h in territory {
        for n in h.neighbors() {
            if state.map.get(n).is_some_and(|nt| nt.owner.is_none()) {
                out.insert(n);
            }
        }
    }
    let out: Vec<Hex> = out.into_iter().collect();
    debug_assert_eq!(
        out,
        claim_candidates(state, city),
        "territory claim candidates"
    );
    out
}

/// How much this city wants `h` as its next claim: the tile's yields by the city's focus
/// (whether or not it is worked, so tiles beyond the work radius rank by quality too), plus the
/// pull toward known unclaimed strategic deposits.
pub fn claim_score(state: &GameState, city: &City, focus: CityFocus, h: Hex) -> i32 {
    let Some(t) = state.map.get(h) else {
        return 0;
    };
    economy::tile_score(economy::owner_tile_potential(state, city.owner, t), focus)
        + CLAIM_PULL_WEIGHT * crate::resources::border_pull(state, city.owner, city.pos, h)
        + if t.wonder.is_some() { 200 } else { 0 }
}

/// The tile this city claims at its next growth, nearest first with quality inside the ring:
/// every claimable tile of the current ring (the smallest hex distance from the city among its
/// candidates) goes before any tile of the next one; within a ring the best by the city's
/// current focus (`claim_score`), ties in hex order (q, then r).
pub fn next_claim(state: &GameState, city: &City) -> Option<Hex> {
    next_claim_among(state, city, claim_candidates(state, city))
}

/// `next_claim` from the city's `claim_candidates`.
pub fn next_claim_among(state: &GameState, city: &City, candidates: Vec<Hex>) -> Option<Hex> {
    // The Nile: an Egyptian city has no use for Production, so it claims for Food.
    let focus = if state.player(city.owner).faction == Faction::Egypt
        && state.setup.rule_experiments.egypt_nile()
    {
        crate::CityFocus::Growth
    } else {
        economy::auto_focus(city, city.producing)
    };
    let ring = candidates.iter().map(|h| h.distance(city.pos)).min()?;
    candidates
        .into_iter()
        .filter(|h| h.distance(city.pos) == ring)
        .max_by_key(|h| (claim_score(state, city, focus, *h), std::cmp::Reverse(*h)))
}

/// Code of Laws grants one permanent ring-2 claim, including to cities already founded.
pub(crate) fn claim_code_of_laws_tile(state: &mut GameState, id: CityId, events: &mut Vec<Event>) {
    let city = &state.cities[&id];
    let candidates = claim_candidates(state, city)
        .into_iter()
        .filter(|h| h.distance(city.pos) == 2)
        .collect();
    let Some(tile) = next_claim_among(state, city, candidates) else {
        return;
    };
    let owner = city.owner;
    let t = state.map.get_mut(tile).expect("on map");
    t.owner = Some(owner);
    t.city = Some(id);
    events.push(Event::TileClaimed {
        city: id,
        owner,
        tile,
    });
}

/// The grown tile this city gives up when it loses population: the worst unworked one (beyond
/// `CITY_WORK_RADIUS`, so nothing is lost but the block), else its most recent claim. Ties go
/// to the more recent claim.
pub fn next_release(state: &GameState, city: &City) -> Option<Hex> {
    let held: Vec<(usize, Hex)> = city
        .grown_tiles
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, h)| state.map.get(*h).is_some_and(|t| t.city == Some(city.id)))
        .filter(|(_, h)| {
            !state.map.get(*h).is_some_and(|t| {
                t.landmark
                    .is_some_and(|site| data::is_great_work(site.building))
            })
        })
        .collect();
    let keep_value = |h: Hex| {
        let t = state.map.get(h).expect("held tile on map");
        let strategic = t.resource.is_some_and(|r| {
            crate::resources::is_strategic(r)
                && crate::resources::known_to(r, &state.player(city.owner).techs)
        });
        economy::tile_score(
            economy::owner_tile_potential(state, city.owner, t),
            CityFocus::Balanced,
        ) + if strategic { 100 } else { 0 }
    };
    held.iter()
        .filter(|(_, h)| h.distance(city.pos) > CITY_WORK_RADIUS)
        .min_by_key(|(i, h)| (keep_value(*h), std::cmp::Reverse(*i)))
        .or(held.last())
        .map(|(_, h)| *h)
}

/// Why this city cannot buy any tile now (the shared purchase slot, the occupation freeze,
/// resistance), or None. The same checks as `economy::buy_check` makes for an item.
pub fn buy_tile_slot_check<R: crate::economy::Reason>(state: &GameState, city: &City) -> Option<R> {
    if city.last_purchase_turn == Some(state.turn) {
        return Some(R::word(|| {
            format!("{} already bought something this turn", city.name)
        }));
    }
    if let Some(reason) = crate::rules::occupation::city_refusal(state, city) {
        return Some(R::word(|| reason));
    }
    if city.resistance > 0 {
        return Some(R::word(|| format!("{} is resisting occupation", city.name)));
    }
    None
}

/// Why this city can't buy `tile` now, or None. Shares the one-purchase-per-turn slot with
/// BuyItem and HireCompany; the tile must be explored by the buyer.
pub fn buy_tile_blocker(state: &GameState, city: &City, tile: Hex) -> Option<String> {
    buy_tile_check(state, city, tile)
}

/// `buy_tile_blocker`, worded or not (`economy::Reason`).
pub fn buy_tile_check<R: crate::economy::Reason>(
    state: &GameState,
    city: &City,
    tile: Hex,
) -> Option<R> {
    buy_tile_check_among(state, city, tile, || buy_candidates(state, city))
}

/// `buy_tile_check` with the city's `buy_candidates` given (computed only if needed).
pub fn buy_tile_check_among<R: crate::economy::Reason>(
    state: &GameState,
    city: &City,
    tile: Hex,
    candidates: impl FnOnce() -> Vec<Hex>,
) -> Option<R> {
    if let Some(block) = buy_tile_slot_check::<R>(state, city) {
        return Some(block);
    }
    let explored = state
        .map
        .index(tile)
        .is_some_and(|i| state.player(city.owner).explored.get(i));
    if !explored {
        return Some(R::word(|| {
            "Explore a tile before buying it: only an explored tile can be bought".into()
        }));
    }
    if state.map.get(tile).is_some_and(|t| t.owner.is_some()) {
        return Some(R::word(|| {
            "That tile is already owned: a tile can be bought only once, by one city".into()
        }));
    }
    if !candidates().contains(&tile) {
        return Some(R::word(|| {
            format!(
                "{} can only buy an unowned tile next to its borders, within {CITY_WORK_RADIUS} tiles; farther tiles come only from growth",
                city.name
            )
        }));
    }
    let price = tile_price(city, tile);
    let gold = state.player(city.owner).gold;
    if gold < price {
        return Some(R::word(|| {
            format!(
                "Not enough Gold: the tile costs {price} Gold; you have {gold} (need {} more)",
                price - gold
            )
        }));
    }
    None
}

/// Give `city` a tile (growth or purchase): ownership and the tile counter. The caller records
/// a growth claim in `grown_tiles`; a bought tile is never one.
fn claim_tile(state: &mut GameState, id: CityId, tile: Hex) {
    let owner = state.cities[&id].owner;
    crate::landmarks::attach_city(state, tile, id);
    let c = state.cities.get_mut(&id).expect("city exists");
    c.tiles_claimed += 1;
    let t = state.map.get_mut(tile).expect("on map");
    t.owner = Some(owner);
    t.city = Some(id);
}

/// Population is territory: bring this city's grown tiles in line with its population (one per
/// population above 1). Extra tiles are released one at a time (`next_release`); missing ones
/// are claimed (`next_claim`), as many as there are free candidates, so a city boxed in by
/// rivals catches up once land frees. The founding ring never counts.
pub(crate) fn settle_territory(state: &mut GameState, id: CityId, events: &mut Vec<Event>) {
    let Some(c) = state.cities.get(&id) else {
        return;
    };
    let held: Vec<Hex> = c
        .grown_tiles
        .iter()
        .copied()
        .filter(|h| {
            state.map.get(*h).is_some_and(|t| {
                t.city == Some(id)
                    && !t
                        .landmark
                        .is_some_and(|site| data::is_great_work(site.building))
            })
        })
        .collect();
    let target = (c.population - 1).max(0) as usize;
    let owner = c.owner;
    let city = state.cities.get_mut(&id).expect("city exists");
    city.tiles_claimed = (city.tiles_claimed - (city.grown_tiles.len() - held.len()) as i32).max(0);
    city.grown_tiles = held;
    while state.cities[&id].grown_tiles.len() > target {
        let Some(tile) = next_release(state, &state.cities[&id]) else {
            break;
        };
        if let Some(site) = state.map.get(tile).and_then(|t| t.landmark)
            && let Some(city) = state.cities.get_mut(&site.city)
        {
            city.buildings.remove(&site.building);
        }
        let c = state.cities.get_mut(&id).expect("city exists");
        c.grown_tiles.retain(|h| *h != tile);
        c.tiles_claimed = (c.tiles_claimed - 1).max(0);
        let t = state.map.get_mut(tile).expect("on map");
        t.owner = None;
        t.city = None;
        events.push(Event::TileReleased {
            city: id,
            owner,
            tile,
        });
    }
    while state.cities[&id].grown_tiles.len() < target {
        let Some(tile) = next_claim(state, &state.cities[&id]) else {
            break;
        };
        claim_tile(state, id, tile);
        state
            .cities
            .get_mut(&id)
            .expect("city exists")
            .grown_tiles
            .push(tile);
        events.push(Event::TileClaimed {
            city: id,
            owner,
            tile,
        });
    }
}

/// Hand an out-of-reach tile to the nearest city of the same empire that can work it.
/// Scan the settled map first, then apply transfers in map order: claim order and city
/// iteration cannot affect the recipient. A transferred growth claim no longer counts
/// against its former city's population; the recipient receives existing land, not a
/// new growth claim.
pub(crate) fn reassign_workable_territory(state: &mut GameState, events: &mut Vec<Event>) {
    let transfers: Vec<_> = state
        .map
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, tile)| {
            let from = tile.city?;
            let source = state.cities.get(&from)?;
            let at = state.map.hex_at(i);
            if tile.owner != Some(source.owner) || at.distance(source.pos) <= CITY_WORK_RADIUS {
                return None;
            }
            let to = state
                .cities
                .values()
                .filter(|city| city.owner == source.owner && city.id != from)
                .filter_map(|city| {
                    let distance = at.distance(city.pos);
                    (distance <= CITY_WORK_RADIUS).then_some((distance, city.id))
                })
                .min()?
                .1;
            Some((at, from, to, source.owner))
        })
        .collect();
    let mut by_pair: BTreeMap<(CityId, CityId, PlayerId), Vec<Hex>> = BTreeMap::new();
    for (at, from, to, owner) in transfers {
        state.map.get_mut(at).expect("tile on map").city = Some(to);
        let source = state.cities.get_mut(&from).expect("source city exists");
        if source.grown_tiles.contains(&at) {
            source.grown_tiles.retain(|h| *h != at);
            source.tiles_claimed = (source.tiles_claimed - 1).max(0);
        }
        by_pair.entry((from, to, owner)).or_default().push(at);
    }
    for ((from, to, owner), tiles) in by_pair {
        events.push(Event::TilesReassigned {
            from,
            to,
            owner,
            tiles,
        });
    }
}

/// Territory phase: every city settles its territory to its population (`settle_territory`), in
/// city order, so the lower city id wins a tile two cities want in the same turn. Also ends the
/// turn's city phases by ticking post-capture resistance down.
pub fn process_borders(state: &mut GameState, events: &mut Vec<Event>) {
    let ids: Vec<CityId> = state.cities.keys().copied().collect();
    for id in &ids {
        settle_territory(state, *id, events);
    }
    reassign_workable_territory(state, events);
    for id in &ids {
        let c = state.cities.get_mut(id).expect("city exists");
        if c.resistance > 0 {
            c.resistance -= 1;
        }
    }
}

#[cfg(test)]
#[path = "cities_tests.rs"]
pub(crate) mod tests;
