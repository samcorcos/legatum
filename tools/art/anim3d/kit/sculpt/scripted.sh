#!/bin/bash
# Scripted sculpt on fresh copies, then rebuild to a separate output (r5 GLB untouched).
set -e
R5=~/LegatumRuns/anim-pilot/fit/r5; cd ~/LegatumRuns/anim-pilot/fit-wt; source tools/art/anim3d/env.sh; B=~/LegatumRuns/anim-pilot/bin/blender-slot
cp $R5/helmet-crest-sculpt.blend $R5/helmet-crest-sculpt-scripted.blend; cp $R5/greaves-sculpt.blend $R5/greaves-sculpt-scripted.blend
$B --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/scripted_sculpt.py -- $R5/helmet-crest-sculpt-scripted.blend helmet $HELMET_ARGS 2>&1 | grep -E "SCULPTED|Error|Trace"
$B --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/scripted_sculpt.py -- $R5/greaves-sculpt-scripted.blend greaves $GREAVE_ARGS 2>&1 | grep -E "SCULPTED|Error|Trace"
sed -e "s|helmet-crest-sculpt.blend|helmet-crest-sculpt-scripted.blend|; s|greaves-sculpt.blend|greaves-sculpt-scripted.blend|; s|units/spearman-fit-casey/spearman.glb|units/spearman-fit-casey-sculpted/spearman.glb|g; s|casey-r6|casey-r6-scripted|g" $R5/rebuild-with-sculpts.sh > $R5/rebuild-scripted.sh
bash $R5/rebuild-scripted.sh
