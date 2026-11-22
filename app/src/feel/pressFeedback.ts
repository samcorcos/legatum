const CONTROLS = 'button, a[href], summary, select, input, textarea, label:has(input[type="checkbox"]), label:has(input[type="radio"]), [role="button"], [role="radio"], [role="tab"], [role="option"], [role="checkbox"], [data-tip-static="true"]';

/** Delegated pointer feedback also covers controls mounted after boot. No synthetic clicks
 * or haptics: each command already owns its semantic feedback. A pan cancels the press. */
export function installPressFeedback() {
  const pointers = new Map<number, { el: HTMLElement; x: number; y: number }>();
  const clear = (id: number) => {
    const press = pointers.get(id);
    pointers.delete(id);
    if (press && ![...pointers.values()].some((p) => p.el === press.el)) press.el.removeAttribute('data-touch-pressed');
  };
  const clearAll = () => [...pointers.keys()].forEach(clear);
  const down = (event: PointerEvent) => {
    if (event.button !== 0 || event.pointerType === 'mouse') return;
    clear(event.pointerId);
    const el = (event.target as Element | null)?.closest<HTMLElement>(CONTROLS);
    if (!el || el.matches(':disabled, [aria-disabled="true"], label:has(input:disabled)')) return;
    pointers.set(event.pointerId, { el, x: event.clientX, y: event.clientY });
    el.setAttribute('data-touch-pressed', 'true');
  };
  const move = (event: PointerEvent) => {
    const p = pointers.get(event.pointerId);
    if (p && Math.hypot(event.clientX - p.x, event.clientY - p.y) >= 8) clear(event.pointerId);
  };
  const up = (event: PointerEvent) => clear(event.pointerId);
  window.addEventListener('pointerdown', down, true);
  window.addEventListener('pointermove', move, true);
  window.addEventListener('pointerup', up, true);
  window.addEventListener('pointercancel', up, true);
  window.addEventListener('blur', clearAll);
  window.addEventListener('scroll', clearAll, true);
  document.addEventListener('visibilitychange', clearAll);
  return () => {
    clearAll();
    window.removeEventListener('pointerdown', down, true);
    window.removeEventListener('pointermove', move, true);
    window.removeEventListener('pointerup', up, true);
    window.removeEventListener('pointercancel', up, true);
    window.removeEventListener('blur', clearAll);
    window.removeEventListener('scroll', clearAll, true);
    document.removeEventListener('visibilitychange', clearAll);
  };
}
