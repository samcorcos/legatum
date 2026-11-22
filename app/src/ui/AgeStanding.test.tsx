// @vitest-environment happy-dom
// World Ages keep their VP in front of the player: the standing line follows the oldest open Age. The score check at each
// new Age (`celebrate/ScoreCheck.test.tsx`) carries the rest.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { withWorldAges } from '../test/vpV2Fixture';
import { buildVpModel } from '../vpModel';
import { SidebarObjective } from './EraObjectives';
import type { PlayerView } from '../types';

let host: HTMLElement, root: Root, base: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  base = structuredClone(client.view(0));
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

const twoAges = () => withWorldAges(base);

describe('the standing line of the oldest open Age', () => {
  it('names the Age, the card, your progress and the 1 VP, without opening anything', () => {
    const view = twoAges();
    act(() => root.render(<SidebarObjective view={view} model={buildVpModel(view)} />));
    const el = host.querySelector('[data-testid=era-hud]')!;
    expect(el.querySelector('.sidebar-section-link')?.textContent).toContain('Classical Age · 1 VP');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(1);
    expect(el.querySelector('.eo-text')?.textContent).toBe('Veterans');
    expect(el.querySelector('.eo-num')?.textContent).toBe('2/4');
  });

  it('moves to the next Age once the oldest is scored', () => {
    const view = twoAges();
    view.eras = view.eras.map((e) => (e.era === 2 ? { ...e, scored_by: [...e.scored_by, { player: view.me, slot: 0, turn: 50 }] } : e));
    act(() => root.render(<SidebarObjective view={view} model={buildVpModel(view)} />));
    expect(host.querySelector('.sidebar-section-link')?.textContent).toContain('Medieval Age · 1 VP');
    expect(host.querySelector('.eo-num')?.textContent).toBe('1/4');
  });
});
