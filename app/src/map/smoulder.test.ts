import { expect, it } from 'vitest';
import type { CityView } from '../types';
import { hasVisibleSmoulder, smoulderVisible } from './smoulder';

const city = { id: 1, pos: { q: 0, r: 0 }, visible: true, hp: 97, max_hp: 100 } as CityView;
const camera = { x: 0, y: 0, zoom: 1 };

it('ticks only when damaged-city smoke can actually be drawn', () => {
  expect(smoulderVisible(city, city.hp, camera, 1000, 600)).toBe(true);
  expect(hasVisibleSmoulder([city], camera, 1000, 600)).toBe(true);
  expect(hasVisibleSmoulder([{ ...city, hp: 98 }], camera, 1000, 600)).toBe(false);
  expect(hasVisibleSmoulder([{ ...city, visible: false }], camera, 1000, 600)).toBe(false);
  expect(hasVisibleSmoulder([{ ...city, max_hp: 0 }], camera, 1000, 600)).toBe(false);
  expect(hasVisibleSmoulder([city], { ...camera, zoom: 0.5 }, 1000, 600)).toBe(false);
  expect(hasVisibleSmoulder([city], { ...camera, x: 2000 }, 1000, 600)).toBe(false);
  expect(hasVisibleSmoulder([city], { ...camera, y: 2000 }, 1000, 600)).toBe(false);
  expect(smoulderVisible(city, 100, camera, 1000, 600)).toBe(false);
});
