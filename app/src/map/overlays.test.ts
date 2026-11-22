import { afterEach, describe, expect, it, vi } from 'vitest';
import type { PlayerView } from '../types';
import { hexAt, neighbors, tileIndex } from '../hex';
import { LENSES, resolveLens } from './lenses';
import {
  artFadeFor, buildOverlayTiles, DEFAULT_OVERLAYS, getOverlays, OVERLAYS, OVERLAYS_STORAGE_KEY, overlayEmpty, ownerWashScale,
  parseOverlays, reloadOverlays, setOverlay, toggleOverlay,
} from './overlays';
import { drawOwnerOverlay } from './drawLens';
import { emphasisRing } from './resourceIcons';

vi.mock('../icons', () => ({ tintedIcon: () => null, resourceArtSprite: () => null }));

const fixture = () => ({
  me: 0, team: 0, width: 12, height: 10,
  tiles: Array.from({ length: 120 }, () => ({ explored: true, visible: true, terrain: 'Grassland', ruin: false, owner: null })),
  units: [], cities: [], players: [{ id: 0, team: 0, faction: 'Celts' }, { id: 1, team: 1, faction: 'Switzerland' }],
} as unknown as PlayerView);

const store = (initial: Record<string, string> = {}) => {
  const data = new Map(Object.entries(initial));
  return { getItem: (k: string) => data.get(k) ?? null, setItem: (k: string, v: string) => { data.set(k, v); }, removeItem: (k: string) => { data.delete(k); }, data };
};
const useStorage = (storage: unknown) => vi.stubGlobal('localStorage', storage);

afterEach(() => { vi.unstubAllGlobals(); useStorage(store()); reloadOverlays(); });

describe('overlay state', () => {
  it('starts with every overlay off on a first run', () => {
    useStorage(store());
    expect(reloadOverlays()).toEqual({ yields: false, resources: false, borders: false });
    expect(DEFAULT_OVERLAYS).toEqual(getOverlays());
  });
  it('toggles on and off, and remembers the choice across a reload', () => {
    const s = store(); useStorage(s); reloadOverlays();
    toggleOverlay('yields');
    expect(getOverlays()).toEqual({ yields: true, resources: false, borders: false });
    expect(JSON.parse(s.data.get(OVERLAYS_STORAGE_KEY)!)).toEqual({ yields: true, resources: false, borders: false });
    setOverlay('borders', true);
    expect(reloadOverlays()).toEqual({ yields: true, resources: false, borders: true });
    toggleOverlay('yields');
    expect(reloadOverlays()).toEqual({ yields: false, resources: false, borders: true });
  });
  it('writes nothing when a toggle changes nothing', () => {
    const s = store(); useStorage(s); reloadOverlays();
    setOverlay('resources', false);
    expect(s.data.has(OVERLAYS_STORAGE_KEY)).toBe(false);
  });
  it('keeps working in memory when storage is blocked or holds rubbish', () => {
    useStorage({ getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } });
    expect(reloadOverlays()).toEqual(DEFAULT_OVERLAYS);
    expect(() => toggleOverlay('resources')).not.toThrow();
    expect(getOverlays().resources).toBe(true);
    for (const raw of ['not json', '[]', '"yields"', '{"yields":"yes"}', null, undefined, '{"nope":true}']) expect(parseOverlays(raw as string | null)).toEqual(DEFAULT_OVERLAYS);
    expect(parseOverlays('{"yields":true,"borders":false,"extra":true}')).toEqual({ yields: true, resources: false, borders: false });
  });
});

