// Map overlays: options that stack on any lens and stay on until the player turns them off.
// A lens (lenses.ts) recolours the map for one decision, so one at a time; an overlay adds
// information to whatever lens is showing. The choice is the viewer's, kept in this browser.
import { useSyncExternalStore } from 'react';
import type { PlayerView } from '../types';
import { tileIndex } from '../hex';
import { tileYields } from '../resources';
import { WONDER_INFO } from '../wonders';
import { yieldEntries, type LensTile, type MapLens } from './lenses';

export const OVERLAYS = ['yields'] as const;
export type Overlay = typeof OVERLAYS[number];
export type OverlaySet = Readonly<Record<Overlay, boolean>>;

export const OVERLAY_NAMES: Record<Overlay, string> = { yields: 'Yields' };
export const OVERLAY_KEYS: Record<Overlay, string> = { yields: 'Y' };
export const OVERLAY_LEGENDS: Record<Overlay, string> = { yields: '1 icon per point · a number from 5' };
export const OVERLAY_HINTS: Record<Overlay, string> = {
  yields: 'Shows what each tile gives you. Hover or tap a tile for exact numbers.',
};
export const DEFAULT_OVERLAYS: OverlaySet = { yields: false };

export const OVERLAYS_STORAGE_KEY = 'legatum.map-overlays.v1';

/** Tolerant parse: a missing, broken or foreign value gives the default for that overlay. */
export function parseOverlays(raw: string | null | undefined): OverlaySet {
  try {
    const data = JSON.parse(raw ?? 'null') as Partial<Record<Overlay, unknown>> | null;
    if (!data || typeof data !== 'object') return DEFAULT_OVERLAYS;
    return Object.fromEntries(OVERLAYS.map((o) => [o, data[o] === true])) as unknown as OverlaySet;
  } catch { return DEFAULT_OVERLAYS; }
}

function readStored(): OverlaySet {
  try { return parseOverlays(globalThis.localStorage?.getItem(OVERLAYS_STORAGE_KEY)); } catch { return DEFAULT_OVERLAYS; }
}

// The in-memory copy is the truth for this session; storage is only a place to remember it, so blocked storage costs nothing but the memory.
let current: OverlaySet = readStored();
const listeners = new Set<() => void>();

export const getOverlays = (): OverlaySet => current;
export function setOverlay(overlay: Overlay, on: boolean): void {
  if (current[overlay] === on) return;
  current = { ...current, [overlay]: on };
  try { globalThis.localStorage?.setItem(OVERLAYS_STORAGE_KEY, JSON.stringify(current)); } catch { /* storage may be blocked */ }
  listeners.forEach((l) => l());
}
export const toggleOverlay = (overlay: Overlay): void => setOverlay(overlay, !current[overlay]);
/** Test hook: forget the session copy and read storage again, as a fresh page load would. */
export function reloadOverlays(): OverlaySet {
  current = readStored();
  listeners.forEach((l) => l());
  return current;
}
const subscribe = (l: () => void) => { listeners.add(l); return () => { listeners.delete(l); }; };
export const useOverlays = (): OverlaySet => useSyncExternalStore(subscribe, getOverlays, getOverlays);

/** The terrain art's fade: Settle fades it to the lens's own level; the Yields overlay fades it a little so icons read in woods and hills. */
export function artFadeFor(lens: MapLens | undefined, overlays: OverlaySet): 'lens' | 'overlay' | 'none' {
  return lens === 'settle' ? 'lens' : overlays.yields ? 'overlay' : 'none';
}

export interface OverlayTiles { yields: LensTile[] }
const NONE: OverlayTiles = { yields: [] };

/** Compute yield clusters when the fogged view or overlay changes, never on pan/zoom. */
export function buildOverlayTiles(view: PlayerView, overlays: OverlaySet): OverlayTiles {
  if (!overlays.yields) return NONE;
  const faction = view.players.find((p) => p.id === view.me)?.faction ?? 'Switzerland';
  const out: OverlayTiles = { yields: [] };
  const towns = new Set(view.cities.map((c) => tileIndex(c.pos, view.width, view.height)));
  view.tiles.forEach((t, index) => {
    if (!t.explored || !t.terrain || towns.has(index)) return;
    const yields = yieldEntries(tileYields(faction, t));
    const bonus = t.wonder ? WONDER_INFO[t.wonder].yield : undefined;
    if (yields.length || bonus) out.yields.push({ index, faded: !t.visible, yields, ...(bonus ? { bonus } : {}) });
  });
  return out;
}
