// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { CRASH_AUTOSAVE_KEY, CrashCard, RootErrorBoundary, autosaveFileName, errorDetails, installGlobalErrorHandlers, isFatal, readAutosaveText, reportFatal, resetCrashStateForTests } from './CrashScreen';

let host: HTMLElement, root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  resetCrashStateForTests();
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function Boom(): never {
  throw new Error('panel exploded');
}

const button = (name: string) => [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith(name)) as HTMLButtonElement | undefined;
const save = (turn: number) => localStorage.setItem(CRASH_AUTOSAVE_KEY, JSON.stringify({ format: 'legatum-save', version: 1, turn, game: '{}' }));

it('a child that throws while drawing shows the crash card, with Reload as the one primary and focused', () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  save(12);
  act(() => root.render(<RootErrorBoundary><Boom /></RootErrorBoundary>));
  const card = host.querySelector('[data-testid="crash-screen"]')!;
  expect(card).not.toBeNull();
  expect(card.getAttribute('role')).toBe('alertdialog');
  expect(host.querySelector('h1')!.textContent).toBe('Something went wrong');
  expect(host.textContent).toContain('Your last finished turn is saved in this browser.');
  expect(host.querySelectorAll('.btn.primary')).toHaveLength(1);
  expect(button('Reload')!.classList.contains('primary')).toBe(true);
  expect(document.activeElement).toBe(button('Reload'));
  expect(button('Save to file')).toBeDefined();
  expect(button('Copy error details')).toBeDefined();
});

it('the healthy tree renders its children', () => {
  act(() => root.render(<RootErrorBoundary><p>map</p></RootErrorBoundary>));
  expect(host.textContent).toBe('map');
});

it('the buttons call the right handlers, and Return reloads', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  save(7);
  const reload = vi.fn();
  const saveFile = vi.fn(async () => true);
  const copy = vi.fn(async () => true);
  act(() => root.render(<RootErrorBoundary handlers={{ reload, save: saveFile, copy }}><Boom /></RootErrorBoundary>));

  act(() => button('Reload')!.click());
  expect(reload).toHaveBeenCalledTimes(1);

  await act(async () => button('Save to file')!.click());
  expect(saveFile).toHaveBeenCalledTimes(1);
  expect(host.textContent).toContain('Saved. Load it later with Load Game on the title screen.');

  await act(async () => button('Copy error details')!.click());
  expect(copy).toHaveBeenCalledTimes(1);
  const details = copy.mock.calls[0][0 as never] as unknown as string;
  expect(details).toContain('Error: Error: panel exploded');
  expect(details).toContain('Build: ');
  expect(details).toContain(`Browser: ${navigator.userAgent}`);
  expect(host.textContent).toContain('Copied.');

  // Return on the card itself (not on another button) is Reload.
  const card = host.querySelector<HTMLElement>('[data-testid="crash-screen"]')!;
  act(() => { card.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })); });
  expect(reload).toHaveBeenCalledTimes(2);
});

it('Tab stays inside the card', () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  save(3);
  act(() => root.render(<RootErrorBoundary handlers={{ reload: vi.fn() }}><Boom /></RootErrorBoundary>));
  const last = button('Copy error details')!;
  last.focus();
  act(() => { last.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true })); });
  expect(document.activeElement).toBe(button('Reload'));
  act(() => { button('Reload')!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true })); });
  expect(document.activeElement).toBe(last);
});

it('with no autosave there is no Save to file and the text does not promise a save', () => {
  act(() => root.render(<CrashCard error={new Error('x')} handlers={{ reload: vi.fn() }} />));
  expect(button('Save to file')).toBeUndefined();
  expect(host.textContent).toContain('The game hit an error. Reload to start again.');
  expect(host.textContent).not.toContain('saved in this browser');
});

it('a failed copy shows the details to copy by hand', async () => {
  const copy = vi.fn(async () => false);
  act(() => root.render(<CrashCard error={new Error('x')} handlers={{ reload: vi.fn(), copy }} />));
  await act(async () => button('Copy error details')!.click());
  expect(host.textContent).toContain('Could not copy. Select the text below and copy it.');
  expect(host.querySelector('textarea')!.value).toContain('Error: Error: x');
});

it('the engine card says the engine did not load', () => {
  act(() => root.render(<CrashCard error={new TypeError('Failed to fetch')} kind="engine" handlers={{ reload: vi.fn() }} />));
  expect(host.querySelector('h1')!.textContent).toBe('The game could not load');
  expect(host.textContent).toContain('Check your connection and reload.');
});

it('a fatal window error reaches the mounted boundary; a benign rejection is only logged', () => {
  const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
  vi.spyOn(console, 'error').mockImplementation(() => {});
  installGlobalErrorHandlers(window);
  act(() => root.render(<RootErrorBoundary handlers={{ reload: vi.fn() }}><p>map</p></RootErrorBoundary>));

  const benign = new Event('unhandledrejection') as Event & { reason: unknown };
  benign.reason = new Error('archive upload failed');
  act(() => { window.dispatchEvent(benign); });
  expect(host.textContent).toBe('map');
  expect(warn).toHaveBeenCalled();

  act(() => reportFatal(new WebAssembly.RuntimeError('unreachable')));
  expect(host.querySelector('[data-testid="crash-screen"]')).not.toBeNull();
});

it('isFatal is an allowlist: engine traps and failed downloads of app code only', () => {
  expect(isFatal(new WebAssembly.RuntimeError('unreachable'))).toBe(true);
  expect(isFatal(new TypeError('Failed to fetch dynamically imported module: /assets/x.js'))).toBe(true);
  expect(isFatal(new Error('Importing a module script failed.'))).toBe(true);
  expect(isFatal(new Error('network down'))).toBe(false);
  expect(isFatal(new TypeError("Cannot read properties of undefined (reading 'x')"))).toBe(false);
  expect(isFatal(undefined)).toBe(false);
});

it('reads the autosave straight from storage and names the file like Save to file', () => {
  expect(readAutosaveText()).toBeNull();
  save(41);
  expect(readAutosaveText()!.turn).toBe(41);
  expect(autosaveFileName(41)).toBe('legatum-turn-41.legatum.json');
  localStorage.setItem(CRASH_AUTOSAVE_KEY, 'not json');
  expect(readAutosaveText()).toEqual({ text: 'not json', turn: null });
  expect(autosaveFileName(null)).toBe('legatum-autosave.legatum.json');
});

it('the details carry the error, build and browser, and nothing from the game', () => {
  const text = errorDetails(new Error('boom'), 'crash', new Date('2026-11-22T10:00:00Z'));
  expect(text).toMatch(/^Legatum error report \(app error\)\nBuild: .+\nTime: 2026-11-22T10:00:00.000Z\nBrowser: /);
  expect(text).toContain('Stack:');
});
