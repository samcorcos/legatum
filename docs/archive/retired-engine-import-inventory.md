> Archived 2026-11-22: kept for history; it no longer describes the project.

# TypeScript engine retirement inventory

This is the complete import inventory from `master` at the start of the retirement. A path appears once, even if it imported several engine modules. All listed imports have been removed. `app/src/mock/` itself contained 86 files: 35 implementation and fixture support files, 36 parity tests and 15 direct unit tests.

All 36 parity test files moved to `app/src/test/` and exercise WASM. Four direct unit test files moved to core-backed tests; the remaining 11 direct unit suites executed only the retired TypeScript implementation and were deleted. Rust and migrated WASM tests exercise the corresponding rules. Five assertion sites from the ruin parity suite could not be carried over literally because they called private functions that were deleted:

- Numeric `ruinRoll` output for five seeds. The core's pinned ruin test checks reward outcomes instead.
- `exploreRuin` population amount and rejection of a second exploration. The core checks population 3 and single consumption.
- `exploreRuin` Gold fallback and no extra Scout when every spawn tile is blocked. The core checks this fallback, and the retained WASM test checks the turn event and unit count.

## Runtime game client and app paths (11)

The client selection, planner, world backdrop and rule helpers now use Rust WASM or core data.

- `app/src/aftermath.ts`
- `app/src/automation.ts`
- `app/src/client.ts`
- `app/src/commands.ts`
- `app/src/format.ts`
- `app/src/map/MapCanvas.tsx`
- `app/src/map/WorldBackdrop.tsx`
- `app/src/map/lenses.ts`
- `app/src/resources.ts`
- `app/src/ui/TileCard.tsx`
- `app/src/ui/UnitPanel.tsx`

## Playwright fixture builders (9)

These build or inspect core-backed fixture saves; no browser game uses the retired client.

- `app/e2e/air-wing.spec.ts`
- `app/e2e/art-distinct.spec.ts`
- `app/e2e/lenses.spec.ts`
- `app/e2e/media/capture.media.ts`
- `app/e2e/settle-lens.spec.ts`
- `app/e2e/terrain-los.spec.ts`
- `app/e2e/terrain-units.spec.ts`
- `app/e2e/territory.spec.ts`
- `app/e2e/tour/era-resources.spec.ts`

## Reusable test fixture builders (44)

These now generate a core save or arrange fields in one, then load it through `WasmClient`.

- `app/src/test/alarmFixture.ts`
- `app/src/test/allianceFixture.ts`
- `app/src/test/arenaView.ts`
- `app/src/test/assaultFixture.ts`
- `app/src/test/bordersFixture.ts`
- `app/src/test/captureOrdersFixture.ts`
- `app/src/test/celebrationFixture.ts`
- `app/src/test/citiesFixture.ts`
- `app/src/test/cityShieldFixture.ts`
- `app/src/test/combinedAssaultFixture.ts`
- `app/src/test/companyState.ts`
- `app/src/test/coreFixture.ts`
- `app/src/test/diplomacyFixture.ts`
- `app/src/test/eliminatedFixture.ts`
- `app/src/test/explorationFixture.ts`
- `app/src/test/forgeHeirFixture.ts`
- `app/src/test/gameOverFixture.ts`
- `app/src/test/goalActionsFixture.ts`
- `app/src/test/hallFixture.ts`
- `app/src/test/heirState.ts`
- `app/src/test/improvementsFixture.ts`
- `app/src/test/japanCityClusterFixture.ts`
- `app/src/test/keshigFixture.ts`
- `app/src/test/lastBridgeFixture.ts`
- `app/src/test/mapArtFixture.ts`
- `app/src/test/mongolsPassageFixture.ts`
- `app/src/test/p881Scene.ts`
- `app/src/test/pathDetourFixture.ts`
- `app/src/test/pieceFixture.ts`
- `app/src/test/playerText.tsx`
- `app/src/test/progressFixture.ts`
- `app/src/test/promotionFixture.ts`
- `app/src/test/reinforcementFixture.ts`
- `app/src/test/researchFixture.ts`
- `app/src/test/roadsFixture.ts`
- `app/src/test/siegeFixture.ts`
- `app/src/test/siegeUnitsFixture.ts`
- `app/src/test/supplyFixture.ts`
- `app/src/test/turnFlowFixture.ts`
- `app/src/test/uniquePiecesFixture.ts`
- `app/src/test/unitEraFixture.ts`
- `app/src/test/wallsFixture.ts`
- `app/src/test/warDeclaredFixture.ts`
- `app/src/test/warPromptFixture.ts`

