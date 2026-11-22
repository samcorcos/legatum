# 8.9.25 Mongols: takeover gap, November 7

## What T37 changed

The designer staged a Horseman toward unwalled Dyrrachium on T37, upgraded two veterans to Keshigs, declared on Byzantium on T38, shot Dyrrachium on T40–T41, captured it on T41, and accepted Peace on T42. Hard from the recorded T37 position **does upgrade those same two Horsemen**, and declares on Byzantium one turn earlier. It does not immediately abandon, retreat, or make Peace. Its opportunity strike moves the Keshigs instead of shooting in T38–T41; it takes Dyrrachium on T41 in the salt-0 continuation, then keeps pursuing Nicomedia through T47. Its T46 response rejects a peace proposal. Hard selects a Peace *plan* at T48, but the actual Byzantine war continues through T70; Sparta declares on T50 and captures a Mongol city on T52, and Hard has four active wars by T70. The plan and actual treaty must be distinguished. These are observed paths, not a proof that any one order accounts for the full 16/48 takeover drop.

The comparison branches begin in different positions. Hard from T31 declares on Byzantium at T33 and takes **Nicomedia** at T37, four turns before the T37 branch takes Dyrrachium; it selects a Peace plan at T38. Hard from T34 declares at T34, changes its target between Nicomedia and Dyrrachium, and takes Dyrrachium on T42. The T31 20/48 to T37 4/48 decline therefore combines the designer's intervening position, Hard's different target and staging, and the continuation after capture. The earlier [window diff](../../../playtests/diffs/mongols-2026-11-06-8.9.25-decision-diff-windows.md) finds the designer's T34–T38 army orders worth +18.6 percentage points [10.5, 26.8] over Hard's, the largest tested five-turn window. It is a paired order effect within that window, distinct from the T31/T37 branch-position comparison.

## T34–T41 decision trace

The recording's human orders and three salt-0 Hard continuations are under `~/LegatumRuns/takeover-gap-1107/takeover-orders/trace/`; the recording tape and all state snapshots remain outside Git. The trace's final-order rows distinguish a scored plan from the tactical order that actually ran. The original-engine window diff has zero recorded-state hash and zero AI strategic-order mismatches. The current-engine one-turn diff has drift from the old engine and is used only to locate candidate passes, not for an effect size.

| Turn | Designer | Hard's relevant path and classification |
| --- | --- | --- |
| 34–36 | Moves Horsemen and a Soldier toward Byzantium without a declaration | T34 Hard selects and executes a Nicomedia campaign while Dyrrachium is **considered and rejected**. In the T34 continuation it executes a Dyrrachium campaign on T36, then retargets Nicomedia on T37. Those are executed plans in different turns, not a tracer `selected_not_executed` event. |
| 37 | Moves the leading Horseman to Dyrrachium's approach; upgrades units 26 and 42 | The T37 continuation **executes** the two upgrades and a Byzantine declaration. A new Dyrrachium campaign candidate is **gated out** by the in-reach force gate (`crates/ai/src/plan.rs:3771`); Nicomedia lacks a funded forecast (`plan.rs:3764`). The ongoing campaign still sends the army forward. |
| 38–39 | Declares T38 and attacks the Dyrrachium garrison/city | Hard **executes** the Nicomedia campaign; Dyrrachium is **considered and rejected** (scores 589 vs 1536 at T38, 414 vs 718 at T39). Peace is **considered and rejected**. Open-city capture is gated by visibility on T38 (`military.rs:361`), then no free charger on T39 (`military.rs:409`); the Nicomedia capture gate sees walls above half HP (`military.rs:386`). The final army orders are moves, with no Keshig shots. |
| 40–41 | Keshig 26 shoots Dyrrachium on both turns; Soldier and Horseman press the city; captures T41 | Hard's Dyrrachium campaign is **considered and rejected** T40 and **executed** T41; Peace remains **considered and rejected**. Dyrrachium's open-city seize is **gated out** for no free charger (`military.rs:409`), although regular combat takes the city after T41. The Keshigs receive move orders from `press_strike` (`military.rs:669`); `dual_fire` runs later (`military.rs:1541`) and cannot give an already used unit a bow shot. |

No relevant T34–T41 plan, capture, or diplomatic option in the three traced continuations has the tracer outcome `selected_not_executed`. The absence of a Keshig shot is a **pass-order execution gap** visible in the final orders, not a rejected scored `RangedAttack` candidate. Gate labels refer to the current source lines; their frequency alone is not a causal estimate.

