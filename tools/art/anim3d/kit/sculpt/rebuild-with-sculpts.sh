#!/bin/bash
# Rebuild spearman-fit-casey (r5 settings) with the hand-sculpted helmet/crest and greaves.
set -e
cd ~/LegatumRuns/anim-pilot/fit-wt && source tools/art/anim3d/env.sh
S=~/LegatumRuns/anim-pilot/premium-source; F=~/LegatumRuns/anim-pilot/fit; R5=$F/r5
~/LegatumRuns/anim-pilot/bin/blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/fit_source_to_0ad.py -- \
  --source $S/hoplite-warrior/scene.gltf --out tools/art/anim3d/units/spearman-fit-casey/spearman.glb --work $F/casey-r6 \
  --credit 'Hoplite Warrior by Casey DeDore (Sketchfab), CC BY 4.0' \
  --part 'drop=Pedestal|Rain|Background|Phalanx|Ground|Sword|^Eye' --part 'spear=^Spear' --part 'shield=^Shield' \
  --shield-glb $F/real/pbr-r4-spearman.glb --crest-glb $F/real/painted-r4-spearman.glb \
  --keep layer --keep shield --lod0 29500 --lod1 2900 --grade-bronze --grade-skin b07249 \
  --sleeves .45 --strip forearm_L --corinthian --press-skin --crest-width 1.9 --knee-smooth --jpeg-quality 72 $EXTRA_FIT \
  --sculpt-override helmet=$R5/helmet-crest-sculpt.blend:LOD0-helmet \
  --sculpt-override layer=$R5/greaves-sculpt.blend:LOD0-layer 2>&1 | tee $F/logs/casey-r6.log | grep -E "sculpt override|wrote|Error|SystemExit"
python3 tools/art/anim3d/validate_glb.py tools/art/anim3d/units/spearman-fit-casey/spearman.glb --lod0-max 30000 --lod-total-max 35000 --max-bytes 3000000 --max-joints 64 | python3 -c "import json,sys;r=json.load(sys.stdin);print('VALID',r['valid'],r['errors'],r['lod_triangles'])"
GLTF_VALIDATOR_MODULE=$HOME/LegatumRuns/anim-pilot/oad/validator/node_modules/gltf-validator node tools/art/anim3d/viewer/validate-khronos.mjs tools/art/anim3d/units/spearman-fit-casey/spearman.glb $F/casey-r6/khronos.json | head -3
