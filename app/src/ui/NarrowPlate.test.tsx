// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { NARROW_PLATE_KEY, NARROW_QUERY, NarrowGate, wantsNarrowPlate } from './NarrowPlate';

let host: HTMLElement, root: Root;
const narrow = (matches: boolean) => vi.spyOn(window, 'matchMedia').mockImplementation((query: string) => ({ matches: matches && query === NARROW_QUERY, media: query } as MediaQueryList));

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  // happy-dom reports itself as automation, which the gate skips; a real phone browser does not.
  vi.spyOn(navigator, 'webdriver', 'get').mockReturnValue(false);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it('a phone-sized touch screen sees the plate first; Continue anyway opens the game and is remembered', () => {
  narrow(true);
  act(() => root.render(<NarrowGate><p>game</p></NarrowGate>));
  expect(host.textContent).toContain('The map and panels are cramped on a phone. Open Legatum on a laptop or desktop for the full map.');
  expect(host.textContent).not.toContain('game');
  const go = host.querySelector<HTMLButtonElement>('.btn.primary')!;
  expect(go.textContent).toBe('Continue anyway');
  expect(document.activeElement).toBe(go);
  act(() => go.click());
  expect(host.textContent).toBe('game');
  expect(localStorage.getItem(NARROW_PLATE_KEY)).toBe('1');
  expect(wantsNarrowPlate()).toBe(false);
});

it('a laptop window, or a remembered choice, goes straight to the game', () => {
  narrow(false);
  act(() => root.render(<NarrowGate><p>game</p></NarrowGate>));
  expect(host.textContent).toBe('game');
  vi.mocked(window.matchMedia).mockRestore();
  narrow(true);
  localStorage.setItem(NARROW_PLATE_KEY, '1');
  expect(wantsNarrowPlate()).toBe(false);
});

it('blocked storage still lets the player continue', () => {
  narrow(true);
  vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('blocked'); });
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('blocked'); });
  act(() => root.render(<NarrowGate><p>game</p></NarrowGate>));
  act(() => host.querySelector<HTMLButtonElement>('.btn.primary')!.click());
  expect(host.textContent).toBe('game');
});
