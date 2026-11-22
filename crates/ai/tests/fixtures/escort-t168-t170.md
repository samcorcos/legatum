# Arrived escort regression

Source: `~/LegatumData/archive/2026/09/native-1790608138605096000-13753-19.tape.json.zst`.
Incidents: I000257, I000259, I000261.

`escort-t168-t170.json.gz` contains P6's exact PlayerViews before T168 and T170,
its derived AI seed, original orders, and source state hashes. The snapshots
were extracted from a native replay at revision
`374047162da395b5c6c308098fb068330a8a6190`, which was also the current branch base.
The rules fingerprint was
`168a0c77bbf48acfc62d7851c5fb7a197ca4a78dab2cd8d51c341af628682cc8`.
The initial hash and all 260 resolved turn hashes matched the archive.
Fresh Normal `ScriptedBrain`s independently reproduced all eight seats' complete
order lists on each of T167–170, using
`Rng::derive(setup.seed, &[purpose::AI, player_id]).next_u64()`.

Soldier 29 was healthy at (38,12) before both T168 and T170. Settler 37 was also
healthy and still headed to (38,12); P6 had no wars. The soldier was nevertheless
ordered to (35,15), returning to the settlement site in between on T169.
The fixed policy changes only the soldier's order to HoldUnit in both recorded
views. No world state or additional knowledge is supplied to the brain.

The separate synthetic regression resolves real turns on current rules with a
three-move Mongols Soldier, two-move Settler, capital garrison, spare soldier,
and a legal destination four hexes from home. Brains are recreated and save/load
orders compared every turn. Retention must also cover the pending founding
order: retaining only walking settlers still sends the soldier home on the
founding turn. Once founded, normal city garrison planning takes over.

Migrated 2026-10-05 (population is territory): in both views' own-city details, the removed border
fields (`border_progress`, `border_growth`, `next_tile_cost`, `next_border_tile`,
`turns_to_next_tile`) were dropped and the new population-claim preview was set to null, so the
views still deserialize. The snapshots' fog, units and orders are untouched.

Migrated again 2026-10-05 (Monument and Assembly cut): Assembly entries were dropped from the
views' production options and tech unlocks (`Monument` loads as the Brood Hall through a serde
alias). Fog, units and orders are untouched.
