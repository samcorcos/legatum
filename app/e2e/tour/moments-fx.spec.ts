// Screen tour, payoff moments on the map (map/momentFx.ts): the border spreads ring by ring while a new city and its
// Walls stand at once (the construction animation is off until the designer approves the clips). Frame-stepped on
// Playwright's clock so the shots land on fixed moments; then the Map animations toggle that turns them off. Run it alone
// with `npx playwright test e2e/tour/moments-fx`. Shots land in test-results/screen-tour/<size>/moments-fx/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { refreshVision, spawnUnit } from '../../src/test/fixtureEngine';
import { fromOffset, hexToPixel } from '../../src/hex';
import { HEX_SIZE } from '../../src/map/grid';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

type MomentWindow = GameWindow & { legatumMap: { perf: { momentFrames: number; reset(): void }; camera: { x: number; y: number; zoom: number }; redraw(): void } };

/** Your capital a turn from finishing its Walls, and a Settler on open ground three hexes away. */
function foundingFixture() {
  const setup = wallsFixture();
  const f = setup.fixture;
  const own = f.cities.find((c) => c.id === setup.own)!;
  own.buildings = [];
  for (const city of f.cities) if (city.id !== own.id) {
    city.producing = null;
    city.productionQueue = [];
    city.prod = 0;
  }
  f.players[0].gold = 900;
  for (const player of f.players.slice(1)) player.gold = 0;
  // Walls need Masonry and can't be bought: they are a turn from done, and complete when it resolves.
  if (!f.players[0].techs.includes('Masonry')) f.players[0].techs.push('Masonry');
  Object.assign(own, { producing: { kind: 'Building', building: 'Walls' }, prod: 59, productionQueue: [] });
  // One human seat, so the turn resolves instead of passing to the next player.
  for (const p of [1, 2]) { f.players[p].isHuman = false; f.setup.players[p].is_human = false; }
  const settler = spawnUnit(f, 0, 'Settler', fromOffset(6, 7), false);
  refreshVision(f);
  return { ...setup, settler: settler.id };
}

/** Found the city (the Walls finish on their own), then end the turn on a paused clock so the moments start at a known instant. */
async function foundAndBuild(page: Page, setup: ReturnType<typeof foundingFixture>) {
  await page.evaluate(({ settler }) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    s.editOrders((o) => [...o, { type: 'FoundCity' as const, unit: settler }]);
    s.select(null);
  }, { settler: setup.settler });
  // Frame both: the new city and the capital, close in.
  const pa = hexToPixel(fromOffset(6, 7), HEX_SIZE), pb = hexToPixel(fromOffset(3, 5), HEX_SIZE);
  await page.evaluate((cam) => {
    const m = (window as unknown as MomentWindow).legatumMap;
    Object.assign(m.camera, cam);
    m.redraw();
  }, { x: (pa.x + pb.x) / 2, y: (pa.y + pb.y) / 2 - 10, zoom: 1.9 });
  await page.mouse.move(5, 300);
  await page.clock.pauseAt(await page.evaluate(() => Date.now()) + 2000);
  await page.evaluate(() => (window as unknown as MomentWindow).legatumMap.perf.reset());
  await endPlanning(page);
  for (let i = 0; (await page.getByTestId('turn-number').textContent()) !== '3'; i++) {
    if (i > 600) throw new Error('the turn did not resolve');
    await page.waitForTimeout(30);
    await page.clock.runFor(5);
  }
}

/** A reward card is separate; let the map shots show the city and border themselves. */
async function dismissMomentCards(page: Page) {
  const reportClose = page.getByRole('button', { name: 'Close report' });
  if (await reportClose.count()) await reportClose.click();
  for (let i = 0; i < 8 && await page.locator('.moment-card:not(.unlock)').count(); i++) {
    await page.locator('.moment-card:not(.unlock) .moment-dismiss').click();
  }
  await expect(page.locator('.moment-card:not(.unlock)')).toHaveCount(0);
}

const momentFrames = (page: Page) => page.evaluate(() => (window as unknown as MomentWindow).legatumMap.perf.momentFrames);
const plan = (page: Page) => page.evaluate(() => {
  const p = (window as unknown as GameWindow).legatum.getState().anim?.moments;
  return p ? { founds: p.founds.length, builds: p.builds.map((b) => b.walls), buildItems: p.builds.map((b) => `${b.city}:${b.item.kind === 'Building' ? b.item.building : b.item.kind}`), tiles: p.tiles.length, total: p.total, found: p.founds[0]?.t ?? 0, build: p.builds[0]?.t ?? 0 } : null;
});

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('payoff moments: the border spreads while the new city and its Walls stand at once (construction animation is off)', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = foundingFixture();
    await page.clock.install();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await foundAndBuild(page, setup);
    await dismissMomentCards(page);
    const p = await plan(page);
    // The founding and the Walls stay in the turn's moment plan (CONSTRUCTION_ANIMATION is off until the clips are approved); only the border spreads.
    expect(p!.buildItems).toEqual([`${setup.own}:Walls`]);
    expect(p).toMatchObject({ founds: 1, builds: [true] });
    expect(p!.tiles).toBeGreaterThan(1);
    let at = 0;
    for (const [ms, name] of [[p!.found + 300, '01-border-spreading'], [p!.found + 900, '02-border-ring'], [p!.found + 1400, '03-border-done']] as const) {
      await page.clock.runFor(Math.max(0, ms - at));
      at = ms;
      await shot(page, size, name);
    }
    expect(await momentFrames(page)).toBeGreaterThan(0);
    await page.clock.runFor(p!.total - at + 500);
    const settled = await momentFrames(page);
    await page.clock.runFor(500);
    // Nothing is drawn for the moments once they are over.
    expect(await momentFrames(page)).toBe(settled);
    await shot(page, size, '04-settled');
  });

  test('payoff moments: with map animations off the new city is simply there', async ({ page }) => {
    test.setTimeout(120_000);
    const setup = foundingFixture();
    await page.clock.install();
    await page.addInitScript(() => localStorage.setItem('legatum.combat-fx', JSON.stringify({ enabled: false, speed: 1 })));
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await foundAndBuild(page, setup);
    await dismissMomentCards(page);
    await page.clock.runFor(800);
    expect(await momentFrames(page)).toBe(0);
    await expect(page.locator('.moment-card')).toHaveCount(0);
    await expect(page.locator('.moment-rays:visible, .alarm-rays:visible')).toHaveCount(0);
    await shot(page, size, '05-off-settled');
  });
});
