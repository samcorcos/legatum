# Simulation: tuning-1-after (2026-09-26)

1000 games, seeds 1.. (+i), brain `scripted`, turn limit 150, 16 threads, started 2026-09-26T23:00:19Z. Wall time 24.6s.

## Outcomes

| Outcome | Games | Share |
| --- | --- | --- |
| VP target | 811 | 81.1% |
| Conquest | 0 | 0.0% |
| Turn limit | 129 | 12.9% |
| Draw | 60 | 6.0% |
| Unfinished (runner stop) | 0 | 0.0% |
| Panic | 0 | 0.0% |

## Win rate by faction

Per seat: wins / seats of that faction. With 8 single-empire teams, an even game is 12.5% per seat.

| Faction | Seats | Wins | Win rate |
| --- | --- | --- | --- |
| Switzerland | 2666 | 312 | 11.7% |
| Japan | 2667 | 296 | 11.1% |
| Byzantium | 2667 | 332 | 12.4% |

## Distributions

| Metric | Min | P10 | Median | P90 | Max |
| --- | --- | --- | --- | --- | --- |
| Game length (turns) | 59 | 90 | 130 | 150 | 150 |
| Winning team VP | 7 | 9 | 10 | 11 | 12 |

Final VP per empire (all seats):

| VP | Empires |
| --- | --- |
| 0 | 225 |
| 1 | 984 |
| 2 | 566 |
| 3 | 1570 |
| 4 | 912 |
| 5 | 1233 |
| 6 | 424 |
| 7 | 716 |
| 8 | 370 |
| 9 | 189 |
| 10 | 683 |
| 11 | 126 |
| 12 | 2 |

## End state

- Average game length: 126.3 turns
- Average cities per empire: 4.15
- Average techs per empire: 8.54
- Average eliminations per game: 0.04

## Throughput

- Turns simulated: 126308 (first play of each game)
- Aggregate: 5138.2 turns/s over 24.6s wall
- Per game thread: 325.7 turns/s

## Determinism

Not checked (run with `--determinism`).

## Panics

None.
