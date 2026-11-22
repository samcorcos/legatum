// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { MapCanvas } from './MapCanvas';
import { draw, HEX_SIZE, type Camera } from './render';
import { hexEq, hexToPixel, neighbors, tileIndex } from '../hex';
import { useGame } from '../store';
import { FixtureClient } from '../test/fixtureEngine';
import { CelebrationProvider, MomentLane } from '../celebrate/MomentLane';
import { celebrationFixture } from '../test/celebrationFixture';
import { cancelFeedback, useCombatFx } from '../feel';
import { knownLegalSite, settlementSite } from '../settlement';
import { hexAt } from '../hex';
import { setOverlay } from './overlays';
import { Game } from '../wasm-pkg/legatum.js';

vi.mock('./render', async (original) => ({ ...await original<typeof import('./render')>(), draw: vi.fn(() => ({ pending: false, labelRects: [], ambient: [] })) }));

let root: Root;
let map: HTMLCanvasElement;
let preference: MediaQueryList;
let frames: Map<number, FrameRequestCallback>;
let frameId: number;
const camera = () => (window as unknown as { legatumMap: { camera: Camera } }).legatumMap.camera;
const frame = (time: number) => act(() => {
  const callbacks = [...frames.values()];
  frames.clear();
  callbacks.forEach((callback) => callback(time));
});
const key = (target: HTMLElement, key: string) => {
  const e = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true });
  act(() => { target.focus(); target.dispatchEvent(e); });
  return e;
};

beforeEach(async () => {
  useCombatFx.getState().update({ enabled: true });
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.clearAllMocks();
  document.body.innerHTML = '';
  frames = new Map();
  frameId = 0;
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { frames.set(++frameId, cb); return frameId; });
  vi.stubGlobal('cancelAnimationFrame', (id: number) => frames.delete(id));
  preference = Object.assign(new EventTarget(), { matches: false }) as MediaQueryList;
  vi.spyOn(window, 'matchMedia').mockReturnValue(preference);
  // A frame for the hover alone clips the map canvas before it draws (the draw itself is mocked): give that canvas a context to clip.
  const clip = { save() {}, restore() {}, setTransform() {}, beginPath() {}, rect() {}, clip() {} } as unknown as CanvasRenderingContext2D;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(function (this: HTMLCanvasElement) { return this.classList.contains('map-canvas') ? clip : null; });
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo');
  setup.rule_experiments = { ...setup.rule_experiments, start_settlers: 1 };
  await client.newGame(setup);
  const view = client.view(0);
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ client, humans: [0], views: [view], orders: [[]], seat: 0, selection: { kind: 'unit', id: view.units[0].id }, focus: { hex: { q: 10, r: 10 }, seq: 1 } });
  const host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(createElement(MapCanvas)));
  map = host.querySelector('canvas')!;
  frame(performance.now());
});
afterEach(() => {
  act(() => root.unmount());
  useCombatFx.getState().update({ enabled: true });
  cancelFeedback();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it.each([
  ['legendary', false], ['legendary', true], ['major', false],
] as const)('%s camera reveal respects reduced motion (%s) and leaves other tiers stationary', async (tier, reduced) => {
  Object.assign(preference, { matches: reduced });
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
    if (this.matches('.map-canvas, .map-wrap, .game')) return new DOMRect(0, 0, 1440, 900);
    if (this.matches('.moment-lane')) return new DOMRect(384, 72, 704, 414);
    return new DOMRect();
  });
  await act(async () => {
    await celebrationFixture(tier);
    root.render(createElement(CelebrationProvider, null, createElement('div', { className: 'game' },
      createElement(MapCanvas), createElement(MomentLane))));
  });
  const at = useGame.getState().report!.moments![0][0].at!;
  const world = hexToPixel(at, HEX_SIZE);
  const start = { ...camera() };
  frame(performance.now());
  if (tier === 'legendary' && reduced) expect(camera().y).toBeLessThan(start.y);
  else expect(camera()).toEqual(start);
  frame(performance.now() + 400);
  if (tier === 'major') expect(camera()).toEqual(start);
  else {
    // The 1440px banner ends at y=486. The full tile plus its label clears that edge.
    const y = (world.y - camera().y) * camera().zoom + 450;
    expect(y - HEX_SIZE * camera().zoom).toBeGreaterThan(486);
    expect(camera().x).toBeCloseTo(start.x);
  }
});

