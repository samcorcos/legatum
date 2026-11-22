import { describe, expect, it } from 'vitest';
import type { PlayerView } from '../types';
import { stationBadgeAt, stationSites } from './yamRelayStation';
import { sealRadius } from './grid';

const view = (halls: PlayerView['courier_halls'], cities: { id: number; owner: number }[]) => ({ courier_halls: halls, cities }) as unknown as PlayerView;

describe('the Yam Relay Station landmark', () => {
  it('joins the public Station list to the cities the viewer knows, and skips a city never seen', () => {
    const v = view([{ of: 1, holder: 2, city: 7 }, { of: 3, holder: 3, city: 9 }], [{ id: 7, owner: 2 }]);
    expect(stationSites(v).map((s) => [s.city.id, s.holder])).toEqual([[7, 2]]);
    expect(stationSites(view(undefined, [{ id: 7, owner: 2 }]))).toEqual([]);
  });

  it('puts the badge up and left of the seal, clear of it, at every far zoom', () => {
    for (const hexPx of [10, 14, 20]) {
      const b = stationBadgeAt({ x: 100, y: 100 }, hexPx);
      const gap = Math.hypot(b.x - 100, b.y - (100 - hexPx * 0.1));
      expect(b.x).toBeLessThan(100);
      expect(b.y).toBeLessThan(100 - hexPx * 0.1);
      expect(gap).toBeGreaterThan(sealRadius(hexPx) * 0.9);
      expect(b.r).toBeGreaterThanOrEqual(6);
    }
  });
});
