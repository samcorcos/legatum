import { revealSidebar } from './sidebarAssertions';
import { endPlanning } from './turnActions';
import { chooseCapitalProduction } from './opening';
import { writeFile } from 'node:fs/promises';
import type { Page, TestInfo } from '@playwright/test';
import { test, expect } from './fixtures';
import { auditContrast, type ContrastReading } from './contrast';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';

// A5 (plans/playtest-feedback.md): hovering a button could wash out its label ("Found City" went
// nearly invisible). Every control on the title, Settings, lobby and game screens is measured from
// real pixels in its default, hover, focus-visible and active states (and disabled controls as
// they are), in Atlas, against WCAG AA: 4.5:1 for labels, 3:1 for large text.
//
// Disabled controls are exempt from WCAG 1.4.3, but a disabled label still says what is
// unavailable (and often why), so they are held to the same AA bar.

const look = 'Atlas';

function failures(readings: ContrastReading[]) {
  return readings
    .filter((r) => r.ratio < r.required)
    .map((r) => `${r.where} [${r.state}] ${r.text}: ${r.ratio} < ${r.required} (${r.fg} on ${r.bg})`);
}

async function report(testInfo: TestInfo, name: string, readings: ContrastReading[]) {
  const body = JSON.stringify(readings, null, 1);
  await testInfo.attach(`${name}.json`, { body, contentType: 'application/json' });
  // CONTRAST_OUT=<dir> keeps every reading, for the a11y contrast notes.
  if (process.env.CONTRAST_OUT) await writeFile(`${process.env.CONTRAST_OUT}/contrast-${name}.json`, body);
}

async function openSettings(page: Page) {
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content',
    await page.locator('html').evaluate((el) => getComputedStyle(el).getPropertyValue('--bg').trim()));
}

/**
 * Every button variant in the stylesheets, enabled and disabled, in its real container classes,
 * laid out on panel paper. The screen flows below only meet the variants a short game shows.
 */
