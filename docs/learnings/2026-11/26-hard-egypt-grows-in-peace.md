# Hard Egypt never grew: every unit it trains stops a Nile city's growth

2026-11-26. On the Nile a city that is building does not grow, so Egypt's growth competes with everything it trains. Hard Egypt's production let combat units and Settlers pass straight through `egypt_nile_choice`, and its army quota kept the capital on Spearmen from turn 1: in a recorded human Egypt seat it was still size 1 at turn 30 and had 11 population at turn 100, against 51 for a human who ran Grow from turn 1, kept 2 to 4 units for 145 turns, and bought a late army with a Gold hoard.

Fix (AI only): `egypt_peace_army` 100 for Egypt. A Hard Egyptian city out of sight of any enemy or Barbarian army grows instead of training a combat unit once the army is one unit per city. 300 paired six-seat Hard games (seed 5001): Egypt wins 14.7% to 53.3% (+38.7 pp ±6.2). `egypt_grow_to` 6 (grow to 6 before a Settler or unit) also helped (+30.3 pp) but less, because it delays Settlers.

Lesson: Egypt's AI win rate (about 15 to 23% before) understated the faction; the Nile is far stronger played for growth. Balance work should re-measure with this AI.
