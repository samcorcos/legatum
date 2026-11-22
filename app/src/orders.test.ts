import { describe, expect, it } from 'vitest';
import type { Order, PlayerView, UnitView } from './types';
import {
  clearUnitOrder,
  describeOrder,
  errorsWhere,
  hasBuy,
  nextUnitNeedingOrders,
  pendingProduction,
  pendingResearch,
  pendingUnitOrder,
  removeOrderAt,
  setProduction,
  setResearch,
  setUnitOrder,
  toggleBuy,
  toggleBuyTile,
  toggleHire,
  pendingTile,
  unitsNeedingOrders,
} from './orders';

const unit = (id: number, over: Partial<UnitView> = {}): UnitView => ({
  id,
  owner: 0,
  kind: 'Spearman',
  pos: { q: id, r: 0 },
  hp: 100,
  moves_left: 2,
  max_moves: 2,
  goto: null,
  fortify: 0,
  strength: 20,
  ranged: 0,
  range: 0,
  military: true,
  upgrade_cost: null,
  ...over,
});

const view = (units: UnitView[], cities: { id: number; owner: number; name: string }[] = []) =>
  ({
    me: 0,
    players: [{ id: 0, faction: 'India' }],
    units,
    cities: cities.map((c) => ({ ...c, pos: { q: 0, r: 0 } })),
  }) as unknown as PlayerView;

describe('unit orders', () => {
  it('one action per unit: a new action replaces the old one', () => {
    let o: Order[] = [];
    o = setUnitOrder(o, { type: 'MoveUnit', unit: 1, to: { q: 3, r: 0 } });
    o = setUnitOrder(o, { type: 'SleepUnit', unit: 2 });
    o = setUnitOrder(o, { type: 'FoundCity', unit: 1 });
    expect(o).toHaveLength(2);
    expect(pendingUnitOrder(o, 1)?.type).toBe('FoundCity');
    expect(pendingUnitOrder(o, 2)?.type).toBe('SleepUnit');
  });

  it('rejects non-unit orders and clears', () => {
    expect(() => setUnitOrder([], { type: 'SetResearch', tech: 'Currency' })).toThrow();
    const o = setUnitOrder([], { type: 'SleepUnit', unit: 5 });
    expect(clearUnitOrder(o, 5)).toEqual([]);
    expect(clearUnitOrder(o, 6)).toEqual(o);
  });

  it('city and research orders stay alongside unit orders', () => {
    let o: Order[] = [{ type: 'SetResearch', tech: 'BronzeWorking' }];
    o = setUnitOrder(o, { type: 'SleepUnit', unit: 1 });
    expect(o).toHaveLength(2);
  });
});

