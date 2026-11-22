import { describe, expect, it, vi } from 'vitest';
import type { CityView, Hex, PlayerView, SettlementScore, UnitView } from '../types';
import { createState, FixtureClient, makeStandardSetup } from '../test/fixtureEngine';
import { coreFixture } from '../test/coreFixture';
import { hexAt, neighbors, tileIndex } from '../hex';
import { buildLensTiles, LENSES, lensMetrics } from './lenses';
import { settleTileNote, withProxySettler } from './settleAdvice';
import { knownLegalSite, knownThreats, reservedSettlements, settlementScores, settlementSites } from '../settlement';
import { drawLens, drawOwnerOverlay } from './drawLens';
import { buildOverlayTiles, DEFAULT_OVERLAYS } from './overlays';
import { MAP_STYLES } from './style';
import { BAND_FLOORS, SITE_BANDS, siteBandOf } from './siteBands';

vi.mock('../icons', () => ({ tintedIcon: () => null }));
let unitTemplate: UnitView;
const fixture = (): PlayerView => {
  const setup = makeStandardSetup('42', 'Celts', 'Switzerland', 'Ada', 'Rival');
  setup.width = 12; setup.height = 10;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i }));
  const client = new FixtureClient();
  client.load(coreFixture(createState(setup)));
  const view = client.view(0);
  unitTemplate = view.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  view.tiles = Array.from({ length: 120 }, () => ({ explored: true, visible: true, terrain: 'Grassland', ruin: false, owner: null }));
  view.units = []; view.cities = []; view.city_gap_blocked = [];
  return view;
};
const settler = (v: PlayerView): UnitView => ({ ...unitTemplate, id: 1, owner: 0, kind: 'Settler', pos: hexAt(54, v.width), max_moves: 2, goto: null, military: false, ranged: 0 });
const city = (id: number, pos: Hex): CityView => ({ id, owner: 0, name: 'Home', is_capital: true, pos,
  population: 2, hp: 200, max_hp: 200, strength: 20, walls: false, great_works: [], visible: true, detail: null });

