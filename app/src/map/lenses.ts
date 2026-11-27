import type { PlayerView, Yields } from '../types';
import type { SettlementScore } from '../types';
import { tileIndex } from '../hex';
import type { SiteBand } from '../types';

/** One lens is active at a time; Yields can stack over either lens. */
export const LENSES = ['default', 'settle'] as const;
export type MapLens = typeof LENSES[number];
export const LENS_NAMES: Record<MapLens, string> = { default: 'Default', settle: 'Settle' };
/**
 * The lens on screen. A selected Settler switches to Settle on its own, unless the player chose another lens for that
 * Settler. Overlays are not part of this: they stay as the player set them through every switch.
 */
export function resolveLens(seat: number, selectedSettlerId: number | null, choices: Readonly<Record<number, MapLens>>, settlerLens: { seat: number; unit: number; lens: MapLens } | null): MapLens {
  if (selectedSettlerId === null) return choices[seat] ?? 'default';
  return settlerLens?.seat === seat && settlerLens.unit === selectedSettlerId ? settlerLens.lens : 'settle';
}
export type LensYield = { kind: 'food' | 'production' | 'gold' | 'science'; value: number };
export interface LensTile { index: number; yields?: LensYield[]; rank?: number; /** Settle: the site's absolute band (Poor to Exceptional), drawn as a paler to deeper green. */ band?: SiteBand; /** Settle, Rome: founding here makes the Colonia Victrix (core `goals::known_colonia_site`): a gilt ring. */ colonia?: boolean; /** Yields overlay: a natural wonder's boosted yield; where the cluster has room its "+25%" follows that yield's icons instead of a separate pill (`foldedBonuses`). */ bonus?: LensYield['kind']; faded: boolean }
export const yieldEntries = (y: Yields): LensYield[] => (['food', 'production', 'gold', 'science'] as const).filter((kind) => y[kind] !== 0).map((kind) => ({ kind, value: y[kind] }));

/** Compute only when the view, lens or settlement analysis changes, never on pan/zoom. */
export function buildLensTiles(view: PlayerView, lens: MapLens, sites: SettlementScore[] = []): LensTile[] {
  if (lens === 'default') return [];
  const scores = new Map(sites.map((s) => [tileIndex(s.pos, view.width, view.height), s]));
  const tiles: LensTile[] = [];
  view.tiles.forEach((t, index) => {
    if (!t.explored || !t.terrain) return;
    const base = { index, faded: !t.visible };
    const site = scores.get(index);
    if (site) tiles.push({ ...base, band: site.band, rank: site.rank ?? undefined, ...(site.colonia ? { colonia: true } : {}) });
  });
  return tiles;
}

/** Screen-space layout stays readable at the map's 0.28–2.4 zoom limits. */
export function lensMetrics(hexPixels: number) {
  const compact = hexPixels < 18;
  return { compact, font: compact ? 9 : Math.min(14, Math.max(11, hexPixels * 0.35)) };
}
