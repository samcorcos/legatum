//! Opt-in experimental personalities for strategy-diversity simulations. These affect only
//! decisions through the same fogged view, never resources, rules, or the default AI.
use crate::Knobs;
use legatum_core::{BuildingKind, ProductionItem, TechId, UnitKind, data};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Rush,
    Wide,
    Science,
    Turtle,
    Economic,
}

impl Strategy {
    pub const ALL: [Self; 5] = [
        Self::Rush,
        Self::Wide,
        Self::Science,
        Self::Turtle,
        Self::Economic,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rush => "rush",
            Self::Wide => "wide",
            Self::Science => "science",
            Self::Turtle => "turtle",
            Self::Economic => "economic",
        }
    }

    pub fn knobs(self) -> Knobs {
        match self {
            Self::Rush => Knobs::new(20, 100, 20, 30, 100),
            Self::Wide => Knobs::new(100, 35, 35, 65, 35),
            Self::Science => Knobs::new(25, 25, 100, 55, 10),
            Self::Turtle => Knobs::new(0, 65, 65, 65, 0),
            Self::Economic => Knobs::new(40, 30, 35, 100, 20),
        }
    }

    pub(crate) fn target_cities(self) -> i32 {
        match self {
            Self::Wide => 8,
            Self::Economic => 4,
            _ => 3,
        }
    }

    /// Multipliers apply to otherwise worthwhile, legal choices; emergency defense and
    /// immediate victory purchases still use the shared planner.
    pub(crate) fn production_weight(self, item: ProductionItem) -> i32 {
        match item {
            ProductionItem::Unit {
                unit: UnitKind::Settler,
            } => match self {
                Self::Wide => 400,
                Self::Rush => 50,
                _ => 100,
            },
            ProductionItem::Unit { unit }
                if !matches!(unit, UnitKind::Scout | UnitKind::Settler) =>
            {
                if self == Self::Rush {
                    400
                } else {
                    100
                }
            }
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                let family = data::building_family(building);
                let preferred = match self {
                    Self::Science => matches!(family, BuildingKind::Laboratory),
                    Self::Turtle => family == BuildingKind::Walls || data::is_great_work(building),
                    Self::Economic => family == BuildingKind::Market,
                    _ => false,
                };
                if preferred { 400 } else { 100 }
            }
            _ => 100,
        }
    }

    /// The fixed order a Rush opens an era with, income before the expensive upgrades. Only the
    /// Ancient era has one; later eras follow the weights below. Techs not in the era are
    /// skipped by the caller.
    pub(crate) fn opening_sequence(self, era: u8) -> &'static [TechId] {
        match (self, era) {
            (Self::Rush, 0) => &[
                TechId::BronzeWorking,
                TechId::Irrigation,
                TechId::Currency,
                TechId::Masonry,
                TechId::Writing,
            ],
            _ => &[],
        }
    }

    pub(crate) fn research_weight(self, tech: TechId) -> i32 {
        use TechId::*;
        let preferred = match self {
            Self::Rush => matches!(
                tech,
                Feudalism | Engineering | Machinery | Mathematics | Chivalry | Gunpowder
            ),
            Self::Wide => matches!(tech, Irrigation | CodeOfLaws | CropRotation),
            Self::Science => matches!(tech, Writing | Education | PrintingPress | Electronics),
            Self::Turtle => matches!(
                tech,
                Masonry | Feudalism | Writing | CodeOfLaws | Philosophy | MilitaryEngineering
            ),
            Self::Economic => matches!(tech, Currency | Banking | Industrialization),
        };
        if preferred { 400 } else { 100 }
    }
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for Strategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|a| a.name() == s)
            .ok_or_else(|| {
                "strategy must be rush, wide, science, turtle, or economic (diplomacy unavailable)"
                    .into()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, assert_valid, off};
    use crate::{Brain, ScriptedBrain};
    use legatum_core::{Faction, Order};

    #[test]
    fn rush_research_funds_the_army_before_the_expensive_ancient_techs() {
        let sequence = Strategy::Rush.opening_sequence(0);
        for i in 0..sequence.len() {
            let view = ViewBuilder::new(20, 20)
                .known_techs(&sequence[..i])
                .city(1, ME, off(5, 5), 3)
                .build();
            let mut planner = crate::scripted::Planner::new(
                &view,
                Strategy::Rush.knobs(),
                Faction::Switzerland,
                7,
            );
            planner.strategy = Some(Strategy::Rush);
            planner.plan_research();
            let Some(Order::SetResearchQueue { techs }) = planner.orders.first() else {
                panic!("no queue: {:?}", planner.orders);
            };
            assert_eq!(techs, &sequence[i..]);
            assert_valid(&view, &planner.orders);
        }
    }

    #[test]
    fn biases_make_distinct_legal_deterministic_research_choices() {
        let view = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::Irrigation, TechId::BronzeWorking])
            .city(1, ME, off(5, 5), 3)
            .unit(2, ME, UnitKind::Soldier, off(5, 5))
            .build();
        let mut choices = std::collections::BTreeSet::new();
        for strategy in Strategy::ALL {
            let mut brain = ScriptedBrain::with_strategy(Faction::Switzerland, 7, ME, strategy);
            let orders = brain.orders(&view);
            assert_valid(&view, &orders);
            let mut resumed = ScriptedBrain::with_strategy(Faction::Switzerland, 7, ME, strategy);
            assert_eq!(orders, resumed.orders(&view));
            let research = orders
                .iter()
                .find_map(|o| match o {
                    Order::SetResearchQueue { techs } => techs.first().copied(),
                    _ => None,
                })
                .unwrap();
            choices.insert(research);
            if strategy == Strategy::Science {
                assert_eq!(research, TechId::Writing);
            }
            if strategy == Strategy::Economic {
                assert_eq!(research, TechId::Currency);
            }
        }
        assert!(
            // Rush now shares the early income prerequisite with Economic; the
            // separate opening-sequence test pins its later military timing.
            choices.len() >= 2,
            "policies must do more than change their label: {choices:?}"
        );
    }
}
