//! The game's record: one integer sample per empire per resolved turn plus a short chronicle of
//! landmark events. It feeds the post-game report and is public only once the game is finished
//! (`PlayerView::history`). Deterministic and integer-only; written by `record` at the end of
//! `rules::resolve_turn`.

use crate::events::Event;
use crate::state::GameState;
use crate::types::*;
use crate::wonders::Wonder;
use crate::{objectives, scoring};
use serde::{Deserialize, Serialize};

/// The game's public record, one entry per empire per resolved turn plus landmark events.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameHistory {
    /// The turn the first entry of every series describes (1 for a game recorded from its start;
    /// later for an imported partial record). Entry i of a series is the state after
    /// turn `first_turn + i` resolved.
    pub first_turn: u32,
    /// One record per player, indexed by player id (barbarians excluded).
    pub empires: Vec<EmpireHistory>,
    /// Landmark events in the order they happened.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chronicle: Vec<Chronicle>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireHistory {
    /// `scoring::vp_total`.
    pub vp: Vec<u32>,
    pub cities: Vec<u32>,
    /// Sum of city populations.
    pub population: Vec<i32>,
    /// Science produced this turn (`Player::science_history`, last entry).
    pub science: Vec<i32>,
    /// Treasury after the turn.
    pub gold: Vec<i32>,
    /// `scoring::army_strength`.
    pub army: Vec<i32>,
    pub techs: Vec<u32>,
    /// 1-based era (`objectives::era_of`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub era: Vec<u8>,
}

/// One landmark event. Internally tagged by "kind"; every variant has the `turn` it happened.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Chronicle {
    Founded {
        turn: u32,
        player: PlayerId,
        city: String,
    },
    Captured {
        turn: u32,
        by: PlayerId,
        from: PlayerId,
        city: String,
    },
    Razed {
        turn: u32,
        by: PlayerId,
        from: PlayerId,
        city: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        ordered: bool,
    },
    War {
        turn: u32,
        a: TeamId,
        b: TeamId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        by: Option<TeamId>,
    },
    Peace {
        turn: u32,
        a: TeamId,
        b: TeamId,
    },
    Era {
        turn: u32,
        player: PlayerId,
        era: u8,
        first: bool,
    },
    GreatWork {
        turn: u32,
        player: PlayerId,
        building: BuildingKind,
        city: String,
    },
    Wonder {
        turn: u32,
        player: PlayerId,
        wonder: Wonder,
    },
    Objective {
        turn: u32,
        player: PlayerId,
        era: u8,
        objective: String,
    },
    Goal {
        turn: u32,
        player: PlayerId,
    },
    Eliminated {
        turn: u32,
        player: PlayerId,
    },
    HoldStarted {
        turn: u32,
        team: TeamId,
    },
    HoldBroken {
        turn: u32,
        team: TeamId,
    },
    Won {
        turn: u32,
        team: TeamId,
        reason: String,
    },
    Drawn {
        turn: u32,
    },
}

impl GameHistory {
    /// True before the first turn resolves.
    pub fn is_empty(&self) -> bool {
        self.empires.is_empty() && self.chronicle.is_empty()
    }

    /// The number of turns recorded (the shortest series).
    pub fn turns(&self) -> usize {
        self.empires.iter().map(|e| e.vp.len()).min().unwrap_or(0)
    }
}

