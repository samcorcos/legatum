//! WASM API over the core and AI for the web UI. JSON strings in and out, so the UI owns its
//! TypeScript types. 64-bit seeds and hashes cross as decimal strings.
//!
//! The logic lives in plain-Rust functions (`Result<_, String>`) so it is testable natively;
//! the `#[wasm_bindgen]` methods only convert errors to `JsError`.

pub mod history;
pub mod host;
pub mod json;
pub mod rusher;
pub mod scenario;
pub mod session;
pub mod tape;

use legatum_core::{
    AlongsideAttack, AttackMode, Faction, GameSetup, Hex, Order, PlayerId, PlayerView, UnitId,
    forecast_attack_mode, forecast_city_assault, player_view, validate_orders,
};
use serde::{Deserialize, Serialize};
use session::{BrainKind, OrderMap, Session};
use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Primitive UI what-if queries. Every answer delegates to the same core function used by
/// resolution or the full fogged forecast; the browser only formats the result.
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum CombatProjection {
    BaseDamage {
        own: i32,
        other: i32,
    },
    Damage {
        own: i32,
        other: i32,
        roll: i32,
    },
    CarriedShare {
        direct: i32,
        leftover: i32,
        damage: i32,
    },
    UnitAttack {
        faction: Faction,
        kind: legatum_core::UnitKind,
        hp: i32,
        level: u8,
    },
    UnitDefense {
        faction: Faction,
        kind: legatum_core::UnitKind,
        hp: i32,
        level: u8,
        terrain: legatum_core::Terrain,
        fortify: i32,
    },
    UnitRanged {
        faction: Faction,
        kind: legatum_core::UnitKind,
        hp: i32,
        level: u8,
    },
    CityCanFire {
        walls: bool,
        hp: i32,
    },
    CityTileDefender {
        hp: i32,
        garrisoned: bool,
    },
    SiegeTier {
        techs: Vec<legatum_core::TechId>,
    },
    SiegeCover {
        kind: legatum_core::UnitKind,
        siege: bool,
        techs: Vec<legatum_core::TechId>,
    },
    WallDamagePercent {
        tier: u8,
        covered: u8,
        air: bool,
    },
    CityDefense {
        faction: Faction,
        population: i32,
        tier: u8,
        covered: u8,
        hp: i32,
        era: u8,
    },
    ExactOdds {
        atk: i32,
        def: i32,
        atk_hp: i32,
        def_hp: i32,
        ranged: bool,
        city_percent: i32,
    },
    UnitStats {
        faction: Faction,
        kind: legatum_core::UnitKind,
    },
    StandardUnitStats {
        faction: Faction,
        kind: legatum_core::UnitKind,
    },
    BuildingStats {
        faction: Faction,
        kind: legatum_core::BuildingKind,
    },
    StandardBuildingStats {
        faction: Faction,
        kind: legatum_core::BuildingKind,
    },
    ClassBonus {
        own: legatum_core::UnitKind,
        other: legatum_core::UnitKind,
        faction: Option<Faction>,
    },
    ClassBonusStrength {
        base: i32,
    },
    MountedCityPenalty {
        faction: Faction,
        kind: legatum_core::UnitKind,
        shot: bool,
    },
    FightBonus {
        own: legatum_core::UnitKind,
        other: legatum_core::UnitKind,
        attacker_side: bool,
        shot: bool,
        faction: Option<Faction>,
    },
    CityGap {
        faction: Faction,
    },
    TerrainYields {
        faction: Faction,
        terrain: legatum_core::Terrain,
    },
    OutriderBlocks {
        attacker: legatum_core::UnitKind,
        defender: legatum_core::UnitKind,
        distance: i32,
    },
    CityFireOutcome {
        forecast: legatum_core::forecast::BattleForecast,
        faction: Option<Faction>,
        kind: legatum_core::UnitKind,
    },
    SplitCounterattackLoss {
        defender: i32,
        attacker: i32,
        attacker_hp: i32,
        attackers: i32,
    },
}

fn combat_projection_json(query_json: &str) -> Result<String, String> {
    use legatum_core::{combat, data, forecast};
    let query: CombatProjection = serde_json::from_str(query_json).map_err(|e| e.to_string())?;
    let result = match query {
        CombatProjection::BaseDamage { own, other } => {
            serde_json::json!(combat::base_damage(own, other))
        }
        CombatProjection::Damage { own, other, roll } => {
            serde_json::json!(combat::damage(own, other, roll))
        }
        CombatProjection::CarriedShare {
            direct,
            leftover,
            damage,
        } => serde_json::json!(combat::carried_share(direct, leftover, damage)),
        CombatProjection::UnitAttack {
            faction,
            kind,
            hp,
            level,
        } => serde_json::json!(combat::unit_attack_strength_for(faction, kind, hp, level)),
        CombatProjection::UnitDefense {
            faction,
            kind,
            hp,
            level,
            terrain,
            fortify,
        } => serde_json::json!(combat::unit_defense_strength_for(
            faction, kind, hp, terrain, fortify, level
        )),
        CombatProjection::UnitRanged {
            faction,
            kind,
            hp,
            level,
        } => serde_json::json!(combat::unit_ranged_strength_for(faction, kind, hp, level)),
        CombatProjection::CityCanFire { walls, hp } => {
            serde_json::json!(combat::city_can_fire(walls, hp))
        }
        CombatProjection::CityTileDefender { hp, garrisoned } => {
            serde_json::json!(combat::city_tile_defender(hp, garrisoned))
        }
        CombatProjection::SiegeTier { techs } => {
            serde_json::json!(data::siege_tier(&techs.into_iter().collect()))
        }
        CombatProjection::SiegeCover { kind, siege, techs } => {
            let known = techs.into_iter().collect();
            serde_json::json!(data::siege_cover(kind, siege.then_some(&known)))
        }
        CombatProjection::WallDamagePercent { tier, covered, air } => {
            serde_json::json!(combat::wall_damage_percent(tier, covered, air))
        }
        CombatProjection::CityDefense {
            faction,
            population,
            tier,
            covered,
            hp,
            era,
        } => serde_json::json!(combat::city_defense_covered(
            faction, population, tier, covered, hp, era
        )),
        CombatProjection::ExactOdds {
            atk,
            def,
            atk_hp,
            def_hp,
            ranged,
            city_percent,
        } => serde_json::json!(forecast::primitive_odds(
            atk,
            def,
            atk_hp,
            def_hp,
            !ranged,
            city_percent
        )),
        CombatProjection::UnitStats { faction, kind } => {
            let s = data::unit_stats_for(faction, kind);
            serde_json::json!({"cost": s.cost, "moves": s.moves, "strength": s.strength,
                "ranged": s.ranged, "range": s.range, "military": s.military,
                "requires": s.requires, "upgradesTo": s.upgrades_to})
        }
        CombatProjection::StandardUnitStats { faction, kind } => {
            let s = data::standard_unit_stats_for(faction, kind);
            serde_json::json!({"cost": s.cost, "moves": s.moves, "strength": s.strength,
                "ranged": s.ranged, "range": s.range, "military": s.military,
                "requires": s.requires, "upgradesTo": s.upgrades_to})
        }
        CombatProjection::BuildingStats { faction, kind } => {
            let s = data::building_stats_for(faction, kind);
            serde_json::json!({"cost": s.cost, "yields": s.yields, "cityStrength": s.city_strength,
                "requires": s.requires, "warBuilding": s.war_building})
        }
        CombatProjection::StandardBuildingStats { faction, kind } => {
            let s = data::standard_building_stats_for(faction, kind);
            serde_json::json!({"cost": s.cost, "yields": s.yields, "cityStrength": s.city_strength,
                "requires": s.requires, "warBuilding": s.war_building})
        }
        CombatProjection::ClassBonus {
            own,
            other,
            faction,
        } => {
            let (label, bonus) = faction.map_or_else(
                || combat::class_bonus(own, other),
                |faction| combat::class_bonus_for(faction, own, other),
            );
            serde_json::json!({"label": label, "bonus": bonus})
        }
        CombatProjection::ClassBonusStrength { base } => {
            serde_json::json!(combat::class_bonus_strength(base))
        }
        CombatProjection::MountedCityPenalty {
            faction,
            kind,
            shot,
        } => serde_json::json!(combat::mounted_city_penalty(faction, kind, shot)),
        CombatProjection::FightBonus {
            own,
            other,
            attacker_side,
            shot,
            faction,
        } => {
            let (label, bonus) = faction.map_or_else(
                || combat::fight_class_bonus(own, other, attacker_side, shot),
                |faction| combat::fight_class_bonus_for(faction, own, other, attacker_side, shot),
            );
            serde_json::json!({"label": label, "bonus": bonus})
        }
        CombatProjection::CityGap { faction } => serde_json::json!(data::min_city_gap(faction)),
        CombatProjection::TerrainYields { faction, terrain } => {
            serde_json::json!(data::terrain_yields(faction, terrain))
        }
        CombatProjection::OutriderBlocks {
            attacker,
            defender,
            distance,
        } => serde_json::json!(combat::outrider_blocks(attacker, defender, distance)),
        CombatProjection::CityFireOutcome {
            forecast,
            faction,
            kind,
        } => serde_json::json!(forecast::city_fire_outcome(
            &forecast,
            faction.unwrap_or(Faction::India),
            kind
        )),
        CombatProjection::SplitCounterattackLoss {
            defender,
            attacker,
            attacker_hp,
            attackers,
        } => serde_json::json!(forecast::split_counterattack_loss(
            defender,
            attacker,
            attacker_hp,
            attackers
        )),
    };
    to_json(&result)
}

