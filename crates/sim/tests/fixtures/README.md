# Determinism fixtures

## Re-pin after an intentional change

From the repository root, run `tools/sim/repin.sh`. It regenerates the three
40-turn mixed AI tapes with `--threads 6` under the shared sim slot, rehashes
every fixed-order JSON tape, recomputes the 80-turn mixed and 160-turn Hard
runner digests in `crates/sim/tests/pins.json`, and runs the release pin and
determinism tests. The runner pin tests read that file and name the command when
a digest differs. Keep `pins.json` outside this directory: replay import and the
WASM checker treat every JSON file here as a tape.

## Rules 53: Humble the mighty counts the top rival when you lead (2026-11-22)

`LEGATUM_REPIN_THREADS=2 tools/sim/repin.sh` on `claude/review-rules-51` merged with `origin/master`
at 61f7dab (rules 52, identifiers renamed) kept all three mixed-AI tapes, every fixed-order replay
hash and all three 80-turn mixed digests. Only the 160-turn scripted-Hard city-fire digest moved
(`6650519879623500650` to `2055543751409732395`): the leader now counts captures from its top
rival for Humble the mighty, and Hard values those captures. No scenario state hash changed. The
pin tests and all 12 determinism tests passed.

## Final rules-50 tree: master merged again, Gold gifts, wonder prices, Star Fort (2026-11-22)

