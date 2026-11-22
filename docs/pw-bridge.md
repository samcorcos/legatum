# Playwright bridge

The lead starts the host daemon from any checkout:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
mkdir -p .progress/pw-bridge
nohup tools/ui/pw-bridge.sh > .progress/pw-bridge/daemon.log 2>&1 &
```

The daemon holds a lock so a second copy exits. It handles requests oldest first, one at a time. Each run has a 30-minute limit and runs at nice 15.

From a Codex task, use:

```sh
tools/ui/pw-request.sh "$PWD" app/e2e/layout.spec.ts
tools/ui/pw-request.sh "$PWD" app/e2e/layout.spec.ts -g 'title Settings' --project game
```

The client polls every 15 seconds for up to 45 minutes, prints the summary, and exits nonzero for a failed test, setup error, or timeout. It prints the result directory, where `summary.json`, `output.log`, `playwright.json`, and copied `shots/` screenshots live. Requests, completed requests, and results are under the main checkout's `.progress/pw-bridge/`. Only the main checkout and paths under `.claude/worktrees/` are accepted; spec paths must name existing `app/e2e/*.spec.ts` files. The daemon creates a missing `app/node_modules` symlink, refreshes stale WASM, and builds the debug server and app for the staging spec. The Vite config must allow the real `node_modules` path when it is symlinked, or fonts may return 403 in browser tests.

For task prompts: after changing the UI, run `tools/ui/pw-request.sh "$PWD" app/e2e/<relevant>.spec.ts` (add `-g 'test name'` for a focused run). Read the printed `summary.json` and `output.log`; inspect PNGs in the printed `shots/` directory before reporting the UI as verified. The host daemon must already be running.
