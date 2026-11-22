# Construction, round 3: a site that advances every turn

> **Update 2026-11-01:** the canvas-drawn ink furniture in this proposal (crane, workers, scaffold ticks, string line, stakes, stock piles) was built and then removed after the designer's complaint ("i thought we fixed all the simple drawings?"). Sites are now the painted stage sprites and the finished landmark only; see `docs/learnings.md` (2026-11-01, no ink construction).

Art direction proposal, 2026-10-24. Nobody builds until the user approves. Rounds 1 and 2 are recorded in the `legatum-art` skill; this round starts from his direction: "do some research on what SimCity looks like when things are built, or Cities: Skylines. The animations of progress towards building things feel so satisfying in those games."

**The one-line answer:** rounds 1 and 2 were both *moments at completion*. The satisfying thing in SimCity, Skylines and Civ VI is not a moment at all. It is a **construction site that exists on the map for the whole build and visibly changes every turn**, drawn as the building's own unfinished geometry, with a quiet finish when the last stage lands. So construction becomes a **map state** (a site at its stage, drawn every turn from the city's production progress), and the animation is only the small step from one stage to the next, plus a restrained finish beat.

## 1. Research

Clips of SimCity (2013), Cities: Skylines (and the older city builders it was compared with), Civilization VI's wonder movies, Anno 1800 and Manor Lords, fetched at 480p and studied frame by frame. Full notes with the clip list: `~/LegatumData/construction-playbook/research-notes.md`; the reference frames are in `~/LegatumData/construction-playbook/refs/` (the videos are deleted).

| Finding | Where it is seen | Reference frame |
| --- | --- | --- |
| **The site is persistent and advances.** Nothing is revealed at the end; the end is the last change. A SimCity neighbourhood shows fenced lots, dug foundations, a steel frame and a finished block side by side, each at its own stage. | SimCity 2013 time-lapse | `refs/simcity-lots-fenced-and-foundations.jpg`, `refs/simcity-steel-frame-rising.jpg` |
| **Site preparation reads as intent.** A cleared, staked plot with blocks beside it says "something is coming" before anything stands. | Civ VI Great Bath (stage 1), Manor Lords | `refs/civ6-greatbath-1-plot.jpg`, `refs/manorlords-plot-staked-timber-piles.jpg`, `refs/manorlords-house-2-fence-and-stakes.jpg` |
| **Stages are the building's own geometry, incomplete.** A truncated pyramid with a work platform and a ramp; the Colosseum's ring footing, then its lower tiers with the top tier missing; a house's post frame, rafters, wattle walls, thatch. Never a cut, faded or outlined picture of the finished thing. | Civ VI Pyramids and Colosseum, Manor Lords house | `refs/civ6-pyramids-2-lower-courses.jpg`, `refs/civ6-colosseum-2-ring-footing.jpg`, `refs/civ6-colosseum-3-tiers.jpg`, `refs/manorlords-house-4-post-frame.jpg` to `-7-thatched.jpg` |
| **Scaffolding hugs the form, only where work is happening.** Tight against the walls at the current working height, with dust at the top; on the top tier only when the top tier is being built. A house has none: its frame is its scaffold. | Civ VI Great Bath stage 3, Colosseum stage 3 | `refs/civ6-greatbath-3-scaffold.jpg`, `refs/civ6-colosseum-3-tiers.jpg` |
| **Materials, machines and people make it a workplace.** Block piles, timber stacks, a ramp, an ox cart, tall timber cranes standing *beside* the plot, tiny workers. | Civ VI Statue of Zeus stage 1, Manor Lords | `refs/civ6-statueofzeus-1-plot-cranes.jpg`, `refs/manorlords-house-3-ox-cart-materials.jpg` |
| **The finish is a locked camera, a last stage change, and life.** Civ VI's wonder movie is one aerial shot stepping through four states, ending on the complete wonder at night with its lights on. SimCity's Great Work finishes with its own event and the camera pulling back. Manor Lords' house is done when smoke rises from the chimney. No bursts, no score pops. | Civ VI, SimCity Great Work, Manor Lords | `refs/civ6-greatbath-4-complete-night.jpg`, `refs/simcity-great-work-launch-camera-pulls-back.jpg`, `refs/manorlords-house-8-lived-in-smoke.jpg` |
| **Upgrades rebuild in place**, house by house, so the district visibly tiers up. | Anno 1800 farmers to workers | `refs/anno-upgrade-sheet.png` |
| **Progress readouts are quiet and local.** The site itself is the readout; the bar lives on the lot or the banner. | SimCity, Civ VI | (sheets) |

