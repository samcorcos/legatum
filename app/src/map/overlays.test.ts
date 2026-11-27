import { afterEach, describe, expect, it, vi } from 'vitest';
import type { PlayerView } from '../types';
import { LENSES, resolveLens } from './lenses';
import { artFadeFor, buildOverlayTiles, DEFAULT_OVERLAYS, getOverlays, OVERLAYS, OVERLAYS_STORAGE_KEY, parseOverlays, reloadOverlays, setOverlay, toggleOverlay } from './overlays';

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
  it('starts off and remembers Yields across a reload', () => {
    const s = store(); useStorage(s); reloadOverlays();
    expect(getOverlays()).toEqual({ yields: false });
    toggleOverlay('yields');
    expect(reloadOverlays()).toEqual({ yields: true });
    expect(JSON.parse(s.data.get(OVERLAYS_STORAGE_KEY)!)).toEqual({ yields: true });
    setOverlay('yields', false);
    expect(reloadOverlays()).toEqual(DEFAULT_OVERLAYS);
  });
  it('ignores removed and foreign stored options', () => {
    expect(parseOverlays('{"yields":true,"resources":true,"borders":true}')).toEqual({ yields: true });
    for (const raw of ['not json', '[]', '"yields"', '{"yields":"yes"}', null]) expect(parseOverlays(raw)).toEqual(DEFAULT_OVERLAYS);
  });
  it('keeps working in memory when storage is blocked', () => {
    useStorage({ getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } });
    expect(reloadOverlays()).toEqual(DEFAULT_OVERLAYS);
    expect(() => toggleOverlay('yields')).not.toThrow();
    expect(getOverlays()).toEqual({ yields: true });
  });
});

describe('lens and Yields', () => {
  it('keeps one lens per seat and auto-activates Settle for a selected founder', () => {
    expect(LENSES).toEqual(['default', 'settle']);
    expect(OVERLAYS).toEqual(['yields']);
    expect(resolveLens(0, null, { 0: 'settle', 1: 'default' }, null)).toBe('settle');
    expect(resolveLens(1, null, { 0: 'settle', 1: 'default' }, null)).toBe('default');
    expect(resolveLens(0, 7, {}, null)).toBe('settle');
    expect(resolveLens(0, 7, {}, { seat: 0, unit: 7, lens: 'default' })).toBe('default');
  });
  it('keeps Yields through a lens switch and fades map art appropriately', () => {
    useStorage(store()); reloadOverlays(); setOverlay('yields', true);
    const before = getOverlays();
    resolveLens(0, 7, {}, null);
    expect(getOverlays()).toBe(before);
    expect(artFadeFor('default', before)).toBe('overlay');
    expect(artFadeFor('settle', before)).toBe('lens');
  });
});

describe('Yields tiles', () => {
  it('draws nothing while off', () => expect(buildOverlayTiles(fixture(), DEFAULT_OVERLAYS)).toEqual({ yields: [] }));
  it('marks a natural wonder bonus on its yield cluster', () => {
    const v = fixture();
    (v.tiles[5] as { wonder?: string }).wonder = 'GiantsCauseway';
    expect(buildOverlayTiles(v, { yields: true }).yields.find((t) => t.index === 5)?.bonus).toBe('production');
  });
  it('omits unexplored terrain and city tiles', () => {
    const v = fixture();
    v.tiles[5] = { explored: false, visible: false, terrain: null, ruin: true, owner: null };
    expect(buildOverlayTiles(v, { yields: true }).yields.some((t) => t.index === 5)).toBe(false);
  });
});
