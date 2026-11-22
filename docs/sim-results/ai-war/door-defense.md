# Defend where the enemy army is: gene `door_defense` (W2, idea #23)

## Round 2 (2026-10-28): gate, attribution, AI-vs-AI census

Branch `claude/w2-door-defense` at `0149180bd` (master `6953d123a` merged: the landed fixtures and the gate fix `faabbc701`, rivals play plain Hard). `door_parts` defaults to 29 (buys, production, strikes, garrisons; no upgrades, no retake). All local, nice 15, 6 threads. The AWS gate wasn't run: launching spot servers was refused by the session's permission check, so the gate below is a quarter-size local run.

### Human-seat gate (W2.0 method, local, 64 salts per map = 512 games per arm)

Control 78/512 (15.2%; the frozen AWS control is 16.2% on 2,048). Paired with `tools/ai-quality/w04_gate.py`, positive = the seat wins more.

| Arm | Seat Δ pp [95%] {adjusted} | Germany map (ctl 7/64) |
| --- | --- | --- |
| Seat only | −1.2 [−4.5, +2.2] {−6.2, +3.9} | 10/64 |
| Rivals only | +1.0 [−2.8, +4.7] {−4.6, +6.6} | 11/64 |
| All seats | −0.2 [−4.0, +3.6] {−5.9, +5.6} | 8/64 |

Neutral within ±4 pp in both roles. Rivals with the gene don't make the seat lose more (the designer-facing measure), and the seat with it doesn't win more. The full 2,048-game AWS run (`docs/sim-results/ai-war/w21/arms.json`, about $0.4) would narrow this to about ±2 pp.

### Attribution: each part off in turn (war suite, 28 cases × 64, gene on the seat, all parts = 63)

| Arm | Mean | What moves |
| --- | --- | --- |
| Gene off | 45.8% | |
| All parts | 51.8% | |
| No buys | 48.2% (−3.6) | Golden Horde hold 100 → 1 |
| No upgrades | 52.4% (+0.6) | Japan T142 73 → 78, Athens T112 9 → 15, German air early 21 → 28 |
| No production | 51.6% (−0.2) | German stand 89 → 85, Berlin T65 100 → 95 |
| No strikes | 51.9% (+0.1) | Japan T142 73 → 76 |
| No garrisons | 48.7% (−3.1) | Japan T43 96 → 28, Japan T142 73 → 64 |
| No retake | 52.1% (+0.3) | Japan T142 73 → 84 |
| **Default 29** (no upgrades, no retake) | **52.3%** | Japan T142 76, Athens T112 15, German air early 28 |
| Default 29, every seat | 42.9% | Golden Horde 96 → 0, thin 87 → 0, Walls Rise 81 → 0 |

**The retake fires** (Berlin T65: 2 of 6 traced runs, Germany assaulting a Spartan city at (19,15) with 2–3 units), but no case gains from it, and with it running outside fronts Japan T142 lost 11 points.

### AI-vs-AI census (512 all-AI 8-empire Hard games, Standard map, seeds 7300001+)

| Measure | Off | On (all seats, default parts) |
| --- | --- | --- |
| City captures per game | 14.6 ± 0.5 | 15.0 ± 0.6 |
| Eliminations per game | 0.64 ± 0.06 | 0.69 ± 0.06 |
| Game length, turns | 261 ± 4 | 264 ± 4 |
| Wars declared / with battles | 25.4 / 21.5 | 25.5 / 21.7 |
| Sieges abandoned | 12.1 | 12.5 |
| Wins best / worst faction | Mongols 106 / Celts 20 | Athens 113 / Celts 18 |
| Sparta wins, captures per game | 89, 4.96 | 73, 5.11 |
| Germany wins | 29 | 41 |

Conquest between AIs doesn't collapse: captures, eliminations and length are unchanged. The Golden Horde and Walls Rise zeros are specific fixtures where an AI defender with a deep treasury (the Swiss with 2,235 Gold) can now buy the counter at the door. In full games treasuries are smaller and attacks still land.

