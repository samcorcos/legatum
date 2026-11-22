# ZDOTDIR for Codex lanes (tools/astra/runner.sh sets ZDOTDIR to this directory). Codex runs every command through
# `/bin/zsh -lc`, and a macOS login shell reorders PATH (path_helper, `brew shellenv`), which would put the real
# npx and npm ahead of the shims in tools/astra/lane-bin. So: load the user's own files, then put the lane bin first.
[ -f "$HOME/.zshenv" ] && . "$HOME/.zshenv"
