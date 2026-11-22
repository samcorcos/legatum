//! Read-only simulation observations. These never participate in orders, RNG, saves or hashes.
//! Definitions extend balance-probe-1 to teams; city captures and eliminations stay in the
//! runner's existing canonical records, including the replay exporter's city-capture count.

use legatum_core::{
    Event, GameState, Order, PlayerId, ProductionItem, UnitKind, VpSource, data, scoring,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize)]
pub struct Definitions {
    pub battles: &'static str,
    pub captures: &'static str,
    pub eliminations: &'static str,
    pub scores: &'static str,
    pub turn_50: &'static str,
    pub openings: &'static str,
    pub victory_holds: &'static str,
    pub denominators: &'static str,
    pub strategy: &'static str,
    pub opponent_assessment: &'static str,
}

pub const DEFINITIONS: Definitions = Definitions {
    opponent_assessment: legatum_sim::opponent_telemetry::DEFINITION,
    strategy: "Classifier v2 uses only integer behavioral evidence through Standard turns 120; first matching threshold wins. Assigned policy and observed archetype are separate. Diversity metrics use finished non-panicked games, with explicit exclusions for missing labels and cap stops. Winner entropy uses a fixed vocabulary including zero-win categories. Matchups weight co-occurrence games equally, exclude teammates, preserve rank ties and report both strict ahead rate and half-tie score. See docs/strategy-diversity.md for all thresholds and denominators. Diplomacy is reported separately and is not an input to classifier v2.",
    battles: "One Event::Battle, including Scout encounters, ranged shots and meeting engagements; not a war, siege or submitted attack order.",
    captures: "City captures use the existing GameRecord.city_captures count of Event::CityCaptured (recaptures count again). Settler captures count Event::SettlerCaptured separately; neither is inferred from ownership or unit losses.",
    eliminations: "Count the existing GameRecord.eliminations entries, one per Event::PlayerEliminated; telemetry.elimination_count is their length. Never infer from final alive flags.",
    scores: "Final-state VP: scored World Age cards and Celtic-sacked rival capitals are banked; held rival capitals count for their holder; Great Works and faction goals use their held core VpSource entries for the current holder. Player sources reconcile to core vp_total; team sources sum each member once and reconcile to core team_vp. Player, team and winning-team populations are separate views, never added together. Eliminated members retain banked VP, as in core scoring.",
    turn_50: "After resolution AND scoring of turn 50, when GameState.turn is 51, before turn-51 orders. Rank all initial TEAMS by core team_vp (sum every member once), matching the probe's all-seat population. Equal VP preserves the entire leader set: no population or ID tie-break. Games ending on turn 50 are eligible; games ending earlier are excluded. Turn-75 replay comeback data is independent.",
    openings: "Per initial empire, record the resolved turn (turn 1 is the first turn) of its first Event::ProductionCompleted for Soldier and Settler, and its second Event::CityFounded. Starting units and captured units do not count as production; purchases count only when production actually completes. City captures do not count as founding, and losing a city does not reset the founding count. Null means never reached by the end of observed play. Distributions pool reached empires only, using the report's existing percentile convention; reached and never counts include eliminated empires and safety-cap stops. Cap stops are censored, not proof a milestone could never happen. Only first plays count; panics and legacy records without opening telemetry are excluded, not never observations.",
    victory_holds: "A team wins by VP only after standing at its VP target at data::VP_HOLD_TURNS consecutive turn ends (10 Standard); the turn it reaches the target is the first held turn. Started = Event::VictoryHoldStarted; broken = Event::VictoryHoldBroken (dropped below the target or was eliminated); completed = the hold reached the full count (a VP win, or a draw when several teams complete with identical standings); unfinished = still running when the game ended otherwise (conquest, turn limit, safety cap). started = broken + completed + unfinished. Break rate = broken / started, and broken / (broken + completed) over resolved holds. First reach to win = win turn minus the first turn the winner ever stood at its target (includes broken holds); final hold start to win is always turns_needed - 1. A comeback is a hold win by a team that broke an earlier hold of its own; wins not first to reach are hold wins where another team first stood at its target strictly earlier. Only games with games[].telemetry.victory_holds count; panics and older reports are excluded, not zero. Outcome `vp` means a completed hold: a winner at or above its target at the turn limit without a completed hold is `turn_limit`.",
    denominators: "Event totals and incidence use first plays with telemetry, including runner-stopped games; panics and legacy records without telemetry are excluded, not zeros. Terminal score populations use finished games only. Snowball cohorts use finished games with a turn-50 snapshot and at least one team; unique and tied leaders are separate. Draws stay in each eligible denominator and count as no leader win. Unfinished games have no eventual result and are excluded. Empty denominators are reported as 0/0 (no rate).",
};

/// Read-only event counters and first-occurrence turns, observed from the start of a new game.
/// Captures of cities and eliminations already have canonical fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Collector {
    pub proxy: Option<crate::proxy_benchmark::Game>,
    pub opponents: legatum_sim::opponent_telemetry::Observer,
    strategy: Option<legatum_sim::diversity::Collector>,
    ruin_outcomes: BTreeMap<u8, BTreeMap<String, u32>>,
    diplomacy: DiplomacyTelemetry,
    fought: BTreeSet<PlayerId>,
    lost_city: BTreeSet<PlayerId>,
    broken_alliances: BTreeSet<u32>,
    battles: u64,
    city_strike_losses: BTreeSet<(u8, legatum_core::UnitId)>,
    settler_captures: u64,
    promotions: BTreeMap<u8, u64>,
    turn_50: Option<Turn50Snapshot>,
    faction_goal_turns: BTreeMap<u8, u32>,
    capital_vp_turns: Vec<(u8, u32)>,
    capture_objective_turns: Vec<(u8, u32)>,
    india_office_first_turn: BTreeMap<u8, u32>,
    india_office_coverage_by_turn: BTreeMap<u8, BTreeMap<u32, [u32; 2]>>,
    openings: BTreeMap<u8, EmpireOpening>,
    cities_founded: BTreeMap<u8, u32>,
    investments: BTreeMap<u8, EmpireInvestments>,
    bounty_gold_received: BTreeMap<u8, i64>,
    swiss_treasury: Vec<SwissTreasury>,
    research_turns: BTreeMap<u8, BTreeMap<String, u32>>,
    /// The resolved turn each empire entered each era (`Event::EraEntered`).
    era_entries: BTreeMap<u8, BTreeMap<u8, u32>>,
    /// Resolved turns an empire had techs left to learn but no research set (the turn an era
    /// is entered is excluded: the queue is necessarily empty then), and the part of them
    /// that followed another such turn.
    research_idle_turns: BTreeMap<u8, u32>,
    research_stall_turns: BTreeMap<u8, u32>,
    aircraft_built: BTreeMap<u8, u32>,
    unsupplied_aircraft_turns: BTreeMap<u8, u64>,
    research_idle_last: BTreeSet<u8>,
    peak_cities: BTreeMap<u8, usize>,
    great_work_turns: BTreeMap<u8, BTreeMap<String, u32>>,
    great_works_bought: BTreeMap<u8, u32>,
    barbarians: crate::barbarian_telemetry::BarbarianTelemetry,
    company_hires: Vec<CompanyHire>,
    companies_departed: BTreeMap<u8, u32>,
    companies_turned: BTreeMap<u8, u32>,
    levies: BTreeMap<u8, LevyTelemetry>,
    celts_razes: BTreeMap<u8, u64>,
    germany_project_city_turns: BTreeMap<String, u64>,
    sparta: Option<SpartaTelemetry>,
    empire_flow: BTreeMap<u8, EmpireFlowTelemetry>,
    teams: BTreeMap<u8, u8>,
    completed_items: Vec<(u8, ProductionItem)>,
    bought_items: Vec<(u8, ProductionItem)>,
    last_city_captor: BTreeMap<u8, u8>,
    /// Units disbanded for unpaid upkeep (`rules::cities::pay_gold`), by owner.
    upkeep_disbands: BTreeMap<u8, u64>,
    borders: crate::border_telemetry::Collector,
    city_combat: CityCombatTelemetry,
    /// City tiles before the turn being resolved: tile -> (city, owner).
    city_tiles: BTreeMap<legatum_core::Hex, (legatum_core::CityId, PlayerId)>,
    /// Open sieges by city tile: (first hit turn, last hit turn).
    sieges: BTreeMap<legatum_core::Hex, (u32, u32)>,
    holds: crate::hold_telemetry::Observer,
    goal_cities: crate::goal_city_telemetry::Observer,
}

/// Read-only city-combat observations: who declares, garrisons lost, and siege lengths.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityCombatTelemetry {
    /// War declarations by declaring team (one per declarer of each `WarDeclared`).
    pub declarations_by_team: BTreeMap<u8, u64>,
    /// Military units destroyed on a tile their owner's city held at the start of the turn.
    pub garrison_deaths: BTreeMap<u8, u64>,
    /// Military units destroyed, by owner.
    pub military_losses: BTreeMap<u8, u64>,
    /// One entry per capture or raze: (captor, turns from the first non-barbarian hit on the
    /// city tile to the capture, inclusive). A siege ends after 5 turns without a hit.
    pub siege_turns: Vec<(u8, u32)>,
}

/// A siege with no hit on the city tile for this many turns is over.
const SIEGE_GAP_TURNS: u32 = 5;

impl Collector {
    pub fn new() -> Self {
        Self {
            strategy: Some(legatum_sim::diversity::Collector::new()),
            ..Self::default()
        }
    }

    pub fn observe_initial(&mut self, state: &GameState) {
        self.goal_cities.observe_initial(state);
        for player in &state.players {
            self.empire_flow
                .insert(player.id.0, EmpireFlowTelemetry::default());
            self.teams.insert(player.id.0, player.team.0);
            if let Some(capital) = state.cities_of(player.id).find(|c| c.original_capital) {
                self.cities_founded.insert(player.id.0, 1);
                self.openings
                    .entry(player.id.0)
                    .or_default()
                    .first_city_turn = Some(capital.founded_turn);
            }
        }
        let Some(player) = state
            .players
            .iter()
            .find(|p| p.faction == legatum_core::Faction::Sparta)
        else {
            return;
        };
        let nearest_rival_hexes = state.capital_of(player.id).and_then(|home| {
            state
                .cities
                .values()
                .filter(|city| city.owner != player.id && !state.same_team(city.owner, player.id))
                .map(|city| home.pos.distance(city.pos))
                .min()
        });
        self.sparta = Some(SpartaTelemetry {
            seat: player.id.0,
            nearest_rival_hexes,
            ..Default::default()
        });
    }

