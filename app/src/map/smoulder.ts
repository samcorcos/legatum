import type { CityView } from '../types';
import { hexToPixel } from '../hex';
import { HEX_SIZE, isFar } from './grid';
import type { Camera } from './render';

/** The same visibility test drives both the idle tick and the smoke draw. */
export function smoulderVisible(city: CityView, hp: number, cam: Camera, w: number, h: number): boolean {
  const hexPx = HEX_SIZE * cam.zoom;
  if (isFar(hexPx) || !city.visible || city.max_hp <= 0 || (city.max_hp - hp) / city.max_hp <= 0.02) return false;
  const at = hexToPixel(city.pos, HEX_SIZE);
  const x = (at.x - cam.x) * cam.zoom + w / 2;
  const y = (at.y - cam.y) * cam.zoom + h / 2;
  return x >= -hexPx * 2 && x <= w + hexPx * 2 && y >= -hexPx * 3 && y <= h + hexPx * 2;
}

export function hasVisibleSmoulder(cities: readonly CityView[], cam: Camera, w: number, h: number): boolean {
  return cities.some((city) => smoulderVisible(city, city.hp, cam, w, h));
}
