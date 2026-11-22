//! Goal-driven plans (step 2): one strategic objective at a time, chosen by its effect on
//! this seat's estimated chance to win (`winchance.rs`), kept with commitment and ended only
//! for an explicit, recorded reason. Hard only.
//!
//! Every candidate is priced in the same currency, hundredths of a percentage point of win
//! chance, by projecting its outcomes onto the seat's own public facts and scoring them again:
//! - **Economy / Peace**: no war objective. The reference plan, worth zero. While at war it is
//!   labelled Peace with the enemy whose war costs most; every war the plan doesn't name is
//!   one the AI tries to end.
//! - **Campaign** against one city: capture (a city, its people and land, and any milestone,
//!   all taken from the owner) at the forecast's odds, against expected losses, a failed
//!   siege, retaliation at home, forgone growth, the treasury runway, and the aggressor's
//!   risks when the war is new.
//! - **Contain** the VP leader (pillar 11's rubber band) or the conqueror the threat model
//!   names: priced by the VP it denies them, not by captures, against attrition and
//!   retaliation. A coalition war that can't take a city is still worth fighting when the
//!   denial pays for it.
//! - **Snipe** an exposed rival Settler: a surprise war and a same-turn capture. Worth about
//!   a city for us plus the city it denies them, against retaliation, the aggressor's risk
//!   and the cost of war; the next turn's plan is usually Peace again.
//! - **FieldStrike** a massed army approaching our city: value the forecast field damage and
//!   city loss avoided against casualties, retaliation and the cost of declaring war.
//!
//! The plan persists in `CampaignMemory::plan` (saved with the opponent memory). A challenger
//! replaces it only by a margin; a plan that becomes infeasible ends with its reason. Every
//! switch and every purchase the plan's reserve blocks is logged (`PlanEvent`).

use crate::campaign::{CampaignEnd, CampaignEvent};
use crate::diplomacy::strength;
use crate::opportunity::legal_declaration;
use crate::scripted::Planner;
use crate::winchance::{TeamFacts, WinFacts};
use legatum_core::data;
use legatum_core::diplomacy::RelationState;
use legatum_core::view::{CityView, PlayerView, UnitView};
use legatum_core::{
    AiDifficulty, AttackMode, CityId, Faction, Hex, Order, ProductionItem, TeamId, UnitId,
    UnitKind, forecast_attack_mode,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The containment term the search adds past its rollout horizon (`search_decision_with`).
const LEADER_RACE: &str = "leader's race";
/// Win-chance value (hundredths of a point) of each era we are ahead of a war target, up to
/// two: the target still fields the older units, a unit-technology window.
const ERA_LEAD_VALUE: i32 = 30;
/// The same per era we are behind a campaign target, up to two: it fields better units.
const ERA_DEFICIT_VALUE: i32 = 40;
/// Extra for the freshest window: we were the first empire into the era we are in.
const FIRST_INTO_ERA_VALUE: i32 = 20;
/// Blocked-expense events kept in memory; later ones are only counted.
const BLOCKED_EVENTS_KEPT: u32 = 100;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "plan", rename_all = "snake_case")]
pub enum PlanKind {
    Economy,
    Peace {
        with: TeamId,
    },
    Campaign {
        against: TeamId,
        target: CityId,
    },
    Contain {
        against: TeamId,
    },
    Snipe {
        against: TeamId,
        settler: UnitId,
        by: UnitId,
    },
    FieldStrike {
        against: TeamId,
        target: UnitId,
        by: UnitId,
    },
    /// Hard+Search only (`SearchConfig::holds`): keep a defender in a threatened city, by
    /// moving the nearest unit that can arrive or buying one. Diplomatically like Economy.
    Hold {
        city: CityId,
    },
}

#[cfg(test)]
mod supply_race_cap_tests {
    use super::*;
    use crate::{Brain, Knobs, OpponentMemory, ScriptedBrain};
    use flate2::read::GzDecoder;
    use legatum_core::rng::{Rng, purpose};
    use std::io::Read;

    fn fixture<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
        let mut json = Vec::new();
        GzDecoder::new(bytes).read_to_end(&mut json).unwrap();
        serde_json::from_slice(&json).unwrap()
    }

    #[test]
    fn athens_alesia_supply_cannot_outbid_race_or_denial() {
        let view: PlayerView = fixture(include_bytes!(
            "fixtures/athens-germany-8943/T163-p3.view.json.gz"
        ));
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        planner.difficulty = AiDifficulty::Hard;
        let alesia = |p: &Planner| {
            p.plan_candidates()
                .into_iter()
                .find(|c| {
                    matches!(
                        c.kind,
                        PlanKind::Campaign {
                            target: CityId(25),
                            ..
                        }
                    )
                })
                .unwrap()
        };
        planner.decisions.supply_race_cap = 0;
        let old = alesia(&planner);
        planner.decisions.supply_race_cap = 1;
        let capped = alesia(&planner);
        let term =
            |c: &Candidate, name: &str| c.terms.iter().find(|(label, _)| label == name).unwrap().1;
        let old_supply = term(&old, "supply");
        let new_supply = term(&capped, "supply");
        let race = old
            .terms
            .iter()
            .filter(|(name, _)| {
                name.starts_with("capture at ")
                    || matches!(name.as_str(), "VP hold closeout" | "rival's next objective")
            })
            .map(|(_, value)| *value)
            .max()
            .unwrap();
        assert!(old_supply > race, "{old:?}");
        assert_eq!(new_supply, race.max(0), "{capped:?}");
        assert_eq!(old.value - capped.value, old_supply - new_supply);
    }

    #[test]
    fn athens_replay_keeps_the_german_containment_instead_of_retargeting_alesia() {
        let view: PlayerView = fixture(include_bytes!(
            "fixtures/athens-germany-8943/T163-p3.view.json.gz"
        ));
        let memory: OpponentMemory = fixture(include_bytes!(
            "fixtures/athens-germany-8943/T163-p3.memory.json.gz"
        ));
        let seed = Rng::derive(7150772683161427, &[purpose::AI, view.me.0 as u64]).next_u64();
        let play = |cap| {
            let mut brain =
                ScriptedBrain::with_difficulty(Faction::Athens, seed, view.me, AiDifficulty::Hard);
            brain.restore_opponent_memory(memory.clone());
            brain.decisions.supply_race_cap = cap;
            // `reachable_wars` also keeps this campaign off Alesia; off here so the cap is what is tested.
            brain.decisions.reachable_wars = 0;
            brain.orders(&view);
            brain.campaigns.plan.active.unwrap().kind
        };
        assert!(matches!(
            play(0),
            PlanKind::Campaign {
                target: CityId(25),
                ..
            }
        ));
        assert!(!matches!(
            play(1),
            PlanKind::Campaign {
                target: CityId(25),
                ..
            }
        ));
    }
}

#[cfg(test)]
mod capture_vp_accounting_tests {
    use super::*;
    use crate::Knobs;
    use flate2::read::GzDecoder;
    use serde::de::DeserializeOwned;
    use std::io::Read;

    fn fixture<T: DeserializeOwned>(bytes: &[u8]) -> T {
        let mut json = Vec::new();
        GzDecoder::new(bytes).read_to_end(&mut json).unwrap();
        serde_json::from_slice(&json).unwrap()
    }

    #[test]
    fn athens_germany_capture_does_not_suspend_the_foundry_goal() {
        let view: PlayerView = fixture(include_bytes!("fixtures/athens-1025/T219-p2.view.json.gz"));
        let city = view.cities.iter().find(|c| c.id == CityId(40)).unwrap();
        let goal = view.players[2]
            .vp_sources
            .iter()
            .find(|(source, _)| {
                matches!(
                    source,
                    legatum_core::VpSource::FactionGoal {
                        of: legatum_core::PlayerId(2)
                    }
                )
            })
            .map(|(_, vp)| *vp);
        assert_eq!(goal, Some(2));
        assert!(
            city.detail.is_none(),
            "enemy buildings are private in the actual view"
        );

        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Germany),
            Faction::Germany,
            1,
        );
        p.decisions.capture_vp_accounting = 1;
        assert_eq!(p.factory_goal_capture_delay(city), 0);
        p.decisions.capture_vp_accounting = 0;
        assert_eq!(p.factory_goal_capture_delay(city), 0);
    }
}

#[cfg(test)]
mod late_capture_value_tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::BuildingKind;
    use legatum_core::diplomacy::Relation;
    use legatum_core::landmarks::Landmark;
    use legatum_core::view::TeamStanding;

    fn late_view(celts: bool, work: bool) -> PlayerView {
        let mut view = ViewBuilder::new(30, 18)
            .city(1, ME, Hex::new(5, 8), 5)
            .city(2, ENEMY, Hex::new(17, 8), 4)
            .city(3, ENEMY, Hex::new(20, 8), 3)
            .city(4, ENEMY, Hex::new(23, 8), 3)
            .build();
        view.turn = 400;
        view.diplomacy.legacy = false;
        let enemy_team = view.team_of(ENEMY);
        view.diplomacy.relations.push(Relation {
            a: view.team,
            b: enemy_team,
            state: RelationState::War,
            effective_turn: 200,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(1),
            declarers: vec![enemy_team],
        });
        view.players[ME.0 as usize].cities = 1;
        view.players[ENEMY.0 as usize].cities = 3;
        view.players[3].alive = false;
        view.players[ME.0 as usize].vp = 9;
        view.players[ENEMY.0 as usize].vp = 4;
        view.standings = vec![
            TeamStanding {
                team: view.team,
                members: vec![ME],
                vp: 9,
                target: 10,
                victory_hold: None,
            },
            TeamStanding {
                team: view.team_of(ENEMY),
                members: vec![ENEMY],
                vp: 4,
                target: 10,
                victory_hold: None,
            },
        ]
        .into();
        if celts {
            view.players[ME.0 as usize].faction = Faction::Celts;
        }
        if work {
            let tile = view.index(Hex::new(18, 8)).unwrap();
            view.tiles[tile].owner = Some(ENEMY);
            view.tiles[tile].landmark = Some(Landmark {
                building: BuildingKind::RelayUplink,
                city: CityId(2),
                of: ENEMY,
            });
            view.standings[1].vp = 6;
            view.players[ENEMY.0 as usize].vp = 6;
        }
        view
    }

    fn priced(view: &PlayerView, gene: i32) -> Candidate {
        priced_city(view, gene, CityId(2))
    }

    fn priced_city(view: &PlayerView, gene: i32, target: CityId) -> Candidate {
        let faction = view.faction_of(ME);
        let mut planner = Planner::new(view, crate::Knobs::for_faction(faction), faction, 7);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.late_capture_value = gene;
        let mut base = WinFacts::of(view)
            .with_relative(&planner.decisions)
            .refined();
        // The reported late stall has a saturated eventual-win estimate despite having
        // no winner. Preserve that adverse case in the unit test.
        base.team_mut(view.team).unwrap().economy = 100_000;
        base.team_mut(view.team_of(ENEMY)).unwrap().economy = 100;
        assert!(base.chance() >= 9_800, "{}", base.chance());
        let city = view.cities.iter().find(|c| c.id == target).unwrap();
        planner.price_campaign(
            &base,
            base.chance(),
            city,
            &Forecast {
                turns: 3,
                losses: 0,
                force: Vec::new(),
                march: 0,
            },
        )
    }

    #[test]
    fn a_three_city_elimination_path_has_value_after_win_chance_saturates() {
        // City 2 is the rival's original capital: taking it at nine VP scores the tenth
        // point (capital VP) and starts a hold, so the elimination path is priced on its
        // second city instead.
        let view = late_view(true, false);
        let old = priced_city(&view, 0, CityId(3));
        let new = priced_city(&view, 100, CityId(3));
        assert!(
            new.terms
                .iter()
                .any(|(label, value)| label == "elimination path" && *value > 0)
        );
        assert!(new.value > old.value, "{old:?} -> {new:?}");
    }

    #[test]
    fn a_transferred_world_work_starts_a_valuable_hold_at_nine_vp() {
        let view = late_view(false, true);
        let old = priced(&view, 0);
        let new = priced(&view, 100);
        assert!(
            new.terms
                .iter()
                .any(|(label, value)| label == "VP hold closeout" && *value > 0)
        );
        assert!(new.value > old.value, "{old:?} -> {new:?}");
    }

    #[test]
    fn a_celtic_sack_values_the_rivals_held_points() {
        let mut view = late_view(true, true);
        view.standings[1].vp = 9;
        view.players[ENEMY.0 as usize].vp = 9;
        let old = priced(&view, 0);
        let new = priced(&view, 100);
        assert!(
            new.terms
                .iter()
                .any(|(label, value)| label == "VP hold closeout" && *value > 0)
        );
        assert!(new.value > old.value, "{old:?} -> {new:?}");
    }
}

impl PlanKind {
    /// The team this plan fights, if any. Every other war is one the plan tries to end.
    pub fn war_target(&self) -> Option<TeamId> {
        match self {
            Self::Campaign { against, .. }
            | Self::Contain { against }
            | Self::Snipe { against, .. }
            | Self::FieldStrike { against, .. } => Some(*against),
            Self::Economy | Self::Peace { .. } | Self::Hold { .. } => None,
        }
    }

    /// The same objective, ignoring labels that follow the situation (which enemy a peace
    /// plan names, which unit makes a snipe).
    pub(crate) fn same_objective(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Economy | Self::Peace { .. }, Self::Economy | Self::Peace { .. }) => true,
            (Self::Campaign { target: a, .. }, Self::Campaign { target: b, .. }) => a == b,
            (Self::Contain { against: a }, Self::Contain { against: b }) => a == b,
            (Self::Snipe { settler: a, .. }, Self::Snipe { settler: b, .. }) => a == b,
            (Self::FieldStrike { target: a, .. }, Self::FieldStrike { target: b, .. }) => a == b,
            (Self::Hold { city: a }, Self::Hold { city: b }) => a == b,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub kind: PlanKind,
    pub start_turn: u32,
    /// Before this turn a challenger needs twice the margin.
    pub review_turn: u32,
    /// Gold kept back from discretionary purchases to fund the plan.
    pub reserve: i32,
    /// Estimated change in win chance (hundredths of a percentage point), last evaluated.
    pub value: i32,
    /// The main terms behind `value`.
    pub terms: Vec<(String, i32)>,
    /// The declaration this plan needs breaks an Alliance (Contain against a near-winner).
    #[serde(default)]
    pub breach: bool,
    /// Consecutive turns a war plan's objective dropped off the candidate list. A plan whose
    /// war is under way survives one such turn (forecasts flicker as armies move) before it
    /// ends as infeasible.
    #[serde(default)]
    pub misses: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanReason {
    /// The first plan of the game.
    Start,
    /// A challenger beat the plan by the switching margin.
    Better,
    /// The campaign's city was destroyed or left the view.
    TargetGone,
    /// The campaign took its city.
    Captured,
    /// The war the plan needed ended in peace.
    PeaceSigned,
    /// The campaign ended for its own recorded reason (deadline, army, walls, value).
    CampaignEnded,
    /// The snipe took its Settler.
    SettlerTaken,
    /// The snipe's Settler escaped, founded or vanished.
    SnipeMissed,
    /// The declaration the plan needs is no longer legal.
    WarIllegal,
    /// The plan's objective is no longer on offer (force too weak, leader no longer leads).
    Infeasible,
    /// Hard+Search: rollouts preferred it to the running plan at a review point.
    Search,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    /// The purchase would leave the treasury runway under its floor (`funding.rs`).
    RunwayFloor,
    /// The purchase would dip into the Gold the plan keeps back.
    PlanReserve,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanEvent {
    Switch {
        turn: u32,
        from: Option<PlanKind>,
        to: PlanKind,
        value: i32,
        /// Estimated chance to win when the switch was made (0..=10,000).
        chance: i32,
        reason: PlanReason,
        terms: Vec<(String, i32)>,
    },
    BlockedExpense {
        turn: u32,
        city: CityId,
        item: ProductionItem,
        cost: i32,
        reason: BlockReason,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlanMemory {
    pub active: Option<Plan>,
    pub events: Vec<PlanEvent>,
    /// Every blocked purchase, including those no longer kept as events.
    pub blocked_expenses: u32,
    /// Hard+Search: the objectives rolled out at the last search. Until the next review a
    /// challenger among them doesn't reopen the decision.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub searched: Vec<PlanKind>,
    /// A rollout seat's candidate plan, kept until it ends or the horizon passes (`search`).
    #[serde(skip)]
    pub(crate) forced: Option<crate::search::ForcedPlan>,
    /// Hard+Search information triggers: what the seat saw last turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signals: Option<Signals>,
    /// Hard+Search with the collapse trigger: the win chance (0-10,000) at the last search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub searched_chance: Option<i32>,
    /// Hard+Search tactics: this turn's stance (None = the scripted default).
    #[serde(skip)]
    pub(crate) stance: Option<crate::search::Stance>,
    /// Hard+Search with fronts: every enemy city a campaign could open on is a candidate.
    #[serde(skip)]
    pub(crate) fronts: bool,
    /// Hard+Search with holds: Hold candidates are offered (set for search seats and kept by
    /// their rollout copies).
    #[serde(skip)]
    pub(crate) holds: bool,
    /// Hard+Search: the posture rollouts chose for the knobs (None = the personality).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posture: Option<Posture>,
    /// Genes `war_pressure` and `war_siege`: per enemy team, the last turn our units attacked it
    /// (from our own orders), so a war nobody prosecutes can be told from one that fights.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub war_fights: BTreeMap<TeamId, u32>,
}

/// A searched lean on the scripted knobs, held until the next search (Hard+Search only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Posture {
    /// Army and defence first: more military, less expansion.
    Military,
    /// Settlers first: more expansion, less aggression.
    Expand,
    /// Buildings and research first: less military.
    Economy,
    /// Build ratio: half the army ranged (the personality keeps a third).
    Ranged,
    /// Build ratio: a fifth of the army ranged.
    Melee,
    /// The faction goal first: its pass switches city production to its steps instead of
    /// waiting for an idle city.
    Goal,
    /// Titles and milestones now: the end-game victory planner runs from this turn.
    Victory,
    /// Buildings before troops: no military production beyond the garrison while a city is
    /// unthreatened, and the Economy lean on the knobs.
    Build,
}

impl Posture {
    pub const ALL: [Posture; 8] = [
        Posture::Military,
        Posture::Expand,
        Posture::Economy,
        Posture::Goal,
        Posture::Victory,
        Posture::Build,
        Posture::Ranged,
        Posture::Melee,
    ];

    /// The army's ranged share target: one ranged unit per this many (3 = a third).
    pub fn ranged_divisor(posture: Option<Self>) -> i32 {
        match posture {
            Some(Self::Ranged) => 2,
            Some(Self::Melee) => 5,
            _ => 3,
        }
    }

    pub fn apply(posture: Option<Self>, mut k: crate::Knobs) -> crate::Knobs {
        match posture {
            None => {}
            Some(Self::Military) => {
                k.military = k.military.max(85);
                k.expansion = k.expansion.min(40);
            }
            Some(Self::Expand) => {
                k.expansion = k.expansion.max(85);
                k.aggression = k.aggression.min(30);
            }
            Some(Self::Economy | Self::Build) => {
                k.economy = k.economy.max(85);
                k.science = k.science.max(70);
                k.military = k.military.min(35);
            }
            Some(Self::Ranged | Self::Melee | Self::Goal | Self::Victory) => {}
        }
        k.clamped()
    }
}

/// What a Hard+Search seat watches between plan reviews (all from its own view).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signals {
    /// Rivals met.
    pub met: u32,
    /// Visible rival military (not Scouts, not barbarians) within 6 hexes of our cities.
    pub near: u32,
    /// Public city count of every rival, by player.
    pub cities: Vec<(legatum_core::PlayerId, u32)>,
    /// Wars anywhere (public relations).
    pub wars: u32,
}

impl Signals {
    pub fn of(v: &PlayerView) -> Self {
        let homes: Vec<_> = v
            .cities
            .iter()
            .filter(|c| c.owner == v.me)
            .map(|c| c.pos)
            .collect();
        Self {
            met: v.diplomacy.met.len() as u32,
            near: v
                .units
                .iter()
                .filter(|u| {
                    u.military
                        && u.kind != UnitKind::Scout
                        && !u.owner.is_barbarian()
                        && v.team_of(u.owner) != v.team
                        && homes.iter().any(|h| h.distance(u.pos) <= 6)
                })
                .count() as u32,
            cities: v
                .players
                .iter()
                .filter(|p| p.team != v.team)
                .map(|p| (p.id, p.cities))
                .collect(),
            wars: v
                .diplomacy
                .relations
                .iter()
                .filter(|r| r.state == RelationState::War)
                .count() as u32,
        }
    }

    /// The first trigger that fires against last turn's signals.
    pub fn fired(&self, last: &Self) -> Option<&'static str> {
        if self.met > last.met {
            return Some("met");
        }
        if self.near >= last.near + 2 {
            return Some("military-near");
        }
        if self
            .cities
            .iter()
            .any(|(p, n)| last.cities.iter().any(|(q, m)| p == q && n < m))
        {
            return Some("rival-lost-city");
        }
        if self.wars > last.wars {
            return Some("war");
        }
        None
    }
}

/// What `choose_plan` does this turn.
enum Decision {
    /// The plan stands, repriced from its own candidate.
    Stand(Candidate),
    /// A war plan's objective dropped off the list for one turn.
    Miss,
    /// A challenger beat the plan by the margin.
    Better {
        best: Candidate,
        same: Candidate,
    },
    Switch(Candidate, PlanReason),
}

/// One priced alternative.
#[derive(Clone, Debug)]
pub(crate) struct Candidate {
    pub kind: PlanKind,
    pub value: i32,
    pub terms: Vec<(String, i32)>,
    pub reserve: i32,
    pub breach: bool,
}

/// What a campaign against one city would look like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CampaignOutlook {
    pub city: CityId,
    /// Turns to capture, march included.
    pub turns: i32,
    pub march: i32,
    /// The plan's capture odds in percent; a campaign under `MIN_SEAT_WAR_ODDS` is not offered.
    pub odds: i32,
}

/// A capture forecast for a campaign candidate.
pub(crate) struct Forecast {
    pub(crate) turns: i32,
    pub(crate) losses: i32,
    pub(crate) force: Vec<UnitId>,
    /// Turns the army marches before the siege (0 for a force already in reach).
    pub(crate) march: i32,
}