The order audit below checks every recorded army or diplomatic order against Hard's final order for that unit on the same turn. It uses the T34 branch for T34–T36 and the T37 branch for T37–T41. A different executed move is an **executed alternative**; the tracer does not enumerate each possible movement or shot as a scored option, so an absent human shot is not falsely called `considered_rejected` or `selected_not_executed`.

| Turn | Same final order | Hard's executed alternative or missing human order |
| --- | --- | --- |
| 34 | Unit 37's move | Human skips 3, 19 and 26; Hard moves all three. |
| 35 | Unit 19's skip | Human paths 3 and 28 and moves 26; Hard moves 3 and 26 to different tiles and skips 28. |
| 36 | None | Human paths 3, 19 and 42 and skips 26; Hard moves 3, 19 and 26 elsewhere and gives 42 no order. |
| 37 | Upgrades 26 and 42 | Human paths 19 toward Dyrrachium and skips 3; Hard moves both to different tiles and executes the Byzantine declaration one turn before the human. |
| 38 | Declaration already executed at T37 | Human moves 3 and 19 into Dyrrachium and paths 26; Hard moves all three to different tiles. |
| 39 | None | Human paths 26, 28 and 42 and moves 3 and 19 into the city; Hard moves the first three elsewhere and skips 3 and 19. |
| 40 | Moves 3 and 19 into Dyrrachium | Human shoots with 26 and paths 42; Hard moves 26 into the city and sends 42 elsewhere. |
| 41 | Moves 3 and 19 into Dyrrachium | Human shoots with 26 and paths 33, 37 and 42; Hard moves all four toward different tiles or the city. |

The continued T37 trace identifies the treaty failure. Byzantine Peace is **gated out** as `peace would be broken` on T41–T43, T47–T48 (`crates/ai/src/diplomacy.rs:2340`). It is **considered and rejected** at treaty scores −7, −5 and −3 on T44–T46 (`diplomacy.rs:1337`), including an actual incoming proposal declined on T46. At T48 the Peace *plan* is selected, but the treaty gate still says it would be broken. At T49–T50 the desire gate opens but treaty scores stay −6 and −5. Hard finally **executes** Byzantine Peace offers on T53, T60 and T67, after the other front opens; Byzantium does not accept them in this salt. This is the direct plan-to-treaty gap that the `mongol_capture_peace` gene (since removed) targeted. T31's salt-0 branch also has a long Byzantine war, but it takes Nicomedia earlier and has only one war by T55, whereas T37 has Byzantium, Sparta and India active by T52.

## Measurement

The `mongol_capture_peace` gene (measured no gain and removed from the code at landing, 2026-11-07; it never defaulted on) honored a nonpositive follow-up campaign or a Peace plan for six turns after Hard Mongols capture and still hold a city against their sole war opponent. The ordinary VP and siege vetoes remain; a legal treaty still requires the rival's consent. In the traced T37 salt-0 continuation, it changes the T46 incoming Byzantine Peace response from **reject** to **accept**, and the war ends at T47. The focused scope and treaty-score test passed. The paired takeover curve is:

| Branch | Gene 0 | Gene 1 | Paired flips: gains / losses |
| ---: | ---: | ---: | ---: |
| T31 | 20/48 | 22/48 | 8 / 6 |
| T34 | 9/48 | 12/48 | 3 / 0 |
| T37 | 4/48 | 2/48 | 1 / 3 |
| T55 | 34/48 | 31/48 | 0 / 3 |

The branch total is 67/192 in both arms, paired 0.0 percentage points [−5.2, +5.2] by 10,000 resamples of the 192 branch/salt differences. The T37 dip is **not repaired** by signing this Peace; the local diplomatic failure is real, but other positioning, target and conversion failures still decide the takeover. Result: measured no gain; the gene was removed rather than kept as dead code.

The [human-seat comparison](human-seat.json) uses 256 paired salts on each of eight maps (2,048 games per arm). Control wins 679/2,048 seats (33.15%). Gene 1 in the human seat wins 680 (+1 flip, +0.049 pp [0, +0.146]); in rivals only it remains 679 (zero flips); in every seat it wins 680 (+1 flip). The human-seat gate is effectively neutral in this sample.

The AI-only census ran 4,096 Hard games per arm on each seed, with zero panics. On seed 9311003, Mongol win share per seat changes **16.80% → 16.72%**, paired −0.08 pp [−0.60, +0.47] over 3,655 matched Mongol seats ([full faction report](census-9311003.json)). On held-out seed 8241001 it changes **17.04% → 16.55%**, paired −0.50 pp [−1.05, +0.06] over 3,626 seats ([full faction report](census-8241001.json)). Neither interval establishes a Mongol gain, and the held-out point estimate is adverse. Germany's −0.38 pp [−0.74, −0.03] on the first seed does not repeat on the held-out seed (−0.08 pp [−0.43, +0.24]). This gene is a traced local correction, not a validated Hard improvement; it stays off by default.

