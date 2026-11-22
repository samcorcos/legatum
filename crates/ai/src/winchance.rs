//! The common estimate behind goal-driven play: this seat's chance to win the VP race, as an
//! integer from 0 to 10,000, read only from its own `PlayerView`.
//!
//! Every alive team gets a score from public facts (VP progress against its target, and its
//! share of cities and public population, economy, army, science, and territory). This seat's
//! own score also carries one private fact it alone knows: how short
//! its treasury runway is. The chance is this team's share of `exp(score)` over all teams
//! (a conditional logit). Unknown rival facts are never synthesized; a rival's private
//! economy simply isn't a term.
//!
//! Winning by VP means holding the target for `data::VP_HOLD_TURNS` turn ends. A team
//! holding it (public countdown) is valued as a probability of surviving its remaining window
//! (`hold_completion`), not as a won game: each turn it breaks with a hazard that falls with its
//! VP margin and rises steeply once a projection takes it below the target. The chance is then that hold's completion, plus the logit
//! share of whatever probability no hold completes.
//!
//! Plans compare projected features ("one more city, two fewer Spearmen, a shorter runway")
//! in the same units, so a war, a peace and a purchase are priced in one currency. The
//! weights are fitted on completed sim games and checked on held-out seeds
//! (`crates/sim/examples/winchance_fit.rs`). Integer math only: weights, shares and the
//! exponential table are fixed-point, so native and WASM agree exactly.

use crate::DecisionParams;
use legatum_core::{PlayerView, TeamId};

/// Bumped whenever features or weights change, so recorded estimates are comparable.
pub const VERSION: u32 = 6;

/// Public per-team features, then this seat's private feature.
pub const FEATURES: [&str; 9] = [
    "VP progress",
    "VP progress late",
    "cities",
    "population",
    "economy",
    "army",
    "science",
    "territory",
    "runway short",
];

/// Milli-logits per permille of each feature (a weight of 3000 means a full share is worth
/// three logits). Fitted by `winchance_fit` on 300 Standard Hard games (seeds 6100001+),
/// checked on held-out Standard seeds (6200001+).
///
/// Fit (2026-10-04, master 0826fbe6, every 5 turns): Brier skill over uniform +26.2% on
/// training and +24.9% on held-out Standard
/// (docs/sim-results/2026-10-04-winchance-calibration.md). The economy share carries the
/// most weight; the runway shortfall has almost none (the reserve ledger guards it instead).
pub const WEIGHTS: [i32; 9] = [1464, 1511, 2979, 2806, 8047, 2480, 528, 3206, -76];

/// Weights for two-team games (1v1), fitted separately: each player count gets its own.
/// Fit (2026-10-06, claude/search-ai 0c37432c, every 5 turns): 512 Small-map Standard Hard
/// duels over all 64 faction pairs (seeds 7300001+), held out on 256 (7400001+). Held-out
/// Brier 0.1858 with `WEIGHTS`, 0.1656 with these (skill over uniform +25.7% → +33.8%).
/// In 1v1 the army and science shares carry almost no weight; economy, late VP progress and
/// cities do.
pub const WEIGHTS_DUEL: [i32; 9] = [986, 2077, 1356, 363, 3801, -68, -184, 876, -67];

/// Per-turn chance (permille) that a hold exactly at its target breaks: an assumption, not yet
/// measured under the current 10-turn Standard hold. Each VP of margin divides it
/// (`hold_hazard`).
pub const HOLD_HAZARD: i64 = 100;

/// Per-turn break chance (permille) of a hold `m100` hundredths of a VP above its target:
/// `HOLD_HAZARD` at the target, divided by 1 + margin above it, rising linearly to certain
/// once a full VP below it.
pub fn hold_hazard(m100: i64) -> i64 {
    if m100 <= -100 {
        1000
    } else if m100 < 0 {
        HOLD_HAZARD + (1000 - HOLD_HAZARD) * -m100 / 100
    } else {
        HOLD_HAZARD * 100 / (100 + m100)
    }
}

/// Chance (permille) that a hold `m100` above its target survives `turns_left` more turn ends.
pub fn hold_completion(turns_left: i64, m100: i64) -> i64 {
    let keep = 1000 - hold_hazard(m100);
    (0..turns_left.max(1)).fold(1000, |q, _| q * keep / 1000)
}

