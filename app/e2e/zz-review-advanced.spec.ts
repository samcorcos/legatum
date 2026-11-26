// TEMPORARY review spec (legatum-ui-reviewer, PR 296). Not committed.
import { mkdir, writeFile } from 'node:fs/promises';
import type { Page } from '@playwright/test';
import { test, expect, testWithFailingRequests } from './fixtures';

test.use({ launchOptions: { executablePath: '/opt/pw-browsers/chromium-1243/chrome-linux64/chrome' } });
testWithFailingRequests.use({ launchOptions: { executablePath: '/opt/pw-browsers/chromium-1243/chrome-linux64/chrome' } });
const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const OUT = '/mnt/project-files/setup-simplify/';
const log: Record<string, unknown> = {};
async function snap(page: Page, size: string, name: string) {
  await mkdir(`${OUT}${size}/`, { recursive: true });
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}${size}/${name}.png`, animations: 'disabled' });
  await page.screenshot({ path: `${OUT}${size}/${name}-full.png`, fullPage: true, animations: 'disabled' });
}
const noHints = (p: Page) => p.addInitScript(() => { try { localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })); } catch {} });
const isOpen = (p: Page) => p.locator('details.lobby-advanced').evaluate((d) => (d as HTMLDetailsElement).open);
const summaryStyle = (p: Page) => p.locator('details.lobby-advanced > summary').evaluate((s) => {
  const cs = getComputedStyle(s); const r = s.getBoundingClientRect();
  const before = getComputedStyle(s, '::marker');
  return { cursor: cs.cursor, color: cs.color, display: cs.display, listStyle: cs.listStyleType, marker: before.content, h: r.height, w: r.width, y: r.y, text: s.textContent };
});

for (const size of SIZES) test.describe(`review ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  test('solo lobby advanced', async ({ page }) => {
    test.setTimeout(120_000);
    await noHints(page);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
    await page.mouse.move(5, 5);
    await snap(page, size.name, '01-solo-top');
    const sum = page.locator('details.lobby-advanced > summary');
    await sum.scrollIntoViewIfNeeded();
    await page.mouse.move(5, 5);
    log[`${size.name}-closed`] = await summaryStyle(page);
    await snap(page, size.name, '02-solo-advanced-closed');
    await sum.hover();
    await snap(page, size.name, '02b-solo-advanced-closed-hover');
    await page.keyboard.press('Tab');
    await sum.focus();
    await snap(page, size.name, '02c-solo-advanced-closed-focus');
    const before = await page.getByRole('button', { name: /^Start game/ }).boundingBox();
    await sum.click();
    expect(await isOpen(page)).toBe(true);
    await page.mouse.move(5, 5);
    await sum.scrollIntoViewIfNeeded();
    const after = await page.getByRole('button', { name: /^Start game/ }).boundingBox();
    log[`${size.name}-startMove`] = { before, after };
    await snap(page, size.name, '03-solo-advanced-open');
    await page.getByRole('radiogroup', { name: 'Coach questions' }).getByRole('radio', { name: /^On/ }).click();
    await page.mouse.move(5, 5);
    await snap(page, size.name, '04-solo-advanced-coach-on');
    await sum.click();
    await page.mouse.move(5, 5);
    log[`${size.name}-coachSummary`] = await summaryStyle(page);
    await snap(page, size.name, '05-solo-advanced-closed-coach-on');
    // Coach back off, then the invalid-seed path.
    await sum.click();
    await page.getByRole('radiogroup', { name: 'Coach questions' }).getByRole('radio', { name: /^Off/ }).click();
    const seed = page.getByLabel('Map seed', { exact: true });
    await seed.fill('');
    log[`${size.name}-openAfterClear`] = await isOpen(page);
    await seed.pressSequentially('12a');
    log[`${size.name}-openAfterBadSeed`] = await isOpen(page);
    await page.mouse.move(5, 5);
    await snap(page, size.name, '06-solo-seed-invalid');
    // Fix it by typing: does the section stay open while you type?
    await seed.fill('');
    await seed.pressSequentially('4');
    log[`${size.name}-openAfterTypingValid`] = await isOpen(page);
    log[`${size.name}-focusAfterTypingValid`] = await page.evaluate(() => document.activeElement?.getAttribute('aria-label'));
    await page.mouse.move(5, 5);
    await snap(page, size.name, '07-solo-seed-fixed-by-typing');
    // Closed with an invalid seed (set through the store path: reload not needed, type invalid while closed impossible).
    await writeFile(`${OUT}${size.name}/log.json`, JSON.stringify(log, null, 2));
  });
});

for (const size of SIZES) testWithFailingRequests.describe(`review online ${size.name}`, () => {
  testWithFailingRequests.use({ viewport: { width: size.width, height: size.height } });
  testWithFailingRequests('host and guest', async ({ page: host, browser, baseURL, relay }) => {
    testWithFailingRequests.setTimeout(240_000);
    expect(relay).toBeTruthy();
    await noHints(host);
    await host.goto('/');
    await host.getByRole('button', { name: /^Multiplayer/ }).click();
    await host.getByLabel('Your name', { exact: true }).fill(`Host ${String(Date.now()).slice(-5)}`);
    const created = host.waitForResponse((r) => r.url().endsWith('/rooms') && r.request().method() === 'POST');
    await host.getByRole('button', { name: /^Host Game/ }).click();
    const listedName = ((await (await created).json()) as { name: string }).name;
    await expect(host.getByRole('heading', { name: 'Game room', exact: true })).toBeVisible();
    await host.mouse.move(5, 5);
    await snap(host, size.name, '10-host-room');
    const sum = host.locator('details.lobby-advanced > summary');
    await sum.scrollIntoViewIfNeeded();
    await host.mouse.move(5, 5);
    await snap(host, size.name, '11-host-advanced-closed');
    await sum.click();
    await host.mouse.move(5, 5);
    await sum.scrollIntoViewIfNeeded();
    await snap(host, size.name, '12-host-advanced-open');
    const g = await browser.newContext({ baseURL, viewport: { width: size.width, height: size.height } });
    const guest = await g.newPage();
    await noHints(guest);
    await guest.goto('/');
    await guest.getByRole('button', { name: /^Multiplayer/ }).click();
    await guest.getByRole('button', { name: /^Join Game/ }).click();
    const row = guest.locator('.online-room').filter({ hasText: listedName });
    await expect(row).toHaveCount(1, { timeout: 20_000 });
    await row.getByRole('button', { name: /^Join/ }).click();
    await expect(guest.getByRole('heading', { name: 'Game room', exact: true })).toBeVisible();
    await guest.mouse.move(5, 5);
    await snap(guest, size.name, '20-guest-room');
    const gs = guest.locator('details.lobby-advanced > summary');
    log[`${size.name}-guestAdvanced`] = await gs.count();
    if (await gs.count()) {
      await gs.scrollIntoViewIfNeeded();
      await guest.mouse.move(5, 5);
      await snap(guest, size.name, '21-guest-advanced-closed');
      await gs.click();
      await guest.mouse.move(5, 5);
      await gs.scrollIntoViewIfNeeded();
      await snap(guest, size.name, '22-guest-advanced-open');
    }
    await writeFile(`${OUT}${size.name}/log-online.json`, JSON.stringify(log, null, 2));
    await g.close();
  });
});