describe('city orders', () => {
  it('production replaces per city and choosing the current item cancels', () => {
    const spearman = { kind: 'Unit', unit: 'Spearman' } as const;
    const granary = { kind: 'Building', building: 'Granary' } as const;
    let o: Order[] = [];
    o = setProduction(o, 7, spearman, null);
    o = setProduction(o, 7, granary, null);
    o = setProduction(o, 8, spearman, null);
    expect(o).toHaveLength(2);
    expect(pendingProduction(o, 7)).toEqual(granary);
    expect(setProduction(o, 7, spearman, spearman).filter((x) => x.type === 'SetProduction' && x.city === 7)).toHaveLength(0);
  });

  it('buy toggles and is at most one per city', () => {
    let o = toggleBuy([], 3, { kind: 'Unit', unit: 'Spearman' });
    expect(hasBuy(o, 3)).toBe(true);
    o = toggleBuy(o, 4, { kind: 'Unit', unit: 'Spearman' });
    o = toggleBuy(o, 3, { kind: 'Unit', unit: 'Spearman' });
    expect(hasBuy(o, 3)).toBe(false);
    expect(o).toEqual([{ type: 'BuyItem', city: 4, item: { kind: 'Unit', unit: 'Spearman' } }]);
  });

  it('a tile purchase shares the city purchase slot with an item and a hire, and toggles off', () => {
    const tile = { q: 3, r: 2 };
    const spearman = { kind: 'Unit', unit: 'Spearman' } as const;
    let o = toggleBuyTile([], 3, tile);
    expect(o).toEqual([{ type: 'BuyTile', city: 3, tile }]);
    expect(hasBuy(o, 3)).toBe(true);
    expect(pendingTile(o, 3)).toEqual({ type: 'BuyTile', city: 3, tile });
    // Another tile replaces it; an item replaces the tile; the tile replaces an item and a hire.
    o = toggleBuyTile(o, 3, { q: 4, r: 2 });
    expect(o).toEqual([{ type: 'BuyTile', city: 3, tile: { q: 4, r: 2 } }]);
    o = toggleBuy(o, 3, spearman);
    expect(o).toEqual([{ type: 'BuyItem', city: 3, item: spearman }]);
    o = toggleBuyTile(o, 3, tile);
    expect(o).toEqual([{ type: 'BuyTile', city: 3, tile }]);
    const hire = { type: 'HireCompany', city: 3, unit: 'Spearman', seller: 1 } as const;
    expect(toggleBuyTile(toggleHire([], hire), 3, tile)).toEqual([{ type: 'BuyTile', city: 3, tile }]);
    // Other cities are untouched, and clicking the queued tile cancels it.
    o = toggleBuyTile([{ type: 'BuyItem', city: 4, item: spearman }, ...o], 3, tile);
    expect(o).toEqual([{ type: 'BuyItem', city: 4, item: spearman }]);
  });

  it('changes production without changing the purchased item or its spending order', () => {
    const granary = { kind: 'Building', building: 'Granary' } as const;
    const spearman = { kind: 'Unit', unit: 'Spearman' } as const;
    const scout = { kind: 'Unit', unit: 'Scout' } as const;
    const original: Order[] = [{ type: 'BuyItem', city: 7, item: { kind: 'Unit', unit: 'Spearman' } }, { type: 'BuyItem', city: 8, item: { kind: 'Unit', unit: 'Spearman' } }];
    const changed = setProduction(original, 7, spearman, granary);
    expect(changed).toEqual([...original, { type: 'SetProduction', city: 7, item: spearman }]);
    const replaced = setProduction(changed, 7, scout, granary);
    expect(replaced).toEqual([...original, { type: 'SetProduction', city: 7, item: scout }]);
    expect(setProduction(replaced, 7, granary, granary)).toEqual(original);
    expect(toggleBuy(changed, 7, { kind: 'Unit', unit: 'Spearman' })).toEqual([
      { type: 'BuyItem', city: 8, item: { kind: 'Unit', unit: 'Spearman' } }, { type: 'SetProduction', city: 7, item: spearman },
    ]);
  });

  it('research: single choice, current tech cancels', () => {
    let o = setResearch([], 'Currency', null);
    o = setResearch(o, 'BronzeWorking', null);
    expect(pendingResearch(o)).toBe('BronzeWorking');
    expect(setResearch(o, 'Feudalism', 'Feudalism')).toEqual([]);
  });

  it('removeOrderAt removes by index', () => {
    const o: Order[] = [
      { type: 'SleepUnit', unit: 1 },
      { type: 'SleepUnit', unit: 2 },
    ];
    expect(removeOrderAt(o, 0)).toEqual([{ type: 'SleepUnit', unit: 2 }]);
  });
});

