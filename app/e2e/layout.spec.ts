import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { auditLayout, type LayoutIssue } from './layoutAudit';
import { FACTIONS, FACTION_INFO } from '../src/data';
import { coreFixture } from '../src/test/coreFixture';
import { citiesFixture } from '../src/test/citiesFixture';

// Layout regressions from the first playtest (plans/playtest-feedback.md, A1–A3), at the desktop
// app's minimum window and two common sizes.

const SIZES = [
  { width: 1100, height: 700 },
  { width: 1280, height: 800 },
  { width: 1440, height: 900 },
];
const look = 'Atlas';

for (const size of [{ width: 1000, height: 600 }, { width: 1024, height: 600 }]) {
  test(`${size.width}x${size.height}: Cities remains a visible icon button`, async ({ page }) => {
    await page.setViewportSize(size);
    const game = coreFixture(citiesFixture());
    await page.addInitScript(({ game }) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, { game });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    const button = page.getByTestId('cities-button');
    await expect(button).toBeVisible();
    await expect(button.locator('.icon')).toBeVisible();
    await expect(button).toHaveAttribute('title', /Every city of yours/);
    const box = (await button.boundingBox())!;
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(size.width);
  });
}

/** Problems that mean something can't be seen or reached. */
const BLOCKING: LayoutIssue['kind'][] = ['offscreen', 'clipped', 'h-overflow', 'text-clipped'];
const blocking = (issues: LayoutIssue[]) => issues.filter((i) => BLOCKING.includes(i.kind));

async function openTitle(page: Page) {
  await page.goto('/?client=wasm');
  await expect(page.getByRole('button', { name: /^Single Player/ })).toBeVisible();
  // Let the title's entrance animation settle before measuring.
  await page.waitForTimeout(1200);
}

/** Open Settings for layout measurements. */
async function openSettings(page: Page) {
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await expect(dialog).toBeVisible();
  return dialog;
}