it('paints exactly one lens, omits unknown terrain and keeps the overlays out of every lens', () => {
  const v = fixture();
  v.tiles[0] = { explored: false, visible: false, terrain: null, ruin: false, owner: null };
  v.tiles[1].owner = 1; v.tiles[1].visible = false; v.tiles[2].resource = 'Iron';
  expect(LENSES).toEqual(['default', 'settle', 'movement', 'threat', 'discovery']);
  for (const lens of LENSES) expect(buildLensTiles(v, lens).some((t) => t.index === 0)).toBe(false);
  expect(buildLensTiles(v, 'default')).toEqual([]);
  // Owned land, yields and resources belong to the overlays: no lens draws a tile for them.
  for (const lens of LENSES) expect(buildLensTiles(v, lens).some((t) => t.yields || t.fill)).toBe(false);
});
it('marks only explored frontier edges and known ruins', () => {
  const v = fixture(); v.tiles[0].explored = false; v.tiles[50].ruin = true;
  const tiles = buildLensTiles(v, 'discovery');
  expect(tiles.find((t) => t.index === 1)?.label).toBe('?');
  expect(tiles.find((t) => t.index === 50)?.label).toBe('◇');
  expect(tiles.some((t) => t.index === 90)).toBe(false);
});
it('shares threat reach with scoring and never extends ranged fire from move endpoints', () => {
  const v = fixture(), u = settler(v);
  const enemy = { ...u, id: 2, owner: 1, kind: 'Archer', military: true, ranged: 15, range: 2 } as UnitView;
  v.units = [enemy];
  const threats = knownThreats(v);
  expect(buildLensTiles(v, 'threat').map((t) => t.index)).toEqual(threats.flatMap((n, i) => n ? [i] : []));
  expect(threats[tileIndex({ q: enemy.pos.q + 3, r: enemy.pos.r }, v.width, v.height)]).toBe(0);
  v.players[1].team = 0;
  expect(buildLensTiles(v, 'threat')).toEqual([]);
});
it('Threat lens counts a visible barbarian raider (no listed player) instead of crashing, whatever borders it crosses', () => {
  const v = fixture(), u = settler(v);
  // Barbarians threaten everyone but the Celts, whom they never attack (the fixture's viewer is the Celts).
  v.players[0].faction = 'Switzerland';
  v.tiles.forEach((t, i) => { if (i % 12 < 5) (t as { owner: number | null }).owner = 1; });
  const raider = { ...u, id: 3, owner: 255, kind: 'Spearman', military: true, ranged: 0, max_moves: 2 } as UnitView;
  v.units = [raider];
  const threats = knownThreats(v);
  expect(threats[tileIndex(raider.pos, v.width, v.height)]).toBe(1);
  // Peace and closed borders do not stop it: it reaches into owned land too.
  expect(threats.some((n, i) => n && v.tiles[i].owner === 1)).toBe(true);
  expect(buildLensTiles(v, 'threat').length).toBeGreaterThan(5);
});
it('Threat lens leaves a barbarian raider out for the Celts, who it never attacks', () => {
  const v = fixture(), u = settler(v);
  const raider = { ...u, id: 3, owner: 255, kind: 'Spearman', military: true, ranged: 0, max_moves: 2 } as UnitView;
  v.units = [raider];
  expect(v.players[0].faction).toBe('Celts');
  expect(knownThreats(v).every((n) => n === 0)).toBe(true);
  expect(buildLensTiles(v, 'threat')).toEqual([]);
});
it('shows every legal site by band with top-three markers, no map scores, and no unreachable recommendations', () => {
  const v = fixture(), u = settler(v); v.units = [u];
  v.tiles[0].terrain = 'Ocean';
  const sites = settlementSites(v, u);
  const tiles = buildLensTiles(v, 'settle', sites);
  expect(tiles.some((t) => t.index === 0)).toBe(false);
  expect(tiles.filter((t) => t.rank)).toHaveLength(3);
  expect(tiles.find((t) => t.rank === 1)?.index).toBe(tileIndex(u.pos, v.width, v.height));
  for (const s of sites) expect(s.breakdown.reduce((n, p) => n + p.value, 0)).toBe(s.score);
  expect(tiles.every((t) => t.label === undefined && t.band !== undefined)).toBe(true);
});
it('leaves an explored site untinted when only the anonymous real-city gap blocks it', () => {
  const v = fixture(), u = settler(v);
  v.units = [u];
  v.city_gap_blocked = [u.pos];
  expect(v.cities).toEqual([]);
  expect(knownLegalSite(v, u.pos)).toBe(false);
  expect(settleTileNote(v, u.pos)).toBe('Too close to another city to found one here');
  const sites = settlementScores(v, u, []);
  expect(sites.some((s) => s.pos.q === u.pos.q && s.pos.r === u.pos.r)).toBe(false);
  expect(buildLensTiles(v, 'settle', sites).some((t) => t.index === tileIndex(u.pos, v.width, v.height))).toBe(false);
  // The other explored land remains scored from the fogged view.
  expect(sites.length).toBeGreaterThan(50);
});
it('with no Settler on the map a stand-in at the capital still tints the explored land and ranks three sites', () => {
  const v = fixture();
  expect(withProxySettler(v)).toBeNull();
  v.cities = [city(5, hexAt(54, v.width))];
  v.tiles[0].terrain = 'Ocean';
  const proxy = withProxySettler(v)!;
  expect(proxy.view.units).toContain(proxy.unit);
  expect(v.units).toEqual([]);
  expect(proxy.unit).toMatchObject({ kind: 'Settler', owner: 0, pos: hexAt(54, v.width) });
  const tiles = buildLensTiles(proxy.view, 'settle', settlementSites(proxy.view, proxy.unit));
  expect(tiles.length).toBeGreaterThan(50);
  expect(tiles.some((t) => t.index === 0)).toBe(false);
  expect(tiles.filter((t) => t.rank)).toHaveLength(3);
  expect(tiles.every((t) => t.band)).toBe(true);
});
it('tints every site the core lists by its absolute band, whatever the other sites are worth', () => {
  const [fair, good, great, exceptional] = BAND_FLOORS;
  const sites: SettlementScore[] = [-40, 0, fair - 1, fair, good - 1, good, great - 1, great, exceptional - 1, exceptional, 900].map((quality, index) => ({
    pos: hexAt(index, 12), score: quality / 2, quality, band: siteBandOf(quality), border_risk: index === 3, rank: null, travel_turns: index === 7 ? null : index,
  }));
  expect(sites.map((s) => s.band)).toEqual(['Poor', 'Poor', 'Poor', 'Fair', 'Fair', 'Good', 'Good', 'Great', 'Great', 'Exceptional', 'Exceptional']);
  const tiles = buildLensTiles(fixture(), 'settle', sites);
  expect(tiles.map((tile) => tile.band)).toEqual(sites.map((s) => s.band));
  expect(tiles.every((tile) => !('risk' in tile))).toBe(true);
  // The same site reads the same on a map full of better ones and on one with none.
  expect(buildLensTiles(fixture(), 'settle', [sites[5]])[0].band).toBe('Good');
  expect(buildLensTiles(fixture(), 'settle', [sites[5], ...sites.slice(8)])[0].band).toBe('Good');
});
it('the tinted tiles are exactly the legal tiles: unreachable, risky and far sites included', () => {
  const v = fixture(), u = settler(v); v.units = [u];
  v.cities = [city(5, hexAt(30, v.width))];
  v.tiles[0].terrain = 'Ocean'; v.tiles[1].terrain = 'Mountains'; v.tiles[2].owner = 1; v.tiles[3].wonder = 'MountFuji';
  // A walled-off pocket nothing can walk to, and fog next to a foreign border (a possible unseen city).
  for (const h of neighbors(hexAt(100, v.width))) v.tiles[tileIndex(h, v.width, v.height)].terrain = 'Mountains';
  v.tiles[60].visible = false; v.tiles[61].owner = 1; v.tiles[61].visible = true;
  const legal = v.tiles.map((_, i) => i).filter((i) => knownLegalSite(v, hexAt(i, v.width)));
  expect(legal.length).toBeGreaterThan(60);
  const tinted = buildLensTiles(v, 'settle', settlementScores(v, u, [])).map((t) => t.index);
  expect(tinted.sort((a, b) => a - b)).toEqual(legal);
  const tiles = buildLensTiles(v, 'settle', settlementScores(v, u, []));
  expect(tiles.find((t) => t.index === 100)?.band).toBeDefined();
  expect(tiles.every((t) => !('risk' in t))).toBe(true);
});
describe.each(['atlas'] as const)('%s screen-space renderer', (look) => {
  it.each([0.28, 0.9, 2.4])('draws all six lenses without errors at zoom %s', (zoom) => {
    const v = fixture(), u = settler(v); v.units = [u]; v.tiles[1].owner = 0; v.tiles[50].ruin = true;
    v.tiles[1].resource = 'Iron'; v.tiles[2].resource = 'Stone'; v.tiles[3].resource = 'Coal'; v.tiles[3].owner = 1;
    const ctx = new Proxy({}, { get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(), set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; } }) as CanvasRenderingContext2D;
    const metrics = lensMetrics(36 * zoom);
    expect(metrics.font).toBeGreaterThanOrEqual(9);
    const everything = buildOverlayTiles(v, { yields: true, resources: true, borders: true });
    for (const pass of ['tint', 'labels'] as const) drawOwnerOverlay(ctx, everything.borders, { x: 180, y: 180, zoom }, 800, 600, v.width, 2, look, pass, 1, new Set([54]));
    for (const lens of LENSES) for (const pass of ['tint', 'labels'] as const) for (const yieldsOn of [false, true]) drawLens(ctx, buildLensTiles(v, lens, settlementSites(v, u)), { x: 180, y: 180, zoom }, 800, 600, v.width, 2, look, pass, new Map([[54, 25]]), [], () => 0, { yieldsOn });
  });
  it('uses five distinct greens, deeper for better sites and readable over every terrain, with only a thin accent line stronger', () => {
    const fills: { color: string; alpha: number }[] = [], edges: { color: string; width: number }[] = [];
    const ctx = new Proxy({ fill: () => fills.push({ color: String(ctx.fillStyle), alpha: ctx.globalAlpha }), stroke: () => edges.push({ color: String(ctx.strokeStyle), width: Number(ctx.lineWidth) }) }, {
      get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(),
      set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; },
    }) as unknown as CanvasRenderingContext2D;
    const lens = MAP_STYLES[look].lens;
    for (const faded of [false, true]) {
      for (const band of SITE_BANDS) drawLens(ctx, [{ index: 0, tint: '', band, faded }], { x: 0, y: 0, zoom: 1 }, 800, 600, 12, 1, look);
    }
    // Poor is a faint tint and the washes deepen with quality, each less covering where the land is only remembered; every edge is a thin line.
    const covers = SITE_BANDS.map((band) => lens.settleAlpha[band]);
    const expected = [...covers, ...covers.map((a) => a * (lens.faded / lens.wash))];
    fills.forEach((fill, i) => expect(fill.alpha).toBeCloseTo(expected[i], 10));
    expect(fills).toHaveLength(10);
    expect(covers[0]).toBeLessThan(0.3);
    expect(covers.every((a, i) => i === 0 || a > covers[i - 1])).toBe(true);
    expect(edges.every((edge) => edge.width <= MAP_STYLES[look].pen.line)).toBe(true);
    expect(new Set(fills.map((fill) => fill.color)).size).toBe(5);
    expect(fills.slice(0, 5).map((fill) => fill.color)).toEqual(SITE_BANDS.map((band) => lens.settle[band]));
  });
  it('draws no risk rim or dots on a site band at any zoom', () => {
    const dashes: unknown[] = [], arcs: number[] = [], strokes: string[] = [];
    const ctx = new Proxy({ setLineDash: (d: unknown) => dashes.push(d), arc: () => arcs.push(1), stroke: () => strokes.push(String(ctx.strokeStyle)) }, {
      get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? (() => undefined),
      set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; },
    }) as unknown as CanvasRenderingContext2D;
    drawLens(ctx, [{ index: 0, tint: '', band: 'Great', faded: false }], { x: 0, y: 0, zoom: 0.9 }, 800, 600, 12, 1, look, 'tint');
    expect(dashes).toEqual([]);
    expect(strokes).toEqual([MAP_STYLES[look].lens.settle.Great]);
    for (const zoom of [0.28, 0.9, 2]) for (const pass of ['tint', 'labels'] as const) drawLens(ctx, [{ index: 0, tint: '', band: 'Great', faded: false }], { x: 0, y: 0, zoom }, 800, 600, 12, 1, look, pass);
    expect(arcs).toEqual([]);
  });
  it('rings the top three in ink with a paper edge, so the ring shows on the deepest green', () => {
    const strokes: { color: string; width: number }[] = [];
    const ctx = new Proxy({ stroke: () => strokes.push({ color: String(ctx.strokeStyle), width: Number(ctx.lineWidth) }) }, {
      get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? (() => undefined),
      set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; },
    }) as unknown as CanvasRenderingContext2D;
    const lens = MAP_STYLES[look].lens;
    drawLens(ctx, [{ index: 0, tint: '', band: 'Exceptional', rank: 1, faded: false }], { x: 0, y: 0, zoom: 0.9 }, 800, 600, 12, 1, look, 'tint');
    const ring = strokes.slice(-2);
    expect(ring.map((s) => s.color)).toEqual([lens.paper, lens.ink]);
    expect(ring[0].width).toBeGreaterThan(ring[1].width);
  });
});

