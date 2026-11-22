//! Replay a scripted-AI tape and record expansion facts per empire and turn, read-only:
//! expansion_trace <tape.json> <out.jsonl> [diagnose=1|0|light]
//! One JSON object per AI empire per turn (state before that turn's orders resolve): cities,
//! population and Production per city, Settler cost, Settlers alive, legal free sites (true
//! state, greedy non-overlapping packing: anywhere, and within 10 hexes of the empire), the
//! sites the empire's own fogged view rates as good, and, with diagnose=1, what every idle city
//! weighed and chose plus each Settler gate (`ScriptedBrain::production_diagnosis`). A final
//! `{"events": ...}` line lists foundings, captures, razes, completions and eliminations.
//! Diagnosis brains are rebuilt like the simulator's (scripted, seat difficulty, brain seed);
//! `order_mismatches` counts turns where they did not reproduce the tape.
use legatum_core::data::{self, MIN_CITY_GAP};
use legatum_core::economy::{city_production_cost, city_yields, tile_score};
use legatum_core::settlement::{base_site_score, border_city_risks, known_legal_site, legal_site};
use legatum_core::{
    CityFocus, Event, GameState, Hex, PlayerId, ProductionItem, UnitKind, player_view,
    resolve_turn, resources,
};
use legatum_wasm::session::brain_seed;
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::io::Write;

const NEAR: i32 = 10;
const GOOD_SITE: i32 = 60;

/// Greedy packing of candidate sites, best first, kept more than MIN_CITY_GAP apart.
fn pack(mut sites: Vec<(i32, Hex)>) -> Vec<(i32, Hex)> {
    sites.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut picked: Vec<(i32, Hex)> = Vec::new();
    for (s, h) in sites {
        if picked.iter().all(|(_, p)| p.distance(h) > MIN_CITY_GAP) {
            picked.push((s, h));
        }
    }
    picked
}

