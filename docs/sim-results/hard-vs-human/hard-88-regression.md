# Hard on 8.8 rules: what went wrong and what fixed it (2026-10-11)

Branch `claude/hard-88-regression`. Human-seat benchmark, all eight maps, salts 1–64 (512 games per arm), one binary per round, paired by salt against the 8.8 control. The control is the 8.8 rebaseline exactly (142/512 Hard, 111/512 Hard + Search; the game code it measured is unchanged).

## Result

Default flips (all Hard only except the first):
- `sparta_search_only` 1 → 0: plain Hard Sparta uses the Sparta war genes Search already had.
- `restless_far_war` 0 → 12: Hard Sparta declares and keeps one war on a rival with no city within 12 hexes of ours when peace would make regiments desert, even under a peaceful plan.
- `troop_runway` 0 → 5 (10 turns at Standard): a city away from the front doesn't start a troop whose upkeep, plus the Strongest Army crown it would win, leaves under that runway.

| Map | Human | Hard before | Hard after | Hard + Search before | after |
|---|---|---:|---:|---:|---:|
| 8.3 Switzerland | won | 1 | 1 | 1 | 1 |
| 8.4 Sparta (Sparta) | won | 30 | **46** | 35 | **44** |
| 8.4 Japan (Japan) | won | 31 | 27 | 18 | 21 |
| 8.4 Celts (Celts) | won | 22 | 22 | 10 | 9 |
| 8.5 Mongols (Mongols) | won | 0 | 0 | 0 | 1 |
| 8.5 Switzerland | won | 28 | 29 | 19 | 15 |
| 8.6 Germany (Germany) | lost | 19 | 21 | 14 | 15 |
| 8.6 Sparta (Sparta) | won | 11 | **17** | 14 | 14 |
| **Score (equal map weight)** | | **27.7%** | **31.8%** | **21.7%** | **23.4%** |

- Hard seat-only: **+4.1 pp [+0.8, +7.4]** (95% paired salt bootstrap). Back to about its pre-era seat-only level (32.8% at `ca3b7b1e`).
- Opponents-only: **−4.9 pp [−9.4, −0.2]**: rivals with the change beat the seat more often, so the AI got better in every seat, not just the recorded one.
- Hard + Search seat-only: +1.8 pp [−1.6, +4.9] (noise).
- War scenarios (Hard seat, 64 runs): Sparta's First Blood 0% → **100%** (as Search). The other six are unchanged (the defences stay at 95–98%, the other attacks at 0–1%).
- AI-vs-AI sanity (the `troop_runway` 10 variant in every seat): the median game ends at T284 (control T288); no stalls. The gate's smoke and duel cover the chosen defaults.

The earlier "55% before the eras" was an all-seats run that included 8.5 Mongols at 60/64, before the Mongol drawbacks dropped that map to 0. The seat-only pre-era level was 32.8%.

## Mechanism

1. **A train-and-disband loop (Sparta, both maps).** Crown upkeep regularly exceeded the seat's whole Gold income. From T10 to T90 the 8.4 Sparta seat paid 175% of its gross Gold in crown upkeep and the 8.6 Sparta seat 134% (gross Gold only 1.0 and 2.3 a turn; other maps 14–46%). Hard's Sparta opening wants `cities + 4` troops before a Settler. Every Spearman it trained took or kept the Strongest Army crown, the treasury emptied, the insolvency logic disbanded a unit, and the opening trained another. On 8.4 the first Settler came at T81 (median of 12 traced salts), with 72 own disbands and 24 desertions in 12 games by T90. `troop_runway` counts a troop's upkeep and the crown it would win: the first Settler moves to T24. The user has since ruled that crown upkeep starts only after a crown is held 30 turns (`overreach-grace`, in progress), which removes most of the early loop at the rule level.
2. **Plain Hard Sparta never started its wars.** The Sparta war genes (expedition, forced feasible Campaign, capture-heal valuation) were confined to Hard + Search by `sparta_search_only=1`. In Sparta's First Blood (5 Spearmen at T29), Hard marched home in every run while Search took both German cities. Flipping it: First Blood 0% → 100%, and the 8.4 Sparta seat +12 wins on its own.
3. **Sparta's regiments deserted at peace.** From T105 on, 8.6 Sparta lost a unit every 5 peaceful turns: 26 a game. The existing restless-war logic never ran under Hard's goal-driven plan (a routed plan only declares the war it names, and its peace logic ends every other war). `restless_far_war` declares a war that can't reach us and refuses peace in it. On its own: 8.6 Sparta 11 → 21 wins, overall +1.6 pp.

What did **not** cause the regression, measured on the same maps and salts (rule experiments are diagnostics only; no rule changed):

| Arm (Hard, 512 games) | Score | vs control |
|---|---:|---:|
| No crown upkeep (`overreach-gold-x10=0`, all seats) | 27.0% | −0.8 [−5.7, +4.3] |
| No decadence (`legacy-overreach`, all seats) | 30.5% | +2.7 [−2.1, +7.6] |
| Neither | 27.0% | −0.8 [−5.1, +3.5] |

