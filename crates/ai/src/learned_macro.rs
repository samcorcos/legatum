//! View-only, integer macro value model. Training is offline; inference never reads a state.
use crate::openings::Opening;
use crate::plan::PlanKind;
use legatum_core::{Faction, PlayerView};

/// Feature order is part of the training artifact's schema.
pub const FEATURES: usize = 30;
pub const HEADS: usize = 10;

pub fn head(plan: &PlanKind, opening: Option<Opening>) -> usize {
    if let Some(opening) = opening {
        return 5 + Opening::ALL.iter().position(|o| *o == opening).unwrap_or(0);
    }
    match plan {
        PlanKind::Economy | PlanKind::Peace { .. } => 0,
        PlanKind::Campaign { .. } => 1,
        PlanKind::Contain { .. } | PlanKind::FieldStrike { .. } => 2,
        PlanKind::Snipe { .. } => 3,
        PlanKind::Hold { .. } => 4,
    }
}

/// Every feature is bounded so an integer dot product stays well within i64.
pub fn features(view: &PlayerView, plan: &PlanKind, opening: Option<Opening>) -> [i32; FEATURES] {
    let own_cities: Vec<_> = view.cities.iter().filter(|c| c.owner == view.me).collect();
    let own_units: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner == view.me && u.military)
        .collect();
    let enemy_units: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner != view.me && u.military && !u.owner.is_barbarian())
        .collect();
    let target_team = plan.war_target();
    let target_city = match plan {
        PlanKind::Campaign { target, .. } => view.cities.iter().find(|c| c.id == *target),
        _ => None,
    };
    let enemy_cities: Vec<_> = view
        .cities
        .iter()
        .filter(|c| target_team.is_some_and(|t| view.team_of(c.owner) == t))
        .collect();
    let near = target_city.or_else(|| {
        enemy_cities
            .iter()
            .min_by_key(|c| {
                own_cities
                    .iter()
                    .map(|o| o.pos.distance(c.pos))
                    .min()
                    .unwrap_or(99)
            })
            .copied()
    });
    let distance = near.map_or(20, |c| {
        own_cities
            .iter()
            .map(|o| o.pos.distance(c.pos))
            .min()
            .unwrap_or(20)
    });
    let target_force: i32 = near.map_or(0, |c| {
        own_units
            .iter()
            .filter(|u| u.pos.distance(c.pos) <= 8)
            .map(|u| u.strength.max(u.ranged))
            .sum()
    });
    let target_defenders: i32 = near.map_or(0, |c| {
        enemy_units
            .iter()
            .filter(|u| u.pos.distance(c.pos) <= 5)
            .map(|u| u.strength.max(u.ranged))
            .sum()
    });
    let nearest_threat = enemy_units
        .iter()
        .flat_map(|u| own_cities.iter().map(move |c| u.pos.distance(c.pos)))
        .min()
        .unwrap_or(20);
    let our_vp = view
        .players
        .iter()
        .find(|p| p.id == view.me)
        .map_or(0, |p| p.vp as i32);
    let lead_vp = view
        .standings
        .iter()
        .filter(|s| s.team != view.team)
        .map(|s| s.vp as i32)
        .max()
        .unwrap_or(0);
    let target_vp = target_team.map_or(0, |t| {
        view.standings
            .iter()
            .find(|s| s.team == t)
            .map_or(0, |s| s.vp as i32)
    });
    let yields = own_cities
        .iter()
        .filter_map(|c| c.detail.as_ref())
        .fold((0, 0, 0), |a, d| {
            (
                a.0 + d.yields.total.production,
                a.1 + d.yields.total.food,
                a.2 + d.yields.total.gold,
            )
        });
    let kind = match plan {
        PlanKind::Economy | PlanKind::Peace { .. } => 0,
        PlanKind::Campaign { .. } => 1,
        PlanKind::Contain { .. } | PlanKind::FieldStrike { .. } => 2,
        PlanKind::Snipe { .. } => 3,
        PlanKind::Hold { .. } => 4,
    };
    let opening_id = match opening {
        None => 0,
        Some(Opening::Default) => 1,
        Some(Opening::Rush) => 2,
        Some(Opening::GreedyExpand) => 3,
        Some(Opening::Snowball) => 4,
        Some(Opening::Economic) => 5,
    };
    let faction = view
        .players
        .iter()
        .find(|p| p.id == view.me)
        .map_or(Faction::Switzerland, |p| p.faction);
    let faction_id = Faction::ALL.iter().position(|f| *f == faction).unwrap_or(0) as i32;
    let mut f = [0; FEATURES];
    f[0] = 1;
    f[1] = (view.turn as i32).min(300);
    f[2] = (view.players.len() as i32).min(8);
    f[3] = faction_id;
    f[4] = (own_cities.len() as i32).min(30);
    f[5] = own_cities
        .iter()
        .map(|c| c.population)
        .sum::<i32>()
        .min(100);
    f[6] = (own_units.len() as i32).min(50);
    f[7] = own_units
        .iter()
        .map(|u| u.strength.max(u.ranged))
        .sum::<i32>()
        .min(2000)
        / 10;
    f[8] = (enemy_units.len() as i32).min(50);
    f[9] = nearest_threat.min(20);
    f[10] = yields.0.min(300);
    f[11] = yields.1.min(300);
    f[12] = yields.2.min(300);
    f[13] = view.economy.gold.clamp(-200, 500) / 5;
    f[14] = view.economy.gold_per_turn.total.clamp(-100, 200);
    f[15] = view.economy.science_per_turn.total.clamp(0, 300);
    f[16] = our_vp.min(30);
    f[17] = lead_vp.min(30);
    f[18] = target_vp.min(30);
    f[19] = (enemy_cities.len() as i32).min(30);
    f[20] = distance.min(20);
    f[21] = near.map_or(0, |c| c.population).min(30);
    f[22] = i32::from(near.is_some_and(|c| c.walls));
    f[23] = i32::from(near.is_some_and(|c| c.is_capital));
    f[24] = target_force.min(1000) / 10;
    f[25] = target_defenders.min(1000) / 10;
    f[26] = i32::from(kind == 1);
    f[27] = i32::from(kind == 2);
    f[28] = i32::from(kind == 3 || kind == 4);
    f[29] = opening_id;
    f
}

