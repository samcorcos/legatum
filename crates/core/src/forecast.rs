//! Battle forecasts from a PlayerView, using the same combat math as resolution.

mod assault;
mod cities;

pub use assault::{
    AlongsideAttack, AssaultGarrison, AssaultSupporter, CityAssaultForecast, forecast_city_assault,
    forecast_city_assault_supported,
};
pub(crate) use cities::forecast_cities;
pub use cities::{BankruptcyForecast, DisbandForecast, ProductionStall};

use crate::combat;
use crate::data;
use crate::experiments::FactionRule;
use crate::hex::Hex;
use crate::los;
use crate::types::*;
use crate::view::PlayerView;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleForecast {
    pub attacker_strength: Breakdown,
    pub defender_strength: Breakdown,
    /// Chance (0-100) the defender is destroyed (or a city drops to 0 HP) in this attack.
    pub win_chance: u32,
    /// Chance (0-100) the attacker is destroyed.
    pub loss_chance: u32,
    pub expected_damage_to_attacker: i32,
    pub expected_damage_to_defender: i32,
    pub attacker_hp: i32,
    pub defender_hp: i32,
    /// A ranged attack: the attacker takes no damage.
    pub ranged: bool,
    /// Cities fire back: the city being attacked will shoot this turn if it has Walls and the
    /// attacker starts within two tiles of it. Omitted when it can't (no Walls, out of range, the
    /// city is at 0 HP, or city strike is off).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city_strike: Option<CityStrikeForecast>,
    /// The known garrison of the attacked enemy city (`combat::city_tile_defender`). While the
    /// city has HP it absorbs the attack and the garrison is safe (`shielded`); at 0 HP the
    /// garrison is the defender above, and a melee win against it captures the city. Omitted
    /// when the target is not a city or no garrison is visible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub garrison: Option<GarrisonForecast>,
    /// Gold a Celtic attacker takes by capturing this city: 60 per population, nothing else
    /// (`economy::celts_sack_gold`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sack_gold: Option<i32>,
    /// What the city's Walls do to this attack. Omitted when the target is not a walled city.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walls: Option<WallsForecast>,
    /// The attacker's own share of its normal damage to the city's HP, before Walls, when it is
    /// not all of it (`data::city_hp_damage_percent`: a Biplane deals half). Omitted otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city_hp_percent: Option<i32>,
    /// An air strike on a city with an Air Defence: the strike will be met and will not land
    /// (`rules::air`). Omitted for every other attack, and for a Stealth Bomber, which passes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air_defence: Option<AirDefenceForecast>,
}

/// What an Air Defence does to one air strike on its city: it takes a Fighter's place in the duel
/// (`rules::air::AIR_DEFENCE_INTERCEPTOR`), so the strike does not land and the aircraft takes
/// the duel's damage. The Air Defence itself takes none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirDefenceForecast {
    /// The interceptor's strength (a Jet Fighter's, standing in the city).
    pub interceptor_strength: i32,
    /// The striking aircraft's strength in the duel.
    pub striker_strength: i32,
    /// Average damage the duel does to the striking aircraft.
    pub expected_damage_to_attacker: i32,
    /// Chance (0-100) the duel destroys the striking aircraft.
    pub loss_chance: u32,
}

impl BattleForecast {
    /// Percent of its normal damage this attack deals to the city's HP: Walls times the
    /// attacker's own share (`combat::city_damage_percent`).
    pub fn city_percent(&self) -> i32 {
        self.walls.map_or(100, |w| w.percent) * self.city_hp_percent.unwrap_or(100) / 100
    }
}

/// What Walls do to one attack on a city's HP (`combat::wall_damage_percent`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallsForecast {
    /// The city's walls: 1 Walls, 2 a Castle, 3 a Star Fort.
    pub tier: u8,
    /// The siege tier of the attacker (`data::siege_cover`), 0 when it has no Siege.
    pub covered: u8,
    /// The attacker has Siege built in (an Industrial or later land unit, `data::built_in_siege`):
    /// it ignores every tier, `data::BUILT_IN_SIEGE_NOTE`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub built_in: bool,
    /// Percent of its normal damage the attack deals to the city's HP.
    pub percent: i32,
}

/// A visible garrison in the attacked city.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GarrisonForecast {
    pub unit: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub hp: i32,
    /// The city has HP left: it takes this attack and the garrison takes no damage.
    pub shielded: bool,
}

/// The city's automatic shot in the ranged phase (`rules::battle::resolve_ranged`), at a neutral
/// roll. Each city fires once, at the visible hostile military unit within two tiles with the
/// lowest HP, so the shot may go to a weaker unit instead of the attacker.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityStrikeForecast {
    pub city: CityId,
    /// The city's strength (its defense breakdown).
    pub city_strength: Breakdown,
    /// The attacker's defense where it stands now (terrain, fortification, promotion, wounds).
    pub attacker_defense: Breakdown,
    /// Damage at a 100% roll (the shot rolls 80-120%).
    pub damage: i32,
}

impl CityStrikeForecast {
    /// Exact integer mean of capped damage over the engine's rolls, not the neutral roll.
    pub fn expected_damage(&self, hp: i32) -> i32 {
        (combat::ROLL_MIN..=combat::ROLL_MAX)
            .map(|roll| self.damage_at_roll(roll).min(hp.max(0)))
            .sum::<i32>()
            / (combat::ROLL_MAX - combat::ROLL_MIN + 1)
    }

    pub fn damage_at_roll(&self, roll: i32) -> i32 {
        combat::damage(self.city_strength.total, self.attacker_defense.total, roll)
    }
}

/// Potential city fire at a military unit's proposed tile, from public information only.
/// Moving loses fortification. This does not claim the unit will be selected: each city
/// shoots once at the lowest (HP, ID), before movement. Callers evaluating arrivals are
/// budgeting the following ranged phase, not inventing a shot during movement.
pub fn forecast_city_strike(
    view: &PlayerView,
    city: &crate::view::CityView,
    unit: &crate::view::UnitView,
    pos: Hex,
) -> Option<CityStrikeForecast> {
    if !city_fires(city)
        || !unit.military
        || !view.is_threat(city.owner, unit.owner)
        || !(1..=2).contains(&city.pos.distance(pos))
    {
        return None;
    }
    let city_strength = combat::city_defense_for_rules(
        view.rule_experiments,
        view.faction_of(city.owner),
        city.population,
        city.walls,
        city.fort_tier(),
        city.hp,
        view.era_of(city.owner),
    );
    let terrain = view.tile(pos)?.terrain.unwrap_or(Terrain::Plains);
    let mut attacker_defense = combat::unit_defense_strength_for_rules(
        view.rule_experiments.faction_rules_off,
        view.faction_of(unit.owner),
        view.made_by(unit),
        unit.kind,
        unit.hp,
        terrain,
        if pos == unit.pos { unit.fortify } else { 0 },
        unit.level,
    );
    if pos == unit.pos {
        combat::add_phalanx_rules(
            &mut attacker_defense,
            view.rule_experiments.faction_rules_off,
            view.made_by(unit),
            unit.kind,
            phalanx_neighbour(view, unit),
        );
    }
    crate::resources::add_unsupplied(
        &mut attacker_defense,
        view.made_by(unit),
        unit.kind,
        false,
        unit.unsupplied,
    );
    Some(CityStrikeForecast {
        city: city.id,
        damage: combat::damage(city_strength.total, attacker_defense.total, 100),
        city_strength,
        attacker_defense,
    })
}

