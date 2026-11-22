//! State renderer for the LLM-teacher experiment (`docs/design/ai-strategy/fable-decision-models-2026-11-07.md`
//! section 3.2 and 5).
//!
//! For each review window on the designer's tape this renders, as text, what the seat could see:
//! the seat's own `PlayerView` and nothing else (the true state is dropped the moment the view is
//! built, `snap` takes `&PlayerView` only, so a fogged garrison or an unseen city can never reach
//! a prompt). Places are named ("two tiles north-east of Sparta, on hills"), not coordinates. An
//! events digest comes from diffing the view with the view two turns earlier. The candidate
//! operations come from the operation-search log (`operation_search --out`), named exactly as
//! the log names them so an answer joins on the name, each with a one-line account of what the
//! executors do. The designer's condensed notes are a separate field (`--notes`), so the harness
//! decides per run whether they are sent.
//!
//! The designer's own recorded orders for the window (`tape`) are a scoring field only. The
//! harness never puts them in a prompt.
//!
//!   teacher_render --tape TAPE.json --states STATES.jsonl --out OUT.jsonl
//!                  (--windows 18-30[,148-152] [--every 2] | --turns 20,36)
//!                  [--game LABEL] [--seat 0] [--ops LOG.jsonl]... [--notes FILE] [--lookback 2]
//!
//! One JSON line per window: `game`, `turn`, `seat`, `faction`, `state` (the text), `events`,
//! `candidates` (`name`, `template`, `line`, plus the log's gain for scoring), `notes`, `tape`.
//! Integer arithmetic only.
use legatum_core::diplomacy::RelationState;
use legatum_core::{
    CityId, Faction, Hex, Order, PlayerId, PlayerView, TeamId, UnitKind, player_view,
};
use legatum_wasm::tape::Tape;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::{BufRead, Write};

struct Args {
    tape: String,
    states: String,
    out: String,
    game: String,
    seat: PlayerId,
    windows: Vec<(u32, u32)>,
    turns: Vec<u32>,
    every: u32,
    ops: Vec<String>,
    notes: Option<String>,
    lookback: u32,
}

fn parse_args() -> Args {
    let mut a = Args {
        tape: String::new(),
        states: String::new(),
        out: String::new(),
        game: "game".into(),
        seat: PlayerId(0),
        windows: vec![],
        turns: vec![],
        every: 2,
        ops: vec![],
        notes: None,
        lookback: 2,
    };
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        let mut v = || it.next().unwrap_or_else(|| panic!("{k} needs a value"));
        match k.as_str() {
            "--tape" => a.tape = v(),
            "--states" => a.states = v(),
            "--out" => a.out = v(),
            "--game" => a.game = v(),
            "--seat" => a.seat = PlayerId(v().parse().expect("--seat")),
            "--windows" => {
                a.windows = v()
                    .split(',')
                    .map(|w| {
                        let (lo, hi) = w.split_once('-').expect("--windows LO-HI");
                        (lo.parse().expect("lo"), hi.parse().expect("hi"))
                    })
                    .collect()
            }
            "--turns" => a.turns = v().split(',').map(|t| t.parse().expect("turn")).collect(),
            "--every" => a.every = v().parse().expect("--every"),
            "--ops" => a.ops.push(v()),
            "--notes" => a.notes = Some(v()),
            "--lookback" => a.lookback = v().parse().expect("--lookback"),
            other => panic!("unknown argument {other}"),
        }
    }
    assert!(
        !a.tape.is_empty() && !a.states.is_empty() && !a.out.is_empty(),
        "--tape, --states and --out are required"
    );
    assert!(
        !a.windows.is_empty() || !a.turns.is_empty(),
        "--windows or --turns is required"
    );
    a
}

fn review_turns(a: &Args) -> BTreeSet<u32> {
    let mut turns: BTreeSet<u32> = a.turns.iter().copied().collect();
    for &(lo, hi) in &a.windows {
        let mut t = lo;
        while t <= hi {
            turns.insert(t);
            t += a.every.max(1);
        }
        turns.insert(hi);
    }
    turns
}

// ---------------------------------------------------------------------------------------------
// The snapshot: everything below is read from a `PlayerView`.

