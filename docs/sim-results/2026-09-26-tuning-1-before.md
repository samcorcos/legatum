# Simulation: tuning-1-before (2026-09-26)

1000 games, seeds 1.. (+i), brain `scripted`, turn limit 150, 16 threads, started 2026-09-26T22:50:22Z. Wall time 24.8s.

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP target | 790 | 79.0% |
| Conquest | 0 | 0.0% |
| Turn limit | 150 | 15.0% |
| Draw | 60 | 6.0% |
| Unfinished (runner stop) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 2666 | 337 | 12.6% |
| Japan | 2667 | 289 | 10.8% |
| Byzantium | 2667 | 314 | 11.8% |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Game length (turns) | 63 | 95 | 134 | 150 | 150 |
| Winning team VP | 6 | 9 | 10 | 11 | 12 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 155 |
| 1 | 1111 |
| 2 | 563 |
| 3 | 1755 |
| 4 | 917 |
| 5 | 1176 |
| 6 | 393 |
| 7 | 635 |
| 8 | 316 |
| 9 | 189 |
| 10 | 683 |
| 11 | 103 |
| 12 | 4 |

## End state

- Average game length: 129.1 turns
- Average cities per empire: 4.05
- Average techs per empire: 8.58
- Average eliminations per game: 0.04

## Throughput

- Turns simulated: 129101 (first play of each game)
- Aggregate: 5197.5 turns/s over 24.8s wall
- Per game thread: 327.6 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.
