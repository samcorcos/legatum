//! Army-strength calibration uses the lab's field arena and Human policy, on real tactical turns.
use super::*;
use legatum_core::{combat, rules::promotion};
use rayon::prelude::*;
use serde_json::json;

const LINES: [[UnitKind; 3]; 7] = [
    [UnitKind::Soldier, UnitKind::Ranger, UnitKind::Horseman],
    [
        UnitKind::Phalangite,
        UnitKind::CompositeArcher,
        UnitKind::Cataphract,
    ],
    [UnitKind::Trooper, UnitKind::Marksman, UnitKind::Knight],
    [
        UnitKind::AssaultTrooper,
        UnitKind::Arquebusier,
        UnitKind::Cuirassier,
    ],
    [UnitKind::Rifleman, UnitKind::FieldCannon, UnitKind::Cavalry],
    [UnitKind::Infantry, UnitKind::Artillery, UnitKind::Tank],
    [
        UnitKind::HunterKiller,
        UnitKind::Drone,
        UnitKind::AutonomousArmour,
    ],
];
const CANDIDATES: [&str; 10] = [
    "old",
    "square_max",
    "cube_max",
    "exp25_max",
    "exp40_max",
    "exp50_max",
    "square_mean",
    "cube_mean",
    "exp40_mean",
    "shipped",
];

fn values(s: &GameState, side: PlayerId) -> Vec<f64> {
    let mut sums = vec![0.0; CANDIDATES.len()];
    for u in s.units_of(side) {
        let f = s.player(side).faction;
        let stats = data::unit_stats_for(f, u.kind);
        if !stats.military {
            continue;
        }
        let level = promotion::level_for(f, u.xp);
        let melee = combat::unit_attack_total(f, u.kind, u.hp, level).max(0) as f64;
        let ranged = combat::unit_ranged_total(f, u.kind, u.hp, level).max(0) as f64;
        let high = melee.max(ranged);
        let mean = if stats.ranged > 0 {
            (melee + ranged) / 2.0
        } else {
            melee
        };
        let hp = f64::from(u.hp) / 100.0;
        let v = [
            f64::from(stats.strength.max(stats.ranged) * u.hp / 100),
            20.0 * (high / 20.0).powi(2) * hp,
            20.0 * (high / 20.0).powi(3) * hp,
            20.0 * ((high - 20.0) / 25.0).exp() * hp,
            20.0 * ((high - 20.0) / 40.0).exp() * hp,
            20.0 * ((high - 20.0) / 50.0).exp() * hp,
            20.0 * (mean / 20.0).powi(2) * hp,
            20.0 * (mean / 20.0).powi(3) * hp,
            20.0 * ((mean - 20.0) / 40.0).exp() * hp,
            f64::from(legatum_core::army::unit_value(s, u)),
        ];
        for (sum, v) in sums.iter_mut().zip(v) {
            *sum += v;
        }
    }
    sums
}

