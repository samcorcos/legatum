import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { auditContrast } from './contrast';
import { explorationFixture, explorationCoreSave } from '../src/test/explorationFixture';
import { fromOffset, tileIndex } from '../src/hex';

const art = path.resolve('../.progress/art/celebrate');
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('No Vite address');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});
test.use({ viewport: { width: 1440, height: 900 }, hasTouch: true });

async function inject(page: import('@playwright/test').Page, tier: string, count = 1) {
  await page.evaluate(async ({ tier, count }) => {
    const file = '/src/test/celebrationFixture.ts';
    const { celebrationFixture } = await import(/* @vite-ignore */ file);
    await celebrationFixture(tier, count);
  }, { tier, count });
}

async function injectCamp(page: import('@playwright/test').Page) {
  await page.evaluate(async () => {
    const file = '/src/test/celebrationFixture.ts';
    const { campCelebrationFixture } = await import(/* @vite-ignore */ file);
    await campCelebrationFixture();
  });
}

async function holdCardsForLayout(page: import('@playwright/test').Page) {
  await page.evaluate(async () => {
    const file = '/src/celebrate/moments.ts';
    const { DURATION } = await import(/* @vite-ignore */ file);
    DURATION.notable = DURATION.major = DURATION.legendary = 120_000;
  });
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  await page.evaluate(() => document.fonts.ready);
  await mkdir(art, { recursive: true });
});

for (const tier of ['minor', 'notable', 'major', 'legendary']) {
  test(`${tier}: visible evidence, exact effects, no modal takeover`, async ({ page }) => {
    const errors: string[] = []; page.on('pageerror', (e) => errors.push(e.message));
    // Only the brief counter-only fixture needs terrain preloaded. Other tiers exercise cold load.
    if (tier === 'minor') {
      await inject(page, 'notable');
      await page.waitForTimeout(800);
      // The production delta lasts one second; hold it while Playwright inspects a busy browser.
      await page.evaluate(async () => {
        const file = '/src/celebrate/moments.ts';
        const { DURATION } = await import(/* @vite-ignore */ file);
        DURATION.minor = 30_000;
      });
    }
    await inject(page, tier);
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    if (tier === 'minor') {
      await expect(page.locator('[data-celebrate-delta="population"]')).toHaveText('+1');
      await expect(page.locator('.moment-card')).toHaveCount(0);
    } else {
      await expect(page.locator(`.moment-card[data-tier="${tier}"]`)).toBeVisible();
      await expect(page.locator('.moment-card')).not.toHaveAttribute('aria-modal', 'true');
    }
    await page.screenshot({ path: path.join(art, `${tier}.png`) });
    if (tier === 'notable' || tier === 'major') {
      await expect.poll(() => page.evaluate(() => window.__feelLog?.filter((e) => e.name === 'discovery').length)).toBe(1);
      expect(await page.evaluate(() => window.__feelLog?.find((e) => e.name === 'discovery')?.soundHook)).toBe(`sting.${tier}`);
    }
    expect(errors).toEqual([]);
  });
}

