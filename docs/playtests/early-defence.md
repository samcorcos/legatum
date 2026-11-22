# Early defence, T1 to T60: what he actually does

The missing chapter of the playbook, taken from his recorded games, not from what seems sensible. It answers the
gaps in `docs/sim-results/ai-quality/fable-replicate-1115/README.md` (branch `claude/fable-replicate`, section 2):
a model seat played Switzerland from the playbook on seed 20261115 and lost every city to Sparta by T42.

The chapter itself is at the end ("The chapter"), followed by the questions the tapes cannot answer.

## Evidence and method

- **Games read: 45**, every game of his on build 8.9.2 or later (2026-10-17 to 2026-11-15) with at least 20 turns,
  replayed hash-exact (0 mismatches) from the app archive (`~/LegatumData/archive`) with `game_dump` built at each
  game's recorded engine commit (23 engine versions, 8.9.2 to 8.9.40), turns 1 to 60. Rules versions differ across
  them; Walls stopped being buyable from 8.9.4, so the 8.9.2 Walls purchase below is an old-rules case.
- **Not read:** the 8.9.17 Mongols game (its archive starts at T96); one 8.9.22 Aztecs game abandoned at T83 (its
  tape drifts from T2 at the recorded engine); 20 stubs under 20 turns, most of 0 to 5 (start re-rolls: 11 Aztecs on 2026-11-01, 4 Germany). The ten 8.4 to
  8.6 games of early October replay too but are left out of the counts (2 to 4 seats, a different rule set). They
  show the same pattern: in the 7 builder games Granary, Farm or a unit first, the first Settler on T15 to T22, no
  capital Walls before T42 and 0 or 1 military unit built by T30; in the 3 Sparta games, Soldiers first.
- **Distances** are hexes between capitals on the real map, not what he had seen at the time. "Rush faction" means
  Sparta or the Mongols; "builder" means any faction other than Sparta, the Aztecs and the Mongols (29 games; the
  war factions are 16).
- **Gold** is the treasury after each turn. "Windfalls" are ruins, natural-wonder discoveries, barbarian camps and
  capture bounties; "net income" is the rest of the change (city Gold, tolls, interest, less upkeep).
- **An early attack** is any turn up to T60 with two or more hostile military units (at War with him, or
  Barbarians) within 2 hexes of one of his cities.
- Out of 45: 27 won, 2 lost, 16 abandoned (14 of those before T100).

## 1. Walls or Settlers

**Finding.** Two openings, and which one he plays changed on 2026-11-07, the day Sparta took Zürich from him and he
wrote down his defence plan ("1) get walls up in the capital ...", `human-reasoning.md`).

- **Builders since 8.9.27 (6 games):** capital Walls first or second (after one Granary or Soldier), done T9 to T18,
  then Settlers, in all 5 games with Sparta's or the Mongols' capital 9 to 20 hexes away. The sixth (8.9.40
  Byzantium, Mongols 27 and Sparta 34 hexes away) opened Horseman, Settler, Settler, Settler and walled the capital
  on T40, which is what he said about that game: "sparta was pretty far away, so I wasn't at risk of an early
  attack. that let me focus on building population, science, and gold."
- **Builders before 8.9.27 (23 games):** Granary, Farm, Pasture or Quarry first (20), then Settlers (first one
  T14 to T35, median T19); capital Walls by T25 in 7, later (T27 to T61) or never by T60 in 16. A unit came
  before the first Settler in 6 of the 25 builder games that trained one. "Granary, Archer, three Settlers" is
  this older opening; the Archer is not a fixed part of it.
- **Distance to the rush capital** (all 29 builder games): capital walled by T25 in 7 of 12 games at 9 hexes or
  less, 4 of 10 at 10 to 14, 1 of 7 at 15 or more (medians 9 hexes with early Walls, 12 without). It leans, it
  does not decide: the date decides more. No distance separates games with early defenders, because builders train
  0 to 2 military units by T30 at every distance (0 in 17 of 29).
- **War factions (16):** Sparta and the Aztecs queue three to seven Soldiers on T2 and train the first Settler on
  T26 to T51, or none before T60 (5 Sparta games and 1 Aztec game grew by capture); capital Walls T34 to T52 or
  never. The Mongols (3) open with a Horseman or a Quarry, then Settlers, with no Walls by T60.

**Ranking of the two playbook lines:** "capital: Walls first" is what he does now whenever a rush capital is within
about 20 hexes (5 of 5 since 8.9.27). "Granary, Archer, three Settlers" is what he did before 2026-11-07 and what he
still does when the rush faction is far (1 game). Walls first wins on the fable-replicate map (Sparta 10 hexes away).

"Nearest aggressive" also counts the Celts and the Aztecs. Distances are in hexes from his capital.

