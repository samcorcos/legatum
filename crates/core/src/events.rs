use crate::hex::Hex;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// A quote at the instant a find made a city's current Buy affordable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuinBuy {
    pub city: CityId,
    pub item: ProductionItem,
    pub cost: i32,
}

/// Filled in during research resolution, using this turn's actual cost and Science income.
/// Multiple finds credit the one that crosses the threshold, never every find in that turn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuinResearch {
    pub tech: TechId,
    pub cost: i32,
    pub progress_before: i32,
    pub income: i32,
    pub turns_saved: Option<i32>,
    pub completes: bool,
}

/// How an attack order on a unit ended this turn. An attack order lasts one turn and makes at
/// most one attack (G18); only `AwaitingWar` keeps it for the next turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PursuitOutcome {
    /// The target moved away and the attacker followed, but couldn't reach it: the order ended.
    Followed,
    /// The target moved away; the attacker followed and attacked it where it went.
    Attacked,
    /// The target left the team's sight; the attack order ended.
    LostSight,
    /// War with the target's team begins next turn; the attacker holds until then.
    AwaitingWar,
    /// The target was destroyed or disbanded before this unit reached it; the order ended.
    TargetGone,
    /// The target stayed out of this turn's reach: the attacker moved toward it and the order
    /// ended. Attacking needs a new order next turn.
    OutOfReach,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RuinReward {
    Gold {
        amount: i32,
        balance: i32,
        affordable: Option<RuinBuy>,
    },
    Science {
        amount: i32,
        research: Option<RuinResearch>,
    },
    Population {
        city: CityId,
        amount: i32,
        population: i32,
        yields: Yields,
    },
    FreeScout {
        unit: UnitId,
    },
}

impl RuinReward {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Gold { .. } => "Gold",
            Self::Science { .. } => "Science",
            Self::Population { .. } => "Population",
            Self::FreeScout { .. } => "FreeScout",
        }
    }
}

/// Unit attackers keep the original numeric JSON encoding. Experimental city shots use
/// {"city": id}, so city and unit IDs can never be confused and old event bytes stay intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Attacker {
    Unit(UnitId),
    City { city: CityId },
}

impl Attacker {
    pub fn unit(self) -> Option<UnitId> {
        match self {
            Self::Unit(u) => Some(u),
            Self::City { .. } => None,
        }
    }
}

impl From<UnitId> for Attacker {
    fn from(unit: UnitId) -> Self {
        Self::Unit(unit)
    }
}

impl std::fmt::Display for Attacker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit(u) => write!(f, "unit {}", u.0),
            Self::City { city } => write!(f, "city {}", city.0),
        }
    }
}

impl PartialEq<UnitId> for Attacker {
    fn eq(&self, other: &UnitId) -> bool {
        self.unit() == Some(*other)
    }
}

/// What the defender in a battle was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Defender {
    Unit {
        unit: UnitId,
        owner: PlayerId,
        unit_kind: UnitKind,
    },
    City {
        city: CityId,
        owner: PlayerId,
    },
}

/// One city's share of the New Fire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwarmLoss {
    pub city: CityId,
    pub lost: i32,
    /// Population after the sacrifice.
    pub population: i32,
    /// Territory tiles the city released with it.
    pub tiles: u32,
}

