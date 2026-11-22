//! Paired Standard coalition evaluation. Same maps and eight faction rotations per map.
//! coalition_bench <games> <first-map-seed> <output.json> [solo|pair|pairs]
use legatum_core::{Event, Faction, GameSetup, GameState, TeamId, scoring};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize, Default)]
struct Episode {
    target: u8,
    warning: u32,
    initial_progress: u32,
    last_warning: u32,
    formed: Option<u32>,
    progress_at_formation: Option<u32>,
    first_allied_war: Option<u32>,
    members: Vec<u8>,
    leader_changed: bool,
    threat_ended: Option<u32>,
    progress_after_8: Option<u32>,
    progress_after_16: Option<u32>,
    initial_alliances: Vec<u32>,
    dissolved: Vec<u32>,
    dissolved_turns: BTreeMap<u32, u32>,
    target_won: bool,
}
#[derive(Serialize)]
struct Record {
    seed: u64,
    rotation: usize,
    layout: String,
    turns: u32,
    winner: Option<u8>,
    winners: Vec<String>,
    stalled: bool,
    /// Teams that reached their VP target and started a hold (reaching is not winning: a team
    /// must hold the target for `data::VP_HOLD_TURNS` turn ends).
    holds_started: u32,
    holds_broken: u32,
    /// The winner completed a VP hold (false for conquest and turn-limit wins).
    winner_by_hold: bool,
    max_quiet_vp_turns: u32,
    winner_max_lead_streak: u32,
    winner_final_lead_streak: u32,
    alliances: u32,
    betrayals: u32,
    alliance_breaches: u32,
    wars_joined: u32,
    declarations: u32,
    captures: u32,
    failed_orders: BTreeMap<String, u32>,
    episodes: Vec<Episode>,
}
fn ranks(s: &GameState) -> Vec<(u32, TeamId)> {
    let mut r: Vec<_> = s
        .teams()
        .into_iter()
        .filter(|t| s.team_alive(*t))
        .map(|t| {
            (
                100 * scoring::team_vp(s, t) / scoring::team_target(s, t).max(1),
                t,
            )
        })
        .collect();
    r.sort_by_key(|(p, t)| (std::cmp::Reverse(*p), *t));
    r
}
fn threat(r: &[(u32, TeamId)]) -> Option<TeamId> {
    let &(p, t) = r.first()?;
    let next = r.get(1)?.0;
    (p > next && (p >= 80 || p >= 60 && p - next >= 15)).then_some(t)
}
// Kept generic so the identical harness compiles against the unmodified slice-2 base.
fn alliance(r: &legatum_core::diplomacy::Relation) -> bool {
    format!("{:?}", r.state) == "Alliance"
}
fn alliance_target(r: &legatum_core::diplomacy::Relation) -> Option<u8> {
    if !alliance(r) {
        return None;
    }
    serde_json::to_value(&r.security)
        .ok()?
        .get("coalition_target")?
        .as_u64()
        .map(|x| x as u8)
}
fn joined_counts(events: &[Event]) -> u32 {
    events
        .iter()
        .filter(|e| matches!(e, Event::WarJoined { .. }))
        .count() as u32
}
fn play(index: usize, start: u64, layout: &str) -> Record {
    let seed = start + (index / 8) as u64;
    let rotation = index % 8;
    let mut setup = GameSetup::all_ai(seed);
    setup.turn_limit = None;
    for (seat, p) in setup.players.iter_mut().enumerate() {
        p.faction = Faction::ALL[(seat + rotation) % 8];
        p.team = TeamId(match layout {
            "pair" if seat < 2 => 0,
            "pairs" => (seat / 2) as u8,
            _ => seat as u8,
        });
    }
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut record = Record {
        seed,
        rotation,
        layout: layout.into(),
        turns: 0,
        winner: None,
        winners: vec![],
        stalled: false,
        holds_started: 0,
        holds_broken: 0,
        winner_by_hold: false,
        max_quiet_vp_turns: 0,
        winner_max_lead_streak: 0,
        winner_final_lead_streak: 0,
        alliances: 0,
        betrayals: 0,
        alliance_breaches: 0,
        wars_joined: 0,
        declarations: 0,
        captures: 0,
        failed_orders: BTreeMap::new(),
        episodes: vec![],
    };
    let mut lead = None;
    let mut streak = 0;
    let mut max_streak = BTreeMap::<TeamId, u32>::new();
    let mut current = None;
    let mut quiet = 0;
    let mut previous = vec![];
    while !session.state.finished && record.turns < 1200 {
        let state = &session.state;
        let turn = state.turn;
        let rank = ranks(state);
        let leader = rank.first().map(|r| r.1);
        if leader == lead {
            streak += 1;
        } else {
            lead = leader;
            streak = 1;
        }
        if let Some(t) = lead {
            max_streak
                .entry(t)
                .and_modify(|x| *x = (*x).max(streak))
                .or_insert(streak);
        }
        let vp: Vec<_> = state
            .teams()
            .into_iter()
            .map(|t| (t, scoring::team_vp(state, t)))
            .collect();
        if vp == previous {
            quiet += 1;
        } else {
            quiet = 0;
        }
        record.max_quiet_vp_turns = record.max_quiet_vp_turns.max(quiet);
        previous = vp;
        let target = threat(&rank);
        if target != current {
            if current.is_some()
                && let Some(e) = record.episodes.last_mut()
            {
                e.threat_ended = Some(turn);
            }
            if let Some(t) = target {
                record.episodes.push(Episode {
                    target: t.0,
                    warning: turn,
                    initial_progress: rank[0].0,
                    last_warning: turn,
                    ..Default::default()
                });
            }
            current = target;
        }
        for e in &mut record.episodes {
            let t = TeamId(e.target);
            let p = 100 * scoring::team_vp(state, t) / scoring::team_target(state, t).max(1);
            if e.formed.is_some() && leader != Some(t) {
                e.leader_changed = true;
            }
            if let Some(f) = e.formed {
                if turn == f + 8 {
                    e.progress_after_8 = Some(p);
                }
                if turn == f + 16 {
                    e.progress_after_16 = Some(p);
                }
            }
            let d = state.diplomacy.as_ref().unwrap();
            for id in &e.initial_alliances {
                if !e.dissolved.contains(id)
                    && !d
                        .relations
                        .iter()
                        .any(|r| alliance(r) && r.security.as_ref().is_some_and(|s| s.id == *id))
                {
                    e.dissolved.push(*id);
                    e.dissolved_turns.insert(*id, turn);
                }
            }
        }
        if let Some(t) = target {
            let e = record.episodes.last_mut().unwrap();
            e.last_warning = turn;
            let d = state.diplomacy.as_ref().unwrap();
            let mut members = BTreeSet::new();
            for r in &d.relations {
                if alliance_target(r) == Some(t.0) && r.a != t && r.b != t {
                    members.extend([r.a.0, r.b.0]);
                    let id = r.security.as_ref().unwrap().id;
                    if !e.initial_alliances.contains(&id) {
                        e.initial_alliances.push(id);
                    }
                }
            }
            if members.len() >= 2 {
                if e.formed.is_none() {
                    e.formed = Some(turn);
                    e.progress_at_formation = Some(rank[0].0);
                }
                e.members = members.iter().copied().collect();
                if members
                    .iter()
                    .filter(|m| state.teams_at_war(TeamId(**m), t))
                    .count()
                    >= 2
                {
                    e.first_allied_war.get_or_insert(turn);
                }
            }
        }
        let orders = session.ai_orders();
        let events = session.step(&orders);
        record.turns += 1;
        record.wars_joined += joined_counts(&events);
        for event in events {
            match event {
                Event::TreatyActivated { relation, .. } if alliance(&relation) => {
                    record.alliances += 1
                }
                Event::VictoryHoldStarted { .. } => record.holds_started += 1,
                Event::VictoryHoldBroken { .. } => record.holds_broken += 1,
                Event::AllianceBetrayed { .. } => record.betrayals += 1,
                Event::TreatyBreached { .. } => record.alliance_breaches += 1,
                Event::WarDeclared { .. } => record.declarations += 1,
                Event::CityCaptured { .. } => record.captures += 1,
                Event::OrderFailed { reason, .. } => {
                    *record.failed_orders.entry(reason).or_default() += 1
                }
                _ => {}
            }
        }
    }
    record.stalled = !session.state.finished;
    if let Some(t) = session.state.winner {
        record.winner = Some(t.0);
        record.winner_by_hold = session
            .state
            .vp_holds
            .get(&t)
            .is_some_and(|held| *held >= legatum_core::data::VP_HOLD_TURNS);
        record.winners = session
            .state
            .players
            .iter()
            .filter(|p| p.team == t)
            .map(|p| format!("{:?}", p.faction))
            .collect();
        record.winner_max_lead_streak = max_streak.get(&t).copied().unwrap_or(0);
        record.winner_final_lead_streak = if lead == Some(t) { streak } else { 0 };
        for e in &mut record.episodes {
            e.target_won = e.target == t.0;
        }
    }
    record
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let games: usize = args[1].parse().unwrap();
    let start: u64 = args[2].parse().unwrap();
    let layout = args.get(4).map_or("solo", String::as_str);
    assert!(matches!(layout, "solo" | "pair" | "pairs"));
    let done = std::sync::atomic::AtomicUsize::new(0);
    let clock = std::time::Instant::now();
    let records: Vec<_> = (0..games)
        .into_par_iter()
        .map(|i| {
            let r = play(i, start, layout);
            let n = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            if n.is_multiple_of(25) || n == games {
                eprintln!("{n}/{games} games, {:.1}s", clock.elapsed().as_secs_f64());
            }
            r
        })
        .collect();
    std::fs::write(&args[3],serde_json::to_vec(&serde_json::json!({"games":games,"map_seed":start,"rotations":8,"safety_cap":1200,"runaway_thresholds":[20,50,100],"seconds":clock.elapsed().as_secs_f64(),"records":records})).unwrap()).unwrap();
}
