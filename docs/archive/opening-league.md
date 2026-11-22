> Archived 2026-11-22: kept for history; it no longer describes the project.

# Opening league

## In the game: cities fire back; Hard v2 is opt-in

- **City strike** (`city-strike` below) is a default rule; the genomes were tuned with it. `no-city-strike` turns it off; the old `city-strike` flag still parses and changes nothing. A city at 0 HP doesn't fire. The former TypeScript client mirrors it, and the turn report, attack forecast and Guide explain it.
- **Hard v2 is not the app's Hard.** The app's Hard seats play the scripted Hard. Hard v2 (`crates/ai/src/hard_v2.rs`) is available only through the `hard-v2-ai` setup switch (`legatum-sim --hard-v2`, or `--rules-experiment hard-v2-ai`), which `legatum_wasm::session::make_brain` honours, and through the sim's per-seat tools below. Validation on 2026-09-30 ([results](../sim-results/2026-09-30-hard-v2-validation.md)) found it easier for a strong player than the scripted Hard: it fights less and doesn't gang up on the leader.
- What Hard v2 does: each Hard seat draws one opening at game start, a seeded, deterministic weighted draw (seat brain seed) from the league's equilibrium mixture, with every named opening between 10% and 40% by default (`--floor`, `--cap`; the committed table uses a 25% cap) so games vary and whole tables rarely mirror one opening. The draw is global, since the league isn't faction-conditioned. A drawn opening plays its tuned genome with the behaviour primitives on and uses its phase machine below; drawing `Default` plays the scripted Hard. Opening memory lives in the brain, so a loaded save redraws the same opening but restarts its phase memory.

The genomes and weights are generated data, `crates/ai/src/tuned_openings.rs`. Refresh them from a league run (the psro directory, or its parent), then run the checks:

```sh
python3 tools/league/export_tuned.py ~/LegatumRuns/2026-09-30-standard/psro
cargo test --release -p legatum-ai hard_v2
```

The script reads `active-members.json` (a null genome means the opening's built-in policy) and `summary.equilibrium.mixture` from the last line of `equilibria.jsonl`, takes the gene order and bounds from the current tree's `--opening-schema`, applies the floor and cap (`--floor 0.10 --cap 0.40` by default) and rewrites the file. It fails if a genome no longer fits the schema, and warns when the run's rules differ from the game's defaults.

Sim tools: `--hard-v2` puts every Hard seat on Hard v2 and `--legacy-hard all|0,3,...` exempts seats from it; `--hard-v2-rotation` plays one Hard v2 seat per game (seat = game index mod 8) against seven scripted Hard seats; `--challenger <Opening>` plays one rotated seat with that opening's tuned genome against the lobby's AIs; `playtest_bench salts` replays a playtest map with seat 0 against scripted Hard or Hard v2. Reports record each Hard v2 seat's drawn opening, brain name (`hard-v2:<Opening>`) and phase switches. In runs with `--seat-opening`, Default seats are pre-v2 Hard, so league results stay comparable.

## League levers and policies

These are experimental rule levers and AI policies, not a balanced replacement for the default game. Pillars 8 and 10 still require faction-balanced, held-out matchup evidence. The built-ins are starting points for tuning; a mixed-lobby win rate does not prove counters.

Apart from city strike (above), no flags retains the default rules and save encoding. `Default` delegates directly to the existing brain unless `--ai-primitives` explicitly opts its scripted seats in. Random seats remain random. Experiments travel in `GameSetup` and tapes. The former TypeScript client does **not** mirror the existing rules experiments, so these levers are implemented in the Rust core (including WASM) only. Experimental city events are not an app/UI feature.

Archive saves carry the engine's source fingerprint, and an archive from any other engine is rejected. (The league branch briefly accepted its parent revision's default archives; with city strike on by default those games no longer replay the same, so master has no such exception.)

```sh
nice -n 10 cargo run --release -p legatum-sim -- --opening-schema > /tmp/opening-schema.json
nice -n 10 cargo run --release -p legatum-sim -- \
  --games 20 --threads 4 --seed 8120001 \
  --seat-opening all=DefendedExpand \
  --seat-opening 0=Default --seat-opening 1=Rush --seat-opening 2=GreedyExpand \
  --seat-opening 4=Snowball --seat-opening 5=Tall --seat-opening 6=Economic \
  --rules-experiment city-strike \
  --out-dir /tmp/opening-league --label smoke
```

