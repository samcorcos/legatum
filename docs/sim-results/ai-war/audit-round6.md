# Decision audit, round 6: emergency production after a safe capture (2026-10-31)

Branch `astra/audit-round6`. Measurements freeze master `06d5ac789` (rounds 1–5 and the mounted rebalance),
with candidate code `f41a124797e7`. The handoff includes master `de13ef230`
(India integration plus documentation) via `9e33ce51`. The paired strength
measurements do **not** claim to measure the later India rules revision.
One candidate, **`capture_alarm` 1**, default **0**. The candidate changes no game rules or information access.
The five prior off genes are unchanged. **Do not run the full gate:** both fresh quarter gates are
inconclusive, the audit is essentially flat, and pooled seat-only strength is slightly negative. Keep `capture_alarm` off.

## 1. Ranking and method

The same four games and 61 sampled turns as round 5: Mongols and Swiss T10–170, Athens T10–160,
Sparta T10–110, every ten turns. Ten-turn policy windows, 32 paired salts, horizons 20 and 50,
equal weight per game. All three continuations are tested; the candidate gene is present in every
candidate continuation, including economic and conquest.

Current control, Hard continuation: **his policy minus Hard**, positive means his policy does better.
Category shares average the two swap directions; they need not add to the whole-policy gap.
95% intervals cluster repeated windows by salt within each game and condition on these four games.

| Category | Cities @20 | VP @50 |
|---|---:|---:|
| Whole | +0.342 [+0.316, +0.369] | +0.080 [+0.011, +0.148] |
| Army | +0.191 [+0.171, +0.210] | +0.041 [-0.003, +0.085] |
| Garrison | +0.172 [+0.153, +0.192] | +0.040 [+0.002, +0.079] |
| Diplomacy | +0.059 [+0.046, +0.071] | +0.007 [-0.023, +0.038] |
| Settling | +0.046 [+0.035, +0.057] | -0.027 [-0.053, -0.000] |
| Production | +0.018 [+0.002, +0.033] | +0.040 [-0.004, +0.085] |
| Other | +0.003 [-0.005, +0.011] | +0.026 [+0.007, +0.046] |
| Research | +0.002 [-0.007, +0.010] | +0.037 [+0.011, +0.064] |

**Replay correction required by the mounted rebalance.** Direct replay on current master first
diverges at Mongols T47, Swiss T72, Athens T50 and Sparta T116. Continuing those replays would
compare against positions the designer never played. `audit_states` exports the original positions
and checks every recorded hash; `decision_audit --states` independently checks the imported states
before using them for shadow observations and branch starts. Actual branches resolve on current rules.
Exports reproduced **172/172 Mongols, 180/180 Swiss, 177/177 Athens and 125/125 Sparta** turns with zero
hash mismatches. Export engines: `cabdbfd5f` (Mongols, same historical rules), `060f78ef4` (Swiss),
`a58e80856` (Athens and Sparta). Current-engine one-turn resolutions can differ from the original
hashes; that counter is retained separately from the zero source-state mismatch count.

Mongols T96 restores the saved AI memory. Current Hard shadows all original positions, as before;
its decisions need not reproduce the older AI. No hidden state enters a live decision.

## 2. Patterns

`--prod-detail` now covers **every own city**, including the passes that set production before
routine scoring. It records incoming production and local defense evidence. Missing scorer rows
are no longer assumed to be early overrides.

| Economy building for him, unit for Hard | Mongols | Swiss | Athens | Sparta | Total |
|---|---:|---:|---:|---:|---:|
| Emergency production | 59 | 116 | 81 | 110 | **366** |
| Threat defense | 67 | 0 | 1 | 9 | **77** |
| Routine production | 18 | 19 | 17 | 11 | **65** |
| Carryover, no new production order | 0 | 0 | 1 | 0 | **1** |
| Total | 144 | 135 | 100 | 130 | **509** |

