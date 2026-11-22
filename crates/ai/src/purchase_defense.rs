//! Paid reinforcements use the same combat forecasts as the human preview, on observed data.
use legatum_core::combat::{self, CityTileDefender};
use legatum_core::path::find_path;
use legatum_core::rules::movement::can_step;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{AttackMode, Hex, PlayerView, UnitId, UnitKind, forecast_attack_mode};

/// Known path and this turn's replenished movement budget, for either side of a forecast.
pub(crate) fn can_arrive(view: &PlayerView, unit: &UnitView, target: Hex) -> bool {
    if unit.pos == target {
        return true;
    }
    if unit.pos.distance(target) > unit.max_moves {
        return false;
    }
    // find_path reads only the map's geometry; costs come from the closure.
    let map = view.tiles.terrain_map(view.width, view.height);
    let path = find_path(map, unit.pos, target, |h| {
        if h != target
            && view
                .units
                .iter()
                .any(|u| u.military && u.id != unit.id && u.pos == h)
        {
            return None;
        }
        view.tile(h)?.move_cost()
    });
    let Some(path) = path else {
        return false;
    };
    let mut moves = unit.max_moves;
    for h in path {
        let cost = view
            .tile(h)
            .and_then(|t| t.move_cost())
            .expect("known route");
        if !can_step(moves, unit.max_moves, cost) {
            return false;
        }
        moves = (moves - cost).max(0);
    }
    true
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DefenseRisk {
    pub threatened: bool,
    pub falls: bool,
    /// Number of resolution turns until capture; None means it stands through the window.
    pub turns_to_fall: Option<i32>,
    /// The city shield reaches zero; a two-round garrison allowance supplies a production
    /// bound when the visible force alone cannot finish the city.
    pub turns_to_breach: Option<i32>,
    /// The garrison is projected to die (only possible once the city is at 0 HP).
    pub garrison_lost: bool,
}

impl DefenseRisk {
    pub(crate) fn production_deadline(self) -> Option<i32> {
        self.turns_to_fall
            .or_else(|| self.turns_to_breach.map(|turns| turns + 2))
    }

    /// An immediate defender still has value when it delays a capture without stopping the
    /// entire eight-turn siege. A missing fall means it stands past the forecast window.
    pub(crate) fn holds_longer_than(self, before: Self) -> bool {
        match (before.turns_to_fall, self.turns_to_fall) {
            (Some(a), Some(b)) => b > a,
            (Some(_), None) => true,
            _ => false,
        }
    }
}

/// The attacks a city faces this turn (`city_risk_with`), in resolution order: ranged units in
/// range (phase 0), then melee units that can arrive (phase 1), by unit id.
pub(crate) fn city_attacks(
    view: &PlayerView,
    city: &CityView,
    arrive: &dyn Fn(&UnitView, legatum_core::Hex) -> bool,
) -> Vec<(i32, legatum_core::UnitId, AttackMode)> {
    let mut attacks: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.military && view.team_of(u.owner) != view.team)
        // Barbarians never attack the Celts: no attack on a Celtic city comes from one.
        .filter(|u| !(u.owner.is_barbarian() && view.barbarians_spare_me()))
        .filter_map(|u| {
            // One order a turn: a unit with both (the Keshig) that can arrive charges, the
            // attack that can take the city.
            let charges = crate::composition::charges(u.kind) && arrive(u, city.pos);
            if u.ranged > 0
                && u.pos.distance(city.pos) <= u.range
                && !(charges && crate::composition::dual(u.kind, u.ranged))
            {
                Some((0, u.id, AttackMode::Ranged))
            } else if charges {
                Some((1, u.id, AttackMode::Melee))
            } else {
                None
            }
        })
        .collect();
    attacks.sort_by_key(|(phase, id, _)| (*phase, *id));
    attacks
}

/// Project visible ranged fire, then reachable melee attacks, using exact roll forecasts.
/// This is a conditional forecast (enemies must actually attack), never hidden enemy orders.
///
/// Civ 6 (`combat::city_tile_defender`): while the city has HP it takes every attack and its
/// garrison is safe. The ranged phase shares one snapshot, so its fire all hits the city (no
/// overflow); melee after it fights the garrison once the city is at 0 HP, and a melee win over
/// the last layer captures. A ranged garrison fires first, at the attacker it can most likely
/// destroy (the same phase as the enemy fire, so the target still shoots).
/// Takes the city's `city_attacks`.
pub(crate) fn city_risk_with(
    view: &PlayerView,
    city: &CityView,
    defender: Option<UnitView>,
    attacks: &[(i32, legatum_core::UnitId, AttackMode)],
) -> DefenseRisk {
    let mut projected = view.clone();
    city_risk_round(&mut projected, city, defender, attacks)
}

