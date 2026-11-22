import type { Locator, Page } from '@playwright/test';
import { expect } from './fixtures';

/** A visible control is whole, in the column, and receives clicks at both edges. */
async function whole(control: Locator) {
  await expect(control).toBeVisible();
  expect(await control.evaluate(el => {
    const b = el.getBoundingClientRect();
    // A disabled control lets the pointer through to its Tip wrapper (styles.css, `.tip-trigger button:disabled`), which is the hover and focus target.
    const on = (y: number) => { const hit = document.elementFromPoint(b.x + b.width / 2, y); return hit === el || (hit && (el.contains(hit) || (hit.classList.contains('tip-trigger') && hit.contains(el)))); };
    return b.left >= 0 && b.right <= innerWidth + 1 && b.top >= 0 && b.bottom <= innerHeight + 1 && on(b.top + 2) && on(b.bottom - 2);
  }), `${await control.textContent()} is whole and unobscured`).toBe(true);
}

export async function expectSidebarGlance(page: Page) {
  const sidebar = page.getByRole('complementary', { name: 'Empire sidebar' });
  await expect(sidebar).toBeVisible();
  for (const selector of ['.race-headline', '.sidebar-objective', '.sidebar-goal > summary, .sidebar-goal-action .goal-action-btn', '.cta', '.minimap']) {
    const el = sidebar.locator(selector);
    if (await el.count() && await el.isVisible()) await whole(el);
  }
  // No persistent panel on the bottom centre. The selection task owns the left side.
  const mini = await sidebar.locator('.minimap').boundingBox();
  if (mini) {
    const col = (await sidebar.boundingBox())!;
    expect(mini.x).toBeGreaterThanOrEqual(col.x);
    expect(mini.y + mini.height).toBeLessThanOrEqual(col.y + col.height + 1);
  } else {
    expect(await page.evaluate(() => innerWidth <= 1100 && innerHeight <= 700 && !!document.querySelector('.unit-panel')),
      'the minimap is hidden only beside an open unit panel on a short screen').toBe(true);
  }
  const overflows = await sidebar.evaluate(el => [...el.querySelectorAll<HTMLElement>('p, summary, .eo-text, .cta-label, .cta-sub, .sidebar-section-link, .vp-team-name, .goal-action-note, .goal-action-why, .siege-title, .siege-who, .siege-forecast')].filter(x => x.checkVisibility() && x.clientWidth > 0 && x.scrollWidth > x.clientWidth + 1).map(x => `${x.className}: ${x.scrollWidth}/${x.clientWidth}`));
  expect(overflows, 'no horizontally clipped sidebar text').toEqual([]);
  await expect(page.locator('.vh-chip')).not.toContainText(/\d|wins? in/);
}

export async function expectSidebarReachable(page: Page) {
  const sidebar = page.getByRole('complementary', { name: 'Empire sidebar' });
  const scroll = sidebar.locator('.sidebar-scroll');
  const groups = ['.sidebar-alerts', '.sidebar-standings', '.sidebar-goal', '.sidebar-turn-more'];
  for (const selector of groups) {
    const detail = sidebar.locator(selector);
    if (!await detail.count()) continue;
    const summary = detail.locator(':scope > summary');
    if (!await summary.count()) {
      await detail.locator('button.goal-action-btn').evaluate(el => el.scrollIntoView({ block: 'center' }));
      continue;
    }
    await summary.evaluate(el => el.scrollIntoView({ block: 'center' }));
    if (!await detail.evaluate(el => (el as HTMLDetailsElement).open)) await summary.click();
    if (selector === '.sidebar-turn-more') {
      const orders = detail.locator('.orders-toggle');
      await orders.evaluate(el => el.scrollIntoView({ block: 'center' }));
      if (await orders.getAttribute('aria-expanded') === 'false') await orders.click();
    }
    const targets = detail.locator('button:visible, .vp-row:visible, p:visible');
    if (await targets.count()) {
      const last = targets.last();
      await last.evaluate(el => el.scrollIntoView({ block: 'center' }));
      await whole(last);
    }
    await whole(sidebar.locator('.cta'));
  }
  expect(await scroll.evaluate(el => getComputedStyle(el).overflowY)).toBe('auto');
  expect(await scroll.evaluate(el => getComputedStyle(el).scrollbarWidth)).not.toBe('none');
  await scroll.evaluate(el => { el.scrollTop = 0; });
  await scroll.hover({ position: { x: 25, y: 120 } });
  await page.mouse.wheel(0, 2000);
  await expect.poll(() => scroll.evaluate(el => el.scrollTop)).toBeGreaterThan(0);
  const last = sidebar.locator('.sidebar-turn-more');
  await last.evaluate(el => el.scrollIntoView({ block: 'end' }));
  await whole(sidebar.locator('.cta'));
  await whole(sidebar.locator('.race-headline'));
  // Keyboard also reaches the expanded content without being captured by the map.
  await sidebar.locator('.sidebar-turn-more > summary').focus();
  await page.keyboard.press('Tab');
  expect(await sidebar.evaluate(el => el.contains(document.activeElement))).toBe(true);
}

