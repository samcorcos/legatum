/** Bring an idle city's production choices into view when fewer than three whole cards fit. */
export function revealIdleProductionPicker(scroller: HTMLElement, minVisibleCards = 3): boolean {
  const title = scroller.querySelector<HTMLElement>('.cp-choose-title');
  const cards = [...scroller.querySelectorAll<HTMLElement>('.production-choice')];
  if (!title || cards.length === 0) return false;

  const panel = scroller.getBoundingClientRect();
  const head = scroller.querySelector<HTMLElement>('.cp-head');
  const edge = head?.getBoundingClientRect().bottom ?? panel.top;
  const visible = cards.filter((card) => {
    const box = card.getBoundingClientRect();
    return box.top >= edge && box.bottom <= panel.bottom;
  }).length;
  if (visible >= minVisibleCards) return false;

  // Keep the empty-queue line whole when present; the heading and cards follow it directly.
  const queueLine = scroller.querySelector<HTMLElement>('.production-queue');
  let delta = queueLine
    ? queueLine.getBoundingClientRect().bottom - edge
    : title.getBoundingClientRect().top - edge - 8;
  // A damaged city's healing line (or why it is not healing) sits under the head: never scroll it away on open.
  const healing = scroller.querySelector<HTMLElement>('.cp-healing-wrap');
  if (healing) delta = Math.min(delta, healing.getBoundingClientRect().top - edge);
  // Nor the red production warning ("Nothing here fits its automation focus"): it is the reason for the choice.
  const warning = scroller.querySelector<HTMLElement>('.production-warning');
  if (warning) delta = Math.min(delta, warning.getBoundingClientRect().top - edge);
  if (Math.abs(delta) < 1) return false;
  scroller.scrollTop += delta;
  return true;
}

/**
 * Scroll up just enough that `el` starts below the sticky head (which carries the yields, so it is tall at
 * 1000x600): a production warning that appears above the build list must never be cut in half under it.
 */
export function revealBelowStickyHead(scroller: HTMLElement, el: HTMLElement | null): boolean {
  if (!el) return false;
  const head = scroller.querySelector<HTMLElement>('.cp-head');
  const edge = head?.getBoundingClientRect().bottom ?? scroller.getBoundingClientRect().top;
  const delta = el.getBoundingClientRect().top - edge;
  if (delta >= 0) return false;
  scroller.scrollTop += delta;
  return true;
}