/// The single-turn combat step also drives the bounded survival clock. It keeps the projected
/// HP, casualties, and city state so the next turn's engine forecasts see the siege's attrition.
fn city_risk_round(
    projected: &mut PlayerView,
    city: &CityView,
    defender: Option<UnitView>,
    attacks: &[(i32, legatum_core::UnitId, AttackMode)],
) -> DefenseRisk {
    if attacks.is_empty() {
        return DefenseRisk::default();
    }
    let owner_team = projected.team_of(city.owner);
    if let Some(mut unit) = defender {
        projected.units.retain(|u| {
            u.id != unit.id && !(u.pos == city.pos && u.military && u.owner == city.owner)
        });
        unit.pos = city.pos;
        unit.fortify = 0;
        projected.units.push(unit);
    }
    let guard_at = |projected: &PlayerView| {
        projected.units.iter().position(|u| {
            u.pos == city.pos && u.military && projected.team_of(u.owner) == owner_team
        })
    };
    // The garrison's own shot: melee attackers it would likely destroy never arrive.
    if let Some(i) = guard_at(projected).filter(|i| projected.units[*i].ranged > 0) {
        let shooter = projected.units[i].id;
        let shot = attacks
            .iter()
            .filter(|(phase, _, _)| *phase == 1)
            .filter_map(|(_, enemy, _)| {
                let at = projected.units.iter().find(|u| u.id == *enemy)?.pos;
                let f = forecast_attack_mode(projected, shooter, at, AttackMode::Ranged)?;
                Some((f.win_chance, f.expected_damage_to_defender, *enemy))
            })
            .max_by_key(|(win, damage, enemy)| (*win, *damage, std::cmp::Reverse(*enemy)));
        if let Some((win, damage, enemy)) = shot {
            if win >= 50 {
                projected.units.retain(|u| u.id != enemy);
            } else if let Some(u) = projected.units.iter_mut().find(|u| u.id == enemy) {
                u.hp = (u.hp - damage).max(1);
            }
        }
    }
    let snapshot_hp = city.hp;
    let mut risk = DefenseRisk::default();
    for &(phase, attacker, mode) in attacks {
        let Some(unit) = projected.units.iter().find(|u| u.id == attacker) else {
            continue;
        };
        projected.me = unit.owner;
        projected.team = projected.team_of(unit.owner);
        let Some(forecast) = forecast_attack_mode(projected, attacker, city.pos, mode) else {
            continue;
        };
        let guard = guard_at(projected);
        let city_hp = projected
            .cities
            .iter()
            .find(|c| c.id == city.id)
            .map_or(0, |c| c.hp);
        // Ranged fire is aimed from the phase's snapshot: once it has broken the city, the
        // rest of the volley is wasted rather than carried into the garrison.
        if phase == 0 && snapshot_hp > 0 && city_hp == 0 {
            continue;
        }
        let hits_city =
            combat::city_tile_defender(city_hp, guard.is_some()) == CityTileDefender::City;
        // Exact kill probability accounts for damaged units, terrain, fortification and faction.
        let killed = forecast.win_chance >= 50;
        if hits_city {
            let target = projected
                .cities
                .iter_mut()
                .find(|c| c.id == city.id)
                .expect("own city");
            target.hp = if killed {
                0
            } else {
                (target.hp - forecast.expected_damage_to_defender).max(1)
            };
            risk.threatened |= city.hp - target.hp >= 30 || killed;
        } else if let Some(i) = guard {
            if killed {
                projected.units.remove(i);
                risk.threatened = true;
                risk.garrison_lost = true;
            } else {
                projected.units[i].hp =
                    (projected.units[i].hp - forecast.expected_damage_to_defender).max(1);
            }
        }
        if mode == AttackMode::Melee {
            if let Some(u) = projected.units.iter_mut().find(|u| u.id == attacker) {
                u.hp -= forecast.expected_damage_to_attacker;
            }
            projected.units.retain(|u| u.hp > 0);
        }
        // A melee win over the last layer (an empty city, or the garrison of a fallen one)
        // captures; ranged fire alone can never capture.
        if killed && mode == AttackMode::Melee && (!hits_city || guard.is_none()) {
            risk.falls = true;
            risk.threatened = true;
            break;
        }
    }
    if risk.falls {
        risk.turns_to_fall = Some(1);
    }
    if projected
        .cities
        .iter()
        .any(|c| c.id == city.id && c.hp == 0)
    {
        risk.turns_to_breach = Some(1);
    }
    risk
}

