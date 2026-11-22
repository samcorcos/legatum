//! Facts derived once per turn from a PlayerView: who is hostile, where threats are, which tiles
//! are frontier, which sites are worth settling, and cheapest-path distance fields.
//!
//! Everything here iterates in a defined order (view vectors sorted by id, tiles by index).

use legatum_core::state::ThreatObservation;
use legatum_core::view::{CityView, PlayerView, UnitView};
use legatum_core::{Hex, PlayerId, Terrain, UnitKind};
use legatum_core::{combat, data};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

/// Enemy military within this distance of an own city threatens it.
pub const THREAT_RADIUS: i32 = 3;

pub const UNREACHABLE: i32 = i32::MAX;

/// Unit movement follows the public faction stats.
pub(crate) fn forecast_moves(view: &PlayerView, owner: PlayerId, kind: UnitKind) -> i32 {
    let faction = view.faction_of(owner);
    view.rule_experiments.unit_moves_for(faction, kind)
}

/// `Ctx::site_score`: the settling score of every tile, computed from the view on first read.
/// Reads and writes behave like the plain vector.
pub struct LazySites<'a> {
    view: &'a PlayerView,
    scores: std::cell::OnceCell<Vec<Option<i32>>>,
    /// The view's settlement facts, shared with settler planning (`settlement_candidates_with`).
    basics: std::cell::OnceCell<legatum_core::settlement::SiteBasics>,
}

impl<'a> LazySites<'a> {
    fn new(view: &'a PlayerView) -> Self {
        Self {
            view,
            scores: std::cell::OnceCell::new(),
            basics: std::cell::OnceCell::new(),
        }
    }

    /// Replace the scores without computing them (fixed benchmark policies).
    pub(crate) fn set(&mut self, scores: Vec<Option<i32>>) {
        self.scores = std::cell::OnceCell::from(scores);
    }

    /// This view's settlement facts (`SiteBasics`), built on first use.
    pub fn basics(&self) -> &legatum_core::settlement::SiteBasics {
        self.basics
            .get_or_init(|| legatum_core::settlement::SiteBasics::new(self.view))
    }

    fn compute(&self) -> Vec<Option<i32>> {
        let basics = self.basics();
        (0..self.view.tiles.len())
            .map(|i| basics.base_score(self.view, i))
            .collect()
    }
}

impl std::ops::Deref for LazySites<'_> {
    type Target = Vec<Option<i32>>;

    fn deref(&self) -> &Self::Target {
        self.scores.get_or_init(|| self.compute())
    }
}

impl std::ops::DerefMut for LazySites<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.scores.get().is_none() {
            let scores = self.compute();
            self.scores = std::cell::OnceCell::from(scores);
        }
        self.scores.get_mut().expect("initialized")
    }
}