it.each(['normal', 'ranged'] as const)('forecasts the Archer attack that right-click submits in %s mode', (mode) => {
  vi.useFakeTimers();
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  for (const r of state.diplomacy?.relations ?? []) r.state = 'War';
  const archer = state.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  const defender = state.units.find((u) => u.owner === 2 && u.kind === 'Spearman')!;
  state.units = [archer, defender];
  state.terrain.fill('Grassland');
  Object.assign(archer, { kind: 'Archer', hp: 20, pos: { q: 5, r: 5 } });
  defender.pos = { q: 6, r: 5 };
  const forecast = vi.spyOn(client, 'forecast');
  act(() => useGame.setState({ views: [client.view(0)], mode, selection: { kind: 'unit', id: archer.id } }));
  const { x, y } = (window as unknown as { legatumMap: { toScreen(h: { q: number; r: number }): { x: number; y: number } } }).legatumMap.toScreen(defender.pos);
  act(() => map.dispatchEvent(new PointerEvent('pointermove', { clientX: x, clientY: y, bubbles: true })));
  expect(forecast).toHaveBeenLastCalledWith(0, archer.id, defender.pos, mode === 'ranged' ? 'Ranged' : 'Melee');
  const shown = forecast.mock.results.at(-1)!.value;
  expect(shown.expected_damage_to_attacker).toBe(mode === 'ranged' ? 0 : 20);
  expect(shown.loss_chance).toBe(mode === 'ranged' ? 0 : 100);
  expect(document.querySelector('.forecast-kind')?.textContent).toBe(mode === 'ranged' ? 'Ranged strike' : 'Attack');
  act(() => map.dispatchEvent(new MouseEvent('contextmenu', { clientX: x, clientY: y, bubbles: true, cancelable: true })));
  expect(useGame.getState().orders[0]).toEqual([mode === 'ranged'
    ? { type: 'RangedAttack', unit: archer.id, target: defender.pos }
    : { type: 'MoveUnit', unit: archer.id, to: defender.pos }]);
});

it('arrow panning leaves text entry alone, pans from plain buttons, and stops a held key when focus or an overlay changes', () => {
  const input = document.body.appendChild(document.createElement('input'));
  const start = { ...camera() };
  expect(key(input, 'ArrowRight').defaultPrevented).toBe(false);
  frame(performance.now());
  expect(camera()).toEqual(start);
  // A clicked button keeps focus in most browsers: arrows still pan from it (the user's report).
  const button = document.body.appendChild(document.createElement('button'));
  expect(key(button, 'ArrowRight').defaultPrevented).toBe(true);
  expect(key(map, 'ArrowRight').defaultPrevented).toBe(true);
  frame(performance.now());
  expect(camera().x).toBeGreaterThan(start.x);
  act(() => input.focus());
  const stopped = { ...camera() };
  frame(performance.now());
  expect(camera()).toEqual(stopped);
  key(map, 'ArrowRight');
  const menu = document.body.appendChild(document.createElement('div'));
  menu.dataset.gameOverlay = '';
  frame(performance.now());
  expect(camera()).toEqual(stopped);
  expect(key(map, 'ArrowDown').defaultPrevented).toBe(false);
});

it('reacts to reduced motion by snapping camera travel, using final unit positions and a stationary ring', () => {
  const target = { q: 15, r: 12 };
  const view = useGame.getState().views![0];
  const unit = view.units[0];
  const time = performance.now();
  act(() => useGame.setState({
    focus: { hex: target, seq: 2 },
    anim: { start: time, duration: 1000, paths: new Map([[unit.id, [{ q: 0, r: 0 }, unit.pos]]]) },
  }));
  frame(time + 100);
  expect(vi.mocked(draw).mock.lastCall![1].unitPos.size).toBeGreaterThan(0);
  act(() => {
    Object.assign(preference, { matches: true });
    preference.dispatchEvent(new Event('change'));
  });
  frame(time + 150);
  const input = vi.mocked(draw).mock.lastCall![1];
  expect(camera()).toMatchObject(hexToPixel(target, HEX_SIZE));
  expect(input.unitPos.size).toBe(0); // Renderer falls back to the new view's final unit positions.
  expect(input.time).toBe(0);
  const draws = vi.mocked(draw).mock.calls.length;
  frame(time + 300);
  expect(vi.mocked(draw).mock.calls.length).toBe(draws); // No selection pulse while idle.
});

it('snaps a running camera focus and map playback when Map animations is switched Off', () => {
  const target = { q: 15, r: 12 };
  const view = useGame.getState().views![0];
  const unit = view.units[0];
  const time = performance.now();
  act(() => useGame.setState({
    focus: { hex: target, seq: 2 },
    anim: { start: time, duration: 1000, paths: new Map([[unit.id, [{ q: 0, r: 0 }, unit.pos]]]) },
  }));
  frame(time + 100);
  expect(vi.mocked(draw).mock.lastCall![1].unitPos.size).toBeGreaterThan(0);
  act(() => useCombatFx.getState().update({ enabled: false }));
  frame(time + 150);
  expect(camera()).toMatchObject(hexToPixel(target, HEX_SIZE));
  expect(vi.mocked(draw).mock.lastCall![1].unitPos.size).toBe(0);
});

it.each(['normal', 'ranged'] as const)('disables right-click commands after the game ends in %s mode', (mode) => {
  const edit = vi.fn();
  act(() => useGame.setState({ mode, editOrders: edit, views: useGame.getState().views!.map((v) => ({ ...v, finished: true })) }));
  act(() => map.dispatchEvent(new MouseEvent('contextmenu', { clientX: 450, clientY: 300, bubbles: true, cancelable: true })));
  expect(edit).not.toHaveBeenCalled();
  expect(useGame.getState().orders).toEqual([[]]);
});

