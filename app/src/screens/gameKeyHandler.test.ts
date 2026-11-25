// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useGame } from '../store';
import { useGuide } from '../ui/guideState';
import { readyAndPass } from '../ui/SeatBar';
import { toggleAutoExploreSelected, cancelSelected, disbandSelected, foundCitySelected, sleepSelected, healSelected, skipSelected, toggleRangedMode, upgradeSelected } from '../commands';
import type { PlayerView } from '../types';
import { gameplayKeyAllowed, gameplayFocusAllowed, handleOverlayKey, notePointerDown, returnFocusAfterPointer } from '../gameplayKeys';
import { handleAircraftKey, handleGameKey } from './gameKeyHandler';
import { FixtureClient } from '../test/fixtureEngine';
import { turnFlowFixture } from '../test/turnFlowFixture';
import { getOverlays, reloadOverlays } from '../map/overlays';

vi.mock('../ui/SeatBar', () => ({ readyAndPass: vi.fn() }));
vi.mock('../commands', () => ({
  toggleAutoExploreSelected: vi.fn(), cancelSelected: vi.fn(), disbandSelected: vi.fn(), foundCitySelected: vi.fn(),
  sleepSelected: vi.fn(), healSelected: vi.fn(), skipSelected: vi.fn(), toggleRangedMode: vi.fn(), upgradeSelected: vi.fn(),
}));

const nextDecision = vi.fn();
const select = vi.fn();
const commands = [toggleAutoExploreSelected, nextDecision, select, readyAndPass, cancelSelected, disbandSelected, foundCitySelected, sleepSelected, healSelected, skipSelected, toggleRangedMode, upgradeSelected];
let map: HTMLCanvasElement;

beforeEach(() => {
  vi.clearAllMocks();
  document.body.innerHTML = '<canvas data-game-map tabindex="0"></canvas>';
  map = document.querySelector('canvas')!;
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient(); client.load(JSON.stringify(turnFlowFixture()));
  useGame.setState({ client, views: [client.view(0)], humans: [0], orders: [[]], errors: [[]], ready: [false], nextDecision, select });
  useGuide.getState().close();
  window.addEventListener('keydown', handleAircraftKey, true);
  window.addEventListener('keydown', handleGameKey);
});
afterEach(() => { window.removeEventListener('keydown', handleAircraftKey, true); window.removeEventListener('keydown', handleGameKey); });

function press(target: HTMLElement, key: string, options: KeyboardEventInit = {}) {
  const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...options });
  target.dispatchEvent(event);
  return event;
}
function unchanged() {
  for (const command of commands) expect(command).not.toHaveBeenCalled();
  expect(useGame.getState().techOpen).toBe(false);
}

it('I toggles Economy building automation for the selected owned city', () => {
  const view = useGame.getState().views![0];
  const city = view.cities.find((c) => c.owner === view.me)!;
  useGame.setState({ selection: { kind: 'city', id: city.id } });
  press(map, 'i');
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'SetCityAutomation', city: city.id, focus: 'Economy' });
  press(map, 'I');
  expect(useGame.getState().orders[0]).toContainEqual({ type: 'SetCityAutomation', city: city.id, focus: null });
});

