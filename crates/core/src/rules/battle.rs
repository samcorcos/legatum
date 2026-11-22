//! Combat resolution: planning fights from a snapshot, applying all damage together, the ranged
//! phase, and city capture.
//!
//! Every fight in a stage (the ranged phase, or one movement round) is planned against the same
//! unmodified state, then [`apply_fights`] sums the damage per unit and city and applies it at
//! once, so kill order never matters and mutual kills are possible.
//!
//! Rolls: `Rng::derive(seed, [turn, COMBAT, stage, key(attacker), key(target)])`, attacker roll
//! drawn first, then defender roll, each in 80..=120. Stage 0 is the ranged phase; movement
//! round `r` is stage `1 + r`. Unit keys are the unit id; city keys are `1 << 32 | city id`.

use super::{hostile, is_military, is_threat, promotion};
use crate::bitset::BitSet;
use crate::combat::{self, ROLL_MAX, ROLL_MIN};
use crate::data::{self, CAPTURE_RESISTANCE_TURNS};
use crate::events::{Attacker, Defender, Event};
use crate::experiments::FactionRule;
use crate::hex::Hex;
use crate::los;
use crate::rng::{Rng, purpose};
use crate::state::GameState;
use crate::types::*;
use crate::visibility;
use std::collections::{BTreeMap, BTreeSet};

pub const STAGE_RANGED: u64 = 0;

