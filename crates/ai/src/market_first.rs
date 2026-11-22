//! Gene `capital_market_first` (2026-11-21): the designer has a Market by
//! T40 in every training game; Hard's first comes at T54-T98, and at T60 a Market is 7-9 Gold of
//! his 12-23 Gold lead. Earlier value floors left the Market count unchanged, so this is a forced
//! pick: an unthreatened capital, or a city of population 3 or more, starts its Market in place
//! of a Granary, a Workshop or (once it has its first defender) another troop. Hard's own view only.
use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{AiDifficulty, BuildingKind, ProductionItem, data};

/// No visible hostile military unit this close to the city.
const SAFE_RADIUS: i32 = 8;

impl Planner<'_> {
    /// The Market a city starts instead of `chosen`, when the gene applies; else `chosen`.
    pub(crate) fn market_first(
        &self,
        c: &CityView,
        chosen: Option<ProductionItem>,
    ) -> Option<ProductionItem> {
        let market = ProductionItem::Building {
            building: BuildingKind::Market,
        };
        if self.decisions.capital_market_first == 0 || self.difficulty != AiDifficulty::Hard {
            return chosen;
        }
        let Some(item) = chosen else {
            return chosen;
        };
        let defended = self
            .ctx
            .own_units
            .iter()
            .any(|u| u.military && u.pos.distance(c.pos) <= 1);
        let displaced = match item {
            ProductionItem::Building {
                building: BuildingKind::Granary | BuildingKind::Workshop,
            } => true,
            ProductionItem::Unit { unit } => crate::composition::is_combat(unit) && defended,
            _ => false,
        };
        if !displaced || !(c.is_capital || c.population >= 3) {
            return chosen;
        }
        let v = self.view();
        if v.units.iter().any(|u| {
            u.military
                && !v.same_team(v.me, u.owner)
                && (u.owner.is_barbarian() || v.at_war(v.me, u.owner))
                && u.pos.distance(c.pos) <= SAFE_RADIUS
        }) {
            return chosen;
        }
        let Some(d) = c.detail.as_ref() else {
            return chosen;
        };
        if data::has_building(&d.buildings, BuildingKind::Market) {
            return chosen;
        }
        d.options
            .iter()
            .any(|o| o.item == market && o.turns > 0 && o.resource_blocker.is_none())
            .then_some(market)
            .or(chosen)
    }
}

#[cfg(test)]
mod tests {
    use crate::fixture::{ENEMY, ME, ViewBuilder, off, planner};
    use legatum_core::{AiDifficulty, BuildingKind, PlayerView, ProductionItem, TechId, UnitKind};

    const MARKET: ProductionItem = ProductionItem::Building {
        building: BuildingKind::Market,
    };
    const SOLDIER: ProductionItem = ProductionItem::Unit {
        unit: UnitKind::Soldier,
    };

    /// A guarded capital that knows Currency, and an enemy Soldier `enemy_at` tiles away.
    fn view(enemy_at: i32) -> PlayerView {
        ViewBuilder::new(40, 20)
            .known_techs(&[TechId::Currency])
            .city_with_detail(1, ME, off(5, 5), 2)
            .unit(1, ME, UnitKind::Soldier, off(5, 5))
            .unit(2, ENEMY, UnitKind::Soldier, off(5 + enemy_at, 5))
            .build()
    }

    fn pick(v: &PlayerView, gene: i32, chosen: ProductionItem) -> Option<ProductionItem> {
        let mut p = planner(v, 1);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.capital_market_first = gene;
        p.decisions.egypt_food_buildings = 0;
        let c = p.ctx.own_cities[0];
        p.market_first(c, Some(chosen))
    }

    #[test]
    fn a_guarded_safe_capital_starts_its_market_instead_of_a_second_troop() {
        let v = view(20);
        assert_eq!(pick(&v, 1, SOLDIER), Some(MARKET));
        assert_eq!(pick(&v, 0, SOLDIER), Some(SOLDIER), "gene off: unchanged");
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        assert_eq!(
            pick(&v, 1, settler),
            Some(settler),
            "Settlers are not displaced"
        );
    }

    #[test]
    fn a_visible_enemy_near_the_city_keeps_the_troop() {
        let v = view(4);
        assert_eq!(pick(&v, 1, SOLDIER), Some(SOLDIER));
    }
}