#[derive(Clone)]
struct CityS {
    id: CityId,
    owner: PlayerId,
    name: String,
    pos: Hex,
    pop: i32,
    hp: i32,
    max_hp: i32,
    walls: bool,
    capital: bool,
    producing: Option<String>,
    turns_left: Option<i32>,
}

#[derive(Clone)]
struct UnitS {
    owner: PlayerId,
    kind: UnitKind,
    pos: Hex,
    hp: i32,
    level: u8,
    terrain: String,
}

#[derive(Clone)]
struct PlayerS {
    id: PlayerId,
    team: TeamId,
    faction: Faction,
    alive: bool,
    vp: u32,
    cities: u32,
    era: u8,
    power: i32,
}

#[derive(Clone)]
struct Snap {
    turn: u32,
    me: PlayerId,
    team: TeamId,
    era_name: String,
    gold: i32,
    gold_per_turn: i32,
    science_per_turn: i32,
    research: Option<String>,
    techs_known: usize,
    cities: Vec<CityS>,
    units: Vec<UnitS>,
    players: Vec<PlayerS>,
    /// (team a, team b, state)
    relations: Vec<(TeamId, TeamId, RelationState)>,
    /// (proposal id, from team, kind) offered to this team.
    offers: Vec<(u32, TeamId, String)>,
    met: Vec<TeamId>,
}

fn snap(view: &PlayerView) -> Snap {
    let cities = view
        .cities
        .iter()
        .map(|c| {
            let d = c.detail.as_ref();
            CityS {
                id: c.id,
                owner: c.owner,
                name: c.name.clone(),
                pos: c.pos,
                pop: c.population,
                hp: c.hp,
                max_hp: c.max_hp,
                walls: c.walls,
                capital: c.is_capital,
                producing: d
                    .and_then(|d| d.producing.as_ref())
                    .map(|p| item_name(&format!("{p:?}"))),
                turns_left: d.and_then(|d| d.turns_to_complete),
            }
        })
        .collect();
    let terrain_at = |h: Hex| -> String {
        let (w, ht) = (view.width, view.height);
        // Odd-r offset storage: row r, column q + (r - (r & 1)) / 2.
        let col = h.q + (h.r - (h.r & 1)) / 2;
        if h.r < 0 || h.r >= ht || col < 0 || col >= w {
            return String::new();
        }
        view.tiles
            .get((h.r * w + col) as usize)
            .and_then(|t| t.terrain)
            .map(|t| format!("{t:?}").to_lowercase())
            .unwrap_or_default()
    };
    let units = view
        .units
        .iter()
        .map(|u| UnitS {
            owner: u.owner,
            kind: u.kind,
            pos: u.pos,
            hp: u.hp,
            level: u.level,
            terrain: terrain_at(u.pos),
        })
        .collect();
    let players = view
        .players
        .iter()
        .map(|p| PlayerS {
            id: p.id,
            team: p.team,
            faction: p.faction,
            alive: p.alive,
            vp: p.vp,
            cities: p.cities,
            era: p.era,
            power: p.power,
        })
        .collect();
    let relations = view
        .diplomacy
        .relations
        .iter()
        .map(|r| (r.a, r.b, r.state))
        .collect();
    let offers = view
        .diplomacy
        .proposals
        .iter()
        .filter(|p| p.to == view.team)
        .map(|p| (p.id.0, p.from, format!("{:?}", p.kind)))
        .collect();
    Snap {
        turn: view.turn,
        me: view.me,
        team: view.team,
        era_name: view.research.era_name.clone(),
        gold: view.economy.gold,
        gold_per_turn: view.economy.gold_per_turn.total,
        science_per_turn: view.economy.science_per_turn.total,
        research: view.research.current.map(|t| format!("{t:?}")),
        techs_known: view.research.known.len(),
        cities,
        units,
        players,
        relations,
        offers,
        met: view.diplomacy.met.clone(),
    }
}

