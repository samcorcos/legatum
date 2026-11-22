// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { RULES } from '../data';
import { FixtureClient } from '../test/fixtureEngine';
import { broadcastState, freeTechsState, retireState, sakokuState, swarmState, wildCalledState, wildState } from '../test/goalActionsFixture';
import { UnitPanel } from './UnitPanel';
import { useGame } from '../store';
import type { MState } from '../test/fixtureState';
import { FreeTechPicker, FreeTechsPrompt } from './FreeTechs';
import { GoalActionDialog, GoalActionRow, useGoalActionKey } from './GoalActions';
import { SidebarFactionGoal } from './FactionGoalProgress';
import { pendingDecisions } from '../turnFlow';
import { goalDecisionText, readyGoalAction, seenGoalsAfterTurn } from '../goalDecision';
import { unitCardLeads } from './turnLead';
import type { PlayerView } from '../types';

let host: HTMLDivElement, root: ReturnType<typeof createRoot>;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  // happy-dom has no modal dialog: open the element the way showModal would.
  HTMLDialogElement.prototype.showModal = function showModal(this: HTMLDialogElement) { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close(this: HTMLDialogElement) { this.removeAttribute('open'); };
  useGame.setState(useGame.getInitialState(), true);
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

function seat(state: MState) {
  const client = new FixtureClient();
  client.load(JSON.stringify(state));
  useGame.setState({ client, views: [client.view(0)], humans: [0], orders: [[]], errors: [[]], seat: 0, ready: [false] });
  return client;
}
const SWARM = RULES.swarmPopulation;
const RETIRE_LEVEL = RULES.spartaRetireLevel;
const render = (ui: React.ReactNode) => act(() => root.render(ui));
const press = (el: Element | null) => act(() => { (el as HTMLElement).click(); });
function GoalKey() { useGoalActionKey(); return null; }

it.each([
  ['Aztecs', swarmState, 'Swarm'],
  ['Japan vow', () => sakokuState(), 'Sakoku'],
  ['Celts near victory', wildState, 'CallTheWild'],
] as const)('%s exposes the action without a disclosure and offers one deferrable decision', (_, fixture, kind) => {
  seat(fixture());
  if (kind === 'CallTheWild') act(() => useGame.setState({ views: [nearVictory(useGame.getState().views![0], 3)] }));
  const view = useGame.getState().views![0];
  render(<SidebarFactionGoal view={view} />);
  expect(host.querySelector('.sidebar-goal')).not.toBeNull();
  expect(host.querySelector('.sidebar-goal details, .sidebar-goal summary')).toBeNull();
  expect((host.querySelector(`[data-goal-action="${kind}"] .goal-action-btn`) as HTMLButtonElement).disabled).toBe(false);
  const decisions = () => { const s = useGame.getState(); return pendingDecisions(s.views![0], s.orders[0], s.errors[0], s); };
  expect(decisions().filter((d) => d.kind === 'goal')).toHaveLength(1);
  act(() => useGame.getState().set({ goalDecisionSeen: { [view.me]: true } }));
  expect(decisions().filter((d) => d.kind === 'goal')).toHaveLength(0);
  expect((host.querySelector(`[data-goal-action="${kind}"] .goal-action-btn`) as HTMLButtonElement).disabled).toBe(false);
});

/** The view with its team `short` VP below the target. */
function nearVictory(view: PlayerView, short: number): PlayerView {
  return { ...view, standings: view.standings.map((t) => t.team === view.team ? { ...t, vp: t.target - short } : t) };
}

it('Raise the Warbands waits in the sidebar until its VP would bring the team within 1 VP of victory', () => {
  seat(wildState());
  const view = useGame.getState().views![0];
  render(<SidebarFactionGoal view={view} />);
  // Far from victory (a fresh start), the turn does not lead with the warbands, but the sidebar button works.
  expect(readyGoalAction(nearVictory(view, 10), [], {})).toBeNull();
  expect(readyGoalAction(nearVictory(view, 4), [], {})).toBeNull();
  expect((host.querySelector('[data-goal-action="CallTheWild"] .goal-action-btn') as HTMLButtonElement).disabled).toBe(false);
  // Its 2 VP leave the team 1 short, or reach the target: now it is offered.
  expect(readyGoalAction(nearVictory(view, 3), [], {})).toMatchObject({ kind: 'CallTheWild', vp: 2 });
  expect(readyGoalAction(nearVictory(view, 2), [], {})).toMatchObject({ kind: 'CallTheWild' });
  // A turn that did not offer it does not use up the one-time offer.
  expect(readyGoalAction(nearVictory(view, 3), [], seenGoalsAfterTurn([nearVictory(view, 10)], {}))).not.toBeNull();
});

it('Share the Learning waits in the sidebar until its VP would reach the victory target', () => {
  seat(broadcastState());
  const view = useGame.getState().views![0];
  // Turn 1 or any turn far from victory: the dock does not lead with handing rivals free technologies.
  expect(readyGoalAction(nearVictory(view, 10), [], {})).toBeNull();
  expect(readyGoalAction(nearVictory(view, 3), [], {})).toBeNull();
  // Its 2 VP reach the target: now it is offered.
  expect(readyGoalAction(nearVictory(view, 2), [], {})).toMatchObject({ kind: 'Broadcast', vp: 2 });
  expect(readyGoalAction(nearVictory(view, 2), [], seenGoalsAfterTurn([nearVictory(view, 10)], {}))).not.toBeNull();
});

it('a ready goal is one decision: the title is the action, the line is what it scores, and it never repeats itself', () => {
  seat(swarmState());
  const s = useGame.getState();
  const goal = pendingDecisions(s.views![0], [], [], s).find((d) => d.kind === 'goal');
  expect(goal).toMatchObject({ title: 'Kindle the New Fire', line: '+2 VP, kept for good' });
  expect(goal!.label).not.toMatch(/is ready/);
  // A goal that takes several steps counts toward its VP; a one-time act keeps it for good.
  expect(goalDecisionText('Retire', 2).line).toBe('Counts toward +2 VP');
  expect(goalDecisionText('Broadcast', 2)).toEqual({ title: 'Share the Learning', line: '+2 VP, kept for good' });
  // The dock's primary leads even with a unit selected: it is not a routine decision, so no outlined stepped-back button.
  expect(unitCardLeads([goal!], { kind: 'unit', id: 0 })).toBe(false);
});

it('Return opens the ready New Fire, and it never recurs after dismissal on later turns', () => {
  seat(swarmState());
  const s = useGame.getState();
  expect(pendingDecisions(s.views![0], [], [], s).some((d) => d.kind === 'goal')).toBe(true);
  act(() => s.nextDecision(true));
  expect(useGame.getState().goalDialog).toBe('Swarm');
  expect(useGame.getState().goalDecisionSeen[s.views![0].me]).toBe(true);
  const later = { ...s.views![0], turn: s.views![0].turn + 1 };
  expect(pendingDecisions(later, [], [], useGame.getState()).some((d) => d.kind === 'goal')).toBe(false);
});

it('an ignored ready-goal offer expires at the turn boundary, while a newly ready goal still appears', () => {
  seat(swarmState());
  const ready = useGame.getState().views![0];
  expect(readyGoalAction(ready, [], {})).not.toBeNull();
  const seen = seenGoalsAfterTurn([ready], {});
  expect(readyGoalAction({ ...ready, turn: ready.turn + 1 }, [], seen)).toBeNull();

  const blockedState = swarmState();
  for (const c of blockedState.cities.filter((c) => c.owner === 0)) c.population = Math.min(c.population, 8);
  seat(blockedState);
  const blocked = useGame.getState().views![0];
  expect(readyGoalAction(blocked, [], {})).toBeNull();
  expect(readyGoalAction(ready, [], seenGoalsAfterTurn([blocked], {}))).not.toBeNull();
});

it('G opens a ready goal, including Sparta’s veteran confirm; a blocked rite stays closed', () => {
  const key = () => act(() => window.dispatchEvent(new KeyboardEvent('keydown', { key: 'g', bubbles: true })));
  seat(swarmState());
  render(<GoalKey />);
  key();
  expect(useGame.getState().goalDialog).toBe('Swarm');
  act(() => root.render(<div />));
  const blocked = swarmState();
  for (const c of blocked.cities.filter((c) => c.owner === 0)) c.population = Math.min(c.population, 8);
  seat(blocked);
  act(() => useGame.getState().set({ goalDialog: null }));
  render(<GoalKey />);
  key();
  expect(useGame.getState().goalDialog).toBeNull();
  act(() => root.render(<div />));
  seat(retireState());
  render(<GoalKey />);
  key();
  expect(useGame.getState().goalDialog).toBe('RetireVeteran');
  expect(useGame.getState().goalDialogUnit).not.toBeNull();
  expect(useGame.getState().goalDecisionSeen[useGame.getState().views![0].me]).toBe(true);
});

it('a queued Spartan retirement does not ask for the same veteran again', () => {
  seat(retireState());
  const s = useGame.getState();
  const ready = s.views![0].actions!.retire!.veterans.filter((v) => !v.blocker);
  const orders = ready.map((v) => ({ type: 'Action' as const, action: { action: 'RetireVeteran' as const, unit: v.unit } }));
  act(() => s.set({ orders: [orders] }));
  expect(pendingDecisions(s.views![0], orders, [], useGame.getState()).some((d) => d.kind === 'goal')).toBe(false);
  render(<GoalActionRow />);
  expect((host.querySelector('[data-goal-action="Retire"] .goal-action-btn') as HTMLButtonElement).disabled).toBe(true);
  expect(host.querySelector('.goal-action-why')?.textContent).toContain('already have retirement orders');
});

it('an empire whose goal rule is the Swarm sees The New Fire in the race panel, opens its confirm, and can cancel the order', () => {
  seat(swarmState());
  render(<><GoalActionRow /><GoalActionDialog /></>);
  const row = host.querySelector('[data-goal-action="Swarm"]')!;
  expect(row).not.toBeNull();
  const button = row.querySelector('button.goal-action-btn') as HTMLButtonElement;
  expect(button.textContent).toContain('Kindle the New Fire');
  expect(button.disabled).toBe(false);
  press(button);
  expect(useGame.getState().goalDialog).toBe('Swarm');
  const dialog = host.querySelector('dialog[data-goal-dialog="Swarm"]')!;
  expect(dialog.hasAttribute('open')).toBe(true);
  expect(dialog.querySelector('.new-fire-torches')!.children).toHaveLength(SWARM);
  // Each city's loss is named, and the cost is in the spend colour.
  const rows = [...dialog.querySelectorAll('tbody tr')].map((r) => r.textContent);
  expect(rows).toHaveLength(2); // the sacrifice comes off the biggest cities first: Cell and Seed give nothing
  expect(rows[0]).toContain('Hive');
  expect(dialog.querySelector('.cost-chip')!.textContent).toContain(`−${SWARM} population58 → ${58 - SWARM}`);
  press([...dialog.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Kindle the New Fire') ?? null);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'Action', action: { action: 'Swarm' } }]);
  expect(useGame.getState().goalDialog).toBeNull();
  expect(host.querySelector('[data-goal-action]')!.textContent).toContain('Ordered');
  press([...host.querySelectorAll('[data-goal-action] button')].find((b) => b.textContent === 'Cancel the order') ?? null);
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('shows a disabled New Fire button and exact population still needed', () => {
  const s = swarmState();
  for (const c of s.cities.filter((c) => c.owner === 0)) c.population = Math.min(c.population, 8);
  seat(s);
  render(<GoalActionRow />);
  const row = host.querySelector('[data-goal-action="Swarm"]')!;
  expect((row.querySelector('button.goal-action-btn') as HTMLButtonElement).disabled).toBe(true);
  expect(row.querySelector('button.goal-action-btn')?.textContent).toContain('Kindle the New Fire');
  expect(row.querySelector('.goal-action-why')!.textContent).toBe(`21 of ${SWARM} population to spare: every city keeps 1.`);
});

it('Athens see Share the Learning; nobody else sees an action row', () => {
  seat(broadcastState());
  render(<GoalActionRow />);
  expect(host.querySelector('[data-goal-action="Broadcast"] button')!.textContent).toContain('Share the Learning');
  act(() => root.render(<div />));
  seat(freeTechsState());
  render(<GoalActionRow />);
  // The Celts's own action is Raise the Warbands (below); nobody else's goal has a row without an action.
  expect(host.querySelector('[data-goal-action="Broadcast"]')).toBeNull();
  expect(host.querySelector('[data-goal-action="Swarm"]')).toBeNull();
});

it('a received broadcast: the prompt comes back each turn, the picker lights eligible techs and orders the choice', () => {
  seat(freeTechsState());
  render(<><FreeTechsPrompt /><FreeTechPicker /></>);
  const prompt = host.querySelector('[data-testid="free-techs-prompt"]')!;
  expect(prompt.textContent).toContain('Choose 2 technologies');
  // Dismissed for the turn: it stays out of the way until the next turn, and the decision remains.
  act(() => useGame.getState().set({ freeTechsPrompted: { 0: useGame.getState().views![0].turn } }));
  expect(host.querySelector('[data-testid="free-techs-prompt"]')).toBeNull();
  act(() => useGame.getState().set({ freeTechsOpen: true }));
  const picker = host.querySelector('[data-testid="free-tech-picker"]')!;
  expect(picker.hasAttribute('open')).toBe(true);
  const take = [...picker.querySelectorAll('footer button')].find((b) => b.textContent?.startsWith('Take 2')) as HTMLButtonElement;
  expect(take.disabled).toBe(true);
  expect(picker.querySelector('.ftp-why')!.textContent).toBe('Choose 2 more');
  const open = () => [...picker.querySelectorAll<HTMLButtonElement>('.ftp-tech.open')];
  // Free picks come only from the current era: two of its five are unknown, and the later eras are not listed.
  expect(picker.querySelectorAll('.ftp-era')).toHaveLength(1);
  expect(open()).toHaveLength(2);
  press(open()[0]);
  press(open()[0]);
  expect(picker.querySelectorAll('.ftp-tech.chosen')).toHaveLength(2);
  expect(take.disabled).toBe(false);
  press(take);
  const orders = useGame.getState().orders[0];
  expect(orders).toHaveLength(1);
  expect(orders[0]).toMatchObject({ type: 'Action', action: { action: 'ChooseFreeTechs' } });
  expect(orders[0].type === 'Action' && orders[0].action.action === 'ChooseFreeTechs' && orders[0].action.techs).toHaveLength(2);
  expect(useGame.getState().freeTechsOpen).toBe(false);
});

it('the Celts see Raise the Warbands: the confirm names each camp, the order is undoable, and the row counts the camps', () => {
  seat(wildState());
  render(<><GoalActionRow /><GoalActionDialog /></>);
  const row = host.querySelector('[data-goal-action="CallTheWild"]')!;
  expect(row).not.toBeNull();
  expect(row.querySelector('[data-testid="wild-progress"]')!.textContent).toBe('0 of 3 cities have a wild camp in their territory.');
  const button = row.querySelector('button.goal-action-btn') as HTMLButtonElement;
  expect(button.textContent).toContain('Raise the Warbands');
  press(button);
  expect(useGame.getState().goalDialog).toBe('CallTheWild');
  const dialog = host.querySelector('dialog[data-goal-dialog="CallTheWild"]')!;
  expect(dialog.hasAttribute('open')).toBe(true);
  const rows = [...dialog.querySelectorAll('tbody tr')].map((r) => r.textContent);
  expect(rows).toHaveLength(3);
  expect(rows[0]).toMatch(/Greenhold\+1 camp\s?\d tiles? out/);
  expect(dialog.textContent).toContain('+2 VP');
  expect(dialog.textContent).toContain('no Gold for clearing the camps you raised');
  press([...dialog.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Raise the Warbands') ?? null);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'Action', action: { action: 'CallTheWild' } }]);
  expect(host.querySelector('[data-goal-action]')!.textContent).toContain('Ordered');
  press([...host.querySelectorAll('[data-goal-action] button')].find((b) => b.textContent === 'Cancel the order') ?? null);
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('after the call the row reports full coverage and offers no redundant camp order', () => {
  seat(wildCalledState());
  render(<GoalActionRow />);
  const row = host.querySelector('[data-goal-action="CallTheWild"]')!;
  expect(row.querySelector('[data-testid="wild-progress"]')!.textContent).toBe('3 of 3 cities have a wild camp in their territory.');
  expect((row.querySelector('button.goal-action-btn') as HTMLButtonElement).disabled).toBe(true);
  expect(row.textContent).toContain('No city needs a wild camp');
});

/** Select an Sparta unit by id and render its panel. */
function panelOf(state: MState, pick: (units: ReturnType<FixtureClient['view']>['units']) => number) {
  const client = seat(state);
  const view = client.view(0);
  const id = pick(view.units.filter((u) => u.owner === view.me));
  render(<><UnitPanel unit={view.units.find((u) => u.id === id)!} view={view} /><GoalActionDialog /></>);
  return id;
}
const retireButton = () => [...host.querySelectorAll('.unit-panel .actions button')].find((b) => b.textContent?.includes('Retire this veteran')) as HTMLButtonElement | undefined;

it('a veteran in an Agoge city is offered retirement, the confirm states the progress, and the order is undoable', () => {
  const id = panelOf(retireState(1), (us) => us.find((u) => (u.level ?? 0) >= RETIRE_LEVEL && u.pos.q === us.find((x) => (x.level ?? 0) >= 6)!.pos.q)!.id);
  const button = retireButton()!;
  expect(button).toBeTruthy();
  expect(button.disabled).toBe(false);
  press(button);
  expect(useGame.getState().goalDialog).toBe('RetireVeteran');
  expect(useGame.getState().goalDialogUnit).toBe(id);
  const dialog = host.querySelector('dialog[data-goal-dialog="RetireVeteran"]')!;
  expect(dialog.hasAttribute('open')).toBe(true);
  expect(dialog.querySelector('[data-testid="retire-progress-line"]')!.textContent).toBe('2 of 3');
  expect(dialog.textContent).toContain('Honour the Veterans (+2 VP)');
  press([...dialog.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Retire this veteran') ?? null);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'Action', action: { action: 'RetireVeteran', unit: id } }]);
  expect(useGame.getState().goalDialog).toBeNull();
  expect(host.querySelector('.up-status')!.textContent).toContain('Retiring with honour');
  press(retireButton()!);
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('a veteran away from an Agoge says where he can retire, and a green soldier is offered nothing', () => {
  panelOf(retireState(), (us) => us.find((u) => (u.level ?? 0) >= RETIRE_LEVEL && u.pos.q !== us.find((x) => (x.level ?? 0) >= 6)!.pos.q)!.id);
  const away = retireButton()!;
  expect(away.disabled).toBe(true);
  expect([...host.querySelectorAll('.unit-panel .act-reason')].map((r) => r.textContent).join(' ')).toContain('Stonewatch has no Agoge');
  act(() => root.render(<div />));
  panelOf(retireState(), (us) => us.find((u) => (u.level ?? 0) < RETIRE_LEVEL)!.id);
  expect(retireButton()).toBeUndefined();
});

it('Sparta row shows progress and a direct route to a veteran', () => {
  seat(retireState(1));
  render(<GoalActionRow />);
  const row = host.querySelector('[data-goal-action="Retire"]')!;
  expect(row).not.toBeNull();
  expect((row.querySelector('button.goal-action-btn') as HTMLButtonElement).disabled).toBe(false);
  expect(row.querySelector('[data-testid="retire-progress"]')!.textContent).toBe(`1 of 3 veterans retired · 1 Agoge city · 4 veterans at level ${RETIRE_LEVEL}+, 3 in place`);
  act(() => root.render(<div />));
  seat(retireState(3));
  render(<GoalActionRow />);
  expect(host.querySelector('[data-goal-action]')).toBeNull();
});

it('a legacy Sakoku scenario closes the country from the race panel and explains the costs', () => {
  seat(sakokuState());
  render(<><GoalActionRow /><GoalActionDialog /></>);
  const row = host.querySelector('[data-goal-action="Sakoku"]')!;
  expect(row).not.toBeNull();
  expect(row.querySelector('[data-testid="sakoku-progress"]')!.textContent).toContain('Not closed yet · 0 of 30 turns');
  const button = row.querySelector('button.goal-action-btn') as HTMLButtonElement;
  expect(button.textContent).toContain('Close the country');
  press(button);
  expect(useGame.getState().goalDialog).toBe('Sakoku');
  const dialog = host.querySelector('dialog[data-goal-dialog="Sakoku"]')!;
  expect(dialog.hasAttribute('open')).toBe(true);
  const blocks = [...dialog.querySelectorAll('[data-testid="sakoku-blocks"] tbody th')].map((th) => th.textContent);
  expect(blocks).toEqual(['War', 'Alliances', 'Tech diffusion', 'Foreign units']);
  expect(dialog.querySelector('.for-you-list')!.textContent).toContain('after 30 turns closed');
  press([...dialog.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Close the country') ?? null);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'Action', action: { action: 'Sakoku', close: true } }]);
  expect(useGame.getState().goalDialog).toBeNull();
  expect(host.querySelector('[data-goal-action]')!.textContent).toContain('Ordered');
  press([...host.querySelectorAll('[data-goal-action] button')].find((b) => b.textContent === 'Cancel the order') ?? null);
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('a closed India sees its count and reopens after a confirm that says what is lost', () => {
  seat(sakokuState(12));
  render(<><GoalActionRow /><GoalActionDialog /></>);
  const row = host.querySelector('[data-goal-action="Sakoku"]')!;
  expect(row.querySelector('[data-testid="sakoku-progress"]')!.textContent).toContain('12 of 30 turns closed');
  expect(row.querySelector('[role="progressbar"]')!.getAttribute('aria-valuenow')).toBe('12');
  const button = row.querySelector('button.goal-action-btn') as HTMLButtonElement;
  expect(button.textContent).toContain('Reopen the country');
  press(button);
  expect(useGame.getState().goalDialog).toBe('SakokuReopen');
  const dialog = host.querySelector('dialog[data-goal-dialog="SakokuReopen"]')!;
  expect(dialog.textContent).toContain('closing again starts the count at zero');
  press([...dialog.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Reopen the country') ?? null);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'Action', action: { action: 'Sakoku', close: false } }]);
});

it('a legacy Sakoku scenario holds after the countdown even when enemy units stand in its land', () => {
  seat(sakokuState(40, true));
  render(<GoalActionRow />);
  const row = host.querySelector('[data-goal-action="Sakoku"]')!;
  expect(row.querySelector('[data-testid="sakoku-progress"]')!.textContent).toContain('The country has stayed closed: holding');
  expect(row.querySelector('[data-testid="sakoku-progress"]')!.textContent).not.toContain('enemy');
});

it('India in an Alliance sees a disabled button and is told to leave it first', () => {
  const s = sakokuState();
  for (const r of s.diplomacy?.relations ?? []) if (r.a === 0 && r.b === 1) { r.state = 'Alliance'; r.security = { id: 1, started_turn: 1, protected_until: 11 } as never; }
  seat(s);
  render(<GoalActionRow />);
  const row = host.querySelector('[data-goal-action="Sakoku"]')!;
  expect((row.querySelector('button.goal-action-btn') as HTMLButtonElement).disabled).toBe(true);
  expect(row.querySelector('.goal-action-why')!.textContent).toContain('Leave your Alliance with');
});
