# The Celtic sack: gates deleted, captures priced as sacks (2026-10-23)

Branch `claude/celts-sack`, from the coach review of the user's 8.9.7 Celts win (`docs/playtests/reviews/2026-10-22-8.9.7-celts.md`, recommendation 1; `ideas.md` #25). Rules text: `game-rules.md` section 10, "The Celtic sack, priced".

## What changed

1. Every early return for `Faction::Celts` in the city-attack paths of `crates/ai` is deleted (`city_assault.rs`; `military.rs` `campaign`, `campaign_target_for`, `seize_open_city`, `rush_siege`, `occupy_rush_city`, `open_front`, the target list, `scout_front`; `plan.rs` `campaign_candidates`, `contain_war`, the Contain start, `peace_would_be_broken`; `campaign.rs`; `victory.rs`; `threat.rs`; `diplomacy.rs`). `war_siege` still skips Celts (not in the brief; `siege.rs`).
2. A Celtic capture is priced as a sack (gene `sack_price`, default 1): Gold taken to army, no kept city, population, land, output, milestone or title, and the VP the victim loses with the city (crowns flipped, held goal).
3. `sack_commit` (default 1): a new Celtic Contain war needs a funded siege route, like the Sparta.
4. `sack_loss_percent` (60): in the opportunistic strike value a sacked city is worth that share of a kept city's value.
5. New diagnostic `crates/sim/examples/celts_wars.rs`: replays `--tape-out` tapes and prints, per Celtic seat, wars declared, sacks (victim, VP rank, army ratio), Raise the Warbands lost, camps cleared, and win rate by sacks and wars.

## Control moved under us

The coach measured 2.4% for the Celts on the 8.9.7 tag. Master had since fixed the Celtic camp count (`e840acb4b`) and other things: the control on master `de34c553a` is **8.3%** (7.3% on a fresh seed). Fair share is 12.5%.

## Balanced 8-empire Hard league (2,048 games each, 12 threads on the box)

| Arm | Seed 9210001 | Seed 9310001 |
|---|---|---|
| Control (master) | 8.3% (169) | 7.3% (150) |
| Gates lifted, kept-city pricing (`sack_price` 0) | 6.5% (134) | |
| Sack priced (`sack_price` 1) | 7.7% (157) | 8.1% (165, with `sack_commit`) |
| Sack priced plus `sack_commit` | 7.6% (156) | |

Paired against the control: -0.6 pp ±1.5 (priced, seed 1), -1.7 ±1.4 (kept pricing), +0.7 ±1.5 (seed 2). No change in win share is detectable; the Celts stay about 8%. Other factions move a little (Byzantium 30.1 -> 28.0, Mongols 11.3 -> 12.2 on seed 1; Japan 1.5 -> 1.7).

What the Celts do now (celts_wars, 1,024 seats on seed 9210001 unless noted):

| | Control | Sack priced | Kept pricing |
|---|---|---|---|
| Sacks per seat | 0.01 | 1.7 to 2.0 | 1.95 |
| Wars declared per seat | 1.4 | 1.1 to 1.9 | 2.75 |
| Gold taken per seat | 1 | 270 to 304 | 302 |
| Era-objective VP | 2.42 | 2.78 | 2.71 |
| Raise the Warbands held at the end (goal VP / 2) | 78% | 67% | 70% |
| Own camps cleared by the Celts' own units per seat | 15.6 | 11.8 | |
| Celtic cities lost per seat | 0.74 | 0.87 to 0.99 | 1.04 |

Sacks by the number a seat made (priced): 0 sacks 2.1% won (33% of seats), 1: 7.8%, 2: 8.3%, 3: 17.9%, 6+: 16.7%. Sacks of the strict VP leader are 15% of all sacks; the victim's army at the sack is a median 61% of the Celts' (quartiles 33 to 99), so the Celts hit rivals whose army is spent. A seat that never sacked wins 2.1% against the control's 6.2% in the same games: the Celts that start wars they cannot pay for in sacks lose more than the ones that stay home.

## Human-seat benchmark (8 maps x 128 salts, dice reseeded, two spot servers)

Control binary and candidate binary on the same maps and salts (`celts-ctrl-hvh-1022-220531`, `celts-cand-hvh-1022-220532` under `~/LegatumRuns/cloud/`):

| Map | Control | Sack priced | Kept pricing |
|---|---|---|---|
| 8.4 Celts (the only Celtic seat) | 10.9% | 20.3% | 25.0% |
| the seven other maps | identical | identical | identical |

Seat-only on the Celtic-seat map +9.4 pp ±8.5 (priced), +14.1 ±8.9 (kept); mean over the eight maps +1.17 pp [+0.10, +2.25]. Seat-only on the seven maps without a Celtic seat and opponents-only are exactly zero: the changes only run when a Celts AI is in the game, and no benchmark map has a Celtic rival. The `sack_commit` and pricing both leave the seat map's gain inside noise of each other.

## War suite

64 runs each, master control against the branch: 42% against 42% (6 of 14 at human parity both). Only `war-german-stand` moved, 100% -> 96% (2 of 64). The attack check `war-golden-horde-thin` reads 0% on both (the control master too, so not this change); it read 85 to 89% on 2026-10-21.

## Not measured

The coach's T50/T100/T150 seat branches need the 8.9.7 tag's engine and the throwaway `coach897_whatif` runner; master changed 169 commits since, so they were not rerun.
