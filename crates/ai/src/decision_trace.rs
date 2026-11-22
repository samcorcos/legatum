//! Observer-only record of options as they pass through Hard's decision pipeline.
use legatum_core::Order;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct DecisionOption {
    pub domain: &'static str,
    pub option: String,
    pub outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner_up_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displaced_by: Option<String>,
    #[serde(skip)]
    pub selected_order: Option<Order>,
}

impl DecisionOption {
    pub(crate) fn new(
        domain: &'static str,
        option: impl Into<String>,
        outcome: &'static str,
    ) -> Self {
        Self {
            domain,
            option: option.into(),
            outcome,
            gate: None,
            score: None,
            winner_score: None,
            runner_up_score: None,
            displaced_by: None,
            selected_order: None,
        }
    }
}

pub(crate) fn reconcile(
    options: &mut [DecisionOption],
    orders: &[Order],
    stages: &[crate::OrderStage],
    passes: &std::collections::BTreeMap<u32, Vec<&'static str>>,
    city_passes: &std::collections::BTreeMap<u32, Vec<&'static str>>,
) {
    for option in options {
        if option.outcome != "selected" {
            continue;
        }
        let Some(order) = &option.selected_order else {
            option.outcome = if option.domain == "plan" {
                "executed"
            } else {
                "selected_not_executed"
            };
            if option.outcome == "selected_not_executed" {
                option.displaced_by = Some("selection produced no final order".into());
            }
            continue;
        };
        if orders.contains(order) {
            option.outcome = "executed";
        } else {
            option.outcome = "selected_not_executed";
            option.displaced_by = stages
                .iter()
                .find_map(|stage| {
                    stage.rejected.iter().any(|r| &r.order == order).then(|| {
                        format!(
                            "{}: {}",
                            stage.name,
                            stage
                                .rejected
                                .iter()
                                .find(|r| &r.order == order)
                                .map_or("", |r| r.reason)
                        )
                    })
                })
                .or_else(|| {
                    match order {
                        Order::SetProduction { city, .. } | Order::BuyItem { city, .. } => {
                            if let Some(replacement) = orders.iter().find(|other| matches!(other,
                                Order::SetProduction { city: id, .. } | Order::BuyItem { city: id, .. } if id == city)) {
                                Some(format!("city order replaced by {replacement:?}; pass {}",
                                    city_passes.get(&city.0).and_then(|p| p.last()).copied().unwrap_or("unrecorded")))
                            } else {
                                Some("production selection emitted no final city order".to_owned())
                            }
                        }
                        other => {
                            let pass = legatum_core::rules::units::order_unit(other)
                                .and_then(|u| passes.get(&u.0))
                                .and_then(|p| p.last());
                            Some(pass.map_or_else(
                                || "unit selection emitted no final order".to_owned(),
                                |p| format!("later pass: {p}"),
                            ))
                        }
                    }
                });
        }
    }
}

/// Earlier city passes can assign production before routine scoring is reached. Include those
/// cities in the census as well, using the actual pass ownership and available item list.
pub(crate) fn complete_production(
    trace: &mut crate::OrderTrace,
    view: &legatum_core::PlayerView,
    orders: &[Order],
) {
    for (order, pass) in &trace.city_choices {
        if trace
            .decisions
            .iter()
            .any(|r| r.selected_order.as_ref() == Some(order))
        {
            continue;
        }
        let (domain, option) = match order {
            Order::SetProduction { city, item } => {
                ("production", format!("city={city:?} item={item:?}"))
            }
            Order::BuyItem { city, item } => (
                "purchase",
                format!("pass={pass} city={city:?} item={item:?}"),
            ),
            _ => continue,
        };
        let mut row = DecisionOption::new(domain, option, "selected");
        row.selected_order = Some(order.clone());
        trace.decisions.push(row);
    }
    let affordable = crate::supply::affordable(view);
    for city in view.cities.iter().filter(|c| c.owner == view.me) {
        let Some(detail) = &city.detail else {
            continue;
        };
        let available = affordable
            .cities
            .iter()
            .find(|c| c.id == city.id)
            .and_then(|c| c.detail.as_ref());
        for opt in &detail.options {
            if available.is_some_and(|d| d.options.iter().any(|a| a.item == opt.item)) {
                continue;
            }
            let mut row = DecisionOption::new(
                "production",
                format!("city={:?} item={:?}", city.id, opt.item),
                "gated_out",
            );
            row.gate = Some("strategic supply affordability".into());
            trace.decisions.push(row);
        }
    }
    for city in affordable
        .cities
        .iter()
        .filter(|c| c.owner == affordable.me)
    {
        let Some(detail) = &city.detail else {
            continue;
        };
        // Choosing the item already under construction needs no SetProduction order.
        // It is being executed by the city, not displaced by a later pass.
        for row in &mut trace.decisions {
            if row.outcome == "selected"
                && matches!(row.selected_order.as_ref(), Some(Order::SetProduction { city: id, item }) if *id == city.id && Some(*item) == detail.producing)
                && !orders.contains(row.selected_order.as_ref().expect("matched above"))
            {
                row.outcome = "executed";
            }
        }
        let prefix = format!("city={:?} item=", city.id);
        let ordered = orders.iter().find_map(|o| match o {
            Order::SetProduction { city: id, item } if *id == city.id => Some(*item),
            _ => None,
        });
        let current = detail.producing;
        let pass = trace
            .city_passes
            .get(&city.id.0)
            .and_then(|p| p.last())
            .copied();
        for opt in &detail.options {
            if trace
                .decisions
                .iter()
                .any(|r| r.domain == "production" && r.option == format!("{prefix}{:?}", opt.item))
            {
                continue;
            }
            let selected =
                ordered == Some(opt.item) || (ordered.is_none() && current == Some(opt.item));
            let mut row = DecisionOption::new(
                "production",
                format!("{prefix}{:?}", opt.item),
                if selected {
                    if ordered.is_some() {
                        "selected"
                    } else {
                        "executed"
                    }
                } else {
                    "gated_out"
                },
            );
            if row.outcome == "gated_out" {
                row.gate = Some(pass.map_or_else(
                    || "current production continues; no item comparison".to_owned(),
                    |p| format!("earlier planning pass: {p}"),
                ));
            }
            if selected && ordered.is_some() {
                row.selected_order = Some(Order::SetProduction {
                    city: city.id,
                    item: opt.item,
                });
            }
            trace.decisions.push(row);
        }
    }
}
