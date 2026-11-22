// @vitest-environment happy-dom
import { act, createElement, type ReactNode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { Tip } from './Tip';
import { FactionPicker } from './FactionPicker';

let host: HTMLElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.useFakeTimers();
  document.body.innerHTML = '';
  vi.spyOn(window, 'matchMedia').mockReturnValue(Object.assign(new EventTarget(), { matches: true }) as MediaQueryList);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });
const render = (children: ReactNode) => act(() => root.render(children));
const tap = (el: HTMLElement) => act(() => {
  el.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, pointerType: 'touch' }));
  el.dispatchEvent(new PointerEvent('pointerup', { bubbles: true, pointerType: 'touch' }));
  el.click();
});
const open = () => document.querySelector<HTMLElement>('[aria-label="Breakdown"]')!;

it('pins a static breakdown through pointer leave and blur, then dismisses with Close or outside touch', () => {
  render(createElement(Tip, { children: '12 Science', content: '8 from cities + 4 from buildings' }));
  const trigger = host.firstElementChild as HTMLElement;
  tap(trigger);
  act(() => {
    trigger.dispatchEvent(new MouseEvent('mouseout', { bubbles: true, relatedTarget: document.body }));
    trigger.blur();
    vi.advanceTimersByTime(500);
  });
  expect(open()?.hidden).toBe(false);
  tap(open().querySelector('button')!);
  expect(open()).toBeNull(); // Portal click must not bubble back into the trigger and reopen it.
  tap(trigger);
  tap(document.body);
  expect(open()).toBeNull();
});

it.each([false, true])('inspects an action separately without executing it (disabled: %s)', (disabled) => {
  const action = vi.fn();
  render(createElement(Tip, { content: 'Upgrade costs 80 Gold', children: createElement('button', { disabled, onClick: action }, 'Upgrade') }));
  const info = host.querySelector<HTMLButtonElement>('[data-tip-info]')!;
  expect(info).not.toBeNull();
  tap(info);
  expect(open()?.textContent).toContain('80 Gold');
  expect(action).not.toHaveBeenCalled();
  tap(open().querySelector('button')!);
  if (!disabled) {
    tap(host.querySelector('button')!);
    expect(action).toHaveBeenCalledOnce();
  }
});

it('touch: one tap on a faction picks it and shows its sheet, and the dialog stays open until Close', () => {
  const change = vi.fn();
  render(createElement(FactionPicker, { value: 'India', onChange: change, label: 'Faction' }));
  tap(host.querySelector('.faction-current')!);
  const tile = (name: string) => [...host.querySelectorAll<HTMLElement>('.fs-tile')].find((o) => o.textContent?.includes(name))!;
  tap(tile('Byzantium'));
  expect(change).toHaveBeenCalledExactlyOnceWith('Byzantium');
  expect(host.querySelector<HTMLDialogElement>('[data-testid="faction-stage"]')!.open).toBe(true);
  expect(host.querySelector('.fs-play')).toBeNull();
  // Closing is a separate, explicit step.
  tap(host.querySelector<HTMLElement>('.fs-close')!);
  expect(host.querySelector<HTMLDialogElement>('[data-testid="faction-stage"]')!.open).toBe(false);
  expect(change).toHaveBeenCalledOnce();
});
