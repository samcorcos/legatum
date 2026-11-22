//! Hand-built PlayerViews for tests (the real `player_view` lives in core and is built
//! elsewhere). Player 0 is "me" on team 0, player 1 is a teammate on team 0, players 2 and 3 are
//! enemies on teams 2 and 3.

use legatum_core::data;
use legatum_core::economy::CityYields;
use legatum_core::view::*;
use legatum_core::*;

/// A stable personality for hand-built AI tests. These fixtures exercise one decision at a
/// time; their unrelated behavior must not move when a faction's climbed defaults are banked.
pub const TEST_KNOBS: crate::Knobs = crate::Knobs::new(55, 40, 45, 80, 35);

/// Preserve the audited behavior for focused decision tests. Production defaults are gated
/// separately; these fixtures assert what each decision does when its gene is enabled.
fn test_decisions() -> crate::DecisionParams {
    crate::DecisionParams {
        modernization_value: 0,
        modernization_upgrade: 0,
        modernization_return: 0,
        peace_exposure_percent: 100,
        threat_horizon_turns: 2,
        assessment_interval: 1,
        fog_escort_percent: 85,
        army_threat_weight: 100,
        field_strike: 100,
        scout_threat: 100,
        // Existing fixtures predate the capitals victory; its tests turn the gene on.
        capitals: 0,
        walled_defense_percent: 100,
        intel_value: 100,
        sentry_posts: 100,
        counter_composition: 100,
        production_threat_agreement: 0,
        era_transition_scoring: 1,
        settlement_claim_timeline: 1,
        ..crate::DecisionParams::default()
    }
}

pub fn planner(view: &PlayerView, seed: u64) -> crate::scripted::Planner<'_> {
    let mut p = crate::scripted::Planner::new(view, TEST_KNOBS, view.faction_of(view.me), seed);
    p.decisions = test_decisions();
    p
}

pub fn brain(view: &PlayerView, seed: u64, difficulty: AiDifficulty) -> crate::ScriptedBrain {
    let faction = view.faction_of(view.me);
    let knobs = crate::difficulty::scale_knobs(TEST_KNOBS, difficulty);
    let mut brain = crate::ScriptedBrain::with_knobs(faction, seed, view.me, knobs);
    brain.base = TEST_KNOBS;
    brain.difficulty = difficulty;
    brain.decisions = test_decisions();
    brain
}

pub const ME: PlayerId = PlayerId(0);
pub const MATE: PlayerId = PlayerId(1);
pub const ENEMY: PlayerId = PlayerId(2);

/// The five ordinary building families, for fixtures that need a developed city.
pub const ORDINARY_BUILDINGS: [BuildingKind; 5] = [
    BuildingKind::Granary,
    BuildingKind::Workshop,
    BuildingKind::Market,
    BuildingKind::Laboratory,
    BuildingKind::Walls,
];

/// The full research queue ordered, if any.
pub fn research_queue(orders: &[Order]) -> Option<Vec<TechId>> {
    orders.iter().find_map(|o| match o {
        Order::SetResearchQueue { techs } => Some(techs.clone()),
        _ => None,
    })
}

/// The technology a set of orders researches first: a `SetResearch`, or the head of a
/// `SetResearchQueue`.
pub fn research_head(orders: &[Order]) -> Option<TechId> {
    orders.iter().find_map(|o| match o {
        Order::SetResearch { tech } => Some(*tech),
        Order::SetResearchQueue { techs } => techs.first().copied(),
        _ => None,
    })
}