| Game | Nearest Sparta/Mongols capital | Nearest aggressive | Capital, first four completed | First Settler | Capital Walls | Military built by T20 / T30 |
|---|---|---|---|---|---|---|
| 8.9.2 Switzerland (8p, abandoned T220) | Mongols 11 | Mongols 11 | Granary, Market, Settler, Ranger | T35 | none by T60 | 0 / 0 |
| 8.9.2 Switzerland (8p, won) | Mongols 12 | Mongols 12 | Granary, Settler, Ranger, Settler | T17 | T61 | 0 / 1 |
| 8.9.2 Sparta (8p, abandoned T86) | Mongols 7 | Mongols 7 | Soldier, Soldier, Soldier, Soldier | - | T52 | 3 / 6 |
| 8.9.2 Sparta (8p, abandoned T42) | Mongols 15 | Mongols 15 | Soldier, Soldier, Soldier, Soldier | - | none by T42 | 2 / 5 |
| 8.9.4 Sparta (4p, won) | Mongols 7 | Mongols 7 | Soldier, Soldier, Soldier, Soldier | - | none by T40 | 5 / 9 |
| 8.9.4 Switzerland (8p, abandoned T67) | Sparta 11 | Celts 6 | Quarry, Settler, Granary, Market | T17 | T36 | 0 / 0 |
| 8.9.4 Switzerland (8p, won) | Mongols 11 | Mongols 11 | Scout, Settler, Horseman, Settler | T24 | none by T60 | 0 / 0 |
| 8.9.5 Athens (8p, abandoned T56) | Sparta 8 | Sparta 8 | Farm, Farm, Walls, Ranger | - | T20 | 0 / 1 |
| 8.9.5 Athens (8p, lost) | Sparta 17 | Celts 13 | Granary, Settler, Settler | T31 | none by T60 | 0 / 0 |
| 8.9.6 Athens (8p, won) | Mongols 6 | Mongols 6 | Pasture, Granary, Settler, Settler | T22 | T42 | 0 / 0 |
| 8.9.6 Mongols (8p, abandoned T92) | Sparta 9 | Sparta 9 | Quarry, Settler, Soldier, Market | T20 | none by T60 | 0 / 1 |
| 8.9.6 Switzerland (8p, abandoned T45) | Mongols 7 | Mongols 7 | Granary, Walls, Settler, Settler | T22 | T15 | 0 / 0 |
| 8.9.6 Switzerland (8p, won) | Sparta 7 | Sparta 7 | Granary, Farm, Quarry, Walls | T33 | T20 | 0 / 1 |
| 8.9.7 Celts (4p, won) | Sparta 18 | Sparta 18 | Farm, Granary, Settler, Settler | T17 | T33 | 0 / 0 |
| 8.9.10 Sparta (4p, won) | none | Celts 7 | Soldier, Soldier, Soldier, Soldier | T47 | T46 | 2 / 4 |
| 8.9.10 Japan (4p, abandoned T60) | Sparta 7 | Sparta 7 | BroodHall, Granary, Settler, Settler | T15 | T27 | 0 / 0 |
| 8.9.10 Athens (8p, won) | Mongols 14 | Celts 10 | Granary, Farm, Settler, Walls | T16 | T21 | 0 / 0 |
| 8.9.11 Japan (5p, won) | Sparta 21 | Sparta 21 | BroodHall, Settler, Settler, Walls | T18 | T35 | 0 / 0 |
| 8.9.13 Athens (5p, won) | Sparta 10 | Celts 10 | Granary, Walls, Ranger, Settler | T31 | T14 | 0 / 1 |
| 8.9.13 Sparta (5p, abandoned T125) | none | none | Soldier, Soldier, Soldier, Soldier | - | T42 | 2 / 4 |
| 8.9.15 Switzerland (4p, won) | Sparta 9 | Sparta 9 | Farm, Walls, Settler, Market | T29 | T23 | 0 / 0 |
| 8.9.20 Aztecs (8p, abandoned T52) | Mongols 12 | Celts 7 | Soldier, Soldier, Soldier, Ranger | T51 | none by T52 | 2 / 3 |
| 8.9.22/23 Aztecs (8p, won) | Mongols 7 | Mongols 7 | Soldier, Soldier, Soldier, Soldier | T50 | none by T60 | 2 / 4 |
| 8.9.22/23 Aztecs (8p, won) | Sparta 8 | Sparta 8 | Soldier, Soldier, Soldier, Soldier | - | T34 | 2 / 4 |
| 8.9.24 Switzerland (8p, abandoned T23) | Sparta 15 | Celts 8 | Settler | T19 | none by T23 | 0 / 0 |
| 8.9.24 Switzerland (5p, won) | Mongols 7 | Mongols 7 | Granary, Settler, Settler, Ranger | T16 | none by T60 | 0 / 0 |
| 8.9.25 Celts (6p, won) | Sparta 12 | Aztecs 12 | Pasture, Settler, Settler, Ranger | T14 | T38 | 0 / 0 |
| 8.9.25 India (5p, won) | Mongols 9 | Mongols 9 | Granary, Settler, Settler, Settler | T14 | T45 | 0 / 1 |
| 8.9.25 Germany (6p, abandoned T22) | Sparta 7 | Sparta 7 | Farm, Granary, Walls | - | T14 | 0 / 0 |
| 8.9.26 Switzerland (6p, abandoned T30) | Sparta 12 | Aztecs 7 | Granary, Settler, Walls | T16 | T27 | 0 / 2 |
| 8.9.26 Switzerland (6p, won) | Sparta 16 | Sparta 16 | Ranger, Settler, Settler, Walls | T19 | T38 | 1 / 1 |
| 8.9.26 Sparta (6p, won) | Mongols 8 | Mongols 8 | Soldier, Soldier, Soldier, Soldier | T40 | none by T60 | 3 / 5 |
| 8.9.26 Celts (6p, won) | Sparta 8 | Sparta 8 | Granary, Quarry, Ranger, Soldier | T24 | none by T60 | 2 / 2 |
| 8.9.27 Germany (6p, won) | Mongols 9 | Mongols 9 | Granary, Walls, Settler, Workshop | T15 | T10 | 0 / 0 |
| 8.9.29 Aztecs (6p, won) | Sparta 8 | Sparta 8 | Soldier, Soldier, Soldier, Soldier | T49 | T45 | 3 / 5 |
| 8.9.30 Switzerland (6p, won) | Mongols 9 | Mongols 9 | Soldier, Walls, Settler, Settler | T24 | T18 | 1 / 1 |
| 8.9.31 Sparta (6p, lost) | none | Aztecs 14 | Soldier, Soldier, Soldier, Soldier | T43 | none by T60 | 3 / 5 |
| 8.9.31 Sparta (6p, abandoned T27) | Mongols 7 | Mongols 7 | Soldier, Soldier, Soldier, Soldier | - | none by T27 | 2 / 4 |
| 8.9.31 Mongols (6p, abandoned T41) | Sparta 11 | Aztecs 9 | Horseman, Settler, Settler | T16 | none by T41 | 1 / 1 |
| 8.9.31 Mongols (6p, won) | none | Celts 24 | Horseman, Settler, Settler, Settler | T16 | none by T60 | 1 / 1 |
| 8.9.32 Athens (6p, won) | Sparta 12 | Sparta 12 | Walls, Laboratory, Settler, Settler | T20 | T9 | 0 / 0 |
| 8.9.32.1 Aztecs (6p, won) | Sparta 9 | Sparta 9 | Soldier, Soldier, Soldier, Settler | T26 | T51 | 2 / 3 |
| 8.9.34 Athens (6p, won) | Sparta 20 | Sparta 20 | Walls, Settler, Horseman, Settler | T18 | T12 | 0 / 1 |
| 8.9.35.1 Byzantium (6p, abandoned T24) | Sparta 11 | Sparta 11 | Soldier, Mine, Walls, Settler | T21 | T17 | 1 / 1 |
| 8.9.40 Byzantium (6p, won) | Mongols 27 | Celts 24 | Horseman, Settler, Settler, Settler | T15 | T40 | 1 / 1 |

## 2. Gold by T30

**Finding.** About 500 Gold by T30 is not his norm. Median treasury 117 at T10, 189 at T20, 219 at T30 (41 games)
and 270 at T40. Only 2 of 41 games reached 500 by T30 (8.9.26 Switzerland 512, 8.9.31 Mongols 521) and 6 reached
400. Switzerland is not much richer this early: median 230 at T30 over 11 games, top 512. The 500-plus figures in
the reviews are from T50 on.

Where it comes from: windfalls are about half of it (median 120 by T30, 47% of the T30 treasury): ruins at 40 each
and natural-wonder discoveries at 80. The rest is income, which is small before T30 (median about 40 over 30
turns; Swiss tolls and interest lift it to 76 to 186). Tribute and sales do not appear: no Gold came from treaties or
selling in any game before T60.

What he skips to get there: everything. He bought nothing before T30 in 43 of 45 games (the exceptions: a Ranger in
Zürich on T27 in the 8.9.26 Swiss loss, and a hired Soldier in an 8.9.35.1 Byzantium game abandoned at T24). One
Settler was bought before T60 in 45 games (8.9.13 Sparta, T50). From T30 to T60 he made 20 purchases in 12 games:
16 military units, most of them at a city an army or a barbarian raid was walking at (Q4), plus two Walls under the 8.9.2 rules, the
Settler and one Roadworks; the rest of his Gold went on upgrades.

