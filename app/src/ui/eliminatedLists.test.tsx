// @vitest-environment happy-dom
// Eliminated empires leave every active list (race, diplomacy, alarms) and remain only
// in history: the turn report and the final standings.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { VpTrack } from './VpTrack';
import { ObjectiveSheet } from './ObjectiveSheet';
import { alarmsForTurn } from '../celebrate/alarms';
import { buildTracks } from '../vp';
import { isTeamAlive, livingPlayers } from '../empires';
import { eliminatedState, ELIMINATED } from '../test/eliminatedFixture';
import { factionName } from '../data';
import type { Event, Relation } from '../types';

let container: HTMLDivElement, root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  client.load(JSON.stringify(eliminatedState()));
  useGame.setState({ client, views: [client.view(0)], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: true });
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

const dead = factionName(ELIMINATED.faction);

it('the fixture view reports the empire as eliminated and still lists its standing', () => {
  const view = useGame.getState().views![0];
  expect(view.players.find((p) => p.id === ELIMINATED.player)!.alive).toBe(false);
  expect(view.standings.some((s) => s.team === ELIMINATED.team)).toBe(true);
  expect(isTeamAlive(view, ELIMINATED.team)).toBe(false);
  expect(livingPlayers(view)).toHaveLength(5);
});

it('the race lists only living empires, with no War badge for the eliminated one', () => {
  act(() => root.render(<VpTrack />));
  const rows = [...container.querySelectorAll('.vp-row')];
  expect(rows).toHaveLength(5);
  expect(container.textContent).not.toContain(dead);
  expect(container.querySelectorAll('.vhc-rel')).toHaveLength(4);
  expect(container.querySelector('.vp-row.dead')).toBeNull();
  // Final standings (history) still carry every empire.
  expect(buildTracks(useGame.getState().views![0])).toHaveLength(6);
});

it('the score ledger skips the eliminated empire', () => {
  useGame.setState({ vpExpanded: true });
  act(() => root.render(<ObjectiveSheet />));
  const sheet = document.body.querySelector('.vh-sheet')!;
  expect(sheet).not.toBeNull();
  expect(sheet.textContent).not.toContain(dead);
  expect(sheet.textContent).not.toContain('AI 5');
  expect(sheet.querySelectorAll('.sl-cols .sl-col')).toHaveLength(5);
  // The dead empire alone had completed a Great Work: nobody living has.
  expect(sheet.querySelector('.sl-row[data-row="RelayUplink"]')).toBeNull();
});

it('diplomacy lists only living empires and no betrayal record against the eliminated one', () => {
  act(() => root.render(<DiplomacyPanel />));
  const list = container.querySelector('.empire-list')!;
  expect(list.querySelectorAll('.empire-row')).toHaveLength(4);
  expect(list.textContent).not.toContain(dead);
  expect(container.querySelector('.betrayal-line')).toBeNull();
});

it('an alarm from an eliminated empire is not raised, but a living one is', () => {
  const view = useGame.getState().views![0];
  const war = (by: number): Event[] => [{ type: 'WarDeclared', relation: { a: 0, b: by, state: 'War', security: null, effective_turn: view.turn, guarantee_until: 0, version: 2, war_id: 9, declarers: [by] } as Relation }];
  expect(alarmsForTurn(war(ELIMINATED.team), view, view)).toEqual([]);
  expect(alarmsForTurn(war(1), view, view).map((a) => a.kind)).toEqual(['war']);
});
