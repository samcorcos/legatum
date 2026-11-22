# Camp and ruin map art

Three un-wired, transparent sprite directions for the settled barbarian camp and ancient ruins. Each subject has a 512 px master plus 128 and 64 px derivatives, and a remembered variant at approximately 60% opacity at all three sizes. `mark-camp-*` and `mark-ruin-*` are high-contrast far-zoom marks with matching remembered variants. The marks are open silhouettes without a surrounding circle: the camp uses a low palisade and pennant; the ruin uses a broken arch and fallen stones. That keeps them distinct from city seals, resource dots, and improvement rings.

The comparison sheet is [camp-ruin-art-directions.png](./camp-ruin-art-directions.png). The map composites use the real supplied screenshot crop `docs/design/art-audit/barbarian-camp.png`; the far marks and ruin placements are concept composites, not new gameplay captures.

## Approved and wired: B, painted jewel

The user approved B on 2026-10-10. The renderer uses the B camp and ruin miniatures under unit counters and city banners, with the camp centered beneath the counter so its counter-sized middle stays covered. At far zoom it uses the open `mark-*` silhouettes. Remembered sites use the supplied variants authored at approximately 60% opacity. App copies of every selected 512, 128 and 64 px asset live in `app/src/assets/map-features/`; their loading and zoom selection are documented there.

The ruin reward emblem is archaeological: a broken arch and buried stones, with the former relay-star relic removed. Notices and tile cards use the ruin's archaeological mark.

## A. Engraved vignette

**Camp prompt.** Transparent, isolated one-hex map sprite for a premium historical 4X game. Atlas ink engraving with fine crosshatching and a muted vellum-compatible earth wash. Show an oval ring of sharpened palisade stakes with an open viewer-facing entrance, hide tents around the back and sides, spears, a small smoking fire near the front, and a small red pennant on a tall rear pole. Keep the center quiet enough for a square barbarian counter; the structure must remain visible around and behind it. Refined surveyor's atlas, clear silhouette at 128 px, no fantasy, sci-fi, text, frame, glow, or shadow.

**Ruin prompt.** Transparent, isolated archaeological site sprite in the same engraved atlas ink and muted wash. Show a toppled short colonnade, a broken standing arch, fragments half-sunk into uneven earth, fallen carved blocks and sparse grass. Treat it as a real site of an earlier ancient civilisation, not a fantasy dungeon. Maintain a crisp one-hex silhouette at 128 px; no text, frame, glow or shadow.

**Legibility.** Pros: strongest match for the map's improvement line language; camp perimeter and pennant remain readable around a centre counter; the broken arch gives ruins an unmistakable place identity. Cons: hatch detail can crowd at 64 px and should simplify aggressively; at high detail, the vignette may carry more visual weight than a normal resource tile.

## B. Painted jewel

**Camp prompt.** Transparent miniature painted vignette, viewed slightly above, with a palisade ring around a clear quiet middle for a unit counter, hide tents around its perimeter, rear spears and a small smoke curl, with a restrained red pennant. Refined oil-and-gouache brushwork, parchment-compatible historic pigments, premium and serious, not a casual-game sticker. No text, border, shadow, glow, fantasy or sci-fi.

**Ruin prompt.** A compact painted archaeological vignette on transparent ground: broken classical colonnade and partial fallen arch, weathered stone sunk into earth, scattered blocks and restrained grasses. Rich but restrained antique pigments and controlled miniature brushwork, in the painted-natural-wonder family. Real archaeological remains, no fantasy or sci-fi, no text, frame or sticker shadow.

**Legibility.** Pros: most immediately beautiful and legible as a special reward destination; depth and material make the camp and ruin feel valuable. Cons: highest visual weight and strongest departure from the map's atlas ink language; the camp's warm interior can compete with the unit counter; needs careful scale and opacity so it does not read as a floating diorama.

## C. Contextual terrain

**Camp prompt.** Transparent one-hex terrain vignette in restrained atlas gouache: disturbed earth, churned grass, cut stumps and logs, partial palisade sections, low hide tents around a deliberately quiet center for a barbarian unit counter, thin smoke and a small red pennant. The camp should feel dug into the land, with a very small crisp marker. No full platform, no fantasy or sci-fi, no text or shadow.

**Ruin prompt.** Transparent terrain-integrated archaeological remains in grass: scattered fallen stones, broken low wall lines, a partial arch and toppled column, uneven earth and sparse plants. Small crisp marker, restrained historical pigment, no fantasy or sci-fi, no text, glow or shadow.

**Legibility.** Pros: most natural fit with a map whose improvements change the land; leaves the map feeling like a place, not a collection of stickers; the far mark can carry identity when the terrain details fall away. Cons: poorest near-zoom discoverability, especially for rewards that players should seek; its stone and grass can merge into the terrain unless the marker and silhouette stay deliberately strong.

## Other directions

A and C remain in the comparison sheet for reference. They were not selected for the app.

## Reward emblem updates that would match

- `app/src/assets/rewards/emblem-camp.svg`: retain the broken standard and cold hearth stones from the established reward-emblem direction, but make the standard a recognizable red pennant and add a small engraved arc of palisade stakes or hide-tent profile. The camp's map pennant and emblem should use the same silhouette. Keep it a bare ink charge without a medallion rim.
- `app/src/assets/rewards/emblem-ruin.svg`: remove the recovered relay-star sphere. Use a broken arch over a partly buried plinth or fallen carved block, with one toppled column and a few excavation stones. Keep the emblem a small, clear engraved charge in ink; no relay-star, futuristic artifact, extra frame, or fantasy dungeon cue.

These notes describe the art direction that led to the selected B map sprites and the revised ruin reward emblem.
