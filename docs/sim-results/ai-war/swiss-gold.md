# Swiss Gold at war: the frontline freeze, the hoard reserve, the banked war genes (W2, playtest 8.9.17)

Measured 2026-10-30/31 on branch `worktree-agent-a276185e512fbb70a` (master `0e59d39bd` plus `claude/hollow-attack`). Everything ran on the cloud box; runner `swiss-gold/run.sh`, arms `swiss-gold/params/`. "Before" is commit `4dc8a7633` (tools only), "after" is `b2b8f2a9a`/`97afbbfeb` (the fix, gene `hoard_at_war`).

**Verdict.** The frontline fix is a bug fix and is neutral: every war suite case keeps its success rate, the census holds. The Swiss freeze is the 5,000 hoard savings, a rule constant used as the reserve, not a banked gene. Releasing it at war (`hoard_at_war` 2) makes the Swiss spend (median Gold at T150 530 -> 190) without costing Swiss wins in the census, but no lever here captures a city in any hollow-leader fixture. All Swiss levers stay off; the arms are ready for the lead's AWS gate.

## 1. The frontline freeze (bug)

`buy_frontline_defender` returned its danger flag whenever any city was threatened, and `plan_buy` returned on it. So a threatened city that already had this turn's purchase (a door-defense or rescue buy) or could buy no unit stopped every routine purchase in every city. Now it returns `Bought`, `Save(reserve + the city's cheapest military quote)` or `Clear`, and the rest of `plan_buy` keeps the saved amount instead of stopping. Unit tests: `crates/ai/src/frontline_tests.rs`.

In the designer's game the freeze was real, but the Swiss gained nothing from fixing it alone: the threatened city's defender was unaffordable because the frontline reserve (`gold_reserve`) is the 5,000 hoard savings, and the routine buys after it use the same reserve. `war_trace` on `war-hollow-swiss-t96` (Hard defender): `gold reserve 5000 ordinary 5000 | saving_for_goal true hoard_held false` on every turn. Gold went 768 -> 1,194 in 20 turns with one door buy.

| Measure (before -> after) | Value |
| --- | --- |
| War suite, seat (33 cases, 64 runs) | 43% / parity 13 -> 43% / 13, every case's success identical; seven cases move in losses only (Golden Horde: attacker cities lost 0.07 -> 0.03) |
| Hollow fixtures (Swiss T76, Swiss T96, Byzantium T86) | 0 / 0 / 0% -> 0 / 0 / 0% |
| `mongols-rich-and-empty` (50 runs) | 0/50 -> 0/50, median 1,240 |
| Census (256 all-AI Hard games, seed 7300001): captures / wars per game | 17.55 / 25.42 -> 17.59 / 25.11 |
| Census: Swiss wins of 256 | 21 -> 20 |
| Determinism pins (`tools/sim/repin.sh` on the box) | unchanged |

## 2. Where the Swiss 5,000 comes from

Not `BUILTIN_FACTION_DECISIONS`. `scripted.rs::ordinary_gold_reserve` returns `goals::SWISS_HOARD_GOLD` (5,000, the rule's Hoard goal) while `victory.rs::saving_for_goal` holds. That holds whenever `gold + income x min(turns left, 300) >= 5,000` with two cities, so almost always. Below the savings (the hoard not yet chosen), the Swiss reserve is the cheapest buyable Great Work quote (1,050 in the T96 fixture), else 100.

With door defense on, the reserve works as a defence fund: `door.rs::door_gold_floor` ignores unheld hoard savings, so the door spends it on a visible army. That is why the attack check went to 0% (part 4). What the savings block is everything else: the frontline defender, routine buys, surplus army, and every purchase for the Swiss's own war.

**Census, Gold held by faction (median treasury; control):** Swiss 275 / 530 / 375 at T100 / T150 / T200 and 360 at war; every other faction 54-110. Hard's 13 Gold a turn (`war-chest.md`) is the rule, the Swiss the exception.

Gene `hoard_at_war` (default 0): while the routed plan fights a war we are in, the hoard isn't held and isn't what wins, the savings are not the reserve. Level 1 keeps the Great Work quote, level 2 keeps only the deficit reserve.

## 3. The banked Swiss war genes

Climb round 1 (`0bfffddd5`, 2026-10-24, arena +3.5 pp for Switzerland) banked `siege_window` 2 (the shared default 0 plays four rounds) and `army_war_extra` 1 (shared 5). In the T96 fixture every Mongol city reads "army too weak" on every turn in every arm. With `siege_window` 0 (and `hoard_at_war` 2) some unwalled towns get a forecast at last (two units, a six-turn march, ten turns to the capture), still under the 15% odds floor.

## Arms (after; suite success identical to control in every arm without `hollow_march`)

| Arm | Suite seat / all | Census captures / wars | Swiss wins | Swiss captures / seat | Swiss cities lost / seat | Swiss Gold T150 / at war |
| --- | --- | --- | --- | --- | --- | --- |
| control (fix only) | 43 / - | 17.59 / 25.11 | 20 | 1.04 | 1.64 | 530 / 365 |
| `hoard_at_war` 1 | 43 / 43 | 18.02 / 25.70 | 13 | 1.23 | 1.64 | 219 / 256 |
| `hoard_at_war` 2 | 43 / 43 | 17.79 / 25.61 | 22 | 1.33 | 1.47 | 190 / 213 |
| Swiss `siege_window` 0 | 43 / 43 | 18.02 / 25.59 | 26 | 1.41 | 1.70 | 344 / 342 |
| Swiss `army_war_extra` 5 | 43 / 43 | 16.79 / 25.48 | 20 | 1.26 | 1.37 | 214 / 230 |
| all three (`hoard_at_war` 1) | 43 / 43 | 17.99 / 26.21 | 19 | 1.66 | 1.55 | 195 / 220 |
| all three (`hoard_at_war` 2) | 43 / 43 | 17.95 / 26.32 | 22 | 1.81 | 1.67 | 174 / 183 |
| `hollow_march` 3 | 44 / 44 | 17.02 / 24.78 | 19 | 1.14 | 1.46 | 478 / 360 |
| `hollow_march` 3 + `hoard_at_war` 2 | 44 / 44 | 17.58 / 25.46 | 16 | 1.34 | 1.49 | 196 / 208 |
| `hollow_march` 3 + all three | 44 / 44 | 17.64 / 26.02 | 17 | 1.66 | 1.84 | 166 / 184 |

- Win counts of 256 have a standard error of about 4.5. `hoard_at_war` 1's 13 and `siege_window` 0's 26 are each about 1.5 of them from the control.
- `hollow_march` 3 is what moves the suite (Chur T137 34 -> 59, Mongol bombers 17 -> 26, Golden Horde thin 0 -> 6, as in W2.4). The Swiss levers add nothing to it.
- With `hoard_at_war` 2, the Swiss in the T96 fixture buy Troopers and Soldiers (768 -> about 200 Gold) and lose 0.17 units a run for 0.67 kills, against 0.01 and 0.89. They have no capture, because five of eight units still stand in their cities.
- The hollow fixtures read 0% in every arm, and `mongols-rich-and-empty` reads 0/50.

## 4. The attack regression check

`war-golden-horde` / `-thin` / `war-walls-rise` with door defense on read 0 / 0 / 0% before and after the fix. With the defenders' door off (`--opponent-params tools/sim/attack-check-defenders.json`) they read 98 / 89 / 81%, the same before and after.

`tools/sim/war-scenarios.sh --check` now plays the defenders with the door off and its floor is 60%. Floors measured on today's defence would be 0%, so the check could catch nothing. The check is meant to test the attacker's execution against a fixed defence; the door's own effect is in the suite's defence cases. A candidate passed with `--ai-params` still plays only the seat.

## Decision cost

Seat planning ms per turn (median per run, `war_trace` runs 0-2, before then after, back to back on the box; a verify job shared the box for part of it). Everything is within run-to-run noise. The fix adds one short list of quotes per threatened city. The gene adds a plan lookup.

| Case | Before (control) | After (control) | `hoard_at_war` 2 | All three, level 2 |
| --- | --- | --- | --- | --- |
| Byzantium T86 | 5.41 / 5.30 / 5.55 | 5.44 / 8.77 / 5.36 | 5.38 / 5.61 / 5.59 | 5.41 / 5.81 / 5.58 |
| Swiss T76 | 7.06 / 5.67 / 6.01 | 5.31 / 5.88 / 6.47 | 5.29 / 6.04 / 6.76 | 2.31 / 6.34 / 7.01 |
| Swiss T96 | 5.61 / 8.44 / 6.79 | 6.00 / 6.31 / 10.28 | 6.10 / 6.22 / 9.63 | 6.18 / 6.89 / 10.06 |
| Golden Horde thin | 6.37 / 10.61 / 9.68 | 7.93 / 10.95 / 6.86 | 6.90 / 8.05 / 6.77 | 9.76 / 10.86 / 7.00 |
