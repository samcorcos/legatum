# Buy a city: the overbid makes the counter cost something, but the AI treasury caps purchases

2026-11-26, Byzantium's Buy a city with overbids (`Order::BuyCity { city, gold }`, core
`buy_city.rs`; AI `crates/ai/src/byzantium.rs`), rules 63 (unreleased).

- Before the overbid, an offer was held at exactly the price, so one armed unit moved next to the
  city (+25% of the base) always broke it: 15 of 18 AI offers in 128 eight-empire Hard games failed
  that way. With a bid of up to 300% of the price, breaking it takes `guards_to_stop` more units
  (price 390: 1 at the price, 2 at +25%, 3 at +50%, 5 at +100%). The view carries the count to both
  sides (`CityBuyView::bids`, `CityOfferView::guards_to_stop`, `walls_to_stop`), so nothing is
  hidden math.
- `defended_price(base, walls, guards, goal)` is the price's whole defence step in one integer
  formula; it equals the breakdown's total because the breakdown adds `base x (walls% + guards%) /
  100` as two lines that sum to the same rounding. The guard and Walls counts reuse it.
- The AI reads machine fields now (`base_price`, `buildings_gold`, `holds_goal`, `guards`,
  `short_of_gold`, `max_bid`), not the breakdown labels or the "Needs " prefix (the dependency
  noted in `26-buy-city-ai-risk.md` is gone).
- Measured (256 eight-empire Hard games each, seed 9311003, Byzantium rotated through the seats
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
  Byzantine economy that earns more Gold, not a braver AI.
- Genes: `buy_city_value` 200 (was 100), new `buy_city_overbid` 50 (half a unit of unseen threat
  each bid also covers), new `buy_city_bank_turns` 10 (the old constant).
