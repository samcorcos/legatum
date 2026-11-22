//! Opening-league configuration and omniscient report collection (never passed to brains).
use legatum_ai::openings::{Opening, OpeningPolicy, PhaseSwitch};
use legatum_core::{Event, GameState, PlayerId, UnitKind, data, events::Defender};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct Config {
    pub ai_primitives: bool,
    pub seats: [Opening; 8],
    pub policies: Option<&'static BTreeMap<Opening, OpeningPolicy>>,
    /// Hard seats that play the scripted Hard even when Hard v2 is on (`--legacy-hard`).
    pub legacy_hard: [bool; 8],
    /// Hard seats that play Hard v2 (set per game by `--hard-v2-rotation`; `--hard-v2`
    /// turns it on for every Hard seat through the `hard-v2-ai` setup switch instead).
    pub hard_v2: [bool; 8],
    /// One Hard v2 seat per game, seat `index % 8`; every other seat is scripted Hard.
    pub hard_v2_rotation: bool,
    /// All eligible Hard v2 seats hand off to scripted Hard after their opening.
    pub hard_v2_handoff: bool,
    /// One seat per game (`index % 8`) plays this opening with its Hard v2 tuned genome
    /// (`--challenger`); the other seats keep the lobby brain (Hard v2, or legacy Hard
    /// with `--legacy-hard`).
    pub challenger: Option<Opening>,
}
impl Config {
    pub fn policy(&self, seat: usize) -> OpeningPolicy {
        self.policies
            .and_then(|ps| ps.get(&self.seats[seat]))
            .cloned()
            .unwrap_or_else(|| match self.challenger {
                Some(_) => legatum_ai::hard_v2::policy(self.seats[seat]),
                None => OpeningPolicy::builtin(self.seats[seat]),
            })
    }
    /// Any seat-policy option is set, so the runner builds brains itself.
    pub fn enabled(&self) -> bool {
        self.league()
            || self.legacy_hard.contains(&true)
            || self.hard_v2.contains(&true)
            || self.hard_v2_handoff
            || self.challenger.is_some()
    }
    /// An opening-league run (named seat openings or primitives on Default seats); its
    /// Default seats are legacy Hard. A challenger run is not a league run.
    pub fn league(&self) -> bool {
        self.challenger.is_none()
            && (self.ai_primitives || self.seats.iter().any(|o| *o != Opening::Default))
    }
}

/// `--legacy-hard` values: `all` or zero-based seat numbers.
pub fn legacy_seats(args: &[String]) -> Result<[bool; 8], String> {
    let mut seats = [false; 8];
    for arg in args {
        if arg == "all" {
            seats = [true; 8];
        } else {
            let seat: usize = arg.parse().map_err(|_| format!("invalid seat: {arg}"))?;
            *seats
                .get_mut(seat)
                .ok_or_else(|| format!("seat {seat} outside 0..7"))? = true;
        }
    }
    Ok(seats)
}

pub fn seats(args: &[String]) -> Result<[Opening; 8], String> {
    let mut seats = [Opening::Default; 8];
    // Process left to right, so `all=X` followed by a specific seat is an override.
    for arg in args {
        let (seat, opening) = arg
            .split_once('=')
            .ok_or_else(|| format!("expected <seat|all>=<Opening>: {arg}"))?;
        let opening = opening.parse()?;
        if seat == "all" {
            seats.fill(opening);
        } else {
            let seat: usize = seat.parse().map_err(|_| format!("invalid seat: {seat}"))?;
            *seats
                .get_mut(seat)
                .ok_or_else(|| format!("seat {seat} outside 0..7"))? = opening;
        }
    }
    Ok(seats)
}

