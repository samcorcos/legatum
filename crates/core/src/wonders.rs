//! Rare, seeded landmarks. A wonder belongs to the city whose territory covers its tile.
use crate::events::Event;
use crate::hex::Hex;
use crate::rng::{Rng, purpose};
use crate::state::{GameSetup, GameState, Map};
use crate::types::{Terrain, Yields};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Wonder {
    MountFuji,
    VictoriaFalls,
    GrandCanyon,
    GiantsCauseway,
    Uluru,
    HaLongBay,
    IguazuFalls,
    GreatBarrierReef,
}

impl Wonder {
    pub const ALL: [Self; 8] = [
        Self::MountFuji,
        Self::VictoriaFalls,
        Self::GrandCanyon,
        Self::GiantsCauseway,
        Self::Uluru,
        Self::HaLongBay,
        Self::IguazuFalls,
        Self::GreatBarrierReef,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::MountFuji => "Mount Fuji",
            Self::VictoriaFalls => "Victoria Falls",
            Self::GrandCanyon => "Grand Canyon",
            Self::GiantsCauseway => "Giant's Causeway",
            Self::Uluru => "Uluru",
            Self::HaLongBay => "Ha Long Bay",
            Self::IguazuFalls => "Iguazu Falls",
            Self::GreatBarrierReef => "Great Barrier Reef",
        }
    }

    pub fn bonus(self, base: Yields) -> Yields {
        let quarter = |n: i32| if n > 0 { (n + 3) / 4 } else { 0 };
        match self {
            Self::MountFuji | Self::GrandCanyon | Self::GreatBarrierReef => {
                Yields::new(0, 0, 0, quarter(base.science))
            }
            Self::GiantsCauseway => Yields::new(0, quarter(base.production), 0, 0),
            Self::VictoriaFalls | Self::HaLongBay | Self::IguazuFalls => {
                Yields::new(quarter(base.food), 0, 0, 0)
            }
            Self::Uluru => Yields::new(0, 0, quarter(base.gold), 0),
        }
    }

    /// The one-time discovery reward: (Gold, Science).
    pub fn discovery(self) -> (i32, i32) {
        if matches!(
            self,
            Self::MountFuji | Self::GrandCanyon | Self::HaLongBay | Self::GreatBarrierReef
        ) {
            (0, 50)
        } else {
            (80, 0)
        }
    }

    fn suitable(self, terrain: Terrain) -> bool {
        match self {
            Self::MountFuji => terrain == Terrain::Mountains,
            Self::VictoriaFalls | Self::GiantsCauseway | Self::HaLongBay => {
                terrain == Terrain::Hills
            }
            Self::GrandCanyon | Self::Uluru => terrain == Terrain::Desert,
            Self::IguazuFalls => terrain == Terrain::Forest,
            Self::GreatBarrierReef => terrain == Terrain::Coast,
        }
    }
}

pub fn target(setup: &GameSetup) -> usize {
    let base = 6;
    ((base * (setup.width * setup.height) + 768) / 1536).clamp(1, 8) as usize
}

/// A derived stream keeps terrain, starts, ruins and resources stable when wonder art/rules change.
pub fn place(map: &mut Map, starts: &[Hex], setup: &GameSetup, attempt: u64) {
    let mut rng = Rng::derive(setup.seed, &[purpose::MAPGEN, attempt, 21]);
    let mut kinds = Wonder::ALL;
    for i in (1..kinds.len()).rev() {
        kinds.swap(i, rng.range(0, i as i64) as usize);
    }
    let mut placed = Vec::<Hex>::new();
    for kind in kinds {
        if placed.len() == target(setup) {
            break;
        }
        let mut candidates: Vec<_> = (0..map.len())
            .filter(|&i| {
                let tile = &map.tiles[i];
                let h = map.hex_at(i);
                kind.suitable(tile.terrain)
                    && !tile.ruin
                    && tile.resource.is_none()
                    && starts.iter().all(|s| s.distance(h) >= 4)
                    && placed.iter().all(|p| p.distance(h) >= 5)
            })
            .map(|i| (rng.next_u64(), i))
            .collect();
        candidates.sort_unstable();
        if let Some((_, i)) = candidates.first() {
            placed.push(map.hex_at(*i));
            map.tiles[*i].wonder = Some(kind);
        }
    }
}