/// The plan's capture odds (percent) for a forecast taking `turns` turns, `march` of them on the
/// road. The forecast's odds fall with the siege's length, and a march loses a tenth of them per
/// turn on the road (the defender sees it coming).
pub(crate) fn capture_odds(turns: i32, march: i32) -> i32 {
    let raw = (85 - 6 * (turns - march)).clamp(35, 85) * (100 - 10 * march).max(10) / 100;
    // Calibrated on matched 100-game Standard Hard cohorts (seeds 5200001+): the raw
    // forecast overstates new wars (61%+ took the city 38% of the time, 41-60% 11%), and
    // a first correction to (raw - 35) x 1.1 understated them (its 28% average took 37%).
    ((raw - 30) * 6 / 5).clamp(0, 70)
}

/// The lowest capture odds (percent) at which a forecast campaign is offered. In four games
/// every forced war at 15% or more started a campaign; those at 12% or less never did (the plan
/// dropped them within 2 to 23 turns, leaving the war on paper).
/// Campaign targets offered per decision with Hard+Search fronts.
const FRONT_TARGETS: usize = 3;

fn adjust(f: &mut WinFacts, team: TeamId, change: impl FnOnce(&mut TeamFacts)) {
    if let Some(t) = f.team_mut(team) {
        change(t);
    }
}

/// Runway shortfall in permille of the floor, as `WinFacts` measures it.
fn runway_short(gold: i32, net: i32, floor: i32) -> i32 {
    let floor = floor.max(1);
    crate::funding::runway_of(gold, net)
        .map_or(0, |r| (floor - (r.min(floor as u32) as i32)) * 1000 / floor)
}

fn alive(v: &PlayerView, team: TeamId) -> bool {
    v.players.iter().any(|p| p.team == team && p.alive)
}

