import { describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { hexAt, neighbors, distance, tileIndex } from '../hex';
import type { PlayerView, SupplyView } from '../types';
import { advisorSuggestion } from './advisorModel';
import { strategyTip, TIP_COOLDOWN, TIP_GAP } from './advisorTips';

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
    v.players = v.players.map((p) => p.id === v.me ? { ...p, army: 12 } : p.id === r.id ? { ...p, army: 40 } : p);
    const city = mine(v);
    v.cities.push({ ...city, id: 7001, owner: r.id, name: 'Rival town', pos: neighbors(neighbors(city.pos)[0])[0] });
    const tip = strategyTip(v, {});
    expect(tip).toMatchObject({ id: 'tip:army', title: 'Build up your army', acknowledge: true, tip: true });
    expect(tip!.why).toMatch(/^Army points: .+ 40, you 12\./);
    expect(tip!.why).not.toContain('AI ');
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
    expect(advisorSuggestion(ctx, ['welcome', 'report', 'race'], [])?.id).toBe('tip:gold');
    const unit = v.units.find((u) => u.owner === v.me)!;
    expect(advisorSuggestion({ ...ctx, selection: { kind: 'unit', id: unit.id } }, ['welcome', 'report', 'race'], [])?.id).not.toBe('tip:gold');
    expect(advisorSuggestion(ctx, ['welcome', 'report', 'race'], ['tip:gold'])?.id).not.toBe('tip:gold');
  });
});
