//! Coach questions (designer, 2026-11-15): "Hard, and it asks why".
//!
//! When the human ends a turn, a shadow Hard brain plans his seat from his own view (its own RNG
//! stream and memory; the game's state, RNG and hash are never touched) and the plan is compared
//! with the orders he submitted. A material divergence becomes a templated question. This is
//! deterministic and calls no model. At most one question comes back per turn, never in the same
//! category as the previous turn's, and never about the same standing fact twice.
//!
//! Categories (`Category`) cover diplomacy, settling, manual builds and purchases, war spending,
//! threatened garrisons, border armies and Great Works. Thresholds are named constants below.
use crate::ScriptedBrain;
use legatum_core::diplomacy::TreatyKind;
use legatum_core::{Hex, Order, PlayerView, ProductionItem, TeamId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};

/// Settler site: Hard's best-site score over its score for the tile he founded on, times 100.
/// 200 = a ratio of 2.0. The paper test (`training-questions-paper-1115`, Byzantium T30) found one
/// real case at 2.3 and none above; settling is rare (7 candidates in 181 turns), so the bar is low.
pub const SETTLE_RATIO_X100: i64 = 200;
/// Production: Hard's score for its pick over its score for his pick, times 100. The paper's
/// threshold of R >= 4 gave 9 questions over 181 turns on the tape, none of which the cap dropped.
pub const PRODUCTION_RATIO_X100: i64 = 400;
/// Purchases: Gold Hard would have spent this turn that he did not (or the reverse). The paper
/// found 8 purchase-days at 750 and 15 at 500 over a game; the brief asks for 500.
pub const PURCHASE_GOLD: i32 = 500;
/// Strategic window: consecutive turns at war with a city damaged or threatened, and no Gold spending.
pub const WINDOW_TURNS: usize = 10;
/// Strategic window: turns in that span on which Hard would have bought something.
pub const WINDOW_HARD_BUY_TURNS: usize = 3;
/// A city counts as threatened when an enemy military unit at war with us is within this range.
pub const THREAT_RANGE: i32 = 3;
pub const BORDER_RANGE: i32 = 6;
pub const BORDER_MIN_UNITS: usize = 4;
pub const BORDER_DEFENCE_RATIO: usize = 2;
pub const EARLY_TURNS: u32 = 60;
pub const PRODUCTION_CAP: usize = 2;
pub const PURCHASE_CAP: usize = 3;
pub const GOLD_RESERVE_GAP: i32 = 500;
pub const WAR_RESERVE_MIN_UNITS: usize = 4;
pub const WIDE_VP_LEAD: u32 = 4;
pub const ROUTINE_MATERIALITY_CAP: i64 = 200_000;
pub const SERIOUS_CITY_DAMAGE_PERCENT: i32 = 25;
pub const SKIPS_TO_RAISE_THRESHOLD: usize = 2;
pub const SKIPPED_THRESHOLD_MULTIPLIER: i64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    WarPeace,
    Settle,
    Production,
    Purchase,
    Window,
    Garrison,
    BorderArmy,
    WarReserve,
    GreatWork,
}

/// What the app remembers between turns so it can pass it back: the keys already asked, and the
/// last question's turn and category.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Asked {
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub last: Option<(u32, Category)>,
    #[serde(default)]
    pub skipped: Vec<Category>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Question {
    pub turn: u32,
    pub category: Category,
    /// The standing fact asked about; it is never asked twice.
    pub key: String,
    pub materiality: i64,
    pub text: String,
    /// His orders and Hard's for this decision, and the scores where the code exposes them.
    pub evidence: Value,
}

#[derive(Clone, Debug, Default)]
struct TurnRow {
    turn: u32,
    at_war: bool,
    war_since: Option<u32>,
    pressure: bool,
    his_gold: i32,
    his_spend: i32,
    hard_gold: i32,
    treasury: i32,
    income: i32,
    hard_buys: Vec<ShadowBuy>,
    war_teams: Vec<TeamId>,
}

#[derive(Clone, Debug, Default)]
struct ShadowBuy {
    label: String,
    cost: i32,
    military: bool,
}

/// Per-turn observations of one seat, kept in memory for the window rule.
#[derive(Clone, Debug, Default)]
pub struct History {
    rows: VecDeque<TurnRow>,
    border_since: BTreeMap<TeamId, u32>,
}

pub struct Review {
    pub question: Option<Question>,
    /// Every candidate that qualified, best first (for the log).
    pub candidates: Vec<Question>,
    pub hard_orders: Vec<Order>,
}

/// Plan the seat with `brain` (a Hard brain used only for this) and compare with `his` orders.
/// `view` is the seat's own full view at the start of the turn.
pub fn review(
    brain: &mut ScriptedBrain,
    history: &mut History,
    view: &PlayerView,
    his: &[Order],
    asked: &Asked,
    next_city_name: &str,
) -> Review {
    let (hard, trace) = brain.order_diagnosis(view);
    let mut found: Vec<Question> = Vec::new();
    war_peace(view, his, &hard, &trace, &mut found);
    settle(brain, view, his, &hard, next_city_name, &mut found);
    production(view, &trace, his, &mut found);
    let row = observe(history, view, his, &hard);
    purchases(view, &row, his, &hard, &mut found);
    window(view, history, &row, &mut found);
    garrison(view, his, &hard, &row, &mut found);
    border_army(view, history, his, &mut found);
    war_reserve(view, history, &row, his, &hard, &mut found);
    great_work(view, his, &mut found);
    if view.turn <= EARLY_TURNS {
        for q in &mut found {
            if !matches!(q.category, Category::Production | Category::Purchase) {
                q.materiality = q.materiality.saturating_mul(2);
            }
        }
    }
    debug_assert!(found.iter().all(|q| player_words_only(&q.text)));
    found.retain(|q| !asked.keys.contains(&q.key));
    found.retain(|q| category_open(asked, q.category));
    found.retain(|q| {
        asked.skipped.iter().filter(|c| **c == q.category).count() < SKIPS_TO_RAISE_THRESHOLD
            || raised_threshold_met(q)
    });
    if win_in_hand(view) {
        found.retain(|q| {
            !matches!(
                q.category,
                Category::Production | Category::Purchase | Category::Window | Category::WarReserve
            )
        });
    }
    let lead = vp_lead(view);
    for q in &mut found {
        if matches!(
            q.category,
            Category::Settle
                | Category::Production
                | Category::Purchase
                | Category::Window
                | Category::Garrison
                | Category::BorderArmy
                | Category::WarReserve
                | Category::GreatWork
        ) {
            q.materiality /= i64::from(lead.saturating_add(1));
        }
    }
    if let Some((turn, category)) = asked.last
        && turn + 1 == view.turn
    {
        found.retain(|q| q.category != category);
    }
    found.sort_by(|a, b| b.materiality.cmp(&a.materiality).then(a.key.cmp(&b.key)));
    Review {
        question: found.first().cloned(),
        candidates: found,
        hard_orders: hard,
    }
}

// ---- helpers ----

fn team_name(view: &PlayerView, team: TeamId) -> String {
    if team == TeamId::BARBARIAN {
        return "barbarians".into();
    }
    let names: Vec<String> = view
        .players
        .iter()
        .filter(|p| p.team == team && !p.id.is_barbarian())
        .map(|p| legatum_core::data::faction_name(p.faction).to_string())
        .collect();
    if names.is_empty() {
        "an unknown empire".into()
    } else {
        names.join(" and ")
    }
}

fn team_subject(view: &PlayerView, team: TeamId) -> (String, &'static str) {
    let name = team_name(view, team);
    if (name.ends_with('s') && name != "Athens") || name.contains(" and ") {
        (format!("the {name}"), "have")
    } else {
        (name, "has")
    }
}

/// Hard's plan this turn as it reports it ("Economy", "Campaign ...", "Peace"), if exposed.
fn hard_plan(trace: &crate::OrderTrace) -> Option<String> {
    trace
        .decisions
        .iter()
        .find(|d| d.domain == "plan" && d.outcome == "executed")
        .map(|d| d.option.clone())
}

/// Hard's plan in player words: its kind and the empires it names ("make peace with Sparta").
fn plan_clause(view: &PlayerView, trace: &crate::OrderTrace, relevant: TeamId) -> String {
    let Some(plan) = hard_plan(trace) else {
        return String::new();
    };
    let kind = plan.split(" {").next().unwrap_or(&plan);
    let mut teams: Vec<TeamId> = view.players.iter().map(|p| p.team).collect();
    teams.sort();
    teams.dedup();
    let named: Vec<String> = teams
        .into_iter()
        .filter(|t| plan.contains(&format!("TeamId({})", t.0)))
        .map(|t| team_name(view, t))
        .collect();
    let who = named.join(" and ");
    if !plan.contains(&format!("TeamId({})", relevant.0)) {
        return String::new();
    }
    match (kind, named.is_empty()) {
        ("Peace", false) => format!(" Hard plans to make peace with {who}."),
        ("Contain", false) => format!(" Hard plans to contain {who}."),
        ("Campaign", false) => format!(" Hard plans a campaign against {who}."),
        ("Economy", _) => " Hard plans to focus on its economy.".to_string(),
        (_, true) => format!(" Hard plans {kind}."),
        _ => format!(" Hard plans {kind} against {who}."),
    }
}

/// Hard's executed plan is peace with `team`, whether or not an order of this turn says so.
fn plans_peace(trace: &crate::OrderTrace, team: TeamId) -> bool {
    hard_plan(trace).is_some_and(|p| p == format!("Peace {{ with: TeamId({}) }}", team.0))
}

