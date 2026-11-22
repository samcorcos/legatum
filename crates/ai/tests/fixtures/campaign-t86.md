# Campaign memory regression

`campaign-t86.json.gz` is the complete, unmodified pre-T86 `GameState` from
`~/LegatumData/archive/2026/09/native-1790558190343840000-39476-6.tape.json.zst`.
Its hash is `3706742736254659107`. Terrain and every player's observations are retained.

Archive engine: `091f4a5868cbf24e2b6bb6759d2db93d65b96795`.
Verified current revision: `625eff57d9cfc7c012d5ef67684d696a45736dd3`.
Rules fingerprint: `8ef43f58da848e20ecfe12583c67693d3aebb043e4b63292e998350c1a1d1bf5`.
Replaying the tape with the current Rust core matched the initial hash and all
194 resolved hashes. Independently generated Normal AI orders, using
`Rng::derive(setup.seed, [purpose::AI, player]).next_u64()`, matched every seat
on every turn. The fixture was serialized immediately before resolving T86.
No archive metadata or replay compatibility checks were changed.

Before the fix, T86 orders u40/u44 from (23,18)/(23,19) to (24,18)/(24,19).
T87's archived position exposes the defenders and orders both Soldiers back.
The pre-T86 view has no visible enemies and remembers u9 at (26,18) and u30 at
(26,17), both seen on T85. Their **actual** pre-T86 positions are already
(27,17) and (26,15); those hidden moves must not inform P1's decision.

The regression resolves eight turns of newly generated P1 orders with every
other player's units holding at their actual positions, including new units.
Production and terrain stay intact. This counterfactual also fails before the
fix: u40 repeats (25,18) / (24,19) on T90–T93, while u44 repeats (24,18) /
(23,18). Both uninterrupted and save/load runs use a fresh brain each turn.
Later states are counterfactuals, not claims to reproduce archived hashes.

The cycle assertion checks resolved battle events too. A reachable attack on a
surviving city can leave its attacker adjacent, then require recovery; actual
damage is progress, not the reported empty approach/retreat cycle. The final
assertion still fails on the original planner. Additional tests cover clearing
the defenders through visible reconnaissance, hidden-position invariance,
observation expiry and deduplication, strength reassessment with reinforcements,
safe staging, and separating a favorable forecast from actual melee reach.

## Rules cleanup (2026-10-31)

The serialized starting-state pin moved when retired score and war-rule fields were removed.
City strikes are now always part of the live rules. The reconnaissance test still verifies that
last-seen defenders clear and subsequent decisions stay legal and stable across save/load; it no
longer requires Soldiers to assault a city based on that cleared memory alone.
