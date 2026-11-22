// @vitest-environment happy-dom
// Allies defend each other and are called to each other's wars, in the diplomacy panel: the wars an
// Alliance would bring you into, the allies who defend or are asked on a declaration, the call to
// arms card, and a Peace that leaves the offerer's ally at war.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { meetAll } from '../test/fixtureEngine';
import { useGame } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { incomingOffers, proposalCounterparty } from '../diplomacy';
import { pendingDecisions } from '../turnFlow';
import type { PlayerView } from '../types';

let container: HTMLDivElement, root: Root, client: FixtureClient;
// Players: 0 Ada and 1 Bo (team 0), 2 (team 1), 3 (team 2), 4 (team 3), ...
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  const s = client.debugState();
  meetAll(s);
  for (const p of s.players) p.gold = 1000;
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

function show(view: PlayerView, extra: Partial<ReturnType<typeof useGame.getState>> = {}) {
  act(() => { useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: true, ...extra }); });
  act(() => root.render(<DiplomacyPanel />));
}
const relation = (a: number, b: number) => client.debugState().diplomacy!.relations.find((r) => r.a === Math.min(a, b) && r.b === Math.max(a, b))!;

it('lists the ally who would join the war you are defending', () => {
  Object.assign(relation(0, 2), { state: 'War', war_id: 1, declarers: [2], effective_turn: 2 });
  client.debugState().diplomacy!.next_war_id = 1;
  show(client.view(0), { diplomacyTeam: 1 });
  const joins = client.view(0).diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.to === 1 && a.order.kind === 'Alliance')!.new_wars;
  expect(joins).toEqual([{ team: 1, enemy: 2 }]);
  expect(container.querySelector('.empire-row.open')!.textContent).toMatch(/would join your war against/);
  expect(container.querySelector('.empire-row.open')!.textContent).not.toMatch(/You would join the war/);
});

it('says which war you would join before you sign', () => {
  Object.assign(relation(1, 2), { state: 'War', war_id: 1, declarers: [2], effective_turn: 2 });
  client.debugState().diplomacy!.next_war_id = 1;
  show(client.view(0), { diplomacyTeam: 1 });
  expect(container.querySelector('.empire-row.open')!.textContent).toMatch(/Joins you to the war against/);
  // Directly under the Offer Alliance button.
  expect(container.querySelector('.empire-row.open .diplomacy-action button[aria-label="Offer Alliance"] + [data-testid=join-war]')).not.toBeNull();
  const joins = client.view(0).diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.to === 1 && a.order.kind === 'Alliance')!.new_wars;
  expect(joins).toEqual([{ team: 0, enemy: 2 }]);
});

it('says which allies defend a declaration\'s target and which of yours are asked to join', () => {
  Object.assign(relation(0, 1), { state: 'Alliance', security: { id: 1, started_turn: 1, protected_until: 11 }, effective_turn: 1 });
  Object.assign(relation(2, 3), { state: 'Alliance', security: { id: 2, started_turn: 1, protected_until: 11 }, effective_turn: 1 });
  client.debugState().diplomacy!.next_treaty_id = 2;
  show(client.view(0), { diplomacyTeam: 2 });
  const text = container.querySelector('.empire-row.open')!.textContent!;
  expect(text).toMatch(/would defend them and be at War with you at once/);
  expect(text).toMatch(/your ally, would be asked to join this war\. Each may decline/);
});

/** Team 1 (player 2) is allied with us (team 0) and at war with team 2 (player 3): a call to arms for us. */
function callToArms() {
  const d = client.debugState().diplomacy!;
  Object.assign(relation(0, 1), { state: 'Alliance', security: { id: 1, started_turn: 1, protected_until: 11 }, effective_turn: 1 });
  Object.assign(relation(1, 2), { state: 'War', war_id: 1, declarers: [1], effective_turn: 2 });
  d.next_treaty_id = 1; d.next_war_id = 1; d.next_proposal_id = 5;
  d.calls = [{ id: 5, team: 0, ally: 1, target: 2, created_turn: client.view(0).turn, respond_by: client.view(0).turn + 1, alliance_version: relation(0, 1).version, war_version: relation(1, 2).version }];
}

it('shows a call to arms as one decision: Join is the primary action, Decline sits beside it and keeps the Alliance', () => {
  callToArms();
  const v = client.view(0);
  show(v);
  const card = container.querySelector('.call-card')!;
  expect(card).not.toBeNull();
  expect(card.textContent).toMatch(/calls you to war with/);
  expect(card.textContent).toContain('Declining keeps your Alliance');
  expect(card.textContent).toContain('Declining costs nothing.');
  const buttons = [...card.querySelectorAll<HTMLButtonElement>('.offer-actions button')];
  expect(buttons.map((b) => b.getAttribute('aria-label'))).toEqual([expect.stringMatching(/^Join the war on /), 'Decline']);
  expect(buttons.filter((b) => b.classList.contains('primary'))).toHaveLength(1);
  expect(buttons[0].classList.contains('primary')).toBe(true);
  // It is a decision to make before ending the turn.
  expect(pendingDecisions(v, [], []).filter((d) => d.kind === 'diplomacy').map((d) => d.id)).toEqual([5]);
});

