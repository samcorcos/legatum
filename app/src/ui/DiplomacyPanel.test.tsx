// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { meetAll } from '../test/fixtureEngine';
import { useGame } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { DeclareWarPrompt } from './DeclareWarPrompt';
import { SeatBar } from './SeatBar';
import { VpTrack } from './VpTrack';
import { TurnReport } from './TurnReport';
import { contextFromViews, formatEvents } from '../format';
import type { DiplomaticAction, Event, PlayerView } from '../types';
import { FACTION_INFO } from '../data';
import { playerColor } from '../theme';
import { ALLIANCE_SHARES, allianceTerms } from '../treaties';
import { empireTitle } from '../empires';

let container: HTMLDivElement, root: Root;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('700040', 'India', 'Byzantium', 'Ada', 'Bo'));
  meetAll(client.debugState());
  client.debugState().diplomacy!.proposals = [
    { id: 1, kind: 'Alliance', from: 1, to: 0, proposer: 2, created_turn: 1, respond_by: 3, relation_version: 0 },
    { id: 2, kind: 'Alliance', from: 0, to: 2, proposer: 0, created_turn: 1, respond_by: 3, relation_version: 0 },
  ];
  const view = client.view(0);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, diplomacyOpen: true });
});
afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });
const button = (prefix: string) => [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent?.startsWith(prefix))!;
const labelled = (label: string) => [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.getAttribute('aria-label') === label);

it('shows the recorded AI level in diplomacy and the victory race', () => {
  act(() => root.render(<><VpTrack /><DiplomacyPanel /></>));
  expect([...container.querySelectorAll('.empire-pick-level')].map((e) => e.textContent)).toContain('AI · Normal');
  const standings = container.querySelector<HTMLDetailsElement>('.sidebar-standings')!;
  act(() => { standings.open = true; });
  expect([...container.querySelectorAll('.vp-ai-level')].map((e) => e.textContent)).toContain('AI · Normal');
});

/** Our view with 120 Gold and +7 net Gold per turn, and the Alliance upkeep. */
function priced(accept: Partial<DiplomaticAction> = {}): PlayerView {
  const view = structuredClone(useGame.getState().views![0]);
  view.economy = { ...view.economy, gold: 120, gold_per_turn: { total: 7, parts: [['Cities', 9], ['Unit upkeep', -2]] } };
  for (const a of view.diplomacy.actions) if (a.order.type === 'RespondToProposal' && a.order.accept) Object.assign(a, { cost: 0, upkeep: 1, blocked_reason: null }, accept);
  return view;
}

type Priced = Pick<DiplomaticAction, 'order' | 'cost'> & Partial<DiplomaticAction>;
/** We are allied with team 1 (no offers pending); its actions are exactly `actions`. */
function allied(protectedUntil: number, actions: Priced[]): PlayerView {
  const view = priced();
  view.diplomacy.proposals = [];
  view.diplomacy.relations = view.diplomacy.relations.map((r) => (r.a === 0 && r.b === 1) || (r.a === 1 && r.b === 0)
    ? { ...r, state: 'Alliance', guarantee_until: 0, security: { id: 1, started_turn: view.turn, protected_until: protectedUntil } } : r);
  const withTeam1 = (o: DiplomaticAction['order']) => ('against' in o && o.against === 1) || ('with' in o && o.with === 1) || ('to' in o && o.to === 1) || ('ally' in o && o.ally === 1);
  view.diplomacy.actions = [
    ...view.diplomacy.actions.filter((a) => !('proposal' in a.order) && !withTeam1(a.order)),
    ...actions.map((a): DiplomaticAction => ({ effective_turn: view.turn + 1, blocked_reason: null, ...a })),
  ];
  return view;
}

