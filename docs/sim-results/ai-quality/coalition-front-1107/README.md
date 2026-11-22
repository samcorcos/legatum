# Runaway coalition and second front, 2026-11-07

**Status:** gene `runaway_coalition` (Hard, bit flags) is **on by default (3, both bits)** since 2026-11-07, by the lead's judgment rather than the gate. It acts on the designer's own scenario: Byzantium keeps 3.1 of its starting cities instead of 0.5, Peace with the Mongols falls from 1.25 to 0.59 a run, and allied Sparta opens a second front. In full games the rivals don't get harder against a fixed seat (+0.54 pp for the seat, CI −0.88 to +1.90): the gate's Hard seat is rarely the runaway, while the designer usually is. Code: `crates/ai/src/runaway.rs`, built on the committed operation executor (`crates/ai/src/operation.rs`). To re-measure with a human-runaway scenario and once battle tactics improve (`claude/training-round2`).

## Why, and what is different from the coalition genes that failed

The designer, after "The Horde Looks East" (Byzantium vs the Mongols, 2026-11-07): the Mongols were running away, nobody would ally with him, and Sparta, his ally, "has a big army just sitting off to the side when the mongols leave to attack me and they don't counter attack". The 8.9.26 reviews found the same: the Celts' 78-turn Contain with no attack (Sparta review), and three AIs offering Peace during his hold (Swiss win review).

`coalition_push`, `leader_commit`, `conquest_alarm` and `see-the-leader` priced the war on the leader, then handed it to the plan layer and the campaign executors, which re-judged it every turn and mostly never marched (`docs/design/ai-strategy/fable-2026-11-07.md`, root cause 3). Here the second front is an **operation**: once committed, the executor marches its roster, declares when assembled and refuses Peace until its own abort conditions end it; the plan layer and the campaign reconciler can't cancel it.

## Mechanism (`runaway.rs`)

- **The read** (public only): the strict VP leader, or a team counting down, scoring 60 or more: 15 per VP of lead (up to 4), 15 for the Strongest Army crown, a quarter of its army's excess over the next team (up to 25), 10 per recent capture (up to 4), 20 per elimination (up to 2), 60 for a hold.
- **Bit 1, diplomacy:** an Alliance with a team at war with the runaway gains 15 plus up to 50 for the share of the runaway's public army the pair can bring (each army weighted by its reach to the runaway's cities); a call to arms against the runaway gains 40; no Peace with the runaway unless one of our cities is about to fall.
- **Bit 2, second front:** at War with the runaway (or allied with a team that is, with a legal declaration), when at least half its public army is visibly 8+ tiles from one of its cities, none of its units is within 6 of ours, and our land units within 12 outweigh the city (1.5x, 2.5x walled) and its visible defenders by threat power: commit a 15-turn strike on that city. The executor is honoured for it under bit 2 even with `operations` 0.

## Scenario Lab card: the designer's own scenario

`byzantium-empty-horde` ("The Horde Looks East", T74, Byzantium seat, Sparta its ally, the Mongols the runaway), every seat Hard on the arm's params, the 20-turn window plus 10 turns, 64 reseeded runs per arm (`crates/sim/examples/coalition_front_probe.rs`, [horde-probe-64.jsonl](horde-probe-64.jsonl)). Means per run:

| Arm | Byzantine starting cities held | Peace signed with the Mongols | Third-empire battles on Mongol cities (runs with any) | Byzantine battles on Mongol cities | Mongol cities at the end | Objective (retake Dyrrachium) |
|---|---:|---:|---:|---:|---:|---:|
| control | 0.50 | 1.25 | 3.81 (41/64) | 0.10 | 15.73 | 0/64 |
| bit 1 | 2.60 | 0.29 | 2.42 (25/64) | 4.31 | 14.25 | 0/64 |
| bit 2 | 2.40 | 1.98 | 5.90 (51/64) | 0.17 | 13.46 | 0/64 |
| bits 1+2 | **3.12** | 0.59 | 2.56 (31/64) | 4.35 | 13.71 | 0/64 |

Bit 2 committed 41 second-front strikes in 64 runs, all by Sparta: 13 struck and ended in the window (4 captured, 2 roster lost, 2 expired, 5 target gone), 28 still running at the end ([horde-probe-ops-64.jsonl](horde-probe-ops-64.jsonl)). Before the gate change (below) it committed none: measured from the Mongols' own cities, their army was never "away", because the cities they had captured from Byzantium stood on the front. The test is now per target city.

What the card shows: the designer's complaint is answered in the scenario. Sparta strikes Mongol cities when the horde goes west, nobody signs a separate Peace that frees the horde, and Byzantium keeps its cities. What it doesn't show: the objective. Nobody retakes Dyrrachium, and Mongol cities that fall are mostly razed (1.06 a run against 0.37 in control) rather than kept (held by the seat at the end 0.04 a run, by third empires 0).

## AI-only census

4,096 eight-empire Hard games per arm, local (`legatum-sim --players 8 --difficulty hard --games 4096`), control (empty params) against bits 1+2, paired by seed; 0 panics in all four. [census-9311003.json](census-9311003.json), [census-8241001.json](census-8241001.json), [census.py](census.py). "Runaway" = a team with a VP lead of 4 or more at T100 (`team_vp_turn_100`, added to the sim output by this branch).