it('Join queues the answer; Decline queues a decline; either settles the decision', () => {
  callToArms();
  show(client.view(0));
  const labelled = (re: RegExp) => [...container.querySelectorAll<HTMLButtonElement>('.call-card button')].find((b) => re.test(b.getAttribute('aria-label') ?? ''))!;
  act(() => labelled(/^Join the war on /).click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'RespondToProposal', proposal: 5, accept: true }]);
  expect(pendingDecisions(client.view(0), useGame.getState().orders[0], []).filter((d) => d.kind === 'diplomacy')).toEqual([]);
  act(() => { useGame.setState({ orders: [[]], diplomacyOpen: true }); });
  act(() => root.render(<DiplomacyPanel />));
  act(() => labelled(/^Decline$/).click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'RespondToProposal', proposal: 5, accept: false }]);
});

it('an incoming Peace ends one war and names the offerers allied war that continues', () => {
  Object.assign(relation(2, 3), { state: 'Alliance', security: { id: 1, started_turn: 1, protected_until: 11 }, effective_turn: 1 });
  for (const [a, b] of [[0, 2], [0, 3]]) Object.assign(relation(a, b), { state: 'War', war_id: 1, declarers: [b], effective_turn: 2 });
  const d = client.debugState().diplomacy!;
  d.next_treaty_id = 1; d.next_war_id = 1;
  d.proposals = [{ id: 5, kind: 'Peace', from: 2, to: 0, proposer: 3, created_turn: 1, respond_by: 4, relation_version: relation(0, 2).version, consents: [2] }];
  const v = client.view(0);
  expect(v.diplomacy.proposals[0].needs).toEqual([0]);
  expect(v.diplomacy.proposals[0].required_teams).toEqual([0, 2]);
  expect(incomingOffers(v)).toHaveLength(1);
  expect(proposalCounterparty(v, v.diplomacy.proposals[0], 0)).toBe(2);
  show(v);
  const card = container.querySelector('.offer-card')!.textContent!;
  expect(card).toContain('offers Peace');
  expect(card).toMatch(/Ends your war with .+\./);
  expect(card).toMatch(/Your war with .+ continues/);
  expect(card).toContain('Both teams must agree.');
  // The fact sits right under Accept, before the meaning and the forecast.
  expect(container.querySelector('.offer-actions + [data-testid=offer-agree]')).not.toBeNull();
  // Accepting is queued like any answer.
  const accept = [...container.querySelectorAll<HTMLButtonElement>('.offer-card button')].find((b) => /^Accept/.test(b.getAttribute('aria-label') ?? ''))!;
  expect(accept.disabled).toBe(false);
  act(() => accept.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'RespondToProposal', proposal: 5, accept: true }]);
});

it('names the allied enemy on an outgoing Peace action', () => {
  Object.assign(relation(2, 3), { state: 'Alliance', security: { id: 1, started_turn: 1, protected_until: 11 }, effective_turn: 1 });
  for (const [a, b] of [[0, 2], [0, 3]]) Object.assign(relation(a, b), { state: 'War', war_id: 1, declarers: [b], effective_turn: 2 });
  client.debugState().diplomacy!.next_treaty_id = 1;
  client.debugState().diplomacy!.next_war_id = 1;
  const view = client.view(0);
  expect(view.diplomacy.actions.some((a) => a.order.type === 'ProposeTreaty' && a.order.kind === 'Peace' && a.order.to === 2)).toBe(true);
  show(view, { diplomacyTeam: 2 });
  const text = container.querySelector('.empire-row.open')!.textContent!;
  expect(text).toMatch(/Peace with .+: ends only your war with/);
  expect(text).toMatch(/Your war with .+ continues/);
});

it('an Alliance offer from an empire at Peace prints no Peace rule', () => {
  const d = client.debugState().diplomacy!;
  expect(relation(0, 2).state).toBe('Peace');
  d.next_treaty_id = 1;
  d.proposals = [{ id: 7, kind: 'Alliance', from: 2, to: 0, proposer: 3, created_turn: 1, respond_by: 4, relation_version: relation(0, 2).version, consents: [2] }];
  const v = client.view(0);
  expect(incomingOffers(v)).toHaveLength(1);
  show(v);
  const card = container.querySelector('.offer-card')!.textContent!;
  expect(card).toContain('Alliance');
  expect(card).not.toContain('Accepting ends your wars');
  expect(card).not.toContain('Every empire on both sides must agree');
  expect(container.querySelector('[data-testid=offer-agree]')).toBeNull();
});
