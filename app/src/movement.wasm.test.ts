import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { WasmClient } from './wasmClient';
import { previewPath } from './path';
import { moveSelected } from './commands';
import { useGame } from './store';
import type { Hex, Order } from './types';

const available = existsSync(fileURLToPath(new URL('./wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
const middle = { q: 4, r: 4 };

async function fixture(city = false, holding = false) {
  const client = makeCore();
  const setup = await client.standardSetup('42', 'Switzerland', 'Switzerland', 'A', 'B');
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: true }));
  setup.width = setup.height = 20;
  await client.newGame(setup);
  // Fixture construction only: replace all u64 bitsets with zeros before serializing.
  // Subsequent save/load round trips always keep the save opaque.
  const s = JSON.parse(client.save());
  // These combat fixtures explicitly assume War; new games otherwise begin at Peace.
  s.diplomacy.relations.forEach((r: { state: string }) => { r.state = 'War'; });
  delete s.archive; // This constructed scenario is not a replay of the generated opening.
  s.map.tiles.forEach((t: { terrain: string; ruin: boolean; owner: number | null; city: number | null }) => Object.assign(t, { terrain: 'Grassland', ruin: false, owner: null, city: null }));
  s.players.forEach((p: { explored: { words: number[] }; tile_memory: unknown[] }) => { p.explored.words.fill(0); p.tile_memory.fill(null); });
  const unit = (id: number, owner: number, kind: string, pos: Hex, hp = 100) => ({ id, owner, kind, pos, hp, moves_left: 2, goto: null, fortify: 0, acted: false });
  s.units = {
    1: unit(1, 0, city ? 'Pikeman' : 'Archer', { q: 5, r: 4 }),
    2: unit(2, city ? 1 : 0, city ? 'Spearman' : 'Archer', holding ? middle : { q: 4, r: 5 }, city ? 10 : 100),
    3: unit(3, 0, 'Settler', { q: 1, r: 1 }),
    4: unit(4, 1, 'Settler', { q: 10, r: 10 }),
  };
  s.next_unit_id = 5;
  s.cities = city ? { 1: {
    id: 1, owner: 1, founder: 1, name: 'Redoubt', pos: middle, population: 3,
    food_stored: 0, production_stored: 0, producing: null, bought: null, buildings: [],
    hp: 0, border_progress: 0, tiles_claimed: 0, is_capital: true, founded_turn: 1, resistance: 0,
  } } : {};
  s.next_city_id = 2;
  client.load(JSON.stringify(s));
  return client;
}

describe.skipIf(!available)('F8/F12 real WASM scenarios', () => {
  beforeAll(async () => {
    const path = './wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('both confirmed Ranger orders follow the actual client preview, through save/load', async () => {
    vi.useFakeTimers();
    try {
      const client = await fixture();
      const view = client.view(0);
      const goals = [{ q: 4, r: 3 }, { q: 5, r: 3 }];
      useGame.setState(useGame.getInitialState(), true);
      useGame.setState({ client, humans: [0, 1], views: [view, client.view(1)], orders: [[], []], seat: 0 });
      const previews: Hex[][] = [];
      for (const [i, id] of [1, 2].entries()) {
        useGame.setState({ selection: { kind: 'unit', id } });
        const u = view.units.find((u) => u.id === id)!;
        previews.push(previewPath(view, u.pos, goals[i], useGame.getState().orders[0], id)!);
        moveSelected(goals[i]);
      }
      const orders = useGame.getState().orders[0];
      expect(orders).toEqual([1, 2].map((unit, i) => ({ type: 'MoveUnitPath', unit, to: goals[i], path: previews[i] })));
      expect(previews[0]).toContainEqual(middle);
      expect(previews[1]).toContainEqual(middle);
      expect(client.validate(0, orders)).toEqual([]);
      client.load(client.save());
      const result = client.endTurn({ 0: orders, 1: [] });
      for (const [i, id] of [1, 2].entries()) {
        const walked = result.events.flatMap((e) => e.type === 'UnitMoved' && e.unit === id ? [e.to] : []);
        expect(walked).toEqual(previews[i]);
        expect(client.view(0).units.find((u) => u.id === id)?.pos).toEqual(goals[i]);
      }
      const moved = result.events.flatMap((e) => e.type === 'UnitMoved' && (e.unit === 1 || e.unit === 2)
        ? [{ unit: e.unit, to: e.to }] : []);
      expect(moved).toEqual([
        { unit: 1, to: middle }, { unit: 1, to: goals[0] },
        { unit: 2, to: middle }, { unit: 2, to: goals[1] },
      ]);
      expect(result.events.some((e) => e.type === 'Bounce' || e.type === 'MoveBlocked')).toBe(false);
    } finally {
      useGame.setState(useGame.getInitialState(), true);
      vi.useRealTimers();
    }
  });

  it.each([false, true])('wins combat and captures a zero-HP city (holding defender: %s)', async (holding) => {
    const client = await fixture(true, holding);
    const forecast = client.forecast(0, 1, middle, 'Melee')!;
    const orders: Record<number, Order[]> = { 0: [{ type: 'MoveUnit', unit: 1, to: middle }], 1: holding ? [] : [{ type: 'MoveUnit', unit: 2, to: middle }] };
    const result = client.endTurn(orders);
    const fights = result.events.filter((e) => e.type === 'Battle');
    expect(fights).toHaveLength(1);
    expect(fights[0].damage_to_defender).toBeGreaterThan(0);
    // An incoming defender reinforces its own city first (user, 2026-10-04): the attack fights the
    // garrison it finds there, a melee rather than a meeting engagement.
    expect(fights[0].meeting).toBe(false);
    expect(result.events.some((e) => e.type === 'CityReinforced')).toBe(!holding);
    if (holding) {
      expect(fights[0].attacker_strength).toEqual(forecast.attacker_strength);
      expect(fights[0].defender_strength).toEqual(forecast.defender_strength);
    }
    expect(client.view(0).cities.find((c) => c.id === 1)?.owner).toBe(0);
    expect(client.view(0).units.find((u) => u.id === 1)?.pos).toEqual(middle);
  });
});
