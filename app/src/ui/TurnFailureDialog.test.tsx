// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { TurnFailureDialog } from './TurnFailureDialog';
import { saveToFile } from '../platform/files';
import { exportRetainedHistories } from '../platform/archive';
import { FixtureClient } from '../test/fixtureEngine';

vi.mock('../platform/files', async (orig) => ({ ...(await orig<typeof import('../platform/files')>()), saveToFile: vi.fn() }));
vi.mock('../platform/archive', () => ({
  exportRetainedHistories: vi.fn().mockResolvedValue(undefined),
  recordGameHistory: vi.fn().mockResolvedValue(undefined),
  abandonGameHistory: vi.fn().mockResolvedValue(undefined),
}));

let host: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
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
  vi.clearAllMocks();
});

const buttons = () => [...host.querySelectorAll('button')];

it('shows nothing until the turn has failed twice', () => {
  act(() => root.render(<TurnFailureDialog />));
  expect(host.textContent).toBe('');
});

it('offers one primary, Save and report, and a secondary that clears the orders', () => {
  const endWithout = vi.fn();
  useGame.setState({ turnFailure: { cleared: false }, endTurnWithoutOrders: endWithout });
  act(() => root.render(<TurnFailureDialog />));
  expect(host.querySelectorAll('.btn.primary')).toHaveLength(1);
  expect(host.querySelector('.btn.primary')!.textContent).toBe('Save and report');
  act(() => buttons().find((b) => b.textContent === 'End the turn with my orders cleared')!.click());
  expect(endWithout).toHaveBeenCalledTimes(1);
});

it('Save and report downloads the save and the history', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  useGame.setState({ client, views: [client.view(0), client.view(1)], turnFailure: { cleared: false } });
  act(() => root.render(<TurnFailureDialog />));
  await act(async () => { buttons().find((b) => b.textContent === 'Save and report')!.click(); });
  expect(saveToFile).toHaveBeenCalledTimes(1);
  expect(exportRetainedHistories).toHaveBeenCalledTimes(1);
  expect(host.querySelector('[role="status"]')!.textContent).toContain('Saved.');
});

it('once the try without orders failed too, Save and report is the only action', () => {
  useGame.setState({ turnFailure: { cleared: true } });
  act(() => root.render(<TurnFailureDialog />));
  expect(buttons().filter((b) => !b.classList.contains('end-turn-close')).map((b) => b.textContent)).toEqual(['Save and report']);
  expect(host.textContent).toContain('even without your orders');
});

it('Esc closes it', () => {
  useGame.setState({ turnFailure: { cleared: false } });
  act(() => root.render(<TurnFailureDialog />));
  act(() => { host.querySelector('dialog')!.dispatchEvent(new Event('cancel', { cancelable: true })); });
  expect(useGame.getState().turnFailure).toBeNull();
});
