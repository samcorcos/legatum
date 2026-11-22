# Decision audit, round 4: where Hard sends its Settlers, and shooters that move instead of killing (2026-10-31)

Branch `claude/audit-round4` from master `530af07cf` (rounds 1 to 3 merged). Three genes, all AI decisions only, all
default 0: `border_explained` and `fallback_risk` (together "site", the Settler's destination), and `shot_sweep`.
Runner `audit-round4/run.sh` (suite, census, trace on the box), arms in `audit-round4/params/` and
`audit-round4/arms*.json` (the quarter gates, spot), outputs in `audit-round4/out/`, analysis scripts beside them.

**Verdict.** The site genes fix the settling pattern the diff found and are the first audit-led change since
round 2 to show a clear quarter-gate gain on three independent salt sets: seat-only **+5.99 pp [+3.91, +8.14]** on
1,536 paired games, rivals-only +2.21 [+0.13, +4.30] (positive = stronger AI both ways), with the seat gain on 7 of
8 maps. The audit agrees in direction but not in size: the settling share halves (+0.070 to +0.037 cities @20),
the whole policy does not move (-0.006). `shot_sweep` does what it targets in the diff (12 skipped kills by shooters
to 7, every swept shot a kill) and nothing in the audit or gate. **Recommend the full 2,048-game gate for
`border_explained` 1 + `fallback_risk` 1 together** (not alone: each alone reads weaker rivals). Keep `shot_sweep` off.

## 1. The ranking (unchanged from round 3)

The control arm reproduces round 3's audit to the third decimal (whole +0.358 cities @20, army +0.166, garrison
+0.147, settling +0.070, diplomacy +0.053, production +0.011; VP @50 production +0.130), so the round 3 table stands:
army and garrison first, settling third, production a VP gap @50 carried by Sparta (+0.37). `out/audit-c20.txt`,
`audit-c50.txt`, `audit-v50.txt`.

## 2. Lead 1: where the search sends Settlers

New diagnosis: `decision_audit --settle-detail` now writes a row for **every Settler of the seat on every turn**, and
`ScriptedBrain::settler_diagnosis` takes a probe tile: the tile the designer later founds on with that Settler. The
site search records its verdict on that tile ("probe tile rejected: ...", "not preferred", "not a candidate: ...",
or its value), plus, for a border verdict, the evidence behind it. `audit-round4/site_diff.py DIR` classes the rows.

Four games, 163 Settler-turns before his founding turn and 37 founding turns (T1 capitals left out). Where Hard's target was not his tile:

| Why his tile lost in Hard's search (control) | Settler-turns | Hard's choice instead |
|---|---:|---|
| **Not a candidate: a foreign border could hide a city** | 30 (all four games) | a site 2 to 10 hexes farther, a weak fallback or home |
| Lower value (travel and land quality) | 21 | a nearer-scored site; a fair trade |
| **Rejected for trip risk** (`legacy_settler_trip_risk` at 100 or more) | 16 | mostly the far fallback |
| Rejected, unreachable (wartime route around danger) | 11 | |
| Rejected, enemy army within 3 | 5 | |
| Hard's Settler exposed where it stands: retreat | 28 (Swiss 17) | |
| No site at all: home | 12 | |

Where Hard's target differs, it takes a fallback 31 times (mean trip 8.5 hexes against his 4.9) and a preferred site
51 times (6.2 against 4.6). Two narrow causes, both in `civilians.rs::choose_site_scored`:

1. **Border risk ignores the city it already knows.** `settlement::border_city_risks` treats every visible foreign
   tile as evidence of an unseen city within the work radius, even when a known city of that owner sits one or two
   hexes from it. In **29 of the 30** cases every foreign tile behind the verdict lay within 2 of a known city of its
   owner (`border tiles explained by a known city within 3: N, not: 0`), and the "possible centre" was explored land.
   The AI's own rule for remembered borders (`remembered_border_risks`) already treats a border near a known city as
   explained; the live one did not.
