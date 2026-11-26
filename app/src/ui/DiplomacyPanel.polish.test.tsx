// @vitest-environment happy-dom
// Diplomacy polish from live play: empires not met stay hidden (as in the rivals table), a waiting offer says whose
// answer it waits for, a turned-down or lapsed offer stays on its empire's row for a few turns, and an offer button
// says which queued order it would replace (one diplomatic order per empire per turn).
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient, meetAll } from '../test/fixtureEngine';
import { useGame, type TurnReport } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { empireTitle } from '../empires';
import { hiddenEmpire, pendingOfferReason, possessiveName, recentOfferOutcomes, replacesText } from '../offerStatus';
import type { Event, PeaceProposal, PlayerView } from '../types';

let container: HTMLDivElement, root: Root;
async function start({ proposals = [] as PeaceProposal[], edit = (_: PlayerView) => {} } = {}) {
  const client = new FixtureClient();
  const setup = await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo');
  setup.deals = true;
  await client.newGame(setup);
  const s = client.debugState();
  meetAll(s);
  s.diplomacy!.proposals = proposals;
  for (const p of s.players) p.gold = 120;
  const view = client.view(0);
  edit(view);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: true, diplomacyTeam: 1 });
  return view;
}
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });
const button = (prefix: string) => [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent?.startsWith(prefix));
const render = () => act(() => root.render(<DiplomacyPanel />));
const rowNames = () => [...container.querySelectorAll('.empire-pick-name')].map((e) => e.textContent);
const unmet = (view: PlayerView, ...teams: number[]) => { view.diplomacy.met = view.diplomacy.met!.filter((t) => !teams.includes(t)); };

describe('empires not met yet', () => {
  it('are hidden and counted on one line, as the rivals table shows them', async () => {
    let name = '';
    const view = await start({ edit: (v) => { name = empireTitle(v, 1); unmet(v, 1); } });
    render();
    expect(rowNames()).not.toContain(name);
    expect(container.textContent).not.toContain(name);
    expect(container.querySelector('[data-testid=unmet-empires]')!.textContent).toBe('?1 empire not met yet. Explore to meet them: offers need contact first.');
    // The panel opens on a met empire instead of the hidden one it was asked for.
    expect(container.querySelector('.empire-row.open .empire-pick-name')!.textContent).not.toBe(name);
    expect(hiddenEmpire(view, 1)).toBe(true);
    expect(hiddenEmpire(view, view.team)).toBe(false);
  });

  it('a War, or your own declaration, names an empire you have not met', async () => {
    const view = await start({ edit: (v) => unmet(v, 1) });
    expect(hiddenEmpire(view, 1, [{ type: 'DeclareWar', against: 1 }])).toBe(false);
    const war = structuredClone(view);
    war.diplomacy.relations = war.diplomacy.relations.map((r) => !(!(!((!(!(r.a === 1) && !(r.b === 1)))) || !((!(!(r.a === view.team) && !(r.b === view.team)))))) ? r : { ...r, state: 'War' });
    expect(hiddenEmpire(war, 1)).toBe(false);
    // A save from before first contact has no record: every empire counts as met.
    expect(hiddenEmpire({ ...view, diplomacy: { ...view.diplomacy, met: undefined } }, 1)).toBe(false);
  });

  it('every empire met: no unmet line', async () => {
    await start();
    render();
    expect(container.querySelector('[data-testid=unmet-empires]')).toBeNull();
  });
});

describe('an offer waiting for an answer', () => {
  it('our own: the other offers say whose answer they wait for, and by when', async () => {
    const view = await start({ proposals: [{ id: 1, kind: 'OpenBorders', from: 0, to: 1, proposer: 0, created_turn: 1, respond_by: 6, relation_version: 0 }] });
    render();
    const them = possessiveName(empireTitle(view, 1));
    expect(pendingOfferReason(view, 1)).toBe(`Waiting for ${them} answer to your Open Borders offer, due turn 6.`);
    expect(container.querySelector('.empire-row.open .diplomacy-unavailable')!.textContent)
      .toBe(`Not now: Offer Alliance, Offer Open Borders. Waiting for ${them} answer to your Open Borders offer, due turn 6.`);
    expect(container.textContent).not.toContain('outstanding proposal');
  });

  it('theirs: the offers point to the Inbox', async () => {
    const view = await start({ proposals: [{ id: 1, kind: 'Peace', from: 1, to: 0, proposer: 2, created_turn: 1, respond_by: 3, relation_version: 0 }] });
    expect(pendingOfferReason(view, 1)).toBe(`${possessiveName(empireTitle(view, 1))} Peace offer waits for your answer in the Inbox, due turn 3.`);
    expect(pendingOfferReason(view, 2)).toBeNull();
  });

  it('names the possessive of a name ending in s with the apostrophe alone', () => {
    expect(possessiveName('Athens')).toBe('Athens’');
    expect(possessiveName('Sparta')).toBe('Sparta’s');
  });
});

