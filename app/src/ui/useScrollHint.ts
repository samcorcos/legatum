import { useLayoutEffect, useRef, useState } from 'react';

/** Keep the affordance outside the scrolling content so it cannot cover an action. */
export function useScrollHint() {
  const scroll = useRef<HTMLDivElement>(null);
  const [overflow, setOverflow] = useState(false);
  useLayoutEffect(() => {
    const el = scroll.current;
    if (!el) return;
    // A box squeezed to nothing keeps its last answer: showing the hint shrinks the box, and re-measuring that would flip the hint back and forth forever.
    const measure = () => setOverflow((was) => el.clientHeight === 0 ? was : el.scrollHeight > el.clientHeight + 1);
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    for (const child of el.children) observer.observe(child);
    measure();
    return () => observer.disconnect();
  });
  return { scroll, overflow };
}
