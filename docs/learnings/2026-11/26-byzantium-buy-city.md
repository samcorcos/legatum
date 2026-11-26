# Byzantium's Buy a city and the fall of Constantinople: non-obvious points (2026-11-26)

Rules 63 (`crates/core/src/buy_city.rs`).

- **A literal "fail if the price rose" breaks offers on growth.** The price counts population and buildings, so a city that grows or finishes a building during the three-turn countdown would fail the offer (and cost Byzantium the 10% fee) without the owner doing anything. The offer stores its first price step (`CityOffer::base`: base, population, buildings) and the failure check (`buy_city::offer_price`) recomputes only Walls, armed units and goal VP: the counters the seller actually controls.
- **The price check skips the order's own turn.** The escrow is priced before movement; a seller unit that happens to step next to the city that same turn would otherwise fail an offer the seller never saw. Cancellation (War, change of owner) is checked from the first turn end.
- **`capture_city` on a city its captor already owns re-applies the capture** (half population again, resistance reset). The collapse takes every Byzantine city at once, so a second planned capture by the same captor in the same movement step would hit an already-taken city; `rules/movement.rs` now skips a planned capture whose city is no longer hostile.
- **A captured capital leaves its empire with no capital at all** (`is_capital` is cleared and nothing moves it), so the Heir refuge sets `is_capital` on the refuge city itself: there is no general capital-move rule to reuse.
- **Validation sees the projected Gold.** `rules::validate_orders` already builds the state after the turn's city orders for aircraft upgrades; `buy_city::validate_orders` reads that state, so a purchase earlier in the list that leaves too little Gold refuses the offer at validation, as resolution would.
