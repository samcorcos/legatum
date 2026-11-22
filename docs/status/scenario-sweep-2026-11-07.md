# Scenario sweep, 2026-11-07

The designer's rule (2026-11-07): "scenarios are just for your learning not permanent fixtures, so do a sweep of scenarios that are no longer relevant and always delete scenarios that are no longer relevant for you to learn something." A scenario is irrelevant when it depends on a removed mechanic or faction, when Hard saturates it (45 or more of 50 met, or 0 of 50 on something that is no longer a target), when it duplicates another's gap, when its AI idea has landed or been dropped, or when it starts after the decisive choice has been made (a lost position). A scenario that tests a current gap stays. Branch `claude/scenario-sweep`.

## Counts

| Folder | Before | Kept | Deleted | Added |
|---|---|---|---|---|
| `scenarios/lab/` | 34 | 14 | 20 | 0 |
| `scenarios/` (top level) | 4 | 4 | 0 | 0 |
| `scenarios/war/` | 34 | 18 | 16 | 0 |

Four battle-tactics Lab cards (`sparta-walled-nicomedia`, `germany-two-cities-one-turn`, `swiss-hold-biel`, `sparta-before-the-walls`) landed from master after these counts were taken, so `scenarios/lab/` held 18 cards then (14 kept plus 4 added); the table above is the sweep's own tally. Four more cards (`germany-defend-essen`, `celts-steal-a-settler`, `india-strike-the-leader`, `mongols-ride-to-dyrrachium`) were cut on 2026-11-08 from his saves where he succeeds and Hard fails, so `scenarios/lab/` held 22. Three more cards from his 8.9.29 Aztecs win (`aztecs-eight-by-seventy`, `aztecs-swiss-spearmen`, `byzantium-break-the-fire`) were added on 2026-11-09, so `scenarios/lab/` holds 25.

Baselines in `docs/sim-results/scenarios/`: each deleted scenario's `-baseline.json` and `.md` are gone, plus the orphan `byzantium-salvage`, whose scenario was deleted earlier.

## War suite

The suite is still an AI gate, so only cases that Hard saturates, that duplicate another case, or that rest on a removed mechanic were deleted. Fresh run on master `dd84bec5` (Hard, 16 runs a case, `tools/sim/war-scenarios.sh --local --runs 16`) on the 19 cases kept before `war-walls-rise` read 100%: 20% mean goal success, 4 of 19 at human parity. Without `war-walls-rise` the mean is 16.2% over 18 cases. The last recorded suite (11-04, 64 runs, 33 cases) read 36% and 12 of 33 at human parity; the 18 kept cases read 13% in that run, so most of the old score came from the cases deleted here. **The new suite baseline is 16% mean goal success over 18 cases; the 11-04 number and the 17-of-34 parity target no longer compare.** `war-sparta-nicaea` stays only as the e2e fixture. The Walls 100 note: the fresh run is on current rules, the 11-04 run was not.

## Scenario Lab (`scenarios/lab/`)