| Game | Gold T10 / T20 / T30 / T40 | To T30: windfalls | To T30: spent | To T30: net income | Windfall sources to T60 |
|---|---|---|---|---|---|
| 8.9.2 Switzerland (8p, abandoned T220) | 161 / 198 / 273 / 373 | 80 | 0 | 132 | wonder discovery 80 |
| 8.9.2 Switzerland (8p, won) | 117 / 147 / 196 / 279 | 40 | 0 | 95 | ruins 40 |
| 8.9.2 Sparta (8p, abandoned T86) | 190 / 205 / 348 / 459 | 131 | 0 | 76 | wonder discovery 80, ruins 40, CityCaptured 73 |
| 8.9.2 Sparta (8p, abandoned T42) | 110 / 114 / 158 / 193 | 40 | 0 | 57 | ruins 40 |
| 8.9.4 Sparta (4p, won) | 227 / 280 / 428 / 518 | 200 | 0 | 167 | wonder discovery 160, ruins 40 |
| 8.9.4 Switzerland (8p, abandoned T67) | 120 / 152 / 226 / 187 | 40 | 0 | 125 | ruins 40 |
| 8.9.4 Switzerland (8p, won) | 199 / 330 / 390 / 498 | 200 | 0 | 129 | ruins 40, wonder discovery 160 |
| 8.9.5 Athens (8p, abandoned T56) | 70 / 86 / 121 / 56 | 0 | 0 | 60 | - |
| 8.9.5 Athens (8p, lost) | 150 / 240 / 250 / 265 | 160 | 0 | 29 | wonder discovery 160, camps 50 |
| 8.9.6 Athens (8p, won) | 150 / 160 / 176 / 206 | 80 | 0 | -45 | wonder discovery 80 |
| 8.9.6 Mongols (8p, abandoned T92) | 190 / 200 / 218 / 292 | 120 | 0 | -3 | ruins 80, wonder discovery 80 |
| 8.9.6 Switzerland (8p, abandoned T45) | 78 / 109 / 150 / 209 | 0 | 0 | 89 | - |
| 8.9.6 Switzerland (8p, won) | 130 / 170 / 219 / 283 | 40 | 0 | 76 | ruins 40 |
| 8.9.7 Celts (4p, won) | 150 / 280 / 261 / 271 | 240 | 0 | -40 | wonder discovery 160, ruins 80 |
| 8.9.10 Sparta (4p, won) | 110 / 114 / 312 / 330 | 40 | 0 | 211 | ruins 40 |
| 8.9.10 Japan (4p, abandoned T60) | 110 / 240 / 167 / 197 | 160 | 0 | -94 | ruins 80, wonder discovery 80 |
| 8.9.10 Athens (8p, won) | 190 / 201 / 221 / 268 | 120 | 0 | 40 | wonder discovery 80, ruins 40 |
| 8.9.11 Japan (5p, won) | 110 / 160 / 180 / 208 | 80 | 0 | 39 | ruins 80 |
| 8.9.13 Athens (5p, won) | 70 / 120 / 170 / 187 | 80 | 0 | 29 | ruins 120 |
| 8.9.13 Sparta (5p, abandoned T125) | 70 / 76 / 65 / 279 | 0 | 0 | 4 | CityCaptured 162 |
| 8.9.15 Switzerland (4p, won) | 288 / 340 / 446 / 301 | 240 | 0 | 145 | ruins 160, wonder discovery 80 |
| 8.9.20 Aztecs (8p, abandoned T52) | 190 / 237 / 290 / 311 | 210 | 0 | 19 | ruins 120, wonder discovery 80, camps 50 |
| 8.9.22/23 Aztecs (8p, won) | 70 / 75 / 68 / 59 | 0 | 0 | 7 | - |
| 8.9.22/23 Aztecs (8p, won) | 110 / 155 / 145 / 142 | 80 | 0 | 4 | ruins 80, camps 50 |
| 8.9.24 Switzerland (8p, abandoned T23) | 165 / 202 / - / - | 80 | 0 | 20 | ruins 80 |
| 8.9.24 Switzerland (5p, won) | 276 / 332 / 487 / 625 | 240 | 0 | 186 | wonder discovery 160, ruins 80 |
| 8.9.25 Celts (6p, won) | 110 / 284 / 414 / 469 | 200 | 0 | 153 | wonder discovery 80, ruins 120 |
| 8.9.25 India (5p, won) | 210 / 240 / 270 / 400 | 120 | 0 | 87 | wonder discovery 160, ruins 40 |
| 8.9.25 Germany (6p, abandoned T22) | 70 / 118 / - / - | 40 | 0 | 17 | ruins 40 |
| 8.9.26 Switzerland (6p, abandoned T30) | 158 / 193 / 36 / - | 120 | 231 | 86 | wonder discovery 80, ruins 40 |
| 8.9.26 Switzerland (6p, won) | 282 / 327 / 512 / 624 | 320 | 0 | 131 | ruins 160, wonder discovery 160 |
| 8.9.26 Sparta (6p, won) | 70 / 151 / 133 / 230 | 80 | 0 | -8 | wonder discovery 80, ruins 40, camps 50 |
| 8.9.26 Celts (6p, won) | 150 / 160 / 172 / 229 | 80 | 0 | 31 | wonder discovery 80 |
| 8.9.27 Germany (6p, won) | 109 / 119 / 135 / 223 | 40 | 0 | 35 | ruins 40 |
| 8.9.29 Aztecs (6p, won) | 110 / 152 / 237 / 318 | 182 | 0 | -6 | ruins 120, CityCaptured 211 |
| 8.9.30 Switzerland (6p, won) | 159 / 189 / 230 / 313 | 80 | 0 | 89 | wonder discovery 160 |
| 8.9.31 Sparta (6p, lost) | 110 / 110 / 114 / 83 | 60 | 0 | -7 | ruins 40, CityCaptured 27 |
| 8.9.31 Sparta (6p, abandoned T27) | 70 / 154 / - / - | 80 | 0 | 13 | wonder discovery 80, ruins 120 |
| 8.9.31 Mongols (6p, abandoned T41) | 190 / 204 / 285 / 415 | 120 | 0 | 24 | wonder discovery 80, ruins 40 |
| 8.9.31 Mongols (6p, won) | 190 / 442 / 521 / 597 | 400 | 0 | -20 | ruins 240, wonder discovery 160 |
| 8.9.32 Athens (6p, won) | 110 / 200 / 216 / 244 | 120 | 0 | -5 | wonder discovery 80, ruins 120 |
| 8.9.32.1 Aztecs (6p, won) | 69 / 194 / 234 / 245 | 160 | 0 | 14 | wonder discovery 80, ruins 120 |
| 8.9.34 Athens (6p, won) | 70 / 200 / 218 / 245 | 120 | 0 | 37 | wonder discovery 80, ruins 40 |
| 8.9.35.1 Byzantium (6p, abandoned T24) | 69 / 7 / - / - | 0 | 80 | 27 | - |
| 8.9.40 Byzantium (6p, won) | 69 / 80 / 126 / 248 | 0 | 0 | 66 | - |

## 3. Army at home

**Finding.** His first unit leaves the capital on turn 1 and spends most of T1 to T60 outside it in 42 of 45 games
(median farthest point 9 hexes): exploring, clearing barbarian camps, escorting Settlers and taking AI Settlers
(Q6). "The capital always has a garrison" is not his play: the capital had a military unit in it at T20 in 3 of 45
games, at T30 in 8 of 41, at T40 in 10 of 40. Builders own one or two military units to T40 (0 built by T30 in 17
of 29); the capital's defence is its Walls. War factions build 3 to 9 by T30 and keep them in the field.

