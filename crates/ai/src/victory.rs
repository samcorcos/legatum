//! Endgame investments, compared by public VP gained per estimated turn. These are plans,
//! not promised points: opponents can move or grow before an item completes.
//! Reaching the VP target doesn't end the game: it starts a public hold that rivals get
//! `data::VP_HOLD_TURNS` turns to break: VP above the target (margin) keeps its value
//! while holding (`missing` is at least 1), and the plan layer prices what risks it.
//! Only our fogged view, public standings and the same faction rules available to players
//! enter the estimates. The opening and tactical defense keep their existing priorities.

use crate::scripted::Planner;
use legatum_core::data;
use legatum_core::view::{CityView, ProductionOption};
use legatum_core::{BuildingKind, CityId, Faction, Order, ProductionItem, UnitKind};
use std::collections::BTreeSet;

mod goals;

impl Planner<'_> {
    /// The 1 VP this seat would hold by capturing a rival's original capital (rules 50: held while
    /// it owns the city, lost if retaken or razed). Public score sources and original ownership
    /// make this exact even if it changed hands.
    pub(crate) fn capital_capture_vp(&self, city: &CityView) -> i32 {
        let my_team = self
            .view()
            .players
            .iter()
            .find(|p| p.id == self.ctx.me)
            .map(|p| p.team);
        let founder_team = city.original_capital_of.and_then(|f| {
            self.view()
                .players
                .iter()
                .find(|p| p.id == f)
                .map(|p| p.team)
        });
        if !city.original_capital
            || city.original_capital_of == Some(self.ctx.me)
            || (my_team.is_some() && founder_team == my_team)
            || city.owner == self.ctx.me
            || self.view().players.iter().any(|p| {
                p.id == self.ctx.me
                    && p.vp_sources.iter().any(|(source, _)| {
                        matches!(source,
                    legatum_core::VpSource::CapitalTaken { capital } if *capital == city.id)
                    })
            })
        {
            0
        } else {
            1
        }
    }

    pub(crate) fn own_original_capital(&self, city: &CityView) -> bool {
        city.original_capital_of == Some(self.ctx.me)
    }

    pub(crate) fn missing_great_work(&self) -> u32 {
        if self
            .view()
            .great_works
            .iter()
            .any(|work| work.player == self.ctx.me && work.completed.is_some())
        {
            0
        } else {
            2
        }
    }

    pub(crate) fn goal_pending(&self) -> bool {
        let rules = self.view().rule_experiments;
        self.view().players.iter().any(|p| {
            p.id == self.ctx.me
                && !p.faction_goal_done
                && !legatum_core::goals::rule_switched_off(rules, p.rule())
        })
    }

    /// Switzerland: the live hoard scores right now. Spending below it loses the VP, so while it
    /// is held the ordinary Gold reserve is the hoard itself.
    pub(crate) fn hoard_held(&self) -> bool {
        self.faction == Faction::Switzerland
            && self.view().faction_goal.as_ref().is_some_and(|g| g.done)
    }

    /// VP-denial value of capturing `city` when it is a Swiss's and its owner scores the
    /// live hoard now (public: its `vp_sources` carry its `FactionGoal`): the public capture
    /// bounty would pull the treasury below the hoard and take the VP away. The treasury is
    /// private, so it is recovered from the public bounty and the city's population share.
    pub(crate) fn hoard_denial_value(&self, city: &legatum_core::view::CityView) -> i32 {
        let v = self.view();
        // A held German goal can fall when a captured city holds one of its five Foundries.
        let forge = self.forge_denial_value(city)
            + self.heir_in_city_value(city)
            + self.hall_in_city_value(city)
            + self.work_in_city_value(city);
        if forge > 0 {
            return forge;
        }
        let Some(bounty) = city.capture_bounty.filter(|b| *b > 0) else {
            return 0;
        };
        let holds = v.players.iter().any(|p| {
            p.id == city.owner
                && p.faction == Faction::Switzerland
                && p.vp_sources.iter().any(|(source, _)| {
                    matches!(source, legatum_core::VpSource::FactionGoal { of } if *of == city.owner)
                })
        });
        if !holds {
            return 0;
        }
        let total: i32 = v
            .cities
            .iter()
            .filter(|c| c.owner == city.owner)
            .map(|c| c.population)
            .sum();
        let share = i64::from(city.population) * if city.is_capital { 2 } else { 1 };
        // A share of the whole treasury or more is capped at the treasury: all of it goes.
        let drops_below = share >= i64::from(total)
            || i64::from(bounty) * i64::from(total) / share.max(1) - i64::from(bounty)
                < i64::from(legatum_core::goals::SWISS_HOARD_GOLD);
        if drops_below {
            50 * legatum_core::goals::spec(Faction::Switzerland).vp as i32
        } else {
            0
        }
    }

    /// The same technology choice and research value used when the AI spends a free pick.
    pub(crate) fn greece_captive_tech_value(&self, city: &legatum_core::view::CityView) -> i32 {
        if self.decisions.greece_captive_attack == 0 || !city.greece_captive_tech {
            return 0;
        }
        crate::goal_actions::pick_free_techs(self.view(), 1, |t| self.tech_value(t))
            .first()
            .map_or(0, |t| {
                self.tech_value(*t) * self.decisions.greece_captive_attack / 100
            })
    }

    /// A built Great Work's two public points move with its tile. City capture normally takes
    /// that tile too; a Celtic sack only denies the holder the points.
    pub(crate) fn work_in_city_value(&self, city: &legatum_core::view::CityView) -> i32 {
        if self.view().same_team(self.view().me, city.owner) {
            return 0;
        }
        self.view()
            .tiles
            .iter()
            .filter(|t| {
                t.owner == Some(city.owner)
                    && t.landmark.is_some_and(|s| {
                        s.city == city.id && legatum_core::data::is_great_work(s.building)
                    })
            })
            .count() as i32
            * if self.faction == Faction::Celts {
                100
            } else {
                200
            }
    }

    pub(crate) fn wonder_guard_city(&self, city: &legatum_core::view::CityView) -> bool {
        self.view().tiles.iter().any(|t| {
            t.owner == Some(self.ctx.me)
                && t.landmark.is_some_and(|s| {
                    s.city == city.id && legatum_core::data::is_great_work(s.building)
                })
        })
    }

    pub(crate) fn diwan_guard_city(&self, city: &legatum_core::view::CityView) -> bool {
        self.view()
            .imperial_diwans
            .iter()
            .any(|d| d.holder == self.ctx.me && d.city == city.id)
    }

    /// Live goal points a city capture actually transfers or removes. Permanent points stay
    /// scored. This is kept separate from the tactical capture bounty: the win model needs
    /// the two teams' VP changed, not a single blended city value.
    pub(crate) fn capture_goal_swing(&self, city: &CityView) -> (i32, i32) {
        if self.decisions.vp_denial == 0
            || city.owner.is_barbarian()
            || self.view().same_team(self.view().me, city.owner)
        {
            return (0, 0);
        }
        let v = self.view();
        let rules = self.view().rule_experiments;
        let goal_vp = |faction| legatum_core::goals::spec_in(rules, faction).vp as i32;
        let mut own = 0;
        let mut denied = 0;
        if self.forge_denial_value(city) > 0 {
            denied += goal_vp(Faction::Germany);
        }
        if v.players.iter().any(|p| {
            p.id == city.owner && p.faction == Faction::Switzerland
                && p.vp_sources.iter().any(|(s, _)| matches!(s, legatum_core::VpSource::FactionGoal { of } if *of == city.owner))
        }) && self.hoard_denial_value(city) > 0 {
            denied += goal_vp(Faction::Switzerland);
        }
        if v.courier_halls
            .iter()
            .any(|h| h.city == city.id && v.team_of(h.holder) != v.team)
        {
            denied += goal_vp(Faction::Mongols);
            own += if self.sacks_city(city) {
                0
            } else {
                goal_vp(Faction::Mongols)
            };
        }
        if v.imperial_diwans
            .iter()
            .any(|d| d.city == city.id && v.team_of(d.holder) != v.team)
        {
            denied += goal_vp(Faction::India);
            if !self.sacks_city(city) {
                own += goal_vp(Faction::India);
            }
        }
        if v.units
            .iter()
            .any(|u| u.kind == UnitKind::Heir && u.pos == city.pos && u.owner == city.owner)
        {
            denied += goal_vp(Faction::Byzantium);
            own += if self.sacks_city(city) {
                0
            } else {
                goal_vp(Faction::Byzantium)
            };
        }
        let works = v
            .tiles
            .iter()
            .filter(|t| {
                t.owner == Some(city.owner)
                    && t.landmark.is_some_and(|s| {
                        s.city == city.id && legatum_core::data::is_great_work(s.building)
                    })
            })
            .count() as i32;
        denied += works * 2;
        if !self.sacks_city(city) {
            own += works * 2;
        }
        (own, denied)
    }

    /// A rival's next public VP can depend on this city even before it scores. Great Work
    /// construction is public and city-specific; a city/territory objective exposes the rule
    /// and era entry, though not the rival's private progress, so count denial is a smaller
    /// estimate rather than an invented completed point.
    pub(crate) fn rival_objective_denial(&self, city: &CityView) -> i32 {
        if self.decisions.vp_denial == 0 || city.owner.is_barbarian() {
            return 0;
        }
        let v = self.view();
        let work = v
            .great_works
            .iter()
            .filter(|w| {
                w.player == city.owner
                    && w.city == Some(city.id)
                    && w.completed.is_none()
                    && w.building.is_some()
            })
            .map(|w| 100 + 200 * w.progress.max(0) / w.cost.max(1))
            .max()
            .unwrap_or(0);
        let count = v
            .eras
            .iter()
            .filter(|era| {
                era.entered_by.contains(&city.owner)
                    && !era.scored_by.iter().any(|s| s.player == city.owner)
            })
            .flat_map(|era| &era.objectives)
            .filter(|o| {
                matches!(
                    o.objective,
                    legatum_core::objectives::Objective::Cities { .. }
                        | legatum_core::objectives::Objective::Territory { .. }
                )
            })
            .count()
            .min(2) as i32
            * 40;
        (work + count) * self.decisions.vp_denial / 100
    }

    /// Whether our captures are priced as sacks (gene `sack_price`): the Celts burn every city
    /// they take, so a capture brings no city, people, land, output, milestone or title.
    /// Whether capturing `city` sacks it: the Celtic sack never touches a city we founded, which
    /// comes home intact (designer, 2026-11-14).
    pub(crate) fn sacks_city(&self, city: &CityView) -> bool {
        self.sacks() && city.founder != Some(self.ctx.me)
    }

    pub(crate) fn sacks(&self) -> bool {
        self.faction == Faction::Celts
            && self.decisions.sack_price > 0
            && self.rule_on(legatum_core::experiments::FactionRule::CeltsNoKeep)
    }

    /// VP a sack of `city` brings through the warband camp it raises: the camp stands on a free
    /// tile within `SACK_CAMP_RADIUS` of the ruins, and it completes Raise the Warbands when it
    /// lands in the territory of the one city of ours that has no camp yet while the goal is not
    /// held. Zero otherwise; the camp's raids and denied tiles are not priced.
    pub(crate) fn sack_camp_vp(&self, city: &CityView) -> i32 {
        use legatum_core::rules::barbarians::{RITE_MIN_CAMP_DISTANCE, SACK_CAMP_RADIUS};
        let v = self.view();
        if !self.sacks_city(city)
            || !self.rule_on(legatum_core::experiments::FactionRule::CeltsSack)
            || v.faction_goal.as_ref().is_none_or(|g| g.done)
        {
            return 0;
        }
        let camps: Vec<legatum_core::Hex> = v.barbarian_camps.iter().map(|c| c.pos).collect();
        let campless: Vec<&CityView> = self
            .ctx
            .own_cities
            .iter()
            .copied()
            .filter(|c| {
                c.detail
                    .as_ref()
                    .is_some_and(|d| !d.territory.iter().any(|h| camps.contains(h)))
            })
            .collect();
        let [last] = campless.as_slice() else {
            return 0;
        };
        let reaches = last.detail.as_ref().is_some_and(|d| {
            d.territory.iter().any(|h| {
                h.distance(city.pos) <= SACK_CAMP_RADIUS
                    && *h != city.pos
                    && camps
                        .iter()
                        .all(|c| c.distance(*h) >= RITE_MIN_CAMP_DISTANCE)
            })
        });
        if reaches {
            legatum_core::goals::spec(Faction::Celts).vp as i32
        } else {
            0
        }
    }

    /// Gene `sack_commit`: a Celtic Contain war is declared only with a funded siege route to a
    /// named city, as Sparta's is (`contain_commit`): a war the Celts cannot pay for in
    /// sacks costs them their Raise the Warbands camps and their cities for nothing.
    pub(crate) fn sack_commits(&self) -> bool {
        self.sacks() && self.decisions.sack_commit > 0
    }

    /// VP the owner of `city` loses when we sack it, from public information only: the held
    /// faction goal the city props up (the Swiss hoard, Germany's Forges, the Heir, the Hall).
    /// Era objectives already scored are permanent, and the
    /// progress a sack sets back on an unscored one is private to its owner, so neither counts.
    pub(crate) fn sack_vp_denied(&self, city: &legatum_core::view::CityView) -> i32 {
        let v = self.view();
        if city.owner.is_barbarian() || v.team_of(city.owner) == v.team {
            return 0;
        }
        if self.decisions.vp_denial > 0 {
            return self.capture_goal_swing(city).1;
        }
        let goals = self.forge_denial_value(city) / 50
            + self.heir_in_city_value(city) / 100
            + self.hall_in_city_value(city) / 50
            + self.work_in_city_value(city) / 50;
        // The hoard value is the Swiss goal's alone once every other goal's share is zero.
        let hoard = if goals == 0 {
            self.hoard_denial_value(city) / 50
        } else {
            0
        };
        goals + hoard
    }

    pub(crate) fn swarm_weakness_value(&self, city: &legatum_core::view::CityView) -> i32 {
        let v = self.view();
        let retired = self.retired_army_weakness_value(city);
        if v.team_of(city.owner) == v.team
            || !v.players.iter().any(|p| {
                p.id == city.owner
                    && p.rule() == legatum_core::goals::GoalRule::Swarm
                    && p.faction_goal_done
            })
        {
            return retired;
        }
        retired + (8 - city.population).max(0) * 15
    }

    pub(crate) fn saving_wins(&self) -> bool {
        self.faction == Faction::Switzerland
            && self.goal_pending()
            && self.view().standings.iter().any(|s| {
                s.team == self.view().team
                    && s.vp + legatum_core::goals::spec(self.faction).vp >= s.target
            })
    }

    pub(crate) fn saving_for_goal(&self) -> bool {
        if self.faction != Faction::Switzerland {
            return false;
        }
        if self.hoard_held() {
            return true;
        }
        if !self.goal_pending() {
            return false;
        }
        let income = self.view().economy.gold_per_turn.total;
        let goal = legatum_core::goals::SWISS_HOARD_GOLD;
        let horizon = (legatum_core::data::EXPECTED_GAME_TURNS.saturating_sub(self.view().turn)
            as i32)
            .min(300);
        self.saving_wins()
            || (income > 0
                && (self.gold_left + income >= goal
                    || (self.ctx.own_cities.len() >= 2
                        && self.gold_left + income * horizon >= goal)))
    }

    pub(crate) fn pursuing_victory(&self) -> bool {
        let v = self.view();
        self.campaigns.plan.posture == Some(crate::plan::Posture::Victory)
            || (self.difficulty == legatum_core::AiDifficulty::Hard
                && v.turn >= self.decisions.victory_close_turn as u32
                && v.standings
                    .iter()
                    .any(|s| s.team == v.team && s.target.saturating_sub(s.vp) <= 4))
            || self.endgame_victory_planning()
            || self.objective_close()
    }

    fn endgame_victory_planning(&self) -> bool {
        let v = self.view();
        v.turn >= self.decisions.endgame_victory_turn as u32
            && v.standings.iter().any(|s| {
                s.team == v.team
                    && (s.target.saturating_sub(s.vp) <= 2
                        || v.turn >= legatum_core::data::EXPECTED_GAME_TURNS)
            })
    }

    /// Make one investment at a time, in the city that can deliver it soonest. Threatened
    /// cities use the defense forecast.
    pub(crate) fn plan_victory(&mut self) {
        let v = self.view();
        if !self.pursuing_victory() {
            return;
        }
        let Some(standing) = v.standings.iter().find(|s| s.team == v.team) else {
            return;
        };
        let missing = standing.target.saturating_sub(standing.vp).max(1);
        let combat = |item| matches!(item, ProductionItem::Unit { unit } if !matches!(unit, UnitKind::Scout | UnitKind::Settler));
        let committed_army = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .count()
            + self
                .ctx
                .own_cities
                .iter()
                .filter(|c| {
                    c.detail
                        .as_ref()
                        .and_then(|d| d.producing)
                        .is_some_and(combat)
                })
                .count();
        let minimum = self.ctx.own_cities.len()
            + usize::from(
                self.difficulty == legatum_core::AiDifficulty::Hard
                    && self.ctx.own_cities.len() >= 3,
            );
        let score = |points: u32, turns: i32| {
            if turns < 1 || turns > self.turns_remaining() {
                0
            } else {
                points.min(missing) as i32 * 10_000 / (turns + 8)
            }
        };
        // Avoid spending Gold to race a Swiss hoard point due next turn.
        let mut best_score = 0;
        if self.faction == Faction::Switzerland
            && self.goal_pending()
            && v.economy.gold_per_turn.total > 0
        {
            let remaining = (legatum_core::goals::SWISS_HOARD_GOLD - self.gold_left).max(0);
            let income = v.economy.gold_per_turn.total;
            best_score = best_score.max(score(
                legatum_core::goals::spec(self.faction).vp,
                ((remaining + income - 1) / income).max(1),
            ));
        }
        let mut best = None;
        let mut best_cost = 0;
        for (ci, c) in self.ctx.own_cities.iter().enumerate() {
            let d = c.detail.as_ref().expect("own city");
            if d.resistance > 0
                || self.orders.iter().any(|o| {
                    matches!(o,
                    Order::SetProduction { city, .. } | Order::BuyItem { city, .. } if *city == c.id)
                })
            {
                continue;
            }
            for opt in &d.options {
                if opt.item
                    == (ProductionItem::Unit {
                        unit: UnitKind::Settler,
                    })
                    && !self.opening_admits_settler()
                    || !self.opening_allows_optional_military(opt.item)
                {
                    continue;
                }
                // An investment must not become a second defensive buying path that bypasses
                // the emergency forecast.
                if self.ctx.threat[ci] > self.local_defense(c) {
                    continue;
                }
                // Every option's turn forecast counts its own banked and unassigned Production.
                if opt.turns < 1 {
                    continue;
                }
                let (points, delay) = self.production_points(c, opt);
                if points == 0 || points < missing && self.opening_choice(c).is_some() {
                    continue;
                }
                // Optional investments must not bypass the city planner's defense
                // floor or make an undeveloped colony spend decades on another settler.
                // A concrete investment that reaches the target may: it starts the hold.
                if points < missing
                    && (committed_army < minimum
                        || (opt.item
                            == (ProductionItem::Unit {
                                unit: UnitKind::Settler,
                            })
                            && self.ctx.own_cities.len() >= 2
                            && !data::has_building(&d.buildings, BuildingKind::Workshop)))
                {
                    continue;
                }
                let upkeep = match opt.item {
                    ProductionItem::Unit { unit } if data::pays_upkeep(unit) => {
                        self.marginal_military_upkeep()
                    }
                    _ => 0,
                };
                let buffer = (-v.economy.gold_per_turn.total + upkeep).max(0) * 40;
                let mut reserve = if points >= missing && self.faction != Faction::Switzerland {
                    buffer
                } else {
                    self.gold_reserve().max(buffer)
                };
                if self.faction == Faction::Switzerland
                    && (self.goal_pending() || self.hoard_held())
                {
                    // Purchases resolve before the treasury goal check. Preserve
                    // its threshold (held or still to bank) even when this turn's income
                    // is negative.
                    reserve = reserve.max(
                        legatum_core::goals::SWISS_HOARD_GOLD
                            .saturating_sub(v.economy.gold_per_turn.total.min(0)),
                    );
                }
                if upkeep > 0 && self.gold_left < buffer {
                    continue;
                }
                let buy = opt
                    .buy_cost
                    .filter(|_| opt.buy_block.is_none())
                    .filter(|cost| *cost > 0 && self.gold_left - cost >= reserve && opt.turns > 1);
                // An optional estimate must not continually interrupt a Workshop,
                // garrison or growing colony. Independent purchases preserve the queue;
                // only an investment that can actually win may redirect construction.
                if points < missing
                    && buy.is_none()
                    && d.producing.is_some_and(|item| item != opt.item)
                {
                    continue;
                }
                let turns = if buy.is_some() { 1 } else { opt.turns } + delay;
                let value = score(points, turns);
                let cost = buy.unwrap_or(0);
                if value > best_score || (value == best_score && cost < best_cost) {
                    best_score = value;
                    best_cost = cost;
                    best = Some((c.id, opt.item, d.producing != Some(opt.item), buy));
                }
            }
        }
        if let Some((city, item, start, buy)) = best {
            if start && buy.is_none() {
                self.orders.push(Order::SetProduction { city, item });
            }
            if let Some(cost) = buy {
                self.gold_left -= cost;
                self.orders.push(Order::BuyItem { city, item });
            }
        }
    }

    /// Public, local progress toward our own victory, rather than only punishing the VP
    /// leader. Population and the city's immediate ring are known; unseen buildings and
    /// distant territory are not. Occupation/resistance and any missing granary still cost
    /// time, so this ranks campaigns without relaxing their combat safety threshold.
    pub(crate) fn conquest_victory_value(&self, city: &CityView) -> i32 {
        let objective = self.objective_capture_value(city);
        // A sacked city is nobody's: only the capture itself counts (`sack_vp_denied`
        // prices what the victim loses).
        objective
    }

    fn production_points(&self, city: &CityView, opt: &ProductionOption) -> (u32, i32) {
        let d = city.detail.as_ref().expect("own city");
        let mut points = 0;
        let mut delay = 0;
        match opt.item {
            ProductionItem::Project { .. } | ProductionItem::Improvement { .. } => {}
            ProductionItem::Unit {
                unit: UnitKind::Settler,
            } => {
                let pending = self
                    .ctx
                    .own_units
                    .iter()
                    .any(|u| u.kind == UnitKind::Settler)
                    || self.ctx.own_cities.iter().any(|c| {
                        c.id != city.id
                            && c.detail
                                .as_ref()
                                .is_some_and(|d| d.producing == Some(opt.item))
                    });
                if pending || (city.population < 2 && d.turns_to_grow.is_none_or(|g| g > opt.turns))
                {
                    return (0, 0);
                }
                let mut sites: Vec<_> = self
                    .ctx
                    .site_score
                    .iter()
                    .enumerate()
                    .filter_map(|(i, score)| {
                        let h = self.ctx.hex_at(i);
                        (score.is_some_and(|s| s >= self.decisions.settler_min_site_score)
                            && city.pos.distance(h) <= self.decisions.settler_max_site_distance
                            && !self.ctx.in_danger(h)
                            && self.ctx.reachable(city.pos, h))
                        .then_some(h)
                    })
                    .collect();
                sites.sort_by_key(|h| (city.pos.distance(*h), *h));
                let Some(&site) = sites.first() else {
                    return (0, 0);
                };
                delay = city.pos.distance(site) + 2;
                points += self.objective_item_points(city, opt.item);
            }
            ProductionItem::Unit { .. } => return (0, 0),
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                if legatum_core::data::has_building(&d.buildings, building)
                    || !self.building_slot_available(city.id, building, &[])
                {
                    return (0, 0);
                }
                if data::is_great_work(building) {
                    points += self.missing_great_work();
                }
                points += self.objective_item_points(city, opt.item);
                if self.goal_pending() && self.building_completes_goal(city, building) {
                    points += legatum_core::goals::spec(self.faction).vp;
                }
            }
        }
        (points, delay)
    }

    /// Reserve core's shared slots across existing buildings, effective queues and every
    /// selected step. Continuing an existing queue consumes its slot only once.
    fn building_slot_available(
        &self,
        city: CityId,
        building: BuildingKind,
        planned: &[(CityId, BuildingKind)],
    ) -> bool {
        let Some(limit) = data::building_empire_limit(building) else {
            return true;
        };
        let mut committed = BTreeSet::new();
        for c in &self.ctx.own_cities {
            let d = c.detail.as_ref().expect("own city");
            if limit.includes_built {
                for &built in &d.buildings {
                    if limit.buildings.contains(&built) {
                        committed.insert((c.id, built));
                    }
                }
            }
            // A pending SetProduction replaces the whole plan; otherwise the saved queue
            // (e.g. inherited from a human seat) also holds core's shared slots.
            let pending = self.orders.iter().rev().find_map(|o| match o {
                Order::SetProduction { city, item } if *city == c.id => Some(*item),
                _ => None,
            });
            let plan: Vec<ProductionItem> = match pending {
                Some(item) => vec![item],
                None => d
                    .producing
                    .into_iter()
                    .chain(d.production_queue.iter().copied())
                    .collect(),
            };
            for item in plan {
                if let ProductionItem::Building { building } = item
                    && limit.buildings.contains(&building)
                {
                    committed.insert((c.id, building));
                }
            }
        }
        for &(city, building) in planned {
            if limit.buildings.contains(&building) {
                committed.insert((city, building));
            }
        }
        committed.insert((city, building));
        committed.len() <= limit.max
    }

    fn building_completes_goal(&self, city: &CityView, building: BuildingKind) -> bool {
        match self.faction {
            Faction::Germany => self.factory_completes_goal(city, building),
            Faction::India => building == BuildingKind::ImperialDiwan,
            // The existing reserve, research, scout and formation planners pursue the other
            // goals; a building alone cannot finish those movement/exploration conditions.
            _ => false,
        }
    }
}
