import { Children, isValidElement, useEffect, useId, useLayoutEffect, useRef, useState, type ReactElement, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import { useCoarsePointer } from './useCoarsePointer';
import { BreakdownTable, Icon, YIELD_LABEL, YieldSourceTable, YieldsAfter } from './bits';
import { Amounts } from './Amounts';

type Placement = 'bottom' | 'top' | 'left' | 'right';
type Mode = 'hover' | 'pinned' | null;

/** Hover explanations wait this long, so sweeping the pointer across the screen shows nothing. */
export const SHOW_DELAY = 450;
/** Once one tooltip is open (or just closed), moving to the next shows it at once. */
const WARM_MS = 400;

type OpenTip = { close: () => void; mode: () => Mode };
/** The one tooltip or breakdown currently shown anywhere on the page. */
let openTip: OpenTip | null = null;
let lastHoverClose = -Infinity;
/** Focus shows a tooltip only when it came from the keyboard, never from a click or a tap. */
let keyboardModality = true;
if (typeof document !== 'undefined') {
  document.addEventListener('keydown', () => { keyboardModality = true; }, true);
  document.addEventListener('pointerdown', () => { keyboardModality = false; }, true);
}

/** A tooltip or breakdown is shown: its Escape closes it before anything else. */
export const tipOpen = () => openTip !== null;

/**
 * Other floating explanations (the unlock hovercard) share the one-at-a-time slot, so a hover elsewhere
 * closes them, Escape peels them before the modal under them, and a pinned one is not displaced by a hover.
 */
export type TipSlot = OpenTip;
export function claimTip(token: TipSlot) {
  if (openTip && openTip !== token) openTip.close();
  openTip = token;
}
export function releaseTip(token: TipSlot) { if (openTip === token) openTip = null; }
/** A pinned explanation elsewhere keeps the stage until it is closed. */
export const tipPinnedElsewhere = (token: TipSlot) => !!openTip && openTip !== token && openTip.mode() === 'pinned';
/** True when moving to this explanation should show it at once (another is open, or one just closed). */
export const tipWarm = (token: TipSlot) => (!!openTip && openTip !== token) || performance.now() - lastHoverClose < WARM_MS;
export const noteTipClosed = () => { lastHoverClose = performance.now(); };

/** Close whatever tooltip is open, e.g. when a modal opens over its trigger. */
export function closeTips() {
  openTip?.close();
  openTip = null;
}

const CONTROLS = 'button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex], [contenteditable="true"]';

/** The first breakdown table inside a tooltip's content, if any: such content is "rich". */
function findBreakdown(node: ReactNode): ReactElement | null {
  let found: ReactElement | null = null;
  Children.forEach(node, (child) => {
    if (found || !isValidElement(child)) return;
    const props = child.props as { className?: unknown; children?: ReactNode };
    if (child.type === BreakdownTable || child.type === YieldSourceTable || child.type === YieldsAfter || (typeof props.className === 'string' && /(^|\s)bd(\s|$)/.test(props.className))) {
      found = child;
      return;
    }
    found = findBreakdown(props.children);
  });
  return found;
}

/** A tooltip's plain-text lines show their amounts as number + icon (legatum-ui: yield amounts carry their icon). */
const withAmounts = (node: ReactNode): ReactNode => typeof node === 'string' ? <Amounts text={node} /> : node;

/** A one-line summary of a breakdown for the compact hover tooltip, e.g. "Production: 3 ⚙ per turn" (the amount renders as number + icon). */
function summarize(table: ReactElement | null): string | null {
  if (!table) return null;
  if (table.type === BreakdownTable) {
    const { b, title, unit } = table.props as Parameters<typeof BreakdownTable>[0];
    return `${title ?? 'Total'}: ${b.total}${unit ? ` ${unit}` : ''}`;
  }
  if (table.type === YieldSourceTable) {
    const { yields, k } = table.props as Parameters<typeof YieldSourceTable>[0];
    return `${YIELD_LABEL[k]}: ${yields.total[k]} ${YIELD_LABEL[k]} per turn`;
  }
  return null;
}

/**
 * After Esc closes an explanation, the trigger that still holds focus wears a focus frame (a click that pinned it
 * left it without one), until focus moves on or the pointer presses anywhere.
 */
function frameFocus(root: HTMLElement | null, bubble: HTMLElement | null) {
  const active = document.activeElement;
  if (!root || !(root.contains(active) || bubble?.contains(active))) return;
  // Focus inside the closed bubble goes back to the trigger, so a keyboard user is not dropped on the page.
  if (!root.contains(active) && root.tabIndex >= 0) root.focus({ preventScroll: true });
  root.setAttribute('data-esc-focus', '');
  const clear = () => {
    root.removeAttribute('data-esc-focus');
    root.removeEventListener('focusout', clear);
    document.removeEventListener('pointerdown', clear, true);
  };
  root.addEventListener('focusout', clear);
  document.addEventListener('pointerdown', clear, true);
}

/**
 * Explanations. Hover (after a short delay) or keyboard focus shows a compact tooltip that never
 * takes the pointer and never covers its trigger or the primary controls. Rich breakdowns open on
 * click (or the ⓘ next to a control) as a pinned popover with a Close button. Touch taps pin too.
 */
export function Tip({
  content,
  children,
  className,
  placement = 'bottom',
  as: As = 'span',
  summary,
  rich,
  onOpenChange,
}: {
  content: ReactNode;
  children: ReactNode;
  className?: string;
  placement?: Placement;
  as?: 'span' | 'div';
  /** The compact hover text for rich content (defaults to the breakdown's title and total). */
  summary?: ReactNode;
  /** Override the rich-content detection (content with a breakdown table opens on click). */
  rich?: boolean;
  /** Fires whenever the tooltip or breakdown opens or closes (e.g. to highlight related map tiles). */
  onOpenChange?: (open: boolean) => void;
}) {
  const id = useId();
  const coarse = useCoarsePointer();
  const ref = useRef<HTMLElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [rect, setRect] = useState<DOMRect | null>(null);
  const [mode, setMode] = useState<Mode>(null);
  const modeRef = useRef<Mode>(null);
  const [ownTrigger, setOwnTrigger] = useState(false);
  const [controlTip, setControlTip] = useState(false);
  const [host, setHost] = useState<Element | null>(null);
  const hasContent = content !== null && content !== undefined && content !== false;
  const table = findBreakdown(content);
  const isRich = hasContent && (rich ?? table !== null);
  const cancelShow = () => { if (timer.current) { clearTimeout(timer.current); timer.current = null; } };
  // This instance's handle on the page-wide "open tip" slot (stable identity, fresh closure).
  const token = useRef<OpenTip>({ close: () => {}, mode: () => null }).current;
  const open = (next: Exclude<Mode, null>) => {
    cancelShow();
    if (!hasContent || !ref.current) return;
    // Only one explanation at a time: a hover and a focus elsewhere must not stack two bubbles.
    if (openTip && openTip !== token) openTip.close();
    openTip = token;
    const was = modeRef.current;
    modeRef.current = next;
    setMode(next);
    setRect(ref.current.getBoundingClientRect());
    if (!was) onOpenChange?.(true);
  };
  const close = () => {
    cancelShow();
    const was = modeRef.current;
    modeRef.current = null;
    setMode(null);
    setRect(null);
    if (openTip === token) openTip = null;
    if (was) onOpenChange?.(false);
  };
  token.close = close;
  token.mode = () => modeRef.current;
  const hoverIn = () => {
    if (!hasContent || modeRef.current) return;
    // A pinned breakdown elsewhere keeps the stage until it is closed.
    if (openTip && openTip !== token && openTip.mode() === 'pinned') return;
    const warm = (openTip && openTip !== token) || performance.now() - lastHoverClose < WARM_MS;
    if (warm) open('hover');
    else {
      cancelShow();
      timer.current = setTimeout(() => open('hover'), SHOW_DELAY);
    }
  };
  const hoverOut = () => {
    cancelShow();
    if (modeRef.current !== 'hover') return;
    close();
    lastHoverClose = performance.now();
  };
  const togglePin = () => (modeRef.current === 'pinned' ? close() : open('pinned'));

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root || !hasContent) { setOwnTrigger(false); return; }
    const controls = [...root.querySelectorAll<HTMLElement>(CONTROLS)].filter((el) => !el.hasAttribute('data-tip-info'));
    setOwnTrigger(controls.length === 0);
    setControlTip(!!root.querySelector('button:not([data-tip-info]), a[href], input, select, textarea'));
    // A body portal would be inert and behind the top layer in a native modal.
    setHost(root.closest('dialog') ?? document.body);
    for (const el of controls) {
      const ids = new Set((el.getAttribute('aria-describedby') ?? '').split(' ').filter(Boolean));
      ids.add(id);
      el.setAttribute('aria-describedby', [...ids].join(' '));
    }
    return () => {
      for (const el of controls) {
        const ids = (el.getAttribute('aria-describedby') ?? '').split(' ').filter((value) => value && value !== id);
        if (ids.length) el.setAttribute('aria-describedby', ids.join(' '));
        else el.removeAttribute('aria-describedby');
      }
    };
  }, [children, hasContent, id]);

  useEffect(() => {
    if (!mode) return;
    const dismiss = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      close();
      frameFocus(ref.current, document.getElementById(id));
      // A tip left behind a modal (research, guide, game over) must not swallow the Escape that
      // closes that modal: it just goes away quietly.
      const modal = [...document.querySelectorAll('dialog')].find((d) => d.matches(':modal'));
      if (modal && !modal.contains(ref.current)) return;
      e.preventDefault();
      e.stopPropagation();
    };
    const outside = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node) && !document.getElementById(id)?.contains(e.target as Node)) close();
    };
    // Hover tooltips go away on any scroll; a pinned breakdown follows its trigger.
    const scrolled = (e: Event) => {
      if (document.getElementById(id)?.contains(e.target as Node)) return;
      if (modeRef.current === 'hover') close();
      else if (ref.current) setRect(ref.current.getBoundingClientRect());
    };
    const reposition = () => { if (ref.current) setRect(ref.current.getBoundingClientRect()); };
    window.addEventListener('resize', reposition);
    document.addEventListener('scroll', scrolled, true);
    document.addEventListener('keydown', dismiss, true);
    document.addEventListener('pointerdown', outside, true);
    return () => {
      window.removeEventListener('resize', reposition);
      document.removeEventListener('scroll', scrolled, true);
      document.removeEventListener('keydown', dismiss, true);
      document.removeEventListener('pointerdown', outside, true);
    };
  }, [mode]);
  useEffect(() => () => {
    if (timer.current) clearTimeout(timer.current);
    if (openTip === token) openTip = null;
    if (modeRef.current) onOpenChange?.(false);
  }, [token]);

  // Controls keep their click for the action: rich content opens from a separate ⓘ, as on touch.
  const infoButton = hasContent && controlTip && (coarse || isRich);
  const autoSummary = summary ?? summarize(table);
  const hoverBody = isRich ? (
    <>
      {autoSummary && <div className="tip-summary">{withAmounts(autoSummary)}</div>}
      <div className="tip-more">{controlTip ? 'ⓘ for the full breakdown' : 'Click for the full breakdown'}</div>
    </>
  ) : withAmounts(content);

  return (
    <As
      ref={ref as never}
      className={`tip-trigger ${className ?? ''}`}
      data-tip-static={ownTrigger && !controlTip ? 'true' : undefined}
      data-tip-control={controlTip ? 'true' : undefined}
      data-tip-rich={isRich && ownTrigger ? 'true' : undefined}
      tabIndex={ownTrigger ? 0 : undefined}
      role={ownTrigger ? 'group' : undefined}
      aria-describedby={ownTrigger ? id : undefined}
      onPointerEnter={(e) => { if (e.pointerType === 'mouse') hoverIn(); }}
      onPointerLeave={(e) => { if (e.pointerType === 'mouse') hoverOut(); }}
      // Pressing a control is not a request to read about it: the hover tooltip steps aside.
      onPointerDown={() => { cancelShow(); if (controlTip && modeRef.current === 'hover') close(); }}
      onClick={(e) => {
        if (!ref.current?.contains(e.target as Node) || controlTip) return;
        e.stopPropagation();
        togglePin();
      }}
      onKeyDown={(e) => {
        if (e.target === e.currentTarget && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); e.stopPropagation(); togglePin(); }
      }}
      onFocus={(e) => { if (keyboardModality && ref.current?.contains(e.target as Node) && !modeRef.current) open('hover'); }}
      onBlur={(e) => {
        if (e.currentTarget.contains(e.relatedTarget)) return;
        if (modeRef.current === 'hover') close();
      }}
    >
      {children}
      {infoButton && <button type="button" className="tip-info" data-tip-info aria-label="Show explanation"
        aria-controls={id} aria-expanded={mode === 'pinned'} onClick={(e) => { e.stopPropagation(); togglePin(); }}>ⓘ</button>}
      {host && hasContent && createPortal(
        <Bubble id={id} rect={rect} trigger={ref} placement={placement} mode={mode} onClose={close}>
          {mode === 'pinned' ? withAmounts(content) : hoverBody}
        </Bubble>, host,
      )}
    </As>
  );
}

