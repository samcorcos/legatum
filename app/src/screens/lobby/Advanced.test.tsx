// @vitest-environment happy-dom
// Advanced settings open themselves for a seed that needs fixing and never close under the player's typing.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { newLobby, type Lobby } from '../../lobby';
import { Settings } from './Settings';

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

it('stays open while a new seed is typed after clearing it', () => {
  let lobby: Lobby = newLobby('solo', '42');
  const render = () => act(() => root.render(<Settings lobby={lobby} onChange={(l) => { lobby = l; render(); }} />));
  render();
  const details = () => host.querySelector<HTMLDetailsElement>('details.lobby-advanced')!;
  expect(details().open).toBe(false);
  const type = (value: string) => {
    const input = host.querySelector<HTMLInputElement>('[aria-label="Map seed"]')!;
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!;
    act(() => { setter.call(input, value); input.dispatchEvent(new Event('input', { bubbles: true })); });
  };
  type('');
  expect(details().open).toBe(true);
  for (const v of ['4', '42', '424', '4242']) type(v);
  expect(lobby.seed).toBe('4242');
  expect(details().open).toBe(true);
});

it('stays open while the seed needs fixing, even when its summary is clicked', () => {
  let lobby: Lobby = { ...newLobby('solo', '42'), seed: 'abc' };
  const render = () => act(() => root.render(<Settings lobby={lobby} onChange={(l) => { lobby = l; render(); }} />));
  render();
  const details = host.querySelector<HTMLDetailsElement>('details.lobby-advanced')!;
  expect(details.open).toBe(true);
  act(() => { details.open = false; details.dispatchEvent(new Event('toggle')); });
  expect(details.open).toBe(true);
});