pub struct Ctx<'a> {
    pub view: &'a PlayerView,
    pub me: PlayerId,
    /// Own units, sorted by id.
    pub own_units: Vec<&'a UnitView>,
    /// Own cities (with detail), sorted by id.
    pub own_cities: Vec<&'a CityView>,
    /// Visible units of other teams, sorted by id.
    pub enemy_units: Vec<&'a UnitView>,
    /// The part of `enemy_units` that threatens us (`PlayerView::is_threat`): all of it, except
    /// that a Celtic viewer drops barbarians, who never attack the Celts. `enemy_units` stays
    /// the list of attack targets; anything that reasons about danger, escorts, garrisons or
    /// retreat reads this one.
    pub threat_units: Vec<&'a UnitView>,
    /// Visible military of non-allied empires at Peace. Settlement prices the possibility
    /// of a declaration; wartime danger and every other threat consumer stay unchanged.
    pub peace_armies: Vec<&'a UnitView>,
    /// Recent unseen sightings, deduplicated against live units. Planning uncertainty only:
    /// these are not targets, current positions, health or combat forecasts.
    pub remembered_threats: Vec<&'a ThreatObservation>,
    /// Known cities of other teams, sorted by id.
    pub enemy_cities: Vec<&'a CityView>,
    /// Enemy military power within THREAT_RADIUS of each own city (same order as own_cities).
    pub threat: Vec<i32>,
    /// The barbarian part of `threat`. Barbarians never assault cities (a public rule), so a
    /// raid moves the army but doesn't change what a city builds.
    pub barbarian_threat: Vec<i32>,
    /// Tiles currently holding a friendly (own or teammate) military unit.
    pub friendly_military_at: Vec<bool>,
    /// Tiles with an enemy unit or city (index-aligned with the map).
    pub hostile_at: Vec<bool>,
    pub neutral_at: Vec<bool>,
    /// Any of `friendly_military_at`, `hostile_at` or `neutral_at`.
    pub occupied: Vec<bool>,
    /// Tiles within 2 of a visible enemy military unit or a known, live hostile city.
    pub danger: Vec<bool>,
    /// Conservative settler avoidance from visible military and recent observations.
    /// Separate from live danger: memories must not become targets or army-strength facts.
    pub settler_danger: Vec<bool>,
    /// Explored passable tiles next to unexplored ones, plus known ruins. Sorted by hex index.
    pub frontier: Vec<Hex>,
    /// Settling score per tile index, None where a city may not (or should not) be founded.
    /// Computed on first use: many planning passes (prospective planners) never read it.
    pub site_score: LazySites<'a>,
    /// Connected components of explored, passable land. Unknown shortcuts are not proof of
    /// reachability: the engine validates a move against the real terrain, even in fog.
    pub(crate) land_region: Vec<Option<usize>>,
    /// `move_cost` per tile (the passability behind `land_region`).
    move_costs: Vec<Option<i32>>,
    /// `view.passage_allowed(me, Spearman, owner)` for every owner (the view is fixed for this
    /// context): where our armed units may stand.
    entry: legatum_core::view::EntryTable,
}

impl<'a> Ctx<'a> {
    pub fn new(view: &'a PlayerView) -> Self {
        Self::with_city_fire(view, true)
    }

    /// Versioned Proxy policies remain a fixed benchmark opponent.
    pub(crate) fn for_proxy(view: &'a PlayerView) -> Self {
        Self::with_city_fire(view, false)
    }

    fn with_city_fire(view: &'a PlayerView, city_fire: bool) -> Self {
        Self::build(view, city_fire, None)
    }

    /// A context for a prospective view of `parent`'s (the same tiles, only diplomatic
    /// relations changed): the frontier is the parent's, and so are the land regions
    /// wherever passability came out the same.
    pub(crate) fn prospective(view: &'a PlayerView, parent: &Ctx<'_>) -> Self {
        Self::build(view, true, Some(parent))
    }