describe('next unit needing orders', () => {
  const v = view([
    unit(1),
    unit(2, { goto: { q: 9, r: 9 } }),
    unit(3, { fortify: 3, standing_order: 'Sleep' }),
    unit(4, { kind: 'Settler', military: false }),
    unit(5, { owner: 1 }),
  ]);

  it('lists idle own units without destinations, an explicit Fortify/Wait order or a pending order', () => {
    expect(unitsNeedingOrders(v, []).map((u) => u.id)).toEqual([1, 4]);
    expect(unitsNeedingOrders(v, [{ type: 'SleepUnit', unit: 1 }]).map((u) => u.id)).toEqual([4]);
    expect(unitsNeedingOrders(v, [{ type: 'SkipUnit', unit: 4 }]).map((u) => u.id)).toEqual([1]);
  });

  it('cycles after the current unit and wraps', () => {
    expect(nextUnitNeedingOrders(v, [], null)?.id).toBe(1);
    expect(nextUnitNeedingOrders(v, [], 1)?.id).toBe(4);
    expect(nextUnitNeedingOrders(v, [], 4)?.id).toBe(1);
    expect(nextUnitNeedingOrders(v, [{ type: 'SleepUnit', unit: 1 }, { type: 'SleepUnit', unit: 4 }], null)).toBeNull();
  });

  it('keeps a healing unit out of the cycle until it wakes healed, then returns it', () => {
    const healing = view([unit(1, { hp: 60, standing_order: 'Heal' }), unit(2, { hp: 100, needs_orders: true })]);
    expect(unitsNeedingOrders(healing, []).map((u) => u.id)).toEqual([2]);
    expect(unitsNeedingOrders(healing, [{ type: 'SleepUnit', unit: 2, until_healed: true }]).map((u) => u.id)).toEqual([]);
    expect(describeOrder({ type: 'SleepUnit', unit: 1, until_healed: true }, healing)).toBe('Spearman: healing until full health, then asks for orders');
    // A later plain Fortify replaces a pending Heal for the same unit.
    const replaced = setUnitOrder([{ type: 'SleepUnit', unit: 1, until_healed: true }], { type: 'SleepUnit', unit: 1 });
    expect(replaced).toEqual([{ type: 'SleepUnit', unit: 1 }]);
  });

  it('returns sleeping units whose automation stopped to the cycle', () => {
    const stopped = view([unit(1, { fortify: 6, needs_orders: true }), unit(2, { fortify: 6, standing_order: 'Sleep' })]);
    expect(nextUnitNeedingOrders(stopped, [], null)?.id).toBe(1);
    expect(nextUnitNeedingOrders(stopped, [], 1)?.id).toBe(1);
    expect(nextUnitNeedingOrders(stopped, [{ type: 'SleepUnit', unit: 1 }], null)).toBeNull();
    expect(nextUnitNeedingOrders(stopped, [{ type: 'SkipUnit', unit: 1 }], null)).toBeNull();
  });
});

describe('errors, pruning, descriptions', () => {
  it('maps validation errors to the orders they belong to', () => {
    const o: Order[] = [
      { type: 'SleepUnit', unit: 1 },
      { type: 'BuyItem', city: 9, item: { kind: 'Unit', unit: 'Spearman' } },
    ];
    const errs = [{ index: 1, message: 'Not enough Gold' }];
    expect(errorsWhere(o, errs, (x) => x.type === 'BuyItem')).toEqual(['Not enough Gold']);
    expect(errorsWhere(o, errs, (x) => x.type === 'SleepUnit')).toEqual([]);
  });

  it('describes orders in plain words', () => {
    const v = view([unit(1)], [{ id: 5, owner: 0, name: 'Tithe' }]);
    expect(describeOrder({ type: 'MoveUnit', unit: 1, to: { q: 0, r: 2 } }, v)).toBe('Spearman: move to 2 tiles SE of Tithe');
    expect(describeOrder({ type: 'SetProduction', city: 5, item: { kind: 'Building', building: 'GreatLibrary' } }, v)).toBe(
      'Tithe: build Great Library',
    );
    expect(describeOrder({ type: 'SetResearch', tech: 'Writing' }, v)).toBe('Research Writing');
    expect(describeOrder({ type: 'SetProduction', city: 5, item: { kind: 'Building', building: 'RiceStorehouse' } }, v)).toBe('Tithe: build Rice Storehouse');
    v.players[0].faction = 'Sparta';
    expect(describeOrder({ type: 'SleepUnit', unit: 1 }, v)).toBe('Spartan Hoplite: fortified until woken');
  });

  it('names a ranged target (its city, or whose unit), never a raw coordinate', () => {
    const v = view([unit(1, { kind: 'Archer' }), unit(4, { owner: 1, kind: 'Scout', pos: { q: 4, r: 0 } })], [{ id: 5, owner: 1, name: 'Harmonia' }]);
    v.players.push({ id: 1, faction: 'Switzerland', team: 1, name: 'AI 2' } as PlayerView['players'][number]);
    expect(describeOrder({ type: 'RangedAttack', unit: 1, target: { q: 0, r: 0 } }, v)).toBe('Archer: ranged attack on Harmonia');
    expect(describeOrder({ type: 'RangedAttack', unit: 1, target: { q: 4, r: 0 } }, v)).toMatch(/^Archer: ranged attack on \S.* Scout$/);
    expect(describeOrder({ type: 'RangedAttack', unit: 1, target: { q: 2, r: 0 } }, v)).toBe('Archer: ranged attack on 2 tiles E of Harmonia');
  });
});
