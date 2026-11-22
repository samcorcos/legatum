# AI objective on short cards (2026-11-10, `claude/ai-objective`)

Three Lab cards the designer finds easy and Hard fails. Each arm is 50 runs of
`legatum-sim scenario scenarios/lab/<card>.json --runs 50 --threads 6 --decision-turns 8`,
same seeds in every arm (run 0 keeps the card's seed), rules version 26.

## Baseline (master ad6b344b7, rules 26)

| Card | Hard wins / 50 | How runs ended |
|---|---|---|
| `swiss-hold-biel` | 34 | 16 x Biel lost (T135 to T138) |
| `mongols-ride-to-dyrrachium` | 2 | 48 x time ran out (the army marches on walled Nicomedia from T40) |
| `germany-two-cities-one-turn` | 0 | 32 x Chalco taken, Xochimilco not; 18 x neither |

The per-card reports are `baseline-<card>.md`. The seat's plan switches by turn are in each run's
`told` list of the JSON (written for every run since this branch).

What the plan layer chose (`plan.rs::choose_plan`, 50 runs each):

- **Hold Biel:** a Campaign on T132 in all 50 runs (against team 1, city 16), then other
  Campaigns or Peace, while ten Indian units stand around Biel. The siege-stance pass then sends
  the units bought for Biel (a Cavalry, a Phalangite, a Soldier) at an enemy city.
- **Ride to Dyrrachium:** a Campaign on Ravenna (city 3) on T37; Dyrrachium (city 8) is not
  a candidate on T37 to T39 (`in-reach campaign force gate`, then not on the nomination list).
  Ravenna is walled by T40 and the plan switches to walled Nicomedia (city 11) in all 50 runs,
  valued 1260 against Dyrrachium's 323: capture 45% against 52%, plus a `supply` term of
  +1080 for Nicomedia's deposit.
- **Two Cities:** Contain against the Aztecs on T142; a Campaign on Chalco only from T145
  (39 runs).

## Told the objective (`--tell-objective`, measurement only)

The seat's Hard brain is told the card's win condition as a pinned plan
(`crates/sim/src/scenario_told.rs`): a Campaign on the first card city it doesn't own, or, for a
hold card, the quiet plan (no Campaign, no Contain war). Every executor is Hard's own.

| Card | Baseline | Told |
|---|---|---|
| `swiss-hold-biel` | 34 | **49** |
| `mongols-ride-to-dyrrachium` | 2 | **50** |
| `germany-two-cities-one-turn` | 0 | 0 (Chalco 43, Xochimilco 0) |

Reading: on Hold Biel and Dyrrachium the whole gap is the objective (Hard executes once it is
pointed at the right thing). On Two Cities it is execution: told, Hard takes Chalco in 43 of
50 runs but never Xochimilco.

Rejected variant: telling a hold card by pinning `PlanKind::Hold` on the most threatened city
(the Hard+Search hold plan) took Hold Biel from 34/50 to 6/50. `plan.rs::execute_hold` runs
before `plan_military` and holds the Field Cannon already in Biel, so `door.rs`'s garrison swap
(Rifleman in, cannon out) never happens and Biel falls on T135.

## Gene `capture_horizon` (Dyrrachium target choice, off by default)

Where the miss comes from (`plan.rs::price_campaign`, traced on run 0): on T40 Nicomedia scores
1260 against Dyrrachium's 323. Capture odds are 45% against 52% because `capture_odds` reads
only the forecast's turns (4 against 3) and `forecast_siege` already halves every attack for the
Walls; the rest is `supply` +1080 for Nicomedia's deposit (2 Production a turn, x `supply_term`
400 x `supply_shortage_percent` 300). Hard actually takes Nicomedia within those 4 turns in 7 of
50 runs.

The gene prices a capture by when it lands (`plan.rs::capture_turns`): a siege of a walled city
whose force has no Siege and none on offer counts its siege turns at the gene's percent, and the
supply term counts only the share of the 10-turn plan horizon left after the capture. At 200, T40
reads Nicomedia at 16% with its supply term cut to a fifth, and the plan takes Dyrrachium.

All arms seat-only (`--ai-params '{"decisions":{"capture_horizon":200}}' --params-seat-only`):

| Card | Baseline | Told | Gene on | Told + gene on |
|---|---|---|---|---|
| `swiss-hold-biel` | 34 | 49 | 34 | 49 |
| `mongols-ride-to-dyrrachium` | 2 | 50 | **48** | 50 |
| `germany-two-cities-one-turn` | 0 | 0 | 0 | 0 |

