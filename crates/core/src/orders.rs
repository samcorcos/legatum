use crate::hex::Hex;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Everything a player can tell the game in one turn. Orders are collected from every player,
/// then resolved together (see rules::resolve_turn).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Order {
    DeclareWar {
        against: TeamId,
        #[serde(default)]
        breach: bool,
    },
    /// Leave an Alliance; takes effect next turn. Inside its protected window it costs Gold
    /// and is a betrayal.
    BreakTreaty {
        with: TeamId,
        agreement: crate::diplomacy::Agreement,
    },
    ProposeTreaty {
        to: TeamId,
        kind: crate::diplomacy::TreatyKind,
    },
    RespondToProposal {
        proposal: crate::diplomacy::ProposalId,
        accept: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        explanation: Option<crate::diplomacy::DecisionExplanation>,
    },
    WithdrawProposal {
        proposal: crate::diplomacy::ProposalId,
    },
    /// Set intent and optionally execute a client-planned route for this turn only.
    /// The route excludes the starting tile. Core validates movement, never chooses goals.
    SetAutomation {
        unit: UnitId,
        automation: Option<Automation>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        path: Vec<Hex>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stop_reason: Option<ExploreStopReason>,
    },
    /// A ranged unit in its own city automatically fires at the best visible hostile target.
    SetAutoFire {
        unit: UnitId,
        enabled: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<Hex>,
    },
    /// Set a persistent destination. The core pathfinds every turn until the unit arrives.
    MoveUnit {
        unit: UnitId,
        to: Hex,
    },
    /// Follow the confirmed preview (excluding the starting hex), across turns if needed.
    /// The route uses explored, adjacent land tiles. Combat can interrupt it.
    MoveUnitPath {
        unit: UnitId,
        to: Hex,
        path: Vec<Hex>,
        /// Keep the route and destination when blocked; only a new order may reroute.
        /// Use for paths whose intermediate tiles satisfy planner-only constraints.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        stop_on_block: bool,
    },
    /// Stay put until woken, given another order, or a new enemy military unit comes into sight
    /// within the unit's movement range. Heals and fortifies as any resting unit does, and does
    /// not move, shoot or fight. The one standing idle order, for every unit.
    /// Recorded order tapes name it `HoldUnit` or `HealUnit`.
    ///
    /// With `until_healed` the unit rests the same way but the standing order ends by itself the
    /// turn its HP is full: it then asks for orders again (`Event::UnitHealed`). Old tapes never
    /// carry the flag, so they replay as before. Refused for a unit at full HP or one that would
    /// not heal where it stands.
    #[serde(alias = "HoldUnit", alias = "HealUnit")]
    SleepUnit {
        unit: UnitId,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        until_healed: bool,
    },
    /// Do nothing this turn. Clears destinations, automation and a standing Sleep; the unit asks
    /// for orders again next turn.
    SkipUnit {
        unit: UnitId,
    },
    FoundCity {
        unit: UnitId,
    },
    DisbandUnit {
        unit: UnitId,
    },
    /// Burn a captured city at the turn boundary for population × 10 Gold.
    Raze {
        city: CityId,
    },
    SetProduction {
        city: CityId,
        item: ProductionItem,
    },
    /// Keep the current build; change what the client will choose at Ready.
    SetCityAutomation {
        city: CityId,
        focus: Option<CityBuildFocus>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        picked: Option<ProductionItem>,
    },
    /// Replace the complete plan, including its active first item. Empty means idle.
    SetProductionQueue {
        city: CityId,
        queue: Vec<ProductionItem>,
    },
    /// Buy a separate item at full price without spending the city's production.
    BuyItem {
        city: CityId,
        item: ProductionItem,
    },
    /// Byzantium only: hire a Mercenary Company (`company`), an armed unit `seller` can field
    /// now, paid to `seller` in Gold. It appears in the city next turn and goes home when its
    /// contract ends. Takes the city's one purchase for the turn.
    HireCompany {
        city: CityId,
        unit: UnitKind,
        seller: PlayerId,
    },
    /// Buy an unowned tile next to the city's territory, within its work radius, for Gold.
    /// Shares the city's one purchase per turn with BuyItem and HireCompany.
    BuyTile {
        city: CityId,
        tile: Hex,
    },
    SetResearch {
        tech: TechId,
    },
    /// Replace the whole plan (including the current technology). Empty cancels
    /// research; stored science is retained. Prerequisites must precede dependents.
    SetResearchQueue {
        techs: Vec<TechId>,
    },
    /// Choose the yield generated from Science after the last technology; persists across turns.
    SetScienceConversion {
        to: ScienceConversion,
    },
    /// Ranged units only: attack a target within range without moving. No retaliation.
    RangedAttack {
        unit: UnitId,
        target: Hex,
    },
    /// Upgrade in own territory for gold (see data::UPGRADE_COST_PERCENT). Unit acts next turn.
    UpgradeUnit {
        unit: UnitId,
    },
    /// Buy Siege for a land military unit in own territory (`rules::siege`): its attacks on a
    /// city then ignore enemy Walls, for good. Costs Gold and 1 Iron supply; the unit acts next
    /// turn.
    UpgradeSiege {
        unit: UnitId,
    },
    /// An aircraft in a city's hangar strikes a tile: a Bomber within `data::BOMBER_RANGE` of its
    /// city, a Fighter within `data::FIGHTER_STRIKE_RANGE`, in the ranged phase with no
    /// retaliation, unless a Fighter on guard intercepts (`rules::air`). A Fighter that strikes
    /// has used its turn and does not intercept.
    AirStrike {
        plane: UnitId,
        target: Hex,
    },
    /// An aircraft moves to another own city's hangar with room, at any distance, at the end
    /// of the ranged phase; it cannot strike that turn.
    Rebase {
        plane: UnitId,
        to: CityId,
    },
    /// An aircraft in a city's hangar scouts a tile within its strike range (any tile on the map,
    /// seen or not): the tiles within `data::RECON_RADIUS` of it are in the team's sight for the
    /// next turn only, so a strike can be ordered then (`rules::air`). It uses the aircraft's one
    /// order, so it cannot strike that turn; a Fighter still guards.
    AirRecon {
        plane: UnitId,
        target: Hex,
    },
    /// Upgrade a based aircraft to its newest researched form for Gold (and a spare of the new
    /// form's resource when it changes): a Fighter or Bomber to its Stealth form, a Biplane to a
    /// Fighter or a Bomber. `to` picks the line where there is a choice (the Biplane's: Fighter
    /// or Bomber, `data::upgrade_choices`); None takes the default line.
    UpgradeAircraft {
        plane: UnitId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<UnitKind>,
    },
    /// A one-time empire or unit action (the faction goals' rites): see [`Action`].
    Action {
        action: Action,
    },
    /// Give Gold to a living teammate (same team, never yourself). It moves after every empire's
    /// purchases this turn, capped at the Gold the giver has left then, so the teammate can spend
    /// it from next turn (`rules::gifts`).
    GiveGold {
        to: PlayerId,
        amount: i32,
    },
}

