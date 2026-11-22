//! Observer-only, hash-checked diagnosis of archived human-playbook games.
//! Usage: hard_defense_audit ARCHIVE_ROOT_OR_FILE BASE_SEED [THREADS] [details]
use legatum_core::{Event, Order, PlayerId, ProductionItem, UnitKind, player_view, resolve_turn};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
};

fn audit(path: &Path, base: u64, details: bool) -> Result<Value, String> {
    let recording = legatum_archive::read(path)?;
    recording.compatible()?;
    let tape = &recording.tape;
    let proxy = PlayerId(((tape.setup.seed - base) % 8) as u8);
    let mut state = match &recording.initial_state {
        Some(s) => legatum_wasm::json::parse_state(s)?,
        None => tape.initial_state(),
    };
    if state.hash() != tape.initial_hash {
        return Err("initial hash mismatch".into());
    }
    let mut captures = Vec::new();
    let mut settlers = Vec::new();
    let mut warnings = BTreeMap::new();
    let mut history = BTreeMap::<_, VecDeque<Value>>::new();
    let mut first_losses = BTreeMap::new();
    let mut recaptures = Vec::new();
    let mut recapture_attacks = Vec::new();
    let mut leader_peaces = Vec::new();
    let mut counterfire = BTreeSet::new();
    let mut withheld_counterfire = BTreeSet::new();
    let mut withheld_wounded_counterfire = BTreeSet::new();
    let mut snowball_turns = 0;
    let mut snowball_no_coalition = 0;
    for t in &tape.turns {
        if first_losses.len() >= 2 && state.cities_of(proxy).count() >= 5 {
            let pv = player_view(&state, proxy);
            snowball_turns += 1;
            if legatum_core::diplomacy::coalition::target_view(&pv) != Some(pv.team) {
                snowball_no_coalition += 1;
            }
        }
        let mut views = BTreeMap::new();
        let mut current_warnings = BTreeMap::new();
        // Observer prefilter only: if fewer than two real units are nearby, a
        // player's view cannot reveal two. Decisions never receive this observer.
        for c in state.cities.values().filter(|c| {
            c.owner != proxy
                && state
                    .units_of(proxy)
                    .filter(|u| {
                        legatum_core::data::unit_stats(u.kind).military
                            && u.kind != UnitKind::Scout
                            && u.pos.distance(c.pos) <= 5
                    })
                    .count()
                    >= 2
        }) {
            let v = views
                .entry(c.owner)
                .or_insert_with(|| player_view(&state, c.owner));
            let pressure = v
                .units
                .iter()
                .filter(|u| {
                    u.owner == proxy
                        && u.military
                        && u.kind != UnitKind::Scout
                        && u.pos.distance(c.pos) <= 5
                })
                .count();
            if pressure >= 2 {
                current_warnings.insert(
                    (c.id, c.owner),
                    *warnings.get(&(c.id, c.owner)).unwrap_or(&t.turn),
                );
                for u in v
                    .units
                    .iter()
                    .filter(|u| u.owner == c.owner && u.ranged > 0 && u.pos.distance(c.pos) <= 3)
                {
                    if !v.units.iter().any(|e| {
                        e.owner == proxy
                            && e.military
                            && legatum_core::forecast_attack_mode(
                                v,
                                u.id,
                                e.pos,
                                legatum_core::AttackMode::Ranged,
                            )
                            .is_some()
                    }) {
                        continue;
                    }
                    let key = (t.turn, u.id);
                    counterfire.insert(key);
                    if !t.orders.get(&c.owner).is_some_and(|orders| {
                        orders
                            .iter()
                            .any(|o| matches!(o, Order::RangedAttack { unit, .. } if *unit == u.id))
                    }) {
                        withheld_counterfire.insert(key);
                        if u.hp < 60 {
                            withheld_wounded_counterfire.insert(key);
                        }
                    }
                }
                if details {
                    let units: Vec<_> = v
                        .units
                        .iter()
                        .filter(|u| u.owner == c.owner && u.pos.distance(c.pos) <= 8)
                        .collect();
                    let cities: Vec<_> = v.cities.iter().filter(|x| x.owner == c.owner).filter_map(|x| {
                        let d = x.detail.as_ref()?;
                        Some(json!({"city":x.id,"pos":x.pos,"producing":d.producing,"turns":d.turns_to_complete,"production":d.yields.total.production}))
                    }).collect();
                    let past = history.entry((c.id, c.owner)).or_default();
                    past.push_back(json!({"turn":t.turn,"units":units,"cities":cities,"gold":v.economy.gold,"income":v.economy.gold_per_turn.total,"orders":t.orders.get(&c.owner)}));
                    if past.len() > 10 {
                        past.pop_front();
                    }
                }
            }
        }
        warnings = current_warnings;
        let (next, events) = resolve_turn(&state, &t.orders);
        if next.hash().to_string() != t.hash {
            return Err(format!(
                "seed {} turn {} hash mismatch",
                tape.setup.seed, t.turn
            ));
        }
        let mut attacked = BTreeSet::new();
        for e in events {
            match e {
                Event::PeaceEstablished { relation }
                    if (relation.a == state.team_of(proxy)
                        || relation.b == state.team_of(proxy))
                        && legatum_core::diplomacy::coalition::target_state(&state)
                            == Some(state.team_of(proxy))
                        && legatum_core::diplomacy::coalition::target_state(&next)
                            == Some(state.team_of(proxy)) =>
                {
                    let other = if relation.a == state.team_of(proxy) {
                        relation.b
                    } else {
                        relation.a
                    };
                    if let Some(player) =
                        state.players.iter().find(|p| state.team_of(p.id) == other)
                    {
                        let v = player_view(&state, player.id);
                        let nearby = v.units.iter().any(|u| {
                            u.owner == proxy
                                && u.military
                                && v.cities
                                    .iter()
                                    .any(|c| c.owner == player.id && c.pos.distance(u.pos) <= 5)
                        });
                        let damaged = v.cities.iter().any(|c| c.owner == player.id && c.hp <= 100);
                        leader_peaces.push(json!({"turn":t.turn,"other":other,
                            "visible_army_near_home":nearby,"damaged_home":damaged}));
                    }
                }
                Event::CityCaptured { city, from, to, .. } if to == proxy => {
                    let c = &state.cities[&city];
                    let v = player_view(&state, from);
                    let cv = v
                        .cities
                        .iter()
                        .find(|x| x.id == city)
                        .ok_or("missing own city")?;
                    let local = v
                        .units
                        .iter()
                        .filter(|u| {
                            u.owner == from
                                && u.military
                                && u.kind != UnitKind::Scout
                                && u.pos.distance(c.pos) <= 2
                        })
                        .count();
                    let garrison = v
                        .units
                        .iter()
                        .find(|u| u.owner == from && u.military && u.pos == c.pos);
                    let bought = t.orders.get(&from).is_some_and(|os| {
                        os.iter()
                            .any(|o| matches!(o, Order::BuyItem {city: id, ..} if *id == city))
                    });
                    let affordable = cv.detail.as_ref().is_some_and(|d| {
                        d.options.iter().any(|o| {
                            matches!(
                                o.item,
                                ProductionItem::Unit {
                                    unit: UnitKind::Soldier
                                        | UnitKind::Trooper
                                        | UnitKind::AssaultTrooper
                                        | UnitKind::Ranger
                                        | UnitKind::Marksman
                                }
                            ) && o.buy_blocker.is_none()
                                && o.buy_cost
                                    .is_some_and(|cost| cost > 0 && cost <= v.economy.gold)
                        })
                    });
                    let isolated = state
                        .cities_of(from)
                        .filter(|x| x.id != city)
                        .map(|x| x.pos.distance(c.pos))
                        .min();
                    let war_age = v
                        .diplomacy
                        .relations
                        .iter()
                        .find(|r| r.between(v.team, state.team_of(proxy)))
                        .map(|r| t.turn.saturating_sub(r.effective_turn));
                    captures.push(json!({"turn":t.turn,"city":city,"from":from,"hp":cv.hp,"garrison_hp":garrison.map(|u|u.hp),"local_defenders":local,"nearest_other_city":isolated,"gold":v.economy.gold,"affordable_defender":affordable,"bought":bought,"warning_age":warnings.get(&(city,from)).map(|turn|t.turn-turn),"war_age":war_age}));
                    if details {
                        captures.last_mut().unwrap()["warning_history"] =
                            json!(history.get(&(city, from)));
                    }
                    first_losses.entry(city).or_insert((t.turn, from));
                }
                Event::CityCaptured { city, from, to, .. } if from == proxy => {
                    recaptures.push(json!({"turn":t.turn,"city":city,"to":to,"original_loss":first_losses.get(&city)}));
                }
                Event::SettlerCaptured { unit, from, to, at } if to == proxy => {
                    let v = player_view(&state, from);
                    let s = state.units.get(&unit);
                    settlers.push(json!({"turn":t.turn,"from":from,"at":at,"peace":!v.at_war(from,proxy),"visible_raider":v.units.iter().any(|u|u.owner==proxy && u.military && s.is_some_and(|s|u.pos.distance(s.pos)<=u.max_moves+1)),"escort":s.is_some_and(|s|v.units.iter().any(|u|u.owner==from && u.military && u.pos.distance(s.pos)<=1))}));
                }
                Event::Battle {
                    at, attacker_owner, ..
                } if attacker_owner != proxy && !attacker_owner.is_barbarian() => {
                    if let Some(c) = state
                        .cities_of(proxy)
                        .find(|c| c.pos == at && first_losses.contains_key(&c.id))
                    {
                        attacked.insert((c.id, attacker_owner));
                    }
                }
                _ => {}
            }
        }
        for (city, player) in attacked {
            recapture_attacks.push(json!({"turn":t.turn,"city":city,"player":player,"recovered":next.cities.get(&city).is_some_and(|c|c.owner!=proxy)}));
        }
        state = next;
    }
    Ok(
        json!({"seed":tape.setup.seed.to_string(),"proxy":proxy,"win":state.winner==Some(state.team_of(proxy)),"turns":tape.turns.len(),"captures":captures,"settlers":settlers,"recaptures":recaptures,"recapture_attacks":recapture_attacks,"leader_peaces":leader_peaces,"counterfire_opportunities":counterfire.len(),"withheld_counterfire":withheld_counterfire.len(),"withheld_wounded_counterfire":withheld_wounded_counterfire.len(),"snowball_turns":snowball_turns,"snowball_no_coalition":snowball_no_coalition}),
    )
}

