//! Read-only observations of the VP race (VP v2): who leads at fixed turns, how often the lead
//! changes, and which eras each empire entered and scored. Never participates in orders, RNG, saves or hashes.

use legatum_core::objectives::Objective;
use legatum_core::{BuildingKind, GameState, scoring};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Turns at which the VP leaders are recorded.
pub const SNAPSHOT_TURNS: [u32; 7] = [25, 50, 75, 100, 125, 150, 200];

/// Power spread across the living empires at a snapshot turn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerSnap {
    pub first: i32,
    pub median: i32,
    pub last: i32,
    pub alive: u32,
    /// The top-Power empire's lead over the average, in percent.
    pub top_lead_percent: i32,
    /// The top-Power empire's decadence percent, zero when decadence is off.
    #[serde(default)]
    pub top_decadence_percent: i32,
    /// The mean Power of the living empires and the top-Power empire's lead over it, in percent.
    #[serde(default)]
    pub mean: i32,
}

/// Turns at which `EarlyBite` records the opening and decadence's 50-turn Power window.
pub const EARLY_TURNS: [u32; 6] = [10, 20, 30, 40, 50, 60];

/// What lead penalties cost the living empires at one early turn (read-only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarlyBite {
    pub alive: u32,
    /// Empires paying decadence, the sum and the largest of their rates.
    pub decadence_empires: u32,
    pub decadence_sum: i64,
    pub decadence_max: i32,
}

/// A turn sample of the whole race, every `SAMPLE_EVERY` turns.
const SAMPLE_EVERY: u32 = 5;

/// What one empire did with each era.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraOutcome {
    pub entered: u32,
    /// Turn it scored the era's objective, if it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scored: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VpRaceTelemetry {
    /// Gold income when each empire entered an era.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub entry_gold_income: BTreeMap<u8, BTreeMap<u8, i32>>,
    /// Leading team(s) by VP at each snapshot turn (ties list every leader).
    pub leaders: BTreeMap<u32, Vec<u8>>,
    /// Times the strict VP leader changed to a different team (a tie in between is ignored).
    pub lead_changes: u32,
    /// Turn of the last such change (0 when the lead never changed).
    pub last_lead_change: u32,
    /// Samples taken every few turns after turn 20, while at least two teams are alive.
    pub samples: u32,
    /// Samples with a strict leader.
    pub leader_samples: u32,
    /// Era outcomes per seat, final.
    #[serde(default)]
    pub eras: BTreeMap<u8, BTreeMap<u8, EraOutcome>>,
    /// Calibration: for every seat and entered era, the first turn each objective of the era's
    /// whole pool (not just the three drawn), at the game's counts, was met by what the empire
    /// gained since counting began for the era (`objectives::EraRecord`), by pool index (`None`
    /// when never). Checked every turn.
    #[serde(default)]
    pub pool_met: BTreeMap<u8, BTreeMap<u8, Vec<Option<u32>>>>,
    /// The pool indices drawn as the era's public objectives, by era (the same for every seat).
    #[serde(default)]
    pub pool_drawn: BTreeMap<u8, Vec<usize>>,
    /// The era's pool at the game's counts as rule text, by era, in `pool_met` index order.
    #[serde(default)]
    pub pool_text: BTreeMap<u8, Vec<String>>,
    /// Calibration: the highest value each objective measure reached, by seat and era, counted
    /// from when the era was entered (Fight measures) or at any sampled time (the rest).
    #[serde(default)]
    pub peaks: BTreeMap<u8, BTreeMap<u8, BTreeMap<String, u32>>>,
    /// Decadence at the early turns (`EARLY_TURNS`), summed over empires.
    #[serde(default)]
    pub early: BTreeMap<u32, EarlyBite>,
    /// Overreach v2: Power spread at the snapshot turns.
    #[serde(default)]
    pub power: BTreeMap<u32, PowerSnap>,
    #[serde(default)]
    pub top_samples: u32,
    /// Overreach v3, over the samples: empire-turns above the average that paid decadence and the
    /// sum of their rates, the top-Power empire's rates, and each faction's rates summed over all
    /// its living empire-turns (with the turn count), to show whether the burden is even.
    #[serde(default)]
    pub decadence_empire_turns: u32,
    #[serde(default)]
    pub decadence_percent_sum: i64,
    #[serde(default)]
    pub top_decadence_percent_sum: i64,
    #[serde(default)]
    pub decadence_by_faction: BTreeMap<String, (i64, u32)>,
    /// Decadence, proportional to the lead: the highest rate any empire paid on any turn of the
    /// game, and the sum over samples of the second-highest rate (the runner-up's, to compare with
    /// `top_decadence_percent_sum`).
    #[serde(default)]
    pub decadence_max_seen: i32,
    #[serde(default)]
    pub second_decadence_percent_sum: i64,
    /// Seats that were alive at the end.
    #[serde(default)]
    pub alive_at_end: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observer {
    data: VpRaceTelemetry,
    last_strict_leader: Option<u8>,
}