Each half alone leaves Dyrrachium at 2/50 (the slower walled siege alone: Nicomedia 383 against
323; the horizon share alone: Nicomedia still well ahead at 45%).

Every Lab card, gene off and on (seat only, 50 runs): only Dyrrachium's wins move.

| Card | Off | On | Median off | Median on |
|---|---|---|---|---|
| `athens-break-the-hold` | 0 | 0 | 590 | 590 |
| `aztecs-eight-by-seventy` | 4 | 4 | 750 | 750 |
| `aztecs-swiss-spearmen` | 0 | 0 | 200 | 200 |
| `byzantium-break-the-fire` | 0 | 0 | 1040 | 1040 |
| `celts-break-the-reserve` | 0 | 0 | 740 | 740 |
| `celts-parked-at-st-gallen` | 1 | 1 | 730 | 730 |
| `celts-steal-a-settler` | 0 | 0 | 150 | 150 |
| `celts-stop-the-horde` | 0 | 0 | 710 | 710 |
| `celts-strike-the-leader` | 0 | 0 | 640 | 640 |
| `germany-defend-essen` | 0 | 0 | 760 | 760 |
| `germany-take-gergovia` | 0 | 0 | 670 | 670 |
| `germany-two-cities-one-turn` | 0 | 0 | 880 | 880 |
| `hold-athens` | 49 | 49 | 180 | 180 |
| `india-no-niter` | 0 | 0 | 950 | 900 |
| `india-strike-the-leader` | 0 | 0 | 2060 | 2060 |
| `india-take-the-settler` | 0 | 0 | 130 | 130 |
| `mongols-eight-points` | 0 | 0 | 1100 | 1110 |
| `mongols-ride-to-dyrrachium` | 2 | 48 | 360 | 730 |
| `mongols-wide-india` | 1 | 1 | 920 | 920 |
| `sparta-before-the-walls` | 0 | 0 | 370 | 370 |
| `sparta-next-door` | 0 | 0 | 840 | 840 |
| `sparta-walled-nicomedia` | 3 | 3 | 970 | 970 |
| `swiss-hold-biel` | 34 | 34 | 1200 | 1200 |
| `swiss-hold-the-south` | 0 | 0 | 480 | 480 |
| `swiss-strongbox` | 13 | 13 | 350 | 350 |
| `swiss-walls-athens` | 0 | 0 | 640 | 640 |

Dyrrachium is a training card in the scenario gate's split; the gene was fitted on it, so its
48/50 is not a held-out result. Not measured: the designer seat, the war suite, censuses. The gene
stays 0 until those run.

## Two Cities (execution, not built)

Told, Hard takes Chalco in 43 of 50 runs and never Xochimilco. Run 0: the Bombers strike a Ranger
and rebase on T143 instead of hitting Chalco; the Cavalry upgrades to a Tank on T144; the Tank
walks back to Colony 7 on T146 (`door_military`); Xochimilco is never a Campaign candidate
(`campaign nomination shortlist`, then `in-reach campaign force gate`). Aircraft orders are added
after the ground plan (`Brain::add_air_orders`), so no ground pass can count on a same-turn Bomber
strike to open a city. The designer's line: Infantry and Cavalry next to both cities on T142, one
Bomber on each city on T143, and walk in.

## Short-horizon objective step (gene `short_objective`, off by default)

Built from the told-arm split (`crates/ai/src/short_objective.rs`; rules reference, AI section).
Each turn, from the seat's own view, it nominates every enemy city its forces in reach take within
5 turns and every home city a visible army is forecast to take within 5 turns. Each capture is priced
with `price_campaign` under `capture_horizon` pricing without the supply term, less the expected
loss of the endangered home cities. It pins the best as an operation: a Campaign, or a Hold on the
most endangered city. It drops a capture whose odds stay under 15% for two turns. It never reads the
card's win condition.

Two companion changes:

- **Gene `strike_upgrade_wait` (off):** a unit in the running strike doesn't upgrade.
- **Bug fix in `plan.rs::execute_hold`:** the Hold now makes `door.rs`'s garrison swap first,
  instead of holding the unit already inside. Told as a pinned Hold on Biel, Hard went from 6/50
  before the fix to **50/50** after it (`--tell-hold-city`).

Untold arm with the new genes on, against the told ceiling (50 runs, same seeds, seat only):