| Scenario | Decision | Why |
|---|---|---|
| `athens-break-the-hold` | Kept | Contesting the leader's hold (Aztecs, T173); Hard 0/50. |
| `athens-empty-capital` | Deleted | Duplicate: an empty walled leader city struck by a small army, the same gap as `celts-strike-the-leader` and `germany-take-gergovia`; Hard 0/50. |
| `athens-next-points` | Deleted | Hard scores 630 in all 50 runs (no signal); the AI idea behind it (`decisive_buy`) has landed. |
| `aztecs-three-crowns` | Deleted | Tied to the cheapest-objective experiment, which concluded; score-only, Hard completes 50/50; VP conversion is covered by `mongols-eight-points`. |
| `byzantium-celtic-walls` | Deleted | Duplicate of `swiss-walls-athens` (a walled city to break); tied to the abandoned `joint_assault` idea; Hard 0/50. |
| `byzantium-empty-horde` | Deleted | Same position (turn 74, 8.9.25 Mongols game) as `celts-stop-the-horde`, from a Byzantine seat; Byzantium is also changing (Mercenary Companies, levy). |
| `byzantium-last-point` | Deleted | Depends on the Byzantine thin levy and hiring an army with Gold (Mercenary Companies), both removed; Hard 7/50. |
| `celtic-woods` | Deleted | Names Japan (now India, no Sakoku) and the pre-rework Celts (archers +3, 11-02 rework); duplicate of the Celtic-town strikes; Hard 0/50. |
| `celts-break-the-reserve` | Kept | Contesting the hold (Swiss 5,000-Gold Reserve); cut 11-07; Hard 0/50. |
| `celts-stop-the-horde` | Kept | The runaway leader nobody checks (Mongol horde, T74); in the Mongol-horde measurements; Hard 0/50. |
| `celts-strike-the-leader` | Kept | Empty walled city, small army, leader's army away (cut 11-07); Hard 0/50. |
| `coalition-against-sparta` | Deleted | Hard wins 50/50 (saturated); carried retired call offers. |
| `german-second-front` | Deleted | Duplicate strike on a busy leader (`ai-strike-escort`, `ai-declare` ideas landed); Germany's changing rules; Hard 0/50. |
| `germany-found-by-the-scout` | Deleted | Hard wins 50/50 (saturated): the `found_now` idea has landed. |
| `germany-take-gergovia` | Kept | Empty walled Celtic city, a small army (cut 11-07); Hard 0/50. |
| `hold-athens` | Kept | Defence at the gate against a veteran Hoplite rush (T10); Hard 0/50. |
| `hold-therapne` | Deleted | Duplicate of `hold-athens` (the same Spartan army, 10 turns later); Hard 0/50. |
| `india-no-niter` | Kept | Resource war (no Niter, take the Niter city); cut 11-07; Hard 2/50. |
| `india-take-the-settler` | Kept | Settler theft and escort (T12); Hard 0/50. |
| `japan-big-empire` | Deleted | Japan is now India: depends on the Sakoku vow and the old Japan's decadence; stale. |
| `japan-finish-the-vow` | Deleted | The Sakoku vow is reserved and no faction uses it; stale. |
| `japan-from-100` | Deleted | Japan's Century: the old Japan, Sakoku and 150 turns of score-only play; stale. |
| `late-game-turns` | Deleted | A turn-time probe, not an AI gap (speed has its own budget); score-only. |
| `mongol-open-gate` | Deleted | Duplicate of `swiss-strongbox` and `mongols-wide-india`; names the old Japan; Hard 0/50. |
| `mongols-eight-points` | Kept | VP conversion with a rival holding Flight (air answer); 11-06; score-only. |
| `mongols-rich-and-empty` | Deleted | Duplicate (strike a rich, empty leader) of `swiss-strongbox`; names the old Japan; Hard 0/50. |
| `mongols-wide-india` | Kept | Thin army against a 13-city leader (cut 11-06); Hard 3/50. |
| `sparta-next-door` | Kept | Air answer: Athens's Bomber against nine cities and three units; Hard 0/50. |
| `sparta-second-celtic-city` | Deleted | Duplicate of `germany-take-gergovia` (the same 8.9.26 Celts game); Hard 0/50. |
| `swiss-hold-the-south` | Kept | Defence against the leader's Field Cannons on five walled cities; Hard 1/50. |
| `swiss-keep-zurich` | Deleted | Starts after the decisive choice (the Aztec Alliance and the war with Sparta): the designer called it unwinnable. Not replaced yet (see Follow-ups). |
| `swiss-strike-the-horde` | Deleted | Duplicate of `swiss-strongbox` and the Celtic strikes; its `dead-war` experiment is over; Hard 0/50. |
| `swiss-strongbox` | Kept | Strike a rich, empty capital before the owner buys an army; gates `march_past_reach`; Hard 3/50. |
| `swiss-walls-athens` | Kept | Breaking a walled city (Walls 100 changed sieges); also the Scenario Lab e2e checks it is listed; Hard 0/50. |

## Top-level fixtures (`scenarios/*.json`)

Deleting these needs the wasm scenario tests, `scenario-build` and the AI fixtures moved to a lab scenario first, so they stay for now.

| Scenario | Decision | Why |
|---|---|---|
| `opening-race` | Kept | Engine fixture (wasm tests, scenario builder); Hard wins 25/50 so it is not saturated, and it measures expansion pace. |
| `runaway-conqueror` | Kept | Engine fixture only (wasm and AI tests); Hard wins 48/50. Delete when the tests move to a lab scenario. |
| `rush-defense` | Kept | Engine fixture only (wasm, python and AI tests); Hard wins 50/50. Same. |
| `rush-offense` | Kept | Engine fixture only (wasm, python and AI tests); Hard wins 50/50. Same. |

## War suite (`scenarios/war/`)

