// Screen tour, combat animations (map/combatFx.ts): a resolving assault played back on the map, frame-stepped on
// Playwright's clock so the shots land on fixed moments; then the Settings toggle that turns the playback off. Run it
// alone with `npx playwright test e2e/tour/combat-fx`. Shots land in test-results/screen-tour/<size>/combat-fx/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { combinedAssaultFixture } from '../../src/test/combinedAssaultFixture';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

type FxWindow = GameWindow & { legatumMap: { perf: { fxFrames: number; reset(): void }; camera: { x: number; y: number; zoom: number }; redraw(): void } };

/** Put the city in the middle of the map at a close zoom (the camera glide runs on the paused clock, so set it directly). */
async function frameCity(page: Page, id: number) {
  await page.evaluate((id) => {
    const w = window as unknown as FxWindow, s = w.legatum.getState(), v = s.views![s.seat];
    const u = v.cities.find((x) => x.id === id)!;
    const m = w.legatumMap, c = m.camera, p = m.toScreen(u.pos);
    const box = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    Object.assign(c, { x: c.x + (p.x - box.width / 2) / c.zoom, y: c.y + (p.y - box.height / 2) / c.zoom });
    c.zoom = 1.6;
    m.redraw();
  }, id);
  await page.clock.runFor(300);
}

/** Both Spearmen storm the broken city; end the turn on a paused clock so the playback starts at a known instant. */
async function storm(page: Page, setup: ReturnType<typeof combinedAssaultFixture>) {
  await page.evaluate(({ units, city }) => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    const to = v.cities.find((c) => c.id === city)!.pos;
    s.editOrders((o) => [...o, ...units.map((unit) => ({ type: 'MoveUnit' as const, unit, to }))]);
    (window as unknown as GameWindow).legatum.getState().select(null);
  }, { units: [setup.nearOpen, setup.second], city: setup.open });
  await frameCity(page, setup.open);
  await page.mouse.move(5, 300);
  await page.clock.pauseAt(await page.evaluate(() => Date.now()) + 2000);
  await page.evaluate(() => (window as unknown as FxWindow).legatumMap.perf.reset());
  await endPlanning(page);
  for (let i = 0; (await page.getByTestId('turn-number').textContent()) !== '3'; i++) {
    if (i > 600) throw new Error('the turn did not resolve');
    await page.waitForTimeout(30);
    await page.clock.runFor(5);
  }
  // The clock crept on while the turn resolved: start the playback from its first frame, so the shots land on fixed moments.
  await page.evaluate(() => {
    const g = (window as unknown as GameWindow).legatum, a = g.getState().anim;
    if (a) g.setState({ anim: { ...a, start: performance.now() } });
  });
}

const fxFrames = (page: Page) => page.evaluate(() => (window as unknown as FxWindow).legatumMap.perf.fxFrames);
const plan = (page: Page) => page.evaluate(() => {
  const p = (window as unknown as GameWindow).legatum.getState().anim?.plan;
  return p ? { clashes: p.clashes.length, shots: p.shots.length, total: p.total } : null;
});

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('combat animations: the assault plays out on the map, then the board settles', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = combinedAssaultFixture();
    await page.clock.install();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await storm(page, setup);
    const p = await plan(page);
    expect(p?.clashes).toBeGreaterThan(0);
    const zeroHpAlarm = page.locator('.siege-card.attack').filter({ hasText: '0/200 HP' });
    if (await zeroHpAlarm.count()) {
      await expect(zeroHpAlarm).not.toContainText('Drops to 0 HP');
      await expect(zeroHpAlarm).toContainText('At 0 HP');
    }
    // The capture's moment card sits over the city at this size; these shots are about the map.
    await page.addStyleTag({ content: '.moment-lane{visibility:hidden!important}' });
    // The clash lands mid-step of the first round: shoot the lunge, the impact and the dust, then the settled board.
    for (const [ms, name] of [[120, '01-fx-lunge'], [90, '02-fx-impact'], [260, '03-fx-dust']] as const) {
      await page.clock.runFor(ms);
      await shot(page, size, name);
    }
    expect(await fxFrames(page)).toBeGreaterThan(0);
    await page.clock.runFor(p!.total + 500);
    const settled = await fxFrames(page);
    await page.clock.runFor(500);
    // Nothing is drawn for the playback once it is over.
    expect(await fxFrames(page)).toBe(settled);
    await shot(page, size, '04-fx-settled');
  });

  test('combat animations: Map animations Off plays no animation', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = combinedAssaultFixture();
    await page.clock.install();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.getByRole('button', { name: 'Menu', exact: true }).click();
    const choices = page.getByRole('group', { name: 'Map animations' });
    await expect(page.locator('.combat-fx-settings')).toContainText('Off: instant moves; damage numbers still show');
    await expect(choices.getByRole('button', { name: 'On' })).toHaveAttribute('aria-pressed', 'true');
    await shot(page, size, '05-fx-setting-on');
    await choices.getByRole('button', { name: 'Off' }).click();
    await expect(choices.getByRole('button', { name: 'Off' })).toHaveAttribute('aria-pressed', 'true');
    await shot(page, size, '06-fx-setting-off');
    // Close the menu as a click elsewhere does (the menu listens for a pointerdown outside it).
    await page.evaluate(() => document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true })));
    await page.clock.runFor(100);
    await expect(page.locator('#game-menu')).toHaveCount(0);
    await storm(page, setup);
    await page.clock.runFor(1500);
    expect(await fxFrames(page)).toBe(0);
    expect(await page.evaluate(() => localStorage.getItem('legatum.combat-fx'))).toContain('"enabled":false');
  });
});
