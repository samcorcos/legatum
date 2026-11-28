//! Observational city-assault cohorts; never used to choose orders.
use legatum_core::{CityId, Event, GameState, PlayerId, TeamId, UnitId, events::Attacker};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub struct Assault {
    city: CityId,
    attacker_team: TeamId,
    start: u32,
    end: Option<u32>,
    // Earliest planning-view sighting among the opening turn's actual attackers.
    first_seen: Option<u32>,
    unseen_attacker: bool,
    left_censored: bool,
    outcome: &'static str,
}

impl Assault {
    fn cohort(&self) -> &'static str {
        if self.left_censored {
            "left_censored"
        } else if self.unseen_attacker {
            "unseen_at_start"
        } else if self.first_seen.is_some_and(|t| self.start - t <= 2) {
            "visible_within_two"
        } else {
            "visible_longer"
        }
    }
}

#[derive(Default)]
pub struct Visibility {
    first_seen: BTreeMap<(TeamId, UnitId), u32>,
    observed_from: Option<u32>,
    active: BTreeMap<(CityId, TeamId), Assault>,
    completed: Vec<Assault>,
    report_from: u32,
    losses_without_assault: Vec<(u32, CityId, TeamId)>,
}

impl Visibility {
    pub fn observe(&mut self, view: &legatum_core::view::PlayerView) {
        self.observed_from.get_or_insert(view.turn);
        for u in view
            .units
            .iter()
            .filter(|u| u.military && !u.owner.is_barbarian())
        {
            let team = view.team_of(u.owner);
            if team != view.team {
                self.first_seen.entry((team, u.id)).or_insert(view.turn);
            }
        }
    }

    fn attack(&mut self, city: CityId, team: TeamId, unit: UnitId, turn: u32) {
        let seen = self.first_seen.get(&(team, unit)).copied();
        let assault = self.active.entry((city, team)).or_insert(Assault {
            city,
            attacker_team: team,
            start: turn,
            end: None,
            first_seen: None,
            unseen_attacker: false,
            left_censored: false,
            outcome: "ongoing",
        });
        // Later reinforcements/replacements must not reset the opening warning window.
        if assault.start == turn {
            assault.unseen_attacker |= seen.is_none();
            if let Some(t) = seen {
                assault.first_seen = Some(assault.first_seen.map_or(t, |old| old.min(t)));
                assault.left_censored |= self
                    .observed_from
                    .is_some_and(|first| first > 1 && t == first);
            }
        }
    }

    fn close(&mut self, key: (CityId, TeamId), turn: u32, outcome: &'static str) {
        if let Some(mut assault) = self.active.remove(&key) {
            assault.end = Some(turn);
            assault.outcome = outcome;
            self.completed.push(assault);
        }
    }

    fn loss(&mut self, city: CityId, captor: Option<TeamId>, turn: u32) {
        if let Some(team) = captor
            && !self.active.contains_key(&(city, team))
        {
            self.losses_without_assault.push((turn, city, team));
        }
        let keys: Vec<_> = self
            .active
            .keys()
            .copied()
            .filter(|(id, _)| *id == city)
            .collect();
        for key in keys {
            self.close(
                key,
                turn,
                if captor == Some(key.1) {
                    "lost_to_attacker"
                } else {
                    "other_ownership_end"
                },
            );
        }
    }

