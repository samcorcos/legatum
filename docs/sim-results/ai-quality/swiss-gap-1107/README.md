# Swiss gap, 2026-11-07

## Matched census and regression

All percentages in the table are Swiss wins divided by games with a Swiss seat. The 11-05 `swiss_playbook` 2 experiment and the nightly census use the same eight-player Hard AI-only setup, 4,096 games starting at seed 9311003, with eight of nine factions in each game. **Their published metrics differ:** the playbook report divides by Swiss seats (3,610), while nightly `win_share_by_faction` divides by all winning seats (4,096). The fair conditional Swiss-seat win rate is 12.5%; the fair share of all game winners is 11.11%. Thus the original 11.58% per Swiss seat is 418/4096 = 10.21% of all winners, and nightly v16's 6.71% of all winners is 275/3610 = 7.62% per Swiss seat. The 11.58% result was measured on `90c9f995-dirty` with the Swiss gene enabled alone, not on a master commit with every later faction playbook. A matched current-master control and gene-off control had identical winners in all 4,096 games on both seed sets.

| Source / transition | Swiss, seed 9311003 | Best/worst | Interpretation |
|---|---:|---:|---|
| `90c9f995-dirty`, Swiss playbook 2 alone | 418/3610 = 11.58% | 2.71 | Original experiment |
| `5bf1b6fec`, combined faction playbooks | 388/3610 = 10.75% | 3.15 | Other playbooks compete; -0.83 pp |
| `70713524d`, shared Walls garrison + lean war army | 272/3610 = 7.53% | 3.14 | -3.22 pp; Swiss already had both valuations, so this is stronger rivals |
| Pre-Walls-price master control `f8bde944` | 279/3610 = 7.73% | 2.20 | Other intervening AI changes; not a single-variable ablation |
| Walls 100 + Scriptorium cut, branch that landed as `9f271ab8` | 238/3610 = 6.59% | 2.66 | -1.14 pp paired, 95% bootstrap interval [-2.19, -0.06]; two rules changed together |
| Rules v15 batch, including no world-wonder buying | 245/3610 = 6.79% | 2.59 | +0.19 pp on this seed; held-out -0.28 pp, so no stable Swiss effect |
| Nightly v14 `810feb2b3` / v16 `66dd75801` | 279/3610 = 7.73% / 275/3610 = 7.62% | 2.25 / 2.32 | Converted every faction to its seat denominator; published Swiss 6.81% / 6.71% and ratios 2.27 / 2.33 used all winners. Core forecast truth and related changes are small in aggregate, not isolated |
| Current master `b50d839c1`, rules v19 | 247/3610 = 6.84% | 2.48 | Fresh matched control; held-out 291/3616 = 8.05%, ratio 2.12 |

The triple Swiss upkeep rule predates the 11.58% experiment, so it did not cause this fall. The isolated Swiss hoard release raised the share in its earlier 4,096-game study (+0.44 pp main, +1.33 pp held-out). The shared garrison change is the largest identified step: its 4,096-game master control and flipped master reproduced the prior lab's games exactly (`docs/sim-results/strategy-gap-1105/README.md`). The Walls study was paired on both seeds but combines the Walls price and Byzantine building adjustment (`docs/sim-results/walls-100-1106/README.md`).

On nightly's all-winners denominator, the fresh rules v19 control is 247/4096 = 6.03% and the shipped-gene arm is 284/4096 = 6.93% on the main seed. The paired conditional-seat comparison in the arm table is the more direct measure of Swiss playing strength.

## His 8.9.24 seat

The source save was copied and SHA-256 checked in `~/LegatumRuns/swiss-gap-1107/`; all work used that copy. Replay on the recording engine verified 160 turns with no hash mismatches. The fresh T1-T70 event-window diff used 48 paired salts, five-turn windows, and reached T161 with 0 of 633 AI strategic-order mismatches. Its raw data and summary are in `~/LegatumRuns/swiss-gap-1107/early-windows*`. That run selected the T41 Settler window; the T45 and T58-T59 entries below are the coach review's earlier single-turn decision diff and purchase ablation, not additional selected windows in the fresh run. The recorded-engine takeover curve in `docs/playtests/reviews/2026-11-02-8.9.24-swiss.md` shows Hard from T1 winning 0/32 and being eliminated in 56%; T40 4%, T42 46% after the Settler capture, T50 56-66%, T70 84-88%, T90 94-97%. Absolute probabilities from the current engine are not comparable to that old curve because rules and AI have since changed; candidate/current-control comparisons below use the same current engine and paired salts.

The later `tools/decision-audit/takeover_gap.py` run on the copied save and already hash-verified states measured a fresh current-engine curve at 48 salts per point (raw output: `~/LegatumRuns/swiss-gap-1107/takeover-gap-current/`). This is after rules v20 and the Mongol horde default and uses a different salt set from the 32-salt candidate takeovers below.

| Start turn | 1 | 20 | 30 | 35 | 40 | 42 | 44 | 46 | 50 | 70 | 90 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Hard seat wins / 48 | 5 | 0 | 1 | 6 | 20 | 12 | 34 | 36 | 16 | 46 | 48 |