it('counts only incoming offers, prioritizes responding, and makes Accept primary', () => {
  act(() => root.render(<><VpTrack /><SeatBar /><DiplomacyPanel /></>));
  expect(button('Diplomacy').textContent).toBe('Diplomacy (1)');
  expect(button('Respond to offer')).toBeTruthy();
  expect(button('Accept Alliance').classList.contains('primary')).toBe(true);
  expect(button('Decline').classList.contains('primary')).toBe(false);
  act(() => button('Accept Alliance').click());
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'RespondToProposal', proposal: 1, accept: true });
  expect(button('Respond to offer')).toBeUndefined();
  // The badge counts offers still awaiting a response, like the primary action.
  expect(button('Diplomacy').textContent).toBe('Diplomacy');
  // G17: answering the last offer closes the inbox; a notice confirms it, with Undo.
  expect(useGame.getState().diplomacyOpen).toBe(false);
  expect(useGame.getState().notice?.text).toMatch(/^Alliance from .+ accepted$/);
  // Reopened, the card shows the answer as its state; the other choice is no longer a primary.
  act(() => useGame.setState({ diplomacyOpen: true }));
  const card = container.querySelector('.offer-card')!;
  expect(card.querySelector('.offer-answer')!.textContent).toMatch(/^\s*Accepted: Alliance starts turn \d+$/);
  expect(card.querySelector('.offer-answer .mask-icon')).not.toBeNull();
  expect(card.querySelector('.btn.primary')).toBeNull();
  expect([...card.querySelectorAll('.offer-answered button')].map((b) => b.getAttribute('aria-label') ?? b.textContent)).toEqual(['Undo', 'Decline instead']);
  act(() => button('Undo').click());
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(button('Accept Alliance').classList.contains('primary')).toBe(true);
});

it('Decline closes the inbox with a notice; Undo from the notice restores the offer and reopens the inbox (G17)', async () => {
  const { undoNotice } = await import('../notice');
  act(() => root.render(<><SeatBar /><DiplomacyPanel /></>));
  act(() => button('Decline').click());
  expect(useGame.getState().diplomacyOpen).toBe(false);
  expect(useGame.getState().notice?.text).toMatch(/^Alliance from .+ declined$/);
  expect(button('Respond to offer')).toBeUndefined();
  act(() => undoNotice());
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(useGame.getState()).toMatchObject({ notice: null, diplomacyOpen: true, diplomacyTeam: 1 });
  expect(button('Respond to offer')).toBeTruthy();
  // Any other order edit ends the chance to undo from the notice (the card and dock still can).
  act(() => button('Decline').click());
  act(() => useGame.getState().editOrders((o) => [...o, { type: 'SkipUnit', unit: 999 }]));
  expect(useGame.getState().notice).toBeNull();
});

it('A accepts and D declines the offer awaiting a reply; Z undoes while the notice shows (G19)', async () => {
  const { DecisionNotice } = await import('./DecisionNotice');
  act(() => root.render(<><DiplomacyPanel /><DecisionNotice /></>));
  const card = container.querySelector('.offer-card')!;
  expect([...card.querySelectorAll('.offer-actions .kbd')].map((k) => k.textContent)).toEqual(['A', 'D']);
  const press = (key: string, init: KeyboardEventInit = {}, target: Element = document.body) => {
    const e = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...init });
    act(() => { target.dispatchEvent(e); });
    return e;
  };
  // Modified or typed letters are not answers.
  expect(press('d', { ctrlKey: true }).defaultPrevented).toBe(false);
  const input = container.appendChild(document.createElement('input'));
  expect(press('d', {}, input).defaultPrevented).toBe(false);
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(press('d').defaultPrevented).toBe(true);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'RespondToProposal', proposal: 1, accept: false }]);
  expect(useGame.getState().diplomacyOpen).toBe(false);
  expect(container.querySelector('.decision-notice')!.textContent).toMatch(/declined/);
  // The inbox is closed: A does nothing now.
  expect(press('a').defaultPrevented).toBe(false);
  expect(press('z').defaultPrevented).toBe(true);
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(useGame.getState().diplomacyOpen).toBe(true);
  expect(press('a').defaultPrevented).toBe(true);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'RespondToProposal', proposal: 1, accept: true }]);
  expect(press('z', { metaKey: true }).defaultPrevented).toBe(true);
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('answering one of two offers goes straight to the next one', () => {
  const client = useGame.getState().client as FixtureClient;
  client.debugState().diplomacy!.proposals.push({ id: 3, kind: 'Peace', from: 2, to: 0, proposer: 3, created_turn: 1, respond_by: 3, relation_version: 0 });
  useGame.setState({ views: [client.view(0)] });
  act(() => root.render(<DiplomacyPanel />));
  expect(container.querySelectorAll('.offer-card')).toHaveLength(2);
  act(() => ([...container.querySelectorAll<HTMLButtonElement>('.offer-card button')].find((b) => b.getAttribute('aria-label') === 'Decline'))!.click());
  expect(useGame.getState().diplomacyOpen).toBe(true);
  expect(useGame.getState().diplomacyTeam).toBe(2);
  expect(container.querySelectorAll('.offer-answered')).toHaveLength(1);
});

