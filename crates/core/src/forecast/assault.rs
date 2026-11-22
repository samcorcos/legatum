//! The whole fight at an enemy city: the city and its garrison together, with every attack the
//! viewer has queued on the tile this turn (user, 2026-10-19, 8.9.6 T187 at Lugano: the forecast
//! "was only showing me taking down the city/walls not the unit under it"). The same stage rules
//! as resolution (`rules::battle::carried_damage`, `rules::movement`): ranged fire is one stage and
//! the melee round the next; inside a stage the hits go one by one: the city's HP absorbs first,
//! and what gets past it is shared among the garrison and the city's supporters, each share
//! judged against the unit that takes it (`combat::carried_share`). Every defender adds its own
//! counterattack to the melee stage's pooled reply (supported defence, designer 2026-11-15).

use super::{AttackMode, phalanx_neighbour, siege_cover};
use crate::combat;
use crate::los;
use crate::types::*;
use crate::view::{CityView, PlayerView, UnitView};
use serde::{Deserialize, Serialize};

/// Another of the viewer's units already ordered to attack the same tile this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlongsideAttack {
    pub unit: UnitId,
    pub mode: AttackMode,
}

/// The garrison's share of the combined assault.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssaultGarrison {
    pub unit: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub hp: i32,
    /// Expected HP left after this turn's attacks, averaged over the attacker's rolls.
    pub hp_after: i32,
    /// Chance (0-100) the garrison is destroyed.
    pub destroyed_chance: u32,
    /// Expected HP the garrison loses to the carried part of the hit that empties the city's HP
    /// (`combat::carried_share`), averaged over the attacker's rolls; later hits are not in it.
    #[serde(default)]
    pub carried: i32,
}

/// A unit of the city's owner ordered into the city this turn: it shares what gets past the
/// city's HP and adds its own counterattack (supported defence, designer 2026-11-15). The
/// attacker never sees the defender's orders, so these are the supporters the caller assumes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssaultSupporter {
    pub unit: UnitId,
    pub kind: UnitKind,
    pub hp: i32,
    /// Expected HP left after this turn's attacks, averaged over the attacker's rolls.
    pub hp_after: i32,
    /// Chance (0-100) the supporter is destroyed.
    pub destroyed_chance: u32,
}

/// What this attack and the other queued attacks on the city do together.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityAssaultForecast {
    /// Attacks pooled, this one included.
    pub attacks: u32,
    pub city_hp: i32,
    /// Expected city HP left, averaged over the attacker's rolls (the other attacks at a 100% roll).
    pub city_hp_after: i32,
    pub garrison: Option<AssaultGarrison>,
    /// The supporters assumed (empty unless the caller names them).
    #[serde(default)]
    pub supporters: Vec<AssaultSupporter>,
    /// Expected damage this attacker takes from the melee stage's pooled counterattack (the
    /// city's or garrison's reply plus one per supporter, split among the melee attackers);
    /// 0 for a ranged attack.
    #[serde(default)]
    pub reply: i32,
    /// Chance (0-100) the city is captured this turn. 0 when no melee attack is queued.
    pub capture_chance: u32,
    /// A melee attack is among the pooled ones, so a fallen city can be taken.
    pub can_capture: bool,
}

#[derive(Clone)]
struct Hit {
    unit: UnitId,
    hp: i32,
    ranged: bool,
    /// Attack strength against the city's HP (a ranged strike adds its siege bonus).
    to_city: i32,
    /// Attack strength against the garrison once the city has fallen.
    to_garrison: i32,
    /// Class bonus of a garrison defending against this particular attack.
    garrison_defense_bonus: i32,
    /// The city's defence against this attacker (the attacker's Siege changes it).
    city_defense: i32,
    /// Percent of the damage that reaches the city's HP (Walls).
    percent: i32,
    /// Per supporter (in `Defenders::supporters` order): this attack's strength against it and
    /// its defence against this attack, class bonuses included.
    vs_supporters: Vec<(i32, i32)>,
}

/// The defending units: the garrison (HP, defence) and the supporters (unit id, HP).
struct Defenders {
    garrison: Option<(i32, i32)>,
    supporters: Vec<(UnitId, i32)>,
}

