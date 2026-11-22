// The build card says how a faction piece differs from the standard item it replaces ("vs Walls: +5 city strength,
// costs 20 more Production"), in a city's production list at the user's laptop size. Runs on the WASM engine.
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';

const out = process.env.PIECE_SHOTS ? path.resolve(process.env.PIECE_SHOTS) : path.resolve('test-results/piece-compare');
const test = base.extend<{}, { devURL: string }>({
  devURL: [async ({}, use) => {
    const server = await createServer({ server: { host: '127.0.0.1', port: 0, strictPort: false } });
    await server.listen();
    try {
      const address = server.httpServer!.address();
      if (!address || typeof address === 'string') throw new Error('Vite did not bind');
      await use(`http://127.0.0.1:${address.port}`);
    } finally { await server.close(); }
  }, { scope: 'worker' }],
  baseURL: async ({ devURL }, use) => { await use(devURL); },
});

const CASES = [
  { faction: 'Switzerland', item: 'b:Walls', says: 'vs Walls: +5 city strength costs 20 more Production' },
  { faction: 'Byzantium', item: 'u:Trooper', says: 'vs Pikeman: +4 melee costs 10 more Production' },
  { faction: 'Athens', item: 'b:Laboratory', says: 'vs Library: +3 Science costs 10 less Production' },
  { faction: 'Sparta', item: 'u:Soldier', says: 'vs Spearman: +5 strength next to another Hoplite (the phalanx) it is lost on upgrade' },
] as const;

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`production cards of faction pieces show what they change, at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(90_000);
    await mkdir(out, { recursive: true });
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.setViewportSize(size);
    await page.goto('/?client=wasm');
    await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
    await page.evaluate(() => document.fonts.ready);
    for (const c of CASES) {
      await page.evaluate(async (faction) => {
        const file = '/src/test/pieceFixture.ts';
        const { pieceFixture } = await import(/* @vite-ignore */ file);
        await pieceFixture(faction);
      }, c.faction);
      const card = page.locator(`.city-panel .production-choice[data-item="${c.item}"]`);
      await card.scrollIntoViewIfNeeded();
      const line = card.getByTestId('piece-vs');
      // A basic unit sits behind the production list's "Also: ..." disclosure (the Spartan Hoplite is the Spearman).
      if (!(await card.isVisible())) for (const also of await page.locator('.city-panel summary', { hasText: /^Also/ }).all()) await also.click();
      const more = card.locator('.production-more');
      if (!(await more.evaluate((el) => (el as HTMLDetailsElement).open))) await more.locator('summary').click();
      await card.scrollIntoViewIfNeeded();
      await expect(line).toBeVisible();
      expect((await line.innerText()).replace(/\s+/g, ' ')).toBe(c.says);
      // The card's text stays inside it.
      const over = await card.locator('.prod').evaluate((el) => el.scrollWidth - el.clientWidth);
      expect(over).toBeLessThanOrEqual(0);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(250);
      await page.screenshot({ path: path.join(out, `card-${c.faction.toLowerCase()}-${size.width}.png`) });
    }
  });
}

// The research screen's unlock hovercard says it too, and still fits the window.
for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`the unlock hovercard of a faction piece compares it, at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(90_000);
    await mkdir(out, { recursive: true });
    await page.addInitScript(() => {
      if (location.protocol.startsWith('http')) localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    });
    await page.setViewportSize(size);
    await page.goto('/?client=wasm');
    await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
    await page.evaluate(() => document.fonts.ready);
    for (const c of [{ faction: 'Switzerland', tech: 'Masonry', unlock: 'Alpine Redoubt', says: ['Replaces Walls', '120 Production (Walls: 100)', '+15 city strength (Walls: +10)'] },
      { faction: 'Athens', tech: 'Writing', unlock: 'Lyceum', says: ['Replaces Library', '70 Production (Library: 80)', '+6 Science (Library: +3)'] }]) {
      await page.evaluate(async (faction) => {
        const file = '/src/test/pieceFixture.ts';
        const { pieceFixture, setTechOpen } = await import(/* @vite-ignore */ file);
        await pieceFixture(faction);
        setTechOpen(true);
      }, c.faction);
      const dialog = page.locator('dialog.tech-modal');
      await expect(dialog).toBeVisible();
      const card = dialog.locator(`.era-card[data-tech="${c.tech}"]`);
      await card.scrollIntoViewIfNeeded();
      // Each fixture reopens the same modal. Move off the previous unlock so
      // pointerenter fires even when the next unlock occupies the same pixels.
      await page.mouse.move(0, 0);
      await card.locator('.era-unlock', { hasText: c.unlock }).hover();
      const hover = page.getByTestId('unlock-card');
      await expect(hover).toBeVisible();
      const piece = hover.getByTestId('uc-piece');
      await expect(piece).toBeVisible();
      const text = (await piece.innerText()).replace(/\s+/g, ' ');
      for (const part of c.says) expect(text.toLowerCase(), `${c.faction}: ${part}`).toContain(part.toLowerCase());
      const box = (await hover.boundingBox())!;
      expect(box.y, 'inside the window, top').toBeGreaterThanOrEqual(0);
      expect(box.y + box.height, 'inside the window, bottom').toBeLessThanOrEqual(size.height + 0.5);
      await page.screenshot({ path: path.join(out, `hover-${c.faction.toLowerCase()}-${size.width}.png`) });
      await page.keyboard.press('Escape');
      await page.evaluate(async () => {
        const file = '/src/test/pieceFixture.ts';
        const { setTechOpen } = await import(/* @vite-ignore */ file);
        setTechOpen(false);
      });
    }
  });
}
