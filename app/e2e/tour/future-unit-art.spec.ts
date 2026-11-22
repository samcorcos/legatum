// Review shots for the Future units' own painted art (claude/future-unit-art): the Robotics and Stealth Technology research cards and their
// unlock cards, the Hunter-Killer on the map beside the units around it (normal and far zoom), its unit panel and build card, and the Stealth
// Fighter and Bomber in a city's air wing. Run alone: `npx playwright test e2e/tour/future-unit-art`.
// Shots land in test-results/screen-tour/<size>/future-unit-art/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { withWorldAges } from '../../src/test/vpV2Fixture';
import { momentsForTurn } from '../../src/celebrate/moments';
import { contextFromViews, formatEvents } from '../../src/format';
import { ERA_NAMES, TECHS, TECH_ERA, unitStatsFor } from '../../src/data';
import type { TechId, UnitKind } from '../../src/types';
import { SIZES, frameHall, loadFixture, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

const FUTURE_TECHS: TechId[] = ['Robotics', 'StealthTechnology'];
const KINDS: UnitKind[] = ['Infantry', 'Rifleman', 'HunterKiller', 'AutonomousArmour', 'Tank', 'CombatDrone', 'Cavalry', 'Artillery'];

for (const size of SIZES) test.describe(`future unit art ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the Hunter-Killer on the map beside the units around it, its panel and build card', async ({ page }) => {
    test.setTimeout(180_000);
    const s = improvementCity({ techs: ['Robotics', 'StealthTechnology', 'Flight'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    const at = { q: city.pos.q + 4, r: city.pos.r - 2 };
    const place = (kinds: UnitKind[]) => page.evaluate(({ kinds, at, numbers }) => {
      const game = (window as unknown as GameWindow).legatum;
      const state = game.getState();
      const views = [...state.views!];
      const v = views[state.seat];
      const template = v.units[0];
      const units = kinds.map((kind, i) => {
        const stats = numbers[i];
        return { ...template, id: 700 + i, kind, military: stats.military, hp: 100,
          strength: stats.strength, ranged: stats.ranged, range: stats.range, moves: stats.moves, max_moves: stats.moves,
          pos: { q: at.q + (i % 4) - Math.floor(i / 4), r: at.r + Math.floor(i / 4) } };
      });
      views[state.seat] = { ...v, units: [...v.units, ...units] };
      game.setState({ views, techOpen: false });
    }, { kinds, at, numbers: kinds.map((kind) => unitStatsFor(s.players[0].faction, kind)) });
    await place(KINDS);
    // The sidebar covers the right 340 px, so aim the camera a few tiles right of the cluster to bring it into the open map.
    await frameHall(page, { q: at.q + 3, r: at.r }, 1.25);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'a1-units-normal');
    await frameHall(page, { q: at.q + 5, r: at.r }, 0.6);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'a2-units-far');
    // The Hunter-Killer selected, at normal zoom again: its unit panel and its portrait.
    await frameHall(page, { q: at.q + 3, r: at.r }, 1.25);
    await page.evaluate(({ id, hex }) => {
      const game = (window as unknown as GameWindow).legatum.getState();
      game.set({ techOpen: false });
      game.select({ kind: 'unit', id }, true);
      game.jumpTo(hex);
    }, { id: 700 + KINDS.indexOf('HunterKiller'), hex: { q: at.q + 2, r: at.r } });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'a3-hunter-killer-selected');
    // The city's build list with the Hunter-Killer card.
    await page.evaluate((id) => {
      const game = (window as unknown as GameWindow).legatum.getState();
      game.select({ kind: 'city', id }, true);
    }, city.id);
    const card = page.locator('.city-panel').getByText('Hunter-Killer', { exact: true }).first();
    await card.scrollIntoViewIfNeeded().catch(() => undefined);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'a4-hunter-killer-build-card');
  });

  test('the Robotics and Stealth Technology research cards and their unlock cards', async ({ page }) => {
    test.setTimeout(180_000);
    const f = researchFixture('Switzerland');
    const future = TECH_ERA.Robotics;
    f.players[0].techs = TECHS.filter((t) => TECH_ERA[t] < future && !FUTURE_TECHS.includes(t));
    await loadFixture(page, coreFixture(f), 2);
    // A Future-era empire is long past the first Age, so stage the last two Ages open (render only): the one before the Future
    // era and the Future Age itself, with their names, instead of "No Age has begun". The empire's own era stays as loaded.
    const base = await page.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(); return structuredClone(st.views![st.seat]); });
    const aged = withWorldAges(base);
    const shift = future - 2; // withWorldAges opens Ages 2 and 3 (1-based); the Future era is `future + 1`.
    const staged = {
      ...aged, players: base.players,
      eras: aged.eras.map((e) => ({ ...e, era: e.era + shift, name: ERA_NAMES[e.era + shift - 1] })),
      objective_progress: aged.objective_progress.map((o) => ({ ...o, era: o.era + shift })),
    };
    await page.evaluate((view) => {
      const store = (window as unknown as GameWindow).legatum, st = store.getState();
      store.setState({ views: st.views!.map((x, i) => (i === st.seat ? view : x)) });
    }, staged);
    const nav = page.locator('.topbar .tb-nav-tech');
    const opener = (await nav.isVisible()) ? nav : page.locator('.topbar .research-chip');
    await opener.click();
    await expect(page.locator('dialog.tech-modal')).toBeVisible();
    await expect(page.locator('dialog.tech-modal').getByTestId('research-objectives')).not.toHaveClass(/\bno-obj\b/);
    // Both cards sit side by side in the Future era's row, so one shot shows them both.
    for (const tech of FUTURE_TECHS) await expect(page.locator(`dialog.tech-modal .era-card[data-tech="${tech}"]`)).toBeVisible();
    await page.locator('dialog.tech-modal .era-card[data-tech="Robotics"]').scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'b-research-future-cards');
    await page.keyboard.press('Escape');
    for (const [tech, name] of [['Robotics', 'robotics'], ['StealthTechnology', 'stealth']] as const) {
      await page.evaluate((current) => {
        const st = (window as unknown as GameWindow).legatum.getState();
        st.set({ techOpen: false, views: st.views!.map((v, i) => i === st.seat ? { ...v, research: { ...v.research, current, turns_left: 12 } } : v) });
      }, 'Currency' as TechId);
      const v = await page.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(); return st.views![st.seat]; });
      const events = [{ type: 'TechResearched' as const, player: v.me, tech }];
      const report = { turn: v.turn - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
      await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
      await expect(page.locator('.moment-card.unlock')).toBeVisible();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `c-unlock-card-${name}`);
    }
  });

  test('the Stealth Fighter and Stealth Bomber in a city air wing', async ({ page }) => {
    test.setTimeout(180_000);
    const setup = siegeFixture({ approach: true });
    setup.fixture.players[0].techs.push('Flight', 'StealthTechnology');
    const planes = [{ id: 900, kind: 'Fighter' }, { id: 901, kind: 'StealthFighter' }, { id: 902, kind: 'StealthBomber' }];
    // The save holds u64 bitsets that JSON.parse would round, so the hangar is added as text.
    const save = coreFixture(setup.fixture).replace(/"next_unit_id":\d+/, '"next_unit_id":902');
    const game = save.replace(/^\{/, `{"aircraft":${JSON.stringify(planes.map((p) => ({ ...p, city: setup.own, hp: 100 })))},`);
    await page.addInitScript((g) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game: g }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, game);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('2', { timeout: 60_000 });
    await expect(page.locator('.loading-screen')).toHaveCount(0);
    await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), setup.own);
    await expect(page.locator('.city-hangar')).toContainText('Hangar');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'd1-city-hangar');
    for (const [id, name] of [[901, 'stealth-fighter'], [902, 'stealth-bomber'], [900, 'fighter']] as const) {
      await page.evaluate((planeId) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'air', id: planeId }, true), id);
      await expect(page.getByTestId('air-wing')).toBeVisible();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `d2-air-wing-${name}`);
    }
  });
});
