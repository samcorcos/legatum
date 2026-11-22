// @vitest-environment happy-dom
import { act, createElement, type ReactNode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TechPanel } from './TechPanel';
import { TopBar } from './TopBar';
import { useGame } from '../store';
import { FixtureClient } from '../test/fixtureEngine';
import type { PlayerView } from '../types';
import { markQueueHintSeen, queueHintSeen, resetQueueHint } from './queueHint';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
  resetQueueHint();
});
afterEach(() => {
  act(() => root.unmount());
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});
const render = (node: ReactNode) => act(() => root.render(node));
const click = (target: Element) => act(() => (target as HTMLElement).click());

async function game(): Promise<PlayerView> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const views = [client.view(0), client.view(1)];
  useGame.setState({ client, views, humans: [0, 1], orders: [[], []], errors: [[], []], ready: [false, false], techOpen: true });
  return views[0];
}
const card = (tech: string) => container.querySelector<HTMLElement>(`.era-card[data-tech="${tech}"]`)!;
const queued = () => [...container.querySelectorAll('.research-queue li')].map((li) => (li as HTMLElement).dataset.tech);
const orders = () => useGame.getState().orders[0];

it('shows the current era as the only choosable cards and the later eras as locked rows, with no prerequisite tree', async () => {
  const view = await game();
  render(createElement(TechPanel));
  const r = view.research;
  expect(container.querySelectorAll('.era-card')).toHaveLength(r.era_total);
  for (const option of r.catalog.filter((t) => t.era === r.era)) expect(card(option.tech)).not.toBeNull();
  expect(container.querySelector('.research-lines, .research-tree, .research-branches')).toBeNull();
  const rows = [...container.querySelectorAll('.era-row')];
  expect(rows.length).toBeGreaterThanOrEqual(5);
  expect(rows.every((row) => (row as HTMLElement).dataset.state === 'locked')).toBe(true);
  expect(rows[0].textContent).toMatch(/Classical.*5 technologies/);
  // Era name and progress are always visible.
  expect(container.querySelector('[data-testid="era-badge"]')?.textContent).toContain(`${r.era_name} era`);
  expect(container.querySelector('[data-testid="era-progress"]')?.textContent).toBe(`${r.era_known} of ${r.era_total} known`);
  // A later era opens as a read-only peek of its names.
  click(rows[0].querySelector('button')!);
  expect(rows[0].querySelectorAll('.era-peek li').length).toBeGreaterThan(0);
});

it('uses the benefit ledger alone and places era objectives below the cards', async () => {
  await game();
  render(createElement(TechPanel));
  const dialog = container.querySelector('dialog.tech-modal')!;
  expect(dialog.querySelectorAll('.benefit-ledger')).toHaveLength(5);
  expect(dialog.querySelector('[aria-label="Tech card design"]')).toBeNull();
  const scroll = dialog.querySelector('.research-scroll')!;
  const current = scroll.querySelector('.era-current')!;
  expect(current.nextElementSibling).toBe(scroll.querySelector('.era-objectives-research'));
  const bronze = card('BronzeWorking');
  expect(bronze.querySelector('.ledger-heading')?.textContent).toBe('Building');
  expect(bronze.querySelector('.era-unlock-art img')).not.toBeNull();
});

it('each card shows its cost and what it unlocks, and no boost', async () => {
  const view = await game();
  const first = view.research.catalog.find((t) => t.era === view.research.era)!;
  render(createElement(TechPanel));
  const el = card(first.tech);
  expect(el.querySelector('[data-testid="era-cost"]')?.textContent).toContain(String(first.cost.total));
  expect(el.querySelector('[data-testid^="boost"]')).toBeNull();
  expect(el.textContent ?? '').not.toMatch(/boost/i);
  expect(el.querySelector('.era-unlocks')).not.toBeNull();
});