impl Planner<'_> {
    /// The army (the win estimate's units) `gold` Gold would buy now: units at the average
    /// purchase price. The same exchange rate prices a Celtic sack's Gold.
    pub(crate) fn gold_as_army(&self, base: &WinFacts, gold: i32) -> i32 {
        let unit_gold = (self.average_unit_cost() * data::BUY_COST_PERCENT / 100).max(1);
        self.unit_army_value(base) * (gold * 1000 / unit_gold) / 1000
    }

    /// The plan the planner routes through: Hard only, set by `choose_plan`.
    /// The field force stages toward a chosen Campaign city, at Peace and at war (gene
    /// `planned_stage`, or `march_past_reach`, whose march forecast needs the army brought up).
    pub(crate) fn stages_campaigns(&self) -> bool {
        self.decisions.planned_stage > 0 || self.decisions.march_past_reach > 0
    }

    /// An early Campaign on an open city declares only once its force in reach takes it
    /// (gene `early_campaign_muster`, or `march_past_reach`).
    pub(crate) fn musters_campaigns(&self) -> bool {
        self.decisions.early_campaign_muster > 0 || self.decisions.march_past_reach > 0
    }

    pub(crate) fn routed_plan(&self) -> Option<&Plan> {
        if self.plan_routing {
            self.campaigns.plan.active.as_ref()
        } else {
            None
        }
    }

    /// Turns a plan looks ahead at the fixed Standard pace.
    /// Hard+Search with `stagger`: the first plan's review comes a few turns later for each seat
    /// (seat number modulo the review interval), so the seats' searches don't all fall on one
    /// turn. Later plans keep whatever offset their reviews have drifted to.
    fn review_stagger(&self) -> u32 {
        let first = self.campaigns.plan.active.is_none();
        match self.search {
            Some(hook) if hook.config.stagger && first => {
                u32::from(self.view().me.0) % self.plan_horizon() as u32
            }
            _ => 0,
        }
    }

    pub(crate) fn plan_horizon(&self) -> i32 {
        10
    }

    /// Army title value of one of our military units, in refined units.
    pub(crate) fn unit_army_value(&self, facts: &WinFacts) -> i32 {
        let units = self.ctx.own_units.iter().filter(|u| u.military).count() as i32;
        facts.team(self.view().team).map_or(0, |t| t.army) / units.max(1)
    }

    /// Average Production cost of our military units (for turning losses into units).
    pub(crate) fn average_unit_cost(&self) -> i32 {
        let costs: Vec<i32> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military)
            .map(|u| {
                crate::tuned(
                    self.view()
                        .rule_experiments
                        .unit_stats_for(self.faction, u.kind)
                        .cost,
                )
            })
            .collect();
        if costs.is_empty() {
            crate::tuned(
                self.view()
                    .rule_experiments
                    .unit_stats_for(self.faction, UnitKind::Soldier)
                    .cost,
            )
            .max(1)
        } else {
            (costs.iter().sum::<i32>() / costs.len() as i32).max(1)
        }
    }

    /// Expected cities lost at home (permille of one city) if `team` answers a war with its
    /// visible army: for each of our cities, the share of their nearby strength our local
    /// defense can't hold. `committed` units are away and don't defend.
    pub(crate) fn retaliation_permille(&self, team: TeamId, committed: &[UnitId]) -> i32 {
        let v = self.view();
        let total: i32 = self
            .ctx
            .own_cities
            .iter()
            .map(|home| {
                let their: i32 = v
                    .units
                    .iter()
                    .filter(|e| {
                        e.military
                            && (e.kind != UnitKind::Scout || self.decisions.scout_threat > 0)
                            && v.team_of(e.owner) == team
                            && e.pos.distance(home.pos) <= 6
                            && self.ctx.reachable(e.pos, home.pos)
                    })
                    .map(|e| crate::analysis::threat_power(v, e))
                    .sum::<i32>();
                if their == 0 {
                    return 0;
                }
                let ours = crate::analysis::city_power(v, home) / 2
                    + self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|u| {
                            u.military
                                && !committed.contains(&u.id)
                                && u.pos.distance(home.pos) <= 2
                        })
                        .map(|u| crate::analysis::defense_power(v, u))
                        .sum::<i32>();
                (their - ours).max(0) * 1000 / their
            })
            .sum();
        // Not every opening is taken: half the exposed share, at most one city.
        (total / 2).min(1000)
    }

    /// Whether a plan's projection moves population and territory. Gene `extent_guard`: only
    /// when the win estimate reads them (`WinFacts::extent_known`); while both are 0 for
    /// every team, moving them for one side alone hands it a 100% share.
    fn projects_extent(&self, facts: &WinFacts) -> bool {
        self.decisions.extent_guard == 0 || facts.extent_known()
    }

    /// Facts after losing our most exposed small city to `team`.
    pub(crate) fn city_lost(&self, facts: &WinFacts, team: TeamId) -> WinFacts {
        let v = self.view();
        let pop = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| !c.is_capital)
            .map(|c| c.population)
            .min()
            .unwrap_or(1);
        let mut f = facts.clone();
        let extent = self.projects_extent(facts);
        adjust(&mut f, v.team, |t| {
            t.cities -= 10;
            if extent {
                t.population -= pop * 10;
                t.territory -= 70;
            }
        });
        adjust(&mut f, team, |t| {
            t.cities += 10;
            if extent {
                t.population += (pop / 2).max(1) * 10;
                t.territory += 70;
            }
        });
        f
    }

    /// Era terms toward a war target (all from public eras): every era we lead is a window of
    /// better units, worth a little more while we were the first into our era; every era we
    /// trail is a threat. Containment only gains from a lead: it is priced by the VP it
    /// denies, and a runaway leader is usually an era ahead already.
    pub(crate) fn era_terms(
        &self,
        team: TeamId,
        penalize_deficit: bool,
        terms: &mut Vec<(String, i32)>,
    ) {
        let v = self.view();
        let theirs = v
            .players
            .iter()
            .filter(|p| p.alive && p.team == team)
            .map(|p| i32::from(p.era))
            .max();
        let Some(theirs) = theirs else { return };
        let mine = i32::from(v.research.era);
        let lead = (mine - theirs).clamp(-2, 2);
        if lead > 0 {
            let first = mine > 0 && v.research.era_firsts.contains(&(v.research.era, v.me));
            terms.push((
                "era lead".into(),
                lead * ERA_LEAD_VALUE + if first { FIRST_INTO_ERA_VALUE } else { 0 },
            ));
        } else if lead < 0 && penalize_deficit {
            terms.push(("era deficit".into(), lead * ERA_DEFICIT_VALUE));
        }
    }

    /// Terms for opening a new war: the visible aggressor attracts containment, a war on
    /// top of another splits the army, and a fresh peace given up loses its value.
    fn new_war_terms(&self, team: TeamId, chance: i32, terms: &mut Vec<(String, i32)>) {
        let v = self.view();
        if v.teams_at_war(v.team, team) {
            return;
        }
        terms.push(("aggressor risk".into(), -chance * 2 / 100));
        let other_wars = v
            .standings
            .iter()
            .filter(|s| s.team != team && alive(v, s.team) && v.teams_at_war(v.team, s.team))
            .count() as i32;
        if other_wars > 0 {
            terms.push(("second front".into(), -chance * 4 * other_wars / 100));
        }
        if let Some(r) = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, team))
            .filter(|r| r.state == RelationState::Peace && r.version >= 2)
        {
            let horizon = 30_u32;
            let left = horizon.saturating_sub(v.turn.saturating_sub(r.effective_turn)) as i32;
            if left > 0 {
                terms.push((
                    "broken peace".into(),
                    -chance * 5 * left / (100 * horizon as i32),
                ));
            }
        }
    }

    /// Gene `peace_credibility`: a new war on a team we made Peace with after a war costs value,
    /// most just after the signing and nothing after the horizon, so a Contain score that moves
    /// a little doesn't undo a treaty within turns. A real opportunity (a Campaign's capture) is
    /// worth far more than the charge; a team counting down to victory is never protected.
    /// Genes `war_pressure` and `war_siege`: turns since our units last attacked `team` in the
    /// war we fight with it (since the war began, if they never have); None at peace.
    pub(crate) fn idle_war_turns(&self, team: TeamId) -> Option<u32> {
        let v = self.view();
        let war = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, team) && r.state == RelationState::War)?;
        let since = war.effective_turn.max(
            self.campaigns
                .plan
                .war_fights
                .get(&team)
                .copied()
                .unwrap_or(0),
        );
        Some(v.turn.saturating_sub(since))
    }

    /// Idle turns past the march grace (eight turns), capped at 60.
    pub(crate) fn idle_war_excess(&self, team: TeamId) -> i32 {
        let grace = 16_u32;
        self.idle_war_turns(team)
            .map_or(0, |t| t.saturating_sub(grace).min(60) as i32)
    }

    /// Gene `war_pressure`: a war we declared and do not prosecute is worth less every idle
    /// turn, so the plan either fights it or ends it (user, 2026-10-18: "you can be 'at war' but
    /// that war doesn't do anything"). Never against a team counting down to victory.
    fn war_pressure_term(&self, team: TeamId, terms: &mut Vec<(String, i32)>) {
        let k = self.decisions.war_pressure;
        if k == 0 || self.leader_war(team) {
            return;
        }
        let v = self.view();
        let declared = v.diplomacy.relations.iter().any(|r| {
            r.between(v.team, team)
                && r.state == RelationState::War
                && r.declarers.contains(&v.team)
        });
        let over = self.idle_war_excess(team);
        if declared && over > 0 {
            terms.push(("idle war".into(), -k * over));
        }
    }

    /// A war on the strict public VP leader, a team counting down to victory or our
    /// diplomatic target: `war_pressure` never pushes these toward peace, since even an idle
    /// war on the leader costs him (the 2026-10-18 Swiss what-ifs: peace with him rose and his
    /// win chance with it); the siege objective (`war_siege`) is what makes them fight.
    pub(crate) fn leader_war(&self, team: TeamId) -> bool {
        crate::hold::rival_hold_of(self.view(), team).is_some()
            || self.contain_leader() == Some(team)
            || self.diplomatic_target() == Some(team)
    }

    /// Record which enemy teams this turn's orders attack (a shot or a move onto a tile holding
    /// one of their units or cities), for `idle_war_turns`. Wars that ended are forgotten.
    pub(crate) fn note_war_fights(&mut self, orders: &[Order]) {
        if self.decisions.war_pressure == 0 && self.decisions.war_siege == 0 {
            return;
        }
        let v = self.view();
        let hit = |at: Hex| {
            v.units
                .iter()
                .find(|u| {
                    u.pos == at && u.military && !u.owner.is_barbarian() && v.at_war(v.me, u.owner)
                })
                .map(|u| v.team_of(u.owner))
                .or_else(|| {
                    v.cities
                        .iter()
                        .find(|c| c.pos == at && !c.owner.is_barbarian() && v.at_war(v.me, c.owner))
                        .map(|c| v.team_of(c.owner))
                })
        };
        let attacked: Vec<TeamId> = orders
            .iter()
            .filter_map(|o| match *o {
                Order::RangedAttack { target, .. } => hit(target),
                Order::MoveUnit { to, .. } => hit(to),
                _ => None,
            })
            .collect();
        let turn = v.turn;
        let at_war: Vec<TeamId> = v
            .players
            .iter()
            .map(|p| p.team)
            .filter(|t| *t != v.team && v.teams_at_war(v.team, *t))
            .collect();
        let fights = &mut self.campaigns.plan.war_fights;
        for t in attacked {
            fights.insert(t, turn);
        }
        fights.retain(|t, _| at_war.contains(t));
    }

    fn peace_credibility_term(&self, team: TeamId, terms: &mut Vec<(String, i32)>) {
        let v = self.view();
        let weight = self.decisions.peace_credibility;
        if weight == 0
            || v.teams_at_war(v.team, team)
            || crate::hold::rival_hold_of(v, team).is_some()
        {
            return;
        }
        if let Some(r) = v
            .diplomacy
            .relations
            .iter()
            .find(|r| r.between(v.team, team))
            .filter(|r| r.state == RelationState::Peace && r.version >= 2)
        {
            let horizon = 30_u32;
            let left = horizon.saturating_sub(v.turn.saturating_sub(r.effective_turn)) as i32;
            if left > 0 {
                terms.push(("peace credibility".into(), -weight * left / horizon as i32));
            }
        }
    }

    /// Gene `peace_redeclare`: would a Peace with `team` be broken within turns? While the army
    /// holds a city it just captured no campaign is priced (the held units are not a strike
    /// force), so the quiet plan wins by default and every war is ended, only for the next
    /// campaign to declare again once the hold is over, with the charges of a new war on top
    /// (playtest 8.9.5 Athens: Sparta signed Peace after every capture, Nagoya changed hands 11
    /// times). Ending the war costs what declaring again costs (aggressor risk, second front,
    /// broken peace, peace credibility); keeping it costs the retaliation `team` can bring on
    /// the damaged new city and the rest. The Peace is a detour when the first is larger.
    pub(crate) fn peace_would_be_broken(&self, team: TeamId) -> bool {
        let v = self.view();
        if self.decisions.peace_redeclare == 0
            || !v.teams_at_war(v.team, team)
            || (if self.decisions.peace_pair_scope > 0 {
                self.endangered_by(team)
            } else {
                self.endangered()
            })
            || !(self.campaigns.held_capture.is_some()
                || self.decisions.peace_redeclare >= 2
                    && self
                        .idle_war_turns(team)
                        .is_some_and(|t| t <= self.decisions.recent_fight_turns as u32))
        {
            return false;
        }
        let base = WinFacts::of(v).with_relative(&self.decisions).refined();
        let c0 = base.chance();
        let other_wars = v
            .standings
            .iter()
            .filter(|s| s.team != team && alive(v, s.team) && v.teams_at_war(v.team, s.team))
            .count() as i32;
        let again = c0 * 2 / 100
            + c0 * 4 * other_wars / 100
            + c0 * 5 / 100
            + self.decisions.peace_credibility;
        let q = self.retaliation_permille(team, &[]);
        let stay = q * (c0 - self.city_lost(&base, team).chance()) / 1000;
        again > stay
    }

    /// Facts common to a war plan lasting `turns`: the runway after paying for the war (and
    /// and the growth its army investment forgoes.
    /// `growth` is the share of our economy (permille over a full horizon) the war's
    /// rebuilding diverts from buildings.
    fn war_footing(&self, base: &WinFacts, turns: i32, growth: i32) -> WinFacts {
        let v = self.view();
        let h = self.plan_horizon();
        let net = v.economy.gold_per_turn.total;
        let gold = v.economy.gold + net * turns.clamp(0, h);
        let mut f = base.clone();
        f.runway_short = f
            .runway_short
            .max(runway_short(gold, net, self.runway_floor() as i32));
        adjust(&mut f, v.team, |t| {
            t.economy -= t.economy * growth * turns.clamp(1, h) / (h * 1000);
        });
        f
    }

    pub(crate) fn campaign_forecast(
        &self,
        city: &CityView,
        committed: Option<&[UnitId]>,
    ) -> Option<Forecast> {
        self.campaign_forecast_with(city, committed, false, None)
    }

    /// `campaign_forecast`, with two options for gene `war_reaim`. `open_front`: the war on the
    /// city's owner is already being fought, so a march opens no second front, and the march is
    /// judged window by window (`earliest_winning_march`). `defended`: a view (at war with the
    /// owner) in which the city holds one bought defender.
    fn campaign_forecast_with(
        &self,
        city: &CityView,
        committed: Option<&[UnitId]>,
        open_front: bool,
        defended: Option<&PlayerView>,
    ) -> Option<Forecast> {
        let v = self.view();
        let mut prospective;
        let view = if let Some(defended) = defended {
            defended
        } else if v.at_war(v.me, city.owner) {
            v
        } else {
            prospective = v.clone();
            let team = v.team_of(city.owner);
            prospective
                .diplomacy
                .relations
                .iter_mut()
                .find(|r| r.between(v.team, team))?
                .state = RelationState::War;
            &prospective
        };
        if open_front {
            return self.earliest_winning_march(view, city);
        }
        // A force in reach is judged by the strike's siege forecast (a fogged city is assumed
        // to hold one fortified Soldier).
        let force: Vec<_> = self
            .strike_force(view, city)
            .into_iter()
            .filter(|(u, _, _)| committed.is_none_or(|c| c.contains(&u.id)))
            .collect();
        if !force.is_empty() || committed.is_some() {
            if force
                .iter()
                .any(|(u, _, _)| crate::composition::charges(u.kind))
                && let Some(siege) = self.forecast_siege(view, city, &force)
            {
                return Some(Forecast {
                    turns: siege.turns,
                    losses: siege.losses,
                    force: force.iter().map(|(u, _, _)| u.id).collect(),
                    march: 0,
                });
            }
            // For a prized wall, the local two-turn wing can be too small while the
            // committed reinforcements are still marching. Price the whole field force
            // below before declaring the siege impossible. Gene `march_past_reach`: so for
            // a new war's campaign (no committed force), whose in-reach units are only part
            // of the army a march would bring; at 2 also for a city we are already at war with.
            let march_past_reach = committed.is_none()
                && match self.decisions.march_past_reach {
                    1 => !v.at_war(v.me, city.owner),
                    2 => true,
                    _ => false,
                };
            if !march_past_reach
                && !self.sparta_walled_prize(city)
                && !self.sparta_opening_byzantium(city)
                && !self.sparta_open_followup(city)
                && !self.sparta_chain_target(city)
            {
                return None;
            }
        }
        // A march opens no second front: the army can't be in two places.
        if v.players.iter().any(|p| {
            p.alive
                && p.team != v.team
                && p.team != v.team_of(city.owner)
                && v.teams_at_war(v.team, p.team)
                && !(self.decisions.march_ignores_dead_war > 0 && p.cities == 0)
                && !(self.decisions.march_ignores_second_war > 0
                    && (self.sparta_chain_target(city)
                        || (self.value_first_target()
                            || v.faction_of(city.owner) == Faction::India)
                            && !city.walls)
                    && self.ctx.own_cities.iter().any(|home| {
                        home.pos.distance(city.pos) <= self.decisions.open_prize_radius
                    })
                    && !self.ctx.enemy_units.iter().any(|u| {
                        u.military
                            && !u.owner.is_barbarian()
                            && v.team_of(u.owner) == p.team
                            && self
                                .ctx
                                .own_cities
                                .iter()
                                .any(|home| home.pos.distance(u.pos) <= 6)
                    }))
        }) {
            return None;
        }
        // The field army that can reach assault tiles within the march window, played out
        // by the same forecast as if already in place; the odds then fall with the distance.
        let far: Vec<_> = self
            .strike_force_within(view, city, self.decisions.march_turns)
            .into_iter()
            .filter(|(u, _, _)| committed.is_none_or(|ids| ids.contains(&u.id)))
            .collect();
        if !far
            .iter()
            .any(|(u, _, _)| crate::composition::charges(u.kind))
            || city.walls
                && !self.sparta_walled_prize(city)
                && !far
                    .iter()
                    .any(|(u, _, _)| crate::composition::shooter_only(u.kind, u.ranged))
        {
            return None;
        }
        let march = far.iter().map(|(_, _, t)| *t).max().unwrap_or(0);
        let placed: Vec<_> = far.iter().map(|(u, h, _)| (*u, *h, 0)).collect();
        let siege = self.forecast_siege(view, city, &placed);
        if siege.is_none() && (self.sparta_open_followup(city) || self.sparta_chain_target(city)) {
            let force: Vec<_> = self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    u.military
                        && u.kind == UnitKind::Soldier
                        && u.hp >= self.decisions.sparta_expedition_min_hp
                        && u.pos.distance(city.pos) <= self.decisions.open_prize_radius
                })
                .map(|u| u.id)
                .collect();
            if force.len() >= if self.sparta_chain_target(city) { 3 } else { 5 } {
                return Some(Forecast {
                    turns: if self.sparta_chain_target(city) { 5 } else { 7 },
                    losses: crate::tuned(
                        self.view()
                            .rule_experiments
                            .unit_stats_for(self.faction, UnitKind::Soldier)
                            .cost,
                    ),
                    force,
                    march: if self.sparta_chain_target(city) { 2 } else { 3 },
                });
            }
        }
        let siege = siege?;
        Some(Forecast {
            turns: march + siege.turns,
            losses: siege.losses,
            force: far.iter().map(|(u, _, _)| u.id).collect(),
            march,
        })
    }

    /// Gene `war_reaim`: the war on the city's owner is already open, so the march is judged
    /// on its own: for each march window from the strike's approach to the full march, the
    /// units that reach an assault tile within it play the siege out, and the window with the
    /// best capture odds wins (a few units close by can beat the whole army five turns out,
    /// since the odds fall with every turn on the road).
    fn earliest_winning_march(&self, view: &PlayerView, city: &CityView) -> Option<Forecast> {
        let mut best: Option<Forecast> = None;
        for window in self.decisions.approach_turns..=self.decisions.march_turns {
            let far = self.strike_force_within(view, city, window);
            if !far
                .iter()
                .any(|(u, _, _)| crate::composition::charges(u.kind))
                || city.walls
                    && !far
                        .iter()
                        .any(|(u, _, _)| crate::composition::shooter_only(u.kind, u.ranged))
            {
                continue;
            }
            let march = far.iter().map(|(_, _, t)| *t).max().unwrap_or(0);
            let placed: Vec<_> = far.iter().map(|(u, h, _)| (*u, *h, 0)).collect();
            let Some(siege) = self.forecast_siege(view, city, &placed) else {
                continue;
            };
            let f = Forecast {
                turns: march + siege.turns,
                losses: siege.losses,
                force: far.iter().map(|(u, _, _)| u.id).collect(),
                march,
            };
            if best
                .as_ref()
                .is_none_or(|b| capture_odds(f.turns, f.march) > capture_odds(b.turns, b.march))
            {
                best = Some(f);
            }
        }
        best
    }

    fn factory_goal_capture_delay(&self, city: &CityView) -> i32 {
        if self.decisions.capture_vp_accounting == 0
            || self.faction != Faction::Germany
            || !self.germany_goal_on()
        {
            return 0;
        }
        if city.detail.as_ref().is_some_and(|detail| {
            data::has_building(&detail.buildings, legatum_core::BuildingKind::MachineShop)
        }) {
            let foundries = self
                .ctx
                .own_cities
                .iter()
                .filter(|c| {
                    c.detail.as_ref().is_some_and(|d| {
                        data::has_building(&d.buildings, legatum_core::BuildingKind::MachineShop)
                    })
                })
                .count() as i32;
            let needed = (data::GERMANY_GOAL_CITIES - foundries).max(0);
            return if needed > 0 {
                -(data::FACTION_GOAL_VP as i32 * 100) / needed
            } else {
                0
            };
        }
        // WinFacts stores hundredths of VP. A city brings Germany closer to the
        // five Foundry slots it needs; once it has five cities, another city has
        // no goal cost or benefit.
        self.factory_goal_expansion_cost() * 100 / 1500
    }

    /// Gene `capture_horizon`: the turn a campaign's capture is priced at. Against a standing
    /// walled city with no Siege in the force and none on offer to it, the siege rounds after
    /// the march count at the gene's percent (the forecast already halves every attack for the
    /// Walls; what it misses is the execution of a slow siege under the city's strike, measured
    /// on the Lab's Nicomedia: forecast 4 turns, taken within 4 in 7 of 50 runs). Otherwise, and
    /// with the gene off, the forecast's own turns.
    pub(crate) fn capture_turns(&self, city: &CityView, f: &Forecast) -> i32 {
        let percent = self.decisions.capture_horizon;
        if percent == 0 || !city.walls || city.hp <= 0 {
            return f.turns;
        }
        let v = self.view();
        let siege_on_hand = f.force.iter().any(|id| {
            v.units.iter().any(|u| {
                u.id == *id
                    && (data::has_siege(u.kind, u.siege)
                        || (self.decisions.siege_for_walls > 0
                            && u.siege_offer
                                .as_ref()
                                .is_some_and(|o| o.blocker.is_none() || o.waits_for_gold)))
            })
        });
        if siege_on_hand {
            return f.turns;
        }
        let siege = (f.turns - f.march).max(0);
        f.march + siege * percent / 100
    }

    pub(crate) fn price_campaign(
        &self,
        base: &WinFacts,
        c0: i32,
        city: &CityView,
        f: &Forecast,
    ) -> Candidate {
        let v = self.view();
        let owner = v.team_of(city.owner);
        // The forecast's odds fall with the siege's length, and a march loses a tenth of
        // them per turn on the road (the defender sees it coming).
        // Gene `capture_horizon`: a walled siege without Siege is priced at its slower pace.
        let capture_turns = self.capture_turns(city, f);
        let p = (capture_odds(capture_turns, f.march) + self.levy_edge_odds(owner)).min(70);
        let unit = self.unit_army_value(base);
        let lost_milli = f.losses * 1000 / self.average_unit_cost();
        let common = self.war_footing(base, f.turns, 20);
        let c_common = common.chance();
        let land = city
            .pos
            .within(1)
            .iter()
            .filter(|h| v.tile(**h).is_some_and(|t| t.owner == Some(city.owner)))
            .count() as i32;
        // Rival city yields are private. The public Largest Economy metric is Gold plus
        // Production; allocate its owner's total across cities by population plus a
        // two-population base for buildings and the center tile. This is the output we
        // gain and the rival loses, before either empire compounds from its new city count.
        let extent = self.projects_extent(base);
        let captured_economy = base.team(owner).map_or(0, |t| {
            let cities = (t.cities / 10).max(1) as i64;
            let weight = (city.population.max(1) * 10 + 20) as i64;
            let owner_population = if extent {
                t.population as i64
            } else {
                // Population is not a public fact: sum the owner's cities we know of, and
                // take any we have not seen at the size of this one.
                let known: Vec<_> = v
                    .cities
                    .iter()
                    .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == owner)
                    .collect();
                let seen: i64 = known.iter().map(|c| c.population.max(1) as i64 * 10).sum();
                seen + (cities - known.len() as i64).max(0) * city.population.max(1) as i64 * 10
            };
            let total_weight = (owner_population + 20 * cities).max(weight);
            (t.economy as i64 * weight / total_weight) as i32
        }) * self.decisions.capture_economy_percent
            / 100;
        // The city's strategic deposits: the Forges and Factories their supply would put to
        // work for us, and what their owner loses (`supply_value.rs`).
        let (supply_gain, supply_loss) = crate::supply_value::capture_value(v, city);
        // A seat short of supply (idle users, or cities ready for a Forge or Factory with no
        // Iron or Coal to spare) counts a deposit city's supply for more: the shortage is the
        // reason. The Production is also a term of its own (`supply_term` win-chance points per
        // Production per turn): its share of the public economy metric is too small to move a
        // target choice, though supply is what Forges, Factories and Pikemen run on.
        let shortage = if crate::supply_value::short(v) {
            self.decisions.supply_shortage_percent
        } else {
            100
        };
        let supply_points =
            (supply_gain + supply_loss / 2) * self.decisions.supply_term * shortage / 100;
        let supply_gain = supply_gain * self.decisions.supply_value_percent / 100;
        let supply_loss = supply_loss * self.decisions.supply_value_percent / 100;
        // A capture after a holder's countdown ends comes too late to matter: it has won.
        let in_time = crate::hold::rival_hold_of(v, owner)
            .is_none_or(|h| f.turns.max(1) <= h.turns_left as i32);
        // Below five cities, a capture offers another Foundry slot. Enemy
        // buildings are normally private, so known Foundries need no estimate.
        let own_goal_delay = self.factory_goal_capture_delay(city);
        let mut win = common.clone();
        adjust(&mut win, v.team, |t| {
            t.army -= unit * lost_milli / 1000;
        });
        if in_time {
            if self.sacks_city(city) {
                // A sack keeps nothing: the Gold taken buys units, the warband camp it raises may complete Raise the Warbands, the
                // capture itself may score (a Captures objective), and the owner loses the city, its people, land and
                // output, and the VP that went with them (`sack_vp_denied`).
                let take = crate::opportunity::sack_gold_value(
                    self.view().rule_experiments,
                    city,
                    self.faction,
                    self.ctx.me,
                );
                let army = self.gold_as_army(base, take);
                let camp_vp = self.sack_camp_vp(city);
                adjust(&mut win, v.team, |t| {
                    t.army += army;
                    t.vp += 100
                        * (self.conquest_victory_value(city)
                            + camp_vp
                            + self.capital_capture_vp(city));
                });
                let denied = self.sack_vp_denied(city);
                adjust(&mut win, owner, |t| {
                    t.vp -= 100 * denied;
                });
                self.project_capital_capture(&mut win, city);
            } else {
                let (goal_taken, _) = self.capture_goal_swing(city);
                adjust(&mut win, v.team, |t| {
                    t.cities += 10;
                    if extent {
                        t.population += (city.population / 2).max(1) * 10;
                        t.territory += land * 10;
                    }
                    t.economy += captured_economy + supply_gain;
                    t.vp += 100
                        * (self.conquest_victory_value(city)
                            + goal_taken
                            + self.capital_capture_vp(city));
                    t.vp -= own_goal_delay;
                });
            }
            if !self.sacks_city(city) {
                self.project_capital_capture(&mut win, city);
                let (_, goal_denied) = self.capture_goal_swing(city);
                adjust(&mut win, owner, |t| t.vp -= 100 * goal_denied);
            }
            adjust(&mut win, owner, |t| {
                t.cities -= 10;
                if extent {
                    t.population -= city.population * 10;
                    t.territory -= land * 10;
                }
                t.economy = (t.economy - captured_economy - supply_loss).max(0);
            });
        }
        let mut lose = common.clone();
        // Most failed campaigns end before the assault (the forecast fails as the defender
        // reinforces) and withdraw: half the forecast losses.
        adjust(&mut lose, v.team, |t| t.army -= unit * lost_milli / 2000);
        let q = self.retaliation_permille(owner, &f.force);
        let hit = self.city_lost(&common, owner);
        let mut terms = vec![
            (format!("capture at {p}%"), p * (win.chance() - c0) / 100),
            (
                "failed siege".into(),
                (100 - p) * (lose.chance() - c0) / 100,
            ),
        ];
        if self.decisions.late_capture_value > 0 && in_time {
            // The fitted softmax can already say 100% while this team is still one point
            // short of winning. Keep the value of a *sooner* victory: a capture that starts
            // our hold, or breaks theirs, has an observable ten-turn payoff even when its
            // marginal eventual win probability rounds to zero.
            let finish_value = |facts: &WinFacts, team| {
                let hold = facts
                    .hold_chances()
                    .into_iter()
                    .find(|(id, _)| *id == team)
                    .map_or(0, |(_, chance)| chance);
                if hold > 0 {
                    return hold;
                }
                // A point short of the hold is not a finished race. Discount the chance
                // of completing a fresh hold for each further point still needed, so
                // stripping a rival's held VP has value before its countdown begins.
                let missing = facts
                    .hold_margin(team)
                    .map_or(7, |m| (-m).max(0) as u64)
                    .div_ceil(100);
                if missing == 0 || missing > 6 {
                    return 0;
                }
                (0..missing).fold(
                    crate::winchance::hold_completion(facts.hold_turns as i64, 0),
                    |value, _| value * 2 / 5,
                )
            };
            let own_hold = (finish_value(&win, v.team) - finish_value(&common, v.team)).max(0);
            let denied_hold = (finish_value(&common, owner) - finish_value(&win, owner)).max(0);
            // A rival's delayed finish is worth our share of the race. Avoid counting the
            // part the ordinary probability delta already saw.
            let hold_value = (own_hold + denied_hold * c0 as i64 / 10_000) * 10;
            let already_priced = (win.chance() - c_common).max(0) as i64;
            let residual = (hold_value - already_priced).max(0) as i32;
            if residual > 0 {
                terms.push((
                    "VP hold closeout".into(),
                    p * residual * self.decisions.late_capture_value / 10_000,
                ));
            }
            // In a duel, taking the last rival city also wins. A sequence of n cities is
            // worth the forecast odds raised to n, discounted early in the game when the
            // ordinary compounding economy term can still distinguish alternatives.
            if base.teams.len() == 2 && win.hold_margin(v.team).is_some_and(|m| m < 0) {
                let cities = base.team(owner).map_or(0, |t| (t.cities / 10).max(0));
                if cities > 0 {
                    let chain =
                        (0..cities.min(12)).fold(10_000_i64, |value, _| value * p as i64 / 100);
                    let tempo = chain * base.phase.min(1000) as i64 / 1000;
                    if tempo > 0 {
                        terms.push((
                            "elimination path".into(),
                            (tempo * self.decisions.late_capture_value as i64 / 100) as i32,
                        ));
                    }
                }
            }
        }
        if in_time {
            let coalition_prize = self.coalition_city_push(city);
            if coalition_prize > 0 {
                terms.push(("coalition city prize".into(), p * coalition_prize / 100));
            }
        }
        // Gene `roman_promotion_value`: each first promotion the siege earns is a Roman
        // city founded later, won or lost (a failed siege still fought, half as long).
        let promotions = self.roman_siege_promotions(&f.force, capture_turns - f.march);
        if promotions > 0 {
            let mut founded = common.clone();
            adjust(&mut founded, v.team, |t| t.cities += 10 * promotions);
            terms.push((
                "veteran founders".into(),
                (founded.chance() - c_common) * (100 + p) / 200
                    * self.decisions.roman_promotion_value
                    / 100,
            ));
        }
        if q > 0 {
            terms.push(("retaliation".into(), q * (hit.chance() - c_common) / 1000));
        }
        if supply_points > 0 && in_time && !self.sacks_city(city) {
            // Gene `capture_horizon`: the deposits work for us only from the capture on, so
            // the term counts the share of the plan horizon left after it.
            let h = self.plan_horizon();
            let share = if self.decisions.capture_horizon > 0 {
                (h - capture_turns.max(0)).clamp(0, h)
            } else {
                h
            };
            terms.push(("supply".into(), p * supply_points / 100 * share / h));
        }
        let captive_tech = self.greece_captive_tech_value(city);
        if captive_tech > 0 && in_time {
            terms.push(("Athenian technology".into(), p * captive_tech / 100));
        }
        if in_time {
            let objective_denial = self.rival_objective_denial(city);
            if objective_denial > 0 {
                terms.push(("rival's next objective".into(), p * objective_denial / 100));
            }
            if self.decisions.vp_hold_break > 0
                && base.team(owner).is_some_and(|t| t.hold_left.is_some())
                && base.hold_margin(owner).is_some_and(|m| m >= 0)
                && win.hold_margin(owner).is_some_and(|m| m < 0)
            {
                terms.push((
                    "break the hold".into(),
                    p * 35 * self.decisions.vp_hold_break / 100,
                ));
            }
        }
        self.era_terms(owner, true, &mut terms);
        self.new_war_terms(owner, c0, &mut terms);
        self.peace_credibility_term(owner, &mut terms);
        self.war_pressure_term(owner, &mut terms);
        let early_era = i32::from(v.research.era);
        if early_era < 2 && self.decisions.early_campaign_value > 0 {
            terms.push((
                "early campaign window".into(),
                self.decisions.early_campaign_value / (early_era + 1),
            ));
        }
        if city.walls && self.decisions.campaign_wall_cost > 0 {
            terms.push((
                "Walls to overcome".into(),
                -self.decisions.campaign_wall_cost,
            ));
        }
        // Gene `celts_playbook` part 4: a new war goes where the defence visibly isn't.
        let soft = self.celtic_soft_target(city);
        if soft > 0 && !v.at_war(v.me, city.owner) {
            terms.push(("soft target (no Walls, garrison or army)".into(), soft));
        }
        if self.difficulty == AiDifficulty::Hard && self.decisions.supply_race_cap > 0 {
            let race = terms
                .iter()
                .filter(|(name, _)| {
                    name.starts_with("capture at ")
                        || matches!(
                            name.as_str(),
                            "VP hold closeout"
                                | "elimination path"
                                | "coalition city prize"
                                | "rival's next objective"
                                | "break the hold"
                        )
                })
                .map(|(_, value)| *value)
                .max()
                .unwrap_or(0)
                .max(0);
            if let Some((_, supply)) = terms.iter_mut().find(|(name, _)| name == "supply") {
                *supply = (*supply).min(race);
            }
        }
        let net = v.economy.gold_per_turn.total;
        let hold = (data::CAPTURE_RESISTANCE_TURNS) + 1;
        Candidate {
            kind: PlanKind::Campaign {
                against: owner,
                target: city.id,
            },
            value: terms.iter().map(|(_, x)| x).sum(),
            terms,
            reserve: if net < 0 {
                (-net * (f.turns.max(0) + hold)).min(v.economy.gold)
            } else {
                0
            },
            breach: false,
        }
    }

    /// Gene `campaign_persist`: a siege under way doesn't end at the first forecast miss. A
    /// defender that buys a garrison unit each turn breaks the capture forecast every turn, but
    /// it pays for each one, and our march to the city is already spent. While our committed
    /// force at the city is still at least the gene's percent of its local defence, the campaign
    /// is priced as a grind: our shooters' fire takes the city's HP down (a city hit this turn
    /// doesn't heal), plus a turn for the assault, at the loss of a strike from the city each
    /// turn. Only the running campaign is ever priced this way, never a new war.
    pub(crate) fn grind_forecast(&self, city: &CityView, committed: &[UnitId]) -> Option<Forecast> {
        let persist = self.decisions.campaign_persist;
        if persist == 0
            || !self.hard_defense()
            || city.hp <= 0
            || self.decisions.plan_persist_skip > 0
        {
            return None;
        }
        if self.siege_local_margin(city, committed)? < persist {
            return None;
        }
        let force: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| committed.contains(&u.id) && u.pos.distance(city.pos) <= 3)
            .collect();
        // Walls halve what the force does to the city's HP (Siege is not counted).
        let percent = if city.walls {
            legatum_core::combat::WALLS_DAMAGE_PERCENT
        } else {
            100
        };
        let fire: i32 = force
            .iter()
            .filter(|u| u.ranged > 0 && u.range > 0)
            .map(|u| {
                legatum_core::combat::damage_to_city_hp(
                    percent,
                    legatum_core::combat::damage(
                        u.ranged * u.hp.clamp(1, 100) / 100,
                        city.strength,
                        100,
                    ),
                )
            })
            .sum();
        if fire == 0 {
            return None;
        }
        let turns = ((city.hp + fire - 1) / fire + 1).min(12);
        // A city strike wounds one unit a turn; the wounded heal behind the line, so about one
        // unit in four strikes is lost behind Walls (one in eight to a garrison's sorties).
        let strike = self.average_unit_cost() / if city.walls { 4 } else { 8 };
        Some(Forecast {
            turns,
            losses: strike * turns,
            force: force.iter().map(|u| u.id).collect(),
            march: 0,
        })
    }

    /// An existing wall war can need the field force to assemble before a short siege
    /// forecast succeeds. Price that staging from the shooters that can reach the front;
    /// the city still gets its expected shot on every siege turn.
    fn wall_staging_forecast(&self, city: &CityView) -> Option<Forecast> {
        let v = self.view();
        if !city.walls
            || city.hp <= 0
            || !v.at_war(v.me, city.owner)
            || self.decisions.war_siege == 0
        {
            return None;
        }
        let near: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| {
                u.military
                    && u.hp >= 30
                    && u.pos.distance(city.pos) <= self.decisions.siege_reach
                    && self.ctx.reachable(u.pos, city.pos)
            })
            .collect();
        if near.len() < 3
            || !near.iter().any(|u| crate::composition::charges(u.kind))
            || near.iter().filter(|u| u.ranged > 0 && u.range > 0).count() < 2
            || self.attack_force(&near) * 100
                < self.city_defence_weight(city) * self.decisions.war_siege
        {
            return None;
        }
        let fire: i32 = near
            .iter()
            .filter(|u| u.ranged > 0 && u.range > 0)
            .map(|u| {
                legatum_core::combat::damage_to_city_hp(
                    legatum_core::combat::WALLS_DAMAGE_PERCENT,
                    legatum_core::combat::damage(u.ranged * u.hp / 100, city.strength, 100),
                )
            })
            .sum();
        if fire <= 0 {
            return None;
        }
        let march = near
            .iter()
            .filter(|u| u.ranged > 0 && u.range > 0)
            .map(|u| (u.pos.distance(city.pos) - u.range).max(0) / u.max_moves.max(1) + 1)
            .min()
            .unwrap_or(2)
            .max(1);
        let turns = ((city.hp + fire - 1) / fire + 1).min(12) + march;
        Some(Forecast {
            turns,
            losses: self.average_unit_cost() * turns / 4,
            force: near.iter().map(|u| u.id).collect(),
            march,
        })
    }

    /// The forecast, if the reserve ledger's gate lets the campaign open: on a runway under the
    /// floor a campaign is open only if its capture comes first (`campaign.rs::campaign_funded`).
    fn funded_forecast(&self, city: &CityView, committed: Option<&[UnitId]>) -> Option<Forecast> {
        let f = self.campaign_forecast(city, committed)?;
        let floor = self.runway_floor();
        if self
            .gold_runway()
            .is_some_and(|r| r < floor && f.turns.max(0) as u32 > r)
        {
            return None;
        }
        Some(f)
    }

    /// The best campaign a new war on `team` could run: its known city with the strongest
    /// calibrated capture odds, then the fastest forecast. Read-only Contain commitment gate.
    /// `Err` carries why no campaign is possible now.
    pub(crate) fn campaign_against(&self, team: TeamId) -> Result<CampaignOutlook, &'static str> {
        let v = self.view();
        let mut cities: Vec<&CityView> = v
            .cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == team)
            .collect();
        if cities.is_empty() {
            return Err("no city seen yet");
        }
        cities.sort_by_key(|c| c.id);
        if !v.teams_at_war(v.team, team) && !legal_declaration(v, team) {
            return Err("war is not legal now");
        }
        let best = cities
            .iter()
            .filter_map(|c| self.campaign_outlook(c))
            .min_by_key(|o| (std::cmp::Reverse(o.odds), o.turns, o.city));
        best.ok_or_else(|| {
            if self.campaigns.held_capture.is_some() {
                "our army is holding a captured city"
            } else if v.players.iter().any(|p| {
                p.alive && p.team != v.team && p.team != team && v.teams_at_war(v.team, p.team)
            }) {
                "already at war with another empire, no second front"
            } else if !self
                .ctx
                .own_units
                .iter()
                .any(|u| u.military && crate::composition::charges(u.kind) && u.kind != UnitKind::Scout)
            {
                "no melee unit to capture a city"
            } else if self.gold_runway().is_some_and(|r| r < self.runway_floor()) {
                "Gold runway too short to fund it"
            } else {
                "army too weak or too far away: forecast capture odds under 15% for every known city"
            }
        })
    }

    /// The forecast campaign against one city (at war, or prospectively), if it can open.
    pub(crate) fn campaign_outlook(&self, city: &CityView) -> Option<CampaignOutlook> {
        let v = self.view();
        if city.owner.is_barbarian() || v.same_team(v.me, city.owner) {
            return None;
        }
        if !v.at_war(v.me, city.owner) && !legal_declaration(v, v.team_of(city.owner)) {
            return None;
        }
        // No new campaign begins while the army holds a captured city (`reconcile_campaign`).
        if self.campaigns.held_capture.is_some() && !self.sparta_chain_target(city) {
            return None;
        }
        if !v.at_war(v.me, city.owner) && self.stale_city(city) {
            return None;
        }
        let aged = self.aged_city(city);
        let city = aged.as_ref().unwrap_or(city);
        let f = self.funded_forecast(city, None)?;
        // A force already in reach must also pass the campaign's own gate, which starts the
        // campaign once at war (`campaign.rs::begin_campaign`), or the war stays on paper.
        if f.march == 0 {
            self.campaign_forecast(city, Some(&self.campaign_force(city)))?;
        }
        let odds = capture_odds(f.turns, f.march);
        (odds >= self.decisions.min_seat_war_odds).then_some(CampaignOutlook {
            city: city.id,
            turns: f.turns,
            march: f.march,
            odds,
        })
    }

    fn campaign_candidates(&self, base: &WinFacts, c0: i32) -> Vec<Candidate> {
        let v = self.view();
        if self.takes_no_cities() {
            return Vec::new();
        }
        let mut cities: Vec<(CityId, Option<Vec<UnitId>>)> = Vec::new();
        if let Some(active) = &self.campaigns.active {
            cities.push((active.target, Some(active.committed.clone())));
        }
        if let Some(strike) = self.opportunity_strike() {
            cities.push((strike.city, None));
        }
        if let Some(city) = self.conquest_city() {
            cities.push((city.id, None));
        }
        if let Some(city) = v
            .cities
            .iter()
            .filter(|c| self.sparta_open_followup(c))
            .min_by_key(|c| {
                (
                    !c.is_capital,
                    self.ctx
                        .own_units
                        .iter()
                        .filter(|u| u.kind == UnitKind::Soldier)
                        .map(|u| u.pos.distance(c.pos))
                        .min()
                        .unwrap_or(99),
                    c.id,
                )
            })
        {
            cities.push((city.id, None));
        }
        if let Some(city) = v
            .cities
            .iter()
            .filter(|c| self.sparta_chain_target(c))
            .min_by_key(|c| {
                (
                    self.ctx
                        .own_units
                        .iter()
                        .filter(|u| u.kind == UnitKind::Soldier)
                        .map(|u| u.pos.distance(c.pos))
                        .min()
                        .unwrap_or(99),
                    c.id,
                )
            })
        {
            cities.push((city.id, None));
        }
        if let Some(PlanKind::Campaign { target, .. }) = self.routed_plan().map(|p| &p.kind) {
            cities.push((*target, None));
        }
        // A wall siege already at war may have no short-horizon capture forecast after
        // city fire. Keep it on the candidate list while a field force assembles.
        for city in v.cities.iter().filter(|city| {
            city.walls
                && city.hp > 0
                && v.at_war(v.me, city.owner)
                && !v.same_team(v.me, city.owner)
        }) {
            if self.wall_staging_forecast(city).is_some() {
                cities.push((city.id, None));
            }
        }
        // Every holder city whose loss takes a live point is a campaign candidate. Prefer
        // an in-time capture that removes enough points to end the countdown.
        for hold in crate::hold::holds(v).iter().filter(|h| h.team != v.team) {
            let mut open: Vec<(u32, CampaignOutlook)> = v
                .cities
                .iter()
                .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == hold.team)
                .filter_map(|c| {
                    let points = self.hold_city_points(c);
                    if points == 0 {
                        return None;
                    }
                    Some((points, self.campaign_outlook(c)?))
                })
                .filter(|(_, o)| {
                    self.decisions.vp_hold_break == 0 || o.turns.max(1) <= hold.turns_left as i32
                })
                .collect();
            open.sort_by_key(|(points, o)| {
                if self.decisions.vp_hold_break > 0 {
                    (
                        std::cmp::Reverse(*points > hold.margin()),
                        std::cmp::Reverse(*points),
                        o.turns,
                        o.city,
                    )
                } else {
                    (
                        std::cmp::Reverse(false),
                        std::cmp::Reverse(0),
                        o.turns,
                        o.city,
                    )
                }
            });
            cities.extend(
                open.into_iter()
                    .take(FRONT_TARGETS)
                    .map(|(_, o)| (o.city, None)),
            );
        }
        // One point short of our own target: every rival city whose capture scores a point for
        // us (a title or milestone it would swing; goals-3 stalls: 9-VP teams sat at peace for
        // 900 turns while a capture would have taken the title they lacked), soonest first.
        let one_short = self.decisions.endgame_push > 0
            && v.standings
                .iter()
                .any(|s| s.team == v.team && s.vp + 1 >= s.target && s.vp < s.target);
        if one_short {
            let mut open: Vec<CampaignOutlook> = v
                .cities
                .iter()
                .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner))
                .filter(|c| {
                    self.conquest_victory_value(c) > 0
                        || self.decisions.late_capture_value > 0 && self.sack_vp_denied(c) > 0
                })
                .filter_map(|c| self.campaign_outlook(c))
                .collect();
            open.sort_by_key(|o| (o.turns, o.city));
            cities.extend(open.into_iter().take(FRONT_TARGETS).map(|o| (o.city, None)));
        }
        for p in v.players.iter().filter(|p| p.alive && p.team != v.team) {
            if let Some(pos) = self.diplomatic_campaign(p.team)
                && let Some(city) = v.cities.iter().find(|c| c.pos == pos)
            {
                cities.push((city.id, None));
            }
        }
        // Hard+Search with fronts: every enemy city a campaign could open on, soonest first
        // (at most `FRONT_TARGETS`), so search chooses where to strike.
        if self.campaigns.plan.fronts {
            let mut open: Vec<CampaignOutlook> = v
                .cities
                .iter()
                .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner))
                .filter_map(|c| self.campaign_outlook(c))
                .collect();
            open.sort_by_key(|o| (o.turns, o.city));
            cities.extend(open.into_iter().take(FRONT_TARGETS).map(|o| (o.city, None)));
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut out: Vec<Candidate> = cities
            .into_iter()
            .filter(|(id, _)| seen.insert(*id))
            .filter_map(|(id, committed)| {
                let city = v.cities.iter().find(|c| c.id == id)?;
                if city.owner.is_barbarian() || v.same_team(v.me, city.owner) {
                    return None;
                }
                let team = v.team_of(city.owner);
                if !v.at_war(v.me, city.owner) && !legal_declaration(v, team) {
                    return None;
                }
                // Gene `stale_intel`: no new war on a picture too old to trust; a war already
                // on prices the city as it has probably grown.
                if !v.at_war(v.me, city.owner) && self.stale_city(city) {
                    return None;
                }
                let aged = self.aged_city(city);
                let city = aged.as_ref().unwrap_or(city);
                let f = self
                    .funded_forecast(city, committed.as_deref())
                    .or_else(|| {
                        committed
                            .as_deref()
                            .and_then(|c| self.grind_forecast(city, c))
                    })
                    .or_else(|| {
                        (city.walls && v.at_war(v.me, city.owner))
                            .then(|| self.campaign_force(city))
                            .and_then(|force| self.grind_forecast(city, &force))
                    })
                    .or_else(|| self.wall_staging_forecast(city));
                let f = f?;
                Some(self.price_campaign(base, c0, city, &f))
            })
            .collect();
        let reaim = self.reaim_candidates(base, c0, &out);
        out.extend(reaim);
        out
    }

    /// Gene `war_reaim`: the enemy, and the city, of the plan's campaign when that campaign has
    /// just ended for want of an army or a target, or its city dropped off the candidates while
    /// the war goes on. Not after a capture (the army holds the city first).
    fn reaim_enemy(&self, open: &[Candidate]) -> Option<(TeamId, CityId)> {
        if self.decisions.war_reaim == 0
            || !self.hard_defense()
            || self.takes_no_cities()
            || self.campaigns.held_capture.is_some()
        {
            return None;
        }
        let v = self.view();
        let Some(PlanKind::Campaign { against, target }) = self.routed_plan().map(|p| &p.kind)
        else {
            return None;
        };
        if !v.teams_at_war(v.team, *against)
            || !self.persists_against(*against)
            || v.cities
                .iter()
                .any(|c| c.id == *target && v.same_team(v.me, c.owner))
        {
            return None;
        }
        let ended = self
            .campaigns
            .events
            .iter()
            .rev()
            .take_while(|e| e.turn == v.turn)
            .any(|e| {
                e.target == *target
                    && matches!(
                        e.end,
                        Some(
                            CampaignEnd::InsufficientArmy
                                | CampaignEnd::WallsBeforeArrival
                                | CampaignEnd::NegativeValue
                                | CampaignEnd::TargetGone
                        )
                    )
            });
        let dropped = !open
            .iter()
            .any(|c| matches!(&c.kind, PlanKind::Campaign { target: t, .. } if t == target));
        (ended || dropped).then_some((*against, *target))
    }

    /// Gene `war_reaim`: the strength of one defender the owner of `city` could buy there. Rival
    /// treasuries are hidden, so every owner is assumed able to buy one unit per city per turn,
    /// of the strongest kind it is seen fielding (a Soldier when none is in sight).
    fn bought_defender(&self, city: &CityView) -> Option<UnitView> {
        let v = self.view();
        let mut unit = match v
            .units
            .iter()
            .filter(|u| u.owner == city.owner && u.military && u.kind != UnitKind::Scout)
            .max_by_key(|u| (u.strength.max(u.ranged), std::cmp::Reverse(u.id)))
        {
            Some(u) => u.clone(),
            None => {
                let mut u = (*self.ctx.own_units.first()?).clone();
                let stats = v
                    .rule_experiments
                    .unit_stats_for(v.faction_of(city.owner), UnitKind::Soldier);
                u.owner = city.owner;
                u.kind = UnitKind::Soldier;
                u.strength = stats.strength;
                u.ranged = 0;
                u.range = 0;
                u.level = 0;
                u
            }
        };
        unit.id = UnitId(u32::MAX - 1);
        unit.pos = city.pos;
        unit.hp = data::UNIT_HP;
        unit.fortify = 0;
        unit.goto = None;
        unit.path.clear();
        Some(unit)
    }

    /// Gene `war_reaim` 2: the view in which an empty `city` holds the defender its owner could
    /// buy there; None when a garrison already stands in it.
    pub(crate) fn with_bought_defender(&self, city: &CityView) -> Option<PlayerView> {
        let v = self.view();
        if v.units
            .iter()
            .any(|u| u.pos == city.pos && u.military && u.owner == city.owner)
        {
            return None;
        }
        let mut defended = v.clone();
        defended.units.push(self.bought_defender(city)?);
        Some(defended)
    }

    /// Gene `war_reaim`: re-aim, don't quit. When the plan's campaign on an enemy ends or loses
    /// its target, the same enemy's other cities within the march window are ranked by value
    /// (population, a capital) over defence (the city's defence weight plus one bought defender)
    /// and march time, and the best few are priced as campaigns. The war on this enemy is
    /// already open, so the march counts even while another war is on. Peace wins only when
    /// none of them is winnable (capture odds under `min_seat_war_odds`).
    fn reaim_candidates(&self, base: &WinFacts, c0: i32, open: &[Candidate]) -> Vec<Candidate> {
        let Some((against, from)) = self.reaim_enemy(open) else {
            return Vec::new();
        };
        let v = self.view();
        let army: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout && u.hp >= 30)
            .collect();
        if !army.iter().any(|u| crate::composition::charges(u.kind)) {
            return Vec::new();
        }
        let mut ranked: Vec<(i32, &CityView)> = v
            .cities
            .iter()
            .filter(|c| c.id != from && !c.owner.is_barbarian() && v.team_of(c.owner) == against)
            .filter(|c| {
                !open.iter().any(
                    |o| matches!(&o.kind, PlanKind::Campaign { target, .. } if *target == c.id),
                )
            })
            .filter_map(|c| {
                let travel = army
                    .iter()
                    .map(|u| {
                        let moves = u.max_moves.max(1);
                        (u.pos.distance(c.pos) + moves - 1) / moves
                    })
                    .min()?;
                if travel > self.decisions.march_turns {
                    return None;
                }
                let garrisoned = v
                    .units
                    .iter()
                    .any(|u| u.pos == c.pos && u.military && u.owner == c.owner);
                let bought = self
                    .bought_defender(c)
                    .map_or(0, |u| crate::analysis::threat_power(v, &u));
                let defence =
                    self.city_defence_weight(c) + if garrisoned { bought / 2 } else { bought };
                let worth = 100 + 25 * c.population + if c.is_capital { 200 } else { 0 };
                Some((worth * 1000 / defence.max(1) - 100 * travel, c))
            })
            .collect();
        ranked.sort_by_key(|(score, c)| (std::cmp::Reverse(*score), c.id));
        let floor = self.runway_floor();
        ranked
            .into_iter()
            .take(FRONT_TARGETS)
            .filter_map(|(_, city)| {
                let aged = self.aged_city(city);
                let city = aged.as_ref().unwrap_or(city);
                let defended = if self.decisions.war_reaim >= 2 {
                    self.with_bought_defender(city)
                } else {
                    None
                };
                let f = self.campaign_forecast_with(city, None, true, defended.as_ref())?;
                if self
                    .gold_runway()
                    .is_some_and(|r| r < floor && f.turns.max(0) as u32 > r)
                    || capture_odds(f.turns, f.march) < self.decisions.min_seat_war_odds
                {
                    return None;
                }
                let mut c = self.price_campaign(base, c0, city, &f);
                c.terms.push(("re-aimed after the last campaign".into(), 0));
                Some(c)
            })
            .collect()
    }

    /// Containment: wars priced by the VP they deny a rival, for the public leader or the
    /// conqueror the threat model names.
    fn contain_candidates(&self, base: &WinFacts, c0: i32) -> Vec<Candidate> {
        let v = self.view();
        let mut teams: Vec<TeamId> = Vec::new();
        if let Some(t) = self.diplomatic_target().filter(|t| *t != v.team) {
            teams.push(t);
        }
        // The public VP leader well on its way, before a coalition rule or a hold names it:
        // pillar 11's rubber band starts before the countdown (playtest 8.4 India: nobody
        // struck the leader's VP before its hold began).
        if let Some(t) = self.contain_leader()
            && !teams.contains(&t)
        {
            teams.push(t);
        }
        if self.decisions.leader_war > 0
            && let Some(t) = self.urgent_vp_contender()
            && !teams.contains(&t)
        {
            teams.push(t);
        }
        // Every team counting down to victory is a public target (`hold.rs`).
        for hold in crate::hold::holds(v).iter().filter(|h| h.team != v.team) {
            if !teams.contains(&hold.team) {
                teams.push(hold.team);
            }
        }
        // A containment war in progress stays on offer while it lasts, even when the public
        // target moves on; the estimate then decides whether to keep fighting it.
        // Gene `contain_after_campaign`: so does the war a running Campaign fights, so a campaign
        // that ends falls back to containing its enemy and not to Peace priced at zero.
        let fought = match self.routed_plan().map(|p| &p.kind) {
            Some(PlanKind::Contain { against }) => Some(against),
            Some(PlanKind::Campaign { against, .. })
                if self.decisions.contain_after_campaign > 0 =>
            {
                Some(against)
            }
            _ => None,
        };
        if let Some(against) = fought
            && v.teams_at_war(v.team, *against)
            && !teams.contains(against)
        {
            teams.push(*against);
        }
        let h = self.plan_horizon();
        teams
            .into_iter()
            .filter(|t| alive(v, *t) && *t != v.team)
            .filter_map(|t| {
                let at_war = v.teams_at_war(v.team, t);
                let progress = v
                    .standings
                    .iter()
                    .find(|s| s.team == t)
                    .map_or(0, |s| 100 * s.vp / s.target.max(1));
                let breach = !at_war
                    && !legal_declaration(v, t)
                    && (progress >= 90 || crate::hold::rival_hold_of(v, t).is_some())
                    && v.diplomacy.actions.iter().any(|a| {
                        a.blocked_reason.is_none()
                            && a.order
                                == Order::DeclareWar {
                                    against: t,
                                    breach: true,
                                }
                    });
                if !at_war && !legal_declaration(v, t) && !breach {
                    return None;
                }
                // A new containment declaration needs a funded route to a city that the
                // normal siege forecast can take. Otherwise the plan would declare first,
                // then find no field campaign (or abort it on arrival).
                // Gene `leader_commit` asks the same of every faction (user, 2026-10-17: "the
                // AIs should commit to war if they're going to declare it").
                if !at_war
                    && (self.faction == Faction::Sparta
                        || self.sack_commits()
                        || self.decisions.leader_commit >= 2)
                    && self.decisions.contain_commit > 0
                    && self.decisions.contain_campaign > 0
                    && !self.busy_leader_front(t)
                    && !self
                        .campaign_against(t)
                        .is_ok_and(|outlook| outlook.odds >= self.decisions.contain_min_odds)
                {
                    return None;
                }
                // No new long war on a runway under the floor, unless the enemy wins otherwise.
                let holder = crate::hold::rival_hold_of(v, t).is_some();
                if !at_war && !holder && self.gold_runway().is_some_and(|r| r < self.runway_floor())
                {
                    return None;
                }
                // Our field force needs a known front. Other empires can help only
                // when they are already at War and have visible troops reaching one.
                let army_near = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| u.military && u.kind != UnitKind::Scout)
                    .flat_map(|u| {
                        v.cities
                            .iter()
                            .filter(|c| v.team_of(c.owner) == t)
                            .map(move |c| c.pos.distance(u.pos))
                    })
                    .min();
                let army_reach = army_near.map_or(0, |d| (15 - d).clamp(0, 10));
                let own_force = strength(v, v.team) * army_reach / 10;
                let theirs = self.war_side_strength(t, v.team, false).max(1);
                let coalition: i32 = if self.decisions.coalition_push > 0 {
                    own_force
                        + v.standings
                            .iter()
                            .filter(|s| {
                                alive(v, s.team) && s.team != v.team && v.teams_at_war(s.team, t)
                            })
                            // Public army is useful only if that empire has a visible front
                            // near a city of the leader. A remote or unseen army cannot be
                            // borrowed to justify our own costly declaration.
                            .map(|s| {
                                let distance = v
                                    .units
                                    .iter()
                                    .filter(|u| {
                                        u.military
                                            && u.kind != UnitKind::Scout
                                            && v.team_of(u.owner) == s.team
                                    })
                                    .flat_map(|u| {
                                        v.cities
                                            .iter()
                                            .filter(|c| v.team_of(c.owner) == t)
                                            .map(move |c| u.pos.distance(c.pos))
                                    })
                                    .min()
                                    .unwrap_or(99);
                                strength(v, s.team) * (15 - distance).clamp(0, 10) / 10
                            })
                            .sum::<i32>()
                } else {
                    let own_side = v.diplomacy.side(v.team);
                    self.war_side_strength(v.team, t, true)
                        + v.standings
                            .iter()
                            .filter(|s| {
                                alive(v, s.team)
                                    && !own_side.contains(&s.team)
                                    && v.teams_at_war(s.team, t)
                            })
                            .map(|s| strength(v, s.team))
                            .sum::<i32>()
                };
                // The war slows their race only where armies can meet: full effect within 5
                // hexes of their nearest known city, none beyond 15.
                let ratio = (coalition * 100 / theirs).min(100);
                let old_army_near = if holder || self.decisions.contain_campaign > 0 {
                    army_near
                } else {
                    None
                };
                let distance = crate::diplomacy::nearest_known(v, t).unwrap_or(99).min(
                    if self.decisions.coalition_push > 0 {
                        army_near
                    } else {
                        old_army_near
                    }
                    .unwrap_or(99),
                );
                let reach = (15 - distance).clamp(0, 10);
                let ours = strength(v, v.team).max(1);
                let mut deny = self.war_footing(base, h, 30);
                let c_footing = deny.chance();
                adjust(&mut deny, t, |f| {
                    // 2% of the target per horizon at full strength and reach (an assumption,
                    // not yet measured).
                    f.vp = (f.vp - f.target * 2 * ratio * reach / 100_000).max(0);
                });
                // Stopping the leader reopens a race for every survivor. Our share of
                // that reopened race has a floor even when today's solo win chance is
                // zero; otherwise a coalition capable of breaking a hold is priced at
                // zero by the empire with the army best placed to do it.
                let contribution = own_force * 100 / coalition.max(1);
                let race = self.leader_race(
                    c_footing,
                    &deny,
                    t,
                    ratio,
                    if self.decisions.coalition_push > 0 {
                        army_reach
                    } else {
                        reach
                    },
                    contribution,
                );
                let attrition =
                    base.team(v.team).map_or(0, |f| f.army) * (theirs * 100 / ours).min(200) / 2000;
                let mut worn = deny.clone();
                adjust(&mut worn, v.team, |f| f.army -= attrition);
                let mut terms = vec![
                    ("VP denied".into(), deny.chance() - c_footing),
                    (LEADER_RACE.into(), race),
                    ("war footing".into(), c_footing - c0),
                    ("attrition".into(), worn.chance() - deny.chance()),
                ];
                if let prize @ 1.. = self.busy_front_prize(t) {
                    terms.push(("exposed leader city".into(), prize));
                }
                // Gene `coalition_front`: another member already striking the target on a
                // separate front splits its defence, so our own war takes more from its race
                // in the same window (user, 2026-10-17: "attack all at once and open multiple
                // fronts"). Only what we see: their units beside the target's cities.
                if self.decisions.coalition_front > 0 && race > 0 {
                    let fronts = self.coalition_fronts(t);
                    if fronts > 0 {
                        terms.push(("joint strike".into(), race * fronts.min(2) / 2));
                    }
                }
                if holder && self.decisions.vp_hold_break > 0 {
                    // War without a reachable front cannot break a hold. An army already in
                    // range can force a held goal loss inside the countdown.
                    let remaining =
                        crate::hold::rival_hold_of(v, t).map_or(10, |h| h.turns_left as i32);
                    let front = (remaining * 2
                        - if self.decisions.coalition_push > 0 {
                            army_near.unwrap_or(99)
                        } else {
                            distance
                        })
                    .clamp(0, 10);
                    let pressure = ratio * front * self.decisions.vp_hold_break / 100
                        * if self.decisions.coalition_push > 0 {
                            contribution
                        } else {
                            100
                        }
                        / 100;
                    terms.push(("contest the hold".into(), pressure));
                }
                let q = self.retaliation_permille(t, &[]);
                if q > 0 {
                    let hit = self.city_lost(&deny, t);
                    terms.push((
                        "retaliation".into(),
                        q * (hit.chance() - deny.chance()) / 1000,
                    ));
                }
                self.era_terms(t, false, &mut terms);
                self.peace_credibility_term(t, &mut terms);
                self.war_pressure_term(t, &mut terms);
                let net = v.economy.gold_per_turn.total;
                Some(Candidate {
                    kind: PlanKind::Contain { against: t },
                    value: terms.iter().map(|(_, x)| x).sum(),
                    terms,
                    reserve: if net < 0 {
                        (-net * h).min(v.economy.gold)
                    } else {
                        0
                    },
                    breach,
                })
            })
            .collect()
    }

    /// The team a routed containment war is fighting, when it marches (`contain_campaign`):
    /// it funds a field force like a campaign (`production.rs::desired_army`).
    pub(crate) fn contain_war(&self) -> Option<TeamId> {
        if self.decisions.contain_campaign == 0 || self.takes_no_cities() {
            return None;
        }
        match self.routed_plan().map(|p| &p.kind) {
            Some(PlanKind::Contain { against })
                if self.view().teams_at_war(self.view().team, *against) =>
            {
                Some(*against)
            }
            _ => None,
        }
    }

    /// The strict public VP leader at `contain_leader_percent` of its target or more (Hard).
    pub(crate) fn contain_leader(&self) -> Option<TeamId> {
        let mut percent = self.decisions.contain_leader_percent;
        if percent <= 0 {
            return None;
        }
        let v = self.view();
        // Gene `runaway_read`: a strict leader showing the public signs of an economic runaway
        // is containable from 30% of its target.
        if self.decisions.runaway_read > 0
            && v.standings
                .iter()
                .any(|s| s.team != v.team && crate::threat::runaway_signs(v, s.team, 0) >= 20)
        {
            percent = percent.min(30);
        }
        let lead = v
            .standings
            .iter()
            .filter(|s| alive(v, s.team))
            .max_by_key(|s| (s.vp * 100 / s.target.max(1), std::cmp::Reverse(s.team)))?;
        let strict = v.standings.iter().all(|s| {
            s.team == lead.team
                || !alive(v, s.team)
                || s.vp * 100 / s.target.max(1) < lead.vp * 100 / lead.target.max(1)
        });
        if lead.team != v.team && strict && (lead.vp * 100 / lead.target.max(1)) as i32 >= percent {
            return Some(lead.team);
        }
        self.hollow_leader()
    }

    /// A rival close enough to the public VP lead to finish a held goal before
    /// the next campaign pays. The force gate still decides whether to turn.
    fn urgent_vp_contender(&self) -> Option<TeamId> {
        let v = self.view();
        let highest = v.standings.iter().map(|s| s.vp).max()?;
        v.standings
            .iter()
            .filter(|s| {
                s.team != v.team
                    && alive(v, s.team)
                    && s.vp * 5 >= s.target * 2
                    && s.vp + 2 >= highest
            })
            .max_by_key(|s| (s.vp, std::cmp::Reverse(s.team)))
            .map(|s| s.team)
    }

    /// Gene `hollow_read`: the rich, empty empire at or near the top of the VP race (nobody more
    /// than a tenth of the target ahead of it, at least a fifth of its own target), with
    /// runaway signs of 20 or more counting its hollowness (`threat::runaway_signs`). It is
    /// containable without a strict VP lead (8.9.15 Swiss review rec 2).
    pub(crate) fn hollow_leader(&self) -> Option<TeamId> {
        if self.decisions.hollow_read == 0 || self.decisions.runaway_read == 0 {
            return None;
        }
        let v = self.view();
        v.standings
            .iter()
            .filter(|s| s.team != v.team && alive(v, s.team) && s.vp * 100 >= s.target * 20)
            .filter(|s| crate::threat::hollow_bonus_from(v, s.team, self.hollow_floor(s.team)) > 0)
            .map(|s| {
                (
                    crate::threat::runaway_signs(v, s.team, self.hollow_floor(s.team)),
                    s.team,
                )
            })
            .filter(|(signs, _)| *signs >= 20)
            .max_by_key(|(signs, team)| (*signs, std::cmp::Reverse(*team)))
            .map(|(_, team)| team)
    }

    /// Gene `persist_hollow`: `team` reads as a hollow leader (`threat::hollow_bonus`: its
    /// public army per city at most half the median empire's, and rich in public), so a war on
    /// it is worth keeping after one bought defender. Hollowness alone is not enough: a Sparta
    /// of four veterans reads 55 to 74 hollow at Japan T142, and Japan's Peace offers were its
    /// defence there.
    pub(crate) fn persists_against(&self, team: TeamId) -> bool {
        self.decisions.persist_hollow == 0
            || crate::threat::hollow_bonus_from(self.view(), team, self.hollow_floor(team)) > 0
    }

    /// The hollowness floor for `team` (`threat::HOLLOW_KEEP` once our tracker reads it hollow).
    pub(crate) fn hollow_floor(&self, team: TeamId) -> i32 {
        self.opponents.map_or(crate::threat::HOLLOW_ENTER, |m| {
            m.threats.hollow_floor(team)
        })
    }

    /// Our share of the leader chance a coalition can remove. The future race
    /// share has an equal-survivor floor: stopping a near-certain winner is a
    /// route back into contention even when our current solo chance rounds to 0.
    /// Our public contribution to the combined force prices a weak partner's
    /// temptation to declare on a much stronger neighbor just to ride the coalition.
    fn leader_race(
        &self,
        ours: i32,
        facts: &WinFacts,
        team: TeamId,
        ratio: i32,
        reach: i32,
        contribution: i32,
    ) -> i32 {
        let share = self.decisions.contain_share;
        if share <= 0 {
            return 0;
        }
        let mut theirs = facts.clone();
        theirs.me = team;
        theirs.runway_short = 0;
        let p = theirs.chance().clamp(0, 10_000) as i64;
        if self.decisions.coalition_push == 0 {
            let p = p.min(9_000);
            let taken = share as i64 * ratio as i64 * reach as i64 / 1000;
            return (ours as i64 * taken * p / ((10_000 - p) * 1000)) as i32;
        }
        let survivors = facts
            .teams
            .iter()
            .filter(|f| f.team != team && f.cities > 0)
            .count();
        let future_share = ours.max(10_000 / survivors.max(1) as i32).min(10_000) as i64;
        // Permille of the leader's chance removed, then our plausible share of
        // that released probability. All factors use only the current PlayerView.
        let removed = p * share as i64 * ratio as i64 * reach as i64 / 1_000_000;
        (removed * future_share * contribution as i64 / 1_000_000) as i32
    }

    /// An exposed rival Settler at plain Peace that one of our melee units can reach in at most
    /// two moves: no owner military beside it, not in a city, a legal surprise declaration.
    pub(crate) fn snipe_targets(&self) -> Vec<(&UnitView, &UnitView)> {
        if self.decisions.settler_raid == 0 {
            return Vec::new();
        }
        let v = self.view();
        let mut pairs = Vec::new();
        for s in v.units.iter().filter(|s| {
            s.kind == UnitKind::Settler
                && !s.owner.is_barbarian()
                && !v.same_team(v.me, s.owner)
                && !v.at_war(v.me, s.owner)
        }) {
            let team = v.team_of(s.owner);
            if !legal_declaration(v, team)
                || v.cities.iter().any(|c| c.pos == s.pos)
                || v.units.iter().any(|u| {
                    u.military && v.same_team(u.owner, s.owner) && u.pos.distance(s.pos) <= 1
                })
                || self.ctx.move_cost(s.pos).is_none()
            {
                continue;
            }
            for u in self.ctx.own_units.iter().copied().filter(|u| {
                let reach = if self.decisions.settler_raid_range == 0 {
                    1
                } else {
                    u.moves_left.saturating_add(
                        u.max_moves
                            .saturating_mul(self.decisions.settler_raid_range - 1),
                    )
                };
                let geometric_reach = if self.decisions.settler_raid_range == 0 {
                    1
                } else {
                    u.max_moves
                        .saturating_mul(self.decisions.settler_raid_range)
                };
                let path_cost = self
                    .view()
                    .index(s.pos)
                    .map(|target| self.ctx.distance_field(u.pos, false)[target])
                    .unwrap_or(crate::analysis::UNREACHABLE);
                // Gene `settler_stalk`: a declaration lands only on contact. A unit that
                // starts the turn beside the Settler pins it (`movement.rs`, contact pins);
                // from two hexes a moving Settler walks away from an equally fast pursuer
                // (8.9.24: Hard in the Swiss seat declared at T41 in 48 of 48 runs with a
                // Scout three hexes off and took nothing). Farther Settlers are stalked at
                // Peace until one is adjacent, as the designer did.
                let contact = !self.stalk_contact()
                    || (u.pos.distance(s.pos) == 1 && self.ctx.reachable(u.pos, s.pos));
                contact
                    && u.military
                    && crate::composition::charges(u.kind)
                    // An Sparta opening with Byzantium in the field is already exposed
                    // to Scout pressure. A speculative two-turn raid by another empire
                    // can pull it into a second war before its city strike assembles.
                    // Keep immediate non-Scout captures, as in the original snipe rule.
                    && (v.faction_of(s.owner) != Faction::Sparta
                        || !v.players.iter().any(|p| p.alive && p.faction == Faction::Byzantium)
                        || (u.kind != UnitKind::Scout
                            && u.pos.distance(s.pos) == 1
                            && self.ctx.reachable(u.pos, s.pos)))
                    && u.hp >= 50
                    && u.max_moves > 0
                    && !self.has_acted(u.id)
                    && (path_cost <= reach || u.pos.distance(s.pos) <= geometric_reach)
            }) {
                pairs.push((u, s));
            }
        }
        pairs.sort_by_key(|(u, s)| (s.id, u.hp < 100, u.id));
        pairs.dedup_by_key(|(_, s)| s.id);
        pairs
    }

    /// Gene `stalk_contact` (with `settler_stalk` on): a Snipe declares only on contact.
    pub(crate) fn stalk_contact(&self) -> bool {
        self.decisions.settler_stalk > 0 && self.decisions.stalk_contact > 0
    }

    /// Gene `settler_stalk`: close on an exposed rival Settler at Peace that is beyond this
    /// turn's snipe reach but within `settler_stalk` turns, without declaring, so the
    /// Snipe plan can price the war once the capture is in reach (the designer's 8.9.23
    /// Byzantine Settler, four hexes from his capital, taken three turns after he went for
    /// it). Only what the AI sees: the Settler must stand in view, outside a city, with no
    /// owner troop beside it and a legal declaration. The stalker is our Scout or a field
    /// troop outside a city, and only while no own city is threatened, so no defender
    /// leaves; at most one per Settler. Moving next to a unit at Peace starts no war.
    pub(crate) fn stalk_settlers(&mut self) {
        let turns = self.decisions.settler_stalk;
        if turns == 0
            || self.decisions.settler_raid == 0
            || !self.hard_defense()
            || self.ctx.threat.iter().any(|t| *t > 0)
            || matches!(
                self.routed_plan().map(|p| &p.kind),
                Some(PlanKind::Snipe { .. })
            )
        {
            return;
        }
        let v = self.view();
        let mut stalked: Vec<legatum_core::UnitId> = Vec::new();
        let mut orders = Vec::new();
        let contact = self.stalk_contact();
        let quiet_garrison = |u: &UnitView| {
            self.ctx
                .own_cities
                .iter()
                .position(|c| c.pos == u.pos)
                .is_none_or(|i| self.quiet_city(i))
        };
        for s in v.units.iter().filter(|s| {
            s.kind == UnitKind::Settler
                && !s.owner.is_barbarian()
                && !v.same_team(v.me, s.owner)
                && !v.at_war(v.me, s.owner)
                && legal_declaration(v, v.team_of(s.owner))
                && v.tile(s.pos).is_some_and(|t| t.visible)
                && !v.cities.iter().any(|c| c.pos == s.pos)
                && !v.units.iter().any(|u| {
                    u.military && v.same_team(u.owner, s.owner) && u.pos.distance(s.pos) <= 1
                })
        }) {
            // A tile beside the Settler that is free to stand on, the side away from its
            // owner's nearest known city first: a Settler walks out from home, so a stalker
            // ahead of it stays in contact when it moves on.
            let home = v
                .cities
                .iter()
                .filter(|c| c.owner == s.owner)
                .map(|c| c.pos)
                .min_by_key(|c| (c.distance(s.pos), *c));
            let mut besides: Vec<usize> = s
                .pos
                .neighbors()
                .into_iter()
                .filter(|h| {
                    !v.units.iter().any(|o| o.pos == *h) && !v.cities.iter().any(|c| c.pos == *h)
                })
                .filter_map(|h| v.index(h))
                .collect();
            if contact {
                besides.sort_by_key(|&i| {
                    let h = self.ctx.hex_at(i);
                    (std::cmp::Reverse(home.map_or(0, |c| c.distance(h))), h)
                });
            }
            let best = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| {
                    u.military
                        && crate::composition::charges(u.kind)
                        && u.hp >= 50
                        && u.max_moves > 0
                        && !self.has_acted(u.id)
                        && !stalked.contains(&u.id)
                        && if contact {
                            u.pos.distance(s.pos) > 1 && quiet_garrison(u)
                        } else {
                            !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                        }
                })
                .filter_map(|u| {
                    let field = self.ctx.distance_field(u.pos, false);
                    if !contact {
                        let (cost, at) = besides.iter().map(|&i| (field[i], i)).min()?;
                        // Beyond what the Snipe plan already reaches (one more step after
                        // this turn's moves), within the stalk horizon.
                        return (cost < crate::analysis::UNREACHABLE
                            && cost > u.moves_left
                            && cost <= u.max_moves.max(1) * turns)
                            .then_some((0, cost, u, at));
                    }
                    let moves = self.turn_moves(u);
                    // Within the stalk horizon; among the tiles it reaches soonest, the one
                    // ahead of the Settler.
                    let soonest = besides
                        .iter()
                        .map(|&i| field[i])
                        .filter(|c| *c < crate::analysis::UNREACHABLE)
                        .map(|c| (c + moves - 1) / moves)
                        .min()?;
                    let (cost, at) = besides.iter().map(|&i| (field[i], i)).find(|(c, _)| {
                        *c < crate::analysis::UNREACHABLE && (c + moves - 1) / moves == soonest
                    })?;
                    (cost <= moves * turns).then_some((soonest, cost, u, at))
                })
                .min_by_key(|(soonest, cost, u, _)| (*soonest, *cost, u.id))
                .map(|(_, cost, u, at)| (cost, u, at));
            let Some((_, u, at)) = best else {
                continue;
            };
            stalked.push(u.id);
            orders.push((u.id, self.ctx.hex_at(at)));
        }
        for (unit, to) in orders {
            self.unit_order(unit, Order::MoveUnit { unit, to });
        }
    }

    fn snipe_candidates(&self, base: &WinFacts, c0: i32) -> Vec<Candidate> {
        let v = self.view();
        self.snipe_targets()
            .into_iter()
            .map(|(u, s)| {
                let team = v.team_of(s.owner);
                // Their Settler founds under every other plan; ours does under this one.
                let extent = self.projects_extent(base);
                let mut theirs = base.clone();
                adjust(&mut theirs, team, |t| {
                    t.cities += 8;
                    if extent {
                        t.territory += 56;
                    }
                });
                let common = self.war_footing(base, 1, 0);
                let mut ours = common.clone();
                adjust(&mut ours, v.team, |t| {
                    t.cities += 8;
                    if extent {
                        t.territory += 56;
                    }
                });
                let q = self.retaliation_permille(team, &[u.id]);
                let mut terms = vec![(
                    "a city and its denial".into(),
                    90 * (ours.chance() - theirs.chance()) / 100,
                )];
                if q > 0 {
                    let hit = self.city_lost(&common, team);
                    terms.push((
                        "retaliation".into(),
                        q * (hit.chance() - common.chance()) / 1000,
                    ));
                }
                self.new_war_terms(team, c0, &mut terms);
                Candidate {
                    kind: PlanKind::Snipe {
                        against: team,
                        settler: s.id,
                        by: u.id,
                    },
                    value: terms.iter().map(|(_, x)| x).sum(),
                    terms,
                    reserve: 0,
                    breach: false,
                }
            })
            .collect()
    }

    /// A local field battle is a strategic objective when it can remove a force before that
    /// force attacks our city. Forecast on a prospective war view so Peace units defend, then
    /// price the damage in the same win-chance currency as other plans.
    fn field_strike_candidates(&self, base: &WinFacts, c0: i32) -> Vec<Candidate> {
        if self.decisions.field_strike == 0 || !self.hard_defense() {
            return Vec::new();
        }
        let v = self.view();
        let mut out = Vec::new();
        for target in v.units.iter().filter(|u| {
            u.military
                && !u.owner.is_barbarian()
                && !v.same_team(v.me, u.owner)
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|c| c.pos.distance(u.pos) <= 3)
        }) {
            let team = v.team_of(target.owner);
            if !v.teams_at_war(v.team, team) && !legal_declaration(v, team) {
                continue;
            }
            let nearby: Vec<_> = v
                .units
                .iter()
                .filter(|u| {
                    u.military && v.team_of(u.owner) == team && u.pos.distance(target.pos) <= 2
                })
                .collect();
            if nearby.len() < 2 {
                continue;
            }
            let mut prospective = v.clone();
            if !v.teams_at_war(v.team, team) {
                let Some(relation) = prospective
                    .diplomacy
                    .relations
                    .iter_mut()
                    .find(|r| r.between(v.team, team))
                else {
                    continue;
                };
                relation.state = RelationState::War;
            }
            let best = self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && !self.has_acted(u.id))
                .filter_map(|u| {
                    let mode = if u.ranged > 0 {
                        AttackMode::Ranged
                    } else {
                        AttackMode::Melee
                    };
                    let f = forecast_attack_mode(&prospective, u.id, target.pos, mode)?;
                    if mode == AttackMode::Melee && u.pos.distance(target.pos) != 1 {
                        return None;
                    }
                    let exchange = f.expected_damage_to_defender - f.expected_damage_to_attacker;
                    (exchange > 0).then_some((exchange, u.id, f))
                })
                .max_by_key(|(exchange, id, _)| (*exchange, std::cmp::Reverse(*id)));
            let Some((exchange, by, forecast)) = best else {
                continue;
            };
            let enemy_power: i32 = nearby
                .iter()
                .map(|u| crate::analysis::threat_power(v, u))
                .sum();
            let our_power: i32 = self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.pos.distance(target.pos) <= 2)
                .map(|u| crate::analysis::defense_power(v, u))
                .sum();
            if our_power < enemy_power || our_power == 0 {
                continue;
            }
            let city_loss = (c0 - self.city_lost(base, team).chance()).max(0);
            let pressure = enemy_power * 1000 / (enemy_power + our_power).max(1);
            let mut terms = vec![(
                "field army stopped at city".into(),
                city_loss * pressure / 1000 * exchange.min(100) / 100 * self.decisions.field_strike
                    / 100,
            )];
            // Destroying public army strength helps us and denies it to the rival. Use the
            // forecast damage share; wounded survivors recover, so only half is lasting.
            let mut attrition = base.clone();
            adjust(&mut attrition, team, |t| {
                let army_power: i32 = nearby.iter().map(|u| crate::hold::army_value(u)).sum();
                t.army -= (forecast.expected_damage_to_defender * army_power / 200).max(1);
            });
            terms.push((
                "field attrition".into(),
                (attrition.chance() - c0).max(0) * self.decisions.field_strike / 100,
            ));
            let q = self.retaliation_permille(team, &[by]);
            if q > 0 {
                terms.push((
                    "retaliation".into(),
                    q * (self.city_lost(base, team).chance() - c0) / 1000,
                ));
            }
            self.new_war_terms(team, c0, &mut terms);
            out.push(Candidate {
                kind: PlanKind::FieldStrike {
                    against: team,
                    target: target.id,
                    by,
                },
                value: terms.iter().map(|(_, value)| value).sum(),
                terms,
                reserve: 0,
                breach: false,
            });
        }
        out
    }

    /// The no-war plan: Economy at peace, or Peace with the enemy whose war is most
    /// dangerous.
    fn quiet_candidate(&self, base: &WinFacts, c0: i32) -> Candidate {
        let v = self.view();
        let enemy = v
            .players
            .iter()
            .filter(|p| p.alive && p.team != v.team && v.teams_at_war(v.team, p.team))
            .map(|p| p.team)
            .max_by_key(|t| {
                (
                    self.retaliation_permille(*t, &[]),
                    strength(v, *t),
                    std::cmp::Reverse(*t),
                )
            });
        let mut terms = Vec::new();
        // Gene `peace_retaliation`: suing for peace doesn't stop the enemy army at the gate
        // unless the enemy accepts. Peace carries the same retaliation as fighting on, times
        // the chance the offer is refused: certain when the enemy declared this war, refused our
        // last offer or is our top threat, otherwise even.
        if self.decisions.peace_retaliation > 0
            && let Some(t) = enemy
        {
            let q = self.retaliation_permille(t, &[]);
            if q > 0 {
                let declared = v
                    .diplomacy
                    .relations
                    .iter()
                    .any(|r| r.between(v.team, t) && r.declarers.contains(&t));
                let refused = v.diplomacy.peace_cooldowns.iter().any(|c| {
                    c.from == v.team
                        && c.to == t
                        && c.kind == legatum_core::diplomacy::TreatyKind::Peace
                        && v.turn < c.retry_turn
                });
                let refuse_permille = if declared || refused || self.top_threat_team() == Some(t) {
                    1000
                } else {
                    500
                };
                let hit = self.city_lost(base, t);
                terms.push((
                    "retaliation".into(),
                    q * (hit.chance() - c0) / 1000 * refuse_permille / 1000,
                ));
            }
        }
        if self.decisions.rival_redeclare_percent > 0
            && let Some(t) = enemy
        {
            let q = self.retaliation_permille(t, &[]);
            let horizon = self.plan_horizon();
            let unprotected = (horizon - legatum_core::diplomacy::PEACE_TRUCE_TURNS as i32).max(0);
            let probability = self.rival_redeclare_permille(t) * unprotected / horizon;
            if q > 0 && probability > 0 {
                let hit = self.city_lost(base, t);
                terms.push((
                    "rival re-declares after Peace".into(),
                    q * (hit.chance() - c0) / 1000 * probability / 1000,
                ));
            }
        }
        Candidate {
            kind: enemy.map_or(PlanKind::Economy, |with| PlanKind::Peace { with }),
            value: terms.iter().map(|(_, x)| x).sum(),
            terms,
            reserve: 0,
            breach: false,
        }
    }

    /// Every plan open this turn, priced, in a stable order.
    pub(crate) fn plan_candidates(&self) -> Vec<Candidate> {
        let mut base = WinFacts::of(self.view())
            .with_relative(&self.decisions)
            .refined();
        base.fitted = self.search.is_some_and(|h| h.config.fitted_plan);
        let c0 = base.chance();
        let mut out = vec![self.quiet_candidate(&base, c0)];
        out.extend(self.campaign_candidates(&base, c0));
        if self.decisions.sparta_force_campaign > 0
            && self.decisions.sparta_valued_expedition == 0
            && !out
                .iter()
                .any(|c| matches!(&c.kind, PlanKind::Campaign { .. }))
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout && u.hp >= 30)
                .count()
                >= 3
            && let Some((_, city)) = self
                .view()
                .cities
                .iter()
                .filter(|c| {
                    !c.owner.is_barbarian()
                        && !self.view().same_team(self.view().me, c.owner)
                        && !c.walls
                        && (self.view().at_war(self.view().me, c.owner)
                            || crate::opportunity::legal_declaration(
                                self.view(),
                                self.view().team_of(c.owner),
                            ))
                })
                .filter_map(|c| {
                    let distance = self
                        .ctx
                        .own_cities
                        .iter()
                        .filter(|home| {
                            let owner = self.view().tile(c.pos).and_then(|tile| tile.owner);
                            legatum_core::path::find_path(
                                self.movement_map(),
                                home.pos,
                                c.pos,
                                |h| {
                                    let tile = self.view().tile(h)?;
                                    (tile.explored
                                        && (self.view().passage_allowed(
                                            self.view().me,
                                            UnitKind::Soldier,
                                            tile.owner,
                                        ) || tile.owner == owner))
                                        .then_some(tile.move_cost())
                                        .flatten()
                                },
                            )
                            .is_some()
                        })
                        .map(|home| home.pos.distance(c.pos))
                        .min()?;
                    (distance <= 18).then_some((distance, c))
                })
                // Gene `value_first_target`: the valued conquest nomination leads the forced
                // expedition when it qualifies; otherwise the nearest city.
                .min_by_key(|(distance, c)| {
                    let valued = self.value_first_target()
                        && self.conquest_city().is_some_and(|n| n.id == c.id);
                    // Gene `campaign_commit`: the expedition we declared for stays the target
                    // while it qualifies (open, in reach).
                    let committed = self.decisions.campaign_commit > 0
                        && self.routed_plan().is_some_and(|p| {
                            matches!(&p.kind, PlanKind::Campaign { target, against }
                                if *target == c.id
                                    && self.view().teams_at_war(self.view().team, *against))
                        });
                    (!committed, !valued, *distance, c.id)
                })
        {
            out.push(Candidate {
                kind: PlanKind::Campaign {
                    against: self.view().team_of(city.owner),
                    target: city.id,
                },
                value: 0,
                terms: vec![("forced expedition before local siege forecast".into(), 0)],
                reserve: 0,
                breach: false,
            });
        }
        out.extend(self.contain_candidates(&base, c0));
        out.extend(self.snipe_candidates(&base, c0));
        out.extend(self.field_strike_candidates(&base, c0));
        self.campaign_commitment(&mut out);
        self.leader_peace_forfeit(&base, c0, &mut out);
        // Peace locks both sides out of this war for the whole planning horizon. A
        // campaign or containment opportunity against this enemy cannot simply be
        // resumed next turn after the treaty, so charge the Peace plan for the part
        // of its best available war plan that the truce would delay.
        if let PlanKind::Peace { with } = &out[0].kind {
            let with = *with;
            let opportunity = out
                .iter()
                .skip(1)
                .filter(|c| c.kind.war_target() == Some(with))
                .map(|c| c.value)
                .max()
                .unwrap_or(0)
                .max(0);
            let delayed = opportunity
                * (legatum_core::diplomacy::PEACE_TRUCE_TURNS as i32).min(self.plan_horizon())
                / self.plan_horizon();
            if delayed > 0 {
                out[0].terms.push(("ten-turn peace truce".into(), -delayed));
                out[0].value -= delayed;
            }
        }
        if self.campaigns.plan.holds
            && let Some(hold) = self.hold_candidate(&out[0])
        {
            out.push(hold);
        }
        out
    }

    /// Gene `campaign_commit`: a Campaign war we declared, on a target still standing open,
    /// is a commitment. Any plan that walks away from it, for peace or another war, is charged
    /// the gene's value: the march and the declaration are spent and the defender walls up
    /// (playtest 8.9.2 Sparta what-ifs: AI Sparta declared on India at T15, switched its plan to
    /// Nicaea the next turn without declaring there, and made peace with India at T24 having
    /// fought neither).
    fn campaign_commitment(&self, out: &mut [Candidate]) {
        let weight = self.decisions.campaign_commit;
        if weight <= 0 || !self.hard_defense() {
            return;
        }
        let v = self.view();
        let Some(PlanKind::Campaign { against, target }) = self.routed_plan().map(|p| &p.kind)
        else {
            return;
        };
        let declared = v.diplomacy.relations.iter().any(|r| {
            r.between(v.team, *against)
                && r.state == RelationState::War
                && r.declarers.contains(&v.team)
        });
        let open = v
            .cities
            .iter()
            .any(|c| c.id == *target && v.team_of(c.owner) == *against && !c.walls);
        if !declared || !open {
            return;
        }
        for c in out.iter_mut() {
            if c.kind.war_target() != Some(*against) {
                c.terms.push(("abandons a declared war".into(), -weight));
                c.value -= weight;
            }
        }
    }

    /// Gene `leader_commit`: a plan that ends our war with the strict VP leader at half its
    /// target or more hands it back the VP that war could still take. Every candidate whose war
    /// is elsewhere pays our share of one VP for the leader, times the odds that this war (ours
    /// plus everyone fighting it, by public strength and reach) takes it (playtest 8.9.2: four
    /// AIs made peace with the leader at 5 to 7 of 10 VP; a what-if where the coalition never
    /// made peace cut his seat 16% to 0%).
    fn leader_peace_forfeit(&self, base: &WinFacts, c0: i32, out: &mut [Candidate]) {
        if self.decisions.leader_commit == 0 && self.decisions.leader_peace_horizon == 0 {
            return;
        }
        let v = self.view();
        let Some(t) = self.strict_leader(if self.decisions.leader_commit > 0 {
            50
        } else {
            90
        }) else {
            return;
        };
        if !v.teams_at_war(v.team, t) || crate::hold::rival_hold_of(v, t).is_some() {
            return;
        }
        let mut minus = base.clone();
        adjust(&mut minus, t, |f| f.vp = (f.vp - base.vp_unit).max(0));
        let gain = minus.chance() - c0;
        if gain <= 0 {
            return;
        }
        let theirs = strength(v, t).max(1);
        let coalition: i32 = v
            .standings
            .iter()
            .filter(|s| alive(v, s.team) && (s.team == v.team || v.teams_at_war(s.team, t)))
            .map(|s| strength(v, s.team))
            .sum();
        let ratio = (coalition * 100 / theirs).min(100);
        let reach = (15 - crate::diplomacy::nearest_known(v, t).unwrap_or(99)).clamp(0, 10);
        let odds = self
            .campaign_against(t)
            .map_or(0, |o| o.odds)
            .max(ratio * reach / 10)
            .max(self.leader_peace_continuation_value(t).clamp(0, 100));
        let forfeit = gain * odds / 100;
        if forfeit <= 0 {
            return;
        }
        for c in out.iter_mut() {
            if c.kind.war_target() != Some(t) {
                c.terms
                    .push(("peace leaves the leader its VP".into(), -forfeit));
                c.value -= forfeit;
            }
        }
    }

    /// The strict public VP leader (another team) at `percent` of its target or more.
    pub(crate) fn strict_leader(&self, percent: u32) -> Option<TeamId> {
        let v = self.view();
        let pct = |s: &legatum_core::view::TeamStanding| s.vp * 100 / s.target.max(1);
        let lead = v
            .standings
            .iter()
            .filter(|s| alive(v, s.team) && s.target > 0)
            .max_by_key(|s| (pct(s), std::cmp::Reverse(s.team)))?;
        let strict = v
            .standings
            .iter()
            .all(|s| s.team == lead.team || !alive(v, s.team) || pct(s) < pct(lead));
        (lead.team != v.team && strict && pct(lead) >= percent).then_some(lead.team)
    }

    /// Gene `coalition_front`: visible armies of other teams at War with `t`
    /// threatening cities in distinct theatres. A second flag beside the same
    /// city is reinforcement, not another front that splits the defence.
    pub(crate) fn coalition_fronts(&self, t: TeamId) -> i32 {
        let v = self.view();
        let cities: Vec<_> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == t)
            .map(|c| c.pos)
            .collect();
        let mut claimed: Vec<Hex> = cities
            .iter()
            .copied()
            .filter(|c| {
                self.ctx
                    .own_units
                    .iter()
                    .any(|u| u.military && u.kind != UnitKind::Scout && u.pos.distance(*c) <= 4)
            })
            .collect();
        let mut teams: Vec<TeamId> = v
            .units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && !u.owner.is_barbarian()
                    && v.team_of(u.owner) != v.team
                    && v.team_of(u.owner) != t
                    && v.teams_at_war(v.team_of(u.owner), t)
                    && cities.iter().any(|c| c.distance(u.pos) <= 4)
            })
            .map(|u| v.team_of(u.owner))
            .collect();
        teams.sort_unstable();
        teams.dedup();
        if self.decisions.coalition_push == 0 {
            return teams.len() as i32;
        }
        let mut fronts = 0;
        for team in teams {
            if let Some(city) = cities.iter().copied().find(|c| {
                !claimed.iter().any(|old| old.distance(*c) <= 4)
                    && v.units.iter().any(|u| {
                        u.military
                            && u.kind != UnitKind::Scout
                            && v.team_of(u.owner) == team
                            && u.pos.distance(*c) <= 4
                    })
            }) {
                claimed.push(city);
                fronts += 1;
            }
        }
        fronts
    }

    /// Hard+Search: hold the most threatened city (visible enemy pressure within the threat
    /// radius, after hard defence), priced like the quiet plan; rollouts decide its worth.
    fn hold_candidate(&self, quiet: &Candidate) -> Option<Candidate> {
        let (i, _) = self
            .ctx
            .threat
            .iter()
            .enumerate()
            .filter(|(_, t)| **t > 0)
            .max_by_key(|(i, t)| (**t, std::cmp::Reverse(self.ctx.own_cities[*i].id)))?;
        let city = self.ctx.own_cities[i];
        Some(Candidate {
            kind: PlanKind::Hold { city: city.id },
            value: quiet.value,
            terms: quiet.terms.clone(),
            reserve: 0,
            breach: false,
        })
    }

    /// Why the current plan can no longer stand, before any comparison of values.
    fn plan_abort_reason(&self, plan: &Plan) -> Option<PlanReason> {
        let v = self.view();
        match &plan.kind {
            PlanKind::Economy | PlanKind::Peace { .. } => None,
            PlanKind::Hold { city } => (!self.ctx.own_cities.iter().any(|c| c.id == *city))
                .then_some(PlanReason::TargetGone),
            PlanKind::Campaign { against, target } => {
                let city = v.cities.iter().find(|c| c.id == *target);
                let ended = self.campaigns.events.iter().rev().any(|e: &CampaignEvent| {
                    e.turn == v.turn && e.target == *target && e.end.is_some()
                });
                match city {
                    None => Some(PlanReason::TargetGone),
                    Some(c) if c.owner == v.me || v.same_team(v.me, c.owner) => {
                        Some(PlanReason::Captured)
                    }
                    Some(c) if v.team_of(c.owner) != *against => Some(PlanReason::TargetGone),
                    Some(_)
                        if !v.teams_at_war(v.team, *against)
                            && plan.start_turn < v.turn
                            && self.campaigns.events.iter().any(|e| {
                                e.target == *target
                                    && e.turn >= plan.start_turn
                                    && e.end == Some(CampaignEnd::Peace)
                            }) =>
                    {
                        Some(PlanReason::PeaceSigned)
                    }
                    Some(_) if ended => Some(PlanReason::CampaignEnded),
                    Some(_)
                        if !v.teams_at_war(v.team, *against) && !legal_declaration(v, *against) =>
                    {
                        Some(PlanReason::WarIllegal)
                    }
                    Some(_) => None,
                }
            }
            PlanKind::Contain { against } => {
                if !alive(v, *against) {
                    Some(PlanReason::TargetGone)
                } else if plan.start_turn < v.turn
                    && !v.teams_at_war(v.team, *against)
                    && !legal_declaration(v, *against)
                    && !plan.breach
                {
                    Some(PlanReason::PeaceSigned)
                } else {
                    None
                }
            }
            PlanKind::Snipe { settler, by, .. } => {
                if plan.start_turn == v.turn {
                    None
                } else if self.ctx.own_units.iter().any(|u| u.id == *settler) {
                    Some(PlanReason::SettlerTaken)
                } else if v.turn == plan.start_turn + 1
                    && self.ctx.own_units.iter().any(|u| {
                        let reach = if self.decisions.settler_raid_range == 0 {
                            1
                        } else if self.stalk_contact() {
                            // The view's `moves_left` is last turn's leftover.
                            self.turn_moves(u)
                        } else {
                            u.moves_left
                        };
                        u.id == *by
                            && v.units.iter().any(|target| {
                                let path_cost = self
                                    .view()
                                    .index(target.pos)
                                    .map(|target_index| {
                                        self.ctx.distance_field(u.pos, false)[target_index]
                                    })
                                    .unwrap_or(crate::analysis::UNREACHABLE);
                                target.id == *settler
                                    && (path_cost <= reach || u.pos.distance(target.pos) <= 1)
                            })
                    })
                {
                    None
                } else {
                    Some(PlanReason::SnipeMissed)
                }
            }
            PlanKind::FieldStrike {
                against, target, ..
            } => {
                if !v
                    .units
                    .iter()
                    .any(|u| u.id == *target && v.team_of(u.owner) == *against)
                {
                    Some(PlanReason::TargetGone)
                } else if !v.teams_at_war(v.team, *against) && !legal_declaration(v, *against) {
                    Some(PlanReason::WarIllegal)
                } else {
                    None
                }
            }
        }
    }

    /// A moving expedition should not be cancelled because one intermediate tile changes
    /// the prospective siege odds. The city must still be open, reachable and backed by
    /// healthy infantry; a new local siege still needs the ordinary forecast.
    fn missing_plan_patience(&self, plan: &Plan) -> bool {
        let PlanKind::Campaign { against, target } = &plan.kind else {
            return plan.misses == 0;
        };
        if let Some(active) = self
            .campaigns
            .active
            .as_ref()
            .filter(|c| c.target == *target)
            && let Some(city) = self.view().cities.iter().find(|c| c.id == *target)
            && (self.retake_hold(city, active)
                // Gene `finish_kill`: the siege that has the city nearly down keeps the plan
                // while its capturer marches in.
                || self.finish_kill_force(city, &active.committed).is_some())
        {
            return true;
        }
        let patience = self.decisions.campaign_march_patience;
        if patience == 0 {
            return plan.misses == 0;
        }
        let v = self.view();
        if plan.misses >= patience.max(1) as u32 {
            return false;
        }
        let Some(city) = v.cities.iter().find(|c| c.id == *target && !c.walls) else {
            return false;
        };
        let staging_at_peace = !v.teams_at_war(v.team, *against)
            && self.decisions.early_campaign_muster > 0
            && v.turn <= legatum_core::data::FIRST_THIRD_TURNS
            && self.sparta_iron_target() == Some(*against);
        let force: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.hp >= self.march_min_hp()
                    && (self.ctx.reachable(u.pos, city.pos)
                        || (staging_at_peace && u.pos.distance(city.pos) <= 14))
            })
            .collect();
        force.len() >= 2 && force.iter().any(|u| crate::composition::charges(u.kind))
    }

    /// A local wall siege can outlive one campaign forecast. If the current objective
    /// is still worth taking and the force still outweighs its defence, keep the war
    /// plan so its siege pass can continue rather than opening another front.
    fn keeps_walled_siege(&self, plan: &Plan, candidate: &Candidate) -> bool {
        let PlanKind::Campaign { against, target } = &plan.kind else {
            return false;
        };
        if !candidate.kind.same_objective(&plan.kind)
            || candidate.value <= 0
            || !self.view().teams_at_war(self.view().team, *against)
        {
            return false;
        }
        let Some(city) = self
            .view()
            .cities
            .iter()
            .find(|c| c.id == *target && c.walls && c.hp > 0)
        else {
            return false;
        };
        let committed: Vec<_> = self.ctx.own_units.iter().map(|u| u.id).collect();
        self.siege_local_margin(city, &committed)
            .is_some_and(|margin| margin >= self.decisions.war_siege)
    }

    /// Choose this turn's plan (Hard only), after the campaign is reconciled. The current
    /// plan stands unless it has become infeasible or a challenger beats it by the margin.
    pub(crate) fn choose_plan(&mut self) {
        if !self.plan_routing {
            return;
        }
        let v = self.view();
        let mut candidates = self.plan_candidates();
        let leader = self.runaway_read().map(|r| r.team);
        let leader_front = (self.decisions.reachable_wars > 0)
            .then(|| {
                leader.filter(|team| {
                    v.teams_at_war(v.team, *team)
                        || self
                            .operation_strike()
                            .is_some_and(|(against, _)| against == *team)
                })
            })
            .flatten();
        let other_front = v
            .players
            .iter()
            .any(|p| p.alive && p.team != v.team && v.teams_at_war(v.team, p.team));
        let recent_other_campaign = |target| {
            self.campaigns.plan.events.iter().any(|event| {
            matches!(event, PlanEvent::Switch { turn, to: PlanKind::Campaign { against, .. }, .. }
                if *against != target && v.turn.saturating_sub(*turn) <= crate::runaway::FRONT_TURNS)
        })
        };
        // A decisive public race can outrank the next conquest even while our army
        // is committed elsewhere. Require a clear valuation gap and a marchable
        // front; an ordinary speculative war still uses every reachable_wars gate.
        let urgent_front = if self.decisions.leader_war > 0 && self.decisions.reachable_wars > 0 {
            let top = candidates.iter().max_by_key(|c| c.value);
            top.and_then(|c| match &c.kind {
                PlanKind::Contain { against } if !v.teams_at_war(v.team, *against) => {
                    let standing = v.standings.iter().find(|s| s.team == *against)?;
                    let highest = v.standings.iter().map(|s| s.vp).max().unwrap_or(0);
                    let next = candidates
                        .iter()
                        .filter(|other| other.kind != c.kind)
                        .map(|other| other.value)
                        .max()
                        .unwrap_or(0);
                    let race = standing.victory_hold.is_some()
                        || ((self.contain_leader() == Some(*against)
                            || self.urgent_vp_contender() == Some(*against))
                            && standing.vp * 5 >= standing.target * 2
                            && standing.vp + 2 >= highest);
                    let turns = crate::hold::rival_hold_of(v, *against)
                        .map_or(self.plan_horizon(), |h| h.turns_left as i32);
                    (race && c.value >= next + 100 && self.reachable_war_force_by(*against, turns))
                        .then_some(*against)
                }
                _ => None,
            })
        } else {
            None
        };
        if self.decisions.reachable_wars > 0 {
            candidates.retain(|c| match &c.kind {
                PlanKind::Contain { against } => {
                    v.teams_at_war(v.team, *against)
                        || urgent_front == Some(*against)
                        || ((self.reachable_war_force(*against)
                            || self.busy_leader_front(*against))
                            && !recent_other_campaign(*against)
                            && (!other_front || self.campaign_against(*against).is_ok()))
                }
                PlanKind::Campaign { against, .. } => {
                    leader_front.is_none_or(|t| t == *against)
                        && (v.teams_at_war(v.team, *against)
                            || (!recent_other_campaign(*against)
                                && (leader != Some(*against)
                                    || self.reachable_war_force(*against))))
                }
                _ => true,
            });
        }
        // Highest value first; ties keep the stable generation order.
        candidates.sort_by_key(|c| std::cmp::Reverse(c.value));
        let best = candidates
            .first()
            .cloned()
            .expect("the quiet plan is always a candidate");
        let best_value = best.value;
        let current = self.campaigns.plan.active.clone();
        // A rollout seat keeps its candidate plan while it's on offer (`search`).
        let rollout = self.campaigns.plan.forced.clone().and_then(|f| {
            candidates
                .iter()
                .find(|c| v.turn < f.until && c.kind.same_objective(&f.kind))
                .cloned()
        });
        if rollout.is_none() {
            self.campaigns.plan.forced = None;
        }
        let stale_other_campaign = current.as_ref().is_some_and(|plan| {
            leader_front.is_some_and(|leader| {
                matches!(
                    &plan.kind, PlanKind::Campaign { against, .. } if *against != leader
                )
            })
        });
        let decision = if let Some(pick) = rollout {
            match &current {
                None => Decision::Switch(pick, PlanReason::Start),
                Some(plan) => match self.plan_abort_reason(plan) {
                    Some(reason) => Decision::Switch(pick, reason),
                    None if pick.kind.same_objective(&plan.kind) => Decision::Stand(pick),
                    None => Decision::Switch(pick, PlanReason::Better),
                },
            }
        } else {
            match &current {
                None => Decision::Switch(best, PlanReason::Start),
                Some(plan) => match self.plan_abort_reason(plan) {
                    Some(reason) => {
                        let same = candidates
                            .iter()
                            .find(|c| c.kind.same_objective(&plan.kind));
                        if reason == PlanReason::CampaignEnded
                            && let Some(same) = same
                            && self.keeps_walled_siege(plan, same)
                        {
                            Decision::Stand(same.clone())
                        } else {
                            // A truly ended objective must not keep the war plan.
                            let next = candidates
                                .iter()
                                .find(|c| !c.kind.same_objective(&plan.kind))
                                .cloned()
                                .unwrap_or(best);
                            Decision::Switch(next, reason)
                        }
                    }
                    None => match candidates
                        .iter()
                        .find(|c| c.kind.same_objective(&plan.kind))
                    {
                        None if !stale_other_campaign
                            && (plan
                                .kind
                                .war_target()
                                .is_some_and(|t| v.teams_at_war(v.team, t))
                                || (self.musters_campaigns()
                                    && matches!(&plan.kind, PlanKind::Campaign { .. })))
                            && self.missing_plan_patience(plan) =>
                        {
                            Decision::Miss
                        }
                        None => Decision::Switch(best, PlanReason::Infeasible),
                        Some(same) => {
                            let margin = if v.turn < plan.review_turn {
                                2 * self.decisions.switch_margin
                            } else {
                                self.decisions.switch_margin
                            };
                            if !best.kind.same_objective(&plan.kind)
                                && best.value > same.value + margin
                            {
                                Decision::Better {
                                    best,
                                    same: same.clone(),
                                }
                            } else {
                                Decision::Stand(same.clone())
                            }
                        }
                    },
                },
            }
        };
        let decision = match self.search {
            Some(hook) => self.search_decision(hook, decision, &candidates, current.as_ref()),
            None => decision,
        };
        // Sparta captures replenish its field army. Keep a feasible Campaign through
        // short-horizon plan and Search valuations, then advance to the next exposed city.
        let decision = if self.decisions.sparta_force_campaign > 0
            && let Some(campaign) = candidates
                .iter()
                .find(|c| {
                    matches!(&c.kind, PlanKind::Campaign { .. })
                        && (self.decisions.sparta_valued_expedition == 0
                            || (c.value > 0 && c.value >= best_value))
                        && current
                            .as_ref()
                            .is_some_and(|p| p.kind.same_objective(&c.kind))
                })
                .or_else(|| {
                    candidates.iter().find(|c| {
                        matches!(&c.kind, PlanKind::Campaign { .. })
                            && (self.decisions.sparta_valued_expedition == 0
                                || (c.value > 0 && c.value >= best_value))
                    })
                }) {
            if current
                .as_ref()
                .is_some_and(|p| p.kind.same_objective(&campaign.kind))
            {
                Decision::Stand(campaign.clone())
            } else {
                Decision::Switch(campaign.clone(), PlanReason::Better)
            }
        } else {
            decision
        };
        let decision = if let Some(followup) = candidates
            .iter()
            .filter(|candidate| {
                (self.decisions.sparta_valued_expedition == 0
                    || (candidate.value > 0 && candidate.value >= best_value))
                    && matches!(&candidate.kind, PlanKind::Campaign { target, .. }
                if v.cities.iter().any(|city| city.id == *target
                    && (self.sparta_open_followup(city)
                        || self.sparta_chain_target(city))))
            })
            .min_by_key(|candidate| {
                let PlanKind::Campaign { target, .. } = &candidate.kind else {
                    unreachable!()
                };
                let city = v.cities.iter().find(|city| city.id == *target).unwrap();
                (
                    !city.is_capital,
                    city.pos.distance(self.ctx.own_cities[0].pos),
                    city.id,
                )
            }) {
            if current
                .as_ref()
                .is_some_and(|plan| plan.kind.same_objective(&followup.kind))
            {
                Decision::Stand(followup.clone())
            } else {
                Decision::Switch(followup.clone(), PlanReason::Better)
            }
        } else {
            decision
        };
        // Gene `operations`: a committed operation's plan outranks the plan layer and the
        // faction rules above until the operation completes or aborts (`operation.rs`).
        let decision = match self.operation_candidate(&candidates) {
            Some(pick) => {
                if current
                    .as_ref()
                    .is_some_and(|p| p.kind.same_objective(&pick.kind))
                {
                    Decision::Stand(pick)
                } else {
                    Decision::Switch(pick, PlanReason::Better)
                }
            }
            None => decision,
        };
        if self.decision_trace.is_some() {
            let selected = match &decision {
                Decision::Switch(c, _) | Decision::Stand(c) => Some(&c.kind),
                Decision::Better { best, .. } => Some(&best.kind),
                Decision::Miss => current.as_ref().map(|p| &p.kind),
            };
            let winner_score = candidates.first().map(|c| c.value);
            let runner_up_score = candidates.get(1).map(|c| c.value);
            for candidate in &candidates {
                let mut row = crate::DecisionOption::new(
                    "plan",
                    format!("{:?}", candidate.kind),
                    if selected.is_some_and(|kind| kind.same_objective(&candidate.kind)) {
                        "selected"
                    } else {
                        "considered_rejected"
                    },
                );
                row.score = Some(candidate.value);
                row.winner_score = winner_score;
                row.runner_up_score = runner_up_score;
                self.note_option(row);
            }
            for city in v
                .cities
                .iter()
                .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner))
            {
                if candidates.iter().any(|candidate| matches!(candidate.kind, PlanKind::Campaign { target, .. } if target == city.id)) { continue; }
                let gate = if !v.at_war(v.me, city.owner)
                    && !legal_declaration(v, v.team_of(city.owner))
                {
                    "war declaration illegal"
                } else if self.campaigns.held_capture.is_some() && !self.sparta_chain_target(city) {
                    "held capture"
                } else if !v.at_war(v.me, city.owner) && self.stale_city(city) {
                    "stale city sighting"
                } else {
                    let aged = self.aged_city(city);
                    let city = aged.as_ref().unwrap_or(city);
                    match self.funded_forecast(city, None) {
                        None => "funded forecast unavailable",
                        Some(f)
                            if f.march == 0
                                && self
                                    .campaign_forecast(city, Some(&self.campaign_force(city)))
                                    .is_none() =>
                        {
                            "in-reach campaign force gate"
                        }
                        Some(f)
                            if capture_odds(f.turns, f.march)
                                < self.decisions.min_seat_war_odds =>
                        {
                            "capture odds below minimum"
                        }
                        Some(_) => "campaign nomination shortlist",
                    }
                };
                let mut row = crate::DecisionOption::new(
                    "plan",
                    format!("Campaign target={:?}", city.id),
                    "gated_out",
                );
                row.gate = Some(gate.into());
                self.note_option(row);
            }
            for standing in v.standings.iter().filter(|s| s.team != v.team) {
                if candidates.iter().any(|c| matches!(&c.kind, PlanKind::Contain { against } if *against == standing.team)) { continue; }
                let team = standing.team;
                let at_war = v.teams_at_war(v.team, team);
                let holder = crate::hold::rival_hold_of(v, team).is_some();
                let fought = match self.routed_plan().map(|p| &p.kind) {
                    Some(PlanKind::Contain { against }) => Some(*against),
                    Some(PlanKind::Campaign { against, .. })
                        if self.decisions.contain_after_campaign > 0 =>
                    {
                        Some(*against)
                    }
                    _ => None,
                };
                let nominated = self.diplomatic_target() == Some(team)
                    || self.contain_leader() == Some(team)
                    || crate::hold::holds(v).iter().any(|h| h.team == team)
                    || (fought == Some(team) && at_war);
                let progress = 100 * standing.vp / standing.target.max(1);
                let breach = !at_war
                    && !legal_declaration(v, team)
                    && (progress >= 90 || holder)
                    && v.diplomacy.actions.iter().any(|a| {
                        a.blocked_reason.is_none()
                            && a.order
                                == Order::DeclareWar {
                                    against: team,
                                    breach: true,
                                }
                    });
                let gate = if !nominated {
                    "not a diplomatic target, leader, hold, or ongoing contain war"
                } else if !alive(v, team) {
                    "containment target eliminated"
                } else if !at_war && !legal_declaration(v, team) && !breach {
                    "containment declaration illegal"
                } else if !at_war
                    && (self.faction == Faction::Sparta
                        || self.sack_commits()
                        || self.decisions.leader_commit >= 2)
                    && self.decisions.contain_commit > 0
                    && self.decisions.contain_campaign > 0
                    && !self.busy_leader_front(team)
                    && !self
                        .campaign_against(team)
                        .is_ok_and(|outlook| outlook.odds >= self.decisions.contain_min_odds)
                {
                    "containment campaign commitment forecast"
                } else if !at_war
                    && !holder
                    && self.gold_runway().is_some_and(|r| r < self.runway_floor())
                {
                    "containment Gold runway floor"
                } else {
                    "containment nomination produced no candidate"
                };
                let mut row = crate::DecisionOption::new(
                    "plan",
                    format!("Contain against={team:?}"),
                    "gated_out",
                );
                row.gate = Some(gate.into());
                self.note_option(row);
            }
            for unit in v.units.iter().filter(|u| {
                u.kind == UnitKind::Settler
                    && !v.same_team(v.me, u.owner)
                    && !u.owner.is_barbarian()
            }) {
                if candidates.iter().any(
                    |c| matches!(&c.kind, PlanKind::Snipe { settler, .. } if *settler == unit.id),
                ) {
                    continue;
                }
                let mut row = crate::DecisionOption::new(
                    "plan",
                    format!("Snipe settler={:?}", unit.id),
                    "gated_out",
                );
                row.gate = Some(
                    if self.decisions.settler_raid == 0 {
                        "settler_raid disabled"
                    } else if v.at_war(v.me, unit.owner) {
                        "Snipe requires Peace"
                    } else if !legal_declaration(v, v.team_of(unit.owner)) {
                        "Snipe declaration illegal"
                    } else if v.cities.iter().any(|c| c.pos == unit.pos) {
                        "Settler inside a city"
                    } else if v.units.iter().any(|u| {
                        u.military
                            && v.same_team(u.owner, unit.owner)
                            && u.pos.distance(unit.pos) <= 1
                    }) {
                        "Settler guarded by adjacent military"
                    } else if self.ctx.move_cost(unit.pos).is_none() {
                        "Settler tile has no move cost"
                    } else {
                        "no eligible raider reaches or contacts Settler"
                    }
                    .into(),
                );
                self.note_option(row);
            }
            for unit in v
                .units
                .iter()
                .filter(|u| u.military && !v.same_team(v.me, u.owner) && !u.owner.is_barbarian())
            {
                if candidates.iter().any(|c| matches!(&c.kind, PlanKind::FieldStrike { target, .. } if *target == unit.id)) { continue; }
                let mut row = crate::DecisionOption::new(
                    "plan",
                    format!("FieldStrike target={:?}", unit.id),
                    "gated_out",
                );
                row.gate = Some(
                    if self.decisions.field_strike == 0 {
                        "field_strike disabled"
                    } else if !self.hard_defense() {
                        "Hard defense unavailable"
                    } else if !self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|c| c.pos.distance(unit.pos) <= 3)
                    {
                        "enemy force beyond three hexes from own city"
                    } else if !v.teams_at_war(v.team, v.team_of(unit.owner))
                        && !legal_declaration(v, v.team_of(unit.owner))
                    {
                        "field strike declaration illegal"
                    } else if v
                        .units
                        .iter()
                        .filter(|u| {
                            u.military
                                && v.team_of(u.owner) == v.team_of(unit.owner)
                                && u.pos.distance(unit.pos) <= 2
                        })
                        .count()
                        < 2
                    {
                        "enemy force below two units"
                    } else {
                        "no positive legal attack forecast or own power below enemy"
                    }
                    .into(),
                );
                self.note_option(row);
            }
            if !self.campaigns.plan.holds {
                let mut row = crate::DecisionOption::new("plan", "Hold", "gated_out");
                row.gate = Some("search holds disabled".into());
                self.note_option(row);
            }
        }
        let (next, reason) = match decision {
            Decision::Switch(next, reason) => (next, reason),
            Decision::Better { best, .. } => (best, PlanReason::Better),
            Decision::Miss => {
                if let Some(active) = self.campaigns.plan.active.as_mut() {
                    active.misses += 1;
                }
                self.start_planned_campaign();
                return;
            }
            Decision::Stand(same) => {
                // The plan stands; its price and label follow the situation.
                let horizon = self.plan_horizon() as u32;
                let active = self.campaigns.plan.active.as_mut().expect("current");
                active.kind = same.kind;
                active.value = same.value;
                active.terms = same.terms;
                active.reserve = same.reserve;
                active.breach = same.breach;
                active.misses = 0;
                if v.turn >= active.review_turn {
                    active.review_turn = v.turn + horizon;
                }
                self.start_planned_campaign();
                return;
            }
        };
        let chance = WinFacts::of(v).with_relative(&self.decisions).chance();
        self.campaigns.plan.events.push(PlanEvent::Switch {
            turn: v.turn,
            from: current.as_ref().map(|p| p.kind.clone()),
            to: next.kind.clone(),
            value: next.value,
            chance,
            reason,
            terms: next.terms.clone(),
        });
        // A campaign the new plan doesn't name ends now, deliberately.
        if let Some(active) = self.campaigns.active.clone()
            && !matches!(next.kind, PlanKind::Campaign { target, .. } if target == active.target)
        {
            self.campaigns.active = None;
            self.end_campaign(active, CampaignEnd::PlanChanged);
        }
        self.campaigns.plan.active = Some(Plan {
            kind: next.kind,
            start_turn: v.turn,
            review_turn: v.turn + self.plan_horizon() as u32 + self.review_stagger(),
            reserve: next.reserve,
            value: next.value,
            terms: next.terms,
            breach: next.breach,
            misses: 0,
        });
        self.start_planned_campaign();
    }

    /// Hard+Search: at a review point (the first plan, an ended plan, the review turn, or a
    /// challenger not yet rolled out) the top candidates and the running plan are played
    /// forward, and the best horizon win chance decides. Elsewhere the plan stands.
    fn search_decision(
        &mut self,
        hook: crate::search::SearchHook<'_>,
        decision: Decision,
        candidates: &[Candidate],
        current: Option<&Plan>,
    ) -> Decision {
        if hook.config.candidates == 0 {
            return decision;
        }
        let v = self.view();
        // Information triggers: what this seat sees changed enough to rethink now.
        let signals = Signals::of(v);
        let signal = if hook.config.triggers {
            self.campaigns
                .plan
                .signals
                .as_ref()
                .and_then(|last| signals.fired(last))
        } else {
            None
        };
        if hook.config.triggers {
            self.campaigns.plan.signals = Some(signals);
        }
        // A newly threatened city (a Hold candidate the last search didn't weigh) reopens
        // the decision.
        let signal = signal.or_else(|| {
            candidates
                .iter()
                .any(|c| {
                    matches!(c.kind, PlanKind::Hold { .. })
                        && !self
                            .campaigns
                            .plan
                            .searched
                            .iter()
                            .any(|k| k.same_objective(&c.kind))
                })
                .then_some("hold")
        });
        // A win chance a third below the last search's reopens the decision (playtest 8.3: the
        // plan stood on Economy for 51 turns while the chance fell from 54% to 1%).
        let signal = signal.or_else(|| {
            (hook.config.collapse
                && self.campaigns.plan.searched_chance.is_some_and(|last| {
                    WinFacts::of(v).with_relative(&self.decisions).chance() * 3 < last * 2
                }))
            .then_some("collapse")
        });
        let trigger: Option<String> = match &decision {
            Decision::Miss => None,
            Decision::Switch(_, PlanReason::Start) => Some("start".into()),
            Decision::Switch(_, reason) => Some(format!("ended:{reason:?}")),
            Decision::Better { best, same } => {
                if self
                    .campaigns
                    .plan
                    .searched
                    .iter()
                    .any(|k| k.same_objective(&best.kind))
                {
                    // The last search already weighed this challenger.
                    return match signal {
                        Some(_) => self.search_decision_with(
                            hook,
                            Decision::Stand(same.clone()),
                            candidates,
                            current,
                            signal.map(|s| format!("signal:{s}")),
                        ),
                        None => Decision::Stand(same.clone()),
                    };
                }
                Some("challenger".into())
            }
            Decision::Stand(_) => {
                if current.is_some_and(|p| v.turn >= p.review_turn) {
                    Some("review".into())
                } else {
                    signal.map(|s| format!("signal:{s}"))
                }
            }
        };
        self.search_decision_with(hook, decision, candidates, current, trigger)
    }

    fn search_decision_with(
        &mut self,
        hook: crate::search::SearchHook<'_>,
        decision: Decision,
        candidates: &[Candidate],
        current: Option<&Plan>,
        trigger: Option<String>,
    ) -> Decision {
        let v = self.view();
        let Some(trigger) = trigger else {
            return decision;
        };
        let running = match &decision {
            Decision::Stand(same) | Decision::Better { same, .. } => Some(same.clone()),
            _ => None,
        };
        // An ended plan's objective isn't on offer again.
        let ended = match &decision {
            Decision::Switch(_, _) => current.map(|p| p.kind.clone()),
            _ => None,
        };
        let mut pool: Vec<Candidate> = running.iter().cloned().collect();
        for c in candidates {
            if pool.len() >= hook.config.candidates as usize + usize::from(running.is_some()) {
                break;
            }
            if ended.as_ref().is_some_and(|k| c.kind.same_objective(k))
                || pool.iter().any(|p| p.kind.same_objective(&c.kind))
            {
                continue;
            }
            pool.push(c.clone());
        }
        // The plan plain Hard would pick is always among them.
        let analytic_pick = match &decision {
            Decision::Switch(next, _) => next.clone(),
            Decision::Better { best, .. } => best.clone(),
            Decision::Stand(same) => same.clone(),
            Decision::Miss => unreachable!("not a review point"),
        };
        if !pool
            .iter()
            .any(|p| p.kind.same_objective(&analytic_pick.kind))
        {
            pool.push(analytic_pick.clone());
        }
        // A Hold is always weighed when on offer (its analytic price is only the quiet
        // plan's, so it never makes the top K by value); with fronts, so is every campaign
        // target on offer.
        let fronts = self.campaigns.plan.fronts;
        for c in candidates.iter().filter(|c| {
            matches!(c.kind, PlanKind::Hold { .. })
                || fronts && matches!(c.kind, PlanKind::Campaign { .. })
        }) {
            if !pool.iter().any(|p| p.kind.same_objective(&c.kind)) {
                pool.push(c.clone());
            }
        }
        self.campaigns.plan.searched = pool.iter().map(|c| c.kind.clone()).collect();
        if hook.config.collapse {
            self.campaigns.plan.searched_chance =
                Some(WinFacts::of(v).with_relative(&self.decisions).chance());
        }
        if hook.config.composites > 0 {
            return self.search_composites(hook, decision, pool, running, &analytic_pick, trigger);
        }
        if pool.len() < 2 {
            if hook.config.postures {
                self.search_posture(hook, &analytic_pick.kind, &trigger);
            }
            return decision;
        }
        if matches!(self.decisions.learned_macro, 1 | 2)
            && v.players.len() >= 4
            && pool.iter().all(|c| {
                matches!(
                    &c.kind,
                    PlanKind::Economy
                        | PlanKind::Peace { .. }
                        | PlanKind::Campaign { .. }
                        | PlanKind::Contain { .. }
                )
            })
        {
            let predictions: Vec<i32> = pool
                .iter()
                .map(|c| {
                    let f = crate::learned_macro::features(v, &c.kind, None);
                    crate::learned_macro::value(&f, &c.kind, None)
                })
                .collect();
            let chosen = (0..pool.len())
                .max_by_key(|i| (predictions[*i], std::cmp::Reverse(*i)))
                .expect("options");
            let analytic = pool
                .iter()
                .position(|p| p.kind.same_objective(&analytic_pick.kind))
                .unwrap_or(0);
            self.search_event = Some(crate::search::SearchEvent {
                turn: v.turn,
                chance: WinFacts::of(v).chance(),
                options: pool
                    .iter()
                    .zip(&predictions)
                    .map(|(c, prediction)| crate::search::SearchOption {
                        kind: c.kind.clone(),
                        opening: None,
                        posture: None,
                        value: c.value,
                        mean: *prediction,
                        samples: Vec::new(),
                        beyond: 0,
                    })
                    .collect(),
                chosen,
                analytic,
                omniscient: false,
                trigger: format!("learned:{trigger}"),
                horizon: 0,
            });
            let pick = pool.swap_remove(chosen);
            return if running
                .as_ref()
                .is_some_and(|r| r.kind.same_objective(&pick.kind))
            {
                Decision::Stand(pick)
            } else {
                Decision::Switch(pick, PlanReason::Better)
            };
        }
        let kinds: Vec<PlanKind> = pool.iter().map(|c| c.kind.clone()).collect();
        let (results, horizon) = hook.evaluate(v, &kinds);
        let means: Vec<i32> = results
            .iter()
            .map(|r| r.iter().sum::<i32>() / r.len().max(1) as i32)
            .collect();
        // A containment war's worth lies mostly past a 10-turn rollout (the leader's VP it
        // denies later; playtest 8.4: every rollout scored Contain like Peace). Its analytic
        // leader's race is added to the rollout mean (gene `search_race`).
        let beyond: Vec<i32> = pool
            .iter()
            .map(|c| {
                if self.decisions.search_race > 0 && matches!(c.kind, PlanKind::Contain { .. }) {
                    c.terms
                        .iter()
                        .filter(|(name, _)| name == LEADER_RACE)
                        .map(|(_, x)| *x)
                        .sum()
                } else {
                    0
                }
            })
            .collect();
        // Highest mean; ties keep the running plan (first), then the analytic order.
        let chosen = (0..pool.len())
            .max_by_key(|i| (means[*i] + beyond[*i], std::cmp::Reverse(*i)))
            .expect("two or more options");
        let analytic = pool
            .iter()
            .position(|p| p.kind.same_objective(&analytic_pick.kind))
            .unwrap_or(0);
        self.search_event = Some(crate::search::SearchEvent {
            turn: v.turn,
            chance: WinFacts::of(v).with_relative(&self.decisions).chance(),
            options: pool
                .iter()
                .zip(&results)
                .zip(&means)
                .zip(&beyond)
                .map(|(((c, r), m), b)| crate::search::SearchOption {
                    kind: c.kind.clone(),
                    opening: None,
                    posture: None,
                    value: c.value,
                    mean: *m,
                    samples: r.clone(),
                    beyond: *b,
                })
                .collect(),
            chosen,
            analytic,
            omniscient: hook.oracle.is_some(),
            trigger: trigger.clone(),
            horizon,
        });
        let pick = pool.swap_remove(chosen);
        if hook.config.postures {
            self.search_posture(hook, &pick.kind, &trigger);
        }
        if running
            .as_ref()
            .is_some_and(|r| r.kind.same_objective(&pick.kind))
        {
            return Decision::Stand(pick);
        }
        match decision {
            Decision::Switch(_, reason) => Decision::Switch(pick, reason),
            _ => Decision::Switch(pick, PlanReason::Search),
        }
    }

    /// Hard+Search with composite candidates (`c=N`): plans and postures searched together.
    /// The first N of: every plan in the pool with the current posture, then every plan with
    /// each other posture in turn. The pick sets both the plan and the posture.
    fn search_composites(
        &mut self,
        hook: crate::search::SearchHook<'_>,
        decision: Decision,
        pool: Vec<Candidate>,
        running: Option<Candidate>,
        analytic_pick: &Candidate,
        trigger: String,
    ) -> Decision {
        let v = self.view();
        let current = self.campaigns.plan.posture;
        let postures: Vec<Option<Posture>> = std::iter::once(current)
            .chain(
                std::iter::once(None)
                    .chain(Posture::ALL.into_iter().map(Some))
                    .filter(|p| *p != current),
            )
            .collect();
        let options: Vec<(usize, Option<Posture>)> = postures
            .iter()
            .flat_map(|p| (0..pool.len()).map(move |i| (i, *p)))
            .take(hook.config.composites as usize)
            .collect();
        if options.len() < 2 {
            return decision;
        }
        let kinds: Vec<(PlanKind, Option<Posture>)> = options
            .iter()
            .map(|(i, p)| (pool[*i].kind.clone(), *p))
            .collect();
        let (results, horizon) = hook.evaluate_composites(v, &kinds);
        let means: Vec<i32> = results
            .iter()
            .map(|r| r.iter().sum::<i32>() / r.len().max(1) as i32)
            .collect();
        // Highest mean; ties keep the earlier option (the running plan, current posture).
        let chosen = (0..options.len())
            .max_by_key(|i| (means[*i], std::cmp::Reverse(*i)))
            .expect("two or more options");
        let analytic = options
            .iter()
            .position(|(i, p)| *p == current && pool[*i].kind.same_objective(&analytic_pick.kind))
            .unwrap_or(0);
        self.search_event = Some(crate::search::SearchEvent {
            turn: v.turn,
            chance: WinFacts::of(v).with_relative(&self.decisions).chance(),
            options: options
                .iter()
                .zip(&results)
                .zip(&means)
                .map(|(((i, p), r), m)| crate::search::SearchOption {
                    kind: pool[*i].kind.clone(),
                    opening: None,
                    posture: Some(p.map_or("personality".to_string(), |p| format!("{p:?}"))),
                    value: pool[*i].value,
                    mean: *m,
                    samples: r.clone(),
                    beyond: 0,
                })
                .collect(),
            chosen,
            analytic,
            omniscient: hook.oracle.is_some(),
            trigger,
            horizon,
        });
        let (index, posture) = options[chosen];
        if posture != current {
            self.campaigns.plan.posture = posture;
            self.knobs = Posture::apply(posture, hook.brain.adapted_knobs());
            // Knob-dependent memos are stale now.
            self.diplomatic_campaigns.borrow_mut().clear();
        }
        let pick = pool[index].clone();
        if running
            .as_ref()
            .is_some_and(|r| r.kind.same_objective(&pick.kind))
        {
            return Decision::Stand(pick);
        }
        match decision {
            Decision::Switch(_, reason) => Decision::Switch(pick, reason),
            _ => Decision::Switch(pick, PlanReason::Search),
        }
    }

    /// Hard+Search with postures: with the plan settled, roll out each posture (and none)
    /// under it and keep the best; it applies from this turn.
    fn search_posture(
        &mut self,
        hook: crate::search::SearchHook<'_>,
        plan: &PlanKind,
        trigger: &str,
    ) {
        let v = self.view();
        let current = self.campaigns.plan.posture;
        let mut options: Vec<Option<Posture>> = vec![current];
        options.extend(
            std::iter::once(None)
                .chain(Posture::ALL.into_iter().map(Some))
                .filter(|p| *p != current),
        );
        let (results, horizon) = hook.evaluate_postures(v, plan, &options);
        let means: Vec<i32> = results
            .iter()
            .map(|r| r.iter().sum::<i32>() / r.len().max(1) as i32)
            .collect();
        // Ties keep the current posture (first).
        let chosen = (0..options.len())
            .max_by_key(|i| (means[*i], std::cmp::Reverse(*i)))
            .expect("postures");
        self.posture_event = Some(crate::search::SearchEvent {
            turn: v.turn,
            chance: WinFacts::of(v).with_relative(&self.decisions).chance(),
            options: options
                .iter()
                .zip(&results)
                .zip(&means)
                .map(|((p, r), m)| crate::search::SearchOption {
                    kind: plan.clone(),
                    opening: None,
                    posture: Some(p.map_or("personality".to_string(), |p| format!("{p:?}"))),
                    value: 0,
                    mean: *m,
                    samples: r.clone(),
                    beyond: 0,
                })
                .collect(),
            chosen,
            analytic: 0,
            omniscient: hook.oracle.is_some(),
            trigger: format!("posture:{trigger}"),
            horizon,
        });
        self.campaigns.plan.posture = options[chosen];
        self.knobs = Posture::apply(options[chosen], hook.brain.adapted_knobs());
        // Knob-dependent memos are stale now.
        self.diplomatic_campaigns.borrow_mut().clear();
    }

    /// The Hold plan's move: a defender in the city, by holding one already there, moving the
    /// nearest that can arrive this turn (not another city's garrison), or buying one.
    pub(crate) fn execute_hold(&mut self) {
        let Some(PlanKind::Hold { city }) = self.routed_plan().map(|p| p.kind.clone()) else {
            return;
        };
        let Some(c) = self.ctx.own_cities.iter().copied().find(|c| c.id == city) else {
            return;
        };
        if self.mil_dest.contains(&c.pos) {
            return;
        }
        let army: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.military && u.kind != UnitKind::Scout && !self.has_acted(u.id))
            .collect();
        if let Some(u) = army.iter().find(|u| u.pos == c.pos) {
            // A stronger defender next to the city swaps in when the one inside would not hold
            // (the swap `door.rs` makes; holding first used up the guard's move and blocked it).
            if let Some(r) = self.garrison_swap(c, u, &army) {
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: r.pos,
                    },
                );
                self.unit_order(
                    r.id,
                    Order::MoveUnit {
                        unit: r.id,
                        to: c.pos,
                    },
                );
                self.mil_dest.insert(r.pos);
                self.mil_dest.insert(c.pos);
                return;
            }
            self.hold_or_upgrade(u);
            return;
        }
        if self
            .ctx
            .view
            .index(c.pos)
            .is_some_and(|i| self.ctx.friendly_military_at[i])
        {
            return;
        }
        let ranged_first = c.hp * 2 > data::CITY_HP;
        let mover = army
            .iter()
            .filter(|u| {
                !self.ctx.own_cities.iter().any(|o| o.pos == u.pos)
                    && self.arrives(u, c.pos)
                    && self.can_defend_at(u, c.pos)
            })
            .min_by_key(|u| {
                // A unit with both attacks (the Keshig) fits either role.
                (
                    !crate::composition::dual(u.kind, u.ranged) && (u.ranged > 0) != ranged_first,
                    u.pos.distance(c.pos),
                    u.id,
                )
            })
            .copied();
        if let Some(u) = mover {
            self.go_to(u, c.pos);
            return;
        }
        if self
            .orders
            .iter()
            .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id))
        {
            return;
        }
        let Some(d) = c.detail.as_ref() else {
            return;
        };
        let buy = d
            .options
            .iter()
            .filter_map(|o| {
                let ProductionItem::Unit { unit } = o.item else {
                    return None;
                };
                let stats = self
                    .view()
                    .rule_experiments
                    .unit_stats_for(self.faction, unit);
                let cost = o.buy_cost?;
                (stats.military
                    && unit != UnitKind::Scout
                    && o.buy_block.is_none()
                    && cost > 0
                    && cost <= self.gold_left)
                    .then_some((
                        !crate::composition::dual(unit, stats.ranged)
                            && (stats.ranged > 0) != ranged_first,
                        std::cmp::Reverse(stats.strength.max(stats.ranged)),
                        cost,
                        o.item,
                    ))
            })
            .min();
        if let Some((_, _, cost, item)) = buy {
            self.orders.push(Order::BuyItem { city: c.id, item });
            self.gold_left -= cost;
        }
    }

    /// Hard+Search tactics: on a turn our military is engaged (within 2 hexes of a visible
    /// enemy unit or city at war), roll out each stance a few turns and keep the one with the
    /// best material balance. Otherwise the default stance.
    pub(crate) fn choose_stance(&mut self) {
        let Some(hook) = self.search.filter(|h| h.config.stances) else {
            return;
        };
        let v = self.view();
        let engaged =
            self.ctx.own_units.iter().any(|u| {
                u.military
                    && (self.ctx.enemy_units.iter().any(|e| {
                        e.military && !e.owner.is_barbarian() && e.pos.distance(u.pos) <= 2
                    }) || self
                        .ctx
                        .enemy_cities
                        .iter()
                        .any(|c| !c.owner.is_barbarian() && c.pos.distance(u.pos) <= 2))
            });
        if !engaged {
            self.campaigns.plan.stance = None;
            return;
        }
        let options: Vec<Option<crate::search::Stance>> = std::iter::once(None)
            .chain(crate::search::Stance::ALL.into_iter().map(Some))
            .collect();
        let plan = self.campaigns.plan.active.as_ref().map(|p| p.kind.clone());
        let (results, horizon) = hook.evaluate_stances(v, &options, plan.as_ref());
        let means: Vec<i32> = results
            .iter()
            .map(|r| r.iter().map(|x| *x as i64).sum::<i64>() / r.len().max(1) as i64)
            .map(|m| m.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
            .collect();
        // Ties keep the default (first).
        let chosen = (0..options.len())
            .max_by_key(|i| (means[*i], std::cmp::Reverse(*i)))
            .expect("stances");
        self.campaigns.plan.stance = options[chosen];
        self.stance_event = Some(crate::search::SearchEvent {
            turn: v.turn,
            chance: WinFacts::of(v).with_relative(&self.decisions).chance(),
            options: options
                .iter()
                .zip(&results)
                .zip(&means)
                .map(|((s, r), m)| crate::search::SearchOption {
                    kind: plan.clone().unwrap_or(PlanKind::Economy),
                    opening: None,
                    posture: Some(s.map_or("default".to_string(), |s| format!("stance:{s:?}"))),
                    value: 0,
                    mean: *m,
                    samples: r.clone(),
                    beyond: 0,
                })
                .collect(),
            chosen,
            analytic: 0,
            omniscient: hook.oracle.is_some(),
            trigger: "stance".into(),
            horizon,
        });
    }

    /// A campaign plan against a city we're already at war with commits its force now. A
    /// containment war (gene `contain_campaign`) commits to the target's city its marching army
    /// has reached (`military.rs::contain_target`), once the campaign's own gates pass there.
    fn start_planned_campaign(&mut self) {
        if self.campaigns.active.is_some()
            || (self.campaigns.held_capture.is_some()
                && !self.routed_plan().is_some_and(|plan| {
                    matches!(&plan.kind, PlanKind::Campaign { target, .. }
                        if self.view().cities.iter().any(|city| city.id == *target
                            && self.sparta_chain_target(city)))
                }))
            || !self.campaign_withdraw.is_empty()
        {
            return;
        }
        let v = self.view();
        match self.routed_plan().map(|p| p.kind.clone()) {
            Some(PlanKind::Campaign { target, .. }) => {
                let Some(city) = v.cities.iter().find(|c| c.id == target) else {
                    return;
                };
                if v.at_war(v.me, city.owner) {
                    self.begin_campaign(city);
                }
            }
            Some(PlanKind::Contain { against })
                if self.decisions.contain_campaign > 0
                    && !self.takes_no_cities()
                    && v.teams_at_war(v.team, against) =>
            {
                let mut cities: Vec<(usize, &CityView)> = v
                    .cities
                    .iter()
                    .filter(|c| !c.owner.is_barbarian() && v.team_of(c.owner) == against)
                    .map(|c| (self.campaign_force(c).len(), c))
                    .filter(|(n, _)| *n >= 2)
                    .collect();
                cities.sort_by_key(|(n, c)| (std::cmp::Reverse(*n), c.id));
                for (_, city) in cities {
                    self.begin_campaign(city);
                    if self.campaigns.active.is_some() {
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    /// Declare and close on an exposed Settler. Adjacent targets are captured immediately;
    /// targets one turn away are pursued, with the plan retained for the next turn.
    pub(crate) fn execute_snipe(&mut self) {
        let Some(PlanKind::Snipe {
            against,
            settler,
            by,
        }) = self.routed_plan().map(|p| p.kind.clone())
        else {
            return;
        };
        let v = self.view();
        let declared = self
            .orders
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { against: a, .. } if *a == against));
        let Some(s) = v.units.iter().find(|u| u.id == settler) else {
            return;
        };
        let at_war = v.teams_at_war(v.team, against);
        if (declared || at_war) && !self.has_acted(by) {
            self.mil_dest.insert(s.pos);
            self.unit_order(
                by,
                Order::MoveUnit {
                    unit: by,
                    to: s.pos,
                },
            );
        }
    }

    pub(crate) fn execute_field_strike(&mut self) {
        let Some(PlanKind::FieldStrike {
            against,
            target,
            by,
        }) = self.routed_plan().map(|p| p.kind.clone())
        else {
            return;
        };
        let v = self.view();
        if !v.teams_at_war(v.team, against)
            && !self.orders.iter().any(|o| {
                matches!(o,
                Order::DeclareWar { against: t, .. } if *t == against)
            })
        {
            return;
        }
        let Some(enemy) = v.units.iter().find(|u| u.id == target) else {
            return;
        };
        let Some(attacker) = self.ctx.own_units.iter().find(|u| u.id == by) else {
            return;
        };
        let order = if attacker.ranged > 0 {
            Order::RangedAttack {
                unit: by,
                target: enemy.pos,
            }
        } else {
            // The strategic forecast paid for this battle. Movement preflight must not veto
            // its melee order using the peacetime tactical retreat threshold.
            self.assault_attacks.insert(by);
            Order::MoveUnit {
                unit: by,
                to: enemy.pos,
            }
        };
        self.mil_dest.insert(enemy.pos);
        self.unit_order(by, order);
    }

    /// A discretionary purchase was withdrawn: count it, and keep the first ones as events.
    pub(crate) fn record_blocked_expense(
        &mut self,
        city: CityId,
        item: ProductionItem,
        cost: i32,
        reason: BlockReason,
    ) {
        if !self.plan_routing {
            return;
        }
        let memory = &mut self.campaigns.plan;
        memory.blocked_expenses += 1;
        if memory.blocked_expenses <= BLOCKED_EVENTS_KEPT {
            memory.events.push(PlanEvent::BlockedExpense {
                turn: self.ctx.view.turn,
                city,
                item,
                cost,
                reason,
            });
        }
    }

    /// The plan's reserve: discretionary purchases (all but `protected` defense) are
    /// withdrawn, newest first, while the Gold left is under what the plan keeps back.
    pub(crate) fn veto_plan_reserve(&mut self, protected: &[Order]) {
        let Some(reserve) = self.routed_plan().map(|p| p.reserve).filter(|r| *r > 0) else {
            return;
        };
        while self.gold_left < reserve {
            let Some(i) = self
                .orders
                .iter()
                .rposition(|o| matches!(o, Order::BuyItem { .. }) && !protected.contains(o))
            else {
                return;
            };
            let Order::BuyItem { city, item } = self.orders.remove(i) else {
                unreachable!("found a purchase")
            };
            let refund = self.purchase_cost(city, item);
            self.gold_left += refund;
            self.record_blocked_expense(city, item, refund, BlockReason::PlanReserve);
        }
    }

    pub(crate) fn purchase_cost(&self, city: CityId, item: ProductionItem) -> i32 {
        self.ctx
            .own_cities
            .iter()
            .find(|c| c.id == city)
            .and_then(|c| c.detail.as_ref())
            .and_then(|d| d.options.iter().find(|o| o.item == item))
            .and_then(|o| o.buy_cost)
            .unwrap_or(0)
    }
}

/// Planning is Hard's: the plan layer routes war, peace and purchases only there.
pub(crate) fn routes_plans(difficulty: AiDifficulty) -> bool {
    difficulty == AiDifficulty::Hard
}

#[cfg(test)]
mod walled_siege_continuation_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::diplomacy::Relation;

    #[test]
    fn stopping_a_holder_has_value_when_our_solo_chance_is_zero() {
        let view = ViewBuilder::new(24, 16)
            .city(1, ME, Hex::new(4, 8), 3)
            .city(2, ENEMY, Hex::new(18, 8), 3)
            .build();
        let target = view.team_of(ENEMY);
        let mut facts = WinFacts::of(&view).refined();
        let rival = facts.teams.iter_mut().find(|f| f.team == target).unwrap();
        rival.vp = rival.target;
        rival.hold_left = Some(1);
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(ME)),
            view.faction_of(ME),
            1,
        );
        planner.decisions.contain_share = 250;
        assert!(planner.leader_race(0, &facts, target, 100, 10, 100) > 0);
        assert_eq!(planner.leader_race(0, &facts, target, 100, 0, 100), 0);
        assert_eq!(planner.leader_race(0, &facts, target, 0, 10, 100), 0);
        assert_eq!(planner.leader_race(0, &facts, target, 100, 10, 0), 0);
        planner.decisions.coalition_push = 0;
        assert_eq!(planner.leader_race(0, &facts, target, 100, 10, 100), 0);
    }

    #[test]
    fn a_positive_campaign_with_a_local_wall_siege_survives_one_failed_forecast() {
        let city_pos = Hex::new(14, 8);
        let mut view = ViewBuilder::new(30, 16)
            .city(1, ME, Hex::new(3, 8), 3)
            .city(9, ENEMY, city_pos, 2)
            .walled(9)
            .unit(1, ME, UnitKind::Ranger, Hex::new(12, 8))
            .unit(2, ME, UnitKind::Ranger, Hex::new(12, 9))
            .unit(3, ME, UnitKind::Soldier, Hex::new(12, 7))
            .unit(4, ME, UnitKind::Ranger, Hex::new(13, 9))
            .unit(5, ME, UnitKind::Ranger, Hex::new(13, 7))
            .unit(6, ME, UnitKind::Soldier, Hex::new(11, 8))
            .build();
        view.diplomacy.legacy = false;
        let against = view.team_of(ENEMY);
        view.diplomacy.relations.push(Relation {
            a: view.team,
            b: against,
            state: RelationState::War,
            effective_turn: 5,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(1),
            declarers: vec![view.team],
        });
        let mut planner = crate::fixture::planner(&view, 7);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.war_siege = 100;
        let kind = PlanKind::Campaign {
            against,
            target: CityId(9),
        };
        let plan = Plan {
            kind: kind.clone(),
            start_turn: view.turn - 1,
            review_turn: view.turn + 9,
            reserve: 0,
            value: 100,
            terms: Vec::new(),
            breach: false,
            misses: 0,
        };
        let mut candidate = Candidate {
            kind,
            value: 100,
            terms: Vec::new(),
            reserve: 0,
            breach: false,
        };
        assert!(planner.keeps_walled_siege(&plan, &candidate));
        candidate.value = 0;
        assert!(!planner.keeps_walled_siege(&plan, &candidate));
        planner.decisions.war_siege = 300;
        candidate.value = 100;
        assert!(!planner.keeps_walled_siege(&plan, &candidate));
    }
}