**443/509 (87.0%)** come from those two early passes. Neither door production nor modernization
explains this set. The override count alone is not a count of errors. A conservative screen finds **57**
emergency overrides (5 Mongols, 18 Swiss, 7 Athens, 27 Sparta) with zero local threat and early-warning power, no
forecast city loss, no visible enemy within six hexes, and a nearby resisting capture, with no visible
enemy within six hexes of any nearby damaged capture. Examples: Sparta's Matsue/Matsuyama T57–61 (Markets/Laboratory to units),
Swiss Geneva/Fribourg T158 (Machine Shops to Riflemen), Athens Phaleron/Laurion T156–158
(Autoworks/Machine Shop to Infantry). The candidate diff, below, tests causation rather than assuming
every screened row changes. With the gene, all 57 screened cases disappear.
Across all positions, economy-versus-unit disagreements fall **509 → 447**, and emergency-pass
disagreements **366 → 304**. Threat (77), routine (65), and carryover (1) totals stay the same.
There are 83 changed production city-turns overall; 63 were in the original economy-versus-unit set,
53 now match his exact building, 9 choose another non-unit item, and 1 still chooses a unit.
A mixed-alarm Athens T167 row is deliberately excluded from the screen: another captured city has
a real nearby enemy, so the candidate correctly keeps its emergency response.

**Cause:** `plan_emergency_production` treats any other own city within eight hexes, at half HP or
less, as an exposed capture if it is **in resistance OR has a visible enemy military unit within six**.
A safe conquest therefore interrupts neighboring economy builds just like a real counterattack.

### Army and garrison windows

Current cities-at-20 shares for the named windows (Hard continuation, 95% intervals):

| Window | Army | Garrison |
|---|---:|---:|
| Sparta T80 | +0.328 [+0.224, +0.432] | +1.750 [+1.611, +1.889] |
| Sparta T100 | +0.078 [-0.081, +0.237] | +1.688 [+1.507, +1.868] |
| Athens T150 | +0.953 [+0.811, +1.095] | +0.953 [+0.765, +1.142] |
| Athens T160 | +0.812 [+0.632, +0.993] | +1.156 [+0.930, +1.383] |
| Swiss T150 | +0.781 [+0.477, +1.086] | +1.188 [+0.964, +1.411] |
| Mongols T160 | +2.266 [+1.974, +2.557] | +1.766 [+1.405, +2.126] |

Non-Scout unit-turns with different orders, covering each named ten-turn window. In the audit,
"garrison" means at or adjacent to an own city, not a particular planning pass. Upgrade orders
belong to production even though these unit traces attribute them to modernization:

| Window | Disagreements | Unit-turns with more attacks for him | Largest passes |
|---|---:|---:|---|
| Sparta T80 | 15 | 2 | defend_cities 5; commit_idle_defenders 3; withdrawal 2 |
| Sparta T100 | 25 | 5 | modernization 6; press_strike 5; garrison_cities#2 5 |
| Athens T150 | 26 | 0 | modernization 13; garrison_cities 4 |
| Athens T160 | 27 | 2 | modernization 10; defend_cities 4; hold_captures 3 |
| Swiss T150 | 24 | 3 | garrison_cities#2 9; several other passes |
| Mongols T160 | 59 | 5 | defend_cities 11; modernization 10; press_strike 9 |

Of 176 disagreements, 39 are modernization, the already-tested `upgrade_wait` lead. The 17 unit-turns
with more attacks for him spread across withdrawals, defensive holds, city-fire retreats, upgrades,
and other passes. These are order disagreements, not 176 demonstrated mistakes: some are Skip versus
no explicit order with the same result. Garrison unit-turns with different resolved destinations are
4/4/6/7/8/17 in the table's order. Concrete costly-looking differences also disagree on the cause:
Sparta T84 withdraws two Troopers instead of attacking; T100 upgrades the two attackers;
Athens T153–154 upgrades a departing Rifleman; Swiss T151 keeps a Field Cannon back and redirects its
melee escort, then T159 retreats both cannons instead of firing; Mongols T164–167 splits between city
defense, a cavalry recovery, door defense and a different retake target. Modernization is an old rejected
lead, and the other holds depend on different local forecasts. No new shared narrow threshold error
was established, so no new offense layer or army gene.