/// Append this turn's sample for every empire and the landmark events in `events`. `prev` is the
/// state the turn started from, `s` the state it resolved to (its `turn` is still the turn being
/// resolved). History that no longer lines up with the turn (a mismatched series length or a
/// different empire count) restarts at this turn instead of failing.
pub fn record(prev: &GameState, s: &mut GameState, events: &[Event]) {
    let turn = s.turn;
    let h = &s.history;
    let aligned = !h.empires.is_empty()
        && h.empires.len() == s.players.len()
        && h.first_turn <= turn
        && h.empires.iter().all(|e| {
            let n = e.vp.len();
            n as u32 == turn - h.first_turn
                && [
                    e.cities.len(),
                    e.population.len(),
                    e.science.len(),
                    e.gold.len(),
                    e.army.len(),
                    e.techs.len(),
                ]
                .iter()
                .all(|l| *l == n)
                && (e.era.is_empty() || e.era.len() == n)
        });
    let mut history = if aligned {
        std::mem::take(&mut s.history)
    } else {
        GameHistory {
            first_turn: turn,
            empires: vec![EmpireHistory::default(); s.players.len()],
            chronicle: Vec::new(),
        }
    };
    for (i, e) in history.empires.iter_mut().enumerate() {
        let p = s.players[i].id;
        let (cities, population) = s
            .cities_of(p)
            .fold((0u32, 0i32), |(n, pop), c| (n + 1, pop + c.population));
        e.vp.push(scoring::vp_total(s, p));
        e.cities.push(cities);
        e.population.push(population);
        e.science
            .push(s.players[i].science_history.last().copied().unwrap_or(0));
        e.gold.push(s.players[i].gold);
        e.army.push(scoring::army_strength(s, p));
        e.techs.push(s.players[i].techs.len() as u32);
        e.era.push(objectives::era_of(s, p));
    }
    for event in events {
        if let Some(entry) = chronicle_entry(turn, prev, s, event) {
            history.chronicle.push(entry);
        }
    }
    s.history = history;
}

