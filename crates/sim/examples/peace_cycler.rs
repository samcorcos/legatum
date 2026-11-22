//! Exploiter seat 1, `peace_cycler` (docs/design/strategy-gap/fable.md, prediction 1): does Hard re-arm from
//! scratch each time a rival makes Peace and redeclares after the truce?
//!
//! Eight Hard seats (one of the nine factions omitted per game, seats rotated, as `aztecs_smoke`). Each seed is
//! played twice: `plain` (seat 0 is plain Hard) and `cycler` (seat 0 is Hard whose diplomacy is scripted).
//! The cycler keeps Hard's economy, armies and wars, and replaces only its Peace decisions with an empire it
//! has fought (a "target"): it accepts any Peace the target offers, offers Peace itself once a war is
//! `--war-turns` old, and declares War again the turn the truce ends (`diplomacy::surprise_war`). With no war
//! by `--first-war` it declares on the nearest empire it has met. Everything else is Hard's own orders.
//!
//!   cargo run --release -p legatum-sim --example peace_cycler -- --games 8 --seed 7700001
//!   tools/cloud/run-batch.sh --bin peace_cycler -- --games 2048 --seed 7700001 --label peace-cycler
//!
//! Output: `<out-dir>/peace-cycler.json` (summary + per-game rows). The summary pairs the two arms by seed:
//! seat 0's win share, VP, cities and eliminations, the cycles completed, and each target's levy (paid
//! military units) at Peace and at the redeclaration, to see whether the AI demobilises in the truce.
use legatum_core::diplomacy::{RelationState, TreatyKind, surprise_war};
use legatum_core::{
    AiDifficulty, Faction, GameSetup, GameState, Order, PlayerId, TeamId, economy, rules, scoring,
};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const SEAT: PlayerId = PlayerId(0);

fn arg(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map_or_else(|| default.into(), |w| w[1].clone())
}

fn setup(seed: u64) -> GameSetup {
    let mut setup = GameSetup::all_ai(seed);
    let all: Vec<_> = Faction::ALL.into_iter().collect();
    let omitted = (seed % all.len() as u64) as usize;
    let mut roster: Vec<_> = all
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| (i != omitted).then_some(f))
        .collect();
    roster.rotate_right((seed % 8) as usize);
    for (p, faction) in setup.players.iter_mut().zip(roster) {
        p.faction = faction;
        p.difficulty = AiDifficulty::Hard;
    }
    setup.turn_limit = None;
    setup
}

#[derive(Default)]
struct Cycler {
    war_turns: u32,
    first_war: u32,
    /// Teams we have been at war with, and the turn the current war with each began.
    targets: BTreeMap<TeamId, Option<u32>>,
    last_state: BTreeMap<TeamId, RelationState>,
    /// (turn, target team, target's paid military units) at each Peace and each redeclaration.
    peaces: Vec<(u32, u8, usize)>,
    redeclared: Vec<(u32, u8, usize)>,
    opened: Option<u32>,
}

fn levy_of_team(state: &GameState, team: TeamId) -> usize {
    state
        .players
        .iter()
        .filter(|p| p.alive && p.team == team)
        .map(|p| economy::levy_count(state, p.id))
        .sum()
}

fn valid(state: &GameState, order: &Order) -> bool {
    rules::validate_orders(state, SEAT, std::slice::from_ref(order)).is_empty()
}

impl Cycler {
    /// Track relation changes after a resolved turn.
    fn observe(&mut self, state: &GameState) {
        let Some(dip) = state.diplomacy.as_ref() else {
            return;
        };
        let me = state.player(SEAT).team;
        for p in state.players.iter().filter(|p| p.team != me) {
            let Some(r) = dip.relation(me, p.team) else {
                continue;
            };
            let before = self.last_state.insert(p.team, r.state);
            if r.state == RelationState::War && before != Some(RelationState::War) {
                self.targets.insert(p.team, Some(state.turn));
                if before == Some(RelationState::Peace) && r.declarers.contains(&me) {
                    self.redeclared
                        .push((state.turn, p.team.0, levy_of_team(state, p.team)));
                }
            } else if r.state == RelationState::Peace && before == Some(RelationState::War) {
                if let Some(start) = self.targets.get_mut(&p.team) {
                    *start = None;
                }
                self.peaces
                    .push((state.turn, p.team.0, levy_of_team(state, p.team)));
            }
        }
    }

