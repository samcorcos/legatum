// @vitest-environment happy-dom
import { act } from 'react';
import { IDBFactory } from 'fake-indexeddb';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { AUTOSAVE_DB } from '../platform/autosaveStore';
import { CRASH_AUTOSAVE_KEY, CrashCard, RootErrorBoundary, autosaveFileName, errorDetails, installGlobalErrorHandlers, isFatal, readAutosaveText, reportFatal, resetCrashStateForTests } from './CrashScreen';

let host: HTMLElement, root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  localStorage.clear();
  sessionStorage.clear();
  vi.stubGlobal('indexedDB', new IDBFactory());
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

const show = async (el: import('react').ReactElement) => { await act(async () => root.render(el)); await act(async () => { await new Promise((r) => setTimeout(r, 500)); }); };
const button = (name: string) => [...host.querySelectorAll('button')].find((b) => b.textContent?.startsWith(name)) as HTMLButtonElement | undefined;
const save = (turn: number) => localStorage.setItem(CRASH_AUTOSAVE_KEY, JSON.stringify({ format: 'legatum-save', version: 1, turn, game: '{}' }));

it('a child that throws while drawing shows the crash card, with Reload as the one primary and focused', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  save(12);
  await show(<RootErrorBoundary><Boom /></RootErrorBoundary>);
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

it('the healthy tree renders its children', async () => {
  await show(<RootErrorBoundary><p>map</p></RootErrorBoundary>);
  expect(host.textContent).toBe('map');
});

it('the buttons call the right handlers, and Return reloads', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  save(7);
  const reload = vi.fn();
  const saveFile = vi.fn(async () => true);
  const copy = vi.fn(async () => true);
  await show(<RootErrorBoundary handlers={{ reload, save: saveFile, copy }}><Boom /></RootErrorBoundary>);

  act(() => button('Reload')!.click());
  expect(reload).toHaveBeenCalledTimes(1);

  await act(async () => { button('Save to file')!.click(); await new Promise((r) => setTimeout(r, 50)); });
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

it('Tab stays inside the card', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  save(3);
  await show(<RootErrorBoundary handlers={{ reload: vi.fn() }}><Boom /></RootErrorBoundary>);
  const last = button('Copy error details')!;
  last.focus();
  act(() => { last.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true })); });
  expect(document.activeElement).toBe(button('Reload'));
  act(() => { button('Reload')!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true })); });
  expect(document.activeElement).toBe(last);
});

it('with no autosave there is no Save to file and the text does not promise a save', async () => {
  await show(<CrashCard error={new Error('x')} handlers={{ reload: vi.fn() }} />);
  expect(button('Save to file')).toBeUndefined();
  expect(host.textContent).toContain('The game hit an error. Reload to start again.');
  expect(host.textContent).not.toContain('saved in this browser');
});

it('a failed copy shows the details to copy by hand', async () => {
  const copy = vi.fn(async () => false);
  await show(<CrashCard error={new Error('x')} handlers={{ reload: vi.fn(), copy }} />);
  await act(async () => button('Copy error details')!.click());
  expect(host.textContent).toContain('Could not copy. Select the text below and copy it.');
  expect(document.activeElement).toBe(host.querySelector('textarea'));
  expect(host.querySelector('textarea')!.value).toContain('Error: Error: x');
});

it('the engine card says the engine did not load', async () => {
  await show(<CrashCard error={new TypeError('Failed to fetch')} kind="engine" handlers={{ reload: vi.fn() }} />);
  expect(host.querySelector('h1')!.textContent).toBe('The game could not load');
  expect(host.textContent).toContain('Check your connection, then reload the page.');
});

