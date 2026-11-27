# Buy a city: a 10% fail fee keeps long-shot offers positive, so the AI caps risk

2026-11-26, AI for Byzantium's Buy a city (`crates/ai/src/byzantium.rs`).

- A failed offer costs only `FAIL_FEE_PERCENT` (10%) of the escrow, so a pure expected-gain test
  (success x (worth - price) - failure x fee) stays positive even at 95% failure when a city is
  worth twice its price. The buyer would then lock its whole treasury for three turns on offers
  that almost always fail. The AI never makes an offer at 60% risk or more (`LIKELY_FAIL`), on top
  of the expected-gain test.
- One more armed unit in or next to the city raises the price by 20% of the offer's base (`GUARD_PERCENT`; each Walls level adds `WALLS_PERCENT`, 25%), so a
  seller who adds any guard the buyer did not see when it made the offer breaks the offer. The
  seller's needed guard count comes from the public `price_now` alone: base = price_now x 100 /
  (100 + 25 x Walls level + 20 x guards). The goal doubling cancels out of the step.
- Every treasury is public, so the buyer can read whether the owner can afford a defender in the
  city (a near-certain break) without cheating; rival unit prices are private, so it uses its own
  cheapest armed unit's price as the stand-in.
- The AI reads three human-facing strings here: the buyer's blocker prefix "Needs " (only the
  treasury blocks the offer) and the price breakdown labels "Buildings" and "Holds a goal". A
  rewording in core `buy_city.rs` silently disables banking, building worth and goal worth; a
  structured blocker or breakdown kind in `CityBuyView` would remove the dependency.

Corrected 2026-11-26 (overbid change, `26-buy-city-overbid.md`): the AI no longer reads strings;
`CityBuyView` carries machine fields and `CityOfferView::guards_to_stop` gives the seller's count
directly, so the base-from-`price_now` derivation above is no longer used.