it('offers exactly three forecasted Science conversions after the tree is complete', async () => {
  const view = await game();
  view.research.known = view.research.catalog.map((t) => t.tech);
  view.research.science_conversion = 'Gold';
  const own = { name: 'Test city', owner: view.me } as PlayerView['cities'][number];
  view.cities.push(own);
  view.economy.science_per_turn = { total: 11, parts: [[own.name, 11]] };
  render(createElement(TechPanel));
  const options = [...container.querySelectorAll<HTMLButtonElement>('.science-conversion-options button')];
  expect(options).toHaveLength(3);
  expect(options[0].textContent).toContain('+5 Gold per turn');
  expect(options[1].textContent).toContain('+5 Food per turn');
  expect(options[1].textContent).toContain(`${own.name} +5`);
  expect(options[2].textContent).toContain('+5 Production per turn');
  click(options[2]);
  expect(orders()).toContainEqual({ type: 'SetScienceConversion', to: 'Production' });
});

it('the discount line names the empires that know the tech', async () => {
  const view = await game();
  const first = view.research.catalog.find((t) => t.era === view.research.era)!;
  first.cost = { total: 60, parts: [['Listed cost', 100], ['Tech diffusion (4 empires know it)', -40]] };
  render(createElement(TechPanel));
  const el = card(first.tech);
  expect(el.querySelector('[data-testid="era-discounts"]')?.textContent).toBe('−40%: known by 4 empires');
  // The struck listed price is the one the discounts start from.
  expect(el.querySelector('.era-cost-base')?.textContent).toBe('100');
});

it('with nothing queued the screen offers one primary: queue the whole era, cheapest first', async () => {
  const view = await game();
  view.research.queue = []; view.research.current = null;
  render(createElement(TechPanel));
  const primary = container.querySelector<HTMLButtonElement>('[data-testid="queue-all"]')!;
  expect(primary.className).toContain('primary');
  click(primary);
  const cheapest = view.research.catalog.filter((t) => t.researchable).sort((a, b) => a.cost.total - b.cost.total).map((t) => t.tech);
  expect(orders()).toEqual([{ type: 'SetResearchQueue', techs: cheapest }]);
  expect(queued()).toEqual(cheapest);
});

it('Research now puts a tech first, + Queue appends, and Remove drops it; each queued tech shows its arrival turn', async () => {
  const view = await game();
  view.research.queue = []; view.research.current = null;
  const [a, b, c] = view.research.catalog.filter((t) => t.researchable).map((t) => t.tech);
  render(createElement(TechPanel));
  click(card(a).querySelector('[data-testid="research-now"]')!);
  click(card(b).querySelector('[data-testid="queue-add"]')!);
  expect(queued()).toEqual([a, b]);
  click(card(c).querySelector('[data-testid="research-now"]')!);
  expect(queued()).toEqual([c, a, b]);
  expect(orders()).toEqual([{ type: 'SetResearchQueue', techs: [c, a, b] }]);
  for (const li of container.querySelectorAll('.research-queue li')) expect(li.querySelector('[data-testid="queue-arrival"]')?.textContent).toMatch(/^(Turn \d+ · in \d+ turns?|Needs Science|No Science yet)$/);
  click(card(a).querySelector('[aria-label^="Remove"]')!);
  expect(queued()).toEqual([c, b]);
});

it('reordering the queue is an obvious control and moves the forecast with it', async () => {
  const view = await game();
  // The cheapest and the dearest: swapping them must move the first arrival.
  const [a, b] = view.research.catalog.filter((t) => t.researchable).sort((x, y) => x.cost.total - y.cost.total).filter((_, i, all) => i === 0 || i === all.length - 1).map((t) => t.tech);
  view.research.queue = [a, b]; view.research.current = a;
  view.economy.science_per_turn = { total: 3, parts: [['Capital', 3]] };
  render(createElement(TechPanel));
  const arrivals = () => [...container.querySelectorAll('[data-testid="queue-arrival"]')].map((e) => e.textContent);
  const before = arrivals();
  expect(container.querySelector<HTMLButtonElement>(`.research-queue li[data-tech="${a}"] [aria-label^="Move"][aria-label$="earlier"]`)!.disabled).toBe(true);
  click(container.querySelector(`.research-queue li[data-tech="${b}"] [aria-label^="Move"][aria-label$="earlier"]`)!);
  expect(queued()).toEqual([b, a]);
  expect(orders()).toEqual([{ type: 'SetResearchQueue', techs: [b, a] }]);
  expect(before[0]).toMatch(/^Turn \d+/);
  expect(arrivals()[0]).not.toBe(before[0]);
});

