# Decadence on the mean, with a 50-turn Power window (2026-10-18, `claude/decadence-mean`)

The user's answer to the runaway question (2026-10-18): "I think it's fine as long as it's managed properly. Some kind of rolling average, and deviance from the median. So something like a 30 or 50 turn rolling average (so the penalties don't start accruing immediately after conquering) and relative to mean." Follow-up to `decadence-proportional.md`, whose runaway-conqueror question this answers.

**Rule.** The lead decadence is measured on is the empire's Power over the **mean** Power of the living empires (the leader is in the mean, so a runaway's lead stays bounded), not the median. Power is the trailing average of an empire's total yields over **50 turns** (was 30), so a conquest raises it, and with it the rate, gradually; decadence needs a full window, so nothing bites before turn 50. Rate = lead x 45 / 100 percent, no cap. Crown upkeep and the Gold surcharge still use the lead over the **median** (the user was asked whether the crown test should move too; it has not). Knobs: `decadence-median` (the old measure), `overreach-window=N`, `decadence-slope=c`.

## Setup

Hard + Search in every seat, Standard, medium map, barbarians on, same seeds in every cell (4p 2026101001+, 8p 2026102001+), 256 games per cell and lobby (512 per cell), spot c8g.16xlarge. Every cell is one binary; the **base** cell is `decadence-median,overreach-window=30,decadence-slope=40`, which is master's rule exactly (the human-seat benchmark reproduces master's number to the decimal with these flags). The m60, win50 and base cells ran on the first build; m45, m80 and m60w30 on a rebased one, which differs by unrelated master commits.

## The sweep (4p + 8p pooled, 512 games per cell)

| Metric | Base (median, 30 turns, c 40) | Median, 50 turns, c 40 | **Mean, 50, c 45 (landed)** | Mean, 50, c 60 | Mean, 30, c 60 | Mean, 50, c 80 |
|---|---:|---:|---:|---:|---:|---:|
| Median game length (turns) | 239 | 238 | 244 | 249 | 249 | 259.5 |
| T100 leader wins (%) | 51.3 | 51.0 | 50.4 | 45.5 | 47.1 | 43.9 |
| Lead changes per game | 9.03 | 8.60 | 8.67 | 9.41 | 9.82 | 10.37 |
| Top Power over median, T100 (%) | 35.8 | 37.4 | 37.6 | 32.8 | 30.3 | 27.4 |
| Top Power over mean, T100 (%) | 37.4 | 37.9 | 37.5 | 34.2 | n/a | 30.6 |
| Top empire's lead (its own measure), T100 median / p90 (%) | 34 / 61 | 34 / 61 | 34 / 64 | 29 / 57 | 28 / 51 | 24 / 49 |
| Top empire's decadence, mean over samples (%) | 13.7 | 13.0 | **14.9** | 18.3 | 17.2 | 22.5 |
| Runner-up's decadence, mean (%) | 6.4 | 6.0 | 7.4 | 9.5 | 8.3 | 12.2 |
| Highest rate seen in a game, median / p90 (%) | 21 / 35 | 22 / 36 | 22 / 33 | 27 / 41 | 28 / 44 | 34 / 51 |
| **Highest rate seen in any game (%)** | 62 | 89 | **66** | 60 | 84 | 85 |
| Games that ever reach 50% / 75% / 100% | 6 / 0 / 0 | 8 / 1 / 0 | **2 / 0 / 0** | 16 / 0 / 0 | 27 / 1 / 0 | 64 / 5 / 0 |
| Units deserted for unpaid overreach per game | 1.74 | 2.66 | 2.88 | 3.05 | 2.04 | 2.93 |

Sampling error is about 2 points on a win share.

## Findings

- **c = 45 keeps a typical leader at today's rate.** The top empire pays 14.9% on average (13.7% under the current rule), and the median game's peak is 22%. The race is where it was: T100 leader wins 50.4% against 51.3%, lead changes 8.7 against 9.0, the game 5 turns longer. c 60 is clearly heavier (T100 45.5%, game 249 turns, 16 games past 50%); c 80 is far too heavy (64 games past 50%).
- **The tail is shorter, not gone.** Games that ever reach a 50% rate fall from 6 (current rule) and 8 (median, 50 turns) to 2 at c 45; none reach 75%. The highest rate is 66%, one game (seed 2026102041, 8p, 363 turns), against 62% for the current rule on this master: the "87%" of the previous study was a different master (the AI has changed since), so the new maximum is not "well under" the old one, it is the same size on one game in 512. The mean does what it should: a conqueror's lead over the mean is bounded by the empire count (a conqueror at 1,000 against three at 100 leads the median by 900% and the mean by 207%, unit test).
- **The 50-turn window helps the tail and costs desertions.** At c 60, 30 turns against 50 turns: games past 50% 27 against 16, p90 peak 44 against 41, but units deserted for unpaid upkeep 2.04 against 3.05 per game. The same +0.9 shows on the median measure (1.74 to 2.66). A longer average keeps the rate up for a while after an empire's yields fall, which squeezes a war economy; the user chose the window to stop penalties starting the moment a conquest lands, and the race metrics do not move, so 50 is kept. Worth watching, not a blocker.
- **Window alone and measure alone barely move the race.** Median/50: T100 51.0%, Power spread 37.4%; the move to the mean at 50 turns changes the shape of the tail and little else.

## Human-seat benchmark (AI-impact gate, all seats)

Hard, 256 salts per map, 8 maps, master control against the branch, then a replicate on salts 257 to 512 (all on one binary: the control is `decadence-median,overreach-window=30,decadence-slope=40`):

| | Salts 1 to 256 | Salts 257 to 512 | Pooled |
|---|---:|---:|---:|
| Control (master's rule) | 26.0% (24.6 to 27.4) | 24.2% (22.8 to 25.6) | 25.1% |
| Mean, 50 turns, c 45 | 23.5% (22.4 to 24.9) | 24.4% (23.1 to 25.7) | 23.9% |
| Median, 50 turns, c 40 (window only) | 25.7% | n/a | |
| Mean, 30 turns, c 45 (measure only) | 25.4% | n/a | |

The first block alone read as -2.5 pp (Japan 8.4 47% to 38%, Switzerland 8.5 21% to 14%); the replicate did not repeat it (+0.2 pp), and each change alone is within 0.6 pp of the control, so the pooled -1.2 pp is inside the noise. Hard is not clearly worse.

## War scenarios, attack check

`tools/sim/war-scenarios.sh --check` (The Golden Horde comes): **master itself fails it** at 1% of 256 runs (floor 30%) on 446ccd4ce, and the branch is 0%. The 44% of the previous change (master `403a2c96f`) has gone since; the cause is one of the 55 master commits in between, not this change (on the branch decadence does not even run in the scenario, whose stored Power history has 30 entries against a 50-turn window; on master it does). Reported to the lead.