    fn rewrite(&mut self, state: &GameState, mine: &mut Vec<Order>) {
        let Some(dip) = state.diplomacy.as_ref() else {
            return;
        };
        let me = state.player(SEAT).team;
        let peace_from_target: BTreeSet<_> = dip
            .proposals
            .iter()
            .filter(|p| p.kind == TreatyKind::Peace && self.targets.contains_key(&p.from))
            .map(|p| p.id)
            .collect();
        // Our Peace decisions with targets are scripted; drop Hard's own.
        mine.retain(|o| match o {
            Order::RespondToProposal { proposal, .. } => !peace_from_target.contains(proposal),
            Order::ProposeTreaty { to, kind } => {
                !(*kind == TreatyKind::Peace && self.targets.contains_key(to))
            }
            _ => true,
        });
        let mut extra = Vec::new();
        for p in dip
            .proposals
            .iter()
            .filter(|p| peace_from_target.contains(&p.id) && p.to == me)
        {
            extra.push(Order::RespondToProposal {
                proposal: p.id,
                accept: true,
                explanation: None,
            });
        }
        let mut have_war = false;
        for (&team, start) in &self.targets {
            let Some(r) = dip.relation(me, team) else {
                continue;
            };
            if !state.players.iter().any(|p| p.alive && p.team == team) {
                continue;
            }
            match r.state {
                RelationState::War => {
                    have_war = true;
                    let ours_pending = dip
                        .proposals
                        .iter()
                        .any(|p| p.kind == TreatyKind::Peace && p.from == me && p.to == team);
                    if start.is_some_and(|s| state.turn >= s + self.war_turns) && !ours_pending {
                        extra.push(Order::ProposeTreaty {
                            to: team,
                            kind: TreatyKind::Peace,
                        });
                    }
                }
                RelationState::Peace if surprise_war(r, state.turn) && r.security.is_none() => {
                    extra.push(Order::DeclareWar {
                        against: team,
                        breach: false,
                    });
                }
                _ => {}
            }
        }
        // The opening war, if Hard has not picked one: the nearest empire we can declare on.
        if !have_war && self.targets.is_empty() && state.turn >= self.first_war {
            let home = state.cities_of(SEAT).next().map(|c| c.pos);
            let nearest = home.and_then(|h| {
                state
                    .cities
                    .values()
                    .filter(|c| state.player(c.owner).team != me)
                    .map(|c| (c.pos.distance(h), state.player(c.owner).team))
                    .filter(|(_, t)| {
                        valid(
                            state,
                            &Order::DeclareWar {
                                against: *t,
                                breach: false,
                            },
                        )
                    })
                    .min()
            });
            if let Some((_, team)) = nearest {
                extra.push(Order::DeclareWar {
                    against: team,
                    breach: false,
                });
                self.opened = Some(state.turn);
            }
        }
        for o in extra {
            if valid(state, &o) {
                mine.push(o);
            }
        }
    }
}

