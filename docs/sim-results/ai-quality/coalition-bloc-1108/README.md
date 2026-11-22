# Wariness, the standing bloc and all-in coalition wars, 2026-11-08

**Status:** genes `runaway_coalition` bits 4 (standing bloc), 8 (strike a hollow runaway), 16 (coalition wars fight at once) and 32 (wary guard). **Default 55 (bits 1, 2, 4, 16, 32; bit 8 off)**, main's call after the 55 arm below (it was 3 when measured). Branch `claude/coalition-bloc`, on `claude/coalition-front` (now on master). Code: `crates/ai/src/runaway.rs`, `diplomacy.rs` (`bloc_size_cost`, `treaty_score`), `operation.rs` (roster home guard), `production.rs` (`desired_army`). Design: `docs/design/ai-strategy/eu4-coalitions-2026-11-08.md` (A1, A2) with the designer's 2026-11-08 revision: a hidden wariness replaces the runaway score as A1's trigger, coalition wars fight all in at once, no cap on bloc size, no hard line anywhere in the coalition path.

## What was built

- **Wariness** (`Planner::wariness(observer, target)`, hidden, public information only): recent captures 0, 3, 8, 30, 45, 55 for 0 to 5 (one or two barely register, a roll-over of three to five does), 20 per elimination (up to 2), a fifth of the visible army's excess over the observer's (up to 30), 6 per VP of lead (up to 24), all scaled from 25% to 100% by nearness (`helper_reach_percent`). Unit test: two captures give at most 8, three give 30 or more, three far away at most 8.
- **No hard line.** Every yes/no choice is `wary_draw`: a seeded stream per purpose, partner and 8-turn bucket, true with the wariness (or score) as its percent. The old runaway cutoff of 60 is gone: `runaway()` is `runaway_read()` (the score) passed through a draw at that score (bits 1, 2 and 8 now act on a 30-point leader 30% of the time, not never); the all-in draw is wariness squared over 100; the bloc waiver is a draw at the weakest member's wariness.
- **Bit 4:** an Alliance, at Peace included, with a partner wary of the same empire is worth 40% of the weaker wariness, 30% of the share of that empire's army the pair can bring and 10 for a partner next door; `bloc_size_penalty` is waived for a bloc wary of one empire (no cap: dogpiles are fine). The existing "allies defend at once" rule makes the bloc a coalition.
- **Bit 16:** at War with an empire that another team outside its side also fights, by the all-in draw, an operation strikes its weakest reachable city with no army-away test, roster radius 20, and the roster keeps one defender in each own city with an enemy unit within 8 tiles (home guard).
- **Bit 8 (A2):** a runaway counting down or read as hollow is struck on any weakly held reachable city with no army-away test and no war under way (the operation declares when assembled).
- **Bit 32:** one extra home troop per city within 10 tiles of the empire we fear most, by draw.
- Not built: same-turn coordination between members (no channel; each member decides from public state), walls on the threat's side (only troops), wariness shown to the player.

## Human-runaway scenario probes

`probes.sh`, `probe-horde.sh`, 64 reseeded runs per arm, window plus 10 turns, every seat Hard on the arm's params ([probes-final.jsonl](probes-final.jsonl), `probe-table.py`). Arms: rc3 = control (the branch default, bits 1 and 2), rc39 = + bits 4 and 32, rc63 = all bits. Means per run. The scenario list is the sweep's survivors plus `byzantium-empty-horde` (restored from `claude/coalition-front`, refreshed to rules 25): `celts-break-the-reserve`, `byzantium-empty-horde`, `celts-stop-the-horde`, `runaway-conqueror`, `war-break-the-hold-t168`.

| Card (runaway) | Arm | Alliances before the first war on it | Wars on the runaway | Second fronts (strikes struck) | Runaway cities taken | Seat objective |
|---|---|---:|---:|---:|---:|---:|
| celts-break-the-reserve (Swiss) | rc3 | 1.00 | 0.03 | 0 | 1.00 | Geneva 0/64 |
| | rc39 | 1.15 | 0.10 | 0 | 1.01 | 0/64 |
| | rc63 | 1.15 | 0.25 | 0.60 (0.06) | 1.00 | 0/64 |
| byzantium-empty-horde (Mongols) | rc3 | 0.14 seat | 2.68 | 0.45 (0.17) | 0.07 | Byzantine cities held 3.20 |
| | rc39 | 1.67 seat | 2.64 | 1.31 (0.98) | 0.04 | 3.01 |
| | rc63 | 1.82 seat | 2.84 | 2.57 (1.60) | 0 (razed 0.81) | 3.12 |
| runaway-conqueror (Mongols) | rc3 | 1.78 | 3.14 | 0 | 0.62 | 39/64 |
| | rc39 | 1.42 | 3.29 | 0.03 | 0.67 | 45/64 |
| | rc63 | 1.42 | 3.28 | 1.29 (0.96) | 0.76 | 47/64 |
| war-break-the-hold-t168 (Athens) | rc3 / rc39 | 1.00 | 1.00 | 0 | 0 | 0/64 |
| | rc63 | 0.85 | 2.00 | 2.06 (0) | 0 | 0/64 |

