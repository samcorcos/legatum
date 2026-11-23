// @vitest-environment happy-dom
import { act, createElement, StrictMode, useState, type ReactNode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { SHOW_DELAY, Tip, placeBubble } from './Tip';
import { BreakdownTable } from './bits';
import { FactionPicker } from './FactionPicker';
import { FACTIONS, FACTION_INFO, uniquePieceRules } from '../data';
import { SeatBar } from './SeatBar';
import { CityList } from './CityList';
import { TurnReport } from './TurnReport';
import { TechPanel } from './TechPanel';
import { TopBar } from './TopBar';
import { useGame } from '../store';
import { FixtureClient } from '../test/fixtureEngine';
import type { CityView, PlayerView } from '../types';
import type { FactionChoice } from '../lobby';
import { setResearchQueue } from '../orders';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});

it('keeps the piece card text to rule sentences, never leftovers of a stat line', () => {
  expect(uniquePieceRules('Celts')).toBe('');
  expect(uniquePieceRules('Mongols')).toBe('');
  expect(uniquePieceRules('Switzerland')).toBe('Needs Masonry.');
  expect(uniquePieceRules('Byzantium')).toBe(`Needs Feudalism. Phalangite upgrade costs ${84} Gold.`);
});

it('gives Random a complete ornamental sheet with every possible faction', () => {
  render(createElement(FactionPicker, { label: 'Seat 1 faction', value: 'Random', onChange: vi.fn() }));
  click(container.querySelector<HTMLButtonElement>('.faction-current')!);
  expect(container.querySelector('.fs-sheet')?.textContent).toContain('Could be any of these');
  expect(container.querySelectorAll('.fs-random-seal')).toHaveLength(FACTIONS.length);
});
afterEach(() => {
  act(() => root.unmount());
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});
const render = (node: ReactNode) => act(() => root.render(node));
const key = (target: Element, value: string, options: KeyboardEventInit = {}) => act(() => {
  target.dispatchEvent(new KeyboardEvent('keydown', { key: value, bubbles: true, cancelable: true, ...options }));
});
const click = (target: HTMLElement) => act(() => target.click());
const mouseOver = (target: HTMLElement) => act(() => { target.dispatchEvent(new MouseEvent('mouseover', { bubbles: true, relatedTarget: document.body })); });
const mouseOut = (target: HTMLElement) => act(() => { target.dispatchEvent(new MouseEvent('mouseout', { bubbles: true, relatedTarget: document.body })); });
// Resting the pointer for longer than the hover-intent delay.
const rest = (ms = 200) => act(() => { vi.advanceTimersByTime(ms); });
const focus = (target: HTMLElement) => act(() => target.focus());

async function game() {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const views = [client.view(0), client.view(1)];
  useGame.setState({ client, views, humans: [0, 1], orders: [[], []], errors: [[], []], ready: [false, false] });
  return views[0];
}

it('static and disabled tooltip triggers are focusable, described, and dismiss with Escape', () => {
  render(createElement(Tip, { content: 'Food minus consumption', children: 'Growth' }));
  const trigger = container.firstElementChild as HTMLElement;
  expect(trigger.tabIndex).toBe(0);
  const bubble = document.getElementById(trigger.getAttribute('aria-describedby')!)!;
  expect(bubble.getAttribute('role')).toBe('tooltip');
  expect(bubble.hidden).toBe(true);
  focus(trigger);
  expect(bubble.hidden).toBe(false);
  key(trigger, 'Escape');
  expect(bubble.hidden).toBe(true);
  expect(document.activeElement).toBe(trigger);

  render(createElement(Tip, { content: 'Requires Currency', children: createElement('button', { disabled: true }, 'Code of Laws') }));
  expect((container.firstElementChild as HTMLElement).tabIndex).toBe(0);
});

const hover = (el: Element, type: 'pointerover' | 'pointerout') =>
  act(() => { el.dispatchEvent(new PointerEvent(type, { bubbles: true, pointerType: 'mouse', relatedTarget: document.body })); });