/// Generated by tools/sim/train-macro.py; the live gene is disabled by default.
pub const WEIGHTS: [[i32; FEATURES]; HEADS] = [
    [
        -129214, 3041, 6587, -12979, 17668, 2588, -24442, 7529, 2206, 6439, -2222, -625, 1366,
        1382, 2363, 29, 10035, -417, 0, 0, -6461, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        -68166, -54, 30743, -5335, 53992, -4204, 7029, -7321, 4999, 1500, 1195, 2224, -4374, -747,
        7746, -612, -21941, -18936, 1758, 4497, -5435, -6070, -42498, -20413, -1495, 475, -68166,
        0, 0, 0,
    ],
    [
        -59124, 2999, -18663, -3008, -8534, 3627, 3074, -8838, 797, -773, 2829, 430, 4423, 1684,
        -525, -974, -18094, -15066, -13401, 27196, -1237, 5535, -7254, 19554, 6685, -3993, 0,
        -59124, 0, 0,
    ],
    [
        256504, -5986, -18668, 21322, -63126, -2011, 14339, 8630, -8002, -7165, -1802, -2029,
        -1415, -2319, -9584, 1557, 30000, 34418, -14546, 80752, 16263, -19490, -185610, 127437,
        8355, -15705, 0, 0, 256504, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        54167, 7, -682, 15975, 0, 0, 375, 129, 0, 2708, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 2708, 0, 0,
        0, 0, 0, 0, 0, 0, 2167,
    ],
    [
        -137601, -17, 75054, 26468, 0, 0, -952, -327, 0, -6880, 0, 0, 0, -165, 0, 0, 0, 0, 0, 0,
        -6880, 0, 0, 0, 0, 0, 0, 0, 0, -11008,
    ],
    [
        77909, 10, -59868, 654, 0, 0, 539, 185, 0, 3895, 0, 0, 0, 93, 0, 0, 0, 0, 0, 0, 3895, 0, 0,
        0, 0, 0, 0, 0, 0, 9349,
    ],
    [
        -75334, -9, 25812, -27723, 0, 0, -521, -179, 0, -3767, 0, 0, 0, -90, 0, 0, 0, 0, 0, 0,
        -3767, 0, 0, 0, 0, 0, 0, 0, 0, -12053,
    ],
    [
        80859, 10, -40316, -15375, 0, 0, 560, 192, 0, 4043, 0, 0, 0, 97, 0, 0, 0, 0, 0, 0, 4043, 0,
        0, 0, 0, 0, 0, 0, 0, 16172,
    ],
];
pub const FACTION_BIAS: [[i32; 8]; HEADS] = [
    [-60093, 70035, -38046, 46258, 35345, -153690, -61523, 32500],
    [119048, -72585, -89660, -60188, 0, -49832, 175134, -90082],
    [2176, 44202, -77096, 4447, -10738, -71910, -3219, 53013],
    [-61130, -41652, 204801, 9484, -24608, 275431, -110391, 4569],
    [0, 0, 0, 0, 0, 0, 0, 0],
    [
        -60817, 81453, -98733, -110866, -42853, 360038, -121009, 46955,
    ],
    [
        -257646, 62604, -127219, -182248, -69635, 209981, 321771, -95209,
    ],
    [136725, -191393, 130176, 113736, 2945, -209283, 8517, 86488],
    [43156, -25985, 79733, 203892, 75372, -306251, -142945, -2305],
    [138583, 73321, 16043, -24514, 34172, -54485, -66333, -35929],
];

pub fn value(features: &[i32; FEATURES], plan: &PlanKind, opening: Option<Opening>) -> i32 {
    let head = head(plan, opening);
    let faction = features[3].clamp(0, 7) as usize;
    let total: i64 = WEIGHTS[head]
        .iter()
        .zip(features)
        .map(|(w, x)| i64::from(*w) * i64::from(*x))
        .sum::<i64>()
        + i64::from(FACTION_BIAS[head][faction]);
    (5_000 + total / 1024).clamp(0, 10_000) as i32
}
