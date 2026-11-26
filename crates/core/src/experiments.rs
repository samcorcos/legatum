//! Live design experiments. Default rules live in core data and economy.
//! Save files and order tapes carry the switches; no process-global rule overrides.
use crate::types::Faction;
use serde::{Deserialize, Serialize};

/// Whether a faction rule helps its owner or costs it something, for the ablation report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleKind {
    Strength,
    Drawback,
}

macro_rules! faction_rules {
    ($( $variant:ident $(= $index:literal)? => ($name:literal, $faction:ident, $kind:ident) ),+ $(,)?) => {
        /// One distinct faction strength, drawback or goal. `RuleExperiments::faction_rules_off`
        /// switches a rule off for ablation: the rule stops applying to its faction's empires and
        /// the AI plans without it. The names are the snake_case words of the `--rule-off` flag.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum FactionRule {
            $($variant $(= $index)?),+
        }

        impl FactionRule {
            /// Every rule, in declaration order.
            pub const ALL: &'static [FactionRule] = &[$(FactionRule::$variant),+];

            /// The snake_case name used by the CLI, saves and reports.
            pub fn name(self) -> &'static str {
                match self {
                    $(FactionRule::$variant => $name),+
                }
            }

            /// The faction whose rule this is.
            pub fn faction(self) -> Faction {
                match self {
                    $(FactionRule::$variant => Faction::$faction),+
                }
            }

            /// Whether switching the rule off should lower (strength) or raise (drawback) the
            /// faction's win share.
            pub fn kind(self) -> RuleKind {
                match self {
                    $(FactionRule::$variant => RuleKind::$kind),+
                }
            }
        }
    };
}

