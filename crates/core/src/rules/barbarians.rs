//! Barbarians: Civ 6 style camps that raid the empires around them.
//!
//! # Representation
//!
//! Barbarian units are ordinary [`crate::state::Unit`]s owned by the reserved
//! [`PlayerId::BARBARIAN`], on the reserved [`TeamId::BARBARIAN`], which is at war with every
//! team (`GameState::teams_at_war`) and has no diplomatic relation. They are not in
//! `GameState::players`, so they never get VP, titles, a turn report, a brain, a diplomacy
//! row or a coalition, and they are never listed as an empire. Keeping them in
//! `GameState::units` lets movement, combat, civilian capture, sight and the fog-of-war views
//! handle them with no special cases; `GameState::player` returns a shared read-only record
//! (neutral India unit stats) for rules that read a unit owner's faction. The camps and raids
//! live here, in [`BarbarianState`], which is `None` when the game has barbarians off.
//!
//! # Rules (Civ 6, with sources in docs/sim-results/g10-barbarians.md)
//!
//! - **Camps** appear from turn [`FIRST_CAMP_TURN`] on unowned, passable land that
//!   no empire can see, at least [`CAMP_MIN_CITY_DISTANCE`] tiles from every city,
//!   [`CAMP_MIN_UNIT_DISTANCE`] from every empire's unit and [`CAMP_MIN_CAMP_DISTANCE`] from
//!   another camp. A third of the map's camp target appears at once (Civ 6 places 33% at
//!   the start), then one more every [`CAMP_INTERVAL`] turns while below the target (one camp
//!   per [`LAND_TILES_PER_CAMP`] land tiles, at most one per two empires).
//! - **Each camp** starts with a guard that never leaves and a scout. A slow timer replaces a
//!   lost guard or scout and adds one roaming raider. Roamers wander within [`ROAM_RADIUS`] of
//!   the camp but keep [`ROAM_CITY_DISTANCE`] from every city; only raids come close. Units
//!   of a cleared camp join the nearest camp with room.
//! - **Scouts** wander up to [`SCOUT_RANGE`] tiles from camp. A scout that sees a city turns
//!   for home; if it gets back alive the camp launches a raid on that city: [`RAID_SIZE`] units
//!   (melee, ranged, melee) muster at the camp, one per [`RAID_SPAWN_INTERVAL`] turns, and march on it.
//!   Killing the scout on its way home stops the raid, as in Civ 6. After a raid the camp
//!   waits [`RAID_COOLDOWN`] turns before its scout reports again (Civ 6's raid throttle).
//! - **Raiders** march on the raided city and prowl [`RAID_PROWL`] tiles from it for
//!   [`RAID_TURNS`] turns, capturing lone civilians and attacking empire units
//!   in the open for a kill or a clear edge ([`RAID_EDGE`]; ranged raiders shoot them). Like Civ 6 below
//!   Emperor they never assault the city itself, and they never capture or raze one
//!   (simplified: Civ 6 razes non-capitals on its hardest raids). Captured civilians become
//!   barbarian and walk to a camp; clearing the camp (or catching them on the way) captures
//!   them back. Barbarians never heal (Civ 6: only by pillaging; this game has no pillage)
//!   and never explore ruins.
//! - **Clearing**: an empire's military unit ending its move on a camp destroys it and its
//!   owner gets [`CAMP_CLEAR_GOLD`] Gold (Civ 6's Prince value). Borders growing over a camp disperse it with no reward.
//! - **Units by era**: barbarians field Pikemen (and later Assault Pikemen) once half of the
//!   living empires know Feudalism (Gunpowder), and Crossbowmen once half know Machinery
//!   (Civ 6's `BARBARIAN_TECH_PERCENT` = 50).
//! - **Experience** (`rules::promotion::combat_xp`): fighting barbarians earns a unit XP only
//!   up to its first promotion ([`cap_barbarian_xp`], Civ 6's `EXPERIENCE_MAX_BARB_LEVEL` = 2,
//!   counting the unpromoted level as 1). Barbarian units never gain XP.
//! - **Knowledge**: barbarians decide from their own units' sight (`native_visible_tiles` of
//!   their team), and empires see camps only where they have sight, remembering the last
//!   known camps per team ([`BarbarianState::known`]).