/// Whether another Spartan Hoplite of `u`'s empire stands next to it among the units the viewer
/// can see (`rules::battle::has_phalanx_neighbour` on the view). A hidden neighbour is not counted.
fn phalanx_neighbour(view: &PlayerView, u: &crate::view::UnitView) -> bool {
    let off = view.rule_experiments.faction_rules_off;
    crate::data::is_hoplite_rules(off, view.made_by(u), u.kind)
        && u.pos.neighbors().into_iter().any(|h| {
            view.units.iter().any(|o| {
                o.pos == h
                    && o.id != u.id
                    && o.owner == u.owner
                    && crate::data::is_hoplite_rules(off, view.made_by(o), o.kind)
            })
        })
}

/// Whether a known city fires in the ranged phase: it has Walls (any tier) and HP left
/// (`combat::city_can_fire`). Walls are public, so every viewer knows which cities shoot.
pub fn city_fires(city: &crate::view::CityView) -> bool {
    combat::city_can_fire(city.walls, city.hp)
}

/// The city's target among known units. Unknown enemies cannot be relied on to absorb fire.
/// City vision covers its two-tile firing radius; ties match resolution's unit ID order.
pub fn city_strike_target(view: &PlayerView, city: &crate::view::CityView) -> Option<UnitId> {
    if !city_fires(city) {
        return None;
    }
    view.units
        .iter()
        .filter(|u| {
            u.military
                && u.hp > 0
                && view.is_threat(city.owner, u.owner)
                && (1..=2).contains(&city.pos.distance(u.pos))
        })
        .min_by_key(|u| (u.hp, u.id))
        .map(|u| u.id)
}

/// What an attack by the viewer on `target` damages when an enemy city stands there, as the
/// viewer knows it (`combat::city_tile_defender` on the city's HP and its visible garrison):
/// `City` while the city has HP (its garrison is safe) or has no visible garrison, `Garrison`
/// once it is at 0 HP. None when no known enemy city is on the tile. AI scoring that values a
/// shot or a melee attack on a city tile should ask this rather than look for a unit there.
pub fn city_attack_target(view: &PlayerView, target: Hex) -> Option<combat::CityTileDefender> {
    let city = view
        .cities
        .iter()
        .find(|c| c.pos == target && is_enemy(view, c.owner))?;
    let garrisoned = view
        .units
        .iter()
        .any(|u| u.pos == target && u.military && is_enemy(view, u.owner));
    Some(combat::city_tile_defender(city.hp, garrisoned))
}

