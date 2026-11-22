# Walls at 100 and the end of the Imperial Scriptorium (2026-11-06)

The designer chose a 100 Production base Walls price and cut Byzantium’s Imperial Scriptorium. This report measures both decisions together against the exact master commit at the start of the task. Master already had the shared Hard `walls_garrison=1` default; the 11-05 Walls-price arm B is not a control for this result.

## Change

- `RULES_VERSION` 13 → 14. Ordinary Walls cost 100 Production; the Swiss Alpine Redoubt keeps its +20 adjustment and costs 120. Castle and Star Fort retain their fixed prices. `walls-cost=N` remains usable, with 100 now the no-op default.
- Byzantium builds the ordinary 80-Production Library with the ordinary name and art. Its extra diffusion, faction-rule switch, AI gene and app presentation were removed. Mercenary Companies, thin levies, Tagmatic Soldier and the Heir remain. Faction-rule mask bit 17 is vacant; `ByzantiumGoal` is explicitly bit 18, so later bit positions do not move.
- Current-rule saves, UI rules, scenario states and determinism tapes were regenerated. Historical saves were left at their original versions.

## Protocol

Control: `f8bde944e8a0bc79dcb3f38a7fad8e23741c7389` (`fe1b2aa6bcfb3e90e1529a98fcf2e5cd925cbf3febd757d8a8d89b6485194c90` Linux sim SHA-256). Candidate: merged branch `9f271ab8c454783c29eafcba1ad302d145fc7a1e` (`82076093c5090b65b577ec5a8102e902b02bd9cf898a559e920dec7e85200bb6` Linux sim SHA-256). The later branch commits format source, refresh fixtures and add this report; they do not change the measured engine.

`tools/cloud/build-linux.sh` built each binary. `tools/cloud/run-batch.sh --count 1 --max-hours 1 -- --players 8 --difficulty hard --games 4096 --seed SEED` ran each binary on each seed set: 9311003–9315098 and held-out 8241001–8245096. All four batches completed 4,096 games with zero panics. Run directories are under `~/LegatumRuns/cloud/`: `walls100-{control,candidate}-{9311003,8241001}-1106-*`. [The existing census comparator](../ai-quality/swiss-playbook-1105/census.py) supplies paired faction win-share differences and 95% intervals from 4,000 game bootstrap resamples (unadjusted across factions). [This analyzer](analyze.py) validates the paired seat schedule and computes the other metrics. First-Walls turn is the median among seats that completed Walls; captures before T60 include recaptures; cities lost uses every seat-game, and eliminations count eliminated seats.

## AI-only census

Win share is wins per faction seat. Each interval is for candidate minus control in percentage points.

| Faction | Main control | Main candidate | Main Δ [95% paired interval] | Held-out control | Held-out candidate | Held-out Δ [95% paired interval] |
|---|---:|---:|---:|---:|---:|---:|
| Athens | 17.04% | 17.50% | +0.47 [-0.99, +1.98] | 16.48% | 16.21% | -0.28 [-1.62, +1.13] |
| Aztecs | 13.55% | 12.92% | -0.63 [-1.99, +0.77] | 12.68% | 12.68% | +0.00 [-1.37, +1.26] |
| Byzantium | 13.77% | 13.24% | -0.52 [-1.87, +0.80] | 13.26% | 12.27% | -0.99 [-2.27, +0.30] |
| Celts | 11.75% | 11.15% | -0.60 [-1.87, +0.60] | 9.68% | 10.69% | +1.02 [-0.16, +2.14] |
| Germany | 16.01% | 15.65% | -0.36 [-1.72, +1.01] | 15.91% | 16.18% | +0.27 [-1.03, +1.65] |
| India | 10.67% | 10.50% | -0.16 [-1.42, +1.09] | 11.94% | 9.80% | -2.14 [-3.38, -0.93] |
| Mongols | 13.82% | 14.99% | +1.18 [-0.19, +2.52] | 16.08% | 17.29% | +1.21 [-0.22, +2.70] |
| Sparta | 8.14% | 9.90% | +1.75 [+0.63, +2.91] | 8.90% | 10.45% | +1.54 [+0.36, +2.78] |
| Switzerland | 7.73% | 6.59% | -1.14 [-2.19, -0.06] | 7.49% | 6.86% | -0.64 [-1.66, +0.44] |