/// Run `f`, turning a panic into an error message. On native targets this catches panics; on
/// wasm32-unknown-unknown panics abort (the call throws a `RuntimeError: unreachable` in JS),
/// so the panic hook below logs the real message to `console.error` first.
fn guard<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    install_panic_hook();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(payload) => Err(format!("internal error: {}", panic_message(&*payload))),
    }
}

pub fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "panic".to_string()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(msg: &str);
}

fn install_panic_hook() {
    #[cfg(target_arch = "wasm32")]
    {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            std::panic::set_hook(Box::new(|info| console_error(&format!("legatum: {info}"))));
        });
    }
}

fn js<T>(r: Result<T, String>) -> Result<T, JsError> {
    r.map_err(|e| JsError::new(&e))
}

fn to_json<T: serde::Serialize>(v: &T) -> Result<String, String> {
    serde_json::to_string(v).map_err(|e| format!("serialize: {e}"))
}

fn parse_faction(s: &str) -> Result<Faction, String> {
    Faction::ALL
        .into_iter()
        .find(|f| format!("{f:?}").eq_ignore_ascii_case(s.trim()))
        .ok_or_else(|| format!("unknown faction ID {s:?}"))
}

/// Every non-human player gets the scripted AI.
fn scripted(_: PlayerId) -> BrainKind {
    BrainKind::Scripted
}

// ---- plain-Rust implementation ----

pub fn standard_setup_json(
    seed: &str,
    faction1: &str,
    faction2: &str,
    name1: &str,
    name2: &str,
) -> Result<String, String> {
    let seed: u64 = seed
        .trim()
        .parse()
        .map_err(|_| format!("seed must be a decimal u64, got {seed:?}"))?;
    let mut setup = GameSetup::standard(seed, [parse_faction(faction1)?, parse_faction(faction2)?]);
    setup.players[0].name = name1.to_string();
    setup.players[1].name = name2.to_string();
    Ok(json::setup_to_value(&setup).to_string())
}

/// Replay and verify a sim tape, including the initial state and continuation after save/load;
/// return per-turn hashes as a JSON array of decimal strings.
pub fn replay_tape_json(tape_json: &str) -> Result<String, String> {
    let tape = tape::Tape::from_json(tape_json)?;
    let replay = tape::replay_checked(&tape)?;
    let hashes: Vec<String> = replay.hashes.iter().map(|h| h.to_string()).collect();
    to_json(&hashes)
}

impl Game {
    fn invalidate_view(&mut self) {
        self.state_change = self.state_change.wrapping_add(1);
        self.view_cache.get_mut().take();
        self.forecast_view.get_mut().take();
    }

    /// The viewer's view for forecasts, built once per state.
    fn forecast_view(&self, p: PlayerId) -> Rc<PlayerView> {
        if let Some((player, change, view)) = self.forecast_view.borrow().as_ref()
            && *player == p
            && *change == self.state_change
        {
            return Rc::clone(view);
        }
        let view = Rc::new(player_view(&self.session.state, p));
        self.forecast_view
            .replace(Some((p, self.state_change, Rc::clone(&view))));
        view
    }

    fn check_player(&self, player: u8) -> Result<PlayerId, String> {
        if (player as usize) < self.session.state.players.len() {
            Ok(PlayerId(player))
        } else {
            Err(format!("no player {player}"))
        }
    }

    pub fn from_setup_json(setup_json: &str) -> Result<Game, String> {
        let setup = json::parse_setup(setup_json)?;
        Ok(Game {
            session: Session::try_new(&setup, scripted)?,
            scenario: None,

            state_change: 0,
            view_cache: RefCell::new(None),
            forecast_view: RefCell::new(None),
        })
    }

    pub fn from_save_json(save_json: &str) -> Result<Game, String> {
        json::check_json_size(save_json)?;
        Ok(Game {
            session: Session::from_save_json(save_json, scripted)?,
            scenario: None,

            state_change: 0,
            view_cache: RefCell::new(None),
            forecast_view: RefCell::new(None),
        })
    }

    /// Start a scenario: its prepared start state with the seat as the only human.
    pub fn from_scenario_json(scenario_json: &str) -> Result<Game, String> {
        let file = scenario::ScenarioFile::parse(scenario_json)?;
        let start = file.prepare(true)?;
        Ok(Game {
            session: Session::from_state(start.clone(), scripted),
            scenario: Some(scenario::ActiveScenario::new(file, start)),

            state_change: 0,
            view_cache: RefCell::new(None),
            forecast_view: RefCell::new(None),
        })
    }

    /// Re-attach a scenario's objective to a restored game (after recovering from a save).
    pub fn attach_scenario_json(&mut self, scenario_json: &str) -> Result<(), String> {
        let file = scenario::ScenarioFile::parse(scenario_json)?;
        let start = file.prepare(true)?;
        self.scenario = Some(scenario::ActiveScenario::new(file, start));
        self.invalidate_view();
        self.session.invalidate_state_hash();
        Ok(())
    }

    /// The objective's status for the current state, as JSON.
    pub fn scenario_status_json(&self) -> Result<String, String> {
        let active = self.scenario.as_ref().ok_or("no scenario in play")?;
        to_json(&active.status(&self.session.state))
    }

    pub fn view_json(&self, player: u8) -> Result<String, String> {
        let p = self.check_player(player)?;
        if let Some(cached) = self.view_cache.borrow().as_ref()
            && cached.player == p
            && cached.state_change == self.state_change
        {
            return Ok(cached.json.clone());
        }
        let json = to_json(&player_view(&self.session.state, p))?;
        self.view_cache.replace(Some(CachedView {
            player: p,
            state_change: self.state_change,
            json: json.clone(),
        }));
        Ok(json)
    }

    pub fn validate_json(&self, player: u8, orders_json: &str) -> Result<String, String> {
        let p = self.check_player(player)?;
        let orders: Vec<Order> =
            serde_json::from_str(orders_json).map_err(|e| format!("invalid orders: {e}"))?;
        to_json(&validate_orders(&self.session.state, p, &orders))
    }

    pub fn forecast_json(
        &self,
        player: u8,
        unit: u32,
        q: i32,
        r: i32,
        mode: &str,
    ) -> Result<String, String> {
        let p = self.check_player(player)?;
        let mode = match mode {
            "Melee" => AttackMode::Melee,
            "Ranged" => AttackMode::Ranged,
            _ => return Err(format!("unknown attack mode {mode:?} (Melee, Ranged)")),
        };
        let view = self.forecast_view(p);
        to_json(&forecast_attack_mode(
            &view,
            UnitId(unit),
            Hex::new(q, r),
            mode,
        ))
    }

    /// The combined fight at an enemy city (`CityAssaultForecast`): `alongside_json` lists the
    /// viewer's other attacks queued on the same tile, `[{"unit":7,"mode":"Melee"}]`.
    pub fn assault_json(
        &self,
        player: u8,
        unit: u32,
        q: i32,
        r: i32,
        mode: &str,
        alongside_json: &str,
    ) -> Result<String, String> {
        let p = self.check_player(player)?;
        let mode = match mode {
            "Melee" => AttackMode::Melee,
            "Ranged" => AttackMode::Ranged,
            _ => return Err(format!("unknown attack mode {mode:?} (Melee, Ranged)")),
        };
        let alongside: Vec<AlongsideAttack> = serde_json::from_str(alongside_json)
            .map_err(|e| format!("invalid alongside attacks: {e}"))?;
        let view = self.forecast_view(p);
        to_json(&forecast_city_assault(
            &view,
            UnitId(unit),
            Hex::new(q, r),
            mode,
            &alongside,
        ))
    }

