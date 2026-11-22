# Pacing study, 2026-10-02 (CPU only)

Binary: master 4a54f0d. Runs: ~/LegatumRuns/pacing-1001/ (run.sh, summarize.py). P = capital-production + opening-ramp (G8 pacing); D = balance bundle D. A first run (invalid-run1/) used identical flags for every condition (macOS bash 3.2 has no associative arrays) and is discarded.

## 8-player, standard

| Condition | Ratio | Best | Worst | Mean turns | Stalls | 1st Settler (median) | 2nd city (median) | Cities at T50 (mean) | Seats with 3+ cities at T50 | Cities at T100 (mean) |
|---|---|---|---|---|---|---|---|---|---|---|
| d | 1.69x | Germany 16.6% | Japan 9.8% | 254 | 2 | 49 | 55 | 1.36 | 0% | 2.09 |
| p | 2.14x | Athens 16.3% | Germany 7.6% | 239 | 0 | 18 | 26 | 2.00 | 8% | 3.05 |
| pd | 1.96x | Athens 16.1% | Byzantium 8.2% | 236 | 3 | 18 | 26 | 2.00 | 8% | 3.05 |
| pd-nogermany | 2.10x | Sparta 17.4% | Germany 8.3% | 238 | 1 | 18 | 26 | 2.00 | 8% | 3.05 |
| pd-light | 2.25x | Sparta 16.9% | Japan 7.5% | 238 | 4 | 18 | 26 | 2.00 | 8% | 3.05 |

## 8-player, quick

| Condition | Ratio | Best | Worst | Mean turns | Stalls | 1st Settler (median) | 2nd city (median) | Cities at T50 (mean) | Seats with 3+ cities at T50 | Cities at T100 (mean) |
|---|---|---|---|---|---|---|---|---|---|---|
| d | 1.91x | Byzantium 17.4% | Japan 9.1% | 131 | 1 | 22 | 31 | 1.95 | 12% | 3.16 |
| p | 1.95x | Byzantium 17.7% | Germany 9.1% | 126 | 0 | 15 | 23 | 2.33 | 37% | 3.48 |
| pd | 1.78x | Germany 17.8% | Celts 10.0% | 124 | 0 | 15 | 23 | 2.33 | 37% | 3.46 |
| pd-nogermany | 1.74x | Athens 15.8% | Germany 9.1% | 128 | 0 | 15 | 23 | 2.33 | 37% | 3.46 |
| pd-light | 1.72x | Byzantium 17.5% | Germany 10.2% | 125 | 0 | 15 | 23 | 2.33 | 37% | 3.46 |

## Human-playbook v6 vs 7 scripted Hard (Standard, 400 games, seed 3610001)

- d: 104/400 (26.0%)
- pd: 90/400 (22.5%)
- pd-nogermany: 94/400 (23.5%)
- pd-light: 90/400 (22.5%)

## Opening duels (built-in genomes, Standard, small maps, 120 games per pair)

- d: GreedyExpand 0.73, Default 0.69, Snowball 0.49, Rush 0.45, Economic 0.15; Rush vs GreedyExpand 0.32; verdict NOT VALID
  - significant edges: Default>Snowball, Default>Economic, GreedyExpand>Rush, GreedyExpand>Snowball, GreedyExpand>Economic, Snowball>Economic
  - cities at T50 (mean; share with 3+): Default 1.60 (0%), Economic 1.00 (0%), GreedyExpand 1.73 (0%), Rush 1.00 (0%), Snowball 1.01 (0%)
  - first war turn (median): Default 181, Economic 302, GreedyExpand 304, Rush 83, Snowball 300
- pd: Default 0.76, GreedyExpand 0.75, Snowball 0.46, Rush 0.38, Economic 0.16; Rush vs GreedyExpand 0.28; verdict NOT VALID
  - significant edges: Default>Rush, Default>Snowball, Default>Economic, GreedyExpand>Rush, GreedyExpand>Snowball, GreedyExpand>Economic, Snowball>Rush, Snowball>Economic
  - cities at T50 (mean; share with 3+): Default 2.13 (19%), Economic 1.60 (0%), GreedyExpand 1.99 (1%), Rush 1.03 (0%), Snowball 1.75 (0%)
  - first war turn (median): Default 148, Economic 305, GreedyExpand 300, Rush 63, Snowball 300

## Greedy expand pace under P + D (duels, 120 games per pair, built-in genomes except as noted)

| Greedy expand variant | Cities at T50 (mean) | Games with 3+ cities at T50 | vs Default | vs Rush |
| --- | --- | --- | --- | --- |
| Built-in (settler_interval 15 Quick = 30 turns Standard) | 1.99 | 1% | 0.48 | 0.73 |
| settler_interval 0 | 2.66 | 68% | 0.55 | 0.77 |
| settler_interval 0, two Settlers back to back | 2.66 | 68% | 0.58 | 0.76 |

## Reading

- Pacing works as intended. At Standard the first Settler arrives on turn 18 instead of 49, the second city on 26 instead of 55, and AIs average 3.05 cities at turn 100 instead of 2.09. Games run 6% shorter (236-239 vs 254 turns), and the human-playbook benchmark drops from 26.0% to 22.5% (Hard gets harder for human-style play).
- It unbalances factions. Bundle D's 1.69x becomes 1.96x at Standard under pacing; the variants without Germany's or Byzantium's levers are worse (2.10-2.25x). Balance needs retuning for the paced game.
- The Civ 6 benchmark (3 cities by turn 50) is reachable: with pacing and no Settler interval, Greedy expand averages 2.66 cities at T50 and has 3+ in 68% of duels. The built-in Greedy expand was held back by its own 30-turn Settler interval.
- Greedy expand still isn't punished: with pacing it beats Rush 72-77%. Rush declares earlier (median T63 vs T83) but loses more. Counters (Rush build, barbarian pressure on unescorted settlers, vulnerable new cities) are the next lever set to test.