    pub fn observe_event(&mut self, resolved_turn: u32, event: &Event) {
        self.holds.observe_event(resolved_turn, event);
        self.goal_cities.observe_event(resolved_turn, event);
        if let Event::ResourcesRevealed {
            player, resource, ..
        } = event
            && let Some(i) = legatum_core::Resource::STRATEGIC
                .iter()
                .position(|r| r == resource)
        {
            self.openings.entry(player.0).or_default().first_reveal_turn[i]
                .get_or_insert(resolved_turn);
        }
        match event {
            Event::WarDeclared { relation } => {
                for (&seat, &team) in &self.teams {
                    if team == relation.a.0 || team == relation.b.0 {
                        self.empire_flow
                            .entry(seat)
                            .or_default()
                            .first_war_turn
                            .get_or_insert(resolved_turn);
                    }
                }
            }
            Event::CityCaptured { to, from, .. } => {
                let flow = self.empire_flow.entry(to.0).or_default();
                flow.first_capture_turn.get_or_insert(resolved_turn);
                flow.captures += 1;
                self.last_city_captor.insert(from.0, to.0);
            }
            Event::CapitalTaken { captor, .. } => {
                self.capital_vp_turns.push((captor.0, resolved_turn))
            }
            Event::EraObjectiveScored {
                player, objective, ..
            } if objective.to_ascii_lowercase().contains("captur") => {
                self.capture_objective_turns.push((player.0, resolved_turn));
            }
            Event::PlayerEliminated { player } => {
                self.empire_flow.entry(player.0).or_default().eliminated_by =
                    self.last_city_captor.get(&player.0).copied();
            }
            Event::ItemBought {
                owner, item, gold, ..
            } => {
                self.empire_flow
                    .entry(owner.0)
                    .or_default()
                    .gold_spent_on_items += i64::from(*gold);
                self.bought_items.push((owner.0, *item));
            }
            Event::UnitUpgraded { owner, gold, .. } => {
                self.empire_flow
                    .entry(owner.0)
                    .or_default()
                    .gold_spent_on_items += i64::from(*gold);
            }
            Event::ProductionCompleted { owner, item, .. } => {
                if let Some(i) = self
                    .bought_items
                    .iter()
                    .position(|entry| *entry == (owner.0, *item))
                {
                    self.bought_items.remove(i);
                } else {
                    self.completed_items.push((owner.0, *item));
                }
            }
            _ => {}
        }
        if let Some(sparta) = &mut self.sparta {
            match event {
                Event::SpartaConquest {
                    player,
                    units_healed,
                    ..
                } if player.0 == sparta.seat => {
                    sparta.captures += 1;
                    sparta.first_capture_turn.get_or_insert(resolved_turn);
                    sparta.heals_triggered += u64::from(*units_healed);
                }
                Event::UnitDestroyed {
                    owner, unit_kind, ..
                } if owner.0 == sparta.seat && data::unit_stats(*unit_kind).military => {
                    sparta.military_destroyed += 1;
                }
                Event::CompanyHired {
                    owner,
                    seller,
                    unit,
                    gold,
                    until,
                    ..
                } => {
                    self.company_hires.push(CompanyHire {
                        turn: resolved_turn,
                        buyer: owner.0,
                        seller: seller.0,
                        unit: format!("{unit:?}"),
                        gold: *gold,
                        until: *until,
                    });
                }
                Event::CompanyDeparted { owner, .. } => {
                    *self.companies_departed.entry(owner.0).or_default() += 1;
                }
                Event::CompanyTurned { owner, .. } => {
                    *self.companies_turned.entry(owner.0).or_default() += 1;
                }
                _ => {}
            }
        }
        // Ranged damage is simultaneous: attribute a destroyed defender to city fire
        // when at least one city hit it in that phase, even if unit fire also contributed.
        // Several cities hitting the same unit still count as one loss.
        if let Event::Battle {
            attacker: legatum_core::events::Attacker::City { .. },
            defender: legatum_core::events::Defender::Unit { unit, owner, .. },
            defender_hp_after: 0,
            ..
        } = event
        {
            self.city_strike_losses.insert((owner.0, *unit));
        }
        self.observe_city_combat(resolved_turn, event);
        if let Some(strategy) = &mut self.strategy {
            strategy.observe_event(resolved_turn, event);
        }
        self.barbarians.observe(resolved_turn, event);
        self.borders.observe_event(resolved_turn, event);
        match event {
            Event::ItemBought {
                owner,
                item: ProductionItem::Building { building },
                ..
            } if data::is_great_work(*building) => {
                *self.great_works_bought.entry(owner.0).or_default() += 1;
            }
            Event::RuinExplored { owner, reward, .. } => {
                let kind = reward.kind_name();
                *self
                    .ruin_outcomes
                    .entry(owner.0)
                    .or_default()
                    .entry(kind.into())
                    .or_default() += 1;
            }
            Event::CityRazed {
                by, ordered: false, ..
            } => {
                *self.celts_razes.entry(by.0).or_default() += 1;
            }
            Event::EraEntered { player, era, .. } => {
                self.era_entries
                    .entry(player.0)
                    .or_default()
                    // The event's era is 1-based (VP numbering); the pacing report is 0-based.
                    .entry(era.saturating_sub(1))
                    .or_insert(resolved_turn);
            }
            Event::TechResearched { player, tech } => {
                self.research_turns
                    .entry(player.0)
                    .or_default()
                    .entry(format!("{tech:?}"))
                    .or_insert(resolved_turn);
            }
            Event::CityCaptured { to, bounty, .. } => {
                self.investments.entry(to.0).or_default().city_captures += 1;
                if let Some(gold) = bounty {
                    *self.bounty_gold_received.entry(to.0).or_default() += i64::from(*gold);
                }
            }
            Event::ProductionCompleted { owner, item, .. } => {
                if let ProductionItem::Unit { unit } = item
                    && data::is_aircraft(*unit)
                {
                    *self.aircraft_built.entry(owner.0).or_default() += 1;
                }
                if let ProductionItem::Building { building } = item
                    && data::is_great_work(*building)
                {
                    self.great_work_turns
                        .entry(owner.0)
                        .or_default()
                        .insert(format!("{building:?}"), resolved_turn);
                }
                if *item
                    == (ProductionItem::Building {
                        building: legatum_core::BuildingKind::Market,
                    })
                {
                    self.india_office_first_turn
                        .entry(owner.0)
                        .or_insert(resolved_turn);
                }
                let totals = self.investments.entry(owner.0).or_default();
                *totals.completed.entry(format!("{item:?}")).or_default() += 1;
            }
            Event::UnitUpgraded { owner, to, .. } => {
                *self
                    .investments
                    .entry(owner.0)
                    .or_default()
                    .upgraded
                    .entry(format!("{to:?}"))
                    .or_default() += 1;
            }
            Event::OrderFailed { player, reason }
                if reason.starts_with("Ranged attack by unit ")
                    && reason.contains("No line of sight:") =>
            {
                self.investments
                    .entry(player.0)
                    .or_default()
                    .blocked_ranged_attacks += 1;
            }
            Event::Battle {
                attacker_owner,
                attacker_strength,
                ranged: true,
                ..
            } => {
                *self
                    .investments
                    .entry(attacker_owner.0)
                    .or_default()
                    .ranged_attacks
                    .entry(
                        attacker_strength
                            .parts
                            .first()
                            .map_or("Unknown", |(label, _)| label.as_str())
                            .to_string(),
                    )
                    .or_default() += 1;
            }
            _ => {}
        }
        match event {
            Event::WarDeclared { relation } => {
                self.diplomacy.declarations += relation.declarers.len() as u64;
                self.diplomacy.war_starts += 1;
            }
            Event::PeaceEstablished { .. } => self.diplomacy.peace_agreements += 1,
            Event::ProposalCreated { proposal }
                if proposal.kind == legatum_core::diplomacy::TreatyKind::Peace =>
            {
                self.diplomacy.peace_offers += 1
            }
            Event::TreatyActivated { relation } => {
                self.diplomacy.alliances_signed += 1;
                if let Some(target) = relation.security.as_ref().and_then(|t| t.coalition_target) {
                    *self
                        .diplomacy
                        .alliances_by_target
                        .entry(target.0.to_string())
                        .or_default() += 1;
                }
                *self
                    .diplomacy
                    .alliances_by_pair
                    .entry(format!("{}:{}", relation.a.0, relation.b.0))
                    .or_default() += 1;
            }
            Event::TreatyBreached { treaty_id, .. } => {
                self.broken_alliances.insert(*treaty_id);
                self.diplomacy.alliances_broken = self.broken_alliances.len() as u64;
            }
            Event::CityCaptured { from, .. } => {
                self.lost_city.insert(*from);
            }
            Event::Battle {
                attacker_owner,
                defender,
                ..
            } => {
                self.battles += 1;
                self.fought.insert(*attacker_owner);
                self.fought.insert(match defender {
                    legatum_core::events::Defender::Unit { owner, .. }
                    | legatum_core::events::Defender::City { owner, .. } => *owner,
                });
            }
            Event::SettlerCaptured { .. } => self.settler_captures += 1,
            Event::UnitDisbanded { owner, reason, .. } if reason.contains("Treasury ran dry") => {
                *self.upkeep_disbands.entry(owner.0).or_default() += 1;
            }
            Event::UnitPromoted { level, .. } => *self.promotions.entry(*level).or_default() += 1,
            Event::ProductionCompleted {
                owner,
                item: ProductionItem::Unit { unit },
                ..
            } => {
                let opening = self.openings.entry(owner.0).or_default();
                match unit {
                    UnitKind::Spearman => {
                        opening.first_soldier_turn.get_or_insert(resolved_turn);
                    }
                    UnitKind::Settler => {
                        opening.first_settler_turn.get_or_insert(resolved_turn);
                    }
                    _ => {}
                }
            }
            Event::ResourcesRevealed {
                player,
                resource,
                city: Some(_),
                ..
            } => {
                let opening = self.openings.entry(player.0).or_default();
                match resource {
                    legatum_core::Resource::Iron => {
                        opening.first_iron_source_turn.get_or_insert(resolved_turn);
                    }
                    legatum_core::Resource::Coal => {
                        opening.first_coal_source_turn.get_or_insert(resolved_turn);
                    }
                    _ => {}
                }
            }
            Event::CityFounded { owner, .. } => {
                let founded = self.cities_founded.entry(owner.0).or_default();
                *founded += 1;
                if *founded == 1 {
                    self.openings.entry(owner.0).or_default().first_city_turn = Some(resolved_turn);
                }
                if *founded == 2 {
                    self.openings.entry(owner.0).or_default().second_city_turn =
                        Some(resolved_turn);
                }
            }
            _ => {}
        }
    }

    fn observe_city_combat(&mut self, turn: u32, event: &Event) {
        let mut ended = |tile: legatum_core::Hex, captor: PlayerId| {
            let start = self.sieges.remove(&tile).map_or(turn, |(start, _)| start);
            self.city_combat
                .siege_turns
                .push((captor.0, turn + 1 - start));
        };
        match event {
            Event::WarDeclared { relation } => {
                for team in &relation.declarers {
                    *self
                        .city_combat
                        .declarations_by_team
                        .entry(team.0)
                        .or_default() += 1;
                }
            }
            Event::UnitDestroyed {
                owner,
                unit_kind,
                at,
                ..
            } if data::unit_stats(*unit_kind).military => {
                *self.city_combat.military_losses.entry(owner.0).or_default() += 1;
                if self.city_tiles.get(at).is_some_and(|(_, o)| o == owner) {
                    *self.city_combat.garrison_deaths.entry(owner.0).or_default() += 1;
                }
            }
            Event::Battle {
                attacker: legatum_core::events::Attacker::Unit(_),
                attacker_owner,
                at,
                ..
            } if !attacker_owner.is_barbarian() && self.city_tiles.contains_key(at) => {
                let siege = self.sieges.entry(*at).or_insert((turn, turn));
                siege.1 = turn;
            }
            Event::CityCaptured { city, to, .. } => {
                if let Some((&tile, _)) = self.city_tiles.iter().find(|(_, (c, _))| c == city) {
                    ended(tile, *to);
                }
            }
            Event::CityRazed { at, by, .. } => ended(*at, *by),
            _ => {}
        }
    }

