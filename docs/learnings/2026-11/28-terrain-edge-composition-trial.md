## 2026-11-28: Moving terrain fillers toward shared edges did not earn its cost

Issue #368 asks for continuous ranges and forests, many variants, and clean biome transitions. The existing 2D organic layer already uses connected regions, world-lattice scatter for large woods/hills, and spine saddles for large mountains. Small regions (up to three tiles) use a main cluster plus coverage fillers per hex.

Two temporary compositions reused those small-region fillers without increasing their count: steer the first companions toward known same-kind neighbours, and select matching sprite variants by the ordered shared-edge endpoints. The displacement was a fraction of the centre-to-centre vector.

- At 0.35 displacement, the three-hex Hills coverage fixture fell to 0.5840, below its 0.60 floor.
- At 0.45 displacement with Hills left unchanged, a three-tile Mountains fixture fell to 0.7926, below its 0.80 floor. The 30 focused organic/coverage tests had one failure in each trial.
- Actual in-game captures at 1000×600 and 1440×900 showed a modest rearrangement, not a convincing continuity improvement. Neither trial was retained. No production code, art asset, or coverage threshold changed.

The current companions fill otherwise passable-looking openings inside a terrain hex; moving them to connect edges can reopen those gaps. The next experiment should use purpose-drawn connecting edge/saddle shapes while preserving interior coverage, compared with the existing region renderer before adding an atlas or increasing draw count. Include isolated tiles, 1–3 tile groups, long ranges, different-biome borders, fog and far zoom. Neighbour-dependent compositions also need chunk signatures to include their known edge mask; the current size/edge-depth/spine signature alone does not encode all six neighbours.

This is a rejected prototype, not completion of #368 or evidence that new art will pass review. Independent art review remains required for any eventual implementation.

### Follow-up: a dedicated low saddle underlay also failed the visual bar

A new transparent low saddle was generated with the built-in image tool, using `mountain-01.png` as its style reference. The prompt requested a short rugged ridge with two low shoulders and a shallow pass, matching the existing angular charcoal/taupe stone and cream snow, without trees, grass, ground ellipse or background. This tested new connecting art rather than relocating coverage fillers.

A temporary renderer drew it under known horizontal mountain neighbours at two sizes (86×30 and 110×38 world units), preserving every existing peak and filler. Baseline, narrow and broad captures each completed at 1000×600 and 1440×900. All 30 organic/coverage tests passed because the existing plan remained unchanged; those tests did **not** certify the new underlay's clipping or fog behavior. At gameplay scale, both connectors mostly disappeared behind the existing peak mass and made no convincing continuity improvement. The source image and temporary wiring were not added to the shipped assets or renderer.

The remaining obvious openings are diagonal/corner junctions, not the already-dense horizontal seam in this fixture. Before more art is produced, isolate such a junction and define an edge-clipped footprint that cannot paint into a third terrain or unexplored neighbour. Both known endpoints alone do not guarantee that a wide connector's entire footprint is known. A future connector also needs fog treatment per covered tile, not merely one opacity chosen for its endpoints. No atlas or production connector should be introduced until a mock improves that concrete junction without these leaks.
