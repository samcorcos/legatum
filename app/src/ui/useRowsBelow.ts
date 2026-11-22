import { useEffect, useState, type RefObject } from 'react';

/**
 * How many rows (matching `selector`) are not wholly inside a scroll box, so a list cut by the box's edge can say
 * "+N more" instead of ending on a sliced row. Closed disclosures have no layout and count as zero.
 */
export function useRowsBelow(scroller: RefObject<HTMLElement | null>, selector: string): number {
  const [below, setBelow] = useState(0);
  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const measure = () => {
      const edge = el.getBoundingClientRect().bottom;
      setBelow([...el.querySelectorAll<HTMLElement>(selector)].filter((r) => r.checkVisibility() && r.getBoundingClientRect().bottom > edge + 1).length);
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    for (const child of el.children) observer.observe(child);
    el.addEventListener('scroll', measure, { passive: true });
    el.addEventListener('toggle', measure, true);
    return () => { observer.disconnect(); el.removeEventListener('scroll', measure); el.removeEventListener('toggle', measure, true); };
  }, [scroller, selector]);
  return below;
}
