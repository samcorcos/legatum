import { describe, expect, it } from 'vitest';
import { itemName } from './data';
import type { CityView, Order, PlayerView, ProductionItem, ProductionOption, TechUnlock, UnavailableOption } from './types';
import { COSTLY_LISTED_COST, fastestSite, plainReason, showsWhereToBuild, sortSites, whereSummary, whereToBuild, type BuildSite } from './whereToBuild';

const GL: ProductionItem = { kind: 'Building', building: 'GreatLibrary' };
const MARKET: ProductionItem = { kind: 'Building', building: 'Market' };
const FACTORY: ProductionItem = { kind: 'Building', building: 'Factory' };
const SPEARMAN: ProductionItem = { kind: 'Unit', unit: 'Spearman' };

const option = (item: ProductionItem, cost: number, stored: number, turns: number): ProductionOption =>
  ({ item, cost, production_stored: stored, production_banked: stored, buy_cost: null, yields: {} as never, turns }) as ProductionOption;

interface CityOpts { id: number; name: string; options?: ProductionOption[]; unavailable?: UnavailableOption[]; producing?: ProductionItem | null; queue?: ProductionItem[]; turnsToComplete?: number | null }
const city = (o: CityOpts): CityView => ({
  id: o.id, owner: 0, name: o.name, pos: { q: o.id, r: 0 }, population: 4, hp: 200, max_hp: 200, strength: 20, is_capital: o.id === 1, walls: false, visible: true,
  detail: {
    options: o.options ?? [], unavailable: o.unavailable ?? [], producing: o.producing ?? null, production_queue: o.queue ?? [],
    turns_to_complete: o.turnsToComplete ?? null,
  },
}) as unknown as CityView;

const unlock = (item: ProductionItem, tech: string, cost: number, cities: Record<number, { blocker?: string; turns?: number }>): TechUnlock =>
  ({ tech, item, name: 'x', cost, cost_breakdown: { total: cost, parts: [['Listed cost', cost]] }, cities: Object.entries(cities).map(([id, c]) => ({ city: Number(id), blocker: c.blocker ?? null, build_turns: c.turns ?? null })) }) as unknown as TechUnlock;

function view(cities: CityView[], unlocks: TechUnlock[], known: string[] = ['Philosophy', 'Electricity', 'Currency']): PlayerView {
  return { me: 0, cities, players: [{ id: 0, faction: 'Switzerland' }], research: { known, unlocks } } as unknown as PlayerView;
}

