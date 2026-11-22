# Late-era resource art review

Chromium Playwright captures from `app/e2e/resource-art-review.spec.ts`, 1440 × 900 viewport. Each crop shows the asset on its resource tile in the live atlas map. The normal-zoom views show the painted medallion at the top of its hex; the close views show the engraved improvement inside the hex. The far-zoom tier continues to use category marks and improvement rings.

| Resource | Medallion on the map | Improvement on the map |
| --- | --- | --- |
| Niter | Niter medallion (archived outside the repository) | Saltpetre Works (archived outside the repository) |
| Oil | Oil medallion (archived outside the repository) | Oil Well (archived outside the repository) |
| Silicon | Silicon medallion (archived outside the repository) | Quartz Mine (archived outside the repository) |

These captures were made before the parallel core rename: the fixture loaded a `Lithium` Future tile and its `LithiumMine` kind, while the visual was already Silicon and Quartz Mine. The capture spec now reads the Future resource from the rules so later captures follow the rename automatically.

The selected medallions passed the repository's art-distinctness test after comparison with every existing resource at 14, 24 and 32 px, including simulated colour-vision and greyscale views. The selected prompts and production notes are beside the shipped PNGs in `app/src/assets/resources/`.