    fn build(view: &'a PlayerView, city_fire: bool, parent: Option<&Ctx<'_>>) -> Self {
        let me = view.me;
        let mut hostile = vec![true; 256];
        for p in &view.players {
            hostile[p.id.0 as usize] = view.at_war(me, p.id);
        }
        hostile[me.0 as usize] = false;

        let mut own_units: Vec<&UnitView> = view.units.iter().filter(|u| u.owner == me).collect();
        own_units.sort_by_key(|u| u.id);
        let mut own_cities: Vec<&CityView> = view
            .cities
            .iter()
            .filter(|c| c.owner == me && c.detail.is_some())
            .collect();
        own_cities.sort_by_key(|c| c.id);
        let mut enemy_units: Vec<&UnitView> = view
            .units
            .iter()
            .filter(|u| hostile[u.owner.0 as usize])
            .collect();
        enemy_units.sort_by_key(|u| u.id);
        // Everything in `enemy_units` is at war with us; barbarians threaten nothing of the Celts.
        let spared = view.barbarians_spare_me();
        let threat_units: Vec<&UnitView> = enemy_units
            .iter()
            .copied()
            .filter(|u| !(spared && u.owner.is_barbarian()))
            .collect();
        let mut peace_armies: Vec<&UnitView> = view
            .units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && !u.owner.is_barbarian()
                    && !view.same_team(me, u.owner)
                    && !view.allied(me, u.owner)
                    && !view.at_war(me, u.owner)
            })
            .collect();
        peace_armies.sort_by_key(|u| u.id);
        let mut remembered = BTreeMap::<_, &ThreatObservation>::new();
        for o in &view.observed_threats {
            if hostile[o.owner.0 as usize]
                && !(spared && o.owner.is_barbarian())
                && data::unit_stats(o.kind).military
                && o.seen_turn <= view.turn
                && view.turn - o.seen_turn < legatum_core::visibility::THREAT_MEMORY_TURNS
                && view.tile(o.pos).is_some_and(|t| !t.visible)
                && !view.units.iter().any(|u| u.id == o.unit)
                && remembered
                    .get(&o.unit)
                    .is_none_or(|old| old.seen_turn < o.seen_turn)
            {
                remembered.insert(o.unit, o);
            }
        }
        let mut enemy_cities: Vec<&CityView> = view
            .cities
            .iter()
            .filter(|c| hostile[c.owner.0 as usize])
            .collect();
        enemy_cities.sort_by_key(|c| c.id);

        let n = view.tiles.len();
        let mut friendly_military_at = vec![false; n];
        let mut hostile_at = vec![false; n];
        let mut danger = vec![false; n];
        let mut neutral_at = vec![false; n];
        for u in &view.units {
            let Some(i) = view.index(u.pos) else { continue };
            if hostile[u.owner.0 as usize] {
                hostile_at[i] = true;
                // Barbarians never attack the Celts: their camps' units are targets, not danger.
                if u.military
                    && u.kind != UnitKind::Scout
                    && !(u.owner.is_barbarian() && view.barbarians_spare_me())
                {
                    for h in u.pos.within(2) {
                        if let Some(j) = view.index(h) {
                            danger[j] = true;
                        }
                    }
                }
            } else if !view.same_team(me, u.owner) && !view.allied(me, u.owner) {
                neutral_at[i] = true;
            } else if u.military {
                friendly_military_at[i] = true;
            }
        }
        for c in &enemy_cities {
            if let Some(i) = view.index(c.pos) {
                hostile_at[i] = true;
            }
            if city_fire && legatum_core::city_fires(c) {
                for h in c.pos.within(2) {
                    if let Some(i) = view.index(h) {
                        danger[i] = true;
                    }
                }
            }
        }

        let threat_by = |barbarian_only: bool| -> Vec<i32> {
            own_cities
                .iter()
                .map(|c| {
                    enemy_units
                        .iter()
                        // Scouts are too weak to count; the standing garrison handles them.
                        .filter(|u| {
                            u.military
                                && u.kind != UnitKind::Scout
                                && u.pos.distance(c.pos) <= THREAT_RADIUS
                                && !(u.owner.is_barbarian() && view.barbarians_spare_me())
                                && (!barbarian_only || u.owner.is_barbarian())
                        })
                        .map(|u| threat_power(view, u))
                        .sum()
                })
                .collect()
        };
        let threat = threat_by(false);
        let barbarian_threat = threat_by(true);

        let mut ctx = Ctx {
            view,
            me,
            own_units,
            own_cities,
            enemy_units,
            threat_units,
            peace_armies,
            remembered_threats: remembered.into_values().collect(),
            enemy_cities,
            threat,
            barbarian_threat,
            occupied: friendly_military_at
                .iter()
                .zip(&hostile_at)
                .zip(&neutral_at)
                .map(|((f, h), n)| *f || *h || *n)
                .collect(),
            friendly_military_at,
            hostile_at,
            neutral_at,
            danger,
            settler_danger: vec![false; n],
            frontier: Vec::new(),
            site_score: LazySites::new(view),
            land_region: vec![None; n],
            move_costs: Vec::new(),
            entry: view.entry_table(me, legatum_core::UnitKind::Spearman),
        };
        ctx.compute_land_regions(parent);
        // A sighting is an uncertainty to route around, not an omniscient moving threat.
        let threats = view
            .observed_threats
            .iter()
            .filter(|o| {
                o.seen_turn <= view.turn
                    && view.turn - o.seen_turn < legatum_core::visibility::THREAT_MEMORY_TURNS
            })
            .map(|o| (o.owner, o.kind, o.pos, None))
            .chain(
                ctx.threat_units
                    .iter()
                    .filter(|u| u.military)
                    .map(|u| (u.owner, u.kind, u.pos, Some(u.max_moves))),
            );
        for (owner, kind, pos, visible_moves) in threats {
            if !hostile[owner.0 as usize] || (spared && owner.is_barbarian()) {
                continue;
            }
            let stats = view
                .rule_experiments
                .unit_stats_for(view.faction_of(owner), kind);
            let ranged_reach = if stats.ranged > 0 { stats.range } else { 0 };
            let moves = visible_moves.unwrap_or_else(|| forecast_moves(view, owner, kind));
            for h in pos.within(moves.max(ranged_reach)) {
                if let Some(i) = view.index(h)
                    && (h.distance(pos) <= ranged_reach
                        || (h.distance(pos) <= moves && ctx.reachable(pos, h)))
                {
                    ctx.settler_danger[i] = true;
                }
            }
        }
        ctx.frontier = match parent {
            Some(p) => p.frontier.clone(),
            None => ctx.compute_frontier(),
        };
        ctx
    }

    pub fn hex_at(&self, i: usize) -> Hex {
        let w = self.view.width as usize;
        Hex::from_offset((i % w) as i32, (i / w) as i32)
    }

    /// Movement cost on known land. Scouts reveal more land by visiting the known frontier.
    pub fn move_cost(&self, h: Hex) -> Option<i32> {
        self.view.index(h).and_then(|i| self.move_costs[i])
    }

    pub fn move_cost_light(&self, h: Hex) -> Option<i32> {
        self.move_cost(h)
    }

    fn compute_land_regions(&mut self, parent: Option<&Ctx<'_>>) {
        // Passability once per tile; the flood then reads the table (same regions: each is
        // labelled by its first tile in index order).
        let n = self.view.tiles.len();
        // Known land we may enter: terrain cost on explored tiles whose owner admits us.
        let move_costs: Vec<Option<i32>> = self
            .view
            .tiles
            .iter()
            .map(|t| {
                if !self.entry.can_enter(self.view, t.owner) {
                    return None;
                }
                Some(t).filter(|t| t.explored)?.move_cost()
            })
            .collect();
        let passable: Vec<bool> = move_costs.iter().map(Option::is_some).collect();
        // Same costs as the parent's: the same components, so the same labels.
        if let Some(p) = parent
            && p.move_costs == move_costs
        {
            self.land_region = p.land_region.clone();
            self.move_costs = move_costs;
            return;
        }
        let table = neighbor_table(self.view.width, self.view.height);
        let mut stack = Vec::new();
        for start in 0..n {
            if self.land_region[start].is_some() || !passable[start] {
                continue;
            }
            self.land_region[start] = Some(start);
            stack.push(start);
            while let Some(i) = stack.pop() {
                for &j in &table[i] {
                    let j = j as usize;
                    if j == OFF_MAP {
                        continue;
                    }
                    if self.land_region[j].is_none() && passable[j] {
                        self.land_region[j] = Some(start);
                        stack.push(j);
                    }
                }
            }
        }
        self.move_costs = move_costs;
    }

    /// The land region of `h` (None off the map or off known passable land).
    pub fn region(&self, h: Hex) -> Option<usize> {
        self.view.index(h).and_then(|i| self.land_region[i])
    }

    pub fn reachable(&self, from: Hex, to: Hex) -> bool {
        let region = |h| self.view.index(h).and_then(|i| self.land_region[i]);
        region(from).is_some_and(|r| region(to) == Some(r))
    }

    pub fn in_danger(&self, h: Hex) -> bool {
        self.view.index(h).is_some_and(|i| self.danger[i])
    }

    /// Distance from `h` to the nearest own city, or None with no cities.
    pub fn nearest_own_city(&self, h: Hex) -> Option<&'a CityView> {
        self.own_cities
            .iter()
            .copied()
            .min_by_key(|c| (c.pos.distance(h), c.id))
    }

    pub fn capital(&self) -> Option<&'a CityView> {
        self.own_cities
            .iter()
            .copied()
            .find(|c| c.is_capital)
            .or_else(|| self.own_cities.first().copied())
    }

    fn compute_frontier(&self) -> Vec<Hex> {
        let v = self.view;
        let table = neighbor_table(v.width, v.height);
        let mut out = Vec::new();
        for (i, t) in v.tiles.iter().enumerate() {
            if !t.explored || t.terrain.and_then(|t| t.move_cost()).is_none() {
                continue;
            }
            let edge = table[i]
                .iter()
                .any(|&j| j as usize != OFF_MAP && !v.tiles[j as usize].explored);
            if edge || t.ruin {
                out.push(self.hex_at(i));
            }
        }
        out
    }

    /// How many unexplored tiles lie within 2 of `h` (what a scout would reveal there).
    pub fn unexplored_near(&self, h: Hex) -> i32 {
        h.within(2)
            .into_iter()
            .filter(|n| self.view.tile(*n).is_some_and(|t| !t.explored))
            .count() as i32
    }

    /// Cheapest movement cost from `from` to every known land tile (Dijkstra).
    /// `avoid_hostile` treats enemy-held tiles as impassable and adds a penalty near danger.
    pub fn distance_field(&self, from: Hex, avoid_hostile: bool) -> Vec<i32> {
        self.distance_field_avoiding(from, avoid_hostile, None)
    }

    /// Civilians need a route that avoids hazards, not a score penalty that still sends
    /// the engine's ordinary shortest path through the same rejected approach.
    pub fn settler_distance_field(&self, from: Hex) -> Vec<i32> {
        self.distance_field_avoiding(from, true, Some(&self.settler_danger))
    }

    pub(crate) fn distance_field_avoiding(
        &self,
        from: Hex,
        avoid_hostile: bool,
        blocked: Option<&[bool]>,
    ) -> Vec<i32> {
        let n = self.view.tiles.len();
        let mut dist = vec![UNREACHABLE; n];
        let Some(start) = self.view.index(from) else {
            return dist;
        };
        dist[start] = 0;
        let table = neighbor_table(self.view.width, self.view.height);
        let mut open = BinaryHeap::new();
        open.push(Reverse((0i32, start)));
        while let Some(Reverse((d, i))) = open.pop() {
            if d > dist[i] {
                continue;
            }
            for &j in &table[i] {
                let j = j as usize;
                if j == OFF_MAP {
                    continue;
                }
                if blocked.is_some_and(|tiles| tiles[j]) {
                    continue;
                }
                let Some(mut step) = self.move_costs[j] else {
                    continue;
                };
                if self.neutral_at[j] {
                    continue;
                }
                if avoid_hostile {
                    if self.hostile_at[j] {
                        continue;
                    }
                    if self.danger[j] {
                        step += 4;
                    }
                }
                let nd = d + step;
                if nd < dist[j] {
                    dist[j] = nd;
                    open.push(Reverse((nd, j)));
                }
            }
        }
        dist
    }
}