2. **The fallback is never charged trip risk.** A preferred site is dropped at a legacy trip risk of 100 (one enemy
   unit whose reach plus two touches the route's ellipse), but a non-preferred fallback site skips the check
   entirely. So a near site with one enemy unit about loses to a site 6 to 11 hexes away past the same units. With
   the border fix on, this became the top reason (26 Settler-turns, 15 of them to a fallback).

### The genes

- **`border_explained`** 1 (`civilians.rs::unexplained_border_risks`, `settlement::settlement_candidates_with_risky`):
  the site search also lists border-risk sites, and drops only those an *unexplained* border puts at risk: a visible
  foreign tile within a known city's work radius of a city of the same owner implies no unseen city. Such a site's
  preferred-quality check uses `SiteBasics::legal_base_score` (the planner's `site_score` is empty for them). Computed
  once per planning pass.
- **`fallback_risk`** 1: when every preferred site is out and the search would take a fallback, the fallback's own
  trip risk is compared with the least risky preferred site dropped for trip risk; a fallback no safer loses to that
  site (its value discounted by the risk as for a kept site).

Diff, four games (`out/diff-sites-*.txt`):

| | Control | `border_explained` | both |
|---|---:|---:|---:|
| Settler-turns where Hard targets his tile | 38 | 44 | **48** |
| ... his tile left out for border risk | 30 | 3 | 3 |
| ... Hard goes home | 12 | 7 | 7 |
| ... Hard takes a far fallback (mean trip, his) | 31 (8.5, 4.9) | 35 (6.7, 4.9) | 6 (5.0, 7.2) |
| His founding turns where Hard does not found | 15 | 12 | 11 |

`fallback_risk` swapped the fallback for the dropped near site in 29 of 31 cases (`fallback ... trip risk` notes).
What is left: the Swiss Settlers that wait exposed for him (17, his choice to wait beside a guard), lower-value trades,
and wartime routes Hard cannot plan around danger.

## 3. Lead 2: the skipped kills, across passes

`audit-round4/kills.py` regroups the 42 kills he made that Hard skipped (round 3 counted them by last pass):

| Shared cause | Kills | Passes behind Hard's order |
|---|---:|---|
| Melee unit held or pulled back | 15 | defend_cities 3, door_military 3, reserve_front_guards 2, press_strike 2, hold_captures 2, 3 others |
| **Shooter moved instead of firing** | **12** | garrison_cities 5, recover_wounded 3, leave_strike_range 2, press_strike 1, seize_open_city 1 |
| Same order, the kill went to another unit (a joint kill) | 9 | |
| Upgrade instead (`upgrade_wait`, round 3) | 5 | |
| Attacked without killing | 1 | |

The one shared cause: a shooter fires only on an explicit order, the fire passes (`focus_fire`, `ranged_fire`) see
only units nothing has ordered yet, and `fire_from_holds` converts only a Skip. A move from an earlier pass (the
routine garrison, leaving a city's strike range, the wounded's retreat, staging) wins by running first.
`strike_kill` (on) covers the opportunity strike's force only, and only by leaving it free. The melee 15 have no
shared pass: the top-down layers that failed three times.

- **`shot_sweep`** 1 (`army_commit.rs::shot_sweep`, after `hold_stragglers`, before `fire_from_holds`): a shooter
  ordered to move (not onto an enemy) that can kill a visible enemy field unit from where it stands, given the shots
  already ordered (kill chance at least 50%, `focus::shot_kill_chance`), fires instead when the target's cost times the
  kill chance beats its own cost times its loss chance if it stays (`incoming_at` without the target), unless it is the
  holder that keeps an own city from falling this turn.

Diff: shooter misses 12 to 7; it fired 5 times, 5 kills. The 7 left need two shots (his kills there finished a target
another unit had wounded) or are a staging move into a stack.

## 4. Measured

### Decision audit (window 10, 32 salts, `--alt` and `--continue` the arm; change against the control)

| Change vs control, mean of 4 games | site | `shot_sweep` | all three |
|---|---:|---:|---:|
| Whole, cities @20 | -0.006 | -0.002 | -0.007 |
| **Settling share, cities @20 (target)** | **-0.033** | 0.000 | -0.033 |
| Army / garrison, cities @20 | -0.021 / +0.012 | -0.001 / +0.003 | -0.025 / +0.016 |
| Whole, cities @50 | +0.012 | +0.007 | +0.025 |
| Whole, VP @50 | -0.024 | +0.016 | -0.009 |

The settling share closes where the diff found the pattern, Mongols -0.09 ±0.04 and Swiss -0.12 ±0.03 (windows
Mongols T60 +0.72 to -0.19, Swiss T120, T130, T150), and opens in Sparta +0.06 ±0.02 (one window, T90, -0.05 to
+0.41, whose diff decisions are identical, so it comes from later play in the branch) and Athens +0.02. The whole
policy moves Mongols -0.10, Swiss -0.06, Athens +0.05, Sparta +0.08: a wash. `audit-round4/turn_change.py` traces a
category per window.

### Quarter gate (`hard_vs_human --arms`, 8 maps x 64 salts, reseeded dice, plain Hard rivals; `w04_gate.py`)

Strength in pp, positive = stronger AI (seat-only +delta, rivals-only -delta). Three batches on different salts, one
binary (`3e0881ff`, `c1a25ceb03886843`), each paired with its own control. `out/gate*-summary.json`, `gate*-maps.txt`;
pooled by `audit-round4/gate_pool.py` (map-stratified bootstrap over the pooled pairs).

| Arm | Batch 1 (270001, ctl 102) | Batch 2 (271001, ctl 83) | Batch 3 (272001, ctl 99) | Pooled |
|---|---|---|---|---|
| site, seat-only | +3.12 [-1.17, +7.42] | +8.40 [+4.39, +12.41] | +6.45 [+2.44, +10.45] | **+5.99 [+3.91, +8.14]** |
| site, rivals-only | +2.93 [-1.22, +7.08] | +0.20 [-3.53, +3.92] | +3.52 [-0.55, +7.58] | **+2.21 [+0.13, +4.30]** |
| `shot_sweep`, seat-only | -1.37 [-3.19, +0.45] | | +1.76 [-0.20, +3.72] | +0.20 [-1.07, +1.46] |
| `shot_sweep`, rivals-only | 0.00 [-2.77, +2.77] | | +1.17 [-1.07, +3.42] | +0.59 [-1.17, +2.34] |
| all three, seat / rivals | +2.73 / +0.78 | | | |
| `border_explained` alone, seat / rivals | | +3.71 / **-3.91** | | |
| `fallback_risk` alone, seat / rivals | | +0.59 / -2.54 | | |

- The site seat gain is spread: in batch 3 it gains on 7 of 8 maps (8.4 Sparta 29 to 37, Germany 10 to 14, Celts 3
  to 8, 8.5 Switzerland 2 to 6), in batch 2 on 6 (Germany 3 to 15, 8.4 Sparta 27 to 43). The round 3 lesson (a seat
  effect on one map does not replicate) does not apply here, but the full gate is still the judge.
- The seat's cities rise: at T100 4.84 to 5.02 (batch 1), 4.89 to 5.13 (batch 2), 4.84 to 5.02 (batch 3).
- The two genes interact: each alone reads *weaker* rivals, together stronger. The border fix lets near sites in,
  and without `fallback_risk` many of them are then dropped for trip risk in favour of a far fallback (the diff's
  legacy-risk rejections rose 16 to 26). Gate them as one change.

### War suite, census, decision cost

- War suite (64 runs, 33 cases): 38% in every arm, seat and all seats (`out/suite-*.log`).
- All-AI census (512 games, the arm on every seat, paired by seed; `out/census-paired.txt`), site: cities per game at
  T100 **+1.61 ±0.23**, T150 **+2.02 ±0.25**, at the end +1.14 ±0.32; captures +1.42 ±0.74 and cities lost +1.48
  ±0.84 (more cities to fight over). `shot_sweep`: every measure inside ±0.5.
- Cost, 16 census games back to back on the Mac, 4 threads, ms per seat-turn: control 1.417, 1.450, 1.424, 1.436;
  site 1.496, 1.459, 1.500; `shot_sweep` 1.458; all three 1.510, 1.518. About +0.04 to +0.07 ms (3 to 5%), part of it
  from bigger empires (more cities to plan). The box census timings are not comparable (24 audit jobs sharing it).

## 5. Risks

- **Founding beside a known foreign city.** The border fix lets the Settler found 4 hexes from a neighbour's city.
  If that owner has a second, unseen city whose border touches the first one's, the found fails (one turn lost; the
  tile goes to `failed_city_sites`). Not seen in the diff.
- **Founding into trip risk.** `fallback_risk` sends a Settler to a near site with an enemy unit about, because the
  far alternative is no safer by the legacy measure. The census's cities lost +1.48 ±0.84 per game (8 empires)
  against +1.61 more cities is the price; the net stays positive to T150 and at the end. The legacy trip risk
  saturates (100 per unit), so "no safer" compares coarse numbers.
- **Interaction:** `border_explained` alone made rivals weaker in its one batch; ship the pair or neither.
- **Faction mix:** in batch 1 the Japan seat won 13 to 5 with the genes (more cities, 6.72 to 7.20 at T100); batches
  2 and 3 read 6 to 9 and 11 to 16. Japan's decadence and network penalty make extra cities a mixed blessing; watch
  it in the full gate.
- `shot_sweep` holds a shooter in place one more turn (inside a city's strike range or wounded) whenever it kills;
  its loss check sees only the visible enemy.

## 6. Readiness

Run the full 2,048-game gate on **`border_explained` 1 + `fallback_risk` 1 together**, seat-only and rivals-only
(fresh salts, the frozen-control method). The quarter gates support it on three salt sets and the census shows the
mechanism (more cities by T100); the audit supports the direction (settling share halved) but not a whole-policy gain,
so the gate decides. `shot_sweep` is correct but too rare to matter: keep it off.

## Reproduce

```
# diff-only, every Settler-turn with the probe, and army rows (local, this checkout, one game at a time)
tools/decision-audit/run.sh --local --engine here --out DIR --ship params/ARM.json SAVE... -- \
  --diff-only --settle-detail --army-detail --alt params:audit-in/ARM.json
python3 docs/sim-results/ai-war/audit-round4/site_diff.py DIR [--show] [--class PREFIX]
python3 docs/sim-results/ai-war/audit-round4/kills.py DIR [--show PREFIX]
# value audits: as round 3 (--engine COMMIT, window 10, 32 salts, --alt and --continue the arm)
# quarter gates, one spot server each (about $0.04 and 4 minutes)
LEGATUM_CLOUD_EXAMPLES=hard_vs_human tools/cloud/build-linux.sh
tools/cloud/run-batch.sh --count 1 --max-hours 1 --yes --bin hard_vs_human --ai-params docs/sim-results/ai-war/audit-round4/arms.json \
  --env LEGATUM_HVH_RESEED_DICE=1 -- --arms --games 64 --seed 270001 --label r4-quarter
python3 tools/ai-quality/w04_gate.py OUT BATCH_DIR
python3 docs/sim-results/ai-war/audit-round4/gate_maps.py BATCH_DIR [ARM...]
python3 docs/sim-results/ai-war/audit-round4/gate_pool.py ARM BATCH_DIR...
# suite and census on the box
tools/cloud/remote.sh . --artifact docs/sim-results/ai-war/audit-round4/out -- docs/sim-results/ai-war/audit-round4/run.sh suite|census ARM...
python3 docs/sim-results/ai-war/audit-round4/census_paired.py docs/sim-results/ai-war/audit-round4/out site sweep all
```

Cost: about 60 box jobs (48 value audits, the suite, the census, builds and tests) on the shared box, roughly $0.70
at $0.73 an hour; three spot batches, 0.18 server-hours, about $0.12.
