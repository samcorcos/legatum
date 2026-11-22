import { useLayoutEffect, useRef, useState } from 'react';
import { placeCard, type Box } from '../map/placeCard';

/** Where a map card sits: its hovered hex (screen px, relative to the map) and what it must not cover. */
export interface CardAnchor {
  x: number;
  y: number;
  r: number;
  avoid: Box[];
}

/** Measure the card, then place it beside its hex inside the map (see map/placeCard.ts). */
export function usePlacedCard<T extends HTMLElement>(anchor: CardAnchor | undefined) {
  const ref = useRef<T>(null);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  useLayoutEffect(() => {
    if (!anchor) return;
    const el = ref.current;
    const parent = el?.offsetParent as HTMLElement | null;
    if (!el || !parent) return;
    const area = { left: 0, top: 56, right: parent.clientWidth, bottom: parent.clientHeight };
    const next = placeCard(anchor, { width: el.offsetWidth, height: el.offsetHeight }, area, anchor.avoid);
    setPos((p) => (p && p.left === next.left && p.top === next.top ? p : next));
  });
  // No anchor: the card is laid out inline (inside the touch action card).
  if (!anchor) return { ref, style: { position: 'static' as const } };
  return { ref, style: pos ? { left: pos.left, top: pos.top } : { left: -9999, top: -9999, visibility: 'hidden' as const } };
}