const GALLERY = `
  <div class="title-menu" style="animation:none"><div class="title-actions">
    <button class="btn big primary">Single Player<span class="btn-sub">1 to 7 AI empires</span></button>
    <button class="btn big primary" disabled>Single Player<span class="btn-sub">1 to 7 AI empires</span></button>
    <button class="btn big">Multiplayer<span class="btn-sub">Hot seat or network</span></button>
    <button class="btn big" disabled>Continue<span class="btn-sub">No autosave yet</span></button>
    <button class="btn big quiet">Load Game…</button>
    <button class="btn big quiet" disabled>Quit</button>
  </div></div>
  <div class="cx-group">
    <span class="vp-label"><button class="relation-badge alliance">Alliance</button>
      <button class="relation-badge alliance" disabled>Alliance</button></span>
    <button class="btn">Button</button><button class="btn" disabled>Button</button>
    <button class="btn primary">Primary <span class="kbd">P</span></button><button class="btn primary" disabled>Primary <span class="kbd">P</span></button>
    <button class="btn quiet">Quiet</button><button class="btn quiet" disabled>Quiet</button>
    <button class="btn big">Big<span class="btn-sub">Sub line</span></button><button class="btn big" disabled>Big<span class="btn-sub">Sub line</span></button>
    <button class="btn big primary">Big primary<span class="btn-sub">Sub line</span></button><button class="btn big primary" disabled>Big primary<span class="btn-sub">Sub line</span></button>
    <button class="link">Link</button><button class="link" disabled>Link</button>
  </div>
  <div class="actions" style="width:420px">
    <span><button class="act primary">Found City <span class="kbd">B</span></button></span>
    <span><button class="act primary" disabled>Found City <span class="kbd">B</span></button></span>
    <span><button class="act primary active">Attack <span class="kbd">A</span></button></span>
    <span><button class="act">Hold <span class="kbd">F</span></button></span><span><button class="act" disabled>Upgrade <span class="act-sub">80 Gold</span></button></span>
    <span><button class="act active">Ranged</button></span><span><button class="act quiet">Skip <span class="kbd">Space</span></button></span>
    <span><button class="act quiet" disabled>Skip</button></span><span><button class="act danger quiet">Disband</button></span><span><button class="act danger" disabled>Disband</button></span>
  </div>
  <div class="cx-group">
    <button class="btn-buy">Buy 120</button><button class="btn-buy" disabled>Buy 120</button><button class="btn-buy active">Buying</button>
    <button class="btn-buy"><span class="cost-chip"><span class="cost-part"><span class="cost-amount">−120</span></span></span></button>
    <button class="btn-buy" disabled><span class="cost-chip"><span class="cost-part"><span class="cost-amount">−120</span></span></span></button>
  </div>
  <div class="cx-group">
    <button class="btn primary spends"><span class="cost-chip"><span class="cost-part"><span class="cost-amount">−10</span><span class="cost-after">16 → 6</span></span></span></button>
    <button class="btn spends"><span class="cost-chip debt"><span class="cost-part"><span class="cost-amount">−10</span><span class="cost-after">4 → −6, debt</span></span></span></button>
    <button class="btn spends" disabled><span class="cost-chip"><span class="cost-part"><span class="cost-amount">−10</span></span></span></button>
    <button class="research-frontier">Available now <span>3</span></button><button class="research-frontier" disabled>Available now <span>0</span></button>
    <div class="chips"><button class="chip">All</button><button class="chip on">Combat</button><button class="chip" disabled>Cities</button></div>
  </div>
  <div class="seg" style="width:300px"><button>Off</button><button class="on">On</button><button disabled>Never</button></div>
  <div class="seg small" style="width:300px"><button>Hidden</button><button class="on">Shown</button><button disabled>Never</button></div>
  <div class="segmented" style="width:360px"><button>Small<span class="seg-sub">2-4 empires</span></button><button class="on">Standard<span class="seg-sub">5-8 empires</span></button><button disabled>Large<span class="seg-sub">8 empires</span></button></div>
  <div class="menu-pop" style="position:static;animation:none"><button>Save to file…</button><button disabled>Load from file…</button>
  </div>
  <div class="cp-great-works" style="width:240px"><button>The Signal Archive</button><button disabled>The Unbuilt Gate</button></div>
  <div class="prod-list" style="width:360px">
    <button class="prod"><span></span><span class="prod-name">Settler</span><span class="prod-cost">60</span><span></span><span class="prod-effect">Founds a new city</span><span class="prod-turns">6 turns</span></button>
    <button class="prod active"><span></span><span class="prod-name">Scout</span><span class="prod-cost">30</span><span></span><span class="prod-effect">Explores</span><span class="prod-turns">3 turns</span></button>
    <button class="prod" disabled><span></span><span class="prod-name">Trooper</span><span class="prod-cost">80</span><span></span><span class="prod-effect">Needs Feudalism</span><span class="prod-turns">8 turns</span></button>
  </div>
  <div style="width:300px"><button class="cta"><span class="cta-label">End turn</span><span class="cta-sub">Everything is ready</span></button>
    <button class="cta" disabled><span class="cta-label">End turn</span><span class="cta-sub">Waiting</span></button>
    <button class="cta waiting"><span class="cta-label">Waiting for Bo</span><span class="cta-sub">You are ready</span></button></div>
  <div style="position:relative;width:640px;height:140px">
    <article class="era-card available" style="position:absolute;left:0;top:0;width:200px"><header class="era-card-head"><span class="era-seal">I</span><h4 class="era-card-name">Bronze Working</h4></header><div class="era-cost-note">−30%: known by 3 empires</div></article>
    <article class="era-card known" style="position:absolute;left:210px;top:0;width:200px"><header class="era-card-head"><span class="era-seal">I</span><h4 class="era-card-name">Irrigation</h4></header></article>
    <article class="era-card queued" style="position:absolute;left:420px;top:0;width:200px"><header class="era-card-head"><span class="era-seal">I</span><h4 class="era-card-name">Masonry</h4><span class="era-queue-tag"><b>Queued #2</b></span></header></article>
  </div>`;

// The audit forces pseudo-states through the Chrome DevTools Protocol, which only Chromium has.
test.skip(({ browserName }) => browserName !== 'chromium', 'contrast audit needs CDP (Chromium only)');

test(`${look}: every button variant keeps AA label contrast in every state, disabled included`, async ({ page }, testInfo) => {
  test.setTimeout(120_000);
  await page.setViewportSize({ width: 1440, height: 1100 });
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  await openSettings(page);
  await page.keyboard.press('Escape');
  await page.evaluate((html) => {
    const gallery = document.createElement('div');
    gallery.id = 'cx-gallery';
    gallery.className = 'panel';
    gallery.style.cssText = 'position:fixed;inset:0;z-index:9999;display:flex;flex-wrap:wrap;align-content:flex-start;gap:12px;padding:12px;background:var(--panel);overflow:hidden';
    gallery.innerHTML = html;
    for (const g of gallery.querySelectorAll<HTMLElement>('.cx-group')) g.style.cssText = 'display:flex;flex-wrap:wrap;gap:8px;align-items:center;width:680px';
    document.body.appendChild(gallery);
  }, GALLERY);
  const readings = await auditContrast(page, 'gallery');
  await report(testInfo, `${look}-gallery`, readings);
  // Alliance/accepted-call badges use the same paired tokens in Atlas.
  expect(readings.filter((r) => r.where.includes('.relation-badge.alliance')).map((r) => r.state))
    .toEqual(expect.arrayContaining(['default', 'hover', 'focus-visible', 'active', 'disabled']));
  // Every variant was measured, disabled ones included.
  expect(readings.filter((r) => r.state === 'disabled').length).toBeGreaterThan(20);
  expect(failures(readings)).toEqual([]);
});