Seat assignments are zero-based, repeatable, and applied left to right. Named openings replace the selected seat's brain; `Default` preserves `--brain` and any AI parameters. Existing `--ai-params` can supply the faction personality/decision parameters beneath an opening. Opening policies mix their phase's preferences with that faction personality; they do not alter faction strengths or drawbacks. Seat openings cannot be combined with Proxy or the older forced-strategy modes.

## Rule levers

| Flag | Meaning |
| --- | --- |
| `city-strike` | Default rule (`no-city-strike` turns it off). Every city with HP above 0 fires once during each ranged phase at the visible hostile military unit within distance 1–2 with lowest HP, then lowest unit ID. Cities use `city_defense_for_rules` as attack strength and the existing damage function, terrain/fortification defense, and identity-derived combat roll. All unit and city shots use one snapshot and apply damage together. No XP is awarded for a city shot. |
| `start-settlers=0..8` | Extra Settlers per player beyond the normal one. Placed on nearest connected passable land in deterministic breadth-first/hex order, without stacking civilians or sharing a foreign unit's tile. Ordinary starting units retain their IDs. |

`Battle.attacker` remains a number for unit attacks, byte-identical to old events. A city attacker is `{"city": <id>}`; it can never alias a unit. Its RNG identity uses the same separate city key space as city defenders.

## Policies and tuning

`crates/ai/src/openings.rs` defines `Default`, `Rush`, `GreedyExpand`, `DefendedExpand`, `Snowball`, `Tall`, and `Economic`. Each non-default policy has four ordered capital build slots with counts, Settler target/timing, escort mode, a saving threshold and burst-purchase item/count, research priorities, war triggers, and phase thresholds. All counts and thresholds, research weights, and the escort enum are in a flat `Vec<i32>` genome. `--opening-schema` prints schema version, named bounds, item/technology identities, and each opening's built-in vector. Use the **per-opening** `genes` bounds: buildings have a maximum count of one. The schema's array order is the genome order; do not sort genes by name.

Write a JSON integer array (not the whole schema) and load it with repeatable `--opening-genome Rush=/tmp/rush.json`. Wrong lengths, floats, unknown openings, out-of-bounds values, duplicate files for one opening, and a `Default` genome are rejected. Time genes use Standard turns. Report checkpoint turns are literal turn numbers.

The build list has first choice at an idle capital when its next item is legal, after any primitive whose weighted production priority exceeds 10,000. A locked item waits for research while ordinary production can use the turn. Completed counts are inferred from observed own buildings and newly seen own units born after turn 1 within one tile of the capital; starting units do not satisfy the list. The Settler target is additional settlements beyond the first city (owned cities plus living/committed Settlers), so captures and free starting Settlers reduce the remaining expansion requirement. List counts are independent of that target; the Settler timing genes also gate list entries and purchases. Settler production is explicitly selected before the time-discount scorer. Only actual empire threats can cancel it for emergency production.

Wait-for-escort reserves a nearby military unit before garrisons, holds the Settler until rendezvous, then sends them one shared safe step. Escort-if-threat does this only near visible enemies or remembered threats; None never reserves an escort. Tactical Settler retreats remain available.

With the weakest-target primitive enabled, war triggers and the Conquer transition use local visible defender-plus-city power divided by our reachable army within the distance gene. Disabling all three war-selection primitives restores the opening's original nearest-neighbour/deadline trigger. The weakness test requires the city to be currently visible; an unseen city is not an empty garrison. The brain uses the view's legal declaration catalog and never breaches a treaty automatically. All phases share the ordinary fog-safe tactical planner.

Phase transitions are recorded: Expand after an unsuccessful/repelled rush or completed expansion opening; Conquer with enough cities and a visible weak neighbour; TurtleTech after excessive losses or a public coalition targeting us; Defend when hostile empire military is close to home. Defend resumes the underlying phase once no threat has been seen for four Standard turns (without that hysteresis an army at the edge of `defend_distance` flipped the phase every turn). Recovery has a tunable minimum duration. Losses are disappeared own non-Scout military IDs observed during an empire war (including attrition/disband losses); this uses no omniscient casualty feed. Opt-in brains retain this observation memory for the sim's lifetime. Replaying saved tapes is exact because tapes retain orders; resuming a policy midway from a bare save does not reconstruct that memory. Default brains remain stateless.