    pub fn resolve(
        &mut self,
        before: &GameState,
        after: &GameState,
        events: &[Event],
        seat: PlayerId,
        min_turn: u32,
    ) {
        let turn = before.turn;
        self.report_from = min_turn;
        // Follow ownership in event order: hits after capture belong to the new owner.
        let mut own: BTreeMap<_, _> = before.cities_of(seat).map(|c| (c.pos, c.id)).collect();
        for event in events {
            match event {
                Event::Battle {
                    attacker: Attacker::Unit(unit),
                    attacker_owner,
                    at,
                    ..
                } if !attacker_owner.is_barbarian()
                    && before.team_of(*attacker_owner) != before.team_of(seat) =>
                {
                    if let Some(city) = own.get(at) {
                        self.attack(*city, before.team_of(*attacker_owner), *unit, turn);
                    }
                }
                Event::CityCaptured { city, from, to, .. } => {
                    if *from == seat {
                        self.loss(*city, Some(before.team_of(*to)), turn);
                        own.retain(|_, id| id != city);
                    }
                    if *to == seat
                        && let Some(c) = before.cities.get(city).or_else(|| after.cities.get(city))
                    {
                        own.insert(c.pos, *city);
                    }
                }
                Event::CityRazed {
                    city,
                    from,
                    by,
                    ordered,
                    ..
                } if *from == seat => {
                    self.loss(*city, (!ordered).then(|| before.team_of(*by)), turn);
                    own.retain(|_, id| id != city);
                }
                _ => {}
            }
        }
        let closed: Vec<_> = self
            .active
            .keys()
            .copied()
            .filter(|(city, team)| {
                !after.cities.get(city).is_some_and(|c| c.owner == seat)
                    || !after.teams_at_war(after.team_of(seat), *team)
            })
            .collect();
        for key in closed {
            let outcome = if after.cities.get(&key.0).is_some_and(|c| c.owner == seat) {
                "peace"
            } else {
                "other_ownership_end"
            };
            self.close(key, turn, outcome);
        }
    }

