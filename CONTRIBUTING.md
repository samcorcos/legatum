# Contributing to Legatum

Thanks for looking. Legatum is a historical 4X on a hex map: a deterministic Rust engine, scripted AI,
a headless simulation runner, and a React/TypeScript UI that runs the same engine as WASM in the browser,
including the host of an online game.

Anyone may open an issue or a pull request. The maintainer, Sam Corcos, decides what merges. The project
is pre-release and changes quickly, so for anything larger than a fix, open an issue first and say what you
have in mind.

Issues and pull requests are for the game. Ones that are clearly about something else (politics, the
maintainer's job, spam, joke edits) are closed with an `off-topic` label, and ones like that in comments are
hidden as off-topic, some of them automatically by [a check](.github/workflows/moderate.yml) when they are
posted. If yours was closed by mistake, say so in
a comment and it will be reopened. See the [Code of Conduct](CODE_OF_CONDUCT.md).

## Where to start reading

- [README.md](README.md): what the game is and how to run it.
- [ARCHITECTURE.md](ARCHITECTURE.md): the crates, the turn pipeline and the determinism rules.
- [The rules reference](.claude/skills/legatum-rules/references/index.md) (one file per system): how the game works and
  which systems do not exist. It is the engineering reference: long, dated and checked against the code.
  How to Play in the app is the short version.
- [docs/README.md](docs/README.md): an index of everything else.

Much of this project is written by AI coding agents. `CLAUDE.md`, `AGENTS.md` and `.claude/` are their
instructions and notes. You do not need to use an agent or follow those files to contribute, but they
explain why things are the way they are.

## Set up

You need:

- [rustup](https://rustup.rs/) (the checked-in toolchain file installs Rust 1.98.1, rustfmt, clippy and the WASM target)
- [`wasm-pack`](https://wasm-bindgen.github.io/wasm-pack/installer/) 0.15.0
- Node.js 22 (see `.nvmrc`) and npm
- Python 3 (`python3`): `scripts/build-wasm.sh` and the art test run small Python tools. On a Mac it comes
  with the Xcode command line tools.
- For the app tests: Playwright's Chromium, installed once with `cd app && npx playwright install chromium`

```sh
git clone https://github.com/samcorcos/legatum && cd legatum
nvm use                                # or install Node.js 22 by another method
scripts/build-wasm.sh                  # the engine as WASM -> app/src/wasm-pkg (git-ignored)
cd app && npm ci && npm run dev        # the game at http://localhost:5173
```

Other entry points:

```sh
cargo run --release -p legatum-sim -- --help    # headless simulation runner
scripts/build-web.sh                            # the hosted web build, as deployed (static files in app/dist)
```

Releases (the web deploy, the secrets it needs) are described in
[docs/ops/releasing.md](docs/ops/releasing.md).

### Art masters (only if you change art)

The full-size art masters are not in git; a build and the tests never need them. To work on a unit source, a people painting or the title key art, fetch it by path:

```sh
export LEGATUM_ART_SOURCES_URL=https://pub-fb5c281dd12d4ea4bbd38d657cdeb907.r2.dev
python3 tools/art/fetch-sources.py app/src/assets/art/people-athens.png
```

`tools/art/sources-manifest.json` lists every master with its size and sha256 (a master made by someone else also carries `source` and `license` fields, which the fetch tool ignores). The `docs/design` and `docs/playtests` entries are kept in a private archive, so `--all` reports those as missing.

## Test

### Quick checks

For a small change these are enough to start with. CI runs the full suite on every pull request, so you do not have to run it all yourself.

```sh
python3 tools/gates/check.py HEAD --base origin/master --checks-only   # the static gates, no build needed
cargo fmt --all --check                                    # formatting
cargo test -p <crate> <name>                               # only the tests you touched, e.g. -p legatum-core
cd app && npm ci && npm test -- <file>                     # UI-only changes: one vitest file
```

`npm test` for the app needs the WASM package, so run `scripts/build-wasm.sh` once first (see Set up above) if the test imports the engine.

### Full run

Run what your change touches, and everything before you open a pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release                # engine, AI, sim, WASM bindings and online host
scripts/build-wasm.sh && cd app && npm ci && npx playwright install chromium && npm test && npm run build
```

`cargo test --workspace --release` also compiles every example program (over a hundred, mostly in
`crates/sim/examples`), which takes a long time on a cold build. While iterating,
`cargo test --workspace --release --lib --bins --tests` runs the same tests without building them.

These are the commands CI runs (`.github/workflows/ci.yml`). Browser end-to-end tests (Playwright) are
described in [app/README.md](app/README.md#browser-end-to-end-tests). CI runs a smoke subset on every pull request (the `e2e-smoke` job); the full suite runs nightly on `master`.

Positions in `scenarios/` are AI measurement fixtures, including the stable `scenarios/lab/` cuts from
recorded games. They are not a player menu. See [scenarios/README.md](scenarios/README.md) for cutting,
checking and measuring a position.

## What a pull request needs

**CI is the bar for an outside pull request.** If the checks above pass and the change is focused, it is
ready for review.

`CLAUDE.md` describes further gates before anything lands on `master`: `tools/gates/check.py`, an AI
documentation review and, for UI changes, an AI UI review and, for art changes, an AI art review. The maintainer runs those himself on your
branch. You do not need to run them or post their verdicts, and a pull request is not held up for
lacking them. They may lead to follow-up commits or requests.

- Keep a change focused, with tests that fail before and pass after.
- Describe what changed for a player and what you ran. Include screenshots for UI changes (at 1000x600 and
  1440x900, the sizes the layout is tuned for).
- If the change makes a line in `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`, `docs/README.md` or the
  in-app How to Play untrue, fix that line in the same pull request.
- Art and audio: add the prompt or source and its licence next to the file (see `LICENSE-ART` and
  `app/public/audio/CREDITS.md`). Third-party assets need a licence that allows redistribution.

## The rules that matter

1. **One engine.** All game rules live in `crates/core`. The UI, the online host and the simulation runner call
   it; nothing re-implements a rule in TypeScript or Python. If the UI needs a number, the engine
   computes it.
2. **Determinism.** Integer maths only, seeded RNG streams, no dependence on hash-map iteration order.
   The same state, orders and seed must give the same next state on every platform; tests compare state
   hashes across native and WASM.
3. **Pinned tapes.** Recorded game tapes and their hashes live in `crates/sim/tests/` (`pins.json` and
   `fixtures/`). A rule change that is meant to change outcomes also changes the pins: regenerate them
   with `tools/sim/repin.sh`, review the diff, and say in the pull request why games changed.
4. **Saves carry a rules version.** Any change to the rules or to the serialized state bumps
   `legatum_core::RULES_VERSION` (`crates/core/src/lib.rs`). A save from another version is refused with
   a clear message; there are no migrations. The hosted site keeps each earlier build for 30 days at
   `v/<id>/` (`docs/ops/releasing.md`), so a game opens in the build with its rules. The one exception to the
   bump is optional client telemetry in the archive (`frame_times`, `turn_times`): a default-empty field that no
   state hash reads and that old saves lack can be added without a bump. Current-version fixtures in
   `crates/wasm/tests/fixtures/saves/` guard the round trip.
5. **No hidden maths, no AI bonuses.** A number shown to a player should open into a breakdown. An AI
   seat at Easy, Normal or Hard plans from its own `PlayerView` and plays under the same rules as a
   human: a level may change how it decides, never its yields, costs, combat strength or sight.
6. **Delete, don't comment out.** Remove dead code and old experiments; git has the history.
7. **Rules docs follow code.** If you change a rule or mechanic, update the matching file under `.claude/skills/legatum-rules/references/` (`index.md` lists them) in the same change.
8. **Branches cut before the 2026-11-22 split.** If your branch conflicts on `docs/learnings.md` or `.claude/skills/legatum-rules/references/game-rules.md`, take master's version of that file (`git checkout origin/master -- <file>`), then either re-add your entry by hand (a learning goes in its own new file, `docs/learnings/<YYYY-MM>/<DD>-<slug>.md`; a rule goes in the matching per-system file) or carry your learnings over with `python3 tools/docs/split_learnings.py <your branch> --base $(git merge-base origin/master <your branch>) --write`. The hygiene gate fails while `docs/learnings.md` is over 30 KB or the `game-rules.md` stub over 5 KB.

## Licence of contributions

By contributing you agree that your contribution is licensed as the repository is: images and video under
CC BY 4.0 ([LICENSE-ART](LICENSE-ART)) and everything else under the MIT licence ([LICENSE](LICENSE)).

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for how we work together and [SECURITY.md](SECURITY.md) to report
a vulnerability.