After the initial build sequence completes or is abandoned for another phase, the ordinary public-VP endgame investment planner can also spend and redirect production. Opening quotas must not prevent an otherwise reachable winning investment indefinitely.

## Reports and validation

Each game's `players` entries contain `opening`, existing final individual `vp`, `rank`, `won`, `cities_at_turns` for 25/50/75/100, `wars_declared`, `cities_captured`, `cities_lost`, and `phase_switches` (`turn`, `phase`). Rank is competition rank by final individual VP, with ties sharing rank. `won` uses the actual winning team. Missing future checkpoints are `null`, not zero. Wars count successful declarations attributed to the seat's permanent team; receiving a declaration does not count. Phase traces contain the initial phase and subsequent changes; Default has no opening phases.

The tests retain the committed determinism fixtures and additionally pin three default AI tapes (`default_ai_tapes_are_pinned`: Standard, mixed brains, barbarians on, 80 turns, seeds max−2 through max), first from parent `12e00c6` and re-pinned when city strike became a default rule. Opening determinism covers a tuned genome, every lever, and save/load at every tape boundary. The 20-game smoke uses Standard seeds 8120001–8120020, the seat mix above, and the three-lever bundle. Larger smoke results and exact check exit codes are recorded in the branch handoff.

The 200-game smoke analysis (`sim-results/2026-09-30-opening-league-analysis.json`; raw run output, not in the public repository) records the command, rates, capped seeds, and verification results; the complete per-game report stays on the `evolution/league-phase2` branch (kept out of master for size). There were 197 VP finishes and three 600-turn caps, with no panics. Default and GreedyExpand dominated this fixed seat mix; Rush won once and Tall never won. These built-ins need tuning and rotated matchup/counter evidence before claiming strategic diversity.


## Composable behaviour primitives (schema 2)

Non-Default openings enable the nine primitives automatically. `--ai-primitives` also enables them on Default **scripted** seats, without assigning those seats an opening build list or phase machine. The flag is incompatible with Proxy and forced-strategy modes. All planning inputs are `PlayerView` fields, quoted prices, public diplomacy/standings, and memory of previous views. Telemetry has no feedback path into a brain.

Schema 2 appends 23 fields to the original 25, retaining their order and bounds. Each primitive has an independent 0–300 weight (default 100); zero disables its contribution. The remaining bounded fields are below. Use `--opening-schema` as the machine-readable authority; an old 25-value genome must be extended with the primitive defaults before loading. Existing Default saves/tapes do not gain any policy or primitive fields.

| Primitive | Trigger fields: default [bounds] | Behavior |
| --- | --- | --- |
| `escort_civilians` | `threat`: 20 [0,200]; `wait`: 4 [0,30] | For a Settler route leaving our borders, weight visible/remembered military risk along the known route. Reserve a healthy escort before garrisons, rendezvous, then move one shared safe step. With no escort, a city wait expires after W Standard turns. Existing tactical retreats take precedence. |
| `snipe_settlers` | `radius`: 5 [1,15]; `war_score`: 60 [0,200] | Closest free non-Scout military pursues a visible hostile civilian without a city or adjacent military escort. At Peace, add weighted S to the declaration score. Legal plain-Peace declarations are projected into our own view so the capture order can resolve that turn; treaty protection is respected. |
| `scout_upkeep` | `minimum`: 1 [0,5]; `until`: 100 [0,300]; `intel_age`: 20 [1,150] | Prioritize replacements below K through N, or below one when a known live rival/unique public leader has stale intelligence. Protect the needed scouts from routine disbanding and revisit remembered cities. Only direct sightings refresh intelligence. |
| `save_and_burst` | `reserve_percent`: 100 [0,300] | Reserve a weighted percentage of the quoted strike-force cost (`war_army` Rangers), or a larger deficit runway. Suppress routine purchases; on a war trigger buy up to weighted `burst_count` military toward `war_army`, including a melee unit when missing. Otherwise buy one Settler for a ready safe site if none is alive or committed. The ordinary winning-investment planner may still finish the game. |
| `weakest_target` | `distance`: 8 [1,25] | Rank visible cities by `(local visible defenders + city strength) / our reachable local army`. No current sighting or no local army means no weakness estimate. The weighted advantage combines with personality aggression and the snipe bonus; the existing `weak_percent` can independently trigger war. |
| `coalition_march` | `leader_percent`: 60 [1,100]; `force_percent`: 50 [1,100] | An active ally commits at least ceil(F% of its non-Scout army) toward the unique leader's nearest known reachable city when the leader's VP/target percentage strictly exceeds L. Ties produce no leader. Healthy free units march before garrisons; routes use known safe approaches. A larger weight can commit more units. |
| `siege_persistence` | `hp_percent`: 60 [1,100] | Reject both offering and accepting Peace when our military is in attack range of a visible enemy city below weighted P% HP. |
| `barbarian_settler_guard` | `threat`: 1 [0,200] | A weighted barbarian-only threat preserves an in-progress Settler and draws a free defender home. Production progress is retained; once the raid disappears normal unit assignments resume. Empire attacks retain emergency production. |
| `market_first_economy` | `horizon`: 60 [1,150] | Add weighted priority to Gold buildings only when incremental Gold by H (after construction time) repays their quoted Gold opportunity cost. Replacement buildings count only their additional yield. |