    pub fn report(mut self, end_turn: u32) -> serde_json::Value {
        for (_, mut assault) in self.active {
            assault.end = Some(end_turn);
            assault.outcome = "right_censored";
            self.completed.push(assault);
        }
        let episodes: Vec<_> = self
            .completed
            .into_iter()
            .filter(|a| a.start >= self.report_from)
            .map(|a| {
                let cohort = a.cohort();
                let mut value = serde_json::to_value(a).expect("assault JSON");
                value["cohort"] = cohort.into();
                value
            })
            .collect();
        serde_json::json!({
            "version": 1,
            "from_turn": self.report_from,
            "episodes": episodes,
            // A capture need not emit Battle (e.g. an undefended city). Do not silently
            // count it as safe or invent an attacking unit's visibility from the captor.
            "losses_without_assault": self.losses_without_assault.into_iter()
                .filter(|(turn, _, _)| *turn >= self.report_from).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker(seen: &[(u32, u32)]) -> Visibility {
        Visibility {
            observed_from: Some(1),
            first_seen: seen
                .iter()
                .map(|(id, turn)| ((TeamId(1), UnitId(*id)), *turn))
                .collect(),
            ..Visibility::default()
        }
    }

    #[test]
    fn event_adapter_records_empty_capture_and_ignores_hits_after_ownership_changes() {
        use legatum_core::{Faction, GameSetup, events::Defender};
        let mut before =
            legatum_core::new_game(&GameSetup::standard(7, [Faction::Athens, Faction::Sparta]));
        before.turn = 122;
        let city = before.cities_of(PlayerId(0)).next().unwrap();
        let (id, at) = (city.id, city.pos);
        let mut after = before.clone();
        after.cities.get_mut(&id).unwrap().owner = PlayerId(2);
        let capture = Event::CityCaptured {
            city: id,
            from: PlayerId(0),
            to: PlayerId(2),
            bounty: None,
        };
        let hit = Event::Battle {
            attacker: UnitId(999).into(),
            attacker_owner: PlayerId(3),
            defender: Defender::City {
                city: id,
                owner: PlayerId(2),
            },
            at,
            attacker_at: None,
            attacker_strength: Default::default(),
            defender_strength: Default::default(),
            damage_to_attacker: 0,
            damage_to_defender: 1,
            attacker_roll: 100,
            defender_roll: 100,
            attacker_hp_after: 100,
            defender_hp_after: 99,
            ranged: true,
            air_strike: false,
            meeting: false,
            defender_leaving: false,
        };
        let mut v = Visibility::default();
        v.resolve(&before, &after, &[capture, hit], PlayerId(0), 120);
        let report = v.report(123);
        assert!(report["episodes"].as_array().unwrap().is_empty());
        assert_eq!(
            report["losses_without_assault"].as_array().unwrap().len(),
            1
        );
    }

    #[test]
    fn two_turn_boundary_and_unseen_are_distinct() {
        for (seen, expected) in [
            (Some(120), "visible_within_two"),
            (Some(119), "visible_longer"),
            (None, "unseen_at_start"),
        ] {
            let mut v = tracker(&seen.map(|t| vec![(1, t)]).unwrap_or_default());
            v.attack(CityId(1), TeamId(1), UnitId(1), 122);
            assert_eq!(v.active[&(CityId(1), TeamId(1))].cohort(), expected);
        }
    }

    #[test]
    fn opening_army_uses_oldest_attacker_not_new_replacement() {
        let mut v = tracker(&[(1, 100), (2, 122), (3, 130)]);
        v.attack(CityId(1), TeamId(1), UnitId(2), 122);
        v.attack(CityId(1), TeamId(1), UnitId(1), 122);
        v.attack(CityId(1), TeamId(1), UnitId(3), 130);
        assert_eq!(v.active.len(), 1);
        assert_eq!(v.active[&(CityId(1), TeamId(1))].cohort(), "visible_longer");
    }

    #[test]
    fn unseen_opening_participant_prevents_observed_cohort() {
        let mut v = tracker(&[(1, 121)]);
        v.attack(CityId(1), TeamId(1), UnitId(1), 122);
        v.attack(CityId(1), TeamId(1), UnitId(2), 122);
        assert_eq!(
            v.active[&(CityId(1), TeamId(1))].cohort(),
            "unseen_at_start"
        );
    }

    #[test]
    fn competing_capture_and_recapture_do_not_credit_old_assault() {
        let mut v = tracker(&[(1, 121)]);
        v.attack(CityId(1), TeamId(1), UnitId(1), 122);
        v.loss(CityId(1), Some(TeamId(2)), 124);
        assert_eq!(v.completed[0].outcome, "other_ownership_end");
        v.attack(CityId(1), TeamId(1), UnitId(1), 140);
        v.loss(CityId(1), Some(TeamId(1)), 142);
        assert_eq!(v.completed.len(), 2);
        assert_eq!(v.completed[1].start, 140);
        assert_eq!(v.completed[1].cohort(), "visible_longer");
        assert_eq!(v.completed[1].outcome, "lost_to_attacker");
    }

    #[test]
    fn unopposed_capture_is_reported_not_invented_as_a_visible_attack() {
        let mut v = tracker(&[]);
        v.loss(CityId(1), Some(TeamId(1)), 123);
        let report = v.report(130);
        assert!(report["episodes"].as_array().unwrap().is_empty());
        assert_eq!(
            report["losses_without_assault"],
            serde_json::json!([[123, 1, 1]])
        );
    }

    #[test]
    fn pre_window_assault_and_own_raze_are_not_new_qualifying_losses() {
        let mut v = tracker(&[(1, 119)]);
        v.report_from = 120;
        v.attack(CityId(1), TeamId(1), UnitId(1), 119);
        v.attack(CityId(1), TeamId(1), UnitId(1), 120);
        v.loss(CityId(1), None, 123);
        let report = v.report(130);
        assert!(report["episodes"].as_array().unwrap().is_empty());
        assert!(
            report["losses_without_assault"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn incomplete_observation_and_unfinished_siege_are_censored() {
        let mut v = tracker(&[(1, 120)]);
        v.observed_from = Some(120);
        v.attack(CityId(1), TeamId(1), UnitId(1), 121);
        let report = v.report(130);
        assert_eq!(report["episodes"][0]["cohort"], "left_censored");
        assert_eq!(report["episodes"][0]["outcome"], "right_censored");
    }
}