it.each([false, true])('disables path dragging when the game ends before or during the drag (%s)', (duringDrag) => {
  const view = useGame.getState().views![0];
  const unit = view.units.find((u) => u.owner === view.me)!;
  const edit = vi.fn();
  const finished = () => useGame.setState({ views: [{ ...view, finished: true }] });
  act(() => {
    useGame.setState({ editOrders: edit, selection: { kind: 'unit', id: unit.id } });
    if (!duringDrag) finished();
  });
  Object.defineProperty(map, 'setPointerCapture', { value: vi.fn() });
  const { x, y } = (window as unknown as { legatumMap: { toScreen(h: { q: number; r: number }): { x: number; y: number } } }).legatumMap.toScreen(unit.pos);
  act(() => map.dispatchEvent(new PointerEvent('pointerdown', { clientX: x, clientY: y, button: 0, pointerId: 1, bubbles: true })));
  act(() => map.dispatchEvent(new PointerEvent('pointermove', { clientX: x + 80, clientY: y, pointerId: 1, bubbles: true })));
  if (duringDrag) act(finished);
  act(() => map.dispatchEvent(new PointerEvent('pointerup', { clientX: x + 80, clientY: y, pointerId: 1, bubbles: true })));
  expect(edit).not.toHaveBeenCalled();
  expect(useGame.getState().orders).toEqual([[]]);
});

function touch(type: string, hex: { q: number; r: number }, id = 1, offset = { x: 0, y: 0 }) {
  const point = (window as unknown as { legatumMap: { toScreen(h: typeof hex): { x: number; y: number } } }).legatumMap.toScreen(hex);
  act(() => map.dispatchEvent(new PointerEvent(type, { clientX: point.x + offset.x, clientY: point.y + offset.y, pointerId: id, pointerType: 'touch', button: 0, bubbles: true, cancelable: true })));
}
function tap(hex: { q: number; r: number }) { touch('pointerdown', hex); touch('pointerup', hex); }
/** A tile beside the unit with none of the player's own units or cities: tapping it previews a move. */
function emptyBeside(unit: { pos: { q: number; r: number } }, step = 1) {
  const v = useGame.getState().views![0];
  for (const [dq, dr] of [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]]) {
    const h = { q: unit.pos.q + dq * step, r: unit.pos.r + dr * step };
    if (!v.units.some((u) => u.pos.q === h.q && u.pos.r === h.r) && !v.cities.some((c) => c.pos.q === h.q && c.pos.r === h.r)) return h;
  }
  throw new Error('no empty tile beside the unit');
}
function capture() { Object.defineProperty(map, 'setPointerCapture', { value: vi.fn(), configurable: true }); }

function settlerAtLegalSite() {
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  state.terrain.fill('Grassland');
  state.players.forEach((player) => player.explored.fill(1));
  const start = client.view(0);
  const site = start.tiles.map((_, i) => hexAt(i, start.width)).find((hex) => knownLegalSite(start, hex))!;
  state.units.find((u) => u.owner === 0 && u.kind === 'Settler')!.pos = site;
  const view = client.view(0);
  act(() => useGame.setState({ views: [view] }));
  return { view, settler: view.units.find((u) => u.owner === view.me && u.kind === 'Settler')! };
}

function selectSpearmanByCity() {
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  state.terrain.fill('Grassland');
  client.endTurn({});
  const founded = client.debugState();
  const city = founded.cities.find((c) => c.owner === 0)!;
  const unit = founded.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  unit.pos = neighbors(city.pos).find((h) => tileIndex(h, founded.width, founded.height) >= 0 && !founded.units.some((u) => hexEq(u.pos, h)))!;
  act(() => useGame.setState({ views: [client.view(0)], selection: { kind: 'unit', id: unit.id } }));
  return { client, unit, city };
}