it('opens the inbox from a received-offer report line', () => {
  const view = useGame.getState().views![0];
  const event: Event = { type: 'ProposalCreated', proposal: view.diplomacy.proposals[0] };
  useGame.setState({ diplomacyOpen: false, reportOpen: true, report: { turn: 1, hash: '1', events: [[event]], lines: [formatEvents([event], contextFromViews(null, view))] } });
  act(() => root.render(<TurnReport />));
  expect(container.textContent).toContain('arrived');
  act(() => button('Open Diplomacy').click());
  expect(useGame.getState()).toMatchObject({ diplomacyOpen: true, diplomacyTeam: 1, reportOpen: false });
});

it('an Alliance offer shows its upkeep as the cost, without a net income line', () => {
  useGame.setState({ views: [priced()] });
  act(() => root.render(<DiplomacyPanel />));
  const card = container.querySelector('.offer-card')!;
  const lines = [...card.querySelectorAll('.offer-for-you li')];
  expect(lines.slice(0, 1).map((li) => li.textContent)).toEqual(['Cost: Upkeep 1 Gold/turn']);
  expect(container.textContent).not.toContain('Net income');
  expect(lines[0].classList.contains('cost')).toBe(true);
  expect(card.textContent).not.toContain('Each empire pays 1 Gold a turn for each city it owns');
  expect(card.textContent).toContain(`Leaving within ${allianceTerms().protectedTurns} turns of signing costs`);
  // The itemised lines carry the cost once; both decision buttons stay short.
  expect(labelled('Accept Alliance')!.querySelector('.cost-chip')).toBeNull();
  expect(labelled('Decline')!.querySelector('.cost-chip')).toBeNull();
});

it('an active Alliance shows its protected window; leaving early and war on the ally are priced betrayals', () => {
  const view = allied(12, [
    { order: { type: 'BreakTreaty', with: 1, agreement: 'Security' }, cost: 60, betrayal: true },
    { order: { type: 'DeclareWar', against: 1, breach: true }, cost: 60, betrayal: true },
    { order: { type: 'DeclareWar', against: 1, breach: false }, cost: 0, blocked_reason: 'You are allied with Bo: leave the Alliance first, or declare war on your ally as a betrayal' },
  ]);
  useGame.setState({ views: [view], diplomacyTeam: 1 });
  act(() => root.render(<DiplomacyPanel />));
  const row = container.querySelector('.empire-row.open')!;
  const them = empireTitle(view, 1);
  expect(row.querySelector('.empire-relation')!.textContent).toBe('Alliance');
  expect(row.textContent).toContain('Protected until turn 12');
  const leave = labelled('Leave the Alliance')!;
  expect(leave.querySelector('.cost-chip')!.textContent).toContain('−60');
  expect(labelled('Declare War on your ally')!.querySelector('.cost-chip')!.textContent).toContain('−60');
  expect(labelled('Declare War')).toBeUndefined();
  const betrayal = `Betrayal: ${them} won't ally with you for ${allianceTerms().betrayalTurns} turns`;
  expect([...row.querySelectorAll('.for-you-line.cost')].filter((li) => li.textContent?.includes(betrayal))).toHaveLength(2);
  act(() => leave.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'BreakTreaty', with: 1, agreement: 'Security' }]);
});