/// Raw public facts for one team, before shares are taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamFacts {
    pub team: TeamId,
    pub vp: i32,
    pub target: i32,
    pub cities: i32,
    pub population: i32,
    pub economy: i32,
    pub army: i32,
    pub science: i32,
    pub territory: i32,
    /// Turn ends left in its public VP hold (`TeamStanding::victory_hold`), if holding.
    pub hold_left: Option<i32>,
    /// Some member's original capital still stands in the team's hands
    /// (`PlayerPublic::capital_stands`): the capitals victory needs one.
    pub capital: bool,
}

/// Everything the estimate reads, so a plan can project a changed world and score it again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WinFacts {
    pub me: TeamId,
    /// Turn as permille of the expected game length (capped at 1500).
    pub phase: i32,
    pub teams: Vec<TeamFacts>,
    /// Treasury runway shortfall against the floor, permille (0 = solvent).
    pub runway_short: i32,
    /// Teams in the game, alive or not: picks the weights fitted for that count.
    pub seats: usize,
    /// Price with the weights fitted for `seats` (`WEIGHTS_DUEL` in 1v1) rather than the
    /// 8-player `WEIGHTS`. Off by default; search scoring always uses the fitted ones.
    pub fitted: bool,
    /// VP units per VP (1, or 100 once `refined`).
    pub vp_unit: i32,
    /// Turn ends a new hold needs (`data::VP_HOLD_TURNS`).
    pub hold_turns: i32,
    /// Multipliers on the public VP, city/population and Production+Gold gaps.
    pub relative: [i32; 3],
    /// Stable rivals for a projected decision: the VP leader and the closest rival at the
    /// start of the decision, not whoever a hypothetical capture makes leader.
    pub race_rivals: [Option<TeamId>; 2],
    pub race_gene: i32,
    pub hold_break_gene: i32,
    /// Gene `learned_value`: 1 scores teams with the learned value network (`value_net`), 2 with
    /// the linear weights refitted on the same games (`value_net::refit_weights`), 0 with
    /// `WEIGHTS`. Only `chance`/`estimate` read it; `estimate_with` and `search_chance` always
    /// use the given weights.
    pub learned: i32,
}

/// Milli-logits shared among the teams still holding an original capital (the capitals victory,
/// rules 37), so a standing capital is worth more as fewer remain. A constant, not a gene.
const CAPITAL_STAKE: i64 = 4000;

/// An estimate with its main terms: this team's score minus the average rival's, per feature,
/// in milli-logits. Terms explain; only `chance` is compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Estimate {
    pub chance: i32,
    pub terms: Vec<(&'static str, i32)>,
}

