// Which cached terrain chunks to keep and which lower levels may stand in for a chunk that is not
// drawn yet. Pure bookkeeping (no canvas), so the rules are unit tested in chunkCache.test.ts.

/** Fewest chunks the cache may hold: about 110 MB of canvases. */
export const MAX_CHUNKS = 110;
/** Room kept beyond the chunks the current view needs, so a pan never evicts what it is about to show. */
export const CACHE_SLACK = 16;
/**
 * How many levels (about 1.4x each) below the current one a chunk may be and still stand in for a missing
 * one. A chunk drawn for a much smaller scale has its pixel-floored strokes (borders, grid, coast) far too
 * fat when stretched over the view, which showed as chunk-shaped blotches of fat blue borders (playtest
 * 8.9.2). A chunk drawn for a larger scale only shrinks, so it is fine at any distance.
 */
const FALLBACK_SPAN = 3;

export interface CacheEntry {
  key: string;
  level: number;
  /** Frame the chunk was last rendered or blitted as part of the view (not as a stand-in). */
  used: number;
  /** Needed now: the current level's visible chunks and the ring around them. */
  pinned: boolean;
}

/** The cache's size limit for a view that pins `pinned` chunks: never below the base, never tight around the view. */
export function cacheLimit(pinned: number, base = MAX_CHUNKS): number {
  return Math.max(base, pinned + CACHE_SLACK);
}

/**
 * Keys to drop to bring the cache down to `limit`: oldest first among the chunks that are not pinned.
 * The kept level (the stand-in layer) is never dropped. A view that pins the whole cache evicts nothing.
 */
export function evictionPlan(entries: readonly CacheEntry[], limit: number, keepLevel: number): string[] {
  const over = entries.length - limit;
  if (over <= 0) return [];
  return entries
    .filter((e) => !e.pinned && e.level !== keepLevel)
    .sort((a, b) => a.used - b.used)
    .slice(0, over)
    .map((e) => e.key);
}

/**
 * Levels that may be blitted under the current level while its chunks fill in, farthest first: the stand-in
 * layer (the whole map, always cached, drawn for a typical zoom, so a view is never blank), then every other
 * cached level that is finer than the current one or at most `FALLBACK_SPAN` levels coarser, the nearest in
 * scale on top (a finer level over a coarser one at equal distance). A quick zoom leaves several levels half
 * filled, and each covers what the others miss. The overview level itself needs no stand-in.
 */
export function fallbackLevels(level: number, present: Iterable<number>, overview: number, standIn: number): number[] {
  const out: number[] = [];
  if (level !== overview) out.push(standIn);
  const near = [...new Set(present)].filter((l) => l >= 0 && l !== level && l !== standIn && (l > level || level - l <= FALLBACK_SPAN));
  near.sort((a, b) => Math.abs(b - level) - Math.abs(a - level) || a - b);
  return out.concat(near);
}
