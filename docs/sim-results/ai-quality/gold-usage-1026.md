# Gold usage in three human wins (26 October 2026)

## Plain answer

**The AI mostly spends as soon as an affordable option scores well; in the Sparta game it also earns much less.** The gap is not a universal lack of Gold income. In the Japan win, AI Sparta earned about **7,007 Gold** and AI Germany **6,870**, against the human's **6,735**. Both spent essentially their entire flow (7,001 and 6,713), leaving average treasuries of **73** and **84**; the human spent 5,501 and held **535** on average. In the Athens win, the human earned 7,270, spent 6,063, and averaged 860; the closest AI earners were Switzerland at 6,293 and Germany at 5,913, which spent 6,190 and 5,845 and averaged 143 and 64. In the Sparta win, conquest and kill rewards helped the human earn **14,165**, versus AI Athens's 4,208 and Japan's 3,037. He still saved amid heavy army spending: 12,539 spent, 1,353 average treasury.

Thus the sawtooth has two causes: **an affordable-only purchase style** in all three games, plus a **large income and expansion gap** in Sparta. Upkeep is material, especially for AI Switzerland (4,413 Gold), but it does not explain the Japan result: those two AI peers earned at least as much as the human. There is no Gold purchase of territory and no tribute system in these versions, so tile buys and tribute are both **zero**. The AI does buy tile *improvements*.

![Per-empire treasury curves and Gold by source and use](gold-usage-1026.png)

The full [per-empire, per-era ledger](gold-usage-1026-eras.csv) gives every category, treasury average, turn count and balance change. Era is the empire's era *before* each recorded turn resolves; eliminations stop its rows. Ancient through Future are separate rows, including eras with only one turn.

## Whole-game ledger

Income columns are **city terrain and other city effects / buildings / trade / interest / sacks / conquest plunder / other rewards**. The `trade` column includes Company sales and Mongol long-road commerce; `other` includes ruins, wonders, camps and Sparta's Iron Price kill reward. Direct spending columns are **unit buys / building buys / improvement buys / upgrades / Siege / treaties / upkeep / Companies**. All values are Gold. Improvement buys are distinct from tile buys (zero). Treaty signing and breach fees are in `treaties`, as is treaty upkeep. `upkeep` contains military, administration and overreach charges. Zero columns are retained in the CSV.

### Japan win (234 turns)

| Empire | Income by source, in order above | Spending by category, in order above | Avg / peak treasury | Final VP / cities taken |
|---|---|---|---:|---:|
| Japan (human) | 2,220 / 4,225 / 160 / 0 / 0 / 0 / 130 | 3,288 / 400 / 0 / 1,288 / 493 / 0 / 32 / 0 | 535 / 1,353 | 14 / 14 |
| Sparta | 3,328 / 2,213 / 0 / 0 / 0 / 100 / 1,366 | 320 / 480 / 1,440 / 1,556 / 1,442 / 468 / 1,295 / 0 | 73 / 259 | 6 / 1 |
| Germany | 3,835 / 2,753 / 242 / 0 / 0 / 0 / 40 | 2,608 / 882 / 760 / 946 / 317 / 0 / 1,200 / 0 | 84 / 367 | 2 / 1 |
| Byzantium | 1,690 / 1,086 / 0 / 0 / 0 / 0 / 0 | 0 / 1,040 / 280 / 504 / 172 / 5 / 312 / 482 | 124 / 641 | 1 / 2 |
| Athens | 3,542 / 2,001 / 80 / 0 / 0 / 0 / 80 | 1,704 / 780 / 660 / 738 / 0 / 216 / 1,564 / 0 | 102 / 451 | 2 / 1 |

### Athens win (223 turns)

