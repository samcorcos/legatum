import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { createState, idxOf, makeStandardSetup, refreshVision, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';
import { fromOffset, neighbors } from '../hex';
import { unitsNeedingOrders } from '../orders';
import { pendingDecisions } from '../turnFlow';
import { releasedGarrison } from '../releasedGarrison';
import { contextFromViews, releasedGarrisonLines } from '../format';
import type { Order } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

const middle = fromOffset(5, 5);

/** Two empires at War; seat 1 owns an undefended city and a Spearman of seat 0 stands beside it, ordered in. */
function captureFixture(): MState {
  const setup = makeStandardSetup('7', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.width = 20; setup.height = 12;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, faction: 'Switzerland', team: i, is_human: true }));
  const s = createState(setup);
  for (const relation of s.diplomacy!.relations) relation.state = 'War';
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
  for (const p of s.players) { p.gold = 10000; p.explored.fill(0); spawnUnit(s, p.id, 'Settler', fromOffset(2 + 6 * p.id, 10), false); }
  const city: MCity = { id: 1, owner: 1, founder: 1, name: 'Redoubt', pos: middle, population: 3, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: 0, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 };
  // Seat 1's own capital stands far away, so losing Redoubt does not end the game by capitals.
  const home = fromOffset(16, 3);
  const capital: MCity = { ...city, id: 0, name: 'Home', pos: home, hp: 200, originalCapital: true };
  s.cities.push(capital, { ...city, isCapital: false }); s.nextCityId = 2;
  s.owner[idxOf(s, home)] = 1; s.tileCity[idxOf(s, home)] = capital.id;
  s.owner[idxOf(s, middle)] = 1; s.tileCity[idxOf(s, middle)] = city.id;
  const spearman = spawnUnit(s, 0, 'Spearman', neighbors(middle)[0], false); spearman.goto = middle;
  refreshVision(s);
  return s;
}

it.skipIf(!available)('wasm: Peace releases a captured city\'s garrison and it asks for orders once', () => {
  const s = captureFixture();
  const client = makeCore(); client.load(coreFixture(s));
  client.endTurn({});
  const held = client.view(0).units.find((u) => u.kind === 'Spearman')!;
  const city = client.view(0).cities.find((c) => c.id === 1)!;
  expect(city.owner).toBe(0);
  // Held: locked in the city, fortified for him, and out of the queue.
  expect(held).toMatchObject({ occupation_lock: 1, standing_order: 'Sleep' });
  expect(unitsNeedingOrders(client.view(0), []).some((u) => u.id === held.id)).toBe(false);

  // Seat 0 proposes Peace, seat 1 accepts; the lock lifts at the end of the turn Peace takes hold.
  client.endTurn({ 0: [{ type: 'ProposeTreaty', to: 1, kind: 'Peace' }] });
  const proposal = client.view(1).diplomacy!.proposals[0].id;
  let before = client.view(0);
  let turn = client.endTurn({ 1: [{ type: 'RespondToProposal', proposal, accept: true }] });
  let after = client.view(0);
  for (let i = 0; i < 3 && after.units.find((u) => u.id === held.id)?.occupation_lock != null; i++) {
    before = after; turn = client.endTurn({}); after = client.view(0);
  }
  expect(before.units.find((u) => u.id === held.id)?.occupation_lock).toBe(1);
  expect(after.units.find((u) => u.id === held.id)?.occupation_lock ?? null).toBeNull();
  expect(after.units.find((u) => u.id === held.id)?.standing_order).toBe('Sleep');

  const released = releasedGarrison(before, after);
  expect(released).toEqual([{ unit: held.id, city: city.name }]);
  const report = { events: [turn.events], released: [released] };
  const local = { humans: [0], orders: [[]], errors: [[]], seat: 0, report };
  const asks = (orders: Order[] = []) => pendingDecisions(after, orders, [], local).filter((d) => d.kind === 'unit' && d.id === held.id);
  expect(asks()).toEqual([expect.objectContaining({ released: 'Redoubt', label: expect.stringMatching(/: Redoubt is secure, needs orders$/) })]);
  const lines = releasedGarrisonLines(before, after, contextFromViews(before, after));
  expect(lines.map((l) => l.text)).toEqual([expect.stringMatching(/: Redoubt is secure\. Needs orders\.$/)]);
  // Fortify or Skip this turn answers it; Skip also clears the standing order in the core.
  expect(asks([{ type: 'SleepUnit', unit: held.id }])).toEqual([]);
  const skip: Order[] = [{ type: 'SkipUnit', unit: held.id }];
  expect(asks(skip)).toEqual([]);
  client.endTurn({ 0: skip });
  const next = client.view(0);
  expect(next.units.find((u) => u.id === held.id)?.standing_order ?? null).toBeNull();
  // The turn after, the prompt is gone for good (no report entry), and a fortified-by-choice unit is never released.
  expect(releasedGarrison(after, next)).toEqual([]);
  expect(pendingDecisions(next, [], [], { ...local, report: { events: [[]], released: [releasedGarrison(after, next)] } })
    .some((d) => d.kind === 'unit' && !!d.released)).toBe(false);
});