/// Conditional capture clock from visible attackers. Each round uses the engine's exact combat
/// forecast against the projected city HP, Walls and garrison. Units already in sight may march
/// into range, while city fire, counterattacks and resting heal change later rounds. No hidden
/// orders or unseen reinforcements are assumed.
pub(crate) fn city_survival_clock(
    view: &PlayerView,
    city: &CityView,
    defender: Option<UnitView>,
    first_attacks: &[(i32, UnitId, AttackMode)],
    horizon: i32,
) -> DefenseRisk {
    if !view.units.iter().any(|u| {
        u.military
            && view.team_of(u.owner) != view.team_of(city.owner)
            && !(u.owner.is_barbarian() && view.barbarians_spare_me())
            && u.pos.distance(city.pos) <= 8
    }) {
        return DefenseRisk::default();
    }
    let capturer_seen = view.units.iter().any(|u| {
        u.military
            && crate::composition::charges(u.kind)
            && view.team_of(u.owner) != view.team_of(city.owner)
            && u.pos.distance(city.pos) <= 8
    });
    let mut projected = view.clone();
    if let Some(mut unit) = defender {
        projected.units.retain(|u| {
            u.id != unit.id
                && !(u.pos == city.pos && u.military && view.team_of(u.owner) == view.team)
        });
        unit.pos = city.pos;
        unit.fortify = 0;
        projected.units.push(unit);
    }
    let mut result = DefenseRisk::default();
    for round in 0..horizon {
        let Some(current_city) = projected.cities.iter().find(|c| c.id == city.id).cloned() else {
            break;
        };
        let moved = if round > 0 {
            advance_visible_attackers(&mut projected, &current_city)
        } else {
            Vec::new()
        };
        // Cities fire before a unit's attack. The lowest-HP visible attacker in range is
        // targeted, as in the engine; a killed attacker cannot strike this round.
        if current_city.walls
            && current_city.hp > 0
            && let Some(target) = projected
                .units
                .iter()
                .filter(|u| {
                    u.military
                        && projected.team_of(u.owner) != projected.team_of(city.owner)
                        && (1..=2).contains(&u.pos.distance(city.pos))
                })
                .min_by_key(|u| (u.hp, u.id))
                .cloned()
            && let Some(fire) =
                legatum_core::forecast_city_strike(&projected, &current_city, &target, target.pos)
        {
            let damage = fire.expected_damage(target.hp);
            if let Some(unit) = projected.units.iter_mut().find(|u| u.id == target.id) {
                unit.hp -= damage;
            }
            projected.units.retain(|u| u.hp > 0);
        }
        projected.me = city.owner;
        projected.team = view.team_of(city.owner);
        let attacks = if round == 0 {
            first_attacks.to_vec()
        } else {
            city_attacks(&projected, &current_city, &|u, h| {
                can_arrive(&projected, u, h)
            })
        };
        let step = city_risk_round(&mut projected, &current_city, None, &attacks);
        result.threatened |= step.threatened;
        result.garrison_lost |= step.garrison_lost;
        if capturer_seen && result.turns_to_breach.is_none() && step.turns_to_breach.is_some() {
            result.turns_to_breach = Some(round + 1);
        }
        if step.falls {
            result.falls = true;
            result.turns_to_fall = Some(round + 1);
            break;
        }
        if attacks.is_empty()
            && !projected.units.iter().any(|u| {
                u.military
                    && view.team_of(u.owner) != view.team_of(city.owner)
                    && u.pos.distance(city.pos) == 1
            })
            && let Some(c) = projected.cities.iter_mut().find(|c| c.id == city.id)
        {
            c.hp = (c.hp + legatum_core::data::CITY_HEAL).min(c.max_hp);
        }
        // A unit that did not join this round's assault can recover while it approaches.
        for unit in &mut projected.units {
            if unit.military
                && view.team_of(unit.owner) != view.team_of(city.owner)
                && !attacks.iter().any(|(_, id, _)| *id == unit.id)
                && !moved.contains(&unit.id)
            {
                unit.hp =
                    (unit.hp + legatum_core::data::HEAL_FIELD).min(legatum_core::data::UNIT_HP);
            }
        }
    }
    result
}

