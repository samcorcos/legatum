# Enlisted promotion insignia

One gold badge family follows the US Army enlisted silhouette: level 1 has one chevron, level 2 two, level 3 three, level 4 three and one rocker, level 5 three and two rockers. Sparta's level 6 has three chevrons and three rockers with a gold Λ in the centre. Level 0 has no mark. These are game levels, not military rank titles; promotion rules are unchanged.

The same paths and colours are rendered as SVG in the unit panel, tile tooltip, combat forecast and turn report, and as Canvas paths on the map. At far zoom the mark stays screen sized; levels 5 and 6 also retain the gilt ring around the unit pip. The dark ink field gives the gold enough contrast on terrain without a glossy or cartoon treatment.

`before-*` and `after-*` are full Chromium screenshots from `app/e2e/max-rank-insignia.spec.ts` at **1000×600** and **1440×900**. Each level 1–5 case uses a Japanese Field Cannon; level 6 uses Sparta. Each set has `normal` map, `far` map and selected `panel` views. The test loads a WASM view and checks the promotion level before capturing. The `before` set is the integrated 8.9.11 emblem implementation; `after` is this enlisted ladder.
