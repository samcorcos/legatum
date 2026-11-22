import { expect, it } from 'vitest';
import { battleCues, cuesAt, CUE_FADE_MS, CUE_HOLD_MS } from './cues';
import type { Event, PlayerView } from '../types';

const view = {
  team: 0,
  players: [{ id: 0, team: 0 }, { id: 2, team: 1 }],
  units: [{ id: 7, owner: 0, pos: { q: 3, r: 3 } }],
} as unknown as PlayerView;
const battle = (over: Partial<Extract<Event, { type: 'Battle' }>>) => ({
  type: 'Battle', attacker: 7, attacker_owner: 0, defender: { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Scout' }, at: { q: 4, r: 3 },
  damage_to_attacker: 6, damage_to_defender: 31, ...over,
}) as Event;

it('marks damage on both hexes, in the colour of who lost it', () => {
  const cues = battleCues([battle({})], view);
  expect(cues).toEqual([
    expect.objectContaining({ hex: { q: 4, r: 3 }, text: '−31', tone: 'good' }),
    expect.objectContaining({ hex: { q: 3, r: 3 }, text: '−6', tone: 'bad' }),
  ]);
});

it('a destroyed unit replaces the damage number on its hex', () => {
  const cues = battleCues([battle({}), { type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Scout', at: { q: 4, r: 3 } } as Event], view);
  expect(cues.find((c) => c.hex.q === 4)).toMatchObject({ text: 'Destroyed', tone: 'good' });
});

it('holds, then fades out, and never drifts with reduced motion', () => {
  const cues = battleCues([battle({})], view);
  expect(cuesAt(cues, 1000, false)[0].alpha).toBe(1);
  expect(cuesAt(cues, CUE_HOLD_MS + CUE_FADE_MS / 2, false)[0].alpha).toBeCloseTo(0.5);
  expect(cuesAt(cues, CUE_HOLD_MS + CUE_FADE_MS + 1, false)).toEqual([]);
  expect(cuesAt(cues, 2000, true)[0].rise).toBe(0);
});

it('a city shot marks only the unit it hit', () => {
  const cues = battleCues([battle({ attacker: { city: 7 }, damage_to_attacker: 0 })], view);
  expect(cues).toEqual([expect.objectContaining({ hex: { q: 4, r: 3 }, text: '−31', tone: 'good' })]);
});