fn chronicle_entry(turn: u32, prev: &GameState, s: &GameState, event: &Event) -> Option<Chronicle> {
    let empire = |p: PlayerId| !p.is_barbarian();
    match event {
        Event::CityFounded { owner, name, .. } if empire(*owner) => Some(Chronicle::Founded {
            turn,
            player: *owner,
            city: name.clone(),
        }),
        Event::CityCaptured { city, from, to, .. } => {
            let name = s.cities.get(city).or_else(|| prev.cities.get(city))?;
            Some(Chronicle::Captured {
                turn,
                by: *to,
                from: *from,
                city: name.name.clone(),
            })
        }
        Event::CityRazed {
            city,
            from,
            by,
            ordered,
            ..
        } => {
            let name = prev.cities.get(city).or_else(|| s.cities.get(city))?;
            Some(Chronicle::Razed {
                turn,
                by: *by,
                from: *from,
                city: name.name.clone(),
                ordered: *ordered,
            })
        }
        Event::WarDeclared { relation } | Event::WarJoined { relation, .. }
            if relation.a != TeamId::BARBARIAN =>
        {
            Some(Chronicle::War {
                turn,
                a: relation.a,
                b: relation.b,
                by: match relation.declarers.as_slice() {
                    [only] => Some(*only),
                    _ => None,
                },
            })
        }
        Event::PeaceEstablished { relation } if relation.a != TeamId::BARBARIAN => {
            Some(Chronicle::Peace {
                turn,
                a: relation.a,
                b: relation.b,
            })
        }
        Event::EraEntered { player, era, first } if empire(*player) => Some(Chronicle::Era {
            turn,
            player: *player,
            era: *era,
            first: *first,
        }),
        Event::ProductionCompleted {
            city,
            owner,
            item: ProductionItem::Building { building } | ProductionItem::Wonder { building, .. },
        } if empire(*owner) && crate::data::is_great_work(*building) => {
            let name = s.cities.get(city).or_else(|| prev.cities.get(city))?;
            Some(Chronicle::GreatWork {
                turn,
                player: *owner,
                building: *building,
                city: name.name.clone(),
            })
        }
        Event::WonderDiscovered { owner, wonder, .. } if empire(*owner) => {
            Some(Chronicle::Wonder {
                turn,
                player: *owner,
                wonder: *wonder,
            })
        }
        Event::EraObjectiveScored {
            player,
            era,
            objective,
            ..
        } if empire(*player) => Some(Chronicle::Objective {
            turn,
            player: *player,
            era: *era,
            objective: objective.clone(),
        }),
        Event::FactionGoalCompleted {
            player,
            again: false,
            ..
        } if empire(*player) => Some(Chronicle::Goal {
            turn,
            player: *player,
        }),
        Event::PlayerEliminated { player } if empire(*player) => Some(Chronicle::Eliminated {
            turn,
            player: *player,
        }),
        Event::VictoryHoldStarted { team, .. } => {
            Some(Chronicle::HoldStarted { turn, team: *team })
        }
        Event::VictoryHoldBroken { team, .. } => Some(Chronicle::HoldBroken { turn, team: *team }),
        Event::GameWon { team, reason } => Some(Chronicle::Won {
            turn,
            team: *team,
            reason: reason.clone(),
        }),
        Event::GameDrawn { .. } => Some(Chronicle::Drawn { turn }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::test_support::*;
    use crate::{Order, PlayerId, ViewDetail, player_view_detail, public_view, resolve_turn};
    use std::collections::BTreeMap;

    fn found_both(s: &mut GameState) -> BTreeMap<PlayerId, Vec<Order>> {
        let a = unit(s, 0, UnitKind::Settler, hex(2, 2));
        let b = unit(s, 1, UnitKind::Settler, hex(9, 7));
        BTreeMap::from([
            (PlayerId(0), vec![Order::FoundCity { unit: a }]),
            (PlayerId(1), vec![Order::FoundCity { unit: b }]),
        ])
    }

    fn play(turns: u32) -> GameState {
        let mut s = game(&[0, 1]);
        let first = found_both(&mut s);
        let (next, _) = resolve_turn(&s, &first);
        s = next;
        for _ in 1..turns {
            s = resolve_turn(&s, &BTreeMap::new()).0;
        }
        s
    }

    #[test]
    fn records_one_entry_per_empire_per_turn_and_chronicles_foundings() {
        let s = play(4);
        let h = &s.history;
        assert_eq!(h.first_turn, 1);
        assert_eq!(h.empires.len(), 2);
        for e in &h.empires {
            for len in [
                e.vp.len(),
                e.cities.len(),
                e.population.len(),
                e.science.len(),
                e.gold.len(),
                e.army.len(),
                e.techs.len(),
                e.era.len(),
            ] {
                assert_eq!(len, 4);
            }
            assert_eq!(e.cities, vec![1, 1, 1, 1]);
            assert!(e.population.iter().all(|p| *p >= 1));
            assert!(e.era.iter().all(|era| *era == 1));
        }
        assert_eq!(h.turns(), 4);
        assert_eq!(s.turn, 5);
        for p in [PlayerId(0), PlayerId(1)] {
            let city = s.cities_of(p).next().expect("city founded").name.clone();
            assert!(
                h.chronicle.iter().any(|c| matches!(c,
                    Chronicle::Founded { turn: 1, player, city: name } if *player == p && *name == city)),
                "no Founded entry for {p:?} ({city})"
            );
        }
    }

    #[test]
    fn deterministic_and_round_trips_through_json() {
        let (a, b) = (play(3), play(3));
        assert_eq!(a.history, b.history);
        let json = serde_json::to_string(&a).unwrap();
        let back: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.history, a.history);
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }

    #[test]
    fn old_saves_without_history_load_and_stay_byte_compatible() {
        let s = game(&[0, 1]);
        let json = serde_json::to_string(&s).unwrap();
        assert!(
            !json.contains("\"history\""),
            "an empty history is not serialized"
        );
        let back: GameState = serde_json::from_str(&json).unwrap();
        assert!(back.history.is_empty());
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
        let played = serde_json::to_string(&play(2)).unwrap();
        assert!(played.contains("\"history\""));
    }

    #[test]
    fn view_carries_history_only_when_finished_and_full() {
        let mut s = play(3);
        let public = public_view(&s);
        let full =
            |s: &GameState| player_view_detail(s, PlayerId(0), &public_view(s), ViewDetail::Full);
        assert!(
            full(&s).history.is_none(),
            "rivals' numbers stay hidden during play"
        );
        s.finished = true;
        assert_eq!(full(&s).history.as_ref(), Some(&s.history));
        for detail in [ViewDetail::Decisions, ViewDetail::Map] {
            assert!(
                player_view_detail(&s, PlayerId(0), &public, detail)
                    .history
                    .is_none()
            );
        }
        let json = serde_json::to_string(&full(&s)).unwrap();
        assert!(json.contains("\"history\""));
        s.finished = false;
        assert!(
            !serde_json::to_string(&full(&s))
                .unwrap()
                .contains("\"history\"")
        );
    }

    #[test]
    fn inconsistent_history_restarts_at_this_turn() {
        let mut s = play(3);
        s.history.empires[1].gold.pop();
        let (next, _) = resolve_turn(&s, &BTreeMap::new());
        assert_eq!(next.history.first_turn, s.turn);
        assert_eq!(next.history.turns(), 1);
        assert!(next.history.chronicle.is_empty());

        let mut s = play(3);
        s.history.empires.pop();
        let (next, _) = resolve_turn(&s, &BTreeMap::new());
        assert_eq!(next.history.empires.len(), 2);
        assert_eq!(next.history.first_turn, s.turn);

        // A game saved before history existed starts recording at the turn it resumes.
        let mut s = game(&[0, 1]);
        s.turn = 40;
        let (next, _) = resolve_turn(&s, &BTreeMap::new());
        assert_eq!(next.history.first_turn, 40);
        assert_eq!(next.history.turns(), 1);
    }

    #[test]
    fn maps_events_to_chronicle_entries() {
        let s = game(&[0, 1]);
        let (a, b) = (PlayerId(0), PlayerId(1));
        let mut relation = s.diplomacy.as_ref().unwrap().relations[0].clone();
        relation.declarers = vec![TeamId(1)];
        let prev = s.clone();
        let entry = |event: Event| chronicle_entry(7, &prev, &s, &event);
        assert_eq!(
            entry(Event::WarDeclared {
                relation: relation.clone()
            }),
            Some(Chronicle::War {
                turn: 7,
                a: relation.a,
                b: relation.b,
                by: Some(TeamId(1))
            })
        );
        relation.declarers = vec![relation.a, relation.b];
        assert!(matches!(
            entry(Event::WarDeclared {
                relation: relation.clone()
            }),
            Some(Chronicle::War { by: None, .. })
        ));
        assert_eq!(
            entry(Event::GameWon {
                team: TeamId(1),
                reason: "vp".into()
            }),
            Some(Chronicle::Won {
                turn: 7,
                team: TeamId(1),
                reason: "vp".into()
            })
        );
        assert_eq!(
            entry(Event::PlayerEliminated { player: b }),
            Some(Chronicle::Eliminated { turn: 7, player: b })
        );
        assert_eq!(
            entry(Event::PlayerEliminated {
                player: PlayerId::BARBARIAN
            }),
            None
        );
        assert_eq!(
            entry(Event::FactionGoalCompleted {
                player: a,
                faction: crate::Faction::Switzerland,
                vp: 2,
                again: true
            }),
            None
        );
        assert!(matches!(
            entry(Event::FactionGoalCompleted {
                player: a,
                faction: crate::Faction::Switzerland,
                vp: 2,
                again: false
            }),
            Some(Chronicle::Goal { turn: 7, .. })
        ));
    }

    #[test]
    fn json_shape_is_the_ui_contract() {
        let h = GameHistory {
            first_turn: 1,
            empires: vec![EmpireHistory {
                vp: vec![0, 0],
                cities: vec![1, 1],
                population: vec![1, 1],
                science: vec![2, 3],
                gold: vec![64, 69],
                army: vec![25, 25],
                techs: vec![0, 0],
                era: vec![1, 1],
            }],
            chronicle: vec![
                Chronicle::Founded {
                    turn: 1,
                    player: PlayerId(0),
                    city: "Bern".into(),
                },
                Chronicle::War {
                    turn: 40,
                    a: TeamId(0),
                    b: TeamId(3),
                    by: Some(TeamId(3)),
                },
                Chronicle::Peace {
                    turn: 50,
                    a: TeamId(0),
                    b: TeamId(3),
                },
            ],
        };
        assert_eq!(
            serde_json::to_string(&h).unwrap(),
            r#"{"first_turn":1,"empires":[{"vp":[0,0],"cities":[1,1],"population":[1,1],"science":[2,3],"gold":[64,69],"army":[25,25],"techs":[0,0],"era":[1,1]}],"chronicle":[{"kind":"Founded","turn":1,"player":0,"city":"Bern"},{"kind":"War","turn":40,"a":0,"b":3,"by":3},{"kind":"Peace","turn":50,"a":0,"b":3}]}"#
        );
        let no_by: Chronicle =
            serde_json::from_str(r#"{"kind":"War","turn":3,"a":0,"b":1}"#).unwrap();
        assert!(matches!(no_by, Chronicle::War { by: None, .. }));
    }
}
