// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { SaveNotice } from './SaveNotice';
import { saveToFile } from '../platform/files';
import { FixtureClient } from '../test/fixtureEngine';

vi.mock('../platform/files', async (orig) => ({ ...(await orig<typeof import('../platform/files')>()), saveToFile: vi.fn() }));

let host: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => {
  act(() => root.unmount());
  host.remove();
  vi.unstubAllGlobals();
});

it('shows nothing until an autosave fails', () => {
  act(() => root.render(<SaveNotice />));
  expect(host.textContent).toBe('');
});

it('says so calmly, offers Save to file as its one action, and goes away after it', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  useGame.setState({ client, views: [client.view(0), client.view(1)], autosaveNotice: true });
  act(() => root.render(<SaveNotice />));
  expect(host.querySelector('[role="status"]')!.textContent).toContain('This game could not be saved in this browser. Save it to a file to keep it.');
  const buttons = [...host.querySelectorAll('button')];
  expect(buttons.filter((b) => b.textContent === 'Save to file')).toHaveLength(1);
  expect(host.querySelector('.btn.primary')).toBeNull();
  act(() => buttons.find((b) => b.textContent === 'Save to file')!.click());
  expect(saveToFile).toHaveBeenCalledTimes(1);
  expect(useGame.getState().autosaveNotice).toBe(false);
  expect(host.textContent).toBe('');
});

it('can be dismissed', () => {
  useGame.setState({ autosaveNotice: true });
  act(() => root.render(<SaveNotice />));
  act(() => host.querySelector<HTMLButtonElement>('[aria-label="Dismiss"]')!.click());
  expect(useGame.getState().autosaveNotice).toBe(false);
});
