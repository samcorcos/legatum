//! Settlers (pick and found the best site) and scouts (explore the frontier and ruins).

use crate::ai_v1_baseline::analysis::UNREACHABLE;
use crate::ai_v1_baseline::production::{MAX_SITE_DISTANCE, MIN_SITE_SCORE};
use crate::ai_v1_baseline::scripted::{Planner, tag};
use legatum_core::data::{INDIA_CAPITAL_RADIUS, MIN_CITY_GAP};
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, UnitKind};

/// A settler with no city yet strongly prefers sites this close (movement cost), so the capital
/// is founded within a turn or two.
const FIRST_CITY_REACH: i32 = 2;

impl Planner<'_> {
    pub fn plan_settlers(&mut self) {
        let settlers: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.kind == UnitKind::Settler)
            .collect();
        for s in settlers {
            self.plan_settler(s);
        }
    }

    fn plan_settler(&mut self, s: &UnitView) {
        let ctx = &self.ctx;
        let in_own_city = ctx.own_cities.iter().any(|c| c.pos == s.pos);
        // A visible enemy army nearby: retreat to the nearest own city.
        if ctx.in_danger(s.pos) && !in_own_city && !ctx.own_cities.is_empty() {
            let home = ctx.nearest_own_city(s.pos).expect("has cities").pos;
            self.move_or_hold(s, home);
            return;
        }
        match self.choose_site(s) {
            Some(site) if site == s.pos => {
                self.settle_targets.push(site);
                // Full moves and no goto after turn 1 means it stood here last turn: most
                // likely its found order clashed with a settler we can't see (found orders
                // within 3 of each other all fail). Flip a coin so the two stop mirroring.
                let idle = s.goto.is_none() && s.moves_left >= s.max_moves;
                let wait =
                    self.view().turn > 1 && idle && self.jitter(tag::SETTLE, s.id.0 as u64, 1) == 0;
                if wait {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                } else {
                    self.unit_order(s.id, Order::FoundCity { unit: s.id });
                }
            }
            Some(site) => {
                self.settle_targets.push(site);
                self.escort_sites.push(site);
                self.move_or_hold(s, site);
            }
            None => match self.ctx.nearest_own_city(s.pos) {
                Some(c) => {
                    let home = c.pos;
                    self.move_or_hold(s, home);
                }
                None => {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
            },
        }
    }

    /// The best reachable site for this settler, trading site quality against travel time.
    pub fn choose_site(&self, s: &UnitView) -> Option<Hex> {
        let ctx = &self.ctx;
        let field = ctx.distance_field(s.pos, true);
        let first_city = ctx.own_cities.is_empty();
        let capital = ctx.capital().map(|c| c.pos);
        let moves = s.max_moves.max(1);
        // Found orders within 3 of each other all fail, whoever gives them: keep clear of
        // other empires' visible settlers too.
        let other_settlers: Vec<Hex> = ctx
            .view
            .units
            .iter()
            .filter(|u| u.owner != ctx.me && u.kind == UnitKind::Settler)
            .map(|u| u.pos)
            .collect();
        let mut best: Option<(i32, Hex)> = None;
        for (i, score) in ctx.site_score.iter().enumerate() {
            let Some(score) = *score else { continue };
            let reach = field[i];
            if reach == UNREACHABLE {
                continue;
            }
            let h = ctx.hex_at(i);
            if ctx.in_danger(h)
                || self
                    .settle_targets
                    .iter()
                    .chain(&other_settlers)
                    .any(|t| t.distance(h) <= MIN_CITY_GAP)
            {
                continue;
            }
            let turns = (reach + moves - 1) / moves;
            let mut value = if first_city {
                // Found the capital fast: anything beyond a short walk must be much better.
                let per_turn = if reach > FIRST_CITY_REACH { 60 } else { 25 };
                score * 100 / (100 + per_turn * turns)
            } else {
                if score < MIN_SITE_SCORE {
                    continue;
                }
                let home = ctx
                    .own_cities
                    .iter()
                    .map(|c| c.pos.distance(h))
                    .min()
                    .unwrap_or(0);
                if home > MAX_SITE_DISTANCE {
                    continue;
                }
                let mut v = score * 100 / (100 + 10 * turns);
                if home > 6 {
                    v = v * 100 / (100 + 15 * (home - 6));
                }
                if self.faction == Faction::India
                    && capital.is_some_and(|c| c.distance(h) > INDIA_CAPITAL_RADIUS)
                {
                    v = v * 60 / 100;
                }
                v
            };
            if h == s.pos {
                value = value * 110 / 100;
            }
            if s.goto == Some(h) {
                if Self::stuck(s) {
                    // Couldn't get there last turn: try somewhere else.
                    continue;
                }
                value = value * 130 / 100;
            }
            if best.is_none_or(|(b, _)| value > b) {
                best = Some((value, h));
            }
        }
        best.map(|(_, h)| h)
    }

    pub fn plan_scouts(&mut self) {
        let scouts: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.kind == UnitKind::Scout)
            .collect();
        let mut targets: Vec<Hex> = Vec::new();
        for s in scouts {
            match self.choose_frontier(s, &targets) {
                Some(t) => {
                    targets.push(t);
                    self.move_or_hold(s, t);
                }
                // Nothing left to explore: stop paying upkeep for it, or rest.
                None if self.view().economy.upkeep > 0 => {
                    self.unit_order(s.id, Order::DisbandUnit { unit: s.id });
                }
                None => {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
            }
        }
    }

    /// The nearest worthwhile frontier tile or ruin, away from visible enemy armies and from
    /// other scouts' targets.
    pub fn choose_frontier(&self, s: &UnitView, taken: &[Hex]) -> Option<Hex> {
        let ctx = &self.ctx;
        let field = ctx.distance_field(s.pos, true);
        let ruin_bonus = if self.faction == Faction::Byzantium {
            80
        } else {
            40
        };
        let mut best: Option<(i32, Hex)> = None;
        for &h in &ctx.frontier {
            if h == s.pos || ctx.in_danger(h) || taken.iter().any(|t| t.distance(h) <= 3) {
                continue;
            }
            let Some(i) = ctx.view.index(h) else { continue };
            let d = field[i];
            if d == UNREACHABLE {
                continue;
            }
            let ruin = ctx.view.tiles[i].ruin;
            let mut cost = d * 10 - ctx.unexplored_near(h) * 3;
            if ruin {
                cost -= ruin_bonus;
            }
            if s.goto == Some(h) {
                if Self::stuck(s) {
                    continue;
                }
                cost -= 15;
            }
            if best.is_none_or(|(b, _)| cost < b) {
                best = Some((cost, h));
            }
        }
        best.map(|(_, h)| h)
    }
}