faction_rules! {
    SwissTolls => ("swiss_tolls", Switzerland, Strength),
    SwissInterest => ("swiss_interest", Switzerland, Strength),
    SwissCheapBuys => ("swiss_cheap_buys", Switzerland, Strength),
    // Keep bit 3 vacant: retired Swiss world-wonder buying.
    SwissRedoubt = 4 => ("swiss_redoubt", Switzerland, Strength),
    SwissUnitUpkeep => ("swiss_unit_upkeep", Switzerland, Drawback),
    SwissCaptureBounty => ("swiss_capture_bounty", Switzerland, Drawback),
    SwissGoal => ("swiss_goal", Switzerland, Strength),
    IndiaFood => ("india_food", India, Strength),
    IndiaNetwork => ("india_network", India, Strength),
    IndiaCloseCities => ("india_close_cities", India, Strength),
    IndiaCorruption => ("india_corruption", India, Drawback),
    IndiaRemittance => ("india_remittance", India, Drawback),
    IndiaGoal => ("india_goal", India, Strength),
    ByzantiumCompanies => ("byzantium_companies", Byzantium, Strength),
    ByzantiumLevy => ("byzantium_levy", Byzantium, Drawback),
    ByzantiumTagmatic => ("byzantium_tagmatic", Byzantium, Strength),
    // Keep bit 17 vacant: retired Byzantium Scriptorium occupied it in saved rule masks.
    ByzantiumGoal = 18 => ("byzantium_goal", Byzantium, Strength),
    GermanyRuhrProduction => ("germany_ruhr_production", Germany, Strength),
    GermanyIndustryFood => ("germany_industry_food", Germany, Drawback),
    GermanyDoubleSupply => ("germany_double_supply", Germany, Strength),
    // Keep bit 22 vacant: retired German new-building discount.
    GermanyIndustryProjects = 23 => ("germany_industry_projects", Germany, Strength),
    GermanyGoal => ("germany_goal", Germany, Strength),
    // Keep bit 25 vacant: retired Celtic Ambusher.
    CeltsSack = 26 => ("celts_sack", Celts, Strength),
    CeltsBarbarianPeace => ("celts_barbarian_peace", Celts, Strength),
    CeltsNoKeep => ("celts_no_keep", Celts, Drawback),
    CeltsGoal => ("celts_goal", Celts, Strength),
    MongolsMoves => ("mongols_moves", Mongols, Strength),
    // Keep bit 31 vacant: retired Mongol Peace passage.
    MongolsCommerce = 32 => ("mongols_commerce", Mongols, Strength),
    MongolsKeshig => ("mongols_keshig", Mongols, Strength),
    MongolsNoTerrain => ("mongols_no_terrain", Mongols, Drawback),
    MongolsNoWalls => ("mongols_no_walls", Mongols, Drawback),
    MongolsGoal => ("mongols_goal", Mongols, Strength),
    SpartaVeterans => ("sparta_veterans", Sparta, Strength),
    SpartaNoWounds => ("sparta_no_wounds", Sparta, Strength),
    SpartaKillHeal => ("sparta_kill_heal", Sparta, Strength),
    SpartaPhalanx => ("sparta_phalanx", Sparta, Strength),
    SpartaPremium => ("sparta_premium", Sparta, Drawback),
    // Keep bit 42 vacant: retired Spartan slow healing.
    // Keep bit 43 vacant: retired Spartan desertion.
    SpartaGoal => ("sparta_goal", Sparta, Strength),
    AthensLyceum => ("athens_lyceum", Athens, Strength),
    AthensRuins => ("athens_ruins", Athens, Strength),
    AthensWeakMarkets => ("athens_weak_markets", Athens, Drawback),
    AthensGreeceCaptive => ("athens_greece_captive", Athens, Drawback),
    AthensGoal => ("athens_goal", Athens, Strength),
    AztecsFlowerWars => ("aztecs_flower_wars", Aztecs, Strength),
    AztecsNoHorses => ("aztecs_no_horses", Aztecs, Drawback),
    AztecsGoal => ("aztecs_goal", Aztecs, Strength),
    AztecsEagleWarrior => ("aztecs_eagle_warrior", Aztecs, Strength),
    CeltsGaesatae => ("celts_gaesatae", Celts, Strength),
    AthensPeltast => ("athens_peltast", Athens, Strength),
    IndiaWarElephant => ("india_war_elephant", India, Strength),
    SwissReislaufer => ("swiss_reislaufer", Switzerland, Strength),
    GermanyPanzer => ("germany_panzer", Germany, Strength),
    EgyptConstruction => ("egypt_construction", Egypt, Strength),
    // Keep bit 58 vacant: retired Egyptian Medjay (replaced by the Chariot Archer, rules 59).
    EgyptRoyalGranary = 59 => ("egypt_royal_granary", Egypt, Strength),
    RomeVeteranCities => ("rome_veteran_cities", Rome, Strength),
    RomeNoSettlers => ("rome_no_settlers", Rome, Drawback),
    RomeLegionary => ("rome_legionary", Rome, Strength),
    RomeCastrum => ("rome_castrum", Rome, Strength),
    // The Nile (designer, 2026-11-21): doubled Food, no Production, surplus Food builds and a
    // Grow project grows. Switching it off restores the earlier rule (`EgyptConstruction`).
    EgyptNile => ("egypt_nile", Egypt, Strength),
    RomeGoal => ("rome_goal", Rome, Strength),
    EgyptGoal => ("egypt_goal", Egypt, Strength),
    // The Chariot Archer (Sam, 2026-11-25): Egypt's Archer, 3 moves, +40% shot against the melee
    // line, and Forest or Hills without a road take its whole turn to enter.
    EgyptChariotArcher => ("egypt_chariot_archer", Egypt, Strength),
    // Kyivan Rus' (`rus`): units trained in Rus' cities are an ally's special version, camps
    // that raid Rus' cities raid again sooner, the Druzhina replaces the Knight, the Lavra (the
    // Library) adds Science per Alliance, and the goal holds three Alliances at once.
    RusMarriage => ("rus_marriage", Rus, Strength),
    RusSteppe => ("rus_steppe", Rus, Drawback),
    RusDruzhina => ("rus_druzhina", Rus, Strength),
    RusLavra => ("rus_lavra", Rus, Strength),
    RusGoal => ("rus_goal", Rus, Strength),
}