impl WinFacts {
    pub fn of(view: &PlayerView) -> Self {
        let alive = |team: TeamId| view.players.iter().any(|p| p.team == team && p.alive);
        let mut teams: Vec<TeamFacts> = view
            .standings
            .iter()
            .filter(|s| alive(s.team))
            .map(|s| TeamFacts {
                team: s.team,
                vp: s.vp as i32,
                target: s.target.max(1) as i32,
                cities: view
                    .players
                    .iter()
                    .filter(|p| p.team == s.team && p.alive)
                    .map(|p| p.cities as i32)
                    .sum(),
                population: 0,
                economy: 0,
                army: 0,
                science: 0,
                territory: 0,
                hold_left: s
                    .victory_hold
                    .map(|h| h.turns_needed.saturating_sub(h.turns_held).max(1) as i32),
                capital: true,
            })
            .collect();
        // Views without public standings (hand-built fixtures) score every alive team on
        // its members' public VP against a nominal target.
        if teams.is_empty() {
            for p in view.players.iter().filter(|p| p.alive) {
                match teams.iter_mut().find(|t| t.team == p.team) {
                    Some(t) => {
                        t.vp += p.vp as i32;
                        t.cities += p.cities as i32;
                    }
                    None => teams.push(TeamFacts {
                        team: p.team,
                        vp: p.vp as i32,
                        target: 10,
                        cities: p.cities as i32,
                        population: 0,
                        economy: 0,
                        army: 0,
                        science: 0,
                        territory: 0,
                        hold_left: None,
                        capital: true,
                    }),
                }
            }
        }
        teams.sort_by_key(|t| t.team);
        for team in &mut teams {
            team.capital = view
                .players
                .iter()
                .any(|p| p.alive && p.team == team.team && p.capital_stands);
        }
        for p in view.players.iter().filter(|p| p.alive) {
            if let Some(t) = teams.iter_mut().find(|t| t.team == p.team) {
                t.economy += p.economy.max(0);
                t.army += p.army.max(0);
                t.science += p.advancement.max(0);
            }
        }
        // Population and land are facts of cities and tiles. Charge unseen cities a modest
        // population rather than treating them as zero.
        for team in &mut teams {
            let known: Vec<_> = view
                .cities
                .iter()
                .filter(|c| view.team_of(c.owner) == team.team)
                .collect();
            let unseen = (team.cities - known.len() as i32).max(0);
            team.population = known.iter().map(|c| c.population.max(1)).sum::<i32>() + unseen * 3;
            team.territory = view
                .tiles
                .iter()
                .filter(|tile| {
                    tile.owner
                        .is_some_and(|owner| view.team_of(owner) == team.team)
                })
                .count() as i32;
        }
        let floor = 4;
        let e = &view.economy;
        let runway = crate::funding::runway_of(e.gold, e.gold_per_turn.total);
        let runway_short =
            runway.map_or(0, |r| (floor - r.min(floor as u32) as i32) * 1000 / floor);
        let expected = legatum_core::data::EXPECTED_GAME_TURNS.max(1) as i64;
        let own_progress = teams
            .iter()
            .find(|t| t.team == view.team)
            .map_or(0, |t| t.vp * 1000 / t.target.max(1));
        let leader = teams
            .iter()
            .filter(|t| t.team != view.team)
            .max_by_key(|t| (t.vp * 1000 / t.target, std::cmp::Reverse(t.team)))
            .map(|t| t.team);
        let closest = teams
            .iter()
            .filter(|t| t.team != view.team && Some(t.team) != leader)
            .min_by_key(|t| ((t.vp * 1000 / t.target.max(1) - own_progress).abs(), t.team))
            .map(|t| t.team);
        Self {
            me: view.team,
            phase: (view.turn as i64 * 1000 / expected).min(1500) as i32,
            seats: view.standings.len().max(teams.len()),
            fitted: false,
            teams,
            runway_short,
            vp_unit: 1,
            hold_turns: legatum_core::data::VP_HOLD_TURNS as i32,
            relative: [100; 3],
            race_rivals: [leader, closest],
            race_gene: 0,
            hold_break_gene: 0,
            learned: 0,
        }
    }

    /// Whether any team's population or territory is read at all. When neither is public both
    /// are 0 for every team, and a projection that moves them for one side alone invents a
    /// 100% share.
    pub fn extent_known(&self) -> bool {
        self.teams
            .iter()
            .any(|t| t.population > 0 || t.territory > 0)
    }

    pub fn with_relative(mut self, decisions: &DecisionParams) -> Self {
        self.relative = [
            decisions.relative_vp_percent,
            decisions.relative_city_percent,
            decisions.relative_economy_percent,
        ];
        self.race_gene = decisions.vp_race;
        self.hold_break_gene = decisions.vp_hold_break;
        self.learned = decisions.learned_value;
        self
    }

    pub fn team_mut(&mut self, team: TeamId) -> Option<&mut TeamFacts> {
        self.teams.iter_mut().find(|t| t.team == team)
    }

    pub fn team(&self, team: TeamId) -> Option<&TeamFacts> {
        self.teams.iter().find(|t| t.team == team)
    }

    /// The same facts in finer units (VP and targets ×100, everything else ×10), so a plan
    /// can project fractions: 0.8 of a city, a tenth of a VP. Shares and VP progress, and
    /// so the estimate, are unchanged.
    pub fn refined(&self) -> Self {
        let mut f = self.clone();
        f.vp_unit *= 100;
        for t in &mut f.teams {
            t.vp *= 100;
            t.target *= 100;
            for x in [
                &mut t.cities,
                &mut t.population,
                &mut t.economy,
                &mut t.army,
                &mut t.science,
                &mut t.territory,
            ] {
                *x *= 10;
            }
        }
        f
    }

    /// Public VP of `team`, as the projection stands.
    fn projected_vp(&self, team: TeamId) -> i32 {
        self.team(team).map_or(0, |t| t.vp)
    }

    /// Hundredths of a VP `team` stands above its target after the projection.
    pub fn hold_margin(&self, team: TeamId) -> Option<i64> {
        let t = self.team(team)?;
        let unit = self.vp_unit.max(1) as i64;
        Some((self.projected_vp(team) - t.target) as i64 * 100 / unit)
    }

