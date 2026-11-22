// Map overlays: options that stack on any lens and stay on until the player turns them off.
// A lens (lenses.ts) recolours the map for one decision, so one at a time; an overlay adds
// information to whatever lens is showing. The choice is the viewer's, kept in this browser.
import { useSyncExternalStore } from 'react';
import type { PlayerView } from '../types';
import { hexAt, neighbors, tileIndex } from '../hex';
import { tileYields } from '../resources';
import { playerColor } from '../theme';
import { yieldEntries, type LensTile, type MapLens } from './lenses';

export const OVERLAYS = ['yields', 'resources', 'borders'] as const;
export type Overlay = typeof OVERLAYS[number];
export type OverlaySet = Readonly<Record<Overlay, boolean>>;

export const OVERLAY_NAMES: Record<Overlay, string> = { yields: 'Yields', resources: 'Resources', borders: 'Borders' };
/** Free keys: R is the ranged attack, so Resources takes O. */
export const OVERLAY_KEYS: Record<Overlay, string> = { yields: 'Y', resources: 'O', borders: 'N' };
export const OVERLAY_LEGENDS: Record<Overlay, string> = {
  yields: '1 icon per point · a number from 5',
  resources: 'Known resources · hover or tap a tile for its bonus',
  borders: 'Owner of each tile · its number is at the border',
};
/** Hover text for each toggle: what it adds, and that it stays on over any lens. */
export const OVERLAY_HINTS: Record<Overlay, string> = {
  yields: "Your faction's yields on every tile, over any lens. Hover or tap a tile for exact values.",
  resources: 'Rings every known resource and shows whose borders it is in, over any lens.',
  borders: 'Marks each empire’s land with its colour and number, over any lens. Faded tiles are out of sight; borders still show their current owner.',
};
export const OVERLAY_EMPTY: Partial<Record<Overlay, string>> = {
  resources: 'No known resources in explored land yet.',
  borders: 'No known borders yet.',
};

/** First run: everything off, so the map starts as it always has. */
export const DEFAULT_OVERLAYS: OverlaySet = { yields: false, resources: false, borders: false };

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

/** The Borders overlay's owner wash under each lens: full over the plain map, lighter under a lens that tints, none under Settle (its bands must stay pure). */
export function ownerWashScale(lens: MapLens | undefined): number {
  return !lens || lens === 'default' ? 1 : lens === 'settle' ? 0 : 0.5;
}
/** The terrain art's fade: Settle fades it to the lens's own level; the Yields overlay fades it a little so icons read in woods and hills. */
export function artFadeFor(lens: MapLens | undefined, overlays: OverlaySet): 'lens' | 'overlay' | 'none' {
  return lens === 'settle' ? 'lens' : overlays.yields ? 'overlay' : 'none';
}

export interface OverlayTiles { yields: LensTile[]; borders: LensTile[] }
const NONE: OverlayTiles = { yields: [], borders: [] };

/** Compute only when the fogged view or an overlay changes, never on pan/zoom. Resources need no tiles: the medallions draw themselves. */
export function buildOverlayTiles(view: PlayerView, overlays: OverlaySet): OverlayTiles {
  if (!overlays.yields && !overlays.borders) return NONE;
  const faction = view.players.find((p) => p.id === view.me)?.faction ?? 'Switzerland';
  const out: OverlayTiles = { yields: [], borders: [] };
  view.tiles.forEach((t, index) => {
    if (!t.explored || !t.terrain) return;
    const base = { index, faded: !t.visible };
    if (overlays.yields) {
      const yields = yieldEntries(tileYields(faction, t));
      if (yields.length) out.yields.push({ ...base, tint: '', yields });
    }
    if (overlays.borders && t.owner !== null) {
      // A light wash with the owner's hairline; the numeral sits only on the border tiles, at normal zoom.
      const owner = t.owner;
      const edge = neighbors(hexAt(index, view.width)).some((h) => view.tiles[tileIndex(h, view.width, view.height)]?.owner !== owner);
      out.borders.push({ ...base, tint: playerColor(owner), fill: 'owner', ...(edge ? { label: String(owner + 1), nearOnly: true } : {}) });
    }
  });
  return out;
}

/** True when the overlay is on but has nothing to mark yet, so the panel can say so instead of looking broken. */
export function overlayEmpty(view: PlayerView, overlay: Overlay, tiles: OverlayTiles): boolean {
  if (overlay === 'resources') return !view.tiles.some((t) => t.explored && t.resource);
  if (overlay === 'borders') return tiles.borders.length === 0;
  return false;
}