/// `Unit { unit: Hoplite }` to `Hoplite`, `Building { building: Walls }` to `Walls`.
fn item_name(debug: &str) -> String {
    let inner = debug
        .split_once(':')
        .map_or(debug, |(_, rest)| rest)
        .trim_matches(|c: char| c == ' ' || c == '}' || c == '{');
    let name = inner.split([',', ' ']).next().unwrap_or(inner);
    name.to_string()
}

// ---------------------------------------------------------------------------------------------
// Naming places.

fn compass(from: Hex, to: Hex) -> &'static str {
    // Pointy-top axial to a plane, scaled: x = 866 * (2q + r), y = 1500 * r (south is +y).
    let dq = i64::from(to.q - from.q);
    let dr = i64::from(to.r - from.r);
    let x = 866 * (2 * dq + dr);
    let y = 1500 * dr;
    let (ax, ay) = (x.abs(), y.abs());
    // tan 22.5 degrees = 0.4142, so a leg is dominant past 2.414 times the other.
    let horizontal = ax * 1000 > ay * 2414;
    let vertical = ay * 1000 > ax * 2414;
    match (horizontal, vertical, x >= 0, y >= 0) {
        (true, _, true, _) => "east",
        (true, _, false, _) => "west",
        (_, true, _, true) => "south",
        (_, true, _, false) => "north",
        (_, _, true, true) => "south-east",
        (_, _, true, false) => "north-east",
        (_, _, false, true) => "south-west",
        (_, _, false, false) => "north-west",
    }
}

/// "in Sparta", "2 tiles north-east of Sparta", from the nearest city this view knows.
fn place(s: &Snap, pos: Hex) -> String {
    let near = s
        .cities
        .iter()
        .map(|c| (c.pos.distance(pos), c.name.as_str(), c.pos))
        .min();
    match near {
        None => "in unexplored country".to_string(),
        Some((0, name, _)) => format!("in {name}"),
        Some((1, name, p)) => format!("1 tile {} of {name}", compass(p, pos)),
        Some((d, name, p)) => format!("{d} tiles {} of {name}", compass(p, pos)),
    }
}

fn owner_label(s: &Snap, p: PlayerId) -> String {
    if p.is_barbarian() {
        return "Barbarian".into();
    }
    s.players
        .iter()
        .find(|q| q.id == p)
        .map_or(format!("player {}", p.0), |q| {
            if q.id == s.me {
                "you".to_string()
            } else {
                format!("{:?}", q.faction)
            }
        })
}

fn relation_to(s: &Snap, t: TeamId) -> &'static str {
    if t == s.team {
        return "your team";
    }
    let (a, b) = if s.team < t { (s.team, t) } else { (t, s.team) };
    match s.relations.iter().find(|r| r.0 == a && r.1 == b) {
        Some((_, _, RelationState::War)) => "AT WAR with you",
        Some((_, _, RelationState::Alliance)) => "allied with you",
        Some((_, _, RelationState::Peace)) => "at Peace with you",
        None => "no treaty",
    }
}

fn plural(n: usize, one: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {one}s")
    }
}

fn rivals_cities(s: &Snap, owner: PlayerId) -> impl Iterator<Item = &CityS> {
    s.cities.iter().filter(move |c| c.owner == owner)
}

fn nearest_own_city(s: &Snap, pos: Hex) -> Option<(i32, &CityS)> {
    s.cities
        .iter()
        .filter(|c| c.owner == s.me)
        .map(|c| (c.pos.distance(pos), c))
        .min_by_key(|(d, c)| (*d, c.id))
}

// ---------------------------------------------------------------------------------------------
// The state text.