| Measure | Seed 9311003 | Held-out 8241001 |
|---|---:|---:|
| Best / worst | Mongols 17.76% / Swiss 7.26% = **2.45** → Mongols 16.96% / Swiss 7.15% = **2.37** | Mongols 19.31% / Swiss 7.69% = **2.51** → Mongols 18.75% / Swiss 7.61% = **2.46** |
| Games with a T100 runaway | 8.18% → 8.28% | 9.79% → 9.30% |
| The runaway goes on to win | 59.7% → 59.9% | 56.4% → 55.6% |
| Same-seed games where the control's runaway won | 200 of 335 → 200 | 226 of 401 → 210 |
| City captures per game | 13.98 → 14.11 | 13.89 → 13.99 |
| Eliminations per game | 0.824 → 0.854 | 0.806 → 0.859 |

The ratio stays under 2.7 and improves slightly on both seeds; the strongest faction (the Mongols) loses 0.6 to 0.8 pp. A runaway at T100 still wins as often as before: the coalition doesn't stop an AI leader in AI-only games.

## Human-seat gate

Eight benchmark maps, 256 reseeded salts each (`LEGATUM_HVH_RESEED_DICE=1`), seed 310001, one spot server ($0.09 and $0.10): [gate-arms.json](gate-arms.json), [gate-comparison.json](gate-comparison.json), [gate-bits-comparison.json](gate-bits-comparison.json). Control: the Hard seat won 460 of 2,048.

| Arm | Seat wins | Seat change, pp [95%] |
|---|---:|---:|
| rivals, bits 1+2 | 471 | +0.54 [−0.88, +1.90] |
| rivals, bit 1 | 471 | +0.54 [−0.83, +1.86] |
| rivals, bit 2 | 464 | +0.20 [−0.20, +0.59] |
| seat, bits 1+2 | 461 | +0.05 [−1.03, +1.07] |

The rivals arm is the key one and should be negative (harder rivals). It isn't: flat, trending softer. Only 154 of 2,048 games differ by T100; the gene acts late, and on these maps the seat is rarely the runaway it reads.

## Designer-seat score

Hard in his seat on his tapes, every seat on the measured commit, on the box: control (this branch, gene 0) against a scratch commit with the default at 3 ([designer-seat-ctl.json](designer-seat-ctl.json), [designer-seat-on.json](designer-seat-on.json); not added to the history chart).

| | Control | Gene 3 |
|---|---:|---:|
| Held out | 57.2% [55.0, 59.4] | 57.4% [55.0, 59.7] |
| Training | 37.6% [35.5, 39.7] | 35.3% [33.3, 37.4] |

Inside the 3 pp noise band on both sets.

## A human-runaway position: the Swiss hold (8.9.26), first read

`celts-break-the-reserve` (T134 of his Swiss win: Switzerland on 9 VP with 6,485 Gold and 4 military units for 11 cities, the Celts the seat, every seat Hard on the arm's params), 64 runs, window plus 10 turns ([swiss-reserve-probe-64.jsonl](swiss-reserve-probe-64.jsonl)):

| Arm | Alliances signed with the Celts | Peace signed with the Swiss | Wars declared on the Swiss | Second fronts | Swiss VP at the end | Geneva taken |
|---|---:|---:|---:|---:|---:|---:|
| gene 0 | 0 | 1.62 | 0.06 | 0 | 10.76 | 0/64 |
| gene 3 | 0.56 | 1.29 | 0.07 | 0 | 10.82 | 0/64 |

Bit 1 shows (the Celts now sign Alliances, a little less Peace with the leader); bit 2 never fires. This runaway has almost no army, so it is never "away", and nobody near it is at War with it. A rich, empty leader needs its own path: a declaration and a strike on an empty city without the army-away test (`hollow_read` already reads such a leader). That is the next increment for this gene, ahead of the battle-tactics re-measure.

## Decision

**On by default (lead, 2026-11-07), against the written landing rule** ("land a default only if rivals get harder"). The reasoning: the full-game gate can't see this gene. Its Hard seat is rarely the runaway, so only 154 of 2,048 games differ by T100. In the designer's own situation, where the human is the runaway every game, the scenario shows what he asked for: allies hold, Sparta strikes, and nobody signs the Peace that frees the horde. The census ratio improves slightly on both seeds, and nothing measured gets worse.

**Why the rivals arm didn't get harder: mostly the strikes don't fire, and when they fire they seldom take the city.** In full games the gene changes only 7.5% of games before T100. In the scenario, over 64 runs, Sparta started 41 second-front strikes. Of the 13 that ended inside the window having struck, 4 captured their city, 2 lost their roster, 2 expired and 5 found the target gone (razed or taken back). The other 28 were still assembling or running at the window's end. Ordered attacks on the target: 11 in all. So: few fronts in AI-only games, and battle results the executor doesn't yet win (the operation search found the same: executed strikes lose).

**The Peace gate's escape is deliberately narrow.** The ordinary Peace gates let any endangered home sue for Peace. Giving the runaway gate the same escape let India and the Celts sign Peace with the Mongols and freed the horde: Byzantine cities held fell from 3.12 to 2.39 and Peace with the runaway rose to 1.96 a run, above control ([horde-probe-final-64.jsonl](horde-probe-final-64.jsonl) is the shipped gate). Only a city about to fall (`collapsing`) allows it.

**Next measurements:** a human-runaway scenario (the Celts 8.9.26 hold, T120–T134, rivals on the gene), and a re-run of this card and the gate once battle tactics improve (`claude/training-round2`).

## Checks

- `cargo test -p legatum-ai --release`: all passed; 10 new tests in `runaway::tests`.
- `tools/cloud/verify-remote.sh claude/coalition-front --profile extended --app`: see the progress log.
