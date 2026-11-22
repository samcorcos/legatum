import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { explorationCoreSave, explorationFixture } from '../test/explorationFixture';
import { FixtureClient } from './fixtureEngine';
import { WasmClient } from '../wasmClient';
import { exploreStep } from './fixtureEngine';
import type { Order, PlayerView } from '../types';
import { distance, fromOffset } from '../hex';
import { nextUnitNeedingOrders, unitsNeedingOrders } from '../orders';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let core: WasmClient;
let plan: (v: PlayerView, id: number) => Order;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  core = new WasmClient(mod.Game);
  plan = (v, id) => JSON.parse(mod.explore_step(JSON.stringify(v), id));
});
it.skipIf(!available)('WASM agree on fog-safe decisions, moves, ruin finds, and stopping', () => {
  const fixture = explorationFixture();
  // Both are human to keep the comparison about exploration only.
  fixture.players[1].isHuman = true; fixture.setup.players[1].is_human = true;
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(fixture)); core.load(explorationCoreSave(fixture));
  let stopped = false, found = false;
  for (let turn = 0; turn < 12; turn++) {
    const view = core.view(0); const order = plan(view, 0);
    expect(exploreStep(view, 0)).toEqual(order);
    const a = fixtureClient.endTurn({ 0: [order] }); const b = core.endTurn({ 0: [order] });
    const notable = (events: typeof a.events) => events.filter((e) => ['UnitMoved', 'RuinExplored', 'AutomationStopped'].includes(e.type));
    (expect(notable(a.events)).toMatchSnapshot(), void (notable(b.events)));
    const au = fixtureClient.view(0).units.find((u) => u.id === 0)!; const bu = core.view(0).units.find((u) => u.id === 0)!;
    expect([au.pos, au.automation, au.fortify, au.needs_orders]).toEqual([bu.pos, bu.automation, bu.fortify, bu.needs_orders]);
    found ||= b.events.some((e) => e.type === 'RuinExplored');
    if (b.events.some((e) => e.type === 'AutomationStopped')) { stopped = true; break; }
  }
  expect(stopped).toBe(true); expect(found).toBe(true);
});
it.skipIf(!available).each([100, 80, 59])('fixtureClient planner matches WASM for threats and damage (%i HP)', (hp) => {
  const s = explorationFixture(); s.units[0].hp = hp;
  s.units.push({ ...s.units[0], id: 9, owner: 1, kind: 'Archer', pos: { q: 3, r: 4 } });
  s.nextUnitId = 9;
  core.load(explorationCoreSave(s));
  const view = core.view(0);
  expect(exploreStep(view, 0)).toEqual(plan(view, 0));
});

it.skipIf(!available)('both planners retreat out of adjacent enemy range before stopping', () => {
  const s = explorationFixture(); s.units[0].hp = 59;
  s.terrain.fill('Plains');
  s.units.push({ ...s.units[0], id: 9, owner: 1, kind: 'Spearman', pos: fromOffset(4, 4) });
  s.nextUnitId = 9;
  core.load(explorationCoreSave(s));
  const view = core.view(0); const order = plan(view, 0);
  expect(exploreStep(view, 0)).toEqual(order);
  expect(order.type).toBe('SetAutomation');
  if (order.type !== 'SetAutomation') throw new Error('expected automation order');
  expect(order.stop_reason).toBe('Damaged');
  expect(order.path!.length).toBeGreaterThanOrEqual(2);
  expect(distance(order.path!.at(-1)!, fromOffset(4, 4))).toBeGreaterThan(view.units.find((u) => u.id === 9)!.max_moves);
});

it.skipIf(!available)('automated rest retains normal fortification and healing in both engines', () => {
  const s = explorationFixture();
  s.players[1].isHuman = true; s.setup.players[1].is_human = true;
  s.units[0].hp = 70; s.units[0].fortify = 3; s.units[0].automation = 'Explore';
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  core.load(explorationCoreSave(s));
  const order: Order = { type: 'SetAutomation', unit: 0, automation: 'Explore' };
  fixtureClient.endTurn({ 0: [order] }); core.endTurn({ 0: [order] });
  const a = fixtureClient.view(0).units.find((u) => u.id === 0)!;
  const b = core.view(0).units.find((u) => u.id === 0)!;
  (expect([a.hp, a.fortify, a.automation]).toMatchSnapshot(), void ([b.hp, b.fortify, b.automation]));
  expect(a.fortify).toBe(6);
});

it.skipIf(!available).each(['complete', 'manual', 'blocked'] as const)('%s auto-explore stop preserves fortification and requests orders in both engines', (stop) => {
  const s = explorationFixture();
  s.players[1].isHuman = true; s.setup.players[1].is_human = true;
  const scout = s.units[0];
  scout.hp = 70; scout.fortify = 3; scout.automation = 'Explore';
  const target = fromOffset(4, 4);
  if (stop === 'blocked') s.units.push({ ...scout, id: 3, kind: 'Spearman', pos: target, automation: null });
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s)); core.load(explorationCoreSave(s));
  const order: Order = stop === 'blocked'
    ? { type: 'SetAutomation', unit: 0, automation: 'Explore', path: [target] }
    : { type: 'SetAutomation', unit: 0, automation: null, path: [], ...(stop === 'complete' ? { stop_reason: 'Complete' as const } : {}) };
  fixtureClient.endTurn({ 0: [order] }); core.endTurn({ 0: [order] });
  const a = fixtureClient.view(0).units.find((u) => u.id === 0)!;
  const b = core.view(0).units.find((u) => u.id === 0)!;
  (expect([a.pos, a.hp, a.fortify, a.automation, a.needs_orders]).toMatchSnapshot(), void ([b.pos, b.hp, b.fortify, b.automation, b.needs_orders]));
  expect(a).toMatchObject({ pos: scout.pos, hp: 80, fortify: 6, needs_orders: true });

  for (const client of [fixtureClient, core]) {
    client.load(client.save());
    client.endTurn({});
    const view = client.view(0);
    expect(view.units.find((u) => u.id === 0)).toMatchObject({ fortify: 6, needs_orders: true });
    expect(unitsNeedingOrders(view, []).map((u) => u.id)).toContain(0);
    expect(nextUnitNeedingOrders(view, [], null)?.id).toBe(0);
    const hold: Order = { type: 'SleepUnit', unit: 0 };
    expect(unitsNeedingOrders(view, [hold]).map((u) => u.id)).not.toContain(0);
    expect(unitsNeedingOrders(view, [{ type: 'SkipUnit', unit: 0 }]).map((u) => u.id)).not.toContain(0);
    client.endTurn({ 0: [hold] });
    const held = client.view(0);
    expect(held.units.find((u) => u.id === 0)).toMatchObject({ fortify: 6 });
    expect(held.units.find((u) => u.id === 0)!.needs_orders).toBeFalsy();
    expect(unitsNeedingOrders(held, []).map((u) => u.id)).not.toContain(0);
  }
});