it('a fatal window error reaches the mounted boundary; a benign rejection is only logged', async () => {
  const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
  vi.spyOn(console, 'error').mockImplementation(() => {});
  installGlobalErrorHandlers(window);
  await show(<RootErrorBoundary handlers={{ reload: vi.fn() }}><p>map</p></RootErrorBoundary>);

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

it('reads the autosave straight from storage and names the file like Save to file', async () => {
  expect(await readAutosaveText()).toBeNull();
  save(41);
  expect((await readAutosaveText())!.turn).toBe(41);
  expect(autosaveFileName(41)).toBe('legatum-turn-41.legatum.json');
  localStorage.setItem(CRASH_AUTOSAVE_KEY, 'not json');
  expect(await readAutosaveText()).toEqual({ text: 'not json', turn: null });
  expect(autosaveFileName(null)).toBe('legatum-autosave.legatum.json');
});

const put = async (records: { id: string; turn: number; savedAt: string; text: string }[]) => {
  const db = await new Promise<IDBDatabase>((resolve, reject) => {
    const open = indexedDB.open(AUTOSAVE_DB, 1);
    open.onupgradeneeded = () => { open.result.createObjectStore('saves', { keyPath: 'id' }); open.result.createObjectStore('orders', { keyPath: 'key' }); };
    open.onsuccess = () => resolve(open.result);
    open.onerror = () => reject(open.error);
  });
  await new Promise<void>((resolve) => {
    const tx = db.transaction('saves', 'readwrite');
    for (const r of records) tx.objectStore('saves').put({ ...r, campaign: 'c', label: 'Ann' });
    tx.oncomplete = () => { db.close(); resolve(); };
  });
};
const rec = (turn: number, minute: number) => ({ id: `c:${turn}`, turn, savedAt: `2026-11-22T10:0${minute}:00Z`, text: JSON.stringify({ turn }) });

it('reads the newest autosave from the database, never creating it, and keeps the localStorage fallback', async () => {
  expect(await readAutosaveText()).toBeNull();
  expect(await indexedDB.databases()).toEqual([]);
  await put([rec(7, 1), rec(8, 2), rec(6, 0)]);
  expect((await readAutosaveText())!.turn).toBe(8);
  // An older build's slot that is newer than the database's records wins; an older one does not.
  localStorage.setItem(CRASH_AUTOSAVE_KEY, JSON.stringify({ turn: 9, savedAt: '2026-11-22T10:05:00Z' }));
  expect((await readAutosaveText())!.turn).toBe(9);
  localStorage.setItem(CRASH_AUTOSAVE_KEY, JSON.stringify({ turn: 2, savedAt: '2026-11-22T09:00:00Z' }));
  expect((await readAutosaveText())!.turn).toBe(8);
});

it('offers the previous turn only the second time the card shows for the same turn, and hides the newest slot', async () => {
  vi.spyOn(console, 'error').mockImplementation(() => {});
  await put([rec(7, 1), rec(8, 2)]);
  const again = async () => {
    const reload = vi.fn();
    act(() => root.unmount());
    root = createRoot(host);
    await show(<CrashCard error={new Error('x')} handlers={{ reload }} />);
    return reload;
  };
  await again();
  expect(button('Save to file')).toBeDefined();
  expect(button('Reload on the previous turn')).toBeUndefined();
  const reload = await again();
  expect(button('Reload on the previous turn')).toBeDefined();
  expect(host.querySelector('#crash-text')!.textContent).toBe('The game hit the same error again. Choose Reload on the previous turn, then Continue, to go back one turn.');
  expect(host.querySelectorAll('.btn.primary')).toHaveLength(1);
  expect(button('Reload')!.classList.contains('primary')).toBe(true);
  await act(async () => { button('Reload on the previous turn')!.click(); await new Promise((r) => setTimeout(r, 500)); });
  expect(reload).not.toHaveBeenCalled(); // the default handler reloads the page itself
  expect(JSON.parse(localStorage.getItem('legatum.autosave.skipped.v1')!)).toEqual(['c:8']);
  expect((await readAutosaveText())!.turn).toBe(7);
});

it('does not offer the previous turn for a different turn, with only one slot, or on the engine card', async () => {
  await put([rec(8, 2)]);
  sessionStorage.setItem('legatum.crash.v1', JSON.stringify({ turn: 8, count: 1 }));
  await show(<CrashCard error={new Error('x')} handlers={{ reload: vi.fn() }} />);
  expect(button('Reload on the previous turn')).toBeUndefined(); // second time, but nothing earlier to load
  act(() => root.unmount());
  root = createRoot(host);
  await put([rec(7, 1)]);
  sessionStorage.setItem('legatum.crash.v1', JSON.stringify({ turn: 3, count: 5 }));
  await show(<CrashCard error={new Error('x')} handlers={{ reload: vi.fn() }} />);
  expect(button('Reload on the previous turn')).toBeUndefined(); // another turn: the count starts again
  act(() => root.unmount());
  root = createRoot(host);
  sessionStorage.setItem('legatum.crash.v1', JSON.stringify({ turn: 8, count: 5 }));
  await show(<CrashCard error={new Error('x')} kind="engine" handlers={{ reload: vi.fn() }} />);
  expect(button('Reload on the previous turn')).toBeUndefined();
});

it('the details carry the error, build and browser, and nothing from the game', () => {
  const text = errorDetails(new Error('boom'), 'crash', new Date('2026-11-22T10:00:00Z'));
  expect(text).toMatch(/^Legatum error report \(app error\)\nError: Error: boom\n\nBuild: .+\nTime: 2026-11-22T10:00:00.000Z\nBrowser: /);
  expect(text).toContain('Stack:');
});
