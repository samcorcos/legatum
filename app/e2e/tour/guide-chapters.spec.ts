// Screen tour, the Guide's chapters that carry rules text for factions, combat, diplomacy and the victory race.
// Each chapter's text is also written to test-results/guide-text/<size>/<chapter>.txt for the copy review.
// Run alone with `npx playwright test e2e/tour/guide-chapters`. Shots land in test-results/screen-tour/<size>/guide-chapters/.
import { mkdir, writeFile } from 'node:fs/promises';
import { test, expect } from '../fixtures';
import { SIZES, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

const CHAPTERS = [
  { id: 'combat', button: /Simultaneous combat/, scrolls: ['Three classes, one triangle', 'Veterans', 'Take cities with the melee line'] },
  { id: 'race', button: /The victory race/, scrolls: ['Original capitals', 'World wonders', 'Diplomacy'] },
  { id: 'peoples', button: /The peoples/, scrolls: ['Byzantium', 'Mongols', 'Sparta'] },
  { id: 'turns', button: /Your first turns/, scrolls: ['Diplomacy'] },
  { id: 'cities', button: /Cities & research/, scrolls: ['Growth and production'] },
  { id: 'controls', button: /Controls & saves/, scrolls: ['Save and resume'] },
] as const;

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the Guide: combat, victory race and peoples chapters', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await expect(guide).toBeVisible();
    const dir = `test-results/guide-text/${size.name}`;
    await mkdir(dir, { recursive: true });
    for (const chapter of CHAPTERS) {
      await guide.getByRole('button', { name: chapter.button }).click();
      await writeFile(`${dir}/${chapter.id}.txt`, await guide.locator('.guide-page').innerText());
      if (chapter.id === 'combat' || chapter.id === 'race') {
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, `${chapter.id}-0-top`);
      }
      let i = 0;
      for (const heading of chapter.scrolls) {
        const h = guide.locator('.guide-page').getByRole('heading', { name: new RegExp(`^${heading}`) }).first();
        if (await h.count() === 0) continue;
        await h.scrollIntoViewIfNeeded();
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, `${chapter.id}-${++i}-${heading.toLowerCase().replace(/[^a-z]+/g, '-')}`);
      }
      if (chapter.id === 'race') {
        await guide.locator('.guide-page .guide-note').last().scrollIntoViewIfNeeded();
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, 'race-score-note');
      }
      if (chapter.id === 'turns') {
        const peace = guide.locator('.guide-page p', { hasText: 'A Peace offer is between two teams.' });
        await peace.scrollIntoViewIfNeeded();
        await expect(peace).toBeVisible();
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, 'turns-peace-rule');
      }
    }
  });
});
