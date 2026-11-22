# Systems that do not exist

Part of the Legatum rules reference (index: `index.md`). Answers: Which Civ 6 systems are absent from Legatum? (read before proposing anything)
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## 9. Systems that do NOT exist (verified absent by code search, re-verified 2026-11-02)

| Civ-6-style system | Status |
|---|---|
| Trade routes / caravans | **Absent** — no match in `crates/` |
| Districts | **Absent** |
| Builders / worker units | **Absent** — improvements are built by cities as production items (§3, Tile improvements) |
| Religion | **Absent** |
| Culture / tourism | **Absent** |
| Great People | **Absent** (world wonders exist, unrelated) |
| Espionage / spies | **Absent** |
| Naval units | **Absent** — Guide: "There is no navy yet" |
| Aircraft on the map / airbases as tiles | **Absent** — aircraft live in a city's hangar (Eras and technologies, "The air layer"); no airfields, carriers, air movement or air superiority over open ground |
| Mounted/ranged/melee class bonus beyond the triangle | **Absent** — only +40% of base strength (`combat::CLASS_BONUS_PERCENT`) melee line vs mounted and mounted vs ranged (in melee only: none when the mounted unit is shot at, designer 2026-11-10), none from the Modern era (`combat::triangle_applies`), and the mounted −33% charging a city; no ranged-vs-melee bonus (the cavalry experiment's +27 backfired). A Keshig's shot fights as ranged (`combat::fight_class`), so it gets no ranged-vs-melee bonus either |
| Loyalty | **Absent** |
| Pillaging | **Absent** — code: "this game has no pillage" |
| Road-laying units / tile-by-tile road building | **Absent** — roads come only from the Yam Relay Station (the Courier Hall, §8) and the Roadworks building (`roads::extend_roadworks`) |
| Military Engineer, Forts | **Removed 2026-10-22** — Siege is a Gold upgrade on an ordinary unit (§4); units laying roads or building Forts are roadmap only |
| City-states | **Absent** |
| Tech trading | **Absent** — only passive diffusion (−10% per other living empire that knows it, max −50%), which a closed Japan (Sakoku, §8) does not get |
| Tech boosts (Eurekas) | **Removed 2026-10-17** — a technology's price depends only on the table and diffusion |
| Endless / repeatable technologies | **Absent** — after the 40th technology Science converts to Gold, Food or Production at 50% (Eras and technologies) |
| Relocating/moving a city | **Absent** |
| Influence (or any culture-like currency) | **Removed 2026-10-03** — borders grow per city (§3); treaties cost Gold (§5) |
| Pacts, notice periods, hidden trust, call commitments | **Removed 2026-10-03** — Peace, Alliance or War; betrayal is one visible record (§5) |
| Alliance signing fee, free shared wars | **Removed** — Alliances cost Gold per turn only (2026-10-31); allies defend at once and answer an ally's offensive war by choice (calls to arms, 2026-10-23, §5) |
| Bought peace / tribute | **Removed 2026-10-22** — Byzantium hires Mercenary Companies instead (§5) |
| Game speeds (Quick) and the opening price ramp | **Removed** (2026-10-10 and 2026-10-22) — one pace, listed prices (§3 Default pacing) |
| Celtic homeland damage and hidden hunters | **Removed 2026-10-23** — every unit on a seen tile is seen (§7) |
| Multi-step/attack-then-move orders | **Absent** — one order/unit/turn; a move onto an enemy fuses into one attack and ends the turn. The Keshig picks a shot or a charge each turn; no shoot-and-move, no hit-and-run |
| Promotion choice/trees | **Absent** — flat automatic bonus only |
| Upgrade choices | **One only** (2026-11-14): a Biplane upgrades to a Fighter or a Bomber, the player's choice; every other unit has one upgrade line |
| Unit-specific Fortify bonuses | **Absent** — every military unit fortifies alike: +`FORTIFY_PER_TURN` 3 defence per full turn a military unit does not act, up to `FORTIFY_MAX` 6 (reset to 0 when it moves, attacks or stands where passage is closed to it, `GameState::unit_in_open_land`), a flat breakdown line "Fortified" not scaled by era (`rules/units.rs::heal_and_fortify`, `combat::unit_defense_strength_for_rules`). Trench Infantry has no bonus of its own (designer, 2026-11-14, final) |
| Sight from an aircraft away from its base | **Absent** — every aircraft sees 4 tiles around its city (`add_air_sight`) and a Recon shows 2 tiles around a tile for one turn (`add_recon_sight`), but there is no sight along a flight, no patrol and no automatic reconnaissance |
| Manual tile-worker assignment | **Absent** — a city works every tile it owns within 3 hexes (§3) |
| Luxury resources / amenities | **Absent** — luxuries are planned as flavour tiles only, like natural wonders (user 2026-10-22); no empire-wide rule |
| Gifts of anything but Gold, or Gold to a non-teammate | **Absent** — only `Order::GiveGold` to a living teammate (§5 "Co-op Gold gifts", `rules/gifts.rs::validate_gifts`) |
| Resource trade or stockpile | **Absent** — supply only (§3); resource-specific objectives were rejected (user 2026-10-22) |
| A conquest requirement for winning by VP | **Absent**: no rule demands a captured city for the ten-turn VP hold (§6). A separate win by capitals exists: the only team still holding an original capital of its own wins (rules 37, §6) |
| A turn-limit winner backstop | **Absent by default** — no turn limit; the 1,200-turn simulation safety cap names no winner (§6) |
| Decadence | **Off by default** — only the opt-in `decadence-slope` experiment turns it on (§6) |
| A playable Japan | **Absent** — Sakoku is reserved and `Japan` is only a save alias of India (§8) |
