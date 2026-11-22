// Where a map card (tile info, battle forecast) goes: beside the hovered hex, never over it, never
// over the selected unit, and inside the map. Tries right, left, below, above, in that order.
export type Box = { left: number; top: number; right: number; bottom: number };

export function placeCard(
  hex: { x: number; y: number; r: number },
  size: { width: number; height: number },
  area: Box,
  avoid: Box[] = [],
): { left: number; top: number } {
  const gap = 10;
  const clampY = (top: number) => Math.max(area.top + 8, Math.min(area.bottom - size.height - 8, top));
  const clampX = (left: number) => Math.max(area.left + 8, Math.min(area.right - size.width - 8, left));
  const hexBox = { left: hex.x - hex.r, right: hex.x + hex.r, top: hex.y - hex.r, bottom: hex.y + hex.r };
  const candidates = [
    { left: hex.x + hex.r + gap, top: clampY(hex.y - size.height / 2) },
    { left: hex.x - hex.r - gap - size.width, top: clampY(hex.y - size.height / 2) },
    { left: clampX(hex.x - size.width / 2), top: hex.y + hex.r + gap },
    { left: clampX(hex.x - size.width / 2), top: hex.y - hex.r - gap - size.height },
  ];
  const box = (c: { left: number; top: number }) => ({ left: c.left, top: c.top, right: c.left + size.width, bottom: c.top + size.height });
  const hits = (a: Box, b: Box) => a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
  const inside = (b: Box) => b.left >= area.left && b.right <= area.right && b.top >= area.top && b.bottom <= area.bottom;
  const ok = candidates.filter((c) => inside(box(c)) && !hits(box(c), hexBox));
  return ok.find((c) => !avoid.some((a) => hits(box(c), a))) ?? ok[0] ?? { left: clampX(candidates[0].left), top: clampY(candidates[0].top) };
}