The defense evidence separates those cases. Sparta's Berlin T82 has threat 20/local defense 91,
Essen T87 45/85, and Athens' Bukhara T162 66/181; those defense totals include the units being held,
so the apparent surplus does not prove that leaving is safe. `defend_cities` uses its existing 200%
margin. Swiss Geneva T151 instead has zero local threat, defense 163, and no visible hostile military
unit: its Field Cannon is held by `garrison_cities#2`, whose default still staffs cities even without
current threat. That is the existing standing-garrison policy (`war_garrison`/`hollow_march` are off),
not the same false production alarm. This round does not revive those broad release-of-garrison layers.

### Mounted units

The current AI leaves pre-Modern mounted units in production on **206 city-turns**, against his **143**
(out of 4,291 city-turns). In the recorded armies, Mongols has 347 mounted unit-turns and Sparta 13;
Swiss and Athens have none. On those same positions the one-turn attack totals are **14 for him and
20 for Hard** (Mongols 10/16, Sparta 4/4), with eight kills and one mounted loss each in those one-turn
comparisons. An apparent missed cavalry kill at Mongols T165 targets a visible 13-HP Soldier, not an
exposed archer: Hard takes `recover_city_fire`, while he charges. That is a general retreat-versus-kill
trade-off, not a newly isolated mounted counter error. These older tapes cannot establish how he would use the newly
buffed horses. The existing counter-composition and Keshig exposed-archer tests exercise horse selection
and charging; no clear new mounted valuation error was isolated here. Production counts alone are
not evidence of better raids.

## 3. Gene

**`capture_alarm` 1**, in `production.rs::plan_emergency_production`: plain Hard no longer treats
resistance alone as a neighboring capture alarm. The existing visible-enemy condition remains.
Local threat deficits, ranged relief, lone-city defense, other production passes, and all non-Hard
behavior are unchanged. Default 0 retains the old path.

## 4. Measured

### Continuation audit

93,696 branches per arm (61 windows × 32 salts × three continuations × 16 whole/hybrid arms),
187,392 total. The candidate gene is present in all three continuation parameter files.
Below is **candidate gap minus control gap**: negative closes his advantage, positive widens it.
Paired 95% salt-cluster intervals; equal game weights. Full control/candidate shares and per-game
values are in `out/audit-summary.txt`.

| Gap change | Hard | Economic | Conquest |
|---|---:|---:|---:|
| Whole, cities @20 | -0.001 [-0.003, +0.002] | -0.004 [-0.008, +0.000] | -0.004 [-0.010, +0.003] |
| Production, cities @20 | +0.000 [-0.001, +0.001] | -0.002 [-0.004, +0.001] | -0.001 [-0.004, +0.002] |
| Whole, cities @50 | -0.008 [-0.039, +0.024] | -0.009 [-0.040, +0.022] | -0.001 [-0.036, +0.034] |
| Production, cities @50 | -0.004 [-0.028, +0.019] | +0.002 [-0.017, +0.021] | -0.008 [-0.032, +0.015] |
| Whole, vp @50 | +0.003 [-0.027, +0.033] | +0.013 [-0.022, +0.048] | -0.008 [-0.037, +0.021] |
| Production, vp @50 | -0.006 [-0.023, +0.011] | -0.003 [-0.022, +0.017] | -0.005 [-0.025, +0.015] |

The target production share in cities @20 does not improve under Hard, and the whole-policy change
is only −0.001 cities [−0.003, +0.002]. There is no robust whole-policy city or VP gain across the three
continuations. The 57 corrected alarms are a real decision-pattern correction, not a demonstrated
strength improvement.

### War suite

64 paired runs in each of 33 scenarios: **39%** mean goal success and **13/33** at human parity
for control, candidate seat-only, and candidate all-seats. Every scenario has exactly the same
success outcomes: **+0.0 pp [0.0, 0.0]** observed paired difference. This is no detected regression,
not evidence of a war benefit or a population-level zero effect. See `out/suite-seat.txt` and
`out/suite-all.txt`.

### Decision cost

