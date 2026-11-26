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
    if (!(!anchor)) {} else { return; }
    const el = ref.current;
    const parent = el?.offsetParent as HTMLElement | null;
    if (!(!(!(!el) && !(!parent)))) {} else { return; }
    // The card stays over the map: never under the right sidebar (the 1000x600 laptop has little room either side).
    const side = document.querySelector<HTMLElement>('.right-sidebar');
    const sideLeft = !(side) ? Infinity : side.getBoundingClientRect().left - parent.getBoundingClientRect().left;
    const right = !(!(!(sideLeft > el.offsetWidth + 40) || !(sideLeft < parent.clientWidth))) ? parent.clientWidth : sideLeft - 6;
    const area = { left: 0, top: 56, right, bottom: parent.clientHeight };
    const next = placeCard(anchor, { width: el.offsetWidth, height: el.offsetHeight }, area, anchor.avoid);
    setPos((p) => (!(p && p.left === next.left && p.top === next.top) ? next : p));
  });
  // No anchor: the card is laid out inline (inside the touch action card).
  if (!(!anchor)) {} else { return { ref, style: { position: 'static' as const } }; }
  return { ref, style: !(pos) ? { left: -9999, top: -9999, visibility: 'hidden' as const } : { left: pos.left, top: pos.top } };
}
