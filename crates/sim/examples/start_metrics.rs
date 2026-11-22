//! Measure start fairness over a fixed Standard, eight-seat seed range.
use legatum_core::mapgen::{generate_counted, start_quality};
use legatum_core::resources::{is_strategic, stats};
use legatum_core::state::Map;
use legatum_core::{Faction, GameSetup, Hex};

fn production(map: &Map, at: Hex) -> (usize, i64) {
    at.within(2)
        .into_iter()
        .filter_map(|h| map.get(h))
        .filter(|t| t.terrain.move_cost().is_some())
        .map(|t| {
            let bonus = t
                .resource
                .filter(|r| !is_strategic(*r))
                .map_or(0, |r| stats(r).bonus.production);
            let amount = t.terrain.yields().production + bonus;
            (usize::from(amount >= 2), i64::from(amount))
        })
        .fold((0, 0), |(count, total), (tile, value)| {
            (count + tile, total + value)
        })
}

/// Food plus Production over radius 3, radius 2 counting triple: the main term of the old start
/// quality, so spreads stay comparable across rule versions.
fn yield_total(map: &Map, at: Hex) -> i64 {
    at.within(3)
        .into_iter()
        .filter_map(|h| map.get(h).map(|t| (h, t)))
        .filter(|(_, t)| t.terrain.move_cost().is_some())
        .map(|(h, t)| {
            let bonus = t
                .resource
                .filter(|r| !is_strategic(*r))
                .map_or(0, |r| stats(r).bonus.food + stats(r).bonus.production);
            let y = t.terrain.yields();
            let weight = if at.distance(h) <= 2 { 3 } else { 1 };
            weight * (i64::from(y.food + y.production) + i64::from(bonus))
        })
        .sum()
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let value = |flag: &str, default: &str| -> String {
        args.windows(2)
            .find(|pair| pair[0] == flag)
            .map_or(default.to_string(), |pair| pair[1].clone())
    };
    let base: u64 = value("--seed", "0").parse().unwrap();
    let count: u64 = value("--games", "2000").parse().unwrap();
    let mut spread = 0i64;
    let mut max_spread = 0;
    let mut yield_spread = 0i64;
    let mut low_tiles = 0;
    let mut retries = 0u64;
    let mut totals = [(0i64, 0u64); 9];
    for seed in base..base + count {
        let mut setup = GameSetup::all_ai(seed);
        // The runner's draw: one seat per faction, shuffled independently of the map.
        let mut factions = Faction::ALL;
        let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
        for i in (1..factions.len()).rev() {
            let j = rng.range(0, i as i64) as usize;
            factions.swap(i, j);
        }
        for (player, faction) in setup.players.iter_mut().zip(factions) {
            player.faction = faction;
        }
        let ((map, starts), attempts) = generate_counted(&setup);
        retries += attempts;
        let quality: Vec<_> = (0..8)
            .map(|seat| start_quality(&map, &starts, &setup, seat))
            .collect();
        let width = quality.iter().max().unwrap() - quality.iter().min().unwrap();
        let yields: Vec<_> = starts.iter().map(|&at| yield_total(&map, at)).collect();
        yield_spread += yields.iter().max().unwrap() - yields.iter().min().unwrap();
        spread += i64::from(width);
        max_spread = max_spread.max(width);
        for (seat, &at) in starts.iter().enumerate() {
            let (tiles, prod) = production(&map, at);
            low_tiles += u64::from(tiles < 2);
            let faction = setup.players[seat].faction;
            let index = Faction::ALL.iter().position(|f| *f == faction).unwrap();
            totals[index].0 += prod;
            totals[index].1 += 1;
        }
    }
    let mut output = format!(
        "seeds {base}..{}; starts {}; mean quality spread {:.3}; max spread {max_spread}; mean yield spread {:.3}; fewer than 2 production tiles {low_tiles} ({:.3}%); retries {retries}\n",
        base + count - 1,
        count * 8,
        spread as f64 / count as f64,
        yield_spread as f64 / count as f64,
        100.0 * low_tiles as f64 / (count * 8) as f64
    );
    for (faction, (production, starts)) in Faction::ALL.into_iter().zip(totals) {
        output.push_str(&format!(
            "{faction:?}: mean radius-2 Production {:.3} ({starts} starts)\n",
            production as f64 / starts as f64
        ));
    }
    print!("{output}");
    if let Some(dir) = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| &pair[1])
    {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(std::path::Path::new(dir).join("start-metrics.txt"), output).unwrap();
    }
}
