import { expect, test } from '@playwright/test';

// Atlas is the only UI look (design decision 2026-10-03); the phone is not a target.
for (const look of ['atlas']) test(`queue and independent purchase on WASM (${look})`, async ({ page }, testInfo) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(e.message));
  await page.goto('/?client=wasm');
  // Arrange a real-engine save with a known city, technology and treasury. All play below
  // uses visible buttons; never replace engine outcomes or intercept requests.
  const before = await page.evaluate(async () => {
    const module = '/src/wasmClient.ts';
    const storeModule = '/src/store.ts';
    const { createWasmClient } = await import(module);
    const { useGame } = await import(storeModule);
    const client = await createWasmClient();
    const setup = await client.standardSetup('424242', 'Athens', 'Japan', 'Ada', 'Bo');
    setup.players = setup.players.slice(0, 2).map((p: object) => ({ ...p, is_human: true }));
    setup.players[1].team = 1;
    await client.newGame(setup);
    const settler = client.view(0).units.find((u: { owner: number; kind: string }) => u.owner === 0 && u.kind === 'Settler');
    client.endTurn({ 0: [{ type: 'FoundCity', unit: settler.id }] });
    const saved = JSON.parse(client.save().replace(/([:[,]\s*)(\d{16,})(?=\s*[,}\]])/g, '$1"u64:$2"'));
    delete saved.archive;
    saved.players[0].gold = 1000; saved.players[0].techs = ['SoilReclamation', 'Exchange', 'Fabrication', 'Fortification', 'Ballistics'];
    const city = Object.values(saved.cities)[0] as { id: number; population: number; production_stored: number; pos: object };
    city.population = 3; city.production_stored = 0;
    client.load(JSON.stringify(saved).replace(/"u64:(\d+)"/g, '$1'));
    const view = client.view(0);
    useGame.setState({ client, screen: 'game', humans: [0], views: [view], seat: 0, orders: [[]], errors: [[]], ready: [false], skipped: [[]], selection: { kind: 'city', id: city.id }, report: null, reportOpen: false, techOpen: false, focus: { hex: city.pos, seq: 1 } });
    const detail = view.cities.find((c: { id: number }) => c.id === city.id).detail;
    return { city: city.id, gold: view.economy.gold, units: view.units.filter((u: { owner: number }) => u.owner === 0).length, turn: view.turn,
      food: detail.yields.total.food, production: detail.yields.total.production, growth: detail.turns_to_grow === null ? detail.yields.total.food < 2 * city.population ? '−1 pop' : 'Stagnant' : `${detail.turns_to_grow} turn${detail.turns_to_grow === 1 ? '' : 's'}` };
  });
  const panel = page.locator('.city-panel');
  await expect(panel.getByText(/Choose what .* builds/)).toBeVisible();
  const economySummary = panel.locator('.cp-economy > summary');
  await expect(panel.locator('.cp-economy')).not.toHaveAttribute('open');
  await expect(economySummary).toBeVisible();
  await expect(economySummary).toContainText(`${before.food} Food`);
  await expect(economySummary).toContainText(`${before.production} Production`);
  // Growth is a progress bar outside the disclosure (turns left, or the state when not growing).
  await expect(panel.getByTestId('city-growth')).toContainText(before.growth);
  const expectChoices = async (atLeast: number) => {
    const visibleOptions = await panel.locator('button.prod').evaluateAll((rows) => rows.filter((row) => {
      const box = row.getBoundingClientRect(), panel = row.closest('.city-panel')!.getBoundingClientRect();
      return box.height > 0 && box.top >= panel.top && box.bottom <= panel.bottom;
    }).length);
    // The Growth bar, coaching and the queue section share the header space; the rule (legatum-ui) is
    // that a whole build card stays in view without scrolling (two fit at 1280x800 with the Settler hint;
    // each queued item adds a row above the list, so one is the floor once something is queued).
    expect(visibleOptions, `at least ${atLeast} build choices below the compact header`).toBeGreaterThanOrEqual(atLeast);
  };
  await expectChoices(2);
  await panel.getByRole('button', { name: 'Build Market', exact: true }).click();
  // Build completes this city's decision and moves on (Civ 6); reopen the city from Cities.
  await expect(panel).toHaveCount(0);
  if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
  const cityButton = page.locator('.city-list-items button').first();
  if (!await cityButton.isVisible()) await page.locator('.city-list summary').click();
  await cityButton.click();
  await panel.getByRole('button', { name: 'Queue Granary', exact: true }).click();
  await panel.evaluate((el) => { el.scrollTop = 0; });
  await expectChoices(1);
  await expect(panel.getByRole('list', { name: 'Production queue' }).locator('li')).toHaveCount(2);
  await panel.getByRole('button', { name: 'Move Granary up' }).click();
  await panel.getByRole('button', { name: 'Move Market up' }).click();
  const older = panel.locator('details.unit-line');
  await expect(older.getByText('Also: Spearman', { exact: true })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Buy Spearman for 160 Gold', exact: true })).toBeHidden();
  await older.locator('summary').click();
  await panel.getByRole('button', { name: 'Buy Spearman for 160 Gold', exact: true }).click();
  await expect(panel.getByText('Buying Spearman for 160 Gold after Ready.', { exact: true })).toBeVisible();
  await panel.getByRole('button', { name: 'Cancel buying Spearman for 160 Gold', exact: true }).scrollIntoViewIfNeeded();
  await page.screenshot({ path: testInfo.outputPath('queue-and-buy.png') });
  await page.getByRole('button', { name: /^(Ready anyway|End turn)/ }).click();
  // With idle units, End turn asks once (B1) before committing.
  await page.getByRole('dialog', { name: 'End turn with decisions pending?' }).getByRole('button', { name: 'End turn anyway', exact: true }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(before.turn + 1));
  const after = await page.evaluate(async () => {
    const path = '/src/store.ts'; const { useGame } = await import(path);
    const state = useGame.getState(), view = state.client.view(0);
    const city = view.cities.find((c: { owner: number }) => c.owner === 0);
    const events = state.report?.events[0] ?? [];
    state.client.load(state.client.save());
    return { gold: view.economy.gold, units: view.units.filter((u: { owner: number }) => u.owner === 0), detail: city.detail, events, savedQueue: state.client.view(0).cities.find((c: { owner: number }) => c.owner === 0).detail.production_queue };
  });
  expect(after.gold).toBeLessThan(before.gold - 150);
  expect(after.events).toContainEqual({ type: 'ItemBought', city: before.city, owner: 0, item: { kind: 'Unit', unit: 'Soldier' }, gold: 160 });
  expect(after.units).toHaveLength(before.units + 1);
  expect(after.detail.producing).toEqual({ kind: 'Building', building: 'Market' });
  expect(after.detail.production_stored).toBeGreaterThan(0);
  expect(after.savedQueue).toEqual([{ kind: 'Building', building: 'Granary' }]);
  expect(errors).toEqual([]);
});