| Card | Baseline | `short_objective` | `strike_upgrade_wait` | Both | Told ceiling |
|---|---|---|---|---|---|
| `swiss-hold-biel` | 34 | **45** | 31 | 37 | 49 (50 with `--tell-hold-city`) |
| `mongols-ride-to-dyrrachium` | 2 | **50** | **50** | 50 | 50 |
| `germany-two-cities-one-turn` | 0 | 0 | 0 | 0 | 0 |

Told with `short_objective` on is unchanged (49 / 50 / 0): a told pin is someone else's operation,
which the step leaves alone.

What each did:

- **Biel:** the step mostly pins a Campaign on India's nearby cities (16, then 28), not a Hold.
  (Corrected in round 3, below: the forecast was right, Biel falls in 3 turns in all 50 runs; the
  win-chance currency was the problem.)
- **Dyrrachium:** both genes reach 50/50 on their own. The step pins Dyrrachium from T40;
  `strike_upgrade_wait` keeps Horseman #42 from upgrading on T37 (the card loop's finding).
- **Combined on Biel:** `strike_upgrade_wait` blocks Rifleman #3's T136 upgrade in 8 of 50 runs,
  even with a home-door exception (units within 4 tiles of a home city with an enemy within 4 of it).
  Combined with the step, Biel drops from 45 to 37. Keep them separate.
- **Two Cities without the supply cut:** the step first pinned Indian Bikaner on T144 in all 50
  runs, and Chalco fell 32 to 0. Bikaner's plan value was 20,626, of which `supply` was 20,592,
  about twice a certain win. Pricing the window without the supply term put the pin on Chalco
  from T145, back to the baseline (Chalco 32, Xochimilco 0).

Every Lab card, 50 runs each, seat only (wins, median score in brackets):

| Card | Off | short_objective | strike_upgrade_wait | Both |
|---|---|---|---|---|
| `athens-break-the-hold` | 0 (590) | 0 (580) | 0 (590) | 0 (580) |
| `aztecs-eight-by-seventy` | 4 (750) | 4 (750) | 4 (750) | 4 (750) |
| `aztecs-swiss-spearmen` | 0 (200) | 0 (200) | 0 (200) | 0 (200) |
| `byzantium-break-the-fire` | 0 (1040) | 0 (1040) | 0 (1040) | 0 (1040) |
| `celts-break-the-reserve` | 0 (740) | 0 (740) | 0 (730) | 0 (730) |
| `celts-parked-at-st-gallen` | 1 (730) | 1 (730) | 1 (730) | 1 (730) |
| `celts-steal-a-settler` | 0 (150) | 0 (150) | 0 (150) | 0 (150) |
| `celts-stop-the-horde` | 0 (710) | 0 (710) | 0 (710) | 0 (710) |
| `celts-strike-the-leader` | 0 (640) | 0 (640) | 0 (640) | 0 (640) |
| `germany-defend-essen` | 0 (760) | 0 (760) | 0 (760) | 0 (760) |
| `germany-take-gergovia` | 0 (670) | 0 (680) | 0 (670) | 0 (680) |
| `germany-two-cities-one-turn` | 0 (880) | 0 (880) | 0 (880) | 0 (880) |
| `hold-athens` | 49 (180) | 49 (180) | 49 (180) | 49 (180) |
| `india-no-niter` | 0 (950) | 0 (920) | 0 (950) | 0 (920) |
| `india-strike-the-leader` | 0 (2060) | 0 (2060) | 0 (2060) | 0 (2060) |
| `india-take-the-settler` | 0 (130) | 0 (130) | 0 (130) | 0 (130) |
| `mongols-eight-points` | 0 (1100) | 0 (1190) | 0 (1060) | 0 (1240) |
| `mongols-ride-to-dyrrachium` | 2 (360) | 50 (760) | 50 (750) | 50 (750) |
| `mongols-wide-india` | 1 (920) | 5 (920) | 2 (910) | 1 (920) |
| `sparta-before-the-walls` | 0 (370) | 0 (370) | 0 (370) | 0 (370) |
| `sparta-next-door` | 0 (840) | 0 (870) | 0 (840) | 0 (860) |
| `sparta-walled-nicomedia` | 3 (970) | 2 (970) | 3 (970) | 2 (970) |
| `swiss-hold-biel` | 34 (1200) | 45 (1200) | 31 (1200) | 37 (1190) |
| `swiss-hold-the-south` | 0 (480) | 0 (480) | 0 (480) | 0 (480) |
| `swiss-strongbox` | 13 (350) | 13 (350) | 13 (350) | 13 (350) |
| `swiss-walls-athens` | 0 (640) | 0 (640) | 0 (640) | 0 (640) |

