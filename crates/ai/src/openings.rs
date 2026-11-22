//! Opt-in, stateful opening policies. Memory is derived only from successive PlayerViews;
//! neither tuning nor observation can read GameState. Sim tapes record the resulting orders.
use crate::{Brain, Knobs, ScriptedBrain, Strategy, analysis::offense_power, scripted::Planner};
use legatum_core::view::CityView;
use legatum_core::{
    BuildingKind, CityId, Order, PlayerView, ProductionItem, TeamId, TechId, UnitId, UnitKind,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod closing_tests;
#[cfg(test)]
mod identity_tests;
mod primitives;
pub(crate) use primitives::PrimitivePlan;
pub use primitives::{CoalitionMarch, PrimitiveMemory, PrimitiveParams};

pub const OPENING_SCHEMA: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Opening {
    #[default]
    Default,
    Rush,
    GreedyExpand,
    Snowball,
    Economic,
}

impl Opening {
    pub const ALL: [Self; 5] = [
        Self::Default,
        Self::Rush,
        Self::GreedyExpand,
        Self::Snowball,
        Self::Economic,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Rush => "Rush",
            Self::GreedyExpand => "GreedyExpand",
            Self::Snowball => "Snowball",
            Self::Economic => "Economic",
        }
    }
}
impl std::str::FromStr for Opening {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|o| o.name().eq_ignore_ascii_case(&s.replace(['-', '_'], "")))
            .ok_or_else(|| {
                format!(
                    "unknown opening {s}; expected Default, Rush, GreedyExpand, Snowball, Economic"
                )
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscortPolicy {
    None,
    WaitForEscort,
    EscortIfThreat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Opening,
    Expand,
    Conquer,
    TurtleTech,
    Defend,
    Close,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseSwitch {
    pub turn: u32,
    pub phase: Phase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffReason {
    CityTarget,
    // Retained for reading reports from the original hand-off experiment. The
    // opening-specific completion rules no longer emit these two reasons.
    ArmyTarget,
    PhaseEnded,
    CoreBuild,
    SnowballReady,
    RushCaptured,
    RushAbandoned,
    TurnCap,
}

/// First turn planned by the unmodified scripted brain, retaining the original opening label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningHandoff {
    pub turn: u32,
    pub reason: HandoffReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildStep {
    pub item: ProductionItem,
    pub count: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Gene {
    pub name: String,
    pub min: i32,
    pub max: i32,
}

macro_rules! numbers {
    ($($name:ident: $default:expr, $min:expr, $max:expr;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Numbers { $(pub $name: i32,)* }
        impl Default for Numbers { fn default() -> Self { Self { $($name: $default,)* } } }
        impl Numbers {
            fn genes() -> Vec<Gene> { vec![$(Gene { name: stringify!($name).into(), min: $min, max: $max },)*] }
            fn genome(self) -> Vec<i32> { vec![$(self.$name,)*] }
            fn decode(values: &mut impl Iterator<Item = i32>) -> Self { Self { $($name: values.next().expect("validated length"),)* } }
        }
    }
}
numbers! {
    settler_target: 2, 0, 8;
    settler_start: 2, 2, 300;
    settler_interval: 30, 0, 200;
    expand_target: 5, 1, 12;
    gold_save: 240, 0, 4000;
    burst_count: 2, 1, 8;
    weak_percent: 70, 0, 200;
    war_army: 5, 1, 20;
    war_deadline: 130, 2, 400;
    capital_radius: 4, 1, 10;
    rush_timeout: 160, 20, 500;
    conquer_cities: 3, 1, 12;
    losses_limit: 3, 0, 20;
    recovery_turns: 40, 2, 200;
    defend_distance: 3, 1, 8;
    escort_radius: 6, 1, 15;
}

/// Items and research identities define each archetype. Counts, priority weights, timing,
/// radii, thresholds and purchase budgets are all exposed in the flat genome, in schema order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningPolicy {
    pub opening: Opening,
    pub builds: [BuildStep; 4],
    pub escort: EscortPolicy,
    pub burst_buy: ProductionItem,
    pub research: [TechId; 4],
    pub research_priority: [i32; 4],
    pub numbers: Numbers,
    #[serde(default)]
    pub primitives: PrimitiveParams,
}

fn unit(unit: UnitKind) -> ProductionItem {
    ProductionItem::Unit { unit }
}
fn building(building: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building }
}

impl OpeningPolicy {
    pub fn builtin(opening: Opening) -> Self {
        use BuildingKind::*;
        use TechId::*;
        use UnitKind::*;
        let (items, counts, escort, burst_buy, research, target, start) = match opening {
            Opening::Rush => (
                [unit(Spearman), unit(Archer), building(Market), unit(Archer)],
                [2, 1, 1, 1],
                EscortPolicy::None,
                unit(Archer),
                [Currency, BronzeWorking, Feudalism, Engineering],
                0,
                2,
            ),
            Opening::GreedyExpand => (
                [
                    unit(Settler),
                    unit(Settler),
                    building(Market),
                    building(Workshop),
                ],
                [1, 1, 1, 1],
                EscortPolicy::None,
                unit(Settler),
                [Irrigation, BronzeWorking, Currency, CodeOfLaws],
                3,
                2,
            ),
            Opening::Snowball => (
                [
                    building(Workshop),
                    unit(Settler),
                    building(Market),
                    unit(Archer),
                ],
                [1, 2, 1, 2],
                EscortPolicy::EscortIfThreat,
                unit(Archer),
                [BronzeWorking, Currency, Irrigation, Feudalism],
                2,
                30,
            ),
            Opening::Economic | Opening::Default => (
                [
                    building(Market),
                    unit(Settler),
                    building(Workshop),
                    building(Library),
                ],
                [1, 1, 1, 1],
                EscortPolicy::EscortIfThreat,
                building(Market),
                [Currency, BronzeWorking, Irrigation, Writing],
                2,
                30,
            ),
        };
        Self {
            opening,
            primitives: PrimitiveParams::default(),
            builds: std::array::from_fn(|i| BuildStep {
                item: items[i],
                count: counts[i],
            }),
            escort,
            burst_buy,
            research,
            research_priority: [400, 300, 200, 100],
            numbers: Numbers {
                settler_target: target,
                settler_start: start,
                conquer_cities: if opening == Opening::Rush { 2 } else { 3 },
                war_army: if opening == Opening::Rush { 5 } else { 7 },
                capital_radius: 4,
                rush_timeout: if opening == Opening::Rush { 80 } else { 160 },
                weak_percent: if matches!(opening, Opening::Economic) {
                    35
                } else {
                    70
                },
                ..Numbers::default()
            },
        }
    }
    pub fn genes() -> Vec<Gene> {
        let mut genes = Numbers::genes();
        genes.extend((0..4).map(|i| Gene {
            name: format!("builds.{i}.count"),
            min: 0,
            max: 8,
        }));
        genes.extend((0..4).map(|i| Gene {
            name: format!("research.{i}.priority"),
            min: 0,
            max: 1000,
        }));
        genes.push(Gene {
            name: "escort".into(),
            min: 0,
            max: 2,
        });
        genes.extend(PrimitiveParams::genes());
        genes
    }
    pub fn genes_for(opening: Opening) -> Vec<Gene> {
        let mut genes = Self::genes();
        if opening == Opening::Rush {
            genes
                .iter_mut()
                .find(|g| g.name == "rush_timeout")
                .unwrap()
                .min = 60;
        }
        let p = Self::builtin(opening);
        let offset = Numbers::genes().len();
        for (i, step) in p.builds.iter().enumerate() {
            if matches!(step.item, ProductionItem::Building { .. }) {
                genes[offset + i].max = 1;
            }
        }
        genes
    }
    pub fn genome(&self) -> Vec<i32> {
        let mut v = self.numbers.genome();
        v.extend(self.builds.map(|s| s.count));
        v.extend(self.research_priority);
        v.push(self.escort as i32);
        v.extend(self.primitives.genome());
        v
    }
    pub fn from_genome(opening: Opening, genome: &[i32]) -> Result<Self, String> {
        if opening == Opening::Default {
            return Err("Default has no tunable opening policy".into());
        }
        let genes = Self::genes_for(opening);
        if genes.len() != genome.len() {
            return Err(format!(
                "opening genome needs {} integers, got {}",
                genes.len(),
                genome.len()
            ));
        }
        for (gene, value) in genes.iter().zip(genome) {
            if !(gene.min..=gene.max).contains(value) {
                return Err(format!(
                    "{} must be {}..={}, got {value}",
                    gene.name, gene.min, gene.max
                ));
            }
        }
        let mut p = Self::builtin(opening);
        let mut values = genome.iter().copied();
        p.numbers = Numbers::decode(&mut values);
        for b in &mut p.builds {
            b.count = values.next().unwrap();
        }
        for r in &mut p.research_priority {
            *r = values.next().unwrap();
        }
        p.escort = match values.next().unwrap() {
            0 => EscortPolicy::None,
            1 => EscortPolicy::WaitForEscort,
            _ => EscortPolicy::EscortIfThreat,
        };
        p.primitives = PrimitiveParams::decode(&mut values);
        Ok(p)
    }
    pub fn schema() -> serde_json::Value {
        serde_json::json!({"schema": OPENING_SCHEMA, "time_unit": "Standard turns",
            "genes": Self::genes(), "openings": Opening::ALL.into_iter().filter(|o| *o != Opening::Default)
                .map(|o| { let p = Self::builtin(o); (o.name(), serde_json::json!({"policy": p, "genome": p.genome(), "genes": Self::genes_for(o)})) })
                .collect::<BTreeMap<_, _>>()})
    }
}

#[derive(Clone, Copy)]
pub(crate) struct OpeningPlan<'a> {
    pub policy: &'a OpeningPolicy,
    pub phase: Phase,
    pub next_build: Option<ProductionItem>,
    pub investment: Option<&'a OpeningInvestment>,
}

/// Completed investments observed from our own units, retained after losses.
#[derive(Clone, Default)]
pub(crate) struct OpeningInvestment {
    military_cost: i32,
    trained: BTreeMap<(CityId, ProductionItem), i32>,
}

pub struct OpeningBrain {
    pub base: ScriptedBrain,
    pub policy: OpeningPolicy,
    pub phase: Phase,
    pub switches: Vec<PhaseSwitch>,
    /// Non-default openings opt in automatically; Default needs the explicit sim flag.
    pub primitives_enabled: bool,
    handoff_enabled: bool,
    handoff: Option<OpeningHandoff>,
    primitive_memory: PrimitiveMemory,
    primitive_march: Option<CoalitionMarch>,
    seen_units: BTreeSet<UnitId>,
    previous_army: BTreeSet<UnitId>,
    known_owners: BTreeMap<CityId, legatum_core::PlayerId>,
    delivered: BTreeMap<ProductionItem, i32>,
    investment: OpeningInvestment,
    losses: i32,
    recovering_losses: i32,
    was_at_war: bool,
    captured: bool,
    abandoned_rush: bool,
    army_ready_turn: Option<u32>,
    strike_target: Option<TeamId>,
    strike_city: Option<CityId>,
    observed_turn: Option<u32>,
    /// Highest public team VP and the turn it last improved. Trading a title back and
    /// forth is not progress toward victory and must not reset the stall window.
    vp_progress: Option<(u32, u32)>,
    resume_phase: Phase,
    recovery_until: u32,
    /// Defend lasts at least until this turn after the last sighting of a threat.
    defend_until: u32,
}

impl OpeningBrain {
    /// Copy the stateful opening for an offline search branch. The shipped opening uses one
    /// instance and never calls this method.
    pub(crate) fn rollout_copy(&self) -> Self {
        Self {
            base: self.base.rollout_copy(),
            policy: self.policy.clone(),
            phase: self.phase,
            switches: self.switches.clone(),
            primitives_enabled: self.primitives_enabled,
            handoff_enabled: self.handoff_enabled,
            handoff: self.handoff,
            primitive_memory: self.primitive_memory.clone(),
            primitive_march: self.primitive_march.clone(),
            seen_units: self.seen_units.clone(),
            previous_army: self.previous_army.clone(),
            known_owners: self.known_owners.clone(),
            delivered: self.delivered.clone(),
            investment: self.investment.clone(),
            losses: self.losses,
            recovering_losses: self.recovering_losses,
            was_at_war: self.was_at_war,
            captured: self.captured,
            abandoned_rush: self.abandoned_rush,
            army_ready_turn: self.army_ready_turn,
            strike_target: self.strike_target,
            strike_city: self.strike_city,
            observed_turn: self.observed_turn,
            vp_progress: self.vp_progress,
            resume_phase: self.resume_phase,
            recovery_until: self.recovery_until,
            defend_until: self.defend_until,
        }
    }

    pub fn new(base: ScriptedBrain, policy: OpeningPolicy) -> Self {
        Self {
            primitives_enabled: policy.opening != Opening::Default,
            handoff_enabled: false,
            handoff: None,
            primitive_memory: PrimitiveMemory::default(),
            primitive_march: None,
            base,
            policy,
            phase: Phase::Opening,
            switches: Vec::new(),
            seen_units: BTreeSet::new(),
            previous_army: BTreeSet::new(),
            known_owners: BTreeMap::new(),
            delivered: BTreeMap::new(),
            investment: OpeningInvestment::default(),
            losses: 0,
            recovering_losses: 0,
            was_at_war: false,
            captured: false,
            abandoned_rush: false,
            army_ready_turn: None,
            strike_target: None,
            strike_city: None,
            observed_turn: None,
            vp_progress: None,
            resume_phase: Phase::Opening,
            recovery_until: 0,
            defend_until: 0,
        }
    }
    /// Opt-in Hard v2 experiment. The existing full-game opening policy is unchanged.
    pub(crate) fn with_handoff(mut self) -> Self {
        self.handoff_enabled = true;
        self
    }

    fn handoff_reason(&self, v: &PlayerView) -> Option<HandoffReason> {
        // Every named opening gets its actual turn-one orders.
        if !self.handoff_enabled || v.turn <= 1 {
            return None;
        }
        // settler_target counts colonies beyond the capital. Only owned cities count;
        // travelling/queued Settlers are investments, not completed settlements.
        let expanded = v.cities.iter().filter(|c| c.owner == v.me).count()
            > self.policy.numbers.settler_target as usize;
        let completed = match self.policy.opening {
            // Readiness/Conquer begins the rush; temporary defense does not end it.
            // observe/phase_for own the capture, timeout, and casualty decisions.
            Opening::Rush if self.captured => Some(HandoffReason::RushCaptured),
            Opening::Rush if self.abandoned_rush => Some(HandoffReason::RushAbandoned),
            Opening::GreedyExpand if expanded => Some(HandoffReason::CityTarget),
            Opening::Economic if self.next_build().is_none() => Some(HandoffReason::CoreBuild),
            Opening::Snowball
                if expanded
                    && self
                        .delivered
                        .get(&building(BuildingKind::Workshop))
                        .is_some_and(|count| *count > 0) =>
            {
                Some(HandoffReason::SnowballReady)
            }
            _ => None,
        };
        completed.or_else(|| {
            // 80 Standard turns.
            (v.turn >= 80).then_some(HandoffReason::TurnCap)
        })
    }
    fn observe(&mut self, v: &PlayerView) {
        if let Some(s) = v.standings.iter().find(|s| s.team == v.team)
            && self.vp_progress.is_none_or(|(best, _)| s.vp > best)
        {
            self.vp_progress = Some((s.vp, v.turn));
        }
        let cap = v.cities.iter().find(|c| c.owner == v.me && c.is_capital);
        // Build quotas count completed units once and retain credit after founding,
        // upgrading, or losses. Starting units (created on turn 1) are never builds.
        for u in v.units.iter().filter(|u| u.owner == v.me) {
            if self.seen_units.insert(u.id) && u.created_turn.is_some_and(|t| t > 1) {
                if cap.is_some_and(|c| c.pos.distance(u.pos) <= 1) {
                    *self.delivered.entry(unit(u.kind)).or_default() += 1;
                }
                if u.military && u.kind != UnitKind::Scout {
                    self.investment.military_cost +=
                        crate::tuned(legatum_core::data::unit_stats(u.kind).cost);
                }
                if let Some(city) = v
                    .cities
                    .iter()
                    .filter(|c| c.owner == v.me && c.pos.distance(u.pos) <= 1)
                    .min_by_key(|c| (c.pos.distance(u.pos), c.id))
                {
                    *self
                        .investment
                        .trained
                        .entry((city.id, unit(u.kind)))
                        .or_default() += 1;
                }
            }
        }
        if let Some(d) = cap.and_then(|c| c.detail.as_ref()) {
            for b in &d.buildings {
                self.delivered.insert(building(*b), 1);
            }
        }
        let at_war = v
            .players
            .iter()
            .any(|p| !p.id.is_barbarian() && v.at_war(v.me, p.id));
        if at_war && !self.was_at_war {
            self.losses = 0;
            self.recovering_losses = 0;
        }
        let army: BTreeSet<_> = v
            .units
            .iter()
            .filter(|u| u.owner == v.me && u.military && u.kind != UnitKind::Scout)
            .map(|u| u.id)
            .collect();
        if at_war || self.was_at_war {
            self.losses += self.previous_army.difference(&army).count() as i32;
        }
        self.previous_army = army;
        if self.previous_army.len() >= self.policy.numbers.war_army as usize {
            self.army_ready_turn.get_or_insert(v.turn);
        }
        self.was_at_war = at_war;
        for c in &v.cities {
            if c.owner == v.me && self.known_owners.get(&c.id).is_some_and(|o| *o != v.me) {
                self.captured = true;
            }
            self.known_owners.insert(c.id, c.owner);
        }
    }
    fn next_build(&self) -> Option<ProductionItem> {
        let mut required = BTreeMap::<_, i32>::new();
        for step in self.policy.builds {
            *required.entry(step.item).or_default() += step.count;
            if self.delivered.get(&step.item).copied().unwrap_or(0) < required[&step.item] {
                return Some(step.item);
            }
        }
        None
    }
    fn has_weak_neighbor(&self, v: &PlayerView) -> bool {
        if !self.primitives_enabled || self.policy.primitives.weakest_target_weight == 0 {
            return weak_neighbor(v, &self.policy).is_some();
        }
        let mut p = Planner::new(v, self.base.knobs, self.base.faction, self.base.seed);
        p.opponents = Some(&self.base.opponents);
        p.primitives = Some(PrimitivePlan {
            params: &self.policy.primitives,
            memory: &self.primitive_memory,
        });
        p.primitive_weakest_city(None).is_some_and(|(_, ratio)| {
            ratio
                < self.policy.numbers.weak_percent * self.policy.primitives.weakest_target_weight
                    / 100
        })
    }
    fn phase_for(&mut self, v: &PlayerView) -> Phase {
        // Let the opening and its development investments mature for the ordinary
        // game horizon. Then retire its build quotas and war deadline permanently:
        // the shared planner can price titles, Great Works and faction goals, or
        // prepare a legal campaign using the same public view and combat gates.
        // Anyone counting down to victory (public) ends the opening: the shared planner must
        // answer the hold now, whatever the build quotas say.
        if self.phase == Phase::Close
            || v.standings.iter().any(|s| s.victory_hold.is_some())
            || v.turn >= legatum_core::data::EXPECTED_GAME_TURNS
                && v.standings.iter().any(|s| {
                    s.team == v.team
                        && (s.target.saturating_sub(s.vp) <= 2
                            || self
                                .vp_progress
                                .is_some_and(|(_, last)| v.turn.saturating_sub(last) >= 120))
                })
        {
            return Phase::Close;
        }
        let n = self.policy.numbers;
        let own: Vec<_> = v.cities.iter().filter(|c| c.owner == v.me).collect();
        let threatened = own.iter().any(|c| {
            let enemies: Vec<_> = v
                .units
                .iter()
                .filter(|u| {
                    u.military
                        && (self.policy.opening != Opening::Rush || u.kind != UnitKind::Scout)
                        && !u.owner.is_barbarian()
                        && v.at_war(v.me, u.owner)
                        && c.pos.distance(u.pos) <= n.defend_distance
                })
                .collect();
            if self.policy.opening == Opening::Rush
                && !self.abandoned_rush
                && !self.captured
                && self.previous_army.len() >= n.war_army as usize
            {
                // A lone raider that the capital can hold must not recall an entire
                // assembled siege. Local defense assignments still cover the home.
                let defense = crate::analysis::city_power(v, c)
                    + v.units
                        .iter()
                        .filter(|u| {
                            u.owner == v.me
                                && u.military
                                && u.kind != UnitKind::Scout
                                && u.pos.distance(c.pos) <= 2
                        })
                        .map(|u| crate::analysis::defense_power(v, u))
                        .sum::<i32>();
                enemies.iter().map(|u| offense_power(v, u)).sum::<i32>() > defense
            } else {
                !enemies.is_empty()
            }
        });
        let coalition = v
            .diplomacy
            .coalition
            .as_ref()
            .is_some_and(|c| c.target == v.team && !c.members.is_empty());
        let base = if self.phase == Phase::Defend {
            self.resume_phase
        } else {
            self.phase
        };
        let losses = self.losses > n.losses_limit
            && self.losses > self.recovering_losses
            && (self.policy.opening != Opening::Rush
                || self.previous_army.len() * 2 < n.war_army as usize);
        let rush_failed = self.policy.opening == Opening::Rush
            && !self.abandoned_rush
            && !self.captured
            && (self
                .army_ready_turn
                .is_some_and(|ready| v.turn >= ready + n.rush_timeout.max(60) as u32)
                || self.army_ready_turn.is_some()
                    && self.losses >= n.losses_limit.max(2)
                    && self.previous_army.len() * 2 < n.war_army as usize);
        self.abandoned_rush |= rush_failed;
        let settlements = own.len()
            + v.units
                .iter()
                .filter(|u| u.owner == v.me && u.kind == UnitKind::Settler)
                .count();
        let mut next = if coalition || losses {
            self.recovering_losses = self.losses;
            self.recovery_until = v.turn + n.recovery_turns as u32;
            Phase::TurtleTech
        } else if base == Phase::TurtleTech && v.turn < self.recovery_until {
            Phase::TurtleTech
        } else if base == Phase::TurtleTech
            || rush_failed
            || self.policy.opening == Opening::Rush && self.captured
        {
            Phase::Expand
        } else if if self.policy.opening == Opening::Rush && !self.abandoned_rush {
            self.previous_army.len() >= n.war_army as usize
        } else {
            own.len() >= n.conquer_cities as usize && self.has_weak_neighbor(v)
        } {
            Phase::Conquer
        } else if base == Phase::Opening
            && self.policy.opening != Opening::Rush
            && self.next_build().is_none()
            && settlements > n.settler_target as usize
            && !matches!(self.policy.opening, Opening::Economic)
        {
            Phase::Expand
        } else {
            base
        };
        // Hysteresis: an army hovering at the edge of defend_distance used to flip the phase
        // (and the production strategy) every turn. Stay in Defend for four Standard turns after
        // the last sighting.
        if threatened {
            self.defend_until = v.turn + 4;
        }
        if threatened || self.phase == Phase::Defend && v.turn < self.defend_until {
            self.resume_phase = next;
            next = Phase::Defend;
        }
        next
    }
}

/// The nearest known empire is selected by a known capital (or its nearest known city).
/// The weakness test additionally requires current sight of that city. Fog is never zero power.
fn nearest_neighbor(v: &PlayerView) -> Option<&CityView> {
    let own: Vec<_> = v.cities.iter().filter(|c| c.owner == v.me).collect();
    v.cities
        .iter()
        .filter(|c| {
            !v.same_team(v.me, c.owner)
                && !c.owner.is_barbarian()
                && v.players.iter().any(|p| p.id == c.owner && p.alive)
        })
        .filter(|c| c.is_capital || !v.cities.iter().any(|x| x.owner == c.owner && x.is_capital))
        .min_by_key(|c| {
            (
                own.iter()
                    .map(|h| h.pos.distance(c.pos))
                    .min()
                    .unwrap_or(i32::MAX),
                c.owner,
                c.id,
            )
        })
}
/// Extra hexes a rush will march to reach an unwalled city instead of one that fires back.
const WALLED_DETOUR: i32 = 6;

fn weak_neighbor(v: &PlayerView, p: &OpeningPolicy) -> Option<TeamId> {
    let c = nearest_neighbor(v).filter(|c| c.visible)?;
    let ours: i32 = v
        .units
        .iter()
        .filter(|u| u.owner == v.me && u.military)
        .map(|u| offense_power(v, u))
        .sum();
    let theirs: i32 = v
        .units
        .iter()
        .filter(|u| {
            u.owner == c.owner && u.military && u.pos.distance(c.pos) <= p.numbers.capital_radius
        })
        .map(|u| offense_power(v, u))
        .sum();
    (ours > 0 && i64::from(theirs) * 100 < i64::from(ours) * i64::from(p.numbers.weak_percent))
        .then(|| v.team_of(c.owner))
}

impl Brain for OpeningBrain {
    fn add_air_orders(&self, view: &PlayerView, orders: &mut Vec<legatum_core::Order>) {
        if self.base.decisions.air_wrapper_orders > 0 {
            self.base.add_air_orders(view, orders);
        }
    }

    fn opponent_memory(&self) -> Option<crate::OpponentMemory> {
        self.base.opponent_memory()
    }
    fn restore_opponent_memory(&mut self, memory: crate::OpponentMemory) {
        self.base.restore_opponent_memory(memory);
    }

    fn strategy_switches(&self) -> &[crate::opponents::StrategySwitch] {
        &self.base.strategy_switches
    }
    fn threat_log(&self) -> &[crate::threat::ThreatEvent] {
        &self.base.threat_log
    }
    fn campaign_events(&self) -> &[crate::campaign::CampaignEvent] {
        &self.base.campaigns.events
    }
    fn opening_handoff(&self) -> Option<OpeningHandoff> {
        self.handoff
    }
    fn primitive_coalition_march(&self) -> Option<&CoalitionMarch> {
        self.primitive_march.as_ref()
    }
    fn opening_phase_switches(&self) -> &[PhaseSwitch] {
        &self.switches
    }
    fn opening(&self) -> Option<Opening> {
        Some(self.policy.opening)
    }
    fn plan(&mut self, v: &PlayerView) -> Vec<Order> {
        if self.handoff.is_some()
            || self.policy.opening == Opening::Default && !self.primitives_enabled
        {
            return self.base.plan(v);
        }
        if v.finished {
            return Vec::new();
        }
        if self.observed_turn != Some(v.turn) {
            self.base.opponents.threats.runaway_read = self.base.decisions.runaway_read;
            self.base.opponents.threats.hollow_read = self.base.decisions.hollow_read;
            self.base.opponents.threats.army_threat_weight = self.base.decisions.army_threat_weight;
            self.base.opponents.threats.threat_horizon_turns =
                self.base.decisions.threat_horizon_turns;
            self.base
                .opponents
                .update_with_risk(v, self.base.decisions.assessment_interval > 0);
            self.primitive_memory.observe(v);
            if self.policy.opening != Opening::Default {
                self.observe(v);
                let previous = self.phase;
                let mut next = self.phase_for(v);
                let mut reason = "opening_phase_conditions";
                let mut rival = None;
                let model = &self.base.opponents;
                let known_targets = model.rivals.values().any(|r| !r.cities.is_empty());
                if (next == Phase::Conquer
                    || next == Phase::Opening && self.policy.opening == Opening::Rush)
                    && known_targets
                    && !model.has_viable_target(crate::adaptation::early(v))
                {
                    next = Phase::Expand;
                    self.abandoned_rush = true;
                    self.strike_city = None;
                    self.strike_target = None;
                    reason = "no_viable_targets";
                    rival = model.rivals.keys().next().copied();
                }
                // Defended expansion keeps the expansion quota while retiring the greedy build list.
                if matches!(next, Phase::Opening | Phase::Expand)
                    && self.policy.opening != Opening::Rush
                    && let Some(threat) = model.threatening_neighbor()
                {
                    next = Phase::Expand;
                    reason = "aggressive_neighbor_defended_expansion";
                    rival = Some(threat.id);
                }
                if next != previous {
                    self.base
                        .strategy_switches
                        .push(crate::opponents::StrategySwitch {
                            turn: v.turn,
                            player: v.me,
                            rival,
                            from: format!("{previous:?}"),
                            to: format!("{next:?}"),
                            reason: reason.into(),
                        });
                }
                if next != self.phase || self.switches.is_empty() {
                    self.switches.push(PhaseSwitch {
                        turn: v.turn,
                        phase: next,
                    });
                    self.phase = next;
                }
            }
            self.observed_turn = Some(v.turn);
        }
        if let Some(reason) = self.handoff_reason(v) {
            self.handoff = Some(OpeningHandoff {
                turn: v.turn,
                reason,
            });
            self.primitive_march = None;
            return self.base.plan(v);
        }
        let strategy = match self.phase {
            Phase::Close => {
                self.primitive_march = None;
                return self.base.plan(v);
            }
            Phase::Expand => Strategy::Wide,
            Phase::Conquer => Strategy::Rush,
            Phase::TurtleTech => Strategy::Science,
            Phase::Defend => Strategy::Turtle,
            Phase::Opening => match self.policy.opening {
                Opening::Rush => Strategy::Rush,
                Opening::GreedyExpand => Strategy::Wide,
                _ => Strategy::Economic,
            },
        };
        let a = self.base.knobs;
        let b = strategy.knobs();
        let knobs = if self.policy.opening == Opening::Default {
            a
        } else {
            Knobs::new(
                (a.expansion + b.expansion) / 2,
                (a.military + b.military) / 2,
                (a.science + b.science) / 2,
                (a.economy + b.economy) / 2,
                (a.aggression + b.aggression) / 2,
            )
        };
        let mut p = Planner::new(v, knobs, self.base.faction, self.base.seed);
        p.personality_aggression = self.base.knobs.aggression;
        p.campaigns = self.base.campaigns.clone();
        p.opponents = Some(&self.base.opponents);
        p.difficulty = self.base.difficulty;
        p.decisions = self.base.decisions.for_faction(self.base.faction);
        p.strategy = if self.policy.opening == Opening::Default {
            self.base.strategy
        } else {
            Some(strategy)
        };
        let opening = (self.policy.opening != Opening::Default).then_some(OpeningPlan {
            policy: &self.policy,
            phase: self.phase,
            investment: Some(&self.investment),
            next_build: matches!(self.phase, Phase::Opening | Phase::Conquer)
                .then(|| self.next_build())
                .flatten(),
        });
        p.opening = opening;
        p.primitives = self.primitives_enabled.then_some(PrimitivePlan {
            params: &self.policy.primitives,
            memory: &self.primitive_memory,
        });
        if p.rushing() {
            p.strike_target = self.strike_target;
            p.strike_city = self.strike_city.filter(|id| {
                v.cities.iter().any(|c| {
                    c.id == *id
                        && !v.same_team(v.me, c.owner)
                        && v.players
                            .iter()
                            .any(|player| player.id == c.owner && player.alive)
                })
            });
            if p.strike_city.is_some_and(|id| {
                v.cities
                    .iter()
                    .any(|c| c.id == id && !p.assessed_campaign_allowed(c))
            }) {
                p.strike_city = None;
                p.strike_target = None;
            }
        }
        p.reconcile_campaign();
        p.plan_diplomacy();
        if p.rushing() {
            // Do not sign away the rush before reconnaissance identifies its target.
            p.orders.retain(|order| match order {
                Order::ProposeTreaty { kind, .. } => {
                    *kind != legatum_core::diplomacy::TreatyKind::Alliance
                }
                Order::RespondToProposal {
                    proposal,
                    accept: true,
                    ..
                } => !v.diplomacy.proposals.iter().any(|offer| {
                    offer.id == *proposal
                        && offer.kind == legatum_core::diplomacy::TreatyKind::Alliance
                }),
                _ => true,
            });
        }
        p.plan_primitive_war();
        if opening.is_some() {
            p.orders.retain(|o| !matches!(o, Order::DeclareWar { .. }));
            p.plan_opening_war();
        }
        p.strike_target = p.strike_target.or_else(|| {
            p.orders.iter().find_map(|o| {
                if let Order::DeclareWar { against, .. } = o {
                    Some(*against)
                } else {
                    None
                }
            })
        });
        // G20: plain-Peace declarations take effect before movement. Project only our
        // legal, public orders into a copy of our own view; never consult hidden state.
        let mut prospective = v.clone();
        for order in &p.orders {
            if let Order::DeclareWar {
                against,
                breach: false,
            } = order
                && let Some(r) = prospective
                    .diplomacy
                    .relations
                    .iter_mut()
                    .find(|r| r.between(v.team, *against))
                && r.state == legatum_core::diplomacy::RelationState::Peace
                && r.security.is_none()
            {
                r.state = legatum_core::diplomacy::RelationState::War;
            }
        }
        let mut next = Planner::new(&prospective, knobs, self.base.faction, self.base.seed);
        next.personality_aggression = p.personality_aggression;
        next.campaigns = p.campaigns;
        next.campaign_withdraw = p.campaign_withdraw;
        next.opponents = p.opponents;
        next.difficulty = p.difficulty;
        next.decisions = p.decisions;
        next.strategy = p.strategy;
        next.opening = opening;
        next.primitives = p.primitives;
        next.strike_target = p.strike_target;
        next.strike_city = p.strike_city;
        next.orders = p.orders;
        let mut p = next;
        p.plan_egress();
        p.plan_opening_research();
        p.plan_research();
        p.plan_yields();
        p.plan_cities();
        p.plan_settlers();
        p.plan_scouts();
        p.plan_military();
        p.dedupe_first_steps();
        p.enforce_levy();
        if opening.is_some() {
            // Movement validates a simultaneous declaration, but ranged fire requires
            // an enemy in the original view. Wait until the next volley in a new war.
            for order in &mut p.orders {
                if let Order::RangedAttack { unit, target } = *order
                    && !v
                        .units
                        .iter()
                        .any(|u| u.military && u.pos == target && v.at_war(v.me, u.owner))
                    && !v
                        .cities
                        .iter()
                        .any(|c| c.pos == target && v.at_war(v.me, c.owner))
                {
                    *order = Order::SkipUnit { unit };
                }
            }
        }
        // A declaration can legalize a direct attack, but confirmed routes must
        // stop at the current border. The planner used the prospective war view;
        // trim any longer route to its last currently legal step.
        for order in &mut p.orders {
            if let Order::MoveUnitPath { unit, path, .. } = order {
                let Some((mut from, kind)) = v
                    .units
                    .iter()
                    .find(|u| u.id == *unit)
                    .map(|u| (u.pos, u.kind))
                else {
                    continue;
                };
                let mut legal = Vec::new();
                for &step in path.iter() {
                    if v.movement_path(v.me, kind, from, step).as_deref() != Some(&[step][..]) {
                        break;
                    }
                    legal.push(step);
                    from = step;
                }
                if legal.len() != path.len() {
                    *order = if let Some(&to) = legal.last() {
                        Order::MoveUnitPath {
                            unit: *unit,
                            to,
                            path: legal,
                            stop_on_block: true,
                        }
                    } else {
                        Order::SkipUnit { unit: *unit }
                    };
                }
            }
        }
        let final_orders = p.orders.clone();
        p.adopt_started_siege(v, &final_orders);
        self.primitive_march = p.primitive_march;
        self.strike_target = p.strike_target;
        self.strike_city = p.strike_city;
        self.base.campaigns = p.campaigns;
        p.orders
    }
}

impl Planner<'_> {
    fn plan_opening_research(&mut self) {
        let Some(o) = self.opening else { return };
        if self.view().research.current.is_some() {
            return;
        }
        if self.prepare_border_defense() {
            return;
        }
        let mut priorities: Vec<_> = o
            .policy
            .research
            .iter()
            .zip(o.policy.research_priority)
            .enumerate()
            .collect();
        priorities.sort_by_key(|(i, (_, weight))| (std::cmp::Reverse(*weight), *i));
        for (_, (tech, weight)) in priorities {
            if weight == 0 || self.view().research.known.contains(tech) {
                continue;
            }
            let mut needed = BTreeSet::new();
            crate::investment::prerequisites(*tech, &self.view().research.known, &mut needed);
            needed.insert(*tech);
            if let Some(opt) = self
                .view()
                .research
                .available
                .iter()
                .find(|t| needed.contains(&t.tech))
            {
                self.orders.push(Order::SetResearch { tech: opt.tech });
                return;
            }
        }
    }
    pub(crate) fn rushing(&self) -> bool {
        self.opening.is_some_and(|o| {
            o.policy.opening == Opening::Rush && matches!(o.phase, Phase::Opening | Phase::Conquer)
        })
    }

    /// One view-derived city is shared by selection, the force gate and marching.
    pub(crate) fn opening_strike_city(&self) -> Option<&CityView> {
        let v = self.view();
        if let Some(id) = self.strike_city
            && let Some(city) = v.cities.iter().find(|c| {
                c.id == id
                    && self.assessed_campaign_allowed(c)
                    && !(legatum_core::city_fires(c) && !v.at_war(v.me, c.owner))
            })
        {
            return Some(city);
        }
        v.cities
            .iter()
            .filter(|c| {
                !v.same_team(v.me, c.owner)
                    && self.assessed_campaign_allowed(c)
                    && !c.owner.is_barbarian()
                    && (self.faction != legatum_core::Faction::Sparta || !c.walls)
                    && self.strike_target.is_none_or(|t| v.team_of(c.owner) == t)
                    && v.players.iter().any(|p| p.id == c.owner && p.alive)
            })
            .filter_map(|c| {
                let distance = self
                    .ctx
                    .own_cities
                    .iter()
                    .filter(|h| self.opening_reachable(h.pos, c.pos))
                    .map(|h| h.pos.distance(c.pos))
                    .min()?;
                let walled = if legatum_core::city_fires(c) {
                    WALLED_DETOUR
                } else {
                    0
                };
                Some((
                    (distance + walled) * 10 - self.assessed_target_value(c),
                    c.id,
                    c,
                ))
            })
            .min_by_key(|(distance, id, _)| (*distance, *id))
            .map(|(_, _, c)| c)
    }

    /// A favorable ratio cannot substitute for a healthy army actually at the front.
    pub(crate) fn opening_reachable(
        &self,
        from: legatum_core::Hex,
        target: legatum_core::Hex,
    ) -> bool {
        if self.ctx.reachable(from, target) {
            return true;
        }
        if self.opening.is_none() {
            return false;
        }
        let v = self.view();
        let owner = v.tile(target).and_then(|t| t.owner);
        // find_path reads only the map's geometry; costs come from the closure.
        let map = self.movement_map();
        // Assess the known land route after war opens the chosen rival's borders.
        // Actual peacetime movement still uses the ordinary legal movement graph.
        legatum_core::path::find_path(map, from, target, |h| {
            let tile = v.tile(h)?;
            (tile.explored
                && (v.passage_allowed(v.me, legatum_core::UnitKind::Spearman, tile.owner)
                    || tile.owner == owner))
                .then_some(tile.move_cost())
                .flatten()
        })
        .is_some()
    }

    fn opening_force_ready(&self, target: legatum_core::Hex) -> bool {
        let Some(o) = self.opening else { return true };
        // Small tuned radii can lie inside closed borders. Troops just outside
        // the city's maximum border must be able to satisfy the peacetime gate.
        let staging_radius = o
            .policy
            .numbers
            .capital_radius
            .max(legatum_core::data::CITY_WORK_RADIUS + 1);
        let force: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.hp >= 60
                    && u.pos.distance(target) <= staging_radius
                    && self.opening_reachable(u.pos, target)
            })
            .collect();
        force.len() >= o.policy.numbers.war_army as usize
            && force.iter().any(|u| crate::composition::charges(u.kind))
    }

    /// A legacy nearest-neighbor nomination must not pin an opening to a rival whose
    /// cities the assessed campaign policy rejects. Rank the model's viable alternatives,
    /// then validate remembered candidates against the current view and known routes.
    fn assessed_opening_city(&self) -> Option<&CityView> {
        let model = self.opponents?;
        let v = self.view();
        model
            .rivals
            .values()
            .flat_map(|rival| {
                rival
                    .viable_targets(crate::adaptation::early(v))
                    .filter_map(move |known| {
                        let city = v
                            .cities
                            .iter()
                            .find(|city| city.id == known.id && city.owner == rival.id)?;
                        if !self.assessed_campaign_allowed(city) {
                            return None;
                        }
                        let distance = self
                            .ctx
                            .own_cities
                            .iter()
                            .filter(|home| self.opening_reachable(home.pos, city.pos))
                            .map(|home| home.pos.distance(city.pos))
                            .min()?;
                        Some((
                            (distance
                                + if legatum_core::city_fires(city) {
                                    WALLED_DETOUR
                                } else {
                                    0
                                })
                                * 10
                                - self.assessed_target_value(city),
                            city.id,
                            city,
                        ))
                    })
            })
            .min_by_key(|(score, id, _)| (*score, *id))
            .map(|(_, _, city)| city)
    }

    fn plan_opening_war(&mut self) {
        let Some(o) = self.opening else { return };
        if matches!(o.phase, Phase::Defend | Phase::TurtleTech) {
            return;
        }
        // Primitives nominate a target; the opening owns permission to declare. Rush
        // also needs a peacetime destination before local superiority can be measured.
        if self.opponents.is_some()
            && self.strike_target.is_some_and(|team| {
                self.opening_strike_city().is_none() && self.primitive_snipe_target(team).is_none()
            })
        {
            self.strike_city = None;
            self.strike_target = None;
        }
        if self.strike_target.is_none() {
            self.strike_target = if self.opponents.is_some() {
                self.assessed_opening_city()
                    .map(|city| self.view().team_of(city.owner))
            } else {
                weak_neighbor(self.view(), o.policy)
            };
        }
        if self.strike_target.is_none() && !self.rushing() {
            return;
        }
        let city = self
            .opening_strike_city()
            .map(|c| (self.view().team_of(c.owner), c.id, c.pos));
        let Some(against) = city.map(|c| c.0).or(self.strike_target) else {
            return;
        };
        self.strike_target = Some(against);
        self.strike_city = city.map(|c| c.1);
        self.retain_strike(against);
        if self.view().teams_at_war(self.view().team, against) {
            return;
        }
        let ready = city.is_some_and(|(_, _, pos)| self.opening_force_ready(pos))
            || self
                .primitive_snipe_target(against)
                .is_some_and(|pos| self.opening_force_ready(pos));
        if !ready || self.view().turn > o.policy.numbers.war_deadline as u32 {
            return;
        }
        // Diplomacy already answered this team's offer; a second action toward the
        // same team would invalidate the whole order batch.
        if self.orders.iter().any(|order| match order {
            Order::RespondToProposal { proposal, .. } => self
                .view()
                .diplomacy
                .proposals
                .iter()
                .any(|p| p.id == *proposal && p.from == against),
            _ => false,
        }) {
            return;
        }
        let order = Order::DeclareWar {
            against,
            breach: false,
        };
        if self
            .view()
            .diplomacy
            .actions
            .iter()
            .any(|a| a.order == order && a.blocked_reason.is_none())
        {
            self.retain_strike(against);
            self.orders.push(order);
        } else {
            // An ally blocks the strike: leave the Alliance once that is free (no betrayal),
            // and declare at plain Peace afterwards.
            let leave = Order::BreakTreaty {
                with: against,
                agreement: legatum_core::diplomacy::Agreement::Security,
            };
            if self.rushing()
                && self.view().diplomacy.actions.iter().any(|a| {
                    a.order == leave && a.blocked_reason.is_none() && a.cost == 0 && !a.betrayal
                })
                && !self.orders.contains(&leave)
            {
                self.orders.push(leave);
            }
        }
    }

    pub(crate) fn retain_strike(&mut self, against: TeamId) {
        let proposals = &self.ctx.view.diplomacy.proposals;
        self.orders.retain(|o| {
            !matches!(o, Order::ProposeTreaty { to, .. } if *to == against)
                && !matches!(o, Order::RespondToProposal { proposal, accept: true, .. }
                    if proposals.iter().any(|p| p.id == *proposal && p.from == against))
        });
    }
    pub(crate) fn opening_target(&self) -> i32 {
        let Some(o) = self.opening else { return 0 };
        if o.phase == Phase::Expand {
            o.policy.numbers.expand_target
        } else {
            o.policy.numbers.settler_target
        }
    }

    /// Effective construction, queued items and independent purchases, each counted once.
    fn opening_commitments(&self, c: &CityView) -> Vec<ProductionItem> {
        let Some(d) = c.detail.as_ref() else {
            return Vec::new();
        };
        // SetProduction clears the old queue; SetProductionQueue includes the
        // active item in its first slot (matching core's city-order semantics).
        let mut items = self
            .orders
            .iter()
            .rev()
            .find_map(|o| match o {
                Order::SetProduction { city, item } if *city == c.id => Some(vec![*item]),
                Order::SetProductionQueue { city, queue } if *city == c.id => Some(queue.clone()),
                _ => None,
            })
            .unwrap_or_else(|| {
                d.producing
                    .into_iter()
                    .chain(d.production_queue.iter().copied())
                    .collect()
            });
        items.extend(self.orders.iter().filter_map(|o| match o {
            Order::BuyItem { city, item } if *city == c.id => Some(*item),
            _ => None,
        }));
        items
    }

    /// Every opening settler producer and purchaser uses the same admission rule.
    /// Travelling units, queues and orders already issued this turn consume the target
    /// and advance the cadence. Default's ordinary expansion planner is untouched.
    pub(crate) fn opening_admits_settler(&self) -> bool {
        let Some(o) = self.opening else { return true };
        let n = o.policy.numbers;
        let pipeline = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.kind == UnitKind::Settler)
            .count() as i32
            + self
                .ctx
                .own_cities
                .iter()
                .flat_map(|c| self.opening_commitments(c))
                .filter(|i| *i == unit(UnitKind::Settler))
                .count() as i32;
        let committed = (self.ctx.own_cities.len() as i32 - 1).max(0) + pipeline;
        committed < self.opening_target()
            && self.view().turn >= (n.settler_start + committed * n.settler_interval) as u32
            && self.count_good_sites(pipeline + 1) > pipeline
    }

    /// A light early army is an investment budget, not a limit on emergency defense.
    pub(crate) fn opening_allows_optional_military(&self, item: ProductionItem) -> bool {
        let Some(o) = self.opening else { return true };
        let cost = |item| match item {
            ProductionItem::Unit { unit }
                if unit != UnitKind::Scout && legatum_core::data::unit_stats(unit).military =>
            {
                crate::tuned(legatum_core::data::unit_stats(unit).cost)
            }
            _ => 0,
        };
        if self
            .ctx
            .own_cities
            .iter()
            .any(|c| self.assessed_border_threat(c.pos))
            || o.policy.opening != Opening::GreedyExpand
            || self.view().turn > 120
            || cost(item) == 0
        {
            return true;
        }
        let spent = o.investment.map_or_else(
            || {
                self.ctx
                    .own_units
                    .iter()
                    .filter(|u| u.created_turn.is_some_and(|t| t > 1))
                    .map(|u| cost(unit(u.kind)))
                    .sum()
            },
            |i| i.military_cost,
        );
        let committed: i32 = self
            .ctx
            .own_cities
            .iter()
            .flat_map(|c| self.opening_commitments(c))
            .map(cost)
            .sum();
        spent + committed + cost(item) <= 140
    }

    pub(crate) fn opening_choice(&self, c: &CityView) -> Option<ProductionItem> {
        let o = self.opening?;
        let d = c.detail.as_ref()?;
        let available = |item| {
            d.options.iter().any(|x| x.item == item)
                && !self.orders.iter().any(|order| {
                    matches!(order,
                Order::BuyItem { city, item: bought } if *city == c.id && *bought == item
                    && matches!(item, ProductionItem::Building { .. }))
                })
        };
        if matches!(o.phase, Phase::Defend | Phase::TurtleTech) {
            return None;
        }
        if c.is_capital {
            if let Some(item) = o.next_build.filter(|i| {
                available(*i) && (*i != unit(UnitKind::Settler) || self.opening_admits_settler())
            }) {
                return Some(item);
            }
            if self.opening_admits_settler() && available(unit(UnitKind::Settler)) {
                return Some(unit(UnitKind::Settler));
            }
        }
        let families: &[BuildingKind] = match o.policy.opening {
            Opening::Snowball if !c.is_capital => &[BuildingKind::Workshop],
            _ => &[],
        };
        for family in families {
            // Snowball's colonies continue investing in their productive capacity.
            if let Some(item) = d.options.iter().map(|o| o.item).find(|item| {
                available(*item)
                    && matches!(item, ProductionItem::Building { building: b }
                    if legatum_core::data::building_family(*b) == *family)
            }) {
                return Some(item);
            }
        }
        None
    }

    pub(crate) fn plan_opening_buy(&mut self) {
        let Some(o) = self.opening else { return };
        // The merged Rush purchaser already buys its missing melee/ranged muster
        // against an upkeep runway. A no-op must not spend that runway here.
        if self.rushing()
            && self
                .primitives
                .is_some_and(|p| p.params.save_and_burst_weight > 0)
        {
            return;
        }
        let threshold = o.policy.numbers.gold_save;
        if self.gold_left < threshold {
            return;
        }
        let mut cities = self.ctx.own_cities.clone();
        cities.sort_by_key(|c| (!c.is_capital, c.id));
        let mut bought = 0;
        for c in cities {
            let target = o.policy.burst_buy;
            // Each Settler bought raises the next one's quote (the trained-settler surcharge),
            // which the view's quotes cannot show: one a turn.
            if bought >= o.policy.numbers.burst_count
                || target == unit(UnitKind::Settler) && bought >= 1
            {
                break;
            }
            if target == unit(UnitKind::Settler) && !self.opening_admits_settler()
                || !self.opening_allows_optional_military(target)
                || self
                    .orders
                    .iter()
                    .any(|x| matches!(x, Order::BuyItem { city, .. } if *city == c.id))
            {
                continue;
            }
            let Some(opt) = c.detail.as_ref().and_then(|d| {
                d.options.iter().find(|p| {
                    let matches = match (target, p.item) {
                        (
                            ProductionItem::Building { building: family },
                            ProductionItem::Building { building: b },
                        ) if matches!(o.policy.opening, Opening::Economic) => {
                            legatum_core::data::building_family(b) == family
                        }
                        _ => p.item == target,
                    };
                    matches
                        && p.buy_block.is_none()
                        && p.buy_cost
                            .is_some_and(|cost| cost > 0 && cost <= self.gold_left)
                })
            }) else {
                continue;
            };
            self.orders.push(Order::BuyItem {
                city: c.id,
                item: opt.item,
            });
            self.gold_left -= opt.buy_cost.expect("affordable quote");
            bought += 1;
        }
    }
    /// Reserve escorts before any garrison orders. Settlers wait for rendezvous and then
    /// move one shared safe step, so a faster settler cannot leave its escort behind.
    pub(crate) fn plan_opening_escorts(&mut self) {
        let Some(o) = self.opening else { return };
        if o.policy.escort == EscortPolicy::None || self.ctx.own_cities.is_empty() {
            return;
        }
        let settlers: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.kind == UnitKind::Settler)
            .collect();
        for s in settlers {
            let threatened = self
                .ctx
                .threat_units
                .iter()
                .any(|e| e.military && e.pos.distance(s.pos) <= o.policy.numbers.escort_radius)
                || self
                    .ctx
                    .remembered_threats
                    .iter()
                    .any(|e| e.pos.distance(s.pos) <= o.policy.numbers.escort_radius);
            if o.policy.escort == EscortPolicy::EscortIfThreat && !threatened {
                continue;
            }
            let escort = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && !self.has_acted(u.id)
                        && u.pos.distance(s.pos) <= o.policy.numbers.escort_radius
                        && self.ctx.reachable(u.pos, s.pos)
                })
                .min_by_key(|u| (u.pos.distance(s.pos), u.id));
            if self
                .orders
                .iter()
                .any(|x| matches!(x, Order::FoundCity { unit } if *unit == s.id))
            {
                if let Some(u) = escort.filter(|u| u.pos == s.pos) {
                    self.go_to(u, s.pos);
                    continue;
                }
                if o.policy.escort != EscortPolicy::WaitForEscort {
                    continue;
                }
                // WaitForEscort covers the founding turn too, including a rendezvous
                // interrupted on the previous turn. The initial capital is exempt above.
            }
            let destination = self.orders.iter().find_map(|x| match x {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                    if *unit == s.id =>
                {
                    Some(*to)
                }
                _ => None,
            });
            let next = escort.filter(|u| u.pos == s.pos).and_then(|u| {
                // A civilian route can cross a friendly garrison that blocks its
                // escort. Find a route both can follow instead of holding the pair
                // forever at that occupied first step.
                // find_path reads only the map's geometry.
                let map = self.movement_map();
                let to = destination?;
                // Unknown terrain is potentially passable, as in PlayerView::movement_path;
                // treating fog as a wall would hold the pair at the frontier forever.
                let cost = |h: legatum_core::Hex| {
                    self.ctx.move_cost(h).or_else(|| {
                        let t = self.view().tile(h)?;
                        (!t.explored
                            && self.view().passage_allowed(
                                self.ctx.me,
                                legatum_core::UnitKind::Spearman,
                                t.owner,
                            ))
                        .then(|| legatum_core::Terrain::Grassland.move_cost())
                        .flatten()
                    })
                };
                let path = legatum_core::path::find_path(map, s.pos, to, |h| {
                    cost(h).filter(|cost| {
                        *cost <= s.max_moves.min(u.max_moves)
                            && self.free_tile_for(u, h)
                            && self.safe_campaign_tile(h)
                    })
                })
                .or_else(|| self.view().movement_path(self.ctx.me, s.kind, s.pos, to))?;
                path.first().copied()
            });
            // Keep tactical escapes; the escort should follow those rather than cancel them.
            if !self.retreats.contains(&s.id) {
                self.orders
                    .retain(|x| legatum_core::rules::units::order_unit(x) != Some(s.id));
                self.orders
                    .push(
                        next.map_or(Order::SkipUnit { unit: s.id }, |to| Order::MoveUnit {
                            unit: s.id,
                            to,
                        }),
                    );
            }
            if let Some(u) = escort {
                let to = next.unwrap_or(s.pos);
                if self.free_tile_for(u, to) {
                    self.go_to(u, to);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::{Faction, Hex, PlayerId};

    fn base() -> ScriptedBrain {
        ScriptedBrain::new(Faction::Switzerland, 7, ME)
    }
    fn planner<'a>(v: &'a PlayerView, p: &'a OpeningPolicy) -> Planner<'a> {
        let mut planner = Planner::new(v, base().knobs, Faction::Switzerland, 7);
        planner.opening = Some(OpeningPlan {
            investment: None,
            policy: p,
            phase: Phase::Opening,
            next_build: Some(p.builds[0].item),
        });
        planner
    }

    #[test]
    fn every_numeric_gene_round_trips_and_rejects_bad_bounds() {
        for opening in Opening::ALL.into_iter().filter(|o| *o != Opening::Default) {
            let p = OpeningPolicy::builtin(opening);
            let values = p.genome();
            assert_eq!(OpeningPolicy::from_genome(opening, &values).unwrap(), p);
            assert!(OpeningPolicy::from_genome(opening, &values[..values.len() - 1]).is_err());
            for (i, gene) in OpeningPolicy::genes_for(opening).iter().enumerate() {
                for v in [gene.min, gene.max] {
                    let mut values = values.clone();
                    values[i] = v;
                    assert_eq!(
                        OpeningPolicy::from_genome(opening, &values)
                            .unwrap()
                            .genome(),
                        values
                    );
                }
                for v in [gene.min - 1, gene.max + 1] {
                    let mut values = values.clone();
                    values[i] = v;
                    assert!(
                        OpeningPolicy::from_genome(opening, &values).is_err(),
                        "{}",
                        gene.name
                    );
                }
            }
        }
        assert!(OpeningPolicy::from_genome(Opening::Default, &[]).is_err());
    }

    #[test]
    fn repeated_planning_cannot_advance_two_phases_in_one_turn() {
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ME, Hex::new(5, 10), 3)
            .city(3, ENEMY, Hex::new(13, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(5, 5))
            .unit(2, ME, UnitKind::Spearman, Hex::new(10, 4))
            .unit(3, ME, UnitKind::Spearman, Hex::new(11, 4))
            .build();
        v.turn = 80;
        let mut policy = OpeningPolicy::builtin(Opening::Rush);
        policy.numbers.war_army = 3;
        let mut brain = OpeningBrain::new(base(), policy);
        let first = brain.orders(&v);
        assert_eq!(brain.phase, Phase::Conquer);
        assert_eq!(brain.orders(&v), first);
        assert_eq!(brain.switches.len(), 1);
        v.turn += 1;
        brain.orders(&v);
        assert_eq!(brain.phase, Phase::Conquer);
    }

    #[test]
    fn settler_purchases_honor_timing_and_count_committed_production() {
        let mut v = ViewBuilder::new(28, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ME, Hex::new(5, 10), 3)
            .gold(2000)
            .build();
        let mut policy = OpeningPolicy::builtin(Opening::GreedyExpand);
        policy.numbers.settler_target = 2;
        policy.numbers.settler_start = 4;
        policy.numbers.settler_interval = 0;
        v.turn = 1;
        for c in &mut v.cities {
            for opt in &mut c.detail.as_mut().unwrap().options {
                if opt.item == policy.burst_buy {
                    opt.buy_cost = Some(100);
                }
            }
        }
        let mut p = planner(&v, &policy);
        p.plan_opening_buy();
        assert!(p.orders.is_empty(), "purchase cannot bypass Settler timing");
        v.turn = 4;
        v.cities[0].detail.as_mut().unwrap().producing = Some(policy.burst_buy);
        let mut p = planner(&v, &policy);
        p.plan_opening_buy();
        assert!(
            p.orders.is_empty(),
            "a Settler under construction already fills the target"
        );
        v.cities[0].detail.as_mut().unwrap().producing = None;
        let mut p = planner(&v, &policy);
        p.plan_opening_buy();
        assert_eq!(p.orders.len(), 1, "burst cannot overshoot the target");
        assert!(matches!(p.orders[0], Order::BuyItem { item, .. } if item == policy.burst_buy));
    }

    #[test]
    fn mixed_opening_orders_validate_in_real_fogged_games() {
        for seed in [8140001, 8140002] {
            let mut setup = legatum_core::GameSetup::all_ai(seed);
            setup.barbarians = true;
            let mut s = legatum_core::new_game(&setup);
            let mut brains: Vec<_> = s
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    OpeningBrain::new(
                        ScriptedBrain::new(p.faction, seed, p.id),
                        OpeningPolicy::builtin(Opening::ALL[i % Opening::ALL.len()]),
                    )
                })
                .collect();
            while !s.finished && s.turn <= 120 {
                let mut orders = BTreeMap::new();
                for p in s.players.iter().filter(|p| p.alive) {
                    let v = legatum_core::player_view(&s, p.id);
                    let os = brains[p.id.0 as usize].orders(&v);
                    let errors = legatum_core::validate_orders(&s, p.id, &os);
                    assert!(
                        errors.is_empty(),
                        "seed {seed} turn {} seat {:?}: {:?}\n{os:?}",
                        s.turn,
                        p.id,
                        errors
                    );
                    orders.insert(p.id, os);
                }
                s = legatum_core::resolve_turn(&s, &orders).0;
            }
        }
    }

    #[test]
    fn default_orders_are_exactly_the_existing_brain() {
        for difficulty in [
            legatum_core::AiDifficulty::Easy,
            legatum_core::AiDifficulty::Normal,
            legatum_core::AiDifficulty::Hard,
        ] {
            let s = legatum_core::new_game(&legatum_core::GameSetup::all_ai(77));
            for player in &s.players {
                let v = legatum_core::player_view(&s, player.id);
                let make =
                    || ScriptedBrain::with_difficulty(player.faction, 8, player.id, difficulty);
                assert_eq!(
                    make().orders(&v),
                    OpeningBrain::new(make(), OpeningPolicy::builtin(Opening::Default)).orders(&v)
                );
            }
        }
    }

    #[test]
    fn opening_settler_bypasses_time_discount_and_barbarian_emergency_cancellation() {
        let home = Hex::new(5, 5);
        let p = OpeningPolicy::builtin(Opening::GreedyExpand);
        let mut v = ViewBuilder::new(24, 18)
            .city(1, ME, home, 3)
            .unit(90, PlayerId::BARBARIAN, UnitKind::Archer, Hex::new(7, 5))
            .build();
        let d = v.cities[0].detail.as_mut().unwrap();
        d.options
            .iter_mut()
            .find(|x| x.item == unit(UnitKind::Settler))
            .unwrap()
            .turns = 150;
        let mut plan = planner(&v, &p);
        plan.plan_cities();
        assert!(
            plan.orders.contains(&Order::SetProduction {
                city: CityId(1),
                item: unit(UnitKind::Settler)
            }),
            "{:?}",
            plan.orders
        );
        v.cities[0].detail.as_mut().unwrap().producing = Some(unit(UnitKind::Settler));
        v.cities[0].detail.as_mut().unwrap().turns_to_complete = Some(50);
        let mut plan = planner(&v, &p);
        plan.plan_cities();
        assert!(
            !plan
                .orders
                .iter()
                .any(|x| matches!(x, Order::SetProduction { .. })),
            "{:?}",
            plan.orders
        );
        let mut stock = Planner::new(&v, base().knobs, Faction::Switzerland, 7);
        stock.plan_cities();
        assert!(
            stock
                .orders
                .iter()
                .any(|x| matches!(x, Order::SetProduction { .. })),
            "regression scenario must trigger the old bug"
        );
    }

    #[test]
    fn settler_timing_genes_gate_build_list_entries() {
        let mut policy = OpeningPolicy::builtin(Opening::GreedyExpand);
        policy.numbers.settler_start = 60;
        let mut v = ViewBuilder::new(24, 18)
            .city(1, ME, Hex::new(5, 5), 3)
            .build();
        v.turn = 59;
        let mut p = planner(&v, &policy);
        p.plan_cities();
        let settler = Order::SetProduction {
            city: CityId(1),
            item: unit(UnitKind::Settler),
        };
        assert!(!p.orders.contains(&settler));
        v.turn = 60;
        let mut p = planner(&v, &policy);
        p.plan_cities();
        assert!(p.orders.contains(&settler));
    }

    #[test]
    fn unknown_or_fogged_capitals_never_count_as_empty_armies() {
        let mut v = ViewBuilder::new(24, 18)
            .city(1, ME, Hex::new(4, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(4, 5))
            .build();
        let p = OpeningPolicy::builtin(Opening::Rush);
        assert_eq!(weak_neighbor(&v, &p), None);
        v = ViewBuilder { view: v }
            .city(2, ENEMY, Hex::new(12, 5), 3)
            .build();
        assert_eq!(weak_neighbor(&v, &p), Some(v.team_of(ENEMY)));
        v.cities[1].visible = false;
        assert_eq!(weak_neighbor(&v, &p), None);
        v.cities[1].visible = true;
        v = ViewBuilder { view: v }
            .unit(2, ENEMY, UnitKind::Pikeman, Hex::new(12, 5))
            .build();
        assert_eq!(weak_neighbor(&v, &p), None);
    }

    fn peace(v: &mut PlayerView) {
        use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
        let against = v.team_of(ENEMY);
        v.diplomacy.legacy = false;
        v.diplomacy.relations = vec![Relation {
            a: v.team,
            b: against,
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 0,
            security: None,
            war_id: None,
            declarers: vec![],
        }];
        v.diplomacy.actions = vec![DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        }];
    }

    fn rival_target_view() -> PlayerView {
        ViewBuilder::new(32, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, Hex::new(10, 5), 3)
            .city(3, PlayerId(3), Hex::new(16, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(6, 5))
            .build()
    }

    #[test]
    fn assessed_opening_skips_nearest_banned_rival_without_primitives() {
        let policy = OpeningPolicy::builtin(Opening::Rush);
        let mut v = rival_target_view();
        v.cities
            .iter_mut()
            .find(|c| c.id == CityId(2))
            .unwrap()
            .walls = true;
        // The old, unassessed policy still nominates the nearest weak army.
        assert_eq!(weak_neighbor(&v, &policy), Some(v.team_of(ENEMY)));
        let mut model = crate::opponents::OpponentAssessments::default();
        model.update(&v);
        let mut p = planner(&v, &policy);
        p.opponents = Some(&model);
        assert!(p.primitives.is_none());
        p.plan_opening_war();
        assert_eq!(p.strike_target, Some(v.team_of(PlayerId(3))));
        assert_eq!(p.strike_city, Some(CityId(3)));
    }

    #[test]
    fn assessed_opening_retargets_new_walls_before_the_next_model_refresh() {
        let policy = OpeningPolicy::builtin(Opening::Rush);
        let mut v = rival_target_view();
        let mut model = crate::opponents::OpponentAssessments::default();
        model.update(&v);
        let mut p = planner(&v, &policy);
        p.opponents = Some(&model);
        p.plan_opening_war();
        assert_eq!(p.strike_city, Some(CityId(2)));
        let retained_team = p.strike_target;
        v.turn += 1;
        v.cities
            .iter_mut()
            .find(|c| c.id == CityId(2))
            .unwrap()
            .walls = true;
        assert!(
            !model.update(&v),
            "the strategic snapshot is intentionally cached"
        );
        for retained_city in [Some(CityId(2)), None] {
            let mut p = planner(&v, &policy);
            p.opponents = Some(&model);
            p.strike_target = retained_team;
            p.strike_city = retained_city;
            p.plan_opening_war();
            assert_eq!(p.strike_target, Some(v.team_of(PlayerId(3))));
            assert_eq!(p.strike_city, Some(CityId(3)));
        }
    }

    #[test]
    fn primitives_cannot_bypass_any_openings_local_force_gate() {
        let target = Hex::new(15, 5);
        let mut v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, target, 2)
            .unit(1, ME, UnitKind::Spearman, Hex::new(13, 5))
            .unit(2, ME, UnitKind::Spearman, Hex::new(13, 6))
            .unit(3, ME, UnitKind::Archer, Hex::new(14, 4))
            .unit(4, ME, UnitKind::Spearman, Hex::new(5, 5))
            .build();
        peace(&mut v);
        let declare = v.diplomacy.actions[0].order.clone();
        let memory = PrimitiveMemory::default();
        for opening in Opening::ALL.into_iter().filter(|o| *o != Opening::Default) {
            let mut policy = OpeningPolicy::builtin(opening);
            policy.numbers.war_army = 4;
            for (position, hp, ready) in [
                (Hex::new(5, 5), 100, false),
                (Hex::new(14, 6), 30, false),
                (Hex::new(14, 6), 100, true),
            ] {
                v.units[3].pos = position;
                v.units[3].hp = hp;
                let mut p = planner(&v, &policy);
                p.knobs.aggression = 100;
                p.primitives = Some(PrimitivePlan {
                    params: &policy.primitives,
                    memory: &memory,
                });
                p.plan_primitive_war();
                assert_eq!(p.strike_target, Some(v.team_of(ENEMY)));
                assert!(
                    !p.orders.contains(&declare),
                    "primitives select, openings declare"
                );
                p.plan_opening_war();
                assert_eq!(p.orders.contains(&declare), ready, "{opening:?}");
            }
        }
    }

    #[test]
    fn rush_stages_at_closed_borders_before_declaring() {
        let target = Hex::new(15, 5);
        let mut v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, target, 2)
            .unit(1, ME, UnitKind::Spearman, Hex::new(6, 5))
            .unit(2, ME, UnitKind::Spearman, Hex::new(6, 6))
            .build();
        peace(&mut v);
        for pos in target.within(2) {
            let i = v.index(pos).unwrap();
            v.tiles[i].owner = Some(ENEMY);
        }
        let policy = OpeningPolicy::builtin(Opening::Rush);
        let mut p = planner(&v, &policy);
        assert!(!p.ctx.reachable(v.units[0].pos, target));
        assert!(p.opening_reachable(v.units[0].pos, target));
        p.plan_opening_war();
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::DeclareWar { .. }))
        );
        p.plan_military();
        let destinations: Vec<_> = p
            .orders
            .iter()
            .filter_map(|o| match o {
                Order::MoveUnit { to, .. } => Some(*to),
                _ => None,
            })
            .collect();
        assert_eq!(destinations.len(), 2, "{:?}", p.orders);
        for to in destinations {
            assert!(to.distance(target) <= 3);
            assert!(v.passage_allowed(
                v.me,
                legatum_core::UnitKind::Spearman,
                v.tile(to).unwrap().owner
            ));
        }
    }

    #[test]
    fn rush_minimum_capital_radius_declares_from_legal_staging_tiles() {
        let target = Hex::new(15, 5);
        let mut v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, target, 2)
            .unit(1, ME, UnitKind::Spearman, Hex::new(11, 5))
            .unit(2, ME, UnitKind::Spearman, Hex::new(11, 6))
            .unit(3, ME, UnitKind::Spearman, Hex::new(11, 7))
            .unit(4, ME, UnitKind::Archer, Hex::new(11, 8))
            .unit(5, ME, UnitKind::Archer, Hex::new(10, 9))
            .build();
        peace(&mut v);
        for pos in target.within(legatum_core::data::CITY_WORK_RADIUS) {
            let i = v.index(pos).unwrap();
            v.tiles[i].owner = Some(ENEMY);
        }
        let mut policy = OpeningPolicy::builtin(Opening::Rush);
        policy.numbers.capital_radius = 1;
        let declare = v.diplomacy.actions[0].order.clone();
        let mut p = planner(&v, &policy);
        p.plan_opening_war();
        assert!(
            !p.orders.contains(&declare),
            "the fifth troop is still too far away"
        );

        v.units[4].pos = Hex::new(11, 9);
        let mut p = planner(&v, &policy);
        for u in &v.units {
            assert_eq!(u.pos.distance(target), 4);
            assert!(v.passage_allowed(
                ME,
                legatum_core::UnitKind::Spearman,
                v.tile(u.pos).unwrap().owner
            ));
            assert!(
                !p.ctx.reachable(u.pos, target),
                "closed borders block the city"
            );
            assert!(p.opening_reachable(u.pos, target));
        }
        p.plan_opening_war();
        assert!(
            p.orders.contains(&declare),
            "a legally staged army must be ready"
        );
    }

    #[test]
    fn rush_cityless_settler_prefers_nearby_sites_but_can_travel_farther() {
        use crate::fixture::MATE;
        use legatum_core::{Terrain, settlement::settlement_candidates};

        let start = Hex::new(8, 8);
        let nearby = Hex::new(12, 8);
        let policy = OpeningPolicy::builtin(Opening::Rush);
        for near_available in [true, false] {
            // An allied city rules out founding nearby. Forest makes every legal
            // site take more than two turns unless we open a short plains route.
            let mut v = ViewBuilder::new(30, 20)
                .city(1, MATE, start, 3)
                .patch(start, 8, Terrain::Forest)
                .unit(1, ME, UnitKind::Settler, start)
                .build();
            if near_available {
                for step in 1..=4 {
                    let i = v.index(Hex::new(start.q + step, start.r)).unwrap();
                    v.tiles[i].terrain = Some(Terrain::Plains);
                }
            }
            let candidates = settlement_candidates(&v, &v.units[0], &[]);
            assert!(candidates.iter().any(|s| s.travel_turns.is_some()));
            assert_eq!(
                candidates
                    .iter()
                    .any(|s| s.travel_turns.is_some_and(|t| t <= 2)),
                near_available
            );
            let mut p = planner(&v, &policy);
            assert!(p.ctx.own_cities.is_empty());
            let site = p
                .choose_site(&v.units[0])
                .expect("safe founding land exists");
            assert!(legatum_core::settlement::known_legal_site(&v, site));
            if near_available {
                assert_eq!(site, nearby);
            } else {
                assert!(candidates.iter().any(|s| {
                    s.pos == site && s.travel_turns.is_some_and(|t| t > 2) && s.threats == 0
                }));
            }
            p.plan_settlers();
            assert!(
                p.orders.iter().any(|o| matches!(
                    o,
                    Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                        if *unit == UnitId(1) && *to != start
                )),
                "{:?}",
                p.orders
            );
        }
    }

    #[test]
    fn a_scout_at_home_does_not_recall_the_rush_army() {
        let v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, Hex::new(15, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(12, 5))
            .unit(2, ME, UnitKind::Spearman, Hex::new(12, 6))
            .unit(3, ME, UnitKind::Archer, Hex::new(13, 4))
            .unit(4, ME, UnitKind::Archer, Hex::new(14, 4))
            .unit(9, ME, UnitKind::Spearman, Hex::new(13, 5))
            .unit(5, ENEMY, UnitKind::Scout, Hex::new(6, 5))
            .build();
        let mut b = OpeningBrain::new(base(), OpeningPolicy::builtin(Opening::Rush));
        b.observe(&v);
        assert_eq!(b.phase_for(&v), Phase::Conquer);
        let v = ViewBuilder { view: v }
            .unit(6, ENEMY, UnitKind::Spearman, Hex::new(7, 5))
            .build();
        assert_eq!(
            b.phase_for(&v),
            Phase::Conquer,
            "the capital can hold one raider"
        );
        let v = ViewBuilder { view: v }
            .unit(7, ENEMY, UnitKind::Pikeman, Hex::new(7, 4))
            .unit(8, ENEMY, UnitKind::Pikeman, Hex::new(6, 6))
            .build();
        assert_eq!(b.phase_for(&v), Phase::Defend);
    }

    #[test]
    fn rush_does_not_accept_alliances_before_finding_a_city() {
        use legatum_core::diplomacy::{DiplomaticAction, PeaceProposal, ProposalId, TreatyKind};
        let mut v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .gold(500)
            .build();
        v.turn = 21;
        v.economy.gold_per_turn.total = 5;
        // A third team near victory makes the Alliance worth its upkeep to an ordinary planner.
        v.standings = [(0, 0), (2, 0), (3, 8)]
            .map(|(team, vp)| legatum_core::view::TeamStanding {
                team: TeamId(team),
                members: vec![PlayerId(team)],
                vp,
                target: 10,
                victory_hold: None,
            })
            .to_vec()
            .into();
        peace(&mut v);
        let accept = Order::RespondToProposal {
            proposal: ProposalId(1),
            accept: true,
            explanation: None,
        };
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: accept.clone(),
            effective_turn: 20,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        let (from, to) = (v.team_of(ENEMY), v.team);
        v.diplomacy.proposals.push(PeaceProposal {
            explanation: None,
            id: ProposalId(1),
            kind: TreatyKind::Alliance,
            from,
            to,
            proposer: ENEMY,
            created_turn: 19,
            respond_by: 21,
            relation_version: 0,
            consents: Vec::new(),
            needs: vec![to],
            required_teams: Vec::new(),
        });
        let mut ordinary = Planner::new(&v, base().knobs, Faction::Switzerland, 7);
        ordinary.plan_diplomacy();
        // Alliance answers carry an explanation; compare the action itself.
        let accepts = |orders: &[Order]| {
            orders
                .iter()
                .any(|o| legatum_core::diplomacy::same_action(o, &accept))
        };
        assert!(
            accepts(&ordinary.orders),
            "fixture must exercise normal acceptance"
        );
        let mut b = OpeningBrain::new(base(), OpeningPolicy::builtin(Opening::Rush));
        assert!(!accepts(&b.orders(&v)));
    }

    #[test]
    fn rush_clock_starts_at_army_readiness_and_capture_releases_expansion() {
        let mut v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(5, 5))
            .build();
        let mut policy = OpeningPolicy::builtin(Opening::Rush);
        policy.numbers.war_army = 4;
        policy.numbers.rush_timeout = 60;
        let mut b = OpeningBrain::new(base(), policy);
        v.turn = 100;
        b.observe(&v);
        assert_eq!(b.phase_for(&v), Phase::Opening, "building is not failure");
        v = ViewBuilder { view: v }
            .city(2, ENEMY, Hex::new(15, 5), 3)
            .unit(3, ME, UnitKind::Spearman, Hex::new(12, 4))
            .unit(4, ME, UnitKind::Spearman, Hex::new(13, 4))
            .unit(5, ME, UnitKind::Archer, Hex::new(14, 4))
            .build();
        v.turn = 101;
        b.observe(&v);
        assert_eq!(b.phase_for(&v), Phase::Conquer, "one city is enough");
        b.phase = Phase::Conquer;
        v.turn = 160;
        b.observe(&v);
        assert_eq!(b.phase_for(&v), Phase::Conquer, "full grace");
        b.losses = 1;
        v.units.retain(|u| u.id != UnitId(5));
        b.observe(&v);
        assert_eq!(
            b.phase_for(&v),
            Phase::Conquer,
            "one casualty is not defeat"
        );
        v.turn = 161;
        assert_eq!(b.phase_for(&v), Phase::Expand);
        assert!(b.abandoned_rush);
        // A successful rush transitions immediately, even if its build list is unfinished.
        b.abandoned_rush = false;
        v.cities[1].owner = ME;
        b.observe(&v);
        assert!(b.captured);
        assert_eq!(b.phase_for(&v), Phase::Expand);
    }

    #[test]
    fn rush_recovers_from_real_losses_and_defends_against_visible_threats() {
        let mut v = ViewBuilder::new(30, 20)
            .city(1, ME, Hex::new(5, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(5, 5))
            .build();
        let mut b = OpeningBrain::new(base(), OpeningPolicy::builtin(Opening::Rush));
        b.army_ready_turn = Some(1);
        b.observe(&v);
        b.losses = b.policy.numbers.losses_limit + 1;
        assert_eq!(b.phase_for(&v), Phase::TurtleTech);
        b.phase = Phase::TurtleTech;
        v = ViewBuilder { view: v }
            .unit(2, ENEMY, UnitKind::Spearman, Hex::new(7, 5))
            .build();
        assert_eq!(b.phase_for(&v), Phase::Defend);
    }

    #[test]
    fn war_trigger_and_gold_policy_use_legal_view_quotes() {
        let mut v = ViewBuilder::new(28, 20)
            .known_techs(&TechId::ALL)
            .city(1, ME, Hex::new(5, 5), 3)
            .city(2, ENEMY, Hex::new(13, 5), 3)
            .unit(1, ME, UnitKind::Spearman, Hex::new(11, 5))
            .gold(1000)
            .build();
        let mut policy = OpeningPolicy::builtin(Opening::Rush);
        policy.numbers.weak_percent = 0;
        policy.numbers.war_army = 1;
        policy.numbers.war_deadline = 20;
        let order = Order::DeclareWar {
            against: v.team_of(ENEMY),
            breach: false,
        };
        peace(&mut v);
        let mut p = planner(&v, &policy);
        p.plan_opening_war();
        assert!(p.orders.contains(&order));
        v.turn = 21;
        let mut p = planner(&v, &policy);
        p.plan_opening_war();
        assert!(!p.orders.contains(&order));
        policy.opening = Opening::Economic;
        policy.burst_buy = building(BuildingKind::Market);
        v.turn = 1;
        let opt = v.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .iter_mut()
            .find(|o| o.item == policy.burst_buy)
            .unwrap();
        opt.buy_block = None;
        opt.buy_cost = Some(opt.cost * 2);
        let quoted = opt.buy_cost.unwrap();
        policy.numbers.gold_save = quoted + 10;
        v.economy.gold = quoted;
        let mut p = planner(&v, &policy);
        p.plan_opening_buy();
        assert!(p.orders.is_empty());
        v.economy.gold = quoted + 10;
        let mut p = planner(&v, &policy);
        p.plan_opening_buy();
        assert!(p.orders.contains(&Order::BuyItem {
            city: CityId(1),
            item: policy.burst_buy
        }));
        assert_eq!(p.gold_left, 10);
    }
}
