//! City and empire yields, with breakdowns. Shared by resolution, views and scoring.

use crate::data::{self, *};
use crate::experiments::FactionRule;
use crate::hex::Hex;
use crate::state::{City, GameState};
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Gold a Celtic sack carries off per population of the sacked city.
pub const SACK_GOLD_PER_POPULATION: i32 = 60;

/// Gold the Celts take when they sack a city: a fixed amount per population, nothing else, so the
/// forecast can show it exactly.
pub fn celts_sack_gold(population: i32) -> i32 {
    population.max(0) * SACK_GOLD_PER_POPULATION
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldSource {
    pub label: String,
    pub yields: Yields,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityYields {
    pub total: Yields,
    pub sources: Vec<YieldSource>,
    pub worked: Vec<Hex>,
    /// Derived from the build and population; never saved as a city setting.
    pub focus: CityFocus,
    pub focus_reason: String,
}

impl CityYields {
    /// Nothing: an after-yields preview a decision view leaves out (`view::ViewDetail`).
    pub fn empty() -> Self {
        Self {
            total: Yields::ZERO,
            sources: Vec::new(),
            worked: Vec::new(),
            focus: CityFocus::Balanced,
            focus_reason: String::new(),
        }
    }

    fn add(&mut self, label: impl AsRef<str> + Into<String>, yields: Yields) {
        // Only kept sources need an owned label.
        if yields != Yields::ZERO
            || label.as_ref().starts_with("India: local network")
            || label.as_ref().starts_with("Corruption:")
        {
            self.total += yields;
            self.sources.push(YieldSource {
                label: label.into(),
                yields,
            });
        }
    }
}

/// Automatic tile-claim priorities, shared by humans, AI and production previews.
/// Scouts retain balanced yields, matching the AI's combat-unit build rule.
pub fn auto_focus(city: &City, item: Option<ProductionItem>) -> CityFocus {
    match item {
        Some(ProductionItem::Unit { unit }) if crate::data::unit_class(unit).is_some() => {
            CityFocus::Production
        }
        Some(item) if data::is_great_work_item(item) => CityFocus::Production,
        Some(ProductionItem::Unit {
            unit: UnitKind::Settler,
        }) if city.population < 2 => CityFocus::Growth,
        _ => CityFocus::Balanced,
    }
}

/// Tile claim score for a city's focus. Claim ties break on hex order in cities::next_claim.
pub fn tile_score(y: Yields, focus: CityFocus) -> i32 {
    match focus {
        CityFocus::Balanced => y.food * 4 + y.production * 3 + y.gold * 2,
        CityFocus::Growth => y.food * 8 + y.production * 2 + y.gold,
        CityFocus::Production => y.food * 3 + y.production * 8 + y.gold,
    }
}

/// Tiles the city works: every tile of its territory within the work radius (population is
/// territory, so there is no citizen assignment). Automatic by design.
pub fn worked_tiles(state: &GameState, city: &City) -> Vec<Hex> {
    worked_tiles_for(state, city)
}

/// Tile yields for this city's owner, including resources it knows (or is about to learn).
pub(crate) fn owner_tile_yields(
    state: &GameState,
    owner: PlayerId,
    tile: &crate::state::Tile,
    research: Option<TechId>,
) -> Yields {
    let player = state.player(owner);
    crate::resources::tile_yields(player.faction, tile, |t| {
        player.techs.contains(&t) || research == Some(t)
    })
}

/// What the tile would yield once its resource is improved: claim and release scoring.
pub(crate) fn owner_tile_potential(
    state: &GameState,
    owner: PlayerId,
    tile: &crate::state::Tile,
) -> Yields {
    let player = state.player(owner);
    crate::resources::tile_potential_yields(player.faction, tile, |t| player.techs.contains(&t))
}

/// Owned tiles within the work radius, in hex order, except tiles barbarians hold. (A city
/// centre always belongs to its own city, so no other centre qualifies.)
fn worked_tiles_for(state: &GameState, city: &City) -> Vec<Hex> {
    worked_tiles_squatted(state, city, &denied_near(state, city))
}

/// `worked_tiles_for` with `denied_near(state, city)` already computed.
fn worked_tiles_squatted(state: &GameState, city: &City, squatted: &[Hex]) -> Vec<Hex> {
    let mut worked: Vec<Hex> = city
        .pos
        .within_iter(CITY_WORK_RADIUS)
        .filter(|h| *h != city.pos)
        .filter(|h| {
            state.map.get(*h).is_some_and(|t| t.city == Some(city.id)) && !squatted.contains(h)
        })
        .collect();
    worked.sort();
    worked
}

/// Hexes in this city's work radius that yield nothing to it: a barbarian camp or a barbarian
/// unit (Civ 6; user, 2026-10-06). One pass over the units instead of one per tile (yield
/// previews call this thousands of times a turn).
pub(crate) fn denied_near(state: &GameState, city: &City) -> Vec<Hex> {
    let mut out: Vec<Hex> = state
        .units
        .values()
        .filter(|u| u.pos.distance(city.pos) <= CITY_WORK_RADIUS && u.owner.is_barbarian())
        .map(|u| u.pos)
        .collect();
    if let Some(barbarians) = &state.barbarians {
        out.extend(
            barbarians
                .camps
                .iter()
                .map(|c| c.pos)
                .filter(|h| h.distance(city.pos) <= CITY_WORK_RADIUS),
        );
    }
    out
}

pub fn city_yields(state: &GameState, city: &City) -> CityYields {
    city_yields_for(state, city, city.producing)
}

/// Income if this build were selected now, without mutating state.
pub fn city_yields_for(state: &GameState, city: &City, item: Option<ProductionItem>) -> CityYields {
    city_yields_with_research(state, city, item, None)
}

pub fn city_yields_with_research(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    research: Option<TechId>,
) -> CityYields {
    city_yields_squatted(state, city, item, research, &denied_near(state, city))
}

/// `city_yields_with_research` with `denied_near(state, city)` already computed. It depends
/// only on the city's position and owner, so previews of one city (other builds, more
/// population, an installed building) can share it.
pub(crate) fn city_yields_squatted(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    research: Option<TechId>,
    squatted: &[Hex],
) -> CityYields {
    yields_in::<true>(state, city, item, research, squatted)
}

/// `city_yields(state, city).total`, without building the labelled sources.
pub fn city_totals(state: &GameState, city: &City) -> Yields {
    city_yield_totals(state, city, city.producing, None)
}

/// Only the totals of `city_yields_with_research`: the same arithmetic without building the
/// labelled sources, the focus reason or the worked-tile list.
pub fn city_yield_totals(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    research: Option<TechId>,
) -> Yields {
    city_totals_squatted(state, city, item, research, &denied_near(state, city))
}

/// `city_yield_totals` with `denied_near(state, city)` already computed.
pub(crate) fn city_totals_squatted(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    research: Option<TechId>,
    squatted: &[Hex],
) -> Yields {
    yields_in::<false>(state, city, item, research, squatted).total
}

/// How a check reports a block: `String` words it, `Blocked` only says it is blocked, so
/// simulations ask whether something is possible without formatting why not
/// (`view::ViewDetail::Decisions`).
pub trait Reason: Sized {
    fn word(text: impl FnOnce() -> String) -> Self;
}

impl Reason for String {
    fn word(text: impl FnOnce() -> String) -> Self {
        text()
    }
}

/// A block without its wording.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blocked;

impl Reason for Blocked {
    fn word(_: impl FnOnce() -> String) -> Self {
        Blocked
    }
}

/// A yield-source label, built only when `FULL` records sources.
pub(crate) trait Label {
    fn text(self) -> String;
}

impl Label for &str {
    fn text(self) -> String {
        self.into()
    }
}

impl Label for String {
    fn text(self) -> String {
        self
    }
}

impl<F: FnOnce() -> String> Label for F {
    fn text(self) -> String {
        self()
    }
}

/// `CityYields` being built: with `FULL`, every source and the focus reason as in
/// `CityYields::add`; without, only the totals (identical, since a skipped source adds zero).
struct YieldsBuilder<const FULL: bool> {
    y: CityYields,
}

impl<const FULL: bool> YieldsBuilder<FULL> {
    fn add(&mut self, label: impl Label, yields: Yields) {
        if FULL {
            self.y.add(label.text(), yields);
        } else {
            self.y.total += yields;
        }
    }

    /// A zero-yield source that only explains something.
    fn note(&mut self, label: impl Label) {
        if FULL {
            self.y.sources.push(YieldSource {
                label: label.text(),
                yields: Yields::ZERO,
            });
        }
    }
}

fn yields_in<const FULL: bool>(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    research: Option<TechId>,
    squatted: &[Hex],
) -> CityYields {
    let base = yields_base::<FULL>(state, city, research, squatted);
    yields_for_item::<FULL>(state, city, item, base)
}

/// `city_yields_squatted`, or with `full` false only what decisions read: totals, worked
/// tiles and focus, without the labelled sources and the focus reason (`ViewDetail`).
pub(crate) fn city_yields_mode(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    research: Option<TechId>,
    squatted: &[Hex],
    full: bool,
) -> CityYields {
    if full {
        yields_in::<true>(state, city, item, research, squatted)
    } else {
        yields_in::<false>(state, city, item, research, squatted)
    }
}

/// Everything in a city's yields that doesn't depend on the build (`yields_base`), with `full`
/// false only what decisions read.
pub(crate) fn city_yields_base_mode(
    state: &GameState,
    city: &City,
    squatted: &[Hex],
    full: bool,
) -> CityYields {
    if full {
        yields_base::<true>(state, city, None, squatted)
    } else {
        yields_base::<false>(state, city, None, squatted)
    }
}

/// `city_yields_squatted(state, city, item, None, squatted)` from
/// `city_yields_base_mode(state, city, squatted, full)`.
pub(crate) fn city_yields_for_item(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    base: &CityYields,
    full: bool,
) -> CityYields {
    if full {
        yields_for_item::<true>(state, city, item, base.clone())
    } else {
        yields_for_item::<false>(state, city, item, base.clone())
    }
}

/// The build-dependent part: focus, its reason and Germany's industry projects.
fn yields_for_item<const FULL: bool>(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    base: CityYields,
) -> CityYields {
    let owner = state.player(city.owner);
    let focus = auto_focus(city, item);
    let focus_reason = match item {
        _ if !FULL => String::new(),
        Some(item) => {
            let name = match item {
                ProductionItem::Unit { unit } => data::unit_name_for(owner.faction, unit),
                ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                    data::building_name_for(owner.faction, building)
                }
                ProductionItem::Project { project } => match project {
                    IndustryProject::Gold => "Gold project",
                    IndustryProject::Science => "Science project",
                    IndustryProject::Food => "Food project",
                    IndustryProject::Grow => "Grow",
                },
                ProductionItem::Improvement { improvement, .. } => {
                    crate::improvements::name(improvement, false)
                }
            };
            if focus == CityFocus::Growth {
                format!(
                    "building {name}; needs population 2 (currently {})",
                    city.population
                )
            } else {
                format!("building {name}")
            }
        }
        None => "no active build".into(),
    };
    let mut y = YieldsBuilder::<FULL> { y: base };
    y.y.focus = focus;
    y.y.focus_reason = focus_reason;
    if city.resistance > 0 {
        return y.y;
    }
    if owner.faction == Faction::Egypt && state.setup.rule_experiments.egypt_nile() {
        nile(state, city, item, &mut y);
    }
    // A Gold or Science project turns part of the city's Production into Gold or Science; after the
    // Nile, so an Egyptian city converts the build progress its surplus Food makes.
    if let Some(ProductionItem::Project { project }) = item
        && project != IndustryProject::Grow
    {
        let pct = state
            .setup
            .rule_experiments
            .project_percent(owner.faction, project);
        let made = y.y.total.production.max(0) * pct / 100;
        // Egypt has no Production: its project converts the build progress its Food makes.
        let from = if owner.faction == Faction::Egypt {
            "build progress"
        } else {
            "Production"
        };
        match project {
            IndustryProject::Gold => y.add(
                || format!("Gold project ({pct}% of {from})"),
                Yields::new(0, 0, made, 0),
            ),
            IndustryProject::Science => y.add(
                || format!("Science project ({pct}% of {from})"),
                Yields::new(0, 0, 0, made),
            ),
            IndustryProject::Food => y.add(
                || format!("Food project ({pct}% of {from})"),
                Yields::new(made, 0, 0, 0),
            ),
            IndustryProject::Grow => {}
        }
    }
    let pct = crate::scoring::decadence_percent(state, city.owner);
    if pct > 0 {
        let cut = decadence_cut(y.y.total, pct);
        let lines = [
            (
                Yields::new(0, -cut.production, 0, 0),
                "of Production: sloth",
            ),
            (Yields::new(0, 0, -cut.gold, 0), "of Gold: luxury"),
            (
                Yields::new(0, 0, 0, -cut.science),
                "of Science: complacency",
            ),
        ];
        for (taken, why) in lines {
            if taken != Yields::ZERO {
                y.add(|| format!("Decadence \u{2212}{pct}% {why}"), taken);
            }
        }
    }
    if crate::eras::all_researched(&owner.techs) {
        let converted = y.y.total.science / 2;
        match owner.science_conversion {
            ScienceConversion::Food => y.add(
                "Science conversion: Food (50%)",
                Yields::new(converted, 0, 0, 0),
            ),
            ScienceConversion::Production => y.add(
                "Science conversion: Production (50%)",
                Yields::new(0, converted, 0, 0),
            ),
            ScienceConversion::Gold => {}
        }
    }
    if owner.tomb_work {
        // The royal tomb (Egypt, `goals::GoalRule::Tomb`): the city's Production and its Food
        // beyond what its people eat are taken, so it neither builds nor grows. A Food
        // deficit is left, so it still starves.
        let surplus = (y.y.total.food - FOOD_PER_POP * city.population).max(0);
        let production = y.y.total.production.max(0);
        if surplus > 0 || production > 0 {
            y.add(
                "The Royal Tomb: this city works on it",
                Yields::new(-surplus, -production, 0, 0),
            );
        }
    }
    y.y
}

/// Whether an Egyptian city on the Nile rule turns its surplus Food into build progress: it does
/// with anything in production except the Grow project, and grows with nothing in production.
pub fn egypt_nile_builds(item: Option<ProductionItem>) -> bool {
    !matches!(
        item,
        None | Some(ProductionItem::Project {
            project: IndustryProject::Grow
        })
    )
}

/// The Nile (designer, 2026-11-21): Egypt's Food is doubled and it has no Production. While the
/// city builds anything but Grow, its surplus Food (above 2 per population) becomes build
/// progress (one per Food, `EGYPT_NILE_BUILD_PERCENT`) instead of growth, so a building city neither grows nor starves; a deficit is left
/// as Food, so it still starves. On Grow, or with nothing in production, the surplus grows it.
fn nile<const FULL: bool>(
    state: &GameState,
    city: &City,
    item: Option<ProductionItem>,
    y: &mut YieldsBuilder<FULL>,
) {
    let rules = state.setup.rule_experiments;
    let food = y.y.total.food;
    if food > 0 {
        let pct = rules.egypt_nile_food_percent();
        let extra = (i64::from(food) * i64::from(pct - 100) / 100) as i32;
        y.add(
            || {
                let whole = pct / 100;
                let tenths = pct % 100 / 10;
                if tenths == 0 {
                    format!("The Nile: Food \u{00d7}{whole}")
                } else {
                    format!("The Nile: Food \u{00d7}{whole}.{tenths}")
                }
            },
            Yields::new(extra, 0, 0, 0),
        );
    }
    let production = y.y.total.production;
    if production != 0 {
        y.add(
            "The Nile: Egypt has no Production",
            Yields::new(0, -production, 0, 0),
        );
    }
    if egypt_nile_builds(item) {
        let surplus = y.y.total.food - FOOD_PER_POP * city.population;
        if surplus > 0 {
            let pct = rules.egypt_nile_build_percent();
            let progress = (i64::from(surplus) * i64::from(pct) / 100) as i32;
            y.add(
                || {
                    if pct == 100 {
                        "The Nile: surplus Food builds instead of growing".to_string()
                    } else {
                        format!("The Nile: surplus Food builds at {pct}% instead of growing")
                    }
                },
                Yields::new(-surplus, progress, 0, 0),
            );
        }
    }
}

/// What decadence takes from a city's Production, Gold and Science at `pct`
/// percent, each rounded to the nearest whole point. Food is not cut here: a city's few spare
/// Food would round to nothing, so decadence slows growth through the growth cost instead
/// (`decadence_growth`), by exactly the same share.
pub fn decadence_cut(total: Yields, pct: i32) -> Yields {
    // An opted-in rate past 100% takes everything, never more.
    let share = |v: i32| {
        let v = v.max(0);
        ((i64::from(v) * i64::from(pct) + 50) / 100).clamp(0, i64::from(v)) as i32
    };
    Yields::new(
        0,
        share(total.production),
        share(total.gold),
        share(total.science),
    )
}

/// The Food a city needs to grow once decadence has made its people have fewer children: the
/// threshold scaled so the same surplus grows the city `pct` percent slower (`100 / (100 - pct)`
/// as much Food per citizen), rounded to the nearest point.
pub fn decadence_growth(threshold: i32, pct: i32) -> i32 {
    if pct <= 0 {
        return threshold;
    }
    let keep = i64::from((100 - pct).max(1));
    ((i64::from(threshold) * 100 + keep / 2) / keep) as i32
}

/// Everything in a city's yields that doesn't depend on the build (`yields_for_item` adds
/// the rest). The focus and its reason are placeholders.
fn yields_base<const FULL: bool>(
    state: &GameState,
    city: &City,
    research: Option<TechId>,
    squatted: &[Hex],
) -> CityYields {
    let owner = state.player(city.owner);
    let knows = |tech| owner.techs.contains(&tech) || research == Some(tech);
    let mut y = YieldsBuilder::<FULL> {
        y: CityYields {
            total: Yields::ZERO,
            sources: Vec::new(),
            worked: worked_tiles_squatted(state, city, squatted),
            focus: CityFocus::Balanced,
            focus_reason: String::new(),
        },
    };

    if city.resistance > 0 {
        y.note(|| format!("Resistance after capture ({} turns left)", city.resistance));
        return y.y;
    }

    for h in city.pos.within_iter(CITY_WORK_RADIUS) {
        if state.map.get(h).is_some_and(|t| t.city == Some(city.id)) && squatted.contains(&h) {
            y.note(|| {
                format!(
                    "Barbarians hold the tile: it yields nothing at {},{}",
                    h.q, h.r
                )
            });
        }
    }

    let center_terrain = state
        .map
        .get(city.pos)
        .map(|t| data::terrain_yields(owner.faction, t.terrain))
        .unwrap_or(Yields::ZERO);
    y.add("City center", CITY_CENTER_YIELDS + center_terrain);

    let mut tiles = Yields::ZERO;
    let mut resources = Yields::ZERO;
    let mut names = Vec::new();
    for h in std::iter::once(&city.pos).chain(&y.y.worked) {
        let Some(tile) = state.map.get(*h) else {
            continue;
        };
        if tile.landmark.is_some() {
            continue;
        }
        let bonus = crate::improvements::tile_resource_yields(tile, *h == city.pos, knows);
        if bonus != Yields::ZERO {
            resources += bonus;
            if !FULL {
                // Names only label the line.
                continue;
            }
            let resource = tile.resource.expect("bonus");
            let upgraded =
                crate::improvements::upgraded(crate::improvements::kind_for(resource), knows);
            names.push(if *h == city.pos {
                crate::resources::name(resource).to_string()
            } else {
                format!(
                    "{} ({})",
                    crate::resources::name(resource),
                    crate::improvements::name(crate::improvements::kind_for(resource), upgraded)
                )
            });
        }
    }
    for h in &y.y.worked {
        tiles += state
            .map
            .get(*h)
            .filter(|t| t.landmark.is_none())
            .map(|t| data::terrain_yields(owner.faction, t.terrain))
            .unwrap_or(Yields::ZERO);
    }
    y.add("Worked tiles", tiles);
    y.add(|| format!("Resources: {}", names.join(", ")), resources);

    // Keep low-production starts moving without accelerating already productive cities.
    // Buildings still add their full output; the final floor survives India modifiers.
    let production_floor = (CITY_MIN_PRODUCTION - y.y.total.production).max(0);
    y.add(
        "Minimum city production",
        Yields::new(0, production_floor, 0, 0),
    );

    y.add("Citizens", Yields::new(0, 0, 0, city.population / 2));
    if state
        .capital_of(city.owner)
        .is_some_and(|capital| capital.id == city.id)
    {
        y.add("Capital", Yields::new(0, 2, 0, CAPITAL_BASE_SCIENCE));
    }

    for b in &city.buildings {
        let mut tier = Some(*b);
        while let Some(building) = tier {
            let yields = state
                .setup
                .rule_experiments
                .building_stats_for(owner.faction, building)
                .yields;
            y.add(data::building_name_for(owner.faction, building), yields);
            // Short of strategic supply, the tier yields nothing until supply returns.
            if let Some(r) = crate::resources::idle_tier(owner, city.id, building) {
                y.add(
                    || {
                        format!(
                            "{} idle: no {}",
                            data::building_name_for(owner.faction, building),
                            crate::resources::name(r)
                        )
                    },
                    Yields::ZERO - yields,
                );
            }
            tier = data::building_predecessor(building);
        }
    }
    if owner.faction == Faction::Germany
        && !state
            .setup
            .rule_experiments
            .rule_off(FactionRule::GermanyIndustryFood)
    {
        let mut names = Vec::new();
        let mut count = 0;
        for &built in &city.buildings {
            let mut tier = Some(built);
            let mut family = Vec::new();
            while let Some(building) = tier {
                family.push(building);
                tier = data::building_predecessor(building);
            }
            for building in family
                .into_iter()
                .rev()
                .filter(|b| data::is_production_building(*b))
            {
                count += 1;
                if FULL {
                    names.push(data::building_name_for(owner.faction, building));
                }
            }
        }
        if count > 0 {
            y.add(
                || format!("Industry: -{count} Food ({})", names.join(", ")),
                Yields::new(-count, 0, 0, 0),
            );
        }
    }
    if knows(TechId::Irrigation) {
        y.add("Irrigation", Yields::new(IRRIGATION_FOOD, 0, 0, 0));
    }
    if knows(TechId::FusionPower) {
        y.add("Fusion Power", Yields::new(0, 5, 0, 0));
    }
    if knows(TechId::Philosophy) {
        y.add("Philosophy", Yields::new(0, 0, 0, 1));
    }
    if knows(TechId::CropRotation) && data::has_building(&city.buildings, BuildingKind::Granary) {
        y.add("Crop Rotation: Granary", Yields::new(1, 0, 0, 0));
    }
    if knows(TechId::Machinery) && data::has_building(&city.buildings, BuildingKind::Workshop) {
        y.add("Machinery: Workshop", Yields::new(0, 1, 0, 0));
    }
    if knows(TechId::ReplaceableParts)
        && data::has_building(&city.buildings, BuildingKind::Factory)
        && crate::resources::idle_tier(owner, city.id, BuildingKind::Factory).is_none()
    {
        y.add("Replaceable Parts: Factory", Yields::new(0, 2, 0, 0));
    }
    if knows(TechId::PrintingPress) && data::has_building(&city.buildings, BuildingKind::Market) {
        y.add("Printing Press: Market", Yields::new(0, 0, 1, 0));
    }
    if data::has_building(&city.buildings, BuildingKind::Library) {
        // Printing Press adds Science to the Library family.
        if knows(TechId::PrintingPress) {
            y.add("Printing Press", Yields::new(0, 0, 0, 1));
        }
    }

    let rules = state.setup.rule_experiments;
    match owner.faction {
        Faction::Switzerland if !rules.rule_off(FactionRule::SwissTolls) => y.add(
            "Switzerland: tolls",
            Yields::new(0, 0, city.population / SWISS_TOLL_POPULATION, 0),
        ),
        Faction::India => {
            if !rules.rule_off(FactionRule::IndiaFood) {
                y.add("India network", Yields::new(INDIA_NETWORK_FOOD, 0, 0, 0));
            }
            let neighbors = state
                .cities_of(city.owner)
                .filter(|other| {
                    other.id != city.id && other.pos.distance(city.pos) <= INDIA_NETWORK_RADIUS
                })
                .count() as i32;
            if neighbors > 0 && !rules.rule_off(FactionRule::IndiaNetwork) {
                let percent = neighbors * INDIA_NETWORK_PERCENT_PER_CITY;
                let bonus = Yields::new(
                    (y.y.total.food * percent + 50) / 100,
                    (y.y.total.production * percent + 50) / 100,
                    0,
                    0,
                );
                y.add(
                    || format!("India: local network (+{}% Food/Production)", percent),
                    bonus,
                );
            }
            if let Some(capital) = state
                .capital_of(city.owner)
                .filter(|_| !rules.rule_off(FactionRule::IndiaCorruption))
            {
                let beyond = (capital.pos.distance(city.pos) - INDIA_CAPITAL_RADIUS).max(0);
                if beyond > 0 {
                    let loss = data::india_distance_loss_percent(beyond);
                    let penalty = y.y.total.percent(100 - loss) - y.y.total;
                    y.add(
                        || {
                            format!(
                                "Corruption: {} tiles from the capital (-{loss}%)",
                                capital.pos.distance(city.pos)
                            )
                        },
                        penalty,
                    );
                }
            }
            y.add(
                "Minimum city production",
                Yields::new(0, (CITY_MIN_PRODUCTION - y.y.total.production).max(0), 0, 0),
            );
        }
        Faction::Mongols
            if !rules.rule_off(FactionRule::MongolsCommerce)
                && data::has_building(&city.buildings, BuildingKind::Market)
                && state
                    .capital_of(city.owner)
                    .is_some_and(|cap| cap.pos.distance(city.pos) >= 8) =>
        {
            let gold = i32::from(
                state
                    .setup
                    .rule_experiments
                    .mongols_commerce_gold
                    .unwrap_or(2),
            );
            y.add("Mongols: long-road commerce", Yields::new(0, 0, gold, 0));
        }
        // The Lavra: the Rus' Library adds Science for each Alliance held (`rus`).
        Faction::Rus if data::has_building(&city.buildings, BuildingKind::Library) => {
            let science = crate::rus::lavra_science(state, city.owner);
            if science > 0 {
                y.add(
                    || {
                        format!(
                            "Lavra: {} Alliances",
                            crate::rus::alliances(state, city.owner)
                        )
                    },
                    Yields::new(0, 0, 0, science),
                );
            }
        }
        _ => {}
    }
    if knows(TechId::RecursiveAi) {
        y.add(
            "Recursive AI (+20% Production)",
            Yields::new(0, y.y.total.production * 20 / 100, 0, 0),
        );
    }
    // Landmarks scale from the city's ordinary output, including faction and building bonuses.
    // Use one pre-wonder base for every landmark so two wonders never compound one another.
    let base = y.y.total;
    for tile in &state.map.tiles {
        if tile.city != Some(city.id) {
            continue;
        }
        if let Some(wonder) = tile.wonder {
            let bonus = wonder.bonus(base);
            y.add(|| format!("{} (+25%)", wonder.name()), bonus);
        }
    }
    // Occupation lock: a frozen city makes no Production (Food, Gold and Science stay).
    if let Some(from) = crate::rules::occupation::city_freeze(state, city) {
        let lost = y.y.total.production;
        y.add(
            || {
                format!(
                    "Occupied: builds nothing until Peace with {}",
                    data::faction_name(state.player(from).faction)
                )
            },
            Yields::new(0, -lost, 0, 0),
        );
    }
    y.y
}

/// Gold paid each turn by this empire for its team's active Alliances, by partner. Every empire
/// on both teams pays until the Alliance ends, except on a turn its members are at War with a
/// team close to winning (`diplomacy::upkeep_waived_by`): the waiver is its own line.
pub fn treaty_upkeep(state: &GameState, p: PlayerId) -> Breakdown {
    let mut b = Breakdown::default();
    let Some(d) = state.diplomacy.as_ref() else {
        return b;
    };
    let team = state.team_of(p);
    for r in d
        .relations
        .iter()
        .filter(|r| r.security.is_some() && (r.a == team || r.b == team))
    {
        let other = if r.a == team { r.b } else { r.a };
        let kind = crate::diplomacy::TreatyKind::Alliance;
        let upkeep = kind.upkeep_for_cities(state.cities_of(p).count());
        let partner = crate::scoring::team_label(state, other);
        b.add(format!("{kind:?} with {partner}"), upkeep);
        if let Some(winner) = crate::diplomacy::upkeep_waived_by(state, team, other) {
            b.add(
                format!(
                    "{kind:?} with {partner} (free while at War with {}, close to winning)",
                    crate::scoring::team_label(state, winner)
                ),
                -upkeep,
            );
        }
    }
    b
}

/// Military upkeep, treaty upkeep and any opt-in city administration cost.
pub fn upkeep(state: &GameState, p: PlayerId) -> i32 {
    upkeep_breakdown(state, p).total
}

/// Round the empire's total paid upkeep up, after faction modifiers and the free allowance.
pub fn upkeep_breakdown(state: &GameState, p: PlayerId) -> Breakdown {
    upkeep_breakdown_with_military(state, p, upkeep_unit_count(state, p) as i32)
}

/// `upkeep_breakdown`, but with an explicit military count: used to preview the upkeep effect of
/// a unit not yet built (e.g. one more than the player's current, forecast-adjusted count).
pub(crate) fn upkeep_breakdown_with_military(
    state: &GameState,
    p: PlayerId,
    military: i32,
) -> Breakdown {
    let paid = (military - FREE_MILITARY_UNITS).max(0);
    let mut b = Breakdown::default();
    b.add(
        "Military units beyond two free (not counting Scouts)",
        paid * UPKEEP_PER_UNIT,
    );
    b.add(
        "Faction",
        data::paid_military_upkeep_rules(
            state.setup.rule_experiments.faction_rules_off,
            state.player(p).faction,
            paid,
        ) - paid * UPKEEP_PER_UNIT,
    );
    b.add(
        "Mercenary Companies",
        crate::company::count(state, p) as i32 * crate::company::UPKEEP,
    );
    b.add("Treaties", treaty_upkeep(state, p).total);
    if state.setup.rule_experiments.city_upkeep {
        b.add("City administration", city_upkeep(state, p));
    }
    b
}

pub fn city_upkeep(state: &GameState, p: PlayerId) -> i32 {
    state
        .setup
        .rule_experiments
        .city_upkeep(state.cities_of(p).count() as i32)
}

/// Food a city needs to grow, less decadence (rounding down once).
pub fn growth_threshold(state: &GameState, city: &City) -> Breakdown {
    let mut b = Breakdown::default();
    b.add(
        if city.population <= 1 {
            "First growth"
        } else {
            "Population growth"
        },
        data::growth_threshold(city.population),
    );
    let pct = crate::scoring::decadence_percent(state, city.owner);
    if pct > 0 {
        b.add(
            format!("Decadence \u{2212}{pct}% growth: smaller families"),
            decadence_growth(b.total, pct) - b.total,
        );
    }
    b
}

/// `growth_threshold(..).total` without the labelled breakdown.
pub fn growth_threshold_total(state: &GameState, city: &City) -> i32 {
    decadence_growth(
        data::growth_threshold(city.population),
        crate::scoring::decadence_percent(state, city.owner),
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerIncome {
    /// Sum of all city yields.
    pub cities: Yields,
    pub upkeep: i32,
    /// Net gold per turn, by source.
    pub gold: Breakdown,
    pub science: Breakdown,
}

/// Interest before decadence: 1 Gold a turn for every 100 Gold held, with no maximum.
pub fn swiss_interest_gross(state: &GameState, p: PlayerId) -> i32 {
    if state.player(p).faction != Faction::Switzerland
        || state
            .setup
            .rule_experiments
            .rule_off(FactionRule::SwissInterest)
    {
        return 0;
    }
    state.player(p).gold.max(0) / SWISS_INTEREST_THRESHOLD
}

/// What decadence takes from this turn's interest (user, 2026-10-17: everything that feels like
/// income is taxed): the empire's rate on the gross interest, to the nearest Gold, like a city's
/// yields. It is a cut from income, not a city yield, so it does not count toward Power.
pub fn swiss_interest_cut(state: &GameState, p: PlayerId) -> i32 {
    decadence_cut(
        Yields::new(0, 0, swiss_interest_gross(state, p), 0),
        crate::scoring::decadence_percent(state, p),
    )
    .gold
}

/// Interest paid at the start of the economy phase, after decadence. It is not a city yield, so it
/// does not count toward Power.
pub fn swiss_interest(state: &GameState, p: PlayerId) -> i32 {
    swiss_interest_gross(state, p) - swiss_interest_cut(state, p)
}

/// Public payout at current treasury and population, before the captured city's population falls.
pub fn swiss_capture_bounty(state: &GameState, city: &City) -> i32 {
    if state.player(city.owner).faction != Faction::Switzerland
        || state
            .setup
            .rule_experiments
            .rule_off(FactionRule::SwissCaptureBounty)
    {
        return 0;
    }
    let total: i32 = state.cities_of(city.owner).map(|c| c.population).sum();
    if total <= 0 {
        return 0;
    }
    let multiplier = if city.is_capital { 2_i64 } else { 1_i64 };
    ((i64::from(state.player(city.owner).gold.max(0)) * i64::from(city.population) * multiplier)
        / i64::from(total))
    .min(i64::from(state.player(city.owner).gold.max(0))) as i32
}

pub fn player_income(state: &GameState, p: crate::types::PlayerId) -> PlayerIncome {
    let mut cities = Yields::ZERO;
    let mut gold = Breakdown::default();
    let mut science = Breakdown::default();
    for c in state.cities_of(p) {
        let y = city_totals(state, c);
        cities += y;
        gold.add(c.name.clone(), y.gold);
        if state.player(p).faction == Faction::India
            && !c.is_capital
            && !state
                .setup
                .rule_experiments
                .rule_off(FactionRule::IndiaRemittance)
        {
            let remitted = if data::has_building(&c.buildings, BuildingKind::Market) {
                y.gold / 2
            } else {
                0
            };
            gold.add(
                format!("{}: regional court keeps Gold", c.name),
                remitted - y.gold,
            );
        }
        science.add(c.name.clone(), y.science);
    }
    let upkeep = upkeep(state, p);
    let administration = city_upkeep(state, p);
    if crate::eras::all_researched(&state.player(p).techs)
        && state.player(p).science_conversion == ScienceConversion::Gold
    {
        gold.add(
            format!(
                "Research after the last technology: {} Science → {} Gold",
                science.total,
                science.total / 2
            ),
            science.total / 2,
        );
    }
    gold.add("Switzerland: interest", swiss_interest_gross(state, p));
    let interest_cut = swiss_interest_cut(state, p);
    if interest_cut > 0 {
        gold.add(
            format!(
                "Decadence \u{2212}{}% of interest: luxury",
                crate::scoring::decadence_percent(state, p)
            ),
            -interest_cut,
        );
    }
    let treaties = treaty_upkeep(state, p);
    gold.add("Unit upkeep", administration + treaties.total - upkeep);
    for (label, amount) in treaties.parts {
        gold.add(format!("Treaty upkeep: {label}"), -amount);
    }
    if state.setup.rule_experiments.city_upkeep {
        gold.add("City administration", -administration);
    }
    PlayerIncome {
        cities,
        upkeep,
        gold,
        science,
    }
}

/// Other living empires that know a tech (what drives its diffusion discount).
pub fn tech_known_by(state: &GameState, p: crate::types::PlayerId, tech: TechId) -> i32 {
    state
        .players
        .iter()
        .filter(|o| o.id != p && o.alive && o.techs.contains(&tech))
        // A faction that skipped it (`data::skipped_techs`) never researched it.
        .filter(|o| !data::tech_skipped(state.setup.rule_experiments, o.faction, tech))
        .count() as i32
}

/// Science cost of a tech for a player: the base cost, less tech diffusion (each other living
/// empire that knows it, up to `TECH_DIFFUSION_MAX`).
pub fn tech_cost(state: &GameState, p: crate::types::PlayerId, tech: TechId) -> Breakdown {
    let base = data::tech_stats(tech).cost;
    // A closed country (Sakoku) learns nothing from abroad: no diffusion discount.
    let closed = crate::goals::is_closed(state, p);
    let others = if closed {
        0
    } else {
        tech_known_by(state, p, tech)
    };
    let pct = (others * TECH_DIFFUSION_PER_EMPIRE).min(TECH_DIFFUSION_MAX);
    // Round the discounted cost, not the discount.
    let discounted = base * (100 - pct) / 100;
    let mut b = Breakdown::default();
    b.add("Listed cost", base);
    b.add(
        if closed {
            "Tech diffusion (the country is closed)".to_string()
        } else {
            format!("Tech diffusion ({others} empires know it)")
        },
        discounted - base,
    );
    if state.setup.rule_experiments.city_research {
        let extra = (state.cities_of(p).count() as i32 - 2).max(0);
        b.add(
            format!("City research (+5% per city beyond two: {extra})"),
            (b.total * extra * 5 + 99) / 100,
        );
    }
    if let Some(percent) = state.setup.rule_experiments.tech_cost_percent {
        let discounted = b.total * i32::from(percent) / 100;
        b.add(
            format!("Experimental tech cost ({percent}%)"),
            discounted - b.total,
        );
    }
    b
}

/// Whether the player may research `tech` now: the strict era gate. It must be unknown and in
/// the player's current era; there are no prerequisites inside an era (`crate::eras`).
pub fn tech_available(state: &GameState, p: crate::types::PlayerId, tech: TechId) -> bool {
    crate::eras::researchable(state.player(p), tech)
}

/// Production cost of an item for this city's owner. Settlers get pricier with each one trained;
/// an empire's first Settler costs `data::FIRST_SETTLER_PERCENT` of its price.
pub fn production_cost(
    state: &GameState,
    owner: crate::types::PlayerId,
    item: ProductionItem,
) -> i32 {
    production_cost_after(state, owner, item, state.player(owner).settlers_trained)
}

/// The first-Settler discount on a Settler's otherwise final price (0 for other items).
fn first_settler_discount(item: ProductionItem, trained: i32, cost: i32) -> i32 {
    if trained == 0
        && item
            == (ProductionItem::Unit {
                unit: UnitKind::Settler,
            })
    {
        cost * data::FIRST_SETTLER_PERCENT / 100 - cost
    } else {
        0
    }
}

/// `production_cost` as if the owner had trained `trained` Settlers.
fn production_cost_after(
    state: &GameState,
    owner: crate::types::PlayerId,
    item: ProductionItem,
    trained: i32,
) -> i32 {
    let full = undiscounted_production_cost(state, owner, item, trained);
    full + first_settler_discount(item, trained, full)
}

fn undiscounted_production_cost(
    state: &GameState,
    owner: crate::types::PlayerId,
    item: ProductionItem,
    trained: i32,
) -> i32 {
    if matches!(item, ProductionItem::Project { .. }) {
        return 0;
    }
    // A Rus' city pays an ally's price for the ally's version (`rus::cost_faction`).
    let faction_cost = state
        .setup
        .rule_experiments
        .base_production_cost(crate::rus::cost_faction(state, owner, item), item)
        + if item
            == (ProductionItem::Unit {
                unit: UnitKind::Settler,
            })
        {
            state.setup.rule_experiments.settler_cost_step() * trained
        } else {
            0
        };
    let full = if sparta_nonmilitary_premium(state, owner, item) {
        (faction_cost * 3 + 1) / 2
    } else {
        faction_cost
    };
    if military_science_discount(state, owner, item) {
        full * 90 / 100
    } else {
        full
    }
}

/// What `buyer` pays in Production-equivalent terms for `version`'s own version of unit `kind`:
/// that faction's price, less the buyer's own Military Science discount (a hired Company,
/// `company::price`).
pub fn version_unit_cost(
    state: &GameState,
    buyer: crate::types::PlayerId,
    version: Faction,
    kind: UnitKind,
) -> i32 {
    let cost = state
        .setup
        .rule_experiments
        .unit_stats_for(version, kind)
        .cost;
    if military_science_discount(state, buyer, ProductionItem::Unit { unit: kind }) {
        cost * 90 / 100
    } else {
        cost
    }
}

fn military_science_discount(
    state: &GameState,
    owner: crate::types::PlayerId,
    item: ProductionItem,
) -> bool {
    state.player(owner).techs.contains(&TechId::MilitaryScience)
        && matches!(item, ProductionItem::Unit { unit } if data::unit_class(unit) != Some(UnitClass::Air))
}

/// The Sparta surcharge applies to civilian production, every Settler included, except
/// war buildings.
pub fn sparta_nonmilitary_premium(
    state: &GameState,
    owner: crate::types::PlayerId,
    item: ProductionItem,
) -> bool {
    state.player(owner).faction == Faction::Sparta
        && !state
            .setup
            .rule_experiments
            .rule_off(FactionRule::SpartaPremium)
        && !matches!(item, ProductionItem::Unit { unit } if data::unit_stats(unit).military)
        && !matches!(item, ProductionItem::Building { building } if data::building_stats(building).war_building)
}

/// A purchased item's quoted cost stays fixed until it completes or the purchase is forfeited.
pub fn city_production_cost(state: &GameState, city: &City, item: ProductionItem) -> i32 {
    city.bought.as_ref().filter(|b| b.item == item).map_or_else(
        || {
            if item
                == (ProductionItem::Building {
                    building: BuildingKind::UpgradeWalls,
                })
            {
                data::upgrade_walls_cost(city.wall_tier())
            } else {
                production_cost(state, city.owner, item)
            }
        },
        |b| b.cost,
    )
}

/// The Gold a purchase costs this empire, in percent of the Production price: 175 for the Swiss
/// (`swiss_cheap_buys`), 200 for everyone else.
pub fn buy_percent(state: &GameState, owner: PlayerId) -> i32 {
    if state.player(owner).faction == Faction::Switzerland
        && !state
            .setup
            .rule_experiments
            .rule_off(FactionRule::SwissCheapBuys)
    {
        SWISS_BUY_PERCENT
    } else {
        BUY_COST_PERCENT
    }
}

/// Items that can never be bought, only produced: the Heir, initial Walls and world wonders.
pub fn is_unbuyable(item: ProductionItem) -> bool {
    item == (ProductionItem::Unit {
        unit: UnitKind::HeirToTheThrone,
    }) || matches!(item, ProductionItem::Building { building } if data::building_family(building) == BuildingKind::Walls)
        || data::is_great_work_item(item)
}

/// Full-price quote for the active item (a separate copy for units).
pub fn buy_cost(state: &GameState, city: &City) -> Option<i32> {
    buy_cost_for(state, city, city.producing?)
}

/// Full faction-adjusted production cost at this turn, never discounted by stored progress.
/// Affordability and temporary blockers are reported separately so the price stays visible.
pub fn buy_cost_for(state: &GameState, city: &City, item: ProductionItem) -> Option<i32> {
    if matches!(item, ProductionItem::Project { .. }) || is_unbuyable(item) {
        return None;
    }
    if !can_produce(state, city, item) {
        return None;
    }
    // `buy_quote`'s production total without its labelled lines: only the first-Settler
    // discount follows the faction-adjusted price.
    let player = state.player(city.owner);
    let trained = player.settlers_trained;
    let full = if item
        == (ProductionItem::Building {
            building: BuildingKind::UpgradeWalls,
        }) {
        city_production_cost(state, city, item)
    } else {
        undiscounted_production_cost(state, city.owner, item, trained)
    };
    let total = full + first_settler_discount(item, trained, full);
    let percent = buy_percent(state, city.owner);
    let price = total * percent / 100;
    debug_assert_eq!(
        Some(price),
        buy_quote(state, city, item).map(|q| q.production.total * q.gold_percent / 100)
    );
    Some(price)
}

/// Engine-owned inputs for a full-price purchase, including pending Settler escalation.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BuyQuote {
    pub production: Breakdown,
    pub gold_percent: i32,
    pub additional_settler_production: i32,
}

pub fn buy_quote(state: &GameState, city: &City, item: ProductionItem) -> Option<BuyQuote> {
    if matches!(item, ProductionItem::Project { .. }) || is_unbuyable(item) {
        return None;
    }
    if !can_produce(state, city, item) {
        return None;
    }
    let player = state.player(city.owner);
    let settler = item
        == (ProductionItem::Unit {
            unit: UnitKind::Settler,
        });
    // Each Settler bought this turn raises the next one (and ends the first-Settler discount).
    let trained = player.settlers_trained;
    let additional = if settler {
        production_cost_after(state, city.owner, item, trained + 1)
            - production_cost_after(state, city.owner, item, trained)
    } else {
        0
    };
    let (base, adjusted) = match item {
        ProductionItem::Building {
            building: BuildingKind::UpgradeWalls,
        } => {
            let cost = data::upgrade_walls_cost(city.wall_tier());
            (cost, cost)
        }
        ProductionItem::Unit { unit } => (
            data::unit_stats(unit).cost,
            state
                .setup
                .rule_experiments
                .unit_stats_for(crate::rus::cost_faction(state, city.owner, item), unit)
                .cost,
        ),
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => (
            data::building_stats(building).cost,
            state
                .setup
                .rule_experiments
                .building_stats_for(player.faction, building)
                .cost,
        ),
        ProductionItem::Improvement { improvement, .. } => {
            let cost = crate::improvements::base_cost(improvement);
            (cost, cost)
        }
        ProductionItem::Project { .. } => return None,
    };
    let mut production = Breakdown::default();
    production.add("Listed cost", base);
    production.add("Faction adjustment", adjusted - base);
    production.add(
        "Balance override",
        if item
            == (ProductionItem::Building {
                building: BuildingKind::UpgradeWalls,
            })
        {
            0
        } else {
            state
                .setup
                .rule_experiments
                .base_production_cost(crate::rus::cost_faction(state, city.owner, item), item)
                - adjusted
        },
    );
    production.add(
        "Previously trained Settlers",
        if settler {
            state.setup.rule_experiments.settler_cost_step() * player.settlers_trained
        } else {
            0
        },
    );
    if sparta_nonmilitary_premium(state, city.owner, item) {
        production.add("Sparta: non-military +50%", (production.total + 1) / 2);
    }
    if military_science_discount(state, city.owner, item) {
        production.add(
            "Military Science: land units -10%",
            production.total * 90 / 100 - production.total,
        );
    }
    let full = undiscounted_production_cost(state, city.owner, item, trained);
    production.add(
        format!("First Settler ({}%)", data::FIRST_SETTLER_PERCENT),
        first_settler_discount(item, trained, full),
    );
    Some(BuyQuote {
        production,
        gold_percent: buy_percent(state, city.owner),
        additional_settler_production: additional,
    })
}

/// Why a purchase is blocked, as data (the AI reads this; `buy_blocker` words it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuyBlock {
    AlreadyBought,
    Resistance,
    SettlerPopulation,
    NotBuyable,
    WallsNeedProduction,
    EgyptBuildingsNeedFood,
    NoSpawnTile,
    /// No strategic supply to spare for the item (`resources::buy_blocker`).
    Resource,
    NotEnoughGold,
    /// Experiment `no_purchase_under_threat`: an enemy military unit stands within
    /// `THREAT_BUY_RADIUS` tiles of the city, so it cannot buy a military unit.
    UnderThreat,
}

/// Experiment `no_purchase_under_threat`: how close (in tiles from the city centre) an enemy
/// military unit at War must stand to stop the city buying military units.
pub const THREAT_BUY_RADIUS: i32 = 2;

/// Whether an at-War military unit stands within `THREAT_BUY_RADIUS` tiles of `city`
/// (barbarians count, except against the Celts: `rules::is_threat`).
pub fn enemy_near_city(state: &GameState, city: &City) -> bool {
    state.units.values().any(|u| {
        u.pos.distance(city.pos) <= THREAT_BUY_RADIUS
            && data::unit_stats(u.kind).military
            && crate::rules::is_threat(state, city.owner, u.owner)
    })
}

/// The first rule that blocks buying `item` in `city` now, if any (see `buy_blocker`).
pub fn buy_block(state: &GameState, city: &City, item: ProductionItem) -> Option<BuyBlock> {
    buy_check::<Blocked>(state, city, item, buy_cost_for(state, city, item)).map(|(block, _)| block)
}

/// Shared by validation, AI-visible quotes and the panel. Purchases happen before movement.
pub fn buy_blocker(state: &GameState, city: &City, item: ProductionItem) -> Option<String> {
    buy_check::<String>(state, city, item, buy_cost_for(state, city, item))
        .map(|(_, reason)| reason)
}

/// The block and its wording together (one rule check).
pub fn buy_block_with_reason(
    state: &GameState,
    city: &City,
    item: ProductionItem,
) -> (Option<BuyBlock>, Option<String>) {
    match buy_check::<String>(state, city, item, buy_cost_for(state, city, item)) {
        Some((block, reason)) => (Some(block), Some(reason)),
        None => (None, None),
    }
}

/// The first block on buying `item`, worded or not (`Reason`), given its
/// `buy_cost_for(state, city, item)`.
pub fn buy_check<R: Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    buy_cost: Option<i32>,
) -> Option<(BuyBlock, R)> {
    buy_check_with_supply(state, city, item, buy_cost, None)
}

