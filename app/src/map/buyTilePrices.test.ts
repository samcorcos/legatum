import { expect, it } from 'vitest';
import { tilePriceTags } from './buyTilePrices';

it('shows view prices and mutes unaffordable tags with the shortfall', () => {
  const tags = tilePriceTags([
    { tile: { q: 2, r: 3 }, ring: 2, price: 50 },
    { tile: { q: 3, r: 3 }, ring: 3, price: 60 },
  ], 55, null);
  expect(tags.map(({ price, affordable, reason, queued }) => ({ price, affordable, reason, queued }))).toEqual([
    { price: 50, affordable: true, reason: null, queued: false },
    { price: 60, affordable: false, reason: 'Need 5 more Gold', queued: false },
  ]);
});

it('keeps the queued tile active so clicking it can cancel', () => {
  const tags = tilePriceTags([{ tile: { q: 2, r: 3 }, ring: 2, price: 50 }], 0, { q: 2, r: 3 });
  expect(tags[0]).toMatchObject({ affordable: true, queued: true, reason: null });
});
