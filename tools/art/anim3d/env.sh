#!/usr/bin/env bash
# Source from the entry points; no system or repository installs.
ANIM_ROOT="${ANIM_ROOT:-$HOME/LegatumRuns/anim-pilot}"
export BLENDER_USER_RESOURCES="$ANIM_ROOT/blender-user"
export BLENDER_USER_CONFIG="$BLENDER_USER_RESOURCES/config"
export BLENDER_USER_SCRIPTS="$BLENDER_USER_RESOURCES/scripts"
export BLENDER_USER_DATAFILES="$BLENDER_USER_RESOURCES/datafiles"
export XDG_CACHE_HOME="$BLENDER_USER_RESOURCES/cache"
export PYTHONDONTWRITEBYTECODE=1
export TMPDIR="$ANIM_ROOT/tmp"
export ANIM_ROOT
MASTER="$ANIM_ROOT/masters/spearman-3d-v2"
REVIEW="$ANIM_ROOT/review/3d-v2"
mkdir -p "$MASTER" "$REVIEW" "$BLENDER_USER_CONFIG" "$XDG_CACHE_HOME" "$TMPDIR"