/// The first empire with native unit/city sight receives the reward once globally.
pub fn discover(state: &mut GameState, events: &mut Vec<Event>) {
    for i in 0..state.map.len() {
        let Some(kind) = state.map.tiles[i].wonder else {
            continue;
        };
        let at = state.map.hex_at(i);
        if state.wonders_discovered.contains(&at) {
            continue;
        }
        let owner = state
            .players
            .iter()
            .find(|player| {
                let sees = |from: Hex, radius: i32| {
                    let hills = state
                        .map
                        .get(from)
                        .is_some_and(|t| t.terrain == Terrain::Hills);
                    from.distance(at) <= radius + i32::from(hills)
                        && crate::los::visible(from, at, |h| state.map.get(h).map(|t| t.terrain))
                };
                state
                    .units_of(player.id)
                    .any(|unit| sees(unit.pos, crate::data::UNIT_SIGHT))
                    || state
                        .cities_of(player.id)
                        .any(|city| sees(city.pos, crate::data::CITY_SIGHT))
            })
            .map(|p| p.id);
        let Some(owner) = owner else { continue };
        let (gold, science) = kind.discovery();
        let player = state.player_mut(owner);
        player.gold = crate::validation::credit_gold(player.gold, gold);
        player.science_stored = crate::validation::add_stored(player.science_stored, science);
        state.wonders_discovered.insert(at);
        events.push(Event::WonderDiscovered {
            owner,
            at,
            wonder: kind,
            gold,
            science,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_support::{city, game as fixture, hex, unit};
    use crate::{Faction, PlayerId, UnitKind};

    #[test]
    fn real_wonders_keep_flavour_matched_existing_yields() {
        let base = Yields::new(8, 12, 16, 20);
        let expected = [
            (Wonder::MountFuji, "Mount Fuji", Yields::new(0, 0, 0, 5)),
            (
                Wonder::VictoriaFalls,
                "Victoria Falls",
                Yields::new(2, 0, 0, 0),
            ),
            (Wonder::GrandCanyon, "Grand Canyon", Yields::new(0, 0, 0, 5)),
            (
                Wonder::GiantsCauseway,
                "Giant's Causeway",
                Yields::new(0, 3, 0, 0),
            ),
            (Wonder::Uluru, "Uluru", Yields::new(0, 0, 4, 0)),
            (Wonder::HaLongBay, "Ha Long Bay", Yields::new(2, 0, 0, 0)),
            (Wonder::IguazuFalls, "Iguazu Falls", Yields::new(2, 0, 0, 0)),
            (
                Wonder::GreatBarrierReef,
                "Great Barrier Reef",
                Yields::new(0, 0, 0, 5),
            ),
        ];
        for (wonder, name, bonus) in expected {
            assert_eq!(wonder.name(), name);
            assert_eq!(wonder.bonus(base), bonus);
        }
    }

    #[test]
    fn seeded_placement_has_count_terrain_and_spacing() {
        let setup = GameSetup::standard(12345, [Faction::India, Faction::Switzerland]);
        let (map, starts) = crate::mapgen::generate(&setup);
        let (again, _) = crate::mapgen::generate(&setup);
        assert_eq!(map.tiles, again.tiles);
        let placed: Vec<_> = map
            .hexes()
            .filter(|&h| map.get(h).unwrap().wonder.is_some())
            .collect();
        assert_eq!(placed.len(), target(&setup));
        for &h in &placed {
            let tile = map.get(h).unwrap();
            assert!(tile.wonder.unwrap().suitable(tile.terrain));
            assert!(!tile.ruin && tile.resource.is_none());
            assert!(starts.iter().all(|s| s.distance(h) >= 4));
            assert!(
                placed
                    .iter()
                    .filter(|&&other| other != h)
                    .all(|other| other.distance(h) >= 5)
            );
        }
    }

    #[test]
    fn starting_sight_remains_eligible_for_a_discovery_notice() {
        for seed in 0..12 {
            let setup = GameSetup::standard(seed, [Faction::India, Faction::Mongols]);
            let game = crate::game::new_game(&setup);
            assert!(game.wonders_discovered.is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn territory_city_gets_scaling_bonus_and_loses_it_on_transfer() {
        let mut s = fixture(&[0, 1]);
        let a = city(&mut s, 0, hex(4, 4), 3);
        let b = city(&mut s, 1, hex(8, 4), 3);
        let h = hex(5, 4);
        let before = crate::economy::city_yields(&s, &s.cities[&a]).total;
        s.map.get_mut(h).unwrap().wonder = Some(Wonder::MountFuji);
        let with = crate::economy::city_yields(&s, &s.cities[&a]).total;
        assert_eq!(with.science - before.science, (before.science + 3) / 4);
        s.map.get_mut(h).unwrap().city = Some(b);
        s.map.get_mut(h).unwrap().owner = Some(PlayerId(1));
        assert_eq!(
            crate::economy::city_yields(&s, &s.cities[&a]).total.science,
            before.science
        );
        assert!(
            crate::economy::city_yields(&s, &s.cities[&b])
                .sources
                .iter()
                .any(|source| source.label.contains("Mount Fuji"))
        );
    }

    #[test]
    fn first_sight_pays_once_even_when_rival_sees_later() {
        let mut s = fixture(&[0, 1]);
        let h = hex(5, 5);
        s.map.get_mut(h).unwrap().wonder = Some(Wonder::VictoriaFalls);
        unit(&mut s, 0, UnitKind::Scout, hex(4, 5));
        let mut events = Vec::new();
        discover(&mut s, &mut events);
        assert!(matches!(
            events.as_slice(),
            [Event::WonderDiscovered {
                owner: PlayerId(0),
                gold: 80,
                science: 0,
                ..
            }]
        ));
        assert_eq!(s.player(PlayerId(0)).gold, 80);
        unit(&mut s, 1, UnitKind::Scout, hex(6, 5));
        discover(&mut s, &mut events);
        assert_eq!(events.len(), 1);
        assert_eq!(s.player(PlayerId(1)).gold, 0);
        let serialized = serde_json::to_string(&s).unwrap();
        let mut reloaded: GameState = serde_json::from_str(&serialized).unwrap();
        discover(&mut reloaded, &mut events);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn a_passable_wonder_cannot_hold_a_city() {
        let mut s = fixture(&[0, 1]);
        let h = hex(5, 5);
        s.map.get_mut(h).unwrap().wonder = Some(Wonder::VictoriaFalls);
        let settler = unit(&mut s, 0, UnitKind::Settler, h);
        crate::visibility::update_explored(&mut s);
        assert!(!crate::settlement::known_legal_site(
            &crate::view::player_view(&s, PlayerId(0)),
            h
        ));
        assert!(
            crate::rules::cities::check_found_city(&s, settler)
                .unwrap_err()
                .contains("Victoria Falls")
        );
    }

    #[test]
    fn science_wonder_pays_stored_science_once() {
        let mut s = fixture(&[0, 1]);
        let h = hex(5, 5);
        s.map.get_mut(h).unwrap().wonder = Some(Wonder::MountFuji);
        unit(&mut s, 0, UnitKind::Scout, hex(4, 5));
        let mut events = Vec::new();
        discover(&mut s, &mut events);
        discover(&mut s, &mut events);
        assert_eq!(s.player(PlayerId(0)).science_stored, 50);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn the_observing_empire_receives_the_reward_on_a_shared_team() {
        let mut s = fixture(&[0, 0]);
        let h = hex(5, 5);
        s.map.get_mut(h).unwrap().wonder = Some(Wonder::IguazuFalls);
        unit(&mut s, 1, UnitKind::Scout, hex(4, 5));
        let mut events = Vec::new();
        discover(&mut s, &mut events);
        assert_eq!(s.player(PlayerId(0)).gold, 0);
        assert_eq!(s.player(PlayerId(1)).gold, 80);
    }

    #[test]
    fn cached_city_view_updates_when_distant_wonder_changes_hands() {
        let mut s = fixture(&[0, 1]);
        let id = city(&mut s, 0, hex(2, 4), 3);
        let h = hex(7, 4); // beyond the city's three-hex work radius
        s.map.get_mut(h).unwrap().wonder = Some(Wonder::VictoriaFalls);
        let mut cache = crate::yields_cache::ViewCache::default();
        let food = |s: &GameState, cache: &mut crate::yields_cache::ViewCache| {
            let public = crate::view::public_view(s);
            crate::view::player_view_cached(
                s,
                PlayerId(0),
                &public,
                crate::view::ViewDetail::Full,
                cache,
            )
            .cities
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .yields
            .total
            .food
        };
        let before = food(&s, &mut cache);
        s.map.get_mut(h).unwrap().city = Some(id);
        s.map.get_mut(h).unwrap().owner = Some(PlayerId(0));
        let after = food(&s, &mut cache);
        assert!(after > before);
        s.map.get_mut(h).unwrap().city = None;
        s.map.get_mut(h).unwrap().owner = None;
        assert_eq!(food(&s, &mut cache), before);
    }
}
