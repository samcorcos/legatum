# Browser specs: fixture saves need one human, and drift outside the smoke set goes unseen

- A save with more than one human player now continues only as an online host (store.ts `needsOnlineHost`), so every
  spec that injected a two-human fixture save into the autosave slot and pressed Continue landed on "Host this save"
  and timed out. `e2e/fixtures.ts` (`soloAutosaves`, fixture option `saves`) now rewrites such a save as it is written:
  the first human keeps its seat, the others become AI empires.
- Never `JSON.parse`/`JSON.stringify` a fixture game string in the browser: its explored bitsets are u64 words that a
  JavaScript number rounds, and the engine then reports the save as damaged. Flip flags in the text instead.
- Pull requests run only four smoke specs; the nightly full suite had failed at collection for days without anyone
  reading it. `tools/gates/e2e-strings.py` (run by `check.py` on every app/src change) flags spec lines that still
  expect a string the branch removes from app/src. It misses a string that is reworded in one place but still used
  elsewhere in the app.
- In the cloud container the repo's Playwright is newer than the preinstalled browser: run with a local config that
  sets `launchOptions.executablePath: '/opt/pw-browsers/chromium'`, and one shared relay via `LEGATUM_E2E_RELAY` so
  several workers can run.
