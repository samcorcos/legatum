// Finishing a technology must feel like an unlock (legatum-ui): the real engine finishes Currency at the
// end of the turn, and the unlock card announces it.
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { auditLayout } from './layoutAudit';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import { TECHS, TECH_ERA } from '../src/data';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];

async function startOnTheLastResearchTurn(page: Page) {
  const state = researchFixture('India');
  state.players[0].research = 'Currency';
  state.players[0].science = 5000;
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(state));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

const card = (page: Page) => page.locator('.moment-card.unlock');

for (const size of SIZES) test.describe(`tech unlock ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('Gunpowder shows each new unit once with its upgrade path', async ({ page }) => {
    const state = researchFixture('India');
    state.players[0].techs = TECHS.filter((tech) => TECH_ERA[tech] < 3);
    state.players[0].research = 'Gunpowder';
    state.players[0].science = 5000;
    await page.addInitScript((game) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, coreFixture(state));
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await endPlanning(page);
    const unlock = card(page);
    await expect(unlock).toBeVisible();
    await expect(unlock.locator('.unlock-tile')).toHaveCount(3);
    await expect(unlock.locator('.unlock-tile.unit')).toHaveCount(2);
    await expect(unlock.locator('.unlock-tile').nth(0)).toContainText('From Pikeman');
    await expect(unlock.locator('.unlock-tile').nth(1)).toContainText('From Crossbowman');
    // Gunpowder also reveals Niter, its era's strategic resource.
    await expect(unlock.locator('.unlock-tile').nth(2)).toContainText('Niter');
    await expect(unlock).not.toContainText('can upgrade to');
    await page.mouse.move(size.width - 2, size.height - 2);
    await page.screenshot({ path: `test-results/gunpowder-unlock-${size.width}x${size.height}.png`, animations: 'disabled' });
    await page.keyboard.press('Escape');
    await expect(page.getByRole('complementary', { name: 'Turn report' })).toContainText('Musketman (from Pikeman)');
  });

  test('the card names the unlock; Return opens the city with its normal build list', async ({ page }) => {
    await startOnTheLastResearchTurn(page);
    await endPlanning(page);
    await expect(card(page)).toBeVisible();
    await expect(card(page)).toContainText('Currency');
    await expect(card(page)).toContainText('Build a Revenue Office in any city: +3 Gold');
    await expect(card(page)).toContainText('Imperial Diwan');
    await expect(card(page).getByRole('button', { name: /^Show Revenue Office in / })).toBeVisible();
    // Nothing is queued behind Currency: the card offers the idle decision as a button too.
    await expect(card(page)).toContainText('No research is set.');
    await expect(card(page).getByRole('button', { name: /^Choose research/ })).toBeVisible();
    await expect(card(page).getByRole('button', { name: /^Later/ })).toBeVisible();
    // One live Return primary: the dock's own Return badge steps back while the card owns Return.
    await expect(page.locator('.seatbar .cta .kbd').first()).toHaveCSS('visibility', 'hidden');
    // The whole card is on screen with nothing clipped or overprinted.
    const box = (await card(page).boundingBox())!;
    expect(box.y).toBeGreaterThanOrEqual(0); expect(box.y + box.height).toBeLessThanOrEqual(size.height);
    const issues = (await auditLayout(page, { minTarget: 24 })).filter((i) => /moment-card|unlock/.test(i.where) && i.kind !== 'overlap');
    expect(issues).toEqual([]);
    await page.keyboard.press('Enter');
    await expect(card(page)).toHaveCount(0);
    await expect(page.locator('.city-panel')).toBeVisible();
    await expect(page.getByTestId('just-unlocked')).toHaveCount(0);
    await expect(page.locator('.city-panel [data-item="b:Market"]')).toHaveCount(1);
    await expect(page.locator('.seatbar .cta .kbd').first()).toHaveCSS('visibility', 'visible');
  });

  test('Later (Esc) puts it off and the research card stays readable', async ({ page }) => {
    await startOnTheLastResearchTurn(page);
    await endPlanning(page);
    await expect(card(page)).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(card(page)).toHaveCount(0);
    await page.locator('.topbar .research-chip').click();
    const node = page.locator('dialog.tech-modal .era-card[data-tech="Currency"]');
    await expect(node).toContainText('Revenue Office');
    await expect(node.locator('.new-diamond, .era-new')).toHaveCount(0);
  });

  test('a click or drag on the map leaves it up; Esc then brings the report, whose line reopens it', async ({ page }) => {
    await startOnTheLastResearchTurn(page);
    await endPlanning(page);
    await expect(card(page)).toBeVisible();
    const map = page.locator('[data-game-map]');
    const at = (await map.boundingBox())!;
    await page.mouse.click(at.x + 60, at.y + at.height - 60);
    await page.mouse.move(at.x + 80, at.y + 120);
    await page.mouse.down(); await page.mouse.move(at.x + 160, at.y + 160, { steps: 5 }); await page.mouse.up();
    await expect(card(page)).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(card(page)).toHaveCount(0);
    const report = page.getByRole('complementary', { name: 'Turn report' });
    await expect(report).toBeVisible();
    const line = report.getByRole('button', { name: /researched Currency · unlocks Revenue Office/ });
    await expect(line).toBeVisible();
    await line.click();
    await expect(card(page)).toBeVisible();
    await expect(card(page)).toContainText('Currency');
    await page.keyboard.press('Escape');
    await expect(card(page)).toHaveCount(0);
  });

  test('it waits out an open research panel instead of hiding behind it, and Esc closes the panel first', async ({ page }) => {
    await startOnTheLastResearchTurn(page);
    await endPlanning(page);
    await expect(card(page)).toBeVisible();
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: true }));
    await expect(page.locator('dialog.tech-modal')).toBeVisible();
    await expect(card(page)).toHaveCount(0);
    await page.keyboard.press('Escape');
    await expect(page.locator('dialog.tech-modal')).toHaveCount(0);
    await expect(card(page)).toBeVisible();
  });

  test('Choose research closes the card and opens the tech panel', async ({ page }) => {
    await startOnTheLastResearchTurn(page);
    await endPlanning(page);
    await card(page).getByRole('button', { name: /^Choose research/ }).click();
    await expect(card(page)).toHaveCount(0);
    await expect(page.locator('dialog.tech-modal')).toBeVisible();
  });

  test('a click on End Turn with the card open ends the turn and the new turn starts without it', async ({ page }) => {
    await startOnTheLastResearchTurn(page);
    await endPlanning(page);
    await expect(card(page)).toBeVisible();
    await endPlanning(page);
    await expect(card(page)).toHaveCount(0);
    await expect(page.getByTestId('turn-number')).toHaveText('4');
  });
});