describe('a turned-down or lapsed offer', () => {
  const report = (turn: number, events: Event[]): TurnReport => ({ turn, hash: '', lines: [[]], events: [events] });
  const offer = (kind: PeaceProposal['kind'], turn: number): PeaceProposal => ({ id: 7, kind, from: 0, to: 1, proposer: 0, created_turn: turn - 2, respond_by: turn - 1, relation_version: 0 });

  it('shows on its empire row for a few turns', async () => {
    const view = await start();
    const t = view.turn;
    useGame.setState({ reportHistory: [
      report(t - 1, [{ type: 'ProposalRejected', proposal: offer('OpenBorders', t - 1), effective_turn: t - 1 }]),
      report(t, [{ type: 'ProposalExpired', proposal: offer('Alliance', t), effective_turn: t }]),
    ] });
    render();
    const lines = [...container.querySelectorAll('.empire-row [data-testid=offer-outcome]')].map((e) => e.textContent);
    expect(lines).toEqual([`Didn’t answer your Alliance offer · turn ${t}`, `Rejected your Open Borders offer · turn ${t - 1}`]);
    // On that empire's row only.
    expect(container.querySelector('.empire-row.open [data-testid=offer-outcome]')).not.toBeNull();
  });

  it('fades after five turns, ignores other empires and offers made to us, and names why an offer ended', async () => {
    const view = await start();
    const t = view.turn;
    const reports = [
      report(t - 5, [{ type: 'ProposalRejected', proposal: offer('Peace', t - 5), effective_turn: t - 5 }]),
      report(t - 1, [
        { type: 'ProposalRejected', proposal: { ...offer('Alliance', t - 1), to: 2 }, effective_turn: t - 1 },
        { type: 'ProposalExpired', proposal: { ...offer('Alliance', t - 1), from: 1, to: 0 }, effective_turn: t - 1 },
        { type: 'ProposalInvalidated', proposal: offer('Peace', t - 1), effective_turn: t - 1, reason: 'War was declared' },
      ]),
    ];
    expect(recentOfferOutcomes(view, reports, 0, 1).map((o) => o.text)).toEqual([`Your Peace offer ended: War was declared · turn ${t - 1}`]);
  });

  it('gives way once the same offer is made again', async () => {
    const view = await start({ proposals: [{ ...offer('OpenBorders', 3), id: 9, respond_by: 9 }] });
    const reports = [report(view.turn, [{ type: 'ProposalRejected', proposal: offer('OpenBorders', view.turn), effective_turn: view.turn }])];
    expect(recentOfferOutcomes(view, reports, 0, 1)).toEqual([]);
  });
});

describe('one diplomatic order per empire per turn', () => {
  it('with an offer queued, the other buttons say they replace it', async () => {
    const view = await start();
    render();
    act(() => button('Offer Open Borders')!.click());
    expect(useGame.getState().orders[0]).toEqual([{ type: 'ProposeTreaty', to: 1, kind: 'OpenBorders' }]);
    const notes = [...container.querySelectorAll('.empire-row.open .diplomacy-action')]
      .flatMap((a) => !(a.querySelector('[data-testid=diplomacy-replaces]')) ? [] : [[a.querySelector('button')!.textContent, a.querySelector('[data-testid=diplomacy-replaces]')!.textContent]]);
    expect(notes).toContainEqual(['Offer Alliance', ' Replaces your Open Borders offer']);
    expect(notes.find(([b]) => b?.startsWith('Declare War'))?.[1]).toBe(' Replaces your Open Borders offer');
    // The queued one itself says nothing of the kind.
    expect(notes.some(([b]) => b?.startsWith('Offer Open Borders'))).toBe(false);
    // Taking another offer replaces the queued one, as the note said.
    act(() => button('Offer Alliance')!.click());
    expect(useGame.getState().orders[0]).toEqual([{ type: 'ProposeTreaty', to: 1, kind: 'Alliance' }]);
    expect(replacesText(view, useGame.getState().orders[0], { type: 'DeclareWar', against: 1 })).toBe('Replaces your Alliance offer');
    expect(replacesText(view, useGame.getState().orders[0], { type: 'DeclareWar', against: 2 })).toBeNull();
  });
});