What our two rejected rounds did wrong, in these terms: round 1 put a generic machine *over* a finished picture; round 2 showed a finished picture arriving (clip, fade, outline-then-fill) in two seconds at the end. Neither had a site, neither had unfinished geometry, and both compressed tens of turns of work into one reveal.

## 2. Design

### 2.1 Construction is a map state, not a moment

- While a city of yours is building a **Great Work** or **Walls**, the map draws a **construction site** at its stage every turn: the Great Work on its landmark tile (`landmarks.ts` chooses it today for finished works; the site takes the same tile from the turn the work is chosen), Walls round the town.
- The stage follows the city's production: `production_stored / production_cost` from the city detail the view already carries (`CityDetail.producing`, `production_stored`, `production_cost`). Four thresholds: **plot** (chosen, under 10%), **footing** (10%), **half built** (40%), **roofline** (75%), **finished** (100%).
- **Within a stage the site still changes every turn:** block piles shrink, the scaffold climbs the outline, the crane's load is at a different height, the ramp lengthens, workers move. These are drawn in ink over the stage art from the progress fraction, so two consecutive turns never look the same.
- **At turn resolution** the site plays a short **step** (about 1.2 s): the next courses laid in along the working line with dust, the crane hoisting, the scaffold moving up. When the progress crosses a threshold, the step lays the next painted stage in over the last **from the working line up** (0.6 s, masked at the construction line, dust along it), so it reads as courses set, not as a picture fading in.
- The site is visible to the viewer for their own cities now; rivals' sites need one public field (see 4.4). Fogged sites show at the last seen stage, like remembered cities.
- **Map animations Off:** the site still shows at its stage (it is state); only the step and the finish are skipped.

### 2.2 What shows on the map, by kind

| Kind | On the map | Why |
| --- | --- | --- |
| **Ordinary building** (Granary, Workshop...) | **No site.** The banner's production meter is the readout. On completion: a 1 s quiet beat on the banner (the meter fills and the item's icon settles in on the meter), plus idle life (smoke) if the town has none yet. | Round 2's granary "bolted on" to the town edge was the wrong model: the town art is one painting per era and size, there is no place in it for a tenth building. Nothing is better than a fake. A later increment can tier buildings into the town art the Anno way (a rebuilt quarter per building family), which is a city-art project, not an animation. |
| **Walls** | **A site round the town, built section by section.** Turn chosen: the wall line staked and strung round the town. Then the painted wall overlay of the owner's era is drawn **clipped by angle from the town centre**, the circuit growing clockwise from the gate as production comes in, with an ink scaffold hugging the live end, a dust puff there, and the string line running ahead of it round the rest of the ring. Finished: the full ring, scaffold struck. | Geometry-faithful with no new art: a wall circuit really is built in sections. The cut end of a stone wall reads as an unfinished wall end. Painted wall stages (held Codex task) can replace the cut ends later. |
| **City founding** | **Manor Lords in two seconds** on the new hex: the Settler's hex staked and strung (0.3 s), timber and stone stacked beside the plot, each hut's **post frame and rafters** rising (0.6 s), walls filled and roofs laid hut by hut (0.8 s), the first chimney smoking and the banner settling (0.3 s). The border rings out as today. | Needs painted hamlet stages (cleared and staked; frames up; walls in, roofs half laid) per era hamlet, same footprint as the finished hamlet. Without them it would again be "outlined, then filled in", so the founding keeps round 2's version until the stage art lands (Ancient and Classical first: most cities are founded then). |
| **Great Work** | **The full site** (2.3) on its landmark tile for the whole build, the step each turn, the finish beat (2.5), then the landmark stands for the rest of the game, with its name lettered under it at near zoom and the gilt-ringed icon badge at the far tier. | The biggest payoff in the game and the trailer's hero shot. |

