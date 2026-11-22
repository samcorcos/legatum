//! Operation search, the offline teacher (AI strategy reset increment 1, the skateboard;
//! `docs/design/ai-strategy/fable-2026-11-07.md` §4.2).
//!
//! At Hard's plan review points inside the designer's decisive windows, on his own tape, this
//! enumerates whole **operations** (not unit moves, not single genes), plays each on the real
//! engine with the existing scripted executors, and logs where one beats Hard's own plan.
//!
//! **Position.** The tape is replayed on its hash-verified states (`audit_states` output, as
//! `seat_run`) with a Hard brain shadowing every seat, so each brain has its real opponent memory
//! at the window turn. The designer's seat becomes Hard from there on.
//!
//! **Operations.** An operation is a forced plan (optional) plus gene overrides on the seat's
//! own decision genes, held for `--duration` turns; the executors (`military.rs`,
//! `city_assault.rs`, `civilians.rs`, `decisive_buy.rs`, ...) carry it out. The generator draws
//! the templates from his decision-diff categories (`docs/playtests/diffs/patterns-2026-11.md`)
//! and his stated reasons (`docs/playtests/human-reasoning.md`): march on a city and strike,
//! buy then strike, found now, escort Settlers, snipe Settlers, post lookouts, wall up, a small
//! army with Production on the economy, refuse or accept Peace, and Hard's own other plan
//! candidates. Most are not in Hard's candidate set: Hard prices plans, not executor policies.
//!
//! **Scoring.** Each operation is played on `--samples` determinized states built from the
//! seat's own `PlayerView` (`search::determinize`, never the true state: an operation chosen
//! for what the seat can't see would teach the wrong reason), the same samples for every
//! operation (common random numbers). Every other seat is Hard. After the operation's turns
//! the seat plays Hard to the end of the game (`--end-cap`), and the label is the win, plus the
//! calibrated win chance (0-10,000) at the operation's horizon. The best operation is then
//! re-scored with Hard's own plan on `--confirm` fresh samples (the selection's winner's curse
//! does not carry over) and, as a diagnostic only, from the true state with `--truth` reseeded
//! dice.
//!
//! One JSON line per window: Hard's chosen plan and its candidates, every operation's wins,
//! paired gain against Hard with a 95% interval, horizon chance, whether the forced plan took,
//! the best operation, its confirmed and true-state gains, the value net's inputs for the seat,
//! and timing.
//!
//!   operation_search --tape TAPE.json --states STATES.jsonl --windows 18-30[,148-152]
//!                    --out OUT.jsonl [--game LABEL] [--seat 0] [--every 2] [--samples 16]
//!                    [--confirm 16] [--truth 16] [--duration 30] [--end-cap 400] [--threads N]
//!                    [--generator 1|2|3] [--commit] [--radius 12] [--skip-hard-wins 14]
//!
//! `--commit` (increment 1b) runs every operation that names a plan through the committed
//! operation executor (`legatum_ai::operation`) and logs its telemetry per operation.
//! `--strike-tactics BUDGET [--tactics-confirm N]` (Training round 2) has a committed strike
//! fight the battle at its target with the Training search (`legatum_ai::training`, round-2
//! settings: BUDGET arena rollouts a battle turn, N full-game rollouts per confirmed candidate)
//! instead of plain Hard tactics.
use legatum_ai::plan::PlanKind;
use legatum_ai::search::OracleNode;
use legatum_ai::value_net;
use legatum_ai::winchance::WinFacts;
use legatum_ai::{Brain, DecisionParams, OpponentMemory, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, CityId, GameState, PlayerId, PlayerView, TeamId, player_view};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::time::Instant;

struct Args {
    tape: String,
    states: String,
    out: String,
    game: String,
    seat: PlayerId,
    windows: Vec<(u32, u32)>,
    every: u32,
    samples: u64,
    confirm: u64,
    truth: u64,
    duration: u32,
    end_cap: u32,
    threads: usize,
    /// 1: the templates; 2: also the designer's kits, combined operations and short strikes;
    /// 3: also the leader operations (strike his empty city, contest his hold, the air answer,
    /// the Celtic bundle).
    generator: u32,
    /// Increment 1b: commit each operation that names a plan through the operation executor
    /// (`legatum_ai::operation`, gene `operations`) instead of forcing the plan while it stays
    /// on offer.
    commit: bool,
    /// Roster radius of a committed strike, in tiles from the target.
    radius: i32,
    /// Skip a window (no operations, no confirmation) where Hard already wins at least this
    /// many of the selection samples (0: never).
    skip_hard_wins: u32,
    /// Training round 2: a committed strike fights its target's battle with the Training
    /// search, this many arena rollouts a battle turn (0: plain Hard tactics).
    strike_tactics: u64,
    /// Full-game rollouts per candidate in that search's check (0: the arena decides).
    tactics_confirm: u32,
}