    /// Call once after Session::step, using the turn number from BEFORE that step.
    pub fn observe_resolved_turn(&mut self, resolved_turn: u32, state: &GameState) {
        self.holds.observe_resolved_turn(resolved_turn);
        self.goal_cities.observe_resolved_turn(resolved_turn, state);
        self.borders.observe_resolved_turn(resolved_turn, state);
        for (seat, item) in self.completed_items.drain(..) {
            self.empire_flow
                .entry(seat)
                .or_default()
                .completed_production_value += i64::from(legatum_core::economy::production_cost(
                state,
                PlayerId(seat),
                item,
            ));
        }
        self.bought_items.clear();
        let mut idle_now = BTreeSet::new();
        for player in state.players.iter().filter(|p| p.alive) {
            self.empire_flow.entry(player.id.0).or_default().upkeep_paid +=
                i64::from(legatum_core::economy::upkeep(state, player.id));
            // The turn an era is entered the plan is necessarily empty (the gate allows a
            // queue only for the current era): that gap is not idleness.
            let just_entered = self
                .era_entries
                .get(&player.id.0)
                .is_some_and(|eras| eras.values().any(|t| *t == resolved_turn));
            if player.researching.is_none()
                && !just_entered
                && !legatum_core::eras::all_researched(&player.techs)
            {
                idle_now.insert(player.id.0);
                *self.research_idle_turns.entry(player.id.0).or_default() += 1;
                if self.research_idle_last.contains(&player.id.0) {
                    *self.research_stall_turns.entry(player.id.0).or_default() += 1;
                }
            }
        }
        self.research_idle_last = idle_now;
        for player in state.players.iter().filter(|p| p.alive) {
            for record in &player.supply {
                if let Some(i) = legatum_core::Resource::STRATEGIC
                    .iter()
                    .position(|r| *r == record.resource)
                {
                    self.openings.entry(player.id.0).or_default().first_use_turn[i]
                        .get_or_insert(resolved_turn);
                }
            }
        }
        for aircraft in &state.aircraft {
            let Some(city) = state.cities.get(&aircraft.city) else {
                continue;
            };
            if legatum_core::resources::unit_need(aircraft.kind).is_some()
                && state.player(city.owner).supply.iter().any(|record| {
                    record.user == legatum_core::resources::SupplyUser::Unit { unit: aircraft.id }
                        && record.idle
                })
            {
                *self
                    .unsupplied_aircraft_turns
                    .entry(city.owner.0)
                    .or_default() += 1;
            }
        }
        for player in &state.players {
            let flow = self.empire_flow.entry(player.id.0).or_default();
            if [50, 100, 150].contains(&resolved_turn) {
                flow.city_count_by_turn
                    .insert(resolved_turn, state.cities_of(player.id).count() as u32);
            }
            if player.faction == legatum_core::Faction::Germany
                && [60, 120].contains(&resolved_turn)
            {
                flow.city_count_by_turn
                    .insert(resolved_turn, state.cities_of(player.id).count() as u32);
                flow.population_by_turn.insert(
                    resolved_turn,
                    state
                        .cities_of(player.id)
                        .map(|city| city.population as u32)
                        .sum(),
                );
            }
            if player.faction == legatum_core::Faction::India && [100, 150].contains(&resolved_turn)
            {
                let provinces: Vec<_> = state
                    .cities_of(player.id)
                    .filter(|c| !c.is_capital)
                    .collect();
                let offices = provinces
                    .iter()
                    .filter(|c| {
                        data::has_building(&c.buildings, legatum_core::BuildingKind::Market)
                    })
                    .count();
                self.india_office_coverage_by_turn
                    .entry(player.id.0)
                    .or_default()
                    .insert(resolved_turn, [offices as u32, provinces.len() as u32]);
            }
            if player.alive
                && state.players.iter().any(|other| {
                    other.alive && other.team != player.team && state.at_war(player.id, other.id)
                })
            {
                flow.war_turns += 1;
            } else if player.alive {
                flow.peace_turns += 1;
            }
        }
        if let Some(sparta) = &mut self.sparta {
            let player = PlayerId(sparta.seat);
            if [25, 50, 75, 100, 150, 200, 300].contains(&resolved_turn) {
                sparta
                    .city_count_by_turn
                    .insert(resolved_turn, state.cities_of(player).count() as u32);
                sparta.army_by_turn.insert(
                    resolved_turn,
                    state
                        .units
                        .values()
                        .filter(|u| {
                            u.owner == player
                                && data::unit_stats(u.kind).military
                                && u.kind != UnitKind::Scout
                        })
                        .count() as u32,
                );
            }
        }
        self.city_tiles = state
            .cities
            .values()
            .map(|c| (c.pos, (c.id, c.owner)))
            .collect();
        self.sieges.retain(|tile, (_, last)| {
            self.city_tiles.contains_key(tile) && *last + SIEGE_GAP_TURNS > resolved_turn
        });
        for city in state.cities.values().filter(|c| c.resistance == 0) {
            if let Some(legatum_core::ProductionItem::Project { project }) = city.producing {
                *self
                    .germany_project_city_turns
                    .entry(format!("{project:?}"))
                    .or_default() += 1;
            }
        }
        let mut progress: Vec<_> = state
            .teams()
            .into_iter()
            .filter(|t| state.team_alive(*t))
            .map(|t| {
                (
                    100 * scoring::team_vp(state, t) / scoring::team_target(state, t).max(1),
                    t,
                )
            })
            .collect();
        progress.sort_by_key(|(p, t)| (std::cmp::Reverse(*p), *t));
        if let Some(&(lead, team)) = progress.first() {
            let next = progress.get(1).map_or(0, |x| x.0);
            if lead > next && (lead >= 80 || (lead >= 60 && lead - next >= 15)) {
                self.diplomacy.leader_eligible_turns += 1;
                self.diplomacy.leader_at_war_turns += u64::from(
                    state
                        .teams()
                        .into_iter()
                        .any(|t| state.team_alive(t) && state.teams_at_war(team, t)),
                );
                self.diplomacy.leader_fought_turns +=
                    u64::from(self.fought.iter().any(|p| state.team_of(*p) == team));
                self.diplomacy.leader_city_loss_turns +=
                    u64::from(self.lost_city.iter().any(|p| state.team_of(*p) == team));
            }
        }
        if let Some(team) = legatum_core::diplomacy::coalition::target_state(state) {
            self.diplomacy.coalition_eligible_turns += 1;
            self.diplomacy.coalition_at_war_turns += u64::from(
                state
                    .teams()
                    .into_iter()
                    .any(|t| state.team_alive(t) && state.teams_at_war(team, t)),
            );
            self.diplomacy.coalition_fought_turns +=
                u64::from(self.fought.iter().any(|p| state.team_of(*p) == team));
            self.diplomacy.coalition_city_loss_turns +=
                u64::from(self.lost_city.iter().any(|p| state.team_of(*p) == team));
            let vp_target = legatum_core::diplomacy::coalition::target(
                progress.iter().map(|(p, t)| (*t, *p, 100)),
            )
            .map(|x| x.0);
            self.diplomacy.expansion_target_turns += u64::from(vp_target != Some(team));
        }
        self.fought.clear();
        self.lost_city.clear();
        if let Some(strategy) = &mut self.strategy {
            strategy.observe_resolved_turn(resolved_turn, state);
        }
        for p in &state.players {
            if p.alive
                && p.faction == legatum_core::Faction::Switzerland
                && SWISS_TREASURY_TURNS.contains(&resolved_turn)
            {
                let income = legatum_core::economy::player_income(state, p.id);
                self.swiss_treasury.push(SwissTreasury {
                    turn: resolved_turn,
                    player: p.id.0,
                    gold: p.gold,
                    interest: legatum_core::economy::swiss_interest(state, p.id),
                    city_gold: income.cities.gold,
                    net_gold: income.gold.total,
                    cities: state.cities_of(p.id).count() as u32,
                });
            }
            if let Some((units, cap)) = legatum_core::economy::levy(state, p.id).filter(|_| p.alive)
            {
                let l = self.levies.entry(p.id.0).or_default();
                l.turns += 1;
                l.unit_turns += units as u64;
                l.cap_turns += cap as u64;
                let full = units >= cap;
                l.at_cap_turns += u32::from(full);
                l.war_at_cap_turns += u32::from(
                    full && state
                        .players
                        .iter()
                        .any(|q| q.alive && q.team != p.team && state.at_war(p.id, q.id)),
                );
                if full {
                    l.stalled_city_turns += state
                        .cities_of(p.id)
                        .filter(|c| {
                            matches!(c.producing, Some(legatum_core::ProductionItem::Unit { unit })
                                if legatum_core::data::pays_upkeep(unit))
                        })
                        .count() as u32;
                }
            }
            let cities = state.cities_of(p.id).count();
            self.peak_cities
                .entry(p.id.0)
                .and_modify(|n| *n = (*n).max(cities))
                .or_insert(cities);
            if scoring::vp(state, p.id)
                .iter()
                .any(|(source, _)| matches!(source, VpSource::FactionGoal { .. }))
            {
                self.faction_goal_turns
                    .entry(p.id.0)
                    .or_insert(resolved_turn);
            }
        }
        if resolved_turn == 50 {
            debug_assert_eq!(state.turn, 51);
            let team_vp: BTreeMap<_, _> = state
                .teams()
                .into_iter()
                .map(|team| (team.0, scoring::team_vp(state, team)))
                .collect();
            let best = team_vp.values().max().copied();
            let leaders = team_vp
                .iter()
                .filter(|(_, vp)| Some(**vp) == best)
                .map(|(team, _)| *team)
                .collect();
            self.turn_50 = Some(Turn50Snapshot {
                resolved_turn,
                next_turn: state.turn,
                team_vp,
                leaders,
            });
        }
    }

    pub fn observe_decisions(
        &mut self,
        state: &GameState,
        orders: &legatum_wasm::session::OrderMap,
    ) {
        if let Some(sparta) = &mut self.sparta {
            let player = PlayerId(sparta.seat);
            let submitted = orders.get(&player);
            for unit in state.units.values().filter(|u| {
                u.owner == player
                    && data::unit_stats(u.kind).military
                    && u.kind != UnitKind::Scout
                    && u.hp < 50
                    && !state.cities_of(player).any(|city| city.pos == u.pos)
            }) {
                sparta.wounded_field_unit_turns += 1;
                if unit.hp < 30 {
                    sparta.critical_field_unit_turns += 1;
                }
                let active_order = submitted.is_some_and(|orders| {
                    orders.iter().any(|o| match o {
                        Order::MoveUnit { unit: id, .. }
                        | Order::MoveUnitPath { unit: id, .. }
                        | Order::RangedAttack { unit: id, .. }
                        | Order::UpgradeUnit { unit: id } => *id == unit.id,
                        _ => false,
                    })
                });
                if !active_order
                    && unit.goto.is_none()
                    && unit.attack_target.is_none()
                    && unit.automation.is_none()
                {
                    sparta.wounded_field_idle_unit_turns += 1;
                    if unit.hp < 30 {
                        sparta.critical_field_idle_unit_turns += 1;
                    }
                    if state.cities.values().any(|city| {
                        !city.owner.is_barbarian()
                            && state.team_of(city.owner) != state.team_of(player)
                            && city.pos.distance(unit.pos) <= 6
                    }) {
                        sparta.wounded_front_idle_unit_turns += 1;
                    }
                }
            }
        }
    }

