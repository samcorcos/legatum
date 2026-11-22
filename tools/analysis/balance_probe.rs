//! Read-only telemetry over the production rules and brains. No modified game rules.
//! Optional knob overrides create explicitly labelled experimental populations.

use clap::Parser;
use legatum_ai::{Brain, Knobs, ScriptedBrain};
use legatum_core::events::Defender;
use legatum_core::{
    BuildingKind, Event, GameSetup, GameState, Order, PlayerId, Title, UnitKind, combat, data,
    new_game, player_view, resolve_turn, scoring,
};
use legatum_wasm::session::{BrainKind, OrderMap, brain_seed, make_brain};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    games: usize,
    #[arg(long)]
    seed: u64,
    /// Stop runaway simulations without awarding victory (default: 4x expected length).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..u32::MAX as i64))]
    safety_cap: Option<u32>,
    /// Optional explicit turn limit; highest VP wins at this turn.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..u32::MAX as i64))]
    turn_limit: Option<u32>,
    #[arg(long, default_value = "scripted", value_parser = ["scripted", "mixed"])]
    brain: String,
    /// In mixed games, swap which seat parity gets RandomBrain.
    #[arg(long)]
    random_even: bool,
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    aggression: Option<u8>,
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    military: Option<u8>,
    #[arg(long)]
    out: PathBuf,
}

impl Args {
    fn safety_cap(&self) -> u32 {
        self.safety_cap.unwrap_or(1200)
    }
}

type Counts = BTreeMap<String, u64>;
type Histograms = BTreeMap<String, BTreeMap<i32, u64>>;

fn inc(c: &mut Counts, name: &str, n: u64) {
    *c.entry(name.to_owned()).or_default() += n;
}

fn hist(h: &mut Histograms, name: &str, value: i32) {
    *h.entry(name.to_owned())
        .or_default()
        .entry(value)
        .or_default() += 1;
}

fn army(kind: UnitKind) -> bool {
    data::unit_stats(kind).military && kind != UnitKind::Scout
}

fn sources(s: &GameState, p: PlayerId) -> BTreeMap<String, u32> {
    scoring::vp(s, p)
        .into_iter()
        .map(|(source, value)| (format!("{source:?}"), value))
        .collect()
}

fn snapshot(s: &GameState, names: &[&str]) -> Vec<Value> {
    s.players
        .iter()
        .map(|p| {
            let mut units = Counts::new();
            for u in s.units_of(p.id) {
                inc(&mut units, &format!("{:?}", u.kind), 1);
            }
            json!({
                "id": p.id.0, "faction": p.faction, "team": p.team.0,
                "brain": names[p.id.0 as usize], "alive": p.alive,
                "vp": scoring::vp_total(s, p.id), "sources": sources(s, p.id),
                "cities": s.cities_of(p.id).count(), "techs": p.techs.len(),
                "population": s.cities_of(p.id).map(|c| c.population).sum::<i32>(),
                "army": s.units_of(p.id).filter(|u| army(u.kind)).count(),
                "units": units,
                "army_strength": scoring::title_metric(s, p.id, Title::StrongestArmy),
            })
        })
        .collect()
}

fn kind(args: &Args, p: PlayerId) -> BrainKind {
    let random = args.brain == "mixed" && (p.0.is_multiple_of(2) == args.random_even);
    if random {
        BrainKind::Random
    } else {
        BrainKind::Scripted
    }
}

