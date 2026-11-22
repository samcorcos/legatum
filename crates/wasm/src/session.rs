//! The game loop shared by the headless runner and the WASM API: a state plus one AI brain per
//! AI-controlled player. Both targets run exactly this code, so native and WASM play the same
//! game from the same setup and orders.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameSetup, GameState, Order, PlayerId, new_game, resolve_turn,
    try_new_game,
};
use std::cell::Cell;
use std::collections::BTreeMap;

pub type OrderMap = BTreeMap<PlayerId, Vec<Order>>;

/// The seed for one player's brain: derived from the game seed and the player id only.
pub fn brain_seed(game_seed: u64, player: PlayerId) -> u64 {
    Rng::derive(game_seed, &[purpose::AI, player.0 as u64]).next_u64()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BrainKind {
    Scripted,
    Random,
    Proxy,
    ProxyV2,
}

impl BrainKind {
    pub fn name(self) -> &'static str {
        match self {
            BrainKind::Scripted => "scripted",
            BrainKind::Random => "random",
            BrainKind::Proxy | BrainKind::ProxyV2 => "proxy",
        }
    }
}

/// Scripted seats play the scripted brain at their difficulty. Hard v2 (league-tuned openings,
/// `legatum_ai::hard_v2`) replaces Hard only when the setup opts in (`hard_v2_ai`), which the
/// app never does: it made Hard easier for a strong player (2026-09-30 validation).
pub fn make_brain(kind: BrainKind, state: &GameState, player: PlayerId) -> Box<dyn Brain> {
    let seed = brain_seed(state.setup.seed, player);
    let faction = state.player(player).faction;
    match kind {
        BrainKind::Scripted => match state
            .setup
            .players
            .get(player.0 as usize)
            .map(|p| p.difficulty)
            .unwrap_or_default()
        {
            AiDifficulty::Hard if state.setup.rule_experiments.hard_v2_ai => {
                legatum_ai::hard_v2::brain(faction, seed, player)
            }
            difficulty => Box::new(ScriptedBrain::with_difficulty(
                faction, seed, player, difficulty,
            )),
        },
        BrainKind::Random => Box::new(legatum_ai::RandomBrain::new(seed, player)),
        BrainKind::Proxy => Box::new(legatum_ai::ProxyBrain),
        BrainKind::ProxyV2 => Box::new(legatum_ai::ProxyV2Brain),
    }
}

/// Milliseconds on a monotonic clock, for per-seat planning times (`seat-timing` builds only;
/// zero otherwise). WASM reads `performance.now()`, which `Instant` cannot.
fn clock_ms() -> f64 {
    #[cfg(all(feature = "seat-timing", target_arch = "wasm32"))]
    {
        use wasm_bindgen::JsCast;
        let performance = js_sys::Reflect::get(&js_sys::global(), &"performance".into());
        if let Ok(p) = performance
            && let Ok(f) = js_sys::Reflect::get(&p, &"now".into())
            && let Ok(v) = f.unchecked_into::<js_sys::Function>().call0(&p)
        {
            return v.as_f64().unwrap_or(0.0);
        }
        0.0
    }
    #[cfg(all(feature = "seat-timing", not(target_arch = "wasm32")))]
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1000.0
    }
    #[cfg(not(feature = "seat-timing"))]
    {
        0.0
    }
}

pub struct Session {
    pub state: GameState,
    pub history: Option<crate::history::Recording>,
    brains: BTreeMap<PlayerId, (BrainKind, Box<dyn Brain>)>,
    /// Each AI seat's city yields, reused across its view builds (never changes a view).
    view_caches: BTreeMap<PlayerId, legatum_core::ViewCache>,
    /// The hash of the current state, filled by a recorded step or on demand by Game::hash.
    cached_hash: Cell<Option<u64>>,
    /// Each AI seat's planning time in ms (view plus brain) in the last `ai_orders`, and whether
    /// it ran a search. Filled only with the `seat-timing` feature (measurement builds).
    pub last_seat_ms: Vec<(PlayerId, f64, bool)>,
    /// Training mode: the AI seats' orders for `turn`, computed once at the start of the turn
    /// (so the search can run while the human plans), with the searched battles applied.
    pending: Option<(u32, OrderMap)>,
    /// Searched engagements waiting for their realised outcome.
    pub outcomes: legatum_ai::training::OutcomeTracker,
    /// Training mode: this turn's battle search when it runs here (WASM, in slices).
    pub training_run: Option<legatum_ai::training::TrainingRun>,
    /// Coach questions: per human seat, the shadow Hard brain that plans the seat to compare
    /// with his orders, and what it has seen (`legatum_ai::why`). Never saved; a loaded game
    /// starts them cold. The shadow has its own seed, so the game's RNG and hash are untouched.
    pub why: BTreeMap<PlayerId, (legatum_ai::ScriptedBrain, legatum_ai::why::History)>,
}

