# VP pacing: longer games so late-era units matter (2026-10-18, `claude/vp-pacing`)

User, 2026-10-18, on air power arriving too late (Flight first known at a median of T205, the first bomber strike at T250 in a median 256-turn game): "the only way to do that in a way that would make sense is to just have fewer average points scored per era so that the game goes longer."

**Landed: every era objective asks 150% of the old count** (`experiments::DEFAULT_OBJECTIVE_SCALE`; knob `objective-scale=N`, 100 is the old rule). Nothing else changes: still 10 VP held for 10 turns, still three crowns, still 1 VP per era objective. The median game goes from about 244 to about 305 turns, and the median game now has 104 turns left after the first Flight (42 before); 83% of games have 60 or more (27% before).

## What was tried

All-AI games, Standard, medium maps, barbarians on, same seeds across arms (paired maps). Commands: `tools/sim/vp-pacing.sh` (arms, on the cloud box), `tools/sim/vp_pacing_report.py` (tables), `tools/sim/vp-pacing-hvh.sh` (human-seat benchmark). Raw numbers: `hard-8p.json`, `hard-4p.json`, `search-4p.json`. Main cohorts: 384 games per arm, Hard in every seat, 8 and 4 empires; confirmation: 128 games per arm, Hard + Search (`k=3,h=20,d=4`) in every seat, 4 empires. Sampling error on a median is about 3 turns; on a win share about 3 points.

| Arm | What it changes |
|---|---|
| base | master |
| t11, t12 | the VP target 10 becomes 11 or 12 (`vp-target=N`); 20 or 24 for a pair |
| os125, os150, os200 | every era-objective count at 125, 150, 200% of the old count (`objective-scale=N`) |
| t11os125 | both |

### 8 empires, Hard (384 games per arm)

| | base | t11 | t12 | os125 | **os150** | os200 | t11os125 |
|---|---:|---:|---:|---:|---:|---:|---:|
| No winner (%) | 0.0 | 4.2 | 43.8 | 0.0 | **1.3** | 2.3 | 6.8 |
| Game length p10 / median / p90 | 205 / 244 / 298 | 226 / 286 / 442 | 262 / 538 / 600 | 229 / 277 / 356 | **251 / 308 / 412** | 280 / 356 / 482 | 259 / 347 / 561 |
| Games over 300 / 400 turns (%) | 9 / 0 | 42 / 14 | 76 / 60 | 31 / 4 | **55 / 12** | 78 / 27 | 72 / 31 |
| Some empire has entered the Future era when the game ends (%) | 88 | 96 | 100 | 98 | **99.7** | 100 | 99.7 |
| Flight first learned (median turn) | 205 | 205 | 205 | 205 | **205** | 204 | 205 |
| Turns from the first Flight to the end: p10 / median / p90 | 15 / 42 / 96 | 27 / 83 / 238 | 58 / 324 / 406 | 28 / 74 / 148 | **48 / 104 / 206** | 76 / 150 / 274 | 52 / 143 / 353 |
| Games with 60+ turns after Flight (%) | 27 | 63 | 90 | 63 | **83** | 94 | 88 |
| Games with an aircraft strike (%) | 45 | 75 | 93 | 76 | **89** | 96 | 93 |
| First aircraft strike (median turn) | 242 | 243 | 244 | 243 | **244** | 246 | 245 |
| Aircraft strikes landed per game | 12 | 65 | 247 | 38 | **72** | 120 | 130 |
| Fighter interceptions (all 384 games) | 95 | 1,117 | 4,460 | 421 | **1,132** | 1,797 | 2,202 |
| T50 / T100 VP leader goes on to win (%) | 42 / 46 | 41 / 43 | 40 / 43 | 41 / 40 | **45 / 43** | 43 / 38 | 39 / 37 |
| VP lead changes per game | 9.5 | 11.2 | 13.5 | 10.1 | **11.5** | 12.4 | 12.3 |
| Last lead change (% of game, median) | 93 | 91 | 78 | 93 | **95** | 95 | 94 |
| Holds broken (% of resolved holds) | 61 | 70 | 74 | 55 | **61** | 55 | 68 |
| Era objectives scored (% of empires that entered the era) | 53 | 65 | 75 | 48 | **49** | 48 | 62 |
| Winner's VP at the end: era objectives / goal / Great Work / crowns | 5.4 / 1.6 / 2.0 / 1.4 | 5.8 / 1.9 / 2.0 / 1.5 | 6.0 / 2.2 / 2.0 / 2.0 | 5.5 / 1.4 / 2.0 / 1.5 | **5.5 / 1.5 / 2.0 / 1.4** | 5.6 / 1.5 / 2.0 / 1.2 | 5.8 / 2.0 / 2.0 / 1.5 |