Field names in the genome are `primitives.<primitive>_<field>` (including each `_weight`). Time fields use Standard turns. Percentages, unit counts, radii, strength and weights are unchanged. Primitive scores compete with existing production choices; spatial commitments take units out of later assignment passes. This makes the weights composable without selecting another fixed strategy template.

Primitive memory has the same lifetime as opening phase memory: repeated planning for one turn does not advance it; leaving a city resets a Settler's wait. Saved order tapes replay exactly. Restarting an opt-in brain from a bare midgame save cannot reconstruct its earlier observations or waits.

## Primitive diagnostics

Every per-game player record includes `ai_primitives` to identify participation and the following counters, including for control seats. These are observations, not claims that an enabled policy succeeded.

| Field | Definition |
| --- | --- |
| `settlers_captured_by`, `settlers_lost` | Resolved capture events attributed to the new and previous owner, including barbarian transfers. |
| `escorted_settler_turns`, `unescorted_settler_turns` | Start-of-turn samples for Settlers outside their owner's territory; escort means permanent-team non-Scout military within one tile. |
| `scouts_alive_by_turn` | Dense map from resolved turn number to surviving own Scout count after resolution (including zero). |
| `gold_spent` | Actual Gold on resolved purchases and unit upgrades; excludes maintenance. |
| `gold_hoarded_peak` | Highest observed treasury at a turn boundary, including the starting treasury. |
| `wars_declared`, `wars_with_battles` | Successful declarations by the permanent team, and how many of those wars subsequently produced at least one battle involving that team. Count each war once; include city shots, exclude barbarian fighting. |
| `coalition_marches` | Dispatch turns on which at least the required F% quota actually finishes closer to the named objective; holding/blocked orders do not count. |
| `sieges_abandoned` | A damaged hostile city had our units in attack range before resolution and none after it, with at least one surviving besieger. Captures, city destruction and destruction of the entire assault force are excluded; making Peace does count. |

For league analysis, compare declarations with wars that fought, escort exposure with losses, and treasury peaks with purchases. An unescorted turn is not automatically a failure: threat thresholds and the wait timeout can deliberately permit it. Win rates from one seat mix are smoke evidence, not proof of balance or a counter matrix.

The [100-game behaviour primitive smoke](../sim-results/2026-09-30-behaviour-primitives.md) reports every counter and each opening's win rate with Hard AI, default rules and all primitives enabled. It finished 100/100 games by VP, with zero stalls and zero panics.

## Head-to-head duels

The duel harness (`tools/duel`) was removed on 2026-11-22; it remains in git history, with the commands in the dated sim-results that used it.
The simulator's opt-in `--players`, `--map-size`, `--factions`, and `--mirror`
flags preserve default eight-player setups and tapes. Duel evidence measures
opening counters in 1v1; it does not establish eight-player leader containment.
