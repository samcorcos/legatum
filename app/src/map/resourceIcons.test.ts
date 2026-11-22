import { afterEach, describe, expect, it, vi } from 'vitest';
import type { PlayerView } from '../types';
import { BANNER_BOTTOM, CITY_UNIT_DROP, HEX_SIZE, tokenHalf } from './grid';
import { COAL_MARK, drawFarMark, drawResourceIcons, GAP, medallionSlots, medallionTop, pickSlot, resourceDetail, STACK_LIFT } from './resourceIcons';

const medallionPlacement = (r: number, city: boolean) => medallionSlots(r, city).top;
import { yieldCluster, yieldIconSize } from './yieldIcons';

vi.mock('../icons', () => ({
  resourceArtSprite: () => ({ width: 32, height: 32 }),
  resourceArtUrl: () => '/art.png',
}));

const ZOOMS = [0.5, 0.6, 0.75, 0.9, 1.2, 1.5, 2, 2.4];
const EPS = 1e-9;

describe('medallion placement', () => {
  it.each(ZOOMS)('at zoom %s sits above a unit token and the yield cluster, at the top of the hex', (zoom) => {
    const r = HEX_SIZE * zoom;
    if (resourceDetail(r) !== 'medallions') return;
    const { dy, radius } = medallionPlacement(r, false);
    const bottom = dy + radius, top = dy - radius; // screen y relative to the hex centre (down is +)
    // A token is a square of half-width tokenHalf centred on the hex.
    expect(bottom).toBeLessThanOrEqual(-tokenHalf(r) - GAP + EPS);
    // The tallest yield cluster (a stack of 4) sits bottom-centre with its bottom at 0.64 r.
    const icon = yieldIconSize(r), cluster = yieldCluster([{ kind: 'food', value: 4 }], icon);
    expect(bottom).toBeLessThan(r * 0.64 - cluster.height);
    // Top of the hex: no higher than just past the top vertex.
    expect(top).toBeGreaterThanOrEqual(-r * 1.07 - EPS);
    expect(dy).toBeLessThan(-r * 0.5);
  });

  it.each(ZOOMS.filter((z) => z >= 0.9))('at zoom %s clears the owner roundel on the token’s top-left corner', (zoom) => {
    // sprites.ts drawPlayerNumber: radius 7.5 at (x - 0.9 tw, y - 0.85 tw).
    const r = HEX_SIZE * zoom, tw = tokenHalf(r);
    const { dy, radius } = medallionPlacement(r, false);
    expect(Math.hypot(0.9 * tw, dy + 0.85 * tw)).toBeGreaterThanOrEqual(radius + 7.5);
  });

  it.each(ZOOMS)('at zoom %s stays above the promotion mark, which never rises above the token', (zoom) => {
    // sprites.ts drawPromotion: the enlisted badge stays below the owner roundel.
    const r = HEX_SIZE * zoom, tw = tokenHalf(r);
    if (resourceDetail(r) !== 'medallions') return;
    const { dy, radius } = medallionPlacement(r, false);
    expect(dy + radius).toBeLessThan(-0.85 * tw + 8);
  });

  it.each(ZOOMS.filter((z) => z >= 0.9))('at zoom %s the side slot (under a banner) stays left of the token, clear of its roundel and inside the hex', (zoom) => {
    const r = HEX_SIZE * zoom, tw = tokenHalf(r);
    for (const city of [false, true]) {
      const { side } = medallionSlots(r, city);
      const unitY = city ? r * CITY_UNIT_DROP : 0; // where a token stands on this tile
      expect(side.dx + side.radius).toBeLessThanOrEqual(-tw - GAP + EPS);
      expect(side.dx - side.radius).toBeGreaterThanOrEqual(-r * 0.95 - EPS);
      expect(Math.hypot(side.dx + 0.9 * tw, side.dy - (unitY - 0.85 * tw))).toBeGreaterThanOrEqual(side.radius + 7.5);
      // Below the banner (its bottom edge is BANNER_BOTTOM r above the centre).
      expect(side.dy - side.radius).toBeGreaterThan(-r * BANNER_BOTTOM);
    }
  });

  it.each(ZOOMS.filter((z) => z >= 0.75))('at zoom %s a two-unit stack keeps the medallion clear of its raised token, that token’s roundel and the upper neighbours', (zoom) => {
    // render.ts: the first token of a two-unit stack is drawn STACK_LIFT tw higher and 1.02 tw right.
    const r = HEX_SIZE * zoom, tw = tokenHalf(r), lift = tw * STACK_LIFT;
    const { stacked } = medallionSlots(r, false);
    const bottom = stacked.dy + stacked.radius, top = stacked.dy - stacked.radius;
    expect(bottom).toBeLessThanOrEqual(-(tw + lift) - GAP + EPS); // the raised token's top edge
    // Its owner roundel: radius 7.5 at (1.02 tw - 0.9 tw, -lift - 0.85 tw).
    expect(Math.hypot(0.12 * tw, stacked.dy + lift + 0.85 * tw)).toBeGreaterThanOrEqual(stacked.radius + 7.5 - EPS);
    expect(top).toBeGreaterThanOrEqual(-r * 1.3 - EPS);
    // The upper neighbours' tokens stand at (±0.866 r, -1.5 r): the medallion stays between them.
    expect(stacked.radius).toBeLessThan(r * 0.866 - tw);
    expect(medallionTop(r, false, true)).toBeCloseTo(-top);
    // A city tile keeps its usual slot (units there drop below the banner instead).
    expect(medallionSlots(r, true).stacked).toEqual(medallionSlots(r, true).top);
  });

  it('moves to the side slot only when a banner covers the top and not the side', () => {
    const r = HEX_SIZE * 0.9, slots = medallionSlots(r, true);
    const banner = { x: -100, y: -r * BANNER_BOTTOM - 24, w: 200, h: 24 };
    expect(pickSlot(0, 0, slots, [])).toBe(slots.top);
    expect(pickSlot(0, 0, slots, [banner])).toBe(slots.side);
    // A banner far to the side leaves the top alone.
    expect(pickSlot(0, 0, slots, [{ ...banner, x: 400 }])).toBe(slots.top);
    // Both covered: stay at the top (under the banner) rather than jump around.
    expect(pickSlot(0, 0, slots, [banner, { x: -60, y: -20, w: 120, h: 80 }])).toBe(slots.top);
  });

  it('is about a third of the hex width at the default zoom, and capped when zoomed in', () => {
    const r = HEX_SIZE * 0.9;
    const d = 2 * medallionPlacement(r, false).radius;
    expect(d / (r * Math.sqrt(3))).toBeGreaterThan(0.3);
    expect(d / (r * Math.sqrt(3))).toBeLessThan(0.4);
    expect(medallionPlacement(HEX_SIZE * 2.4, false).radius).toBeLessThanOrEqual(15);
  });

  it('leaves the upper neighbours’ bottom-centre yield clusters alone', () => {
    // Upper-right neighbour centre: (√3/2 r, -1.5 r); a food 2 + production 1 cluster ends 0.64 r below it.
    for (const zoom of ZOOMS) {
      const r = HEX_SIZE * zoom;
      if (resourceDetail(r) !== 'medallions') continue;
      const { dy, radius } = medallionPlacement(r, false);
      const icon = yieldIconSize(r), c = yieldCluster([{ kind: 'food', value: 2 }, { kind: 'production', value: 1 }], icon);
      const left = Math.sqrt(3) / 2 * r - c.width / 2, bottom = -1.5 * r + 0.64 * r;
      // Closest point of the cluster's box to the medallion centre (0, dy).
      const px = Math.max(left, 0), py = Math.min(Math.max(dy, bottom - c.height), bottom);
      expect(Math.hypot(px, py - dy)).toBeGreaterThan(radius);
    }
  });
});

