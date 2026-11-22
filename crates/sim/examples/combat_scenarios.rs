//! Small, deterministic tactical arenas using core's real unit-phase resolver.
//! cargo run --release -p legatum-sim --example combat_scenarios -- --games 8 --out-dir DIR
//! Reports: `--eras`, `--triangle`, `--responses`, `--keshig`, `--phalanx`, `--mounted` (mounted.rs).
use legatum_ai::ScriptedBrain;
use legatum_core::{
    AiDifficulty, BuildingKind, CityId, Event, Faction, GameSetup, GameState, Hex, Order, PlayerId,
    TeamId, Terrain, UnitKind,
    data::{self, CITY_HP},
    game::{blank_game, spawn_unit},
    rules::{is_military, resolve_tactical_turn},
    state::{City, Map},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fs, path::PathBuf, time::Instant};

#[path = "combat_scenarios/army_measure.rs"]
mod army_measure;
#[path = "combat_scenarios/mounted.rs"]
mod mounted;

const WIDTH: i32 = 18;
const HEIGHT: i32 = 14;
const LIMIT: u32 = 12;
const COMPOSITION_BUDGET: i32 = 320;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Goal {
    Destroy,
    TakeCity,
    DefendCity,
    HoldPass,
    RaidSettler,
    EscortSettler,
    RelieveCity,
    BreakSiege,
    Retreat,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Ground {
    Open,
    Forest,
    Hills,
    River,
    Pass,
    Mixed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Policy {
    Hard,
    Human,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Case {
    id: String,
    category: String,
    goal: Goal,
    ground: Ground,
    distance: i32,
    attacker: Vec<UnitKind>,
    defender: Vec<UnitKind>,
    veteran: u8,
    city: bool,
    walls: bool,
    garrison: bool,
    city_hp: i32,
    low_hp: bool,
    #[serde(default)]
    kiting: bool,
    /// Each side's faction (India when absent): the Keshig report puts the Mongols on one side.
    #[serde(default)]
    factions: Option<[Faction; 2]>,
    /// A side whose mounted archers never shoot (the Human policy plays them as plain Horsemen):
    /// the Keshig report's counterfactual.
    #[serde(default)]
    no_bow: [bool; 2],
    expectation: String,
}
#[derive(Serialize, Deserialize)]
struct ResultRow {
    case: Case,
    seed: u64,
    policy: Policy,
    winner: String,
    turns: u32,
    losses: [usize; 2],
    hp: [i32; 2],
    survivors: [Vec<(u32, UnitKind, i32)>; 2],
    cities_taken: u32,
    goal_met: bool,
    invalid_orders: usize,
    actions: usize,
    trace: Vec<String>,
}
fn kind(name: &str) -> UnitKind {
    serde_json::from_str(&format!("\"{name}\"")).expect("known unit kind")
}
fn roster(rules: &str) -> Result<Vec<UnitKind>, String> {
    if rules != "default" {
        return Err(format!("unknown rules: {rules}"));
    }
    Ok(vec![
        UnitKind::Spearman,
        UnitKind::Archer,
        UnitKind::Scout,
        UnitKind::Horseman,
    ])
}
fn repeat(k: UnitKind, n: usize) -> Vec<UnitKind> {
    vec![k; n]
}
#[allow(clippy::too_many_arguments)] // Scenario rows are clearest with their independent axes visible.
fn case(
    id: &str,
    category: &str,
    goal: Goal,
    ground: Ground,
    distance: i32,
    attacker: Vec<UnitKind>,
    defender: Vec<UnitKind>,
    expectation: &str,
) -> Case {
    Case {
        id: id.into(),
        category: category.into(),
        goal,
        ground,
        distance,
        attacker,
        defender,
        veteran: 0,
        city: false,
        walls: false,
        garrison: false,
        city_hp: CITY_HP,
        low_hp: false,
        kiting: false,
        factions: None,
        no_bow: [false; 2],
        expectation: expectation.into(),
    }
}
fn cases(roster: &[UnitKind]) -> Vec<Case> {
    let mut out = Vec::new();
    let soldier = kind("Spearman");
    let ranger = kind("Archer");
    let scout = kind("Scout");
    for (a, b) in [(1, 1), (2, 2), (3, 3), (4, 5), (5, 4), (6, 6), (3, 6)] {
        for k in roster {
            for d in [1, 2, 4] {
                let expectation = if a == b && d == 1 {
                    "roughly_even"
                } else if a > b {
                    "numbers"
                } else if a < b {
                    "numbers_defender"
                } else {
                    "valid_orders"
                };
                out.push(case(
                    &format!("{a}v{b}-{k:?}-d{d}"),
                    "open_matchup",
                    Goal::Destroy,
                    Ground::Open,
                    d,
                    repeat(*k, a),
                    repeat(*k, b),
                    expectation,
                ));
            }
        }
    }
    for g in [
        Ground::Forest,
        Ground::Hills,
        Ground::River,
        Ground::Pass,
        Ground::Mixed,
    ] {
        out.push(case(
            &format!("terrain-{g:?}"),
            "terrain",
            Goal::Destroy,
            g,
            3,
            repeat(soldier, 4),
            repeat(soldier, 4),
            "defender_edge",
        ));
    }
    for walls in [false, true] {
        for garrison in [false, true] {
            for damaged in [false, true] {
                for attackers in [2, 4, 6] {
                    let mut c = case(
                        &format!("city-w{walls}-g{garrison}-d{damaged}-a{attackers}"),
                        "city",
                        Goal::TakeCity,
                        Ground::Open,
                        3,
                        repeat(soldier, attackers),
                        if garrison { vec![soldier] } else { vec![] },
                        if walls && garrison && !damaged && attackers < 6 {
                            "wall_holds"
                        } else if !walls && attackers >= 4 {
                            "unwalled_falls"
                        } else {
                            "valid_orders"
                        },
                    );
                    c.city = true;
                    c.walls = walls;
                    c.garrison = garrison;
                    c.city_hp = if damaged { 80 } else { CITY_HP };
                    out.push(c);
                }
            }
        }
    }
    for (a, b, label) in [
        (soldier, ranger, "melee_vs_ranged"),
        (ranger, soldier, "ranged_vs_melee"),
        (soldier, scout, "melee_vs_scout"),
    ] {
        out.push(case(
            label,
            "class",
            Goal::Destroy,
            Ground::Open,
            2,
            repeat(a, (160 / data::unit_stats(a).cost) as usize),
            repeat(b, (160 / data::unit_stats(b).cost) as usize),
            if a == ranger || (a == soldier && b == scout) {
                "attacker_wins"
            } else {
                "valid_orders"
            },
        ));
    }
    if let Some(&mounted) = roster.get(4) {
        out.push(case(
            "mounted-vs-ranged",
            "class",
            Goal::Destroy,
            Ground::Open,
            3,
            repeat(mounted, 3),
            repeat(ranger, 3),
            "attacker_wins",
        ));
        out.push(case(
            "melee-vs-mounted",
            "class",
            Goal::Destroy,
            Ground::Open,
            2,
            repeat(soldier, 3),
            repeat(mounted, 3),
            "attacker_wins",
        ));
    }
    let mut kite = case(
        "ranger-kite-vs-soldier",
        "kiting",
        Goal::Destroy,
        Ground::Open,
        4,
        repeat(ranger, 4),
        repeat(soldier, 5),
        "ranged_not_dominant",
    );
    kite.kiting = true;
    out.push(kite);
    for level in [1, 3, 5] {
        let mut v = case(
            &format!("veteran-l{level}-3v3"),
            "veterans",
            Goal::Destroy,
            Ground::Open,
            2,
            repeat(soldier, 3),
            repeat(soldier, 3),
            "attacker_wins",
        );
        v.veteran = level;
        out.push(v);
    }
    let goals = [
        Goal::TakeCity,
        Goal::DefendCity,
        Goal::HoldPass,
        Goal::RaidSettler,
        Goal::EscortSettler,
        Goal::RelieveCity,
        Goal::BreakSiege,
        Goal::Retreat,
    ];
    for (i, goal) in goals.into_iter().enumerate() {
        let expectation = match goal {
            Goal::TakeCity => "wall_holds",
            Goal::DefendCity => "defend_holds",
            Goal::HoldPass => "hold_pass",
            Goal::RaidSettler => "escort_protects",
            Goal::EscortSettler => "escort_reaches",
            Goal::RelieveCity => "relief_succeeds",
            Goal::BreakSiege => "siege_broken",
            Goal::Retreat => "retreat_survives",
            Goal::Destroy => "valid_orders",
        };
        let mut c = case(
            &format!("goal-{goal:?}"),
            "goal",
            goal,
            if matches!(goal, Goal::HoldPass) {
                Ground::Pass
            } else {
                Ground::Mixed
            },
            3,
            vec![soldier, ranger, scout],
            vec![soldier, ranger, scout],
            expectation,
        );
        c.city = matches!(
            goal,
            Goal::TakeCity | Goal::DefendCity | Goal::RelieveCity | Goal::BreakSiege
        );
        c.walls = i % 2 == 0;
        c.garrison = c.city && !matches!(goal, Goal::RelieveCity | Goal::BreakSiege);
        out.push(c);
    }
    for (id, a, b, low) in [
        ("all-scout", repeat(scout, 6), repeat(soldier, 3), false),
        (
            "one-veteran-six-green",
            repeat(soldier, 1),
            repeat(soldier, 6),
            false,
        ),
        ("ten-v-one", repeat(soldier, 10), repeat(soldier, 1), false),
        ("one-hp", vec![soldier, ranger], vec![soldier, ranger], true),
        (
            "settler-in-mix",
            vec![soldier, UnitKind::Settler, ranger],
            vec![soldier, scout],
            false,
        ),
    ] {
        let mut c = case(id, "edge", Goal::Destroy, Ground::Open, 2, a, b, "no_loop");
        c.low_hp = low;
        if id == "one-veteran-six-green" {
            c.veteran = 4;
        }
        out.push(c);
    }
    out.push(case(
        "scout-raids-escorted-settler",
        "edge",
        Goal::RaidSettler,
        Ground::Mixed,
        3,
        repeat(scout, 3),
        vec![soldier],
        "valid_orders",
    ));
    out
}
fn arena(c: &Case, seed: u64) -> (GameState, Option<CityId>, Option<Hex>) {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(2);
    setup.width = WIDTH;
    setup.height = HEIGHT;
    setup.legacy_all_war = true;
    setup.barbarians = false;
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = c.factions.map_or(Faction::India, |f| f[i]);
        p.team = TeamId(i as u8);
        p.difficulty = AiDifficulty::Hard;
    }
    let mut map = Map::new(WIDTH, HEIGHT, Terrain::Plains);
    let defender_col = 6 + c.distance;
    for row in 0..HEIGHT {
        let h = Hex::from_offset(defender_col, row);
        map.get_mut(h).unwrap().terrain = match c.ground {
            Ground::Forest => Terrain::Forest,
            Ground::Hills => Terrain::Hills,
            Ground::Mixed => {
                if row % 2 == 0 {
                    Terrain::Hills
                } else {
                    Terrain::Forest
                }
            }
            _ => Terrain::Plains,
        };
        if matches!(c.ground, Ground::River) && row != 6 && row != 7 {
            map.get_mut(Hex::from_offset(7, row)).unwrap().terrain = Terrain::Coast;
        }
        if matches!(c.ground, Ground::Pass) && !(5..=7).contains(&row) {
            map.get_mut(Hex::from_offset(7, row)).unwrap().terrain = Terrain::Mountains;
        }
    }
    let mut s = blank_game(&setup, map);
    for p in &mut s.players {
        for i in 0..s.map.len() {
            p.explored.set(i);
        }
    }
    let city = if c.city {
        let pos = Hex::from_offset(defender_col + 1, 6);
        let id = s.alloc_city_id();
        let owner = if matches!(c.goal, Goal::RelieveCity | Goal::BreakSiege) {
            PlayerId(0)
        } else {
            PlayerId(1)
        };
        let mut buildings = Default::default();
        if c.walls {
            std::collections::BTreeSet::insert(&mut buildings, BuildingKind::Walls);
        }
        s.cities.insert(
            id,
            City {
                last_attacked_turn: None,
                id,
                owner,
                founder: owner,
                name: "Arena".into(),
                pos,
                population: 3,
                food_stored: 0,
                production_stored: 0,
                producing: None,
                production_banked: Vec::new(),
                production_queue: Vec::new(),
                build_automation: None,
                last_auto_pick: None,
                auto_queued: None,
                production_queue_notified: Default::default(),
                wonder_lost: None,
                last_purchase_turn: None,
                bought: None,
                buildings,
                hp: c.city_hp,
                tiles_claimed: 0,
                grown_tiles: Vec::new(),
                is_capital: false,
                original_capital: false,
                greece_captive_paid: Default::default(),
                occupied_from: None,
                founded_turn: 1,
                resistance: 0,
            },
        );
        for h in pos.within(1) {
            if let Some(t) = s.map.get_mut(h) {
                t.owner = Some(owner);
                t.city = Some(id);
            }
        }
        Some(id)
    } else {
        None
    };
    let site = Hex::from_offset(defender_col + 2, 6);
    for (side, army) in [(0, &c.attacker), (1, &c.defender)] {
        for (i, &k) in army.iter().enumerate() {
            let row = if side == 1
                && (matches!(c.ground, Ground::River | Ground::Pass)
                    || matches!(c.goal, Goal::RelieveCity | Goal::BreakSiege))
            {
                4 + i as i32
            } else {
                2 + i as i32
            };
            let mut pos = Hex::from_offset(if side == 0 { 6 } else { defender_col }, row);
            if c.city && c.garrison && i == 0 && s.cities[&city.unwrap()].owner == PlayerId(side) {
                pos = s.cities[&city.unwrap()].pos;
            }
            let id = spawn_unit(&mut s, PlayerId(side), k, pos);
            if c.low_hp {
                s.units.get_mut(&id).unwrap().hp = 1;
            }
            if side == 0 && c.veteran > 0 {
                s.units.get_mut(&id).unwrap().xp =
                    legatum_core::rules::promotion::threshold(c.veteran).unwrap_or(0);
            }
        }
    }
    if matches!(c.goal, Goal::RaidSettler) {
        spawn_unit(&mut s, PlayerId(1), UnitKind::Settler, site);
    }
    if matches!(c.goal, Goal::EscortSettler) {
        spawn_unit(
            &mut s,
            PlayerId(0),
            UnitKind::Settler,
            Hex::from_offset(5, 6),
        );
    }
    (s, city, Some(site))
}
fn human_orders(
    s: &GameState,
    side: PlayerId,
    c: &Case,
    city: Option<CityId>,
    site: Option<Hex>,
) -> Vec<Order> {
    let view = legatum_core::player_view(s, side);
    let own: Vec<_> = view.units.iter().filter(|u| u.owner == side).collect();
    let foes: Vec<_> = view.units.iter().filter(|u| u.owner != side).collect();
    let mut os = Vec::new();
    for u in &own {
        if u.kind == UnitKind::Settler {
            if matches!(c.goal, Goal::EscortSettler) {
                os.push(Order::MoveUnit {
                    unit: u.id,
                    to: site.unwrap(),
                });
            }
            continue;
        }
        if matches!(c.goal, Goal::Retreat) || u.hp <= 20 {
            let to = u
                .pos
                .neighbors()
                .into_iter()
                .filter(|h| {
                    s.map.move_cost(*h).is_some()
                        && !s.units.values().any(|v| v.pos == *h && v.owner == side)
                })
                .max_by_key(|h| {
                    (
                        foes.iter().map(|e| e.pos.distance(*h)).min().unwrap_or(0),
                        -h.q,
                        -h.r,
                    )
                });
            if let Some(to) = to {
                os.push(Order::MoveUnit { unit: u.id, to });
            }
            continue;
        }
        let target = foes
            .iter()
            .filter(|e| is_military(e.kind) || matches!(c.goal, Goal::RaidSettler))
            .min_by_key(|e| {
                (
                    if matches!(c.goal, Goal::RaidSettler) && e.kind == UnitKind::Settler {
                        0
                    } else {
                        1
                    },
                    // Riders go for the shooters: that is what the class is for.
                    u32::from(
                        data::unit_class(u.kind) != Some(legatum_core::UnitClass::Mounted)
                            || !matches!(
                                data::unit_class(e.kind),
                                Some(legatum_core::UnitClass::Ranged)
                            ),
                    ),
                    e.hp,
                    u.pos.distance(e.pos),
                    e.id,
                )
            })
            .copied();
        let city_pos = city
            .and_then(|id| s.cities.get(&id))
            .filter(|x| x.owner != side)
            .map(|x| x.pos);
        let waypoint = Hex::from_offset(
            if side == PlayerId(0) {
                6 + c.distance
            } else {
                6
            },
            6,
        );
        let destination = if matches!(c.goal, Goal::TakeCity) {
            city_pos.or_else(|| target.map(|e| e.pos))
        } else {
            target.map(|e| e.pos)
        }
        .or(Some(waypoint));
        let Some(dest) = destination else { continue };
        let faction = view.faction_of(side);
        let mut stats = data::unit_stats_for(faction, u.kind);
        // A mounted archer (the Keshig) picks a charge or a shot by value each turn; without its
        // bow it is played as the Horseman it replaces.
        if stats.ranged > 0 && data::attacks_in_melee(u.kind) {
            if c.no_bow[side.0 as usize] {
                stats.ranged = 0;
            } else if let Some(order) = horse_archer_order(s, &view, u, &foes, city_pos) {
                os.push(order);
                continue;
            }
        }
        if c.kiting
            && side == PlayerId(0)
            && stats.ranged > 0
            && foes.iter().any(|e| u.pos.distance(e.pos) <= 1)
            && let Some(to) = u
                .pos
                .neighbors()
                .into_iter()
                .filter(|h| {
                    s.map.move_cost(*h).is_some() && !own.iter().any(|friend| friend.pos == *h)
                })
                .max_by_key(|h| {
                    (
                        foes.iter().map(|e| e.pos.distance(*h)).min().unwrap_or(0),
                        -h.q,
                        -h.r,
                    )
                })
        {
            os.push(Order::MoveUnit { unit: u.id, to });
            continue;
        }
        if stats.ranged > 0 {
            let shot = foes
                .iter()
                .filter(|e| {
                    e.military
                        && u.pos.distance(e.pos) <= stats.range
                        && legatum_core::los::visible(u.pos, e.pos, |h| {
                            s.map.get(h).map(|t| t.terrain)
                        })
                })
                .min_by_key(|e| (e.hp, e.id))
                .map(|e| e.pos)
                .or_else(|| {
                    city_pos.filter(|p| {
                        u.pos.distance(*p) <= stats.range
                            && legatum_core::los::visible(u.pos, *p, |h| {
                                s.map.get(h).map(|t| t.terrain)
                            })
                    })
                });
            if let Some(target) = shot {
                os.push(Order::RangedAttack { unit: u.id, target });
                continue;
            }
        }
        // Keep ranged support one tile behind the nearest melee unit while it approaches.
        let move_to = if stats.ranged > 0 {
            own.iter()
                .filter(|ally| {
                    ally.id != u.id
                        && data::unit_stats(ally.kind).ranged == 0
                        && is_military(ally.kind)
                })
                .min_by_key(|ally| (ally.pos.distance(dest), ally.id))
                .and_then(|ally| {
                    ally.pos
                        .neighbors()
                        .into_iter()
                        .filter(|h| {
                            s.map.move_cost(*h).is_some()
                                && !own.iter().any(|friend| friend.pos == *h)
                        })
                        .max_by_key(|h| (h.distance(dest), -h.q, -h.r))
                })
                .unwrap_or(dest)
        } else {
            dest
        };
        if (matches!(c.goal, Goal::HoldPass | Goal::DefendCity)
            || !matches!(c.ground, Ground::Open))
            && side == PlayerId(1)
            && u.pos.distance(move_to) > 1
        {
            os.push(Order::SkipUnit { unit: u.id });
        } else {
            os.push(Order::MoveUnit {
                unit: u.id,
                to: move_to,
            });
        }
    }
    os
}
/// The Human policy's mounted archer: of every charge it can make this turn and every shot in
/// range, the best trade (expected damage dealt, plus the defender's value when it dies, minus
/// the damage taken); otherwise ride to the nearest tile it can shoot from.
fn horse_archer_order(
    s: &GameState,
    view: &legatum_core::PlayerView,
    u: &legatum_core::view::UnitView,
    foes: &[&legatum_core::view::UnitView],
    city: Option<Hex>,
) -> Option<Order> {
    use legatum_core::forecast::{AttackMode, forecast_attack_mode};
    let value = |f: &legatum_core::BattleForecast| {
        f.expected_damage_to_defender + f.win_chance as i32
            - f.expected_damage_to_attacker
            - f.loss_chance as i32
    };
    let mut best: Option<(i32, Order)> = None;
    let targets = foes
        .iter()
        .filter(|e| e.military)
        .map(|e| e.pos)
        .chain(city)
        .collect::<Vec<_>>();
    for at in targets {
        let d = u.pos.distance(at);
        if d <= u.range
            && let Some(f) = forecast_attack_mode(view, u.id, at, AttackMode::Ranged)
            && best.as_ref().is_none_or(|(v, _)| value(&f) > *v)
        {
            best = Some((
                value(&f),
                Order::RangedAttack {
                    unit: u.id,
                    target: at,
                },
            ));
        }
        if d <= u.moves_left.max(1)
            && let Some(f) = forecast_attack_mode(view, u.id, at, AttackMode::Melee)
            && best.as_ref().is_none_or(|(v, _)| value(&f) > *v)
        {
            best = Some((value(&f), Order::MoveUnit { unit: u.id, to: at }));
        }
    }
    if let Some((_, order)) = best {
        return Some(order);
    }
    let target = foes
        .iter()
        .filter(|e| e.military)
        .map(|e| e.pos)
        .chain(city)
        .min_by_key(|p| (u.pos.distance(*p), p.q, p.r))?;
    let to = target
        .ring(u.range)
        .into_iter()
        .filter(|h| s.map.move_cost(*h).is_some() && !view.units.iter().any(|v| v.pos == *h))
        .min_by_key(|h| (u.pos.distance(*h), -h.q, -h.r))?;
    Some(Order::MoveUnit { unit: u.id, to })
}
fn play(c: &Case, seed: u64, policy: Policy, keep_trace: bool) -> ResultRow {
    let (mut s, city, site) = arena(c, seed);
    let initial = [
        s.units_of(PlayerId(0)).count(),
        s.units_of(PlayerId(1)).count(),
    ];
    let factions = c.factions.unwrap_or([Faction::India; 2]);
    let brains = [
        ScriptedBrain::with_difficulty(factions[0], seed ^ 11, PlayerId(0), AiDifficulty::Hard),
        ScriptedBrain::with_difficulty(factions[1], seed ^ 29, PlayerId(1), AiDifficulty::Hard),
    ];
    let mut trace = Vec::new();
    let mut turns = 0;
    let mut cities_taken = 0;
    let mut invalid_orders = 0;
    let mut actions = 0;
    let mut held_pass = true;
    while turns < LIMIT {
        let mut orders = BTreeMap::new();
        for side in 0..2u8 {
            let id = PlayerId(side);
            let os = match policy {
                Policy::Hard => {
                    let target = if side == 0 && matches!(c.goal, Goal::TakeCity | Goal::DefendCity)
                    {
                        city
                    } else {
                        None
                    };
                    brains[side as usize]
                        .tactical_orders(&legatum_core::player_view(&s, id), target)
                }
                Policy::Human => human_orders(&s, id, c, city, site),
            };
            orders.insert(id, os);
        }
        let (next, events) = resolve_tactical_turn(&s, &orders);
        invalid_orders += events
            .iter()
            .filter(|e| matches!(e, Event::OrderFailed { .. }))
            .count();
        actions += events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::Battle { .. } | Event::UnitMoved { .. } | Event::CityCaptured { .. }
                )
            })
            .count();
        if keep_trace {
            trace.push(format!(
                "turn {} orders {:?} events {:?}",
                s.turn, orders, events
            ));
        }
        cities_taken += events
            .iter()
            .filter(|e| matches!(e, Event::CityCaptured { .. }))
            .count() as u32;
        s = next;
        turns += 1;
        if matches!(c.goal, Goal::HoldPass) {
            held_pass &= s
                .units_of(PlayerId(1))
                .any(|u| (7..=9).contains(&u.pos.to_offset().0) && is_military(u.kind));
        }
        let a = s.units_of(PlayerId(0)).any(|u| is_military(u.kind));
        let b = s.units_of(PlayerId(1)).any(|u| is_military(u.kind));
        let hostile_city_stands = city.is_some_and(|id| s.cities[&id].owner == PlayerId(1));
        let raid_done = matches!(c.goal, Goal::RaidSettler)
            && !s.units_of(PlayerId(1)).any(|u| u.kind == UnitKind::Settler);
        let escort_done = matches!(c.goal, Goal::EscortSettler)
            && s.units_of(PlayerId(0))
                .any(|u| u.kind == UnitKind::Settler && Some(u.pos) == site);
        let relief_done = matches!(c.goal, Goal::RelieveCity)
            && city.is_some_and(|id| {
                s.cities[&id].owner == PlayerId(0)
                    && s.units_of(PlayerId(0))
                        .any(|u| is_military(u.kind) && u.pos.distance(s.cities[&id].pos) <= 1)
            });
        let siege_broken = matches!(c.goal, Goal::BreakSiege)
            && city.is_some_and(|id| {
                s.cities[&id].owner == PlayerId(0)
                    && !s
                        .units_of(PlayerId(1))
                        .any(|u| is_military(u.kind) && u.pos.distance(s.cities[&id].pos) <= 2)
            });
        if (!a && !matches!(c.goal, Goal::HoldPass))
            || (!b && !hostile_city_stands)
            || raid_done
            || escort_done
            || relief_done
            || siege_broken
            || (matches!(c.goal, Goal::TakeCity | Goal::DefendCity)
                && city.is_some_and(|id| s.cities[&id].owner == PlayerId(0)))
        {
            break;
        }
    }
    let hp = [0, 1].map(|side| {
        s.units_of(PlayerId(side))
            .filter(|u| is_military(u.kind))
            .map(|u| u.hp)
            .sum::<i32>()
    });
    let losses = [0, 1].map(|side| initial[side] - s.units_of(PlayerId(side as u8)).count());
    let survivors = [0, 1].map(|side| {
        s.units_of(PlayerId(side))
            .map(|u| (u.id.0, u.kind, u.hp))
            .collect()
    });
    let winner = if hp[0] > 0 && hp[1] > 0 && (hp[0] - hp[1]).abs() < 20 {
        "tie"
    } else if hp[0] > hp[1] {
        "attacker"
    } else if hp[1] > hp[0] {
        "defender"
    } else {
        "tie"
    };
    let took = city.is_some_and(|id| s.cities[&id].owner == PlayerId(0));
    let goal_met = match c.goal {
        Goal::TakeCity => took,
        Goal::RelieveCity => {
            city.is_some_and(|id| s.cities[&id].owner == PlayerId(0))
                && s.units_of(PlayerId(0)).any(|u| {
                    u.pos.distance(s.cities[&city.unwrap()].pos) <= 1 && is_military(u.kind)
                })
        }
        Goal::BreakSiege => {
            city.is_some_and(|id| s.cities[&id].owner == PlayerId(0))
                && !s.units_of(PlayerId(1)).any(|u| {
                    u.pos.distance(s.cities[&city.unwrap()].pos) <= 2 && is_military(u.kind)
                })
        }
        Goal::DefendCity => !took,
        Goal::HoldPass => held_pass && turns == LIMIT,
        Goal::RaidSettler => !s.units_of(PlayerId(1)).any(|u| u.kind == UnitKind::Settler),
        Goal::EscortSettler => s
            .units_of(PlayerId(0))
            .any(|u| u.kind == UnitKind::Settler && u.pos == site.unwrap()),
        Goal::Retreat => losses[0] == 0,
        Goal::Destroy => hp[0] > hp[1],
    };
    ResultRow {
        case: c.clone(),
        seed,
        policy,
        winner: winner.into(),
        turns,
        losses,
        hp,
        survivors,
        cities_taken,
        goal_met,
        invalid_orders,
        actions,
        trace,
    }
}
fn compositions(roster: &[UnitKind], budget: i32) -> Vec<Vec<UnitKind>> {
    let mut out = Vec::new();
    fn rec(
        roster: &[UnitKind],
        i: usize,
        left: i32,
        army: &mut Vec<UnitKind>,
        out: &mut Vec<Vec<UnitKind>>,
    ) {
        if i == roster.len() {
            if !army.is_empty() && army.len() <= 10 {
                out.push(army.clone());
            }
            return;
        }
        let cost = data::unit_stats(roster[i]).cost;
        for n in 0..=(left / cost).min((10 - army.len()) as i32) {
            for _ in 0..n {
                army.push(roster[i]);
            }
            rec(roster, i + 1, left - n * cost, army, out);
            for _ in 0..n {
                army.pop();
            }
        }
    }
    rec(roster, 0, budget, &mut Vec::new(), &mut out);
    out.retain(|a| a.iter().map(|k| data::unit_stats(*k).cost).sum::<i32>() >= budget - 30);
    out
}
#[derive(Serialize)]
struct CompositionRow {
    family: String,
    goal: Goal,
    fixed_best: Vec<UnitKind>,
    response_best: Vec<UnitKind>,
    fixed_score: i32,
    worst_score: i32,
}
fn search(roster: &[UnitKind], seed: u64) -> Vec<CompositionRow> {
    let options = compositions(roster, COMPOSITION_BUDGET);
    let mut rows = Vec::new();
    for (family, goal, ground, city, walls) in [
        ("open", Goal::Destroy, Ground::Open, false, false),
        ("forest", Goal::Destroy, Ground::Forest, false, false),
        ("hills", Goal::Destroy, Ground::Hills, false, false),
        ("river", Goal::Destroy, Ground::River, false, false),
        ("city", Goal::TakeCity, Ground::Open, true, false),
        ("walls", Goal::TakeCity, Ground::Open, true, true),
        ("defend_city", Goal::DefendCity, Ground::Open, true, true),
        ("pass", Goal::HoldPass, Ground::Pass, false, false),
        ("raid", Goal::RaidSettler, Ground::Open, false, false),
        ("escort", Goal::EscortSettler, Ground::Mixed, false, false),
        ("relieve", Goal::RelieveCity, Ground::Mixed, true, true),
        ("break_siege", Goal::BreakSiege, Ground::Mixed, true, true),
        ("retreat", Goal::Retreat, Ground::Open, false, false),
    ] {
        let fixed = vec![UnitKind::Spearman; 8];
        let eval = |a: &Vec<UnitKind>, b: &Vec<UnitKind>| -> i32 {
            let defending = matches!(goal, Goal::DefendCity | Goal::HoldPass);
            let mut c = case(
                "composition",
                "composition",
                goal,
                ground,
                3,
                if defending { b.clone() } else { a.clone() },
                if defending { a.clone() } else { b.clone() },
                "valid_orders",
            );
            c.city = city;
            c.walls = walls;
            c.garrison = city && !matches!(goal, Goal::RelieveCity | Goal::BreakSiege);
            let r = play(&c, seed, Policy::Human, false);
            let material = if defending {
                r.hp[1] - r.hp[0]
            } else {
                r.hp[0] - r.hp[1]
            };
            material + if r.goal_met { 100 } else { 0 }
                - if matches!(goal, Goal::Retreat) {
                    r.losses[0] as i32 * 100
                } else {
                    0
                }
        };
        let (fixed_best, fixed_score) = options
            .iter()
            .map(|a| (a.clone(), eval(a, &fixed)))
            .max_by_key(|(_, score)| *score)
            .unwrap();
        // Maximin best response: every candidate is tested against every affordable defender.
        let (response_best, worst_score) = options
            .iter()
            .map(|a| {
                let worst = options.iter().map(|b| eval(a, b)).min().unwrap();
                (a.clone(), worst)
            })
            .max_by_key(|(_, score)| *score)
            .unwrap();
        rows.push(CompositionRow {
            family: family.into(),
            goal,
            fixed_best,
            response_best,
            fixed_score,
            worst_score,
        });
    }
    rows
}
fn passed(c: &Case, rows: &[&ResultRow]) -> Option<bool> {
    let rate =
        |f: fn(&ResultRow) -> bool| rows.iter().filter(|r| f(r)).count() * 100 / rows.len().max(1);
    // A stationary draw is not evidence that a combat expectation holds.
    if !matches!(c.expectation.as_str(), "valid_orders" | "retreat_survives")
        && rate(|r| r.actions > 0) < 50
    {
        return Some(false);
    }
    let attacker_share = rows
        .iter()
        .map(|r| {
            if r.winner == "attacker" {
                2
            } else if r.winner == "tie" {
                1
            } else {
                0
            }
        })
        .sum::<usize>()
        * 50
        / rows.len().max(1);
    match c.expectation.as_str() {
        "roughly_even" => Some((30..=65).contains(&attacker_share)),
        "numbers" | "attacker_wins" => Some(rate(|r| r.winner == "attacker") >= 60),
        "defender_edge" => Some(rate(|r| r.winner == "defender") >= 55),
        "wall_holds" => Some(rate(|r| !r.goal_met) >= 75),
        "unwalled_falls" => Some(rate(|r| r.goal_met) >= 50),
        "ranged_not_dominant" => Some(rate(|r| r.winner == "attacker") <= 60),
        "defend_holds" | "escort_reaches" | "relief_succeeds" | "siege_broken" => {
            Some(rate(|r| r.goal_met) >= 50)
        }
        "hold_pass" => Some(rate(|r| r.goal_met) >= 55),
        "escort_protects" => Some(rate(|r| !r.goal_met) >= 60),
        "retreat_survives" => Some(rate(|r| r.goal_met) >= 75),
        "numbers_defender" => Some(rate(|r| r.winner == "defender") >= 60),
        "no_loop" => Some(rows.iter().all(|r| r.turns < LIMIT)),
        "valid_orders" => Some(rows.iter().all(|r| r.invalid_orders == 0)),
        _ => None,
    }
}
fn violating_trial(c: &Case, row: &ResultRow) -> bool {
    match c.expectation.as_str() {
        "numbers" | "attacker_wins" => row.winner != "attacker",
        "numbers_defender" | "defender_edge" => row.winner != "defender",
        "wall_holds" => row.goal_met,
        "unwalled_falls" => !row.goal_met,
        "walled_falls_with_siege" => !row.goal_met || row.turns > 8,
        "ranged_not_dominant" => row.winner == "attacker",
        "siege_cannot_occupy" => row.goal_met,
        "defend_holds" | "escort_reaches" | "relief_succeeds" | "siege_broken" | "hold_pass"
        | "retreat_survives" => !row.goal_met,
        "escort_protects" => row.goal_met,
        "no_loop" => row.turns == LIMIT,
        "valid_orders" => row.invalid_orders > 0,
        "roughly_even" => true,
        _ => true,
    }
}
/// The class triangle at equal Production: per era tier, melee line / ranged / mounted, and the
/// cross-era checks (a unit one era ahead against its counter, two eras ahead against the
/// triangle). `--triangle` prints the win share of the first-named class over the second.
const TRIANGLE_TIERS: [(&str, [UnitKind; 3]); 7] = [
    (
        "Ancient",
        [UnitKind::Spearman, UnitKind::Archer, UnitKind::Horseman],
    ),
    (
        "Classical",
        [
            UnitKind::Phalangite,
            UnitKind::CompositeArcher,
            UnitKind::Cataphract,
        ],
    ),
    (
        "Medieval",
        [UnitKind::Pikeman, UnitKind::Crossbowman, UnitKind::Knight],
    ),
    (
        "Renaissance",
        [
            UnitKind::Musketman,
            UnitKind::Arquebusier,
            UnitKind::Cuirassier,
        ],
    ),
    (
        "Industrial",
        [UnitKind::Rifleman, UnitKind::FieldCannon, UnitKind::Cavalry],
    ),
    (
        "Modern",
        [UnitKind::Infantry, UnitKind::Artillery, UnitKind::Tank],
    ),
    (
        "Future",
        [
            UnitKind::HunterKiller,
            UnitKind::CombatDrone,
            UnitKind::AutonomousArmour,
        ],
    ),
];
/// A tier's budget: eight units of its melee line, so every era fights with armies of a size.
fn tier_budget(tier: [UnitKind; 3]) -> i32 {
    8 * data::unit_stats(tier[0]).cost
}
fn triangle_army(k: UnitKind, budget: i32) -> Vec<UnitKind> {
    repeat(
        k,
        ((budget + data::unit_stats(k).cost / 2) / data::unit_stats(k).cost).clamp(1, 10) as usize,
    )
}
/// Win share of army `a` over army `b` (ties half), both sides of the arena, both policies.
fn army_share(
    a: &[UnitKind],
    b: &[UnitKind],
    distance: i32,
    ground: Ground,
    seeds: usize,
    seed0: u64,
) -> [i32; 2] {
    let mut out = [0, 0];
    for (pi, policy) in [Policy::Hard, Policy::Human].into_iter().enumerate() {
        let (mut points, mut n) = (0, 0);
        for k in 0..seeds {
            for swap in [false, true] {
                let (left, right) = if swap { (b, a) } else { (a, b) };
                let c = case(
                    "triangle",
                    "triangle",
                    Goal::Destroy,
                    ground,
                    distance,
                    left.to_vec(),
                    right.to_vec(),
                    "valid_orders",
                );
                let r = play(&c, seed0 + k as u64, policy, false);
                let a_side = if swap { "defender" } else { "attacker" };
                points += if r.winner == a_side {
                    2
                } else if r.winner == "tie" {
                    1
                } else {
                    0
                };
                n += 2;
            }
        }
        out[pi] = points * 100 / n;
    }
    out
}
/// The three doctrines at one era tier: a pure line, a gunline (a third of the Production in the
/// melee screen, two thirds in shooters) and a pure mounted force.
fn doctrines(tier: [UnitKind; 3]) -> [(&'static str, Vec<UnitKind>); 3] {
    let [m, r, h] = tier;
    let budget = tier_budget(tier);
    let mut gun = triangle_army(m, budget / 3);
    gun.extend(triangle_army(r, budget * 2 / 3));
    [
        ("line", triangle_army(m, budget)),
        ("gunline", gun),
        ("horse", triangle_army(h, budget)),
    ]
}
fn mean_share(
    a: &[UnitKind],
    b: &[UnitKind],
    ground: Ground,
    seeds: usize,
    seed0: u64,
) -> [i32; 2] {
    let distances = [2, 3, 4, 5];
    let mut sum = [0, 0];
    for d in distances {
        let v = army_share(a, b, d, ground, seeds, seed0);
        sum[0] += v[0];
        sum[1] += v[1];
    }
    [
        sum[0] / distances.len() as i32,
        sum[1] / distances.len() as i32,
    ]
}
fn triangle_report(seeds: usize, seed0: u64) -> String {
    let mut text = String::new();
    text.push_str(
        "win share % of the first army over the second (Hard / Human policy), mean of starts 2-5 hexes apart, equal Production\n",
    );
    for (tier, units) in TRIANGLE_TIERS {
        let [line, gun, horse] = doctrines(units);
        let budget = tier_budget(units);
        let pure = |k: UnitKind| triangle_army(k, budget);
        let [m, r, h] = units;
        for (x, y, label) in [
            (pure(m), pure(h), "melee > mounted"),
            (pure(h), pure(r), "mounted > ranged"),
            (pure(r), pure(m), "ranged > melee"),
            (gun.1.clone(), line.1.clone(), "gunline > line"),
            (horse.1.clone(), gun.1.clone(), "horse > gunline"),
            (line.1.clone(), horse.1.clone(), "line > horse"),
            (horse.1.clone(), pure(r), "horse > pure ranged"),
        ] {
            let cells: Vec<_> = [Ground::Open, Ground::Forest]
                .into_iter()
                .map(|g| {
                    let [hard, human] = mean_share(&x, &y, g, seeds, seed0);
                    format!("{:?} {hard:>3}/{human:>3}", g)
                })
                .collect();
            text.push_str(&format!(
                "{tier:<10} {label:<20} ({}v{})  {}\n",
                x.len(),
                y.len(),
                cells.join("   ")
            ));
        }
    }
    text
}
/// The size of the army that stays whole in `blended_share`; the other side grows with the cost ratio, up to 18.
const FIXED_UNITS: usize = 4;
/// Win share of `FIXED_UNITS` `fixed` units against `varied` units worth the same Production. The count
/// of `varied` is fractional (cost ratios do not divide evenly), so the share is blended between
/// the two whole counts either side of it: no era pair is rewarded or punished by rounding.
fn blended_share(fixed: UnitKind, varied: UnitKind, seeds: usize, seed0: u64) -> [i32; 2] {
    let budget = FIXED_UNITS as i32 * data::unit_stats(fixed).cost;
    let exact = budget as f64 / data::unit_stats(varied).cost as f64;
    let low = exact.floor() as usize;
    let frac = exact - low as f64;
    let share = |n: usize| {
        mean_share(
            &repeat(fixed, FIXED_UNITS),
            &repeat(varied, n.clamp(1, 18)),
            Ground::Open,
            seeds,
            seed0,
        )
    };
    let (a, b) = (share(low), share(low + 1));
    // `share` is the fixed army's win share; the varied army's is its complement.
    [0, 1].map(|i| {
        let fixed_share = (1.0 - frac) * a[i] as f64 + frac * b[i] as f64;
        fixed_share.round() as i32
    })
}
/// Era gaps at equal Production (`--eras`): the same class one, two and three eras ahead, and a
/// counter-class unit one era behind against the class it counters. Win share of the first
/// named army (Hard / Human policy), open ground, mean of starts 2 to 5 hexes apart. The
/// budget is ten units of the newer unit, so the older army is larger by the cost ratio.
fn eras_report(seeds: usize, seed0: u64) -> String {
    let mut text = String::new();
    text.push_str("win share % of the newer / ahead army (Hard / Human), open ground, mean of starts 2-5 hexes apart, four of that unit against the same Production of the other\n");
    text.push_str("same class, newer era over older\n");
    let class_names = ["melee", "ranged", "mounted"];
    for (ci, class) in class_names.iter().enumerate() {
        for gap in 1..=3usize {
            let mut cells = Vec::new();
            for e in gap..TRIANGLE_TIERS.len() {
                let new = TRIANGLE_TIERS[e].1[ci];
                let old = TRIANGLE_TIERS[e - gap].1[ci];
                let [h, m] = blended_share(new, old, seeds, seed0);
                cells.push(format!(
                    "{}>{} {h:>3}/{m:>3}",
                    &TRIANGLE_TIERS[e].0[..3],
                    &TRIANGLE_TIERS[e - gap].0[..3]
                ));
            }
            text.push_str(&format!("{class:<8} +{gap}  {}\n", cells.join("  ")));
        }
    }
    text.push_str("counter-class unit one era behind against the class it counters: win share of the counter unit\n");
    for (label, counter, target) in [
        ("melee(e-1) vs mounted(e)", 0usize, 2usize),
        ("mounted(e-1) vs ranged(e)", 2, 1),
        ("ranged(e-1) vs melee(e)", 1, 0),
    ] {
        let mut cells = Vec::new();
        for e in 1..TRIANGLE_TIERS.len() {
            let behind = TRIANGLE_TIERS[e - 1].1[counter];
            let ahead = TRIANGLE_TIERS[e].1[target];
            let [h, m] = blended_share(ahead, behind, seeds, seed0);
            cells.push(format!(
                "{} {:>3}/{:>3}",
                &TRIANGLE_TIERS[e].0[..3],
                100 - h,
                100 - m
            ));
        }
        text.push_str(&format!("{label:<26} {}\n", cells.join("  ")));
    }
    text
}
/// The Keshig (`--keshig`): Mongol mounted armies against each opponent at equal Production,
/// Human policy, open ground, mean of starts 2 to 5 hexes apart, both arena sides. The Keshig is
/// the Mongols' Classical mounted unit (`data::KESHIG_KIND`, the Cataphract's slot), so its foes
/// are that era's units (and one Ancient line). Each row compares the Keshig (it charges or
/// shoots by value) with the Cataphract it replaces (the same Mongol rider at the Cataphract's
/// price, never shooting), and a India Cataphract for reference.
const KESHIG_FOES: [(&str, &[UnitKind]); 6] = [
    ("6 Phalangites", &[UnitKind::Phalangite; 6]),
    ("6 Composite Archers", &[UnitKind::CompositeArcher; 6]),
    ("6 Cataphracts", &[UnitKind::Cataphract; 6]),
    (
        "3 Phalangites + 3 Comp. Archers",
        &[
            UnitKind::Phalangite,
            UnitKind::Phalangite,
            UnitKind::Phalangite,
            UnitKind::CompositeArcher,
            UnitKind::CompositeArcher,
            UnitKind::CompositeArcher,
        ],
    ),
    (
        "4 Phalangites + 2 Cataphracts",
        &[
            UnitKind::Phalangite,
            UnitKind::Phalangite,
            UnitKind::Phalangite,
            UnitKind::Phalangite,
            UnitKind::Cataphract,
            UnitKind::Cataphract,
        ],
    ),
    ("6 Spearmen (Ancient)", &[UnitKind::Spearman; 6]),
];
/// Win share of `n` Mongol `rider`s against `foe` (India), Human policy, ties half.
fn rider_share(
    n: usize,
    rider_faction: Faction,
    no_bow: bool,
    foe: &[UnitKind],
    ground: Ground,
    seeds: usize,
    seed0: u64,
) -> i32 {
    let (mut points, mut games) = (0, 0);
    for d in [2, 3, 4, 5] {
        for k in 0..seeds {
            for swap in [false, true] {
                let riders = repeat(data::KESHIG_KIND, n.clamp(1, 18));
                let (left, right) = if swap {
                    (foe.to_vec(), riders)
                } else {
                    (riders, foe.to_vec())
                };
                let mut c = case(
                    "keshig",
                    "keshig",
                    Goal::Destroy,
                    ground,
                    d,
                    left,
                    right,
                    "valid_orders",
                );
                let rider_side = usize::from(swap);
                let mut factions = [Faction::India; 2];
                factions[rider_side] = rider_faction;
                c.factions = Some(factions);
                c.no_bow[rider_side] = no_bow;
                let r = play(&c, seed0 + k as u64, Policy::Human, false);
                let rider_won = if swap { "defender" } else { "attacker" };
                points += if r.winner == rider_won {
                    2
                } else if r.winner == "tie" {
                    1
                } else {
                    0
                };
                games += 2;
            }
        }
    }
    points * 100 / games
}
/// `rider_share` at a Production budget: the rider count is fractional, so blend the two whole
/// counts either side of it (as `blended_share` does).
#[allow(clippy::too_many_arguments)] // The report's axes, as `case` keeps them visible.
fn rider_blend(
    budget: i32,
    cost: i32,
    rider_faction: Faction,
    no_bow: bool,
    foe: &[UnitKind],
    ground: Ground,
    seeds: usize,
    seed0: u64,
) -> i32 {
    let exact = budget as f64 / cost as f64;
    let low = exact.floor() as usize;
    let frac = exact - low as f64;
    let a = rider_share(low, rider_faction, no_bow, foe, ground, seeds, seed0);
    let b = rider_share(low + 1, rider_faction, no_bow, foe, ground, seeds, seed0);
    ((1.0 - frac) * a as f64 + frac * b as f64).round() as i32
}
/// Share of assaults in which `n` Mongol riders take a city held by a Phalangite garrison with a
/// Phalangite and a Composite Archer beside it (Human policy, `Goal::TakeCity`).
fn rider_city_share(n: usize, no_bow: bool, walls: bool, seeds: usize, seed0: u64) -> i32 {
    let (mut met, mut games) = (0, 0);
    for d in [3, 4, 5] {
        for k in 0..seeds {
            let mut c = case(
                "keshig-city",
                "keshig",
                Goal::TakeCity,
                Ground::Open,
                d,
                repeat(data::KESHIG_KIND, n.clamp(1, 18)),
                vec![
                    UnitKind::Phalangite,
                    UnitKind::Phalangite,
                    UnitKind::CompositeArcher,
                ],
                "valid_orders",
            );
            c.city = true;
            c.garrison = true;
            c.walls = walls;
            c.factions = Some([Faction::Mongols, Faction::India]);
            c.no_bow[0] = no_bow;
            met += i32::from(play(&c, seed0 + k as u64, Policy::Human, false).goal_met);
            games += 1;
        }
    }
    met * 100 / games
}
fn keshig_report(seeds: usize, seed0: u64) -> String {
    let keshig = data::unit_stats_for(Faction::Mongols, data::KESHIG_KIND);
    let replaced = data::unit_stats(data::KESHIG_KIND);
    let mut text = format!(
        "Keshig: {} Production, {} melee, {} ranged at range {}, {} moves (Mongol); Cataphract {} Production\n\
         win share % of the riders, Human policy, mean of starts 2-5 hexes apart, both arena sides, equal Production\n\
         columns: Keshig (Mongol, charges or shoots) / Mongol Cataphract (same rider, no bow, Cataphract price) / India Cataphract\n",
        keshig.cost, keshig.strength, keshig.ranged, keshig.range, keshig.moves, replaced.cost
    );
    for ground in [Ground::Open, Ground::Forest] {
        for (label, foe) in KESHIG_FOES {
            let budget: i32 = foe.iter().map(|k| data::unit_stats(*k).cost).sum();
            let share = |cost: i32, faction: Faction, no_bow: bool| {
                rider_blend(budget, cost, faction, no_bow, foe, ground, seeds, seed0)
            };
            let k = share(keshig.cost, Faction::Mongols, false);
            let m = share(replaced.cost, Faction::Mongols, true);
            let c = share(replaced.cost, Faction::India, false);
            text.push_str(&format!(
                "{:<7} vs {label:<31} Keshig {k:>3}   Mongol Cataphract {m:>3}   India Cataphract {c:>3}\n",
                format!("{ground:?}")
            ));
        }
    }
    // An assault on a town: 450 Production of riders (six Cataphracts) against a garrisoned city.
    for walls in [false, true] {
        let blend = |cost: i32, no_bow: bool| {
            let exact = 450.0 / cost as f64;
            let low = exact.floor() as usize;
            let frac = exact - low as f64;
            let a = rider_city_share(low, no_bow, walls, seeds, seed0);
            let b = rider_city_share(low + 1, no_bow, walls, seeds, seed0);
            ((1.0 - frac) * a as f64 + frac * b as f64).round() as i32
        };
        text.push_str(&format!(
            "TakeCity {} (Phalangite garrison, Phalangite + Composite Archer beside it), 450 Production: Keshig {:>3}   Mongol Cataphract {:>3}\n",
            if walls { "walled" } else { "unwalled" },
            blend(keshig.cost, false),
            blend(replaced.cost, true)
        ));
    }
    text
}
/// Best responses by composition: every mix of one era's three classes within 8% of the tier's
/// budget, against four archetypes (line, gunline, horse, balanced). The triangle is working when
/// the best answer to each archetype differs and follows the counters, and no mix is best against
/// all of them. Human policy, open ground, four hexes apart, both sides of the arena.
fn response_report(seeds: usize, seed0: u64) -> String {
    let mut text = String::new();
    text.push_str("best responses by composition (units: melee/ranged/mounted), Human policy, open ground, d4\n");
    for (tier, units) in TRIANGLE_TIERS {
        let [m, r, h] = units;
        let budget = tier_budget(units);
        let cost = |k: UnitKind| data::unit_stats(k).cost;
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
        let [line, gun, horse] = doctrines(units);
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
        let mut grid = vec![vec![0i32; enemies.len()]; options.len()];
        for (i, &(nm, nr, nh)) in options.iter().enumerate() {
            let mine = build(nm, nr, nh);
            for (j, (_, foe)) in enemies.iter().enumerate() {
                grid[i][j] = army_share(&mine, foe, 4, Ground::Open, seeds, seed0)[1];
            }
        }
        text.push_str(&format!("{tier} ({} candidate mixes)\n", options.len()));
        for (j, (name, _)) in enemies.iter().enumerate() {
            let mut order: Vec<usize> = (0..options.len()).collect();
            order.sort_by_key(|&i| (-grid[i][j], i));
            let best: Vec<String> = order
                .iter()
                .take(3)
                .map(|&i| {
                    let (nm, nr, nh) = options[i];
                    format!("{nm}/{nr}/{nh}={}", grid[i][j])
                })
                .collect();
            text.push_str(&format!("  best vs {name:<8} {}\n", best.join("  ")));
        }
        // Dominance: a mix that scores at least 60% against every archetype.
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
            "  mixes that beat every archetype at 60%+: {}   best worst-case: {worst_best}\n",
            if dominant.is_empty() {
                "none".to_string()
            } else {
                dominant.join(", ")
            }
        ));
    }
    text
}
/// The Spartan phalanx (`--phalanx`, how `data::HOPLITE_PHALANX_STRENGTH` was chosen): a small
/// Spartan line of Hoplites (Spearmen) against the other side's army, win share % of the Spartans
/// (Hard / Human policy), mean of starts 2-5 hexes apart, open ground and forest. The first rows
/// say how much a pair beats a plain Spearman line; the counter-class rows say a ranged unit one
/// era ahead (and the era-ahead melee line) still does well against it. Spartans already win most
/// arena fights at equal Production (wound immunity, kills heal), so read the change between
/// strengths, not the level.
fn phalanx_report(seeds: usize, seed0: u64) -> String {
    let n = data::HOPLITE_PHALANX_STRENGTH;
    let mut text = format!(
        "phalanx +{n}: win share % of the Spartan Hoplites over the other army (Hard / Human), mean of starts 2-5 hexes apart\n"
    );
    let sparta = [Faction::Sparta, Faction::India];
    let rows: [(&str, usize, Vec<UnitKind>); 7] = [
        (
            "2 Spearmen (same era, same class)",
            2,
            repeat(UnitKind::Spearman, 2),
        ),
        (
            "3 Spearmen (outnumbered 2 to 3)",
            2,
            repeat(UnitKind::Spearman, 3),
        ),
        (
            "1 Composite Archer (ranged, one era ahead, same cost)",
            2,
            repeat(UnitKind::CompositeArcher, 1),
        ),
        (
            "2 Archers (ranged, same era; 132 against 120)",
            3,
            repeat(UnitKind::Archer, 2),
        ),
        (
            "2 Phalangites (melee, one era ahead; 110 against 120)",
            3,
            repeat(UnitKind::Phalangite, 2),
        ),
        (
            "3 Phalangites (outnumbered 3 to 3, one era ahead)",
            3,
            repeat(UnitKind::Phalangite, 3),
        ),
        (
            "2 Horsemen (the class Hoplites counter)",
            2,
            repeat(UnitKind::Horseman, 2),
        ),
    ];
    for (label, count, army) in rows {
        let hoplites = repeat(UnitKind::Spearman, count);
        let cells: Vec<_> = [Ground::Open, Ground::Forest]
            .into_iter()
            .map(|g| {
                let [hard, human] = faction_share(sparta, &hoplites, &army, g, seeds, seed0);
                format!("{g:?} {hard:>3}/{human:>3}")
            })
            .collect();
        text.push_str(&format!(
            "{label:<56} ({}v{})  {}\n",
            hoplites.len(),
            army.len(),
            cells.join("   ")
        ));
    }
    text
}
/// `army_share` with each side's faction: `a` plays `factions[0]`, `b` plays `factions[1]` (swapped with the sides).
fn faction_share(
    factions: [Faction; 2],
    a: &[UnitKind],
    b: &[UnitKind],
    ground: Ground,
    seeds: usize,
    seed0: u64,
) -> [i32; 2] {
    let mut sum = [0, 0];
    let distances = [2, 3, 4, 5];
    for distance in distances {
        for (pi, policy) in [Policy::Hard, Policy::Human].into_iter().enumerate() {
            let (mut points, mut n) = (0, 0);
            for k in 0..seeds {
                for swap in [false, true] {
                    let (left, right) = if swap { (b, a) } else { (a, b) };
                    let mut c = case(
                        "phalanx",
                        "phalanx",
                        Goal::Destroy,
                        ground,
                        distance,
                        left.to_vec(),
                        right.to_vec(),
                        "valid_orders",
                    );
                    c.factions = Some(if swap {
                        [factions[1], factions[0]]
                    } else {
                        factions
                    });
                    let r = play(&c, seed0 + k as u64, policy, false);
                    let a_side = if swap { "defender" } else { "attacker" };
                    points += if r.winner == a_side {
                        2
                    } else if r.winner == "tie" {
                        1
                    } else {
                        0
                    };
                    n += 2;
                }
            }
            sum[pi] += points * 100 / n;
        }
    }
    [
        sum[0] / distances.len() as i32,
        sum[1] / distances.len() as i32,
    ]
}
fn arg(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|v| v[0] == key)
        .map_or(default.to_owned(), |v| v[1].clone())
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--army-measure") {
        army_measure::run(&args);
        return;
    }
    let rules = arg(&args, "--rules", "default");
    let roster = roster(&rules).unwrap_or_else(|e| panic!("{e}"));
    let seeds: usize = arg(&args, "--games", "4").parse().unwrap();
    let start_seed: u64 = arg(&args, "--seed", "101000").parse().unwrap();
    let out = PathBuf::from(arg(&args, "--out-dir", "."));
    let label = arg(&args, "--label", "local");
    if args.iter().any(|a| a == "--responses") {
        let report = response_report(seeds, start_seed);
        print!("{report}");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join(format!("responses-{label}.txt")), &report).unwrap();
        return;
    }
    if args.iter().any(|a| a == "--mounted") {
        let report = mounted::report(seeds, start_seed);
        print!("{report}");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join(format!("mounted-{label}.txt")), &report).unwrap();
        return;
    }
    if args.iter().any(|a| a == "--keshig") {
        let report = keshig_report(seeds, start_seed);
        print!("{report}");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join(format!("keshig-{label}.txt")), &report).unwrap();
        return;
    }
    if args.iter().any(|a| a == "--eras") {
        let report = eras_report(seeds, start_seed);
        print!("{report}");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join(format!("eras-{label}.txt")), &report).unwrap();
        return;
    }
    if args.iter().any(|a| a == "--phalanx") {
        let report = phalanx_report(seeds, start_seed);
        print!("{report}");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join(format!("phalanx-{label}.txt")), &report).unwrap();
        return;
    }
    if args.iter().any(|a| a == "--triangle") {
        let report = triangle_report(seeds, start_seed);
        print!("{report}");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join(format!("triangle-{label}.txt")), &report).unwrap();
        return;
    }
    if let Some(path) = args
        .windows(2)
        .find(|v| v[0] == "--replay-fixtures")
        .map(|v| &v[1])
    {
        let fixtures: Vec<ResultRow> = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let mut drift = 0;
        for old in &fixtures {
            let now = play(&old.case, old.seed, old.policy, false);
            if (
                now.winner,
                now.hp,
                now.goal_met,
                now.invalid_orders,
                now.turns,
            ) != (
                old.winner.clone(),
                old.hp,
                old.goal_met,
                old.invalid_orders,
                old.turns,
            ) {
                drift += 1;
            }
        }
        println!(
            "replayed {} fixtures; {} changed outcomes",
            fixtures.len(),
            drift
        );
        if drift > 0 {
            std::process::exit(1)
        };
        return;
    }
    let start = Instant::now();
    fs::create_dir_all(&out).unwrap();
    let cases = cases(&roster);
    let mut rows = Vec::new();
    for c in &cases {
        for n in 0..seeds {
            for policy in [Policy::Hard, Policy::Human] {
                rows.push(play(c, start_seed + n as u64, policy, false));
            }
        }
    }
    let mut failed = Vec::new();
    let mut categories: BTreeMap<String, BTreeMap<String, (usize, usize, usize)>> = BTreeMap::new();
    for c in &cases {
        for policy in [Policy::Hard, Policy::Human] {
            let subset: Vec<_> = rows
                .iter()
                .filter(|r| r.case.id == c.id && r.policy == policy)
                .collect();
            let verdict = passed(c, &subset);
            let key = match policy {
                Policy::Hard => "hard",
                Policy::Human => "human",
            };
            let entry = categories
                .entry(c.category.clone())
                .or_default()
                .entry(key.into())
                .or_default();
            entry.0 += 1;
            if let Some(ok) = verdict {
                entry.1 += 1;
                if ok {
                    entry.2 += 1;
                } else {
                    let seed = subset
                        .iter()
                        .find(|r| violating_trial(c, r))
                        .map_or(start_seed, |r| r.seed);
                    failed.push(play(c, seed, policy, true));
                }
            }
        }
    }
    let composition_rows = search(&roster, start_seed);
    let elapsed_ms = start.elapsed().as_millis();
    let candidates = compositions(&roster, COMPOSITION_BUDGET).len();
    fs::write(
        out.join(format!("combat-{label}-raw.json")),
        serde_json::to_vec(&rows).unwrap(),
    )
    .unwrap();
    fs::write(
        out.join(format!("combat-{label}-fixtures.json")),
        serde_json::to_vec_pretty(&failed).unwrap(),
    )
    .unwrap();
    fs::write(out.join(format!("combat-{label}-summary.json")),serde_json::to_vec_pretty(&serde_json::json!({"rules":rules,"seed":start_seed,"seeds_per_case":seeds,"case_count":cases.len(),"category_counts":categories,"failed_cases":failed.iter().map(|r|&r.case.id).collect::<Vec<_>>(),"composition_budget":COMPOSITION_BUDGET,"composition_candidates":candidates,"compositions":composition_rows,"elapsed_ms":elapsed_ms})).unwrap()).unwrap();
    println!(
        "{} cases, {} trials, {} failed expectations, {}s",
        cases.len(),
        rows.len(),
        failed.len(),
        start.elapsed().as_secs()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_covers_goals_force_sizes_and_city_states() {
        let all = cases(&roster("default").unwrap());
        // Four default unit kinds produce 84 open matchups; the other families add 50.
        assert_eq!(all.len(), 134);
        // Four current-cost Spearmen exactly fill the 320-production composition budget.
        assert!(
            compositions(&roster("default").unwrap(), COMPOSITION_BUDGET)
                .contains(&vec![UnitKind::Spearman; 4])
        );
        assert!(
            all.iter()
                .any(|c| c.attacker.len() == 10 && c.defender.len() == 1)
        );
        assert!(
            all.iter()
                .any(|c| c.attacker.len() == 3 && c.defender.len() == 6)
        );
        assert!(
            all.iter()
                .any(|c| c.city && c.walls && c.garrison && c.city_hp == CITY_HP)
        );
        assert!(
            all.iter()
                .any(|c| c.city && !c.walls && !c.garrison && c.city_hp < CITY_HP)
        );
        for goal in [
            Goal::Destroy,
            Goal::TakeCity,
            Goal::DefendCity,
            Goal::HoldPass,
            Goal::RaidSettler,
            Goal::EscortSettler,
            Goal::RelieveCity,
            Goal::BreakSiege,
            Goal::Retreat,
        ] {
            assert!(
                all.iter()
                    .any(|c| std::mem::discriminant(&c.goal) == std::mem::discriminant(&goal))
            );
        }
    }

    #[test]
    fn tactical_case_repeats_from_seed() {
        let c = case(
            "repeat",
            "test",
            Goal::Destroy,
            Ground::Open,
            2,
            vec![UnitKind::Spearman, UnitKind::Archer],
            vec![UnitKind::Spearman],
            "valid_orders",
        );
        let a = play(&c, 9181, Policy::Human, true);
        let b = play(&c, 9181, Policy::Human, true);
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert_eq!(a.invalid_orders, 0);
    }
}