use super::is_military;
use crate::bitset::BitSet;
use crate::combat;
use crate::data;
use crate::events::Event;
use crate::hex::Hex;
use crate::los;
use crate::rng::{Rng, purpose};
use crate::state::{City, GameState, Map};
use crate::types::*;
use crate::visibility;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// First turn a camp can appear.
pub const FIRST_CAMP_TURN: u32 = 10;
/// Turns between new camps once the first wave is placed.
pub const CAMP_INTERVAL: u32 = 12;
/// Camp target: one per this many passable land tiles, at most one per two empires.
pub const LAND_TILES_PER_CAMP: usize = 120;
/// Civ 6 uses 4; our cities sit closer together (4 apart, 2-tile work radius), and at 4 a
/// camp's guard pinned many capitals' first settling sites (sims: 5-7% of empires never
/// founded a second city), so camps keep two more tiles away.
pub const CAMP_MIN_CITY_DISTANCE: i32 = 6;
pub const CAMP_MIN_UNIT_DISTANCE: i32 = 3;
pub const CAMP_MIN_CAMP_DISTANCE: i32 = 6;
/// The camp's slow timer (replace the guard or scout, or add a roamer), in turns.
pub const CAMP_SPAWN_INTERVAL: u32 = 20;
/// Units in a raid wave, mustered one per `RAID_SPAWN_INTERVAL` turns.
pub const RAID_SIZE: u8 = 3;
pub const RAID_SPAWN_INTERVAL: u32 = 2;
/// Raiders and roamers a camp can field at once (the guard and scout not included).
pub const MAX_RAIDERS: usize = 4;
pub const SCOUT_RANGE: i32 = 7;
pub const ROAM_RADIUS: i32 = 3;
/// Raiders attack a unit in the open only to kill it or when they expect to deal this much
/// more damage than they take.
pub const RAID_EDGE: i32 = 10;
/// Raiders prowl this far from the raided city.
pub const RAID_PROWL: i32 = 2;
/// Turns a raid lasts before its raiders go home.
pub const RAID_TURNS: u32 = 12;
/// Turns after a raid ends before the camp can raid again (Civ 6 throttles raids).
pub const RAID_COOLDOWN: u32 = 30;
/// Roamers (not raiding) keep at least this far from every city, so a camp's everyday
/// wandering never besieges a neighbour; only raids come close.
pub const ROAM_CITY_DISTANCE: i32 = 5;
/// Gold for clearing a camp.
pub const CAMP_CLEAR_GOLD: i32 = 50;
/// Fighting barbarians can earn a unit this many promotion levels, no more (Civ 6).
pub const BARBARIAN_XP_MAX_LEVEL: u8 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarbarianState {
    /// Live camps in id order.
    pub camps: Vec<Camp>,
    pub next_camp_id: u32,
    /// Turn the last camp appeared; 0 before the first wave.
    pub last_camp_turn: u32,
    /// Last-known camp tiles per team: what each team's fog of war shows.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub known: BTreeMap<TeamId, BTreeSet<Hex>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Camp {
    pub id: u32,
    pub pos: Hex,
    pub founded_turn: u32,
    pub guard: Option<UnitId>,
    pub scout: Option<UnitId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raiders: Vec<UnitId>,
    /// A city the scout saw and is carrying home.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scout_report: Option<CityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raid: Option<Raid>,
    /// No new raid before this turn (the cooldown after the last one).
    #[serde(default)]
    pub next_raid_turn: u32,
    pub next_spawn_turn: u32,
    /// Raised by this empire's Call the warbands rite (`Action::CallTheWild`): an ordinary camp in
    /// every way except that borders never disperse it, it is outside the natural camp target,
    /// and its raiser gets no Gold for clearing it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rite: Option<PlayerId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Raid {
    pub city: CityId,
    /// Raiders still to muster.
    pub to_spawn: u8,
    pub next_spawn_turn: u32,
    /// The raiders go home at this turn and the camp can raid again.
    pub until_turn: u32,
}

impl BarbarianState {
    /// Update one team's camp memory from what it sees now (called by `update_explored`).
    pub fn remember(&mut self, team: TeamId, seen: &BitSet, map: &Map) {
        let visible = |h: Hex| map.index(h).is_some_and(|i| seen.get(i));
        let live: BTreeSet<Hex> = self.camps.iter().map(|c| c.pos).collect();
        let known = self.known.entry(team).or_default();
        known.retain(|h| !visible(*h) || live.contains(h));
        known.extend(live.iter().copied().filter(|h| visible(*h)));
        if known.is_empty() {
            self.known.remove(&team);
        }
    }

    pub fn camp_at(&self, h: Hex) -> Option<&Camp> {
        self.camps.iter().find(|c| c.pos == h)
    }

    /// Camps the natural spawner counts toward the map's target: rite camps are outside it.
    pub fn natural_camps(&self) -> usize {
        self.camps.iter().filter(|c| c.rite.is_none()).count()
    }
}

/// Of `gained` XP from a fight against a barbarian, what a unit with `xp_before` may keep when
/// its first promotion needs `first_promotion_xp` (Civ 6 `EXPERIENCE_MAX_BARB_LEVEL`:
/// barbarians can teach a unit its first promotion only). Used by `promotion::combat_xp`.
pub fn cap_barbarian_xp(xp_before: u32, gained: u32, first_promotion_xp: u32) -> u32 {
    gained.min(first_promotion_xp.saturating_sub(xp_before))
}

/// Barbarians never heal (Civ 6: only by pillaging, which this game doesn't have).
pub fn resting_heal() -> Breakdown {
    Breakdown {
        total: 0,
        parts: vec![("Barbarians never heal".into(), 0)],
    }
}

/// The melee and ranged unit types barbarians field now: a technology counts once half the
/// living empires know it.
pub fn unit_kinds(state: &GameState) -> (UnitKind, UnitKind) {
    let alive: Vec<_> = state.players.iter().filter(|p| p.alive).collect();
    let known = |t: TechId| {
        !alive.is_empty()
            && alive.iter().filter(|p| p.techs.contains(&t)).count() * 2 >= alive.len()
    };
    let melee = if known(TechId::Gunpowder) {
        UnitKind::Musketman
    } else if known(TechId::Feudalism) {
        UnitKind::Pikeman
    } else {
        UnitKind::Spearman
    };
    let ranged = if known(TechId::Machinery) {
        UnitKind::Crossbowman
    } else {
        UnitKind::Archer
    };
    (melee, ranged)
}