After merging `origin/master` again (Egypt's Settler values) and adding the Gold gift order, the
wonder price curve (Pyramids 360, Great Library 400) and the Star Fort at Machinery,
`LEGATUM_REPIN_THREADS=4 tools/sim/repin.sh` rewrote all three mixed-AI tapes. Against
`origin/master` (rules 49), `pins.json` moved: `8858327448064776127` to `8490559115788180378`,
`16340034733011875483` to `10053475625268228984`, `7765247304216437743` to `7018178352511085109`,
and the city-fire digest `3483322890443682319` to `5506138808811481219`. The AI never gives Gold,
so the gift order is not a cause; the wonder prices and the Star Fort tech change AI build choices.
No scenario state hash changed. The pin tests and all 12 determinism tests passed.

## World Ages only, Celtic sack point, master merged; still rules 50 (2026-11-22)

On the combined tree (the rules batch, the Celtic sack exception, `origin/master` merged in and World
Ages as the only era scoring with three peaceful cards per Age), `LEGATUM_REPIN_THREADS=4
tools/sim/repin.sh` rewrote all three mixed-AI tapes and the fixed-order tape
`city-strike-awareness.json`. In `pins.json`, all three 80-turn mixed digests moved
(`6795863348978152354` to `4630297540251091882`, `5356065880517054856` to `7388004569627699792`,
`7765247304216437743` to `11442789816384383562`). The 160-turn scripted-Hard city-fire digest also
moved (`9439788634490833228` to `12822712017351877491`). The causes are a different card per Age
(the draw is over the new pool) and the AI chasing the peaceful cards. The pin tests and all 12
determinism tests passed after regeneration.

## Fable review batch, rules 50 (2026-11-22)

`LEGATUM_REPIN_THREADS=4 tools/sim/repin.sh` regenerated the mixed-AI tapes. Only
`game-00002-seed-18446744073709551615.json` changed. Every fixed-order replay hash stayed the same.
In `pins.json`, the first two 80-turn mixed digests moved (`8858327448064776127` to
`6795863348978152354`, `16340034733011875483` to `5356065880517054856`), and so did the 160-turn
scripted-Hard city-fire digest (`3483322890443682319` to `9439788634490833228`). These moves come
from held capital VP, the Alliance upkeep waiver and the seeded same-turn ties. The pin tests and
all 12 determinism tests passed after regeneration.

## Neighbour warning on the rules-43 calendar line (2026-11-16)

`tools/sim/repin.sh` kept all three mixed-AI tapes and all fixed-order replay
hashes unchanged. Only the 160-turn scripted-Hard city-fire digest moved from
`4376497779619858579` to `10818495909813147018`: the default-on
`neighbour_fall` and `busy_leader_front` genes change Hard's orders in that run.
The pin tests and all 12 determinism tests passed after regeneration.

If a merge conflicts in these fixtures, `crates/sim/tests/pins.json`, or this
README, take either side to finish the merge, then run `tools/sim/repin.sh` on
the merged code.
Review the resulting tape, hash, and pin diff for understood changes before
committing. A clean run is idempotent. Do not use regeneration to explain away
an unexpected mismatch.

## Rules cleanup and decadence off (2026-10-31)

`tools/sim/repin.sh` regenerated the three mixed-AI tapes, rehashed every
fixed-order tape, and updated both runner digests. The old title and milestone
schema was removed and the default decadence slope became zero. Fixed-order
orders are unchanged; their initial and resolved hashes changed under the new
state schema and rules. The three mixed-AI tapes keep their 40 orders each.

## Ten-turn Peace truce (2026-10-25)

`tools/sim/repin.sh` regenerated the tapes after Peace began a ten-turn truce and
the AI started pricing the campaign opportunity it gives up. The initial hashes
did not change. In the three 40-turn mixed tapes, seed MAX-2 first changes on
turn 25 and seed MAX on turn 21: the AI no longer offers Peace on those turns.
Seed MAX-1 is unchanged. The fixed-order `diplomacy-slice-1.json` keeps its
orders and changes ten resolved hashes starting on turn 3. All four runner pins
changed. The release pin tests and all 12 determinism tests passed on the
regenerated files.

**Speed cleanup (2026-10-10):** ordered tapes `diplomacy-alliance.json`,
`production-queue-buy.json`, and `research-queue.json` were re-recorded from their examples
under Standard rules. The Quick-only `promotion.json` tape, its replay test, and the
`promotion_tape` recorder were dropped: its 60-turn chase no longer earns a Standard promotion.
Quick-only offer-pacing boundary cases were also removed; the Standard turn-15/16 cases remain.

The three mixed-AI version-4 tapes pin the initial state and all 40 per-turn hashes for
seeds `18446744073709551613`, `18446744073709551614`, and `18446744073709551615`
(`u64::MAX - 2` through `u64::MAX`). These seeds exceed both wasm32's `usize` range
and JavaScript's exact-integer range. Seeds and hashes stay decimal strings in
the tape format. The games use eight AI players with alternating scripted and
random brains (`--brain mixed`); replay uses their recorded orders, without AI.

## Free Scouts and starting Gold (2026-10-06)

Every empire now starts with 30 Gold at Quick speed (60 at Standard, `game.rs::new_game`), so every
tape's initial hash changed, and Scouts no longer count toward unit upkeep
(`data::pays_upkeep`), so resolved hashes moved too. The scripted AI counts only upkeep-paying units
in its upkeep, runway and disband maths, and its "disband a Scout the rush opening cannot fund"
branch is gone. `tools/sim/repin.sh` regenerated the three 40-turn mixed tapes, `rehash_fixtures`
rehashed the fixed-order tapes (orders kept) and the runner digests in `pins.json` were repinned;
the two archived garrison replay hashes in `crates/ai/tests/ranged_garrison.rs` followed.

## City healing +20 (2026-10-06)

A city now regains 20 HP a turn (was 5) unless it took damage that turn or an at-war military unit
is adjacent, and a city that took battle damage records `last_attacked_turn`. The three 40-turn
mixed tapes were regenerated with the documented command: the recorded orders came out identical
for all three seeds, and the per-turn hashes diverge only where a damaged city healed: turn 26 for
seed MAX-2, turn 37 for MAX, none for MAX-1 (its file is byte-identical). The fixed-order
scenario tapes kept their hashes. The three default-AI runner digests and the scripted-Hard
city-fire digest in `pins.json` were re-pinned, and the two archived garrison replay hashes in
`crates/ai/tests/ranged_garrison.rs` followed (that test's expected city HP is now
`117 + 20 * (turn - 32)`).

## City-built improvements (2026-10-06)

A resource now yields its bonus only once a city has improved its tile (a production item that
names the tile), so the very first resolution differs: every tape's turn-1 hash changed while the
initial hashes stayed fixed. The scripted AI also builds improvements (valued by yield gain over
cost, `production.rs::improvement_value`), so the three 40-turn mixed tapes were regenerated with
the documented command: orders first changed on turn 2, 3 and 2 for seeds MAX-2, MAX-1 and MAX.
`rehash_fixtures` kept every fixed-order scenario's orders and initial hash
(`city-strike-awareness`, `production-queue-buy`, `research-queue`, `diplomacy-alliance`); only
resolved hashes changed. The 80-turn mixed and 160-turn Hard runner digests in `pins.json` were
re-pinned for the same rules and AI, and the two archived garrison replay hashes in
`crates/ai/tests/ranged_garrison.rs` followed. After merging master (map art pass, LLM seat removal) and letting a captured city keep its improvements (lead decision 2026-10-06), only the 80-turn runner digest in `pins.json` moved; the regenerated tapes were byte-identical. A 48-game Standard Hard opening probe (seed 9100,
60 turns, `OPENING_TURNS=60`) measured first Settler T19 and second city T26 (T18/T24 before):
the cost of building the first Farm or Pasture before its Food counts. Not balanced.

## Production banking at 65% on the combined batch (2026-10-04)

After merging the Sparta redesign, Japan/Swiss tuning, and the other
batch changes, the 48-game opening probe measured first Settler/second city
medians of T18/T25 Standard and T16/T23 Quick at 75%; T17/T24 and T16/T22 at
70%; and T17/T23 and T15/T22 at 65%. The 65% price meets all four turn targets.
The documented three-game mixed command regenerated the 40-turn tapes on this
combined code. Initial hashes and turn-1 hashes stayed fixed; all 39 later turn
hashes changed. Orders first changed on turn 4, 3, and 2 for seeds MAX-2,
MAX-1, and MAX respectively. `rehash_fixtures` kept every fixed-order scenario
order and initial hash; only `city-strike-awareness.json` changed resolved
hashes, starting on turn 2 (159 of 160). The longer runner pins were updated
for the same rules and AI. The separate archived garrison replay in
`crates/ai/tests/ranged_garrison.rs` kept its historical input and orders;
its two current-rules state hashes were re-pinned after the combined rule
changes altered unrelated empires during resolution.

## Production banking merged with terrain sight and the Byzantium limiter (2026-10-04)

After the terrain sight/Celts resync landed on master, the 48-game-per-speed
opening probe measured Standard first Settler/second city T18/T26 and Quick
T16/T23 at 80%, versus T17/T25 and T15/T23 at 75%. The latter was closer to
the T18/T25 and T15/T22 targets, so the price at that stage was 75%.
The documented three-game mixed command regenerated all three 40-turn tapes
on the combined code. The
fixed-order `city-strike-awareness.json`, `production-queue-buy.json`, and
`research-queue.json` were rehashed with their orders intact; their initial
hashes stayed fixed and their first changed resolved hash is turn 2. The other
fixed-order tapes replay unchanged. The longer 80-turn mixed and 160-turn
scripted-Hard runner digests were re-pinned to the combined rules and AI.

## Production banking merged with the Byzantium knowledge limiter (2026-10-04)

The documented three-game mixed command on the combined code reproduced all three
of the banking branch's 40-turn tapes exactly: same setup, orders and hashes.
`rehash_fixtures` found no changed initial or turn hash in any fixed-order tape.
The longer runner tapes did change: the 80-turn mixed pin keeps MAX-2 but changes
MAX-1 and MAX, and the 160-turn scripted-Hard pin changes. These were re-pinned
to the combined rules and AI; the shorter fixture content needed no new bytes.

## Tuned first Settler cost, 80% (2026-10-04)

The opening probe measured 48 eight-player Hard games per speed, seed 5200001, through
turn 60. At 75%, median first Settler/second city turns were Standard 16/24 and
Quick 15/23; at 85%, they were 18/26 and 16/25; at 80%, they were 17/25 and
15/24. The target was Standard 18/25 and Quick 15/22. The 80% cost is the closest
tested fit across all four targets; Quick's second city remains two turns late.
The documented three-game command regenerated the mixed tapes for this intentional
cost change. Initial hashes are unchanged. Every game changes all 40 resolved
hashes, starting on turn 1; AI orders change on turns 2–40. The fixed-order
`city-strike-awareness.json` keeps its orders and initial hash and changes 155 of
160 resolved hashes, first on turn 6. All seven other fixed-order fixtures replay
unchanged. The runner's mixed-brain and scripted-Hard pins were updated.

## Per-item production banking, population-2 Settlers, 10-Food first growth (2026-10-03)

Production progress now belongs to its item (switching banks it in `City.production_banked`),
a Settler needs population 2 to be chosen, and the first growth costs 10 Food at every speed;
the AI builds a Monument (or a Scout if it has none, or a Soldier when exposed) before its
Settler. Barbarians also stop targeting concealed Celts hunters, which they could not hit. The
documented three-game command regenerated the mixed tapes: initial hashes are unchanged; each
tape's first resolved hash changes on turn 1 and its first order on turn 2 (40/40 hashes).
`rehash_fixtures` recomputed the fixed-order tapes with their orders unchanged:
`city-strike-awareness.json` first changes on turn 2 (159/160); `production-queue-buy.json` and
`research-queue.json` also change from turn 2; the diplomacy, movement and promotion tapes are
unchanged. The runner's mixed-brain and scripted-Hard pins were updated for the same reasons.
`production_banked` is omitted when empty, so states without banked progress serialize as
before. This is an intentional rules change, not drift repair.

## Persistent AI campaigns (2026-10-04)

The campaign planner changes AI orders but no core rule. Regenerate the three mixed
40-turn tapes and the two longer runner pins from the same release binary, through
the shared simulation slot:

```sh
tools/sim/slot.sh -- target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports --threads 6
tools/sim/slot.sh -- target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 80 --brain mixed --threads 6 --label runner-mixed --tape-out target/runner-tapes/mixed --out-dir target/runner-tapes/reports
tools/sim/slot.sh -- target/release/legatum-sim --games 1 --seed 3200001 --turn-limit 160 --difficulty hard --brain scripted --players 8 --threads 6 --label runner-hard --tape-out target/runner-tapes/hard --out-dir target/runner-tapes/reports
```

Compute the runner pins with `legatum_core::state::fnv1a` over the generated
compact tape JSON bytes. In seed order the 80-turn pins are
`8309441851396734430`, `863643882075495608`, and `7248796923556689640`;
the 160-turn Hard pin is `12666606629334071971`. The funded-declaration gate
changes this long Hard tape; the three 80-turn mixed pins and 40-turn mixed
fixtures remain unchanged. The fixed-order replay
fixtures do not change because core resolution and serialization did not change.
All three mixed setups and initial hashes match the preceding fixtures; their
resolved hashes change on 23, 8, and 25 of the 40 turns respectively.

## Combined strategic AI on the faction batch (2026-10-04)

After merging master at `73bfc59f`, the three mixed tapes were regenerated from
the combined threat and opportunism AI with the documented command:

```sh
target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports --threads 6
```

All three initial hashes match master. AI orders and resolved hashes first
change on turns 9, 33 and 10 for seeds MAX-2, MAX-1 and MAX, affecting 32, 8
and 31 of their 40 turns. The 80-turn mixed runner digests are
`16336190467209105122`, `8961243552110020597` and
`6078601647579298630`; the 160-turn scripted Hard digest is
`5204814294275879430`. Fixed-order tapes keep the batch master's rules and
orders, including its rehashed diplomatic and promotion fixtures.

## Threat and opportunism AI on terrain LOS (2026-10-04)

After merging master at `72587bc6`, the combined AI's three mixed tapes were
regenerated with the documented command and `--threads 6`:

```sh
target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports --threads 6
```

The initial hashes match master. Against master's terrain-LOS tapes, the first
changed orders and resolved hashes are on turns 9, 22 and 10 for seeds MAX-2,
MAX-1 and MAX; 32, 19 and 31 of their 40 turns change. The fixed-order tapes
retain master's LOS-aware orders and hashes. The 80-turn mixed runner digests
are `9026508932219507021`, `18334272047492661112` and
`9101082093672576895`; the 160-turn scripted Hard digest is
`4453314882339681383` after the last-city siege production fix. These pin the
combined AI decisions and engine state.

## Combined AI after the Byzantium knowledge limiter (2026-10-04)

After merging master at `47dc9397`, the combined threat and opportunism AI was built and
the three mixed tapes regenerated with the documented command and `--threads 6`:

```sh
target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports --threads 6
```

The fixtures remained byte-identical. The 80-turn mixed runner pin changed only for
seed MAX, from `12972022002686359581` to `6253293679175101360`; the 160-turn scripted
Hard pin changed from `17090810214578239967` to `5127135788600067089`. Those longer
games reach the new Byzantium salvage economy and research valuations. Native and WASM
replays below verify the unchanged 40-turn fixtures.

## Threat response and attack opportunism merged (2026-10-04)

The merged AI keeps threat tracking, defensive garrisons and coalitions alongside
forecast opportunistic strikes, capture holding and front reconnaissance. The documented
three-game command was run on the combined tree, with `--threads 6` for the shared
simulation slot:

```sh
target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports --threads 6
```

All three initial hashes are unchanged. Compared with the defender branch before the
opportunism merge, the first changed turns are 19, 5 and 10 for seeds MAX-2, MAX-1 and MAX;
22, 36 and 31 of their 40 resolved hashes changed. These are changed AI orders, not
engine rule changes. The 80-turn mixed and 160-turn scripted Hard runner pins were
recomputed on this combined tree. Applying the same peace assessment to incoming
proposals changed the 160-turn scripted Hard pin once more; the 40-turn fixtures and
80-turn mixed pins stayed unchanged. A later merge of the Byzantium knowledge limiter
requires another fixture and runner pin pass, recorded above this historical section.

## Batch landing merge: Byzantium limiter + terrain LOS/Celts resync + Sparta + Japan/Swiss tuning + perf + build-label (2026-10-04)

Combines the sections below (each already merged with master on its own branch) into one
tree, landed as a single 5-branch batch (Sparta, Switzerland/Japan tuning, the
PlayerView perf change, the build-label branch, and this file's own earlier terrain-LOS/
Byzantium-limiter landing). The three mixed 40-turn tapes and the two longer runner pins
(80-turn mixed, 160-turn scripted Hard) were regenerated once, after every branch in the
batch was merged, from the fully combined code: `18446744073709551613` through `..615`
now hash `3363021069328874025`, `11320608226716604024` and `1166800409526109987`; the
160-turn scripted Hard digest is `10651798691129246807`. `rehash_fixtures` changed
`city-strike-awareness.json`, `diplomacy-alliance.json`, `diplomacy-slice-1.json` and
`diplomacy-slice-2.json` (all four initial hashes change too: the Japan start
reassignment changes their setups), while `movement-path.json`, `production-queue-buy.json`
and `research-queue.json` stayed byte-identical. `promotion.json`'s archived orders no
longer produced a promotion under the combined rules (the same failure mode documented
below for the original Swiss/Japan tuning), so it was regenerated with
`cargo run --release -p legatum-wasm --example promotion_tape`: a fresh 12-turn tape with
two battles and a promotion on turn 9. The `ranged_garrison` archive-replay pins
(`crates/ai/tests/ranged_garrison.rs`) also needed new values under the combined rules:
`14620570585179503101` then `9280572318996843714`, keeping the original archived orders
and archived-hash provenance checks. Byzantium's once-per-landowner salvage knowledge, the
terrain-LOS/Celts resync, the Sparta redesign and the Japan/Swiss tuning are
independent systems, so this is a mechanical recombination, not a new design decision.

## Landing merge: Byzantium knowledge limiter + terrain LOS/Celts resync (2026-10-04)

## Byzantium knowledge paid once per landowner (2026-10-04)

The documented three-game command below regenerated all mixed tapes after salvage knowledge
changed from a payment per camp to one payment per camped landowner. Their decoded setups,
orders, and all 40 turn hashes matched the committed fixtures, so the existing bytes were kept
(the generator only removed their trailing newlines). Native and WASM replay still check them.
The 80-turn mixed runner pin changes only for seed MAX, the game with a Byzantium seat whose
later salvage and value-based choices differ; the 160-turn scripted Hard pin changes too,
including when the goal rises from 1,900 to 1,950 Science at Quick speed.
Both pins were updated to the observed deterministic digests. This is an intentional economy
 and AI valuation change, not fixture drift.

## Terrain LOS and Celts concealment resync (2026-10-04)

This merge combines terrain-blocked team sight with per-team Celts hunter reveal
persistence. `visibility::rehide_hunters` now uses the same LOS-aware
`visible_tiles` function as exploration and player views: a hunter spotted on a
hill can move behind a Forest or Mountains tile and hide again from a low
viewer, leaving its last-seen sighting. AI city-shield pressure also excludes
volley damage where terrain blocks the shot. Those rule and AI changes alter
both state hashes and later orders, so the three mixed 40-turn tapes are
regenerated with the documented command below and the longer runner AI tapes
are re-pinned.
Against this worktree's master base, all three initial hashes, 120/120 mixed
turn hashes and 120/120 mixed order maps change, beginning at turn 1: different
terrain sight changes the initial views and the AI's first decisions.
The 80-turn mixed FNV-1a digests are `16623041536539354427`,
`14510060268021274621`, and `7534888248038844876`; the 160-turn scripted
Hard digest is `15615839427733491166`.

`city-strike-awareness.json` is a fixed-order scenario, not an AI recording.
Its setup and all 160 recorded order maps are kept exactly; only the initial
and resolved hashes are recomputed with `rehash_fixtures` (the initial hash and
all 160 resolved hashes change, beginning at turn 1). The LOS-aware fog and
concealment state change serialized hashes under those same orders. This
keeps the scenario's original decisions while testing their replay under the
merged rules. The companion resync report in `docs/sim-results/` records the
same order-preservation check and cohort results.
The other seven fixed-order scenarios keep their setup, orders and hashes;
the rehash command confirmed no changes there.

## Sparta with Celts concealment and the removed map ruin (2026-10-04)

After merging master `cced5de1`, the documented three-game command regenerated all
three mixed tapes on the combined tree. Their initial states remain identical to
the preceding Sparta branch. Relative to that branch, resolved hashes change
on 40, 28 and 40 of the 40 turns (first on turns 1, 13 and 1); AI orders change
on 39, 17 and 39 turns (first on turns 2, 19 and 2). The fixed-order
`city-strike-awareness.json` retains every order and its preceding Sparta
initial hash; `rehash_fixtures` changes 155 of 160 resolved hashes, first on
turn 6. Relative to master before Sparta, its initial hash and all 160 turn
hashes differ, with the same recorded orders. All other fixed-order tapes keep
their states and orders. The longer AI pins in `runner.rs` were measured on the
combined tree: the three mixed hashes are `9866485191403189394`,
`4274625637088624419` and `11160434528780099101`, and the Hard scripted
hash is `6250713201838669054`. Native and WASM replay check every recorded state.

## Swiss Great Work spending on the merged master (2026-10-04)

The documented three-game command was rerun after the Swiss AI began buying a
planned Great Work prerequisite after its treasury award and protecting the
same-turn award against a projected Gold deficit. All three mixed fixture files
were byte-identical to the committed tapes, so their orders and hashes needed no
new pin. The 80-turn mixed and 160-turn scripted Hard runner pins also passed
unchanged; these recordings did not encounter the new purchase decision.

## Celts concealment merged with final Japan/Swiss tuning (2026-10-04)

Master `cced5de1` added per-team hunter reveals, bump-and-reveal movement and
last-seen sightings. The documented three-game command regenerated the mixed
tapes on the combined source: against the preceding tuning commit, all three
initial hashes, recorded orders and 40/40 resolved hashes remain identical.
Against Celts master, the combined Japan/Swiss rules change all three
initial hashes and the first order and hash of each game. `rehash_fixtures`
preserved every fixed-order scenario order; only `city-strike-awareness.json`
changes against the preceding tuning commit, first at turn 115 (46/160 hashes),
as the Celts concealment movement rules take effect. Its initial hash is
unchanged against that commit. The 80-turn mixed runner digests are
`10977399024212467500`, `15932542596191660624`, and
`1187540482387340099`; the 160-turn scripted Hard digest is
`6781019303312620286`. These longer games do exercise the combined AI and
engine changes, so both runner tests are re-pinned.

## Final 7% Japan network and 5 Gold interest cap (2026-10-04)

After the combined master merge, 400-game diagnostic cohorts showed that a 10%
Japan Food/Production link bonus favored Japan too strongly at Quick. The selected
bonus is 7% per linked city, and the Swiss interest cap returns to 5 Gold.
The documented three-game command regenerated the mixed tapes: all three initial
hashes, every recorded order, and all 40 turn hashes stayed identical. The
fixed-order rehash kept every order; only `city-strike-awareness.json` changes
resolved hashes from turn 109 (52/160), when the changed Japan yield affects that
scenario. The 80-turn mixed digests are `12977101156074693336`,
`15932542596191660624`, and `16253764251683420872`; the 160-turn scripted
Hard digest is `2599977848232902457`. Runner tests pin these results.
The later Sparta route guard against unwalled hostile cities reran the same
generator; all three mixed files were byte-identical, and both runner pins held.

## Map-reveal ruin removal and intermediate faction retune (2026-10-04)

Master removed `RuinReward::MapReveal` and redistributed its roll slots. Japan's
Food/Production network increased from 5% to 10% per linked city while its
capital-distance penalty stayed 5%; Swiss interest cap rose from 5 to 10.
The documented three-game command regenerated the mixed tapes on the combined
source. Relative to the preceding merged tuning commit, initial hashes are
unchanged; MAX-2 first changes an order on turn 2 and a hash on turn 1 (40/40
hashes), MAX-1 on turns 6 and 5 (36/40), and MAX on turns 2 and 1 (40/40).
`rehash_fixtures` kept all fixed-order scenario orders. Its
`city-strike-awareness.json` hash first changes on turn 6 (155/160); the other
fixed-order tapes retained their hashes. The 80-turn mixed digests are
`12977101156074693336`, `15797781097221776082`, and
`16253764251683420872`; the 160-turn scripted Hard digest is
`17657255812316906843`. Runner tests pin these intentional combined changes.

## Swiss/Japan tuning merged with faction-aware technology (2026-10-04)

The documented three-game command was rerun after merging the tuning branch with
master's city shields, routes around friends, and Byzantium Salvage Works. Against the
tuning branch, MAX-2 and MAX-1 kept every order and hash; MAX first changes an
order and resolved hash on turn 10 (31/40 hashes change) as the merged Byzantium
technology and squatting rules alter its AI decisions. Against master, all three
initial states and all 40 hashes change because the faction economy and Japan start
rules differ. `rehash_fixtures` recomputed the fixed-order
`city-strike-awareness.json` state (initial state and first turn changed); the
other seven fixed-order scenario tapes kept their hashes. The merged 80-turn mixed
digests are `10309569735695614121`, `17557276209062789883`, and
`16156494646008290110`; the 160-turn scripted Hard digest is
`2903065429795809733`. These are intentional combined gameplay changes.

## Celts hunter concealment merged with Salvage Works and the ruin MapReveal removal (2026-10-04)

Merging master at 973851cf: the documented three-game command reproduced master's mixed tapes
exactly (same JSON; only the trailing newline differed, so master's files were kept).
`rehash_fixtures` changed only `city-strike-awareness.json`: initial hash unchanged, first
changed resolved hash now turn 77 (orders untouched). The turn-69 barbarian divergence below is
now on master too. On turn 77 (`game_dump` on both trees) a barbarian Scout at (6,14) heads east
and is bumped by a hidden Celts Ranger at (8,14) (`HunterBumped`, no combat), where on master it
steps away to (5,15): routes no longer bend around hunters the mover cannot see. The barbarian fix (raiders plan only against units
their team can see) now comes from master and is no longer duplicated here. Runner pins:
`default_ai_tapes_are_pinned` changes seed MAX-1 only, and `scripted_hard_city_fire_tape_is_pinned`
changes.

## Celts hunter concealment (2026-10-04, branch claude/celts-concealment, merged with master)

A hunter is now revealed per team (`Unit::revealed_to`) by fighting in that team's sight or by a
unit running into it (bump and reveal, no combat), stays revealed while the team keeps its tile in
sight, and leaves a last-seen sighting (`Player::hunter_sightings`). Both fields are omitted when
empty, so initial hashes and archived states are unchanged. After merging master (city shields,
routes around friends) the documented three-game command was rerun: all three mixed tapes came out
byte-identical to master's (same orders and all 40 hashes). No Celts hunter fights, is bumped or is
crossed in these 40-turn games, so nothing new is ever serialized.
`rehash_fixtures` (orders untouched) changed only `city-strike-awareness.json`: initial hash
unchanged, first changed resolved hash on turn 69 (before the later merge). Cause, checked with
`game_dump` on both trees:
on turn 69 a barbarian Scout at (5,13) moves away from two Celts hunters hidden 3 tiles off on
master, but not on this branch, because barbarians now plan only against units they can see
(`rules/barbarians.rs::raider_plan`). Movers' routes also no longer bend around (or treat as parked
friends) hunters they cannot see (`movement.rs::next_step`). Every other fixed-order fixture is
unchanged.
The runner pins moved: the 80-turn mixed tapes for seeds MAX-1 and MAX (`default_ai_tapes_are_pinned`;
MAX-2 keeps its value) and the 160-turn scripted Hard tape (`scripted_hard_city_fire_tape_is_pinned`),
because in those longer games hunters fight, stay revealed or are bumped, and the Celts AI now holds
fire when a shot is not worth its cover.

## Attack opportunism (2026-10-04)

The scripted AI now forecasts opportunistic strikes (`crates/ai/src/opportunity.rs`), keeps a
defender in a city it just captured, scouts a neighbour whose cities it hasn't found, and covers a
bankruptcy by disbanding the least valuable unit (or leaving the engine's single forced disband)
instead of several units (`production.rs::settle_insolvency`). The documented command regenerated
the three mixed tapes (on master with routes around friends and city shields); only AI orders changed, so every initial hash is the same and hashes change
from the first different order:
- MAX-2: first different order and hash on turn 19 (22/40 hashes): Switzerland (seat 0) disbands
  unit 2 instead of 3 for the same deficit.
- MAX-1: turn 5 (36/40): Switzerland (seat 6) disbands unit 20 instead of 21.
- MAX: turn 10 (31/40): Byzantium (seat 4) sends unit 27 out instead of holding it.

The runner's 80-turn mixed-AI pins and the 160-turn scripted Hard pin moved for the same reasons.
No fixed-order fixture changed (no engine rule changed).

## Sparta with faction tech and Byzantium salvage (2026-10-04)

Master added faction-specific research and Byzantium's Salvage Works and value-based
squatting after the Sparta, city-shield and route merge below. Its speed list is now
Quick and Standard only. On the combined tree, the documented three-game command
reproduced the first two mixed tapes byte for byte. In the third, the initial state
and turns 1–4 remain unchanged; Byzantium's salvage movement changes the turn-5
orders and 36 of 40 resolved hashes. The 160-turn fixed-order city-strike tape
keeps every recorded order and, after `rehash_fixtures`, is byte-identical to the
prior Sparta version; it differs from master's pre-Sparta initial state and
all 160 hashes. The other fixed-order tapes also replay without a changed state.
The longer AI-order pins in `runner.rs` were measured on this combined tree:
the mixed pin changes only for seed `u64::MAX` (to `8025389616153047218`), and
the scripted Hard pin changes to `13502903196381201255`.

## Sparta, city shields and routes around friends (2026-10-04)

On master before Sparta merged, city shields and routes around friends
produced the same three 40-turn mixed tapes as routes alone: no garrisoned city
was attacked in those games. The fixed-order tapes also replayed unchanged,
while both longer AI-order pins changed. Those observations describe master
before this Sparta integration.

On the combined Sparta branch, the documented three-game command again
produced byte-identical initial states, all 40 orders and hashes in each mixed
tape. Rehashing the 160-turn fixed-order `city-strike-awareness.json` also
produced identical orders and hashes. The city shield's effect is therefore
recorded by the longer AI-order pins below, without changing these fixtures.
In `default_ai_tapes_are_pinned`, seed MAX-2 keeps the Sparta-and-routes
hash while MAX-1 and MAX change; the 160-turn scripted Hard pin changes too.
The updated expected hashes in `runner.rs` were measured on this combined tree.

This merge combines Sparta's revised starting military, conquest rewards and
restlessness with city shields and `path::route_around_friends`. A city with HP
now absorbs attacks on its tile before its garrison is exposed. The pathfinding rule keeps land
routes out of a friendly unit's tile when that unit stays to defend or attack,
and prefers free tiles when a friend may move out. Native resolution, AI
preflight and the mock share the rule. The resulting mixed-AI tapes were
regenerated with the three-game command below. Compared with this branch just
before the pathfinding merge, all three initial hashes stay the same, all 40
resolved hashes change from turn 1, and AI orders first change on turns 2, 3
and 2 (39, 38 and 39 turns differ). Compared with pathfinding master before
Sparta, the initial and all 40 resolved hashes differ in each tape, while
orders first differ on turns 10, 32 and 10 (31, 9 and 31 turns differ). These
comparisons isolate each side of the merge instead of attributing all changes
to one rule.

The fixed-order `city-strike-awareness.json` keeps its initial hash and all
160 recorded order maps relative to this branch before pathfinding;
`rehash_fixtures` changed 139 resolved hashes beginning at turn 22. The other
seven fixed-order tapes remain byte-identical. The new AI tape hashes are
pinned in `runner.rs`; native save/load and WASM replay check every recorded
state. Sparta's starting promotion changes each setup's initial state;
route selection changes resolution on turn 1 even when orders match.

## Byzantium Salvage Works and squatting AI (2026-10-04)

The Byzantium's Laboratory became the Salvage Works (+1 salvage Science per camp within 4 hexes per
Laboratory-family tier; Byzantium buildings give no city Science), and the scripted Byzantium AI now
squats value-based, values the Works by its camps and treats no-benefit techs as stepping stones.
The documented three-game command was rerun: MAX-2 and MAX-1 came out identical; MAX
(`u64::MAX`) keeps its initial hash and first changes on turn 5, where the Byzantium seat (player 4)
sends Scout 14 to a salvage site instead of exploring (36/40 hashes change). Fixed-order
`city-strike-awareness.json` keeps its orders; `rehash_fixtures` recomputed hashes from turn 152
(a Byzantium building no longer adds city Science). The runner's 80-turn mixed pin changes only its
third game, and `scripted_hard_city_fire_tape_is_pinned` was re-pinned. Intentional rules and AI
change, not drift repair.
On the later merge with the city shield the three mixed tapes regenerated unchanged (the
committed MAX tape already carried this change); both runner pins were re-pinned for the
combination.

## City shield merged with value-ranked ranged targets (2026-10-03)

Merging the two changes below made ranged shot scoring follow the city shield: a garrison in a
city with HP is never a unit target, shots at that tile score as capture progress against the
city's remaining HP, and `enemy_threat` drops shielded garrisons and gates the capture stake on
the shield. The documented three-game command was rerun on the merged tree: all three mixed tapes
came out identical apart from the trailing newline (same initial hashes, all 40 turn hashes and
every order), so the committed files were kept. The fixed-order fixtures still replay unchanged.
Of the runner pins, only `scripted_hard_city_fire_tape_is_pinned` changed and was re-pinned;
`default_ai_tapes_are_pinned` keeps the city-shield branch's values.

## Ranged garrisons fire; value-ranked ranged targets (2026-10-03)

The playtest-6 fix lets a ranged unit's hold become a shot and ranks shots by the threat they
remove (`crates/ai/tests/fixtures/garrison-t32.md`). The documented three-game command was rerun:
all three tapes came out identical (same orders and all 40 hashes; only the trailing newline
differed, so the committed files were kept). No ranged unit in these 40-turn mixed games has a
hold or a target choice that the change affects. The runner's 80-turn mixed-brain pin is also
unchanged; only `scripted_hard_city_fire_tape_is_pinned` was re-pinned (first different order on
T49, a different Ranger target).

## Cities shield their garrisons (2026-10-03)

Civ 6 rule (core `combat::city_tile_defender`): while a city has HP it absorbs every attack on its
tile and its garrison takes no damage; at 0 HP the garrison defends and a melee win over it
captures. The AI's damage and exposure maths follow it. The documented three-game command was
rerun: all three mixed tapes came out byte-identical apart from the trailing newline (same
initial hashes, all 40 turn hashes and every order), because no garrisoned city is attacked in
their first 40 Standard turns, so they were left untouched. The fixed-order fixtures, including
`city-strike-awareness.json`, still replay unchanged. The pinned runner tapes changed and were
re-pinned for this intentional rules change: `default_ai_tapes_are_pinned` (80-turn mixed games;
seeds MAX-2 and MAX-1 changed, MAX unchanged) and `scripted_hard_city_fire_tape_is_pinned`.

## Walls-only city strike (2026-10-03)

Cities now fire only with Walls (any tier, including the Switzerland' Customs Bastion), as in
Civ 6 (`combat::city_can_fire`). The AI also values Walls near aggressive neighbours and prefers
unwalled assault targets. The documented three-game command regenerated the mixed tapes. Initial
hashes are unchanged. MAX-2 first changes a hash on turn 24 and an order on turn 31 (17/40 hashes);
MAX-1 on turns 12 and 19 (29/40); MAX keeps every order and changes 4 hashes from turn 37, where
an unwalled city no longer shoots. Hashes change before orders because the same orders now resolve
without unwalled city fire. `city-strike-awareness.json` keeps its initial state and all 160 turns
of recorded orders; `rehash_fixtures` recomputed its hashes (first change turn 15, 146/160). The
pinned runner tape hashes were updated for the same reason. This is an intentional rules change,
not drift repair.
## Sparta iron price and walls-only strikes (2026-10-03)

The three mixed-AI tapes were regenerated with the command below after Sparta
military began at promotion level 1, gained faster XP, and gained conquest rewards
and restlessness. The walls-only city strike rule was merged first. Initial hashes
change because every setup contains Sparta starting military; the first resolved
hash differs on turn 1 in all three. In that earlier integration, AI orders first
diverged on turns 10, 19, and 10; the final master merge is described above.
Each new tape still records 40 turns. The fixture changes are expected
game rules and AI consequences, not unexplained replay drift.

Fixed-order tapes keep their original orders. `rehash_fixtures` recomputed hashes:
`city-strike-awareness` (160 turns), `diplomacy-alliance` (18), and both diplomacy
slices (12 and 15) change from the initial state because they include Sparta.
`promotion` has no Sparta and first changes on turn 8 under the walls-only city
fire rule. `movement-path`, `production-queue-buy`, and `research-queue` remain
byte-identical. Native save/load and WASM replay checks below pin the new results.

## Byzantium and Celts defaults (2026-10-03)

The command below regenerated all three tapes after the default Byzantium salvage and Celts homeland
rules replaced their earlier yield and goal rules. Initial setups and hashes are unchanged. The
first changed resolved hashes occur on turns 4, 2, and 1 for the three seeds in order; the first
different AI orders occur on turns 7, 16, and 10. The earliest hash changes precede changed orders
because the same orders now resolve under different faction income, building prices, attrition,
and visibility. The new field for a revealed Celts ranged attacker defaults to zero in old saves.
The three tapes still record 40 turns each; native save/load and WASM replay checks verify their
new hashes. This is an intentional rules update, not a repair for unexplained drift.

Three longer fixed-order tapes also changed under these defaults. Their initial states and every
recorded order remain byte-for-byte the same; only resolved hashes were recomputed by replaying
those orders under the new rules. The first changed hash is turn 1 for `research-queue.json`
(120/120 hashes), `city-strike-awareness.json` (160/160), and `production-queue-buy.json`
(150/150). The `promotion.json` scenario is unchanged because its attackers are not Celts hunters.
Replaying the revised tapes with save/load
at every boundary confirms the new hashes while preserving each scenario's original order path.

Run the fast native regression from the repository root:

```sh
cargo test -p legatum-sim --test determinism
```

This test also runs in `cargo test --workspace`. It never generates or rewrites
expected hashes. It checks uninterrupted replay and a second replay that saves
and loads before the first turn and after every turn, then continues from the
loaded state. The final save is checked too. The same checked replay is used by
the native `--tape-in` CLI and WASM `Game.replay_tape`; a mismatched initial hash,
per-turn hash, or turn number fails instead of being accepted.

To check these fixtures in release and WASM:

```sh
cargo test --release -p legatum-sim --test determinism
wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node
node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node
```

Regenerate only when an intentional rules, serialization, map-generation, or AI
change has been reviewed and its changed results are understood. From the
repository root, the following overwrites the three fixtures; reports stay in
the ignored `target/` directory:

```sh
cargo run --release -p legatum-sim -- --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
```

Review the fixture diff, run the native debug/release and WASM commands above,
and commit the new tapes with the intentional engine change and its rationale.
Do not regenerate fixtures just to make an unexplained hash failure pass.

For the larger S8 check, use the 30-game commands and recorded results in
[`docs/sim-results/2026-09-26-cross-target-determinism.md`](../../../../docs/sim-results/2026-09-26-cross-target-determinism.md).

## Eight-faction regeneration (2026-09-26)

B6/B7 intentionally changes city income/costs, faction replacements, military
movement/healing/combat, goals and AI decisions. Simulation setups now shuffle
one of each of the eight factions into seats using a separate seeded setup RNG.
The three committed tapes were regenerated with the command above; each still
contains all 40 turns, with mixed scripted/random brains and high-u64 seeds.
Both initial states and all turn hashes change. The regression separately pins
the three new seat permutations. This is a rules update, not hash drift repair.
Old original-three enum strings, save fields and order item IDs still deserialize;
compatibility fixtures remain frozen under core/wasm tests. Historical rule
versions are not emulated, so old expected replay hashes are not silently accepted.

## Game-speed regeneration (2026-09-27)

The default is now Standard speed (twice the previous economic costs, growth
thresholds, gold goals and timers). Limits are optional; these fixtures retain
an explicit 40-turn limit. Speed applies after faction and diffusion modifiers,
and AI planning uses the same pace. The command above intentionally regenerates
all three initial hashes and all 120 turn hashes under these rules. Legacy
numeric limits still deserialize; a missing speed defaults to Standard, so old
saves load under the new rules, not an emulation of the historical rules.

## Recovery charters removed (2026-09-26)

Recovery charters (doctrines) were removed from the rules and the AI. The command
above was rerun intentionally and produced byte-identical tapes: under Standard
speed no AI adopted a charter within these 40 turns, so no state hash ever
contained one. Old saves with a `doctrine` field still load (the field is ignored);
old `ChooseDoctrine` order JSON is rejected.

## Automatic city priorities (2026-09-27)

City focus is no longer a stored setting or an order. Core derives Production
for Soldier/Ranger/Trooper or Relay Uplink builds, Growth for a Settler below
population 2, and Balanced otherwise (including Scouts). This uses the requested
build rule; the old AI's additional threat, low-population and faction exceptions
are removed. Borders now rank adjacent unowned tiles within radius 3 by total
faction-adjusted yield, with ties by axial q then r, independently of production.
The city view exposes the next tile and build-specific yield previews.

The following command intentionally regenerates all three tapes under these
rules, removing obsolete focus orders and changing hashes after cities are founded:

```sh
cargo build --release -p legatum-sim && ./target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
```

Old saves still load: serde ignores each city's obsolete `focus` field, and new
saves omit it. Other old order JSON remains supported; `SetFocus` is rejected as
an unknown order variant. Old tapes containing that order are not compatible;
historical rules and hashes are not emulated.

## Tech-tree Bicycle first slice (2026-09-27)

The first three eras now contain 25 technologies, paid building-family upgrades,
Marksmen and siege. Research costs/prerequisites, Scout sight, city defense,
healing and AI investment/campaign choices intentionally change. Fogged city
memory also records whether a Citadel was observed; old saves default that field
to false. The enum additions preserve the old identifiers and variant order.

The regeneration command above was run intentionally. All three initial hashes
are unchanged. The first different orders and resulting hashes are at turn 1 in
each tape: the expanded available research list changes random-brain choices.
Later turns also include the revised scripted priorities and rules. These are
deliberate engine/AI changes, not unexplained hash drift. Each tape still has all
40 turns, and native debug/release plus WASM replay checks validate its initial,
every-turn and save/load hashes.

Campaign results and limitations are recorded in
[`the first-slice comparison`](../../../../docs/sim-results/2026-09-27-tech-bicycle-1-comparison.md).

## Bicycle investment and pacing tuning (2026-09-27)

Industrial family prices, faction building yields/costs, research prices and the
Relay prerequisite route intentionally change. The scripted AI also evaluates
industrial upgrades against their quoted marginal production payback. The
regeneration command above was run intentionally under these rules.

All three initial hashes are unchanged. In seed `18446744073709551613`, the first
changed orders and hash are at turn 5: scripted Sparta seat 2 chooses Signal
Theory instead of Fortification. Signal Theory now leads directly to Relay
Restoration, which changes its downstream research value. The other two tapes
are byte-identical to the prior fixtures. Each still records all 40 turns.

Native debug/release and WASM replay check initial, every-turn and save/load
hashes. Additional full-game Quick and Standard tapes exercise later paid
upgrades; commands, results and the held-out campaign are recorded in the
[tuning comparison](../../../../docs/sim-results/2026-09-27-tech-bicycle-tuning-comparison.md).

## Reactor capture limit (2026-09-27)

Capturing a Reactor now dismantles it immediately if the captor already owns or
is constructing one in another city. The capture emits a `BuildingDismantled`
event explaining the limit. The slot belongs to an empire, not its team, and a
Reactor in resistance still reserves it. Other captured buildings are retained.

The regeneration command above was rerun intentionally. All three tapes are
byte-identical, including their initial and 120 turn hashes: their 40-turn
reports contain no completed Reactors or city captures. Targeted core and mock
tests exercise the changed capture rule, report visibility and city forecasts
after resistance; the fixed tapes continue to cover native/WASM replay and
save/load determinism.

## Tech-tree Bicycle second slice (2026-09-27)

Four Cities now also accepts two population-8 cities, without another point.
The legacy `RelayUplink` milestone identifier now represents the shared Great
Work award. Planetary Foundry, World Exchange and Relay Uplink each cost 400
base Production, have explicit infrastructure requirements, and share a single
empire construction/completion slot. Saves retain the old identifiers and Relay
completion flag; the new optional completion-route field defaults to absent.
Relay Restoration returns to the approved 220S / Recovered Power + Civic
Networks route. The scripted AI compares paid construction/research paths and
tall growth forecasts through its fogged view.

The documented regeneration command was intentionally rerun against the final
candidate. All three initial hashes are unchanged. For seed
`18446744073709551613`, the first different order/hash is turn 5: scripted
seat 2 chooses Fortification instead of Signal Theory, whose downstream Relay
shortcut was removed. The other two 40-turn tapes are byte-identical to master.
These opening tapes do not reach Great Works. Separate core, mock and real-WASM
app tests exercise all three routes, public progress, scaled costs, exclusive
completion, interruptions, capture, the tall alternative and save/load.

Native debug/release and WASM replay check initial, every-turn and save/load
hashes. Cohort results and unresolved balance questions are in the
[second-slice comparison](../../../../docs/sim-results/2026-09-27-tech-bicycle-2-comparison.md).

## Great Work and tall-path tuning (2026-09-27)

Great Works cost 300 base Production; Foundry needs one Autoworks host. Industrial,
energy, habitat and route research prices change, and the AI plans overlapping
research/construction, intermediate population gates and committed settlers.
Faction traits, VP values and the two-population-8 alternative remain intact.

The regeneration command was intentionally rerun from the frozen final executable.
All three tapes are byte-identical to the merged second slice, including their
initial and 120 turn hashes. These 40-turn Standard openings do not reach the
changed investments. Native debug/release and WASM checks pass without changing
expected hashes to suppress a failure.

Eight additional full-game development replays cover every Work-winning route
and a sustained compact winner at each speed. They add 1,895 turns of native/WASM
initial/every-turn/save-load validation beyond the opening fixtures. The selected
seeds, archived tapes, complete 2,500-game final results and limitations are in
the [tuning comparison](../../../../docs/sim-results/2026-09-27-tech-bicycle-2-tuning-comparison.md)
and its linked implementation record.

## Full tech tree (2026-09-27)

Seven Inheritance technologies, the paid Inference Core, Assault Troopers,
faction inheritance, and joint late research/construction planning now complete
the tree. Growth, upgrade and upkeep forecasts share the new integer rules with
resolution. Research prices are tuned using the discovery-to-investment window.
Production-choice jitter now keys on the stable item identifier, so adding an
unrelated available item cannot perturb another item's tie-break.

The documented regeneration command was intentionally rerun from the frozen
final executable. All three tapes are byte-identical, including their initial
and 120 turn hashes. These Standard 40-turn openings do not reach the changed
late investments, and their chosen production orders are unchanged.

Two additional complete development-seed tapes (Quick 700723 and Standard
700281) each reach all seven final technologies and paid Inference Cores,
Assault Trooper production and infantry upgrades. They cover another 506 turns
of native/WASM initial, every-turn and save/load replay. The tapes, commands,
measurement definition, cohort results and remaining uncertainties are linked
from the [Full game comparison](../../../../docs/sim-results/2026-09-27-tech-full-comparison.md).

## Persistent research queues (2026-09-27)

Player state now serializes `research_queue`, including the current technology.
Old saves default it to empty and keep their `researching` target; `SetResearch`
remains supported and replaces the plan with a singleton. Queue orders are
validated in prerequisite order and complete at most one technology per turn.

The documented three-game regeneration was run intentionally. All three initial
hashes and all 120 turn hashes change because the state includes the new field;
the 120 recorded order maps are unchanged. A separate 120-turn Quick tape includes
`SetResearchQueue`, explicit clearing, automatic advancement and a finished plan.
It uses an above-JavaScript-precision seed and saves/loads at every replay boundary.
Regenerate that tape only intentionally:

```sh
cargo run --release -p legatum-wasm --example research_queue_tape > crates/sim/tests/fixtures/research-queue.json
```

The standard native determinism test and the directory-wide WASM tape check cover
all four tapes. The original-three compatibility fixtures remain frozen.

## Diplomacy slice 1 (2026-09-27)

Diplomacy slice 1 intentionally changes new games from all-war to Peace. Version 1
saves/tapes retain the legacy all-war rules; version 2 records the new relationships.
`diplomacy-slice-1.json` adds a hand-authored 12-turn sequence covering mutual
war declarations, superseded teammate orders, pending peace offers, acceptance,
the three-turn guarantee, rejection and expiry. It is also replayed with save/load
at every boundary, natively and by the Node/WASM check above. Regenerate it with:

```sh
cargo run -p legatum-wasm --example diplomacy_tape -- crates/sim/tests/fixtures/diplomacy-slice-1.json
```


## Diplomacy Skateboard slice 2 (2026-09-27)

Version 3 records pact terms and irrevocable notice dates, shared empire Influence/debt,
public directed trust/history, and explicit breach declarations. Saves from the all-war
rules remain all-war with local meters; version-2 diplomacy saves consolidate their
city meters into the empire pool when loaded. Historical tape versions identify their
starting rules, but historical engine/AI hashes are not emulated after rule changes.

The three 40-turn mixed-AI fixtures were intentionally regenerated with the documented
simulation command. Their new initial hashes include the new canonical diplomacy fields.
Pooled border allocation and the approved diplomacy scores intentionally change later
orders and hashes; the regenerated expectations are verified natively and in WASM.

- `game-00000-seed-18446744073709551613.json`: initial hash changes; first changed orders at turn 1.
- `game-00001-seed-18446744073709551614.json`: initial hash changes; first changed orders at turn 1.
- `game-00002-seed-18446744073709551615.json`: initial hash changes; first changed orders at turn 1.

`diplomacy-slice-1.json` was regenerated with its existing example; the new
`diplomacy-slice-2.json` covers 15 boundaries with pending offers, notice deadlines,
signing and breach debt, a superseded teammate order, paid breach/trust history, an honored
term followed by ordinary declaration at its expiry, peace, and an unanswered offer.
Regenerate the second tape with:

```sh
cargo run -p legatum-wasm --example diplomacy_slice_2_tape -- crates/sim/tests/fixtures/diplomacy-slice-2.json
```

All five tapes contain 147 resolved turns. Native and WASM checks verify initial hashes,
every turn, and continuation through save/load at every boundary. The slice-2 native
test also asserts that the recorded sequence actually contains offers, notice, debt
and a breach. [The 1,000-game report](../../../../docs/sim-results/2026-09-27-diplomacy-slice-2.md)
records the complete development cohort and remaining balance failure.

## Diplomacy integration with current master (2026-09-27)

All six tapes were intentionally regenerated after combining diplomacy slice 2
with persistent research queues, the full tech tree, game speeds and the archive.
The three mixed-AI tapes retain exactly the slice-2 branch's 120 order maps;
their initial and every-turn hashes change because the combined player state
serializes `research_queue`. Compared with pre-merge master, the initial hashes
and the first turn's orders/hashes change in all three games, reflecting the
new Peace start and diplomacy decisions.

The three hand-authored examples were also rerun: research queues (120 turns),
peace boundaries (12 turns), and pact/notice/debt/breach boundaries (15 turns).
Their order sequences remain unchanged. Together with the mixed-AI tapes, the
six fixtures contain 267 turns. Native replay and the directory-wide WASM tape
check verify the initial state, every turn, and save/load at every boundary.
The original-three compatibility fixtures remain frozen. A final regeneration after
the closed-territory healing-forecast correction was byte-identical for all six tapes.

Commands (run from the repository root):

```sh
cargo build --release -p legatum-sim
./target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
cargo run --release -p legatum-wasm --example research_queue_tape > crates/sim/tests/fixtures/research-queue.json
cargo run --release -p legatum-wasm --example diplomacy_tape -- crates/sim/tests/fixtures/diplomacy-slice-1.json
cargo run --release -p legatum-wasm --example diplomacy_slice_2_tape -- crates/sim/tests/fixtures/diplomacy-slice-2.json
```


## Production queues and independent purchases (2026-09-27)

Cities serialize an empty-default production queue and purchase-turn marker.
BuyItem now carries the requested item, pays its full faction- and speed-adjusted
cost, and completes before movement without spending active production. A purchase
needs population and spawn space immediately; one per city per turn remains.
Queued construction advances automatically with one natural completion per turn.

The documented three-game regeneration plus the research-queue example were run
intentionally after merging master research queues and combat AI. City fields change
hashes once founded; full-price buying and the merged combat decisions can change
orders. Compared with merged master `80111d4`, the three initial hashes are unchanged;
all first turn hashes change. The first changed order maps are turns 8, 7 and 5,
respectively. Frozen original-three compatibility fixtures were not rewritten.

Older core state saves default the added city fields, and legacy funded production
is honored at its recorded price. Historical tapes with item-less `BuyItem` orders
are not replayed under the new rules. The existing archive loader still requires a
matching rules fingerprint; this is not an archive migration.

The added 150-turn Quick production tape uses an above-JavaScript-precision seed,
queues Market then Monument, buys a Soldier while the Market keeps progressing,
and verifies both queued completions. Every boundary is replayed through save/load.
Regenerate it with:

```sh
cargo run --release -p legatum-wasm --example production_queue_tape > crates/sim/tests/fixtures/production-queue-buy.json
```

Native determinism and the WASM directory tape check cover all five fixtures (390 turns).
## Friendly movement and contested captures (2026-09-27)

F8 changes same-owner traffic: automatic routes avoid reserved intermediate
steps, and colliding own units take turns by unit id. F12 makes hostile arrivals
at an ungarrisoned 0-HP city fight meeting engagements, and permits capture after
killing its garrison. Combat strengths and rolls retain the existing shared math.
`MoveUnitPath` preserves the confirmed client route across turns; its remaining
path is private to its owner, defaults to empty in old saves, and is omitted when
empty, preserving hashes for unchanged states.

The isolated three-game regeneration in `dbb116f` was run intentionally. Initial
hashes are unchanged. Against the pre-fix `9419e00` tapes, first changed hashes are turns
9, 2 and 1 for seeds MAX-2, MAX-1 and MAX respectively (32, 39 and 40 changed
hashes). The submitted orders at each first changed turn are identical; subsequent
AI orders first change on the following turn as movement produces a different
state. The research-queue tape was regenerated and remains byte-identical.

A new eight-turn fixture also exercises `MoveUnitPath`, including nonempty
remaining routes at a save boundary and arrival with an empty route. Regenerate:

```sh
cargo run --release -p legatum-wasm --example movement_path_tape > crates/sim/tests/fixtures/movement-path.json
```

The native debug/release tests and directory-wide WASM check cover all five tapes
(248 turns), including save/load at every boundary. Exact F8/F12 constructed
scenarios also run through the real WASM API in `app/src/movement.wasm.test.ts`.

After merging master through `e77e7a8`, the documented three-game regeneration
was run again using the final combined executable. Initial hashes still match.
Against the isolated movement tapes in `dbb116f`, AI orders and hashes first
change on turns 8, 7 and 5 (33, 34 and 36 hash changes); these additional changes
come from the merged tactical/scout decision rules. The final committed tapes
contain those integrated orders. Against the original `9419e00` baseline, their
first different hashes are turns 8, 2 and 1.

## Capital Science (2026-09-27)

D1 adds an unscaled +2 Science to each empire's current capital, beginning on
the founding turn. Capture clears capital status; the next founded city becomes
the replacement. The three-game command above and `research_queue_tape` were
intentionally rerun. All four initial hashes are unchanged; all 240 resolved-turn
hashes change from turn 1, when capitals first earn Science. The mixed-AI tapes
first change orders on turns 8, 6 and 5 respectively as earlier research becomes
affordable. The 120-turn queue tape retains every recorded order. Frozen
original-three compatibility fixtures remain untouched.
## Structured and varied ruins (2026-09-27)

Ruin rolls now use 24 slots: 8 Gold, 8 Science, 2 population, 3 map caches
(radius 4, cartography only), and 3 free Scouts. Byzantium still doubles only
Gold/Science; Athens still always receive 30 Science. The reward event
now has a typed payload with exact amounts and consequences. It is team-private;
seeing a rival's finder is no longer permission to read its hidden city's growth,
research or Buy quote. The Fragments codex is not part of this change.

The documented three-tape regeneration and `research_queue_tape` example were
run intentionally. All initial hashes are unchanged. The first changed hash is
turn 1 in all three opening tapes; all 120 turn hashes change. Their first ruin
finds now resolve against the expanded table. The 120-turn research-queue tape
is byte-identical. Native and WASM checks still verify every save/load boundary.

## Review fix: safe ruin Scout placement (2026-09-27)

The celebration branch is integrated with current master, including capital Science.
Free Scouts now spawn on the first legal adjacent tile in hex order (never the finder's
tile); a blocked ring gives the normal faction-adjusted Gold reward instead. This
intentionally changes movement occupancy, sight, later orders, and saved-state hashes.
The documented mixed-brain three-tape command and `research_queue_tape` example are
rerun for this integration. Historical simulation/balance reports stay historical;
they are not evidence for the revised spawn rule or subsequent master AI changes.

After the final integration through master `625eff5`, the three mixed-brain tapes
were regenerated again with the combined movement, threat-memory and ruin rules.
The research-queue and movement-path examples were also rerun and remain unchanged.
The five current tapes cover 248 turns, including save/load at every boundary.


## Diplomacy integration follow-up (2026-09-27)

The six fixtures were intentionally regenerated with the commands in the preceding
integration section after merging current master and applying the shared 70% pact
cutoff, 60%/20-turn proposal re-entry rule, and no honor bonus for notice ending a
term at its minimum. Notice history survives save/load. Capital Science from master
also changes the mixed-AI and research-queue turn hashes.

Relative to the integrated baseline `355c96e`, all six initial hashes are unchanged.
The three mixed-AI fixtures first change orders on turns 9, 6, and 9 respectively;
their hashes change from turn 1. The research-queue tape keeps all 120 order maps
and changes hashes from turn 1. The 12-turn peace tape is byte-identical. The
15-turn pact tape keeps its orders and first changes hash at turn 3, when the
public notice is recorded for hysteresis. Historical version-2 tapes are now
explicitly rejected with a regeneration message, rather than interpreted as v3.

The regeneration commands were repeated after merging master through `4de3655`,
including auto-explore fortification, scout/garrison tactics, strategy metrics,
and the optional Proxy benchmark.
All six tapes were byte-identical to the fixtures above. A separate 20-turn
auto-explore tape was also generated for native/WASM and save/load verification.

## Movement review fixes and current master (2026-09-27)

The documented three-game regeneration command was rerun after integrating master
through `4de3655` and the confirmed-route/city-entry fixes. The old fixed-order
replays first diverged on turns 9, 2 and 1 (MAX-2, MAX-1, MAX), exercising the
movement changes. Regenerating with the integrated AI leaves all initial hashes
unchanged; the new tapes first change hashes on turns 3, 2 and 1, and orders on
turns 3, 3 and 2. Respectively 38, 39 and 40 turn hashes change. The research-queue
and confirmed-route tapes are unchanged. All five tapes still cover 248 turns,
including an independent save/load replay at every boundary.

Additional native and WASM scenarios cover blocked confirmed routes, known
hostile route rejection, non-capturing ranged entrants, simultaneous garrison kills
and ruined-city meetings, friendly chains/cycles, owner-local retries, and city
capture by teammates. Team capture ranks survivors by HP; ties use seed, turn and
target tile over candidates sorted by approach tile, never unit IDs or seats.

## Diplomacy with movement contention (2026-09-27)

After merging master `2aa0058`, the commands above were rerun, including the
confirmed-route example. Against the diplomacy integration at `4815bc0`, the
three mixed-AI tapes keep their initial hashes and first change hashes on turns
9, 2, and 1; orders first change on turns 10, 3, and 2. Respectively 32, 39, and
40 resolved hashes change under the integrated traffic/capture and retreat rules.
The research, Peace, and Pact tapes are byte-identical. The movement-path tape
keeps all eight order maps from master but intentionally changes from legacy
all-war version 1 to the current Peace-start version 3, changing its initial and
all eight resolved hashes. Seven committed fixtures now cover 275 turns.

## Recent threat observations (2026-09-27)

Saved player state now retains bounded, expiring observations of visible military
units. They are separate from live unit views and inform only scripted settler
route safety. The same observation rules apply to humans and AI.

The three mixed tapes keep every recorded order and initial hash. Only 39, 36 and
38 resolved-turn hashes respectively were refreshed. For all four tapes, replaying
the original orders and clearing **only** `Player.observed_threats` on a clone of
each resulting state reproduces every prior hash. This isolates the entire hash
change to the added observation field, with no movement or combat rule change.
The 120-turn research-queue tape is byte-identical. Frozen original-three
compatibility fixtures and the historical settler regression snapshots stay intact.

Native debug/release and WASM checked replay cover initial state, every turn and
continuation across save/load. See [the investigation](../../../../docs/sim-results/settler-hazards.md)
for the original-engine replay and policy counterfactuals.

After the diplomacy integration merged master `625eff5`, all seven tapes were
regenerated again. Every initial hash and recorded order is unchanged from
`0140fc1`. The mixed-AI tapes first change hashes on turns 2, 5, and 3, with
39, 36, and 38 changed boundaries respectively due to saved observations.
Research, Peace, Pact, and confirmed-route tapes remain byte-identical.
## Production review fixes and refreshed master (2026-09-28)

Regenerated the three opening tapes with the documented command above, and reran
`research_queue_tape`, `movement_path_tape`, and `production_queue_tape`, after
merging master `625eff5` into the production branch. Independent purchases now
carry their item explicitly; the AI retains its personality reserve, forecasts
local defense, and caps military purchases using saved owner-only history.
Temporary population/spawn blockers retain the queued item and let later work run.

Against the merged parent `ca4e183`, five initial hashes are unchanged. Each of
the three opening tapes changes all 40 resolved hashes (the merged master tapes
predate the city queue/purchase fields). MAX-2 first changes orders at turn 27,
and MAX-1 at turn 37: seat 2 no longer buys below its saving reserve. MAX retains
all recorded orders. Research and production tapes retain all orders but change
all 120 and 150 turn hashes respectively after the integrated state/rules changes.
The production example now uses seed `18446744073709551605` (MAX-10), distinct
from the movement tape's MAX-9. Its initial hash changes too; directory replay
export deliberately rejects duplicate seeds. The eight-turn movement tape is
byte-identical. Frozen original-three fixtures and the archived tactical
regression inputs were not rewritten. The settler archive regression verifies
its original pre-queue serialization hash, then checks current-rule save/load;
its old item-less buys are translated from the recorded active build for that
test's counterfactual only.

Native and WASM checks cover these six tapes' 398 turns, including save/load at
every boundary. Use `WASM_PACK_CACHE="$PWD/target/wasm-pack-cache"` when the
system cache is not writable; no build-script change or skipped optimizer is needed.

## Quick endgame pursuit (2026-09-27)

The documented mixed-brain regeneration command was rerun during the Quick
long-game investigation. The three initial hashes are unchanged; the first
different orders/hashes are turns 3, 6 and 4 respectively. Regenerating with the
preserved master executable (`83ff4b1`) yields byte-identical new tapes, so this
refresh catches up with master's existing opening AI decisions rather than
changing them through the new endgame policy. The research-queue tape is unchanged.

Native debug/release determinism tests and the directory-wide WASM tape check
pass. Eighteen full-game candidate tapes also pass initial, every-turn and
save/load WASM checks. See the [Quick endgame report](../../../../docs/sim-results/2026-09-27-quick-long-games.md)
for before/after cohorts, outlier diagnoses, provenance and remaining limitations.

After the diplomacy branch integrated the endgame changes and campaign threat
memory through master `fcdb97d`, all seven committed fixtures were regenerated
with their documented generators. They are byte-identical to the previous
integrated fixtures: initial states, recorded orders and every resolved hash.

After merging structured ruins and celebrations from master `ebad737`, the same
generators were rerun against the combined diplomacy rules. All seven initial
hashes remain unchanged. The mixed-AI tapes first change recorded orders on turns
2, 5 and 2; all 40 boundary hashes change in each tape under the new ruin reward
stream, Scout gifts and cartography consequences. Research, Peace, Pact and
confirmed-route tapes remain byte-identical. The shared gift/production spawn
helper retains diplomacy's closed-border and foreign-occupant restrictions.
The production review branch subsequently merged endgame pursuit and campaign
memory through master `fcdb97d`, keeping the forecast/reserve/cap checks for
defensive buys and sharing the cap with safe-city Army-title purchases. The
documented regeneration was repeated. Against `7492068`, the MAX tape first
changes at turn 31: Celts seat 4 moves unit 25 toward `(20,14)` rather than
`(20,15)`. Its ten remaining hashes change; the other five tapes are byte-identical.
The campaign's frozen turn-86 snapshot is also retained verbatim; its provenance
hash uses the pre-queue serialization, while its counterfactual tests compare
current-rule uninterrupted and save/load continuations.

The final review integration merges ruin rewards and celebrations through master
`ebad737`. All six tapes were regenerated with the same documented commands.
Against `01b2dd0`, all initial hashes and the research, movement, and production
tapes are unchanged. Each opening tape changes all 40 resolved hashes under the
new ruin reward rules; the first changed orders are recorded in the review report.
Frozen historical snapshots remain unchanged.

## 2026-09-27 (ruin Scout rests)
Regenerated the three opening fixtures with the documented command after the free ruin Scout became `acted` on its birth turn (no same-turn fortification).

## Explicit standing orders (2026-09-27)

Units now save an optional `standing_order` (`Fortify` or `Heal`). It is omitted
when absent, and old saves default to awake regardless of their resting defense.
`HoldUnit` explicitly persists Fortify; Skip and Wake clear it, and Heal clears
itself with a notice at full HP. Resting defense and healing amounts are unchanged.

The three mixed-AI tapes retain all recorded orders and initial hashes. All 120
resolved-turn hashes change because those orders include explicit Holds. Clearing
only `standing_order` on a clone of each resolved state reproduces every previous
hash. The research-queue and movement-path tapes remain byte-identical. The one-off
guarded refresh used for that serialization migration checked
the isolation before writing any tape, then checked uninterrupted and save/load
replay. That migration example has since been removed; use the three-game
regeneration command at the top of this file for future rules changes.

After merging master through `9195478` (ruin-reward and ruin-Scout regenerations),
master's three tapes were taken and refreshed again with the same guarded example.
The documented regeneration command on the merged branch produces the same orders
as on master, so this branch changes no AI decision, only hashes.

## Endgame planning corrections (2026-09-27)

The documented mixed-brain regeneration command was rerun against the endgame
limit/capture corrections based on master `fcdb97d`. MAX-2 and MAX-1 are
byte-identical. MAX retains its initial hash but changes four turns of orders and
all ten hashes from turn 31 onward. Its first change is player 4's unit 25 moving
to `(20, 14)` instead of `(20, 15)`, following master's campaign-memory change.

The frozen `fcdb97d` master executable independently generates exactly the same
three tapes as the candidate. This refresh therefore catches up with existing
master AI behavior; these endgame corrections do not change the opening tapes.
The research-queue and movement-path fixtures are unchanged. The paired Quick
comparison and verification results are in the
[endgame corrections report](../../../../docs/sim-results/2026-09-27-endgame-planning-fixes.md).

The diplomacy integration regenerated all seven tapes again after merging master
`ca7e3f7`. All initial hashes and recorded orders are unchanged. The mixed tapes
change respectively three, one and one boundary hashes (first at turns 1, 4 and
7) under the newborn Scout's no-rest rule. The other four tapes are byte-identical.

After integrating master `b1590a0` (explicit standing orders, turn controls,
defense suitability and coordinated staging), the guarded standing-order refresh
passes on all seven diplomacy-integrated tapes. Clearing only the new field
reproduces every old mixed-tape boundary hash. Full regeneration then preserves
all initial hashes and recorded orders; only those 120 standing-order hashes
change. The other four tapes remain byte-identical.
## Production queue merge with master `ca7e3f7` (2026-09-28)

Merging the production-queue branch with master's ruin-Scout rest and shared
building-limit changes conflicted on the three opening tapes. Master's tapes were
taken and every fixture was regenerated with the documented commands above. All
initial hashes are unchanged. Against master, every opening tape's hashes change
from turn 1 (the city queue fields are serialized once a city exists); the first
changed orders are turns 28, 37 and 23 (full-price buying). Against the branch, the
recorded orders are identical and only hashes change (from turns 1, 4 and 23) under
master's ruin-Scout rule. The research, production and movement tapes are unchanged.

A second merge with master `78d81d1` (explicit standing orders, defense AI)
conflicted on the same three tapes; master's were taken and the documented
three-game command was rerun. Master's own committed tapes are not what that
command produces on master: master's executable first chooses different orders
at turns 8, 9 and 10 (its later defense-AI changes were refreshed with the
order-preserving `standing_orders_tapes` example). Against master's executable,
the merged tapes have identical orders for MAX-2 and first differ at turns 31
(MAX-1) and 23 (MAX), the full-price purchase change; every hash differs from
turn 1 because cities serialize their queue fields. The research, production and
movement tapes are unchanged.

## Wake on nearby visible enemies (2026-09-28)

Fortify/Sleep now ends with `needs_orders` and a team-private notice when an enemy
military unit is in current shared sight within the sleeper's maximum movement
radius. Each movement round is checked, including enemies that pass out of sight
before the turn ends. Heal retains its separate full-health completion.

The documented three-game regeneration was rerun. Initial hashes are unchanged;
the research-queue and movement-path tapes remain byte-identical. Replaying the
previously committed orders first changes state on turns 2, 6 and 15 (MAX-2,
MAX-1, MAX). At each first divergence, restoring only the awakened unit's
`standing_order` and `needs_orders` fields reproduces the exact old hash.

The unchanged starting commit `78d81d1911c3` was independently regenerated from
an isolated source archive. Its AI orders already differ from the committed tapes starting
on turns 8, 9 and 10. The refreshed candidate and starting-commit tapes have
**identical orders on all 120 turns**; the order changes in these fixtures are
already present in the starting commit. Compared with that regenerated baseline,
wake state changes 27, 17 and 21 hashes,
first on turns 2, 6 and 10. The five tapes retain 248 turns of native/WASM replay
and continuation across every save/load boundary.

Merging this branch with master `0c7c032` (production queues and buying) conflicted on
the three opening tapes. They were regenerated with the documented three-game command
on the merged tree. Every recorded order is identical to master's tapes and all initial
hashes are unchanged; only hashes differ, first on turns 2, 6 and 10, from the woken units'
`standing_order` and `needs_orders`. The research, production and movement tapes are unchanged.

## AI macro regeneration (2026-09-28)

The scripted policy now schedules growth, production and science investment, commits
settlers to their chosen site, buys surplus army with idle gold, renews its optional
military purchase allowance over time, and (Hard only) expands to four cities before
developing and reacts earlier to the public VP leader. The three tapes were regenerated
intentionally with the documented command after merging master `d223d1e`. No rules or
serialization changes are part of this patch; `PROXY_VERSION = 2` is unchanged and these
mixed scripted/random fixtures contain no Proxy.

All three initial hashes are unchanged. Compared with master's executable output (not its
order-preserved committed tapes, see the 2026-09-28 learnings entry), the first changed
orders are:

- Seed `18446744073709551613`, turn 10: seat 2 researches Soil Reclamation instead of Fortification.
- Seed `18446744073709551614`, turn 21: seat 6 builds a Ranger instead of a Granary in the exposed city.
- Seed `18446744073709551615`, turn 18: seat 4 researches Fabrication instead of Salvage Protocols.

Each tape retains all 40 turns; native release and WASM replay (initial, per-turn and save/load hashes) pass.

## Fortify holds; Sleep wakes on new contacts (2026-09-28)

This supersedes the nearby-enemy wake rule above. Military Fortify never auto-wakes.
Sleep is a distinct standing order for military and civilian units; civilian Hold
orders now serialize as Sleep. Sleep compares visible military contacts between
resolution phases and wakes once when an enemy military unit newly comes into sight
within its movement radius, by arriving or by walking closer. Re-sleeping beside a
visible enemy does not immediately wake the unit. The baseline comes
from the current state on each turn, including after save/load; it adds no saved
contact-history fields.

The documented three-game command was rerun on master `2d9a076` plus this change.
Master's committed tapes were already not what master's executable produces (its
orders first differ on turns 3, 6 and 4), so the comparison is against master's
executable run with the same command: all initial hashes and all 120 turns of
recorded orders are identical, and 27, 18 and 19 hashes change (MAX-2, MAX-1, MAX),
first at turn 1, from the civilian Sleep state and the revised wake behavior. The
research, production and movement tapes are unchanged.

## Final diplomacy integration sync (2026-09-28)

Merged master `0c7c032`, including production queues, independent purchases,
Proxy v2, wound-aware incoming damage and celebration polish. All eight fixtures
were regenerated with the commands above. Initial hashes remain unchanged for
the seven existing diplomacy fixtures. MAX-2 and MAX-1 keep every order; MAX
changes five order maps starting at turn 23 under the new purchase contract.
All 120 mixed-tape and 120 research-tape boundary hashes change with city
production-queue serialization. The production example now uses the current
version-3 diplomacy format. Eight committed tapes cover 425 turns; a separate
20-turn auto-explore tape brings native/WASM coverage to 445 turns.

After the final master sync at `8ec4bc4` (shared settlement scoring and map lenses),
all eight fixtures were regenerated once more with the same commands. They are
byte-identical to the production-sync fixtures: initial hashes, recorded orders
and all 425 committed boundaries are unchanged. The separate automation replay
also remains covered. The final 500-game campaign is rerun because the settlement
planner changes behavior beyond these short recorded scenarios.

## Diplomacy merge with wake-on-sight (2026-09-28)

Merged master `b15ab47` (wake sleeping units near visible enemies, scout
reconnaissance safety, blocked production queues). Wake-on-sight now counts only
military units of teams at War with the sleeper's team, so Peace and pact
neighbours never wake sleepers (core `wake_threatened_sleepers` and the mock).
The three mixed tapes were taken from master and regenerated with the documented
command. Against master they differ from turn 1 because new games start at Peace.
Against the previous diplomacy branch tapes, MAX-1 and MAX are byte-identical and
MAX-2 keeps its initial hash and first changes orders and hashes on turn 32. The
research-queue, both diplomacy, production and movement-path examples were rerun
and are byte-identical.

## Diplomacy merge with AI macro (2026-09-28)

Merged master `b89d4e5` (AI macro: scheduled investment, committed settlers, idle-gold
army purchases, Hard's earlier VP-leader reaction). The mixed tapes were taken from
master and regenerated with the documented command. All initial hashes match the
previous diplomacy tapes; the first changed orders are on turns 10, 21 and 17 (MAX-2,
MAX-1, MAX), in line with master's own AI-macro refresh (turns 10, 21 and 18). The
other five fixtures are byte-identical after rerunning their examples.

## Diplomacy merge with Fortify/Sleep split (2026-09-28)

Merged master `30675f6` (Fortify holds; Sleep wakes once per enemy newly in range;
garrison hold). Sleep threats count only military units of teams at War with the
sleeper (core `sleeping_threats` and the mock). The mixed tapes were taken from master
and regenerated with the documented command: initial hashes and every recorded order
match the previous diplomacy tapes; 5, 2 and 6 boundary hashes change (first on turn 1)
from the new standing-order state. The other five fixtures are byte-identical.

## AI offer pacing (2026-09-28)

AIs no longer open with pacts: a first pact offer waits for the acquaintance period
(15 turns at Standard, scaled by speed), and AI offers to a team with a human are
staggered to one new offer per turn. The three mixed tapes were regenerated with the
documented command. Initial hashes are unchanged; recorded orders first change on turn 1
in all three tapes (the turn-1 pact offers are gone), so every boundary hash changes.
The other five fixtures are byte-identical after rerunning their examples.


## 2026-09-28: alliances and explicit coalition calls

`diplomacy-alliance.json` adds an 18-turn version-4 tape covering direct alliances,
one-hop shared sight, pending and accepted calls, explicit abandonment, public notice,
Influence debt and a 50-Influence alliance breach. It is generated by:

```sh
cargo run --release -p legatum-wasm --example alliance_tape -- crates/sim/tests/fixtures/diplomacy-alliance.json
```

The eight existing version-3 fixtures are unchanged. Empty alliance/call metadata is
omitted from canonical serialization, preserving their hashes. Version-1 all-war saves
and tapes retain their old diplomacy mode; version-3 saves gain empty optional fields.
Version-2 tape rejection remains explicit. Saves and newly recorded tapes now use version 4.
Native and WASM replay cover nine tapes and 443 turns, including a save/load at every boundary.
The alliance tape is an engine contract fixture, not evidence of AI coalition effectiveness;
that is measured by `coalition_bench` and the paired campaign report.

## Blocked moves stop after two turns (2026-09-28)

`Unit.blocked_turns` counts consecutive turns a move made no progress because its
next step was blocked; at 2 the move stops (MoveBlocked naming the blocker) and the
unit asks for orders. The field is omitted from JSON at 0, so initial hashes are
unchanged. The documented three-game regeneration was run intentionally: all 120
recorded order maps are identical; seed …614 changes hashes from turn 32 and seed
…615 from turn 8 (the first turn a unit ends blocked); seed …613 is unchanged.

## 2026-09-28: alliance observation boundary repair

Regenerated only `diplomacy-alliance.json` with the documented `alliance_tape`
command above. The initial state and all 18 order maps are unchanged; only the
turn-2 hash changes. Alliance activation now records newly borrowed observations
at that boundary, instead of waiting for the following resolution. Native
exploration credit remains separate. The other eight tapes are unchanged.

## Coach settler commitment deadline (2026-09-28)

Units now persist their creation turn so an unfounded settler's speed-scaled deadline
survives save/load and reconstruction of the stateless AI. Starting and legacy units
use turn one, which is omitted from serialization. Only the owner sees this timestamp.
This adds decision metadata; movement, combat, yields, costs and enemy visibility are
unchanged.

The documented mixed-tape and six example generators were rerun after the coalition
merge. All nine initial hashes and the order lists for all 443 recorded turns are
unchanged. The three mixed tapes first change hash at turns **8, 4 and 7**, respectively;
the production-queue tape first changes at turn **81**. These are unit creations after
turn one. The other five tapes retain their original contents and versions. The four
updated tapes use the current version-4 generator; unchanged version-3 tapes remain as
compatibility coverage. An independent replay audit restored only each unit's creation
turn to one before hashing and reproduced **every original hash**, including all 174
changed ones.
The fixtures therefore reflect understood serialization changes, not unexplained drift.
After the final master merge and four-proposal evaluation, the frozen final executable
reran the mixed-brain command and reproduced all three committed tapes exactly. The
six manual example generators do not invoke the AI.

The native and WASM checks above verify every regenerated hash and save/load boundary.
Benchmark commands, results and validation receipts are in
[the coach proposal report](../../../../docs/sim-results/2026-09-28-coach-proposals.md).

## Promotions (2026-09-28)

Units now carry combat experience (`Unit.xp`, omitted from JSON at 0) and fights
grant it (`rules::promotion`). The documented mixed-brain command was rerun with the
branch executable and reproduced all three committed tapes byte for byte, as did the
master executable at `56521b1`: those 40-turn games start at Peace and never fight, so
they cannot cover this rule. The other seven tapes replay unchanged for the same
reason.

A new twelve-turn tape covers it: two empires at War (`legacy_all_war`) march their
Soldiers into each other until one reaches level 1 (turn 9), then hold.
The native test replays it with save/load at every boundary and checks the promotion
and the promoted unit's level at the end; the directory-wide WASM check covers it too.

```sh
cargo run --release -p legatum-wasm --example promotion_tape > crates/sim/tests/fixtures/promotion.json
```

## Attack pursuit (G2, 2026-09-28)

A melee attack order now follows its target unit (`Unit::attack_target`). The
promotion tape's Soldiers are ordered onto each other's tiles, so they chase a
target that moves instead of walking into the vacated tile, and the tape's hashes
change from turn 8. It was regenerated with its documented command (still 2
battles, first promotion on turn 9):

```sh
cargo run --release -p legatum-wasm --example promotion_tape > crates/sim/tests/fixtures/promotion.json
```

The other nine tapes replay unchanged: their games either start at Peace and never
fight, or never order an attack onto a unit that moves.

## Map resources, G11 increment 1 (2026-09-29)

Maps now carry resources (Wheat, Cattle, Stone, Copper, Iron, Coal) placed by a seeded
mapgen layer after the starts, `Tile.resource` is serialized when present, and players
serialize strategic stockpiles and paid builds when non-empty. Every initial hash changes
because every generated map now has resources. All nine committed tapes were regenerated
intentionally with the documented mixed-brain command and the seven example generators
(research queue, production queue, movement path, promotion, diplomacy slices 1 and 2,
alliance). The seven hand-authored example tapes keep every recorded order; only hashes
change. The mixed-AI tapes first change orders on turns 17, 8 and 15 (seeds MAX-2, MAX-1,
MAX): resource yields change worked tiles, border growth and settle scores, and Troopers
and Siege Engines now need Iron and Coal.

## G2 + G11 merge (2026-09-29)

After merging attack pursuit (G2) and map resources (G11), every tape was regenerated again
from the merged rules with the commands above, so both changes are in all ten tapes.

## Barbarians (G10, 2026-09-29)

New games now default to barbarian camps (`GameSetup.barbarians`), and the three
mixed-brain tapes use the default, so they now record barbarian camps, scouts,
raids and fights: 4 camps and 1 or 2 raids each within the 40 Standard turns.
Their setups serialize `"barbarians": true`, so all three initial hashes and all
120 turn hashes change. Every command in this file was rerun. The scripted
scenario tapes (research queue, diplomacy slices, alliance, production queue,
movement path, promotion, auto-explore) set `barbarians = false` in their
examples. They came out byte-identical, so they keep testing only their own
feature. Saves and setups from before barbarians have no field and load with
barbarians off. Native debug and release and the WASM tape check replay all
fixtures, including every-turn save/load.

## One attack per order, surprise war, strike then retreat (G18, G20, G21, 2026-09-29)

Three combat-timing rules changed together. G18: an attack order is one attack; a unit
whose fight leaves the other side standing drops its destination and attack target, and a
pursuit ends when the turn's movement is over. G20: a declaration at plain Peace (no Pact or
Alliance, past any guaranteed-Peace turns) starts War at the start of that turn's resolution.
G21: a unit attacked while stepping away takes the hit without fortification and completes
its step if it survives, instead of being pinned. Every command in this file was rerun.
Initial hashes are unchanged. Six tapes changed: the three mixed-brain games (first hash
change on turns 18, 14 and 24; first different orders on turns 19, 20 and 30), the promotion
tape (from turn 8; still 2 battles and the first promotion on turn 9), diplomacy slice 1
(from turn 1: its declaration now takes effect in the turn it is ordered) and the alliance
tape (from turn 3). Recorded orders in the hand-authored tapes are unchanged. Research queue,
diplomacy slice 2, production queue and movement path came out byte-identical.

## Cities fire back by default; Hard v2 (2026-09-30)

City strike (from the opening league's rule levers) is now a default rule: in each ranged
phase every city with HP above 0 fires once at the lowest-HP visible hostile military unit
within two tiles (`no_city_strike` in `rule_experiments` opts out). Every command in this
file was rerun. Initial hashes are unchanged. Only the three mixed-brain games changed
(first hash change on turns 27, 14 and 15; first different orders on turns 40, 24 and 23);
the seven scripted scenario tapes came out byte-identical. The mixed-brain tapes use Normal
scripted seats, so they don't exercise Hard v2 (Hard seats now play a league-tuned opening,
see `crates/ai/src/hard_v2.rs`); tapes replay recorded orders, so no AI change can alter a
committed tape by itself.

## Re-land V9 on current master (2026-10-01)

Re-landed `b25bbbe` on `11db0da`, retaining current diplomacy, resources, promotions,
barbarians and default city strike. V9 intentionally changes faction costs, yields,
healing, goal awards and the included faction-planning policies. The mixed-brain
generator above was rerun with explicit `--difficulty normal --threads 4`; these
engine fixtures retain their original difficulty, independently of the separate
scripted **Hard** balance campaign. All ten setups and initial hashes are unchanged.

| Tape | Changed turn hashes | First changed hash | Changed order maps | First changed orders |
| --- | ---: | ---: | ---: | ---: |
| MAX-2 (40 turns) | 40 | 1 | 15 | 24 |
| MAX-1 (40 turns) | 30 | 4 | 0 | none |
| MAX (40 turns) | 40 | 1 | 20 | 15 |

These three tapes require new expectations because they exercise the changed V9
factions: two diverge in orders as the changed rules feed back into planning, while
MAX-1 retains every order and changes only resolution. The seven manual generators
were also rerun: research queue (120 turns), diplomacy slice 1 (12), diplomacy slice
2 (15), production queue (150), movement path (8), promotion (12), and alliance
(18). All seven produced byte-identical files and therefore are not re-recorded in
this change. Historical save/archive fixtures remain frozen.

The three 80-turn mixed-brain golden tape digests in `runner.rs` were also re-pinned.
The frozen master executable independently reproduces all three old digests, and
the frozen V9 executable reproduces all three new ones. These pin full tape JSON
(including AI orders), so the same intentional V9 rule/planning changes require the
update. Both sets of tapes and exact commands are retained with the report artifacts.

Native replay and the WASM directory check cover all ten tapes, 455 turns, and
save/load at every boundary. The rules fingerprint changes with the engine source;
it is not a migration of old archives. Exact generation commands, before/after
fixture comparison, validation receipts and matched Hard balance results are
recorded with [the re-land report](../../../../docs/sim-results/2026-10-01-reland-balance-v9.md).

## AI city-fire awareness, original pre-V9 experiment (2026-10-01)

Historical receipt from `e6a961d`; superseded by the current-master revision below.

Default scripted AI now budgets hostile cities' ranged shots in danger, recovery,
scouting and assault planning. This applies to Easy, Normal, Hard and every opening
policy. The versioned Proxy remains a fixed benchmark. City fire resolves before
movement; a city defeated by a simultaneous volley still fires. Coordinated melee
now receives credit for committed ranged support, without spilling damage through
a killed garrison into its city. All damage comes from core forecasts/combat functions.

The documented three-game 40-turn command and the same command with `--turn-limit 80`
were both rerun on the final implementation (one worker). Every initial hash, order
and state hash is unchanged for each of these recordings:

| Seed | 40-turn tape | 80-turn golden FNV-1a |
| --- | --- | ---: |
| 18446744073709551613 | Byte-identical | 15030415405075546679 (unchanged) |
| 18446744073709551614 | Byte-identical | 6699008936867188249 (unchanged) |
| 18446744073709551615 | Byte-identical | 7820597218741664499 (unchanged) |

These opening recordings do not exercise a decision changed by this policy within
their recorded horizon. Keeping their hashes is intentional, not a skipped re-record.
Their original versions and regenerated reports are under `/tmp/ai-city-strike/`.
The seven hand-authored feature tapes need no regeneration because combat resolution
and serialization are unchanged; all are replayed natively and in WASM.

`city-strike-awareness.json` adds a 160-turn Standard recording with eight scripted
Hard players, seed 3200001, default rules and barbarians. It uses no `--hard-v2` or
opening switch: this is the scripted Hard the app uses. The matching native golden
in `runner.rs` also regenerates the AI orders instead of merely replaying them.

```sh
nice -n 10 target/release/legatum-sim --games 1 --seed 3200001 --turn-limit 160 \
  --difficulty hard --brain scripted --players 8 --threads 1 --label hard160 \
  --tape-out /tmp/ai-city-strike/final/hard160 --out-dir /tmp/ai-city-strike/final
cp /tmp/ai-city-strike/final/hard160/game-00000-seed-3200001.json \
  crates/sim/tests/fixtures/city-strike-awareness.json
```

On the original AI the same recording hashes to `8646551925739195594`; the final
recording hashes to `11218452013502579654`. The initial state is identical. Orders
and state first differ at turn 105: player 3's Soldier 83 follows Ranger 86's
committed volley from (18,7) toward (21,4), instead of holding. The longer recording
also reaches city fire; it records one player-owned unit and 36 barbarians destroyed
in phases with city fire. This fixture complements the focused city-only, wounded
retreat, one-shot targeting, staging and volley/capture tests.

The replay-export regression expects eleven fixture tapes plus its separately
labeled single export. Native and WASM replay check all eleven tapes (615 turns),
including every save/load boundary. The comparison receipt is
[`2026-10-01-ai-city-strike-awareness.md`](../../../../docs/sim-results/2026-10-01-ai-city-strike-awareness.md).

## Coordinated city assaults on current master (2026-10-01)

This revision starts at `a9f6224` (V9, real Rush, and opening identity fixes) and
cherry-picks `e6a961d`. The two receipt sections above describe their original
baselines; the pre-V9 city-fire hashes are historical, not current expectations.
No resolution, map, setup, faction, or serialization rule changes in this revision.

The scripted planner now reserves a viable combined city assault before recovery.
It forecasts ranged suppression, one city shot per city, visible supporting
attackers and sequential melee attrition over at most four turns, including the
approach. Surviving wounded participants stay committed when capture is forecast.
Movement reconciliation preserves the group and treats reaching combat as a
successful approach, rather than requiring occupation before combat resolves.
Units outside a viable assault still avoid fire and recover. An assembled army
can buy a survivable sighting from range two rather than remain blind at range three.

All generators were rerun against the final release executable, with an independent
current-master recording for the generated AI tapes:

| Recording | Current-master comparison | Final FNV-1a of tape JSON |
| --- | --- | ---: |
| MAX-2, 40 turns | Byte-identical | 12812114631433145198 |
| MAX-1, 40 turns | Byte-identical | 7631398847740848120 |
| MAX, 40 turns | Byte-identical | 11354600854661930473 |
| MAX-2, 80 turns | Byte-identical | 5639197132889317457 |
| MAX-1, 80 turns | Byte-identical | 5205266159733089792 |
| MAX, 80 turns | Byte-identical | 15256394797693125308 |
| Hard 3200001, 160 turns | First different orders/hash: turn 132 | 16391342214061743277 |

The three 40-turn tapes are mixed scripted Normal/random, as before. The seven
manual scenarios (research queue, both diplomacy slices, production queue,
movement path, promotion and alliance) are also byte-identical. Their setups,
initial hashes and all recorded orders/hashes are unchanged. They do not need
new expectations merely because an AI policy changed.

The Hard tape is re-recorded, not replayed with old orders. Against current master,
its setup and initial hash match; 29 order maps and 29 turn hashes differ. At turn
132, player 3 changes unit 77's destination from `(9,20)` to `(9,19)`. Current
master's tape JSON hashes to `12601280874207566053`. Compared with the cherry-picked
pre-V9 fixture, resolution first differs at turn 1 and orders at turn 25: those
include the already merged V9 changes, which cannot be attributed to this policy.
The initial hash remains the same. The 80-turn golden assertions retain master's
V9 digests; only the longer Hard assertion changes.

Reproduction (the 80-turn recording uses the same first command with `80`):

```sh
./target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 \
  --brain mixed --threads 1 --label fixtures40 --tape-out /tmp/ai-city-strike-v2/final/fixtures40 \
  --out-dir /tmp/ai-city-strike-v2/final/fixture-reports
./target/release/legatum-sim --games 1 --seed 3200001 --turn-limit 160 \
  --difficulty hard --brain scripted --players 8 --threads 1 --label hard160 \
  --tape-out /tmp/ai-city-strike-v2/final/hard160 \
  --out-dir /tmp/ai-city-strike-v2/final/fixture-reports
```

All seven scenario generators use their commands above. The fixtures remain eleven
tapes / 615 turns. Native/WASM replay checks every initial and per-turn hash and
save/load boundary. Detailed paired results and check receipts are in
[the revised assault report](../../../../docs/sim-results/2026-10-01-ai-city-strike-awareness-v2.md).

## Shared faction-goal planning (2026-10-01)

This is an AI-only change on top of V9. Mongols plans an entire reachable,
pairwise-eight-hex trio and its paid Markets/settlers/escorts, including replacement
settlements. Sparta coordinates two Walls, two protected home guards and one
veteran expedition, keeping the route outside known city fire and considering only
legal expedition declarations. The shared planner includes Default scripted seats;
this intentionally changes its order tapes without changing any engine rule/value.

The three 40-turn mixed-brain generators were rerun with explicit
`--difficulty normal --threads 8`. All three are byte-identical: these Standard
openings have not yet made a changed goal investment. Their setups, initial states
and all 120 turn hashes stay unchanged. The manually authored scenario generators
are independent of the AI and also retain their recorded orders and hashes.

The frozen master executable independently reproduced the old 80-turn Default
golden digests. Re-recording the same seeds and options with this policy gives:

| Seed suffix | Changed order maps | First order change | Changed turn hashes | First hash change |
| --- | ---: | ---: | ---: | ---: |
| MAX-2 | 2 | 68 | 13 | 68 |
| MAX-1 | 0 | none | 0 | none |
| MAX | 0 | none | 0 | none |

At MAX-2 turn 68, scripted Sparta seat 2 builds a Granary instead of applying
the old blanket Walls bonus before it has two homes to defend. MAX-1 and MAX
remain byte-identical. Goal guard reservations start only after both Walls are
built or committed; active opening build sequences retain their priority.

The generator commands remain the ones above; the golden run uses `--turn-limit 80`.
Native and WASM replay validation and the separately requested matched **Hard**
600-game comparison are recorded in
[the faction-goal report](../../../../docs/sim-results/2026-10-01-ai-faction-goal-planning.md).
That report retains the old/new 80-turn tapes and the exact commands. Historical
saved-game and archive regression inputs remain frozen.

## Faction-goal v2 integration with city assaults and closing (2026-10-01)

The integration starts at master `b824eb6`, including `11b111a` (opening closing),
`187f165` (coordinated city fire), and `b824eb6` (simultaneous melee and civilian
exposure fixes). It carries goal-planning base `5086beb` and revision `39199cc`.

Both frozen binaries independently regenerated the three 40-turn mixed-brain
fixtures, the same seeds at 80 turns, and the Standard Hard 160-turn city-fire tape.
The setups and initial hashes match in every pair. The commands retain the options
above, with explicit `--difficulty normal` for mixed brains and `--difficulty hard`
for the eight scripted seats.

| Recording | Changed order maps | First changed order/hash | Changed hashes | Combined FNV-1a |
| --- | ---: | --- | ---: | ---: |
| MAX-2, 40 turns | 0 | none | 0 | 12812114631433145198 |
| MAX-1, 40 turns | 0 | none | 0 | 7631398847740848120 |
| MAX, 40 turns | 0 | none | 0 | 11354600854661930473 |
| MAX-2, 80 turns | 2 | 68 | 13 | 15915321984924008312 |
| MAX-1, 80 turns | 0 | none | 0 | 5205266159733089792 |
| MAX, 80 turns | 0 | none | 0 | 15256394797693125308 |
| Hard 3200001, 160 turns | 57 | 84 | 77 | 8031244405549724855 |

At MAX-2 turn 68, Sparta seat 2 builds a Granary instead of applying the old
blanket Walls preference before it has two homes. This retains the v2 golden.
At Hard turn 84, Sparta seat 5 commits city 10 to Walls instead of Workshop,
funding the coordinated home plan. Later orders and states diverge as that paid
goal plan interacts with the unchanged city-assault policy. This is a new AI
recording, not a replay of old orders with substituted hashes. The Hard tape and
its generator assertion are updated together; the three 40-turn fixtures are
byte-identical after regeneration. No engine rule, faction value, or serialization
format changed.

All seven manual scenario generators were also rerun: research queue, both
diplomacy slices, production queue, movement path, promotion, and alliance remain
byte-identical. The eleven committed fixtures still contain 615 turns. The
additional paired 80-turn, full-game, and master Hard recordings bring the native
and WASM cross-target check to 30 recordings / 4,571 turns, with every initial,
per-turn, and save/load hash checked.

The complete comparisons, regenerated longer tapes, full-game goal tapes, matched
600-seed Hard results, and check receipts are in
[the integration report](../../../../docs/sim-results/2026-10-01-ai-faction-goal-planning-v2-rebase.md).


## Hard defence against the human playbook (2026-10-01)

Default scripted Hard now assesses visible neighbouring armies, protects front
positions and settler routes, funds defence, and maintains containment of an
expanding aggressor. The selected candidate 26 requires ranged support before
optional sieges and permits separate peace when visible local pressure overruns
home. Later counterfire, reinforcement, Scout
sight and blocked-production experiments are excluded. Normal policies, the
versioned human-playbook opponents, and engine rules are unchanged.

The frozen parent `0368ff0` and the candidate independently regenerate the three
40-turn mixed-brain tapes, their 80-turn golden counterparts, and the Standard
Hard 160-turn recording. Both Normal sets remain byte-identical. The seven
manual scenario fixtures retain their authored orders; replay checks cover them
along with the regenerated AI fixtures.

For Hard seed `3200001`, the initial hash is unchanged. The first different
order and hash are at turn **35**: Sparta seat 5 holds Settler 36 at `(24, 7)`
instead of routing it through `(23, 7)` toward `(14, 9)`. The actual pre-turn
PlayerView shows player 0's Scout at `(22, 8)`, two tiles away with three moves,
under ordinary Peace. The new policy accounts for visible rival reach when an
ordinary declaration is legal. This is intentional route caution, not changed
movement rules. The remaining decisions and states then diverge.

The Hard recording has **126 changed order maps and 126 changed turn hashes**.
Its FNV-1a digest changes from `8031244405549724855` to
`17683572426304174917`; its SHA-256 is
`103766feb0312114196da3a5ce1304e7f2c5af6557ebfaa15999061f8748510f`.
The generated tape and the generator assertion are updated together. No hash is
substituted into a replay of the old orders.

Commands, paired recordings, validation receipts, and benchmark limitations are
in the [Hard-vs-human report](../../../../docs/sim-results/2026-10-01-hard-vs-human-playbook.md).

## Default G8 pacing and balance bundle D (2026-10-02 decision)

The 2026-10-02 default gave capitals +2 Production, phased the speed surcharge on
Production over the first third of the expected game length, priced ordinary
Byzantium and Celts buildings at 130% and 110%, charged Sparta Settlers +5
base Production, and awarded Germany and Mongols goals 3 VP. The later iron-price
redesign removed the Sparta Settler surcharge. The seven old
flag spellings remain accepted; spelling the chosen default has no effect on
serialized setup or gameplay. See the [pacing study](../../../../docs/sim-results/2026-10-02-pacing-study.md)
and [bundle D study](../../../../docs/sim-results/2026-10-01-balance-v10-iteration.md).

The three 40-turn mixed-brain tapes (`game-00000` through `game-00002`)
retain their initial hash and change from turn 1 as founded capitals and early
prices change. Recorded orders first differ on turns 10, 9 and 10,
respectively. Their matching 80-turn golden recordings were rerun with the
same seed, brain and speed and `--turn-limit 80`; their FNV-1a tape digests in
`runner.rs` are now `8989719149424629528`, `1299835907916899890` and
`17586992540925537863`. The 160-turn scripted Hard
`city-strike-awareness.json` also keeps its initial hash and changes from
turn 1; its first changed order map is turn 10, and its new digest is
`7595470966912950594`. These four tapes were freshly simulated using the
commands at the top of this README and in the city-strike section, with no
`--rules-experiment` arguments.

All seven hand-authored scenario generators were rerun with their commands in
this README. `research-queue.json` and `production-queue-buy.json` change
hashes from turn 1 while retaining every order, because capital output and
Production prices change state. `diplomacy-slice-1.json`,
`diplomacy-slice-2.json`, `diplomacy-alliance.json`, `movement-path.json`, and
`promotion.json` are byte-identical: their scenarios do not exercise the new
opening economy. Their recorded orders, 12/15/18/8/12 turn counts, and
scenario assertions remain intact. Historical compatibility save fixtures in
core and WASM were not rewritten.

All eleven current tapes preserve their turn counts (three × 40, one × 160,
plus 120/150/12/15/18/8/12 scenario turns). Native determinism replays each
initial and turn hash with save/load at every boundary; the WASM directory
check replays the same eleven tapes.

## Swiss and Japan defaults (2026-10-02 decision)

The documented three-game command above and the `hard160` command in the city-strike section
were rerun after merging the current Hard-defense fixes from master. The rules now pay Swiss
interest, charge flat three-Gold paid military upkeep, permit their Great Work purchases and
treasury bounties, and apply the Japan hive bonus and capital-distance penalty. The scripted AI
also hoards for the Swiss goal and a legal Great Work, and rates linked Japan settlement
sites. These are intentional economy and AI changes; historical rules are not replayed.

All four initial hashes are unchanged. The three mixed tapes first change resolved hashes on
turns **8, 4 and 9**, before their first changed order maps on turns **20, 6 and 16**. Their
40-turn recordings change **32, 37 and 32** hashes and **13, 35 and 25** order maps. The
160-turn Hard tape first changes hash on turn **27** and orders on turn **55**, with **134**
changed hashes and **91** changed order maps. These early hash changes follow the new faction
economy even where the AI has not yet picked a different order; later orders and states diverge.

The associated 80-turn mixed-tape FNV-1a digests in `crates/sim/src/runner.rs` are now
`12666474553472207829`, `4411868595215096096` and `13211799777621726028`; the Hard
160-turn digest is `5729075503071875681`. The seven hand-authored scenario tapes are
byte-identical and continue to replay; their orders do not depend on this scripted AI change.
Native determinism checks each initial and per-turn hash, including save/load boundaries.

## Germany industry defaults (2026-10-03)

The three 40-turn mixed-AI tapes were regenerated with the command above after
Germany's unit surcharge became a one-move penalty (minimum one), its cities
gained continuous Gold/Science projects, and the AI changed its defense and
project choices. The matched Hard tuning selected a 90% price for new building
families, +3 Production Foundry and a 2-VP city goal. Initial hashes change in
all three tapes because starting units now have different movement capacity.
Their first changed resolved hash is turn 1; their first changed AI orders are
turns 3, 5 and 2, respectively. These are intentional rule and planning changes.

The fixed-order tapes `city-strike-awareness.json`, `diplomacy-alliance.json`,
`diplomacy-slice-1.json`, and `diplomacy-slice-2.json` keep every recorded order.
Their initial hashes and first resolved hashes change because their initial
Germany unit movement and later faction rules changed. The first changed
resolved hash is turn 1 in each. `movement-path.json`, `promotion.json`,
`production-queue-buy.json`, and `research-queue.json` remain byte-identical.
The changed fixed-order hashes were recalculated with:

```sh
cargo run --release -p legatum-sim --example rehash_fixtures -- crates/sim/tests/fixtures/*.json
```

The helper preserves setups and orders and only rewrites initial and resolved
hashes. Native debug/release and WASM replay checks still verify all fixtures;
old tapes are not silently accepted under the new default rules.

The separate 80-turn mixed AI tape digests in `crates/sim/src/runner.rs` now pin
`86120295859578366`, `7002321918239962847`, and `3891191962037508943`.
The 160-turn scripted Hard city-fire tape pins `9977205506083178724`.

## Mongols courier defaults (2026-10-03)

The three 40-turn mixed tapes and the 160-turn scripted Hard city-fire tape were
re-recorded with the same seed, speed, brain and turn-limit commands documented
above. Initial hashes are unchanged. Foreign-border passage and atomic transit
through occupied tiles change resolution; the scripted scout and military choices
then diverge. This is an intentional rule and AI change, not an unexplained hash
repair. Relative to the preceding Germany fixtures, the three mixed tapes first
change resolved hashes on turns 4, 13 and 10, changing 37, 28 and 30 hashes;
their orders first change on turns 8, 13 and 12, changing 33, 28 and 26 maps.
The Hard tape first changes both hash and orders on turn 8, changing 153 of
160 hashes and order maps. All four keep their turn counts. The seven fixed-order
scenario tapes are unchanged. Native replay and save/load verify every recorded
hash; WASM replay verifies the same tapes.

The default 80-turn mixed AI FNV-1a digests in `crates/sim/src/runner.rs` are
`4286222829803011483`, `9451741975518146157` and `13515488138155506036`.
The Hard 160-turn digest is `7660627626898572822`.

## Five opening policies (2026-10-03)

The three 40-turn mixed scripted/random tapes were re-recorded with the documented
seed and options after changing scripted strategy selection and investment. Their
contents are byte-identical to the Mongols recordings: the affected scripted
choices did not change any order in those three 40-turn games. The 160-turn
scripted Hard tape was independently re-recorded with the command in the
city-strike section and is also byte-identical. The pinned 80-turn mixed and
160-turn Hard tapes in `crates/sim/src/runner.rs` are checked separately;
no expected digest was changed without a matching observed tape.

### Opponent assessment with conquest and walls-only fire (2026-10-03)

Regenerated the three 40-turn mixed tapes and the 160-turn scripted Hard
city-strike-awareness tape from fresh AI orders. All four initial hashes remain
unchanged. The first differences are Fortification research at turn 17/player 4
(seed MAX-2), a pact proposal at turn 16/player 0 (MAX-1), Fortification at turn
6/player 2 (MAX), and Fortification at turn 11/player 7 (Hard seed 3200001).
These reflect public-rival defense priorities and strategy-dependent diplomacy.
The corresponding 80-turn mixed and 160-turn Hard order digests are re-pinned
in runner.rs. Authored rule scenarios are unchanged.

These recordings include the merged conquest and walls-only prerequisites. Relative
to the walls-only tapes, 15, 25 and 26 mixed order maps and 146 Hard order maps
change. The first changed orders described above remain the same. Fresh 80-turn
mixed digests are `9672856429157821512`, `14341698393192657377`, and
`1283429449208512473`; the Hard 160-turn digest is `2080823837244570889`.
The Hard runner also replays its newly recorded tape and verifies every hash.

## Terrain sight and ranged line of sight (2026-10-03)

All eleven tapes have new initial and every-turn hashes because the initial explored
set and later fog memory now use terrain-blocked hex lines. The three 40-turn mixed
AI recordings and the 160-turn scripted Hard recording also have different orders
from turn 1 onward: each seat plans from its changed visible terrain and units.
They were regenerated with the 40-turn command above and the `hard160` command in
the city-strike section, with no experimental flags. Each retains its original
turn count (3 × 40 and 160). The other seven scenario tapes retain every recorded
order and their original 12/15/18/8/150/12/120 turn counts; only their hashes
were recomputed with `cargo run --release -p legatum-sim --example rehash_fixtures --
crates/sim/tests/fixtures/*.json`. This is the expected effect of sight on the
serialized explored set, not an unrelated order edit.

The 80-turn mixed AI tape digests pinned separately in `runner.rs` are
`1659461865273077947`, `13166404625179174380`, and
`837448409075404171`; the 160-turn scripted Hard digest is
`5599458438574053599`. The 80-turn tapes were regenerated from the same three
seeds with `--turn-limit 80`; the Hard digest uses the 160-turn tape above.
Native save/load and WASM replay must match each new initial and turn hash.

## Combined terrain sight and walls-only city fire (2026-10-03)

After merging the walls-only rule into terrain sight, the three mixed 40-turn
tapes and the scripted Hard 160-turn tape were simulated again with the commands
above. Relative to the terrain-only recordings, their initial hashes are
unchanged: the initial terrain and visibility are the same. The first resolved
hash changes at turns 21, 19, 19, and 21 respectively when unwalled city fire
would previously have happened. The first changed AI orders are turns 35, 20,
24, and 29. The seven fixed-order scenario tapes retain their orders and hashes
under the combined rules; `rehash_fixtures` checked them without changes. The
80-turn digests above were also recorded again from the combined simulator.
The final settled simulator includes the shared LOS check for barbarian raiders
and trims scripted confirmed routes before a newly seen hostile. Rebuilding it
after those edits left the three 40-turn mixed recordings unchanged, but changed
the first 80-turn mixed digest; the other two 80-turn digests stayed the same.
Against the previous terrain-and-walls recording, the Hard 160-turn tape keeps
its initial hash and changes seven resolved hashes and six order maps beginning
at turn 154. Its digest above agrees with the pinned runner test. The seven
fixed-order scenarios were rehashed again against that exact simulator.
## Swiss and Japan tuning (2026-10-03)

Japan now receives its 5% nearby-city network bonus on Food and Production only,
and the Japan seat takes the most land-rich start among the already picked,
quality-banded starts. Swiss interest is capped at 5 per turn, and its
scripted AI spends beyond a defensive reserve after earning its treasury goal,
while saving a quoted Great Work price once that purchase becomes legal. The
three 40-turn mixed tapes were recorded again with the command near the top of
this file. All three initial hashes and all 40 resolved hashes and order maps
change, beginning on turn 1. The start swap changes the opening state, then the
economy and AI rules change the ensuing orders. This is intentional rule drift.

The fixed-order tapes for city-strike awareness and the three diplomacy
scenarios retain every recorded order. Their initial hashes and all resolved
hashes change from turn 1 because their setups include the reassigned Japan
start. The promotion scenario was generated again with the documented
`promotion_tape` example: its old orders no longer led to a promotion after
the start swap; the new 12-turn tape records two battles and a promotion on
turn 9. Its initial hash, all turn hashes, and orders change from turn 1.
The movement-path, production-queue-buy, and research-queue tapes remain
byte-identical. Fixed-order hashes were recalculated with the documented
`rehash_fixtures` example; native and WASM replay check each initial state,
turn, and save/load boundary.

The separate 80-turn mixed AI FNV-1a digests are now
`15107064506173654142`, `1110980210111484140`, and
`16483498028734520968`; the 160-turn scripted Hard digest is
`4319567997856860834`. These pin newly recorded AI orders as well as the
resolved state hashes.

## Merged Swiss/Japan tuning and walls-only city strike (2026-10-03)

After merging the walls-only city-strike rule, the three 40-turn mixed tapes
were recorded again with the command above. Their fixed-order replay hashes
already matched the recorded states. Rehashing the seven scenario tapes changed
`city-strike-awareness.json` from its initial state and first turn; the other
six scenarios kept their existing hashes and orders. The 80-turn mixed AI tape
digests are `2504841095865569349`, `16155837941404543511`, and
`9540091506108302212`. The 160-turn scripted Hard digest is
`2452337662575972327`. Both runner tests observed these values from the
merged gameplay code before pinning them.

## Hard conquest merge with Swiss and Japan tuning (2026-10-03)

Master's funded Hard conquest planner changes the 160-turn scripted Hard
recording's orders. The merged tape digest is `7774952460016751505`; the runner
test also replays that tape. The three mixed Normal AI digests above and the
fixed-order scenario tapes are unaffected by this AI-only change.
## AI upkeep budget (2026-10-03)

Scripted AIs were self-inflicting bankruptcy: 200 Hard-only 8-player Standard games
(seed 3200001) showed 11.3 upkeep-forced unit disbands per game (`rules::cities::pay_gold`,
always the costliest unit). An earlier candidate also tightened routine production and
`buy_surplus_army` to require income covering the next unit's marginal upkeep, but that
broadly suppressed army growth enough to reopen a title-race stall
(`crates/sim/tests/coalition_stall.rs`, held-out seed 910030: the game no longer finished
by turn 900). Production and purchase decisions are therefore unchanged. Instead,
`crates/ai/src/production.rs` adds two safety nets: `disband_insolvent_unit` (start of
`plan_cities`, triggered by `PlayerView.economy.bankruptcy`) and
`disband_if_this_turn_overspent` (end of `plan_cities`, catching the case where this turn's
own purchases are what tip net gold negative, crediting this turn's disbands/purchases
against the pre-turn income). Both deliberately disband our own weakest unit — preferring
one away from a threatened city, then one within 3 hexes of an owned city (an idle
garrison, not a unit mid-campaign) — before the engine's bankruptcy rule would otherwise
pick the costliest one.

`cargo test -p legatum-sim --test determinism` (the committed 40-turn fixture files) and
`cargo test -p legatum-sim --test coalition_stall` both still pass: short openings and this
title-race game never reach a budget tight enough to disband. The pinned 80-turn mixed
digest in `crates/sim/src/runner.rs` (same three seeds, longer games) changed and was
re-pinned; the 160-turn Hard digest was unaffected on its own.

Merged with walls-only city strike (2026-10-03, unwalled cities no longer fire): both
pinned digests were re-pinned again against the merged tree, since walls change combat and
economy enough that the Hard seat now also reaches a tight budget once (previously it
didn't, on its own). 200 Hard-only 8-player Standard games (seed 3200001) confirm the fix:
see `docs/sim-results/` for the before/after comparison.

## Byzantium salvage-Science faction goal (2026-10-03)

`Player` now serializes `byzantium_salvage_science`, a cumulative counter of Science
earned from salvage camps on other empires' land (`economy::salvage_science`,
applied in `rules/cities.rs::process_cities`). It replaces the old Deep Survey +
100-explored-land-tiles goal, which completed in about 98% of games and was a
freebie rather than a choice. No AI code reads the new field or the derived
`PlayerView::byzantium_goal` progress view (confirmed by inspection), so no AI
decision changes; only serialized state hashes change, because every player now
carries one more always-present integer field (0 outside Byzantium, and 0 for a
Byzantium empire with no salvage income yet).

All eleven current fixtures in this directory were rehashed with
`cargo run --release -p legatum-sim --example rehash_fixtures -- crates/sim/tests/fixtures/*.json`.
Every initial hash changes, and the first changed resolved hash is turn 1 in each
tape (the new field is present from the very first serialized state). Every
recorded order map is byte-identical; `rehash_fixtures` only recomputes hashes
and never touches orders or setups.

The pinned 80-turn mixed-AI tape digests in `crates/sim/src/runner.rs` are now
`9186916849508251262`, `5504241813155953862`, and `18435840220172831580`. The
160-turn scripted Hard tape digest is now `5530546092219924443`. These pin the
full tape JSON (state hashes included), so they move with any field addition
even when, as here, no AI order changes.

Native debug/release determinism tests and the directory-wide WASM tape check
(`node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node`)
verify every regenerated hash and save/load boundary.

## Combined playtest-6 merge (2026-10-03)

Merging the Byzantium salvage-Science goal (a new serialized faction-goal field) onto master with
walls-only city strike, Hard's funded conquest and the AI upkeep-budget fix. The three mixed tapes
were regenerated with the documented command on the merged code; `rehash_fixtures` recomputed
`city-strike-awareness.json` (initial state changes because of the new field) and confirmed the
other fixed-order fixtures unchanged. Runner pins updated for the same combined change.

## Byzantium salvage field skipped while zero (2026-10-03)

`Player.byzantium_salvage_science` now uses `skip_serializing_if = "is_zero_i32"`, the convention for
new state fields, so archived states (crates/ai/tests fixtures) keep their pinned hashes. Every
fixture was rehashed or regenerated again on that basis, and the runner pins updated. Orders are
unchanged; only the serialized state differs.

## Threat response on opponent assessment (2026-10-03, claude/ai-threat-response)

This branch keeps Astra's opponent assessment (`crates/ai/src/opponents.rs`,
`adaptation.rs`) and adds the Hard threat response (`crates/ai/src/threat.rs`). The three
40-turn mixed tapes were regenerated with the documented command:

```sh
cargo build --release -p legatum-sim && ./target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
```

Initial hashes are unchanged. Compared with master `2d50165f` recordings of the same games,
the first changed orders are: MAX-2 turn 17, player 4 researches Fortification instead of Soil
Reclamation (14/40 order maps and 24/40 hashes differ); MAX-1 turn 16, player 0 offers player
1 a Pact (25/40 and 25/40); MAX turn 6, player 2 researches Fortification instead of
Fabrication (26/40 and 35/40). These seats are Normal, so the differences come from opponent
assessment (border-defence research and strategy-dependent diplomacy); the threat response is
Hard-only. The pinned 80-turn mixed digests in `crates/sim/src/runner.rs` are now
`6631353086437098788`, `17002029135301505262` and `2441005981477055559`; the 160-turn scripted
Hard digest is `11653130744062113496` (first changed order turn 11: player 7 researches
Fortification instead of Exchange; 145/160 order maps differ). Fixed-order fixtures, including
`city-strike-awareness.json`, replay unchanged because no core rule changed.

## Threat response re-pinned on routes around friends (2026-10-04, claude/ai-threat-response)

After merging master `4c833157` (routes around friends) and changing the threat response
(Walls answer observed aggression rather than a rush-capable faction prior; a garrisoned shooter
before Walls; second fronts need a known city of the threat within 12 reachable hexes; scouts
revisit the threat's front), the three 40-turn mixed tapes were regenerated with the documented
command. Initial hashes are unchanged. Compared with master's tapes, the first changed orders are:
MAX-2 turn 28, player 2 adds a Pact offer to player 0 (9/40 order maps, 13/40 hashes differ);
MAX-1 turn 16, player 0 offers player 1 a Pact (25/40, 25/40); MAX turn 16, player 6 offers
player 0 a Pact (15/40, 25/40). These seats are Normal, so the differences come from opponent
assessment's strategy-dependent diplomacy; the early Fortification research the faction prior
caused (turns 6 and 17 in the previous regeneration) is gone. The 80-turn mixed digests in
`crates/sim/src/runner.rs` are now `9881291553555868247`, `12382156318996264786` and
`10542392459545366349`; the 160-turn scripted Hard digest is `16810274357082222845`.
Fixed-order fixtures replay unchanged because no core rule changed.

Re-run after merging master `075757e4` (cities shield their garrisons) and relaxing the
conquest nomination (`conquest.rs::assessed_conquest_allowed`: only Celts land and early Walls
rule a city out): the documented command reproduced the three tapes byte for byte (no city
assault in these 40 turns reaches the changed code). The 80-turn mixed digests are now
`6924625740608483303`, `9224472725858897275` and `1579728244672317460`; the 160-turn scripted
Hard digest is `11533285354215583339`.

## Merge: map-reveal ruin removal onto master (2026-10-04)

Removing `RuinReward::MapReveal` changes ruin outcomes. On the merged tree the three mixed tapes were regenerated with the documented command, `city-strike-awareness.json` was rehashed with its orders kept, and both runner pins were re-pinned.


## Earlier terrain sight integration (2026-10-03)

The three mixed 40-turn tapes and scripted Hard 160-turn tape were regenerated from
the merged simulator with the documented seeds, speed, brains and turn limits above.
All four initial hashes changed versus this master; all 280 resolved turn hashes and
all 280 turn order maps changed (first at turn 1). Terrain sight changes the initial
PlayerViews, which changes AI decisions immediately. The seven fixed-order scenario
tapes retain every order and turn count (18/12/15/8/150/12/120), but all initial
and resolved hashes changed. `rehash_fixtures` recomputed these with the merged
core; for `production-queue-buy` and `research-queue`, this also incorporates
master's newer Byzantium state. Native and WASM replay checks verify the hashes.

The 80-turn mixed tapes were separately simulated with the same three seeds and
`--turn-limit 80`. Their FNV-1a digests pinned in `runner.rs` are
`5648409198044568754`, `18144573642312775128`, and `13018396163716077927`.
The scripted Hard 160-turn digest is `14716709611220375435`. These pins represent
the combined terrain sight, walls-only city strike, Hard conquest, upkeep budget,
and Byzantium rule tree; earlier sections above describe the historical stages.

## Terrain sight with ranked garrison fire (2026-10-03)

Master `fcefc16d` adds threat-ranked ranged shots and lets a garrison fire from
its hold order. The merged AI still validates shots with the core LOS forecast;
its threat estimate likewise counts only ranged attacks with LOS through known
terrain (`crates/ai/src/tactics.rs::enemy_threat,ranged_line_clear`). The three
mixed 40-turn tapes are byte-identical to the previous terrain recordings, as
are their 80-turn pinned digests above. The scripted Hard 160-turn tape keeps
its initial hash but changes 102 resolved hashes and 54 order maps, first at
turn 29. It was regenerated from seed 3200001 with the documented Hard command.
The new `runner.rs` Hard digest is `13290855666370141044`. The seven fixed-order
scenario tapes retain their prior hashes and orders because this merge changes
AI decisions rather than core resolution. Native and WASM replay check the
combined tape hashes and save boundaries.

## Earlier terrain sight integration with master `210cf2e1` (2026-10-04)

This integration combines terrain-blocked vision/shot LOS with city shields,
value-ranked ranged garrison fire, routes around friends, concealed Celts filtering
for barbarians, and faction-accurate research/Byzantium Salvage Works. LOS changes
initial exploration and later fog memory; those PlayerViews also change AI orders.
City-shield threat estimates now exclude ranged volleys blocked by terrain. These
are intentional combined rule/AI changes, not unexplained determinism drift.

The three mixed fixtures were regenerated with the documented command (with an
explicit worker bound):

```sh
cargo run --release -p legatum-sim -- --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports --threads 4
cargo run --release -p legatum-sim --example rehash_fixtures -- crates/sim/tests/fixtures/*.json
```

`city-strike-awareness.json` retains master `210cf2e1`'s setup and all 160 recorded
order maps; only its hashes are recomputed. The other seven hand-authored scenarios
also retain their recorded orders. None of these fixed-order tapes is replaced by
a fresh AI recording.

The runner tests separately pin fresh AI decisions over the longer horizons:

```sh
target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 80 --brain mixed --threads 4 --label runner-mixed --tape-out target/runner-tapes/mixed --out-dir target/runner-tapes/reports
target/release/legatum-sim --games 1 --seed 3200001 --turn-limit 160 --difficulty hard --brain scripted --players 8 --threads 4 --label runner-hard --tape-out target/runner-tapes/hard --out-dir target/runner-tapes/reports
```

The mixed FNV-1a digests are `12363205608729738500`, `10543410998865524826`,
`5221049512727040339`; the independent Hard digest is `17142988170161240657`.
Native replay/save-load and WASM tape checks validate the fixtures, while runner
pin tests validate the freshly generated AI decisions.

Compared with master `210cf2e1`, all eleven initial hashes and all 615 turn hashes
change, first at turn 1. The three mixed recordings change all 120 order maps,
also beginning at turn 1. All eight fixed-order recordings retain their setups,
turn counts and every order map, including all 160 city-strike turns. The
comparison is checked structurally, independently of the new replay hashes.
## Final Swiss/Japan merge onto playtest-6 (2026-10-03)

The three mixed 40-turn tapes were regenerated with the command above after
merging the Byzantium salvage goal, AI upkeep budget, and ranged garrison fixes
with Swiss/Japan tuning. `rehash_fixtures` changed the initial and first
turn hashes for `city-strike-awareness.json`; the other seven scenario tapes
kept their orders and hashes. The merged 80-turn mixed AI digests are
`15901276988210293707`, `16658924726541588277`, and
`5086366039525964848`. The 160-turn scripted Hard digest is
`1040590216658763371`. Runner tests pin these observed recordings and replay
the Hard tape.

## Merged pathfinding and Swiss/Japan tuning (2026-10-04)

After merging `route_around_friends`, the three mixed 40-turn tapes were regenerated
on the combined source with seeds `18446744073709551613` through
`18446744073709551615`. `rehash_fixtures` then refreshed the fixed-order
`city-strike-awareness.json` tape; its first changed resolved state is turn 26.
The other seven fixed-order scenario tapes kept their hashes. The 80-turn mixed
AI digests are `791335054882420287`, `12785907402236455462`, and
`14724403475977499972`; the 160-turn scripted Hard digest is
`2445887388831954207`.
## Influence removal merged with the faction batch (2026-10-04)

After merging the Sparta, Japan and Swiss batch into the Influence-free
rules, the three 40-turn mixed AI tapes were regenerated on this combined code
with the documented high-u64-seed command, through `tools/sim/slot.sh` at six
threads. Their initial hashes were unchanged by the merge, while AI decisions
and resolved states changed. `rehash_fixtures` preserved every fixed-order tape's
orders and refreshed the initial and resolved hashes for
`city-strike-awareness.json`, `diplomacy-alliance.json`, and
`diplomacy-slice-1.json` (first resolved change at turn 1 in each). The other
four fixed-order tapes kept their hashes. The three 80-turn mixed AI digests are
`10305787774307362533`, `3623469149401595164`, and
`4748206261717324242`; the 160-turn scripted Hard digest is
`6057694067425766674`. Native determinism and the directory-wide WASM replay
check validate the recorded orders and states.

## Value-based colony integration (2026-10-04)

This branch retains the original `ca4eb54a` value-based colony valuation after merging the later master AI and rules changes. Four subsequent v4 experiments were rolled back: good-site tally cap four, population-four unassisted export, Hard pipeline two, and no Settler training from threatened cities. A paired 100-game Standard ablation averaged 4.11 cities at T200 for v4 and 4.12 for the rollback; the small-sample faction win ratios were 10.67 and 4.67. See `docs/sim-results/expansion-status-2026-10-04.md` for the 400-game validation on the pinned earlier master. The mixed tapes and scripted Hard pin were regenerated on this final merged tree.

On the latest master merge, the three 40-turn mixed tapes regenerated to the same decoded setup, orders, initial hashes and all turn hashes as master; the committed master bytes were retained. The 80-turn mixed runner pin also remained unchanged. The combined 160-turn scripted Hard tape changed to FNV-1a `7496051003047737313`, re-pinned from its newly generated orders and hashes. Native and WASM replay check the final fixtures.

## AI campaigns and the reserve ledger (2026-10-04)

`astra/campaign-funding` merges `astra/ai-campaigns` (persistent war campaigns) and adds
the AI reserve ledger (`crates/ai/src/funding.rs`: runway-gated troop builds, purchases,
upgrades and campaigns; insolvency never deferred to the engine). Both change scripted
orders, so `tools/sim/repin.sh` regenerated the mixed 40-turn tapes for seeds
`18446744073709551613` and `18446744073709551615` (seed `18446744073709551614` kept its
hash) and repinned `pins.json`. Fixed-order tapes kept their hashes.

## Uncapped Swiss interest (2026-10-17)

Switzerland's treasury interest lost its cap of 5 Gold a turn (floor(treasury / 100) with no maximum).
`tools/sim/repin.sh` rehashed three fixed-order tapes with a Swiss seat, keeping
every order: `city-strike-awareness.json`, `diplomacy-alliance.json` and `production-queue-buy.json`.
The mixed tapes and `pins.json` did not move.

## Allies fight their wars together (2026-10-18)

An Alliance is one side in a war, Call to Arms is gone and the scripted AI prices the wars an Alliance brings
(`docs/learnings.md`, "Allies fight their wars together"). `tools/sim/repin.sh` regenerated the mixed 40-turn
tapes: seeds `...613` and `...614` changed (their AIs score Alliances differently), `...615` kept its bytes.
`diplomacy-alliance.json` was re-recorded from `crates/wasm/examples/alliance_tape.rs` (the call phase is replaced
by an ally joining the declared war, and the determinism test now asserts a `WarJoined` event).
`city-strike-awareness.json` kept its setup and every order except the 17 `CallToArms` orders (the order no longer
exists), then was rehashed with `rehash_fixtures`; `diplomacy-slice-1.json` was only rehashed. The 80-turn
mixed digests and the 160-turn scripted Hard digest in `pins.json` moved with them. Scenario states that
carried retired call offers (`scenarios/lab/coalition-against-sparta.json`, `late-game-turns.json`,
`scenarios/war/war-golden-horde*.json`) were re-encoded with `rehash_scenarios` (the loader drops the retired
entries, so their hashes moved).

## Idle orders folded into Sleep and Skip (2026-10-20)

`HoldUnit`, `HealUnit` and `WakeUnit` no longer exist: serde reads `HoldUnit` and `HealUnit` as `SleepUnit` and `WakeUnit`
as `SkipUnit`, and a saved `Fortify` or `Heal` standing order as `Sleep`. The AI now issues `SkipUnit` where it issued
`HoldUnit` (it re-decides every turn and never reads `standing_order`, so its play is unchanged). Intentional changes from
`tools/sim/repin.sh` and `rehash_scenarios`: the three mixed tapes were regenerated (their orders say `SkipUnit`; every
hash moved because no unit carries a standing order), `city-strike-awareness.json` kept its recorded orders and was
rehashed (its old Holds now replay as Sleeps, so its first turn hash changed), the runner pins in `pins.json` moved with
them, and every scenario state was re-encoded (loaded Fortify and Heal standing orders read as Sleep). AI test pins on
archived positions (`attack_opportunism`, `ranged_garrison`, `scout_threats`, `settler_routes`, `threat_response`) were
re-pinned for the same reason.

## Outrider is an escape (2026-10-31)

Melee units may order attacks on visible pre-Modern mounted units from beyond adjacency. A horse
that stays can be hit; one that rides away escapes without being pursued. `tools/sim/repin.sh`
regenerated the three mixed tapes and rehashed every fixed-order tape: all nine fixture hashes
stayed unchanged. The AI can now value a distant attack on a stationary horse, so the first
`default_ai_tapes` digest and the `scripted_hard_city_fire_tape` digest in `pins.json` moved.
The other two default AI digests stayed unchanged. Historical single-game JSON reports produced
by `crates/sim/examples/hard_vs_human.rs` are prior-run results, not live rule pins; new runs
write their own per-game snapshots.
