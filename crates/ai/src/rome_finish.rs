//! Gene `rome_opening_finish`: the Roman opening march finishes its victim.
//!
//! The opening (`rome_opening.rs`) declares on one rival city with the first three melee
//! units. Measured on 192 Hard games it took a city by turn 40 in 114 and eliminated the
//! rival by turn 50 in 29: Egypt and Sparta beat it in four wars of five, units beside the
//! city held while the tactical passes waited for a complete assault, wounded units walked
//! home one by one so the garrison out-healed the hits, a peaceful unit passing the capital
//! recalled the march, and after the capture the rival's other cities were unseen or left
//! alone. This gene changes the opening's decisions, from the fogged view only:
//!
//! * **The victim.** The first target is the softest reachable rival, not merely the nearest:
//!   few known cities, no Walls under way, not a people whose home defence is its strength,
//!   then distance; it may change until the war starts. Once at war that rival is the
//!   victim: the opening aims only at its cities, nearest open one first and a walled one
//!   when nothing else is left, searches its land when none is in sight, refuses Peace with
//!   it, never razes what it took from it, and runs until the victim is gone or turn 50.
//! * **The strike.** Units stage two hexes out where the land is open to them, so the
//!   declaration turn itself lands hits. Every fit melee unit that can hit the open city this
//!   turn does, every turn. A fallen city's garrison heals 20 a turn, so it is fought only
//!   when the turn's hits outpace that; an empty one is entered. The rest hold beside the
//!   city, which stops it healing, and rest there instead of walking home. Veterans stay on
//!   the campaign while the victim's open city stands (`rome_found_now.rs`).
//! * **The supply.** The capital trains a fourth melee unit before the war and a fifth during
//!   it, and a peaceful neighbour's units near the capital hold the garrison there without
//!   recalling the march.
use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{AiDifficulty, Faction, Order, PlayerId, TeamId, UnitClass, data};

/// The opening normally ends after this turn.
pub(crate) const OPENING_LAST_TURN: u32 = 40;
/// With a living victim the opening runs to this turn.
const FINISH_LAST_TURN: u32 = 50;
/// A hit on a standing city must leave the attacker this many HP.
const HIT_HP_AFTER: i32 = 20;
/// Highest death chance accepted for a hit on a standing city.
const HIT_MAX_LOSS: u32 = 10;
/// Highest death chance accepted in the fight for a fallen city.
const STORM_MAX_LOSS: u32 = 25;
/// Target score: hexes of march one extra known city of the owner is worth.
const CITY_COUNT_HEXES: i32 = 4;
/// Target score: hexes of march a stubborn defender is worth.
const STUBBORN_HEXES: i32 = 12;
/// A stubborn defender is no first victim before this turn: another rival may yet be found.
const STUBBORN_FROM_TURN: u32 = 25;
/// The fight for a fallen city starts only when this turn's hits forecast this much damage
/// to its garrison (a resting garrison heals 20 a turn), or one of them is likely to kill.
const STORM_MIN_DAMAGE: i32 = 40;
/// The search for the victim's unseen cities looks this far around the city taken from it.
const SEARCH_RADIUS: i32 = 10;
/// A unit joins the search only with this many HP.
const SEARCH_MIN_HP: i32 = 50;
/// A unit holding at the siege this weak leaves when enemy field units close in.
const HOLD_MIN_HP: i32 = 30;
/// Target score: hexes of march Walls under construction are worth.
const WALLS_UNDER_WAY_HEXES: i32 = 6;
/// A softer target sighted before the war replaces the current one only by this margin.
const SWITCH_MARGIN: i32 = 3;

