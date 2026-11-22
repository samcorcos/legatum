//! Immediate combat and recovery outrank routine garrison and campaign assignments.
//! All inputs are from the fogged view; forecasts use the engine's combat math.

use crate::analysis::threat_power;
use crate::composition;
use crate::scripted::Planner;
use legatum_core::los;
use legatum_core::path::find_path;
use legatum_core::rules::movement::can_step;
use legatum_core::view::UnitView;
use legatum_core::{Hex, Order, Terrain, city_strike_target, forecast_city_strike};
use legatum_core::{combat, data};

/// Exact mean of the engine's damage rolls, capped at the target's current HP.
pub(crate) fn expected_damage(attack: i32, defense: i32, hp: i32) -> i32 {
    (combat::ROLL_MIN..=combat::ROLL_MAX)
        .map(|roll| combat::damage(attack, defense, roll).min(hp))
        .sum::<i32>()
        / (combat::ROLL_MAX - combat::ROLL_MIN + 1)
}

/// The class bonuses of one fight (core `combat::fight_class_bonus`): the attacker's against the
/// defender, then the defender's against the attacker, when the attacker `shot` or charged. A
/// shot fights as the ranged class, so a Keshig's arrows give a spear line no bonus and a
/// mounted target its "Mounted against ranged"; for every other unit it matches `class_bonus`.
pub(crate) fn fight_bonuses(
    attacker: legatum_core::UnitKind,
    defender: legatum_core::UnitKind,
    shot: bool,
) -> (i32, i32) {
    (
        combat::fight_class_bonus(attacker, defender, true, shot).1,
        combat::fight_class_bonus(defender, attacker, false, shot).1,
    )
}

/// How much of a target with `remaining` HP one shot removes, out of 10,000: a kill (by the
/// forecast's win chance) removes all of it, damage the target survives counts half.
pub(crate) fn removed_share(remaining: i32, f: &legatum_core::BattleForecast) -> i64 {
    let rem = i64::from(remaining.max(1));
    let win = i64::from(f.win_chance.min(100));
    let dealt = i64::from(f.expected_damage_to_defender).min(rem);
    let survived_damage = (100 * dealt - win * rem).max(0);
    win * 100 + survived_damage * 100 / rem / 2
}

/// How much visible enemy pressure an own city absorbs for its garrison this turn. Core
/// (`combat::city_tile_defender`): while the city has HP every attack on its tile hits the city,
/// and damage past 0 HP never carries into the garrison within a stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CityShield {
    /// Not an own city with HP: the unit takes every attack.
    None,
    /// The worst-case fire and melee that can reach the city this turn cannot break it.
    Full,
    /// The city may fall this turn: ranged fire still only hits the city (the garrison is
    /// exposed from the movement rounds on), but melee may reach the garrison.
    MeleeOnly,
}

