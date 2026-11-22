import { expect, it } from 'vitest';
import type { CityView, PlayerView } from '../types';
import { tileIndex } from '../hex';
import { unworkedClaimIndexes } from './unworkedClaims';

it('does not hatch the saved Shizuoka tile that Colony 11 can work', () => {
  const saved = { q: 12, r: 11 };
  const stillUnworked = { q: 9, r: 14 };
  const cities = [
    { id: 11, owner: 0, pos: { q: 13, r: 14 }, detail: { territory: [saved, stillUnworked] } },
    { id: 27, owner: 0, pos: { q: 12, r: 10 }, detail: { territory: [] } },
    { id: 28, owner: 2, pos: { q: 8, r: 14 }, detail: { territory: [] } },
  ] as CityView[];
  const view = { me: 0, cities } as PlayerView;
  const hatch = unworkedClaimIndexes(view, 40, 26);
  expect(hatch.has(tileIndex(saved, 40, 26))).toBe(false);
  expect(hatch.has(tileIndex(stillUnworked, 40, 26))).toBe(true);
});
