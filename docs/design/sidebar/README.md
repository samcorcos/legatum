# Right sidebar design review

One column holds the race, this turn's action, and the minimap. The soonest hold countdown is the headline. With no hold, the headline is your standing. Coalition and desertion share a disclosure; How You Win shows one objective and opens the full score ledger. Three crown medallions show their holders, with the existing rule cards on hover or keyboard focus. The faction goal is one line, with its rule and action behind that line.

The top bar now opens the score ledger without repeating the points and countdown. Standings, other holds, unclaimed points, and diplomacy sit behind All empires. City alarms, orders, and turn reviews sit behind Turn details. The existing next-decision logic still determines the primary button.

The column has one capped scroll area. Its headline sticks while the details scroll; the next action, Ready anyway, and minimap stay outside that scroll area. There is no height-fitting observer or nested standings scroll. The left selection area is reserved for the separate unit/city panel work.

## Screenshot pairs

| State | 1000 × 600 | 1440 × 900 |
| --- | --- | --- |
| Early, turn 20 | [Before](before/early-1000x600.png) · [After](after/early-1000x600.png) | [Before](before/early-1440x900.png) · [After](after/early-1440x900.png) |
| Mid, turn 117 | [Before](before/mid-1000x600.png) · [After](after/mid-1000x600.png) | [Before](before/mid-1440x900.png) · [After](after/mid-1440x900.png) |
| Late, you hold 10 VP; coalition and desertion | [Before](before/holding-1000x600.png) · [After](after/holding-1000x600.png) | [Before](before/holding-1440x900.png) · [After](after/holding-1440x900.png) |
| Late, a rival holds 10 VP | [Before](before/rival-1000x600.png) · [After](after/rival-1000x600.png) | [Before](before/rival-1440x900.png) · [After](after/rival-1440x900.png) |

[Expanded states](expanded/) capture the column with disclosures open after the wheel and keyboard reachability checks, with the headline and action still visible. The outline shows keyboard focus on the scroll region.

## Provenance and reproduction

The before UI is commit `11a70e2baa74`. Before and after use the same Chromium screenshot spec, `app/e2e/tour/sidebar.spec.ts`, fonts, dimensions, and view staging. Early and mid use archived saves at turns 20 and 117. The late states start from the turn-203 archive, then stage turn 199, Future-era objectives, a consistent 10-point holder, coalition, desertion, retirement progress, and last-turn report counts in the rendered view. These are layout scenarios, not evidence of a simulated turn or a rules change. Selection and migration notices are cleared for the comparison.

```sh
tools/cloud/remote.sh . --with-uncommitted --npm --artifact app/test-results -- bash -c 'scripts/build-wasm.sh && cd app && npm run build && LEGATUM_E2E_BROWSER=chromium npx playwright test e2e/tour/sidebar.spec.ts e2e/race-panel-late.spec.ts e2e/era-hud-compact.spec.ts e2e/era-objectives.spec.ts e2e/tour/victory-race.spec.ts'
```

For before images, restore the eight changed UI/map TSX files from `11a70e2baa74` **in the disposable cloud checkout only**, then build and run the sidebar spec with `SIDEBAR_PHASE=before`. Do not revert the working branch. Both capture passes disable Chromium’s headless `--hide-scrollbars` default; the UI itself uses a visible, styled scrollbar. Screenshots return under the cloud job's `artifacts/app/test-results/sidebar/` directory.

The sidebar assertions check unclipped default priorities, horizontal text overflow, every disclosure's final control, wheel scrolling and a visible scrollbar, keyboard access, sticky headline, and the pinned action. Related tours cover title changes, crown cards, eliminated empires, the ledger, and research objectives.

Designer approval is pending; this branch is not merged into master.

## Validation receipts

All browser runs used Chromium on the cloud box.

| Check | Result | Cloud job |
| --- | --- | --- |
| `npx tsc --noEmit -p .` | Passed | `20261025T152725Z-b5f84e` |
| Full `npx vitest run` | 233 files passed; 2,755 tests passed, 2 skipped | `20261025T152725Z-b5f84e` |
| Related race/objective tours | 34 passed | `20261025T152352Z-63319a` |
| Broader affected browser specs | 159 passed; 13 old navigation assumptions needed updates | `20261025T153702Z-b7f265` |
| Focused browser rerun | All 13 corrected cases passed; 59 of 60 total passed, with one guide/menu navigation assertion then corrected | `20261025T154446Z-d01887` |
| Focused unit check after preserving VP effect anchors | 57 passed across 3 files | `20261025T154446Z-d01887` |
| Before capture matrix | 8 passed | `20261025T155004Z-dfdd53` |
| Final sidebar, guide and siege browser checks | 18 passed, including all 8 after/expanded captures | `20261025T154946Z-1e5795` |

The full suite ran once. Subsequent code refinements were checked with the affected tests and the production build. Master `86c627fb2` was merged into this branch without conflicts after those checks; it contained no app changes. The screenshots show the reviewed UI from `9ec4a9b28`. Only determinism and pin checks ran after that merge, as requested: **15 passed**, with 1,940 unrelated native tests filtered out (`20261025T155528Z-12011b`). The initial `cargo test` attempt was rejected by the cloud wrapper's profile translation before executing tests; the documented Nextest command below passed with the same optimized profile, overflow checks and debug assertions enabled.

```sh
tools/cloud/remote.sh . --with-uncommitted -- bash -c 'tools/sim/build-slot.sh -- cargo nextest run --workspace --cargo-profile release-checked -E "binary(=determinism) or test(pinned)"'
```