## External parity tests (2)

The siege comparison was deleted as parity-only. The diplomacy panel test remains a core-backed UI test.

- `app/src/test/siege.wasm.test.ts`
- `app/src/ui/DiplomacyPanel.wasm.test.tsx`

## Other app tests (61)

These used a test client or fixture helper and now assert against the WASM engine.

- `app/src/celebrate/EraUpTakeover.test.tsx`
- `app/src/celebrate/alarms.test.ts`
- `app/src/celebrate/moments.test.ts`
- `app/src/citiesOverview.test.ts`
- `app/src/cityProgress.test.ts`
- `app/src/client.test.ts`
- `app/src/eraObjectives.test.ts`
- `app/src/feel/turnFeedback.test.ts`
- `app/src/format.test.ts`
- `app/src/map/MapCanvas.test.ts`
- `app/src/map/lenses.test.ts`
- `app/src/map/unitEra.test.ts`
- `app/src/pathFriends.test.ts`
- `app/src/research.test.ts`
- `app/src/screens/TitleScreen.test.tsx`
- `app/src/screens/gameKeyHandler.test.ts`
- `app/src/siege.test.ts`
- `app/src/store.automation.test.ts`
- `app/src/store.autosave.test.ts`
- `app/src/store.feel.test.ts`
- `app/src/store.onlineExit.test.ts`
- `app/src/store.observer.test.ts`
- `app/src/store.remote.test.ts`
- `app/src/store.test.ts`
- `app/src/turnFlow.test.ts`
- `app/src/ui/AlliesFight.test.tsx`
- `app/src/ui/CitiesOverview.test.tsx`
- `app/src/ui/CityPanel.keyboard.test.tsx`
- `app/src/ui/CityPanel.test.tsx`
- `app/src/ui/CompanyShop.test.tsx`
- `app/src/ui/DecadenceNote.test.tsx`
- `app/src/ui/DiplomacyPanel.test.tsx`
- `app/src/ui/EraObjectives.test.tsx`
- `app/src/ui/FactionGoalProgress.test.tsx`
- `app/src/ui/GameOver.test.tsx`
- `app/src/ui/GoalActions.test.tsx`
- `app/src/ui/JustUnlocked.test.tsx`
- `app/src/ui/Levy.test.tsx`
- `app/src/ui/ObjectiveSheet.test.tsx`
- `app/src/ui/PhoneSheet.test.tsx`
- `app/src/ui/SeatBar.online.test.tsx`
- `app/src/ui/SeatBar.test.tsx`
- `app/src/ui/TechPanel.test.tsx`
- `app/src/ui/TitleChips.test.tsx`
- `app/src/ui/TopBar.test.tsx`
- `app/src/ui/TurnReport.test.tsx`
- `app/src/ui/UnitPanel.test.tsx`
- `app/src/ui/VpHoldings.test.tsx`
- `app/src/ui/WhereToBuild.test.tsx`
- `app/src/ui/accessibility.test.ts`
- `app/src/ui/courierHall.test.tsx`
- `app/src/ui/eliminatedLists.test.tsx`
- `app/src/ui/forecastWarnings.test.ts`
- `app/src/ui/goalsThree.test.tsx`
- `app/src/ui/hintState.test.ts`
- `app/src/ui/productionForecasts.test.ts`
- `app/src/ui/productionPurchases.test.ts`
- `app/src/ui/textArtifacts.screens.test.tsx`
- `app/src/ui/unlockCardModel.test.ts`
- `app/src/unseenUnlocks.test.ts`
- `app/src/vpModel.test.ts`
