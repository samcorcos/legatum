// A unique piece always says how it differs from what it replaces (user, 2026-10-17: "it's not clear to me why this
// is better than what it replaces"). The faction picker's sheet at the user's laptop size and a large one: every
// faction's piece card shows the standard item's numbers beside its own, fits without pushing the sheet into a
// scroll, and keeps its text inside the card. Screenshots of two factions go to PIECE_SHOTS (default test-results/piece-compare).
import { test as base, expect } from '@playwright/test';
import { createServer } from 'vite';
import { mkdir } from 'node:fs/promises';
import path from 'node:path';
import { FACTIONS, FACTION_INFO } from '../src/data';

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

// What each faction's card must say about its piece (the standard item's number, or the plain note).
const SAYS: Record<(typeof FACTIONS)[number], string[]> = {
  Aztecs: ['Eagle Warrior replaces Spearman', '(Spearman: 20)'],
  Switzerland: ['(Walls: 100)', '(Walls: +10)', '+50% of its strength against mounted units'],
  India: ['Revenue Office replaces Market', '+3 Gold', 'half the Gold the city makes', 'War Elephant replaces Knight', '(Knight: 42)', 'only India can build it'],
  Rome: ['50 HP'],
  Byzantium: ['(Pikeman: 152)', '(Pikeman: 32)'],
  Germany: ['(Forge: +3)', '(Forge: 0)', 'Panzer replaces Tank', '(Tank: 5)'],
  Celts: ['Gaesatae replaces Phalangite', '+8 strength when attacking.', '−5 strength when defending.'],
  Mongols: ['(Cataphract: 150)', '(Cataphract: 0)', 'Shoots or charges each turn, never both', 'Yam Relay Station only Mongols can build it'],
  Sparta: ['+5 strength next to another Hoplite (the phalanx).', 'It is lost on upgrade.', 'Agoge only Sparta can build it', 'start one level higher'],
  Athens: ['(Library: 80)', '(Library: +3)', 'Peltast replaces Composite Archer', '(Composite Archer: 2)'],
  Egypt: ['Royal Granary replaces Granary', 'Medjay replaces Spearman'],
};

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`every faction's piece card compares with what it replaces, at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(120_000);
    await mkdir(out, { recursive: true });
    await page.setViewportSize(size);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await page.evaluate(() => document.fonts.ready.then(() => undefined));
    for (const faction of FACTIONS) {
      const name = FACTION_INFO[faction].name;
      await page.getByRole('option', { name }).click();
      const sheet = stage.getByTestId('faction-sheet');
      await expect(sheet.locator('.fs-sheet-name')).toContainText(name);
      await page.mouse.move(size.width - 2, size.height - 2);
      await page.waitForTimeout(300);
      const card = sheet.locator('.pc');
      await expect(card.first()).toBeVisible();
      const text = (await card.allInnerTexts()).join(' ').replace(/\s+/g, ' ');
      for (const part of SAYS[faction]) expect(text, `${faction}: ${part}`).toContain(part);
      // Every card shows its painting, keeps its text inside, and the sheet still fits without scrolling.
      expect(await card.locator('.pc-icon.painted img').count(), `${faction} cards without painted art`).toBe(await card.count());
      const over = await card.evaluateAll((els) => els.flatMap((el) => [...el.querySelectorAll<HTMLElement>('.pc-head, .pc-stats, .pc-stat, .pc-vs, .pc-text')].filter((c) => c.scrollWidth > c.clientWidth + 1).map((c) => c.className)));
      expect(over, `${faction} card overflows sideways`).toEqual([]);
      const scroll = await sheet.evaluate((el) => el.scrollHeight - el.clientHeight);
      expect(scroll, `${faction} sheet scrolls by ${scroll}px`).toBeLessThanOrEqual(0);
      // Arrow and colour: a number marked better or worse carries an arrow icon too.
      const marked = await card.locator('.pc-stat[data-diff]').count();
      const arrows = await card.locator('.pc-stat[data-diff] .ui-mark').count();
      expect(arrows).toBe(marked);
      if (faction === 'Switzerland' || faction === 'Celts' || faction === 'Sparta') {
        await page.screenshot({ path: path.join(out, `sheet-${faction.toLowerCase()}-${size.width}.png`) });
        await card.first().screenshot({ path: path.join(out, `piece-${faction.toLowerCase()}-${size.width}.png`) });
      }
    }
  });
}
