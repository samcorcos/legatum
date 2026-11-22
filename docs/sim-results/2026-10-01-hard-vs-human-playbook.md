# Scripted Hard against the human playbook — 2026-10-01

**Selected: candidate 26. The 22% v6 target was missed.** The fresh rebased
run gives **104/400 (26.00%)**, down from **125/400 (31.25%)**: 21 fewer proxy
wins, or 5.25 percentage points. The target permits at most 88 wins, so this
misses by 16 wins / 4 percentage points. Candidate 47 finishes all 400 seeds at
**110/400 (27.50%)**, so the later ranged-strength work is excluded.

## Control and measurement

The original experimental parent is `0368ff0`. The retained v6/v5 benchmark
source manifest matches every Rust/Cargo source file from that starting checkout.
This continuation starts from master `a4134ad` and cherry-picks `812b7e2`, then
restores the selected candidate from its retained source snapshot. Master
changes since `0368ff0` add opt-in balance levers; those remain disabled here.
The unchanged parent executable was built separately and frozen. The v6 control
is therefore the existing **125/400 (31.25%)** measurement, and the matched v5
control is **60/400 (15.00%)**, not the older 17.25% historical result.

The proxy schedule is seeds 3600001–3600400, rotating seat/faction, eight
independent players, Standard speed, Peace, barbarians, default city fire, and
seven scripted Hard opponents. There is no game turn limit; the 1,200-turn
safety cap counts as a non-win. The policy never reads the proxy seat, seed
schedule, observer telemetry, or hidden state. No rules experiment is enabled.

The independent Hard-only control runs 200 seeds 3200001–3200200 with eight
scripted Hard seats. It finishes all 200 games by VP: mean 244.13 turns, no
stalls, and faction best/worst win-rate ratio 42/16 = 2.625. The requested
limits are mean 219.717–268.543, no stalls, and ratio at most 2.825.

The fresh selected-candidate Hard-only run exits 0: all 200 games finish by VP,
with no panics. All per-seed outcomes, turns and winners match the original
candidate-26 guard run. Hard-only reports do not retain per-turn hashes.

| Acceptance measure | Control | Selected candidate 26 | Required limit | Result |
|---|---:|---:|---:|---|
| v6 wins / 400 | 125 (31.25%) | 104 (26.00%) | At most 88 (22%) | **Missed** |
| Hard-only mean turns / 200 | 244.130 | 262.280 (+7.43%) | 219.717–268.543 | Pass |
| Hard-only stalls | 0 | 0 | No increase | Pass |
| Hard-only faction best/worst ratio | 2.625 | 37/15 = 2.467 | At most 2.825 | Pass |

The policy is retained for branch review under the explicit instruction to
commit the best guarded improvement even if it misses 22%. It is not recorded
as satisfying the original combined acceptance target.

## Diagnosis and measured impact

The observer replays **all 400 baseline tapes / 96,007 turns**, checking the
initial and every resulting turn hash. City-loss conditions use the defender's
actual `PlayerView` before resolution. A warning requires at least two visible
proxy combat units within five tiles on consecutive turns. These are overlapping
conditions and game cohorts, not additive estimates of wins recoverable by a fix.
The ranking uses the number of proxy-winning games containing each condition.
It is descriptive exposure, not an isolated causal effect: several conditions
usually coexist in the same conquest. Only paired policy runs measure changes
in wins, and those measure the combined implementation.

| Rank | Failure or vulnerability | Measurement in the 400-game v6 control |
|---|---|---|
| 1 | Thin city defence despite advance warning | 545/580 captures follow at least three consecutive visible-warning turns, across 260 games including 117/125 proxy wins. 554 captures begin without a garrison; 463 have at most one non-Scout defender within two tiles, across 237 games / 115 proxy wins. |
| 2 | Failure to recover conquered cities | Of 561 captures with ten subsequent turns available, 513 are not retaken by any AI in that window, across 250 games / 115 proxy wins. Actual recapture attacks occur on 268 city/player/turn observations; 207 do not recover the city that turn, across 114 games / 49 proxy wins. These counts do not establish that each city was recoverable. |
| 3 | Isolated cities | 195 captures are more than six tiles from the defender's next city, or are its last city, affecting 159 games / 85 proxy wins. |
| 4 | Late or abandoned containment | After at least two distinct captures and five currently held cities, v6 has 15,951 observed turns; on 9,512 (59.6%) it is not the public coalition target. This occurs in 156 games / 73 proxy wins. Absence does not establish that a legal, viable coalition attack existed. Separately, 760 peace agreements while the proxy remains coalition target have no local army or badly damaged home, across 95 games / 76 proxy wins. |
| 5 | Exposed settlers | 336/391 stolen settlers have no friendly military within one tile, across 205 games / 71 proxy wins. In 378 captures the victim sees a proxy military unit within movement radius plus one before resolution. |
| 6 | Slow response to a new war | 112 city captures occur within five turns of the current war, across 94 games / 56 proxy wins. This heavily overlaps with advance warning and thin defence. |
| 7 | Gold not spent at capture | Only one lost city has an affordable, unblocked military purchase quote and no purchase ordered that turn; that game is not a proxy win. The existing hoarding detector reports zero incidents. Evidence favors earlier allocation of gold and production, not widespread idle treasuries at capture. |