pub(crate) fn buy_check_with_supply<R: Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    buy_cost: Option<i32>,
    supply: Option<&crate::resources::ViewSupply>,
) -> Option<(BuyBlock, R)> {
    debug_assert_eq!(buy_cost, buy_cost_for(state, city, item));
    if state.player(city.owner).faction == Faction::Egypt
        && state.setup.rule_experiments.egypt_construction()
        && matches!(item, ProductionItem::Building { .. })
    {
        return Some((
            BuyBlock::EgyptBuildingsNeedFood,
            R::word(|| {
                "Egypt builds ordinary buildings with surplus Food; Gold cannot buy them".into()
            }),
        ));
    }
    if matches!(item, ProductionItem::Building { building } if data::building_family(building) == BuildingKind::Walls)
    {
        return Some((
            BuyBlock::WallsNeedProduction,
            R::word(|| "Walls must be built with Production; they can't be bought.".into()),
        ));
    }
    if item
        == (ProductionItem::Building {
            building: BuildingKind::UpgradeWalls,
        })
        && crate::rules::cities::walls_work_blocked(state, city)
    {
        return Some((
            BuyBlock::UnderThreat,
            R::word(|| crate::rules::cities::WALLS_UNDER_ATTACK.into()),
        ));
    }
    if city.last_purchase_turn == Some(state.turn) {
        return Some((
            BuyBlock::AlreadyBought,
            R::word(|| format!("{} already bought something this turn", city.name)),
        ));
    }
    // The freeze outlasts resistance and is the rule a player can end, so it is the reason given first.
    if let Some(reason) = crate::rules::occupation::city_refusal(state, city) {
        return Some((BuyBlock::Resistance, R::word(|| reason)));
    }
    if city.resistance > 0 {
        return Some((
            BuyBlock::Resistance,
            R::word(|| format!("{} is resisting occupation", city.name)),
        ));
    }
    if state.setup.rule_experiments.no_purchase_under_threat
        && matches!(item, ProductionItem::Unit { unit } if data::unit_stats(unit).military)
        && enemy_near_city(state, city)
    {
        return Some((
            BuyBlock::UnderThreat,
            R::word(|| {
                format!(
                    "{} can't buy military units while an enemy unit is within {THREAT_BUY_RADIUS} tiles",
                    city.name
                )
            }),
        ));
    }
    if item
        == (ProductionItem::Unit {
            unit: UnitKind::Settler,
        })
        && city.population < SETTLER_MIN_POP
    {
        return Some((
            BuyBlock::SettlerPopulation,
            R::word(|| {
                format!(
                    "Settler needs population {SETTLER_MIN_POP} (currently {})",
                    city.population
                )
            }),
        ));
    }
    let Some(cost) = buy_cost else {
        return Some((
            BuyBlock::NotBuyable,
            R::word(|| {
                let faction = state.player(city.owner).faction;
                if faction == Faction::Aztecs
                    && matches!(item, ProductionItem::Unit { unit } if data::is_horse_mounted(unit))
                {
                    return data::faction_unavailable_reason(item, faction).expect("no horses");
                }
                format!("{} cannot be bought", crate::rules::cities::item_name(item))
            }),
        ));
    };
    if let ProductionItem::Unit { unit } = item
        && crate::rules::cities::spawn_tile(state, city.owner, city.pos, unit).is_none()
    {
        return Some((
            BuyBlock::NoSpawnTile,
            R::word(|| {
                if data::is_aircraft(unit) {
                    format!(
                        "The hangar is full: a city bases {} aircraft",
                        data::air_base_cap(&state.player(city.owner).techs)
                    )
                } else {
                    "No free spawn tile in or beside the city; move a blocking unit first".into()
                }
            }),
        ));
    }
    if let Some(reason) = crate::resources::buy_check_with_supply::<R>(state, city, item, supply) {
        return Some((BuyBlock::Resource, reason));
    }
    let gold = state.player(city.owner).gold;
    if gold < cost {
        return Some((
            BuyBlock::NotEnoughGold,
            R::word(|| {
                format!(
                    "Not enough Gold: costs {cost} Gold; you have {gold} (need {} more)",
                    cost - gold
                )
            }),
        ));
    }
    None
}

