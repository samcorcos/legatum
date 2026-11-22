// @vitest-environment happy-dom
import { IDBFactory } from 'fake-indexeddb';
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { EarlierTurnDialog } from './EarlierTurnDialog';
import { resetAutosaveStoreForTests, writeAutosave } from '../platform/autosaveStore';

let host: HTMLDivElement;
let root: Root;
const items = new Map<string, string>();
beforeEach(() => {
  resetAutosaveStoreForTests();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('indexedDB', new IDBFactory());
  vi.stubGlobal('localStorage', { getItem: (k: string) => items.get(k) ?? null, setItem: (k: string, v: string) => items.set(k, v), removeItem: (k: string) => items.delete(k) });
  HTMLDialogElement.prototype.showModal ??= function showModal(this: HTMLDialogElement) { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close ??= function close(this: HTMLDialogElement) { this.removeAttribute('open'); };
  useGame.setState(useGame.getInitialState(), true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.unstubAllGlobals();
});

const save = (turn: number) => JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', savedAt: new Date(Date.UTC(2026, 10, 22, 10, turn)).toISOString(), turn, names: ['Ann'], game: '{}' });

it('lists the earlier kept turns, newest first, and goes back to the one chosen', async () => {
  for (let turn = 1; turn <= 5; turn++) await writeAutosave({ campaign: 'g', turn, savedAt: new Date(Date.UTC(2026, 10, 22, 10, turn)).toISOString(), label: 'Ann', text: save(turn) });
  const rewindTo = vi.fn(async () => undefined);
  useGame.setState({ rewindTo });
  const onClose = vi.fn();
  await act(async () => root.render(<EarlierTurnDialog campaign="g" current={5} onClose={onClose} />));
  await vi.waitFor(() => expect(host.querySelectorAll('.earlier-turn-row')).toHaveLength(4));
  expect([...host.querySelectorAll('.earlier-turn-name')].map((n) => n.textContent)).toEqual(['Turn 4', 'Turn 3', 'Turn 2', 'Turn 1']);
  const primary = () => host.querySelector<HTMLButtonElement>('.btn.primary')!;
  expect(primary().textContent).toBe('Go back to turn 4');
  const forecast = () => host.querySelector('.earlier-turn-forecast')!.textContent;
  expect(forecast()).toBe('Turn 4 is played again. What you did in it is lost.');
  // Turn 2 is gzipped in storage: it still opens.
  await act(async () => host.querySelectorAll<HTMLInputElement>('.earlier-turn-row input')[2].click());
  expect(primary().textContent).toBe('Go back to turn 2');
  expect(forecast()).toBe('Turns 2 to 4 are played again. What you did in them is lost.');
  await act(async () => primary().click());
  await vi.waitFor(() => expect(rewindTo).toHaveBeenCalledOnce());
  expect(rewindTo.mock.calls[0]).toMatchObject(['g', { turn: 2 }]);
  expect(onClose).toHaveBeenCalled();
});

it('says so when no earlier turn is kept, with Cancel as the only action', async () => {
  await act(async () => root.render(<EarlierTurnDialog campaign="none" current={3} onClose={() => {}} />));
  await vi.waitFor(() => expect(host.textContent).toContain('No earlier turn is saved yet.'));
  expect([...host.querySelectorAll('button')].map((b) => b.textContent)).toEqual(['Cancel']);
});
