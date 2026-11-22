//! How side A loses cities in a 1v1 duel-matrix run (e.g. plain Hard against the proxy).
//!   duel_loss_audit <tapes-dir>
//! Replays every `game-NNNNN-seed-S.json` tape (side A is seat NNNNN mod 2) and, for each city
//! side A loses, reports what side A could see beforehand: how many consecutive turns a
//! visible enemy military unit stood within 3 hexes of the city, whether a side-A military
//! unit stood in the city when the losing turn began, how many side-A military units were
//! within 3 hexes, and whether side A took the city back within 10 turns. Also how each game
//! ended.
//!   duel_loss_audit <tapes-dir> <file-name-prefix>
//! also prints the five turns before each loss in the matching games: side A's military within 8
//! of the city (kind, HP, distance, order) and side B's within 5.
use legatum_core::{Event, GameState, PlayerId, player_view, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

#[derive(Default)]
struct Loss {
    warned: u32,
    garrisoned: bool,
    defenders: u32,
    attackers: u32,
    our_army: u32,
    their_army: u32,
    turn: u32,
    // Our military by distance from the city: 4-6, 7-10, over 10; and in another own city.
    bands: [u32; 4],
    retaken: bool,
    // In the three turns after the loss: side A had more military within 3 than the captor.
    could_retake: bool,
    // ... and the city had at most half its HP then.
    weak_after: bool,
}

fn military(state: &GameState, kind: legatum_core::UnitKind, owner: PlayerId) -> bool {
    let s = legatum_core::data::unit_stats_for(state.player(owner).faction, kind);
    s.military && kind != legatum_core::UnitKind::Scout
}

fn main() {
    let dir = std::env::args().nth(1).expect("tapes dir");
    let trace = std::env::args().nth(2);
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut losses: Vec<Loss> = Vec::new();
    let (mut games, mut a_wins, mut a_eliminated) = (0, 0, 0);
    let mut ended = BTreeMap::<String, u32>::new();
    // Per side (A, B): military units lost, damage dealt to the other side's units and cities,
    // military units built or bought by turn 100, and over the whole game.
    let mut lost = [0u64; 2];
    let mut dealt = [0u64; 2];
    let mut built100 = [0u64; 2];
    let mut built = [0u64; 2];
    // What side A's military within 6 of a city did on the turn before the city fell.
    let mut acts = BTreeMap::<String, u32>::new();
    let mut kinds = BTreeMap::<String, u32>::new();
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let index: usize = name[5..10].parse().expect("game index");
        let a = PlayerId((index % 2) as u8);
        let tape = Tape::from_json(&std::fs::read_to_string(file).unwrap()).unwrap();
        let mut state = tape.initial_state();
        // City id -> consecutive turns a visible enemy unit stood within 3 hexes.
        let mut warned: BTreeMap<legatum_core::CityId, u32> = BTreeMap::new();
        let mut open: Vec<(legatum_core::CityId, u32, usize)> = Vec::new();
        let tracing = trace.as_ref().is_some_and(|p| name.starts_with(p.as_str()));
        let mut history: std::collections::VecDeque<(GameState, &legatum_wasm::tape::TapeTurn)> =
            std::collections::VecDeque::new();
        for t in &tape.turns {
            if tracing {
                history.push_back((state.clone(), t));
                if history.len() > 5 {
                    history.pop_front();
                }
            }
            for (id, at, li) in &open {
                if t.turn > *at
                    && t.turn <= at + 3
                    && let Some(c) = state.cities.get(id)
                    && c.owner != a
                {
                    let count = |p: PlayerId| {
                        state
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == p
                                    && military(&state, u.kind, p)
                                    && u.pos.distance(c.pos) <= 3
                            })
                            .count()
                    };
                    if count(a) > count(c.owner) {
                        losses[*li].could_retake = true;
                        if c.hp * 2 <= legatum_core::data::CITY_HP {
                            losses[*li].weak_after = true;
                        }
                    }
                }
            }
            let view = player_view(&state, a);
            let mut near = BTreeMap::new();
            for c in state.cities.values().filter(|c| c.owner == a) {
                let threatened = view.units.iter().any(|u| {
                    u.owner != a
                        && !u.owner.is_barbarian()
                        && u.military
                        && u.kind != legatum_core::UnitKind::Scout
                        && u.pos.distance(c.pos) <= 3
                });
                let streak = if threatened {
                    warned.get(&c.id).copied().unwrap_or(0) + 1
                } else {
                    0
                };
                near.insert(c.id, streak);
            }
            let before = state.clone();
            let (next, events) = resolve_turn(&state, &t.orders);
            let side = |p: PlayerId| {
                if p == a {
                    Some(0)
                } else if !p.is_barbarian() {
                    Some(1)
                } else {
                    None
                }
            };
            for e in &events {
                match e {
                    Event::UnitDestroyed {
                        owner, unit_kind, ..
                    } if military(&before, *unit_kind, *owner) => {
                        if let Some(k) = side(*owner) {
                            lost[k] += 1;
                        }
                    }
                    Event::Battle {
                        attacker_owner,
                        defender,
                        damage_to_attacker,
                        damage_to_defender,
                        ..
                    } => {
                        let d_owner = match defender {
                            legatum_core::events::Defender::Unit { owner, .. }
                            | legatum_core::events::Defender::City { owner, .. } => *owner,
                        };
                        if let (Some(x), Some(y)) = (side(*attacker_owner), side(d_owner))
                            && x != y
                        {
                            dealt[x] += *damage_to_defender as u64;
                            dealt[y] += *damage_to_attacker as u64;
                        }
                    }
                    Event::ProductionCompleted {
                        owner,
                        item: legatum_core::ProductionItem::Unit { unit },
                        ..
                    }
                    | Event::ItemBought {
                        owner,
                        item: legatum_core::ProductionItem::Unit { unit },
                        ..
                    } if military(&before, *unit, *owner) => {
                        if let Some(k) = side(*owner) {
                            *kinds
                                .entry(format!("{}{:?}", if k == 0 { "A " } else { "B " }, unit))
                                .or_default() += 1;
                            built[k] += 1;
                            if t.turn <= 100 {
                                built100[k] += 1;
                            }
                        }
                    }
                    _ => {}
                }
                if let Event::CityCaptured { city, from, to, .. } = e {
                    if *from == a {
                        let c = &before.cities[city];
                        for u in before.units.values().filter(|u| {
                            u.owner == a
                                && military(&before, u.kind, a)
                                && u.pos.distance(c.pos) <= 6
                        }) {
                            let order = t.orders.get(&a).and_then(|os| {
                                os.iter().find(|o| match o {
                                    legatum_core::Order::MoveUnit { unit, .. }
                                    | legatum_core::Order::MoveUnitPath { unit, .. }
                                    | legatum_core::Order::SkipUnit { unit }
                                    | legatum_core::Order::RangedAttack { unit, .. } => {
                                        *unit == u.id
                                    }
                                    _ => false,
                                })
                            });
                            let ring = if u.pos == c.pos {
                                "in city"
                            } else if before
                                .cities
                                .values()
                                .any(|o| o.owner == a && o.pos == u.pos)
                            {
                                "other city"
                            } else if u.pos.distance(c.pos) <= 3 {
                                "1-3"
                            } else {
                                "4-6"
                            };
                            let what = match order {
                                Some(legatum_core::Order::MoveUnit { to, .. }) => {
                                    let (d0, d1) = (u.pos.distance(c.pos), to.distance(c.pos));
                                    if before.units.values().any(|e| e.pos == *to && e.owner != a) {
                                        "attack".to_string()
                                    } else if d1 < d0 {
                                        "move closer".to_string()
                                    } else if d1 > d0 {
                                        "move away".to_string()
                                    } else {
                                        "move same ring".to_string()
                                    }
                                }
                                Some(legatum_core::Order::MoveUnitPath { .. }) => {
                                    "path move".to_string()
                                }
                                Some(legatum_core::Order::SkipUnit { .. }) => "hold".to_string(),
                                Some(legatum_core::Order::RangedAttack { .. }) => {
                                    "shoot".to_string()
                                }
                                _ => "no order".to_string(),
                            };
                            let hurt = if u.hp < 60 { " (hp<60)" } else { "" };
                            *acts.entry(format!("{ring}: {what}{hurt}")).or_default() += 1;
                        }
                        if tracing {
                            println!("{name}: {} lost on turn {} to {:?}", c.name, t.turn, to);
                            for (st, tt) in &history {
                                let cc = &st.cities[city];
                                println!(
                                    "  turn {} city hp {} pop {}",
                                    tt.turn, cc.hp, cc.population
                                );
                                for u in st.units.values() {
                                    let d = u.pos.distance(c.pos);
                                    let mil = military(st, u.kind, u.owner);
                                    if mil && u.owner == a && d <= 8 {
                                        let order = tt.orders.get(&a).and_then(|os| {
                                            os.iter().find(|o| {
                                                format!("{o:?}")
                                                    .contains(&format!("unit: {:?}", u.id))
                                            })
                                        });
                                        println!(
                                            "    A {:?} hp {} d {} fort {} -> {}",
                                            u.kind,
                                            u.hp,
                                            d,
                                            u.fortify,
                                            order.map_or("none".to_string(), |o| format!("{o:?}")
                                                .chars()
                                                .take(70)
                                                .collect())
                                        );
                                    } else if mil && u.owner == *to && d <= 5 {
                                        println!("    B {:?} hp {} d {}", u.kind, u.hp, d);
                                    }
                                }
                            }
                        }
                        let in_city = before.units.values().any(|u| {
                            u.owner == a && u.pos == c.pos && military(&before, u.kind, a)
                        });
                        let defenders = before
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == a
                                    && military(&before, u.kind, a)
                                    && u.pos.distance(c.pos) <= 3
                            })
                            .count() as u32;
                        let b = to;
                        let attackers = before
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == *b
                                    && military(&before, u.kind, *b)
                                    && u.pos.distance(c.pos) <= 3
                            })
                            .count() as u32;
                        let army = |p: PlayerId| {
                            before
                                .units
                                .values()
                                .filter(|u| u.owner == p && military(&before, u.kind, p))
                                .count() as u32
                        };
                        let mut bands = [0u32; 4];
                        for u in before
                            .units
                            .values()
                            .filter(|u| u.owner == a && military(&before, u.kind, a))
                        {
                            let d = u.pos.distance(c.pos);
                            if before
                                .cities
                                .values()
                                .any(|o| o.owner == a && o.pos == u.pos && o.id != c.id)
                            {
                                bands[3] += 1;
                            } else if (4..=6).contains(&d) {
                                bands[0] += 1;
                            } else if (7..=10).contains(&d) {
                                bands[1] += 1;
                            } else if d > 10 {
                                bands[2] += 1;
                            }
                        }
                        losses.push(Loss {
                            bands,
                            attackers,
                            our_army: army(a),
                            their_army: army(*b),
                            turn: t.turn,
                            warned: near.get(city).copied().unwrap_or(0).saturating_sub(1),
                            garrisoned: in_city,
                            defenders,
                            retaken: false,
                            could_retake: false,
                            weak_after: false,
                        });
                        open.push((*city, t.turn, losses.len() - 1));
                    } else if *to == a
                        && let Some(k) = open
                            .iter()
                            .position(|(id, at, _)| id == city && t.turn <= at + 10)
                    {
                        let (_, _, li) = open.remove(k);
                        losses[li].retaken = true;
                    }
                }
            }
            warned = near;
            state = next;
        }
        games += 1;
        let won = state.winner == Some(state.team_of(a));
        a_wins += u32::from(won);
        if !state.player(a).alive {
            a_eliminated += 1;
        }
        let how = if won {
            "side A won"
        } else if !state.player(a).alive {
            "side A eliminated"
        } else if state.winner.is_some() {
            "side B won by VP"
        } else {
            "no winner"
        };
        *ended.entry(how.to_string()).or_default() += 1;
    }
    let n = losses.len().max(1) as f64;
    let pct = |k: usize| format!("{:.0}%", 100.0 * k as f64 / n);
    println!("games {games}, side A wins {a_wins}, eliminated {a_eliminated}, endings {ended:?}");
    let g = games.max(1) as f64;
    println!(
        "per game (A vs B): military lost {:.1} vs {:.1}; damage dealt {:.0} vs {:.0}; military built by T100 {:.1} vs {:.1}, in all {:.1} vs {:.1}",
        lost[0] as f64 / g,
        lost[1] as f64 / g,
        dealt[0] as f64 / g,
        dealt[1] as f64 / g,
        built100[0] as f64 / g,
        built100[1] as f64 / g,
        built[0] as f64 / g,
        built[1] as f64 / g
    );
    println!("military built by kind: {kinds:?}");
    println!("cities side A lost: {}", losses.len());
    println!("  side A military within 6 on the losing turn: {acts:?}");
    println!(
        "  warned 3+ turns: {} ({}); warned 1-2: {}; unwarned: {}",
        losses.iter().filter(|l| l.warned >= 3).count(),
        pct(losses.iter().filter(|l| l.warned >= 3).count()),
        losses.iter().filter(|l| (1..3).contains(&l.warned)).count(),
        losses.iter().filter(|l| l.warned == 0).count()
    );
    println!(
        "  no garrison when the turn began: {} ({})",
        losses.iter().filter(|l| !l.garrisoned).count(),
        pct(losses.iter().filter(|l| !l.garrisoned).count())
    );
    println!(
        "  no side-A military within 3: {} ({})",
        losses.iter().filter(|l| l.defenders == 0).count(),
        pct(losses.iter().filter(|l| l.defenders == 0).count())
    );
    let mean = |f: &dyn Fn(&Loss) -> u32| losses.iter().map(|l| f(l) as f64).sum::<f64>() / n;
    println!(
        "  mean at the loss: our military within 3 {:.1}, theirs within 3 {:.1}; our army {:.1}, theirs {:.1}",
        mean(&|l| l.defenders),
        mean(&|l| l.attackers),
        mean(&|l| l.our_army),
        mean(&|l| l.their_army)
    );
    println!(
        "  our other military: 4-6 away {:.1}, 7-10 away {:.1}, over 10 {:.1}, garrisoning other cities {:.1}",
        mean(&|l| l.bands[0]),
        mean(&|l| l.bands[1]),
        mean(&|l| l.bands[2]),
        mean(&|l| l.bands[3])
    );
    let mut by_turn = BTreeMap::<u32, u32>::new();
    for l in &losses {
        *by_turn.entry(l.turn / 25 * 25).or_default() += 1;
    }
    println!("  losses by turn bucket: {by_turn:?}");
    let outnumbered = losses.iter().filter(|l| l.attackers > l.defenders).count();
    let out_army = losses.iter().filter(|l| l.their_army > l.our_army).count();
    println!(
        "  outnumbered locally: {} ({}); outnumbered overall: {} ({})",
        outnumbered,
        pct(outnumbered),
        out_army,
        pct(out_army)
    );
    let cr = losses.iter().filter(|l| l.could_retake).count();
    println!(
        "  outnumbered the captor locally within 3 turns after: {} ({}), with the city at half HP or less {}, of which retaken {}",
        cr,
        pct(cr),
        losses.iter().filter(|l| l.weak_after).count(),
        losses.iter().filter(|l| l.weak_after && l.retaken).count()
    );
    println!(
        "  retaken within 10 turns: {} ({})",
        losses.iter().filter(|l| l.retaken).count(),
        pct(losses.iter().filter(|l| l.retaken).count())
    );
}