it.each(['unit', 'city'] as const)('phone taps select a %s without a tile card; holding or Inspect opens it without orders', (kind) => {
  vi.useFakeTimers(); capture();
  const { unit, city } = selectSpearmanByCity();
  act(() => {
    Object.assign(preference, { matches: true });
    preference.dispatchEvent(new Event('change'));
    useGame.getState().select(null);
  });
  const target = kind === 'unit' ? unit : city;
  tap(target.pos);
  expect(useGame.getState().selection).toEqual({ kind, id: target.id });
  expect(document.querySelector('.map-touch-card')).toBeNull();
  touch('pointerdown', target.pos);
  act(() => vi.advanceTimersByTime(600));
  touch('pointerup', target.pos);
  expect(document.querySelector('[aria-label="Tile actions"]')).not.toBeNull();
  expect(useGame.getState().orders[0]).toEqual([]);
  act(() => (document.querySelector('[aria-label="Close tile actions"]') as HTMLButtonElement).click());
  expect(document.querySelector('.map-touch-card')).toBeNull();
  act(() => root.render(createElement(MapCanvas, { inspectRequest: { ...target.pos } })));
  expect(document.querySelector('[aria-label="Tile actions"]')).not.toBeNull();
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('defers map resize writes until the next frame and ignores unchanged canvas dimensions', () => {
  let notify: ResizeObserverCallback | undefined;
  vi.stubGlobal('ResizeObserver', class {
    constructor(private callback: ResizeObserverCallback) {}
    observe(el: Element) { if (el.matches('.map-wrap')) notify = this.callback; }
    disconnect() {}
  });
  let height = 600;
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
    return this.matches('.map-wrap') ? new DOMRect(0, 0, 800, height) : new DOMRect();
  });
  act(() => root.render(createElement(MapCanvas, { key: 'resize-regression' })));
  const canvas = document.querySelector<HTMLCanvasElement>('[data-game-map]')!;
  const before = canvas.height;
  const writes = vi.spyOn(canvas, 'height', 'set');
  height = 420;
  act(() => { notify!([], {} as ResizeObserver); notify!([], {} as ResizeObserver); });
  expect(canvas.height).toBe(before); // No writes inside observer delivery.
  frame(performance.now());
  expect(canvas.style.height).toBe('420px');
  expect(writes).toHaveBeenCalledTimes(1);
  act(() => notify!([], {} as ResizeObserver));
  frame(performance.now());
  expect(writes).toHaveBeenCalledTimes(1);
});

it('touch previews the path without orders, then tap-tap or Confirm uses the desktop command', () => {
  vi.useFakeTimers(); capture();
  // Open ground, so the generated start's terrain cannot block the second target.
  const client = useGame.getState().client as FixtureClient;
  client.debugState().terrain.fill('Grassland');
  act(() => useGame.setState({ views: [client.view(0)] }));
  const unit = useGame.getState().views![0].units[0];
  const target = emptyBeside(unit);
  tap(target);
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(document.querySelector('.confirm-order')?.textContent).toBe('Confirm move');
  frame(performance.now());
  expect(vi.mocked(draw).mock.lastCall![1].path?.length).toBeGreaterThan(0);
  tap(target);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnit', unit: unit.id, to: target }]);
  expect(document.querySelector('.confirm-order')).toBeNull();
  const other = emptyBeside(unit, 2);
  tap(other);
  act(() => vi.advanceTimersByTime(20));
  frame(performance.now() + 100);
  const confirmedPath = vi.mocked(draw).mock.lastCall![1].path!.map((step) => step.hex);
  act(() => (document.querySelector('.confirm-order') as HTMLButtonElement).click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnitPath', unit: unit.id, to: other, path: confirmedPath }]);
});

it.each(['city', 'territory'] as const)('touch previews and confirms a Spearman move into empty owned %s', (destination) => {
  vi.useFakeTimers(); capture();
  const { unit, city } = selectSpearmanByCity();
  const view = useGame.getState().views![0];
  const target = destination === 'city' ? city.pos : neighbors(city.pos).find((h) =>
    view.tiles[tileIndex(h, view.width, view.height)]?.owner === view.me && !view.units.some((u) => hexEq(u.pos, h)))!;
  expect(view.tiles[tileIndex(target, view.width, view.height)].owner).toBe(view.me);
  expect(view.units.some((u) => hexEq(u.pos, target))).toBe(false);
  tap(target);
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: unit.id });
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(document.querySelector('.confirm-order')?.textContent).toBe('Confirm move');
  frame(performance.now());
  expect(vi.mocked(draw).mock.lastCall![1].path?.at(-1)?.hex).toEqual(target);
  tap(target);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnit', unit: unit.id, to: target }]);
  expect(useGame.getState().errors[0]).toEqual([]);
  expect(document.querySelector('.confirm-order')).toBeNull();
});

it.each([false, true])('long-press on an empty owned city offers a garrison move without committing (preview already open: %s)', (previewOpen) => {
  vi.useFakeTimers(); capture();
  const { unit, city } = selectSpearmanByCity();
  if (previewOpen) tap(city.pos);
  touch('pointerdown', city.pos);
  act(() => vi.advanceTimersByTime(600));
  expect(document.querySelector('[aria-label="Tile actions"]')).not.toBeNull();
  expect(document.querySelector('.confirm-order')?.textContent).toBe('Confirm move');
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: unit.id });
  expect(useGame.getState().orders[0]).toEqual([]);
  act(() => map.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true })));
  touch('pointerup', city.pos);
  expect(useGame.getState().orders[0]).toEqual([]);
  act(() => (document.querySelector('.confirm-order') as HTMLButtonElement).click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnit', unit: unit.id, to: city.pos }]);
  expect(useGame.getState().errors[0]).toEqual([]);
});

