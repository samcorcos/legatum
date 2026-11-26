import { expect, it } from 'vitest';
import { bannerCityAt3d } from './marks2d';

it('a click on a 3D city banner or its meters finds that city, the banner drawn last winning', () => {
  const hits = [
    { city: 1, parts: [{ x: 0, y: 0, w: 100, h: 20 }, { x: 20, y: 22, w: 60, h: 8 }] },
    { city: 2, parts: [{ x: 50, y: 10, w: 100, h: 20 }] },
  ];
  expect(bannerCityAt3d(hits, 10, 10)).toBe(1);
  expect(bannerCityAt3d(hits, 30, 25)).toBe(1);
  expect(bannerCityAt3d(hits, 60, 15)).toBe(2);
  expect(bannerCityAt3d(hits, 200, 200)).toBeNull();
});