describe('GameScreen keyboard scope', () => {
  it('keeps Return and other orders out of tile placement until Esc cancels', () => {
    const city = useGame.getState().views![0].cities.find((c) => c.owner === useGame.getState().views![0].me)!;
    useGame.setState({ selection: { kind: 'city', id: city.id }, place: { city: city.id, building: 'Pyramids', intent: 'build', pick: null } });
    press(map, 'Enter');
    press(map, ' ');
    expect(nextDecision).not.toHaveBeenCalled();
    expect(readyAndPass).not.toHaveBeenCalled();
    expect(skipSelected).not.toHaveBeenCalled();
    press(map, 'Escape');
    expect(useGame.getState().place).toBeNull();
  });
  it.each([false, true])('never consumes Tab (shift=%s), even on the map or body', (shiftKey) => {
    for (const target of [document.body, map]) expect(press(target, 'Tab', { shiftKey }).defaultPrevented).toBe(false);
    unchanged();
  });

  it.each([
    '<button><span>Activate</span></button>', '<a href="#">Link</a>', '<input>', '<textarea></textarea>',
    '<select><option>One</option></select>', '<div contenteditable="true">Edit</div>',
    '<div tabindex="0">Info</div>', '<div tabindex="-1" role="button">Custom control</div>',
    '<details><summary>Cities</summary></details>',
  ])('leaves native keys and gameplay commands alone on %s', (html) => {
    const host = document.createElement('div');
    host.innerHTML = html;
    document.body.append(host);
    const control = host.querySelector<HTMLElement>('span, summary') ?? host.firstElementChild as HTMLElement;
    for (const key of ['Tab', ' ', 'Enter', 'f', 'h', 'j', 'b', 'r', 'u', 't', '1', 'Delete', 'Backspace', 'X', 'Escape']) {
      expect(press(control, key, { shiftKey: key === 'X' }).defaultPrevented).toBe(false);
    }
    unchanged();
  });

  it.each(['body', 'map'])('Return selects decisions, Shift+Return reviews End Turn, and Space skips on %s', (where) => {
    const target = where === 'map' ? map : document.body;
    expect(press(target, 'Enter').defaultPrevented).toBe(true);
    expect(nextDecision).toHaveBeenCalledOnce();
    expect(readyAndPass).not.toHaveBeenCalled();
    expect(press(target, 'Enter', { shiftKey: true }).defaultPrevented).toBe(true);
    expect(readyAndPass).toHaveBeenCalledOnce();
    expect(press(target, ' ').defaultPrevented).toBe(true);
    expect(skipSelected).toHaveBeenCalledOnce();
  });

  it('aircraft keys act on the selected plane and take precedence over map pan and ground ranged attack', () => {
    const view = useGame.getState().views![0];
    const city = view.cities.find((candidate) => candidate.owner === view.me)!;
    const destination = { ...city, id: 9876, name: 'Emain Macha', pos: { q: city.pos.q + 1, r: city.pos.r } };
    view.cities.push(destination);
    view.aircraft = [{ id: 900, owner: view.me, kind: 'Bomber', city: city.id, hp: 100 }];
    const rival = view.players.find((player) => player.id !== view.me && player.team !== view.team)!;
    const enemy = view.units.find((unit) => unit.owner === rival.id)!;
    view.units.push({ ...enemy, id: 9877, pos: { q: city.pos.q + 1, r: city.pos.r } });
    useGame.setState({ selection: { kind: 'air', id: 900 }, air: { plane: null, what: null, hover: null } });

    expect(press(map, 's').defaultPrevented).toBe(true);
    expect(useGame.getState().air).toMatchObject({ plane: 900, what: 'strike' });
    expect(toggleRangedMode).not.toHaveBeenCalled();
    expect(press(map, 'a').defaultPrevented).toBe(true);
    expect(useGame.getState().air).toMatchObject({ plane: 900, what: 'recon' });
    press(map, 'Escape');
    expect(useGame.getState().air).toMatchObject({ plane: null, what: null });
    expect(useGame.getState().selection).toEqual({ kind: 'air', id: 900 });
    expect(press(map, 'r').defaultPrevented).toBe(true);
    expect(useGame.getState().air).toMatchObject({ plane: 900, what: 'rebase' });
    expect(press(map, 'f').defaultPrevented).toBe(true);
    expect(useGame.getState().airHeld[view.me].asleep).toContain(900);
    expect(useGame.getState().air).toMatchObject({ plane: null, what: null });
    expect(press(map, ' ').defaultPrevented).toBe(true);
    expect(useGame.getState().airHeld[view.me].planes).toEqual([900]);
    expect(skipSelected).not.toHaveBeenCalled();
  });

  it.each(['research', 'guide', 'menu', 'dialog', 'busy', 'finished', 'no game'])('suspends gameplay and panning for %s, even from body/map', (overlay) => {
    if (overlay === 'research') useGame.setState({ techOpen: true });
    if (overlay === 'guide') useGuide.getState().show();
    if (overlay === 'busy') useGame.setState({ busy: 'Resolving…' });
    if (overlay === 'finished') useGame.setState({ views: [{ finished: true }] as PlayerView[] });
    if (overlay === 'no game') useGame.setState({ views: null });
    if (overlay === 'menu') document.body.insertAdjacentHTML('beforeend', '<div data-game-overlay></div>');
    if (overlay === 'dialog') document.body.insertAdjacentHTML('beforeend', '<dialog open></dialog>');
    for (const target of [map, document.body]) {
      for (const key of ['Enter', ' ', 'f', 'b', '1', 't', 'Escape']) expect(press(target, key).defaultPrevented).toBe(false);
      expect(gameplayFocusAllowed(target)).toBe(false);
      expect(gameplayKeyAllowed(press(target, 'ArrowDown'))).toBe(false);
    }
    for (const command of commands) expect(command).not.toHaveBeenCalled();
    expect(useGame.getState().techOpen).toBe(overlay === 'research');
  });

  it('the open turn report blocks nothing but the first Return: arrows pan, Return closes it, then selects decisions', () => {
    // The user's report: after most turns the report drawer opened and every game key went dead.
    // Playtest 3 (G15): the first Return should close it so the whole map shows.
    useGame.setState({ reportOpen: true, report: { turn: 1, lines: [], events: [], hash: '0' } as never });
    const report = document.body.appendChild(document.createElement('aside'));
    report.dataset.dismissible = 'report';
    window.addEventListener('keydown', handleOverlayKey, true);
    try {
      expect(gameplayKeyAllowed(press(map, 'ArrowRight'))).toBe(true);
      expect(press(map, 'Enter').defaultPrevented).toBe(true);
      expect(useGame.getState().reportOpen).toBe(false);
      expect(nextDecision).not.toHaveBeenCalled();
      report.remove();
      expect(press(map, 'Enter').defaultPrevented).toBe(true);
      expect(nextDecision).toHaveBeenCalledOnce();
      expect(readyAndPass).not.toHaveBeenCalled();
      // Escape peels the same way: the report first, then the selection.
      useGame.setState({ reportOpen: true });
      document.body.append(report);
      press(document.body, 'Escape');
      expect(useGame.getState().reportOpen).toBe(false);
      expect(select).not.toHaveBeenCalled();
      report.remove();
      press(document.body, 'Escape');
      expect(select).toHaveBeenCalledWith(null);
    } finally { window.removeEventListener('keydown', handleOverlayKey, true); }
  });

  describe('Return and Escape close what covers the map (G15)', () => {
    beforeEach(() => window.addEventListener('keydown', handleOverlayKey, true));
    afterEach(() => { window.removeEventListener('keydown', handleOverlayKey, true); notePointerDown(); });
    const surface = (name: string, tag = 'aside') => {
      const el = document.body.appendChild(document.createElement(tag));
      el.dataset.dismissible = name;
      if (tag === 'dialog') el.setAttribute('open', '');
      return el;
    };

    it('closes the top surface first: guide, research, diplomacy, victory, then the report', () => {
      useGame.setState({ reportOpen: true, vpExpanded: true, diplomacyOpen: true, techOpen: true });
      useGuide.getState().show();
      const els = [surface('report'), surface('victory', 'section'), surface('diplomacy'), surface('research', 'dialog'), surface('guide', 'dialog')];
      const open = () => [useGuide.getState().open, useGame.getState().techOpen, useGame.getState().diplomacyOpen, useGame.getState().vpExpanded, useGame.getState().reportOpen];
      for (let i = 0; i < 5; i++) {
        expect(press(map, i % 2 ? 'Escape' : 'Enter').defaultPrevented).toBe(true);
        expect(open().filter(Boolean)).toHaveLength(4 - i);
        els.pop()!.remove();
      }
      unchanged();
    });

    it('closes research before a deferred unlock card on Escape', () => {
      useGame.setState({ techOpen: true });
      const lane = document.body.appendChild(document.createElement('div'));
      lane.className = 'moment-lane';
      lane.innerHTML = '<div class="moment-card unlock"></div>';
      const research = surface('research', 'dialog');
      try {
        expect(press(research, 'Escape').defaultPrevented).toBe(true);
        expect(useGame.getState().techOpen).toBe(false);
      } finally {
        research.remove();
        lane.remove();
      }
    });

    it('leaves decisions alone: the end-turn review, Declare War and the hand-off keep Return', () => {
      useGame.setState({ reportOpen: true });
      surface('report');
      for (const html of ['<dialog open></dialog>', '<div data-game-overlay></div>']) {
        const host = document.body.appendChild(document.createElement('div'));
        host.innerHTML = html;
        expect(press(document.body, 'Enter').defaultPrevented).toBe(false);
        expect(press(document.body, 'Escape').defaultPrevented).toBe(false);
        expect(useGame.getState().reportOpen).toBe(true);
        host.remove();
      }
    });

    it('a closed guide dialog is not a surface; a hidden-but-mounted dialog never blocks Return', () => {
      const guide = document.body.appendChild(document.createElement('dialog'));
      guide.dataset.dismissible = 'guide';
      useGame.setState({ reportOpen: true });
      surface('report');
      expect(press(map, 'Enter').defaultPrevented).toBe(true);
      expect(useGame.getState().reportOpen).toBe(false);
    });

    it('Return keeps its native meaning in text entry and on a control reached with Tab', () => {
      useGame.setState({ diplomacyOpen: true });
      const panel = surface('diplomacy');
      panel.innerHTML = '<input><button>Accept</button>';
      const input = panel.querySelector('input')!, button = panel.querySelector('button')!;
      input.focus();
      expect(press(input, 'Enter').defaultPrevented).toBe(false);
      press(input, 'Tab');
      button.focus();
      expect(press(button, 'Enter').defaultPrevented).toBe(false);
      expect(useGame.getState().diplomacyOpen).toBe(true);
      // A button the pointer clicked last does not own Return: it closes the panel.
      notePointerDown();
      expect(press(button, 'Enter').defaultPrevented).toBe(true);
      expect(useGame.getState().diplomacyOpen).toBe(false);
    });

    it('Escape waits for a playing celebration; modified Return is not a dismissal', () => {
      useGame.setState({ reportOpen: true });
      surface('report');
      const lane = document.body.appendChild(document.createElement('div'));
      lane.className = 'moment-lane';
      lane.innerHTML = '<article class="moment-card"></article>';
      expect(press(map, 'Escape').defaultPrevented).toBe(false);
      lane.remove();
      for (const option of ['shiftKey', 'ctrlKey', 'metaKey', 'altKey'] as const) expect(press(map, 'Enter', { [option]: true }).defaultPrevented).toBe(option === 'shiftKey');
      expect(useGame.getState().reportOpen).toBe(true);
    });
  });

  it('Escape on a keyboard-focused control hands focus back to the map before it deselects', () => {
    // Playtest review: after choosing production by keyboard, Return kept re-pressing that button.
    const button = document.body.appendChild(document.createElement('button'));
    button.focus();
    press(button, 'Escape');
    expect(document.activeElement).toBe(map);
    expect(select).not.toHaveBeenCalled();
    press(map, 'Escape');
    expect(select).toHaveBeenCalledWith(null);
    const input = document.body.appendChild(document.createElement('input'));
    input.focus();
    press(input, 'Escape');
    expect(document.activeElement).toBe(input);
  });

  it('a mouse-clicked map lens (a radio) hands focus back to the map; text entry keeps it', () => {
    // Play-through: after clicking a lens, arrows cycled lenses instead of panning and Return did nothing.
    const host = document.body.appendChild(document.createElement('div'));
    host.innerHTML = '<div role="radiogroup"><button role="radio" aria-checked="true">Default</button></div><select><option>A</option></select>';
    const click = (el: HTMLElement, detail: number) => {
      el.focus();
      window.addEventListener('click', returnFocusAfterPointer, true);
      el.dispatchEvent(new MouseEvent('click', { bubbles: true, detail }));
      window.removeEventListener('click', returnFocusAfterPointer, true);
    };
    const lens = host.querySelector<HTMLElement>('[role="radio"]')!;
    click(lens, 1);
    expect(document.activeElement).toBe(map);
    click(lens, 0);
    expect(document.activeElement).toBe(lens);
    // Keyboard users on the radio still move between lenses with arrows.
    expect(gameplayKeyAllowed(press(lens, 'ArrowRight'))).toBe(false);
    const select = host.querySelector('select')!;
    click(select, 1);
    expect(document.activeElement).toBe(select);
  });

  it('a pointer-activated control hands focus back to the map; a keyboard-activated one keeps it', () => {
    // The user's report: after clicking any button, arrows stopped panning and Enter re-pressed it.
    const button = document.body.appendChild(document.createElement('button'));
    button.focus();
    const click = (detail: number) => {
      button.focus();
      const e = new MouseEvent('click', { bubbles: true, detail });
      window.addEventListener('click', returnFocusAfterPointer, true);
      button.dispatchEvent(e);
      window.removeEventListener('click', returnFocusAfterPointer, true);
    };
    click(1);
    expect(document.activeElement).toBe(map);
    click(0);
    expect(document.activeElement).toBe(button);
    // Text entry and controls inside dialogs keep focus.
    const dialog = document.body.appendChild(document.createElement('dialog'));
    const inner = dialog.appendChild(document.createElement('button'));
    dialog.setAttribute('open', '');
    inner.focus();
    window.addEventListener('click', returnFocusAfterPointer, true);
    inner.dispatchEvent(new MouseEvent('click', { bubbles: true, detail: 1 }));
    window.removeEventListener('click', returnFocusAfterPointer, true);
    expect(document.activeElement).toBe(inner);
  });

  it.each(['ctrlKey', 'metaKey', 'altKey', 'isComposing', 'repeat'] as const)('ignores Enter with %s', (option) => {
    expect(press(map, 'Enter', { [option]: true }).defaultPrevented).toBe(false);
    unchanged();
  });

  it('respects an earlier handler; arrows pan from a plain button but not from text entry or listboxes', () => {
    const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
    event.preventDefault();
    map.dispatchEvent(event);
    unchanged();
    expect(gameplayKeyAllowed(press(map, 'ArrowUp'))).toBe(true);
    expect(gameplayKeyAllowed(press(document.body, 'w'))).toBe(true);
    const button = document.body.appendChild(document.createElement('button'));
    button.focus();
    expect(gameplayKeyAllowed(press(button, 'ArrowDown'))).toBe(true);
    expect(gameplayKeyAllowed(press(button, 'Enter'))).toBe(false);
    expect(gameplayFocusAllowed(document.activeElement)).toBe(false);
    for (const html of ['<input>', '<select><option>A</option></select>', '<div role="listbox"><button role="option">A</button></div>', '<textarea></textarea>']) {
      const host = document.body.appendChild(document.createElement('div'));
      host.innerHTML = html;
      const el = host.querySelector<HTMLElement>('[role="option"]') ?? host.firstElementChild as HTMLElement;
      expect(gameplayKeyAllowed(press(el, 'ArrowDown'))).toBe(false);
    }
    map.focus();
    expect(gameplayFocusAllowed(document.activeElement)).toBe(true);
  });

  it('Y, O and N toggle the Yields, Resources and Borders overlays from the map; R stays the ranged attack', () => {
    vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => {} });
    reloadOverlays();
    try {
      press(map, 'y'); press(map, 'O');
      expect(getOverlays()).toEqual({ yields: true, resources: true, borders: false });
      press(map, 'n'); press(map, 'Y');
      expect(getOverlays()).toEqual({ yields: false, resources: true, borders: true });
      expect(toggleRangedMode).not.toHaveBeenCalled();
      press(map, 'r');
      expect(toggleRangedMode).toHaveBeenCalledOnce();
      // A held key, a modifier or a focused control leave the overlays alone.
      press(map, 'o', { repeat: true }); press(map, 'o', { ctrlKey: true });
      const button = document.body.appendChild(document.createElement('button'));
      press(button, 'o'); press(button, 'n');
      expect(getOverlays()).toEqual({ yields: false, resources: true, borders: true });
    } finally { vi.unstubAllGlobals(); reloadOverlays(); }
  });

  it('keeps map commands, including shifted disband and research', () => {
    for (const key of ['f', 'b', 'r', 'u', 'Delete']) press(map, key);
    for (const command of [sleepSelected, foundCitySelected, toggleRangedMode, upgradeSelected, cancelSelected]) expect(command).toHaveBeenCalledOnce();
        press(map, 'x');
    expect(disbandSelected).not.toHaveBeenCalled();
    press(map, 'X', { shiftKey: true });
    expect(disbandSelected).toHaveBeenCalledOnce();
    press(map, 't');
    expect(useGame.getState().techOpen).toBe(true);
  });
});

