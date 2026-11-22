# VP v2 skateboard: before and after

Three crowns, imperial overreach and era objectives replace the five titles and the size milestones. This directory records the historical comparisons used to settle those rules; its older experiment names are not current simulator controls. Current rules: `.claude/skills/legatum-rules/references/game-rules.md` section 6. Phone chart: `vp-v2-mobile.png`.

**Setup.** Hard + Search in every seat (`search k=3,h=20,d=4`), Standard speed, medium map, barbarians on. Cloud, c8g.16xlarge spot, about $3 in all. Same seeds before and after (4p 2026101001+, 8p 2026102001+), 256 games per cell; the sweep cells use the first 128 seeds of each lobby. Sampling error is about 3 points. Commands: `tools/sim/vp-v2-batch.sh`; metrics: `tools/sim/vp_v2_report.py`; raw numbers: `main-4p.json`, `main-8p.json`, `sweep.json`. The before cell reproduces the late-game study (T50 51%, T100 65% at 8p).

## Before (five titles) and after (Medium)

| | 4p before | 4p after | 8p before | 8p after |
|---|---:|---:|---:|---:|
| T50 leader goes on to win | 55.5% | 53.3% | 50.8% | 55.7% |
| T100 leader goes on to win | 71.1% | 65.9% | 69.7% | 60.8% |
| VP lead changes per game | 7.8 | 9.2 | 10.3 | 7.6 |
| Last lead change (median turn, % of game) | 97 (54%) | 164 (79%) | 155 (74%) | 183 (86%) |
| Titles held by the VP leader | 3.6 of 5 | 2.1 of 3 | 3.1 of 5 | 1.9 of 3 |
| Samples where one team holds every title | 19.9% | 19.0% | 8.8% | 10.6% |
| Overreach paid by the leader | 0 | 9.0 Gold (23% of its city Gold) | 0 | 8.0 Gold (23%) |
| Median game length / games with no winner | 194 / 0% | 217 / 0.4% | 214 / 3.1% | 219 / 0.4% |
| Units disbanded for unpaid upkeep per game | 1.3 | 1.2 | 7.9 | 2.6 |

## Era objectives (share of empires that entered the era and scored it)

| | Era 2 | Era 3 | Era 4 | Target |
|---|---:|---:|---:|---|
| 4p | 67% | 54% | 39% | 60 / 50 / 40, band 40-60 overall |
| 8p | 58% | 50% | 34% | |

At least one empire scores: 99-100% of games for eras 2 and 3, 70-76% for era 4 (few empires reach it), 89% pooled against a 90% goal. Every empire scores: 2-9% pooled against the 15% cap (4p era 2 is 25%). Thresholds are in `crates/core/src/objectives.rs::era_pool`, written for 8 empires and raised 12% per missing empire; calibrated on Hard (no search) sims from the peak values in `vp_race.rs`.

## Overreach sweep (4p + 8p pooled, 256 games)

| Per-era Gold, escalation | Leader's Gold to crowns | One team holds all 3 | Lead changes | Units lost to upkeep per game | T100 leader wins |
|---|---:|---:|---:|---:|---:|
| **Gentle** 0.5, x1/x2/x3 | 13% | 18% | 7.5 | 1.2 | 65% |
| **Medium** 1, x1/x2/x3 | 22% | 15% | 8.4 | 1.9 | 64% |
| **Strong** 2, x1/x2/x3 | 40% | 10% | 8.8 | 5.8 | 67% |
| 0.5 / 1 / 2, flat | 8 / 15 / 28% | 20 / 18 / 15% | 6.9 / 7.9 / 7.8 | 1.1 / 1.7 / 4.7 | 70 / 63 / 73% |

## Findings