test(`${look}: title, Settings and lobby controls keep AA label contrast in every state`, async ({ page }, testInfo) => {
  test.setTimeout(240_000);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  await openSettings(page);
  const readings: ContrastReading[] = [];
  readings.push(...await auditContrast(page, 'settings'));
  await page.keyboard.press('Escape');
  readings.push(...await auditContrast(page, 'title'));
  await page.getByRole('button', { name: /^Multiplayer/ }).click();
  readings.push(...await auditContrast(page, 'multiplayer'));
  await page.getByRole('button', { name: /Back/ }).click();
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
  // Let the lobby's entrance settle: measured at once, Back read mid-fade (4.39 instead of 5.35).
  await page.waitForTimeout(800);
  readings.push(...await auditContrast(page, 'lobby'));
  await page.getByRole('button', { name: /^Seat 1 faction/ }).click();
  // Same as the lobby: measured at once, the picker reads mid-fade (1.2:1 washed-out labels) when its
  // entrance animation has not started yet, so auditContrast cannot finish it.
  await page.waitForTimeout(800);
  readings.push(...await auditContrast(page, 'faction picker'));
  await page.keyboard.press('Escape');
  await report(testInfo, `${look}-front`, readings);
  expect(readings.length).toBeGreaterThan(40);
  expect(failures(readings)).toEqual([]);
});

test(`${look}: game screen controls keep AA label contrast in every state`, async ({ page }, testInfo) => {
  test.setTimeout(240_000);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  await openSettings(page);
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  const readings: ContrastReading[] = [];
  // Turn 1: the founded capital's production choice is the primary action.
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  readings.push(...await auditContrast(page, 'game'));
  await chooseCapitalProduction(page);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  await page.getByRole('button', { name: 'Turn report (L)', exact: true }).click();
  await expect(page.getByRole('complementary', { name: 'Turn report' })).toBeVisible();
  readings.push(...await auditContrast(page, 'turn report'));
  await page.getByRole('button', { name: 'Close report', exact: true }).click();
  await page.getByRole('button', { name: /^Research/ }).click();
  readings.push(...await auditContrast(page, 'research'));
  await page.locator('.era-card[data-tech="Irrigation"]').getByTestId('research-now').click();
  await expect(page.locator('.research-queue li[data-tech="Irrigation"]')).toHaveCount(1);
  readings.push(...await auditContrast(page, 'research chosen'));
  await page.getByRole('button', { name: 'Close research', exact: true }).click();
  // The diplomacy inbox (no AI offers alliances this early; its actions still show).
  await revealSidebar(page, 'standings');
  await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click();
  const inbox = page.getByRole('complementary', { name: 'Diplomacy', exact: true });
  await expect(inbox).toBeVisible();
  // Audit only what the player sees once it settles: no story card over the inbox and no
  // layout still moving.
  await expect(page.locator('.moment-card')).toHaveCount(0);
  const firstAction = inbox.locator('.diplomacy-action button').first();
  await expect.poll(async () => {
    const before = JSON.stringify(await firstAction.boundingBox());
    await page.waitForTimeout(250);
    return before === JSON.stringify(await firstAction.boundingBox()) && !await page.locator('.moment-card').count();
  }).toBe(true);
  readings.push(...await auditContrast(page, 'diplomacy inbox'));
  await page.getByRole('button', { name: 'Close diplomacy', exact: true }).click();
  await revealSidebar(page, 'turn-more');
  const city = page.locator('.city-list-items button').first();
  if (!await city.isVisible()) await page.locator('.city-list > summary').click();
  await city.click();
  await expect(page.locator('.city-panel')).toBeVisible();
  readings.push(...await auditContrast(page, 'city'));
  await page.getByRole('button', { name: 'Menu', exact: true }).click();
  readings.push(...await auditContrast(page, 'game menu'));
  await report(testInfo, `${look}-game`, readings);
  expect(readings.length).toBeGreaterThan(40);
  expect(failures(readings)).toEqual([]);
});

test(`${look}: the tech unlock card and city build list keep AA label contrast in every state`, async ({ page }, testInfo) => {
  test.setTimeout(120_000);
  await page.setViewportSize({ width: 1000, height: 600 });
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
  await endPlanning(page);
  const card = page.locator('.moment-card.unlock');
  await expect(card).toBeVisible();
  await page.waitForTimeout(500);
  const readings: ContrastReading[] = [...await auditContrast(page, 'unlock card')];
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toBeVisible();
  const revenueOffice = page.locator('.city-panel [data-item="b:Market"] .prod');
  await revenueOffice.scrollIntoViewIfNeeded();
  await expect(revenueOffice).toBeInViewport();
  readings.push(...await auditContrast(page, 'city panel'));
  await report(testInfo, `${look}-unlock`, readings);
  // The card and the ordinary city build row were measured.
  for (const label of ['unlock card: button.btn.primary "SHOW REVENUE OFFICE IN', 'unlock card: button.btn.quiet "CHOOSE RESEARCH', 'city panel: button.prod ']) expect(readings.some((r) => r.where.startsWith(label)), label).toBe(true);
  expect(failures(readings)).toEqual([]);
});