| Game | Military units T10 / T20 / T30 / T40 (where) | First unit: turns outside the capital (T1-T60), farthest |
|---|---|---|
| 8.9.2 Switzerland (8p, abandoned T220) | 1 (1 field) / 1 (1 field) / 1 (1 beside a city) / 1 (1 field) | 49 of 60, 9 hexes |
| 8.9.2 Switzerland (8p, won) | 1 (1 field) / 1 (1 field) / 2 (1 city, 1 field) / 2 (1 city, 1 field) | 57 of 60, 10 hexes |
| 8.9.2 Sparta (8p, abandoned T86) | 2 (2 field) / 4 (1 escort, 3 field) / 7 (1 beside a city, 6 field) / 8 (4 beside a city, 1 city, 3 field) | 60 of 60, 14 hexes |
| 8.9.2 Sparta (8p, abandoned T42) | 2 (2 field) / 3 (3 field) / 6 (1 capital, 5 field) / 8 (1 capital, 7 field) | 42 of 42, 16 hexes |
| 8.9.4 Sparta (4p, won) | 3 (3 field) / 5 (3 beside a city, 1 city, 1 field) / 8 (1 beside a city, 7 field) / 9 (4 beside a city, 1 city, 4 field) | 28 of 40, 15 hexes |
| 8.9.4 Switzerland (8p, abandoned T67) | 1 (1 field) / 1 (1 field) / 1 (1 city) / 2 (1 beside a city, 1 city) | 60 of 60, 8 hexes |
| 8.9.4 Switzerland (8p, won) | 1 (1 field) / 1 (1 field) / 1 (1 escort) / 1 (1 city) | 60 of 60, 9 hexes |
| 8.9.5 Athens (8p, abandoned T56) | 1 (1 field) / 1 (1 field) / 2 (2 field) / 3 (3 field) | 41 of 56, 7 hexes |
| 8.9.5 Athens (8p, lost) | 1 (1 field) / 1 (1 field) / 1 (1 field) / 1 (1 capital) | 57 of 60, 11 hexes |
| 8.9.6 Athens (8p, won) | 1 (1 field) / 1 (1 field) / 1 (1 field) / 2 (1 city, 1 escort) | 48 of 60, 9 hexes |
| 8.9.6 Mongols (8p, abandoned T92) | 1 (1 field) / 1 (1 field) / 2 (1 capital, 1 field) / 3 (1 capital, 1 city, 1 field) | 60 of 60, 11 hexes |
| 8.9.6 Switzerland (8p, abandoned T45) | 1 (1 field) / 1 (1 field) / 1 (1 city) / 1 (1 field) | 43 of 45, 8 hexes |
| 8.9.6 Switzerland (8p, won) | 1 (1 field) / 1 (1 field) / 2 (1 capital, 1 field) / 2 (1 city, 1 field) | 42 of 60, 5 hexes |
| 8.9.7 Celts (4p, won) | 1 (1 field) / 1 (1 field) / 1 (1 city) / 2 (1 capital, 1 city) | 60 of 60, 7 hexes |
| 8.9.10 Sparta (4p, won) | 2 (2 field) / 3 (3 field) / 5 (5 field) / 4 (4 field) | 38 of 60, 14 hexes |
| 8.9.10 Japan (4p, abandoned T60) | 1 (1 field) / 1 (1 escort) / 0 / 2 (1 capital, 1 field) | 28 of 60, 7 hexes |
| 8.9.10 Athens (8p, won) | 1 (1 field) / 1 (1 city) / 1 (1 escort) / 3 (1 capital, 1 city, 1 field) | 60 of 60, 7 hexes |
| 8.9.11 Japan (5p, won) | 1 (1 field) / 1 (1 escort) / 1 (1 field) / 0 | 29 of 60, 9 hexes |
| 8.9.13 Athens (5p, won) | 1 (1 field) / 1 (1 field) / 2 (1 capital, 1 field) / 2 (1 capital, 1 field) | 59 of 60, 8 hexes |
| 8.9.13 Sparta (5p, abandoned T125) | 2 (1 capital, 1 field) / 3 (3 field) / 5 (5 field) / 7 (7 field) | 52 of 60, 22 hexes |
| 8.9.15 Switzerland (4p, won) | 1 (1 field) / 1 (1 field) / 1 (1 field) / 2 (1 beside a city, 1 city) | 60 of 60, 6 hexes |
| 8.9.20 Aztecs (8p, abandoned T52) | 1 (1 field) / 3 (3 field) / 4 (4 field) / 4 (1 beside a city, 3 field) | 47 of 52, 10 hexes |
| 8.9.22/23 Aztecs (8p, won) | 2 (1 beside a city, 1 field) / 3 (3 field) / 5 (1 beside a city, 2 city, 2 field) / 6 (2 beside a city, 4 field) | 60 of 60, 9 hexes |
| 8.9.22/23 Aztecs (8p, won) | 2 (2 field) / 3 (1 capital, 2 field) / 5 (2 beside a city, 1 capital, 1 city, 1 field) / 3 (1 beside a city, 1 capital, 1 city) | 60 of 60, 12 hexes |
| 8.9.24 Switzerland (8p, abandoned T23) | 1 (1 field) / 1 (1 field) / - / - | 23 of 23, 11 hexes |
| 8.9.24 Switzerland (5p, won) | 1 (1 field) / 1 (1 city) / 1 (1 field) / 2 (1 beside a city, 1 capital) | 60 of 60, 9 hexes |
| 8.9.25 Celts (6p, won) | 1 (1 field) / 1 (1 city) / 1 (1 escort) / 2 (2 city) | 59 of 60, 7 hexes |
| 8.9.25 India (5p, won) | 1 (1 field) / 1 (1 field) / 2 (1 city, 1 field) / 2 (2 city) | 51 of 60, 8 hexes |
| 8.9.25 Germany (6p, abandoned T22) | 1 (1 field) / 1 (1 field) / - / - | 22 of 22, 7 hexes |
| 8.9.26 Switzerland (6p, abandoned T30) | 1 (1 field) / 1 (1 field) / 1 (1 field) / - | 29 of 30, 11 hexes |
| 8.9.26 Switzerland (6p, won) | 1 (1 field) / 2 (2 field) / 2 (2 field) / 2 (2 field) | 60 of 60, 13 hexes |
| 8.9.26 Sparta (6p, won) | 2 (2 field) / 4 (4 field) / 6 (6 field) / 6 (2 beside a city, 1 city, 3 field) | 60 of 60, 21 hexes |
| 8.9.26 Celts (6p, won) | 1 (1 field) / 3 (1 beside a city, 1 capital, 1 field) / 3 (1 beside a city, 1 capital, 1 field) / 3 (1 beside a city, 1 capital, 1 field) | 60 of 60, 9 hexes |
| 8.9.27 Germany (6p, won) | 1 (1 field) / 1 (1 field) / 1 (1 escort) / 1 (1 city) | 60 of 60, 10 hexes |
| 8.9.29 Aztecs (6p, won) | 2 (2 field) / 4 (1 beside a city, 1 capital, 2 field) / 6 (1 beside a city, 1 capital, 4 field) / 8 (1 beside a city, 7 field) | 60 of 60, 15 hexes |
| 8.9.30 Switzerland (6p, won) | 2 (2 field) / 2 (2 field) / 2 (1 escort, 1 field) / 2 (1 city, 1 field) | 60 of 60, 10 hexes |
| 8.9.31 Sparta (6p, lost) | 2 (2 field) / 4 (1 beside a city, 3 field) / 6 (3 beside a city, 1 city, 2 field) / 8 (1 city, 7 field) | 42 of 60, 13 hexes |
| 8.9.31 Sparta (6p, abandoned T27) | 2 (2 field) / 3 (3 field) / - / - | 23 of 27, 14 hexes |
| 8.9.31 Mongols (6p, abandoned T41) | 2 (1 capital, 1 field) / 2 (1 city, 1 field) / 2 (1 beside a city, 1 city) / 2 (2 field) | 39 of 41, 9 hexes |
| 8.9.31 Mongols (6p, won) | 2 (1 capital, 1 field) / 2 (2 field) / 2 (1 capital, 1 field) / 3 (1 beside a city, 2 field) | 60 of 60, 18 hexes |
| 8.9.32 Athens (6p, won) | 1 (1 field) / 1 (1 field) / 1 (1 escort) / 1 (1 escort) | 59 of 60, 5 hexes |
| 8.9.32.1 Aztecs (6p, won) | 2 (1 capital, 1 field) / 3 (3 field) / 3 (1 beside a city, 1 escort, 1 field) / 3 (2 city, 1 field) | 21 of 60, 6 hexes |
| 8.9.34 Athens (6p, won) | 1 (1 field) / 1 (1 field) / 2 (2 field) / 2 (1 city, 1 field) | 60 of 60, 9 hexes |
| 8.9.35.1 Byzantium (6p, abandoned T24) | 2 (2 field) / 3 (3 field) / - / - | 24 of 24, 16 hexes |
| 8.9.40 Byzantium (6p, won) | 2 (1 capital, 1 field) / 2 (1 escort, 1 field) / 2 (1 city, 1 field) / 2 (1 beside a city, 1 field) | 43 of 60, 8 hexes |