type Box = { left: number; top: number; right: number; bottom: number };

const FALLBACKS: Record<Placement, Placement[]> = {
  bottom: ['bottom', 'top', 'right', 'left'],
  top: ['top', 'bottom', 'right', 'left'],
  right: ['right', 'left', 'bottom', 'top'],
  left: ['left', 'right', 'bottom', 'top'],
};

const overlapArea = (a: Box, b: Box) =>
  Math.max(0, Math.min(a.right, b.right) - Math.max(a.left, b.left)) * Math.max(0, Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top));

/**
 * Where a bubble of `size` goes next to `trigger`, kept 8px inside the viewport. It never covers
 * its own trigger (the viewport clamp flips it to another side instead), and among the sides that
 * don't, it takes the one covering the least of `avoid`: the panel it explains and the primary
 * controls (unit actions, End turn). Ties keep the preferred order.
 */
export function placeBubble(
  trigger: Box,
  size: { width: number; height: number },
  placement: Placement,
  viewport: { width: number; height: number; top?: number },
  avoid: Box[] = [],
): { left: number; top: number } {
  const gap = 8;
  // The usable area starts below the top bar (unless the trigger is in it).
  const minTop = (viewport.top ?? 0) + gap;
  const at = (side: Placement) => {
    let left = trigger.left + (trigger.right - trigger.left) / 2 - size.width / 2;
    let top = side === 'top' ? trigger.top - size.height - gap : trigger.bottom + gap;
    if (side === 'left' || side === 'right') {
      left = side === 'left' ? trigger.left - size.width - gap : trigger.right + gap;
      top = trigger.top + (trigger.bottom - trigger.top) / 2 - size.height / 2;
    }
    left = Math.max(gap, Math.min(viewport.width - size.width - gap, left));
    top = Math.max(minTop, Math.min(viewport.height - size.height - gap, top));
    return { left, top };
  };
  const box = ({ left, top }: { left: number; top: number }) => ({ left, top, right: left + size.width, bottom: top + size.height });
  const clamp = ({ left, top }: { left: number; top: number }) => ({
    left: Math.max(gap, Math.min(viewport.width - size.width - gap, left)),
    top: Math.max(minTop, Math.min(viewport.height - size.height - gap, top)),
  });
  // Beside a box that contains the trigger (its panel), on the given side: clear of the whole panel.
  const escape = (side: Placement, a: Box) => {
    const pos = at(side);
    if (side === 'right') return clamp({ ...pos, left: a.right + gap });
    if (side === 'left') return clamp({ ...pos, left: a.left - size.width - gap });
    if (side === 'top') return clamp({ ...pos, top: a.top - size.height - gap });
    return clamp({ ...pos, top: a.bottom + gap });
  };
  const holders = avoid.filter((a) => a.left <= trigger.left && a.right >= trigger.right && a.top <= trigger.top && a.bottom >= trigger.bottom);
  const candidates = FALLBACKS[placement].flatMap((side) => [at(side), ...holders.map((a) => escape(side, a))]);
  let best: { pos: { left: number; top: number }; score: number } | null = null;
  for (const pos of candidates) {
    const b = box(pos);
    const score = (overlapArea(b, trigger) > 0 ? 1e12 : 0) + avoid.reduce((sum, a) => sum + overlapArea(b, a), 0);
    if (!best || score < best.score) best = { pos, score };
    if (score === 0) break;
  }
  return best!.pos;
}

