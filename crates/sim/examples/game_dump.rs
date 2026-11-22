//! Replay a tape and dump everything needed for a post-game analysis:
//! game_dump <tape.json> <out-dir>
//! Writes <out-dir>/events.jsonl ({turn, event} for every event), <out-dir>/snaps.jsonl (per
//! turn, after resolution: the state minus map/fog, plus public metrics, VP breakdowns and
//! territory), and <out-dir>/sites.jsonl (each city founding's site score from the founder's
//! view the turn before). Reports hash mismatches.
use legatum_core::settlement::{base_site_score, site_potential, site_strategic};
use legatum_core::{Event, player_view, resolve_turn, scoring};
use legatum_wasm::tape::Tape;
use serde_json::{Value, json};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let out = std::path::Path::new(&a[2]);
    std::fs::create_dir_all(out)?;
    let mut ev = std::io::BufWriter::new(std::fs::File::create(out.join("events.jsonl"))?);
    let mut sn = std::io::BufWriter::new(std::fs::File::create(out.join("snaps.jsonl"))?);
    let mut si = std::io::BufWriter::new(std::fs::File::create(out.join("sites.jsonl"))?);
    let mut state = tape.initial_state();
    println!(
        "initial hash matches: {}",
        state.hash() == tape.initial_hash
    );
    let mut mismatches = 0;
    for turn in &tape.turns {
        let (next, events) = resolve_turn(&state, &turn.orders);
        if next.hash().to_string() != turn.hash {
            mismatches += 1;
            if mismatches == 1 {
                println!("first hash mismatch at turn {}", turn.turn);
            }
        }
        for e in &events {
            writeln!(ev, "{}", json!({"turn": turn.turn, "event": e}))?;
            if let Event::CityFounded {
                owner, at, city, ..
            } = e
            {
                let view = player_view(&state, *owner);
                let (y, foreign) = site_potential(&view, *at);
                // The founder's capital, scored from the same view, for scale.
                let cap = state
                    .capital_of(*owner)
                    .map(|c| base_site_score(&view, c.pos));
                writeln!(
                    si,
                    "{}",
                    json!({"turn": turn.turn, "owner": owner, "city": city, "at": at,
                        "score": base_site_score(&view, *at), "yields": y, "foreign": foreign,
                        "strategic": site_strategic(&view, *at), "capital_score": cap})
                )?;
            }
        }
        let mut v = serde_json::to_value(&next)?;
        if let Value::Object(m) = &mut v {
            m.remove("map");
            m.remove("archive");
            if let Some(Value::Array(ps)) = m.get_mut("players") {
                for p in ps {
                    if let Value::Object(pm) = p {
                        pm.remove("tile_memory");
                        pm.remove("explored");
                    }
                }
            }
        }
        let vp: Vec<Value> = next
            .players
            .iter()
            .map(|p| json!(scoring::vp(&next, p.id)))
            .collect();
        let metrics: Vec<Value> = [
            ("economy", scoring::economy_value as fn(&_, _) -> i32),
            ("army", scoring::army_strength),
            ("advancement", scoring::advancement),
        ]
        .into_iter()
        .map(|(name, metric)| {
            let values: Vec<(legatum_core::PlayerId, i32)> = next
                .players
                .iter()
                .filter(|p| p.alive)
                .map(|p| (p.id, metric(&next, p.id)))
                .collect();
            json!({"metric": name, "values": values})
        })
        .collect();
        writeln!(
            sn,
            "{}",
            json!({"turn": turn.turn, "state": v, "vp": vp, "metrics": metrics})
        )?;
        state = next;
    }
    println!("turns {} hash mismatches {}", tape.turns.len(), mismatches);
    Ok(())
}
