import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { turnFlowFixture } from '../test/turnFlowFixture';
import { coreFixture } from '../test/coreFixture';
import { unitsNeedingOrders } from '../orders';
import { contextFromViews, formatEvents } from '../format';
import type { GameClient } from '../client';
import { explorationFixture } from '../test/explorationFixture';
import { spawnUnit } from './fixtureEngine';
import { fromOffset } from '../hex';
import type { Event, Order } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

for (const engine of ['wasm'] as const) {
  function start(fixture = turnFlowFixture()): GameClient {
    const client = makeCore();
    client.load(coreFixture(fixture));
    return client;
  }
  it.skipIf(engine === 'wasm' && !available)(`${engine}: Sleep heals through a save and stays asleep at full HP without a notice`, () => {
    const client = start();
    client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }, { type: 'SleepUnit', unit: 4 }] });
    let view = client.view(0);
    expect(view.units.find((u) => u.id === 0)).toMatchObject({ hp: 95, standing_order: 'Sleep' });
    expect(unitsNeedingOrders(view, [])).toHaveLength(0);
    client.load(client.save());
    const result = client.endTurn({});
    view = client.view(0);
    expect(view.units.find((u) => u.id === 0)).toMatchObject({ hp: 100, standing_order: 'Sleep' });
    expect(view.units.find((u) => u.id === 0)?.needs_orders).toBeFalsy();
    expect(view.units.find((u) => u.id === 4)?.standing_order).toBe('Sleep');
    expect(unitsNeedingOrders(view, [])).toHaveLength(0);
    expect(result.events.some((e) => e.type === 'UnitAwakened')).toBe(false);
  });

  it.skipIf(engine === 'wasm' && !available)(`${engine}: Heal rests like Sleep, wakes at full HP with one private notice and returns to the queue`, () => {
    const client = start();
    const heal = { type: 'SleepUnit', unit: 0, until_healed: true } as const;
    const unit = () => client.view(0).units.find((u) => u.id === 0)!;
    expect(client.validate(0, [heal])).toEqual([]);
    // 75 HP in own territory heals 20 a turn: 95 after the first, full on the second.
    const first = client.endTurn({ 0: [heal] });
    expect(unit()).toMatchObject({ hp: 95, standing_order: 'Heal' });
    expect(first.events.some((e) => e.type === 'UnitHealed')).toBe(false);
    expect(unitsNeedingOrders(client.view(0), []).some((u) => u.id === 0)).toBe(false);
    client.load(client.save());
    expect(unit()).toMatchObject({ hp: 95, standing_order: 'Heal' });
    const before = client.view(0);
    const third = client.endTurn({});
    const after = client.view(0);
    expect(unit()).toMatchObject({ hp: 100, needs_orders: true });
    expect(unit().standing_order).toBeUndefined();
    expect(unitsNeedingOrders(after, []).some((u) => u.id === 0)).toBe(true);
    expect(third.events.filter((e) => e.type === 'UnitHealed')).toEqual([
      { type: 'UnitHealed', unit: 0, owner: 0, unit_kind: 'Spearman', at: unit().pos },
    ]);
    // Visible to the owner's team only, and the report says so in plain words.
    expect(third.events_by_player?.[1]?.some((e) => e.type === 'UnitHealed')).toBe(false);
    const lines = formatEvents(third.events, contextFromViews(before, after));
    expect(lines.some((l) => l.text.includes('healed to full') && l.tone === 'good')).toBe(true);
    // Ordering Heal again at full health is refused with a reason; the notice never repeats.
    expect(client.validate(0, [heal]).map((e) => e.message)).toEqual([expect.stringContaining('full health')]);
    expect(client.endTurn({}).events.some((e) => e.type === 'UnitHealed')).toBe(false);
  });

  it.skipIf(engine === 'wasm' && !available)(`${engine}: a healing unit wakes early for a new enemy in sight, still hurt`, () => {
    const { fixture, enemy } = threatFixture('Spearman');
    fixture.units[0].hp = 40;
    const client = start(fixture);
    client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0, until_healed: true }] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Heal');
    const result = client.endTurn({ 1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(8, 4) }] });
    const woken = client.view(0).units.find((u) => u.id === 0)!;
    expect(woken.standing_order).toBeUndefined();
    expect(woken.hp).toBeLessThan(100);
    expect(unitsNeedingOrders(client.view(0), []).some((u) => u.id === 0)).toBe(true);
    expect(result.events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
    expect(result.events.some((e) => e.type === 'UnitHealed')).toBe(false);
  });

  function threatFixture(kind: 'Settler' | 'Spearman' | 'Scout', enemyKind: 'Spearman' | 'Settler' = 'Spearman', enemyColumn = 10) {
    const fixture = explorationFixture();
    fixture.terrain.fill('Plains'); fixture.ruin.fill(0);
    fixture.players[1].isHuman = true; fixture.setup.players[1].is_human = true;
    fixture.units[0].kind = kind; fixture.units[0].pos = fromOffset(6, 4);
    fixture.units[1].pos = fromOffset(0, 0); fixture.units[2].pos = fromOffset(19, 9);
    for (const player of fixture.players) player.explored.fill(1);
    const enemy = spawnUnit(fixture, 1, enemyKind, fromOffset(enemyColumn, 4), false);
    // Only armies of a team at War are threats; these tactical fixtures start at War.
    for (const r of fixture.diplomacy!.relations) r.state = 'War';
    return { fixture, enemy: enemy.id };
  }

  it.skipIf(engine === 'wasm' && !available)(`${engine}: an approaching army of a team at Peace never wakes a sleeper`, () => {
    const { fixture, enemy } = threatFixture('Spearman');
    for (const r of fixture.diplomacy!.relations) r.state = 'Peace';
    const client = start(fixture);
    client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }] });
    const result = client.endTurn({ 1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(7, 4) }] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
    expect(result.events.some((e) => e.type === 'UnitAwakened')).toBe(false);
  });

  it.skipIf(engine === 'wasm' && !available).each(['Settler', 'Spearman'] as const)(`${engine}: sleeping %s wakes from an approaching army with one private notice after saving`, (kind) => {
    const { fixture, enemy } = threatFixture(kind);
    const client = start(fixture);
    client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
    client.load(client.save());
    const before = client.view(0);
    const result = client.endTurn({ 1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(8, 4) }] });
    const after = client.view(0);
    expect(after.units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
    expect(unitsNeedingOrders(after, []).some((u) => u.id === 0)).toBe(true);
    expect(result.events.filter((e) => e.type === 'UnitAwakened')).toEqual([
      { type: 'UnitAwakened', unit: 0, owner: 0, unit_kind: kind, at: fromOffset(6, 4) },
    ]);
    const lines = formatEvents(result.events, contextFromViews(before, after));
    expect(lines.some((l) => l.text.includes('woke: enemy military in sight') && l.at?.q === fromOffset(6, 4).q)).toBe(true);
    expect(client.endTurn({}).events.some((e) => e.type === 'UnitAwakened')).toBe(false);
  });

  it.skipIf(engine === 'wasm' && !available).each(['Settler', 'Spearman'] as const)(`${engine}: a declaration wakes a sleeping %s for an existing peaceful army only once`, (kind) => {
    for (const declarer of [0, 1]) {
      const { fixture, enemy } = threatFixture(kind, 'Spearman', 8);
      for (const r of fixture.diplomacy!.relations) r.state = 'Peace';
      const arrival = spawnUnit(fixture, 1, 'Spearman', fromOffset(6, 8), false);
      const client = start(fixture);
      const sleep = { 0: [{ type: 'SleepUnit', unit: 0 }] } satisfies Record<number, Order[]>;
      const wakes = (events: Event[]) => events.filter((e) => e.type === 'UnitAwakened');
      expect(wakes(client.endTurn(sleep).events)).toEqual([]);
      expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
      expect(client.view(0).units.some((u) => u.id === enemy)).toBe(true);
      client.load(client.save());

      const result = client.endTurn({ [declarer]: [{ type: 'DeclareWar', against: 1 - declarer, breach: false }] });
      expect(result.events.some((e) => e.type === 'WarDeclared')).toBe(true);
      expect(client.view(0).units.find((u) => u.id === 0)).toMatchObject({ needs_orders: true });
      expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
      expect(wakes(result.events)).toEqual([{ type: 'UnitAwakened', unit: 0, owner: 0, unit_kind: kind, at: fromOffset(6, 4) }]);
      expect(wakes(result.events_by_player?.[1] ?? [])).toEqual([]);

      for (const orders of [sleep, {}]) {
        expect(wakes(client.endTurn(orders).events)).toEqual([]);
        expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
        expect(unitsNeedingOrders(client.view(0), []).some((u) => u.id === 0)).toBe(false);
        client.load(client.save());
      }
      const next = client.endTurn({ 1: [{ type: 'MoveUnit', unit: arrival.id, to: fromOffset(6, 6) }] });
      expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
      expect(wakes(next.events)).toHaveLength(1);
    }
  });

  it.skipIf(engine === 'wasm' && !available).each(['Settler', 'Spearman'] as const)(`${engine}: re-slept %s ignores the old contact after saving, then wakes for a new enemy`, (kind) => {
    const { fixture, enemy } = threatFixture(kind);
    const arrival = spawnUnit(fixture, 1, 'Spearman', fromOffset(6, 8), false);
    const client = start(fixture);
    const first = client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }],
      1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(8, 4) }] });
    expect(first.events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
    for (let turn = 0; turn < 3; turn++) {
      const result = client.endTurn(turn === 0 ? { 0: [{ type: 'SleepUnit', unit: 0 }] } : {});
      expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
      expect(unitsNeedingOrders(client.view(0), []).some((u) => u.id === 0)).toBe(false);
      expect(result.events.some((e) => e.type === 'UnitAwakened')).toBe(false);
      client.load(client.save());
    }
    const second = client.endTurn({ 1: [{ type: 'MoveUnit', unit: arrival.id, to: fromOffset(6, 6) }] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
    expect(second.events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
    expect(second.events_by_player?.[1]?.some((e) => e.type === 'UnitAwakened')).toBe(false);
  });

  it.skipIf(engine === 'wasm' && !available)(`${engine}: an army already visible outside movement range wakes Sleep once when it approaches`, () => {
    const { fixture, enemy } = threatFixture('Spearman');
    spawnUnit(fixture, 0, 'Scout', fromOffset(10, 5), false);
    const client = start(fixture);
    expect(client.view(0).units.some((u) => u.id === enemy)).toBe(true);
    expect(client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }] }).events.some((e) => e.type === 'UnitAwakened')).toBe(false);
    const result = client.endTurn({ 1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(8, 4) }] });
    expect(client.view(0).units.find((u) => u.id === enemy)?.pos).toEqual(fromOffset(8, 4));
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
    expect(result.events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
    expect(client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }] }).events.some((e) => e.type === 'UnitAwakened')).toBe(false);
    expect(client.endTurn({}).events.some((e) => e.type === 'UnitAwakened')).toBe(false);
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
  });

  it.skipIf(engine === 'wasm' && !available)(`${engine}: Sleep accepts a visible enemy until it leaves and reenters sight`, () => {
    const { fixture, enemy } = threatFixture('Spearman', 'Spearman', 8);
    const client = start(fixture);
    expect(client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }] }).events.some((e) => e.type === 'UnitAwakened')).toBe(false);
    expect(client.endTurn({ 1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(10, 4) }] }).events.some((e) => e.type === 'UnitAwakened')).toBe(false);
    expect(client.endTurn({ 1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(8, 4) }] }).events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
  });

  it.skipIf(engine === 'wasm' && !available)(`${engine}: a passing enemy wakes Sleep even when it ends out of sight`, () => {
    const { fixture, enemy } = threatFixture('Spearman');
    fixture.units[0].pos = { q: 4, r: 4 };
    Object.assign(fixture.units.find((u) => u.id === enemy)!, { kind: 'Scout', pos: { q: 7, r: 1 } });
    const path = [{ q: 6, r: 2 }, { q: 7, r: 2 }, { q: 8, r: 2 }];
    const client = start(fixture);
    const result = client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }],
      1: [{ type: 'MoveUnitPath', unit: enemy, to: path[2], path }] });
    expect(client.view(0).units.some((u) => u.id === enemy)).toBe(false);
    expect(client.view(0).units.find((u) => u.id === 0)?.needs_orders).toBe(true);
    expect(result.events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
  });

  it.skipIf(engine === 'wasm' && !available).each([false, true])(`${engine}: Sleep and a legacy Fortify standing order both wake for an arriving enemy (legacy=%s)`, (legacy) => {
    const { fixture, enemy } = threatFixture('Settler');
    if (legacy) (fixture.units[0] as { standing_order?: string }).standing_order = 'Fortify';
    const client = start(fixture);
    const result = client.endTurn({ 0: legacy ? [] : [{ type: 'SleepUnit', unit: 0 }],
      1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(8, 4) }] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
    expect(result.events.filter((e) => e.type === 'UnitAwakened')).toHaveLength(1);
  });

  it.skipIf(engine === 'wasm' && !available).each([
    ['Scout', 'Spearman', 9], // within movement 3, outside current sight 2 (already explored)
    ['Spearman', 'Spearman', 9], // outside movement range, even with extra friendly sight
    ['Settler', 'Settler', 7], // enemy civilian in sight is not a military threat
  ] as const)(`${engine}: sleeping %s ignores %s at column %s when not a visible army within range`, (kind, enemyKind, col) => {
    const { fixture, enemy } = threatFixture(kind, enemyKind, col + 2);
    if (kind === 'Spearman') spawnUnit(fixture, 0, 'Scout', fromOffset(10, 5), false);
    const client = start(fixture);
    const result = client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }],
      1: [{ type: 'MoveUnit', unit: enemy, to: fromOffset(col, 4) }] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBe('Sleep');
    expect(result.events.some((e) => e.type === 'UnitAwakened')).toBe(false);
  });

  it.skipIf(engine === 'wasm' && !available).each(['SkipUnit', 'MoveUnit'] as const)(`${engine}: %s cancels Sleep and replaces its pending intent`, (type) => {
    const client = start();
    client.endTurn({ 0: [{ type: 'SleepUnit', unit: 0 }] });
    const to = client.view(0).units.find((u) => u.id === 0)!.pos;
    const order: Order = type === 'MoveUnit' ? { type, unit: 0, to } : { type, unit: 0 };
    expect(client.validate(0, [order])).toEqual([]);
    client.endTurn({ 0: [order] });
    expect(client.view(0).units.find((u) => u.id === 0)?.standing_order).toBeUndefined();
  });
}