The paired AI-only batches are `~/LegatumRuns/cloud/gap-peace-{ctl-9311003,g1-9311003-r2,ctl-8241001,g1-8241001}-1107-*`; the four human-seat arms are `gap-peace-hvh-{ctl-r2,seat-r3,rivals-r3,all-r3}-1107-*` in the same directory. Each AI-only arm used the same Linux ARM `legatum-sim` binary (SHA-256 `047476a7ba6285547cef81771a7d15ab4d36fd2d4d5edd181a3ad2bad95e1298`); each human-seat arm used the same `hard_vs_human` binary (SHA-256 `51aba55c554644b3cda3777913fe418db3bc50e2758201e4a7062990a15b0c62`). The gene arms supplied only `mongol_capture_peace=1`. Both binaries were built from the candidate logic before formatting, which changed no behavior.

Two narrower experiments were not landed. A bow-siege pass did produce the missing T40 Keshig shot in the salt-0 trace, but the paired curve moved T31 20→16, T34 9→10, T37 4→2 and T55 34→28 of 48. An earlier captured-city handoff experiment left T37 at 4/48 while T55 rose 34→38/48; it released the army only around T45 and did not change the Nicomedia pursuit. These screens show that matching a visible designer order or removing one hold is insufficient to match his full campaign.

## Reusable audit

`python3 tools/decision-audit/takeover_gap.py SAVE.legatum --out ~/LegatumRuns/takeover-gap-REVIEW` copies the save/tape, exports hash-checked recording-engine states, branches current Hard every five turns by default with 48 salted continuations per branch, picks the largest adjacent rise, runs the recording-engine window diff, traces Hard across that interval, and ranks observed gates by seat-turn. `--turns`, `--salts`, `--seat`, `--end-turn`, and `--states` narrow a study; `--engine` supplies a recording commit for a bare tape. `gates.json` retains every traced outcome, while the generated README shows the leading exclusions and downstream losses. The ranking identifies where to inspect; it does not assign win-rate effects to individual gates. The coach skill now calls for this audit in every review.

The one-salt tool smoke with supplied states completed the curve, trace, and window diff. Its 83-turn recording-engine replay had zero state-hash and zero AI strategic-order mismatches. A separate no-states end-to-end smoke exported all 83 historical states with zero hash mismatches, completed the curve, window diff and trace, and wrote `~/LegatumRuns/takeover-gap-1107/tool-export-smoke/README.md`; that window replay also had zero hash and AI strategic-order mismatches.

## Checks and limits

On the pre-merge candidate, `cargo fmt --all`, `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`, and `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked` passed; the release-checked run passed 2,274 tests across 85 binaries (17 skipped) and all doc tests. `npx tsc -b --pretty false` passed. `npx vitest run` failed on one unrelated icon test (`src/icons.test.ts`, `Celts Ranger`: missing `unit-thornhunter` asset); 3,225 tests passed and two skipped. This branch changes no UI assets or icon mappings. The full `npx playwright test` run was stopped when master moved, after 513/1,057 reported tests, with two failures and ten skips. Both failures were `e2e/tour/athens-greece-captive.spec.ts` at different viewport sizes: the test expects “gains a free technology” and the UI says “gets one free technology.” The process exit code was 143 after termination; it is not a full-suite result. The merged branch is checked below with determinism and pin tests only, as required by the task's landing instructions.

These three Hard traces use salt 0. The gates and final orders explain those observed continuations; the 48-salt takeover curve measures their aggregate win effect. A gate's frequency in the trace is not its marginal win-rate contribution. The gene repairs one treaty decision but leaves the T37 takeover deficit and does not pass the broader AI-only benefit screen.

After these measurements and full checks, master advanced to `92d41edae` with rule changes including Mongol Peace passage. It was merged cleanly into this branch. The numbers above therefore describe the measured pre-merge candidate on `6a6fbcce7` plus this branch's default-off gene, not a new census of the merged rules. Per the task's landing instructions, only the determinism and pin tests are rerun on the merged code; a future balance claim for the merged rules needs a new census.

On the merged branch, `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked --test determinism` passed 12/12 tests. `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked -- pinned` passed 3/3 matching tests (one ruin-roll pin and two runner tape pins), with no changed files sent back by either run.