it('the top bar names the era and opens research', async () => {
  const view = await game();
  useGame.setState({ techOpen: false });
  render(createElement(TopBar));
  const chip = container.querySelector<HTMLButtonElement>('[data-testid="era-chip"]')!;
  expect(chip.textContent).toContain(view.research.era_name);
  expect(chip.textContent).toContain(`${view.research.era_known}/${view.research.era_total}`);
  click(chip);
  expect(useGame.getState().techOpen).toBe(true);
});

// ---- Whole-card shortcuts, states and the fixed-height queue strip (user 2026-10-11) -----------------

const clickCard = (el: Element, init: MouseEventInit = {}) => act(() => { el.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, ...init })); });
const key = (el: Element, k: string, shiftKey = false) => act(() => { el.dispatchEvent(new KeyboardEvent('keydown', { key: k, shiftKey, bubbles: true, cancelable: true })); });
const slotCount = () => container.querySelectorAll('.research-queue-slots li, .research-queue-slots .rq-ghost').length;
const emptyQueue = async () => {
  const view = await game();
  view.research.queue = []; view.research.current = null;
  const [a, b, c] = view.research.catalog.filter((t) => t.researchable).map((t) => t.tech);
  render(createElement(TechPanel));
  return { view, a, b, c };
};

it('clicking anywhere on a card researches it now; Shift+click queues it at the end', async () => {
  const { a, b, c } = await emptyQueue();
  clickCard(card(a).querySelector('.era-card-name')!);
  expect(queued()).toEqual([a]);
  clickCard(card(b), { shiftKey: true });
  clickCard(card(c).querySelector('.era-unlocks')!, { shiftKey: true });
  expect(queued()).toEqual([a, b, c]);
  // A plain click on a queued card moves it to the front, exactly like "Research now".
  clickCard(card(c));
  expect(queued()).toEqual([c, a, b]);
  expect(orders()).toEqual([{ type: 'SetResearchQueue', techs: [c, a, b] }]);
  // Shift+click on something already queued changes nothing.
  clickCard(card(b), { shiftKey: true });
  expect(queued()).toEqual([c, a, b]);
});

it('a click on a button inside the card does that button\'s job once, not the card\'s as well', async () => {
  const { a, b } = await emptyQueue();
  click(card(a).querySelector('[data-testid="research-now"]')!);
  click(card(b).querySelector('[data-testid="queue-add"]')!);
  expect(queued()).toEqual([a, b]);
  click(card(b).querySelector('.era-details-chevron')!);
  expect(queued()).toEqual([a, b]);
  expect(container.querySelector('.research-detail')).not.toBeNull();
});

it('Enter on a focused card researches it, Shift+Enter queues it; the buttons name the shortcuts', async () => {
  const { a, b, c } = await emptyQueue();
  expect(card(a).tabIndex).toBe(0);
  key(card(a), 'Enter');
  key(card(b), 'Enter', true);
  expect(queued()).toEqual([a, b]);
  key(card(c), 'Enter', true);
  key(card(c), 'Enter');
  expect(queued()).toEqual([c, a, b]);
  expect(card(a).querySelector('[data-testid="research-now"]')!.getAttribute('title')).toMatch(/click anywhere on the card/i);
  expect(card(b).querySelector('[data-testid="queue-add"]')).toBeNull();
  expect(card(c).querySelector('[data-testid="queue-add"]')).toBeNull();
});