pub(super) fn run(args: &[String]) {
    let repeats: u64 = arg(args, "--games", "8").parse().unwrap();
    let seed0: u64 = arg(args, "--seed", "102500").parse().unwrap();
    let mut grid = Vec::new();
    for (era, line) in LINES.iter().enumerate() {
        for (old, earlier) in LINES.iter().enumerate().take(era + 1) {
            for (a, &ak) in line.iter().enumerate() {
                for (b, &bk) in earlier.iter().enumerate() {
                    for n in [2, 4, 8] {
                        for distance in [1, 3] {
                            grid.push(case(
                                &format!("e{era}-{a}-e{old}-{b}-4v{n}-d{distance}"),
                                if era == old { "same_era" } else { "era_gap" },
                                Goal::Destroy,
                                Ground::Open,
                                distance,
                                vec![ak; 4],
                                vec![bk; n],
                                "calibration",
                            ));
                        }
                    }
                }
            }
        }
    }
    for (era, line) in LINES.iter().enumerate() {
        for (old, earlier) in LINES.iter().enumerate().take(era + 1) {
            for n in [1, 2, 3] {
                let mut c = case(
                    &format!("mix-e{era}-e{old}-6v{}", 3 * n),
                    "mixed",
                    Goal::Destroy,
                    Ground::Open,
                    3,
                    line.repeat(2),
                    earlier.repeat(n),
                    "validation",
                );
                // Additional held-out variation: veterans and wounds, plus two factions.
                c.factions = Some([Faction::Sparta, Faction::Athens]);
                c.veteran = 2;
                grid.push(c);
            }
        }
    }
    for (era, line) in LINES.iter().enumerate() {
        for (class, &kind) in line.iter().enumerate() {
            for n in [2, 4, 8] {
                let mut c = case(
                    &format!("wounds-e{era}-c{class}-4v{n}-hp60"),
                    "wounded",
                    Goal::Destroy,
                    Ground::Open,
                    3,
                    vec![kind; 4],
                    vec![kind; n],
                    "validation",
                );
                c.veteran = 2;
                grid.push(c);
            }
        }
    }
    for row in saved()
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| [189, 190, 195].contains(&r["turn"].as_u64().unwrap()))
    {
        let rosters: Vec<Vec<UnitKind>> = row["armies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|units| {
                units
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|u| serde_json::from_value(u["kind"].clone()).unwrap())
                    .filter(|k| data::unit_stats(*k).military)
                    .collect()
            })
            .collect();
        let mut c = case(
            &format!("save-t{}", row["turn"]),
            "saved",
            Goal::Destroy,
            Ground::Open,
            3,
            rosters[0].clone(),
            rosters[1].clone(),
            "validation",
        );
        c.factions = Some([Faction::Sparta, Faction::Athens]);
        grid.push(c);
    }
    let rows: Vec<_> = grid.par_iter().map(|c| {
        let initial = study_arena(c, seed0, false);
        let scores = [values(&initial, PlayerId(0)), values(&initial, PlayerId(1))];
        let mut wins = [0_u32; 3];
        let mut invalid = 0;
        let mut unfinished = 0;
        for seed in seed0..seed0+repeats {
            // Mirror the sides to separate policy/initiative bias from army quality.
            for mirror in [false, true] {
                let mut c = c.clone();
                if mirror {
                    std::mem::swap(&mut c.attacker, &mut c.defender);
                    if let Some(f) = c.factions.as_mut() { f.swap(0, 1); }
                }
                let mut s = study_arena(&c, seed, mirror);
                for _ in 0..40 {
                    let orders = [PlayerId(0), PlayerId(1)].into_iter()
                        .map(|p| (p, human_orders(&s, p, &c, None, None))).collect();
                    let (next, events) = resolve_tactical_turn(&s, &orders);
                    invalid += events.iter().filter(|e| matches!(e, Event::OrderFailed { .. })).count();
                    s = next;
                    if [PlayerId(0), PlayerId(1)].iter().any(|p| s.units_of(*p).next().is_none()) { break; }
                }
                let alive = [PlayerId(0), PlayerId(1)].map(|p| s.units_of(p).next().is_some());
                let winner = match alive { [true, false] => usize::from(mirror), [false, true] => usize::from(!mirror), _ => { unfinished += 1; 2 } };
                wins[winner] += 1;
            }
        }
        json!({"id": c.id, "category": c.category, "split": if c.id.bytes().fold(0_u64, |h,b| h.wrapping_mul(31).wrapping_add(u64::from(b)))%3 == 0 {"held_out"} else {"fit"}, "scores":scores,"wins":wins,"unfinished":unfinished,"invalid_orders":invalid})
    }).collect();
    let path = PathBuf::from(arg(args, "--out-dir", "."));
    fs::create_dir_all(&path).unwrap();
    fs::write(
        path.join("army-calibration.json"),
        serde_json::to_vec(
            &json!({"candidates": CANDIDATES,"repeats":repeats,"seed0":seed0,"rows":rows}),
        )
        .unwrap(),
    )
    .unwrap();
    println!(
        "{} cells, {} real battles; {}",
        grid.len(),
        grid.len() as u64 * repeats * 2,
        path.display()
    );
}

fn saved() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../core/src/testdata/strongest_army.json"
    ))
    .unwrap()
}

fn study_arena(c: &Case, seed: u64, mirror: bool) -> GameState {
    let (mut s, _, _) = arena(c, seed);
    for u in s.units.values_mut() {
        let original_side = usize::from(u.owner.0) ^ usize::from(mirror);
        u.xp = if original_side == 0 {
            promotion::threshold(c.veteran).unwrap_or(0)
        } else {
            0
        };
        if original_side == 0 && c.id.ends_with("-hp60") {
            u.hp = 60;
        }
    }
    if let Some(t) = c.id.strip_prefix("save-t") {
        let rows = saved();
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["turn"].as_u64().unwrap().to_string() == t)
            .unwrap();
        for side in 0..2 {
            let original = side ^ usize::from(mirror);
            let records: Vec<_> = row["armies"][original]
                .as_array()
                .unwrap()
                .iter()
                .filter(|u| {
                    data::unit_stats(serde_json::from_value(u["kind"].clone()).unwrap()).military
                })
                .collect();
            for (u, r) in s
                .units
                .values_mut()
                .filter(|u| u.owner == PlayerId(side as u8))
                .zip(records)
            {
                u.hp = r["hp"].as_i64().unwrap() as i32;
                u.xp = r["xp"].as_u64().unwrap() as u32;
            }
        }
    }
    s
}