it('tooltips describe the existing button without another tab stop, wait before showing, never take the pointer and hide on leave', () => {
  vi.useFakeTimers();
  render(createElement(Tip, { content: 'Buy with Gold', children: createElement('button', { 'aria-describedby': 'existing' }, 'Buy') }));
  const trigger = container.firstElementChild as HTMLElement;
  const button = container.querySelector('button')!;
  expect(trigger.hasAttribute('tabindex')).toBe(false);
  expect(button.getAttribute('aria-describedby')).toContain('existing ');
  const bubble = document.querySelector<HTMLElement>('[role="tooltip"]')!;
  expect(bubble.classList.contains('tip-hover')).toBe(true); // pointer-events: none in the stylesheet
  // Sweeping across a control shows nothing: the tooltip waits for a deliberate hover.
  hover(trigger, 'pointerover');
  act(() => { vi.advanceTimersByTime(SHOW_DELAY - 50); });
  expect(bubble.hidden).toBe(true);
  act(() => { vi.advanceTimersByTime(50); });
  expect(bubble.hidden).toBe(false);
  // Leaving hides it at once (the bubble can't be hovered, so there is nothing to cross to).
  hover(trigger, 'pointerout');
  expect(bubble.hidden).toBe(true);
  // Pressing the control never leaves its tooltip over the game.
  hover(trigger, 'pointerover');
  act(() => { vi.advanceTimersByTime(SHOW_DELAY); });
  expect(bubble.hidden).toBe(false);
  act(() => { button.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, pointerType: 'mouse' })); });
  expect(bubble.hidden).toBe(true);
});

it('rich breakdowns show a compact summary on hover and open pinned, with a close control, on click', () => {
  vi.useFakeTimers();
  const b = { total: 3, parts: [['City center', 1], ['Worked tiles', 2]] as [string, number][] };
  render(createElement(Tip, { content: createElement(BreakdownTable, { b, title: 'Production', unit: 'per turn' }), children: '3 Production' }));
  const trigger = container.firstElementChild as HTMLElement;
  const bubble = () => document.getElementById(trigger.getAttribute('aria-describedby')!)!;
  hover(trigger, 'pointerover');
  act(() => { vi.advanceTimersByTime(SHOW_DELAY); });
  expect(bubble().textContent).toContain('Production: 3 per turn');
  expect(bubble().textContent).not.toContain('Worked tiles');
  click(trigger);
  expect(bubble().getAttribute('aria-label')).toBe('Breakdown');
  expect(bubble().classList.contains('tip-pinned')).toBe(true);
  expect(bubble().textContent).toContain('Worked tiles');
  // Pinned survives the pointer leaving; its ✕ closes it.
  hover(trigger, 'pointerout');
  expect(bubble().hidden).toBe(false);
  click(bubble().querySelector<HTMLButtonElement>('[aria-label="Close breakdown"]')!);
  expect(bubble().hidden).toBe(true);
});

it('a control with a rich breakdown keeps its click and offers a separate ⓘ', () => {
  const action = vi.fn();
  const b = { total: 120, parts: [['Base upgrade Gold', 120]] as [string, number][] };
  render(createElement(Tip, { content: createElement(BreakdownTable, { b, title: 'Upgrade cost' }), children: createElement('button', { onClick: action }, 'Upgrade') }));
  click(container.querySelector<HTMLButtonElement>('button:not([data-tip-info])')!);
  expect(action).toHaveBeenCalledOnce();
  expect(document.querySelector('[aria-label="Breakdown"]')).toBeNull();
  click(container.querySelector<HTMLButtonElement>('[data-tip-info]')!);
  expect(document.querySelector('[aria-label="Breakdown"]')?.textContent).toContain('Upgrade cost');
  expect(action).toHaveBeenCalledOnce();
});