fn run(args: &Args, seed: u64) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    setup.turn_limit = args.turn_limit;
    let mut state = new_game(&setup);
    let safety_cap = args.safety_cap();
    let names: Vec<_> = state
        .players
        .iter()
        .map(|p| kind(args, p.id).name())
        .collect();
    // The same constructor, seed derivation, seat order and player views as Session.
    let mut brains: Vec<Box<dyn Brain>> = state
        .players
        .iter()
        .map(|p| {
            if kind(args, p.id) == BrainKind::Scripted
                && (args.aggression.is_some() || args.military.is_some())
            {
                let mut k = Knobs::for_faction(p.faction);
                if let Some(a) = args.aggression {
                    k.aggression = i32::from(a);
                }
                if let Some(m) = args.military {
                    k.military = i32::from(m);
                }
                Box::new(ScriptedBrain::with_knobs(
                    p.faction,
                    brain_seed(seed, p.id),
                    p.id,
                    k,
                )) as Box<dyn Brain>
            } else {
                make_brain(kind(args, p.id), &state, p.id)
            }
        })
        .collect();
    let mut counts = Counts::new();
    let mut h = Histograms::new();
    let mut at50 = None;
    let mut before_final = Vec::new();
    let mut scoring_events = Vec::new();
    let mut captures = Vec::new();
    let mut eliminations = Vec::new();
    let mut city_first_hit = BTreeMap::new();
    let mut city_hits = BTreeMap::new();
    let mut city_zeros = BTreeMap::new();
    let mut first_battle = None;
    let mut first_city_battle = None;
    let mut turns = 0;
    // Core scoring handles the optional turn limit; this cap never awards victory.
    while !state.finished && turns < safety_cap {
        let turn = state.turn;
        let mut orders = OrderMap::new();
        for p in state.players.iter().filter(|p| p.alive) {
            let view = player_view(&state, p.id);
            let own_cities: Vec<_> = view.cities.iter().filter(|c| c.owner == p.id).collect();
            let threatened = view.units.iter().any(|u| {
                state.team_of(u.owner) != p.team
                    && u.military
                    && combat::unit_attack_strength(u.kind, u.hp)
                        .total
                        .max(if u.ranged > 0 {
                            combat::unit_ranged_strength(u.kind, u.hp).total
                        } else {
                            0
                        })
                        .max(0)
                        * u.hp.max(0)
                        / data::UNIT_HP
                        > 0
                    && own_cities.iter().any(|c| c.pos.distance(u.pos) <= 3)
            });
            let known_enemy = view.cities.iter().any(|c| state.team_of(c.owner) != p.team);
            let prefix = names[p.id.0 as usize];
            inc(&mut counts, &format!("{prefix}_empire_turns"), 1);
            inc(
                &mut counts,
                &format!("{prefix}_threatened_empire_turns"),
                threatened as u64,
            );
            inc(
                &mut counts,
                &format!("{prefix}_known_enemy_city_turns"),
                known_enemy as u64,
            );
            let os = brains[p.id.0 as usize].orders(&view);
            for o in &os {
                let (id, action) = match o {
                    Order::MoveUnit { unit, .. } => (*unit, "move"),
                    Order::SkipUnit { unit } => (*unit, "hold"),
                    Order::RangedAttack { unit, .. } => (*unit, "ranged"),
                    Order::UpgradeUnit { unit } => (*unit, "upgrade"),
                    _ => continue,
                };
                if let Some(u) = state.units.get(&id).filter(|u| army(u.kind)) {
                    inc(&mut counts, &format!("{prefix}_army_orders"), 1);
                    inc(&mut counts, &format!("{prefix}_army_order_{action}"), 1);
                    if action == "hold" && own_cities.iter().any(|c| c.pos == u.pos) {
                        inc(&mut counts, &format!("{prefix}_army_hold_in_city"), 1);
                    }
                    if let Order::MoveUnit { to, .. } = o
                        && state
                            .city_at(*to)
                            .is_some_and(|c| state.team_of(c.owner) != p.team)
                    {
                        inc(&mut counts, &format!("{prefix}_army_move_to_enemy_city"), 1);
                    }
                }
            }
            orders.insert(p.id, os);
        }
        let before: Vec<_> = state
            .players
            .iter()
            .map(|p| sources(&state, p.id))
            .collect();
        let (next, events) = resolve_turn(&state, &orders);
        let mut turn_battles = 0;
        for e in &events {
            match e {
                Event::Battle {
                    attacker_owner, defender, at, ranged, meeting, damage_to_attacker,
                    damage_to_defender, attacker_strength, defender_strength, defender_hp_after, ..
                } => {
                    turn_battles += 1;
                    first_battle.get_or_insert(turn);
                    inc(&mut counts, "battles", 1);
                    inc(&mut counts, &format!("{}_battles", names[attacker_owner.0 as usize]), 1);
                    inc(&mut counts, "ranged_battles", *ranged as u64);
                    inc(&mut counts, "meeting_battles", *meeting as u64);
                    let target = if matches!(defender, Defender::City { .. }) { "city" } else { "unit" };
                    let attack = if *ranged { "ranged" } else { "melee" };
                    inc(&mut counts, &format!("{target}_{attack}_battles"), 1);
                    hist(&mut h, &format!("{target}_{attack}_damage_dealt"), *damage_to_defender);
                    hist(&mut h, &format!("{target}_{attack}_damage_received"), *damage_to_attacker);
                    if let Some(c) = state.city_at(*at) {
                        // Includes hits absorbed by a garrison, for a city's encounter history.
                        city_first_hit.entry(c.id).or_insert(turn);
                        *city_hits.entry(c.id).or_insert(0) += 1;
                        if matches!(defender, Defender::Unit { .. }) {
                            inc(&mut counts, "garrison_battles", 1);
                        }
                    }
                    if let Defender::City { city, .. } = defender {
                        first_city_battle.get_or_insert(turn);
                        let c = &state.cities[city];
                        inc(&mut counts, "city_battles", 1);
                        inc(&mut counts, "city_wall_battles", c.buildings.contains(&BuildingKind::Walls) as u64);
                        hist(&mut h, "city_population_at_hit", c.population);
                        hist(&mut h, "city_defense_at_hit", defender_strength.total);
                        hist(&mut h, "city_attacker_strength", attacker_strength.total);
                        if *defender_hp_after == 0 {
                            inc(&mut counts, "city_zero_hp_battles", 1);
                            city_zeros.entry(*city).or_insert(turn);
                        }
                    }
                }
                Event::CityCaptured { city, from, to, .. } => {
                    inc(&mut counts, "captures", 1);
                    inc(&mut counts, &format!("{}_captures", names[to.0 as usize]), 1);
                    captures.push(json!({
                        "turn": turn, "city": city.0, "from": from.0, "to": to.0,
                        "from_brain": names[from.0 as usize], "to_brain": names[to.0 as usize],
                        "first_hit": city_first_hit.get(city), "hits": city_hits.get(city),
                    }));
                }
                Event::UnitDestroyed { unit_kind, .. } => {
                    inc(&mut counts, "units_destroyed", 1);
                    inc(&mut counts, "army_destroyed", army(*unit_kind) as u64);
                }
                Event::UnitDisbanded { .. } => inc(&mut counts, "units_disbanded", 1),
                Event::UnitMoved { unit, .. } => {
                    if state.units.get(unit).is_some_and(|u| army(u.kind)) {
                        inc(&mut counts, "army_move_events", 1);
                    }
                }
                Event::MoveBlocked { reason, .. } => {
                    inc(&mut counts, "move_blocked", 1);
                    inc(&mut counts, &format!("blocked: {reason}"), 1);
                }
                Event::Bounce { .. } => inc(&mut counts, "bounces", 1),
                Event::OrderFailed { .. } => inc(&mut counts, "order_failed", 1),
                Event::TitleChanged { title, from, to } => scoring_events.push(json!({
                    "turn": turn, "source": format!("{title:?}"), "from": from.map(|p| p.0), "to": to.map(|p| p.0), "vp": 1,
                })),
                Event::EraObjectiveScored { player, era, .. } => scoring_events.push(json!({
                    "turn": turn, "source": format!("EraObjective{era}"), "to": player.0, "vp": 1,
                })),
                Event::GreatWorkCompleted { player, .. } => scoring_events.push(json!({
                    "turn": turn, "source": "GreatWork", "to": player.0, "vp": 2,
                })),
                Event::FactionGoalCompleted { player, vp, .. } => scoring_events.push(json!({
                    "turn": turn, "source": "Faction goal", "to": player.0, "vp": vp,
                })),
                Event::PlayerEliminated { player } => eliminations.push(json!({"player": player.0, "turn": turn})),
                _ => {}
            }
        }
        hist(&mut h, "battles_per_turn", turn_battles);
        for p in next.players.iter().filter(|p| p.alive) {
            let units: Vec<_> = next.units_of(p.id).filter(|u| army(u.kind)).collect();
            hist(&mut h, "army_per_empire_turn", units.len() as i32);
            hist(
                &mut h,
                &format!("{}_army_per_empire_turn", names[p.id.0 as usize]),
                units.len() as i32,
            );
            for u in &units {
                inc(&mut counts, "army_unit_turns", 1);
                inc(
                    &mut counts,
                    "army_unit_turns_on_own_city",
                    next.city_at(u.pos).is_some_and(|c| c.owner == p.id) as u64,
                );
                inc(
                    &mut counts,
                    "army_unit_turns_within_2_own_city",
                    next.cities_of(p.id).any(|c| c.pos.distance(u.pos) <= 2) as u64,
                );
                inc(
                    &mut counts,
                    "army_unit_turns_adjacent_enemy_city",
                    next.cities
                        .values()
                        .any(|c| !next.same_team(c.owner, p.id) && c.pos.distance(u.pos) == 1)
                        as u64,
                );
            }
        }
        inc(&mut counts, "city_turns", next.cities.len() as u64);
        for c in next.cities.values() {
            inc(&mut counts, "city_zero_hp_end_turns", (c.hp == 0) as u64);
        }
        turns += 1;
        if turns == 50 {
            at50 = Some(snapshot(&next, &names));
        }
        if next.finished {
            before_final = before;
        }
        state = next;
    }
    let outcome = match state.winner {
        _ if !state.finished => "stalled",
        None => "draw",
        Some(_) if state.players.iter().filter(|p| p.alive).count() <= 1 => "conquest",
        Some(w) if scoring::team_vp(&state, w) >= scoring::team_target(&state, w) => "vp",
        Some(_) => "turn_limit",
    };
    json!({
        "seed": seed.to_string(), "brain_mode": args.brain, "random_even": args.random_even,
        "aggression": args.aggression, "military": args.military,
        "turn_limit": state.setup.turn_limit, "safety_cap": safety_cap,
        "finished": state.finished,
        "turns": turns, "winner_team": state.winner.map(|t| t.0), "outcome": outcome,
        "final_hash": state.hash().to_string(), "players": snapshot(&state, &names),
        "turn50": at50, "before_final": before_final, "counts": counts, "histograms": h,
        "first_battle": first_battle, "first_city_battle": first_city_battle,
        "captures": captures, "eliminations": eliminations,
        "cities_hit": city_hits.len(), "cities_reduced_to_zero": city_zeros.len(),
        "scoring_events": scoring_events,
    })
}

