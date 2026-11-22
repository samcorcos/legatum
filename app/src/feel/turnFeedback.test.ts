import { beforeAll, describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import type { Event, PlayerView } from '../types';
import { feedbackForTurn, type TurnFeedback } from './turnFeedback';

let base: PlayerView;
beforeAll(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  client.endTurn({});
  base = client.view(0);
  base.economy.bankruptcy = null;
  base.units = base.units.filter((u) => u.owner === 0);
  base.cities = base.cities.filter((c) => c.owner === 0);
  base.cities[0].detail!.yields.total.food = base.cities[0].population * 2;
}, 30_000); // Map generation can exceed Vitest's 10s hook default on a shared build box.

const battle: Extract<Event, { type: 'Battle' }> = {
  type: 'Battle', attacker: 1, attacker_owner: 0,
  defender: { kind: 'Unit', unit: 2, owner: 2, unit_kind: 'Soldier' }, at: { q: 2, r: 2 },
  attacker_strength: { total: 10, parts: [] }, defender_strength: { total: 10, parts: [] },
  damage_to_attacker: 20, damage_to_defender: 100, attacker_roll: 1, defender_roll: 1,
  attacker_hp_after: 80, defender_hp_after: 0, ranged: false, meeting: false,
};

const examples: [TurnFeedback, Event][] = [
  ['battleWon', battle],
  ['battleLost', { type: 'UnitDestroyed', unit: 1, owner: 0, unit_kind: 'Soldier', at: { q: 2, r: 2 } }],
  ['cityFounded', { type: 'CityFounded', city: 1, owner: 0, name: 'Home', at: { q: 2, r: 2 } }],
  ['cityCaptured', { type: 'CityCaptured', city: 2, from: 2, to: 0 }],
  ['cityLost', { type: 'CityCaptured', city: 1, from: 0, to: 2 }],
  ['techComplete', { type: 'TechResearched', player: 0, tech: 'BronzeWorking' }],
  ['buildingComplete', { type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Building', building: 'Granary' } }],
  ['greatWork', { type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Building', building: 'PlanetaryFoundry' } }],
  ['greatWork', { type: 'GreatWorkCompleted', player: 0, of: 0, building: 'RelayUplink', vp: 2, again: false }],
  ['greatWork', { type: 'EraObjectiveScored', player: 0, era: 2, slot: 1, objective: 'Gain 4 cities' }],
  ['vpGained', { type: 'GreatWorkCompleted', player: 0, of: 2, building: 'RelayUplink', vp: 2, again: true }],
  ['vpLost', { type: 'GreatWorkLost', player: 0, of: 0, building: 'RelayUplink', vp: 2 }],
  ['warning', { type: 'CityStarved', city: 1, owner: 0, population: 1 }],
  ['eliminated', { type: 'PlayerEliminated', player: 0 }],
  ['victory', { type: 'GameWon', team: 0, reason: 'VP' }],
  ['defeat', { type: 'GameWon', team: 2, reason: 'VP' }],
];

describe('feedbackForTurn', () => {
  it('acknowledges a new planning turn, but not a finished game', () => {
    expect(feedbackForTurn([], base)).toEqual(['turnResolved']);
    expect(feedbackForTurn([], { ...base, finished: true })).toEqual([]);
  });

  it.each(examples)('maps %s from %j', (call, event) => {
    expect(feedbackForTurn([event], base)).toEqual(['turnResolved', call]);
  });

  it('a routine city shot is no victory; a city shot that destroys the unit is', () => {
    expect(feedbackForTurn([{ ...battle, attacker: { city: 1 }, damage_to_attacker: 0, defender_hp_after: 80 }], base)).not.toContain('battleWon');
    expect(feedbackForTurn([{ ...battle, attacker: { city: 1 }, damage_to_attacker: 0, defender_hp_after: 0 }], base)).toContain('battleWon');
  });

  it('recognizes no-loss attacks and successful unit defenses, without celebrating a mutual loss', () => {
    expect(feedbackForTurn([{ ...battle, damage_to_attacker: 0, defender_hp_after: 80 }], base)).toContain('battleWon');
    expect(feedbackForTurn([{ ...battle, attacker_owner: 2, attacker_hp_after: 0,
      defender: { kind: 'Unit', unit: 1, owner: 0, unit_kind: 'Soldier' }, defender_hp_after: 70 }], base)).toContain('battleWon');
    expect(feedbackForTurn([{ ...battle, attacker_hp_after: 0 }], base)).not.toContain('battleWon');
    expect(feedbackForTurn([{ ...battle, attacker_hp_after: 80, defender_hp_after: 80 }], base)).not.toContain('battleWon');
  });

  it('ignores rival/teammate private events, unit production and unrelated movement', () => {
    const events: Event[] = [
      { ...battle, attacker_owner: 2 },
      { type: 'TechResearched', player: 1, tech: 'BronzeWorking' },
      { type: 'CityFounded', city: 5, owner: 2, name: 'Hidden', at: { q: 999, r: 999 } },
      { type: 'CityCaptured', city: 5, from: 2, to: 3 },
      { type: 'ProductionCompleted', city: 5, owner: 2, item: { kind: 'Building', building: 'PlanetaryFoundry' } },
      { type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Unit', unit: 'Soldier' } },
      { type: 'UnitMoved', unit: 1, owner: 0, from: null, to: null },
      { type: 'PlayerEliminated', player: 2 },
      { type: 'FactionGoalCompleted', player: 2, faction: 'India', vp: 2 },
    ];
    expect(feedbackForTurn(events, base)).toEqual(['turnResolved']);
  });

  it('deduplicates each semantic event and keeps the most important four in narrative order', () => {
    const events = examples.filter(([name]) => !['victory', 'defeat', 'warning'].includes(name)).map(([, event]) => event);
    expect(feedbackForTurn([...events, ...events], base)).toEqual(['cityCaptured', 'cityLost', 'greatWork', 'eliminated']);
    expect(feedbackForTurn([...events, { type: 'GameWon', team: 0, reason: 'VP' }], { ...base, finished: true }))
      .toEqual(['cityCaptured', 'cityLost', 'eliminated', 'victory']);
    const tech = examples.find(([name]) => name === 'techComplete')![1];
    expect(feedbackForTurn([tech, tech, tech], base)).toEqual(['turnResolved', 'techComplete']);
  });

  it.each(['bankruptcy', 'starving', 'enemy'] as const)('warns about %s only when it first appears in the viewer view', (kind) => {
    const next = structuredClone(base);
    if (kind === 'bankruptcy') next.economy.bankruptcy = {
      gold: { total: -5, parts: [] }, disband: { unit: 1, kind: 'Soldier', pos: { q: 0, r: 0 }, city: null },
    };
    if (kind === 'starving') next.cities[0].detail!.yields.total.food = 0;
    if (kind === 'enemy') next.units.push({ ...next.units[0], id: 999, owner: 2,
      pos: { q: next.cities[0].pos.q + 1, r: next.cities[0].pos.r } });
    expect(feedbackForTurn([], next, base)).toEqual(['turnResolved', 'warning']);
    expect(feedbackForTurn([], next, next)).toEqual(['turnResolved']);
    // Ending and later renewing a threat is a new warning.
    expect(feedbackForTurn([], next, base)).toContain('warning');
  });

  it('does not infer unseen enemies from public army counts or repeat persistent starvation events', () => {
    const prior = structuredClone(base);
    prior.cities[0].detail!.yields.total.food = 0;
    expect(feedbackForTurn([{ type: 'CityStarved', city: prior.cities[0].id, owner: 0, population: 1 }], prior, prior))
      .toEqual(['turnResolved']);
    expect(feedbackForTurn([], base, base)).toEqual(['turnResolved']);
  });
});
