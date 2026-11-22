//! Deterministic real-engine unit matchup and campaign combat study.
//! Usage: unit_matchups --mode skirmish|campaign --out-dir DIR [--seed N] [--games N] [--players 4|8]

use legatum_ai::{
    Brain, ScriptedBrain,
    search::{SearchBrain, SearchConfig},
};
use legatum_core::{
    AiDifficulty, Event, GameSetup, Hex, Order, PlayerId, TeamId, Terrain,
    data::{self, UNIT_HP},
    events::{Attacker, Defender},
    new_game, resolve_turn,
    state::Unit,
    types::UnitKind,
};
use legatum_wasm::session::{OrderMap, brain_seed};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
};

const COMBAT_UNITS: [UnitKind; 6] = [
    UnitKind::Scout,
    UnitKind::Soldier,
    UnitKind::Ranger,
    UnitKind::Trooper,
    UnitKind::Marksman,
    UnitKind::AssaultTrooper,
];
type VictimCredit = (String, String, i32);

fn class(k: UnitKind) -> &'static str {
    match data::unit_class(k) {
        Some(legatum_core::UnitClass::Melee) => "melee",
        Some(legatum_core::UnitClass::Ranged) => "ranged",
        Some(legatum_core::UnitClass::Mounted) => "mounted",
        Some(legatum_core::UnitClass::Air) => "air",
        None if k == UnitKind::Scout => "scout",
        None => "civilian",
    }
}

fn requirement(k: UnitKind) -> String {
    data::unit_stats(k)
        .requires
        .map_or("None".into(), |x| format!("{x:?}"))
}

fn roster() -> Value {
    let rows: Vec<_> = legatum_core::Faction::ALL
        .into_iter()
        .flat_map(|f| UnitKind::ALL.into_iter().map(move |k| (f, k)))
        .map(|(f, k)| {
            let s = data::unit_stats_for(f, k);
            json!({"faction":format!("{f:?}"),"unit":format!("{k:?}"),"class":class(k),"cost":s.cost,
            "strength":s.strength,"ranged_strength":s.ranged,"range":s.range,
            "moves":s.moves,"requires":requirement(k)})
        })
        .collect();
    json!(rows)
}

#[derive(Default, Clone)]
struct PairStats {
    battles: u64,
    damage: u64,
    credited_kills_milli: u64,
}

#[derive(Default)]
struct CampaignStats {
    pairs: BTreeMap<(String, String), PairStats>,
    kills: BTreeMap<String, u64>,
    losses: BTreeMap<String, u64>,
    melee_ranged_contacts: u64,
    melee_killed_at_contact: u64,
    melee_hit_survived_contact: u64,
    melee_reached: BTreeSet<(u64, u32)>,
    melee_killed_before_reach: BTreeSet<(u64, u32)>,
    melee_killed_at_contact_units: BTreeSet<(u64, u32)>,
    melee_survived_contact_units: BTreeSet<(u64, u32)>,
    spent: BTreeMap<String, BTreeSet<(u64, u32, i32)>>,
    damage: BTreeMap<String, u64>,
}