test('End Turn remains clickable while a card is visible and the gesture still commits', async ({ page }) => {
  await inject(page, 'notable');
  await expect(page.locator('.moment-card')).toBeVisible();
  const endTurn = page.locator('.touch-turn-controls').getByRole('button', { name: 'End Turn', exact: true });
  await expect(endTurn).toBeEnabled();
  await endTurn.click();
  // With decisions pending, End Turn asks once (B1); confirming still commits the gesture.
  await page.getByRole('dialog', { name: 'End turn with decisions pending?' }).getByRole('button', { name: 'End turn anyway', exact: true }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await expect(page.locator('.moment-card', { hasText: 'The keepers’ vault' })).toHaveCount(0);
});

test('three-card cap, +N more opens Discoveries, and a tap interrupts without blocking map controls', async ({ page }) => {
  await inject(page, 'notable', 5);
  await expect(page.getByRole('button', { name: '+2 more in the turn report', exact: true })).toBeVisible();
  for (let i = 0; i < 3; i++) {
    await expect(page.locator('.moment-card')).toBeVisible();
    await page.getByRole('button', { name: 'Dismiss celebration' }).click();
  }
  await expect(page.locator('.moment-card')).toHaveCount(0);
  await expect(page.locator('.moment-more')).toHaveCount(0);
  await inject(page, 'notable', 5);
  await page.getByRole('button', { name: '+2 more in the turn report', exact: true }).click();
  await expect(page.locator('.moment-card')).toHaveCount(0);
  await expect(page.locator('.moment-more')).toHaveCount(0);
  await expect(page.getByRole('complementary', { name: 'Turn report' })).toBeVisible();
  await expect(page.locator('.report-group[data-category="discoveries"] .rline')).toHaveCount(5);
  await page.screenshot({ path: path.join(art, 'collapsed.png') });
});

test('reduced motion: no moving CSS, no changing map pixels or counter positions', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await inject(page, 'notable');
  await expect.poll(() => page.evaluate(() => window.__feelLog?.filter((e) => e.name === 'discovery').length)).toBe(1);
  await page.waitForTimeout(800); // Decode the terrain assets before pixel-stability comparisons.
  await inject(page, 'major');
  await expect(page.locator('.moment-card')).toBeVisible();
  await expect(page.locator('[data-celebrate-delta="gold"]')).toHaveText('+40');
  // Wait for actual semantic delivery, not a wall-clock guess after a cold map render.
  await expect.poll(() => page.evaluate(() => window.__feelLog?.filter((e) => e.name === 'discovery').length)).toBe(1);
  const position = await page.locator('.moment-card').boundingBox();
  const canvas = page.locator('canvas').first();
  const first = await canvas.screenshot();
  await page.waitForTimeout(300);
  expect(await canvas.screenshot()).toEqual(first);
  expect(await page.locator('.moment-card').boundingBox()).toEqual(position);
  expect(await page.evaluate(() => document.getAnimations().filter((a) => a.playState === 'running').length)).toBe(0);
  await page.screenshot({ path: path.join(art, 'reduced-motion.png') });
  expect(await page.evaluate(() => window.__feelLog?.filter((e) => e.name === 'discovery').length)).toBe(1);
});