impl FactionRule {
    /// Parse a snake_case rule name; an unknown name lists the valid ones.
    pub fn parse(name: &str) -> Result<FactionRule, String> {
        Self::ALL
            .iter()
            .copied()
            .find(|r| r.name() == name)
            .ok_or_else(|| {
                format!(
                    "unknown faction rule: {name} (valid: {})",
                    Self::ALL
                        .iter()
                        .map(|r| r.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }

    /// Every rule of one faction, in declaration order.
    pub fn of(faction: Faction) -> impl Iterator<Item = FactionRule> {
        Self::ALL
            .iter()
            .copied()
            .filter(move |r| r.faction() == faction)
    }
}

impl std::fmt::Display for FactionRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Serialize for FactionRule {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for FactionRule {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        FactionRule::parse(&name).map_err(serde::de::Error::custom)
    }
}

/// An ordered set of `FactionRule`s, as a bit mask so `RuleExperiments` stays `Copy`. It behaves
/// as a `BTreeSet<FactionRule>`: iteration is in declaration order, never by hash or address, and
/// it is saved as a sorted list of rule names (absent when empty).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FactionRuleSet(u128);

impl FactionRuleSet {
    pub const EMPTY: FactionRuleSet = FactionRuleSet(0);

    const fn bit(rule: FactionRule) -> u128 {
        1u128 << (rule as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, rule: FactionRule) -> bool {
        self.0 & Self::bit(rule) != 0
    }

    pub fn insert(&mut self, rule: FactionRule) {
        self.0 |= Self::bit(rule);
    }

    pub fn remove(&mut self, rule: FactionRule) {
        self.0 &= !Self::bit(rule);
    }

    pub fn iter(self) -> impl Iterator<Item = FactionRule> {
        FactionRule::ALL
            .iter()
            .copied()
            .filter(move |r| self.contains(*r))
    }

    /// Parse a comma-separated list of rule names (the `--rule-off` value).
    pub fn parse_list(list: &str) -> Result<FactionRuleSet, String> {
        let mut set = FactionRuleSet::EMPTY;
        for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            set.insert(FactionRule::parse(name)?);
        }
        Ok(set)
    }
}

impl std::fmt::Debug for FactionRuleSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl FromIterator<FactionRule> for FactionRuleSet {
    fn from_iter<I: IntoIterator<Item = FactionRule>>(iter: I) -> Self {
        let mut set = FactionRuleSet::EMPTY;
        for rule in iter {
            set.insert(rule);
        }
        set
    }
}

impl Serialize for FactionRuleSet {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for FactionRuleSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut set = Self::EMPTY;
        for name in Vec::<String>::deserialize(d)? {
            // Old experimental saves may name rules removed from the default game. Their bits
            // stay vacant, so loading those saves simply drops the obsolete off switches.
            if matches!(
                name.as_str(),
                "swiss_wonder_buys"
                    | "celts_ambusher"
                    | "mongols_peace_passage"
                    | "sparta_slow_heal"
            ) {
                continue;
            }
            set.insert(FactionRule::parse(&name).map_err(serde::de::Error::custom)?);
        }
        Ok(set)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuleExperiments {
    /// Ablation: faction rules switched off (`--rule-off`). Each stops applying to its faction's
    /// empires, in the engine and in what the AI reads; empty in every app game.
    #[serde(skip_serializing_if = "FactionRuleSet::is_empty")]
    pub faction_rules_off: FactionRuleSet,
    /// Settler escalation is +150 Production per trained Settler, instead of +50.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub settler_escalation: bool,
    /// +5% research cost per owned city beyond two, rounded up.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub city_research: bool,
    /// Each owned city costs max(city count - 2, 0) Gold per turn.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub city_upkeep: bool,
    /// A city cannot buy a military unit while an enemy military unit at War stands within two
    /// tiles of it (Sam, 2026-11-22, via the Fable review; judged on his Swiss and Mongol games).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub no_purchase_under_threat: bool,
    /// Coalition pressure adds 10 points per city per permanent-team member beyond two.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub expansion_coalitions: bool,
    /// Unwalled cities below population four lose 4 strength per missing population.
    pub frontier_cities: bool,
    /// Capturing an unwalled Market-family building dismantles it for 2x nominal cost.
    pub market_plunder: bool,
    /// Walls add eight strength on top of their existing faction-specific contribution.
    pub wall_masonry: bool,
    /// Experimental listed Production price of Walls (default 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walls_cost: Option<u16>,
    /// Hard AI seats play Hard v2 (league-tuned openings, `legatum_ai::hard_v2`) instead of the
    /// scripted Hard. An AI choice, not a rule; it lives here so saves and tapes reload the
    /// same brains. Off in every app game (validation: harder for AIs, easier for humans).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub hard_v2_ai: bool,
    /// Extra starting Settlers per seat, placed around the already-founded capital.
    #[serde(skip_serializing_if = "is_zero")]
    pub start_settlers: u8,
    /// V10 candidate: Gold per active remote Market-family city (default: two).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mongols_commerce_gold: Option<u16>,
    /// Experimental research price as a percent of normal cost, after diffusion and city research.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tech_cost_percent: Option<u8>,
    /// Every team starts having met every other: for fixtures that test treaties without
    /// exploring (a team's first contact is otherwise recorded as units and cities see each other).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub start_met: bool,
    /// Decadence is proportional to the lead (user, 2026-10-17): the rate is `L * c / 100` percent
    /// for a lead `L`, and this is `c` in percent of the lead (default zero).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decadence_slope: Option<u16>,
    /// Overrides the solo VP target (`solo_vp_target`); a team's target is this times its members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vp_target: Option<u8>,
    /// World Age count-card thresholds (Patrons, Prosperity, Frontier), in percent of the counts in
    /// `objectives::age_pool` (default `DEFAULT_OBJECTIVE_SCALE`; 100 restores the counts as
    /// written). Applied after the lobby-size scaling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective_scale: Option<u16>,
    /// The Nile: Egypt's Food in percent of normal (default `EGYPT_NILE_FOOD_PERCENT`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egypt_nile_food: Option<u16>,
    /// The Nile: build progress per 100 surplus Food (default `EGYPT_NILE_BUILD_PERCENT`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egypt_nile_build: Option<u16>,
    /// A Gold project's share of city Production for every empire but Germany, in percent
    /// (default `data::PROJECT_GOLD_PERCENT`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_gold: Option<u16>,
    /// A Science project's share, in percent (default `data::PROJECT_SCIENCE_PERCENT`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_science: Option<u16>,
    /// A Food project's share, in percent (default `data::PROJECT_FOOD_PERCENT`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_food: Option<u16>,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

/// Turns in the trailing window behind an empire's Power (user, 2026-10-18: 50, so a conquest raises
/// the rate gradually instead of at once; it was 30). Standard is the only pace, so it is not scaled.
/// The app reads it from `rules.generated.json` (`decadence.powerWindowTurns`).
pub const POWER_WINDOW_TURNS: u8 = 50;
/// Decadence, in percent of the lead: an empire `L` percent ahead of the mean empire pays `L * c / 100`
/// percent of every income (user, 2026-10-17: proportional to how far ahead each empire is; the mean
/// since 2026-10-18). Zero disables the rule; positive values have no maximum.
pub const DEFAULT_DECADENCE_SLOPE: u16 = 0;
/// World Age count cards use the counts written in `objectives::age_pool`, so 100 percent.
/// `objective-scale` can vary the counts in VP pacing batches.
pub const DEFAULT_OBJECTIVE_SCALE: u16 = 100;

/// The Nile doubles Egypt's Food (designer, 2026-11-21: "double food yield, no production").
pub const EGYPT_NILE_FOOD_PERCENT: u16 = 200;
/// Build progress per 100 surplus Food: one per Food (designer, 2026-11-21, after the view fix:
/// 17.5% of eight-seat Hard games against an even 12.5%; 85 gave 10.5%;
/// `docs/sim-results/egypt-nile-2026-11-21.md`).
pub const EGYPT_NILE_BUILD_PERCENT: u16 = 100;

impl RuleExperiments {
    /// Whether Egypt plays the Nile rule (the default).
    pub fn egypt_nile(self) -> bool {
        !self.rule_off(FactionRule::EgyptNile)
    }

    /// Whether Egypt plays the earlier Food-funded construction rule: only with the Nile off.
    pub fn egypt_construction(self) -> bool {
        !self.egypt_nile() && !self.rule_off(FactionRule::EgyptConstruction)
    }

    pub fn egypt_nile_food_percent(self) -> i32 {
        i32::from(self.egypt_nile_food.unwrap_or(EGYPT_NILE_FOOD_PERCENT))
    }

    pub fn egypt_nile_build_percent(self) -> i32 {
        i32::from(self.egypt_nile_build.unwrap_or(EGYPT_NILE_BUILD_PERCENT))
    }

    /// The share of city Production (Egypt: of its Food-funded build progress) a project turns
    /// into Gold, Science or Food each turn, in percent. Germany's Gold and Science are better (its faction
    /// strength); switched off, Germany falls back to everyone's rate.
    pub fn project_percent(self, faction: crate::Faction, project: crate::IndustryProject) -> i32 {
        use crate::IndustryProject::{Food, Gold, Grow, Science};
        let german = faction == crate::Faction::Germany
            && !self.rule_off(FactionRule::GermanyIndustryProjects);
        match (project, german) {
            (Gold, true) => crate::data::GERMANY_GOLD_PROJECT_PERCENT,
            (Science, true) => crate::data::GERMANY_SCIENCE_PROJECT_PERCENT,
            (Gold, false) => i32::from(
                self.project_gold
                    .unwrap_or(crate::data::PROJECT_GOLD_PERCENT),
            ),
            (Science, false) => i32::from(
                self.project_science
                    .unwrap_or(crate::data::PROJECT_SCIENCE_PERCENT),
            ),
            (Food, _) => i32::from(
                self.project_food
                    .unwrap_or(crate::data::PROJECT_FOOD_PERCENT),
            ),
            (Grow, _) => 0,
        }
    }

    /// Whether `rule` is switched off.
    pub fn rule_off(self, rule: FactionRule) -> bool {
        self.faction_rules_off.contains(rule)
    }

    /// `data::building_stats_for` with the switched-off faction rules applied.
    pub fn building_stats_for(
        self,
        faction: crate::Faction,
        kind: crate::BuildingKind,
    ) -> crate::data::BuildingStats {
        crate::data::building_stats_for_rules(self.faction_rules_off, faction, kind)
    }

    /// `data::unit_stats_for` with the switched-off faction rules applied.
    pub fn unit_stats_for(
        self,
        faction: crate::Faction,
        kind: crate::UnitKind,
    ) -> crate::data::UnitStats {
        crate::data::unit_stats_for_rules(self.faction_rules_off, faction, kind)
    }

    /// `data::unit_moves_for` with the switched-off faction rules applied.
    pub fn unit_moves_for(self, faction: crate::Faction, kind: crate::UnitKind) -> i32 {
        crate::data::unit_moves_for_rules(self.faction_rules_off, faction, kind)
    }

    /// The moves of a unit of `made_by`'s version in `faction`'s hands (`data::unit_stats_made_by_rules`).
    pub fn unit_moves_made_by(
        self,
        faction: crate::Faction,
        made_by: crate::Faction,
        kind: crate::UnitKind,
    ) -> i32 {
        crate::data::unit_stats_made_by_rules(self.faction_rules_off, faction, made_by, kind).moves
    }

    /// `data::tech_card_effects_for` with the switched-off faction rules applied.
    pub fn tech_card_effects_for(
        self,
        faction: crate::Faction,
        tech: crate::TechId,
    ) -> Vec<&'static str> {
        crate::data::tech_card_effects_for_rules(self.faction_rules_off, faction, tech)
    }

    /// `data::min_city_gap` with the switched-off faction rules applied.
    pub fn min_city_gap(self, faction: crate::Faction) -> i32 {
        crate::data::min_city_gap_rules(self.faction_rules_off, faction)
    }

    /// `data::available_to_player` with the switched-off faction rules applied.
    pub fn available_to_player(
        self,
        faction: crate::Faction,
        era: u8,
        item: crate::ProductionItem,
    ) -> bool {
        crate::data::available_to_player_rules(self.faction_rules_off, faction, era, item)
    }

    /// The proportional rate of decadence.
    pub fn decadence_slope(self) -> i32 {
        i32::from(self.decadence_slope.unwrap_or(DEFAULT_DECADENCE_SLOPE))
    }

    /// Decadence, in percent of every income, for a lead over the average empire in percent: zero
    /// at or below the average, then in proportion to the lead, `L * c / 100` rounded to the nearest
    /// percent (`c` is `decadence_slope`). A leader twice as far ahead as the runner-up pays twice
    /// the rate. No maximum. Integer maths.
    pub fn decadence_percent(self, lead_percent: i32) -> i32 {
        let l = i64::from(lead_percent);
        if l <= 0 {
            return 0;
        }
        let c = i64::from(self.decadence_slope());
        ((l * c + 50) / 100).min(i64::from(i32::MAX)) as i32
    }

    /// The solo VP target is the number of eras plus four, capped at 10. It is one above the
    /// permanent VP ceiling until that cap; with nine eras, the cap of 10 binds (eight objectives,
    /// a faction goal and a Great Work could reach 12), as it did with eight. The `vp-target` knob
    /// overrides it.
    pub fn solo_vp_target(self) -> u32 {
        self.vp_target.map_or_else(
            || (u32::from(crate::data::era_count()) + 4).min(10),
            u32::from,
        )
    }

    /// World Age count-card thresholds in percent of the written counts.
    pub fn objective_scale_percent(self) -> u16 {
        self.objective_scale.unwrap_or(DEFAULT_OBJECTIVE_SCALE)
    }

    /// Shared CLI parser; reject misspelled or out-of-range experiments before simulating.
    pub fn apply(&mut self, flag: &str) -> Result<(), String> {
        match flag {
            "settler-escalation" => self.settler_escalation = true,
            "city-research" => self.city_research = true,
            "city-upkeep" => self.city_upkeep = true,
            "expansion-coalitions" => self.expansion_coalitions = true,
            "no-purchase-under-threat" => self.no_purchase_under_threat = true,
            "frontier-cities" => self.frontier_cities = true,
            "market-plunder" => self.market_plunder = true,
            "wall-masonry" => self.wall_masonry = true,
            "hard-v2-ai" => self.hard_v2_ai = true,
            "start-met" => self.start_met = true,
            _ => {
                let (name, value) = flag
                    .split_once('=')
                    .ok_or_else(|| format!("unknown rules experiment: {flag}"))?;
                let value: u16 = value
                    .parse()
                    .map_err(|_| format!("invalid experiment value: {flag}"))?;
                match (name, value) {
                    ("start-settlers", 0..=8) => self.start_settlers = value as u8,
                    ("mongols-commerce-gold", 2..=8) => self.mongols_commerce_gold = Some(value),
                    ("tech-cost-percent", 1..=100) => self.tech_cost_percent = Some(value as u8),
                    ("decadence-slope", 0..=1000) => {
                        self.decadence_slope = (value != DEFAULT_DECADENCE_SLOPE).then_some(value)
                    }
                    ("vp-target", 1..=60) => self.vp_target = Some(value as u8),
                    ("objective-scale", 50..=1000) => {
                        self.objective_scale = (value != DEFAULT_OBJECTIVE_SCALE).then_some(value)
                    }
                    ("walls-cost", 1..=1000) => self.walls_cost = (value != 100).then_some(value),
                    ("egypt-nile-food", 100..=400) => {
                        self.egypt_nile_food = (value != EGYPT_NILE_FOOD_PERCENT).then_some(value)
                    }
                    ("egypt-nile-build", 25..=400) => {
                        self.egypt_nile_build = (value != EGYPT_NILE_BUILD_PERCENT).then_some(value)
                    }
                    ("project-gold", 1..=100) => {
                        self.project_gold =
                            (value != crate::data::PROJECT_GOLD_PERCENT).then_some(value)
                    }
                    ("project-science", 1..=100) => {
                        self.project_science =
                            (value != crate::data::PROJECT_SCIENCE_PERCENT).then_some(value)
                    }
                    ("project-food", 1..=100) => {
                        self.project_food =
                            (value != crate::data::PROJECT_FOOD_PERCENT).then_some(value)
                    }
                    _ => {
                        return Err(format!(
                            "unknown or out-of-range experiment: {flag} (tech costs 1..100, settlers 0..8, commerce 2..8, decadence 0..1000, VP 1..60, objectives 50..1000, walls 1..1000, Egypt Nile Food 100..400, build 25..400, projects 1..100)"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Faction price before trained-Settler escalation.
    /// Great Works keep their standard prices. Optional overrides replace base drawbacks.
    pub fn base_production_cost(self, faction: crate::Faction, item: crate::ProductionItem) -> i32 {
        use crate::ProductionItem;
        match item {
            ProductionItem::Unit { unit } => self.unit_stats_for(faction, unit).cost,
            ProductionItem::Project { .. } => 0,
            ProductionItem::Improvement { improvement, .. } => {
                crate::improvements::base_cost(improvement)
            }
            ProductionItem::Building {
                building: crate::BuildingKind::Walls,
            } => {
                self.building_stats_for(faction, crate::BuildingKind::Walls)
                    .cost
                    + self.walls_cost.map_or(0, |cost| i32::from(cost) - 100)
            }
            ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                self.building_stats_for(faction, building).cost
            }
        }
    }

    pub fn settler_cost_step(self) -> i32 {
        crate::data::SETTLER_COST_STEP + if self.settler_escalation { 100 } else { 0 }
    }

    pub fn city_upkeep(self, cities: i32) -> i32 {
        if self.city_upkeep {
            cities * (cities - 2).max(0)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_settlers_are_deterministic_connected_unstacked_and_do_not_change_existing_ids() {
        use crate::{GameSetup, UnitKind, new_game};
        for seed in [1, 2, 42, u64::MAX] {
            let mut setup = GameSetup::all_ai(seed);
            let original = new_game(&setup);
            for n in [0, 1, 3, 8] {
                setup.rule_experiments.start_settlers = n;
                let s = new_game(&setup);
                assert_eq!(s, new_game(&setup));
                for (id, unit) in &original.units {
                    assert_eq!(s.units[id], *unit);
                }
                let mut occupied = std::collections::BTreeSet::new();
                for u in s.units.values().filter(|u| u.kind == UnitKind::Settler) {
                    assert!(occupied.insert(u.pos));
                }
                for p in &s.players {
                    assert_eq!(
                        s.units_of(p.id)
                            .filter(|u| u.kind == UnitKind::Settler)
                            .count(),
                        usize::from(n)
                    );
                    let start = original.capital_of(p.id).unwrap().pos;
                    for u in s.units_of(p.id) {
                        assert!(
                            crate::path::find_path(&s.map, start, u.pos, |h| s.map.move_cost(h))
                                .is_some()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn opening_flags_validate_and_default_encoding_is_unchanged() {
        let mut r = RuleExperiments::default();
        assert_eq!(
            serde_json::to_string(&r).unwrap(),
            r#"{"frontier_cities":false,"market_plunder":false,"wall_masonry":false}"#
        );
        for flag in ["tech-cost-percent=65", "start-settlers=2"] {
            r.apply(flag).unwrap();
        }
        assert_eq!(
            serde_json::from_str::<RuleExperiments>(&serde_json::to_string(&r).unwrap()).unwrap(),
            r
        );
        for flag in [
            "tech-cost-percent=0",
            "tech-cost-percent=101",
            "start-settlers=9",
            "unknown",
        ] {
            assert!(RuleExperiments::default().apply(flag).is_err(), "{flag}");
        }
    }

    #[test]
    fn removed_opening_flags_no_longer_parse() {
        for flag in ["opening-ramp", "capital-production"] {
            assert!(RuleExperiments::default().apply(flag).is_err(), "{flag}");
        }
    }

    #[test]
    fn a_save_with_the_removed_military_discount_still_loads() {
        let r: RuleExperiments = serde_json::from_str(r#"{"early_military_discount":60}"#).unwrap();
        assert!(r.is_default());
        assert!(
            RuleExperiments::default()
                .apply("early-military-discount=60")
                .is_err()
        );
    }
}