`short_objective` changes these Lab cards' wins:

- Biel +11, Dyrrachium +48 and wide India +4.
- Walled Nicomedia -1, one run.

Medians move by at most 90 (Mongols eight points +90, India no niter -30). `strike_upgrade_wait`
changes Dyrrachium +48, Biel -3 and wide India +1.

War suite (19 cases from his playtests, 128 runs each, local, seat only,
`war-suite-short-objective-128.md`): the mean is +0.5 pp.

| Case | Change in goal met | 95% interval |
|---|---|---|
| Hold Biel (war cut) | +15.6 | [+7.8, +23.4] |
| Mongol bombers T150 | +15.6 | [+6.2, +25.0] |
| Athens two pikemen T112 | **-7.0** | [-11.7, -2.3] |
| India vs Sparta T142 | -7.0 | [-17.2, +3.9] |
| German air early | -4.7 | [-10.2, +0.8] |

The other 14 cases are within 1 point. The on arm took 224 s against 189 s.

What the war suite shows, and what it doesn't:

- **It shows:** in positions cut from his real games, played 4 to 44 turns, the step does not
  break wars on average. It also has one clear loss, the Athens defence case, which is not traced.
- **It doesn't show:** full games, the designer-seat score, AI-vs-AI censuses or turn time in WASM.
  The designer-seat tool measures a commit's defaults and takes no gene parameters, about 25
  minutes on 16 cores, so it was not run.
- Hold Biel in the suite is the same position as the Lab card, so the two are not independent.
- Biel and Dyrrachium are training cards that the step was built from.

## Two Cities: air-ground strike design (not built; `claude/air-war` is changing air rules)