| Empire | Income by source | Spending by category | Avg / peak treasury | Final VP / cities taken |
|---|---|---|---:|---:|
| Athens (human) | 4,954 / 2,096 / 0 / 0 / 0 / 0 / 220 | 1,456 / 3,000 / 160 / 1,122 / 0 / 0 / 325 / 0 | 860 / 3,152 | 10 / 1 |
| Byzantium | 1,651 / 1,470 / 0 / 0 / 0 / 0 / 160 | 160 / 100 / 200 / 244 / 0 / 525 / 917 / 1,120 | 51 / 213 | 5 / 1 |
| Germany | 3,118 / 2,575 / 0 / 0 / 0 / 0 / 220 | 0 / 828 / 880 / 2,702 / 333 / 0 / 1,102 / 0 | 64 / 274 | 8 / 2 |
| Mongols | 1,143 / 844 / 960 / 0 / 0 / 0 / 0 | 640 / 280 / 120 / 904 / 0 / 266 / 705 / 0 | 63 / 352 | 3 / 3 |
| Japan | 423 / 875 / 160 / 0 / 0 / 0 / 40 | 0 / 200 / 60 / 0 / 0 / 219 / 953 / 0 | 57 / 116 | 2 / 0 |
| Switzerland | 4,421 / 1,652 / 0 / 220 / 0 / 0 / 0 | 140 / 332 / 262 / 974 / 0 / 69 / 4,413 / 0 | 143 / 433 | 3 / 0 |
| Celts | 1,871 / 889 / 0 / 0 / 216 / 0 / 0 | 640 / 120 / 360 / 924 / 0 / 108 / 748 / 0 | 53 / 285 | 5 / 0 |
| Sparta | 1,794 / 251 / 0 / 0 / 0 / 397 / 1,842 | 160 / 0 / 90 / 1,490 / 489 / 75 / 1,923 / 0 | 43 / 241 | 3 / 5 |

### Sparta win (207 turns)

| Empire | Income by source | Spending by category | Avg / peak treasury | Final VP / cities taken |
|---|---|---|---:|---:|
| Sparta (human) | 5,247 / 4,393 / 0 / 0 / 0 / 1,667 / 2,858 | 7,972 / 420 / 0 / 1,538 / 1,109 / 869 / 631 / 0 | 1,353 / 4,771 | 10 / 14 |
| Celts (eliminated T22) | 41 / 0 / 0 / 0 / 0 / 0 / 120 | 0 / 0 / 140 / 0 / 0 / 0 / 0 / 0 | 63 / 131 | 0 / 0 |
| Japan (eliminated T192) | 1,948 / 1,089 / 0 / 0 / 0 / 0 / 0 | 0 / 0 / 120 / 1,312 / 520 / 0 / 1,011 / 0 | 42 / 114 | 3 / 1 |
| Athens | 3,021 / 1,067 / 0 / 0 / 0 / 0 / 120 | 480 / 600 / 380 / 1,468 / 0 / 406 / 617 / 0 | 80 / 313 | 3 / 0 |

## What the purchases delivered

**The human bought fewer, larger items at concrete moments.** Across the three games, he bought 17 units for 12,716 Gold (median quote 692), seven buildings for 3,820 (median 480), and two improvements for 160. The 14 AI seats bought 41 units for 6,852 (median 160), 33 buildings for 5,642 (median 120), and **76 improvements for 5,752** (median 80). They also made **246 unit upgrades** for 13,762, 37 Siege upgrades for 3,273, and 19 Company hires for 1,602. Upkeep cost 16,760. The AI is not making an item purchase literally every turn; recurring upkeep and those upgrades contribute to the sawtooth. Counts and prices are from successful resolution events, rather than submitted orders.

Every bought human unit (17/17) and AI unit (41/41) was purchased during a war or within three turns of a war turn. The difference is therefore not simply whether purchases happen near wars: the human concentrated much larger quotes in modern units, while AI unit buys were predominantly 160-Gold Soldiers. This proximity measure is descriptive; long wars make it generous.

- **Japan:** a 400-Gold Bank bought on T146 was followed by the “Add a Bank to 3 more cities” era VP on T147. He bought an Assault Trooper on T121 before the T124 capture, Bombers on T172 and T185, and a Tank on T204; he took 14 cities by T230. The Bank/VP sequence is a close observed link; captures cannot be assigned to one bought unit from the event ledger. AI Sparta spent 1,440 on 15 improvements, 1,556 on upgrades and 1,442 on Siege for one capture. AI Germany bought 13 Soldiers among 15 unit purchases for 2,608 and took one city.
- **Athens:** T180–T192 building buys totalled 3,000, including an 880-Gold Factory on T192 when the Factory era objective scored, and a 120-Gold Workshop on T181 before the Workshop objective scored on T182. He also bought a Tank for 1,028 on T207 and took the VP leader's city on T214. AI Germany spent 2,702 upgrading units and 880 on improvements, with two captures; AI Switzerland spent 4,413 on upkeep and 974 on upgrades, scored no Great Work, and took no city.
- **Sparta:** four Field Cannons cost 2,768 on T165; ten captures followed on T169–T192. Four Tanks cost 4,112 on T198, after those captures, so they cannot explain them. The human took 14 cities; AI Sparta in the Athens game took five cities after only a 160-Gold Soldier buy. This counterexample shows why dividing captures by purchased-unit Gold would misstate value: Production, starting units, tactics, opponents and timing also matter.