it.each([false, true])('touch still selects another own unit instead of previewing a move (in city: %s)', (inCity) => {
  vi.useFakeTimers(); capture();
  const { client, city } = selectSpearmanByCity();
  const other = client.debugState().units.find((u) => u.owner === 0 && u.kind === 'Scout')!;
  if (inCity) other.pos = city.pos;
  act(() => useGame.setState({ views: [client.view(0)] }));
  tap(other.pos);
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: other.id });
  expect(document.querySelector('.confirm-order')).toBeNull();
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('touch still selects an empty owned city when no unit is selected', () => {
  vi.useFakeTimers(); capture();
  const { city } = selectSpearmanByCity();
  act(() => useGame.getState().select(null));
  tap(city.pos);
  expect(useGame.getState().selection).toEqual({ kind: 'city', id: city.id });
  expect(document.querySelector('.confirm-order')).toBeNull();
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('desktop hover keeps the city selection affordance and right-click still orders a garrison move', () => {
  vi.useFakeTimers();
  const { unit, city } = selectSpearmanByCity();
  const { x, y } = (window as unknown as { legatumMap: { toScreen(h: typeof city.pos): { x: number; y: number } } }).legatumMap.toScreen(city.pos);
  act(() => map.dispatchEvent(new PointerEvent('pointermove', { clientX: x, clientY: y, pointerType: 'mouse', bubbles: true })));
  frame(performance.now());
  expect(vi.mocked(draw).mock.lastCall![1].hoverSelect).toBe(true);
  expect(vi.mocked(draw).mock.lastCall![1].path).toBeNull();
  expect(useGame.getState().orders[0]).toEqual([]);
  act(() => map.dispatchEvent(new MouseEvent('contextmenu', { clientX: x, clientY: y, bubbles: true, cancelable: true })));
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnit', unit: unit.id, to: city.pos }]);
  expect(useGame.getState().errors[0]).toEqual([]);
});

it('shows a tile card from mousemove when no pointermove arrives', () => {
  vi.useFakeTimers();
  act(() => useGame.getState().select(null));
  const unit = useGame.getState().views![0].units[0];
  const { x, y } = (window as unknown as { legatumMap: { toScreen(h: typeof unit.pos): { x: number; y: number } } }).legatumMap.toScreen(unit.pos);
  act(() => map.dispatchEvent(new MouseEvent('mousemove', { clientX: x, clientY: y, bubbles: true })));
  act(() => vi.advanceTimersByTime(700));
  expect(document.querySelector('.tilecard')).not.toBeNull();
});

it.each(['normal', 'ranged'] as const)('touch previews the same combat forecast as right-click in %s mode', (mode) => {
  vi.useFakeTimers(); capture();
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  for (const r of state.diplomacy?.relations ?? []) r.state = 'War';
  const archer = state.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  const defender = state.units.find((u) => u.owner === 2 && u.kind === 'Spearman')!;
  state.units = [archer, defender]; state.terrain.fill('Grassland');
  Object.assign(archer, { kind: 'Archer', pos: { q: 5, r: 5 } }); defender.pos = { q: 6, r: 5 };
  const forecast = vi.spyOn(client, 'forecast');
  act(() => useGame.setState({ views: [client.view(0)], mode, selection: { kind: 'unit', id: archer.id } }));
  tap(defender.pos);
  expect(forecast).toHaveBeenLastCalledWith(0, archer.id, defender.pos, mode === 'ranged' ? 'Ranged' : 'Melee');
  expect(document.querySelector('.forecast-kind')?.textContent).toBe(mode === 'ranged' ? 'Ranged strike' : 'Attack');
  expect(useGame.getState().orders[0]).toEqual([]);
  tap(defender.pos);
  expect(useGame.getState().orders[0]).toEqual([mode === 'ranged' ? { type: 'RangedAttack', unit: archer.id, target: defender.pos } : { type: 'MoveUnit', unit: archer.id, to: defender.pos }]);
});

it('long-press opens actions without an order, including the native contextmenu and release', () => {
  vi.useFakeTimers(); capture();
  const unit = useGame.getState().views![0].units[0];
  const target = emptyBeside(unit);
  touch('pointerdown', target);
  act(() => vi.advanceTimersByTime(600));
  expect(document.querySelector('[aria-label="Tile actions"]')).not.toBeNull();
  act(() => map.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true })));
  touch('pointerup', target);
  expect(useGame.getState().orders[0]).toEqual([]);
  act(() => (document.querySelector('.confirm-order') as HTMLButtonElement).click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnit', unit: unit.id, to: target }]);
});

it('canceled touch and lost capture never select or order, and selection changes discard previews', () => {
  vi.useFakeTimers(); capture();
  const before = useGame.getState().selection;
  const unit = useGame.getState().views![0].units[0];
  const target = emptyBeside(unit);
  touch('pointerdown', target); touch('pointercancel', target); touch('pointerup', target);
  act(() => vi.advanceTimersByTime(600));
  expect(useGame.getState().selection).toEqual(before);
  expect(useGame.getState().orders[0]).toEqual([]);
  touch('pointerdown', target); touch('lostpointercapture', target); touch('pointerup', target);
  expect(document.querySelector('.confirm-order')).toBeNull();
  tap(target);
  act(() => useGame.getState().select(null));
  expect(document.querySelector('.confirm-order')).toBeNull();
});