/// Who defends the target hex, as the viewer knows it.
enum Defender {
    Unit {
        faction: Faction,
        made_by: Faction,
        kind: UnitKind,
        hp: i32,
        fortify: i32,
        level: u8,
    },
    City {
        faction: Faction,
        population: i32,
        walls: bool,
        fort: u8,
        hp: i32,
        era: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackMode {
    /// Combat after a MoveUnit order reaches the target.
    Melee,
    /// Combat from a RangedAttack order, requiring a target within range.
    Ranged,
}

fn is_enemy(view: &PlayerView, owner: PlayerId) -> bool {
    view.at_war(view.me, owner)
}

/// Forecast `attacker` attacking whatever is at `target`. None if there is nothing to attack.
///
/// The attacker must be one of the viewer's own military units. On an enemy city's tile the
/// defender is the city while it has HP (its garrison is safe, reported in `garrison`), else its
/// visible garrison (`combat::city_tile_defender`); elsewhere the visible enemy military unit. A unit with a ranged attack and the target within range attacks at range and takes no
/// damage; otherwise the attack is melee and both sides take damage.
/// Use [`forecast_attack_mode`] to match a specific order instead of this ranged-first default.
///
/// Chances and expected damage are exact over every pair of rolls in
/// `combat::ROLL_MIN..=combat::ROLL_MAX`. Chances are percentages rounded down; expected damage
/// is the integer average of the damage actually dealt (capped at the side's current HP).
pub fn forecast_attack(view: &PlayerView, attacker: UnitId, target: Hex) -> Option<BattleForecast> {
    let att = view.units.iter().find(|u| u.id == attacker)?;
    let mode = if att.ranged > 0
        && att.pos.distance(target) <= att.range
        && los::ranged_clear_with_tech(
            att.kind,
            view.research.known.contains(&TechId::Steel),
            view.cities
                .iter()
                .any(|c| c.owner == att.owner && c.pos == att.pos && c.walls),
            att.pos,
            target,
            |h| view.tile(h).and_then(|t| t.terrain),
        ) {
        AttackMode::Ranged
    } else {
        AttackMode::Melee
    };
    forecast_attack_mode(view, attacker, target, mode)
}

/// Forecast an aircraft's strike (`Order::AirStrike`): the same combat math as a ranged shot, from
/// its city's tile, with the aircraft's own strike strength and range (a Bomber's, or a Fighter's
/// shorter one with no city bonus). `None` when the aircraft is not the viewer's or the target is
/// out of its range or has nothing to hit. A Fighter's interception is not predicted: rival hangars
/// are hidden.
pub fn forecast_air_strike(
    view: &PlayerView,
    plane: UnitId,
    target: Hex,
) -> Option<BattleForecast> {
    let a = view
        .aircraft
        .iter()
        .find(|a| a.id == plane && a.owner == view.me)?;
    let city = view
        .cities
        .iter()
        .find(|c| c.id == a.city && c.owner == view.me)?;
    let faction = view.faction_of(view.me);
    let stats = crate::data::unit_stats_for(faction, a.kind);
    if stats.ranged <= 0 || !(1..=stats.range).contains(&city.pos.distance(target)) {
        return None;
    }
    // The plane as a temporary unit standing in its city, as resolution does.
    let temporary = crate::view::UnitView {
        id: a.id,
        owner: a.owner,
        kind: a.kind,
        pos: city.pos,
        hp: a.hp,
        moves_left: 1,
        max_moves: 1,
        goto: None,
        path: Vec::new(),
        automation: None,
        standing_order: None,
        needs_orders: false,
        attack_target: None,
        fortify: 0,
        created_turn: None,
        strength: stats.strength,
        ranged: stats.ranged,
        range: stats.range,
        military: true,
        level: a.level,
        xp: None,
        xp_next: None,
        occupation_lock: None,
        upgrade_cost: None,
        upgrade_to: None,
        upgrade_steps: Vec::new(),
        upgrade_breakdown: None,
        upgrade_need: None,
        unsupplied: a.unsupplied,
        unsupplied_loss: a
            .unsupplied
            .map(|_| crate::resources::unsupplied_penalty(stats.ranged)),
        upgrade_supply_blocker: None,
        siege: false,
        siege_offer: None,
        sight: None,
        resting_heal: None,
        heir_of: None,
        contract_until: None,
        origin: None,
    };
    let mut with_plane = view.clone();
    let mut units = view.units.to_vec();
    units.push(temporary.clone());
    with_plane.units = units.into();
    let mut forecast = forecast_attack_mode(&with_plane, plane, target, AttackMode::Ranged)?;
    forecast.air_defence = air_defence_forecast(view, &temporary, target);
    Some(forecast)
}

/// The Air Defence duel an aircraft's strike on `target` would meet, if a visible enemy city
/// there has one and the aircraft is not a Stealth Bomber (`rules::air::air_defence_at`).
fn air_defence_forecast(
    view: &PlayerView,
    plane: &crate::view::UnitView,
    target: Hex,
) -> Option<AirDefenceForecast> {
    if plane.kind == UnitKind::StealthBomber {
        return None;
    }
    let city = view
        .cities
        .iter()
        .find(|c| c.pos == target && c.has_air_defence && is_enemy(view, c.owner))?;
    let off = view.rule_experiments.faction_rules_off;
    let interceptor = crate::rules::air::AIR_DEFENCE_INTERCEPTOR;
    let faction = view.faction_of(city.owner);
    let mut striker = combat::attack_strength_rules(
        off,
        view.faction_of(plane.owner),
        view.made_by(plane),
        plane.kind,
        plane.hp,
        plane.level,
        false,
        false,
        false,
        plane.unsupplied,
        false,
    );
    combat::add_fight_class_bonus_rules(
        &mut striker,
        off,
        view.made_by(plane),
        plane.kind,
        interceptor,
        true,
        false,
    );
    let terrain = view
        .tile(target)
        .and_then(|t| t.terrain)
        .unwrap_or(Terrain::Grassland);
    let mut defender = combat::unit_defense_strength_for_rules(
        off,
        faction,
        faction,
        interceptor,
        crate::data::UNIT_HP,
        terrain,
        0,
        0,
    );
    combat::add_fight_class_bonus_rules(
        &mut defender,
        off,
        faction,
        interceptor,
        plane.kind,
        false,
        false,
    );
    let odds = primitive_odds(
        striker.total,
        defender.total,
        plane.hp,
        crate::data::UNIT_HP,
        true,
        100,
    );
    Some(AirDefenceForecast {
        interceptor_strength: defender.total,
        striker_strength: striker.total,
        expected_damage_to_attacker: odds.exp_to_atk,
        loss_chance: odds.loss,
    })
}

/// Forecast the specified attack mode using only the player's view and the same combat math
/// as resolution. Melee assumes the unit reaches the target; it does not predict pathfinding or
/// simultaneous moves. Ranged returns None if the unit cannot shoot the target at this distance.
/// Both modes return None when the attacker is in closed foreign territory.
/// Defender selection and exact roll averages are the same as [`forecast_attack`].
pub fn forecast_attack_mode(
    view: &PlayerView,
    attacker: UnitId,
    target: Hex,
    mode: AttackMode,
) -> Option<BattleForecast> {
    // An aircraft in a hangar: its strike is forecast like a shot from its city.
    if !view.units.iter().any(|u| u.id == attacker)
        && view.aircraft.iter().any(|a| a.id == attacker)
    {
        return (mode == AttackMode::Ranged)
            .then(|| forecast_air_strike(view, attacker, target))
            .flatten();
    }
    let att = view
        .units
        .iter()
        .find(|u| u.id == attacker && u.owner == view.me && u.military)?
        .clone();
    if !view.passage_allowed(att.owner, att.kind, view.tile(att.pos)?.owner) {
        return None;
    }
    if att.pos == target {
        return None;
    }
    let ranged = mode == AttackMode::Ranged;
    if ranged
        && (att.ranged <= 0
            || att.pos.distance(target) > att.range
            || !los::ranged_clear_with_tech(
                att.kind,
                view.research.known.contains(&TechId::Steel),
                view.cities
                    .iter()
                    .any(|c| c.owner == att.owner && c.pos == att.pos && c.walls),
                att.pos,
                target,
                |h| view.tile(h).and_then(|t| t.terrain),
            ))
    {
        return None;
    }

    let city = view
        .cities
        .iter()
        .find(|c| c.pos == target && is_enemy(view, c.owner));
    let sack_gold = if !ranged
        && view.faction_of(att.owner) == Faction::Celts
        && !view.rule_experiments.rule_off(FactionRule::CeltsNoKeep)
    {
        let takes = !view.rule_experiments.rule_off(FactionRule::CeltsSack);
        city.filter(|c| c.founder != Some(att.owner)).map(|c| {
            if takes {
                crate::economy::celts_sack_gold(c.population)
            } else {
                0
            }
        })
    } else {
        None
    };
    let city_strike = city.and_then(|c| forecast_city_strike(view, c, &att, att.pos));
    // One military unit per tile; lowest id if the view somehow holds more.
    let enemy_unit = view
        .units
        .iter()
        .filter(|u| u.pos == target && u.military && is_enemy(view, u.owner))
        .min_by_key(|u| u.id);
    // Civ 6 (`combat::city_tile_defender`): a city with HP absorbs the attack and shields its
    // garrison; at 0 HP the garrison defends.
    let garrison = city.and_then(|c| {
        let u = enemy_unit?;
        Some(GarrisonForecast {
            unit: u.id,
            owner: u.owner,
            kind: u.kind,
            hp: u.hp,
            shielded: combat::city_tile_defender(c.hp, true) == combat::CityTileDefender::City,
        })
    });
    let unit_defends = match city {
        Some(c) => {
            combat::city_tile_defender(c.hp, enemy_unit.is_some())
                == combat::CityTileDefender::Garrison
        }
        None => true,
    };
    let defender = match (enemy_unit.filter(|_| unit_defends), city) {
        (Some(u), _) => Defender::Unit {
            faction: view.faction_of(u.owner),
            made_by: view.made_by(u),
            kind: u.kind,
            hp: u.hp,
            fortify: u.fortify,
            level: u.level,
        },
        (None, Some(c)) => Defender::City {
            faction: view.faction_of(c.owner),
            population: c.population,
            walls: c.walls,
            fort: c.fort_tier(),
            hp: c.hp,
            era: view.era_of(c.owner),
        },
        (None, None) => return None,
    };

    let defeated_city = matches!(defender, Defender::City { hp: 0, .. });
    let mut attacker_strength = combat::attack_strength_rules(
        view.rule_experiments.faction_rules_off,
        view.faction_of(att.owner),
        view.made_by(&att),
        att.kind,
        att.hp,
        att.level,
        ranged,
        false,
        phalanx_neighbour(view, &att),
        att.unsupplied,
        matches!(defender, Defender::City { .. }),
    );
    let wall_tier = match defender {
        Defender::City { walls, fort, .. } => u8::from(walls) + fort,
        Defender::Unit { .. } => 0,
    };
    let covered = siege_cover(view, &att);
    let walls = (wall_tier > 0).then(|| WallsForecast {
        tier: wall_tier,
        covered,
        built_in: crate::data::built_in_siege(att.kind),
        percent: combat::wall_damage_percent(
            wall_tier,
            covered,
            crate::data::is_aircraft(att.kind),
        ),
    });
    if let Defender::Unit { kind, .. } = defender {
        combat::add_fight_class_bonus_rules(
            &mut attacker_strength,
            view.rule_experiments.faction_rules_off,
            view.made_by(&att),
            att.kind,
            kind,
            true,
            ranged,
        );
    }
    let (mut defender_strength, defender_hp) = match defender {
        Defender::Unit {
            faction,
            made_by,
            kind,
            hp,
            fortify,
            level,
        } => {
            // A visible unit stands on an explored tile; unknown terrain gives no bonus.
            let terrain = view
                .tile(target)
                .and_then(|t| t.terrain)
                .unwrap_or(Terrain::Grassland);
            let mut strength = combat::unit_defense_strength_for_rules(
                view.rule_experiments.faction_rules_off,
                faction,
                made_by,
                kind,
                hp,
                terrain,
                fortify,
                level,
            );
            if let Some(u) = enemy_unit {
                combat::add_phalanx_rules(
                    &mut strength,
                    view.rule_experiments.faction_rules_off,
                    made_by,
                    kind,
                    phalanx_neighbour(view, u),
                );
                crate::resources::add_unsupplied(&mut strength, made_by, kind, false, u.unsupplied);
            }
            (strength, hp)
        }
        Defender::City {
            faction,
            population,
            walls,
            fort,
            hp,
            era,
        } => (
            combat::city_defense_vs(
                view.rule_experiments,
                faction,
                population,
                u8::from(walls) + fort,
                covered,
                hp,
                era,
            ),
            hp,
        ),
    };

    if let Defender::Unit { made_by, kind, .. } = defender {
        combat::add_fight_class_bonus_rules(
            &mut defender_strength,
            view.rule_experiments.faction_rules_off,
            made_by,
            kind,
            att.kind,
            false,
            ranged,
        );
    }
    let city_hp_percent = (matches!(defender, Defender::City { .. })
        && crate::data::city_hp_damage_percent(att.kind) != 100)
        .then(|| crate::data::city_hp_damage_percent(att.kind));
    let odds = primitive_odds(
        attacker_strength.total,
        defender_strength.total,
        att.hp,
        defender_hp,
        !ranged && !defeated_city,
        if matches!(defender, Defender::City { .. }) {
            walls.map_or(100, |w| w.percent) * city_hp_percent.unwrap_or(100) / 100
        } else {
            100
        },
    );

    Some(BattleForecast {
        attacker_strength,
        defender_strength,
        win_chance: odds.win,
        loss_chance: odds.loss,
        expected_damage_to_attacker: odds.exp_to_atk,
        expected_damage_to_defender: odds.exp_to_def,
        attacker_hp: att.hp,
        defender_hp,
        ranged,
        city_strike,
        garrison,
        sack_gold,
        walls,
        city_hp_percent,
        air_defence: None,
    })
}

/// Exact uniform-roll forecast for an already computed strength pair. Used by full battle
/// forecasts and by UI what-if explanations (for example, if fortification is lost first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimitiveOdds {
    pub win: u32,
    pub loss: u32,
    pub exp_to_atk: i32,
    pub exp_to_def: i32,
}

/// The attacked city's shot and the following attack share a turn but not a snapshot: the
/// shot can wound or destroy a melee attacker before its attack. Keep the combined roll
/// calculation in core so the card does not run a second combat engine in TypeScript.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityFireOutcome {
    pub fire: i32,
    pub attack: i32,
    pub hp_after: i32,
    pub win: i32,
    pub loss: i32,
    pub dealt: i32,
    pub can_die: bool,
}