pub fn genomes(args: &[String]) -> Result<&'static BTreeMap<Opening, OpeningPolicy>, String> {
    let mut policies = BTreeMap::new();
    for arg in args {
        let (name, path) = arg
            .split_once('=')
            .ok_or_else(|| format!("expected Opening=path.json: {arg}"))?;
        let opening = name.parse()?;
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let genome: Vec<i32> = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
        let policy =
            OpeningPolicy::from_genome(opening, &genome).map_err(|e| format!("{path}: {e}"))?;
        if policies.insert(opening, policy).is_some() {
            return Err(format!("duplicate genome for {name}"));
        }
    }
    Ok(Box::leak(Box::new(policies)))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeatRecord {
    pub ai_primitives: bool,
    #[serde(flatten)]
    pub primitives: crate::primitive_telemetry::Counters,
    pub opening: Opening,
    /// Competition rank by final individual VP; tied VP has tied rank.
    pub rank: u32,
    pub won: bool,
    /// After resolving these literal turns. Null = game ended earlier.
    pub cities_at_turns: BTreeMap<String, Option<u32>>,
    /// Buildings finished or bought by this seat during the observed game.
    pub buildings_completed: u32,
    /// Roman non-capital foundations observed from city founding events.
    pub roman_foundations: u32,
    /// First resolved military foundation, for the T60 Hard expansion check.
    pub first_roman_foundation_turn: Option<u32>,
    /// First turn a Roman melee unit earned promotion level 1.
    pub first_roman_promotion_turn: Option<u32>,
    /// Turns on which this seat founded cities, including its capital.
    pub city_founding_turns: Vec<u32>,
    pub wars_declared: u32,
    pub cities_captured: u32,
    pub cities_lost: u32,
    /// Cities lost within 30 turns of their founding (placement risk).
    pub young_cities_lost: u32,
    pub phase_switches: Vec<PhaseSwitch>,
    pub strategy_switches: Vec<legatum_ai::opponents::StrategySwitch>,
    /// First resolved completion of Walls or their faction replacement.
    pub first_walls_turn: Option<u32>,
    pub opening_handoff: Option<legatum_ai::openings::OpeningHandoff>,
    /// Duel-only first declaration by this seat against the other player's team.
    pub first_opponent_war_turn: Option<u32>,
    /// Duel-only first non-Scout military adjacency at war, attack, or occupation,
    /// against the other player's city. Barbarian combat and city strikes do not count.
    pub first_opponent_city_pressure_turn: Option<u32>,
    /// Founding turns with another owned city and a friendly non-Scout military
    /// unit within one hex of the new city after resolution (escort or garrison).
    pub guarded_expansion_turns: Vec<u32>,
    /// Threat-driven decisions from the brain (`Brain::threat_log`), copied at game end.
    pub threat_log: Vec<legatum_ai::threat::ThreatEvent>,
    /// Persistent campaign starts and explicit endings from the scripted seat.
    #[serde(default)]
    pub campaign_events: Vec<legatum_ai::CampaignEvent>,
    /// Goal-driven plan switches (each with its reason) and blocked purchases (Hard).
    #[serde(default)]
    pub plan_events: Vec<legatum_ai::plan::PlanEvent>,
    /// Every purchase the plan layer blocked, including those not kept as events.
    #[serde(default)]
    pub blocked_expenses: u32,
    /// Hard+Search rollouts at plan review points (`--search`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_events: Vec<legatum_ai::search::SearchEvent>,
    /// Wall time of this seat's decisions per turn (ms); only in games with a search seat.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_ms: Vec<u32>,
    /// Every resolved Walls (or replacement) completion by this seat.
    pub walls_completed_turns: Vec<u32>,
    /// Agreements that took effect for this seat's team.
    pub treaties: Vec<TreatyRecord>,
    /// Wars this seat's team declared, with what each gained and lost (observer truth).
    pub wars: Vec<WarRecord>,
    /// Turn this seat was eliminated.
    pub eliminated_turn: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatyRecord {
    pub turn: u32,
    pub with: u8,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarRecord {
    pub turn: u32,
    pub against: u8,
    /// Cities this seat took from the enemy team while this war lasted.
    pub captures: u32,
    /// Cities this seat lost to the enemy team while this war lasted.
    pub losses: u32,
    pub ended_turn: Option<u32>,
}
impl Default for SeatRecord {
    fn default() -> Self {
        Self {
            ai_primitives: false,
            primitives: Default::default(),
            opening: Opening::Default,
            rank: 0,
            won: false,
            cities_at_turns: [25, 50, 75, 100]
                .into_iter()
                .map(|t| (t.to_string(), None))
                .collect(),
            buildings_completed: 0,
            roman_foundations: 0,
            first_roman_foundation_turn: None,
            first_roman_promotion_turn: None,
            city_founding_turns: Vec::new(),
            wars_declared: 0,
            cities_captured: 0,
            cities_lost: 0,
            young_cities_lost: 0,
            phase_switches: Vec::new(),
            strategy_switches: Vec::new(),
            first_walls_turn: None,
            opening_handoff: None,
            first_opponent_war_turn: None,
            first_opponent_city_pressure_turn: None,
            guarded_expansion_turns: Vec::new(),
            threat_log: Vec::new(),
            campaign_events: Vec::new(),
            plan_events: Vec::new(),
            blocked_expenses: 0,
            search_events: Vec::new(),
            decision_ms: Vec::new(),
            walls_completed_turns: Vec::new(),
            treaties: Vec::new(),
            wars: Vec::new(),
            eliminated_turn: None,
        }
    }
}
impl SeatRecord {
    pub fn observe(&mut self, seat: PlayerId, turn: u32, state: &GameState, events: &[Event]) {
        let team = state.team_of(seat);
        let opponent = (state.players.len() == 2)
            .then(|| state.players.iter().find(|p| p.team != team).map(|p| p.id))
            .flatten();
        let military = |kind| kind != UnitKind::Scout && data::unit_stats(kind).military;
        if let Some(count) = self.cities_at_turns.get_mut(&turn.to_string()) {
            *count = Some(state.cities_of(seat).count() as u32);
        }
        if state.player(seat).faction == legatum_core::Faction::Rome
            && self.first_roman_promotion_turn.is_none()
            && state.units_of(seat).any(|u| {
                data::unit_class(u.kind) == Some(legatum_core::UnitClass::Melee)
                    && legatum_core::rules::promotion::level_for(legatum_core::Faction::Rome, u.xp)
                        >= 1
            })
        {
            self.first_roman_promotion_turn = Some(turn);
        }
        for event in events {
            if let Event::CityFounded { owner, veteran, .. } = event
                && *owner == seat
            {
                self.city_founding_turns.push(turn);
                if *veteran {
                    self.roman_foundations += 1;
                    self.first_roman_foundation_turn.get_or_insert(turn);
                }
            }
            if matches!(event,
                Event::ProductionCompleted { owner, item: legatum_core::ProductionItem::Building { .. }, .. }
                | Event::ItemBought { owner, item: legatum_core::ProductionItem::Building { .. }, .. }
                if *owner == seat
            ) {
                self.buildings_completed += 1;
            }
            match event {
                Event::ProductionCompleted {
                    owner,
                    item: legatum_core::ProductionItem::Building { building },
                    ..
                } if *owner == seat
                    && data::has_building(
                        &std::collections::BTreeSet::from([*building]),
                        legatum_core::BuildingKind::Walls,
                    ) =>
                {
                    self.first_walls_turn.get_or_insert(turn);
                    self.walls_completed_turns.push(turn);
                }
                Event::TreatyActivated { relation, .. }
                    if (relation.a == team || relation.b == team)
                        && relation.state == legatum_core::diplomacy::RelationState::Alliance =>
                {
                    self.treaties.push(TreatyRecord {
                        turn,
                        with: if relation.a == team {
                            relation.b.0
                        } else {
                            relation.a.0
                        },
                        kind: format!("{:?}", relation.state),
                    });
                }
                Event::PeaceEstablished { relation } => {
                    for war in self.wars.iter_mut().filter(|w| w.ended_turn.is_none()) {
                        let other = legatum_core::TeamId(war.against);
                        if (relation.a == team && relation.b == other)
                            || (relation.b == team && relation.a == other)
                        {
                            war.ended_turn = Some(turn);
                        }
                    }
                }
                Event::PlayerEliminated { player } if *player == seat => {
                    self.eliminated_turn = Some(turn);
                    for war in self.wars.iter_mut().filter(|w| w.ended_turn.is_none()) {
                        war.ended_turn = Some(turn);
                    }
                }
                Event::WarDeclared { relation }
                    if relation.declarers.contains(&state.team_of(seat)) =>
                {
                    self.wars.push(WarRecord {
                        turn,
                        against: if relation.a == team {
                            relation.b.0
                        } else {
                            relation.a.0
                        },
                        captures: 0,
                        losses: 0,
                        ended_turn: None,
                    });
                    self.wars_declared += 1;
                    if opponent.is_some_and(|other| {
                        let other_team = state.team_of(other);
                        (relation.a == team && relation.b == other_team)
                            || (relation.b == team && relation.a == other_team)
                    }) {
                        self.first_opponent_war_turn.get_or_insert(turn);
                    }
                }
                Event::CityCaptured { city, from, to, .. } => {
                    self.cities_captured += u32::from(*to == seat);
                    self.cities_lost += u32::from(*from == seat);
                    if *from == seat
                        && state
                            .cities
                            .get(city)
                            .is_some_and(|c| turn.saturating_sub(c.founded_turn) <= 30)
                    {
                        self.young_cities_lost += 1;
                    }
                    for war in self.wars.iter_mut().filter(|w| w.ended_turn.is_none()) {
                        let other = legatum_core::TeamId(war.against);
                        if *to == seat && state.team_of(*from) == other {
                            war.captures += 1;
                        }
                        if *from == seat && state.team_of(*to) == other {
                            war.losses += 1;
                        }
                    }
                    // Occupying an already-zero-HP city need not emit a Battle.
                    // Observe its old owner from the event, before it became ours.
                    if *to == seat
                        && Some(*from) == opponent
                        && state.cities.get(city).is_some_and(|city| {
                            state
                                .units_of(seat)
                                .any(|u| military(u.kind) && u.pos.distance(city.pos) <= 1)
                        })
                    {
                        self.first_opponent_city_pressure_turn.get_or_insert(turn);
                    }
                }
                Event::CityFounded { owner, at, .. }
                    if *owner == seat
                        && state.cities_of(seat).count() >= 2
                        && state.units.values().any(|u| {
                            state.team_of(u.owner) == team
                                && military(u.kind)
                                && u.pos.distance(*at) <= 1
                        }) =>
                {
                    self.guarded_expansion_turns.push(turn);
                }
                Event::Battle {
                    attacker,
                    attacker_owner,
                    defender: Defender::City { owner, .. },
                    ..
                } if *attacker_owner == seat && Some(*owner) == opponent => {
                    // A military attacker can die in this very battle. Recover its
                    // kind from the destruction event, not from a surviving-unit count.
                    let kind = attacker.unit().and_then(|id| {
                        state.units.get(&id).map(|u| u.kind).or_else(|| {
                            events.iter().find_map(|e| match e {
                                Event::UnitDestroyed {
                                    unit, unit_kind, ..
                                } if *unit == id => Some(*unit_kind),
                                _ => None,
                            })
                        })
                    });
                    if kind.is_some_and(military) {
                        self.first_opponent_city_pressure_turn.get_or_insert(turn);
                    }
                }
                _ => {}
            }
        }
        if self.first_opponent_city_pressure_turn.is_none()
            && let Some(other) = opponent
            && state.at_war(seat, other)
            && state.units_of(seat).any(|u| {
                military(u.kind)
                    && state
                        .cities_of(other)
                        .any(|city| u.pos.distance(city.pos) <= 1)
            })
        {
            self.first_opponent_city_pressure_turn = Some(turn);
        }
    }
    pub fn finish(&mut self, seat: PlayerId, state: &GameState, phases: &[PhaseSwitch]) {
        let vp = legatum_core::scoring::vp_total(state, seat);
        self.rank = 1 + state
            .players
            .iter()
            .filter(|p| legatum_core::scoring::vp_total(state, p.id) > vp)
            .count() as u32;
        self.won = state.winner == Some(state.team_of(seat));
        self.phase_switches = phases.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::game::{blank_game, spawn_unit};
    use legatum_core::rules::cities::found_city;
    use legatum_core::state::Map;
    use legatum_core::{GameSetup, Hex, Order, TeamId, Terrain, resolve_turn};

    fn duel() -> GameState {
        let mut setup = GameSetup::all_ai(93);
        setup.players.truncate(2);
        let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Plains));
        for (seat, pos) in [(0, Hex::new(4, 5)), (1, Hex::new(12, 5))] {
            let settler = spawn_unit(&mut state, PlayerId(seat), UnitKind::Settler, pos);
            found_city(&mut state, settler, &mut Vec::new()).unwrap();
        }
        state
    }

    #[test]
    fn identity_counts_only_the_seats_own_declaration_against_its_duel_opponent() {
        let s = duel();
        let mut r = SeatRecord::default();
        let mut relation = s.diplomacy.as_ref().unwrap().relations[0].clone();
        relation.declarers = vec![s.team_of(PlayerId(1))];
        r.observe(
            PlayerId(0),
            5,
            &s,
            &[Event::WarDeclared {
                relation: relation.clone(),
            }],
        );
        assert_eq!(r.first_opponent_war_turn, None);
        relation.declarers = vec![s.team_of(PlayerId(0))];
        relation.b = TeamId::BARBARIAN;
        r.observe(
            PlayerId(0),
            6,
            &s,
            &[Event::WarDeclared {
                relation: relation.clone(),
            }],
        );
        assert_eq!(r.first_opponent_war_turn, None);
        relation.b = s.team_of(PlayerId(1));
        r.observe(
            PlayerId(0),
            7,
            &s,
            &[Event::WarDeclared {
                relation: relation.clone(),
            }],
        );
        r.observe(PlayerId(0), 9, &s, &[Event::WarDeclared { relation }]);
        assert_eq!(r.first_opponent_war_turn, Some(7));
        assert_eq!(r.first_opponent_city_pressure_turn, None);
    }

    #[test]
    fn identity_adjacency_requires_war_and_non_scout_military_and_is_read_only() {
        let mut s = duel();
        let me = PlayerId(0);
        let mut r = SeatRecord::default();
        let unit = spawn_unit(&mut s, me, UnitKind::Soldier, Hex::new(11, 5));
        r.observe(me, 1, &s, &[]);
        assert_eq!(
            r.first_opponent_city_pressure_turn, None,
            "peaceful adjacency"
        );
        s.diplomacy.as_mut().unwrap().relations[0].state =
            legatum_core::diplomacy::RelationState::War;
        s.units.get_mut(&unit).unwrap().kind = UnitKind::Scout;
        r.observe(me, 2, &s, &[]);
        assert_eq!(
            r.first_opponent_city_pressure_turn, None,
            "scout is not an assault force"
        );
        s.units.get_mut(&unit).unwrap().kind = UnitKind::Soldier;
        s.units.get_mut(&unit).unwrap().owner = PlayerId::BARBARIAN;
        r.observe(me, 3, &s, &[]);
        assert_eq!(r.first_opponent_city_pressure_turn, None);
        s.units.get_mut(&unit).unwrap().owner = me;
        let hash = s.hash();
        r.observe(me, 4, &s, &[]);
        r.observe(me, 5, &s, &[]);
        assert_eq!(r.first_opponent_city_pressure_turn, Some(4));
        assert_eq!(s.hash(), hash);
        assert_eq!(
            r.first_opponent_war_turn, None,
            "being at war is not declaring"
        );
    }

    #[test]
    fn identity_counts_actual_ranged_city_attack_even_if_attacker_dies() {
        let mut s = duel();
        let me = PlayerId(0);
        s.diplomacy.as_mut().unwrap().relations[0].state =
            legatum_core::diplomacy::RelationState::War;
        let unit = spawn_unit(&mut s, me, UnitKind::Ranger, Hex::new(10, 5));
        let mut r = SeatRecord::default();
        r.observe(me, 1, &s, &[]);
        assert_eq!(
            r.first_opponent_city_pressure_turn, None,
            "in range is not an attack"
        );
        let orders = BTreeMap::from([(
            me,
            vec![Order::RangedAttack {
                unit,
                target: Hex::new(12, 5),
            }],
        )]);
        let (mut next, mut events) = resolve_turn(&s, &orders);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Battle {
                defender: Defender::City { .. },
                ..
            }
        )));
        next.units.remove(&unit);
        events.push(Event::UnitDestroyed {
            unit,
            owner: me,
            unit_kind: UnitKind::Ranger,
            at: Hex::new(10, 5),
        });
        r.observe(me, 2, &next, &events);
        assert_eq!(r.first_opponent_city_pressure_turn, Some(2));
        let mut defender = SeatRecord::default();
        defender.observe(PlayerId(1), 2, &next, &events);
        assert_eq!(defender.first_opponent_city_pressure_turn, None);
    }

    #[test]
    fn identity_counts_occupying_zero_hp_city_without_a_battle() {
        let mut s = duel();
        let me = PlayerId(0);
        s.diplomacy.as_mut().unwrap().relations[0].state =
            legatum_core::diplomacy::RelationState::War;
        let city = s.cities_of(PlayerId(1)).next().unwrap().id;
        s.cities.get_mut(&city).unwrap().hp = 0;
        let unit = spawn_unit(&mut s, me, UnitKind::Soldier, Hex::new(11, 5));
        let orders = BTreeMap::from([(
            me,
            vec![Order::MoveUnit {
                unit,
                to: Hex::new(12, 5),
            }],
        )]);
        let (next, events) = resolve_turn(&s, &orders);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::CityCaptured { .. }))
        );
        assert!(
            !events.iter().any(
                |e| matches!(e, Event::Battle { attacker_owner, .. } if *attacker_owner == me)
            )
        );
        let mut r = SeatRecord::default();
        r.observe(me, 2, &next, &events);
        assert_eq!(r.first_opponent_city_pressure_turn, Some(2));
        assert_eq!(r.cities_captured, 1);
        assert_eq!(r.first_opponent_war_turn, None);
    }

    #[test]
    fn guarded_expansion_excludes_capital_scouts_and_enemy_units() {
        let mut s = duel();
        let me = PlayerId(0);
        let mut r = SeatRecord::default();
        let capital = s.cities_of(me).next().unwrap().clone();
        spawn_unit(&mut s, me, UnitKind::Soldier, capital.pos);
        r.observe(
            me,
            1,
            &s,
            &[Event::CityFounded {
                city: capital.id,
                owner: me,
                name: capital.name,
                at: capital.pos,
                veteran: false,
            }],
        );
        assert!(r.guarded_expansion_turns.is_empty());
        let pos = Hex::new(5, 10);
        let settler = spawn_unit(&mut s, me, UnitKind::Settler, pos);
        let mut events = Vec::new();
        found_city(&mut s, settler, &mut events).unwrap();
        let scout = spawn_unit(&mut s, me, UnitKind::Scout, pos);
        spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, pos);
        r.observe(me, 40, &s, &events);
        assert!(r.guarded_expansion_turns.is_empty());
        s.units.get_mut(&scout).unwrap().kind = UnitKind::Soldier;
        r.observe(me, 41, &s, &events);
        assert_eq!(r.guarded_expansion_turns, vec![41]);
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(serde_json::from_value::<SeatRecord>(json).unwrap(), r);
        assert_eq!(
            serde_json::from_str::<SeatRecord>("{}").unwrap(),
            SeatRecord::default()
        );
    }
    #[test]
    fn first_walls_tracks_actual_completion_for_this_seat_only() {
        let s = duel();
        let mut record = SeatRecord::default();
        let completion = |owner| Event::ProductionCompleted {
            city: legatum_core::CityId(0),
            owner,
            item: legatum_core::ProductionItem::Building {
                building: legatum_core::BuildingKind::Walls,
            },
        };
        record.observe(PlayerId(0), 12, &s, &[completion(PlayerId(1))]);
        assert_eq!(record.first_walls_turn, None);
        record.observe(PlayerId(0), 19, &s, &[completion(PlayerId(0))]);
        record.observe(PlayerId(0), 24, &s, &[completion(PlayerId(0))]);
        assert_eq!(record.first_walls_turn, Some(19));
    }

    #[test]
    fn seat_assignments_are_repeatable_and_left_to_right() {
        assert_eq!(seats(&[]).unwrap(), [Opening::Default; 8]);
        let s = seats(&[
            "all=Rush".into(),
            "3=Economic".into(),
            "7=greedy-expand".into(),
        ])
        .unwrap();
        assert_eq!(
            (s[0], s[3], s[7]),
            (Opening::Rush, Opening::Economic, Opening::GreedyExpand)
        );
        for arg in ["8=Economic", "-1=Rush", "Economic", "all=unknown"] {
            assert!(seats(&[arg.into()]).is_err());
        }
    }
    #[test]
    fn opening_report_counts_actual_events_and_missing_checkpoints() {
        let setup = legatum_core::GameSetup::all_ai(4);
        let mut s = legatum_core::new_game(&setup);
        let mut r = SeatRecord::default();
        let mut relation = s.diplomacy.as_ref().unwrap().relations[0].clone();
        relation.declarers = vec![s.team_of(PlayerId(0))];
        let events = vec![
            Event::WarDeclared { relation },
            Event::CityCaptured {
                city: legatum_core::CityId(9),
                from: PlayerId(1),
                to: PlayerId(0),
                bounty: None,
            },
        ];
        r.observe(PlayerId(0), 25, &s, &events);
        assert_eq!(
            (r.wars_declared, r.cities_captured, r.cities_lost),
            (1, 1, 0)
        );
        assert_eq!(r.cities_at_turns["25"], Some(1));
        assert_eq!(r.cities_at_turns["50"], None);
        let mut other = SeatRecord::default();
        other.observe(PlayerId(1), 25, &s, &events);
        assert_eq!(
            (
                other.wars_declared,
                other.cities_captured,
                other.cities_lost
            ),
            (0, 0, 1)
        );
        s.winner = Some(s.team_of(PlayerId(0)));
        r.finish(
            PlayerId(0),
            &s,
            &[PhaseSwitch {
                turn: 3,
                phase: legatum_ai::openings::Phase::Defend,
            }],
        );
        assert_eq!(r.rank, 1);
        assert!(r.won);
        assert_eq!(r.phase_switches.len(), 1);
    }
}