it('a hoverable bubble near the viewport edge flips sides instead of covering (and swallowing clicks on) its trigger', () => {
  // A right-placed bubble near the right edge used to be clamped over its own trigger, so a click
  // on the trigger landed on the bubble. It flips to the left instead.
  const option = { left: 995, right: 1310, top: 180, bottom: 228 };
  const bubbleSize = { width: 320, height: 300 };
  const viewport = { width: 1440, height: 1100 };
  const flipped = placeBubble(option, bubbleSize, 'right', viewport);
  expect(flipped.left < option.right && flipped.left + bubbleSize.width > option.left && flipped.top < option.bottom && flipped.top + bubbleSize.height > option.top).toBe(false);
  expect(flipped.left + bubbleSize.width).toBeLessThanOrEqual(option.left);

  // Where it fits, the requested side is kept; a bottom bubble at the viewport's foot goes on top.
  const trigger = { left: 100, right: 200, top: 100, bottom: 130 };
  expect(placeBubble(trigger, bubbleSize, 'right', viewport)).toEqual({ left: 208, top: 8 });
  const low = { left: 100, right: 200, top: 1000, bottom: 1030 };
  expect(placeBubble(low, bubbleSize, 'bottom', viewport).top).toBe(1000 - 8 - bubbleSize.height);

  // It keeps clear of the panel it explains and of the primary controls when another side is free:
  // a yield in a left-docked city panel explains itself over the map, not over the panel.
  const cityPanel = { left: 16, right: 336, top: 72, bottom: 784 };
  const yieldCell = { left: 80, right: 140, top: 250, bottom: 290 };
  const beside = placeBubble(yieldCell, bubbleSize, 'top', { width: 1280, height: 800 }, [cityPanel]);
  expect(beside.left).toBeGreaterThanOrEqual(cityPanel.right);
  // A milestone note left of the race panel never drops over the End turn panel below it.
  const endTurn = { left: 1100, right: 1424, top: 640, bottom: 884 };
  const note = { left: 1120, right: 1400, top: 300, bottom: 340 };
  const clear = placeBubble(note, bubbleSize, 'bottom', viewport, [endTurn]);
  expect(clear.top + bubbleSize.height <= endTurn.top || clear.left + bubbleSize.width <= endTurn.left).toBe(true);
});

it('research is a named native modal, focuses Close, traps both Tab directions, and hosts its dismissible tooltip', async () => {
  await game();
  const showModal = vi.spyOn(HTMLDialogElement.prototype, 'showModal');
  const opener = document.body.appendChild(document.createElement('button'));
  opener.textContent = 'Research';
  focus(opener);
  useGame.setState({ techOpen: true });
  // Also exercise StrictMode's setup/cleanup/setup cycle with an initially open modal.
  render(createElement(StrictMode, null, createElement(TechPanel)));
  const dialog = container.querySelector('dialog')!;
  expect(showModal).toHaveBeenCalled();
  expect(dialog.open).toBe(true);
  expect(document.getElementById(dialog.getAttribute('aria-labelledby')!)?.textContent).toBe('Research');
  const close = dialog.querySelector<HTMLButtonElement>('[aria-label="Close research"]')!;
  expect(document.activeElement).toBe(close);
  const tip = dialog.querySelector<HTMLElement>('.era-card .era-cost [tabindex="0"]')!;
  const bubble = document.getElementById(tip.getAttribute('aria-describedby')!)!;
  expect(bubble.parentElement).toBe(dialog);
  expect(bubble.hidden).toBe(true);

  key(close, 'Tab', { shiftKey: true });
  const last = [...dialog.querySelectorAll<HTMLElement>('button:not(:disabled), summary, [tabindex="0"]')].filter((el) => !el.closest('details:not([open])') || el.tagName === 'SUMMARY').at(-1)!;
  expect(document.activeElement).toBe(last);
  key(last, 'Tab');
  expect(document.activeElement).toBe(close);
  focus(tip);
  expect(bubble.hidden).toBe(false);
  // Escape dismisses the current tooltip first, without dismissing its dialog.
  key(tip, 'Escape');
  expect(bubble.hidden).toBe(true);
  expect(dialog.open).toBe(true);
  focus(close);
  key(close, 'Escape');
  expect(useGame.getState().techOpen).toBe(false);
  expect(container.querySelector('dialog')).toBe(null);
  expect(document.activeElement).toBe(opener);
});

it.each(['Close button', 'native cancel', 'native close', 'external close', 'unmount'] as const)('research restores the opener after %s and can reopen', async (method) => {
  await game();
  const opener = document.body.appendChild(document.createElement('button'));
  focus(opener);
  render(createElement(TechPanel));
  act(() => useGame.setState({ techOpen: true }));
  const dialog = container.querySelector('dialog')!;
  if (method === 'Close button') click(dialog.querySelector<HTMLButtonElement>('[aria-label="Close research"]')!);
  else if (method === 'native cancel') act(() => dialog.dispatchEvent(new Event('cancel', { cancelable: true })));
  else if (method === 'native close') act(() => dialog.close());
  else if (method === 'external close') act(() => useGame.setState({ techOpen: false }));
  else render(null);
  expect(dialog.open).toBe(false);
  expect(document.activeElement).toBe(opener);
  if (method !== 'unmount') {
    expect(useGame.getState().techOpen).toBe(false);
    act(() => useGame.setState({ techOpen: true }));
    expect(container.querySelector('dialog')?.open).toBe(true);
    expect(document.activeElement?.getAttribute('aria-label')).toBe('Close research');
  }
});