fn observe(
    events: &[Event],
    turn: u32,
    game: u64,
    attacker_kinds: &BTreeMap<u32, (UnitKind, i32)>,
    stats: &mut CampaignStats,
) {
    let mut by_victim: BTreeMap<(u32, u32), Vec<VictimCredit>> = BTreeMap::new();
    for e in events {
        if let Event::UnitDestroyed { unit_kind, .. } = e {
            *stats.losses.entry(class(*unit_kind).into()).or_default() += 1;
        }
    }
    // Event damage values are from the complete resolution stage. Credit each killed defender
    // in proportion to the damage each distinct attacker class contributed in that stage.
    for e in events {
        let Event::Battle {
            attacker: Attacker::Unit(aid),
            defender,
            damage_to_defender,
            attacker_hp_after,
            defender_hp_after,
            ranged,
            ..
        } = e
        else {
            continue;
        };
        let (ak, unit_cost) = attacker_kinds
            .get(&aid.0)
            .copied()
            .unwrap_or((UnitKind::Scout, data::unit_stats(UnitKind::Scout).cost));
        let (victim, dc) = match defender {
            Defender::Unit {
                unit, unit_kind, ..
            } => (Some(unit.0), class(*unit_kind)),
            Defender::City { .. } => (None, "city"),
        };
        let ac = class(ak).to_string();
        let dc = dc.to_string();
        let p = stats.pairs.entry((ac.clone(), dc.clone())).or_default();
        p.battles += 1;
        p.damage += *damage_to_defender as u64;
        *stats.damage.entry(ac.clone()).or_default() += *damage_to_defender as u64;
        stats
            .spent
            .entry(ac.clone())
            .or_default()
            .insert((game, aid.0, unit_cost));
        if dc == "ranged" && ac == "melee" && !ranged {
            stats.melee_ranged_contacts += 1;
            stats.melee_reached.insert((game, aid.0));
            if *attacker_hp_after <= 0 {
                stats.melee_killed_at_contact += 1;
                stats.melee_killed_at_contact_units.insert((game, aid.0));
            } else if *damage_to_defender > 0 {
                stats.melee_hit_survived_contact += 1;
                stats.melee_survived_contact_units.insert((game, aid.0));
            }
        }
        if *ranged
            && ac == "ranged"
            && dc == "melee"
            && *defender_hp_after <= 0
            && let Some(victim) = victim
            && !stats.melee_reached.contains(&(game, victim))
        {
            stats.melee_killed_before_reach.insert((game, victim));
        }
        if *defender_hp_after <= 0
            && let Some(victim) = victim
        {
            by_victim.entry((turn, victim)).or_default().push((
                ac.clone(),
                dc,
                *damage_to_defender,
            ));
        }
    }
    for (_, attackers) in by_victim {
        let total: i32 = attackers.iter().map(|x| x.2.max(0)).sum();
        if total == 0 {
            continue;
        }
        for (ac, dc, d) in attackers {
            let credited = ((d.max(0) as u64 * 1000) / total as u64).min(1000);
            *stats.kills.entry(ac.clone()).or_default() += credited;
            stats
                .pairs
                .entry((ac, dc))
                .or_default()
                .credited_kills_milli += credited;
        }
    }
}

fn campaign(seed: u64, games: usize, players: usize) -> Value {
    let mut stats = CampaignStats::default();
    let mut finished = 0;
    for i in 0..games {
        let mut setup = GameSetup::all_ai(seed + i as u64);
        setup.players.truncate(players);
        setup.turn_limit = None;
        for (seat, p) in setup.players.iter_mut().enumerate() {
            p.team = TeamId(seat as u8);
            p.difficulty = AiDifficulty::Hard;
            p.search = true;
        }
        if players == 4 {
            (setup.width, setup.height) = (24, 16);
        }
        let mut state = new_game(&setup);
        let mut brains: Vec<Box<dyn Brain>> = (0..players)
            .map(|s| {
                let id = PlayerId(s as u8);
                let base = ScriptedBrain::with_difficulty(
                    state.player(id).faction,
                    brain_seed(setup.seed, id),
                    id,
                    AiDifficulty::Hard,
                );
                Box::new(SearchBrain::new(base, SearchConfig::in_app(players))) as Box<dyn Brain>
            })
            .collect();
        let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
        while !state.finished && state.turn < cap {
            let orders: OrderMap = brains
                .iter_mut()
                .enumerate()
                .filter(|(s, _)| state.player(PlayerId(*s as u8)).alive)
                .map(|(s, b)| {
                    (
                        PlayerId(s as u8),
                        b.orders(&legatum_core::player_view(&state, PlayerId(s as u8))),
                    )
                })
                .collect();
            let t = state.turn;
            let attacker_kinds = state
                .units
                .iter()
                .map(|(id, u)| {
                    let faction = state.player(u.owner).faction;
                    (id.0, (u.kind, data::unit_stats_for(faction, u.kind).cost))
                })
                .collect();
            let (next, events) = resolve_turn(&state, &orders);
            observe(&events, t, seed + i as u64, &attacker_kinds, &mut stats);
            state = next;
        }
        finished += u32::from(state.finished);
    }
    let pairs: Vec<_> = stats
        .pairs
        .iter()
        .map(|((a, b), v)| {
            json!({"attacker_class":a,
        "defender_class":b,"battle_events":v.battles,"damage_dealt":v.damage,
        "credited_kills":v.credited_kills_milli as f64/1000.0})
        })
        .collect();
    let efficiency: Vec<_> = stats
        .damage
        .iter()
        .map(|(c, d)| {
            let costs: usize = stats
                .spent
                .get(c)
                .map_or(0, |x| x.iter().map(|row| row.2 as usize).sum());
            let units = stats.spent.get(c).map_or(0, |x| x.len());
            json!({"class":c,"damage":d,"unique_attackers":units,
            "production_spent":costs,
            "damage_per_production":*d as f64/costs.max(1) as f64})
        })
        .collect();
    json!({"games":games,"players":players,"ai":"Hard + Search",
        "finished_games":finished,"roster":roster(),"class_pairs":pairs,
        "kills_milli":stats.kills,"losses":stats.losses,"efficiency_proxy":efficiency,
        "melee_vs_ranged_contacts":stats.melee_ranged_contacts,
        "melee_killed_at_contact":stats.melee_killed_at_contact,
        "melee_hit_and_survived_contact":stats.melee_hit_survived_contact,
        "melee_units_reached_and_hit_ranged":stats.melee_reached.len(),
        "melee_units_killed_by_ranged_before_contact":stats.melee_killed_before_reach.len(),
        "melee_units_killed_at_contact":stats.melee_killed_at_contact_units.len(),
        "melee_units_hit_and_survived_contact":stats.melee_survived_contact_units.len(),
        "notes":"Kill shares are damage weighted within turn/victim. Production efficiency counts each distinct attacker unit's base Production cost once per game; it is engaged-force cost, not all units built."})
}

