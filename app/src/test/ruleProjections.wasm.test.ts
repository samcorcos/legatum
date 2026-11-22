import { expect, it } from 'vitest';
import { Game } from '../wasm-pkg/legatum.js';
import { buildingStatsFor, minCityGap, standardBuildingStatsFor, standardUnitStatsFor, terrainYieldsFor, unitStatsFor } from '../data';
import { addFightClassBonus, classBonus, classBonusStrength, cityDefenseCovered, damage, exactOdds, mountedCityPenalty, outriderBlocks, splitCounterattackLoss, unitDefenseStrength } from '../combat';
import { foundCityBlocker, settlementScores, siteBlocker, terrainYields } from '../settlement';
import { buildLensTiles } from '../map/lenses';
import { withProxySettler } from '../map/settleAdvice';
import { tileIndex } from '../hex';
import type { Breakdown, Faction, PlayerView, UnitKind } from '../types';

const core = (query: Record<string, unknown>): unknown => JSON.parse(Game.combat_projection(JSON.stringify(query)));

it('reads faction and baseline piece stats, terrain yields, and spacing from core projections', () => {
  for (const faction of ['India', 'Germany', 'Mongols', 'Celts', 'Athens', 'Switzerland'] as Faction[]) {
    for (const kind of ['Settler', 'Soldier', 'Cataphract', 'Ranger'] as UnitKind[]) {
      expect(unitStatsFor(faction, kind)).toMatchObject(core({ op: 'unitStats', faction, kind }) as object);
      expect(standardUnitStatsFor(faction, kind)).toMatchObject(core({ op: 'standardUnitStats', faction, kind }) as object);
    }
    for (const kind of ['Walls', 'Workshop', 'Laboratory'] as const) {
      expect(buildingStatsFor(faction, kind)).toMatchObject(core({ op: 'buildingStats', faction, kind }) as object);
      expect(standardBuildingStatsFor(faction, kind)).toMatchObject(core({ op: 'standardBuildingStats', faction, kind }) as object);
    }
    expect(minCityGap(faction)).toBe(core({ op: 'cityGap', faction }));
    for (const terrain of ['Grassland', 'Coast', 'Mountains'] as const) {
      expect(terrainYieldsFor(faction, terrain)).toEqual(core({ op: 'terrainYields', faction, terrain }));
      expect(terrainYields(faction, terrain)).toEqual(core({ op: 'terrainYields', faction, terrain }));
    }
  }
});

it('gives a mounted unit no bonus under fire, and keeps it charging and meeting', () => {
  const bonus = (own: UnitKind, other: UnitKind, attackerSide: boolean, shot: boolean) => {
    const b: Breakdown = { total: 0, parts: [] };
    addFightClassBonus(b, own, other, attackerSide, shot);
    return b.total;
  };
  expect(bonus('Cavalry', 'FieldCannon', false, true)).toBe(0);
  expect(bonus('Cavalry', 'FieldCannon', true, false)).toBeGreaterThan(0);
  expect(bonus('Cavalry', 'FieldCannon', false, false)).toBeGreaterThan(0);
  expect(bonus('Soldier', 'Cavalry', true, false)).toBeGreaterThan(0);
});

it('uses core combat projections for class, shot, wound, city, and exact-roll what-ifs', () => {
  for (const [own, other] of [['Soldier', 'Cataphract'], ['Cataphract', 'Ranger'], ['Tank', 'Ranger'], ['Cataphract', 'Soldier']] as [UnitKind, UnitKind][]) {
    expect(classBonus(own, other)).toEqual(core({ op: 'classBonus', own, other }));
    expect(classBonus(own, other, 'Mongols')).toEqual(core({ op: 'classBonus', own, other, faction: 'Mongols' }));
    for (const attacker_side of [false, true]) for (const shot of [false, true]) {
      const b: Breakdown = { total: 0, parts: [] };
      addFightClassBonus(b, own, other, attacker_side, shot, 'Mongols');
      const expected = core({ op: 'fightBonus', own, other, attacker_side, shot, faction: 'Mongols' }) as { label: string; bonus: number };
      expect(b.total).toBe(expected.bonus);
      expect(b.parts).toEqual(expected.bonus ? [[expected.label, expected.bonus]] : []);
    }
    for (const distance of [1, 2, 4]) expect(outriderBlocks(own, other, distance))
      .toBe(core({ op: 'outriderBlocks', attacker: own, defender: other, distance }));
  }
  for (const base of [20, 33, 79]) expect(classBonusStrength(base)).toBe(core({ op: 'classBonusStrength', base }));
  for (const kind of ['Horseman', 'Cataphract', 'Tank'] as UnitKind[]) for (const shot of [false, true])
    expect(mountedCityPenalty('Mongols', kind, shot)).toBe(core({ op: 'mountedCityPenalty', faction: 'Mongols', kind, shot }));
  for (const [own, other] of [[20, 26], [60, 20], [14, 80]]) {
    expect(damage(own, other, 87)).toBe(core({ op: 'damage', own, other, roll: 87 }));
    expect(exactOdds(own, other, 72, 100, false, 50)).toEqual(core({ op: 'exactOdds', atk: own, def: other, atk_hp: 72, def_hp: 100, ranged: false, city_percent: 50 }));
    expect(splitCounterattackLoss(other, own, 20, 2)).toBe(core({ op: 'splitCounterattackLoss', defender: other, attacker: own, attacker_hp: 20, attackers: 2 }));
  }
  expect(unitDefenseStrength('Soldier', 65, 'Forest', 6, 'Sparta', 2))
    .toEqual(core({ op: 'unitDefense', kind: 'Soldier', hp: 65, terrain: 'Forest', fortify: 6, faction: 'Sparta', level: 2 }));
  expect(cityDefenseCovered(4, 3, 1, 143, 'Switzerland', 2))
    .toEqual(core({ op: 'cityDefense', population: 4, tier: 3, covered: 1, hp: 143, faction: 'Switzerland', era: 2 }));
});

it('feeds the Settle lens and its blocker text from the same WASM site projection', () => {
  const game = new Game(Game.standard_setup('42', 'Germany', 'India', 'A', 'B'));
  try {
    const view: PlayerView = JSON.parse(game.view(0));
    const proxy = withProxySettler(view);
    expect(proxy).not.toBeNull();
    const unit = proxy!.unit;
    expect(unit.max_moves).toBe(unitStatsFor('Germany', 'Settler').moves);
    const scores = settlementScores(proxy!.view, unit, []);
    expect(scores).toEqual(JSON.parse(Game.settlement_sites(JSON.stringify(proxy!.view), unit.id, '[]')));
    const painted = buildLensTiles(proxy!.view, 'settle', scores);
    expect(painted.map((tile) => tile.index).sort((a, b) => a - b))
      .toEqual(scores.map((site) => tileIndex(site.pos, view.width, view.height)).sort((a, b) => a - b));
    for (const pos of [unit.pos, scores[0]?.pos].filter((p) => !!p)) {
      expect(siteBlocker(view, pos)).toEqual(JSON.parse(Game.site_blocker(JSON.stringify(view), pos.q, pos.r)));
      expect(foundCityBlocker(view, pos)).toEqual(siteBlocker(view, pos) === null ? null : expect.any(String));
    }
  } finally { game.free(); }
});
