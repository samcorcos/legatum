# Decision audit, round 3: the ranking on today's master, and two narrow genes (2026-10-31)

Branch `claude/audit-round3`. Control: master `69b371941` (`raze_value` 1, `buy_hold` 3, `capture_hold` 1,
`strike_kill` 1 all on). Genes `found_here` and `upgrade_wait`, both AI decisions only, both default 0. Runner
`audit-round3/run.sh`, arms in `audit-round3/params/`, outputs in `audit-round3/out/` (the first, broken
`found_here` in `out-v1/`). Everything heavy ran on the cloud box.

**Verdict.** The audit's gap to the designer is now army and garrison at about +0.15 cities @20 each; production,
raze and diplomacy have shrunk to near zero since the first audit. The two genes each fix the narrow mistake they
target in the diff, but neither moves the audit or the quarter gate by more than noise: `found_here` 2 trims the
whole-policy gap by 0.013 cities @20 (Mongols -0.07 ±0.04) and adds +0.3 cities at T100 and T150 in the all-AI
census, with the gate at -1 pp both ways; `upgrade_wait` reads +3.71 pp seat-only on the quarter gate, all of it on
one map, and nothing in the audit. Neither is ready for the full gate on this evidence.

## 1. The ranking now (his minus Hard, 10-turn windows, 32 salts, mean of four games)

Mongols (T96 save for the memory restore, then the 10-30 autosave), Swiss 10-28, Athens bombers 10-27, Sparta 10-27;
Japan skipped (drifts from T1). 61 sampled turns. "First" is the 10-30 audit on each game's own engine, the same
four games. `audit_shares.py H METRIC NAME=GLOB ...`.

| Category | Cities @20, first | now | Cities @50, first | now | VP @50, first | now |
|---|---:|---:|---:|---:|---:|---:|
| Whole policy | +0.54 | **+0.36** | +0.51 | **+0.32** | +0.29 | **+0.23** |
| Army | +0.21 | **+0.17** | +0.18 | +0.10 | +0.11 | +0.02 |
| Garrison | +0.09 | **+0.15** | +0.12 | **+0.11** | +0.08 | +0.05 |
| Settling | +0.05 | +0.07 | +0.03 | +0.03 | -0.01 | +0.02 |
| Diplomacy | +0.08 | +0.05 | +0.05 | +0.04 | -0.02 | +0.01 |
| Production and buys | +0.15 | +0.01 | +0.18 | +0.05 | +0.16 | **+0.13** |
| Research | +0.01 | +0.01 | -0.00 | +0.03 | +0.02 | +0.01 |
| Other (Raze) | +0.12 | +0.01 | +0.09 | -0.01 | +0.09 | -0.01 |

- Raze and production are gone as gaps (`raze_value`, `buy_hold`); production keeps a VP lead @50 (+0.13), nearly
  all Sparta (+0.37: his Markets, Workshops and Forges against Hard's units). The army gap shrank only a little
  (`capture_hold`); garrison grew, partly because a held capture counts as garrison.
- By game @20, whole: Mongols +0.25, Swiss +0.40, Athens +0.31, Sparta +0.48. Army and garrison are lumpy (a few
  windows: Athens T150/T160, Sparta T80/T100, Mongols T160). Settling is the most even: Mongols +0.12 and Swiss
  +0.15, positive in 6 windows each.

## 2. The patterns (staged order trace and the new settler diagnosis)

Army and garrison have no dominant pass left: the 42 kills he made that Hard didn't spread over twelve passes
(`press_strike` 7, `cities` 5, `garrison_cities` 5, ...); garrison disagreements split between `defend_cities`,
`campaign` and the routine garrison, which are the top-down layers that failed three times. Two narrow patterns:

| Pattern | Frequency (four games) | Value | Cause |
|---|---|---|---|
| **Settler walks off his founding tile** | 15 of his 39 mid-game founding turns (Mongols 7, Swiss 6, Athens 1, Sparta 1; T1 capitals left out) | settling share +0.07 cities @20 (Mongols +0.12, Swiss +0.15) | `choose_site` drops the tile for Settler hazards (an enemy army within 3, danger, trip risk; Hard then walks 2 to 10 hexes away or home) or for a foreign border that could hide a city (the tile is not even a candidate); an exposed Settler retreats |
| **Upgrade instead of attack** | 24 unit-turns at war where he attacked or marched and Hard upgraded (5 kills) | not separable in the audit (inside army/garrison) | `plan_modernization` runs in the city pass, before any attack pass; an upgrade takes all moves |

`ScriptedBrain::settler_diagnosis` and `decision_audit --settle-detail` (`settle.jsonl`) give, for every Settler the
tape founds with, Hard's order, the plan branch and the verdict on its own tile.

## 3. The genes

- **`found_here`** (`civilians.rs::found_here_value`, `plan_settler`). A Settler on a known legal site (border risk
  included) founds there when the site's value less its loss as a fresh city (the at-war army that can reach it by
  next turn against a population-1 city, both sides of the swing) is at least the value of the site the search
  picked, or when it would retreat or go home. **It must be safe this turn:** founding resolves after movement and
  combat, so a Settler an enemy can reach is captured first. The first version missed this and lost the 8.3
  Switzerland gate map 18 -> 0 of 64 (its Settler founded beside a Mongol Soldier and was taken; `out-v1/`). A found
  that failed on that tile the turn before is not repeated (`CampaignMemory::found_tried`). Level 1 needs the
  preferred quality (`settler_min_site_score`), level 2 any positive quality.
  Diff: his founding turns where Hard does not found, 15 -> 7 (level 1) -> 5 (level 2); the five left are a
  better tile next door, another Settler's target, two the enemy could reach, and a negative-score tile.