fn render_state(s: &Snap) -> String {
    let mut o = String::new();
    let me = s.players.iter().find(|p| p.id == s.me);
    let faction = me.map_or_else(|| "?".into(), |p| format!("{:?}", p.faction));
    let _ = writeln!(
        o,
        "You are {faction} (the seat to advise), turn {}, {}. Victory points: you {}.",
        s.turn,
        s.era_name,
        me.map_or(0, |p| p.vp)
    );
    let _ = writeln!(
        o,
        "Treasury {} Gold ({:+} a turn), science {} a turn. Researching {} ({} techs known).",
        s.gold,
        s.gold_per_turn,
        s.science_per_turn,
        s.research.as_deref().unwrap_or("nothing"),
        s.techs_known
    );
    let _ = writeln!(o, "\nYour cities:");
    for c in s.cities.iter().filter(|c| c.owner == s.me) {
        let _ = writeln!(
            o,
            "- {}{} (pop {}, {}, hp {}/{}), builds {}{}.",
            c.name,
            if c.capital { ", capital" } else { "" },
            c.pop,
            if c.walls { "Walls" } else { "no Walls" },
            c.hp,
            c.max_hp,
            c.producing.as_deref().unwrap_or("nothing"),
            c.turns_left
                .map_or(String::new(), |t| format!(" ({t} turns)")),
        );
    }
    let _ = writeln!(o, "\nYour units:");
    let mut groups: BTreeMap<(String, String), Vec<&UnitS>> = BTreeMap::new();
    for u in s.units.iter().filter(|u| u.owner == s.me) {
        let mut at = place(s, u.pos);
        if !u.terrain.is_empty() && !at.starts_with("in ") {
            let _ = write!(at, ", on {}", u.terrain);
        }
        groups
            .entry((format!("{:?}", u.kind), at))
            .or_default()
            .push(u);
    }
    for ((kind, at), us) in &groups {
        let hp: Vec<String> = us.iter().map(|u| u.hp.to_string()).collect();
        let vets = us.iter().filter(|u| u.level > 1).count();
        let _ = writeln!(
            o,
            "- {}x {kind} {at} (hp {}{})",
            us.len(),
            hp.join("/"),
            if vets > 0 {
                format!(", {vets} promoted")
            } else {
                String::new()
            }
        );
    }
    let _ = writeln!(o, "\nRivals (public information):");
    for p in s.players.iter().filter(|p| p.team != s.team) {
        let met = s.met.contains(&p.team);
        if !met {
            let _ = writeln!(o, "- {:?}: not met.", p.faction);
            continue;
        }
        let _ = writeln!(
            o,
            "- {:?}: {}, {}, {} city(ies), {} VP, era {}, strength {}.",
            p.faction,
            if p.alive { "alive" } else { "eliminated" },
            relation_to(s, p.team),
            p.cities,
            p.vp,
            p.era,
            p.power
        );
        for c in rivals_cities(s, p.id) {
            let d = nearest_own_city(s, c.pos).map_or(String::new(), |(d, n)| {
                format!(", {d} tiles from your {}", n.name)
            });
            let _ = writeln!(
                o,
                "    city {} (pop {}, {}, hp {}/{}{}{})",
                c.name,
                c.pop,
                if c.walls { "Walls" } else { "no Walls" },
                c.hp,
                c.max_hp,
                if c.capital { ", capital" } else { "" },
                d
            );
        }
    }
    let _ = writeln!(o, "\nVisible foreign units:");
    let mut seen = 0;
    let mut foreign: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    for u in s
        .units
        .iter()
        .filter(|u| u.owner != s.me && !s.same_team(u.owner))
    {
        let near = nearest_own_city(s, u.pos).map_or(String::new(), |(d, c)| {
            format!(" ({d} tiles from your {})", c.name)
        });
        *foreign
            .entry((
                owner_label(s, u.owner),
                format!("{:?}", u.kind),
                format!("{}{near}", place(s, u.pos)),
            ))
            .or_default() += 1;
        seen += 1;
    }
    if seen == 0 {
        let _ = writeln!(o, "- none in sight.");
    }
    for ((who, kind, at), n) in &foreign {
        let _ = writeln!(o, "- {who}: {n}x {kind}, {at}");
    }
    let _ = writeln!(o, "\nDiplomacy:");
    let wars: Vec<String> = s
        .players
        .iter()
        .filter(|p| p.team != s.team && relation_to(s, p.team) == "AT WAR with you")
        .map(|p| format!("{:?}", p.faction))
        .collect();
    let _ = writeln!(
        o,
        "- At war with: {}.",
        if wars.is_empty() {
            "nobody".to_string()
        } else {
            wars.join(", ")
        }
    );
    for (id, from, kind) in &s.offers {
        let who = s
            .players
            .iter()
            .find(|p| p.team == *from)
            .map_or("?".to_string(), |p| format!("{:?}", p.faction));
        let _ = writeln!(
            o,
            "- Offer waiting for your answer: {who} proposes {kind} (proposal {id})."
        );
    }
    o
}