/// The empire has completed its one Great Work.
pub fn great_work_done(state: &GameState, owner: PlayerId) -> bool {
    state.player(owner).great_work_completed.is_some()
}

/// City requirements independent of research, also used by unlock forecasts.
pub fn building_blocker(
    state: &GameState,
    city: &City,
    building: BuildingKind,
    starting: bool,
) -> Option<String> {
    building_check(state, city, building, starting)
}

/// `building_blocker`, worded or not (`Reason`).
pub fn building_check<R: Reason>(
    state: &GameState,
    city: &City,
    building: BuildingKind,
    starting: bool,
) -> Option<R> {
    building_check_at_era(state, city, building, starting, state.era_of(city.owner))
}

fn building_check_at_era<R: Reason>(
    state: &GameState,
    city: &City,
    building: BuildingKind,
    starting: bool,
    era: u8,
) -> Option<R> {
    if let Some(reason) = data::faction_excluded_building_at_rules(
        state.setup.rule_experiments.faction_rules_off,
        state.player(city.owner).faction,
        building,
        era,
    ) {
        return Some(R::word(|| reason.into()));
    }
    if data::is_great_work(building) {
        if let Some(winner) = state
            .players
            .iter()
            .find(|p| p.great_work_completed == Some(building))
        {
            return Some(R::word(|| {
                format!(
                    "{} was completed by {}; only one can stand in the world",
                    data::building_name(building),
                    data::faction_name(winner.faction)
                )
            }));
        }
        if great_work_done(state, city.owner) {
            return Some(R::word(|| {
                "This empire has already completed its world wonder".into()
            }));
        }
        if let Some(other) = building_limit_holder(state, city.owner, city.id, building) {
            let status = if other.producing.is_some_and(data::is_great_work_item) {
                "under construction"
            } else {
                "queued"
            };
            return Some(R::word(|| {
                format!(
                    "World wonder {status} in {}; only one per empire",
                    other.name
                )
            }));
        }
        if let Some((required, count)) = data::great_work_requirement(building) {
            if !data::has_building(&city.buildings, required) {
                return Some(R::word(|| {
                    format!("Build in a city with {}", data::building_name(required))
                }));
            }
            let owned = state
                .cities_of(city.owner)
                .filter(|c| data::has_building(&c.buildings, required))
                .count();
            if owned < count {
                return Some(R::word(|| {
                    format!(
                        "Requires {count} owned {} buildings (currently {owned})",
                        data::building_name(required)
                    )
                }));
            }
        }
    }
    if data::has_building(&city.buildings, building) {
        let standing = city
            .buildings
            .iter()
            .find(|built| data::building_satisfies(**built, building))
            .expect("has_building found a matching building");
        return Some(R::word(|| {
            format!(
                "The {} already stands in this city",
                data::building_name_for(state.player(city.owner).faction, *standing)
            )
        }));
    }
    if let Some(previous) = data::building_predecessor(building)
        && !city.buildings.contains(&previous)
    {
        return Some(R::word(|| {
            format!(
                "Requires {}",
                data::building_name_for(state.player(city.owner).faction, previous)
            )
        }));
    }
    let required = data::building_min_population(building);
    if (starting
        || (state.player(city.owner).faction == Faction::Egypt
            && state.setup.rule_experiments.egypt_construction()))
        && city.population < required
    {
        return Some(R::word(|| {
            format!(
                "Requires population {required} (currently {})",
                city.population
            )
        }));
    }
    if building == BuildingKind::YamRelayStation
        && let Some(reason) = yam_relay_station_blocker(state, city)
    {
        return Some(R::word(|| reason));
    }
    if building == BuildingKind::ImperialDiwan
        && let Some(reason) = imperial_diwan_blocker(state, city)
    {
        return Some(R::word(|| reason));
    }
    None
}