impl Session {
    /// A new game. `brain_for` picks the brain of each non-human player (by id).
    pub fn new(setup: &GameSetup, brain_for: impl Fn(PlayerId) -> BrainKind) -> Self {
        Self::from_state(new_game(setup), brain_for)
    }

    pub fn try_new(
        setup: &GameSetup,
        brain_for: impl Fn(PlayerId) -> BrainKind,
    ) -> Result<Self, String> {
        Ok(Self::from_state(try_new_game(setup)?, brain_for))
    }

    /// Create fresh brains for a raw state. Use `from_save_json` to resume AI memory.
    pub fn from_state(state: GameState, brain_for: impl Fn(PlayerId) -> BrainKind) -> Self {
        let brains = state
            .players
            .iter()
            .filter(|p| !p.is_human)
            .map(|p| {
                let kind = brain_for(p.id);
                (p.id, (kind, make_brain(kind, &state, p.id)))
            })
            .collect();
        let history = crate::history::Recording::new(&state);
        let initial_hash = history.tape.initial_hash;
        Self {
            state,
            brains,
            history: Some(history),
            view_caches: BTreeMap::new(),
            cached_hash: Cell::new(Some(initial_hash)),
            last_seat_ms: Vec::new(),
            pending: None,
            outcomes: Default::default(),
            training_run: None,
            why: BTreeMap::new(),
        }
    }

    /// Swap the brain of an AI seat (the simulator plays scenarios with tuned parameters).
    pub fn replace_brain(&mut self, player: PlayerId, brain: Box<dyn Brain>) {
        if let Some((_, b)) = self.brains.get_mut(&player) {
            *b = brain;
        }
    }

    pub fn brain_kind(&self, player: PlayerId) -> Option<BrainKind> {
        self.brains.get(&player).map(|(k, _)| *k)
    }

    pub fn brain(&self, player: PlayerId) -> Option<&dyn Brain> {
        self.brains.get(&player).map(|(_, b)| b.as_ref())
    }

    pub fn is_ai(&self, player: PlayerId) -> bool {
        self.brains.contains_key(&player)
    }

    pub fn state_hash(&self) -> u64 {
        if let Some(hash) = self.cached_hash.get() {
            return hash;
        }
        let hash = self.state.hash();
        self.cached_hash.set(Some(hash));
        hash
    }

    pub fn invalidate_state_hash(&self) {
        self.cached_hash.set(None);
    }

    /// Training mode (the lobby's "Training" level, saved as `search` on a Hard seat): a Hard
    /// AI whose battles are searched (`legatum_ai::training`). The seat itself plays plain Hard.
    pub fn is_training_seat(&self, player: PlayerId) -> bool {
        self.is_ai(player)
            && self.state.player(player).alive
            && self
                .state
                .setup
                .players
                .get(player.0 as usize)
                .is_some_and(|p| p.search && p.difficulty == AiDifficulty::Hard && !p.is_human)
    }

    pub fn has_training(&self) -> bool {
        self.state
            .players
            .iter()
            .any(|p| self.is_training_seat(p.id))
    }

    /// Coach questions: plan `player`'s seat with Hard from his own view and compare with
    /// `his` orders. Read-only on the game state (see the field `why`).
    pub fn why_review(
        &mut self,
        player: PlayerId,
        his: &[Order],
        asked: &legatum_ai::why::Asked,
    ) -> legatum_ai::why::Review {
        let state = &self.state;
        let (brain, history) = self.why.entry(player).or_insert_with(|| {
            let seed = brain_seed(state.setup.seed, player) ^ 0x5748_595f_5452_4149;
            (
                legatum_ai::ScriptedBrain::with_difficulty(
                    state.player(player).faction,
                    seed,
                    player,
                    AiDifficulty::Hard,
                ),
                legatum_ai::why::History::default(),
            )
        });
        let view = legatum_core::player_view(state, player);
        let next_city_name = legatum_core::rules::cities::next_city_name(state, player);
        legatum_ai::why::review(brain, history, &view, his, asked, &next_city_name)
    }

    /// This turn's AI orders, computed once and kept until the turn resolves.
    pub fn pending_ai_orders(&mut self) -> &OrderMap {
        let turn = self.state.turn;
        if self.pending.as_ref().is_none_or(|(t, _)| *t != turn) {
            let orders = self.ai_orders();
            self.pending = Some((turn, orders));
        }
        &self.pending.as_ref().expect("pending orders").1
    }