#[cfg(test)]
mod wall_staging_tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use legatum_core::diplomacy::Relation;

    #[test]
    fn an_existing_wall_war_can_stage_a_campaign_when_the_short_forecast_fails() {
        let target = Hex::new(14, 8);
        let mut view = ViewBuilder::new(30, 16)
            .city(1, ME, Hex::new(3, 8), 3)
            .city(9, ENEMY, target, 3)
            .walled(9)
            .unit(1, ME, UnitKind::Ranger, Hex::new(8, 8))
            .unit(2, ME, UnitKind::Ranger, Hex::new(8, 9))
            .unit(3, ME, UnitKind::Ranger, Hex::new(9, 8))
            .unit(4, ME, UnitKind::Ranger, Hex::new(9, 9))
            .unit(5, ME, UnitKind::Ranger, Hex::new(9, 7))
            .unit(6, ME, UnitKind::Ranger, Hex::new(10, 8))
            .unit(7, ME, UnitKind::Soldier, Hex::new(8, 7))
            .unit(8, ME, UnitKind::Soldier, Hex::new(10, 7))
            .build();
        view.diplomacy.legacy = false;
        let against = view.team_of(ENEMY);
        view.diplomacy.relations.push(Relation {
            a: view.team,
            b: against,
            state: RelationState::War,
            effective_turn: 5,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(1),
            declarers: vec![view.team],
        });
        let mut planner = crate::fixture::planner(&view, 7);
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.war_siege = 100;
        planner.decisions.siege_rounds = 1;
        let city = view.cities.iter().find(|c| c.id == CityId(9)).unwrap();
        assert!(planner.campaign_forecast(city, None).is_none());
        assert!(planner.wall_staging_forecast(city).is_some());
        assert!(planner.plan_candidates().iter().any(|c| matches!(
            c.kind,
            PlanKind::Campaign {
                target: CityId(9),
                ..
            }
        )));
    }
}

