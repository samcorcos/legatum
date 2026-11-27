import { describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { hexAt, neighbors, distance, tileIndex } from '../hex';
import type { Event, Order, PlayerView, SupplyView } from '../types';
import { advisorSuggestion } from './advisorModel';
import { strategyTip, TIP_COOLDOWN, TIP_GAP } from './advisorTips';
import { ownerName } from '../barbarians';
import { itemKey, itemName } from '../data';

async function game(turn = 20): Promise<PlayerView> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  const v = client.view(0);
  v.turn = turn;
  v.economy = { ...v.economy, gold: 50 };
  v.supply = [];
  return v;
}
const mine = (v: PlayerView) => v.cities.find((c) => c.owner === v.me)!;
const rival = (v: PlayerView) => v.players.find((p) => p.id !== v.me && p.team !== v.team)!;
const iron = (over: Partial<SupplyView> = {}): SupplyView => ({ resource: 'Iron', revealed_by: 'BronzeWorking' as SupplyView['revealed_by'], revealed: true,
  supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, spare: 0, ...over });

describe('situational tips', () => {
  it('warns about a war declared on the team and an exposed city, using a military Hard build', async () => {
    const v = await game();
    const city = mine(v), r = rival(v);
    const relation = { ...v.diplomacy.relations[0], a: v.team, b: r.team, state: 'War' as const, declarers: [r.team] };
    v.diplomacy.relations = [relation];
    const events: Event[] = [{ type: 'WarDeclared', relation }];
    const unit = v.units.find((u) => u.owner === v.me && u.military)!;
    v.units.filter((u) => u.owner === v.me && u.military).forEach((u) => { u.pos = neighbors(city.pos)[0]; });
    const build = city.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit !== 'Scout')!.item;
    const plan: Order[] = [{ type: 'SetProduction', city: city.id, item: build }];
    const people = ownerName(v, r.id);
    expect(strategyTip(v, {}, false, null, [], null, {}, events, plan)).toEqual({
      id: `tip:war:${city.id}`, title: `${people} declared war`, why: `${people} has 0 military units near ${city.name}.`,
      steps: [{ instruction: `Click ${city.name}.`, target: { kind: 'tile', hex: city.pos } },
        { instruction: `Build ${itemName(build, v.players.find((p) => p.id === v.me)?.faction)}.`, target: { kind: 'control', name: 'build', value: itemKey(build) } }],
      current: 0, acknowledge: true, tip: true,
    });
    expect(strategyTip(v, {}, false, null, [], null, {}, events, [])?.steps[1].instruction).toBe('Build a military unit.');
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)?.id).not.toBe(`tip:war:${city.id}`);
    const enemy = { ...unit, id: unit.id + 9000, owner: r.id, kind: 'Spearman' as const, pos: v.tiles.map((_, i) => hexAt(i, v.width)).find((pos) => distance(pos, city.pos) === 4)! };
    v.units.push(enemy);
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)).toMatchObject({
      id: `tip:war:${city.id}`, title: `Defend ${city.name}`, why: `${people} has 1 military unit near ${city.name}.`,
    });
    unit.pos = city.pos;
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)?.id).not.toBe(`tip:war:${city.id}`);
    unit.pos = neighbors(city.pos)[0];
    r.army = 100;
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)?.id).toBe('tip:army');
    r.army = 0;
    expect(strategyTip(v, { [`war:${city.id}`]: v.turn - 1, '#war': 1 }, false, null, [], null, {}, [], plan)?.id).not.toBe(`tip:war:${city.id}`);
    v.turn += TIP_COOLDOWN.war;
    expect(strategyTip(v, { [`war:${city.id}`]: 20, '#war': 1 }, false, null, [], null, {}, [], plan)?.id).toBe(`tip:war:${city.id}`);
    enemy.pos = neighbors(city.pos)[0];
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)?.id).not.toBe(`tip:war:${city.id}`);
    enemy.pos = v.tiles.map((_, i) => hexAt(i, v.width)).find((pos) => distance(pos, city.pos) === 3)!;
    v.units.push({ ...enemy, id: enemy.id + 1 });
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)?.id).not.toBe(`tip:war:${city.id}`);
    v.diplomacy.relations[0].state = 'Peace';
    expect(strategyTip(v, {}, false, null, [], null, {}, [], plan)?.id).not.toBe(`tip:war:${city.id}`);
  });

  it('suggests Heal for a badly hurt unlocked unit near a visible enemy', async () => {
    const v = await game();
    const r = rival(v), unit = v.units.find((u) => u.owner === v.me && u.military)!;
    const enemy = { ...unit, id: unit.id + 9000, owner: r.id, kind: 'Spearman' as const, pos: neighbors(mine(v).pos)[0] };
    v.units.push(enemy);
    v.diplomacy.relations = [{ ...v.diplomacy.relations[0], a: v.team, b: r.team, state: 'War' }];
    unit.hp = 39;
    const name = itemName({ kind: 'Unit', unit: unit.kind }, v.players.find((p) => p.id === v.me)?.faction);
    expect(strategyTip(v, {})).toEqual({ id: `tip:heal:${unit.id}`, title: `Pull your ${name} back to heal`,
      why: 'Units heal faster in your territory. A badly hurt unit can die in one attack.',
      steps: [{ instruction: `Click your ${name}.`, target: { kind: 'tile', hex: unit.pos } },
        { instruction: `Use Heal, or move toward ${mine(v).name}.`, target: { kind: 'control', name: 'heal' } }],
      current: 0, acknowledge: true, tip: true });
    expect(strategyTip(v, {}, false, { kind: 'unit', id: unit.id })?.current).toBe(1);
    expect(strategyTip(v, { [`heal:${unit.id}`]: v.turn - 1, '#heal': 1 })?.id).not.toBe(`tip:heal:${unit.id}`);
    v.turn += TIP_COOLDOWN.heal;
    expect(strategyTip(v, { [`heal:${unit.id}`]: 20, '#heal': 1 })?.id).toBe(`tip:heal:${unit.id}`);
    unit.resting_heal = { total: 0, parts: [] };
    expect(strategyTip(v, {})?.steps[1]).toEqual({ instruction: `Move toward ${mine(v).name} to heal.`, target: { kind: 'tile', hex: mine(v).pos } });
    unit.resting_heal = null;
    unit.hp = 40;
    expect(strategyTip(v, {})?.id).not.toBe(`tip:heal:${unit.id}`);
    unit.hp = 39;
    unit.occupation_lock = r.id;
    expect(strategyTip(v, {})?.id).not.toBe(`tip:heal:${unit.id}`);
    unit.occupation_lock = null;
    v.diplomacy.relations[0].state = 'Peace';
    expect(strategyTip(v, {})?.id).not.toBe(`tip:heal:${unit.id}`);
  });

  it('points a starving city at an available Food build and stops when its net Food recovers', async () => {
    const v = await game();
    const city = mine(v), detail = city.detail!;
    city.population = 2;
    const granary = detail.options.find((o) => o.item.kind === 'Building' && o.item.building === 'Granary')!;
    detail.yields = { ...detail.yields, total: { ...detail.yields.total, food: 0 } };
    granary.yields = { ...granary.yields, total: { ...granary.yields.total, food: 2 } };
    expect(strategyTip(v, {})).toEqual({ id: `tip:food:${city.id}`, title: `${city.name} is starving`,
      why: `${city.name} loses Food each turn and will shrink.`,
      steps: [{ instruction: `Click ${city.name}.`, target: { kind: 'tile', hex: city.pos } },
        { instruction: 'Build Granary.', target: { kind: 'control', name: 'build', value: 'b:Granary' } }],
      current: 0, acknowledge: true, tip: true });
    expect(strategyTip(v, {}, false, { kind: 'city', id: city.id })?.current).toBe(1);
    expect(strategyTip(v, { [`food:${city.id}`]: v.turn - 1, '#food': 1 })?.id).not.toBe(`tip:food:${city.id}`);
    v.turn += TIP_COOLDOWN.food;
    expect(strategyTip(v, { [`food:${city.id}`]: 20, '#food': 1 })?.id).toBe(`tip:food:${city.id}`);
    city.population = 1;
    expect(strategyTip(v, {})?.why).toBe(`${city.name} needs more Food but cannot shrink below 1 population.`);
    city.population = 2;
    detail.yields.total.food = 2 * city.population;
    expect(strategyTip(v, {})?.id).not.toBe(`tip:food:${city.id}`);
    detail.yields.total.food = 0;
    detail.options = [];
    expect(strategyTip(v, {})?.id).not.toBe(`tip:food:${city.id}`);
  });

  it('suggests Peace for two locked units, but only with a living rival still at War', async () => {
    const v = await game();
    const r = rival(v), city = mine(v), unit = v.units.find((u) => u.owner === v.me)!;
    v.diplomacy.relations = [{ ...v.diplomacy.relations[0], a: v.team, b: r.team, state: 'War' }];
    unit.pos = city.pos;
    unit.occupation_lock = r.id;
    expect(strategyTip(v, {})?.id).not.toBe(`tip:garrison:${r.id}`);
    v.units.push({ ...unit, id: unit.id + 1000 });
    const people = ownerName(v, r.id);
    expect(strategyTip(v, {})).toEqual({ id: `tip:garrison:${r.id}`, title: 'Free your army',
      why: `A conquered city holds part of your army while you fight ${people}. Peace would free it.`,
      steps: [{ instruction: `Consider Peace with ${people} in Diplomacy.`, target: { kind: 'control', name: 'diplomacy' } }],
      current: 0, acknowledge: true, tip: true });
    v.diplomacy.relations[0].state = 'Peace';
    expect(strategyTip(v, {})?.id).not.toBe(`tip:garrison:${r.id}`);
    v.diplomacy.relations[0].state = 'War';
    r.alive = false;
    expect(strategyTip(v, {})?.id).not.toBe(`tip:garrison:${r.id}`);
    r.alive = true;
    unit.occupation_lock = null;
    v.units.find((u) => u.id === unit.id + 1000)!.occupation_lock = null;
    expect(strategyTip(v, {})?.id).not.toBe(`tip:garrison:${r.id}`);
  });

  it('suggests Peace after ten observed locked turns and rations repeat advice', async () => {
    const v = await game(30);
    const r = rival(v), unit = v.units.find((u) => u.owner === v.me)!;
    v.diplomacy.relations = [{ ...v.diplomacy.relations[0], a: v.team, b: r.team, state: 'War' }];
    unit.occupation_lock = r.id;
    const key = `${unit.id}:${r.id}`;
    expect(strategyTip(v, {}, false, null, [], null, { [key]: 21 })?.id).not.toBe(`tip:garrison:${r.id}`);
    expect(strategyTip(v, {}, false, null, [], null, { [key]: 20 })?.id).toBe(`tip:garrison:${r.id}`);
    const log = { [`garrison:${r.id}`]: 30, '#garrison': 1 };
    v.turn = 31;
    expect(strategyTip(v, log, false, null, [], null, { [key]: 20 })?.id).not.toBe(`tip:garrison:${r.id}`);
    v.turn = 30 + TIP_COOLDOWN.garrison;
    expect(strategyTip(v, log, false, null, [], null, { [key]: 20 })?.id).toBe(`tip:garrison:${r.id}`);
  });

  it('counts locked units across different wars toward the two-unit reminder', async () => {
    const v = await game();
    const r = rival(v), other = { ...r, id: 88, team: 88 };
    v.players.push(other);
    v.diplomacy.relations = [
      { ...v.diplomacy.relations[0], a: v.team, b: r.team, state: 'War' },
      { ...v.diplomacy.relations[0], a: v.team, b: other.team, state: 'War' },
    ];
    const unit = v.units.find((u) => u.owner === v.me)!;
    unit.occupation_lock = r.id;
    v.units.push({ ...unit, id: unit.id + 1000, occupation_lock: other.id });
    expect(strategyTip(v, {})?.id).toBe(`tip:garrison:${r.id}`);
  });
  it('Byzantium: a city it can afford to buy; the seller: what keeps its city', async () => {
    const v = await game();
    const r = rival(v);
    const city = mine(v);
    v.cities.push({ ...city, id: 7002, owner: r.id, name: 'Pylos', is_capital: false, pos: neighbors(neighbors(city.pos)[0])[0] });
    const breakdown = { total: 400, parts: [['Starting price', 400]] as [string, number][] };
    v.city_buys = [{ city: 7002, buyer: v.me, price: 400, breakdown }];
    expect(strategyTip(v, {})?.id).not.toBe('tip:buy');
    v.economy.gold = 450;
    const buy = strategyTip(v, {});
    expect(buy).toMatchObject({ id: 'tip:buy', title: 'You can buy Pylos', steps: [{ target: { kind: 'tile' } }, { target: { kind: 'control', name: 'buyCity' } }] });
    expect(buy!.why).toContain('costs 400 Gold');
    // A queued offer, or a refusal, ends it.
    expect(strategyTip(v, {}, false, null, [{ type: 'BuyCity', city: 7002 }])?.id).not.toBe('tip:buy');
    v.city_buys = [{ city: 7002, buyer: v.me, price: 400, breakdown, blocker: 'You are already buying Nicaea: one offer at a time' }];
    expect(strategyTip(v, {})?.id).not.toBe('tip:buy');

    // The seller's side: its own city under offer, while the offer still holds.
    v.city_buys = [];
    v.economy.gold = 50;
    v.city_offers = [{ city: city.id, buyer: r.id, seller: v.me, escrow: 500, ends_turn: 22, turns_left: 3, price_now: 450 }];
    const keep = strategyTip(v, {});
    expect(keep).toMatchObject({ id: `tip:offer:${city.id}:22`, title: `Keep ${city.name}?` });
    expect(keep!.steps[0].instruction).toContain('Move military units in or next to it, or declare War');
    v.city_offers = [{ ...v.city_offers[0], guards_to_stop: 2, walls_to_stop: 1 }];
    expect(strategyTip(v, {})!.steps[0].instruction).toContain('Move 2 more military units in or next to it, or declare War');
    v.city_offers = [{ ...v.city_offers[0], price_now: 600 }];
    expect(strategyTip(v, {})?.id ?? '').not.toMatch(/^tip:offer/);
  });

  it('suggests a buy only when the Gold is free after the other queued orders', async () => {
    const v = await game();
    const city = mine(v);
    v.cities.push({ ...city, id: 7002, owner: rival(v).id, name: 'Pylos', is_capital: false, pos: neighbors(neighbors(city.pos)[0])[0] });
    v.city_buys = [{ city: 7002, buyer: v.me, price: 400, breakdown: { total: 400, parts: [['Starting price', 400]] } }];
    v.economy.gold = 450;
    expect(strategyTip(v, {})?.id).toBe('tip:buy');
    const gift: Order = { type: 'GiveGold', to: rival(v).id, amount: 100 } as unknown as Order;
    expect(strategyTip(v, {}, false, null, [gift])?.id).not.toBe('tip:buy');
  });

  it('shows the seller notice even right after another tip', async () => {
    const v = await game();
    const r = rival(v);
    const city = mine(v);
    v.city_offers = [{ city: city.id, buyer: r.id, seller: v.me, escrow: 500, ends_turn: 22, turns_left: 3, price_now: 450 }];
    expect(strategyTip(v, { gold: v.turn - 1, '#gold': 1 })?.id).toBe(`tip:offer:${city.id}:22`);
  });

  it('stays quiet when nothing calls for advice, and in the first turns', async () => {
    const v = await game();
    expect(strategyTip(v, {})).toBeNull();
    v.turn = 2;
    v.economy.gold = 999;
    expect(strategyTip(v, {})).toBeNull();
  });

  it('warns that a much stronger neighbour may attack, with the numbers', async () => {
    const v = await game();
    const r = rival(v);
    // Team totals: your ally has no army here.
    v.players = v.players.map((p) => p.id === v.me ? { ...p, army: 12 } : p.team === v.team ? { ...p, army: 0 } : p.id === r.id ? { ...p, army: 40 } : p);
    const city = mine(v);
    v.cities.push({ ...city, id: 7001, owner: r.id, name: 'Rival town', pos: neighbors(neighbors(city.pos)[0])[0] });
    const tip = strategyTip(v, {});
    expect(tip).toMatchObject({ id: 'tip:army', title: 'Build up your army', acknowledge: true, tip: true });
    expect(tip!.why).toMatch(/^Army points: .+ 40, you 12\./);
    expect(tip!.why).not.toContain('AI ');
    // An ally's army counts as yours, so a strong team gets no warning.
    const ally = v.players.find((p) => p.id !== v.me && p.team === v.team);
    expect(ally).toBeTruthy();
    if (ally) {
      v.players = v.players.map((p) => (p.id === ally.id ? { ...p, army: 30 } : p));
      expect(strategyTip(v, {})?.id).not.toBe('tip:army');
    }
  });

  it('offers to buy a tile with the resource you lack, else to found a city by it', async () => {
    const v = await game();
    v.economy.gold = 100;
    const city = mine(v);
    const spot = v.tiles.map((t, i) => ({ t, pos: hexAt(i, v.width) })).find(({ t, pos }) => t.explored && t.owner === null && distance(pos, city.pos) === 3)!;
    v.tiles[tileIndex(spot.pos, v.width, v.height)] = { ...spot.t, resource: 'Iron' };
    v.supply = [iron()];
    city.detail = { ...city.detail!, buyable_tiles: [{ tile: spot.pos, ring: 3, price: 75 }] };
    const buy = strategyTip(v, {});
    expect(buy).toMatchObject({ id: 'tip:resource:Iron', title: `Buy the Iron tile near ${city.name}`, steps: [{ target: { kind: 'tile' } }, { target: { kind: 'control', name: 'buyTile' } }, { target: { kind: 'tile', hex: spot.pos } }] });
    // With Buy a tile open in that city, the last step rings the tile itself.
    expect(strategyTip(v, {}, false, { kind: 'city', id: city.id }, [], city.id)?.current).toBe(2);
    expect(buy!.why).toBe(`You have no Iron, and some units and buildings need it. ${city.name} can buy it for 75 Gold; then improve it.`);
    expect(strategyTip(v, {}, false, { kind: 'city', id: city.id })?.current).toBe(1);
    // Not when the treasury is short, the city can't buy, or the tile is already ordered: then settle instead.
    v.economy.gold = 74;
    expect(strategyTip(v, {})?.title).toBe('Found a city near the Iron');
    v.economy.gold = 200;
    expect(strategyTip(v, {}, false, null, [{ type: 'BuyTile', city: city.id, tile: spot.pos }])).toBeNull();
    expect(strategyTip(v, {}, false, null, [{ type: 'BuyItem', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }])?.title).toBe('Found a city near the Iron');
    city.detail = { ...city.detail!, buy_tile_blocker: 'Bought already this turn' };
    expect(strategyTip(v, {})?.title).toBe('Found a city near the Iron');
    city.detail = { ...city.detail!, buy_tile_blocker: null };
    city.detail = { ...city.detail!, buyable_tiles: [] };
    expect(strategyTip(v, {})).toMatchObject({ title: 'Found a city near the Iron', steps: [{ target: { kind: 'tile', hex: spot.pos } }] });
    v.supply = [iron({ deposits: [{ hex: spot.pos, city: city.id, mined: true, supply: 1 }], supplied: { total: 1, parts: [] } })];
    expect(strategyTip(v, {})).toBeNull();
  });

  it('never repeats: cooldown per tip, a gap between tips, but holds for the rest of its own turn', async () => {
    const v = await game(30);
    v.economy.gold = 400;
    expect(strategyTip(v, {})).toMatchObject({ id: 'tip:gold', steps: [{ target: { kind: 'tile', hex: mine(v).pos } }, { target: null }] });
    expect(strategyTip(v, { gold: 30 })?.id).toBe('tip:gold');
    v.turn = 31;
    expect(strategyTip(v, { gold: 30 })).toBeNull();
    v.turn = 30 + TIP_COOLDOWN.gold;
    expect(strategyTip(v, { gold: 30 })?.id).toBe('tip:gold');
    expect(strategyTip(v, { gold: 30, army: v.turn - TIP_GAP + 1 })).toBeNull();
    // Advice not taken fades: each repeat of a kind doubles its wait.
    expect(strategyTip(v, { gold: 30, '#gold': 2 })).toBeNull();
    v.turn = 30 + 2 * TIP_COOLDOWN.gold;
    expect(strategyTip(v, { gold: 30, '#gold': 2 })?.id).toBe('tip:gold');
  });

  it('a log left by a longer earlier game does not silence the next one', async () => {
    const v = await game(30);
    v.economy.gold = 400;
    expect(strategyTip(v, { gold: 100, '#gold': 3, army: 95 })?.id).toBe('tip:gold');
  });

  it('never tells the Swiss to spend Gold that earns them interest, and names a rival the plural way', async () => {
    const v = await game(30);
    v.economy.gold = 400;
    v.players = v.players.map((p) => p.id === v.me ? { ...p, faction: 'Switzerland' } : p);
    expect(strategyTip(v, {})).toBeNull();
  });

  it('comes before routine decisions, never over a unit in hand, and Got it dismisses it for the turn', async () => {
    const v = await game(30);
    v.economy.gold = 400;
    const ctx = { view: v, orders: [], selection: null, humans: 1, touch: false, plan: [] };
    expect(advisorSuggestion(ctx, ['welcome', 'report', 'howToWin'], [])?.id).toBe('tip:gold');
    const unit = v.units.find((u) => u.owner === v.me)!;
    expect(advisorSuggestion({ ...ctx, selection: { kind: 'unit', id: unit.id } }, ['welcome', 'report', 'howToWin'], [])?.id).not.toBe('tip:gold');
    expect(advisorSuggestion(ctx, ['welcome', 'report', 'howToWin'], ['tip:gold'])?.id).not.toBe('tip:gold');
  });
});