fn paths(root: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    if root.is_file() {
        out.push(root.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            paths(&path, out)?;
        } else if path.to_string_lossy().ends_with(".tape.json.zst") {
            out.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let root = args.get(1).ok_or("expected archive root")?;
    let base = args
        .get(2)
        .ok_or("expected base seed")?
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let threads = args
        .get(3)
        .map_or(Ok(4), |s| s.parse::<usize>())
        .map_err(|e| e.to_string())?;
    let details = args.get(4).is_some_and(|arg| arg == "details");
    let mut files = Vec::new();
    paths(Path::new(root), &mut files)?;
    files.sort();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.clamp(1, 8))
        .build()
        .map_err(|e| e.to_string())?;
    let done = std::sync::atomic::AtomicUsize::new(0);
    let mut rows = pool.install(|| {
        files
            .par_iter()
            .map(|p| {
                let result = audit(p, base, details);
                let count = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if count.is_multiple_of(40) {
                    eprintln!("{count}/{} replays", files.len());
                }
                result
            })
            .collect::<Result<Vec<_>, _>>()
    })?;
    rows.sort_by_key(|r| r["seed"].as_str().unwrap().parse::<u64>().unwrap());
    println!(
        "{}",
        serde_json::to_string(&rows).map_err(|e| e.to_string())?
    );
    Ok(())
}
