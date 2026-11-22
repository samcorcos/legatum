# Each era is defined by its resource (2026-10-22)

The user, after two games without once needing a resource: "a big part of civ 6 is fighting over
strategic and luxury resources. this game doesn't have that." Then: "it's almost like each age
should be defined by its resource. bronze (i.e. copper), iron, niter, oil... and it can be used for
weapons or buildings. i think you understand my intent so please implement that." Luxuries are
"more like flavor but not a core part of the game... kind of like the world wonders". No
resource-specific objectives. A missing-resource alert: yes (lead).

**2026-10-25 update:** Copper is a bonus tile resource and supplies nothing. Ancient units, the Market and the Workshop need no strategic resource. Iron Working reveals Iron. See the current `legatum-rules` resource table; the earlier Copper supply proposal below is historical.

This replaces the 2026-10-19 building mapping (Bank Silver, University Copper, Research Lab
Copper + Coal). The supply rule itself is unchanged (game-rules §3): a known deposit inside your
borders supplies 2, 3 with a Mine or a city on it; starting, buying or upgrading into a user needs
1 spare; short of supply the newest users go idle (an idle building yields nothing, an idle unit
fights 25% weaker).

## The rule, in one sentence for players

**An era's new units and its key building use 1 of that era's resource.** Everything on offer on
turn 1 is free, and so are Walls, food buildings, Roadworks, Great Works and faction landmarks.

## Era to resource

| Era | Resource | Why this one (one line) | Revealed by | Users |
|---|---|---|---|---|
| Ancient | **Copper** | Bronze is copper; the first coins were bronze and copper (Roman *aes*, Chinese cash). | visible from turn 1 | Market |
| Classical | **Iron** | The Iron Age: iron swords, armour and tools. | Bronze Working (as today) | Phalangite, Composite Archer, Cataphract (Keshig), Forge, Siege |
| Medieval | **Iron** (continues) | Mail, plate, pike heads and crossbow prods were iron and steel; the user's own list goes bronze, iron, niter. | (Bronze Working) | Pikeman (Legionary), Crossbowman, Knight, University |
| Renaissance | **Niter** | Saltpetre makes gunpowder; states fought over and monopolised it (Bihar, Chile later). | Education (Roger Bacon wrote down the recipe at Oxford, 1267) | Musketman, Arquebusier, Cuirassier, Bank |
| Industrial | **Coal** | Coal drove steam, steel and railways. | Metallurgy (coke smelting) | Rifleman, Field Cannon, Cavalry, Factory |
| Modern | **Oil** | Oil fuelled tanks, aircraft and two world wars. | Industrialization (the first oil wells, 1859) | Infantry, Artillery, Tank, Research Lab (aircraft: after an AI pass) |
| Future | **Lithium** | Batteries for drones, robots and autonomous machines; already fought over today. | Electricity | Humanoid Robot, Combat Drone, Autonomous Armour |

Lead's picks the user's intent didn't settle, with the reason:

- **Medieval keeps Iron, not Horses.** A Pikeman or Crossbowman needing Horses reads wrong to a
  history-minded player, and the user's own list skips from iron to niter. Iron's demand grows in
  the Medieval era (every Classical unit upgrades into a Medieval Iron user, Universities join
  Forges), so Iron becomes the long war of the early game. No Horses resource, so the Mongols get
  no resource rule: their Keshig is a Classical Iron user like the Cataphract it replaces.
- **Future is Lithium**, not Uranium (no nuclear weapons or reactors exist in the game; Fusion
  Power is the Future tech) or Aluminium (airframes are Modern, and they run on Oil here).
- **Ancient Copper gates the Market only.** Turn-1 units (Spearman, Archer, Horseman), the Granary
  and the Brood Hall stay free, so rushes and the first Settler are untouched. The Workshop and
  Library stay free so Production and Science in the opening are untouched. The Market is the
  Ancient building that uses Copper: coinage is the strongest historical tie, Copper's tile bonus
  was already Gold, and a missing Market costs Gold, not growth or tempo.
- **Only one key building per era**, the new tier of the Production, Gold and Science lines:
  Market, Forge, University, Bank, Factory, Research Lab. Every building of every era would put
  three or four users in each city per era, which no realistic deposit count can supply without
  covering the map. A city keeps the needs of its lower tiers (a Factory city uses the Forge's Iron
  and its own Coal; a Bank city the Market's Copper and its own Niter), as Forge and Factory do
  today: old resources stay worth holding all game.
- **Walls (all tiers), food buildings (Granary, Mill), Roadworks, Great Works and faction
  landmarks stay free.** Walls are the counter to rushes (pillar 10) and the user decided food and
  walls stay free on 2026-10-19. Great Works are wonders, which need no resources in Civ 6.
- **An upgraded unit switches to its new era's resource** (as recommended). A Pikeman upgrading to
  a Musketman frees 1 Iron and needs 1 spare Niter. Every era is a new scramble, and upgrading is
  how old supply is released. A unit that changes resource counts as a new user of the new one
  (it is the newest, so it is first to go idle).
- **Siege uses Iron in every era** (the Siege upgrade's own rule, kept consistent): it is the
  siegework of the Classical era that bought it.
- **Reveal one era early.** Each resource appears with a technology of the era before it (Copper
  is always visible), so the scramble starts before you need it, as Bronze Working reveals Iron
  today and Civ 6 shows Iron before Swordsmen. Coal moves from Iron Working to Metallurgy.

## Map

- **Copper**: one per start, 2 hexes from it (inside the capital's first claims, nearer to it than
  to any other start; border growth already leans toward known deposits), plus more by land share
  away from the starts. Copper is no longer a bonus resource; the start's two Food or Production
  bonus resources are unchanged (Stone may now sit on Hills so a hilly start keeps its two).
- **Iron**: one per start 3 to 5 hexes out (as today), plus more by land share.
- **Niter, Coal, Oil, Lithium**: no guarantee; all in the borderlands, at least 4 hexes from every
  start and no more than 1.5x farther from the nearest rival start than from the nearest start
  (today's Coal rule), at most a few per nearest start, spaced apart.
- Counts are tuned with `crates/sim/examples/supply_census.rs` so that a typical empire's own
  supply covers well under its era's army plus key buildings: it must expand or fight for more.

## What the player sees (pillar 9)

- **Top bar:** the current era's resource always has a chip ("Niter 4 / 6"), plus any resource
  that is short. When the era's resource is missing (none supplied while you have, or are about
  to have, users for it), the chip carries the alert and its tooltip says: "Missing Niter: your
  gunpowder units and Banks need it", with who holds Niter deposits you have seen.
- **Resources overlay:** deposits of a resource you lack are marked, and the tile card names the
  holder.
- The production card already shows "Uses 1 Niter" and the blocker text; the Guide gets the era
  table.

## Luxuries

Flavour only, like natural wonders: a handful of named tiles per map (Silk, Spices, Wine, Ivory,
Incense, Furs) with a small local tile bonus and a story line in the tooltip, no empire-wide rule.
Built last and small; if their art isn't ready they are a follow-up.

## Measures

Supply census per era (deposits per empire, users, idle users, deposits changing hands in wars,
captures of deposit cities); opening pacing (first Settler about T18, second city about T25);
AI-impact gate (human-seat benchmark, seat-only and opponents-only, on spot); war suite `--check`;
stalls.