it('E toggles auto-explore when the map is focused', () => {
  map.focus(); press(map, 'e');
  expect(toggleAutoExploreSelected).toHaveBeenCalledTimes(1);
});

it('1 does nothing now that the seat keys are gone', () => {
  map.focus(); press(map, '1');
  for (const c of commands) expect(c).not.toHaveBeenCalled();
});


it('Return ends planning when all decisions are handled, but never unreadies a waiting seat', () => {
  const view = useGame.getState().views![0];
  useGame.getState().editOrders(() => [
    ...view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit' as const, unit: u.id })),
    { type: 'SetProduction', city: view.cities[0].id, item: { kind: 'Unit', unit: 'Spearman' } },
    { type: 'SetResearch', tech: 'BronzeWorking' },
  ]);
  press(map, 'Enter');
  expect(nextDecision).not.toHaveBeenCalled();
  expect(readyAndPass).toHaveBeenCalledOnce();
  useGame.setState({ ready: [true] });
  press(map, 'Enter');
  expect(readyAndPass).toHaveBeenCalledOnce();
});

it('invalid orders keep Return on pending decisions', () => {
  const view = useGame.getState().views![0];
  useGame.setState({ orders: [[
    { type: 'SkipUnit', unit: 0 }, { type: 'SkipUnit', unit: 4 },
    { type: 'SetProduction', city: view.cities[0].id, item: { kind: 'Unit', unit: 'Spearman' } },
    { type: 'SetResearch', tech: 'BronzeWorking' },
  ]], errors: [[{ index: 0, message: 'Invalid unit order' }]] });
  press(map, 'Enter');
  expect(nextDecision).toHaveBeenCalledOnce();
  expect(readyAndPass).not.toHaveBeenCalled();
});

it('W stays available for panning, F sleeps and H heals without moving the camera; J does nothing', () => {
  expect(gameplayKeyAllowed(press(map, 'w'))).toBe(true);
  press(map, 'J');
  expect(sleepSelected).not.toHaveBeenCalled();
  expect(healSelected).not.toHaveBeenCalled();
  press(map, 'F');
  expect(sleepSelected).toHaveBeenCalledOnce();
  expect(healSelected).not.toHaveBeenCalled();
  press(map, 'H');
  expect(healSelected).toHaveBeenCalledOnce();
  expect(sleepSelected).toHaveBeenCalledOnce();
  press(map, 'h', { repeat: true }); press(map, 'h', { ctrlKey: true });
  expect(healSelected).toHaveBeenCalledOnce();
});
