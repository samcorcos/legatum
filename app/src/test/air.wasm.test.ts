// Aircraft on the Rust engine: a Bomber in a hangar strikes through the real client API, the forecast and
// validation see it, and the order helpers and descriptions read it. The demo engine has no hangars.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { clearAirOrder, describeOrder, pendingAirOrder, setAirOrder } from '../orders';
import { coreFixture } from './coreFixture';
import { siegeFixture } from './siegeFixture';
import type { Order } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let core: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  core = new WasmClient(mod.Game);
});

function withBomber() {
  const s = siegeFixture({ fresh: true });
  // The save holds u64 bitsets that JSON.parse would round, so the hangar is added as text.
  const aircraft = JSON.stringify([{ id: 900, kind: 'Bomber', city: s.own, hp: 100 }, { id: 901, kind: 'Fighter', city: s.own, hp: 100 }]);
  const save = coreFixture(s.fixture).replace(/"next_unit_id":\d+/, '"next_unit_id":901');
  core.load(save.replace(/^\{/, `{"aircraft":${aircraft},`));
  return s;
}

it.skipIf(!available)('a hangar shows in the view, forecasts a strike and strikes through an order', () => {
  const s = withBomber();
  const view = core.view(0);
  expect(view.aircraft).toEqual([
    { id: 900, owner: 0, kind: 'Bomber', city: s.own, hp: 100, level: 0, xp: 0, xp_next: 16, unsupplied: null },
    { id: 901, owner: 0, kind: 'Fighter', city: s.own, hp: 100, level: 0, xp: 0, xp_next: 16, unsupplied: null },
  ]);
  const target = view.units.find((u) => u.owner !== view.me && u.military)!;
  const forecast = core.forecast(0, 900, target.pos, 'Ranged');
  expect(forecast).not.toBeNull();
  expect(forecast!.attacker_strength.total).toBe(114); // Bomber 64 on the era curve (Modern)
  expect(forecast!.expected_damage_to_attacker).toBe(0);
  // A Fighter strikes too: its own strength, no city bonus, within its shorter range.
  const fighter = core.forecast(0, 901, target.pos, 'Ranged');
  expect(fighter?.attacker_strength.total).toBe(103); // Fighter 58 at the Modern era curve
  expect(fighter?.expected_damage_to_attacker).toBe(0);
  expect(core.validate(0, [{ type: 'AirStrike', plane: 901, target: target.pos }])).toEqual([]);
  const strike: Order = { type: 'AirStrike', plane: 900, target: target.pos };
  expect(core.validate(0, [strike])).toEqual([]);
  expect(core.validate(0, [strike, { ...strike }]).length, 'one order per aircraft').toBe(1);
  const result = core.endTurn({ 0: [strike] });
  expect(result.events.some((e) => e.type === 'Battle' && e.attacker === 900 && e.ranged)).toBe(true);
  const after = core.view(0);
  expect(after.units.every((u) => u.kind !== 'Bomber' && u.kind !== 'Fighter'), 'aircraft never stand on the map').toBe(true);
  expect(after.aircraft!.find((a) => a.id === 900)!.hp).toBe(100);
});

it.skipIf(!available)('the order helpers keep one air order per plane and describe it in words', () => {
  const s = withBomber();
  const view = core.view(0);
  const target = view.units.find((u) => u.owner !== view.me && u.military)!;
  const strike: Order = { type: 'AirStrike', plane: 900, target: target.pos };
  let orders = setAirOrder([], strike);
  orders = setAirOrder(orders, { type: 'Rebase', plane: 900, to: s.own });
  expect(orders).toEqual([{ type: 'Rebase', plane: 900, to: s.own }]);
  expect(pendingAirOrder(orders, 900)).toBeDefined();
  expect(clearAirOrder(orders, 900)).toEqual([]);
  expect(describeOrder(strike, view)).toMatch(/^Bomber: air strike on /);
  expect(describeOrder({ type: 'Rebase', plane: 901, to: s.own }, view)).toMatch(/^Fighter: rebase to Waypost Seven$/);
});