### Adaptive Sparta (Hard in his seat from T40, 32 salts, default parts)

Control 19/32 · gene in the other AIs 10/32 · gene in every seat 14/32 (cities Sparta took 12.1 / 10.7 / 11.4).

## Round 1 (2026-10-27)

First measured version, 2026-10-27, branch `claude/w2-door-defense` (master `1e481f8c3`, playtest 8.9.14, plus the 10-27 fixtures merged from `45c102d79`). Design: `crates/ai/src/door.rs`, rules reference `legatum-rules` §10. All runs local (M3 Max, 6 threads, nice 15), 64 runs per case, seeded by index.

## War suite, gene on the scenario's seat (`--params-seats seat`)

| Case (seat) | Off | On (100) |
| --- | --- | --- |
| **Defence** | | |
| Golden Horde hold (Swiss vs Mongols) | 1% | **100%** |
| Germany's last stand | 85% | 89% |
| Japan v Sparta T142 | 84% | 84% |
| Japan v Hoplites T43 (new) | 71% | **98%** |
| Germany Berlin T65 (new) | 95% | 100% |
| Athens two Pikemen T97 (new) | 100% | 100% |
| Athens two Pikemen T112 (new) | 14% | 9% |
| German air early / T214 / T219 | 23 / 100 / 0% | 21 / 100 / 0% |
| Mongol walls | 100% | 100% |
| Mongol bombers T150 (new) | 14% | 20% |
| **Attack checks** | | |
| Golden Horde | 96% | 98% |
| Golden Horde, thin treasury | 87% | 89% |
| Sparta's First Blood | 100% | 100% |
| **Suite** (28 cases) | 47.9% mean, 12 at parity | 52.8% mean, 14 at parity |

- **Gene off reproduces master run by run** (21 original cases against a pre-change master build, and all 28 against the merged branch), and a parameter file with the gene at 0 matches the built-in path.
- **Unchanged elsewhere:** the other 13 cases are identical.
- **Golden Horde hold:** the Swiss sat on 2,235 Gold and bought one Spearman when Aarau was at 13 HP. With the gene they buy four Horsemen (the counter to the Crossbowmen in front of them) in four turns for about 550 Gold, and keep the rest of the treasury.
- **Gene in every seat** (`--params-seats all`, information only): the AI attackers lose to AI defenders that now defend. Golden Horde 96% → 0%, Walls Rise 81% → 0%, Japan T142 84% → 73%; mean 42.7%.

## Adaptive Sparta (the coach's check), Hard in his seat from T40, 32 salts

Runner: the coach's what-if (`~/LegatumRuns/coach-tools/coach-8913-athens/coach8913_whatif.rs`) built on this branch with the 8.9.13 Sparta tape; warm-up 0 hash mismatches.

| Arm | Sparta wins | Cities Sparta took | Cities Sparta lost |
| --- | --- | --- | --- |
| Control (this branch's defaults) | 19/32 | 12.1 | 3.5 |
| Gene in every other AI | **12/32** | 9.8 | 2.9 |
| Gene in every seat, Sparta included | 20/32 | 12.0 | 2.7 |

The gene doesn't make the adaptive attacker stronger: with it only in the defenders, Hard Sparta wins less (the quota and floor arms in the review went 7 → 16 and 31 of 32).

## Cost

Seat planning time per turn (native, `war_trace`, run 0), off → on: Japan T142 3.0 → 2.8 ms, Golden Horde hold 2.2 → 2.7 ms, German stand 1.1 → 1.1 ms, Japan T43 0.6 → 0.6 ms.

## What was tried and dropped

Reinforcing empty threatened cities with units two or three turns away, or with garrisons of cities without a front, gave German stand 89% → 95–98% but took Japan T142 from 84% to 67–68% (walkers picked off, emptied cities taken next). Only units that get in this turn go.
