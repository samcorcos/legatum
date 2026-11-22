# Coach proposal evaluation — 2026-09-28

Final retained policy: **All four high-confidence proposals**, with the two evolution genes.

All final comparisons use merged master **36aa7740d64dde76f509c1b7d255c11ad6262852**,
including evolution-v2, settler detour/escort repairs and the frozen Proxy v2 implementation. Master now defaults
to Proxy v3, so these commands explicitly select **`--proxy-version 2`**.

Each of the five proposals was built and tested alone against that master. Every policy
and the control ran **300 Standard games against Proxy v2 and seven Hard AIs**, with
`--proxy-start war --rotate-proxy`, held-out seeds **951201–951500**. Each also ran
**200 Quick AI-only games**, Normal difficulty and the current Peace start, seeds
**810000–810199**. All games, including capped games, enter the denominators.

Frozen binaries, exact commands, source hashes, per-game results and check logs remain
outside git at `/tmp/legatum-sim-artifacts/coach-20260928/`.

## Standard results

Positive AI change means fewer Proxy wins. Intervals are conservative paired 95%
intervals. The p-value is the exact two-sided test on paired discordant winners.

| Policy | Proxy wins / 300 | AI change (pp), 95% interval | Better / worse seeds | p |
| --- | ---: | ---: | ---: | ---: |
| Master control | 195 (65.00%) | — | — | — |
| Research fallback | 195 (65.00%) | +0.00 [-1.65, +1.65] | 0 / 0 | 1.0000 |
| Bombardment relief | 171 (57.00%) | +8.00 [+0.13, +15.61] | 43 / 19 | 0.0032 |
| Idle defenders | 181 (60.33%) | +4.67 [-2.56, +11.74] | 32 / 18 | 0.0649 |
| Settler timeout | 175 (58.33%) | +6.67 [+0.73, +12.38] | 27 / 7 | 0.0008 |
| Wide targeting | 199 (66.33%) | -1.33 [-7.86, +5.24] | 18 / 22 | 0.6358 |
| Research + bombardment + settler | 157 (52.33%) | +12.67 [+4.89, +20.03] | 51 / 13 | 1.9e-06 |
| Research + settler | 175 (58.33%) | +6.67 [+0.73, +12.38] | 27 / 7 | 0.0008 |
| Research + settler + idle | 181 (60.33%) | +4.67 [-2.56, +11.74] | 32 / 18 | 0.0649 |
| All four high-confidence proposals | 174 (58.00%) | +7.00 [-1.18, +14.95] | 44 / 23 | 0.0139 |

No final Standard batch stalled or panicked.

## Quick guardrail

| Policy | Median / P90 / max turns | Stalls / panics | Final gold mean / median / max |
| --- | ---: | ---: | ---: |
| Master control | 141 / 176 / 311 | 0 / 0 | 53.83 / 42.0 / 436 |
| Research fallback | 141 / 176 / 231 | 0 / 0 | 53.59 / 42.0 / 436 |
| Bombardment relief | 141 / 174 / 600 | 1 / 0 | 63.93 / 42.0 / 5115 |
| Idle defenders | 142 / 172 / 292 | 0 / 0 | 54.92 / 42.0 / 443 |
| Settler timeout | 142 / 174 / 600 | 1 / 0 | 73.15 / 43.0 / 8458 |
| Wide targeting | 139 / 169 / 274 | 0 / 0 | 56.48 / 45.0 / 455 |
| Research + bombardment + settler | 143 / 176 / 600 | 1 / 0 | 73.18 / 41.5 / 6320 |
| Research + settler | 142 / 174 / 600 | 1 / 0 | 97.48 / 43.0 / 8796 |
| Research + settler + idle | 143 / 174 / 228 | 0 / 0 | 54.19 / 43.0 / 436 |
| All four high-confidence proposals | 141 / 172 / 278 | 0 / 0 | 52.71 / 42.0 / 442 |

Lengths use the simulator's nearest-rank percentiles. Gold is the last recorded treasury across all 1,600 player seats, including eliminated players and capped games. Lower treasury alone is not evidence of stronger play.