fn parse_args() -> Args {
    let mut a = Args {
        tape: String::new(),
        states: String::new(),
        out: String::new(),
        game: String::new(),
        seat: PlayerId(0),
        windows: Vec::new(),
        every: 2,
        samples: 16,
        confirm: 16,
        truth: 16,
        duration: 30,
        end_cap: 400,
        threads: 0,
        generator: 1,
        commit: false,
        radius: 12,
        skip_hard_wins: 0,
        strike_tactics: 0,
        tactics_confirm: 16,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut v = || it.next().expect("option needs a value");
        match flag.as_str() {
            "--tape" => a.tape = v(),
            "--states" => a.states = v(),
            "--out" => a.out = v(),
            "--game" => a.game = v(),
            "--seat" => a.seat = PlayerId(v().parse().expect("--seat")),
            "--windows" => {
                a.windows = v()
                    .split(',')
                    .map(|w| {
                        let (lo, hi) = w.split_once('-').unwrap_or((w, w));
                        (lo.parse().expect("window"), hi.parse().expect("window"))
                    })
                    .collect()
            }
            "--every" => a.every = v().parse().expect("--every"),
            "--samples" => a.samples = v().parse().expect("--samples"),
            "--confirm" => a.confirm = v().parse().expect("--confirm"),
            "--truth" => a.truth = v().parse().expect("--truth"),
            "--duration" => a.duration = v().parse().expect("--duration"),
            "--end-cap" => a.end_cap = v().parse().expect("--end-cap"),
            "--threads" => a.threads = v().parse().expect("--threads"),
            "--generator" => a.generator = v().parse().expect("--generator"),
            "--commit" => a.commit = true,
            "--radius" => a.radius = v().parse().expect("--radius"),
            "--skip-hard-wins" => a.skip_hard_wins = v().parse().expect("--skip-hard-wins"),
            "--strike-tactics" => a.strike_tactics = v().parse().expect("--strike-tactics"),
            "--tactics-confirm" => a.tactics_confirm = v().parse().expect("--tactics-confirm"),
            other => panic!("unknown option {other}"),
        }
    }
    assert!(
        !a.tape.is_empty() && !a.states.is_empty() && !a.out.is_empty() && !a.windows.is_empty(),
        "--tape, --states, --windows and --out are required"
    );
    a.every = a.every.max(1);
    a
}

/// The review turns: every `--every` turns through each window, and its last turn.
fn review_turns(a: &Args) -> BTreeSet<u32> {
    let mut turns = BTreeSet::new();
    for &(lo, hi) in &a.windows {
        let mut t = lo;
        while t <= hi {
            turns.insert(t);
            t += a.every;
        }
        turns.insert(hi);
    }
    turns
}

fn hard_brain(state: &GameState, p: PlayerId) -> ScriptedBrain {
    let seed = legatum_wasm::session::brain_seed(state.setup.seed, p);
    ScriptedBrain::with_difficulty(state.player(p).faction, seed, p, AiDifficulty::Hard)
}

struct Snapshot {
    state: GameState,
    memory: BTreeMap<PlayerId, OpponentMemory>,
}

/// Replay the tape on its recorded states with Hard shadowing every seat (`seat_run::warm`).
fn warm(tape: &Tape, a: &Args, turns: &BTreeSet<u32>) -> BTreeMap<u32, Snapshot> {
    let last = *turns.iter().max().expect("turns");
    let reader = std::io::BufReader::new(std::fs::File::open(&a.states).expect("states file"));
    let mut lines = reader.lines();
    let mut read = |expect: &str| -> GameState {
        let line = lines.next().expect("states file too short").unwrap();
        let hash = legatum_core::state::fnv1a(line.as_bytes());
        assert_eq!(hash.to_string(), expect, "recorded state hash");
        serde_json::from_str(&line).expect("recorded state json")
    };
    let mut state = read(&tape.initial_hash.to_string());
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .map(|p| (p.id, hard_brain(&state, p.id)))
        .collect();
    let mut snaps = BTreeMap::new();
    for t in &tape.turns {
        assert_eq!(state.turn, t.turn, "recorded turn sequence");
        if turns.contains(&t.turn) {
            snaps.insert(
                t.turn,
                Snapshot {
                    state: state.clone(),
                    memory: brains
                        .iter()
                        .filter_map(|(id, b)| b.opponent_memory().map(|m| (*id, m)))
                        .collect(),
                },
            );
        }
        if t.turn >= last {
            break;
        }
        for p in state.players.iter().filter(|p| p.alive) {
            let _ = brains
                .get_mut(&p.id)
                .unwrap()
                .orders(&player_view(&state, p.id));
        }
        state = read(&t.hash);
    }
    for t in turns {
        assert!(snaps.contains_key(t), "review turn {t} is not on the tape");
    }
    snaps
}

/// One operation: an optional forced plan plus the seat's genes for its duration.
#[derive(Clone)]
struct Operation {
    name: String,
    template: &'static str,
    plan: Option<PlanKind>,
    genes: Option<DecisionParams>,
    /// Hard's own candidate list holds this plan (with no gene change).
    in_hard_candidates: bool,
    /// Turns the operation is held (None: `--duration`).
    turns: Option<u32>,
}

fn strike_genes(base: &DecisionParams) -> DecisionParams {
    let mut d = *base;
    // March every unit in reach on the target and keep at it: no odds floor, no early-Walls
    // veto, the war army quota up, peace refused while the target stands open.
    d.min_seat_war_odds = 0;
    d.campaign_range = 24;
    d.march_turns = 12;
    d.early_walled_skip = 0;
    d.campaign_wall_cost = 0;
    d.target_wall_cost = 0;
    d.early_campaign_value = 400;
    d.army_war_extra = 4;
    d.war_commit_ratio = 100;
    d.campaign_commit = 1000;
    d.campaign_persist = 50;
    d.march_past_reach = 2;
    d.war_reaim = 2;
    d.peace_redeclare = 2;
    d.army_item_bonus = 200;
    d
}

