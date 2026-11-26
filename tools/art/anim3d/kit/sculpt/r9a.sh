cd ~/LegatumRuns/anim-pilot/fit-wt && source tools/art/anim3d/env.sh; S=~/LegatumRuns/anim-pilot/premium-source; F=~/LegatumRuns/anim-pilot/fit
HELM=${HELM:-$HOME/LegatumRuns/anim-pilot/premium-source/helmets/pbr_corinthian_helmet.glb:helmet_low}; HELM_CREDIT=${HELM_CREDIT:-"PBR Corinthian Helmet by timothyhsu (Sketchfab), CC BY 4.0"}
mkdir -p tools/art/anim3d/units/spearman-fit-casey-r9a
~/LegatumRuns/anim-pilot/bin/blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/fit_source_to_0ad.py -- \
  --source $S/hoplite-warrior/scene.gltf --out tools/art/anim3d/units/spearman-fit-casey-r9a/spearman.glb --work $F/casey-r9a \
  --credit 'Hoplite Warrior by Casey DeDore (Sketchfab), CC BY 4.0' \
  --part 'drop=Pedestal|Rain|Background|Phalanx|Ground|Sword|^Eye' --part 'spear=^Spear' --part 'shield=^Shield' \
  --shield-glb $F/real/pbr-r4-spearman.glb --crest-mass --crest-tail .9 --crest-shift .03 --helmet-uv-share 2 --hide-skin-lod0 .015 \
  --keep layer --keep shield --keep crest --keep helmet --lod0 29500 --lod1 2900 --grade-bronze --bronze-palette 4a2c0c,915d20,e5b475 --bronze-rough .38 --bronze-detail .9 --bronze-ao .3 --grade-skin b07249 \
  --sleeves .45 --strip forearm_L --press-skin --knee-smooth --no-greave-dome --jpeg-quality 72 \
  --replace-part helmet=$HELM --replace-credit "helmet=$HELM_CREDIT" --replace-license helmet=CC-BY-4.0 \
  --replace-part greaves=$F/r9/aegis-greaves.glb --replace-credit 'greaves=Aegis Sentinel - Ancient Blue Hoplite Warrior V1 by pixforge.id (Sketchfab), AI-generated (Tripo) source, CC BY 4.0' --replace-license greaves=CC-BY-4.0 \
  "$@" > $F/logs/casey-r9a.log 2>&1
grep -E "^FIT|Error|Trace|SystemExit" $F/logs/casey-r9a.log | grep -E "replaced|crest mass|lods|wrote|Error|Trace|File|Exit" | cut -c1-500