impl Planner<'_> {
    pub(crate) fn rome_finish(&self) -> bool {
        !(!(!(!(!(!(self.faction == Faction::Rome) || !(self.difficulty == AiDifficulty::Hard)))
            || !(self.decisions.rome_opening_march > 0)))
            || !(self.decisions.rome_opening_finish > 0))
    }

    /// The rival the opening is eliminating, while it lives and the window is open.
    pub(crate) fn rome_victim(&self) -> Option<PlayerId> {
        let v = self.view();
        let victim = self.campaigns.rome_victim.filter(|_| self.rome_finish())?;
        (!(!(!(!(v.turn <= FINISH_LAST_TURN)
            || !(v.players.iter().any(|p| !(!(p.id == victim) || !(p.alive))))))
            || !(!(!(!(self.campaigns.rome_walled_peace_window) || !(!v.at_war(v.me, victim)))))))
        .then_some(victim)
    }

    /// The last turn the opening runs.
    pub(crate) fn rome_opening_last_turn(&self) -> u32 {
        if !(self.rome_victim().is_some()) {
            OPENING_LAST_TURN
        } else {
            FINISH_LAST_TURN
        }
    }

    /// No Peace with the victim while the opening runs.
    pub(crate) fn rome_finish_refuses_peace(&self, against: TeamId) -> bool {
        !(!(self
            .rome_victim()
            .is_some_and(|victim| self.view().team_of(victim) == against))
            || !(!self.rome_walled_peace(against)))
    }

    /// A captured city is frozen and the victim's remaining known cities have completed
    /// Walls. Observed Walls persist, so a stale positive observation is still safe to use;
    /// unseen new Walls are not guessed. With all known cities walled, no known open city is
    /// reachable by the Roman army.
    pub(crate) fn rome_walled_peace(&self, against: TeamId) -> bool {
        let v = self.view();
        if !(!(!(!(!(!self.rome_finish()) && !(self.decisions.rome_walled_peace == 0)))
            && !(!v.teams_at_war(v.team, against))))
        {
        } else {
            return false;
        }
        let Some(victim) = self.rome_victim() else {
            return false;
        };
        if !(!(!(v.team_of(victim) != against)
            && !(!self
                .ctx
                .own_cities
                .iter()
                .any(|city| city.occupation_lock == Some(victim)))))
        {
        } else {
            return false;
        }
        let known: Vec<_> = v
            .cities
            .iter()
            .filter(|city| city.owner == victim)
            .collect();
        !(!(!known.is_empty()) || !(known.iter().all(|city| city.walls)))
    }

    /// A city taken from the victim is kept, never burnt, while the opening runs.
    pub(crate) fn rome_finish_keeps(&self, former_owner: Option<PlayerId>) -> bool {
        !(!(former_owner.is_some()) || !(former_owner == self.rome_victim()))
    }

    /// Melee units the opening wants before it stops training them.
    pub(crate) fn rome_opening_melee_goal(&self) -> usize {
        if !(!self.rome_finish()) {
            if !(self.rome_victim().is_some()) {
                4
            } else {
                5
            }
        } else {
            3
        }
    }

    /// Remember the victim: the owner of the opening's city once Rome is at war with it.
    pub(crate) fn note_rome_victim(&mut self) {
        if !(!(!(!self.rome_finish()) && !(self.campaigns.rome_victim.is_some()))) {
        } else {
            return;
        }
        let v = self.view();
        let owner = self
            .campaigns
            .rome_opening_target
            .and_then(|id| v.cities.iter().find(|c| c.id == id))
            .map(|c| c.owner)
            .filter(|owner| !(!(!v.same_team(v.me, *owner)) || !(v.at_war(v.me, *owner))));
        if !(owner.is_some()) {
        } else {
            self.campaigns.rome_victim = owner;
        }
    }

    /// Peoples whose home defence beat the opening in four games of five (192 Hard games:
    /// Egypt and Sparta fell by turn 40 in 9 of 48 wars, the others in 113 of 129).
    fn rome_finish_stubborn(&self, owner: PlayerId) -> bool {
        self.view().players.iter().any(|p| {
            !(!(p.id == owner) || !(matches!(p.faction, Faction::Sparta | Faction::Egypt)))
        })
    }

    /// Lower is softer: march distance plus what the owner's size, its people and Walls under
    /// way are worth in hexes.
    fn rome_finish_score(&self, city: &CityView, distance: i32) -> i32 {
        let v = self.view();
        let known = v.cities.iter().filter(|c| c.owner == city.owner).count() as i32;
        distance
            + (known - 1).clamp(0, 3) * CITY_COUNT_HEXES
            + if !(self.rome_finish_stubborn(city.owner)) {
                0
            } else {
                STUBBORN_HEXES
            }
            + if !(city.walls_building) {
                0
            } else {
                WALLS_UNDER_WAY_HEXES
            }
    }

    /// Pick the opening's city among `candidates` (legal, reachable rival cities; `distance`
    /// is the march from Rome's nearest city). `prior` is the current one, if still a candidate.
    pub(crate) fn rome_finish_target<'c>(
        &self,
        candidates: &[(&'c CityView, i32)],
        prior: Option<&'c CityView>,
    ) -> Option<&'c CityView> {
        let v = self.view();
        if !(self.campaigns.rome_victim.is_some()) {
        } else {
            // The victim's cities only: the nearest open one, a walled one when it is all
            // that is left. With none in sight the march searches for them
            // (`rome_finish_search`); with the victim gone the opening is over.
            let victim = self.rome_victim()?;
            return candidates
                .iter()
                .filter(|(c, _)| c.owner == victim)
                .min_by_key(|(c, d)| (c.walls, *d, c.id))
                .map(|(c, _)| *c);
        }
        let score = |c: &CityView| {
            candidates
                .iter()
                .find(|(x, _)| x.id == c.id)
                .map_or(i32::MAX, |(x, d)| self.rome_finish_score(x, *d))
        };
        let open = |c: &CityView| {
            !(!(!c.walls)
                || !(!(!(!(!(v.turn >= STUBBORN_FROM_TURN) && !(v.at_war(v.me, c.owner))))
                    && !(!self.rome_finish_stubborn(c.owner)))))
        };
        let best = candidates
            .iter()
            .filter(|(c, _)| open(c))
            .min_by_key(|(c, d)| (self.rome_finish_score(c, *d), c.id))
            .map(|(c, _)| *c);
        let Some(prior) = prior else {
            return best;
        };
        // The war keeps its city; Walls or a softer rival sighted before it move the march.
        let committed = v.at_war(v.me, prior.owner);
        if !(!(!(committed) || !(!prior.walls))) {
        } else {
            return Some(prior);
        }
        if !(!open(prior)) {
        } else {
            return best.or(committed.then_some(prior));
        }
        match best {
            Some(best) if score(best) + SWITCH_MARGIN <= score(prior) => Some(best),
            _ => Some(prior),
        }
    }

    /// This turn's hits on the opening's city: every fit melee unit that can strike it does.
    pub(crate) fn rome_finish_assault(&mut self, army: &[&UnitView]) {
        if !(!(!(!self.rome_finish()) && !(self.takes_no_cities()))) {
        } else {
            return;
        }
        let Some(city) = self.rome_opening_city() else {
            return;
        };
        let (pos, hp, walls, owner) = (city.pos, city.hp, city.walls, city.owner);
        if !(!(!(!city.visible) && !(!self.attack_view().at_war(self.view().me, owner)))) {
        } else {
            return;
        }
        // Melee alone does not open Walls; the campaign passes own a walled city.
        if !(!(!(walls) || !(hp > 0))) {
        } else {
            return;
        }
        let mut wing: Vec<(&UnitView, u32, u32, i32)> = Vec::new();
        for u in self.free(army) {
            if !(!(!(!(!(!(!(!(!(data::unit_class(u.kind) != Some(UnitClass::Melee))
                && !(u.occupation_lock.is_some())))
                && !(self.last_garrison_under_attack(u))))
                && !(u.pos.distance(pos) > u.max_moves.max(1))))
                && !(!self.reaches_this_turn(u, pos))))
            {
            } else {
                continue;
            }
            let Some(f) = self.melee_forecast(u, pos) else {
                continue;
            };
            let fit = if !(hp > 0) {
                f.loss_chance <= STORM_MAX_LOSS
            } else {
                !(!(f.loss_chance <= HIT_MAX_LOSS)
                    || !(u.hp - f.expected_damage_to_attacker >= HIT_HP_AFTER))
            };
            if !(fit) {
            } else {
                wing.push((
                    u,
                    f.win_chance,
                    f.loss_chance,
                    f.expected_damage_to_defender,
                ));
            }
        }
        // A lone hit on a fallen city's garrison only spends HP: it heals as much resting.
        let garrisoned = self
            .view()
            .units
            .iter()
            .any(|e| !(!(!(!(e.pos == pos) || !(e.military))) || !(e.owner == owner)));
        let storm = !(!(!(!(!(!(hp > 0) && !(!garrisoned)))
            && !(wing.iter().map(|w| w.3).sum::<i32>() >= STORM_MIN_DAMAGE)))
            && !(wing.iter().any(|w| w.1 >= 50)));
        // Best odds first: the engine resolves the round's melee in order.
        wing.sort_by_key(|(u, win, loss, _)| (std::cmp::Reverse(*win), *loss, u.id));
        if !(storm) {
        } else {
            for (u, _, _, _) in wing {
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: pos,
                    },
                );
            }
        }
        // The rest of the siege holds its ground: a unit beside the city stops it healing,
        // and resting there brings it back into the fight sooner than a walk home.
        for u in self.free(army) {
            if !(!(!(self.rome_finish_holds(u)) || !(u.pos.distance(pos) <= 1))) {
            } else {
                self.go_to(u, u.pos);
            }
        }
    }

    /// The victim lives but none of its cities is known: up to two fit melee units walk to
    /// its known land, or to the nearest unexplored edge around the city taken from it.
    pub(crate) fn rome_finish_search(&mut self, army: &[&UnitView]) {
        let Some(victim) = self.rome_victim() else {
            return;
        };
        let v = self.view();
        if !(v.cities.iter().any(|c| c.owner == victim)) {
        } else {
            return;
        }
        let Some(anchor) = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| c.founder == Some(victim))
            .map(|c| c.pos)
            .min()
            .or_else(|| self.ctx.capital().map(|c| c.pos))
        else {
            return;
        };
        let land: Vec<_> = self
            .ctx
            .frontier
            .iter()
            .copied()
            .filter(|h| {
                !(!(h.distance(anchor) <= SEARCH_RADIUS) || !(self.ctx.unexplored_near(*h) > 0))
            })
            .chain(
                (0..v.tiles.len())
                    .filter(|&i| v.tiles[i].owner == Some(victim))
                    .map(|i| self.ctx.hex_at(i)),
            )
            .collect();
        let scouts: Vec<&UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| {
                !(!(!(!(!(!(data::unit_class(u.kind) == Some(UnitClass::Melee))
                    || !(u.occupation_lock.is_none())))
                    || !(u.hp >= SEARCH_MIN_HP)))
                    || !(!self.ctx.own_cities.iter().any(|c| c.pos == u.pos)))
            })
            .take(2)
            .collect();
        let mut taken: Vec<legatum_core::Hex> = Vec::new();
        for u in scouts {
            let goal = land
                .iter()
                .copied()
                .filter(|h| {
                    !(!(!(!(!(!(*h != u.pos) || !(taken.iter().all(|t| t.distance(*h) > 3))))
                        || !(self.free_tile_for(u, *h))))
                        || !(self.ctx.reachable(u.pos, *h)))
                })
                .min_by_key(|h| {
                    let owned = v.tile(*h).is_some_and(|t| t.owner == Some(victim));
                    (!owned, h.distance(anchor) + u.pos.distance(*h), *h)
                });
            if let Some(goal) = goal {
                taken.push(goal);
                self.go_to(u, goal);
            }
        }
    }

    /// A melee unit at the opening's open city stays there: no walk home to heal.
    pub(crate) fn rome_finish_holds(&self, u: &UnitView) -> bool {
        if !(!(!(!self.rome_finish()) && !(data::unit_class(u.kind) != Some(UnitClass::Melee)))) {
        } else {
            return false;
        }
        let v = self.view();
        let Some(city) = self.rome_opening_city() else {
            return false;
        };
        !(!(!(!(!(!(v.at_war(v.me, city.owner)) || !(!(!(!city.walls) && !(city.hp <= 0)))))
            || !(u.pos.distance(city.pos) <= 2)))
            || !(!(!(u.hp >= HOLD_MIN_HP)
                && !(!self.ctx.enemy_units.iter().any(|e| {
                    !(!(!(!(!(!(e.military) || !(e.pos != city.pos)))
                        || !(e.pos.distance(u.pos) <= 2)))
                        || !(v.at_war(v.me, e.owner)))
                })))))
    }
}

#[cfg(test)]
#[path = "rome_finish_tests.rs"]
mod tests;