    pub fn finish(&self, state: &GameState, elimination_count: usize) -> GameTelemetry {
        GameTelemetry {
            vp_race: None,
            opponent_assessment: Some(self.opponents.seats.clone()),
            sparta: self.sparta.clone(),
            empire_flow: self.empire_flow.clone(),
            barbarians: state.setup.barbarians.then(|| self.barbarians.clone()),
            proxy: self.proxy.clone(),
            strategy: self.strategy.as_ref().map(|s| s.telemetry.clone()),
            ruin_outcomes: self.ruin_outcomes.clone(),
            peak_cities: self.peak_cities.clone(),
            great_work_turns: self.great_work_turns.clone(),
            great_works_bought: self.great_works_bought.clone(),
            bounty_gold_received: self.bounty_gold_received.clone(),
            swiss_treasury: self.swiss_treasury.clone(),
            company_hires: self.company_hires.clone(),
            companies_departed: self.companies_departed.clone(),
            companies_turned: self.companies_turned.clone(),
            levies: self.levies.clone(),
            goal_cities: self.goal_cities.finish(),
            celts_razes: self.celts_razes.clone(),
            upkeep_disbands: self.upkeep_disbands.clone(),
            city_combat: Some(self.city_combat.clone()),
            germany_project_city_turns: self.germany_project_city_turns.clone(),
            india_nearest_city_hexes_x100: state
                .players
                .iter()
                .filter(|p| p.faction == legatum_core::Faction::India)
                .filter_map(|p| {
                    let cities: Vec<_> = state.cities_of(p.id).collect();
                    (cities.len() >= 2).then(|| {
                        let total: i32 = cities
                            .iter()
                            .map(|city| {
                                cities
                                    .iter()
                                    .filter(|other| other.id != city.id)
                                    .map(|other| city.pos.distance(other.pos))
                                    .min()
                                    .unwrap_or(0)
                            })
                            .sum();
                        (p.id.0, total * 100 / cities.len() as i32)
                    })
                })
                .collect(),
            india_linked_city_share_x10000: state
                .players
                .iter()
                .filter(|p| p.faction == legatum_core::Faction::India)
                .map(|p| {
                    let cities: Vec<_> = state.cities_of(p.id).collect();
                    let linked = cities
                        .iter()
                        .filter(|city| {
                            cities.iter().any(|other| {
                                other.id != city.id
                                    && city.pos.distance(other.pos) <= data::INDIA_NETWORK_RADIUS
                            })
                        })
                        .count();
                    let share = if cities.is_empty() {
                        0
                    } else {
                        linked as i32 * 10_000 / cities.len() as i32
                    };
                    (p.id.0, share)
                })
                .collect(),
            research_turns: self.research_turns.clone(),
            era_entries: self.era_entries.clone(),
            research_idle_turns: self.research_idle_turns.clone(),
            research_stall_turns: self.research_stall_turns.clone(),
            aircraft_built: self.aircraft_built.clone(),
            unsupplied_aircraft_turns: self.unsupplied_aircraft_turns.clone(),
            research_eras: state
                .players
                .iter()
                .map(|p| {
                    let mut counts: BTreeMap<String, u32> = legatum_core::eras::ERA_NAMES
                        .iter()
                        .map(|name| (name.to_string(), 0))
                        .collect();
                    for tech in &p.techs {
                        *counts.entry(data::tech_era(*tech).to_string()).or_default() += 1;
                    }
                    (p.id.0, counts)
                })
                .collect(),
            investments: state
                .players
                .iter()
                .map(|p| {
                    (
                        p.id.0,
                        self.investments.get(&p.id.0).cloned().unwrap_or_default(),
                    )
                })
                .collect(),
            diplomacy: Some(self.diplomacy.clone()),
            borders: Some(self.borders.telemetry.clone()),
            battles: self.battles,
            city_strike_losses: Some(self.city_strike_losses.iter().fold(
                BTreeMap::new(),
                |mut counts, (owner, _)| {
                    *counts.entry(*owner).or_default() += 1;
                    counts
                },
            )),
            settler_captures: self.settler_captures,
            promotions: Some(PromotionTelemetry {
                promoted_to: self.promotions.clone(),
                final_military_by_level: state
                    .units
                    .values()
                    .filter(|u| data::unit_stats(u.kind).military)
                    .fold(BTreeMap::new(), |mut m, u| {
                        *m.entry(legatum_core::rules::promotion::level_for(
                            state.player(u.owner).faction,
                            u.xp,
                        ))
                        .or_default() += 1;
                        m
                    }),
            }),
            elimination_count: elimination_count as u64,
            final_scores: FinalScores::of(state),
            turn_50: self.turn_50.clone(),
            faction_goal_turns: self.faction_goal_turns.clone(),
            capital_vp_turns: self.capital_vp_turns.clone(),
            capital_capture_objective_stacks: self
                .capital_vp_turns
                .iter()
                .filter(|award| self.capture_objective_turns.contains(award))
                .count() as u32,
            india_office_first_turn: self.india_office_first_turn.clone(),
            india_office_coverage_by_turn: self.india_office_coverage_by_turn.clone(),
            victory_holds: Some(self.holds.finish(state)),
            openings: Some(
                state
                    .players
                    .iter()
                    .map(|p| {
                        (
                            p.id.0,
                            self.openings.get(&p.id.0).copied().unwrap_or_default(),
                        )
                    })
                    .collect(),
            ),
        }
    }
}

/// Read-only promotion observations (rules::promotion).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionTelemetry {
    /// UnitPromoted events by the level reached.
    pub promoted_to: BTreeMap<u8, u64>,
    /// Military units alive in the final state, by promotion level (0 = never promoted).
    pub final_military_by_level: BTreeMap<u8, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn50Snapshot {
    pub resolved_turn: u32,
    pub next_turn: u32,
    pub team_vp: BTreeMap<u8, u32>,
    /// All teams at maximum VP, ordered by team ID solely for stable serialization.
    pub leaders: Vec<u8>,
}