fn imperial_diwan_blocker(state: &GameState, city: &City) -> Option<String> {
    if let Some(existing) = state
        .player(city.owner)
        .diwan_city
        .and_then(|id| state.cities.get(&id))
        && data::has_building(&existing.buildings, BuildingKind::ImperialDiwan)
    {
        return Some(format!(
            "The Imperial Diwan stands in {}; retake that city instead",
            existing.name
        ));
    }
    if let Some(missing) = state
        .cities_of(city.owner)
        .find(|c| !data::has_building(&c.buildings, BuildingKind::Market))
    {
        return Some(format!("Build a Revenue Office in {} first", missing.name));
    }
    if let Some(other) = state.cities_of(city.owner).find(|c| {
        c.id != city.id
            && (c.producing
                == Some(ProductionItem::Building {
                    building: BuildingKind::ImperialDiwan,
                })
                || c.production_queue.contains(&ProductionItem::Building {
                    building: BuildingKind::ImperialDiwan,
                }))
    }) {
        return Some(format!(
            "The Imperial Diwan is already being built in {}",
            other.name
        ));
    }
    None
}

/// One Yam Relay Station per Mongols empire, rebuilt only once it no longer exists (a Celts capture
/// razed it): while it stands in another city, even one a rival captured, retake that city
/// instead. An active or queued build in another city counts too.
fn yam_relay_station_blocker(state: &GameState, city: &City) -> Option<String> {
    if let Some((tile, holder)) =
        crate::landmarks::site(state, city.owner, BuildingKind::YamRelayStation)
        && holder.is_none()
    {
        return Some(format!(
            "The Yam Relay Station still stands on unclaimed land at {},{}; reclaim its tile instead",
            tile.q, tile.r
        ));
    }
    if let Some(hall) = crate::goals::hall_city(state, city.owner) {
        return Some(if hall.owner == city.owner {
            format!("The Yam Relay Station already stands in {}", hall.name)
        } else {
            format!(
                "The Yam Relay Station stands in {}; retake the city instead",
                hall.name
            )
        });
    }
    let hall = |item: &ProductionItem| {
        matches!(
            item,
            ProductionItem::Building {
                building: BuildingKind::YamRelayStation
            } | ProductionItem::Wonder {
                building: BuildingKind::YamRelayStation,
                ..
            }
        )
    };
    let elsewhere = state.cities_of(city.owner).find(|other| {
        other.id != city.id
            && (other.producing.as_ref().is_some_and(hall)
                || other.production_queue.iter().any(hall))
    })?;
    Some(format!(
        "The Yam Relay Station is already being built in {}",
        elsewhere.name
    ))
}