fn fresh_unit(template: &Unit, id: u32, owner: u8, kind: UnitKind, pos: Hex) -> Unit {
    let mut u = template.clone();
    u.id = legatum_core::UnitId(id);
    u.owner = PlayerId(owner);
    u.kind = kind;
    u.pos = pos;
    u.hp = UNIT_HP;
    u.moves_left = data::unit_stats(kind).moves;
    u.goto = None;
    u.attack_target = None;
    u.path.clear();
    u.fortify = 0;
    u.acted = false;
    u.xp = 0;
    u
}

fn force_counts(a: UnitKind, b: UnitKind) -> (usize, usize) {
    let ca = data::unit_stats(a).cost;
    let cb = data::unit_stats(b).cost;
    let gcd = |mut x: i32, mut y: i32| {
        while y != 0 {
            let r = x % y;
            x = y;
            y = r;
        }
        x
    };
    let budget = ca / gcd(ca, cb) * cb;
    ((budget / ca) as usize, (budget / cb) as usize)
}

fn skirmish(seed: u64, a: UnitKind, b: UnitKind, terrain: &str) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(2);
    setup.width = 32;
    setup.height = 18;
    setup.turn_limit = None;
    setup.legacy_all_war = true;
    setup.barbarians = false;
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    let mut st = new_game(&setup);
    let template = st.units.values().next().unwrap().clone();
    st.units.clear();
    st.next_unit_id = 1;
    for p in &mut st.players {
        for i in 0..st.map.len() {
            p.explored.set(i);
        }
    }
    for h in st.map.hexes().collect::<Vec<_>>() {
        st.map.get_mut(h).unwrap().terrain = if terrain == "plains" {
            Terrain::Plains
        } else {
            Terrain::Grassland
        };
    }
    if terrain == "barrier" {
        for row in 3..15 {
            let f = Hex::from_offset(16, row);
            let hill = Hex::from_offset(16, row + 1);
            if let Some(t) = st.map.get_mut(f) {
                t.terrain = Terrain::Forest;
            }
            if let Some(t) = st.map.get_mut(hill) {
                t.terrain = Terrain::Hills;
            }
        }
    }
    let (na, nb) = force_counts(a, b);
    let swap = seed % 2 == 1;
    let side_a = usize::from(swap);
    let mut id = 1;
    for (side, kind, n, col) in [
        (side_a, a, na, if swap { 22 } else { 10 }),
        (1 - side_a, b, nb, if swap { 10 } else { 22 }),
    ] {
        for j in 0..n {
            let pos = Hex::from_offset(col, 2 + j as i32);
            st.units.insert(
                legatum_core::UnitId(id),
                fresh_unit(&template, id, side as u8, kind, pos),
            );
            id += 1;
        }
    }
    // The game eliminates a cityless player with no Settler. Keep one harmless Settler
    // behind each force so the normal turn resolver does not end the skirmish after one turn.
    for (owner, pos) in [(0, Hex::from_offset(1, 1)), (1, Hex::from_offset(30, 16))] {
        st.units.insert(
            legatum_core::UnitId(id),
            fresh_unit(&template, id, owner, UnitKind::Settler, pos),
        );
        id += 1;
    }
    let mut killed = [0, 0];
    let mut dmg = [0u64, 0u64];
    let mut turn = 0;
    while turn < 80
        && st
            .units
            .values()
            .any(|u| u.owner == PlayerId(0) && COMBAT_UNITS.contains(&u.kind))
        && st
            .units
            .values()
            .any(|u| u.owner == PlayerId(1) && COMBAT_UNITS.contains(&u.kind))
    {
        let mut orders = OrderMap::new();
        for side in 0..2u8 {
            let mut os = Vec::new();
            let own: Vec<_> = st
                .units
                .values()
                .filter(|u| u.owner == PlayerId(side))
                .filter(|u| COMBAT_UNITS.contains(&u.kind))
                .collect();
            for u in own {
                let enemy = st
                    .units
                    .values()
                    .filter(|e| e.owner != u.owner && COMBAT_UNITS.contains(&e.kind))
                    .min_by_key(|e| (u.pos.distance(e.pos), e.id))
                    .unwrap();
                if data::unit_stats(u.kind).ranged > 0
                    && u.pos.distance(enemy.pos) <= data::unit_stats(u.kind).range
                {
                    os.push(Order::RangedAttack {
                        unit: u.id,
                        target: enemy.pos,
                    });
                } else {
                    os.push(Order::MoveUnit {
                        unit: u.id,
                        to: enemy.pos,
                    });
                }
            }
            orders.insert(PlayerId(side), os);
        }
        let (next, events) = resolve_turn(&st, &orders);
        for e in events {
            if let Event::UnitDestroyed { owner, .. } = e {
                killed[owner.0 as usize] += 1;
            }
            if let Event::Battle {
                attacker_owner,
                damage_to_defender,
                damage_to_attacker,
                ..
            } = e
            {
                dmg[attacker_owner.0 as usize] += damage_to_defender as u64;
                dmg[1 - attacker_owner.0 as usize] += damage_to_attacker as u64;
            }
        }
        st = next;
        turn += 1;
    }
    let hp = [0, 1].map(|s| {
        st.units
            .values()
            .filter(|u| u.owner == PlayerId(s) && COMBAT_UNITS.contains(&u.kind))
            .map(|u| u.hp)
            .sum::<i32>()
    });
    let remaining_a = hp[side_a];
    let remaining_b = hp[1 - side_a];
    let losses_a = killed[side_a];
    let losses_b = killed[1 - side_a];
    let damage_a = dmg[side_a];
    let damage_b = dmg[1 - side_a];
    json!({"seed":seed,"a":format!("{a:?}"),"b":format!("{b:?}"),"terrain":terrain,
        "a_count":na,"b_count":nb,"production_each":data::unit_stats(a).cost as usize*na,
        "winner":if remaining_a>remaining_b{"a"}else if remaining_b>remaining_a{"b"}else{"tie"},
        "remaining_hp_a":remaining_a,"remaining_hp_b":remaining_b,"losses_a":losses_a,"losses_b":losses_b,
        "damage_dealt_a":damage_a,"damage_dealt_b":damage_b,"turns":turn})
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let value = |key: &str, default: &str| -> String {
        args.windows(2)
            .find(|w| w[0] == key)
            .map(|w| w[1].clone())
            .unwrap_or(default.into())
    };
    let mode = value("--mode", "skirmish");
    let out = PathBuf::from(value("--out-dir", "."));
    let seed: u64 = value("--seed", "8100001").parse().unwrap();
    let games: usize = value("--games", "256").parse().unwrap();
    let label = value("--label", "study");
    let _players: usize = value("--players", "8").parse().unwrap();
    let replicates: usize = value("--replicates", "200").parse().unwrap();
    fs::create_dir_all(&out).unwrap();
    let data = if mode == "campaign" {
        campaign(seed, games, _players)
    } else {
        let mut rows = Vec::new();
        for terrain in ["open", "plains", "barrier"] {
            for (i, a) in COMBAT_UNITS.into_iter().enumerate() {
                for b in COMBAT_UNITS.into_iter().skip(i) {
                    for n in 0..replicates {
                        rows.push(skirmish(seed + n as u64, a, b, terrain));
                    }
                }
            }
        }
        json!({"seeds_per_pair_terrain":replicates,"pairs":"unordered, self-pairs included",
            "terrain_scenarios":["open=Grassland","plains=Plains","barrier=alternating Forest and Hills between forces"],
            "roster":roster(),"results":rows})
    };
    let path = out.join(format!("unit-matchups-{label}-{mode}.json"));
    fs::write(&path, serde_json::to_vec_pretty(&data).unwrap()).unwrap();
    println!(
        "{}",
        json!({"output":path,"mode":mode,"rows":data["results"].as_array().map_or(1,Vec::len)})
    );
}
