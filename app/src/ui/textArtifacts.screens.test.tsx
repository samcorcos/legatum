// @vitest-environment happy-dom
// The artifact guard and the copy lint on live screens: the diplomacy panel in its richest states (offers, wars an Alliance joins).
// See textArtifacts.test.tsx for the data-level guard and the rule.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { meetAll, spawnUnit } from '../test/fixtureEngine';
import { fromOffset } from '../hex';
import { useGame } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { auditBlocks, auditCopy, htmlBlocks } from '../test/textArtifacts';
import type { PlayerView } from '../types';

let container: HTMLDivElement, root: Root, client: FixtureClient;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

function show(view: PlayerView, extra: Partial<ReturnType<typeof useGame.getState>> = {}) {
  act(() => {
    useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: true, ...extra });
    root.render(<DiplomacyPanel />);
  });
}
const audit = (label: string) => { const blocks = htmlBlocks(container.innerHTML); return [...auditBlocks(label, blocks), ...auditCopy(label, blocks)]; };

it('the diplomacy panel reads cleanly with offers pending, at war, and with an Alliance that carries it into a war', async () => {
  const hits: string[] = [];

  client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  let s = client.debugState();
  meetAll(s);
  for (const p of s.players) p.gold = 1000;
  s.diplomacy!.proposals = [
    { id: 1, kind: 'Alliance', from: 1, to: 0, proposer: 2, created_turn: 1, respond_by: 3, relation_version: 0 },
    { id: 2, kind: 'Alliance', from: 0, to: 2, proposer: 0, created_turn: 1, respond_by: 3, relation_version: 0 },
  ];
  show(client.view(0));
  hits.push(...audit('diplomacy offers'));
  show(client.view(0), { diplomacyTeam: 1 });
  hits.push(...audit('diplomacy empire row'));

  act(() => root.unmount());
  root = createRoot(container);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'Byzantium', 'India', 'Ada', 'Bo'));
  s = client.debugState();
  meetAll(s);
  for (const p of s.players) p.gold = 500;
  s.units = s.units.filter((u) => u.owner !== 2);
  for (let i = 0; i < 4; i++) spawnUnit(s, 2, 'Soldier', fromOffset(10 + i, 10), false);
  show(client.view(0), { diplomacyTeam: 1 });
  hits.push(...audit('diplomacy at peace (row)'));
  s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!.state = 'War';
  show(client.view(0), { diplomacyTeam: 1 });
  hits.push(...audit('diplomacy at war (row)'));

  // An Alliance offered by an empire at war: the offer says which war you would join; a declaration says which allies join.
  act(() => root.unmount());
  root = createRoot(container);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  s = client.debugState();
  meetAll(s);
  for (const p of s.players) p.gold = 1000;
  Object.assign(s.diplomacy!.relations.find((r) => r.a === 1 && r.b === 2)!, { state: 'War', war_id: 1, declarers: [2], effective_turn: 2 });
  s.diplomacy!.next_war_id = 1;
  s.diplomacy!.proposals = [{ id: 1, kind: 'Alliance', from: 1, to: 0, proposer: 2, created_turn: 1, respond_by: 3, relation_version: 0 }];
  show(client.view(0));
  expect(container.textContent).toContain('You would join the war against');
  hits.push(...audit('diplomacy alliance offer in wartime'));
  show(client.view(0), { diplomacyTeam: 2 });
  hits.push(...audit('diplomacy row of the enemy of an ally-to-be'));

  // A call to arms from an ally: Join or Decline, and what each does.
  const d = s.diplomacy!;
  d.proposals = [];
  Object.assign(d.relations.find((r) => r.a === 0 && r.b === 1)!, { state: 'Alliance', security: { id: 1, started_turn: 1, protected_until: 11 }, effective_turn: 1 });
  Object.assign(d.relations.find((r) => r.a === 1 && r.b === 2)!, { state: 'War', war_id: 1, declarers: [1], effective_turn: 2 });
  d.next_treaty_id = 1; d.next_proposal_id = 7;
  const rel = (a: number, b: number) => d.relations.find((r) => r.a === a && r.b === b)!;
  d.calls = [{ id: 7, team: 0, ally: 1, target: 2, created_turn: client.view(0).turn, respond_by: client.view(0).turn + 1, alliance_version: rel(0, 1).version, war_version: rel(1, 2).version }];
  show(client.view(0));
  expect(container.textContent).toContain('calls you to war with');
  hits.push(...audit('diplomacy call to arms'));

  expect(container.innerHTML.length).toBeGreaterThan(500);
  expect(hits, hits.join('\n')).toEqual([]);
});