/** Navigate through the real disclosure before interacting with its controls. */
export async function revealSidebar(page: Page, group: 'standings' | 'goal' | 'alerts' | 'turn-more') {
  const detail = page.locator(`.right-sidebar .sidebar-${group}`);
  if (group === 'goal' && await detail.locator(':scope > .goal-action').count()) return;
  if (!await detail.count() || await detail.evaluate(el => (el as HTMLDetailsElement).open)) return;
  const summary = detail.locator(':scope > summary');
  await summary.evaluate(el => el.scrollIntoView({ block: 'center' }));
  await summary.click();
}

/**
 * The slim 196 px rail (1000x600 with the unit panel open, the commonest state there) is its own layout: every row
 * stays inside the rail, a summary's count never spills out of its row onto the next (8.9.14 review: "0/3 · +2"
 * wrapped across the divider), and a race list cut by the column says "+N more".
 */
export async function expectSlimRail(page: Page) {
  const sidebar = page.getByRole('complementary', { name: 'Empire sidebar' });
  await expect(page.locator('.game:has(.unit-panel)')).toBeVisible();
  await revealSidebar(page, 'standings');
  await revealSidebar(page, 'goal');
  const problems = await sidebar.evaluate((root) => {
    const rail = root.getBoundingClientRect();
    const out: string[] = [];
    const name = (el: Element) => `${el.tagName.toLowerCase()}.${String(el.className).split(' ')[0]}`;
    const rows = root.querySelectorAll<HTMLElement>('.race-headline, .sidebar-alerts > summary, .sidebar-objective, .sidebar-goal > summary, .sidebar-standings > summary, .sidebar-turn-more > summary, .vp-row, .turn-details > .ph, .cta, .cta-sub, .cta .kbd, .sidebar-more');
    for (const el of rows) {
      if (!el.checkVisibility()) continue;
      const b = el.getBoundingClientRect();
      if (b.right > rail.right + 1 || b.left < rail.left - 1) out.push(`${name(el)} sticks out of the rail (${Math.round(b.left)}..${Math.round(b.right)} of ${Math.round(rail.left)}..${Math.round(rail.right)})`);
      if (el.scrollWidth > el.clientWidth + 1) out.push(`${name(el)} is clipped (${el.scrollWidth}/${el.clientWidth})`);
    }
    // A float or wrapped count must stay inside its own summary, never across the divider below it.
    for (const summary of root.querySelectorAll<HTMLElement>('summary')) {
      if (!summary.checkVisibility()) continue;
      const box = summary.getBoundingClientRect();
      for (const child of summary.children) if (child.getBoundingClientRect().bottom > box.bottom + 1) out.push(`${name(child)} spills out of its summary`);
    }
    return out;
  });
  expect(problems, 'the slim rail keeps every row whole').toEqual([]);
  const more = sidebar.getByTestId('sidebar-more');
  if (await more.count()) {
    await expect(more).toHaveText(/^\+\d+ more empires?$/);
    await whole(more);
  }
}