describe('lenses stay exclusive and overlays stay put', () => {
  it('a seat has one lens at a time, defaulting to Default', () => {
    expect(resolveLens(0, null, {}, null)).toBe('default');
    expect(resolveLens(0, null, { 0: 'threat', 1: 'movement' }, null)).toBe('threat');
    expect(resolveLens(1, null, { 0: 'threat', 1: 'movement' }, null)).toBe('movement');
    for (const o of OVERLAYS) expect((LENSES as readonly string[]).includes(o)).toBe(false);
  });
  it('a selected Settler turns Settle on by itself, unless the player picked another lens for that Settler', () => {
    expect(resolveLens(0, 7, { 0: 'threat' }, null)).toBe('settle');
    expect(resolveLens(0, 7, {}, { seat: 0, unit: 7, lens: 'movement' })).toBe('movement');
    expect(resolveLens(0, 8, {}, { seat: 0, unit: 7, lens: 'movement' })).toBe('settle');
    expect(resolveLens(1, 7, {}, { seat: 0, unit: 7, lens: 'movement' })).toBe('settle');
  });
  it('auto-activating a lens leaves the overlays as the player set them', () => {
    useStorage(store()); reloadOverlays();
    setOverlay('yields', true); setOverlay('borders', true);
    const before = getOverlays();
    resolveLens(0, 7, { 0: 'threat' }, null);
    resolveLens(0, null, { 0: 'threat' }, null);
    expect(getOverlays()).toBe(before);
    expect(getOverlays()).toEqual({ yields: true, resources: false, borders: true });
  });
  it('the Settle lens keeps its colours pure: no owner wash, and only the Yields overlay fades the art a little', () => {
    expect(ownerWashScale('default')).toBe(1);
    expect(ownerWashScale(undefined)).toBe(1);
    expect(ownerWashScale('settle')).toBe(0);
    for (const lens of ['movement', 'threat', 'discovery'] as const) expect(ownerWashScale(lens)).toBeGreaterThan(0);
    expect(artFadeFor('settle', DEFAULT_OVERLAYS)).toBe('lens');
    expect(artFadeFor('settle', { ...DEFAULT_OVERLAYS, yields: true })).toBe('lens');
    expect(artFadeFor('default', { ...DEFAULT_OVERLAYS, yields: true })).toBe('overlay');
    expect(artFadeFor('threat', { ...DEFAULT_OVERLAYS, resources: true, borders: true })).toBe('none');
  });
});