impl Snap {
    fn same_team(&self, p: PlayerId) -> bool {
        self.players
            .iter()
            .any(|q| q.id == p && q.team == self.team)
    }
}

// ---------------------------------------------------------------------------------------------
// The events digest: what changed in the seat's view since `before`. The view has no event log,
// so this diffs two views, the same information a player would notice.

fn render_events(before: &Snap, now: &Snap) -> Vec<String> {
    let mut ev = Vec::new();
    let span = now.turn.saturating_sub(before.turn);
    let who = |s: &Snap, p: PlayerId| owner_label(s, p);
    let old_cities: BTreeMap<CityId, &CityS> = before.cities.iter().map(|c| (c.id, c)).collect();
    for c in &now.cities {
        match old_cities.get(&c.id) {
            None => ev.push(format!(
                "{} city {} newly appears in your view ({} from your nearest city).",
                who(now, c.owner),
                c.name,
                nearest_own_city(now, c.pos)
                    .map_or("far".to_string(), |(d, _)| format!("{d} tiles"))
            )),
            Some(o) => {
                if o.owner != c.owner {
                    ev.push(format!(
                        "{} changed hands: {} to {}.",
                        c.name,
                        who(before, o.owner),
                        who(now, c.owner)
                    ));
                }
                if c.hp + 3 <= o.hp {
                    ev.push(format!(
                        "{} lost {} hp ({} to {}): it was attacked.",
                        c.name,
                        o.hp - c.hp,
                        o.hp,
                        c.hp
                    ));
                }
                if c.walls && !o.walls {
                    ev.push(format!("{} finished Walls.", c.name));
                }
                if c.pop > o.pop {
                    ev.push(format!("{} grew to pop {}.", c.name, c.pop));
                }
                if c.owner == now.me
                    && o.producing != c.producing
                    && o.turns_left.is_some_and(|t| t <= span as i32)
                {
                    ev.push(format!(
                        "{} finished {} and now builds {}.",
                        c.name,
                        o.producing.as_deref().unwrap_or("an item"),
                        c.producing.as_deref().unwrap_or("nothing")
                    ));
                }
            }
        }
    }
    let new_ids: BTreeSet<CityId> = now.cities.iter().map(|c| c.id).collect();
    for o in old_cities.values().filter(|o| !new_ids.contains(&o.id)) {
        ev.push(format!("{} dropped out of your view.", o.name));
    }
    let mine = |s: &Snap| {
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for u in s.units.iter().filter(|u| u.owner == s.me) {
            *m.entry(format!("{:?}", u.kind)).or_default() += 1;
        }
        m
    };
    let (mb, mn) = (mine(before), mine(now));
    for k in mb.keys().chain(mn.keys()).collect::<BTreeSet<_>>() {
        let (b, n) = (
            mb.get(k).copied().unwrap_or(0),
            mn.get(k).copied().unwrap_or(0),
        );
        if n > b {
            ev.push(format!(
                "You gained {} ({k}); now {n}.",
                plural(n - b, "unit")
            ));
        } else if n < b {
            ev.push(format!(
                "You have {} fewer {k} ({b} to {n}): lost, upgraded or disbanded.",
                b - n
            ));
        }
    }
    let foreign = |s: &Snap| -> BTreeSet<String> {
        s.units
            .iter()
            .filter(|u| u.owner != s.me && !s.same_team(u.owner))
            .map(|u| owner_label(s, u.owner))
            .collect()
    };
    for f in foreign(now).difference(&foreign(before)) {
        ev.push(format!("{f} units are newly in sight."));
    }
    for r in &now.relations {
        let old = before.relations.iter().find(|o| o.0 == r.0 && o.1 == r.1);
        if old.map(|o| o.2) != Some(r.2) && (r.0 == now.team || r.1 == now.team) {
            let other = if r.0 == now.team { r.1 } else { r.0 };
            let name = now
                .players
                .iter()
                .find(|p| p.team == other)
                .map_or("?".to_string(), |p| format!("{:?}", p.faction));
            ev.push(format!(
                "Relation with {name} is now: {}.",
                relation_to(now, other)
            ));
        }
    }
    for p in &now.players {
        if let Some(o) = before.players.iter().find(|o| o.id == p.id) {
            if o.alive && !p.alive {
                ev.push(format!("{:?} was eliminated.", p.faction));
            }
            if p.team != now.team && p.cities != o.cities {
                ev.push(format!(
                    "{:?} now has {} cities (was {}).",
                    p.faction, p.cities, o.cities
                ));
            }
            if p.vp != o.vp {
                ev.push(format!("{:?} VP {} to {}.", p.faction, o.vp, p.vp));
            }
        }
    }
    if now.techs_known > before.techs_known {
        ev.push(format!(
            "You learned {} tech(s).",
            now.techs_known - before.techs_known
        ));
    }
    if (now.gold - before.gold).abs() >= 30 {
        ev.push(format!(
            "Treasury moved {} to {} Gold.",
            before.gold, now.gold
        ));
    }
    for (id, from, kind) in &now.offers {
        if !before.offers.iter().any(|o| o.0 == *id) {
            let who = now
                .players
                .iter()
                .find(|p| p.team == *from)
                .map_or("?".to_string(), |p| format!("{:?}", p.faction));
            ev.push(format!("New offer: {who} proposes {kind}."));
        }
    }
    if ev.is_empty() {
        ev.push("Nothing notable changed in your view.".into());
    }
    ev
}