it('Discovery can show a Deep Survey ruin without revealing surrounding terrain', () => {
  const v = fixture();
  v.tiles[0] = { explored: false, visible: false, terrain: null, ruin: true, owner: null };
  expect(buildLensTiles(v, 'discovery').find((t) => t.index === 0)?.label).toBe('◇');
  expect(buildOverlayTiles(v, { ...DEFAULT_OVERLAYS, yields: true }).yields.some((t) => t.index === 0)).toBe(false);
});

it('draws only the Best spot badge above an occupied tile, with no score or empty label plate', () => {
  const text = vi.fn(), rectangles = vi.fn();
  const ctx = new Proxy({ fillText: text, fillRect: rectangles }, { get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(), set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; } }) as unknown as CanvasRenderingContext2D;
  const tile = { index: 0, tint: '', band: 'Great' as const, rank: 1, faded: false };
  const cam = { x: 0, y: 0, zoom: 0.9 };
  drawLens(ctx, [tile], cam, 800, 600, 12, 1, 'atlas', 'tint');
  expect(text).not.toHaveBeenCalled();
  drawLens(ctx, [tile], cam, 800, 600, 12, 1, 'atlas', 'labels', new Map([[0, 25]]));
  expect(text.mock.calls.map((c) => c[0])).toEqual(['Best spot']);
  const r = 36 * cam.zoom;
  expect(rectangles).toHaveBeenCalledTimes(1);
  const [rank] = rectangles.mock.calls as number[][];
  expect(rank[1] + rank[3]).toBeLessThan(300 - r * 0.5);
  expect(rank.slice(2)).toEqual([68, 20]);
});