/// Camps the map supports: one per `LAND_TILES_PER_CAMP` passable land tiles, at most one per
/// two empires, at least one.
pub fn camp_target(state: &GameState) -> usize {
    let land = state
        .map
        .tiles
        .iter()
        .filter(|t| t.terrain.move_cost().is_some())
        .count();
    (land / LAND_TILES_PER_CAMP)
        .min(state.players.len().div_ceil(2))
        .max(1)
}

/// Compass direction from `from` toward `to` ("north-east", ...), for raid warnings that
/// must not reveal the camp's tile.
pub fn direction(from: Hex, to: Hex) -> &'static str {
    let (dq, dr) = (i64::from(to.q - from.q), i64::from(to.r - from.r));
    // Pointy-top hexes: x = sqrt(3) (q + r / 2), y = 1.5 r (south is +r). Scaled by 1000.
    let x = (2 * dq + dr) * 866;
    let y = dr * 1500;
    let (ax, ay) = (x.abs(), y.abs());
    // tan(22.5 deg) is about 2/5.
    if ax * 5 < ay * 2 {
        if y < 0 { "north" } else { "south" }
    } else if ay * 5 < ax * 2 {
        if x < 0 { "west" } else { "east" }
    } else {
        match (y < 0, x < 0) {
            (true, false) => "north-east",
            (true, true) => "north-west",
            (false, false) => "south-east",
            (false, true) => "south-west",
        }
    }
}

fn passable(state: &GameState, h: Hex) -> bool {
    state.map.move_cost(h).is_some()
}

/// A barbarian unit of this class may appear on `h`: passable, no city, and only barbarian
/// units of the other class there.
fn can_place(state: &GameState, h: Hex, military: bool) -> bool {
    passable(state, h)
        && state.city_at(h).is_none()
        && state
            .units_at(h)
            .all(|u| u.owner.is_barbarian() && is_military(u.kind) != military)
}

/// The nearest free tile around a camp (never the camp tile, which is the guard's).
fn place_near(state: &GameState, camp: Hex, military: bool) -> Option<Hex> {
    (1..=2).find_map(|radius| {
        let mut ring = camp.ring(radius);
        ring.sort();
        ring.into_iter().find(|h| can_place(state, *h, military))
    })
}

fn spawn(state: &mut GameState, kind: UnitKind, at: Hex) -> UnitId {
    crate::game::spawn_unit(state, PlayerId::BARBARIAN, kind, at)
}

fn alive_barbarian(state: &GameState, id: UnitId) -> bool {
    state.units.get(&id).is_some_and(|u| u.owner.is_barbarian())
}

/// Forget dead (or recaptured) units.
fn prune(state: &GameState, barb: &mut BarbarianState) {
    for camp in &mut barb.camps {
        camp.guard = camp.guard.filter(|id| alive_barbarian(state, *id));
        camp.scout = camp.scout.filter(|id| alive_barbarian(state, *id));
        camp.raiders.retain(|id| alive_barbarian(state, *id));
    }
}

fn rng(state: &GameState, parts: &[u64]) -> Rng {
    let mut all = vec![state.turn as u64, purpose::BARBARIANS];
    all.extend_from_slice(parts);
    Rng::derive(state.dice_seed(), &all)
}

/// A random passable tile within `radius` of `center` (never `center` itself or a city, so
/// wandering never turns into an attack on one). `city_gap` keeps it that far from cities.
fn random_tile(
    state: &GameState,
    center: Hex,
    radius: i32,
    city_gap: i32,
    rng: &mut Rng,
) -> Option<Hex> {
    let tiles: Vec<Hex> = center
        .within(radius)
        .into_iter()
        .filter(|h| {
            *h != center
                && passable(state, *h)
                && state.city_at(*h).is_none()
                && state.units_at(*h).next().is_none()
        })
        .filter(|h| {
            state
                .cities
                .values()
                .all(|c| c.pos.distance(*h) >= city_gap)
        })
        .collect();
    (!tiles.is_empty()).then(|| tiles[rng.range(0, tiles.len() as i64 - 1) as usize])
}

/// The free passable tile next to `target` nearest to `from` (ties by tile order).
/// In particular, a camp guard does not leave its tile for a raider's raid or scout's return.
fn beside(state: &GameState, target: Hex, from: Hex, distance: i32) -> Option<Hex> {
    let mut ring = target.ring(distance);
    ring.sort();
    ring.into_iter()
        .filter(|h| {
            passable(state, *h)
                && state.city_at(*h).is_none()
                && state.units_at(*h).next().is_none()
        })
        .min_by_key(|h| h.distance(from))
}