// A parent that keeps the seat's faction, as the lobby does: the picker is controlled.
function PickerHarness({ initial, onChange, label = 'Seat 1 faction' }: { initial: FactionChoice; onChange: (f: FactionChoice) => void; label?: string }) {
  const [value, setValue] = useState<FactionChoice>(initial);
  return createElement(FactionPicker, { value, label, onChange: (f: FactionChoice) => { setValue(f); onChange(f); } });
}

it('the faction stage shows one sheet at a time, never stacked tooltips', () => {
  // Choosing a tile swaps the one sheet for that faction; there are no tooltip bubbles at all.
  vi.useFakeTimers();
  render(createElement(PickerHarness, { initial: 'India', onChange: vi.fn(), label: 'Your faction' }));
  click(container.querySelector<HTMLElement>('.faction-current')!);
  const options = () => [...container.querySelectorAll<HTMLButtonElement>('[role="option"]')];
  expect(document.activeElement?.getAttribute('aria-selected')).toBe('true');
  for (let i = 0; i < options().length; i++) {
    click(options()[i]);
    const option = options()[i];
    const sheet = container.querySelector<HTMLElement>('.fs-sheet')!;
    const name = option.querySelector('.fs-tile-name')!.textContent!;
    expect(sheet.textContent).toContain(name);
    expect(document.activeElement).toBe(option);
    expect(option.getAttribute('aria-selected')).toBe('true');
    expect(container.querySelectorAll('[aria-selected="true"]')).toHaveLength(1);
    // Only the chosen option is described by the sheet; no tooltip bubbles exist at all.
    expect(container.querySelectorAll('[aria-describedby]')).toHaveLength(1);
    expect(option.getAttribute('aria-describedby')).toBe(sheet.id);
    expect(document.querySelectorAll('[role="tooltip"]:not([hidden])')).toHaveLength(0);
    expect(container.querySelectorAll('.fs-sheet')).toHaveLength(1);
  }
});

it('only one tooltip is open at a time, whether opened by hover or focus', () => {
  render(createElement('div', null,
    createElement(Tip, { content: 'First', children: createElement('button', null, 'One') }),
    createElement(Tip, { content: 'Second', children: createElement('button', null, 'Two') }),
    createElement(Tip, { content: 'Third', children: 'Three' })));
  const [one, two] = [...container.querySelectorAll('button')];
  const three = container.querySelector<HTMLElement>('[role="group"]')!;
  const open = () => [...document.querySelectorAll<HTMLElement>('[role="tooltip"]')].filter((b) => !b.hidden).map((b) => b.textContent);
  focus(one);
  expect(open()).toEqual(['First']);
  // With one already open, the next shows at once (no second delay) and replaces it.
  act(() => { two.parentElement!.dispatchEvent(new PointerEvent('pointerover', { bubbles: true, pointerType: 'mouse', relatedTarget: document.body })); });
  expect(open()).toEqual(['Second']);
  focus(three);
  expect(open()).toEqual(['Third']);
});

it('the faction stage names its value; arrows, Home and End pick as they move; Return closes and keeps the pick', () => {
  const onChange = vi.fn();
  render(createElement(PickerHarness, { initial: 'India', onChange }));
  const trigger = container.querySelector<HTMLButtonElement>('.faction-current')!;
  expect(trigger.getAttribute('aria-label')).toMatch(/Seat 1 faction: .*India/);
  click(trigger);
  const list = container.querySelector('[role="listbox"]')!;
  expect(list.getAttribute('aria-label')).toBe('Seat 1 faction');
  const options = [...list.querySelectorAll<HTMLButtonElement>('[role="option"]')];
  // Opening the stage focuses the current value, not just the first tile.
  expect(document.activeElement?.getAttribute('aria-selected')).toBe('true');
  expect(document.activeElement?.textContent).toContain('India');
  expect(onChange).not.toHaveBeenCalled();
  key(document.activeElement!, 'End');
  expect(document.activeElement).toBe(options.at(-1));
  expect(onChange).toHaveBeenLastCalledWith(FACTIONS.at(-1));
  key(document.activeElement!, 'Home');
  expect(document.activeElement).toBe(options[0]);
  expect(onChange).toHaveBeenLastCalledWith('Random');
  key(document.activeElement!, 'ArrowUp');
  expect(document.activeElement).toBe(options.at(-1));
  // Arrowing picks: the sheet follows and the seat's trigger already shows the choice.
  key(document.activeElement!, 'ArrowUp');
  expect(onChange).toHaveBeenLastCalledWith(FACTIONS.at(-2));
  expect(container.querySelector('.fs-sheet')!.textContent).toContain(options.at(-2)!.querySelector('.fs-tile-name')!.textContent);
  expect(trigger.getAttribute('aria-label')).toContain(FACTION_INFO[FACTIONS.at(-2)!].name);
  // Return closes the stage and keeps the choice; it does not pick anything new.
  onChange.mockClear();
  key(container.querySelector('dialog')!, 'Enter');
  expect(onChange).not.toHaveBeenCalled();
  expect(container.querySelector('dialog')!.open).toBe(false);
  expect(document.activeElement).toBe(trigger);
  expect(trigger.getAttribute('aria-label')).toContain(FACTION_INFO[FACTIONS.at(-2)!].name);
});