it('moves the Best spot badge below its hex when a panel covers the space above it', () => {
  const rectangles = vi.fn();
  const ctx = new Proxy({ fillRect: rectangles }, { get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(), set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; } }) as unknown as CanvasRenderingContext2D;
  const tile = { index: 0, tint: '', band: 'Great' as const, rank: 1, faded: false };
  const cam = { x: 0, y: 0, zoom: 0.9 }, r = 36 * cam.zoom;
  // Hex 0 centres at (400, 300); a first-game hint covers everything above its centre.
  drawLens(ctx, [tile], cam, 800, 600, 12, 1, 'atlas', 'labels', new Map([[0, 25]]), [{ x: 0, y: 0, w: 800, h: 300 }]);
  const [badge] = rectangles.mock.calls as number[][];
  expect(badge[1]).toBeGreaterThanOrEqual(300);
  expect(badge[1] + badge[3]).toBeLessThan(300 + r + 30);
});

it('skips a recommendation when every candidate badge position is under a HUD panel', () => {
  const text = vi.fn(), rectangles = vi.fn();
  const ctx = new Proxy({ fillRect: rectangles, fillText: text }, { get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(), set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; } }) as unknown as CanvasRenderingContext2D;
  const tile = { index: 0, tint: '', band: 'Great' as const, rank: 1, faded: false };
  drawLens(ctx, [tile], { x: 0, y: 0, zoom: 0.9 }, 800, 600, 12, 1, 'atlas', 'labels', new Map(), [{ x: 0, y: 0, w: 800, h: 600 }]);
  expect(text).not.toHaveBeenCalled();
  expect(rectangles).not.toHaveBeenCalled();
});