## 4. Under early attack

**There are such cases: 25 rival attacks on his own cities before T61 in 15 games** (plus 9 barbarian raids, none
of which took a city, and 7 counterattacks on cities he had just captured, left out below). The table lists every
case, including those three groups.

What decided them:

- **Walls.** No walled city fell (0 of 8 attacks). Unwalled cities fell 11 times; with three or more attackers, 10
  of 12 unwalled cities fell. Every city he lost before T61 was unwalled and most were still building their Walls.
- **Bought defenders.** He bought one or two units in the attacked city in 10 of the 25 attacks (and Walls once, under the 8.9.2 rules). They held only
  where the city was walled or faced two attackers (Aarau, Basel, Lugano 8.9.4, Winterthur, Piraeus at first), and
  once against three (Lugano, 8.9.24 capital, two Composite Archers on T58 and T59 out of a 1,036 treasury, plus a
  switch from Settler to Walls). Against three or four attackers at an unwalled city, buys failed 5 times in 6
  (Piraeus, Geneva, Lugano 8.9.6, Zürich, Hamburg with two buys). One bought defender does not end an early
  campaign in his tapes, before or after the 8.9.41 peace fix.
- **A buy before the declaration.** Once (8.9.15 Basel): three Spartan units stood 2 to 4 hexes out at Peace from
  T34; he bought a Ranger there on T37, Sparta declared on T40, the Walls finished on T44 and the city held at 49 HP.
- **Expansion did not pause.** His Settler queues kept running (a Settler finished during 8 of the 25 attacks);
  only once did he take a Settler out of a queue for Walls (Lugano, 8.9.24, T59).
- **Diplomacy.** He proposed Peace in 2 of 25 attacks (Texcoco T56 and Sakai T60, neither accepted in time) and
  refused most Peace offers from the attacker. He accepted Peace from other empires at war with him (Hamburg T60,
  Shangdu T37). He never proposed an Alliance before T61 in any game; the one he accepted under pressure (the
  Aztecs, 8.9.26, T24) is what brought Sparta onto Zürich.
- **Afterwards.** In 9 games he lost a city before T61. He abandoned 5 of them within 30 turns (8.9.5 Athens,
  8.9.6 Switzerland, 8.9.10 Japan, 8.9.26 Switzerland, 8.9.31 Mongols) and won the other 4 (8.9.6 Athens, 8.9.6
  Switzerland, 8.9.22 Aztecs, 8.9.27 Germany). In all 9 the capital held.

None of his capitals fell before T61 (6 attacks, 5 walled). The game the fable seat lost is the case his record
avoids: an unwalled capital and a second city in reach of three Hoplites before T40. In his tapes that position
either never arises (capital walled by T18 since 8.9.27) or he restarts.

Every early attack, T1 to T60. Kind: his capital, a city he founded, or a city he had just captured. "Held": still
his three turns after the attack; "game ends": the window (T62) or the game closed first (Lugano 8.9.24 held, per
its coach review). "Bought" covers 5 turns before to 3 after; "(other city)" is a purchase elsewhere.