    /// Hard's orders this turn for every training seat: what the battle search starts from.
    pub fn training_hard_orders(&mut self) -> BTreeMap<PlayerId, Vec<Order>> {
        let orders = self.pending_ai_orders().clone();
        orders
            .into_iter()
            .filter(|(id, _)| self.is_training_seat(*id))
            .collect()
    }

    /// Play the searched orders for this turn's training seats, and start tracking the
    /// searched engagements' outcomes. A result for another turn is refused.
    pub fn apply_training(
        &mut self,
        result: &legatum_ai::training::TrainingResult,
    ) -> Result<(), String> {
        let turn = self.state.turn;
        if result.turn != turn {
            return Err(format!(
                "training result for turn {}, game is on turn {turn}",
                result.turn
            ));
        }
        let seats: Vec<PlayerId> = result
            .orders
            .keys()
            .copied()
            .filter(|id| self.is_training_seat(*id))
            .collect();
        self.pending_ai_orders();
        let pending = &mut self.pending.as_mut().expect("pending orders").1;
        for id in seats {
            pending.insert(id, result.orders[&id].clone());
        }
        self.outcomes.open(&result.records);
        Ok(())
    }

    /// The AI orders to resolve this turn with: the pending ones (training) or fresh ones.
    pub fn take_ai_orders(&mut self) -> OrderMap {
        match self.pending.take() {
            Some((turn, orders)) if turn == self.state.turn => orders,
            _ => self.ai_orders(),
        }
    }

    /// Orders for every alive AI player, each computed from that player's own view.
    pub fn ai_orders(&mut self) -> OrderMap {
        let state = &self.state;
        let mut caches = std::mem::take(&mut self.view_caches);
        // Each alive AI seat with its brain and view cache, in seat order.
        let mut seats: Vec<_> = self
            .brains
            .iter_mut()
            .filter(|(id, _)| state.player(**id).alive)
            .map(|(id, (_, brain))| (*id, brain, caches.remove(id).unwrap_or_default()))
            .collect();
        if seats.is_empty() {
            self.view_caches = caches;
            return OrderMap::new();
        }
        // Shared by every seat's view of this state.
        let public = legatum_core::public_view(state);
        let plan =
            |(id, brain, cache): &mut (PlayerId, &mut Box<dyn Brain>, legatum_core::ViewCache)| {
                let start = clock_ms();
                // AI seats' views are never shown: build only what decisions read.
                let view = legatum_core::player_view_cached(
                    state,
                    *id,
                    &public,
                    legatum_core::ViewDetail::Decisions,
                    cache,
                );
                let searches = brain.search_events().len();
                let orders = brain.orders(&view);

                let timing = (
                    clock_ms() - start,
                    cfg!(feature = "seat-timing") && brain.search_events().len() > searches,
                );
                (*id, orders, timing)
            };
        // Each seat decides from its own view and its own brain, so seats can plan at once;
        // orders are still gathered by seat id. Native only, and only outside a rayon worker:
        // batch simulations already run one game per core.
        #[cfg(not(target_arch = "wasm32"))]
        let orders: Vec<_> = if seats.len() > 1 && rayon::current_thread_index().is_none() {
            use rayon::prelude::*;
            seats.par_iter_mut().map(plan).collect()
        } else {
            seats.iter_mut().map(plan).collect()
        };
        #[cfg(target_arch = "wasm32")]
        let orders: Vec<_> = seats.iter_mut().map(plan).collect();
        for (id, _, cache) in seats {
            caches.insert(id, cache);
        }
        self.view_caches = caches;
        if cfg!(feature = "seat-timing") {
            self.last_seat_ms = orders
                .iter()
                .map(|(id, _, (ms, searched))| (*id, *ms, *searched))
                .collect();
        }
        orders.into_iter().map(|(id, o, _)| (id, o)).collect()
    }

