# Unit art directions

![Three map-unit and panel-portrait directions across five eras](unit-art-directions.png)

Art exploration only. Nothing here is wired into the game. The study carries five examples through each direction: Ancient Spearman, Archer and Horseman; a Modern Tank; and a Future military drone swarm. The generated pieces can serve as the map token illustration and its larger unit-panel portrait crop.

## A · Painted miniature on an owner-coloured base

A hand-painted figure or vehicle stands on a small sculpted base. The base is the ownership cue; the subject keeps its natural historical colours.

**Pros:** tactile and covetable; the spear, bow, horse, tracks and rotors have clear physical silhouettes; the miniature has strong presence in a selected-unit portrait.

**Cons:** the relief and bases feel more three-dimensional than Legatum's flat atlas map; small details and narrow weapons soften at far zoom; the Future drones float above their base in this study, so the owner base does not connect to their silhouette as strongly.

## B · Engraved figure in an atlas roundel

A sepia engraved subject sits inside a parchment circle with an ink edge. A thin owner-coloured rim carries the seat colour.

**Pros:** closest to the map's atlas line language; all five silhouettes fit the same footprint; the roundel remains distinct from the terrain and works as a compact, consistent marker.

**Cons:** fine crosshatching can crowd at 48 px; the roundel reduces the open shape of a full miniature; the panel portrait has less painted warmth than C.

## C · Painted portrait medallion with an owner-coloured rim

A painted bust, vehicle or drone vignette fills a circular enamel-and-parchment portrait; the owner colour traces its rim.

**Pros:** strongest painted portrait for the unit panel; human classes get face and equipment together; a common rim unifies human, tank and drone imagery.

**Cons:** the rim adds a second circular frame at map scale; mounted and ranged roles rely on bow, spear and horse details staying visible; circles read more like collectible badges than figures standing on the terrain.

## Comparison method

The map background is the real screen-tour crop archived at [`../art-audit/unit-counters.png`](../art-audit/unit-counters.png). The owner's blue is sampled from that crop's seat border and applied consistently: as a base accent in A, as a rim in B and C. Map examples also show a stack count and a segmented health bar. The normal panels use the crop at its source scale. The far panels shrink that same crop and the tokens together to approximate the smaller far-zoom footprint; they are not a separate live screen-tour capture. Existing map contents, including older square counters, remain in the screenshot underneath the overlaid art.

The 256 px files are transparent PNG masters. Each has 64 px and 48 px derivatives in `assets/`, named `<direction>-<unit>-<size>.png`. The original generated sheets are retained in `sources/`; the exact prompts are in `prompts/`. No real player-colour palette or game renderer was changed.

The class check here is visual, not player-tested: at 48 and 64 px, the foot soldier's upright spear and shield, the archer's drawn bow, and the horse with rider keep different outer silhouettes. The distinction is carried by shape and equipment rather than hue, so the classes remain separable in a grayscale or color-vision simulation. Owner identity uses both a contrasting rim/base and a numeric stack badge; health uses a segmented bar whose filled length is visible without relying on green alone. A playtest with color-blind viewers is still needed before choosing a production direction.

## Prompts and sources

- [A prompt](prompts/a-painted-miniatures.prompt.txt) · [source sheet](sources/a-painted-miniatures.png)
- [B prompt](prompts/b-atlas-roundels.prompt.txt) · [source sheet](sources/b-atlas-roundels.png)
- [C prompt](prompts/c-painted-medallions.prompt.txt) · [source sheet](sources/c-painted-medallions.png)

All sheets were generated with the built-in image generation tool. The deliverable is a design comparison, not approved production art.