16 paired all-AI census games per arm, four threads, interleaved control/candidate twice in one box job:
**2.963 / 2.751 ms** per seat-turn, then **2.751 / 2.302 ms**. No observed overhead. These wall-clock
numbers reflect a changing shared-box load and different resulting game states; they do not establish
a speed improvement. The live change adds a boolean guard, not a search (`out/cost.txt`).

### Quarter gates

Two new seed sets, **300001 and 301001**, each 64 salts × eight maps = 512 games per arm.
`w04_gate.py --family 4`: four planned comparisons (two roles × two seed sets), role-normalised strength
positive means stronger AI. For rivals-only this flips the raw human-seat win change. Intervals below
are percentage points; 95% descriptive first, family-adjusted second. All four decisions are **inconclusive**.

| Seeds | Role | Control seat wins | Candidate seat wins | AI strength | 95% interval | Adjusted interval |
|---|---|---:|---:|---:|---:|---:|
| 300001 | Seat-only | 122/512 | 119/512 | −0.59 | [−2.93, +1.76] | [−3.57, +2.40] |
| 300001 | Rivals-only | 122/512 | 122/512 | 0.00 | [−3.02, +3.02] | [−3.85, +3.85] |
| 301001 | Seat-only | 109/512 | 107/512 | −0.39 | [−2.64, +1.86] | [−3.26, +2.47] |
| 301001 | Rivals-only | 109/512 | 98/512 | +2.15 | [−0.74, +5.04] | [−1.53, +5.83] |

Pooled, descriptive map-stratified bootstrap over both batches (1,024 games per arm): **seat-only −0.49
[−2.15, +1.07]**, **rivals-only +1.07 [−1.07, +3.12]**. The latter's raw seat-win delta is −1.07 pp.
The control wins 231/1,024 (22.56%); the equal-seat reference is 28.1%, not 12.5%, and is not a prediction
for these asymmetric maps. These samples do not establish that the designer would lose more often.

Batches `r6-quarter1-1031-130342` and `r6-quarter2-1031-130834`; both commit `f41a124797e7`, binary
`7df6d90ab4203b02`. Gate summaries are in `out/quarter1/`, `out/quarter2/`, and `out/gate-pool.txt`.

## 5. Risks

A hidden counterattack can reach a captured city after the visible enemies disappear. This candidate
removes only the neighboring resistance shortcut; the captured city's own defense, observed local
threat and other warning systems still operate. A lower army bill may help economy without converting
into cities or VP, and extra economic strength among AI rivals need not make them harder for the designer.

Audit intervals condition on four recorded winning games and fixed sampled windows. They do not
measure the designer's future loss rate. The continuation legality counter is not zero: control has
2,192 invalid proposed orders across 1,929 of 93,696 branches; candidate 2,160 across 1,900 branches.
Even pure alternative-policy branches have 68/78 such proposals (control/candidate). These are
rejected by the engine. Recorded orders can also become invalid once a branch diverges,
and stronger mounted units change the consequences of his historical orders. Gate confirmation remains necessary.

## 6. Readiness and reproduction

**Do not run the full 2,048-game gate.** Neither the audit nor the quarter gates establish a strength
improvement; the pooled seat-only estimate is negative. A corrected production pattern alone is not
enough to recommend more gate spend. `capture_alarm` stays at 0. The later India merge also means
these frozen-revision strength estimates are not a fresh measurement of the handoff revision.

**Checks:** extended box verify passed on merged code `95d997251` (job
`20261031T170424Z-204731`, 913 seconds inside the verifier). It ran `cargo fmt --all --check`,
clippy `--workspace --all-targets --release -- -D warnings`, and the full workspace suite under
`release-checked` (overflow checks and debug assertions enabled): **2,093 passed, 11 ignored**, plus
workspace doctests. `tools/sim/repin.sh` passed, including all 12 determinism tests: **no pins diff,
no moved pins** with the gene off. Branch and master smoke each completed 50 games with zero panics
and zero stalls; their invalid-order count was unreported. Cargo work used the build-slot wrapper.
The earlier scoped regression tests and trace/order-equivalence test also passed. Subsequent changes
are analysis scripts, the runner's explicit-continuation requirement (`bash -n` passed), and reporting.

