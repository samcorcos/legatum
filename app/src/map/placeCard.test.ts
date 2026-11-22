import { expect, it } from 'vitest';
import { placeCard } from './placeCard';

const area = { left: 0, top: 56, right: 1280, bottom: 800 };
const size = { width: 220, height: 160 };
const hits = (p: { left: number; top: number }, b: { left: number; top: number; right: number; bottom: number }) =>
  p.left < b.right && p.left + size.width > b.left && p.top < b.bottom && p.top + size.height > b.top;

it('never covers the hovered hex', () => {
  const hex = { x: 600, y: 400, r: 27 };
  const p = placeCard(hex, size, area);
  expect(hits(p, { left: 573, top: 373, right: 627, bottom: 427 })).toBe(false);
});

it('moves to another side rather than cover the selected unit', () => {
  // The user's report: the tile card sat beside the cursor, over the selected unit next door.
  const hex = { x: 600, y: 400, r: 27 };
  const unit = { left: 640, top: 360, right: 700, bottom: 420 };
  const p = placeCard(hex, size, area, [unit]);
  expect(hits(p, unit)).toBe(false);
  expect(p.left + size.width).toBeLessThanOrEqual(573);
});

it('stays inside the map at the edges', () => {
  const p = placeCard({ x: 1260, y: 780, r: 27 }, size, area);
  expect(p.left + size.width).toBeLessThanOrEqual(area.right);
  expect(p.top + size.height).toBeLessThanOrEqual(area.bottom);
});