describe('where to build', () => {
  const bern = () => city({ id: 1, name: 'Bern', options: [option(GL, 600, 0, 37)] });
  const zurich = () => city({ id: 2, name: 'Zurich', options: [option(GL, 600, 120, 22)] });
  const basel = () => city({ id: 3, name: 'Basel' });
  const gl = unlock(GL, 'Philosophy', 600, { 1: {}, 2: {}, 3: { blocker: 'Build in a city with Library' } });

  it("takes each city's turns from the core's option, so banked progress counts, and names the fastest", () => {
    const w = whereToBuild(view([bern(), zurich(), basel()], [gl]), GL, []);
    expect(w.sites.map((s) => [s.name, s.state, s.turns])).toEqual([['Zurich', 'ready', 22], ['Bern', 'ready', 37], ['Basel', 'blocked', null]]);
    expect(w.fastest?.name).toBe('Zurich');
    expect(w.sites[0].stored).toBe(120);
    expect(whereSummary(w).text).toBe('Best city for this: Zurich, done in 22 turns.');
    expect(whereSummary(w, 1).text).toBe('Best city for this: Zurich, done in 22 turns. Here: 37 turns.');
    expect(whereSummary(w, 2).text).toBe('Best city for this: here, done in 22 turns.');
    expect(whereSummary(w, 3).text).toBe('Best city for this: Zurich, done in 22 turns. Not here yet: needs a Library first.');
  });

  it('calls a tie "fastest here" in the panel of a city that matches the best', () => {
    const tied = city({ id: 1, name: 'Bern', options: [option(GL, 600, 0, 22)] });
    const w = whereToBuild(view([tied, zurich()], [gl]), GL, []);
    expect(whereSummary(w, 1).text).toBe('Best city for this: here, done in 22 turns.');
    expect(whereSummary(w).text).toBe('Best city for this: Zurich, done in 22 turns.');
  });

  it('says plainly why a city cannot build it', () => {
    const w = whereToBuild(view([bern(), basel()], [gl]), GL, []);
    expect(w.sites.find((s) => s.name === 'Basel')).toMatchObject({ state: 'blocked', reason: 'Needs a Library first', turns: null });
  });

  it('breaks a tie on turns by the progress already made, then by name', () => {
    const a = city({ id: 1, name: 'Ur', options: [option(GL, 600, 0, 10)] });
    const b = city({ id: 2, name: 'Kish', options: [option(GL, 600, 50, 10)] });
    const c = city({ id: 3, name: 'Eridu', options: [option(GL, 600, 50, 10)] });
    expect(whereToBuild(view([a, b, c], [gl]), GL, []).sites.map((s) => s.name)).toEqual(['Eridu', 'Kish', 'Ur']);
  });

  it('never picks a city with no output as the fastest, and says so when none can build it', () => {
    const stuck = city({ id: 1, name: 'Ur', options: [option(GL, 600, 0, -1)] });
    const w = whereToBuild(view([stuck, basel()], [gl]), GL, []);
    expect(w.sites[0].turns).toBeNull();
    expect(w.fastest).toBeNull();
    expect(whereSummary(w).text).toBe('No city can build it now: needs a Library first');
  });

  it('says where the one allowed Great Work is already being built, and the summary follows it', () => {
    const building = city({ id: 1, name: 'Bern', options: [option(GL, 600, 300, 8)], producing: GL, turnsToComplete: 8 });
    const other = city({ id: 2, name: 'Zurich', unavailable: [] });
    const w = whereToBuild(view([building, other], [gl]), GL, []);
    expect(w.sites.find((s) => s.name === 'Bern')).toMatchObject({ state: 'building', turns: 8 });
    expect(w.sites.find((s) => s.name === 'Zurich')).toMatchObject({ state: 'elsewhere', reason: 'Already building in Bern; one per empire' });
    expect(whereSummary(w).text).toBe('Building in Bern; 8 turns left');
    expect(whereSummary(w, 1).text).toBe('Building here; 8 turns left');
  });

  it('counts a choice made this turn (a pending order) as taken, before the core knows', () => {
    const orders: Order[] = [{ type: 'SetProduction', city: 2, item: GL }];
    const w = whereToBuild(view([bern(), zurich()], [gl]), GL, orders);
    expect(w.sites.find((s) => s.name === 'Zurich')).toMatchObject({ state: 'building', pending: true });
    expect(w.sites.find((s) => s.name === 'Bern')).toMatchObject({ state: 'elsewhere', reason: 'Already building in Zurich; one per empire' });
  });

  it('says what choosing it in a busy city would replace', () => {
    const busy = city({ id: 1, name: 'Bern', options: [option(GL, 600, 0, 37)], producing: MARKET });
    expect(whereToBuild(view([busy, zurich()], [gl]), GL, []).sites.find((s) => s.name === 'Bern')?.replaces).toBe(itemName(MARKET, 'Switzerland'));
    const idle = whereToBuild(view([bern(), zurich()], [gl]), GL, []).sites.find((s) => s.name === 'Bern')!;
    expect(idle).toMatchObject({ replaces: null, idle: true });
  });

  it('previews an item that is not researched yet from the research forecast, with no actions', () => {
    const later = unlock(GL, 'Philosophy', 600, { 1: { turns: 30 }, 2: { turns: 20 }, 3: { blocker: 'Build in a city with Library' } });
    const w = whereToBuild(view([city({ id: 1, name: 'Bern' }), city({ id: 2, name: 'Zurich' }), city({ id: 3, name: 'Basel' })], [later], []), GL, []);
    expect(w.researched).toBe(false);
    expect(w.sites.map((s) => [s.name, s.state, s.turns])).toEqual([['Zurich', 'later', 20], ['Bern', 'later', 30], ['Basel', 'blocked', null]]);
    expect(whereSummary(w).text).toBe('Best city for this once researched: Zurich, done in 20 turns.');
  });

  it('is worth showing for Great Works and costly buildings with two or more cities, and for nothing else', () => {
    const cheap = unlock(MARKET, 'Currency', 120, { 1: {}, 2: {} });
    const big = unlock(FACTORY, 'Industrialization', COSTLY_LISTED_COST, { 1: {}, 2: {} });
    const two = [city({ id: 1, name: 'A', options: [option(GL, 600, 0, 9), option(MARKET, 120, 0, 3), option(FACTORY, COSTLY_LISTED_COST, 0, 9), option(SPEARMAN, 60, 0, 2)] }),
      city({ id: 2, name: 'B', options: [option(GL, 600, 0, 9), option(MARKET, 120, 0, 3), option(FACTORY, COSTLY_LISTED_COST, 0, 9), option(SPEARMAN, 60, 0, 2)] })];
    const v = view(two, [gl, cheap, big]);
    expect(showsWhereToBuild(whereToBuild(v, GL, []))).toBe(true);
    expect(showsWhereToBuild(whereToBuild(v, FACTORY, []))).toBe(true);
    expect(showsWhereToBuild(whereToBuild(v, MARKET, []))).toBe(false);
    expect(showsWhereToBuild(whereToBuild(v, SPEARMAN, []))).toBe(false);
    expect(showsWhereToBuild(whereToBuild(view([two[0]], [gl]), GL, []))).toBe(false);
  });

  it('ignores cities that are not yours and cities you cannot see into', () => {
    const foreign = { ...bern(), id: 9, owner: 1 } as CityView;
    const unseen = { ...zurich(), id: 8, detail: null } as CityView;
    expect(whereToBuild(view([bern(), foreign, unseen], [gl]), GL, []).sites.map((s) => s.name)).toEqual(['Bern']);
  });
});

describe('plain reasons', () => {
  it.each([
    ['Build in a city with Library', 'Needs a Library first'],
    ['Requires Market', 'Needs a Market first'],
    ['Requires population 5 (currently 3)', 'Needs population 5 (has 3)'],
    ['Requires 2 owned Bank buildings (currently 1)', 'Needs 2 Bank buildings (you have 1)'],
    ['Resisting occupation', 'Resisting occupation'],
    ['This empire has already completed its world wonder', 'Your empire has already finished its world wonder'],
  ])('%s', (core, plain) => expect(plainReason(core)).toBe(plain));
});

describe('ordering', () => {
  const site = (name: string, turns: number | null, stored = 0): BuildSite => ({ cityId: 1, name, state: 'ready', turns, stored, cost: 100, reason: null, replaces: null, idle: false, pending: false, position: null });
  it('puts the fastest first, cities with no answer last', () => {
    const sites = [site('C', null), site('B', 12), site('A', 5), site('D', 12, 30)];
    expect(sortSites(sites).map((s) => s.name)).toEqual(['A', 'D', 'B', 'C']);
    expect(fastestSite(sites)?.name).toBe('A');
    expect(fastestSite([site('X', null)])).toBeNull();
  });
});
