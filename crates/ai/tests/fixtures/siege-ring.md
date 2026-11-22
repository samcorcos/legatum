# Siege ring regression

`siege-ring-t139-t145.json.gz` holds two complete, unmodified pre-turn `GameState`s (start of
T139 and of T143), Hard India's (player 2) `OpponentMemory` at each of those points, and every
other seat's recorded orders for T139–T145. It is not the designer's save: it was extracted from
the tape of his 8.9.36 Celts game (autosave `autosave-t144.legatum`, diagnosed 2026-11-14) by
replaying the tape on the exact 8.9.36 crates with a fresh Hard brain for player 2 seeded with
`Rng::derive(setup.seed, [purpose::AI, 2]).next_u64()`. That replay matched the initial hash, every
resolved hash and every one of India's recorded orders on all 145 turns. The memory carries the
committed Strike on Celts Colony 7 (city 36 at (37,7)), which a fresh brain would not have.

On the current rules (34, special units) the tape still replays hash for hash up to T139, so the
two states are the real ones; later turns in the tests are counterfactuals. Seats other than
India replay their recorded orders where they are still legal (`validate_orders`), so the human
re-garrisons the city as he did, but an order the new position makes illegal is dropped.

The bug: the city sat at 0 HP from T138. On T139–T142 three wounded units attacked it each turn
while the 100-HP Cuirassier 234 walked between (35,8) and (35,9), two tiles out: the joint
assault only takes tiles a unit reaches this turn by a route that never crosses a friend, and
the free tiles beside the city were on the far side. From T143 (a fresh 78-HP Rifleman inside)
all of India's melee units skipped: the wounded failed `too_risky`, and the strike's staging
filter `safe_campaign_tile` rejects every tile beside a garrisoned city, so the healthy units
had nowhere to go. The tests assert that a healthy unit attacks or closes the ring on T143, that
not every roster melee unit skips on T143 and T144, and that the Cuirassier attacks or stands
beside the city during T139–T142.