fn strengths(view: &PlayerView, u: &UnitView, ranged: bool) -> (i32, i32) {
    let faction = view.faction_of(u.owner);
    let made_by = view.made_by(u);
    let strength = |city| {
        combat::attack_strength_rules(
            view.rule_experiments.faction_rules_off,
            faction,
            made_by,
            u.kind,
            u.hp,
            u.level,
            ranged,
            false,
            phalanx_neighbour(view, u),
            u.unsupplied,
            city,
        )
        .total
    };
    (strength(true), strength(false))
}

/// Class bonuses of attacker `u` against defender `d`: (to the attack, to the defence).
fn class_bonuses(view: &PlayerView, u: &UnitView, d: &UnitView, ranged: bool) -> (i32, i32) {
    let off = view.rule_experiments.faction_rules_off;
    let attack =
        combat::fight_class_bonus_for_rules(off, view.made_by(u), u.kind, d.kind, true, ranged).1
            + combat::fight_reislaufer_bonus_for_rules(
                off,
                view.made_by(u),
                u.kind,
                d.kind,
                true,
                ranged,
            )
            .1;
    let defense =
        combat::fight_class_bonus_for_rules(off, view.made_by(d), d.kind, u.kind, false, ranged).1
            + combat::fight_reislaufer_bonus_for_rules(
                off,
                view.made_by(d),
                d.kind,
                u.kind,
                false,
                ranged,
            )
            .1;
    (attack, defense)
}

/// A unit's defence on the city's tile: its terrain, `fortify`, phalanx and supply.
fn defense_on_city(view: &PlayerView, city: &CityView, u: &UnitView, fortify: i32) -> i32 {
    let terrain = view
        .tile(city.pos)
        .and_then(|t| t.terrain)
        .unwrap_or(Terrain::Plains);
    let made_by = view.made_by(u);
    let mut d = combat::unit_defense_strength_for_rules(
        view.rule_experiments.faction_rules_off,
        view.faction_of(u.owner),
        made_by,
        u.kind,
        u.hp,
        terrain,
        fortify,
        u.level,
    );
    combat::add_phalanx_rules(
        &mut d,
        view.rule_experiments.faction_rules_off,
        made_by,
        u.kind,
        phalanx_neighbour(view, u),
    );
    crate::resources::add_unsupplied(&mut d, made_by, u.kind, false, u.unsupplied);
    d.total
}

fn hit_for(
    view: &PlayerView,
    city: &CityView,
    u: &UnitView,
    ranged: bool,
    supporters: &[&UnitView],
) -> Hit {
    let (to_city, mut to_garrison) = strengths(view, u, ranged);
    let mut garrison_defense_bonus = 0;
    if let Some(garrison) = view
        .units
        .iter()
        .find(|g| g.owner == city.owner && g.pos == city.pos && g.military)
    {
        let (attack, defense) = class_bonuses(view, u, garrison, ranged);
        to_garrison += attack;
        garrison_defense_bonus = defense;
    }
    let (_, base_attack) = strengths(view, u, ranged);
    let vs_supporters = supporters
        .iter()
        .map(|s| {
            let (attack, defense) = class_bonuses(view, u, s, ranged);
            (
                base_attack + attack,
                defense_on_city(view, city, s, 0) + defense,
            )
        })
        .collect();
    let tier = u8::from(city.walls) + city.fort_tier();
    let covered = siege_cover(view, u);
    Hit {
        unit: u.id,
        hp: u.hp,
        ranged,
        to_city,
        to_garrison,
        garrison_defense_bonus,
        city_defense: combat::city_defense_vs(
            view.rule_experiments,
            view.faction_of(city.owner),
            city.population,
            tier,
            covered,
            city.hp,
            view.era_of(city.owner),
        )
        .total,
        percent: combat::wall_damage_percent(tier, covered, false),
        vs_supporters,
    }
}

