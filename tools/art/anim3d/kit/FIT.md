# Fitting a third-party model onto the 0 A.D. hoplite rig

`fit_source_to_0ad.py` takes a character model (glTF/GLB or FBX, any pose, with
or without an armature) and writes a contract GLB that uses the approved 0 A.D.
Spearman skeleton and its five clips. The skeleton nodes, inverse binds and every
animation accessor are copied byte for byte from
`units/spearman-0ad/spearman.glb`, so the motion is identical to the approved
round by construction (checked: all 5 x 87 channels and the inverse binds match).

## Inputs (masters outside git)

The approved rig GLB and the sources this kit was proven on are art masters listed in
`tools/art/sources-manifest.json`; fetch them with
`python3 tools/art/fetch-sources.py tools/art/anim3d/units/spearman-0ad/spearman.glb` (and the
`tools/art/anim3d/sources/...` paths; listed: the Hoplite Warrior, the 0 A.D. hoplite actor, the approved rig GLB, the PBR-lane aspis and painted-lane crest GLBs, the two CC BY helmets and tolnaydavid's `hoplite-tolnay` glTF). `--rig` defaults to that GLB. The three.js review
viewer and recorder (`viewer/record.mjs`) referenced below live on the animation branches
(`astra/unit-anim-3d-fit`), not on master yet. Manifest entries for third-party masters carry two extra fields, `source` (where the file came from: an author and URL, or an upstream repository and revision) and `license` (an SPDX id such as `CC-BY-4.0` or `CC-BY-SA-3.0`), because these files are not our own work: the credit and licence must travel with them. The fetch tooling ignores both fields. Needs Blender 4.5+ (5.3 used) with numpy;
`fit_test_source.py` also needs the MPFB 2 extension (`env.sh`).

## Usage

```sh
source tools/art/anim3d/env.sh
~/LegatumRuns/anim-pilot/bin/blender-slot --factory-startup --python-exit-code 1 \
  --python tools/art/anim3d/kit/fit_source_to_0ad.py -- \
  --source ~/LegatumRuns/anim-pilot/premium-source/hoplite-warrior/scene.gltf \
  --out tools/art/anim3d/units/spearman-fit/spearman.glb \
  --credit '"Hoplite Warrior" by caseydedore, CC BY 4.0 (Sketchfab)'
python3 tools/art/anim3d/validate_glb.py tools/art/anim3d/units/spearman-fit/spearman.glb
GLTF_VALIDATOR_MODULE=$HOME/LegatumRuns/anim-pilot/oad/validator/node_modules/gltf-validator \
  node tools/art/anim3d/viewer/validate-khronos.mjs tools/art/anim3d/units/spearman-fit/spearman.glb report.json
```

The `viewer/` scripts (`record.mjs`, `validate-khronos.mjs`) live on the `astra/unit-anim-3d-fit` branch, not on master yet.

The run takes under a minute on 3 CPU threads at the 2048 bake size. Heavy outputs
go to `--work` (default `~/LegatumRuns/anim-pilot/fit/<out stem>/`):
`landmarks.blend` (the normalised source with a marker at every detected joint and
the limb centre lines), `fitted-high.blend` and `fitted-check.blend` (the
unposed full-resolution model, the second with the 0 A.D. joints marked),
`fitted-lods.blend`, the 2048 masters (`basecolor`, `normal`, `ao`, `roughness`,
`metallic`, `orm`), the shipped 1024 JPEGs and `fit-report.json`. A stub
`PROVENANCE.md` and a copy of `LICENSE-ART` are written next to the GLB if missing;
replace the stub with a reviewed record before shipping.

| Option | Use |
| --- | --- |
| `--part PART=REGEX` | Force objects or materials matching REGEX to `spear`, `shield`, `helmet`, `cloak`, `body` or `drop` (removed). Repeatable. |
| `--team REGION=REGEX` | Faces of matching materials/objects become team colour; REGION is `crest`, `shield rim` or `cloak trim`. |
| `--no-auto-team` | Only the `--team` rules. |
| `--landmarks FILE` | JSON `{joint: [x, y, z]}` in the source's glTF coordinates (Y up, before normalising) to override detected joints, e.g. `forearm_R` for a misplaced elbow. Joint names are the 0 A.D. ones in `fit_rig.CHAINS`. |
| `--yaw DEG` | Facing correction when the toes mislead the automatic one. |
| `--flip-spear` | The blade/butt test (wider end is the blade) got it backwards. |
| `--lod0 / --lod1` | Triangle budgets (defaults 9,600 / 2,900, under the 10k / 3k contract). |
| `--bake-size / --ship-size / --jpeg-quality` | 2048 masters, 1024 shipped, quality 88 by default. |
| `--max-height` | Ceiling for the crest (default 1.93 m; the contract allows 1.65 to 1.95). |
| `--skin REGEX` | Objects that are the body's skin (default `body`); every other body object is a clothing layer that takes the unpose of the skin under it, keeps 1 cm clearance, and is baked on its own. |
| `--add-crest` | Model a horsehair crest arc on a crestless helmet (team colour). |
| `--shield-glb GLB` | Use the aspis (vertices rigid to `shield_arm`) of another unit GLB on this rig instead of the source shield. |
| `--texture-format` | `webp` (default; keeps a 1024 set well under 2 MB) or `jpeg`. |
| `--smart-uv` | Smart-project a new atlas instead of repacking the source's own UV charts. |
| `--crest-glb GLB` | Take the strand crest (team region `crest` plus `crest_base`) from another unit GLB on this rig, seated on this helmet's crown. |
| `--sculpt-override PART=FILE.blend:OBJECT` | Replace PART's LOD0 geometry with a hand-sculpted copy of that LOD0 object (saved from `<work>/fitted-lods.blend`). Vertex count and per-vertex UVs must match, else the run stops; the full-resolution part follows by the sculpt delta (4 nearest LOD0 vertices), so LOD1 and every bake are re-derived. Repeatable. |
| `--keep PART` | Keep PART at full source topology in LOD0 (thin garments shred when decimated). |
| `--grade-bronze` | Regrade metallic texels to the painting's warm gold-brown: value from the source, darker cavities (AO), brighter convex edges (baked pointiness), rougher. |
| `--grade-skin HEX` | Move skin texels' mean to this sRGB colour, keeping 90% of their variation. |
| `--sleeves FRACTION` | Replace the upper-arm garment with fitted linen sleeve shells (`fit_shells.sleeve`) down to FRACTION of the upper arm. |
| `--strip BONE` | Delete clothing around a 0 A.D. limb bone where it cannot follow the clips cleanly. |
| `--corinthian` | Close an open helmet with a face plate: cheek and nasal guards, dark T opening, dark interior. |
| `--press-skin` | Sink skin a few mm under any clothing or armour close above it. |
| `--stop-after-landmarks` | Write `landmarks.blend` and stop; check the joints first on a new model. |

`fit_sheet.py` renders Workbench contact sheets: the source, the fitted rest
(`--yup` for the work scenes, `--xray --colour material` to see the joint
markers), and every clip with close crops on named joints (`--close
arm_R,forearm_R,arm_L,forearm_L,leg_L,leg_R --no-props --matcap`) for checking
candy-wrapping. `FIT_DEBUG=1` colours `fitted-high.blend` by the bones that
unposed each vertex (`--colour vertex`).

## Pipeline

1. **Import and weld.** Meshes are evaluated (armature modifiers applied), moved to
   glTF space (Y up, +Z forward), welded at split seams and stripped of shape keys.
2. **Props and parts** (`fit_props.py`). Each loose part is classified: CLI
   overrides, then name hints on objects/materials (spear, shield/aspis,
   helm/crest/plume, cloak/cape), then geometry (a long thin part is the spear,
   a large flat round part the shield, small parts inside those envelopes join
   them). The helmet is found after the head: loose parts around it.
3. **Orientation and scale.** The toes give the facing; the body is scaled so its
   crown-to-toe geodesic length matches the 0 A.D. body's.
4. **Joints** (`fit_landmarks.py`). With an armature whose bones map to the 0 A.D.
   joints (MPFB/Unreal, Mixamo and Rigify-like names), its posed joint positions
   are used. Otherwise the body is voxelised as a solid (flood fill from outside),
   and Dijkstra from the body centre finds the limb tips by farthest-point growth:
   each new tip is the voxel geodesically farthest from all paths found so far.
   The two lowest tips are the feet, the bulkiest remaining tip is the head, and
   the two longest branches are the hands. Joints sit at the 0 A.D. bone lengths
   (scaled) from each tip along the limb centre line; elbows and knees snap to the
   centre line's greatest bend when the limb is bent.
5. **Unpose** (`fit_pose.py`). Every bone maps its source segment onto the 0 A.D.
   one: a rotation (full frame for the torso, swing only for limbs), a stretch to
   the 0 A.D. length, girth unchanged. Vertices blend at most two neighbouring
   bones, slerping the rotation about their shared joint (no collapse when a
   raised arm comes down 150 degrees). Guards keep the belly, crotch and waist on
   the pelvis map and the shoulders on the chest; stretched edges and the shoulders
   are then relaxed by smoothing the displacement, not the positions. Heels are
   shortened to the 0 A.D. heel. Cloaks follow the nearest torso vertex and are cut
   to the 0 A.D. hem.
6. **Rigid props.** The helmet follows the head map (shrunk only if the crest would
   break the height ceiling). The spear is aligned to the 0 A.D. spear's axis and
   grip fraction, blade forward as in 0 A.D., and scaled to its length. The shield
   goes rim centre to rim centre, outer face out, inner face at the 0 A.D. strap
   depth, at most 10% wider than the aspis.
7. **LODs.** Per-part collapse decimation to budget; one smart-UV atlas over all
   LOD0 parts; LOD1 is decimated from LOD0 and keeps its UVs.
8. **Skin weights.** Blender Data Transfer modifiers, nearest face interpolated,
   from the 0 A.D. body split by chain (torso, each arm, each leg), each masked to
   the vertices unposed with that chain, then limited to four influences and
   normalised. Cloaks take the 0 A.D. cape's weights. Props are 100% to
   `weapon_R`, `shield_arm` or `helmet`.
9. **Bake** (Cycles CPU, 2048). Tangent normals from the full-resolution fit
   (kept in the receiving hemisphere, relief limited to a 30-degree cone), AO of
   the low mesh at 3 cm reach (floored at 0.35), and base colour, roughness and
   metallic through emission. ORM packs R = AO, G = roughness, B = metallic.
10. **Team mask.** Red-hued helmet faces are the crest, the outer 14% of the shield
    radius the rim, the bottom 12% and outer edges of a cloak its trim; `--team`
    rules add more.
11. **Floor and export.** A bind-space push keeps every vertex above the floor in
    all clips at 120 fps (`fit_ground.py`, from the painted lane), then
    `fit_pack.py` writes the GLB: two skinned meshes, one material, quantised
    attributes, JPEG maps, the combined credit in `asset.copyright` and
    `asset.extras`.

## Proof run (2026-11-25)

Source: `fit_test_source.py`, a CC0 MPFB body posed into an overhand-thrust
lunge with generated spear, shield, helmet and crest and a cloak, exported as a
mesh-only GLB (no armature), like Sketchfab model #1. Result: valid contract
(LOD0 9,592 / LOD1 2,897 triangles, 30 joints, 1.93 m, 1.68 MB), Khronos 0 errors,
0 warnings, three.js runtime check passes, clips identical to the approved GLB.
Elbows and knees show no candy-wrapping in any clip. Review files:
`~/LegatumRuns/anim-pilot/review/3d-v2/spearman-fit-test-*`.

## Limits

- **Shoulders are the weak joint** after a large unpose: a small peak remains on
  the raised-arm shoulder in the proof. A sculpted source in a less extreme pose
  is easier; if it shows, pass `--landmarks` for `arm_R`/`arm_L`.
- **A cloak that wraps the shoulders** can clip into the upper arms when they
  move, and the body can poke through a loose cloak. Cloak shape is not re-draped.
- **Fused props**: a helmet, spear or shield that is one connected piece with the
  body is not split automatically; use `--part` with a material name, or separate
  it in Blender first.
- **The 0 A.D. skeleton is thin and short-limbed.** A bulky body inherits its
  weights by nearest face, so very heavy arms or thighs can pick up neighbouring
  weights; check the clips.
- **Thin double-sided shells** (cloaks, shield dishes) bake with some AO noise.
- Hands keep their source pose (a fist around a spear is fine); fingers are not
  re-posed to the 0 A.D. hand.
- Detection expects a standing or striding humanoid: two feet near the ground and
  the head above half height. Crouching or kneeling sources need `--landmarks`.

## When the real models arrive

1. Put each download in its own folder under
   `~/LegatumRuns/anim-pilot/premium-source/` and note its licence page.
2. Run with `--stop-after-landmarks` and render
   `fit_sheet.py <work>/landmarks.blend out.png --yup --xray --colour material`.
   Check the joint markers; fix with `--landmarks` or `--yaw` if needed.
3. Check the `parts` line in the log: each of spear, shield, helmet must be found
   (the run stops if not); fix with `--part`.
4. Run fully, validate, and look at the clips and joint close-ups with
   `fit_sheet.py`; record with
   `record.mjs --glb <out> --prefix spearman-fit-<model>`.
5. Model #2 (404k triangles) needs no change: decimation and the bake use the
   full-resolution fit; expect a few minutes.

## Real models, round r1 (2026-11-25)

```sh
S=~/LegatumRuns/anim-pilot/premium-source
# Hoplite Warrior (Casey DeDore): glTF, action pose, no crest, bull hoplon swapped for the PBR-lane aspis
fit_source_to_0ad.py -- --source $S/hoplite-warrior/scene.gltf \
  --out tools/art/anim3d/units/spearman-fit-casey/spearman.glb --work ~/LegatumRuns/anim-pilot/fit/casey \
  --credit 'Hoplite Warrior by Casey DeDore (Sketchfab), CC BY 4.0' \
  --part 'drop=Pedestal|Rain|Background|Phalanx|Ground|Sword' --part 'spear=^Spear' --part 'shield=^Shield' \
  --shield-glb <pbr-lane spearman.glb> --add-crest --jpeg-quality 72
# Hoplite (tolnaydavid): 23 meshes, ~404k triangles, near A-pose, cm units
fit_source_to_0ad.py -- --source $S/hoplite-tolnay/scene.gltf \
  --out tools/art/anim3d/units/spearman-fit-tolnay/spearman.glb --work ~/LegatumRuns/anim-pilot/fit/tolnay \
  --credit 'Hoplite by tolnaydavid (Sketchfab), CC BY 4.0' \
  --part 'drop=^Sword|^Spear_straps' --part 'spear=^Spear_low' --part 'shield=^Shield|^wire_low' \
  --part 'helmet=^Helmet' --skin '^Body_low' --team 'crest=Helmet_fur' --jpeg-quality 72
```

Both pass `validate_glb.py` and Khronos (0/0/0/0) at 1.26 MB and 1.40 MB, and their clips
and inverse binds are byte-identical to round 6. Record with
`record.mjs --glb <out> --prefix spearman-fit-<model>-r1`. It now also writes the 160 px
sprite MP4s at the legacy rates and `<prefix>-sheet-48.png`, and it no longer fails a clip
whose full extent (spear, fall) is taller than the figure.

**What still blocks a premium result** (both models, clearer on tolnaydavid's):
decimating many thin, layered garment shells (sleeves, pteruges, cape, skirt) by 6x to 40x
to reach 10k triangles shreds them into jagged flaps at close range, and those flaps poke
out at the knees and shoulders in the clips. Fixing this needs per-garment retopology or
remeshing (or merging garment shells into the body before decimating), not a faster
decimator. Separately, tolnaydavid's crest is a stack of translucent cards (alpha
blend 0.18); drawn opaque it reads as a dark spiky mass.

## Hoplite Warrior, round r2 (test limits, 2026-11-25)

The map thread allowed, for this test only: LOD0 up to 30k triangles, LODs together under
35k, at most 64 bones, one material, 1024 textures, about 3 MB, and KHR_mesh_quantization or
EXT_meshopt_compression. The output uses KHR_mesh_quantization (as every LOD here does) and
EXT_texture_webp; no meshopt. Validate with the explicit flags (the defaults are unchanged):

```sh
python3 tools/art/anim3d/validate_glb.py tools/art/anim3d/units/spearman-fit-casey/spearman.glb \
  --lod0-max 30000 --lod-total-max 35000 --max-bytes 3000000 --max-joints 64
```

Build: the r1 command plus `--crest-glb <painted-lane spearman.glb> --keep layer --lod0 29500
--lod1 2900 --grade-bronze --grade-skin a8724f` (drop `--add-crest`). Clothing now also
takes its weights from the skin under it (after the skin's 0 A.D. transfer), so cloth and
skin deform together. `record.mjs --squad` adds a 400 x 160 squad of three (idle, walk).

## Hoplite Warrior, round r3

r2 plus `--part 'drop=...|^Eye' --sleeves .45 --strip forearm_L --corinthian --press-skin
--grade-skin b07249` (same test limits and validator flags). The shield now slides out up to
4 cm before any body vertex is pressed behind it; pressing many forearm vertices flat had
left a jagged forearm. Recorded with `record.mjs --squad`; the helmet close-up is
`fit_sheet.py --clips idle,attack,death --close head` in front, three-quarter and side views.

## Hoplite Warrior, round r4 (LOD1 garments)

r3's LOD1 had lost every garment: decimated hard, the coarse clothing sank under the
less-decimated skin. LOD1 now drops skin hidden under clothing first (faces whose
vertices all lie within 3 cm under a garment), budgets garments ahead of skin
(weights layer 1.8, cloak 1.0, body 1.0, helmet 0.6, shield 0.35, spear 0.12), and never
takes the excess from garments. Garments export as the region part `garment`, and
`validate_glb.py` now fails a LOD1 that lacks a part LOD0 has or keeps under half of a
part's surface (30% for `body`, whose hidden skin may go). `record.mjs --lod-compare` writes
`<prefix>-lod-compare.png`. The face plate is hammered dark bronze before the grade; the
aspis keeps the device only on its outer face, its inner face plain bronze.

## Hoplite Warrior, round r5 (close-up pass)

r4 plus `--keep shield --crest-width 1.9 --knee-smooth`. `kit/closeups.py` renders 2048 px Eevee
crops (head, LOD1 shield rim, left forearm, knees in idle, walk and attack) and
`kit/closeup_sheets.py` puts each region beside the painting. LOD1 keeps the aspis's rolled
rim at LOD0 topology and decimates only the field; rim neutral texels are pale bronze. The
crest front and the face plate's side, and the greave tops, need hand sculpting:
`~/LegatumRuns/anim-pilot/fit/r5/*-sculpt.blend`.

## Hand-sculpt round trip

LOD0 is rebuilt deterministically from the same arguments, so a sculpted LOD0 object drops
back in by vertex index: `--sculpt-override helmet=helmet-crest-sculpt.blend:LOD0-helmet`.
Proven on unmodified copies of the r5 sculpt files: geometry, weights, UVs, colours, regions,
JSON and textures identical to r5, 9 of 20,960 tangents one int16 step apart. The ready
command is `~/LegatumRuns/anim-pilot/fit/r5/rebuild-with-sculpts.sh` (see `SCULPT-README.md`).

## Scripted sculpt experiment (r5)

`kit/scripted_sculpt.py <copy.blend> helmet|greaves` moves vertices only (crest fan and
grooves, face-plate rear edge onto the dome, greave tops re-profiled with a rim), so
`--sculpt-override` accepts it; `kit/sculpt/scripted.sh` runs it on fresh `*-scripted.blend`
copies and rebuilds to `units/spearman-fit-casey-sculpted/`. `--crest-strands N` adds
horsehair bump to the crest bake. Result: the crest and plate barely change and the greaves
got worse (pointed front, skin slits); scripting cannot stand in for the sculpt at these
vertex counts. Use the GUI sculpt of the untouched `*-sculpt.blend` files.

## r5/r6 summary (Hoplite Warrior, for provenance)

**Outputs.** r5: `units/spearman-fit-casey/spearman.glb` (the map's GLB, sha1 0d3a1de3).
r6: `units/spearman-fit-casey-r6/spearman.glb` (test path). Both validate with the map
flags (`--lod0-max 30000 --lod-total-max 35000 --max-bytes 3000000 --max-joints 64`), Khronos
0/0/0/0, and carry the round-6 skeleton and clips byte for byte.

**r5 changes (close-up pass).** Corinthian face plate with two eye slots split by a raised,
bevelled nasal plus the cheek-guard gap, in geometry; the aspis rim kept at LOD0 topology in
LOD1 with pale-bronze neutral texels and a 32 px bake margin (no pale sliver); the PBR aspis's
handgrip no longer imported (it stood off the forearm as flaps); knee weights smoothed.
Close-up harness: `kit/closeups.py`, `kit/closeup_sheets.py`.

**r6 crest part.** `--crest-mass` builds the crest as its own part (`fit_shells.crest_mass`):
a lofted mass along a front-to-back arc over the crown that leans back and trails behind the
neck guard, a teardrop section fuller toward the top, 11 strand grooves in real geometry
(about 9 mm) following the flow, tufted top edge, streaked near-neutral texture with darker
roots. 44 x 22 sections, 1,976 triangles in LOD0, decimated to 70 in LOD1 (silhouette kept).
Rigid on `helmet`, team colour via COLOR_0, baked into the shared atlas, same single material.
`kit/crest_clearance.py` checks every clip at 30 fps: no contact with the spear shaft or the
shield disc (no cloak on this model).

**r6 budget trims.** LOD0 stays at 29,497 with the crest kept whole: `--hide-skin-lod0 .015`
drops skin faces within 1.5 cm under clothing or armour (1,848 faces, never visible), and the
proportional budget takes the rest from the helmet shell (2,095), skin (3,250) and spear
shaft (274). LOD1 2,758 (LOD1 also drops hidden skin, 1,030 faces).

**Sculpt round trip.** `--sculpt-override PART=FILE.blend:OBJECT` re-imports hand-sculpted
LOD0 parts (vertex count and UVs must match); proven on unmodified copies (identical
geometry). A scripted stand-in (`kit/scripted_sculpt.py`) did not help (greaves got worse) and
is not used. With the new crest part, a helmet sculpt must be re-saved from an r6 work file.

**Still open.** From the side the face plate reads as a separate visor; the greave tops end in
pointed, notched edges instead of a rounded knee guard; the bronze is still mottled in places.
These need hand sculpting/painting, which is not available in this pipeline.

## r7: replacing the helmet and greaves (`--replace-part`)

> **Historical (r7 to r9b):** the tolnaydavid `hoplite-tolnay` greaves used in r7, r8, r8b and
> r9b, and their recipes `sculpt/r7.sh`, `r8.sh`, `r8b.sh` and `r9b.sh`, are kept for the record
> only. That source is not archived and is not needed to rebuild the shipped model: from r10 the
> greaves are built from the fitted shin (`--replace-part greaves=built`).

`kit/fit_replace.py`. The source part should be Y up, facing +Z (glTF convention); fix others
with `--replace-rot`. `:OBJECT` picks one mesh (prefix match), e.g. dropping a helmet's own plume.
- **helmet**: deletes the fitted helmet and face plate, then a bounding-shell fit: scaled so its
  width and depth enclose the head skin with ~8 mm clearance (shell walls taken as 12%), crown just
  above the head top, then grown 3% at a time until no crown/brow skin pokes out (BVH rays). Rigid
  on `helmet`; the `--crest-mass` crest is built afterwards on the new shell.
- **greaves**: deletes source clothing faces on the shins (any vertex in the shin region, so no
  leftover rim strips), splits a both-legs object at x = 0 (or mirrors a single greave), and fits
  each to its shin from 7 cm above the ankle to 5 cm over the knee, cross-section from that shin's
  own skin (2-98 percentile, ~1 cm clearance, at most +12% growth). Skin bulging through is pressed
  back under it. Weights are direct, not transferred: 100% `leg_X`, easing to 50% `thigh_X` across
  the knee (a nearest-face transfer picked up the other leg and threw spikes). Its own part `greave`.
- Credits and licences go into the GLB credit, `asset.extras.replaced_parts` and the stub PROVENANCE.

Placeholder proof: `kit/placeholder_parts.py` writes a CC0 Corinthian-style helmet and greave;
output `units/spearman-fit-casey-r7-test/` (not committed). Real round r7: the timothyhsu helmet
(`:helmet_low`) and tolnaydavid's `Leg_armor_low`, output `units/spearman-fit-casey-r7/`:

```sh
S=~/LegatumRuns/anim-pilot/premium-source; F=~/LegatumRuns/anim-pilot/fit
fit_source_to_0ad.py -- <r6 arguments without --corinthian> --crest-tail .95 --keep helmet \
  --replace-part helmet=$S/helmets/pbr_corinthian_helmet.glb:helmet_low \
  --replace-credit 'helmet=PBR Corinthian Helmet by timothyhsu (Sketchfab), CC BY 4.0' --replace-license helmet=CC-BY-4.0 \
  --replace-part greaves=$S/hoplite-tolnay/scene.gltf:Leg_armor_low \
  --replace-credit 'greaves=Hoplite by tolnaydavid (Sketchfab), CC BY 4.0' --replace-license greaves=CC-BY-4.0
```
(The full command is `kit/sculpt/r7.sh`.) `kit/crest_clearance.py` must report no contact.

## Round r8 (helmet scale, crest seat, greave finish)

- Helmet: `fit_helmet` now fits the shell at head size (`--helmet-size`, 1.0 = inner width/depth
  of the shell on the head plus 4 mm); it never grows. `tuck_skin` moves skull and ear skin that
  pokes through inward (not the face behind the openings, not the neck below the rim). The
  helmet's inner faces are baked dark (eye and mouth openings show a void). `--helmet-uv-share 2`
  gives the helmet four times its packed atlas area.
- Crest: `--crest-shift` moves the arc forward (r8: 3 cm), `--crest-tail .9`.
- Greaves: `finish_greave` clamps the top to a smoothed arc, adds a rolled rim tube and a kneecap
  dome; `delete_covered_skin` deletes skin in the greave's angular/height span and sinks the
  edge skin under it, in the fitted mesh and both LODs.
- Bronze: the regrade high-passes the source luminance (24 px masked box blur) so fine
  engraving stays and dirt blotches go; replaced helmet and greaves are always graded as bronze.
- Checks: `kit/armour_cover_check.py` (skin outside the greave surface per frame, in the shin's
  moving frame) and `kit/crest_clearance.py`.
- Still open after three rounds: tolnaydavid's greaves bake pale and their lower edge shows skin
  slivers (the check still finds up to 21 vertices in walk); the kneecap dome reads as a ball.

## Round r8b (neck, bronze palette)

`tuck_skin` now also tucks jaw and neck skin a horizontal ray shows under the helmet rim
(toward the neck axis, level), keeping the face behind the eye slots and the nasal gap.
`--bronze-palette dark,mid,light` (r8b: `4a2c0c,915d20,e5b475`, sampled from
`spearman-panel.png`), `--bronze-rough .38`, `--bronze-detail .9` and `--bronze-ao .3`
(darker cavities on metal, metallic at least 0.95). Build: `kit/sculpt/r8b.sh`.

## Round r9 (greave comparison)

- `kit/extract_greaves.py <scan.glb> <out.glb>` cuts greaves out of a fused scan: faces in each
  leg's shin band (by height fraction, near a trimmed-mean leg axis so capes drop out) whose base
  colour is not skin, the most upright large piece per side, holes filled, decimated. On the Aegis
  scan (an AI-generated Tripo model, CC BY 4.0, evaluated and not shipped) only the left greave
  came out clean; the fit mirrors it.
- `--no-greave-dome`; export now drops or sinks skin that `armour_cover_check.flagged` finds
  outside a greave in any walk/attack/idle/hit frame (sinks 5 mm toward the shin axis, up to six
  passes, then drops what is left). `armour_cover_check.py` checks both LODs and only drawn vertices.
- r9a (the Aegis greaves, an AI-generated (Tripo) CC BY 4.0 source that was evaluated and not
  shipped; it is not a shipped source and is not in the manifest) and r9b (tolnaydavid, no dome): both 0 in the check.

## Round r10 (greaves built from the shin)

`--replace-part greaves=built` grows each greave from the fitted shin (`fit_shells.greave_from_skin`):
an offset sheet (6 mm) of the shin/calf skin from 6 cm above the ankle to just below the knee,
closed except a narrow back seam, smoothed, two raised embossed ridges, rolled top and bottom rims,
and a domed oval knee guard with a median ridge and raised border. No third-party source. Built
greaves are weighted to the shin up to 12 cm above the knee, so the guard rides the shin. Export
sinks skin flagged by `armour_cover_check.flagged` (both its own bins and bins up to the rims,
all five clips, with a 2 mm stricter margin), then drops what is left. New
`kit/greave_shard_check.py` (Blender): ray grids over each shin from six directions, every third
frame of walk and attack, counting rays whose first hit is skin with a greave less than 2 cm
behind it. `--skin-max-value` caps pale skin texels.

## Round r11 (one-piece knee guard, knee holes traced)

- The pale "shards" above the knees were holes in the thigh skin, seen through to the far side
  of the leg. Traced with an ID render built straight from the GLB data (skin pink, garment
  green, greave yellow; `closeups-r11/id-parts-before-after.png`). Two causes: hidden-skin
  removal deleted skin merely *near* clothing (just below the skirt hem), and the export dropped
  skin triangles with any flagged vertex. Now skin counts as hidden only if a ray out along its
  normal hits a covering shell (`covered_along_normal`), and export sinks flagged skin (up to 20
  passes) and drops only triangles whose three vertices are all still flagged.
- `greave_from_skin` now raises the shell's top edge to a point over the kneecap (one piece,
  `rise` 8.5 cm, `dome` 24 mm) with a median ridge, instead of a separate oval cap.

## Round r12 (cover, LOD1 room, repoussé, skin normals)

- Export now alternates: on even passes (up to 16) it pushes greave vertices within 2.5 cm of
  flagged skin outward 1.5 mm (capped at 6 mm per vertex), on the others it sinks flagged skin
  5 mm toward the shin axis; only after 26 passes does it drop triangles whose three vertices are
  all still flagged. `--lod1-greave-offset` (default 3 mm) inflates the LOD1 greave about the shin
  axis so the coarser shell clears the coarser skin.
- Repoussé is baked, not modelled: `greave_from_skin` paints a 512 px height image (a ridge down
  the shin, a pair of scrolls on the knee guard) used as a bump on the high greave and baked into
  the normal and colour maps. Gotcha: set the image's colorspace to Non-Color *before*
  `pixels.foreach_set`; changing it afterwards reloads the buffer and the pattern silently becomes 0.
- Knee guard point raised (`rise` 10.5 cm); the top rim now rolls outward so a high camera sees
  less of the inside.
- `--smooth-skin-normals` transfers custom normals from the undecimated fitted body onto both LOD
  skins, and skin texels get softer baked AO (0.85 + 0.15 AO). The thigh facets above the knee
  are still visible in close-ups: they come from the decimated skin's silhouette and baked
  shading, not only from its normals.

## Round r13 (in progress)

The LOD1 helmet and shield-shard fixes are in progress; this section will be filled in when they land.