test.describe(`${look} look`, () => {
  for (const size of SIZES) {
    test(`title Settings fits the window at ${size.width}×${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      await openTitle(page);
      const dialog = await openSettings(page);
      await page.waitForTimeout(300);
      const box = (await dialog.boundingBox())!;
      expect(box.y).toBeGreaterThanOrEqual(0);
      expect(box.x).toBeGreaterThanOrEqual(0);
      expect(box.y + box.height).toBeLessThanOrEqual(size.height);
      expect(box.x + box.width).toBeLessThanOrEqual(size.width);
      expect(blocking(await auditLayout(page))).toEqual([]);
      // Closing it leaves the menu as it was: nothing grew past the frame.
      await page.keyboard.press('Escape');
      await expect(dialog).toBeHidden();
      expect(blocking(await auditLayout(page))).toEqual([]);
      // The whole menu shows without scrolling.
      const fits = () => page.locator('.title-menu').evaluate((menu) => {
        const settings = [...menu.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Settings')!.getBoundingClientRect();
        const box = menu.getBoundingClientRect();
        const inner = menu.closest('.title-inner')!;
        const foot = document.querySelector('.title-foot')!.getBoundingClientRect();
        return {
          scroll: menu.scrollHeight - menu.clientHeight + inner.scrollHeight - inner.clientHeight,
          settingsInside: settings.top >= box.top && settings.bottom <= box.bottom && box.bottom <= foot.top,
        };
      });
      expect(await fits()).toEqual({ scroll: 0, settingsInside: true });
      await page.locator('.title-actions').evaluate((actions) => {
        const quit = actions.querySelector('.btn.big.quiet')!.cloneNode(true) as HTMLElement;
        quit.textContent = 'Quit';
        actions.appendChild(quit);
      });
      expect(await fits()).toEqual({ scroll: 0, settingsInside: true });
    });

    test(`faction picker: a full-screen stage that fits the window at ${size.width}×${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      await openTitle(page);
      await openSettings(page);
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: /^Single Player/ }).click();
      await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
      // A new lobby has 6 seats; add the last two so the eighth seat's picker is checked, low on the page where panels used to run off.
      for (let i = 0; i < 2; i++) await page.getByRole('button', { name: /^\+ Add an AI empire/ }).click();
      // The first seat's picker, and the last one.
      for (const seat of [1, 8]) {
        const trigger = page.getByRole('button', { name: new RegExp(`^Seat ${seat} faction`) });
        await trigger.scrollIntoViewIfNeeded();
        await trigger.click();
        // Every seat has its own (closed) picker dialog in the DOM; data-testid matches by
        // attribute regardless of visibility, so scope to the one with the open attribute.
        const stage = page.locator('dialog[data-testid="faction-stage"][open]');
        await expect(stage).toBeVisible();
        await stage.evaluate((el) => Promise.all(el.getAnimations().map((a) => a.finished)));
        const options = page.getByRole('listbox', { name: `Seat ${seat} faction` }).getByRole('option');
        const count = await options.count();
        expect(count).toBeGreaterThan(8);
        // The whole stage sits inside the window: no scrolling panel, no clipped gallery or sheet.
        const box = (await stage.boundingBox())!;
        expect(box.x).toBeGreaterThanOrEqual(0);
        expect(box.y).toBeGreaterThanOrEqual(0);
        expect(box.x + box.width).toBeLessThanOrEqual(size.width);
        expect(box.y + box.height).toBeLessThanOrEqual(size.height);
        for (let i = 0; i < count; i++) {
          // Pick each tile the way a mouse does: scroll only the gallery, then click it. The click
          // chooses the faction and the dialog stays open.
          await options.nth(i).evaluate((el) => el.scrollIntoView({ block: 'nearest' }));
          const o = (await options.nth(i).boundingBox())!;
          expect(o.y, `tile ${i} clipped by the gallery`).toBeGreaterThanOrEqual(box.y);
          expect(o.y + o.height, `tile ${i} clipped by the gallery`).toBeLessThanOrEqual(box.y + box.height + 0.5);
          await page.mouse.click(o.x + o.width / 2, o.y + o.height / 2);
          // No stray tooltips, and the sheet names the chosen faction.
          expect(await page.locator('[role="tooltip"]:visible').count()).toBe(0);
          const name = await options.nth(i).locator('.fs-tile-name').innerText();
          const sheet = stage.getByTestId('faction-sheet');
          await expect(sheet.locator('.fs-sheet-name')).toContainText(name);
          // The sheet's own content never scrolls at this size: it was laid out to fit instead.
          const overflow = await sheet.evaluate((el) => el.scrollHeight - el.clientHeight);
          expect(overflow, `faction sheet for ${name} scrolls at ${size.width}×${size.height}`).toBeLessThanOrEqual(1);
        }
        expect(blocking(await auditLayout(page))).toEqual([]);
        await page.keyboard.press('Escape');
        await expect(stage).toBeHidden();
      }
    });

    test(`lobby: Start sits in its own dock and its summary fits at ${size.width}×${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      await openTitle(page);
      await openSettings(page);
      await page.keyboard.press('Escape');
      await page.getByRole('button', { name: /^Single Player/ }).click();
      await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
      await page.waitForTimeout(600);
      expect(blocking(await auditLayout(page))).toEqual([]);
      const start = page.getByRole('button', { name: /^Start game/ });
      const dock = page.locator('.lobby-dock');
      await expect(dock).toContainText('Start game');
      // The dock is outside the scrolling lobby: nothing in the list can pass underneath it.
      const scroller = (await page.locator('.lobby-inner').boundingBox())!;
      const bar = (await dock.boundingBox())!;
      expect(scroller.y + scroller.height).toBeLessThanOrEqual(bar.y + 0.5);
      expect(bar.y + bar.height).toBeLessThanOrEqual(size.height + 0.5);
      // Scrolled to the end, the map seed shows whole above the dock.
      await page.locator('.lobby-inner').evaluate((el) => el.scrollTo(0, el.scrollHeight));
      const seed = (await page.getByLabel('Map seed', { exact: true }).boundingBox())!;
      expect(seed.y + seed.height).toBeLessThanOrEqual(bar.y);
      // The summary line fits inside the button, at this size and a range of widths above it.
      for (const width of [size.width, size.width + 81, 1920]) {
        await page.setViewportSize({ width, height: size.height });
        const fit = await start.locator('.btn-sub').evaluate((el) => {
          const sub = el.getBoundingClientRect(), button = el.closest('button')!.getBoundingClientRect();
          return { overflow: el.scrollWidth - el.clientWidth, left: sub.left - button.left, right: button.right - sub.right };
        });
        expect(fit.overflow, `summary overflows at ${width}px`).toBeLessThanOrEqual(0);
        expect(fit.left, `summary cut on the left at ${width}px`).toBeGreaterThanOrEqual(0);
        expect(fit.right, `summary cut on the right at ${width}px`).toBeGreaterThanOrEqual(0);
      }
    });
  }
});

// The hard constraint (user's laptop): every faction sheet must fit whole, with no internal
// scrolling, at exactly 1000x600. The sizes above start at 1100x700, which has enough extra
// height to hide a sheet that only fits there and not at the real minimum (Sparta once
// overflowed by 73px here while passing every check at 1100x700 and up).
test(`${look} look: every faction sheet fits at 1000x600 with no internal scroll`, async ({ page }) => {
  await page.setViewportSize({ width: 1000, height: 600 });
  await openTitle(page);
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  const stage = page.locator('dialog[data-testid="faction-stage"][open]');
  await expect(stage).toBeVisible();
  // Measure the settled stage, not mid-way through its 0.2 s rise.
  await stage.evaluate((el) => Promise.all(el.getAnimations().map((a) => a.finished)));
  const box = (await stage.boundingBox())!;
  expect(box.y).toBeGreaterThanOrEqual(0);
  expect(box.y + box.height).toBeLessThanOrEqual(600);
  const sheet = stage.getByTestId('faction-sheet');
  for (const faction of FACTIONS) {
    const name = FACTION_INFO[faction].name;
    // A click chooses the faction and the sheet follows; the dialog stays open.
    await page.getByRole('option', { name }).click();
    await expect(sheet.locator('.fs-sheet-name')).toContainText(name);
    const overflow = await sheet.evaluate((el) => el.scrollHeight - el.clientHeight);
    expect(overflow, `${faction}'s sheet scrolls at 1000x600`).toBeLessThanOrEqual(1);
    // The one primary action (Close) is whole and inside the window, and the portrait has loaded.
    const close = (await stage.locator('.fs-close').boundingBox())!;
    expect(close.y + close.height, `Close is below the window at 1000x600`).toBeLessThanOrEqual(600);
    expect(await sheet.locator('.fs-portrait > img').evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0), `${faction}'s portrait did not load`).toBe(true);
  }
});

