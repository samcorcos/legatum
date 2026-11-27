import { test, expect } from './fixtures';
import { openLateRace } from './lateRaceSave';
import { expectSidebarGlance } from './sidebarAssertions';
for (const viewport of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`the completed Age summary and full ledger stay one click away ${viewport.width}`, async ({ page }) => {
    await page.setViewportSize(viewport);
    // An old expanded preference must not recreate the cluttered sidebar.
    await page.addInitScript(() => localStorage.setItem('legatum.era-hud-open.v1', '1'));
    await openLateRace(page);
    await expectSidebarGlance(page);
    const hud = page.getByTestId('era-hud');
    await expect(hud.locator('.sidebar-next')).toHaveText('Age scored');
    await expect(hud).not.toContainText('of 10 VP');
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    await page.keyboard.press('Escape');
    await expectSidebarGlance(page);
  });
}
