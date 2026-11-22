> Archived 2026-11-22: kept for history; it no longer describes the project.

# Human-proxy benchmark

`ProxyBrain` is a fixed opponent for step 1 of [AI competitiveness](../ai-competitiveness.md).
It receives only `PlayerView`, has no persistent state or random decisions, and uses the same
orders, resources, movement and combat rules as every other player. It does not read telemetry.

`PROXY_VERSION = 4` identifies the alliance-aware policy. `--proxy-version 2` selects the
frozen `ProxyV2Brain`, `--proxy-version 3` selects the frozen `ProxyV3Brain`, and
`--proxy-version 4` (the default) selects `ProxyBrain`. Every simulation report records the
version in run metadata, per-game observations and the aggregate distribution. Older reports
without a version stay explicitly unversioned. A policy version alone does not pin game
rules or shared fogged geography helpers; record the engine revision and source manifest too.

V4 extends v3's diplomacy using the same `PlayerView`. It accepts trustworthy alliances that
protect a known flank or support a shared war, while preserving its planned conquest route.
Outbound alliances need a shared enemy; a flank without a shared war gets a pact. Legal calls
are evaluated against a reachable front, local reserves and the public strength of enemies
and existing allied combatants. Only one call is accepted per turn, and it replaces a new
independent declaration. The engine's eight-turn commitment is honored. The proxy calls its
own allies when a useful war needs additional strength, rather than inviting them into every
siege. Pending offers and unanswered calls never count as military support or flank protection.

A new war budgets for the target's unprotected local allies and the proxy's existing fronts.
When the proxy is the public coalition target, it also budgets for unprotected neighboring
teams. Only direct alliances count; a call does not automatically traverse an alliance graph.
Near the public threat threshold, an exposed empire assembles a larger reserve. A new Great
Work can wait behind a defender or existing investment when its two points would provoke a
coalition stronger than the proxy; projects already underway and immediate winning projects
proceed. This is an estimate from public information, not a prediction of unseen movement.

The [v4 recalibration report](../sim-results/2026-09-28-proxy-v4-recalibration.md) compares v3 and
v4 on the same master containing alliances and coalitions, with both Peace and legacy War
starts. The previous v3 report predates those rules.

V3 starts with v2's economy and combat orders and adds diplomacy from the same fogged view. It
chooses the weakest known, reachable neighbor with a viable infantry/ranged campaign, checks
public combat forecasts and reserves strength for visible flanks before declaring. Declarations
take effect next turn; projected permissions are never used to issue premature attacks. It
requests or accepts peace when home defenses or the combined enemy army become overwhelming,
or a campaign has no viable continuation. It accepts and proposes flank pacts, preserves an
expansion route while assembling its opening army, honors peace guarantees and cooldowns, and
gives lawful notice to reopen a fully protected expansion route or challenge a pact partner
approaching victory, only with a viable campaign and no active local war. Public notice history
keeps the army focused on that one intended front until the notice matures. Pacts that are
merely offered do not count as protection. Units caught inside closed borders take legal steps
toward known exits.

At Peace, combat units assemble outside the intended neighbor's borders while scouts explore.
Visible unprotected flanks retain a local reserve. Once it has two cities, one city can replace
an extra settler build with reinforcements toward two infantry and three ranged units; the
normal opening, funded purchases, infrastructure underway and Great Works retain v2's plan. If
upkeep is consuming a nearly empty treasury, it unlocks Exchange at the next research choice and
replaces unit builds with Markets. The immediate home-defense check takes priority over this
Market override. This breaks the Peace-start loop of building replacements that immediately
disband. Distant wars without a known reachable front can be wound down without blocking local campaigns.
Declaration forecasts credit guaranteed ranged damage before the infantry follow-up, without
spilling damage through a garrison into its city. These projections never change tactical
permissions before a declaration actually resolves.

This remains a deterministic approximation, not a guarantee against unseen attacks or future
pact breaches. Public army-title totals are available to every player; they do not reveal unseen
unit locations or composition. V3 does not call the scripted opponent's diplomacy planner.