Examples: seed 3600001, city 3 falls at T130 with no local defenders after nine
consecutive warning turns, only three turns into the current war. Seed 3600006,
city 9 falls at T124 with no local defenders after eight warning turns. Neither
has an affordable defender at that instant. Waiting until a one-turn capture
forecast is therefore too late to establish a useful army.

All 391 settler captures occur while the defender's pre-resolution view already
shows war. The pre-war routing gap is real in source and covered by a focused
test, but these observations do not establish it as the main cause of those
captures. Similarly, a city missing its garrison on the final turn may have
lost that unit earlier; the audit does not label all 554 as voluntary abandonment.

The extended audit also finds 12,856 unit-turn opportunities for a defender
within three tiles of a warned city to shoot a visible proxy military unit.
On 2,660 it issues no ranged attack; 2,516 of these involve a shooter below
60 HP. Those wounded non-shots occur in 345 games, including 108 proxy wins.
This does not classify every retreat as a mistake. Broad changes to recovery
failed the Hard-only guards. Later experiments limited combined counterfire to
concentrated sieges and immediate capture threats; their results appear below.
Those experiments are excluded from the selected candidate 26.


Coalition persistence is also weak: 1,165 peace agreements occur while the proxy
is the public coalition target both before and after resolution. On 760, the
other AI sees no proxy military within five tiles of home and has no city at
100 HP or less. Those agreements occur in 95 games, including 76 proxy wins.
This measures an apparent mismatch between the public objective and the peace
choice, not the counterfactual benefit of continuing each war.

## Completed resumed v6 measurement

Both fresh 400-game runs exit 0 with no panics. Candidate 26 matches the original
candidate-26 result for every seed, outcome, winner, turn count and all 105,100
per-turn hashes. Candidate 47 matches all 353 previously completed games on
winner, outcome, turn count and final seat records. Its fresh full report has
397 VP finishes and three capped games, at mean 259.625 turns.

The selected candidate's paired comparison has 60 proxy wins in both runs,
65 control wins that cease to be wins, 44 new wins and 231 non-wins in both.
Its descriptive 95% Wilson interval is 21.94%–30.51%. These seeds were reused
while tuning, so this is a result on the specified schedule, not a fresh holdout
or a demonstrated human win rate.

Candidate 26 also has 397 VP finishes and three 1,200-turn caps: seeds
`3600090`, `3600141`, and `3600176`. The first two were proxy wins in the control.
The cap counts as a non-win by the predefined schedule; uncapped outcomes are
unknown. The control has one capped proxy game. The no-added-stalls acceptance
guard applies to the separate 200-game Hard-only cohort, not this mixed cohort.

The reduction in wins is not a uniform tactical improvement: proxy city
captures increase from 580 to 592 and stolen settlers from 391 to 434. The
combined policy changes the wider contest; these measurements do not isolate
which individual heuristic caused the improvement.

## Selected implementation

Candidate 26 ties candidate 4 for the lowest completed v6 win rate among
policies that pass all Hard-only guards. It has a shorter Hard-only mean
(262.280 versus 265.675 turns) and fewer capped proxy games (three versus four).
Candidate 47 passes the guards but gives v6 six more wins, 110 versus 104.
The selected policy therefore keeps the smaller candidate-26 implementation.

- Hard assesses visible, reachable military within five tiles. At Peace, two
  combat units from one neighbour and a legal ordinary declaration are required
  before treating the group as a city threat. Scouts and barbarians do not
  trigger this additional mobilization.
- Settlers avoid the visible reach of neighbours that can legally declare war.
  A local escort follows the settler's next step, with legal movement and
  occupancy checks, instead of moving toward its distant founding destination.
  An occupied rendezvous cannot cancel the civilian's safe route indefinitely.