/// Read-only diagnostic for simulation runners. Returns capture and production deadlines from
/// the same clock Hard uses; the latter allows two garrison rounds after a forecast shield
/// breach when no capture is seen.
pub fn city_siege_times(
    view: &PlayerView,
    city: &CityView,
    horizon: i32,
) -> (Option<i32>, Option<i32>) {
    let attacks = city_attacks(view, city, &|u, h| can_arrive(view, u, h));
    let risk = if horizon <= 1 {
        city_risk_with(view, city, None, &attacks)
    } else {
        city_survival_clock(view, city, None, &attacks, horizon)
    };
    (risk.turns_to_fall, risk.production_deadline())
}

fn advance_visible_attackers(view: &mut PlayerView, city: &CityView) -> Vec<UnitId> {
    let map = view.tiles.terrain_map(view.width, view.height);
    let ids: Vec<UnitId> = view
        .units
        .iter()
        .filter(|u| {
            u.military
                && view.team_of(u.owner) != view.team_of(city.owner)
                && !(u.owner.is_barbarian() && view.barbarians_spare_me())
                && u.pos.distance(city.pos) <= 8
        })
        .map(|u| u.id)
        .collect();
    let mut moved = Vec::new();
    for id in ids {
        let Some(unit) = view.units.iter().find(|u| u.id == id).cloned() else {
            continue;
        };
        let reach = if unit.ranged > 0 { unit.range } else { 1 };
        if unit.pos.distance(city.pos) <= reach {
            continue;
        }
        let route = find_path(map, unit.pos, city.pos, |h| {
            if h != city.pos
                && view
                    .units
                    .iter()
                    .any(|u| u.id != id && u.military && u.pos == h)
            {
                return None;
            }
            view.tile(h)?.move_cost()
        });
        let Some(route) = route else { continue };
        let mut left = unit.max_moves;
        let mut pos = unit.pos;
        for h in route {
            if h == city.pos || pos.distance(city.pos) <= reach {
                break;
            }
            let Some(cost) = view.tile(h).and_then(|t| t.move_cost()) else {
                break;
            };
            if !can_step(left, unit.max_moves, cost) {
                break;
            }
            left = (left - cost).max(0);
            pos = h;
        }
        if let Some(u) = view.units.iter_mut().find(|u| u.id == id) {
            if pos != u.pos {
                u.fortify = 0;
                moved.push(id);
            }
            u.pos = pos;
        }
    }
    moved
}

pub(crate) fn purchased_defender(view: &PlayerView, city: &CityView, kind: UnitKind) -> UnitView {
    let stats = view
        .rule_experiments
        .unit_stats_for(view.faction_of(city.owner), kind);
    UnitView {
        id: UnitId(u32::MAX),
        owner: city.owner,
        kind,
        pos: city.pos,
        hp: legatum_core::data::UNIT_HP,
        moves_left: 0,
        max_moves: crate::analysis::forecast_moves(view, city.owner, kind),
        goto: None,
        path: Vec::new(),
        automation: None,
        needs_orders: false,
        attack_target: None,
        fortify: 0,
        created_turn: Some(view.turn),
        strength: stats.strength,
        ranged: stats.ranged,
        range: stats.range,
        military: stats.military,
        level: 0,
        xp: None,
        xp_next: None,
        occupation_lock: None,
        upgrade_cost: None,
        upgrade_to: None,
        upgrade_steps: Vec::new(),
        upgrade_breakdown: None,
        upgrade_need: None,
        unsupplied: None,
        unsupplied_loss: None,
        upgrade_supply_blocker: None,
        siege: false,
        siege_offer: None,
        sight: None,
        resting_heal: None,
        standing_order: None,
        heir_of: None,
        contract_until: None,
        origin: None,
    }
}

impl crate::scripted::Planner<'_> {
    /// `can_arrive` on this planner's view, memoized for the pass.
    pub(crate) fn arrives(&self, u: &UnitView, target: legatum_core::Hex) -> bool {
        if let Some(hit) = self.arrivals.borrow().get(&(u.id, target)) {
            return *hit;
        }
        let out = can_arrive(self.view(), u, target);
        self.arrivals.borrow_mut().insert((u.id, target), out);
        out
    }

    /// `city_risk` on this planner's view; the attacks are computed once per city per pass.
    pub(crate) fn defense_risk(&self, city: &CityView, defender: Option<UnitView>) -> DefenseRisk {
        let baseline = defender.is_none();
        if baseline && let Some(risk) = self.defense_clocks.borrow().get(&city.id) {
            return *risk;
        }
        let cached = self.city_attacks.borrow().get(&city.id).cloned();
        let attacks = match cached {
            Some(a) => a,
            None => {
                let a =
                    std::rc::Rc::new(city_attacks(self.view(), city, &|u, h| self.arrives(u, h)));
                self.city_attacks.borrow_mut().insert(city.id, a.clone());
                a
            }
        };
        let risk = if self.hard_defense() && self.decisions.siege_survival_clock > 0 {
            city_survival_clock(self.view(), city, defender, &attacks, 8)
        } else {
            city_risk_with(self.view(), city, defender, &attacks)
        };
        if baseline {
            self.defense_clocks.borrow_mut().insert(city.id, risk);
        }
        risk
    }
}

