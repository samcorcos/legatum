import { mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { test, expect } from './fixtures';
import { TECHS, TECH_ERA, eraTechs } from '../src/data';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';

const sizes = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const eras = ['ancient', 'classical', 'medieval', 'renaissance', 'industrial', 'modern', 'future'];
const shots = process.env.TECH_CARD_SHOTS ?? '../docs/design/tech-cards/final';

for (const size of sizes) for (const [era, name] of eras.entries()) {
  test(`${name} benefit ledger at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(90_000);
    await page.setViewportSize(size);
    const f = researchFixture('India');
    f.players[0].techs = TECHS.filter((t) => TECH_ERA[t] < era);
    f.players[0].research = null as never;
    f.players[0].research_queue = [];
    const game = coreFixture(f);
    await page.addInitScript((game) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: '2026-10-26T12:00:00Z', game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, game);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await page.locator('.topbar .research-chip').click();
    const dialog = page.locator('dialog.tech-modal');
    await expect(dialog.locator('.era-card')).toHaveCount(5);
    await expect(dialog.locator('.benefit-ledger')).toHaveCount(5);
    await expect(dialog.locator('.era-card.stepping')).toHaveCount(0);
    await expect(dialog.getByLabel('Tech card design')).toHaveCount(0);
    await expect(dialog.locator('.research-scroll > .era-current + .era-objectives-research')).toHaveCount(1);
    await page.mouse.move(2, 2);
    await page.evaluate(async () => {
      await document.fonts.ready;
      await Promise.all([...document.querySelectorAll<HTMLImageElement>('dialog img')].map((img) => img.decode()));
    });
    if (['medieval', 'renaissance', 'industrial', 'modern', 'future'].includes(name)) {
      await mkdir(resolve(shots), { recursive: true });
      await page.screenshot({ path: resolve(shots, `after-${name}-${size.width}x${size.height}.png`), animations: 'disabled' });
    }
    const geometry = await dialog.evaluate((el) => {
      const scroll = el.querySelector('.research-scroll')!.getBoundingClientRect();
      const cards = [...el.querySelectorAll<HTMLElement>('.era-card')];
      const rows = [...el.querySelectorAll<HTMLElement>('.era-unlock-label, .ledger-upgrade')];
      return {
        tops: [...new Set(cards.map((c) => Math.round(c.getBoundingClientRect().top)))],
        clipped: cards.filter((c) => { const b = c.getBoundingClientRect(); return b.left < scroll.left || b.right > scroll.right || b.bottom > scroll.bottom + 1; }).map((c) => c.dataset.tech),
        overflow: cards.filter((c) => c.scrollWidth > c.clientWidth + 1).map((c) => c.dataset.tech),
        overflowParts: cards.flatMap((c) => [...c.querySelectorAll<HTMLElement>('*')].filter((e) => e.scrollWidth > e.clientWidth + 1 && getComputedStyle(e).overflowX === 'visible').map((e) => `${c.dataset.tech}: ${e.className}`)),
        multiline: rows.filter((r) => {
          const style = getComputedStyle(r);
          const contentHeight = r.getBoundingClientRect().height - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom) - parseFloat(style.borderTopWidth) - parseFloat(style.borderBottomWidth);
          // Two lines for a rule; the merged Stealth upgrade sentence the designer worded may take a third.
          return contentHeight > (/can upgrade to their/.test(r.textContent ?? '') ? 3 : 2) * parseFloat(style.lineHeight) + 1;
        }).map((r) => r.textContent),
        // A unit's subline (strength, reach, supply) wraps whole: never clipped, never an ellipsis.
        clippedUnitLines: [...el.querySelectorAll<HTMLElement>('.era-unlock-gain.unit-line, .era-unlock-gain.unit-line > span')].filter((r) => r.scrollWidth > r.clientWidth + 1 || getComputedStyle(r).textOverflow === 'ellipsis').map((r) => r.textContent),
        truncatedItems: [...el.querySelectorAll<HTMLElement>('.era-unlock:not(.effect) .era-unlock-label')].filter((r) => r.scrollWidth > r.clientWidth + 1).map((r) => r.textContent),
        benefits: cards.map((c) => c.querySelectorAll('.era-unlock').length),
      };
    });
    expect(geometry.tops).toHaveLength(1);
    expect(geometry.clipped, 'all five complete cards in view at scroll zero').toEqual([]);
    expect(geometry.overflow, `overflowing descendants: ${geometry.overflowParts.join(', ')}`).toEqual([]);
    expect(geometry.multiline, 'at most two lines per effect and upgrade (three for the merged Stealth upgrade sentence): the deciding number wraps, it is never cut off').toEqual([]);
    expect(geometry.clippedUnitLines, 'unit sublines wrap, they are never cut off').toEqual([]);
    expect(geometry.truncatedItems, 'complete building and unit names, including the longest names').toEqual([]);
    expect(geometry.benefits.filter((n) => n === 0)).toHaveLength(0);
    if (name === 'classical') expect(geometry.benefits.some((n) => n >= 3), 'Iron Working supplies the three-benefit stress case').toBe(true);
    if (name === 'medieval') {
      const feudalism = dialog.locator('.era-card[data-tech="Feudalism"]');
      await expect(feudalism.locator('.era-unlock.unit')).toHaveCount(1);
      await expect(feudalism.locator('.ledger-upgrade')).toContainText('From Phalangite');
      const machinery = dialog.locator('.era-card[data-tech="Machinery"] .benefit-ledger');
      const education = dialog.locator('.era-card[data-tech="Education"] .benefit-ledger');
      const wonder = machinery.locator('.era-unlock').filter({ hasText: 'Hagia Sophia' });
      await expect(wonder).toContainText('World wonder');
      await expect(education.locator('.era-unlock').filter({ hasText: 'University' })).toHaveCount(1);
      await expect(wonder.locator('.era-unlock-art img')).toHaveCount(1);
      await expect(wonder.locator('.era-unlock-art .mask-icon'), 'Hagia Sophia is painted, not an engraved icon').toHaveCount(0);
    }
    if (name === 'future') {
      // A rule that only describes a unit this card unlocks lives on that unit's row, so Stealth Technology is two units (with their upgrade paths) and one rule.
      const stealth = dialog.locator('.era-card[data-tech="StealthTechnology"] .era-unlock');
      await expect(stealth).toHaveCount(3);
      await expect(stealth.nth(0)).toContainText('Stealth Fighter');
      await expect(stealth.nth(0)).toContainText('strikes 7 tiles, guards 5 · uses Silicon');
      await expect(stealth.nth(1)).toContainText('strikes 10 tiles; only Stealth Fighters stop it · uses Silicon');
      await expect(stealth.nth(0)).toContainText('From Fighter');
      await expect(stealth.nth(2)).toContainText('Hangars hold 3 aircraft');
      await expect(dialog.locator('.era-card[data-tech="StealthTechnology"]')).not.toContainText('can upgrade');
    }
    // The ledger remains an active research UI, with detail, queue, and keyboard pick actions.
    const [first, second] = eraTechs(era);
    await dialog.locator(`.era-card[data-tech="${first}"]`).getByTestId('queue-add').click();
    await expect(dialog.locator('.research-queue li')).toHaveCount(1);
    const secondCard = dialog.locator(`.era-card[data-tech="${second}"]`);
    await secondCard.focus();
    await page.keyboard.press('Enter');
    await expect(secondCard).toHaveClass(/researching/);
    await secondCard.locator('.era-details-chevron').click();
    await expect(dialog.locator('.research-detail')).toBeVisible();
  });
}