it('clicking a faction tile picks it and the stage stays open; Close, Return and Escape keep the choice; there is no Play button', () => {
  vi.useFakeTimers();
  const onChange = vi.fn();
  render(createElement(PickerHarness, { initial: 'India', onChange }));
  const trigger = container.querySelector<HTMLButtonElement>('.faction-current')!;
  const dialog = container.querySelector('dialog')!;
  const options = () => [...container.querySelectorAll<HTMLButtonElement>('[role="option"]')];
  const tile = (name: string) => options().find((o) => o.textContent?.includes(name))!;
  click(trigger);
  expect(container.querySelector('.fs-play')).toBeNull();
  expect([...container.querySelectorAll('dialog button')].some((b) => /^Play/.test(b.textContent ?? ''))).toBe(false);
  // A click is the selection: the seat changes at once, the tile is current and the dialog stays.
  click(tile('Mongols'));
  expect(onChange).toHaveBeenCalledExactlyOnceWith('Mongols');
  expect(dialog.open).toBe(true);
  expect(tile('Mongols').getAttribute('aria-selected')).toBe('true');
  expect(tile('India').getAttribute('aria-selected')).toBe('false');
  expect(container.querySelector('.fs-sheet')!.textContent).toContain('Mongols');
  expect(container.querySelector('[data-testid="faction-chosen"]')!.textContent).toContain('Mongols');
  // Clicking another tile toggles the choice again; Random is a gallery entry like any other.
  click(tile('Random'));
  expect(onChange).toHaveBeenLastCalledWith('Random');
  expect(container.querySelector('.fs-sheet')!.textContent).toContain('Could be any of these');
  click(tile('Byzantium'));
  expect(onChange).toHaveBeenLastCalledWith('Byzantium');
  // Close keeps the current choice and returns focus to the seat.
  onChange.mockClear();
  click([...container.querySelectorAll<HTMLButtonElement>('.fs-close')][0]);
  expect(dialog.open).toBe(false);
  expect(onChange).not.toHaveBeenCalled();
  expect(document.activeElement).toBe(trigger);
  expect(trigger.getAttribute('aria-label')).toMatch(/Byzantium/);
  // Tab only visits Close and the chosen tile, so the wrap keeps focus inside the dialog.
  click(trigger);
  const stops = [...dialog.querySelectorAll<HTMLElement>('button:not(:disabled):not([tabindex="-1"])')];
  expect(stops.map((b) => b.className.includes('fs-close') || b.getAttribute('aria-selected') === 'true')).toEqual([true, true]);
  // A click on the backdrop (the dialog element itself) closes and keeps the choice.
  click(dialog);
  expect(dialog.open).toBe(false);
  expect(trigger.getAttribute('aria-label')).toMatch(/Byzantium/);
  // Escape closes too, keeping the choice (no revert).
  click(trigger);
  click(tile('Mongols'));
  act(() => { dialog.dispatchEvent(new Event('cancel', { cancelable: true })); });
  expect(dialog.open).toBe(false);
  expect(trigger.getAttribute('aria-label')).toMatch(/Mongols/);
});