- **`upgrade_wait`** (`modernization.rs::upgrade_waits_for_attack`). A unit with an attack the military passes would
  take (`best_shot`, or `best_strike` / `counterattack_target`) is not upgraded this turn. Diff: skipped attacks
  from upgrades 11 -> 3, kills 5 -> 0.

## 4. Measured

### Decision audit (`--alt params:` and `--continue params:` the arm; paired change against the control)

| Change vs control, mean of 4 games | `found_here` 1 | `found_here` 2 | `upgrade_wait` | 1 + upgrade |
|---|---:|---:|---:|---:|
| Whole, cities @20 | -0.013 | -0.013 | -0.001 | -0.013 |
| Settling share, cities @20 (target) | -0.006 | -0.007 | ~0 | -0.005 |
| Army / garrison, cities @20 | -0.001 / +0.010 | +0.005 / +0.011 | +0.013 / -0.004 | +0.010 / +0.007 |
| Whole, cities @50 | -0.012 | -0.018 | +0.027 | +0.009 |
| Whole, VP @50 | -0.008 | -0.006 | +0.002 | -0.006 |

The settling change comes from Mongols (-0.03 ±0.03), whose whole policy drops -0.07 ±0.04 @20 and -0.08 @50;
the Swiss settling share does not move with either level. In a live window Hard's Settler takes its own route and
rarely stands on his tile; the Swiss gap is where Hard sends it (far sites, such as (21,2) from (12,13) at T121, while the near ones
are dropped for hazards or border risk), which `found_here` does not touch. No other category grows by more than 0.011.

### Quarter gate (`hard_vs_human`, 8 maps x 64 salts, paired, strength in pp, positive = stronger AI)

Control 127/512. `gate_paired.py`, per map `gate_maps.py` (`out/gate-maps.txt`).

| Arm | Seat-only | Rivals-only |
|---|---|---|
| `found_here` 1 | -0.98 [-3.12, +1.17] | -1.95 [-5.27, +1.37] |
| `found_here` 2 | -0.98 [-3.12, +1.17] | -1.17 [-4.49, +2.15] |
| `upgrade_wait` | +3.71 [+0.98, +6.45] | -0.78 [-3.32, +1.76] |
| `found_here` 1 + `upgrade_wait` | +1.56 [-1.37, +4.49] | -1.17 [-4.69, +2.34] |

The `upgrade_wait` seat gain is the 8.3 Switzerland map (18 -> 28 of 64) and 8.4 Sparta (44 -> 50); the other six
maps move by 0 to 2. By the 10-31 lesson, that is not evidence. The `found_here` rivals-only arms read weaker
rivals because of one map (8.6 Germany loss, seat 8 -> 19).

### War suite (64 runs, 33 cases), all-AI census (512 games, paired by seed), decision cost

- Suite: 38% in every arm, seat and all seats (39% for two all-seat arms).
- Census, `found_here` 2: cities at T100 +0.30 ±0.21, T150 +0.38 ±0.23, at the end +0.08 ±0.34; captures +0.27
  ±0.65, cities lost +0.40 ±0.77. `upgrade_wait`: every measure inside ±0.3 (`out/census-paired.txt`).
- Cost: 16 census games back to back on the Mac, 4 threads: 1.386 / 1.392 ms per seat-turn (control) against 1.379 /
  1.388 (`found_here` 2). The box census's +1.2 to +3.9 ms came from 24 audit jobs sharing the box.

## 5. Risks

- `found_here` founds next to armies the search avoided; a fresh city near a front may fall later (the census's
  cities lost +0.40 ±0.77 is noise either way). Hidden enemies in the fog are as for any Settler.
- A found that fails on a hidden-city tile costs one turn and is not repeated the next turn, but can recur every
  other turn if the rescout finds nothing.
- `upgrade_wait` delays upgrades; a unit that keeps finding attacks keeps its old kind (census upgrades -1 ±4).

## 6. Readiness

Neither gene is ready for the 2,048-game gate: the audit change on the target is small (the settling share is the
third category at +0.07 and the gene removes a tenth of it), and the quarter gate reads noise. If the lead wants a
gate anyway, `found_here` 2 is the one with an audit and census signal. The larger settling value is the
destination choice (`choose_site` dropping near sites), which needs its own round.

## Reproduce

```
tools/decision-audit/run.sh --engine COMMIT --out DIR --ship params/ARM.json SAVE... -- --window 10 \
  --turns 10,40,70,... --salts 32 --horizons 20,50 --alt params:audit-in/ARM.json --continue params:audit-in/ARM.json
tools/decision-audit/run.sh --local --engine here --out DIR --ship params/ARM.json SAVE... -- \
  --diff-only --settle-detail --army-detail --alt params:audit-in/ARM.json
tools/cloud/remote.sh <ref> --artifact docs/sim-results/ai-war/audit-round3/out -- \
  docs/sim-results/ai-war/audit-round3/run.sh suite|census|gate|trace ARM...
docs/sim-results/ai-war/audit-round3/{audit_shares,gate_paired,gate_maps,census_paired}.py
```
