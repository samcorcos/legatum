# Premium 3D units: start from a well-made model, don't model the hero in headless Blender

2026-11-25. The 0 A.D.-based Spearman's motion (round 6) was approved, but its look read as a dated game unit. Three premium rounds had Codex/Astra model the body, helmet, cuirass and cloak procedurally in headless Blender (MPFB body, modifiers, baked detail), in two lanes. Each round fixed one layer and exposed the next: ball-shaped knees, an egg-shaped helmet, a lumpy torso. Round 3 was further from the painting than round 6.

- Codex can fit, rig, bake, decimate, validate and render well. It can't sculpt a premium character from scratch without seeing its work as it goes.
- The route that works is the studio route: take a well-made, licence-clean source model (CC0 or CC BY, for example from Sketchfab), fit it to the existing skeleton, transfer the weights, bake its textures to our budget and keep the approved clips.
- Every Sketchfab download needs a login, so a person with an account has to sign in for the download. Store-bought models (Sketchfab Standard, CGTrader, TurboSquid) can ship in a game but can't sit in a public repo.
- Judge the look side by side with the painted unit art, in the game's renderer (three.js), not Cycles alone.
- Follow-up the same day: fitting "Hoplite Warrior" (CC BY) at a 10k LOD0 shredded its thin cloth. At about 30k LOD0 (LOD1 3k), with garment shells, a regraded bronze and a modelled crest, it read like the painting at game size in three rounds. The flaws left (a fused forearm wrap, knees, helmet sides) need hand modelling.
