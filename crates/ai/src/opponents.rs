//! Persistent rival assessments from the same fogged view available to a human.
//!
//! Public faction rules are priors, not knowledge of an opponent's private opening. Army
//! changes are changes in sightings, not inferred production; an unseen garrison is unknown.

use legatum_core::data;
use legatum_core::diplomacy::RelationState;
use legatum_core::view::{PlayerView, UnitView};
use legatum_core::{CityId, Faction, Hex, PlayerId, UnitKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const ASSESSMENT_INTERVAL: u32 = 5;
const BORDER_RADIUS: i32 = 5;
pub(crate) const NEIGHBOR_RADIUS: i32 = 10;

/// A faction built to rush, by its public unit rules: Spearmen with more moves, more strength
/// per Production, or infantry that ignore wounds.
pub fn rush_capable(faction: Faction) -> bool {
    let soldier = data::unit_stats_for(faction, UnitKind::Spearman);
    let baseline = data::unit_stats(UnitKind::Spearman);
    soldier.moves > baseline.moves
        || soldier.strength * baseline.cost > baseline.strength * soldier.cost
        || data::ignores_wounds(faction, UnitKind::Spearman)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySwitch {
    pub turn: u32,
    pub player: PlayerId,
    pub rival: Option<PlayerId>,
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssessedCity {
    pub id: CityId,
    pub pos: Hex,
    /// Whether the wall and garrison observation is current at assessment time.
    pub visible: bool,
    /// Last known wall state; it may have changed when `visible` is false.
    pub walls: bool,
    /// Visible military on the city and its adjacent tiles. None means unknown, not zero.
    pub garrison_strength: Option<i32>,
    /// Shortest known land route in hex steps from our cities (army if cityless), allowing war with this
    /// owner but respecting other empires' borders. None is no *known* route, not impossible.
    pub distance: Option<i32>,
    pub capture_bounty: Option<i32>,
    /// Known links only; unobserved cities never contribute to this number.
    #[serde(default, alias = "japan_links", alias = "choir_links")]
    pub india_links: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpponentAssessment {
    pub id: PlayerId,
    pub faction: Faction,
    pub first_contact_turn: u32,
    pub assessed_turn: u32,
    pub rush_capable: bool,
    pub border_strength: i32,
    /// Change in visible strength since the last assessment (not production or true losses).
    pub border_strength_change: i32,
    pub observed_city_count: u32,
    pub city_count_change: i32,
    /// Elapsed turns underlying the two observed changes.
    pub observation_turns: u32,
    /// Public city count, including cities whose location we have not explored.
    pub public_city_count: u32,
    pub public_city_count_change: i32,
    /// Distinct declarations observed in public relations/history; a lower bound.
    pub declared_wars: u32,
    pub nearest_distance: Option<i32>,
    pub can_attack: bool,
    pub cities: Vec<AssessedCity>,
    seen_wars: BTreeSet<u32>,
    own_army_strength: i32,
}

impl OpponentAssessment {
    pub fn aggressive_neighbor(&self) -> bool {
        self.nearest_distance.is_some_and(|d| d <= NEIGHBOR_RADIUS)
            && (self.rush_capable
                || self.border_strength >= 2 * data::unit_stats(UnitKind::Spearman).strength
                || self.border_strength_change >= data::unit_stats(UnitKind::Spearman).strength / 2
                || self.declared_wars > 0)
    }

    /// Aggression we have actually observed from a neighbour: an army massing at our border
    /// or a war it declared. Unlike `aggressive_neighbor` this ignores faction priors, so a
    /// rush-capable neighbour at peace earns more units (`DefendedExpand`), not Walls.
    pub fn observed_aggression(&self) -> bool {
        self.nearest_distance.is_some_and(|d| d <= NEIGHBOR_RADIUS)
            && (self.border_strength >= 2 * data::unit_stats(UnitKind::Spearman).strength
                || self.border_strength_change >= data::unit_stats(UnitKind::Spearman).strength / 2
                || self.declared_wars > 0)
    }

    pub fn viable_targets(&self, early: bool) -> impl Iterator<Item = &AssessedCity> {
        self.cities.iter().filter(move |city| {
            self.can_attack
                && city.distance.is_some()
                && (!early || !city.walls)
                && city.garrison_strength.is_some_and(|s| {
                    s <= if early {
                        data::unit_stats(UnitKind::Spearman).strength
                    } else {
                        (self.own_army_strength / 2)
                            .max(data::unit_stats(UnitKind::Spearman).strength * 2)
                    }
                })
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpponentAssessments {
    pub rivals: BTreeMap<PlayerId, OpponentAssessment>,
    /// Per-turn public conquest evidence and threat scores (absent in older saves).
    #[serde(default)]
    pub threats: crate::threat::ThreatTracker,
}

impl OpponentAssessments {
    /// Refresh every five turns and immediately on first physical contact with a rival.
    /// Public roster entries alone do not disclose location or imply contact. Returns true
    /// if any assessment changed. Repeated calls for the same view are idempotent.
    pub fn update(&mut self, view: &PlayerView) -> bool {
        self.update_with_risk(view, false)
    }

    /// Refresh a rival on every turn that a visible army could reach a home city
    /// within two movement turns. Distant records keep the five-turn cadence.
    pub fn update_with_risk(&mut self, view: &PlayerView, rapid_near_army: bool) -> bool {
        let mut changed = false;
        self.rivals.retain(|id, _| {
            let retain = view
                .players
                .iter()
                .any(|p| p.id == *id && p.alive && p.team != view.team);
            changed |= !retain;
            retain
        });
        for player in view
            .players
            .iter()
            .filter(|p| p.alive && p.team != view.team)
        {
            let old = self.rivals.get(&player.id);
            let imminent_units: BTreeSet<_> = if rapid_near_army {
                view.units
                    .iter()
                    .filter(|u| {
                        u.owner == player.id
                            && u.military
                            && view.cities.iter().any(|c| {
                                c.owner == view.me
                                    && crate::hard_defense::declaration_arrival_turns(
                                        view, u, c.pos, 2,
                                    )
                                    .is_some()
                            })
                    })
                    .map(|u| u.id)
                    .collect()
            } else {
                BTreeSet::new()
            };
            let imminent = !imminent_units.is_empty();
            let interval = if imminent { 1 } else { ASSESSMENT_INTERVAL };
            if old.is_some_and(|old| view.turn.saturating_sub(old.assessed_turn) < interval) {
                continue;
            }
            if old.is_none()
                && !view.units.iter().any(|u| u.owner == player.id)
                && !view.cities.iter().any(|c| c.owner == player.id)
                && !view
                    .tiles
                    .iter()
                    .any(|t| t.explored && t.owner == Some(player.id))
            {
                continue;
            }
            let distances = known_distances(view, player.id);
            let border_tiles: Vec<_> = view
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.owner == Some(view.me))
                .map(|(i, _)| Hex::from_offset(i as i32 % view.width, i as i32 / view.width))
                .chain(
                    view.cities
                        .iter()
                        .filter(|c| c.owner == view.me)
                        .map(|c| c.pos),
                )
                .collect();
            let border_strength = view
                .units
                .iter()
                .filter(|u| {
                    u.owner == player.id
                        && u.military
                        && (border_tiles
                            .iter()
                            .any(|h| h.distance(u.pos) <= BORDER_RADIUS)
                            || imminent_units.contains(&u.id))
                })
                .map(unit_power)
                .sum();
            let known_cities: Vec<_> = view
                .cities
                .iter()
                .filter(|c| c.owner == player.id)
                .collect();
            let mut cities: Vec<_> = known_cities
                .iter()
                .map(|city| AssessedCity {
                    id: city.id,
                    pos: city.pos,
                    visible: city.visible,
                    walls: city.walls,
                    garrison_strength: city.visible.then(|| {
                        view.units
                            .iter()
                            .filter(|u| {
                                u.owner == player.id && u.military && u.pos.distance(city.pos) <= 1
                            })
                            .map(unit_power)
                            .sum()
                    }),
                    distance: view.index(city.pos).and_then(|i| distances[i]),
                    capture_bounty: city.capture_bounty,
                    india_links: if player.faction == Faction::India {
                        known_cities
                            .iter()
                            .filter(|other| {
                                other.id != city.id
                                    && other.pos.distance(city.pos) <= data::INDIA_NETWORK_RADIUS
                            })
                            .count() as u32
                    } else {
                        0
                    },
                })
                .collect();
            cities.sort_by_key(|c| c.id);
            let nearest_distance = known_cities
                .iter()
                .map(|c| c.pos)
                .chain(
                    view.units
                        .iter()
                        .filter(|u| u.owner == player.id)
                        .map(|u| u.pos),
                )
                .filter_map(|h| view.index(h).and_then(|i| distances[i]))
                .min();
            let mut seen_wars = old.map(|o| o.seen_wars.clone()).unwrap_or_default();
            for relation in &view.diplomacy.relations {
                if relation.declarers.contains(&player.team)
                    && let Some(id) = relation.war_id
                {
                    seen_wars.insert(id);
                }
            }
            let declared_wars = (seen_wars.len() as u32).max(old.map_or(0, |o| o.declared_wars));
            let rush_capable = rush_capable(player.faction);
            let can_attack = view.diplomacy.legacy
                || view
                    .diplomacy
                    .relations
                    .iter()
                    .find(|r| r.between(view.team, player.team))
                    .is_none_or(|r| {
                        r.state == RelationState::War
                            || (r.state == RelationState::Peace && r.guarantee_until <= view.turn)
                    });
            let observed_city_count = cities.len() as u32;
            let assessment = OpponentAssessment {
                id: player.id,
                faction: player.faction,
                first_contact_turn: old.map_or(view.turn, |o| o.first_contact_turn),
                assessed_turn: view.turn,
                rush_capable,
                border_strength,
                border_strength_change: old.map_or(0, |o| border_strength - o.border_strength),
                observed_city_count,
                city_count_change: old.map_or(0, |o| {
                    observed_city_count as i32 - o.observed_city_count as i32
                }),
                observation_turns: old.map_or(0, |o| view.turn.saturating_sub(o.assessed_turn)),
                public_city_count: player.cities,
                public_city_count_change: old
                    .map_or(0, |o| player.cities as i32 - o.public_city_count as i32),
                declared_wars,
                nearest_distance,
                can_attack,
                cities,
                seen_wars,
                own_army_strength: view
                    .units
                    .iter()
                    .filter(|u| u.owner == view.me && u.military)
                    .map(unit_power)
                    .sum(),
            };
            self.rivals.insert(player.id, assessment);
            changed = true;
        }
        self.threats.observe(view, &self.rivals);
        changed
    }

    /// Deterministic strongest perceived threat, with player ID breaking ties.
    pub fn threatening_neighbor(&self) -> Option<&OpponentAssessment> {
        self.rivals
            .values()
            .filter(|r| r.aggressive_neighbor())
            .max_by_key(|r| (r.border_strength, r.rush_capable, std::cmp::Reverse(r.id)))
    }

    pub fn has_viable_target(&self, early: bool) -> bool {
        self.rivals
            .values()
            .any(|r| r.viable_targets(early).next().is_some())
    }
}

fn unit_power(unit: &UnitView) -> i32 {
    crate::supply::power(unit)
}

/// One inexpensive flood fill per rival per refresh. This certifies only known land routes;
/// unknown terrain never proves that a rival across water is reachable. Units can move, so
/// occupancy is left to the tactical planner, as are movement costs and simultaneous orders.
fn known_distances(view: &PlayerView, rival: PlayerId) -> Vec<Option<i32>> {
    let mut distances = vec![None; view.tiles.len()];
    let mut queue = VecDeque::new();
    let cityless = !view.cities.iter().any(|c| c.owner == view.me);
    for pos in view
        .cities
        .iter()
        .filter(|c| c.owner == view.me)
        .map(|c| c.pos)
        .chain(
            view.units
                .iter()
                .filter(|u| cityless && u.owner == view.me && u.military)
                .map(|u| u.pos),
        )
    {
        if let Some(i) = view.index(pos)
            && distances[i].is_none()
        {
            distances[i] = Some(0);
            queue.push_back(pos);
        }
    }
    while let Some(pos) = queue.pop_front() {
        let distance = distances[view.index(pos).expect("queued tile is on map")].unwrap_or(0);
        for next in pos.neighbors() {
            let Some(i) = view.index(next) else { continue };
            let tile = &view.tiles[i];
            if distances[i].is_none()
                && tile.explored
                && tile.terrain.is_some_and(|t| t.move_cost().is_some())
                && (tile.owner == Some(rival)
                    || view.passage_allowed(view.me, legatum_core::UnitKind::Spearman, tile.owner))
            {
                distances[i] = Some(distance + 1);
                queue.push_back(next);
            }
        }
    }
    distances
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};

    #[test]
    fn old_japan_link_assessments_load_as_india_links() {
        let city: AssessedCity = serde_json::from_value(serde_json::json!({
            "id": 4,
            "pos": { "q": 2, "r": 3 },
            "visible": true,
            "walls": false,
            "garrison_strength": null,
            "distance": 5,
            "capture_bounty": null,
            "japan_links": 2
        }))
        .unwrap();
        assert_eq!(city.india_links, 2);
    }

    fn view(faction: Faction) -> PlayerView {
        let mut view = ViewBuilder::new(18, 8)
            .city(0, ME, off(2, 3), 3)
            .city(1, ENEMY, off(9, 3), 3)
            .build();
        view.players
            .iter_mut()
            .find(|p| p.id == ENEMY)
            .unwrap()
            .faction = faction;
        view
    }

    #[test]
    fn cadence_and_first_contact_are_independent_per_rival() {
        let mut view = view(Faction::Mongols);
        let mut assessments = OpponentAssessments::default();
        assert!(assessments.update(&view));
        assert_eq!(assessments.rivals.len(), 1);
        assert!(assessments.rivals[&ENEMY].rush_capable);
        assert!(!assessments.update(&view));
        view.turn += 1;
        let third = PlayerId(3);
        view.tiles[0].owner = Some(third);
        assert!(assessments.update(&view));
        assert_eq!(assessments.rivals[&third].first_contact_turn, view.turn);
        assert_eq!(assessments.rivals[&ENEMY].assessed_turn, view.turn - 1);
        view.turn += ASSESSMENT_INTERVAL - 1;
        assert!(assessments.update(&view));
        assert_eq!(assessments.rivals[&ENEMY].assessed_turn, view.turn);
        assert_eq!(assessments.rivals[&third].assessed_turn, view.turn - 4);
    }

    #[test]
    fn walls_vey_r_and_fog_are_not_soft_targets() {
        let mut view = view(Faction::Athens);
        let mut assessments = OpponentAssessments::default();
        assessments.update(&view);
        assert!(assessments.has_viable_target(true));
        view.cities[1].visible = false;
        view.turn += ASSESSMENT_INTERVAL;
        assessments.update(&view);
        assert_eq!(assessments.rivals[&ENEMY].cities[0].garrison_strength, None);
        assert!(!assessments.has_viable_target(true));
        view.cities[1].visible = true;
        view.cities[1].walls = true;
        view.turn += ASSESSMENT_INTERVAL;
        assessments.update(&view);
        assert!(!assessments.has_viable_target(true));
        assert!(assessments.has_viable_target(false));
    }

    #[test]
    fn rivals_treat_an_open_mongols_city_as_an_early_target() {
        let mut view = view(Faction::Mongols);
        let mut assessments = OpponentAssessments::default();
        assessments.update(&view);
        assert!(assessments.has_viable_target(true));
        view.cities[1].walls = true; // A captured city can retain another empire's Walls.
        view.turn += ASSESSMENT_INTERVAL;
        assessments.update(&view);
        assert!(!assessments.has_viable_target(true));
    }

    #[test]
    fn an_ordinary_starting_garrison_is_not_an_army_buildup() {
        let view = ViewBuilder {
            view: view(Faction::Athens),
        }
        .unit(1, ENEMY, UnitKind::Spearman, off(6, 3))
        .build();
        let mut model = OpponentAssessments::default();
        model.update(&view);
        assert!(model.threatening_neighbor().is_none());
        let mut buildup = ViewBuilder { view }
            .unit(2, ENEMY, UnitKind::Spearman, off(7, 3))
            .build();
        buildup.turn += ASSESSMENT_INTERVAL;
        model.update(&buildup);
        assert_eq!(model.threatening_neighbor().map(|r| r.id), Some(ENEMY));
    }

    #[test]
    fn unknown_land_does_not_establish_reachability() {
        let mut view = view(Faction::Mongols);
        for row in 0..view.height {
            let i = view.index(off(6, row)).unwrap();
            view.tiles[i].explored = false;
            view.tiles[i].terrain = None;
        }
        let mut assessments = OpponentAssessments::default();
        assessments.update(&view);
        assert_eq!(assessments.rivals[&ENEMY].nearest_distance, None);
        assert!(!assessments.has_viable_target(true));
        assert!(assessments.threatening_neighbor().is_none());
    }

    #[test]
    fn observations_are_deterministic_under_view_reordering() {
        let mut view = ViewBuilder::new(18, 8)
            .city(0, ME, off(2, 3), 3)
            .city(1, ENEMY, off(9, 3), 3)
            .city(2, ENEMY, off(13, 3), 3)
            .unit(1, ENEMY, UnitKind::Spearman, off(5, 3))
            .build();
        view.players
            .iter_mut()
            .find(|p| p.id == ENEMY)
            .unwrap()
            .faction = Faction::India;
        let mut first = OpponentAssessments::default();
        first.update(&view);
        let mut reordered = view.clone();
        reordered.cities.reverse();
        reordered.units.reverse();
        reordered.players.reverse();
        let mut second = OpponentAssessments::default();
        second.update(&reordered);
        assert_eq!(first, second);
        assert_eq!(first.rivals[&ENEMY].cities[0].india_links, 1);
        view.turn += ASSESSMENT_INTERVAL;
        view.units.clear();
        first.update(&view);
        assert!(first.rivals[&ENEMY].border_strength_change < 0);
        // A disappearing sighting is only a change in observed strength.
        assert_eq!(first.rivals[&ENEMY].observation_turns, ASSESSMENT_INTERVAL);
    }

    #[test]
    fn public_bounty_and_declarations_survive_cadence_without_double_counting() {
        use legatum_core::TeamId;
        use legatum_core::diplomacy::{Relation, pair};
        let mut view = view(Faction::Switzerland);
        view.cities[1].capture_bounty = Some(240);
        let (a, b) = pair(TeamId(ME.0), TeamId(ENEMY.0));
        view.diplomacy.relations.push(Relation {
            a,
            b,
            state: RelationState::War,
            effective_turn: view.turn.saturating_sub(1),
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(1),
            declarers: vec![TeamId(ENEMY.0)],
        });
        let mut assessments = OpponentAssessments::default();
        assessments.update(&view);
        assert_eq!(
            assessments.rivals[&ENEMY].cities[0].capture_bounty,
            Some(240)
        );
        assert_eq!(assessments.rivals[&ENEMY].declared_wars, 1);
        view.turn += ASSESSMENT_INTERVAL;
        assessments.update(&view);
        assert_eq!(assessments.rivals[&ENEMY].declared_wars, 1);
        // The API keeps a declaration already observed even if relations later change.
        view.diplomacy.relations.clear();
        view.turn += ASSESSMENT_INTERVAL;
        assessments.update(&view);
        assert_eq!(assessments.rivals[&ENEMY].declared_wars, 1);
    }

    #[test]
    fn remote_scout_does_not_make_a_rival_a_homeland_neighbor() {
        let view = ViewBuilder::new(30, 8)
            .city(0, ME, off(2, 3), 3)
            .city(1, ENEMY, off(24, 3), 3)
            .unit(1, ME, UnitKind::Scout, off(23, 3))
            .build();
        let mut assessments = OpponentAssessments::default();
        assessments.update(&view);
        assert!(assessments.rivals[&ENEMY].nearest_distance.unwrap() > NEIGHBOR_RADIUS);
    }

    #[test]
    fn sparta_prior_uses_the_public_unit_rules() {
        let view = view(Faction::Sparta);
        let mut assessments = OpponentAssessments::default();
        assessments.update(&view);
        assert!(assessments.threatening_neighbor().is_some());
    }
}