fn buy_genes(base: &DecisionParams) -> DecisionParams {
    let mut d = *base;
    // Spend the hoard on the decisive war: release the reserve for a forecast gain, buy freely.
    d.decisive_buy = 2;
    d.decisive_buy_min = 0;
    d.buy_hold = 0;
    d.hoard_at_war = 1;
    d.swiss_hoard_any_war = 1;
    d.surplus_purchase_percent = 100;
    d.surplus_upkeep_turns = 10;
    d.reserve_deficit_turns = 0;
    d.purchase_min_value = 0;
    d
}

/// The candidate operations at this review point, from the seat's own view.
fn generate(
    view: &PlayerView,
    base: &DecisionParams,
    hard_pick: Option<&PlanKind>,
    candidates: &[(PlanKind, i32)],
) -> Vec<Operation> {
    let me = view.me;
    let mut ops = vec![Operation {
        name: "hard".into(),
        template: "hard",
        plan: None,
        genes: None,
        in_hard_candidates: true,
        turns: None,
    }];
    // Hard's own other candidates, forced (what Hard considers but did not pick).
    let mut ranked = candidates.to_vec();
    ranked.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
    for (kind, _) in ranked.iter().filter(|(k, _)| Some(k) != hard_pick).take(3) {
        ops.push(Operation {
            name: format!("plan:{kind:?}"),
            template: "hard_candidate",
            plan: Some(kind.clone()),
            genes: None,
            in_hard_candidates: true,
            turns: None,
        });
    }
    // March the army on city X and strike when assembled; and the bought version.
    let own: Vec<_> = view.cities.iter().filter(|c| c.owner == me).collect();
    let mut targets: Vec<(i32, CityId, TeamId, String)> = view
        .cities
        .iter()
        .filter(|c| {
            !c.owner.is_barbarian()
                && !view.same_team(me, c.owner)
                && view.players.iter().any(|p| p.id == c.owner && p.alive)
        })
        .filter_map(|c| {
            let d = own.iter().map(|o| o.pos.distance(c.pos)).min()?;
            (d <= 16).then(|| (d, c.id, view.team_of(c.owner), c.name.clone()))
        })
        .collect();
    targets.sort();
    for (i, (_, city, team, name)) in targets.iter().take(3).enumerate() {
        let plan = PlanKind::Campaign {
            against: *team,
            target: *city,
        };
        ops.push(Operation {
            name: format!("strike:{name}"),
            template: "strike",
            plan: Some(plan.clone()),
            genes: Some(strike_genes(base)),
            in_hard_candidates: false,
            turns: None,
        });
        if i == 0 {
            ops.push(Operation {
                name: format!("buy_strike:{name}"),
                template: "buy_strike",
                plan: Some(plan),
                genes: Some(buy_genes(&strike_genes(base))),
                in_hard_candidates: false,
                turns: None,
            });
        }
    }
    let economy = candidates
        .iter()
        .find(|(k, _)| matches!(k, PlanKind::Economy))
        .map(|(k, _)| k.clone());
    // Found now: Settlers first, found on the tile, the nearest decent site.
    let mut d = *base;
    d.found_now = 100;
    d.found_here = 2;
    d.settler_priority = 4000;
    d.settler_chain = 1;
    d.settler_chain_priority = 10000;
    d.production_settler_percent = 200;
    d.settler_pipeline = 4;
    d.settler_open_cities = 8;
    d.settle_patience = 8;
    d.settler_min_site_score = 40;
    d.settler_travel_penalty = 20;
    d.keep_settler = 100;
    ops.push(Operation {
        name: "expand_now".into(),
        template: "expand_now",
        plan: economy.clone(),
        genes: Some(d),
        in_hard_candidates: false,
        turns: None,
    });
    // Escort every Settler with the nearest unit, buying one if needed.
    let mut d = *base;
    d.settler_escort = 2;
    d.settler_safety = 1;
    d.settler_risk_escort = 1;
    d.escort_home_distance = 0;
    d.escort_search_distance = 12;
    d.escort_buy = 100;
    d.capital_guard = 1;
    ops.push(Operation {
        name: "escort_settlers".into(),
        template: "escort",
        plan: None,
        genes: Some(d),
        in_hard_candidates: false,
        turns: None,
    });
    // Snipe exposed rival Settlers: stalk them, declare for the capture.
    let mut d = *base;
    d.settler_raid = 1;
    d.settler_raid_range = 2;
    d.peace_settler_capture = 200;
    d.settler_stalk = 4;
    d.stalk_contact = 1;
    let snipe = candidates
        .iter()
        .find(|(k, _)| matches!(k, PlanKind::Snipe { .. }))
        .map(|(k, _)| k.clone());
    ops.push(Operation {
        name: "snipe_settlers".into(),
        template: "snipe",
        plan: snipe,
        genes: Some(d),
        in_hard_candidates: false,
        turns: None,
    });
    // Post lookouts on the passes toward every bordering rival.
    let mut d = *base;
    d.sentry = 2;
    d.sentry_posts = 300;
    d.military_scout = 300;
    ops.push(Operation {
        name: "lookouts".into(),
        template: "lookouts",
        plan: None,
        genes: Some(d),
        in_hard_candidates: false,
        turns: None,
    });
    // Wall up: Walls first near any rival, Walls as the garrison, never swapped out.
    let mut d = *base;
    d.walls_near_enemy_bonus = 400;
    d.rush_walls = 1;
    d.keep_walls = 1;
    d.walls_garrison = 1;
    d.walls_garrison_value = 200;
    d.army_ratio_walls = 100;
    ops.push(Operation {
        name: "wall_up".into(),
        template: "wall_up",
        plan: None,
        genes: Some(d),
        in_hard_candidates: false,
        turns: None,
    });
    // A small veteran army; Production and Gold on the economy.
    let mut d = *base;
    d.army_banker = 1;
    d.army_hard_extra = 0;
    d.army_military_bonus = 0;
    d.army_war_extra = 0;
    d.walls_garrison = 1;
    d.walls_peace_quota = 1;
    d.troop_runway = 80;
    d.buy_hold = 3;
    d.research_economy_percent = 200;
    ops.push(Operation {
        name: "small_army_economy".into(),
        template: "small_army_economy",
        plan: economy,
        genes: Some(d),
        in_hard_candidates: false,
        turns: None,
    });
    // Peace: refuse it while at war, or take the offered Peace with each enemy.
    let at_war: Vec<TeamId> = view
        .players
        .iter()
        .filter(|p| p.alive && !p.id.is_barbarian() && view.at_war(me, p.id))
        .map(|p| view.team_of(p.id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if !at_war.is_empty() {
        let mut d = *base;
        d.peace_strong = 50;
        d.peace_redeclare = 2;
        d.campaign_commit = 1000;
        d.leader_commit = 2;
        d.dead_war_peace = 60;
        d.vp_peace = 300;
        ops.push(Operation {
            name: "refuse_peace".into(),
            template: "refuse_peace",
            plan: None,
            genes: Some(d),
            in_hard_candidates: false,
            turns: None,
        });
    }
    for team in at_war {
        let plan = PlanKind::Peace { with: team };
        let mut d = *base;
        d.peace_follows_plan = 2;
        ops.push(Operation {
            name: format!("accept_peace:{}", team.0),
            template: "accept_peace",
            plan: Some(plan),
            genes: Some(d),
            in_hard_candidates: false,
            turns: None,
        });
    }
    ops
}

/// Several operations' gene changes at once: every gene any of them moves from `base`.
fn combine(base: &DecisionParams, parts: &[&DecisionParams]) -> DecisionParams {
    let b = serde_json::to_value(base).expect("genes");
    let mut out = b.clone();
    for part in parts {
        let p = serde_json::to_value(part).expect("genes");
        for (k, v) in p.as_object().expect("genes") {
            if b[k] != *v {
                out[k] = v.clone();
            }
        }
    }
    serde_json::from_value(out).expect("genes")
}

/// Generator 2: the designer's own faction kit, his combined operations (a fortress economy,
/// walled expansion, a small veteran army that keeps coming) and short strikes.
fn widen(ops: &mut Vec<Operation>, base: &DecisionParams, faction: legatum_core::Faction) {
    use legatum_core::Faction;
    let genes = |ops: &[Operation], name: &str| {
        ops.iter()
            .find(|o| o.name == name)
            .and_then(|o| o.genes)
            .unwrap_or(*base)
    };
    let mut kit = *base;
    match faction {
        Faction::Sparta => {
            kit.sparta_force_campaign = 1;
            kit.sparta_conquest_bundle = 1;
            kit.veteran_drill = 6;
        }
        Faction::Switzerland => kit.swiss_playbook = 3,
        Faction::India => kit.india_playbook = 6,
        Faction::Mongols => kit.mongol_horde = 3,
        Faction::Celts => kit.celts_playbook = 7,
        Faction::Aztecs => kit.aztecs_playbook = 7,
        _ => {}
    }
    if kit != *base {
        ops.push(Operation {
            name: "faction_kit".into(),
            template: "faction_kit",
            plan: None,
            genes: Some(kit),
            in_hard_candidates: false,
            turns: None,
        });
    }
    let fortress = combine(
        base,
        &[
            &genes(ops, "small_army_economy"),
            &genes(ops, "wall_up"),
            &genes(ops, "lookouts"),
        ],
    );
    let economy_plan = ops
        .iter()
        .find(|o| o.name == "small_army_economy")
        .and_then(|o| o.plan.clone());
    ops.push(Operation {
        name: "fortress_economy".into(),
        template: "fortress_economy",
        plan: economy_plan,
        genes: Some(fortress),
        in_hard_candidates: false,
        turns: None,
    });
    let walled = combine(
        base,
        &[
            &genes(ops, "expand_now"),
            &genes(ops, "wall_up"),
            &genes(ops, "escort_settlers"),
        ],
    );
    ops.push(Operation {
        name: "expand_walled_escorted".into(),
        template: "expand_walled",
        plan: None,
        genes: Some(walled),
        in_hard_candidates: false,
        turns: None,
    });
    let strikes: Vec<Operation> = ops
        .iter()
        .filter(|o| o.template == "strike")
        .take(2)
        .cloned()
        .collect();
    for (i, strike) in strikes.into_iter().enumerate() {
        let target = strike.name.trim_start_matches("strike:").to_string();
        if i == 0 {
            let mut veteran = combine(base, &[&strike.genes.unwrap_or(*base), &kit]);
            veteran.veteran_drill = 6;
            veteran.army_war_extra = 2;
            veteran.hollow_march = 15;
            ops.push(Operation {
                name: format!("veteran_strike:{target}"),
                template: "veteran_strike",
                plan: strike.plan.clone(),
                genes: Some(veteran),
                in_hard_candidates: false,
                turns: None,
            });
        }
        ops.push(Operation {
            name: format!("short_strike:{target}"),
            template: "short_strike",
            turns: Some(10),
            ..strike
        });
    }
}

/// Generator 3 (the lead, 2026-11-07, from the three 8.9.26 coach reviews): strike the VP
/// leader's empty city while his army is away, contest his hold (his camps or a city), the air
/// answer to his Flight, and the Celtic bundle (light garrisons, Settler theft, one bought war).
fn widen_leader(
    ops: &mut Vec<Operation>,
    view: &PlayerView,
    base: &DecisionParams,
    faction: legatum_core::Faction,
) {
    let me = view.me;
    let leader = view
        .players
        .iter()
        .filter(|p| p.alive && !p.id.is_barbarian() && !view.same_team(me, p.id))
        .max_by_key(|p| (p.vp, std::cmp::Reverse(p.id.0)))
        .map(|p| p.id);
    let own: Vec<_> = view.cities.iter().filter(|c| c.owner == me).collect();
    let strike = strike_genes(base);
    if let Some(leader) = leader {
        // His undefended cities: no visible unit of his within 1 tile, his army 6+ tiles away.
        let mut empty: Vec<(i32, CityId, String)> = view
            .cities
            .iter()
            .filter(|c| c.owner == leader)
            .filter(|c| {
                !view.units.iter().any(|u| {
                    u.owner == leader && u.military && u.pos.distance(c.pos) < 6
                        || u.owner == leader && u.pos.distance(c.pos) <= 1
                })
            })
            .filter_map(|c| {
                let d = own.iter().map(|o| o.pos.distance(c.pos)).min()?;
                (d <= 16).then(|| (d, c.id, c.name.clone()))
            })
            .collect();
        empty.sort();
        if let Some((_, city, name)) = empty.first() {
            let mut d = strike;
            d.hollow_march = 15;
            d.march_turns = 8;
            ops.push(Operation {
                name: format!("strike_empty:{name}"),
                template: "strike_empty",
                plan: Some(PlanKind::Campaign {
                    against: view.team_of(leader),
                    target: *city,
                }),
                genes: Some(d),
                in_hard_candidates: false,
                turns: None,
            });
        }
    }
    let mut hold = strike;
    hold.contest_hold = 300;
    hold.camp_land = 300;
    hold.vp_hold_break = 300;
    hold.late_capture_value = 200;
    hold.leader_commit = 2;
    hold.coalition_push = 1;
    ops.push(Operation {
        name: "contest_hold".into(),
        template: "contest_hold",
        plan: None,
        genes: Some(hold),
        in_hard_candidates: false,
        turns: None,
    });
    let mut air = *base;
    air.air_defense = 300;
    air.air_commitment = 1;
    air.door_defense = 300;
    air.walls_garrison = 1;
    air.keep_walls = 1;
    air.decisive_buy = 2;
    air.buy_hold = 0;
    ops.push(Operation {
        name: "air_answer".into(),
        template: "air_answer",
        plan: None,
        genes: Some(air),
        in_hard_candidates: false,
        turns: None,
    });
    let snipe = ops
        .iter()
        .find(|o| o.template == "snipe")
        .and_then(|o| o.genes)
        .unwrap_or(*base);
    let war = ops.iter().find(|o| o.template == "buy_strike").cloned();
    let mut light = *base;
    light.walls_garrison = 1;
    light.walls_peace_quota = 1;
    light.army_banker = 1;
    if faction == legatum_core::Faction::Celts {
        light.celts_playbook = 7;
    }
    let parts: Vec<DecisionParams> = [Some(snipe), war.as_ref().and_then(|w| w.genes), Some(light)]
        .into_iter()
        .flatten()
        .collect();
    ops.push(Operation {
        name: "celts_bundle".into(),
        template: "celts_bundle",
        plan: war.and_then(|w| w.plan),
        genes: Some(combine(base, &parts.iter().collect::<Vec<_>>())),
        in_hard_candidates: false,
        turns: None,
    });
}

#[derive(Clone, Default)]
struct Outcome {
    won: bool,
    /// The tape's human (the designer) won.
    human_won: bool,
    took: bool,
    /// The committed operation as it ended (`--commit`, operations that name a plan).
    op: Option<legatum_ai::operation::Operation>,
    /// What the searched strike tactics did (`--strike-tactics`).
    tactics: Option<legatum_ai::search::TacticsStats>,
    chance: i32,
    vp: u32,
    cities: usize,
    end_turn: u32,
}

/// Play `op` from `node` and return its outcome.
fn play(mut node: OracleNode, op: &Operation, a: &Args, human: PlayerId) -> Outcome {
    use legatum_ai::operation::{Operation as Committed, OperationKind};
    let turns = op.turns.unwrap_or(a.duration);
    let (took, committed) = match (&op.plan, a.commit) {
        (Some(plan), true) => {
            let kind = match plan {
                PlanKind::Campaign { against, target } => OperationKind::Strike {
                    against: *against,
                    target: *target,
                    buy: matches!(op.template, "buy_strike" | "celts_bundle"),
                },
                other => OperationKind::Pin {
                    plan: other.clone(),
                },
            };
            let genes = op.genes.unwrap_or(node.seat_decisions());
            if a.strike_tactics > 0 {
                node.set_strike_tactics(Some(
                    legatum_ai::training::TrainingConfig {
                        rollouts: 32,
                        screen: 8,
                        horizon: 12,
                        max_passes: 2,
                        cap_ms: 0,
                        max_battles: 1,
                        max_rollouts_per_battle: a.strike_tactics,
                        source: "operation".into(),
                        ..Default::default()
                    }
                    .round2()
                    .with_confirm(a.tactics_confirm),
                ));
            }
            let ended = node.advance_committed(
                Committed::new(kind, node.state().turn, turns, a.radius),
                &genes,
                turns,
            );
            (ended.log.took, Some(ended))
        }
        _ => (
            node.advance_operation(op.plan.as_ref(), op.genes.as_ref(), turns),
            None,
        ),
    };
    let tactics = node.tactics_stats().cloned();
    node.set_strike_tactics(None);
    // Every operation is read at the same horizon.
    node.continue_opening(a.duration.saturating_sub(turns));
    let m = node.metrics();
    let start = node.state().turn;
    node.continue_opening(a.end_cap.saturating_sub(start));
    let end = node.metrics();
    let st = node.state();
    Outcome {
        won: end.won,
        human_won: st.finished && st.winner == Some(st.player(human).team),
        took,
        op: committed,
        tactics,
        chance: m.chance,
        vp: m.vp,
        cities: m.cities,
        end_turn: end.turn,
    }
}

/// The committed operations' logs over one operation's samples (`--commit`).
fn committed_stats(r: &[Outcome]) -> Value {
    let ops: Vec<&legatum_ai::operation::Operation> =
        r.iter().filter_map(|o| o.op.as_ref()).collect();
    if ops.is_empty() {
        return Value::Null;
    }
    let n = ops.len() as f64;
    let mean = |f: &dyn Fn(&legatum_ai::operation::Operation) -> f64| {
        round1(ops.iter().map(|o| f(o)).sum::<f64>() / n)
    };
    let mut outcomes: BTreeMap<String, u32> = BTreeMap::new();
    for o in &ops {
        *outcomes
            .entry(
                serde_json::to_value(o.outcome)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default(),
            )
            .or_default() += 1;
    }
    json!({
        "samples": ops.len(),
        "took": ops.iter().filter(|o| o.log.took).count(),
        "struck": ops.iter().filter(|o| o.log.strike_turn.is_some()).count(),
        "attacked": ops.iter().filter(|o| o.log.attacks > 0).count(),
        "stormed": ops.iter().filter(|o| o.log.storm_turn.is_some()).count(),
        "captured": outcomes.get("captured").copied().unwrap_or(0),
        "roster_start": mean(&|o| f64::from(o.log.roster_start)),
        "roster_max": mean(&|o| f64::from(o.log.roster_max)),
        "arrived": mean(&|o| f64::from(o.log.arrived)),
        "attacks": mean(&|o| f64::from(o.log.attacks)),
        "bought": mean(&|o| f64::from(o.log.bought)),
        "gold_spent": mean(&|o| f64::from(o.log.gold_spent)),
        "overrides": mean(&|o| f64::from(o.log.overrides)),
        "strike_turn": mean(&|o| o.log.strike_turn.map_or(0.0, |t| f64::from(t.saturating_sub(o.start_turn)))),
        "outcomes": outcomes,
        "tactics": tactics_stats(r),
    })
}

/// The searched strike tactics over one operation's samples (`--strike-tactics`).
fn tactics_stats(r: &[Outcome]) -> Value {
    let t: Vec<&legatum_ai::search::TacticsStats> =
        r.iter().filter_map(|o| o.tactics.as_ref()).collect();
    if t.iter().all(|t| t.turns == 0) {
        return Value::Null;
    }
    let mut verdicts: BTreeMap<String, u32> = BTreeMap::new();
    for s in &t {
        for (k, v) in &s.verdicts {
            *verdicts.entry(k.clone()).or_default() += v;
        }
    }
    json!({
        "samples": t.iter().filter(|t| t.turns > 0).count(),
        "turns": t.iter().map(|t| t.turns).sum::<u32>(),
        "changed": t.iter().map(|t| t.changed).sum::<u32>(),
        "rollouts": t.iter().map(|t| t.rollouts).sum::<u64>(),
        "verdicts": verdicts,
    })
}

/// Mean paired gain (op minus Hard) in percentage points, with a 95% normal interval.
fn paired(op: &[Outcome], hard: &[Outcome], f: impl Fn(&Outcome) -> f64) -> (f64, f64, f64) {
    let d: Vec<f64> = op.iter().zip(hard).map(|(x, y)| f(x) - f(y)).collect();
    let n = d.len().max(1) as f64;
    let mean = d.iter().sum::<f64>() / n;
    let var = d.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let half = 1.96 * (var / n).sqrt();
    (mean, mean - half, mean + half)
}

fn win(o: &Outcome) -> f64 {
    if o.won { 100.0 } else { 0.0 }
}

fn human_win(o: &Outcome) -> f64 {
    if o.human_won { 100.0 } else { 0.0 }
}

fn chance_pp(o: &Outcome) -> f64 {
    f64::from(o.chance) / 100.0
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn search(snap: &Snapshot, a: &Args) -> Value {
    let started = Instant::now();
    let seat = a.seat;
    let mut state = snap.state.clone();
    let human = state
        .players
        .iter()
        .find(|p| p.is_human)
        .map_or(seat, |p| p.id);
    state.players[seat.0 as usize].is_human = false;
    if let Some(p) = state.setup.players.get_mut(seat.0 as usize) {
        p.is_human = false;
        p.difficulty = AiDifficulty::Hard;
    }
    let brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            let mut b = hard_brain(&state, p.id);
            if let Some(m) = snap.memory.get(&p.id) {
                b.restore_opponent_memory(m.clone());
            }
            b
        })
        .collect();
    let seat_brain = &brains[seat.0 as usize];
    let view = player_view(&state, seat);
    // Hard's own decision this turn, on a copy.
    let mut probe = seat_brain.rollout_copy();
    let _ = probe.orders(&view);
    let hard_pick = probe
        .opponent_memory()
        .and_then(|m| m.campaigns.plan.active.map(|p| p.kind));
    let candidates = OracleNode::from_recorded_state(&state, &brains, seat).choices();
    let mut ops = generate(
        &view,
        &seat_brain.decisions,
        hard_pick.as_ref(),
        &candidates,
    );
    if a.generator >= 2 {
        widen(&mut ops, &seat_brain.decisions, state.player(seat).faction);
    }
    if a.generator >= 3 {
        widen_leader(
            &mut ops,
            &view,
            &seat_brain.decisions,
            state.player(seat).faction,
        );
    }

    // Selection: every operation on the same determinized samples. Hard first: a window it
    // already (nearly) always wins has no room for a 10 pp gain (`--skip-hard-wins`).
    let hard_only: Vec<Outcome> = (0..a.samples)
        .into_par_iter()
        .map(|s| play(OracleNode::new(&view, seat_brain, s), &ops[0], a, human))
        .collect();
    let hard_wins = hard_only.iter().filter(|o| o.won).count() as u32;
    if a.skip_hard_wins > 0 && hard_wins >= a.skip_hard_wins {
        return json!({
            "game": a.game, "turn": state.turn, "seat": seat.0, "human": human.0,
            "faction": format!("{:?}", state.player(seat).faction),
            "rules_version": legatum_core::RULES_VERSION,
            "hard_plan": hard_pick.as_ref().map(|k| format!("{k:?}")),
            "skipped": format!("Hard wins {hard_wins} of {}", a.samples),
            "hard_wins": hard_wins, "samples": a.samples, "commit": a.commit,
            "timing_ms": {"total": started.elapsed().as_millis() as u64},
            "rollouts": hard_only.len(),
        });
    }
    let jobs: Vec<(usize, u64)> = (1..ops.len())
        .flat_map(|o| (0..a.samples).map(move |s| (o, s)))
        .collect();
    let mut results: Vec<Outcome> = hard_only;
    results.extend(
        jobs.par_iter()
            .map(|&(o, s)| play(OracleNode::new(&view, seat_brain, s), &ops[o], a, human))
            .collect::<Vec<_>>(),
    );
    let by_op: Vec<&[Outcome]> = results.chunks(a.samples as usize).collect();
    let hard = by_op[0];
    let mut rows = Vec::new();
    let mut best: Option<(usize, (u32, i64))> = None;
    for (i, op) in ops.iter().enumerate() {
        let r = by_op[i];
        let wins = r.iter().filter(|o| o.won).count() as u32;
        let chance: i64 = r.iter().map(|o| i64::from(o.chance)).sum();
        let (g, lo, hi) = paired(r, hard, win);
        let (cg, clo, chi) = paired(r, hard, chance_pp);
        rows.push(json!({
            "name": op.name, "template": op.template,
            "plan": op.plan.as_ref().map(|k| format!("{k:?}")),
            "in_hard_candidates": op.in_hard_candidates,
            "plan_listed": op.plan.as_ref().is_some_and(|p| candidates.iter().any(|(k, _)| k == p)),
            "wins": wins, "samples": a.samples,
            "win_share": round1(100.0 * f64::from(wins) / a.samples as f64),
            "gain_pp": round1(g), "gain_lo": round1(lo), "gain_hi": round1(hi),
            "horizon_chance_pp": round1(chance as f64 / a.samples as f64 / 100.0),
            "horizon_gain_pp": round1(cg), "horizon_gain_lo": round1(clo), "horizon_gain_hi": round1(chi),
            "turns": op.turns.unwrap_or(a.duration),
            "human_wins": r.iter().filter(|o| o.human_won).count(),
            "human_gain_pp": round1(paired(r, hard, human_win).0),
            "took": r.iter().filter(|o| o.took).count(),
            "committed": committed_stats(r),
            "horizon_vp": round1(r.iter().map(|o| f64::from(o.vp)).sum::<f64>() / a.samples as f64),
            "horizon_cities": round1(r.iter().map(|o| o.cities as f64).sum::<f64>() / a.samples as f64),
            "mean_end_turn": round1(r.iter().map(|o| f64::from(o.end_turn)).sum::<f64>() / a.samples as f64),
        }));
        if i > 0 && best.is_none_or(|(_, k)| (wins, chance) > k) {
            best = Some((i, (wins, chance)));
        }
    }
    let selected = Instant::now();
    let (bi, _) = best.expect("an operation besides Hard");
    // Confirmation on fresh samples: the selection's maximum is biased upward.
    let fresh: Vec<(usize, u64)> = [0, bi]
        .iter()
        .flat_map(|&o| (0..a.confirm).map(move |s| (o, 1_000_000 + s)))
        .collect();
    let confirm: Vec<Outcome> = fresh
        .par_iter()
        .map(|&(o, s)| play(OracleNode::new(&view, seat_brain, s), &ops[o], a, human))
        .collect();
    let (ch, cb) = confirm.split_at(a.confirm as usize);
    let confirmed = (a.confirm > 0).then(|| {
        let (g, lo, hi) = paired(cb, ch, win);
        let (cg, clo, chi) = paired(cb, ch, chance_pp);
        json!({"hard_wins": ch.iter().filter(|o| o.won).count(),
            "best_wins": cb.iter().filter(|o| o.won).count(), "samples": a.confirm,
            "gain_pp": round1(g), "gain_lo": round1(lo), "gain_hi": round1(hi),
            "horizon_gain_pp": round1(cg), "horizon_gain_lo": round1(clo), "horizon_gain_hi": round1(chi)})
    });
    // Diagnostic only: the true state with reseeded dice (omniscient regret, never a label).
    let truth_jobs: Vec<(usize, u64)> = [0, bi]
        .iter()
        .flat_map(|&o| (0..a.truth).map(move |s| (o, s)))
        .collect();
    let truth: Vec<Outcome> = truth_jobs
        .par_iter()
        .map(|&(o, s)| {
            let mut st = state.clone();
            st.dice_seed = Some(
                Rng::derive(
                    state.setup.seed,
                    &[purpose::AI, 0x09e5, state.turn as u64, s],
                )
                .next_u64(),
            );
            play(
                OracleNode::from_recorded_state(&st, &brains, seat),
                &ops[o],
                a,
                human,
            )
        })
        .collect();
    let (th, tb) = truth.split_at(a.truth as usize);
    let truth_row = (a.truth > 0).then(|| {
        let (g, lo, hi) = paired(tb, th, win);
        json!({"hard_wins": th.iter().filter(|o| o.won).count(),
            "best_wins": tb.iter().filter(|o| o.won).count(), "samples": a.truth,
            "gain_pp": round1(g), "gain_lo": round1(lo), "gain_hi": round1(hi)})
    });
    let facts = WinFacts::of(&view);
    let team = view.team;
    let features = value_net::inputs(&facts)
        .into_iter()
        .find(|(t, _)| *t == team)
        .map(|(_, x)| x.to_vec());
    let raw = facts
        .features()
        .into_iter()
        .find(|(t, _)| *t == team)
        .map(|(_, x)| x.to_vec());
    json!({
        "game": a.game, "turn": state.turn, "seat": seat.0, "human": human.0,
        "faction": format!("{:?}", state.player(seat).faction),
        "rules_version": legatum_core::RULES_VERSION,
        "hard_plan": hard_pick.as_ref().map(|k| format!("{k:?}")),
        "hard_candidates": candidates.iter().map(|(k, v)| json!([format!("{k:?}"), v])).collect::<Vec<_>>(),
        "duration": a.duration, "end_cap": a.end_cap, "generator": a.generator,
        "commit": a.commit, "radius": a.radius,
        "strike_tactics": a.strike_tactics, "tactics_confirm": a.tactics_confirm,
        "operations": rows,
        "best": ops[bi].name, "best_template": ops[bi].template,
        "regret_pp": rows[bi]["gain_pp"], "regret_lo": rows[bi]["gain_lo"], "regret_hi": rows[bi]["gain_hi"],
        "confirm": confirmed, "truth": truth_row,
        "features": {"value_net_inputs": features, "win_features": raw,
            "hand_chance": facts.chance()},
        "timing_ms": {"selection": selected.duration_since(started).as_millis() as u64,
            "total": started.elapsed().as_millis() as u64},
        "rollouts": results.len() + confirm.len() + truth.len(),
    })
}

fn main() {
    let a = parse_args();
    if a.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(a.threads)
            .build_global()
            .expect("thread pool");
    }
    let tape =
        Tape::from_json(&std::fs::read_to_string(&a.tape).expect("read tape")).expect("tape");
    let turns = review_turns(&a);
    let t0 = Instant::now();
    let snaps = warm(&tape, &a, &turns);
    eprintln!(
        "operation_search {}: {} review points warmed in {} ms",
        a.game,
        snaps.len(),
        t0.elapsed().as_millis()
    );
    let mut out = std::fs::File::create(&a.out).expect("out file");
    for (turn, snap) in &snaps {
        let row = search(snap, &a);
        eprintln!(
            "{} T{turn}: hard {} | best {} {:+} pp (confirm {:+}, truth {:+}) | {} rollouts {} ms",
            a.game,
            row["hard_plan"],
            row["best"],
            row["regret_pp"],
            row["confirm"]["gain_pp"],
            row["truth"]["gain_pp"],
            row["rollouts"],
            row["timing_ms"]["total"]
        );
        writeln!(out, "{row}").expect("write");
        out.flush().expect("flush");
    }
}