/// Strength scaled by health: a unit's weight in local strength comparisons.
fn scaled(strength: i32, hp: i32, max_hp: i32) -> i32 {
    strength.max(0) * hp.max(0) / max_hp.max(1)
}

/// How dangerous an enemy unit is to things near it (its best attack, scaled by HP).
pub fn threat_power(view: &PlayerView, u: &UnitView) -> i32 {
    let melee = crate::supply::attack_total(view.faction_of(u.owner), u, u.hp);
    let ranged = if u.ranged > 0 {
        crate::supply::ranged_total(view.faction_of(u.owner), u, u.hp)
    } else {
        0
    };
    scaled(melee.max(ranged), u.hp, data::UNIT_HP)
}

/// A unit's melee attack weight.
pub fn attack_power(view: &PlayerView, u: &UnitView) -> i32 {
    scaled(
        crate::supply::attack_total(view.faction_of(u.owner), u, u.hp),
        u.hp,
        data::UNIT_HP,
    )
}

/// A unit's weight when attacking: its ranged strength for ranged units, else melee, and the
/// stronger of the two for a unit with both (the Keshig charges or shoots).
pub fn offense_power(view: &PlayerView, u: &UnitView) -> i32 {
    if u.ranged > 0 {
        let ranged = scaled(
            crate::supply::ranged_total(view.faction_of(u.owner), u, u.hp),
            u.hp,
            data::UNIT_HP,
        );
        if crate::composition::dual(u.kind, u.ranged) {
            ranged.max(attack_power(view, u))
        } else {
            ranged
        }
    } else {
        attack_power(view, u)
    }
}