it.each([0.28, 0.9, 2.4])('keeps all three badges distinct at zoom %s', (zoom) => {
  const text = vi.fn(), rectangles = vi.fn();
  const ctx = new Proxy({ fillText: text, fillRect: rectangles }, { get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(), set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; } }) as unknown as CanvasRenderingContext2D;
  drawLens(ctx, [0, 1, 2].map((index) => ({ index, tint: '', band: 'Great' as const, rank: index + 1, faded: false })), { x: 0, y: 0, zoom }, 800, 600, 12, 1, 'atlas', 'labels');
  expect(text.mock.calls.map((c) => c[0])).toEqual(['Best spot', '#2', '#3']);
  const boxes = rectangles.mock.calls as number[][];
  for (const [i, a] of boxes.entries()) for (const b of boxes.slice(i + 1)) {
    expect(a[0] < b[0] + b[2] && a[0] + a[2] > b[0] && a[1] < b[1] + b[3] && a[1] + a[3] > b[1]).toBe(false);
  }
});

it('keeps settle badges clear of all HUD panels at 1000x600', () => {
  const text = vi.fn(), rectangles = vi.fn();
  const ctx = new Proxy({ fillText: text, fillRect: rectangles }, { get: (target, key) => (target as Record<PropertyKey, unknown>)[key] ?? vi.fn(), set: (target, key, value) => { (target as Record<PropertyKey, unknown>)[key] = value; return true; } }) as unknown as CanvasRenderingContext2D;
  const panels = [
    { x: 760, y: 20, w: 220, h: 210 },  // race panel
    { x: 720, y: 360, w: 260, h: 220 }, // This Turn
    { x: 10, y: 20, w: 220, h: 250 },  // lens panel
  ];
  drawLens(ctx, [0, 1, 2].map((index) => ({ index, tint: '', band: 'Great' as const, rank: index + 1, faded: false })), { x: 0, y: 0, zoom: 0.9 }, 1000, 600, 12, 1, 'atlas', 'labels', new Map(), panels);
  const boxes = rectangles.mock.calls as number[][];
  for (const [x, y, w, h] of boxes) for (const panel of panels) {
    expect(x < panel.x + panel.w && x + w > panel.x && y < panel.y + panel.h && y + h > panel.y).toBe(false);
  }
});

it('human Mongols advice honors the other Settlers’ current orders before their persistent routes', () => {
  const v = fixture(), u = settler(v);
  v.players[0].faction = 'Mongols';
  const destination = { q: 8, r: 5 };
  const other = { ...u, id: 3, pos: { q: 7, r: 4 }, goto: { q: 9, r: 4 } };
  v.units = [u, other, { ...other, id: 4, owner: 1 }];
  v.cities = [city(10, { q: 0, r: 0 })];
  expect(reservedSettlements(v, u.id, [])).toEqual([other.goto]);
  expect(reservedSettlements(v, u.id, [{ type: 'SleepUnit', unit: other.id }])).toEqual([]);
  expect(reservedSettlements(v, u.id, [{ type: 'FoundCity', unit: other.id }])).toEqual([other.pos]);
  const reserved = reservedSettlements(v, u.id, [{ type: 'MoveUnit', unit: other.id, to: destination }]);
  expect(reserved).toEqual([destination]);
  const free = settlementSites(v, u).find((s) => s.pos.q === destination.q && s.pos.r === destination.r)!;
  const taken = settlementSites(v, u, reserved).find((s) => s.pos.q === destination.q && s.pos.r === destination.r)!;
  // The spaced-network score is gone: reserved sites only rule out nearby spots, they do not rescore Mongols's.
  for (const site of [free, taken]) expect(site.breakdown.some((p) => p.label.startsWith('Mongols'))).toBe(false);
  expect(taken.score).toBe(free.score);
  expect(taken.rank).toBeNull();
});