describe('overlay tiles', () => {
  it('draw nothing while every overlay is off', () => {
    expect(buildOverlayTiles(fixture(), DEFAULT_OVERLAYS)).toEqual({ yields: [], borders: [] });
  });
  it('Yields: faction tile yields, one entry per tile that has any, and an improved resource’s bonus', () => {
    const v = fixture(); v.tiles[0].terrain = 'Hills'; v.tiles[1].terrain = 'Ocean'; v.tiles[2].resource = 'Wheat';
    v.tiles[5] = { explored: false, visible: false, terrain: null, ruin: true, owner: null };
    const yields = (v: PlayerView) => buildOverlayTiles(v, { ...DEFAULT_OVERLAYS, yields: true }).yields;
    const bare = yields(v);
    expect(bare.find((t) => t.index === 0)?.yields).toEqual([{ kind: 'production', value: 2 }]);
    expect(bare.some((t) => t.index === 5)).toBe(false);
    const farm = bare.find((t) => t.index === 2)!.yields!.find((y) => y.kind === 'food')!.value;
    v.tiles[2].improvement = { kind: 'Farm' };
    expect(yields(v).find((t) => t.index === 2)!.yields!.find((y) => y.kind === 'food')!.value).toBe(farm + 1);
    expect(buildOverlayTiles(v, { ...DEFAULT_OVERLAYS, yields: true }).borders).toEqual([]);
  });
  it('Borders: a wash on every owned tile, the number only on border tiles and only at normal zoom', () => {
    const v = fixture();
    const inner = tileIndex({ q: 5, r: 4 }, v.width, v.height);
    for (const h of [{ q: 5, r: 4 }, ...neighbors({ q: 5, r: 4 })]) v.tiles[tileIndex(h, v.width, v.height)].owner = 1;
    v.tiles[1].owner = 1; v.tiles[1].visible = false;
    const tiles = buildOverlayTiles(v, { ...DEFAULT_OVERLAYS, borders: true }).borders;
    expect(tiles).toHaveLength(8);
    expect(tiles.every((t) => t.fill === 'owner')).toBe(true);
    expect(tiles.find((t) => t.index === inner)?.label).toBeUndefined();
    expect(tiles.find((t) => t.index === 1)).toMatchObject({ label: '2', faded: true, nearOnly: true });
    expect(tiles.filter((t) => t.label === '2' && t.nearOnly)).toHaveLength(7);
    const text = (zoom: number, busy: number[] = []) => {
      const calls: string[] = [];
      const ctx = new Proxy({}, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? ((..._: unknown[]) => { calls.push(String(k)); }), set: (t, k, val) => { (t as Record<PropertyKey, unknown>)[k] = val; return true; } }) as CanvasRenderingContext2D;
      drawOwnerOverlay(ctx, tiles, { x: 300, y: 200, zoom }, 800, 600, v.width, 1, 'atlas', 'labels', 1, new Set(busy));
      return calls.filter((c) => c === 'fillText').length;
    };
    expect(text(1)).toBe(7);
    expect(text(1, [tiles.find((t) => t.label)!.index])).toBe(6); // a token or city brings its own number
    expect(text(0.5)).toBe(0);
  });
  it('Borders under Settle leaves the band colours alone: a hairline, no wash', () => {
    const tiles = buildOverlayTiles((() => { const v = fixture(); v.tiles[3].owner = 1; return v; })(), { ...DEFAULT_OVERLAYS, borders: true }).borders;
    const fills: number[] = [], strokes: number[] = [];
    const ctx = new Proxy({ fill: () => fills.push(ctx.globalAlpha), stroke: () => strokes.push(ctx.globalAlpha) }, { get: (t, k) => (t as Record<PropertyKey, unknown>)[k] ?? vi.fn(), set: (t, k, val) => { (t as Record<PropertyKey, unknown>)[k] = val; return true; } }) as unknown as CanvasRenderingContext2D;
    drawOwnerOverlay(ctx, tiles, { x: 300, y: 200, zoom: 1 }, 800, 600, 12, 1, 'atlas', 'tint', ownerWashScale('settle'));
    expect(fills).toEqual([]);
    expect(strokes).toHaveLength(1);
    drawOwnerOverlay(ctx, tiles, { x: 300, y: 200, zoom: 1 }, 800, 600, 12, 1, 'atlas', 'tint', ownerWashScale('default'));
    expect(fills).toHaveLength(1);
    expect(fills[0]).toBeLessThanOrEqual(0.3);
  });
  it('says so when an overlay has nothing to mark', () => {
    const v = fixture();
    const tiles = buildOverlayTiles(v, { yields: true, resources: true, borders: true });
    expect(overlayEmpty(v, 'resources', tiles)).toBe(true);
    expect(overlayEmpty(v, 'borders', tiles)).toBe(true);
    expect(overlayEmpty(v, 'yields', tiles)).toBe(false);
    v.tiles[3].resource = 'Iron'; v.tiles[4].owner = 0;
    const filled = buildOverlayTiles(v, { yields: true, resources: true, borders: true });
    expect(overlayEmpty(v, 'resources', filled)).toBe(false);
    expect(overlayEmpty(v, 'borders', filled)).toBe(false);
    void hexAt;
  });
});

describe('Resources overlay rings', () => {
  it('tell whose land a resource is in by shape, not colour: thick solid yours, thin unclaimed, double foreign', () => {
    const yours = emphasisRing(0, 0), free = emphasisRing(null, 0), foreign = emphasisRing(1, 0);
    expect([yours.owned, free.owned, foreign.owned]).toEqual(['yours', 'unclaimed', 'foreign']);
    expect(yours.width).toBeGreaterThan(free.width);
    expect([yours.double, free.double, foreign.double]).toEqual([false, false, true]);
  });
});
