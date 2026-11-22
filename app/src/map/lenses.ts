import type { PlayerView, Yields } from '../types';
import type { SettlementScore } from '../types';
import { knownThreats } from '../settlement';
import { tileMoveCost } from '../data';
import { hexAt, neighbors, tileIndex } from '../hex';
import type { SiteBand } from '../types';
import { ATLAS } from './style';

/** Lenses recolour the map for one decision, so only one is on at a time. Yields, Resources and Borders are overlays (`overlays.ts`) that stack on any of them. */
export const LENSES = ['default', 'settle', 'movement', 'threat', 'discovery'] as const;
export type MapLens = typeof LENSES[number];
export const LENS_NAMES: Record<MapLens, string> = { default: 'Default', settle: 'Settle', movement: 'Movement', threat: 'Threat', discovery: 'Discovery' };
/** One line per lens (pillar 9); the caveats live in `LENS_CAVEATS`, shown on hover. */
export const LENS_LEGENDS: Record<MapLens, string> = {
  default: 'Choose a lens to inspect the map.',
  settle: 'Better places to found a city are darker green; hover a tile for why; land with no tint cannot be settled',
  movement: 'Cost to enter: 1 easy · 2 rough · × impassable',
  threat: 'Red ×N: N known enemy units can reach or fire here next turn',
  discovery: '? unexplored edge · ◇ known ruin',
};
export const LENS_CAVEATS: Partial<Record<MapLens, string>> = {
  movement: 'A unit with all its moves left can always take one expensive step.',
  threat: 'Combat can stop movement, and forces out of sight are not counted.',
  discovery: 'A ruin out of sight may have been explored since you saw it.',
};

/**
 * The lens on screen. A selected Settler switches to Settle on its own, unless the player chose another lens for that
 * Settler. Overlays are not part of this: they stay as the player set them through every switch.
 */
export function resolveLens(seat: number, selectedSettlerId: number | null, choices: Readonly<Record<number, MapLens>>, settlerLens: { seat: number; unit: number; lens: MapLens } | null): MapLens {
  if (selectedSettlerId === null) return choices[seat] ?? 'default';
  return settlerLens?.seat === seat && settlerLens.unit === selectedSettlerId ? settlerLens.lens : 'settle';
}
export const LENS_ALPHA = 0.48;
export const LENS_FADED_ALPHA = 0.3;
export type LensYield = { kind: 'food' | 'production' | 'gold' | 'science'; value: number };
/** How a tile is painted: a wash (default), or an owner wash with a hairline (the Borders overlay). */
type LensFill = 'owner';
export interface LensTile { index: number; tint: string; label?: string; yields?: LensYield[]; rank?: number; /** Settle: the site's absolute band (Poor to Exceptional), drawn as a paler to deeper green. */ band?: SiteBand; faded: boolean; fill?: LensFill; /** The numeral is drawn only at normal zoom; the far tier keeps the tint alone. */ nearOnly?: boolean }
export const yieldEntries = (y: Yields): LensYield[] => (['food', 'production', 'gold', 'science'] as const).filter((kind) => y[kind] !== 0).map((kind) => ({ kind, value: y[kind] }));

/** Compute only when the view, lens or settlement analysis changes, never on pan/zoom. */
export function buildLensTiles(view: PlayerView, lens: MapLens, sites: SettlementScore[] = []): LensTile[] {
  if (lens === 'default') return [];
  const scores = new Map(sites.map((s) => [tileIndex(s.pos, view.width, view.height), s]));
  const threats = lens === 'threat' ? knownThreats(view) : [];
  const tiles: LensTile[] = [];
  view.tiles.forEach((t, index) => {
    // Deep Survey can disclose a ruin without disclosing its surrounding terrain.
    if (lens === 'discovery' && t.ruin) {
      tiles.push({ index, faded: !t.visible, tint: ATLAS.lens.tint.ruin, label: '◇' });
      return;
    }
    if (!t.explored || !t.terrain) return;
    const base = { index, faded: !t.visible };
    if (lens === 'settle') {
      const site = scores.get(index);
      // Every legal site the core lists is tinted, by its absolute band; the top three also wear a marker.
      // The renderer resolves the band's look token; no score or empty label plate on the map.
      if (site) tiles.push({ ...base, tint: '', band: site.band, rank: site.rank ?? undefined });
    } else if (lens === 'movement') {
      const cost = tileMoveCost(t.terrain, t.road);
      tiles.push({ ...base, tint: cost === null ? ATLAS.lens.tint.blocked : cost === 1 ? ATLAS.lens.tint.easy : ATLAS.lens.tint.rough, label: cost === null ? '×' : String(cost) });
    } else if (lens === 'threat' && threats[index]) {
      tiles.push({ ...base, tint: ATLAS.lens.tint.threat, label: `×${threats[index]}` });
    } else if (lens === 'discovery') {
      const edge = neighbors(hexAt(index, view.width)).some((h) => view.tiles[tileIndex(h, view.width, view.height)]?.explored === false);
      if (t.ruin || edge) tiles.push({ ...base, tint: t.ruin ? ATLAS.lens.tint.ruin : ATLAS.lens.tint.edge, label: t.ruin ? '◇' : '?' });
    }
  });
  return tiles;
}

/** Screen-space layout stays readable at the map's 0.28–2.4 zoom limits. */
export function lensMetrics(hexPixels: number) {
  const compact = hexPixels < 18;
  return { compact, font: compact ? 9 : Math.min(14, Math.max(11, hexPixels * 0.35)) };
}