pub fn off(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

/// Set a player's public army strength in `view`.
pub fn set_army(view: &mut PlayerView, player: PlayerId, value: i32) {
    for p in view.players.iter_mut().filter(|p| p.id == player) {
        p.army = value;
    }
}

/// Set several players' public army strengths in `view`.
pub fn set_armies(view: &mut PlayerView, values: &[(PlayerId, i32)]) {
    for (player, value) in values {
        set_army(view, *player, *value);
    }
}

/// Quote every Settler in `view` as the empire's `trained`-th next one: the base price plus the
/// per-Settler step, with no first-Settler discount. A fixture is built with none trained.
pub fn settlers_trained(view: &mut PlayerView, trained: i32) {
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    let base = legatum_core::experiments::RuleExperiments::default()
        .base_production_cost(view.faction_of(ME), settler);
    let cost = base + data::SETTLER_COST_STEP * trained;
    for option in view
        .cities
        .iter_mut()
        .filter_map(|c| c.detail.as_mut())
        .flat_map(|d| d.options.iter_mut())
        .filter(|o| o.item == settler)
    {
        option.cost = cost;
        let production = option.yields.total.production.max(1);
        option.turns = (cost + production - 1) / production;
    }
}

pub struct ViewBuilder {
    pub view: PlayerView,
}

impl ViewBuilder {
    /// A fully explored, visible map of plains, turn 20 in the Standard horizon.
    pub fn new(width: i32, height: i32) -> Self {
        let tile = TileView {
            landmark: None,
            explored: true,
            visible: true,
            terrain: Some(Terrain::Plains),
            ruin: false,
            wonder: None,
            wonder_found: false,
            owner: None,
            resource: None,
            improvement: None,
            road: 0,
        };
        let players: Vec<PlayerPublic> = [(0u8, 0u8), (1, 0), (2, 2), (3, 3)]
            .iter()
            .map(|&(id, team)| PlayerPublic {
                id: PlayerId(id),
                name: format!("P{id}"),
                faction: Faction::Switzerland,
                team: TeamId(team),
                is_human: false,
                difficulty: AiDifficulty::Normal,
                search: false,
                alive: true,
                vp: 0,
                vp_sources: Vec::new(),
                capitals_taken: Vec::new(),
                capitals_sacked: Vec::new(),
                capital_stands: true,
                cities: 0,
                faction_goal_done: true,
                sakoku_turns: None,
                goal_rule: None,
                era: 1,
                gold: 0,
                economy: 0,
                army: 0,
                advancement: 0,
                power: 0,
                lead_percent: 0,
                decadence_percent: 0,
                levy: None,
                companies: 0,
            })
            .collect();
        let mut b = Self {
            view: PlayerView {
                army_breakdown: Vec::new(),
                rule_experiments: legatum_core::experiments::RuleExperiments::default(),
                // Existing tactical fixtures explicitly model the legacy all-war rules.
                diplomacy: legatum_core::diplomacy::DiplomacyView {
                    replies: Vec::new(),
                    coalition: None,
                    exploration: Default::default(),
                    betrayals: Vec::new(),
                    queued: vec![],
                    legacy: true,
                    relations: vec![],
                    alliances: vec![],
                    proposals: vec![],
                    calls: vec![],
                    superseded: vec![],
                    peace_cooldowns: vec![],
                    met: vec![],
                    actions: vec![],
                }
                .into(),
                turn: 20,
                turn_limit: Some(300),
                me: ME,
                team: TeamId(0),
                width,
                height,
                tiles: vec![tile; (width * height) as usize].into(),
                units: Default::default(),
                cities: Default::default(),
                failed_city_sites: Default::default(),
                city_gap_blocked: Default::default(),
                observed_threats: Vec::new(),
                barbarian_camps: Vec::new(),
                players: players.into(),
                economy: EconomyView {
                    military_purchases: 0,
                    gold: 100,
                    gold_per_turn: Breakdown {
                        total: 5,
                        parts: Vec::new(),
                    },
                    science_per_turn: Breakdown {
                        total: 3,
                        parts: Vec::new(),
                    },
                    upkeep: 0,
                    upkeep_breakdown: Default::default(),
                    bankruptcy: None,
                }
                .into(),
                supply: Vec::new(),
                research: ResearchView {
                    science_conversion: Default::default(),
                    era: 0,
                    era_name: String::new(),
                    era_total: 0,
                    era_known: 0,
                    era_firsts: Vec::new(),
                    schedule: Vec::new(),
                    queue: Vec::new(),
                    current: None,
                    progress: 0,
                    cost: None,
                    turns_left: None,
                    known: Vec::new(),
                    skipped: Vec::new(),
                    available: Vec::new(),
                    catalog: Vec::new(),
                    unlocks: Vec::new(),
                }
                .into(),
                eras: Default::default(),
                objective_progress: Vec::new(),
                objective_accounting: Vec::new(),
                development: None,
                faction_goal: None,
                actions: Default::default(),
                courier_halls: Vec::new(),
                imperial_diwans: Vec::new(),
                aircraft: Vec::new(),
                air_sightings: Vec::new(),
                air_contacts: Vec::new(),
                company_offers: Vec::new(),
                great_works: Vec::new(),
                standings: Default::default(),
                winner: None,
                finished: false,
                history: None,
            },
        };
        b.refresh_research();
        b
    }

    /// `available` is the strict gate: unknown technologies of the current era.
    fn refresh_research(&mut self) {
        let known: std::collections::BTreeSet<TechId> =
            self.view.research.known.iter().copied().collect();
        let era = legatum_core::eras::current_era(&known);
        let option = |t: TechId| {
            let def = legatum_core::eras::tech_def(t);
            TechOption {
                known_by: 0,
                effects: Default::default(),
                income_change: Vec::new(),
                tech: t,
                cost: Breakdown {
                    total: def.cost,
                    parts: Vec::new(),
                },
                turns: Some(5),
                era: def.era,
                researchable: !known.contains(&t) && def.era == era,
            }
        };
        self.view.research.era = era;
        self.view.research.era_name = legatum_core::eras::era_name(era).into();
        self.view.research.era_total = legatum_core::eras::era_techs(era).count() as u32;
        self.view.research.era_known = legatum_core::eras::era_techs(era)
            .filter(|t| known.contains(t))
            .count() as u32;
        self.view.players[0].era = era;
        let catalog: Vec<TechOption> = TechId::ALL.iter().map(|t| option(*t)).collect();
        self.view.research.available = catalog.iter().filter(|o| o.researchable).cloned().collect();
        self.view.research.catalog = catalog;
    }

    /// Sets another empire's public era.
    pub fn era_of(mut self, player: u8, era: u8) -> Self {
        self.view.players[usize::from(player)].era = era;
        self
    }

    pub fn known_techs(mut self, techs: &[TechId]) -> Self {
        self.view.research.known = techs.to_vec();
        self.refresh_research();
        self
    }

    pub fn researching(mut self, tech: TechId) -> Self {
        self.view.research.current = Some(tech);
        self
    }

    pub fn faction(mut self, f: Faction) -> Self {
        self.view.players[0].faction = f;
        self
    }

    pub fn gold(mut self, gold: i32) -> Self {
        self.view.economy.gold = gold;
        self
    }

    pub fn terrain(mut self, h: Hex, t: Terrain) -> Self {
        let i = self.view.index(h).expect("on map");
        self.view.tiles[i].terrain = Some(t);
        self
    }

    /// Every tile within `radius` of `h` gets terrain `t`.
    pub fn patch(mut self, h: Hex, radius: i32, t: Terrain) -> Self {
        for n in h.within(radius) {
            if let Some(i) = self.view.index(n) {
                self.view.tiles[i].terrain = Some(t);
            }
        }
        self
    }

    /// Tiles in columns >= `col` are unexplored (unknown terrain).
    pub fn unexplored_from_col(mut self, col: i32) -> Self {
        for i in 0..self.view.tiles.len() {
            if (i as i32) % self.view.width >= col {
                let t = &mut self.view.tiles[i];
                t.explored = false;
                t.visible = false;
                t.terrain = None;
            }
        }
        self
    }

    pub fn owner(mut self, h: Hex, p: PlayerId) -> Self {
        let i = self.view.index(h).expect("on map");
        self.view.tiles[i].owner = Some(p);
        self
    }

    pub fn ruin(mut self, h: Hex) -> Self {
        let i = self.view.index(h).expect("on map");
        self.view.tiles[i].ruin = true;
        self
    }

    /// An aircraft in `city`'s hangar (`legatum_core::rules::air`), owned by the city's owner.
    pub fn aircraft(mut self, id: u32, kind: UnitKind, city: u32) -> Self {
        let owner = self
            .view
            .cities
            .iter()
            .find(|c| c.id == CityId(city))
            .expect("hangar city")
            .owner;
        self.view.aircraft.push(legatum_core::view::AircraftView {
            id: UnitId(id),
            owner,
            kind,
            city: CityId(city),
            hp: data::UNIT_HP,
            level: 0,
            xp: 0,
            xp_next: None,
            upgrade_cost: None,
            upgrade_options: Vec::new(),
            unsupplied: None,
        });
        self
    }

    pub fn unit(self, id: u32, owner: PlayerId, kind: UnitKind, pos: Hex) -> Self {
        self.unit_hp(id, owner, kind, pos, data::UNIT_HP)
    }

    pub fn unit_hp(mut self, id: u32, owner: PlayerId, kind: UnitKind, pos: Hex, hp: i32) -> Self {
        let s = data::unit_stats_for(self.view.faction_of(owner), kind);
        self.view.units.push(UnitView {
            upgrade_to: None,
            upgrade_steps: Vec::new(),
            upgrade_breakdown: None,
            upgrade_need: None,
            unsupplied: None,
            unsupplied_loss: None,
            upgrade_supply_blocker: None,
            siege: false,
            siege_offer: None,
            sight: None,
            resting_heal: None,
            id: UnitId(id),
            owner,
            kind,
            pos,
            hp,
            moves_left: s.moves,
            max_moves: s.moves,
            goto: None,
            path: Vec::new(),
            automation: None,
            standing_order: None,
            needs_orders: false,
            attack_target: None,
            fortify: 0,
            created_turn: None,
            strength: s.strength,
            ranged: s.ranged,
            range: s.range,
            military: s.military,
            level: 0,
            xp: None,
            xp_next: None,
            occupation_lock: None,
            upgrade_cost: None,
            heir_of: None,
            contract_until: None,
            origin: None,
        });
        self
    }

    pub fn goto(mut self, id: u32, to: Hex) -> Self {
        let u = self
            .view
            .units
            .iter_mut()
            .find(|u| u.id == UnitId(id))
            .expect("unit exists");
        u.goto = Some(to);
        // It walked last turn (a unit with full moves and a goto was stuck).
        u.moves_left = 0;
        self
    }

    /// A city. Own cities (and, adversarially, any city passed `detail = true`) get full detail
    /// with every currently producible item as an option and nothing in production.
    pub fn city(mut self, id: u32, owner: PlayerId, pos: Hex, pop: i32) -> Self {
        let detail = (owner == ME).then(|| self.detail(pos, pop));
        self.push_city(id, owner, pos, pop, detail);
        self
    }

    pub fn city_with_detail(mut self, id: u32, owner: PlayerId, pos: Hex, pop: i32) -> Self {
        let detail = Some(self.detail(pos, pop));
        self.push_city(id, owner, pos, pop, detail);
        self
    }

    fn push_city(
        &mut self,
        id: u32,
        owner: PlayerId,
        pos: Hex,
        pop: i32,
        detail: Option<CityDetail>,
    ) {
        let strength = legatum_core::combat::city_defense_strength(pop, false, data::CITY_HP).total;
        for h in pos.within(1) {
            if let Some(i) = self.view.index(h) {
                self.view.tiles[i].owner = Some(owner);
            }
        }
        let is_capital = !self.view.cities.iter().any(|c| c.owner == owner);
        self.view.cities.push(CityView {
            citadel: false,
            star_fort: false,
            has_workshop: false,
            has_air_defence: false,
            occupation_lock: None,
            construction: None,
            great_works: detail
                .as_ref()
                .map(|d| {
                    d.buildings
                        .iter()
                        .copied()
                        .filter(|b| data::is_great_work(*b))
                        .collect()
                })
                .unwrap_or_default(),
            id: CityId(id),
            owner,
            name: format!("C{id}"),
            pos,
            population: pop,
            hp: data::CITY_HP,
            max_hp: data::CITY_HP,
            strength,
            capture_bounty: None,
            greece_captive_tech: false,
            greece_captive_at_risk: false,
            is_capital,
            original_capital: is_capital,
            original_capital_of: is_capital.then_some(owner),
            founder: Some(owner),
            walls: false,
            visible: true,
            detail: detail.map(Into::into),
        });
    }

    fn detail(&self, pos: Hex, population: i32) -> CityDetail {
        let known = &self.view.research.known;
        let mut items: Vec<ProductionItem> = UnitKind::ALL
            .iter()
            .map(|u| ProductionItem::Unit { unit: *u })
            .chain(
                BuildingKind::ALL
                    .iter()
                    .map(|b| ProductionItem::Building { building: *b }),
            )
            .filter(|i| data::item_requires(*i).is_none_or(|t| known.contains(&t)))
            // The Brood Hall, Courier Hall and Heir belong to one faction each, as in core.
            .filter(|i| {
                data::available_to_faction(self.view.faction_of(ME), *i)
                    || !matches!(
                        i,
                        ProductionItem::Building {
                            building: BuildingKind::BroodHall | BuildingKind::CourierHall
                        } | ProductionItem::Unit {
                            unit: UnitKind::Heir
                        }
                    )
            })
            .collect();
        items.sort();
        let production = 4;
        CityDetail {
            captured: false,
            original_capital: false,
            former_owner: None,
            raze_blocker: None,

            idle_yields: None,
            growth_breakdown: Default::default(),
            yields: CityYields {
                total: Yields::new(5, production, 2, 2),
                sources: Vec::new(),
                worked: Vec::new(),
                focus: CityFocus::Balanced,
                focus_reason: "no active build".into(),
            },
            food_stored: 0,
            growth_threshold: data::growth_threshold(population),
            turns_to_grow: Some(5),
            food_construction_paused: None,
            producing: None,
            production_queue: Vec::new(),
            wonder_lost: None,
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_stored: 0,
            production_cost: None,
            turns_to_complete: None,
            production_stall: None,
            idle_production_cap: legatum_core::rules::cities::IDLE_PRODUCTION_CAP,
            idle_production_waste: 0,
            buy_cost: None,
            buildings: Vec::new(),
            idle_buildings: Vec::new(),
            unavailable: Vec::new(),
            production_banked: Vec::new(),
            options: items
                .into_iter()
                .map(|item| {
                    let faction = self.view.faction_of(ME);
                    let experiments = legatum_core::experiments::RuleExperiments::default();
                    let base = experiments.base_production_cost(faction, item);
                    let sparta_premium = faction == Faction::Sparta
                        && !matches!(item, ProductionItem::Unit { unit } if data::unit_stats(unit).military)
                        && !matches!(item, ProductionItem::Building { building } if data::building_stats(building).war_building);
                    let adjusted = if sparta_premium {
                        (base * 3 + 1) / 2
                    } else {
                        base
                    };
                    let full = adjusted;
                    // Fixtures have trained no Settler yet: the first uses the core discount
                    // after the current Standard quote has been scaled.
                    let cost = if item
                        == (ProductionItem::Unit {
                            unit: UnitKind::Settler,
                        }) {
                        full * data::FIRST_SETTLER_PERCENT / 100
                    } else {
                        full
                    };
                    ProductionOption {
                        wonder: None,
                        item,
                        cost,
                        production_stored: 0,
                        production_banked: 0,
                        buy_cost: None,
                        buy_blocker: None,
                        buy_block: None,
                        buy_quote: None,
                        turns: (cost + production - 1) / production,
                        yields: CityYields {
                            total: Yields::new(5, production, 2, 2),
                            sources: Vec::new(),
                            worked: Vec::new(),
                            focus: CityFocus::Balanced,
                            focus_reason: "test fixture".into(),
                        },
                        supply_need: None,
                        resource_blocker: None,
                        upkeep: None,
                        improvement: None,
                    }
                })
                .collect(),
            territory: pos.within(1),
            next_claim: None,
            tile_base_price: 40,
            buyable_tiles: Vec::new(),
            buy_tile_blocker: None,
            resistance: 0,
            turns_to_starve: None,
            turns_to_heal: None,
            heal_blocked: false,
            attacked_last_turn: false,
            population_unlocks: Vec::new(),
        }
    }

    /// Give a city Walls: it gains their strength and, being walled, fires (city strike).
    pub fn walled(mut self, city: u32) -> Self {
        let c = self
            .view
            .cities
            .iter_mut()
            .find(|c| c.id == CityId(city))
            .expect("city exists");
        c.walls = true;
        c.strength = legatum_core::combat::city_defense_strength(c.population, true, c.hp).total;
        if let Some(d) = c.detail.as_mut() {
            d.buildings.push(BuildingKind::Walls);
        }
        self
    }

    /// Put an own city's production in progress (with a buy price).
    pub fn producing(mut self, city: u32, item: ProductionItem, buy_cost: i32) -> Self {
        let c = self
            .view
            .cities
            .iter_mut()
            .find(|c| c.id == CityId(city))
            .expect("city exists");
        let d = c.detail.as_mut().expect("own city");
        d.producing = Some(item);
        d.buy_cost = Some(buy_cost);
        if let Some(opt) = d.options.iter_mut().find(|o| o.item == item) {
            opt.buy_cost = Some(buy_cost);
        }
        self
    }

    pub fn build(self) -> PlayerView {
        self.view
    }
}

/// Check orders the way validate_orders would, judged from the view: own alive units and
/// cities only, one unit-action order per unit, one BuyItem per city, passable known move
/// targets, legal found sites, producible items for idle cities, available research.
pub fn assert_valid(view: &PlayerView, orders: &[Order]) {
    use std::collections::BTreeSet;
    let own_unit = |id: UnitId| {
        view.units
            .iter()
            .find(|u| u.id == id && u.owner == view.me)
            .unwrap_or_else(|| panic!("order for a unit that isn't mine: {id:?} in {orders:?}"))
    };
    let own_city = |id: CityId| {
        view.cities
            .iter()
            .find(|c| c.id == id && c.owner == view.me && c.detail.is_some())
            .unwrap_or_else(|| panic!("order for a city that isn't mine: {id:?}"))
    };
    let mut acted = BTreeSet::new();
    let mut bought = BTreeSet::new();
    let mut research = 0;
    for o in orders {
        match o {
            Order::SetAutoFire { .. } => panic!("AI seats do not set human garrison automation"),
            Order::SetCityAutomation { .. } => panic!("AI seats do not delegate city production"),
            Order::BuyTile { .. } => panic!("AI seats do not buy tiles"),
            Order::GiveGold { .. } => panic!("AI seats do not give Gold"),
            Order::Raze { city } => {
                let c = own_city(*city);
                assert!(
                    c.detail
                        .as_ref()
                        .is_some_and(|d| d.captured && d.raze_blocker.is_none())
                );
                assert!(!orders.iter().any(|other| matches!(other,
                    Order::SetProduction { city: id, .. } | Order::SetProductionQueue { city: id, .. }
                    | Order::BuyItem { city: id, .. } | Order::HireCompany { city: id, .. } if id == city)));
            }
            Order::BreakTreaty { .. }
            | Order::DeclareWar { .. }
            | Order::ProposeTreaty { .. }
            | Order::RespondToProposal { .. }
            | Order::WithdrawProposal { .. } => {
                assert!(
                    view.diplomacy
                        .actions
                        .iter()
                        .any(|a| legatum_core::diplomacy::same_action(&a.order, o)
                            && a.blocked_reason.is_none())
                );
            }
            Order::SetAutomation { unit, .. } => {
                own_unit(*unit);
                assert!(acted.insert(*unit));
            }
            Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                let u = own_unit(*unit);
                assert!(acted.insert(*unit), "two unit orders for {unit:?}");
                assert_ne!(u.pos, *to, "move to own position");
                let t = view.tile(*to).expect("target on map");
                assert!(t.explored, "move target unexplored: {to:?}");
                assert!(
                    t.terrain.and_then(|t| t.move_cost()).is_some(),
                    "move target impassable"
                );
            }
            Order::SleepUnit { unit, .. }
            | Order::SkipUnit { unit }
            | Order::DisbandUnit { unit } => {
                own_unit(*unit);
                assert!(acted.insert(*unit), "two unit orders for {unit:?}");
            }
            Order::AirStrike { plane, .. }
            | Order::Rebase { plane, .. }
            | Order::AirRecon { plane, .. }
            | Order::UpgradeAircraft { plane, .. } => {
                assert!(
                    view.aircraft
                        .iter()
                        .any(|a| a.id == *plane && a.owner == view.me),
                    "order for an aircraft that isn't mine: {plane:?}"
                );
                assert!(acted.insert(*plane), "two orders for aircraft {plane:?}");
            }
            Order::UpgradeUnit { unit } => {
                let u = own_unit(*unit);
                assert!(u.upgrade_cost.is_some(), "upgrade not available");
                assert!(acted.insert(*unit), "two unit orders for {unit:?}");
            }
            Order::UpgradeSiege { unit } => {
                let u = own_unit(*unit);
                assert!(
                    u.siege_offer.as_ref().is_some_and(|o| o.blocker.is_none()),
                    "Siege not available"
                );
                assert!(acted.insert(*unit), "two unit orders for {unit:?}");
            }
            Order::FoundCity { unit } => {
                let u = own_unit(*unit);
                assert_eq!(u.kind, UnitKind::Settler);
                assert!(acted.insert(*unit), "two unit orders for {unit:?}");
                let t = view.tile(u.pos).expect("on map");
                assert!(
                    t.terrain
                        .is_some_and(|t| t.is_land() && t.move_cost().is_some())
                );
                assert!(
                    t.owner.is_none_or(|o| o == view.me),
                    "found on foreign land"
                );
                for c in &view.cities {
                    assert!(
                        c.pos.distance(u.pos) > data::MIN_CITY_GAP,
                        "found within {} of a city",
                        data::MIN_CITY_GAP
                    );
                }
            }
            Order::RangedAttack { unit, target } => {
                let u = own_unit(*unit);
                assert!(acted.insert(*unit), "two unit orders for {unit:?}");
                assert!(u.ranged > 0);
                let d = u.pos.distance(*target);
                assert!(d >= 1 && d <= u.range, "ranged target out of range");
                let hostile = |p: PlayerId| {
                    view.players
                        .iter()
                        .find(|x| x.id == p)
                        .is_some_and(|x| x.team != view.team)
                };
                let unit_there = view
                    .units
                    .iter()
                    .any(|e| e.pos == *target && e.military && hostile(e.owner));
                let city_there = view
                    .cities
                    .iter()
                    .any(|c| c.pos == *target && c.visible && hostile(c.owner));
                assert!(unit_there || city_there, "nothing to shoot at {target:?}");
            }
            Order::SetProduction { city, item } => {
                let c = own_city(*city);
                let d = c.detail.as_ref().unwrap();
                assert!(
                    d.options.iter().any(|o| o.item == *item),
                    "item not available"
                );
            }
            Order::SetProductionQueue { city, queue } => {
                own_city(*city);
                assert!(queue.len() <= 50);
            }
            Order::BuyItem { city, item } => {
                let c = own_city(*city);
                let d = c.detail.as_ref().unwrap();
                assert!(bought.insert(*city), "two buys for one city");
                let cost = d
                    .options
                    .iter()
                    .find(|o| o.item == *item)
                    .and_then(|o| o.buy_cost)
                    .expect("buyable");
                assert_ne!(
                    *item,
                    ProductionItem::Building {
                        building: BuildingKind::RelayUplink
                    }
                );
                assert!(cost <= view.economy.gold);
            }
            Order::HireCompany { city, unit, seller } => {
                own_city(*city);
                assert!(bought.insert(*city), "two buys for one city");
                let offer = view
                    .company_offers
                    .iter()
                    .find(|o| o.unit == *unit && o.seller == *seller)
                    .expect("company not on offer");
                assert!(offer.price <= view.economy.gold);
            }
            Order::SetResearchQueue { techs } => {
                research += 1;
                let mut seen = BTreeSet::new();
                for tech in techs {
                    assert!(
                        view.research.available.iter().any(|t| t.tech == *tech),
                        "{tech:?} queued outside the current era or already known"
                    );
                    assert!(seen.insert(*tech), "{tech:?} queued twice");
                }
            }
            Order::Action { action } => {
                use legatum_core::orders::Action;
                match action {
                    Action::Swarm => assert!(
                        view.actions
                            .swarm
                            .as_ref()
                            .is_some_and(|s| s.blocker.is_none()),
                        "Swarm ordered while unavailable"
                    ),
                    Action::Sakoku { close } => {
                        assert!(
                            view.actions.sakoku.as_ref().is_some_and(
                                |s| s.closed != *close && (!*close || s.blocker.is_none())
                            ),
                            "Sakoku ordered while unavailable"
                        )
                    }
                    Action::Broadcast => assert!(
                        view.actions
                            .broadcast
                            .as_ref()
                            .is_some_and(|b| b.blocker.is_none()),
                        "Broadcast ordered while unavailable"
                    ),
                    Action::RetireVeteran { unit } => assert!(
                        view.actions.retire.as_ref().is_some_and(|r| {
                            r.veterans
                                .iter()
                                .any(|v| v.unit == *unit && v.blocker.is_none())
                        }),
                        "veteran retired while unavailable"
                    ),
                    Action::CallTheWild => assert!(
                        view.actions
                            .wild
                            .as_ref()
                            .is_some_and(|w| w.blocker.is_none()),
                        "the Wild called while unavailable"
                    ),
                    Action::ChooseFreeTechs { techs } => {
                        assert!(
                            view.actions.free_techs.as_ref().is_some_and(
                                |f| !techs.is_empty() && techs.len() <= f.pending as usize
                            ),
                            "free techs chosen without a pending grant"
                        )
                    }
                }
            }
            Order::SetResearch { tech } => {
                research += 1;
                assert!(view.research.available.iter().any(|t| t.tech == *tech));
            }
            Order::SetScienceConversion { .. } => {
                assert_eq!(view.research.known.len(), legatum_core::TechId::ALL.len());
            }
        }
    }
    assert!(research <= 1, "more than one research order");
}
