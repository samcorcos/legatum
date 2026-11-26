import { create } from 'zustand';

export const MAP_VIEW_KEY = 'legatum.map-view';

/** Which map the game shows: the 3D map or the 2D atlas map. 2D is the default until the 3D map is finished (user, 2026-11-25). */
export type MapViewKind = '2d' | '3d';

/** 3D needs WebGL2 and a pointer that isn't a phone or tablet (phones and tablets stay on 2D for now). */
export function canUse3d(): boolean {
  try {
    if (typeof document === 'undefined' || typeof matchMedia === 'undefined') return false;
    if (matchMedia('(pointer: coarse)').matches) return false;
    return !!document.createElement('canvas').getContext('webgl2');
  } catch {
    return false;
  }
}

export function readMapView(): MapViewKind {
  try { return globalThis.localStorage?.getItem(MAP_VIEW_KEY) === '3d' ? '3d' : '2d'; }
  catch { return '2d'; }
}

export const useMapView = create<{ kind: MapViewKind; set(kind: MapViewKind): void; toggle(): void }>((set, get) => ({
  kind: readMapView(),
  set: (kind) => {
    set({ kind });
    try { globalThis.localStorage?.setItem(MAP_VIEW_KEY, kind); } catch { /* keep the session choice */ }
  },
  toggle: () => get().set(get().kind === '3d' ? '2d' : '3d'),
}));

