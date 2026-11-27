# Egypt army Food: eleven-faction Hard census (2026-11-27)

2,000 games across 11 cyclic eight-faction rosters, all scripted Hard. Every faction appears in eight rosters. Each roster rotates its eight factions among seat positions; eight rosters ran 184 games and three ran 176, all in multiples of eight. Medium map, standard rules, seed base 9311003 plus 100000 per roster, six threads. The default Egyptian rule charges 3 Food per eligible land military unit. Raw per-roster JSON remains outside the repository.

**Best/worst: Germany and the Celts tie at 17.0%; Rome is 3.8%, for a 4.47× ratio.** The 1.5× balance target is not met.

| Faction | Seats | Wins | Win rate | 95% Wilson interval |
| --- | ---: | ---: | ---: | ---: |
| Germany | 1448 | 246 | 17.0% | 15.1–19.0% |
| Celts | 1448 | 246 | 17.0% | 15.1–19.0% |
| Egypt | 1456 | 237 | 16.3% | 14.5–18.3% |
| Athens | 1472 | 226 | 15.4% | 13.6–17.3% |
| Sparta | 1464 | 208 | 14.2% | 12.5–16.1% |
| Byzantium | 1448 | 182 | 12.6% | 11.0–14.4% |
| Aztecs | 1464 | 176 | 12.0% | 10.5–13.8% |
| India | 1448 | 161 | 11.1% | 9.6–12.8% |
| Switzerland | 1448 | 150 | 10.4% | 8.9–12.0% |
| Mongols | 1456 | 108 | 7.4% | 6.2–8.9% |
| Rome | 1448 | 55 | 3.8% | 2.9–4.9% |

## Run details

| Roster | Games | First seed | Factions | Panics | Stalls |
| ---: | ---: | ---: | --- | ---: | ---: |
| 0 | 184 | 9311003 | Switzerland, India, Byzantium, Germany, Celts, Mongols, Sparta, Athens | 0 | 0 |
| 1 | 184 | 9411003 | India, Byzantium, Germany, Celts, Mongols, Sparta, Athens, Aztecs | 0 | 0 |
| 2 | 184 | 9511003 | Byzantium, Germany, Celts, Mongols, Sparta, Athens, Aztecs, Egypt | 0 | 0 |
| 3 | 184 | 9611003 | Germany, Celts, Mongols, Sparta, Athens, Aztecs, Egypt, Rome | 0 | 0 |
| 4 | 184 | 9711003 | Celts, Mongols, Sparta, Athens, Aztecs, Egypt, Rome, Switzerland | 0 | 0 |
| 5 | 184 | 9811003 | Mongols, Sparta, Athens, Aztecs, Egypt, Rome, Switzerland, India | 0 | 0 |
| 6 | 184 | 9911003 | Sparta, Athens, Aztecs, Egypt, Rome, Switzerland, India, Byzantium | 0 | 0 |
| 7 | 184 | 10011003 | Athens, Aztecs, Egypt, Rome, Switzerland, India, Byzantium, Germany | 0 | 0 |
| 8 | 176 | 10111003 | Aztecs, Egypt, Rome, Switzerland, India, Byzantium, Germany, Celts | 0 | 0 |
| 9 | 176 | 10211003 | Egypt, Rome, Switzerland, India, Byzantium, Germany, Celts, Mongols | 0 | 0 |
| 10 | 176 | 10311003 | Rome, Switzerland, India, Byzantium, Germany, Celts, Mongols, Sparta | 0 | 0 |

A win rate is wins divided by all seats of that faction, including eliminated seats and drawn games. The eight-seat even rate is 12.5%. The 300-game Egypt ladder at seed 5001 measured 25.7%; its six-seat format has a different even rate and is reported separately in `2026-11-26-egypt-army-food-ladder.md`.