/// The turn report. Every outcome of resolution is an event, with the numbers that decided it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    WonderDiscovered {
        owner: PlayerId,
        at: Hex,
        wonder: crate::wonders::Wonder,
        gold: i32,
        science: i32,
    },
    TreatyActivated {
        relation: crate::diplomacy::Relation,
    },
    TreatyEnded {
        treaty_id: u32,
        a: TeamId,
        b: TeamId,
        effective_turn: u32,
        reason: String,
    },
    TreatyBreached {
        treaty_id: u32,
        by: TeamId,
        with: TeamId,
        player: PlayerId,
        cost: i32,
        effective_turn: u32,
    },
    /// A diplomatic Gold payment for breaking an Alliance early.
    TreatyPayment {
        player: PlayerId,
        gold: i32,
        cause: String,
        treaty_id: Option<u32>,
        effective_turn: u32,
    },
    /// `betrayal.by` left an Alliance inside its protected window or declared war on its ally.
    AllianceBetrayed {
        betrayal: crate::diplomacy::Betrayal,
    },
    WarDeclared {
        relation: crate::diplomacy::Relation,
    },
    /// An Alliance carried `team` into `ally`'s war (declared, or already under way when the
    /// Alliance was signed): `relation` is `team`'s war with the enemy, whose `declarers` are
    /// the war's.
    WarJoined {
        relation: crate::diplomacy::Relation,
        team: TeamId,
        ally: TeamId,
    },
    PeaceEstablished {
        relation: crate::diplomacy::Relation,
    },
    /// `call.ally` declared war on `call.target` (or was already at war with it when the
    /// Alliance was signed): `call.team`, its ally, may join or decline (`WarCall`).
    CallToArms {
        call: crate::diplomacy::WarCall,
    },
    /// The called team answered: `joined` puts it at War with the target (its `WarJoined` comes
    /// with this event); declining keeps the Alliance and the Peace.
    CallAnswered {
        call: crate::diplomacy::WarCall,
        joined: bool,
        effective_turn: u32,
    },
    /// A call ended unanswered or can no longer be answered (the window passed, the war ended,
    /// the Alliance ended, someone was eliminated).
    CallLapsed {
        call: crate::diplomacy::WarCall,
        effective_turn: u32,
        reason: String,
    },
    ProposalCreated {
        proposal: crate::diplomacy::PeaceProposal,
    },
    ProposalAccepted {
        proposal: crate::diplomacy::PeaceProposal,
        effective_turn: u32,
    },
    ProposalRejected {
        proposal: crate::diplomacy::PeaceProposal,
        effective_turn: u32,
    },
    ProposalWithdrawn {
        proposal: crate::diplomacy::PeaceProposal,
        effective_turn: u32,
    },
    ProposalExpired {
        proposal: crate::diplomacy::PeaceProposal,
        effective_turn: u32,
    },
    ProposalInvalidated {
        proposal: crate::diplomacy::PeaceProposal,
        effective_turn: u32,
        reason: String,
    },
    OrderSuperseded {
        decision: crate::diplomacy::SupersededOrder,
    },
    UnitAwakened {
        unit: UnitId,
        owner: PlayerId,
        unit_kind: UnitKind,
        at: Hex,
    },
    /// A unit on a heal order reached full HP; the order ended and it needs orders.
    UnitHealed {
        unit: UnitId,
        owner: PlayerId,
        unit_kind: UnitKind,
        at: Hex,
    },
    AutomationStopped {
        unit: UnitId,
        owner: PlayerId,
        at: Hex,
        reason: ExploreStopReason,
    },
    UnitMoved {
        unit: UnitId,
        owner: PlayerId,
        from: Hex,
        to: Hex,
    },
    MoveBlocked {
        unit: UnitId,
        owner: PlayerId,
        at: Hex,
        toward: Hex,
        reason: String,
    },
    /// A unit stepping into its owner's own city reached it before an enemy's attack on that
    /// city this round and became its garrison (`rules::movement`, "reinforcement goes first"),
    /// or, as a supporter, took over as its garrison after the round's fighting when the one
    /// before it fell or stepped out (supported defence, `rules::battle::carried_damage`,
    /// `battle::soak`). Follows the unit's `UnitMoved`. A supporter takes its share of the
    /// assault where it stands and has no `Battle` of its own; one that falls has a
    /// `UnitDestroyed` on its own tile.
    CityReinforced {
        unit: UnitId,
        owner: PlayerId,
        unit_kind: UnitKind,
        city: CityId,
        at: Hex,
    },
    Bounce {
        units: Vec<UnitId>,
        tile: Hex,
    },
    /// What became of an explicit melee attack order whose target is a unit (private to the
    /// attacker's team). `target_at` is where the target ended, if the attacker still sees it.
    AttackPursuit {
        unit: UnitId,
        owner: PlayerId,
        at: Hex,
        target: UnitId,
        target_owner: PlayerId,
        target_kind: UnitKind,
        target_at: Option<Hex>,
        outcome: PursuitOutcome,
    },
    Battle {
        attacker: Attacker,
        attacker_owner: PlayerId,
        defender: Defender,
        at: Hex,
        /// Where the attacker stood when it fought: the shooter's or striking city's tile, or
        /// the tile a melee attacker struck from. Player reports drop it (`None`) when the
        /// viewer's team saw neither that tile nor the attacking unit itself.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attacker_at: Option<Hex>,
        attacker_strength: Breakdown,
        defender_strength: Breakdown,
        /// Damage dealt to each side, and the rolls (80-120%) behind it.
        damage_to_attacker: i32,
        damage_to_defender: i32,
        attacker_roll: i32,
        defender_roll: i32,
        attacker_hp_after: i32,
        defender_hp_after: i32,
        /// Ranged attack (no damage to the attacker).
        ranged: bool,
        /// Public report that the attacker was an aircraft; its exact kind can remain hidden.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        air_strike: bool,
        /// Meeting engagement: both sides were moving, no defensive bonuses.
        meeting: bool,
        /// A melee hit on a unit moving away (G21): it defended without fortification and,
        /// if it survived, went on with its move.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        defender_leaving: bool,
    },
    UnitDestroyed {
        unit: UnitId,
        owner: PlayerId,
        unit_kind: UnitKind,
        at: Hex,
    },
    SpartaKillHeal {
        player: PlayerId,
        unit: UnitId,
        victim_owner: PlayerId,
        unit_kind: UnitKind,
        hp_healed: i32,
    },
    UnitDisbanded {
        unit: UnitId,
        owner: PlayerId,
        reason: String,
    },
    SettlerCaptured {
        unit: UnitId,
        from: PlayerId,
        to: PlayerId,
        at: Hex,
    },
    CityFounded {
        city: CityId,
        owner: PlayerId,
        name: String,
        at: Hex,
        /// A Roman melee veteran was consumed to found this city.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        veteran: bool,
    },
    /// Flower Wars: one captive grows the nearest city; contains no hidden battle details.
    FlowerWars {
        city: CityId,
        owner: PlayerId,
        population: i32,
    },
    CityGrew {
        city: CityId,
        owner: PlayerId,
        population: i32,
    },
    CityStarved {
        city: CityId,
        owner: PlayerId,
        population: i32,
    },
    CityCaptured {
        city: CityId,
        from: PlayerId,
        to: PlayerId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bounty: Option<i32>,
    },
    /// Public: a first capture of a city owned by Athens grants the captor one free technology.
    AthensCityCaptured {
        city: CityId,
        from: PlayerId,
        to: PlayerId,
    },
    /// Public: this empire now holds a rival's original capital and scores its 1 VP while it
    /// holds it (reported at the end of the turn it changed hands, like a world wonder).
    CapitalTaken {
        captor: PlayerId,
        capital: CityId,
        name: String,
    },
    /// Public: this empire no longer holds a rival's original capital it scored (retaken by
    /// anyone, or razed), so its 1 VP is gone the same turn.
    CapitalLost {
        player: PlayerId,
        capital: CityId,
        name: String,
    },
    /// Team only: Gold one empire gave a teammate (`Order::GiveGold`), after this turn's
    /// purchases. `amount` is what moved, capped at the Gold the giver had left.
    GoldGiven {
        from: PlayerId,
        to: PlayerId,
        amount: i32,
    },
    /// Public: a Celtic empire burned a rival's original capital in a sack and keeps its 1 VP
    /// for the rest of the game.
    CapitalSacked {
        captor: PlayerId,
        capital: CityId,
        name: String,
    },
    SpartaConquest {
        city: CityId,
        player: PlayerId,
        units_healed: u32,
    },
    /// A Celtic capture sack or an ordered Raze: the city burned. `sack_gold` is the Gold taken
    /// (60 per population for a sack, 10 for an ordered Raze); `camp` marks a Celtic warband camp.
    CityRazed {
        city: CityId,
        from: PlayerId,
        by: PlayerId,
        at: Hex,
        /// Original-capital status before the city was removed; only automatic sacks count as captures.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        original_capital: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sack_gold: Option<i32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        camp: Option<Hex>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        ordered: bool,
        /// The city was a capital or held a wonder (World Age Fight cards never count it).
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        prize: bool,
        /// The city had Walls of any tier.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        walled: bool,
    },
    BuildingDismantled {
        city: CityId,
        owner: PlayerId,
        building: BuildingKind,
        reason: String,
    },
    ItemBought {
        city: CityId,
        owner: PlayerId,
        item: ProductionItem,
        gold: i32,
    },
    /// Byzantium hired a Mercenary Company (`company`): `gold` went to `seller`, and the unit
    /// serves until the end of turn `until`.
    CompanyHired {
        city: CityId,
        owner: PlayerId,
        seller: PlayerId,
        unit: UnitKind,
        gold: i32,
        until: u32,
    },
    /// A Company's contract ended and it went home from `pos`.
    CompanyDeparted {
        unit: UnitKind,
        owner: PlayerId,
        origin: Option<Faction>,
        pos: Hex,
    },
    /// An unpaid Company turned barbarian where it stood, at `pos`.
    CompanyTurned {
        unit: UnitKind,
        owner: PlayerId,
        origin: Option<Faction>,
        pos: Hex,
    },
    /// Byzantium's Heir was born in `city` (public: every empire is told, and his position is
    /// public from then on; whoever captures him scores his goal).
    HeirBorn {
        city: CityId,
        owner: PlayerId,
        city_name: String,
        at: Hex,
    },
    ProductionCompleted {
        city: CityId,
        owner: PlayerId,
        item: ProductionItem,
    },
    MegaprojectStarted {
        city: CityId,
        owner: PlayerId,
        building: BuildingKind,
    },
    WonderRaceLost {
        city: CityId,
        owner: PlayerId,
        winner: PlayerId,
        building: BuildingKind,
        production: i32,
        gold: i32,
        /// The city was building it (it stopped); otherwise it was only queued or banked there.
        #[serde(default)]
        active: bool,
    },
    TechResearched {
        player: PlayerId,
        tech: TechId,
    },
    /// An empire finished the last technology of its era and entered the next (`eras`). Public.
    /// `era` is the era entered, **1-based** like the VP model (2 Classical .. 8 Future; the
    /// view's `ResearchView::era` is 0-based). `first` is set for the first empire into that era (`GameState::era_firsts`): the UI
    /// and AI treat it as the big moment. The full-screen era-up takeover (phase C) keys off
    /// this event for the receiving player.
    EraEntered {
        player: PlayerId,
        era: u8,
        first: bool,
    },
    /// Research completed and the plan advanced; None means choose a new plan.
    ResearchQueueAdvanced {
        player: PlayerId,
        next: Option<TechId>,
    },
    /// A city claimed a tile as its population grew (population is territory).
    TileClaimed {
        city: CityId,
        owner: PlayerId,
        tile: Hex,
    },
    /// A city bought a tile with Gold (Order::BuyTile).
    TileBought {
        city: CityId,
        owner: PlayerId,
        tile: Hex,
        gold: i32,
    },
    /// A city lost population (a Settler, starvation or capture) and gave up a grown tile.
    TileReleased {
        city: CityId,
        owner: PlayerId,
        tile: Hex,
    },
    /// Existing land passed between two cities of the same empire so it can be worked.
    TilesReassigned {
        from: CityId,
        to: CityId,
        owner: PlayerId,
        tiles: Vec<Hex>,
    },
    /// An aircraft moved to another city's hangar (`Order::Rebase`).
    AircraftRebased {
        plane: UnitId,
        owner: PlayerId,
        kind: UnitKind,
        from: CityId,
        to: CityId,
    },
    RuinExplored {
        unit: UnitId,
        owner: PlayerId,
        at: Hex,
        reward: RuinReward,
    },
    GreatWorkCompleted {
        player: PlayerId,
        of: PlayerId,
        building: BuildingKind,
        vp: u32,
        again: bool,
    },
    GreatWorkLost {
        player: PlayerId,
        of: PlayerId,
        building: BuildingKind,
        vp: u32,
    },
    /// An empire completes an Age's card and earns that Age's 1 permanent VP. `slot` is the
    /// card's slot in its Age (always 0). Public.
    EraObjectiveScored {
        player: PlayerId,
        era: u8,
        slot: u8,
        objective: String,
    },
    /// World Ages: an Age began, so its card is open to every empire.
    /// `era` is 1-based like `EraEntered`; `first` is the empire that entered the era first (it may
    /// be you). Emitted once per Age, the turn `objectives::update` opens it. Public.
    AgeBegan {
        era: u8,
        first: PlayerId,
    },
    /// An empire's own faction goal starts scoring: first completed, or (`again`) a Live goal
    /// regained.
    FactionGoalCompleted {
        player: PlayerId,
        faction: Faction,
        vp: u32,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        again: bool,
    },
    /// A Live faction goal stopped holding (the Switzerland spent below their hoard): its VP is
    /// gone until it holds again.
    FactionGoalLost {
        player: PlayerId,
        faction: Faction,
        vp: u32,
    },
    /// A legacy Sakoku scenario closed the country; the vow is public.
    SakokuClosed {
        player: PlayerId,
    },
    /// A Sakoku empire reopened the country: `turns` turn ends were spent closed, and the countdown
    /// restarts at the next vow. Public.
    SakokuReopened {
        player: PlayerId,
        turns: u32,
    },
    /// The Aztecs kindled the New Fire (`Action::Swarm`): each city's sacrifice. Public.
    SwarmUnleashed {
        player: PlayerId,
        cities: Vec<SwarmLoss>,
    },
    /// The Athens broadcast (`Action::Broadcast`): `receivers` each owe `techs_each` free
    /// technologies of their own choice. Public.
    SignalBroadcast {
        from: PlayerId,
        receivers: Vec<PlayerId>,
        techs_each: u8,
    },
    /// The first Courier Hall's roads were laid (`roads::extend`): every city now joins `city`,
    /// the Hall's city of `owner` (a Mongols empire), at `at`; `linked` cities were joined.
    /// Public.
    RoadsLaid {
        owner: PlayerId,
        city: CityId,
        at: crate::hex::Hex,
        linked: u32,
    },
    /// The Sparta retired a veteran (`Action::RetireVeteran`): `retired` of `of` so far.
    /// Visible to the retiring team; the goal's completion is public.
    VeteranRetired {
        player: PlayerId,
        unit: UnitId,
        kind: UnitKind,
        city: CityId,
        retired: u8,
        of: u8,
    },
    /// The Celts called the warbands (`Action::CallTheWild`): `camps` new wild camps stand in their
    /// territory (each also has a full-log `BarbarianCampSpawned`). Public.
    WildCalled {
        player: PlayerId,
        camps: u32,
    },
    /// An empire spent a broadcast's free technologies (`Action::ChooseFreeTechs`). Each also
    /// has its own `TechResearched` line. Visible to the chooser's team.
    FreeTechsChosen {
        player: PlayerId,
        from: Option<PlayerId>,
        techs: Vec<TechId>,
    },
    /// A unit was given Siege (`Order::UpgradeSiege`).
    SiegeBought {
        unit: UnitId,
        owner: PlayerId,
        gold: i32,
    },
    UnitUpgraded {
        unit: UnitId,
        owner: PlayerId,
        to: UnitKind,
        gold: i32,
    },
    /// A barbarian camp appeared on a tile no empire could see. In the full log only: no team
    /// is shown it (a team learns of a camp by seeing it: `BarbarianCampDiscovered`).
    BarbarianCampSpawned {
        camp: u32,
        at: Hex,
    },
    /// `team` saw this barbarian camp for the first time.
    BarbarianCampDiscovered {
        camp: u32,
        team: TeamId,
        at: Hex,
    },
    /// A barbarian scout saw `city` and turned back toward its camp. Killing it before it gets
    /// home prevents the raid.
    BarbarianScoutSpotted {
        unit: UnitId,
        city: CityId,
        owner: PlayerId,
        at: Hex,
    },
    /// A scout reached its camp: raiders muster there and march on `city`. `direction` is the
    /// compass direction from the city toward the camp, so the camp's tile stays hidden.
    BarbarianRaidLaunched {
        camp: u32,
        city: CityId,
        owner: PlayerId,
        direction: String,
    },
    /// A camp is gone: a military unit entered it (`by`, `unit`, `gold` reward) or an empire's
    /// borders grew over it (`by` is the tile owner, no unit and no Gold).
    BarbarianCampCleared {
        camp: u32,
        at: Hex,
        by: PlayerId,
        unit: Option<UnitId>,
        gold: i32,
    },
    /// A strategic resource became usable (G11): deposits near `city` now add `supply`, or
    /// (with no city) a technology revealed the resource with no deposit inside the empire's
    /// borders yet. Team-private.
    ResourcesRevealed {
        player: PlayerId,
        resource: crate::resources::Resource,
        city: Option<CityId>,
        sources: u32,
        #[serde(alias = "per_turn")]
        supply: i32,
    },
    /// The number of this empire's buildings and units idle for lack of `resource` changed
    /// (0: supply covers every user again). Team-private.
    SupplyShortage {
        player: PlayerId,
        resource: crate::resources::Resource,
        idle: u32,
        supplied: i32,
        used: u32,
    },
    /// A unit's combat experience reached a new promotion level (see rules::promotion).
    UnitPromoted {
        unit: UnitId,
        owner: PlayerId,
        unit_kind: UnitKind,
        level: u8,
        at: Hex,
    },
    /// An order that could not be carried out at resolution time.
    OrderFailed {
        player: PlayerId,
        reason: String,
    },
    PlayerEliminated {
        player: PlayerId,
    },
    /// A team reached its VP target and must now hold it for `turns_needed` turn ends to win.
    /// Public: every player gets the alarm.
    VictoryHoldStarted {
        team: TeamId,
        vp: u32,
        target: u32,
        turns_needed: u32,
    },
    /// A team that was holding its VP target dropped below it (or was eliminated): its count is
    /// back to zero. Public.
    VictoryHoldBroken {
        team: TeamId,
        turns_held: u32,
        vp: u32,
        target: u32,
    },
    GameWon {
        team: TeamId,
        reason: String,
    },
    /// The game ended with no winner (exact tie).
    GameDrawn {
        reason: String,
    },
}
