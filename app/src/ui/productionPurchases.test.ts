import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import type { GameClient } from '../client';
import { itemKey, sameItem, TECHS } from '../data';
import { fromOffset, neighbors } from '../hex';
import { FixtureClient } from '../test/fixtureEngine';
import { idxOf } from '../test/fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from '../test/fixtureEngine';
import type { MCity } from '../test/fixtureState';
import { RemoteClient } from '../remote';
import { coreFixture } from '../test/coreFixture';
import { FakeSocket } from '../test/fakeSocket';
import type { Order, OrderError, PlayerView, ProductionItem, UnitKind } from '../types';
import { WasmClient } from '../wasmClient';
import { pendingSpend, purchasePreviews, tilePurchasePreview } from './productionPurchases';

const soldier: ProductionItem = { kind: 'Unit', unit: 'Spearman' };
const settler: ProductionItem = { kind: 'Unit', unit: 'Settler' };
const relay: ProductionItem = { kind: 'Wonder', building: 'GreatLibrary', tile: fromOffset(5, 4) };
const spawnBlocker = 'No free spawn tile in or beside the city; move a blocking unit first';
const remote = () => new RemoteClient(0, 'P0', 'ws://unused/ws');

function previews(client: GameClient, view: PlayerView, orders: Order[], errors: OrderError[] = []) {
  return purchasePreviews(client, view.cities.find((c) => c.id === 1)!, view, orders, errors).options;
}

function fixture() {
  const setup = makeStandardSetup('700001', 'Athens', 'India', 'P0', 'P1');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 3).map((p) => ({ ...p, is_human: true }));
  const initial = createState(setup);
  initial.units = [];
  initial.nextUnitId = 0;
  initial.terrain.fill('Grassland');
  initial.ruin.fill(0);
  initial.owner.fill(-1);
  initial.tileCity.fill(-1);
  for (const p of initial.players) {
    p.gold = 1000;
    p.techs = [...TECHS];
    p.explored.fill(1);
  }
  initial.cities = [4, 13].map((col, i): MCity => ({
    id: i + 1, owner: 0, founder: 0, name: `City ${i + 1}`, pos: fromOffset(col, 4),
    population: 8, food: 0, prod: 0, producing: null, bought: null,
    buildings: [], hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: i === 0, foundedTurn: 1, resistance: 0,
  }));
  initial.nextCityId = 3;
  for (const c of initial.cities) {
    initial.owner[idxOf(initial, c.pos)] = c.owner;
    initial.tileCity[idxOf(initial, c.pos)] = c.id;
  }
  const client = new FixtureClient();
  client.load(JSON.stringify(initial));
  const s = client.debugState();
  const city = s.cities[0];
  for (const h of neighbors(city.pos)) s.terrain[idxOf(s, h)] = 'Mountains';
  const unit = spawnUnit(s, 0, 'Spearman', city.pos, false);
  const disband: Order = { type: 'DisbandUnit', unit: unit.id };
  const buy: Order = { type: 'BuyItem', city: city.id, item: soldier };
  return { client, s, city, unit, disband, buy };
}

function foreignSpawnFixture(relation: 'Peace' | 'War') {
  const f = fixture();
  f.unit.pos = neighbors(f.city.pos)[0];
  const tile = idxOf(f.s, f.unit.pos);
  f.s.terrain[tile] = 'Grassland';
  f.s.owner[tile] = 2;
  f.s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!.state = relation;
  spawnUnit(f.s, 0, 'Spearman', f.city.pos, false);
  return f;
}