    /// Resolve one turn with the given orders. Returns the turn report.
    pub fn step(&mut self, orders: &OrderMap) -> Vec<Event> {
        let turn = self.state.turn;
        let mut effective = orders.clone();
        for player in self.state.players.iter().filter(|p| p.alive) {
            let unit_ids: Vec<_> = self
                .state
                .units
                .values()
                .filter(|u| {
                    u.owner == player.id
                        && u.standing_order == Some(legatum_core::StandingOrder::AutoFire)
                })
                .map(|u| u.id)
                .collect();
            let toggles: Vec<_> = effective
                .get(&player.id)
                .into_iter()
                .flatten()
                .filter_map(|o| match o {
                    Order::SetAutoFire {
                        unit,
                        enabled: true,
                        ..
                    } => Some(*unit),
                    _ => None,
                })
                .collect();
            if unit_ids.is_empty() && toggles.is_empty() {
                continue;
            }
            let view = legatum_core::player_view(&self.state, player.id);
            let os = effective.entry(player.id).or_default();
            for id in unit_ids {
                if os
                    .iter()
                    .any(|o| legatum_core::rules::units::order_unit(o) == Some(id))
                {
                    continue;
                }
                let u = &self.state.units[&id];
                if !self
                    .state
                    .cities
                    .values()
                    .any(|c| c.owner == player.id && c.pos == u.pos)
                {
                    os.push(Order::SetAutoFire {
                        unit: id,
                        enabled: false,
                        target: None,
                    });
                    continue;
                }
                if let Some(target) = legatum_ai::automatic_garrison_target(&view, id) {
                    os.push(Order::RangedAttack { unit: id, target });
                }
            }
            for order in os {
                if let Order::SetAutoFire {
                    unit,
                    enabled: true,
                    target,
                } = order
                {
                    *target = legatum_ai::automatic_garrison_target(&view, *unit);
                }
            }
        }
        let (next, events) = resolve_turn(&self.state, &effective);
        self.state = next;
        self.invalidate_state_hash();
        if let Some(history) = &mut self.history {
            let hash = self.state.hash();
            self.cached_hash.set(Some(hash));
            history.observe_with_hash(turn, &effective, &self.state, hash);
        }
        events
    }

    pub fn save_json(&self) -> String {
        let mut value = crate::json::state_to_value(&self.state);
        if let Some(history) = &self.history {
            value["archive"] = serde_json::to_value(history).expect("archive JSON");
        }
        let memories: BTreeMap<_, _> = self
            .brains
            .iter()
            .filter_map(|(id, (_, brain))| brain.opponent_memory().map(|memory| (*id, memory)))
            .collect();
        if !memories.is_empty() {
            value["ai_opponent_memory"] =
                serde_json::to_value(memories).expect("AI opponent memory JSON");
        }
        value.to_string()
    }