// ---------------------------------------------------------------------------------------------
// Candidates: the operation-search log's operations, with what the executors do.

fn label(name: &str, plan: &Option<String>) -> String {
    match (name.strip_prefix("plan:"), plan) {
        (Some(_), Some(p)) => format!("plan:{}", p.replace(' ', "_")),
        _ => name.to_string(),
    }
}

fn template_line(template: &str, name: &str, hard_plan: &str, plan: Option<&str>) -> String {
    let target = name.split_once(':').map_or("", |(_, t)| t);
    match template {
        "hard" => format!("Keep Hard's own plan unchanged: {hard_plan}."),
        "hard_candidate" => format!(
            "Switch to Hard's runner-up plan ({}); the executors follow it.",
            plan.unwrap_or(target)
        ),
        "strike" => format!("March every unit in reach on {target}, ignore the odds and Walls, storm it when assembled, refuse Peace while it stands. Held 30 turns."),
        "buy_strike" => format!("As the strike on {target}, and also spend the Gold hoard on the war (buy units and Walls freely)."),
        "short_strike" => format!("A 10-turn strike on {target}, then back to Hard's plan."),
        "veteran_strike" => format!("Strike {target} with a small veteran army kept alive and drilled, with the faction's own playbook."),
        "expand_now" => "Settlers first: found cities at once on the nearest decent site, run a Settler pipeline.".into(),
        "escort" => "Escort every Settler with the nearest unit (buy one if needed).".into(),
        "snipe" => "Stalk exposed rival Settlers and declare war to capture them.".into(),
        "lookouts" => "Post lookouts on the passes toward every bordering rival, so an attack is seen in time.".into(),
        "wall_up" => "Walls first in cities near a rival, Walls as the garrison, never swapped out.".into(),
        "small_army_economy" => "Keep a small veteran army; put Production and Gold on the economy.".into(),
        "refuse_peace" => "Refuse Peace offers while at war; keep the war going.".into(),
        "accept_peace" => format!("Accept the offered Peace with team {target}."),
        "faction_kit" => "Play the faction's own tested playbook (its signature units and rules) at full weight.".into(),
        "fortress_economy" => "Small army, Walls everywhere near a rival, lookouts on the passes; Production on the economy.".into(),
        "expand_walled" => "Expand with Settlers, Walls in each new city and every Settler escorted.".into(),
        other => format!("Operation template {other}."),
    }
}

