//! Fable seat: a file-based step loop that lets a language model play one seat turn by turn,
//! seeing only that seat's `PlayerView` as text, against Hard in every other seat.
//!
//!   (order verb `explore U` asks the app's auto-explore helper for this turn's Scout step)
//!   fable_seat init --dir D (--scenario FILE | --new-game --seed S --faction F) [--seat N] [--run R]
//!   fable_seat view --dir D [--city ID] [--all] [--map all|none]
//!   fable_seat check --dir D --orders FILE      validate orders and preview paths; resolves nothing
//!   fable_seat step --dir D --orders FILE [--force]
//!   fable_seat hard --dir D                     what Hard would order for the seat this turn
//!   fable_seat autoplay --dir D [--turns N] [--until T]   Hard plays the seat
//!   fable_seat fork --dir D --to D2 [--turn T]  copy a game, cut back to the start of turn T
//!   fable_seat score --dir D                    one JSON line: scenario status or VP and cities
//!
//! The directory holds `config.json` and `tape.jsonl` (one line per resolved turn: the seat's
//! orders, the orders the seat's shadow Hard brain gave from the same view, and the state hash).
//! Nothing else is saved: every command rebuilds the game by replaying the tape from the start,
//! so every Hard brain (including the seat's shadow) keeps its real memory. The replay checks
//! each turn's hash and stops on a mismatch.
//!
//! The seat is an AI seat for the engine (`is_human` false), exactly as in `legatum-sim scenario`
//! baselines, so its results compare with Hard's on the same card. `--run R` is that command's run
//! index (`run_seed`): the same map and position, different brain seeds and dice.
//!
//! Orders file: one order per line, `#` starts a comment.
//!   move U q,r | shoot U q,r | sleep U | heal U | skip U | found U | disband U | upgrade U
//!   siege U | autofire U on|off | build C Item | queue C Item, Item | buy C Item | raze C
//!   research Tech [Tech...] | war T | peace T | ally T | accept P | reject P
//!   air P q,r | rebase P C | upgradeair P | raw {"type":...}
use legatum_core::forecast::{forecast_air_strike, forecast_attack};
use legatum_core::scenario::Outcome;
use legatum_core::view::PlayerView;
use legatum_core::{
    AiDifficulty, Event, Faction, GameSetup, GameState, Hex, Order, PlayerId, ProductionItem,
    Terrain, new_game, player_view, scoring, validate_orders,
};
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::{BrainKind, Session};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn die(msg: impl AsRef<str>) -> ! {
    eprintln!("fable_seat: {}", msg.as_ref());
    std::process::exit(2);
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(rest: &[String]) -> Self {
        let mut map = BTreeMap::new();
        let mut i = 0;
        while i < rest.len() {
            let key = rest[i].trim_start_matches("--").to_string();
            let flag = matches!(key.as_str(), "new-game" | "force" | "all");
            if flag {
                map.insert(key, "1".into());
                i += 1;
            } else {
                let value = rest
                    .get(i + 1)
                    .unwrap_or_else(|| die(format!("--{key} needs a value")));
                map.insert(key, value.clone());
                i += 2;
            }
        }
        Args(map)
    }
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
    fn need(&self, key: &str) -> &str {
        self.get(key)
            .unwrap_or_else(|| die(format!("missing --{key}")))
    }
    fn num<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.get(key).map(|v| {
            v.parse()
                .unwrap_or_else(|_| die(format!("--{key}: bad number {v}")))
        })
    }
}

struct Entry {
    by: String,
    orders: Vec<Order>,
    hard: Vec<Order>,
    hash: String,
}

struct Game {
    session: Session,
    active: Option<ActiveScenario>,
    seat: PlayerId,
    played: u32,
}

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| die(format!("read {path}: {e}")))
}

/// The seed of baseline run `run`, as in `legatum-sim scenario` (crates/sim/src/scenario.rs).
fn run_seed(seed: u64, run: u32) -> u64 {
    if run == 0 {
        seed
    } else {
        legatum_core::rng::Rng::derive(seed, &[0x5CE4_A210, u64::from(run)]).next_u64()
    }
}

fn start_game(config: &Value) -> Game {
    let run = config["run"].as_u64().unwrap_or(0) as u32;
    let (mut state, file): (GameState, Option<ScenarioFile>) =
        if let Some(path) = config["scenario"].as_str() {
            let file = ScenarioFile::parse(&read(path)).unwrap_or_else(|e| die(e));
            (file.prepare(false).unwrap_or_else(|e| die(e)), Some(file))
        } else {
            let setup: GameSetup = legatum_wasm::json::setup_from_value(config["setup"].clone())
                .unwrap_or_else(|e| die(e));
            (new_game(&setup), None)
        };
    state.setup.seed = run_seed(state.setup.seed, run);
    let seat = match &file {
        Some(f) => f.seat,
        None => PlayerId(config["seat"].as_u64().unwrap_or(0) as u8),
    };
    let active = file.map(|f| ActiveScenario::new(f, state.clone()));
    let mut session = Session::from_state(state, |_| BrainKind::Scripted);
    session.history = None;
    Game {
        session,
        active,
        seat,
        played: 0,
    }
}

impl Game {
    /// Resolve one turn. `mine` replaces the seat's orders; `None` lets its shadow Hard play.
    /// Returns the events and the shadow's orders.
    fn play(&mut self, mine: Option<&[Order]>) -> (Vec<Event>, Vec<Order>) {
        let mut orders = self.session.ai_orders();
        if let Some(r) = self.active.as_ref().and_then(|a| a.file.rusher.as_ref()) {
            legatum_wasm::rusher::apply(&mut orders, &self.session.state, r);
        }
        let shadow = orders.get(&self.seat).cloned().unwrap_or_default();
        if let Some(mine) = mine {
            orders.insert(self.seat, mine.to_vec());
        }
        let events = self.session.step(&orders);
        self.session.invalidate_state_hash();
        self.played += 1;
        (events, shadow)
    }

    fn hash(&self) -> String {
        self.session.state.hash().to_string()
    }