After the extended verify, documentation-only updates `a902f1269` and `de13ef230` were merged
(at `07d7d2b80` and `9e33ce51`). Their Rust trees are identical to `95d997251`. The corresponding
pin/determinism-only rechecks, jobs `20261031T175524Z-8fc6ce` and `20261031T180320Z-df21dd`, both
passed both pin assertions and all 12 determinism tests. All **12 returned pin/fixture files are
byte-identical** in each readback. No full suite was repeated.

**AWS:** about **$1.32 compute**, conservatively charging this task the entire shared-box interval
16:18:25–18:07:51 UTC at the highest observed rate ($0.6923/hour plus $0.005 IPv4), plus the two quarter
batches ($0.03 and $0.02, 0.07 server-hours total). A further $0.15 artifact-transfer/storage allowance
puts the task allocation at about **$1.50, below $3**; this is an estimate, not an invoice. Both quarter
instances are confirmed terminated. No infrastructure configuration was changed.

Runner: `audit-round6/run.sh`; parameters and the three candidate continuations: `audit-round6/params/`;
role-separated gate arms: `audit-round6/arms.json`; summaries: `audit-round6/out/`.
Raw tapes, states and runs are kept under `~/LegatumRuns/audit-round6/`, not committed.


### Reproduce

Input hashes and historical export revisions: `audit-round6/inputs.json`. Build each tape with
`tools/decision-audit/build_tape.py`; run `audit_states TAPE OUT.jsonl` on its listed historical engine,
then gzip the result into `audit-round6-input/GAME.jsonl.gz`. The tape and optional Mongols T96 memory
belong in `audit-round6-input/GAME/`. Do not regenerate recorded positions on the current engine.

```sh
# On the frozen candidate code; one game per box job (GAME = mongols, swiss, athens, sparta).
tools/cloud/remote.sh . --with-uncommitted --env RAYON_NUM_THREADS=8 \
  --artifact audit-round6-out --artifact-dir RESULTS -- \
  docs/sim-results/ai-war/audit-round6/run.sh audit ctl GAME \
  hard,params:tools/decision-audit/continuations/economic.json,params:tools/decision-audit/continuations/conquest.json
tools/cloud/remote.sh . --with-uncommitted --env RAYON_NUM_THREADS=8 \
  --artifact audit-round6-out --artifact-dir RESULTS -- \
  docs/sim-results/ai-war/audit-round6/run.sh audit alarm GAME \
  params:docs/sim-results/ai-war/audit-round6/params/alarm.json,params:docs/sim-results/ai-war/audit-round6/params/alarm-economic.json,params:docs/sim-results/ai-war/audit-round6/params/alarm-conquest.json
# Collect each arm/GAME into RESULTS, then summarize; raw rows stay outside git.
python3 docs/sim-results/ai-war/audit-round6/analyze.py RESULTS alarm
# Suite and cost are separate short box jobs:
tools/cloud/remote.sh . --artifact audit-round6-out -- \
  docs/sim-results/ai-war/audit-round6/run.sh suite ctl alarm
tools/cloud/remote.sh . --artifact audit-round6-out -- \
  docs/sim-results/ai-war/audit-round6/run.sh cost ctl alarm
# Each quarter gate: 8 maps x 64 salts x 3 arms, one server, 15-minute hard cap.
# Build a clean committed revision; the gate verifies commit and binary provenance.
LEGATUM_CLOUD_EXAMPLES=hard_vs_human tools/cloud/build-linux.sh
tools/cloud/run-batch.sh --count 1 --max-hours 0.25 --yes --bin hard_vs_human \
  --ai-params docs/sim-results/ai-war/audit-round6/arms.json --env LEGATUM_HVH_RESEED_DICE=1 -- \
  --arms --games 64 --seed 300001 --label r6-quarter1
# Repeat with seed 301001, label r6-quarter2.
python3 tools/ai-quality/w04_gate.py SUMMARY BATCH --family 4
python3 docs/sim-results/ai-war/audit-round4/gate_pool.py alarm_seat BATCH1 BATCH2
python3 docs/sim-results/ai-war/audit-round4/gate_pool.py alarm_opp BATCH1 BATCH2
```
