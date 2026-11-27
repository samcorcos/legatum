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
  }
  const client = new WasmClient(Game);
  client.load('{}');
  expect(client.forecast(0, 7, { q: 6, r: 5 }, mode)).toEqual({
    ranged: mode === 'Ranged', expected_damage_to_attacker: mode === 'Ranged' ? 0 : 20,
  });
  expect(forecast).toHaveBeenCalledExactlyOnceWith(0, 7, 6, 5, mode);
});

it('keeps the live game when replacement construction or loading fails', async () => {
  const freed: number[] = [];
  let nextId = 0;
  class Game {
    id = ++nextId;
    static failNew = false;
    static failLoad = false;
    constructor(_setup?: string) { if (Game.failNew) throw new Error('new failed'); }
    static load(_json: string) { if (Game.failLoad) throw new Error('load failed'); return new Game(); }
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
  expect(client.save()).toBe(original);
  expect(freed).toEqual([]);

  client.load('{}');
  expect(freed).toEqual([Number(original)]);
});

it('routes live-view settlement and reach queries through WasmGame instance methods and caches repeated calls', async () => {
  const { settlementScores, settlementSite, siteBlocker, unitsCanReach } = await import('./settlement');
  const playerSettlementSites = vi.fn(() => '[]');
  const playerSettlementSite = vi.fn(() => JSON.stringify({ pos: { q: 1, r: 1 }, score: 42, quality: 'Good', band: 'Good', border_risk: false, rank: 1, travel_turns: 0, breakdown: [] }));
  const playerSiteBlockers = vi.fn(() => JSON.stringify([null, null, null, null]));
  const playerUnitsCanReach = vi.fn(() => '[9]');
  const staticSites = vi.fn(() => '[]');
  const staticSite = vi.fn(() => 'null');
  const staticBlocker = vi.fn(() => 'null');

  class FakeGame {
    static load() { return new FakeGame(); }
    static standard_setup() { return '{}'; }
    static settlement_sites = staticSites;
    static settlement_site = staticSite;
    static site_blocker = staticBlocker;
    view() { return JSON.stringify({ me: 0, width: 2, height: 2, units: [], cities: [], players: [], tiles: [] }); }
    validate() { return '[]'; }
    forecast() { return 'null'; }
    assault() { return 'null'; }
    player_settlement_sites = playerSettlementSites;
    player_settlement_site = playerSettlementSite;
    player_site_blocker = vi.fn(() => 'null');
    player_site_blockers = playerSiteBlockers;
    player_units_can_reach = playerUnitsCanReach;
    end_turn() { return '{}'; }
    save() { return '{}'; }
    hash() { return '0'; }
    turn() { return 1; }
    archive() { return '{}'; }
    fork_archive() {}
  }

  const client = new WasmClient(FakeGame);
  client.load('{}');
  const v = client.view(0);
  const unit = { id: 3, pos: { q: 1, r: 1 } } as never;
  const enemy = { id: 9, pos: { q: 0, r: 0 } } as never;

  expect(siteBlocker(v, { q: 0, r: 0 })).toBeNull();
  expect(siteBlocker(v, { q: 1, r: 0 })).toBeNull();
  expect(playerSiteBlockers).toHaveBeenCalledTimes(1);

  expect(settlementScores(v, unit, [])).toEqual([]);
  expect(settlementScores(v, unit, [])).toEqual([]);
  expect(playerSettlementSites).toHaveBeenCalledTimes(1);

  expect(settlementSite(v, unit, { q: 1, r: 1 }, [])?.score).toBe(42);
  expect(settlementSite(v, unit, { q: 1, r: 1 }, [])?.score).toBe(42);
  expect(playerSettlementSite).toHaveBeenCalledTimes(1);

  expect([...unitsCanReach(v, [enemy], { q: 1, r: 1 })]).toEqual([9]);
  expect([...unitsCanReach(v, [enemy], { q: 1, r: 1 })]).toEqual([9]);
  expect(playerUnitsCanReach).toHaveBeenCalledTimes(1);

  // Client methods also use instance methods for live views and fall back for detached views.
  client.settlementSites(v, 3, []);
  client.settlementSite(v, 3, { q: 1, r: 1 }, []);
  expect(staticSites).not.toHaveBeenCalled();
  expect(staticSite).not.toHaveBeenCalled();

  client.endTurn({});
  client.settlementSites(v, 3, []);
  expect(staticSites).toHaveBeenCalledTimes(1);
});