    /// Resolve the turn. `human_orders_json` maps human player ids to their orders; missing
    /// humans submit nothing. AI players' orders are computed here.
    pub fn end_turn_json(&mut self, human_orders_json: &str) -> Result<String, String> {
        if self.session.state.finished {
            return Err("the game is finished".into());
        }
        if let Some(active) = &self.scenario
            && active.status(&self.session.state).outcome
                != legatum_core::scenario::Outcome::Playing
        {
            return Err("the scenario is over".into());
        }
        let human: OrderMap = if human_orders_json.trim().is_empty() {
            OrderMap::new()
        } else {
            serde_json::from_str(human_orders_json)
                .map_err(|e| format!("invalid orders map: {e}"))?
        };
        for p in human.keys() {
            self.check_player(p.0)?;
            if self.session.is_ai(*p) {
                return Err(format!("player {} is AI-controlled", p.0));
            }
        }
        let mut orders = self.session.take_ai_orders();
        if let Some(r) = self.scenario.as_ref().and_then(|a| a.file.rusher.as_ref()) {
            rusher::apply(&mut orders, &self.session.state, r);
        }
        orders.extend(human);
        let before: Vec<_> = self
            .session
            .state
            .players
            .iter()
            .filter(|p| p.is_human)
            .map(|p| {
                legatum_core::event_visibility::event_visibility_view(&self.session.state, p.id)
            })
            .collect();
        let resolved = self.session.state.turn;
        // Training mode: the human seats' battle positions with the orders they gave, logged
        // for offline labelling (never searched live).
        let mut outcome_lines: Vec<String> = Vec::new();
        if self.session.has_training() {
            for p in self
                .session
                .state
                .players
                .iter()
                .filter(|p| p.is_human && p.alive)
            {
                let given = orders.get(&p.id).map(Vec::as_slice).unwrap_or_default();
                let found =
                    legatum_ai::training::positions(&self.session.state, p.id, given, "app");
                self.session.outcomes.open_positions(&found);
                outcome_lines.extend(
                    found
                        .into_iter()
                        .map(|r| legatum_ai::training::LogLine::Position(Box::new(r)).json()),
                );
            }
        }
        let events = self.session.step(&orders);
        self.session.training_run = None;
        outcome_lines.extend(
            self.session
                .outcomes
                .close(&self.session.state)
                .into_iter()
                .map(|o| legatum_ai::training::LogLine::Outcome(o).json()),
        );
        self.invalidate_view();
        let s = &self.session.state;
        let events_by_player: std::collections::BTreeMap<_, _> = before
            .iter()
            .map(|view| -> Result<_, String> {
                let after = player_view(s, view.me);
                let visible = legatum_core::event_visibility::visible_events(&events, view, &after);
                let cache_empty = self.view_cache.borrow().is_none();
                if cache_empty {
                    self.view_cache.replace(Some(CachedView {
                        player: view.me,
                        state_change: self.state_change,
                        json: to_json(&after)?,
                    }));
                }
                Ok((view.me, visible))
            })
            .collect::<Result<_, _>>()?;
        let mut result = serde_json::json!({
            "turn": resolved,
            "events": events,
            "events_by_player": events_by_player,
            "hash": self.session.state_hash().to_string(),
            "finished": s.finished,
            "winner": s.winner,
        });
        // Training mode: realised outcomes of earlier searched engagements, as log lines.
        if !outcome_lines.is_empty() {
            result["training_lines"] = serde_json::json!(outcome_lines);
        }
        to_json(&result)
    }

    pub fn save_json(&self) -> String {
        self.session.save_json()
    }

    /// Training mode, at the start of a turn: compute the AI seats' orders now and start
    /// studying the training seats' battles. Returns the progress, or `null` when this game
    /// has no training seat. Idempotent within a turn.
    pub fn training_begin_json(&mut self, cap_ms: f64) -> Result<String, String> {
        if !self.session.has_training() || self.session.state.finished {
            return Ok("null".into());
        }
        let turn = self.session.state.turn;
        if self
            .session
            .training_run
            .as_ref()
            .is_none_or(|r| r.turn() != turn)
        {
            let hard = self.session.training_hard_orders();
            let cfg = legatum_ai::training::TrainingConfig {
                cap_ms: cap_ms.max(0.0) as u64,
                ..Default::default()
            };
            self.session.training_run = Some(legatum_ai::training::TrainingRun::new(
                &self.session.state,
                hard,
                &cfg,
            ));
        }
        to_json(&self.session.training_run.as_ref().map(|r| r.progress()))
    }

    /// Run up to `max_rollouts` rollouts. `elapsed_ms` since the turn began.
    pub fn training_step_json(
        &mut self,
        max_rollouts: u32,
        elapsed_ms: f64,
    ) -> Result<String, String> {
        let Some(run) = self.session.training_run.as_mut() else {
            return Ok("null".into());
        };
        run.step(max_rollouts as usize, elapsed_ms.max(0.0) as u64);
        to_json(&run.progress())
    }

    /// Stop, play the best orders found and return the result (log lines
    /// as JSON strings in `lines`).
    pub fn training_finish_json(&mut self, elapsed_ms: f64) -> Result<String, String> {
        let Some(run) = self.session.training_run.take() else {
            return Ok("null".into());
        };
        let result = run.finish(elapsed_ms.max(0.0) as u64);
        self.apply_training(result)
    }

    fn apply_training(
        &mut self,
        result: legatum_ai::training::TrainingResult,
    ) -> Result<String, String> {
        self.session.apply_training(&result)?;
        let lines: Vec<String> = result
            .records
            .iter()
            .map(|r| legatum_ai::training::LogLine::Decision(Box::new(r.clone())).json())
            .collect();
        to_json(&serde_json::json!({
            "turn": result.turn,
            "progress": result.progress,
            "lines": lines,
        }))
    }

    /// Coach questions: before `end_turn`, compare `player`'s orders with what Hard would
    /// have ordered in his seat. `asked_json` is `{"keys":[...],"last":[turn,"category"]|null}`.
    /// Returns `{"question":Question|null,"candidates":[Question...],"hard_orders":[Order...]}`. The game is
    /// not changed (state, RNG and hash stay as they were).
    pub fn why_review_json(
        &mut self,
        player: u8,
        orders_json: &str,
        asked_json: &str,
    ) -> Result<String, String> {
        let p = self.check_player(player)?;
        if self.session.is_ai(p) {
            return Err(format!("player {player} is AI-controlled"));
        }
        let his: Vec<Order> = if orders_json.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(orders_json).map_err(|e| format!("invalid orders: {e}"))?
        };
        let asked: legatum_ai::why::Asked = if asked_json.trim().is_empty() {
            Default::default()
        } else {
            serde_json::from_str(asked_json).map_err(|e| format!("invalid asked record: {e}"))?
        };
        let review = self.session.why_review(p, &his, &asked);
        to_json(&serde_json::json!({
            "question": review.question,
            "candidates": review.candidates,
            "hard_orders": review.hard_orders,
        }))
    }

    /// Per-seat planning time of the last End Turn as `[[seat, ms, searched], ...]`
    /// (`seat-timing` builds; empty otherwise).
    pub fn seat_timings_json(&self) -> String {
        let rows: Vec<_> = self
            .session
            .last_seat_ms
            .iter()
            .map(|(id, ms, searched)| (id.0, *ms, *searched))
            .collect();
        to_json(&rows).unwrap_or_default()
    }
}

// ---- JavaScript surface ----

struct CachedView {
    player: PlayerId,
    state_change: u64,
    json: String,
}

#[wasm_bindgen]
pub struct Game {
    session: Session,
    scenario: Option<scenario::ActiveScenario>,
    state_change: u64,
    view_cache: RefCell<Option<CachedView>>,
    /// The viewer's `PlayerView` for hover forecasts, kept until the state changes: hovering a fight asks for
    /// two or three forecasts per tile, and rebuilding the whole view for each made late-game hover lag.
    forecast_view: RefCell<Option<(PlayerId, u64, Rc<PlayerView>)>>,
}

#[wasm_bindgen]
impl Game {
    /// Core-computed projection for combat card what-if values.
    pub fn combat_projection(query_json: &str) -> Result<String, JsError> {
        js(guard(|| combat_projection_json(query_json)))
    }

    /// Start a game from GameSetup JSON (seed as a number or a decimal string).
    #[wasm_bindgen(constructor)]
    pub fn new(setup_json: &str) -> Result<Game, JsError> {
        js(guard(|| Game::from_setup_json(setup_json)))
    }

    /// The standard two-humans-plus-six-AIs setup, as GameSetup JSON (seed as a string).
    pub fn standard_setup(
        seed: &str,
        faction1: &str,
        faction2: &str,
        name1: &str,
        name2: &str,
    ) -> Result<String, JsError> {
        js(guard(|| {
            standard_setup_json(seed, faction1, faction2, name1, name2)
        }))
    }

    /// UI hints accept only the already fogged view, never privileged state or an oracle.
    pub fn settlement_sites(
        view_json: &str,
        unit: u32,
        reserved_json: &str,
    ) -> Result<String, JsError> {
        js(guard(|| {
            let view: legatum_core::PlayerView =
                serde_json::from_str(view_json).map_err(|e| e.to_string())?;
            let reserved: Vec<Hex> =
                serde_json::from_str(reserved_json).map_err(|e| e.to_string())?;
            let sites = view
                .units
                .iter()
                .find(|u| u.id == UnitId(unit))
                .map(|u| legatum_core::settlement::settlement_scores(&view, u, &reserved))
                .unwrap_or_default();
            to_json(&sites)
        }))
    }

    /// Full explanation rows for inspection and fixture assertions; the live lens uses scores.
    pub fn settlement_sites_full(
        view_json: &str,
        unit: u32,
        reserved_json: &str,
    ) -> Result<String, JsError> {
        js(guard(|| {
            let view: legatum_core::PlayerView =
                serde_json::from_str(view_json).map_err(|e| e.to_string())?;
            let reserved: Vec<Hex> =
                serde_json::from_str(reserved_json).map_err(|e| e.to_string())?;
            let sites = view
                .units
                .iter()
                .find(|u| u.id == UnitId(unit))
                .map(|u| legatum_core::settlement::settlement_sites(&view, u, &reserved))
                .unwrap_or_default();
            to_json(&sites)
        }))
    }

    /// Explain only the inspected hex, using the same fog-safe scoring as the map.
    pub fn settlement_site(
        view_json: &str,
        unit: u32,
        q: i32,
        r: i32,
        reserved_json: &str,
    ) -> Result<String, JsError> {
        js(guard(|| {
            let view: legatum_core::PlayerView =
                serde_json::from_str(view_json).map_err(|e| e.to_string())?;
            let reserved: Vec<Hex> =
                serde_json::from_str(reserved_json).map_err(|e| e.to_string())?;
            let site = view
                .units
                .iter()
                .find(|u| u.id == UnitId(unit))
                .and_then(|u| {
                    legatum_core::settlement::settlement_site(&view, u, &reserved, Hex::new(q, r))
                });
            to_json(&site)
        }))
    }

