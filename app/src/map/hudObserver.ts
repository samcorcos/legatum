/**
 * Re-run `sample` whenever HUD panels mount, unmount, change class/style/hidden, or resize —
 * replacing permanent 250 ms polling. Coalesces bursts onto one animation frame and ignores
 * mutations on elements the callback itself repositions (such as `.map-compass`).
 */
export function observeHudLayout(
  wrap: HTMLElement,
  panelSelector: string,
  sample: () => void,
  ignoreSelector = '.map-compass, .map3d-flat-chip',
): () => void {
  const root = wrap.closest('.game') ?? wrap;
  const observed = new Set<Element>();
  let raf = 0;

  const syncObserved = (ro: ResizeObserver | null) => {
    if (!ro) return;
    const next = new Set<Element>([wrap, ...document.querySelectorAll(panelSelector)]);
    for (const el of observed) {
      if (!next.has(el)) { ro.unobserve(el); observed.delete(el); }
    }
    for (const el of next) {
      if (!observed.has(el)) { ro.observe(el); observed.add(el); }
    }
  };

  const run = () => {
    raf = 0;
    syncObserved(ro);
    sample();
  };

  const schedule = () => {
    if (raf) return;
    if (typeof requestAnimationFrame === 'function') raf = requestAnimationFrame(run);
    else run();
  };

  const ro = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(schedule) : null;
  const mo = typeof MutationObserver !== 'undefined'
    ? new MutationObserver((records) => {
      for (const rec of records) {
        const target = rec.target instanceof Element ? rec.target : rec.target.parentElement;
        if (target && ignoreSelector && target.closest(ignoreSelector)) continue;
        schedule();
        return;
      }
    })
    : null;

  syncObserved(ro);
  sample();
  mo?.observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ['class', 'style', 'hidden'] });
  window.addEventListener('resize', schedule);

  return () => {
    if (raf && typeof cancelAnimationFrame === 'function') cancelAnimationFrame(raf);
    ro?.disconnect();
    mo?.disconnect();
    observed.clear();
    window.removeEventListener('resize', schedule);
  };
}
