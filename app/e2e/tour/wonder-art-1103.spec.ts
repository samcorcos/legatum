// Review shots for the painted world wonders (claude/wonder-art-1103): all seven on the map beside a city at both window sizes,
// each close up, the unlock cards that show them and the Guide's seven-wonder row. Run alone:
// `npx playwright test e2e/tour/wonder-art-1103`. Shots land in test-results/screen-tour/<size>/wonder-art-1103/.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { momentsForTurn } from '../../src/celebrate/moments';
import { contextFromViews, formatEvents } from '../../src/format';
import { ring, tileIndex } from '../../src/hex';
import type { BuildingKind, TechId } from '../../src/types';
import { SIZES, frameHall, loadFixture, shot, type GameWindow } from './helpers';

const WORKS: [BuildingKind, string][] = [
  ['Pyramids', 'pyramids'], ['RelayUplink', 'great-library'], ['HagiaSophia', 'hagia-sophia'], ['WorldExchange', 'grand-bazaar'],
  ['PlanetaryFoundry', 'royal-arsenal'], ['EiffelTower', 'eiffel-tower'], ['IterReactor', 'iter-reactor'],
];

for (const size of SIZES) test.describe(`wonder art ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the seven wonders beside a city in two groups, then each close up', async ({ page }) => {
    test.setTimeout(240_000);
    const s = improvementCity({ techs: ['IronWorking', 'Masonry'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    // Six sites spaced three hexes apart on the outer ring (each painting is about three hexes wide).
    const sites = [0, 3, 6, 9, 12, 15].map((i) => ring(city.pos, 3)[i]);
    const place = (works: BuildingKind[]) => page.evaluate(({ indices, cityId, works }) => {
      const game = (window as unknown as GameWindow).legatum;
      const state = game.getState();
      const views = [...state.views!];
      const v = views[state.seat];
      const tiles = [...v.tiles];
      for (const [i, building] of works.entries()) {
        const n = indices[i];
        tiles[n] = { ...tiles[n], terrain: 'Plains', resource: null, landmark: { building, city: cityId, of: v.me } };
      }
      views[state.seat] = { ...v, tiles };
      game.setState({ views, techOpen: false });
    }, { indices: sites.map((h) => tileIndex(h, s.width, s.height)), cityId: city.id, works });
    const groups: BuildingKind[][] = [
      ['Pyramids', 'RelayUplink', 'HagiaSophia', 'WorldExchange', 'PlanetaryFoundry', 'EiffelTower'],
      ['IterReactor', 'PlanetaryFoundry', 'Pyramids', 'EiffelTower', 'RelayUplink', 'HagiaSophia'],
    ];
    for (const [g, works] of groups.entries()) {
      await place(works);
      await frameHall(page, city.pos, 0.9);
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `a1${'ab'[g]}-wonders-normal-${g + 1}`);
      if (g === 0) {
        await frameHall(page, city.pos, 0.4);
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, 'a2-wonders-far');
        // The Pyramids with a unit standing on its tile: the painting moves clear of the counter.
        await page.evaluate(({ at }) => {
          const game = (window as unknown as GameWindow).legatum;
          const state = game.getState();
          const views = [...state.views!];
          const v = views[state.seat];
          views[state.seat] = { ...v, units: v.units.map((u, i) => i === 0 ? { ...u, pos: at } : u) };
          game.setState({ views });
        }, { at: sites[0] });
        await frameHall(page, sites[0], 0.9);
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, 'a3-pyramids-with-unit');
      }
      for (const [i, building] of works.entries()) {
        const name = WORKS.find(([b]) => b === building)![1];
        if (g === 1 && building !== 'IterReactor') continue;
        await frameHall(page, sites[i], 1.15);
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, `b-close-${name}`);
      }
    }
  });

  test('the four new wonders under construction at each painted stage', async ({ page }) => {
    test.setTimeout(240_000);
    const s = improvementCity({ techs: ['IronWorking', 'Masonry'] });
    const city = s.cities[0];
    await loadFixture(page, coreFixture(s), 12);
    const site = ring(city.pos, 2)[4];
    for (const [building, name] of [['Pyramids', 'pyramids'], ['HagiaSophia', 'hagia-sophia'], ['EiffelTower', 'eiffel-tower'], ['IterReactor', 'iter-reactor'], ['RelayUplink', 'great-library']] as const) {
      for (const [stage, progress] of [[1, 200], [2, 550], [3, 850], [4, 1000]] as const) {
        await page.evaluate(({ cityId, building, tile, hex, progress, stage }) => {
          const game = (window as unknown as GameWindow).legatum;
          const state = game.getState();
          const views = [...state.views!];
          const v = views[state.seat];
          const tiles = [...v.tiles];
          tiles[tile] = { ...tiles[tile], terrain: 'Plains', resource: null, landmark: progress >= 1000 ? { building, city: cityId, of: v.me } : null };
          const cities = v.cities.map((c) => c.id === cityId ? { ...c, detail: c.detail ? { ...c.detail, producing: null } : c.detail, construction: progress >= 1000 ? null : { building, tile: hex, stage, progress } } : c);
          views[state.seat] = { ...v, tiles, cities };
          game.setState({ views, techOpen: false });
        }, { cityId: city.id, building, tile: tileIndex(site, s.width, s.height), hex: site, progress, stage });
        await frameHall(page, site, 1.15);
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, `e-construction-${name}-stage${stage}`);
      }
    }
  });

  test('the unlock cards that show the new wonders, and the Guide row', async ({ page }) => {
    test.setTimeout(180_000);
    await loadFixture(page, coreFixture(researchFixture('India')), 2);
    const unlockReport = async (p: Page, tech: TechId) => {
      await p.evaluate((current) => {
        const st = (window as unknown as GameWindow).legatum.getState();
        st.set({ views: st.views!.map((v, i) => i === st.seat ? { ...v, research: { ...v.research, current, turns_left: 12 } } : v) });
      }, 'Currency' as TechId);
      const v = await p.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(); return st.views![st.seat]; });
      const events = [{ type: 'TechResearched' as const, player: v.me, tech }];
      const report = { turn: v.turn - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
      await p.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
    };
    for (const [tech, name] of [['Masonry', 'masonry'], ['Education', 'education'], ['Steel', 'steel'], ['FusionPower', 'fusion']] as const) {
      await unlockReport(page, tech);
      await expect(page.locator('.moment-card.unlock')).toBeVisible();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `c-unlock-card-${name}`);
    }
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /The victory race/ }).click();
    await guide.locator('.guide-page').getByRole('heading', { name: /^World wonders/ }).first().scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'd-guide-world-wonders');
  });
});