it('a researched card is not a target', async () => {
  const view = await game();
  const known = view.research.catalog.find((t) => view.research.known.includes(t.tech) && t.era === view.research.era);
  render(createElement(TechPanel));
  if (!known) return;
  expect(card(known.tech).hasAttribute('tabindex')).toBe(false);
  clickCard(card(known.tech));
  expect(queued()).toEqual([]);
});

it('the researching card says "Researching · N turns" with progress and no Research now button', async () => {
  const { a } = await emptyQueue();
  clickCard(card(a));
  const el = card(a);
  expect(el.dataset.state).toBe('researching');
  expect(el.querySelector('[data-testid="era-researching"]')?.textContent).toMatch(/^Researching · (\d+ turns?|needs Science|No Science yet)$/);
  expect(el.querySelector('.era-progress')?.getAttribute('role')).toBe('progressbar');
  expect(el.querySelector('[data-testid="research-now"]')).toBeNull();
  expect(el.querySelector('[data-testid="queue-add"]')).toBeNull();
  expect(el.querySelector('[data-testid="research-order-badge"]')).toBeNull();
});

it('a queued card shows "Queued #N" and a remove x in a corner tag, and no + Queue', async () => {
  const { a, b, c } = await emptyQueue();
  clickCard(card(a));
  clickCard(card(b), { shiftKey: true });
  clickCard(card(c), { shiftKey: true });
  for (const [tech, n] of [[b, 2], [c, 3]] as const) {
    const el = card(tech);
    expect(el.dataset.state).toBe('queue');
    expect(el.querySelector('[data-testid="research-order-badge"]')?.textContent).toBe(`Queued #${n}`);
    expect(el.querySelector('[data-testid="queue-add"]')).toBeNull();
    expect(el.querySelectorAll('.era-card-foot .btn')).toHaveLength(1); // one action left: Research now (quiet: the state is the tag)
    // The tag is a direct child of the card, not part of the head's flow.
    expect(el.querySelector('.era-queue-tag')?.parentElement).toBe(el);
  }
  click(card(b).querySelector('[aria-label="Remove ' + (container.querySelector(`.research-queue li[data-tech="${b}"] b`)?.textContent ?? '') + ' from the queue"]')!);
  expect(queued()).toEqual([a, c]);
  expect(card(b).querySelector('[data-testid="queue-add"]')).not.toBeNull();
  expect(card(c).querySelector('[data-testid="research-order-badge"]')?.textContent).toBe('Queued #2');
});

it('the queue strip keeps the same slots, so queueing never changes its height', async () => {
  const { view, a, b, c } = await emptyQueue();
  const slots = view.research.catalog.filter((t) => t.era === view.research.era && t.researchable && !view.research.known.includes(t.tech)).length;
  expect(slotCount()).toBe(slots);
  clickCard(card(a));
  expect(slotCount()).toBe(slots);
  clickCard(card(b), { shiftKey: true });
  clickCard(card(c), { shiftKey: true });
  expect(slotCount()).toBe(slots);
  expect(container.querySelectorAll('.research-queue li')).toHaveLength(3);
  expect(container.querySelectorAll('.research-queue-slots .rq-ghost')).toHaveLength(slots - 3);
  click(container.querySelector('[data-testid="queue-rest"]') ?? document.body);
  expect(slotCount()).toBe(slots);
});

it('the first-time hint shows in the next free slot until the player has queued once, and is remembered', async () => {
  const { a, b } = await emptyQueue();
  const hint = () => container.querySelector('[data-testid="queue-hint"]');
  expect(hint()?.textContent).toContain('Shift+click a technology to queue it next.');
  clickCard(card(a)); // researching alone is not queueing
  expect(hint()).not.toBeNull();
  clickCard(card(b), { shiftKey: true });
  expect(hint()).toBeNull();
  expect(queueHintSeen()).toBe(true);
  expect(localStorage.getItem('legatum.research-queue-hint.v1')).toBe('1');
});