| Policy | Median change, 95% interval | P90 change, 95% interval | Faction win-spread change (pp), 95% interval |
| --- | ---: | ---: | ---: |
| Research fallback | +0.00 [+0.00, +0.00] | +0.00 [-5.00, +3.00] | +1.00 [-1.50, +3.00] |
| Bombardment relief | +0.00 [-4.00, +3.00] | -2.00 [-12.00, +6.00] | +0.00 [-5.00, +3.50] |
| Idle defenders | +1.00 [-2.00, +5.00] | -4.00 [-13.00, +2.00] | +0.00 [-4.00, +5.00] |
| Settler timeout | +1.00 [-4.00, +5.00] | -2.00 [-13.00, +7.00] | +5.50 [-3.00, +10.50] |
| Wide targeting | -2.00 [-7.00, +1.00] | -7.00 [-17.00, +6.00] | -2.50 [-7.00, +3.00] |
| Research + bombardment + settler | +2.00 [-5.00, +5.00] | +0.00 [-11.00, +9.00] | +3.00 [-4.00, +8.50] |
| Research + settler | +1.00 [-4.00, +5.00] | -2.00 [-13.00, +8.00] | +4.00 [-3.50, +9.50] |
| Research + settler + idle | +2.00 [-5.00, +6.00] | -2.00 [-14.00, +8.00] | +5.00 [-2.50, +10.00] |
| All four high-confidence proposals | +0.00 [-4.00, +4.00] | -4.00 [-15.00, +6.00] | +4.50 [-2.00, +9.50] |

| Policy | Max-length change, paired 95% interval | Stall-rate change (pp), 95% interval | Paired stall p |
| --- | ---: | ---: | ---: |
| Research fallback | -80.00 [-80.00, +0.00] | +0.00 [-2.45, +2.45] | 1.0000 |
| Bombardment relief | +289.00 [-11.00, +289.00] | +0.50 [-2.38, +3.35] | 1.0000 |
| Idle defenders | -19.00 [-23.00, +47.00] | +0.00 [-2.45, +2.45] | 1.0000 |
| Settler timeout | +289.00 [-56.00, +377.00] | +0.50 [-2.38, +3.35] | 1.0000 |
| Wide targeting | -37.00 [-104.00, +51.00] | +0.00 [-2.45, +2.45] | 1.0000 |
| Research + bombardment + settler | +289.00 [-76.00, +377.00] | +0.50 [-2.38, +3.35] | 1.0000 |
| Research + settler | +289.00 [-6.00, +377.00] | +0.50 [-2.38, +3.35] | 1.0000 |
| Research + settler + idle | -83.00 [-106.00, +5.00] | +0.00 [-2.45, +2.45] | 1.0000 |
| All four high-confidence proposals | -33.00 [-89.00, +55.00] | +0.00 [-2.45, +2.45] | 1.0000 |

| Faction | Master | Research | Bombardment | Idle | Settler | Wide | R+B+S | R+S | R+S+I | All four |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Japan | 31 | 30 | 23 | 26 | 19 | 28 | 19 | 20 | 22 | 25 |
| Germany | 34 | 33 | 32 | 39 | 44 | 37 | 43 | 45 | 39 | 39 |
| Mongols | 11 | 10 | 14 | 11 | 16 | 13 | 17 | 16 | 11 | 13 |
| Sparta | 11 | 12 | 11 | 11 | 6 | 17 | 6 | 6 | 5 | 4 |
| Byzantium | 15 | 16 | 16 | 14 | 13 | 12 | 15 | 13 | 14 | 13 |
| Athens | 15 | 14 | 20 | 17 | 17 | 18 | 18 | 18 | 21 | 19 |
| Switzerland | 41 | 43 | 41 | 42 | 48 | 37 | 41 | 45 | 46 | 44 |
| Celts | 42 | 42 | 42 | 40 | 36 | 38 | 40 | 36 | 42 | 43 |

Faction entries are wins out of 200 games. Win spread is the highest minus lowest faction win rate. Per-faction paired intervals and mean-gold differences are in each `comparison.json`.

## Adoption decisions

Retain research, bombardment relief, idle-defender commitment and settler timeout
**together**. The full package improves the observed AI win rate by **7.0 percentage
points** (44 better versus 23 worse seeds, exact paired p=0.01393). Its conservative
95% interval is [-1.18, +14.95] points. Quick median is unchanged, P90 falls four turns,
and all 200 games finish. The intervals for length and faction spread include zero.
These are sample results, not proof of general noninferiority or a robust strength gain.

Research changes no Standard winners in this sample. Bombardment, idle and settler
isolation improve observed AI wins by 8.0, 4.7 and 6.7 points, respectively. Research's
fallback and the other local mechanisms have separate failing-before/passing-after tests.

**The individual caps remain material caveats:**

| Candidate | Quick seed | Candidate turns | Control turns |
| --- | ---: | ---: | ---: |
| Bombardment alone | 810149 | 600 (capped) | 311 |
| Settler alone | 810182 | 600 (capped) | 138 |
| Research + settler | 810182 | 600 (capped) | 138 |
| Research + bombardment + settler | 810059 | 600 (capped) | 210 |