it('double-tap zooms on a selected unit without ordering, and a subsequent touch drag only pans', () => {
  vi.useFakeTimers(); capture();
  const unit = useGame.getState().views![0].units[0];
  const zoom = camera().zoom;
  tap(unit.pos); tap(unit.pos);
  expect(camera().zoom).toBeCloseTo(zoom * 1.5);
  expect(useGame.getState().orders[0]).toEqual([]);
  const start = { ...camera() };
  touch('pointerdown', unit.pos);
  touch('pointermove', unit.pos, 1, { x: 100, y: 0 });
  touch('pointerup', unit.pos);
  expect(camera().x).not.toBe(start.x);
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('a moving pinch captures both fingers and never commits when either is lifted', () => {
  vi.useFakeTimers(); capture();
  const unit = useGame.getState().views![0].units[0];
  touch('pointerdown', unit.pos, 1, { x: -50, y: 0 });
  touch('pointerdown', unit.pos, 2, { x: 50, y: 0 });
  expect(map.setPointerCapture).toHaveBeenCalledTimes(2);
  touch('pointermove', unit.pos, 2, { x: 100, y: 0 });
  act(() => vi.advanceTimersByTime(600));
  touch('pointerup', unit.pos, 1); touch('pointerup', unit.pos, 2);
  expect(document.querySelector('.map-touch-card')).toBeNull();
  expect(useGame.getState().orders[0]).toEqual([]);
});

function chooseLens(name: string) {
  const button = [...document.querySelectorAll<HTMLButtonElement>('[role="radio"]')].find((b) => b.textContent === name)!;
  act(() => button.click());
}
const chosenLens = () => document.querySelector('[role="radio"][aria-checked="true"]')?.textContent;

it('auto-enables Settle temporarily, remembers explicit choices, and restores them after leaving a Settler', () => {
  const v = useGame.getState().views![0];
  const settler = v.units.find((u) => u.owner === v.me && u.kind === 'Settler')!;
  const spearman = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
  act(() => useGame.setState({ selection: { kind: 'unit', id: spearman.id } }));
  chooseLens('Movement');
  // An overlay the player turned on rides through every automatic switch.
  act(() => setOverlay('yields', true));
  act(() => useGame.setState({ selection: { kind: 'unit', id: settler.id } }));
  expect(chosenLens()).toBe('Settle');
  frame(performance.now());
  expect(vi.mocked(draw).mock.lastCall![1].lensTiles!.filter((t) => t.rank)).toHaveLength(3);
  expect(vi.mocked(draw).mock.lastCall![1].overlays).toMatchObject({ yields: true });
  expect(vi.mocked(draw).mock.lastCall![1].yieldTiles!.length).toBeGreaterThan(0);
  act(() => useGame.setState({ selection: { kind: 'unit', id: spearman.id } }));
  expect(chosenLens()).toBe('Movement');
  expect(document.querySelector('[role="checkbox"][data-overlay="yields"]')?.getAttribute('aria-checked')).toBe('true');
  act(() => useGame.setState({ selection: { kind: 'unit', id: settler.id } }));
  chooseLens('Threat');
  act(() => useGame.setState({ selection: null }));
  expect(chosenLens()).toBe('Threat');
  act(() => setOverlay('yields', false));
});

it('lens arrow keys select and focus the next radio without panning or ending the turn', () => {
  chooseLens('Default');
  const selected = document.querySelector<HTMLElement>('[role="radio"][aria-checked="true"]')!;
  const before = { ...camera() };
  expect(key(selected, 'ArrowRight').defaultPrevented).toBe(true);
  expect(chosenLens()).toBe('Settle');
  expect(document.activeElement?.textContent).toBe('Settle');
  frame(performance.now());
  expect(camera()).toEqual(before);
  expect(useGame.getState().views![0].turn).toBe(1);
});

it('A4 draws selection without even a queued route when hovering another own unit', () => {
  const v = useGame.getState().views![0];
  const spearman = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
  const settler = v.units.find((u) => u.owner === v.me && u.kind === 'Settler')!;
  const order = { type: 'MoveUnit' as const, unit: spearman.id, to: emptyBeside(spearman, 2) };
  act(() => useGame.setState({ selection: { kind: 'unit', id: spearman.id }, orders: [[order]] }));
  const { x, y } = (window as unknown as { legatumMap: { toScreen(h: typeof settler.pos): { x: number; y: number } } }).legatumMap.toScreen(settler.pos);
  act(() => map.dispatchEvent(new PointerEvent('pointermove', { clientX: x, clientY: y, bubbles: true })));
  frame(performance.now());
  const rendered = vi.mocked(draw).mock.lastCall![1];
  expect(rendered.hoverSelect).toBe(true);
  expect(rendered.path).toBeNull();
  expect(rendered.pathUnreachable).toBeNull();
  expect(useGame.getState().orders[0]).toEqual([order]);
});

function mouse(type: string, hex: { q: number; r: number }) {
  const point = (window as unknown as { legatumMap: { toScreen(h: typeof hex): { x: number; y: number } } }).legatumMap.toScreen(hex);
  act(() => map.dispatchEvent(new PointerEvent(type, { clientX: point.x, clientY: point.y, pointerId: 1, pointerType: 'mouse', button: 0, bubbles: true, cancelable: true })));
}

it('keeps a mouse drag route when an escort targets a friendly Settler tile', () => {
  capture();
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  state.terrain.fill('Grassland');
  const settler = state.units.find((u) => u.owner === 0 && u.kind === 'Settler')!;
  const spearman = state.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  spearman.pos = neighbors(settler.pos).find((h) => tileIndex(h, state.width, state.height) >= 0)!;
  act(() => useGame.setState({ views: [client.view(0)], selection: { kind: 'unit', id: spearman.id } }));
  mouse('pointerdown', spearman.pos);
  mouse('pointermove', settler.pos);
  frame(performance.now());
  expect(vi.mocked(draw).mock.lastCall![1].path?.at(-1)?.hex).toEqual(settler.pos);
  mouse('pointerup', settler.pos);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'MoveUnit', unit: spearman.id, to: settler.pos }]);
});