fn all_none(turns: &[Option<u32>; 5]) -> bool {
    turns.iter().all(Option::is_none)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireOpening {
    /// First founding event, including the starting capital.
    #[serde(default)]
    pub first_city_turn: Option<u32>,
    /// Resolved production-completion turns, excluding starting and captured units.
    pub first_soldier_turn: Option<u32>,
    pub first_settler_turn: Option<u32>,
    /// Second founding event, irrespective of subsequent city captures or losses.
    pub second_city_turn: Option<u32>,
    /// First turn a known Iron / Coal deposit inside the empire's borders fed its stockpile
    /// (G11 `ResourcesRevealed` with a city). Absent in older reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_iron_source_turn: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_coal_source_turn: Option<u32>,
    /// By `Resource::STRATEGIC` order (Iron, Niter, Coal, Oil, Silicon): the first turn the empire
    /// knew the resource (its revealing technology learned), and the first turn a building or
    /// unit used it (a supply record). Rules version 25, same-era reveals.
    #[serde(default, skip_serializing_if = "all_none")]
    pub first_reveal_turn: [Option<u32>; 5],
    #[serde(default, skip_serializing_if = "all_none")]
    pub first_use_turn: [Option<u32>; 5],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireInvestments {
    pub city_captures: u32,
    pub completed: BTreeMap<String, u32>,
    pub upgraded: BTreeMap<String, u32>,
    pub ranged_attacks: BTreeMap<String, u32>,
    /// Attempted unit shots rejected by resolution for terrain LOS (not AI-filtered targets).
    #[serde(default)]
    pub blocked_ranged_attacks: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyTelemetry {
    /// Actual coalition trigger; unlike leader_* this includes opt-in expansion pressure.
    #[serde(default)]
    pub coalition_eligible_turns: u64,
    #[serde(default)]
    pub coalition_at_war_turns: u64,
    #[serde(default)]
    pub coalition_fought_turns: u64,
    #[serde(default)]
    pub coalition_city_loss_turns: u64,
    /// Turns the actual trigger identifies a different target from the VP-only rule.
    #[serde(default)]
    pub expansion_target_turns: u64,
    #[serde(default)]
    pub alliances_signed: u64,
    #[serde(default)]
    pub alliances_by_pair: BTreeMap<String, u64>,
    /// Alliances signed while a coalition target existed, by target team id.
    #[serde(default)]
    pub alliances_by_target: BTreeMap<String, u64>,
    #[serde(default)]
    pub alliances_broken: u64,
    #[serde(default)]
    pub leader_eligible_turns: u64,
    #[serde(default)]
    pub leader_at_war_turns: u64,
    #[serde(default)]
    pub leader_fought_turns: u64,
    #[serde(default)]
    pub leader_city_loss_turns: u64,
    pub declarations: u64,
    pub war_starts: u64,
    pub peace_offers: u64,
    pub peace_agreements: u64,
}

/// One Sparta seat's observed outcomes. Older reports omit this field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpartaTelemetry {
    pub seat: u8,
    pub nearest_rival_hexes: Option<i32>,
    pub captures: u64,
    pub heals_triggered: u64,
    pub city_count_by_turn: BTreeMap<u32, u32>,
    /// Surviving military regiments after selected resolved turns.
    #[serde(default)]
    pub army_by_turn: BTreeMap<u32, u32>,
    /// Wounded (<50 HP) military unit-turns outside own cities when orders were planned.
    #[serde(default)]
    pub wounded_field_unit_turns: u64,
    /// Of those, no active order or persistent route; a field unit unable to rest-heal.
    #[serde(default)]
    pub wounded_field_idle_unit_turns: u64,
    /// Below the 30 HP march floor, among wounded field regiment unit-turns.
    #[serde(default)]
    pub critical_field_unit_turns: u64,
    #[serde(default)]
    pub critical_field_idle_unit_turns: u64,
    /// Idle wounded regiments within six hexes of a rival city.
    #[serde(default)]
    pub wounded_front_idle_unit_turns: u64,
    /// Military destroyed by combat (disband excluded).
    #[serde(default)]
    pub military_destroyed: u64,
    /// Turn of this seat's first city capture. Older reports omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_capture_turn: Option<u32>,
}

/// Per-seat, observer-only flow measurements for matched faction comparisons.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireFlowTelemetry {
    pub city_count_by_turn: BTreeMap<u32, u32>,
    /// Germany-only samples for the Industry Food rule census; absent in other factions' records.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub population_by_turn: BTreeMap<u32, u32>,
    pub first_war_turn: Option<u32>,
    pub first_capture_turn: Option<u32>,
    pub captures: u32,
    pub eliminated_by: Option<u8>,
    pub war_turns: u32,
    pub peace_turns: u32,
    /// Exact ItemBought and UnitUpgraded event Gold; excludes treaty and upkeep payments.
    pub gold_spent_on_items: i64,
    /// Sum of naturally completed item quotes at turn end; excludes Gold purchases and
    /// incomplete or lost production. Dynamic Settler pricing makes it an estimate.
    pub completed_production_value: i64,
    /// Upkeep owed after each resolved turn while alive, summed (`economy::upkeep`: military
    /// beyond the free units, faction modifiers, companies and treaties). Older reports omit it.
    #[serde(default)]
    pub upkeep_paid: i64,
}

/// Turns after whose resolution a Switzerland seat's treasury is recorded.
const SWISS_TREASURY_TURNS: [u32; 4] = [50, 100, 150, 200];

/// One Switzerland seat's economy after a resolved turn: what its hoard pays as interest, against
/// the Gold its cities earn (tolls included) and its net Gold per turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwissTreasury {
    pub turn: u32,
    pub player: u8,
    pub gold: i32,
    /// Interest the treasury pays on the next turn (`economy::swiss_interest`).
    pub interest: i32,
    /// Gold from the cities' yields, before interest and upkeep.
    pub city_gold: i32,
    /// Net Gold per turn, interest included.
    pub net_gold: i32,
    pub cities: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTelemetry {
    /// VP race observations (leaders, lead changes, decadence, era objectives).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vp_race: Option<crate::vp_race::VpRaceTelemetry>,
    /// None on legacy reports; all initial seats are present in newly observed games.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent_assessment: Option<BTreeMap<u8, legatum_sim::opponent_telemetry::Counters>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparta: Option<SpartaTelemetry>,
    /// Captor seat and resolved turn for each original-capital point.
    #[serde(default)]
    pub capital_vp_turns: Vec<(u8, u32)>,
    /// Capital points in a turn when the same seat also scored a capture Fight objective.
    #[serde(default)]
    pub capital_capture_objective_stacks: u32,
    #[serde(default)]
    pub empire_flow: BTreeMap<u8, EmpireFlowTelemetry>,
    /// Present only for games with barbarians on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barbarians: Option<crate::barbarian_telemetry::BarbarianTelemetry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<crate::proxy_benchmark::Game>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<legatum_sim::diversity::StrategyTelemetry>,
    /// Actual resolved finds by seat, counted once on the first play only.
    #[serde(default)]
    pub ruin_outcomes: BTreeMap<u8, BTreeMap<String, u32>>,
    /// Largest resolved-turn owned city count, including captures. Missing in legacy reports.
    #[serde(default)]
    pub peak_cities: BTreeMap<u8, usize>,
    #[serde(default)]
    pub great_work_turns: BTreeMap<u8, BTreeMap<String, u32>>,
    /// Resolved ItemBought events for national Great Works, by seat.
    #[serde(default)]
    pub great_works_bought: BTreeMap<u8, u32>,
    /// First resolved completion turn per technology; missing means never observed.
    /// Read-only pacing observations, independent of orders, state and hashes.
    #[serde(default)]
    pub research_turns: BTreeMap<u8, BTreeMap<String, u32>>,
    #[serde(default)]
    pub research_eras: BTreeMap<u8, BTreeMap<String, u32>>,
    /// Seat -> era -> resolved turn the seat entered that era (eras never entered are absent).
    #[serde(default)]
    pub era_entries: BTreeMap<u8, BTreeMap<u8, u32>>,
    /// Seat -> resolved turns with techs left but no research set.
    #[serde(default)]
    pub research_idle_turns: BTreeMap<u8, u32>,
    /// Seat -> idle turns that followed another idle turn (a real stall, not a boundary gap).
    #[serde(default)]
    pub research_stall_turns: BTreeMap<u8, u32>,
    /// Airframes completed by this empire. Captured aircraft do not count as new builds.
    #[serde(default)]
    pub aircraft_built: BTreeMap<u8, u32>,
    /// One for each unsupplied aircraft in each resolved turn's final hangars.
    #[serde(default)]
    pub unsupplied_aircraft_turns: BTreeMap<u8, u64>,
    #[serde(default)]
    pub investments: BTreeMap<u8, EmpireInvestments>,
    /// Gold transferred by Swiss-city capture, keyed by captor seat.
    #[serde(default)]
    pub bounty_gold_received: BTreeMap<u8, i64>,
    /// Each living Switzerland seat's treasury and interest after resolving turns 50, 100, 150 and
    /// 200 (`SWISS_TREASURY_TURNS`). Absent on reports from before the uncapped-interest rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub swiss_treasury: Vec<SwissTreasury>,
    /// Every Mercenary Company hired this game, in order (`company`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub company_hires: Vec<CompanyHire>,
    /// Companies that served out their contract, and that turned barbarian unpaid, per employer seat.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub companies_departed: BTreeMap<u8, u32>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub companies_turned: BTreeMap<u8, u32>,
    /// Thin levies (`economy::levy`), per Byzantium seat.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub levies: BTreeMap<u8, LevyTelemetry>,
    /// Rome's Colonia Victrix and Egypt's royal tomb, per Rome or Egypt seat
    /// (`goal_city_telemetry`). Older reports omit it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub goal_cities: BTreeMap<u8, crate::goal_city_telemetry::GoalCityTelemetry>,
    #[serde(default)]
    pub celts_razes: BTreeMap<u8, u64>,
    /// Units disbanded for unpaid upkeep (`rules::cities::pay_gold`), by owner seat.
    #[serde(default)]
    pub upkeep_disbands: BTreeMap<u8, u64>,
    /// Active, nonresisting Germany project city-turns, by output type.
    #[serde(default)]
    pub germany_project_city_turns: BTreeMap<String, u64>,
    /// City-combat observations; None on reports from before they existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city_combat: Option<CityCombatTelemetry>,
    /// Final mean nearest owned city distance ×100 for India seats with at least two cities.
    #[serde(default)]
    pub india_nearest_city_hexes_x100: BTreeMap<u8, i32>,
    /// Final share of India cities linked to at least one other within network radius,
    /// in basis points. Zero includes one-city and eliminated India seats.
    #[serde(default)]
    pub india_linked_city_share_x10000: BTreeMap<u8, i32>,
    /// None on reports produced before diplomacy instrumentation, never an observed zero.
    #[serde(default)]
    pub diplomacy: Option<DiplomacyTelemetry>,
    /// Owned tiles over time, border claims and purchases, treaty costs. None on older reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub borders: Option<crate::border_telemetry::BorderTelemetry>,
    pub battles: u64,
    /// Unique units destroyed in a ranged phase in which a city hit them, by owner.
    /// Includes combined city/unit fire; several city hits count once. None on old reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city_strike_losses: Option<BTreeMap<u8, u64>>,
    pub settler_captures: u64,
    /// None on reports from before promotions existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promotions: Option<PromotionTelemetry>,
    /// Derived from the existing elimination records, not a second event accumulator.
    pub elimination_count: u64,
    /// Final observed state; only finished games enter the terminal-score aggregates.
    pub final_scores: FinalScores,
    pub turn_50: Option<Turn50Snapshot>,
    /// First resolved scoring turn to award each player their permanent goal.
    #[serde(default)]
    pub faction_goal_turns: BTreeMap<u8, u32>,
    /// First Market completion by seat; for India this is its first Revenue Office.
    #[serde(default)]
    pub india_office_first_turn: BTreeMap<u8, u32>,
    /// India seat -> resolved turn -> [non-capital cities with offices, non-capital cities].
    #[serde(default)]
    pub india_office_coverage_by_turn: BTreeMap<u8, BTreeMap<u32, [u32; 2]>>,
    /// VP holds of the game (started, broken, completed). None on reports from before holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub victory_holds: Option<crate::hold_telemetry::VictoryHoldTelemetry>,
    /// All initial empires, including eliminated ones. None means legacy/unavailable data;
    /// individual null turns mean an observed empire never reached that milestone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openings: Option<BTreeMap<u8, EmpireOpening>>,
}

/// A levy-capped empire's armed units against its cap over a game, per resolved turn.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevyTelemetry {
    pub turns: u32,
    /// Sum over turns of armed units and of the cap.
    pub unit_turns: u64,
    pub cap_turns: u64,
    /// Turns the empire stood at its cap.
    pub at_cap_turns: u32,
    /// City-turns a city's production sat on an armed unit the cap blocked.
    pub stalled_city_turns: u32,
    /// Turns the empire was at war while at its cap.
    pub war_at_cap_turns: u32,
}

/// One Mercenary Company hired: who paid whom, for what, and when the contract ends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyHire {
    pub turn: u32,
    pub buyer: u8,
    pub seller: u8,
    pub unit: String,
    pub gold: i32,
    pub until: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreSources {
    /// Held Great Work points, including works captured from another empire.
    #[serde(default)]
    pub great_works: u64,
    pub faction_goals: u64,
    /// VP v2: permanent points for scored Age cards.
    #[serde(default)]
    pub era_objectives: u64,
    #[serde(default)]
    pub capitals_taken: u64,
    /// The core's total, independently checked against the source sum.
    pub total: u64,
}

impl ScoreSources {
    fn for_player(state: &GameState, player: PlayerId) -> Self {
        let vp = scoring::vp(state, player);
        Self {
            great_works: vp
                .iter()
                .filter(|(source, _)| matches!(source, VpSource::GreatWork { .. }))
                .map(|(_, vp)| u64::from(*vp))
                .sum(),
            faction_goals: vp
                .iter()
                .filter(|(source, _)| matches!(source, VpSource::FactionGoal { .. }))
                .map(|(_, vp)| u64::from(*vp))
                .sum(),
            era_objectives: vp
                .iter()
                .filter(|(source, _)| matches!(source, VpSource::EraObjective { .. }))
                .map(|(_, vp)| u64::from(*vp))
                .sum(),
            capitals_taken: vp
                .iter()
                .filter(|(source, _)| {
                    matches!(
                        source,
                        VpSource::CapitalTaken { .. } | VpSource::CapitalSacked { .. }
                    )
                })
                .map(|(_, vp)| u64::from(*vp))
                .sum(),
            total: u64::from(scoring::vp_total(state, player)),
        }
    }

    pub fn source_sum(&self) -> u64 {
        self.great_works + self.faction_goals + self.era_objectives + self.capitals_taken
    }

