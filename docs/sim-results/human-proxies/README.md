# Mongols and Sparta human proxies

![Portrait chart of fixed-map results](head-to-head.png)

This compares two playbooks on one fixed map, not faction balance across maps. The source setups are [8.4](8.4-setup.json) and [8.5](8.5-setup.json). The 8.5 human tape replays with matching initial and all 123 turn hashes. Only the two opt-in proxy brains changed; default AI, rules, and pins did not.

## Source checks

The checkout baseline was measured at e4d30c49 immediately before these proxy edits. The published v8 rows come from the earlier archived validation; this distinction matters because the Mongols baseline shifted with intervening engine changes. T62 snapshots include all captures and foundings resolved on that turn.

| Sparta on 8.4 source | Cities T30 / T50 / T62 | Level-3 veterans T60 | First war | Outcome |
| --- | ---: | ---: | ---: | --- |
| Human tape | 2 / 5 / 7 | 4 at T61 | T26 | Elimination T62 |
| Published v8 proxy | 2 / 4 / 4 | 0 | T24 | Lost T182 |
| Checkout baseline | 2 / 4 / 4 | 0 | T24 | Lost T182 |
| Revised proxy | 1 / 3 / 5; sixth at T73 | 2 | T24 | Elimination T213 |

| Mongols on 8.5 source | Cities T30 / T50 | VP T60 | Sparta Settler steal | Walls / Hall | Outcome |
| --- | ---: | ---: | ---: | --- | --- |
| Human winning tape | 2 / 5 | 7 | T28 | Capital Walls bought T42; Hall T91 | VP hold T123 |
| Published v8 proxy | 2 / 3 | 2 | none | First Walls T41; Hall T103 | Lost T125 |
| Checkout baseline | 2 / 5 | 5 | none | First Walls T44; Hall T100 | Lost T160 |
| Revised proxy | 2 / 7 | 7 | T29 | Capital Walls bought T32; Hall T101 | VP hold T111 |

Mongols now queues Quarry and Granary at T2, then Farm and five Settlers from T12. One Scout watches a known rival's settlement corridor while other Scouts keep exploring. It sees the Sparta Settler at T27, captures it at T29, accepts offered peace after the raid, protects the capital, and builds the Courier Hall. It exceeds the human city count at T50 and buys Walls ten turns earlier, so its win is closer in mechanism than in exact tempo.

Sparta finds and attacks the nearest neighbour first, sends a Scout to identify the next one after committing, then captures three Japan cities before turning on Germany. It keeps two level-3 veterans at T60, one short of the three-veteran target and two short of the human tape. It reaches its sixth city at T73, eleven turns after the T62 target, and its elimination win is much later than the human's. The early-Wall abort and later concentrated siege remain forecast-gated; this source run eventually takes two walled cities, at T208 and T212. These differences still limit how directly the proxy duel answers which human playbook would win.

The [source validation traces](source-validation.jsonl.gz) contain the four published checks, both checkout baselines, and both revised source runs. Every revised decision derives from the acting player's fogged view.

## Fixed-map head-to-head

All 1,024 games use the 8.5 Standard 40×26 setup and seed 874654782136188 with barbarians on. The original keeps Mongols proxy in the human seat 0 and Sparta proxy in seat 2; the mirror swaps those factions and proxy brains. Seat 1 remains Mongols Hard + Search and seat 3 Athens Hard + Search, with the app's four-player Search settings and alliance handling. The simulator asserts that the mirror preserves map tiles, starting Settler positions, and player control flags. Only the Hard brains' RNG seeds vary over salts 1–512 in each layout; core map, setup, combat, ruin, and barbarian streams retain the recorded seed. Wilson 95% intervals describe variation across these AI salts on this one map.

| Layout (512 each) | Mongols wins (95% CI) | Sparta wins (95% CI) | Other Hard wins (95% CI) | T600 stalls |
| --- | ---: | ---: | ---: | ---: |
| Original | 0 (0–0.7%) | 6 (1.2%; 0.5–2.5%) | 482 (94.1%; 91.8–95.9%) | 24 |
| Mirror | 22 (4.3%; 2.9–6.4%) | 9 (1.8%; 0.9–3.3%) | 420 (82.0%; 78.5–85.1%) | 61 |

All six original Sparta wins were eliminations, with median winning turn T363.5. The other 482 original wins were Hard-seat VP holds, median T185. In the mirror, the 22 Mongols wins, 9 Sparta wins, and 420 Hard wins were all VP holds; their median winning turns were T365.5, T264, and T201. These medians describe completed wins only.

Sparta declared on Mongols in every game, at median T38 original and T68 mirror. In the original it declared before any Mongols Wall in all 512 games and took a Mongols city at median T41, also before the first Wall in every game. Mongols built no Courier Hall and stole no Sparta Settler there. In the mirror, Sparta declared after Mongols had a median three walled cities; it took a city at median T73. The first city it captured was unwalled in every game in both layouts, so these openings do not measure an assault against a walled Mongols capital. Mongols completed a Hall in 384 mirror games (median T165) and stole an Sparta Settler in 35. The seat swap changes starting position and faction placement together; these are observed associations, not isolated effects of Walls, Hall, or raid.

The reproducible outputs are [original games](original.jsonl.gz), [mirror games](mirror.jsonl.gz), and [summary](summary.json). From the repository root, `python3 tools/sim/human-proxy-report.py docs/sim-results/human-proxies/{original,mirror}.jsonl.gz --chart /tmp/human-proxies.png` redraws the chart. The two AWS batches used `tools/cloud/run-batch.sh --type c7g.16xlarge --max-hours 1 --bin human_proxy_duel` with Linux ARM binary SHA-256 prefix `d5cc535875d46101`; each contained exactly salts 1–512. They used 0.02 and 0.03 server-hours, about $0.03 combined at the $0.6496/hour launch-time spot rate plus IPv4; the AWS bill is authoritative. `tools/cloud/status.sh` showed no legatum-sims servers running afterward. Both batches stayed below the $5 limit even at their combined $4.64 on-demand ceiling.

## Three most important things learned

1. The source checks changed the interpretation of proxy behavior. Mongols's second Scout now sees and steals the exposed Sparta Settler at T29 and the proxy wins its source game, but it never makes that steal in the original head-to-head. The opponent's movement and timing matter as much as the raid rule.
2. Sparta needs serial conquest and a real late siege path. It takes its sixth city at T73 and wins its source game by elimination, including two later walled captures. It still has only two level-3 veterans at T60 against the human's four at T61, and its elimination is 151 turns later than the tape.
3. The fixed-map duel still does not identify which human playbook wins. Mongols wins 0/512 from the human start yet 22/512 after the swap; Sparta wins 6/512 and 9/512. The Hard seats win 902/1,024 games, largely through earlier VP holds, while the proxies' source timing is still imperfect.
