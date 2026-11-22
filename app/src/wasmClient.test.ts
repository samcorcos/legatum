import { expect, it, vi } from 'vitest';
import { WasmClient } from './wasmClient';
import type { AttackMode } from './types';

it.each(['Melee', 'Ranged'] as const)('passes explicit %s mode through the WASM boundary', (mode) => {
  const forecast = vi.fn((_player: number, _unit: number, _q: number, _r: number, attackMode: AttackMode) => JSON.stringify({
    ranged: attackMode === 'Ranged',
    expected_damage_to_attacker: attackMode === 'Ranged' ? 0 : 20,
  }));
  class Game {
    static load() { return new Game(); }
    static settlement_sites() { return '[]'; }
    static settlement_site() { return 'null'; }
    static site_blocker() { return 'null'; }
    static standard_setup() { return '{}'; }
    view() { return '{}'; }
    validate() { return '[]'; }
    assault() { return 'null'; }
    forecast = forecast;
    end_turn() { return '{}'; }
    save() { return '{}'; }
    hash() { return '0'; }
    turn() { return 1; }
    archive() { return '{}'; }
    fork_archive() {}
    static load_scenario() { return new Game(); }
    attach_scenario() {}
    scenario_status() { return '{}'; }
  }
  const client = new WasmClient(Game);
  client.load('{}');
  expect(client.forecast(0, 7, { q: 6, r: 5 }, mode)).toEqual({
    ranged: mode === 'Ranged', expected_damage_to_attacker: mode === 'Ranged' ? 0 : 20,
  });
  expect(forecast).toHaveBeenCalledExactlyOnceWith(0, 7, 6, 5, mode);
});

it('keeps the live game when replacement construction, loading, or scenario attachment fails', async () => {
  const freed: number[] = [];
  let nextId = 0;
  class Game {
    id = ++nextId;
    static failNew = false;
    static failLoad = false;
    static failScenario = false;
    static failAttach = false;
    constructor(_setup?: string) { if (Game.failNew) throw new Error('new failed'); }
    static load(_json: string) { if (Game.failLoad) throw new Error('load failed'); return new Game(); }
    static load_scenario(_json: string) { if (Game.failScenario) throw new Error('scenario failed'); return new Game(); }
    static standard_setup() { return '{}'; }
    static settlement_sites() { return '[]'; }
    static settlement_site() { return 'null'; }
    static site_blocker() { return 'null'; }
    view() { return '{}'; }
    validate() { return '[]'; }
    forecast() { return 'null'; }
    assault() { return 'null'; }
    end_turn() { return '{}'; }
    save() { return String(this.id); }
    hash() { return '0'; }
    turn() { return 1; }
    archive() { return '{}'; }
    fork_archive() {}
    attach_scenario() { if (Game.failAttach) throw new Error('attach failed'); }
    scenario_status() { return '{}'; }
    free() { freed.push(this.id); }
  }
  const client = new WasmClient(Game);
  await client.newGame({} as never);
  const original = client.save();
  Game.failNew = true;
  await expect(client.newGame({} as never)).rejects.toThrow('new failed');
  Game.failNew = false;
  Game.failLoad = true;
  expect(() => client.load('{}')).toThrow('load failed');
  Game.failLoad = false;
  Game.failScenario = true;
  expect(() => client.loadScenario('{}')).toThrow('scenario failed');
  Game.failScenario = false;
  expect(client.save()).toBe(original);
  expect(freed).toEqual([]);

  client.loadScenario('scenario');
  expect(freed).toEqual([Number(original)]);
  const scenarioGame = client.save();
  Game.failAttach = true;
  expect(() => client.load('{}')).toThrow('attach failed');
  expect(client.save()).toBe(scenarioGame);
  expect(freed).toEqual([Number(original), Number(scenarioGame) + 1]);
  Game.failAttach = false;
  client.load('{}');
  expect(freed).toContain(Number(scenarioGame));
});