- The skateboard does not close the T50 gap: the leader at T50 still wins 53-56%, against a start target of 45% (4p) and 35% (8p). Early snowball comes from the opening, which crowns cannot touch. T100 predictability falls about 7 points on average (70% to 63%), short of 55% and 45%. Lead changes rise at 4p and fall at 8p; the last change comes much later in both (54% to 79% and 74% to 86% of the game).
- Crown concentration barely moves at Medium (2.1 of 3 held by the leader, one team holds all three in 15-19% of samples at 4p). Strong halves it, but the AI then loses 5.8 units a game to unpaid upkeep: the AI appears to pay overreach by disbanding units rather than dropping a crown (not measured directly).
- Nothing is broken at Medium: 0.4% of games have no winner (3.1% before at 8p), crown holders have an empty treasury 1.5% of turns.
- **Win target.** With four eras the permanent ceiling is 3 era points + 2 goal + 2 Great Work = 7, so the specified 10 stalled 75% of 4p games at the 450-turn cap and 22% at 9; 8 gives the same game length as before. `solo_vp_target` is therefore ceiling + 1 capped at 10 (8 today, 10 with seven eras); a pair needs 16 today (not 20). `vp-target=N` overrides it.
- Recommended: **Medium** (landed). Strong is the setting to playtest next if concentration still reads as a problem, with the AI taught to drop a crown instead of disbanding.

---

# Overreach v2: the lead surcharge (2026-10-10, `claude/overreach-v2`)

The user's direction: overreach should be punitive, scale with how far ahead an empire is, hit everyone above the median (not only first place), use a trailing window so an empire being conquered is not penalised, and have no hard cap. Rules: `.claude/skills/legatum-rules/references/game-rules.md` section 6. Phone chart: `overreach-v2-mobile.png`. Raw numbers: `overreach-v2.json` (every cell, 4p, 8p and pooled). UI screenshots at 1000x600 and 1440x900: `overreach-ui/`.

**Rule.** Power is the trailing 30-turn average of an empire's total yields (Food + Production + Gold + Science; 15 turns at Quick). Lead = (Power - median Power of living empires) / median. The surcharge is a share of gross city Gold: `100 L / (L + k)` percent, zero at or below the median, rising forever, never reaching 100%. Crown upkeep (era x 1/2/3) is unchanged. Unpaid overreach uses the bankruptcy rule: the treasury resets and a unit deserts, with an "Overreach: your treasury can't pay your armies" notice and a warning up to 5 turns ahead. Knobs: `overreach-k=N` (default 45), `overreach-free=N` (free lead percent, default 0), `overreach-window=N`, `legacy-overreach` (crown upkeep only).

**Setup.** As above: Hard + Search in every seat, Standard speed, medium map, barbarians on, same seeds across cells (4p 2026101001+, 8p 2026102001+), 256 games per cell and lobby on cloud c8g.16xlarge spot (a 256-game cell is 0.1 server-hour at 4p, 0.3 at 8p; the whole study is under $5). Sampling error on a win share is about 3 points (4p and 8p pooled: 2). This historical sweep predates the current simulator controls and is retained as a results record. Built on master `ca3b7b1e` (before the historical faction rename merged); the AI's strain rule (rivals paying 20% or more of their Gold are softer campaign targets) is in every "on" cell.

## Before and after (Off = crown upkeep only; Strong = k 45, the landed default)

| | 4p off | 4p Strong | 8p off | 8p Strong | pooled off | pooled Strong |
|---|---:|---:|---:|---:|---:|---:|
| T50 leader goes on to win | 56.6% | 57.4% | 56.5% | 49.8% | 56.6% | 53.6% |
| T100 leader goes on to win | 68.8% | 62.1% | 60.8% | 53.0% | 64.8% | 57.6% |
| VP lead changes per game | 8.8 | 10.2 | 7.6 | 9.4 | 8.2 | 9.8 |
| Last lead change (median, % of game) | 77% | 88% | 87% | 94% | 83% | 91% |
| Top Power over median Power, T100 | 38.7% | 37.9% | 67.3% | 67.9% | 53.0% | 52.9% |
| Top Power over median Power, T150 | 40.9% | 39.3% | 72.8% | 69.1% | 56.9% | 54.2% |
| Units deserted for overreach per game (all empires) | 1.2 | 3.5 | 1.2 | 6.6 | 1.2 | 5.0 |
| ...by the VP leader's team | 0.6 | 2.4 | 0.4 | 2.7 | 0.5 | 2.5 |
| ...by the top-Power empire | 0.6 | 2.7 | 0.3 | 4.2 | 0.5 | 3.5 |
| Median game length / games with no winner | 217 / 0.0% | 235 / 0.0% | 221 / 0.4% | 243 / 2.0% | 218 / 0.2% | 239 / 1.0% |

## The sweep (pooled 4p + 8p, 512 games per setting)

