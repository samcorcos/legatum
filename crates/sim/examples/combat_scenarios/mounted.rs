//! The mounted lab (`--mounted`, 2026-10-31): does the mounted class pull its weight in each era
//! where the triangle applies, with the class bonus as a percentage? Every army fight here is at
//! equal Production (a budget of eight of the era's melee line), Human policy (focus fire,
//! advance and strike), open ground, starts 2 to 5 hexes apart, both arena sides.
//!
//! - Duels: one unit of each at full health, mean rolls, open ground: damage dealt / taken.
//! - Armies: win share of the first army, and the exchange: Production the second army lost per
//!   Production the first lost (wounds count by HP, so 1.0 is an even trade and above 1 favours
//!   the first army).
//! - Responses: every melee/ranged/mounted mix at 90 to 100% of the budget against four
//!   archetypes; the best mix per archetype, whether one mix beats all four at 60%+, and the
//!   mounted share of the Production in the top responses.
use super::{
    Ground, Policy, TRIANGLE_TIERS, case, doctrines, play, repeat, tier_budget, triangle_army,
};
use legatum_core::{Terrain, UnitKind, combat, data};
use rayon::prelude::*;

/// The eras in which the triangle applies (Ancient to Industrial).
const TIERS: usize = 5;
const DISTANCES: [i32; 4] = [2, 3, 4, 5];

fn cost(k: UnitKind) -> i32 {
    data::unit_stats(k).cost
}

/// Mean-roll damage `a` deals when it charges `d` (or shoots it, `shot`), and what it takes back.
fn duel(a: UnitKind, d: UnitKind, shot: bool) -> (i32, i32) {
    let att = if shot {
        combat::unit_ranged_total(legatum_core::Faction::India, a, 100, 0)
    } else {
        combat::unit_attack_total(legatum_core::Faction::India, a, 100, 0)
    } + combat::fight_class_bonus(a, d, true, shot).1;
    let def = combat::unit_defense_total(
        legatum_core::Faction::India,
        d,
        100,
        Terrain::Grassland,
        0,
        0,
    ) + combat::fight_class_bonus(d, a, false, shot).1;
    let dealt = combat::damage(att, def, 100);
    let taken = if shot {
        0
    } else {
        combat::damage(def, att, 100)
    };
    (dealt, taken)
}

/// Production a side lost: dead units whole, survivors by the HP they lost.
fn lost(initial: &[UnitKind], survivors: &[(u32, UnitKind, i32)]) -> i64 {
    let start: i64 = initial.iter().map(|&k| i64::from(cost(k)) * 100).sum();
    let left: i64 = survivors
        .iter()
        .map(|&(_, k, hp)| i64::from(cost(k)) * i64::from(hp))
        .sum();
    (start - left).max(0)
}

/// Win points (2 a win, 1 a tie) out of 2, and the Production each side lost (x100), for army
/// `a` against army `b` from one start.
fn one(a: &[UnitKind], b: &[UnitKind], distance: i32, seed: u64, swap: bool) -> (i64, i64, i64) {
    let (left, right) = if swap { (b, a) } else { (a, b) };
    let c = case(
        "mounted",
        "mounted",
        super::Goal::Destroy,
        Ground::Open,
        distance,
        left.to_vec(),
        right.to_vec(),
        "valid_orders",
    );
    let r = play(&c, seed, Policy::Human, false);
    let (a_side, a_idx) = if swap {
        ("defender", 1)
    } else {
        ("attacker", 0)
    };
    let points = if r.winner == a_side {
        2
    } else if r.winner == "tie" {
        1
    } else {
        0
    };
    let lost_a = lost(a, &r.survivors[a_idx]);
    let lost_b = lost(b, &r.survivors[1 - a_idx]);
    (points, lost_a, lost_b)
}

/// Win share % of `a` over `b` and the exchange (Production `b` lost per Production `a` lost).
fn fight(a: &[UnitKind], b: &[UnitKind], seeds: usize, seed0: u64) -> (i32, f64) {
    let jobs: Vec<(i32, u64, bool)> = DISTANCES
        .iter()
        .flat_map(|&d| {
            (0..seeds)
                .flat_map(move |k| [(d, seed0 + k as u64, false), (d, seed0 + k as u64, true)])
        })
        .collect();
    let (points, la, lb, n) = jobs
        .par_iter()
        .map(|&(d, seed, swap)| {
            let (p, la, lb) = one(a, b, d, seed, swap);
            (p, la, lb, 2)
        })
        .reduce(
            || (0, 0, 0, 0),
            |x, y| (x.0 + y.0, x.1 + y.1, x.2 + y.2, x.3 + y.3),
        );
    (
        (points * 100 / n.max(1)) as i32,
        lb as f64 / la.max(1) as f64,
    )
}