/// A queued attack that can really be made: an own, ready military unit with the mode's weapon.
fn alongside_hit(
    view: &PlayerView,
    city: &CityView,
    primary: UnitId,
    a: &AlongsideAttack,
    supporters: &[&UnitView],
) -> Option<Hit> {
    let u = view
        .units
        .iter()
        .find(|u| u.id == a.unit && u.owner == view.me && u.military && u.hp > 0)?;
    if u.id == primary || u.pos == city.pos {
        return None;
    }
    match a.mode {
        AttackMode::Ranged => {
            let clear = u.ranged > 0
                && u.pos.distance(city.pos) <= u.range
                && los::ranged_clear_with_tech(
                    u.kind,
                    view.research.known.contains(&crate::TechId::Steel),
                    view.cities
                        .iter()
                        .any(|c| c.owner == u.owner && c.pos == u.pos && c.walls),
                    u.pos,
                    city.pos,
                    |h| view.tile(h).and_then(|t| t.terrain),
                );
            clear.then(|| hit_for(view, city, u, true, supporters))
        }
        AttackMode::Melee => {
            crate::data::attacks_in_melee(u.kind).then(|| hit_for(view, city, u, false, supporters))
        }
    }
}

/// One simulated turn: what is left of each defender and what the primary attacker suffers.
struct Outcome {
    city: i32,
    garrison: Option<i32>,
    supporters: Vec<i32>,
    /// Defending rolls (of 41) that permit capture.
    capture_rolls: i32,
    /// Garrison HP lost to the carried part of the hit that empties the city.
    carried: i32,
    /// The primary attacker's share of the pooled reply, summed over the 41 defending rolls.
    reply: i64,
}

/// Simulate the stages once with the primary attacker's roll `roll` (every other attack at 100%).
fn run(hits: &[Hit], primary: usize, roll: i32, city_hp: i32, defenders: &Defenders) -> Outcome {
    let mut city = city_hp;
    let mut g = defenders.garrison.map(|(hp, _)| hp);
    let mut sup: Vec<i32> = defenders.supporters.iter().map(|(_, hp)| *hp).collect();
    let (mut capture_rolls, mut carried_total, mut reply) = (0, 0, 0i64);
    for ranged in [true, false] {
        let stage: Vec<(usize, &Hit)> = hits
            .iter()
            .enumerate()
            .filter(|(_, h)| h.ranged == ranged)
            .collect();
        if stage.is_empty() {
            continue;
        }
        let roll_of = |i: usize| if i == primary { roll } else { 100 };
        let city_before = city;
        let garrison_alive_before = g.is_some_and(|hp| hp > 0);
        let supporters_before: Vec<bool> = sup.iter().map(|hp| *hp > 0).collect();
        // Hit by hit, in order: the city's HP absorbs first; what gets past it (all of a hit on
        // the garrison or on an emptied city) is shared evenly among the living defenders, the
        // remainder to the garrison, each share judged against the unit that takes it.
        for (i, h) in &stage {
            let r = roll_of(*i);
            let direct_g = defenders.garrison.map(|(_, defense)| {
                combat::damage(h.to_garrison, defense + h.garrison_defense_bonus, r)
            });
            let to_city = || {
                combat::damage_to_city_hp(h.percent, combat::damage(h.to_city, h.city_defense, r))
            };
            let (base, past, empties) = if city > 0 {
                let d = to_city();
                let past = d - city;
                city = (city - d).max(0);
                (d, past, true)
            } else if let Some(d) = direct_g {
                (d, d, false)
            } else {
                let d = to_city();
                (d, d, false)
            };
            if past <= 0 || base <= 0 {
                continue;
            }
            let mut living: Vec<Option<usize>> = Vec::new();
            if g.is_some_and(|hp| hp > 0) {
                living.push(None);
            }
            living.extend((0..sup.len()).filter(|k| sup[*k] > 0).map(Some));
            let n = living.len() as i32;
            for (k, who) in living.into_iter().enumerate() {
                let part = past / n + if k == 0 { past % n } else { 0 };
                match who {
                    None => {
                        let hp = g.unwrap_or(0);
                        let share = combat::carried_share(direct_g.unwrap_or(0), part, base);
                        if empties {
                            carried_total += share.min(hp.max(0));
                        }
                        g = Some(hp - share);
                    }
                    Some(s) => {
                        let (attack, defense) = h.vs_supporters[s];
                        let direct = combat::damage(attack, defense, r);
                        sup[s] -= combat::carried_share(direct, part, base);
                    }
                }
            }
        }
        if ranged {
            continue;
        }
        let strongest = stage.iter().copied().max_by_key(|(_, h)| {
            (
                if city_before > 0 {
                    h.to_city
                } else {
                    h.to_garrison
                },
                std::cmp::Reverse(h.unit),
            )
        });
        let Some((strongest_index, strongest_hit)) = strongest else {
            continue;
        };
        let defense = if city_before > 0 {
            strongest_hit.city_defense
        } else {
            defenders.garrison.map_or(0, |(_, defense)| {
                defense + strongest_hit.garrison_defense_bonus
            })
        };
        let attack = if city_before > 0 {
            strongest_hit.to_city
        } else {
            strongest_hit.to_garrison
        };
        let n = stage.len() as i32;
        let falls = city == 0 && g.is_none_or(|hp| hp <= 0) && sup.iter().all(|hp| *hp <= 0);
        for defense_roll in combat::ROLL_MIN..=combat::ROLL_MAX {
            let mut retaliation = if city_before == 0 && !garrison_alive_before {
                0
            } else {
                combat::damage(defense, attack, defense_roll)
            };
            for (s, alive) in supporters_before.iter().enumerate() {
                if *alive {
                    let (attack, defense) = strongest_hit.vs_supporters[s];
                    retaliation += combat::damage(defense, attack, defense_roll);
                }
            }
            let share_of = |i: usize| {
                retaliation / n
                    + if i == strongest_index {
                        retaliation % n
                    } else {
                        0
                    }
            };
            if stage.iter().any(|(i, _)| *i == primary) {
                reply += i64::from(share_of(primary));
            }
            if falls {
                capture_rolls += i32::from(stage.iter().any(|(i, h)| h.hp > share_of(*i)));
            }
        }
    }
    Outcome {
        city,
        garrison: g.map(|hp| hp.max(0)),
        supporters: sup.into_iter().map(|hp| hp.max(0)).collect(),
        capture_rolls,
        carried: carried_total,
        reply,
    }
}

