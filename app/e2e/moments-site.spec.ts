// Construction review frames on chosen tiles. Chromium only.
import { expect, test } from './fixtures';
import { wallsFixture } from '../src/test/wallsFixture';
import { coreFixture } from '../src/test/coreFixture';
import { fromOffset, hexToPixel } from '../src/hex';
import { HEX_SIZE } from '../src/map/grid';
import { SIZES, loadFixture, shot, type GameWindow } from './tour/helpers';

type MapWindow = GameWindow & { legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void } };

for (const size of SIZES) test.describe(`construction site ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('Great Library stages stand on the chosen tile at near and far zoom', async ({ page }) => {
    test.setTimeout(180_000);
    const chosen = fromOffset(4, 5);
    for (const stored of [0, 90, 270, 480, 596]) {
      const setup = wallsFixture(), f = setup.fixture;
      const own = f.cities.find(c => c.id === setup.own)!;
      own.producing = { kind: 'Wonder', building: 'RelayUplink', tile: chosen };
      own.prod = stored; own.productionQueue = [];
      const stagePage = await page.context().newPage();
      await loadFixture(stagePage, coreFixture(f), 2);
      const work = await stagePage.evaluate(() => (window as unknown as GameWindow).legatum.getState().views![0].cities.find((c: any) => c.owner === 0)?.construction);
      expect(work?.tile).toEqual(chosen);
      const p = hexToPixel(chosen, HEX_SIZE);
      for (const [tier, zoom] of [['near', 2], ['far', .4]] as const) {
        await stagePage.evaluate(({ x, y, zoom }) => { const m = (window as unknown as MapWindow).legatumMap; Object.assign(m.camera, { x, y, zoom }); m.redraw(); }, { x: p.x, y: p.y, zoom });
        await shot(stagePage, size, `great-library-${stored}-${tier}`);
      }
      await stagePage.close();
    }
  });

  test('Winterthur Walls advance around the town', async ({ page }) => {
    test.setTimeout(120_000);
    for (const stored of [20, 40, 60]) {
      const setup = wallsFixture(), f = setup.fixture;
      const own = f.cities.find(c => c.id === setup.own)!;
      own.buildings = [];
      own.producing = { kind: 'Building', building: 'Walls' };
      own.prod = stored; own.productionQueue = [];
      const stagePage = await page.context().newPage();
      await loadFixture(stagePage, coreFixture(f), 2);
      const p = hexToPixel(own.pos, HEX_SIZE);
      for (const [tier, zoom] of [['near', 2], ['far', .4]] as const) {
        await stagePage.evaluate(({ x, y, zoom }) => { const m = (window as unknown as MapWindow).legatumMap; Object.assign(m.camera, { x, y, zoom }); m.redraw(); }, { x: p.x, y: p.y, zoom });
        await shot(stagePage, size, `walls-${stored}-${tier}`);
      }
      await stagePage.close();
    }
  });
});
