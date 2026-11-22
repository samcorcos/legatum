//! Built wonders occupy a worked tile. The tile owns the landmark, so its present owner
//! receives a held Great Work's points even if borders change without a city capture.
use crate::{
    data, economy,
    hex::Hex,
    state::{City, GameState},
    types::*,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Landmark {
    pub building: BuildingKind,
    pub city: CityId,
    /// The empire that built this work or faction goal.
    pub of: PlayerId,
}

pub fn is_landmark(building: BuildingKind) -> bool {
    data::is_great_work(building) || building == BuildingKind::CourierHall
}

pub fn check<R: economy::Reason>(
    state: &GameState,
    city: &City,
    building: BuildingKind,
    tile: Hex,
) -> Option<R> {
    if !is_landmark(building) {
        return Some(R::word(|| {
            "This building does not use a wonder tile".into()
        }));
    }
    let Some(t) = state.map.get(tile) else {
        return Some(R::word(|| "That tile is off the map".into()));
    };
    if !t.terrain.is_land()
        || t.city != Some(city.id)
        || t.owner != Some(city.owner)
        || tile == city.pos
        || tile.distance(city.pos) > data::CITY_WORK_RADIUS
    {
        return Some(R::word(|| {
            format!(
                "Choose land owned by {} within three tiles, outside its centre",
                city.name
            )
        }));
    }
    if t.landmark.is_some() || t.wonder.is_some() {
        return Some(R::word(|| "That tile already holds a wonder".into()));
    }
    None
}

pub fn candidates(state: &GameState, city: &City, building: BuildingKind) -> Vec<ProductionItem> {
    city.pos
        .within(data::CITY_WORK_RADIUS)
        .into_iter()
        .filter(|h| check::<economy::Blocked>(state, city, building, *h).is_none())
        .map(|tile| ProductionItem::Wonder { building, tile })
        .collect()
}

pub fn complete(state: &mut GameState, city: CityId, building: BuildingKind, tile: Hex) {
    let owner = state.cities[&city].owner;
    let t = state.map.get_mut(tile).expect("validated wonder tile");
    t.improved = false;
    t.landmark = Some(Landmark {
        building,
        city,
        of: owner,
    });
}

/// A border claim can attach a surviving wonder tile to a different city.
pub fn attach_city(state: &mut GameState, tile: Hex, city: CityId) {
    let Some(site) = state.map.get(tile).and_then(|t| t.landmark) else {
        return;
    };
    if site.city != city
        && let Some(old) = state.cities.get_mut(&site.city)
    {
        old.buildings.remove(&site.building);
    }
    if let Some(new) = state.cities.get_mut(&city) {
        new.buildings.insert(site.building);
    }
    if site.building == BuildingKind::CourierHall {
        state.player_mut(site.of).hall_city = Some(city);
    }
    state
        .map
        .get_mut(tile)
        .expect("site exists")
        .landmark
        .as_mut()
        .expect("site exists")
        .city = city;
}

pub fn site(
    state: &GameState,
    of: PlayerId,
    building: BuildingKind,
) -> Option<(Hex, Option<PlayerId>)> {
    state.map.tiles.iter().enumerate().find_map(|(i, t)| {
        let site = t.landmark?;
        (site.of == of && site.building == building).then_some((state.map.hex_at(i), t.owner))
    })
}

pub fn find(state: &GameState, of: PlayerId, building: BuildingKind) -> Option<(Hex, PlayerId)> {
    site(state, of, building).and_then(|(tile, owner)| Some((tile, owner?)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::cities::tests::{P0, P1, at, found, test_game, two_players};

    #[test]
    fn placement_requires_owned_land_outside_centre_and_an_empty_site() {
        let mut s = two_players();
        let id = found(&mut s, P0, at(5, 5));
        assert!(
            check::<economy::Blocked>(&s, &s.cities[&id], BuildingKind::RelayUplink, at(5, 5))
                .is_some()
        );
        assert!(
            check::<economy::Blocked>(&s, &s.cities[&id], BuildingKind::RelayUplink, at(6, 5))
                .is_none()
        );
        assert!(
            check::<economy::Blocked>(&s, &s.cities[&id], BuildingKind::RelayUplink, at(9, 5))
                .is_some()
        );
        s.map.get_mut(at(6, 5)).unwrap().landmark = Some(Landmark {
            building: BuildingKind::CourierHall,
            city: id,
            of: P0,
        });
        assert!(
            check::<economy::Blocked>(&s, &s.cities[&id], BuildingKind::RelayUplink, at(6, 5))
                .is_some()
        );
    }

    #[test]
    fn completed_site_replaces_tile_yield_and_improvement() {
        let mut s = two_players();
        let id = found(&mut s, P0, at(5, 5));
        let tile = at(6, 5);
        let before =
            crate::resources::tile_yields(Faction::Athens, s.map.get(tile).unwrap(), |_| false);
        assert_ne!(before, Yields::ZERO);
        s.map.get_mut(tile).unwrap().improved = true;
        complete(&mut s, id, BuildingKind::RelayUplink, tile);
        assert!(!s.map.get(tile).unwrap().improved);
        assert_eq!(
            crate::resources::tile_yields(Faction::Athens, s.map.get(tile).unwrap(), |_| false),
            Yields::ZERO
        );
    }

    #[test]
    fn changing_production_banks_progress_for_the_chosen_site() {
        let mut s = two_players();
        let id = found(&mut s, P0, at(5, 5));
        let item = ProductionItem::Wonder {
            building: BuildingKind::RelayUplink,
            tile: at(6, 5),
        };
        let city = s.cities.get_mut(&id).unwrap();
        city.producing = Some(item);
        city.production_stored = 100;
        city.switch_production(None);
        assert_eq!(city.banked(item), 100);
        city.switch_production(Some(item));
        assert_eq!(city.production_stored, 100);
    }

    #[test]
    fn celtic_sack_destroys_the_site_and_its_held_points() {
        let mut s = test_game(&[(Faction::Athens, 0), (Faction::Celts, 1)]);
        let id = found(&mut s, P0, at(5, 5));
        found(&mut s, P1, at(12, 12));
        s.cities
            .get_mut(&id)
            .unwrap()
            .buildings
            .insert(BuildingKind::RelayUplink);
        complete(&mut s, id, BuildingKind::RelayUplink, at(6, 5));
        s.player_mut(P0).great_work_completed = Some(BuildingKind::RelayUplink);
        assert!(
            crate::scoring::vp(&s, P0)
                .iter()
                .any(|(src, _)| matches!(src, VpSource::GreatWork { .. }))
        );
        crate::rules::battle::capture_city(&mut s, id, P1, &mut Vec::new());
        assert!(s.map.get(at(6, 5)).unwrap().landmark.is_none());
        assert!(
            !crate::scoring::vp(&s, P1)
                .iter()
                .any(|(src, _)| matches!(src, VpSource::GreatWork { .. }))
        );
    }

    #[test]
    fn population_loss_keeps_the_work_site_until_city_capture() {
        let mut s = two_players();
        let first = found(&mut s, P0, at(5, 5));
        found(&mut s, P1, at(10, 5));
        let tile = at(7, 5);
        let t = s.map.get_mut(tile).unwrap();
        t.owner = Some(P0);
        t.city = Some(first);
        s.cities.get_mut(&first).unwrap().grown_tiles.push(tile);
        s.cities
            .get_mut(&first)
            .unwrap()
            .buildings
            .insert(BuildingKind::RelayUplink);
        complete(&mut s, first, BuildingKind::RelayUplink, tile);
        s.player_mut(P0).great_work_completed = Some(BuildingKind::RelayUplink);
        assert!(
            crate::scoring::vp(&s, P0)
                .iter()
                .any(|(src, _)| matches!(src, VpSource::GreatWork { .. }))
        );
        crate::rules::cities::settle_territory(&mut s, first, &mut Vec::new());
        assert!(s.map.get(tile).unwrap().landmark.is_some());
        assert_eq!(s.map.get(tile).unwrap().city, Some(first));
        assert_eq!(s.map.get(tile).unwrap().owner, Some(P0));
        assert!(
            s.cities[&first]
                .buildings
                .contains(&BuildingKind::RelayUplink)
        );
        assert!(
            crate::scoring::vp(&s, P0)
                .iter()
                .any(|(src, _)| matches!(src, VpSource::GreatWork { .. }))
        );
        crate::rules::battle::capture_city(&mut s, first, P1, &mut Vec::new());
        assert_eq!(s.map.get(tile).unwrap().city, Some(first));
        assert_eq!(s.map.get(tile).unwrap().owner, Some(P1));
        assert!(
            s.cities[&first]
                .buildings
                .contains(&BuildingKind::RelayUplink)
        );
        assert!(
            crate::scoring::vp(&s, P1)
                .iter()
                .any(|(src, _)| matches!(src, VpSource::GreatWork { of: P0 }))
        );
    }
}
