import { describe, expect, it } from 'vitest';
import { airLabelLayout, airRings } from './airOverlay';
import { HEX_SIZE } from './grid';

describe('air range label placement', () => {
  it('keeps five ring labels clear of banners, panels and one another at both review sizes', () => {
    const rings = airRings(['Fighter', 'StealthFighter', 'StealthBomber']);
    expect(rings).toHaveLength(5);
    for (const [w, h] of [[1000, 600], [1440, 900]]) {
      const avoid = [
        { x: w / 2 - 105, y: h / 2 - 68, w: 210, h: 45 },
        { x: w / 2 + 75, y: h / 2 + 25, w: 150, h: 45 },
        { x: w - 190, y: 55, w: 190, h: h - 55 },
      ];
      const placed = airLabelLayout({ center: { q: 0, r: 0 }, rings, candidates: [], pickKind: null, hover: null },
        (x, y) => ({ x: w / 2 + x * 0.65, y: h / 2 + y * 0.65 }), (label) => label.length * 6, { w, h }, avoid);
      expect(placed, `${w}x${h}: all rings should be labelled`).toHaveLength(5);
      for (const [i, p] of placed.entries()) {
        const box = { x: p.x - p.w / 2, y: p.y - 9, w: p.w, h: 18 };
        expect(box.x).toBeGreaterThanOrEqual(0);
        expect(box.x + box.w).toBeLessThanOrEqual(w);
        expect(box.y).toBeGreaterThanOrEqual(0);
        expect(box.y + box.h).toBeLessThanOrEqual(h);
        for (const other of [...avoid, ...placed.slice(0, i).map((q) => ({ x: q.x - q.w / 2, y: q.y - 9, w: q.w, h: 18 }))]) {
          expect(box.x + box.w <= other.x || other.x + other.w <= box.x || box.y + box.h <= other.y || other.y + other.h <= box.y).toBe(true);
        }
      }
    }
  });

  it('never puts a label on the base city tile, even zoomed in with every ring off screen (train-1102 review)', () => {
    const rings = airRings(['Fighter', 'StealthFighter']);
    for (const [w, h] of [[1000, 600], [1440, 900]]) for (const zoom of [1, 1.5, 2, 3, 4]) {
      const toScreen = (x: number, y: number) => ({ x: w / 2 + x * zoom, y: h / 2 + y * zoom });
      // The sidebar on the right and the base city's banner just above its tile.
      const avoid = [{ x: w - 370, y: 0, w: 370, h }, { x: w / 2 - 120, y: h / 2 - 1.3 * HEX_SIZE * zoom, w: 240, h: 30 }];
      const placed = airLabelLayout({ center: { q: 0, r: 0 }, rings, candidates: [], pickKind: null, hover: null },
        toScreen, (label) => label.length * 6, { w, h }, avoid);
      expect(placed.length).toBeGreaterThan(0);
      const a = toScreen(-HEX_SIZE, -HEX_SIZE), b = toScreen(HEX_SIZE, HEX_SIZE);
      for (const p of placed) {
        const box = { x: p.x - p.w / 2, y: p.y - 9, w: p.w, h: 18 };
        expect(box.x + box.w <= a.x || b.x <= box.x || box.y + box.h <= a.y || b.y <= box.y, `${w}x${h} zoom ${zoom}: ${p.ring.label}`).toBe(true);
      }
    }
  });
});
