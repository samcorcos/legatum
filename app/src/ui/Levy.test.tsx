// @vitest-environment happy-dom
// Thin levies in the UI: the armed-unit count wherever the army shows.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { meetAll, spawnUnit } from '../test/fixtureEngine';
import { fromOffset } from '../hex';
import { useGame } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { TopBar } from './TopBar';
import { LevyNote } from './Levy';
import { THIN_LEVIES } from '../data';
import type { PlayerView } from '../types';

let container: HTMLDivElement, root: Root, client: FixtureClient;
// Players: 0 Ada (Byzantium, team 0), 1 Bo (team 0), 2 team 1 (a rival), 3 team 2, ...
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'Byzantium', 'India', 'Ada', 'Bo'));
  const s = client.debugState();
  meetAll(s);
  for (const p of s.players) p.gold = 500;
  Object.assign(s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!, { state: 'War', war_id: 1, declarers: [0] });
  s.diplomacy!.next_war_id = 1;
  show(client.view(0));
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

function show(view: PlayerView, extra: Partial<ReturnType<typeof useGame.getState>> = {}) {
  act(() => { useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: true, diplomacyTeam: 2, ...extra }); });
}
it('shows the armed-unit count: top bar, city panel note, and a rival\'s row; a full levy warns in words and marks', async () => {
  const s = client.debugState();
  const view = () => client.view(0);
  s.units = s.units.filter((u) => u.owner !== 0 || u.kind === 'Settler');
  const home = s.cities.find((c) => c.owner === 0) ?? null;
  if (!home) {
    client.endTurn({});
  }
  const st = client.debugState();
  for (const u of st.units.filter((u) => u.owner === 0 && u.kind !== 'Settler')) st.units.splice(st.units.indexOf(u), 1);
  spawnUnit(st, 0, 'Soldier', fromOffset(1, 1), false);
  show(view(), { diplomacyOpen: false });
  act(() => root.render(<><TopBar /><LevyNote /></>));
  const l = view().players.find((p) => p.id === 0)!.levy!;
  expect(l.cap).toBe(view().cities.filter((c) => c.owner === 0).length + 1);
  const counter = container.querySelector('.levy-counter')!;
  expect(counter.textContent).toContain(`Military units ${l.units} / ${l.cap}`);
  expect(counter.querySelector('[aria-label]')!.getAttribute('aria-label')).toBe(`Military units ${l.units} / ${l.cap}`);
  expect(container.querySelector('.levy-note')!.textContent).toContain(`Military units ${l.units} / ${l.cap}`);
  // Fill the levy.
  for (let i = l.units; i < l.cap; i++) spawnUnit(st, 0, 'Soldier', fromOffset(1 + i, 2), false);
  show(view(), { diplomacyOpen: false });
  act(() => root.render(<><TopBar /><LevyNote /></>));
  // The top-bar chip stays short ("3 / 3" in red with the alert mark); the state word is in its tooltip and label.
  const full = container.querySelector('.levy-counter.full')!;
  expect(full.textContent).not.toContain('full');
  expect(full.querySelector('[aria-label]')!.getAttribute('aria-label')).toBe(`Military units ${l.cap} / ${l.cap} · full`);
  expect(container.querySelector('.levy-counter .ui-mark, .levy-counter svg, .levy-counter .mask-icon')).not.toBeNull();
  expect(container.querySelector('.levy-note.full')!.textContent).toContain('full: the military units below cannot be started or bought');
  // A rival sees it on Byzantium's diplomacy row.
  const rival = client.view(3);
  show(rival, { seat: 0, humans: [3], diplomacyOpen: true, diplomacyTeam: 0 });
  act(() => root.render(<DiplomacyPanel />));
  expect(container.querySelector('[data-testid="rival-levy"]')!.textContent).toBe(`Thin levies: ${l.cap} / ${l.cap} military units (it cannot raise another)`);
});

it('shows nothing about levies for any other faction, and states the rule once in plain words', async () => {
  const other = new FixtureClient();
  await other.newGame(await other.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  show(other.view(0), { client: other, diplomacyOpen: false });
  act(() => root.render(<><TopBar /><LevyNote /></>));
  expect(container.querySelector('.levy-counter')).toBeNull();
  expect(container.querySelector('.levy-note')).toBeNull();
  expect(THIN_LEVIES).toBe("Thin levies: you can field at most one military unit per city, plus one more. Aircraft count; Scouts don't. Losing a city never disbands units, but you can't raise new ones until you are back under the limit.");
});
