//! Diagnose Hard's expansion from one recorded start, read-only:
//! expansion_probe SAVE.legatum SALTS TURNS [PARAMS.json] > out.jsonl
//! SAVE is a designer save; its human seat and every other seat play
//! plain Hard (PARAMS, when given, in every seat); each salt reseeds the dice and the AI streams
//! as `hard_vs_human` does with `LEGATUM_HVH_RESEED_DICE=1`. One line per salt: the human
//! seat's turn-by-turn cities, Settlers, army and, for each idle city, what it weighed and the
//! Settler gates (`ScriptedBrain::production_diagnosis`, which leaves the brain untouched), plus
//! every empire's foundings and Settlers lost.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, Event, GameSetup, PlayerId, UnitKind, player_view, resolve_turn};
use rayon::prelude::*;
use serde_json::{Value, json};

fn salted(seed: u64, salt: u64) -> u64 {
    Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
}

/// Greedy best-first packing of sites more than the city gap apart.
fn pack(mut sites: Vec<(i32, legatum_core::Hex)>, gap: i32) -> usize {
    sites.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut picked: Vec<legatum_core::Hex> = Vec::new();
    for (_, h) in sites {
        if picked.iter().all(|p| p.distance(h) > gap) {
            picked.push(h);
        }
    }
    picked.len()
}

/// Sites within 8 hexes of the seat's cities at turns 40, 60 and 80: legal in the true state
/// and in the seat's view, any quality and at least 37 (Germany's floor) for each.
fn site_counts(state: &legatum_core::GameState, seat: PlayerId) -> Value {
    use legatum_core::settlement::{base_site_score, known_legal_site, legal_site};
    let faction = state.player(seat).faction;
    let gap = legatum_core::data::min_city_gap(faction);
    let mine: Vec<_> = state.cities_of(seat).map(|c| c.pos).collect();
    let near = |h: legatum_core::Hex| mine.iter().any(|c| c.distance(h) <= 8);
    let cities: Vec<_> = state.cities.values().map(|c| c.pos).collect();
    let truth: Vec<_> = state
        .map
        .hexes()
        .filter(|h| near(*h))
        .filter(|h| {
            state.map.get(*h).is_some_and(|t| {
                legal_site(
                    seat,
                    legatum_core::data::min_city_gap(faction),
                    t.terrain,
                    t.owner,
                    *h,
                    cities.iter().copied(),
                )
            })
        })
        .map(|h| {
            let mut y = legatum_core::Yields::ZERO;
            let mut foreign = 0;
            for n in h.within(legatum_core::data::CITY_WORK_RADIUS) {
                let Some(t) = state.map.get(n) else { continue };
                if t.owner.is_some_and(|o| o != seat) {
                    foreign += 1;
                } else if t.terrain.is_land() {
                    let tile = legatum_core::data::terrain_yields(faction, t.terrain)
                        + t.resource.map_or(legatum_core::Yields::ZERO, |r| {
                            legatum_core::resources::stats(r).bonus
                        });
                    y += if n.distance(h) >= legatum_core::data::CITY_WORK_RADIUS {
                        legatum_core::Yields::new(
                            tile.food / 2,
                            tile.production / 2,
                            tile.gold / 2,
                            tile.science / 2,
                        )
                    } else {
                        tile
                    };
                }
            }
            (
                legatum_core::economy::tile_score(y, legatum_core::CityFocus::Balanced)
                    - foreign * 8,
                h,
            )
        })
        .collect();
    let truth_good: Vec<_> = truth.iter().copied().filter(|(s, _)| *s >= 37).collect();
    let view = player_view(state, seat);
    let seen: Vec<_> = (0..view.tiles.len())
        .map(|i| state.map.hex_at(i))
        .filter(|h| near(*h) && known_legal_site(&view, *h))
        .map(|h| (base_site_score(&view, h), h))
        .collect();
    let seen_good: Vec<_> = seen.iter().copied().filter(|(s, _)| *s >= 37).collect();
    // Land within 8 the seat has never explored, and good legal sites (true score) centred on
    // it, and on explored tiles whose remembered owner differs from the truth.
    let unexplored = |h: legatum_core::Hex| view.tile(h).is_none_or(|t| !t.explored);
    let fog_land = state
        .map
        .hexes()
        .filter(|h| {
            near(*h) && state.map.get(*h).is_some_and(|t| t.terrain.is_land()) && unexplored(*h)
        })
        .count();
    let fog_sites: Vec<_> = truth_good
        .iter()
        .copied()
        .filter(|(_, h)| unexplored(*h))
        .collect();
    // Of those, reachable from our cities over passable land that is ours or unowned (true state),
    // and with a barbarian camp within 3.
    let mut reach = vec![false; state.map.tiles.len()];
    let mut queue: std::collections::VecDeque<legatum_core::Hex> = mine.iter().copied().collect();
    for h in &mine {
        if let Some(i) = state.map.index(*h) {
            reach[i] = true;
        }
    }
    while let Some(h) = queue.pop_front() {
        for n in h.neighbors() {
            let Some(i) = state.map.index(n) else {
                continue;
            };
            if !reach[i]
                && state.map.move_cost(n).is_some()
                && state
                    .map
                    .get(n)
                    .is_some_and(|t| t.owner.is_none_or(|o| o == seat))
            {
                reach[i] = true;
                queue.push_back(n);
            }
        }
    }
    let camps: Vec<legatum_core::Hex> = state
        .barbarians
        .as_ref()
        .map(|b| b.camps.iter().map(|c| c.pos).collect())
        .unwrap_or_default();
    let fog_reach = fog_sites
        .iter()
        .filter(|(_, h)| state.map.index(*h).is_some_and(|i| reach[i]))
        .count();
    let fog_camp = fog_sites
        .iter()
        .filter(|(_, h)| camps.iter().any(|c| c.distance(*h) <= 3))
        .count();
    let fog_hexes: Vec<_> = state
        .map
        .hexes()
        .filter(|h| {
            near(*h) && state.map.get(*h).is_some_and(|t| t.terrain.is_land()) && unexplored(*h)
        })
        .collect();
    json!([
        pack(truth, gap),
        pack(truth_good, gap),
        pack(seen, gap),
        pack(seen_good, gap),
        fog_land,
        fog_sites.len(),
        fog_hexes,
        mine,
        fog_reach,
        fog_camp
    ])
}