#[cfg(test)]
mod sparta_expedition_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::diplomacy::{DiplomaticAction, Relation};

    #[test]
    fn forced_sparta_plan_can_stage_before_a_distant_siege_is_forecastable() {
        let mut v = ViewBuilder::new(40, 20)
            .faction(Faction::Sparta)
            .city(1, ME, off(3, 5), 3)
            .city(2, ENEMY, off(18, 5), 3)
            .unit(1, ME, UnitKind::Soldier, off(5, 5))
            .unit(2, ME, UnitKind::Soldier, off(5, 6))
            .unit(3, ME, UnitKind::Soldier, off(5, 7))
            .build();
        v.diplomacy.legacy = false;
        let against = v.team_of(ENEMY);
        v.diplomacy.relations.push(Relation {
            a: v.team,
            b: against,
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
        v.diplomacy.actions.push(DiplomaticAction {
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
        });
        let mut p = Planner::new(&v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.sparta_force_campaign = 0;
        assert!(
            !p.plan_candidates()
                .iter()
                .any(|c| matches!(&c.kind, PlanKind::Campaign { .. }))
        );
        p.decisions.sparta_force_campaign = 1;
        p.decisions.sparta_valued_expedition = 0;
        assert!(p.plan_candidates().iter().any(|c| matches!(&c.kind, PlanKind::Campaign { target, .. } if *target == legatum_core::CityId(2))));
        p.decisions.sparta_valued_expedition = 1;
        assert!(
            !p.plan_candidates()
                .iter()
                .any(|c| matches!(&c.kind, PlanKind::Campaign { target, .. }
                if *target == legatum_core::CityId(2))),
            "an unpriced expedition must not replace the quiet plan"
        );
    }
}

#[cfg(test)]
mod march_past_reach_tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::diplomacy::{DiplomaticAction, Relation};

    /// The swiss-strongbox shape: shooters within the two-turn approach, the melee units that
    /// would take the city a few turns behind them.
    #[test]
    fn shooters_in_reach_do_not_hide_the_march_that_takes_an_open_city() {
        let mut v = ViewBuilder::new(40, 20)
            .faction(Faction::Mongols)
            .city(1, ME, off(3, 5), 3)
            .city(2, ENEMY, off(20, 5), 3)
            .unit(1, ME, UnitKind::Ranger, off(17, 5))
            .unit(2, ME, UnitKind::Ranger, off(17, 6))
            .unit(3, ME, UnitKind::Soldier, off(11, 5))
            .unit(4, ME, UnitKind::Soldier, off(11, 6))
            .unit(5, ME, UnitKind::Soldier, off(11, 4))
            .build();
        v.diplomacy.legacy = false;
        let against = v.team_of(ENEMY);
        v.diplomacy.relations.push(Relation {
            a: v.team,
            b: against,
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
        v.diplomacy.actions.push(DiplomaticAction {
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
        });
        let city = v.cities.iter().find(|c| c.id == CityId(2)).unwrap().clone();
        let forecast = |gene: i32| {
            let mut p = Planner::new(
                &v,
                Knobs::for_faction(Faction::Mongols),
                Faction::Mongols,
                1,
            );
            p.difficulty = AiDifficulty::Hard;
            p.decisions.march_past_reach = gene;
            p.campaign_forecast(&city, None)
        };
        assert!(forecast(0).is_none());
        let f = forecast(1).expect("the march forecast");
        assert!(
            f.march > 0 && f.force.len() >= 3,
            "{:?}",
            (f.march, f.force.len())
        );
    }
}
