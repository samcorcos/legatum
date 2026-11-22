import type { Faction } from './types';
import { ATLAS } from './map/style';

/** The enlisted ladder is visual only: promotion levels and their combat rules do not change. */
export type RankLevel = 1 | 2 | 3 | 4 | 5 | 6;

export function rankLevel(level: number, faction?: Faction): RankLevel | null {
  if (!Number.isInteger(level) || level < 1 || level > 6 || (level === 6 && faction !== 'Sparta')) return null;
  return level as RankLevel;
}

export function isTopRank(faction: Faction | undefined, level: number): boolean {
  return faction === 'Sparta' ? level === 6 : !!faction && level === 5;
}

/** One engraved cloth badge, shared by the SVG and Canvas renderers. Units are 24 × 34. */
export const RANK_INK = ATLAS.tone.ink;
export const RANK_GOLD = ATLAS.tone.chevron;
export const RANK_EDGE = ATLAS.tone.chevronEdge;
export const RANK_PATCH = 'M3 1 H21 Q23 1 23 3 V30 Q23 33 20 33 H4 Q1 33 1 30 V3 Q1 1 3 1 Z';
export const RANK_LAMBDA = 'M9 21 L12 16 L15 21';

/** Three upward chevrons, then one, two or three curved rockers under them. */
export function rankStrokes(level: RankLevel): { chevrons: string[]; rockers: string[]; device: boolean } {
  const count = Math.min(level, 3);
  const tops = level === 1 ? [10] : level === 2 ? [7, 12] : level === 3 ? [4, 9, 14] : [2, 7, 12];
  const chevrons = tops.slice(0, count).map((top) => `M3 ${top + 7} L12 ${top} L21 ${top + 7}`);
  const rockerTops = level === 4 ? [22] : level === 5 ? [20, 25] : level === 6 ? [18, 23, 28] : [];
  const rockers = rockerTops.map((top) => `M3 ${top} Q12 ${top + 4} 21 ${top}`);
  return { chevrons, rockers, device: level === 6 };
}