fn play(seed: u64, cycle: bool, war_turns: u32, first_war: u32) -> Value {
    let setup = setup(seed);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    let mut c = Cycler {
        war_turns,
        first_war,
        ..Default::default()
    };
    let cap = 1200;
    let mut levy_t100 = BTreeMap::new();
    while !session.state.finished && session.state.turn <= cap {
        let mut orders = session.ai_orders();
        if cycle && session.state.player(SEAT).alive {
            let mut mine = orders.remove(&SEAT).unwrap_or_default();
            c.rewrite(&session.state, &mut mine);
            orders.insert(SEAT, mine);
        }
        session.step(&orders);
        c.observe(&session.state);
        if session.state.turn == 100 {
            for p in &session.state.players {
                levy_t100.insert(p.id.0, economy::levy_count(&session.state, p.id));
            }
        }
    }
    let state = &session.state;
    let me = state.player(SEAT);
    let target_teams: BTreeSet<u8> = c.targets.keys().map(|t| t.0).collect();
    let targets: Vec<_> = state
        .players
        .iter()
        .filter(|p| target_teams.contains(&p.team.0))
        .map(|p| {
            json!({"faction": p.faction, "alive": p.alive, "cities": state.cities_of(p.id).count(),
                   "vp": scoring::vp_total(state, p.id), "won": state.winner == Some(p.team)})
        })
        .collect();
    json!({
        "seed": seed, "arm": if cycle { "cycler" } else { "plain" }, "turns": state.turn,
        "stalled": !state.finished, "faction": me.faction,
        "won": state.winner == Some(me.team), "alive": me.alive,
        "vp": scoring::vp_total(state, SEAT), "cities": state.cities_of(SEAT).count(),
        "levy_t100": levy_t100.get(&SEAT.0).copied(),
        "rivals_levy_t100": levy_t100.iter().filter(|(k, _)| **k != SEAT.0).map(|(_, v)| *v).sum::<usize>(),
        "targets": targets, "opened": c.opened,
        "peaces": c.peaces, "redeclared": c.redeclared,
    })
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let games: usize = arg(&args, "--games", "8").parse().unwrap();
    let seed: u64 = arg(&args, "--seed", "7700001").parse().unwrap();
    let war_turns: u32 = arg(&args, "--war-turns", "8").parse().unwrap();
    let first_war: u32 = arg(&args, "--first-war", "30").parse().unwrap();
    let jobs: Vec<_> = (0..games)
        .flat_map(|i| [(seed + i as u64, false), (seed + i as u64, true)])
        .collect();
    let rows: Vec<Value> = jobs
        .into_par_iter()
        .map(|(s, cycle)| {
            match std::panic::catch_unwind(|| play(s, cycle, war_turns, first_war)) {
                Ok(row) => row,
                Err(e) => json!({"seed": s, "arm": if cycle {"cycler"} else {"plain"}, "panic":
                e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()}),
            }
        })
        .collect();
    let by: BTreeMap<(u64, String), &Value> = rows
        .iter()
        .filter(|r| r.get("panic").is_none())
        .map(|r| {
            (
                (
                    r["seed"].as_u64().unwrap(),
                    r["arm"].as_str().unwrap().into(),
                ),
                r,
            )
        })
        .collect();
    let seeds: Vec<u64> = (0..games as u64)
        .map(|i| seed + i)
        .filter(|s| {
            by.contains_key(&(*s, "plain".into())) && by.contains_key(&(*s, "cycler".into()))
        })
        .collect();
    let f = |arm: &str, key: &str| -> Vec<f64> {
        seeds
            .iter()
            .map(|s| {
                let v = &by[&(*s, arm.to_string())][key];
                v.as_f64()
                    .or_else(|| v.as_bool().map(|b| f64::from(u8::from(b))))
                    .unwrap_or(0.0)
            })
            .collect()
    };
    let wins_p = f("plain", "won");
    let wins_c = f("cycler", "won");
    let d: Vec<f64> = wins_c.iter().zip(&wins_p).map(|(a, b)| a - b).collect();
    let n = d.len() as f64;
    let sd = (d.iter().map(|x| (x - mean(&d)).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
    let cycles: Vec<f64> = seeds
        .iter()
        .map(|s| {
            by[&(*s, "cycler".to_string())]["redeclared"]
                .as_array()
                .unwrap()
                .len() as f64
        })
        .collect();
    let mut levy_at_peace = Vec::new();
    let mut levy_at_redeclare = Vec::new();
    for s in &seeds {
        let r = by[&(*s, "cycler".to_string())];
        let peaces = r["peaces"].as_array().unwrap();
        for red in r["redeclared"].as_array().unwrap() {
            let (turn, team) = (red[0].as_u64().unwrap(), red[1].as_u64().unwrap());
            if let Some(p) = peaces
                .iter()
                .rev()
                .find(|p| p[1].as_u64() == Some(team) && p[0].as_u64().unwrap() < turn)
            {
                levy_at_peace.push(p[2].as_f64().unwrap());
                levy_at_redeclare.push(red[2].as_f64().unwrap());
            }
        }
    }
    let target_won = |arm: &str| -> f64 {
        mean(
            &seeds
                .iter()
                .map(|s| {
                    let ts = by[&(*s, arm.to_string())]["targets"].as_array().unwrap();
                    f64::from(u8::from(ts.iter().any(|t| t["won"] == true)))
                })
                .collect::<Vec<_>>(),
        )
    };
    let summary = json!({
        "games": games, "paired": seeds.len(), "war_turns": war_turns, "first_war": first_war,
        "panics": rows.iter().filter(|r| r.get("panic").is_some()).count(),
        "stalls": rows.iter().filter(|r| r["stalled"] == true).count(),
        "seat_win_pct": {"plain": 100.0 * mean(&wins_p), "cycler": 100.0 * mean(&wins_c),
                         "delta_pp": 100.0 * mean(&d), "ci95_pp": [100.0 * (mean(&d) - 1.96 * sd / n.sqrt()), 100.0 * (mean(&d) + 1.96 * sd / n.sqrt())]},
        "seat_vp": {"plain": mean(&f("plain", "vp")), "cycler": mean(&f("cycler", "vp"))},
        "seat_cities": {"plain": mean(&f("plain", "cities")), "cycler": mean(&f("cycler", "cities"))},
        "seat_eliminated_pct": {"plain": 100.0 * (1.0 - mean(&f("plain", "alive"))), "cycler": 100.0 * (1.0 - mean(&f("cycler", "alive")))},
        "seat_levy_t100": {"plain": mean(&f("plain", "levy_t100")), "cycler": mean(&f("cycler", "levy_t100"))},
        "rivals_levy_t100": {"plain": mean(&f("plain", "rivals_levy_t100")), "cycler": mean(&f("cycler", "rivals_levy_t100"))},
        "redeclarations_per_game": mean(&cycles),
        "target_levy": {"at_peace": mean(&levy_at_peace), "at_redeclare": mean(&levy_at_redeclare), "pairs": levy_at_peace.len()},
        "a_target_won_pct": {"plain": 100.0 * target_won("plain"), "cycler": 100.0 * target_won("cycler")},
    });
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    let out = PathBuf::from(arg(&args, "--out-dir", "."));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("peace-cycler.json"),
        serde_json::to_vec(&json!({"summary": summary, "games": rows})).unwrap(),
    )
    .unwrap();
}