fn plan_text(plan: &str, s: &Snap) -> String {
    // "Campaign { against: TeamId(1), target: CityId(2) }" -> campaign against a team's city.
    let team = plan
        .split("TeamId(")
        .nth(1)
        .and_then(|r| r.split(')').next())
        .and_then(|n| n.parse::<u8>().ok());
    let city = plan
        .split("CityId(")
        .nth(1)
        .and_then(|r| r.split(')').next())
        .and_then(|n| n.parse::<u32>().ok());
    if plan.starts_with("Campaign") {
        let tname = team
            .and_then(|t| s.players.iter().find(|p| p.team == TeamId(t)))
            .map_or("a rival".to_string(), |p| format!("{:?}", p.faction));
        let cname = city
            .and_then(|c| s.cities.iter().find(|x| x.id == CityId(c)))
            .map_or("a city".to_string(), |c| c.name.clone());
        return format!("a Campaign against {tname}, target {cname}");
    }
    format!("the {plan} plan")
}

fn render_candidates(log: &Value, s: &Snap) -> (Vec<Value>, String) {
    let hard_plan = log["hard_plan"]
        .as_str()
        .map_or("none".to_string(), |p| plan_text(p, s));
    let mut out = Vec::new();
    for op in log["operations"].as_array().into_iter().flatten() {
        let name = op["name"].as_str().unwrap_or("?");
        let template = op["template"].as_str().unwrap_or("?");
        let plan = op["plan"].as_str().map(|p| plan_text(p, s));
        out.push(json!({
            "name": name,
            "label": label(name, &plan),
            "template": template,
            "line": template_line(template, name, &hard_plan, plan.as_deref()),
            "gain_pp": op["gain_pp"], "gain_lo": op["gain_lo"], "gain_hi": op["gain_hi"],
            "wins": op["wins"], "samples": op["samples"],
        }));
    }
    (out, hard_plan)
}

// ---------------------------------------------------------------------------------------------
// The designer's recorded orders over the window: a scoring field, never shown to a model.

fn tape_digest(tape: &Tape, seat: PlayerId, from: u32, to: u32, at: &Snap) -> Value {
    let mut declared = Vec::new();
    let mut founded = 0;
    let mut bought = Vec::new();
    let mut produced = Vec::new();
    let mut accepted = Vec::new();
    let mut refused = Vec::new();
    let mut proposed = Vec::new();
    let mut marched: BTreeMap<String, usize> = BTreeMap::new();
    let mut moves = 0;
    let fname = |t: TeamId| {
        at.players
            .iter()
            .find(|p| p.team == t)
            .map_or(format!("team {}", t.0), |p| format!("{:?}", p.faction))
    };
    for t in tape.turns.iter().filter(|t| t.turn >= from && t.turn < to) {
        for o in t.orders.get(&seat).into_iter().flatten() {
            match o {
                Order::DeclareWar { against, .. } => declared.push(fname(*against)),
                Order::FoundCity { .. } => founded += 1,
                Order::BuyItem { item, .. } => bought.push(item_name(&format!("{item:?}"))),
                Order::SetProduction { item, .. } => produced.push(item_name(&format!("{item:?}"))),
                Order::ProposeTreaty { to, kind } => {
                    proposed.push(format!("{kind:?} to {}", fname(*to)))
                }
                Order::RespondToProposal {
                    proposal, accept, ..
                } => {
                    let from = at
                        .offers
                        .iter()
                        .find(|o| o.0 == proposal.0)
                        .map(|o| format!("{} {}", fname(o.1), o.2));
                    let label = from.unwrap_or(format!("proposal {}", proposal.0));
                    if *accept {
                        accepted.push(label)
                    } else {
                        refused.push(label)
                    }
                }
                Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => {
                    moves += 1;
                    let city = at
                        .cities
                        .iter()
                        .filter(|c| c.owner != at.me && !at.same_team(c.owner))
                        .map(|c| (c.pos.distance(*to), &c.name))
                        .min();
                    if let Some((d, name)) = city
                        && d <= 3
                    {
                        *marched.entry(name.clone()).or_default() += 1;
                    }
                }
                _ => {}
            }
        }
    }
    json!({"from": from, "to": to, "declared": declared, "founded": founded, "bought": bought,
        "produced": produced, "proposed": proposed, "accepted": accepted, "refused": refused,
        "marched_on": marched, "moves": moves})
}