    /// Chance (permille) that each team completes a VP hold: a holder survives its turns left,
    /// a team projected at its target starts a full window; others 0.
    pub fn hold_chances(&self) -> Vec<(TeamId, i64)> {
        self.teams
            .iter()
            .filter_map(|t| {
                let m100 = self.hold_margin(t.team)?;
                let left = match t.hold_left {
                    Some(left) => left,
                    None if m100 >= 0 => self.hold_turns.max(1),
                    None => return None,
                };
                Some((t.team, hold_completion(left as i64, m100)))
            })
            .filter(|(_, q)| *q > 0)
            .collect()
    }

    /// Feature vectors (permille) for every team, in team order; the private feature is set
    /// only for this seat's team.
    pub fn features(&self) -> Vec<(TeamId, [i64; 9])> {
        let n = self.teams.len().max(1) as i64;
        let total = |f: fn(&TeamFacts) -> i32| -> i64 {
            self.teams.iter().map(|t| f(t).max(0) as i64).sum()
        };
        let share = |value: i32, sum: i64| -> i64 {
            if sum <= 0 {
                1000 / n
            } else {
                value.max(0) as i64 * 1000 / sum
            }
        };
        let sums = [
            total(|t| t.cities),
            total(|t| t.population),
            total(|t| t.economy),
            total(|t| t.army),
            total(|t| t.science),
            total(|t| t.territory),
        ];
        self.teams
            .iter()
            .map(|t| {
                let progress = (self.projected_vp(t.team).max(0) as i64 * 1000
                    / t.target.max(1) as i64)
                    .min(1000);
                (
                    t.team,
                    [
                        progress,
                        progress * self.phase as i64 / 1000,
                        share(t.cities, sums[0]),
                        share(t.population, sums[1]),
                        share(t.economy, sums[2]),
                        share(t.army, sums[3]),
                        share(t.science, sums[4]),
                        share(t.territory, sums[5]),
                        if t.team == self.me {
                            self.runway_short as i64
                        } else {
                            0
                        },
                    ],
                )
            })
            .collect()
    }

    /// The chance to win, 0..=10,000, with per-feature terms (own minus mean rival).
    pub fn estimate(&self) -> Estimate {
        match self.learned {
            1 => self.estimate_inner(self.weights(), true),
            2 => self.estimate_inner(
                crate::value_net::refit_weights().unwrap_or(self.weights()),
                false,
            ),
            _ => self.estimate_inner(self.weights(), false),
        }
    }