The records support *timing and outcomes*, not a causal VP-per-Gold or cities-per-Gold estimate. Direct VP links are the Bank and Factory purchases above; other VP often came from naturally produced buildings, captures, territory and combat. A proper value-per-Gold comparison needs matched counterfactual replays, especially for the proposed savings gene.

## Why the AI behaves this way

`crates/ai/src/production.rs::plan_buy` evaluates ordinary options **only after** checking that their quote leaves the current reserve; it then chooses the best `item_value / buy_cost` among those affordable options. If none qualifies, `buy_surplus_army` spends excess on an affordable unit. Emergency defenders and goal items use earlier passes. `scripted.rs::ordinary_gold_reserve` is zero for a solvent non-Swiss empire outside an opening; its default reserve covers a deficit, not an intended purchase. `funding.rs` protects a four-turn negative-income runway, and `plan.rs::veto_plan_reserve` holds an amount for the selected plan, but neither names an ordinary building quote. Existing exceptions save for a Swiss Great Work/hoard and Siege. Thus the AI *can* save in special cases, but lacks a general way to compare an unavailable high-value purchase with this turn's small affordable one.

The default-off gene `decision.purchase_savings_turns` (0–12, default **0**) now lets that comparison include a legal but Gold-blocked **building** quote if positive planned net income can reach it within the horizon and natural Production will not finish it first. With the gene on, near-term purchases are ranked by total `item_value`, with their Gold quote determining affordability and arrival time; the default retains value per Gold. If a cheaper buy would delay the more valuable target, routine spending waits; emergency defense and existing goal passes retain their priority. A fixture makes the baseline buy a cheap Granary, the gene save for Athens's third Laboratory, and the next funded turn buy the Laboratory. It changes AI decisions only, not rules or player costs.

**Next search step for the lead:** measure the gene on the saved positions and human-seat benchmark, then extend the same named-target ledger to a campaign's *decisive force package* (unit kind, quantity, travel/arrival turn, upkeep and capture-value delta). Compare that package's expected win/VP value per Gold against an immediate improvement or upgrade; reserve only until it can be bought on schedule and release the reserve when the war forecast or target changes. The current implementation deliberately covers buildings first; it is not yet a war-chest planner, and its value score is the AI's current heuristic rather than counterfactual purchase value. Leave enabling and gating to the lead.

## Replay and accounting

The copies of `japan-win-bombers-20261026.legatum`, `athens-win-swiss-tepid-20261025.legatum`, and `sparta-win-final-20261025-0902.legatum` were replayed with their metadata engine commits `d1233d48678a`, `8e9433e5cf53`, and `8e9433e5cf53`. The read-only `crates/sim/examples/gold_audit.rs` verifies the initial hash and every turn hash: **234 + 223 + 207 = 664 turns; zero mismatches**. `tools/ai-quality/gold_usage_1026.py` computes the ledger and chart from those JSONL outputs. The originals were only read; temporary input copies and full replay outputs are outside the repository.

Income *source* columns are the engine's start-of-turn city-yield and Gold forecasts. A purchase, capture, or production completion before the income phase can change that turn's actual payout. Successful purchases and direct payments are exact resolution events; treasury curves are exact pre-turn balances. Source totals therefore differ slightly from the fully reconciled treasury changes (maximum 152 Gold for a surviving seat, 2.1% of the Athens human's attributed gross income). `actual_regular_net` and `treasury_change` in the CSV make that gap auditable. No claim about an AI's unseen intent is inferred from the full-state replay.

Validation on this branch: the savings fixture passed (1/1); `cargo fmt --all` and workspace Clippy with `-D warnings` passed; `cargo test --workspace --profile release-checked` passed **1,961/1,961 tests** (10 skipped) and all workspace doctests. Cargo commands used `tools/sim/build-slot.sh --`.
