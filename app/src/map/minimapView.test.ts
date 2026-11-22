import { describe, expect, it } from 'vitest';
import type { CityView, PlayerView, UnitView } from '../types';
import { fromOffset, hexToPixel, tileIndex, toOffset } from '../hex';
import { HEX_SIZE, worldBounds } from './render';
import { ATLAS } from './style';
import {
  cityNear, drawMinimapBase, drawMinimapOverlay, minimapCities, minimapLayout, minimapTiles, minimapUnits,
  miniToHex, miniToWorld, viewportRect, worldToMini,
} from './minimapView';

const W = 12, H = 8;
const fixture = () => ({
  me: 0, team: 0, width: W, height: H,
  tiles: Array.from({ length: W * H }, () => ({ explored: true, visible: true, terrain: 'Grassland', ruin: false, owner: null })),
  units: [], cities: [],
  players: [{ id: 0, team: 0 }, { id: 1, team: 1 }],
} as unknown as PlayerView);

const at = (col: number, row: number) => fromOffset(col, row);
const unit = (id: number, owner: number, col: number, row: number, kind = 'Spearman', extra: Partial<UnitView> = {}) =>
  ({ id, owner, kind, pos: at(col, row), automation: null, ...extra }) as unknown as UnitView;
const city = (id: number, owner: number, col: number, row: number, capital = false) =>
  ({ id, owner, name: `City ${id}`, pos: at(col, row), is_capital: capital, original_capital: capital, original_capital_of: capital ? owner : null }) as unknown as CityView;
const tile = (v: PlayerView, col: number, row: number) => v.tiles[tileIndex(at(col, row), v.width, v.height)];

/** A 2D context stand-in that records every fill (at its path's centroid) and every arc center. */
function recorder() {
  const fills: { style: string; x: number; y: number }[] = [];
  const arcs: { x: number; y: number }[] = [];
  let path: [number, number][] = [];
  const ctx = {
    canvas: { width: 1000, height: 1000 },
    fillStyle: '', strokeStyle: '', lineWidth: 1, lineCap: 'butt', globalAlpha: 1,
    setTransform() {}, setLineDash() {}, clearRect() {}, drawImage() {}, fillRect() {}, strokeRect() {}, closePath() {}, stroke() {},
    rect(x: number, y: number, w: number, h: number) { path.push([x + w / 2, y + h / 2]); },
    beginPath() { path = []; },
    moveTo(x: number, y: number) { path.push([x, y]); },
    lineTo(x: number, y: number) { path.push([x, y]); },
    arc(x: number, y: number) { arcs.push({ x, y }); path.push([x, y]); },
    fill() {
      if (!path.length) return;
      const x = path.reduce((a, p) => a + p[0], 0) / path.length, y = path.reduce((a, p) => a + p[1], 0) / path.length;
      fills.push({ style: String(ctx.fillStyle), x, y });
    },
  };
  return { ctx: ctx as unknown as CanvasRenderingContext2D, fills, arcs };
}

