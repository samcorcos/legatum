cd ~/LegatumRuns/anim-pilot/fit-wt && source tools/art/anim3d/env.sh; S=~/LegatumRuns/anim-pilot/premium-source; F=~/LegatumRuns/anim-pilot/fit
HELM=${HELM:-$HOME/LegatumRuns/anim-pilot/premium-source/helmets/pbr_corinthian_helmet.glb:helmet_low}; HELM_CREDIT=${HELM_CREDIT:-"PBR Corinthian Helmet by timothyhsu (Sketchfab), CC BY 4.0"}
mkdir -p tools/art/anim3d/units/spearman-fit-casey-r13b
~/LegatumRuns/anim-pilot/bin/blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/fit_source_to_0ad.py -- \
  --source $S/hoplite-warrior/scene.gltf --out tools/art/anim3d/units/spearman-fit-casey-r13b/spearman.glb --work $F/casey-r13b \
  --credit 'Hoplite Warrior by Casey DeDore (Sketchfab), CC BY 4.0' \
  --part 'drop=Pedestal|Rain|Background|Phalanx|Ground|Sword|^Eye' --part 'spear=^Spear' --part 'shield=^Shield' \
  --shield-glb $F/real/pbr-r4-spearman.glb --crest-mass --crest-tail .9 --crest-shift .03 --helmet-uv-share 2 --hide-skin-lod0 .015 \
  --keep layer --keep shield --keep crest --keep helmet --lod0 29500 --lod1 3450 --grade-bronze --bronze-palette 4a2c0c,915d20,e5b475 --bronze-rough .38 --bronze-detail .9 --bronze-ao .3 --grade-skin b07249 \
  --sleeves .45 --strip forearm_L --press-skin --knee-smooth --no-greave-dome --jpeg-quality 72 \
  --replace-part helmet=$HELM --replace-credit "helmet=$HELM_CREDIT" --replace-license helmet=CC-BY-4.0 \
  --replace-part greaves=built --smooth-skin-normals --skin-max-value 1.12 --lod1-helmet-face .5 .45 --lod1-helmet-face-dissolve 10 --lod1-part-tris layer=1000 --lod1-part-tris greave=462 --lod1-part-tris spear=60 --lod1-part-tris crest=60 --lod1-feet-shell 70 --lod1-sleeve-tris 120 \
  "$@" > $F/logs/casey-r13b.log 2>&1
grep -E "^FIT|Error|Trace|SystemExit" $F/logs/casey-r13b.log | grep -E "replaced|crest mass|lods|wrote|Error|Trace|File|Exit" | cut -c1-500
