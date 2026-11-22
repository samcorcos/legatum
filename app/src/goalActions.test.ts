import { describe, expect, it } from 'vitest';
import { TECHS, TECH_ERA } from './data';
import { clearAction, clearRetire, freeTechTarget, pendingAction, pendingRetire, pickState, retireEntry, retirePlaces, retirePreview, retireProgress, setAction, swarmPreview, swarmRows, swarmTotals, togglePick, wildPreview, wildRows } from './goalActions';
import type { Order, PlayerView, TechId } from './types';

const view = (patch: Partial<PlayerView> = {}) => ({
  me: 0, cities: [
    { id: 1, owner: 0, name: 'Hive', pos: { q: 1, r: 1 }, population: 30 },
    { id: 2, owner: 0, name: 'Cell', pos: { q: 5, r: 5 }, population: 22 },
    { id: 3, owner: 1, name: 'Elsewhere', pos: { q: 9, r: 9 }, population: 4 },
  ],
  research: { known: ['BronzeWorking'] },
  actions: {
    swarm: { population: 50, spare: 49, blocker: null, cities: [
      { city: 1, lost: 27, population: 3, tiles: [{ q: 2, r: 1 }, { q: 3, r: 1 }] },
      { city: 2, lost: 23, population: -1, tiles: [{ q: 6, r: 5 }] },
    ] },
    free_techs: { pending: 3, from: 1, choices: [] },
  },
  ...patch,
}) as unknown as PlayerView;

describe('goal action orders', () => {
  it('replaces and clears the pending action of its kind only', () => {
    const base: Order[] = [{ type: 'SetResearch', tech: 'BronzeWorking' }];
    const swarm = setAction(base, { action: 'Swarm' });
    expect(swarm).toHaveLength(2);
    expect(setAction(swarm, { action: 'Swarm' })).toHaveLength(2);
    expect(setAction(swarm, { action: 'Broadcast' })).toHaveLength(3);
    expect(pendingAction(swarm, 'Swarm')).toBeTruthy();
    expect(pendingAction(swarm, 'Broadcast')).toBeUndefined();
    expect(clearAction(swarm, 'Swarm')).toEqual(base);
  });
});

describe('retiring veterans and calling the warbands', () => {
  it('each retirement is its own order, per unit', () => {
    const one = setAction([], { action: 'RetireVeteran', unit: 7 });
    const two = setAction(one, { action: 'RetireVeteran', unit: 8 });
    expect(two).toHaveLength(2);
    expect(setAction(two, { action: 'RetireVeteran', unit: 7 })).toHaveLength(2);
    expect(pendingRetire(two, 8)).toBeTruthy();
    expect(pendingRetire(two, 9)).toBeUndefined();
    expect(clearRetire(two, 7)).toEqual([{ type: 'Action', action: { action: 'RetireVeteran', unit: 8 } }]);
    expect(setAction(two, { action: 'CallTheWild' })).toHaveLength(3);
  });

  const withActions = (actions: object) => view({ actions: actions as PlayerView['actions'] });
  it('reads the veterans, the shrine cities and the progress from the core view', () => {
    const v = withActions({ retire: { retired: 1, target: 3, level: 4, shrines: [2], veterans: [{ unit: 5, level: 4, blocker: null }, { unit: 6, level: 5, blocker: 'away' }] } });
    expect(retireEntry(v, 5)).toMatchObject({ level: 4 });
    expect(retireEntry(v, 99)).toBeUndefined();
    expect(retirePlaces(v)).toEqual([{ q: 5, r: 5 }]);
    expect(retireProgress(v)).toEqual({ retired: 1, target: 3, after: 2 });
    expect(retireProgress(withActions({ retire: { retired: 2, target: 3, level: 4, shrines: [], veterans: [] } }))?.after).toBe(3);
    expect(retireProgress(view())).toBeNull();
  });

  it('marks the Agoge cities only for a selected veteran or while the confirm is open', () => {
    const v = withActions({ retire: { retired: 0, target: 3, level: 4, shrines: [1, 2], veterans: [{ unit: 5, level: 4, blocker: null }] } });
    const unit = (id: number) => ({ id }) as never;
    expect(retirePreview(v, unit(5), false)).toEqual({ cities: [], released: [], camps: [], places: [{ q: 1, r: 1 }, { q: 5, r: 5 }] });
    expect(retirePreview(v, unit(9), false)).toBeNull();
    expect(retirePreview(v, null, false)).toBeNull();
    expect(retirePreview(v, null, true)?.places).toHaveLength(2);
    const none = withActions({ retire: { retired: 0, target: 3, level: 4, shrines: [], veterans: [{ unit: 5, level: 4, blocker: 'no shrine' }] } });
    expect(retirePreview(none, unit(5), true)).toBeNull();
  });

  it('previews the camps the call raises, city by city, and marks them on the map', () => {
    const v = withActions({ wild: { blocker: null, with_camp: 0, cities: 2, sites: [{ city: 1, at: { q: 1, r: 4 } }, { city: 2, at: { q: 5, r: 8 } }, { city: 99, at: { q: 0, r: 0 } }] } });
    expect(wildRows(v).map((r) => [r.city.name, r.hexes])).toEqual([['Hive', 3], ['Cell', 3]]);
    expect(wildPreview(v)).toEqual({ cities: [{ q: 1, r: 1 }, { q: 5, r: 5 }], released: [], camps: [{ q: 1, r: 4 }, { q: 5, r: 8 }], places: [] });
    expect(wildPreview(view())).toBeNull();
  });
});

