# Resource art directions

The original exploration compared three directions for wheat, cattle, stone, copper, iron, and coal. The painted-medallion direction was adopted for the shipped set; the other directions remain visual explorations. Each direction has a 256 × 256 master and 64 × 64 and 32 × 32 derivatives under its own directory. The source sheets and the Pillow extraction/layout script are retained beside them.

[View the portrait comparison sheet](resource-art-variants.png)

The sheet uses the real `app/test-results/screen-tour/1000x600/05d-eight-empires-first-turn.png` map crop. The normal tier places the paintings at 36 px. The far tier demonstrates the map's simple category-mark treatment, matching `resourceIcons.ts`; the far-tier dots are intentionally not mini paintings.

## A. Painted medallions

A small painted vignette for each deposit, held in a fine antique-brass and ink ring. This makes the art feel collectible and gives each resource a clear boundary against terrain. The wheat, cattle, and stone read quickly at small sizes. The darker iron and coal pictures have strong atmosphere, but their dark interiors can merge with a dark map tile or low-contrast display. The brass ring also introduces a second circular frame beside the resource category rim already used by the map, so the shipped UI would need a deliberate ring hierarchy.

Files: `painted-medallions/{wheat,cattle,stone,copper,iron,coal}-{256,64,32}.png`.

## B. Naturalistic objects

Unframed, individually painted objects with a soft silhouette edge: sheaf, pair of cattle, masonry, copper ore, iron and anvil, and coal. This direction most directly makes deposits desirable and tangible, and avoids adding a frame on top of the map's own category rim. It also has the least consistent outer silhouette: broad cattle and stone can feel heavier than the ore icons, and some interior details disappear at 32 px. Review the extracted alpha edges at actual device scale before considering it for production.

Files: `natural-objects/{wheat,cattle,stone,copper,iron,coal}-{256,64,32}.png`.

## C. Illuminated miniatures

Watercolour subjects on vellum panels with restrained gilding, ink contours, and manuscript corner ornaments. This direction most explicitly belongs to an atlas and gives the set a cohesive visual system. The detailed backgrounds and corner work will be less distinct at 32 px, and the rectangular panels may compete with the hex map grid; the far tier should continue to use the simple resource mark.

Files: `illuminated/{wheat,cattle,stone,copper,iron,coal}-{256,64,32}.png`.

## Generation and preparation

ImageGen produced one six-cell contact sheet per direction; each cell is the source for one icon. The generator rendered opaque fields even when transparency was requested. `render_variants.py` clears the area outside each circular medallion, natural-object silhouette, or manuscript panel to alpha, then exports the three square sizes with Lanczos resampling. The natural-object edges use a soft mask; inspect those edges at 1× before any adoption. The comparison-sheet far zoom intentionally reuses the current simple category marks, because `resourceIcons.ts` switches to dots below 18 px.

The original prompts are recorded below so the directions can be regenerated or refined consistently.

## Coal and Iron repaint 2026-10-11

The shipped Iron and Coal medallions were repainted after the original dark rock subjects proved too similar at map scale. Iron now reads as a low, wide stack of three cool steel-blue forged ingots in front of oxblood and burnt-sienna hematite, on a mid-light rust/terracotta field. Coal is a round mound of six cool blue-black faceted lumps on a much darker indigo slate field. Their shape, hue, and grayscale value separate the resources at 32 px while retaining the set's fine antique-brass and dark-ink ring and painterly oil-and-watercolour finish.

ImageGen source masters and the exact per-resource prompts are saved in `sources/iron-repaint-2026-10-11.png`, `sources/coal-repaint-2026-10-11.png`, and their adjacent `.prompt.txt` files. The generated alpha was retained and multiplied by an antialiased circular matte to remove isolated pixels beyond the ring; the 256 px masters and their 64 px and 32 px Lanczos derivatives replaced the shipped files. `iron-coal-review-2026-10-11.png` compares the two outputs with Copper and Stone at 32 px, grayscale, and 64 px.

### A prompt

> Transparent-background 3-column by 2-row contact sheet, six isolated round resource medallions, equal cells, no labels/text. Sophisticated historical strategy game atlas aesthetic: each miniature inside thin restrained antique brass + dark ink circular ring, painterly oil and watercolor, elegant old-master naturalism, nuanced visible brushwork, parchment-compatible muted earth pigments. Reading order: golden tied sheaf of wheat; pair of tawny cattle; stacked pale limestone blocks; rough copper ore nugget with subtle verdigris; dark iron ore with small forged anvil; three faceted coal lumps. Bold recognizable subjects, legible at 32px. Transparent everywhere outside rings. No glows, gradients, bevels, stickers, cartoon style, background, shadows outside medallions, borders between cells, words, letters, symbols.

### B prompt

> A precise 3-column by 2-row contact sheet of six distinct isolated square resource paintings, same exact scale, generous transparent/empty gutters, no borders or labels, no words. Direction: hand-painted naturalistic objects with no frame. Premium historical 4X strategy game art, refined oil and watercolor miniatures, parchment-compatible restrained earth palette, subtle brush texture, muted natural pigments, confident shapes and clear silhouette, soft grounded shadow directly under each object only, no halo or sticker effect. Reading order: 1 tied sheaf of ripe wheat; 2 pair of tawny cattle, full figures; 3 stack of pale cut limestone blocks; 4 rough copper ore nugget with visible subdued green patina; 5 iron ore rock beside a small forged anvil; 6 three faceted charcoal-black coal lumps. Center each object and keep it bold enough for 32px reduction. Background transparent or uniform pale parchment, no setting, no ground plane, no framing, no glow, no gradients, no typography.

### C prompt

> Create a 3-column by 2-row contact sheet of six separate square resource illustrations for a serious historical 4X game, no text or labels. Direction: illuminated manuscript miniatures, each resource within a neat square-ish miniature field with fine inked contour, restrained gilt leaf accents, delicate watercolor washes and fine hatch marks on warm pale vellum; refined medieval atlas marginalia, not childish or ornate overload. Consistent palette of parchment, antique gold, subdued ochre, umber, sage and iron blue-grey. Reading order: tied wheat sheaf; two cattle; stack of cut stone blocks; copper ore with subtle green patina; iron ore with a small anvil; three faceted coal pieces. Keep each object large and unmistakable at 32px. Equal cells and clear separation; ideally transparent outside each miniature, otherwise flat pale parchment background. No glow, glossy bevel, saturated colors, modern icon geometry, words, letters, borders between cells or page mockup.

## Current map fit

`app/src/map/resourceIcons.ts` draws medallions at normal zoom and changes to category dots below an 18 px hex radius. This exploration evaluates painted icons at 36 px and preserves the simple dot tier in the far-zoom comparison. No production choice is made here; the SVGs and map pass remain unchanged.