/// A city holding the last shared empire slot for `building`, if the limit is reached
/// elsewhere. Built copies (when the limit keeps them), active builds and queued builds count.
fn building_limit_holder(
    state: &GameState,
    owner: PlayerId,
    city: CityId,
    building: BuildingKind,
) -> Option<&City> {
    let limit = data::building_empire_limit(building)?;
    let limited = |item: &ProductionItem| matches!(item, ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } if limit.buildings.contains(building));
    let holders: Vec<&City> = state
        .cities_of(owner)
        .filter(|other| other.id != city)
        .filter(|other| {
            (limit.includes_built && other.buildings.iter().any(|b| limit.buildings.contains(b)))
                || other.producing.as_ref().is_some_and(limited)
                || other.production_queue.iter().any(limited)
        })
        .collect();
    (holders.len() >= limit.max)
        .then(|| holders.first().copied())
        .flatten()
}

/// Population gates apply when starting or buying, not after a construction commitment.
pub fn can_produce(state: &GameState, city: &City, item: ProductionItem) -> bool {
    can_produce_phase(state, city, item, true)
}

pub fn can_continue_production(state: &GameState, city: &City, item: ProductionItem) -> bool {
    can_produce_phase(state, city, item, false)
}

fn can_produce_phase(state: &GameState, city: &City, item: ProductionItem, starting: bool) -> bool {
    production_check::<Blocked>(state, city, item, starting).is_none()
}

