// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { ObjectiveSheet } from './ObjectiveSheet';
import { useGame } from '../store';
import { withV2 } from '../test/vpV2Fixture';
import { buildVpModel } from '../vpModel';
import type { VpSource } from '../types';
import { meetEveryone } from '../test/meetEveryone';

let host: HTMLElement, root: Root;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.stubGlobal('matchMedia', (query: string) => ({ matches: query.includes('min-height') && false, addEventListener() {}, removeEventListener() {} }));
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const base = meetEveryone(client.view(0));
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

it('keeps the dialog shell and close control', () => {
  act(() => root.render(<ObjectiveSheet />));
  expect(sheet().getAttribute('data-dismissible')).toBe('victory');
  expect(sheet().querySelector('.sl-table')).not.toBeNull();
  expect(sheet().querySelector('.vh-card')).toBeNull();
  expect(document.getElementById(sheet().getAttribute('aria-labelledby')!)?.textContent).toBe('Score ledger');
  expect(sheet().querySelector('.vh-close')?.getAttribute('aria-label')).toBe('Close score ledger');
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
  expect(row('era-2').querySelector('.sl-token')?.getAttribute('aria-label')).toMatch(/scored the Classical Age/);
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

it('an Age scored with a card no longer drawn shows a token beside "scored", never an empty socket', () => {
  // A turn-203 save: you scored Age II with a card whose slot is gone; the status word and the token use one check.
  const view = structuredClone(useGame.getState().views![0]);
  view.eras = view.eras.map((ev) => ev.era === 2
    ? { ...ev, objectives: ev.objectives.slice(0, 1), scored_by: ev.scored_by.map((s) => s.player === view.me ? { ...s, slot: 7 } : s) }
    : ev);
  useGame.setState({ views: [view] });
  act(() => root.render(<ObjectiveSheet />));
  const era2 = row('era-2');
  expect(era2.textContent).toContain('scored');
  const mine = era2.querySelector('.sl-token.mine')!;
  expect(mine.getAttribute('aria-label')).toBe('You scored the Classical Age');
  expect(era2.querySelector('.sl-socket[aria-label^="You"]')).toBeNull();
});


it('hides an empire you have not met: only its rank and VP show, no name, crest, goal, Gold or AI level', async () => {
  const view = useGame.getState().views![0];
  const rival = view.players.find((p) => p.team !== view.team)!;
  const { factionName, FACTION_GOAL } = await import('../data');
  const others = [...new Set(view.players.map((p) => p.team))].filter((t) => t !== view.team);
  const unmet = { ...view, diplomacy: { ...view.diplomacy, met: others.filter((t) => t !== rival.team), relations: view.diplomacy.relations.filter((r) => r.state !== 'War') } };
  useGame.setState({ views: [unmet] });
  act(() => root.render(<ObjectiveSheet />));
  const col = [...sheet().querySelectorAll<HTMLElement>('th.sl-col')].find((c) => c.classList.contains('unmet'))!;
  expect(col).toBeTruthy();
  expect(col.querySelector('.sl-col-name')!.textContent).toBe('Unknown');
  expect(col.querySelector('.sl-gold')).toBeNull();
  expect(col.querySelector('.vh-unknown')).not.toBeNull();
  expect(col.querySelector('.sl-col-vp b')!.textContent).toBe(String(view.standings.find((s) => s.team === rival.team)!.vp));
  // Open every faction-goal row: the unmet rival's faction and goal appear nowhere in the ledger.
  for (const id of ['goal', 'held-goal', 'capitals-held']) {
    const btn = row(id)?.querySelector<HTMLButtonElement>('.sl-row-btn');
    if (btn && btn.getAttribute('aria-expanded') !== 'true') act(() => btn.click());
    const text = sheet().textContent!;
    expect(text).not.toContain(factionName(rival.faction));
    expect(text).not.toContain(FACTION_GOAL[rival.faction].name);
  }
  expect(sheet().textContent).toContain('1 empire not met yet. Explore to meet them.');
  const model = buildVpModel(unmet);
  expect(model.empires.find((e) => e.id === rival.id)!.name).toBe('Unknown empire');
  expect(model.rows.find((r) => r.team === rival.team)!.label).toBe('Unknown');
  // A War names the enemy even before contact.
  const atWar = { ...unmet, diplomacy: { ...unmet.diplomacy, relations: [...unmet.diplomacy.relations, { a: view.team, b: rival.team, state: 'War' } as (typeof view.diplomacy.relations)[number]] } };
  expect(buildVpModel(atWar).empires.find((e) => e.id === rival.id)!.unmet).toBe(false);
});