it('after the protected window, leaving is free and says so plainly', () => {
  useGame.setState({ views: [allied(1, [{ order: { type: 'BreakTreaty', with: 1, agreement: 'Security' }, cost: 0 }])], diplomacyTeam: 1 });
  act(() => root.render(<DiplomacyPanel />));
  const row = container.querySelector('.empire-row.open')!;
  expect(labelled('Leave the Alliance')!.querySelector('.cost-chip')).toBeNull();
  expect(row.textContent).toContain(`Free: the ${allianceTerms().protectedTurns}-turn protected window has passed`);
  expect(row.textContent).not.toContain('Betrayal');
  expect(row.textContent).toContain('leaving is free');
});

it('a rival row shows its public betrayals, and an Alliance offer it refuses says why; no trust numbers', () => {
  const view = priced();
  view.diplomacy.betrayals = [{ by: 1, against: 2, turn: 52, until: 72 }];
  const refused = 'Switzerland won’t ally with Ada until turn 72: betrayed on turn 52';
  view.diplomacy.proposals = view.diplomacy.proposals.filter((p) => p.to !== 2 && p.from !== 2);
  for (const a of view.diplomacy.actions) if (a.order.type === 'ProposeTreaty' && a.order.to === 2 && a.order.kind === 'Alliance') a.blocked_reason = refused;
  useGame.setState({ views: [view], diplomacyTeam: 2 });
  act(() => root.render(<DiplomacyPanel />));
  const rows = [...container.querySelectorAll('.empire-row')];
  const betrayer = rows.find((r) => r.querySelector('.empire-pick-name')!.textContent === empireTitle(view, 1))!;
  expect(betrayer.querySelector('.betrayal-line')!.textContent!.trim()).toBe(`Broke an alliance on turn 52: ${empireTitle(view, 2)} won't ally until turn 72`);
  expect(container.querySelectorAll('.betrayal-line')).toHaveLength(1);
  expect(container.querySelector('.empire-row.open .diplomacy-unavailable')!.textContent).toContain(`Offer Alliance. ${refused}`);
  expect(container.textContent!.toLowerCase()).not.toMatch(/\btrust\b|\bnotice\b|\bpacts?\b/);
});

it('an Alliance offer from an empire row shows upkeep and net income before you send it', () => {
  const view = priced();
  view.diplomacy.proposals = [];
  for (const a of view.diplomacy.actions) if (a.order.type === 'ProposeTreaty' && a.order.kind === 'Alliance') Object.assign(a, { cost: 0, upkeep: 1, blocked_reason: null });
  useGame.setState({ views: [view], diplomacyTeam: 1 });
  act(() => root.render(<DiplomacyPanel />));
  const row = container.querySelector('.empire-row.open')!;
  const offer = labelled('Offer Alliance')!;
  expect(offer.querySelector('.cost-chip')).toBeNull();
  expect([...row.querySelectorAll('.diplomacy-action-lines .cost')].map((li) => li.textContent)).toEqual(['Cost: Upkeep 1 Gold/turn']);
});

it('opened for one empire, scrolls to its row and actions unless an offer awaits a reply', () => {
  const scroll = vi.fn();
  const original = window.Element.prototype.scrollIntoView;
  window.Element.prototype.scrollIntoView = scroll;
  try {
    useGame.setState({ diplomacyTeam: 1 });
    act(() => root.render(<DiplomacyPanel />));
    // The inbox holds an offer to us: the reply is the decision, so the panel stays at the inbox.
    expect(scroll).not.toHaveBeenCalled();
    act(() => root.unmount());
    root = createRoot(container);
    const view = structuredClone(useGame.getState().views![0]);
    view.diplomacy.proposals = view.diplomacy.proposals.filter((p) => p.to !== view.team);
    useGame.setState({ views: [view], diplomacyOpen: true, diplomacyTeam: 2 });
    act(() => root.render(<DiplomacyPanel />));
    expect(scroll).toHaveBeenCalledTimes(1);
    // The empire's own row (its relation and actions), not the list heading above six empires.
    const row = scroll.mock.contexts[0] as HTMLElement;
    expect(row.classList.contains('empire-row') && row.classList.contains('open')).toBe(true);
    expect(row.querySelector('.empire-detail')).not.toBeNull();
  } finally { window.Element.prototype.scrollIntoView = original; }
});

