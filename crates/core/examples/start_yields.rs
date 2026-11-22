//! Founding-territory yields of generated starts (the start tile plus ring 1).
//!
//! Usage: `cargo run --release -p legatum-core --example start_yields -- [maps] [width] [height]`
//!
//! Generates `maps` maps (seeds 1..=maps, 2-8 players cycling) and prints the spread of each
//! start's total Food and Production, counting visible bonus resources as a city would work them.

use legatum_core::resources::{is_strategic, stats};
use legatum_core::state::{AiDifficulty, GameSetup, PlayerSetup};
use legatum_core::types::{Faction, TeamId, Yields};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<i32> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let maps = args.first().copied().unwrap_or(1000) as u64;
    let (w, h) = (
        args.get(1).copied().unwrap_or(48),
        args.get(2).copied().unwrap_or(32),
    );
    let (mut foods, mut prods) = (Vec::new(), Vec::new());
    let (mut spread_f, mut spread_p) = (Vec::new(), Vec::new());
    let mut pairs: BTreeMap<(i32, i32), usize> = BTreeMap::new();
    for seed in 1..=maps {
        let players = 2 + (seed % 7) as usize;
        let setup = GameSetup {
            seed,
            width: w,
            height: h,
            players: (0..players)
                .map(|i| PlayerSetup {
                    name: format!("P{i}"),
                    faction: Faction::ALL[i % Faction::ALL.len()],
                    team: TeamId(i as u8),
                    is_human: i == 0,
                    difficulty: AiDifficulty::Normal,
                    search: false,
                })
                .collect(),
            turn_limit: Some(150),
            legacy_all_war: false,
            barbarians: false,
            rule_experiments: Default::default(),
        };
        let (map, starts) = legatum_core::mapgen::generate(&setup);
        let (mut lo_f, mut hi_f, mut lo_p, mut hi_p) = (i32::MAX, 0, i32::MAX, 0);
        for s in &starts {
            let mut y = Yields::ZERO;
            for hex in s.within(1) {
                let tile = map.get(hex).unwrap();
                y += tile.terrain.yields();
                if let Some(r) = tile.resource.filter(|r| !is_strategic(*r)) {
                    y += stats(r).bonus;
                }
            }
            foods.push(y.food);
            prods.push(y.production);
            *pairs.entry((y.food, y.production)).or_default() += 1;
            (lo_f, hi_f) = (lo_f.min(y.food), hi_f.max(y.food));
            (lo_p, hi_p) = (lo_p.min(y.production), hi_p.max(y.production));
        }
        spread_f.push(hi_f - lo_f);
        spread_p.push(hi_p - lo_p);
    }
    let pct = |v: &mut Vec<i32>, p: usize| {
        v.sort();
        v[(v.len() - 1) * p / 100]
    };
    for (name, v) in [
        ("Food", &mut foods),
        ("Production", &mut prods),
        ("within-map Food spread", &mut spread_f),
        ("within-map Production spread", &mut spread_p),
    ] {
        println!(
            "{name}: min {} p10 {} p25 {} p50 {} p75 {} p90 {} max {}",
            pct(v, 0),
            pct(v, 10),
            pct(v, 25),
            pct(v, 50),
            pct(v, 75),
            pct(v, 90),
            pct(v, 100)
        );
    }
    let total: usize = pairs.values().sum();
    println!("starts: {total}; (Food, Production) -> count");
    for ((f, p), n) in pairs {
        println!("  ({f},{p}): {n}");
    }
}
