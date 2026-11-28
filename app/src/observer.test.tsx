// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { buildSetup, newLobby, problems, setObserver } from './lobby';
import { INACTIVE_OBSERVER, OBSERVER_SPEEDS } from './observer';
import type { GameClient } from './client';
import type { PlayerView, TurnResult } from './types';
import { useGame } from './store';
import { ObserverBar } from './ui/ObserverBar';
import { handleGameKey } from './screens/gameKeyHandler';

function makeView(turn: number, me: number, god: boolean, finished = false): PlayerView {
  return {
    turn,
    turn_limit: 200,
    me,
    team: me,
    width: 4,
    height: 4,
    tiles: Array.from({ length: 16 }, (_, i) => ({
      explored: god || i < 4,
      visible: god || i < 2,
      terrain: god || i < 4 ? 'Plains' : null,
      ruin: false,
      wonder: null,
      landmark: null,
      wonder_found: false,
      owner: null,
      resource: null,
      improvement: null,
      road: 0,
    })),
    units: [],
    cities: [],
    failed_city_sites: [],
    city_gap_blocked: [],
    observed_threats: [],
    barbarian_camps: [],
    players: [
      { id: 0, name: 'AI 1', faction: 'Rome', team: 0, is_human: false, alive: true, vp: turn, vp_sources: [], cities: 1, gold: 100, army: 50, science: 10, era: 1, faction_goal_done: false },
      { id: 1, name: 'AI 2', faction: 'Egypt', team: 1, is_human: false, alive: true, vp: turn, vp_sources: [], cities: 1, gold: 120, army: 40, science: 12, era: 1, faction_goal_done: false },
    ],
    diplomacy: {
      my_team: me,
      relations: [],
      pending_offers: [],
      met: god ? [1] : [],
      deals: true,
    },
    economy: {
      gold: 100 + me * 20,
      military_purchases: 0,
      gold_per_turn: { total: 10, parts: [] },
      science_per_turn: { total: 10 + me * 2, parts: [] },
      upkeep: 0,
      upkeep_breakdown: [],
      bankruptcy: null,
    },
    supply: [],
    research: {
      known: [],
      current: null,
      cost: null,
      progress: 0,
      queue: [],
      era: 1,
      era_known: 0,
      era_total: 4,
      options: [],
      schedule: [],
    },
    army_breakdown: [],
    eras: [],
    objective_progress: [],
    objective_accounting: [],
    development: null,
    faction_goal: null,
    actions: {},
    courier_halls: [],
    imperial_diwans: [],
    colonias: [],
    royal_tombs: [],
    tomb_sites: [],
    aircraft: [],
    air_sightings: [],
    air_contacts: [],
    city_buys: [],
    city_offers: [],
    byzantine_capitals: [],
    great_works: [],
    standings: { rankings: [] },
    winner: null,
    finished,
    history: null,
    rule_experiments: {},
    ai_alliances: false,
    added_city_upkeep: 0,
  } as unknown as PlayerView;
}

function makeFakeObserverClient() {
  let turn = 1;
  let endTurnCalls = 0;
  const loadCalls: string[] = [];
  const client: GameClient = {
    kind: 'wasm',
    async newGame() { turn = 1; },
    async standardSetup() { throw new Error('unused'); },
    view(player: number) { return makeView(turn, player, false); },
    observerView() { return makeView(turn, 0, true); },
    hash() { return `hash-t${turn}`; },
    validate() { return []; },
    forecast() { return null; },
    endTurn(): TurnResult {
      endTurnCalls++;
      turn++;
      return { turn: turn - 1, events: [], hash: `hash-t${turn}`, finished: false, winner: null };
    },
    save() { return JSON.stringify({ turn }); },
    load(json: string) {
      loadCalls.push(json);
      turn = (JSON.parse(json) as { turn: number }).turn;
    },
  };
  return { client, getEndTurnCalls: () => endTurnCalls, loadCalls };
}