#[cfg(test)]
mod tests {
    use crate::{Knobs, scripted::Planner};
    use legatum_core::{AiDifficulty, CityId, GameState, PlayerId, UnitId, player_view};

    #[test]
    fn gergovia_t29_survival_clock_warns_before_the_recorded_t32_fall() {
        let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../tests/fixtures/gergovia-t29.json.gz")[..],
        ))
        .unwrap();
        assert_eq!(state.turn, 29);
        let view = player_view(&state, PlayerId(5));
        let city = view.cities.iter().find(|c| c.id == CityId(5)).unwrap();
        assert_eq!(city.name, "Gergovia");
        assert!(city.walls);
        let mut old = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(view.me)),
            view.faction_of(view.me),
            7,
        );
        old.difficulty = AiDifficulty::Hard;
        old.decisions.siege_survival_clock = 0;
        old.decisions.production_threat_agreement = 1;
        old.decisions.walled_defense_percent = 100;
        let one = old.defense_risk(city, None);
        assert!(!one.falls, "the former one-turn read should miss the siege");
        let mut new = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(view.me)),
            view.faction_of(view.me),
            7,
        );
        new.difficulty = AiDifficulty::Hard;
        new.decisions.siege_survival_clock = 1;
        new.decisions.production_threat_agreement = 1;
        new.decisions.walled_defense_percent = 100;
        let clock = new.defense_risk(city, None);
        assert!(clock.threatened);
        // Two of the four Spartans that take Gergovia are still in fog on T29. Show the
        // information boundary explicitly, then reveal only these already-present units.
        assert!(
            !clock.falls,
            "unseen reinforcements cannot enter a legal forecast"
        );
        assert!(
            clock.turns_to_breach.is_some_and(|turns| turns < 8),
            "{clock:?}"
        );
        let sparta = player_view(&state, PlayerId(0));
        let mut scouted = view.clone();
        for id in [UnitId(28), UnitId(30)] {
            scouted
                .units
                .push(sparta.units.iter().find(|u| u.id == id).unwrap().clone());
        }
        let city = scouted.cities.iter().find(|c| c.id == CityId(5)).unwrap();
        let mut scouted_planner = Planner::new(
            &scouted,
            Knobs::for_faction(scouted.faction_of(scouted.me)),
            scouted.faction_of(scouted.me),
            7,
        );
        scouted_planner.difficulty = AiDifficulty::Hard;
        scouted_planner.decisions.siege_survival_clock = 1;
        scouted_planner.decisions.production_threat_agreement = 1;
        scouted_planner.decisions.walled_defense_percent = 100;
        let warned = scouted_planner.defense_risk(city, None);
        assert!(
            warned.falls,
            "scouting the existing reinforcements should expose the siege: {warned:?}"
        );
        assert!(
            warned.turns_to_fall.is_some_and(|turns| turns <= 5),
            "{warned:?}"
        );
    }

    #[test]
    fn gergovia_t30_clock_uses_the_newly_visible_reinforcement() {
        let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../tests/fixtures/gergovia-t30.json.gz")[..],
        ))
        .unwrap();
        let view = player_view(&state, PlayerId(5));
        let city = view.cities.iter().find(|c| c.id == CityId(5)).unwrap();
        let mut planner = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(view.me)),
            view.faction_of(view.me),
            7,
        );
        planner.difficulty = AiDifficulty::Hard;
        planner.decisions.siege_survival_clock = 1;
        planner.decisions.production_threat_agreement = 1;
        planner.decisions.walled_defense_percent = 100;
        let risk = planner.defense_risk(city, None);
        assert!(risk.falls, "{risk:?}");
        assert!(
            risk.turns_to_fall.is_some_and(|turns| turns <= 3),
            "{risk:?}"
        );
    }
}
