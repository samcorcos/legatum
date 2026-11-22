import { revealSidebar } from './sidebarAssertions';
import { endPlanning } from './turnActions';
import { test, expect } from './fixtures';
import { explorationCoreSave, explorationFixture } from '../src/test/explorationFixture';

// Resume a real engine fixture through the supported autosave path, then use UI controls only.
// A finite island makes both continued exploration and completion deterministic.
for (const viewport of [{ name: 'desktop', width: 1440, height: 1100 }]) {
  test(`Auto-explore (${viewport.name}): explores, finds a ruin, stops, and asks for orders`, async ({ browser, baseURL, watchErrors }) => {
    const context = await browser.newContext({ baseURL, viewport });
    watchErrors(context);
    await context.addInitScript((game) => {
      // Init scripts also run in the page's initial about:blank, where storage access throws.
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Explorer'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, explorationCoreSave());
    const page = await context.newPage();
    try {
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      const actions = page;
      const auto = actions.getByRole('button', { name: /^Auto-explore/ });
      await expect(auto).toBeVisible();
      await auto.click();
      await expect(actions.getByRole('button', { name: /^Stop auto-explore/ })).toHaveAttribute('aria-pressed', 'true');
      await revealSidebar(page, 'turn-more');
      await page.getByRole('button', { name: /^Pending orders/ }).click();
      await expect(page.locator('#pending-orders')).toContainText('auto-explore (automated)');
      await revealSidebar(page, 'turn-more');
      await page.getByRole('button', { name: /^Pending orders/ }).click();
      const positions = new Set<string>(); const explored: number[] = [];
      let stopped = false, ruin = false;
      for (let turn = 2; turn <= 12; turn++) {
        const close = page.getByRole('button', { name: 'Close report', exact: true });
        if (await close.isVisible()) await close.click();
        await endPlanning(page);
        await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
        // Ruin celebrations hold the requested report; skip them before reading it.
        if (await page.locator('.moment-card, [data-celebrate-delta]').count()) await page.keyboard.press('Escape');
        const state = await page.evaluate(() => JSON.parse(JSON.parse(localStorage.getItem('legatum.autosave.v1')!).game));
        positions.add(JSON.stringify(state.units['0'].pos));
        explored.push(state.players[0].tile_memory.filter(Boolean).length);
        ruin ||= state.players[0].ruins_explored > 0;
        if (!state.units['0'].automation) {
          stopped = true;
          await expect(page.getByRole('complementary', { name: 'Turn report' })).toContainText('stopped auto-explore: nothing reachable left to explore. Needs orders.');
          await page.getByRole('button', { name: 'Close report', exact: true }).click();
          await expect(page.getByRole('button', { name: /^2 units need orders/ })).toBeVisible();
          await expect(actions.getByRole('button', { name: /^Auto-explore/ })).toBeVisible();
          break;
        }
      }
      expect(stopped).toBe(true); expect(ruin).toBe(true);
      expect(positions.size).toBeGreaterThanOrEqual(3);
      expect(explored.at(-1)!).toBeGreaterThan(explored[0]);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      await page.screenshot({ path: `test-results/auto-explore-${viewport.name}.png` });
    } finally { await context.close(); }
  });
}

// Every military unit can auto-explore with E; it is secondary to Fortify, and a Settler never gets it.
test('E toggles auto-explore on a military unit, and it moves on the next turn', async ({ browser, baseURL, watchErrors }) => {
  const context = await browser.newContext({ baseURL, viewport: { width: 1440, height: 900 } });
  watchErrors(context);
  const fixture = explorationFixture();
  fixture.units[0].kind = 'Soldier';
  await context.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 1, names: ['Explorer'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, explorationCoreSave(fixture));
  const page = await context.newPage();
  try {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    const auto = page.getByRole('button', { name: /^Auto-explore/ });
    await expect(auto).toBeVisible();
    await expect(page.getByRole('button', { name: /^Fortify/ })).toBeVisible();
    await expect(page.locator('.actions .act-wrap.primary')).not.toContainText('Auto-explore');
    await page.keyboard.press('e');
    await expect(page.getByRole('button', { name: /^Stop auto-explore/ })).toHaveAttribute('aria-pressed', 'true');
    await page.keyboard.press('e');
    await expect(page.getByRole('button', { name: /^Auto-explore/ })).toBeVisible();
    await page.keyboard.press('e');
    await expect(page.getByRole('button', { name: /^Stop auto-explore/ })).toBeVisible();
    const start = await page.evaluate(() => JSON.stringify(JSON.parse(JSON.parse(localStorage.getItem('legatum.autosave.v1')!).game).units['0'].pos));
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    const after = await page.evaluate(() => JSON.parse(JSON.parse(localStorage.getItem('legatum.autosave.v1')!).game).units['0']);
    expect(JSON.stringify(after.pos)).not.toBe(start);
    expect(after.kind).toBe('Soldier');
    expect(after.automation).toBe('Explore');
  } finally { await context.close(); }
});
