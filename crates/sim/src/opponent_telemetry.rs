//! Observer-only target-quality metrics. This module depends on core, never the AI,
//! so exactly the same observer can measure a baseline without opponent assessment.
use legatum_core::{
    BuildingKind, CityId, Event, Faction, GameState, Hex, PlayerId, TeamId, UnitKind, data,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFINITION: &str = "Target status is snapshotted before resolution. An assault-turn is one player/city/turn with a unit Battle against the city or a defending-team unit on its hex (multiple attacks count once). Unit defenders must belong to the city's pre-turn team, so fighting a new captor's garrison is not attributed to the former defender. A failed assault-turn has a lethal melee city or garrison attack and no CityCaptured by that attacker that turn; noncapture assault-turns also include ordinary siege progress and are reported separately. Walled uses core has_building(Walls), including replacements; Celts uses the defending owner's faction. Soft means unwalled, non-Celts, at most one defending-team non-Scout military unit within two hexes before resolution. Captures count CityCaptured events, including captures without Battle and recaptures. Cities founded during the turn lack a pre-turn snapshot and are excluded. Observer truth never enters AI decisions. Walled and Celts categories overlap; never sum them as distinct assaults.";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Counters {
    pub assault_turns: u64,
    pub walled_assault_turns: u64,
    pub celts_assault_turns: u64,
    pub failed_assault_turns: u64,
    pub failed_walled_assault_turns: u64,
    pub failed_celts_assault_turns: u64,
    pub noncapture_walled_assault_turns: u64,
    pub noncapture_celts_assault_turns: u64,
    pub soft_target_captures: u64,
}

impl Counters {
    pub fn add(&mut self, other: &Self) {
        self.assault_turns += other.assault_turns;
        self.walled_assault_turns += other.walled_assault_turns;
        self.celts_assault_turns += other.celts_assault_turns;
        self.failed_assault_turns += other.failed_assault_turns;
        self.failed_walled_assault_turns += other.failed_walled_assault_turns;
        self.failed_celts_assault_turns += other.failed_celts_assault_turns;
        self.noncapture_walled_assault_turns += other.noncapture_walled_assault_turns;
        self.noncapture_celts_assault_turns += other.noncapture_celts_assault_turns;
        self.soft_target_captures += other.soft_target_captures;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Target {
    team: TeamId,
    walled: bool,
    celts: bool,
    soft: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observer {
    before: BTreeMap<CityId, Target>,
    city_at: BTreeMap<Hex, CityId>,
    teams: BTreeMap<PlayerId, TeamId>,
    pub seats: BTreeMap<u8, Counters>,
}

impl Observer {
    pub fn before(&mut self, state: &GameState) {
        self.city_at = state
            .cities
            .values()
            .map(|city| (city.pos, city.id))
            .collect();
        self.teams = state
            .players
            .iter()
            .map(|player| (player.id, player.team))
            .collect();
        for player in &state.players {
            self.seats.entry(player.id.0).or_default();
        }
        self.before = state
            .cities
            .values()
            .map(|city| {
                let walled = data::has_building(&city.buildings, BuildingKind::Walls);
                let celts = state.player(city.owner).faction == Faction::Celts;
                let defenders = state
                    .units
                    .values()
                    .filter(|unit| {
                        state.team_of(unit.owner) == state.team_of(city.owner)
                            && unit.kind != UnitKind::Scout
                            && data::unit_stats(unit.kind).military
                            && unit.pos.distance(city.pos) <= 2
                    })
                    .take(2)
                    .count();
                (
                    city.id,
                    Target {
                        team: state.team_of(city.owner),
                        walled,
                        celts,
                        soft: !walled && !celts && defenders <= 1,
                    },
                )
            })
            .collect();
    }

    pub fn after(&mut self, events: &[Event]) {
        let captures: BTreeSet<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::CityCaptured { city, to, .. } => Some((*to, *city)),
                _ => None,
            })
            .collect();
        let mut assaults: BTreeMap<(PlayerId, CityId), bool> = BTreeMap::new();
        for event in events {
            match event {
                Event::Battle {
                    attacker,
                    attacker_owner,
                    defender,
                    at,
                    attacker_hp_after,
                    ranged,
                    ..
                } if attacker.unit().is_some() && *attacker_owner != PlayerId::BARBARIAN => {
                    let city = match defender {
                        legatum_core::events::Defender::City { city, .. } => Some(*city),
                        legatum_core::events::Defender::Unit { owner, .. } => {
                            self.city_at.get(at).copied().filter(|city| {
                                self.before.get(city).is_some_and(|target| {
                                    self.teams.get(owner) == Some(&target.team)
                                        && self.teams.get(attacker_owner) != Some(&target.team)
                                })
                            })
                        }
                    };
                    if let Some(city) = city {
                        *assaults.entry((*attacker_owner, city)).or_default() |=
                            !ranged && *attacker_hp_after == 0;
                    }
                }
                Event::CityCaptured { city, to, .. }
                    if *to != PlayerId::BARBARIAN
                        && self.before.get(city).is_some_and(|target| target.soft) =>
                {
                    self.seats.entry(to.0).or_default().soft_target_captures += 1;
                }
                _ => {}
            }
        }
        for ((player, city), lethal) in assaults {
            let Some(target) = self.before.get(&city) else {
                continue;
            };
            let uncaptured = !captures.contains(&(player, city));
            let failed = lethal && uncaptured;
            let counters = self.seats.entry(player.0).or_default();
            counters.assault_turns += 1;
            counters.walled_assault_turns += u64::from(target.walled);
            counters.celts_assault_turns += u64::from(target.celts);
            counters.failed_assault_turns += u64::from(failed);
            counters.failed_walled_assault_turns += u64::from(failed && target.walled);
            counters.failed_celts_assault_turns += u64::from(failed && target.celts);
            counters.noncapture_walled_assault_turns += u64::from(uncaptured && target.walled);
            counters.noncapture_celts_assault_turns += u64::from(uncaptured && target.celts);
        }
        self.before.clear();
        self.city_at.clear();
        self.teams.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{
        GameSetup, Hex, Terrain, UnitId,
        events::{Attacker, Defender},
        game::{blank_game, spawn_unit},
        rules::cities::found_city,
        state::Map,
    };

    fn duel() -> (GameState, CityId) {
        let mut setup = GameSetup::all_ai(93);
        setup.players.truncate(2);
        let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Plains));
        state.players[1].faction = Faction::Mongols;
        let settler = spawn_unit(&mut state, PlayerId(1), UnitKind::Settler, Hex::new(12, 5));
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
        let city = state.cities_of(PlayerId(1)).next().unwrap().id;
        (state, city)
    }
    fn hit(city: CityId, hp: i32) -> Event {
        Event::Battle {
            attacker: Attacker::Unit(UnitId(500)),
            attacker_owner: PlayerId(0),
            defender: Defender::City {
                city,
                owner: PlayerId(1),
            },
            at: Hex::new(12, 5),
            attacker_at: None,
            attacker_strength: Default::default(),
            defender_strength: Default::default(),
            damage_to_attacker: 30,
            damage_to_defender: 30,
            attacker_roll: 100,
            defender_roll: 100,
            attacker_hp_after: hp,
            defender_hp_after: 80,
            ranged: false,
            meeting: false,
            air_strike: false,
            defender_leaving: false,
        }
    }
    fn capture(city: CityId) -> Event {
        Event::CityCaptured {
            city,
            from: PlayerId(1),
            to: PlayerId(0),
            bounty: None,
        }
    }
    #[test]
    fn lethal_assaults_deduplicate_and_successful_capture_is_not_failure() {
        let (mut state, city) = duel();
        state.players[1].faction = Faction::Celts;
        state
            .cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        let hash = state.hash();
        let mut observer = Observer::default();
        observer.before(&state);
        observer.after(&[hit(city, 0), hit(city, 0)]);
        let counters = &observer.seats[&0];
        assert_eq!(counters.assault_turns, 1);
        assert_eq!(counters.failed_walled_assault_turns, 1);
        assert_eq!(counters.failed_celts_assault_turns, 1);
        observer.before(&state);
        observer.after(&[hit(city, 0), capture(city)]);
        assert_eq!(observer.seats[&0].failed_assault_turns, 1);
        observer.before(&state);
        observer.after(&[hit(city, 40)]);
        assert_eq!(observer.seats[&0].failed_assault_turns, 1);
        assert_eq!(observer.seats[&0].noncapture_walled_assault_turns, 2);
        assert_eq!(state.hash(), hash);
    }
    fn garrison_hit(city: CityId, hp: i32, owner: PlayerId, at: Hex) -> Event {
        let mut event = hit(city, hp);
        if let Event::Battle {
            defender, at: pos, ..
        } = &mut event
        {
            *defender = Defender::Unit {
                unit: UnitId(600),
                owner,
                unit_kind: UnitKind::Spearman,
            };
            *pos = at;
        }
        event
    }

    #[test]
    fn lethal_garrison_attack_without_capture_is_a_failed_assault() {
        let (mut state, city) = duel();
        state.players[1].faction = Faction::Celts;
        state
            .cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        let mut observer = Observer::default();
        observer.before(&state);
        observer.after(&[
            garrison_hit(city, 0, PlayerId(1), Hex::new(12, 5)),
            hit(city, 30),
        ]);
        assert_eq!(observer.seats[&0].assault_turns, 1);
        assert_eq!(observer.seats[&0].failed_assault_turns, 1);
        assert_eq!(observer.seats[&0].failed_walled_assault_turns, 1);
        assert_eq!(observer.seats[&0].failed_celts_assault_turns, 1);
        observer.before(&state);
        observer.after(&[
            garrison_hit(city, 0, PlayerId(1), Hex::new(12, 5)),
            capture(city),
        ]);
        assert_eq!(observer.seats[&0].assault_turns, 2);
        assert_eq!(observer.seats[&0].failed_assault_turns, 1);
    }

    #[test]
    fn field_combat_and_new_captors_garrison_are_not_old_city_assaults() {
        let (state, city) = duel();
        let mut observer = Observer::default();
        observer.before(&state);
        observer.after(&[garrison_hit(city, 0, PlayerId(1), Hex::new(13, 5))]);
        assert_eq!(observer.seats[&0].assault_turns, 0);
        let mut recapture_attack = garrison_hit(city, 0, PlayerId(0), Hex::new(12, 5));
        if let Event::Battle { attacker_owner, .. } = &mut recapture_attack {
            *attacker_owner = PlayerId(1);
        }
        observer.before(&state);
        observer.after(&[capture(city), recapture_attack]);
        assert_eq!(observer.seats[&1].assault_turns, 0);
    }

    #[test]
    fn ranged_garrison_pressure_is_not_a_lethal_melee_failure() {
        let (state, city) = duel();
        let mut ranged = garrison_hit(city, 0, PlayerId(1), Hex::new(12, 5));
        if let Event::Battle { ranged, .. } = &mut ranged {
            *ranged = true;
        }
        let mut observer = Observer::default();
        observer.before(&state);
        observer.after(&[ranged]);
        assert_eq!(observer.seats[&0].assault_turns, 1);
        assert_eq!(observer.seats[&0].failed_assault_turns, 0);
    }

    #[test]
    fn soft_captures_use_pre_turn_walls_faction_and_nearby_defenders() {
        let (mut state, city) = duel();
        let mut observer = Observer::default();
        observer.before(&state);
        state
            .cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        observer.after(&[capture(city)]);
        assert_eq!(observer.seats[&0].soft_target_captures, 1);
        observer.before(&state);
        observer.after(&[capture(city)]);
        assert_eq!(observer.seats[&0].soft_target_captures, 1);
        state.cities.get_mut(&city).unwrap().buildings.clear();
        spawn_unit(&mut state, PlayerId(1), UnitKind::Spearman, Hex::new(12, 5));
        spawn_unit(&mut state, PlayerId(1), UnitKind::Spearman, Hex::new(13, 5));
        observer.before(&state);
        observer.after(&[capture(city)]);
        assert_eq!(observer.seats[&0].soft_target_captures, 1);
        state.units.clear();
        state.players[1].faction = Faction::Celts;
        observer.before(&state);
        observer.after(&[capture(city)]);
        assert_eq!(observer.seats[&0].soft_target_captures, 1);
        let counters = observer.seats[&0].clone();
        assert_eq!(
            serde_json::from_value::<Counters>(serde_json::to_value(&counters).unwrap()).unwrap(),
            counters
        );
    }
}
