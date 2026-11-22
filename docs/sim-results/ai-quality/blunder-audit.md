# Blunder audit

Run: salts 1–64 on each checked-in 8.3 and 8.6 playtest map, using the real `legatum-core` game loop and Hard AI in the human seat. This covered 192 games and 54,953 live human-seat turns. The full machine-readable per-map, per-seat counts, rates, and examples are in [blunder-audit.json](blunder-audit.json). The engine/AI base was `42422c36`; `master` advanced to `e04bc968` during the task and was merged afterward. These measurements were not rerun on the merged revision, per the requested post-merge gate of determinism and pin checks only.

The requested Graviton batch was unavailable: the remote build box had exceeded its daily cap, and the local Docker fallback could not write Docker buildx activity state (`operation not permitted`). I ran the complete batch locally with `tools/sim/slot.sh` instead. Process CPU time was 12,193.22 seconds total, **63.51 CPU seconds per game**; this includes all Rayon workers. The JSON reports the exact measurement.

## Rates

Each class cell is `count (events per 100 live seat-turns)`. Seat 0 is the Hard seat replacing the human; the remaining numbered seats are the map’s AI opponents. Rates use each seat’s own live-turn denominator.

| Map | Seat | Role | Turns | Exposed Settler | Low-chance attack loss | Idle army at war | Abandoned siege | Unguarded city lost |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| playtest-8.3-switzerland-win | 0 | Hard human seat | 15057 | 243 (1.61) | 617 (4.10) | 164 (1.09) | 32 (0.21) | 29 (0.19) |
| playtest-8.3-switzerland-win | 1 | AI opponent | 15057 | 45 (0.30) | 211 (1.40) | 242 (1.61) | 26 (0.17) | 0 (0.00) |
| playtest-8.6-germany-loss | 0 | Hard human seat | 19324 | 277 (1.43) | 273 (1.41) | 650 (3.36) | 50 (0.26) | 16 (0.08) |
| playtest-8.6-germany-loss | 1 | AI opponent | 19366 | 169 (0.87) | 684 (3.53) | 752 (3.88) | 150 (0.77) | 37 (0.19) |
| playtest-8.6-germany-loss | 2 | AI opponent | 19366 | 139 (0.72) | 274 (1.41) | 523 (2.70) | 65 (0.34) | 33 (0.17) |
| playtest-8.6-germany-loss | 3 | AI opponent | 19366 | 463 (2.39) | 608 (3.14) | 676 (3.49) | 145 (0.75) | 45 (0.23) |
| playtest-8.6-sparta-win | 0 | Hard human seat | 20572 | 59 (0.29) | 206 (1.00) | 459 (2.23) | 6 (0.03) | 1 (0.00) |
| playtest-8.6-sparta-win | 1 | AI opponent | 20570 | 118 (0.57) | 262 (1.27) | 477 (2.32) | 31 (0.15) | 54 (0.26) |
| playtest-8.6-sparta-win | 2 | AI opponent | 20572 | 220 (1.07) | 114 (0.55) | 651 (3.16) | 68 (0.33) | 12 (0.06) |
| playtest-8.6-sparta-win | 3 | AI opponent | 20572 | 116 (0.56) | 323 (1.57) | 644 (3.13) | 29 (0.14) | 6 (0.03) |

## Rule-based definitions

- **Exposed Settler:** At each turn boundary, count each Settler whose owner is at war with a combat unit within the enemy unit’s full movement points by hex distance, with no friendly combat unit on or adjacent to the Settler. This is a geometric threat proxy; terrain, occupancy and path blockers are ignored.
- **Low-chance attack loss:** Count a unit attacker absent after resolution when the core `forecast_attack` forecast against the battle tile, from the attacker’s pre-turn player view, gives the attacker less than 30% win chance.
- **Idle army at war:** Count one event when a seat first has at least three combat units, each idle for three consecutive turns (neither moved nor attacked), while at war and with at least one qualifying unit within six hexes of an at-war enemy unit or city. A unit’s streak resets when it moves or attacks; peace resets that owner’s streaks. A condition that later clears can qualify again.
- **Abandoned siege:** Count a moving unit that leaves an at-war enemy city’s adjacent ring while pre-turn city HP is strictly below half of `CITY_HP`.
- **Unguarded city lost:** At `CityCaptured`, count the former city owner when the pre-turn state has no friendly combat unit on the city tile and has a combat unit within three hexes whose base movement points cover that distance in one turn. Terrain, occupancy and path blockers are ignored.

Each class retains at most five `(salt, turn)` examples per map and seat in the JSON. Barbarians are excluded from seat counts. All events are observations of states and events returned by the core; this example does not implement game rules.

## Next gene task

**Idle army at war — improve campaign engagement so three-unit armies at war do not leave nearby enemy forces or cities idle for three turns.** Across the Hard seat, this class led at 1273 events / 54953 live seat-turns (2.32 per 100). This is the next gene task; this audit change makes no AI tuning changes.
