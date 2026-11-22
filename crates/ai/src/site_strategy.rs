//! Hard's city placement by neighbour (the designer's placement strategy, 2026-11-07,
//! `docs/playtests/human-reasoning.md`): a site is worth its yields plus the rival expansion
//! it denies. Valuation only: the term adjusts the site value `choose_site_scored` already
//! compares, and gene `site_denial` at 0 leaves that value untouched. Only public
//! information and our own map are read.

use crate::scripted::Planner;
use legatum_core::{Hex, PlayerId, data, view::UnitView};

impl Planner<'_> {
    /// The placement gene's adjustment to a preferred site's `value`: plus denial. 0 when
    /// the gene is off, for the first city, or below Hard.
    pub(crate) fn site_strategy_value(&self, site: Hex, value: i32, settler: &UnitView) -> i32 {
        if self.decisions.site_denial == 0
            || !self.hard_defense()
            || value <= 0
            || self.ctx.own_cities.is_empty()
        {
            return 0;
        }
        let denial = self.site_denial_value(site, value);
        if denial != 0 {
            crate::civilians::settler_note(settler.id, || {
                format!("site {site:?} value {value}: denial +{denial}")
            });
        }
        denial
    }

    /// The rival whose nearest known city is closest to `site`, with that distance and the
    /// reach of its next Settlers (twice its city gap plus 2).
    fn nearest_rival_city(&self, site: Hex) -> Option<(PlayerId, Hex, i32, i32)> {
        let v = self.view();
        v.cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner))
            .map(|c| (c.pos.distance(site), c.owner, c.pos))
            .min_by_key(|(d, owner, pos)| (*d, owner.0, pos.q, pos.r))
            .map(|(d, owner, pos)| {
                let reach = 2 * v.rule_experiments.min_city_gap(v.faction_of(owner)) + 2;
                (owner, pos, d, reach)
            })
    }

    /// Gene `site_denial`: what founding here takes from the nearest rival. Inside its
    /// Settlers' reach, the site is credited for the open land behind it from the rival's
    /// side (land farther from the rival than the site and nearer the site than the rival,
    /// which our city would claim or cut off) and for strategic deposits it would hold.
    /// The gene is a percent of the site's value at full denial.
    pub(crate) fn site_denial_value(&self, site: Hex, value: i32) -> i32 {
        let gene = self.decisions.site_denial;
        if gene == 0 || value <= 0 {
            return 0;
        }
        let Some((_, rival_city, d, reach)) = self.nearest_rival_city(site) else {
            return 0;
        };
        if d > reach {
            return 0;
        }
        let v = self.view();
        let gap = v.rule_experiments.min_city_gap(v.faction_of(v.me));
        // The lane: open, settleable land behind the site, within two city gaps of it.
        let mut lane = 0;
        for h in site.within(2 * gap) {
            let Some(i) = v.index(h) else { continue };
            let t = &v.tiles[i];
            if !t.explored
                || t.owner.is_some()
                || !t
                    .terrain
                    .is_some_and(|x| x.is_land() && x.move_cost().is_some())
            {
                continue;
            }
            let from_rival = rival_city.distance(h);
            if from_rival > d && h.distance(site) < from_rival {
                lane += 1;
            }
        }
        // A full lane is two rings of land behind the site.
        let lane_percent = (lane * 100 / 18).min(100);
        let deposits = site
            .within(data::CITY_WORK_RADIUS)
            .into_iter()
            .filter_map(|h| v.tile(h))
            .filter(|t| {
                t.owner.is_none()
                    && t.resource
                        .is_some_and(legatum_core::resources::is_strategic)
            })
            .count() as i32;
        let denial_percent = (lane_percent + 30 * deposits.min(2)).min(150);
        let closeness = (reach + 1 - d) * 100 / reach;
        value * gene / 100 * denial_percent / 100 * closeness / 100
    }
}

#[cfg(test)]
mod tests {
    //! Gene `site_denial` (placement by neighbour, 2026-11-07).
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::{AiDifficulty, Faction, PlayerView, UnitKind};

    fn hard(v: &PlayerView, denial: i32) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(Faction::Germany), Faction::Germany, 3);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.site_denial = denial;
        p
    }

    fn with_enemy() -> PlayerView {
        ViewBuilder::new(30, 12)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(16, 6), 3)
            .unit(10, ME, UnitKind::Settler, off(5, 6))
            .build()
    }

    #[test]
    fn denial_credits_land_toward_a_rival_and_is_hard_only() {
        let v = with_enemy();
        let p = hard(&v, 100);
        assert!(p.site_denial_value(off(11, 6), 200) > 0);
        assert_eq!(
            p.site_denial_value(off(1, 2), 200),
            0,
            "out of the rival's reach"
        );
        assert_eq!(hard(&v, 0).site_denial_value(off(11, 6), 200), 0);
        let mut normal = hard(&v, 100);
        normal.difficulty = AiDifficulty::Normal;
        let settler = v.units[0].clone();
        assert_eq!(
            normal.site_strategy_value(off(11, 6), 200, &settler),
            0,
            "Hard only"
        );
    }
}