    /// The weights `chance` uses.
    pub fn weights(&self) -> &'static [i32; 9] {
        if self.fitted {
            self.fitted_weights()
        } else {
            &WEIGHTS
        }
    }

    pub fn chance(&self) -> i32 {
        self.estimate().chance
    }

    /// The chance a search rollout is scored by: always the weights fitted for this game's
    /// team count.
    pub fn search_chance(&self) -> i32 {
        self.estimate_with(self.fitted_weights()).chance
    }

    fn fitted_weights(&self) -> &'static [i32; 9] {
        if self.seats == 2 {
            &WEIGHTS_DUEL
        } else {
            &WEIGHTS
        }
    }

    /// Teams (alive, in the facts) that still hold an original capital of their own.
    fn capital_holders(&self) -> usize {
        self.teams.iter().filter(|t| t.capital).count()
    }

    /// Milli-logits a team's standing capital is worth: the capitals victory goes to the last
    /// team holding one, so each capital that falls raises the price of those still standing
    /// (`CAPITAL_STAKE` shared among the holders: 500 with eight, 2,000 with two).
    fn capital_stake(&self) -> i64 {
        match self.capital_holders() {
            0 => 0,
            n => CAPITAL_STAKE / n as i64,
        }
    }

    /// This seat's VP race term (milli-logits) added to its own score (gene `vp_race`).
    pub fn race_term(&self) -> i64 {
        // A point is a race result. Price the leader's lost point as well as ours, with
        // extra urgency near the target and during a public hold. Keeping the opponent ids
        // stable makes projected alternatives comparable even when a capture changes rank.
        let race_progress = |team: TeamId| -> i64 {
            let Some(t) = self.team(team) else {
                return 0;
            };
            let vp = self.projected_vp(team).max(0) as i64 * 1000 / t.target.max(1) as i64;
            let urgency = 25
                + vp.min(1000) * 75 / 1000
                + if t.hold_left.is_some() {
                    self.hold_break_gene as i64
                } else {
                    0
                };
            vp * urgency / 100
        };
        let own_race = race_progress(self.me);
        let leader_race = self.race_rivals[0].map_or(0, &race_progress);
        let closest_race = self.race_rivals[1].map_or(0, &race_progress);
        let rival_race = if self.race_rivals[1].is_some() {
            (leader_race * 70 + closest_race * 30) / 100
        } else {
            leader_race
        };
        (own_race - rival_race) * self.race_gene as i64 * 3 / 100
    }

    pub fn estimate_with(&self, weights: &[i32; 9]) -> Estimate {
        self.estimate_inner(weights, false)
    }

    fn estimate_inner(&self, weights: &[i32; 9], learned: bool) -> Estimate {
        let features = self.features();
        let learned = learned
            .then(|| crate::value_net::scores(self, &features))
            .flatten();
        let score = |x: &[i64; 9]| -> i64 {
            x.iter()
                .zip(weights)
                .enumerate()
                .map(|(k, (x, w))| {
                    let percent = match k {
                        0 | 1 => self.relative[0],
                        2 | 3 => self.relative[1],
                        4 => self.relative[2],
                        _ => 100,
                    };
                    x * *w as i64 * percent as i64 / 100
                })
                .sum::<i64>()
                / 1000
        };
        let race_term = self.race_term();
        let holders = self.capital_holders();
        let stake = self.capital_stake();
        let scores: Vec<(TeamId, i64)> = features
            .iter()
            .enumerate()
            .map(|(i, (t, x))| {
                let base = learned.as_ref().map_or_else(|| score(x), |l| l[i]);
                let capital = if self.team(*t).is_some_and(|f| f.capital) {
                    stake
                } else {
                    0
                };
                (
                    *t,
                    base + capital + if *t == self.me { race_term } else { 0 },
                )
            })
            .collect();
        let Some(mine) = scores.iter().find(|(t, _)| *t == self.me).map(|(_, s)| *s) else {
            return Estimate {
                chance: 0,
                terms: Vec::new(),
            };
        };
        let top = scores.iter().map(|(_, s)| *s).max().unwrap_or(0);
        let total: i128 = scores.iter().map(|(_, s)| exp_neg(top - s) as i128).sum();
        let chance = (exp_neg(top - mine) as i128 * 10_000 / total.max(1)) as i64;
        // Victory holds: a completed hold wins outright; the logit shares the rest.
        let holds = self.hold_chances();
        let sum: i64 = holds.iter().map(|(_, q)| *q).sum();
        let scale = sum.max(1000);
        let own_hold = holds
            .iter()
            .find(|(t, _)| *t == self.me)
            .map_or(0, |(_, q)| *q * 1000 / scale);
        let rest = 1000 - sum.min(1000);
        let mut chance = (own_hold * 10 + rest * chance / 1000) as i32;
        // The capitals victory ends the game at once: the only team still holding a capital wins.
        if holders == 1 && self.teams.len() > 1 {
            chance = if self.team(self.me).is_some_and(|t| t.capital) {
                10_000
            } else {
                0
            };
        }
        let own = features
            .iter()
            .find(|(t, _)| *t == self.me)
            .map(|(_, x)| *x);
        let rivals: Vec<_> = features.iter().filter(|(t, _)| *t != self.me).collect();
        let mut terms = own.map_or_else(Vec::new, |own| {
            (0..9)
                .map(|k| {
                    let mean = if rivals.is_empty() {
                        0
                    } else {
                        rivals.iter().map(|(_, x)| x[k]).sum::<i64>() / rivals.len() as i64
                    };
                    (
                        FEATURES[k],
                        ((own[k] - mean)
                            * weights[k] as i64
                            * match k {
                                0 | 1 => self.relative[0],
                                2 | 3 => self.relative[1],
                                4 => self.relative[2],
                                _ => 100,
                            } as i64
                            / 100_000) as i32,
                    )
                })
                .filter(|(_, v)| *v != 0)
                .collect()
        });
        if race_term != 0 {
            terms.push(("relative VP race", race_term as i32));
        }
        let own_capital = self.team(self.me).is_some_and(|t| t.capital);
        if stake != 0 && !rivals.is_empty() {
            let rival_held = self
                .teams
                .iter()
                .filter(|t| t.team != self.me && t.capital)
                .count() as i64;
            let gap = stake * (i64::from(own_capital) * rivals.len() as i64 - rival_held)
                / rivals.len() as i64;
            if gap != 0 {
                terms.push(("capital standing", gap as i32));
            }
        }
        Estimate {
            chance: chance.clamp(0, 10_000),
            terms,
        }
    }
}