it('keeps the settlement card route over friendly units and gives mouse-specific confirmation copy', () => {
  capture();
  const v = useGame.getState().views![0];
  const settler = v.units.find((u) => u.owner === v.me && u.kind === 'Settler')!;
  const spearman = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
  act(() => useGame.setState({ selection: { kind: 'unit', id: settler.id } }));
  const recommendations = [...document.querySelectorAll<HTMLButtonElement>('[data-settle-rank]')];
  for (const button of recommendations) {
    act(() => button.click());
    if ([...document.querySelectorAll('button')].some((b) => b.textContent === 'Preview move to this site')) break;
  }
  const preview = [...document.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent === 'Preview move to this site')!;
  expect(preview).toBeTruthy();
  act(() => preview.click());
  frame(performance.now());
  const path = vi.mocked(draw).mock.lastCall![1].path;
  expect(path?.length).toBeGreaterThan(0);
  expect(document.querySelector('.map-touch-card')?.textContent).toContain('right-click to move here');
  mouse('pointermove', spearman.pos);
  frame(performance.now());
  expect(vi.mocked(draw).mock.lastCall![1].path).toEqual(path);
});

it('fetches a breakdown only for an inspected site, caches it across hover movement, and keeps hover compact', () => {
  vi.useFakeTimers();
  const { view: v, settler } = settlerAtLegalSite();
  const site = { ...settlementSite(v, settler, settler.pos, [])!, score: 100, breakdown: [{ label: 'Food', value: 120 }, { label: 'Travel', value: -20 }, { label: 'No effect', value: 0 }] };
  const detail = vi.spyOn(Game, 'settlement_site').mockReturnValue(JSON.stringify(site));
  act(() => useGame.setState({ selection: { kind: 'unit', id: settler.id } }));
  expect(detail).not.toHaveBeenCalled();
  mouse('pointermove', settler.pos);
  act(() => vi.advanceTimersByTime(700));
  expect(detail).toHaveBeenCalledTimes(1);
  expect(document.querySelector('.settlement-card')?.textContent).toContain('Food');
  expect(document.querySelector('.settlement-card')?.textContent).not.toContain('No effect');
  // The hover leads with the absolute band and its number, and never the full calculation.
  expect(document.querySelector('.settlement-card')?.textContent).toMatch(/^(Poor|Fair|Good|Great|Exceptional) spot for a city: /);
  expect(document.querySelector('.settlement-math')).toBeNull();
  mouse('pointermove', settler.pos);
  expect(detail).toHaveBeenCalledTimes(1);
});

