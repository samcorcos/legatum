//! Does the AI contest the VP leader? (pillar 11; playtest 8.4 Celts/India follow-up)
//!   contest_leader <games> [seed=9600001] [threads=8] [search=none|all|one] [cap=1200]
//! Eight-player Standard games, solo teams, every seat Hard (factions shuffled per seed as the
//! simulator does). `search=all` makes every seat Hard + Search with the in-app budget, `one`
//! one seat per game (game index mod 8). One JSON line per game, then a summary:
//! - **Campaign on the leader:** a trigger is a team at >= 7 VP at the end of turn 80, or a
//!   team starting a hold (first `VictoryHoldStarted`). It is answered when, within 15 turns,
//!   another team at war with it has two or more non-Scout military units within 4 hexes of one
//!   of its cities (campaign staging stands 3-4 out of city fire), at least one of them more
//!   than 2 hexes from every city of its own (an army on the move, not a border garrison), or
//!   an AI starts a campaign (its own memory) on one of its cities.
//! - **Coalition speed:** each time a second team goes to war with the public VP leader (strict
//!   leader, >= 4 VP), the turns until two teams at war with it are allied.
//! - Alliance refusals that cite the treasury between two teams at war with the same team.
//! - Stalls: games still running at the cap.
//! - Search seats: searches where a Contain option and a Peace/Economy option had identical
//!   per-sample rollout values.
//!
//! Ad hoc evidence; ignores the --out flags the cloud runner appends.
use legatum_core::diplomacy::{RelationState, TreatyKind};
use legatum_core::rng::{Rng, purpose};
use legatum_core::scoring::team_vp;
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, GameState, Order, TeamId};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde_json::json;

const WINDOW: u32 = 15;

