//! B13's single-threaded, fixed-input release profiler. No AI policy changes or new deps.
//!
//! `perf_profile record target/performance-pass/corpus`
//! `perf_profile measure target/performance-pass/corpus 31 5 5 > samples.csv`
//! Recording is exclusive-create. Measurement replays recorded orders, checks every state
//! hash and AI decision, and compares complete debug snapshots before starting any timers.
//! The core integration test additionally records/checks canonical JSON states and views.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{
    BuildingKind, CityId, GameSetup, GameState, Hex, Order, PlayerId, PlayerView, ProductionItem,
    TechId, UnitId, UnitKind, new_game, player_view, resolve_turn, visibility,
};
use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

type Orders = BTreeMap<PlayerId, Vec<Order>>;

struct Frame {
    stage: &'static str,
    state: GameState,
    views: Vec<PlayerView>,
    orders: Orders,
}

fn views(state: &GameState) -> Vec<PlayerView> {
    state
        .players
        .iter()
        .map(|p| player_view(state, p.id))
        .collect()
}

fn decisions(state: &GameState, views: &[PlayerView]) -> Orders {
    state
        .players
        .iter()
        .zip(views)
        .filter(|(p, _)| p.alive)
        .map(|(p, v)| {
            let mut brain = ScriptedBrain::new(p.faction, state.setup.seed, p.id);
            (p.id, brain.orders(v))
        })
        .collect()
}

// Numeric tape: the original four fields, followed only for response explanations by
// JSON UTF-8 bytes as integers. This preserves old rows and arbitrary Unicode/comma/newline
// reasons without CSV escaping or a new dependency. Enum numbers are indices in ALL.
fn encode(order: &Order) -> Vec<i32> {
    let mut encoded = match *order {
        Order::BuyTile { .. } => panic!("AI seats do not buy tiles"),
        Order::HireCompany { city, unit, seller } => {
            [91, city.0 as i32, unit as i32, i32::from(seller.0)]
        }
        Order::DeclareWar { against, breach } => [9, i32::from(against.0), i32::from(breach), 0],
        Order::BreakTreaty { with, .. } => [14, i32::from(with.0), 0, 0],
        Order::ProposeTreaty { to, kind } => [10, i32::from(to.0), kind as i32, 0],
        Order::RespondToProposal {
            proposal, accept, ..
        } => [11, proposal.0 as i32, i32::from(accept), 0],
        Order::WithdrawProposal { proposal } => [12, proposal.0 as i32, 0, 0],
        Order::SetAutomation { .. }
        | Order::SetAutoFire { .. }
        | Order::SetCityAutomation { .. }
        | Order::SleepUnit { .. } => {
            unreachable!("empire AI does not issue client automation orders")
        }
        Order::MoveUnit { unit, to } => [0, unit.0 as i32, to.q, to.r],
        Order::SkipUnit { unit } => [1, unit.0 as i32, 0, 0],
        Order::FoundCity { unit } => [2, unit.0 as i32, 0, 0],
        Order::DisbandUnit { unit } => [3, unit.0 as i32, 0, 0],
        Order::Raze { city } => [92, city.0 as i32, 0, 0],
        Order::SetProduction { city, item } => match item {
            ProductionItem::Unit { unit } => [4, city.0 as i32, 0, unit as i32],
            ProductionItem::Building { building } => [4, city.0 as i32, 1, building as i32],
            ProductionItem::Wonder { building, tile } => {
                [4, city.0 as i32, building as i32 * 1000 + tile.q, tile.r]
            }
            ProductionItem::Project { project } => [4, city.0 as i32, 2, project as i32],
            ProductionItem::Improvement { improvement, tile } => [
                16,
                city.0 as i32,
                improvement as i32 * 1000 + tile.q,
                tile.r,
            ],
        },
        Order::BuyItem { city, item } => match item {
            ProductionItem::Unit { unit } => [5, city.0 as i32, 0, unit as i32],
            ProductionItem::Building { building } => [5, city.0 as i32, 1, building as i32],
            ProductionItem::Wonder { building, tile } => {
                [5, city.0 as i32, building as i32 * 1000 + tile.q, tile.r]
            }
            ProductionItem::Project { .. } => panic!("projects cannot be purchased"),
            ProductionItem::Improvement { .. } => {
                panic!("the profiling tape never buys improvements")
            }
        },
        Order::SetProductionQueue { .. } => {
            panic!("numeric profiling tape does not encode player queues")
        }
        Order::SetResearch { tech } => [6, tech as i32, 0, 0],
        Order::SetScienceConversion { to } => [18, to as i32, 0, 0],
        // Player plans use the standard JSON tape rather than this AI-only codec.
        Order::SetResearchQueue { .. } | Order::MoveUnitPath { .. } => {
            panic!("variable-length plans require the standard JSON tape format")
        }
        Order::RangedAttack { unit, target } => [7, unit.0 as i32, target.q, target.r],
        Order::UpgradeUnit { unit } => [8, unit.0 as i32, 0, 0],
        Order::UpgradeSiege { unit } => [17, unit.0 as i32, 0, 0],
        Order::Action { .. } => panic!("numeric profiling tape does not encode empire actions"),
        Order::GiveGold { .. } => panic!("numeric profiling tape does not encode Gold gifts"),
        Order::AirStrike { .. }
        | Order::Rebase { .. }
        | Order::AirRecon { .. }
        | Order::UpgradeAircraft { .. } => {
            panic!("numeric profiling tape does not encode air orders")
        }
    }
    .to_vec();
    if let Order::RespondToProposal {
        explanation: Some(explanation),
        ..
    } = order
    {
        encoded.extend(
            serde_json::to_vec(explanation)
                .unwrap()
                .into_iter()
                .map(i32::from),
        );
    }
    encoded
}

