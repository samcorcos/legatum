# TI4 public objectives: how many are scored, and by how many players

Researched 2026-10-10 for Legatum's per-era objective target.

## Sources (every URL actually read)
- https://ti4stats.com/games?page=1..199 (game list, 26,326 games total) and, for each of 2,549 completed 6-player games, `https://ti4stats.com/games/<id>` (revealed Stage I/II cards) and `https://ti4stats.com/games/<id>/rounds` (per-round objectives each player scored). These are AsyncTI4 bot games, 2026-06-04 to 2026-10-08.
- https://carlcolglazier.com/notes/ti4-analysis/ (read only to confirm other TI4 analyses exist; it has no objective-scoring numbers).
- Rules facts (reveal pacing, VP values, card costs) are from memory, so approximate. The official rules PDF and the Fandom wiki were not read (Fandom returns 403).

## Method and sample
- n = 1,974 games: 6 players, 10-VP target, complete. 12-VP (276) and 14-VP (95) games were excluded.
- I parsed the HTML myself. Objective chips are typed Stage I, Stage II, secret, or other VP source. Public-objective scorers per card = distinct players who scored it.
- Check: summed component points match the player's final score exactly for 82% of 2,711 players (first 600 games) and within 1 VP for 94%. Treat the numbers as +-3% reliable.
- Bias: these are fast async games (5 rounds), and the game ends the moment someone hits 10 VP, so late cards had less time on the table.

## 1. Cards revealed in a typical game
- Rounds: mean 5.13, median 5 (4 rounds 11%, 5 rounds 65%, 6 rounds 22%, 7+ 2%).
- Revealed: Stage I mean 5.07, Stage II mean 1.24, total 6.3 (about rounds + 1.2). All 5 Stage I cards come out before any Stage II (Stage II count: 0 in 11% of games, 1 in 56%, 2 in 29%, 3+ in 3%).
- From memory (approx.): 2 Stage I are revealed at setup, then one card per Status Phase. 5 Stage I (1 VP) and 5 Stage II (2 VP) are used per game, so at most 10 public cards.

## 2. Share of revealed cards scored by at least one player
- All cards: 90.8%. Excluding the last-revealed card (it often gets no scoring window): 97.6%.
- Stage I: 98.5%. Stage II: 59.1% overall, but about 80% when it was not the last card revealed.
- Per game, the mean share of revealed cards that were scored was 91% (median 87.5%).

## 3. Scorers per card (of 6 players)
| Set | Mean | Median | Scored by 0 | Scored by all 6 |
| --- | --- | --- | --- | --- |
| Stage I | 4.19 (70%) | 4 | 1.5% | 21% |
| Stage II, all | 1.11 (18%) | 1 | 41% | 0.9% |
| Stage II, not last-revealed | 1.87 (31%) | 2 | about 20% | n/a |
| All public cards | 3.58 (60%) | 4 | 9% | 17% |
- All-card distribution of scorers 0 to 6: 9, 10, 10, 14, 19, 21, 17 percent.
- Time on the table matters. Stage I cards by reveal order: 4.7, 4.7, 4.5, 4.1, 3.2 scorers. The 6th Stage I (when it exists) gets 2.0 and a 2nd Stage II gets 0.5.
- Spread by card: Stage I from 2.75 (Found Research Outposts) to 5.25 (Engineer a Marvel). Stage II from 0.33 (Form Galactic Brain Trust) to 1.76 (Found a Golden Age); the five weakest Stage II cards are scored in only 27-45% of the games they appear in.
- Per player: mean 3.74 public objectives scored (median 4). Only 0.8% score none, 2.6% score one, and 64% score 4+.

## 4. Where points come from (mean VP per player)
| Source | Winner (avg 10.05 VP) | Every player (avg 7.05 VP) |
| --- | --- | --- |
| Public Stage I | 4.09 (41%) | 3.51 (50%) |
| Public Stage II (2 VP each; 0.60 cards for winners) | 1.20 (12%) | 0.46 (6%) |
| Secret objectives | 2.76 (27%) | 1.87 (27%) |
| Mecatol Custodians + Imperial card | 0.56 (6%) | 0.34 (5%) |
| Agenda-deck VP cards (Mutiny, Seed of an Empire, Imperial Rider, Political Censure) | 0.21 (2%) | 0.20 (3%) |
| Relics (Shard, Crown) | 0.26 (3%) | 0.12 (2%) |
| Support for the Throne (promissory) | 0.49 (5%) | 0.42 (6%) |
| Other chips (unclassified) | 0.38 (4%) | 0.18 (3%) |
- Public objectives are 53% of a winner's points (56% for all players). The winner scores 4.69 public cards against 3.74 for the average player, so public objectives are a floor everyone reaches and the winner's edge is about one extra card plus secrets.
- Caveat: agenda VP is probably undercounted. Some agenda results may sit in the unclassified chips.

## 5. How "hard but achievable" is built (data plus memory, approximate)
- Stage I (1 VP) is designed to be reachable by nearly everyone: 70% of players score a given card, 99% of cards get scored, and no card is out of reach. Typical costs are modest holdings or spends, such as 5 trade goods or 2 unit upgrades.
- Stage II (2 VP) costs more and is chosen by contest and position, not by being possible for everybody. A typical card is scored by about 2 of 6 players. Examples are 8 non-fighter ships in one system, or planets next to rivals' homes.
- The spread inside a tier is wide. The hardest Stage II cards are about 3x less scored than the easiest, and many players skip the hard cards and win on the easy ones. The Stage I -> Stage II step in scorers (4.2 -> 1.9) is roughly x2.2 for 2x the VP.
- The reveal is public, so players plan toward it. The cost is a planning cost (expanding or hoarding toward one condition), not a luck cost.

## Implication for Legatum
- Legatum's start target is about 50% of empires scoring each era's single objective. That sits between TI4 Stage I (70%) and Stage II (31% when it was on the table) and a little under the all-card blend (60%). It is a sound middle, not a hard card and not a gimme.
- Recommended band: 40-60% of empires score, overall mean about 50%. Ramp it: eras 1-3 at 55-65% (Stage I feel), eras 4-5 at 40-55%, eras 6-7 at 30-45% (Stage II feel, still scored by 2-3 of 8).
- Guardrails from TI4: at least one empire scores in 90% or more of eras (TI4 gets 91-98%), and at most about 15% of eras are scored by every empire (TI4 Stage I gets 21%, which leaves no differentiation). An era scored by 0 or by all is wasted.
- Window matters. A TI4 card stays scoreable for the rest of the game (3-4 rounds), and scoring rate fell from 4.7 to 3.2 as the window shrank. If a Legatum objective expires with its era, the same cost scores fewer empires. Set the rate by measuring it in sims, not by copying the TI4 cost.
- Check the weight. Public objectives are about 53% of TI4 winners' VP. Legatum has only 7 once-per-era cards, so each should carry a share of the 20-VP team target, with the 50% scoring rate producing most of the spread.