describe('minimap knowledge filtering', () => {
  it('never draws unexplored tiles, and dims remembered ones', () => {
    const v = fixture();
    tile(v, 3, 2).explored = false; tile(v, 3, 2).visible = false; tile(v, 3, 2).terrain = null;
    tile(v, 4, 2).visible = false;
    const tiles = minimapTiles(v);
    expect(tiles).toHaveLength(W * H - 1);
    expect(tiles.some((t) => toOffset(t.hex).join() === '3,2')).toBe(false);
    expect(tiles.find((t) => toOffset(t.hex).join() === '4,2')?.dim).toBe(true);
    expect(tiles.find((t) => toOffset(t.hex).join() === '5,2')?.dim).toBe(false);

    // The drawn base paints no hex at the unexplored tile's center.
    const l = minimapLayout(v, 240, 160);
    const { ctx, fills } = recorder();
    drawMinimapBase(ctx, v, l, ATLAS, 1, () => '#ff0000');
    const hidden = hexToPixel(at(3, 2), HEX_SIZE), m = worldToMini(l, hidden.x, hidden.y);
    const r = HEX_SIZE * l.scale;
    expect(fills.filter((f) => Math.hypot(f.x - m.x, f.y - m.y) < r * 0.5)).toHaveLength(0);
    const seen = hexToPixel(at(4, 2), HEX_SIZE), n = worldToMini(l, seen.x, seen.y);
    expect(fills.filter((f) => Math.hypot(f.x - n.x, f.y - n.y) < r * 0.5)).toHaveLength(1);
    expect(fills.length).toBeGreaterThan(0);
  });

  it('shows territory only on explored tiles', () => {
    const v = fixture();
    tile(v, 1, 1).owner = 1;
    tile(v, 2, 1).owner = 1; tile(v, 2, 1).explored = false; tile(v, 2, 1).terrain = null;
    expect(minimapTiles(v).filter((t) => t.owner === 1)).toHaveLength(1);
  });

  it('shows your units anywhere, other units only on tiles in sight', () => {
    const v = fixture();
    tile(v, 6, 3).visible = false; // remembered, not in sight
    tile(v, 7, 3).visible = false; tile(v, 7, 3).explored = false; tile(v, 7, 3).terrain = null;
    v.units = [
      unit(1, 0, 6, 3), // own, in fog: shown
      unit(2, 0, 1, 1, 'Scout'),
      unit(3, 0, 2, 2, 'Spearman', { automation: 'Explore' }),
      unit(4, 1, 5, 5), // foreign, in sight: shown
      unit(5, 1, 6, 3), // foreign, remembered tile: hidden
      unit(6, 1, 7, 3), // foreign, unexplored tile: hidden
    ];
    const units = minimapUnits(v);
    expect(units.map((u) => u.hex).map((h) => toOffset(h).join())).toEqual(['6,3', '1,1', '2,2', '5,5']);
    expect(units.filter((u) => u.scout).length).toBe(2);
    expect(units.find((u) => !u.mine)?.scout).toBe(false);

    // Nothing is drawn where the hidden foreign units stand.
    const l = minimapLayout(v, 240, 160);
    const { ctx, arcs } = recorder();
    drawMinimapOverlay(ctx, {} as CanvasImageSource, v, l, 1, () => '#ff0000', { dark: '#000000', light: '#ffffff' }, null);
    const hiddenAt = (col: number, row: number) => { const p = hexToPixel(at(col, row), HEX_SIZE); return worldToMini(l, p.x, p.y); };
    const own = hiddenAt(6, 3), gone = hiddenAt(7, 3);
    expect(arcs.filter((a) => Math.hypot(a.x - own.x, a.y - own.y) < 0.01)).toHaveLength(1); // only the own unit there
    expect(arcs.filter((a) => Math.hypot(a.x - gone.x, a.y - gone.y) < 0.01)).toHaveLength(0);
  });

  it('lists only cities on explored tiles and marks yours and capitals', () => {
    const v = fixture();
    tile(v, 9, 6).explored = false; tile(v, 9, 6).terrain = null;
    v.cities = [city(1, 0, 2, 2, true), city(2, 1, 8, 5, true), city(3, 1, 9, 6)];
    const cities = minimapCities(v);
    expect(cities.map((c) => c.name)).toEqual(['City 1', 'City 2']);
    expect(cities[0]).toMatchObject({ mine: true, capital: true });
    expect(cities[1]).toMatchObject({ mine: false, capital: true });
  });

  it('marks the original capital after it is captured, and not a replacement capital', () => {
    const v = fixture();
    // City 1: your original capital, now held by empire 1. City 2: empire 1's replacement capital (current capital, not original).
    v.cities = [
      { ...city(1, 1, 2, 2), is_capital: false, original_capital: true, original_capital_of: 0 } as CityView,
      { ...city(2, 1, 8, 5), is_capital: true } as CityView,
    ];
    const cities = minimapCities(v);
    expect(cities[0]).toMatchObject({ capital: true, founder: 0, scored: false });
    expect(cities[1]).toMatchObject({ capital: false });
  });
});

describe('minimap geometry', () => {
  it('fits the whole map in the box, keeping its aspect ratio', () => {
    const v = fixture();
    const b = worldBounds(v);
    const l = minimapLayout(v, 240, 240);
    expect(l.w).toBe(240);
    expect(l.h).toBeLessThanOrEqual(240);
    expect(Math.abs(l.w / l.h - (b.maxX - b.minX) / (b.maxY - b.minY))).toBeLessThan(0.02);
  });

  it('maps a click to the hex under it, round-tripping every tile center', () => {
    const v = fixture();
    const l = minimapLayout(v, 240, 160);
    for (let row = 0; row < H; row++) for (let col = 0; col < W; col++) {
      const p = hexToPixel(at(col, row), HEX_SIZE), m = worldToMini(l, p.x, p.y);
      expect(toOffset(miniToHex(l, m.x, m.y))).toEqual([col, row]);
      const back = miniToWorld(l, m.x, m.y);
      expect(back.x).toBeCloseTo(p.x, 6);
      expect(back.y).toBeCloseTo(p.y, 6);
    }
  });

  it('clamps clicks on the margin to the nearest edge tile', () => {
    const v = fixture();
    const l = minimapLayout(v, 240, 160);
    expect(toOffset(miniToHex(l, -20, -20))).toEqual([0, 0]);
    expect(toOffset(miniToHex(l, l.w + 30, l.h + 30))).toEqual([W - 1, H - 1]);
  });

  it('frames the main map viewport at the right place and size', () => {
    const v = fixture();
    const l = minimapLayout(v, 240, 160);
    const c = hexToPixel(at(6, 4), HEX_SIZE);
    const r = viewportRect(l, { x: c.x, y: c.y, zoom: 2, w: 800, h: 600 });
    expect(r.w).toBeCloseTo(400 * l.scale, 6);
    expect(r.h).toBeCloseTo(300 * l.scale, 6);
    const center = worldToMini(l, c.x, c.y);
    expect(r.x + r.w / 2).toBeCloseTo(center.x, 6);
    expect(r.y + r.h / 2).toBeCloseTo(center.y, 6);
  });

  it('finds the city under the pointer for its name tooltip', () => {
    const v = fixture();
    v.cities = [city(1, 1, 4, 4)];
    const l = minimapLayout(v, 240, 160);
    const p = hexToPixel(at(4, 4), HEX_SIZE), m = worldToMini(l, p.x, p.y);
    expect(cityNear(v, l, m.x + 2, m.y + 1)?.name).toBe('City 1');
    expect(cityNear(v, l, m.x + 40, m.y)).toBeNull();
  });
});