- A surviving melee garrison stays when multiple enemies are nearby and an
  enemy capture unit can reach the city this turn. Other troops still act.
- Immediate rescue purchases can serve several cities, once per city. Earlier
  readiness purchases fund local defence toward 125% of visible threat, using
  actual engine quotes, one shared treasury and an upkeep reserve. A winning
  Swiss cash goal is protected from optional readiness spending.
- Public city counts, a recent declared war and known nearby cities identify
  an expanding aggressor before the global VP warning. Permanent-team size is
  normalized. Ordinary treaty and declaration legality still apply.
- A discretionary siege requires two healthy, reachable ranged units after
  reserving home guards. Existing wars, containment and the separately planned
  Sparta expedition retain their own urgency checks.
- Hard does not leave a continuing coalition merely because the rival's public
  total army is larger. A threatened home can still justify peace; local visible
  enemy pressure and city health determine that exception.
- Hard no longer orders a ranged unit to occupy a zero-HP enemy city. The core
  forbids ranged city capture; those units remain available for useful orders.

The later experimental coordinated volleys, infantry follow-ups, phase-adjusted
reinforcement forecasts, Scout-sight change, blocked-production reassignment,
and ranged-strength quotas are removed from this candidate. Their historical
measurements remain below; they are not part of the selected policy. The
archived observer and stall-inspection examples remain as diagnostic tools.

The Hard-only control records 3,387 war declarations and 1,794 declared wars
with a subsequent battle before the game ends (53.0%); it captures only 37 cities.
These are declaration/participant counts, not unique war-to-capture conversion
rates. In the rejected broad-counterfire policy, declarations rise to 4,793,
with 2,610 fought wars and 58 captures, while mean duration rises to 269.225.
This motivated testing war readiness and coalition persistence directly.

A retained failing-game trace for candidate 26 (seed 3600006) shows the
remaining deployment problem: city 1 falls at T169 after 20 visible-warning
turns, while six friendly Rangers and one Soldier survive elsewhere. Ranger
120 alternates approach and retreat; the army does not establish a firing line
before the city falls. This trace motivated the later reinforcement and counterfire experiments;
the full benchmarks did not establish an improvement over candidate 26.

The shared context constructor and all versioned proxy policies are unchanged.
The change uses only `PlayerView`, ordered iteration, integer calculations, and
explicit tie breaks. It adds no persistent brain state, economic advantage,
vision, combat bonus, or faction-specific handicap.

## Development and rejected policies

The separate 64-game development schedule (3700001–3700064) is reused while
iterating. Its source-matched control gives v6 17 wins. It is a screen, not an
independent holdout: the first candidate scored 11/64 there but 114/400 on the
requested schedule. Full acceptance seeds were subsequently reused during
tuning too. All final claims must therefore be limited to the specified seeds;
another seed range and a human playtest would be separate evidence.

The most informative development measurements, including explicitly marked
partial runs, are:

| Policy | v6 / 400 | Hard-only mean | Hard stalls | Faction ratio | Decision |
|---|---:|---:|---:|---:|---|
| Frozen master | 125 (31.25%) | 244.130 | 0 | 2.625 | Control |
| Initial readiness / guard / purchase policy | 114 (28.50%) | 258.390 | 1 | 2.923 | Reject: stalls and faction ratio |
| Broader standing-army quota and escorts | Not run | 271.615 | 0 | 1.882 | Reject: duration +11.26% |
| Candidate 4: focused guards, escorts, early containment | 104 (26.00%) | 265.675 | 0 | 2.333 | Hard-only guards pass; proxy target fails |
| Candidate 26: optional war readiness and local peace risk | 104 (26.00%) | 262.280 | 0 | 2.467 | Hard-only guards pass; proxy target fails |
| Candidate 27: earlier victim-relative containment | Interrupted after 93 wins / 325 completed | 266.950 | 1 | 2.125 | Reject: proxy target and stall |
| Candidate 28: remove optional advance purchases | Interrupted after 90 wins / 350 completed | 262.975 | 0 | 2.250 | Proxy target fails |
| Candidate 29: compact settlement and spawn clearance | Interrupted after 93 wins / 327 completed | 257.180 | 0 | 2.438 | Proxy target fails |
| Candidate 30: relief after ranged fire | Interrupted after 101 wins / 392 completed | 257.215 | 0 | 2.438 | Proxy target fails |
| Candidate 31: rear-city siege reinforcement | Interrupted after 19 wins / 89 completed | 272.795 | 2 | 2.467 | Reject: duration and stalls |
| Candidate 32: pinned-settler rescue, no extra production | Interrupted after 27 wins / 98 completed | 263.855 | 1 | 1.850 | Reject: stall |
| Candidate 33: account for existing field reinforcements | Interrupted after 51 wins / 195 completed | 262.430 | 1 | 1.900 | Reject: stall |
| Candidate 35: matched counterfire, progressing escorts | 108 (27.00%) | 254.395 | 0 | 2.786 | Proxy target fails |
| Candidate 36: persistent containment and matching campaign target | Not run | 253.815 | 0 | 3.154 | Reject: faction ratio |
| Candidate 37: neutral siege-battery preparation | Interrupted after 51 wins / 195 completed | 259.270 | 0 | 3.071 | Reject: faction ratio |
| Candidate 38: isolated coordinated defensive volleys | Interrupted after 52 wins / 226 completed | 271.195 | 0 | 3.167 | Reject: duration and faction ratio |
| Candidate 39: counterfire limited to concentrated sieges | 108 (27.00%) | 263.025 | 0 | 2.375 | Proxy target fails |
| Candidate 40: guaranteed infantry follow-up | Interrupted after 93 wins / 339 completed | 260.440 | 0 | 2.571 | Proxy target fails |
| Candidate 41: ranged relief after the fire phase | Interrupted after 95 wins / 330 completed | 260.800 | 0 | 2.571 | Proxy target fails |
| Candidate 42: isolated arrival forecast | Not run | 265.970 | 1 | 2.467 | Reject: stall |
| Candidate 43: earlier counterfire deployment | Not run | 269.290 | 1 | 2.313 | Reject: duration and stall |
| Candidate 45: combined counterfire and corrected Scout sight | Interrupted after 61 wins / 236 completed | 265.420 | 1 | 2.400 | Reject: stall |
| Candidate 46: reuse persistently spawn-blocked production | 109 (27.25%) | 260.615 | 0 | 2.400 | Hard-only guards pass; proxy target fails |
| Candidate 47: match local ranged strength | 110 (27.50%) | 258.160 | 0 | 1.550 | Full resumed run; guards pass, worse than 26 |
| Candidate 48: match legally hostile peacetime composition | Interrupted; no final result | 259.005 | 0 | 1.722 | Hard-only guards pass |
| City covering fire with two-unit peace trigger | Stopped after Hard-only failure | 269.920 | 0 | 3.231 | Reject: duration +10.56%, faction ratio |
| Earlier containment and rear-garrison release | Interrupted after 96 wins / 325 completed | 266.435 | 0 | 1.889 | Cannot finish with at most 88 wins |
| Completed opening army before fourth city | Not run | 285.395 | 1 | 2.923 | Reject: duration +16.90%, stalls and faction ratio |
| Distributed reinforcement production | Not run | 295.890 | 1 | 2.692 | Reject: duration +21.20%, one stall |
| Expanded ranged relief and campaign coordination | Interrupted after 101 wins / 346 completed | Not run | — | — | Cannot finish with at most 88 wins |

The WIP label lagged the source: `812b7e2` also contained subsequent
composition/reinforcement work from candidates 48/49. Candidate 47 was rebuilt
from its exact retained source snapshot on `a4134ad`, with a hash-verified patch
retained in the evidence bundle. The uncompleted later variants are removed;
this continuation introduces no new policy candidates.

Interrupted runs have exit 130 and retained archive metadata; their fractions
are not reported as completed 400-game results. Parallel completion order means
they are not necessarily contiguous seed prefixes. The resumed candidate-47
run matches every shared completed seed checked against the old interrupted
run; its full 400-seed result above replaces the unfinished measurement. The measured improvements are
for combined policies, not causal estimates for individual heuristics. In the
completed candidate-4 comparison, 65 seeds are proxy wins under both policies,
60 cease to be wins, 39 become wins, and 236 are wins under neither.