pub fn city_fire_outcome(
    f: &BattleForecast,
    faction: Faction,
    kind: UnitKind,
) -> Option<CityFireOutcome> {
    let cs = f.city_strike.as_ref()?;
    let hp = f.attacker_hp;
    let defender = f.defender_strength.total;
    let wounds = |remaining: i32| {
        if data::ignores_wounds(faction, kind) {
            0
        } else {
            (data::UNIT_HP - remaining) / 10
        }
    };
    let rolls = i64::from(combat::ROLL_MAX - combat::ROLL_MIN + 1);
    let (mut fire, mut attack, mut win, mut loss, mut dealt) = (0i64, 0i64, 0i64, 0i64, 0i64);
    for city_roll in combat::ROLL_MIN..=combat::ROLL_MAX {
        let shot =
            combat::damage(cs.city_strength.total, cs.attacker_defense.total, city_roll).min(hp);
        let left = hp - shot;
        fire += i64::from(shot) * rolls * rolls;
        if left <= 0 {
            loss += rolls * rolls;
        }
        if left <= 0 && !f.ranged {
            continue;
        }
        let attacker = if f.ranged {
            f.attacker_strength.total
        } else {
            f.attacker_strength.total - (wounds(left) - wounds(hp))
        };
        for attack_roll in combat::ROLL_MIN..=combat::ROLL_MAX {
            for defense_roll in combat::ROLL_MIN..=combat::ROLL_MAX {
                let to_defender = combat::damage_to_city_hp(
                    f.city_percent(),
                    combat::damage(attacker, defender, attack_roll),
                );
                let to_attacker = if f.ranged {
                    0
                } else {
                    combat::damage(defender, attacker, defense_roll)
                };
                if to_defender >= f.defender_hp {
                    win += 1;
                }
                if !f.ranged && to_attacker >= left {
                    loss += 1;
                }
                dealt += i64::from(to_defender.min(f.defender_hp));
                attack += i64::from(to_attacker.min(left));
            }
        }
    }
    let outcomes = rolls * rolls * rolls;
    let fire = (fire / outcomes) as i32;
    let attack = (attack / outcomes) as i32;
    Some(CityFireOutcome {
        fire,
        attack,
        hp_after: (hp - fire - attack).max(0),
        can_die: loss > 0,
        win: (win * 100 / outcomes) as i32,
        loss: (loss * 100 / outcomes) as i32,
        dealt: (dealt / outcomes) as i32,
    })
}

pub fn primitive_odds(
    a: i32,
    d: i32,
    attacker_hp: i32,
    defender_hp: i32,
    counterattack: bool,
    city_percent: i32,
) -> PrimitiveOdds {
    // Both rolls are independent, so their 41x41 pairs factor into one sum per side.
    let rolls = i64::from(combat::ROLL_MAX - combat::ROLL_MIN + 1);
    let (mut win_rolls, mut loss_rolls, mut dealt_sum, mut taken_sum) = (0i64, 0i64, 0i64, 0i64);
    for roll in combat::ROLL_MIN..=combat::ROLL_MAX {
        let normal_damage = combat::damage(a, d, roll);
        let to_defender = combat::damage_to_city_hp(city_percent, normal_damage);
        let to_attacker = if counterattack {
            combat::damage(d, a, roll)
        } else {
            0
        };
        if to_defender >= defender_hp {
            win_rolls += 1;
        }
        if to_attacker >= attacker_hp {
            loss_rolls += 1;
        }
        dealt_sum += i64::from(to_defender.min(defender_hp));
        taken_sum += i64::from(to_attacker.min(attacker_hp));
    }
    let pairs = rolls * rolls;
    let wins = win_rolls * rolls;
    let losses = loss_rolls * rolls;
    let dealt = dealt_sum * rolls;
    let taken = taken_sum * rolls;

    PrimitiveOdds {
        win: (wins * 100 / pairs) as u32,
        loss: (losses * 100 / pairs) as u32,
        exp_to_atk: (taken / pairs) as i32,
        exp_to_def: (dealt / pairs) as i32,
    }
}

/// Chance that a split counterattack destroys this attacker, at uniform defender rolls.
pub fn split_counterattack_loss(
    defender: i32,
    attacker: i32,
    attacker_hp: i32,
    attackers: i32,
) -> i32 {
    if attackers <= 0 {
        return 0;
    }
    let lethal = (combat::ROLL_MIN..=combat::ROLL_MAX)
        .filter(|roll| {
            (combat::damage(defender, attacker, *roll) + attackers - 1) / attackers >= attacker_hp
        })
        .count() as i32;
    lethal * 100 / (combat::ROLL_MAX - combat::ROLL_MIN + 1)
}

