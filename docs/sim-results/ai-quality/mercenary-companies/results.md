# Mercenary Companies replace bought peace (2026-10-22/23)

Branch `claude/mercenary-companies`. Rule: `crates/core/src/company.rs` and game-rules §5. Raw runs: `~/LegatumRuns/cloud/<batch>/merged.json.gz` (not committed; each regenerates in about 3 minutes for cents). All runs: 2,048 balanced 8-empire Hard games, `legatum-sim --players 8 --difficulty hard --games 2048 --seed 9210001`, Standard speed, shuffled factions. Fair share is 12.5%.

## Step 1: how much of Byzantium's share did bought peace carry? (before building)

Same binary on the 8.9.7-era engine, sim-only environment override (never landed) switching bought peace off, against the control:

| | Control (bought peace) | Bought peace off |
|---|---|---|
| Byzantium | 30.1% (616) | 27.0% (553) |
| Athens | 17.2% | 17.9% |
| Switzerland | 14.8% | 16.1% |
| Mongols | 11.3% | 11.2% |
| Germany | 9.8% | 9.5% |
| Celts | 8.3% | 8.7% |
| Sparta | 7.1% | 7.6% |
| Japan | 1.5% | 1.9% |

Bought peace was used in 1,239 of 2,048 games (2,240 payments, median turn 195, median price 130 Gold). Removing it moves Byzantium about 3 pp (within roughly 2 sigma of 2,048 games): **it was not the gimmick.**

What was: the **Heir goal**. Byzantium completes it in 86% of games (median turn 138, tied with Switzerland as the earliest goal) and wins 34.8% of those against 0.4% of the rest. A sim-only batch with the Heir unavailable (bought peace on, `noheir8`): **Byzantium 13.0%** (fair share), Athens 20.9%, Switzerland 20.1%, Germany 13.4%, Mongols 13.3%, Celts 8.7%, Sparta 7.9%, Japan 2.5%. The user's decision (2026-10-22): the Heir stays; birth and position become public so rivals can snatch him.

## Step 3: Mercenary Companies and the public Heir, together

Control is master (`f078b1937`, bought peace on, Heir private); candidate is the branch with the Heir's birth announced and his position in every empire's view (`merc8b`).

| | Master control | Companies + public Heir |
|---|---|---|
| Byzantium | 31.0% (635) | **22.9%** (469) |
| Athens | 18.0% | 19.3% |
| Mongols | 12.0% | 11.4% |
| Switzerland | 10.5% | 12.7% |
| Germany | 10.1% | 12.2% |
| Celts | 7.8% | 8.7% |
| Sparta | 6.9% | 7.8% |
| Japan | 3.7% | 5.0% |
| Best over worst | 8.4x | 4.6x |

Companies alone (before the public Heir, `merc8`, same engine as the 8.9.7 control): 22.7%. So the public Heir moved Byzantium by 0.2 pp: the rival AIs already priced capturing a city that shelters an Heir (`heir_in_city_value`, `held_goal_denial`) and rarely meet him in the field; the Companies carry the drop.

Byzantium under Companies: Heir goal completed 80.0% (86.7% control), median turn 142, VP 7.0 (7.7), eliminated 9.2% of games (4.5%), cities lost 2.4 per game (1.9), captured 2.3 (1.9).

Hiring (`merc8b`): 1,968 of 2,048 games hire; 46,489 hires (22.7 a game), median turn 245, median price 80 Gold; 14,942 served out their contract, 33 turned barbarian unpaid. Kinds: Soldier 43,167, Horseman 1,594, Phalangite 1,157, then Trooper, Assault Trooper, Field Cannon, Artillery, Rifleman. Gold paid to sellers is spread evenly (Japan 632k, Sparta 602k, Mongols 573k, Celts 570k, Switzerland 535k, Athens 522k, Germany 485k, summed over 2,048 games): no seller is favoured.

## AI-impact gate

Human-seat benchmark (8 maps, 256 salts each, Hard in every seat, reseeded dice, `hard_vs_human --brain hard --games 256 --seed 1`; this is a rule change, so seat-split arms cannot isolate it: both arms are all-seats): master `f078b1937` 39.2%, candidate 40.1%, **+0.9 pp, paired salt-bootstrap 95% [-0.9, +2.6]**. Per map (master / candidate): 8.3 Switzerland 4.7 / 4.7, 8.4 Sparta 36.3 / 43.8, 8.4 Japan 65.6 / 64.5, 8.4 Celts 25.8 / 25.8, 8.5 Mongols 12.5 / 12.5, 8.5 Switzerland 62.5 / 62.5, 8.6 Germany 46.1 / 46.1, 8.6 Sparta 59.8 / 60.9. Only the Sparta maps (which have a Byzantine rival) move, upward. No drop.

War suite `--check` (attack regression, `war-golden-horde-thin`, 64 Hard runs, floor 30%): on a base from before the Alliance pricing fix (`98e01c48c`) it read 0% on master and branch alike (free Alliances had broken it); on the branch merged with master `b687e6d20` it reads **68%** (median 10 turns, human 12), above the floor.
