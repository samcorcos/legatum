import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { explorationFixture, explorationCoreSave } from '../test/explorationFixture';
import { coreFixture } from '../test/coreFixture';
import { FixtureClient } from './fixtureEngine';
import { WasmClient } from '../wasmClient';

import { refreshVision, idxOf } from './fixtureEngine';
import { fromOffset, within, neighbors, distance } from '../hex';
import type { MState } from './fixtureState';
import type { Event } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let core: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  core = new WasmClient(mod.Game);
});

function ruinCoreSave(s: MState): string {
  const save = JSON.parse(explorationCoreSave(s));
  s.players.forEach((p, i) => Object.assign(save.players[i], { science_stored: p.science, researching: p.research, research_queue: p.research_queue, techs: p.techs }));
  s.terrain.forEach((_, i) => Object.assign(save.map.tiles[i], { owner: s.owner[i] < 0 ? null : s.owner[i], city: s.tileCity[i] < 0 ? null : s.tileCity[i] }));
  save.cities = Object.fromEntries(s.cities.map((c) => [c.id, { id: c.id, owner: c.owner, founder: c.founder, name: c.name, pos: c.pos,
    population: c.population, food_stored: c.food, production_stored: c.prod, producing: c.producing, bought: c.bought,
    buildings: c.buildings, hp: c.hp, grown_tiles: c.grownTiles, tiles_claimed: c.tilesClaimed,
    is_capital: c.isCapital, founded_turn: c.foundedTurn, resistance: c.resistance }]));
  save.next_city_id = s.nextCityId;
  return JSON.stringify(save);
}

for (const withCity of [false, true]) it.skipIf(!available).each(['India', 'Byzantium', 'Athens'] as const)(`matches WASM reward events and consequences (city ${withCity}): %s`, (faction) => {
  for (const seed of [0, 1, 6, 9, 16, '18446744073709551615']) {
    const s = explorationFixture();
    s.setup.seed = String(seed); s.terrain.fill('Plains'); s.ruin.fill(0);
    s.players[0].faction = faction; s.setup.players[0].faction = faction;
    s.players[1].isHuman = true; s.setup.players[1].is_human = true;
    // One plain step, with the seeded finder fixed to id 0 in both saves.
    const at = fromOffset(4, 4); s.ruin[idxOf(s, at)] = 1;
    for (const p of s.players) p.explored.fill(0);
    s.knownOwner = {}; s.knownCities = {}; s.knownRuins = {};
    refreshVision(s);
    if (withCity) {
      const pos = fromOffset(2, 4);
      s.cities = [{ id: 1, owner: 0, founder: 0, name: 'Home', pos, population: 2, food: 0, prod: 0,
        producing: { kind: 'Unit', unit: 'Scout' }, bought: null, buildings: [], hp: 200, grownTiles: [],
        tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0 }];
      s.nextCityId = 1;
      for (const h of within(pos, 1)) { s.owner[idxOf(s, h)] = 0; s.tileCity[idxOf(s, h)] = 1; }
      s.players[0].gold = 20; s.players[0].science = 20; s.players[0].research = 'BronzeWorking';
      refreshVision(s);
    }
    const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
    core.load(ruinCoreSave(s));
    const order = { type: 'MoveUnit', unit: 0, to: at } as const;
    const a = fixtureClient.endTurn({ 0: [order], 1: [] }); const b = core.endTurn({ 0: [order], 1: [] });
    const finds = (events: Event[]) => events.filter((e) => e.type === 'RuinExplored');
    (expect(finds(a.events)).toMatchSnapshot(), void (finds(b.events)));
    expect(finds(a.events)).toHaveLength(1);
    const reward = finds(a.events)[0].reward;
    if (reward.kind === 'FreeScout') {
      const id = reward.unit;
      const spawned = fixtureClient.view(0).units.find((u) => u.id === id)!;
      expect(distance(spawned.pos, at)).toBe(1);
      expect(core.view(0).units.find((u) => u.id === id)?.pos).toEqual(spawned.pos);
    }
  }
});

it.skipIf(!available)('matches WASM when Peace borders block every Scout gift tile', () => {
  const s = explorationFixture();
  s.setup.seed = '9'; s.terrain.fill('Plains'); s.ruin.fill(0);
  s.players[1].isHuman = true; s.setup.players[1].is_human = true;
  const at = fromOffset(4, 4);
  s.ruin[idxOf(s, at)] = 1;
  for (const h of neighbors(at)) s.owner[idxOf(s, h)] = 1;
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  core.load(coreFixture(s));
  const order = { type: 'MoveUnit', unit: 0, to: at } as const;
  const a = fixtureClient.endTurn({ 0: [order], 1: [] });
  const b = core.endTurn({ 0: [order], 1: [] });
  const finds = (events: Event[]) => events.filter((e) => e.type === 'RuinExplored');
  (expect(finds(a.events)).toMatchSnapshot(), void (finds(b.events)));
  expect(finds(a.events)).toMatchObject([{ reward: { kind: 'Gold', amount: 40 } }]);
  expect(fixtureClient.view(0).units.filter((u) => u.kind === 'Scout')).toHaveLength(1);
  expect(core.view(0).units.filter((u) => u.kind === 'Scout')).toHaveLength(1);
});