/// The same specific requirement is shown in choices, validation and skipped-queue notices.
pub fn production_blocker(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    starting: bool,
) -> Option<String> {
    production_check(state, city, item, starting)
}

/// `production_blocker`, worded or not (`Reason`).
pub fn production_check<R: Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    starting: bool,
) -> Option<R> {
    production_check_at_era(state, city, item, starting, state.era_of(city.owner))
}

/// A view can reuse its owner's era across every production option in a city.
pub(crate) fn production_check_at_era<R: Reason>(
    state: &GameState,
    city: &City,
    item: ProductionItem,
    starting: bool,
    era: u8,
) -> Option<R> {
    if matches!(
        item,
        ProductionItem::Building {
            building: BuildingKind::Castle | BuildingKind::StarFort
        }
    ) {
        return Some(R::word(|| "Build Upgrade Walls instead.".into()));
    }
    let player = state.player(city.owner);
    let faction = player.faction;
    let off = state.setup.rule_experiments.faction_rules_off;
    if !data::available_to_player_rules(off, faction, era, item) {
        return Some(R::word(|| {
            data::player_unavailable_reason_rules(off, item, faction, era)
                .expect("unavailable item")
        }));
    }
    if let Some(tech) = data::item_requires_for_rules(state.setup.rule_experiments, item)
        .filter(|t| !state.player(city.owner).techs.contains(t))
    {
        return Some(R::word(|| format!("Requires {}", data::tech_name(tech))));
    }
    match item {
        ProductionItem::Building {
            building: BuildingKind::UpgradeWalls,
        } => {
            let level = city.wall_tier();
            if level == 0 {
                Some(R::word(|| "Build Walls first.".into()))
            } else if city.walls_damage > 0 && (level >= data::MAX_WALL_TIER || level > era) {
                // At the cap, a damaged bar is repaired instead (Repair Walls).
                None
            } else if level >= data::MAX_WALL_TIER {
                Some(R::word(|| {
                    format!(
                        "Walls are at level {}, the top level.",
                        data::wall_level_label(level)
                    )
                }))
            } else if level > era {
                Some(R::word(|| {
                    format!(
                        "Walls are at level {}, the most you can have until you reach the next era.",
                        data::wall_level_label(level)
                    )
                }))
            } else {
                None
            }
        }
        ProductionItem::Building { building } => {
            if crate::landmarks::is_landmark(building) {
                Some(R::word(|| "Choose a tile for this wonder".into()))
            } else {
                building_check_at_era(state, city, building, starting, era)
            }
        }
        ProductionItem::Wonder { building, tile } => {
            building_check_at_era(state, city, building, starting, era)
                .or_else(|| crate::landmarks::check(state, city, building, tile))
        }
        // As in Civ 6, a Settler needs population 2 to be chosen, queued or continued: it
        // consumes one population on completion.
        ProductionItem::Unit {
            unit: UnitKind::Settler,
        } if city.population < SETTLER_MIN_POP => Some(R::word(|| {
            format!(
                "Settler needs population {SETTLER_MIN_POP} (currently {})",
                city.population
            )
        })),
        ProductionItem::Unit {
            unit: UnitKind::HeirToTheThrone,
        } => heir_check(state, city),
        ProductionItem::Improvement { improvement, tile } => {
            crate::improvements::check(state, city, improvement, tile)
        }
        ProductionItem::Unit { unit } if data::pays_upkeep(unit) => levy_check(state, city.owner),
        ProductionItem::Unit { .. } | ProductionItem::Project { .. } => None,
    }
}

