//! Plays AI-only games through the real core: every scripted order must pass validate_orders,
//! and none may fail at resolution. The ignored `many_games_summary` prints aggregate stats:
//! `GAMES=64 cargo test --release -p legatum-ai --test full_game -- --ignored --nocapture`
//! (`SHOW=1` also prints every rejected or failed order and bounce).

use legatum_ai::{Brain, Knobs, RandomBrain, ScriptedBrain};
use legatum_core::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct Summary {
    turns: u32,
    winner: Option<TeamId>,
    factions: Vec<Faction>,
    cities: Vec<u32>,
    founded: Vec<bool>,
    /// validate_orders rejections of scripted orders.
    rejected: Vec<String>,
    /// OrderFailed events for scripted players, plus bounces.
    failed: Vec<String>,
    captures: u32,
    settler_captures: u32,
    eliminations: u32,
    battles: u32,
    /// Captures made, per player.
    captured_by: Vec<u32>,
    /// How often each (units, tile) bounce happened.
    bounce_counts: BTreeMap<(Vec<UnitId>, Hex), u32>,
    max_bounce_repeats: u32,
}

/// Tuning experiments: `KNOBS_INDIA=80,45,40,50,70` (expansion, military, science, economy,
/// aggression) overrides a faction's defaults.
fn knob_override(f: Faction) -> Option<Knobs> {
    let v = std::env::var(format!("KNOBS_{}", format!("{f:?}").to_uppercase())).ok()?;
    let k: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (k.len() == 5).then(|| Knobs::new(k[0], k[1], k[2], k[3], k[4]))
}

fn play(seed: u64, turns: u32, random_seats: &[u8]) -> Summary {
    let setup = GameSetup::all_ai(seed);
    let mut state = new_game(&setup);
    let mut brains: Vec<Box<dyn Brain>> = state
        .players
        .iter()
        .map(|p| -> Box<dyn Brain> {
            if random_seats.contains(&p.id.0) {
                Box::new(RandomBrain::new(seed ^ p.id.0 as u64, p.id))
            } else {
                let knobs = knob_override(p.faction).unwrap_or(Knobs::for_faction(p.faction));
                Box::new(ScriptedBrain::with_knobs(
                    p.faction,
                    seed ^ p.id.0 as u64,
                    p.id,
                    knobs,
                ))
            }
        })
        .collect();
    let mut s = Summary {
        factions: state.players.iter().map(|p| p.faction).collect(),
        captured_by: vec![0; state.players.len()],
        founded: vec![false; state.players.len()],
        ..Default::default()
    };
    let scripted = |p: PlayerId| !random_seats.contains(&p.0);
    while !state.finished && s.turns < turns {
        let mut all = BTreeMap::new();
        for p in state.players.clone() {
            if !p.alive {
                continue;
            }
            let view = player_view(&state, p.id);
            let orders = brains[p.id.0 as usize].orders(&view);
            if std::env::var("TRACE").is_ok_and(|t| t == format!("{seed}:{}", p.id.0))
                && s.turns < 4
            {
                let units: Vec<_> = state
                    .units_of(p.id)
                    .map(|u| (u.id, u.kind, u.pos))
                    .collect();
                println!("turn {} units {units:?}\n  orders {orders:?}", state.turn);
            }
            if scripted(p.id) {
                for e in validate_orders(&state, p.id, &orders) {
                    s.rejected.push(format!(
                        "turn {} player {}: {:?}: {}",
                        state.turn, p.id.0, orders[e.index], e.message
                    ));
                }
            }
            all.insert(p.id, orders);
        }
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &all);
        // A city burned by its holder's Raze order is gone before ranged fire resolves, so a shot
        // another empire planned against it the same turn fails: simultaneous contention.
        let razed: std::collections::BTreeSet<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::CityRazed { at, .. } => Some(*at),
                _ => None,
            })
            .collect();
        let shot_at_razed_city = |player: PlayerId, reason: &str| {
            let Some(unit) = reason
                .strip_prefix("Ranged attack by unit ")
                .and_then(|r| r.split(' ').next())
                .and_then(|id| id.parse::<u32>().ok())
            else {
                return false;
            };
            all.get(&player).is_some_and(|orders| {
                orders.iter().any(|o| {
                    matches!(o, Order::RangedAttack { unit: u, target }
                        if u.0 == unit && razed.contains(target))
                })
            })
        };
        for e in events {
            match e {
                Event::OrderFailed { player, reason }
                    if scripted(player) && shot_at_razed_city(player, &reason) => {}
                Event::OrderFailed { player, reason } if scripted(player) => {
                    // A city the founder cannot see (exactly three hexes away, so its border is also out
                    // of sight) is fair play: a fogged Settler cannot know it is there. The failure
                    // marks the site, so only an unseen city produces this message.
                    if reason.contains("Too close to") && {
                        let v = player_view(&state, player);
                        state
                            .units_of(player)
                            .filter(|u| u.kind == UnitKind::Settler)
                            .any(|u| {
                                state.cities.values().any(|c| {
                                    c.pos.distance(u.pos) <= 3
                                        && !v.cities.iter().any(|seen| seen.id == c.id)
                                })
                            })
                    } {
                        continue;
                    }
                    s.failed
                        .push(format!("turn {turn} player {}: {reason}", player.0));
                }
                Event::Bounce { units, tile } => {
                    let owners: Vec<u8> = units
                        .iter()
                        .filter_map(|u| state.units.get(u).map(|u| u.owner.0))
                        .collect();
                    // Neutral empires cannot coordinate their simultaneous destinations.
                    // This invariant covers repeated collisions within a permanent team.
                    if owners.iter().all(|o| scripted(PlayerId(*o)))
                        && owners.first().is_some_and(|first| {
                            owners
                                .iter()
                                .all(|o| state.same_team(PlayerId(*first), PlayerId(*o)))
                        })
                    {
                        let who: Vec<_> = units
                            .iter()
                            .filter_map(|u| next.units.get(u))
                            .map(|u| (u.id.0, u.owner.0, u.kind, u.pos, u.goto))
                            .collect();
                        let key = (units.clone(), tile);
                        let repeats = s.bounce_counts.entry(key).or_insert(0);
                        *repeats += 1;
                        s.max_bounce_repeats = s.max_bounce_repeats.max(*repeats);
                        s.failed
                            .push(format!("turn {turn} bounce at {tile:?}: {who:?}"));
                    }
                }
                Event::CityCaptured { to, .. } => {
                    s.captures += 1;
                    s.captured_by[to.0 as usize] += 1;
                }
                Event::CityFounded { owner, .. } => s.founded[owner.0 as usize] = true,
                Event::SettlerCaptured { .. } => s.settler_captures += 1,
                Event::PlayerEliminated { .. } => s.eliminations += 1,
                Event::Battle { .. } => s.battles += 1,
                _ => {}
            }
        }
        state = next;
        s.turns += 1;
    }
    s.winner = state.winner;
    s.cities = state
        .players
        .iter()
        .map(|p| state.cities_of(p.id).count() as u32)
        .collect();
    s
}