One paired cap in 200 has p=1.0 for a two-sided stall-rate test; it does not establish a
population-rate regression beyond sampling noise, but it is a real deterministic failure
on that seed. The initial conservative screen provisionally excluded policies with any
new cap. Final adoption additionally evaluated their interactions: both the three-change
research/settler/idle package and the full four-change package have zero caps. The final
branch retains the full package, with unchanged mechanism thresholds. Do not treat an
isolated settler or bombardment cherry-pick as the same validated policy. This is an
exploratory selection using the requested held-out comparisons, not a preregistered test.

Faction balance remains uncertain: Sparta wins fall from 11 to 4 out of 200; the
combined max-minus-min faction win spread rises 4.5 points, with interval [-2.0, +9.5].
Final mean treasury is 52.71 versus control 53.83. Keep watching faction performance and
rare long games in the next evolution run.

**Exclude Wide targeting.** Its final result is -1.3 AI points with p=0.636, which is
inconclusive. An earlier independent matched cohort regressed by 6.0 points with
p=0.00393. There is no convincing evidence that this heuristic is a useful counter to
Wide, and the earlier harm argues against adoption. The implementation and regression
are preserved in `1c2cd26`, followed by exclusion commit `ba90223`.

## Mechanisms and regression evidence

- Research (`307bf93`): preserve current research, positive-value VP routes and explicit
  finish deadlines; otherwise choose the cheapest available eligible technology.
  `research_falls_back_to_cheapest_unvalued_technology` fails on stock and passes after.
- Bombardment (`4d76b40`): stock already fires ranged garrisons and buys defenders.
  The candidate fixes out-of-range coverage, sends healthy free ranged relief and
  redirects long production/purchases toward counterfire without stripping garrisons.
  Counterfire movement and out-of-range purchase regressions fail before and pass after.
- Idle defenders (`3fa67fd`): after two held turns, a healthy surplus defender near its
  threatened city chooses a favorable attack or safe defensive position, even after the
  ordinary defense quota is filled. Its stock-failing regression covers the quota case,
  the two-turn deadline, the city garrison and absence of enemies.
- Settlers (`df67e84`, `bc7c0c8`): persist owner-visible creation turns; after the scaled
  deadline, choose safe legal nearby land reachable within two movement phases and
  confirm the route explicitly. Skip optional founding waits. If there is no such site,
  continue normal safe expansion rather than returning into a crowded homeland.
  Deadline and immediate-founding regressions fail on stock and pass after. Tests cover
  all four speeds, save/load, private metadata, border inference and actual movement.
- Wide (`1c2cd26`, excluded by `ba90223`): the candidate weights known cities against
  visible/remembered defenders and
  favors thin, newer border cities, subject to favorable local army power. Both the
  target-preference and local-commitment regressions fail on stock and pass afterward.
- War gene (`7870865`): the gene-wiring regression fails without the campaign multiplier
  and passes with it; the default multiplier preserves the original threshold.

`checks/coalitions-red-green.json` records eight expected behavior assertion failures
(exit 101) with stock decision routines, followed by a passing candidate run (exit 0).
Age/gene/route scaffolding was present solely so the tests compiled. This mechanism
check used master `3740471`; final performance results above use `36aa774`. The tests
reproduce the decision mechanisms; they do not replay the original evolved coach policy.

Merging the later detour repair exposed a real timeout visibility loop: a settler
returned to a site when its remembered foreign border faded from view. The unmodified
archive regression failed before `bc7c0c8`; the library/border/route/threat suites then
passed (204 passed, 1 ignored). The repair rejects short routes that would bring the
same known border risk back into sight, without extrapolating from distant claims.
An imported escort assertion was updated to accept an explicit route to the same site;
its legal movement, sole escort, held garrison and successful founding checks remain.

Coach evidence in the overnight run's `coach/10.json` and `coach/20.json`:

| Proposal | Game, player, turn |
| --- | --- |
| Research | g20 `native-1790594692665592000-70283-47`, P7 T147–148; `native-1790594686225359000-70283-18`, P3 T131–132 |
| Bombardment | g20 `native-1790594686370630000-70283-19`, P3 T80–81 |
| Idle | g20 `native-1790594683761282000-70283-8`, P5 T28–29 |
| Settler | g10 `native-1790589094877907000-38765-53`, P5 T31 |
| Wide | g20 `native-1790594688761153000-70283-34`, P0 T122 |

## Genes and fixtures

`war_commit_ratio` is integer hundredths, **100–250 = 1.00–2.50×** the existing
personality-derived campaign threshold; default **100** is neutral. It does not change
local attack forecasts. `settle_patience` is **4–16 Quick turns**, default **10**, scaled
by speed (20 turns at Standard). Empty and built-in params reproduce the stock policy's
serialized tapes at both speeds and all three difficulties. No evolved params were
installed as defaults. Rust and Python agree on all **71 genes**, including 31 shared
ones, their bounds and defaults.