pub(super) fn report(seeds: usize, seed0: u64) -> String {
    let mut text = format!(
        "mounted lab: mounted strength {}% of the era's melee line, class bonus +{}% of base strength\n",
        data::MOUNTED_STRENGTH_PERCENT,
        combat::CLASS_BONUS_PERCENT
    );
    text.push_str("duels, mean rolls, open ground, full health: damage dealt/taken\n");
    text.push_str("era          str m/r/h      bonus m/h   horse>spear  spear>horse  horse>archer  archer shoots horse  spear>archer\n");
    for (tier, [m, r, h]) in TRIANGLE_TIERS.iter().take(TIERS) {
        let s = |k: UnitKind| data::unit_stats(k).strength;
        let (hs, sh, ha, ah, sa) = (
            duel(*h, *m, false),
            duel(*m, *h, false),
            duel(*h, *r, false),
            duel(*r, *h, true),
            duel(*m, *r, false),
        );
        text.push_str(&format!(
            "{tier:<12} {:>3}/{:>3}/{:>3}    +{:>2}/+{:>2}     {:>3}/{:>3}      {:>3}/{:>3}      {:>3}/{:>3}       {:>3}                  {:>3}/{:>3}\n",
            s(*m),
            s(*r),
            s(*h),
            combat::class_bonus(*m, *h).1,
            combat::class_bonus(*h, *r).1,
            hs.0, hs.1, sh.0, sh.1, ha.0, ha.1, ah.0, sa.0, sa.1,
        ));
    }
    text.push_str("\narmies at equal Production (8 melee's worth), Human policy, open ground: win share % of the first / exchange (Production the second lost per Production the first lost)\n");
    text.push_str("era          cost m/r/h     line>horse    horse>ranged   horse>gunline  ranged>line   gunline>line\n");
    for (tier, units) in TRIANGLE_TIERS.iter().take(TIERS) {
        let [m, r, h] = *units;
        let budget = tier_budget(*units);
        let pure = |k: UnitKind| triangle_army(k, budget);
        let [line, gun, horse] = doctrines(*units);
        let cells: Vec<String> = [
            (line.1.clone(), horse.1.clone()),
            (horse.1.clone(), pure(r)),
            (horse.1.clone(), gun.1.clone()),
            (pure(r), line.1.clone()),
            (gun.1.clone(), line.1.clone()),
        ]
        .iter()
        .map(|(a, b)| {
            let (share, exchange) = fight(a, b, seeds, seed0);
            format!("{share:>3}% {exchange:>5.2}x")
        })
        .collect();
        text.push_str(&format!(
            "{tier:<12} {:>3}/{:>3}/{:>3}    {}\n",
            cost(m),
            cost(r),
            cost(h),
            cells.join("   ")
        ));
    }
    text.push_str("\nresponses: melee/ranged/mounted mixes at 90-100% of the budget against four archetypes, Human policy, open ground, mean of starts 2-5\n");
    for (tier, units) in TRIANGLE_TIERS.iter().take(TIERS) {
        let [m, r, h] = *units;
        let budget = tier_budget(*units);
        let build = |nm: usize, nr: usize, nh: usize| -> Vec<UnitKind> {
            let mut v = repeat(m, nm);
            v.extend(repeat(r, nr));
            v.extend(repeat(h, nh));
            v
        };
        let mut options = Vec::new();
        for nm in 0..=12usize {
            for nr in 0..=(12 - nm) {
                for nh in 0..=(12 - nm - nr) {
                    let total = nm as i32 * cost(m) + nr as i32 * cost(r) + nh as i32 * cost(h);
                    if nm + nr + nh >= 2 && total * 100 >= budget * 90 && total <= budget {
                        options.push((nm, nr, nh));
                    }
                }
            }
        }
        let [line, gun, horse] = doctrines(*units);
        let third = budget / 3;
        let mut balanced = triangle_army(m, third);
        balanced.extend(triangle_army(r, third));
        balanced.extend(triangle_army(h, third));
        let enemies = [
            ("line", line.1),
            ("gunline", gun.1),
            ("horse", horse.1),
            ("balanced", balanced),
        ];
        let grid: Vec<Vec<i32>> = options
            .par_iter()
            .map(|&(nm, nr, nh)| {
                let mine = build(nm, nr, nh);
                enemies
                    .iter()
                    .map(|(_, foe)| fight(&mine, foe, seeds, seed0).0)
                    .collect()
            })
            .collect();
        text.push_str(&format!("{tier} ({} mixes)\n", options.len()));
        let mut mounted_share = (0i64, 0i64);
        for (j, (name, _)) in enemies.iter().enumerate() {
            let mut order: Vec<usize> = (0..options.len()).collect();
            order.sort_by_key(|&i| (-grid[i][j], i));
            let best: Vec<String> = order
                .iter()
                .take(3)
                .map(|&i| {
                    let (nm, nr, nh) = options[i];
                    mounted_share.0 += nh as i64 * i64::from(cost(h));
                    mounted_share.1 += nm as i64 * i64::from(cost(m))
                        + nr as i64 * i64::from(cost(r))
                        + nh as i64 * i64::from(cost(h));
                    format!("{nm}/{nr}/{nh}={}", grid[i][j])
                })
                .collect();
            text.push_str(&format!("  best vs {name:<8} {}\n", best.join("  ")));
        }
        let dominant: Vec<String> = (0..options.len())
            .filter(|&i| grid[i].iter().all(|&v| v >= 60))
            .map(|i| format!("{}/{}/{}", options[i].0, options[i].1, options[i].2))
            .collect();
        let worst_best = (0..options.len())
            .map(|i| (grid[i].iter().copied().min().unwrap_or(0), i))
            .max()
            .map(|(v, i)| format!("{}/{}/{}={v}", options[i].0, options[i].1, options[i].2))
            .unwrap_or_default();
        text.push_str(&format!(
            "  beats every archetype at 60%+: {}   best worst-case: {worst_best}   mounted share of the top-3 responses' Production: {}%\n",
            if dominant.is_empty() { "none".to_string() } else { dominant.join(", ") },
            mounted_share.0 * 100 / mounted_share.1.max(1)
        ));
    }
    text
}
