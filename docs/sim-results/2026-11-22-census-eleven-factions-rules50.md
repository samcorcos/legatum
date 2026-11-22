# Eleven-faction census, rules 50 (2026-11-22)

Published as measured, with no tuning. The pillar-8 aim is that the best faction's win rate is at most 1.5x the worst's. **It is not met: the best/worst ratio is 9.1 (Egypt 21.4% over Rome 2.3%).** Without Rome it is 2.6 (Egypt over India, 8.3%). The previous census (2026-11-08, rules 25, nine factions) read 1.83.

## Setup

- Engine: master 22cb224 (RULES_VERSION 50) plus the census metrics in this change (they only read the final state; no rule or AI change).
- 8 empires, all Hard, Medium map, Standard speed (the only speed), default barbarians, `legatum-sim --players 8 --difficulty hard --rotate-factions`.
- Explicit rosters: the 11 cyclic windows of 8 over the faction order Switzerland, India, Byzantium, Germany, Celts, Mongols, Sparta, Athens, Aztecs, Egypt, Rome. Every faction is in 8 of the 11 rosters, and each roster rotates its factions one seat per game, so every faction played the same number of seats (N = 1,536 each) and every seat position equally often.
- 2 batches x 11 rosters x 96 games = **2,112 games** (seed bases 9311003 and 9411003, plus 100000 per roster). This is above the 2,000-game standard. Held-out seeds were not run.
- Raw per-game JSON is not committed (kept outside the repo).

## Win rate per seat (wins / seats the faction played; a win is the winning team's seat)

| Faction | Seats (N) | Wins | Win rate | 95% interval (Wilson) |
| --- | --- | --- | --- | --- |
| Egypt | 1,536 | 329 | 21.4% | 19.4 to 23.5 |
| Mongols | 1,536 | 287 | 18.7% | 16.8 to 20.7 |
| Athens | 1,536 | 223 | 14.5% | 12.8 to 16.4 |
| Germany | 1,536 | 219 | 14.3% | 12.6 to 16.1 |
| Celts | 1,536 | 216 | 14.1% | 12.4 to 15.9 |
| Aztecs | 1,536 | 197 | 12.8% | 11.2 to 14.6 |
| Sparta | 1,536 | 185 | 12.0% | 10.5 to 13.8 |
| Switzerland | 1,536 | 154 | 10.0% | 8.6 to 11.6 |
| Byzantium | 1,536 | 138 | 9.0% | 7.7 to 10.5 |
| India | 1,536 | 127 | 8.3% | 7.0 to 9.8 |
| Rome | 1,536 | 36 | 2.3% | 1.7 to 3.2 |

An even share in an 8-seat game is 12.5%. Best/worst 9.14 (interval 6.0 to 13.9, best's lower bound over worst's upper bound, and the reverse). Best over second-worst (India) 2.59. Each batch alone gave the same spread (8.9 and 9.4). Egypt and Rome are early designs without a faction goal; the nine finished factions alone span 8.3% to 21.4%, a ratio of 2.6.

## Length and ending (the new metrics)

| Metric | Value |
| --- | --- |
| Games / panics | 2,112 / 0 |
| End turn, median (mean) | 220 (225.2) |
| End turn, 10th / 90th percentile, min-max | 181 / 270, 139 to 1,200 |
| Ended by VP hold | 2,088 (98.9%) |
| Ended by capitals victory | 23 (1.1%) |
| Ended at the turn limit | 0 |
| Other (stalled at the safety cap) | 1 (0.05%) |
| Games where any empire reached the World War era | 2,095 of 2,112 (99.2%) |
| Furthest era at the end: Machine Age / World War / Modern / Future | 17 / 258 / 680 / 1,157 games |

The game's target is 250 to 300 turns. Measured, half of games end by turn 220 and 90% by turn 270, so games are shorter than the target but not by the 110 to 190 of the outside review. The late eras are played: the Future era was reached in 55% of games.

**Turn-cap games:** 1 of 2,112 ran to the 1,200-turn safety cap, seed 9811057 (roster 4, Celts, Mongols, Sparta, Athens, Aztecs, Egypt, Rome, Switzerland, game 54 of that roster). Not investigated here.
