//! threat_trace <tape.json> <from> <to> <threat-player>
//! Replays a tape with its recorded orders. For each AI seat and each turn in [from, to],
//! prints what that AI's PlayerView shows about the threat player and what the brain orders.
use legatum_core::diplomacy::RelationState;
use legatum_core::{Order, PlayerId, ProductionItem, player_view, resolve_turn};
use legatum_wasm::session::{BrainKind, make_brain};
use legatum_wasm::tape::Tape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let from: u32 = a[2].parse()?;
    let to: u32 = a[3].parse()?;
    let threat = PlayerId(a[4].parse()?);
    let mut state = tape.initial_state();
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| make_brain(BrainKind::Scripted, &state, p.id))
        .collect();
    for turn in &tape.turns {
        let t = turn.turn;
        for p in state.players.clone() {
            if p.is_human || !p.alive {
                continue;
            }
            let v = player_view(&state, p.id);
            let orders = brains[p.id.0 as usize].orders(&v);
            if t < from || t > to {
                continue;
            }
            let memory = brains[p.id.0 as usize].opponent_memory();
            let threat_line = memory.as_ref().map_or("-".to_string(), |m| {
                let top = m.assessments.threats.top().map(|x| (x.rival.0, x.score));
                let me = m
                    .assessments
                    .threats
                    .get(threat)
                    .map(|x| format!("score {} [{}]", x.score, x.reasons().join(", ")));
                format!(
                    "top {top:?} threat-model P{}: {}",
                    threat.0,
                    me.unwrap_or("-".into())
                )
            });
            let tt = v.team_of(threat);
            let pub_t = v.players.iter().find(|x| x.id == threat).unwrap();
            let rel = v
                .diplomacy
                .relations
                .iter()
                .find(|r| r.between(v.team, tt))
                .map(|r| format!("{:?}", r.state))
                .unwrap_or("-".into());
            let threat_wars: Vec<_> = v
                .diplomacy
                .relations
                .iter()
                .filter(|r| r.state == RelationState::War && (r.a == tt || r.b == tt))
                .map(|r| {
                    format!(
                        "{}v{}(decl {:?} eff {})",
                        r.a.0, r.b.0, r.declarers, r.effective_turn
                    )
                })
                .collect();
            let own_cities: Vec<_> = v.cities.iter().filter(|c| c.owner == v.me).collect();
            let walls = own_cities.iter().filter(|c| c.walls).count();
            let seen: Vec<_> = v
                .units
                .iter()
                .filter(|u| u.owner == threat && u.military)
                .collect();
            let seen_dist = seen
                .iter()
                .flat_map(|u| own_cities.iter().map(move |c| c.pos.distance(u.pos)))
                .min();
            let known_threat_cities: Vec<_> =
                v.cities.iter().filter(|c| c.owner == threat).collect();
            let city_dist = known_threat_cities
                .iter()
                .flat_map(|tc| own_cities.iter().map(move |c| c.pos.distance(tc.pos)))
                .min();
            let army = |team| {
                v.players
                    .iter()
                    .filter(|p| p.team == team)
                    .map(|p| p.army)
                    .sum::<i32>()
            };
            let dead = v.players.iter().filter(|x| !x.alive).count();
            let coal = v.diplomacy.coalition.as_ref().map(|c| c.target.0);
            let mut dip = Vec::new();
            let mut prod = Vec::new();
            let mut research = None;
            for o in &orders {
                match o {
                    Order::DeclareWar { .. }
                    | Order::ProposeTreaty { .. }
                    | Order::RespondToProposal { .. }
                    | Order::WithdrawProposal { .. }
                    | Order::BreakTreaty { .. } => dip.push(format!("{o:?}")),
                    Order::SetProduction { item, .. } => prod.push(match item {
                        ProductionItem::Unit { unit } => format!("{unit:?}"),
                        ProductionItem::Building { building } => format!("{building:?}"),
                        other => format!("{other:?}"),
                    }),
                    Order::BuyItem { item, .. } => prod.push(format!("BUY {item:?}")),
                    Order::SetResearch { tech } => research = Some(*tech),
                    _ => {}
                }
            }
            let rels: Vec<_> = v
                .diplomacy
                .relations
                .iter()
                .filter(|r| (r.a == v.team || r.b == v.team) && r.state != RelationState::Peace)
                .map(|r| format!("{}-{}:{:?}", r.a.0, r.b.0, r.state))
                .collect();
            let proposals: Vec<_> = v
                .diplomacy
                .proposals
                .iter()
                .filter(|pp| pp.to == v.team || pp.from == v.team)
                .map(|pp| format!("{}->{} {:?}", pp.from.0, pp.to.0, pp.kind))
                .collect();
            println!(
                "T{t} P{} {:?}: {threat_line} || cities {} walls {} mil {} army {} | threat P{}: rel {rel} vp {}/{} cities {} army {} wars {:?} | seen mil {} (str {}) nearest {:?} city-dist {:?} | dead {dead} coalition {:?} | rels {:?} props {:?} | research {:?} prod {:?} dip {:?}",
                p.id.0,
                p.faction,
                own_cities.len(),
                walls,
                v.units
                    .iter()
                    .filter(|u| u.owner == v.me && u.military)
                    .count(),
                army(v.team),
                threat.0,
                pub_t.vp,
                v.standings
                    .iter()
                    .find(|s| s.team == tt)
                    .map_or(0, |s| s.target),
                pub_t.cities,
                army(tt),
                threat_wars,
                seen.len(),
                seen.iter().map(|u| u.strength.max(u.ranged)).sum::<i32>(),
                seen_dist,
                city_dist,
                coal,
                rels,
                proposals,
                research,
                prod,
                dip
            );
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        if t >= from.saturating_sub(20) && t <= to {
            for e in &events {
                let s = format!("{e:?}");
                if s.starts_with("CityCaptured")
                    || s.starts_with("PlayerEliminated")
                    || s.starts_with("WarDeclared")
                    || s.starts_with("TreatyActivated")
                    || s.starts_with("PeaceEstablished")
                {
                    println!("  EVENT T{t}: {}", &s[..s.len().min(220)]);
                }
            }
        }
        state = next;
    }
    Ok(())
}