fn main() {
    let args = Args::parse();
    assert!(args.games > 0, "games must be positive");
    assert!(
        !args.random_even || args.brain == "mixed",
        "random-even requires mixed"
    );
    let records: Vec<_> = (0..args.games)
        .into_par_iter()
        .map(|i| {
            run(
                &args,
                args.seed.checked_add(i as u64).expect("seed overflow"),
            )
        })
        .collect();
    let mut out = BufWriter::new(File::create(&args.out).expect("create telemetry"));
    for r in &records {
        serde_json::to_writer(&mut out, r).expect("write record");
        writeln!(out).expect("write newline");
    }
    out.flush().expect("flush telemetry");
    eprintln!("wrote {} games to {}", records.len(), args.out.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(extra: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from(
            [
                "balance-probe",
                "--games",
                "1",
                "--seed",
                "1",
                "--out",
                "unused.jsonl",
            ]
            .into_iter()
            .chain(extra.iter().copied()),
        )
    }

    #[test]
    fn safety_cap_defaults_to_standard_length_without_imposing_a_turn_limit() {
        let args = parse_args(&[]).unwrap();
        assert_eq!(args.safety_cap(), 1200);
        assert_eq!((args.turn_limit, args.safety_cap), (None, None));
        let args = parse_args(&["--safety-cap", "77"]).unwrap();
        assert_eq!(args.safety_cap(), 77);
        assert_eq!(args.turn_limit, None);
    }

    #[test]
    fn rejects_removed_speed_flag_and_invalid_limits() {
        for extra in [
            ["--speed", "quick"],
            ["--safety-cap", "0"],
            ["--safety-cap", "4294967295"],
            ["--turn-limit", "0"],
            ["--turn-limit", "4294967295"],
        ] {
            assert!(parse_args(&extra).is_err(), "{extra:?}");
        }
    }

    #[test]
    fn safety_cap_stalls_but_optional_turn_limit_finishes() {
        let mut args = parse_args(&["--safety-cap", "2"]).unwrap();
        for limit in [None, Some(3)] {
            args.turn_limit = limit;
            let record = run(&args, 1);
            assert_eq!(record["turns"], 2);
            assert_eq!(record["outcome"], "stalled");
            assert_eq!(record["finished"], false);
            assert_eq!(record["winner_team"], Value::Null);
            assert_eq!(record["turn_limit"], json!(limit));
            assert_eq!(record["safety_cap"], 2);
        }
        for limit in [1, 2] {
            args.turn_limit = Some(limit);
            let record = run(&args, 1);
            assert_eq!(record["turns"], limit);
            assert_eq!(record["finished"], true);
            assert!(matches!(
                record["outcome"].as_str(),
                Some("draw" | "turn_limit")
            ));
        }
    }
}
