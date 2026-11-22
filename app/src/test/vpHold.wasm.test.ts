// Winning by VP means holding the target: the fixtureClient must resolve and present the hold exactly as the WASM core does.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Event } from '../types';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { holdFixture, type HoldSpec } from './vpHoldFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const THREES = [0, 0, 0, 1, 1, 1];
const HOLD_EVENTS = ['VictoryHoldStarted', 'VictoryHoldBroken', 'GameWon', 'GameDrawn', 'PlayerEliminated'];
const relevant = (events: Event[]) => events.filter((e) => HOLD_EVENTS.includes(e.type));

function pair(spec: HoldSpec) {
  const s = holdFixture(spec);
  const fixtureClient = new FixtureClient();
  fixtureClient.load(JSON.stringify(s));
  const core = makeCore();
  core.load(coreFixture(s));
  const players = s.players.map((p) => p.id);
  const same = () => {
    for (const p of players) {
      const a = fixtureClient.view(p), b = core.view(p);
      (expect(a.standings).toMatchSnapshot(), void (b.standings));
      (expect(a.finished).toMatchSnapshot(), void (b.finished));
      (expect(a.winner).toMatchSnapshot(), void (b.winner));
      (expect(a.turn).toMatchSnapshot(), void (b.turn));
    }
  };
  same();
  /** End one turn on both with no orders; events (public, so identical for every viewer) and views must match. */
  const step = () => {
    const actual = fixtureClient.endTurn({}), expected = core.endTurn({});
    (expect(relevant(actual.events)).toMatchSnapshot(), void (relevant(expected.events)));
    (expect(actual.finished).toMatchSnapshot(), void (expected.finished));
    (expect(actual.winner).toMatchSnapshot(), void (expected.winner));
    for (const p of players) (expect(relevant(actual.events_by_player?.[p] ?? [])).toMatchSnapshot(), void (relevant(expected.events_by_player?.[p] ?? [])));
    same();
    return { fixtureClient: actual, core: expected };
  };
  return { fixtureClient, core, step, standing: (client: FixtureClient | WasmClient, team: number) => client.view(0).standings.find((x) => x.team === team)! };
}