| Game | City | Turns | Attackers (most at once) | His units in or beside | Walls | Lowest HP | Bought (T-5 to end+3) | City queue changes | Diplomacy | Settlers finished meanwhile | Outcome |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 8.9.2 Switzerland | Biel (his city) | T47-T58 | Barbarians 4 | 1 | no | 200 | T49 Ranger; T49 Ranger (other city) | none | T57 refused a proposal | 1 | held |
| 8.9.2 Sparta | Sellasia (his city) | T37-T38 | Mongols 2 | 0 | yes | 200 | T38 Walls | none | none | 0 | held |
| 8.9.2 Sparta | Kyoto (captured) | T39-T39 | Japan 2 | 5 | no | 0 | T38 Walls (other city) | T40 Market | none | 0 | held |
| 8.9.4 Sparta | Amyclae (capital) | T30-T37 | Barbarians 2 | 1 | no | 200 | none | T31 Soldier; T37 Phalangite | none | 0 | held |
| 8.9.4 Sparta | Karakorum (captured) | T35-T35 | Mongols 2 | 4 | no | 0 | none | T36 Market | none | 0 | held |
| 8.9.4 Switzerland | Aarau (capital) | T59-T59 | Sparta 2 | 1 | yes | 200 | T59 CompositeArcher | none | none | 1 | held |
| 8.9.4 Switzerland | Zürich (his city) | T61-T61 | Sparta 2 | 2 | no | 200 | T59 CompositeArcher (other city) | T58 Walls | none | 0 | game ends |
| 8.9.4 Switzerland | Lugano (his city) | T45-T45 | Mongols 2 | 1 | no | 200 | T46 CompositeArcher; T46 Horseman (other city) | none | none | 0 | held |
| 8.9.5 Athens | Athens (his city) | T27-T29 | Sparta 2 | 0 | no | 0 | none | none | none | 0 | lost T30; game abandoned T56 |
| 8.9.5 Athens | Phaleron (capital) | T56-T56 | Barbarians 2 | 0 | yes | 200 | none | none | none | 0 | game ends; game abandoned T56 |
| 8.9.5 Athens | Eleusis (capital) | T38-T41 | Barbarians 2 | 1 | no | 200 | none | none | none | 0 | held |
| 8.9.6 Athens | Piraeus (his city) | T43-T45 | Mongols 2 | 2 | no | 131 | T44 Ranger | T42 Walls | T44 refused a proposal | 0 | held |
| 8.9.6 Athens | Piraeus (his city) | T48-T50 | Mongols 3 | 2 | no | 40 | T44 Ranger | T53 Walls | T44 refused a proposal | 0 | lost T53 |
| 8.9.6 Mongols | Bukhara (his city) | T42-T46 | Sparta 3 | 1 | no | 179 | none | T38 Market; T49 Cataphract | none | 0 | held |
| 8.9.6 Switzerland | Geneva (his city) | T41-T44 | Mongols 4 | 2 | no | 0 | T44 CompositeArcher | T38 Walls | none | 0 | lost T45; game abandoned T45 |
| 8.9.6 Switzerland | Lugano (his city) | T41-T45 | Sparta 4, Byzantium 2 | 3 | no | 0 | T42 Phalangite | T37 Walls | T40 refused a proposal; T47 refused a proposal | 1 | lost T46 |
| 8.9.6 Switzerland | Bellinzona (capital) | T48-T49 | Sparta 2 | 2 | yes | 200 | none | T43 Phalangite; T48 Settler | T47 refused a proposal | 0 | held |
| 8.9.10 Sparta | Marathon (captured) | T42-T42 | Barbarians 2 | 4 | no | 40 | none | T42 Walls | T40 refused a proposal | 0 | held |
| 8.9.10 Sparta | Bibracte (captured) | T61-T61 | Japan 2 | 1 | yes | 200 | none | none | none | 1 | game ends |
| 8.9.10 Japan | Sendai (capital) | T27-T30 | Sparta 2, Athens 1 | 0 | yes | 186 | none | T24 Walls; T28 Ranger; T32 Ranger | none | 1 | held; game abandoned T60 |
| 8.9.10 Japan | Sakai (his city) | T28-T29 | Sparta 4 | 1 | no | 23 | none | T24 Walls | none | 1 | lost T30; game abandoned T60 |
| 8.9.10 Japan | Sendai (capital) | T44-T44 | Athens 1, Sparta 1 | 1 | yes | 96 | none | T41 Horseman | T41 refused a proposal | 0 | held; game abandoned T60 |
| 8.9.10 Japan | Sakai (his city) | T50-T55 | Sparta 3, Barbarians 2 | 2 | no | 0 | T57 Soldier (other city) | T51 Walls | T50 refused a proposal | 0 | lost T56; game abandoned T60 |
| 8.9.10 Japan | Sakai (his city) | T60-T60 | Sparta 3 | 1 | no | 0 | T57 Soldier (other city) | none | T59 refused a proposal; T60 proposed Peace | 1 | game ends; game abandoned T60 |
| 8.9.10 Athens | Phaleron (capital) | T27-T30 | Barbarians 2 | 0 | yes | 200 | none | T30 Settler | none | 1 | held |
| 8.9.10 Athens | Athens (his city) | T61-T61 | Barbarians 2 | 1 | yes | 200 | none | T57 Granary | none | 1 | game ends |
| 8.9.13 Athens | Athens (capital) | T16-T16 | Sparta 2 | 0 | yes | 200 | none | T14 Walls | none | 0 | held |
| 8.9.13 Sparta | Matsue (captured) | T44-T44 | Japan 2 | 5 | yes | 20 | none | T45 Market | T44 refused a proposal | 0 | held |
| 8.9.15 Switzerland | Basel (his city) | T40-T44 | Sparta 3 | 2 | yes | 49 | T37 Ranger | none | none | 0 | held |
| 8.9.22/23 Aztecs | Texcoco (his city) | T33-T35 | Sparta 3 | 0 | no | 83 | none | T30 Walls | T34 refused a proposal | 0 | lost T36 |
| 8.9.22/23 Aztecs | Texcoco (his city) | T56-T61 | Sparta 4 | 2 | no | 0 | none | T54 Walls | T51 refused a proposal; T56 proposed Peace | 2 | lost T62 |
| 8.9.22/23 Aztecs | Delhi (captured) | T57-T57 | Sparta 2 | 0 | yes | 200 | none | T57 Roadworks | none | 0 | held |
| 8.9.24 Switzerland | Winterthur (his city) | T57-T57 | Mongols 2 | 0 | yes | 200 | T58 CompositeArcher; T59 CompositeArcher (other city) | none | none | 1 | held |
| 8.9.24 Switzerland | Lugano (capital) | T58-T60 | Mongols 3 | 1 | no | 61 | T58 CompositeArcher (other city); T59 CompositeArcher | T59 Walls | none | 1 | game ends |
| 8.9.26 Switzerland | Zürich (his city) | T27-T29 | Sparta 3 | 1 | no | 40 | T27 Ranger | none | T24 accepted a proposal | 0 | lost T30; game abandoned T30 |
| 8.9.26 Celts | Emain Macha (his city) | T39-T40 | Barbarians 2 | 0 | no | 200 | none | T37 Walls | T35 accepted a proposal; T40 refused a proposal | 1 | held |
| 8.9.27 Germany | Hamburg (his city) | T57-T59 | Mongols 4 | 2 | no | 0 | T58 CompositeArcher; T60 Phalangite | none | T60 accepted a proposal | 0 | lost T60 |
| 8.9.30 Switzerland | Shangdu (captured) | T53-T54 | Mongols 2 | 4 | no | 0 | none | T54 Walls | T48 refused a proposal; T50 refused a proposal | 0 | held |
| 8.9.31 Sparta | Sparta (capital) | T17-T25 | Barbarians 3 | 1 | no | 200 | none | none | none | 0 | game ends; game abandoned T27 |
| 8.9.31 Mongols | Shangdu (his city) | T34-T39 | Sparta 3 | 1 | no | 0 | none | T36 Horseman | T37 accepted a proposal | 0 | lost T40; game abandoned T41 |
| 8.9.32.1 Aztecs | Texcoco (his city) | T41-T43 | Barbarians 2 | 1 | yes | 200 | none | T43 Quarry | T40 refused a proposal; T41 refused a proposal | 0 | held |

## 5. Where he settles, and unit or Walls first in a new city

**Finding.** He packs: 73 of 99 new cities (T2 to T60) stand 4 hexes or less from his capital (median 4), and
the nearest rival city is a median 5 hexes away when he founds. He does not steer away from the rush faction in
the opening: of 84 cities in the games where he was not Sparta or the Mongols, 41 were closer to the nearest Sparta or Mongol capital than his own capital is,
34 farther, 9 level; 16 stood within 6 hexes of it. "Never settle toward the aggressive neighbour" is what he says
(2026-11-07), not what his first 60 turns show.

New cities queue Walls first in 82 of 99; the other 17 are mostly Mongol towns (Market first) and a few Markets
or Mines. A military unit stands in the new city on its founding turn in 49 of 99 (the escort, which stays) and at
some point before T60 in 72. Walls finish a median 8 turns after founding (61 walled by T60). A unit was queued
before Walls in 2 of 99 new cities (a Ranger, a Cataphract); otherwise the garrison is the escort he already had,
never a build. The cities that fell early (Q4) were the ones still waiting for their Walls with three or more
attackers at the gate.