/// The siege tier of the viewer's unit `u` (0 when it has no Siege): the view-side twin of
/// `rules::battle::siege_cover` (`data::siege_cover`), from the viewer's own technologies.
pub fn siege_cover(view: &PlayerView, u: &crate::view::UnitView) -> u8 {
    let techs: std::collections::BTreeSet<TechId> = view.research.known.iter().copied().collect();
    crate::data::siege_cover(u.kind, u.siege.then_some(&techs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::spawn_unit;
    use crate::state::GameState;
    use crate::view::player_view;
    use crate::view::tests::{P0, P1, P2, add_city, at, test_state};
    use crate::visibility;

    fn view0(s: &mut GameState) -> PlayerView {
        visibility::update_explored(s);
        player_view(s, P0)
    }

    #[test]
    fn experimental_city_strength_is_public_and_matches_forecast() {
        for walls in [false, true] {
            let mut s = test_state(24, 16);
            s.setup.rule_experiments.frontier_cities = true;
            s.setup.rule_experiments.wall_masonry = true;
            let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
            let c = add_city(&mut s, P2, at(6, 5), 1);
            s.cities.get_mut(&c).unwrap().population = 1;
            if walls {
                s.cities
                    .get_mut(&c)
                    .unwrap()
                    .buildings
                    .insert(BuildingKind::Walls);
            }
            let v = view0(&mut s);
            let f = forecast_attack(&v, a, at(6, 5)).unwrap();
            let city = v.cities.iter().find(|city| city.id == c).unwrap();
            assert_eq!(f.defender_strength.total, city.strength);
            let default = combat::city_defense_with_fort(
                s.player(P2).faction,
                1,
                walls,
                0,
                crate::data::CITY_HP,
                0,
            );
            assert_eq!(city.strength, default.total + if walls { 8 } else { -12 });
        }
    }

    #[test]
    fn celtic_melee_forecast_shows_the_exact_gold_of_a_sack() {
        let mut s = test_state(24, 16);
        s.player_mut(P0).faction = Faction::Celts;
        s.player_mut(P2).gold = 103;
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        let c = add_city(&mut s, P2, at(6, 5), 4);
        s.cities.get_mut(&c).unwrap().population = 4;
        let v = view0(&mut s);
        let melee = forecast_attack_mode(&v, a, at(6, 5), AttackMode::Melee).unwrap();
        assert_eq!(melee.sack_gold, Some(240));
        let r = spawn_unit(&mut s, P0, UnitKind::Ranger, at(5, 6));
        let v = view0(&mut s);
        let ranged = forecast_attack_mode(&v, r, at(6, 5), AttackMode::Ranged).unwrap();
        assert_eq!(ranged.sack_gold, None);
    }

    #[test]
    fn celtic_forecast_shows_no_sack_for_a_city_the_attacker_founded() {
        let mut s = test_state(24, 16);
        s.player_mut(P0).faction = Faction::Celts;
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        let c = add_city(&mut s, P2, at(6, 5), 4);
        s.cities.get_mut(&c).unwrap().founder = P0;
        let v = view0(&mut s);
        let melee = forecast_attack_mode(&v, a, at(6, 5), AttackMode::Melee).unwrap();
        assert_eq!(
            melee.sack_gold, None,
            "a retaken home city is kept, not sacked"
        );
    }

    #[test]
    fn city_strike_forecast_matches_the_shot_resolution_fires() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        let c = add_city(&mut s, P2, at(7, 5), 3);
        s.units.get_mut(&a).unwrap().hp = 70;
        s.units.get_mut(&a).unwrap().fortify = 3;
        let unwalled = forecast_attack(&view0(&mut s), a, at(7, 5)).unwrap();
        assert!(
            unwalled.city_strike.is_none(),
            "an unwalled city cannot fire"
        );
        s.cities
            .get_mut(&c)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        let v = view0(&mut s);
        let strike = forecast_attack(&v, a, at(7, 5))
            .unwrap()
            .city_strike
            .expect("the city is two tiles away");
        assert_eq!(strike.city, c);
        let mut events = Vec::new();
        crate::rules::battle::resolve_ranged(&mut s, &[], &mut events);
        let Some(crate::events::Event::Battle {
            attacker_strength,
            defender_strength,
            ..
        }) = events.first()
        else {
            panic!("the city fires: {events:?}")
        };
        assert_eq!(*attacker_strength, strike.city_strength);
        assert_eq!(*defender_strength, strike.attacker_defense);
        assert_eq!(
            strike.damage,
            combat::damage(
                strike.city_strength.total,
                strike.attacker_defense.total,
                100
            )
        );
        // Three tiles away the city cannot reach it.
        s.units.get_mut(&a).unwrap().pos = at(4, 5);
        assert!(
            forecast_attack(&view0(&mut s), a, at(7, 5))
                .unwrap()
                .city_strike
                .is_none()
        );
        s.units.get_mut(&a).unwrap().pos = at(6, 5);
        s.cities.get_mut(&c).unwrap().hp = 0;
        assert!(
            forecast_attack(&view0(&mut s), a, at(7, 5))
                .unwrap()
                .city_strike
                .is_none(),
            "a city at 0 HP doesn't fire"
        );
        s.cities.get_mut(&c).unwrap().hp = 1;
        assert!(
            forecast_attack(&view0(&mut s), a, at(7, 5))
                .unwrap()
                .city_strike
                .is_some()
        );
    }

    #[test]
    fn city_fire_forecast_applies_walls_to_the_return_shot() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Ranger, at(5, 5));
        let c = add_city(&mut s, P2, at(6, 5), 1);
        s.cities.get_mut(&c).unwrap().population = 1;
        s.cities
            .get_mut(&c)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        let f = forecast_attack_mode(&view0(&mut s), a, at(6, 5), AttackMode::Ranged)
            .expect("Archer can shoot the walled city");
        let fire =
            city_fire_outcome(&f, s.player(P0).faction, UnitKind::Ranger).expect("the city fires");
        assert_eq!(f.walls.unwrap().percent, 50);
        assert_eq!(f.expected_damage_to_defender, 11);
        assert_eq!(
            fire.dealt, 11,
            "the card uses this number when city fire is shown"
        );
        assert_eq!(fire.win, 0);
    }

    #[test]
    fn mounted_city_penalty_is_shared_by_single_and_combined_forecasts() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Horseman, at(5, 5));
        let c = add_city(&mut s, P2, at(6, 5), 1);
        s.cities.get_mut(&c).unwrap().population = 1;
        let v = view0(&mut s);
        let single = forecast_attack_mode(&v, a, at(6, 5), AttackMode::Melee).unwrap();
        let combined = forecast_city_assault(&v, a, at(6, 5), AttackMode::Melee, &[]).unwrap();
        assert_eq!(single.expected_damage_to_defender, 37);
        assert_eq!(
            combined.city_hp_after, 162,
            "the integer mean of the remaining HP"
        );
        assert_eq!(single.attacker_strength.total, 18);
    }

    #[test]
    fn dying_melee_attacker_cannot_capture_a_fallen_city() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        s.units.get_mut(&a).unwrap().hp = 1;
        let c = add_city(&mut s, P2, at(6, 5), 1);
        s.cities.get_mut(&c).unwrap().population = 1;
        s.cities.get_mut(&c).unwrap().hp = 1;
        let forecast =
            forecast_city_assault(&view0(&mut s), a, at(6, 5), AttackMode::Melee, &[]).unwrap();
        assert!(forecast.can_capture);
        assert_eq!(forecast.city_hp_after, 0);
        assert_eq!(forecast.capture_chance, 0);
        s.units.get_mut(&a).unwrap().goto = Some(at(6, 5));
        crate::rules::movement::resolve_movement(&mut s, &mut Vec::new());
        assert_eq!(s.cities[&c].owner, P2);
        assert!(!s.units.contains_key(&a));
    }

    #[test]
    fn city_fire_arrival_forecast_and_target_match_resolution_without_hidden_state() {
        let mut s = test_state(24, 16);
        let c = add_city(&mut s, P2, at(7, 5), 3);
        s.cities
            .get_mut(&c)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        let b = spawn_unit(&mut s, P0, UnitKind::Soldier, at(6, 5));
        let civilian = spawn_unit(&mut s, P0, UnitKind::Settler, at(6, 6));
        s.units.get_mut(&a).unwrap().hp = 65;
        s.units.get_mut(&b).unwrap().hp = 65;
        s.units.get_mut(&a).unwrap().fortify = 6;
        let mut v = view0(&mut s);
        let city = v.cities.iter().find(|x| x.id == c).unwrap();
        let u = v.units.iter().find(|u| u.id == a).unwrap();
        assert_eq!(city_strike_target(&v, city), Some(a));
        let here = forecast_city_strike(&v, city, u, u.pos).unwrap();
        let arrival = forecast_city_strike(&v, city, u, at(6, 5)).unwrap();
        assert_eq!(
            here.attacker_defense.total - arrival.attacker_defense.total,
            6
        );
        assert_eq!(here.expected_damage(1), 1, "cap damage at remaining HP");
        let average = (combat::ROLL_MIN..=combat::ROLL_MAX)
            .map(|roll| {
                combat::damage(here.city_strength.total, here.attacker_defense.total, roll)
                    .min(u.hp)
            })
            .sum::<i32>()
            / 41;
        assert_eq!(here.expected_damage(u.hp), average);
        assert!(forecast_city_strike(&v, city, u, at(4, 5)).is_none());
        assert!(
            forecast_city_strike(
                &v,
                city,
                v.units.iter().find(|u| u.id == civilian).unwrap(),
                at(6, 5)
            )
            .is_none()
        );
        let mut events = Vec::new();
        crate::rules::battle::resolve_ranged(&mut s, &[], &mut events);
        assert!(events.iter().any(|e| matches!(e, crate::Event::Battle {
            attacker: crate::events::Attacker::City { city },
            defender: crate::events::Defender::Unit { unit, .. }, ..
        } if *city == c && *unit == a)));
        v.units.reverse();
        let city = v.cities.iter().find(|x| x.id == c).unwrap();
        assert_eq!(
            city_strike_target(&v, city),
            Some(a),
            "iteration order cannot break HP ties"
        );
    }

    #[test]
    fn city_strike_forecast_and_target_ignore_forest_and_hill_blockers() {
        for blocker in [Terrain::Forest, Terrain::Hills] {
            let mut s = test_state(24, 16);
            let city_id = add_city(&mut s, P2, at(7, 5), 3);
            s.cities
                .get_mut(&city_id)
                .unwrap()
                .buildings
                .insert(BuildingKind::Walls);
            let unit_id = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
            s.map.get_mut(at(6, 5)).unwrap().terrain = blocker;
            spawn_unit(&mut s, P2, UnitKind::Settler, at(5, 4));
            spawn_unit(&mut s, P0, UnitKind::Settler, at(7, 4));
            let approaching_id = spawn_unit(&mut s, P0, UnitKind::Soldier, at(4, 5));
            let v = view0(&mut s);
            let city = v.cities.iter().find(|c| c.id == city_id).unwrap();
            let unit = v.units.iter().find(|u| u.id == unit_id).unwrap();
            assert_eq!(city_strike_target(&v, city), Some(unit_id));
            assert!(forecast_city_strike(&v, city, unit, unit.pos).is_some());
            let approaching = v.units.iter().find(|u| u.id == approaching_id).unwrap();
            assert!(forecast_city_strike(&v, city, approaching, approaching.pos).is_none());
            assert!(forecast_city_strike(&v, city, approaching, unit.pos).is_some());
            assert!(
                forecast_attack(&v, unit_id, city.pos)
                    .unwrap()
                    .city_strike
                    .is_some()
            );
        }
    }

    #[test]
    fn promotions_are_their_own_forecast_lines_and_match_resolution() {
        use crate::rules::promotion;
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        let d = spawn_unit(&mut s, P2, UnitKind::Soldier, at(6, 5));
        let r = spawn_unit(&mut s, P0, UnitKind::Ranger, at(4, 5));
        s.units.get_mut(&a).unwrap().xp = promotion::threshold(2).unwrap();
        s.units.get_mut(&d).unwrap().xp = promotion::threshold(1).unwrap();
        s.units.get_mut(&r).unwrap().xp = promotion::threshold(3).unwrap();
        let v = view0(&mut s);
        let f = forecast_attack(&v, a, at(6, 5)).unwrap();
        assert_eq!(
            f.attacker_strength.parts[1],
            ("Level 2: +20%".to_string(), 4)
        );
        assert_eq!(
            f.defender_strength.parts[1],
            ("Level 1: +10%".to_string(), 2)
        );
        assert_eq!(
            (f.attacker_strength.total, f.defender_strength.total),
            (24, 22)
        );
        let shot = forecast_attack(&v, r, at(6, 5)).unwrap();
        assert!(shot.ranged);
        assert_eq!(
            shot.attacker_strength.parts[1],
            ("Level 3: +30%".to_string(), 4)
        );
        // The same lines resolution uses.
        let fight = crate::rules::battle::plan_fight(
            &s,
            1,
            a,
            crate::rules::battle::Target::Unit(d),
            crate::rules::battle::FightKind::Melee,
        );
        assert_eq!(fight.attacker_strength, f.attacker_strength);
        assert_eq!(fight.defender_strength, f.defender_strength);
        // Level is public; XP is the owner's.
        let enemy = v.units.iter().find(|u| u.id == d).unwrap();
        assert_eq!((enemy.level, enemy.xp, enemy.xp_next), (1, None, None));
        let own = v.units.iter().find(|u| u.id == a).unwrap();
        assert_eq!(own.level, 2);
        assert_eq!(own.xp, promotion::threshold(2));
        assert_eq!(own.xp_next, promotion::threshold(3));
    }

    #[test]
    fn melee_matches_hand_checked_numbers() {
        // Soldier (20) vs Soldier (20) on grassland: 30 base damage each way, rolls 80..=120.
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(6, 5));
        let f = forecast_attack(&view0(&mut s), a, at(6, 5)).unwrap();
        assert_eq!(
            (f.attacker_strength.total, f.defender_strength.total),
            (20, 20)
        );
        assert_eq!((f.win_chance, f.loss_chance), (0, 0));
        assert_eq!(f.expected_damage_to_defender, 29);
        assert_eq!(f.expected_damage_to_attacker, 29);
        assert!(!f.ranged);
        assert_eq!(
            f.expected_damage_to_defender,
            (80..=120).map(|r| combat::damage(20, 20, r)).sum::<i32>() / 41
        );

        // Trooper (32 on the era curve) vs Soldier at 40 HP on hills, fortified +6: 20 + 5 + 6 - 6 = 25.
        // Difference 7 -> kills when the roll is high enough: 51%.
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Trooper, at(5, 5));
        let d = spawn_unit(&mut s, P2, UnitKind::Soldier, at(6, 5));
        s.map.get_mut(at(6, 5)).unwrap().terrain = Terrain::Hills;
        let du = s.units.get_mut(&d).unwrap();
        du.hp = 40;
        du.fortify = 6;
        let f = forecast_attack(&view0(&mut s), a, at(6, 5)).unwrap();
        assert_eq!(f.defender_strength.total, 25);
        assert_eq!(
            f.defender_strength.parts,
            vec![
                ("Spearman".to_string(), 20),
                ("Hills terrain".to_string(), 5),
                ("Fortified".to_string(), 6),
                ("Wounded".to_string(), -6),
            ]
        );
        assert_eq!((f.win_chance, f.loss_chance), (51, 0));
        assert_eq!(
            f.expected_damage_to_defender, 37,
            "capped at the 40 HP left"
        );
        assert_eq!(f.expected_damage_to_attacker, 22);
        assert_eq!((f.attacker_hp, f.defender_hp), (100, 40));

        // Wounded Scout (70 HP: 5 - 3 = 2) into a Soldier: dies when the roll >= 113: 8/41 = 19%.
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Scout, at(5, 5));
        s.units.get_mut(&a).unwrap().hp = 70;
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(6, 5));
        let f = forecast_attack(&view0(&mut s), a, at(6, 5)).unwrap();
        assert_eq!((f.win_chance, f.loss_chance), (0, 19));
        assert_eq!(f.expected_damage_to_attacker, 61);
        assert_eq!(f.expected_damage_to_defender, 14);
    }

    #[test]
    fn adjacent_ranger_melee_forecast_matches_its_movement_attack() {
        use crate::{AttackMode, forecast_attack_mode};
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Ranger, at(5, 5));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(6, 5));
        let v = view0(&mut s);
        let melee = forecast_attack_mode(&v, a, at(6, 5), AttackMode::Melee).unwrap();
        assert!(!melee.ranged);
        assert_eq!(melee.attacker_strength.total, 8);
        assert_eq!(melee.expected_damage_to_defender, 18);
        assert_eq!(melee.expected_damage_to_attacker, 47);
        let ranged = forecast_attack_mode(&v, a, at(6, 5), AttackMode::Ranged).unwrap();
        assert!(ranged.ranged);
        assert_eq!(ranged.attacker_strength.total, 15);
        assert_eq!(ranged.expected_damage_to_defender, 24);
        assert_eq!(ranged.expected_damage_to_attacker, 0);
        assert_eq!(forecast_attack(&v, a, at(6, 5)), Some(ranged));

        s.units.get_mut(&a).unwrap().goto = Some(at(6, 5));
        let mut events = Vec::new();
        crate::rules::movement::resolve_movement(&mut s, &mut events);
        let battle = events
            .iter()
            .find(|e| matches!(e, crate::Event::Battle { .. }))
            .unwrap();
        let crate::Event::Battle {
            attacker_strength,
            defender_strength,
            attacker_roll,
            defender_roll,
            damage_to_attacker,
            damage_to_defender,
            ranged,
            ..
        } = battle
        else {
            unreachable!()
        };
        assert!(!ranged);
        assert_eq!(attacker_strength, &melee.attacker_strength);
        assert_eq!(defender_strength, &melee.defender_strength);
        assert_eq!(
            *damage_to_defender,
            combat::damage(
                attacker_strength.total,
                defender_strength.total,
                *attacker_roll
            )
        );
        assert_eq!(
            *damage_to_attacker,
            combat::damage(
                defender_strength.total,
                attacker_strength.total,
                *defender_roll
            )
        );
    }

    #[test]
    fn explicit_ranged_forecast_requires_a_ranged_unit_and_target_in_range() {
        use crate::{AttackMode, forecast_attack_mode};
        let mut s = test_state(24, 16);
        let ranger = spawn_unit(&mut s, P0, UnitKind::Ranger, at(5, 5));
        let soldier = spawn_unit(&mut s, P0, UnitKind::Soldier, at(6, 5));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(7, 5));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(8, 5));
        let v = view0(&mut s);
        assert!(forecast_attack_mode(&v, soldier, at(7, 5), AttackMode::Ranged).is_none());
        assert!(forecast_attack_mode(&v, ranger, at(8, 5), AttackMode::Ranged).is_none());
        assert!(!forecast_attack(&v, ranger, at(8, 5)).unwrap().ranged);
    }

    #[test]
    fn ranged_attack_deals_no_damage_to_attacker() {
        let mut s = test_state(24, 16);
        let r = spawn_unit(&mut s, P0, UnitKind::Ranger, at(5, 5));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(7, 5));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(8, 5));
        // A spotter so the farther target is in sight.
        spawn_unit(&mut s, P0, UnitKind::Scout, at(10, 5));
        let v = view0(&mut s);
        let f = forecast_attack(&v, r, at(7, 5)).unwrap();
        assert!(f.ranged);
        assert_eq!(f.attacker_strength.total, 15);
        assert_eq!(f.expected_damage_to_attacker, 0);
        assert_eq!(f.loss_chance, 0);
        assert_eq!((f.win_chance, f.expected_damage_to_defender), (0, 24));

        // Out of range (3 hexes): the Ranger would have to walk in and fight in melee.
        let f = forecast_attack(&v, r, at(8, 5)).unwrap();
        assert!(!f.ranged);
        assert_eq!(f.attacker_strength.total, 8);
        assert!(f.expected_damage_to_attacker > 0);
    }

    #[test]
    fn a_city_with_hp_shields_its_garrison_and_a_fallen_one_does_not() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        add_city(&mut s, P2, at(7, 5), 3);
        let target = at(7, 5);

        // Ungarrisoned: the city defends with 10 + 2 x 3 = 16 and 200 HP.
        let f = forecast_attack(&view0(&mut s), a, target).unwrap();
        assert_eq!(f.defender_strength.total, 16);
        assert_eq!(f.defender_hp, 200);
        assert_eq!((f.win_chance, f.loss_chance), (0, 0));
        assert_eq!(f.expected_damage_to_defender, 34);
        assert_eq!(f.expected_damage_to_attacker, 25);

        assert!(f.garrison.is_none());
        assert!(!serde_json::to_string(&f).unwrap().contains("garrison"));

        // Garrisoned by a Soldier fortified +3: the city still takes the attack (Civ 6) with
        // the same numbers, and the forecast says the garrison is safe.
        let g = spawn_unit(&mut s, P2, UnitKind::Soldier, target);
        s.units.get_mut(&g).unwrap().fortify = 3;
        let shielded = forecast_attack(&view0(&mut s), a, target).unwrap();
        assert_eq!(
            (
                shielded.defender_strength.total,
                shielded.defender_hp,
                shielded.expected_damage_to_defender,
                shielded.expected_damage_to_attacker
            ),
            (16, 200, 34, 25)
        );
        assert_eq!(
            shielded.garrison,
            Some(GarrisonForecast {
                unit: g,
                owner: P2,
                kind: UnitKind::Soldier,
                hp: 100,
                shielded: true,
            })
        );

        // At 0 HP the Soldier (23) defends; beating it captures the city.
        s.cities.values_mut().for_each(|c| c.hp = 0);
        let f = forecast_attack(&view0(&mut s), a, target).unwrap();
        assert_eq!(f.defender_strength.total, 23);
        assert_eq!(f.defender_hp, 100);
        assert_eq!(f.expected_damage_to_defender, 26);
        assert_eq!(f.expected_damage_to_attacker, 33);
        assert!(f.garrison.is_some_and(|g| !g.shielded));

        // The forecast's defender is the one resolution fights.
        s.cities.values_mut().for_each(|c| c.hp = 200);
        s.units.get_mut(&a).unwrap().pos = at(6, 5);
        s.units.get_mut(&a).unwrap().goto = Some(target);
        let mut events = Vec::new();
        crate::rules::movement::resolve_movement(&mut s, &mut events);
        assert!(events.iter().any(|e| matches!(e, crate::Event::Battle {
            defender: crate::events::Defender::City { .. }, defender_strength, ..
        } if defender_strength.total == 16)));
        assert_eq!(s.units[&g].hp, 100);
    }

    #[test]
    fn a_supported_assault_shares_the_damage_and_pools_the_reply_as_resolution_does() {
        // Supported defence (designer, 2026-11-15): two Soldiers on a fallen city's fortified
        // garrison, with two of its owner's Soldiers beside it ordered in.
        let mut s = test_state(24, 16);
        let target = at(7, 5);
        let around = target.neighbors();
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, around[3]);
        let b = spawn_unit(&mut s, P0, UnitKind::Soldier, around[4]);
        let c = add_city(&mut s, P2, target, 3);
        s.cities.get_mut(&c).unwrap().hp = 0;
        let g = spawn_unit(&mut s, P2, UnitKind::Soldier, target);
        s.units.get_mut(&g).unwrap().fortify = 3;
        let s1 = spawn_unit(&mut s, P2, UnitKind::Soldier, around[0]);
        let s2 = spawn_unit(&mut s, P2, UnitKind::Soldier, around[1]);
        let v = view0(&mut s);
        let with_b = [AlongsideAttack {
            unit: b,
            mode: AttackMode::Melee,
        }];
        let alone = forecast_city_assault(&v, a, target, AttackMode::Melee, &with_b).unwrap();
        let supported = super::forecast_city_assault_supported(
            &v,
            a,
            target,
            AttackMode::Melee,
            &with_b,
            &[s1, s2],
        )
        .unwrap();
        assert!(alone.supporters.is_empty());
        assert_eq!(supported.supporters.len(), 2);
        let (alone_g, sup_g) = (alone.garrison.unwrap(), supported.garrison.clone().unwrap());
        assert!(
            sup_g.hp_after > alone_g.hp_after,
            "{alone_g:?} vs {sup_g:?}"
        );
        assert!(supported.supporters.iter().all(|x| x.hp_after < x.hp));
        assert!(
            supported.reply > 2 * alone.reply,
            "three replies, not one: {} vs {}",
            supported.reply,
            alone.reply
        );
        assert!(supported.capture_chance <= alone.capture_chance);

        // Resolution at average rolls lands close to the forecast's means.
        crate::rules::battle::AVERAGE_ROLLS.with(|r| r.set(true));
        for (u, to) in [(a, target), (b, target), (s1, target), (s2, target)] {
            s.units.get_mut(&u).unwrap().goto = Some(to);
        }
        let mut events = Vec::new();
        crate::rules::movement::resolve_movement(&mut s, &mut events);
        crate::rules::battle::AVERAGE_ROLLS.with(|r| r.set(false));
        let near = |x: i32, y: i32| (x - y).abs() <= 3;
        assert!(
            near(s.units[&g].hp, sup_g.hp_after),
            "{} vs {sup_g:?}",
            s.units[&g].hp
        );
        for (u, f) in [s1, s2].iter().zip(&supported.supporters) {
            assert!(
                near(s.units[u].hp, f.hp_after),
                "{} vs {f:?}",
                s.units[u].hp
            );
        }
        assert!(
            near(100 - s.units[&a].hp, supported.reply),
            "{}",
            s.units[&a].hp
        );
    }

    #[test]
    fn the_assault_pools_every_queued_attack_and_matches_resolution() {
        let mut s = test_state(24, 16);
        let target = at(7, 5);
        let around = target.neighbors();
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, around[0]);
        let b = spawn_unit(&mut s, P0, UnitKind::Soldier, around[1]);
        let c = add_city(&mut s, P2, target, 3);
        s.cities.get_mut(&c).unwrap().hp = 30;
        let g = spawn_unit(&mut s, P2, UnitKind::Soldier, target);
        s.units.get_mut(&g).unwrap().fortify = 3;
        let v = view0(&mut s);

        let alone = forecast_city_assault(&v, a, target, AttackMode::Melee, &[]).unwrap();
        assert_eq!((alone.attacks, alone.city_hp), (1, 30));
        assert!(alone.can_capture, "a melee attack can take the city");
        let both = forecast_city_assault(
            &v,
            a,
            target,
            AttackMode::Melee,
            &[AlongsideAttack {
                unit: b,
                mode: AttackMode::Melee,
            }],
        )
        .unwrap();
        assert_eq!(both.attacks, 2);
        let (alone_g, both_g) = (alone.garrison.unwrap(), both.garrison.unwrap());
        assert!(
            both_g.hp_after < alone_g.hp_after,
            "the second attack's excess reaches the garrison: {alone_g:?} then {both_g:?}"
        );
        assert_eq!(both.city_hp_after, 0);
        // The carried part of the hit that empties the city is reported on its own: with one attack
        // it is the whole of what the garrison loses (to the integer mean).
        assert!(alone_g.carried > 0, "{alone_g:?}");
        assert!(
            (alone_g.hp - alone_g.hp_after - alone_g.carried).abs() <= 1,
            "{alone_g:?}"
        );
        assert!(
            both_g.carried >= alone_g.carried,
            "{alone_g:?} then {both_g:?}"
        );

        // The forecast's garrison HP is the mean of what resolution leaves over many seeds.
        let (mut sum, n) = (0i64, 300u64);
        for seed in 0..n {
            let mut t = s.clone();
            t.setup.seed = seed;
            for u in [a, b] {
                t.units.get_mut(&u).unwrap().goto = Some(target);
            }
            crate::rules::movement::resolve_movement(&mut t, &mut Vec::new());
            sum += i64::from(t.units.get(&g).map_or(0, |u| u.hp));
        }
        let mean = (sum / n as i64) as i32;
        assert!(
            (mean - both_g.hp_after).abs() <= 6,
            "resolution leaves about {mean} HP, the forecast says {}",
            both_g.hp_after
        );
    }

    #[test]
    fn the_assault_names_capture_only_with_a_melee_attack_and_a_fallen_garrison() {
        let mut s = test_state(24, 16);
        let target = at(7, 5);
        let around = target.neighbors();
        let archer = spawn_unit(&mut s, P0, UnitKind::Ranger, around[0]);
        let soldier = spawn_unit(&mut s, P0, UnitKind::Soldier, around[1]);
        let c = add_city(&mut s, P2, target, 3);
        s.cities.get_mut(&c).unwrap().hp = 0;
        let g = spawn_unit(&mut s, P2, UnitKind::Soldier, target);
        s.units.get_mut(&g).unwrap().hp = 1;
        let v = view0(&mut s);

        // A shot alone never captures, however weak the garrison.
        let shot = forecast_city_assault(&v, archer, target, AttackMode::Ranged, &[]).unwrap();
        assert!(!shot.can_capture);
        assert_eq!(shot.capture_chance, 0);
        // The shot kills the 1 HP garrison; the Soldier's charge then walks into the city.
        let both = forecast_city_assault(
            &v,
            archer,
            target,
            AttackMode::Ranged,
            &[AlongsideAttack {
                unit: soldier,
                mode: AttackMode::Melee,
            }],
        )
        .unwrap();
        assert!(both.can_capture);
        assert_eq!(both.capture_chance, 100);
        assert_eq!(both.garrison.unwrap().destroyed_chance, 100);
    }

    #[test]
    fn nothing_attackable_is_none() {
        let mut s = test_state(24, 16);
        let a = spawn_unit(&mut s, P0, UnitKind::Soldier, at(5, 5));
        let settler = spawn_unit(&mut s, P0, UnitKind::Settler, at(5, 6));
        let mate = spawn_unit(&mut s, P1, UnitKind::Soldier, at(6, 5));
        let enemy = spawn_unit(&mut s, P2, UnitKind::Soldier, at(4, 5));
        spawn_unit(&mut s, P2, UnitKind::Settler, at(6, 6));
        spawn_unit(&mut s, P2, UnitKind::Soldier, at(20, 12));
        add_city(&mut s, P1, at(10, 10), 2);
        let v = view0(&mut s);

        assert!(forecast_attack(&v, a, at(5, 4)).is_none(), "empty tile");
        assert!(forecast_attack(&v, a, at(5, 5)).is_none(), "own tile");
        assert!(forecast_attack(&v, a, at(6, 5)).is_none(), "teammate unit");
        assert!(
            forecast_attack(&v, a, at(10, 10)).is_none(),
            "teammate city"
        );
        assert!(forecast_attack(&v, a, at(6, 6)).is_none(), "civilian only");
        assert!(
            forecast_attack(&v, a, at(20, 12)).is_none(),
            "hidden by fog"
        );
        assert!(
            forecast_attack(&v, settler, at(4, 5)).is_none(),
            "civilian attacker"
        );
        assert!(
            forecast_attack(&v, enemy, at(5, 5)).is_none(),
            "not our unit"
        );
        assert!(
            forecast_attack(&v, mate, at(4, 5)).is_none(),
            "teammate's unit"
        );
        assert!(
            forecast_attack(&v, UnitId(999), at(4, 5)).is_none(),
            "no such unit"
        );
        assert!(forecast_attack(&v, a, at(4, 5)).is_some());
    }
}
