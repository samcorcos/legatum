# spearman-fit-casey/spearman.glb: provenance

Review candidate (round r5, a test with raised map-thread limits: LOD0 up to 30k triangles, about 3 MB), not a shipped unit. Licence of the combined asset: CC BY-SA 3.0
(the 0 A.D. rig and clips are share-alike; the model is CC BY 4.0, which permits this).

## Credits

- **Model**: "Hoplite Warrior" by Casey DeDore (Sketchfab), CC BY 4.0.
  https://sketchfab.com/3d-models/hoplite-warrior-62d5291dd0454cb28a29a37056a59013
  Source: `premium-source/hoplite-warrior/scene.gltf`, SHA-256
  `257bcce19fcdb01e0c0a7b685d3d791150e790a92c3f035851749f3ac6a9e60f`.
- **Rig and the five clips** (idle, walk, attack, hit, death): 0 A.D. art by Wildfire Games,
  CC BY-SA 3.0; adapted for Legatum. Copied byte for byte from
  `units/spearman-0ad/spearman.glb` (round 6, SHA-256
  `84bf794f543fd4a1af2d1b9c2b48c89a6c52533a00d7803e553b49a22513f92c`).
- **Shield**: the sunburst aspis of the Legatum premium PBR lane (round 4, branch
  `astra/unit-anim-3d-spearman-0ad-pbr` at `2f602479`, GLB SHA-256
  `6dd1c8b15456b6a97d6341fa6532cf860a331fc1a23e8430ede23178f5bf8622`), itself built on the
  0 A.D. rig (CC BY-SA 3.0). It replaces the model's own bull hoplon.
- **Crest**: the strand crest and holder of the Legatum premium painted lane (round 4, branch
  `astra/unit-anim-3d-spearman-0ad-painted` at `bdb41206`, GLB SHA-256
  `366f770517d06e23499c0adb221e7243f07095e9396b206159ba167f5dc977d3`), built on the 0 A.D. rig
  (CC BY-SA 3.0) and seated on this helmet's crown. The model's helmet has no crest.

The exact credit is embedded in `asset.copyright` and `asset.extras.credit`.

## Changes to the model

Removed the pedestal, rain, background, sword and the bull hoplon; unposed from its
action pose onto the 0 A.D. bind pose and bone lengths; spear seated on the 0 A.D. spear
axis and scaled to its length; re-skinned with 0 A.D. weights; clothing kept at its source
topology in LOD0 (29,496 triangles; LOD1 2,864); materials baked to one 1024 WebP set (base
colour, normal, ORM); bronze regraded to warm gold-brown and skin to a tanned tone;
crest and shield rim carry the COLOR_0 team mask with neutral texels.
Round r3 adds generated linen sleeve shells (replacing the torn source sleeves), a generated
Corinthian face plate with a dark T opening, skin sunk a few millimetres under clothing, and
curvature-driven edge wear on the bronze. The model's eyes are removed (hidden by the plate).

Rebuild: see `tools/art/anim3d/kit/FIT.md` ("Real models, round r1").

Round r5: raised bevelled nasal and split eye slots on the face plate; crest widened and lifted; aspis handgrip not imported (its leather flaps stood off the thicker forearm); LOD1 keeps the aspis rim at LOD0 topology; knee weights smoothed.
