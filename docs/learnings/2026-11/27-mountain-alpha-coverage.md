# Mountain alpha coverage needs high and low overlap (2026-11-27)

The shipped mountain sprites are opaque across the middle but taper at the top and end before the bottom of a hex. A main peak and spine saddles can therefore look continuous across a range while leaving grass visible inside individual impassable tiles. The gap also appears in pairs, which have no spine.

Measure the union of shipped sprite alpha within each hex, not the number of planned peaks. For the sampled isolated, pair, small group and range fixtures, the weakest tile before the fill was 0.524 covered. Three smaller, overlapping peaks around each main peak raised every sampled tile above 0.8; a position sweep of isolated and paired tiles also stayed above 0.8. Keep a centre-band check and inspect the rendered range because a total-area score alone can miss a passable-looking valley.