The [version 3 calibration report](../sim-results/2026-09-28-proxy-v3-diplomacy.md) records Peace
and legacy War results against the pre-alliance Hard AI, with deterministic replay and exact check
receipts.

The [version 2 calibration report](../sim-results/2026-09-28-proxy-v2-calibration.md) records 300
rotated Standard games per difficulty on master `ca7e3f7`: 89.0% wins against Easy, 74.0% against
Normal and 83.0% against Hard, with median winning turns 202, 216.5 and 203. All 900 games
passed deterministic replay checks. **This still misses the requested yardstick:** Normal is
below 85%, typical turn-100 city counts are only four or five, and the Proxy does not establish
a turn-150 technology lead over the strongest AI. The report includes the survival metrics,
loss replays and exact check receipts. Later master changes are not included in that baseline.

The opening fields two ranged units and a melee escort, then prioritizes expansion to four
cities before filling out a seven-city economy. Site scoring values the first citizens' food
and production and discounts travel. Workshops, Granaries, Laboratories and Markets support
replacement troops, research and purchases. Queued units count toward composition targets.
The treasury funds quoted upgrades and builds while retaining an upkeep buffer; an affordable
emergency defender can interrupt construction. The fastest eligible city can start the Great
Work even while another build is in progress, using the engine's transferable earned production.

Tactics coordinate guaranteed minimum ranged damage before melee decisions. Volleys cannot
spill through a killed garrison into its city, and enemy ranged fire still happens in the same
phase. Infantry can follow a lethal volley into the city, multiple infantry can assault a
defended city, and a single occupier reserves an empty defeated city. Safe finishing attacks
precede healing. Scouts take exposed settlers or defeated cities when the visible forecast
supports it. Retreats and staging use explicit known routes and public combat formulas; nearby
reinforcements intercept threats at home. Marching units wait for complementary support rather
than continually walking back to fetch new troops.

The policy also pursues permanent VP through research, faction goals and a Great Work. Wins
may therefore be VP victories, unlike the historical conquest. Neither seven cities nor a
particular army size is guaranteed under attack.

The policy lives separately from the scripted planner and its difficulty knobs. Shared helpers
provide fogged geography and the engine's public combat forecasts. Changes to those helpers or
the rules can still change the benchmark; preserve the report's source revision when comparing.
This is a scripted approximation of a strong human, not a reconstruction of the turn-197 game.
Report measured strength honestly; the expectation that it will crush current AI is a hypothesis.

## Running it

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
./target/release/legatum-sim --proxy 0 --proxy-version 4 --proxy-start peace --rotate-proxy --difficulty hard \
  --games 300 --seed 700001 --determinism \
  --out-dir /tmp/legatum-proxy-v4 --label proxy-v4-vs-hard-standard-peace