/// Omniscient site score: the shared Balanced tile score of the land a city here could work.
fn true_site_score(state: &GameState, me: PlayerId, pos: Hex) -> i32 {
    let faction = state.player(me).faction;
    let mut y = legatum_core::Yields::ZERO;
    let mut foreign = 0;
    for h in pos.within(data::CITY_WORK_RADIUS) {
        let Some(t) = state.map.get(h) else { continue };
        if t.owner.is_some_and(|o| o != me) {
            foreign += 1;
            continue;
        }
        if t.terrain.is_land() {
            y += data::terrain_yields(faction, t.terrain)
                + t.resource
                    .map_or(legatum_core::Yields::ZERO, |r| resources::stats(r).bonus);
        }
    }
    tile_score(y, CityFocus::Balanced) - foreign * 8
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let mut out = std::io::BufWriter::new(std::fs::File::create(&a[2])?);
    let diagnose = a.get(3).is_none_or(|x| x != "0");
    let light = a.get(3).is_some_and(|x| x == "light");
    let mut state = tape.initial_state();
    if state.hash() != tape.initial_hash {
        return Err("tape initial hash does not match this rules build".into());
    }
    let mut mismatches = 0u32;
    let mut events_out = Vec::new();
    for turn in &tape.turns {
        let city_hexes: Vec<Hex> = state.cities.values().map(|c| c.pos).collect();
        for p in &state.players {
            if !p.alive || p.id.is_barbarian() {
                continue;
            }
            let me = p.id;
            if light {
                // Turn 1 preserves every seat's faction even if it is eliminated
                // before the first measurement point.
                if [1, 50, 100, 150, 200].contains(&turn.turn) {
                    let city_count = state.cities_of(me).count();
                    serde_json::to_writer(
                        &mut out,
                        &json!({"t": turn.turn, "p": me.0, "f": p.faction,
                            "cities": vec![0; city_count]}),
                    )?;
                    out.write_all(b"\n")?;
                }
                continue;
            }
            let mine: Vec<_> = state.cities_of(me).collect();
            let legal: Vec<(i32, Hex)> = state
                .map
                .hexes()
                .filter_map(|h| {
                    let t = state.map.get(h)?;
                    legal_site(
                        me,
                        state.setup.rule_experiments.min_city_gap(p.faction),
                        t.terrain,
                        t.owner,
                        h,
                        city_hexes.iter().copied(),
                    )
                    .then(|| (true_site_score(&state, me, h), h))
                })
                .collect();
            let near = |h: Hex| mine.iter().any(|c| c.pos.distance(h) <= NEAR);
            let all_sites = pack(legal.clone());
            let good_sites = pack(
                legal
                    .iter()
                    .copied()
                    .filter(|(s, _)| *s >= GOOD_SITE)
                    .collect(),
            );
            let near_sites = pack(legal.iter().copied().filter(|(_, h)| near(*h)).collect());
            let near_good = pack(
                legal
                    .iter()
                    .copied()
                    .filter(|(s, h)| *s >= GOOD_SITE && near(*h))
                    .collect(),
            );
            let view = player_view(&state, me);
            let risks = border_city_risks(&view);
            let seen_good = pack(
                (0..view.tiles.len())
                    .filter_map(|i| {
                        let h = state.map.hex_at(i);
                        (!risks[i] && known_legal_site(&view, h) && near(h))
                            .then(|| (base_site_score(&view, h), h))
                            .filter(|(s, _)| *s >= GOOD_SITE)
                    })
                    .collect(),
            );
            let settler = ProductionItem::Unit {
                unit: UnitKind::Settler,
            };
            let cities: Vec<_> = mine
                .iter()
                .map(|c| {
                    let y = city_yields(&state, c).total;
                    json!({"id": c.id.0, "pop": c.population, "prod": y.production,
                        "food": y.food, "producing": c.producing, "stored": c.production_stored,
                        "cost": c.producing.map(|i| city_production_cost(&state, c, i)),
                        "settler_cost": city_production_cost(&state, c, settler),
                        "resistance": c.resistance, "capital": c.is_capital,
                        "founder": c.founder.0, "buildings": c.buildings.len()})
                })
                .collect();
            let settlers = state
                .units_of(me)
                .filter(|u| u.kind == UnitKind::Settler)
                .count();
            let military = state
                .units_of(me)
                .filter(|u| data::unit_stats(u.kind).military && u.kind != UnitKind::Scout)
                .count();
            let mut row = json!({"t": turn.turn, "p": me.0, "f": p.faction, "human": p.is_human, "cities": cities,
                "settlers": settlers, "military": military, "gold": p.gold,
                "trained": p.settlers_trained,
                "sites_all": all_sites.len(), "sites_good": good_sites.len(),
                "sites_near": near_sites.len(), "sites_near_good": near_good.len(),
                "sites_seen_good": seen_good.len(),
                "at_war": state.players.iter().any(|o| o.id != me && o.alive
                    && !o.id.is_barbarian() && state.at_war(me, o.id))});
            if diagnose && !p.is_human {
                let difficulty = state
                    .setup
                    .players
                    .get(me.0 as usize)
                    .map(|s| s.difficulty)
                    .unwrap_or_default();
                let brain = legatum_ai::ScriptedBrain::with_difficulty(
                    p.faction,
                    brain_seed(state.setup.seed, me),
                    me,
                    difficulty,
                );
                let (orders, trace) = brain.production_diagnosis(&view);
                let taped = turn.orders.get(&me).cloned().unwrap_or_default();
                if serde_json::to_string(&orders)? != serde_json::to_string(&taped)? {
                    mismatches += 1;
                }
                let set: Vec<_> = orders
                    .iter()
                    .filter_map(|o| match o {
                        legatum_core::Order::SetProduction { city, item } => {
                            Some(json!([city.0, item]))
                        }
                        legatum_core::Order::BuyItem { city, item } => {
                            Some(json!([city.0, item, "buy"]))
                        }
                        _ => None,
                    })
                    .collect();
                let trace: Vec<_> = trace
                    .iter()
                    .map(|t| {
                        let mut opts = t.options.clone();
                        opts.sort_by_key(|a| std::cmp::Reverse(a.2));
                        let settler_opt = t.options.iter().find(|o| o.0 == settler).copied();
                        json!({"city": t.city.0, "chosen": t.chosen, "settler": t.settler,
                            "settler_opt": settler_opt,
                            "top": opts.into_iter().take(3).collect::<Vec<_>>()})
                    })
                    .collect();
                row["orders"] = json!(set);
                row["trace"] = json!(trace);
            }
            serde_json::to_writer(&mut out, &row)?;
            out.write_all(b"\n")?;
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        if next.hash().to_string() != turn.hash {
            return Err(format!("tape hash differs after turn {}", turn.turn).into());
        }
        for e in &events {
            match e {
                Event::CityFounded { .. }
                | Event::CityCaptured { .. }
                | Event::CityRazed { .. }
                | Event::SettlerCaptured { .. }
                | Event::PlayerEliminated { .. }
                | Event::GameWon { .. }
                | Event::GameDrawn { .. }
                | Event::ProductionCompleted { .. }
                | Event::ItemBought { .. }
                | Event::WarDeclared { .. }
                | Event::PeaceEstablished { .. } => {
                    events_out.push(json!({"t": turn.turn, "e": e}));
                }
                Event::UnitDestroyed { unit_kind, .. } if *unit_kind == UnitKind::Settler => {
                    events_out.push(json!({"t": turn.turn, "e": e}));
                }
                _ => {}
            }
        }
        state = next;
    }
    let winner: Vec<_> = state
        .players
        .iter()
        .filter(|p| p.alive && !p.id.is_barbarian())
        .map(|p| json!({"p": p.id.0, "f": p.faction, "cities": state.cities_of(p.id).count()}))
        .collect();
    serde_json::to_writer(
        &mut out,
        &json!({"events": events_out, "final_turn": state.turn, "alive": winner,
            "map": [state.map.width, state.map.height],
            "land": state.map.tiles.iter().filter(|t| t.terrain.is_land()
                && t.terrain.move_cost().is_some()).count(),
            "players": state.players.iter().filter(|p| !p.id.is_barbarian()).count(),
            "order_mismatches": mismatches}),
    )?;
    out.write_all(b"\n")?;
    eprintln!("{}: order mismatches {mismatches}", a[1]);
    Ok(())
}