/// The combined assault on the enemy city at `target` by `attacker` and the `alongside` attacks
/// the viewer has queued on the same tile. None when the single attack has no forecast or the
/// target is not a known enemy city (a garrison alone is not a city).
pub fn forecast_city_assault(
    view: &PlayerView,
    attacker: UnitId,
    target: crate::hex::Hex,
    mode: AttackMode,
    alongside: &[AlongsideAttack],
) -> Option<CityAssaultForecast> {
    forecast_city_assault_supported(view, attacker, target, mode, alongside, &[])
}

/// [`forecast_city_assault`] assuming the city's owner orders `supporters` (its visible
/// non-ranged military units next to the city) into it this turn: they share what gets past
/// the city's HP and each adds its own counterattack, as in resolution. Others are ignored.
pub fn forecast_city_assault_supported(
    view: &PlayerView,
    attacker: UnitId,
    target: crate::hex::Hex,
    mode: AttackMode,
    alongside: &[AlongsideAttack],
    supporters: &[UnitId],
) -> Option<CityAssaultForecast> {
    let forecast = super::forecast_attack_mode(view, attacker, target, mode)?;
    let city = view
        .cities
        .iter()
        .find(|c| c.pos == target && view.at_war(view.me, c.owner))?;
    let att = view.units.iter().find(|u| u.id == attacker)?;
    let garrison = forecast
        .garrison
        .as_ref()
        .and_then(|g| view.units.iter().find(|u| u.id == g.unit));
    let mut backers: Vec<&UnitView> = view
        .units
        .iter()
        .filter(|u| {
            supporters.contains(&u.id)
                && u.owner == city.owner
                && u.military
                && u.hp > 0
                && u.pos.distance(city.pos) == 1
                && crate::data::attacks_in_melee(u.kind)
        })
        .collect();
    backers.sort_by_key(|u| u.id);
    let ranged = forecast.ranged;
    // The primary attacker's hit at a given HP: the city's shot, fired before movement, wounds a
    // melee attacker and so weakens its charge (a ranged strike is planned from the same snapshot).
    let primary_at = |hp: i32| -> Hit {
        let mut u = att.clone();
        u.hp = hp;
        hit_for(view, city, &u, ranged, &backers)
    };
    let mut others = Vec::new();
    let mut seen = vec![att.id];
    for a in alongside {
        if seen.contains(&a.unit) {
            continue;
        }
        if let Some(h) = alongside_hit(view, city, att.id, a, &backers) {
            seen.push(a.unit);
            others.push(h);
        }
    }
    let shot_rolls: Vec<Option<(UnitId, i32)>> = super::city_strike_target(view, city)
        .and_then(|id| {
            let target = view.units.iter().find(|u| u.id == id)?;
            if id != att.id && !others.iter().any(|h| h.unit == id) {
                return None;
            }
            let strike = super::forecast_city_strike(view, city, target, target.pos)?;
            Some(
                (combat::ROLL_MIN..=combat::ROLL_MAX)
                    .map(|roll| Some((id, strike.damage_at_roll(roll).min(target.hp))))
                    .collect(),
            )
        })
        .unwrap_or_else(|| vec![None]);
    let defenders = Defenders {
        garrison: garrison.map(|u| (u.hp, defense_on_city(view, city, u, u.fortify))),
        supporters: backers.iter().map(|u| (u.id, u.hp)).collect(),
    };
    let mut runs = 0i64;
    let (mut city_sum, mut garrison_sum, mut captured, mut destroyed, mut carried_sum) =
        (0i64, 0i64, 0i64, 0i64, 0i64);
    let mut reply_sum = 0i64;
    let mut sup_sum = vec![0i64; backers.len()];
    let mut sup_dead = vec![0i64; backers.len()];
    for shot in &shot_rolls {
        let left = forecast.attacker_hp
            - shot
                .as_ref()
                .filter(|(id, _)| *id == att.id)
                .map_or(0, |(_, damage)| *damage);
        for roll in combat::ROLL_MIN..=combat::ROLL_MAX {
            // A melee attacker the shot destroys never attacks; a ranged strike still lands.
            let mut hits: Vec<Hit> = Vec::new();
            let primary = if left > 0 || ranged {
                hits.push(primary_at(if ranged { forecast.attacker_hp } else { left }));
                0
            } else {
                usize::MAX
            };
            for h in &others {
                let damage = shot
                    .as_ref()
                    .filter(|(id, _)| *id == h.unit)
                    .map_or(0, |(_, damage)| *damage);
                if !h.ranged && h.hp <= damage {
                    continue;
                }
                if damage > 0 && !h.ranged {
                    let mut u = view.units.iter().find(|u| u.id == h.unit).unwrap().clone();
                    u.hp -= damage;
                    hits.push(hit_for(view, city, &u, false, &backers));
                } else {
                    hits.push(h.clone());
                }
            }
            let out = run(&hits, primary, roll, city.hp, &defenders);
            let defender_rolls = i64::from(combat::ROLL_MAX - combat::ROLL_MIN + 1);
            runs += defender_rolls;
            city_sum += i64::from(out.city) * defender_rolls;
            garrison_sum += i64::from(out.garrison.unwrap_or(0)) * defender_rolls;
            captured += i64::from(out.capture_rolls);
            carried_sum += i64::from(out.carried) * defender_rolls;
            destroyed += i64::from(out.garrison.is_some_and(|hp| hp <= 0)) * defender_rolls;
            reply_sum += out.reply;
            for (k, hp) in out.supporters.iter().enumerate() {
                sup_sum[k] += i64::from(*hp) * defender_rolls;
                sup_dead[k] += i64::from(*hp <= 0) * defender_rolls;
            }
        }
    }
    let rolls = runs;
    let attacks = 1 + others.len() as u32;
    let can_capture = !ranged || others.iter().any(|h| !h.ranged);
    Some(CityAssaultForecast {
        attacks,
        city_hp: city.hp,
        city_hp_after: (city_sum / rolls) as i32,
        garrison: garrison
            .zip(defenders.garrison)
            .map(|(u, (hp, _))| AssaultGarrison {
                unit: u.id,
                owner: u.owner,
                kind: u.kind,
                hp,
                hp_after: (garrison_sum / rolls) as i32,
                destroyed_chance: (destroyed * 100 / rolls) as u32,
                carried: (carried_sum / rolls) as i32,
            }),
        supporters: backers
            .iter()
            .enumerate()
            .map(|(k, u)| AssaultSupporter {
                unit: u.id,
                kind: u.kind,
                hp: u.hp,
                hp_after: (sup_sum[k] / rolls) as i32,
                destroyed_chance: (sup_dead[k] * 100 / rolls) as u32,
            })
            .collect(),
        reply: (reply_sum / rolls) as i32,
        capture_chance: (captured * 100 / rolls) as u32,
        can_capture,
    })
}
