// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { SHOW_DELAY, Tip } from './Tip';

let host: HTMLElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.useFakeTimers();
  document.body.innerHTML = '';
  // A mouse, not a touch screen.
  vi.spyOn(window, 'matchMedia').mockReturnValue(Object.assign(new EventTarget(), { matches: false }) as MediaQueryList);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

const shown = () => [...document.querySelectorAll<HTMLElement>('.tip-bubble')].filter((b) => !b.hidden);
function hover() {
  act(() => root.render(createElement('div', { className: 'list' }, createElement(Tip, { content: 'Grows in 3 turns', children: 'Food' }))));
  const trigger = host.querySelector<HTMLElement>('.tip-trigger')!;
  act(() => {
    trigger.dispatchEvent(new PointerEvent('pointerover', { bubbles: true, pointerType: 'mouse', relatedTarget: document.body }));
    vi.advanceTimersByTime(SHOW_DELAY + 10);
  });
  expect(shown().map((b) => b.textContent)).toEqual(['Grows in 3 turns']);
  return trigger;
}

it('a hover tip closes when the list under it scrolls the trigger away, and does not come back', () => {
  const trigger = hover();
  // The browser sends the pointer's leave and the list's scroll in the same frame, before React re-renders.
  act(() => {
    trigger.dispatchEvent(new PointerEvent('pointerout', { bubbles: true, pointerType: 'mouse', relatedTarget: document.body }));
    host.querySelector('.list')!.dispatchEvent(new Event('scroll'));
  });
  expect(shown()).toEqual([]);
});

it('a hover tip closes on a scroll even while the pointer stays put', () => {
  hover();
  act(() => { host.querySelector('.list')!.dispatchEvent(new Event('scroll')); });
  expect(shown()).toEqual([]);
});