fn held_back_clause(view: &PlayerView, trace: &crate::OrderTrace, team: TeamId) -> String {
    let candidate = trace.decisions.iter().find(|d| {
        d.domain == "plan"
            && d.outcome == "gated_out"
            && d.option.starts_with("Campaign target=CityId(")
            && view.cities.iter().any(|c| {
                view.team_of(c.owner) == team && d.option == format!("Campaign target={:?}", c.id)
            })
    });
    if let Some(d) = candidate {
        let city = view
            .cities
            .iter()
            .find(|c| d.option == format!("Campaign target={:?}", c.id));
        if let Some(city) = city {
            return match d.gate.as_deref() {
                Some(
                    "funded forecast unavailable"
                    | "in-reach campaign force gate"
                    | "capture odds below minimum",
                ) => format!(" Hard forecast the siege of {} failing.", city.name),
                Some("stale city sighting") => {
                    format!(" Hard had an old sighting of {}.", city.name)
                }
                Some("held capture") => " Hard was finishing another capture first.".into(),
                _ => format!(" Hard had no campaign ready against {}.", city.name),
            };
        }
    }
    if trace.production.iter().any(|p| {
        matches!(
            p.chosen,
            Some(ProductionItem::Building {
                building: legatum_core::BuildingKind::Walls
            })
        )
    }) {
        return " Hard was building Walls.".into();
    }
    match hard_plan(trace).as_deref() {
        Some("Economy") => " Hard was investing in its economy.".into(),
        _ => String::new(),
    }
}

fn spaced(raw: &str) -> String {
    let mut out = String::new();
    for (i, c) in raw.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

fn relative(from: Hex, to: Hex) -> String {
    let distance = from.distance(to);
    if distance == 0 {
        return "here".to_string();
    }
    let dq = i64::from(to.q - from.q);
    let dr = i64::from(to.r - from.r);
    let x = (2 * dq + dr) * 866;
    let y = dr * 1500;
    let (ax, ay) = (x.abs(), y.abs());
    let direction = if ay * 1000 >= ax * 2414 {
        if y < 0 { "N" } else { "S" }
    } else if ax * 1000 >= ay * 2414 {
        if x < 0 { "W" } else { "E" }
    } else {
        match (y < 0, x < 0) {
            (true, false) => "NE",
            (true, true) => "NW",
            (false, false) => "SE",
            (false, true) => "SW",
        }
    };
    format!(
        "{distance} tile{} {direction}",
        if distance == 1 { "" } else { "s" }
    )
}

fn place(view: &PlayerView, tile: Hex) -> String {
    view.cities
        .iter()
        .filter(|c| c.owner == view.me)
        .min_by_key(|c| c.pos.distance(tile))
        .map(|c| {
            if c.pos == tile {
                c.name.clone()
            } else {
                format!("{} of {}", relative(c.pos, tile), c.name)
            }
        })
        .unwrap_or_else(|| format!("tile ({}, {})", tile.q, tile.r))
}

pub(crate) fn item_label(view: &PlayerView, item: &ProductionItem) -> String {
    match item {
        ProductionItem::Improvement { improvement, tile } => {
            let resource = view
                .index(*tile)
                .and_then(|i| view.tiles.get(i))
                .and_then(|t| t.resource)
                .map(legatum_core::resources::name);
            let name = spaced(&format!("{improvement:?}"));
            match resource {
                Some(r) => format!("{name} on the {r} {}", place(view, *tile)),
                None => format!("{name} {}", place(view, *tile)),
            }
        }
        _ => basic_item_label(item),
    }
}

fn basic_item_label(item: &ProductionItem) -> String {
    let raw = match item {
        ProductionItem::Unit { unit } => format!("{unit:?}"),
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            format!("{building:?}")
        }
        ProductionItem::Project { project } => format!("{project:?}"),
        ProductionItem::Improvement { improvement, .. } => format!("{improvement:?}"),
    };
    spaced(&raw)
}

fn short_orders(orders: &[&Order]) -> Vec<String> {
    orders.iter().map(|o| format!("{o:?}")).collect()
}

fn player_words_only(text: &str) -> bool {
    !["{", "TeamId", "PlayerId", "Hex"]
        .iter()
        .any(|marker| text.contains(marker))
        && !text
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|w| {
                let number = w[1].trim_end_matches(|c: char| !c.is_ascii_digit());
                w[0].eq_ignore_ascii_case("team")
                    && !number.is_empty()
                    && number.chars().all(|c| c.is_ascii_digit())
            })
        && !text
            .split_whitespace()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|w| w[0] == "AI" && w[1].chars().next().is_some_and(|c| c.is_ascii_digit()))
}

fn raised_threshold_met(q: &Question) -> bool {
    let number = |key: &str| q.evidence.get(key).and_then(Value::as_i64).unwrap_or(0);
    match q.category {
        Category::Purchase => {
            number("his_gold").max(number("hard_gold"))
                >= i64::from(PURCHASE_GOLD) * SKIPPED_THRESHOLD_MULTIPLIER
        }
        Category::Production => {
            number("ratio_x100") >= PRODUCTION_RATIO_X100 * SKIPPED_THRESHOLD_MULTIPLIER
        }
        Category::Settle => {
            number("ratio_x100") >= SETTLE_RATIO_X100 * SKIPPED_THRESHOLD_MULTIPLIER
        }
        Category::Window => {
            number("hard_buy_turns") >= WINDOW_HARD_BUY_TURNS as i64 * SKIPPED_THRESHOLD_MULTIPLIER
        }
        Category::Garrison => {
            number("forecast_damage") * 100
                >= number("city_hp")
                    * i64::from(SERIOUS_CITY_DAMAGE_PERCENT)
                    * SKIPPED_THRESHOLD_MULTIPLIER
        }
        _ => q.materiality >= category_threshold(q.category) * SKIPPED_THRESHOLD_MULTIPLIER,
    }
}

fn category_threshold(category: Category) -> i64 {
    match category {
        Category::WarPeace => 1_000_000,
        Category::Garrison => 950_000,
        Category::Window | Category::WarReserve => 900_000,
        Category::GreatWork | Category::BorderArmy => 800_000,
        Category::Settle | Category::Production | Category::Purchase => 1,
    }
}

fn vp_lead(view: &PlayerView) -> u32 {
    let own = view
        .standings
        .iter()
        .find(|s| s.team == view.team)
        .map_or(0, |s| s.vp);
    let rival = view
        .standings
        .iter()
        .filter(|s| s.team != view.team)
        .map(|s| s.vp)
        .max()
        .unwrap_or(0);
    own.saturating_sub(rival)
}

fn win_in_hand(view: &PlayerView) -> bool {
    let Some(own) = view.standings.iter().find(|s| s.team == view.team) else {
        return false;
    };
    if own.vp.saturating_add(1) >= own.target || own.victory_hold.is_some() {
        return true;
    }
    view.faction_goal.as_ref().is_some_and(|goal| {
        !goal.done
            && goal.value > 0
            && own.vp.saturating_add(legatum_core::data::FACTION_GOAL_VP) >= own.target
    })
}

fn category_open(asked: &Asked, category: Category) -> bool {
    let (prefix, cap) = match category {
        Category::Production => ("production:", PRODUCTION_CAP),
        Category::Purchase => ("purchase:", PURCHASE_CAP),
        _ => return true,
    };
    asked
        .keys
        .iter()
        .filter(|key| key.starts_with(prefix))
        .count()
        < cap
}

fn purchase_city(orders: &[Order]) -> Option<legatum_core::CityId> {
    orders.iter().find_map(|o| match o {
        Order::BuyItem { city, .. } => Some(*city),
        _ => None,
    })
}

fn stake(view: &PlayerView, city: Option<legatum_core::CityId>, base: i64) -> i64 {
    let own = view.cities.iter().filter(|c| c.owner == view.me);
    let total: i64 = own
        .clone()
        .filter_map(|c| c.detail.as_ref())
        .map(|d| i64::from(d.yields.total.production.max(0)))
        .sum();
    let local = city
        .and_then(|id| own.clone().find(|c| c.id == id))
        .and_then(|c| c.detail.as_ref())
        .map_or(total, |d| i64::from(d.yields.total.production.max(0)));
    let share = if total > 0 {
        (local * 100 / total).clamp(5, 100)
    } else {
        100
    };
    let my_vp = view
        .players
        .iter()
        .find(|p| p.id == view.me)
        .map_or(0, |p| p.vp);
    let rival_vp = view
        .players
        .iter()
        .filter(|p| p.team != view.team && p.alive)
        .map(|p| p.vp)
        .max()
        .unwrap_or(0);
    let under_attack = view
        .players
        .iter()
        .any(|p| p.team != view.team && p.alive && view.teams_at_war(view.team, p.team))
        || view.cities.iter().filter(|c| c.owner == view.me).any(|c| {
            view.units
                .iter()
                .any(|u| u.military && u.owner != view.me && u.pos.distance(c.pos) <= THREAT_RANGE)
        });
    let race = if !under_attack && my_vp >= rival_vp.saturating_add(WIDE_VP_LEAD) {
        10
    } else {
        100
    };
    let early = if view.turn <= EARLY_TURNS { 3 } else { 1 };
    (base
        .saturating_mul(share)
        .saturating_mul(race)
        .saturating_mul(early)
        / 10_000)
        .min(ROUTINE_MATERIALITY_CAP)
}

// ---- war and peace ----

/// Teams each side declares war on, and makes peace with, this turn.
fn stances(view: &PlayerView, orders: &[Order]) -> (Vec<TeamId>, Vec<TeamId>) {
    let mut wars = Vec::new();
    let mut peaces = Vec::new();
    for o in orders {
        match o {
            Order::DeclareWar { against, .. } => wars.push(*against),
            Order::ProposeTreaty {
                to,
                kind: TreatyKind::Peace,
            } => peaces.push(*to),
            Order::RespondToProposal {
                proposal,
                accept: true,
                ..
            } => {
                if let Some(p) = view
                    .diplomacy
                    .proposals
                    .iter()
                    .find(|p| p.id == *proposal && p.kind == TreatyKind::Peace)
                {
                    peaces.push(view.diplomacy.counterparty(p, view.team));
                }
            }
            _ => {}
        }
    }
    wars.sort();
    wars.dedup();
    peaces.sort();
    peaces.dedup();
    (wars, peaces)
}

