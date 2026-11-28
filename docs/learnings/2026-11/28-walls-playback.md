# Walls playback uses the city damage event, not city HP

`rules/battle.rs::apply_fights` reports a city hit's actual Walls loss + HP loss + carried damage in `Battle.damage_to_defender`. `defender_hp_after` is the city HP result, not its Walls bar. The UI therefore consumes event damage from the visible pre-turn Walls balance on a separate `w<city>` playback track; it never subtracts that damage from city HP a second time.

A breach resets the resulting view's Walls maximum to zero, so playback keeps the visible pre-turn maximum until the bar drains. The fracture uses the town artwork's ground anchor rather than the hex centre above its roofs. Same-level repairs trigger the existing light sweep and refill from the balance after combat; a lingering combat animation must not mask the later production payoff. Animations off and reduced motion continue to show the final view immediately through the existing MapCanvas gate.

Validation: focused playback tests cover successive hits, breach-to-HP transition, hidden cities, same-level repairs and combat followed by repair. Chromium review frames use the real GameScreen at 1000x600 and 1440x900 with synthetic public-view playback fixtures; they are rendering checks, not a new simulation result.

Review correction: mixed projectile speeds can reverse event order on screen. Sort city hits by impact time, then original event index, before accumulating Walls loss or selecting the breach impact; sorting already-accumulated values can make the bar refill. Regression coverage includes both surviving Walls and a breach in a Catapult/Crossbowman volley.
