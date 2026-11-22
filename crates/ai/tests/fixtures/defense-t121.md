# Defense assignment regression

`defense-t121.json.gz` is the complete, unmodified pre-T121 state from
`~/LegatumData/archive/2026/09/native-1790558190343840000-39476-6.tape.json.zst`.
Its hash is `4739007321766802230`; terrain, cities, units and observations are intact.

Archive engine: `091f4a5868cbf24e2b6bb6759d2db93d65b96795`.
Verified current revision: `fcdb97d` (including the separate campaign-memory fix).
Rules fingerprint: `8ef43f58da848e20ecfe12583c67693d3aebb043e4b63292e998350c1a1d1bf5`.
The current engine's archive replay passed its compatibility check and matched
the initial hash and all 194 resolved hashes. A separate Rust probe extracted
this snapshot while checking every archived hash. Current Normal AI orders for
u49 matched its archived orders on T121 through T126. Seeds were derived with
`Rng::derive(setup.seed, [purpose::AI, player]).next_u64()`.

Before the fix, six resolved turns of generated P4 unit orders move u49 between
support (12,16) and city (13,16), three times, while visible u42 holds (16,13).
With a friendly reinforcement occupying (12,16) and u49 starting in the city,
the same loop moves to (12,17). Unit u69 does not exist at pre-T121; the blocked
variant adds that reinforcement early and holds it at the escape tile.

The tests use a fresh brain and compare uninterrupted and save/load resolution
each turn. Other players hold all their units, including newly produced ones;
P4 uses generated unit orders, with u69 held in the blocked variant. Existing
production continues. Later states are counterfactuals, not archived hashes.
Additional variants give u49 a suitable defender's stats and visibly clear u42
to check stable garrison duty and renewed occupation.

The original city's mountain/hill approach costs more than u42's three-move
budget. Its reported threat was the planner's coarse hex-distance estimate;
it was not evidence of a reachable attack. With the fix, u49 can occupy and hold
this city. A second variant opens that approach by changing (14,15) and (15,14)
to Plains. There the Ranger stays fortified at support, a Trooper can hold the
city, and visibly removing u42 permits the Ranger to reoccupy. Hidden relocation
tests preserve the complete PlayerView, including public army standings.