/// Barbarians never attack the Celts, all game (designer, 2026-11-02): no unit of theirs attacks,
/// shoots or captures a Celtic unit or city, and no scout reports a Celtic city for a raid. It is
/// the Celtic empire only: a teammate or ally of the Celts gets no protection. The Celts may
/// still attack and clear camps.
pub fn spares(state: &GameState, owner: PlayerId) -> bool {
    !owner.is_barbarian()
        && state.player(owner).faction == Faction::Celts
        && !state
            .setup
            .rule_experiments
            .rule_off(crate::experiments::FactionRule::CeltsBarbarianPeace)
}

/// Phase 1 (with the players' orders): barbarian units choose their moves from what the
/// barbarians can see at the start of the turn. Scouts that got home launch raids. Returns
/// ranged attacks for the ranged phase.
pub fn plan_turn(state: &mut GameState, events: &mut Vec<Event>) -> Vec<(PlayerId, UnitId, Hex)> {
    let Some(mut barb) = state.barbarians.take() else {
        return Vec::new();
    };
    prune(state, &mut barb);
    adopt_orphans(state, &mut barb);
    let sight = visibility::native_visible_tiles(state, TeamId::BARBARIAN);
    let mut shots = Vec::new();
    let mut claimed = BTreeSet::new();
    let mut gotos: BTreeMap<UnitId, Option<Hex>> = BTreeMap::new();
    for camp in &mut barb.camps {
        if let Some(g) = camp.guard {
            gotos.insert(g, (state.units[&g].pos != camp.pos).then_some(camp.pos));
        }
        if let Some(s) = camp.scout {
            let goto = scout_goto(state, camp, s, events);
            gotos.insert(s, goto);
        }
        let raid = camp.raid.as_ref().map(|r| r.city);
        for r in camp.raiders.clone() {
            let (goto, shot) = raider_plan(state, &sight, r, Some(camp.pos), raid, &mut claimed);
            gotos.insert(r, goto);
            shots.extend(shot.map(|t| (PlayerId::BARBARIAN, r, t)));
        }
    }
    let homed: BTreeSet<UnitId> = barb
        .camps
        .iter()
        .flat_map(|c| c.guard.into_iter().chain(c.scout).chain(c.raiders.clone()))
        .collect();
    let loose: Vec<UnitId> = state
        .units
        .values()
        .filter(|u| u.owner.is_barbarian() && !homed.contains(&u.id))
        .map(|u| u.id)
        .collect();
    for id in loose {
        let u = &state.units[&id];
        if is_military(u.kind) {
            let (goto, shot) = raider_plan(state, &sight, id, None, None, &mut claimed);
            gotos.insert(id, goto);
            shots.extend(shot.map(|t| (PlayerId::BARBARIAN, id, t)));
        } else {
            // A captive walks to the nearest camp and waits there to be freed.
            let camp = barb
                .camps
                .iter()
                .min_by_key(|c| (c.pos.distance(u.pos), c.id))
                .map(|c| c.pos);
            gotos.insert(id, camp.filter(|h| *h != u.pos));
        }
    }
    for (id, goto) in gotos {
        let u = state.units.get_mut(&id).expect("barbarian unit");
        u.goto = goto;
        u.path.clear();
        u.path_stop_on_block = false;
    }
    state.barbarians = Some(barb);
    shots
}

/// Military units whose camp is gone (or captured freed ones) join the nearest camp with room,
/// in unit id order, like Civ 6 raiders drifting back to a camp.
fn adopt_orphans(state: &GameState, barb: &mut BarbarianState) {
    let homed: BTreeSet<UnitId> = barb
        .camps
        .iter()
        .flat_map(|c| c.guard.into_iter().chain(c.scout).chain(c.raiders.clone()))
        .collect();
    let orphans: Vec<(UnitId, Hex)> = state
        .units
        .values()
        .filter(|u| u.owner.is_barbarian() && is_military(u.kind) && !homed.contains(&u.id))
        .map(|u| (u.id, u.pos))
        .collect();
    for (id, pos) in orphans {
        if let Some(camp) = barb
            .camps
            .iter_mut()
            .filter(|c| c.raiders.len() < MAX_RAIDERS)
            .min_by_key(|c| (c.pos.distance(pos), c.id))
        {
            camp.raiders.push(id);
        }
    }
}

fn scout_goto(
    state: &GameState,
    camp: &mut Camp,
    scout: UnitId,
    events: &mut Vec<Event>,
) -> Option<Hex> {
    let u = &state.units[&scout];
    if let Some(city) = camp.scout_report {
        match state.cities.get(&city) {
            None => camp.scout_report = None,
            Some(c) if spares(state, c.owner) => camp.scout_report = None,
            Some(c) if u.pos.distance(camp.pos) <= 1 => {
                camp.scout_report = None;
                if camp.raid.is_none() && state.turn >= camp.next_raid_turn {
                    camp.raid = Some(Raid {
                        city,
                        to_spawn: RAID_SIZE,
                        next_spawn_turn: state.turn,
                        until_turn: state.turn + RAID_TURNS,
                    });
                    events.push(Event::BarbarianRaidLaunched {
                        camp: camp.id,
                        city,
                        owner: c.owner,
                        direction: direction(c.pos, camp.pos).into(),
                    });
                }
            }
            Some(_) => return beside(state, camp.pos, u.pos, 1),
        }
    }
    if camp.raid.is_none() && state.turn >= camp.next_raid_turn {
        let sight = data::UNIT_SIGHT;
        if let Some(c) = state
            .cities
            .values()
            .filter(|c| c.pos.distance(u.pos) <= sight && !spares(state, c.owner))
            .min_by_key(|c| (c.pos.distance(camp.pos), c.id))
        {
            camp.scout_report = Some(c.id);
            events.push(Event::BarbarianScoutSpotted {
                unit: scout,
                city: c.id,
                owner: c.owner,
                at: u.pos,
            });
            return beside(state, camp.pos, u.pos, 1);
        }
    }
    match u.goto {
        Some(g) if g != u.pos && g.distance(camp.pos) <= SCOUT_RANGE => Some(g),
        _ => random_tile(
            state,
            camp.pos,
            SCOUT_RANGE,
            1,
            &mut rng(state, &[1, scout.0 as u64]),
        ),
    }
}