describe('the The New Fire preview', () => {
  it('names each city with its loss and tiles, and totals the population', () => {
    const v = view();
    expect(swarmRows(v).map((r) => [r.city.name, r.lost, r.tiles.length])).toEqual([['Hive', 27, 2], ['Cell', 23, 1]]);
    expect(swarmPreview(v)).toEqual({ cities: [{ q: 1, r: 1 }, { q: 5, r: 5 }], released: [{ q: 2, r: 1 }, { q: 3, r: 1 }, { q: 6, r: 5 }], camps: [], places: [] });
    expect(swarmTotals(v)).toEqual({ before: 52, after: 2 });
    expect(swarmPreview(view({ actions: {} }))).toBeNull();
  });
});

describe('the free technology picker', () => {
  const research = (known: TechId[]): PlayerView['research'] => {
    const era = known.length >= 5 ? 1 : 0;
    return { known, era, era_total: 5, era_known: Math.min(known.length, 5), catalog: TECHS.map((tech) => ({ tech, researchable: !known.includes(tech) && TECH_ERA[tech] === era })) } as unknown as PlayerView['research'];
  };
  const known: TechId[] = ['BronzeWorking'];

  it('asks for the pending grant, or what is left to learn in the current era', () => {
    expect(freeTechTarget(view({ research: research(known) }))).toBe(3);
    expect(freeTechTarget(view({ research: research(['BronzeWorking', 'Irrigation', 'Masonry', 'Currency']) }))).toBe(1);
    expect(freeTechTarget(view({ actions: {} }))).toBe(0);
  });

  it('classifies techs: only unknown ones of the current era are open', () => {
    const r = research(known);
    expect(pickState(r, [], 'BronzeWorking')).toBe('known');
    expect(pickState(r, [], 'Irrigation')).toBe('open');
    expect(pickState(r, ['Irrigation'], 'Irrigation')).toBe('chosen');
    expect(pickState(r, [], 'Mathematics')).toBe('blocked');
  });

  it('chooses within the limit and unchooses with a second click', () => {
    const r = research(known);
    let chosen = togglePick(r, [], 'Mathematics', 3);
    expect(chosen).toEqual([]);
    chosen = togglePick(r, chosen, 'Irrigation', 3);
    chosen = togglePick(r, chosen, 'Masonry', 3);
    expect(chosen).toEqual(['Irrigation', 'Masonry']);
    expect(togglePick(r, chosen, 'Irrigation', 3)).toEqual(['Masonry']);
    // At the limit nothing more is added.
    expect(togglePick(r, chosen, 'Currency', 2)).toEqual(chosen);
  });
});
