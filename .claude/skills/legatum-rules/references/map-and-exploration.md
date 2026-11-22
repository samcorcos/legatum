# Map, exploration and vision

Part of the Legatum rules reference (index: `index.md`). Answers: How are maps and founding sites generated, how do ruins and sight work?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## 7. Map and exploration

**Balanced founding sites (designer, 2026-11-07; RULES_VERSION 24).** Every empire's founding territory, the start tile plus ring 1 (the seven tiles a new city owns and works), totals exactly **8 Food and 8 Production** (`mapgen.rs::START_FOOD`, `START_PRODUCTION`, counted as terrain yields before the city center's own bonus). After the starts are picked (and India's seat chosen), `balance_founding_sites` sculpts ring 1: water never changes, land becomes Grassland, Plains, Forest or Hills (a Mountains, Desert or Tundra tile in ring 1 is replaced too; no mountain is ever added), and the start tile itself only flips between Grassland and Plains. Among the exact solutions it picks the one that changes the fewest tiles, ties broken by a seeded draw (`LAYER_BALANCE` 8). Bonus resources stay out of ring 1: the two guaranteed Food/Production bonuses go exactly two hexes out (`resources/placement.rs`), the guaranteed Copper is two out, Iron three to five, and every other deposit is beyond three, so the ring's count never changes when a resource is revealed. A start with more than two water tiles in ring 1 is not a candidate (`START_MAX_RING_WATER`; with three the target is unreachable because water yields no Production). `validate` rejects a map where any start's ring differs from the target or holds a resource. Everything beyond ring 1 stays as generated, including strategic deposits and radius-2 production (the 2-unit floor above still applies to radius 2). The search's `site_quality` treats the founding territory as its fixed totals, so quality bands and spread are judged on what lies beyond it.
After selection, India's seat takes the picked start with the most passable land within radius 8,
with a deterministic seat tie-break. Other picked starts are swapped, not regenerated, and the final
assignment still meets the quality spread and team spacing rules
(`crates/core/src/mapgen.rs::pick_starts,assign_india_starts`). No other faction has a start
preference: a lean toward each faction's style was measured and left out
(`docs/sim-results/start-lean-2026-11-02.md`). This changes generated maps for a seed; rules version
10 rejects old saves without migration.

**Ruins**: trigger on landing for empires, never barbarians (`crates/core/src/rules/units.rs::explore_ruin`).
The 24 equally likely roll slots pay Gold (9 slots, 40), Science (9 slots, 25), +1 population
(2 slots, if a city exists), or a free Scout (4 slots, if a spawn tile exists); unavailable
population or Scout rewards fall back to Gold (`crates/core/src/rules/units.rs::explore_ruin`).
Athens receive the same reward as everyone else, plus 20 stored Science on every ruin
(`crates/core/src/rules/units.rs::explore_ruin`).
The scripted AI sends a nearby idle military unit to a known safe ruin when its `ruin_bonus`
outweighs travel, lost fortification, injury and the value of its current city post; emergency
defence and attacks take priority. Scouts still rank ruins with frontier exploration. A Settler
with no worthwhile founding site may take a nearby own-border ruin on a short, safe route; a
threatened city garrison retains its high defensive value (`crates/ai/src/military.rs::collect_ruins`,
`crates/ai/src/civilians.rs::idle_settler_ruin`, `crates/ai/src/civilians.rs::choose_frontier`).

**Vision**: unit sight is 2 by default, city sight 3; Hills give either source +1 sight. The Mongol
Scout no longer sees further (the Mongol Outrider and its sight 3 were removed with the Keshig, 2026-10-17):
every unit sees `data::UNIT_SIGHT` (2). No technology extends sight (Salvage Protocols was removed).
Each team always sees every tile owned by any of its members and every tile within one hex of that
land, regardless of terrain, line of sight or distance (`visibility.rs::native_visible_tiles`,
`add_territory_sight`). This applies to AI views and shared team sight too. Outside that territory
visibility, a tile within the radius is visible only when its hex line is unblocked. The blocking
tile itself remains visible, and every
tile adjacent to the sight source is always visible. This governs fog, including which targets a walled city may strike; a visible target within two tiles needs no separate line of sight for the city shot (§4 City strike). Mountains block everyone; Forest and Hills
block beyond themselves from low ground; a Hills viewer sees over Forest and Hills. Edge-aligned
lines are blocked only when both edge tiles block. The line uses integer cube-coordinate
interpolation and opposite fixed tie-break nudges (`crates/core/src/los.rs::blocker,rounded_step`,
`visibility.rs::native_visible_tiles`, `data.rs::UNIT_SIGHT`, `CITY_SIGHT`, `view.rs::unit_view`). On every explored
tile, `view.rs::player_view_detail` shows current territory ownership and current city existence,
owner and name: founding appears, capture changes the banner and razing removes it even under fog.
`CityView.great_works` also lists the city's completed world wonders on visible and explored city tiles,
current through fog; `CityView.detail.buildings` remains private to the owner. The map draws each
painted landmark on its chosen, stored wonder tile within the city's three-hex work radius. A built
landmark and that tile's owner are public even if the tile is unexplored (`view.rs::player_view_detail`,
`app/src/map/landmarks.ts::landmarksOf`). Other unexplored tiles disclose neither borders nor cities.
`CityView.construction` publicly names a queued wonder's chosen tile, building and progress stage,
including through fog once its city is explored; the owner's private production detail still carries
the exact cost and stored production (`view.rs::player_view_detail`, `app/src/map/constructionSites.ts`).
`TileMemory` retains last-seen private facts
(improvements and city population, HP and defenses); a newly public city with no observation
uses a neutral 1-population, full-HP estimate until sighted. Units remain sight-limited. Fogged city
banners and cards omit population, defenses, HP and production (`app/src/map/render.ts::cityBanners`,
`app/src/ui/CityPanel.tsx`, `app/src/ui/TileCard.tsx`).
`visibility.rs::contact_pairs` still uses native sight only: public changes under fog never create
contact. The AI reads this same `PlayerView`.

**Shared sight API:**
`los::visible(from: Hex, to: Hex, terrain: impl FnMut(Hex) -> Option<Terrain>) -> bool`
checks geometry only; it does not check range, team or fog. Unknown terrain
is opaque. Callers still check range and target visibility. `visibility::native_visible_tiles(&GameState,
TeamId) -> BitSet` unions the team's own sight sources with their radii and hill bonuses.
`visibility::visible_tiles(&GameState, TeamId) -> BitSet` also includes active allies' native
sight, without forwarding borrowed sight. Native sight includes owned territory and its adjacent
ring. Bits are indexed by `state.map.index(hex)`.
Target selection requires the current tile bit; explored memory alone never reveals a unit. Every
unit on a tile a team sees is seen by that team; no unit is hidden.


**Ordinary attackers** are revealed to the attacked empire and its permanent teammate through the
next planning view, even if they stand outside sight (`Unit::attack_revealed_turn`,
`visibility.rs::attack_revealed_to`). Their battle report retains the attacker position when that unit
is revealed (`event_visibility.rs::visible_events`).