A recorded reproduction of stalled seed 3200077 matches every original
outcome/telemetry field (apart from run index and timing) and replays all 1,200
hashes. Inspecting the saved PlayerView exposed an escort rendezvous bug: a
settler's safe path was replaced with Hold while the chosen escort tried to
enter an occupied friendly city. The city garrison never left. The fix checks
military occupancy before assigning that rendezvous. The new regression covers
this case. An earlier hypothesis about reciprocal peacetime mobilization was
not supported by the saved planner view (all six city threats were zero), and
excluding peaceful home guards did not eliminate the stalled seed. A second
recorded stall, seed 3200012 under candidate 27, also matches its original
outcome/telemetry and all 1,200 replay hashes. Multiple completed unit queues
are blocked by friendly units on every eligible spawn tile; the view explicitly
reports the spawn blocker, with stored production and available resources.
A tested clearance policy for idle field troops did not meet the proxy target.
Candidate 46 instead redirects persistent surplus production to infrastructure.
The reinforcement variant (candidate 31) still stalls on seeds 3200012 and
3200067. Re-recording 3200012 checks every one of its 1,200 turn hashes;
several empires finish with 42–63 military units and blocked queues. The
198 non-stalled games average 263.429 turns, so its failed duration guard is
largely explained by those two capped games. This motivates counting available
field troops in the reinforcement budget; it does not by itself prove that
reinforcement production caused either stall.

A later candidate-45 reproduction of seed 3200081 matches all original result
fields except index/timing and replays all 1,200 hashes. At the cap, seven of
eight empires have 23 blocked military queues in total (one is a Scout), while
295 military units (including seven Scouts) already exist. Some queues hold over 20,000 earned
production. The current queue-reallocation rule is covered by an actual-engine
regression: it finishes infrastructure without granting production or deleting
units. Candidate 46 finishes all 200 Hard-only seeds, including this seed.
This is a cohort result, not proof that every possible stall is eliminated.

The retained per-candidate screens also include rejected early upgrades,
replacement garrisons, a larger containment army, defended-border declarations,
and settler shelter. These small screens did not reliably predict full-run
acceptance. In particular, responding to every front by diverting additional
cities into troop construction substantially damages normal Hard-only pacing.

## Why 22% was missed; remaining priorities

The selected policy improves readiness and diplomatic commitment, but retains
the existing movement, recapture and counterfire decisions. The later local
ranged-strength candidate does not improve the full v6 result, and broader troop
or recovery changes repeatedly violate Hard-only pacing, stall or faction guards.
The measured target therefore remains unmet; the surviving failure modes are
not grounds to claim an unmeasured 22% policy.

The final observer independently replays all 400 selected-policy archives,
checking every initial hash and all 105,100 resulting turn hashes. This ranking
uses the number of proxy-winning games exposed to each condition. These cohorts
overlap; they are not additive estimates of recoverable wins or isolated causal
explanations of the 16-win target miss.

| Priority | Remaining gap | Selected candidate evidence | Next question to test |
|---|---|---|---|
| 1 | Recapture and consolidation after a city falls | 486/577 captures with ten subsequent turns are not retaken by any AI in that window, across 245 games including 93/104 proxy wins. On 302/424 city/player/turn observations with a recapture attack, the city is not recovered that turn. | Can a reachable group coordinate a viable recapture and hold the city, without pulling needed home guards away? These observations do not prove that every city was recoverable or every attritional attack was mistaken. |
| 2 | Thin local defence after advance warning | 390 captures combine at least three consecutive warning turns with at most one non-Scout defender within two tiles, across 223 games including 91/104 proxy wins. Overall, 529/592 captures have advance warning and 537 have no garrison immediately before resolution. | Can existing defenders establish and use a firing line before the city falls, instead of merely adding production quotas? A missing final garrison may have been killed earlier. |

The retained seed `3600006` trace is reproduced in the selected audit: city 1
falls at turn 169, after 20 warning turns, with no local defenders. Six friendly
Rangers and a Soldier were elsewhere in the retained planner trace. This is a
specific deployment failure to investigate, not proof that a particular movement
change would save the city or meet the win target.

A visible diplomatic change is that peace agreements while the proxy remains
the public coalition target, with no local army or damaged home, fall from 760
to 57 observations. Meanwhile the recapture and warned-defence exposures above
remain large. These are combined-policy observations; no individual heuristic's
contribution to the win-rate change has been isolated. The three capped proxy
games also remain unresolved, despite zero Hard-only stalls.

## Regression and fixture coverage

Six regressions call `ScriptedBrain::plan` and validate the resulting orders after
all planning phases and final movement reconciliation. They cover two-front
readiness purchases from one treasury, protected-treaty and Normal controls,
retaining a surviving melee garrison, ranged units forbidden from capturing an
empty city, an occupied escort rendezvous, coalition persistence with an
emergency-peace exception, and ranged support before an optional declaration.
Existing focused tests retain the route, threat, containment and budget cases.