fn leaders(state: &GameState) -> Vec<u8> {
    let alive: Vec<_> = state
        .teams()
        .into_iter()
        .filter(|t| {
            state
                .team_members(*t)
                .iter()
                .any(|p| state.player(*p).alive)
        })
        .map(|t| (t.0, scoring::team_vp(state, t)))
        .collect();
    let best = alive.iter().map(|(_, v)| *v).max().unwrap_or(0);
    alive
        .into_iter()
        .filter(|(_, v)| *v == best)
        .map(|(t, _)| t)
        .collect()
}

/// Zero-target objectives whose `progress` reads each measure.
fn probes() -> Vec<(String, Objective)> {
    use BuildingKind as B;
    let mut v = vec![
        ("cities".to_string(), Objective::Cities { n: 0 }),
        ("population".into(), Objective::Population { n: 0 }),
        ("territory".into(), Objective::Territory { n: 0 }),
        ("humbled".into(), Objective::HumbleTheMighty),
        ("walled".into(), Objective::Breach),
        ("promotions".into(), Objective::Veterans { n: 0 }),
    ];
    for b in [
        B::Workshop,
        B::Market,
        B::Laboratory,
        B::Walls,
        B::Hydroponics,
        B::MachineShop,
        B::College,
        B::Clearinghouse,
        B::Autoworks,
        B::InferenceCore,
        B::Castle,
        B::StarFort,
    ] {
        v.push((
            format!("{b:?}"),
            Objective::Buildings {
                building: b,
                cities: 0,
            },
        ));
    }
    v
}