for (const width of [1024, 1440]) for (const look of ['atlas']) {
  test(`${look} ${width}: every tier stays outside the VP race, including arrival frames`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await holdCardsForLayout(page);
    await page.evaluate(async () => {
      const hints = '/src/ui/hintState.ts';
      const preferences = (await import(/* @vite-ignore */ hints)).useHintPreferences.getState();
      preferences.reset();
      preferences.setEnabled(false);
      const moments = '/src/celebrate/moments.ts';
      const { DURATION } = await import(/* @vite-ignore */ moments);
      DURATION.notable = DURATION.major = DURATION.legendary = 120_000;
    });
    for (const tier of ['notable', 'major', 'legendary']) {
      await inject(page, tier);
      await expect(page.locator('.moment-card')).toBeVisible();
      // A selection can survive a turn. Exercise the real panel alongside every tier.
      await page.evaluate(async () => {
        const file = '/src/store.ts'; const { useGame } = await import(/* @vite-ignore */ file);
        const s = useGame.getState(), view = s.views[s.seat];
        useGame.setState({ selection: { kind: 'unit', id: view.units.find((u: { owner: number }) => u.owner === view.me).id } });
      });
      await expect(page.locator('.unit-panel')).toBeVisible();
      const frames = await page.evaluate(async () => {
        const overlaps: string[] = [];
        const intersect = (a: DOMRect, b: DOMRect) => a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
        const start = performance.now();
        await new Promise<void>((resolve) => {
          const frame = () => {
            const card = document.querySelector('.moment-card')!.getBoundingClientRect();
            const vp = document.querySelector('.vp')!.getBoundingClientRect();
            if (intersect(card, vp)) overlaps.push('card overlaps VP');
            for (const control of document.querySelectorAll('.unit-panel, .touch-turn-controls')) {
              if (intersect(card, control.getBoundingClientRect())) overlaps.push(`card overlaps ${control.className}`);
            }
            if (card.left < 0 || card.right > innerWidth) overlaps.push('card outside screen');
            for (const child of document.querySelectorAll('.moment-card h2, .moment-effect')) {
              if (intersect(child.getBoundingClientRect(), vp)) overlaps.push('card text overlaps VP');
            }
            if (performance.now() - start < 300) requestAnimationFrame(frame); else resolve();
          };
          frame();
        });
        return overlaps;
      });
      expect(frames).toEqual([]);
      await expect(page.getByTestId('first-game-hint')).toHaveCount(0);
      await expect(page.getByRole('button', { name: 'Show on map', exact: true })).toBeVisible();
      if (width < 700) {
        expect((await page.locator('.moment-card').boundingBox())!.height).toBeLessThanOrEqual(184);
        await expect(page.locator('.population-counter')).toBeHidden();
        await expect(page.locator('.science-stored')).toBeHidden();
        const boxes = await page.locator('.gold-stat, .science-stat, .research-slot, .topbar .turn, .topbar .menu').evaluateAll((els) => els.map((el) => {
          const r = el.getBoundingClientRect(); return { left: r.left, right: r.right, width: r.width };
        }));
        expect(boxes).toHaveLength(5);
        for (let i = 0; i < boxes.length; i++) {
          expect(boxes[i].width).toBeGreaterThanOrEqual(44);
          expect(boxes[i].left).toBeGreaterThanOrEqual(0);
          expect(boxes[i].right).toBeLessThanOrEqual(width);
          if (i) expect(boxes[i].left).toBeGreaterThanOrEqual(boxes[i - 1].right - .5);
        }
        expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(width);
        if (tier === 'major') {
          await expect(page.locator('.moment-card')).toHaveCSS('border-image-source', 'none');
          await expect(page.locator('.moment-card')).toHaveCSS('border-left-width', '6px');
          await expect(page.locator('.moment-rays:visible')).toHaveCount(0);
        }
        if (tier === 'legendary') {
          await expect(page.locator('.moment-kicker')).toBeVisible();
          await expect(page.locator('.moment-kicker')).toHaveCSS('background-color', 'rgb(224, 189, 102)');
          await expect(page.locator('.moment-rays:visible')).toHaveCount(0);
        }
      }
      if (tier === 'legendary') await expect(page.locator('button.moment-effect')).toHaveCount(0);
      await page.screenshot({ path: path.join(art, `${look}-${width}-${tier}.png`) });
    }
  });
}

for (const width of [1440]) {
  test(`${width}: the report opens after the first card while the remaining cards keep their own space`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await holdCardsForLayout(page);
    await inject(page, 'notable', 5);
    await page.evaluate(async () => {
      const file = '/src/store.ts'; (await import(/* @vite-ignore */ file)).useGame.setState({ reportOpen: true });
    });
    await expect(page.locator('.report')).toHaveCount(0);
    await page.getByRole('button', { name: 'Dismiss celebration' }).click();
    await expect(page.locator('.report')).toBeVisible();
    await expect(page.locator('.moment-card')).toBeVisible();
    await expect(page.locator('.moment-more')).toBeVisible();
    const lane = (await page.locator('.moment-lane').boundingBox())!;
    const report = (await page.locator('.report').boundingBox())!;
    expect(report.y).toBeGreaterThanOrEqual(lane.y + lane.height);
    expect(report.y + report.height).toBeLessThanOrEqual(900);
    await page.getByRole('button', { name: 'Dismiss celebration' }).click();
    await expect(page.locator('.report')).toBeVisible();
    await expect(page.locator('.moment-card')).toBeVisible();
  });
}

for (const look of ['atlas']) for (const reducedMotion of ['no-preference', 'reduce'] as const) {
  test(`${look} ${reducedMotion}: legendary keeps its city tile clear at 1440`, async ({ page }) => {
    await page.emulateMedia({ reducedMotion });
    await page.evaluate(async () => {
    });
    await inject(page, 'legendary');
    await expect(page.locator('.moment-card')).toBeVisible();
    await expect.poll(() => page.evaluate(async () => {
      const file = '/src/store.ts'; const s = (await import(/* @vite-ignore */ file)).useGame.getState();
      const at = s.report.moments[0][0].at;
      const map = (window as unknown as { legatumMap: { toScreen(h: typeof at): { x: number; y: number }; camera: { zoom: number } } }).legatumMap;
      const point = map.toScreen(at), canvas = document.querySelector('canvas')!.getBoundingClientRect();
      const x = point.x + canvas.x, y = point.y + canvas.y, radius = 36 * map.camera.zoom;
      const card = document.querySelector('.moment-card')!.getBoundingClientRect();
      return x - radius >= 0 && x + radius <= innerWidth && y - radius >= 0 && y + radius <= innerHeight &&
        !(x + radius > card.left && x - radius < card.right && y + radius > card.top && y - radius < card.bottom);
    })).toBe(true);
    await page.screenshot({ path: path.join(art, `${look}-1440-legendary-city-${reducedMotion}.png`) });
  });
}