it('the hint is hidden once it has been seen, and still works with storage blocked', async () => {
  markQueueHintSeen();
  const { a, b } = await emptyQueue();
  expect(container.querySelector('[data-testid="queue-hint"]')).toBeNull();
  resetQueueHint();
  vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('blocked'); });
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('blocked'); });
  expect(queueHintSeen()).toBe(false);
  act(() => root.render(createElement(TechPanel)));
  clickCard(card(a));
  clickCard(card(b), { shiftKey: true });
  expect(queueHintSeen()).toBe(true); // remembered for the session even without storage
  expect(container.querySelector('[data-testid="queue-hint"]')).toBeNull();
});

it('picking a card glows it and runs the fly-to-queue chip, then clears both', async () => {
  vi.useFakeTimers();
  try {
    const { a } = await emptyQueue();
    clickCard(card(a));
    expect(card(a).classList.contains('picked')).toBe(true);
    expect(container.querySelector('.research-queue li.landed')?.getAttribute('data-tech')).toBe(a);
    act(() => { vi.advanceTimersByTime(600); });
    expect(card(a).classList.contains('picked')).toBe(false);
    expect(container.querySelector('.fly-chip')).toBeNull();
  } finally { vi.useRealTimers(); }
});

// ---- The unlock hovercard -----------------------------------------------------------------------

const pointer = (el: Element, type: string, pointerType = 'mouse') => act(() => { el.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, pointerType, relatedTarget: document.body })); });
const unlockRow = (kind: string) => container.querySelector<HTMLElement>(`.era-unlock.${kind}`)!;
const hovercard = () => document.querySelector<HTMLElement>('[data-testid="unlock-card"]');

it('hovering an unlock shows its card after a short delay, and leaving hides it', async () => {
  vi.useFakeTimers();
  try {
    await game();
    render(createElement(TechPanel));
    const row = unlockRow('building');
    pointer(row, 'pointerover');
    expect(hovercard()).toBeNull();
    act(() => { vi.advanceTimersByTime(300); });
    expect(hovercard()).not.toBeNull();
    expect(hovercard()!.dataset.type).toBe('building');
    expect(hovercard()!.textContent).toContain('What it gives');
    expect(hovercard()!.textContent).toContain('Why build it');
    expect(hovercard()!.parentElement?.tagName).toBe('DIALOG');
    pointer(row, 'pointerout');
    expect(hovercard()).toBeNull();
  } finally { vi.useRealTimers(); }
});

it('keyboard focus shows nothing until Enter pins it; Escape closes the card, not the research screen', async () => {
  await game();
  render(createElement(TechPanel));
  const row = unlockRow('building');
  expect(row.tabIndex).toBe(0);
  expect(row.hasAttribute('data-enter-activates')).toBe(true);
  act(() => row.focus());
  key(row, 'Enter');
  expect(hovercard()?.getAttribute('role')).toBe('dialog');
  expect(hovercard()?.querySelector('.unlock-card-close')).not.toBeNull();
  act(() => { document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })); });
  expect(hovercard()).toBeNull();
  expect(useGame.getState().techOpen).toBe(true);
});

it('a tap pins the card instead of researching the tech; a mouse click on the row still researches it', async () => {
  const { a } = await emptyQueue();
  const row = card(a).querySelector<HTMLElement>('.era-unlock')!;
  pointer(row, 'pointerdown', 'touch');
  clickCard(row);
  expect(hovercard()).not.toBeNull();
  expect(queued()).toEqual([]);
  act(() => { (hovercard()!.querySelector('.unlock-card-close') as HTMLElement).click(); });
  expect(hovercard()).toBeNull();
  pointer(row, 'pointerdown', 'mouse');
  clickCard(row);
  expect(queued()).toEqual([a]);
});
