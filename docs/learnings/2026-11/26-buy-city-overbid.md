# Buy a city: the overbid makes the counter cost something; the price had to fall to a third

2026-11-26, Byzantium's Buy a city with overbids (`Order::BuyCity { city, gold }`, core
`buy_city.rs`; AI `crates/ai/src/byzantium.rs`), rules 63 (unreleased).

- Before the overbid, an offer was held at exactly the price, so one armed unit moved next to the
  city (+25% of the base, the first constants) always broke it: 15 of 18 AI offers in 128 eight-empire Hard games failed
  that way. With a bid of up to 300% of the price, breaking it takes `guards_to_stop` more units
  (price 175 under the retuned constants: 1 at the price, 2 at +25%, 3 at +50%, 6 at +100%). The view carries the count to both
  sides (`CityBuyView::bids`, `CityOfferView::guards_to_stop`, `walls_to_stop`), so nothing is
  hidden math.
- `defended_price(base, walls, guards, goal)` is the price's whole defence step in one integer
  formula; it equals the breakdown's total because the breakdown adds `base x (walls% + guards%) /
  100` as two lines that sum to the same rounding. The guard and Walls counts reuse it.
- The AI reads machine fields now (`base_price`, `buildings_gold`, `holds_goal`, `guards`,
  `short_of_gold`, `max_bid`), not the breakdown labels or the "Needs " prefix (the dependency
  noted in `26-buy-city-ai-risk.md` is gone).
- First measured with the first constants (256 eight-empire Hard games each, seed 9311003, Byzantium rotated through the seats
  with Switzerland, India, Germany, Celts, Mongols, Sparta and Athens; release build, 4 threads):

  | Run | Offers/game | Purchases/game | Failed | Cancelled | First purchase (median turn) | Collapses/game | Byzantium wins | Others' mean |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | Overbid AI, `buy_city_value` 100 | 0.07 | 0.02 | 0.03 | 0.02 | 114 | 0.54 | 11.3% | 12.7% |
  | Overbid AI, `buy_city_value` 200 (shipped) | 0.20 | 0.13 | 0.05 | 0.02 | 162 | 0.53 | 12.1% | 12.6% |
  | Same, `--rule-off byzantium_buy_city` | 0 | 0 | 0 | 0 | - | 0.58 | 9.4% | 12.9% |
  | Same, `--rule-off byzantium_collapse` | 0.22 | 0.10 | 0.08 | 0.04 | 172 | 0 | 11.7% | 12.6% |

  Buying off lowers Byzantium's share (12.1% to 9.4%, about 2.7 points, at the edge of 256-game
  noise). Collapse off did not raise it (11.7%): the drawback is not measurable at this size.
  Overbids cut the failure rate from 15 of 18 offers to 12 of 50.
- The target of 0.5 to 2 purchases a game is not reachable from the AI side. Over 4,277 buyable
  city-turns in 16 diagnostic games the median price was 1,522 Gold (base 860, buildings' full Gold
  price 500, one guard and one Walls level), the median threat was 3 more units, and Byzantium's
  median treasury was 99 Gold (income 5 to 35 a turn). Raising `buy_city_value` past 200 did not add
  purchases (300: 0.12 a game); banking for 25 or 40 turns instead of 10 did not either (0.08 to
  0.13) and cost Byzantium 3 to 6 points of win share, because the held Gold no longer buys units.
  More purchases need a rule lever (a lower price, or cheaper buildings and Walls in it) or a
  Byzantine economy that earns more Gold, not a braver AI. The price lever was taken (below).
- Genes: `buy_city_value` 200 (was 100), new `buy_city_overbid` 50 (half a unit of unseen threat
  each bid also covers), new `buy_city_bank_turns` 10 (the old constant).
- **Price retune (same day; the spec's constants were placeholders).** `BASE` 150 to 75, `PER_POP`
  60 to 25, `BUILDING_PERCENT` 50 to 20, `WALLS_PERCENT` 50 to 25, `GUARD_PERCENT` 25 to 20; the 3x
  bid cap and the goal doubling are unchanged. One guard or Walls level still lifts the price by a
  fifth or a quarter, but no longer half. Over 5,907 buyable city-turns in 32 diagnostic games
  (median turn 114) the median price is 500 Gold (p25 326, p75 735; was 1,522), the median first
  step 346, one guard, a threat of 2 more units, and Byzantium's median treasury 91 Gold. A Gold
  Settler then costs 240 to 440 (the second to fourth Settler), so a typical city is about 1.2 to 2
  Settlers: the 500 to 700 Gold median target was kept over the 2 to 4 Settlers rule of
  thumb. A first step at `BASE` 100 and buildings 25% (median price 543) gave 0.41 purchases a game
  (Byzantium 12.5%); the final constants:

  | Run | Offers/game | Purchases/game | Failed | Cancelled | First purchase (median turn) | Collapses/game | Byzantium wins | Others' mean | Best faction |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | Shipped | 0.91 | 0.56 | 0.25 | 0.09 | 114 | 0.52 | 16.0% | 12.0% | Athens 18.0% |
  | `--rule-off byzantium_buy_city` | 0 | 0 | 0 | 0 | - | 0.58 | 9.4% | 12.9% | Athens 17.2% |
  | `--rule-off byzantium_collapse` | 1.05 | 0.66 | 0.28 | 0.09 | 114.5 | 0 | 11.3% | 12.7% | Celts 15.2% |

  Buying is now worth about 6.6 points of Byzantium's share (9.4% to 16.0%), below the best faction
  and under 20%. 87 of 256 Byzantine seats bought at least one city; about a quarter of offers
  fail. The collapse ablation came out lower than shipped (11.3%), which at 256 games is noise: the
  drawback is still not measurable at this size and needs a larger run before it is judged.
- The seller AI weighed a bought Walls level as two guards (true at 50% / 25%). It now uses
  `WALLS_GUARDS` (`WALLS_PERCENT / GUARD_PERCENT`, at least one), which is one at 25% / 20%;
  `offer_threat` counts Walls under way the same way.
