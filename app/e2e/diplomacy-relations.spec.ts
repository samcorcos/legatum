import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { test, expect } from './fixtures';
import { allianceCoreSave } from '../src/test/allianceFixture';
import { SIZES, loadFixture } from './tour/helpers';
import { revealSidebar } from './sidebarAssertions';

const shots = fileURLToPath(new URL('./__screenshots__/diplomacy-relations/', import.meta.url));

for (const size of SIZES) {
  test(`relations overview at ${size.name}`, async ({ page }) => {
    await page.setViewportSize({ width: size.width, height: size.height });
    await loadFixture(page, allianceCoreSave(), 50);
    await revealSidebar(page, 'standings');
    await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click();
    const panel = page.getByRole('complementary', { name: 'Diplomacy', exact: true });
    await panel.getByRole('tab', { name: 'Relations' }).click();
    const tableRegion = panel.getByRole('region', { name: 'Relations table' });
    await expect(tableRegion).toBeVisible();
    await expect(panel.locator('.relations-table tbody tr')).toHaveCount(4);
    expect(await tableRegion.evaluate((el) => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
    await expect(panel.locator('.relations-cell.war')).not.toHaveCount(0);
    await expect(panel.locator('.relations-cell.alliance')).not.toHaveCount(0);
    await expect(panel.getByRole('tab', { name: 'Relations' })).toHaveAttribute('aria-selected', 'true');
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    await mkdir(shots, { recursive: true });
    await page.screenshot({ path: `${shots}/${size.name}.png`, animations: 'disabled' });
  });
}
