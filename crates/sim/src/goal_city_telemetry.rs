//! Read-only observations of the two held-city faction goals (Rules 51): Rome's Colonia Victrix
//! and Egypt's royal tomb, one record per Rome or Egypt seat. They never touch orders, RNG, saves
//! or hashes.

use legatum_core::{Event, Faction, GameState, PlayerId, UnitClass, data, goals};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One Rome or Egypt seat's goal city over a game. Older reports omit it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalCityTelemetry {
    pub faction: Option<Faction>,
    /// First resolved turn the goal city stood (the Colonia founded, the tomb raised).
    pub established_turn: Option<u32>,
    /// Goal cities established this game (a razed one can be replaced).
    pub established: u32,
    /// First resolved turn the seat itself scored the goal's VP.
    pub first_scored_turn: Option<u32>,
    /// Resolved turns the seat held its own goal city.
    pub held_turns: u32,
    /// Resolved turns a rival held it.
    pub rival_held_turns: u32,
    /// Times the holder changed (captures and recaptures of the goal city).
    pub holder_changes: u32,
    /// At the end: the seat holds its goal city; a rival holds it.
    pub held_at_end: bool,
    pub rival_holds_at_end: bool,
    /// Rome: first resolved turn the seat had a level-3 melee unit while the Colonia was open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub founder_turn: Option<u32>,
    /// Rome: resolved turns with a level-3 melee unit and the Colonia open.
    #[serde(default)]
    pub founder_turns: u32,
    /// Rome: of those, turns on which no legal Colonia site existed anywhere on the map (full
    /// information, reachability ignored): a level-3 veteran had nowhere to found it.
    #[serde(default)]
    pub no_site_turns: u32,
    /// Rome: first resolved turn the seat had a level-2 melee unit while the Colonia was open
    /// (the Colonia's founder in waiting).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level2_turn: Option<u32>,
    /// Rome: resolved turns with at least one level-2 melee unit and the Colonia open.
    #[serde(default)]
    pub level2_turns: u32,
    /// Rome: distinct melee units seen at level 2 while the Colonia was open.
    #[serde(default)]
    pub level2_units: u32,
    /// Rome: turns on which every level-3 melee unit was held in a captured city by the
    /// occupation lock (it cannot leave to found).
    #[serde(default)]
    pub founder_locked_turns: u32,
    /// Egypt: first resolved turn the work was on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_started_turn: Option<u32>,
    /// Egypt: turn ends the work was on (each one counted toward a tomb).
    #[serde(default)]
    pub work_turns: u32,
    /// Egypt: starts (including resumes) and pauses of the work.
    #[serde(default)]
    pub work_starts: u32,
    #[serde(default)]
    pub work_pauses: u32,
}

/// The collector: per seat, plus each seat's last holder to count changes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observer {
    seats: BTreeMap<u8, GoalCityTelemetry>,
    last: BTreeMap<u8, (Option<legatum_core::CityId>, Option<PlayerId>)>,
    level2_seen: std::collections::BTreeSet<legatum_core::UnitId>,
}

/// Whether a legal site beside a rival's border exists anywhere for `p` now.
fn colonia_site_exists(state: &GameState, p: PlayerId) -> bool {
    let gap = state
        .setup
        .rule_experiments
        .min_city_gap(state.player(p).faction);
    (0..state.map.len()).any(|i| {
        let pos = state.map.hex_at(i);
        let tile = &state.map.tiles[i];
        tile.wonder.is_none()
            && legatum_core::settlement::legal_site(
                p,
                gap,
                tile.terrain,
                tile.owner,
                pos,
                state.cities.values().map(|c| c.pos),
            )
            && goals::colonia_site(state, p, pos)
    })
}