fn decode(encoded: impl AsRef<[i32]>) -> Order {
    let encoded = encoded.as_ref();
    let [kind, a, b, c]: [i32; 4] = encoded[..4].try_into().unwrap();
    assert!(kind == 11 || encoded.len() == 4, "unexpected order payload");
    let unit = UnitId(a as u32);
    let city = CityId(a as u32);
    match kind {
        92 => Order::Raze { city },
        91 => Order::HireCompany {
            city,
            unit: legatum_core::UnitKind::ALL[b as usize],
            seller: legatum_core::PlayerId(c as u8),
        },
        9 => Order::DeclareWar {
            against: legatum_core::TeamId(a as u8),
            breach: b != 0,
        },
        10 => Order::ProposeTreaty {
            to: legatum_core::TeamId(a as u8),
            kind: match b {
                0 => legatum_core::diplomacy::TreatyKind::Peace,
                1 => legatum_core::diplomacy::TreatyKind::Alliance,
                _ => panic!("invalid treaty kind"),
            },
        },
        11 => Order::RespondToProposal {
            proposal: legatum_core::diplomacy::ProposalId(a as u32),
            accept: b != 0,
            explanation: (encoded.len() > 4).then(|| {
                let json: Vec<_> = encoded[4..]
                    .iter()
                    .map(|&byte| u8::try_from(byte).expect("UTF-8 byte"))
                    .collect();
                serde_json::from_slice(&json).expect("response explanation")
            }),
        },
        12 => Order::WithdrawProposal {
            proposal: legatum_core::diplomacy::ProposalId(a as u32),
        },
        14 => Order::BreakTreaty {
            with: legatum_core::TeamId(a as u8),
            agreement: legatum_core::diplomacy::Agreement::Security,
        },
        16 => Order::SetProduction {
            city,
            item: ProductionItem::Improvement {
                improvement: legatum_core::ImprovementKind::ALL[(b / 1000) as usize],
                tile: Hex::new(b % 1000, c),
            },
        },
        0 => Order::MoveUnit {
            unit,
            to: Hex::new(b, c),
        },
        1 => Order::SkipUnit { unit },
        2 => Order::FoundCity { unit },
        3 => Order::DisbandUnit { unit },
        4 => Order::SetProduction {
            city,
            item: match b {
                0 => ProductionItem::Unit {
                    unit: UnitKind::ALL[c as usize],
                },
                1 => ProductionItem::Building {
                    building: BuildingKind::ALL[c as usize],
                },
                _ => panic!("invalid production kind"),
            },
        },
        5 => Order::BuyItem {
            city,
            item: match b {
                0 => ProductionItem::Unit {
                    unit: UnitKind::ALL[c as usize],
                },
                1 => ProductionItem::Building {
                    building: BuildingKind::ALL[c as usize],
                },
                _ => panic!("invalid purchase kind"),
            },
        },
        6 => Order::SetResearch {
            tech: TechId::ALL[a as usize],
        },
        18 => Order::SetScienceConversion {
            to: match a {
                0 => legatum_core::ScienceConversion::Gold,
                1 => legatum_core::ScienceConversion::Food,
                2 => legatum_core::ScienceConversion::Production,
                _ => panic!("invalid Science conversion"),
            },
        },
        7 => Order::RangedAttack {
            unit,
            target: Hex::new(b, c),
        },
        8 => Order::UpgradeUnit { unit },
        17 => Order::UpgradeSiege { unit },
        _ => panic!("invalid order kind"),
    }
}