fn war_peace(
    view: &PlayerView,
    his: &[Order],
    hard: &[Order],
    trace: &crate::OrderTrace,
    out: &mut Vec<Question>,
) {
    let (his_wars, his_peaces) = stances(view, his);
    let (hard_wars, hard_peaces) = stances(view, hard);
    let orders_of = |orders: &[Order], team: TeamId, war: bool| -> Vec<String> {
        let list: Vec<&Order> = orders
            .iter()
            .filter(|o| match o {
                Order::DeclareWar { against, .. } => war && *against == team,
                Order::ProposeTreaty { to, kind } => {
                    !war && *to == team && *kind == TreatyKind::Peace
                }
                Order::RespondToProposal {
                    proposal,
                    accept: true,
                    ..
                } => {
                    !war && view.diplomacy.proposals.iter().any(|p| {
                        p.id == *proposal && view.diplomacy.counterparty(p, view.team) == team
                    })
                }
                _ => false,
            })
            .collect();
        short_orders(&list)
    };
    let mut push = |kind: &str, team: TeamId, his_did: bool, war: bool| {
        let who = team_name(view, team);
        let text = match (his_did, war) {
            (true, true) => format!("you declared war on {who}. Hard would not have declared war.{}", held_back_clause(view, trace, team)),
            (false, true) => format!("you did not declare war on {who}. Hard would have."),
            (true, false) => {
                if his.iter().any(|o| matches!(o, Order::ProposeTreaty { to, kind: TreatyKind::Peace } if *to == team)) {
                    format!("you offered {who} peace. Hard would not have made that offer.")
                } else {
                    format!("you accepted peace with {who}. Hard would not have.")
                }
            },
            (false, false) => {
                if view.diplomacy.proposals.iter().any(|p| {
                    p.kind == TreatyKind::Peace && view.diplomacy.counterparty(p, view.team) == team
                }) {
                    format!("{who} offered Peace. Why keep fighting?")
                } else {
                    format!("you did not make peace with {who}. Hard would have.")
                }
            }
        };
        let direct = !his_did && !war && text.ends_with('?');
        out.push(Question {
            turn: view.turn,
            category: Category::WarPeace,
            key: format!("warpeace:{kind}:{}", team.0),
            materiality: 1_000_000,
            text: if direct {
                format!("Turn {}: {text}", view.turn)
            } else {
                format!(
                    "Turn {}: {text}{} What made the difference?",
                    view.turn,
                    plan_clause(view, trace, team)
                )
            },
            evidence: json!({
                "his": orders_of(his, team, war),
                "hard": orders_of(hard, team, war),
                "hard_plan": hard_plan(trace),
            }),
        });
    };
    let at_war_with = |team: TeamId| view.teams_at_war(view.team, team);
    for &t in his_wars.iter().filter(|t| !hard_wars.contains(t)) {
        push("declare", t, true, true);
    }
    for &t in hard_wars.iter().filter(|t| !his_wars.contains(t)) {
        if !at_war_with(t) {
            push("declare", t, false, true);
        }
    }
    for &t in his_peaces.iter().filter(|t| !hard_peaces.contains(t)) {
        if !plans_peace(trace, t) {
            push("peace", t, true, false);
        }
    }
    for &t in hard_peaces.iter().filter(|t| !his_peaces.contains(t)) {
        if at_war_with(t) {
            push("peace", t, false, false);
        }
    }
    // An explicit refusal is a decision even when the Hard plan says Peace but it placed no
    // acceptance order (for example while another offer took the proposal slot).
    for o in his {
        let Order::RespondToProposal {
            proposal,
            accept: false,
            ..
        } = o
        else {
            continue;
        };
        let Some(p) = view
            .diplomacy
            .proposals
            .iter()
            .find(|p| p.id == *proposal && p.kind == TreatyKind::Peace)
        else {
            continue;
        };
        let team = view.diplomacy.counterparty(p, view.team);
        if plans_peace(trace, team) && !hard_peaces.contains(&team) {
            out.push(Question {
                turn: view.turn,
                category: Category::WarPeace,
                key: format!("warpeace:refuse:{}:{}", team.0, view.turn),
                materiality: 1_000_000,
                text: format!(
                    "Turn {}: {} offered Peace. Why keep fighting?",
                    view.turn,
                    team_name(view, team)
                ),
                evidence: json!({"his": format!("{o:?}"), "hard_plan": hard_plan(trace)}),
            });
        }
    }
}

// ---- observation and the strategic window ----

fn observe(history: &mut History, view: &PlayerView, his: &[Order], hard: &[Order]) -> TurnRow {
    let me = view.me;
    let at_war = view
        .players
        .iter()
        .any(|p| p.alive && p.id != me && !p.id.is_barbarian() && view.at_war(me, p.id));
    let enemy_near = |pos| {
        view.units.iter().any(|u| {
            u.military
                && u.owner != me
                && !view.team_of(u.owner).eq(&view.team)
                && view.at_war(me, u.owner)
                && u.pos.distance(pos) <= THREAT_RANGE
        })
    };
    let pressure = view
        .cities
        .iter()
        .filter(|c| c.owner == me)
        .any(|c| c.hp < c.max_hp || enemy_near(c.pos));
    let prev = history.rows.back().filter(|r| r.turn + 1 == view.turn);
    let war_since = match (at_war, prev.and_then(|r| r.war_since)) {
        (false, _) => None,
        (true, Some(since)) => Some(since),
        (true, None) => Some(view.turn),
    };
    let row = TurnRow {
        turn: view.turn,
        at_war,
        war_since,
        pressure,
        his_gold: purchase_gold(view, his),
        his_spend: his
            .iter()
            .filter_map(|o| shadow_spend(view, o))
            .map(|b| b.cost)
            .sum(),
        hard_gold: purchase_gold(view, hard),
        treasury: view.economy.gold,
        income: view.economy.gold_per_turn.total,
        hard_buys: hard.iter().filter_map(|o| shadow_spend(view, o)).collect(),
        war_teams: view
            .players
            .iter()
            .filter(|p| {
                p.alive
                    && p.team != view.team
                    && !p.id.is_barbarian()
                    && view.teams_at_war(view.team, p.team)
            })
            .map(|p| p.team)
            .collect(),
    };
    if history.rows.back().is_some_and(|r| r.turn + 1 != view.turn) {
        history.rows.clear();
    }
    history.rows.push_back(row.clone());
    while history.rows.len() > 30 {
        history.rows.pop_front();
    }
    let active: Vec<TeamId> = view
        .players
        .iter()
        .filter(|p| p.alive && p.team != view.team && !p.id.is_barbarian())
        .map(|p| p.team)
        .filter(|team| {
            view.cities.iter().filter(|c| c.owner == view.me).any(|c| {
                let enemy = view
                    .units
                    .iter()
                    .filter(|u| {
                        u.military
                            && view.team_of(u.owner) == *team
                            && u.pos.distance(c.pos) <= BORDER_RANGE
                    })
                    .count();
                let enemy_strength: i32 = view
                    .units
                    .iter()
                    .filter(|u| {
                        u.military
                            && view.team_of(u.owner) == *team
                            && u.pos.distance(c.pos) <= BORDER_RANGE
                    })
                    .map(|u| u.strength.max(0))
                    .sum();
                let local_strength: i32 = view
                    .units
                    .iter()
                    .filter(|u| {
                        u.owner == view.me && u.military && u.pos.distance(c.pos) <= BORDER_RANGE
                    })
                    .map(|u| u.strength.max(0))
                    .sum();
                enemy >= BORDER_MIN_UNITS
                    && enemy_strength >= local_strength.saturating_mul(BORDER_DEFENCE_RATIO as i32)
            })
        })
        .collect();
    history.border_since.retain(|team, _| active.contains(team));
    for team in active {
        history.border_since.entry(team).or_insert(view.turn);
    }
    row
}

fn shadow_spend(view: &PlayerView, order: &Order) -> Option<ShadowBuy> {
    match order {
        Order::BuyItem { city, item } => {
            let c = view.cities.iter().find(|c| c.id == *city)?;
            let cost = c
                .detail
                .as_ref()?
                .options
                .iter()
                .find(|op| op.item == *item)?
                .buy_cost?;
            Some(ShadowBuy {
                label: format!("{} in {}", item_label(view, item), c.name),
                cost,
                military: matches!(item, ProductionItem::Unit { unit } if *unit != legatum_core::UnitKind::Settler && *unit != legatum_core::UnitKind::Scout && *unit != legatum_core::UnitKind::Heir),
            })
        }
        Order::HireCompany { city, unit, seller } => Some(ShadowBuy {
            label: format!(
                "{} in {}",
                spaced(&format!("{unit:?}")),
                view.cities.iter().find(|c| c.id == *city)?.name
            ),
            cost: view
                .company_offers
                .iter()
                .find(|o| o.seller == *seller && o.unit == *unit)?
                .price,
            military: true,
        }),
        Order::UpgradeUnit { unit } => {
            let u = view.units.iter().find(|u| u.id == *unit)?;
            Some(ShadowBuy {
                label: format!(
                    "{} upgrade",
                    u.upgrade_to
                        .map_or("unit".into(), |kind| spaced(&format!("{kind:?}")))
                ),
                cost: u.upgrade_cost?,
                military: true,
            })
        }
        Order::UpgradeSiege { unit } => Some(ShadowBuy {
            label: "Siege upgrade".into(),
            cost: view
                .units
                .iter()
                .find(|u| u.id == *unit)?
                .siege_offer
                .as_ref()?
                .price
                .total,
            military: true,
        }),
        Order::UpgradeAircraft { plane, to } => {
            let a = view.aircraft.iter().find(|a| a.id == *plane)?;
            let cost = to
                .and_then(|kind| {
                    a.upgrade_options
                        .iter()
                        .find(|op| op.via == kind)
                        .map(|op| op.cost)
                })
                .or(a.upgrade_cost)?;
            Some(ShadowBuy {
                label: format!(
                    "{} upgrade",
                    to.map_or_else(
                        || spaced(&format!("{:?}", a.kind)),
                        |kind| spaced(&format!("{kind:?}"))
                    )
                ),
                cost,
                military: true,
            })
        }
        _ => None,
    }
}