| Scenario | Decision | Why |
|---|---|---|
| `war-athens-two-pikemen-t112` | Kept | Defence at the gate against a small veteran army; Hard 37%. |
| `war-athens-two-pikemen-t97` | Deleted | Hard 100% (saturated); the same gap as t112. |
| `war-break-the-hold-t168` | Kept | Contesting the leader's hold; 0%. |
| `war-celts-wild-hold-t150` | Deleted | Hard 100% (saturated); the hold-break gap is kept in `war-break-the-hold-t168`. |
| `war-german-air-early` | Kept | Air answers; 0%. |
| `war-german-air-t214` | Deleted | Hard 100% (saturated); same air gap as the other two. |
| `war-german-air-t219` | Kept | Air answers; 0%. |
| `war-german-stand` | Deleted | Hard 98% (saturated). |
| `war-germany-berlin-t65` | Deleted | Hard 100% (saturated). |
| `war-golden-horde` | Deleted | Duplicate of `-thin` (same numbers); the attack check uses `-thin`. |
| `war-golden-horde-hold` | Deleted | Hard 100% (saturated); a Lab card removed. |
| `war-golden-horde-thin` | Kept | The attack regression check (`war-scenarios.sh --check`); 0% (known stale, kept as the check). |
| `war-hollow-byzantium-t86` | Deleted | Its predictions rest on Mercenary Companies and the Byzantine thin levy, both removed; 0%. |
| `war-hollow-swiss-t76` | Kept | Striking a hollow leader with a small army; 0%. |
| `war-hollow-swiss-t96` | Deleted | Duplicate of t76, twenty turns later. |
| `war-india-chur-ring` | Kept | Walled-city siege puzzle, still legal as India. Renamed from `war-japan-chur-ring` in this sweep (id and seat text; the city names and game history keep Japan's). |
| `war-japan-hoplites-t43` | Deleted | Hard 98% (saturated); an early Japan, a removed faction. |
| `war-india-sparta-t142` | Kept | Defence against a veteran; Hard 62% in the fresh run. Renamed from `war-japan-sparta-t142` in this sweep. |
| `war-mongol-bombers-t150` | Kept | Air answers; 12%. |
| `war-mongol-rush` | Kept | Attack on neighbours (AI attacks are rare); 0%. Its tribute sentences were removed (bought peace is gone). |
| `war-mongol-walls` | Deleted | Hard 100% (saturated). |
| `war-mongols-chur-t137` | Kept | Striking an empty small town with a large army; 18%. |
| `war-mongols-solothurn-t158` | Deleted | Hard 100% (saturated). |
| `war-sparta-all-out` | Kept | Finishing two weak empires; 0%. Its tribute sentence was removed. |
| `war-sparta-first-blood` | Deleted | Hard 100% (saturated); the declaration came from the desertion clock, which is removed. |
| `war-sparta-next-door-t101` | Kept | Striking an unguarded leader; 0%. |
| `war-sparta-nicaea` | Kept | Hard 100% (saturated) but kept as the Scenario Lab e2e and tour fixture. Replace the fixture and delete. |
| `war-sparta-walls` | Kept | Walls to break at once; 0%. |
| `war-swiss-poachers` | Kept | Settler theft; 6%. |
| `war-swiss-tepid-t180` | Kept | Strike a leader's walled city; 0%. |
| `war-swiss-tepid-t210` | Deleted | Hard 100% (saturated). |
| `war-swiss-tepid-t220` | Kept | Finishing an exposed city before the hold ends; 56%. |
| `war-swiss-tepid-t220-fresh` | Deleted | Duplicate of t220 with a fresh AI memory; identical results. |
| `war-walls-rise` | Deleted | Hard 100% in the fresh 11-08 run (saturated). |

## Follow-ups

- **Replacement for `swiss-keep-zurich` (designer's idea via the lead):** cut a decision scenario before the choice, from the 8.9.26 Swiss game at turn 24 (`~/LegatumRuns/coach-tools/coach-8926-swiss/extract/T24.json`): Switzerland with Bern and Zürich, 209 Gold, a Soldier and a Scout, the Aztec Alliance offer (proposal 5, answer by turn 26) pending, four Spartan Hoplites at Tlatelolco three tiles from Zürich; objective: hold both cities for 15 turns. Not done: that state file is rules version 19 and `cut.sh --state` refuses it on version 21 (`--allow-mismatch` does not apply to `--state`), so it needs the stamping route used for `germany-found-by-the-scout` (`audit_states`, then stamp) before cutting.
- Replace the e2e and tour fixture `war-sparta-nicaea` with a scenario Hard does not saturate, then delete it.
- Move the wasm and AI tests off `rush-offense`, `rush-defense` and `runaway-conqueror`, then delete them.
- Old records (learnings, ideas, coach reviews, past run logs) still name deleted scenarios as evidence; they were left as history.
- Nightly `scenario-lab.json` and `war-suite.json` are regenerated by the nightly run; their means change with this sweep.