describe('purchase previews', () => {
  it('reserves a queued Siege upgrade before a later purchase even when Siege appears last', () => {
    const { client, unit } = fixture();
    const view = client.view(0);
    view.units.find((u) => u.id === unit.id)!.siege_offer = { price: { total: 75, parts: [['Siege', 75]] }, blocker: null };
    const order: Order = { type: 'UpgradeSiege', unit: unit.id };
    const forecast = pendingSpend(view, [order], []);
    expect(forecast.gold).toBe(view.economy.gold - 75);
    expect(forecast.before({ type: 'BuyItem', city: 1, item: soldier }).gold).toBe(view.economy.gold - 75);
    expect(forecast.supply.get('Iron')).toBe(1);
  });

  it('reserves an aircraft upgrade after city purchases, in aircraft id order', () => {
    const { client } = fixture();
    const view = client.view(0);
    view.aircraft = [{ id: 900, owner: 0, kind: 'Fighter', city: 1, hp: 100, upgrade_cost: 200, upgrade_options: [{ via: 'StealthFighter', to: 'StealthFighter', cost: 200 }] }, { id: 901, owner: 0, kind: 'Bomber', city: 1, hp: 100, upgrade_cost: 150, upgrade_options: [{ via: 'StealthBomber', to: 'StealthBomber', cost: 150 }] }];
    const orders: Order[] = [{ type: 'UpgradeAircraft', plane: 901 }, { type: 'UpgradeAircraft', plane: 900 }];
    const forecast = pendingSpend(view, orders, []);
    expect(forecast.before(orders[0] as Extract<Order, { type: 'UpgradeAircraft' }>, 0).gold).toBe(view.economy.gold - 200);
    expect(forecast.before({ type: 'BuyItem', city: 1, item: soldier }).gold).toBe(view.economy.gold);
    expect(forecast.gold).toBe(view.economy.gold - 350);
    expect(forecast.supply.get('Silicon')).toBe(2);
  });

  it('blocks a new city buy when the queued aircraft upgrade needs the remaining Gold', () => {
    const { client, disband } = fixture();
    const view = client.view(0);
    const option = view.cities[0].detail!.options.find((o) => sameItem(o.item, soldier))!;
    view.aircraft = [{ id: 900, owner: 0, kind: 'Fighter', city: 1, hp: 100, upgrade_cost: 200, upgrade_options: [{ via: 'StealthFighter', to: 'StealthFighter', cost: 200 }] }];
    view.economy.gold = option.buy_cost! + 199;
    const orders: Order[] = [{ type: 'UpgradeAircraft', plane: 900 }, disband];
    expect(previews(remote(), view, orders).get(itemKey(soldier))?.reason).toBe(`Not enough Gold: costs ${option.buy_cost} Gold; you have ${option.buy_cost! - 1}`);
  });

  it('walks a mixed queue by resolution phase, using each core view price and supply need', () => {
    const { client, unit } = fixture();
    const view = client.view(0);
    view.economy.gold = 1000;
    view.units.find((u) => u.id === unit.id)!.siege_offer = { price: { total: 70, parts: [['Siege', 70]] }, blocker: null };
    view.units.push({ ...view.units[0], id: 99, upgrade_cost: 120, upgrade_breakdown: { total: 120, parts: [['Upgrade', 120]] }, upgrade_need: 'Iron' });
    view.aircraft = [{ id: 900, owner: 0, kind: 'Fighter', city: 1, hp: 100, upgrade_cost: 200, upgrade_options: [{ via: 'StealthFighter', to: 'StealthFighter', cost: 200 }] }];
    view.company_offers = [{ seller: 2, unit: 'Spearman', price: 80 }];
    const buyCost = view.cities[1].detail!.options.find((o) => sameItem(o.item, soldier))!.buy_cost!;
    const orders: Order[] = [
      { type: 'UpgradeAircraft', plane: 900 },
      { type: 'HireCompany', city: 1, seller: 2, unit: 'Spearman' },
      { type: 'BuyItem', city: 2, item: soldier },
      { type: 'UpgradeSiege', unit: unit.id },
      { type: 'UpgradeUnit', unit: 99 },
    ];
    const forecast = pendingSpend(view, orders, []);
    expect(forecast.before(orders[1] as Extract<Order, { type: 'HireCompany' }>, 1).gold).toBe(810);
    expect(forecast.before(orders[0] as Extract<Order, { type: 'UpgradeAircraft' }>, 0).gold).toBe(730 - buyCost);
    expect(forecast.gold).toBe(530 - buyCost);
    expect(forecast.available({ type: 'UpgradeUnit', unit: 555 }).gold).toBe(530 - buyCost);
    expect(forecast.available(orders[0] as Extract<Order, { type: 'UpgradeAircraft' }>, 0).gold).toBe(730 - buyCost);
    expect(forecast.supply.get('Iron')).toBe(2);
    expect(forecast.supply.get('Silicon')).toBe(1);
  });
  it.each(['Peace', 'War'] as const)('checks borders before a disband frees the only foreign spawn tile at %s', (relation) => {
    const { client, disband, buy } = foreignSpawnFixture(relation);
    const blocked = relation === 'Peace';
    expect(client.validate(0, [disband, buy])).toEqual(blocked ? [{ index: 1, message: spawnBlocker }] : []);
    const view = client.view(0);
    for (const orders of [[disband], [disband, buy], [buy, disband]]) {
      expect(previews(remote(), view, orders).get(itemKey(soldier))?.reason).toBe(blocked ? spawnBlocker : null);
    }
  });

  it('allows a remote purchase when a valid disband frees the only spawn tile', () => {
    const { client, city, disband, buy } = fixture();
    const view = client.view(0);
    const cityView = view.cities.find((c) => c.id === city.id)!;
    const remoteClient = remote();
    expect(client.validate(0, [buy])).toEqual([{ index: 0, message: expect.stringContaining('No free spawn tile') }]);
    expect(client.validate(0, [disband, buy])).toEqual([]);
    expect(client.validate(0, [buy, disband])).toEqual([]);
    const preview = purchasePreviews(remoteClient, cityView, view, [disband], client.validate(0, [disband]));
    expect(preview.options.get(itemKey(soldier))?.reason).toBeNull();
    for (const orders of [[disband, buy], [buy, disband]]) {
      expect(previews(remoteClient, view, orders).get(itemKey(soldier))).toMatchObject({ reason: null, selected: true });
    }
  });

  it('restores the blocker when the disband is cancelled, including for an already selected purchase', () => {
    const { client, disband, buy } = fixture();
    const view = client.view(0);
    const before = JSON.stringify(view);
    expect(previews(remote(), view, []).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    expect(previews(remote(), view, [disband]).get(itemKey(soldier))?.reason).toBeNull();
    expect(previews(remote(), view, []).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    expect(previews(remote(), view, [buy]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    expect(JSON.stringify(view)).toBe(before);
  });

  it.each([1, 2])('does not disband a visible foreign unit owned by player %i without cached errors', (owner) => {
    const { client, unit, disband, buy } = fixture();
    unit.owner = owner;
    const view = client.view(0);
    expect(view.units.some((u) => u.id === unit.id)).toBe(true);
    expect(client.validate(0, [disband, buy]).map((e) => e.index)).toEqual([0, 1]);
    expect(previews(remote(), view, [disband]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
  });

  it('does not accept a missing unit or a known rejected disband', () => {
    const { client, disband } = fixture();
    const view = client.view(0);
    expect(previews(remote(), view, [{ type: 'DisbandUnit', unit: 9999 }]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    expect(previews(remote(), view, [disband], [{ index: 0, message: 'Disband rejected' }]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
  });

  it.each(['SleepUnit', 'FoundCity', 'MoveUnit', 'MoveUnitPath', 'SetAutomation', 'RangedAttack', 'UpgradeUnit', 'SkipUnit'] as const)(
    'does not free a unit already given %s, even before validation arrives', (type) => {
      const { client, unit, disband, buy } = fixture();
      const first = { type, unit: unit.id, to: { q: -100, r: -100 }, path: [], automation: 'Explore', target: unit.pos } as unknown as Order;
      const orders = [first, disband];
      expect(client.validate(0, [...orders, buy]).some((e) => e.index === 1)).toBe(true);
      expect(previews(remote(), client.view(0), orders).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
      // An invalid first action still reserves the unit, per authoritative validation.
      expect(previews(remote(), client.view(0), orders, client.validate(0, orders)).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    },
  );

  it('keeps the first valid disband when a later duplicate action is invalid', () => {
    const { client, disband, buy } = fixture();
    const orders = [disband, disband, buy];
    const errors = client.validate(0, orders);
    expect(errors.map((e) => e.index)).toEqual([1]);
    expect(previews(remote(), client.view(0), orders, errors).get(itemKey(soldier))?.reason).toBeNull();
  });

  it('does not free space for an eliminated empire', () => {
    const { client, s, disband, buy } = fixture();
    s.players[0].alive = false;
    expect(client.validate(0, [disband, buy])).toHaveLength(2);
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
  });

  it.each([
    ['Spearman', 'Scout', 0, true],
    ['Settler', 'Settler', 0, true],
    ['Spearman', 'Settler', 0, false],
    ['Settler', 'Scout', 1, false],
    ['Spearman', 'Settler', 2, true],
    ['Settler', 'Scout', 2, true],
  ] as const)('checks remaining occupancy when buying %s beside %s owned by %i', (kind, remaining, owner, blocked) => {
    const { client, s, city, unit, disband } = fixture();
    unit.kind = kind;
    spawnUnit(s, owner, remaining, city.pos, false);
    const item: ProductionItem = { kind: 'Unit', unit: kind };
    const orders: Order[] = [disband, { type: 'BuyItem', city: city.id, item }];
    expect(client.validate(0, orders).some((e) => e.index === 1)).toBe(blocked);
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(item))?.reason).toBe(blocked ? spawnBlocker : null);
  });

  it('can free an adjacent tile while the city center stays occupied', () => {
    const { client, s, city, unit, disband, buy } = fixture();
    unit.pos = neighbors(city.pos)[0];
    s.terrain[idxOf(s, unit.pos)] = 'Grassland';
    spawnUnit(s, 0, 'Spearman', city.pos, false);
    expect(client.validate(0, [buy])).toHaveLength(1);
    expect(client.validate(0, [disband, buy])).toEqual([]);
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(soldier))?.reason).toBeNull();
  });

  it.each(['Mountains', 'Coast', 'Ocean'] as const)('cannot create a spawn tile on %s', (terrain) => {
    const { client, s, city, disband, buy } = fixture();
    s.terrain[idxOf(s, city.pos)] = terrain;
    expect(client.validate(0, [disband, buy])).toEqual([{ index: 1, message: spawnBlocker }]);
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
  });

  it('cannot free a tile containing a hostile city', () => {
    const { client, s, city, unit, disband, buy } = fixture();
    unit.pos = neighbors(city.pos)[0];
    s.terrain[idxOf(s, unit.pos)] = 'Grassland';
    s.cities[1].pos = unit.pos;
    s.cities[1].owner = 2;
    spawnUnit(s, 0, 'Spearman', city.pos, false);
    expect(client.validate(0, [disband, buy])).toEqual([{ index: 1, message: spawnBlocker }]);
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
  });

  it('does not infer free space from fog or an unrelated disband', () => {
    const { client, s, city, disband } = fixture();
    const view = client.view(0);
    view.tiles[idxOf(s, city.pos)].visible = false;
    expect(previews(remote(), view, [disband]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    view.tiles[idxOf(s, city.pos)].visible = true;
    view.tiles[idxOf(s, city.pos)].terrain = null;
    expect(previews(remote(), view, [disband]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    const other = spawnUnit(s, 0, 'Spearman', s.cities[1].pos, false);
    expect(previews(remote(), client.view(0), [{ type: 'DisbandUnit', unit: other.id }]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
  });

  it('still requires gold after a disband removes the spawn blocker, including a selected purchase with no cached errors', () => {
    const { client, s, disband, buy } = fixture();
    s.players[0].gold = 0;
    expect(client.validate(0, [disband, buy])).toEqual([{ index: 1, message: expect.stringContaining('Not enough Gold:') }]);
    for (const orders of [[disband], [disband, buy]]) {
      expect(previews(remote(), client.view(0), orders).get(itemKey(soldier))?.reason).toMatch(/^Not enough Gold:/);
    }
  });

  it('retains population, resistance and already-purchased restrictions', () => {
    const { client, city, s, disband } = fixture();
    city.population = 1;
    // A Settler at population 1 is not offered at all: its greyed-out card says why instead.
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(settler))).toBeUndefined();
    expect(client.view(0).cities.find((c) => c.id === city.id)!.detail!.unavailable.map((u) => u.reason)).toContain('Settler needs population 2 (currently 1)');
    city.resistance = 1;
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(soldier))?.reason).toContain('resisting occupation');
    city.resistance = 0;
    city.lastPurchaseTurn = s.turn;
    expect(previews(remote(), client.view(0), [disband]).get(itemKey(soldier))?.reason).toContain('already bought something this turn');
  });

  it('keeps prerequisites in the offered options', () => {
    const { client, s, disband } = fixture();
    s.players[0].techs = [];
    const trooper: ProductionItem = { kind: 'Unit', unit: 'Pikeman' };
    expect(previews(remote(), client.view(0), [disband]).has(itemKey(trooper))).toBe(false);
    expect(client.validate(0, [disband, { type: 'BuyItem', city: 1, item: trooper }])).toHaveLength(1);
    s.players[0].techs = [...TECHS];
  });

  it('does not make Great Works purchasable', () => {
    const { client, s, city, disband } = fixture();
    city.buildings = ['Library'];
    const site = idxOf(s, relay.kind === 'Wonder' ? relay.tile : city.pos);
    s.terrain[site] = 'Grassland'; s.owner[site] = city.owner; s.tileCity[site] = city.id;
    if (s.wonder) s.wonder[site] = null;
    const option = previews(remote(), client.view(0), [disband]).get(itemKey(relay));
    expect(option).toMatchObject({ cost: null, reason: 'Great Library cannot be bought' });
    expect(client.validate(0, [disband, { type: 'BuyItem', city: city.id, item: relay }])).toHaveLength(1);
  });

  it('does not replace another purchase in the same city', () => {
    const { client, disband, buy } = fixture();
    const orders = [disband, buy];
    const options = previews(remote(), client.view(0), orders);
    expect(options.get(itemKey(soldier))?.reason).toBeNull();
    expect(options.get(itemKey(settler))?.reason).toBe('One purchase per city per turn; cancel the current purchase first');
    expect(client.validate(0, [...orders, { type: 'BuyItem', city: 1, item: settler }])).toHaveLength(1);
  });

  it('preserves every unrelated snapshot blocker and selected server error', () => {
    const { client, disband, buy } = fixture();
    const view = client.view(0);
    const option = view.cities[0].detail!.options.find((o) => itemKey(o.item) === itemKey(soldier))!;
    for (const reason of ['Requires Feudalism', 'A new server restriction']) {
      option.buy_blocker = reason;
      expect(previews(remote(), view, [disband]).get(itemKey(soldier))?.reason).toBe(reason);
    }
    option.buy_blocker = spawnBlocker;
    for (const message of [spawnBlocker, 'Not enough Gold: server rejected purchase', 'One purchase per city per turn']) {
      expect(previews(remote(), view, [disband, buy], [{ index: 1, message }]).get(itemKey(soldier))?.reason).toBe(message);
    }
  });

  it('keeps local previews on synchronous validation, regardless of stale supplied errors', () => {
    const { client, disband, buy } = fixture();
    const view = client.view(0);
    expect(previews(client, view, []).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    expect(previews(client, view, [disband], [{ index: 0, message: 'stale' }]).get(itemKey(soldier))?.reason).toBeNull();
    expect(previews(client, view, [buy]).get(itemKey(soldier))?.reason).toBe(spawnBlocker);
    expect(previews(client, view, [buy, disband]).get(itemKey(soldier))?.reason).toBeNull();
  });

  it('does not send orders, query remote validation, or mutate the asynchronous validation cache during previews', () => {
    const { client, disband, buy } = fixture();
    const view = client.view(0);
    const socket = new FakeSocket('ws://unused/ws');
    const remoteClient = new RemoteClient(0, 'P0', socket.url, () => socket);
    remoteClient.token = 'fixture-token';
    remoteClient.connect();
    socket.open();
    socket.push({ type: 'welcome', seat: 0, game_id: 'preview-fixture', orders: [] });
    socket.push({ type: 'view', view });
    try {
      const orders = [disband, buy];
      remoteClient.syncOrders(orders);
      const errors = [{ index: 1, message: 'Purchase rejected by server' }];
      socket.push({ type: 'validation', errors });
      const sent = [...socket.sent];
      const validate = vi.spyOn(remoteClient, 'validate');
      expect(previews(remoteClient, view, orders, errors).get(itemKey(soldier))?.reason).toBe(errors[0].message);
      previews(remoteClient, view, [disband]);
      previews(remoteClient, view, []);
      expect(validate).not.toHaveBeenCalled();
      expect(socket.sent).toEqual(sent);
      expect(remoteClient.pendingOrders()).toEqual(orders);
      expect(remoteClient.validate(0, orders)).toEqual(errors);
    } finally {
      remoteClient.disconnect(false);
    }
  });
});

const wasmAvailable = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
describe.skipIf(!wasmAvailable)('purchase preview contract with the authoritative Rust engine', () => {
  let makeCore: () => WasmClient;
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it.each(['Peace', 'War'] as const)('matches Rust validation and resolution when disbanding the only foreign spawn tile at %s', (relation) => {
    const { client, s, unit, disband, buy } = foreignSpawnFixture(relation);
    const core = makeCore();
    core.load(coreFixture(s));
    const blocked = relation === 'Peace';
    for (const engine of [core, client]) {
      const orders = [disband, buy];
      expect(engine.validate(0, orders)).toEqual(blocked ? [{ index: 1, message: spawnBlocker }] : []);
      const view = engine.view(0);
      expect(previews(remote(), view, [disband]).get(itemKey(soldier))?.reason).toBe(blocked ? spawnBlocker : null);
      expect(previews(remote(), view, orders).get(itemKey(soldier))?.reason).toBe(blocked ? spawnBlocker : null);
      const { events } = engine.endTurn({ 0: orders });
      expect(events).toContainEqual(expect.objectContaining({ type: 'UnitDisbanded', unit: unit.id }));
      expect(events.some((e) => e.type === 'ItemBought')).toBe(!blocked);
      expect(events.some((e) => e.type === 'OrderFailed')).toBe(blocked);
    }
  });

  it.each(['Spearman', 'Settler'] as UnitKind[])('validates and resolves disband plus %s purchase in either order', (kind) => {
    for (const disbandFirst of [true, false]) {
      const { client, s, unit, disband } = fixture();
      unit.kind = kind;
      const core = makeCore();
      core.load(coreFixture(s));
      const item: ProductionItem = { kind: 'Unit', unit: kind };
      const buy: Order = { type: 'BuyItem', city: 1, item };
      const orders = disbandFirst ? [disband, buy] : [buy, disband];
      for (const engine of [core, client]) {
        expect(engine.validate(0, [buy])).toEqual([{ index: 0, message: spawnBlocker }]);
        expect(engine.validate(0, orders)).toEqual([]);
        const view = engine.view(0);
        expect(previews(remote(), view, [disband]).get(itemKey(item))?.reason).toBeNull();
        expect(previews(engine, view, orders).get(itemKey(item))?.reason).toBeNull();
        const { events } = engine.endTurn({ 0: orders });
        expect(events.filter((e) => e.type === 'OrderFailed')).toEqual([]);
        expect(events).toContainEqual(expect.objectContaining({ type: 'ItemBought', city: 1, item }));
        expect(events).toContainEqual(expect.objectContaining({ type: 'UnitDisbanded', unit: unit.id }));
        expect(engine.view(0).units.some((u) => u.id !== unit.id && u.kind === kind && u.owner === 0)).toBe(true);
      }
    }
  });
});

describe('tile purchases', () => {
  it('price tags come from the city view and read the treasury less other queued spending', () => {
    const { client } = fixture();
    const view = client.view(0);
    const city = view.cities.find((c) => c.id === 1)!;
    const tiles = city.detail!.buyable_tiles;
    expect(tiles.length).toBeGreaterThan(0);
    expect(tiles.every((b, i) => i === 0 || tiles[i - 1].ring <= b.ring)).toBe(true);
    const cheapest = tiles[0];
    view.economy.gold = cheapest.price;
    // A queued item purchase elsewhere takes the Gold: the tile is out of reach.
    const buyCost = view.cities[1].detail!.options.find((o) => sameItem(o.item, soldier))!.buy_cost!;
    view.economy.gold = cheapest.price + buyCost - 1;
    const taken = tilePurchasePreview(city, view, [{ type: 'BuyItem', city: 2, item: soldier }], []);
    expect(taken.tags[0]).toMatchObject({ affordable: false, reason: `Need 1 more Gold` });
    expect(taken.cheapest).toBe(cheapest.price);
    // An item purchase queued in this city holds its Gold and blocks the tile control: it is never silently replaced.
    view.economy.gold = buyCost;
    const blocked = tilePurchasePreview(city, view, [{ type: 'BuyItem', city: 1, item: soldier }], []);
    expect(blocked.other).toMatchObject({ type: 'BuyItem' });
    expect(blocked.tags[0].affordable).toBe(false);
  });

  it('the queued tile reserves its price, shows the cost chip inputs and blocks every item row', () => {
    const { client } = fixture();
    const view = client.view(0);
    const city = view.cities.find((c) => c.id === 1)!;
    const tile = city.detail!.buyable_tiles[0];
    view.economy.gold = 500;
    const orders: Order[] = [{ type: 'BuyTile', city: 1, tile: tile.tile }];
    const preview = tilePurchasePreview(city, view, orders, []);
    expect(preview).toMatchObject({ price: tile.price, goldBefore: 500, funds: 500, error: null });
    expect(pendingSpend(view, orders, []).gold).toBe(500 - tile.price);
    expect(preview.tags[0]).toMatchObject({ queued: true, affordable: true });
    const reasons = [...previews(remote(), view, orders).values()].map((p) => p.reason);
    expect(reasons.every((r) => r === 'One purchase per city per turn; cancel the current purchase first')).toBe(true);
    // A rejected order reserves nothing and shows its message.
    const failed = tilePurchasePreview(city, view, orders, [{ index: 0, message: 'Not enough Gold' }]);
    expect(failed.error).toBe('Not enough Gold');
  });

  it('a city that cannot buy shows the core reason and no price tags', () => {
    const { client } = fixture();
    const view = client.view(0);
    const city = view.cities.find((c) => c.id === 1)!;
    city.detail!.buy_tile_blocker = 'Occupied: City 1 builds and buys nothing until Peace with India';
    const preview = tilePurchasePreview(city, view, [], []);
    expect(preview.tags).toEqual([]);
    expect(preview.blocker).toContain('Occupied');
  });
});
