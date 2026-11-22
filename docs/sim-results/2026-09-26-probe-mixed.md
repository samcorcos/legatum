# Simulation: probe-mixed (2026-09-26)

500 games, seeds 5000.. (+i), brain `mixed`, turn limit 150, 16 threads, started 2026-09-26T22:24:05Z. Wall time 6.7s.

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP target | 407 | 81.4% |
| Conquest | 0 | 0.0% |
| Turn limit | 72 | 14.4% |
| Draw | 21 | 4.2% |
| Unfinished (runner stop) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 1334 | 168 | 12.6% |
| Japan | 1333 | 143 | 10.7% |
| Byzantium | 1333 | 168 | 12.6% |

## Win rate by brain

| Brain | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| random | 2000 | 1 | 0.1% |
| scripted | 2000 | 478 | 23.9% |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Game length (turns) | 64 | 87 | 131 | 150 | 150 |
| Winning team VP | 7 | 9 | 10 | 11 | 11 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 559 |
| 1 | 934 |
| 2 | 237 |
| 3 | 793 |
| 4 | 265 |
| 5 | 307 |
| 6 | 137 |
| 7 | 177 |
| 8 | 117 |
| 9 | 67 |
| 10 | 356 |
| 11 | 51 |

## End state

- Average game length: 123.2 turns
- Average cities per empire: 3.43
- Average techs per empire: 7.65
- Average eliminations per game: 0.08

## Throughput

- Turns simulated: 61618 (first play of each game)
- Aggregate: 9225.6 turns/s over 6.7s wall
- Per game thread: 584.0 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.