The acceptance summarizer now checks the actual `stalled` outcome identifier;
its previous `stall` lookup could silently ignore stalls. Four Python
regressions verify the complete schedule, added-stall rejection, incomplete-run
rejection and separation of a failed proxy target from passing Hard-only guards.
The historical table already reported actual stalls; this corrects automated
acceptance rather than reclassifying those historical runs.

The selected policy re-records the three 40-turn Normal mixed-brain fixtures,
their 80-turn checks, and the 160-turn Hard fixture. Both Normal sets remain
byte-identical. The Hard initial hash stays unchanged; 126 order maps and hashes
change, starting at turn 35. Its new FNV-1a is `17683572426304174917` and SHA-256
is `103766feb0312114196da3a5ce1304e7f2c5af6557ebfaa15999061f8748510f`.
The [fixture notes](../../crates/sim/tests/fixtures/README.md) explain the visible
Scout and legal-declaration risk behind the first changed settler route. The
tape is generated from fresh planner decisions, and its generator assertion is
updated together with it. Manual scenario tapes retain their authored orders.

## Validation

All commands below completed with exit code 0 on the selected implementation.
Rust test totals include unit, integration and documentation tests; ignored
tests remain ignored. Build and test jobs use at most four workers per process,
and the check sequence runs one command at a time.

| Command | Result |
|---|---|
| `cargo fmt --all` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace --release --no-fail-fast` | 1,231 passed, 0 failed, 4 ignored; exit code checked |
| `cargo test --workspace` | 1,231 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | 14 passed |
| `scripts/build-wasm.sh` | Pass |
| `wasm-pack build crates/wasm --release --target nodejs --out-dir ../../target/wasm-node` | Pass |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | 11 tapes / 615 turns; native and WASM initial, turn and save/load hashes match |
| `cd app && npx tsc --noEmit` | Pass |
| `cd app && npx vitest run --maxWorkers=4` | 105 files; 1,381 tests passed, 1 skipped |
| `python3 -m unittest discover -s tools/analysis -p test_hard_vs_human_playbook.py` | 4 passed |
| `git diff --check` | Pass |

The evidence bundle retains each command, working directory, duration, exit
code and check-log hash. The final branch also passes a conflict-marker scan;
`CLAUDE.md` and `docs/learnings.md` are unchanged from the resumed master.

## Reproduction

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=1 RAYON_NUM_THREADS=4 BINARYEN_CORES=4
PLAYBOOK_RUN_DIR="$(mktemp -d /tmp/legatum-hard-human-repro.XXXXXX)"
cargo build --release -p legatum-sim
target/release/legatum-sim \
  --proxy 0 --proxy-version 6 --proxy-start peace --rotate-proxy \
  --legacy-hard all --difficulty hard --barbarians true \
  --games 400 --seed 3600001 --players 8 --speed standard --threads 4 \
  --archive --archive-dir "$PLAYBOOK_RUN_DIR/v6-archive" \
  --out-dir "$PLAYBOOK_RUN_DIR/v6" --label v6
target/release/legatum-sim \
  --games 200 --seed 3200001 --players 8 --brain scripted \
  --legacy-hard all --difficulty hard --barbarians true \
  --speed standard --threads 4 --out-dir "$PLAYBOOK_RUN_DIR/hard" --label hard
cargo run --release -p legatum-sim --example hard_defense_audit -- \
  "$PLAYBOOK_RUN_DIR/v6-archive" 3600001 4 > "$PLAYBOOK_RUN_DIR/audit.json"
```

The resumed acceptance comparison uses v6; the v5 figure above is a retained
control, not a post-change measurement. Use a fresh archive directory for each
policy: an existing directory would combine recordings from different runs.
The resumed runs use four worker threads each, with at most eight compute
workers scheduled across concurrent jobs. No new policy candidate is introduced
in this continuation.

The observer accepts archived recordings only as an independent observer,
verifies every hash, and emits compact JSON. Historical raw runs remain in
`/tmp/legatum-hard-human/`; fresh runs, frozen binaries, source manifests and
check logs are in `/tmp/legatum-hard-human-resume/`.

The committed [evidence bundle](2026-10-01-hard-vs-human-playbook.evidence.json.gz)
retains run metadata, per-seed results and hashes, both full observer audits,
fixture comparisons, source/binary hashes, the measured candidate-47 patch and
check receipts. Recompute its summaries with:

```sh
python3 tools/analysis/hard_vs_human_playbook.py \
  docs/sim-results/2026-10-01-hard-vs-human-playbook.evidence.json.gz
```