/**
 * What a tooltip should stay clear of: its own panel (when it isn't most of the screen), the
 * primary controls (unit actions, the This Turn panel with End turn) and the top bar's readouts.
 */
const PRIMARY = '.unit-panel .actions, .seatbar, .topbar, .research-detail .primary, [data-tip-avoid]';
function avoidBoxes(trigger: HTMLElement | null): Box[] {
  if (!trigger) return [];
  const screen = window.innerWidth * window.innerHeight;
  const boxes: Box[] = [];
  const panel = trigger.closest('.panel');
  const modal = trigger.closest('dialog');
  if (panel) {
    const r = panel.getBoundingClientRect();
    if (r.width * r.height < 0.45 * screen) boxes.push(r);
  }
  for (const el of document.querySelectorAll(PRIMARY)) {
    // Behind a modal, the page's controls don't matter.
    if (modal && !modal.contains(el)) continue;
    const r = el.getBoundingClientRect();
    if (r.width && r.height) boxes.push(r);
  }
  return boxes;
}

function Bubble({ id, rect, trigger, placement, children, mode, onClose }: {
  id: string;
  rect: DOMRect | null;
  trigger: { current: HTMLElement | null };
  placement: Placement;
  children: ReactNode;
  mode: Mode;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
  const pinned = mode === 'pinned';
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el || !rect) { setPos(null); return; }
    const b = el.getBoundingClientRect();
    const bar = document.querySelector('.topbar');
    const top = bar && !bar.contains(trigger.current) ? bar.getBoundingClientRect().bottom : 0;
    setPos(placeBubble(rect, b, placement, { width: window.innerWidth, height: window.innerHeight, top }, avoidBoxes(trigger.current)));
  }, [rect, placement, mode, children]);
  return (
    <div
      ref={ref}
      id={id}
      role={pinned ? 'dialog' : 'tooltip'}
      aria-label={pinned ? 'Breakdown' : undefined}
      hidden={!rect}
      className={`tip-bubble ${pinned ? 'tip-pinned' : 'tip-hover'}`}
      style={{ left: pos?.left ?? -9999, top: pos?.top ?? -9999, opacity: pos ? 1 : 0 }}
    >
      {pinned && <button type="button" className="tip-close" aria-label="Close breakdown" onClick={(e) => { e.stopPropagation(); onClose(); }}><Icon name="ui-cross" size={11} /></button>}
      {children}
    </div>
  );
}
