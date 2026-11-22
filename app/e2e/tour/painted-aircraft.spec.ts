// Review shots for the painted aircraft in the air-strike animation (claude/painted-aircraft): each aircraft kind's plan-view
// sprite crossing over the target and dropping its bombs, at normal and close map zoom, frame-stepped on Playwright's clock.
// Run alone: `npx playwright test e2e/tour/painted-aircraft`. Shots land in test-results/screen-tour/<size>/painted-aircraft/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { ring, tileIndex, hexEq } from '../../src/hex';
import { spawnUnit } from '../../src/test/fixtureEngine';
import { SIZES, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

type MapWindow = GameWindow & { legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void } };
const KINDS = ['Fighter', 'Bomber', 'StealthFighter', 'StealthBomber', 'CombatDrone'] as const;
const ZOOMS = [{ name: 'normal', zoom: 1.0 }, { name: 'close', zoom: 1.8 }] as const;

/** Waypost Seven with one aircraft in the hangar and enemy Soldiers three tiles out (the siege fixture's approach). */
async function start(page: Page, kind: string) {
  const setup = siegeFixture({ approach: true });
  let game: string;
  if (kind === 'CombatDrone') {
    // A Drone Swarm two tiles from the nearest Soldier, ordered to shoot it.
    const f = setup.fixture;
    const foe = f.units.find((u) => u.owner !== 0)!;
    const free = ring(foe.pos, 2).find((h) => f.terrain[tileIndex(h, f.width, f.height)] === 'Plains' && !f.units.some((u) => hexEq(u.pos, h)));
    spawnUnit(f, 0, 'CombatDrone', free!, false);
    game = coreFixture(f);
  } else {
    const aircraft = JSON.stringify([{ id: 900, kind, city: setup.own, hp: 100 }]);
    const save = coreFixture(setup.fixture).replace(/"next_unit_id":\d+/, '"next_unit_id":900');
    game = save.replace(/^\{/, `{"aircraft":${aircraft},`);
  }
  await page.clock.install();
  await page.addInitScript((g) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game: g }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.clock.runFor(800);
}

for (const size of SIZES) test.describe(`painted aircraft ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  for (const kind of KINDS) for (const z of ZOOMS) test(`${kind} air strike, ${z.name} zoom`, async ({ page }) => {
    test.setTimeout(180_000);
    await start(page, kind);
    const at = await page.evaluate((kind) => {
      const g = (window as unknown as GameWindow).legatum, s = g.getState(), v = s.views![s.seat];
      const home = v.cities.find((c) => c.owner === v.me)!.pos;
      const d = (a: { q: number; r: number }, b: { q: number; r: number }) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
      const foe = v.units.filter((u) => u.owner !== v.me && u.military).sort((a, b) => d(a.pos, home) - d(b.pos, home))[0];
      const drone = kind === 'CombatDrone' ? v.units.find((u) => u.kind === 'CombatDrone')! : null;
      s.editOrders((o) => [...o, drone ? { type: 'RangedAttack' as const, unit: drone.id, target: foe.pos } : { type: 'AirStrike' as const, plane: 900, target: foe.pos }]);
      g.getState().select(null);
      return foe.pos;
    }, kind);
    expect(at).toBeTruthy();
    // Centre the target at this zoom (the camera glide runs on the paused clock, so set it directly).
    const frame = () => page.evaluate(({ at, zoom }) => {
      const m = (window as unknown as MapWindow).legatumMap, c = m.camera;
      c.zoom = zoom; m.redraw();
      const p = m.toScreen(at), box = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      Object.assign(c, { x: c.x + (p.x - box.width / 2) / c.zoom, y: c.y + (p.y - box.height / 2) / c.zoom });
      m.redraw();
    }, { at, zoom: z.zoom });
    await frame();
    await page.clock.runFor(300);
    await page.addStyleTag({ content: '.moment-lane{visibility:hidden!important}' });
    await page.mouse.move(5, 300);
    await page.clock.pauseAt(await page.evaluate(() => Date.now()) + 2000);
    await endPlanning(page);
    for (let i = 0; (await page.getByTestId('turn-number').textContent()) !== '3'; i++) {
      if (i > 600) throw new Error('the turn did not resolve');
      await page.waitForTimeout(30);
      await page.clock.runFor(5);
    }
    await page.evaluate(() => {
      const g = (window as unknown as GameWindow).legatum, a = g.getState().anim;
      if (a) g.setState({ anim: { ...a, start: performance.now() } });
    });
    await frame();
    const plan = await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().anim?.plan?.shots.map((s) => ({ kind: s.kind, craft: s.craft, launch: s.launch, flight: s.flight })));
    expect(plan?.some((s) => kind === 'CombatDrone' ? s.kind === 'drone' : s.kind === 'bombs' && s.craft === kind)).toBe(true);
    const bombs = plan!.find((s) => s.kind === (kind === 'CombatDrone' ? 'drone' : 'bombs'))!;
    // The pass runs from launch - 1.3 flights to launch + 1.3 flights; shoot it entering, over the target, and leaving.
    const pass0 = kind === 'CombatDrone' ? bombs.launch : bombs.launch - bombs.flight * 1.3, span = kind === 'CombatDrone' ? bombs.flight : bombs.flight * 2.6;
    let t = 0;
    for (const [k, name] of [[0.22, '1-entering'], [0.5, '2-over-target'], [0.78, '3-leaving']] as const) {
      const to = pass0 + span * k;
      await page.clock.runFor(Math.max(1, to - t)); t = to;
      await shot(page, size, `${kind}-${z.name}-${name}`);
    }
  });
});