/// Expected damage both ways if barbarian `u` attacks unit `d` in melee (no rolls).
fn melee_odds(state: &GameState, u: &crate::state::Unit, d: &crate::state::Unit) -> (i32, i32) {
    // Barbarians never promote (level 0); the defender's promotions count.
    let atk = combat::unit_attack_total_rules(
        state.setup.rule_experiments.faction_rules_off,
        Faction::India,
        Faction::India,
        u.kind,
        u.hp,
        0,
    );
    let terrain = state.map.get(d.pos).map_or(Terrain::Plains, |t| t.terrain);
    let def = combat::unit_defense_total_rules(
        state.setup.rule_experiments.faction_rules_off,
        state.player(d.owner).faction,
        state.made_by(d),
        d.kind,
        d.hp,
        terrain,
        d.fortify,
        super::promotion::level_for_rules(
            state.setup.rule_experiments.faction_rules_off,
            state.player(d.owner).faction,
            d.xp,
        ),
    );
    (combat::base_damage(atk, def), combat::base_damage(def, atk))
}

/// One raider's move (a goto) or ranged shot. `home` is its camp (None once the camp is
/// gone), `raid` the city its camp is raiding. `claimed` keeps two raiders off one captive.
fn raider_plan(
    state: &GameState,
    sight: &BitSet,
    id: UnitId,
    home: Option<Hex>,
    raid: Option<CityId>,
    claimed: &mut BTreeSet<Hex>,
) -> (Option<Hex>, Option<Hex>) {
    let u = &state.units[&id];
    let stats = data::unit_stats(u.kind);
    let seen = |h: Hex| state.map.index(h).is_some_and(|i| sight.get(i));
    let foe =
        |v: &&crate::state::Unit| !v.owner.is_barbarian() && !spares(state, v.owner) && seen(v.pos);
    // Ranged units shoot the weakest visible military unit in the open with a clear shot.
    if stats.ranged > 0 {
        let unit = state
            .units
            .values()
            .filter(foe)
            .filter(|v| is_military(v.kind) && (1..=stats.range).contains(&v.pos.distance(u.pos)))
            .filter(|v| los::visible(u.pos, v.pos, |h| state.map.get(h).map(|t| t.terrain)))
            .filter(|v| state.city_at(v.pos).is_none())
            .min_by_key(|v| (v.hp, v.id))
            .map(|v| v.pos);
        if let Some(t) = unit {
            return (None, Some(t));
        }
    } else {
        // Capture a lone civilian within reach.
        let captive = state
            .units
            .values()
            .filter(foe)
            .filter(|v| !is_military(v.kind) && v.pos.distance(u.pos) <= stats.moves)
            .filter(|v| !claimed.contains(&v.pos) && state.city_at(v.pos).is_none())
            .filter(|v| {
                state
                    .units_at(v.pos)
                    .filter(foe)
                    .all(|w| !is_military(w.kind))
            })
            .min_by_key(|v| (v.pos.distance(u.pos), v.id))
            .map(|v| v.pos);
        if let Some(t) = captive {
            claimed.insert(t);
            return (Some(t), None);
        }
        // Attack a unit in the open when the odds favor it.
        let prey = state
            .units
            .values()
            .filter(foe)
            .filter(|v| is_military(v.kind) && v.pos.distance(u.pos) <= stats.moves)
            .filter(|v| state.city_at(v.pos).is_none())
            .filter_map(|v| {
                let (dealt, taken) = melee_odds(state, u, v);
                (dealt >= v.hp || (dealt >= taken + RAID_EDGE && taken < u.hp)).then_some((
                    v.pos.distance(u.pos),
                    -dealt,
                    v.id,
                    v.pos,
                ))
            })
            .min()
            .map(|t| t.3);
        if let Some(t) = prey {
            return (Some(t), None);
        }
    }
    // Raid: march on the city and prowl RAID_PROWL tiles from it, picking off units in the
    // open (above). Like Civ 6 below Emperor, raiders never assault the city itself.
    if let Some(c) = raid
        .and_then(|c| state.cities.get(&c))
        .filter(|c| !spares(state, c.owner))
    {
        let d = c.pos.distance(u.pos);
        return (
            (d != RAID_PROWL)
                .then(|| beside(state, c.pos, u.pos, RAID_PROWL))
                .flatten(),
            None,
        );
    }
    // Roam near home (or wander if homeless), away from cities.
    let center = home.unwrap_or(u.pos);
    let off_cities = |h: Hex| {
        state
            .cities
            .values()
            .all(|c| c.pos.distance(h) >= ROAM_CITY_DISTANCE)
    };
    if let Some(g) = u.goto.filter(|g| {
        *g != u.pos
            && g.distance(center) <= ROAM_RADIUS
            && off_cities(*g)
            && state.units_at(*g).next().is_none()
    }) {
        return (Some(g), None);
    }
    let mut r = rng(state, &[2, id.0 as u64]);
    if home.is_some_and(|h| h.distance(u.pos) > ROAM_RADIUS) || !off_cities(u.pos) || r.chance(34) {
        let radius = if home.is_some() {
            ROAM_RADIUS
        } else {
            SCOUT_RANGE
        };
        return (
            random_tile(state, center, radius, ROAM_CITY_DISTANCE, &mut r),
            None,
        );
    }
    (None, None)
}

