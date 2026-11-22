# Hard learns the round-2 battle moves: breach and siege stance (2026-11-08)

Branch `claude/hard-breach` (built on `claude/training-round2`, `RULES_VERSION` 23). Increment 3 of the AI plan: distil the Training search's round-2 wins into plain Hard as genes. Both genes default to 0.

## What was built (`crates/ai/src/breach.rs`)

- **`breach`** (percent weight on the city's value; candidate 100). A pass right after door defence. For each visible enemy city at war, the shots already ordered at it count first, then the free shooters and aircraft in range are added (ground first, largest expected damage first) until the city falls 95% of the time. The nearest free melee unit that reaches the city this turn walks in, and its route is reserved. The zero chance is exact over the uniform rolls (`zero_chance`). A garrison, or an owner's melee unit that can step in first (owners reinforce their attacked cities before movement), multiplies it by the walker's win chance on the fallen city. The pass orders the breach when capture × value (400 + 80 per population) × hold beats the walker's expected loss. The air planner leaves a plane the ground planner ordered alone (`scripted.rs::extend_air`).
- **`siege_stance`** (turns; candidate 4). A pass right after the city-assault coordinator. **Storm** when the strike forecast on the free units (`forecast_siege_rounds`, this many rounds) takes the city for fewer Production lost than its value: the units that hit it this turn do, and the rest stay with the other passes. Otherwise **siege** when the shooters near it and our aircraft zero it within this many turns at expected damage per turn: shooters take firing tiles (`siege_city`), and melee hold out of the city's strike but within one move of it. The same turns-to-zero test replaces `seize_open_city`'s wait while a walled city still fires above half HP (`military.rs`).
- Tools: `combat_lab --params-compare PARAMS OUT` (Hard against Hard with genes, same dice) and `crates/sim/examples/breach_agreement.rs` (agreement with the search).

## Gates (candidate: `breach` 100, `siege_stance` 4; files in this folder)

| Gate | Mark | Result |
|---|---|---|
| Combat lab, 21 cells × 8 dice | ≥ 38% (from 29.8%) | **34.5%** (58/168 against 50/168), 12 fights only the candidate took and 4 only Hard took; Production lost 283 → 299. **Miss** (`lab/bs4-compare.json`) |
| War suite, 34 × 32 runs, seat only | mean up, no scenario significantly worse | **+5.0** (45% → 50%). Significantly better: Hollow Swiss T76 0 → 100%, Tepid Swiss T220 and T220 fresh 62 → 100%. None significantly worse; worst Celts wild hold −6.2 [−18.8, +6.2]. **Pass** (`war/compare-bs4.md`) |
| Designer seat, 48 salts × 5 start turns × 7 tapes, both arms on the box | improve | Training 38.5% → 37.0% (−1.6 [−5.9, +2.8]), held out 56.8% → 58.8% (+1.9 [−3.2, +7.0]), all −0.1 [−3.4, +3.3]. **Flat: not improved** (`designer-seat/`) |
| Human-seat gate, 8 maps × 256 salts, reseeded dice | seat up, rivals not softer | Seat-only **+2.05 pp [+0.54, +3.61]** (22.75% → 24.80%), rivals-only **−1.51 pp [−3.17, +0.10]** (the rivals got harder). **Pass** (`human-seat-compare.json`) |
| Tactics cards (`claude/tactics-scenarios`, rehashed to rules 23), 50 runs | wins | 0/50 → 0/50 on all four. Swiss Biel median score 1,080 → 1,130 (`tactics/`) |

**Decision: both defaults stay 0.** The rule is to ship only if the lab, the war suite and the designer seat all improve. The designer seat is flat and the lab misses its mark.

### Which gene does what

| Arm (war suite delta) | Suite | Notes |
|---|---:|---|
| `siege_stance` 4 alone | +5.0 | All of the gain |
| `breach` 100 alone | +0.0 | It fired once in 8 × 34 runs |
| Both | +5.0 | |

The first breach (before the fixes on this branch) took Tepid Swiss T220 from 62% to 50%. Its Bomber hit a city the Artillery was already zeroing, while the Bomber's usual target, a Spearman beside the city, walked in first to reinforce. A later single-walker rule made it worse (15%): own units claiming one tile take turns and do not bounce (`rules/movement.rs`), so that rule was removed.

### Agreement with the search (`agreement-bs4.json`)

There are 100 lab positions where the round-2 full-game check played the search's plan over Hard's (`confirm.reason == clear`, largest gains first; log regenerated with `combat_lab --training-compare OUT --training-round2 --repeats 8`, which reproduced 42.9% exactly). Each lesson unit's action (shoot or strike a tile, attack a tile, move, hold) is compared with Hard replanning the logged arena:

| | Whole position agrees | Lesson units agree (of 181) |
|---|---:|---:|
| Plain Hard | 7 | 10 |
| Candidate | 8 | 16 |
| By played stance (candidate) | Focus 4/20, Hard 2/48, Storm 2/7, Siege 0/19, Hold 0/6 | |

The genes barely move agreement. The search's siege holds its melee in place inside 2 tiles; `siege_hold` steps them out of the strike, so the actions differ.

### Why the tactics cards stay at 0/50

- **Germany two cities:** at T142 neither Aztec city is visible, so no bomber strikes (plain Hard's air planner doesn't either). Xochimilco is 9 tiles from Cologne, out of Bomber range. The designer rebased forward and moved to see the cities first. The breach can't fire until staging and vision put a plane in range.
- **Sparta Nicomedia:** six melee units and no shooters, so the siege branch has nothing to shoot with. Hard's plan goes for Syracuse, not Nicomedia.
- **Swiss Biel and Sparta before the walls:** defence and declaration, which these genes don't touch.

## Follow-up: `siege_stance` 4 alone, 96 salts (2026-11-08)

Run locally, both arms on one commit (control `16dd54e8c`; the arm differs only in the default). The bar set for turning it on: the held-out set no worse, with its interval excluding a drop of more than 2 points, and every tape neutral or positive.

| Set or tape | Control | `siege_stance` 4 | Delta [95%] |
|---|---:|---:|---|
| Training | 37.3% | 36.9% | −0.4 [−3.5, +2.6] |
| Held out | 57.0% | 58.5% | +1.5 [**−2.1**, +5.1] |
| All | 45.8% | 46.2% | +0.4 [−2.0, +2.8] |
| India 8.9.25 | 13.3% | 10.8% | −2.5 [−6.6, +1.6] |
| Swiss 8.9.24 | 40.0% | 42.3% | +2.3 [−3.9, +8.5] |
| Other tapes | | | between −0.6 and +1.5, none significant |

The held-out interval reaches −2.1, so it doesn't exclude a drop of more than 2 points. **The default stays 0.** It misses the bar narrowly: no tape is significantly worse, and India is the only negative tape beyond 1 point. Files: `designer-seat/control-96.json`, `designer-seat/siege4-96.json`.

## Next

1. **The lab gap is the strike forecast.** On the hills cell (Training 7/8, the candidate 0/8) and the walled 100-HP Sparta cell, `forecast_siege` returns no capture where the search's rollouts take the city. Calibrate the forecast against lab rollouts before adding more stance logic.
2. **Air staging for the breach:** rebase Bombers to a city in range of a target a walker can reach, and send a unit to see the city (the Germany card).
3. `siege_stance` alone is the measured win (war suite +5.0, human seat seat +2.05 and rivals −1.51). It is held back only by the flat designer seat. A larger designer-seat run (96 salts) would settle whether it is truly flat.
