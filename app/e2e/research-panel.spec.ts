import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { auditContrast } from './contrast';
import { chooseCapitalProduction } from './opening';

const sizes = [
  { width: 1000, height: 600 },
  { width: 1440, height: 900 },
];

/** The playtest-6 state: one technology researching and one queued behind it, with the
 * dialog reopened the way a player returns to it. */
async function openResearchWithQueue(page: Page) {
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
  const dismiss = page.getByRole('button', { name: /^Dismiss hint/ });
  if (await dismiss.count()) await dismiss.click();
  await chooseCapitalProduction(page);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2', { timeout: 60_000 });
  const celebration = page.getByRole('button', { name: 'Dismiss celebration', exact: true });
  if (await celebration.count()) await celebration.click();
  const report = page.getByRole('button', { name: 'Close report', exact: true });
  if (await report.count()) await report.click();
  const dialog = page.locator('dialog.tech-modal');
  const open = async () => {
    await page.locator('.topbar .research-chip').click();
    await expect(dialog).toBeVisible();
  };
  await open();
  // Research Bronze Working now and queue Masonry behind it.
  await dialog.locator('[data-tech="BronzeWorking"]').getByTestId('research-now').click();
  await dialog.locator('[data-tech="Masonry"]').getByTestId('queue-add').click();
  await expect(dialog.locator('.research-queue li')).toHaveCount(2);
  await dialog.getByRole('button', { name: 'Close research', exact: true }).click();
  await expect(dialog).toHaveCount(0);
  await open();
  // Let the entrance fade finish before measuring pixels.
  await page.waitForTimeout(300);
  return dialog;
}

for (const size of sizes) test.describe(`${size.width}×${size.height}`, () => {
  test.use({ viewport: size });

  test('the current card is legible; unlocks show icons; every card is whole and clear of the queue', async ({ page }, info) => {
    test.setTimeout(180_000);
    const dialog = await openResearchWithQueue(page);
    const shots = process.env.TECHPANEL_SHOTS;
    const shot = (name: string) => page.screenshot({ path: shots ? `${shots}/${name}-${size.width}x${size.height}.png` : info.outputPath(`${name}-${size.width}x${size.height}.png`) });
    await shot('research');

    // The current research: highlighted, ribboned with its turns, with progress, and legible.
    const current = dialog.locator('.era-card.researching');
    await expect(current).toHaveCount(1);
    await expect(current).toHaveAttribute('data-tech', 'BronzeWorking');
    await expect(current.locator('.era-now')).toHaveText(/^Researching · \d+ turns?$/);
    await expect(current.locator('.era-progress')).toBeVisible();
    await expect(current.getByTestId('research-order-badge')).toHaveCount(0);
    await expect(current.getByTestId('research-now')).toHaveCount(0);
    const readings = (await auditContrast(page, 'research')).filter((r) => r.where.includes('"Bronze Working') && r.text.includes('era-card-title') && r.state === 'default');
    expect(readings.length, 'the current card title was measured').toBeGreaterThan(0);
    for (const r of readings) expect(r.ratio, `${r.where} ${r.text}: ${r.fg} on ${r.bg}`).toBeGreaterThanOrEqual(r.required);

    // Every unlock (unit, building and effect) shows its painted miniature named beside it (its hovercard has the rest; e2e/unlock-hover.spec.ts).
    const workshop = dialog.locator('[data-tech="BronzeWorking"] .era-unlock');
    await expect(workshop).toHaveCount(1);
    await expect(workshop.locator('.era-unlock-art img')).toBeVisible();
    expect(await workshop.locator('.era-unlock-art img').evaluate((img: HTMLImageElement) => img.naturalWidth), 'the painting loaded').toBeGreaterThan(0);
    await expect(workshop).toContainText('Workshop');
    await expect(dialog.locator('[data-tech="Irrigation"] .era-unlock.effect')).toContainText('+1');

    // Every card of the era is whole inside the dialog, and the five fit one row at both sizes.
    const geometry = await dialog.evaluate((el) => {
      const box = el.getBoundingClientRect();
      const cards = [...el.querySelectorAll<HTMLElement>('.era-card')].map((c) => c.getBoundingClientRect());
      return { inside: cards.every((b) => b.left >= box.left && b.right <= box.right), rows: new Set(cards.map((b) => Math.round(b.top))).size };
    });
    expect(geometry.inside).toBe(true);
    expect(geometry.rows, 'five cards in one row').toBe(1);
    for (const card of await dialog.locator('.era-card').all()) {
      await card.scrollIntoViewIfNeeded();
      await expect(card.locator('.era-unlock :is(.icon, .era-unlock-art img, .uc-figure)').first()).toBeVisible();
      await expect(card.getByTestId('era-cost')).toBeVisible();
    }
  });
});

test('Mongols: Masonry lists the Walls the Mongols raise from the Medieval era', async ({ page }) => {
  // Until 2026-10-19 Masonry gave the Mongols nothing (they could not build Walls). A tech that gives a faction nothing is now skipped
  // (known from the start), so no card ever says it does nothing.
  const { researchFixture } = await import('../src/test/researchFixture');
  const { coreFixture } = await import('../src/test/coreFixture');
  await page.setViewportSize({ width: 1000, height: 600 });
  const game = coreFixture(researchFixture('Mongols'));
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.locator('.topbar .research-chip').click();
  const dialog = page.locator('dialog.tech-modal');
  const masonry = dialog.locator('[data-tech="Masonry"]');
  await masonry.scrollIntoViewIfNeeded();
  await expect(masonry).not.toContainText('No benefit');
  await expect(masonry).toContainText('Walls for the Mongols from the Medieval era');
  await expect(dialog).not.toContainText('No benefit');
});
