// Every reward kind wears its own engraved emblem, not the player's faction crest (art pass 2).
// Also the tour that shoots each card at 1000x600 and 1440x900: REWARD_SHOTS=<dir> picks where.
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { auditLayout } from './layoutAudit';

// REWARD_BEFORE=1 only shoots (the pre-change baseline): no emblem or layout assertions.
const baseline = process.env.REWARD_BEFORE === '1';
const shots = process.env.REWARD_SHOTS ? path.resolve(process.env.REWARD_SHOTS) : path.resolve('../.progress/art/rewards');
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      // A cold dev server transforms the app and optimises its dependencies on the first request; under a full suite on a
      // busy machine that outlasted the first navigation's wait ("Single Player" not found: ruin-gold, title, captured
      // cards failed in archived runs while the same cards passed alone). Do that work before any test navigates, and
      // import the fixture module here too, so no test pays for it.
      await server.warmupRequest('/src/main.tsx');
      await server.warmupRequest('/src/test/celebrationFixture.ts');
      await server.waitForRequestsIdle();
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('No Vite address');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

/** kind -> the emblem its card must wear (null: the faction crest stays). */
const EXPECTED: Record<string, string | null> = {
  camp: 'camp', 'ruin-gold': 'ruin', 'ruin-science': 'ruin', 'ruin-population': 'ruin', 'ruin-scout': 'ruin',
  captured: 'captured', lost: 'captured', founded: 'founded', growth: 'growth',
  greatwork: 'greatwork', goal: null, 'tech-partner': 'tech',
};
const VIGNETTES: Record<string, string> = {
  camp: 'camp-cleared', 'ruin-gold': 'ruin-explored', 'ruin-science': 'ruin-explored',
  'ruin-population': 'ruin-explored', 'ruin-scout': 'ruin-explored', captured: 'captured', lost: 'lost',
  founded: 'founded', growth: 'growth',
  greatwork: 'greatwork', 'tech-partner': 'tech',
};

test.beforeEach(async ({ page }) => {
  test.setTimeout(90_000);
  await page.addInitScript(() => {
    if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
  await page.goto('/?client=wasm');
  const solo = page.getByRole('button', { name: /^Single Player/ });
  // Retry one blank initial Vite navigation; a persistent load failure still fails the test.
  try { await expect(solo).toBeVisible({ timeout: 15_000 }); }
  catch { await page.reload(); await expect(solo).toBeVisible({ timeout: 30_000 }); }
  await page.evaluate(() => document.fonts.ready);
  await mkdir(shots, { recursive: true });
});

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test.describe(`${width}x${height}`, () => {
    test.use({ viewport: { width, height } });
    for (const kind of Object.keys(EXPECTED)) {
      test(`${kind} card`, async ({ page }) => {
        const errors: string[] = []; page.on('pageerror', (e) => errors.push(e.message));
        await page.evaluate(async (kind) => {
          const file = '/src/test/celebrationFixture.ts';
          const { rewardFixture } = await import(/* @vite-ignore */ file);
          // Cards dismiss themselves (minor ones after 1 s), and this test measures and shoots them for longer than that, so
          // on a busy machine a minor card (tech-partner, 1 of 140 runs) was gone before its layout was read. Pin them.
          const moments = '/src/celebrate/moments.ts'; const { DURATION } = await import(/* @vite-ignore */ moments);
          for (const tier of Object.keys(DURATION)) DURATION[tier] = 120_000;
          await rewardFixture(kind);
        }, kind);
        const card = page.locator('.moment-card');
        await expect(card).toBeVisible();
        await page.waitForTimeout(900);
        const want = EXPECTED[kind];
        if (baseline) {
          await card.screenshot({ path: path.join(shots, `${kind}-${width}x${height}.png`) });
          return;
        }
        if (want) await expect(card.locator(`[data-emblem="${want}"]`)).toBeVisible();
        else await expect(card.locator('[data-emblem]')).toHaveCount(0);
        const vignette = card.locator('.moment-vignette');
        if (VIGNETTES[kind]) {
          await expect(vignette).toHaveAttribute('src', new RegExp(`moment-${VIGNETTES[kind]}-104\\.png`));
          await expect(vignette).toBeVisible();
          expect(await vignette.evaluate((image: HTMLImageElement) => image.naturalWidth)).toBe(104);
          expect(await vignette.evaluate((image) => image.getBoundingClientRect().width)).toBe(104);
        } else await expect(vignette).toHaveCount(0);
        if (kind === 'lost') {
          await expect(card).toHaveClass(/\bloss\b/);
          await expect(card.locator('.moment-kicker')).toHaveText('A city is lost');
          await expect(card).toHaveCSS('border-left-width', '6px');
        }
        // Gains are listed in full, never behind a chevron.
        await expect(card.locator('.moment-chevron')).toHaveCount(0);
        await card.screenshot({ path: path.join(shots, `${kind}-${width}x${height}.png`) });
        await page.screenshot({ path: path.join(shots, `full-${kind}-${width}x${height}.png`) });
        // The whole card shows: it never scrolls inside its cap, and both buttons sit inside it and the window.
        const fit = await card.evaluate((el) => {
          const box = el.getBoundingClientRect();
          const buttons = [...el.querySelectorAll('footer button')].map((b) => b.getBoundingClientRect());
          return { scrolls: el.scrollHeight - el.clientHeight, buttons: buttons.length,
            clipped: buttons.filter((b) => b.bottom > box.bottom + 0.5 || b.bottom > window.innerHeight).length };
        });
        expect(fit.buttons).toBe(2);
        expect(fit.scrolls).toBeLessThanOrEqual(1);
        expect(fit.clipped).toBe(0);
        const issues = (await auditLayout(page, { minTarget: 24 })).filter((i) => /moment-card|moment-lane/.test(i.where) && i.kind !== 'overlap' && !/sr-only/.test(i.where));
        expect(issues).toEqual([]);
        expect(errors).toEqual([]);
      });
    }
    test('own tech: the unlock card', async ({ page }) => {
      await page.evaluate(async () => {
        const file = '/src/test/celebrationFixture.ts';
        const { unlockCelebrationFixture } = await import(/* @vite-ignore */ file);
        await unlockCelebrationFixture(true, true, 0);
      });
      const card = page.locator('.moment-card.unlock');
      await expect(card).toBeVisible();
      await page.waitForTimeout(900);
      await card.screenshot({ path: path.join(shots, `tech-own-${width}x${height}.png`) });
    });
  });
}
