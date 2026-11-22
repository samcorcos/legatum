# Camp and ruin map sprites

These are the app copies of approved direction B, “painted jewel,” from `docs/design/camp-ruin-art/`. Each camp and ruin has its 512 px master and 128/64 px derivatives, plus matching remembered variants authored at approximately 60% opacity. The `mark-*` family is the supplied open silhouette for far zoom, with its remembered counterpart.

`map/mapFeatures.ts` chooses a 512 px miniature at 42 px hex radius and above, a 128 px miniature at normal zoom, and the 64 px far mark below `isFar`'s zoom threshold. The canvas draws both miniatures under units and banners, centered on the tile. The camp's quiet center stays behind the unit counter. Remembered tiles use the authored remembered image directly, without applying a second fade.

The originals and comparison sheet remain in `docs/design/camp-ruin-art/`.