/// Gold the `BuyItem` orders cost, read from the city's option prices in the view.
pub(crate) fn purchase_gold(view: &PlayerView, orders: &[Order]) -> i32 {
    orders
        .iter()
        .filter_map(|o| match o {
            Order::BuyItem { city, item } => view
                .cities
                .iter()
                .find(|c| c.id == *city)
                .and_then(|c| c.detail.as_ref())
                .and_then(|d| d.options.iter().find(|op| op.item == *item))
                .and_then(|op| op.buy_cost),
            _ => None,
        })
        .sum()
}

fn window(view: &PlayerView, history: &History, now: &TurnRow, out: &mut Vec<Question>) {
    let Some(since) = now.war_since else { return };
    if history.rows.len() < WINDOW_TURNS {
        return;
    }
    let span: Vec<&TurnRow> = history.rows.iter().rev().take(WINDOW_TURNS).rev().collect();
    let contiguous = span.windows(2).all(|w| w[0].turn + 1 == w[1].turn);
    let all_war = span.iter().all(|r| r.at_war && r.war_since == Some(since));
    let pressured = span.iter().filter(|r| r.pressure).count();
    let his_total: i32 = span.iter().map(|r| r.his_spend).sum();
    let mut treasury = span.first().map_or(0, |r| r.treasury.max(0));
    let available = span
        .iter()
        .take(WINDOW_TURNS - 1)
        .fold(treasury, |gold, r| gold.saturating_add(r.income));
    let mut hard_days = 0;
    let mut hard_total = 0;
    let mut kinds = std::collections::BTreeMap::<String, usize>::new();
    for (index, r) in span.iter().enumerate() {
        if index > 0 {
            treasury = treasury.saturating_add(span[index - 1].income);
        }
        let mut bought = false;
        for buy in &r.hard_buys {
            if buy.cost > 0 && buy.cost <= treasury && buy.military {
                treasury -= buy.cost;
                hard_total += buy.cost;
                *kinds.entry(buy.label.clone()).or_default() += 1;
                bought = true;
            } else if buy.cost > 0 && buy.cost <= treasury {
                treasury -= buy.cost;
            }
        }
        hard_days += usize::from(bought);
    }
    if !(contiguous
        && all_war
        && pressured >= 1
        && his_total == 0
        && hard_days >= WINDOW_HARD_BUY_TURNS)
    {
        return;
    }
    let first = span.first().map_or(view.turn, |r| r.turn);
    let named = kinds
        .into_iter()
        .map(|(label, count)| {
            if count == 1 {
                label
            } else {
                format!("{count} {label}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    out.push(Question {
        turn: view.turn,
        category: Category::Window,
        key: format!("window:war_no_buy:{since}"),
        materiality: 900_000,
        text: format!(
            "Turns {first} to {}: a city of yours was threatened during this war, and you bought nothing. With the Gold available across these turns, Hard would have bought {named} for {hard_total} Gold. What is your Gold for in this war?",
            view.turn
        ),
        evidence: json!({
            "war_since": since, "window": [first, view.turn],
            "his_purchase_gold": 0, "hard_purchase_gold": hard_total,
            "hard_buy_turns": hard_days, "turns_with_pressure": pressured,
            "available_gold": available,
        }),
    });
}

// ---- Settler sites ----

/// The first integer after `marker` in `note`.
fn int_after(note: &str, marker: &str) -> Option<i64> {
    let rest = &note[note.find(marker)? + marker.len()..];
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    digits.parse().ok()
}

/// Hard's best-site score and its score for `probe` from `settler_diagnosis` notes. The notes are
/// diagnostic strings ("chosen Some((212, Hex ..)) from ..", "probe tile candidate: score 94 ..").
pub(crate) fn site_scores(notes: &[String]) -> Option<(i64, i64)> {
    let best = notes
        .iter()
        .filter_map(|n| n.strip_prefix("chosen Some(("))
        .find_map(|n| int_after(n, ""))?;
    let probe = notes.iter().find_map(|n| {
        n.starts_with("probe tile candidate")
            .then(|| int_after(n, "score"))
            .flatten()
    })?;
    Some((best, probe))
}

fn settle(
    brain: &ScriptedBrain,
    view: &PlayerView,
    his: &[Order],
    hard: &[Order],
    next_city_name: &str,
    out: &mut Vec<Question>,
) {
    for o in his {
        let Order::FoundCity { unit } = o else {
            continue;
        };
        if hard.contains(o) {
            continue;
        }
        let Some(u) = view.units.iter().find(|u| u.id == *unit) else {
            continue;
        };
        let notes = brain.settler_diagnosis(view, *unit, Some(u.pos));
        let Some((best, here)) = site_scores(&notes) else {
            continue;
        };
        let ratio_x100 = if here <= 0 {
            100_000
        } else {
            best * 100 / here
        };
        if best > here && ratio_x100 >= SETTLE_RATIO_X100 {
            let hard_tile = notes
                .iter()
                .find(|n| n.starts_with("chosen Some(("))
                .and_then(|n| {
                    Some(Hex::new(
                        int_after(n, "q:")? as i32,
                        int_after(n, "r:")? as i32,
                    ))
                });
            let nearest = view
                .cities
                .iter()
                .filter(|c| c.owner == view.me)
                .min_by_key(|c| c.pos.distance(u.pos));
            let where_founded = nearest.map_or_else(
                || format!("tile ({}, {})", u.pos.q, u.pos.r),
                |c| format!("{} of {}", relative(c.pos, u.pos), c.name),
            );
            let hard_where = hard_tile.map_or_else(
                || "another site".to_string(),
                |h| format!("{} of your site", relative(u.pos, h)),
            );
            out.push(Question {
                turn: view.turn,
                category: Category::Settle,
                key: format!("settle:site:{}", unit.0),
                materiality: ratio_x100 * 100,
                text: format!(
                    "Turn {}: you founded {}, {where_founded}. Hard preferred a site {hard_where}. What made your site the right one?",
                    view.turn, next_city_name
                ),
                evidence: json!({"founded": format!("{:?}", u.pos), "hard_score_here": here,
                    "hard_best_score": best, "ratio_x100": ratio_x100,
                    "hard": short_orders(&hard.iter().filter(|h| legatum_core::rules::units::order_unit(h) == Some(*unit)).collect::<Vec<_>>())}),
            });
        }
    }
    // Hard would have founded this turn and he left the Settler without any order.
    for h in hard {
        let Order::FoundCity { unit } = h else {
            continue;
        };
        let ordered = his
            .iter()
            .any(|o| legatum_core::rules::units::order_unit(o) == Some(*unit));
        if ordered
            || view
                .units
                .iter()
                .all(|u| u.id != *unit || u.owner != view.me)
        {
            continue;
        }
        out.push(Question {
            turn: view.turn,
            category: Category::Settle,
            key: format!("settle:idle:{}", unit.0),
            materiality: 15_000,
            text: format!(
                "Turn {}: Hard would have founded a city with this Settler now, and you gave it no order. What are you waiting for?",
                view.turn
            ),
            evidence: json!({"hard": short_orders(&[h]), "his": []}),
        });
    }
}

// ---- production ----

fn production(
    view: &PlayerView,
    trace: &crate::OrderTrace,
    his: &[Order],
    out: &mut Vec<Question>,
) {
    for o in his {
        let Order::SetProduction { city, item } = o else {
            continue;
        };
        let Some(c) = view
            .cities
            .iter()
            .find(|c| c.id == *city && c.owner == view.me)
        else {
            continue;
        };
        // The client sends an automated choice as SetCityAutomation with `picked`, while older
        // saves may still carry SetProduction. Either way this city did not make a human choice.
        if c.detail.as_ref().is_some_and(|d| d.build_automation.is_some() || d.auto_queued == Some(*item))
            || his.iter().any(|o| matches!(o, Order::SetCityAutomation { city: id, focus: Some(_), .. } if id == city))
        { continue; }
        if c.detail.as_ref().and_then(|d| d.producing) == Some(*item) {
            continue;
        }
        let Some(entry) = trace.production.iter().find(|p| p.city == *city) else {
            continue;
        };
        let Some(pick) = entry.chosen.filter(|p| p != item) else {
            continue;
        };
        let score =
            |i: &ProductionItem| entry.options.iter().find(|o| &o.0 == i).map(|o| o.2 as i64);
        let (Some(hard_score), Some(his_score)) = (score(&pick), score(item)) else {
            continue;
        };
        let ratio_x100 = hard_score * 100 / his_score.max(1);
        if hard_score > 0 && ratio_x100 >= PRODUCTION_RATIO_X100 {
            out.push(Question {
                turn: view.turn,
                category: Category::Production,
                key: format!("production:{}:{}", city.0, item_label(view, item)),
                materiality: stake(view, Some(c.id), ratio_x100 * 50),
                text: format!(
                    "Turn {}: {} will build {}. Hard would build {} there. Why choose {} now?",
                    view.turn,
                    c.name,
                    item_label(view, item),
                    item_label(view, &pick),
                    item_label(view, item)
                ),
                evidence: json!({"city": c.name, "his": item_label(view, item), "hard": item_label(view, &pick),
                    "his_score": his_score, "hard_score": hard_score, "ratio_x100": ratio_x100}),
            });
        }
    }
}

// ---- purchases ----

fn buys(view: &PlayerView, orders: &[Order]) -> Vec<String> {
    orders
        .iter()
        .filter_map(|o| match o {
            Order::BuyItem { city, item } => {
                let c = view.cities.iter().find(|c| c.id == *city)?;
                let gold = c
                    .detail
                    .as_ref()?
                    .options
                    .iter()
                    .find(|op| op.item == *item)?
                    .buy_cost?;
                Some(format!(
                    "{} in {} ({gold} Gold)",
                    item_label(view, item),
                    c.name
                ))
            }
            _ => None,
        })
        .collect()
}

fn purchases(
    view: &PlayerView,
    row: &TurnRow,
    his: &[Order],
    hard: &[Order],
    out: &mut Vec<Question>,
) {
    let bucket = view.turn / 20;
    if row.hard_gold >= PURCHASE_GOLD && row.his_gold == 0 {
        out.push(Question {
            turn: view.turn,
            category: Category::Purchase,
            key: format!("purchase:hard_only:{bucket}"),
            materiality: stake(view, purchase_city(hard), i64::from(row.hard_gold) * 50),
            text: format!(
                "Turn {}: you bought nothing. Hard would have spent {} Gold: {}. What are you keeping your Gold for?",
                view.turn, row.hard_gold, buys(view, hard).join(", ")
            ),
            evidence: json!({"his_gold": 0, "hard_gold": row.hard_gold, "hard": buys(view, hard)}),
        });
    } else if row.his_gold >= PURCHASE_GOLD && row.hard_gold == 0 {
        out.push(Question {
            turn: view.turn,
            category: Category::Purchase,
            key: format!("purchase:his_only:{bucket}"),
            materiality: stake(view, purchase_city(his), i64::from(row.his_gold) * 50),
            text: format!(
                "Turn {}: you spent {} Gold on {}. Hard would have bought nothing. What did this buy that waiting would not?",
                view.turn, row.his_gold, buys(view, his).join(", ")
            ),
            evidence: json!({"his_gold": row.his_gold, "hard_gold": 0, "his": buys(view, his)}),
        });
    }
}

fn garrison(
    view: &PlayerView,
    his: &[Order],
    hard: &[Order],
    row: &TurnRow,
    out: &mut Vec<Question>,
) {
    let destination = |orders: &[Order], unit| {
        orders.iter().find_map(|o| match o {
            Order::MoveUnit { unit: id, to } | Order::MoveUnitPath { unit: id, to, .. }
                if *id == unit =>
            {
                Some(*to)
            }
            _ => None,
        })
    };
    for city in view.cities.iter().filter(|c| c.owner == view.me) {
        let stationed: Vec<_> = view
            .units
            .iter()
            .filter(|u| u.owner == view.me && u.military && u.pos == city.pos)
            .collect();
        let Some(defender) = stationed
            .iter()
            .find(|u| destination(his, u.id).is_some_and(|to| to != city.pos))
        else {
            continue;
        };
        if stationed
            .iter()
            .any(|u| destination(his, u.id).is_none_or(|to| to == city.pos))
            || view
                .units
                .iter()
                .filter(|u| u.owner == view.me && u.military && u.pos != city.pos)
                .any(|u| destination(his, u.id) == Some(city.pos))
        {
            continue;
        }
        let threats: Vec<_> = view
            .units
            .iter()
            .filter(|u| {
                u.military
                    && u.owner != view.me
                    && view.at_war(view.me, u.owner)
                    && u.pos.distance(city.pos) <= u.max_moves.max(u.range).max(1)
            })
            .collect();
        if threats.is_empty() {
            continue;
        }
        let Some(forecast_damage) = city_threatened_by_strike(view, city, &threats, defender.id)
        else {
            continue;
        };
        let hard_keeps = stationed
            .iter()
            .any(|u| destination(hard, u.id).is_none_or(|to| to == city.pos))
            || view
                .units
                .iter()
                .filter(|u| u.owner == view.me && u.military && u.pos != city.pos)
                .any(|u| destination(hard, u.id) == Some(city.pos));
        if !hard_keeps {
            continue;
        }
        let foe_team = view.team_of(threats[0].owner);
        let foe = team_name(view, foe_team);
        let war_id = view
            .diplomacy
            .relations
            .iter()
            .find(|r| {
                (r.a == view.team && r.b == foe_team) || (r.b == view.team && r.a == foe_team)
            })
            .and_then(|r| r.war_id)
            .unwrap_or(row.war_since.unwrap_or(view.turn));
        let kinds = threats
            .iter()
            .map(|u| spaced(&format!("{:?}", u.kind)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push(Question { turn: view.turn, category: Category::Garrison,
            key: format!("garrison:{}:{}:{war_id}", city.id.0, foe_team.0), materiality: 950_000,
            text: format!("Turn {}: you ordered your garrison out of {} while {} could strike it next turn ({} nearby). Hard kept a defender in the city. Why move it?", view.turn, city.name, foe, kinds),
            evidence: json!({"city": city.name, "enemy_units": threats.len(), "unit": defender.id.0, "forecast_damage": forecast_damage, "city_hp": city.hp}) });
    }
}

fn city_threatened_by_strike(
    view: &PlayerView,
    city: &legatum_core::view::CityView,
    threats: &[&legatum_core::view::UnitView],
    leaving: legatum_core::UnitId,
) -> Option<i32> {
    // A completed or affordable unit can take the vacated garrison tile before the next attack.
    let can_reinforce = city.detail.as_ref().is_some_and(|d| {
        let finishes = matches!(d.producing, Some(ProductionItem::Unit { unit }) if legatum_core::data::pays_upkeep(unit))
            && d.turns_to_complete.is_some_and(|turns| turns <= 1)
            && d.production_stall.is_none();
        let buys = d.options.iter().any(|o| matches!(o.item, ProductionItem::Unit { unit } if legatum_core::data::pays_upkeep(unit))
            && o.buy_cost.is_some_and(|gold| gold <= view.economy.gold)
            && o.buy_block.is_none());
        finishes || buys
    });
    if can_reinforce || (city.walls && threats.iter().all(|u| u.owner.is_barbarian())) {
        return None;
    }
    if !threats.iter().any(|u| {
        legatum_core::data::pays_upkeep(u.kind)
            && legatum_core::data::attacks_in_melee(u.kind)
            && u.pos.distance(city.pos) <= u.max_moves
    }) {
        return None;
    }
    let mut projected = view.clone();
    projected.units.retain(|u| u.id != leaving);
    let mut damage = 0;
    let mut capture = false;
    for enemy in threats
        .iter()
        .filter(|u| legatum_core::data::pays_upkeep(u.kind))
    {
        projected.me = enemy.owner;
        let melee = legatum_core::data::attacks_in_melee(enemy.kind)
            && enemy.pos.distance(city.pos) <= enemy.max_moves;
        let mode = if melee {
            legatum_core::AttackMode::Melee
        } else {
            legatum_core::AttackMode::Ranged
        };
        if let Some(forecast) =
            legatum_core::forecast_attack_mode(&projected, enemy.id, city.pos, mode)
        {
            damage += forecast.expected_damage_to_defender;
            capture |= melee && forecast.win_chance > 0;
        }
    }
    (capture || damage * 100 >= city.hp.max(1) * SERIOUS_CITY_DAMAGE_PERCENT).then_some(damage)
}

fn border_army(view: &PlayerView, history: &History, his: &[Order], out: &mut Vec<Question>) {
    let own_units = view
        .units
        .iter()
        .filter(|u| u.owner == view.me && u.military)
        .count();
    for city in view.cities.iter().filter(|c| c.owner == view.me) {
        for team in view
            .players
            .iter()
            .filter(|p| p.alive && p.team != view.team && !p.id.is_barbarian())
            .map(|p| p.team)
        {
            let enemy = view
                .units
                .iter()
                .filter(|u| {
                    u.military
                        && view.team_of(u.owner) == team
                        && u.pos.distance(city.pos) <= BORDER_RANGE
                })
                .count();
            let local = view
                .units
                .iter()
                .filter(|u| {
                    u.owner == view.me && u.military && u.pos.distance(city.pos) <= BORDER_RANGE
                })
                .count();
            let enemy_strength: i32 = view
                .units
                .iter()
                .filter(|u| {
                    u.military
                        && view.team_of(u.owner) == team
                        && u.pos.distance(city.pos) <= BORDER_RANGE
                })
                .map(|u| u.strength.max(0))
                .sum();
            let local_strength: i32 = view
                .units
                .iter()
                .filter(|u| {
                    u.owner == view.me && u.military && u.pos.distance(city.pos) <= BORDER_RANGE
                })
                .map(|u| u.strength.max(0))
                .sum();
            if enemy < BORDER_MIN_UNITS
                || enemy_strength < local_strength.saturating_mul(BORDER_DEFENCE_RATIO as i32)
            {
                continue;
            }
            let reacting = his.iter().any(|o| match o {
                Order::BuyItem {
                    city: id,
                    item: ProductionItem::Unit { unit },
                }
                | Order::SetProduction {
                    city: id,
                    item: ProductionItem::Unit { unit },
                } => {
                    *unit != legatum_core::UnitKind::Scout
                        && *unit != legatum_core::UnitKind::Settler
                        && view
                            .cities
                            .iter()
                            .any(|c| c.id == *id && c.pos.distance(city.pos) <= BORDER_RANGE)
                }
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => {
                    view.units.iter().any(|u| {
                        u.id == *unit
                            && u.owner == view.me
                            && u.military
                            && to.distance(city.pos) < u.pos.distance(city.pos)
                    })
                }
                Order::DeclareWar { against, .. } => *against == team,
                _ => false,
            });
            if reacting {
                continue;
            }
            let since = history
                .border_since
                .get(&team)
                .copied()
                .unwrap_or(view.turn);
            let (rival, verb) = team_subject(view, team);
            out.push(Question { turn: view.turn, category: Category::BorderArmy,
                key: format!("border:{}:{since}", team.0), materiality: 820_000 + (enemy as i64 - local as i64) * 1_000,
                text: if view.teams_at_war(view.team, team) {
                    format!("Turn {}: {rival} {verb} {enemy} military units within {BORDER_RANGE} tiles of {} and you have {own_units} in your whole empire. How will you defend it?", view.turn, city.name)
                } else {
                    format!("Turn {}: {rival} {verb} {enemy} military units within {BORDER_RANGE} tiles of {} and you have {own_units} in your whole empire. What is your plan if they declare?", view.turn, city.name)
                },
                evidence: json!({"enemy_near": enemy, "local_defenders": local, "empire_defenders": own_units}) });
        }
    }
}

fn war_reserve(
    view: &PlayerView,
    history: &History,
    row: &TurnRow,
    his: &[Order],
    hard: &[Order],
    out: &mut Vec<Question>,
) {
    let previous = history.rows.iter().rev().nth(1);
    let Some(previous) = previous else { return };
    let new_teams: Vec<_> = row
        .war_teams
        .iter()
        .filter(|t| !previous.war_teams.contains(t))
        .copied()
        .collect();
    if new_teams.is_empty() || his.iter().any(|o| matches!(o, Order::DeclareWar { .. })) {
        return;
    }
    let mut shadow = history.rows.front().map_or(row.treasury, |r| r.treasury);
    for r in history
        .rows
        .iter()
        .take(history.rows.len().saturating_sub(1))
    {
        for buy in &r.hard_buys {
            if buy.cost > 0 && buy.cost <= shadow {
                shadow -= buy.cost;
            }
        }
        shadow = shadow.saturating_add(r.income).max(0);
    }
    if row.treasury.saturating_sub(shadow) < GOLD_RESERVE_GAP {
        return;
    }
    for team in new_teams {
        let enemy = view
            .units
            .iter()
            .filter(|u| {
                u.military
                    && view.team_of(u.owner) == team
                    && view
                        .cities
                        .iter()
                        .any(|c| c.owner == view.me && c.pos.distance(u.pos) <= BORDER_RANGE)
            })
            .count();
        if enemy < WAR_RESERVE_MIN_UNITS {
            continue;
        }
        let (rival, _) = team_subject(view, team);
        let his_buys = buys(view, his);
        let hard_buys = buys(view, hard);
        if row.his_gold > 0 && his_buys == hard_buys {
            continue;
        }
        let (text, reason) = if row.his_gold > 0 && !hard_buys.is_empty() {
            (
                format!(
                    "Turn {}: {rival} just declared with {enemy} visible units while you held {} Gold. You bought {}. Hard would have bought {}. Why these units, and where will they defend?",
                    view.turn,
                    row.treasury,
                    his_buys.join(", "),
                    hard_buys.join(", ")
                ),
                "different purchases",
            )
        } else if row.his_gold > 0 {
            (
                format!(
                    "Turn {}: {rival} just declared with {enemy} visible units while you held {} Gold. You bought {}. Why this purchase, and where will it defend?",
                    view.turn,
                    row.treasury,
                    his_buys.join(", ")
                ),
                "purchase",
            )
        } else if row.his_spend > 0 {
            let spent = his
                .iter()
                .filter_map(|o| shadow_spend(view, o))
                .map(|b| b.label)
                .collect::<Vec<_>>()
                .join(", ");
            (
                format!(
                    "Turn {}: {rival} just declared with {enemy} visible units while you held {} Gold. You spent Gold on {spent}. Why that response?",
                    view.turn, row.treasury
                ),
                "other spending",
            )
        } else {
            (
                format!(
                    "Turn {}: you hold {} Gold and {rival} just declared with {enemy} visible units. What will you buy, and where?",
                    view.turn, row.treasury
                ),
                "unspent reserve",
            )
        };
        out.push(Question { turn: view.turn, category: Category::WarReserve,
            key: format!("war_reserve:{}:{}", team.0, view.turn), materiality: 910_000,
            text,
            evidence: json!({"gold": row.treasury, "shadow_reserve": shadow, "enemy_visible": enemy, "reason": reason}) });
    }
}

fn great_work(view: &PlayerView, his: &[Order], out: &mut Vec<Question>) {
    if !view
        .players
        .iter()
        .any(|p| p.team != view.team && p.alive && view.teams_at_war(view.team, p.team))
    {
        return;
    }
    for o in his {
        let (city, item) = match o {
            Order::SetProduction { city, item } => (city, item),
            Order::SetProductionQueue { city, queue } => {
                let Some(item) = queue.first() else { continue };
                (city, item)
            }
            _ => continue,
        };
        let ProductionItem::Wonder { building, .. } = item else {
            continue;
        };
        let Some(c) = view
            .cities
            .iter()
            .find(|c| c.id == *city && c.owner == view.me)
        else {
            continue;
        };
        if c.detail.as_ref().is_some_and(|d| d.build_automation.is_some() || matches!(d.producing, Some(ProductionItem::Wonder { building: current, .. }) if current == *building))
            || his.iter().any(|o| matches!(o, Order::SetCityAutomation { city: id, focus: Some(_), .. } if id == city)) { continue; }
        out.push(Question {
            turn: view.turn,
            category: Category::GreatWork,
            key: format!("great_work:{}:{building:?}", city.0),
            materiality: 850_000,
            text: format!(
                "Turn {}: Why start the {} now, in the middle of a war?",
                view.turn,
                spaced(&format!("{building:?}"))
            ),
            evidence: json!({"city": c.name, "work": format!("{building:?}")}),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{Faction, GameSetup, GameState, new_game, player_view};

    /// A game with seat 0's starting capital and `gold` in the treasury.
    fn with_city(gold: i32) -> GameState {
        let mut state = new_game(&GameSetup::standard(77, [Faction::Athens, Faction::India]));
        state.player_mut(legatum_core::PlayerId(0)).gold = gold;
        state
    }

    #[test]
    fn site_scores_read_the_settler_notes() {
        let notes = vec![
            "chosen Some((212, Hex { q: -2, r: 13 })) from Hex { q: 0, r: 0 }".to_string(),
            "probe tile candidate: score 94 quality Some(120) travel 3 nearest city None"
                .to_string(),
        ];
        assert_eq!(site_scores(&notes), Some((212, 94)));
        assert_eq!(site_scores(&notes[..1]), None);
        assert_eq!(
            site_scores(&["probe tile not a candidate: no route".to_string()]),
            None
        );
    }

    #[test]
    fn improvement_label_names_resource_and_relative_tile() {
        let state = with_city(100);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let city = view.cities.iter().find(|c| c.owner == view.me).unwrap();
        let tile = city.pos.neighbor(0);
        let index = view.index(tile).unwrap();
        view.tiles[index].resource = Some(legatum_core::resources::Resource::Cattle);
        let label = item_label(
            &view,
            &ProductionItem::Improvement {
                improvement: legatum_core::ImprovementKind::Pasture,
                tile,
            },
        );
        assert!(label.contains("Pasture on the Cattle"));
        assert!(label.contains("1 tile E of"));
        assert!(label.contains(&city.name));
        assert!(player_words_only(&label));
    }

    #[test]
    fn a_purchase_gap_of_500_gold_either_way_is_a_question_and_a_smaller_one_is_not() {
        let state = with_city(5000);
        let view = player_view(&state, legatum_core::PlayerId(0));
        let city = view.cities.iter().find(|c| c.owner == view.me).unwrap();
        let buy = city
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .filter_map(|o| o.buy_cost.map(|g| (o.item, g)))
            .max_by_key(|(_, g)| *g)
            .unwrap();
        assert!(buy.1 > 0);
        let order = Order::BuyItem {
            city: city.id,
            item: buy.0,
        };
        let gold = purchase_gold(&view, std::slice::from_ref(&order));
        assert_eq!(gold, buy.1);
        let row = |his, hard| TurnRow {
            his_gold: his,
            his_spend: his,
            hard_gold: hard,
            ..Default::default()
        };
        let mut out = Vec::new();
        purchases(
            &view,
            &row(0, gold.max(PURCHASE_GOLD)),
            &[],
            std::slice::from_ref(&order),
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].category, Category::Purchase);
        assert!(out[0].key.starts_with("purchase:hard_only"));
        let mut out = Vec::new();
        purchases(
            &view,
            &row(PURCHASE_GOLD, 0),
            std::slice::from_ref(&order),
            &[],
            &mut out,
        );
        assert!(out[0].key.starts_with("purchase:his_only"));
        let mut out = Vec::new();
        purchases(&view, &row(0, PURCHASE_GOLD - 1), &[], &[], &mut out);
        purchases(&view, &row(PURCHASE_GOLD - 1, 0), &[], &[], &mut out);
        purchases(&view, &row(900, 700), &[], &[], &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn a_new_build_that_hard_scores_four_times_lower_is_a_production_question() {
        let state = with_city(0);
        let view = player_view(&state, legatum_core::PlayerId(0));
        let city = view.cities.iter().find(|c| c.owner == view.me).unwrap();
        let current = city.detail.as_ref().unwrap().producing;
        let items: Vec<ProductionItem> = city
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|o| o.item)
            .filter(|i| Some(*i) != current)
            .collect();
        let (pick, his) = (items[0], items[1]);
        let mut trace = crate::OrderTrace::default();
        let scored = |ratio_x100: i32| crate::production::ProductionTrace {
            city: city.id,
            chosen: Some(pick),
            options: vec![(pick, 1000, ratio_x100 * 1000), (his, 100, 1000)],
            settler: None,
            gates: vec![],
            early_gate: None,
        };
        let order = vec![Order::SetProduction {
            city: city.id,
            item: his,
        }];
        trace.production = vec![scored(4)];
        let mut out = Vec::new();
        production(&view, &trace, &order, &mut out);
        assert_eq!(out.len(), 1, "ratio 4.0 asks");
        assert_eq!(out[0].category, Category::Production);
        assert!(out[0].text.contains("Hard would build"), "{}", out[0].text);
        trace.production = vec![scored(3)];
        let mut out = Vec::new();
        production(&view, &trace, &order, &mut out);
        assert!(out.is_empty(), "ratio 3.0 does not");
        // Keeping the build the city already has is not a decision.
        trace.production = vec![scored(9)];
        if let Some(cur) = current {
            let mut out = Vec::new();
            production(
                &view,
                &trace,
                &[Order::SetProduction {
                    city: city.id,
                    item: cur,
                }],
                &mut out,
            );
            assert!(out.is_empty());
        }
    }

    fn row(turn: u32, his: i32, hard: i32, since: u32) -> TurnRow {
        TurnRow {
            turn,
            at_war: true,
            war_since: Some(since),
            pressure: turn == 5,
            his_gold: his,
            his_spend: his,
            hard_gold: hard,
            treasury: 1000,
            income: 0,
            hard_buys: if hard > 0 {
                vec![ShadowBuy {
                    label: "Spearman in Athens".into(),
                    cost: hard,
                    military: true,
                }]
            } else {
                Vec::new()
            },
            war_teams: Vec::new(),
        }
    }

    #[test]
    fn ten_war_turns_with_pressure_no_purchase_and_three_hard_buys_is_a_window_question() {
        let state = with_city(0);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.turn = 14;
        let mk = |his: i32, buys: &[u32]| History {
            rows: (5..=14)
                .map(|t| {
                    row(
                        t,
                        if t == 14 { his } else { 0 },
                        if buys.contains(&t) { 300 } else { 0 },
                        3,
                    )
                })
                .collect(),
            ..Default::default()
        };
        let go = |h: &History| {
            let mut out = Vec::new();
            window(&view, h, h.rows.back().unwrap(), &mut out);
            out
        };
        let q = go(&mk(0, &[6, 9, 12]));
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].category, Category::Window);
        assert_eq!(q[0].key, "window:war_no_buy:3");
        assert!(q[0].text.contains("Spearman in Athens"));
        assert!(
            go(&mk(0, &[6, 9])).is_empty(),
            "two Hard buys is not enough"
        );
        assert!(go(&mk(300, &[6, 9, 12])).is_empty(), "he bought something");
        let mut short = mk(0, &[6, 9, 12]);
        short.rows.pop_front();
        assert!(go(&short).is_empty(), "under ten turns of history");
        let mut calm = mk(0, &[6, 9, 12]);
        calm.rows.iter_mut().for_each(|r| r.pressure = false);
        assert!(go(&calm).is_empty(), "no city damaged or threatened");
    }

    #[test]
    fn athens_170_gold_cannot_be_spent_eight_times() {
        let state = with_city(170);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.turn = 46;
        let mut history = History {
            rows: (37..=46)
                .map(|t| {
                    let mut r = row(t, 0, 127, 37);
                    r.treasury = 170 + (t - 37) as i32 * 2;
                    r.income = 2;
                    r.pressure = true;
                    r
                })
                .collect(),
            ..Default::default()
        };
        let mut out = Vec::new();
        window(&view, &history, history.rows.back().unwrap(), &mut out);
        assert!(out.is_empty(), "only one 127-Gold purchase is affordable");
        for r in &mut history.rows {
            r.hard_buys[0].cost = 50;
        }
        window(&view, &history, history.rows.back().unwrap(), &mut out);
        assert_eq!(out.len(), 1);
        assert!(out[0].evidence["hard_purchase_gold"].as_i64().unwrap() <= 170 + 9 * 2);
        assert!(out[0].text.contains("Spearman in Athens"));
    }

    #[test]
    fn an_economic_shadow_window_is_not_a_defence_question() {
        let state = with_city(1000);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.turn = 14;
        let history = History {
            rows: (5..=14)
                .map(|t| {
                    let mut r = row(t, 0, 100, 5);
                    r.hard_buys[0].label = "Pasture in Athens".into();
                    r.hard_buys[0].military = false;
                    r.pressure = true;
                    r
                })
                .collect(),
            ..Default::default()
        };
        let mut out = Vec::new();
        window(&view, &history, history.rows.back().unwrap(), &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn moving_a_garrison_out_of_strike_range_asks_once_per_city_and_war() {
        let state = with_city(100);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let city = view
            .cities
            .iter()
            .find(|c| c.owner == view.me)
            .unwrap()
            .clone();
        let ours = view
            .units
            .iter()
            .find(|u| u.owner == view.me && u.military)
            .unwrap()
            .id;
        for u in view
            .units
            .iter_mut()
            .filter(|u| u.owner == view.me && u.military && u.id != ours)
        {
            u.pos = city.pos.neighbor(3);
        }
        view.units.iter_mut().find(|u| u.id == ours).unwrap().pos = city.pos;
        let opponent = view
            .players
            .iter()
            .find(|p| p.team != view.team)
            .unwrap()
            .id;
        let mut enemy = view.units.iter().find(|u| u.id == ours).unwrap().clone();
        enemy.id = legatum_core::UnitId(999);
        enemy.owner = opponent;
        enemy.kind = legatum_core::UnitKind::Horseman;
        enemy.pos = city.pos.neighbor(0);
        view.units.push(enemy);
        let threatened = view.cities.iter_mut().find(|c| c.id == city.id).unwrap();
        threatened.hp = 1;
        threatened.detail.as_mut().unwrap().options.clear();
        threatened.detail.as_mut().unwrap().producing = None;
        view.diplomacy.legacy = true;
        let his = [Order::MoveUnit {
            unit: ours,
            to: city.pos.neighbor(1),
        }];
        let mut out = Vec::new();
        garrison(&view, &his, &[], &row(40, 0, 0, 37), &mut out);
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains(&city.name));
        assert!(out[0].materiality > 900_000);
        let mut retained = Vec::new();
        garrison(&view, &his, &his, &row(40, 0, 0, 37), &mut retained);
        assert!(retained.is_empty(), "Hard also moved it");
    }

    #[test]
    fn scout_single_horseman_and_walled_barbarian_are_not_garrison_threats() {
        let state = with_city(0);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.diplomacy.legacy = true;
        let mut city = view
            .cities
            .iter()
            .find(|c| c.owner == view.me)
            .unwrap()
            .clone();
        city.hp = city.max_hp;
        let mut enemy = view
            .units
            .iter()
            .find(|u| u.owner == view.me && u.military)
            .unwrap()
            .clone();
        enemy.owner = view
            .players
            .iter()
            .find(|p| p.team != view.team)
            .unwrap()
            .id;
        enemy.pos = city.pos.neighbor(0);
        enemy.id = legatum_core::UnitId(998);
        view.units.push(enemy.clone());
        enemy.kind = legatum_core::UnitKind::Scout;
        assert!(
            city_threatened_by_strike(&view, &city, &[&enemy], legatum_core::UnitId(999)).is_none()
        );
        enemy.kind = legatum_core::UnitKind::Horseman;
        view.units
            .iter_mut()
            .find(|u| u.id == enemy.id)
            .unwrap()
            .kind = enemy.kind;
        assert!(
            city_threatened_by_strike(&view, &city, &[&enemy], legatum_core::UnitId(999)).is_none()
        );
        enemy.kind = legatum_core::UnitKind::CompositeArcher;
        assert!(
            city_threatened_by_strike(&view, &city, &[&enemy], legatum_core::UnitId(999)).is_none()
        );
        enemy.owner = legatum_core::PlayerId::BARBARIAN;
        city.walls = true;
        assert!(
            city_threatened_by_strike(&view, &city, &[&enemy], legatum_core::UnitId(999)).is_none()
        );
        assert_eq!(team_name(&view, TeamId::BARBARIAN), "barbarians");
    }

    #[test]
    fn victory_proximity_and_skips_suppress_routine_questions() {
        let state = with_city(0);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let own = view
            .standings
            .iter_mut()
            .find(|s| s.team == view.team)
            .unwrap();
        own.vp = own.target.saturating_sub(1);
        assert!(win_in_hand(&view));
        let asked = Asked {
            skipped: vec![Category::Purchase, Category::Purchase],
            ..Default::default()
        };
        assert_eq!(
            asked
                .skipped
                .iter()
                .filter(|c| **c == Category::Purchase)
                .count(),
            SKIPS_TO_RAISE_THRESHOLD
        );
        let q = Question {
            turn: 1,
            category: Category::Purchase,
            key: "purchase:x".into(),
            materiality: 1,
            text: String::new(),
            evidence: json!({"hard_gold": PURCHASE_GOLD}),
        };
        assert!(!raised_threshold_met(&q));
        let larger = Question {
            evidence: json!({"hard_gold": PURCHASE_GOLD * 2}),
            ..q
        };
        assert!(raised_threshold_met(&larger));
        assert!(vp_lead(&view) > 0);
    }

    #[test]
    fn automated_production_and_pending_automation_are_never_human_choices() {
        let state = with_city(0);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let city = view.cities.iter().find(|c| c.owner == view.me).unwrap().id;
        let items: Vec<_> = view
            .cities
            .iter()
            .find(|c| c.id == city)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|o| o.item)
            .take(2)
            .collect();
        let mut trace = crate::OrderTrace::default();
        trace.production.push(crate::production::ProductionTrace {
            city,
            chosen: Some(items[0]),
            options: vec![(items[0], 1000, 5000), (items[1], 100, 1000)],
            settler: None,
            gates: vec![],
            early_gate: None,
        });
        let order = Order::SetProduction {
            city,
            item: items[1],
        };
        view.cities
            .iter_mut()
            .find(|c| c.id == city)
            .unwrap()
            .detail
            .as_mut()
            .unwrap()
            .build_automation = Some(legatum_core::CityBuildFocus::Balanced);
        let mut out = Vec::new();
        production(&view, &trace, std::slice::from_ref(&order), &mut out);
        assert!(out.is_empty());
        view.cities
            .iter_mut()
            .find(|c| c.id == city)
            .unwrap()
            .detail
            .as_mut()
            .unwrap()
            .build_automation = None;
        production(
            &view,
            &trace,
            &[
                order,
                Order::SetCityAutomation {
                    city,
                    focus: Some(legatum_core::CityBuildFocus::Balanced),
                    picked: Some(items[1]),
                },
            ],
            &mut out,
        );
        assert!(out.is_empty());
    }
    #[test]
    fn a_peace_plan_counts_as_peace_and_reads_in_player_words() {
        let state = with_city(100);
        let view = player_view(&state, legatum_core::PlayerId(0));
        let other = view.players.iter().find(|p| p.team != view.team).unwrap();
        let mut trace = crate::OrderTrace::default();
        trace.decisions.push(crate::DecisionOption::new(
            "plan",
            format!("Peace {{ with: TeamId({}) }}", other.team.0),
            "executed",
        ));
        assert!(plans_peace(&trace, other.team));
        assert!(!plans_peace(&trace, view.team));
        assert_eq!(
            plan_clause(&view, &trace, other.team),
            format!(
                " Hard plans to make peace with {}.",
                legatum_core::data::faction_name(other.faction)
            )
        );
    }

    #[test]
    fn offered_peace_does_not_claim_acceptance_or_name_an_unrelated_plan() {
        let state = with_city(0);
        let view = player_view(&state, legatum_core::PlayerId(0));
        let other = view.players.iter().find(|p| p.team != view.team).unwrap();
        let mut trace = crate::OrderTrace::default();
        trace.decisions.push(crate::DecisionOption::new(
            "plan",
            "Peace { with: TeamId(99) }",
            "executed",
        ));
        let mut out = Vec::new();
        war_peace(
            &view,
            &[Order::ProposeTreaty {
                to: other.team,
                kind: TreatyKind::Peace,
            }],
            &[],
            &trace,
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert!(
            out[0]
                .text
                .contains(&format!("offered {} peace", team_name(&view, other.team)))
        );
        assert!(!out[0].text.contains("team 99"));
        assert!(!out[0].text.contains("made peace"));
    }

    #[test]
    fn declined_war_uses_its_city_plan_gate() {
        let state = with_city(0);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let other = view
            .players
            .iter()
            .find(|p| p.team != view.team)
            .unwrap()
            .clone();
        let mut target = view
            .cities
            .iter()
            .find(|c| c.owner == view.me)
            .unwrap()
            .clone();
        target.owner = other.id;
        target.id = legatum_core::CityId(500);
        target.name = "Tara".into();
        view.cities.push(target);
        let mut trace = crate::OrderTrace::default();
        let mut row =
            crate::DecisionOption::new("plan", "Campaign target=CityId(500)", "gated_out");
        row.gate = Some("funded forecast unavailable".into());
        trace.decisions.push(row);
        let mut out = Vec::new();
        war_peace(
            &view,
            &[Order::DeclareWar {
                against: other.team,
                breach: false,
            }],
            &[],
            &trace,
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert!(
            out[0]
                .text
                .contains("Hard forecast the siege of Tara failing")
        );
    }

    #[test]
    fn every_question_category_rejects_engine_debug_text() {
        let categories = [
            Category::WarPeace,
            Category::Settle,
            Category::Production,
            Category::Purchase,
            Category::Window,
            Category::Garrison,
            Category::BorderArmy,
            Category::WarReserve,
            Category::GreatWork,
        ];
        for category in categories {
            let q = Question {
                turn: 1,
                category,
                key: String::new(),
                materiality: 1,
                text: "A question about Athens".into(),
                evidence: Value::Null,
            };
            assert!(player_words_only(&q.text));
            for marker in ["{", "TeamId", "PlayerId", "Hex", "AI 4", "team 255"] {
                assert!(
                    !player_words_only(&format!("{} {marker}", q.text)),
                    "{category:?}: {marker}"
                );
            }
        }
    }

    #[test]
    fn asked_keys_cap_only_production_and_purchase() {
        let asked = Asked {
            keys: [
                "production:1:a",
                "production:2:b",
                "purchase:a",
                "purchase:b",
                "purchase:c",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            last: None,
            skipped: vec![],
        };
        assert!(!category_open(&asked, Category::Production));
        assert!(!category_open(&asked, Category::Purchase));
        for category in [
            Category::WarPeace,
            Category::Settle,
            Category::Window,
            Category::Garrison,
            Category::BorderArmy,
            Category::WarReserve,
            Category::GreatWork,
        ] {
            assert!(category_open(&asked, category));
        }
    }

    #[test]
    fn materiality_follows_city_stake_war_and_vp_race() {
        let state = with_city(100);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.turn = 130;
        let main = view
            .cities
            .iter()
            .find(|c| c.owner == view.me)
            .unwrap()
            .clone();
        let mut colony = main.clone();
        colony.id = legatum_core::CityId(999);
        colony.detail.as_mut().unwrap().yields.total.production = 1;
        view.cities.push(colony);
        let city_share = stake(&view, Some(main.id), 100_000);
        let colony_share = stake(&view, Some(legatum_core::CityId(999)), 100_000);
        assert!(colony_share < city_share);
        view.players
            .iter_mut()
            .find(|p| p.id == view.me)
            .unwrap()
            .vp = 10;
        let won_race = stake(&view, Some(main.id), 100_000);
        assert!(won_race < city_share);
        view.diplomacy.legacy = true;
        assert!(stake(&view, Some(main.id), 100_000) > won_race);
        assert_eq!(
            stake(&view, Some(main.id), i64::MAX),
            ROUTINE_MATERIALITY_CAP
        );
    }

    #[test]
    fn a_visible_border_build_up_asks_unless_he_reacts() {
        let state = with_city(100);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let city = view
            .cities
            .iter()
            .find(|c| c.owner == view.me)
            .unwrap()
            .clone();
        let opponent = view
            .players
            .iter()
            .find(|p| p.team != view.team)
            .unwrap()
            .id;
        let mut enemy = view
            .units
            .iter()
            .find(|u| u.owner == view.me && u.military)
            .unwrap()
            .clone();
        enemy.owner = opponent;
        for u in view
            .units
            .iter_mut()
            .filter(|u| u.owner == view.me && u.military)
        {
            u.pos = Hex::new(-100, -100);
        }
        for i in 0..BORDER_MIN_UNITS {
            let mut u = enemy.clone();
            u.id = legatum_core::UnitId(200 + i as u32);
            u.pos = city.pos.neighbor(i);
            view.units.push(u);
        }
        let mut history = History::default();
        history.border_since.insert(view.team_of(enemy.owner), 10);
        let mut out = Vec::new();
        border_army(&view, &history, &[], &mut out);
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains(&city.name));
        let mut reacted = Vec::new();
        border_army(
            &view,
            &history,
            &[Order::DeclareWar {
                against: view.team_of(enemy.owner),
                breach: false,
            }],
            &mut reacted,
        );
        assert!(reacted.is_empty());
    }

    #[test]
    fn a_new_war_with_gold_far_above_shadow_reserve_asks() {
        let state = with_city(800);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.turn = 50;
        let enemy = view
            .players
            .iter()
            .find(|p| p.team != view.team)
            .unwrap()
            .team;
        let mut source = view
            .units
            .iter()
            .find(|u| u.owner == view.me && u.military)
            .unwrap()
            .clone();
        source.owner = view.players.iter().find(|p| p.team == enemy).unwrap().id;
        let city_pos = view.cities.iter().find(|c| c.owner == view.me).unwrap().pos;
        for i in 0..WAR_RESERVE_MIN_UNITS {
            let mut u = source.clone();
            u.id = legatum_core::UnitId(300 + i as u32);
            u.pos = city_pos.neighbor(i);
            view.units.push(u);
        }
        let mut before = row(49, 0, 0, 49);
        before.treasury = 100;
        before.war_teams.clear();
        let mut now = row(50, 0, 0, 50);
        now.treasury = 800;
        now.war_teams = vec![enemy];
        let history = History {
            rows: [before, now.clone()].into(),
            ..Default::default()
        };
        let mut out = Vec::new();
        war_reserve(&view, &history, &now, &[], &[], &mut out);
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains("800 Gold"));
        let city = view.cities.iter().find(|c| c.owner == view.me).unwrap();
        let items: Vec<_> = city
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .filter(|o| o.buy_cost.is_some())
            .map(|o| o.item)
            .take(2)
            .collect();
        let his = [Order::BuyItem {
            city: city.id,
            item: items[0],
        }];
        let hard = [Order::BuyItem {
            city: city.id,
            item: items[1],
        }];
        now.his_gold = purchase_gold(&view, &his);
        let mut different = Vec::new();
        war_reserve(&view, &history, &now, &his, &hard, &mut different);
        assert_eq!(different.len(), 1);
        assert!(different[0].text.contains("You bought"));
        assert!(different[0].text.contains("Hard would have bought"));
    }

    #[test]
    fn starting_a_great_work_during_war_asks_but_automation_does_not() {
        let state = with_city(100);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        view.diplomacy.legacy = true;
        let city = view
            .cities
            .iter()
            .find(|c| c.owner == view.me)
            .unwrap()
            .clone();
        let work = ProductionItem::Wonder {
            building: legatum_core::BuildingKind::EiffelTower,
            tile: city.pos,
        };
        let order = Order::SetProduction {
            city: city.id,
            item: work,
        };
        let mut out = Vec::new();
        great_work(&view, std::slice::from_ref(&order), &mut out);
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains("Eiffel Tower"));
        let mut queued = Vec::new();
        great_work(
            &view,
            &[Order::SetProductionQueue {
                city: city.id,
                queue: vec![work],
            }],
            &mut queued,
        );
        assert_eq!(queued.len(), 1);
        view.cities
            .iter_mut()
            .find(|c| c.id == city.id)
            .unwrap()
            .detail
            .as_mut()
            .unwrap()
            .build_automation = Some(legatum_core::CityBuildFocus::Balanced);
        let mut automated = Vec::new();
        great_work(&view, &[order], &mut automated);
        assert!(automated.is_empty());
    }

    #[test]
    fn refusing_a_peace_offer_hard_would_accept_asks_in_player_words() {
        use legatum_core::diplomacy::{PeaceProposal, ProposalId};
        let state = with_city(100);
        let mut view = player_view(&state, legatum_core::PlayerId(0));
        let enemy = view
            .players
            .iter()
            .find(|p| p.team != view.team)
            .unwrap()
            .clone();
        view.diplomacy.legacy = true;
        view.diplomacy.proposals.push(PeaceProposal {
            explanation: None,
            id: ProposalId(7),
            kind: TreatyKind::Peace,
            from: enemy.team,
            to: view.team,
            proposer: enemy.id,
            created_turn: 1,
            respond_by: 3,
            relation_version: 0,
            consents: vec![enemy.team],
            needs: vec![view.team],
            required_teams: vec![enemy.team, view.team],
        });
        let mut trace = crate::OrderTrace::default();
        trace.decisions.push(crate::DecisionOption::new(
            "plan",
            format!("Peace {{ with: TeamId({}) }}", enemy.team.0),
            "executed",
        ));
        let refusal = Order::RespondToProposal {
            proposal: ProposalId(7),
            accept: false,
            explanation: None,
        };
        let mut out = Vec::new();
        war_peace(&view, &[refusal], &[], &trace, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].text,
            format!(
                "Turn {}: {} offered Peace. Why keep fighting?",
                view.turn,
                legatum_core::data::faction_name(enemy.faction)
            )
        );
        assert!(player_words_only(&out[0].text));
    }
}