for (const look of ['atlas']) {
  test(`${look}: compact major and legendary controls retain contrast`, async ({ page, browserName }) => {
    test.skip(browserName !== 'chromium', 'contrast audit needs CDP (Chromium only)');
    test.setTimeout(120_000);
    await page.setViewportSize({ width: 1000, height: 600 });
    await page.evaluate(async () => {
      // Hold only the fixture's lifetime while the screenshot-based contrast audit runs.
      const moments = '/src/celebrate/moments.ts'; const { DURATION } = await import(/* @vite-ignore */ moments);
      DURATION.major = DURATION.legendary = 120_000;
    });
    for (const tier of ['major', 'legendary']) {
      await inject(page, tier);
      await expect(page.locator('.moment-card')).toBeVisible();
      const readings = (await auditContrast(page, `${look} ${tier}`)).filter((r) => r.where.includes('moment-'));
      expect(readings.length).toBeGreaterThan(0);
      expect(readings.filter((r) => r.ratio < r.required)).toEqual([]);
    }
  });
}

test('a real first-turn ruin discovery fires discovery on the first game screen', async ({ page }) => {
  const scene = explorationFixture();
  scene.setup.seed = '1'; scene.ruin.fill(0);
  const at = fromOffset(4, 4); scene.ruin[tileIndex(at, scene.width, scene.height)] = 1;
  const game = explorationCoreSave(scene);
  await page.evaluate(async ({ game, at }) => {
    const store = '/src/store.ts';
    const { useGame } = await import(/* @vite-ignore */ store);
    await useGame.getState().loadSave({ format: 'legatum-save', version: 1, client: 'wasm', game, savedAt: new Date().toISOString(), turn: 1, names: ['Explorer'] });
    useGame.getState().editOrders(() => [{ type: 'MoveUnit', unit: 0, to: at }]);
    window.__feelLog = [];
    useGame.getState().toggleReady(0);
  }, { game, at });
  // The same turn can also claim a title (a major card), which then plays first: the report is
  // held only behind the first card, so assert the hold there and find the ruin in either place.
  await expect(page.locator('.moment-card')).toBeVisible();
  await expect(page.getByRole('complementary', { name: 'Turn report' })).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => window.__feelLog?.filter((e) => e.name === 'discovery').length)).toBe(1);
  await page.getByRole('button', { name: 'Dismiss celebration' }).click();
  const report = page.getByRole('complementary', { name: 'Turn report' });
  await expect(report).toBeVisible();
  await expect(report).toContainText('explored an ancient ruin: +40 Gold');
});

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  {
    test(`${width}x${height}: a cleared barbarian camp celebrates like a ruin`, async ({ page }) => {
      await page.setViewportSize({ width, height });
      await holdCardsForLayout(page);
      await injectCamp(page);
      await expect(page.locator('.moment-card')).toBeVisible();
      await expect(page.locator('.moment-card')).toContainText('The raiders’ camp falls');
      // The gains are listed in full with their icons, never behind a chevron.
      await expect(page.locator('.moment-gains li')).toHaveText(['+50 Gold']);
      await expect(page.locator('.moment-gains li .amt-icon')).toHaveCount(1);
      await expect(page.locator('.moment-chevron')).toHaveCount(0);
      await expect(page.getByRole('button', { name: 'Show on map', exact: true })).toBeVisible();
      await expect(page.getByRole('button', { name: 'Turn report', exact: true })).toBeVisible();
      await page.screenshot({ path: path.join(art, `camp-${width}x${height}.png`) });
    });
  }
}
