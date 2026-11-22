// @vitest-environment happy-dom
// World Ages is the only way Ages score (2026-11-22), so the lobby has no rule switch: setups and LAN room configs carry none.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { buildSetup, newLobby } from '../../lobby';
import { configFromLobby } from '../../staging';
import { Settings } from './Settings';

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

it('the lobby offers no rule experiment, and setups and room configs carry no switch', () => {
  const lobby = newLobby('solo', '42');
  act(() => root.render(<Settings lobby={lobby} onChange={() => {}} />));
  expect(host.querySelector('[data-testid=rule-experiments]')).toBeNull();
  expect(host.textContent).not.toContain('World Ages (experiment)');
  expect('rule_experiments' in buildSetup(lobby)).toBe(false);
  expect('rule_experiments' in configFromLobby(newLobby('host', '42'))).toBe(false);
});
