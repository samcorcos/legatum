// Promotions: the fixtureClient mirrors core's XP awards, promotion events, views and forecast lines.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { FixtureClient } from './fixtureEngine';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { explorationFixture } from '../test/explorationFixture';
import { spawnUnit } from './fixtureEngine';
import { fromOffset } from '../hex';
import { contextFromViews, formatEvents } from '../format';
import { cityAttackXp, nextLevelXp, promotionLevel, PROMOTION_XP, strengthBonus, unitFightXp } from '../promotion';
import type { GameClient } from '../client';
import type { Event } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

describe('promotion rules (mirror of core rules::promotion)', () => {
  it('uses the core thresholds and Civ 6 XP per fight', () => {
    expect(PROMOTION_XP).toEqual([16, 46, 90, 150, 226]);
    expect(promotionLevel(15)).toBe(0);
    expect(promotionLevel(16)).toBe(1);
    expect(promotionLevel(9999)).toBe(5);
    expect(nextLevelXp(20)).toBe(46);
    expect(nextLevelXp(9999)).toBeNull();
    expect(unitFightXp(20, 25, true, false, true)).toBe(6);
    expect(unitFightXp(25, 20, true, false, false)).toBe(4);
    expect(unitFightXp(20, 20, false, false, true)).toBe(4);
    expect(unitFightXp(15, 20, false, true, true)).toBe(3);
    expect(unitFightXp(20, 15, false, true, false)).toBe(2);
    expect(unitFightXp(5, 44, true, false, true)).toBe(8);
    expect(unitFightXp(0, 20, false, false, true)).toBe(0);
    expect([cityAttackXp(200, 150), cityAttackXp(20, 0), cityAttackXp(0, 0)]).toEqual([3, 10, 0]);
    expect([strengthBonus(20, 1), strengthBonus(20, 3), strengthBonus(15, 1), strengthBonus(5, 1), strengthBonus(20, 9)]).toEqual([2, 6, 1, 0, 12]);
  });
});

function fixture(attackerXp: number) {
  const s = explorationFixture();
  s.terrain.fill('Plains'); s.ruin.fill(0);
  for (const p of s.players) p.explored.fill(1);
  s.players[1].isHuman = true; s.setup.players[1].is_human = true;
  s.units[0].pos = fromOffset(1, 1); s.units[1].pos = fromOffset(0, 0); s.units[2].pos = fromOffset(19, 9);
  const soldier = spawnUnit(s, 0, 'Soldier', fromOffset(6, 4), false);
  soldier.xp = attackerXp;
  const ranger = spawnUnit(s, 0, 'Ranger', fromOffset(5, 3), false);
  const enemy = spawnUnit(s, 1, 'Soldier', fromOffset(7, 4), false);
  for (const r of s.diplomacy!.relations) r.state = 'War';
  return { s, soldier: soldier.id, ranger: ranger.id, enemy: enemy.id };
}

function play(engine: 'fixtureClient' | 'wasm') {
  const { s, soldier, ranger, enemy } = fixture(13);
  const client: GameClient = engine === 'fixtureClient' ? new FixtureClient() : makeCore();
  client.load(engine === 'fixtureClient' ? JSON.stringify(s) : coreFixture(s));
  const target = fromOffset(7, 4);
  const before = client.view(0);
  const forecastBefore = client.forecast(0, soldier, target, 'Melee');
  const melee = client.endTurn({ 0: [{ type: 'MoveUnit', unit: soldier, to: target }] });
  const afterMelee = client.view(0);
  client.load(client.save());
  const forecastAfter = client.forecast(0, soldier, target, 'Melee');
  const shotForecast = client.forecast(0, ranger, target, 'Ranged');
  const shot = client.endTurn({ 0: [{ type: 'RangedAttack', unit: ranger, target }, { type: 'SleepUnit', unit: soldier }] });
  const pick = (events: Event[]) => events.filter((e) => e.type === 'Battle' || e.type === 'UnitPromoted' || e.type === 'UnitDestroyed');
  const unitsOf = (player: number) => client.view(player).units
    .filter((u) => [soldier, ranger, enemy].includes(u.id))
    .map((u) => ({ id: u.id, hp: u.hp, level: u.level, xp: u.xp ?? null, xp_next: u.xp_next ?? null }));
  return {
    ids: { soldier, ranger, enemy },
    forecastBefore, forecastAfter, shotForecast,
    melee: pick(melee.events), shot: pick(shot.events),
    lines: formatEvents(melee.events, contextFromViews(before, afterMelee)).map((l) => l.text),
    mine: unitsOf(0), theirs: unitsOf(1),
  };
}

describe('WASM promotion behavior', () => {
  it('fixtureClient: a melee fight gives both survivors XP, promotes at the threshold and shows in forecasts', () => {
    const r = play('fixtureClient');
    const { soldier, ranger, enemy } = r.ids;
    expect(r.forecastBefore!.attacker_strength.parts.some(([l]) => l.startsWith('Promotion'))).toBe(false);
    expect(r.melee.filter((e) => e.type === 'UnitPromoted')).toEqual([
      { type: 'UnitPromoted', unit: soldier, owner: 0, unit_kind: 'Soldier', level: 1, at: fromOffset(6, 4) },
    ]);
    expect(r.lines).toContain('Your Spearman was promoted to level 1 (+10% strength)');
    expect(r.forecastAfter!.attacker_strength.parts[1]).toEqual(['Level 1: +10%', 2]);
    expect(r.shotForecast!.ranged).toBe(true);
    // 13 + 4 melee attack; the Ranger's shot is 20/15 + 1 + 1 = 3; the enemy defended twice: 3 + 2.
    expect(r.mine.find((u) => u.id === soldier)).toMatchObject({ level: 1, xp: 17, xp_next: 46 });
    expect(r.mine.find((u) => u.id === ranger)).toMatchObject({ level: 0, xp: 3, xp_next: 16 });
    expect(r.mine.find((u) => u.id === enemy)).toMatchObject({ level: 0, xp: null, xp_next: null });
    expect(r.theirs.find((u) => u.id === enemy)).toMatchObject({ level: 0, xp: 5, xp_next: 16 });
  });

  it.skipIf(!available)('wasm matches the fixtureClient exactly', () => {
    (expect(play('wasm')).toMatchSnapshot(), void (play('fixtureClient')));
  });
});