    fn over(&self) -> Option<String> {
        let state = &self.session.state;
        if let Some(a) = &self.active {
            let s = a.status(state);
            if s.outcome != Outcome::Playing || self.played > a.file.objective.turns {
                return Some(format!(
                    "SCENARIO OVER: {:?} score {} ({})",
                    s.outcome, s.score, s.reason
                ));
            }
        }
        if state.finished {
            return Some(format!("GAME OVER: winner team {:?}", state.winner));
        }
        if !state.player(self.seat).alive {
            return Some("GAME OVER: your empire was eliminated".into());
        }
        None
    }
}

fn load_tape(dir: &str) -> Vec<Entry> {
    let path = format!("{dir}/tape.jsonl");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: Value = serde_json::from_str(l).unwrap_or_else(|e| die(format!("tape: {e}")));
            let orders = |k: &str| -> Vec<Order> {
                serde_json::from_value(v[k].clone())
                    .unwrap_or_else(|e| die(format!("tape {k}: {e}")))
            };
            Entry {
                by: v["by"].as_str().unwrap_or("me").to_string(),
                orders: orders("orders"),
                hard: orders("hard"),
                hash: v["hash"].as_str().unwrap_or("").to_string(),
            }
        })
        .collect()
}

fn append_tape(dir: &str, turn: u32, entry: &Entry) {
    use std::io::Write;
    let line = json!({
        "turn": turn, "by": entry.by, "orders": entry.orders, "hard": entry.hard,
        "hash": entry.hash,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/tape.jsonl"))
        .unwrap_or_else(|e| die(format!("tape: {e}")));
    writeln!(f, "{line}").unwrap_or_else(|e| die(format!("tape: {e}")));
}

fn load(dir: &str) -> Game {
    let config: Value = serde_json::from_str(&read(&format!("{dir}/config.json")))
        .unwrap_or_else(|e| die(format!("config: {e}")));
    let mut game = start_game(&config);
    for (i, entry) in load_tape(dir).iter().enumerate() {
        let mine = (entry.by == "me").then_some(entry.orders.as_slice());
        let (_, shadow) = game.play(mine);
        if shadow != entry.hard || game.hash() != entry.hash {
            die(format!(
                "replay diverged at tape line {} (turn {}): the game is not deterministic here",
                i + 1,
                game.session.state.turn
            ));
        }
    }
    game
}

// ---- text ----

fn hx(h: Hex) -> String {
    format!("{},{}", h.q, h.r)
}

/// Compact text for any serialized value: hexes as `q,r`, objects as `k=v`.
fn fv(v: &Value) -> String {
    match v {
        Value::Null => "-".into(),
        Value::String(s) => s.clone(),
        Value::Array(a) => format!("[{}]", a.iter().map(fv).collect::<Vec<_>>().join(" ")),
        Value::Object(o) => {
            if o.len() == 2
                && let (Some(q), Some(r)) = (o.get("q"), o.get("r"))
                && q.is_number()
            {
                return format!("{q},{r}");
            }
            let parts: Vec<_> = o
                .iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| format!("{k}={}", fv(v)))
                .collect();
            format!("{{{}}}", parts.join(" "))
        }
        other => other.to_string(),
    }
}

fn item_name(item: &ProductionItem) -> String {
    match item {
        ProductionItem::Unit { unit } => format!("{unit:?}"),
        ProductionItem::Building { building } => format!("{building:?}"),
        ProductionItem::Wonder { building, tile } => format!("{building:?}@{}", hx(*tile)),
        other => {
            let v = serde_json::to_value(other).unwrap_or_default();
            let mut name = String::new();
            let mut at = String::new();
            if let Value::Object(o) = &v {
                for (k, v) in o {
                    match v {
                        Value::String(s) if k != "type" && k != "kind" => name.push_str(s),
                        Value::Object(_) => at = format!("@{}", fv(v)),
                        _ => {}
                    }
                }
            }
            format!("{name}{at}")
        }
    }
}

fn unit_label(view: &PlayerView, id: u64) -> String {
    if let Some(u) = view.units.iter().find(|u| u64::from(u.id.0) == id) {
        return format!("U{id}:{:?}", u.kind);
    }
    if let Some(a) = view.aircraft.iter().find(|a| u64::from(a.id.0) == id) {
        return format!("U{id}:{:?}", a.kind);
    }
    format!("U{id}")
}

fn city_label(view: &PlayerView, id: u64) -> String {
    match view.cities.iter().find(|c| u64::from(c.id.0) == id) {
        Some(c) => format!("C{id}:{}", c.name),
        None => format!("C{id}"),
    }
}

/// One line for an order or an event: its type, then its fields, with unit and city ids named.
fn describe(views: &[&PlayerView], v: &Value) -> String {
    let (name, fields) = match v {
        Value::Object(o) if o.get("type").is_some_and(Value::is_string) => {
            (o["type"].as_str().unwrap_or("").to_string(), o.clone())
        }
        Value::Object(o) if o.len() == 1 => {
            let (k, inner) = o.iter().next().expect("one key");
            (k.clone(), inner.as_object().cloned().unwrap_or_default())
        }
        Value::String(s) => (s.clone(), Default::default()),
        other => (fv(other), Default::default()),
    };
    let mut out = name;
    for (k, val) in &fields {
        if k == "type" || val.is_null() {
            continue;
        }
        let text = match val.as_u64() {
            Some(id)
                if matches!(
                    k.as_str(),
                    "unit" | "attacker" | "defender" | "plane" | "killer" | "target_unit"
                ) =>
            {
                views
                    .iter()
                    .map(|view| unit_label(view, id))
                    .find(|l| l.contains(':'))
                    .unwrap_or(format!("U{id}"))
            }
            Some(id) if k == "city" => city_label(views[0], id),
            _ => fv(val),
        };
        let _ = write!(out, " {k}={text}");
    }
    out
}

fn terrain_char(t: Terrain) -> char {
    match t {
        Terrain::Ocean => '~',
        Terrain::Coast => '-',
        Terrain::Grassland => 'g',
        Terrain::Plains => 'p',
        Terrain::Forest => 'f',
        Terrain::Hills => 'h',
        Terrain::Desert => 'd',
        Terrain::Tundra => 't',
        Terrain::Mountains => '^',
    }
}

fn hostile(view: &PlayerView, owner: PlayerId) -> bool {
    owner != view.me && (owner.is_barbarian() || view.at_war(view.me, owner))
}

fn owner_name(view: &PlayerView, owner: PlayerId) -> String {
    if owner.is_barbarian() {
        return "Barbarian".into();
    }
    format!("{:?}", view.faction_of(owner))
}

struct ViewOpts {
    city: Option<u32>,
    all: bool,
    map: String,
}

fn render(view: &PlayerView, active: Option<&ActiveScenario>, state: &GameState, o: &ViewOpts) {
    let mut out = String::new();
    let me = view.me;
    let econ = &view.economy;
    let res = &view.research;
    let _ = writeln!(
        out,
        "== TURN {} == you are P{} {:?} (team {}) | gold {} ({:+}/turn, upkeep {}) | science {}/turn | era {}",
        view.turn,
        me.0,
        view.faction_of(me),
        view.team.0,
        econ.gold,
        econ.gold_per_turn.total,
        econ.upkeep,
        econ.science_per_turn.total,
        res.era_name,
    );
    match res.current {
        Some(t) => {
            let _ = writeln!(
                out,
                "research: {:?} {}/{} ({} turns) queue {:?}",
                t,
                res.progress,
                res.cost.unwrap_or(0),
                res.turns_left.unwrap_or(-1),
                res.queue
            );
        }
        None => {
            let _ = writeln!(out, "research: NOTHING SELECTED");
        }
    }
    if res.current.is_none() || o.all {
        let opts: Vec<_> = res
            .available
            .iter()
            .filter(|t| t.researchable)
            .map(|t| {
                format!(
                    "{:?} {}sci/{}t",
                    t.tech,
                    t.cost.total,
                    t.turns.unwrap_or(-1)
                )
            })
            .collect();
        let _ = writeln!(out, "  can research: {}", opts.join(" | "));
    }
    // The scenario's objective is public to the seat (the Lab shows it); it reads the state only
    // for what the seat's own score needs.
    if let Some(a) = active {
        let s = a.status(state);
        let _ = writeln!(
            out,
            "SCENARIO: {} | turns left {} (ends T{}) | score now {} | {:?}",
            a.file.objective.summary, s.turns_left, s.end_turn, s.score, s.outcome
        );
        for c in &s.conditions {
            let _ = writeln!(
                out,
                "  {} {}: {} [{}]",
                if c.keep { "KEEP" } else { "WIN" },
                c.label,
                c.progress,
                if c.met { "met" } else { "not met" }
            );
        }
        for l in &s.score_lines {
            let _ = writeln!(
                out,
                "  score {}: {} x {} = {}",
                l.label, l.value, l.weight, l.points
            );
        }
        for t in &s.targets {
            let _ = writeln!(
                out,
                "  target C{} {} at {} (owner P{})",
                t.city.0,
                t.name,
                hx(t.pos),
                t.owner.0
            );
        }
    }
    let _ = writeln!(
        out,
        "PLAYERS (vp target {:?}):",
        view.standings.first().map(|s| s.target)
    );
    for p in view.players.iter() {
        let rel = if p.id == me {
            "me".to_string()
        } else if !p.alive {
            "dead".into()
        } else if p.team == view.team {
            "TEAMMATE".into()
        } else if view.at_war(me, p.id) {
            "WAR".into()
        } else if !view.diplomacy.met.contains(&p.team) {
            "unmet".into()
        } else {
            let allied = view
                .diplomacy
                .alliances
                .iter()
                .any(|a| a.contains(&view.team) && a.contains(&p.team));
            if allied {
                "ALLY".into()
            } else {
                "peace".into()
            }
        };
        let _ = writeln!(
            out,
            "  P{} T{} {:?}: vp {} cities {} era {} power {} army {} econ {} [{}]",
            p.id.0, p.team.0, p.faction, p.vp, p.cities, p.era, p.power, p.army, p.economy, rel
        );
        if p.id == me {
            let src: Vec<_> = p
                .vp_sources
                .iter()
                .map(|(s, n)| format!("{s:?} {n}"))
                .collect();
            let _ = writeln!(out, "    my vp: {}", src.join(", "));
        }
    }
    for s in view.standings.iter() {
        if let Some(h) = &s.victory_hold {
            let _ = writeln!(
                out,
                "  VICTORY HOLD team {}: {}",
                s.team.0,
                fv(&serde_json::to_value(h).unwrap_or_default())
            );
        }
    }
    if let Some(g) = &view.faction_goal {
        let _ = writeln!(
            out,
            "faction goal: {} {}/{}{}",
            g.label,
            g.value,
            g.target,
            if g.done { " DONE" } else { "" }
        );
    }
    let objectives: Vec<_> = view
        .objective_progress
        .iter()
        .filter(|p| p.entered && !p.era_scored)
        .map(|p| {
            let text = view
                .eras
                .iter()
                .find(|e| e.era == p.era)
                .and_then(|e| e.objectives.iter().find(|x| x.slot == p.slot))
                .map(|x| x.text.clone())
                .unwrap_or_default();
            format!("era{} \"{}\" {}/{}", p.era, text, p.current, p.target)
        })
        .collect();
    if !objectives.is_empty() {
        let _ = writeln!(out, "era objectives open: {}", objectives.join(" | "));
    }
    for p in &view.diplomacy.proposals {
        let _ = writeln!(
            out,
            "PROPOSAL P{}: {:?} from team {} to team {} (answer by T{})",
            p.id.0, p.kind, p.from.0, p.to.0, p.respond_by
        );
    }
    for c in &view.diplomacy.calls {
        let _ = writeln!(
            out,
            "CALL TO ARMS: {}",
            fv(&serde_json::to_value(c).unwrap_or_default())
        );
    }
    if o.all {
        for a in &view.diplomacy.actions {
            let order = serde_json::to_value(&a.order).unwrap_or_default();
            let _ = writeln!(
                out,
                "  diplomacy option: {} cost {}{}{}",
                describe(&[view], &order),
                a.cost,
                if a.betrayal { " BETRAYAL" } else { "" },
                a.blocked_reason
                    .as_ref()
                    .map(|r| format!(" BLOCKED: {r}"))
                    .unwrap_or_default()
            );
        }
        let actions = serde_json::to_value(&view.actions).unwrap_or_default();
        let _ = writeln!(out, "  faction actions: {}", fv(&actions));
        if !view.supply.is_empty() {
            let _ = writeln!(
                out,
                "  supply: {}",
                fv(&serde_json::to_value(&view.supply).unwrap_or_default())
            );
        }
    }

    // My cities.
    let _ = writeln!(out, "MY CITIES:");
    let mut anchors: Vec<Hex> = Vec::new();
    for c in view.cities.iter().filter(|c| c.owner == me) {
        anchors.push(c.pos);
        let mut tags = String::new();
        for (on, tag) in [
            (c.is_capital, " capital"),
            (c.walls, " walls"),
            (c.citadel, " citadel"),
            (c.star_fort, " starfort"),
            (c.greece_captive_at_risk, " captive-tech-at-risk"),
        ] {
            if on {
                tags.push_str(tag);
            }
        }
        let garrison: Vec<_> = view
            .units
            .iter()
            .filter(|u| u.pos == c.pos && u.owner == me)
            .map(|u| format!("U{}:{:?}", u.id.0, u.kind))
            .collect();
        let planes: Vec<_> = view
            .aircraft
            .iter()
            .filter(|a| a.city == c.id && a.owner == me)
            .map(|a| format!("U{}:{:?} hp{}", a.id.0, a.kind, a.hp))
            .collect();
        let _ = write!(
            out,
            "  C{} {} ({}) pop {} hp {}/{} str {}{}",
            c.id.0,
            c.name,
            hx(c.pos),
            c.population,
            c.hp,
            c.max_hp,
            c.strength,
            tags
        );
        if !garrison.is_empty() {
            let _ = write!(out, " | in city: {}", garrison.join(" "));
        }
        if !planes.is_empty() {
            let _ = write!(out, " | hangar: {}", planes.join(" "));
        }
        let _ = writeln!(out);
        let Some(d) = &c.detail else { continue };
        let y = &d.yields.total;
        let producing = match &d.producing {
            Some(item) => format!(
                "{} {}/{} ({}t)",
                item_name(item),
                d.production_stored,
                d.production_cost.unwrap_or(0),
                d.turns_to_complete.unwrap_or(-1)
            ),
            None => "IDLE".into(),
        };
        let queue: Vec<_> = d.production_queue.iter().map(item_name).collect();
        let _ = writeln!(
            out,
            "     making {}{} | food {} prod {} gold {} sci {} | grows in {:?}{}{}",
            producing,
            if queue.is_empty() {
                String::new()
            } else {
                format!(" then {}", queue.join(", "))
            },
            y.food,
            y.production,
            y.gold,
            y.science,
            d.turns_to_grow,
            if d.resistance > 0 {
                format!(" | resistance {}", d.resistance)
            } else {
                String::new()
            },
            if d.attacked_last_turn {
                " | ATTACKED last turn"
            } else {
                ""
            },
        );
        if d.producing.is_none() || o.all || o.city == Some(c.id.0) {
            let mut wonders: BTreeMap<String, (i32, i32, Vec<String>)> = BTreeMap::new();
            let mut blocker: Option<&String> = None;
            let mut opts: Vec<String> = Vec::new();
            for p in d.options.iter() {
                if let ProductionItem::Wonder { building, tile } = &p.item
                    && o.city != Some(c.id.0)
                {
                    let w = wonders.entry(format!("{building:?}")).or_insert((
                        p.cost,
                        p.turns,
                        Vec::new(),
                    ));
                    w.2.push(hx(*tile));
                    continue;
                }
                let buy = match (p.buy_cost, &p.buy_blocker) {
                    (Some(g), None) => format!(" buy {g}g"),
                    (_, Some(b)) => {
                        blocker = Some(b);
                        String::new()
                    }
                    _ => String::new(),
                };
                let stats = match &p.item {
                    ProductionItem::Unit { unit } => {
                        let st = legatum_core::data::unit_stats_for(view.faction_of(me), *unit);
                        if st.ranged > 0 {
                            format!(
                                "[{} r{}x{} mv{}]",
                                st.strength, st.ranged, st.range, st.moves
                            )
                        } else {
                            format!("[{} mv{}]", st.strength, st.moves)
                        }
                    }
                    _ => String::new(),
                };
                opts.push(format!(
                    "{}{} {}p/{}t{}",
                    item_name(&p.item),
                    stats,
                    p.cost,
                    p.turns,
                    buy
                ));
            }
            for (name, (cost, turns, tiles)) in wonders {
                opts.push(format!(
                    "{name}@tile {cost}p/{turns}t ({} tiles, e.g. {})",
                    tiles.len(),
                    tiles[0]
                ));
            }
            if let Some(b) = blocker {
                opts.push(format!("(some buys blocked: {b})"));
            }
            let _ = writeln!(out, "     options: {}", opts.join(" | "));
            let _ = writeln!(
                out,
                "     buildings: {}",
                d.buildings
                    .iter()
                    .map(|b| format!("{b:?}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }

    // My units.
    let _ = writeln!(out, "MY UNITS (! = needs orders):");
    let mut mine: Vec<_> = view.units.iter().filter(|u| u.owner == me).collect();
    mine.sort_by_key(|u| u.id);
    let targets: Vec<(Hex, String)> = view
        .units
        .iter()
        .filter(|u| hostile(view, u.owner))
        .map(|u| (u.pos, format!("U{}:{:?}", u.id.0, u.kind)))
        .chain(
            view.cities
                .iter()
                .filter(|c| hostile(view, c.owner))
                .map(|c| (c.pos, format!("C{}:{}", c.id.0, c.name))),
        )
        .collect();
    for u in &mine {
        anchors.push(u.pos);
        let mut tags = String::new();
        if u.ranged > 0 {
            let _ = write!(tags, " ranged {} range {}", u.ranged, u.range);
        }
        if u.level > 0 {
            let _ = write!(tags, " L{}", u.level);
        }
        if u.fortify > 0 {
            let _ = write!(tags, " fort+{}", u.fortify);
        }
        if let Some(s) = u.standing_order {
            let _ = write!(tags, " [{s:?}]");
        }
        if let Some(g) = u.goto {
            let _ = write!(tags, " [goto {}]", hx(g));
        }
        if let (Some(cost), Some(to)) = (u.upgrade_cost, u.upgrade_to) {
            let _ = write!(tags, " [upgrade->{to:?} {cost}g]");
        }
        if u.unsupplied.is_some() {
            tags.push_str(" UNSUPPLIED");
        }
        if u.occupation_lock.is_some() {
            tags.push_str(" LOCKED-IN-CITY");
        }
        let _ = writeln!(
            out,
            " {}U{} {:?} ({}) hp {} mv {}/{} str {}{}",
            if u.needs_orders { "!" } else { " " },
            u.id.0,
            u.kind,
            hx(u.pos),
            u.hp,
            u.moves_left,
            u.max_moves,
            u.strength,
            tags
        );
        if !u.military {
            continue;
        }
        let reach = u.range.max(u.max_moves).max(1);
        let mut near: Vec<_> = targets
            .iter()
            .filter(|(pos, _)| u.pos.distance(*pos) <= reach)
            .filter_map(|(pos, label)| forecast_attack(view, u.id, *pos).map(|f| (pos, label, f)))
            .collect();
        near.sort_by_key(|(pos, _, _)| u.pos.distance(**pos));
        for (pos, label, f) in near.into_iter().take(6) {
            let _ = writeln!(
                out,
                "      {} {} ({}) d{}: deal {} of {}hp, take {} | kill {}% die {}%{}",
                if f.ranged { "shoot" } else { "melee" },
                label,
                hx(*pos),
                u.pos.distance(*pos),
                f.expected_damage_to_defender,
                f.defender_hp,
                f.expected_damage_to_attacker,
                f.win_chance,
                f.loss_chance,
                if f.city_strike.is_some() {
                    " (city fires back)"
                } else {
                    ""
                },
            );
        }
    }
    for a in view.aircraft.iter().filter(|a| a.owner == me) {
        let mut strikes: Vec<_> = targets
            .iter()
            .filter_map(|(pos, label)| {
                forecast_air_strike(view, a.id, *pos).map(|f| (pos, label, f))
            })
            .collect();
        strikes.sort_by_key(|(_, _, f)| -f.expected_damage_to_defender);
        let _ = writeln!(
            out,
            "  U{} {:?} in C{} hp {} L{}{}",
            a.id.0,
            a.kind,
            a.city.0,
            a.hp,
            a.level,
            a.upgrade_cost
                .map(|c| format!(" [upgradeair {c}g]"))
                .unwrap_or_default()
        );
        for (pos, label, f) in strikes.into_iter().take(if o.all { 20 } else { 8 }) {
            let _ = writeln!(
                out,
                "      air {} ({}): deal {} of {}hp | kill {}%",
                label,
                hx(*pos),
                f.expected_damage_to_defender,
                f.defender_hp,
                f.win_chance
            );
        }
    }

    // Others.
    let near = |pos: Hex, radius: i32| anchors.iter().any(|a| a.distance(pos) <= radius);
    let _ = writeln!(
        out,
        "OTHER CITIES (hostile, or near me; --all for every known city):"
    );
    for c in view.cities.iter().filter(|c| c.owner != me) {
        if !(o.all || hostile(view, c.owner) || near(c.pos, 8)) {
            continue;
        }
        let _ = writeln!(
            out,
            "  C{} {} [{} P{}{}] ({}) pop {} hp {}/{} str {}{}{}",
            c.id.0,
            c.name,
            owner_name(view, c.owner),
            c.owner.0,
            if hostile(view, c.owner) { " WAR" } else { "" },
            hx(c.pos),
            c.population,
            c.hp,
            c.max_hp,
            c.strength,
            if c.walls { " walls" } else { "" },
            if c.visible { "" } else { " (last seen)" },
        );
    }
    let _ = writeln!(out, "OTHER UNITS IN SIGHT:");
    let mut others: Vec<_> = view.units.iter().filter(|u| u.owner != me).collect();
    others.sort_by_key(|u| (u.owner, u.id));
    let mut hidden = 0;
    for u in others {
        let war = hostile(view, u.owner);
        if !(o.all || war || near(u.pos, 4)) {
            hidden += 1;
            continue;
        }
        let nearest = view
            .cities
            .iter()
            .filter(|c| c.owner == me)
            .map(|c| (c.pos.distance(u.pos), c.id.0))
            .min();
        let _ = writeln!(
            out,
            "  U{} {} {:?} ({}) hp {} str {}{}{}{}",
            u.id.0,
            owner_name(view, u.owner),
            u.kind,
            hx(u.pos),
            u.hp,
            u.strength,
            if u.ranged > 0 {
                format!(" ranged {} range {}", u.ranged, u.range)
            } else {
                String::new()
            },
            if war { " WAR" } else { "" },
            nearest
                .map(|(d, id)| format!(" | {d} from my C{id}"))
                .unwrap_or_default(),
        );
    }
    if hidden > 0 {
        let _ = writeln!(out, "  ({hidden} neutral units farther away not listed)");
    }
    for s in &view.air_sightings {
        let _ = writeln!(
            out,
            "  air sighting: {}",
            fv(&serde_json::to_value(s).unwrap_or_default())
        );
    }
    for camp in &view.barbarian_camps {
        let _ = writeln!(
            out,
            "  barbarian camp: {}",
            fv(&serde_json::to_value(camp).unwrap_or_default())
        );
    }

    // Map.
    if o.map != "none" && !anchors.is_empty() {
        let (mut c0, mut r0, mut c1, mut r1) = (view.width, view.height, 0, 0);
        if o.map == "all" {
            (c0, r0, c1, r1) = (0, 0, view.width - 1, view.height - 1);
        } else {
            for a in &anchors {
                let (c, r) = a.to_offset();
                c0 = c0.min(c - 4);
                c1 = c1.max(c + 4);
                r0 = r0.min(r - 3);
                r1 = r1.max(r + 3);
            }
            c0 = c0.max(0);
            r0 = r0.max(0);
            c1 = c1.min(view.width - 1);
            r1 = r1.min(view.height - 1);
        }
        let _ = writeln!(
            out,
            "MAP (hex q,r; each row shows r and the q of its first cell; q grows by 1 per cell; \
             odd rows sit half a cell right). Terrain ~ocean -coast g p f(orest) h(ills) d t \
             ^mountain, UPPERCASE = my land, blank = unexplored. Then: # my city, M my unit, \
             m my civilian, @ enemy city, X enemy unit, C other city, u other unit, . empty"
        );
        for row in r0..=r1 {
            let first = Hex::from_offset(c0, row);
            let _ = write!(
                out,
                "r{:>2} q{:>3} |{}",
                row,
                first.q,
                if row & 1 == 1 { " " } else { "" }
            );
            for col in c0..=c1 {
                let h = Hex::from_offset(col, row);
                let tile = view.tile(h);
                let t = match tile.and_then(|t| t.terrain) {
                    Some(t) if tile.is_some_and(|t| t.explored) => {
                        let ch = terrain_char(t);
                        if tile.and_then(|t| t.owner) == Some(me) {
                            ch.to_ascii_uppercase()
                        } else {
                            ch
                        }
                    }
                    _ => ' ',
                };
                let occupant = if let Some(c) = view.cities.iter().find(|c| c.pos == h) {
                    if c.owner == me {
                        '#'
                    } else if hostile(view, c.owner) {
                        '@'
                    } else {
                        'C'
                    }
                } else if let Some(u) = view
                    .units
                    .iter()
                    .filter(|u| u.pos == h)
                    .max_by_key(|u| u.military)
                {
                    if u.owner == me {
                        if u.military { 'M' } else { 'm' }
                    } else if hostile(view, u.owner) {
                        'X'
                    } else {
                        'u'
                    }
                } else if t == ' ' {
                    ' '
                } else {
                    '.'
                };
                out.push(t);
                out.push(occupant);
            }
            let _ = writeln!(out);
        }
    }
    print!("{out}");
}

// ---- orders ----

fn parse_hex(s: &str) -> Result<Value, String> {
    let (q, r) = s
        .split_once(',')
        .ok_or(format!("bad hex {s:?}, want q,r"))?;
    let n = |x: &str| {
        x.trim()
            .parse::<i32>()
            .map_err(|_| format!("bad hex {s:?}"))
    };
    Ok(json!({"q": n(q)?, "r": n(r)?}))
}

fn parse_id(s: &str) -> Result<u32, String> {
    s.trim_start_matches(['U', 'C', 'P', 'T', 'u', 'c', 'p', 't'])
        .parse()
        .map_err(|_| format!("bad id {s:?}"))
}

fn find_item(view: &PlayerView, city: u32, name: &str) -> Result<Value, String> {
    let c = view
        .cities
        .iter()
        .find(|c| c.id.0 == city && c.owner == view.me)
        .ok_or(format!("C{city} is not your city"))?;
    let detail = c.detail.as_ref().ok_or("no city detail")?;
    let wanted = name.trim().to_ascii_lowercase();
    let exact: Vec<_> = detail
        .options
        .iter()
        .filter(|o| item_name(&o.item).to_ascii_lowercase() == wanted)
        .collect();
    let found = if exact.is_empty() {
        detail
            .options
            .iter()
            .filter(|o| item_name(&o.item).to_ascii_lowercase().starts_with(&wanted))
            .collect()
    } else {
        exact
    };
    match found.as_slice() {
        [one] => serde_json::to_value(one.item).map_err(|e| e.to_string()),
        [] => Err(format!(
            "{} cannot make {name:?}; options: {}",
            c.name,
            detail
                .options
                .iter()
                .map(|o| item_name(&o.item))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        many => Err(format!(
            "{name:?} is ambiguous: {}",
            many.iter()
                .map(|o| item_name(&o.item))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn find_tech(view: &PlayerView, name: &str) -> Result<Value, String> {
    view.research
        .available
        .iter()
        .chain(view.research.catalog.iter())
        .find(|t| format!("{:?}", t.tech).eq_ignore_ascii_case(name.trim()))
        .map(|t| serde_json::to_value(t.tech).unwrap_or_default())
        .ok_or(format!("unknown tech {name:?}"))
}

fn parse_line(view: &PlayerView, line: &str) -> Result<Value, String> {
    let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
    let rest = rest.trim();
    let words: Vec<&str> = rest.split_whitespace().collect();
    let word = |i: usize| {
        words
            .get(i)
            .copied()
            .ok_or(format!("{verb}: missing argument"))
    };
    let unit =
        |t: &str| -> Result<Value, String> { Ok(json!({"type": t, "unit": parse_id(word(0)?)?})) };
    let treaty = |kind: &str| -> Result<Value, String> {
        Ok(json!({"type": "ProposeTreaty", "to": parse_id(word(0)?)?, "kind": kind}))
    };
    let city_item = |t: &str| -> Result<Value, String> {
        let city = parse_id(word(0)?)?;
        let name = rest.split_once(' ').map(|(_, n)| n).ok_or("missing item")?;
        Ok(json!({"type": t, "city": city, "item": find_item(view, city, name)?}))
    };
    match verb {
        "explore" => {
            // The app's auto-explore step for a Scout: the same public helper the client calls.
            let id = legatum_core::UnitId(parse_id(word(0)?)?);
            legatum_ai::explore_step(view, id)
                .map(|o| serde_json::to_value(o).unwrap_or_default())
                .ok_or(format!("explore: U{} is not one of your Scouts", id.0))
        }
        "move" => {
            Ok(json!({"type": "MoveUnit", "unit": parse_id(word(0)?)?, "to": parse_hex(word(1)?)?}))
        }
        "shoot" => Ok(
            json!({"type": "RangedAttack", "unit": parse_id(word(0)?)?, "target": parse_hex(word(1)?)?}),
        ),
        "sleep" => {
            Ok(json!({"type": "SleepUnit", "unit": parse_id(word(0)?)?, "until_healed": false}))
        }
        "heal" => {
            Ok(json!({"type": "SleepUnit", "unit": parse_id(word(0)?)?, "until_healed": true}))
        }
        "skip" => unit("SkipUnit"),
        "found" => unit("FoundCity"),
        "disband" => unit("DisbandUnit"),
        "upgrade" => unit("UpgradeUnit"),
        "siege" => unit("UpgradeSiege"),
        "autofire" => Ok(json!({"type": "SetAutoFire", "unit": parse_id(word(0)?)?,
            "enabled": word(1)? == "on", "target": null})),
        "build" => city_item("SetProduction"),
        "buy" => city_item("BuyItem"),
        "queue" => {
            let city = parse_id(word(0)?)?;
            let names = rest
                .split_once(' ')
                .map(|(_, n)| n)
                .ok_or("missing items")?;
            let items: Result<Vec<_>, _> =
                names.split(',').map(|n| find_item(view, city, n)).collect();
            Ok(json!({"type": "SetProductionQueue", "city": city, "queue": items?}))
        }
        "raze" => Ok(json!({"type": "Raze", "city": parse_id(word(0)?)?})),
        "research" => {
            let techs: Result<Vec<_>, _> = words.iter().map(|w| find_tech(view, w)).collect();
            let techs = techs?;
            match techs.as_slice() {
                [] => Err("research: missing tech".into()),
                [one] => Ok(json!({"type": "SetResearch", "tech": one})),
                _ => Ok(json!({"type": "SetResearchQueue", "techs": techs})),
            }
        }
        "war" => Ok(json!({"type": "DeclareWar", "against": parse_id(word(0)?)?, "breach": false})),
        "peace" => treaty("Peace"),
        "ally" => treaty("Alliance"),
        "accept" | "reject" => Ok(json!({"type": "RespondToProposal",
            "proposal": parse_id(word(0)?)?, "accept": verb == "accept", "explanation": null})),
        "air" => Ok(
            json!({"type": "AirStrike", "plane": parse_id(word(0)?)?, "target": parse_hex(word(1)?)?}),
        ),
        "rebase" => {
            Ok(json!({"type": "Rebase", "plane": parse_id(word(0)?)?, "to": parse_id(word(1)?)?}))
        }
        "upgradeair" => Ok(json!({"type": "UpgradeAircraft", "plane": parse_id(word(0)?)?})),
        "raw" => serde_json::from_str(rest).map_err(|e| format!("raw: {e}")),
        _ => Err(format!("unknown order {verb:?}")),
    }
}

/// Parse an orders file. Returns the orders with their source lines, and the problems found.
fn parse_orders(view: &PlayerView, text: &str) -> (Vec<(String, Order)>, Vec<String>) {
    let mut orders = Vec::new();
    let mut problems = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        match parse_line(view, line)
            .and_then(|v| serde_json::from_value::<Order>(v).map_err(|e| e.to_string()))
        {
            Ok(order) => orders.push((line.to_string(), order)),
            Err(e) => problems.push(format!("REJECTED `{line}`: {e}")),
        }
    }
    (orders, problems)
}

fn check(game: &Game, view: &PlayerView, text: &str) -> (Vec<Order>, Vec<String>) {
    let (parsed, mut problems) = parse_orders(view, text);
    let orders: Vec<Order> = parsed.iter().map(|(_, o)| o.clone()).collect();
    for e in validate_orders(&game.session.state, game.seat, &orders) {
        let line = parsed.get(e.index).map(|(l, _)| l.as_str()).unwrap_or("?");
        problems.push(format!("REJECTED `{line}`: {}", e.message));
    }
    (orders, problems)
}

fn preview(view: &PlayerView, orders: &[Order]) {
    for o in orders {
        match o {
            Order::MoveUnit { unit, to } => {
                let Some(u) = view.units.iter().find(|u| u.id == *unit) else {
                    continue;
                };
                let attack = forecast_attack(view, *unit, *to).map(|f| {
                    format!(
                        " ATTACK deal {} take {} kill {}% die {}%",
                        f.expected_damage_to_defender,
                        f.expected_damage_to_attacker,
                        f.win_chance,
                        f.loss_chance
                    )
                });
                match view.movement_path(u.owner, u.kind, u.pos, *to) {
                    Some(path) => println!(
                        "  U{} {:?} {} -> {}: {} steps via {}{}",
                        unit.0,
                        u.kind,
                        hx(u.pos),
                        hx(*to),
                        path.len().saturating_sub(1),
                        path.iter()
                            .take(6)
                            .map(|h| hx(*h))
                            .collect::<Vec<_>>()
                            .join(" "),
                        attack.unwrap_or_default()
                    ),
                    None => println!(
                        "  U{} {:?} {} -> {}: NO PATH{}",
                        unit.0,
                        u.kind,
                        hx(u.pos),
                        hx(*to),
                        attack.unwrap_or_default()
                    ),
                }
            }
            Order::RangedAttack { unit, target } => {
                if let Some(f) = forecast_attack(view, *unit, *target) {
                    println!(
                        "  U{} shoots {}: deal {} of {}hp, kill {}%",
                        unit.0,
                        hx(*target),
                        f.expected_damage_to_defender,
                        f.defender_hp,
                        f.win_chance
                    );
                }
            }
            _ => {}
        }
    }
}

const QUIET: [&str; 12] = [
    "UnitMoved",
    "TileClaimed",
    "TileReleased",
    "TilesReassigned",
    "ResearchQueueAdvanced",
    "RoadsLaid",
    "UnitHealed",
    "CityGrew",
    "AutomationStopped",
    "UnitAwakened",
    "ResourcesRevealed",
    "OrderSuperseded",
];

fn print_events(events: &[Event], before: &PlayerView, after: &PlayerView) {
    let visible = legatum_core::event_visibility::visible_events(events, before, after);
    println!("EVENTS YOU SAW LAST TURN:");
    for e in &visible {
        let v = serde_json::to_value(e).unwrap_or_default();
        let line = describe(&[before, after], &v);
        let name = line.split(' ').next().unwrap_or("");
        if !QUIET.contains(&name) {
            println!("  {line}");
        }
    }
}

fn score_line(game: &Game) -> Value {
    let state = &game.session.state;
    let mut v = json!({
        "turn": state.turn,
        "vp": scoring::vp_total(state, game.seat),
        "cities": state.cities_of(game.seat).count(),
        "alive": state.player(game.seat).alive,
        "finished": state.finished,
        "won": state.finished && state.winner == Some(state.player(game.seat).team),
    });
    if let Some(a) = &game.active {
        let s = a.status(state);
        v["outcome"] = json!(format!("{:?}", s.outcome));
        v["score"] = json!(s.score);
        v["reason"] = json!(s.reason);
    }
    v
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = argv.first().cloned() else {
        die("usage: fable_seat init|view|check|step|hard|autoplay|fork|score --dir D ...")
    };
    let args = Args::parse(&argv[1..]);
    let dir = args.need("dir").to_string();
    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .ok();
    let opts = ViewOpts {
        city: args.num("city"),
        all: args.get("all").is_some(),
        map: args.get("map").unwrap_or("near").to_string(),
    };
    if cmd == "init" {
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| die(format!("{dir}: {e}")));
        let run: u32 = args.num("run").unwrap_or(0);
        let config = if let Some(path) = args.get("scenario") {
            json!({"scenario": path, "run": run})
        } else {
            let seed: u64 = args.num("seed").unwrap_or_else(|| die("missing --seed"));
            let seat: u8 = args.num("seat").unwrap_or(0);
            let faction = args.need("faction");
            let faction = Faction::ALL
                .into_iter()
                .find(|f| format!("{f:?}").eq_ignore_ascii_case(faction))
                .unwrap_or_else(|| die(format!("unknown faction {faction}")));
            let mut setup = GameSetup::all_ai(seed);
            let had = setup.players[seat as usize].faction;
            for p in &mut setup.players {
                p.difficulty = AiDifficulty::Hard;
                if p.faction == faction {
                    p.faction = had;
                }
            }
            setup.players[seat as usize].faction = faction;
            setup.players[seat as usize].name = "Fable".into();
            json!({"setup": legatum_wasm::json::setup_to_value(&setup), "seat": seat, "run": run})
        };
        std::fs::write(format!("{dir}/config.json"), config.to_string())
            .unwrap_or_else(|e| die(format!("config: {e}")));
        std::fs::write(format!("{dir}/tape.jsonl"), "").unwrap_or_else(|e| die(format!("{e}")));
    }
    if cmd == "fork" {
        let to = args.need("to");
        std::fs::create_dir_all(to).unwrap_or_else(|e| die(format!("{to}: {e}")));
        std::fs::copy(format!("{dir}/config.json"), format!("{to}/config.json"))
            .unwrap_or_else(|e| die(format!("fork: {e}")));
        let start = start_game(
            &serde_json::from_str(&read(&format!("{dir}/config.json"))).unwrap_or_default(),
        )
        .session
        .state
        .turn;
        let tape = read(&format!("{dir}/tape.jsonl"));
        let keep = args
            .num::<u32>("turn")
            .map(|t| t.saturating_sub(start) as usize)
            .unwrap_or(usize::MAX);
        let kept: String = tape.lines().take(keep).map(|l| format!("{l}\n")).collect();
        std::fs::write(format!("{to}/tape.jsonl"), kept).unwrap_or_else(|e| die(format!("{e}")));
        println!("forked to {to}");
        return;
    }
    let mut game = load(&dir);
    let seat = game.seat;
    match cmd.as_str() {
        "init" | "view" => {
            if let Some(over) = game.over() {
                println!("{over}");
            }
            let view = player_view(&game.session.state, seat);
            if let Some(id) = args.num::<u32>("unit") {
                // The raw view of one unit in sight (every field the seat's view carries).
                for u in view.units.iter().filter(|u| u.id.0 == id) {
                    println!("{}", fv(&serde_json::to_value(u).unwrap_or_default()));
                }
                return;
            }
            render(&view, game.active.as_ref(), &game.session.state, &opts);
        }
        "score" => println!("{}", score_line(&game)),
        "check" | "step" => {
            if let Some(over) = game.over() {
                die(over);
            }
            let view = player_view(&game.session.state, seat);
            let (orders, problems) = check(&game, &view, &read(args.need("orders")));
            for p in &problems {
                println!("{p}");
            }
            if cmd == "check" {
                println!(
                    "{} orders parsed, {} problems",
                    orders.len(),
                    problems.len()
                );
                preview(&view, &orders);
                return;
            }
            if !problems.is_empty() && args.get("force").is_none() {
                println!("NOT RESOLVED: fix the rejected orders (or pass --force to drop them).");
                std::process::exit(1);
            }
            let turn = game.session.state.turn;
            let before =
                legatum_core::event_visibility::event_visibility_view(&game.session.state, seat);
            let (events, shadow) = game.play(Some(&orders));
            append_tape(
                &dir,
                turn,
                &Entry {
                    by: "me".into(),
                    orders,
                    hard: shadow,
                    hash: game.hash(),
                },
            );
            let after = player_view(&game.session.state, seat);
            print_events(&events, &before, &after);
            if let Some(over) = game.over() {
                println!("{over}");
                println!("{}", score_line(&game));
            }
            render(&after, game.active.as_ref(), &game.session.state, &opts);
        }
        "hard" => {
            let view = player_view(&game.session.state, seat);
            let orders = game.session.ai_orders();
            println!("HARD WOULD ORDER (turn {}):", view.turn);
            for o in orders.get(&seat).into_iter().flatten() {
                let v = serde_json::to_value(o).unwrap_or_default();
                println!("  {}", describe(&[&view], &v));
            }
        }
        "autoplay" => {
            let turns: u32 = args.num("turns").unwrap_or(u32::MAX);
            let until: u32 = args.num("until").unwrap_or(u32::MAX);
            let mut n = 0;
            while n < turns && game.session.state.turn < until && game.over().is_none() {
                let turn = game.session.state.turn;
                let (_, shadow) = game.play(None);
                append_tape(
                    &dir,
                    turn,
                    &Entry {
                        by: "hard".into(),
                        orders: Vec::new(),
                        hard: shadow,
                        hash: game.hash(),
                    },
                );
                n += 1;
            }
            if let Some(over) = game.over() {
                println!("{over}");
            }
            println!("{}", score_line(&game));
        }
        other => die(format!("unknown command {other}")),
    }
}
