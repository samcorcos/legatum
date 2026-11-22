//! Legatum simulation core: all game rules, pure and deterministic.
//!
//! Determinism rules (see plans/first-playable.md): integer math only, one derived RNG,
//! ordered collections only. The same seed and orders must produce the same state hash on
//! every platform, native and WASM.
#![deny(clippy::float_arithmetic)]

pub mod army;
pub mod bitset;
pub mod calendar;
pub mod combat;
pub mod company;
pub mod data;
pub mod diplomacy;
pub mod economy;
pub mod eras;
pub mod event_visibility;
pub mod events;
pub mod experiments;
pub mod forecast;
pub mod game;
pub mod goals;
pub mod hex;
pub mod history;
pub mod improvements;
pub mod landmarks;
pub mod los;
pub mod mapgen;
pub mod objectives;
pub mod orders;
pub mod path;
pub mod projects;
pub mod research;
pub mod resources;
pub mod rng;
pub mod roads;
pub mod rules;
/// Increment whenever game rules or the serialized save shape changes.
/// 40: World Age Veterans card replaces Demobilise (designer, 2026-11-15): `ClaimObjective` removed.
/// 41: Egypt joins; Food constructs ordinary buildings, with no faction goal.
/// 42: Rome joins; veterans found cities, the Legionary and Castrum replace Phalangite and Walls, with no faction goal.
/// 43: German Production buildings each cost 1 Food; the Ruhr Foundry's own Food cost is removed.
/// 44: Roman veterans found cities without Gold or Settler price escalation.
/// 45: Every empire starts with its capital founded on its start hex, without a starting Settler.
/// 46: Cities can buy a tile with Gold again (`Order::BuyTile`), sharing the city's one purchase a turn.
/// 47: A shot at a fallen, empty hostile city waits for whoever moves in to defend it and hits them before that round's melee; an attack whose target dies before movement goes on against the next enemy on its tile (Sam, 2026-11-21).
/// 48: Egypt's Nile: double Food, no Production, surplus Food builds everything and the Grow project grows.
/// 49: A faction skips every technology that gives it nothing (Egypt: Bronze Working), known from the start; Egypt's cards drop Production-only lines.
/// 50: Fable review batch (Sam, 2026-11-22): a rival's original capital scores 1 VP only while held; Alliance upkeep is waived at War with a team close to winning; experiment `no_purchase_under_threat`; same-turn tile buys, era firsts and exact world-wonder ties between empires are drawn by seed. `Player::capitals_taken` is gone from the save.
/// 51: Units of empires not at War stepping onto one tile in the same round no longer all bounce: the team with the most combined strength among its claimants moves in, the rest stay put, nobody fights; equal totals are drawn by seed, and so is an exact force tie at a fallen city, which used to bounce everyone (Sam, 2026-11-22).
/// 52: No rule changed. Unit, building and improvement identifiers were renamed to the PascalCase of their display names (`Soldier` is `Spearman`, `RelayUplink` is `GreatLibrary`, ...), which changes the serialized state and so every state hash. The old names still load through serde aliases (`types.rs`), for tapes, archived games and fixtures.
/// 53: Humble the mighty: when no rival team is ahead of yours on VP, a capture from the rival team with the most VP counts (owner, 2026-11-22). The save shape is unchanged.
/// 54: Rome and Egypt get faction goals (Sam, 2026-11-22). Rome's Colonia Victrix: a city founded by a level-3 melee veteran within two tiles of a rival's land is held 2 VP. Egypt's royal tomb (`Action::Tomb`): while it is built no Egyptian city builds or grows; after 15 turn ends of work it stands in the capital, held 2 VP. `Player` gains `colonia_city`, `tomb_work`, `tomb_turns` and `tomb_city`.
/// 55: City automation focuses are Military, Economy and a Gold, Science or Food stream; every empire but Nile Egypt gains a Food project (Sam, 2026-11-22).
pub const RULES_VERSION: u32 = 55;
pub mod scenario;
pub mod scoring;
pub mod settlement;
pub mod state;
pub mod types;
pub mod validation;
pub mod view;
pub mod visibility;
pub mod wonders;
pub mod yields_cache;

pub use events::Event;
pub use forecast::{
    AlongsideAttack, AssaultGarrison, AttackMode, BattleForecast, CityAssaultForecast,
    GarrisonForecast, city_attack_target, city_fires, city_strike_target, forecast_attack,
    forecast_attack_mode, forecast_city_assault, forecast_city_strike,
};
pub use game::{new_game, try_new_game};
pub use hex::Hex;
pub use orders::{Order, OrderError};
pub use resources::{Resource, ResourceClass, SupplyUser};
pub use rules::{resolve_turn, validate_orders};
pub use state::{AiDifficulty, GameSetup, GameState, PlayerSetup};

#[cfg(test)]
mod bicycle_tests;
#[cfg(test)]
mod eras_tests;
#[cfg(test)]
mod faction_rules_tests;
#[cfg(test)]
mod keshig_tests;
#[cfg(test)]
mod modern_era_tests;
#[cfg(test)]
mod pricing_tests;
#[cfg(test)]
mod projects_tests;
#[cfg(test)]
mod recon_tests;
#[cfg(test)]
mod units_tests;
pub use types::*;
pub use view::{
    PlayerView, PublicView, ViewDetail, player_view, player_view_cached, player_view_detail,
    player_view_with, public_view,
};
pub use yields_cache::ViewCache;

#[cfg(test)]
mod aztecs_tests;
#[cfg(test)]
mod rome_tests;
