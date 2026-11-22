# Decision audit, round 5: Peace offers Hard signs while it is winning, and Settlers that leave held cities (2026-10-31)

Branch `claude/audit-round5` from master `cabdbfd5f` (rounds 1 to 4 merged; `border_explained` and `fallback_risk`
on), with master `edb8d75a2` (audit continuations) merged in. Two genes, both AI decisions only, both default 0:
`peace_strong` and `settler_shelter`. Runner `audit-round5/run.sh` (suite, census, cost on the box), arms in
`audit-round5/params/` and `audit-round5/arms.json` (the quarter gates, spot), outputs in `audit-round5/out/`, analysis
scripts beside them.

**Verdict.** Both genes fix what they target in the diff, and neither moves the quarter gate. `peace_strong` is the
first gene since round 2 to close a clear piece of the whole-policy gap in the audit: **-0.076 cities @20** (Sparta
**-0.36 ±0.08**, from his conquest windows T90 to T110), but the gain does not carry to VP @50 (+0.016, the wrong
sign, inside noise), and the quarter gate pooled over three salt sets reads seat-only -0.13 pp [-1.69, +1.43] and
rivals-only -1.04 [-2.73, +0.72]. The cities gain is robust across master's three continuation policies (hard, economic,
conquest: Sparta -0.36, -0.32, -0.24, each ±0.08); the VP @50 null is robust too. `settler_shelter` turns 11 of the
12 in-city retreats into waits and leaves the audit flat (-0.002) and the gate at seat-only -0.98 [-2.41, +0.46]. **Do not run the full gate on either.** The
audit and the gate disagree for `peace_strong`, and the quarter gates do not support it.

## 1. The ranking on today's master (window 10, 32 salts, four games, mean of game means)

Engine `cabdbfd5f`, the same four games, turn sets and salts as rounds 3 and 4 (`out/audit-c20.txt`, `audit-c50.txt`,
`audit-v50.txt`, the `ctl` rows).

| Category | Cities @20, r4 | **r5** | Cities @50 | VP @50, r4 | **r5** |
|---|---:|---:|---:|---:|---:|
| Whole policy | +0.358 | **+0.317** | +0.374 | +0.234 | **+0.173** |
| Army | +0.166 | **+0.147** | +0.110 | +0.016 | +0.012 |
| Garrison | +0.147 | **+0.143** | +0.177 | +0.053 | +0.049 |
| Diplomacy | +0.053 | +0.051 | +0.082 | n/a | +0.029 |
| Settling | +0.070 | +0.041 | +0.023 | +0.015 | +0.004 |
| Production and buys | +0.011 | +0.017 | +0.052 | +0.130 | **+0.047** |
| Research | +0.007 | -0.001 | +0.011 | n/a | +0.032 |
| Other (Raze) | +0.006 | -0.005 | -0.019 | n/a | -0.004 |