    fn add(&mut self, other: &Self) {
        self.great_works += other.great_works;
        self.faction_goals += other.faction_goals;
        self.era_objectives += other.era_objectives;
        self.capitals_taken += other.capitals_taken;
        self.total += other.total;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalScores {
    pub players: BTreeMap<u8, ScoreSources>,
    pub teams: BTreeMap<u8, ScoreSources>,
}

impl FinalScores {
    fn of(state: &GameState) -> Self {
        let players: BTreeMap<_, _> = state
            .players
            .iter()
            .map(|p| (p.id.0, ScoreSources::for_player(state, p.id)))
            .collect();
        let teams = state
            .teams()
            .into_iter()
            .map(|team| {
                let mut sources = ScoreSources::default();
                for member in state.team_members(team) {
                    sources.add(&players[&member.0]);
                }
                sources.total = u64::from(scoring::team_vp(state, team));
                (team.0, sources)
            })
            .collect();
        Self { players, teams }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct EventCount {
    pub total: u64,
    pub games_with_events: u64,
    pub observed_games: u64,
}

impl EventCount {
    fn observe(&mut self, count: u64) {
        self.total += count;
        self.games_with_events += u64::from(count > 0);
        self.observed_games += 1;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct EventSummary {
    pub declarations: EventCount,
    pub peace_offers: EventCount,
    pub peace_agreements: EventCount,
    pub battles: EventCount,
    pub city_captures: EventCount,
    pub settler_captures: EventCount,
    pub eliminations: EventCount,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ScorePopulation {
    pub observations: u64,
    pub vp: ScoreSources,
    pub reconciliation_failures: u64,
}

impl ScorePopulation {
    fn observe(&mut self, sources: &ScoreSources) {
        self.observations += 1;
        self.vp.add(sources);
        self.reconciliation_failures += u64::from(sources.source_sum() != sources.total);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ScoreSummary {
    pub finished_games: u64,
    pub all_players: ScorePopulation,
    pub all_teams: ScorePopulation,
    pub winning_teams: ScorePopulation,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct LeaderCohort {
    pub eventual_wins: u64,
    pub eligible_games: u64,
    pub draws: u64,
}

impl LeaderCohort {
    fn observe(&mut self, leaders: &[u8], winner: Option<u8>) {
        self.eligible_games += 1;
        self.eventual_wins += u64::from(winner.is_some_and(|t| leaders.contains(&t)));
        self.draws += u64::from(winner.is_none());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SnowballSummary {
    pub ended_before_turn_50: u64,
    pub excluded_unfinished_games: u64,
    pub excluded_snapshots_without_teams: u64,
    pub unique_leader: LeaderCohort,
    pub tied_leaders: LeaderCohort,
    pub any_leader: LeaderCohort,
}

/// The caller supplies existing canonical counts and outcomes; no second capture accumulator.
pub struct Sample<'a> {
    pub telemetry: Option<&'a GameTelemetry>,
    pub city_captures: u32,
    pub eliminations: usize,
    pub finished: bool,
    pub winner: Option<u8>,
    /// Seats of the winning team.
    pub winner_players: &'a [u8],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TelemetrySummary {
    pub opponent_assessment_games: u64,
    pub opponent_assessment: legatum_sim::opponent_telemetry::Counters,
    pub sample_games: u64,
    pub excluded_games_without_telemetry: u64,
    pub era_pacing: EraPacing,
    pub events: EventSummary,
    pub terminal_scores: ScoreSummary,
    pub turn_50: SnowballSummary,
}

/// When empires enter each era, pooled over observed games, for pacing validation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct EraPacing {
    /// Seats observed (every initial empire of every observed game).
    pub seats: u64,
    /// Entry turns of every empire that entered, by era (era 0 is the start and never entered).
    pub entry_turns: BTreeMap<u8, Vec<u32>>,
    /// Winning empires' entry turn minus the game's median entry turn, by era, in games that
    /// finished (negative means the winner got there first). Empires that never entered an era
    /// are left out of that era's median.
    pub winner_vs_median: BTreeMap<u8, Vec<i32>>,
    /// Technology name -> resolved turn each empire that learned it did, pooled over games.
    pub tech_turns: BTreeMap<String, Vec<u32>>,
    /// Resolved turns with techs left but no research set, and the stalled part of them.
    pub idle_turns: u64,
    pub stall_turns: u64,
}

impl EraPacing {
    fn observe(&mut self, metrics: &GameTelemetry, finished: bool, winners: &[u8]) {
        self.seats += metrics.research_eras.len() as u64;
        self.idle_turns += metrics
            .research_idle_turns
            .values()
            .map(|n| u64::from(*n))
            .sum::<u64>();
        self.stall_turns += metrics
            .research_stall_turns
            .values()
            .map(|n| u64::from(*n))
            .sum::<u64>();
        for techs in metrics.research_turns.values() {
            for (tech, turn) in techs {
                self.tech_turns.entry(tech.clone()).or_default().push(*turn);
            }
        }
        let mut by_era: BTreeMap<u8, Vec<(u8, u32)>> = BTreeMap::new();
        for (seat, eras) in &metrics.era_entries {
            for (era, turn) in eras {
                by_era.entry(*era).or_default().push((*seat, *turn));
            }
        }
        for (era, mut entries) in by_era {
            self.entry_turns
                .entry(era)
                .or_default()
                .extend(entries.iter().map(|(_, t)| *t));
            if !finished {
                continue;
            }
            entries.sort_by_key(|(_, t)| *t);
            let median = entries[entries.len() / 2].1 as i32;
            for (seat, turn) in &entries {
                if winners.contains(seat) {
                    self.winner_vs_median
                        .entry(era)
                        .or_default()
                        .push(*turn as i32 - median);
                }
            }
        }
    }

    pub fn render_markdown(&self) -> String {
        let mut out = String::from("\n## Era pacing\n\n");
        out += "Resolved turn each empire entered each era. `Winner vs median` is the winning team's seats' entry turn minus the median entry of the empires that entered it in the same finished game (negative: winners got there first).\n\n";
        out += "| Era | Entered / seats | Fastest | p10 | Median | p90 | Slowest | Winner vs median (mean, n) |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n";
        for era in 1..legatum_core::eras::ERA_COUNT as u8 {
            let mut turns = self.entry_turns.get(&era).cloned().unwrap_or_default();
            turns.sort_unstable();
            let cell = |p: usize| {
                crate::stats::percentile(&turns, p).map_or("n/a".into(), |t| t.to_string())
            };
            let delta = self.winner_vs_median.get(&era);
            let delta_cell = match delta {
                Some(d) if !d.is_empty() => format!(
                    "{:+.1} ({})",
                    d.iter().map(|x| f64::from(*x)).sum::<f64>() / d.len() as f64,
                    d.len()
                ),
                _ => "n/a".into(),
            };
            out += &format!(
                "| {} | {}/{} | {} | {} | {} | {} | {} | {} |\n",
                legatum_core::eras::era_name(era),
                turns.len(),
                self.seats,
                cell(0),
                cell(10),
                cell(50),
                cell(90),
                cell(100),
                delta_cell
            );
        }
        out += &format!(
            "\nResearch idle turns (techs left, no research set): {}, of which stalled (two or more in a row): {}.\n",
            self.idle_turns, self.stall_turns
        );
        out += &self.render_tech_calendar();
        out
    }

    /// The designers' tech-pace gauge: the median turn and display year each technology is
    /// learned, beside the year it really arrived (`calendar::tech_real_year`).
    fn render_tech_calendar(&self) -> String {
        use legatum_core::calendar::{calendar_label, format_year, tech_real_year};
        let mut out = String::from("\n## Tech pace against the calendar\n\n");
        out += "Median resolved turn each technology was learned, the display year of that turn (`calendar::calendar_label`) and the year the technology really arrived (n/a when no single year is established). Empires that never learned it are left out.\n\n";
        out += "| Era | Technology | Learned / seats | Median turn | Median year | Real year |\n| --- | --- | --- | --- | --- | --- |\n";
        for tech in legatum_core::TechId::ALL {
            let def = legatum_core::eras::tech_def(tech);
            let mut turns = self
                .tech_turns
                .get(&format!("{tech:?}"))
                .cloned()
                .unwrap_or_default();
            turns.sort_unstable();
            let (turn, year) = match crate::stats::percentile(&turns, 50) {
                Some(t) => (t.to_string(), calendar_label(t)),
                None => ("n/a".into(), "n/a".into()),
            };
            out += &format!(
                "| {} | {} | {}/{} | {} | {} | {} |\n",
                legatum_core::eras::era_name(def.era),
                def.name,
                turns.len(),
                self.seats,
                turn,
                year,
                tech_real_year(tech).map_or("n/a".into(), format_year)
            );
        }
        out
    }
}

impl TelemetrySummary {
    pub fn observe(&mut self, sample: Sample<'_>) {
        let Some(metrics) = sample.telemetry else {
            self.excluded_games_without_telemetry += 1;
            return;
        };
        self.era_pacing
            .observe(metrics, sample.finished, sample.winner_players);
        if let Some(seats) = &metrics.opponent_assessment {
            self.opponent_assessment_games += 1;
            for counters in seats.values() {
                self.opponent_assessment.add(counters);
            }
        }
        if let Some(d) = &metrics.diplomacy {
            self.events.declarations.observe(d.declarations);
            self.events.peace_offers.observe(d.peace_offers);
            self.events.peace_agreements.observe(d.peace_agreements);
        }
        self.sample_games += 1;
        self.events.battles.observe(metrics.battles);
        self.events
            .city_captures
            .observe(u64::from(sample.city_captures));
        self.events
            .settler_captures
            .observe(metrics.settler_captures);
        self.events.eliminations.observe(sample.eliminations as u64);
        if !sample.finished {
            self.turn_50.excluded_unfinished_games += 1;
            return;
        }
        let scores = &mut self.terminal_scores;
        scores.finished_games += 1;
        for sources in metrics.final_scores.players.values() {
            scores.all_players.observe(sources);
        }
        for (team, sources) in &metrics.final_scores.teams {
            scores.all_teams.observe(sources);
            if Some(*team) == sample.winner {
                scores.winning_teams.observe(sources);
            }
        }
        let snow = &mut self.turn_50;
        let Some(snapshot) = &metrics.turn_50 else {
            snow.ended_before_turn_50 += 1;
            return;
        };
        match snapshot.leaders.len() {
            0 => {
                snow.excluded_snapshots_without_teams += 1;
                return;
            }
            1 => snow.unique_leader.observe(&snapshot.leaders, sample.winner),
            _ => snow.tied_leaders.observe(&snapshot.leaders, sample.winner),
        }
        snow.any_leader.observe(&snapshot.leaders, sample.winner);
    }

    pub fn scores_reconcile(&self) -> bool {
        self.terminal_scores.all_players.reconciliation_failures == 0
            && self.terminal_scores.all_teams.reconciliation_failures == 0
    }
}

fn fraction(n: u64, d: u64) -> String {
    if d == 0 {
        format!("{n}/{d} (n/a)")
    } else {
        format!("{n}/{d} ({:.1}%)", 100.0 * n as f64 / d as f64)
    }
}

pub fn render_markdown(s: &TelemetrySummary) -> String {
    let mut out = String::from("\n## Battle and capture telemetry\n\n");
    out += &format!(
        "{} observed first plays; {} records without telemetry excluded.\n\n",
        s.sample_games, s.excluded_games_without_telemetry
    );
    out += "| Event | Total / observed games | Mean per game | Games with event / observed games |\n| --- | --- | --- | --- |\n";
    for (label, count) in [
        ("War declarations", &s.events.declarations),
        ("Peace offers", &s.events.peace_offers),
        ("Peace agreements", &s.events.peace_agreements),
        ("Battles", &s.events.battles),
        ("City captures", &s.events.city_captures),
        ("Settler captures", &s.events.settler_captures),
        ("Eliminations", &s.events.eliminations),
    ] {
        let mean = if count.observed_games == 0 {
            "n/a".into()
        } else {
            format!("{:.3}", count.total as f64 / count.observed_games as f64)
        };
        out += &format!(
            "| {label} | {}/{} | {mean} | {} |\n",
            count.total,
            count.observed_games,
            fraction(count.games_with_events, count.observed_games)
        );
    }
    out += "\n## Terminal VP sources\n\n";
    let scores = &s.terminal_scores;
    out += &format!(
        "{} finished games. Columns are separate populations; totals across columns must not be added. Every cell is VP / population observations.\n\n",
        scores.finished_games
    );
    out += "| Source | All players | All teams | Winning teams |\n| --- | --- | --- | --- |\n";
    let populations = [
        &scores.all_players,
        &scores.all_teams,
        &scores.winning_teams,
    ];
    let mut row = |label: &str, value: &dyn Fn(&ScoreSources) -> u64| {
        out += &format!("| {label} |");
        for p in populations {
            out += &format!(" {}/{} |", value(&p.vp), p.observations);
        }
        out += "\n";
    };
    row("Held Great Works", &|vp| vp.great_works);
    row("Faction goals", &|vp| vp.faction_goals);
    row("Era objectives", &|vp| vp.era_objectives);
    row("Capitals taken", &|vp| vp.capitals_taken);
    row("Total VP", &|vp| vp.total);
    out += &format!(
        "\nReconciliation failures: {} player observations, {} team observations.\n",
        scores.all_players.reconciliation_failures, scores.all_teams.reconciliation_failures
    );
    out += "\n## Turn-50 team leaders\n\n";
    let snow = &s.turn_50;
    out += "| Cohort | Eventual leader-set wins / eligible games | Draws (included) |\n| --- | --- | --- |\n";
    for (label, cohort) in [
        ("Unique leader", &snow.unique_leader),
        ("Tied leaders (winner in set)", &snow.tied_leaders),
        ("Any leader set", &snow.any_leader),
    ] {
        out += &format!(
            "| {label} | {} | {} |\n",
            fraction(cohort.eventual_wins, cohort.eligible_games),
            cohort.draws
        );
    }
    out += &format!(
        "\nExcluded: {} games ended before turn 50, {} unfinished games, {} snapshots without teams, {} records without telemetry.\n",
        snow.ended_before_turn_50,
        snow.excluded_unfinished_games,
        snow.excluded_snapshots_without_teams,
        s.excluded_games_without_telemetry
    );
    out += &s.era_pacing.render_markdown();
    out += "\n## Metric definitions\n\n";
    for definition in [
        DEFINITIONS.battles,
        DEFINITIONS.captures,
        DEFINITIONS.eliminations,
        DEFINITIONS.scores,
        DEFINITIONS.turn_50,
        DEFINITIONS.openings,
        DEFINITIONS.victory_holds,
        DEFINITIONS.denominators,
    ] {
        out += definition;
        out += "\n\n";
    }
    out += "Definitions reuse [balance-probe-1](../analysis/balance-probe-1.md) and [the probe's definitions](../../tools/analysis/README.md); this report is a fresh simulation sample, not a rerun of those campaigns.\n";
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::game::blank_game;
    use legatum_core::state::Map;
    use legatum_core::{Faction, GameSetup, TeamId, Terrain};

    #[test]
    fn isolated_start_distance_uses_initial_capitals() {
        let state = legatum_core::new_game(&GameSetup::all_ai(5300001));
        assert_eq!(state.cities.len(), state.players.len());
        let mut collector = Collector::default();
        collector.observe_initial(&state);
        assert!(
            collector
                .sparta
                .unwrap()
                .nearest_rival_hexes
                .is_some_and(|d| d > 0)
        );
    }

    fn state() -> GameState {
        let mut setup = GameSetup::all_ai(1);
        setup.players.truncate(3);
        setup.players[0].team = TeamId(7);
        setup.players[1].team = TeamId(7);
        let mut state = blank_game(&setup, Map::new(2, 2, Terrain::Grassland));
        // Most telemetry fixtures toggle the completed-goal flag; Athens has a permanent
        // goal, whereas India's Diwan is scored from its city holder.
        for player in &mut state.players {
            player.faction = Faction::Athens;
        }
        state
    }

    fn snapshot(state: &mut GameState) -> GameTelemetry {
        let mut collector = Collector::default();
        state.turn = 51;
        collector.observe_resolved_turn(50, state);
        collector.finish(state, 0)
    }

    #[test]
    fn great_work_purchase_count_excludes_natural_completion() {
        use legatum_core::{BuildingKind, CityId};
        let item = ProductionItem::Building {
            building: BuildingKind::GreatLibrary,
        };
        let mut collector = Collector::default();
        collector.observe_event(
            1,
            &Event::ProductionCompleted {
                city: CityId(0),
                owner: PlayerId(0),
                item,
            },
        );
        assert!(collector.finish(&state(), 0).great_works_bought.is_empty());
        collector.observe_event(
            2,
            &Event::ItemBought {
                city: CityId(0),
                owner: PlayerId(0),
                item,
                gold: 525,
            },
        );
        assert_eq!(collector.finish(&state(), 0).great_works_bought[&0], 1);
    }

    #[test]
    fn city_strike_losses_deduplicate_simultaneous_shots_and_exclude_survivors() {
        let shot = |city, unit, hp| Event::Battle {
            attacker: legatum_core::events::Attacker::City {
                city: legatum_core::CityId(city),
            },
            attacker_owner: PlayerId(0),
            attacker_at: None,
            defender: legatum_core::events::Defender::Unit {
                unit: legatum_core::UnitId(unit),
                owner: PlayerId(2),
                unit_kind: UnitKind::Spearman,
            },
            at: legatum_core::Hex::new(0, 0),
            attacker_strength: Default::default(),
            defender_strength: Default::default(),
            damage_to_attacker: 0,
            damage_to_defender: 30,
            attacker_roll: 100,
            defender_roll: 0,
            attacker_hp_after: 200,
            defender_hp_after: hp,
            ranged: true,
            meeting: false,
            air_strike: false,
            defender_leaving: false,
        };
        let mut collector = Collector::default();
        for event in [shot(1, 1, 0), shot(2, 1, 0), shot(2, 2, 70), shot(3, 3, 0)] {
            collector.observe_event(1, &event);
        }
        assert_eq!(
            collector.finish(&state(), 0).city_strike_losses,
            Some(BTreeMap::from([(2, 2)]))
        );
    }

    fn add(s: &mut TelemetrySummary, metrics: &GameTelemetry, finished: bool, winner: Option<u8>) {
        s.observe(Sample {
            telemetry: Some(metrics),
            city_captures: 0,
            eliminations: 0,
            finished,
            winner,
            winner_players: &[],
        });
    }

    #[test]
    fn openings_keep_first_completions_and_count_foundings_despite_captures_or_elimination() {
        use legatum_core::{CityId, Hex, UnitId};

        let mut state = state();
        let mut collector = Collector::default();
        let founding = |id| Event::CityFounded {
            city: CityId(id),
            owner: PlayerId(0),
            name: format!("City {id}"),
            at: Hex::new(0, 0),
            veteran: false,
        };
        let completion = |unit| Event::ProductionCompleted {
            city: CityId(0),
            owner: PlayerId(0),
            item: ProductionItem::Unit { unit },
        };
        collector.observe_event(1, &founding(0));
        collector.observe_event(
            2,
            &Event::CityCaptured {
                city: CityId(0),
                from: PlayerId(0),
                to: PlayerId(1),
                bounty: None,
            },
        );
        collector.observe_event(
            3,
            &Event::SettlerCaptured {
                unit: UnitId(0),
                from: PlayerId(1),
                to: PlayerId(0),
                at: Hex::new(0, 0),
            },
        );
        collector.observe_event(
            4,
            &Event::ItemBought {
                city: CityId(0),
                owner: PlayerId(0),
                item: ProductionItem::Unit {
                    unit: UnitKind::Spearman,
                },
                gold: 100,
            },
        );
        collector.observe_event(5, &completion(UnitKind::Scout));
        let early = collector.finish(&state, 0).openings.unwrap();
        assert_eq!(early[&0].first_city_turn, Some(1));
        assert_eq!(early[&0].first_soldier_turn, None);
        assert_eq!(early[&0].first_settler_turn, None);

        collector.observe_event(12, &completion(UnitKind::Spearman));
        collector.observe_event(21, &completion(UnitKind::Settler));
        collector.observe_event(23, &founding(1));
        collector.observe_event(24, &completion(UnitKind::Spearman));
        collector.observe_event(30, &founding(2));
        collector.observe_event(33, &completion(UnitKind::Settler));
        state.players[0].alive = false;
        let openings = collector.finish(&state, 1).openings.unwrap();
        assert_eq!(openings.len(), 3);
        assert_eq!(
            openings[&0],
            EmpireOpening {
                first_city_turn: Some(1),
                first_soldier_turn: Some(12),
                first_settler_turn: Some(21),
                second_city_turn: Some(23),
                ..Default::default()
            }
        );
        assert_eq!(openings[&1], EmpireOpening::default());
        assert_eq!(openings[&2], EmpireOpening::default());
    }

    #[test]
    fn score_sources_reconcile_permanent_points() {
        let mut state = state();
        // Permanent goals score from the flag; Byzantium's held goal needs its Heir.
        state.players[0].faction = Faction::Athens;
        state.players[1].faction = Faction::Athens;
        state.players[0].faction_goal_done = true;
        // Elimination does not erase banked score in core scoring.
        state.players[1].alive = false;
        state.players[1].faction_goal_done = true;
        // Player 0 holds the original capital of player 2 (another team): 1 VP while held.
        let settler = legatum_core::game::spawn_unit(
            &mut state,
            PlayerId(2),
            legatum_core::UnitKind::Settler,
            legatum_core::Hex::from_offset(0, 0),
        );
        legatum_core::rules::cities::found_city(&mut state, settler, &mut Vec::new()).unwrap();
        for city in state.cities.values_mut() {
            city.owner = PlayerId(0);
        }
        let collector = Collector::default();
        let scores = collector.finish(&state, 0).final_scores;
        assert_eq!(scores.players[&0].total, 3);
        assert_eq!(scores.players[&0].capitals_taken, 1);
        assert_eq!(scores.players[&1].total, 2);
        assert_eq!(scores.teams[&7].total, 5);
        assert_eq!(scores.teams[&7].capitals_taken, 1);
        assert_eq!(scores.teams[&7].faction_goals, 4);
        for (player, sources) in scores.players {
            assert_eq!(sources.source_sum(), sources.total);
            assert_eq!(
                sources.total,
                u64::from(scoring::vp_total(&state, PlayerId(player)))
            );
        }
        for (team, sources) in scores.teams {
            assert_eq!(sources.source_sum(), sources.total);
            assert_eq!(
                sources.total,
                u64::from(scoring::team_vp(&state, TeamId(team)))
            );
        }
    }

    #[test]
    fn captured_great_work_points_reconcile_for_the_holder() {
        use legatum_core::{BuildingKind, CityId, landmarks::Landmark};
        let mut state = state();
        state.players[0].great_work_completed = Some(BuildingKind::GreatLibrary);
        state.map.tiles[0].owner = Some(PlayerId(1));
        state.map.tiles[0].landmark = Some(Landmark {
            building: BuildingKind::GreatLibrary,
            city: CityId(0),
            of: PlayerId(0),
        });
        let holder = ScoreSources::for_player(&state, PlayerId(1));
        assert_eq!(holder.great_works, 2);
        assert_eq!(holder.source_sum(), holder.total);
        assert_eq!(ScoreSources::for_player(&state, PlayerId(0)).great_works, 0);
    }

    #[test]
    fn two_player_team_is_ranked_and_credited_once() {
        let mut state = state();
        for p in &mut state.players {
            p.faction = Faction::Athens;
        }
        for p in &mut state.players {
            p.faction_goal_done = true;
        }
        let metrics = snapshot(&mut state);
        let turn50 = metrics.turn_50.as_ref().unwrap();
        assert_eq!(turn50.team_vp, BTreeMap::from([(2, 2), (7, 4)]));
        assert_eq!(turn50.leaders, [7]); // Highest individual is player 2, but team 7 leads.
        let mut s = TelemetrySummary::default();
        add(&mut s, &metrics, true, Some(7));
        assert_eq!(s.terminal_scores.all_players.observations, 3);
        assert_eq!(s.terminal_scores.all_teams.observations, 2);
        assert_eq!(s.terminal_scores.all_players.vp.total, 6);
        assert_eq!(s.terminal_scores.all_teams.vp.total, 6);
        assert_eq!(s.terminal_scores.winning_teams.observations, 1);
        assert_eq!(s.terminal_scores.winning_teams.vp.total, 4);
        assert_eq!(s.turn_50.unique_leader.eventual_wins, 1);
        assert_eq!(s.turn_50.unique_leader.eligible_games, 1);
    }

    #[test]
    fn swiss_treasury_is_recorded_on_the_listed_turns_only() {
        let mut state = state();
        state.players[0].faction = Faction::Switzerland;
        state.players[0].gold = 1_250;
        state.players[1].faction = Faction::India;
        state.players[1].gold = 9_000;
        let mut collector = Collector::default();
        for turn in [99, 100, 101, 150] {
            state.turn = turn + 1;
            collector.observe_resolved_turn(turn, &state);
        }
        let rows = collector.finish(&state, 0).swiss_treasury;
        assert_eq!(rows.iter().map(|r| r.turn).collect::<Vec<_>>(), [100, 150]);
        assert!(rows.iter().all(|r| r.player == 0 && r.gold == 1_250));
        assert!(rows.iter().all(|r| r.interest == 12));
    }

    #[test]
    fn turn_50_is_after_scoring_and_not_replaced_by_later_turns() {
        let mut state = state();
        let mut collector = Collector::default();
        state.turn = 50;
        collector.observe_resolved_turn(49, &state);
        assert!(collector.finish(&state, 0).turn_50.is_none());
        state.players[0].faction_goal_done = true;
        state.turn = 51;
        collector.observe_resolved_turn(50, &state);
        state.players[0].faction_goal_done = false;
        state.turn = 76;
        collector.observe_resolved_turn(75, &state);
        let snapshot = collector.finish(&state, 0).turn_50.unwrap();
        assert_eq!((snapshot.resolved_turn, snapshot.next_turn), (50, 51));
        assert_eq!(snapshot.team_vp[&7], 2);
        assert_eq!(snapshot.leaders, [7]);
    }

    #[test]
    fn tied_leaders_keep_the_whole_team_set_without_id_tiebreaks() {
        let mut state = state();
        for p in &mut state.players {
            p.faction = Faction::Athens;
        }
        state.players[0].faction_goal_done = true;
        state.players[2].faction_goal_done = true;
        let metrics = snapshot(&mut state);
        assert_eq!(metrics.turn_50.as_ref().unwrap().leaders, [2, 7]);
        let mut s = TelemetrySummary::default();
        add(&mut s, &metrics, true, Some(7));
        add(&mut s, &metrics, true, Some(2));
        assert_eq!(s.turn_50.unique_leader.eligible_games, 0);
        assert_eq!(s.turn_50.tied_leaders.eligible_games, 2);
        assert_eq!(s.turn_50.tied_leaders.eventual_wins, 2);
        assert_eq!(s.turn_50.any_leader.eventual_wins, 2);
    }

    #[test]
    fn games_ending_before_turn_50_are_excluded_but_turn_50_finishes_count() {
        let mut state = state();
        state.finished = true;
        state.winner = Some(TeamId(7));
        state.turn = 50; // Turn 49 has just resolved.
        let mut collector = Collector::default();
        collector.observe_resolved_turn(49, &state);
        let mut s = TelemetrySummary::default();
        add(&mut s, &collector.finish(&state, 0), true, Some(7));
        assert_eq!(s.turn_50.ended_before_turn_50, 1);
        assert_eq!(s.turn_50.any_leader.eligible_games, 0);
        add(&mut s, &snapshot(&mut state), true, Some(7));
        assert_eq!(s.turn_50.any_leader.eligible_games, 1);
        assert_eq!(s.turn_50.any_leader.eventual_wins, 1);
        assert_eq!(s.terminal_scores.finished_games, 2);
    }

    #[test]
    fn era_entries_are_recorded_and_winners_compared_with_the_median_empire() {
        let mut collector = Collector::default();
        for (turn, seat, era) in [(30, 0, 2), (40, 1, 2), (50, 2, 2), (45, 0, 3)] {
            collector.observe_event(
                turn,
                &Event::EraEntered {
                    player: PlayerId(seat),
                    era,
                    first: false,
                },
            );
        }
        let metrics = collector.finish(&state(), 0);
        assert_eq!(metrics.era_entries[&0], BTreeMap::from([(1, 30), (2, 45)]));
        let mut s = TelemetrySummary::default();
        s.observe(Sample {
            telemetry: Some(&metrics),
            city_captures: 0,
            eliminations: 0,
            finished: true,
            winner: Some(0),
            winner_players: &[0],
        });
        // Era 1 entries 30, 40, 50: median 40, winner 30. Era 2: only the winner entered.
        assert_eq!(s.era_pacing.winner_vs_median[&1], vec![-10]);
        assert_eq!(s.era_pacing.winner_vs_median[&2], vec![0]);
        assert!(s.era_pacing.render_markdown().contains("| Classical | 3/"));
        assert!(
            s.era_pacing
                .render_markdown()
                .contains("| Machine Age | Flight | 0/3 | n/a | n/a | 1903 AD |")
        );
    }

    #[test]
    fn a_stall_is_two_idle_turns_in_a_row_not_one_boundary_gap() {
        let mut collector = Collector::default();
        let mut st = state();
        st.players[0].researching = None;
        for turn in 1..=3 {
            collector.observe_resolved_turn(turn, &st);
        }
        st.players[0].researching = Some(legatum_core::TechId::Irrigation);
        collector.observe_resolved_turn(4, &st);
        st.players[0].researching = None;
        collector.observe_resolved_turn(5, &st);
        let m = collector.finish(&st, 0);
        assert_eq!(m.research_idle_turns[&0], 4);
        assert_eq!(m.research_stall_turns[&0], 2);
    }

    #[test]
    fn draws_stay_in_each_cohort_and_unfinished_games_have_no_eventual_outcome() {
        let mut state = state();
        let tied = snapshot(&mut state);
        state.players[0].faction_goal_done = true;
        let unique = snapshot(&mut state);
        let mut s = TelemetrySummary::default();
        add(&mut s, &tied, true, None);
        add(&mut s, &unique, true, None);
        add(&mut s, &unique, true, Some(2)); // Unique leader loses.
        add(&mut s, &unique, false, None);
        s.observe(Sample {
            telemetry: None,
            city_captures: 9,
            eliminations: 5,
            finished: false,
            winner: None,
            winner_players: &[],
        });
        assert_eq!(
            (
                s.turn_50.tied_leaders.eligible_games,
                s.turn_50.tied_leaders.draws
            ),
            (1, 1)
        );
        assert_eq!(
            (
                s.turn_50.unique_leader.eligible_games,
                s.turn_50.unique_leader.draws
            ),
            (2, 1)
        );
        assert_eq!(
            (
                s.turn_50.any_leader.eventual_wins,
                s.turn_50.any_leader.eligible_games,
                s.turn_50.any_leader.draws
            ),
            (0, 3, 2)
        );
        assert_eq!(s.turn_50.excluded_unfinished_games, 1);
        assert_eq!(s.excluded_games_without_telemetry, 1);
        assert_eq!(s.events.city_captures.observed_games, 4);
        assert_eq!(s.events.city_captures.total, 0);
        assert_eq!(s.terminal_scores.finished_games, 3);
        assert_eq!(s.terminal_scores.winning_teams.observations, 1);
    }

    #[test]
    fn empty_sample_has_explicit_zero_denominators_without_nan() {
        let s = TelemetrySummary::default();
        assert_eq!(s.sample_games, 0);
        assert_eq!(s.events.battles.observed_games, 0);
        assert_eq!(s.terminal_scores.winning_teams.observations, 0);
        assert!(s.scores_reconcile());
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("NaN"));
        let md = render_markdown(&s);
        assert!(md.contains("0/0 (n/a)"));
        assert!(!md.contains("NaN"));
    }

    #[test]
    fn goal_payoffs_reconcile_with_the_core_score() {
        let mut s = state();
        for (player, faction, expected) in [(0, Faction::Germany, 2), (1, Faction::Athens, 2)] {
            s.players[player].faction = faction;
            s.players[player].faction_goal_done = true;
            let sources = ScoreSources::for_player(&s, PlayerId(player as u8));
            assert_eq!(sources.faction_goals, expected);
            assert_eq!(sources.source_sum(), sources.total);
        }
        let mut summary = TelemetrySummary::default();
        add(
            &mut summary,
            &Collector::default().finish(&s, 0),
            true,
            None,
        );
        assert!(summary.scores_reconcile());
    }

    #[test]
    fn serialization_is_stable_and_observation_does_not_change_state() {
        let mut state = state();
        state.players[0].faction_goal_done = true;
        state.turn = 51;
        let hash = state.hash();
        let first = snapshot(&mut state);
        let mut collector = Collector::default();
        collector.observe_resolved_turn(50, &state);
        let second = collector.finish(&state, 0);
        let bytes = serde_json::to_vec(&first).unwrap();
        assert_eq!(bytes, serde_json::to_vec(&second).unwrap());
        assert_eq!(
            serde_json::from_slice::<GameTelemetry>(&bytes).unwrap(),
            first
        );
        assert_eq!(state.hash(), hash);
        let (mut a, mut b) = (TelemetrySummary::default(), TelemetrySummary::default());
        add(&mut a, &first, true, Some(7));
        add(&mut a, &second, true, None);
        add(&mut b, &second, true, None);
        add(&mut b, &first, true, Some(7));
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert_eq!(render_markdown(&a), render_markdown(&b));
    }

    #[test]
    fn reconciliation_failures_are_visible() {
        let mut metrics = Collector::default().finish(&state(), 0);
        metrics.final_scores.players.get_mut(&0).unwrap().total += 1;
        metrics.final_scores.teams.get_mut(&7).unwrap().total += 1;
        let mut s = TelemetrySummary::default();
        add(&mut s, &metrics, true, Some(7));
        assert!(!s.scores_reconcile());
        assert_eq!(s.terminal_scores.all_players.reconciliation_failures, 1);
        assert_eq!(s.terminal_scores.all_teams.reconciliation_failures, 1);
    }
}