/// Land units that pay ordinary military upkeep. Companies have their separate bill.
pub fn upkeep_unit_count(state: &GameState, owner: PlayerId) -> usize {
    state
        .units_of(owner)
        .filter(|u| data::pays_upkeep(u.kind) && !crate::company::is_company(u))
        .count()
}

/// Armed units an empire fields now, the count thin levies cap (`data::levy_cap`). Hired
/// Companies are outside the levy: they have their own contract and upkeep (`company`).
pub fn levy_count(state: &GameState, owner: PlayerId) -> usize {
    let land_units = upkeep_unit_count(state, owner);
    let aircraft = state
        .aircraft
        .iter()
        .filter(|plane| {
            state
                .cities
                .get(&plane.city)
                .is_some_and(|city| city.owner == owner)
        })
        .count();
    land_units + aircraft
}

/// The empire's levy: (armed units, cap), when its faction has a cap.
pub fn levy(state: &GameState, owner: PlayerId) -> Option<(usize, usize)> {
    data::levy_cap_rules(
        state.setup.rule_experiments.faction_rules_off,
        state.player(owner).faction,
        state.cities_of(owner).count(),
    )
    .map(|cap| (levy_count(state, owner), cap))
}

/// Thin levies: no new armed unit while the empire is at its cap. Losing a city never disbands
/// units; it only blocks new ones until the empire is back under the cap.
fn levy_check<R: Reason>(state: &GameState, owner: PlayerId) -> Option<R> {
    let (units, cap) = levy(state, owner).filter(|(units, cap)| units >= cap)?;
    Some(R::word(|| format!("Levies full: {units} of {cap}")))
}