### 4 empires, Hard (384 games) and Hard + Search (128 games)

| | Hard base | Hard t11 | Hard os125 | **Hard os150** | Hard os200 | Search base | Search t11 | Search os125 | **Search os150** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| No winner (%) | 0.0 | 4.7 | 0.5 | **0.8** | 2.6 | 0.0 | 3.9 | 0.0 | **0.8** |
| Length p10 / median / p90 | 197 / 239 / 300 | 213 / 270 / 461 | 222 / 269 / 353 | **242 / 296 / 391** | 260 / 323 / 462 | 201 / 237 / 298 | 219 / 272 / 440 | 222 / 271 / 347 | **241 / 304 / 389** |
| Turns after Flight: median / games with 60+ (%) | 49 / 35 | 84 / 67 | 77 / 70 | **105 / 88** | 132 / 94 | 45 / 33 | 83 / 67 | 81 / 64 | **117 / 85** |
| T100 VP leader goes on to win (%) | 56 | 55 | 55 | **54** | 49 | 52 | 56 | 52 | **47** |
| VP lead changes per game | 9.5 | 11.6 | 10.9 | **12.5** | 13.5 | 9.5 | 10.3 | 10.0 | **12.0** |

Air usage needs recorded tapes, which the Search arms do not replay; the Hard cohorts above carry it.

## Why this lever

