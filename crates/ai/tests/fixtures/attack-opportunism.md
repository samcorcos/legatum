# Attack opportunism regressions (playtest-6.2 scenario runs)

Extracts from the user's scenario runs in `~/LegatumData/scenarios/` (format
`legatum-scenario-run`, written by the app). The run files stay in the user's application
data; only these extracts are committed. Both runs were played on playtest-6.2: archive engine
`58981447e68bd7e8628871247508b546c1e33ab2`, rules fingerprint
`3f00f77336653262e30474201b82652a18e8b663eab7dd135a2014725fe28261`. The current engine
(branch base `e53d6a36`) reproduces each archive's initial hash and every resolved hash, and
the current Hard `ScriptedBrain` (fresh each turn, seed
`Rng::derive(setup.seed, [purpose::AI, player]).next_u64()`) reproduced every recorded AI order
in both runs before the fix.

The archive stores states in the app's save JSON (`legatum_wasm::json`, which packs bitsets);
the fixtures are the same states parsed by that loader and re-serialized as plain `GameState`
JSON, so `crates/ai` tests can read them. The state hash is unchanged by the conversion.

## rush-offense ("The Last Bridge")

`rush-offense-t22.json.gz` is the run's `archive.initial_state` from
`rush-offense-2026-10-04T00-53-04Z.json` (pre-T22, hash `1590038518724073440`, scenario
revision 1). The user (Sparta, seat 5) declared war on Mongols on T22, moved all four
strike units the same turn, shot with the Ranger from T24 and captured Last Bridge (21,16) on
T26. Every one of 50 Hard baselines fortified for about six turns and declared around T28:
`diplomatic_campaign` refused an optional war without `conquest_ready`'s fixed recipe of two
ranged and two melee units, so the AI waited for a second Ranger. The tests make the seat a
Hard AI (as the scenario baselines do) and check the T22 orders, then play every seat with the
scripted AI until the capture.

## rush-defense ("Glass at the Gate"), the Mongols attacker

`rush-defense-t24.json.gz` starts from the run's `archive.initial_state` in
`rush-defense-2026-10-04T00-46-02Z.json` (pre-T21, hash `6551600801439521389`, scenario
revision 2) with T21-T23 replayed from the recorded orders on the archive-compatible engine
(branch commit `561f6a3d`; every resolved hash matched), so it is the archived pre-T24 state
(hash `2882848999431146048`). `rush-defense-t21-t29-orders.json` holds every player's recorded
orders for T21 to T29 with the archive's resolved hashes. Master's routes around friends
(2026-10-03) resolve the same T21-T23 orders differently, so the tests start from T24.

What the Mongols Hard AI (player 2) did, from its own orders and the replayed states:
- T21-T23: Rangers u39/u40 shot Hymnal (23,15) and Soldier u25 assaulted it; Hymnal fell on T23.
  The treasury fell 9, 6, 3, 0: six military units on one city (two spawned by the scenario
  builder) cost 4 Gold a turn.
- T24: `disband_insolvent_unit` plus `disband_if_this_turn_overspent` disbanded u25 (the captor,
  standing in Hymnal at 64 HP) and both Rangers to cover a 3-Gold deficit, although a bankrupt
  treasury resets to zero and the engine disbands only one unit. Units within 3 hexes of an owned
  city counted as idle garrisons, and the newly captured city was owned.
- T24-T27: with two cities, the Hard conquest planner chose Keystone Wreck (a neutral Byzantium
  city): Requiem Deep, Japan's other city four hexes from Hymnal, had never been seen. The
  campaign sent u9 and u43 to stage at (16,21) and (18,21), leaving Hymnal (5 HP) empty.
- T28-T29: Japan units came into sight; `defend_cities` turned u33 and u43 back, too late. Japan
  Soldier u41 walked into the empty city on T29.

The counterfactual test keeps the user's recorded orders (and every other seat's) for T24-T29
and lets the current Mongols AI decide; orders the changed position makes invalid simply fail.
