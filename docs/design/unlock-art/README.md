# Unlock art: directions

The user, 2026-10-11, on the research screen's era cards: "In Civ, I remember looking at the techs like this and seeing what they enable and thinking 'ooh, I want that'. This one doesn't have that same feeling because the things you're getting feel stale." Today each unlock is a 16 px engraved line icon in a 28 px frame with its stat ("Workshop +2 Production").

**Comparison sheet (phone portrait): [`directions.png`](directions.png).** Pick one direction; nothing is wired into the app yet.

## The three directions

The same five Ancient unlocks are painted in each direction (Workshop, Walls, Market, Library, and Irrigation's "+1 Food in every city"). Files, prompts and derivatives are in [`samples/`](samples/); `samples/contact.png` puts all fifteen side by side.

| | Direction | What it is | Good | Bad |
| --- | --- | --- | --- | --- |
| **A** | Painted miniature (**recommended**) | The building as a hand-painted tabletop miniature on a small patch of ground, with the city art's camera and finish. No frame: it stands on the card. | One family with the city and unit miniatures already on the map; legible at 64 px; frameless, so two or three unlocks share a card cleanly. | Smallest area of paint, so less atmosphere; abstract effects (cheaper tiles, cheaper upgrades) need an object to stand for them. |
| **B** | Illuminated roundel | A round painted vignette inside a fine gilt and ink double ring, like a struck medal. | Ceremonial; one shape for buildings, effects and units; matches the ringed medallions used across the UI. | Leans on people rather than places (the Library is a scribe's portrait); close to the resource medallions, so a building can read as a resource. |
| **C** | Painted scene plate | A 3:2 oil painting of the place in use, edge to edge across the card, like the era and moment paintings. | The most atmosphere and story; the widest image. | At 1000 × 600 it shrinks to a thin 166 × 66 strip and the detail goes; cards with two or three unlocks (most Classical-and-later cards) don't fit; units have no matching scenes. |

### Recommendation: A

A Workshop, a Knight and a Market then read as one collection of painted pieces, on the card, in the production list and on the map. It needs no frame, it holds up at 64 px, and when a card unlocks several things (Chivalry: Agoge and Knight) the first stands large and the rest sit beside it as smaller miniatures. B is the fallback if the user wants more ceremony; C's plates are better saved for the unlock moment card, where there is room.

## The card (mocked on the real screen)

The mocks are the real research screen with the art injected after it renders: `app/e2e/unlock-art-mock.spec.ts` and `unlock-art-sheet.spec.ts` on branch `claude/unlock-art-mocks` (run on the cloud box). The shots are in [`mocks/`](mocks/) (`<size>-current.png` is today's screen); the sheet's copy is `sheet-copy.json`. Changes to the card in all three:

- The unlock moves to the top of the card, under the tech name: the payoff first, then the cost and boost, then the one primary (Research now).
- The art is 68 px tall at 1000 × 600 and 122 px at 1440 × 900, with the name and stat under it (one line at 1000 × 600).
- The separate "Details" link goes; the tech name, already a button, carries a chevron and opens the details. That pays for the art: at 1000 × 600 every card still ends whole above the panel's bottom with Research now in view (cards end at y = 545 of 550; today 521).

Not yet mocked: several unlocks on one card. Planned layout for A: the first unlock (wonder, then building, unit, improvement, effect) at full size; the rest as 40 px miniatures in a row under it, each with its name.

## What the full set needs (inventory, 2026-10-11)

From `app/src/rules.generated.json` and core `research::tech_unlocks`.

| Kind | Items | New paintings |
| --- | --- | --- |
| Buildings unlocked by a tech | Workshop, Walls, Market, Library, Roadworks, Forge, Castle, Agoge, University, Mill, Star Fort, Bank, Factory, Research Lab (upgrade tiers are their own buildings: Forge and Factory after Workshop, Castle and Star Fort after Walls, University and Research Lab after Library, Bank after Market, Mill after Granary) | 16 |
| Wonders (Great Works) | Great Library, Grand Bazaar, Royal Arsenal. They have no art today beyond the shared `moment-greatwork` painting and engraved icons | 3 |
| Buildings with no tech, shown in production and the city panel | Granary, Rice Storehouse (Japan), Yam Relay Station (Mongols' goal), Machine Works (Germany's goal) | 4 |
| Faction replacements, own name | Alpine Redoubt (Swiss Walls), Ruhr Foundry (German Factory), Lyceum (Athens' Library), Imperial Scriptorium (Byzantine Library) | 4 |
| Tile improvements | Farm, Pasture, Quarry, Mine, and their upgrades Irrigated Farm, Ranch, Deep Quarry, Deep Mine (Crop Rotation and Iron Working cards) | 8 |
| Effects | Irrigation +1 Food in every city; Code of Laws bought tiles 25% cheaper and a ring-2 claim; Military Engineering upgrades 50% cheaper; Printing Press and Recursive AI Science per Library; Railways +2 moves on roads; Quantum Computing +4 Science per Research Lab | 8 (7 if Quantum Computing reuses its building's art) |
| Units (28 kinds) | Painted miniatures exist. Five (Phalangite, Composite Archer, Cataphract, Arquebusier, Cuirassier) are painted on `astra/unit-art-five-new`, not yet on master. The Tagmatic Soldier and Celtic Ambusher faction pieces share the generic figure | 0 for A (B rings them in code; C would need 28 scenes) |
| Unit upgrade lines ("Spearmen can upgrade to Phalangites", 17 lines) | Shown as the two unit miniatures, from and to | 0 |

About **46 paintings** for A or B. Each building is painted in the era its tech belongs to.