/// A rite camp never stands on its city's own tile; it takes the farthest free tile of the
/// territory, which is next to a young city (its territory is the one ring) and farther out as
/// the city grows.
pub const RITE_MIN_CITY_DISTANCE: i32 = 1;
/// Rite camps keep this far from every other camp.
pub const RITE_MIN_CAMP_DISTANCE: i32 = 2;

/// Where Call the warbands would raise a camp for `city`, or None when its territory has no valid
/// tile: the farthest tile from the city (ties: the lowest hex, never the city's own) of its own territory,
/// passable, free of cities, ruins, units and camps, with a passable neighbour for the scout
/// and `RITE_MIN_CAMP_DISTANCE` from every camp (`taken` are camps raised earlier in the same
/// call).
fn rite_site(state: &GameState, p: PlayerId, city: &City, taken: &[Hex]) -> Option<Hex> {
    let camps: Vec<Hex> = state
        .barbarians
        .iter()
        .flat_map(|b| b.camps.iter().map(|c| c.pos))
        .chain(taken.iter().copied())
        .collect();
    city.pos
        .within(data::CITY_WORK_RADIUS)
        .into_iter()
        .filter(|h| {
            let Some(t) = state.map.get(*h) else {
                return false;
            };
            t.owner == Some(p)
                && t.city == Some(city.id)
                && !t.ruin
                && passable(state, *h)
                && city.pos.distance(*h) >= RITE_MIN_CITY_DISTANCE
                && state.city_at(*h).is_none()
                && state.units_at(*h).next().is_none()
                && camps
                    .iter()
                    .all(|c| c.distance(*h) >= RITE_MIN_CAMP_DISTANCE)
                && h.neighbors().iter().any(|n| passable(state, *n))
        })
        .min_by_key(|h| (std::cmp::Reverse(city.pos.distance(*h)), *h))
}

/// What Call the warbands raises now: one camp per city without a camp and with a valid tile,
/// in city-id order.
/// Validation, the view's preview and resolution all read this.
pub fn rite_plan(state: &GameState, p: PlayerId) -> Vec<(CityId, Hex)> {
    let mut taken = Vec::new();
    let mut plan = Vec::new();
    for city in state.cities_of(p) {
        if state.barbarians.iter().flat_map(|b| &b.camps).any(|camp| {
            state
                .map
                .get(camp.pos)
                .is_some_and(|tile| tile.city == Some(city.id))
        }) {
            continue;
        }
        if let Some(h) = rite_site(state, p, city, &taken) {
            taken.push(h);
            plan.push((city.id, h));
        }
    }
    plan
}

/// Call the warbands: raise the planned camps, each with its guard and scout, as ordinary barbarian
/// camps. A game with barbarians off gets its camp state here (natural camps stay off).
/// Returns what was raised.
pub fn call_the_wild(
    state: &mut GameState,
    p: PlayerId,
    events: &mut Vec<Event>,
) -> Vec<(CityId, Hex)> {
    let plan = rite_plan(state, p);
    for (_, pos) in &plan {
        raise_rite_camp(state, p, *pos, events);
    }
    plan
}

/// Raise one camp of `p`'s rite at `pos`, with its guard and scout: the camp Call the warbands
/// and a Celtic sack both raise. A game with barbarians off gets its camp state here.
fn raise_rite_camp(state: &mut GameState, p: PlayerId, pos: Hex, events: &mut Vec<Event>) {
    let mut barb = state.barbarians.take().unwrap_or_default();
    let (melee, _) = unit_kinds(state);
    let id = barb.next_camp_id;
    barb.next_camp_id += 1;
    let guard = spawn(state, melee, pos);
    let scout = place_near(state, pos, true).map(|h| spawn(state, UnitKind::Scout, h));
    barb.camps.push(Camp {
        id,
        pos,
        founded_turn: state.turn,
        guard: Some(guard),
        scout,
        raiders: Vec::new(),
        scout_report: None,
        raid: None,
        next_raid_turn: 0,
        next_spawn_turn: state.turn + CAMP_SPAWN_INTERVAL,
        rite: Some(p),
    });
    state.barbarians = Some(barb);
    events.push(Event::BarbarianCampSpawned { camp: id, at: pos });
}

