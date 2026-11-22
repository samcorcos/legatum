# 8.9.11 UI review comparisons

The `before/` and `after/` PNGs use the same 1000×600 and 1440×900 viewports. The numbered review frames in `before/` come from `LegatumRuns/tours/playtest-8.9.11-review`. The archive did not include a constructed-landmark map shot, so `before/*-01-landmarks.png` was captured with the original master landmark size and placement on the fixture in `app/e2e/tour/landmarks.spec.ts`.

| Number | Comparison | After shot source |
| --- | --- | --- |
| 01 | Great Works and Courier Hall on the map | `tour/landmarks.spec.ts` |
| 02 | Blocked Outrider hover and right click; extra toast placement frame | `tour/outrider.spec.ts` |
| 03 | Selected unit with both side panels open | `tour/unit-panel.spec.ts` |
| 04 | Four strategic supply chips; paired Niter and Oil 0/0 frames | `tour/era-resources.spec.ts` |
| 05 | Alliance offer at the top of the inbox | `tour/diplomacy-war.spec.ts` |
| 06 | Phalangite portrait | `tour/unit-panel.spec.ts` |
| 07 | Iron Working map and reveal card, Copper, Silicon/Quartz Mine and foreign Niter tour frames | `tour/era-resources.spec.ts` |

The landmark tour also takes an individual shot of each constructed landmark. The resource tour covers the Iron Working reveal card and map state, Copper as a bonus resource, Silicon's Quartz Mine, and the foreign Niter tile card.
The archived 04 frame shows the industrial rail, where Iron and Coal have real shortages. The paired `04-neutral-zero-supply` frames use the same fixture with only future, unresearched upgrades; its before frame was captured with the original master chip logic.
