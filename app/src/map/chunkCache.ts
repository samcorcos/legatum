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
export const FALLBACK_SPAN = 3;

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

/**
 * Whether the terrain cache paints one more stage of a chunk now. A forced caller always gets its first stage (the map
 * must fill in however slow the machine); after that, and for a caller that is not forced, a stage runs only if it is
 * expected to end inside the budget, `stageMs` being what a stage has cost lately.
 */
export function stageFits(ran: number, force: boolean, spentMs: number, stageMs: number, limitMs: number): boolean {
  if (force && ran === 0) return true;
  return spentMs + stageMs <= limitMs;
}

/**
 * Whether chunks painted between frames are worth a redraw of the map now. A machine that paints fast shows each one
 * as it lands. A slow one pays more for the redraw than for the chunk, so it shows them `batchMs` apart, and at once
 * when the view has no more to come (`more`).
 */
export function showLanded(more: boolean, slow: boolean, sinceDrawMs: number, batchMs: number): boolean {
  return !more || !slow || sinceDrawMs >= batchMs;
}

/**
 * The level to draw while a zoom gesture runs and the level in use (`cur`) no longer scales to the view.
 * Nothing changes until the level in use is complete, so the one left behind is a finished stand-in. Then it
 * goes straight toward the level the zoom wants and does not stop at each one on the way: every level passed
 * through was a whole view of chunks painted for a few frames' use. Zooming in, it goes at most `FALLBACK_SPAN`
 * levels at once, the farthest a coarser level may stand in from; zooming out there is no limit, since a finer
 * level only shrinks.
 */
export function gestureLevel(cur: number, wanted: number, missing: number): number {
  if (missing > 0 || wanted === cur) return cur;
  return wanted > cur ? Math.min(wanted, cur + FALLBACK_SPAN) : wanted;
}