it('turns an incoming offer into one decision card: who, what it means, what it costs, Accept or Decline', () => {
  const view = priced();
  useGame.setState({ views: [view] });
  const from = view.players.find((p) => p.team === 1)!;
  act(() => root.render(<DiplomacyPanel />));
  const card = container.querySelector<HTMLElement>('.offer-card')!;
  const headline = card.querySelector('.offer-headline')!;
  expect(headline.querySelector(':scope > span:last-child')!.textContent).toBe(`${empireTitle(view, from.team)} offers an Alliance`);
  // The swatch is the empire's crest and seat number in its map color.
  expect(headline.querySelector<HTMLElement>('.empire-mark .player-badge')!.style.background).toBe(playerColor(from.id));
  expect(card.querySelector('.empire-where')!.textContent).toMatch(/^(Nearest city|City not seen yet|Location unknown)/);
  expect(card.querySelector('.offer-meaning')!.textContent).toBe(ALLIANCE_SHARES);
  // The proposer is a face: their leader's portrait, the treaty's emblem on it, and their motto as the quote.
  expect(card.querySelector('.offer-portrait img')).not.toBeNull();
  expect(card.querySelector('.offer-portrait .te.alliance')).not.toBeNull();
  expect(card.querySelector('.offer-quote')!.textContent).toBe(`“${FACTION_INFO[from.faction].motto}”`);
  // Upkeep and net income are red-ringed gold medallions; the gains are green rings.
  expect(card.querySelectorAll('.offer-for-you .for-you-line.cost .rm.bad').length).toBeGreaterThanOrEqual(1);
  expect(card.querySelector('.offer-for-you .for-you-line.cost .rm.bad .rm-sign')).not.toBeNull();
  expect(card.querySelectorAll('.offer-for-you .for-you-line.gain .rm.good').length).toBeGreaterThanOrEqual(1);
  // G16: costs are red − lines (Gold first), gains green + lines.
  const lines = [...card.querySelectorAll('.offer-for-you li')];
  expect(lines[0].classList.contains('cost')).toBe(true);
  expect(lines[0].textContent).toBe('Cost: Upkeep 1 Gold/turn');
  expect(lines.at(-1)!.classList.contains('gain')).toBe(true);
  expect([...card.querySelectorAll('.offer-actions button')].map((b) => b.getAttribute('aria-label'))).toEqual(['Accept Alliance', 'Decline']);
  expect(card.querySelectorAll('.offer-actions .cost-chip')).toHaveLength(0);
  expect(card.querySelector('.offer-deadline')!.textContent).toBe('Respond by turn 3');
  // Every rule stays reachable, collapsed.
  const rules = card.querySelector<HTMLDetailsElement>('details.rules-details')!;
  expect(rules.open).toBe(false);
  expect(rules.textContent).toContain('A betrayed team won’t ally with its betrayer');
  expect(rules.textContent).toContain('Teammates share diplomacy');
  expect(container.querySelector<HTMLDetailsElement>('details.diplomacy-rules')!.open).toBe(false);
});

it('lists empires with mark, faction name, relation and VP, and collapses unusable actions to one line', () => {
  useGame.setState({ diplomacyTeam: 1 });
  act(() => root.render(<DiplomacyPanel />));
  const rows = [...container.querySelectorAll('.empire-row')];
  expect(rows.length).toBeGreaterThan(1);
  const open = container.querySelector('.empire-row.open')!;
  const view = useGame.getState().views![0];
  expect(open.querySelector('.empire-pick-name')!.textContent).toBe(empireTitle(view, 1));
  expect(open.querySelector('.empire-relation')!.textContent).toBe('Peace');
  expect(open.querySelector('.empire-vp')!.textContent).toMatch(/^\d+\/\d+ VP$/);
  // One sentence says what Peace means on its row (the generic definition is not repeated above it).
  expect(open.querySelector('.treaty-definition')).toBeNull();
  expect(open.textContent).toContain('No combat or captures; closed borders.');
  // Team 1 has an outstanding proposal to us: its offers are one muted line, not disabled buttons.
  expect([...open.querySelectorAll('button')].some((b) => b.textContent?.startsWith('Offer'))).toBe(false);
  expect(open.querySelector('.diplomacy-unavailable')!.textContent).toMatch(/^Not now: Offer Alliance.*outstanding proposal/);
  act(() => (rows.find((r) => r !== open)!.querySelector('.empire-pick') as HTMLButtonElement).click());
  expect(useGame.getState().diplomacyTeam).not.toBe(1);
});

