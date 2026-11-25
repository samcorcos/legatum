// Screen tour, Walls levels outside the map: the city build list's Upgrade Walls item (no Walls, available,
// at the era limit, top level, the Swiss Alpine Redoubt), the attack forecast's Walls line (built-in Siege,
// partial Siege), and the Machinery and Metallurgy tech cards. Run alone with
// `npx playwright test e2e/tour/wall-levels-ui`. Shots land in test-results/screen-tour/<size>/wall-levels-ui/.
import type { Page } from '@playwright/test';
import type { BuildingKind, TechId } from '../../src/types';
import { test, expect } from '../fixtures';
import { wallsFixture } from '../../src/test/wallsFixture';
import { coreFixture } from '../../src/test/coreFixture';
import { spawnUnit, refreshVision } from '../../src/test/fixtureEngine';
import { eraTechs } from '../../src/data';
import { fromOffset } from '../../src/hex';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

// wallsFixture seats three humans; Continue would then open it as an online host. The tour plays seat 0 alone.
function soloSeat<F extends ReturnType<typeof wallsFixture>['fixture']>(f: F): F {
  for (const p of f.players.slice(1)) { p.isHuman = false; f.setup.players[p.id].is_human = false; }
  return f;
}

const techsThrough = (era: number): TechId[] => Array.from({ length: era + 1 }, (_, e) => eraTechs(e)).flat();

async function openCityList(page: Page, id: number) {
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    s.set({ techOpen: false });
    s.select({ kind: 'city', id }, true);
    s.jumpTo(v.cities.find((c) => c.id === id)!.pos);
  }, id);
  // With no Walls, Upgrade Walls is not its own card: it nests under Walls as "Leads to: Upgrade Walls".
  const own = page.locator('.city-panel .production-choice[data-item="b:UpgradeWalls"]');
  // At level IX neither is listed: the shot shows where the Walls line would be (the war buildings).
  await page.locator('.city-panel .production-choice').first().waitFor();
  const item = await own.count() ? own : page.locator('.city-panel .production-choice[data-item="b:Walls"]');
  if (await item.count()) await item.scrollIntoViewIfNeeded();
  else await page.locator('.city-panel .production-choice[data-item="b:Roadworks"]').scrollIntoViewIfNeeded();
  await page.mouse.move(2, 2);
  return item;
}

/** Screen point of a city once the camera has settled. */
async function cityPoint(page: Page, id: number) {
  const at = () => page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), view = s.views![s.seat];
    const p = (window as unknown as GameWindow).legatumMap.toScreen(view.cities.find((c) => c.id === id)!.pos);
    const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
  }, id);
  let last = await at();
  await expect.poll(async () => { const now = await at(); const still = now.x === last.x && now.y === last.y; last = now; return still; }).toBe(true);
  return last;
}

for (const size of SIZES) test.describe(`wall levels UI ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  // techsThrough(2) puts the empire in the Renaissance era (level IV is its limit).
  const cases: { name: string; era: number; level: number; swiss?: boolean; hover?: boolean; producing?: boolean }[] = [
    { name: 'build-no-walls', era: 2, level: 0 },
    { name: 'build-walls-i-medieval', era: 2, level: 1, hover: true },
    { name: 'build-to-iv', era: 2, level: 3, hover: true },
    { name: 'build-at-era-limit', era: 2, level: 4 },
    { name: 'build-producing-upgrade', era: 2, level: 2, producing: true },
    { name: 'build-top-level', era: 8, level: 9 },
    { name: 'build-swiss-redoubt', era: 2, level: 2, swiss: true },
  ];
  for (const c of cases) test(`city build list: ${c.name}`, async ({ page }) => {
    const setup = wallsFixture(), f = soloSeat(setup.fixture);
    const own = f.cities.find((x) => x.id === setup.own)!;
    own.buildings = (c.level > 0 ? ['Walls'] : []) as BuildingKind[];
    own.wallsLevel = c.level;
    own.producing = c.producing ? { kind: 'Building', building: 'UpgradeWalls' } as typeof own.producing : null; own.productionQueue = [];
    if (c.producing) own.prod = 120;
    f.players[0].techs = techsThrough(c.era);
    if (c.swiss) { f.players[0].faction = 'Switzerland'; f.setup.players[0].faction = 'Switzerland'; }
    await loadFixture(page, coreFixture(f), 2);
    const item = await openCityList(page, setup.own);
    await shot(page, size, `${c.name}`);
    if (c.producing) {
      await page.locator('.city-panel .cp-scroll, .city-panel').first().evaluate((el) => { el.scrollTop = 0; });
      await shot(page, size, `${c.name}-top`);
    }
    if (c.hover) {
      await item.locator('button.prod').hover();
      await page.waitForTimeout(700);
      await shot(page, size, `${c.name}-hover`);
    }
  });

  for (const kind of ['built-in', 'partial'] as const) test(`attack forecast Walls line: ${kind}`, async ({ page }) => {
    const setup = wallsFixture(), f = soloSeat(setup.fixture);
    const berea = f.cities.find((x) => x.id === setup.walled)!;
    berea.wallsLevel = kind === 'built-in' ? 4 : 3;
    f.units = f.units.filter((u) => u.id !== setup.nearWalled);
    f.players[0].techs = techsThrough(kind === 'built-in' ? 4 : 1);
    const attacker = spawnUnit(f, 0, kind === 'built-in' ? 'Rifleman' : 'Spearman', fromOffset(8, 5), false);
    if (kind === 'partial') attacker.siege = true;
    refreshVision(f);
    await loadFixture(page, coreFixture(f), 2);
    await page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      s.set({ techOpen: false });
      s.select({ kind: 'unit', id }, true);
      s.jumpTo(v.units.find((u) => u.id === id)!.pos);
    }, attacker.id);
    const line = page.getByTestId('forecast-walls').first();
    await expect.poll(async () => {
      const at = await cityPoint(page, setup.walled);
      await page.mouse.move(at.x + 3, at.y + 3);
      await page.mouse.move(at.x, at.y, { steps: 3 });
      return line.isVisible();
    }, { intervals: [500, 1000, 1000, 2000] }).toBe(true);
    await page.waitForTimeout(500);
    await shot(page, size, `forecast-walls-${kind}`);
  });

  test('tech cards: Machinery and Metallurgy name the Walls level Siege covers', async ({ page }) => {
    const setup = wallsFixture(), f = soloSeat(setup.fixture);
    f.players[0].techs = [...techsThrough(1), 'Machinery'];
    await loadFixture(page, coreFixture(f), 2);
    await page.locator('.topbar .research-chip').click();
    await expect(page.locator('dialog.tech-modal')).toBeVisible();
    for (const tech of ['Machinery', 'Metallurgy']) {
      const card = page.locator(`dialog.tech-modal .era-card[data-tech="${tech}"]`);
      if (await card.count() === 0) continue;
      await card.scrollIntoViewIfNeeded();
      const chevron = card.locator('.era-details-chevron');
      if (await chevron.count()) await chevron.click();
      await page.mouse.move(2, 2);
      await shot(page, size, `tech-card-${tech.toLowerCase()}`);
    }
  });

  test('How to Play: Walls levels', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /Simultaneous combat/ }).click();
    const p = guide.locator('.guide-page p', { hasText: 'Walls begin at level I.' });
    await p.scrollIntoViewIfNeeded();
    await p.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'guide-walls-levels');
    const s = guide.locator('.guide-page p', { hasText: 'ignores enemy Walls up to a set level' });
    await s.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await shot(page, size, 'guide-siege-levels');
  });
});
