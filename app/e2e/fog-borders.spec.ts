import { fileURLToPath } from 'node:url';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';

test.use({ viewport: { width: 1000, height: 600 } });

test('captured city and borders remain current under fog at 1000x600', async ({ page }) => {
  const { fixture, walled } = wallsFixture();
  const city = fixture.cities.find((c) => c.id === walled)!;
  const oldOwner = city.owner;
  // The fixture saw the city before capture. Remove the observer; leave its private memory.
  fixture.units = fixture.units.filter((u) => u.owner !== 0 || u.kind === 'Settler');
  city.owner = 2;
  city.name = 'Waypost Seven';
  city.hp = 17;
  for (let i = 0; i < fixture.tileCity.length; i++) if (fixture.tileCity[i] === walled) fixture.owner[i] = 2;
  const save = coreFixture(fixture);
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: save, turn: fixture.turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.waitForFunction(() => (window as unknown as { legatum?: { getState(): { views: unknown[] | null } } }).legatum?.getState().views?.length);
  const observed = await page.evaluate((id) => {
    const w = window as unknown as { legatum: { getState(): { views: import('../src/types').PlayerView[]; seat: number; jumpTo(h: import('../src/types').Hex): void } }; legatumMap: { redraw(): void } };
    const s = w.legatum.getState(), view = s.views[s.seat], c = view.cities.find((v) => v.id === id)!;
    s.jumpTo(c.pos);
    w.legatumMap.redraw();
    return { city: c, border: view.tiles.find((t) => t.explored && !t.visible && t.owner === c.owner)?.owner };
  }, walled);
  expect(observed.city).toMatchObject({ owner: 2, name: 'Waypost Seven', visible: false });
  expect(observed.border).toBe(2);
  expect(observed.city.owner).not.toBe(oldOwner);
  await page.screenshot({ path: fileURLToPath(new URL('../test-results/fog-borders-1000x600.png', import.meta.url)), animations: 'disabled' });
});