fn read_orders(path: &Path) -> BTreeMap<u32, Orders> {
    parse_orders(&fs::read_to_string(path).unwrap())
}

fn parse_orders(csv: &str) -> BTreeMap<u32, Orders> {
    let mut tape = BTreeMap::<u32, Orders>::new();
    for row in csv.lines().skip(1) {
        let n: Vec<i32> = row.split(',').map(|s| s.parse().unwrap()).collect();
        assert!(n.len() >= 6);
        let orders = tape
            .entry(n[0] as u32)
            .or_default()
            .entry(PlayerId(n[1] as u8))
            .or_default();
        // -1 records a live player with zero orders, preserving the exact map.
        if n[2] != -1 {
            orders.push(decode(&n[2..]));
        }
    }
    tape
}

fn exclusive_write(path: &Path, bytes: impl AsRef<[u8]>) {
    use std::io::Write;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap()
        .write_all(bytes.as_ref())
        .unwrap();
}

fn corpus(root: &Path, record: bool) -> Vec<(String, Frame)> {
    let mut frames = Vec::new();
    for (name, width, height, shared) in [
        ("sim48", 48, 32, false),
        ("team48", 48, 32, true),
        ("prd72", 72, 44, false),
    ] {
        for seed in [1, 7] {
            let name = format!("{name}-seed{seed}");
            let dir = root.join(&name);
            let mut setup = GameSetup::all_ai(seed);
            setup.width = width;
            setup.height = height;
            setup.turn_limit = Some(150);
            if shared {
                setup.players[1].team = setup.players[0].team;
            }
            if record {
                fs::create_dir_all(&dir).unwrap();
                exclusive_write(&dir.join("setup.txt"), format!("{setup:?}\n"));
            } else {
                assert_eq!(
                    fs::read_to_string(dir.join("setup.txt")).unwrap(),
                    format!("{setup:?}\n")
                );
            }
            let tape = if record {
                BTreeMap::new()
            } else {
                read_orders(&dir.join("orders.csv"))
            };
            let mut rows = String::from("turn,player,kind,a,b,c,explanation_utf8...\n");
            let mut hashes = String::from("turn,state_hash\n");
            let mut state = new_game(&setup);
            let mut selected = Vec::new();
            let mut last = None;
            hashes.push_str(&format!("{},{}\n", state.turn, state.hash()));
            while !state.finished {
                assert!(state.turn <= 150, "unbounded game");
                let views = views(&state);
                let generated = decisions(&state, &views);
                let orders = if record {
                    generated.clone()
                } else {
                    let recorded = &tape[&state.turn];
                    assert_eq!(
                        &generated, recorded,
                        "AI changed: {name} turn {}",
                        state.turn
                    );
                    recorded.clone()
                };
                for (p, os) in &orders {
                    if os.is_empty() {
                        rows.push_str(&format!("{},{},-1,0,0,0\n", state.turn, p.0));
                    }
                    for o in os {
                        let encoded = encode(o);
                        assert_eq!(decode(&encoded), *o, "tape codec");
                        let fields = encoded
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        rows.push_str(&format!("{},{},{fields}\n", state.turn, p.0));
                    }
                }
                let (next, _) = resolve_turn(&state, &orders);
                let stage = match state.turn {
                    5 => "early",
                    50 => "middle",
                    _ => "late",
                };
                let frame = Frame {
                    stage,
                    state,
                    views,
                    orders,
                };
                if stage == "late" {
                    last = Some(frame);
                } else {
                    selected.push(frame);
                }
                state = next;
                hashes.push_str(&format!("{},{}\n", state.turn, state.hash()));
            }
            assert_eq!(selected.len(), 2, "case must reach the middle game");
            selected.push(last.expect("late game"));
            for frame in selected {
                let path = dir.join(format!("{}.snapshot.txt", frame.stage));
                let snapshot =
                    format!("{:?}\n{:?}\n{:?}\n", frame.state, frame.views, frame.orders);
                if record {
                    exclusive_write(&path, snapshot);
                } else {
                    assert_eq!(
                        fs::read_to_string(path).unwrap(),
                        snapshot,
                        "{name} snapshot differs"
                    );
                }
                frames.push((name.clone(), frame));
            }
            if record {
                exclusive_write(&dir.join("orders.csv"), rows);
                exclusive_write(&dir.join("hashes.csv"), hashes);
            } else {
                assert_eq!(
                    fs::read_to_string(dir.join("hashes.csv")).unwrap(),
                    hashes,
                    "{name} hashes differ"
                );
                assert_eq!(tape.len(), state.turn as usize - 1);
            }
            eprintln!("checked {name}: {} recorded turns", state.turn - 1);
        }
    }
    frames
}