it('shows a Peace truce and the disabled Declare War reason on its empire row', () => {
  const view = structuredClone(useGame.getState().views![0]);
  const through = view.turn + 9;
  const relation = view.diplomacy.relations.find((r) => (r.a === 0 && r.b === 1) || (r.a === 1 && r.b === 0))!;
  relation.state = 'Peace';
  relation.guarantee_until = through + 1;
  view.diplomacy.actions = [
    ...view.diplomacy.actions.filter((a) => !(a.order.type === 'DeclareWar' && a.order.against === 1)),
    { order: { type: 'DeclareWar', against: 1, breach: false }, cost: 0,
      effective_turn: through + 1, blocked_reason: `Peace with ${empireTitle(view, 1)} is guaranteed through turn ${through}` },
  ];
  useGame.setState({ views: [view], diplomacyTeam: 1 });
  act(() => root.render(<DiplomacyPanel />));
  const row = container.querySelector('.empire-row.open')!;
  expect(row.querySelector('.empire-relation')!.textContent).toContain(`truce through turn ${through}`);
  const war = [...row.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Declare War') as HTMLButtonElement;
  expect(war.disabled).toBe(true);
  expect(war.parentElement!.textContent).toContain(`guaranteed through turn ${through}`);
});

it('keeps Declare War disabled with the truce end turn in the attack dialog', () => {
  const view = structuredClone(useGame.getState().views![0]);
  const through = view.turn + 9;
  view.diplomacy.actions = view.diplomacy.actions.filter((a) => !(a.order.type === 'DeclareWar' && a.order.against === 1));
  view.diplomacy.actions.push({ order: { type: 'DeclareWar', against: 1, breach: false }, cost: 0,
    effective_turn: through + 1, blocked_reason: `Peace with ${empireTitle(view, 1)} is guaranteed through turn ${through}` });
  useGame.setState({ views: [view], warPrompt: { seat: 0, view, unit: null, target: null, team: 1 } });
  act(() => root.render(<DeclareWarPrompt />));
  const dialog = container.querySelector('dialog')!;
  const war = [...dialog.querySelectorAll('button')].find((b) => b.textContent === 'Declare War') as HTMLButtonElement;
  expect(war.disabled).toBe(true);
  expect(war.parentElement!.textContent).toContain(`guaranteed through turn ${through}`);
});

it('shows an empire on the map from its known location', () => {
  const view = structuredClone(useGame.getState().views![0]);
  const their = view.players.find((p) => p.team === 1)!;
  // Isolate a known home city and a remembered rival city six tiles east.
  const settler = view.units.find((u) => u.owner === view.me)!;
  const mine = { id: 998, owner: view.me, name: 'Astra', pos: settler.pos, population: 1, hp: 100, max_hp: 100, strength: 10, is_capital: true, walls: false, great_works: [], visible: true, detail: null };
  view.cities = [mine, { ...mine, id: 999, owner: their.id, name: 'Harmonia', pos: { q: mine.pos.q + 6, r: mine.pos.r }, visible: false }];
  useGame.setState({ views: [view], diplomacyTeam: 1 });
  act(() => root.render(<DiplomacyPanel />));
  const where = container.querySelector('.offer-card .empire-where')!;
  expect(where.textContent).toContain(`Nearest city Harmonia ~6 tiles E of ${mine.name}`);
  act(() => (where.querySelector('button') as HTMLButtonElement).click());
  expect(useGame.getState().focus?.hex).toEqual({ q: mine.pos.q + 6, r: mine.pos.r });
  expect(useGame.getState().diplomacyOpen).toBe(false);
});
