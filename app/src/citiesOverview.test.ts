import { describe, expect, it } from 'vitest';
import { cityRows, DEFAULT_CITY_SORT, nextSort, sortRows, type CityRow } from './citiesOverview';
import { FixtureClient } from './test/fixtureEngine';
import { citiesFixture } from './test/citiesFixture';
import type { Order } from './types';

function fixtureView() {
  const client = new FixtureClient();
  client.load(JSON.stringify(citiesFixture()));
  return client.view(0);
}

const row = (id: number, name: string, over: Partial<CityRow> = {}): CityRow => ({
  id, name, isCapital: false, population: 3, growth: null, growthText: '3 turns', growthBad: false, foodNet: 1, food: 4, production: 5, gold: 1, science: 1,
  strength: 20, hp: 200, maxHp: 200, fort: null, strengthRule: '', build: null, idle: false, ...over,
});

describe('the rows', () => {
  it('has one row per city of yours with the numbers the city panel shows', () => {
    const view = fixtureView();
    const rows = cityRows(view, []);
    const mine = view.cities.filter((c) => c.owner === view.me);
    expect(rows.map((r) => r.id).sort()).toEqual(mine.map((c) => c.id).sort());
    for (const r of rows) {
      const c = mine.find((m) => m.id === r.id)!, d = c.detail!;
      expect([r.food, r.production, r.gold, r.science]).toEqual([d.yields.total.food, d.yields.total.production, d.yields.total.gold, d.yields.total.science]);
      expect([r.population, r.strength, r.hp]).toEqual([c.population, c.strength, c.hp]);
      expect(r.build?.turns ?? null).toBe(d.producing ? d.turns_to_complete : null);
    }
  });

  it('flags the city with nothing chosen as idle, and a pending order clears it', () => {
    const view = fixtureView();
    const geneva = view.cities.find((c) => c.name === 'Geneva')!;
    expect(cityRows(view, []).filter((r) => r.idle).map((r) => r.name)).toEqual(['Geneva']);
    const choose: Order[] = [{ type: 'SetProduction', city: geneva.id, item: { kind: 'Unit', unit: 'Scout' } }];
    const after = cityRows(view, choose).find((r) => r.name === 'Geneva')!;
    expect(after.idle).toBe(false);
    expect(after.build).toMatchObject({ pending: true, state: 'building' });
  });
});

describe('sorting', () => {
  const rows = [row(1, 'Bern', { production: 12, population: 6 }), row(2, 'Zurich', { production: 7, population: 4 }), row(3, 'Basel', { production: 12, population: 7 })];
  it('opens on Production, highest first, and breaks a tie by name', () => {
    expect(sortRows(rows, DEFAULT_CITY_SORT).map((r) => r.name)).toEqual(['Basel', 'Bern', 'Zurich']);
  });
  it('sorts any column, either way', () => {
    expect(sortRows(rows, { key: 'population', dir: 'desc' }).map((r) => r.name)).toEqual(['Basel', 'Bern', 'Zurich']);
    expect(sortRows(rows, { key: 'population', dir: 'asc' }).map((r) => r.name)).toEqual(['Zurich', 'Bern', 'Basel']);
    expect(sortRows(rows, { key: 'name', dir: 'asc' }).map((r) => r.name)).toEqual(['Basel', 'Bern', 'Zurich']);
    expect(sortRows(rows, { key: 'name', dir: 'desc' }).map((r) => r.name)).toEqual(['Zurich', 'Bern', 'Basel']);
  });
  it('puts idle cities first, then the nearest to finishing, then waiting ones, when sorting by what is building', () => {
    const build = (turns: number | null) => ({ item: { kind: 'Unit' as const, unit: 'Scout' as const }, name: 'Scout', stored: 0, cost: 30, turns, state: turns === null ? 'waiting' as const : 'building' as const, pending: false });
    const list = [row(1, 'A', { build: build(9) }), row(2, 'B', { build: build(null) }), row(3, 'C', { idle: true }), row(4, 'D', { build: build(2) })];
    expect(sortRows(list, { key: 'build', dir: 'asc' }).map((r) => r.name)).toEqual(['C', 'D', 'A', 'B']);
  });
  it('flips a column on a second click, and starts each new one in its natural direction', () => {
    expect(nextSort({ key: 'production', dir: 'desc' }, 'production')).toEqual({ key: 'production', dir: 'asc' });
    expect(nextSort({ key: 'production', dir: 'desc' }, 'name')).toEqual({ key: 'name', dir: 'asc' });
    expect(nextSort({ key: 'name', dir: 'asc' }, 'gold')).toEqual({ key: 'gold', dir: 'desc' });
    expect(nextSort({ key: 'gold', dir: 'desc' }, 'build')).toEqual({ key: 'build', dir: 'asc' });
  });
});
