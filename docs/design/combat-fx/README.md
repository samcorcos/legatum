# Combat animations

The user (2026-10-24): "it would be nice to see arrows fly and hit someone, city walls hit a unit, units move to a space so i can see things happen... work backwards from what animations would make for an awesome short clip in our promo video."

A resolved turn is now played back on the map in the order the rules resolved it. First comes the ranged phase: every shot, staggered about 190 ms apart, with the city walls answering. Then come the movement rounds, with each melee clash on the step where it happened. Losses and captures follow. Everything is built from the viewer's own events, so nothing hidden is shown, and it is all drawn in the atlas's engraved line.

## The signature effects (what the trailer's war beat needs)

| Effect | How it looks | By era |
| --- | --- | --- |
| Volleys | A fan of ink-stroke missiles with fletching and a fading trail, arcing high and landing across the hex. Shafts stand in the ground a moment. | Arrows (Archer, Composite Archer, Keshig), crossbow bolts (flatter, heavier), musket shot (a streak and a bank of smoke), cannonballs (a dark ball with a vellum halo, a trail and a ground shadow), shells (a smoke trail and a soot column on impact), drones (a small engraved quadcopter). A siege crew throws stones or shot at a city. |
| The walls answer | A walled city fires from the wall that faces its target. | Arrows (Ancient and Classical), bolts (Medieval), cannon (Renaissance), shells (Industrial on). |
| Clash | The attacker lunges, or charges with its own step if it takes the tile. At contact: a vellum flash, a woodcut sunburst with one to three red wedges if blood was drawn, engraved dust and thrown debris, and the defender flinches. | Riders kick up a dust trail on the charge. From the Renaissance on, muskets and rifles fire as the lines close. |
| Bombardment | Heavy shot hits with a flash, a sunburst, an ink shock ring, engraved dust and debris. Bars drain as the blow lands (about 0.4 s) instead of jumping. | |
| A unit destroyed | It stands until the killing blow lands, then topples about its feet, sinks and fades, with dust where it falls and a soot plume. | |
| The capture | When the taker's unit arrives: ink shock rings (the taker's colour inside) and dust off the walls. The capture celebration's gilt sunburst now waits for this moment. | |
| A city under siege | It smoulders for the whole turn, not only on impact: engraved smoke columns, fires at their feet below half HP, and at 0 HP a breach (rubble spilled from the wall and a third column). | |
| Air strike | An engraved aircraft silhouette crosses high over the target, its shadow sliding below, and drops its bombs. | Bomber and Fighter (Modern on). |

Every damage cue ("−31", "Destroyed") waits for its own blow, so cues appear in sequence, not all at once.

## Art rules (legatum-art)

- Ink, vellum and earth tones; the one accent red only where a blow lands. No glow, no blur, no neon particles.
- Smoke and dust are engraved clouds, as old maps draw them: round lobes inside one ink outline over a pale wash, with a few hatches on the shaded side. Smoke is grey-blue, never pure white. The first soft-gradient puffs read as airbrushed smudges.
- Missiles carry a vellum halo, so they read over dark forest.
- Zoomed out (the far tier), only arcs, dots and rings are drawn.

## Setting and accessibility

Settings (title screen and game menu) has **Map animations: On or Off** (user, 2026-10-24: On/Off only, no speed choice; a saved Fast reads as On). Off is the map without animations: every change is instant, units stand where they ended, and the damage numbers still show. Under the system's reduced-motion setting the choice shows Off and cannot be changed. The playback never blocks input or the next turn. If a new turn resolves, its playback replaces the old one.

## Clips and stills

- `combat-fx-v2.mp4`: 16.6 s, 1080p60, frame-stepped from real saves, five real turns. Lugano (8.9.6, T190): the Swiss cannon volley, the Mongol arquebus answer, a kill, the cavalry riding up, Lugano smouldering at 0 HP. Aarau (8.9.2): crossbow bolts, a musket answer, a unit toppling, the capture. Lugano's capture (T200). Shells at Geneva (8.9.4). A bomber pass over Osaka (8.9.5).
- `stills/`: single frames from those turns.
- Rebuild: `app/e2e/media/combat-fx.media.ts` (see its header), on the cloud box, with copies of the saves in `app/e2e/media/fx-saves/` (never the designer's own files).

## Pacing (user, 2026-10-24)

A unit's steps in consecutive rounds run as one smooth glide, eased only at its ends with an even pace in the middle. A multi-hex charge is one charge, and units still meet where the rules had them meet. A unit that waits a round starts a new glide. A step lasts 280 ms, and all the moves fit in at most 1.4 s.

Measured on busy real turns, with the human seat moved to the empire with the biggest army in contact (copies of 8.9.2 T221, 8.9.4 T234, 8.9.5 T172 and 8.9.6 T185: up to 13 movers, 5 clashes and 3 deaths): the playback lasts 3.1 to 3.9 s, with the last action at 1.3 to 2.3 s. The target is 4 to 6 s. Before and after clips: `~/LegatumRuns/combat-fx/before4/` and `after3/` (`battle185.mp4`, `battle172.mp4`).

`app/src/map/combatFxPacing.test.ts` holds three deterministic synthetic budgets (total / last action): a quiet one-shot turn is **1.574 / 0.974 s** (limit 2.5 s); a typical war turn with 6 shots, 4 clashes, 6 movers over 4 rounds and 2 deaths is **4.274 / 2.997 s** (limit 4.5 s); a huge turn with 30 shots, 20 clashes, 25 movers over 6 rounds, 8 deaths and 2 captures is **5.878 / 4.078 s** (limit 6.0 s). Short volleys retain 150–300 ms gaps. Large volleys use an even gap, shrinking toward 60 ms; for 30 shots the 60 ms legibility floor spreads launches over 1.74 s, beyond the 1.1 s target span. Flights keep at least their kind's nominal duration, movement fits within 1.4 s, and the final effect gets a 500 ms tail (at least 600 ms after the volley marker on a shot-only turn).

## Frame cost

Measured on the cloud box. It draws without a GPU, so compare on with off rather than reading absolutes. `perf` and `perf-off` in the media spec, at 1440x810, 4/3 device scale.

| Case | Map draw per frame (mean / max) | Of which effects |
| --- | --- | --- |
| Turn 190 playback, animations off | 1.42 / 3.6 ms | 0 |
| Turn 190 playback, animations on | 1.49 / 3.3 ms | under 0.1 ms |
| Stress: 80 shots, 20 clashes, 20 deaths on screen at once | 2.70 / 6.3 ms | 1.35 / 4.8 ms |

## Code

`app/src/map/combatFx.ts` builds the plan and the per-frame unit state, as pure functions of time; its header says how to add an effect. `combatFxDraw.ts` holds the drawing and the shared engraved blocks (`cloud`, `rays`, `flash`, `ring`, `debris`). The preference lives in `feel/combatFxPreferences.ts` and the control in `ui/CombatFxSettings.tsx`. The tests are `combatFx.test.ts` (unit tests) and `e2e/tour/combat-fx.spec.ts` (the tour, frame-stepped).