| Metric | Main control | Main candidate | Held-out control | Held-out candidate |
|---|---:|---:|---:|---:|
| Best/worst win share | 2.20× | 2.66× | 2.20× | 2.52× |
| Median first Walls turn | T29 | T33 | T29 | T32 |
| Captures before T60 / game | 1.031 | 1.171 | 1.016 | 1.213 |
| Cities lost / seat-game | 1.644 | 1.729 | 1.627 | 1.699 |
| Eliminations / 32,768 seats | 3022 | 3349 | 2952 | 3266 |

Sources: [main paired census](census-main.json), [main metrics](summary-main.json), [held-out paired census](census-heldout.json), [held-out metrics](summary-heldout.json).

**Reading.** Walls rise 4 turns later on the main seeds and 3 turns later held out. Captures before T60, city losses and eliminations all increase in both sets. Sparta gains +1.75 points [ +0.63, +2.91 ] on the main set and +1.54 [ +0.36, +2.78 ] held out. Switzerland loses −1.14 [ −2.19, −0.06 ] on the main set; India loses −2.14 [ −3.38, −0.93 ] held out. Best/worst widens from 2.20× to 2.66× and 2.52×. These runs combine the price rise and Scriptorium cut, so they cannot assign a faction change to either rule alone. The intervals are descriptive per faction; no multiple-comparison correction was applied.

## Human-seat gate

`tools/sim/quality_gate.py` froze the eight-map control at 512 salts, then repeated its first 128 salts with the same source, binary and game hashes. The combined candidate stopped at the first predeclared look: 128 salts × 8 maps = 1,024 paired games. The human seat won 183/1,024 control games (17.87%) and 240/1,024 candidate games (23.44%), a +5.57 percentage point paired difference. The descriptive 95% interval is [+2.43, +8.70] points; the gate's simultaneous interval is [+0.88, +10.26] points. Its default seat-only calculation says `noninferior`, but the [all-seats interpretation](gate/all-seats-comparison.json) correctly says `role-mixed`: the Walls price changes the human and rival seats together. The win difference describes the combined game, not an isolated AI-strength effect. The frozen [protocol](gate/protocol.json), [control](gate/control/result.json), [repeat](gate/repeat/result.json), [candidate](gate/candidate/result.json), and [seat-only calculation](gate/seat-comparison.json) are retained.

## UI review

Playwright screen-tour shots show the [Byzantium sheet at 1000×600](screenshots/1000x600/byzantium-faction-sheet.png) and [1440×900](screenshots/1440x900/byzantium-faction-sheet.png), plus the [100-Production Walls build card at 1000×600](screenshots/1000x600/walls-build-card.png) and [1440×900](screenshots/1440x900/walls-build-card.png). The sheet has no Scriptorium row and the card quotes 100 Production. All four were inspected at their native size.

## Checks

`cargo fmt --all`, `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`, and `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked` passed (2,255 tests; 17 skipped). `npx vitest run` passed (3,192 tests; 2 skipped), as did `npx tsc -b --noEmit`. The full `npx playwright test` run had 1,001 passed, 25 skipped and two failures at the two viewport sizes: `piece-compare-card.spec.ts` still expected Byzantium's removed Library comparison. That obsolete case was removed; the focused `npx playwright test e2e/piece-compare-card.spec.ts` then passed all four tests. The four targeted screen-tour checks passed. The full suite was run once, before this test-only correction. Repinned runner pins and 12 determinism tests passed.