it('hovering a faction tile never changes the sheet or the choice', () => {
  vi.useFakeTimers();
  const onChange = vi.fn();
  render(createElement(PickerHarness, { initial: 'India', onChange }));
  click(container.querySelector<HTMLElement>('.faction-current')!);
  const options = [...container.querySelectorAll<HTMLButtonElement>('[role="option"]')];
  const tile = (name: string) => options.find((o) => o.textContent?.includes(name))!;
  const shown = () => container.querySelector('.fs-sheet')!.textContent!;
  for (const name of ['Mongols', 'Sparta', 'Byzantium']) {
    mouseOver(tile(name));
    rest(1000);
    expect(shown()).not.toContain(name);
    expect(shown()).toContain('India');
    expect(tile('India').getAttribute('aria-selected')).toBe('true');
    mouseOut(tile(name));
  }
  expect(onChange).not.toHaveBeenCalled();
});

it('readiness and pending orders are accessible controls', async () => {
  await game();
  useGame.setState({ orders: [[{ type: 'SleepUnit', unit: 0 }], []] });
  render(createElement(SeatBar));
  expect(container.querySelector('button button')).toBe(null);
  const disclosure = container.querySelector<HTMLButtonElement>('.orders-toggle')!;
  const orders = document.getElementById(disclosure.getAttribute('aria-controls')!)!;
  expect(orders.hidden).toBe(true);
  click(disclosure);
  expect(disclosure.getAttribute('aria-expanded')).toBe('true');
  expect(orders.hidden).toBe(false);
  expect(orders.querySelector('button')?.getAttribute('aria-label')).toMatch(/^Cancel .+/);
});

it('the Cities list selects producing cities as well as idle and foreign cities', () => {
  const select = vi.fn();
  const cities = [
    { id: 10, name: 'Working', owner: 0, detail: { producing: { kind: 'Unit', unit: 'Scout' } } },
    { id: 11, name: 'Idle', owner: 0, detail: { producing: null } },
    { id: 12, name: 'Known rival', owner: 1, detail: null },
  ] as CityView[];
  const view = { me: 0, cities, players: [{ id: 0, name: 'Ada', faction: 'India' }, { id: 1, name: 'Bo', faction: 'Byzantium' }] } as PlayerView;
  useGame.setState({ select });
  render(createElement(CityList, { view }));
  const buttons = [...container.querySelectorAll<HTMLButtonElement>('button')];
  expect(buttons).toHaveLength(3);
  click(buttons.find((button) => button.textContent?.includes('Working'))!);
  expect(select).toHaveBeenCalledWith({ kind: 'city', id: 10 }, true);
});

it('turn-report actions are buttons with filter and expansion states', async () => {
  await game();
  const jumpTo = vi.fn();
  useGame.setState({ reportOpen: true, jumpTo, report: { turn: 2, events: [], hash: 'test', lines: [[{
    category: 'combat', text: 'Battle near Ada', details: ['One unit defeated'],
    at: { q: 2, r: 3 }, tone: 'good', weight: 2, players: [0],
  }], []] } });
  render(createElement(TurnReport));
  const line = container.querySelector<HTMLButtonElement>('button.rline-text')!;
  expect(line).not.toBe(null);
  expect(line.getAttribute('aria-expanded')).toBe('false');
  const details = document.getElementById(line.getAttribute('aria-controls')!)!;
  expect(details.hidden).toBe(true);
  click(line);
  expect(jumpTo).toHaveBeenCalledWith({ q: 2, r: 3 }, true);
  expect(line.getAttribute('aria-expanded')).toBe('true');
  expect(details.hidden).toBe(false);
  expect(container.querySelector('.seg button.on')?.getAttribute('aria-pressed')).toBe('true');
});

it('a cleared active queue updates the top bar and the next-action CTA immediately', async () => {
  const view = await game();
  view.research.current = 'Irrigation'; view.research.queue = ['Irrigation', 'Writing'];
  useGame.setState({ orders: [view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit', unit: u.id })), []] });
  render(createElement('div', null, createElement(TopBar), createElement(SeatBar)));
  expect(container.querySelector('.research-slot')?.textContent).toContain('Writing');
  // Clearing lives in the research panel now; the top bar never offers a destructive action.
  expect(container.querySelector('[aria-label="Clear research queue"]')).toBeNull();
  act(() => useGame.getState().editOrders((o) => setResearchQueue(o, [], view.research), 'commit'));
  expect(container.querySelector('.research-slot')?.textContent).toContain('Choose a technology');
  expect(container.querySelector('.cta')?.textContent).toContain('Choose production');
  act(() => useGame.setState({ report: { turn: view.turn, hash: '0', lines: [[], []], events: [[{ type: 'ResearchQueueAdvanced', player: view.me, next: null }], []] } }));
  expect(container.querySelector('.cta')?.textContent).toContain('Choose production');
});
