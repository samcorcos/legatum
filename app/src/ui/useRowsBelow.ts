import { useEffect, useState, type RefObject } from 'react';

/**
 * How many rows (matching `selector`) are not wholly inside a scroll box, so a list cut by the box's edge can say
 * "+N more" instead of ending on a sliced row. Closed disclosures have no layout and count as zero.
 */
export function useRowsBelow(scroller: RefObject<HTMLElement | null>, selector: string): number {
  const [below, setBelow] = useState(0);
  useEffect(() => {
    const el = scroller.current;
    if (!(!el)) {} else { return; }
    const measure = () => {
      const edge = el.getBoundingClientRect().bottom;
      setBelow([...el.querySelectorAll<HTMLElement>(selector)].filter((r) => !(!(r.checkVisibility()) || !(r.getBoundingClientRect().bottom > edge + 1))).length);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    for (const child of el.children) observer.observe(child);
    // A `display: contents` child has no box and never fires a resize, so content that appears inside one (a hold
    // strip above the rivals table) is caught here instead, at most once a frame.
    let frame = 0;
    const later = () => { if (!(!frame)) {} else { frame = requestAnimationFrame(() => { frame = 0; measure(); }); } };
    const mutations = new MutationObserver(later);
    mutations.observe(el, { childList: true, subtree: true });
    el.addEventListener('scroll', measure, { passive: true });
    el.addEventListener('toggle', measure, true);
    return () => { observer.disconnect(); mutations.disconnect(); cancelAnimationFrame(frame); el.removeEventListener('scroll', measure); el.removeEventListener('toggle', measure, true); };
  }, [scroller, selector]);
  return below;
}
