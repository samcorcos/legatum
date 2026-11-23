import { useGame } from './store';
import { useGuide } from './ui/guideState';
import { tipOpen } from './ui/Tip';

const PAN_KEYS = ['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'];
/** Keys a plain button has no native use for: panning, and Escape to hand focus back to the map. */
const CONTROL_PASS_KEYS = [...PAN_KEYS, 'escape'];

/** Where a key has its own meaning: typing, choosing from a list, or a widget that uses arrows. */
function ownsKeys(el: HTMLElement): boolean {
  return !!el.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"], [role="listbox"], [role="menu"], [role="radiogroup"], [role="slider"], [role="tablist"]');
}

/**
 * Widgets that keep focus after a mouse click: text entry and lists. A clicked radio or tab (the
 * map lenses, report filters) is a one-shot choice; its arrow keys matter only to keyboard users.
 */
function keepsPointerFocus(el: HTMLElement): boolean {
  return !!el.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"], [role="listbox"], [role="menu"], [role="slider"]');
}

/** No modal surface is up (research, guide, game menu, game over) and a live game is on screen. */
function gameplayOpen(doc: Document): boolean {
  const s = useGame.getState();
  return !!s.views && !s.busy && !s.views[s.seat]?.finished && !s.techOpen &&
    !useGuide.getState().open && !doc.querySelector('dialog[open], [data-game-overlay]');
}

/**
 * Who owns a key. Commands (Enter, Space, hotkeys) belong to the map and the unfocused page only,
 * so a focused control keeps its native Enter and Space. Panning keys (arrows, WASD) have no
 * meaning on a plain button or link, so they pan from there too, and Escape there returns focus to
 * the map; only text entry and widgets that use arrows (listboxes, menus, radio groups) keep them.
 * The non-modal turn report blocks nothing but the first Return or Escape, which close it
 * (`handleOverlayKey`).
 */
export function gameplayFocusAllowed(target: EventTarget | null, key?: string): boolean {
  const el = target as HTMLElement | null;
  const doc = el?.ownerDocument;
  if (!doc || !gameplayOpen(doc)) return false;
  if (el === doc.body || el?.matches?.('[data-game-map]')) return true;
  return !!key && CONTROL_PASS_KEYS.includes(key.toLowerCase()) && !ownsKeys(el!);
}

/** Shared by game commands and arrow/WASD panning, including keys handled by another widget. */
export function gameplayKeyAllowed(e: KeyboardEvent): boolean {
  return !e.defaultPrevented && !e.isComposing && !e.metaKey && !e.ctrlKey && !e.altKey &&
    gameplayFocusAllowed(e.target, e.key);
}

/**
 * Surfaces that cover the map without asking for a decision, topmost first (G15). Each carries
 * `data-dismissible="<name>"` while it is shown. Decisions (the end-turn review, Declare War, game
 * over) and the game menu have no such mark and keep their own keys.
 */
const DISMISSIBLE: readonly [name: string, close: () => void][] = [
  ['guide', () => useGuide.getState().close()],
  ['research', () => useGame.getState().set({ techOpen: false })],
  ['cities', () => useGame.getState().set({ citiesOpen: false })],
  ['diplomacy', () => useGame.getState().set({ diplomacyOpen: false })],
  ['victory', () => useGame.getState().set({ vpExpanded: false })],
  ['report', () => useGame.getState().set({ reportOpen: false })],
];

/** The dismissible surface on top, or null when there is none or a decision modal is up. */
export function topDismissible(doc: Document): string | null {
  if (doc.querySelector('dialog[open]:not([data-dismissible]), [data-game-overlay]:not([data-dismissible])')) return null;
  return DISMISSIBLE.find(([name]) => doc.querySelector(`dialog[open][data-dismissible="${name}"], :not(dialog)[data-dismissible="${name}"]`))?.[0] ?? null;
}

/** Focus last moved with Tab: Return then presses the focused control, as keyboard users expect. */
let tabbed = false;
export function notePointerDown(): void { tabbed = false; }

/**
 * G15: while something covers the map, the first Return or Escape closes the top surface and is
 * consumed, so the map shows; the next Return is the turn's primary action again (next decision,
 * then End Turn). Installed on window in the capture phase, ahead of every panel's own key
 * handling. Escape defers to an open tooltip or a playing celebration, which each peel first;
 * Return defers to text entry and to a control the player reached with Tab.
 */
export function handleOverlayKey(e: KeyboardEvent): void {
  if (e.key === 'Tab') { tabbed = true; return; }
  if ((e.key !== 'Enter' && e.key !== 'Escape') || e.defaultPrevented || e.isComposing || e.repeat || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return;
  const target = e.target as HTMLElement | null;
  const doc = target?.ownerDocument ?? document;
  const name = topDismissible(doc);
  if (!name) return;
  // A pinned tip inside the modal peels first. Hover hints and celebrations cannot hold it open.
  const modal = doc.querySelector('dialog[open]');
  const tipAbove = modal ? !!modal.querySelector('.tip-bubble.tip-pinned, .unlock-card.pinned') : tipOpen();
  if (e.key === 'Escape' && (tipAbove || (!modal && doc.querySelector('.moment-lane .moment-card, .moment-lane .era-takeover')))) return;
  // A moment card (tech unlock, wonder, reward) or the era-up takeover owns Return (its primary action) until it ends.
  if (e.key === 'Enter' && doc.querySelector('.moment-lane .moment-card, .moment-lane .era-takeover')) return;
  if (e.key === 'Enter' && target && target !== doc.body) {
    // A focused target that acts on Return itself (a tech card: Return researches it) keeps it.
    if (ownsKeys(target) || target.closest('[data-enter-activates]')) return;
    if (tabbed && target.closest('button, a[href], summary, [role="button"], [role="option"], [role="tab"], [role="checkbox"], [role="switch"]')) return;
  }
  e.preventDefault();
  e.stopPropagation();
  DISMISSIBLE.find(([n]) => n === name)![1]();
  // Return must hand focus to the map even if the menu trigger still has it;
  // otherwise the next Return reopens that control instead of advancing the turn.
  requestAnimationFrame(() => {
    const active = doc.activeElement;
    if (e.key === 'Enter' || !active || active === doc.body || !active.isConnected) doc.querySelector<HTMLElement>('[data-game-map]')?.focus({ preventScroll: true });
  });
}

/**
 * Mouse and touch players keep working shortcuts: a control activated by the pointer hands focus
 * back to the map, so arrows pan and Return advances decisions or ends the turn instead of re-pressing that control.
 * Keyboard activation (click.detail === 0) keeps focus where it is. Text entry, and controls inside
 * modal surfaces or popovers, manage their own focus.
 */
export function returnFocusAfterPointer(e: MouseEvent): void {
  if (e.detail === 0) return;
  const control = (e.target as HTMLElement | null)?.closest?.<HTMLElement>('button, a[href], summary, [role="button"], [role="option"], [role="tab"], label');
  if (!control || keepsPointerFocus(control) || control.closest('dialog, [data-game-overlay], .faction-picker')) return;
  const map = control.ownerDocument.querySelector<HTMLElement>('[data-game-map]');
  if (!map) return;
  // Run before the control's own handler: a dialog it opens then restores focus to the map.
  if (control.ownerDocument.activeElement === control || control.contains(control.ownerDocument.activeElement)) {
    map.focus({ preventScroll: true });
  }
}
