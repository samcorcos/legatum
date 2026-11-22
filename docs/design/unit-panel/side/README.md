# Unit panel on the left

The selected unit now shares the city's left dock: 320 px wide at 1000×600 and 352 px at 1440×900, starting at the same top edge. Its painted portrait, name and promotion sit in a ruled header, followed by compact stats, the primary action and Improve ledger, Fortify / Skip, and a separate quiet Disband footer. Aircraft use the same dock with their specialized actions. No unit strip occupies the bottom of the map.

The lens control folds while a unit or city is selected. Its overlay checkboxes and lens choices remain available from the bottom-left toggle. Camera clearance uses the measured HUD bounds and leaves room around the selected tile for adjacent destinations; clicking a unit underneath the new panel nudges it into view. Explicit map navigation cancels an outstanding layout nudge. Long exceptional details still scroll inside the panel with a visible hint.

The five requested states are captured from real WASM fixtures in Chromium on the cloud box. Before images use `62ebdd6a2` (current master plus the earlier content refinement). The fresh Hoplite has Sparta's normal starting promotion.

| State | Before 1000×600 | After 1000×600 | Before 1440×900 | After 1440×900 |
| --- | --- | --- | --- | --- |
| Fresh | [Before](before/1000x600/fresh.png) | [After](after/1000x600/fresh.png) | [Before](before/1440x900/fresh.png) | [After](after/1440x900/fresh.png) |
| Upgrade available | [Before](before/1000x600/upgrade-available.png) | [After](after/1000x600/upgrade-available.png) | [Before](before/1440x900/upgrade-available.png) | [After](after/1440x900/upgrade-available.png) |
| Resource blocked | [Before](before/1000x600/resource-blocked.png) | [After](after/1000x600/resource-blocked.png) | [Before](before/1440x900/resource-blocked.png) | [After](after/1440x900/resource-blocked.png) |
| Siege available | [Before](before/1000x600/siege-available.png) | [After](after/1000x600/siege-available.png) | [Before](before/1440x900/siege-available.png) | [After](after/1440x900/siege-available.png) |
| Aircraft | [Before](before/1000x600/aircraft.png) | [After](after/1000x600/aircraft.png) | [Before](before/1440x900/aircraft.png) | [After](after/1440x900/aircraft.png) |

The tour checks matching city-panel placement, whole buttons and stats, one primary, shared wrapping reasons, adjacent map tiles clear of the HUD, camera recovery when a selection opens over its unit, and the compact lens chooser. It exercises upgrade and Siege orders, move targeting, Fortify and Skip shortcuts, role-specific units and call-to-arms priority.

```sh
tools/cloud/remote.sh . --with-uncommitted --npm --artifact app/test-results -- bash -c 'scripts/build-wasm.sh && cd app && npm run build && LEGATUM_E2E_BROWSER=chromium npx playwright test e2e/tour/unit-panel.spec.ts'
```

Baseline cloud run: `20261025T134305Z-ad0493` (10 passed).


## Final checks

All heavy checks ran on the cloud box, using Chromium only.

- `npx tsc --noEmit -p .`: passed.
- `npx vitest run`: 233 files passed; 2,763 tests passed, 2 skipped.
- WASM and production app build: passed.
- Focused browser coverage: 31 distinct cases passed, 5 skipped. The 36-case run had 26 passes, 5 old layout-assumption failures and 5 skips; after fixing those assertions, all 6 affected cases passed (including both fresh-unit sizes). No application code changed after the full checks.
- The five skipped browser cases are four aircraft checks requiring external evidence saves and one duplicate shortcut check restricted to the laptop viewport.
- All 20 PNGs have their exact named dimensions. The ten final unit-panel screenshot audits have no unit-panel findings. The adjacent-tile and HUD-overlap assertions pass at both sizes; role-specific and call-to-arms states also pass.
- Master rechecked before committing: already included, `11a70e2ba`.

Full check run: `20261025T135310Z-423e32`. Corrected lens, first-turn and camera-clearance assertions: `20261025T135743Z-a62b89` (6/6 passed). Final images come from those two runs. Earlier visual/state iteration: `20261025T135236Z-c163da`.

A camera nudge can legitimately find the nearest clear space below the side panel at a tall viewport. Its test checks clearance around the unit rather than requiring a horizontal displacement. Lens tests open the compact chooser to inspect its choices and close it before checking the panel's hit targets.