### 2.3 The site grammar (Great Work)

Drawn in the atlas line (`style.ts` pen weights and tone inks), every element placed from the stage art's own alpha outline so it fits the building, never a generic frame:

1. **Plot:** the ground patch cleared to pale earth, four ink stakes, a dashed red string line round the footprint (the existing `drawPlot`), and the first stone and timber stacked beside it.
2. **Stock:** squared limestone blocks (pale fill, ink outline) in piles at the front corner; timber stacks at the back; both shrink as the work rises. Industrial era: iron girders and brick.
3. **Ramp:** an earth-and-log ramp against one side, rising to the working height (antiquity and the Middle Ages); a hoist tower instead from the Renaissance.
4. **Crane, beside the hall, never over it:** shear legs (Ancient, Classical), a treadwheel crane (Medieval, Renaissance), a lattice tower crane (Industrial on). Drawn heavier than the scaffold, so it says "building site" from across the map (legatum-art). Its load hangs at a height set by the progress fraction; during the step it hoists.
5. **Scaffold that hugs the form:** poles and ledgers along the stage art's outline at the working line, only on the segments that are rising this stage (the outside walls, never the courtyard), climbing with progress.
6. **Workers:** tiny ink figures at the working line, two to five, moving between turns.
7. **Dust** at the working line during the step: engraved clouds (round lobes, one ink outline, pale wash), never soft puffs (combat-fx rule).

### 2.4 How a stage is drawn

- **Painted stages, matched to the finished painting.** Each Great Work gets three painted stages from its 1254 px master using image editing (never text-to-image from scratch), same canvas, camera, ground patch, lighting and palette, so the lay-in from one stage to the next never jumps: **footing** (the footprint as low courses with a flat top, podium steps roughed in), **half built** (walls and column drums to half height with flat tops, no capitals or entablature, scaffold tight on the rising walls), **roofline** (walls complete, roof framed in open rafters, scaffold on the upper storey only, planting and ornament missing). The Codex task for the Great Library is `.progress/astra-queue/hold/art-great-library-stages.md` (released when this proposal lands on master); the full set is `hold/art-construction-stages.md`, rewritten to this spec.
- **Engraved line work on top** for everything that moves turn to turn (2.3). Both, not either: the painting carries the geometry, the ink carries the progress.
- **Hamlets and Walls** follow the same pattern (the full-set task). Walls work without stage art through the angular build (2.2).

### 2.5 The finish beat

The last step, about 2.5 s, no louder than the rest of the game's moments:

1. the final stage laid in along the working line (0.6 s);
2. the scaffold struck: it drops a little and fades (0.5 s); the crane and the last pile go with it;
3. dust settles at the foot of the walls (0.6 s);
4. the work's name is lettered under it in the map's small caps over a hairline gilt rule (0.8 s), and stays as its label at near zoom;
5. life: the existing idle layer (glints, smoke) includes the landmark tile from now on.

The VP seal on the capital (moments-fx) and the Tier B moment card ("The Great Library stands in Bellinzona", painted art, "Show on map") are unchanged and carry the celebration. No sunburst, no gilt ring, no +N pop on the map.

### 2.6 Camera and trailer

