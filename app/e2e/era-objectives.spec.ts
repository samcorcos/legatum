// World Ages are how you win, so they are always in view (legatum-ui "era objectives are always visible with progress"):
// the race panel's block and the sidebar line with live progress, the ledger, and the research screen's Age header (before
// any Age begins, and once one has). The numbers that need a long game are staged render-only
// on the live view with `withV2`. Set ERA_OBJ_SHOTS to a folder to keep screenshots at both laptop sizes.
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import type { useGame } from '../src/store';
import { withV2, withWorldAges, type V2Options } from '../src/test/vpV2Fixture';
import { expectSidebarGlance } from './sidebarAssertions';

type GameWindow = Window & { legatum: typeof useGame };

const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const shots = process.env.ERA_OBJ_SHOTS;

async function start(page: Page) {
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

/** Stage the v2 numbers on the live view (render only): your era, what you scored, your progress. */
async function stage(page: Page, options: V2Options) {
  const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
  const view = withV2(base, options);
  await page.evaluate((v) => { const store = (window as unknown as GameWindow).legatum, s = store.getState(); store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) }); }, view);
  await expect(page.getByTestId('era-hud')).toBeVisible();
}

async function shot(page: Page, name: string, size: { width: number }) {
  await page.mouse.move(4, 300);
  await page.waitForTimeout(300);
  if (shots) await page.screenshot({ path: join(shots, `${name}-${size.width}.png`) });
}

for (const size of SIZES) {
  test.describe(`World Ages at ${size.width}x${size.height}`, () => {
    test.use({ viewport: size });

    test('World Age cards and hidden next Age', async ({ page }) => {
      await start(page);
      await stage(page, { era: 2, scored: [] });
      await page.evaluate(() => {
        const store = (window as unknown as GameWindow).legatum;
        const s = store.getState();
        const v = structuredClone(s.views![s.seat]);
        v.eras = v.eras.filter(e => e.era === 2);
        v.eras[0].objectives = [
          { slot: 0, family: 'Fight', text: 'Veterans: earn 4 promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)', target: 4, objective: { kind: 'Veterans', n: 4 } },
          { slot: 1, family: 'Fight', text: 'Breach: capture a walled city', target: 1, objective: { kind: 'Breach' } },
          { slot: 2, family: 'Fight', text: 'Humble the mighty: capture a city from an empire ahead of you on VP or, if you lead, from the rival with the most VP', target: 1, objective: { kind: 'HumbleTheMighty' } },
        ];
        v.objective_progress = [
          { era: 2, slot: 0, current: 2, target: 4, entered: true, era_scored: false, ready: false },
          { era: 2, slot: 1, current: 0, target: 1, entered: true, era_scored: false, ready: false },
          { era: 2, slot: 2, current: 0, target: 1, entered: true, era_scored: false, ready: false },
        ];
        store.setState({ views: s.views!.map((x, i) => i === s.seat ? v : x) });
      });
      const race = page.getByRole('region', { name: 'Victory race', exact: true });
      await expect(race).toContainText('The next Age begins when any empire enters the Medieval era');
      await race.getByRole('button', { name: 'Score ledger', exact: true }).click();
      const sheet = page.getByTestId('objective-sheet');
      await expect(sheet).not.toContainText('Age III: Medieval');
      const era = sheet.locator('.sl-row[data-row="era-2"] button');
      if (await era.getAttribute('aria-expanded') === 'false') await era.click();
      await expect(sheet.locator('#sl-detail-era-2 .vh-obj')).toHaveCount(3);
      await expect(sheet.locator('#sl-detail-era-2')).toContainText('Promotions earned since the Age began: 2 of 4');
      await shot(page, 'objective-deck', size);
    });

    test('World Ages: the oldest open Age is one line in the sidebar at scroll 0, and a click opens How you win', async ({ page }) => {
      await start(page);
      const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
      const view = withWorldAges(base);
      await page.evaluate((v) => { const store = (window as unknown as GameWindow).legatum, s = store.getState(); store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) }); }, view);
      const line = page.getByRole('complementary', { name: 'Empire sidebar' }).locator('.sidebar-objective');
      await expect(line).toBeVisible();
      await expect(line).toContainText('Classical Age · 1 VP');
      await expect(line.getByTestId('era-objective')).toHaveCount(1);
      await expect(line.locator('.eo-text')).toHaveText('Veterans');
      await expect(line.locator('.eo-num')).toHaveText('2/4');
      // Visible without scrolling the sidebar.
      const box = (await line.boundingBox())!;
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height).toBeLessThanOrEqual(size.height);
      await expectSidebarGlance(page);
      await shot(page, 'age-standing-line', size);
      await line.getByRole('button', { name: /How you win/ }).click();
      await expect(page.getByTestId('objective-sheet')).toBeVisible();
    });

    test('one objective with its progress; alternatives open in the ledger', async ({ page }) => {
      await start(page);
      await stage(page, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } });
      const hud = page.getByTestId('era-hud');
      await expect(hud.getByTestId('era-objective')).toHaveCount(1);
      await expect(hud).toContainText('3/4');
      await expectSidebarGlance(page);
      await hud.getByRole('button', { name: /How you win/ }).click();
      await expect(page.getByTestId('objective-sheet')).toBeVisible();
      const era = page.locator('.sl-row[data-row="era-2"] button');
      if (await era.getAttribute('aria-expanded') === 'false') await era.click();
      await expect(page.locator('#sl-detail-era-2 .vh-obj')).toHaveCount(3);
      await shot(page, 'objective-ledger', size);
    });

    test('the research screen header: no Age before the first begins, then the Age in focus and when the next begins', async ({ page }) => {
      await start(page);
      await page.locator('.topbar .research-chip').click();
      const dialog = page.locator('dialog.tech-modal[open]');
      const header = dialog.getByTestId('research-objectives');
      await expect(header).toBeVisible();
      await expect(header.getByTestId('research-objectives-current')).toContainText('No Age has begun');
      await expect(header.getByTestId('research-objectives-next')).toHaveCount(0);
      await expect(header.getByTestId('era-objective')).toHaveCount(0);
      await shot(page, 'research-ancient', size);
      await page.keyboard.press('Escape');
      // The Medieval Age in focus shows real progress: one card part way, one met.
      await stage(page, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1], '3:0': [9, 14], '3:1': [40, 40], '3:2': [0, 1] } });
      await page.locator('.topbar .research-chip').click();
      // The fixture's rival began the Medieval Age, so it is the Age in focus.
      await expect(header.getByTestId('research-objectives-current')).toContainText(/\w+ Age: score 1 VP/);
      await expect(header.getByTestId('research-objectives-current').getByTestId('era-objective')).toHaveCount(3);
      await expect(header.getByTestId('research-objectives-next')).toContainText('The next Age begins when any empire enters the Renaissance era.');
      await expect(header.getByTestId('research-objectives-current').locator('.eo-line.met')).toHaveCount(1);
      // The seal stamps in and the met card flashes the first time this Age is shown; let both finish before the shot.
      await header.evaluate((el) => Promise.all(el.getAnimations({ subtree: true }).map((a) => a.finished)));
      await shot(page, 'research-classical', size);
    });
  });
}
