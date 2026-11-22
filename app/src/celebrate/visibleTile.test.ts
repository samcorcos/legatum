import { expect, it } from 'vitest';
import { visibleTilePoint } from './visibleTile';

const area = { left: 8, top: 8, right: 1432, bottom: 892 };
it('clears the full celebrated tile below a 1440px legendary banner with the shortest pan', () => {
  const point = visibleTilePoint({ x: 720, y: 450 }, 60, area, [
    { left: 384, top: 72, right: 1088, bottom: 486 },
    { left: 1104, top: 72, right: 1424, bottom: 450 },
  ]);
  expect(point).toEqual({ x: 720, y: 546 });
});

it('keeps an already-visible tile stationary, including near a map edge', () => {
  expect(visibleTilePoint({ x: 80, y: 750 }, 60, area, [])).toEqual({ x: 80, y: 750 });
});

it('finds a clear phone tile between the banner and turn controls, including an offscreen tile', () => {
  const phone = { left: 8, top: 8, right: 382, bottom: 836 };
  expect(visibleTilePoint({ x: -200, y: -100 }, 48, phone, [
    { left: 0, top: 0, right: 390, bottom: 352 },
    { left: 8, top: 600, right: 382, bottom: 844 },
  ])).toEqual({ x: 56, y: 400 });
});

it('never chooses a tile under the report or a selection panel, and declines a fully covered map', () => {
  const cover = { left: 0, top: 0, right: 1440, bottom: 900 };
  expect(visibleTilePoint({ x: 720, y: 450 }, 60, area, [cover])).toBeNull();
  expect(visibleTilePoint({ x: 720, y: 450 }, 60, area, [
    { ...cover, right: 368 }, { ...cover, left: 728 }, { ...cover, bottom: 486 },
  ])).toEqual({ x: 668, y: 546 });
});