- The site genes closed the whole-policy gap by 0.04 cities @20 and 0.06 VP @50 since round 4. The production VP gap
  shrank from +0.130 to +0.047; Sparta is still +0.32, now mostly its T10 to T20 opening (+1.36 VP in T10), which
  is the opening rush, not peacetime economy. Athens reads -0.20 (Hard's production is better there).
- Army and garrison are still first and still lumpy: by window the garrison share is Sparta T100 +1.70, T80 +1.39,
  Athens T150 +1.09, T160 +1.06, Swiss T150 +1.12, Mongols T160 +1.02 (and T150 -1.88). No single pass explains them
  (section 4).
- Diplomacy is third and the one with a pattern behind it: its windows are Sparta T100 +1.42, Athens T160 +1.20,
  Swiss T70 +0.67 and T160 +0.62 (Mongols T150 -1.98, where Hard's second declaration was better than his play).

## 2. Lead 1: Peace offers he refuses and Hard signs

New diagnosis: `decision_audit --peace-detail` writes `peace.jsonl`, one row per Peace offer the seat must answer, with
his answer, Hard's, and `ScriptedBrain::peace_diagnosis` (the gate that decided, the score and its terms).
`PeaceDiagnosis` now also lists every own city `endangered()` weighs (HP, pressure, defence, the proposer's share of
the pressure, resistance). `audit-round5/peace.py DIR` prints the disagreements.

Four games, 25 Peace offers he had to answer. They agree on 12. **He refused 11 that Hard would sign**, he accepted 2
that Hard would refuse (`out/diff-peace-ctl.txt`):

| Hard signs, he refuses | Offers | Why the score was positive |
|---|---:|---|
| Our army at least 1.5 times the proposer's, "endangered" +30 | **8** (Sparta T65, T87, T112, T119; Athens T160, T174; Swiss T167, T176) | base 10 + war age 20 + endangered 30 beat army advantage -25 and "truce forgoes pressing" -25: score 5 to 32 |
| Our army about even, endangered | 1 (Sparta T105) | score 49 |
| Not endangered | 2 (Athens T148, Swiss T149 dead war) | score 13 and 53 |

The cause is one term of `Planner::peace_terms`: **"endangered" adds 30 whenever any own city has visible enemy
pressure above its local defence or stands at 100 HP or less**, and it does not look at how the war is going.
In the eight cases a freshly captured city in resistance (0 to 20 HP) or a thin new garrison triggered it (Sparta T112:
city 15, pressure 32 against defence 10, and the captured city 19 at 0 HP) while Sparta's army was more than half
again the proposer's and the plan was the campaign against that proposer. He kept fighting and kept taking cities
(Sparta 14 to 17 cities over T105 to T119).

### The gene

- **`peace_strong`** 50 (`diplomacy.rs::peace_terms`): when our army is at least (100 + gene) percent of the
  proposer's (the army-advantage term counts at most 50, so 50 is the most the gene can ask), danger to our cities adds
  nothing to the Peace score. The plan's gates, every other term and the VP cost are unchanged.

Diff (`out/diff-peace-peace.txt`): disagreements 13 to 7, agreements 12 to 18. Six of the eight flip to his answer
(Sparta T65, T87, T112, T119, Athens T174, Swiss T167); Athens T160 stays (army ratio 1.46) and Swiss T176 stays
("own hold" +40). No offer he signed is refused.

## 3. Lead 2: Settlers that retreat out of held cities

Of the 28 Settler-turns where Hard's Settler retreated as exposed and he did something else (round 4's pattern,
unchanged on today's master, `out/diff-exposed-ctl.txt`):

| Where his Settler stood | Settler-turns | What he did |
|---|---:|---|
| **In one of his cities** | **12** (Swiss 8, Mongols 3, Athens 1) | waited there (Skip) until the way was clear, then walked out |
| Stacked with his own military unit | 11 | walked or waited with the escort |
| Beside a unit of his | 3 | |
| Alone, as far as the diff shows | 4 | |

The cause for the first group: `tactics.rs::unit_exposure_key` gives a military unit in a city the city's shield (a
city the worst-case attack cannot break this turn takes every attack), but for a civilian it counts every enemy that
can reach the tile, as if the city were open ground. So Hard walks a Settler out of a garrisoned, full-HP city to a
tile farther from the enemy, or to another city (Swiss T113 to T115: (5,13) to (3,15), three turns running). A
civilian in a city can only be taken with the city (`rules/movement.rs`: a hostile city's tile is entered only at 0 HP).
The stacked-escort group is a different stance: Hard's field units treat any exposed tile outside a city as
unsustainable (`exchange_is_sustainable`), so a guard-aware Settler would need the escort held with it; not attempted.

- **`settler_shelter`** 1 (`tactics.rs::unit_exposure_key`): a Settler in an own city whose shield is Full
  (`city_shield`, the test garrisons already use) is not exposed there; it neither retreats from that city nor avoids
  it as a retreat tile. With no safe route out, `move_settler` already waits (Skip).

Diff (`out/diff-exposed-shelter.txt`, `diff-sites-shelter.txt`): in-city retreats 12 to 1; "exposed: retreat" 28 to
17; "Hard keeps it home" 7 to 15 (the wait he made).

## 4. Garrison and army: what the diff shows, and why no gene

`audit-round5/garrison.py DIR` classes every garrison-category unit-turn the two sides leave differently
(`out/diff-garrison-ctl.txt`). The largest class, "he keeps the unit in the city, Hard moves it out" at war, is 228
unit-turns, but 158 of them are Scouts (he parks them, Hard's `plan_scouts` walks them a hex out): cheap moves, not
where the value is. The rest spread over `defend_cities` (holding units he sends out), `campaign`,
`garrison_cities#2` (pulling a unit back to an interior city, Swiss T150 to T179), `reserve_front_guards`, and
upgrades instead of attacks (`cities`, Sparta T100 to T108: the round 3 `upgrade_wait` pattern, left off as
instructed). The skipped kills are unchanged from round 4 (`out/diff-kills-ctl.txt`: 15 melee holds across 8 passes,
12 shooters moved, 5 upgrades). No narrow shared cause, so no gene: the top-down layers are what failed before.

Production: `decision_audit --prod-detail` (`prod.jsonl`) records what the seat's Hard scored per city. Of the 508
city-turns where he builds an economy building and Hard a unit, only 74 come from the routine scorer
(`out/diff-prod-ctl.txt`): 47 of those are the Hard ranged minimum (value 3,300 while shooters number under half the
cities, at most 2), 6 the garrison minimum. The other 434 come from rules that set production before the scorer
(threat, door, emergency and modernization passes). With the production share at +0.017 cities @20 and +0.047 VP @50
there is too little left to chase this round; the tool is there for a later one.

## 5. Measured

### Decision audit (window 10, 32 salts, `--alt` and `--continue` the arm; change against the control)

| Change vs control, mean of 4 games | `peace_strong` | `settler_shelter` | both |
|---|---:|---:|---:|
| **Whole, cities @20** | **-0.076** (Sparta -0.36 ±0.08) | -0.002 | -0.079 |
| Diplomacy share, cities @20 (target) | -0.024 (Sparta -0.11 ±0.04) | -0.003 | -0.028 |
| Settling share, cities @20 (target) | -0.008 | +0.011 (Mongols +0.03) | +0.002 |
| Army / garrison, cities @20 | -0.023 / -0.022 | +0.002 / -0.006 | -0.022 / -0.028 |
| Whole, cities @50 | -0.056 (Sparta -0.24 ±0.23) | -0.027 | -0.079 |
| Whole, VP @50 | +0.016 | -0.011 | +0.013 |

`peace_strong` closes Sparta's windows T90 (-1.00), T100 (-1.88) and T110 (-0.78) cities @20, the windows of the
refused offers: Hard keeps its campaign and takes the cities he took. At VP @50 the same windows move +0.03, +0.34
and +0.22 (his VP lead grows a little), so the extra cities do not turn into VP within 50 turns. Negative is a
smaller gap; `audit-round4/turn_change.py` traces a window.

**Robustness across continuations** (master's `--continue` list; Sparta, Swiss and Athens, the three games with
refused offers; `out/audit-continuations.txt`). The control ran with `hard,economic,conquest`, `peace_strong` with the
same three policies each carrying the gene (`params/peace-economic.json`, `peace-conquest.json`). The gene has to be in
every continuation: in `decision_audit.rs::branch` the seat's live brain in the 10-turn window is the continuation's,
so `--alt params:GENE` with plain continuations changes only the branch turn (its alt rows matched the control's
256 of 256 per window).

| Sparta, `peace_strong` minus control | hard | economic | conquest | Robust |
|---|---:|---:|---:|---|
| Whole, cities @20 | -0.36 ±0.08 | -0.32 ±0.08 | -0.24 ±0.08 | **yes** |
| Diplomacy share, cities @20 | -0.11 ±0.04 | -0.11 ±0.04 | -0.10 ±0.04 | **yes** |
| Whole, cities @50 | -0.24 ±0.23 | -0.30 ±0.27 | -0.05 ±0.23 | no |
| Whole, VP @50 | +0.04 ±0.09 | -0.01 ±0.09 | +0.06 ±0.09 | no (zero) |

Swiss and Athens move by 0.06 or less under every policy. So the mistake is real and holds under every way of
playing on: refusing these offers keeps Sparta's conquest going for the next 20 turns. It does not hold up as a VP
gain at 50 turns under any of them.

### Quarter gate (`hard_vs_human --arms`, 8 maps x 64 salts, reseeded dice, plain Hard rivals; `w04_gate.py`)

Strength in pp, positive = stronger AI (seat-only +delta, rivals-only -delta). Three batches on new salts (273001,
274001, 275001; the full gate used 280001), one binary (`ed51b174b`, `b3eb899b4d7c21b8`), each with its own control
(102, 94 and 103 of 512). `out/gate*-summary.json`, `gate*-maps.txt`, pooled by `audit-round4/gate_pool.py`
(`out/gate-pooled.txt`).

| Arm | Batch 1 | Batch 2 | Batch 3 | Pooled (1,536 games) |
|---|---|---|---|---|
| `peace_strong`, seat-only | -1.37 [-4.10, +1.37] | +1.56 [-1.21, +4.33] | -0.59 [-3.32, +2.15] | **-0.13 [-1.69, +1.43]** |
| `peace_strong`, rivals-only | -1.56 [-4.67, +1.55] | -1.76 [-4.89, +1.37] | +0.20 [-2.87, +3.26] | **-1.04 [-2.73, +0.72]** |
| `settler_shelter`, seat-only | -1.56 [-4.26, +1.14] | 0.00 [-2.84, +2.84] | -1.37 [-3.93, +1.20] | **-0.98 [-2.41, +0.46]** |
| `settler_shelter`, rivals-only | -0.39 [-3.84, +3.06] | +0.20 [-3.20, +3.59] | +1.37 [-2.13, +4.86] | +0.39 [-1.56, +2.41] |
| both, seat-only | -1.76 | +1.17 | -1.17 | -0.59 [-2.47, +1.24] |
| both, rivals-only | -3.12 | -1.37 | -0.39 | -1.63 [-3.78, +0.46] |

- No arm is outside noise on any batch. The `peace_strong` rivals arm reads weaker rivals on two of three batches: a
  rival that refuses an AI neighbour's Peace while ahead keeps the AI-against-AI war going, the 10-31 lesson that AIs
  fighting each other leave the designer's seat to grow. In his seat the same refusal is what wins.
- The seat's cities at T100: 5.04 control, 5.03 `peace_strong`, 5.09 `settler_shelter` (batch 1).

### War suite, census, decision cost

- War suite (64 runs, 33 cases): 38% in every seat arm; all-seat arms 38% (`shelter`) and 37% (`peace`, `both`):
  with every seat on the gene the Spartan attacker refuses Japan's offers more often and `war-japan-sparta-t142` (Japan
  survives by signing) drops 17% to 9% [-15.6, 0.0], the stronger-rival direction (`out/suite-*.log`).
- All-AI census (512 games, the arm on every seat, paired by seed; `out/census-paired.txt`): every measure inside its
  interval or nearly so. `peace_strong`: cities at T150 -0.20 ±0.15, at the end -0.27 ±0.29, captures +0.31 ±0.72;
  `settler_shelter`: cities at T100 +0.16 ±0.18, at the end -0.02 ±0.32.
- Decision cost, ms per seat-turn (16 all-AI census games per arm, 4 threads, the arms interleaved twice in one box
  job while the audits shared the box; `run.sh cost`): control 2.134 / 2.150, `settler_shelter` 2.213 / 2.224 (about
  +0.08, 4%: `city_shield` for every Settler exposure check), `peace_strong` 2.108 / 2.112, both 2.127 / 2.127. The
  peace gene costs nothing measurable; the census job's +0.3 to +0.6 ms came from a loaded box.

## 6. Risks

- **`peace_strong` keeps wars going that a strong AI would have ended**, which is good in his seat and can feed
  AI-against-AI wars as a rival (rivals-only -1.04 pooled). The army ratio is the public strength estimate; a rival
  that hides units, or a far front, can make "half again stronger" wrong. The term only removes the +30; the war-age,
  "own hold", VP and plan gates still sign Peace.
- **`settler_shelter` keeps a Settler inside a city for as long as the way out is unsafe**, which can idle it for
  several turns (the diff: "home" 7 to 15 Settler-turns). The shield is computed from the visible attack; an army in
  the fog that breaks the city captures the Settler with it, as it would any garrison.
- The merge commit from master (`e22d8a98c`) carries git's default message without the attribution line; the
  permission system refused the reset needed to redo it, and amending is not allowed.

## 7. Readiness

**Do not run the full 2,048-game gate on either gene.** `peace_strong` has the audit's support (the whole-policy gap
in cities shrinks where the pattern is) but not VP @50, and three quarter gates read it neutral in the seat and
slightly weaker as rivals. `settler_shelter` is correct and too rare to matter. If the lead wants one more look,
the candidate is `peace_strong` seat-only, and the evidence that would justify it is a VP gain in his conquest
windows, which this round did not find.

## Reproduce

```
# diffs with the new detail files (box, any committed engine; one game each, in parallel)
tools/decision-audit/run.sh --engine COMMIT --out DIR --ship params/ARM.json SAVE... -- \
  --diff-only --settle-detail --army-detail --prod-detail --peace-detail --alt params:audit-in/ARM.json
python3 docs/sim-results/ai-war/audit-round5/peace.py DIR        # Peace answers and the endangered evidence
python3 docs/sim-results/ai-war/audit-round5/exposed.py DIR      # Settler retreats: in a city, stacked, beside
python3 docs/sim-results/ai-war/audit-round5/garrison.py DIR     # garrison unit-turns by where each side leaves them
python3 docs/sim-results/ai-war/audit-round5/prod.py DIR; python3 docs/sim-results/ai-war/audit-round5/prod_cover.py DIR
# value audits as round 3 (window 10, 32 salts, --alt and --continue the arm); robustness with continuations:
#   control: --ship tools/decision-audit/continuations/{economic,conquest}.json ... --continue hard,params:audit-in/economic.json,params:audit-in/conquest.json
#   gene:    --ship params/{peace,peace-economic,peace-conquest}.json ... --continue params:audit-in/peace.json,params:audit-in/peace-economic.json,params:audit-in/peace-conquest.json
#   (rows carry "continuation"; split them per policy before audit_shares.py)
# quarter gates, one spot server each (about $0.04 and 4 minutes)
LEGATUM_CLOUD_EXAMPLES=hard_vs_human tools/cloud/build-linux.sh
tools/cloud/run-batch.sh --count 1 --max-hours 1 --yes --bin hard_vs_human --ai-params docs/sim-results/ai-war/audit-round5/arms.json \
  --env LEGATUM_HVH_RESEED_DICE=1 -- --arms --games 64 --seed 273001 --label r5-quarter1
python3 tools/ai-quality/w04_gate.py OUT BATCH_DIR
python3 docs/sim-results/ai-war/audit-round4/gate_pool.py ARM BATCH_DIR...
# suite, census and cost on the box
tools/cloud/remote.sh . --artifact docs/sim-results/ai-war/audit-round5/out -- docs/sim-results/ai-war/audit-round5/run.sh suite|census|cost ARM...
python3 docs/sim-results/ai-war/audit-round4/census_paired.py DIR shelter peace both
```

Cost: about 100 box jobs (48 value audits, 29 continuation audits, 7 diffs, the suite, the census, the cost runs,
builds and two verifies) on the shared box, roughly $1.50 at $0.73 an hour; three spot batches, 0.16 server-hours,
about $0.11.
