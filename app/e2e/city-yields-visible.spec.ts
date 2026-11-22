// The four city totals and the next tile's yield must read separately at both review sizes.
// Screenshots are the review artifact for the newly founded and large-city cases.
import { mkdir } from 'node:fs/promises';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { citiesFixture } from '../src/test/citiesFixture';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }] as const;
const shots = '../docs/sim-results/ui/city-yields-visible';

for (const size of SIZES) test.describe(`city yields ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  for (const [name, population] of [['new-city', 1], ['large-city', 6]] as const) {
    test(`${name}: totals, next tile, build rate, and comparison are visible`, async ({ page }) => {
      const fixture = citiesFixture();
      const city = fixture.cities.find((c) => c.owner === 0 && c.name === (population === 1 ? 'Geneva' : 'Basel'))!;
      if (population === 1) {
        city.population = 1;
        city.producing = { kind: 'Unit', unit: 'Soldier' };
      } else {
        // Keep the large city growing so the shot reviews ordinary yields, not starvation.
        city.population = 6;
        city.buildings.push('Granary');
        city.producing = { kind: 'Unit', unit: 'Soldier' };
      }
      const game = coreFixture(fixture);
      await page.addInitScript((game) => {
        if (!location.protocol.startsWith('http')) return;
        localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
        localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      }, game);
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.getByTestId('turn-number')).toHaveText('2');
      await page.evaluate((id) => {
        const s = (window as unknown as GameWindow).legatum.getState();
        s.set({ techOpen: false });
        s.select({ kind: 'city', id }, true);
      }, city.id);

      const panel = page.locator('.city-panel');
      await expect(panel).toContainText(city.name);
      const yields = panel.getByTestId('city-yields');
      await expect(yields).toBeInViewport({ ratio: 1 });
      await expect(yields.locator('.cp-yield')).toHaveCount(4);
      await expect(yields.locator('.cp-yield-l')).toHaveText(['Food', 'Production', 'Gold', 'Science']);
      await expect(panel.locator('.cp-economy')).not.toHaveAttribute('open', '');
      if (population === 1) await expect(panel.getByTestId('city-growth').locator('.pbar-foot-left')).toContainText('Next tile');
      await expect(panel.getByTestId('city-production')).toContainText(/\+\d+ Production\/turn/);
      const compare = panel.getByRole('button', { name: 'Compare cities C' });
      await expect(compare).toBeInViewport({ ratio: 1 });
      await page.evaluate(() => document.fonts.ready.then(() => undefined));
      await page.mouse.move(5, size.height / 2);
      await mkdir(shots, { recursive: true });
      await page.screenshot({ path: `${shots}/${name}-${size.width}x${size.height}.png`, animations: 'disabled' });
      await compare.click();
      await expect(page.getByTestId('cities-overview')).toBeVisible();
      await expect(page.getByTestId('cities-overview').getByRole('columnheader', { name: 'Production' })).toHaveAttribute('aria-sort', 'descending');
    });
  }
});