    pub fn from_save_json(
        save: &str,
        brain_for: impl Fn(PlayerId) -> BrainKind,
    ) -> Result<Self, String> {
        crate::json::check_json_size(save)?;
        let mut value: serde_json::Value = serde_json::from_str(save).map_err(|e| e.to_string())?;
        // Take the session's own fields off the tree; the rest is the engine state, decoded from
        // this same tree rather than by parsing the text a second time.
        let (archive, opponent_memory) = match value.as_object_mut() {
            Some(fields) => (
                fields.remove("archive"),
                fields.remove("ai_opponent_memory"),
            ),
            None => (None, None),
        };
        let state = crate::json::state_from_value(value, true)?;
        let archive = archive
            .map(|v| {
                let recording: crate::history::Recording =
                    serde_json::from_value(v).map_err(|e| format!("invalid saved archive: {e}"))?;
                recording.validate()?;
                Ok::<_, String>(recording)
            })
            .transpose()?;
        let mut session = Self::from_state(state, brain_for);
        // Brains with no assessment history learn from their next view.
        if let Some(value) = opponent_memory {
            let memories: BTreeMap<PlayerId, legatum_ai::OpponentMemory> =
                serde_json::from_value(value)
                    .map_err(|e| format!("invalid saved AI opponent memory: {e}"))?;
            for (id, memory) in memories {
                if let Some((_, brain)) = session.brains.get_mut(&id) {
                    brain.restore_opponent_memory(memory);
                }
            }
        }
        if let Some(history) = archive.filter(|h| h.compatible().is_ok()) {
            let hash = history
                .tape
                .turns
                .last()
                .map(|t| t.hash.clone())
                .unwrap_or_else(|| history.tape.initial_hash.to_string());
            if hash != session.state.hash().to_string() || history.tape.setup != session.state.setup
            {
                return Err("saved archive does not match game state".into());
            }
            session.history = Some(history);
        }
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_auto_fire_uses_the_ai_target_and_holds_without_orders() {
        let setup = GameSetup::standard(
            51,
            [legatum_core::Faction::Athens, legatum_core::Faction::India],
        );
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        let me = PlayerId(0);
        let city_pos = session
            .state
            .cities
            .values()
            .find(|c| c.owner == me)
            .unwrap()
            .pos;
        let bow = legatum_core::game::spawn_unit(
            &mut session.state,
            me,
            legatum_core::UnitKind::Archer,
            city_pos,
        );
        let mut session = Session::from_state(session.state, |_| BrainKind::Scripted);
        session.step(&OrderMap::from([(
            me,
            vec![Order::SetAutoFire {
                unit: bow,
                enabled: true,
                target: None,
            }],
        )]));
        assert_eq!(
            session.state.units[&bow].standing_order,
            Some(legatum_core::StandingOrder::AutoFire)
        );
        assert!(!session.state.units[&bow].needs_orders);

        let mut state = session.state;
        let target = city_pos
            .neighbors()
            .into_iter()
            .find(|h| state.map.get(*h).is_some() && !state.units.values().any(|u| u.pos == *h))
            .unwrap();
        state.map.get_mut(target).unwrap().terrain = legatum_core::Terrain::Grassland;
        let enemy = PlayerId(2);
        legatum_core::game::spawn_unit(&mut state, enemy, legatum_core::UnitKind::Spearman, target);
        let mine = state.team_of(me);
        let theirs = state.team_of(enemy);
        state
            .diplomacy
            .as_mut()
            .unwrap()
            .relations
            .iter_mut()
            .find(|r| (r.a == mine && r.b == theirs) || (r.a == theirs && r.b == mine))
            .unwrap()
            .state = legatum_core::diplomacy::RelationState::War;
        let mut session = Session::from_state(state, |_| BrainKind::Scripted);
        let events = session.step(&OrderMap::new());
        assert!(events.iter().any(|e| matches!(e, Event::Battle { attacker, ranged: true, .. } if attacker.unit() == Some(bow))));
        assert_eq!(session.state.units[&bow].pos, city_pos);
        assert_eq!(
            session.state.units[&bow].standing_order,
            Some(legatum_core::StandingOrder::AutoFire)
        );
        assert!(!session.state.units[&bow].needs_orders);
    }

    #[test]
    fn app_hard_is_the_scripted_brain_and_hard_v2_needs_the_setup_switch() {
        let mut setup = GameSetup::all_ai(11);
        for p in &mut setup.players {
            p.difficulty = legatum_core::AiDifficulty::Hard;
        }
        let s = Session::new(&setup, |_| BrainKind::Scripted);
        assert!(
            s.state
                .players
                .iter()
                .all(|p| s.brain(p.id).unwrap().opening().is_none())
        );
        setup.rule_experiments.hard_v2_ai = true;
        let s = Session::new(&setup, |_| BrainKind::Scripted);
        for p in &s.state.players {
            let drawn = legatum_ai::hard_v2::draw_opening(brain_seed(setup.seed, p.id));
            let expected = (drawn != legatum_ai::openings::Opening::Default).then_some(drawn);
            assert_eq!(s.brain(p.id).unwrap().opening(), expected);
        }
    }

    /// A 1v1 on a small map: seat 0 plain Hard, seat 1 Hard (+Search when `search`). The seed is
    /// one where the search seat searches early; AI changes can move that, so this seed was
    /// rechecked after the 2026-10-07 Sparta campaign valuation change.
    fn duel_setup(search: bool) -> GameSetup {
        let mut setup = GameSetup::all_ai(7_100_010);
        setup.players.truncate(2);
        (setup.width, setup.height) = (40, 26);
        for p in &mut setup.players {
            p.difficulty = AiDifficulty::Hard;
        }
        setup.players[1].search = search;
        setup
    }

    fn play(session: &mut Session, turns: u32) {
        for _ in 0..turns {
            let orders = session.ai_orders();
            session.step(&orders);
        }
    }

    #[test]
    fn search_flag_is_omitted_when_off_and_round_trips_when_on() {
        let plain = duel_setup(false);
        assert!(
            !serde_json::to_string(&plain).unwrap().contains("search"),
            "an off flag must leave saves and hashes unchanged"
        );
        let json = serde_json::to_string(&duel_setup(true)).unwrap();
        assert!(json.contains("\"search\":true"));
        let back: GameSetup = serde_json::from_str(&json).unwrap();
        assert_eq!(back, duel_setup(true));
        // A setup from before the flag existed reads as off.
        let old: GameSetup = serde_json::from_str(&serde_json::to_string(&plain).unwrap()).unwrap();
        assert!(old.players.iter().all(|p| !p.search));
    }

    #[test]
    fn a_training_seat_plays_hard_with_its_battles_searched_and_replays_exactly() {
        use legatum_ai::training::{TrainingConfig, search_turn};
        let cfg = TrainingConfig {
            rollouts: 4,
            screen: 2,
            horizon: 3,
            max_passes: 1,
            max_battles: 1,
            cap_ms: 0,
            ..TrainingConfig::default()
        };
        let mut a = Session::new(&duel_setup(true), |_| BrainKind::Scripted);
        let mut b = Session::new(&duel_setup(true), |_| BrainKind::Scripted);
        assert!(a.is_training_seat(PlayerId(1)));
        assert!(!a.is_training_seat(PlayerId(0)));
        // The flagged seat is plain Hard: its unsearched orders match a plain Hard game's.
        let mut plain = Session::new(&duel_setup(false), |_| BrainKind::Scripted);
        assert_eq!(a.pending_ai_orders().clone(), plain.ai_orders());
        let turn = |s: &mut Session| {
            let hard = s.training_hard_orders();
            let r = search_turn(&s.state, hard, &cfg, || 0, |_| {});
            s.apply_training(&r).unwrap();
            let orders = s.take_ai_orders();
            s.step(&orders);
            (orders, r.records.len())
        };
        let mut searched = 0;
        let mut after = 0;
        while !a.state.finished && a.state.turn < 400 && after < 5 {
            let (oa, n) = turn(&mut a);
            let (ob, _) = turn(&mut b);
            assert_eq!(oa, ob, "turn {}", a.state.turn);
            assert_eq!(a.state.hash(), b.state.hash());
            searched += n;
            if searched > 0 {
                after += 1;
            }
        }
        assert!(
            searched > 0,
            "the training seat must have fought within 400 turns"
        );
    }

    fn neighboring_hard_session() -> Session {
        use legatum_core::{
            Faction, Hex, Terrain, UnitKind, game, rules::cities::found_city, state::Map,
        };
        let mut setup = GameSetup::all_ai(5400001);
        setup.players.truncate(2);
        setup.width = 20;
        setup.height = 14;
        setup.barbarians = false;
        setup.players[0].faction = Faction::Mongols;
        setup.players[1].faction = Faction::Sparta;
        for player in &mut setup.players {
            player.difficulty = AiDifficulty::Hard;
        }
        let mut state = game::blank_game(&setup, Map::new(20, 14, Terrain::Plains));
        for (seat, position) in [(0, Hex::new(3, 5)), (1, Hex::new(8, 5))] {
            let settler = game::spawn_unit(&mut state, PlayerId(seat), UnitKind::Settler, position);
            found_city(&mut state, settler, &mut Vec::new()).unwrap();
        }
        game::spawn_unit(&mut state, PlayerId(0), UnitKind::Scout, Hex::new(5, 5));
        game::spawn_unit(&mut state, PlayerId(1), UnitKind::Spearman, Hex::new(6, 5));
        let mut session = Session::from_state(state, |_| BrainKind::Scripted);
        // Hand-built state has no generated-map archive; this tests the actual session save path.
        session.history = None;
        session
    }

    #[test]
    fn scripted_opponent_memory_survives_every_turn_save_load_and_assessment_cadence() {
        let mut uninterrupted = neighboring_hard_session();
        let mut resumed =
            Session::from_save_json(&uninterrupted.save_json(), |_| BrainKind::Scripted).unwrap();
        resumed.history = None;
        let mut saw_cached_assessment = false;
        let mut saw_refreshed_assessment = false;
        for _ in 0..12 {
            let turn = uninterrupted.state.turn;
            let expected = uninterrupted.ai_orders();
            // Save before planning, then restore before every decision including first contact.
            resumed =
                Session::from_save_json(&resumed.save_json(), |_| BrainKind::Scripted).unwrap();
            resumed.history = None;
            let actual = resumed.ai_orders();
            assert_eq!(expected, actual, "orders after reload on turn {turn}");
            for player in &uninterrupted.state.players {
                let memory = uninterrupted.brain(player.id).unwrap().opponent_memory();
                assert_eq!(memory, resumed.brain(player.id).unwrap().opponent_memory());
                let memory = memory.unwrap();
                for rival in memory.assessments.rivals.values() {
                    saw_cached_assessment |= rival.assessed_turn < turn;
                    saw_refreshed_assessment |= rival.assessed_turn > rival.first_contact_turn;
                }
            }
            let me = uninterrupted
                .brain(PlayerId(0))
                .unwrap()
                .opponent_memory()
                .unwrap();
            assert!(
                !me.assessments.rivals.is_empty(),
                "fixture must make actual contact"
            );
            assert!(
                !me.switches.is_empty(),
                "fixture must exercise persisted adaptation"
            );
            // A second reload after planning also retains the exact snapshot and orders.
            let mut after_plan =
                Session::from_save_json(&resumed.save_json(), |_| BrainKind::Scripted).unwrap();
            assert_eq!(after_plan.ai_orders(), actual);
            uninterrupted.step(&expected);
            resumed.step(&actual);
            assert_eq!(uninterrupted.state.hash(), resumed.state.hash());
        }
        assert!(saw_cached_assessment, "must cross turns between refreshes");
        assert!(
            saw_refreshed_assessment,
            "must cross assessment refresh cadence"
        );
    }

    #[test]
    fn legacy_save_without_opponent_memory_loads_and_invalid_memory_is_rejected() {
        let mut session = neighboring_hard_session();
        session.ai_orders();
        let mut value: serde_json::Value = serde_json::from_str(&session.save_json()).unwrap();
        assert!(
            value
                .as_object_mut()
                .unwrap()
                .remove("ai_opponent_memory")
                .is_some()
        );
        let legacy = Session::from_save_json(&value.to_string(), |_| BrainKind::Scripted).unwrap();
        assert_eq!(legacy.state.hash(), session.state.hash());
        assert!(
            legacy
                .brain(PlayerId(0))
                .unwrap()
                .opponent_memory()
                .unwrap()
                .assessments
                .rivals
                .is_empty()
        );
        value["ai_opponent_memory"] = serde_json::json!({"0": "invalid"});
        let result = Session::from_save_json(&value.to_string(), |_| BrainKind::Scripted);
        assert!(matches!(result, Err(error) if error.contains("invalid saved AI opponent memory")));
    }

    #[test]
    fn save_round_trips_to_identical_text_and_state_hash() {
        let mut session = scripted_game(5, 8);
        session.ai_orders();
        let saved = session.save_json();
        // The text the save path wrote before it reused the state's JSON tree.
        let mut before: serde_json::Value =
            serde_json::from_str(&crate::json::state_to_json(&session.state)).unwrap();
        before["archive"] = serde_json::to_value(session.history.as_ref().unwrap()).unwrap();
        let memories: BTreeMap<_, _> = session
            .brains
            .iter()
            .filter_map(|(id, (_, brain))| brain.opponent_memory().map(|m| (*id, m)))
            .collect();
        assert!(!memories.is_empty(), "fixture must save opponent memory");
        before["ai_opponent_memory"] = serde_json::to_value(memories).unwrap();
        assert_eq!(saved, before.to_string());
        let loaded = Session::from_save_json(&saved, |_| BrainKind::Scripted).unwrap();
        assert_eq!(loaded.state, session.state);
        assert_eq!(loaded.state.hash(), session.state.hash());
        assert_eq!(loaded.history, session.history);
        assert_eq!(loaded.save_json(), saved);
    }

    fn scripted_game(seed: u64, turns: u32) -> Session {
        let mut session = Session::new(&GameSetup::all_ai(seed), |_| BrainKind::Scripted);
        play(&mut session, turns);
        session
    }

    #[test]
    fn game_history_has_one_entry_per_turn_survives_saves_and_is_deterministic() {
        use legatum_core::history::Chronicle;
        let session = scripted_game(31, 60);
        let h = &session.state.history;
        assert_eq!(session.state.turn, 61);
        assert_eq!(h.first_turn, 1);
        assert_eq!(h.empires.len(), session.state.players.len());
        for e in &h.empires {
            for len in [
                e.vp.len(),
                e.cities.len(),
                e.population.len(),
                e.science.len(),
                e.gold.len(),
                e.army.len(),
                e.techs.len(),
                e.era.len(),
            ] {
                assert_eq!(len, 60);
            }
        }
        for p in &session.state.players {
            let founded = h
                .chronicle
                .iter()
                .any(|c| matches!(c, Chronicle::Founded { player, .. } if *player == p.id));
            assert!(
                founded || !p.alive,
                "{} is alive and never founded a city",
                p.name
            );
        }
        // The history agrees with the telemetry the archive recorded for the same turns.
        let recording = session.history.as_ref().unwrap();
        for (i, frame) in recording.telemetry.iter().skip(1).enumerate() {
            for (e, p) in h.empires.iter().zip(&frame.players) {
                assert_eq!(
                    (e.vp[i], e.cities[i], e.gold[i], e.techs[i]),
                    (p.vp, p.cities, p.gold, p.techs)
                );
            }
        }
        // Saving and loading keeps the history and the state exactly.
        let resumed =
            Session::from_save_json(&session.save_json(), |_| BrainKind::Scripted).unwrap();
        assert_eq!(resumed.state.history, session.state.history);
        assert_eq!(resumed.state.hash(), session.state.hash());
        // A compatible archive is kept as it was (its tape hashes include the history), not rebuilt.
        assert_eq!(
            resumed.history.as_ref().unwrap().metadata.game_id,
            recording.metadata.game_id
        );
        // The same seed gives the same history.
        assert_eq!(scripted_game(31, 60).state.history, session.state.history);
    }

    #[test]
    fn brain_seeds_differ_per_player_and_game() {
        let a = brain_seed(1, PlayerId(0));
        assert_eq!(a, brain_seed(1, PlayerId(0)));
        assert_ne!(a, brain_seed(1, PlayerId(1)));
        assert_ne!(a, brain_seed(2, PlayerId(0)));
    }

    fn why_setup() -> GameSetup {
        GameSetup::standard(
            7_100_011,
            [legatum_core::Faction::Athens, legatum_core::Faction::India],
        )
    }

    #[test]
    fn planning_his_seat_leaves_the_state_hash_and_play_unchanged() {
        let mut watched = Session::new(&why_setup(), |_| BrainKind::Scripted);
        let mut plain = Session::new(&why_setup(), |_| BrainKind::Scripted);
        assert!(!watched.is_ai(PlayerId(0)));
        let asked = legatum_ai::why::Asked::default();
        for _ in 0..25 {
            let before = watched.state.hash();
            let state_before = serde_json::to_string(&watched.state).unwrap();
            watched.why_review(PlayerId(0), &[], &asked);
            assert_eq!(watched.state.hash(), before);
            assert_eq!(serde_json::to_string(&watched.state).unwrap(), state_before);
            let (oa, ob) = (watched.ai_orders(), plain.ai_orders());
            assert_eq!(oa, ob, "turn {}: AI orders moved", watched.state.turn);
            watched.step(&oa);
            plain.step(&ob);
            assert_eq!(watched.state.hash(), plain.state.hash());
        }
        assert_eq!(
            crate::json::state_to_json(&watched.state),
            crate::json::state_to_json(&plain.state)
        );
    }

    #[test]
    fn a_war_hard_would_not_declare_becomes_a_war_and_peace_question() {
        use legatum_ai::why::Category;
        let mut probe = Session::new(&why_setup(), |_| BrainKind::Scripted);
        let none = legatum_ai::why::Asked::default();
        let hard = probe.why_review(PlayerId(0), &[], &none).hard_orders;
        let enemy = legatum_core::TeamId(1);
        assert!(!hard.contains(&Order::DeclareWar {
            against: enemy,
            breach: false
        }));
        let mut s = Session::new(&why_setup(), |_| BrainKind::Scripted);
        let mut his = hard.clone();
        his.push(Order::DeclareWar {
            against: enemy,
            breach: false,
        });
        let review = s.why_review(PlayerId(0), &his, &none);
        let q = review.question.expect("a divergence on war");
        assert_eq!(q.category, Category::WarPeace);
        assert_eq!(q.key, "warpeace:declare:1");
        assert!(q.text.contains("declared war"), "{}", q.text);
        // Already asked: nothing comes back for the same standing fact.
        let again = legatum_ai::why::Asked {
            keys: vec![q.key.clone()],
            last: None,
            skipped: vec![],
        };
        let mut s2 = Session::new(&why_setup(), |_| BrainKind::Scripted);
        assert!(s2.why_review(PlayerId(0), &his, &again).question.is_none());
        // Same category on consecutive turns is skipped.
        let last = legatum_ai::why::Asked {
            keys: vec![],
            last: Some((0, Category::WarPeace)),
            skipped: vec![],
        };
        let mut s3 = Session::new(&why_setup(), |_| BrainKind::Scripted);
        assert!(s3.why_review(PlayerId(0), &his, &last).question.is_none());
    }

    #[test]
    fn hard_opens_with_capital_production_and_no_founding_order() {
        let mut s = Session::new(&why_setup(), |_| BrainKind::Scripted);
        let none = legatum_ai::why::Asked::default();
        let review = s.why_review(PlayerId(0), &[], &none);
        assert!(s.state.capital_of(PlayerId(0)).is_some());
        assert!(
            review
                .hard_orders
                .iter()
                .any(|o| matches!(o, Order::SetProduction { .. }))
        );
        assert!(
            !review
                .hard_orders
                .iter()
                .any(|o| matches!(o, Order::FoundCity { .. }))
        );
        // Matching every Hard order makes no divergence.
        let mut t = Session::new(&why_setup(), |_| BrainKind::Scripted);
        let same = t.why_review(PlayerId(0), &review.hard_orders, &none);
        assert!(
            same.question.is_none(),
            "{:?}",
            same.question.map(|q| q.text)
        );
    }
}
