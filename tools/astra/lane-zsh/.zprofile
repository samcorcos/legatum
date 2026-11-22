[ -f "$HOME/.zprofile" ] && . "$HOME/.zprofile"
if [ -n "${LEGATUM_LANE_BIN:-}" ]; then
  PATH="$LEGATUM_LANE_BIN:${PATH//$LEGATUM_LANE_BIN:/}"
  export PATH
fi
