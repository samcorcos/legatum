// @vitest-environment happy-dom
import { afterEach, expect, it } from 'vitest';
import { installPressFeedback } from './pressFeedback';

let cleanup: (() => void) | undefined;
afterEach(() => { cleanup?.(); document.body.replaceChildren(); });
const pointer = (el: HTMLElement | Window, type: string, id = 1, x = 10) => el.dispatchEvent(new PointerEvent(type, { bubbles: true, pointerType: 'touch', pointerId: id, button: 0, clientX: x, clientY: 10 }));

it('acknowledges new controls immediately and clears on release, cancel, scroll and teardown', () => {
  cleanup = installPressFeedback();
  const button = document.body.appendChild(document.createElement('button'));
  const child = button.appendChild(document.createElement('span'));
  for (const stop of ['pointerup', 'pointercancel']) {
    pointer(child, 'pointerdown'); expect(button.dataset.touchPressed).toBe('true');
    pointer(window, stop); expect(button.dataset.touchPressed).toBeUndefined();
  }
  pointer(button, 'pointerdown');
  window.dispatchEvent(new Event('scroll'));
  expect(button.dataset.touchPressed).toBeUndefined();
  pointer(button, 'pointerdown'); cleanup();
  expect(button.dataset.touchPressed).toBeUndefined();
  pointer(button, 'pointerdown'); expect(button.dataset.touchPressed).toBeUndefined();
});

it('a pan or window blur cancels feedback; disabled controls and mouse clicks do not get touch feedback', () => {
  cleanup = installPressFeedback();
  const button = document.body.appendChild(document.createElement('button'));
  pointer(button, 'pointerdown'); pointer(window, 'pointermove', 1, 19);
  expect(button.dataset.touchPressed).toBeUndefined();
  pointer(button, 'pointerdown'); window.dispatchEvent(new Event('blur'));
  expect(button.dataset.touchPressed).toBeUndefined();
  button.disabled = true; pointer(button, 'pointerdown');
  expect(button.dataset.touchPressed).toBeUndefined();
  button.disabled = false;
  button.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, pointerType: 'mouse', button: 0 }));
  expect(button.dataset.touchPressed).toBeUndefined();
});

it('one lifted finger does not clear a second finger pressing the same control', () => {
  cleanup = installPressFeedback();
  const button = document.body.appendChild(document.createElement('button'));
  pointer(button, 'pointerdown', 1); pointer(button, 'pointerdown', 2);
  pointer(window, 'pointerup', 1); expect(button.dataset.touchPressed).toBe('true');
  pointer(window, 'pointercancel', 2); expect(button.dataset.touchPressed).toBeUndefined();
});
