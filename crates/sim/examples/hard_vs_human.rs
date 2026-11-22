//! Fixed-map, human-seat benchmark. The checked-in maps include the tape-derived setup and
//! human milestones, so cloud runs need only this binary and its arguments.
//!
//! hard_vs_human --brain hard|hard+search [--salts 64] [--salt-start 1] [--seat-ai-rng-offset 0]
//! hard_vs_human --brain normal|easy ...   (calibration: a weaker seat must score lower)
//! hard_vs_human --brain params|params+search --ai-params FILE [--salts 64]
//! Add --seat-brain or --opponent-brain to apply FILE to only those seats; default is all seats.
//!
//! The rivals play plain `ScriptedBrain` Hard, which is what the app's default Hard lobby gives
//! the designer. The checked-in maps carry `search: true` on their AI seats (the experimental
//! "Hard + Search" lobby level), but Search drops air strikes unless `search_air_orders` is on,
//! so searching rivals never bomb, unlike the app's Hard. `--rival-search` (or an arm's
//! `"rival_search": true`) puts the Search wrapper back on the rivals that have the flag.
//! hard_vs_human --extract OUT.json SAVE.legatum [SAVE.legatum ...]

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, PlayerId, scoring};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

const MAPS: &str = include_str!("../data/hard-vs-human-maps.json");
const CHECKPOINTS: [u32; 5] = [30, 50, 60, 62, 100];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeatMode {
    All,
    Seat,
    Opponents,
}