/// How far from a sacked city's site its warband camp may stand.
pub const SACK_CAMP_RADIUS: i32 = 3;

/// Where a sack of the city that stood at `site` raises `p`'s warband camp, or None when no tile
/// qualifies: a tile within `SACK_CAMP_RADIUS` of the site that is passable, free of cities,
/// ruins, units and camps, with a passable neighbour for the scout, `RITE_MIN_CAMP_DISTANCE` from
/// every camp, and owned by nobody or by `p`. A tile in the territory of a city of `p` that has no
/// camp yet comes first (it completes Raise the Warbands for that city), then the nearest to the
/// site; ties go to the lowest hex.
pub fn sack_camp_site(state: &GameState, p: PlayerId, site: Hex) -> Option<Hex> {
    let camps: Vec<Hex> = state
        .barbarians
        .iter()
        .flat_map(|b| b.camps.iter().map(|c| c.pos))
        .collect();
    let campless: BTreeSet<CityId> = state
        .cities_of(p)
        .filter(|c| {
            !camps
                .iter()
                .any(|h| state.map.get(*h).is_some_and(|t| t.city == Some(c.id)))
        })
        .map(|c| c.id)
        .collect();
    site.within(SACK_CAMP_RADIUS)
        .into_iter()
        .filter(|h| {
            let Some(t) = state.map.get(*h) else {
                return false;
            };
            t.owner.is_none_or(|o| o == p)
                && !t.ruin
                && passable(state, *h)
                && state.city_at(*h).is_none()
                && state.units_at(*h).next().is_none()
                && camps
                    .iter()
                    .all(|c| c.distance(*h) >= RITE_MIN_CAMP_DISTANCE)
                && h.neighbors().iter().any(|n| passable(state, *n))
        })
        .min_by_key(|h| {
            let completes = state
                .map
                .get(*h)
                .and_then(|t| t.city)
                .is_some_and(|c| campless.contains(&c));
            (!completes, site.distance(*h), *h)
        })
}

/// A sack raises its warband camp: one rite camp at `sack_camp_site`. Returns where, if one stood.
pub fn raise_sack_camp(
    state: &mut GameState,
    p: PlayerId,
    site: Hex,
    events: &mut Vec<Event>,
) -> Option<Hex> {
    let pos = sack_camp_site(state, p, site)?;
    raise_rite_camp(state, p, pos, events);
    Some(pos)
}

/// After movement: an empire's military unit standing on a camp clears it, and its owner
/// collects the Gold.
pub fn clear_camps(state: &mut GameState, events: &mut Vec<Event>) {
    let Some(mut barb) = state.barbarians.take() else {
        return;
    };
    let gold = CAMP_CLEAR_GOLD;
    let mut kept = Vec::new();
    for camp in std::mem::take(&mut barb.camps) {
        let clearer = state
            .units_at(camp.pos)
            .filter(|u| is_military(u.kind) && !u.owner.is_barbarian())
            .min_by_key(|u| u.id)
            .map(|u| (u.id, u.owner));
        match clearer {
            Some((unit, by)) => {
                // The Celts (and their team) get nothing for a camp their own rite raised.
                let gold = match camp.rite {
                    Some(raiser) if state.team_of(raiser) == state.team_of(by) => 0,
                    _ => gold,
                };
                let p = state.player_mut(by);
                p.gold = crate::validation::credit_gold(p.gold, gold);
                events.push(Event::BarbarianCampCleared {
                    camp: camp.id,
                    at: camp.pos,
                    by,
                    unit: Some(unit),
                    gold,
                });
            }
            None => kept.push(camp),
        }
    }
    barb.camps = kept;
    state.barbarians = Some(barb);
}

/// End of turn, after the last sight update: report newly seen camps, disperse camps under
/// new borders, place new camps and muster units. New camps and units appear only where no
/// empire looks, or at their camp.
pub fn end_of_turn(before: &GameState, state: &mut GameState, events: &mut Vec<Event>) {
    let Some(mut barb) = state.barbarians.take() else {
        return;
    };
    let old = before.barbarians.as_ref().map(|b| &b.known);
    for (team, known) in &barb.known {
        for h in known {
            if old.is_some_and(|o| o.get(team).is_some_and(|k| k.contains(h))) {
                continue;
            }
            if let Some(c) = barb.camp_at(*h) {
                events.push(Event::BarbarianCampDiscovered {
                    camp: c.id,
                    team: *team,
                    at: *h,
                });
            }
        }
    }
    prune(state, &mut barb);
    let mut kept = Vec::new();
    for camp in std::mem::take(&mut barb.camps) {
        // A rite camp stands inside its raiser's borders by design.
        let owner = state
            .map
            .get(camp.pos)
            .and_then(|t| t.owner)
            .filter(|_| camp.rite.is_none());
        match owner {
            Some(by) => events.push(Event::BarbarianCampCleared {
                camp: camp.id,
                at: camp.pos,
                by,
                unit: None,
                gold: 0,
            }),
            None => kept.push(camp),
        }
    }
    barb.camps = kept;
    spawn_camps(state, &mut barb, events);
    muster(state, &mut barb);
    state.barbarians = Some(barb);
}

