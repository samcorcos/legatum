# An e2e autosave readback can race the turn number

2026-11-27. `e2e/turn-flow.spec.ts` "Fortify heals while the unit rests" failed now and then in hosted CI with HP 95
where it expected 100, on branches that did not touch it and then on master, and passed on a re-run. The test waited
for the turn number to read 4 and then read the newest autosave (`e2e/autosave.ts::newestGameState`). The autosave is
written to the browser database after the turn number changes, so a fast read returned turn 3's save, one heal short.

Rule: a test that reads the autosave after ending a turn first waits for the save of that turn
(`await expect.poll(async () => (await newestAutosave(page))?.turn).toBe(n)`). The turn number on screen is not the
signal that the save exists.