impl Planner<'_> {
    /// Price a visible rival's city attack with the cover its public unit and era prove.
    fn enemy_city_damage(
        &self,
        e: &UnitView,
        city: &legatum_core::view::CityView,
        attack: i32,
        roll: i32,
    ) -> i32 {
        let tier = u8::from(city.walls) + city.fort_tier();
        let covered = if data::built_in_siege(e.kind) {
            data::MAX_WALL_TIER
        } else if e.siege {
            match self.view().era_of(e.owner) {
                0..=2 => 1,
                3 => 2,
                _ => 3,
            }
        } else {
            0
        };
        let defense = combat::city_defense_vs(
            self.view().rule_experiments,
            self.faction,
            city.population,
            tier,
            covered,
            city.hp,
            self.view().era_of(city.owner),
        )
        .total;
        combat::damage_to_city_hp(
            combat::wall_damage_percent(tier, covered, false),
            combat::damage(attack, defense, roll),
        )
        .min(city.hp)
    }

    pub(crate) fn ranged_unit_clear(&self, u: &UnitView, from: Hex, to: Hex) -> bool {
        // A visible rival gun line proves Steel under the era research gate.
        let steel = if u.owner == self.view().me {
            self.view()
                .research
                .known
                .contains(&legatum_core::TechId::Steel)
        } else {
            matches!(
                u.kind,
                legatum_core::UnitKind::Artillery
                    | legatum_core::UnitKind::RocketArtillery
                    | legatum_core::UnitKind::Drone
            )
        };
        let walled_garrison = self
            .view()
            .cities
            .iter()
            .any(|c| c.owner == u.owner && c.pos == from && c.walls);
        los::ranged_clear_with_tech(u.kind, steel, walled_garrison, from, to, |h| {
            self.view().tile(h).and_then(|t| t.terrain)
        })
    }
    /// What a visible enemy with `hp` left can do to us next turn, in HP of damage: its best
    /// expected hit on one of our units or cities in its reach, plus a city's full HP when its
    /// melee attack could take that city. Shots are chosen by how much of this they remove,
    /// so shooting would-be captors first follows from the stake, not from a unit-type rule.
    /// A garrison is not harm while its city shields it (core `combat::city_tile_defender`).
    pub(crate) fn enemy_threat(&self, e: &UnitView, hp: i32) -> i32 {
        // Barbarians never attack the Celts, so their units threaten nothing of ours.
        if e.owner.is_barbarian() && self.view().barbarians_spare_me() {
            return 0;
        }
        let faction = self.view().faction_of(e.owner);
        let melee = crate::supply::attack_total(faction, e, hp);
        let ranged = (e.ranged > 0).then(|| crate::supply::ranged_total(faction, e, hp));
        // Its charge when it reaches the tile, its shot when the tile is in range: (attack, shot).
        let attacks_on = |pos: Hex| {
            [
                self.threat_reaches_this_turn(e, pos)
                    .then_some((melee, false)),
                ranged
                    .filter(|_| {
                        e.pos.distance(pos) <= e.range && self.ranged_unit_clear(e, e.pos, pos)
                    })
                    .map(|r| (r, true)),
            ]
        };
        let attack_on = |pos: Hex| attacks_on(pos).into_iter().flatten().map(|(a, _)| a).max();
        let unit_defense = |u: &UnitView| {
            let terrain = self
                .view()
                .tile(u.pos)
                .and_then(|t| t.terrain)
                .unwrap_or(Terrain::Grassland);
            crate::supply::defense_total(self.faction, u, u.hp, terrain, u.fortify)
        };
        let mut harm = 0;
        for u in &self.ctx.own_units {
            let in_city = self.ctx.own_cities.iter().any(|c| c.pos == u.pos);
            // Civilians in a city are never targets (a capture takes them: the stake below).
            if in_city && !u.military {
                continue;
            }
            // While the city has HP its fire lands on the city; only a city that can be broken
            // this turn leaves its garrison open, and then only to melee.
            let attacks = match self.city_shield(u.pos) {
                CityShield::Full => continue,
                CityShield::MeleeOnly => [
                    self.threat_reaches_this_turn(e, u.pos)
                        .then_some((melee, false)),
                    None,
                ],
                CityShield::None => attacks_on(u.pos),
            };
            // The class matchup (melee beats mounted, mounted beats ranged) works both ways, and
            // follows the fight: a shot fights as the ranged class (the Keshig's arrows).
            for (attack, shot) in attacks.into_iter().flatten() {
                let (bonus, against) = fight_bonuses(e.kind, u.kind, shot);
                harm = harm.max(expected_damage(
                    attack + bonus,
                    unit_defense(u) + against,
                    u.hp,
                ));
            }
        }
        // Barbarians never assault cities (a public rule).
        if e.owner.is_barbarian() {
            return harm;
        }
        let mut stake = 0;
        for c in &self.ctx.own_cities {
            let Some(attack) = attack_on(c.pos) else {
                continue;
            };
            let expected = (combat::ROLL_MIN..=combat::ROLL_MAX)
                .map(|roll| self.enemy_city_damage(e, c, attack, roll))
                .sum::<i32>()
                / (combat::ROLL_MAX - combat::ROLL_MIN + 1);
            // A point of damage matters more as the city approaches a breach: removing
            // the attacker preserves both that HP and the chance to hold the city next
            // round. This prices the city's actual remaining shield, rather than a
            // fixed preference for mounted or melee targets.
            let shield_value =
                (expected * (c.max_hp + c.hp.max(1)) / c.hp.max(1)).min(data::CITY_HP - 1);
            harm = harm.max(shield_value);
            // Ranged fire alone never captures. Melee captures an ungarrisoned city it breaks.
            // Against a garrisoned city with HP it only hits the city, so it captures only by
            // killing a garrison the city no longer shields: at 0 HP, or when this turn's fire
            // can break the city first (`CityShield::MeleeOnly`; the ranged phase precedes
            // melee, so a broken city's garrison fights the first movement round).
            if !composition::charges(e.kind) || !self.threat_reaches_this_turn(e, c.pos) {
                continue;
            }
            let garrison = self
                .ctx
                .own_units
                .iter()
                .find(|u| u.military && u.pos == c.pos);
            let captures = match garrison {
                None => self.enemy_city_damage(e, c, melee, combat::ROLL_MAX) >= c.hp,
                Some(_) if self.city_shield(c.pos) == CityShield::Full => false,
                Some(g) => combat::damage(melee, unit_defense(g), combat::ROLL_MAX) >= g.hp,
            };
            if captures {
                // Losing a city costs its population and future output as well as its
                // current shield. The same city value is used in assault forecasts.
                stake = 400 + 80 * c.population;
            }
        }
        harm + stake + self.relief_stake(e)
    }

    /// Gene `relief_kill` (lesson 2 of the Fable seat, 2026-11-13): an enemy melee unit that can
    /// step into its owner's city when we assault it (`opportunity::relief_units`) is what
    /// stands between our captor and the city. While one of our melee units can reach that
    /// city, removing it is worth the gene's percent of the city's value (400 + 80 per
    /// population, as in the assault forecasts), scaled by how far the city is broken.
    fn relief_stake(&self, e: &UnitView) -> i32 {
        if self.decisions.relief_kill == 0 || !e.military || e.ranged > 0 {
            return 0;
        }
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| {
                c.visible
                    && c.owner == e.owner
                    && c.pos != e.pos
                    && e.pos.distance(c.pos) <= e.max_moves.max(1)
                    && c.max_hp > 0
                    && self.ctx.own_units.iter().any(|m| {
                        m.military
                            && composition::charges(m.kind)
                            && m.pos.distance(c.pos) <= m.max_moves.max(1)
                    })
            })
            .map(|c| {
                (400 + 80 * c.population) * self.decisions.relief_kill / 100 * (c.max_hp - c.hp)
                    / c.max_hp
            })
            .max()
            .unwrap_or(0)
    }

    /// The threat a ranged shot removes (see `enemy_threat`), for an enemy with `remaining` HP
    /// after shots already ordered this turn. A kill removes all of it; damage it survives
    /// counts half, since the unit still acts next turn, weakened and closer to death.
    /// Sparta regiments heal only by kills and captures: a kill restores up to 100 HP to
    /// the units that damaged the victim. Worth the attacker's missing HP at its kill chance,
    /// in `shot_value` units (a whole kill against a 20-strength threat is 200,000).
    pub(crate) fn sparta_kill_heal_value(
        &self,
        u: &UnitView,
        f: &legatum_core::BattleForecast,
    ) -> i64 {
        if self.faction != legatum_core::Faction::Sparta {
            return 0;
        }
        i64::from(f.win_chance.min(100)) * 20 * i64::from((data::UNIT_HP - u.hp).clamp(0, 100))
    }

    pub(crate) fn flower_wars_value(
        &self,
        u: &UnitView,
        e: &UnitView,
        f: &legatum_core::BattleForecast,
    ) -> i32 {
        if self.faction != legatum_core::Faction::Aztecs
            || !self.rule_on(legatum_core::experiments::FactionRule::AztecsFlowerWars)
            || e.owner.is_barbarian()
            || e.kind == legatum_core::UnitKind::Scout
            || !e.military
            || !crate::composition::is_melee_line(u.kind)
        {
            return 0;
        }
        let food = self
            .ctx
            .own_cities
            .iter()
            .min_by_key(|c| (c.pos.distance(e.pos), c.id))
            .and_then(|c| c.detail.as_ref())
            .map_or(0, |d| d.growth_threshold);
        food * f.win_chance.min(100) as i32 * (100 - f.loss_chance.min(100)) as i32 / 10_000
            * self.decisions.flower_wars_value
            / 100
    }

    /// Visible armies that can feed Flower Wars, priced only from information this seat has.
    pub(crate) fn flower_wars_campaign_value(&self, city: &legatum_core::view::CityView) -> i32 {
        if self.faction != legatum_core::Faction::Aztecs
            || !self.rule_on(legatum_core::experiments::FactionRule::AztecsFlowerWars)
        {
            return 0;
        }
        let captors = self
            .ctx
            .own_units
            .iter()
            .filter(|u| crate::composition::is_melee_line(u.kind))
            .count();
        let prey = self
            .view()
            .units
            .iter()
            .filter(|u| {
                u.owner == city.owner
                    && u.kind != legatum_core::UnitKind::Scout
                    && (crate::composition::is_melee_line(u.kind)
                        || crate::composition::is_mounted(u.kind))
                    && u.pos.distance(city.pos) <= 6
            })
            .count()
            .min(captors);
        let food = self
            .ctx
            .own_cities
            .iter()
            .min_by_key(|c| (c.pos.distance(city.pos), c.id))
            .and_then(|c| c.detail.as_ref())
            .map_or(0, |d| d.growth_threshold);
        prey as i32 * food * self.decisions.flower_wars_value / 100
    }

    pub(crate) fn shot_value(
        &self,
        e: &UnitView,
        remaining: i32,
        f: &legatum_core::BattleForecast,
    ) -> i64 {
        removed_share(remaining, f) * i64::from(self.enemy_threat(e, remaining).max(1))
    }

    /// A shot at a hostile city's tile while the city has HP lands on the city and leaves any
    /// garrison untouched (core `combat::city_tile_defender`), so its value is progress toward
    /// a capture: the share of the city's `remaining` HP it removes (scaled like `shot_value`),
    /// times the city's full HP when one of our melee units is close enough to take it, and
    /// next to nothing otherwise.
    pub(crate) fn city_shot_value(
        &self,
        city: Hex,
        remaining: i32,
        f: &legatum_core::BattleForecast,
    ) -> i64 {
        let captor_near = self.ctx.own_units.iter().any(|m| {
            m.military && composition::charges(m.kind) && m.pos.distance(city) <= m.max_moves.max(1)
        });
        let stake = if captor_near { data::CITY_HP } else { 1 };
        removed_share(remaining, f) * i64::from(stake)
    }

    /// Whether this own city absorbs every attack on its garrison this turn (`CityShield`).
    pub(crate) fn city_shield(&self, pos: Hex) -> CityShield {
        let Some(city) = self
            .ctx
            .own_cities
            .iter()
            .find(|c| c.pos == pos && c.hp > 0)
        else {
            return CityShield::None;
        };
        let mut worst = 0;
        for e in self.ctx.threat_units.iter().filter(|e| e.military) {
            let faction = self.view().faction_of(e.owner);
            // Only volleys that can actually cross the terrain count toward breaking
            // the city shield and exposing its garrison this turn.
            let shot = (e.ranged > 0
                && e.pos.distance(pos) <= e.range
                && self.ranged_unit_clear(e, e.pos, pos))
            .then(|| {
                crate::supply::ranged_total(faction, e, e.hp) + data::siege_city_bonus(e.kind)
            });
            let charge = || {
                (composition::charges(e.kind) && self.threat_reaches_this_turn(e, pos))
                    .then(|| crate::supply::attack_total(faction, e, e.hp))
            };
            // A unit with both (the Keshig) uses the stronger; every other unit has one.
            let attack = if composition::dual(e.kind, e.ranged) {
                shot.max(charge())
            } else {
                shot.or_else(charge)
            };
            let Some(attack) = attack else {
                continue;
            };
            worst += self.enemy_city_damage(e, city, attack, combat::ROLL_MAX);
        }
        if worst < city.hp {
            CityShield::Full
        } else {
            CityShield::MeleeOnly
        }
    }

    /// Shared by retreat and routine defense assignments. A unit already garrisoning a
    /// city holds it (the city absorbs attacks for it while it has HP, as in Civ 6) unless
    /// it is doomed for nothing. Field troops and arriving defenders use the retreat-before-worst-case
    /// forecast; moving discards fortification, so arrivals cannot borrow a holding bonus.
    pub(crate) fn can_defend_at(&self, u: &UnitView, pos: Hex) -> bool {
        if u.military
            && u.pos == pos
            && self.exposure_at(pos) > 0
            && self.ctx.own_cities.iter().any(|c| c.pos == pos)
        {
            return !self.garrison_doomed(u);
        }
        self.exchange_is_sustainable(u, pos)
    }

    /// The retreat-before-worst-case forecast for field units and arriving defenders:
    /// they avoid exposure unless the exchange is sustainable or they alone can stop an
    /// immediate capture. A unit already garrisoning the city is judged by
    /// `garrison_doomed` instead.
    pub(crate) fn exchange_is_sustainable(&self, u: &UnitView, pos: Hex) -> bool {
        self.defensive_exchange(u, pos, false)
    }

    fn defensive_exchange(&self, u: &UnitView, pos: Hex, counterfire_arrival: bool) -> bool {
        if self.exposure_at(pos) == 0 {
            return true;
        }
        let city = self.ctx.own_cities.iter().find(|c| c.pos == pos);
        if city.is_none() && !counterfire_arrival {
            return false;
        }
        // A garrison is safe while its city cannot be broken this turn, even when wounded.
        let shield = if u.military {
            self.city_shield(pos)
        } else {
            CityShield::None
        };
        if shield == CityShield::Full {
            return true;
        }
        if u.hp < self.decisions.recover_hp {
            return false;
        }
        let terrain = self
            .view()
            .tile(pos)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense_at = |hp| {
            crate::supply::defense_total(
                self.faction,
                u,
                hp,
                terrain,
                if u.pos == pos { u.fortify } else { 0 },
            )
        };
        let defense = defense_at(u.hp);
        let city_defense = city.map_or(0, |city| {
            combat::city_defense_for_rules(
                self.view().rule_experiments,
                self.faction,
                city.population,
                city.walls,
                city.fort_tier(),
                city.hp,
                self.view().era_of(city.owner),
            )
            .total
        });
        let mut incoming = 0;
        let mut worst_incoming = 0;
        let mut attacks = Vec::new();
        let mut retaliation = 0;
        let mut counterfire = 0;
        let mut capture_damage = 0;
        for e in self.ctx.threat_units.iter().filter(|e| e.military) {
            let faction = self.view().faction_of(e.owner);
            let melee = self.threat_reaches_this_turn(e, pos);
            let ranged = e.ranged > 0
                && e.pos.distance(pos) <= e.range
                && shield != CityShield::MeleeOnly
                && self.ranged_unit_clear(e, e.pos, pos);
            // Its most damaging attack here, with the class bonus of that fight (a shot fights as
            // the ranged class: the Keshig's arrows differ from its charge).
            let attack = [
                melee.then(|| (crate::supply::attack_total(faction, e, e.hp), false)),
                ranged.then(|| (crate::supply::ranged_total(faction, e, e.hp), true)),
            ]
            .into_iter()
            .flatten()
            .map(|(attack, shot)| {
                let (bonus, against) = fight_bonuses(e.kind, u.kind, shot);
                let (attack, defense) = (attack + bonus, defense + against);
                (
                    (expected_damage(attack, defense, u.hp), attack, shot),
                    defense,
                )
            })
            .max_by_key(|(key, _)| *key);
            let Some(((_, attack, ranged), defense)) = attack else {
                continue;
            };
            incoming += expected_damage(attack, defense, u.hp);
            let worst_damage = combat::damage(attack, defense, combat::ROLL_MAX);
            worst_incoming += worst_damage;
            attacks.push((attack, worst_damage));
            if !ranged {
                retaliation += expected_damage(defense, attack, e.hp);
            }
            if melee && composition::charges(e.kind) {
                capture_damage += combat::damage(attack, city_defense, combat::ROLL_MAX);
            }
            if u.ranged > 0
                && pos.distance(e.pos) <= u.range
                && self.ranged_unit_clear(u, pos, e.pos)
            {
                let enemy_terrain = self
                    .view()
                    .tile(e.pos)
                    .and_then(|t| t.terrain)
                    .unwrap_or(Terrain::Grassland);
                let enemy_defense =
                    crate::supply::defense_total(faction, e, e.hp, enemy_terrain, e.fortify)
                        + fight_bonuses(u.kind, e.kind, true).1;
                // One volley, even when several enemies could attack the city.
                counterfire = counterfire.max(expected_damage(
                    crate::supply::ranged_total(self.faction, u, u.hp)
                        + fight_bonuses(u.kind, e.kind, true).0,
                    enemy_defense,
                    e.hp,
                ));
            }
        }
        for city in &self.ctx.enemy_cities {
            if let Some(shot) = forecast_city_strike(self.view(), city, u, pos) {
                incoming += shot.expected_damage(u.hp);
                let worst = shot.damage_at_roll(combat::ROLL_MAX);
                worst_incoming += worst;
                attacks.push((shot.city_strength.total, worst));
            }
        }
        // Bound wounds across possible attack rounds without assuming hidden orders.
        // Each pass allows one more earlier attack; n hits need at most n - 1 passes.
        // Use the previous bounds for all OTHER attackers: a unit attacks only once.
        // This overestimates simultaneous hits, but cannot pick a favorable hit order.
        for _ in 1..attacks.len() {
            if worst_incoming >= u.hp {
                return false;
            }
            let previous_incoming = worst_incoming;
            worst_incoming = attacks
                .iter_mut()
                .map(|(attack, damage)| {
                    let hp_before = u.hp - (previous_incoming - *damage);
                    *damage = combat::damage(*attack, defense_at(hp_before), combat::ROLL_MAX);
                    *damage
                })
                .sum();
            if worst_incoming == previous_incoming {
                break;
            }
        }
        // A city's HP matters when a surviving garrison can prevent immediate capture.
        worst_incoming < u.hp
            // Movement precedes next turn's counterfire. A relief unit must survive
            // that exposure healthy enough to shoot, without assuming an arrival fortify bonus.
            && (!counterfire_arrival || u.hp - worst_incoming >= self.decisions.recover_hp)
            && (incoming * 100
                <= (retaliation + counterfire) * self.decisions.hold_exchange_percent
                || city.is_some_and(|c| capture_damage > 0 && capture_damage >= c.hp))
    }

    /// Put ranged relief in a position from which it can answer a visible bombarder
    /// next turn. Existing garrisons keep holding; only free field troops move.
    pub(crate) fn answer_bombardment(&mut self, army: &[&UnitView]) {
        let cities = self.ctx.own_cities.clone();
        for city in cities {
            let mut bombarding: Vec<_> = self
                .ctx
                .threat_units
                .iter()
                .copied()
                .filter(|e| e.ranged > 0 && e.pos.distance(city.pos) <= e.range)
                .collect();
            bombarding.sort_by_key(|e| (e.hp, e.id));
            for enemy in bombarding {
                let mut relief: Vec<_> = self
                    .free(army)
                    .into_iter()
                    .filter(|u| {
                        u.ranged > 0
                            && u.hp >= self.decisions.recover_hp
                            && u.pos.distance(city.pos) <= 3
                            && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                    })
                    .collect();
                relief.sort_by_key(|u| (u.pos.distance(enemy.pos), u.id));
                if let Some((unit, to)) = relief.into_iter().find_map(|u| {
                    let spot = enemy
                        .pos
                        .within(u.range)
                        .into_iter()
                        .filter(|h| {
                            *h != u.pos
                                && *h != enemy.pos
                                && h.distance(city.pos) <= 2
                                && self.free_tile_for(u, *h)
                                && self.outside_remembered_reach(*h)
                                && self.reaches_this_turn(u, *h)
                                && self.defensive_exchange(u, *h, true)
                        })
                        .min_by_key(|h| (self.exposure_key(*h), h.distance(u.pos), *h));
                    spot.map(|to| (u, to))
                }) {
                    self.go_to(unit, to);
                }
            }
        }
    }

    /// A garrison leaves only when the city is certain to fall this turn and staying
    /// would destroy the unit with no gain: even the lowest damage rolls of the visible
    /// attackers that can reach it add up to its HP, and one of them is melee, so the
    /// city is captured whether or not it stays. Anything short of certainty holds.
    pub(crate) fn garrison_doomed(&self, u: &UnitView) -> bool {
        // The city absorbs every attack while it has HP: only a city that can be broken this
        // turn leaves its garrison exposed, and then only to melee.
        let shield = self.city_shield(u.pos);
        if shield == CityShield::Full {
            return false;
        }
        let terrain = self
            .view()
            .tile(u.pos)
            .and_then(|t| t.terrain)
            .unwrap_or(Terrain::Grassland);
        let defense = crate::supply::defense_total(self.faction, u, u.hp, terrain, u.fortify);
        let mut certain = 0;
        let mut capture = false;
        for e in self.ctx.threat_units.iter().filter(|e| e.military) {
            let faction = self.view().faction_of(e.owner);
            let melee = composition::charges(e.kind) && self.threat_reaches_this_turn(e, u.pos);
            let ranged = e.ranged > 0
                && e.pos.distance(u.pos) <= e.range
                && self.ranged_unit_clear(e, e.pos, u.pos);
            let attack = if melee {
                crate::supply::attack_total(faction, e, e.hp)
            } else if ranged && shield == CityShield::None {
                crate::supply::ranged_total(faction, e, e.hp)
            } else {
                continue;
            };
            // Full-HP defense for every hit only understates the damage: a lower bound.
            let (bonus, against) = fight_bonuses(e.kind, u.kind, !melee);
            certain += combat::damage(attack + bonus, defense + against, combat::ROLL_MIN);
            capture |= melee;
        }
        for city in &self.ctx.enemy_cities {
            if city_strike_target(self.view(), city) == Some(u.id)
                && let Some(shot) = forecast_city_strike(self.view(), city, u, u.pos)
            {
                certain += shot.damage_at_roll(combat::ROLL_MIN);
            }
        }
        capture && certain * 100 >= u.hp * self.decisions.garrison_doom_percent
    }

    /// Potential incoming melee: use terrain and the engine's movement budget, not
    /// hex distance alone. Unknown terrain is conservatively cheap; do not rely on
    /// other units staying put to block a threat during simultaneous movement.
    pub(crate) fn threat_reaches_this_turn(&self, e: &UnitView, to: Hex) -> bool {
        if e.pos.distance(to) > e.max_moves.max(1) {
            return false;
        }
        // find_path reads only the map's geometry.
        let map = self.movement_map();
        let cost = |h| {
            let t = self.view().tile(h)?;
            t.terrain.map_or(Some(1), |_| t.move_cost())
        };
        let Some(path) = find_path(map, e.pos, to, cost) else {
            return false;
        };
        let mut moves = e.max_moves;
        for h in path {
            let cost = cost(h).expect("path is passable");
            if !can_step(moves, e.max_moves, cost) {
                return false;
            }
            moves = (moves - cost).max(0);
        }
        true
    }

    /// A conservative exposure estimate, not a combat forecast: visible enemies that could
    /// fire here or close to melee next turn. Unknown units never contribute.
    pub(crate) fn exposure_at(&self, pos: Hex) -> i32 {
        self.exposure_key(pos).0
    }

    /// Prefer fewer/weaker threats, then less distance left to escape their reach.
    pub(crate) fn exposure_key(&self, pos: Hex) -> (i32, i32) {
        let Some(i) = self.view().index(pos) else {
            return self.compute_exposure_key(pos);
        };
        if let Some(key) = self.exposure_memo.borrow().get(i).copied().flatten() {
            return key;
        }
        let key = self.compute_exposure_key(pos);
        let mut memo = self.exposure_memo.borrow_mut();
        if memo.is_empty() {
            memo.resize(self.view().tiles.len(), None);
        }
        memo[i] = Some(key);
        key
    }

    fn compute_exposure_key(&self, pos: Hex) -> (i32, i32) {
        let city_exposure = self
            .ctx
            .enemy_cities
            .iter()
            .fold((0, 0), |(power, depth), c| {
                let remaining = (3 - c.pos.distance(pos)).max(0);
                if legatum_core::city_fires(c) && remaining > 0 {
                    let strength = combat::city_defense_for_rules(
                        self.view().rule_experiments,
                        self.view().faction_of(c.owner),
                        c.population,
                        c.walls,
                        c.fort_tier(),
                        c.hp,
                        self.view().era_of(c.owner),
                    )
                    .total;
                    (power + strength, depth + remaining)
                } else {
                    (power, depth)
                }
            });
        let units = self.enemy_exposure_key(pos);
        (city_exposure.0 + units.0, city_exposure.1 + units.1)
    }

    pub(crate) fn unit_exposure_key(&self, u: &UnitView, pos: Hex) -> (i32, i32) {
        if u.military && self.city_shield(pos) == CityShield::Full {
            // Its own city takes every attack this turn (core `combat::city_tile_defender`).
            (0, 0)
        } else if u.kind == legatum_core::UnitKind::Settler
            && self.decisions.settler_shelter > 0
            && self.city_shield(pos) == CityShield::Full
        {
            // Gene `settler_shelter`: a civilian in a city is captured only with the city.
            (0, 0)
        } else if u.military {
            self.exposure_key(pos)
        } else {
            self.enemy_exposure_key(pos)
        }
    }

    pub(crate) fn enemy_exposure_key(&self, pos: Hex) -> (i32, i32) {
        let war = self.ctx.threat_units.iter().filter(|e| e.military).fold(
            (0, 0),
            |(power, depth), e| {
                // A reach that cannot cover the distance adds nothing, so skip its (costly)
                // check: `remaining` below is the same either way.
                let distance = e.pos.distance(pos);
                let melee_reach = if e.max_moves + 1 > distance && self.ctx.reachable(e.pos, pos) {
                    e.max_moves
                } else {
                    0
                };
                let ranged_reach = if e.ranged > 0
                    && e.range + 1 > distance
                    && self.ranged_unit_clear(e, e.pos, pos)
                {
                    e.range
                } else {
                    0
                };
                let reach = melee_reach.max(ranged_reach);
                let remaining = (reach + 1 - e.pos.distance(pos)).max(0);
                if remaining > 0 {
                    (power + threat_power(self.view(), e), depth + remaining)
                } else {
                    (power, depth)
                }
            },
        );
        if self.decisions.peace_exposure_percent == 0 {
            return war;
        }
        self.view()
            .units
            .iter()
            .filter(|e| {
                e.military
                    && !self.view().same_team(self.view().me, e.owner)
                    && !self.view().at_war(self.view().me, e.owner)
            })
            .fold(war, |(power, depth), e| {
                let distance = e.pos.distance(pos);
                let reach =
                    if e.ranged > 0 && distance <= e.range && self.ranged_unit_clear(e, e.pos, pos)
                    {
                        e.range
                    } else if distance <= e.max_moves {
                        e.max_moves
                    } else {
                        0
                    };
                if reach == 0 || distance > reach {
                    return (power, depth);
                }
                let chance = self.peace_declaration_percent(e.owner, e.pos)
                    * self.decisions.peace_exposure_percent
                    / 100;
                (
                    power + threat_power(self.view(), e) * chance / 100,
                    depth + (reach + 1 - distance) * chance / 100,
                )
            })
    }

    /// A public, tunable prior rather than a claim to know the rival's hidden orders.
    /// The guarantee is a real legal barrier; proximity, past declarations and a force
    /// farther from home raise the price of standing in that force's reach.
    pub(crate) fn peace_declaration_percent(&self, owner: legatum_core::PlayerId, at: Hex) -> i32 {
        let v = self.view();
        if v.same_team(v.me, owner) || v.at_war(v.me, owner) {
            return 0;
        }
        let relation = v.diplomacy.relations.iter().find(|r| {
            (r.a == v.team && r.b == v.team_of(owner)) || (r.b == v.team && r.a == v.team_of(owner))
        });
        if relation.is_some_and(|r| r.guarantee_until > v.turn) {
            return 0;
        }
        let wars = self
            .opponents
            .and_then(|o| o.rivals.get(&owner))
            .map_or_else(
                || {
                    v.diplomacy
                        .relations
                        .iter()
                        .filter(|r| r.declarers.contains(&v.team_of(owner)))
                        .count() as i32
                },
                |r| r.declared_wars as i32,
            );
        let near_ours = self.ctx.own_cities.iter().map(|c| c.pos.distance(at)).min();
        let near_theirs = v
            .cities
            .iter()
            .filter(|c| c.owner == owner)
            .map(|c| c.pos.distance(at))
            .min();
        let mut chance = 10 + 8 * wars.min(4);
        if let Some(d) = near_ours {
            chance += (8 - d).max(0) * 4;
            if near_theirs.is_some_and(|home| d < home) {
                chance += 15;
            }
            if d <= 8 {
                let invaders: i32 = v
                    .units
                    .iter()
                    .filter(|u| u.owner == owner && u.military && u.pos.distance(at) <= 3)
                    .map(|u| threat_power(v, u))
                    .sum();
                let defenders: i32 = v
                    .units
                    .iter()
                    .filter(|u| v.same_team(v.me, u.owner) && u.military && u.pos.distance(at) <= 3)
                    .map(|u| threat_power(v, u))
                    .sum();
                chance += ((invaders - defenders).max(0) / 2).min(35);
            }
        }
        if self.top_threat_team() == Some(v.team_of(owner)) {
            chance += 15;
        }
        if v.allied(v.me, owner) {
            chance /= 4;
        }
        chance.clamp(0, 90)
    }

    /// Potential fire at an arrival, without assuming another unit will absorb the shot.
    pub(crate) fn city_damage_at(&self, u: &UnitView, pos: Hex) -> i32 {
        self.ctx
            .enemy_cities
            .iter()
            .filter_map(|c| forecast_city_strike(self.view(), c, u, pos))
            .map(|s| s.expected_damage(u.hp))
            .sum::<i32>()
            + self.regen_strike_at(u, pos)
    }

    /// Gene `regen_strike` (lesson 5 of the Fable seat, 2026-11-13: an Arquebusier parked two
    /// tiles from a broken walled city died to its strike after the city healed 20 HP): a
    /// walled enemy city at 0 HP with none of our military units beside it heals `CITY_HEAL`
    /// at the end of the turn and fires at the next turn's ranged phase, before any movement.
    /// A unit standing within its reach is charged that strike at the gene's percent.
    fn regen_strike_at(&self, u: &UnitView, pos: Hex) -> i32 {
        if self.decisions.regen_strike == 0 {
            return 0;
        }
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| {
                c.hp == 0
                    && c.walls
                    && !self
                        .ctx
                        .own_units
                        .iter()
                        .any(|m| m.military && m.id != u.id && m.pos.distance(c.pos) == 1)
                    && pos.distance(c.pos) != 1
            })
            .filter_map(|c| {
                let mut healed = (*c).clone();
                healed.hp = data::CITY_HEAL.min(c.max_hp);
                forecast_city_strike(self.view(), &healed, u, pos)
            })
            .map(|s| s.expected_damage(u.hp) * self.decisions.regen_strike / 100)
            .sum()
    }

    pub(crate) fn max_city_damage_at(&self, u: &UnitView, pos: Hex) -> i32 {
        self.ctx
            .enemy_cities
            .iter()
            .filter_map(|c| forecast_city_strike(self.view(), c, u, pos))
            .map(|s| s.damage_at_roll(combat::ROLL_MAX))
            .sum()
    }

    /// A healthy scout can buy one sighting at range two and escape next turn.
    /// Never march deeper into the ring or assume another unit absorbs its shot.
    pub(crate) fn safe_scout_tile(&self, u: &UnitView, pos: Hex) -> bool {
        self.enemy_exposure_key(pos).0 == 0
            && self.outside_remembered_reach(pos)
            && self.max_city_damage_at(u, pos) < u.hp
            && self
                .ctx
                .enemy_cities
                .iter()
                .all(|c| !legatum_core::city_fires(c) || pos.distance(c.pos) >= 2)
    }

    /// Damage before this turn's movement. All cities choose from the same snapshot.
    pub(crate) fn city_damage_now(&self, u: &UnitView) -> i32 {
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| city_strike_target(self.view(), c) == Some(u.id))
            .filter_map(|c| forecast_city_strike(self.view(), c, u, u.pos))
            .map(|s| s.expected_damage(u.hp))
            .sum()
    }

    /// Pay the forecast's city shot only when this unit is its selected target. Other
    /// hostile cities may also fire; the attacked city's shot must not be counted twice.
    pub(crate) fn attack_city_damage(&self, u: &UnitView, f: &legatum_core::BattleForecast) -> i32 {
        let forecast_shot = f.city_strike.as_ref().filter(|s| {
            self.ctx
                .enemy_cities
                .iter()
                .any(|c| c.id == s.city && city_strike_target(self.view(), c) == Some(u.id))
        });
        forecast_shot.map_or(0, |s| s.expected_damage(u.hp))
            + self
                .ctx
                .enemy_cities
                .iter()
                .filter(|c| {
                    forecast_shot.is_none_or(|s| s.city != c.id)
                        && city_strike_target(self.view(), c) == Some(u.id)
                })
                .filter_map(|c| forecast_city_strike(self.view(), c, u, u.pos))
                .map(|s| s.expected_damage(u.hp))
                .sum::<i32>()
    }

    /// Melee forecasts assume arrival. Check a known, unoccupied route and the engine's
    /// movement budget separately, including its full-budget expensive-first-step rule.
    pub(crate) fn reaches_this_turn(&self, u: &UnitView, to: Hex) -> bool {
        self.known_path_this_turn(u, to).is_some()
    }

    pub(crate) fn known_path_this_turn(&self, u: &UnitView, to: Hex) -> Option<Vec<Hex>> {
        if u.pos.distance(to) > u.max_moves.max(1) {
            return None;
        }
        // Only the shape is needed: the cost closure supplies fogged terrain/occupancy.
        // find_path reads only the map's geometry.
        let path = find_path(self.movement_map(), u.pos, to, |h| {
            let i = self.view().index(h)?;
            self.ctx.move_cost(h).filter(|_| {
                !(u.military && self.mil_dest.contains(&h))
                    && !self.friendly_blocks(u, h)
                    && (h == to || !self.ctx.hostile_at[i])
            })
        })?;
        // Resolution replenishes moves before orders; moves_left describes last turn.
        let mut moves = u.max_moves;
        for &h in &path {
            let cost = self.ctx.move_cost(h).expect("known path");
            if !can_step(moves, u.max_moves, cost) {
                return None;
            }
            moves = (moves - cost).max(0);
        }
        Some(path)
    }

    /// A friendly garrison is a legal destination for a supporter during an assault.
    pub(crate) fn support_path_this_turn(&self, u: &UnitView, to: Hex) -> Option<Vec<Hex>> {
        if u.pos.distance(to) > u.max_moves.max(1) {
            return None;
        }
        let path = find_path(self.movement_map(), u.pos, to, |h| {
            let i = self.view().index(h)?;
            self.ctx.move_cost(h).filter(|_| {
                (h == to || !self.friendly_blocks(u, h)) && (h == to || !self.ctx.hostile_at[i])
            })
        })?;
        let mut moves = u.max_moves;
        for &h in &path {
            let cost = self.ctx.move_cost(h)?;
            if !can_step(moves, u.max_moves, cost) {
                return None;
            }
            moves = (moves - cost).max(0);
        }
        Some(path)
    }

    fn friendly_blocks(&self, u: &UnitView, h: Hex) -> bool {
        if u.military {
            self.view()
                .index(h)
                .is_some_and(|i| self.ctx.friendly_military_at[i])
        } else {
            // Civilians can shelter with a friendly garrison, but not another civilian.
            self.view()
                .units
                .iter()
                .any(|other| other.pos == h && !other.military)
        }
    }

    pub(crate) fn recovery_key(
        &self,
        u: &UnitView,
        h: Hex,
    ) -> ((i32, i32), i32, i32, i32, i32, Hex) {
        let city = self.ctx.own_cities.iter().any(|c| c.pos == h);
        let home = self
            .view()
            .tile(h)
            .is_some_and(|t| t.owner == Some(u.owner));
        (
            self.unit_exposure_key(u, h),
            self.city_damage_at(u, h),
            -i32::from(city),
            -i32::from(home),
            self.ctx
                .nearest_own_city(h)
                .map_or(0, |c| c.pos.distance(h)),
            h,
        )
    }

    /// Pick an arrival this turn, not a distant home whose first step is still exposed.
    pub(crate) fn safer_tile(&self, u: &UnitView) -> Option<Hex> {
        let current = self.unit_exposure_key(u, u.pos);
        let mut spots: Vec<_> = u
            .pos
            .within(u.max_moves.max(1))
            .into_iter()
            .filter(|h| {
                *h != u.pos
                    && self.ctx.reachable(u.pos, *h)
                    && !(u.military && self.mil_dest.contains(h))
                    && !self.friendly_blocks(u, *h)
                    && self
                        .view()
                        .index(*h)
                        .is_some_and(|i| !self.ctx.hostile_at[i])
            })
            .filter(|h| self.unit_exposure_key(u, *h) < current)
            .collect();
        spots.sort_by_key(|h| self.recovery_key(u, *h));
        spots.into_iter().find(|h| self.reaches_this_turn(u, *h))
    }

    pub(crate) fn retreat(&mut self, u: &UnitView) -> bool {
        if self.unit_exposure_key(u, u.pos).0 > 0
            && let Some(to) = self.safer_tile(u)
        {
            if u.kind == legatum_core::UnitKind::Settler {
                self.retreat_settler(u, to);
            } else if u.military {
                self.go_to(u, to);
            } else {
                self.move_or_hold(u, to);
            }
            self.retreats.insert(u.id);
            return true;
        }
        false
    }

    pub(crate) fn recover_wounded(&mut self, army: &[&UnitView]) {
        let recover_hp = self.decisions.recover_hp;
        for u in self.free(army).into_iter().filter(|u| u.hp < recover_hp) {
            if self.exposure_at(u.pos) == 0 {
                let home = self
                    .view()
                    .tile(u.pos)
                    .is_some_and(|t| t.owner == Some(u.owner));
                // Heal here, or in a safe city reachable this turn.
                let city = self.ctx.own_cities.iter().find(|c| {
                    !home
                        && self.free_tile_for(u, c.pos)
                        && self.exposure_at(c.pos) == 0
                        && self.reaches_this_turn(u, c.pos)
                });
                if let Some(c) = city {
                    self.go_to(u, c.pos);
                } else {
                    self.hold_or_upgrade(u);
                }
            } else if self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                && self.can_defend_at(u, u.pos)
            {
                // A wounded garrison still holds its city (and heals there). Leave it
                // free so it can still fire or sortie; later phases keep it in place.
            } else if let Some(to) = self.safer_tile(u) {
                self.go_to(u, to);
                self.retreats.insert(u.id);
            }
            // If trapped, retain the option to fire or finish a favorable melee attack.
        }
    }

    /// Healthy melee defenders must get this choice before a garrison can reserve them.
    /// Exact kill/loss chances and damage replace a raw strength comparison for this strike.
    pub(crate) fn counterattack(&mut self, army: &[&UnitView]) {
        for u in self
            .free(army)
            .into_iter()
            .filter(|u| composition::charges(u.kind))
        {
            if let Some(to) = self.counterattack_target(u, None) {
                self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
            }
        }
    }

    /// A favorable sortie can end bombardment even when the garrison needs healing.
    pub(crate) fn sortie_garrisons(&mut self, army: &[&UnitView]) {
        for u in self
            .free(army)
            .into_iter()
            .filter(|u| composition::charges(u.kind))
        {
            if self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                && let Some(to) = self.counterattack_target(u, Some(u.pos))
                && self.charge_over_shot(u, to)
            {
                self.unit_order(u.id, Order::MoveUnit { unit: u.id, to });
            }
        }
    }

    /// From a city, consider any attacker able to damage or enter it now.
    pub(crate) fn counterattack_target(&self, u: &UnitView, city: Option<Hex>) -> Option<Hex> {
        let mut targets = Vec::new();
        for e in &self.ctx.enemy_units {
            if !e.military
                    || u.pos.distance(e.pos) > u.max_moves.max(1)
                    || city.is_some_and(|h| {
                        e.pos.distance(h) > 1
                            && (e.ranged <= 0 || e.pos.distance(h) > e.range)
                    })
                    // Storming a city still needs the campaign's local force comparison.
                    || self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos)
            {
                continue;
            }
            let Some(f) = self.melee_forecast(u, e.pos) else {
                continue;
            };
            let city_damage = self.attack_city_damage(u, &f);
            let favorable = city_damage + f.expected_damage_to_attacker < u.hp
                && self.melee_reply_pays(u, e, &f)
                && f.loss_chance <= self.decisions.counterattack_loss_percent as u32
                && (f.win_chance >= self.decisions.counterattack_win_percent as u32
                    || self.faction == legatum_core::Faction::Sparta
                        && u.hp < data::UNIT_HP
                        && f.win_chance >= 50
                    || f.expected_damage_to_defender * 100
                        >= (f.expected_damage_to_attacker + city_damage) * self.attack_threshold());
            if favorable {
                targets.push((
                    (
                        std::cmp::Reverse(self.enemy_threat(e, e.hp)),
                        std::cmp::Reverse(f.win_chance as i32 + self.flower_wars_value(u, e, &f)),
                        std::cmp::Reverse(f.expected_damage_to_defender * 100 / e.hp.max(1)),
                        e.hp,
                        e.id,
                    ),
                    e.pos,
                ));
            }
        }
        targets.sort_by_key(|(key, _)| *key);
        targets
            .into_iter()
            .find(|(_, h)| self.reaches_this_turn(u, *h))
            .map(|(_, h)| h)
    }

    /// A strike that wins a duel can still lose the attacker when it advances into a screen.
    pub(crate) fn melee_reply_pays(
        &self,
        u: &UnitView,
        e: &UnitView,
        f: &legatum_core::BattleForecast,
    ) -> bool {
        if self.decisions.melee_response_weight == 0 {
            return true;
        }
        let my_cost = self
            .view()
            .rule_experiments
            .unit_stats_for(self.faction, u.kind)
            .cost
            .max(1);
        let their_cost = self
            .view()
            .rule_experiments
            .unit_stats_for(self.view().faction_of(e.owner), e.kind)
            .cost
            .max(1);
        let hp_after = (u.hp - f.expected_damage_to_attacker).max(1);
        // While the city shield stands the strike damages the city, not its garrison.
        // That garrison remains alive to answer from the tile on the next turn.
        let shield = self
            .ctx
            .enemy_cities
            .iter()
            .find(|c| c.pos == e.pos && c.hp > 0);
        let removed = if shield.is_some() { None } else { Some(e.id) };
        let reply_here = self.incoming_at(u, e.pos, hp_after, removed);
        let reply_pursuit = e
            .pos
            .neighbors()
            .into_iter()
            .filter(|h| {
                self.view()
                    .tile(*h)
                    .and_then(|t| t.terrain)
                    .is_some_and(|t| t.move_cost().is_some())
            })
            .map(|h| self.incoming_at(u, h, hp_after, removed))
            .max()
            .unwrap_or(0)
            / 2;
        let reply = reply_here.max(reply_pursuit);
        let loss = if reply >= hp_after {
            my_cost * 2
        } else {
            reply * my_cost / 100
        };
        let duel_loss = f.expected_damage_to_attacker * my_cost / u.hp.max(1);
        let prize = if let Some(city) = shield {
            f.expected_damage_to_defender.min(city.hp) * my_cost / city.hp.max(1)
        } else {
            f.expected_damage_to_defender.min(e.hp) * their_cost / e.hp.max(1)
        };
        (prize + self.flower_wars_value(u, e, f)) * 100
            >= (loss + duel_loss) * self.decisions.melee_response_weight
    }

    pub(crate) fn avoid_free_damage(&mut self, army: &[&UnitView]) {
        for u in self.free(army) {
            // Relieving an attacked city means standing in its attackers' reach.
            let relieving = u.hp >= self.decisions.recover_hp
                && self
                    .relief
                    .iter()
                    .any(|c| c.distance(u.pos) <= self.decisions.relieve_radius);
            if !relieving && !self.can_defend_at(u, u.pos) {
                self.retreat(u);
            }
        }
    }
}

