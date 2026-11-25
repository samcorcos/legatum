# Advisor situational tips: ration by kind, and back off (2026-11-25)

A first-time playtester asked for advice like "build up your army or you might get attacked" and "buy a tile to get iron", with a reason, without repetitive advice. Simulating 120-turn games with the Hard plan in the seat (FixtureClient plus `advisor_orders` from the WASM package; the app's `advisorOrders` returns null in node until `loadWasmModule` runs, which silently made the seat passive) showed what repeats:

- A threat tip on any enemy unit near a city fired 13 times in one game, mostly for single barbarians the Hard plan rightly ignores. Requiring 2+ enemy units within 3 tiles, more than the guards, cut it to 4.
- Keying cooldowns per rival or per city let the same advice return under a new name ("Build up your army" for each stronger neighbour). The cooldown now applies to the kind, and each repeat doubles it, so advice the player chose not to follow fades.
- "Gold in the treasury does nothing" is false for Switzerland, whose Gold earns interest: the Gold tip skips the Swiss.

With these rules a game shows 5 to 10 tips in 120 turns, never the same kind twice within its cooldown.
