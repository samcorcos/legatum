# Research cards: benefit ledger

Selected 2026-10-26 from option C on `astra/tech-card-alternatives`. The shipped research screen uses one illustrated row for each benefit. Its heading says Building, Unit, Upgrade, Rule, or Great Work; the effect and gain are one line each. Hover or focus opens the complete description. A technology with one buildable keeps a large painting, even if it also reveals a resource (Education's University and Niter). The current era's VP objectives are immediately below the five cards.

The shared `techCardUnlocks` model folds an upgrade permission into the unit unlocked by the same technology. The unit appears once with a short “From …” line. An upgrade whose destination is not unlocked by that technology keeps its own row. The tech-unlocked moment uses the same labelled, illustrated rows.

## Before and after

Each is a full Chromium viewport capture of the Japan research fixture, with all previous eras known and the current era untouched. Both layouts use the same five technology cards and VP rules.

| Era | 1000×600 | 1440×900 |
| --- | --- | --- |
| Medieval | [Before](final/before-medieval-1000x600.png) · [After](final/after-medieval-1000x600.png) | [Before](final/before-medieval-1440x900.png) · [After](final/after-medieval-1440x900.png) |
| Renaissance | [Before](final/before-renaissance-1000x600.png) · [After](final/after-renaissance-1000x600.png) | [Before](final/before-renaissance-1440x900.png) · [After](final/after-renaissance-1440x900.png) |
| Modern | [Before](final/before-modern-1000x600.png) · [After](final/after-modern-1000x600.png) | [Before](final/before-modern-1440x900.png) · [After](final/after-modern-1440x900.png) |

`app/e2e/benefit-ledger.spec.ts` checks every era from Ancient through Future at both sizes. The research and tour specs cover faction replacements, the upgrade line, queue and keyboard actions, and the tech-unlocked moment.

## Refreshed tour shots

The research tour was captured at both sizes. The selected frames show the faction-specific research card, a single unlock, a two-benefit unlock with its upgrade path, a locked stepping stone, the longest unlock copy, Education's large painting, and the queue forecast.

| Scene | 1000×600 | 1440×900 |
| --- | --- | --- |
| Byzantium research | [View](final/tour/1000x600/60-research-byzantium.png) | [View](final/tour/1440x900/60-research-byzantium.png) |
| Single unlock | [View](final/tour/1000x600/80-unlock-card.png) | [View](final/tour/1440x900/80-unlock-card.png) |
| Upgrade path | [View](final/tour/1000x600/80b-unlock-card-two-unlocks.png) | [View](final/tour/1440x900/80b-unlock-card-two-unlocks.png) |
| Stepping stone | [View](final/tour/1000x600/80d-unlock-card-stepping-stone.png) | [View](final/tour/1440x900/80d-unlock-card-stepping-stone.png) |
| Longest copy | [View](final/tour/1000x600/80e-unlock-card-longest.png) | [View](final/tour/1440x900/80e-unlock-card-longest.png) |
| Education | [View](final/tour/1000x600/80h-unlock-card-education.png) | [View](final/tour/1440x900/80h-unlock-card-education.png) |
| Queue forecast | [View](final/tour/1000x600/90-research-forecast.png) | [View](final/tour/1440x900/90-research-forecast.png) |