    /// Why a hex is not a legal city site for the viewer (`null` when it is one): the same rule
    /// as the map's Settle tint, from the view's explored map and anonymous city-gap mask.
    pub fn site_blocker(view_json: &str, q: i32, r: i32) -> Result<String, JsError> {
        js(guard(|| {
            let view: legatum_core::PlayerView =
                serde_json::from_str(view_json).map_err(|e| e.to_string())?;
            to_json(&legatum_core::settlement::site_blocker(
                &view,
                Hex::new(q, r),
            ))
        }))
    }

    /// Threat tint from the fogged view, using the same reach calculation as settlement advice.
    pub fn known_threats(view_json: &str) -> Result<String, JsError> {
        js(guard(|| {
            let view: legatum_core::PlayerView =
                serde_json::from_str(view_json).map_err(|e| e.to_string())?;
            to_json(&legatum_core::settlement::known_threats(&view))
        }))
    }

    /// Which of these known enemies (`[id, ...]`) can reach a tile by moving next turn, as a JSON id list: the view is
    /// parsed once for all of them (aftermath text asked per enemy, and a late-game view is large).
    pub fn units_can_reach(
        view_json: &str,
        units_json: &str,
        q: i32,
        r: i32,
    ) -> Result<String, JsError> {
        js(guard(|| {
            let view: legatum_core::PlayerView =
                serde_json::from_str(view_json).map_err(|e| e.to_string())?;
            let ids: Vec<u32> = serde_json::from_str(units_json).map_err(|e| e.to_string())?;
            let Some(index) = view.index(Hex::new(q, r)) else {
                return to_json(&Vec::<u32>::new());
            };
            let mut alone = view.clone();
            let reach: Vec<u32> = ids
                .into_iter()
                .filter(|&id| {
                    let Some(unit) = view.units.iter().find(|u| u.id == UnitId(id)) else {
                        return false;
                    };
                    let mut unit = unit.clone();
                    unit.ranged = 0;
                    alone.units = vec![unit].into();
                    legatum_core::settlement::known_threats(&alone)[index] > 0
                })
                .collect();
            to_json(&reach)
        }))
    }

    /// Production price from a real save, for fixtures that need to set up a nearly completed item.
    pub fn production_cost(save_json: &str, owner: u8, item_json: &str) -> Result<i32, JsError> {
        js(guard(|| {
            let state = json::parse_state(save_json)?;
            let item: legatum_core::ProductionItem =
                serde_json::from_str(item_json).map_err(|e| e.to_string())?;
            if owner as usize >= state.players.len() {
                return Err(format!("no player {owner}"));
            }
            Ok(legatum_core::economy::production_cost(
                &state,
                PlayerId(owner),
                item,
            ))
        }))
    }

    /// PlayerView JSON for `player`.
    pub fn view(&self, player: u8) -> Result<String, JsError> {
        js(guard(|| self.view_json(player)))
    }

    /// `Vec<OrderError>` JSON for a list of orders.
    pub fn validate(&self, player: u8, orders_json: &str) -> Result<String, JsError> {
        js(guard(|| self.validate_json(player, orders_json)))
    }

    /// BattleForecast JSON for mode `Melee` or `Ranged`, or `null` when there is nothing to attack.
    pub fn forecast(
        &self,
        player: u8,
        unit: u32,
        q: i32,
        r: i32,
        mode: &str,
    ) -> Result<String, JsError> {
        js(guard(|| self.forecast_json(player, unit, q, r, mode)))
    }

    /// `CityAssaultForecast` JSON, or `null` when the target is not an enemy city.
    pub fn assault(
        &self,
        player: u8,
        unit: u32,
        q: i32,
        r: i32,
        mode: &str,
        alongside_json: &str,
    ) -> Result<String, JsError> {
        js(guard(|| {
            self.assault_json(player, unit, q, r, mode, alongside_json)
        }))
    }

    /// Input `{"0":[...],"1":[...]}`. Returns
    /// `{"turn":<resolved turn>,"events":[...],"events_by_player":{"0":[...],...},
    /// "hash":"<decimal>","finished":bool,"winner":team|null}`.
    /// UI reports and animations must use `events_by_player`; `events` is the complete log.
    pub fn end_turn(&mut self, human_orders_json: &str) -> Result<String, JsError> {
        js(guard(|| self.end_turn_json(human_orders_json)))
    }

    /// The id training log lines are filed under: the archive's game id.
    pub fn training_game_id(&self) -> String {
        self.session
            .history
            .as_ref()
            .map(|h| h.metadata.game_id.clone())
            .unwrap_or_else(|| format!("seed-{}", self.session.state.setup.seed))
    }

    /// Training mode: start this turn's battle search (see `training_begin_json`).
    pub fn training_begin(&mut self, cap_ms: f64) -> Result<String, JsError> {
        js(guard(|| self.training_begin_json(cap_ms)))
    }

    pub fn training_step(&mut self, max_rollouts: u32, elapsed_ms: f64) -> Result<String, JsError> {
        js(guard(|| self.training_step_json(max_rollouts, elapsed_ms)))
    }

    pub fn training_finish(&mut self, elapsed_ms: f64) -> Result<String, JsError> {
        js(guard(|| self.training_finish_json(elapsed_ms)))
    }

    /// Coach questions (see `why_review_json`).
    pub fn why_review(
        &mut self,
        player: u8,
        orders_json: &str,
        asked_json: &str,
    ) -> Result<String, JsError> {
        js(guard(|| {
            self.why_review_json(player, orders_json, asked_json)
        }))
    }

    /// Per-seat planning time of the last End Turn (measurement builds, `seat-timing`).
    #[cfg(feature = "seat-timing")]
    pub fn seat_timings(&self) -> String {
        self.seat_timings_json()
    }

    /// Full state JSON. Opaque to JavaScript: pass it back to `load`.
    pub fn save(&self) -> Result<String, JsError> {
        js(guard(|| Ok(self.save_json())))
    }

    /// Portable full-information archive for the owner. This never includes online seat tokens.
    pub fn archive(&self) -> Result<String, JsError> {
        js(guard(|| {
            to_json(self.session.history.as_ref().ok_or("No recorded history")?)
        }))
    }

    /// The archive with every seat under its default name ("Player 1", "AI 2"), safe to upload.
    pub fn public_archive(&self) -> Result<String, JsError> {
        js(guard(|| {
            let history = self.session.history.as_ref().ok_or("No recorded history")?;
            to_json(&history.anonymized()?)
        }))
    }

    /// Attach nondeterministic client timing to a resolved archive turn, outside GameState.
    pub fn record_turn_time(
        &mut self,
        turn: u32,
        ended_at: &str,
        active_seconds: u32,
        era: u8,
        session_id: &str,
    ) -> Result<(), JsError> {
        self.invalidate_view();
        self.session.invalidate_state_hash();
        js(guard(|| {
            self.session
                .history
                .as_mut()
                .ok_or("No recorded history")?
                .record_turn_time(history::TurnPlayTime {
                    turn,
                    ended_at: ended_at.to_owned(),
                    active_seconds,
                    era,
                    session_id: session_id.to_owned(),
                })
        }))
    }

    /// Attach a turn's browser frame and input timing (`TurnFrameTime` JSON) to the archive, outside GameState.
    pub fn record_frame_time(&mut self, time_json: &str) -> Result<(), JsError> {
        self.invalidate_view();
        self.session.invalidate_state_hash();
        js(guard(|| {
            let time: history::TurnFrameTime =
                serde_json::from_str(time_json).map_err(|e| format!("invalid frame time: {e}"))?;
            self.session
                .history
                .as_mut()
                .ok_or("No recorded history")?
                .record_frame_time(time)
        }))
    }

    pub fn fork_archive(&mut self) {
        self.invalidate_view();
        self.session.invalidate_state_hash();
        if let Some(history) = &mut self.session.history {
            history.fork();
        }
    }

    pub fn load(json: &str) -> Result<Game, JsError> {
        js(guard(|| Game::from_save_json(json)))
    }

    /// Start a scenario from its file JSON (repo `scenarios/<id>.json`).
    pub fn load_scenario(scenario_json: &str) -> Result<Game, JsError> {
        js(guard(|| Game::from_scenario_json(scenario_json)))
    }

    /// Re-attach a scenario's objective after `load` restored its game.
    pub fn attach_scenario(&mut self, scenario_json: &str) -> Result<(), JsError> {
        js(guard(|| self.attach_scenario_json(scenario_json)))
    }

    /// Objective status JSON (`legatum_core::scenario::Status`) for the current state.
    pub fn scenario_status(&self) -> Result<String, JsError> {
        js(guard(|| self.scenario_status_json()))
    }

    /// Current state hash, decimal.
    pub fn hash(&self) -> String {
        self.session.state_hash().to_string()
    }

    /// The turn now being planned.
    pub fn turn(&self) -> u32 {
        self.session.state.turn
    }

