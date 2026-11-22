//! Strongest Army: comparable, additive field-combat value, shared by scoring, the ledger and AI.
//! The square is calibrated on real tactical battles, not on production costs. It rewards
//! quality without the unbounded exponential's extreme late-era ratios (damage caps at +/-50).
use crate::{
    GameState, PlayerId, UnitKind, data, resources,
    rules::promotion,
    state::{Aircraft, Unit},
};
use serde::{Deserialize, Serialize};

/// A healthy strength-20 Spearman contributes 20 points.
pub const STRENGTH_REFERENCE: i32 = 20;

/// Strength before situational bonuses: the better attack, promotions and supply included.
/// Health scales endurance separately; terrain, fortification and matchup bonuses depend on
/// where/whom a unit fights, so neither they nor the wound penalty alter this standing value.
pub fn effective_strength(strength: i32, ranged: i32, level: u8, unsupplied: bool) -> i32 {
    let base = strength.max(ranged).max(0);
    base + promotion::strength_bonus(base, level)
        - if unsupplied {
            resources::unsupplied_penalty(base)
        } else {
            0
        }
}

/// The integer points at full health. i64 intermediates keep the calculation deterministic.
pub fn full_health_value(strength: i32) -> i32 {
    let s = i64::from(strength.max(0));
    (s * s / i64::from(STRENGTH_REFERENCE)).min(i64::from(i32::MAX)) as i32
}

/// Health is applied to the displayed full-health value so the ledger's numbers reproduce it.
pub fn combat_value(strength: i32, ranged: i32, hp: i32, level: u8, unsupplied: bool) -> i32 {
    let full = full_health_value(effective_strength(strength, ranged, level, unsupplied));
    (i64::from(full) * i64::from(hp.clamp(0, data::UNIT_HP)) / i64::from(data::UNIT_HP)) as i32
}

pub fn unit_value(state: &GameState, unit: &Unit) -> i32 {
    let f = state.player(unit.owner).faction;
    let off = state.setup.rule_experiments.faction_rules_off;
    let stats = state.unit_stats_of(unit);
    if !stats.military || data::is_aircraft(unit.kind) {
        return 0;
    }
    combat_value(
        stats.strength,
        stats.ranged,
        unit.hp,
        promotion::level_for_rules(off, f, unit.xp),
        resources::unit_unsupplied(state, unit).is_some(),
    )
}

/// A hangar aircraft uses the same standing combat formula as a map unit.
pub fn aircraft_value(state: &GameState, aircraft: &Aircraft) -> i32 {
    let Some(owner) = state.cities.get(&aircraft.city).map(|city| city.owner) else {
        return 0;
    };
    let stats = state
        .setup
        .rule_experiments
        .unit_stats_for(state.player(owner).faction, aircraft.kind);
    let unsupplied = resources::is_idle(
        state.player(owner),
        resources::SupplyUser::Unit { unit: aircraft.id },
    );
    let level = promotion::level_for_rules(
        state.setup.rule_experiments.faction_rules_off,
        state.player(owner).faction,
        aircraft.xp,
    );
    combat_value(stats.strength, stats.ranged, aircraft.hp, level, unsupplied)
}

/// A fresh unit's contribution, for the AI's army purchase and production forecasts.
pub fn fresh_value(faction: crate::Faction, kind: UnitKind) -> i32 {
    let stats = data::unit_stats_for(faction, kind);
    if !stats.military && !data::is_aircraft(kind) {
        return 0;
    }
    combat_value(
        stats.strength,
        stats.ranged,
        data::UNIT_HP,
        u8::from(faction == crate::Faction::Sparta),
        false,
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    pub unit: crate::UnitId,
    pub kind: UnitKind,
    pub strength: i32,
    pub ranged: i32,
    pub level: u8,
    pub unsupplied: bool,
    pub promotion_bonus: i32,
    pub supply_penalty: i32,
    pub combat_strength: i32,
    pub full_health: i32,
    pub hp: i32,
    pub value: i32,
}

/// Owner-only detail: a public total must never reveal fogged rivals' unit types or locations.
pub fn breakdown(state: &GameState, player: PlayerId) -> Vec<Contribution> {
    let mut contributions: Vec<_> = state
        .units_of(player)
        .filter_map(|u| {
            let f = state.player(player).faction;
            let off = state.setup.rule_experiments.faction_rules_off;
            let stats = state.unit_stats_of(u);
            if !stats.military || data::is_aircraft(u.kind) {
                return None;
            }
            let level = promotion::level_for_rules(off, f, u.xp);
            let unsupplied = resources::unit_unsupplied(state, u).is_some();
            let strength = effective_strength(stats.strength, stats.ranged, level, unsupplied);
            Some(Contribution {
                unit: u.id,
                kind: u.kind,
                strength: stats.strength,
                ranged: stats.ranged,
                level,
                unsupplied,
                promotion_bonus: promotion::strength_bonus(stats.strength.max(stats.ranged), level),
                supply_penalty: if unsupplied {
                    resources::unsupplied_penalty(stats.strength.max(stats.ranged))
                } else {
                    0
                },
                combat_strength: strength,
                full_health: full_health_value(strength),
                hp: u.hp,
                value: unit_value(state, u),
            })
        })
        .collect();
    for aircraft in state.aircraft.iter().filter(|aircraft| {
        state
            .cities
            .get(&aircraft.city)
            .is_some_and(|city| city.owner == player)
    }) {
        let stats = data::unit_stats_for(state.player(player).faction, aircraft.kind);
        let unsupplied = resources::is_idle(
            state.player(player),
            resources::SupplyUser::Unit { unit: aircraft.id },
        );
        let strength = effective_strength(stats.strength, stats.ranged, 0, unsupplied);
        contributions.push(Contribution {
            unit: aircraft.id,
            kind: aircraft.kind,
            strength: stats.strength,
            ranged: stats.ranged,
            level: 0,
            unsupplied,
            promotion_bonus: 0,
            supply_penalty: if unsupplied {
                resources::unsupplied_penalty(stats.strength.max(stats.ranged))
            } else {
                0
            },
            combat_strength: strength,
            full_health: full_health_value(strength),
            hp: aircraft.hp,
            value: aircraft_value(state, aircraft),
        });
    }
    contributions
}
