# Unit action panel — 2026-10-25

Placement update: [the vertical side panel and current evidence](side/README.md). The comparison below records the earlier content refinement.

The unit card now reads as a small atlas dossier: a framed painted portrait and compact figures beside a ruled action ledger. Upgrade and Siege share an **Improve** group, each with its benefit, an attached explanation button, and an aligned red cost column. Gold is a payment; strategic resources are labelled as supply the unit uses.

An available upgrade leads, followed by an available Siege improvement. Otherwise the primary is **Move or attack**, which opens explicit map targeting through the existing move command, previews and validation. Scouts and Settlers retain their role actions, and urgent turn-dock decisions retain priority. Fortify and Skip are secondary. Disband has a quiet, separated place in the footer.

Equivalent resource refusals become one wrapping explanation linked to both disabled controls. Other engine refusals remain intact. Full requirements, treasury forecasts and price breakdowns are available from the attached information controls. No engine rules or prices changed.

At 1440×900 the portrait and statistics sit alongside the actions. At 1000×600 the panel stacks vertically and the secondary actions share a footer, with a rule separating Disband. All five requested states fit without scrolling. Aircraft retain their existing specialized card.

## Screenshot comparison

These are full Chromium screenshots from the same real-WASM fixtures. The baseline is `d4cb0e29b7af` with only the new tour fixture added; after images show this branch's redesign. The fresh Hoplite has Sparta's normal starting promotion.

| State | Before 1000×600 | After 1000×600 | Before 1440×900 | After 1440×900 |
| --- | --- | --- | --- | --- |
| Fresh unit | [Before](before/1000x600/fresh.png) | [After](after/1000x600/fresh.png) | [Before](before/1440x900/fresh.png) | [After](after/1440x900/fresh.png) |
| Upgrade available | [Before](before/1000x600/upgrade-available.png) | [After](after/1000x600/upgrade-available.png) | [Before](before/1440x900/upgrade-available.png) | [After](after/1440x900/upgrade-available.png) |
| Resource blocked | [Before](before/1000x600/resource-blocked.png) | [After](after/1000x600/resource-blocked.png) | [Before](before/1440x900/resource-blocked.png) | [After](after/1440x900/resource-blocked.png) |
| Siege available | [Before](before/1000x600/siege-available.png) | [After](after/1000x600/siege-available.png) | [Before](before/1440x900/siege-available.png) | [After](after/1440x900/siege-available.png) |
| Aircraft | [Before](before/1000x600/aircraft.png) | [After](after/1000x600/aircraft.png) | [Before](before/1440x900/aircraft.png) | [After](after/1440x900/aircraft.png) |

## Verification

`app/e2e/tour/unit-panel.spec.ts` captures these five cases at both sizes. It asserts one primary, whole controls, no horizontal or vertical panel overflow, and wrapping reasons; exercises the information buttons, upgrade and Siege orders, movement targeting and Escape, and Fortify/Skip shortcuts. Component and map tests cover shared blockers, retained detailed requirements, costs, promotion effects, and left-click movement through the existing validated command.

Reproduce on the cloud box:

```sh
tools/cloud/remote.sh . --with-uncommitted --npm --artifact app/test-results -- bash -c 'scripts/build-wasm.sh && cd app && npm run build && LEGATUM_E2E_BROWSER=chromium npx playwright test e2e/tour/unit-panel.spec.ts'
```

Set `UNIT_PANEL_BASELINE=1` on the old UI to capture reference shots without asserting the redesigned layout.

Final cloud results:

- `npx tsc --noEmit -p .`: passed.
- `npx vitest run`: 233 files passed; 2,763 tests passed, 2 skipped.
- WASM and production app builds: passed.
- Chromium unit-panel tour: 10/10 passed, including the five states at both sizes.
- Related aircraft, overlay and strategic-supply checks: 19 passed, 3 skipped. The broader run caught the old Fortify-primary assertion in both strategic-supply cases; both passed after updating that assertion to the new hierarchy.
- All 20 committed PNGs have their exact named viewport dimensions. Visual review completed; no unit-panel findings in the final tour layout audits.

Evidence runs: baseline `20261025T034001Z-88ff80`; final unit suite, build and screenshots `20261025T040133Z-9347fb`; corrected strategic-supply tour `20261025T040629Z-6e6e79`. All heavy checks ran on the cloud box, using Chromium only.