| Setting | k, free | A 60% lead pays | Top empire's mean rate | Units deserted per game (top empire) | T50 | T100 | Lead changes | Top/median Power T150 | Game length |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Off | | 0% | 0% | 1.2 (0.5) | 56.6% | 64.8% | 8.2 | 56.9% | 218 |
| **Gentle** | 180 | 25% | 20% | 1.8 (0.9) | 56.6% | 62.6% | 9.0 | 55.5% | 227 |
| **Medium** | 90 | 40% | 33% | 2.7 (1.5) | 55.6% | 62.3% | 9.5 | 56.4% | 228 |
| **Strong** (default) | 45 | 57% | 46% | 5.0 (3.5) | 53.6% | 57.6% | 9.8 | 54.2% | 239 |
| k 30 | 30 | 66% | 55% | 8.2 (5.9) | 51.8% | 55.9% | 9.7 | 52.9% | 241 |
| k 20 | 20 | 75% | 62% | 10.8 (8.2) | 49.8% | 53.7% | 9.8 | 50.7% | 243 |
| Medium, 20% free | 90, 20 | 30% | 21% | 2.3 (1.1) | 55.6% | 66.7% | 9.2 | 54.8% | 226 |
| Strong, 20% free | 45, 20 | 47% | 31% | 3.3 (2.2) | 51.5% | 59.1% | 9.3 | 54.3% | 230 |

**The surcharge at typical and extreme leads (Strong).** The top-Power empire's lead over the median is 30% (4p) to 60% (8p) at T100 (median) and about 100% at p90. Strong charges 40% of Gold at a 30% lead, 57% at 60%, 68% at 100%, 81% at 200% and 90% at 450%. Medium: 25 / 40 / 52 / 68 / 83%. Gentle: 14 / 25 / 35 / 52 / 71%. The top empire actually paid a mean of 46% (Strong), 33% (Medium), 20% (Gentle).

## Findings

- **It moves the race, not the economy.** Strong makes the race longer and less decided: the T100 leader wins 7 points less (64.8% to 57.6%), the T50 leader 3 points less (the opening still decides most of T50, as expected), lead changes rise by 1.6 a game and the last one comes at 91% of the game rather than 83%. But Power itself does not compress: the gap between the top empire and the median at T100 is unchanged (53.0% pooled) and only 3 points lower at T150. The surcharge makes the leader pay in units (3.5 deserters a game from the top empire, against 0.5) and so in crowns and army, not in yields.
- **Gentle and Medium are inside the noise.** Medium moves T100 by 2.5 points (62.3% against 64.8%), adds 1.3 lead changes a game and 10 turns of length; Gentle does less of the same. The user's anchor (a typical strong lead pays about 40%) is Medium at a 60% lead; it is too light to matter on these metrics. Only Strong and harder move T100 by more than the sampling error.
- **The cost of Strong is real but not broken.** Median game length +21 turns (+10%), 1.0% of games without a winner (0.2% off; the worst cell, 8p Strong, 2.0%), 5 units deserting per game across 4 or 8 empires. The k 30 and k 20 settings roughly double and triple the desertions for another 2 to 4 points of T100; they were not recommended.
- **A free zone costs more than it saves.** Making the first 20% of lead free (the "slope near zero") cuts desertions by a third and lets the T100 leader win more (66.7% at Medium, a rise within noise); the kink is also harder to explain. Not recommended.
- **It reshapes the factions (information only).** Pooled win shares, off to Strong: Switzerland 23.7% to 9.3% and Sparta 9.4% to 3.9% (the Gold and army factions pay the most), Byzantium 4.7% to 12.7%, Celts 11.4% to 16.2%, Japan 13.2% to 17.5%. Athens 30% and Mongols 21% do not move. Best over worst moves from 6.4x to 8.2x, far from the 1.5x pillar either way. A Gold surcharge falls hardest on the factions whose identity is Gold; the Switzerland' signature hoard now needs a rethink under Strong.
- **Recommendation: Strong (k 45), landed as the default.** It is the first setting that clearly moves the race (T100 -7 points, later last lead change), the user leans punitive and accepts decay as the cost, and nothing is broken. Gentle and Medium stay one knob away (`overreach-k=180`, `overreach-k=90`). The honest limit: a Gold surcharge cannot flatten Power. If the user wants the pack closer together in yields, not only a contested VP race, the next levers are the ones he named: rising building costs and weaker military units for empires that overreach.