describe('detail levels', () => {
  it('medallions from the default zoom, a category dot for every resource zoomed out, never nothing', () => {
    expect(resourceDetail(HEX_SIZE * 0.9)).toBe('medallions');
    expect(resourceDetail(HEX_SIZE * 0.5)).toBe('medallions');
    expect(resourceDetail(HEX_SIZE * 0.4)).toBe('dots');
    expect(resourceDetail(HEX_SIZE * 0.28)).toBe('dots');
  });
});

describe('drawResourceIcons', () => {
  afterEach(() => vi.unstubAllGlobals());
  const recorder = () => {
    const calls: { name: string; args: unknown[] }[] = [];
    const ctx = new Proxy({} as Record<PropertyKey, unknown>, {
      get: (t, k) => t[k] ?? ((...args: unknown[]) => { calls.push({ name: String(k), args }); }),
      set: (t, k, v) => { t[k] = v; return true; },
    }) as unknown as CanvasRenderingContext2D;
    return { ctx, calls };
  };
  const view = () => {
    const tiles = Array.from({ length: 48 }, () => ({ explored: true, visible: true, terrain: 'Grassland', ruin: false, owner: null, resource: null as string | null }));
    tiles[0].resource = 'Wheat';
    tiles[1].resource = 'Iron';
    tiles[3].resource = 'Coal'; tiles[3].explored = false; // never drawn: unexplored
    tiles[14].resource = 'Stone';
    return { me: 0, width: 8, height: 6, tiles, units: [], cities: [{ id: 1, owner: 0, pos: { q: -1 + 0, r: 2 } }], players: [] } as unknown as PlayerView;
  };

  it('draws exactly the explored tiles the view lists: medallions up close, a category dot each when zoomed out', () => {
    const sprite = recorder();
    vi.stubGlobal('document', { createElement: () => ({ width: 0, height: 0, getContext: () => sprite.ctx }) });
    const cam = { x: 150, y: 100, zoom: 0.9 };
    const { ctx, calls } = recorder();
    drawResourceIcons(ctx, view(), cam, 800, 600, 2, 'atlas');
    expect(calls.filter((c) => c.name === 'drawImage')).toHaveLength(3);
    calls.length = 0;
    drawResourceIcons(ctx, view(), { ...cam, zoom: 0.4 }, 800, 600, 2, 'atlas');
    expect(calls.filter((c) => c.name === 'drawImage')).toHaveLength(0);
    expect(calls.filter((c) => c.name === 'arc')).toHaveLength(2); // Wheat and Stone (Iron is a square)
    expect(calls.filter((c) => c.name === 'rect')).toHaveLength(1); // Iron
    calls.length = 0;
    drawResourceIcons(ctx, view(), { ...cam, zoom: 0.28 }, 800, 600, 2, 'atlas');
    expect(calls.filter((c) => c.name === 'drawImage')).toHaveLength(0);
    expect(calls.filter((c) => c.name === 'arc')).toHaveLength(2);
    expect(calls.filter((c) => c.name === 'rect')).toHaveLength(1);
  });

  it('zoomed out Iron and Coal differ in shape and value, not only a shared red (user, 2026-10-11)', () => {
    const mark = (res: string) => {
      const { ctx, calls } = recorder();
      const fills: string[] = [];
      Object.defineProperty(ctx, 'fill', { value: () => { fills.push(String((ctx as unknown as { fillStyle: string }).fillStyle)); } });
      drawFarMark(ctx, 50, 50, 4, res, 'atlas');
      return { shapes: calls.map((c) => c.name).filter((n) => n === 'rect' || n === 'arc'), fills };
    };
    const iron = mark('Iron'), coal = mark('Coal'), wheat = mark('Wheat');
    expect(iron.shapes[0]).toBe('rect');
    expect(coal.shapes[0]).toBe('arc');
    expect(iron.fills[0]).not.toBe(coal.fills[0]);
    expect(coal.fills[0]).toBe(COAL_MARK);
    expect(wheat.shapes[0]).toBe('arc');
    expect(wheat.fills[0]).not.toBe(coal.fills[0]);
  });

  it('the six strategic far marks differ in silhouette or value from each other and from a bonus disc', () => {
    const mark = (res: string) => {
      const { ctx, calls } = recorder();
      const fills: string[] = [];
      Object.defineProperty(ctx, 'fill', { value: () => { fills.push(String((ctx as unknown as { fillStyle: string }).fillStyle)); } });
      drawFarMark(ctx, 50, 50, 4, res, 'atlas');
      const lines = calls.filter((c) => c.name === 'lineTo').length;
      return `${calls.some((c) => c.name === 'rect') ? 'rect' : calls.some((c) => c.name === 'arc') ? 'arc' : `poly${lines}`}/${fills[0]}/${JSON.stringify(calls.filter((c) => c.name === 'moveTo').map((c) => c.args))}`;
    };
    const marks = ['Copper', 'Iron', 'Niter', 'Coal', 'Oil', 'Silicon', 'Wheat'].map(mark);
    expect(new Set(marks).size).toBe(marks.length);
    // Distinct by shape family where it matters: diamond, square, hexagon, disc, droplet, crystal.
    expect(marks[0]).toMatch(/^poly3/); // diamond: 3 closing lineTo calls
    expect(marks[2]).toMatch(/^poly5/); // hexagon
    expect(marks[5]).toMatch(/^poly5/); // quartz crystal
    for (const m of marks) expect(m).not.toMatch(/^poly2/);
  });

  it('Niter uses its painted medallion at normal zoom', () => {
    const v = view();
    for (const t of v.tiles) t.resource = null;
    v.tiles[0].resource = 'Niter';
    const { ctx, calls } = recorder();
    drawResourceIcons(ctx, v, { x: 150, y: 100, zoom: 0.9 }, 800, 600, 2, 'atlas');
    expect(calls.filter((c) => c.name === 'drawImage')).toHaveLength(1);
  });

  it('the Resources overlay badges a deposit of a resource you lack that someone else holds or nobody does, not your own or ones you have', () => {
    const v = view();
    for (const t of v.tiles) t.resource = null;
    v.tiles[0].resource = 'Iron'; (v.tiles[0] as { owner: number | null }).owner = 1; // foreign, lacked
    v.tiles[1].resource = 'Iron'; (v.tiles[1] as { owner: number | null }).owner = 0; // yours
    v.tiles[2].resource = 'Coal'; // unclaimed, but Coal is supplied
    (v as unknown as { supply: unknown }).supply = [
      { resource: 'Iron', revealed: true, supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, spare: 0 },
      { resource: 'Coal', revealed: true, supplied: { total: 2, parts: [] }, used: { total: 0, parts: [] }, spare: 2 },
    ];
    const badges = (emphasis: boolean) => {
      const { ctx, calls } = recorder();
      drawResourceIcons(ctx, v, { x: 150, y: 100, zoom: 0.9 }, 800, 600, 2, 'atlas', [], emphasis);
      return calls.filter((c) => c.name === 'fillRect').length; // the badge's exclamation bar
    };
    expect(badges(false)).toBe(0);
    expect(badges(true)).toBe(1);
  });

  it('draws nothing on a map with no known resources', () => {
    const v = view();
    for (const t of v.tiles) t.resource = null;
    const { ctx, calls } = recorder();
    drawResourceIcons(ctx, v, { x: 150, y: 100, zoom: 0.9 }, 800, 600, 2, 'atlas');
    expect(calls.filter((c) => c.name === 'drawImage' || c.name === 'arc')).toEqual([]);
  });
});