    /// Verify a sim order tape, including the initial state and save/load between turns.
    /// Returns the per-turn hashes as a JSON array of decimal strings, or an error on mismatch.
    pub fn replay_tape(tape_json: &str) -> Result<String, JsError> {
        js(guard(|| replay_tape_json(tape_json)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn view_json_exposes_a_locked_building_predecessor() {
        use legatum_core::TechId;

        let setup = GameSetup::standard(7, [Faction::Switzerland, Faction::Byzantium]);
        let mut game = Game::from_setup_json(&json::setup_to_value(&setup).to_string()).unwrap();
        game.session.state.players[0].techs.insert(TechId::Chivalry);
        game.invalidate_view();

        let view: Value = serde_json::from_str(&game.view_json(0).unwrap()).unwrap();
        let unavailable = view["cities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|city| city["owner"] == 0 && !city["detail"].is_null())
            .unwrap()["detail"]["unavailable"]
            .as_array()
            .unwrap();
        let castle = unavailable
            .iter()
            .find(|entry| entry["item"]["building"] == "Castle")
            .unwrap();
        assert_eq!(castle["prerequisite_building"], "Walls");
    }

    #[test]
    fn delegated_city_automation_orders_respect_player_queue_and_save_round_trip() {
        use legatum_core::{CityBuildFocus, ProductionItem, UnitKind};
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let mut game = Game::from_setup_json(&setup).unwrap();
        let first: legatum_core::PlayerView =
            serde_json::from_str(&game.view_json(0).unwrap()).unwrap();
        let city = game
            .session
            .state
            .cities
            .values()
            .find(|c| c.owner == first.me)
            .unwrap()
            .id;
        let state_city = game.session.state.cities.get_mut(&city).unwrap();
        state_city.producing = None;
        state_city.production_queue.clear();
        state_city.build_automation = Some(CityBuildFocus::Balanced);
        game.invalidate_view();
        let view = game.view_json(0).unwrap();
        let planned = city_automation_orders_json(&view, "[]").unwrap();
        let plan: CityAutomationPlan = serde_json::from_str(&planned).unwrap();
        let orders = &plan.orders;
        assert!(plan.unchosen_cities.is_empty());
        assert!(
            orders
                .iter()
                .any(|o| matches!(o, Order::SetProduction { city: id, .. } if *id == city))
        );
        assert!(!orders.iter().any(|o| matches!(o, Order::BuyItem { .. })));
        assert_eq!(planned, city_automation_orders_json(&view, "[]").unwrap());
        let rival_view: legatum_core::PlayerView =
            serde_json::from_str(&game.view_json(1).unwrap()).unwrap();
        assert!(
            rival_view
                .cities
                .iter()
                .all(|c| c.id != city || c.detail.is_none())
        );
        assert_eq!(
            city_automation_orders_json(&game.view_json(1).unwrap(), "[]").unwrap(),
            r#"{"orders":[],"unchosen_cities":[]}"#
        );
        let manual = vec![Order::SetProductionQueue {
            city,
            queue: vec![ProductionItem::Unit {
                unit: UnitKind::Scout,
            }],
        }];
        assert_eq!(
            city_automation_orders_json(&view, &to_json(&manual).unwrap()).unwrap(),
            r#"{"orders":[],"unchosen_cities":[]}"#
        );
        game.session
            .state
            .cities
            .get_mut(&city)
            .unwrap()
            .production_queue = vec![ProductionItem::Unit {
            unit: UnitKind::Scout,
        }];
        game.invalidate_view();
        assert_eq!(
            city_automation_orders_json(&game.view_json(0).unwrap(), "[]").unwrap(),
            r#"{"orders":[],"unchosen_cities":[]}"#
        );
        game.session
            .state
            .cities
            .get_mut(&city)
            .unwrap()
            .production_queue
            .clear();
        let current = ProductionItem::Unit {
            unit: UnitKind::Soldier,
        };
        let cost = legatum_core::economy::city_production_cost(
            &game.session.state,
            &game.session.state.cities[&city],
            current,
        );
        let state_city = game.session.state.cities.get_mut(&city).unwrap();
        state_city.producing = Some(current);
        state_city.production_stored = cost - 1;
        game.invalidate_view();
        let finishing = game.view_json(0).unwrap();
        let finishing_plan: CityAutomationPlan =
            serde_json::from_str(&city_automation_orders_json(&finishing, "[]").unwrap()).unwrap();
        assert!(finishing_plan.orders.iter().any(|o| matches!(o, Order::SetProductionQueue { city: id, queue } if *id == city && queue.first() == Some(&current) && queue.len() == 2)));

        // Save through a real turn: direct test mutations above intentionally leave the
        // replay archive unchanged, and archived games correctly reject that mismatch.
        let mut saved_game = Game::from_setup_json(&setup).unwrap();
        let saved_city = saved_game
            .session
            .state
            .cities
            .values()
            .find(|c| c.owner == first.me)
            .unwrap()
            .id;
        saved_game
            .end_turn_json(&format!(
                r#"{{"0":[{{"type":"SetCityAutomation","city":{},"focus":"Balanced"}}]}}"#,
                saved_city.0
            ))
            .unwrap();
        let saved = saved_game.save_json();
        let reloaded = Game::from_save_json(&saved).unwrap();
        let reloaded_view: legatum_core::PlayerView =
            serde_json::from_str(&reloaded.view_json(0).unwrap()).unwrap();
        assert_eq!(
            reloaded_view
                .cities
                .iter()
                .find(|c| c.id == saved_city)
                .unwrap()
                .detail
                .as_ref()
                .unwrap()
                .build_automation,
            Some(CityBuildFocus::Balanced)
        );
        // A missing field in an older city record is off by default.
        let mut old = serde_json::to_value(&reloaded.session.state.cities[&saved_city]).unwrap();
        old.as_object_mut().unwrap().remove("build_automation");
        let legacy: legatum_core::state::City = serde_json::from_value(old).unwrap();
        assert_eq!(legacy.build_automation, None);
    }

    #[test]
    fn delegated_city_with_every_option_excluded_asks_for_a_manual_choice() {
        use legatum_core::{CityBuildFocus, UnitKind};
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let game = Game::from_setup_json(&setup).unwrap();
        let mut view: legatum_core::PlayerView =
            serde_json::from_str(&game.view_json(0).unwrap()).unwrap();
        let city = view.cities.iter_mut().find(|c| c.owner == view.me).unwrap();
        let id = city.id;
        let detail = city.detail.as_mut().unwrap();
        detail.producing = None;
        detail.production_queue.clear();
        detail.build_automation = Some(CityBuildFocus::Science);
        detail.options.retain(|o| {
            matches!(
                o.item,
                legatum_core::ProductionItem::Unit {
                    unit: UnitKind::Soldier
                }
            )
        });
        assert_eq!(detail.options.len(), 1); // Science excludes this build, but the player can choose it.
        let json = to_json(&view).unwrap();
        let plan: CityAutomationPlan =
            serde_json::from_str(&city_automation_orders_json(&json, "[]").unwrap()).unwrap();
        assert_eq!(plan.unchosen_cities, vec![id]);
        assert!(plan.orders.is_empty());
        let manual = vec![Order::SetProduction {
            city: id,
            item: legatum_core::ProductionItem::Unit {
                unit: UnitKind::Soldier,
            },
        }];
        let manual_plan: CityAutomationPlan = serde_json::from_str(
            &city_automation_orders_json(&json, &to_json(&manual).unwrap()).unwrap(),
        )
        .unwrap();
        assert!(manual_plan.unchosen_cities.is_empty());
    }

    #[test]
    fn a_city_frozen_by_the_occupation_lock_is_never_an_unchosen_city() {
        use legatum_core::{CityBuildFocus, UnitKind};
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let game = Game::from_setup_json(&setup).unwrap();
        let mut view: legatum_core::PlayerView =
            serde_json::from_str(&game.view_json(0).unwrap()).unwrap();
        let city = view.cities.iter_mut().find(|c| c.owner == view.me).unwrap();
        let id = city.id;
        let detail = city.detail.as_mut().unwrap();
        detail.producing = None;
        detail.production_queue.clear();
        detail.build_automation = Some(CityBuildFocus::Science);
        detail.options.retain(|o| {
            matches!(
                o.item,
                legatum_core::ProductionItem::Unit {
                    unit: UnitKind::Soldier
                }
            )
        });
        let json = to_json(&view).unwrap();
        let plan: CityAutomationPlan =
            serde_json::from_str(&city_automation_orders_json(&json, "[]").unwrap()).unwrap();
        assert_eq!(plan.unchosen_cities, vec![id]);
        let city = view.cities.iter_mut().find(|c| c.id == id).unwrap();
        city.occupation_lock = Some(legatum_core::PlayerId(1));
        let frozen: CityAutomationPlan = serde_json::from_str(
            &city_automation_orders_json(&to_json(&view).unwrap(), "[]").unwrap(),
        )
        .unwrap();
        assert!(frozen.unchosen_cities.is_empty());
        assert!(frozen.orders.is_empty());
    }

    #[test]
    fn standard_setup_rejects_bad_input() {
        assert!(standard_setup_json("-1", "India", "India", "a", "b").is_err());
        assert!(standard_setup_json("1", "Elves", "India", "a", "b").is_err());
    }

    #[test]
    fn malformed_local_setups_return_errors() {
        let base: Value = serde_json::from_str(
            &standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap(),
        )
        .unwrap();
        for (name, mutate) in [
            (
                "negative width",
                Box::new(|v: &mut Value| v["width"] = (-1).into()) as Box<dyn Fn(&mut Value)>,
            ),
            (
                "huge width",
                Box::new(|v: &mut Value| v["width"] = i32::MAX.into()),
            ),
            (
                "no players",
                Box::new(|v: &mut Value| v["players"] = Value::Array(vec![])),
            ),
            (
                "too many players",
                Box::new(|v: &mut Value| {
                    let players = v["players"].as_array_mut().unwrap();
                    while players.len() <= 8 {
                        players.push(players[0].clone());
                    }
                }),
            ),
            (
                "reserved team",
                Box::new(|v: &mut Value| v["players"][0]["team"] = 255.into()),
            ),
            (
                "impossible starts",
                Box::new(|v: &mut Value| {
                    v["width"] = 8.into();
                    v["height"] = 8.into();
                }),
            ),
            (
                "exhausted turn limit",
                Box::new(|v: &mut Value| v["turn_limit"] = u32::MAX.into()),
            ),
        ] {
            let mut value = base.clone();
            mutate(&mut value);
            assert!(Game::from_setup_json(&value.to_string()).is_err(), "{name}");
        }
    }

    #[test]
    fn malformed_local_saves_return_errors_before_session_construction() {
        let state = legatum_core::new_game(&GameSetup::all_ai(7));
        let base: Value = serde_json::from_str(&json::state_to_json(&state)).unwrap();
        for (name, mutate) in [
            (
                "short map",
                Box::new(|v: &mut Value| {
                    v["map"]["tiles"].as_array_mut().unwrap().pop();
                }) as Box<dyn Fn(&mut Value)>,
            ),
            (
                "player id",
                Box::new(|v: &mut Value| v["players"][0]["id"] = 200.into()),
            ),
            (
                "explored storage",
                Box::new(|v: &mut Value| {
                    v["players"][0]["explored"]["words"]
                        .as_array_mut()
                        .unwrap()
                        .pop();
                }),
            ),
            (
                "unit owner",
                Box::new(|v: &mut Value| {
                    let units = v["units"].as_object_mut().unwrap();
                    units.values_mut().next().unwrap()["owner"] = 200.into();
                }),
            ),
            (
                "tile owner",
                Box::new(|v: &mut Value| v["map"]["tiles"][0]["owner"] = 200.into()),
            ),
            (
                "missing city",
                Box::new(|v: &mut Value| v["map"]["tiles"][0]["city"] = 123.into()),
            ),
            (
                "exhausted turn",
                Box::new(|v: &mut Value| v["turn"] = u32::MAX.into()),
            ),
            (
                "near exhausted turn",
                Box::new(|v: &mut Value| {
                    v["turn"] = (u32::MAX - legatum_core::validation::COUNTER_HEADROOM).into()
                }),
            ),
            (
                "exhausted unit ids",
                Box::new(|v: &mut Value| v["next_unit_id"] = u32::MAX.into()),
            ),
            (
                "exhausted city ids",
                Box::new(|v: &mut Value| v["next_city_id"] = u32::MAX.into()),
            ),
        ] {
            let mut value = base.clone();
            mutate(&mut value);
            assert!(Game::from_save_json(&value.to_string()).is_err(), "{name}");
        }
    }

    #[test]
    fn malformed_tape_setup_returns_error() {
        let mut setup: Value = serde_json::from_str(
            &standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap(),
        )
        .unwrap();
        setup["width"] = 8.into();
        setup["height"] = 8.into();
        let tape = serde_json::json!({
            "version": tape::TAPE_VERSION,
            "setup": setup,
            "initial_hash": "0",
            "turns": [],
        });
        assert!(replay_tape_json(&tape.to_string()).is_err());
    }

    #[test]
    fn guard_turns_panics_into_errors() {
        let r: Result<(), String> = guard(|| panic!("boom"));
        assert_eq!(r.unwrap_err(), "internal error: boom");
    }

    #[test]
    fn replay_rejects_garbage() {
        assert!(replay_tape_json("{}").is_err());
        assert!(replay_tape_json("not json").is_err());
    }

    #[test]
    fn cached_hash_matches_state_after_turn_save_load_and_scenario_attach() {
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let mut game = Game::from_setup_json(&setup).unwrap();
        assert_eq!(game.hash(), game.session.state.hash().to_string());
        let report: Value = serde_json::from_str(&game.end_turn_json("{}").unwrap()).unwrap();
        assert_eq!(report["hash"], Value::String(game.hash()));
        assert_eq!(game.hash(), game.session.state.hash().to_string());
        let save = game.save_json();
        assert_eq!(game.hash(), game.session.state.hash().to_string());
        let mut loaded = Game::from_save_json(&save).unwrap();
        assert_eq!(loaded.hash(), loaded.session.state.hash().to_string());
        loaded
            .attach_scenario_json(include_str!("../../../scenarios/opening-race.json"))
            .unwrap();
        assert_eq!(loaded.hash(), loaded.session.state.hash().to_string());
    }

    #[test]
    fn cached_view_is_byte_identical_and_invalidated_by_mutation() {
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let mut game = Game::from_setup_json(&setup).unwrap();
        let fresh = |game: &Game| to_json(&player_view(&game.session.state, PlayerId(0))).unwrap();
        assert_eq!(game.view_json(0).unwrap(), fresh(&game));
        game.end_turn_json("{}").unwrap();
        assert_eq!(
            game.view_cache.borrow().as_ref().unwrap().player,
            PlayerId(0)
        );
        assert_eq!(game.view_json(0).unwrap(), fresh(&game));
        assert_eq!(game.view_json(0).unwrap(), fresh(&game));
        game.view_json(1).unwrap();
        assert_eq!(game.view_json(0).unwrap(), fresh(&game));
        game.fork_archive();
        assert!(game.view_cache.borrow().is_none());
        assert_eq!(game.view_json(0).unwrap(), fresh(&game));
    }

    #[test]
    fn forecast_view_is_reused_until_the_state_changes() {
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let mut game = Game::from_setup_json(&setup).unwrap();
        let first = game.forecast_view(PlayerId(0));
        assert!(Rc::ptr_eq(&first, &game.forecast_view(PlayerId(0))));
        assert!(!Rc::ptr_eq(&first, &game.forecast_view(PlayerId(1))));
        game.end_turn_json("{}").unwrap();
        let after = game.forecast_view(PlayerId(0));
        assert_eq!(
            to_json(&*after).unwrap(),
            to_json(&player_view(&game.session.state, PlayerId(0))).unwrap()
        );
        assert_ne!(to_json(&*first).unwrap(), to_json(&*after).unwrap());
    }

    /// Lobby setups: solo on a small map with per-AI difficulty. Normal is omitted from JSON (old
    /// hashes unchanged), other difficulties survive save/load, and difficulty changes AI play.
    #[test]
    fn lobby_setup_with_difficulty() {
        let setup = |d: &str| {
            let mut v: Value = serde_json::from_str(
                &standard_setup_json("99", "India", "Byzantium", "A", "B").unwrap(),
            )
            .unwrap();
            v["width"] = Value::from(40);
            v["height"] = Value::from(26);
            let players = v["players"].as_array_mut().unwrap();
            players.truncate(4);
            players[1]["is_human"] = Value::from(false);
            players[1]["team"] = Value::from(7);
            for p in players.iter_mut().skip(1) {
                p["difficulty"] = Value::from(d);
            }
            v.to_string()
        };
        let normal = Game::from_setup_json(&setup("Normal")).unwrap();
        assert!(
            !json::state_to_json(&normal.session.state).contains("difficulty"),
            "Normal is not serialized in hashed state (archive metadata is explicit)"
        );
        let mut hashes = Vec::new();
        for d in ["Easy", "Normal", "Hard"] {
            let mut g = Game::from_setup_json(&setup(d)).unwrap();
            assert_eq!(g.session.state.players.len(), 4);
            let view: Value = serde_json::from_str(&g.view_json(0).unwrap()).unwrap();
            for player in view["players"].as_array().unwrap().iter().skip(1) {
                if d == "Normal" {
                    assert!(player.get("difficulty").is_none());
                } else {
                    assert_eq!(player["difficulty"], d);
                }
            }
            for _ in 0..30 {
                g.end_turn_json("{}").unwrap();
            }
            let saved = g.save_json();
            if d != "Normal" {
                assert!(saved.contains(&format!("\"difficulty\":\"{d}\"")));
            }
            let mut g2 = Game::from_save_json(&saved).unwrap();
            assert_eq!(
                g.end_turn_json("{}").unwrap(),
                g2.end_turn_json("{}").unwrap()
            );
            hashes.push(g.session.state.hash());
        }
        assert_ne!(hashes[0], hashes[1], "Easy plays differently from Normal");
        assert_ne!(hashes[2], hashes[1], "Hard plays differently from Normal");
    }

    #[test]
    fn research_queue_json_round_trips_and_advances_after_load() {
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let mut g = Game::from_setup_json(&setup).unwrap();
        g.session.state.players[0].science_stored = 1000;
        g.session.history = None;
        let queue = r#"[{"type":"SetResearchQueue","techs":["BronzeWorking","Writing"]}]"#;
        assert_eq!(g.validate_json(0, queue).unwrap(), "[]");
        assert_ne!(
            g.validate_json(
                0,
                r#"[{"type":"SetResearchQueue","techs":["Mathematics"]}]"#
            )
            .unwrap(),
            "[]"
        );
        g.end_turn_json(&format!(r#"{{"0":{queue}}}"#)).unwrap();
        let view: Value = serde_json::from_str(&g.view_json(0).unwrap()).unwrap();
        assert_eq!(view["research"]["queue"], serde_json::json!(["Writing"]));
        let mut loaded = Game::from_save_json(&g.save_json()).unwrap();
        assert_eq!(
            g.end_turn_json("{}").unwrap(),
            loaded.end_turn_json("{}").unwrap()
        );
        let view: Value = serde_json::from_str(&loaded.view_json(0).unwrap()).unwrap();
        assert_eq!(view["research"]["queue"], serde_json::json!([]));
        assert!(view["research"]["current"].is_null());
    }

    #[test]
    fn research_tree_is_exposed_and_old_charter_saves_still_load() {
        let setup = standard_setup_json("7", "India", "Byzantium", "A", "B").unwrap();
        let g = Game::from_setup_json(&setup).unwrap();
        let view: Value = serde_json::from_str(&g.view_json(0).unwrap()).unwrap();
        assert_eq!(view["research"]["catalog"].as_array().unwrap().len(), 45);
        assert_eq!(view["research"]["unlocks"].as_array().unwrap().len(), 55);
        let unlocks = view["research"]["unlocks"].as_array().unwrap();
        let core = unlocks
            .iter()
            .find(|u| u["item"]["building"] == "InferenceCore")
            .unwrap();
        assert_eq!(core["predecessor"], "College");
        assert_eq!(core["min_population"], 8);
        assert_eq!(core["cost"], 360);
        assert_eq!(core["yields"]["science"], 6);
        let armor = unlocks
            .iter()
            .find(|u| u["item"]["unit"] == "AssaultTrooper")
            .unwrap();
        assert_eq!(armor["upgrade_from"], "Trooper");
        assert_eq!(armor["upgrade_cost"], 116);
        assert_eq!(armor["unit_strength"], 44);
        let college = unlocks
            .iter()
            .find(|u| u["item"]["building"] == "College")
            .unwrap();
        assert_eq!(college["predecessor"], "Laboratory");
        assert_eq!(college["min_population"], 5);
        assert_eq!(college["cost"], 240);
        // The Military Engineer retired with the Siege upgrade: it is no unit unlock.
        assert!(
            unlocks
                .iter()
                .all(|u| u["item"]["unit"] != "MilitaryEngineer")
        );
        let ranger_upgrade: Vec<Order> =
            serde_json::from_str(r#"[{"type":"UpgradeUnit","unit":1}]"#).unwrap();
        assert!(matches!(ranger_upgrade[0], Order::UpgradeUnit { .. }));

        // Saves from before charters were removed carry a "doctrine" field; it is ignored.
        let mut save: Value = serde_json::from_str(&g.save_json()).unwrap();
        save["players"][0]["doctrine"] = Value::from("FrontierCharter");
        let loaded = Game::from_save_json(&save.to_string()).unwrap();
        assert_eq!(loaded.session.state.hash(), g.session.state.hash());
        assert!(!loaded.save_json().contains("doctrine"));

        // The old order is gone, so its JSON is rejected rather than silently applied.
        let order = r#"[{"type":"ChooseDoctrine","doctrine":"FrontierCharter"}]"#;
        assert!(g.validate_json(0, order).is_err());
    }

    #[test]
    fn local_turn_events_are_filtered_independently_for_each_human_team() {
        use legatum_core::{
            TeamId, Terrain, UnitKind,
            game::{blank_game, spawn_unit},
            state::Map,
        };
        let mut setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
        setup.players.truncate(3);
        setup.players[1].team = TeamId(1);
        setup.players[2].team = TeamId(0);
        setup.players[2].is_human = true;
        let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
        let mut orders = OrderMap::new();
        for (i, pos) in [Hex::new(3, 3), Hex::new(25, 20), Hex::new(25, 3)]
            .into_iter()
            .enumerate()
        {
            let id = PlayerId(i as u8);
            let settler = spawn_unit(&mut state, id, UnitKind::Settler, pos);
            orders.insert(id, vec![Order::FoundCity { unit: settler }]);
        }
        legatum_core::visibility::update_explored(&mut state);
        let mut game = Game {
            session: Session::from_state(state, scripted),
            scenario: None,

            state_change: 0,
            view_cache: RefCell::new(None),
            forecast_view: RefCell::new(None),
        };
        let result: Value =
            serde_json::from_str(&game.end_turn_json(&to_json(&orders).unwrap()).unwrap()).unwrap();
        let foundations = |events: &Value| -> Vec<u64> {
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["type"] == "CityFounded")
                .map(|e| e["owner"].as_u64().unwrap())
                .collect()
        };
        assert_eq!(foundations(&result["events"]), vec![0, 1, 2]);
        assert_eq!(foundations(&result["events_by_player"]["0"]), vec![0, 2]);
        assert_eq!(foundations(&result["events_by_player"]["1"]), vec![1]);
        assert_eq!(
            result["events_by_player"]["0"],
            result["events_by_player"]["2"]
        );
        for id in [0, 2] {
            let view = player_view(&game.session.state, PlayerId(id));
            assert!(!view.tile(Hex::new(25, 20)).unwrap().explored);
        }
    }

    #[test]
    fn wounded_ranger_forecasts_match_melee_and_ranged_resolution() {
        use legatum_core::{
            BattleForecast, Event, TeamId, Terrain, UnitKind, combat, game::spawn_unit,
        };

        for mode in ["Melee", "Ranged"] {
            let mut setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
            setup.players.truncate(2);
            setup.players[1].team = TeamId(1);
            let mut g = Game::from_setup_json(&json::setup_to_value(&setup).to_string()).unwrap();
            g.end_turn_json(r#"{"0":[{"type":"DeclareWar","against":1}]}"#)
                .unwrap();
            let state = &mut g.session.state;
            state.units.clear();
            for tile in &mut state.map.tiles {
                tile.terrain = Terrain::Grassland;
                tile.ruin = false;
            }
            let target = Hex::new(6, 5);
            let attacker = spawn_unit(state, PlayerId(0), UnitKind::Ranger, Hex::new(5, 5));
            spawn_unit(state, PlayerId(1), UnitKind::Soldier, target);
            state.units.get_mut(&attacker).unwrap().hp = 20;
            legatum_core::visibility::update_explored(state);

            let forecast: BattleForecast = serde_json::from_str(
                &g.forecast_json(0, attacker.0, target.q, target.r, mode)
                    .unwrap(),
            )
            .unwrap();
            assert!(
                g.forecast_json(0, attacker.0, target.q, target.r, "Auto")
                    .is_err()
            );
            let ranged = mode == "Ranged";
            assert_eq!(forecast.ranged, ranged);
            assert_eq!(forecast.loss_chance, if ranged { 0 } else { 100 });
            assert_eq!(
                forecast.expected_damage_to_attacker,
                if ranged { 0 } else { 20 }
            );

            let order = if ranged {
                Order::RangedAttack {
                    unit: attacker,
                    target,
                }
            } else {
                Order::MoveUnit {
                    unit: attacker,
                    to: target,
                }
            };
            let orders = serde_json::json!({"0": [order]}).to_string();
            let result: Value = serde_json::from_str(&g.end_turn_json(&orders).unwrap()).unwrap();
            let events: Vec<Event> = serde_json::from_value(result["events"].clone()).unwrap();
            let battle = events
                .iter()
                .find(|e| matches!(e, Event::Battle { .. }))
                .unwrap();
            let Event::Battle {
                attacker_strength,
                defender_strength,
                attacker_roll,
                defender_roll,
                damage_to_attacker,
                damage_to_defender,
                attacker_hp_after,
                ranged: actual_ranged,
                ..
            } = battle
            else {
                unreachable!()
            };
            assert_eq!(*actual_ranged, ranged);
            assert_eq!(attacker_strength, &forecast.attacker_strength);
            assert_eq!(defender_strength, &forecast.defender_strength);
            let a = attacker_strength.total;
            let d = defender_strength.total;
            assert_eq!(*damage_to_defender, combat::damage(a, d, *attacker_roll));
            assert_eq!(
                *damage_to_attacker,
                if ranged {
                    0
                } else {
                    combat::damage(d, a, *defender_roll)
                }
            );
            assert_eq!(*attacker_hp_after, if ranged { 20 } else { 0 });
            // The displayed expectation averages the same damage math over all possible rolls.
            let rolls = combat::ROLL_MIN..=combat::ROLL_MAX;
            let count = rolls.clone().count() as i32;
            assert_eq!(
                forecast.expected_damage_to_defender,
                rolls
                    .clone()
                    .map(|r| combat::damage(a, d, r).min(forecast.defender_hp))
                    .sum::<i32>()
                    / count
            );
            assert_eq!(
                forecast.expected_damage_to_attacker,
                if ranged {
                    0
                } else {
                    rolls
                        .map(|r| combat::damage(d, a, r).min(forecast.attacker_hp))
                        .sum::<i32>()
                        / count
                }
            );
        }
    }

    #[test]
    fn full_api_round_trip() {
        let setup = standard_setup_json("7", "India", "Switzerland", "A", "B").unwrap();
        let mut g = Game::from_setup_json(&setup).unwrap();
        let v: Value = serde_json::from_str(&g.view_json(0).unwrap()).unwrap();
        assert_eq!(v["me"], Value::from(0));
        assert_eq!(g.validate_json(0, "[]").unwrap(), "[]");
        assert!(g.check_player(8).is_err());
        let r: Value =
            serde_json::from_str(&g.end_turn_json(r#"{"0":[],"1":[]}"#).unwrap()).unwrap();
        assert_eq!(r["turn"], Value::from(1));
        assert!(r["hash"].is_string());
        assert!(g.end_turn_json(r#"{"2":[]}"#).is_err(), "AI seat rejected");
        // Save/load keeps the hash, and both copies then play the same next turn.
        let saved = g.save_json();
        let mut g2 = Game::from_save_json(&saved).unwrap();
        assert_eq!(g2.session.state.hash(), g.session.state.hash());
        let a = g.end_turn_json("{}").unwrap();
        let b = g2.end_turn_json("{}").unwrap();
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod bicycle_two_tests {
    use super::*;
    use legatum_core::{
        BuildingKind, ProductionItem, TeamId, TechId, Terrain, UnitKind, VpSource, data,
        game::spawn_unit, rules::cities::found_city,
    };
    #[test]
    fn all_great_work_orders_views_and_completion_credit_round_trip_through_json() {
        for building in data::GREAT_WORKS {
            let mut setup = GameSetup::all_ai(700001);
            setup.players.truncate(2);
            for (i, p) in setup.players.iter_mut().enumerate() {
                p.team = TeamId(i as u8);
                p.is_human = true;
            }
            let mut g = Game::from_setup_json(&json::setup_to_value(&setup).to_string()).unwrap();
            let s = &mut g.session.state;
            s.units.clear();
            s.cities.clear();
            for t in &mut s.map.tiles {
                t.terrain = Terrain::Grassland;
                t.owner = None;
                t.city = None;
                t.ruin = false;
            }
            s.players[0].techs.extend(TechId::ALL);
            let mut ids = Vec::new();
            for pos in [Hex::from_offset(5, 5), Hex::from_offset(12, 5)] {
                let unit = spawn_unit(s, PlayerId(0), UnitKind::Settler, pos);
                let id = found_city(s, unit, &mut Vec::new()).unwrap();
                ids.push(id);
                let c = s.cities.get_mut(&id).unwrap();
                c.population = 8;
                c.buildings.insert(BuildingKind::Hydroponics);
                if let Some((required, _)) = data::great_work_requirement(building) {
                    c.buildings.insert(required);
                }
            }
            s.cities.get_mut(&ids[0]).unwrap().production_stored =
                data::building_stats(building).cost - 1;
            let tile = Hex::from_offset(6, 5);
            let order = serde_json::json!([{"type":"SetProduction","city":ids[0].0,"item":{"kind":"Wonder","building":building,"tile":tile}}]);
            assert_eq!(g.validate_json(0, &order.to_string()).unwrap(), "[]");
            let result: serde_json::Value = serde_json::from_str(
                &g.end_turn_json(&serde_json::json!({"0":order,"1":[]}).to_string())
                    .unwrap(),
            )
            .unwrap();
            assert!(
                result["events"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|e| e["type"] == "ProductionCompleted"
                        && e["item"]["building"] == serde_json::json!(building))
            );
            let loaded = Game::from_save_json(&g.save_json()).unwrap();
            assert_eq!(g.hash(), loaded.hash());
            let view: legatum_core::PlayerView =
                serde_json::from_str(&loaded.view_json(0).unwrap()).unwrap();
            assert!(view.development.is_some());
            assert_eq!(view.great_works[0].completed, Some(building));
            assert_eq!(
                loaded
                    .session
                    .state
                    .map
                    .get(tile)
                    .unwrap()
                    .landmark
                    .unwrap()
                    .building,
                building
            );
            assert!(
                view.players[0]
                    .vp_sources
                    .contains(&(VpSource::GreatWork { of: PlayerId(0) }, 2))
            );
            assert!(!view.cities.iter().filter_map(|c|c.detail.as_ref()).flat_map(|d|&d.options).any(|o|matches!(o.item,ProductionItem::Wonder{building:b,..} if data::is_great_work(b))));
        }
    }
}

/// The cities a scenario's objective is about, from its start state, for the briefing before
/// it starts (`legatum_core::scenario::objective_targets`).
#[wasm_bindgen]
pub fn scenario_targets(scenario_json: &str) -> Result<String, JsError> {
    js(scenario_targets_json(scenario_json))
}

pub fn scenario_targets_json(scenario_json: &str) -> Result<String, String> {
    let file = scenario::ScenarioFile::parse(scenario_json)?;
    let start = file.stored_state()?;
    to_json(&legatum_core::scenario::objective_targets(
        &file.objective,
        file.seat,
        &start,
        &start,
    ))
}

/// Stateless planning for local and online clients. The only input is the seat's fogged view.
#[wasm_bindgen]
pub fn explore_step(view_json: &str, unit: u32) -> Result<String, JsError> {
    js(explore_step_json(view_json, unit))
}

/// Filter a proposed event list with the core's privacy rules for one player view.
#[wasm_bindgen]
pub fn visible_events_json(
    events_json: &str,
    before_json: &str,
    after_json: &str,
) -> Result<String, JsError> {
    js(guard(|| {
        let events: Vec<legatum_core::Event> =
            serde_json::from_str(events_json).map_err(|e| e.to_string())?;
        let before: legatum_core::PlayerView =
            serde_json::from_str(before_json).map_err(|e| e.to_string())?;
        let after: legatum_core::PlayerView =
            serde_json::from_str(after_json).map_err(|e| e.to_string())?;
        to_json(&legatum_core::event_visibility::visible_events(
            &events, &before, &after,
        ))
    }))
}

pub fn explore_step_json(view_json: &str, unit: u32) -> Result<String, String> {
    let view = serde_json::from_str(view_json).map_err(|e| format!("invalid view: {e}"))?;
    to_json(&legatum_ai::explore_step(&view, UnitId(unit)))
}

/// Production orders for delegated human cities, from the exact fogged view supplied by
/// the client. `unchosen_cities` is the seam for the chooser's eventual per-city reason.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CityAutomationPlan {
    orders: Vec<Order>,
    unchosen_cities: Vec<legatum_core::CityId>,
}

pub fn city_automation_orders_json(view_json: &str, orders_json: &str) -> Result<String, String> {
    let view: legatum_core::PlayerView =
        serde_json::from_str(view_json).map_err(|e| format!("invalid view: {e}"))?;
    let orders: Vec<Order> =
        serde_json::from_str(orders_json).map_err(|e| format!("invalid orders: {e}"))?;
    let mut result = CityAutomationPlan {
        orders: Vec::new(),
        unchosen_cities: Vec::new(),
    };
    let mut planned = Vec::new();
    for city in view.cities.iter().filter(|c| c.owner == view.me) {
        let Some(detail) = city.detail.as_ref() else {
            continue;
        };
        let focus = orders
            .iter()
            .rev()
            .find_map(|o| match o {
                Order::SetCityAutomation {
                    city: id, focus, ..
                } if *id == city.id => Some(*focus),
                _ => None,
            })
            .unwrap_or(detail.build_automation);
        let Some(focus) = focus else { continue };
        let changing = orders.iter().any(|o| matches!(o, Order::SetCityAutomation { city: id, focus: next, .. } if *id == city.id && *next != detail.build_automation));
        let replacing_automatic_tail = changing
            && detail.production_queue.len() == 1
            && detail.auto_queued == detail.production_queue.first().copied();
        // A city frozen by the occupation lock builds nothing, so it is never a choice to make.
        if detail.resistance > 0 || city.occupation_lock.is_some() || (!detail.production_queue.is_empty() && !replacing_automatic_tail)
            || orders.iter().any(|o| matches!(o, Order::SetProduction { city: id, .. } | Order::SetProductionQueue { city: id, .. } if *id == city.id))
        { continue }
        let next = match detail.producing {
            None => true,
            Some(legatum_core::ProductionItem::Project { .. }) => true,
            Some(_) => detail.turns_to_complete == Some(1),
        };
        if !next {
            continue;
        }
        let Some(item) =
            legatum_ai::city_automation_choice_with_pipeline(&view, city.id, focus, &planned)
        else {
            result.unchosen_cities.push(city.id);
            continue;
        };
        planned.push(item);
        if let Some(current) = detail
            .producing
            .filter(|item| !matches!(item, legatum_core::ProductionItem::Project { .. }))
        {
            result.orders.push(Order::SetProductionQueue {
                city: city.id,
                queue: vec![current, item],
            });
        } else {
            result.orders.push(Order::SetProduction {
                city: city.id,
                item,
            });
        }
        result.orders.push(Order::SetCityAutomation {
            city: city.id,
            focus: Some(focus),
            picked: Some(item),
        });
    }
    to_json(&result)
}

#[wasm_bindgen]
pub fn city_automation_orders(view_json: &str, orders_json: &str) -> Result<String, JsError> {
    js(guard(|| {
        city_automation_orders_json(view_json, orders_json)
    }))
}