- **Raising the target (t11, t12) is a cliff.** The permanent ceiling is 10 (six era objectives, goal 2, Great Work 2), so a winner already needs nearly every permanent point (the table's last row: 5.4 of 6 era objectives, 2.0 of 2 Great Work, 1.6 of 2 goal) and 11 or 12 forces crowns held at the same time as everything else, while the crowns' upkeep and decadence pull the leader back. 11 gives a median of 286 but a fat tail (p90 442, 14% over 400 turns, 4% of games with no winner); 12 gives a median of 538 with 44% of games unfinished at the 600-turn cap. It also makes the faction goal mandatory (winner's goal VP 1.6 to 1.9 and 2.2), narrowing the winning paths (pillar 10).
- **Scaling the objectives is a dial, not a cliff.** 125, 150 and 200% give medians of 277, 308 and 356 in the 8-empire lobby, with a stalled share of 0, 1.3 and 2.3%. The mechanism: the thresholds are met later, the late-era objectives most of all, so the last permanent points arrive later (a larger empire is needed for every one of them) while the typical empire's score barely moves; a winner still holds about 10 of 10 VP at the end.
- **It is the user's own wording.** Fewer average points scored per era: 53% of entering empires score an era now against 49% at 150% (8 empires; 60 to 55% at 4 empires), and the pooled share stays inside the 40 to 60% calibration band.
- **It adds no rule to explain.** Same one-VP-per-era objectives, same 10 VP target; the thresholds shown on the objective cards simply read 50% higher. No UI or Guide wording changes (the cards show the engine's counts).
- **Lengthening the hold alone cannot do it.** A win is the first turn the target is reached plus the hold, so raising the hold from 10 to 20 turns adds at most 10 turns to a game (the hold is already broken in 61% of the holds that start); the median needs +60. Not built, since the hold is a constant used across the AI, view and app.

## Competitiveness and the other guardrails (os150)

- Lead changes rise (9.5 to 11.5 at 8 empires, 9.5 to 12.5 at 4 with Hard, 9.5 to 12.0 with Hard + Search), the last lead change stays at the very end of the game (95%), and holds are broken as often as before (61% of resolved holds at 8 empires, 61% on master).
- The T100 leader wins slightly less often (46 to 43% at 8 empires, 56 to 54% at 4; 52 to 47% with Search), the T50 leader about the same. Pillar 11 is not hurt; the 4p Search T50 reading (50 to 56%) is inside the noise of 128 games.
- Several VP paths stay: the winner's mix is unchanged from master (5.5 / 1.5 / 2.0 / 1.4 against 5.4 / 1.6 / 2.0 / 1.4), where raising the target forces the goal and crowns.
- No stall problem: 1.3% of games have no winner at 8 empires (0.8% at 4), against 0% before and 4% for a target of 11. The longest games (p95) reach 458 turns.

## Air power (8 empires, Hard, 384 games per arm)

Flight is learned at the same turn (T205, T191 at 4 empires): the lever delays the end, not the research. Games with 60 or more turns left after the first Flight go from 27% to 83% (4 empires: 35% to 88%), the median window from 42 to 104 turns (4 empires: 49 to 105), games with an aircraft strike from 45% to 89%, strikes landed per game from 12 to 72 and Fighter interceptions from 95 to 1,132 (all games). The first strike of a game is still around T244: the first bomber is built within a few turns of Flight whichever way the game is run, and what changes is how many games are still going to see it and how long the air war runs after it. The goal of 60 or more turns of Modern-era play after Flight holds for 83% of games, not all of them.

## Wall-clock time (the trade-off)

Turns, not minutes, are what the sims measure. The 3 to 6 hour target was set against games of about 220 to 250 turns, which is about 0.8 to 1.5 minutes a turn; I could not measure real turn times (the archive's `finished_at - created_at` includes pauses and resumed saves, so it is no use). On that assumption:

| | median | p90 |
|---|---:|---:|
| Before (244 / 298 turns) | 3.3 to 6.1 hours | 4.0 to 7.5 hours |
| os150 (308 / 412 turns) | 4.1 to 7.7 hours | 5.5 to 10.3 hours |

So the median game lands near the top of the 3 to 6 hour band and one game in four (over 360 turns, 8 empires) goes past it. The extra turns are the late ones, when the board is largest, but also when most of a human's turn is "press return" or war; AI-only sims cannot say how fast a human plays them. If 6 hours is a hard ceiling, `objective-scale=125` (median 277, 4% over 400 turns, 74 turns after Flight) is the cheaper step: roughly +33 turns instead of +64.

## Human-seat benchmark and war suite (all seats)

`hard_vs_human --all-seats`, 8 playtest maps, same binary with and without the rule (`--rules-experiment objective-scale=150` against the control), `hvh-*.json`:

| | control | os150 |
|---|---:|---:|
| Hard, 128 salts per map: win rate in the human seat | 27.6% (25.6 to 29.7) | 31.2% (28.8 to 33.4) |
| Hard + Search, 32 salts per map | 21.5% (18.0 to 25.0) | 26.2% (22.3 to 30.5) |
| Milestone ratio (cities T30/T50, VP T60) | 0.367 | 0.367 |

No loss: the opening milestones are untouched (they come before any era objective matters) and the win rate rises a few points because a game that runs longer gives the human-seat AI more turns to reach the target on the maps it wins late. Median win turn per map goes from 232 to 336 on the Japan map and 252 to 299 on the Sparta one.

War suite (`tools/sim/war-scenarios.sh --brain hard --runs 64`, ledger rows "vp-pacing control" and "vp-pacing candidate"): all ten scenarios identical to the control (suite 40% both, delta 0.0). **The control itself is off the older baselines**: on current master `war-golden-horde` is 0% (43 to 50% before, `--check` floor 30%) and the "bugs" column reads 743 (0 before) with units standing in foreign cities; it is the same with and without this change, so it is a separate regression on master, for the lead to chase.

## Not measured

- Real human turn time at turns 250 to 400.
- 2- and 3-empire lobbies (the scaling is the same rule).
- Whether the AI should exploit the longer game on purpose (nothing was tuned for it).
