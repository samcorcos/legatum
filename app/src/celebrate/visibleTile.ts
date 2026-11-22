import type { Box } from '../map/placeCard';

/** Nearest unobscured tile center, including a margin for its city label and arrival motion.
 * Candidate coordinates are the screen edges and obstacle edges: any nearest free point lies
 * on one of those boundaries (or at the original coordinate). No camera movement if already clear.
 */
export function visibleTilePoint(origin: { x: number; y: number }, radius: number, area: Box, obstacles: Box[]) {
  const left = area.left + radius, right = area.right - radius;
  const top = area.top + radius, bottom = area.bottom - radius;
  if (left > right || top > bottom) return null;
  const clampX = (x: number) => Math.max(left, Math.min(right, x));
  const clampY = (y: number) => Math.max(top, Math.min(bottom, y));
  const xs = [clampX(origin.x), left, right, ...obstacles.flatMap((b) => [b.left - radius, b.right + radius])];
  const ys = [clampY(origin.y), top, bottom, ...obstacles.flatMap((b) => [b.top - radius, b.bottom + radius])];
  let best: { x: number; y: number } | null = null, distance = Infinity;
  for (const x of xs) for (const y of ys) {
    if (x < left || x > right || y < top || y > bottom) continue;
    if (obstacles.some((b) => x + radius > b.left && x - radius < b.right && y + radius > b.top && y - radius < b.bottom)) continue;
    const d = (x - origin.x) ** 2 + (y - origin.y) ** 2;
    if (d < distance) { best = { x, y }; distance = d; }
  }
  return best;
}
