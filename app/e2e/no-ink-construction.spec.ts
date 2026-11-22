// Construction sites are painted stages only: no canvas-drawn crane, workers, scaffold or string line (legatum-ui, 2026-11-01).
// Frames a wonder site on the tile beside its city, Walls going up and an ordinary building in production, at both window sizes.
import { test } from './fixtures';
import { wallsFixture } from '../src/test/wallsFixture';
import { coreFixture } from '../src/test/coreFixture';
import { fromOffset, hexToPixel } from '../src/hex';
import { HEX_SIZE } from '../src/map/grid';
import { SIZES, loadFixture, shot, type GameWindow } from './tour/helpers';

type MapWindow = GameWindow & { legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void } };

const SITE = fromOffset(4, 5);
const cases = [
  { name: 'wonder-stage1', producing: { kind: 'Wonder', building: 'GreatLibrary', tile: SITE }, stored: 90, buildings: ['Walls', 'Castle'] },
  { name: 'wonder-stage2', producing: { kind: 'Wonder', building: 'GreatLibrary', tile: SITE }, stored: 270, buildings: ['Walls', 'Castle'] },
  { name: 'wonder-stage3', producing: { kind: 'Wonder', building: 'GreatLibrary', tile: SITE }, stored: 480, buildings: ['Walls', 'Castle'] },
  { name: 'walls-in-progress', producing: { kind: 'Building', building: 'Walls' }, stored: 40, buildings: [] },
  { name: 'granary-in-progress', producing: { kind: 'Building', building: 'Granary' }, stored: 15, buildings: [] },
];

for (const size of SIZES) test.describe(`construction without ink ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  for (const c of cases) test(c.name, async ({ page }) => {
    test.setTimeout(120_000);
    const setup = wallsFixture(), f = setup.fixture;
    const own = f.cities.find(x => x.id === setup.own)!;
    own.buildings = c.buildings as typeof own.buildings;
    own.producing = c.producing as typeof own.producing;
    own.prod = c.stored; own.productionQueue = [];
    await loadFixture(page, coreFixture(f), 2);
    const a = hexToPixel(own.pos, HEX_SIZE), b = hexToPixel(SITE, HEX_SIZE);
    await page.evaluate(({ x, y }) => { const m = (window as unknown as MapWindow).legatumMap; Object.assign(m.camera, { x, y, zoom: 2.2 }); m.redraw(); }, { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
    await shot(page, size, c.name);
  });
});
