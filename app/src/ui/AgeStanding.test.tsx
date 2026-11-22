// @vitest-environment happy-dom
// World Ages keep their VP in front of the player: the standing line follows the oldest open Age, and the era-up takeover
// opens with "Victory points open now" for every Age still unscored.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { withWorldAges } from '../test/vpV2Fixture';
import { buildVpModel } from '../vpModel';
import { SidebarObjective } from './EraObjectives';
import { EraUpTakeover } from '../celebrate/EraUpTakeover';
import { momentsForTurn } from '../celebrate/moments';
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
    expect(el.querySelector('.sidebar-section-link')?.textContent).toContain('Age of Classical · 1 VP');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(1);
    expect(el.querySelector('.eo-text')?.textContent).toBe('Veterans');
    expect(el.querySelector('.eo-num')?.textContent).toBe('2/4');
  });

  it('moves to the next Age once the oldest is scored', () => {
    const view = twoAges();
    view.eras = view.eras.map((e) => (e.era === 2 ? { ...e, scored_by: [...e.scored_by, { player: view.me, slot: 0, turn: 50 }] } : e));
    act(() => root.render(<SidebarObjective view={view} model={buildVpModel(view)} />));
    expect(host.querySelector('.sidebar-section-link')?.textContent).toContain('Age of Medieval · 1 VP');
    expect(host.querySelector('.eo-num')?.textContent).toBe('1/4');
  });
});

describe('the era-up takeover in a World Age game', () => {
  it('opens with the victory points open now: every unscored Age with your progress and who has scored it', () => {
    const view = twoAges();
    useGame.setState({ views: [view], seat: 0 });
    const m = momentsForTurn([{ type: 'EraEntered', player: 0, era: 2, first: false }], 0, base, view)[0];
    act(() => root.render(<EraUpTakeover moment={m} dismiss={vi.fn()} />));
    const panel = host.querySelector('[data-testid=era-up-objectives]')!;
    expect(panel.querySelector('h3')?.textContent).toContain('Victory points open now');
    expect(host.querySelector('.et-changes > .et-col')?.getAttribute('data-testid')).toBe('era-up-objectives');
    expect([...panel.querySelectorAll('.et-age h4')].map((h) => h.textContent)).toEqual(['Classical Age', 'Medieval Age']);
    expect([...panel.querySelectorAll('.eo-line .eo-num')].map((n) => n.textContent)).toEqual(['2/4', '1/4']);
    expect(panel.textContent).toContain('Scored by');
    expect(panel.textContent).not.toMatch(/crown/i);
  });

  it('leaves out an Age you have already scored', () => {
    const view = twoAges();
    view.eras = view.eras.map((e) => (e.era === 2 ? { ...e, scored_by: [...e.scored_by, { player: view.me, slot: 0, turn: 50 }] } : e));
    useGame.setState({ views: [view], seat: 0 });
    const m = momentsForTurn([{ type: 'EraEntered', player: 0, era: 2, first: false }], 0, base, view)[0];
    act(() => root.render(<EraUpTakeover moment={m} dismiss={vi.fn()} />));
    expect([...host.querySelectorAll('.et-age h4')].map((h) => h.textContent)).toEqual(['Medieval Age']);
  });
});