fn spawn_camps(state: &mut GameState, barb: &mut BarbarianState, events: &mut Vec<Event>) {
    // A game with barbarians off still holds a camp state once a Celts rite raised a wild camp.
    if !state.setup.barbarians || state.turn < FIRST_CAMP_TURN {
        return;
    }
    let target = camp_target(state);
    let want = if barb.last_camp_turn == 0 {
        target.div_ceil(3)
    } else if state.turn >= barb.last_camp_turn + CAMP_INTERVAL {
        1
    } else {
        0
    };
    let want = want.min(target.saturating_sub(barb.natural_camps()));
    if want == 0 {
        return;
    }
    let mut seen = BitSet::new(state.map.len());
    for team in state.teams() {
        for i in visibility::native_visible_tiles(state, team).iter_ones() {
            seen.set(i);
        }
    }
    for _ in 0..want {
        let Some(pos) = pick_site(state, barb, &seen) else {
            break;
        };
        let id = barb.next_camp_id;
        barb.next_camp_id += 1;
        barb.last_camp_turn = state.turn;
        let (melee, _) = unit_kinds(state);
        let guard = spawn(state, melee, pos);
        let scout = place_near(state, pos, true).map(|h| spawn(state, UnitKind::Scout, h));
        barb.camps.push(Camp {
            id,
            pos,
            founded_turn: state.turn,
            guard: Some(guard),
            scout,
            raiders: Vec::new(),
            scout_report: None,
            raid: None,
            next_raid_turn: 0,
            next_spawn_turn: state.turn + CAMP_SPAWN_INTERVAL,
            rite: None,
        });
        events.push(Event::BarbarianCampSpawned { camp: id, at: pos });
    }
}

fn pick_site(state: &GameState, barb: &BarbarianState, seen: &BitSet) -> Option<Hex> {
    let cities: Vec<Hex> = state.cities.values().map(|c| c.pos).collect();
    let units: Vec<Hex> = state
        .units
        .values()
        .filter(|u| !u.owner.is_barbarian())
        .map(|u| u.pos)
        .collect();
    let sites: Vec<Hex> = (0..state.map.len())
        .filter(|&i| !seen.get(i))
        .filter_map(|i| {
            let t = &state.map.tiles[i];
            // Never on a resource (G11), so camps can't sit on a start's guaranteed Iron.
            (t.terrain.move_cost().is_some()
                && t.owner.is_none()
                && !t.ruin
                && t.resource.is_none())
            .then(|| state.map.hex_at(i))
        })
        .filter(|h| state.units_at(*h).next().is_none())
        .filter(|h| {
            cities
                .iter()
                .all(|c| c.distance(*h) >= CAMP_MIN_CITY_DISTANCE)
        })
        .filter(|h| {
            units
                .iter()
                .all(|u| u.distance(*h) >= CAMP_MIN_UNIT_DISTANCE)
        })
        .filter(|h| {
            barb.camps
                .iter()
                .all(|c| c.pos.distance(*h) >= CAMP_MIN_CAMP_DISTANCE)
        })
        .filter(|h| h.neighbors().iter().any(|n| passable(state, *n)))
        .collect();
    if sites.is_empty() {
        return None;
    }
    let mut r = rng(state, &[3, barb.next_camp_id as u64]);
    Some(sites[r.range(0, sites.len() as i64 - 1) as usize])
}

/// Raid waves and each camp's slow timer.
fn muster(state: &mut GameState, barb: &mut BarbarianState) {
    let (melee, ranged) = unit_kinds(state);
    for camp in &mut barb.camps {
        if let Some(raid) = &mut camp.raid {
            if !state.cities.contains_key(&raid.city) || state.turn >= raid.until_turn {
                camp.raid = None;
                camp.next_raid_turn = state.turn + RAID_COOLDOWN;
            } else if raid.to_spawn > 0 {
                if state.turn >= raid.next_spawn_turn && camp.raiders.len() < MAX_RAIDERS {
                    let kind = if raid.to_spawn == 2 { ranged } else { melee };
                    if let Some(h) = place_near(state, camp.pos, true) {
                        camp.raiders.push(spawn(state, kind, h));
                        raid.to_spawn -= 1;
                        raid.next_spawn_turn = state.turn + RAID_SPAWN_INTERVAL;
                    }
                }
            } else if camp.raiders.is_empty() {
                camp.raid = None;
                camp.next_raid_turn = state.turn + RAID_COOLDOWN;
            }
        }
        if state.turn < camp.next_spawn_turn {
            continue;
        }
        camp.next_spawn_turn = state.turn + CAMP_SPAWN_INTERVAL;
        if camp.guard.is_none() {
            if can_place(state, camp.pos, true) {
                camp.guard = Some(spawn(state, melee, camp.pos));
            }
        } else if camp.scout.is_none() {
            camp.scout =
                place_near(state, camp.pos, true).map(|h| spawn(state, UnitKind::Scout, h));
        } else if camp.raiders.is_empty()
            && let Some(h) = place_near(state, camp.pos, true)
        {
            camp.raiders.push(spawn(state, melee, h));
        }
    }
}

#[cfg(test)]
mod tests;
