# Check results

- Requested Vitest command: **PASS**, 4 files, 16 tests; 1.56s test duration. Cloud upload timed out before execution; automatic local fallback ran the tests.
- Wall loader/construction Vitest: **PASS**, 2 files, 12 tests; 1.10s. Same automatic fallback. The existing loader expectation was updated for the four new wall families.
- PNG validation: **PASS**, 372 RGBA files decoded at expected dimensions. See `asset-validation.txt`.
- `git diff --check`: **PASS**.
- Fresh distinctness check: **FAIL, exit 1**, 19 sets, 604 renders, 9,750 comparisons, 114 flagged, 49 accepted, 54 advisory, **11 unaccepted unit pairs**. Eight are gist-only; three cross the colour threshold. See `distinctness-lead.md` for inspected pairs and proposed entries. The cloud retry rendered every image freshly; no stale cache was used.
- Master merged cleanly at `327fad56766f`, incorporating `a76c7a592` (two documentation files; no conflict).
- Post-merge pin filter: **PASS, exit 0**, 5 tests (including both simulator tape pins), no failures.
- Post-merge determinism: **PASS, exit 0**, 12 tests, no failures. The box routed the Cargo command to nextest; summary 12 passed, 0 skipped.

The initial cloud uploads timed out before starting. Vitest used its automatic local fallback. Local Chromium was then blocked by the sandbox; the successful committed cloud retry was job `20261117T022430Z-a39c4f` at `327fad56766f`. That job's overall exit 1 is the distinctness result; its pin and determinism commands both returned 0. `postmerge-status.txt` records all three statuses.

Commands:

```sh
cd app && npx vitest run src/unlockArt.test.ts src/portraits.test.ts src/factionSheets.test.ts src/ui/factionSheetCards.test.tsx
cd app && npx vitest run src/map/cityArt.test.ts src/map/constructionFx.test.ts
python3 tools/art/distinctness.py --render --check --report docs/design/art-pass-3/distinctness.md --sheet docs/design/art-pass-3/distinctness-pairs.png
tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked -- pinned
tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked --test determinism
```

The last three commands ran on the cloud box via `postmerge-checks.sh`. The optimized checked profile retains overflow checks and debug assertions. No pins needed regeneration. No acceptance-file edit, no push, and no merge into master.
