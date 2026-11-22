//! Observer-only diagnostics. No observations are fed back into AI planning or game state.
use legatum_ai::openings::CoalitionMarch;
use legatum_core::{CityId, Event, GameState, PlayerId, TeamId, UnitId, UnitKind, data};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Counters {
    pub settlers_captured_by: u32,
    pub settlers_lost: u32,
    /// Captured in the same resolution as the captor's surprise declaration.
    pub settlers_lost_to_surprise_war: u32,
    pub escorted_settler_turns: u32,
    pub unescorted_settler_turns: u32,
    pub scouts_alive_by_turn: BTreeMap<u32, u32>,
    pub gold_spent: i64,
    pub gold_hoarded_peak: i32,
    pub wars_with_battles: u32,
    pub coalition_marches: u32,
    pub sieges_abandoned: u32,
}

#[derive(Default)]
pub struct Observer {
    /// Current declared war against each opponent; true once its first battle is counted.
    wars: BTreeMap<TeamId, bool>,
    sieges: BTreeMap<CityId, Vec<UnitId>>,
}

fn besiegers(s: &GameState, seat: PlayerId, city: CityId) -> Vec<UnitId> {
    let Some(c) = s.cities.get(&city) else {
        return Vec::new();
    };
    if !s.at_war(seat, c.owner) {
        return Vec::new();
    }
    s.units_of(seat)
        .filter(|u| {
            let stats = data::unit_stats_for(s.player(seat).faction, u.kind);
            stats.military
                && u.kind != UnitKind::Scout
                && u.pos.distance(c.pos) <= stats.range.max(1)
        })
        .map(|u| u.id)
        .collect()
}

impl Observer {
    pub fn before(&mut self, c: &mut Counters, s: &GameState, seat: PlayerId) {
        c.gold_hoarded_peak = c.gold_hoarded_peak.max(s.player(seat).gold);
        for settler in s.units_of(seat).filter(|u| u.kind == UnitKind::Settler) {
            if s.map
                .get(settler.pos)
                .is_some_and(|t| t.owner == Some(seat))
            {
                continue;
            }
            let escorted = s.units.values().any(|u| {
                s.team_of(u.owner) == s.team_of(seat)
                    && data::unit_stats(u.kind).military
                    && u.kind != UnitKind::Scout
                    && u.pos.distance(settler.pos) <= 1
            });
            c.escorted_settler_turns += u32::from(escorted);
            c.unescorted_settler_turns += u32::from(!escorted);
        }
        self.sieges = s
            .cities
            .values()
            .filter(|city| city.hp < data::CITY_HP)
            .filter_map(|city| {
                let units = besiegers(s, seat, city.id);
                (!units.is_empty()).then_some((city.id, units))
            })
            .collect();
    }

    pub fn after(
        &mut self,
        c: &mut Counters,
        s: &GameState,
        seat: PlayerId,
        turn: u32,
        events: &[Event],
        march: Option<&CoalitionMarch>,
    ) {
        let team = s.team_of(seat);
        c.scouts_alive_by_turn.insert(
            turn,
            s.units_of(seat)
                .filter(|u| u.kind == UnitKind::Scout)
                .count() as u32,
        );
        c.gold_hoarded_peak = c.gold_hoarded_peak.max(s.player(seat).gold);
        let surprise_attackers: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::WarDeclared { relation } => Some(&relation.declarers),
                _ => None,
            })
            .flatten()
            .copied()
            .collect();
        for event in events {
            match event {
                Event::SettlerCaptured { from, to, .. } => {
                    c.settlers_captured_by += u32::from(*to == seat);
                    c.settlers_lost += u32::from(*from == seat);
                    c.settlers_lost_to_surprise_war +=
                        u32::from(*from == seat && surprise_attackers.contains(&s.team_of(*to)));
                }
                Event::ItemBought { owner, gold, .. } | Event::UnitUpgraded { owner, gold, .. }
                    if *owner == seat =>
                {
                    c.gold_spent += i64::from(*gold);
                }
                Event::WarDeclared { relation } if relation.declarers.contains(&team) => {
                    self.wars.insert(
                        if relation.a == team {
                            relation.b
                        } else {
                            relation.a
                        },
                        false,
                    );
                }
                Event::PeaceEstablished { relation }
                    if relation.a == team || relation.b == team =>
                {
                    self.wars.remove(&if relation.a == team {
                        relation.b
                    } else {
                        relation.a
                    });
                }
                Event::Battle {
                    attacker_owner,
                    defender,
                    ..
                } => {
                    let defender_owner = match defender {
                        legatum_core::events::Defender::Unit { owner, .. }
                        | legatum_core::events::Defender::City { owner, .. } => *owner,
                    };
                    let a = s.team_of(*attacker_owner);
                    let b = s.team_of(defender_owner);
                    if a == team || b == team {
                        let other = if a == team { b } else { a };
                        if let Some(fought) = self.wars.get_mut(&other)
                            && !*fought
                        {
                            *fought = true;
                            c.wars_with_battles += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(m) = march {
            let advanced = m
                .units
                .iter()
                .filter(|(id, from)| {
                    s.units.get(id).is_some_and(|u| {
                        u.owner == seat && u.pos.distance(m.target) < from.distance(m.target)
                    })
                })
                .count();
            c.coalition_marches += u32::from(advanced >= m.minimum);
        }
        for (city, units) in &self.sieges {
            // A capture, destroyed city or dead assault force is not a voluntary withdrawal.
            if s.cities
                .get(city)
                .is_some_and(|c| s.team_of(c.owner) != team)
                && !events
                    .iter()
                    .any(|e| matches!(e, Event::CityCaptured { city: id, .. } if id == city))
                && units
                    .iter()
                    .any(|id| s.units.get(id).is_some_and(|u| u.owner == seat))
                && besiegers(s, seat, *city).is_empty()
            {
                c.sieges_abandoned += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests;
