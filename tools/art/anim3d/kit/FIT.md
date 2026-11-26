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
`tools/art/anim3d/sources/...` paths; only the Hoplite Warrior and the 0 A.D. hoplite actor are listed, the second r1 source, `hoplite-tolnay`, is not). `--rig` defaults to that GLB. The three.js review
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
