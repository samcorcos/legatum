//! Diagnostics (`war_trace`): why a Hard seat at war does or doesn't march on an enemy city.
//! Read-only: one full planning pass on the seat's own view, then the war layer's own gates
//! (the plan, the campaign forecast per enemy city, the field army, Gold) printed as lines.
use crate::ScriptedBrain;
use legatum_core::{Order, PlayerView, view::CityView};

impl ScriptedBrain {
    /// Lines describing this turn's war decisions for the seat. Exactly the planning pass
    /// `plan` makes; nothing is kept.
    #[doc(hidden)]
    pub fn war_diagnosis(&self, view: &PlayerView) -> Vec<String> {
        let affordable = crate::supply::affordable(view);
        let p = self.planner_with(&affordable, false);
        let v = p.view();
        let mut out = Vec::new();
        let plan = p.campaigns.plan.active.as_ref();
        out.push(format!(
            "plan {:?} value {} | contain_war {:?} | conquest_city {:?} | campaign {:?} | gold {} runway {:?} floor {}",
            plan.map(|p| &p.kind),
            plan.map_or(0, |p| p.value),
            p.contain_war().map(|t| t.0),
            p.conquest_city().map(|c| c.id.0),
            p.campaigns
                .active
                .as_ref()
                .map(|c| (c.target.0, c.committed.len())),
            v.economy.gold,
            p.gold_runway(),
            p.runway_floor(),
        ));
        out.push(format!(
            "alternatives {:?}",
            p.plan_candidates()
                .iter()
                .filter(|c| matches!(
                    &c.kind,
                    crate::plan::PlanKind::Campaign { .. }
                        | crate::plan::PlanKind::Contain { .. }
                        | crate::plan::PlanKind::Peace { .. }
                ))
                .map(|c| (&c.kind, c.value, &c.terms))
                .collect::<Vec<_>>()
        ));
        out.push(format!(
            "gold reserve {} ordinary {} upgrade {} | income {} | saving_for_goal {} hoard_held {} hoard_at_war {}",
            p.gold_reserve(),
            p.ordinary_gold_reserve(),
            p.frontline_upgrade_reserve(),
            v.economy.gold_per_turn.total,
            p.saving_for_goal(),
            p.hoard_held(),
            p.hoard_spent_at_war(),
        ));
        out.push(format!(
            "decisive buys {:?}",
            p.decisive_candidates()
                .iter()
                .map(|c| (
                    c.city.0, c.item, c.cost, c.target.0, c.odds, c.gain, c.price
                ))
                .collect::<Vec<_>>()
        ));
        out.push(format!(
            "hollow_leader {:?} | hollow_war_target {:?} | reads {:?}",
            p.hollow_leader().map(|t| t.0),
            p.hollow_war_target().map(|t| t.0),
            v.players
                .iter()
                .filter(|o| o.alive && o.team != v.team && o.cities > 0)
                .map(|o| (
                    o.team.0,
                    crate::threat::hollowness(v, o.team),
                    p.hollow_floor(o.team),
                    crate::threat::hollow_bonus_from(v, o.team, p.hollow_floor(o.team)),
                ))
                .collect::<Vec<_>>(),
        ));
        let military: Vec<_> = p
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != legatum_core::UnitKind::Scout)
            .collect();
        let garrisons = military
            .iter()
            .filter(|u| p.ctx.own_cities.iter().any(|c| c.pos == u.pos))
            .count();
        out.push(format!(
            "army {} military, {} in own cities, {} in the field",
            military.len(),
            garrisons,
            military.len() - garrisons
        ));
        let enemies: Vec<_> = v
            .players
            .iter()
            .filter(|o| o.alive && o.team != v.team && v.teams_at_war(v.team, o.team))
            .map(|o| o.team)
            .collect();
        for team in &enemies {
            let targets: Vec<_> = v
                .cities
                .iter()
                .filter(|c| v.team_of(c.owner) == *team)
                .collect();
            out.push(format!(
                "target census T{}: {} known, {} own-VP captures, {} sack-VP denials, {} funded outlooks",
                team.0,
                targets.len(),
                targets.iter().filter(|c| p.conquest_victory_value(c) > 0).count(),
                targets.iter().filter(|c| p.sacks() && p.sack_vp_denied(c) > 0).count(),
                targets.iter().filter(|c| p.campaign_outlook(c).is_some()).count(),
            ));
            out.push(format!(
                "campaign_against T{}: {:?}",
                team.0,
                p.campaign_against(*team)
                    .map(|o| (o.city.0, o.turns, o.march, o.odds))
            ));
            let mut cities: Vec<&CityView> = v
                .cities
                .iter()
                .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == *team)
                .collect();
            cities.sort_by_key(|c| {
                military
                    .iter()
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(99)
            });
            for c in cities.iter().take(4) {
                let near = military
                    .iter()
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(99);
                let far = p.strike_force_within(v, c, p.decisions.march_turns);
                let chargers = far
                    .iter()
                    .filter(|(u, _, _)| crate::composition::charges(u.kind))
                    .count();
                let shooters = far
                    .iter()
                    .filter(|(u, _, _)| crate::composition::shooter_only(u.kind, u.ranged))
                    .count();
                let placed: Vec<_> = far.iter().map(|(u, h, _)| (*u, *h, 0)).collect();
                let siege = (!placed.is_empty())
                    .then(|| p.forecast_siege(v, c, &placed))
                    .flatten()
                    .map(|s| (s.turns, s.losses));
                out.push(format!(
                    "  city {} {} walls {} hp {} visible {} | nearest unit {} | march force {} (melee {}, shooters {}) | siege {:?} | outlook {:?}",
                    c.id.0,
                    c.name,
                    c.walls,
                    c.hp,
                    c.visible,
                    near,
                    far.len(),
                    chargers,
                    shooters,
                    siege,
                    p.campaign_outlook(c).map(|o| (o.turns, o.march, o.odds)),
                ));
                let aged = p.aged_city(c);
                let city = aged.as_ref().unwrap_or(c);
                out.push(format!(
                    "    aged walls {} hp {} | forecast {:?}",
                    city.walls,
                    city.hp,
                    p.campaign_forecast(city, None)
                        .map(|f| (f.turns, f.march, f.force.len())),
                ));
            }
        }
        // At Peace: the cities a new war could open on, judged on the prospective war view
        // `campaign_forecast` builds, nearest our army first.
        for o in v.players.iter().filter(|o| {
            o.alive
                && o.team != v.team
                && o.cities > 0
                && !v.teams_at_war(v.team, o.team)
                && crate::opportunity::legal_declaration(v, o.team)
        }) {
            let mut prospective = v.clone();
            if let Some(r) = prospective
                .diplomacy
                .relations
                .iter_mut()
                .find(|r| r.between(v.team, o.team))
            {
                r.state = legatum_core::diplomacy::RelationState::War;
            }
            out.push(format!(
                "prospect T{}: campaign_against {:?} | strike {:?}",
                o.team.0,
                p.campaign_against(o.team)
                    .map(|o| (o.city.0, o.turns, o.march, o.odds)),
                p.strike_against(o.team).map(|s| s.city.0),
            ));
            let mut cities: Vec<&CityView> = v
                .cities
                .iter()
                .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == o.team)
                .collect();
            cities.sort_by_key(|c| {
                military
                    .iter()
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(99)
            });
            for c in cities.iter().take(3) {
                out.push(format!(
                    "    units {:?}",
                    military
                        .iter()
                        .map(|u| (
                            u.id.0,
                            format!("{:?}", u.kind),
                            u.hp,
                            u.pos.distance(c.pos),
                            p.ctx.own_cities.iter().any(|h| h.pos == u.pos)
                        ))
                        .collect::<Vec<_>>()
                ));
                let all: Vec<_> = military
                    .iter()
                    .filter(|u| u.hp >= 50)
                    .zip(c.pos.ring(1).into_iter().chain(c.pos.ring(2)))
                    .map(|(u, h)| (**u, h, 0))
                    .collect();
                out.push(format!(
                    "    whole army beside it: siege {:?}",
                    p.forecast_siege(&prospective, c, &all)
                        .map(|s| (s.turns, s.losses))
                ));
                let near = military
                    .iter()
                    .map(|u| u.pos.distance(c.pos))
                    .min()
                    .unwrap_or(99);
                let reach = p.strike_force(&prospective, c);
                let far = p.strike_force_within(&prospective, c, p.decisions.march_turns);
                let placed: Vec<_> = far.iter().map(|(u, h, _)| (*u, *h, 0)).collect();
                let siege = (!placed.is_empty())
                    .then(|| p.forecast_siege(&prospective, c, &placed))
                    .flatten()
                    .map(|s| (s.turns, s.losses));
                let reach_siege = (!reach.is_empty())
                    .then(|| p.forecast_siege(&prospective, c, &reach))
                    .flatten()
                    .map(|s| (s.turns, s.losses));
                out.push(format!(
                    "  prospect city {} {} cap {} walls {} hp {}/{} str {} pop {} visible {} | nearest unit {} | reach {} (melee {}) siege {:?} | march {} (melee {}, shooters {}, max turns {}) siege {:?} | forecast {:?} | outlook {:?}",
                    c.id.0,
                    c.name,
                    c.is_capital,
                    c.walls,
                    c.hp,
                    c.max_hp,
                    c.strength,
                    c.population,
                    c.visible,
                    near,
                    reach.len(),
                    reach
                        .iter()
                        .filter(|(u, _, _)| crate::composition::charges(u.kind))
                        .count(),
                    reach_siege,
                    far.len(),
                    far.iter()
                        .filter(|(u, _, _)| crate::composition::charges(u.kind))
                        .count(),
                    far.iter()
                        .filter(|(u, _, _)| crate::composition::shooter_only(u.kind, u.ranged))
                        .count(),
                    far.iter().map(|(_, _, t)| *t).max().unwrap_or(0),
                    siege,
                    p.campaign_forecast(c, None)
                        .map(|f| (f.turns, f.march, f.force.len())),
                    p.campaign_outlook(c).map(|o| (o.turns, o.march, o.odds)),
                ));
            }
        }
        let (mut units_built, mut buys) = (Vec::new(), Vec::new());
        for o in &p.orders {
            match o {
                Order::SetProduction { city, item } => {
                    units_built.push(format!("{}:{item:?}", city.0))
                }
                Order::BuyItem { city, item } => buys.push(format!("{}:{item:?}", city.0)),
                _ => {}
            }
        }
        out.push(format!("production {units_built:?} | buys {buys:?}"));
        for c in &p.ctx.own_cities {
            if let Some(d) = &c.detail {
                let here = military.iter().filter(|u| u.pos == c.pos).count();
                out.push(format!(
                    "  own {} {} | garrison {here} | building {:?} {}/{:?} ({:?} turns, stall {:?}) | prod {}",
                    c.id.0,
                    c.name,
                    d.producing,
                    d.production_stored,
                    d.production_cost,
                    d.turns_to_complete,
                    d.production_stall,
                    d.yields.total.production,
                ));
            }
        }
        out
    }
}