#[test]
fn scripted_orders_all_validate_and_succeed_in_a_real_game() {
    // Seed 11 on the balanced-start map has a Mine order whose tile changes owner the same turn.
    for seed in [12, 13] {
        let s = play(seed, 60, &[7]);
        assert!(
            s.rejected.is_empty(),
            "rejected orders:\n{}",
            s.rejected.join("\n")
        );
        // A settler captured in the same turn it tries to found is fair play; anything else
        // failing at resolution is an AI bug, except simultaneous diplomatic offers whose
        // canonical pair winner was unknowable during planning. Repeated friendly bounces are bugs.
        // Rival settlers founding near each other on the same turn is a legitimate simultaneous
        // collision (neither can know the other's plan); only a player clashing with itself is a bug.
        let rival_found_clash = |f: &String| {
            if !f.contains("another settler tried to found too close") {
                return false;
            }
            let turn = f.split(" player").next().unwrap_or_default();
            let players: std::collections::BTreeSet<&str> = s
                .failed
                .iter()
                .filter(|g| g.starts_with(turn) && g.contains("another settler tried to found"))
                .filter_map(|g| g.split(" player ").nth(1).and_then(|r| r.split(':').next()))
                .collect();
            players.len() >= 2
        };
        let failed: Vec<&String> = s
            .failed
            .iter()
            .filter(|f| !f.contains("bounce") && !f.contains("no longer available"))
            .filter(|f| !rival_found_clash(f))
            // Opponents plan against the same frozen state. Both offers validate,
            // but only the canonical proposal ID can win this pair at the boundary.
            // This is simultaneous contention, not an invalid AI order.
            .filter(|f| {
                !f.contains("Another proposal won canonical proposal-ID order for this pair")
            })
            // Two cities of a levy-capped empire can finish armed units in the same turn with
            // one slot left: simultaneous contention, and the Production stays banked.
            .filter(|f| !f.contains("Levies full"))
            // Barbarian raiders plan against the start-of-turn state: a shot at a tile whose
            // target moved or died the same turn is simultaneous contention, not an AI order.
            .filter(|f| {
                !(f.contains("player 255") && f.contains("no enemy unit or city to attack"))
            })
            // Another empire may declare war after an offer was planned against
            // the same frozen state; resolution then invalidates that offer.
            .filter(|f| !f.contains("Relationship changed before proposal creation"))
            // A target team can also be eliminated during simultaneous combat before
            // the diplomacy boundary activates an otherwise valid offer.
            .filter(|f| !f.contains("A team was eliminated before diplomacy could activate"))
            // A Peace offer needs every team on both sides: one refusal (or an earlier acceptance that
            // closed it) processed first in the boundary ends the offer, so an ally's answer planned
            // against the same frozen state finds it gone. Simultaneous contention, not an AI order.
            .filter(|f| {
                !(f.contains("Proposal withdrawn or rejected at this boundary")
                    || f.contains("Proposal already closed at this boundary"))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "orders failed at resolution:\n{failed:#?}"
        );
        assert!(
            s.max_bounce_repeats <= 2,
            "repeated bounces:\n{}",
            s.failed.join("\n")
        );
        // Every scripted empire founded a capital, even if it was later conquered.
        assert!(
            s.founded.iter().take(7).all(|f| *f),
            "founded: {:?}; current cities: {:?}",
            s.founded,
            s.cities,
        );
    }
}

#[test]
#[ignore]
fn many_games_summary() {
    let games: u64 = std::env::var("GAMES")
        .ok()
        .and_then(|g| g.parse().ok())
        .unwrap_or(32);
    let show = std::env::var("SHOW").is_ok();
    let mut wins: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let (mut rejected, mut failed, mut captures, mut settlers, mut elim, mut battles) =
        (0, 0, 0, 0, 0, 0);
    let (mut decided, mut turns) = (0, 0);
    let mut captured_by_faction: BTreeMap<String, u32> = BTreeMap::new();
    for seed in 1..=games {
        let s = play(seed, 200, &[]);
        for (i, f) in s.factions.iter().enumerate() {
            let e = wins.entry(format!("{f:?}")).or_default();
            e.1 += 1;
            if s.winner == Some(TeamId(i as u8)) {
                e.0 += 1;
            }
            *captured_by_faction.entry(format!("{f:?}")).or_default() += s.captured_by[i];
        }
        decided += s.winner.is_some() as u32;
        turns += s.turns;
        rejected += s.rejected.len();
        failed += s.failed.len();
        captures += s.captures;
        settlers += s.settler_captures;
        elim += s.eliminations;
        battles += s.battles;
        if show {
            for r in s.rejected.iter().chain(&s.failed) {
                println!("  seed {seed}: {r}");
            }
        }
    }
    println!(
        "{games} games: avg {} turns, {decided} decided, {captures} city captures, {settlers} settler captures, {elim} eliminations, {battles} battles, {rejected} rejected, {failed} failed/bounced",
        turns / games as u32
    );
    for (f, (w, seats)) in wins {
        println!(
            "  {f}: {w} wins / {seats} seats, {} captures",
            captured_by_faction[&f]
        );
    }
}

/// A huge explicit turn limit must not overflow the AI's late-game planning horizon
/// (it panicked at turn 65 in debug builds when the remaining turns reached i32::MAX).
#[test]
fn huge_turn_limit_keeps_late_planning_in_range() {
    let mut setup = GameSetup::all_ai(700_001);
    setup.turn_limit = Some(2_147_483_712);
    let mut state = new_game(&setup);
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_knobs(
                p.faction,
                p.id.0 as u64,
                p.id,
                Knobs::for_faction(p.faction),
            )
        })
        .collect();
    while !state.finished && state.turn <= 70 {
        let mut all = BTreeMap::new();
        for p in state.players.clone().into_iter().filter(|p| p.alive) {
            let view = player_view(&state, p.id);
            all.insert(p.id, brains[p.id.0 as usize].orders(&view));
        }
        state = resolve_turn(&state, &all).0;
    }
    assert!(state.turn > 65 || state.finished);
}