- **In play, the camera never moves on its own** (pillar 7, "nothing blocks input"). The site is where you left it; the card's "Show on map" pans to it.
- **For the trailer:** Civ VI's locked aerial shot. Hold one camera over the landmark tile and step through the turns of a real game at one turn per 0.4 s (the frame-stepping in `trailer.media.ts`): plot, footing, columns, roofline, the finish beat landing on a music hit, then a slow pull-back to the whole city. Because the site is state, this needs no special staging beyond replaying the turns.
- **Far tier:** a site is a dashed gilt-ringed badge with the work's icon (dashed means open, the ledger's grammar); finished, the solid badge the landmarks draw today.

### 2.7 Elegant, serious, Civ-like

- Nothing is dropped on top of a finished picture; nothing is revealed. The map shows a real site at a real stage.
- Ink for the site, paint for the building, gilt only for the name rule. The house pen weights and inks, no fills but the painting.
- Short steps (about 1.2 s) so a turn with many builds does not drag; the finish beat 2.5 s. Honours "Map animations" Off and reduced motion.
- Scale: a Great Work's site is about 1.75 hexes wide like the landmark; Walls stay inside the town's ring; nothing spills onto a neighbour's marks (the wonder-art rule).

## 3. Concept frames

Composited on a real 2x map frame of the designer's 8.9.6 game at turn 151 (Bellinzona, staged with the Great Library one turn from done, captured headless in Chromium on the cloud box with `app/e2e/media/construction-stills.media.ts`; the saves are copies).

- `great-library-storyboard.png`: six panels, the site at 0% (plot), 15% (footing), 45% (columns), 80% (roofline), the finish beat, and the landmark standing (the real render after the turn).
- `walls-storyboard.png`: Winterthur's Walls (T63, real 2x frame) built section by section: the circuit staked, 35%, 70%, finished. The built sections are the game's own wall overlay clipped by angle from the town centre, so this one needs no new art.
- `overview-site-on-map.jpg`: the whole frame before the build with the site ringed, to show where it sits beside the city.
- `panels/`: every panel at full size. `compositor.py` made them (run from the scratch stills; kept for re-rendering with the painted stages).

**What is mocked:** the stage paintings do not exist yet, so the stages in the strip are the finished painting cut at the construction line with a fresh-course band, plus the site grammar in ink. The painted stages (real podium, drums without capitals, open rafters) replace the cut. The strip is for judging the grammar: a persistent site, the crane beside the hall, hugging scaffold, stock and workers, a quiet finish. When the Codex task lands, `great-library-stages.png` (stages 1-3 beside the master) joins this folder and the strip is re-rendered with them.

## 4. Implementation plan

Skateboard first: one Great Work, on the map, advancing every turn, in a real game.

### 4.1 Increment 1 (skateboard): the Great Library site

- `app/src/map/constructionSites.ts` (new): `sitesOf(view)` lists, for each own city whose `detail.producing` is a Great Work or Walls, `{ city, tile, building, era, stage, k, seed }` with `k = production_stored / production_cost`; memoised per view like `landmarksOf`. `landmarks.ts` gives the tile from the turn the work is chosen (`landmarkTile` already takes a city; add the in-progress case so a site and its later landmark share the tile).
- `app/src/map/constructionFx.ts` (rewrite, keep the name): the site grammar of 2.3 as pure functions of `(site, k, t)`: plot, stock, ramp, crane by era beside the box, scaffold along the stage art's outline at the working line, workers, dust. The outline comes from the stage sprite's alpha (sampled once per sprite and cached). The old generic frame-over-the-box scaffold goes.
- `render.ts`: draw sites right after `drawLandmarks` (under units, banners and lenses, over terrain and improvements), with the landmarks' `skip` so a tile never draws both.
- Stage art: `assets/construction/building-relayuplink-stage{1,2,3}.png` from the Codex task; `momentFxDraw.ts`'s `stageImage` lookup already reads that folder.
- Settings: sites draw in every animation mode; the step and finish only when animations are on.

### 4.2 Increment 2: the step and the finish in moments-fx

Hooks the moments-fx branch needs (it is landing with construction switched off):

