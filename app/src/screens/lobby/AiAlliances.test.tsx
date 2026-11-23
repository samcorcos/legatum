// @vitest-environment happy-dom
// The Co-op preset starts the AI empires allied in pairs; the lobby shows it and lets the host turn it off.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { aiAlliancesOn, aiOnlyTeams, applyPreset, buildSetup, newLobby } from '../../lobby';
import { configFromLobby, lobbyFromStaging, type StagingState } from '../../staging';
import { Settings } from './Settings';

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

const radio = (name: string, label: string) =>
  [...host.querySelectorAll<HTMLElement>(`[role=radiogroup][aria-label="${name}"] [role=radio]`)].find((b) => b.textContent?.startsWith(label))!;

it('only the Co-op preset turns AI alliances on', () => {
  expect(newLobby('host', '42').aiAlliances).toBe(true);
  expect(newLobby('host', '42').aiAlliances).toBe(true);
  expect(newLobby('solo', '42').aiAlliances).toBe(false);
  expect(applyPreset(newLobby('solo', '42'), 'teams').aiAlliances).toBe(false);
  expect(applyPreset(newLobby('host', '42'), 'ffa').aiAlliances).toBe(false);
  expect(applyPreset(newLobby('solo', '42'), 'coop').aiAlliances).toBe(true);
});

it('the setup asks for AI alliances only when it is on and two AI teams exist', () => {
  const coop = newLobby('host', '42');
  expect(aiOnlyTeams(coop)).toBe(6);
  expect(buildSetup(coop).ai_alliances).toBe(true);
  expect('ai_alliances' in buildSetup({ ...coop, aiAlliances: false })).toBe(false);
  expect('ai_alliances' in buildSetup(newLobby('solo', '42'))).toBe(false);
});

it('a room config carries the switch, and an old room without it reads as off', () => {
  const lobby = newLobby('host', '42');
  const config = configFromLobby(lobby);
  expect(config.ai_alliances).toBe(true);
  expect('ai_alliances' in configFromLobby({ ...lobby, aiAlliances: false })).toBe(false);
  const { seed, ...settings } = config;
  const state = { room_id: 'r', config: settings, seed, occupants: config.seats.map(() => null) } as StagingState;
  expect(lobbyFromStaging(state).aiAlliances).toBe(true);
  const { ai_alliances, ...old } = settings;
  expect(lobbyFromStaging({ ...state, config: old }).aiAlliances).toBe(false);
});

it('the lobby shows the setting and the host can turn it off', () => {
  let lobby = newLobby('host', '42');
  const render = () => act(() => root.render(<Settings lobby={lobby} onChange={(l) => { lobby = l; render(); }} />));
  render();
  expect(radio('AI alliances', 'On').getAttribute('aria-checked')).toBe('true');
  expect(host.textContent).toContain('AI empires start allied in pairs');
  act(() => radio('AI alliances', 'Off').click());
  expect(lobby.aiAlliances).toBe(false);
  expect('ai_alliances' in buildSetup(lobby)).toBe(false);
});

it('a read-only view shows the setting without a way to change it', () => {
  act(() => root.render(<Settings lobby={newLobby('host', '42')} />));
  expect(radio('AI alliances', 'On').getAttribute('aria-checked')).toBe('true');
  expect(radio('AI alliances', 'Off').hasAttribute('disabled')).toBe(true);
});

it('the Start button line and the setup agree on when AI alliances are on', () => {
  const coop = newLobby('host', '42');
  expect(aiAlliancesOn(coop)).toBe(true);
  expect(aiAlliancesOn({ ...coop, aiAlliances: false })).toBe(false);
  expect(aiAlliancesOn(newLobby('solo', '42'))).toBe(false);
  expect(aiAlliancesOn(coop)).toBe('ai_alliances' in buildSetup(coop));
});