The largest rise on those sampled branch turns is T50 to T70 (+62.5 pp), after the Lugano siege and colony growth. The new tool's T50-T70 event window found production, settling, garrison and purchase differences across 21 turns but produced **zero valued swap runs**, so its counts alone do not measure the value of any decision. Its two-branch trace records site-score rejections on 15 seat-turns and unaffordable decisive buys on 14; these are gate counts, not effect estimates. The valued T41 window and the coach's earlier T45 and purchase ablations remain the causal estimates below.

| Early divergence | Measured cost to Hard in his seat | Mechanism |
|---|---:|---|
| T37-T41 Settler shadow, declaration and capture | Human window +54.4 pp win-chance [46.8, 62.0] and +3.6 VP [3.1, 4.1] across 48 salts | He stole the Mongol Settler T41 and founded Biel T44. Hard from T40 declared but captured nothing, leaving 3 cities at T50 against the Mongols' 4. |
| T45 Settler queue vs Hard Soldier | Human order +1.2 VP [0.5, 1.9]; win-chance +2.1 pp with wide interval | The opening land pipeline creates a large compounding production lead: human 4 cities T50 / 9 T100 versus original Hard 2 / 3. His Masonry-first T1 research queue matters on this map, but unconditional Masonry-first previously hurt the broad Swiss census. |
| T58-T59 two Composite Archer buys vs Hard's Soldier/Horseman preference | Removing either human purchase loses Lugano by T75 in 32/32 replays; one-turn T59 category swap +0.6 VP and +2.1 pp with wide interval | Lugano was unwalled at 30/200 HP in the raid. Gold at the door, then Walls, saved the capital. Hard's early spending leaves less cash for this defense. |

The Swiss kit supports the human strategy through toll and interest income and a 175% Gold purchase quote. It does not by itself create his land lead. He banked 835 Gold at T50, 1,280 at T100 and 7,387 at the finish, but made only seven purchases; two Archers and the later Bomber were decisive. The Redoubt protected colonies, while the capital's Walls arrived only on T63.

## Valuation arms

All arms below use the same `dd7f3177` binary and explicit `AiParams` for Hard Switzerland only; rules and non-Swiss gene defaults are unchanged. Raw censuses are under `~/LegatumRuns/cloud/swiss-gap-*` and paired comparisons are the `census*.json` files here. The `swiss_playbook` 3 opt-in war-quota arm prices a Redoubt as a garrison even at war and keeps a four-unit floor. Expansion A/B/C increase Settler-chain priority and lower the site floor (A/B also raise Settler production weight). Market D raises the Swiss Market priority while the treasury goal is pending. A default is conditional on higher Swiss share on *both* seed sets without worsening best/worst ratio.

| Arm | 9311003 Swiss | Ratio | 8241001 Swiss | Ratio | T1/T40/T50 seat wins, 32 salts | Decision |
|---|---:|---:|---:|---:|---|---|
| Control, `swiss_playbook` 2 | 6.84% | 2.48 | 8.05% | 2.12 | 2 / 19 / 15 | Current master |
| Opt-in war quota, level 3 | 6.95% | 2.41 | 8.19% | 2.12 | 3 / 24 / 8 | Census gains too small; T50 harmed |
| Expansion A | 7.89% | 2.22 | 7.94% | 2.13 | 4 / 15 / 10 | Held-out share and ratio fail |
| Expansion B | 7.95% | 2.13 | 7.88% | 2.13 | — | Held-out share and ratio fail |
| Expansion C → `swiss_playbook` 3 default | 7.87% | 2.16 | 8.19% | 2.06 | 3 / 18 / 12 | Both census and ratio gates pass; seat gate noninferior |
| Market D | 7.78% | 2.11 | 7.58% | 2.21 | 4 / 20 / 7 | Held-out share and ratio fail; T50 harmed |

The 256-salt × eight-map human-seat gate for the war-quota arm gave the Swiss candidate seat 491 wins versus 474 control (+0.83 pp [0.24, 1.42]) and the rivals arm 485 versus 474 (rivals 0.54 pp softer, interval crosses zero); seat noninferior, rivals inconclusive. This does not override the poor T50 takeover. Expansion C's seat-only arm won 472/2,048 versus 474 control (-0.10 pp [-0.53, +0.33]), noninferior. Its rivals-only arm gave the human seat 497 wins versus 474 (+1.12 pp [0.02, 2.22]), so Swiss rivals may be softer; the simultaneous interval crosses the gate boundary and the result is **inconclusive**. The promotion uses the explicit two-seed Swiss-share and faction-ratio rule, not a claim that this rivals gate proves strength. Market D failed its census gate, so no human-seat gate is needed for it.

The opt-in param arm for Expansion C has the same effective Swiss decisions as shipped `swiss_playbook` 3: level 2's shared Settler weight 100, plus chain priority 1500 and site floor 40. The code keeps Easy and Normal at level 2. Rules v20 and the Mongol horde AI default merged after these rules v19 measurements; their effects are not included in these estimates. The Swiss remain below the 12.5% fair rate per Swiss seat and the 8.9.24 T50 takeover trends worse in 32 salts, so this is a partial correction rather than an explanation for all four human wins.
