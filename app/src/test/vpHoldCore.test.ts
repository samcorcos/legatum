// Winning by VP means holding the target (core `scoring::update_vp_holds`): the client engine alone.
import { describe, expect, it } from 'vitest';
import type { Event } from '../types';
import { FixtureClient } from './fixtureEngine';
import { holdFixture, type HoldSpec } from './vpHoldFixture';

const THREES = [0, 0, 0, 1, 1, 1];
const holdEvents = (events: Event[]) => events.filter((e) => ['VictoryHoldStarted', 'VictoryHoldBroken', 'GameWon', 'GameDrawn'].includes(e.type));

function start(spec: HoldSpec) {
  const client = new FixtureClient();
  client.load(JSON.stringify(holdFixture(spec)));
  return client;
}
const standing = (client: FixtureClient, team: number) => client.view(0).standings.find((s) => s.team === team)!;

describe('client VP hold', () => {
  it('starts a hold at the first turn at the target and reports it publicly', () => {
    const client = start({ teams: THREES, cities: { 0: 4 }, bonusEra: [0] });
    expect(standing(client, 0).victory_hold).toBeUndefined();
    const { events } = client.endTurn({});
    const a = standing(client, 0);
    expect(a.vp).toBeGreaterThanOrEqual(a.target);
    expect(a.victory_hold).toEqual({ turns_held: 1, turns_needed: 10 });
    expect(standing(client, 1).victory_hold).toBeUndefined();
    expect(holdEvents(events)).toEqual([{ type: 'VictoryHoldStarted', team: 0, vp: a.vp, target: a.target, turns_needed: 10 }]);
    expect(client.view(3).standings.find((s) => s.team === 0)!.victory_hold).toEqual({ turns_held: 1, turns_needed: 10 });
  });

  it('wins after holding the target for 10 turn ends', () => {
    const needed = 10;
    const client = start({ teams: THREES, cities: { 0: 4 }, bonusEra: [0] });
    for (let turn = 1; turn < needed; turn++) {
      const { events, finished } = client.endTurn({});
      expect(finished).toBe(false);
      expect(standing(client, 0).victory_hold).toEqual({ turns_held: turn, turns_needed: needed });
      expect(holdEvents(events).map((e) => e.type)).toEqual(turn === 1 ? ['VictoryHoldStarted'] : []);
    }
    const last = client.endTurn({});
    expect(last.finished).toBe(true);
    expect(last.winner).toBe(0);
    expect(holdEvents(last.events)).toEqual([expect.objectContaining({ type: 'GameWon', team: 0 })]);
    expect(client.view(0).turn).toBe(needed + 1);
  });

  it('dropping below the target ends the hold and resets the count', () => {
    const client = start({ teams: THREES, holds: { 0: 4 } });
    const a = standing(client, 0);
    expect(a.victory_hold).toEqual({ turns_held: 4, turns_needed: 10 });
    const { events, finished } = client.endTurn({});
    expect(finished).toBe(false);
    expect(holdEvents(events)).toEqual([{ type: 'VictoryHoldBroken', team: 0, turns_held: 4, vp: standing(client, 0).vp, target: a.target }]);
    expect(standing(client, 0).victory_hold).toBeUndefined();
    expect(client.debugState().vpHolds).toBeUndefined();
  });

  it('a team that never held loses nothing: no events below the target', () => {
    const client = start({ teams: THREES });
    expect(holdEvents(client.endTurn({}).events)).toEqual([]);
  });

  it('elimination still wins at once, before any hold completes', () => {
    const client = start({ teams: THREES, holds: { 0: 3 }, noSettler: [3, 4, 5] });
    const { events, finished, winner } = client.endTurn({});
    expect(finished).toBe(true);
    expect(winner).toBe(0);
    expect(holdEvents(events)).toEqual([{ type: 'GameWon', team: 0, reason: 'P0 & P1 & P2 is the last team standing' }]);
  });

  it('an eliminated team loses its hold (with the event) while the game goes on', () => {
    const client = start({ teams: [0, 0, 0, 1, 1, 1, 2], holds: { 0: 3 }, noSettler: [0, 1, 2], oneShort: [6] });
    const { events, finished } = client.endTurn({});
    expect(finished).toBe(false);
    expect(holdEvents(events)).toEqual([expect.objectContaining({ type: 'VictoryHoldBroken', team: 0, turns_held: 3 })]);
    expect(standing(client, 0).victory_hold).toBeUndefined();
  });

  it('teams completing the same turn: the higher VP wins', () => {
    const client = start({ teams: THREES, holds: { 0: 9, 1: 9 }, cities: { 0: 4 }, bonusEra: [0, 1, 3] });
    const { events, winner } = client.endTurn({});
    expect(winner).toBe(0);
    expect(holdEvents(events).filter((e) => e.type === 'GameWon')).toHaveLength(1);
  });

  it('completing the same turn on equal VP and population is a draw', () => {
    const client = start({ teams: THREES, holds: { 0: 9, 1: 9 }, cities: { 0: 3, 3: 3 }, bonusEra: [0, 3] });
    const { events, finished, winner } = client.endTurn({});
    expect(finished).toBe(true);
    expect(winner).toBeNull();
    expect(holdEvents(events).map((e) => e.type)).toEqual(['GameDrawn']);
  });

  it('survives a save and load mid-hold', () => {
    const client = start({ teams: THREES, cities: { 0: 4 }, bonusEra: [0] });
    for (let i = 0; i < 3; i++) client.endTurn({});
    client.load(client.save());
    expect(standing(client, 0).victory_hold).toEqual({ turns_held: 3, turns_needed: 10 });
  });
});