impl SeatMode {
    fn candidate(self, is_human_seat: bool) -> bool {
        match self {
            Self::All => true,
            Self::Seat => is_human_seat,
            Self::Opponents => !is_human_seat,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "all-seats",
            Self::Seat => "seat-only",
            Self::Opponents => "opponent-only",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MapRecord {
    id: String,
    source: String,
    setup: GameSetup,
    human_seat: u8,
    human: HumanMilestones,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recorded_winner: Option<WinnerMilestones>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HumanMilestones {
    cities_t30: u32,
    cities_t50: u32,
    vp_t60: u32,
    vp_t100: Option<u32>,
    win_turn: Option<u32>,
    win_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WinnerMilestones {
    seat: u8,
    faction: Faction,
    cities_t30: u32,
    cities_t50: u32,
    cities_t80: Option<u32>,
    vp_t60: u32,
    vp_t80: Option<u32>,
    final_vp: u32,
    win_turn: u32,
    win_type: String,
}

fn salted(seed: u64, salt: u64) -> u64 {
    Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
}

fn seat_ai_seed(seed: u64, salt: u64, offset: u64) -> u64 {
    if offset == 0 {
        // Keep the historical stream byte-for-byte for the default arm.
        salted(seed, salt)
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt, offset]).next_u64()
    }
}

fn player_ai_seed(seed: u64, salt: u64, player: PlayerId, seat: PlayerId, offset: u64) -> u64 {
    if player == seat {
        seat_ai_seed(seed, salt, offset)
    } else {
        salted(seed, salt)
    }
}

fn load_save(path: &Path) -> legatum_wasm::history::Recording {
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(path).expect("read save"))
        .expect("parse save");
    let game = outer["game"].as_str().expect("save game JSON string");
    let game_value: Value = serde_json::from_str(game).expect("parse game JSON");
    serde_json::from_value(game_value["archive"].clone()).expect("read archived recording")
}

fn milestones_from_tape(
    recording: &legatum_wasm::history::Recording,
) -> (u8, HumanMilestones, Option<WinnerMilestones>) {
    recording.validate().expect("validate tape archive");
    let setup = &recording.tape.setup;
    let human_seat = setup
        .players
        .iter()
        .position(|p| p.is_human)
        .expect("one human seat") as u8;
    let checkpoint = |turn, seat| {
        let frame = recording
            .telemetry
            .iter()
            .find(|frame| frame.turn == turn)?;
        let player = frame
            .players
            .iter()
            .find(|player| player.player == seat)
            .expect("telemetry seat");
        Some((player.cities, player.vp))
    };
    let win_turn = recording.metadata.turn_count;
    let win_type = match recording.metadata.outcome.as_str() {
        "vp" => "vp_hold",
        "elimination" => "elimination",
        other => panic!("expected completed game, got {other}"),
    };
    let winner = recording
        .metadata
        .winner
        .expect("completed game has winner");
    let winner_seat = recording
        .metadata
        .seats
        .iter()
        .find(|seat| seat.team == winner)
        .expect("winning team has seat");
    let human_won = winner_seat.player == human_seat;
    let human = HumanMilestones {
        cities_t30: checkpoint(30, human_seat).expect("human tape covers T30").0,
        cities_t50: checkpoint(50, human_seat).expect("human tape covers T50").0,
        vp_t60: checkpoint(60, human_seat).expect("human tape covers T60").1,
        vp_t100: checkpoint(100, human_seat).map(|values| values.1),
        win_turn: human_won.then_some(win_turn),
        win_type: human_won.then(|| win_type.into()),
    };
    let ai_winner = (!human_won).then(|| WinnerMilestones {
        seat: winner_seat.player,
        faction: winner_seat.faction,
        cities_t30: checkpoint(30, winner_seat.player)
            .expect("winner tape covers T30")
            .0,
        cities_t50: checkpoint(50, winner_seat.player)
            .expect("winner tape covers T50")
            .0,
        cities_t80: checkpoint(80, winner_seat.player).map(|values| values.0),
        vp_t60: checkpoint(60, winner_seat.player)
            .expect("winner tape covers T60")
            .1,
        vp_t80: checkpoint(80, winner_seat.player).map(|values| values.1),
        final_vp: winner_seat.final_vp,
        win_turn,
        win_type: win_type.into(),
    });
    (human_seat, human, ai_winner)
}

#[derive(Default)]
struct Metrics {
    /// Checkpoint turn -> (cities, VP). A game that ends before a checkpoint has no entry.
    snapshots: BTreeMap<u32, (u32, u32)>,
    win_turn: Option<u32>,
    win_type: Option<String>,
    until: Option<Value>,
    /// Diagnostic counters for the recorded seat (built units by class, disbands, captures).
    diag: BTreeMap<&'static str, i64>,
    /// Turn -> every living empire's VP standing (sources, era, cities).
    standings: BTreeMap<u32, Value>,
    /// Team pair -> turn their latest Peace took effect, for the peace-flip counters.
    peace_turn: BTreeMap<(u8, u8), u32>,
    /// Turn -> every living empire's Iron and Coal supply, what uses it, and its units.
    strategic: BTreeMap<u32, Value>,
    /// Per player: [Troopers completed, Military Engineers completed] over the whole game.
    strategic_built: BTreeMap<u8, [i64; 2]>,
    /// Per AI: reached Flight, Fighters completed (including buys), Bomber strikes intercepted.
    air: BTreeMap<u8, AirMetrics>,
    /// Ruins collected and first diplomatic contact, per simulated AI.
    ruins_by_ai: BTreeMap<u8, u32>,
    first_contact_by_ai: BTreeMap<u8, u32>,
    /// Every war between empires, for the war-quality counters (`note_war`).
    wars: WarLog,
    /// Sparta's own wars and captures, across seat and rival roles.
    sparta: SpartaMetrics,
    /// Per AI, campaign starts/ends within three turns and lost-city retakes.
    campaign_quality: BTreeMap<u8, CampaignQuality>,
    /// Per player: cities at T50, T81 and T100, Settlers lost (captured or destroyed) and
    /// rival Settlers captured, for the expansion measures.
    expansion: BTreeMap<u8, ExpansionMetrics>,
}

#[derive(Default, Serialize)]
struct ExpansionMetrics {
    cities_t50: Option<u32>,
    cities_t81: Option<u32>,
    cities_t100: Option<u32>,
    settlers_lost: u32,
    settlers_taken: u32,
}

#[derive(Default, Serialize)]
struct CampaignQuality {
    flaps: u32,
    retake_opportunities: u32,
    retake_successes: u32,
}

#[derive(Default, Serialize)]
struct SpartaMetrics {
    first_war: Option<u32>,
    first_declared_war: Option<u32>,
    captures: u32,
    walled_captures: u32,
}

#[derive(Default, Serialize)]
struct AirMetrics {
    flight: bool,
    fighters_built: u32,
    bomber_strikes_intercepted: u32,
    aircraft_turns: u32,
    unsupplied_aircraft_turns: u32,
    oil_deposit_turns: u32,
    unimproved_oil_deposit_turns: u32,
    oil_shortage_turns: u32,
    oil_blocked_upgrade_turns: u32,
    oil_blocked_tank_upgrade_turns: u32,
    oil_air_build_turns: u32,
    oil_air_starts_with_waiting_upgrades: u32,
    oil_air_starts_with_waiting_tank_upgrades: u32,
    oil_air_starts_with_waiting_labs: u32,
    oil_nearby_unclaimed_deposit_turns: u32,
    oil_enemy_deposit_turns: u32,
    oil_threatened_deposit_turns: u32,
    oil_undefended_threatened_deposit_turns: u32,
    oil_claims: u32,
    oil_takeovers: u32,
    oil_losses: u32,
    oil_improvements: u32,
}

/// One war between two teams: when it started and ended, battles fought between them and
/// cities taken from each other while it lasted.
#[derive(Clone)]
struct WarRec {
    a: u8,
    b: u8,
    start: u32,
    end: Option<u32>,
    battles: u32,
    captures: u32,
    /// Teams that declared (both for a mutual declaration).
    declarers: Vec<u8>,
    /// Attacks by units of team `a` on team `b`, and of `b` on `a`.
    attacks: [u32; 2],
    /// `LEGATUM_HVH_WAR_DIAG=1`: what each side's plan and army did while the war lasted.
    sides: [SideDiag; 2],
}

/// One side of a war, sampled every turn of it (`LEGATUM_HVH_WAR_DIAG=1` only): turns by the
/// side's plan toward this enemy and how close its army stood to the enemy's nearest city,
/// the campaigns it ended, and its closest approach.
#[derive(Clone, Default)]
struct SideDiag {
    turns: BTreeMap<String, u32>,
    campaign_ends: BTreeMap<String, u32>,
    campaign_starts: u32,
    min_dist: Option<i32>,
}

/// War-quality counters (user, 2026-10-17: coalitions "declare 'war' but don't do anything"):
/// wars with no battle, captures per war, and coalition members striking the same target in
/// the same 3-turn window.
#[derive(Default)]
struct WarLog {
    open: BTreeMap<(u8, u8), WarRec>,
    done: Vec<WarRec>,
    /// (target team, attacking team) -> turns its units attacked the target during a war.
    attacks: BTreeMap<(u8, u8), Vec<u32>>,
}

fn team_of(state: &legatum_core::GameState, p: PlayerId) -> Option<u8> {
    (!p.is_barbarian()).then(|| state.player(p).team.0)
}

fn note_war(state: &legatum_core::GameState, log: &mut WarLog, turn: u32, event: &Event) {
    let key = |a: u8, b: u8| if a <= b { (a, b) } else { (b, a) };
    match event {
        Event::WarDeclared { relation } => {
            let k = key(relation.a.0, relation.b.0);
            log.open.entry(k).or_insert(WarRec {
                a: k.0,
                b: k.1,
                start: turn,
                end: None,
                battles: 0,
                captures: 0,
                declarers: relation.declarers.iter().map(|t| t.0).collect(),
                attacks: [0; 2],
                sides: Default::default(),
            });
        }
        Event::PeaceEstablished { relation } => {
            if let Some(mut w) = log.open.remove(&key(relation.a.0, relation.b.0)) {
                w.end = Some(turn);
                log.done.push(w);
            }
        }
        Event::Battle {
            attacker,
            attacker_owner,
            defender,
            ..
        } => {
            let owner = match defender {
                legatum_core::events::Defender::Unit { owner, .. }
                | legatum_core::events::Defender::City { owner, .. } => *owner,
            };
            if let (Some(ta), Some(td)) = (team_of(state, *attacker_owner), team_of(state, owner))
                && let Some(w) = log.open.get_mut(&key(ta, td))
            {
                w.battles += 1;
                if attacker.unit().is_some() {
                    log.attacks.entry((td, ta)).or_default().push(turn);
                    w.attacks[usize::from(ta != w.a)] += 1;
                }
            }
        }
        Event::CityCaptured { from, to, .. } => {
            if let (Some(a), Some(b)) = (team_of(state, *from), team_of(state, *to))
                && let Some(w) = log.open.get_mut(&key(a, b))
            {
                w.captures += 1;
            }
        }
        _ => {}
    }
}

/// Close the open wars and write the counters: `wars`, `wars_no_battle`, `wars_captured`
/// (wars with a capture), `war_captures`; pairs of teams at war with the same target at
/// overlapping times (`coalition_pairs`) and those whose units attacked it within 2 turns of
/// each other (`coalition_joint`); each also for wars on the seat's team (`_on_seat`).
fn close_wars(log: &mut WarLog, diag: &mut BTreeMap<&'static str, i64>, turn: u32, seat_team: u8) {
    for (_, mut w) in std::mem::take(&mut log.open) {
        w.end = Some(turn);
        log.done.push(w);
    }
    let mut add = |k: &'static str, ks: &'static str, on_seat: bool, n: i64| {
        *diag.entry(k).or_default() += n;
        if on_seat {
            *diag.entry(ks).or_default() += n;
        }
    };
    for w in &log.done {
        let on_seat = w.a == seat_team || w.b == seat_team;
        add("wars", "wars_on_seat", on_seat, 1);
        add(
            "wars_no_battle",
            "wars_no_battle_on_seat",
            on_seat,
            i64::from(w.battles == 0),
        );
        add(
            "wars_captured",
            "wars_captured_on_seat",
            on_seat,
            i64::from(w.captures > 0),
        );
        add(
            "war_captures",
            "war_captures_on_seat",
            on_seat,
            i64::from(w.captures),
        );
        // The declaring side's units never attacked: the war did nothing for the AI that
        // started it (user, 2026-10-18: "you can be 'at war' but that war doesn't do anything").
        let declarer_attacked = w
            .declarers
            .iter()
            .any(|t| (*t == w.a && w.attacks[0] > 0) || (*t == w.b && w.attacks[1] > 0));
        add(
            "wars_declarer_idle",
            "wars_declarer_idle_on_seat",
            on_seat,
            i64::from(!w.declarers.is_empty() && !declarer_attacked),
        );
    }
    let wars = &log.done;
    for t in wars
        .iter()
        .flat_map(|w| [w.a, w.b])
        .collect::<std::collections::BTreeSet<_>>()
    {
        let against: Vec<&WarRec> = wars.iter().filter(|w| w.a == t || w.b == t).collect();
        for (i, x) in against.iter().enumerate() {
            for y in &against[i + 1..] {
                let (ex, ey) = (x.end.unwrap_or(turn), y.end.unwrap_or(turn));
                let (from, to) = (x.start.max(y.start), ex.min(ey));
                let mx = if x.a == t { x.b } else { x.a };
                let my = if y.a == t { y.b } else { y.a };
                if from > to || mx == my {
                    continue;
                }
                let on_seat = t == seat_team;
                add("coalition_pairs", "coalition_pairs_on_seat", on_seat, 1);
                let hits = |m: u8| -> Vec<u32> {
                    log.attacks
                        .get(&(t, m))
                        .map(|v| {
                            v.iter()
                                .copied()
                                .filter(|d| (from..=to).contains(d))
                                .collect()
                        })
                        .unwrap_or_default()
                };
                let (hx, hy) = (hits(mx), hits(my));
                let joint = hx.iter().any(|a| hy.iter().any(|b| a.abs_diff(*b) <= 2));
                add(
                    "coalition_joint",
                    "coalition_joint_on_seat",
                    on_seat,
                    i64::from(joint),
                );
            }
        }
    }
}

/// `LEGATUM_HVH_WAR_DIAG=1`: sample every open war once a turn, for both sides. A side is its
/// team's first living member; its plan is the goal-driven plan in its AI memory (Hard), read
/// relative to this enemy: `campaign`, `contain` or `snipe` on it, `other_war` (the plan fights
/// someone else), `peace` (Economy or Peace: the plan wants this war over), `hold` or `none`.
/// The army part is the side's nearest armed unit to the enemy's nearest city: `near` (3 hexes
/// or less), `mid` (4 to 8), `far`, or `no_army`. Campaign starts and ends are read from the
/// memory's campaign log and charged to the war with the target city's owner.
fn sample_wars(
    state: &legatum_core::GameState,
    brains: &[Box<dyn Brain>],
    log: &mut WarLog,
    seen: &mut BTreeMap<u8, usize>,
) {
    use legatum_ai::plan::PlanKind;
    let mut memories: BTreeMap<u8, Option<legatum_ai::OpponentMemory>> = BTreeMap::new();
    let mut memory = |p: PlayerId| {
        memories
            .entry(p.0)
            .or_insert_with(|| brains[p.0 as usize].opponent_memory())
            .clone()
    };
    let keys: Vec<(u8, u8)> = log.open.keys().copied().collect();
    for k in keys {
        for (side, (mine, theirs)) in [(k.0, k.1), (k.1, k.0)].into_iter().enumerate() {
            let Some(&p) = state
                .team_members(legatum_core::TeamId(mine))
                .iter()
                .find(|p| state.player(**p).alive)
            else {
                continue;
            };
            let m = memory(p);
            let plan = m
                .as_ref()
                .and_then(|m| m.campaigns.plan.active.as_ref())
                .map(|plan| &plan.kind);
            let enemy = legatum_core::TeamId(theirs);
            let label = match plan {
                None => "none",
                Some(PlanKind::Campaign { against, .. }) if *against == enemy => "campaign",
                Some(PlanKind::Contain { against }) if *against == enemy => "contain",
                Some(PlanKind::Snipe { against, .. }) if *against == enemy => "snipe",
                Some(PlanKind::Hold { .. }) => "hold",
                Some(kind) if kind.war_target().is_some() => "other_war",
                Some(_) => "peace",
            };
            let enemy_cities: Vec<_> = state
                .cities
                .values()
                .filter(|c| !c.owner.is_barbarian() && state.player(c.owner).team == enemy)
                .map(|c| c.pos)
                .collect();
            let dist = state
                .units
                .values()
                .filter(|u| u.owner == p && legatum_core::data::pays_upkeep(u.kind))
                .flat_map(|u| enemy_cities.iter().map(move |c| u.pos.distance(*c)))
                .min();
            let army = match dist {
                None => "no_army",
                Some(d) if d <= 3 => "near",
                Some(d) if d <= 8 => "mid",
                Some(_) => "far",
            };
            let w = log.open.get_mut(&k).expect("open war");
            let sd = &mut w.sides[side];
            *sd.turns.entry(format!("{label}:{army}")).or_default() += 1;
            if let Some(d) = dist {
                sd.min_dist = Some(sd.min_dist.map_or(d, |old: i32| old.min(d)));
            }
            if let Some(m) = &m {
                let from = seen.get(&p.0).copied().unwrap_or(0);
                for e in m.campaigns.events.iter().skip(from) {
                    let owner_team = state
                        .cities
                        .get(&e.target)
                        .filter(|c| !c.owner.is_barbarian())
                        .map(|c| state.player(c.owner).team);
                    if owner_team != Some(enemy) {
                        continue;
                    }
                    match e.end {
                        None => sd.campaign_starts += 1,
                        Some(end) => {
                            let name = serde_json::to_value(end)
                                .ok()
                                .and_then(|v| v.as_str().map(str::to_string))
                                .unwrap_or_default();
                            *sd.campaign_ends.entry(name).or_default() += 1;
                        }
                    }
                }
            }
        }
    }
    for (p, m) in memories {
        if let Some(m) = m {
            seen.insert(p, m.campaigns.events.len());
        }
    }
}

fn war_log_json(log: &WarLog) -> Value {
    log.done
        .iter()
        .map(|w| {
            json!({
                "a": w.a, "b": w.b, "start": w.start, "end": w.end,
                "declarers": w.declarers, "battles": w.battles, "captures": w.captures,
                "attacks": w.attacks,
                "sides": w.sides.iter().map(|s| json!({
                    "turns": s.turns, "campaign_starts": s.campaign_starts,
                    "campaign_ends": s.campaign_ends, "min_dist": s.min_dist,
                })).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn strategic_standings(state: &legatum_core::GameState) -> Value {
    use legatum_core::{Resource, UnitKind};
    state
        .players
        .iter()
        .filter(|p| p.alive && !p.id.is_barbarian())
        .map(|p| {
            let count = |kinds: &[UnitKind]| {
                state.units.values().filter(|u| u.owner == p.id && kinds.contains(&u.kind)).count()
            };
            json!({
                "p": p.id.0, "f": p.faction,
                "iron": legatum_core::resources::supplied(state, p.id, Resource::Iron),
                "iron_used": legatum_core::resources::in_use(state, p.id, Resource::Iron),
                "coal": legatum_core::resources::supplied(state, p.id, Resource::Coal),
                "coal_used": legatum_core::resources::in_use(state, p.id, Resource::Coal),
                "troopers": count(&[UnitKind::Trooper]),
                "siege_units": state.units.values().filter(|u| u.owner == p.id && u.siege).count(),
                "military": state.units.values().filter(|u| u.owner == p.id && class_key(u.kind).is_some()).count(),
            })
        })
        .collect()
}

fn standings(state: &legatum_core::GameState) -> Value {
    use legatum_core::VpSource;
    state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            let (mut objectives, mut goal, mut other, mut capitals) = (0, 0, 0, 0);
            for (source, vp) in scoring::vp(state, p.id) {
                match source {
                    VpSource::EraObjective { .. } => objectives += vp,
                    VpSource::FactionGoal { .. } => goal += vp,
                    VpSource::GreatWork { .. } => other += vp,
                    VpSource::CapitalTaken { .. } | VpSource::CapitalSacked { .. } => {
                        capitals += vp
                    }
                }
            }
            json!({
                "p": p.id.0, "f": p.faction, "vp": scoring::vp_total(state, p.id),
                "obj": objectives, "goal": goal, "gw": other, "capitals": capitals,
                "era": legatum_core::eras::current_era(&p.techs) + 1,
                "cities": state.cities_of(p.id).count(),
                "army": scoring::army_strength(state, p.id),
                "econ": scoring::economy_value(state, p.id),
            })
        })
        .collect()
}

fn class_key(kind: legatum_core::UnitKind) -> Option<&'static str> {
    use legatum_core::UnitClass;
    match legatum_core::data::unit_class(kind)? {
        UnitClass::Melee => Some("melee"),
        UnitClass::Ranged => Some("ranged"),
        UnitClass::Mounted => Some("mounted"),
        UnitClass::Air => Some("air"),
    }
}

fn note_diag(metrics: &mut Metrics, seat: PlayerId, turn: u32, event: &Event) {
    let d = &mut metrics.diag;
    match event {
        Event::ProductionCompleted {
            owner,
            item: legatum_core::ProductionItem::Unit { unit },
            ..
        } if *owner == seat => {
            if *unit == legatum_core::UnitKind::Settler {
                *d.entry("settlers").or_default() += 1;
                d.entry("first_settler_turn").or_insert(i64::from(turn));
            } else if let Some(class) = class_key(*unit) {
                let cost = i64::from(legatum_core::data::unit_stats(*unit).cost);
                let (n, prod) = match class {
                    "melee" => ("built_melee", "prod_melee"),
                    "ranged" => ("built_ranged", "prod_ranged"),
                    "mounted" => ("built_mounted", "prod_mounted"),
                    _ => ("built_air", "prod_air"),
                };
                *d.entry(n).or_default() += 1;
                *d.entry(prod).or_default() += cost;
            }
        }
        Event::ItemBought { owner, gold, .. } if *owner == seat => {
            *d.entry("gold_bought").or_default() += i64::from(*gold);
        }
        Event::UnitDisbanded { owner, reason, .. } if *owner == seat => {
            *d.entry(if reason.contains("Treasury ran dry") {
                "disband_upkeep"
            } else {
                "disband_other"
            })
            .or_default() += 1;
        }
        Event::UnitDestroyed { owner, .. } if *owner == seat => {
            *d.entry("units_lost").or_default() += 1;
        }
        // Every empire, not only the seat: a Peace followed within 5 or 10 turns by a declaration
        // from either side (`peace_flip_*`), and those aimed at the seat (`_on_seat`). The playtest
        // 8.9.2 Mongols sued the VP leader for peace and declared again four turns later.
        Event::PeaceEstablished { relation } => {
            *d.entry("peace_signed").or_default() += 1;
            metrics
                .peace_turn
                .insert((relation.a.0, relation.b.0), relation.effective_turn);
        }
        Event::WarDeclared { relation } => {
            if let Some(&signed) = metrics.peace_turn.get(&(relation.a.0, relation.b.0)) {
                let age = relation.effective_turn.saturating_sub(signed);
                let on_seat = relation.a.0 == seat.0 || relation.b.0 == seat.0;
                for (limit, key, key_seat) in [
                    (5, "peace_flip_5", "peace_flip_5_on_seat"),
                    (10, "peace_flip_10", "peace_flip_10_on_seat"),
                ] {
                    if age <= limit {
                        *d.entry(key).or_default() += 1;
                        if on_seat {
                            *d.entry(key_seat).or_default() += 1;
                        }
                    }
                }
            }
        }
        Event::CityCaptured { from, to, .. } => {
            if *to == seat {
                *d.entry("cities_captured").or_default() += 1;
            } else if *from == seat {
                *d.entry("cities_lost").or_default() += 1;
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn play(
    map: &MapRecord,
    brain: &str,
    params: Option<&AiParams>,
    mode: SeatMode,
    salt: u64,
    until_turn: Option<u32>,
    include_cities_t100: bool,
    rules: &[String],
    seat_ai_rng_offset: u64,
    rival_search: bool,
    other_params: Option<&AiParams>,
) -> Value {
    let trace_spec = std::env::var("LEGATUM_HVH_TRACE").ok();
    let trace_seat = trace_spec.as_deref().and_then(|wanted| {
        let (prefix, selected) = wanted.rsplit_once(':')?;
        (prefix == format!("{}:{salt}", map.id))
            .then(|| selected.parse::<u8>().ok())
            .flatten()
    });
    let trace =
        trace_spec.as_deref() == Some(&format!("{}:{salt}", map.id)) || trace_seat.is_some();
    let mut setup = map.setup.clone();
    for flag in rules {
        setup
            .rule_experiments
            .apply(flag)
            .expect("valid --rules experiment");
    }
    let seat_difficulty = match brain {
        "easy" => AiDifficulty::Easy,
        "normal" => AiDifficulty::Normal,
        _ => AiDifficulty::Hard,
    };
    setup.players[map.human_seat as usize].is_human = false;
    setup.players[map.human_seat as usize].difficulty = seat_difficulty;
    let mut state = legatum_core::new_game(&setup);
    // `LEGATUM_HVH_RESEED_DICE=1`: every salt also plays its own combat, movement and barbarian
    // dice (as the war suite's `reseed` does). Otherwise all salts of a map share one set of
    // dice, so a map's win rate is dominated by how that one set treats the seat's opening and
    // a change in behaviour can flip a whole map (see docs/learnings.md, 2026-10-18).
    if std::env::var("LEGATUM_HVH_RESEED_DICE").is_ok_and(|v| v == "1") {
        state.setup.seed = salted(state.setup.seed, salt ^ 0xD1CE);
    }
    let seat = PlayerId(map.human_seat);
    // Seats the arm's mode leaves out play `other_params` (an explicit override, so an arm can
    // switch a default-on gene off in the seat), else the built-in parameters.
    let baseline = other_params.cloned().unwrap_or_else(AiParams::builtin);
    let mut brains: Vec<Box<dyn Brain>> = setup
        .players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let id = PlayerId(i as u8);
            let seed = player_ai_seed(
                legatum_wasm::session::brain_seed(map.setup.seed, id),
                salt,
                id,
                seat,
                seat_ai_rng_offset,
            );
            let difficulty = if id == seat {
                seat_difficulty
            } else {
                player.difficulty
            };
            let scripted = match params {
                Some(candidate) => ScriptedBrain::with_params(
                    player.faction,
                    seed,
                    id,
                    difficulty,
                    if mode.candidate(id == seat) {
                        candidate
                    } else {
                        &baseline
                    },
                ),
                None => ScriptedBrain::with_difficulty(player.faction, seed, id, difficulty),
            };
            let brain: Box<dyn Brain> = if (id == seat
                && (brain == "hard+search" || brain == "params+search"))
                || (id != seat && rival_search && player.search)
            {
                Box::new(legatum_ai::search::SearchBrain::new(
                    scripted,
                    legatum_ai::search::SearchConfig::in_app(setup.players.len()),
                ))
            } else {
                Box::new(scripted)
            };
            brain
        })
        .collect();
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut metrics = Metrics::default();
    let war_diag = std::env::var_os("LEGATUM_HVH_WAR_DIAG").is_some();
    let war_trace = std::env::var("LEGATUM_HVH_WAR_TRACE").ok();
    let mut seen_campaigns = BTreeMap::new();
    let mut quality_seen = BTreeMap::<u8, usize>::new();
    let mut campaign_starts = BTreeMap::<(u8, legatum_core::CityId), u32>::new();
    let mut recent_losses = BTreeMap::<(u8, legatum_core::CityId), u32>::new();
    while !state.finished && state.turn < cap && until_turn.is_none_or(|limit| state.turn <= limit)
    {
        let turn = state.turn;
        if turn >= 300
            && turn.is_multiple_of(100)
            && let Some(ids) = &war_trace
        {
            for id in ids.split(',').filter_map(|s| s.parse::<u8>().ok()) {
                let player = PlayerId(id);
                if !state.player(player).alive {
                    continue;
                }
                let view = legatum_core::player_view(&state, player);
                eprintln!(
                    "WAR_TRACE seed={salt} turn={turn} seat={id} vp={} cities={} army={}",
                    scoring::vp_total(&state, player),
                    state.cities_of(player).count(),
                    scoring::army_strength(&state, player)
                );
                for line in brains[id as usize].war_diagnosis(&view) {
                    eprintln!("WAR_TRACE {line}");
                }
            }
        }
        if trace && turn > 90 {
            break;
        }
        let orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let view = legatum_core::player_view(&state, p.id);
                if !p.id.is_barbarian() && !view.diplomacy.met.is_empty() {
                    metrics.first_contact_by_ai.entry(p.id.0).or_insert(turn);
                }
                (p.id, brains[p.id.0 as usize].orders(&view))
            })
            .collect();
        // Full-state replay census, sampled before resolution. These counters are diagnostic
        // only; the brains still receive their ordinary fogged PlayerView above.
        for player in state
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian())
        {
            let metric = metrics.air.entry(player.id.0).or_default();
            let oil = legatum_core::resources::Resource::Oil;
            for plane in &state.aircraft {
                if state
                    .cities
                    .get(&plane.city)
                    .is_some_and(|c| c.owner == player.id)
                {
                    metric.aircraft_turns += 1;
                    metric.unsupplied_aircraft_turns += u32::from(player.supply.iter().any(|s| {
                        s.user == legatum_core::resources::SupplyUser::Unit { unit: plane.id }
                            && s.idle
                    }));
                }
            }
            let deposits = legatum_core::resources::deposits(&state, player.id);
            metric.oil_deposit_turns +=
                deposits.iter().filter(|d| d.resource == oil).count() as u32;
            metric.unimproved_oil_deposit_turns += deposits
                .iter()
                .filter(|d| d.resource == oil && !d.mined)
                .count() as u32;
            let spare = legatum_core::resources::spare(&state, player.id, oil, None);
            metric.oil_shortage_turns += u32::from(
                legatum_core::resources::in_use(&state, player.id, oil)
                    > legatum_core::resources::supplied(&state, player.id, oil),
            );
            if spare <= 0 {
                let view = legatum_core::player_view(&state, player.id);
                metric.oil_blocked_upgrade_turns += view
                    .units
                    .iter()
                    .filter(|u| u.owner == player.id && u.upgrade_need == Some(oil))
                    .count() as u32;
                metric.oil_blocked_tank_upgrade_turns += view
                    .units
                    .iter()
                    .filter(|u| {
                        u.owner == player.id
                            && u.upgrade_need == Some(oil)
                            && u.upgrade_to == Some(legatum_core::UnitKind::Tank)
                    })
                    .count() as u32;
            }
            let view = legatum_core::player_view(&state, player.id);
            let waiting_upgrades = view
                .units
                .iter()
                .any(|u| u.owner == player.id && u.upgrade_need == Some(oil));
            let waiting_tanks = view.units.iter().any(|u| {
                u.owner == player.id
                    && u.upgrade_need == Some(oil)
                    && u.upgrade_to == Some(legatum_core::UnitKind::Tank)
            });
            let waiting_labs = view.cities.iter().any(|c| {
                c.owner == player.id
                    && c.detail.as_ref().is_some_and(|d| {
                        d.options.iter().any(|o| {
                            o.item
                                == legatum_core::ProductionItem::Building {
                                    building: legatum_core::BuildingKind::InferenceCore,
                                }
                                && o.supply_need == Some(oil)
                        })
                    })
            });
            if let Some(turn_orders) = orders.get(&player.id) {
                for order in turn_orders {
                    let item = match order {
                        legatum_core::Order::SetProduction { item, .. }
                        | legatum_core::Order::BuyItem { item, .. } => Some(*item),
                        _ => None,
                    };
                    if item.is_some_and(|item| {
                        matches!(item,
                        legatum_core::ProductionItem::Unit { unit }
                        if legatum_core::data::is_aircraft(unit))
                    }) {
                        metric.oil_air_starts_with_waiting_upgrades += u32::from(waiting_upgrades);
                        metric.oil_air_starts_with_waiting_tank_upgrades +=
                            u32::from(waiting_tanks);
                        metric.oil_air_starts_with_waiting_labs += u32::from(waiting_labs);
                    }
                }
            }
            if player
                .techs
                .contains(&legatum_core::TechId::Industrialization)
            {
                for (index, tile) in state.map.tiles.iter().enumerate() {
                    if tile.resource != Some(oil) {
                        continue;
                    }
                    let at = state.map.hex_at(index);
                    metric.oil_nearby_unclaimed_deposit_turns += u32::from(
                        tile.owner.is_none()
                            && state.cities_of(player.id).any(|c| {
                                c.pos.distance(at) <= legatum_core::data::CITY_WORK_RADIUS
                            }),
                    );
                    metric.oil_enemy_deposit_turns +=
                        u32::from(tile.owner.is_some_and(|owner| {
                            owner != player.id && state.at_war(player.id, owner)
                        }));
                    if tile.owner == Some(player.id) {
                        let threatened = state.units.values().any(|u| {
                            state.at_war(player.id, u.owner)
                                && legatum_core::data::unit_stats(u.kind).military
                                && u.pos.distance(at) <= 3
                        });
                        if threatened {
                            metric.oil_threatened_deposit_turns += 1;
                            metric.oil_undefended_threatened_deposit_turns +=
                                u32::from(!state.units.values().any(|u| {
                                    u.owner == player.id
                                        && legatum_core::data::unit_stats(u.kind).military
                                        && u.pos.distance(at) <= 3
                                }));
                        }
                    }
                }
            }
            metric.oil_air_build_turns += state
                .cities_of(player.id)
                .filter(|c| matches!(c.producing, Some(legatum_core::ProductionItem::Unit { unit }) if legatum_core::data::is_aircraft(unit)))
                .count() as u32;
        }
        for player in state.players.iter().filter(|p| !p.id.is_barbarian()) {
            let id = player.id.0;
            let events = brains[id as usize].campaign_events();
            let from = quality_seen.insert(id, events.len()).unwrap_or(0);
            for event in events.iter().skip(from) {
                let key = (id, event.target);
                if event.end.is_none() {
                    campaign_starts.insert(key, event.turn);
                } else if let Some(start) = campaign_starts.remove(&key)
                    && event.turn.saturating_sub(start) <= 3
                {
                    metrics.campaign_quality.entry(id).or_default().flaps += 1;
                }
            }
        }
        if trace && turn <= 90 {
            let observed = PlayerId(trace_seat.unwrap_or(seat.0));
            let memory = brains[observed.0 as usize].opponent_memory();
            let observed_view = legatum_core::player_view(&state, observed);
            let known_cities = &observed_view.cities;
            eprintln!(
                "{}",
                json!({
                    "turn": turn,
                    "seat": observed,
                    "gold": state.player(observed).gold,
                    "gpt": observed_view.economy.gold_per_turn.total,
                    "upkeep": observed_view.economy.upkeep_breakdown.parts,
                    "decadence": state.player(observed).decadence_percent,
                    "era": legatum_core::eras::current_era(&state.player(observed).techs),
                    "techs": state.player(observed).techs.len(),
                    "researching": state.player(observed).researching,
                    "cities": state.cities.values().map(|c| json!({"id":c.id,"owner":c.owner,"pos":c.pos,"hp":c.hp,"population":c.population,"walls":c.buildings.contains(&legatum_core::BuildingKind::Walls),"producing":c.producing,"progress":c.production_stored})).collect::<Vec<_>>(),
                    "known_cities": known_cities.iter().map(|c| json!({"id":c.id,"visible":c.visible,"walls":c.walls,"hp":c.hp})).collect::<Vec<_>>(),
                    "wars": observed_view.players.iter().filter(|p| p.team != observed_view.team && observed_view.teams_at_war(observed_view.team, p.team)).map(|p| p.team).collect::<Vec<_>>(),
                    "legal_declarations": observed_view.diplomacy.actions.iter().filter_map(|a| match a.order {legatum_core::Order::DeclareWar { against, breach: false } if a.blocked_reason.is_none() => Some(against), _ => None}).collect::<Vec<_>>(),
                    "visible_enemy_units": observed_view.units.iter().filter(|u| u.owner != observed).map(|u| json!({"id":u.id,"owner":u.owner,"kind":u.kind,"pos":u.pos,"hp":u.hp})).collect::<Vec<_>>(),
                    "units": state.units.values().filter(|u| u.owner == observed).map(|u| json!({"id":u.id,"kind":u.kind,"pos":u.pos,"hp":u.hp,"xp":u.xp,"created":u.created_turn})).collect::<Vec<_>>(),
                    "orders": orders.get(&observed),
                    "plan": memory.as_ref().and_then(|m| m.campaigns.plan.active.as_ref()),
                    "campaign": memory.as_ref().and_then(|m| m.campaigns.active.as_ref()),
                    "plan_event": brains[observed.0 as usize].plan_events().last(),
                    "campaign_event": memory.as_ref().and_then(|m| m.campaigns.events.last()),
                })
            );
        }
        if war_diag {
            sample_wars(&state, &brains, &mut metrics.wars, &mut seen_campaigns);
        }
        // The air duel is reported as a Battle, with temporary map units. Capture hangar
        // identities before resolution so destroyed aircraft remain attributable afterward.
        let aircraft: BTreeMap<_, _> = state
            .aircraft
            .iter()
            .filter_map(|a| state.cities.get(&a.city).map(|c| (a.id, (c.owner, a.kind))))
            .collect();
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        for (old, new) in state.map.tiles.iter().zip(&next.map.tiles) {
            if old.resource != Some(legatum_core::resources::Resource::Oil) {
                continue;
            }
            if old.owner != new.owner {
                if let Some(owner) = old.owner.filter(|p| !p.is_barbarian()) {
                    metrics.air.entry(owner.0).or_default().oil_losses += 1;
                }
                if let Some(owner) = new.owner.filter(|p| !p.is_barbarian()) {
                    let metric = metrics.air.entry(owner.0).or_default();
                    if old.owner.is_none() {
                        metric.oil_claims += 1;
                    } else {
                        metric.oil_takeovers += 1;
                    }
                }
            }
            if !old.improved
                && new.improved
                && let Some(owner) = new.owner.filter(|p| !p.is_barbarian())
            {
                metrics.air.entry(owner.0).or_default().oil_improvements += 1;
            }
        }
        let sparta = state
            .players
            .iter()
            .find(|p| p.faction == Faction::Sparta)
            .map(|p| (p.id, p.team));
        if let Some((sparta_id, sparta_team)) = sparta {
            for event in &events {
                match event {
                    Event::WarDeclared { relation } | Event::WarJoined { relation, .. }
                        if relation.a == sparta_team || relation.b == sparta_team =>
                    {
                        metrics.sparta.first_war.get_or_insert(turn);
                        if relation.declarers.contains(&sparta_team) {
                            metrics.sparta.first_declared_war.get_or_insert(turn);
                        }
                    }
                    Event::CityCaptured { city, to, .. } if *to == sparta_id => {
                        metrics.sparta.captures += 1;
                        if state
                            .cities
                            .get(city)
                            .is_some_and(|c| legatum_core::data::wall_tier(&c.buildings) > 0)
                        {
                            metrics.sparta.walled_captures += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        if trace && turn <= 90 {
            eprintln!("{}", json!({"turn":turn,"events":events}));
        }
        state = next;
        for event in &events {
            match event {
                Event::SettlerCaptured { from, to, .. } => {
                    metrics.expansion.entry(from.0).or_default().settlers_lost += 1;
                    metrics.expansion.entry(to.0).or_default().settlers_taken += 1;
                }
                Event::UnitDestroyed {
                    owner,
                    unit_kind: legatum_core::UnitKind::Settler,
                    ..
                } => metrics.expansion.entry(owner.0).or_default().settlers_lost += 1,
                _ => {}
            }
        }
        if matches!(state.turn, 50 | 81 | 100) {
            for player in state.players.iter().filter(|p| !p.id.is_barbarian()) {
                let cities = Some(state.cities_of(player.id).count() as u32);
                let entry = metrics.expansion.entry(player.id.0).or_default();
                match state.turn {
                    50 => entry.cities_t50 = cities,
                    81 => entry.cities_t81 = cities,
                    _ => entry.cities_t100 = cities,
                }
            }
        }
        for event in &events {
            note_diag(&mut metrics, seat, turn, event);
            if let Event::CityCaptured { city, from, to, .. } = event {
                if !from.is_barbarian() {
                    recent_losses.insert((from.0, *city), turn);
                    metrics
                        .campaign_quality
                        .entry(from.0)
                        .or_default()
                        .retake_opportunities += 1;
                }
                if !to.is_barbarian()
                    && let Some(lost) = recent_losses.remove(&(to.0, *city))
                    && turn.saturating_sub(lost) <= 20
                {
                    metrics
                        .campaign_quality
                        .entry(to.0)
                        .or_default()
                        .retake_successes += 1;
                }
            }
            match event {
                Event::RuinExplored { owner, .. } => {
                    *metrics.ruins_by_ai.entry(owner.0).or_default() += 1;
                }
                Event::ProductionCompleted {
                    owner,
                    item:
                        legatum_core::ProductionItem::Unit {
                            unit: legatum_core::UnitKind::Fighter,
                        },
                    ..
                } => metrics.air.entry(owner.0).or_default().fighters_built += 1,
                Event::Battle {
                    attacker: legatum_core::events::Attacker::Unit(attacker),
                    defender: legatum_core::events::Defender::Unit { unit: defender, .. },
                    ranged: false,
                    ..
                } if aircraft
                    .get(attacker)
                    .is_some_and(|(_, kind)| *kind == legatum_core::UnitKind::Bomber)
                    && aircraft
                        .get(defender)
                        .is_some_and(|(_, kind)| *kind == legatum_core::UnitKind::Fighter) =>
                {
                    let owner = aircraft[defender].0;
                    metrics
                        .air
                        .entry(owner.0)
                        .or_default()
                        .bomber_strikes_intercepted += 1;
                }
                _ => {}
            }
            if let Event::SiegeBought { owner, .. } = event {
                metrics.strategic_built.entry(owner.0).or_default()[1] += 1;
            }
            if let Event::ProductionCompleted {
                owner,
                item: legatum_core::ProductionItem::Unit { unit },
                ..
            } = event
            {
                let row = metrics.strategic_built.entry(owner.0).or_default();
                if *unit == legatum_core::UnitKind::Trooper {
                    row[0] += 1;
                }
            }
            note_war(&state, &mut metrics.wars, turn, event);
        }
        for event in events {
            if let Event::GameWon { team, reason } = event
                && team == state.players[map.human_seat as usize].team
            {
                metrics.win_turn = Some(turn);
                metrics.win_type = Some(if reason.contains("held") || reason.contains("VP") {
                    "vp_hold".into()
                } else {
                    "elimination".into()
                });
            }
        }
        if matches!(turn, 60 | 100 | 150 | 200 | 250) {
            metrics.standings.insert(turn, standings(&state));
            metrics.strategic.insert(turn, strategic_standings(&state));
        }
        if CHECKPOINTS.contains(&turn) {
            metrics.snapshots.insert(
                turn,
                (
                    state.cities_of(seat).count() as u32,
                    scoring::vp_total(&state, seat),
                ),
            );
        }
        if until_turn == Some(turn) {
            let values = state
                .teams()
                .into_iter()
                .filter(|team| {
                    state
                        .team_members(*team)
                        .iter()
                        .any(|player| state.player(*player).alive)
                })
                .map(|team| (team, scoring::team_vp(&state, team)))
                .collect::<Vec<_>>();
            let highest = values.iter().map(|(_, vp)| *vp).max();
            let leaders = values
                .iter()
                .filter(|(_, vp)| Some(*vp) == highest)
                .map(|(team, _)| team.0)
                .collect::<Vec<_>>();
            let capital_held = state
                .capital_of(seat)
                .is_some_and(|capital| capital.owner == seat);
            metrics.until = Some(json!({
                "turn": turn,
                "cities": state.cities_of(seat).count(),
                "capital_held": capital_held,
                "vp": scoring::vp_total(&state, seat),
                "public_vp_leader": {"teams": leaders, "vp": highest},
            }));
        }
    }
    let seat_team = state.player(seat).team.0;
    for player in state.players.iter().filter(|p| !p.id.is_barbarian()) {
        metrics.air.entry(player.id.0).or_default().flight =
            player.techs.contains(&legatum_core::TechId::Flight);
    }
    close_wars(&mut metrics.wars, &mut metrics.diag, state.turn, seat_team);
    let at = |turn: u32| metrics.snapshots.get(&turn).copied();
    let mut result = json!({
        "win": state.winner == Some(state.players[map.human_seat as usize].team),
        "win_turn": metrics.win_turn,
        "win_type": metrics.win_type,
        "cities_t30": at(30).map(|v| v.0),
        "cities_t50": at(50).map(|v| v.0),
        "cities_t62": at(62).map(|v| v.0),
        "vp_t60": at(60).map(|v| v.1),
        "vp_t100": at(100).map(|v| v.1),
        "cities_t100": at(100).map(|v| v.0),
        "end_turn": state.turn,
        "diag": metrics.diag,
        "sparta": metrics.sparta,
        "campaign_quality": metrics.campaign_quality,
        "standings": metrics.standings,
        "strategic": metrics.strategic,
        "strategic_built": metrics.strategic_built,
        "air": metrics.air,
        "ruins_by_ai": metrics.ruins_by_ai,
        "first_contact_by_ai": metrics.first_contact_by_ai,
        "expansion": metrics.expansion,
        "final": standings(&state),
        "winner_team": state.winner.map(|t| t.0),
    });
    if war_diag {
        result["war_log"] = war_log_json(&metrics.wars);
    }
    if let Some(limit) = until_turn {
        let object = result.as_object_mut().expect("game result object");
        object.remove("win");
        object.remove("win_turn");
        object.remove("win_type");
        object.insert("until".into(), metrics.until.unwrap_or(Value::Null));
        object.insert("until_turn".into(), json!(limit));
        object.insert("cities_t100".into(), json!(at(100).map(|v| v.0)));
    } else if include_cities_t100 {
        result["cities_t100"] = json!(at(100).map(|v| v.0));
    }
    result
}

fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(if values.len() % 2 == 1 {
        values[values.len() / 2]
    } else {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    })
}

/// AI median over human milestone for each checkpoint the human reached. A checkpoint the AI never
/// reached counts as zero.
fn milestone_ratios(
    human: &HumanMilestones,
    medians: &BTreeMap<&'static str, Option<f64>>,
) -> BTreeMap<&'static str, f64> {
    let ratio = |key: &str, human: u32| medians[key].unwrap_or(0.0) / human.max(1) as f64;
    let mut ratios = BTreeMap::from([
        ("cities_t30", ratio("cities_t30", human.cities_t30)),
        ("cities_t50", ratio("cities_t50", human.cities_t50)),
        ("vp_t60", ratio("vp_t60", human.vp_t60)),
    ]);
    if let Some(vp) = human.vp_t100 {
        ratios.insert("vp_t100", ratio("vp_t100", vp));
    }
    ratios
}

fn medians_of(games: &[Value]) -> BTreeMap<&'static str, Option<f64>> {
    [
        "cities_t30",
        "cities_t50",
        "cities_t62",
        "cities_t100",
        "vp_t60",
        "vp_t100",
    ]
    .into_iter()
    .map(|key| {
        let mut values = games
            .iter()
            .filter_map(|g| g[key].as_f64())
            .collect::<Vec<_>>();
        (key, median(&mut values))
    })
    .collect()
}

/// 95% intervals for the overall score and the pooled milestone ratio, from resampling each map's
/// games with replacement. Wins are the mean of the per-map win rates, as in the headline.
fn bootstrap(maps: &[MapRecord], games: &[Vec<Value>]) -> (Option<[f64; 2]>, Option<[f64; 2]>) {
    const RESAMPLES: usize = 400;
    let mut rng = Rng::derive(0xb007, &[purpose::AI, 0xc1]);
    let (mut scores, mut ratios) = (Vec::new(), Vec::new());
    for _ in 0..RESAMPLES {
        let (mut score, mut pooled) = (0.0, Vec::new());
        for (map, map_games) in maps.iter().zip(games) {
            let sample = (0..map_games.len())
                .map(|_| map_games[(rng.next_u64() % map_games.len() as u64) as usize].clone())
                .collect::<Vec<_>>();
            score += sample.iter().filter(|g| g["win"] == true).count() as f64
                / sample.len() as f64
                / maps.len() as f64;
            pooled.extend(milestone_ratios(&map.human, &medians_of(&sample)).into_values());
        }
        scores.push(score);
        ratios.push(median(&mut pooled).unwrap_or(0.0));
    }
    let interval = |values: &mut Vec<f64>| {
        values.sort_by(f64::total_cmp);
        Some([
            values[RESAMPLES * 25 / 1000],
            values[RESAMPLES * 975 / 1000],
        ])
    };
    (interval(&mut scores), interval(&mut ratios))
}

fn summarize(map: &MapRecord, games: &[Value]) -> Value {
    let n = games.len() as f64;
    let wins = games.iter().filter(|g| g["win"] == true).count();
    let mut turns = games
        .iter()
        .filter_map(|g| g["win_turn"].as_f64())
        .collect::<Vec<_>>();
    let medians = medians_of(games);
    let milestones = json!({
        "cities_t30": medians["cities_t30"], "cities_t50": medians["cities_t50"],
        "cities_t62": medians["cities_t62"],
        "vp_t60": medians["vp_t60"], "vp_t100": medians["vp_t100"],
        "win_turn": median(&mut turns),
    });
    let human = &map.human;
    let ratios = milestone_ratios(human, &medians);
    // Speed of the win: the human's win turn over the AI's median win turn (1.0 = as fast as the
    // human, below 1.0 = slower). Absent when the AI never won. Reported beside the milestone
    // ratio, not folded into it.
    let win_turn_ratio = human
        .win_turn
        .and_then(|turn| median(&mut turns).map(|t| turn as f64 / t));
    let mut map_ratios = ratios.values().copied().collect::<Vec<_>>();
    let milestone_ratio = median(&mut map_ratios).unwrap_or(0.0);
    json!({
        "id": map.id, "n": games.len(), "wins": wins,
        "win_rate": if n > 0.0 { wins as f64 / n } else { 0.0 },
        "milestones": milestones, "human": human,
        "recorded_winner": map.recorded_winner,
        "milestone_ratio": milestone_ratio,
        "milestone_ratios": ratios,
        "win_turn_ratio": win_turn_ratio,
        "win_types": {
            "vp_hold": games.iter().filter(|g| g["win_type"] == "vp_hold").count(),
            "elimination": games.iter().filter(|g| g["win_type"] == "elimination").count(),
        }
    })
}

fn summarize_until(map: &MapRecord, games: &[Value], turn: u32) -> Value {
    let metric_median = |key: &str| {
        let mut values = games
            .iter()
            .filter_map(|game| {
                game["until"][key]
                    .as_f64()
                    .or_else(|| game["until"][key].as_bool().map(u8::from).map(f64::from))
            })
            .collect::<Vec<_>>();
        median(&mut values)
    };
    let mut leader_vps = games
        .iter()
        .filter_map(|game| game["until"]["public_vp_leader"]["vp"].as_f64())
        .collect::<Vec<_>>();
    let mut leader_counts = BTreeMap::<String, u32>::new();
    for game in games {
        if let Some(teams) = game["until"]["public_vp_leader"]["teams"].as_array() {
            for team in teams {
                *leader_counts.entry(team.to_string()).or_default() += 1;
            }
        }
    }
    json!({
        "id": map.id,
        "n": games.len(),
        "milestones": {
            "cities_t30": if turn >= 30 { medians_of(games)["cities_t30"] } else { None },
            "cities_t50": if turn >= 50 { medians_of(games)["cities_t50"] } else { None },
            "cities_t100": if turn >= 100 { medians_of(games)["cities_t100"] } else { None },
            "capital_held_tN": metric_median("capital_held"),
            "vp_tN": metric_median("vp"),
            "public_vp_leader_vp_tN": median(&mut leader_vps),
            "public_vp_leader_seat_counts_tN": leader_counts,
        }
    })
}

fn run(args: &[String]) {
    let mut brain = "hard".to_string();
    let mut salts = 64u64;
    let mut salt_start = 1u64;
    let mut seat_ai_rng_offset = 0u64;
    let mut params_path: Option<String> = None;
    let mut mode = SeatMode::All;
    let mut mode_selected = false;
    let mut out_dir: Option<String> = None;
    let mut selected_map: Option<String> = None;
    let mut rules_flags: Vec<String> = Vec::new();
    let mut until_turn: Option<u32> = None;
    let mut include_cities_t100 = false;
    let mut rules: Vec<String> = Vec::new();
    let mut arms_mode = false;
    let mut rival_search = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--brain" => {
                brain = args[i + 1].clone();
                i += 2;
            }
            "--salts" | "--games" => {
                salts = args[i + 1].parse().expect("integer salts");
                i += 2;
            }
            "--salt-start" | "--seed" => {
                salt_start = args[i + 1].parse().expect("integer salt start");
                i += 2;
            }
            "--seat-ai-rng-offset" => {
                seat_ai_rng_offset = args[i + 1].parse().expect("integer seat AI RNG offset");
                i += 2;
            }
            "--ai-params" => {
                params_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--seat-brain" => {
                assert!(!mode_selected, "seat modes are mutually exclusive");
                mode = SeatMode::Seat;
                mode_selected = true;
                i += 1;
            }
            "--opponent-brain" => {
                assert!(!mode_selected, "seat modes are mutually exclusive");
                mode = SeatMode::Opponents;
                mode_selected = true;
                i += 1;
            }
            "--all-seats" => {
                assert!(!mode_selected, "seat modes are mutually exclusive");
                mode_selected = true;
                i += 1;
            }
            "--map" => {
                selected_map = Some(args[i + 1].clone());
                i += 2;
            }
            // Rules experiments applied to every map's setup (comma separated).
            "--rules-experiment" => {
                rules_flags.extend(args[i + 1].split(',').map(str::to_string));
                i += 2;
            }
            "--out-dir" => {
                out_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--until-turn" => {
                until_turn = Some(args[i + 1].parse().expect("integer turn"));
                i += 2;
            }
            "--rules" => {
                rules.push(args[i + 1].clone());
                i += 2;
            }
            // The --ai-params file is an arms list: several brains, params, seat modes and rule
            // experiments in one flat parallel pass (one cloud batch for a whole experiment).
            "--arms" => {
                arms_mode = true;
                i += 1;
            }
            "--rival-search" => {
                rival_search = true;
                i += 1;
            }
            "--include-cities-t100" => {
                include_cities_t100 = true;
                i += 1;
            }
            "--label" | "--out" | "--status-file" => {
                i += 2;
            }
            "--" => i += 1,
            other => panic!("unknown argument {other}"),
        }
    }
    assert!(salts > 0);
    if arms_mode {
        run_arms(
            params_path
                .as_deref()
                .expect("--arms needs --ai-params ARMS.json"),
            selected_map,
            salt_start,
            salts,
            until_turn,
            out_dir,
        );
        return;
    }
    assert!(matches!(
        brain.as_str(),
        "hard" | "hard+search" | "params" | "params+search" | "normal" | "easy"
    ));
    let params: Option<AiParams> = params_path.as_ref().map(|path| {
        let params: AiParams =
            serde_json::from_str(&std::fs::read_to_string(path).expect("read params"))
                .expect("parse params");
        params.validate().expect("validate params");
        params
    });
    if brain == "params" || brain == "params+search" {
        assert!(params.is_some(), "parameterized brain needs --ai-params");
    }
    if mode != SeatMode::All {
        assert!(params.is_some(), "seat mode needs --ai-params");
    }
    let params = params.as_ref();
    let mut maps: Vec<MapRecord> = serde_json::from_str(MAPS).expect("checked-in maps.json");
    for map in &mut maps {
        for flag in &rules_flags {
            map.setup
                .rule_experiments
                .apply(flag)
                .expect("valid rules experiment");
        }
    }
    if let Some(id) = selected_map {
        maps.retain(|map| map.id == id);
        assert!(!maps.is_empty(), "unknown map {id}");
    }
    // One flat parallel pass over every (map, salt), so a slow game on one map does not idle cores.
    let jobs = maps
        .iter()
        .enumerate()
        .flat_map(|(m, _)| (salt_start..salt_start + salts).map(move |salt| (m, salt)))
        .collect::<Vec<_>>();
    let played: Vec<(usize, Value)> = jobs
        .into_par_iter()
        .map(|(m, salt)| {
            let game = play(
                &maps[m],
                &brain,
                params,
                mode,
                salt,
                until_turn,
                include_cities_t100,
                &rules,
                seat_ai_rng_offset,
                rival_search,
                None,
            );
            (m, game)
        })
        .collect();
    let mut all_games: Vec<Vec<Value>> = vec![Vec::new(); maps.len()];
    for (m, game) in played {
        all_games[m].push(game);
    }
    let per_map = maps
        .iter()
        .zip(&all_games)
        .map(|(map, games)| match until_turn {
            Some(turn) => summarize_until(map, games, turn),
            None => summarize(map, games),
        })
        .collect::<Vec<_>>();
    let brain_label = match brain.as_str() {
        "params" => "hard",
        "params+search" => "hard+search",
        other => other,
    };
    let mut report = if let Some(turn) = until_turn {
        json!({
            "date": "", "commit": "", "brain": brain_label,
            "candidate_params": params.map(|p| &p.name),
            "seat_mode": mode.label(), "truncated": true, "until_turn": turn,
            "salt_start": salt_start, "salts_per_map": salts, "maps": per_map,
        })
    } else {
        let (overall_ci95, milestone_ratio_ci95) = bootstrap(&maps, &all_games);
        let overall = per_map
            .iter()
            .map(|m| m["win_rate"].as_f64().unwrap())
            .sum::<f64>()
            / per_map.len() as f64;
        let mut ratios = per_map
            .iter()
            .flat_map(|m| m["milestone_ratios"].as_object().unwrap().values())
            .map(|v| v.as_f64().unwrap())
            .collect::<Vec<_>>();
        json!({
            "date": "", "commit": "", "brain": brain_label,
            "candidate_params": params.map(|p| &p.name),
            "seat_mode": mode.label(),
            "salt_start": salt_start, "salts_per_map": salts,
            "maps": per_map, "overall_score": overall, "overall_ci95": overall_ci95,
            "milestone_ratio": median(&mut ratios), "milestone_ratio_ci95": milestone_ratio_ci95,
        })
    };
    if seat_ai_rng_offset != 0 {
        report["seat_ai_rng_offset"] = json!(seat_ai_rng_offset);
    }
    report["rival_search"] = json!(rival_search);
    let output = serde_json::to_string_pretty(&report).unwrap();
    if let Some(dir) = out_dir {
        std::fs::write(Path::new(&dir).join("hard-vs-human.json"), &output)
            .expect("write cloud report");
        let games_report = if let Some(turn) = until_turn {
            json!({
                "truncated": true,
                "until_turn": turn,
                "salt_start": salt_start,
                "salts_per_map": salts,
                "maps": maps.iter().map(|m| &m.id).collect::<Vec<_>>(),
                "games": all_games,
            })
        } else {
            json!({ "maps": maps.iter().map(|m| &m.id).collect::<Vec<_>>(), "games": all_games })
        };
        std::fs::write(
            Path::new(&dir).join("hard-vs-human-games.json"),
            serde_json::to_string(&games_report).unwrap(),
        )
        .expect("write per-game results");
    }
    println!("{output}");
}

#[derive(Deserialize)]
struct ArmsFile {
    arms: Vec<ArmSpec>,
}

#[derive(Deserialize)]
struct ArmSpec {
    label: String,
    brain: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    params: Option<AiParams>,
    #[serde(default)]
    rules: Vec<String>,
    /// Rivals that carry the map's `search` flag play Hard+Search (default: plain Hard).
    #[serde(default)]
    rival_search: bool,
    /// Parameters for the seats `mode` leaves out (default: the built-in parameters). A
    /// rivals-only arm of a default-on gene sets the gene off here for the human seat.
    #[serde(default)]
    other_params: Option<AiParams>,
}

fn run_arms(
    path: &str,
    selected_map: Option<String>,
    salt_start: u64,
    salts: u64,
    until_turn: Option<u32>,
    out_dir: Option<String>,
) {
    let file: ArmsFile = serde_json::from_str(&std::fs::read_to_string(path).expect("read arms"))
        .expect("parse arms");
    for arm in &file.arms {
        for p in arm.params.iter().chain(&arm.other_params) {
            p.validate().expect("validate arm params");
        }
    }
    let modes: Vec<SeatMode> = file
        .arms
        .iter()
        .map(|arm| match arm.mode.as_deref() {
            None | Some("all") => SeatMode::All,
            Some("seat") => SeatMode::Seat,
            Some("opponents") => SeatMode::Opponents,
            Some(other) => panic!("unknown arm mode {other}"),
        })
        .collect();
    let mut maps: Vec<MapRecord> = serde_json::from_str(MAPS).expect("checked-in maps.json");
    if let Some(id) = selected_map {
        maps.retain(|map| map.id == id);
        assert!(!maps.is_empty(), "unknown map {id}");
    }
    let jobs = (0..file.arms.len())
        .flat_map(|a| {
            (0..maps.len())
                .flat_map(move |m| (salt_start..salt_start + salts).map(move |salt| (a, m, salt)))
        })
        .collect::<Vec<_>>();
    let played: Vec<(usize, usize, Value)> = jobs
        .into_par_iter()
        .map(|(a, m, salt)| {
            let arm = &file.arms[a];
            let game = play(
                &maps[m],
                &arm.brain,
                arm.params.as_ref(),
                modes[a],
                salt,
                until_turn,
                true,
                &arm.rules,
                0,
                arm.rival_search,
                arm.other_params.as_ref(),
            );
            (a, m, game)
        })
        .collect();
    let mut reports = serde_json::Map::new();
    for (a, arm) in file.arms.iter().enumerate() {
        let mut all_games: Vec<Vec<Value>> = vec![Vec::new(); maps.len()];
        for (pa, m, game) in &played {
            if *pa == a {
                all_games[*m].push(game.clone());
            }
        }
        let per_map = maps
            .iter()
            .zip(&all_games)
            .map(|(map, games)| match until_turn {
                Some(turn) => summarize_until(map, games, turn),
                None => summarize(map, games),
            })
            .collect::<Vec<_>>();
        let mut report = json!({
            "label": arm.label, "brain": arm.brain, "seat_mode": modes[a].label(),
            "candidate_params": arm.params.as_ref().map(|p| &p.name), "rules": arm.rules,
            "rival_search": arm.rival_search,
            "salt_start": salt_start, "salts_per_map": salts, "maps": per_map,
        });
        if until_turn.is_none() {
            let (overall_ci95, _) = bootstrap(&maps, &all_games);
            let overall = per_map
                .iter()
                .map(|m| m["win_rate"].as_f64().unwrap())
                .sum::<f64>()
                / per_map.len() as f64;
            report["overall_score"] = json!(overall);
            report["overall_ci95"] = json!(overall_ci95);
        }
        if let Some(dir) = &out_dir {
            std::fs::write(
                Path::new(dir).join(format!("{}-games.json", arm.label)),
                serde_json::to_string(&json!({
                    "maps": maps.iter().map(|m| &m.id).collect::<Vec<_>>(),
                    "games": all_games,
                }))
                .unwrap(),
            )
            .expect("write arm games");
        }
        reports.insert(arm.label.clone(), report);
    }
    let output = serde_json::to_string_pretty(&Value::Object(reports)).unwrap();
    if let Some(dir) = &out_dir {
        std::fs::write(Path::new(dir).join("hard-vs-human-arms.json"), &output)
            .expect("write arms report");
    }
    println!("{output}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|arg| arg == "--extract") {
        let output = &args[2];
        let maps = args[3..]
            .iter()
            .map(|path| {
                let recording = load_save(Path::new(path));
                let (human_seat, human, recorded_winner) = milestones_from_tape(&recording);
                MapRecord {
                    id: Path::new(path)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    source: Path::new(path)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    setup: recording.tape.setup,
                    human_seat,
                    human,
                    recorded_winner,
                }
            })
            .collect::<Vec<_>>();
        std::fs::write(output, serde_json::to_string_pretty(&maps).unwrap()).expect("write maps");
    } else {
        run(&args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_offset_preserves_the_historical_seat_stream() {
        for seed in [0, 1, 0x5eed, u64::MAX] {
            for salt in [1, 2, 64, u64::MAX] {
                assert_eq!(seat_ai_seed(seed, salt, 0), salted(seed, salt));
            }
        }
    }

    fn milestones(output: &Value) -> Value {
        // Compare the stable milestone fields; the live report has since gained diagnostics
        // and standings.
        json!({
            "cities_t30": output["cities_t30"],
            "cities_t50": output["cities_t50"],
            "cities_t62": output["cities_t62"],
            "vp_t100": output["vp_t100"],
            "vp_t60": output["vp_t60"],
            "win": output["win"],
            "win_turn": output["win_turn"],
            "win_type": output["win_type"],
        })
    }

    fn first_map_game(rival_search: bool) -> Value {
        let maps: Vec<MapRecord> = serde_json::from_str(MAPS).expect("checked-in maps.json");
        milestones(&play(
            &maps[0],
            "hard",
            None,
            SeatMode::All,
            1,
            None,
            false,
            &[],
            0,
            rival_search,
            None,
        ))
    }

    #[test]
    fn rival_search_is_the_old_benchmark() {
        // The former benchmark: rivals carry the map's `search: true`. Its later
        // outcome changes under the current rules. The valued Spartan expedition changes
        // the rival war on this fixed start; this batch leaves the seat at four cities by T62
        // after two-party Peace (rules version 20) and the mounted Mongol rival; with Hard's
        // `site_denial` (2026-11-07) the seat holds 1 VP at T100; balanced founding sites
        // (rules version 24) regenerate this map and leave it with four cities at T50 and none at T100;
        // the pre-war coalition bloc (runaway_coalition 55, 2026-11-08) leaves three at T50 and T62; valued city defence (2026-11-08) keeps four at T50 and T62; relief chaining into an assaulted city (rules version 27) leaves three at T50 and T62; per-hit overflow (rules version 27) leaves three at T50 and two at T62; the 8.9.38 line (rules version 35 merged with the special units, the siege ring and the waiting Settler's escort call) leaves two at T50 and T62; Hard's fog-aware Settler guard (2026-11-15) leaves four at T50 and three at T62.
        // Hard's named Gold fund (2026-11-15) leaves four at T62 and two VP at T100; together with the
        // door defence, supply cap, reachable wars and concentrated production of the same day, three and none.
        // Hard's purchase-generation floor (2026-11-15) returns the T62 city count to four.
        // Founded capitals (rules 45) move this fixed tape to 3/4/5 cities and 2 VP at T100.
        // The survive-him genes (2026-11-21: neighbour_alarm, the production-thrash fix) leave four cities at T62 and no VP at T100;
        // with searching rivals the seat then wins by elimination at T173 on this fixed tape.
        // The leader war and the siege clock (2026-11-21) returned it to five cities at T62 and two VP at T100, with no win; with the clock off by default (held-out finding, 2026-11-21) it is four cities at T62, no VP at T100 and the elimination win at T173 of the survive-him genes.
        // World Ages as the only era scoring, with its peaceful cards (rules 50, 2026-11-22), keeps four cities at T62 and no VP at T100, and the seat no longer wins on this tape.
        assert_eq!(
            first_map_game(true),
            json!({
                "cities_t30": 3,
                "cities_t50": 4,
                "cities_t62": 4,
                "vp_t100": 0,
                "vp_t60": 0,
                "win": false,
                "win_turn": null,
                "win_type": null,
            })
        );
    }

    #[test]
    fn plain_hard_rivals_are_the_default_benchmark() {
        // The app's default Hard: the rivals are plain ScriptedBrain, whatever the map's flag.
        // The valued Spartan expedition changes the rival war on this fixed start;
        // the seat had two VP at T100 until the faction playbooks (2026-11-05), then none;
        // the 100-Production Walls and ordinary Byzantine Library left it with one; two-party
        // Peace (rules version 20) and the mounted Mongol rival leave it with none; balanced founding sites (rules version 24) regenerate the map: four cities at T50, no VP at T60;
        // same-era resource reveals (rules version 25) leave two cities at T50 and three at T62
        // (the fixed dice flip this map; on reseeded dice the gate reads control 21.9%, branch 23.8%);
        // the siege stance defaults (2026-11-08) bring it back to four at T50 and T62;
        // with the crowns cut (2026-11-11) the seat has two VP at T100;
        // with the rivals' finish bonus counting the active build (2026-11-15) it has none;
        // with Hard's fog-aware Settler guard (2026-11-15) it has two again.
        // with the capitals victory and Hard's pressed-capital guard (rules version 37) it has none.
        // with Hard's concentrated combat-unit production (2026-11-15) it has two again.
        // with Hard's named Gold fund (2026-11-15) it has two VP at T100.
        // fresh conquest nominees and targeted re-scouting (2026-11-15) join them.
        // Hard's supported defence (2026-11-15) keeps four cities; with the day's other genes and the fund off, no VP at T100.
        // Founded capitals (rules 45) move this fixed tape to 3/4/5 cities and 2 VP at T100.
        // The survive-him genes (2026-11-21: neighbour_alarm, the production-thrash fix) leave four cities at T62 and no VP at T100.
        // The leader war and the siege clock (2026-11-21) left five cities at T62, no VP at T100 and an elimination win at T169; with the clock off by default (held-out finding, 2026-11-21) it is four cities at T62, no VP at T100 and no win.
        assert_eq!(
            first_map_game(false),
            json!({
                "cities_t30": 3,
                "cities_t50": 4,
                "cities_t62": 4,
                "vp_t100": 0,
                "vp_t60": 0,
                "win": false,
                "win_turn": null,
                "win_type": null,
            })
        );
    }

    #[test]
    fn nonzero_offset_changes_only_the_seat_stream() {
        let seat = PlayerId(3);
        for offset in 1..8 {
            let seeds = (0..8)
                .map(|player| {
                    let player = PlayerId(player);
                    player_ai_seed(0x5eed + u64::from(player.0), 17, player, seat, offset)
                })
                .collect::<Vec<_>>();
            let baseline = (0..8)
                .map(|player| {
                    let player = PlayerId(player);
                    player_ai_seed(0x5eed + u64::from(player.0), 17, player, seat, 0)
                })
                .collect::<Vec<_>>();
            assert_ne!(seeds[seat.0 as usize], baseline[seat.0 as usize]);
            for player in 0..8 {
                if player != seat.0 as usize {
                    assert_eq!(seeds[player], baseline[player]);
                }
            }
        }
    }
}
