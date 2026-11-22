//! Trace one coalition_bench game (same setup: solo teams, rotated factions, Standard).
//! coalition_trace <map-seed> <rotation> [every=10] [cap=1200] [save-at-turn save-path]
//! One line per sampled turn: coalition target, declarations, peace, captures, each team's VP
//! sources, cities, gold and units, non-Peace relations. The optional save
//! writes that turn's GameState JSON (e.g. to load as a browser save with a human seat).
use legatum_core::diplomacy::RelationState;
use legatum_core::{Event, Faction, GameSetup, TeamId, scoring};
use legatum_wasm::session::{BrainKind, Session};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: u64 = a[1].parse().unwrap();
    let rotation: usize = a[2].parse().unwrap();
    let every: u32 = a.get(3).map_or(10, |x| x.parse().unwrap());
    let cap: u32 = a.get(4).map_or(1200, |x| x.parse().unwrap());
    let save_at: Option<u32> = a.get(5).map(|x| x.parse().unwrap());
    let mut setup = GameSetup::all_ai(seed);
    setup.turn_limit = None;
    for (seat, p) in setup.players.iter_mut().enumerate() {
        p.faction = Faction::ALL[(seat + rotation) % 8];
        p.team = TeamId(seat as u8);
    }
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut decl = 0;
    let mut caps = 0;
    let mut peace = 0;
    while !session.state.finished && session.state.turn < cap {
        let s = &session.state;
        if save_at == Some(s.turn) {
            std::fs::write(&a[6], serde_json::to_string(s).unwrap()).unwrap();
        }
        if s.turn.is_multiple_of(every) {
            let d = s.diplomacy.as_ref().unwrap();
            let mut line = format!(
                "t{} coal={:?} decl={decl} peace={peace} caps={caps} |",
                s.turn,
                legatum_core::diplomacy::coalition::target_state(s).map(|t| t.0)
            );
            for t in s.teams() {
                let p = s.team_members(t)[0];
                let pl = s.player(p);
                let src: Vec<String> = scoring::vp(s, p)
                    .iter()
                    .map(|(n, v)| {
                        let n = format!("{n:?}");
                        format!("{}{}", &n[..3.min(n.len())], v)
                    })
                    .collect();
                line += &format!(
                    " T{}{} {}vp c{} g{} u{} [{}] |",
                    t.0,
                    if pl.alive { "" } else { "x" },
                    scoring::team_vp(s, t),
                    s.cities_of(p).count(),
                    pl.gold,
                    s.units_of(p).count(),
                    src.join(",")
                );
            }
            let rel: Vec<String> = d
                .relations
                .iter()
                .filter(|r| r.state != RelationState::Peace)
                .map(|r| {
                    format!(
                        "{}{}{}",
                        r.a.0,
                        match r.state {
                            RelationState::War => "W",
                            RelationState::Alliance => "A",
                            _ => ".",
                        },
                        r.b.0
                    )
                })
                .collect();
            line += &format!(" rel {}", rel.join(" "));
            println!("{line}");
        }
        let orders = session.ai_orders();
        for e in session.step(&orders) {
            match e {
                Event::WarDeclared { .. } => decl += 1,
                Event::CityCaptured { .. } => caps += 1,
                Event::PeaceEstablished { .. } => peace += 1,
                _ => {}
            }
        }
    }
    println!(
        "end turn {} finished={} winner={:?}",
        session.state.turn, session.state.finished, session.state.winner
    );
}
