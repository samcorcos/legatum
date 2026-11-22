import { expectSidebarReachable } from './sidebarAssertions';
import { revealSidebar } from './sidebarAssertions';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';

// Playtest 2 pre-gate: at 1440×900, opening a long Cities seen list grew This Turn up over
// the Victory Race links and clipped the first city row. The dock must stay in its slot.
for (const look of ['atlas'] as const) {
  test(`${look} 1440×900: a long Cities seen list scrolls inside This Turn, below the Victory Race`, async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1');
    await page.keyboard.press('b');
    await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn)/ }).click();
    const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
    if (await confirm.isVisible().catch(() => false)) await confirm.click();
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    // Many known cities (as in a long game): add foreign cities to the local view only.
    await page.evaluate(() => {
      const store = (window as unknown as { legatum: { getState(): { views: unknown[]; set(p: object): void } } }).legatum;
      const s = store.getState(), v = structuredClone(s.views[0]) as { cities: { id: number; owner: number; name: string; detail: unknown }[] };
      const base = v.cities[0];
      for (let i = 0; i < 12; i++) v.cities.push({ ...base, id: 900 + i, owner: (i % 7) + 1, name: `Far City ${i}`, detail: null });
      s.set({ views: [v] });
    });
    await revealSidebar(page, 'turn-more');
    await page.locator('details.city-list summary').click();
    await expectSidebarReachable(page);
    await page.locator('.sidebar-scroll').evaluate(el => { el.scrollTop = 0; });
    // The Victory Race links stay clickable.
    await page.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
  });
}

// Playtest-6 B1/H5: at the 1000x600 laptop window, each human seat added a row to This Turn and
// pushed End Turn and Ready anyway off the bottom with 2 or 3 humans. The primary action stays
// whole in the window in every seat setup, and the objective sheet closes back onto it.
for (const humans of [1, 2, 3]) {
  test(`1000×600 with ${humans} human seat${humans > 1 ? 's' : ''}: the turn's primary action stays in the window`, async ({ page }) => {
    await page.setViewportSize({ width: 1000, height: 600 });
    const fixture = wallsFixture().fixture;
    fixture.players.forEach((p, i) => { p.isHuman = i < humans; fixture.setup.players[i].is_human = i < humans; });
    await page.addInitScript((game) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, coreFixture(fixture));
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    await expect(page.locator('.seatbar .local-seat')).toHaveCount(humans > 1 ? humans : 0);
    const whole = async (selector: string) => {
      const el = page.locator(selector).first();
      await expect(el).toBeVisible();
      const b = (await el.boundingBox())!;
      expect(b.y, `${selector} top`).toBeGreaterThanOrEqual(0);
      expect(b.y + b.height, `${selector} bottom`).toBeLessThanOrEqual(600);
      // Nothing clips or covers it: its centre hits the control itself.
      expect(await el.evaluate((node) => {
        const r = node.getBoundingClientRect();
        return node.contains(document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2));
      }), `${selector} is not covered or clipped`).toBe(true);
      return b;
    };
    const dock = (await page.locator('.seatbar').boundingBox())!;
    const cta = await whole('.seatbar .cta');
    await whole('.seatbar .ready-anyway');
    expect(cta.y + cta.height).toBeLessThanOrEqual(dock.y + dock.height);
    const scroll = (await page.locator('.sidebar-scroll').boundingBox())!;
    expect(scroll.y + scroll.height).toBeLessThanOrEqual(cta.y);
    // The objective sheet is a modal: it covers the dock while open and gives it back whole on Escape.
    await page.locator('button.vh-chip').click();
    await expect(page.getByTestId('objective-sheet')).toBeVisible();
    const sheet = (await page.getByTestId('objective-sheet').boundingBox())!;
    expect(sheet.y).toBeGreaterThanOrEqual(0);
    expect(sheet.y + sheet.height).toBeLessThanOrEqual(600);
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('objective-sheet')).toHaveCount(0);
    await whole('.seatbar .cta');
  });
}