/// The one-time actions an empire can order (`Order::Action`). One order variant carries them
/// all, so a new action adds a variant here, its check in `rules::actions::validate_action` and
/// its resolution in `rules::actions::apply_actions`, never a new order shape. Serialized with
/// the tag `"action"`: `{"type":"Action","action":{"action":"Swarm"}}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Action {
    /// The Swarm (`goals::GoalRule::Swarm`; the Aztecs' New Fire): sacrifice
    /// `goals::AZTEC_NEW_FIRE_POPULATION` population, biggest cities first, every city keeping at least 1.
    /// Completes the goal for good.
    Swarm,
    /// Japan, Sakoku (`goals::GoalRule::Sakoku`): `close: true` closes the country, `false`
    /// reopens it. While closed, the empire can't declare war or join an Alliance, gets no tech
    /// diffusion, and no foreign unit may enter its land at Peace. The goal is live: after
    /// `goals::sakoku_target` turn ends closed, it scores regardless of enemy units in its
    /// land. Reopening loses the VP and restarts the count.
    Sakoku { close: bool },
    /// Athens, Share the Signal: every living empire outside the broadcaster's team
    /// gains free technologies of its own choice. Completes the Athens' goal for good.
    Broadcast,
    /// Spend a broadcast's free technologies (`Player::free_techs_pending`): 1 to the pending
    /// count of unknown technologies, each with its prerequisites known or listed before it.
    ChooseFreeTechs { techs: Vec<TechId> },
    /// Sparta, Honour the Fallen: retire a veteran (promotion level
    /// `goals::SPARTA_RETIRE_LEVEL` or higher) standing in an own city with a War Shrine. The
    /// unit leaves for good and counts toward `goals::SPARTA_RETIRE_COUNT`; the last one
    /// completes the goal for good.
    RetireVeteran { unit: UnitId },
    /// Celts, Call the warbands: raise a barbarian camp in the territory of every Celts city, once per
    /// turn. The goal is live: held while every city owned has a camp in its territory.
    CallTheWild,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderError {
    /// Index into the submitted orders.
    pub index: usize,
    pub message: String,
}