fn load_ops(paths: &[String], game: &str) -> BTreeMap<u32, Value> {
    let mut out = BTreeMap::new();
    for p in paths {
        let f = std::io::BufReader::new(
            std::fs::File::open(p).unwrap_or_else(|_| panic!("ops log {p}")),
        );
        for line in f
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.trim().is_empty())
        {
            let v: Value = serde_json::from_str(&line).expect("ops log line");
            if v["game"].as_str().is_none_or(|g| g == game)
                && let Some(t) = v["turn"].as_u64()
            {
                out.insert(t as u32, v);
            }
        }
    }
    out
}

fn condense_notes(path: &str) -> String {
    let text = std::fs::read_to_string(path).expect("notes file");
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() <= 48,
        "notes must be about 40 lines, got {}",
        lines.len()
    );
    lines.join("\n")
}

fn main() {
    let a = parse_args();
    let tape =
        Tape::from_json(&std::fs::read_to_string(&a.tape).expect("read tape")).expect("tape");
    let turns = review_turns(&a);
    let mut want: BTreeSet<u32> = turns.clone();
    for t in &turns {
        want.insert(t.saturating_sub(a.lookback).max(1));
    }
    let last = *want.iter().max().expect("turns");
    let mut snaps: BTreeMap<u32, Snap> = BTreeMap::new();
    let reader = std::io::BufReader::new(std::fs::File::open(&a.states).expect("states file"));
    for line in reader.lines() {
        let line = line.expect("states line");
        let state: legatum_core::GameState = serde_json::from_str(&line).expect("recorded state");
        if want.contains(&state.turn) {
            // The true state ends here: from this line on only the seat's view is used.
            let view = player_view(&state, a.seat);
            drop(state);
            snaps.insert(view.turn, snap(&view));
        }
        if snaps.len() == want.len() || snaps.keys().max().is_some_and(|t| *t >= last) {
            break;
        }
    }
    let ops = load_ops(&a.ops, &a.game);
    let notes = a.notes.as_deref().map(condense_notes);
    let mut out = std::fs::File::create(&a.out).expect("out file");
    for t in &turns {
        let now = snaps
            .get(t)
            .unwrap_or_else(|| panic!("turn {t} is not in the states file"));
        let before = snaps
            .get(&t.saturating_sub(a.lookback).max(1))
            .expect("lookback state");
        let events = render_events(before, now);
        let (candidates, hard_plan) = ops.get(t).map_or((vec![], "unknown".to_string()), |log| {
            render_candidates(log, now)
        });
        let faction = now
            .players
            .iter()
            .find(|p| p.id == now.me)
            .map_or("?".into(), |p| format!("{:?}", p.faction));
        let row = json!({
            "game": a.game, "turn": t, "seat": a.seat.0, "faction": faction,
            "state": render_state(now), "events": events,
            "hard_plan": hard_plan, "candidates": candidates,
            "notes": notes,
            "tape": tape_digest(&tape, a.seat, *t, t + a.every.max(1), now),
        });
        writeln!(out, "{row}").expect("write");
    }
    eprintln!(
        "teacher_render {}: {} windows rendered",
        a.game,
        turns.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_points_follow_the_screen() {
        let o = Hex::new(0, 0);
        assert_eq!(compass(o, Hex::new(3, 0)), "east");
        assert_eq!(compass(o, Hex::new(-3, 0)), "west");
        assert_eq!(compass(o, Hex::new(3, -3)), "north-east");
        assert_eq!(compass(o, Hex::new(0, -3)), "north-west");
        assert_eq!(compass(o, Hex::new(-3, 3)), "south-west");
        assert_eq!(compass(o, Hex::new(2, -4)), "north");
        assert_eq!(compass(o, Hex::new(-2, 4)), "south");
    }

    #[test]
    fn item_names_are_short() {
        assert_eq!(item_name("Unit { unit: Hoplite }"), "Hoplite");
        assert_eq!(item_name("Building { building: Walls }"), "Walls");
    }

    #[test]
    fn template_lines_name_the_target() {
        let l = template_line("strike", "strike:Karakorum", "x", None);
        assert!(l.contains("Karakorum"));
    }
}