/// A unit's weight when defending its tile (terrain and fortification included).
pub fn defense_power(view: &PlayerView, u: &UnitView) -> i32 {
    let terrain = view
        .tile(u.pos)
        .and_then(|t| t.terrain)
        .unwrap_or(legatum_core::Terrain::Grassland);
    scaled(
        crate::supply::defense_total(view.faction_of(u.owner), u, u.hp, terrain, u.fortify),
        u.hp,
        data::UNIT_HP,
    )
}

/// Apply one combat stage's `damage` aimed at `target` to a projected view, the way core
/// resolves it (`combat::city_tile_defender`): city HP absorbs the stage's pooled damage first,
/// and excess reaches the garrison at the snapshot defense ratio.
pub fn damage_tile_defender(view: &mut PlayerView, target: Hex, damage: i32) {
    let me = view.me;
    let unit = view
        .units
        .iter()
        .filter(|u| u.pos == target && u.military && view.at_war(me, u.owner))
        .min_by_key(|u| u.id)
        .map(|u| u.id);
    let city = view
        .cities
        .iter()
        .position(|c| c.pos == target && view.at_war(me, c.owner));
    match city {
        Some(i)
            if legatum_core::city_attack_target(view, target)
                == Some(combat::CityTileDefender::City) =>
        {
            let city = &view.cities[i];
            let excess = damage - city.hp.max(0);
            let city_defense = combat::city_defense_for_rules(
                view.rule_experiments,
                view.faction_of(city.owner),
                city.population,
                city.walls,
                city.fort_tier(),
                city.hp,
                view.era_of(city.owner),
            )
            .total;
            let garrison_damage = unit
                .and_then(|id| view.units.iter().find(|u| u.id == id))
                .map(|u| {
                    let terrain = view
                        .tile(target)
                        .and_then(|t| t.terrain)
                        .unwrap_or(Terrain::Plains);
                    let defense = crate::supply::defense_total(
                        view.faction_of(u.owner),
                        u,
                        u.hp,
                        terrain,
                        u.fortify,
                    );
                    // The hit's direct damage on the garrison is estimated by the defence ratio.
                    combat::carried_share(damage * city_defense / defense.max(1), excess, damage)
                })
                .unwrap_or(0);
            view.cities[i].hp = (view.cities[i].hp - damage).max(0);
            if let Some(id) = unit {
                if let Some(u) = view.units.iter_mut().find(|u| u.id == id) {
                    u.hp -= garrison_damage;
                }
                view.units.retain(|u| u.id != id || u.hp > 0);
            }
        }
        _ => {
            if let Some(id) = unit {
                let u = view.units.iter_mut().find(|u| u.id == id).expect("unit");
                u.hp -= damage;
                view.units.retain(|u| u.id != id || u.hp > 0);
            }
        }
    }
}