Creation turns are owner-only and serialized only when different from the legacy
turn-one default. Old saves therefore conservatively age pre-existing settlers from
turn one. The documented fixture generators were rerun and native/WASM replay checked.
The final frozen all-four binary regenerated the three AI-driven tapes byte-for-byte;
the six manual examples do not invoke the AI.
Compared with final master, all **nine initial hashes and 443 turn-order lists remain
unchanged**. Four tapes have **174 changed hashes**, solely from creation metadata.
A separate replay audit restored that metadata to one and recovered the original hashes.
Updated tapes use version 4; unchanged version-3 tapes retain compatibility coverage.
See [fixture instructions](../../crates/sim/tests/fixtures/README.md).

The control's **73 AI/core source files** match the stated baseline exactly. The retained
binary's AI/core/sim/WASM source hashes match the final branch
(`checks/head-retained-source-audit.json`).

## Checks

| Command | Exit | Result |
| --- | ---: | --- |
| `cargo fmt --all --check` | 0 | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Passed |
| `cargo test --workspace --release --no-fail-fast` | 0 | 910 passed; 0 failed; 4 ignored |
| `cargo test --workspace` | 0 | 910 passed; 0 failed; 4 ignored |
| `cargo test -p legatum-sim --test determinism` | 0 | 12 passed; 0 failed; 0 ignored |
| `cargo fmt --all` | 0 | Applied before checks |
| Python evolution tests | 0 | 24 passed |
| Optimized WASM build and tape replay | 0 | 9 tapes, 443 turns, every save/load boundary |
| `git diff --check`; conflict-marker scan | 0 | Before each commit |

Four ignored tests are existing opt-in performance/evaluation/corpus checks. One earlier
release run exited 101 on the escort order-representation assertion described above;
the final complete rerun is the result in this table.

## Protocol and limitations

```sh
$BINARY --threads 4 --out-dir "$OUT" --games 300 --seed 951201 \
  --speed standard --proxy 0 --proxy-version 2 --rotate-proxy \
  --proxy-start war --difficulty hard --label "$LABEL-proxy"
$BINARY --threads 4 --out-dir "$OUT" --games 200 --seed 810000 \
  --speed quick --difficulty normal --label "$LABEL-quick"
```

There is no explicit turn limit; existing safety caps apply. Win intervals subtract
simultaneous 97.5% Wilson bounds for paired better/worse frequencies. Exact paired
p-values condition on discordant games. Quick intervals use 4,000 paired whole-game
bootstrap draws (seed 9173). Gold intervals use paired per-game eight-seat means.
These samples do not prove noninferiority or a general strength improvement.

All intermediate runs remain in the artifact directory. They are not pooled with the
final matched sample. The first and intermediate settlement implementations are marked
`ABANDONED.md`, having failed integration regressions; their performance was excluded.
Earlier coalition master `3740471` gave Wide 27 worse versus 9 better seeds (p=0.00393)
and a four-proposal combination stalled seed 810114 at 600 turns. Removing any one
proposal made that seed finish, so no unique cause was established. The stall also
reproduced on `0db3a5f`. Earlier three-proposal Quick median degradation on `3740471`
(+4 turns, interval [1, 8]) is preserved as adverse historical evidence. Master changes
and interactions materially affect results; passing a later sample does not erase that
sensitivity. Numeric results in early commit messages refer to those earlier baselines.

| Final binary | SHA-256 |
| --- | --- |
| head-master | `2bfa805202d6e7e5b95448fecbe033298a21bf69903e5fd2794164c6fd4757a7` |
| head-research | `6f867fea0788eeef4803b0fa6c289c50b95d5f8ddc83cf37640264ec61a1333e` |
| head-bombardment | `1a007cd97f43db837703fc2f2c74f5649d6c1738174af87a9a9e0a5ac850257d` |
| head-idle | `1e2ec464618eb7fed8f4bf0d0a89d875257ee66a605791b58802a1edb44bc375` |
| head-settler-fixed | `3ba6b035fc7a54723fabd1eee72c8d3a507746e7b8c407dece4842ca2577068c` |
| head-wide | `631a478b4de9a3f72579d7ea43c4890d2327c3c095b0a2130c228ee192f0649a` |
| head-accepted-fixed | `d422d6192dc673cef17023cf10414e2c6101c9697146776c43bacd67a6bcc28f` |
| head-research-settler | `db5094e0ae56adc897d42b1983edf67654156a430b614e98d1a7216bc0eec01b` |
| head-research-settler-idle | `9e87366a23235f65ea261b44b0693ca7d0e940f2f20d27d713c198d346e4eaaa` |
| head-four | `db84d45a8fed7e603b96743de97fc36d00dc29e15c4fc44e9cf4cdd9af7c7d27` |