/// (leader, trigger turn, kind, answered turn and by whom)
type Trigger = (TeamId, u32, &'static str, Option<(u32, TeamId)>);

fn main() {
    let a: Vec<String> = std::env::args()
        .take_while(|x| !x.starts_with("--"))
        .collect();
    let mut seed: u64 = a.get(2).map_or(9_600_001, |x| x.parse().unwrap());
    let threads: usize = a.get(3).map_or(8, |x| x.parse().unwrap());
    let search = a.get(4).cloned().unwrap_or_else(|| "none".into());
    let cap: u32 = a.get(5).map_or(1200, |x| x.parse().unwrap());
    // The cloud runner appends `--seed S` and `--games N` for each server's range; a suite
    // (`games` and `search` as matching comma lists, e.g. `512,128 ... none,all`) ignores the
    // latter.
    let all: Vec<String> = std::env::args().collect();
    if let Some(i) = all.iter().position(|x| x == "--seed") {
        seed = all[i + 1].parse().unwrap();
    }
    let modes: Vec<&str> = search.split(',').collect();
    let mut counts: Vec<u64> = a[1].split(',').map(|x| x.parse().unwrap()).collect();
    if modes.len() == 1
        && let Some(i) = all.iter().position(|x| x == "--games")
    {
        counts = vec![all[i + 1].parse().unwrap()];
    }
    let dir = all
        .iter()
        .position(|x| x == "--out-dir")
        .map(|i| all[i + 1].clone());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    for (mode, games) in modes.iter().zip(counts) {
        let results: Vec<serde_json::Value> = pool.install(|| {
            (0..games)
                .into_par_iter()
                .map(|g| run(seed + g, g as usize, mode, cap))
                .collect()
        });
        let mut lines = String::new();
        for r in &results {
            lines.push_str(&r.to_string());
            lines.push('\n');
        }
        match &dir {
            Some(d) => std::fs::write(format!("{d}/contest_leader-{mode}.jsonl"), &lines).unwrap(),
            None => print!("{lines}"),
        }
        eprint!("{mode}: ");
        summary(&results);
    }
}

fn summary(results: &[serde_json::Value]) {
    let mut triggers = 0;
    let mut answered = 0;
    let mut gaps: Vec<u32> = Vec::new();
    let mut never = 0;
    let mut dissolved = 0;
    let mut refusals = 0;
    let mut capped = 0;
    let mut searches = 0;
    let mut same = 0;
    let mut paired = 0;
    let mut campaign_starts = 0;
    let mut insufficient_aborts = 0;
    let mut turns = 0u64;
    for r in results {
        for t in r["triggers"].as_array().unwrap() {
            triggers += 1;
            if !t["answered"].is_null() {
                answered += 1;
            }
        }
        for e in r["coalitions"].as_array().unwrap() {
            match e["gap"].as_u64() {
                Some(g) => gaps.push(g as u32),
                None if !e["dissolved"].is_null() => dissolved += 1,
                None => never += 1,
            }
        }
        refusals += r["treasury_refusals"].as_u64().unwrap();
        capped += u32::from(r["capped"].as_bool().unwrap());
        searches += r["searches"].as_u64().unwrap();
        same += r["contain_peace_identical"].as_u64().unwrap();
        paired += r["contain_peace_paired"].as_u64().unwrap();
        campaign_starts += r["campaign_starts"].as_u64().unwrap();
        insufficient_aborts += r["insufficient_army_aborts"].as_u64().unwrap();
        turns += r["turns"].as_u64().unwrap();
    }
    gaps.sort();
    // Never-allied episodes count as longer than any observed gap.
    let n = gaps.len() + never;
    let median = if n == 0 {
        "-".to_string()
    } else if n / 2 < gaps.len() {
        gaps[n / 2].to_string()
    } else {
        "never".to_string()
    };
    eprintln!(
        "SUMMARY games {} | leader triggers {triggers}, answered within {WINDOW} turns {answered} ({}%) \
         | coalition episodes {n}: median gap {median}, allied {} never {never} (dissolved first {dissolved}), gaps {:?} \
         | treasury refusals {refusals} | stalls {capped} | mean length {} \
         | searches {searches}, with contain and peace {paired}, identical {same} \
         | campaign starts {campaign_starts}, insufficient army aborts {insufficient_aborts}",
        results.len(),
        if triggers > 0 {
            answered * 100 / triggers
        } else {
            0
        },
        gaps.len(),
        gaps,
        turns / results.len().max(1) as u64,
    );
}

fn setup(seed: u64, index: usize, search: &str) -> GameSetup {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(8);
    (setup.width, setup.height) = (48, 32);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (seat, (p, f)) in setup.players.iter_mut().zip(factions).enumerate() {
        p.faction = f;
        p.team = TeamId(seat as u8);
        p.difficulty = AiDifficulty::Hard;
        p.search = search == "all" || (search == "one" && seat == index % 8);
    }
    setup.turn_limit = None;
    setup
}

/// An army on the move at a city of `leader`: two non-Scout military units of a team at war
/// with it within 4 hexes of one of its cities, one of them away from its own cities.
fn pressure(state: &GameState, leader: TeamId) -> Option<TeamId> {
    let cities: Vec<_> = state
        .cities
        .values()
        .filter(|c| state.team_of(c.owner) == leader)
        .collect();
    for team in state.teams() {
        if team == leader || !state.teams_at_war(team, leader) || team == TeamId::BARBARIAN {
            continue;
        }
        let own: Vec<_> = state
            .cities
            .values()
            .filter(|c| state.team_of(c.owner) == team)
            .map(|c| c.pos)
            .collect();
        for c in &cities {
            let near: Vec<_> = state
                .units
                .values()
                .filter(|u| {
                    !u.owner.is_barbarian()
                        && state.team_of(u.owner) == team
                        && legatum_core::rules::is_military(u.kind)
                        && u.kind != legatum_core::UnitKind::Scout
                        && u.pos.distance(c.pos) <= 4
                })
                .collect();
            if near.len() >= 2
                && near
                    .iter()
                    .any(|u| own.iter().all(|h| h.distance(u.pos) > 2))
            {
                return Some(team);
            }
        }
    }
    None
}

fn allied(state: &GameState, a: TeamId, b: TeamId) -> bool {
    state
        .diplomacy
        .as_ref()
        .and_then(|d| d.relation(a, b))
        .is_some_and(|r| r.state == RelationState::Alliance)
}

fn leader(state: &GameState) -> Option<TeamId> {
    let mut vp: Vec<(u32, TeamId)> = state
        .teams()
        .into_iter()
        .filter(|t| {
            *t != TeamId::BARBARIAN && state.players.iter().any(|p| p.alive && p.team == *t)
        })
        .map(|t| (team_vp(state, t), t))
        .collect();
    vp.sort_by(|a, b| b.cmp(a));
    match vp.as_slice() {
        [(a, t), (b, _), ..] if a > b && *a >= 4 => Some(*t),
        _ => None,
    }
}

fn run(seed: u64, index: usize, search: &str, cap: u32) -> serde_json::Value {
    let setup = setup(seed, index, search);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    // (team, trigger turn, kind, answered turn, by)
    let mut triggers: Vec<Trigger> = Vec::new();
    // Per trigger, at the window's end: teams at war with the leader and each rival's plan.
    let mut window_end: Vec<serde_json::Value> = Vec::new();
    // (leader, turn the second team went to war, allied turn, dissolved turn)
    let mut coalitions: Vec<(TeamId, u32, Option<u32>, Option<u32>)> = Vec::new();
    let mut held: std::collections::BTreeSet<TeamId> = Default::default();
    let mut refusals = 0u32;
    // Alliance answers between two teams at war with the public leader: (accepted, reasons).
    let mut answers: Vec<(bool, Option<i32>, Vec<String>)> = Vec::new();
    let mut offers = 0u32;
    let mut winner = None;
    while !session.state.finished && session.state.turn < cap {
        let orders = session.ai_orders();
        // Alliance refusals on the treasury between two teams at war with the same team.
        for (player, list) in &orders {
            for o in list {
                if let Order::RespondToProposal {
                    proposal,
                    accept: false,
                    explanation: Some(e),
                } = o
                    && e.reasons.iter().any(|r| r.contains("empty our treasury"))
                    && let Some(p) = session.state.diplomacy.as_ref().and_then(|d| {
                        d.proposals
                            .iter()
                            .find(|p| p.id == *proposal && p.kind == TreatyKind::Alliance)
                    })
                {
                    let me = session.state.team_of(*player);
                    let shared = session.state.teams().into_iter().any(|t| {
                        t != me
                            && t != p.from
                            && session.state.teams_at_war(me, t)
                            && session.state.teams_at_war(p.from, t)
                            && (leader(&session.state) == Some(t)
                                || session.state.vp_holds.contains_key(&t))
                    });
                    refusals += u32::from(shared);
                }
            }
        }
        if let Some(l) = leader(&session.state) {
            let st = &session.state;
            for (player, list) in &orders {
                let me = st.team_of(*player);
                if !st.teams_at_war(me, l) {
                    continue;
                }
                for o in list {
                    match o {
                        Order::ProposeTreaty {
                            to,
                            kind: TreatyKind::Alliance,
                        } if st.teams_at_war(*to, l) => offers += 1,
                        Order::RespondToProposal {
                            proposal,
                            accept,
                            explanation,
                        } => {
                            if let Some(p) = st.diplomacy.as_ref().and_then(|d| {
                                d.proposals
                                    .iter()
                                    .find(|p| p.id == *proposal && p.kind == TreatyKind::Alliance)
                            }) && st.teams_at_war(p.from, l)
                            {
                                answers.push((
                                    *accept,
                                    explanation.as_ref().and_then(|e| e.score),
                                    explanation
                                        .as_ref()
                                        .map(|e| e.reasons.clone())
                                        .unwrap_or_default(),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let before: Vec<(TeamId, Vec<TeamId>)> = session
            .state
            .teams()
            .into_iter()
            .map(|l| {
                let w = session
                    .state
                    .teams()
                    .into_iter()
                    .filter(|t| *t != TeamId::BARBARIAN && session.state.teams_at_war(*t, l))
                    .collect();
                (l, w)
            })
            .collect();
        let events = session.step(&orders);
        let s = &session.state;
        let turn = s.turn.saturating_sub(1);
        for e in &events {
            match e {
                Event::VictoryHoldStarted { team, .. } if held.insert(*team) => {
                    if !triggers.iter().any(|t| t.0 == *team) {
                        triggers.push((*team, turn, "hold", None));
                    }
                }
                Event::GameWon { team, .. } => winner = Some(team.0),
                _ => {}
            }
        }
        if turn == 80 {
            for t in s.teams() {
                if t != TeamId::BARBARIAN
                    && team_vp(s, t) >= 7
                    && !triggers.iter().any(|x| x.0 == t)
                {
                    triggers.push((t, turn, "vp7@80", None));
                }
            }
        }
        if std::env::var("CONTEST_TRACE").is_ok() {
            for tr in triggers
                .iter()
                .filter(|t| turn >= t.1 && turn <= t.1 + WINDOW)
            {
                let lc: Vec<_> = s
                    .cities
                    .values()
                    .filter(|c| s.team_of(c.owner) == tr.0)
                    .map(|c| c.pos)
                    .collect();
                let mut line =
                    format!("seed {seed} t{turn} leader {} cities {}:", tr.0.0, lc.len());
                for t in s.teams() {
                    if t == tr.0 || t == TeamId::BARBARIAN || !s.teams_at_war(t, tr.0) {
                        continue;
                    }
                    let mil: Vec<_> = s
                        .units
                        .values()
                        .filter(|u| {
                            s.team_of(u.owner) == t
                                && legatum_core::rules::is_military(u.kind)
                                && u.kind != legatum_core::UnitKind::Scout
                        })
                        .collect();
                    let dmin = mil
                        .iter()
                        .flat_map(|u| lc.iter().map(move |h| h.distance(u.pos)))
                        .min();
                    let home = s
                        .cities
                        .values()
                        .filter(|c| s.team_of(c.owner) == t)
                        .flat_map(|c| lc.iter().map(move |h| h.distance(c.pos)))
                        .min();
                    let near = mil
                        .iter()
                        .filter(|u| lc.iter().any(|h| h.distance(u.pos) <= 3))
                        .count();
                    let pl = s.players.iter().find(|p| p.team == t).unwrap();
                    let plan = session.brain(pl.id).and_then(|b| {
                        b.plan_events().iter().rev().find_map(|e| match e {
                            legatum_ai::plan::PlanEvent::Switch { to, .. } => Some(to.clone()),
                            _ => None,
                        })
                    });
                    line.push_str(&format!(
                        " [T{} {:?} units {} near3 {} unitdist {:?} citydist {:?} gold {} plan {:?}]",
                        t.0, pl.faction, mil.len(), near, dmin, home, pl.gold, plan
                    ));
                }
                eprintln!("{line}");
            }
        }
        for tr in triggers.iter_mut() {
            if tr.3.is_some() || turn > tr.1 + WINDOW {
                continue;
            }
            // A campaign an AI started this turn on one of the leader's cities.
            let started = s
                .players
                .iter()
                .filter(|p| p.alive && p.team != tr.0)
                .find_map(|p| {
                    session.brain(p.id)?.campaign_events().iter().find_map(|e| {
                        (e.end.is_none()
                            && e.turn == turn
                            && s.cities
                                .get(&e.target)
                                .is_some_and(|c| s.team_of(c.owner) == tr.0))
                        .then_some(p.team)
                    })
                });
            if let Some(by) = started.or_else(|| pressure(s, tr.0)) {
                tr.3 = Some((turn, by));
            }
        }
        for tr in triggers.iter() {
            if turn == tr.1 + WINDOW {
                let plans: Vec<String> = s
                    .players
                    .iter()
                    .filter(|p| p.alive && p.team != tr.0)
                    .filter_map(|p| {
                        let b = session.brain(p.id)?;
                        let kind = b.plan_events().iter().rev().find_map(|e| match e {
                            legatum_ai::plan::PlanEvent::Switch { to, .. } => Some(to.clone()),
                            _ => None,
                        })?;
                        Some(match kind {
                            legatum_ai::plan::PlanKind::Contain { against } if against == tr.0 => {
                                "contain".to_string()
                            }
                            legatum_ai::plan::PlanKind::Campaign { against, .. }
                                if against == tr.0 =>
                            {
                                "campaign".to_string()
                            }
                            k if k.war_target().is_some() => "war-other".to_string(),
                            _ => "quiet".to_string(),
                        })
                    })
                    .collect();
                let at_war = s
                    .teams()
                    .into_iter()
                    .filter(|t| *t != TeamId::BARBARIAN && s.teams_at_war(*t, tr.0))
                    .count();
                window_end.push(json!({"team": tr.0.0, "at_war": at_war, "plans": plans}));
            }
        }
        // Second war on the public leader.
        if let Some(l) = leader(s) {
            let at_war: Vec<TeamId> = s
                .teams()
                .into_iter()
                .filter(|t| *t != TeamId::BARBARIAN && s.teams_at_war(*t, l))
                .collect();
            let had = before
                .iter()
                .find(|(x, _)| *x == l)
                .map_or(0, |(_, w)| w.len());
            if at_war.len() >= 2
                && had < 2
                && !coalitions
                    .iter()
                    .any(|c| c.0 == l && c.2.is_none() && c.3.is_none())
            {
                coalitions.push((l, turn, None, None));
            }
        }
        for c in coalitions
            .iter_mut()
            .filter(|c| c.2.is_none() && c.3.is_none())
        {
            let at_war: Vec<TeamId> = s
                .teams()
                .into_iter()
                .filter(|t| *t != TeamId::BARBARIAN && s.teams_at_war(*t, c.0))
                .collect();
            if at_war
                .iter()
                .any(|a| at_war.iter().any(|b| a < b && allied(s, *a, *b)))
            {
                c.2 = Some(turn);
            } else if at_war.len() < 2 || !s.players.iter().any(|p| p.alive && p.team == c.0) {
                // The coalition dissolved (peace, or the leader fell) before an Alliance.
                c.3 = Some(turn);
            }
        }
    }
    let mut searches = 0u32;
    let mut identical = 0u32;
    let mut paired = 0u32;
    let mut campaign_starts = 0u32;
    let mut insufficient_aborts = 0u32;
    let mut detail: Vec<serde_json::Value> = Vec::new();
    for p in &session.state.players {
        if let Some(b) = session.brain(p.id) {
            campaign_starts += b
                .campaign_events()
                .iter()
                .filter(|e| e.end.is_none())
                .count() as u32;
            insufficient_aborts += b
                .campaign_events()
                .iter()
                .filter(|e| e.end == Some(legatum_ai::CampaignEnd::InsufficientArmy))
                .count() as u32;
            for e in b.search_events() {
                searches += 1;
                let contain = e
                    .options
                    .iter()
                    .find(|o| matches!(o.kind, legatum_ai::plan::PlanKind::Contain { .. }));
                let quiet = e.options.iter().find(|o| {
                    matches!(
                        o.kind,
                        legatum_ai::plan::PlanKind::Peace { .. }
                            | legatum_ai::plan::PlanKind::Economy
                    )
                });
                if let (Some(c), Some(q)) = (contain, quiet) {
                    paired += 1;
                    if c.samples == q.samples {
                        identical += 1;
                        detail.push(json!({
                            "seat": p.id.0, "turn": e.turn, "chance": e.chance,
                            "trigger": e.trigger, "contain": c.kind, "quiet": q.kind,
                            "samples": c.samples,
                        }));
                    }
                }
            }
        }
    }
    let factions: Vec<String> = session
        .state
        .players
        .iter()
        .map(|p| format!("{:?}", p.faction))
        .collect();
    json!({
        "seed": seed,
        "turns": session.state.turn,
        "capped": !session.state.finished,
        "winner": winner,
        "factions": factions,
        "triggers": triggers.iter().map(|(t, at, kind, ans)| json!({
            "team": t.0, "turn": at, "kind": kind,
            "answered": ans.map(|(turn, by)| json!({"turn": turn, "by": by.0})),
        })).collect::<Vec<_>>(),
        "coalitions": coalitions.iter().map(|(l, at, allied, dissolved)| json!({
            "leader": l.0, "turn": at, "allied": allied, "dissolved": dissolved,
            "gap": allied.map(|x| x - at),
        })).collect::<Vec<_>>(),
        "window_end": window_end,
        "treasury_refusals": refusals,
        "leader_alliance_offers": offers,
        "leader_alliance_answers": answers.iter().map(|(a, score, r)| json!({"accept": a, "score": score, "reasons": r})).collect::<Vec<_>>(),
        "searches": searches,
        "contain_peace_identical": identical,
        "contain_peace_paired": paired,
        "campaign_starts": campaign_starts,
        "insufficient_army_aborts": insufficient_aborts,
        "identical_detail": detail,
    })
}