| Game | Cities founded T2-T60 | Hexes from own capital | Nearest rival city at founding (median, min) | First queue Walls | Unit inside on founding turn | Walls done (turns after founding, median) |
|---|---|---|---|---|---|---|
| 8.9.2 Switzerland (8p, abandoned T220) | 1 | 4 | 7, 7 | 0 of 1 | 1 of 1 | none by T60 |
| 8.9.2 Switzerland (8p, won) | 4 | 4 (4-6) | 5, 4 | 4 of 4 | 3 of 4 | 12 (3 walled) |
| 8.9.2 Sparta (8p, abandoned T86) | 1 | 4 (4-4) | 4, 4 | 0 of 1 | 0 of 1 | 14 (1 walled) |
| 8.9.2 Sparta (8p, abandoned T42) | 0 | | | | | |
| 8.9.4 Sparta (4p, won) | 0 | | | | | |
| 8.9.4 Switzerland (8p, abandoned T67) | 2 | 4 (4-4) | 5.5, 4 | 1 of 2 | 2 of 2 | 15 (1 walled) |
| 8.9.4 Switzerland (8p, won) | 2 | 4 (4-4) | 4.5, 4 | 2 of 2 | 2 of 2 | 23 (1 walled) |
| 8.9.5 Athens (8p, abandoned T56) | 1 | 5 | 4, 4 | 1 of 1 | 0 of 1 | none by T60 |
| 8.9.5 Athens (8p, lost) | 2 | 4.5 (4-5) | 6.5, 5 | 2 of 2 | 0 of 2 | 6 (1 walled) |
| 8.9.6 Athens (8p, won) | 3 | 5 (4-5) | 5, 4 | 3 of 3 | 2 of 3 | 7.5 (2 walled) |
| 8.9.6 Mongols (8p, abandoned T92) | 2 | 7 | 7.5, 6 | 0 of 2 | 0 of 2 | none by T60 |
| 8.9.6 Switzerland (8p, abandoned T45) | 2 | 4 (4-4) | 4, 4 | 2 of 2 | 1 of 2 | 8 (1 walled) |
| 8.9.6 Switzerland (8p, won) | 2 | 4 | 7, 5 | 2 of 2 | 0 of 2 | none by T60 |
| 8.9.7 Celts (4p, won) | 3 | 4 (4-5) | 8, 6 | 3 of 3 | 1 of 3 | 10 (2 walled) |
| 8.9.10 Sparta (4p, won) | 1 | 4 (4-4) | 8, 8 | 1 of 1 | 0 of 1 | 5 (1 walled) |
| 8.9.10 Japan (4p, abandoned T60) | 2 | 3 (3-3) | 6, 6 | 1 of 2 | 1 of 2 | 5 (1 walled) |
| 8.9.10 Athens (8p, won) | 4 | 4 (4-4) | 5.5, 4 | 4 of 4 | 4 of 4 | 6 (4 walled) |
| 8.9.11 Japan (5p, won) | 3 | 3 (3-3) | 5, 5 | 3 of 3 | 2 of 3 | 16 (2 walled) |
| 8.9.13 Athens (5p, won) | 3 | 4 (4-4) | 8, 4 | 3 of 3 | 3 of 3 | 8.5 (2 walled) |
| 8.9.13 Sparta (5p, abandoned T125) | 0 | | | | | |
| 8.9.15 Switzerland (4p, won) | 2 | 4.5 (4-5) | 6, 5 | 2 of 2 | 1 of 2 | 12 (1 walled) |
| 8.9.20 Aztecs (8p, abandoned T52) | 1 | 5 (5-5) | 4, 4 | 1 of 1 | 1 of 1 | 8 (1 walled) |
| 8.9.22/23 Aztecs (8p, won) | 3 | 4 (4-4) | 6, 5 | 3 of 3 | 2 of 3 | 5 (1 walled) |
| 8.9.22/23 Aztecs (8p, won) | 0 | | | | | |
| 8.9.24 Switzerland (8p, abandoned T23) | 0 | | | | | |
| 8.9.24 Switzerland (5p, won) | 4 | 4 (4-8) | 5, 5 | 4 of 4 | 4 of 4 | 6 (3 walled) |
| 8.9.25 Celts (6p, won) | 3 | 4 (4-5) | 8, 4 | 3 of 3 | 2 of 3 | 8 (3 walled) |
| 8.9.25 India (5p, won) | 4 | 3 (3-3) | 4.5, 4 | 4 of 4 | 2 of 4 | 4 (4 walled) |
| 8.9.25 Germany (6p, abandoned T22) | 0 | | | | | |
| 8.9.26 Switzerland (6p, abandoned T30) | 1 | 4 | 4, 4 | 1 of 1 | 0 of 1 | none by T60 |
| 8.9.26 Switzerland (6p, won) | 3 | 6 (4-7) | 7, 5 | 2 of 3 | 0 of 3 | 7 (2 walled) |
| 8.9.26 Sparta (6p, won) | 2 | 6 | 15.5, 15 | 0 of 2 | 0 of 2 | none by T60 |
| 8.9.26 Celts (6p, won) | 6 | 4 (4-8) | 5, 4 | 6 of 6 | 0 of 6 | 7 (5 walled) |
| 8.9.27 Germany (6p, won) | 5 | 6 (4-8) | 5, 4 | 4 of 5 | 1 of 5 | 14 (3 walled) |
| 8.9.29 Aztecs (6p, won) | 1 | 4 | 5, 5 | 1 of 1 | 0 of 1 | none by T60 |
| 8.9.30 Switzerland (6p, won) | 3 | 4 (4-4) | 5, 4 | 3 of 3 | 2 of 3 | 9 (3 walled) |
| 8.9.31 Sparta (6p, lost) | 2 | 4 (4-4) | 6.5, 6 | 2 of 2 | 0 of 2 | 12 (1 walled) |
| 8.9.31 Sparta (6p, abandoned T27) | 0 | | | | | |
| 8.9.31 Mongols (6p, abandoned T41) | 3 | 4 | 5, 4 | 0 of 3 | 2 of 3 | none by T60 |
| 8.9.31 Mongols (6p, won) | 4 | 4.5 | 8, 4 | 0 of 4 | 2 of 4 | none by T60 |
| 8.9.32 Athens (6p, won) | 4 | 4 (4-4) | 4.5, 4 | 4 of 4 | 2 of 4 | 6 (3 walled) |
| 8.9.32.1 Aztecs (6p, won) | 3 | 4 (4-4) | 6, 4 | 3 of 3 | 3 of 3 | 6 (3 walled) |
| 8.9.34 Athens (6p, won) | 3 | 4 (4-5) | 4, 4 | 3 of 3 | 2 of 3 | 6 (2 walled) |
| 8.9.35.1 Byzantium (6p, abandoned T24) | 0 | | | | | |
| 8.9.40 Byzantium (6p, won) | 4 | 6.5 (4-8) | 16.5, 13 | 4 of 4 | 1 of 4 | 16 (4 walled) |

## 6. Site races