The rubber bands move single maps a lot (8.4 Japan 31 → 3 with neither; 8.4 Celts 22 → 49), but they don't explain the overall drop.

## Unit classes: what the AI learned from forcing compositions

The user's steer was that the AI doesn't know how to use the new mounted units. The `force_mix` experiment gene (default 0) swaps the class of every combat unit a city not under threat trains, to steer toward a fixed melee/ranged/mounted mix. Hard seat-only, 512 games each:

| Mix (melee/ranged/mounted) | Score | vs control | Units lost a game | Cities lost a game |
|---|---:|---:|---:|---:|
| Control (the AI's own, about 56/43/2) | 27.7% | | 26.2 | 1.8 |
| 1: line 8/1/1 | 19.3% | −8.4 [−13.7, −3.5] | 22.2 | 2.1 |
| 2: mounted 4/2/4 | 17.2% | −10.5 [−14.6, −6.2] | 33.0 | 2.9 |
| 3: ranged 3/5/2 | 26.6% | −1.2 [−6.4, +3.9] | 24.7 | 1.8 |
| 4: screen and gunline 5/5/0 | 28.5% | +0.8 [−4.3, +5.7] | 25.3 | 2.2 |
| 5: combined 5/2/3 | 19.9% | −7.8 [−12.1, −3.1] | 31.8 | 3.0 |
| 6: combat simulator's best response for the era to the armies seen | 22.3% | −5.5 [−10.7, −0.2] | 31.9 | 2.6 |

- **Every mix with a real mounted share lost.** Mounted units die more (units lost +22 to +27%), and cities fall more often (+45 to +60%): a horseman garrison is the worst defender against a Spearman (melee +18 against mounted).
- **The AI's own mix is already about the best tested.** Screen and gunline (5/5/0) is the only mix level with it.
- **The simulator's best responses don't transfer to full games** (mix 6, −5.5 pp). Its arena is open ground at equal Production with no cities, Walls or garrison duty; in full games those dominate. So the archer-heavy fixed priorities are not the regression either. Replacing them with the simulator's margins made Hard worse, so the default valuation is unchanged.
- **Why mounted fails here isn't mainly tactical.** A gene that skipped melee kills whose advance leaves the attacker exposed (the advance-on-kill finding from the horseman investigation) never fired: Hard orders a unit-on-unit melee strike in the field about twice a game (15 checks in 8 games to T150). The army rarely fights in the open, so a fast class has little to do. The war scenarios say the same: at war, Hard mostly idles (Mongol Storm: 12 turns at war without a battle).

## Crown upkeep after T50 (for the rule decision)

Every empire in the control games, T50 onward (1,920 empires, 559,168 empire-turns; current rules, before the 30-turns-held change):

| Faction | Crown upkeep > gross city Gold | > half of it | Treasury at 0 | Crown upkeep / gross Gold |
|---|---:|---:|---:|---:|
| Sparta (Sparta) | 10.3% | 27.2% | 1.5% | 21.5% |
| Mongols (Mongols) | 7.0% | 28.9% | 14.0% | 38.8% |
| Byzantium (Byzantium) | 5.7% | 20.8% | 2.9% | 20.9% |
| Japan (Japan) | 5.0% | 15.7% | 3.5% | 17.6% |
| Germany (Germany) | 2.3% | 18.0% | 0.1% | 21.1% |
| Athens (Athens) | 2.0% | 19.7% | 1.6% | 25.6% |
| Celts (Celts) | 0.9% | 22.0% | 1.1% | 29.5% |
| Switzerland (Switzerland) | 0.7% | 3.8% | 0.1% | 9.4% |
| **All** | **4.5%** | **18.2%** | **2.7%** | **18.9%** |

Winners pay 27% of their gross Gold in crown upkeep after T50 (exceeding it in 6.4% of turns). After T50, crown upkeep exceeds income only occasionally overall, but for Sparta it's one turn in ten, and the Mongols' treasury is empty one turn in seven. A share-of-income cap (for example crown upkeep at most 50% of gross city Gold) would mostly touch Sparta and the Mongols. Re-measure once the 30-turns-held rule lands before proposing it.

## Not adopted

- `hard_fronts` (plain Hard prices a campaign on every reachable enemy city, as Search does): −0.4 pp, no war-scenario change; dropped.
- `advance_risk` (skip melee kills whose advance is lethal): never fired; dropped.
- `troop_runway` 10 / 20: −0.6 / +0.6 pp alone; 5 chosen (+2.9 alone, and it keeps 8.4 Japan at 27 where 10 dropped it to 23).
- `force_mix` stays as an experiment gene, default 0.

## Artifacts

`runs/hard-88/`: the arms files for each round (`arms-r1..r6.json`), per-game outcomes for every reported arm (`rN-<arm>-games.json.gz`; `r2-ctl` keeps the full VP standings at T60/100/150/200/250 and the per-empire crown burden), and the two report scripts (`tools/arms_report.py`, `tools/merge_slices.py`). Cloud batches `hard88-r1` to `hard88-r6`, about 4.1 server-hours (about $3). War-suite reports: `war-scenarios/runs/20261011T15*-hard-war-*`.