```

Repeat with `--difficulty easy` and `normal` and matching labels.

Proxy v2 has no diplomacy code and was calibrated when every game was at war. Since
diplomacy (games start at Peace) it only fights AIs that declare war on it. V2 therefore
defaults to `--proxy-start war`, the legacy all-war ruleset. V3 and v4 default to `--proxy-start peace`,
the current rules. An explicit `--proxy-start war|peace` overrides either default. The chosen
version and start are recorded in `meta.proxy` and the version is repeated per game and in
the aggregate distribution. Legacy all-war disables diplomacy entirely, and v3/v4 emit exactly
v2's orders in that mode. Always report both starts when calibrating the proxy. `--proxy 0` alone
fixes seat 0;
`--rotate-proxy` rotates both seat and faction. Seats are zero-based. All eight seats have separate
teams, with one of each faction. The seven AIs fight each other as well as the Proxy, as in the
playtest; they can form alliances and coalitions through the ordinary diplomacy rules. Difficulty changes only their decisions.
Proxy mode rejects `--strategies` and `--strategy-matchups` so the benchmark keeps the ordinary
scripted opponents. The newer strategy-diversity observer can still report their behavior.

For game index `i`, seed is `700001 + i`, Proxy seat is `(initial seat + i) % 8`, and Proxy faction
is `Faction::ALL[(i % 8 + i / 8) % 8]`, swapped into the ordinary seeded faction shuffle. A block
of 64 games covers every seat/faction pair once. In 300 games every seat and faction appears
37 or 38 times. Difficulty runs use the same maps and assignments. No explicit turn limit is
added; the Standard safety cap is 1,200 turns and a cap stop is a stall, never a win.

## Reading the reports

The headline table gives Proxy win rate and median winning turn; seven-AI city totals at turns
100/150/200; cities founded per original AI seat; techs per original AI seat at 150; treasury per
eliminated AI; Proxy captures; and incident counts from the master AI analysis detector.
Proxy cities and techs are reported at 100/150/200, together with the tech-count difference
from the **strongest** original AI seat and the number of surviving AI empires. A positive
average-vs-AI comparison must not be mistaken for a lead over every opponent.

Snapshots are taken after resolving the specified turn. Earlier finishes are excluded, with
explicit observation counts. They are not zero-filled or extrapolated. Consequently late-turn
city averages are conditional on games lasting that long; read them alongside win rate and
winning turn. Eliminated AI seats still contribute their retained tech count in reached snapshots.
Foundings include capitals but exclude captures. Captures include recaptures. Gold samples
exclude survivors. Medians average the middle pair for even sample sizes.

Proxy runs automatically record and hash-verify a tape through `legatum-analysis`, even without
`--analyze`. AI incident counts and the appended opportunity/rate tables exclude the Proxy. Separate
Proxy incident counts have their own coverage denominator. The
detector version is recorded per game; its default settler threshold is 12 turns. Rules overlap,
so categories are not independent failures. `--determinism` also plays the entire game a second
time and checks every state hash and the final save/load hash. Only first plays enter metrics.

JSON contains the configuration, command, source revision, per-game observations and hashes,
aggregate denominators, and incident rates. Keep large JSON, tapes, binaries and check logs
outside Git; commit a compact Markdown receipt with commands, paths and checksums. The
three committed `proxy-vs-*-standard` reports use 300 games each, starting at seed 700001.

The first [900-game baseline](../sim-results/2026-09-27-proxy-baseline-comparison.md) is pinned to
`31b8eaf53cd9f6bd52d26fad3a097d899d03afe1`, before the +2 capital Science and scout/garrison
tactics changes merged from master at handoff. Its win rates (40.0% / 18.0% / 24.3%) did not reproduce the expected human
strength. Treat this as an initial policy baseline; further Proxy calibration is needed before
using the plan's under-25% Hard win-rate threshold as an acceptance test for AI competitiveness.

## Profiling a loss

Add `--archive --archive-dir /tmp/legatum-proxy` to a development run. The archive index maps
seed to game ID; per-game JSON identifies the Proxy seat and faction. Preserve the policy and
engine revisions, since a tape replays recorded orders rather than rerunning the current brain.

```sh
./target/release/legatum-sim --replay /path/to/game.tape.json.zst --analyze \
  --out /tmp/proxy-loss
cargo run --release -p legatum-sim --example proxy_trace -- \
  /path/to/game.tape.json.zst 0 > /tmp/proxy-loss-trace.jsonl
```

Both paths verify replay hashes. The trace separates the pre-order PlayerView from the resolved
city, technology and unit totals. It includes orders, own events, production options and visible
threats. It is an observer tool and is never called by the policy.

Per-game `telemetry.proxy.diagnostics` records first attack/capture, founding turns, research
completion turns, production completions, destroyed units, disbands by reason, cities lost and
Proxy incidents. Snapshot diagnostics add science, gold income, treasury, army composition,
foundings and captures. Diagnose investment allocation as well as idleness: a city can spend
every turn producing and still defer expansion or science too long.

The calibration targets are at least 70% wins against seven Hard and 85% against seven Normal,
a median win before about Standard turn 220, usually 6–8 cities at turn 100, and a clear tech
lead by turn 150. Use the full 300-game rotated reports for acceptance, not the smaller
development samples. Report any missed target explicitly before treating the Proxy as the
human-strength yardstick.

## Opt-in human playbook

Proxy v5 and the matched GreedyExpand comparison are described in
[Human playbook benchmark](human-playbook-benchmark.md). Select v5 with
`--proxy 0 --proxy-version 5`; the default remains frozen v4.
