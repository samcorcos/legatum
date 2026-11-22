//! A random-but-plausible baseline brain, used as the Elo floor in simulations.

use crate::Brain;
use legatum_core::rng::{Rng, purpose};
use legatum_core::view::PlayerView;
use legatum_core::{Order, PlayerId, UnitKind};

/// Stream tag separating RandomBrain draws from ScriptedBrain jitter.
const RANDOM_STREAM: u64 = 0x5241_4e44;

/// Picks uniformly among plausible orders: random nearby moves, random production and research.
/// Only ever orders its own units and cities, and only targets explored passable tiles.
pub struct RandomBrain {
    pub player: PlayerId,
    pub seed: u64,
}

impl RandomBrain {
    pub fn new(seed: u64, player: PlayerId) -> Self {
        Self { player, seed }
    }
}

/// Uniform index in `0..n` (n > 0), reduced in u64 so wasm32 and native agree.
fn pick(rng: &mut Rng, n: usize) -> usize {
    (rng.next_u64() % n as u64) as usize
}

impl Brain for RandomBrain {
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        let mut orders = Vec::new();
        if view.finished {
            return orders;
        }
        let me = view.me;
        let mut rng = Rng::derive(
            self.seed,
            &[view.turn as u64, purpose::AI, me.0 as u64, RANDOM_STREAM],
        );

        if view.research.current.is_none() && !view.research.available.is_empty() {
            let i = pick(&mut rng, view.research.available.len());
            orders.push(Order::SetResearch {
                tech: view.research.available[i].tech,
            });
        }

        let mut cities: Vec<_> = view
            .cities
            .iter()
            .filter(|c| c.owner == me && c.detail.is_some())
            .collect();
        cities.sort_by_key(|c| c.id);
        for c in cities {
            let d = c.detail.as_ref().expect("filtered");
            // Only items that finish: a never-ending project would park the city for good.
            let finite: Vec<_> = d
                .options
                .iter()
                .filter(|o| !matches!(o.item, legatum_core::ProductionItem::Project { .. }))
                .collect();
            if d.producing.is_none() && !finite.is_empty() {
                let i = pick(&mut rng, finite.len());
                orders.push(Order::SetProduction {
                    city: c.id,
                    item: finite[i].item,
                });
            }
        }

        let mut units: Vec<_> = view.units.iter().filter(|u| u.owner == me).collect();
        units.sort_by_key(|u| u.id);
        for u in units {
            // Settlers found where they stand now and then (if it looks legal).
            if u.kind == UnitKind::Settler {
                let clear = !view.failed_city_sites.contains(&u.pos)
                    && view.cities.iter().all(|c| c.pos.distance(u.pos) > 3);
                let land = view
                    .tile(u.pos)
                    .and_then(|t| t.terrain)
                    .is_some_and(|t| t.is_land());
                let unowned = view
                    .tile(u.pos)
                    .is_some_and(|t| t.owner.is_none_or(|o| o == me));
                if clear && land && unowned && rng.range(0, 3) == 0 {
                    orders.push(Order::FoundCity { unit: u.id });
                    continue;
                }
            }
            let targets: Vec<_> = u
                .pos
                .within(3)
                .into_iter()
                .filter(|h| *h != u.pos)
                .filter(|h| {
                    view.tile(*h).is_some_and(|t| {
                        t.explored
                            && t.terrain.and_then(|t| t.move_cost()).is_some()
                            && view.movement_path(me, u.kind, u.pos, *h).is_some()
                    })
                })
                .collect();
            if targets.is_empty() || rng.range(0, 4) == 0 {
                orders.push(Order::SkipUnit { unit: u.id });
            } else {
                let to = targets[pick(&mut rng, targets.len())];
                orders.push(Order::MoveUnit { unit: u.id, to });
            }
        }
        orders
    }
}