- `momentFx.ts`: `FxBuild` gains `from` and `to` (progress fractions before and after the turn, from the two views) and is also planned for **progress without completion** (a `ProductionProgressed` moment from the view delta, no event needed). `BUILD_MS` becomes the step (about 1200 ms) and the finish (about 2500 ms). `holdAt` holds a site at `from` until its step plays, exactly as towns are held today.
- `momentFxDraw.ts` `drawBuild`: replace `drawStages` (the clip-rise) with: the site at `from`, then the step (courses along the working line, crane hoist, scaffold climb), then a masked lay-in of the next stage when a threshold is crossed; the finish of 2.5 for the last step. The gilt ring and the ordinary building's sketch-on-the-town-edge are removed; ordinary buildings get the banner beat.
- `landmarks.ts`: `drawLandmarks` draws the finished work from the turn after the finish; the name label at near zoom.

### 4.3 Increment 3: Walls by section, then the founding

- Walls: the angular clip of the era's wall overlay by `k`, the live end's scaffold and dust, the string line ahead. No new art.
- Founding: the hamlet stage art (Ancient, Classical first) and the Manor Lords sequence in `drawFound`; until the art lands the founding keeps round 2's version.

### 4.4 Increment 4: everyone's sites, every work, the trailer shot

- Public field: the queued `public-great-works` Codex task adds rivals' *completed* works to the city view; extend it with the work *in progress* and its stage (public like the city owner), so a rival's wonder race is visible on the map.
- The other two Great Works and the wall stages from `hold/art-construction-stages.md`.
- A `trailer.media.ts` beat that holds the aerial over a landmark tile and steps the turns.

### 4.5 Who builds it: Codex Sol, Claude reviews the clip

Claude credit is short, so the increments above are written as Codex Sol tasks, each tight enough to build without Claude in the loop; Claude reviews only the final clips (re-filmed with `moments-fx.media.ts`) against legatum-art before anything merges.

| Task (`.progress/astra-queue/`) | Builds | Done when |
| --- | --- | --- |
| `hold/art-great-library-stages.md` (release first) | the Great Library's three painted stages, matched to the master, with the alpha and alignment checks and a contact sheet | `docs/design/construction/great-library-stages.png` shows stages 1-3 beside the master with the 50% overlays aligned |
| `construction-site` (to queue; spec below) | `constructionSites.ts`, the `constructionFx.ts` rewrite, the render hook, the far-tier badge, the walls-by-angle build | a site at every threshold and a wall ring at 35/70/100% in `e2e/moments` shots at both sizes, near and far zoom; `constructionFx.test.ts` passes |
| `construction-moments` (to queue; spec below) | the step and the finish in `momentFx.ts` and `momentFxDraw.ts`, the banner beat for ordinary buildings, the removal of the clip-rise, gilt ring and town-edge sketch | the Great Library clip re-filmed from T112 to T151 of the staged save, one locked camera, and the walls clip at Winterthur |
| `hold/art-construction-stages.md` (after the proof is accepted) | the other two Great Works, the hamlets, the wall stages | the contact sheet and the stage-art vitest over every source |
| `public-great-works` (pending) plus one line: the work *in progress* and its stage | rivals' sites | a rival's site in the e2e shot |

