//! Trace one empire through a barbarian game (the runner's ordinary setup: all-AI, shuffled
//! factions, barbarians on). barbarian_trace <seed> <player> [turns=300]
//! BARBARIANS_OFF=1 plays the same game without barbarians. One line per 5 turns: the empire's cities and each city's build, its settlers and army, the
//! barbarian units within 4 of its cities, and its known camps with their distances.
use legatum_core::rng::{Rng, purpose};
use legatum_core::{Faction, GameSetup, PlayerId, UnitKind, player_view};
use legatum_wasm::session::{BrainKind, Session};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: u64 = a[1].parse().unwrap();
    let me = PlayerId(a[2].parse().unwrap());
    let turns: u32 = a.get(3).map_or(300, |x| x.parse().unwrap());
    let mut setup = GameSetup::all_ai(seed);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (p, f) in setup.players.iter_mut().zip(factions) {
        p.faction = f;
    }
    setup.barbarians = std::env::var_os("BARBARIANS_OFF").is_none();
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    while !session.state.finished && session.state.turn < turns {
        let s = &session.state;
        let every: u32 = std::env::var("EVERY")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(5);
        if s.turn.is_multiple_of(every) {
            let cities: Vec<String> = s
                .cities_of(me)
                .map(|c| {
                    let near = s
                        .units
                        .values()
                        .filter(|u| u.owner.is_barbarian() && u.pos.distance(c.pos) <= 4)
                        .map(|u| format!("{:?}@{}", u.kind, u.pos.distance(c.pos)))
                        .collect::<Vec<_>>();
                    let y = legatum_core::economy::city_yields(s, c).total;
                    format!(
                        "{} p{} {:?} stored {} +{}p +{}f tiles-owned {} barb[{}]",
                        c.name,
                        c.population,
                        c.producing,
                        c.production_stored,
                        y.production,
                        y.food,
                        s.map.tiles.iter().filter(|t| t.city == Some(c.id)).count(),
                        near.join(",")
                    )
                })
                .collect();
            let settler_pos: Vec<String> = s
                .units_of(me)
                .filter(|u| u.kind == UnitKind::Settler)
                .map(|u| {
                    let d = s.cities_of(me).map(|c| c.pos.distance(u.pos)).min();
                    format!("{:?}@{:?}", u.goto.map(|g| g.distance(u.pos)), d)
                })
                .collect();
            let settlers = settler_pos.join(",");
            let army = s
                .units_of(me)
                .filter(|u| u.kind != UnitKind::Settler)
                .count();
            let v = player_view(s, me);
            let camps: Vec<String> = v
                .barbarian_camps
                .iter()
                .map(|c| {
                    let d = s.cities_of(me).map(|x| x.pos.distance(c.pos)).min();
                    format!("{}{:?}", if c.visible { "" } else { "~" }, d)
                })
                .collect();
            println!(
                "t{} gold {} settlers {settlers} army {army} camps [{}] | {}",
                s.turn,
                s.player(me).gold,
                camps.join(" "),
                cities.join(" | ")
            );
        }
        let orders = session.ai_orders();
        for e in session.step(&orders) {
            use legatum_core::Event;
            match &e {
                Event::SettlerCaptured { from, to, .. } if *from == me || *to == me => {
                    println!("  t{} {e:?}", session.state.turn - 1)
                }
                Event::OrderFailed { player, reason }
                    if *player == me && reason.contains("ettler") =>
                {
                    println!("  t{} {reason}", session.state.turn - 1)
                }
                Event::UnitDestroyed {
                    owner, unit_kind, ..
                } if *owner == me => {
                    println!("  t{} lost {unit_kind:?}", session.state.turn - 1)
                }
                _ => {}
            }
        }
    }
}