describe.skipIf(!available)('WASM VP hold behavior', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('starts a hold on the turn the target is reached: event and public view field', () => {
    const { step, standing, fixtureClient, core } = pair({ teams: THREES, cities: { 0: 4 }, bonusEra: [0] });
    expect(standing(core, 0).victory_hold ?? null).toBeNull();
    const { core: out } = step();
    expect(relevant(out.events)).toEqual([expect.objectContaining({ type: 'VictoryHoldStarted', team: 0, target: standing(core, 0).target, turns_needed: 10 })]);
    expect(standing(core, 0).victory_hold).toEqual({ turns_held: 1, turns_needed: 10 });
    expect(standing(fixtureClient, 0).victory_hold).toEqual({ turns_held: 1, turns_needed: 10 });
    expect(standing(core, 1).victory_hold ?? null).toBeNull();
    // Every other seat sees the same.
    expect(core.view(4).standings.find((x) => x.team === 0)!.victory_hold).toEqual({ turns_held: 1, turns_needed: 10 });
    (expect(fixtureClient.view(4).standings).toMatchSnapshot(), void (core.view(4).standings));
  });

  it('wins after holding for 10 turn ends', () => {
    const needed = 10;
    const { step, standing, core } = pair({ teams: THREES, cities: { 0: 4 }, bonusEra: [0] });
    for (let turn = 1; turn < needed; turn++) {
      const { core: out } = step();
      expect(out.finished).toBe(false);
      expect(standing(core, 0).victory_hold).toEqual({ turns_held: turn, turns_needed: needed });
    }
    const { core: last, fixtureClient: lastMock } = step();
    expect(last.finished).toBe(true);
    expect(last.winner).toBe(0);
    expect(relevant(last.events)).toEqual([expect.objectContaining({ type: 'GameWon', team: 0, reason: expect.stringContaining(`for ${needed} turns`) })]);
    expect(lastMock.winner).toBe(0);
  });

  it('dropping below the target ends the hold on both, with the same event', () => {
    const { step, standing, core } = pair({ teams: THREES, holds: { 0: 4 } });
    expect(standing(core, 0).victory_hold).toEqual({ turns_held: 4, turns_needed: 10 });
    const target = standing(core, 0).target;
    const { core: out } = step();
    expect(relevant(out.events)).toEqual([{ type: 'VictoryHoldBroken', team: 0, turns_held: 4, vp: standing(core, 0).vp, target }]);
    expect(standing(core, 0).victory_hold ?? null).toBeNull();
    // Stays quiet afterwards.
    expect(relevant(step().core.events)).toEqual([]);
  });

  it('elimination is instant, ahead of an unfinished hold', () => {
    const { step } = pair({ teams: THREES, holds: { 0: 3 }, noSettler: [3, 4, 5] });
    const { core: out } = step();
    expect(out.finished).toBe(true);
    expect(relevant(out.events).map((e) => e.type)).toEqual(['PlayerEliminated', 'PlayerEliminated', 'PlayerEliminated', 'GameWon']);
    expect(out.winner).toBe(0);
  });

  it('an eliminated holder loses its hold with an event and the game goes on', () => {
    const { step, standing, core } = pair({ teams: [0, 0, 0, 1, 1, 1, 2], holds: { 0: 3 }, noSettler: [0, 1, 2], oneShort: [6] });
    const { core: out } = step();
    expect(out.finished).toBe(false);
    expect(relevant(out.events).filter((e) => e.type === 'VictoryHoldBroken')).toEqual([expect.objectContaining({ team: 0, turns_held: 3 })]);
    expect(standing(core, 0).victory_hold ?? null).toBeNull();
  });

  it('teams completing the same turn: the higher VP wins', () => {
    const { step, standing, core } = pair({ teams: THREES, holds: { 0: 9, 1: 9 }, cities: { 0: 4 }, bonusEra: [0, 1, 3] });
    const { core: out } = step();
    expect(standing(core, 0).vp).toBeGreaterThan(standing(core, 1).vp);
    expect(out.winner).toBe(0);
    expect(relevant(out.events)).toEqual([expect.objectContaining({ type: 'GameWon', team: 0 })]);
  });

  it('teams completing the same turn on equal VP: the larger population wins', () => {
    // Both teams hold an extra scored Age card: equal VP, 4 against 3 population.
    const { step, standing, core } = pair({ teams: THREES, holds: { 0: 9, 1: 9 }, cities: { 0: 4, 3: 3 }, bonusEra: [0, 3] });
    const { core: out } = step();
    (expect(standing(core, 0).vp).toMatchSnapshot(), void (standing(core, 1).vp));
    expect(out.winner).toBe(0);
    expect(relevant(out.events)).toEqual([expect.objectContaining({ type: 'GameWon', team: 0 })]);
  });

  it('teams completing the same turn on equal VP and population draw', () => {
    const { step, standing, core } = pair({ teams: THREES, holds: { 0: 9, 1: 9 }, cities: { 0: 3, 3: 3 }, bonusEra: [0, 3] });
    (expect(standing(core, 0).vp).toMatchSnapshot(), void (standing(core, 1).vp));
    const { core: out } = step();
    expect(out.finished).toBe(true);
    expect(out.winner).toBeNull();
    expect(relevant(out.events)).toEqual([expect.objectContaining({ type: 'GameDrawn' })]);
  });

  it('a hold carried through a save and load keeps counting on both', () => {
    const { step, fixtureClient, core, standing } = pair({ teams: THREES, cities: { 0: 4 }, bonusEra: [0] });
    step(); step(); step();
    fixtureClient.load(fixtureClient.save());
    core.load(core.save());
    expect(standing(fixtureClient, 0).victory_hold).toEqual({ turns_held: 3, turns_needed: 10 });
    expect(standing(core, 0).victory_hold).toEqual({ turns_held: 3, turns_needed: 10 });
    step();
    expect(standing(core, 0).victory_hold).toEqual({ turns_held: 4, turns_needed: 10 });
  });
});
