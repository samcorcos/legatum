# Simulation: tuning-1-production (2026-09-26)

1000 games, seeds 1.. (+i), brain `scripted`, turn limit 150, 16 threads, started 2026-09-26T22:52:55Z. Wall time 44.5s.

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP target | 812 | 81.2% |
| Conquest | 0 | 0.0% |
| Turn limit | 119 | 11.9% |
| Draw | 69 | 6.9% |
| Unfinished (runner stop) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 2666 | 316 | 11.9% |
| Japan | 2667 | 289 | 10.8% |
| Byzantium | 2667 | 326 | 12.2% |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Game length (turns) | 59 | 90 | 130 | 150 | 150 |
| Winning team VP | 8 | 9 | 10 | 11 | 12 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 218 |
| 1 | 995 |
| 2 | 572 |
| 3 | 1557 |
| 4 | 926 |
| 5 | 1234 |
| 6 | 417 |
| 7 | 704 |
| 8 | 379 |
| 9 | 186 |
| 10 | 681 |
| 11 | 126 |
| 12 | 5 |

## End state

- Average game length: 126.4 turns
- Average cities per empire: 4.15
- Average techs per empire: 8.55
- Average eliminations per game: 0.04

## Throughput

- Turns simulated: 126389 (first play of each game)
- Aggregate: 2840.1 turns/s over 44.5s wall
- Per game thread: 182.1 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.