fn measure(frame: &Frame, metric: &str, batch: usize) -> u128 {
    // Mutable observations start from identical independent states. Cloning happens outside
    // this timer, while resolve_turn retains its normal internal clone. No stale cache.
    let mut scratch = if metric == "observe" {
        (0..batch).map(|_| frame.state.clone()).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let mut observations = scratch.iter_mut();
    let start = Instant::now();
    for _ in 0..batch {
        let state = black_box(&frame.state);
        match metric {
            "resolve" => {
                black_box(resolve_turn(state, black_box(&frame.orders)));
            }
            "views" => {
                black_box(views(state));
            }
            "ai" => {
                black_box(decisions(state, black_box(&frame.views)));
            }
            "total" => {
                let views = views(state);
                let orders = decisions(state, &views);
                black_box(resolve_turn(state, &orders));
            }
            "observe" => {
                visibility::update_explored(black_box(observations.next().unwrap()));
            }
            "visible" => {
                for team in state.teams() {
                    black_box(visibility::visible_tiles(state, team));
                }
            }
            "clone" => {
                black_box(state.clone());
            }
            _ => unreachable!(),
        }
    }
    start.elapsed().as_nanos()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 3,
        "usage: perf_profile <record|measure> <corpus-dir> [samples=31] [warmup=5] [batch=5]"
    );
    let record = match args[1].as_str() {
        "record" => true,
        "measure" => false,
        _ => panic!("unknown mode"),
    };
    let frames = corpus(Path::new(&args[2]), record);
    if record {
        return;
    }
    if cfg!(debug_assertions) {
        panic!("profile with a release build");
    }
    let arg = |i: usize, default| args.get(i).map_or(default, |s| s.parse::<usize>().unwrap());
    let (samples, warmup, batch) = (arg(3, 31), arg(4, 5), arg(5, 5));
    assert!(samples > 0 && warmup > 0 && batch > 0);
    let metrics = [
        "resolve", "views", "ai", "total", "observe", "visible", "clone",
    ];
    println!("case,stage,turn,tiles,units,cities,metric,sample,batch,ns");
    for (name, frame) in frames {
        // Rotate metric order across samples to avoid always warming one component first.
        for sample in 0..warmup + samples {
            for j in 0..metrics.len() {
                let metric = metrics[(sample + j) % metrics.len()];
                let elapsed = measure(&frame, metric, batch);
                if sample >= warmup {
                    println!(
                        "{name},{},{},{},{},{},{metric},{},{batch},{elapsed}",
                        frame.stage,
                        frame.state.turn,
                        frame.state.map.len(),
                        frame.state.units.len(),
                        frame.state.cities.len(),
                        sample - warmup
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod codec_tests {
    use super::*;
    use legatum_core::{
        TeamId,
        diplomacy::{DecisionExplanation, ProposalId, TreatyKind},
    };

    fn assert_round_trip(order: Order) {
        let encoded = encode(&order);
        assert_eq!(decode(&encoded), order);
        let row = encoded
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let tape = parse_orders(&format!(
            "turn,player,kind,a,b,c,explanation_utf8...\n1,0,{row}\n1,1,-1,0,0,0\n"
        ));
        assert_eq!(tape[&1][&PlayerId(0)], vec![order]);
        assert!(tape[&1][&PlayerId(1)].is_empty());
    }

    #[test]
    fn every_treaty_kind_round_trips() {
        for kind in [TreatyKind::Peace, TreatyKind::Alliance] {
            let order = Order::ProposeTreaty {
                to: TeamId(3),
                kind,
            };
            assert_round_trip(order);
        }
    }

    #[test]
    fn response_explanations_round_trip() {
        for accept in [false, true] {
            for explanation in [
                None,
                Some(DecisionExplanation {
                    score: None,
                    reasons: vec![],
                }),
                Some(DecisionExplanation {
                    score: Some(-12),
                    reasons: vec!["Trust: -12, \"ally\"\nΔ".into(), "".into()],
                }),
            ] {
                let order = Order::RespondToProposal {
                    proposal: ProposalId(u32::MAX),
                    accept,
                    explanation,
                };
                assert_round_trip(order);
            }
        }
    }
}