**`construction-site` spec (mode: write, model: sol, effort: high).** In `app/src/map/`: (1) `constructionSites.ts` exporting `sitesOf(view)`: for each city of the viewer with `detail.producing` a Great Work (`isGreatWork`) or `Walls`, `{ city, owner, tile, building, era, k, stage, seed }`, `k = production_stored / production_cost` clamped to [0, 1], `stage` from the thresholds 0.10 / 0.40 / 0.75, the tile from `landmarkTile` (Great Work; add the in-progress case so `landmarksOf` later picks the same tile: pass the building and make `taken` include sites) or the city tile (Walls); memoised per view with a `WeakMap`. (2) `constructionFx.ts` rewritten to this proposal's 2.3: `drawSite(ctx, site, box, sprites, k, t)` draws, in order, the plot (earth wash and stakes with the red string when `k < 0.1`), stock (block piles front corner, timber at the back, counts falling with `k`), the ramp (eras 0-3) or hoist tower (4+) on the far side from the city, the crane of the era *beside* the box (shear legs 0-1, treadwheel 2-3, lattice 4+; its base outside the stage sprite's alpha footprint), the stage sprite, scaffold frames hugging the sprite's outline at the working line (sample the alpha of the stage sprite once per sprite, cache the outline runs; poles and ledgers only on the outermost runs), workers, and dust only while a step plays. The old `drawScaffold`/`drawCrane`-over-the-box code goes. (3) `render.ts`: draw sites after `drawLandmarks`, under units and banners, every frame (the crane's hook height and the workers are functions of `k` and `seed`, not of time, so the terrain cache is untouched); far tier: a dashed gilt-ringed badge with the work's icon. (4) Walls: `drawWallsSite` clips the era's wall overlay (`cityArt` walls sprite) with a pie slice from the town centre, start angle at the gate (bottom), clockwise by `k`, the string line and stakes ahead, a scaffold frame and dust at the live end. (5) Tests: `constructionFx.test.ts` (pure in `k`; crane base outside the footprint; scaffold only on outline runs) and `constructionSites.test.ts` (thresholds, tile stability from site to landmark). (6) `e2e/moments-site.spec.ts` shots at 1000x600 and 1440x900, near and far, of the staged T151 save at `production_stored` 0, 90, 270, 480, 596 (edit the save copy as `moments-fx.media.ts` does) and Winterthur at 20, 40, 60 of 60. Run `npx tsc --noEmit -p .`, `npx vitest run src/map`, `npm run build`, the spec on the box. Commit "Construction sites: a Great Work or Walls under construction is drawn at its stage every turn".

**`construction-moments` spec (mode: write, model: sol, effort: high; after `construction-site` and the stage art land; on top of `claude/moments-fx`).** In `momentFx.ts`: `FxBuild` gains `from` and `to` (the site's `k` in the before and after views; a completion has `to = 1`); plan a build for every own city whose `k` rose this turn, not only on `ProductionCompleted`; `BUILD_MS = { step: 1200, finish: 2500 }`; `holdAt` holds a site at `from` until its step (`sites` set in `MomentHold`, read by the site drawing). In `momentFxDraw.ts` `drawBuild`: the site at `from`, then over the step the working line moves from `from` to `to` (courses laid in along it with dust, the crane hoisting, the frames climbing), a masked lay-in of the next stage sprite from the working line up when a threshold is crossed (0.6 s), and for `to = 1` the finish of 2.5: last lay-in, scaffold struck (drops 6 px and fades), dust at the foot, the name lettered under the landmark in the map's small caps over a gilt hairline (stays as the near-zoom label in `landmarks.ts`). Remove `drawStages`, the gilt rings and the ordinary building's town-edge sketch; an ordinary building gets a 1 s banner beat (meter fills, the item's icon settles). Walls use the pie-slice clip with the live end advancing over the step. Honour the On/Off setting and reduced motion (Off: no step, the site at `to` at once). Tests in `momentFx.test.ts` for the new plan entries and the hold; re-film `wonder151` (from T112 so the site is seen advancing, one locked camera, `MOMENT_FRAMES` per turn) and `found63` with `moments-fx.media.ts`; update `docs/design/moments-fx/README.md`. Commit "Construction moments: the site advances each turn, the finish beat replaces the reveal".

### 4.6 Checks

- `constructionFx.test.ts`: the site is a pure function of `k` (same input, same drawing calls), the crane is never inside the stage art's footprint, and the scaffold only touches outline segments.
- The stage art vitest from the Codex task (dimensions, alpha corners, every size).
- `e2e/moments` shots: the site at each threshold and the finish, at 1000x600 and 1440x900, near and far zoom; the UI reviewer judges them against legatum-art before anything merges.
- Clips re-filmed with `moments-fx.media.ts` for the user to react to before increment 2 ships.