#[cfg(test)]
mod bombardment_tests {
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder},
    };
    use legatum_core::{Faction, UnitId, UnitKind};

    #[test]
    fn regen_strike_charges_a_tile_near_a_broken_walled_city_that_will_heal() {
        let city = Hex::new(10, 8);
        let mut view = ViewBuilder::new(24, 16)
            .city(9, ENEMY, city, 3)
            .walled(9)
            .unit(1, ME, UnitKind::Ranger, Hex::new(8, 8))
            .build();
        view.cities[0].hp = 0;
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        let u = p.ctx.own_units[0];
        assert_eq!(p.city_damage_at(u, u.pos), 0, "a broken city does not fire");
        p.decisions.regen_strike = 100;
        assert!(
            p.city_damage_at(u, u.pos) > 0,
            "it heals and fires next turn"
        );
        let mut blocked = view.clone();
        let mut beside = blocked.units[0].clone();
        beside.id = UnitId(2);
        beside.pos = Hex::new(9, 8);
        blocked.units.push(beside);
        let mut p = Planner::new(
            &blocked,
            Knobs::for_faction(Faction::Athens),
            Faction::Athens,
            7,
        );
        p.decisions.regen_strike = 100;
        let u = p.ctx.own_units.iter().find(|u| u.id == UnitId(1)).unwrap();
        assert_eq!(
            p.city_damage_at(u, u.pos),
            0,
            "a unit beside the city stops the heal"
        );
    }

    #[test]
    fn spear_can_plan_a_ranged_charge_on_a_stationary_horse() {
        let target = Hex::new(10, 8);
        let view = ViewBuilder::new(24, 16)
            .unit(1, ME, UnitKind::Soldier, Hex::new(8, 8))
            .unit_hp(50, ENEMY, UnitKind::Horseman, target, 1)
            .build();
        let p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        assert_eq!(
            p.counterattack_target(p.ctx.own_units[0], None),
            Some(target)
        );
    }

    #[test]
    fn a_winning_duel_is_rejected_when_the_spear_screen_takes_the_attacker() {
        let view = ViewBuilder::new(24, 16)
            .unit(1, ME, UnitKind::Horseman, Hex::new(8, 8))
            .unit_hp(50, ENEMY, UnitKind::Soldier, Hex::new(9, 8), 1)
            .unit(51, ENEMY, UnitKind::Phalangite, Hex::new(10, 7))
            .unit(52, ENEMY, UnitKind::Phalangite, Hex::new(10, 9))
            .build();
        let p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        let horse = p.ctx.own_units[0];
        assert_ne!(p.counterattack_target(horse, None), Some(Hex::new(9, 8)));
    }

    #[test]
    fn melee_capture_stake_follows_the_city_shield() {
        // Core `combat::city_tile_defender`: melee captures an ungarrisoned city it breaks, but
        // a garrisoned city with HP only takes the hit, so a capture needs the garrison exposed
        // (the city broken first) and killed.
        let city = Hex::new(8, 8);
        let build = |city_hp: i32, garrison_hp: Option<i32>, enemy_ranger: bool| {
            let mut b = ViewBuilder::new(24, 16).city(1, ME, city, 2).unit(
                50,
                ENEMY,
                UnitKind::Soldier,
                Hex::new(9, 8),
            );
            if let Some(hp) = garrison_hp {
                b = b.unit_hp(1, ME, UnitKind::Ranger, city, hp);
            }
            if enemy_ranger {
                b = b.unit(51, ENEMY, UnitKind::Ranger, Hex::new(10, 8));
            }
            let mut v = b.build();
            v.cities.iter_mut().find(|c| c.pos == city).unwrap().hp = city_hp;
            v
        };
        let threat = |v: &legatum_core::PlayerView| {
            let p = Planner::new(
                v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            let e = p
                .ctx
                .enemy_units
                .iter()
                .find(|e| e.id == UnitId(50))
                .unwrap();
            p.enemy_threat(e, e.hp)
        };
        // Ungarrisoned and nearly broken: the Soldier takes it.
        assert!(threat(&build(10, None, false)) >= data::CITY_HP);
        // A full-HP garrisoned city: the Soldier's attack only hits the city.
        assert!(threat(&build(200, Some(100), false)) < data::CITY_HP);
        // The enemy Ranger's volley can break the 10-HP city, exposing a 20-HP garrison that
        // the Soldier can kill in the first movement round.
        assert!(threat(&build(10, Some(20), true)) >= data::CITY_HP);
        // The same, but the garrison survives the Soldier's best roll.
        assert!(threat(&build(10, Some(100), true)) < data::CITY_HP);
    }

    #[test]
    fn blocked_volley_does_not_expose_a_city_garrison() {
        let city = Hex::new(8, 8);
        for (terrain, expected) in [
            (Terrain::Plains, CityShield::MeleeOnly),
            (Terrain::Forest, CityShield::Full),
            (Terrain::Mountains, CityShield::Full),
        ] {
            let mut v = ViewBuilder::new(24, 16)
                .city(1, ME, city, 2)
                .unit(1, ME, UnitKind::Ranger, city)
                .unit(50, ENEMY, UnitKind::Ranger, Hex::new(10, 8))
                .terrain(Hex::new(9, 8), terrain)
                .build();
            v.cities[0].hp = 10;
            let p = Planner::new(
                &v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            assert_eq!(p.city_shield(city), expected, "{terrain:?}");
        }
    }

    #[test]
    fn a_shot_at_a_shielded_garrison_is_capture_progress_when_a_captor_is_near() {
        // An enemy city with HP shields its garrison, so a shot at its tile damages the city.
        // With our Soldier beside the city that damage is progress toward a capture and beats
        // a shot at a field unit; with no captor near, the field unit is worth more.
        let city = Hex::new(8, 8);
        let ranger = Hex::new(6, 8);
        let field = Hex::new(6, 10);
        let shot = |captor: bool| {
            let mut b = ViewBuilder::new(24, 16)
                .city(1, ME, Hex::new(2, 8), 2)
                .city(2, ENEMY, city, 2)
                .unit(50, ENEMY, UnitKind::Soldier, city)
                .unit(51, ENEMY, UnitKind::Soldier, field)
                .unit(1, ME, UnitKind::Ranger, ranger);
            if captor {
                b = b.unit(2, ME, UnitKind::Soldier, Hex::new(7, 8));
            }
            let mut v = b.build();
            v.cities.iter_mut().find(|c| c.pos == city).unwrap().hp = 40;
            let mut p = Planner::new(
                &v,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            p.decisions.capitals = 0;
            p.plan_military();
            p.orders.iter().find_map(|o| match *o {
                Order::RangedAttack {
                    unit: UnitId(1),
                    target,
                } => Some(target),
                _ => None,
            })
        };
        assert_eq!(shot(true), Some(city));
        assert_eq!(shot(false), Some(field));
    }

    #[test]
    fn ranged_relief_moves_into_counterfire_without_stripping_the_garrison() {
        // Coach g20 native-1790594686370630000-70283-19, P3 T80-81:
        // a melee garrison cannot safely sortie; nearby ranged relief must close.
        let city = Hex::new(8, 8);
        let enemy = Hex::new(10, 8);
        let view = ViewBuilder::new(24, 16)
            .city(1, ME, city, 2)
            .unit_hp(1, ME, UnitKind::Soldier, city, 20)
            .unit(2, ME, UnitKind::Marksman, Hex::new(7, 7))
            .unit(50, ENEMY, UnitKind::Ranger, enemy)
            .build();
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.plan_military();
        assert!(
            p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit, to }
            if *unit == UnitId(2) && to.distance(enemy) <= 2)),
            "{:?}",
            p.orders
        );
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit, .. } if *unit == UnitId(1)))
        );
    }

    #[test]
    fn ranged_garrison_fires_after_the_early_mongols_garrison_pass() {
        // Playtest-6 (fixture: tests/fixtures/garrison-t32.md): at war with Mongols, the
        // garrison pass runs before ranged fire and its hold used to consume the shot.
        let city = Hex::new(8, 8);
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, city, 2)
            .unit(1, ME, UnitKind::Ranger, city)
            .unit(50, ENEMY, UnitKind::Ranger, Hex::new(10, 8))
            .build();
        view.players[ENEMY.0 as usize].faction = Faction::Mongols;
        for difficulty in [
            legatum_core::AiDifficulty::Easy,
            legatum_core::AiDifficulty::Normal,
            legatum_core::AiDifficulty::Hard,
        ] {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Athens),
                Faction::Athens,
                7,
            );
            p.difficulty = difficulty;
            p.plan_military();
            assert!(
                p.orders.contains(&Order::RangedAttack {
                    unit: UnitId(1),
                    target: Hex::new(10, 8)
                }),
                "{difficulty:?}: {:?}",
                p.orders
            );
        }
    }

    #[test]
    fn wounded_ranged_garrison_still_fires_and_relief_rejects_lethal_crossfire() {
        let city = Hex::new(8, 8);
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, city, 2)
            .unit_hp(1, ME, UnitKind::Marksman, city, 40)
            .unit(50, ENEMY, UnitKind::Ranger, Hex::new(10, 8))
            .build();
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.plan_military();
        assert!(
            p.orders
                .iter()
                .any(|o| matches!(o, Order::RangedAttack { unit, .. } if *unit == UnitId(1)))
        );
        view = ViewBuilder { view }
            .unit(2, ME, UnitKind::Ranger, Hex::new(7, 7))
            .unit(51, ENEMY, UnitKind::Marksman, Hex::new(10, 7))
            .build();
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.plan_military();
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit, to }
            if *unit == UnitId(2) && to.distance(Hex::new(10, 8)) <= 2)),
            "{:?}",
            p.orders
        );
    }
}