fn play(setup: &GameSetup, seat: PlayerId, salt: u64, turns: u32, params: &AiParams) -> Value {
    let mut setup = setup.clone();
    for p in setup.players.iter_mut() {
        p.is_human = false;
        p.difficulty = AiDifficulty::Hard;
    }
    let mut state = legatum_core::new_game(&setup);
    state.setup.seed = salted(state.setup.seed, salt ^ 0xD1CE);
    let mut brains: Vec<ScriptedBrain> = setup
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let id = PlayerId(i as u8);
            let seed = salted(legatum_wasm::session::brain_seed(setup.seed, id), salt);
            ScriptedBrain::with_params(p.faction, seed, id, AiDifficulty::Hard, params)
        })
        .collect();
    let mut rows = Vec::new();
    let mut founded: Vec<Value> = Vec::new();
    let mut lost: Vec<Value> = Vec::new();
    while !state.finished && state.turn <= turns {
        let view = player_view(&state, seat);
        let (_, trace) = brains[seat.0 as usize].production_diagnosis(&view);
        // Exposed rival Settlers in the seat's own view: at Peace, outside cities, no owner
        // military beside them; the nearest seat military unit's hex distance.
        let exposed: Vec<Value> = view
            .units
            .iter()
            .filter(|s| {
                s.kind == UnitKind::Settler
                    && s.owner != seat
                    && !s.owner.is_barbarian()
                    && !view.at_war(seat, s.owner)
                    && !view.cities.iter().any(|c| c.pos == s.pos)
                    && !view
                        .units
                        .iter()
                        .any(|u| u.military && u.owner == s.owner && u.pos.distance(s.pos) <= 1)
            })
            .filter_map(|s| {
                let d = view
                    .units
                    .iter()
                    .filter(|u| u.owner == seat && u.military)
                    .map(|u| u.pos.distance(s.pos))
                    .min()?;
                (d <= 4).then(|| json!([s.owner.0, d]))
            })
            .collect();
        let me = state.player(seat);
        rows.push(json!({
            "t": state.turn,
            "gold": me.gold,
            "prod": view.cities.iter().filter_map(|c| c.detail.as_ref()).map(|d| d.yields.total.production).sum::<i32>(),
            "food": view.cities.iter().filter_map(|c| c.detail.as_ref()).map(|d| d.yields.total.food).sum::<i32>(),
            "improved": state.map.tiles.iter().filter(|t| t.owner == Some(seat) && t.improved).count(),
            "pop": state.cities_of(seat).map(|c| c.population).sum::<i32>(),
            "cities": state.cities_of(seat).map(|c| json!({
                "id": c.id, "pos": c.pos, "pop": c.population, "producing": c.producing,
                "stored": c.production_stored,
            })).collect::<Vec<_>>(),
            "settlers": state.units.values().filter(|u| u.owner == seat && u.kind == UnitKind::Settler).count(),
            "units": state.units.values().filter(|u| u.owner == seat).map(|u| json!([u.kind, u.pos, u.hp, u.id, u.goto])).collect::<Vec<_>>(),
            "wars": view.players.iter().filter(|p| p.alive && view.at_war(seat, p.id)).map(|p| p.id.0).collect::<Vec<_>>(),
            "settler_notes": state.units.values().filter(|u| u.owner == seat && u.kind == UnitKind::Settler
                    && std::env::var_os("PROBE_SETTLER_NOTES").is_some())
                .map(|u| json!([u.id, brains[seat.0 as usize].settler_diagnosis(&view, u.id, None)]))
                .collect::<Vec<_>>(),
            "sites": if matches!(state.turn, 40 | 60 | 80) { site_counts(&state, seat) } else { Value::Null },
            "exposed": exposed,
            // To T40, every rival Settler outside a city in the true state: owner, hex distance
            // to the seat's nearest military unit and capital, whether the seat sees it, and
            // whether an owner troop stands beside it.
            "rival_settlers": if state.turn <= 40 {
                let cap = state.cities_of(seat).map(|c| c.pos).next();
                state.units.values().filter(|s| s.kind == UnitKind::Settler && s.owner != seat
                    && !s.owner.is_barbarian() && !state.cities.values().any(|c| c.pos == s.pos)).map(|s| {
                    let d = state.units.values().filter(|u| u.owner == seat && legatum_core::data::unit_stats(u.kind).military)
                        .map(|u| u.pos.distance(s.pos)).min();
                    let seen = view.units.iter().any(|u| u.id == s.id);
                    let esc = state.units.values().any(|u| u.owner == s.owner && legatum_core::data::unit_stats(u.kind).military && u.pos.distance(s.pos) <= 1);
                    json!([s.owner.0, d, cap.map(|c| c.distance(s.pos)), seen, esc])
                }).collect::<Vec<_>>()
            } else { Vec::new() },
            // Every 10th turn: true barbarian camps within 10 of the seat's cities, whether the
            // seat's view knows them, and the distance to its nearest city.
            "camps": if state.turn.is_multiple_of(10) {
                let mine: Vec<_> = state.cities_of(seat).map(|c| c.pos).collect();
                state.barbarians.as_ref().map_or(Vec::new(), |b| b.camps.iter().filter_map(|c| {
                    let d = mine.iter().map(|m| m.distance(c.pos)).min()?;
                    (d <= 10).then(|| json!([c.pos, view.barbarian_camps.iter().any(|k| k.pos == c.pos), d]))
                }).collect::<Vec<_>>())
            } else { Vec::new() },
            "idle": trace.iter().map(|t| {
                let mut top = t.options.clone();
                top.sort_by_key(|o| -o.2);
                top.truncate(4);
                json!({"city": t.city, "chosen": t.chosen, "top": top, "settler": t.settler})
            }).collect::<Vec<_>>(),
        }));
        let orders: legatum_wasm::session::OrderMap = brains
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| state.player(PlayerId(*i as u8)).alive)
            .map(|(i, b)| {
                let id = PlayerId(i as u8);
                (id, b.orders(&player_view(&state, id)))
            })
            .collect();
        let (next, events) = resolve_turn(&state, &orders);
        for e in &events {
            match e {
                Event::CityFounded { owner, at, .. } => {
                    founded.push(json!([state.turn, owner.0, at]));
                }
                Event::SettlerCaptured { unit, from, to, .. } => {
                    // The capture as it stood at the start of the turn (true state).
                    let before = state.units.get(unit);
                    let pos = before.map(|u| u.pos);
                    let near = |owner: PlayerId, r: i32| {
                        pos.map_or(0, |p| {
                            state
                                .units
                                .values()
                                .filter(|u| {
                                    u.owner == owner
                                        && legatum_core::data::unit_stats(u.kind).military
                                        && u.pos.distance(p) <= r
                                })
                                .count()
                        })
                    };
                    let captor = pos.and_then(|p| {
                        state
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == *to && legatum_core::data::unit_stats(u.kind).military
                            })
                            .min_by_key(|u| u.pos.distance(p))
                    });
                    let captor_d = captor.zip(pos).map(|(u, p)| u.pos.distance(p));
                    let captor_kind = captor.map(|u| u.kind);
                    let home_d =
                        pos.and_then(|p| state.cities_of(*from).map(|c| c.pos.distance(p)).min());
                    lost.push(json!({"t": state.turn, "from": from.0, "to": to.0, "how": "captured",
                        "peace_before": !state.at_war(*from, *to),
                        "escort1": near(*from, 1), "own2": near(*from, 2), "captor_d": captor_d, "captor_kind": captor_kind,
                        "home_d": home_d, "cities": state.cities_of(*from).count(),
                        "age": before.map(|u| state.turn.saturating_sub(u.created_turn))}));
                }
                Event::UnitDestroyed {
                    owner,
                    unit_kind: UnitKind::Settler,
                    ..
                } => {
                    lost.push(json!({"t": state.turn, "from": owner.0, "how": "destroyed"}));
                }
                _ => {}
            }
        }
        state = next;
    }
    let plans: Vec<Value> = brains[seat.0 as usize]
        .plan_events()
        .iter()
        .map(|e| json!(e))
        .collect();
    json!({"salt": salt, "plans": plans, "seat": seat.0, "factions": setup.players.iter().map(|p| p.faction).collect::<Vec<_>>(),
           "rows": rows, "founded": founded, "settlers_lost": lost})
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let game: Value = serde_json::from_str(outer["game"].as_str().unwrap()).unwrap();
    let rec: legatum_wasm::history::Recording =
        serde_json::from_value(game["archive"].clone()).unwrap();
    let setup: GameSetup = rec.tape.setup;
    let seat = PlayerId(setup.players.iter().position(|p| p.is_human).unwrap() as u8);
    let salts: u64 = a[2].parse().unwrap();
    let turns: u32 = a[3].parse().unwrap();
    let params = match a.get(4) {
        Some(path) => serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(),
        None => AiParams::builtin(),
    };
    let out: Vec<Value> = (1..=salts)
        .into_par_iter()
        .map(|salt| play(&setup, seat, salt, turns, &params))
        .collect();
    for row in out {
        println!("{row}");
    }
}
