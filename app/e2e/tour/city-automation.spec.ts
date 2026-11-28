// Economic delegation: one toggle in a city and bulk controls in Cities.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { citiesFixture } from '../../src/test/citiesFixture';
import { SIZES, loadFixture, selectThing, shot, dismissCelebrations, closeReportIfOpen } from './helpers';

for (const size of SIZES) test.describe(`city automation ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  test('one economic toggle, manual queue, and bulk automation', async ({ page }) => {
    test.setTimeout(180_000);
    const fixture = citiesFixture();
    const mine = fixture.cities.filter((c) => c.owner === 0).sort((a, b) => a.id - b.id);
    await loadFixture(page, coreFixture(fixture), 2);
    await selectThing(page, 'city', mine[3].id);
    const toggle = page.getByRole('button', { name: /^Automate / });
    await expect(toggle).toHaveAttribute('aria-pressed', 'false');
    await expect(page.getByRole('combobox', { name: /^Automate / })).toHaveCount(0);
    await shot(page, size, '01-idle-manual');
    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await shot(page, size, '02-idle-automated');
    await page.keyboard.press('i');
    await expect(toggle).toHaveAttribute('aria-pressed', 'false');
    await page.keyboard.press('i');
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await selectThing(page, 'city', mine[0].id);
    await toggle.click();
    await expect(page.getByText('Your queue goes first.', { exact: false })).toBeVisible();
    await shot(page, size, '03-busy-automated');
    await page.keyboard.press('Escape');
    await page.evaluate(() => (window as unknown as { legatum: { getState(): { set(p: object): void } } }).legatum.getState().set({ citiesOpen: true }));
    const sheet = page.getByTestId('cities-overview');
    await expect(sheet).toBeVisible();
    await expect(sheet.getByRole('combobox', { name: 'Building focus for selected cities' })).toHaveCount(0);
    await sheet.getByRole('checkbox', { name: 'Select all cities' }).check();
    await sheet.getByRole('button', { name: 'Automate', exact: true }).click();
    await shot(page, size, '04-cities-automated');
    await page.keyboard.press('Escape');
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 60_000 });
    await dismissCelebrations(page);
    await closeReportIfOpen(page);
    await selectThing(page, 'city', mine[3].id);
    await expect(toggle).toHaveAttribute('aria-pressed', 'true');
    await shot(page, size, '05-after-turn');
  });
});