/// The Heir: one at a time. He is retrained only once no Heir of this Byzantium exists anywhere,
/// held by anyone, and no other city of the empire is already building one.
fn heir_check<R: Reason>(state: &GameState, city: &City) -> Option<R> {
    if let Some(heir) = crate::goals::heir(state, city.owner) {
        let own = heir.owner == city.owner;
        return Some(R::word(|| {
            if own {
                "The Heir to the Throne still lives; disband him to train another".to_string()
            } else {
                "The Heir to the Throne lives, held by another; retake him instead".to_string()
            }
        }));
    }
    let heir = ProductionItem::Unit {
        unit: UnitKind::HeirToTheThrone,
    };
    let elsewhere = state.cities_of(city.owner).find(|other| {
        other.id != city.id
            && (other.producing == Some(heir) || other.production_queue.contains(&heir))
    })?;
    Some(R::word(|| {
        format!(
            "A Heir to the Throne is already being trained in {}",
            elsewhere.name
        )
    }))
}

/// Temporary completion gates. Queues retain these items and try later choices.
pub fn placement_blocker(state: &GameState, city: &City, item: ProductionItem) -> Option<String> {
    if let ProductionItem::Unit { unit } = item
        && crate::rules::cities::spawn_tile(state, city.owner, city.pos, unit).is_none()
    {
        return Some(if data::is_aircraft(unit) {
            format!(
                "The hangar is full: a city bases {} aircraft",
                data::air_base_cap(&state.player(city.owner).techs)
            )
        } else {
            "No free spawn tile in or beside the city; move a blocking unit first".into()
        });
    }
    None
}

/// Complete a paid tier in the same family. Earlier tiers are represented by this tier.
pub fn install_building(city: &mut City, building: BuildingKind) {
    city.buildings
        .retain(|b| data::building_family(*b) != data::building_family(building));
    city.buildings.insert(building);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::blank_game;
    use crate::hex::Hex;
    use crate::state::{GameSetup, Map};
    use crate::types::PlayerId;

    fn city(id: u32, owner: PlayerId, pos: Hex, capital: bool) -> City {
        City {
            last_attacked_turn: None,
            id: CityId(id),
            owner,
            founder: owner,
            name: format!("C{id}"),
            pos,
            population: 1,
            food_stored: 0,
            production_stored: 0,
            producing: None,
            bought: None,
            production_queue: Vec::new(),
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_banked: Vec::new(),
            production_queue_notified: Default::default(),
            wonder_lost: None,
            last_purchase_turn: None,
            buildings: Default::default(),
            walls_level: 0,
            walls_damage: 0,
            hp: CITY_HP,
            grown_tiles: Vec::new(),
            tiles_claimed: 0,
            is_capital: capital,
            original_capital: capital,
            greece_captive_paid: Default::default(),
            occupied_from: None,
            founded_turn: 1,
            resistance: 0,
        }
    }

    #[test]
    fn tech_diffusion_rounds_the_discounted_cost() {
        let mut s = blank_game(&GameSetup::all_ai(1), Map::new(20, 20, Terrain::Grassland));
        s.player_mut(PlayerId(1))
            .techs
            .insert(TechId::BronzeWorking);
        // The discounted cost is rounded once: the listed cost times 90 / 100, rounded down.
        let base = data::tech_stats(TechId::BronzeWorking).cost;
        assert_eq!(
            tech_cost(&s, PlayerId(0), TechId::BronzeWorking).total,
            base * 90 / 100
        );
    }

    #[test]
    fn india_penalty_reduces_small_yields() {
        let mut s = blank_game(&GameSetup::all_ai(1), Map::new(30, 10, Terrain::Grassland));
        let p = PlayerId(0);
        s.player_mut(p).faction = Faction::India;
        let far = Hex::from_offset(20, 5);
        s.cities
            .insert(CityId(1), city(1, p, Hex::from_offset(1, 5), true));
        s.cities.insert(CityId(2), city(2, p, far, false));
        // Nineteen hexes from the capital is fifteen beyond the radius: the
        // linear loss is capped at 100%, including the centre's 1 Gold.
        let y = city_yields(&s, &s.cities[&CityId(2)]);
        assert_eq!(y.total.gold, 0);
        // A penalty never turns a yield negative.
        assert!(y.total.food >= 0 && y.total.production >= 0 && y.total.gold >= 0);
        // The capital is unaffected.
        assert_eq!(city_yields(&s, &s.cities[&CityId(1)]).total.gold, 1);
    }

    #[test]
    fn completed_research_converts_half_the_current_science_to_gold() {
        let mut s = blank_game(&GameSetup::all_ai(1), Map::new(20, 20, Terrain::Grassland));
        let p = PlayerId(0);
        s.cities
            .insert(CityId(1), city(1, p, Hex::from_offset(4, 4), true));
        s.player_mut(p).techs.insert(TechId::Chivalry);
        let before = player_income(&s, p);
        s.player_mut(p).techs.extend(TechId::ALL);
        let after = player_income(&s, p);
        let science = after.science.total;
        assert_eq!(after.gold.total - before.gold.total, science / 2);
        assert!(after.gold.parts.iter().any(|(label, amount)| label
            == &format!(
                "Research after the last technology: {science} Science → {} Gold",
                science / 2
            )
            && *amount == science / 2));
    }
    /// Owns the city centre's ring and returns the city's id and one tile it works.
    fn owned_city(s: &mut GameState) -> (CityId, Hex) {
        let p = PlayerId(0);
        let home = Hex::from_offset(5, 5);
        s.cities.insert(CityId(1), city(1, p, home, true));
        for h in home.within_iter(CITY_WORK_RADIUS) {
            let t = s.map.get_mut(h).unwrap();
            t.owner = Some(p);
            t.city = Some(CityId(1));
        }
        let worked = worked_tiles(s, &s.cities[&CityId(1)]);
        (CityId(1), *worked.iter().find(|h| **h != home).unwrap())
    }

    #[test]
    fn barbarian_unit_on_a_worked_tile_removes_its_yields() {
        let mut s = blank_game(&GameSetup::all_ai(1), Map::new(12, 12, Terrain::Grassland));
        let (id, tile) = owned_city(&mut s);
        let before = city_totals(&s, &s.cities[&id]);
        crate::game::spawn_unit(&mut s, PlayerId::BARBARIAN, UnitKind::Spearman, tile);
        assert!(!worked_tiles(&s, &s.cities[&id]).contains(&tile));
        let after = city_totals(&s, &s.cities[&id]);
        assert!(after.food < before.food || after.production < before.production);
        assert!(
            city_yields(&s, &s.cities[&id])
                .sources
                .iter()
                .any(|src| src.label.starts_with("Barbarians hold the tile"))
        );
    }

    #[test]
    fn barbarian_camp_on_a_worked_tile_removes_its_yields() {
        use crate::rules::barbarians::{BarbarianState, Camp};
        let mut s = blank_game(&GameSetup::all_ai(1), Map::new(12, 12, Terrain::Grassland));
        let (id, tile) = owned_city(&mut s);
        let before = city_totals(&s, &s.cities[&id]);
        s.barbarians = Some(BarbarianState {
            camps: vec![Camp {
                id: 1,
                pos: tile,
                founded_turn: 1,
                guard: None,
                scout: None,
                raiders: Vec::new(),
                scout_report: None,
                raid: None,
                next_raid_turn: 0,
                next_spawn_turn: 99,
                rite: None,
            }],
            ..Default::default()
        });
        assert!(!worked_tiles(&s, &s.cities[&id]).contains(&tile));
        let after = city_totals(&s, &s.cities[&id]);
        assert!(after.food < before.food || after.production < before.production);
        // A camp outside the work radius changes nothing.
        s.barbarians.as_mut().unwrap().camps[0].pos = Hex::from_offset(11, 11);
        assert_eq!(city_totals(&s, &s.cities[&id]), before);
    }
}