/// `2^30 · e^(−d/1000)` for `d ≥ 0` milli-logits: a 1/64-logit table, linearly interpolated.
fn exp_neg(d: i64) -> i64 {
    const STEPS: usize = 1024;
    const ONE: i64 = 1 << 30;
    // round(2^30 · e^(−1/64))
    const RATIO: i64 = 1_057_095_000;
    static TABLE: std::sync::OnceLock<Vec<i64>> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = Vec::with_capacity(STEPS + 2);
        let mut v = ONE;
        for _ in 0..=STEPS + 1 {
            t.push(v);
            v = ((v as i128 * RATIO as i128) >> 30) as i64;
        }
        t
    });
    let pos = d.max(0) * 64;
    let i = (pos / 1000) as usize;
    if i >= STEPS {
        return table[STEPS];
    }
    let frac = pos % 1000;
    table[i] - (table[i] - table[i + 1]) * frac / 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_facts_are_read_from_cities() {
        let view = crate::fixture::ViewBuilder::new(24, 16)
            .city(1, crate::fixture::ME, crate::fixture::off(3, 5), 4)
            .city(2, crate::fixture::ENEMY, crate::fixture::off(12, 5), 2)
            .city(3, crate::fixture::ENEMY, crate::fixture::off(18, 5), 7)
            .build();
        assert_eq!(
            WinFacts::of(&view)
                .team(view.team_of(crate::fixture::ENEMY))
                .unwrap()
                .population,
            9
        );
    }

    fn team(id: u8, vp: i32, cities: i32, army: i32) -> TeamFacts {
        TeamFacts {
            team: TeamId(id),
            vp,
            target: 10,
            cities,
            population: cities * 3,
            economy: cities * 5,
            army,
            science: cities * 4,
            territory: cities * 7,
            hold_left: None,
            capital: true,
        }
    }

    fn facts(teams: Vec<TeamFacts>) -> WinFacts {
        let third = (teams.len() > 2).then_some(TeamId(2));
        WinFacts {
            me: TeamId(0),
            phase: 500,
            seats: teams.len(),
            fitted: false,
            teams,
            runway_short: 0,
            vp_unit: 1,
            hold_turns: 10,
            relative: [100; 3],
            race_rivals: [Some(TeamId(1)), third],
            race_gene: 0,
            hold_break_gene: 0,
            learned: 0,
        }
    }

    #[test]
    fn equal_teams_split_the_chance_evenly() {
        let f = facts((0..4).map(|i| team(i, 2, 3, 40)).collect());
        assert!((f.chance() - 2500).abs() <= 2, "{}", f.chance());
    }

    #[test]
    fn race_values_a_vp_swing_and_nearby_leader_denial() {
        let mut base = facts(vec![
            team(0, 7, 3, 40),
            team(1, 9, 3, 40),
            team(2, 5, 3, 40),
        ]);
        base.race_gene = 100;
        let mut ours = base.clone();
        ours.team_mut(TeamId(0)).unwrap().vp += 1;
        let mut transfer = ours.clone();
        transfer.team_mut(TeamId(1)).unwrap().vp -= 1;
        assert!(transfer.chance() > ours.chance());
        assert!(ours.chance() > base.chance());
        let mut distant = ours.clone();
        distant.team_mut(TeamId(2)).unwrap().vp -= 1;
        assert!(transfer.chance() - ours.chance() > distant.chance() - ours.chance());
        base.race_gene = 0;
        assert!(
            !base
                .estimate()
                .terms
                .iter()
                .any(|(name, _)| *name == "relative VP race")
        );
    }

    #[test]
    fn a_city_an_army_and_vp_each_raise_the_chance_and_a_short_runway_lowers_it() {
        let base = facts((0..4).map(|i| team(i, 2, 3, 40)).collect());
        let v0 = base.chance();
        let mut more = base.clone();
        more.team_mut(TeamId(0)).unwrap().cities += 1;
        assert!(more.chance() > v0);
        let mut army = base.clone();
        army.team_mut(TeamId(0)).unwrap().army += 20;
        assert!(army.chance() > v0);
        let mut vp = base.clone();
        vp.team_mut(TeamId(0)).unwrap().vp += 2;
        assert!(vp.chance() > v0);
        let mut broke = base.clone();
        broke.runway_short = 1000;
        assert!(broke.chance() < v0);
        // Taking a rival's city is worth more than founding one: it also denies them.
        let mut taken = more.clone();
        taken.team_mut(TeamId(1)).unwrap().cities -= 1;
        assert!(taken.chance() > more.chance());
    }

    #[test]
    fn denying_a_leader_is_worth_more_than_denial_of_a_distant_rival() {
        let mut f = facts(vec![
            team(0, 5, 3, 40),
            team(1, 8, 5, 40),
            team(2, 2, 2, 40),
        ]);
        let base = f.chance();
        f.team_mut(TeamId(1)).unwrap().economy -= 5;
        let leader_denial = f.chance() - base;
        f.team_mut(TeamId(1)).unwrap().economy += 5;
        f.team_mut(TeamId(2)).unwrap().economy -= 5;
        assert!(leader_denial > f.chance() - base);
    }

    #[test]
    fn exp_table_is_monotone_and_bounded() {
        let mut last = i64::MAX;
        for d in (0..20_000).step_by(7) {
            let e = exp_neg(d);
            assert!(e <= last && e >= 0);
            last = e;
        }
        assert_eq!(exp_neg(0), 1 << 30);
        // e^-1 within 0.1%.
        let e1 = exp_neg(1000) as f64 / (1u64 << 30) as f64;
        assert!((e1 - (-1f64).exp()).abs() < 0.001, "{e1}");
    }

    #[test]
    fn terms_explain_the_lead() {
        let mut f = facts((0..3).map(|i| team(i, 2, 3, 40)).collect());
        f.team_mut(TeamId(0)).unwrap().vp = 6;
        let e = f.estimate();
        assert!(
            e.terms
                .iter()
                .any(|(name, v)| *name == "VP progress" && *v > 0)
        );
    }

    #[test]
    fn a_rival_hold_decays_with_the_turns_it_still_needs() {
        let base = facts((0..3).map(|i| team(i, 6, 3, 40)).collect());
        let free = base.chance();
        let with_rival = |left: i32, vp: i32| {
            let mut f = base.clone();
            let t = f.team_mut(TeamId(1)).unwrap();
            t.vp = vp;
            t.hold_left = Some(left);
            f.chance()
        };
        // A rival one turn from winning leaves us almost nothing; ten turns out, much more.
        let (one, five, ten) = (with_rival(1, 10), with_rival(5, 10), with_rival(10, 10));
        assert!(
            one < five && five < ten && ten < free,
            "{one} {five} {ten} {free}"
        );
        assert!(one * 5 < free, "{one} vs {free}");
        // A VP of margin makes the hold sturdier.
        assert!(with_rival(5, 11) < five);
        // Our own hold: worth more the closer it is to done, and never a certainty.
        let own = |left: i32| {
            let mut f = base.clone();
            let t = f.team_mut(TeamId(0)).unwrap();
            t.vp = 10;
            t.hold_left = Some(left);
            f.chance()
        };
        assert!(own(1) > own(5) && own(5) > own(10) && own(10) > free);
        assert!(own(1) < 10_000);
    }

    #[test]
    fn projecting_a_holder_below_its_target_breaks_the_hold() {
        let mut base = facts((0..3).map(|i| team(i, 6, 3, 40)).collect());
        let t = base.team_mut(TeamId(1)).unwrap();
        t.vp = 10;
        t.hold_left = Some(6);
        let holding = base.chance();
        // Two tenths of a VP below the target (refined units) already helps a lot.
        let mut denied = base.refined();
        denied.team_mut(TeamId(1)).unwrap().vp -= 20;
        assert!(
            denied.chance() > holding + 200,
            "{} {holding}",
            denied.chance()
        );
        // A whole VP below the target breaks it: back to the race.
        let mut beaten = base.clone();
        beaten.team_mut(TeamId(1)).unwrap().vp = 9;
        assert_eq!(beaten.hold_margin(TeamId(1)), Some(-100));
        assert!(beaten.chance() > denied.chance());
        assert_eq!(hold_completion(6, -100), 0);
    }
}