Measured here: gene `breach` (off by default, the round-2 distilled "zero the city with shots and
aircraft, walk in") gives 0/50 untold, told and with `short_objective`. The melee units are never
next to the cities on the strike turn. The design:

1. **Air in the ground plan.** Today the move planner returns the ground orders and
   `Brain::add_air_orders` (`air.rs::air_orders_with_prizes`) assigns planes afterwards. Instead:
   - before `plan_military`, compute each in-range plane's expected damage on each enemy city in
     reach (the `forecast_siege_air` call already prices planes);
   - reserve those planes for the city the plan pins;
   - hand the reservation to the breach and campaign passes;
   - have `add_air_orders` honour the reservation first.
2. **A two-turn strike-and-walk-in objective.** The short-horizon step should also nominate a city
   whose HP the reserved planes and shooters zero next turn, with a melee unit that can be one tile
   away by then. Price it as a capture in 2 turns:
   - turn 1: stage the walkers adjacent, out of the city's strike if possible, and rebase planes in
     range;
   - turn 2: strike, then walk in, as the breach pass does.
3. **Pair the targets.** Pair targets by walker: each melee unit is assigned to at most one city,
   and two cities are struck the same turn when there are two walkers and the planes zero both
   (his line: two Bombers, one city each).
4. **Keep the strike force moving** (`strike_upgrade_wait`), and keep the walkers out of
   `door_military`'s home return when no home city is endangered (the Tank went home on T146).

Measure it on Two Cities untold against his own line, and on the war suite's two German air cases.

## Round 3: Athens loss traced, Biel gap traced, supply term measured, designer seat

Commit 114972075. Every number below is seat only, same seeds.

### Athens two pikemen T112 (-7.0 pp in round 2): a Hold pinned over the war at home

`war_suite` with `WAR_PLAN_TRACE=1,7` (now takes a run list and prints the short objective and the
operation), on runs the gene lost (1, 7, 29, 35, 53, 80, 83, 88, 114, 120 of 128):

- The memory restored at T112 has Athens' plan on Contain against Sparta, the team at its door.
- The step pinned a Hold on the most endangered city every turn a city was forecast to fall
  (Hold on city 9 on T113 and T115 to T118, then on cities 5, 22, 11 and 17).
- With the gene on, cities lost fell from 19 runs to 10, but the Pikemen lived. The objective
  needs both Pikemen dead, so 93 runs timed out on it, 111 with the gene.

This was a pricing error. The Hold is worth a home city's loss only against a plan that takes the
army away from home. A Contain war on the attacker already fights at home.

Fix (`short_objective.rs`): pin a Hold only when the routed plan is a Campaign, a Hold, none, or a
Contain against a team that isn't at our door. Rerun at 128 runs (Base = off from round 2):

| Case | Base | Round 2 | Round 3 |
|---|---|---|---|
| Athens two pikemen T112 | 12% | 5% | 12% |
| Hold Biel (war cut) | 65% | 81% | 98% |
| Mongol bombers T150 | 16% | 32% | 32% |

### Biel gap: not the forecast, the currency

Round 2 said the attacker-side forecast underrates Biel. That was wrong. I logged `home_fall`:
on T132 it says Biel falls in 3 turns in **all 50** runs (8 Indian units, both Cavalry beside it).
The 8 of 50 was how often the step pinned a Hold, not how often the forecast saw Biel fall.

The step chose Campaigns because it priced in the plan layer's win chance, which saturates for a
leader:

- The Swiss sit at c0 = 9,934 to 9,999 (99.3% to 99.99%).
- So Biel's expected loss came to 13 points.
- Every capture came to 4 to 35 points.
- From T133, every home loss priced at 0.

The forecast was not changed. The step now prices in Production, the tactical layer's currency:

- a city is worth 400 + 80 per population;
- a capture is worth that city at its capture odds, less the forecast's expected losses;
- a home city's expected loss is its fall odds times its worth.

The supply term goes with the win chance. Only cities of teams already at war, or of the team the
routed plan is about to declare on, are nominated; the step starts no war. With war targets only,
Dyrrachium fell to 14/50: the army marched on Nicomedia during T37 to T39, before the declaration.
The planned-war nominations restore it.

On T132 Biel's nominations now read: home loss 1,099 (Biel 387, two other cities 387 and 325),
the Campaign on city 16 at -825, the Hold at 0. The step holds.

| Card | Baseline | Round 2 | Round 3 | Told ceiling |
|---|---|---|---|---|
| `swiss-hold-biel` | 34 | 45 | **50** | 49 |
| `mongols-ride-to-dyrrachium` | 2 | 50 | **50** | 50 |
| `germany-two-cities-one-turn` | 0 | 0 | 0 | 0 |

Untold, round 3 beats the told arm on Biel (50 against 49). The told arm pins the quiet plan; the
step pins Holds, which `execute_hold` now defends well.

Every Lab card, round 3 against off (wins, median score in brackets):

| Card | Off | short_objective (round 3) |
|---|---|---|
| `athens-break-the-hold` | 0 (590) | 0 (580) |
| `aztecs-eight-by-seventy` | 4 (750) | 3 (750) |
| `aztecs-swiss-spearmen` | 0 (200) | 0 (200) |
| `byzantium-break-the-fire` | 0 (1040) | 0 (1050) |
| `celts-break-the-reserve` | 0 (740) | 0 (750) |
| `celts-parked-at-st-gallen` | 1 (730) | 1 (730) |
| `celts-steal-a-settler` | 0 (150) | 0 (150) |
| `celts-stop-the-horde` | 0 (710) | 0 (710) |
| `celts-strike-the-leader` | 0 (640) | 0 (640) |
| `germany-defend-essen` | 0 (760) | 1 (750) |
| `germany-take-gergovia` | 0 (670) | 0 (670) |
| `germany-two-cities-one-turn` | 0 (880) | 0 (880) |
| `hold-athens` | 49 (180) | 49 (180) |
| `india-no-niter` | 0 (950) | 0 (920) |
| `india-strike-the-leader` | 0 (2060) | 0 (2070) |
| `india-take-the-settler` | 0 (130) | 0 (130) |
| `mongols-eight-points` | 0 (1100) | 0 (1110) |
| `mongols-ride-to-dyrrachium` | 2 (360) | 50 (760) |
| `mongols-wide-india` | 1 (920) | 1 (950) |
| `sparta-before-the-walls` | 0 (370) | 0 (370) |
| `sparta-next-door` | 0 (840) | 0 (870) |
| `sparta-walled-nicomedia` | 3 (970) | 3 (970) |
| `swiss-hold-biel` | 34 (1200) | 50 (1190) |
| `swiss-hold-the-south` | 0 (480) | 0 (480) |
| `swiss-strongbox` | 13 (350) | 13 (350) |
| `swiss-walls-athens` | 0 (640) | 0 (640) |

Wins change on:

- Biel +16 and Dyrrachium +48;
- Defend Essen +1;
- Aztecs eight by seventy -1.

Medians move by at most 30.

War suite, 19 cases at 128 runs (`war-suite-short-objective-128-r3.md`): the mean is **+2.5 pp**.
Hold Biel's war cut gains +32.8 [+24.2, +41.4] and Mongol bombers T150 +15.6 [+6.2, +25.0]. No
case is significantly worse; the largest drops are India vs Sparta T142 -0.8 [-10.9, +9.4] and
Mongols Chur T137 -1.6 [-9.4, +7.0].

### Supply term: how often the priced supply arrives

`WAR_SUPPLY_LOG=1 war_suite` logs every plan switch to a Campaign with its `supply` term and the
seat's supply in use each turn. `supply_report.py` checks each switch:

- **Captured:** the target city was taken within the 10-turn plan horizon.
- **Collected:** the seat's strategic supply in use rose after the capture, inside that horizon.

Default genes, the war suite at 64 runs and the 26 Lab cards at 32 runs (`war_suite --dir
scenarios/lab`).

| | War suite | Lab cards | Both |
|---|---|---|---|
| Campaign switches | 3,394 | 1,616 | 5,010 |
| with a supply term | 1,982 | 696 | 2,678 |
| window ended before 10 turns (censored) | 962 | 279 | 1,241 |
| target captured within 10 turns | 129 | 203 | 332 |
| supply in use rose after the capture (collected) | 91 | 86 | 177 |
| supply term priced, sum | 4,520,388 | 2,012,944 | 6,533,332 |
| supply term of the collected, sum | 146,804 | 364,512 | 511,316 |
| median supply term / whole campaign value | 1.00 | 0.91 | 0.99 |

Of the 1,437 switches with a full 10-turn horizon, at most 23% took the city and 12% collected
supply. Those are upper bounds, because a capture or a supply rise inside a shorter window also
counts.

The term already multiplies by the forecast odds, usually 30% to 60%. In half the switches it is
as large as everything else in the campaign's value together. Only 7.8% of the priced supply term
belongs to switches where supply was actually collected in the horizon.

Proposed correction (not built, default pricing unchanged), as a default-off gene:

1. Count the supply term only for the plan-horizon turns left after the forecast capture, which
   is the second half of `capture_horizon` (already a gene).
2. Scale the odds the term uses by the measured collection rate against the forecast odds, about
   12% / 45%, so about a quarter. Recalibrate this from the same log after any change to captures.
3. Cap the term at the rest of the campaign's value, so a deposit alone can't double a target.

Measure it on the war suite and the Lab cards as here, then on the designer seat.

Caveat: "collected" counts any rise in supply in use after the capture, not only from the
captured deposit, so it is an upper bound too.

### Designer seat with the gene

`designer_seat.py` now takes `--seat-params FILE`. It plays the commit's Hard and Hard with
those parameters in the designer's seat on the same branch turns, salts and dice
(`seat_run --continue hard,params:FILE`), and writes both JSONs. It also takes `--jobs`, which
caps how many tapes run at once.

Run: commit 114972075, 48 salts, start turns T2 to T80, 12 threads, 6 tapes at a time, 30.7
minutes. No Legatum app was running and nothing was connected to the local server; the
`designer-seat-*.json` files hold the results.

| Tape | Set | Control | `short_objective` | Delta |
|---|---|---|---|---|
| Aztecs 8.9.29 | training | 24.2% | 25.4% | +1.2 |
| Germany 8.9.27 | training | 8.8% | 7.1% | -1.7 |
| Sparta 8.9.26 | training | 84.6% | 85.4% | +0.8 |
| Swiss 8.9.26 | training | 51.7% | 52.1% | +0.4 |
| India 8.9.25 | training | 11.2% | 11.2% | +0.0 |
| Mongols 8.9.25 | held out | 44.2% | 45.4% | +1.3 |
| Celts 8.9.26 | held out | 87.9% | 90.8% | +2.9 |
| Swiss 8.9.24 | held out | 44.2% | 42.9% | -1.2 |
| Celts 8.9.25 | retired | 1.2% | 2.1% | +0.8 |
| **Training set** | | 36.1% | 36.2% | **+0.2** |
| **Held-out set** | | 58.8% | 59.7% | **+1.0** |

What it shows:

- Over whole games from his positions, the gene costs nothing measurable: no tape drops by more
  than 1.7 points.
- It also gains nothing measurable: the tool's own note puts run-to-run noise at about 3 pp on the
  held-out score.
- The arms share seeds, but this is not a paired interval.

What it does not show:

- AI-only censuses or turn time in WASM.
- Whether the AI's rivals get harder. Only the designer's seat plays the gene.
- The held-out 58.8% control here is on rules 26 at this commit, not the 2026-11-07 control.