/// What a fight is aimed at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Unit(UnitId),
    City(CityId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightKind {
    /// A unit stepping into a hostile unit or city. Defender gets terrain and fortify bonuses.
    Melee,
    /// Both sides moving into each other (same empty tile, or a swap). No bonuses for anyone.
    Meeting,
    /// No retaliation.
    Ranged,
}

/// One planned fight, with every number that decides it.
#[derive(Clone, Debug)]
pub struct Fight {
    pub attacker: UnitId,
    pub target: Target,
    pub kind: FightKind,
    pub at: Hex,
    pub attacker_strength: Breakdown,
    pub defender_strength: Breakdown,
    pub attacker_roll: i32,
    pub defender_roll: i32,
    pub damage_to_attacker: i32,
    pub damage_to_defender: i32,
    /// A melee hit on a unit moving away this round (G21): no fortification for it.
    pub defender_leaving: bool,
}

fn key(t: Target) -> u64 {
    match t {
        Target::Unit(u) => u.0 as u64,
        Target::City(c) => (1u64 << 32) | c.0 as u64,
    }
}

#[cfg(test)]
thread_local! {
    /// Tests only: every combat roll is the average (100), for turn-by-turn worked examples.
    pub(crate) static AVERAGE_ROLLS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn average_rolls() -> bool {
    AVERAGE_ROLLS.with(|c| c.get())
}

#[cfg(not(test))]
fn average_rolls() -> bool {
    false
}

/// The two rolls (attacker, defender) for a fight, derived from its identity only.
pub fn rolls(state: &GameState, stage: u64, attacker: UnitId, target: Target) -> (i32, i32) {
    if average_rolls() {
        return (100, 100);
    }
    let mut rng = Rng::derive(
        state.dice_seed(),
        &[
            state.turn as u64,
            purpose::COMBAT,
            stage,
            attacker.0 as u64,
            key(target),
        ],
    );
    let a = rng.range(ROLL_MIN as i64, ROLL_MAX as i64) as i32;
    let d = rng.range(ROLL_MIN as i64, ROLL_MAX as i64) as i32;
    (a, d)
}

/// Whether another Spartan Hoplite of `u`'s empire stands on a tile next to it in this snapshot:
/// the phalanx (`data::HOPLITE_PHALANX_STRENGTH`). False for any unit that is not a Hoplite.
pub fn has_phalanx_neighbour(state: &GameState, u: &crate::state::Unit) -> bool {
    let off = state.setup.rule_experiments.faction_rules_off;
    data::is_hoplite_rules(off, state.made_by(u), u.kind)
        && u.pos.neighbors().into_iter().any(|h| {
            state.units_at(h).any(|o| {
                o.id != u.id
                    && o.owner == u.owner
                    && data::is_hoplite_rules(off, state.made_by(o), o.kind)
            })
        })
}

/// The siege tier of `u`, the Walls its attacks on a city ignore (`data::siege_cover`): every
/// tier for an Industrial or later land unit, the highest siege technology its owner knows when
/// the unit has bought Siege, 0 otherwise.
pub fn siege_cover(state: &GameState, u: &crate::state::Unit) -> u8 {
    data::siege_cover(u.kind, u.siege.then(|| &state.player(u.owner).techs))
}

fn unit_defense_on(
    state: &GameState,
    u: &crate::state::Unit,
    terrain: Terrain,
    fortify: i32,
) -> Breakdown {
    let off = state.setup.rule_experiments.faction_rules_off;
    let faction = state.player(u.owner).faction;
    let made_by = state.made_by(u);
    let mut strength = combat::unit_defense_strength_for_rules(
        off,
        faction,
        made_by,
        u.kind,
        u.hp,
        terrain,
        fortify,
        promotion::level_for_rules(off, faction, u.xp),
    );
    combat::add_phalanx_rules(
        &mut strength,
        off,
        made_by,
        u.kind,
        has_phalanx_neighbour(state, u),
    );
    crate::resources::add_unsupplied(
        &mut strength,
        made_by,
        u.kind,
        false,
        crate::resources::unit_unsupplied(state, u),
    );
    strength
}

/// Plan a fight against the current (snapshot) state. Does not change anything.
pub fn plan_fight(
    state: &GameState,
    stage: u64,
    attacker: UnitId,
    target: Target,
    kind: FightKind,
) -> Fight {
    plan_fight_on(state, stage, attacker, target, kind, false)
}

/// `plan_fight`, where `defender_leaving` marks a melee hit on a unit that is moving away this
/// round (G21): caught leaving, it gets its terrain but no fortification.
pub fn plan_fight_on(
    state: &GameState,
    stage: u64,
    attacker: UnitId,
    target: Target,
    kind: FightKind,
    defender_leaving: bool,
) -> Fight {
    plan_fight_inner(state, stage, attacker, target, kind, defender_leaving, None)
}

/// `plan_fight_on`, where `defend_on` makes a unit target defend on that terrain with no
/// fortification: a relief unit not yet in the city it is about to hold.
fn plan_fight_inner(
    state: &GameState,
    stage: u64,
    attacker: UnitId,
    target: Target,
    kind: FightKind,
    defender_leaving: bool,
    defend_on: Option<Terrain>,
) -> Fight {
    let level = |u: &crate::state::Unit| {
        promotion::level_for_rules(
            state.setup.rule_experiments.faction_rules_off,
            state.player(u.owner).faction,
            u.xp,
        )
    };
    let a = &state.units[&attacker];
    let mut attacker_strength = combat::attack_strength_rules(
        state.setup.rule_experiments.faction_rules_off,
        state.player(a.owner).faction,
        state.made_by(a),
        a.kind,
        a.hp,
        level(a),
        kind == FightKind::Ranged,
        kind == FightKind::Meeting,
        has_phalanx_neighbour(state, a),
        crate::resources::unit_unsupplied(state, a),
        matches!(target, Target::City(_)),
    );
    let rules = state.setup.rule_experiments;
    let covered = siege_cover(state, a);
    if let Target::Unit(d) = target {
        combat::add_fight_class_bonus_rules(
            &mut attacker_strength,
            state.setup.rule_experiments.faction_rules_off,
            state.made_by(a),
            a.kind,
            state.units[&d].kind,
            true,
            kind == FightKind::Ranged,
        );
    }
    let (at, mut defender_strength) = match target {
        Target::Unit(d) => {
            let d = &state.units[&d];
            let strength = match kind {
                FightKind::Meeting => combat::unit_meeting_strength_for_rules(
                    state.setup.rule_experiments.faction_rules_off,
                    state.player(d.owner).faction,
                    state.made_by(d),
                    d.kind,
                    d.hp,
                    level(d),
                ),
                _ => {
                    let terrain = defend_on.unwrap_or_else(|| {
                        state.map.get(d.pos).map_or(Terrain::Plains, |t| t.terrain)
                    });
                    unit_defense_on(
                        state,
                        d,
                        terrain,
                        if defender_leaving || defend_on.is_some() {
                            0
                        } else {
                            d.fortify
                        },
                    )
                }
            };
            let mut strength = strength;
            if kind == FightKind::Meeting {
                combat::add_phalanx_rules(
                    &mut strength,
                    state.setup.rule_experiments.faction_rules_off,
                    state.made_by(d),
                    d.kind,
                    has_phalanx_neighbour(state, d),
                );
                crate::resources::add_unsupplied(
                    &mut strength,
                    state.made_by(d),
                    d.kind,
                    false,
                    crate::resources::unit_unsupplied(state, d),
                );
            }
            (d.pos, strength)
        }
        Target::City(c) => {
            let c = &state.cities[&c];
            (
                c.pos,
                combat::city_defense_vs(
                    rules,
                    state.player(c.owner).faction,
                    c.population,
                    data::wall_tier(&c.buildings),
                    covered,
                    c.hp,
                    state.era_of(c.owner),
                ),
            )
        }
    };
    if let Target::Unit(d) = target {
        combat::add_fight_class_bonus_rules(
            &mut defender_strength,
            state.setup.rule_experiments.faction_rules_off,
            state.made_by(&state.units[&d]),
            state.units[&d].kind,
            a.kind,
            false,
            kind == FightKind::Ranged,
        );
    }
    let (attacker_roll, mut defender_roll) = rolls(state, stage, attacker, target);
    let normal_damage = combat::damage(
        attacker_strength.total,
        defender_strength.total,
        attacker_roll,
    );
    let damage_to_defender = if let Target::City(c) = target {
        combat::damage_to_city_hp(
            combat::city_damage_percent(
                a.kind,
                data::wall_tier(&state.cities[&c].buildings),
                covered,
            ),
            normal_damage,
        )
    } else {
        normal_damage
    };
    let damage_to_attacker = if kind == FightKind::Ranged {
        defender_roll = 0;
        0
    } else {
        combat::damage(
            defender_strength.total,
            attacker_strength.total,
            defender_roll,
        )
    };
    Fight {
        attacker,
        target,
        kind,
        at,
        attacker_strength,
        defender_strength,
        attacker_roll,
        defender_roll,
        damage_to_attacker,
        damage_to_defender,
        defender_leaving: defender_leaving && kind == FightKind::Melee,
    }
}

/// Apply every fight's damage together, emit Battle events (HP after = HP after the whole
/// stage's damage) and destroy dead units. Marks every surviving participant as having fought
/// and grants it the stage's experience (`promotion`). Returns the units destroyed.
pub fn apply_fights(
    state: &mut GameState,
    fights: &[Fight],
    events: &mut Vec<Event>,
) -> BTreeSet<UnitId> {
    let carry = carried_damage(state, fights, &BTreeSet::new(), &BTreeMap::new());
    apply_stage(state, fights, &[], &carry, events)
}

/// [`apply_fights`] for a movement round: `carry` is what [`carried_damage`] planned for the
/// round (it already skips a garrison stepping off its tile).
pub fn apply_movement_fights(
    state: &mut GameState,
    fights: &[Fight],
    carry: &Carry,
    events: &mut Vec<Event>,
) -> BTreeSet<UnitId> {
    apply_stage(state, fights, &[], carry, events)
}

/// What the hits of a stage on a city tile do beyond the target they were aimed at.
#[derive(Clone, Debug, Default)]
pub struct Carry {
    /// Shared damage per unit (the garrison, or a supporter), on top of its direct hits.
    pub damage: BTreeMap<UnitId, i32>,
    /// A hit aimed at a garrison that shares it with supporters: the garrison's own share
    /// replaces the hit's damage (by index into the stage's fights).
    pub hit: BTreeMap<usize, i32>,
    /// The supporters the assault damaged, with the city tile they fought for: a supporter
    /// killed there is a kill by everyone who hit that tile.
    pub relief: BTreeMap<UnitId, Hex>,
    /// The supporters of each defending side this stage (keyed by the city, and by its garrison
    /// when it has one), with the city's terrain they defend on: each adds its own
    /// counterattack to the side's reply (`split_counterattacks`).
    pub support: BTreeMap<Target, (Terrain, Vec<UnitId>)>,
}

/// One defender sharing what gets past a city's HP: the garrison or a supporter.
struct Member {
    unit: UnitId,
    hp: i32,
    relief: bool,
}

/// Supported defence (designer, 2026-11-15; per-hit judging from 2026-11-11, "the damage each
/// unit gets should be judged on the damage inflicted to that specific unit type"). The stage's
/// hits on a city tile go one by one in `fights` order. The city's HP absorbs first (or, at a
/// fallen city, a garrison leaving it takes its parting hit first; one leaving a city with HP
/// is not reached). What gets past it is shared among the living defenders: the garrison and
/// every supporter `reserves` lists (the owner's units ordered into the city this round). A
/// hit of damage `B` with `P` of it past the city (all of it, for a hit on the garrison or on
/// a city already emptied this stage) gives each of the `n` living defenders `P / n`, the
/// remainder to the garrison (or, without one, the lowest unit id), and each takes that share
/// of what the same attacker, with the same roll, deals to it directly:
/// `floor(direct × share / B)` (`combat::carried_share`; its defence on the city's terrain,
/// class bonuses, no fortification for a supporter). Overkill on a defender is wasted; later
/// hits are shared among those still alive. With no supporters this is the city-to-garrison
/// overflow of 2026-10-04 unchanged. Planned from the stage snapshot.
pub fn carried_damage(
    state: &GameState,
    fights: &[Fight],
    leaving: &BTreeSet<UnitId>,
    reserves: &BTreeMap<CityId, Vec<UnitId>>,
) -> Carry {
    let mut out = Carry::default();
    let mut cities: BTreeSet<CityId> = reserves.keys().copied().collect();
    cities.extend(fights.iter().filter_map(|f| match f.target {
        Target::City(c) => Some(c),
        Target::Unit(_) => None,
    }));
    for id in cities {
        let city = &state.cities[&id];
        let terrain = state
            .map
            .get(city.pos)
            .map_or(Terrain::Plains, |t| t.terrain);
        let mut supporters: Vec<UnitId> = reserves.get(&id).cloned().unwrap_or_default();
        supporters.sort();
        let garrison = state
            .units_at(city.pos)
            .find(|u| is_military(u.kind) && u.owner == city.owner && !supporters.contains(&u.id))
            .map(|u| u.id);
        let garrison_leaving = garrison.is_some_and(|g| leaving.contains(&g));
        // The front layer: the city's HP, or a garrison leaving a fallen city.
        let front_is_garrison = city.hp <= 0 && garrison_leaving;
        let mut front_hp = if city.hp > 0 {
            city.hp
        } else if front_is_garrison {
            garrison.map_or(0, |g| state.units[&g].hp)
        } else {
            0
        };
        let mut pool: Vec<Member> = Vec::new();
        if let Some(g) = garrison.filter(|_| !garrison_leaving) {
            pool.push(Member {
                unit: g,
                hp: state.units[&g].hp,
                relief: false,
            });
        }
        for r in &supporters {
            pool.push(Member {
                unit: *r,
                hp: state.units[r].hp,
                relief: true,
            });
        }
        if !supporters.is_empty() {
            out.support
                .insert(Target::City(id), (terrain, supporters.clone()));
            if let Some(g) = garrison {
                out.support
                    .insert(Target::Unit(g), (terrain, supporters.clone()));
            }
        }
        for (index, f) in fights.iter().enumerate() {
            let at_front = match f.target {
                Target::City(c) if c == id => !front_is_garrison,
                Target::Unit(u) if garrison == Some(u) => front_is_garrison,
                _ => continue,
            };
            let base = f.damage_to_defender;
            // The part of the hit that gets past the front layer.
            let past = if at_front && front_hp > 0 {
                let left = front_hp - base;
                front_hp = left.max(0);
                -left
            } else {
                base
            };
            if past <= 0 || base <= 0 {
                continue;
            }
            let living: Vec<usize> = (0..pool.len()).filter(|&k| pool[k].hp > 0).collect();
            let n = living.len() as i32;
            for (k, &m) in living.iter().enumerate() {
                let part = past / n + if k == 0 { past % n } else { 0 };
                let aimed = f.target == Target::Unit(pool[m].unit);
                let direct = if aimed {
                    base
                } else {
                    direct_damage(state, f, pool[m].unit, pool[m].relief, leaving, terrain)
                };
                let share = combat::carried_share(direct, part, base);
                pool[m].hp -= share;
                if aimed {
                    out.hit.insert(index, share);
                } else {
                    *out.damage.entry(pool[m].unit).or_default() += share;
                    if pool[m].relief && share > 0 {
                        out.relief.insert(pool[m].unit, city.pos);
                    }
                }
            }
        }
    }
    out
}

/// What `f`'s attacker (same roll) deals to `unit` directly: its defence on the city's terrain
/// with its class bonuses, fortified unless it is a supporter or stepping off.
fn direct_damage(
    state: &GameState,
    f: &Fight,
    unit: UnitId,
    supporter: bool,
    leaving: &BTreeSet<UnitId>,
    terrain: Terrain,
) -> i32 {
    let planned = plan_fight_inner(
        state,
        STAGE_RANGED,
        f.attacker,
        Target::Unit(unit),
        if f.kind == FightKind::Ranged {
            FightKind::Ranged
        } else {
            FightKind::Melee
        },
        leaving.contains(&unit),
        supporter.then_some(terrain),
    );
    combat::damage(
        planned.attacker_strength.total,
        planned.defender_strength.total,
        f.attacker_roll,
    )
}

/// How much attack a defender stepping into its city can soak (designer, 2026-11-15:
/// "healthiest and strongest... a warrior in the late game might have the most HP, but they
/// can't absorb it"): its HP over the damage the worst of `attackers` deals it at an average
/// roll (on the city's terrain, unfortified, class bonuses), scaled by 1000; the lowest over the
/// attackers counts. With no attacker left, HP × defence strength. Ranks who takes over a city
/// as its garrison; higher soaks more.
pub fn soak(state: &GameState, unit: UnitId, attackers: &[UnitId], terrain: Terrain) -> i64 {
    let u = &state.units[&unit];
    let hp = i64::from(u.hp.max(0));
    attackers
        .iter()
        .filter(|a| state.units.contains_key(a))
        .map(|a| {
            let f = plan_fight_inner(
                state,
                STAGE_RANGED,
                *a,
                Target::Unit(unit),
                FightKind::Melee,
                true,
                Some(terrain),
            );
            let hit = combat::damage(f.attacker_strength.total, f.defender_strength.total, 100);
            hp * 1000 / i64::from(hit.max(1))
        })
        .min()
        .unwrap_or_else(|| hp * i64::from(unit_defense_on(state, u, terrain, 0).total))
}

/// A unit's force against a city (user, 2026-11-11, Lucerne): the damage its attack would deal
/// to the city's HP at an average (100%) roll, not limited by the HP the city has left. Shots
/// (`ranged`) and moves onto the tile alike, with the city's current defence, Walls and Siege as
/// in resolution. Decides which of two rival teams takes a city both assault in one step.
pub fn city_force(state: &GameState, unit: UnitId, city: CityId, ranged: bool) -> i32 {
    let kind = if ranged {
        FightKind::Ranged
    } else {
        FightKind::Melee
    };
    let f = plan_fight(state, STAGE_RANGED, unit, Target::City(city), kind);
    let u = &state.units[&unit];
    combat::damage_to_city_hp(
        combat::city_damage_percent(
            u.kind,
            data::wall_tier(&state.cities[&city].buildings),
            siege_cover(state, u),
        ),
        combat::damage(f.attacker_strength.total, f.defender_strength.total, 100),
    )
}

struct CityShot {
    city: CityId,
    unit: UnitId,
    attack: Breakdown,
    defense: Breakdown,
    roll: i32,
    damage: i32,
}

/// Whether `target`'s one counterattack this turn was already spent in an earlier stage: a
/// melee Battle against it is in this turn's event log. A city and its garrison are one
/// defending side, so a garrison whose city was assaulted earlier this turn is spent too.
/// Ranged fire and meeting engagements never spend it.
fn counterattack_spent(state: &GameState, target: Target, events: &[Event]) -> bool {
    events.iter().any(|event| {
        let Event::Battle {
            defender,
            at,
            ranged: false,
            meeting: false,
            ..
        } = event
        else {
            return false;
        };
        match (target, defender) {
            (Target::Unit(id), Defender::Unit { unit, .. }) => id == *unit,
            (Target::City(id), Defender::City { city, .. }) => id == *city,
            (Target::Unit(id), Defender::City { city, .. }) => {
                state.cities.get(city).is_some_and(|c| {
                    c.pos == *at
                        && state
                            .units
                            .get(&id)
                            .is_some_and(|u| u.pos == *at && u.owner == c.owner)
                })
            }
            _ => false,
        }
    })
}

/// The split counterattack (user decision 2026-10-04): a defender (a unit, or a city's melee
/// defense) fights back once per turn. Its counterattack is computed once, from the snapshot,
/// as its retaliation against the strongest melee attacker this stage (highest attack strength,
/// then lowest unit id), and split evenly with integer division among every melee attacker
/// that hits it this stage; the strongest takes the remainder. A defender already attacked in
/// melee in an earlier stage this turn retaliates against nobody. Attackers' own hits are
/// unchanged. Ranged and meeting fights are untouched.
///
/// Supported defence (designer, 2026-11-15): each supporter of the defending side (`carry`)
/// adds its own counterattack, computed the same way from the snapshot (its defence on the
/// city's terrain, unfortified, against the same strongest attacker, with the same defender
/// roll), once per turn; the replies are pooled and split the same way.
fn split_counterattacks(state: &GameState, fights: &mut [Fight], events: &[Event], carry: &Carry) {
    let mut groups: BTreeMap<Target, Vec<usize>> = BTreeMap::new();
    for (i, fight) in fights.iter().enumerate() {
        if fight.kind == FightKind::Melee {
            groups.entry(fight.target).or_default().push(i);
        }
    }
    for (target, indices) in groups {
        let strongest = indices
            .iter()
            .copied()
            .max_by_key(|&i| {
                (
                    fights[i].attacker_strength.total,
                    std::cmp::Reverse(fights[i].attacker),
                )
            })
            .expect("nonempty counterattack group");
        let mut total = if counterattack_spent(state, target, events) {
            0
        } else {
            fights[strongest].damage_to_attacker
        };
        if let Some((terrain, supporters)) = carry.support.get(&target) {
            for s in supporters {
                if !state.units.contains_key(s)
                    || counterattack_spent(state, Target::Unit(*s), events)
                {
                    continue;
                }
                let reply = plan_fight_inner(
                    state,
                    STAGE_RANGED,
                    fights[strongest].attacker,
                    Target::Unit(*s),
                    FightKind::Melee,
                    true,
                    Some(*terrain),
                );
                total += combat::damage(
                    reply.defender_strength.total,
                    reply.attacker_strength.total,
                    fights[strongest].defender_roll,
                );
            }
        }
        let n = indices.len() as i32;
        for i in indices {
            fights[i].damage_to_attacker = total / n + if i == strongest { total % n } else { 0 };
        }
    }
}

/// City and unit shots share a snapshot and damage application. A killed shooter still
/// fires; multiple cities may choose the same lowest-HP target. Cities earn no XP.
fn apply_stage(
    state: &mut GameState,
    fights: &[Fight],
    shots: &[CityShot],
    carry: &Carry,
    events: &mut Vec<Event>,
) -> BTreeSet<UnitId> {
    let stage_start = events.len();
    let mut fights = fights.to_vec();
    split_counterattacks(state, &mut fights, events, carry);
    for (index, share) in &carry.hit {
        fights[*index].damage_to_defender = *share;
    }
    let fights = fights.as_slice();
    let extra: Vec<_> = shots.iter().map(|s| (s.unit, s.damage)).collect();
    let xp = promotion::combat_xp(state, fights, &extra);
    let mut unit_damage: BTreeMap<UnitId, i32> = BTreeMap::new();
    let mut city_damage: BTreeMap<CityId, i32> = BTreeMap::new();
    for s in shots {
        *unit_damage.entry(s.unit).or_default() += s.damage;
    }
    for f in fights {
        *unit_damage.entry(f.attacker).or_default() += f.damage_to_attacker;
        match f.target {
            Target::Unit(u) => *unit_damage.entry(u).or_default() += f.damage_to_defender,
            Target::City(c) => *city_damage.entry(c).or_default() += f.damage_to_defender,
        }
    }
    for (unit, damage) in &carry.damage {
        *unit_damage.entry(*unit).or_default() += damage;
    }
    record_attack_reveals(state, fights);
    for (id, dmg) in &unit_damage {
        if let Some(u) = state.units.get_mut(id) {
            u.hp -= dmg;
            u.acted = true;
        }
    }
    for (id, dmg) in &city_damage {
        if let Some(c) = state.cities.get_mut(id) {
            c.hp = (c.hp - dmg).max(0);
            if *dmg > 0 {
                c.last_attacked_turn = Some(state.turn);
            }
        }
    }

    // Snapshot identities for events before anything is removed.
    for f in fights {
        let a = &state.units[&f.attacker];
        let (defender, defender_hp_after) = match f.target {
            Target::Unit(d) => {
                let d = &state.units[&d];
                (
                    Defender::Unit {
                        unit: d.id,
                        owner: d.owner,
                        unit_kind: d.kind,
                    },
                    d.hp.max(0),
                )
            }
            Target::City(c) => {
                let c = &state.cities[&c];
                (
                    Defender::City {
                        city: c.id,
                        owner: c.owner,
                    },
                    c.hp,
                )
            }
        };
        events.push(Event::Battle {
            attacker: a.id.into(),
            attacker_owner: a.owner,
            defender,
            at: f.at,
            attacker_at: Some(a.pos),
            attacker_strength: f.attacker_strength.clone(),
            defender_strength: f.defender_strength.clone(),
            damage_to_attacker: f.damage_to_attacker,
            damage_to_defender: f.damage_to_defender,
            attacker_roll: f.attacker_roll,
            defender_roll: f.defender_roll,
            attacker_hp_after: a.hp.max(0),
            defender_hp_after,
            ranged: f.kind == FightKind::Ranged,
            air_strike: data::is_aircraft(a.kind),
            meeting: f.kind == FightKind::Meeting,
            defender_leaving: f.defender_leaving,
        });
    }

    for s in shots {
        let city = &state.cities[&s.city];
        let unit = &state.units[&s.unit];
        events.push(Event::Battle {
            attacker: crate::events::Attacker::City { city: city.id },
            attacker_owner: city.owner,
            defender: Defender::Unit {
                unit: unit.id,
                owner: unit.owner,
                unit_kind: unit.kind,
            },
            at: unit.pos,
            attacker_at: Some(city.pos),
            attacker_strength: s.attack.clone(),
            defender_strength: s.defense.clone(),
            damage_to_attacker: 0,
            damage_to_defender: s.damage,
            attacker_roll: s.roll,
            defender_roll: 0,
            attacker_hp_after: city.hp,
            defender_hp_after: unit.hp.max(0),
            ranged: true,
            air_strike: false,
            meeting: false,
            defender_leaving: false,
        });
    }
    let dead: BTreeSet<UnitId> = unit_damage
        .keys()
        .copied()
        .filter(|id| state.units.get(id).is_some_and(|u| u.hp <= 0))
        .collect();
    let military_dead: Vec<_> = dead
        .iter()
        .filter_map(|id| {
            let u = &state.units[id];
            is_military(u.kind).then_some((*id, u.owner, u.kind, u.pos))
        })
        .collect();
    for id in &dead {
        destroy_unit(state, *id, events);
    }
    for (id, owner, kind, pos) in military_dead {
        let chained = carry.relief.get(&id).map(|_| stage_start);
        let pos = carry.relief.get(&id).copied().unwrap_or(pos);
        heal_sparta_killers(state, events, id, owner, kind, pos, chained);
        flower_wars(state, events, id, owner, kind, pos, stage_start, chained);
    }
    promotion::grant_xp(state, &xp, events);
    dead
}

/// Battle events include earlier stages of this turn, so a ranged hit can share a later kill.
/// All stage damage has landed before this runs; fallen attackers cannot receive healing.
/// A garrison killed by overflow damage (user decision 2026-10-04) is a kill by everyone who
/// damaged its city's tile: Battle events against the victim's own city at `victim_at` count
/// like Battle events against the victim. A relief unit killed by the assault's chained overflow
/// (`carried_damage`; `chained` is its stage's first event) is a kill by everyone who hit a unit of
/// its owner on that tile in that stage.
fn kill_participants(
    events: &[Event],
    victim_id: UnitId,
    victim_owner: PlayerId,
    victim_at: Hex,
    city_since: usize,
    chained: Option<usize>,
) -> BTreeSet<UnitId> {
    let mut participants = BTreeSet::new();
    for (index, event) in events.iter().enumerate() {
        let Event::Battle {
            attacker,
            defender,
            at,
            damage_to_attacker,
            damage_to_defender,
            ..
        } = event
        else {
            continue;
        };
        if let Defender::City { owner, .. } = defender
            && index >= city_since
            && *owner == victim_owner
            && *at == victim_at
            && *damage_to_defender > 0
            && let Attacker::Unit(attacker_id) = attacker
        {
            participants.insert(*attacker_id);
        }
        if let Defender::Unit { unit, owner, .. } = defender
            && chained.is_some_and(|since| index >= since)
            && *unit != victim_id
            && *owner == victim_owner
            && *at == victim_at
            && *damage_to_defender > 0
            && let Attacker::Unit(attacker_id) = attacker
        {
            participants.insert(*attacker_id);
        }
        if let Defender::Unit { unit, .. } = defender {
            if *unit == victim_id
                && *damage_to_defender > 0
                && let Attacker::Unit(attacker_id) = attacker
            {
                participants.insert(*attacker_id);
            }
            if attacker.unit() == Some(victim_id) && *damage_to_attacker > 0 {
                participants.insert(*unit);
            }
        }
    }
    participants
}

/// One population per slain military unit per participating Aztec empire, never per attacker.
/// Economic faction rules belong to the owner, not a hired Company's origin.
#[allow(clippy::too_many_arguments)]
fn flower_wars(
    state: &mut GameState,
    events: &mut Vec<Event>,
    victim_id: UnitId,
    victim_owner: PlayerId,
    victim_kind: UnitKind,
    victim_at: Hex,
    stage_start: usize,
    chained: Option<usize>,
) {
    if victim_owner.is_barbarian() || victim_kind == UnitKind::Scout {
        return;
    }
    let owners: BTreeSet<_> = kill_participants(
        events,
        victim_id,
        victim_owner,
        victim_at,
        stage_start,
        chained,
    )
    .into_iter()
    .filter_map(|id| state.units.get(&id))
    .filter(|u| {
        u.hp > 0
            && !u.owner.is_barbarian()
            && u.owner != victim_owner
            && state.player(u.owner).faction == Faction::Aztecs
            && !state
                .setup
                .rule_experiments
                .rule_off(FactionRule::AztecsFlowerWars)
            && data::unit_class(u.kind) == Some(crate::types::UnitClass::Melee)
    })
    .map(|u| u.owner)
    .collect();
    for owner in owners {
        let city = state
            .cities_of(owner)
            .min_by_key(|c| (c.pos.distance(victim_at), c.id))
            .map(|c| c.id);
        if let Some(city) = city {
            let c = state.cities.get_mut(&city).expect("nearest city");
            c.population += 1;
            let population = c.population;
            super::cities::settle_territory(state, city, events);
            events.push(Event::FlowerWars {
                city,
                owner,
                population,
            });
        }
    }
}

fn heal_sparta_killers(
    state: &mut GameState,
    events: &mut Vec<Event>,
    victim_id: UnitId,
    victim_owner: PlayerId,
    victim_kind: UnitKind,
    victim_at: Hex,
    chained: Option<usize>,
) {
    let participants = kill_participants(events, victim_id, victim_owner, victim_at, 0, chained);
    let mut active: Vec<_> = participants
        .into_iter()
        .filter(|id| {
            state.units.get(id).is_some_and(|u| {
                u.hp > 0
                    && u.owner != victim_owner
                    && state.player(u.owner).faction == Faction::Sparta
                    && !state
                        .setup
                        .rule_experiments
                        .rule_off(FactionRule::SpartaKillHeal)
            })
        })
        .collect();
    let mut remaining = 100;
    let mut healed = BTreeMap::<PlayerId, i32>::new();
    while remaining > 0 && !active.is_empty() {
        let share = (remaining / active.len() as i32).max(1);
        let mut next = Vec::new();
        for id in active {
            if remaining == 0 {
                break;
            }
            let u = state.units.get_mut(&id).expect("living participant");
            let amount = share.min(remaining).min(data::UNIT_HP - u.hp);
            u.hp += amount;
            remaining -= amount;
            *healed.entry(u.owner).or_default() += amount;
            if u.hp < data::UNIT_HP {
                next.push(id);
            }
        }
        active = next;
    }
    for (player, hp_healed) in healed {
        if hp_healed > 0 {
            events.push(Event::SpartaKillHeal {
                player,
                unit: victim_id,
                victim_owner,
                unit_kind: victim_kind,
                hp_healed,
            });
        }
    }
}

/// An attacker is seen by the team it attacked for one planning turn, even when it stands out
/// of that team's sight (`Unit::attack_revealed_to`). Planned from the stage's snapshot.
fn record_attack_reveals(state: &mut GameState, fights: &[Fight]) {
    let mut attack_reveals = Vec::new();
    for f in fights {
        let opponent = match f.target {
            Target::Unit(d) => state.team_of(state.units[&d].owner),
            Target::City(c) => state.team_of(state.cities[&c].owner),
        };
        attack_reveals.push((f.attacker, opponent));
    }
    for (attacker, team) in attack_reveals {
        let turn = state.turn;
        let unit = state.units.get_mut(&attacker).expect("attacker");
        if unit.attack_revealed_turn != Some(turn) {
            unit.attack_revealed_to.clear();
            unit.attack_revealed_turn = Some(turn);
        }
        unit.attack_revealed_to.insert(team);
    }
}

pub fn destroy_unit(state: &mut GameState, id: UnitId, events: &mut Vec<Event>) {
    if let Some(u) = state.units.remove(&id) {
        events.push(Event::UnitDestroyed {
            unit: u.id,
            owner: u.owner,
            unit_kind: u.kind,
            at: u.pos,
        });
    }
}

/// What a ranged attack on `target` by `player` would hit. On an enemy city's tile,
/// `combat::city_tile_defender` decides: the city while it has HP (its garrison is safe), its
/// visible garrison at 0 HP. Elsewhere, the visible enemy military unit. Lone civilians cannot
/// be shot.
pub fn ranged_target(
    state: &GameState,
    player: PlayerId,
    unit: UnitId,
    target: Hex,
    vision: &mut Option<BitSet>,
) -> Result<Target, String> {
    let u = state
        .units
        .get(&unit)
        .filter(|u| u.owner == player)
        .ok_or_else(|| "That unit is not yours".to_string())?;
    if !state.unit_in_open_land(u) {
        return Err("Cannot attack from closed foreign territory".into());
    }
    let stats = state.unit_stats_of(u);
    if stats.ranged <= 0 {
        return Err(format!("{} has no ranged attack", data::unit_name(u.kind)));
    }
    let dist = u.pos.distance(target);
    if dist < 1 || dist > stats.range {
        return Err(format!(
            "target is out of range: {dist} tiles away, range {}",
            stats.range
        ));
    }
    check_visible(state, player, target, vision)?;
    if !los::ranged_clear_with_tech(
        u.kind,
        state.player(u.owner).techs.contains(&crate::TechId::Steel),
        state
            .cities
            .values()
            .any(|c| c.owner == u.owner && c.pos == u.pos && data::wall_tier(&c.buildings) > 0),
        u.pos,
        target,
        |h| state.map.get(h).map(|t| t.terrain),
    ) {
        let at = los::blocker(u.pos, target, |h| state.map.get(h).map(|t| t.terrain)).unwrap();
        let terrain = state.map.get(at).map(|t| t.terrain);
        let mut options = if terrain == Some(Terrain::Mountains) {
            vec!["Move adjacent".to_string(), "around it".to_string()]
        } else {
            vec!["Fire from a hill".to_string(), "move adjacent".to_string()]
        };
        if let Some(c) = state
            .cities
            .values()
            .find(|c| c.owner == u.owner && c.pos == u.pos)
        {
            options.push(format!("build Walls in {}", c.name));
        }
        let last = options.pop().unwrap_or_default();
        let comma = if options.len() > 1 { "," } else { "" };
        let remedy = format!("{}{comma} or {last}.", options.join(", "));
        return Err(format!(
            "No line of sight: the {:?} at ({}, {}) blocks the shot. {remedy}",
            terrain.unwrap_or(Terrain::Mountains),
            at.q,
            at.r
        ));
    }
    target_at(state, player, target)
}

/// Why `player` cannot see `target` (a ranged or air target must be in sight).
pub fn check_visible(
    state: &GameState,
    player: PlayerId,
    target: Hex,
    vision: &mut Option<BitSet>,
) -> Result<(), String> {
    let idx = state
        .map
        .index(target)
        .ok_or_else(|| "target is off the map".to_string())?;
    let seen =
        vision.get_or_insert_with(|| visibility::visible_tiles(state, state.team_of(player)));
    if seen.get(idx) {
        Ok(())
    } else {
        Err("target is not visible".into())
    }
}

/// What a ranged or air attack by `player` on `target` would hit (range, sight and line of
/// sight already checked): see `ranged_target`.
pub fn target_at(state: &GameState, player: PlayerId, target: Hex) -> Result<Target, String> {
    let enemy_unit = state
        .units_at(target)
        .find(|v| is_military(v.kind) && hostile(state, player, v.owner));
    match (state.city_at(target), enemy_unit) {
        // Civ 6: the city absorbs the shot while it has HP; its garrison only at 0 HP.
        (Some(c), garrison) if hostile(state, player, c.owner) => Ok(
            match (
                combat::city_tile_defender(c.hp, garrison.is_some()),
                garrison,
            ) {
                (combat::CityTileDefender::Garrison, Some(g)) => Target::Unit(g.id),
                _ => Target::City(c.id),
            },
        ),
        (_, Some(m)) => Ok(Target::Unit(m.id)),
        // Say why a real target is off limits, rather than that nothing is there.
        (city, None) => {
            let not_at_war = |owner: PlayerId| !state.same_team(player, owner);
            let unit_there = state
                .units_at(target)
                .any(|v| is_military(v.kind) && not_at_war(v.owner));
            if city.is_some_and(|c| not_at_war(c.owner)) || unit_there {
                Err("you are not at War with its owner; declare War first to attack there".into())
            } else {
                Err("no enemy unit or city to attack there".into())
            }
        }
    }
}

/// A shot at a fallen, empty hostile city (0 HP, no garrison), held from the ranged phase for
/// whoever comes in to defend it (Sam, 2026-11-21, Geneva T67: "the order of events should
/// be: slip into garrison (with support), ranged attack, melee attack"). It fires in the
/// first movement round in which a defender stands on or steps into the city, before that
/// round's melee lands; if nobody comes, it lands on the empty city after movement as before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldShot {
    pub attacker: UnitId,
    pub owner: PlayerId,
    pub city: CityId,
}

/// Is a ranged hit on this city tile one to hold? Only a fallen city with nothing on it to
/// hit, by a ground unit (aircraft return to their hangars after the ranged phase).
fn held_city(state: &GameState, unit: UnitId, target: Target) -> Option<CityId> {
    let Target::City(c) = target else { return None };
    let city = state.cities.get(&c)?;
    let garrisoned = state
        .units_at(city.pos)
        .any(|u| is_military(u.kind) && u.owner == city.owner);
    let ground = state
        .units
        .get(&unit)
        .is_some_and(|u| !data::is_aircraft(u.kind));
    (city.hp <= 0 && !garrisoned && ground).then_some(c)
}

/// The held shots that found no defender: after movement they land on the city they were aimed
/// at, exactly as an unheld shot at an empty fallen city would have (no damage). A city that
/// changed hands or is no longer hostile is left alone.
pub fn release_held_shots(state: &mut GameState, held: &[HeldShot], events: &mut Vec<Event>) {
    let fights: Vec<Fight> = held
        .iter()
        .filter(|h| state.units.contains_key(&h.attacker))
        .filter(|h| {
            state
                .cities
                .get(&h.city)
                .is_some_and(|c| hostile(state, h.owner, c.owner))
        })
        .map(|h| {
            let pos = state.cities[&h.city].pos;
            let target = target_at(state, h.owner, pos).unwrap_or(Target::City(h.city));
            plan_fight(state, STAGE_RANGED, h.attacker, target, FightKind::Ranged)
        })
        .collect();
    if fights.is_empty() {
        return;
    }
    let carry = carried_damage(state, &fights, &BTreeSet::new(), &BTreeMap::new());
    apply_stage(state, &fights, &[], &carry, events);
}

/// Phase 2: every ranged attack is planned from the same snapshot, then all damage is applied
/// together. No retaliation. The attackers then cannot move this turn. Shots at a fallen, empty
/// hostile city are returned as [`HeldShot`]s for movement to fire at whoever comes in.
pub fn resolve_ranged(
    state: &mut GameState,
    orders: &[(PlayerId, UnitId, Hex)],
    events: &mut Vec<Event>,
) -> Vec<HeldShot> {
    let mut vision: BTreeMap<TeamId, Option<BitSet>> = BTreeMap::new();
    let mut fights = Vec::new();
    let mut held = Vec::new();
    for (player, unit, target) in orders {
        let ready = state
            .units
            .get(unit)
            .is_some_and(|u| u.moves_left > 0 && !u.acted);
        let result = if ready {
            let v = vision.entry(state.team_of(*player)).or_default();
            ranged_target(state, *player, *unit, *target, v)
        } else {
            Err("the unit cannot act this turn".into())
        };
        match result {
            Ok(t) => match held_city(state, *unit, t) {
                Some(city) => held.push(HeldShot {
                    attacker: *unit,
                    owner: *player,
                    city,
                }),
                None => fights.push(plan_fight(state, STAGE_RANGED, *unit, t, FightKind::Ranged)),
            },
            Err(reason) => events.push(Event::OrderFailed {
                player: *player,
                reason: format!("Ranged attack by unit {} failed: {reason}", unit.0),
            }),
        }
    }
    let mut shots = Vec::new();
    {
        // Only walled cities fire, and a defeated city (0 HP in this round's snapshot) doesn't,
        // as in Civ 6.
        let fires = |c: &&crate::state::City| {
            combat::city_can_fire(data::has_building(&c.buildings, BuildingKind::Walls), c.hp)
        };
        for city in state.cities.values().filter(fires) {
            let v = vision
                .entry(state.team_of(city.owner))
                .or_default()
                .get_or_insert_with(|| visibility::visible_tiles(state, state.team_of(city.owner)));
            let target = state
                .units
                .values()
                .filter(|u| {
                    is_military(u.kind)
                        && is_threat(state, city.owner, u.owner)
                        && (1..=2).contains(&city.pos.distance(u.pos))
                        && state.map.index(u.pos).is_some_and(|i| v.get(i))
                })
                .min_by_key(|u| (u.hp, u.id));
            let Some(u) = target else { continue };
            let attack = combat::city_defense_for_rules(
                state.setup.rule_experiments,
                state.player(city.owner).faction,
                city.population,
                data::has_building(&city.buildings, BuildingKind::Walls),
                data::fort_tier(&city.buildings),
                city.hp,
                state.era_of(city.owner),
            );
            let mut defense = combat::unit_defense_strength_for_rules(
                state.setup.rule_experiments.faction_rules_off,
                state.player(u.owner).faction,
                state.made_by(u),
                u.kind,
                u.hp,
                state.map.get(u.pos).map_or(Terrain::Plains, |t| t.terrain),
                u.fortify,
                promotion::level_for_rules(
                    state.setup.rule_experiments.faction_rules_off,
                    state.player(u.owner).faction,
                    u.xp,
                ),
            );
            combat::add_phalanx_rules(
                &mut defense,
                state.setup.rule_experiments.faction_rules_off,
                state.made_by(u),
                u.kind,
                has_phalanx_neighbour(state, u),
            );
            crate::resources::add_unsupplied(
                &mut defense,
                state.made_by(u),
                u.kind,
                false,
                crate::resources::unit_unsupplied(state, u),
            );
            let mut rng = Rng::derive(
                state.dice_seed(),
                &[
                    u64::from(state.turn),
                    purpose::COMBAT,
                    STAGE_RANGED,
                    key(Target::City(city.id)),
                    key(Target::Unit(u.id)),
                ],
            );
            let roll = if average_rolls() {
                100
            } else {
                rng.range(i64::from(ROLL_MIN), i64::from(ROLL_MAX)) as i32
            };
            let damage = combat::damage(attack.total, defense.total, roll);
            shots.push(CityShot {
                city: city.id,
                unit: u.id,
                attack,
                defense,
                roll,
                damage,
            });
        }
    }
    let carry = carried_damage(state, &fights, &BTreeSet::new(), &BTreeMap::new());
    apply_stage(state, &fights, &shots, &carry, events);
    let shooters = fights.iter().map(|f| f.attacker);
    for id in shooters
        .chain(held.iter().map(|h| h.attacker))
        .collect::<Vec<_>>()
    {
        if let Some(u) = state.units.get_mut(&id) {
            u.moves_left = 0;
            u.fortify = 0;
            u.acted = true;
        }
    }
    held
}

/// A city that changes hands is no longer a destination to attack. Every unit of the captor's
/// team or an ally still ordered at it (a goto, a confirmed route or an attack aimed at the city
/// tile) drops the order and returns to its owner's decision cycle. The units standing on the
/// city, including the one that moved in, are untouched; ordinary stacking rules govern any
/// later move in.
fn end_orders_into_city(state: &mut GameState, city_pos: Hex, captor: PlayerId) {
    let ended: Vec<UnitId> = state
        .units
        .values()
        .filter(|u| {
            u.pos != city_pos
                && (state.same_team(captor, u.owner) || state.allied(captor, u.owner))
                && (u.goto == Some(city_pos) || u.path.last() == Some(&city_pos))
        })
        .map(|u| u.id)
        .collect();
    for id in ended {
        let u = state.units.get_mut(&id).expect("unit");
        u.goto = None;
        u.path.clear();
        u.path_stop_on_block = false;
        u.blocked_turns = 0;
        u.attack_target = None;
        u.needs_orders = true;
    }
}

/// A melee attacker took the city: it changes hands and hostile military units are destroyed.
/// Civilians remain for movement's capture pass.
pub fn capture_city(
    state: &mut GameState,
    city: CityId,
    captor: PlayerId,
    events: &mut Vec<Event>,
) {
    // Barbarians besiege cities but never take them (`rules::barbarians`). Movement never names
    // one as a captor; refusing here keeps the resolver from reaching for the barbarians' missing
    // player record (`GameState::player_mut`) if a caller ever does.
    if captor.is_barbarian() {
        return;
    }
    // Pay once per city and captor, including a Celtic capture that immediately sacks it.
    let captive_from = if greece_captive_available(state, city, captor) {
        let from = state.cities[&city].owner;
        state
            .cities
            .get_mut(&city)
            .unwrap()
            .greece_captive_paid
            .insert(captor);
        let p = state.player_mut(captor);
        p.free_techs_pending = p.free_techs_pending.saturating_add(1);
        p.free_techs_from = Some(from);
        p.free_techs_captured = Some(city);
        Some(from)
    } else {
        None
    };
    // An original capital's VP is held, not scored here: `scoring::vp` counts it while a rival of
    // its founder owns it, and end-of-turn scoring reports the change of hands. The one exception
    // is the Celtic sack below, which keeps the point for good.
    let rules = state.setup.rule_experiments;
    // A city the captor founded comes home intact (designer, 2026-11-14): the sack is for
    // everyone else's cities, teammates' included.
    let own_founding = state.cities.get(&city).is_some_and(|c| c.founder == captor);
    if state.player(captor).faction == Faction::Celts
        && !rules.rule_off(FactionRule::CeltsNoKeep)
        && !own_founding
    {
        let Some(victim) = state.cities.get(&city) else {
            return;
        };
        let (from, at, original_capital) = (victim.owner, victim.pos, victim.original_capital);
        // A rival's original capital burned in the sack keeps its point for good (Celtic
        // exception to held capital VP; the city is gone, so nobody can take it back).
        let sacked_capital = (original_capital
            && state.team_of(victim.founder) != state.team_of(captor))
        .then(|| victim.name.clone());
        let (prize, walled) = (
            crate::objectives::prize_city(victim),
            crate::objectives::walled_city(victim),
        );
        let takes = !rules.rule_off(FactionRule::CeltsSack);
        let sack_gold = if takes {
            crate::economy::celts_sack_gold(victim.population)
        } else {
            0
        };
        state.player_mut(captor).gold =
            crate::validation::credit_gold(state.player(captor).gold, sack_gold);
        super::air::lose_city_aircraft(state, city, events);
        state.cities.remove(&city);
        for i in 0..state.map.tiles.len() {
            if state.map.tiles[i].city == Some(city) {
                let t = &mut state.map.tiles[i];
                t.city = None;
                t.owner = None;
                t.improved = false;
                t.landmark = None;
            }
        }
        // The sack also raises a Celtic warband camp near the ruins.
        let camp = if takes {
            super::barbarians::raise_sack_camp(state, captor, at, events)
        } else {
            None
        };
        events.push(Event::CityRazed {
            city,
            from,
            by: captor,
            at,
            original_capital,
            sack_gold: Some(sack_gold),
            camp,
            ordered: false,
            prize,
            walled,
        });
        if let Some(from) = captive_from {
            events.push(Event::AthensCityCaptured {
                city,
                from,
                to: captor,
            });
        }
        if let Some(name) = sacked_capital {
            state
                .player_mut(captor)
                .capitals_sacked
                .push((city, name.clone()));
            events.push(Event::CapitalSacked {
                captor,
                capital: city,
                name,
            });
        }
        return;
    }
    let bounty = state
        .cities
        .get(&city)
        .map_or(0, |c| crate::economy::swiss_capture_bounty(state, c));
    let sparta = state.player(captor).faction == Faction::Sparta
        && !state
            .setup
            .rule_experiments
            .rule_off(FactionRule::SpartaKillHeal);
    // The captor takes the city, not its aircraft: they are lost with it.
    super::air::lose_city_aircraft(state, city, events);
    // Occupation lock: the new record is the civ just taken from; none for a city its captor's
    // team is retaking, and barbarians never lock.
    let occupied_from = state.cities.get(&city).and_then(|c| {
        (!captor.is_barbarian()
            && !c.owner.is_barbarian()
            && c.occupied_from
                .is_none_or(|prev| state.team_of(captor) != state.team_of(prev)))
        .then_some(c.owner)
    });
    let Some(c) = state.cities.get_mut(&city) else {
        return;
    };
    let from = c.owner;
    let pos = c.pos;
    let plundered: Vec<_> = if state.setup.rule_experiments.market_plunder
        && !data::has_building(&c.buildings, BuildingKind::Walls)
    {
        c.buildings
            .iter()
            .copied()
            .filter(|b| data::building_family(*b) == BuildingKind::Market)
            .map(|b| (b, data::building_stats(b).cost * 2))
            .collect()
    } else {
        Vec::new()
    };
    // The paid infrastructure is consumed: swapping the same city back and forth
    // cannot mint another payout without rebuilding the market.
    for (building, _) in &plundered {
        c.buildings.remove(building);
    }
    c.occupied_from = occupied_from;
    c.owner = captor;
    c.build_automation = None;
    c.last_auto_pick = None;
    c.auto_queued = None;
    // A captured city loses half its population (rounded down, never below 1); one coming home
    // to its founder keeps it all.
    if c.founder != captor {
        c.population = (c.population / 2).max(1);
    }
    c.food_stored = 0;
    c.production_stored = 0;
    c.production_banked.clear();
    c.producing = None;
    c.production_queue.clear();
    c.production_queue_notified.clear();
    c.wonder_lost = None;
    c.bought = None;
    c.is_capital = false;
    c.resistance = CAPTURE_RESISTANCE_TURNS;
    for i in 0..state.map.tiles.len() {
        if state.map.tiles[i].city == Some(city) {
            // A captured city keeps its improved land: the captor inherits it.
            state.map.tiles[i].owner = Some(captor);
        }
    }
    let doomed: Vec<UnitId> = state
        .units_at(pos)
        .filter(|u| is_military(u.kind) && hostile(state, captor, u.owner))
        .map(|u| u.id)
        .collect();
    for id in doomed {
        destroy_unit(state, id, events);
    }
    if bounty > 0 {
        state.player_mut(from).gold -= bounty;
        state.player_mut(captor).gold =
            crate::validation::credit_gold(state.player(captor).gold, bounty);
    }
    let mut conquest_healed = 0;
    if sparta {
        let mut healed = 0;
        for unit in state.units.values_mut() {
            if unit.owner == captor && is_military(unit.kind) && unit.pos.distance(pos) <= 2 {
                if unit.hp < data::UNIT_HP {
                    healed += 1;
                }
                unit.hp = data::UNIT_HP;
            }
        }
        // Report follows CityCaptured, matching the mock and the turn report's chronology.
        conquest_healed = healed;
    }
    events.push(Event::CityCaptured {
        city,
        from,
        to: captor,
        bounty: (bounty > 0).then_some(bounty),
    });
    if let Some(from) = captive_from {
        events.push(Event::AthensCityCaptured {
            city,
            from,
            to: captor,
        });
    }
    end_orders_into_city(state, pos, captor);
    if sparta {
        events.push(Event::SpartaConquest {
            city,
            player: captor,
            units_healed: conquest_healed,
        });
    }
    for (building, gold) in plundered {
        state.player_mut(captor).gold =
            crate::validation::credit_gold(state.player(captor).gold, gold);
        events.push(Event::BuildingDismantled {
            city,
            owner: captor,
            building,
            reason: format!("experimental capture plunder: +{gold} Gold to captor"),
        });
    }
}

/// A public city prize, viewed using only the captor's own payout history and known techs.
pub fn greece_captive_available(state: &GameState, city: CityId, captor: PlayerId) -> bool {
    let Some(c) = state.cities.get(&city) else {
        return false;
    };
    !captor.is_barbarian()
        && !c.owner.is_barbarian()
        && c.owner != captor
        && state.player(c.owner).faction == Faction::Athens
        && state.team_of(c.owner) != state.team_of(captor)
        && !state
            .setup
            .rule_experiments
            .rule_off(FactionRule::AthensGreeceCaptive)
        && !c.greece_captive_paid.contains(&captor)
        && crate::rules::actions::unknown_techs(state, captor) > 0
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use crate::orders::Order;

    #[test]
    fn flower_wars_grows_nearest_city_once_and_claims_a_tile() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Aztecs;
        let near = city(&mut s, 0, hex(2, 3), 1);
        let far = city(&mut s, 0, hex(10, 8), 1);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let b = unit(&mut s, 0, UnitKind::Spearman, hex(4, 2));
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        let fights = [
            forced_hit(&s, a, victim, 50, 0),
            forced_hit(&s, b, victim, 50, 0),
        ];
        let before = s.cities[&near].grown_tiles.len();
        let mut events = vec![];
        apply_fights(&mut s, &fights, &mut events);
        assert_eq!(s.cities[&near].population, 2);
        assert_eq!(s.cities[&far].population, 1);
        assert_eq!(s.cities[&near].grown_tiles.len(), before + 1);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::FlowerWars { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn flower_wars_excludes_scouts_barbarians_ranged_mounted_and_dead_participants() {
        for (killer, victim_kind, barbarian, taken) in [
            (UnitKind::Spearman, UnitKind::Scout, false, 0),
            (UnitKind::Spearman, UnitKind::Spearman, true, 0),
            (UnitKind::Archer, UnitKind::Spearman, false, 0),
            (UnitKind::Horseman, UnitKind::Spearman, false, 0),
            (UnitKind::Spearman, UnitKind::Spearman, false, 100),
        ] {
            let mut s = game(&[0, 1]);
            s.player_mut(PlayerId(0)).faction = Faction::Aztecs;
            let c = city(&mut s, 0, hex(2, 3), 1);
            let a = unit(&mut s, 0, killer, hex(3, 3));
            let v = unit(&mut s, 1, victim_kind, hex(4, 3));
            if barbarian {
                s.units.get_mut(&v).unwrap().owner = PlayerId::BARBARIAN;
            }
            let hit = forced_hit(&s, a, v, 100, taken);
            let mut events = vec![];
            apply_fights(&mut s, &[hit], &mut events);
            assert_eq!(
                s.cities[&c].population, 1,
                "{killer:?} {victim_kind:?} {barbarian} {taken}"
            );
            assert!(!events.iter().any(|e| matches!(e, Event::FlowerWars { .. })));
        }
    }

    #[test]
    fn flower_wars_counts_defensive_kills_and_breaks_city_ties_by_id() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Aztecs;
        let at = hex(5, 4);
        let spots = at.neighbors();
        let first = city(&mut s, 0, spots[0], 1);
        let second = city(&mut s, 0, spots[3], 1);
        let a = unit(&mut s, 1, UnitKind::Spearman, at);
        let d = unit(&mut s, 0, UnitKind::Spearman, spots[1]);
        let fight = forced_hit(&s, a, d, 0, 100);
        apply_fights(&mut s, &[fight], &mut vec![]);
        assert_eq!(s.cities[&first].population, 2);
        assert_eq!(s.cities[&second].population, 1);
    }

    #[test]
    fn flower_wars_city_overflow_counts_surviving_melee_participation() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Aztecs;
        let home = city(&mut s, 0, hex(2, 3), 1);
        let c = city(&mut s, 1, hex(5, 3), 2);
        s.cities.get_mut(&c).unwrap().hp = 1;
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        s.units.get_mut(&g).unwrap().hp = 1;
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let mut fight = plan_fight(&s, 1, a, Target::City(c), FightKind::Melee);
        fight.damage_to_defender = 100;
        fight.damage_to_attacker = 0;
        apply_fights(&mut s, &[fight], &mut vec![]);
        assert!(!s.units.contains_key(&g));
        assert_eq!(s.cities[&home].population, 2);
    }

    #[test]
    fn flower_wars_without_a_city_does_not_panic_or_create_population() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Aztecs;
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let v = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        let hit = forced_hit(&s, a, v, 100, 0);
        let mut events = vec![];
        apply_fights(&mut s, &[hit], &mut events);
        assert!(!events.iter().any(|e| matches!(e, Event::FlowerWars { .. })));
    }

    fn shape(f: &Fight) -> (i32, crate::Breakdown) {
        (f.damage_to_defender, f.defender_strength.clone())
    }

    fn melee_fight(s: &GameState, attacker: UnitId, city: CityId) -> Fight {
        plan_fight(s, 1, attacker, Target::City(city), FightKind::Melee)
    }

    /// Damage the same fight deals with no Walls, for comparison.
    fn open_city_fight(s: &mut GameState, attacker: UnitId, city: CityId) -> Fight {
        let walls: Vec<BuildingKind> = s.cities[&city].buildings.iter().copied().collect();
        s.cities.get_mut(&city).unwrap().buildings.clear();
        let fight = melee_fight(s, attacker, city);
        s.cities.get_mut(&city).unwrap().buildings.extend(walls);
        fight
    }

    #[test]
    fn walls_halve_every_attack_on_a_citys_hp_but_not_on_its_garrison() {
        let mut s = game(&[0, 1]);
        let city = walled_city(&mut s, 1, hex(5, 3), 2);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let shooter = unit(&mut s, 0, UnitKind::Archer, hex(3, 3));
        let walled = melee_fight(&s, attacker, city);
        let normal = combat::damage(
            walled.attacker_strength.total,
            walled.defender_strength.total,
            walled.attacker_roll,
        );
        assert_eq!(walled.damage_to_defender, normal / 2);
        let shot = plan_fight(&s, 1, shooter, Target::City(city), FightKind::Ranged);
        let normal = combat::damage(
            shot.attacker_strength.total,
            shot.defender_strength.total,
            shot.attacker_roll,
        );
        assert_eq!(shot.damage_to_defender, normal / 2);

        s.cities.get_mut(&city).unwrap().hp = 0;
        let garrison = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        let direct = plan_fight(&s, 1, attacker, Target::Unit(garrison), FightKind::Melee);
        assert_eq!(
            direct.damage_to_defender,
            combat::damage(
                direct.attacker_strength.total,
                direct.defender_strength.total,
                direct.attacker_roll
            )
        );
    }

    #[test]
    fn overflow_uses_the_same_unsupplied_garrison_defense_as_direct_combat() {
        let mut s = game(&[0, 1]);
        let pos = hex(5, 3);
        set_terrain(&mut s, pos, Terrain::Plains);
        let c = city(&mut s, 1, pos, 1);
        s.cities.get_mut(&c).unwrap().hp = 200;
        let g = unit(&mut s, 1, UnitKind::Pikeman, pos);
        s.player_mut(PlayerId(1))
            .supply
            .push(crate::resources::SupplyRecord {
                user: crate::resources::SupplyUser::Unit { unit: g },
                resource: crate::resources::Resource::Iron,
                since: 0,
                idle: true,
            });
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let direct = plan_fight(&s, 1, a, Target::Unit(g), FightKind::Melee);
        assert_eq!(direct.defender_strength.total, 24);

        let mut fight = plan_fight(&s, 1, a, Target::City(c), FightKind::Melee);
        fight.damage_to_defender = 216;
        let carry = carried_damage(
            &s,
            std::slice::from_ref(&fight),
            &BTreeSet::new(),
            &BTreeMap::new(),
        );
        let direct_damage = combat::damage(direct.attacker_strength.total, 24, fight.attacker_roll);
        assert_eq!(
            carry.damage,
            BTreeMap::from([(g, combat::carried_share(direct_damage, 16, 216))]),
            "16 of the 216 carries on, as that share of the damage the unsupplied garrison (24) takes directly"
        );
    }

    #[test]
    fn an_unwalled_city_takes_full_damage() {
        let mut s = game(&[0, 1]);
        let city = city(&mut s, 1, hex(5, 3), 2);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let f = melee_fight(&s, attacker, city);
        assert_eq!(
            f.damage_to_defender,
            combat::damage(
                f.attacker_strength.total,
                f.defender_strength.total,
                f.attacker_roll
            )
        );
    }

    #[test]
    fn a_siege_unit_ignores_walls_of_its_tier() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).techs.insert(TechId::Mathematics);
        let city = walled_city(&mut s, 1, hex(5, 3), 2);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let alone = melee_fight(&s, attacker, city);
        s.units.get_mut(&attacker).unwrap().siege = true;
        let covered = melee_fight(&s, attacker, city);
        let open = open_city_fight(&mut s, attacker, city);
        assert_eq!(covered.defender_strength, open.defender_strength);
        assert_eq!(covered.damage_to_defender, open.damage_to_defender);
        assert!(covered.damage_to_defender > alone.damage_to_defender * 2);
        assert!(covered.defender_strength.total < alone.defender_strength.total);
    }

    #[test]
    fn siege_belongs_to_the_unit_that_bought_it() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).techs.insert(TechId::Mathematics);
        let city = walled_city(&mut s, 1, hex(5, 3), 2);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let alone = melee_fight(&s, attacker, city);
        let other = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        s.units.get_mut(&other).unwrap().siege = true;
        assert_eq!(shape(&melee_fight(&s, attacker, city)), shape(&alone));
    }

    #[test]
    fn siege_without_a_siege_technology_ignores_nothing() {
        let mut s = game(&[0, 1]);
        let city = walled_city(&mut s, 1, hex(5, 3), 2);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let alone = melee_fight(&s, attacker, city);
        s.units.get_mut(&attacker).unwrap().siege = true;
        assert_eq!(
            shape(&melee_fight(&s, attacker, city)),
            shape(&alone),
            "no Mathematics, no siege tier"
        );
    }

    #[test]
    fn an_industrial_unit_takes_a_star_fort_city_at_full_damage_without_siege() {
        // Designer, 2026-10-28: Siege is built in from the Industrial era, whatever the owner knows.
        let mut s = game(&[0, 1]);
        let city = walled_city(&mut s, 1, hex(5, 3), 3);
        s.cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .extend([BuildingKind::Castle, BuildingKind::StarFort]);
        assert_eq!(data::wall_tier(&s.cities[&city].buildings), 3);
        let rifle = unit(&mut s, 0, UnitKind::Rifleman, hex(4, 3));
        assert!(!s.units[&rifle].siege, "nothing bought");
        assert_eq!(
            data::siege_tier(&s.player(PlayerId(0)).techs),
            0,
            "no siege technology"
        );
        assert_eq!(siege_cover(&s, &s.units[&rifle]), data::MAX_WALL_TIER);
        let walled = melee_fight(&s, rifle, city);
        let open = open_city_fight(&mut s, rifle, city);
        assert_eq!(
            shape(&walled),
            shape(&open),
            "no wall strength, full damage"
        );
        assert!(
            walled
                .defender_strength
                .parts
                .iter()
                .all(|(label, _)| !label.contains("Castle")
                    && !label.contains("Star Fort")
                    && !label.contains("Walls"))
        );
        // A Renaissance Musketman with the same (lack of) technology is halved.
        let musket = unit(&mut s, 0, UnitKind::Musketman, hex(4, 4));
        let halved = melee_fight(&s, musket, city);
        let musket_open = open_city_fight(&mut s, musket, city);
        assert!(halved.damage_to_defender < musket_open.damage_to_defender);
        // The forecast says the same, and why.
        crate::visibility::update_explored(&mut s);
        let view = crate::view::player_view(&s, PlayerId(0));
        let f = crate::forecast::forecast_attack(&view, rifle, hex(5, 3)).expect("forecast");
        let w = f.walls.expect("a walled city");
        assert_eq!(
            (w.tier, w.covered, w.percent, w.built_in),
            (3, 3, 100, true)
        );
        assert_eq!(f.defender_strength, walled.defender_strength);
    }

    #[test]
    fn siege_below_the_walls_tier_ignores_only_the_lower_tiers() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).techs.insert(TechId::Mathematics);
        let city = walled_city(&mut s, 1, hex(5, 3), 3);
        s.cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .insert(BuildingKind::Castle);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        s.units.get_mut(&attacker).unwrap().siege = true;
        let partial = melee_fight(&s, attacker, city);
        let walls = data::building_stats_for(Faction::India, BuildingKind::Walls).city_strength;
        let castle = data::building_stats(BuildingKind::Castle).city_strength;
        let base = combat::city_defense_vs(
            s.setup.rule_experiments,
            Faction::India,
            3,
            0,
            0,
            s.cities[&city].hp,
            0,
        );
        assert_eq!(partial.defender_strength.total, base.total + castle);
        let uncovered = combat::city_defense_vs(
            s.setup.rule_experiments,
            Faction::India,
            3,
            2,
            0,
            s.cities[&city].hp,
            0,
        );
        assert_eq!(uncovered.total, base.total + walls + castle);
        let normal = combat::damage(
            partial.attacker_strength.total,
            partial.defender_strength.total,
            partial.attacker_roll,
        );
        assert_eq!(
            partial.damage_to_defender,
            normal * combat::SIEGE_PARTIAL_PERCENT / 100
        );
        s.player_mut(PlayerId(0)).techs.insert(TechId::Machinery);
        let full = melee_fight(&s, attacker, city);
        assert_eq!(full.defender_strength.total, base.total);
        assert_eq!(
            full.damage_to_defender,
            combat::damage(
                full.attacker_strength.total,
                full.defender_strength.total,
                full.attacker_roll
            )
        );
    }

    #[test]
    fn air_strikes_are_not_halved_by_walls() {
        assert_eq!(combat::wall_damage_percent(1, 0, true), 100);
        assert_eq!(
            combat::wall_damage_percent(1, 0, false),
            combat::WALLS_DAMAGE_PERCENT
        );
    }

    #[test]
    fn mongols_cannot_buy_siege_before_medieval_even_after_a_workshop_capture() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Mongols;
        let city_id = city(&mut s, 1, hex(5, 3), 2);
        s.cities
            .get_mut(&city_id)
            .unwrap()
            .buildings
            .insert(BuildingKind::Workshop);
        assert!(!data::siege_available(Faction::Mongols, 0));

        let mut events = Vec::new();
        capture_city(&mut s, city_id, PlayerId(0), &mut events);

        assert!(!data::siege_available(Faction::Mongols, 0));
        assert!(data::siege_available(
            Faction::Mongols,
            data::MONGOL_WALLS_ERA
        ));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Event::CityCaptured { city, .. } if *city == city_id))
        );
    }

    fn forced_hit(
        state: &GameState,
        attacker: UnitId,
        victim: UnitId,
        dealt: i32,
        taken: i32,
    ) -> Fight {
        let mut fight = plan_fight(state, 1, attacker, Target::Unit(victim), FightKind::Melee);
        fight.damage_to_defender = dealt;
        fight.damage_to_attacker = taken;
        fight
    }

    #[test]
    fn sparta_overflow_kill_of_a_garrison_shares_the_heal_among_the_city_attackers() {
        // Nobody attacked the garrison itself: the city absorbed the hits and the excess
        // killed it. Everyone who damaged the city tile shares the 100 HP.
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let c = city(&mut s, 1, hex(5, 3), 2);
        s.cities.get_mut(&c).unwrap().hp = 1;
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        s.units.get_mut(&g).unwrap().hp = 1;
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
        let b = unit(&mut s, 0, UnitKind::Spearman, hex(5, 2));
        let bystander = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
        for id in [a, b, bystander] {
            s.units.get_mut(&id).unwrap().hp = 40;
        }
        let fights: Vec<_> = [a, b]
            .into_iter()
            .map(|id| {
                let mut f = plan_fight(&s, 1, id, Target::City(c), FightKind::Melee);
                f.damage_to_defender = 30;
                f.damage_to_attacker = 0;
                f
            })
            .collect();
        let mut events = Vec::new();
        apply_fights(&mut s, &fights, &mut events);
        assert!(!s.units.contains_key(&g), "overflow killed the garrison");
        assert_eq!((s.units[&a].hp, s.units[&b].hp), (90, 90));
        assert_eq!(
            s.units[&bystander].hp, 40,
            "only those who hit the city tile heal"
        );
        let healed: i32 = events
            .iter()
            .filter_map(|e| match e {
                Event::SpartaKillHeal { hp_healed, .. } => Some(*hp_healed),
                _ => None,
            })
            .sum();
        assert_eq!(healed, 100);
    }

    #[test]
    fn sparta_mutual_kill_removes_both_before_any_healing() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        s.units.get_mut(&attacker).unwrap().hp = 10;
        s.units.get_mut(&victim).unwrap().hp = 10;
        let fight = forced_hit(&s, attacker, victim, 10, 10);
        let mut events = Vec::new();
        apply_fights(&mut s, &[fight], &mut events);
        assert!(!s.units.contains_key(&attacker));
        assert!(!s.units.contains_key(&victim));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SpartaKillHeal { .. }))
        );
    }

    #[test]
    fn sparta_scout_kill_heals_solo_survivor_up_to_one_hundred() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let victim = unit(&mut s, 1, UnitKind::Scout, hex(4, 3));
        s.units.get_mut(&attacker).unwrap().hp = 1;
        s.units.get_mut(&victim).unwrap().hp = 10;
        let mut events = Vec::new();
        let fight = forced_hit(&s, attacker, victim, 10, 0);
        apply_fights(&mut s, &[fight], &mut events);
        assert_eq!(s.units[&attacker].hp, 100);
        assert_eq!(s.player(PlayerId(0)).gold, 0);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::SpartaKillHeal { hp_healed: 99, .. }))
        );
    }

    #[test]
    fn sparta_ranged_hit_in_an_earlier_stage_shares_a_later_melee_kill() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let shooter = unit(&mut s, 0, UnitKind::Archer, hex(2, 3));
        let soldier = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        s.units.get_mut(&shooter).unwrap().hp = 40;
        s.units.get_mut(&soldier).unwrap().hp = 40;
        s.units.get_mut(&victim).unwrap().hp = 30;
        let mut events = Vec::new();
        let mut shot = plan_fight(&s, 0, shooter, Target::Unit(victim), FightKind::Ranged);
        shot.damage_to_defender = 20;
        shot.damage_to_attacker = 0;
        apply_fights(&mut s, &[shot], &mut events);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SpartaKillHeal { .. }))
        );
        let blow = forced_hit(&s, soldier, victim, 10, 0);
        apply_fights(&mut s, &[blow], &mut events);
        assert!(!s.units.contains_key(&victim));
        assert_eq!((s.units[&shooter].hp, s.units[&soldier].hp), (90, 90));
    }

    #[test]
    fn non_sparta_kills_heal_nobody() {
        let mut s = game(&[0, 1]);
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        s.units.get_mut(&attacker).unwrap().hp = 40;
        s.units.get_mut(&victim).unwrap().hp = 10;
        let mut events = Vec::new();
        let fight = forced_hit(&s, attacker, victim, 10, 0);
        apply_fights(&mut s, &[fight], &mut events);
        assert!(!s.units.contains_key(&victim));
        assert_eq!(
            s.units[&attacker].hp, 40,
            "only Spartan units heal by killing"
        );
    }

    #[test]
    fn sparta_three_way_kill_splits_and_water_fills_by_unit_id() {
        for starting_hp in [[1, 1, 1], [99, 1, 1]] {
            let mut s = game(&[0, 1]);
            s.player_mut(PlayerId(0)).faction = Faction::Sparta;
            let attackers = [hex(3, 3), hex(3, 4), hex(4, 4)]
                .map(|pos| unit(&mut s, 0, UnitKind::Spearman, pos));
            let victim = unit(&mut s, 1, UnitKind::Scout, hex(4, 3));
            s.units.get_mut(&victim).unwrap().hp = 30;
            for (id, hp) in attackers.iter().zip(starting_hp) {
                s.units.get_mut(id).unwrap().hp = hp;
            }
            let fights: Vec<_> = attackers
                .iter()
                .map(|&id| forced_hit(&s, id, victim, 10, 0))
                .collect();
            let mut events = Vec::new();
            apply_fights(&mut s, &fights, &mut events);
            let actual = attackers.map(|id| s.units[&id].hp);
            assert_eq!(
                actual,
                if starting_hp[0] == 1 {
                    [35, 34, 34]
                } else {
                    [100, 51, 50]
                }
            );
            assert_eq!(
                events
                    .iter()
                    .filter_map(|e| match e {
                        Event::SpartaKillHeal { hp_healed, .. } => Some(*hp_healed),
                        _ => None,
                    })
                    .sum::<i32>(),
                100
            );
        }
    }

    #[test]
    fn sparta_capture_heals_nearby_without_gold() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        s.player_mut(PlayerId(1)).faction = Faction::India;
        let city_id = city(&mut s, 1, hex(5, 5), 3);
        city(&mut s, 1, hex(9, 5), 3);
        s.player_mut(PlayerId(1)).gold = 120;
        s.cities.get_mut(&city_id).unwrap().production_stored = 13;
        let near = unit(&mut s, 0, UnitKind::Spearman, hex(4, 5));
        let far = unit(&mut s, 0, UnitKind::Spearman, hex(1, 1));
        s.units.get_mut(&near).unwrap().hp = 20;
        s.units.get_mut(&far).unwrap().hp = 20;
        let mut events = Vec::new();
        capture_city(&mut s, city_id, PlayerId(0), &mut events);
        assert_eq!(s.player(PlayerId(0)).gold, 0);
        assert_eq!(s.player(PlayerId(1)).gold, 120);
        assert_eq!(s.units[&near].hp, 100);
        assert_eq!(s.units[&far].hp, 20);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::SpartaConquest {
                units_healed: 1,
                ..
            }
        )));
    }

    #[test]
    fn swiss_bounty_is_the_only_gold_from_sparta_capture() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let city_id = city(&mut s, 1, hex(5, 5), 3);
        s.player_mut(PlayerId(1)).gold = 100;
        let mut events = Vec::new();
        capture_city(&mut s, city_id, PlayerId(0), &mut events);
        assert_eq!(s.player(PlayerId(1)).gold, 0);
        assert_eq!(s.player(PlayerId(0)).gold, 100);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::SpartaConquest {
                units_healed: 0,
                ..
            }
        )));
    }

    #[test]
    fn city_shots_choose_lowest_hp_then_id_once_and_resolve_simultaneously() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let c = walled_city(&mut s, 0, hex(3, 3), 2);
        let target = unit(&mut s, 1, UnitKind::Archer, hex(5, 3));
        let tie = unit(&mut s, 1, UnitKind::Spearman, hex(3, 5));
        let healthy = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        let civilian = unit(&mut s, 1, UnitKind::Settler, hex(3, 4));
        for id in [target, tie, civilian] {
            s.units.get_mut(&id).unwrap().hp = 1;
        }
        let old_hp = s.cities[&c].hp;
        let mut events = Vec::new();
        resolve_ranged(&mut s, &[(PlayerId(1), target, hex(3, 3))], &mut events);
        assert!(!s.units.contains_key(&target));
        assert_eq!(s.units[&tie].hp, 1);
        assert_eq!(s.units[&healthy].hp, 100);
        assert_eq!(s.units[&civilian].hp, 1);
        assert!(
            s.cities[&c].hp < old_hp,
            "the killed Ranger still shoots from the same snapshot"
        );
        let shot: Vec<_> = events.iter().filter(|e| matches!(e, Event::Battle { attacker: crate::events::Attacker::City { city }, .. } if *city == c)).collect();
        assert_eq!(shot.len(), 1);
        let Event::Battle {
            damage_to_defender,
            attacker_strength,
            defender_strength,
            attacker_roll,
            damage_to_attacker,
            ..
        } = shot[0]
        else {
            unreachable!()
        };
        assert_eq!(
            *damage_to_defender,
            combat::damage(
                attacker_strength.total,
                defender_strength.total,
                *attacker_roll
            )
        );
        assert_eq!(*damage_to_attacker, 0);
        assert_eq!(
            serde_json::to_value(shot[0]).unwrap()["attacker_at"],
            serde_json::to_value(hex(3, 3)).unwrap(),
            "a city's shot comes from the city"
        );
        assert_eq!(
            serde_json::to_value(shot[0]).unwrap()["attacker"],
            serde_json::json!({"city": c.0})
        );
        assert_eq!(s.units[&tie].xp, 0);
    }

    #[test]
    fn city_strike_ignores_peace_civilians_and_out_of_range() {
        let mut s = game(&[0, 1]);
        walled_city(&mut s, 0, hex(3, 3), 2);
        let soldier = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        unit(&mut s, 1, UnitKind::Settler, hex(3, 4));
        let mut events = Vec::new();
        resolve_ranged(&mut s, &[], &mut events);
        assert_eq!(
            events.len(),
            1,
            "on by default: the city fires at the Soldier"
        );
        events.clear();
        s.units.get_mut(&soldier).unwrap().hp = 100;
        s.units.get_mut(&soldier).unwrap().pos = hex(6, 3);
        resolve_ranged(&mut s, &[], &mut events);
        assert!(events.is_empty());
        s.units.get_mut(&soldier).unwrap().pos = hex(5, 3);
        for r in &mut s.diplomacy.as_mut().unwrap().relations {
            r.state = crate::diplomacy::RelationState::Peace;
        }
        resolve_ranged(&mut s, &[], &mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn walled_city_strikes_visible_units_over_forest_and_hills() {
        for blocker in [Terrain::Forest, Terrain::Hills] {
            let mut s = game(&[0, 1]);
            let city = walled_city(&mut s, 0, hex(3, 3), 2);
            let soldier = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
            s.map.get_mut(hex(4, 3)).unwrap().terrain = blocker;
            // Remove the normal city-border sight ring so only native sight governs this case.
            for h in hex(3, 3).neighbors() {
                s.map.get_mut(h).unwrap().owner = None;
            }
            assert!(
                !visibility::visible_tiles(&s, s.team_of(PlayerId(0)))
                    .get(s.map.index(hex(5, 3)).unwrap()),
                "fog still hides the target behind {blocker:?}"
            );
            let mut hidden_events = Vec::new();
            resolve_ranged(&mut s, &[], &mut hidden_events);
            assert!(hidden_events.is_empty(), "a fogged unit cannot be struck");
            unit(&mut s, 0, UnitKind::Settler, hex(5, 4));
            assert!(
                visibility::visible_tiles(&s, s.team_of(PlayerId(0)))
                    .get(s.map.index(hex(5, 3)).unwrap())
            );
            let hp = s.units[&soldier].hp;
            let mut events = Vec::new();
            resolve_ranged(&mut s, &[], &mut events);
            assert!(
                s.units[&soldier].hp < hp,
                "{blocker:?} must not block city fire"
            );
            assert!(events.iter().any(|e| matches!(e, Event::Battle {
                attacker: Attacker::City { city: firing },
                defender: Defender::Unit { unit: target, .. }, ..
            } if *firing == city && *target == soldier)));
        }
    }

    #[test]
    fn a_city_at_zero_hp_does_not_fire_but_one_hp_does() {
        let mut s = game(&[0, 1]);
        let c = walled_city(&mut s, 0, hex(3, 3), 2);
        let soldier = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        s.cities.get_mut(&c).unwrap().hp = 0;
        let mut events = Vec::new();
        resolve_ranged(&mut s, &[], &mut events);
        assert!(events.is_empty());
        assert_eq!(s.units[&soldier].hp, 100);
        s.cities.get_mut(&c).unwrap().hp = 1;
        resolve_ranged(&mut s, &[], &mut events);
        assert_eq!(events.len(), 1);
        assert!(s.units[&soldier].hp < 100);
    }

    #[test]
    fn only_walled_cities_fire_with_citadel_and_customs_bastion_counting() {
        // Civ 6: an unwalled city relies on its garrison and HP; Walls (any tier) let it fire.
        let mut s = game(&[0, 1]);
        s.players[0].faction = Faction::India;
        let c = city(&mut s, 0, hex(3, 3), 2);
        let soldier = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        let mut events = Vec::new();
        resolve_ranged(&mut s, &[], &mut events);
        assert!(events.is_empty(), "an unwalled city cannot fire");
        assert_eq!(s.units[&soldier].hp, 100);
        for (faction, walls) in [
            (Faction::India, BuildingKind::Walls),
            (Faction::India, BuildingKind::Castle),
            (Faction::Switzerland, BuildingKind::Walls),
        ] {
            let mut t = s.clone();
            t.players[0].faction = faction;
            t.cities.get_mut(&c).unwrap().buildings = [walls].into_iter().collect();
            let mut events = Vec::new();
            resolve_ranged(&mut t, &[], &mut events);
            assert_eq!(events.len(), 1, "{faction:?} {walls:?} fires");
            assert!(t.units[&soldier].hp < 100);
        }
    }

    #[test]
    fn a_city_damaged_this_turn_does_not_heal_until_a_quiet_turn_and_the_turn_survives_save_load() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 0, hex(3, 3), 2);
        city(&mut s, 1, hex(9, 7), 2);
        let ranger = unit(&mut s, 1, UnitKind::Archer, hex(5, 3));
        s.cities.get_mut(&c).unwrap().hp = 100;
        let (mut next, events) = crate::resolve_turn(
            &s,
            &orders(
                1,
                vec![Order::RangedAttack {
                    unit: ranger,
                    target: hex(3, 3),
                }],
            ),
        );
        let damage = events
            .iter()
            .find_map(|e| match e {
                Event::Battle {
                    damage_to_defender, ..
                } => Some(*damage_to_defender),
                _ => None,
            })
            .unwrap();
        assert!(damage > 0);
        // Damaged this turn: no heal at the end of it, even though the Archer is two tiles off.
        assert_eq!(next.cities[&c].hp, 100 - damage);
        assert_eq!(next.cities[&c].last_attacked_turn, Some(1));
        next = serde_json::from_str(&serde_json::to_string(&next).unwrap()).unwrap();
        // The next turn is quiet: the full heal arrives.
        let (next, _) = crate::resolve_turn(&next, &Default::default());
        assert_eq!(next.cities[&c].hp, 100 - damage + data::CITY_HEAL);
        // A city never attacked carries no attack memory in its save.
        let (quiet, _) = crate::resolve_turn(&s, &Default::default());
        assert_eq!(quiet.cities[&c].hp, 100 + data::CITY_HEAL);
        assert!(
            !serde_json::to_string(&quiet)
                .unwrap()
                .contains("last_attacked_turn")
        );
    }

    fn battles(events: &[Event]) -> Vec<&Event> {
        events
            .iter()
            .filter(|e| matches!(e, Event::Battle { .. }))
            .collect()
    }

    #[test]
    fn ranged_attacks_are_simultaneous_and_unanswered() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Archer, hex(3, 3));
        let b = unit(&mut s, 1, UnitKind::Archer, hex(5, 3));
        let mut os = orders(
            0,
            vec![Order::RangedAttack {
                unit: a,
                target: hex(5, 3),
            }],
        );
        os.insert(
            PlayerId(1),
            vec![Order::RangedAttack {
                unit: b,
                target: hex(3, 3),
            }],
        );
        let (next, events) = resolve_units(&s, &os);
        let bs = battles(&events);
        assert_eq!(bs.len(), 2);
        for e in bs {
            let Event::Battle {
                attacker,
                attacker_at,
                ranged,
                damage_to_attacker,
                attacker_strength,
                defender_strength,
                attacker_roll,
                damage_to_defender,
                ..
            } = e
            else {
                unreachable!()
            };
            assert!(*ranged);
            // "From where?": each shot names the shooter's tile.
            let shooter = if *attacker == a { hex(3, 3) } else { hex(5, 3) };
            assert_eq!(*attacker_at, Some(shooter));
            assert_eq!(*damage_to_attacker, 0, "no retaliation");
            // Both computed from full-HP snapshot strengths.
            assert_eq!(attacker_strength.total, 15);
            assert_eq!(defender_strength.total, 8);
            assert_eq!(*damage_to_defender, combat::damage(15, 8, *attacker_roll));
        }
        let (ra, _) = rolls(&s, STAGE_RANGED, a, Target::Unit(b));
        let (rb, _) = rolls(&s, STAGE_RANGED, b, Target::Unit(a));
        // Healing does not apply: both fought.
        assert_eq!(next.units[&b].hp, 100 - combat::damage(15, 8, ra));
        assert_eq!(next.units[&a].hp, 100 - combat::damage(15, 8, rb));
        assert_eq!(next.units[&a].moves_left, 0);
    }

    #[test]
    fn ranged_attack_on_empty_tile_fails_at_resolution() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Archer, hex(3, 3));
        let os = orders(
            0,
            vec![Order::RangedAttack {
                unit: a,
                target: hex(4, 3),
            }],
        );
        let (_, events) = resolve_units(&s, &os);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
        assert!(battles(&events).is_empty());
    }

    fn shoot(s: &GameState, shooters: &[UnitId], at: Hex) -> (GameState, Vec<Event>) {
        let os = orders(
            0,
            shooters
                .iter()
                .map(|&unit| Order::RangedAttack { unit, target: at })
                .collect(),
        );
        resolve_units(s, &os)
    }

    #[test]
    fn ranged_fire_at_a_city_with_hp_hits_the_city_and_spares_its_garrison() {
        // The city absorbs a shot within its HP; only damage past 0 HP reaches the garrison.
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Archer, hex(3, 3));
        let c = city(&mut s, 1, hex(5, 3), 2);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        let (next, events) = shoot(&s, &[a], hex(5, 3));
        assert_eq!(next.units[&g].hp, 100);
        assert!(next.cities[&c].hp < data::CITY_HP);
        let fight = plan_fight(&s, STAGE_RANGED, a, Target::City(c), FightKind::Ranged);
        assert_eq!(
            next.cities[&c].hp,
            data::CITY_HP - fight.damage_to_defender,
            "the city's defense (not the Soldier's) decides the damage"
        );
        assert!(events.iter().any(|e| matches!(e,
            Event::Battle { defender: Defender::City { city, .. }, .. } if *city == c)));
        assert_eq!(next.units[&g].xp, 0, "the garrison did not fight");

        // At 0 HP the garrison is exposed: the shot hits it and the city stays at 0.
        s.cities.get_mut(&c).unwrap().hp = 0;
        let (next, _) = shoot(&s, &[a], hex(5, 3));
        assert!(next.units[&g].hp < 100);
        assert_eq!(next.cities[&c].hp, 0);
        assert_eq!(
            next.cities[&c].owner,
            PlayerId(1),
            "ranged fire cannot capture"
        );
    }

    #[test]
    fn a_volley_that_breaks_the_city_damages_its_garrison_without_capturing() {
        let mut s = game(&[0, 1]);
        let shooters: Vec<_> = [hex(3, 3), hex(4, 3), hex(6, 3)]
            .into_iter()
            .map(|h| unit(&mut s, 0, UnitKind::Archer, h))
            .collect();
        let c = city(&mut s, 1, hex(5, 3), 2);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        s.cities.get_mut(&c).unwrap().hp = 10;
        let (next, events) = shoot(&s, &shooters, hex(5, 3));
        assert_eq!(next.cities[&c].hp, 0);
        assert!(next.units[&g].hp < 100);
        assert_eq!(next.cities[&c].owner, PlayerId(1));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(
                    e,
                    Event::Battle {
                        defender: Defender::City { .. },
                        ..
                    }
                ))
                .count(),
            3
        );
    }

    #[test]
    fn partial_ranged_overflow_wounds_garrison_without_capture_or_melee() {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Archer, hex(3, 3));
        let c = city(&mut s, 1, hex(5, 3), 2);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        s.cities.get_mut(&c).unwrap().hp = 1;
        let (next, events) = shoot(&s, &[a], hex(5, 3));
        assert_eq!(next.cities[&c].hp, 0);
        assert!(next.units[&g].hp > 0 && next.units[&g].hp < 100);
        assert_eq!(next.cities[&c].owner, PlayerId(1));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CityCaptured { .. }))
        );
    }

    #[test]
    fn melee_counterattack_is_split_once_with_remainder_to_strongest() {
        let mut s = game(&[0, 1]);
        let defender = unit(&mut s, 1, UnitKind::Spearman, hex(5, 5));
        let attackers: Vec<_> = [hex(4, 5), hex(5, 4), hex(6, 5)]
            .into_iter()
            .map(|p| unit(&mut s, 0, UnitKind::Spearman, p))
            .collect();
        let mut fights: Vec<_> = attackers
            .iter()
            .map(|&a| plan_fight(&s, 1, a, Target::Unit(defender), FightKind::Melee))
            .collect();
        for f in &mut fights {
            f.damage_to_attacker = 21;
            f.damage_to_defender = 1;
        }
        fights[1].attacker_strength.total += 1;
        let mut events = Vec::new();
        apply_fights(&mut s, &fights, &mut events);
        assert_eq!(
            attackers.iter().map(|a| s.units[a].hp).collect::<Vec<_>>(),
            vec![93, 93, 93]
        );
        assert_eq!(s.units[&defender].hp, 97);

        let mut s = game(&[0, 1]);
        let defender = unit(&mut s, 1, UnitKind::Spearman, hex(5, 5));
        let attackers: Vec<_> = [hex(4, 5), hex(5, 4), hex(6, 5)]
            .into_iter()
            .map(|p| unit(&mut s, 0, UnitKind::Spearman, p))
            .collect();
        let mut fights: Vec<_> = attackers
            .iter()
            .map(|&a| plan_fight(&s, 1, a, Target::Unit(defender), FightKind::Melee))
            .collect();
        for f in &mut fights {
            f.damage_to_attacker = 22;
            f.damage_to_defender = 1;
        }
        fights[1].attacker_strength.total += 1;
        let mut events = Vec::new();
        apply_fights(&mut s, &fights, &mut events);
        assert_eq!(
            attackers.iter().map(|a| s.units[a].hp).collect::<Vec<_>>(),
            vec![93, 92, 93]
        );
        let later = plan_fight(
            &s,
            2,
            attackers[0],
            Target::Unit(defender),
            FightKind::Melee,
        );
        apply_fights(&mut s, &[later], &mut events);
        assert_eq!(
            s.units[&attackers[0]].hp, 93,
            "later stage gets no retaliation"
        );
    }

    #[test]
    fn city_counterattack_splits_and_single_attacker_is_unchanged() {
        let mut s = game(&[0, 1]);
        let city = city(&mut s, 1, hex(5, 5), 2);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 5));
        let b = unit(&mut s, 0, UnitKind::Spearman, hex(5, 4));
        let mut fights = vec![
            plan_fight(&s, 1, a, Target::City(city), FightKind::Melee),
            plan_fight(&s, 1, b, Target::City(city), FightKind::Melee),
        ];
        for f in &mut fights {
            f.damage_to_attacker = 21;
            f.damage_to_defender = 1;
        }
        let mut events = Vec::new();
        apply_fights(&mut s, &fights, &mut events);
        // Equal strength: the lower id is the "strongest" and takes the remainder (10 + 1).
        assert_eq!((s.units[&a].hp, s.units[&b].hp), (89, 90));
        let mut s = game(&[0, 1]);
        let d = unit(&mut s, 1, UnitKind::Spearman, hex(5, 5));
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 5));
        let mut fight = plan_fight(&s, 1, a, Target::Unit(d), FightKind::Melee);
        fight.damage_to_attacker = 21;
        apply_fights(&mut s, &[fight], &mut Vec::new());
        assert_eq!(s.units[&a].hp, 79);
    }

    #[test]
    fn ranged_attackers_neither_share_nor_dilute_the_counterattack() {
        let mut s = game(&[0, 1]);
        let d = unit(&mut s, 1, UnitKind::Spearman, hex(5, 5));
        let melee: Vec<_> = [hex(4, 5), hex(5, 4), hex(6, 5)]
            .into_iter()
            .map(|p| unit(&mut s, 0, UnitKind::Spearman, p))
            .collect();
        let archer = unit(&mut s, 0, UnitKind::Archer, hex(3, 5));
        let mut fights: Vec<_> = melee
            .iter()
            .map(|&a| plan_fight(&s, 1, a, Target::Unit(d), FightKind::Melee))
            .collect();
        for f in &mut fights {
            f.damage_to_attacker = 21;
            f.damage_to_defender = 1;
        }
        let shot = plan_fight(&s, 1, archer, Target::Unit(d), FightKind::Ranged);
        assert_eq!(shot.damage_to_attacker, 0);
        fights.push(shot);
        apply_fights(&mut s, &fights, &mut Vec::new());
        assert_eq!(
            melee.iter().map(|a| s.units[a].hp).collect::<Vec<_>>(),
            vec![93, 93, 93],
            "21 split three ways; the shot is not a fourth share"
        );
        assert_eq!(s.units[&archer].hp, 100);
    }

    /// The user's game (2026-10-04): three Oathbearers attacked one nearly dead Spearman in the
    /// same movement round, and each took the Spearman's full counterattack. Now they share one,
    /// and the Spearman is destroyed once.
    #[test]
    fn three_oathbearers_share_a_dying_soldiers_one_counterattack() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let d = unit(&mut s, 1, UnitKind::Spearman, hex(5, 5));
        s.units.get_mut(&d).unwrap().hp = 6;
        let attackers: Vec<_> = [hex(4, 5), hex(5, 4), hex(6, 5)]
            .into_iter()
            .map(|p| unit(&mut s, 0, UnitKind::Spearman, p))
            .collect();
        let planned: Vec<_> = attackers
            .iter()
            .map(|&a| plan_fight(&s, 1, a, Target::Unit(d), FightKind::Melee))
            .collect();
        let strongest = planned
            .iter()
            .max_by_key(|f| (f.attacker_strength.total, std::cmp::Reverse(f.attacker)))
            .unwrap();
        let full = strongest.damage_to_attacker;
        let strongest = strongest.attacker;
        let mut results = Vec::new();
        for order in [[0, 1, 2], [2, 1, 0], [1, 2, 0]] {
            let os = orders(
                0,
                order
                    .iter()
                    .map(|&i| Order::MoveUnit {
                        unit: attackers[i],
                        to: hex(5, 5),
                    })
                    .collect(),
            );
            let (next, events) = resolve_units(&s, &os);
            let taken: Vec<_> = attackers
                .iter()
                .map(|a| {
                    events
                        .iter()
                        .find_map(|e| match e {
                            Event::Battle {
                                attacker: crate::events::Attacker::Unit(u),
                                damage_to_attacker,
                                ..
                            } if u == a => Some(*damage_to_attacker),
                            _ => None,
                        })
                        .unwrap()
                })
                .collect();
            assert_eq!(
                taken.iter().sum::<i32>(),
                full,
                "one counterattack in total"
            );
            for (a, t) in attackers.iter().zip(&taken) {
                let share = full / 3 + if *a == strongest { full % 3 } else { 0 };
                assert_eq!(*t, share);
            }
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::UnitDestroyed { unit, .. } if *unit == d))
                    .count(),
                1
            );
            results.push(next.hash());
        }
        assert!(
            results.windows(2).all(|w| w[0] == w[1]),
            "order-independent"
        );
    }

    #[test]
    fn a_volley_that_kills_the_garrison_without_melee_leaves_the_city_uncaptured() {
        let mut s = game(&[0, 1]);
        let shooters: Vec<_> = [hex(3, 3), hex(4, 3), hex(6, 3)]
            .into_iter()
            .map(|h| unit(&mut s, 0, UnitKind::Archer, h))
            .collect();
        let c = city(&mut s, 1, hex(5, 3), 2);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 3));
        s.cities.get_mut(&c).unwrap().hp = 5;
        s.units.get_mut(&g).unwrap().hp = 5;
        let (next, events) = shoot(&s, &shooters, hex(5, 3));
        assert_eq!(next.cities[&c].hp, 0);
        assert!(!next.units.contains_key(&g), "overflow kills the garrison");
        assert_eq!(
            next.cities[&c].owner,
            PlayerId(1),
            "ranged fire never captures"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CityCaptured { .. }))
        );
    }

    #[test]
    fn combined_melee_assault_captures_after_overflow_kills_garrison() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 5), 2);
        let g = unit(&mut s, 1, UnitKind::Spearman, hex(5, 5));
        s.cities.get_mut(&c).unwrap().hp = 1;
        s.units.get_mut(&g).unwrap().hp = 1;
        let attackers: Vec<_> = [hex(4, 5), hex(5, 4), hex(6, 5)]
            .into_iter()
            .map(|p| unit(&mut s, 0, UnitKind::Spearman, p))
            .collect();
        for &a in &attackers {
            goto(&mut s, a, hex(5, 5));
        }
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(next.cities[&c].owner, PlayerId(0));
        assert!(!next.units.contains_key(&g));
        assert_eq!(
            next.units
                .values()
                .filter(|u| u.owner == PlayerId(0) && u.pos == hex(5, 5))
                .count(),
            1
        );
        assert_eq!(events.iter().filter(|e| matches!(e, Event::Battle { defender: Defender::City { city, .. }, .. } if *city == c)).count(), 3);
    }

    #[test]
    fn experimental_plunder_requires_unwalled_paid_market_and_cannot_repeat() {
        for enabled in [false, true] {
            for walls in [false, true] {
                let mut s = game(&[0, 1]);
                s.setup.rule_experiments.market_plunder = enabled;
                let c = city(&mut s, 1, hex(5, 5), 3);
                s.cities
                    .get_mut(&c)
                    .unwrap()
                    .buildings
                    .insert(BuildingKind::Market);
                if walls {
                    s.cities
                        .get_mut(&c)
                        .unwrap()
                        .buildings
                        .insert(BuildingKind::Walls);
                }
                let mut events = Vec::new();
                capture_city(&mut s, c, PlayerId(0), &mut events);
                let loot = if enabled && !walls { 120 } else { 0 };
                assert_eq!(s.player(PlayerId(0)).gold, loot);
                assert_eq!(
                    s.cities[&c].buildings.contains(&BuildingKind::Market),
                    loot == 0
                );
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| matches!(e, Event::BuildingDismantled { .. }))
                        .count(),
                    usize::from(loot > 0)
                );
                capture_city(&mut s, c, PlayerId(1), &mut events);
                // The sole captured city holds the entire Swiss treasury.
                assert_eq!(s.player(PlayerId(1)).gold, loot);
            }
        }
    }

    #[test]
    fn a_captured_city_loses_half_its_population_rounded_down_never_below_one() {
        for (before, after) in [(1, 1), (2, 1), (3, 1), (7, 3), (8, 4)] {
            let mut s = game(&[0, 1]);
            let c = city(&mut s, 1, hex(5, 5), before);
            capture_city(&mut s, c, PlayerId(0), &mut Vec::new());
            assert_eq!(s.cities[&c].population, after, "size {before}");
        }
    }

    #[test]
    fn a_city_returning_to_its_founder_keeps_its_population() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 5), 8);
        capture_city(&mut s, c, PlayerId(0), &mut Vec::new());
        assert_eq!(s.cities[&c].population, 4);
        s.cities.get_mut(&c).unwrap().population = 6;
        capture_city(&mut s, c, PlayerId(1), &mut Vec::new());
        assert_eq!(s.cities[&c].owner, PlayerId(1));
        assert_eq!(s.cities[&c].population, 6, "the founder's people stay");
    }

    #[test]
    fn capture_city_changes_hands() {
        let mut s = game(&[0, 1]);
        let c = city(&mut s, 1, hex(5, 5), 3);
        let civ = unit(&mut s, 1, UnitKind::Settler, hex(5, 5));
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        let city = &s.cities[&c];
        assert_eq!(city.owner, PlayerId(0));
        assert_eq!(city.population, 1);
        assert_eq!((city.food_stored, city.production_stored), (0, 0));
        assert!(!city.is_capital);
        assert_eq!(city.resistance, CAPTURE_RESISTANCE_TURNS);
        assert_eq!(city.founder, PlayerId(1));
        for h in hex(5, 5).within(1) {
            assert_eq!(s.map.get(h).unwrap().owner, Some(PlayerId(0)));
        }
        assert!(
            s.units.contains_key(&civ),
            "civilian waits for movement's capture pass"
        );
        assert_eq!(s.units[&civ].owner, PlayerId(1));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::UnitDestroyed { unit, .. } if *unit == civ))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::CityCaptured { .. }))
        );
    }

    #[test]
    fn greece_captive_pays_each_rival_once_and_uses_the_free_tech_prompt() {
        let mut s = game(&[0, 1, 2, 1]);
        s.players[1].faction = Faction::Athens;
        s.players[3].faction = Faction::Athens;
        let c = city(&mut s, 1, hex(5, 5), 3);
        let mut events = Vec::new();
        assert!(greece_captive_available(&s, c, PlayerId(0)));
        assert!(!greece_captive_available(&s, c, PlayerId(3)));
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert_eq!(s.player(PlayerId(0)).free_techs_pending, 1);
        assert_eq!(s.player(PlayerId(0)).free_techs_from, Some(PlayerId(1)));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::AthensCityCaptured { city, from, to }
            if *city == c && *from == PlayerId(1) && *to == PlayerId(0)))
        );
        assert_eq!(
            crate::view::player_view(&s, PlayerId(0))
                .actions
                .free_techs
                .as_ref()
                .map(|f| f.pending),
            Some(1)
        );
        crate::rules::actions::apply_actions(
            &mut s,
            PlayerId(0),
            &[Order::Action {
                action: crate::orders::Action::ChooseFreeTechs {
                    techs: vec![TechId::Irrigation],
                },
            }],
            &mut events,
        );
        assert!(s.player(PlayerId(0)).techs.contains(&TechId::Irrigation));
        assert_eq!(s.player(PlayerId(0)).free_techs_pending, 0);
        assert!(
            events.iter().any(
                |e| matches!(e, Event::FreeTechsChosen { player, .. } if *player == PlayerId(0))
            )
        );
        capture_city(&mut s, c, PlayerId(1), &mut events);
        assert_eq!(s.player(PlayerId(1)).free_techs_pending, 0);
        assert!(!greece_captive_available(&s, c, PlayerId(0)));
        assert!(greece_captive_available(&s, c, PlayerId(2)));
        events.clear();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::AthensCityCaptured { .. }))
        );
        capture_city(&mut s, c, PlayerId(3), &mut events);
        assert_eq!(s.player(PlayerId(3)).free_techs_pending, 0);
        capture_city(&mut s, c, PlayerId(1), &mut events);
        capture_city(&mut s, c, PlayerId(2), &mut events);
        assert_eq!(s.player(PlayerId(2)).free_techs_pending, 1);
        assert!(s.cities[&c].greece_captive_paid.contains(&PlayerId(0)));
        assert!(s.cities[&c].greece_captive_paid.contains(&PlayerId(2)));
        assert_eq!(
            serde_json::from_str::<GameState>(&serde_json::to_string(&s).unwrap())
                .unwrap()
                .cities[&c]
                .greece_captive_paid,
            s.cities[&c].greece_captive_paid
        );
    }

    #[test]
    fn greece_captive_ablation_gives_no_technology() {
        let mut s = game(&[0, 1]);
        s.players[1].faction = Faction::Athens;
        s.setup
            .rule_experiments
            .faction_rules_off
            .insert(FactionRule::AthensGreeceCaptive);
        let c = city(&mut s, 1, hex(5, 5), 3);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert_eq!(s.player(PlayerId(0)).free_techs_pending, 0);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::AthensCityCaptured { .. }))
        );
    }

    #[test]
    fn celtic_sack_of_athenian_city_still_grants_a_free_technology() {
        let mut s = game(&[0, 1]);
        s.players[0].faction = Faction::Celts;
        s.players[1].faction = Faction::Athens;
        let c = city(&mut s, 1, hex(5, 5), 3);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert!(!s.cities.contains_key(&c));
        assert_eq!(s.player(PlayerId(0)).free_techs_pending, 1);
        assert!(events.windows(2).any(|pair| matches!(pair,
            [Event::CityRazed { city, ordered: false, .. }, Event::AthensCityCaptured { city: tech_city, .. }]
                if *city == c && *tech_city == c)));
    }

    fn holds_capital_point(s: &GameState, p: PlayerId, capital: CityId) -> bool {
        crate::scoring::vp(s, p).contains(&(crate::VpSource::CapitalTaken { capital }, 1))
    }

    #[test]
    fn an_original_capital_point_is_held_by_whoever_owns_it_now() {
        let mut s = game(&[0, 1, 2]);
        let capital = city(&mut s, 1, hex(5, 5), 3);
        let mut events = Vec::new();
        capture_city(&mut s, capital, PlayerId(0), &mut events);
        // Scored by holding it, not by the capture: end-of-turn scoring reports the change.
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CapitalTaken { .. }))
        );
        assert!(holds_capital_point(&s, PlayerId(0), capital));
        // A third empire takes it: the point moves with the city.
        capture_city(&mut s, capital, PlayerId(2), &mut events);
        assert!(!holds_capital_point(&s, PlayerId(0), capital));
        assert!(holds_capital_point(&s, PlayerId(2), capital));
        // The founder takes it back: nobody scores it.
        capture_city(&mut s, capital, PlayerId(1), &mut events);
        assert!(
            (0..3).all(|p| !holds_capital_point(&s, PlayerId(p), capital)),
            "the founder never scores its own capital"
        );
        // Taking it again scores again: the point is held, not once per captor.
        capture_city(&mut s, capital, PlayerId(0), &mut events);
        assert!(holds_capital_point(&s, PlayerId(0), capital));
        let loaded: GameState = serde_json::from_value(serde_json::to_value(&s).unwrap()).unwrap();
        assert!(holds_capital_point(&loaded, PlayerId(0), capital));
    }

    #[test]
    fn founders_teammate_does_not_score_for_retaking_an_allied_original_capital() {
        // Players 0 and 2 share a team; player 1 is a rival of both.
        let mut s = game(&[0, 1, 0]);
        let capital = city(&mut s, 0, hex(5, 5), 3);
        let mut events = Vec::new();
        capture_city(&mut s, capital, PlayerId(1), &mut events);
        assert!(holds_capital_point(&s, PlayerId(1), capital));
        // The founder's teammate takes it back from the rival: nothing is scored, and the rival's
        // point is gone.
        capture_city(&mut s, capital, PlayerId(2), &mut events);
        assert!(!holds_capital_point(&s, PlayerId(2), capital));
        assert!(!holds_capital_point(&s, PlayerId(1), capital));
    }

    #[test]
    fn replacement_capital_does_not_award_original_capital_vp() {
        let mut s = game(&[0, 1]);
        let original = city(&mut s, 1, hex(5, 5), 3);
        let replacement = city(&mut s, 1, hex(8, 5), 3);
        s.cities.get_mut(&replacement).unwrap().original_capital = false;
        assert!(s.cities[&original].original_capital);
        assert!(!s.cities[&replacement].original_capital);
        s.cities.get_mut(&replacement).unwrap().is_capital = true;
        let mut events = Vec::new();
        capture_city(&mut s, replacement, PlayerId(0), &mut events);
        assert!(crate::scoring::capitals_held(&s, PlayerId(0)).is_empty());
        assert!(!holds_capital_point(&s, PlayerId(0), replacement));
    }

    #[test]
    fn celtic_sack_of_a_rival_original_capital_keeps_its_point_for_good() {
        let mut s = game(&[0, 1]);
        let capital = city(&mut s, 1, hex(5, 5), 3);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        let mut events = Vec::new();
        capture_city(&mut s, capital, PlayerId(0), &mut events);
        assert!(!s.cities.contains_key(&capital));
        assert!(events.iter().any(
            |e| matches!(e, Event::CityRazed { city, original_capital: true, ordered: false, .. } if *city == capital)
        ));
        assert!(events.iter().any(
            |e| matches!(e, Event::CapitalSacked { captor: PlayerId(0), capital: c, .. } if *c == capital)
        ));
        // The city is gone, but the Celts keep its point: nobody can take it back.
        let sacked = (crate::VpSource::CapitalSacked { capital }, 1);
        assert!(crate::scoring::vp(&s, PlayerId(0)).contains(&sacked));
        let (next, _) = crate::resolve_turn(&s, &BTreeMap::new());
        assert!(crate::scoring::vp(&next, PlayerId(0)).contains(&sacked));
        assert!(!holds_capital_point(&next, PlayerId(0), capital));
    }

    #[test]
    fn celtic_sack_of_a_teammates_capital_scores_nothing() {
        let mut s = game(&[0, 0, 1]);
        let capital = city(&mut s, 1, hex(5, 5), 3);
        s.cities.get_mut(&capital).unwrap().owner = PlayerId(2);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        let mut events = Vec::new();
        capture_city(&mut s, capital, PlayerId(0), &mut events);
        assert!(!s.cities.contains_key(&capital), "the sack still burns it");
        assert!(s.player(PlayerId(0)).capitals_sacked.is_empty());
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CapitalSacked { .. }))
        );
    }

    fn razed(events: &[Event]) -> Event {
        events
            .iter()
            .find(|e| matches!(e, Event::CityRazed { .. }))
            .cloned()
            .expect("the city was razed")
    }

    #[test]
    fn celtic_sack_takes_sixty_gold_per_population_and_nothing_else() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        s.player_mut(PlayerId(0)).gold = 17;
        s.player_mut(PlayerId(1)).gold = 103;
        let c = city(&mut s, 1, hex(5, 5), 4);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        // The victim's treasury plays no part: the Celts gain exactly what the forecast showed
        // and the victim loses no Gold.
        assert_eq!(s.player(PlayerId(0)).gold, 17 + 4 * 60);
        assert_eq!(s.player(PlayerId(1)).gold, 103);
        assert!(matches!(
            razed(&events),
            Event::CityRazed {
                sack_gold: Some(240),
                ..
            }
        ));
    }

    #[test]
    fn the_celts_retaking_a_city_they_founded_do_not_sack_it() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        s.player_mut(PlayerId(0)).gold = 17;
        let c = city(&mut s, 0, hex(5, 5), 6);
        capture_city(&mut s, c, PlayerId(1), &mut Vec::new());
        assert_eq!(s.cities[&c].owner, PlayerId(1));
        s.cities.get_mut(&c).unwrap().population = 5;
        let camps_before = s.barbarians.as_ref().map_or(0, |b| b.camps.len());
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert_eq!(s.cities[&c].owner, PlayerId(0), "the city comes home");
        assert_eq!(s.cities[&c].population, 5, "the founder's people stay");
        assert_eq!(s.player(PlayerId(0)).gold, 17, "no sack Gold");
        assert!(!events.iter().any(|e| matches!(e, Event::CityRazed { .. })));
        assert_eq!(
            s.barbarians.as_ref().map_or(0, |b| b.camps.len()),
            camps_before,
            "no camp"
        );
    }

    #[test]
    fn a_sack_changes_no_technology() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        // The victim knows Masonry, Currency and Mathematics; the Celts know only Irrigation.
        for tech in [
            TechId::Irrigation,
            TechId::Masonry,
            TechId::Currency,
            TechId::Mathematics,
        ] {
            s.player_mut(PlayerId(1)).techs.insert(tech);
        }
        s.player_mut(PlayerId(0)).techs.insert(TechId::Irrigation);
        let (celts_before, victim_before) = (
            s.player(PlayerId(0)).techs.clone(),
            s.player(PlayerId(1)).techs.clone(),
        );
        let c = city(&mut s, 1, hex(5, 5), 3);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert_eq!(s.player(PlayerId(0)).techs, celts_before);
        assert_eq!(s.player(PlayerId(1)).techs, victim_before);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::TechResearched { .. }))
        );
    }

    #[test]
    fn a_sack_raises_a_celtic_warband_camp_near_the_ruins() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        let c = city(&mut s, 1, hex(5, 5), 3);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        let Event::CityRazed { camp: Some(at), .. } = razed(&events) else {
            panic!("a camp was raised");
        };
        assert!(at.distance(hex(5, 5)) <= crate::rules::barbarians::SACK_CAMP_RADIUS);
        let camps = &s.barbarians.as_ref().expect("camp state").camps;
        assert_eq!(camps.len(), 1);
        assert_eq!(camps[0].pos, at);
        assert_eq!(camps[0].rite, Some(PlayerId(0)));
        assert!(camps[0].guard.is_some());
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::BarbarianCampSpawned { at: a, .. } if *a == at))
        );
    }

    #[test]
    fn a_sack_camp_in_a_celtic_citys_land_completes_its_warbands() {
        let mut s = game(&[0, 1]);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        let home = city(&mut s, 0, hex(8, 5), 3);
        let victim = city(&mut s, 1, hex(5, 5), 3);
        // The camp prefers a tile of the Celts' own territory that has no camp yet.
        for h in hex(8, 5).within(2) {
            if let Some(t) = s.map.get_mut(h) {
                t.owner = Some(PlayerId(0));
                t.city = Some(home);
            }
        }
        assert_eq!(crate::goals::wild_camp_cities(&s, PlayerId(0)), (0, 1));
        let mut events = Vec::new();
        capture_city(&mut s, victim, PlayerId(0), &mut events);
        assert_eq!(crate::goals::wild_camp_cities(&s, PlayerId(0)), (1, 1));
    }
    fn off(rule: FactionRule) -> crate::experiments::RuleExperiments {
        let mut r = crate::experiments::RuleExperiments::default();
        r.faction_rules_off.insert(rule);
        r
    }

    #[test]
    fn celts_sack_off_burns_the_city_for_no_gold_and_no_camp() {
        let mut s = game(&[0, 1]);
        s.setup.rule_experiments = off(FactionRule::CeltsSack);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        s.player_mut(PlayerId(0)).gold = 17;
        let c = city(&mut s, 1, hex(5, 5), 4);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert!(!s.cities.contains_key(&c), "still burned");
        assert_eq!(s.player(PlayerId(0)).gold, 17);
        assert!(matches!(
            razed(&events),
            Event::CityRazed {
                sack_gold: Some(0),
                camp: None,
                ..
            }
        ));
    }

    #[test]
    fn celts_no_keep_off_keeps_the_city_like_any_captor() {
        let mut s = game(&[0, 1]);
        s.setup.rule_experiments = off(FactionRule::CeltsNoKeep);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        s.player_mut(PlayerId(0)).gold = 17;
        let c = city(&mut s, 1, hex(5, 5), 4);
        let mut events = Vec::new();
        capture_city(&mut s, c, PlayerId(0), &mut events);
        assert_eq!(s.cities[&c].owner, PlayerId(0), "kept");
        assert_eq!(s.player(PlayerId(0)).gold, 17, "no sack gold");
        assert!(!events.iter().any(|e| matches!(e, Event::CityRazed { .. })));
    }

    #[test]
    fn sparta_kill_heal_off_heals_neither_kills_nor_captures() {
        let mut s = game(&[0, 1]);
        s.setup.rule_experiments = off(FactionRule::SpartaKillHeal);
        s.player_mut(PlayerId(0)).faction = Faction::Sparta;
        let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let victim = unit(&mut s, 1, UnitKind::Scout, hex(4, 3));
        s.units.get_mut(&attacker).unwrap().hp = 1;
        s.units.get_mut(&victim).unwrap().hp = 10;
        let mut events = Vec::new();
        let fight = forced_hit(&s, attacker, victim, 10, 0);
        apply_fights(&mut s, &[fight], &mut events);
        assert_eq!(s.units[&attacker].hp, 1, "no kill heal");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SpartaKillHeal { .. }))
        );

        s.player_mut(PlayerId(1)).faction = Faction::India;
        let city_id = city(&mut s, 1, hex(5, 5), 3);
        city(&mut s, 1, hex(9, 5), 3);
        let near = unit(&mut s, 0, UnitKind::Spearman, hex(4, 5));
        s.units.get_mut(&near).unwrap().hp = 20;
        let mut events = Vec::new();
        capture_city(&mut s, city_id, PlayerId(0), &mut events);
        assert_eq!(s.units[&near].hp, 20, "no conquest heal");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SpartaConquest { .. }))
        );
    }

    #[test]
    fn aztecs_flower_wars_off_grows_no_city() {
        let mut s = game(&[0, 1]);
        s.setup.rule_experiments = off(FactionRule::AztecsFlowerWars);
        s.player_mut(PlayerId(0)).faction = Faction::Aztecs;
        let near = city(&mut s, 0, hex(2, 3), 1);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 3));
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(4, 3));
        let fights = [forced_hit(&s, a, victim, 100, 0)];
        let mut events = vec![];
        apply_fights(&mut s, &fights, &mut events);
        assert!(!s.units.contains_key(&victim), "the kill still happens");
        assert_eq!(s.cities[&near].population, 1);
        assert!(!events.iter().any(|e| matches!(e, Event::FlowerWars { .. })));
    }
}
