cd ~/LegatumRuns/anim-pilot/fit-wt && source tools/art/anim3d/env.sh; S=~/LegatumRuns/anim-pilot/premium-source; F=~/LegatumRuns/anim-pilot/fit
HELM=${HELM:-$HOME/LegatumRuns/anim-pilot/premium-source/helmets/pbr_corinthian_helmet.glb:helmet_low}; HELM_CREDIT=${HELM_CREDIT:-"PBR Corinthian Helmet by timothyhsu (Sketchfab), CC BY 4.0"}
mkdir -p tools/art/anim3d/units/spearman-fit-casey-r7
~/LegatumRuns/anim-pilot/bin/blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/fit_source_to_0ad.py -- \
  --source $S/hoplite-warrior/scene.gltf --out tools/art/anim3d/units/spearman-fit-casey-r7/spearman.glb --work $F/casey-r7 \
  --credit 'Hoplite Warrior by Casey DeDore (Sketchfab), CC BY 4.0' \
  --part 'drop=Pedestal|Rain|Background|Phalanx|Ground|Sword|^Eye' --part 'spear=^Spear' --part 'shield=^Shield' \
  --shield-glb $F/real/pbr-r4-spearman.glb --crest-mass --crest-tail .95 --hide-skin-lod0 .015 \
  --keep layer --keep shield --keep crest --keep helmet --lod0 29500 --lod1 2900 --grade-bronze --grade-skin b07249 \
  --sleeves .45 --strip forearm_L --press-skin --knee-smooth --jpeg-quality 72 \
  --replace-part helmet=$HELM --replace-credit "helmet=$HELM_CREDIT" --replace-license helmet=CC-BY-4.0 \
  --replace-part greaves=$S/hoplite-tolnay/scene.gltf:Leg_armor_low --replace-credit 'greaves=Hoplite by tolnaydavid (Sketchfab), CC BY 4.0' --replace-license greaves=CC-BY-4.0 \
  "$@" > $F/logs/casey-r7.log 2>&1
grep -E "^FIT|Error|Trace|SystemExit" $F/logs/casey-r7.log | grep -E "replaced|crest mass|lods|wrote|Error|Trace|File|Exit" | cut -c1-500
