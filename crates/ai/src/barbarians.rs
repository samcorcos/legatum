//! Barbarian camps: an early army's job in Civ 6. Barbarian units are already enemies
//! everywhere else (analysis marks every non-empire owner hostile), so garrisons, escorts,
//! retreats and attacks treat them like any army at war. This pass adds only what no enemy
//! empire has: camps to clear. It reads `PlayerView::barbarian_camps` (seen or remembered
//! camps only), never the camps' true state.

use crate::analysis::{defense_power, offense_power};
use crate::goal_sparta_wild::DENIAL_CAMP_RANGE;
use crate::scripted::Planner;
use legatum_core::Hex;
use legatum_core::view::UnitView;

/// Known camps this close to one of our cities are worth clearing.
const CAMP_RANGE: i32 = 8;
/// Units this close to a camp may be sent to it.
const CLEAR_SEARCH: i32 = 10;
/// Healthy enough to walk out to a camp.
const CLEAR_MIN_HP: i32 = 60;
/// Gene `camp_land`: extra hexes from home for a camp whose land is worth a site.
const CAMP_LAND_RANGE: i32 = 4;
/// Gene `camp_land`: the land a camp blocks lies this close to it.
const CAMP_LAND_RADIUS: i32 = 3;

impl Planner<'_> {
    /// Gene `camp_land`: what clearing the camp at `pos` opens for settling, in site-score
    /// points: the best known legal site within `CAMP_LAND_RADIUS` and 3 per unexplored land
    /// tile there (the fog a camp keeps Scouts and Settlers out of), scaled by the gene's
    /// percent. Only our view; 0 with the gene off.
    pub(crate) fn camp_land_value(&self, pos: Hex) -> i32 {
        let gene = self.decisions.camp_land;
        if gene == 0 || !self.hard_defense() {
            return 0;
        }
        let v = self.view();
        let mut best = 0;
        let mut fog = 0;
        for h in pos.within(CAMP_LAND_RADIUS) {
            let Some(i) = v.index(h) else { continue };
            match v.tile(h) {
                Some(t) if t.explored => {
                    best = best.max(self.ctx.site_score[i].unwrap_or(0));
                }
                // Unexplored: count it as land only when we cannot rule it out.
                Some(t) if t.terrain.is_some_and(|t| !t.is_land()) => {}
                _ => fog += 1,
            }
        }
        (best + 3 * fog) * gene / 100
    }

    /// Send a free, healthy unit into an empty camp (or to look at a remembered one) along a
    /// safe route. Against a guarded camp, a ranged unit shoots the guard; otherwise two units
    /// close in when their strength beats the guard's, so the usual attack pass can take it.
    pub(crate) fn clear_barbarian_camps(&mut self, army: &[&UnitView]) {
        // A Celts that wants its Wild keeps one camp in every city (`wild_wanted`) and clears only
        // the doubles; rivals go farther, and first, for the camps of a Celts that scores them.
        // Gene `celts_playbook` part 2: the Celts always keep the camps in their land.
        let keep_own = self.wild_wanted() || self.celts_playbook(crate::celts_playbook::WILD);
        let mut own_left: std::collections::BTreeMap<legatum_core::CityId, usize> =
            std::collections::BTreeMap::new();
        if keep_own {
            for c in &self.view().barbarian_camps {
                if let Some(city) = self.territory_city(c.pos) {
                    *own_left.entry(city).or_default() += 1;
                }
            }
        }
        let mut camps: Vec<(bool, i32, Hex, bool)> = self
            .view()
            .barbarian_camps
            .iter()
            .filter_map(|c| {
                let home = self.ctx.nearest_own_city(c.pos)?.pos.distance(c.pos);
                let denial = self.wild_camp_denial(c.pos);
                // Gene `camp_land`: the land behind the camp sets the order, and land worth
                // a site lets a camp a little farther out be cleared.
                let land = self.camp_land_value(c.pos);
                let far = if land >= self.decisions.settler_min_site_score {
                    CAMP_LAND_RANGE
                } else {
                    0
                };
                let range = CAMP_RANGE + far + if denial { DENIAL_CAMP_RANGE } else { 0 };
                (home <= range).then_some((!denial, home * 10 - land, c.pos, c.visible))
            })
            .collect();
        camps.sort();
        for (not_denial, _, pos, visible) in camps {
            let denial = !not_denial;
            if keep_own && let Some(city) = self.territory_city(pos) {
                match own_left.get_mut(&city) {
                    Some(n) if *n > 1 => *n -= 1,
                    _ => continue,
                }
            }
            if self.mil_dest.contains(&pos) {
                continue;
            }
            let mut ready: Vec<&UnitView> = self
                .free(army)
                .into_iter()
                .filter(|u| u.hp >= CLEAR_MIN_HP && u.pos.distance(pos) <= CLEAR_SEARCH)
                .filter(|u| self.ctx.reachable(u.pos, pos))
                .collect();
            ready.sort_by_key(|u| (u.pos.distance(pos), u.id));
            let guard = self
                .ctx
                .enemy_units
                .iter()
                .find(|e| e.pos == pos && e.military)
                .copied();
            match guard {
                Some(g) if visible => {
                    // Guards never leave and barbarians never heal: a ranged unit can wear
                    // the guard down without retaliation.
                    if let Some(r) = ready
                        .iter()
                        .copied()
                        .find(|u| crate::composition::shooter_only(u.kind, u.ranged))
                    {
                        self.strike(r, pos, false);
                        continue;
                    }
                    let pair: Vec<&UnitView> = ready.into_iter().take(2).collect();
                    let mine: i32 = pair.iter().map(|u| offense_power(self.view(), u)).sum();
                    if pair.len() < 2
                        || mine * 100 / defense_power(self.view(), g).max(1)
                            < self.attack_threshold()
                    {
                        continue;
                    }
                    for u in pair {
                        if u.pos.distance(pos) <= 2 {
                            continue;
                        }
                        // The goal's denial walk is short and bounded; ordinary camp targeting
                        // keeps its broader approach rules.
                        let to = if denial {
                            self.wild_camp_approach(u, pos)
                        } else {
                            self.campaign_staging_spot(u, pos, 2)
                        };
                        if let Some(to) = to {
                            self.go_to(u, to);
                        }
                    }
                }
                _ => {
                    // Empty when seen, or remembered: walk in (clearing it) or go and look.
                    if let Some(u) = ready.first().copied()
                        && let Some(step) = if denial {
                            self.wild_camp_approach(u, pos)
                        } else {
                            self.campaign_approach(u, pos)
                        }
                    {
                        self.go_to(u, step);
                        self.mil_dest.insert(pos);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod camp_land_tests {
    //! Gene `camp_land` (land race, 2026-11-02).
    use crate::{
        Knobs,
        fixture::{ME, ViewBuilder, off},
        scripted::Planner,
    };
    use legatum_core::{AiDifficulty, Faction};

    #[test]
    fn a_camp_in_front_of_unexplored_land_is_worth_its_fog() {
        let v = ViewBuilder::new(30, 12)
            .city(1, ME, off(4, 6), 3)
            .unexplored_from_col(14)
            .build();
        let mut p = Planner::new(
            &v,
            Knobs::for_faction(Faction::Germany),
            Faction::Germany,
            3,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.camp_land = 100;
        let edge = p.camp_land_value(off(14, 6));
        let home = p.camp_land_value(off(7, 6));
        assert!(edge > home, "fog behind the camp counts ({edge} vs {home})");
        p.decisions.camp_land = 0;
        assert_eq!(p.camp_land_value(off(14, 6)), 0, "gene off");
    }
}
