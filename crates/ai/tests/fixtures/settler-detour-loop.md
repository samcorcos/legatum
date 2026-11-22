# Settler detour commitment regression

`settler-detour-loop.json.gz` contains the pre-order turn-150 state and five
original order frames from historical seed 990146 at master `4827ca7`. The initial
state hash is checked in `settler_routes.rs`; the source tape was replayed with
every hash verified before extraction. The fixture is deterministic gzip JSON.

Switzerland' settler U72 selects the safe site `(2, 12)` but receives only the
first tile of its detour. On turns 150–152 it walks `(20, 11) -> (19, 11) ->
(18, 11) -> (20, 11)`. Each waypoint arrival clears `goto`, so site selection
loses its commitment bonus and chooses a different destination.

The test checks the recorded three-waypoint circuit, then replaces only P5's
orders with the current AI. V9 prices and yields change the resolution of the
other players' historical orders; its larger goal awards can also finish this
late-game state early. The route-only test clears the finished/winner fields
after each turn to preserve all five turns of coverage. Confirmed safe paths
must retain the settlement destination and make progress without revisiting
a tile, including save/load and fresh brains.
The full 453-turn tape, analysis, and 200-seed reports remain outside git.

Migrated 2026-10-05 (Monument and Assembly cut): `Assembly` was removed from the saved cities'
building lists (the state hash is recomputed at load; `Monument` loads as the Brood Hall).
