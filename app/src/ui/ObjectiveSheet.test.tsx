// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { ObjectiveSheet, VpChip } from './ObjectiveSheet';
import { useGame } from '../store';
import { withV2 } from '../test/vpV2Fixture';
import { buildVpModel } from '../vpModel';
import type { VpSource } from '../types';

let host: HTMLElement, root: Root;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('matchMedia', (query: string) => ({ matches: query.includes('min-height') && false, addEventListener() {}, removeEventListener() {} }));
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const base = client.view(0);
  const rival = base.players.find((p) => p.team !== base.team)!.id;
  const view = withV2(base, { era: 2, scored: [2] });
  view.players = view.players.map((p) => [view.me, rival].includes(p.id)
    ? { ...p, vp_sources: [...p.vp_sources, [{ kind: 'GreatWork', of: p.id }, 2] as [VpSource, number]] }
    : p);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0, vpExpanded: true });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });
const sheet = () => document.querySelector('.vh-sheet')!;
const row = (id: string) => sheet().querySelector(`.sl-row[data-row="${id}"]`)!;

it('keeps the dialog shell, close control and VP chip entry point', () => {
  useGame.setState({ vpExpanded: false });
  act(() => root.render(<><VpChip /><ObjectiveSheet /></>));
  expect(sheet()).toBeNull();
  const chip = host.querySelector<HTMLButtonElement>('.vh-chip')!;
  expect(chip.getAttribute('aria-label')).toMatch(/^Every way to score/);
  act(() => chip.click());
  expect(sheet().getAttribute('data-dismissible')).toBe('victory');
  expect(sheet().querySelector('.sl-table')).not.toBeNull();
  expect(sheet().querySelector('.vh-card')).toBeNull();
  expect(document.getElementById(sheet().getAttribute('aria-labelledby')!)?.textContent).toBe('Every Way to Score');
  act(() => sheet().querySelector<HTMLButtonElement>('.vh-close')!.click());
  expect(useGame.getState().vpExpanded).toBe(false);
});

it('shows all score sources in rank ordered columns, with the viewer gilt', () => {
  act(() => root.render(<ObjectiveSheet />));
  const view = useGame.getState().views![0], model = buildVpModel(view);
  expect([...sheet().querySelectorAll('.sl-cols .sl-col')].map((c) => c.getAttribute('title'))).toHaveLength(model.empires.length);
  expect([...sheet().querySelectorAll('.sl-cols .sl-col .sl-col-name')].map((c) => c.textContent)).toEqual(model.rows.flatMap((r) => r.members.map((e) => e.isMe ? 'You' : e.name.replace(/^Switzerland · AI /, 'Swiss · AI '))));
  expect([...sheet().querySelectorAll('.sl-ai-level')].map((e) => e.textContent)).toEqual(model.empires.filter((e) => e.aiLevel).map(() => 'AI · Normal'));
  expect(sheet().querySelectorAll('.sl-cols .sl-col.mine')).toHaveLength(1);
  expect(sheet().querySelectorAll('.sl-row.k-work')).toHaveLength(1);
  expect(sheet().querySelectorAll('.sl-row.k-goal')).toHaveLength(1);
  expect(sheet().querySelectorAll('.sl-row.k-heldGoal')).toHaveLength(1);
  // The Ages that have begun; the newest one you have not scored opens with the ledger.
  expect(sheet().querySelectorAll('.sl-row.k-era')).toHaveLength(2);
  expect(sheet().querySelectorAll('.sl-row-detail')).toHaveLength(1);
});

it('opens one row at a time with the public numbers, rule and era progress', () => {
  act(() => root.render(<ObjectiveSheet />));
  act(() => row('GreatWork').querySelector<HTMLButtonElement>('button')!.click());
  expect(sheet().querySelector('#sl-detail-GreatWork')).not.toBeNull();
  act(() => row('era-2').querySelector<HTMLButtonElement>('button')!.click());
  expect(sheet().querySelector('#sl-detail-GreatWork')).toBeNull();
  expect(sheet().querySelectorAll('#sl-detail-era-2 .vh-obj')).toHaveLength(3);
  expect(row('era-2').querySelector('.sl-token')?.getAttribute('title')).toMatch(/scored the Classical Age/);
});

it('shows team VP over its first member and hold turns in that head', () => {
  const v = structuredClone(useGame.getState().views![0]);
  const team = v.standings.find((s) => s.team === v.team)!;
  team.victory_hold = { turns_held: 4, turns_needed: 10 };
  team.vp = team.target;
  act(() => useGame.setState({ views: [v] }));
  act(() => root.render(<ObjectiveSheet />));
  const mine = [...sheet().querySelectorAll('.sl-col.own-team')];
  expect(mine).toHaveLength(team.members.length);
  expect(sheet().querySelectorAll('.sl-col.mine')).toHaveLength(1);
  expect(mine[0].querySelector('.sl-col-vp')!.textContent).toBe(`${team.target}/${team.target} VP`);
  expect(mine[0].querySelector('.sl-hold')!.textContent).toContain('Your team wins in 6');
  for (const other of mine.slice(1)) expect(other.querySelector('.sl-col-vp')!.textContent).toBe(`${team.target}/${team.target} VP`);
});