it('Settle tints exactly the tiles a city could be founded on, and a tile it leaves plain says why', () => {
  vi.useFakeTimers();
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  const settler = state.units.find((u) => u.owner === 0 && u.kind === 'Settler')!;
  // Foreign land, a wonder and the sea next to the Settler, all explored.
  const foreign = neighbors(neighbors(settler.pos)[0]).find((h) => !hexEq(h, settler.pos) && tileIndex(h, state.width, state.height) >= 0)!;
  const fi = tileIndex(foreign, state.width, state.height);
  state.owner[fi] = 1; state.terrain[fi] = 'Grassland';
  act(() => useGame.setState({ views: [client.view(0)], selection: { kind: 'unit', id: settler.id } }));
  const view = useGame.getState().views![0];
  frame(performance.now());
  const tiles = vi.mocked(draw).mock.lastCall![1].lensTiles!;
  const legal = view.tiles.map((_, i) => i).filter((i) => knownLegalSite(view, hexAt(i, view.width)));
  expect(legal.length).toBeGreaterThan(20);
  expect(tiles.filter((t) => t.band).map((t) => t.index).sort((a, b) => a - b)).toEqual(legal);
  expect(tiles.some((t) => t.index === tileIndex(foreign, view.width, view.height))).toBe(false);
  // Hovering the plain foreign tile says why, in the tile card.
  mouse('pointermove', foreign);
  act(() => vi.advanceTimersByTime(700));
  expect(document.querySelector('[data-testid="tile-note"]')?.textContent).toMatch(/^Cannot settle here: Foreign territory \(/);
});

it.each(['mouse', 'touch'] as const)('inspects an occupied scored tile by %s without changing selection or issuing orders', (pointer) => {
  capture();
  const { settler } = settlerAtLegalSite();
  act(() => useGame.setState({ selection: { kind: 'unit', id: settler.id } }));
  if (pointer === 'touch') tap(settler.pos);
  else { mouse('pointerdown', settler.pos); mouse('pointerup', settler.pos); }
  const card = document.querySelector('.map-touch-card')!;
  expect(card.querySelector('[data-settle-band]')?.textContent).toMatch(/^(?:Poor|Fair|Good|Great|Exceptional) spot for a city: /);
  expect(card.querySelector('.settlement-reasons li')?.textContent).toMatch(/^Nearby land: /);
  expect(card.querySelector('.settlement-route')?.textContent).toMatch(/turn|settle here now/i);
  const breakdown = card.querySelector<HTMLDetailsElement>('.settlement-breakdown')!;
  // The arithmetic is behind a Details toggle, closed for mouse and touch alike.
  expect(breakdown.open).toBe(false);
  expect(breakdown.querySelector('summary')?.textContent).toBe('Details');
  act(() => breakdown.querySelector('summary')!.click());
  expect(breakdown.open).toBe(true);
  expect(breakdown.textContent).toContain('Site quality:');
  expect(breakdown.textContent).toContain('fixed scale');
  expect(breakdown.querySelector('[aria-label="Site quality calculation"]')).not.toBeNull();
  expect(breakdown.querySelector('[aria-label="Suitability calculation"]')).not.toBeNull();
  expect(useGame.getState().selection).toEqual({ kind: 'unit', id: settler.id });
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('a ranged click on an enemy fires even when the Settle lens marks that tile as a site', () => {
  capture();
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  state.terrain.fill('Grassland');
  const settler = state.units.find((u) => u.owner === 0 && u.kind === 'Settler')!;
  const archer = state.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  const defender = state.units.find((u) => u.owner === 2 && u.kind === 'Spearman')!;
  state.units = [settler, archer, defender];
  Object.assign(archer, { kind: 'Archer', pos: { q: 12, r: 10 } }); defender.pos = { q: 13, r: 10 };
  settler.pos = { q: 10, r: 10 };
  for (const r of state.diplomacy?.relations ?? []) r.state = 'War';
  // The enemy stands on a tile the Settle lens scores as a site (as on neutral land it can).
  (client as unknown as { settlementSites: () => { pos: typeof defender.pos; score: number; rank: number }[] })
    .settlementSites = () => [{ pos: defender.pos, score: 30, rank: 1 }];
  act(() => useGame.setState({ views: [client.view(0)], selection: { kind: 'unit', id: archer.id } }));
  chooseLens('Settle');
  act(() => useGame.setState({ mode: 'ranged' }));
  mouse('pointerdown', defender.pos);
  mouse('pointerup', defender.pos);
  expect(useGame.getState().orders[0]).toEqual([{ type: 'RangedAttack', unit: archer.id, target: defender.pos }]);
  expect(useGame.getState().errors[0]).toEqual([]);
});

it('a ranged click on a unit of a team at Peace asks to declare War instead of queueing an order that fails', () => {
  capture();
  const client = useGame.getState().client as FixtureClient;
  const state = client.debugState();
  state.terrain.fill('Grassland');
  const archer = state.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  const defender = state.units.find((u) => u.owner === 2 && u.kind === 'Spearman')!;
  state.units = [archer, defender];
  Object.assign(archer, { kind: 'Archer', pos: { q: 12, r: 10 } }); defender.pos = { q: 13, r: 10 };
  act(() => useGame.setState({ views: [client.view(0)], selection: { kind: 'unit', id: archer.id }, mode: 'ranged' }));
  mouse('pointerdown', defender.pos);
  mouse('pointerup', defender.pos);
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(useGame.getState().warPrompt).toMatchObject({ unit: archer.id, target: defender.pos, ranged: true });
});


it('the unit card movement picker accepts a left-click through the existing validated move command', () => {
  const v = useGame.getState().views![0];
  const spearman = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
  const destination = emptyBeside(spearman, 2);
  act(() => useGame.setState({ selection: { kind: 'unit', id: spearman.id }, mode: 'move' }));
  mouse('pointermove', destination);
  mouse('pointerdown', destination);
  mouse('pointerup', destination);
  expect(useGame.getState().orders[0]).toContainEqual(expect.objectContaining({ unit: spearman.id, to: destination }));
  expect(useGame.getState().errors[0]).toEqual([]);
  expect(useGame.getState().mode).toBe('normal');
});