---

# Overreach v3: decadence (2026-10-10, `claude/overreach-v3`)

The user's direction: a Gold-only surcharge makes a few factions bear the brunt, so make it a blanket tax: "decreased production due to sloth and decadence, decreased science for the same reason... decreased population growth because people don't reproduce as much when they get wealthy... decrease them proportionately, and perhaps pull back on the gold penalty." Keep it strong. Rules: `.claude/skills/legatum-rules/references/game-rules.md` section 6. Phone chart: `overreach-v3-mobile.png`. Raw numbers: `overreach-v3.json` (every cell, 4p, 8p and pooled). UI screenshots at 1000x600 and 1440x900: `overreach-ui/v3-*`.

**Rule.** One rate, `R x L / (L + k)` percent (R 45, k 45 by default), where L is the empire's Power lead over the median living empire (Power is still the trailing 30-turn average of Food + Production + Gold + Science, now measured after decadence). It is zero at or below the median, never capped, and starts once an empire has a full 30-turn Power window (the opening's pace is tuned and a lead over fewer turns is not a verdict). It takes that percent from the city's Production, Gold and Science, and slows growth by the same share by raising the growth cost to `cost x 100 / (100 - rate)`. Food is not cut as a yield because a city's spare Food is a few points and a cut rounds to nothing; the growth cost has no rounding problem and cannot starve a city. The Gold surcharge is off (`surcharge-max=0`; v2 was 100). Crown upkeep is unchanged. Nothing else feeds Power (checked: `economy::player_income().cities`, the four yields only; population and territory do not). Knobs: `decadence-max=R`, `decadence-k=k`, `surcharge-max=S` (with `overreach-k`), `legacy-overreach`.

**Setup.** As v2: Hard + Search in every seat, Standard, medium map, barbarians on, same seeds in every cell (4p 2026101001+, 8p 2026102001+), 256 games per cell and lobby (512 per setting) on cloud c8g.16xlarge spot, about $5 in all (including one aborted round). The AI's strain rule counts a rival's decadence as burden (a point of decadence counts as 2 points of surcharge, `decadence_weight`). Off and v2 cells were rerun on this build (master has moved since the v2 study, so their numbers differ a little from the v2 section above). Commands: `tools/sim/overreach-v3-sweep.sh`; tables: `tools/sim/overreach_v3_summary.py`; chart: `tools/sim/overreach_v3_chart.py`. Sampling error is about 3 points on a win share.

## The sweep (4p + 8p pooled, 512 games per setting)

| Setting | T50 | T100 | Lead changes | Last change (% of game) | Top/median Power T100 | T150 | Top empire's decadence (mean) | Units lost to upkeep | Game length | No winner |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Off (crown upkeep only) | 55% | 68.2% | 8.2 | 83% | 56% | 62% | 0 | 1.8 | 218 | 0.6% |
| v2 Strong (Gold surcharge, k 45) | 54% | 60.5% | 8.9 | 87% | 54% | 55% | Gold 47% | 4.9 | 231 | 1.2% |
| **Gentle** R 20 | 54% | 66.0% | 9.3 | 86% | 44% | 46% | 9% | 1.6 | 235 | 1.2% |
| **Medium** R 30 | 55% | 60.3% | 10.0 | 91% | 38% | 38% | 12% | 1.8 | 240 | 0.6% |
| **Strong** R 45 (default) | 52% | 54.3% | 11.2 | 92% | 33% | 32% | 17% | 1.6 | 246 | 1.2% |
| R 60 | 51% | 54.6% | 12.0 | 93% | 28% | 25% | 20% | 1.7 | 257 | 0.6% |
| R 22, k 20 | 50% | 54.7% | 10.3 | 89% | 39% | 38% | 13% | 1.8 | 240 | 1.8% |
| R 45, k 90 | 56% | 60.3% | 10.3 | 92% | 38% | 39% | 12% | 1.7 | 240 | 1.2% |
| R 30 + Gold surcharge 15% | 51% | 59.9% | 10.4 | 91% | 38% | 39% | 12% | 1.6 | 240 | 0.6% |