impl Observer {
    pub fn observe_initial(&mut self, state: &GameState) {
        for p in &state.players {
            if matches!(
                goals::rule(state, p.id),
                goals::GoalRule::Colonia | goals::GoalRule::Tomb
            ) {
                self.seats.insert(
                    p.id.0,
                    GoalCityTelemetry {
                        faction: Some(p.faction),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn observe_event(&mut self, resolved_turn: u32, event: &Event) {
        match event {
            Event::TombStarted { player, .. } => {
                if let Some(t) = self.seats.get_mut(&player.0) {
                    t.work_starts += 1;
                    t.work_started_turn.get_or_insert(resolved_turn);
                }
            }
            Event::TombPaused { player, .. } => {
                if let Some(t) = self.seats.get_mut(&player.0) {
                    t.work_pauses += 1;
                }
            }
            // The turn end that raises the tomb also ends the work: count it here.
            Event::TombRaised { player, .. } => {
                if let Some(t) = self.seats.get_mut(&player.0) {
                    t.work_turns += 1;
                }
            }
            _ => {}
        }
    }

    pub fn observe_resolved_turn(&mut self, resolved_turn: u32, state: &GameState) {
        for (&seat, t) in &mut self.seats {
            let p = PlayerId(seat);
            let player = state.player(p);
            let (city, holder) = match goals::rule(state, p) {
                goals::GoalRule::Colonia => (
                    goals::colonia_city(state, p).map(|c| c.id),
                    goals::colonia_holder(state, p),
                ),
                _ => (
                    goals::tomb_city(state, p).map(|c| c.id),
                    goals::tomb_holder(state, p),
                ),
            };
            let (last_city, last) = self.last.insert(seat, (city, holder)).unwrap_or_default();
            if city.is_some() && city != last_city {
                t.established += 1;
                t.established_turn.get_or_insert(resolved_turn);
            } else if last.is_some() && holder.is_some() && last != holder {
                t.holder_changes += 1;
            }
            match holder {
                Some(h) if h == p => {
                    t.held_turns += 1;
                    t.first_scored_turn.get_or_insert(resolved_turn);
                }
                Some(_) => t.rival_held_turns += 1,
                None => {}
            }
            t.held_at_end = holder == Some(p);
            t.rival_holds_at_end = holder.is_some_and(|h| h != p);
            if player.tomb_work {
                t.work_turns += 1;
            }
            if player.alive && goals::colonia_open(state, p) {
                let melee = || {
                    state.units.values().filter(|u| {
                        u.owner == p && data::unit_class(u.kind) == Some(UnitClass::Melee)
                    })
                };
                let level = |u: &legatum_core::state::Unit| {
                    legatum_core::rules::promotion::level_for(player.faction, u.xp)
                };
                let mut level2 = false;
                for u in melee().filter(|u| level(u) == goals::COLONIA_LEVEL - 1) {
                    level2 = true;
                    if self.level2_seen.insert(u.id) {
                        t.level2_units += 1;
                    }
                }
                if level2 {
                    t.level2_turns += 1;
                    t.level2_turn.get_or_insert(resolved_turn);
                }
                let founders: Vec<_> = melee()
                    .filter(|u| level(u) >= goals::COLONIA_LEVEL)
                    .collect();
                if !founders.is_empty()
                    && founders
                        .iter()
                        .all(|u| legatum_core::rules::occupation::lock(state, u).is_some())
                {
                    t.founder_locked_turns += 1;
                }
                let founder = !founders.is_empty();
                if founder {
                    t.founder_turns += 1;
                    t.founder_turn.get_or_insert(resolved_turn);
                    if !colonia_site_exists(state, p) {
                        t.no_site_turns += 1;
                    }
                }
            }
        }
    }

    pub fn finish(&self) -> BTreeMap<u8, GoalCityTelemetry> {
        self.seats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{GameSetup, PlayerId};

    fn game() -> (GameState, PlayerId) {
        let mut setup = GameSetup::all_ai(3);
        setup.players[0].faction = Faction::Egypt;
        setup.players[1].faction = Faction::Rome;
        (legatum_core::new_game(&setup), PlayerId(0))
    }

    #[test]
    fn egypt_work_pauses_and_the_tomb_changing_hands_are_counted() {
        let (mut state, egypt) = game();
        let mut o = Observer::default();
        o.observe_initial(&state);
        assert_eq!(o.seats.len(), 2, "one Rome and one Egypt seat");
        o.observe_event(
            3,
            &Event::TombStarted {
                player: egypt,
                turns: 0,
            },
        );
        state.player_mut(egypt).tomb_work = true;
        o.observe_resolved_turn(3, &state);
        o.observe_resolved_turn(4, &state);
        o.observe_event(
            5,
            &Event::TombPaused {
                player: egypt,
                turns: 2,
            },
        );
        state.player_mut(egypt).tomb_work = false;
        o.observe_resolved_turn(5, &state);
        let t = &o.finish()[&egypt.0];
        assert_eq!(
            (
                t.work_started_turn,
                t.work_turns,
                t.work_starts,
                t.work_pauses
            ),
            (Some(3), 2, 1, 1)
        );
        // The tomb stands in Egypt's capital, then a rival takes it.
        let capital = state.cities_of(egypt).next().unwrap().id;
        state.player_mut(egypt).tomb_city = Some(capital);
        o.observe_resolved_turn(20, &state);
        state.cities.get_mut(&capital).unwrap().owner = PlayerId(2);
        o.observe_resolved_turn(21, &state);
        let t = &o.finish()[&egypt.0];
        assert_eq!(t.established_turn, Some(20));
        assert_eq!(t.first_scored_turn, Some(20));
        assert_eq!(
            (t.held_turns, t.rival_held_turns, t.holder_changes),
            (1, 1, 1)
        );
        assert!(!t.held_at_end && t.rival_holds_at_end);
    }

    #[test]
    fn a_roman_level_three_veteran_is_counted_with_or_without_a_site() {
        let (mut state, _) = game();
        let rome = PlayerId(1);
        let mut o = Observer::default();
        o.observe_initial(&state);
        o.observe_resolved_turn(1, &state);
        assert_eq!(o.finish()[&1].founder_turns, 0);
        let unit = state
            .units
            .values_mut()
            .find(|u| u.owner == rome && data::unit_class(u.kind) == Some(UnitClass::Melee))
            .expect("a Roman melee unit at start");
        unit.xp = (0..1000)
            .find(|xp| legatum_core::rules::promotion::level_for(Faction::Rome, *xp) == 2)
            .unwrap();
        o.observe_resolved_turn(2, &state);
        o.observe_resolved_turn(3, &state);
        let t = &o.finish()[&1];
        assert_eq!(
            (t.level2_turn, t.level2_turns, t.level2_units),
            (Some(2), 2, 1)
        );
        assert_eq!(t.founder_turns, 0);
        let unit = state
            .units
            .values_mut()
            .find(|u| u.owner == rome && data::unit_class(u.kind) == Some(UnitClass::Melee))
            .unwrap();
        unit.xp = 1000;
        o.observe_resolved_turn(4, &state);
        let t = &o.finish()[&1];
        assert_eq!((t.founder_turn, t.founder_turns), (Some(4), 1));
        assert_eq!(t.founder_locked_turns, 0, "not in a captured city");
        // At the start no rival land reaches ring 2, so no Colonia site exists yet.
        assert_eq!(t.no_site_turns, 1);
    }
}