describe('AI-vs-AI Observer Mode', () => {
  beforeEach(() => {
    useGame.setState({
      screen: 'title',
      client: null,
      humans: [],
      views: null,
      seat: 0,
      orders: [],
      errors: [],
      ready: [],
      observer: INACTIVE_OBSERVER,
      selection: null,
      techOpen: false,
      citiesOpen: false,
      vpExpanded: false,
      error: null,
    });
  });

  afterEach(() => {
    useGame.getState().observerPause();
    cleanup();
    vi.restoreAllMocks();
  });

  it('lobby setObserver produces a valid 0-human GameSetup and toggles cleanly back to human', () => {
    const solo = newLobby('solo', '42');
    expect(solo.slots[0].kind).toBe('you');
    expect(solo.slots[0].name).toBe('Player 1');

    const obs = setObserver(solo, true);
    expect(obs.observer).toBe(true);
    expect(obs.training).toBe(false);
    expect(obs.slots[0].kind).toBe('ai');
    expect(obs.slots[0].name).toBe('AI 1');
    expect(problems(obs)).toEqual([]);

    const setup = buildSetup(obs);
    expect(setup.players.every((p) => !p.is_human)).toBe(true);
    expect(setup.players.length).toBe(solo.slots.length);
    expect(setup.seed).toBe('42');

    const back = setObserver(obs, false);
    expect(back.observer).toBe(false);
    expect(back.slots[0].kind).toBe('you');
    expect(back.slots[0].name).toBe('Player 1');
  });

  it('supports deterministic step-forward, step-back rewind, cached replay, and perspective switching', () => {
    const { client, getEndTurnCalls, loadCalls } = makeFakeObserverClient();
    const initialGod = client.observerView!();
    useGame.setState({
      screen: 'game',
      client,
      humans: [],
      views: [initialGod],
      seat: 0,
      orders: [[]],
      errors: [[]],
      ready: [false],
      observer: {
        active: true,
        playing: false,
        speed: 1,
        perspective: 'god',
        history: [{ turn: 1, save: client.save(), hash: client.hash!() }],
        cursor: 0,
      },
    });

    // Step forward twice (turns 1 -> 2 -> 3)
    useGame.getState().observerStepForward();
    expect(useGame.getState().views?.[0].turn).toBe(2);
    expect(useGame.getState().observer.cursor).toBe(1);
    expect(useGame.getState().observer.history.length).toBe(2);

    useGame.getState().observerStepForward();
    expect(useGame.getState().views?.[0].turn).toBe(3);
    expect(useGame.getState().observer.cursor).toBe(2);
    expect(getEndTurnCalls()).toBe(2);

    // Switch to faction 1 fogged view
    useGame.getState().observerSetPerspective(1);
    const p1View = useGame.getState().views?.[0];
    expect(useGame.getState().observer.perspective).toBe(1);
    expect(p1View?.me).toBe(1);
    expect(p1View?.tiles.some((t) => !t.explored)).toBe(true);

    // Step backward to turn 2: stays on faction 1 perspective and loads snapshot
    useGame.getState().observerStepBack();
    expect(useGame.getState().views?.[0].turn).toBe(2);
    expect(useGame.getState().views?.[0].me).toBe(1);
    expect(useGame.getState().observer.cursor).toBe(1);
    expect(loadCalls.length).toBe(1);

    // Step forward from rewound turn 2 -> 3: replays cached snapshot without calling endTurn again
    useGame.getState().observerStepForward();
    expect(useGame.getState().views?.[0].turn).toBe(3);
    expect(getEndTurnCalls()).toBe(2);
    expect(loadCalls.length).toBe(2);

    // Switch back to God view: all tiles explored & visible
    useGame.getState().observerSetPerspective('god');
    const godView = useGame.getState().views?.[0];
    expect(useGame.getState().observer.perspective).toBe('god');
    expect(godView?.tiles.every((t) => t.explored && t.visible)).toBe(true);

    // Observer mode is strictly read-only: editOrders is a no-op
    useGame.getState().editOrders((o) => [...o, { type: 'SkipUnit', unit: 1 }]);
    expect(useGame.getState().orders[0]).toEqual([]);
  });

  it('renders ObserverBar only in Observer Mode and handles controls + keyboard shortcuts', () => {
    // Hidden in normal human games
    const { rerender } = render(<ObserverBar />);
    expect(screen.queryByTestId('observer-bar')).toBeNull();

    const { client } = makeFakeObserverClient();
    useGame.setState({
      screen: 'game',
      client,
      humans: [],
      views: [client.observerView!()],
      seat: 0,
      orders: [[]],
      errors: [[]],
      ready: [false],
      observer: {
        active: true,
        playing: false,
        speed: 1,
        perspective: 'god',
        history: [{ turn: 1, save: client.save(), hash: client.hash!() }],
        cursor: 0,
      },
    });
    rerender(<ObserverBar />);

    expect(screen.getByTestId('observer-bar')).toBeTruthy();
    const backBtn = screen.getByTestId('obs-step-back') as HTMLButtonElement;
    const playBtn = screen.getByTestId('obs-play-pause') as HTMLButtonElement;
    const fwdBtn = screen.getByTestId('obs-step-forward') as HTMLButtonElement;
    const speedSel = screen.getByTestId('obs-speed') as HTMLSelectElement;
    const perspSel = screen.getByTestId('obs-perspective') as HTMLSelectElement;

    expect(backBtn.disabled).toBe(true);
    expect(playBtn.getAttribute('aria-pressed')).toBe('false');
    expect(speedSel.value).toBe('1');
    expect(OBSERVER_SPEEDS).toContain(Number(speedSel.value));

    // Step forward via button enables step back
    fireEvent.click(fwdBtn);
    expect(useGame.getState().views?.[0].turn).toBe(2);
    expect(backBtn.disabled).toBe(false);

    // Speed change
    fireEvent.change(speedSel, { target: { value: '2' } });
    expect(useGame.getState().observer.speed).toBe(2);

    // Perspective change via dropdown
    fireEvent.change(perspSel, { target: { value: '1' } });
    expect(useGame.getState().observer.perspective).toBe(1);

    // Keyboard shortcuts: Right advances, Left rewinds, Space toggles Play/Pause
    handleGameKey(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    expect(useGame.getState().views?.[0].turn).toBe(3);

    handleGameKey(new KeyboardEvent('keydown', { key: 'ArrowLeft' }));
    expect(useGame.getState().views?.[0].turn).toBe(2);

    handleGameKey(new KeyboardEvent('keydown', { key: ' ' }));
    expect(useGame.getState().observer.playing).toBe(true);

    handleGameKey(new KeyboardEvent('keydown', { key: ' ' }));
    expect(useGame.getState().observer.playing).toBe(false);
  });
});
