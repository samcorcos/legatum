# Human-seat benchmark baseline, `playtest-8.9.7` (2026-10-22)

Hard in every seat (`--brain hard`, no params, so seat-only, opponents-only and all-seats are the same run), the eight `maps.json` maps, **256 salts each (salts 1-256), dice reseeded per salt** (`LEGATUM_HVH_RESEED_DICE=1`). 2,048 games. Engine: tag `playtest-8.9.7` (`5637d8115`). Master has had no `crates/` change since (the six later commits are docs and one e2e), so this is also master's engine at `0ac97a0f8`.

Run: `tools/cloud/run-batch.sh --count 1 --bin hard_vs_human --env LEGATUM_HVH_RESEED_DICE=1 -- --brain hard --games 256 --seed 1`, one c8g.16xlarge spot server, 28 minutes, about $0.31. Per-game results: `~/LegatumRuns/cloud/baseline-897-1021-223214/slice-0/out/hard-vs-human-games.json.gz` (not committed; 1.1 MB, regenerates in 28 minutes). Per-faction numbers come from `faction_table.py` in this folder, reading the `crowns` list each game carries (every empire's faction, seat and result).

**Never compare these numbers with a non-reseeded run** (README: the reseeded control scores lower).

## Human seat, per map

Overall (mean of the eight map win rates): **38.4%**, 95% interval [36.4%, 40.2%]. Median milestone ratio (AI over the user's cities and VP): 0.45.

| Map (user's seat) | Rivals | AI wins in the seat | AI median win turn (user's) | Milestone ratio |
|---|---|---|---|---|
| 8.3 Switzerland | Mongols (Search) | 24.2% | 283 (91) | 0.75 |
| 8.4 Sparta | Japan, Byzantium, Germany | 49.6% | 343 (61) | 0.40 |
| 8.4 Japan | Switzerland, Japan, Byzantium | 58.2% | 304 (162) | 0.50 |
| 8.4 Celts | Sparta, Athens, Switzerland | **5.9%** | 331 (149) | 0.33 |
| 8.5 Mongols | Mongols, Sparta, Athens | **3.9%** | 374 (123) | 0.10 |
| 8.5 Switzerland | Athens, Switzerland, Japan | 80.9% | 268 (110) | 0.57 |
| 8.6 Germany (user lost) | Sparta, Athens, Switzerland | 34.8% | 346 (user lost) | 1.00 |
| 8.6 Sparta | Japan, Byzantium, Germany | 49.6% | 330 (84) | 0.50 |

The AI wins its seat on the user's maps about twice as often as a fair 4-player share on the Switzerland and Japan maps, and almost never as the Celts or the Mongols. The map decides a lot (both Sparta maps read 127/256).

## Per-faction win rates (diagnostic)

Win rate per appearance in these eight maps: "in the seat" when the faction holds the user's recorded seat, "as an opponent" for each AI-held copy. Fair share is 1/players on each map, so the **index** is win rate times players, averaged over appearances (1.0 is a fair share; the 2-player map counts double). Flag: index below a third of the average (0.89 / 3 = 0.30).

| Faction | In the seat (games) | As an opponent (games) | All appearances | Index | Flag |
|---|---|---|---|---|---|
| **Celts** (Celts) | 5.9% (256) | none | 5.9% | 0.23 | **below a third** |
| Mongols (Mongols) | **3.9%** (256) | 40.4% (512) | 28.3% | 0.62 | seat rate is the 3% case, see below |
| Germany (Germany) | 34.8% (256) | 9.4% (512) | 17.8% | 0.71 | |
| Athens (Athens) | none | 20.3% (1,024) | 20.3% | 0.81 | |
| Japan (Japan) | 58.2% (256) | 11.2% (1,024) | 20.6% | 0.82 | |
| Switzerland (Switzerland) | 52.5% (512) | 13.7% (1,024) | 26.6% | 0.98 | |
| Byzantium (Byzantium) | none | 27.0% (768) | 27.0% | 1.08 | |
| Sparta (Sparta) | 49.6% (512) | 43.9% (768) | 46.2% | 1.85 | |

Read it with care: the benchmark has 2 to 4 empires per map and **each faction sits in only two to five of them**, so a faction's rate is mostly its maps' starts and its neighbours. The Mongols win 76% against Switzerland (8.3) but 4% to 5% in both seats of the 8.5 map, where the Spartan AI wins 70%. The Celts have one map. Per map and seat: 8.3 Switzerland 24 / Mongols 76; 8.4 Sparta Sparta 50, Japan 14, Byzantium 21, Germany 15; 8.4 Japan Japan 58, Swiss 3, Japan (AI) 3, Byzantium 36; 8.4 Celts Celts 6, Sparta 55, Athens 21, Swiss 19; 8.5 Mongols Mongols 4, Mongols (AI) 5, Sparta 70, Athens 21; 8.5 Swiss Swiss 81, Athens 13, Swiss (AI) 1, Japan 5; 8.6 Germany Germany 35, Sparta 7, Athens 27, Swiss 32; 8.6 Sparta Sparta 50, Japan 23, Byzantium 24, Germany 4 (percent).

### Cross-check without map effects: 8 empires, every faction once

2,048 games of Hard in all eight seats on the 48x32 map, shuffled factions, Standard speed (`legatum-sim --players 8 --difficulty hard --games 2048 --seed 9210001`, same engine, 2 minutes, about $0.02; `~/LegatumRuns/cloud/factions8-897-1021-230146`). Fair share is 12.5%.

| Faction | Wins | Share | Against fair |
|---|---|---|---|
| Byzantium (Byzantium) | 597 | 29.2% | 2.33x |
| Athens (Athens) | 346 | 16.9% | 1.35x |
| Mongols (Mongols) | 332 | 16.2% | 1.30x |
| Switzerland (Switzerland) | 282 | 13.8% | 1.10x |
| Sparta (Sparta) | 214 | 10.4% | 0.84x |
| Germany (Germany) | 165 | 8.1% | 0.64x |
| **Japan (Japan)** | 63 | **3.1%** | **0.25x, below a third** |
| **Celts (Celts)** | 49 | **2.4%** | **0.19x, below a third** |

Best over worst is **12.2x** (pillar 8 asks for at most 1.5x). On 2026-10-17 the same measure read Japan 2.9% and the Mongols 7.6%, so Japan has not moved and the Mongols have improved.

## What this says

1. **Two factions are below a third of the average on the balanced measure: Japan (3.1%) and the Celts (2.4%)**; the Celts also fail it on the user's maps (5.9% in their one seat, index 0.23).
2. **The Mongols' seat rate (3.9%) is a map effect, not a weak faction:** they win 16.2% of balanced 8-player games and 76% of the 8.3 duel. On the 8.5 map the Spartan neighbour takes 70%. Treat it as a map to study, not as a faction to tune.
3. **Germany is third weakest** on both measures (8.1% balanced, index 0.71 here) but above the flag.
4. **Byzantium (29%) and Sparta (46% on the user's maps) are the strong end.** A fix for the weak end that only weakens them would break pillar 8's "never weaken strengths" rule; this is diagnostic only.

Weakest three for the stage 2 climbs (`ai-quality/climb-1021/`): the Celts, Japan, Germany by the balanced measure; the Mongols were climbed as a fourth because the user's seat rate flags them.