/// A city's own defensive weight (walls included), scaled by its HP on the unit HP scale: a
/// full-health city (200 HP) weighs twice its strength, since it takes twice the damage to break.
pub fn city_power(view: &PlayerView, c: &CityView) -> i32 {
    scaled(
        combat::city_defense_for_rules(
            view.rule_experiments,
            view.faction_of(c.owner),
            c.population,
            c.walls,
            c.fort_tier(),
            c.hp,
            view.era_of(c.owner),
        )
        .total,
        c.hp,
        data::UNIT_HP,
    )
}

/// A set of hexes with O(1) membership on the map (a bitmap beside the ordered set, which
/// keeps any iteration order identical to `BTreeSet<Hex>`).
#[derive(Clone, Debug, Default)]
pub struct HexSet {
    set: std::collections::BTreeSet<Hex>,
    bits: Vec<bool>,
    width: i32,
    height: i32,
}

impl HexSet {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            set: Default::default(),
            bits: vec![false; (width.max(0) * height.max(0)) as usize],
            width,
            height,
        }
    }

    fn index(&self, h: Hex) -> Option<usize> {
        let (c, r) = h.to_offset();
        (c >= 0 && r >= 0 && c < self.width && r < self.height)
            .then(|| (r * self.width + c) as usize)
    }

    pub fn insert(&mut self, h: Hex) -> bool {
        if let Some(i) = self.index(h) {
            self.bits[i] = true;
        }
        self.set.insert(h)
    }

    pub fn contains(&self, h: &Hex) -> bool {
        match self.index(*h) {
            Some(i) => self.bits[i],
            None => self.set.contains(h),
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}

/// `neighbor_table` entry for a neighbour off the map.
const OFF_MAP: usize = u32::MAX as usize;

/// Map indices of every tile's six neighbours (`Hex::neighbors` order, `OFF_MAP` where off
/// the map), built once per map size and thread.
type NeighborTable = std::rc::Rc<Vec<[u32; 6]>>;

fn neighbor_table(width: i32, height: i32) -> NeighborTable {
    thread_local! {
        static TABLE: std::cell::RefCell<Option<((i32, i32), NeighborTable)>> =
            const { std::cell::RefCell::new(None) };
    }
    TABLE.with(|cell| {
        if let Some((dims, t)) = cell.borrow().as_ref()
            && *dims == (width, height)
        {
            return t.clone();
        }
        let index = |h: Hex| -> u32 {
            let (c, r) = h.to_offset();
            if c < 0 || r < 0 || c >= width || r >= height {
                u32::MAX
            } else {
                (r * width + c) as u32
            }
        };
        let n = (width.max(0) * height.max(0)) as usize;
        let t: Vec<[u32; 6]> = (0..n)
            .map(|i| {
                let h = Hex::from_offset((i as i32) % width, (i as i32) / width);
                h.neighbors().map(index)
            })
            .collect();
        let t = std::rc::Rc::new(t);
        *cell.borrow_mut() = Some(((width, height), t.clone()));
        t
    })
}

#[cfg(test)]
mod barbarian_peace_tests {
    //! Barbarians never attack the Celts (`PlayerView::is_threat`): they stay attack targets
    //! (`enemy_units`) but are no threat, danger or Settler risk for a Celtic viewer.
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::{Faction, UnitKind};

    #[test]
    fn a_barbarian_is_a_target_but_no_threat_to_the_celts() {
        for (faction, threatens) in [(Faction::Celts, false), (Faction::Switzerland, true)] {
            let v = ViewBuilder::new(14, 10)
                .faction(faction)
                .city_with_detail(1, ME, off(4, 5), 3)
                .unit(2, PlayerId::BARBARIAN, UnitKind::Spearman, off(6, 5))
                .build();
            let ctx = Ctx::new(&v);
            assert_eq!(ctx.enemy_units.len(), 1, "{faction:?}: still a target");
            assert_eq!(
                ctx.threat_units.len(),
                usize::from(threatens),
                "{faction:?}"
            );
            assert_eq!(ctx.threat[0] > 0, threatens, "{faction:?}: city threat");
            let at = v.index(off(6, 5)).unwrap();
            assert_eq!(
                ctx.settler_danger[at], threatens,
                "{faction:?}: Settler danger"
            );
        }
    }
}
