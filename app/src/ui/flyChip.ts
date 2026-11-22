/**
 * The pick animation: a small chip with the technology's name leaves its card and settles into its slot in
 * the queue strip. About 0.35 s, position: fixed and removed afterwards, so it never moves layout; skipped
 * under reduced motion (the card and slot states still change at once). Returns a cancel for cleanup.
 */
const FLY_MS = 360;

export function flyToQueue(host: HTMLElement, from: HTMLElement, to: HTMLElement, label: string): (() => void) | undefined {
  if (typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches) return undefined;
  const chip = document.createElement('div');
  if (typeof chip.animate !== 'function') return undefined;
  const a = from.getBoundingClientRect();
  const b = to.getBoundingClientRect();
  chip.className = 'fly-chip';
  chip.setAttribute('aria-hidden', 'true');
  chip.textContent = label;
  chip.style.left = `${a.left + a.width / 2}px`;
  chip.style.top = `${a.top + 22}px`;
  host.appendChild(chip);
  const dx = b.left + b.width / 2 - (a.left + a.width / 2);
  const dy = b.top + b.height / 2 - (a.top + 22);
  const run = chip.animate([
    { transform: 'translate(-50%, -50%) scale(1)', opacity: 1 },
    { transform: `translate(calc(-50% + ${dx}px), calc(-50% + ${dy}px)) scale(0.82)`, opacity: 0.9, offset: 0.85 },
    { transform: `translate(calc(-50% + ${dx}px), calc(-50% + ${dy}px)) scale(0.82)`, opacity: 0 },
  ], { duration: FLY_MS, easing: 'cubic-bezier(.22,.7,.25,1)', fill: 'forwards' });
  const remove = () => chip.remove();
  // Cancelling an animation rejects its `finished` promise: nobody awaits it here.
  run.finished?.catch(() => undefined);
  run.onfinish = remove;
  run.oncancel = remove;
  return () => { run.cancel(); remove(); };
}
