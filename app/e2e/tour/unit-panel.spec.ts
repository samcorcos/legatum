// The unit action panel's five common states, served from the real build and core view.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { spawnUnit, refreshVision } from '../../src/test/fixtureEngine';
import { ring, tileIndex } from '../../src/hex';
import { callToArmsCoreSave } from '../../src/test/allianceFixture';
import { SIZES, loadFixture, selectThing, shot, expectReasonsFit, type GameWindow } from './helpers';

const cases = ['fresh', 'upgrade-available', 'resource-blocked', 'siege-available', 'aircraft', 'aircraft-upgrade'] as const;
const baseline = process.env.UNIT_PANEL_BASELINE === '1';
for (const size of SIZES) test.describe(`unit panel ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  for (const scenario of cases) test(scenario, async ({ page }) => {
    const s = improvementCity({ techs: scenario === 'fresh' ? [] : scenario === 'aircraft-upgrade' ? ['Mathematics', 'IronWorking', 'StealthTechnology'] : ['Mathematics', 'IronWorking'] });
    const air = scenario.startsWith('aircraft');
    s.players[0].faction = 'Sparta';
    s.cities[0].name = 'Sparta'; s.cities[1].name = 'Bern';
    s.players[0].name = 'Sparta'; s.players[1].name = 'Switzerland';
    s.cities[1].hp = 200;
    s.units = s.units.filter((u) => u.owner !== 0);
    const unit = spawnUnit(s, 0, scenario === 'siege-available' ? 'Phalangite' : 'Spearman', ring(s.cities[0].pos, 1)[1], false);
    if (scenario === 'resource-blocked') {
      spawnUnit(s, 0, 'Phalangite', ring(s.cities[0].pos, 1)[2], false);
      spawnUnit(s, 0, 'Phalangite', ring(s.cities[0].pos, 1)[4], false);
    }
    refreshVision(s);
    let save = coreFixture(s);
    if (air) save = save.replace(/"next_unit_id":\d+/, '"next_unit_id":901')
      .replace(/^\{/, `{"aircraft":[{"id":900,"kind":"Fighter","city":${s.cities[0].id},"hp":100}],`);
    await loadFixture(page, save, s.turn);
    if (air) await page.evaluate(() => {
      const st = (window as unknown as GameWindow).legatum.getState();
      st.set({ techOpen: false }); st.select({ kind: 'air', id: 900 }, true);
    });
    else await selectThing(page, 'unit', unit.id);
    await expect(page.locator('.unit-panel')).toBeVisible();
    await expectReasonsFit(page);
    await page.mouse.move(20, 25);
    await shot(page, size, scenario);
    if (baseline) return;
    const panel = page.locator('.unit-panel');
    await expect(panel.locator('button.primary')).toHaveCount(1);
    const layout = await panel.evaluate((el) => {
      const p = el.getBoundingClientRect();
      return { top: p.top, bottom: p.bottom, width: el.scrollWidth - el.clientWidth,
        panelOverflow: el.scrollHeight - el.clientHeight,
        clippedButtons: [...el.querySelectorAll('button')].filter((b) => {
          const r = b.getBoundingClientRect();
          return r.width > 0 && (r.left < p.left || r.right > p.right || r.top < p.top || r.bottom > p.bottom || b.scrollWidth > b.clientWidth + 1);
        }).map((b) => b.textContent),
        dimensions: [...el.querySelectorAll('.up-head, .up-stats, .actions, .unit-panel-body, .up-hints')].map((e) => ({ name: e.className, height: e.clientHeight, scroll: e.scrollHeight })),
        clipped: [...el.querySelectorAll('.unit-panel-body')].some((e) => e.scrollHeight > e.clientHeight + 1 && getComputedStyle(e).display !== 'contents') };
    });
    expect(layout.top).toBeGreaterThanOrEqual(0);
    expect(layout.bottom).toBeLessThanOrEqual(size.height);
    expect(layout.width).toBeLessThanOrEqual(1);
    expect(layout.panelOverflow).toBeLessThanOrEqual(1);
    expect(layout.clippedButtons).toEqual([]);
    expect(layout.clipped, JSON.stringify(layout.dimensions)).toBe(false);
    // Both window sizes use the city dock, with no command strip across the map.
    const dock = await page.locator('.left-dock').boundingBox();
    const box = await panel.boundingBox();
    if (size.width === 1000) {
      const sidebar = await page.locator('.right-sidebar').boundingBox();
      expect(sidebar!.x - box!.x - box!.width, 'map retains at least 40% of the viewport').toBeGreaterThanOrEqual(400);
    }
    expect(box!.x).toBe(dock!.x);
    expect(box!.width).toBe(dock!.width);
    expect(box!.y).toBeCloseTo(dock!.y, 0);
    const clear = await page.evaluate((positions) => {
      const w = window as unknown as GameWindow;
      const obstacles = [...document.querySelectorAll('.unit-panel, .map-lenses, .vp, .seatbar, .minimap, .topbar')]
        .map((el) => ({ name: el.className, box: el.getBoundingClientRect() }));
      const p = document.querySelector('.unit-panel')!.getBoundingClientRect();
      return {
        overlaps: obstacles.filter((o) => !o.name.includes('unit-panel') && p.left < o.box.right && p.right > o.box.left && p.top < o.box.bottom && p.bottom > o.box.top).map((o) => o.name),
        covered: positions.flatMap((h) => {
          const point = w.legatumMap.toScreen(h);
          return obstacles.filter(({ box: b }) => point.x + 12 > b.left && point.x - 12 < b.right && point.y + 12 > b.top && point.y - 12 < b.bottom).map((o) => o.name);
        }),
      };
    }, air ? [s.cities[0].pos] : [unit.pos, ...ring(unit.pos, 1)]);
    expect(clear.overlaps).toEqual([]);
    expect(clear.covered, 'selected tile and adjacent destinations remain visible').toEqual([]);
    if (scenario === 'fresh') {
      await selectThing(page, 'city', s.cities[0].id);
      const cityBox = await page.locator('.city-panel').boundingBox();
      expect(cityBox!.x).toBe(box!.x);
      expect(cityBox!.width).toBe(box!.width);
      await expect.poll(async () => (await page.locator('.city-panel').boundingBox())!.y).toBeCloseTo(box!.y, 0);
      // A unit clicked near the left edge must be revealed when the panel opens.
      await page.evaluate((id) => {
        const w = window as unknown as GameWindow & { legatumMap: { camera: { x: number; zoom: number } } };
        const st = w.legatum.getState();
        st.select(null);
        const at = st.views![st.seat].units.find((u) => u.id === id)!.pos;
        const point = w.legatumMap.toScreen(at);
        w.legatumMap.camera.x += (point.x - 120) / w.legatumMap.camera.zoom;
      }, unit.id);
      await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'unit', id }), unit.id);
      await expect.poll(() => page.evaluate((id) => {
        const w = window as unknown as GameWindow, st = w.legatum.getState();
        const point = w.legatumMap.toScreen(st.views![st.seat].units.find((u) => u.id === id)!.pos);
        const panel = document.querySelector('.unit-panel')!.getBoundingClientRect();
        return point.x - 100 >= panel.right || point.x + 100 <= panel.left || point.y - 100 >= panel.bottom || point.y + 100 <= panel.top;
      }, unit.id)).toBe(true);
      const lens = page.getByRole('button', { name: /^Lens: / });
      await lens.click();
      await expect(page.getByRole('radiogroup', { name: 'Map lens', exact: true })).toBeVisible();
      await lens.click();
    }
    if (air) {
      const head = await panel.locator('.up-head').evaluate((e) => ({ scroll: e.scrollHeight, client: e.clientHeight }));
      expect(head.scroll, `aircraft head is whole (${JSON.stringify(head)})`).toBeLessThanOrEqual(head.client + 1);
      await expect(panel.locator('.up-hints .hint')).toBeInViewport({ ratio: 1 });
    }
    if (scenario === 'resource-blocked') {
      const ironReason = panel.locator('.improve-reason', { hasText: 'need 1 Iron. Your 2 Iron are all in use' });
      await expect(ironReason.first()).toBeVisible();
      await expect(ironReason.first().locator('.resource-reason-link')).toHaveAttribute('aria-label', 'Open Iron supply');
      for (const button of await panel.locator('.improve-row button.act').all()) await expect(button).toBeDisabled();
    }
    if (scenario === 'upgrade-available' || scenario === 'siege-available') {
      const row = panel.locator('.improve-row').filter({ has: page.locator('button.primary') });
      await row.getByRole('button', { name: 'Show explanation' }).click();
      await expect(page.locator('.tip-bubble.tip-pinned')).toContainText(scenario === 'upgrade-available' ? 'Upgrade cost' : 'Siege cost');
      await page.keyboard.press('Escape');
      if (scenario === 'upgrade-available') {
        await page.locator('[data-game-map]').focus();
        await page.keyboard.press('u');
      } else await row.locator('button.act').click();
      const result = await page.evaluate(() => {
        const st = (window as unknown as GameWindow).legatum.getState();
        return { orders: st.orders[st.seat], errors: st.errors[st.seat] };
      });
      expect(result.errors).toEqual([]);
      expect(result.orders).toContainEqual({ type: scenario === 'upgrade-available' ? 'UpgradeUnit' : 'UpgradeSiege', unit: unit.id });
    }
    if (scenario === 'fresh') {
      await panel.getByRole('button', { name: 'Move or attack' }).click();
      await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().mode)).toBe('move');
      await page.keyboard.press('Escape');
      await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().mode)).toBe('normal');
      await page.keyboard.press('f');
      await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().orders[0].some((o) => o.type === 'SleepUnit'))).toBe(true);
      await selectThing(page, 'unit', unit.id);
      await page.locator('[data-game-map]').focus();
      await page.keyboard.press('Space');
      await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatum.getState().orders[0].some((o) => o.type === 'SkipUnit'))).toBe(true);
    }
  });
  test('queued Siege and aircraft upgrades reduce the next land upgrade treasury', async ({ page }) => {
    const s = improvementCity({ techs: ['Mathematics', 'IronWorking', 'Feudalism', 'Electricity', 'AutonomousSystems', 'JetEngines', 'StealthTechnology'] });
    s.units = s.units.filter((u) => u.owner !== 0);
    const siege = spawnUnit(s, 0, 'Phalangite', ring(s.cities[0].pos, 1)[1], false);
    const next = spawnUnit(s, 0, 'Spearman', ring(s.cities[0].pos, 1)[2], false);
    const silicon = tileIndex(ring(s.cities[0].pos, 3)[5], s.width, s.height);
    s.terrain[silicon] = 'Hills'; s.resource![silicon] = 'Silicon';
    s.improved![tileIndex(ring(s.cities[0].pos, 3)[2], s.width, s.height)] = 1; // The Iron Mine leaves one slot for the next upgrade.
    s.players[0].gold = 600;
    refreshVision(s);
    const save = coreFixture(s).replace(/"next_unit_id":\d+/, '"next_unit_id":901')
      .replace(/^\{/, `{"aircraft":[{"id":900,"kind":"Fighter","city":${s.cities[0].id},"hp":100}],`);
    await loadFixture(page, save, s.turn);
    await selectThing(page, 'unit', siege.id);
    await page.locator('.unit-panel .improve-row').filter({ hasText: 'Siege' }).locator('button.act').click();
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select({ kind: 'air', id: 900 }, true));
    await page.locator('[data-testid="air-wing"] .improve-row button.act').click();
    await selectThing(page, 'unit', next.id);
    const amounts = await page.evaluate(() => {
      const st = (window as unknown as GameWindow).legatum.getState();
      const view = st.views![st.seat];
      const siegeOrder = st.orders[st.seat].find((o) => o.type === 'UpgradeSiege');
      const siege = view.units.find((u) => u.id === (siegeOrder && 'unit' in siegeOrder ? siegeOrder.unit : -1))!;
      const air = view.aircraft!.find((a) => a.id === 900)!;
      return { gold: view.economy.gold, reduced: view.economy.gold - siege.siege_offer!.price.total - air.upgrade_cost!, errors: st.errors[st.seat], orders: st.orders[st.seat] };
    });
    expect(amounts.errors).toEqual([]);
    expect(amounts.orders.map((o) => o.type)).toEqual(['UpgradeSiege', 'UpgradeAircraft']);
    await expect(page.locator('.gold-stat [data-celebration-counter="gold"]')).toContainText(String(amounts.reduced));
    const row = page.locator('.unit-panel .improve-row').filter({ hasText: 'Upgrade to' });
    await row.getByRole('button', { name: 'Show explanation' }).click();
    await expect(page.locator('.tip-bubble.tip-pinned')).toContainText(`Treasury: ${amounts.reduced} Gold`);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'queued-siege-air-next-upgrade');
  });
  test('role actions and a selected unit beside the call to arms', async ({ page }) => {
    if (baseline) return;
    for (const kind of ['Scout', 'Settler', 'Archer'] as const) {
      const state = improvementCity({ techs: [] });
      state.units = state.units.filter((u) => u.owner !== 0);
      const unit = spawnUnit(state, 0, kind, ring(state.cities[0].pos, 1)[1], false);
      refreshVision(state);
      await loadFixture(page, coreFixture(state), state.turn);
      await selectThing(page, 'unit', unit.id);
      const panel = page.locator('.unit-panel');
      await expect(panel.locator('button.primary')).toHaveCount(1);
      await expect(panel.locator('button.primary')).toContainText(kind === 'Scout' ? 'Auto-explore' : kind === 'Settler' ? 'Found City' : 'Move');
      await expectReasonsFit(page);
      await shot(page, size, `role-${kind.toLowerCase()}`);
      const fit = await panel.evaluate((el) => {
        const body = el.querySelector('.unit-panel-body')!;
        return { scroll: body.scrollHeight, client: body.clientHeight };
      });
      expect(fit.scroll, `${kind} actions fit at scroll zero (${JSON.stringify(fit)})`).toBeLessThanOrEqual(fit.client + 1);
    }
    await loadFixture(page, callToArmsCoreSave(), 16);
    await page.evaluate(() => {
      const st = (window as unknown as GameWindow).legatum.getState();
      const unit = st.views![st.seat].units.find((u) => u.owner === st.views![st.seat].me)!;
      st.select({ kind: 'unit', id: unit.id }, true);
    });
    await expect(page.locator('.unit-panel')).toBeVisible();
    const answer = page.getByRole('button', { name: /Answer call to arms/ }).first();
    await expect(answer).toBeVisible();
    await expect(page.locator('.unit-panel button.primary')).toHaveCount(0);
    await page.mouse.move(size.width / 2, 5);
    await shot(page, size, 'call-to-arms-unit');
    await answer.click();
    await expect(page.locator('.call-card')).toBeVisible();
    await page.locator('.call-card').getByRole('button', { name: 'Decline', exact: true }).click();
    await expect(page.locator('.call-card')).toHaveCount(0);
  });

});