Read: the bloc forms (Horde: the seat's Alliances 0.14 to 1.7 a run; Reserve +0.15 before the first war), all-in strikes fire (Horde second fronts 0.45 to 2.57, 0.17 to 1.60 struck) and the hollow strike declares (hold card wars on the runaway 1.00 to 2.00), but **they do not take cities**: Geneva is 0/64 in every arm, runaway cities taken do not rise, and city battles per war on the Horde stay at about 3 (rc3 3.3, rc63 2.9). The scenario `celts-break-the-reserve` is hard to reach: the Celts' army fights Sparta and Athens far away, no unit is within 12 tiles of Geneva (the first all-in radius test found `force 0`), and even at radius 20 only 0.06 strikes a run strike. The one scenario that moves is `runaway-conqueror` (human Swiss holds both cities and caps the Mongols at 3 cities: 39 to 47 of 64).

## AI-only census

512 eight-Hard games per arm, same seeds, [census.sh](census.sh) (tapes replayed by `alliance_metrics`; control rc3).

| | rc3 | rc39 | rc63 |
|---|---:|---:|---:|
| Wars per game | 16.49 | 16.31 | 16.31 |
| Alliances formed per game | 5.06 | 6.77 | 6.72 |
| Captures per game | 13.74 | 14.21 | 13.20 |
| Wars with a capture | 46.3% | 46.1% | 43.4% |
| Largest bloc, T50 / T100 (mean) | 1.79 / 2.07 | 1.94 / 2.45 | 1.95 / 2.46 |

AI-only wars per game do not rise (16.49 to 16.31), the old free-bloc failure (36 wars a game) does not return. Blocs grow by about 0.4 teams at T100 (max 6 against 5), and by 0.15 at T50, so single early captures do not spawn blocs in bulk (and the score test above covers one or two captures). Captures per game fall 4% with all bits.

## Human-seat gate

Eight maps, 256 reseeded salts, seed 310001, spot ($0.09 to $0.18 a batch). `gate-cmp.sh`. Control 30.8% (final design).

| Arm | Seat win % | Change, pp [95%] |
|---|---:|---:|
| seat, bits 3+4+32 (39) | 31.4 | +0.63 [-0.88, +2.15] |
| seat, all bits (63) | 30.1 | -0.63 [-2.25, +0.98] |
| rivals, 39 | 32.2 | +1.46 [-0.29, +3.27] |
| rivals, 63 | 32.7 | +1.95 [+0.15, +3.81] |

Rivals do not get harder (the seat does 1.5 to 2 pp better, borderline significant), as in `claude/coalition-front` (+0.54): the gate's seat is rarely the runaway. The seat arm is the AI's own play: neutral. **The first all-in version cost the seat 3.6 pp** (all bits -3.27 [-4.74, -1.86]; every war with a wary enemy another team fought was all-in, floor 20; bit 16 alone -3.56 [-4.88, -2.25]): all-in is expensive when it commits armies to wars the AI loses (the tactics bottleneck). It was repaired by making the all-in a draw on wariness squared (seat 19: -0.29 [-1.2, +0.6] at the 45 line, final design -0.63 for all bits). Earlier arms: [gate-bits-comparison.json](gate-bits-comparison.json), [gate-comparison.json](gate-comparison.json), `gate-arms*.json`.

## Designer seat

Hard in his seat on his tapes, 96 salts, every seat on the measured commit, run locally (control: this branch, default 3; the other two are scratch commits with the default at 39 and at 63; not added to the history chart).

| | Control (3) | Default 39 (bits 4+32) | Default 63 (all) |
|---|---:|---:|---:|
| Held out | 57.4% [55.8, 59.0] | 59.3% [57.7, 60.9] | 58.5% [57.0, 60.1] |
| Training | 36.9% [35.5, 38.3] | 36.2% [34.8, 37.6] | 37.7% [36.2, 39.2] |

The control here is the branch's smooth-draw bits 1+2, not master's cutoff version (nothing compares the two for the default). Files: `designer-seat-ctl.json`, `-39.json`, `-63.json`. `scenarios/lab/byzantium-empty-horde.json` is kept out of the repo (`git show claude/coalition-front:scenarios/lab/byzantium-empty-horde.json`, then `tools/sim/refresh-save-version-fixtures.py OLD NEW`).

## Decision (for main)

Proposed defaults: **bits 4 and 32 (39): yes**, AI-only wars do not rise, the seat gate is neutral (+0.63) and the bloc forms in his scenarios (Horde Alliances 0.14 to 1.7, `runaway-conqueror` objective 39 to 45 of 64); designer seat per the table above. **Bit 16 (all-in): only with the wariness-squared draw, and its value is tactics-limited**: strikes fire 4 times as often and the human-runaway scenario improves (47 of 64), but captures per war do not move, so more all-in wars mostly cost units until battle tactics improve (`claude/training-round2`); I would leave it off until then or accept it on his call for the human-runaway case. **Bit 8: leave off**: it declares (wars on the hold runaway 1.00 to 2.00) but takes nothing (Geneva 0/64, hold card 0/64).

## Default 55 (main's decision, 2026-11-08)

Arm 55 (1+2+4+16+32, no bit 8), same gate and designer seat, control = default 3: seat-only +0.39 pp [-1.17, +1.95], rivals-only +2.00 [+0.20, +3.81]; designer seat held out 57.4% to 58.5% [57.0, 60.1], training 36.9% to 37.4% [35.9, 38.9]. Held-out lower bound is well above -2.0 and the seat is not significantly worse, so the default is 55. See `gate-55-comparison.txt`, `designer-seat-55.json`. 

**Smooth draw against the 60 cutoff for the old default (bits 1+2), human-seat gate, same 8 maps x 256 reseeded salts:** the cutoff arm (a scratch build that restores master's `score >= 60` line in `runaway()`, gene bit 64, not committed) against the branch's draw (control 30.8%): seat-only 30.7%, -0.05 pp [-0.98, +0.93]; rivals-only 30.5%, -0.24 pp [-1.61, +1.07]. So the draw is indistinguishable from the cutoff on the gate (`gate-cutoff-comparison.txt`). The Horde card's second fronts under the draw (0.45 a run at control) are close to the cutoff-era 0.64.
