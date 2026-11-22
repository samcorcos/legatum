// @vitest-environment happy-dom
// The Rule experiments section: two switches, off by default, carried into the game setup and the LAN room
// config, and read-only for a guest.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { buildSetup, newLobby, type Lobby } from '../../lobby';
import { configFromLobby, lobbyFromStaging, type StagingState } from '../../staging';
import { Settings } from './Settings';

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

const radio = (label: string, value: string) =>
  host.querySelector<HTMLButtonElement>(`[role=radiogroup][aria-label="${label}"] [role=radio]:is(:nth-child(${value === 'Off' ? 1 : 2}))`)!;

it('the switch starts off, says what it does, and turns on through the lobby', () => {
  let lobby = newLobby('solo', '42');
  const render = (onChange?: (l: Lobby) => void) => act(() => root.render(<Settings lobby={lobby} onChange={onChange} />));
  render((l) => { lobby = l; });
  const section = host.querySelector('[data-testid=rule-experiments]')!;
  expect(section.textContent).toContain('World Ages (experiment)');
  expect(section.textContent).toContain('Each era opens one randomly drawn card to every empire. Score it for 1 VP, kept for good.');
  expect(section.textContent).not.toContain('Occupation lock');
  expect(radio('World Ages (experiment)', 'Off').getAttribute('aria-checked')).toBe('true');
  expect(buildSetup(lobby).rule_experiments).toBeUndefined();
  act(() => radio('World Ages (experiment)', 'On').click());
  expect(buildSetup(lobby).rule_experiments).toEqual({ objectives_deck: true });
  render((l) => { lobby = l; });
  act(() => radio('World Ages (experiment)', 'Off').click());
  expect(buildSetup(lobby).rule_experiments).toBeUndefined();
});

it('the room config carries the switch, stays out of default rooms, and a guest sees it read-only', () => {
  const lobby = { ...newLobby('host', '42'), rule_experiments: { objectives_deck: true } };
  expect('rule_experiments' in configFromLobby(newLobby('host', '42'))).toBe(false);
  expect(configFromLobby({ ...newLobby('host', '42'), rule_experiments: { objectives_deck: false } }).rule_experiments).toBeUndefined();
  const config = configFromLobby(lobby);
  expect(config.rule_experiments).toEqual({ objectives_deck: true });
  const { seed, ...settings } = config;
  const guest = lobbyFromStaging({ room_id: 'r', config: settings, occupants: [] } as StagingState);
  expect(guest.rule_experiments).toEqual({ objectives_deck: true });
  act(() => root.render(<Settings lobby={guest} />));
  expect(radio('World Ages (experiment)', 'On').getAttribute('aria-checked')).toBe('true');
  expect(radio('World Ages (experiment)', 'Off').disabled).toBe(true);
  expect(seed).toBe('42');
});