The rate at typical leads (Strong): 8% at a 10% lead, 18% at 30%, 23% at 45%, 26% at 60%, 31% at 100%, 37% at 200%, 43% at 900%. The top empire's median lead is 27 to 32% (p90 60 to 70%) once decadence runs, so it pays 17% on average. Medium (R 30) is 5, 12, 15, 17, 21, 24 and 29% at the same leads; Gentle (R 20) is 4, 8, 10, 11, 14, 16 and 19%.

By lobby (off / v2 / Strong): T100 leader wins 4p 73.6 / 63.1 / 58.3, 8p 62.7 / 57.9 / 50.4; top over median at T100 4p 42 / 40 / 27, 8p 70 / 68 / 40.

## Findings

- **It does what v2 could not: it compresses Power.** Top Power over the median falls from 56% (off) and 54% (v2) to 33% at T100 and from 62% and 55% to 32% at T150, and the p90 lead from about 105% to 60%. The leader still clearly benefits from leading: it keeps a third more Power than the median empire and still wins 54% of games it leads at T100, against 25% (4p) and 12.5% (8p) by chance.
- **The race is livelier than v2.** T100 leader wins 54.3% (v2 60.5%, off 68.2%), lead changes 11.2 a game (8.9, 8.2), the last lead change at 92% of the game. All three goals (T100 at most about 58%, lead changes at least v2, Power compression) hold at Strong; Medium matches v2 on the race (60.3%, 10.0 changes) and still compresses Power to 38%.
- **It costs fewer units.** 1.6 units a game are lost to unpaid upkeep, the same as no overreach (1.8), against 4.9 for the Gold surcharge: decadence lowers Gold income a little everywhere instead of draining a rich treasury, so armies are rarely unpayable.
- **The price is game length.** Median game 246 turns against 218 (off) and 231 (v2): empires snowball more slowly. No-winner games stay at 1.2%. R 60 compresses Power further (25%) but adds another 11 turns and buys nothing on T100, so Strong is the knee.
- **Shape does not matter much.** R 22 / k 20, R 45 / k 90 and Medium (R 30 / k 45) all pay about 17% at a 60% lead and land within the noise of each other (T100 54.7, 60.3, 60.3; Power T150 38-39%). k 45 keeps the rate proportional to the lead through the typical range, which is the user's wording.
- **The Gold surcharge is not worth keeping.** Adding a 15%-ceiling surcharge to Medium changes nothing measurable (T100 59.9 against 60.3, lead changes 10.4 against 10.0, same Power spread). It is removed by default (`surcharge-max=0`) and stays a knob.
- **Burden by faction (information only).** Mean decadence over all living empire-samples at Strong: Mongols 8.6%, Sparta 7.8%, Japan 5.9%, Celts 5.3%, Switzerland 5.1%, Athens 4.9%, Germany 4.3%, Byzantium 2.6%. It falls on whoever is ahead, so it follows how strong a faction plays (Byzantium, which does not research and is rarely ahead, pays least; mobile Mongols and Sparta, which grow fastest, pay most), not on a Gold identity. Win shares, off / v2 / Strong: Switzerland 21.7 / 12.3 / 17.9 (v2's Gold surcharge cut it by 9 points, the blanket tax by 4), Sparta 10.4 / 4.7 / 5.2 (hit by both; its army is Production), Athens 27.9 / 33.9 / 34.7, Germany 18.3 / 20.2 / 20.7, Mongols 20.7 / 19.4 / 17.9, Byzantium 6.1 / 12.1 / 11.1, Japan 14.2 / 13.4 / 12.4, Celts 12.7 / 15.6 / 11.7. Best over worst is 4.6x off, 7.2x v2, 6.7x Strong, far from the 1.5x pillar either way; the Sparta and Athens gaps are the next balance work, not this change.
- **Crown upkeep is kept as v2 has it.** The sweep did not vary it (crown holders pay 4 Gold a turn on average at Strong; the leader pays about 21% of its city Gold, as before, and holders' treasuries are empty 1.4% of turns). Nothing suggested changing it.
- **Era objectives still land in band** (era 2 / 3 / 4 scored by 73 / 63 / 48% of the empires that entered, against 60 / 50 / 40; slower empires leave the objective thresholds a little easier, the same drift v2 showed).
- **Recommendation: Strong (R 45, k 45), landed as the default.** Medium (R 30) and Gentle (R 20) are one knob away (`decadence-max=30`, `decadence-max=20`); Gentle is inside the noise of off on T100 (66.0% against 68.2%).