impl Observer {
    pub fn observe_resolved_turn(&mut self, resolved_turn: u32, state: &GameState) {
        let d = &mut self.data;
        let lead = leaders(state);
        if SNAPSHOT_TURNS.contains(&resolved_turn) {
            d.leaders.insert(resolved_turn, lead.clone());
        }
        if EARLY_TURNS.contains(&resolved_turn) {
            let mut e = EarlyBite::default();
            for p in state.players.iter().filter(|p| p.alive) {
                e.alive += 1;
                let dec = scoring::decadence_percent(state, p.id);
                if dec > 0 {
                    e.decadence_empires += 1;
                    e.decadence_sum += i64::from(dec);
                    e.decadence_max = e.decadence_max.max(dec);
                }
            }
            d.early.insert(resolved_turn, e);
        }
        for p in state.players.iter().filter(|p| p.alive) {
            d.decadence_max_seen = d
                .decadence_max_seen
                .max(scoring::decadence_percent(state, p.id));
        }
        let strict = (lead.len() == 1).then(|| lead[0]);
        if let Some(team) = strict {
            if let Some(prev) = self.last_strict_leader
                && prev != team
                && resolved_turn > 10
            {
                d.lead_changes += 1;
                d.last_lead_change = resolved_turn;
            }
            self.last_strict_leader = Some(team);
        }
        let top_power = state
            .players
            .iter()
            .filter(|p| p.alive)
            .max_by_key(|p| (scoring::power(state, p.id), std::cmp::Reverse(p.id.0)))
            .map(|p| p.id);
        if SNAPSHOT_TURNS.contains(&resolved_turn)
            && let Some(top) = top_power
        {
            let mut powers: Vec<i32> = state
                .players
                .iter()
                .filter(|p| p.alive)
                .map(|p| scoring::power(state, p.id))
                .collect();
            powers.sort_unstable();
            d.power.insert(
                resolved_turn,
                PowerSnap {
                    first: powers[powers.len() - 1],
                    median: scoring::median_power(state),
                    mean: scoring::mean_power(state),
                    last: powers[0],
                    alive: powers.len() as u32,
                    top_lead_percent: scoring::lead_percent(state, top),
                    top_decadence_percent: scoring::decadence_percent(state, top),
                },
            );
        }
        for p in state.players.iter().filter(|p| p.alive) {
            for era in 2..=legatum_core::objectives::era_of(state, p.id)
                .max(legatum_core::objectives::revealed_era(state))
            {
                if state
                    .player(p.id)
                    .eras
                    .get(&era)
                    .is_some_and(|r| r.entered == resolved_turn)
                {
                    d.entry_gold_income.entry(p.id.0).or_default().insert(
                        era,
                        legatum_core::economy::player_income(state, p.id).gold.total,
                    );
                }
                let pool = legatum_core::objectives::scaled_pool(&state.setup, era);
                d.pool_text
                    .entry(era)
                    .or_insert_with(|| pool.iter().map(|o| o.text()).collect());
                d.pool_drawn.entry(era).or_insert_with(|| {
                    legatum_core::objectives::drawn_from_pool(&state.setup, era)
                        .into_iter()
                        .map(|(i, _)| i)
                        .collect()
                });
                let row = d
                    .pool_met
                    .entry(p.id.0)
                    .or_default()
                    .entry(era)
                    .or_insert_with(|| vec![None; pool.len()]);
                for (i, o) in pool.iter().enumerate() {
                    if row[i].is_none() && o.met(state, p.id, era) {
                        row[i] = Some(resolved_turn);
                    }
                }
                if resolved_turn.is_multiple_of(SAMPLE_EVERY) {
                    let peaks = d.peaks.entry(p.id.0).or_default().entry(era).or_default();
                    for (name, probe) in probes() {
                        let v = probe.progress(state, p.id, era);
                        let e = peaks.entry(name).or_insert(0);
                        *e = (*e).max(v);
                    }
                }
            }
        }
        if resolved_turn > 20 && resolved_turn.is_multiple_of(SAMPLE_EVERY) {
            d.samples += 1;
            for p in state.players.iter().filter(|p| p.alive) {
                let decadence = scoring::decadence_percent(state, p.id);
                if decadence > 0 {
                    d.decadence_empire_turns += 1;
                    d.decadence_percent_sum += i64::from(decadence);
                }
                let by_faction = d
                    .decadence_by_faction
                    .entry(format!("{:?}", p.faction))
                    .or_default();
                by_faction.0 += i64::from(decadence);
                by_faction.1 += 1;
            }
            if let Some(top) = top_power {
                d.top_samples += 1;
                d.top_decadence_percent_sum += i64::from(scoring::decadence_percent(state, top));
                let mut rates: Vec<i32> = state
                    .players
                    .iter()
                    .filter(|p| p.alive)
                    .map(|p| scoring::decadence_percent(state, p.id))
                    .collect();
                rates.sort_unstable_by(|a, b| b.cmp(a));
                d.second_decadence_percent_sum += i64::from(rates.get(1).copied().unwrap_or(0));
            }
            if strict.is_some() {
                d.leader_samples += 1;
            }
        }
    }

    pub fn finish(mut self, state: &GameState) -> VpRaceTelemetry {
        for p in &state.players {
            let eras = p
                .eras
                .iter()
                .map(|(era, r)| {
                    (
                        *era,
                        EraOutcome {
                            entered: r.entered,
                            scored: r.scored.map(|s| s.turn),
                            slot: r.scored.map(|s| s.slot),
                        },
                    )
                })
                .collect();
            self.data.eras.insert(p.id.0, eras);
            if p.alive {
                self.data.alive_at_end.push(p.id.0);
            }
        }
        self.data
    }
}