**Finding: none on tape.** In 45 games no rival founded a city within 2 hexes of the tile his Settler was first
ordered to, between that order and his founding. Of his Settlers that ended more than 2 hexes from their first
destination, none was beaten to the site; the one traced case turned away from a barbarian fight on the way
(8.9.30, T25). He is the one who races: he took 21 AI Settlers in 17 games before T61 (usually declaring war for
it, often making Peace soon after), and lost 3 of his own (8.9.22 Aztecs T29, 8.9.31 Mongols T30, and an 8.9.24
Swiss game he abandoned on T23, the turn its Settler was taken).

| Game | AI Settlers he took (turn) | His Settlers lost (turn) |
|---|---|---|
| 8.9.2 Switzerland | T24 | - |
| 8.9.2 Sparta | T20 | - |
| 8.9.5 Athens | T17 | - |
| 8.9.6 Athens | T17 | - |
| 8.9.6 Mongols | T35 | - |
| 8.9.10 Athens | T21 | - |
| 8.9.20 Aztecs | T26 | - |
| 8.9.22/23 Aztecs | T21, T23 | - |
| 8.9.22/23 Aztecs | T19 | T29 |
| 8.9.24 Switzerland | - | T23 |
| 8.9.24 Switzerland | T41 | - |
| 8.9.26 Celts | T17, T36 | - |
| 8.9.27 Germany | T35, T47 | - |
| 8.9.30 Switzerland | T24 | - |
| 8.9.31 Mongols | T14, T25 | T30 |
| 8.9.31 Mongols | T44 | - |
| 8.9.34 Athens | T46 | - |
| 8.9.40 Byzantium | T17 | - |

## The chapter: his early game, T1 to T60

Counts are games of his (45 read) that show the rule. Rules marked "since 8.9.27" describe how he plays now; the
older games show what he did before he lost Zürich to Sparta on 2026-11-07.

1. **When he plays a builder and Sparta's or the Mongols' capital is within about 20 hexes, the capital builds
   Walls first or second** (at most a Granary or a Soldier before them), finished by T9 to T18, then Settlers.
   Since 8.9.27: 5 of 5. Before: 7 of 23, mostly Granary then Settlers.
2. **When the rush faction is far (27 hexes or more), he skips early Walls**: one unit, then three Settlers back to
   back, capital Walls around T40; population, Science and Gold first. 1 game (8.9.40) plus his own words.
3. **When he plays Sparta or the Aztecs, the capital queues three to seven Soldiers on T2 to T4** and expands by capture
   or from T26 to T51. 13 of 13. The Mongols: a Horseman or a Quarry, then Settlers, no Walls (3 of 3).
4. **He spends nothing before T30.** 43 of 45. His treasury is about 220 at T30 (median), half of it ruins and
   natural wonders; 500 by T30 happened twice. No Settler is bought (44 of 45 to T60).
5. **His first unit leaves the capital on T1 and works outside**: scouting, camps, escorts, taking AI Settlers. 42
   of 45. The capital stands empty behind its Walls (garrisoned at T30 in 8 of 41). Builders own one or two
   military units to T40.
6. **He settles 4 hexes out in every direction, Walls first in every new city**, the escort staying as the
   garrison. Walls first in 82 of 99 cities, a unit first in 2. He does not hold back from the rush faction's
   side in the opening (41 cities toward it, 34 away).
7. **When an AI army stands next to a city at Peace, he buys a shooter there before it declares.** 1 game
   (Basel, 8.9.15, three turns before Sparta declared).
8. **When an army attacks a city, he buys one or two shooters in it and keeps expanding.** Bought in 10 of 25
   attacks; only once did a Settler leave a queue for Walls. It holds only behind Walls or against two attackers: walled
   cities 8 of 8 held, unwalled cities with three or more attackers 2 of 12 held.
9. **He does not ask for peace or allies when attacked early.** Peace proposed in 2 of 25 attacks, an Alliance
   proposed in 0 of 45 games; he takes Peace offered by other empires at war with him (2).
10. **When he loses a city before T61, he often restarts.** 5 of 9 such games abandoned within 30 turns; the 4 he
    kept, he won, and in all 9 the capital held.
11. **Nobody raced him to a site**: 0 cases in 45 games. He races others instead: 21 AI Settlers taken in 17 games.

**For a model seat next to Sparta (the fable-replicate map):** Walls in the capital before the first Settler
(rule 1) and Walls first in every new city (rule 6). His tapes have no answer for an unwalled city with three Hoplites at it before T40, other
than losing it.

## The fable-replicate gaps, answered

| Gap (README section 2) | What his tapes say |
| --- | --- |
| "Capital: Walls first" against "Granary, Archer, three Settlers" | Walls first since 8.9.27 with a rush capital within about 20 hexes (5 of 5); the Settler line is his older and his far-rush opening (Q1). |
| About 500 Gold by T30 | Not his norm: median 219, 500 in 2 of 41 games; half is windfalls, and he buys nothing before T30 (Q2). |
| One bought defender ends AI campaigns | False before T61: unwalled cities with three or more attackers fell 10 of 12 times, with a purchase 5 of 6 (Q4). The evidence for the line is from T58 (8.9.24, two purchases), T137 to T160 (8.9.15) and T142 (8.9.4), and since 8.9.41 Hard no longer makes Peace while its offensive pays. |
| A three-unit attack on a core city before T30 | Three cases. Sendai (8.9.10 Japan capital, walled) held; Sakai (8.9.10) and Zürich (8.9.26), both unwalled, fell, and he abandoned those games 30 turns and 0 turns later (Q4). |
| Does expansion pause under attack | No, apart from one Settler swapped for Walls in the capital (Q4). |
| Seek an Alliance when the capital is about to fall | He never proposed one before T61 in 45 games; the one he accepted (8.9.26) caused the loss (Q4). Intent is asked below. |
| Redoubt first in a 4-Production frontier city | Walls first in 82 of 99 new cities, a unit first in 2; his early losses are cities still building them (Q5). |
| Rival Settler racing for the same site | Never happened in 45 games; he takes AI Settlers instead (21) (Q6). |
| Eight seats, land closing | 8-seat games show the same opening and no Settler purchases (Q1, Q2). |

## Tapes are silent (questions for him, about intent)

1. **Rush distance.** With Sparta 20 hexes away you still walled the capital first (8.9.34 Athens); at 27 to 34
   you did not (8.9.40 Byzantium). Is your line set by distance (and is it nearer 20 or 30 hexes), or by whether
   you have seen their army?
2. **Expansion under attack.** When three units reach an unwalled city before T40 and your Gold buys at most one
   unit: do you keep the capital on Settlers, or switch it to units and Walls until the attack ends?
3. **Peace.** When a rival is winning an early siege of one of your cities: do you propose peace yourself, or only
   accept theirs?
4. **Alliance.** When your capital is about to fall: would you ask a strong neighbour for an Alliance, or never?
5. **Restarts.** You abandoned 5 of the 9 games in which you lost a city before T61. Was that because the game was
   lost, or for another reason (a test, a new build, a better start)?
6. **Frontier city.** For a new city at about 4 Production beside an aggressive neighbour: Walls first anyway, or
   a unit first?
7. **Site race.** If an AI Settler is walking to the site your Settler is heading for: race it, settle short, or
   take it?