// Era overhaul: the top bar's era chip and the research screen fit the window in a two-human game too
// (the second seat adds a row to the dock and a second research state).
for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`${size.width}×${size.height} two humans: the era chip and the research screen fit`, async ({ page }) => {
    const { researchFixture } = await import('../src/test/researchFixture');
    const { coreFixture } = await import('../src/test/coreFixture');
    await page.setViewportSize(size);
    const fixture = researchFixture('India');
    fixture.players.forEach((p, i) => { p.isHuman = i < 2; fixture.setup.players[i].is_human = i < 2; });
    fixture.players[0].research = 'Masonry'; fixture.players[0].research_queue = ['Masonry', 'Writing'];
    await page.addInitScript((game) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada', 'Bo'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, coreFixture(fixture));
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    await expect(page.locator('.seatbar .local-seat')).toHaveCount(2);
    const chip = page.getByTestId('era-chip');
    await expect(chip).toContainText('Ancient');
    const bar = (await page.locator('.topbar').boundingBox())!, box = (await chip.boundingBox())!;
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(bar.x + bar.width);
    expect(blocking(await auditLayout(page, { minTarget: 24 })).filter((i) => /topbar|era-chip|research-chip/.test(i.where))).toEqual([]);
    await chip.click();
    const dialog = page.locator('dialog.tech-modal');
    await expect(dialog).toBeVisible();
    await page.waitForTimeout(300);
    const d = (await dialog.boundingBox())!;
    expect(d.y).toBeGreaterThanOrEqual(0);
    expect(d.y + d.height).toBeLessThanOrEqual(size.height);
    await expect(dialog.locator('.era-card')).toHaveCount(5);
    expect(blocking(await auditLayout(page, { minTarget: 24 })).filter((i) => /tech-modal|era-|research-/.test(i.where))).toEqual([]);
  });
}
