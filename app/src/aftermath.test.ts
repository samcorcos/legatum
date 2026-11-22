import { describe, expect, it } from 'vitest';
import { attackAftermath } from './aftermath';
import { addCity, addUnit, arenaView } from './test/arenaView';
import { tileIndex } from './hex';
import type { BattleForecast } from './types';
import { classBonusStrength } from './combat';
import { unitStatsFor } from './data';

/** A Spearman's class bonus against a mounted unit (core `combat::class_bonus_strength`). */
const SPEAR = classBonusStrength(unitStatsFor('India', 'Spearman').strength);

const forecast = (over: Partial<BattleForecast> = {}): BattleForecast => ({
  attacker_strength: { total: 36, parts: [] },
  defender_strength: { total: 14, parts: [['Fortified', 6]] },
  win_chance: 60, loss_chance: 0, expected_damage_to_attacker: 20, expected_damage_to_defender: 90,
  attacker_hp: 100, defender_hp: 100, ranged: false, ...over,
});

const at = (q: number, r: number) => ({ q, r });

describe('the aftermath of a melee win', () => {
  it('is Safe when no known enemy can reach the advanced-to tile', async () => {
    const v = await arenaView();
    const horse = addUnit(v, 0, 'Horseman', at(5, 5));
    addUnit(v, 2, 'Archer', at(7, 5));
    const a = attackAftermath(v, horse, at(7, 5), forecast(), 80)!;
    expect(a.risk).toBe('Safe');
    expect(a.sources).toEqual([]);
    expect(a.severe).toBe(false);
    expect(a.line).toBe("If it destroys the Archer, your Horseman moves into its tile: no visible enemy can reach it next turn. Safe as far as you can see.");
  });

  it('counts a walled city in range and the units that can reach the tile, with the class bonus', async () => {
    const v = await arenaView('Switzerland');
    const horse = addUnit(v, 0, 'Horseman', at(8, 5));
    addUnit(v, 2, 'Archer', at(10, 5));
    addCity(v, 2, 'Nara', at(12, 5));
    addUnit(v, 2, 'Spearman', at(10, 7));
    addUnit(v, 2, 'Spearman', at(11, 6));
    const a = attackAftermath(v, horse, at(10, 5), forecast(), 63)!;
    expect(a.sources.map((s) => `${s.count} ${s.label}`)).toEqual(["1 Nara's walls", '2 Spearmen']);
    const spearmen = a.sources[1];
    expect(spearmen.bonus).toBe(`+${SPEAR} vs mounted`);
    expect(a.risk).toBe('Likely lost');
    expect(a.severe).toBe(true);
    expect(a.line).toMatch(/^If it destroys the Archer, your Horseman moves into its tile: next turn Nara's walls \(about −\d+\) and 2 Spearmen \(about −\d+ each, \+\d+ vs mounted\) can reach it\. Likely lost\.$/);
    expect(a.line).toContain(`+${SPEAR} vs mounted`);
  });

  it.each(['Forest', 'Hills'] as const)('warns about city fire past a %s', async (blocker) => {
    const v = await arenaView();
    const horse = addUnit(v, 0, 'Horseman', at(8, 5));
    addUnit(v, 2, 'Archer', at(10, 5));
    addCity(v, 2, 'Nara', at(12, 5));
    v.tiles[tileIndex(at(11, 5), v.width, v.height)].terrain = blocker;
    const a = attackAftermath(v, horse, at(10, 5), forecast(), 80)!;
    expect(a.sources.map((s) => s.label)).toContain("Nara's walls");
  });

  it('is Exposed when it survives the expected hits', async () => {
    const v = await arenaView('Switzerland');
    const spear = addUnit(v, 0, 'Spearman', at(8, 5));
    addUnit(v, 2, 'Archer', at(10, 5));
    addUnit(v, 2, 'Spearman', at(10, 7));
    const a = attackAftermath(v, spear, at(10, 5), forecast(), 100)!;
    expect(a.risk).toBe('Exposed');
    expect(a.sources).toHaveLength(1);
  });

  it('counts only units in the view (fog is never read)', async () => {
    const v = await arenaView();
    const horse = addUnit(v, 0, 'Horseman', at(8, 5));
    addUnit(v, 2, 'Archer', at(10, 5));
    // A Spearman out of sight is simply not in the view.
    expect(attackAftermath(v, horse, at(10, 5), forecast(), 100)!.risk).toBe('Safe');
  });

  it('is absent for a ranged attack, a city target or a hopeless attack', async () => {
    const v = await arenaView();
    const horse = addUnit(v, 0, 'Horseman', at(8, 5));
    addUnit(v, 2, 'Archer', at(10, 5));
    addCity(v, 2, 'Nara', at(5, 8));
    expect(attackAftermath(v, horse, at(10, 5), forecast({ ranged: true }), 100)).toBeNull();
    expect(attackAftermath(v, horse, at(10, 5), forecast({ win_chance: 0 }), 100)).toBeNull();
    expect(attackAftermath(v, horse, at(5, 8), forecast(), 100)).toBeNull();
  });

  it('puts a surviving Sparta winner back at full HP (kills heal)', async () => {
    const v = await arenaView();
    const horse = addUnit(v, 0, 'Horseman', at(8, 5));
    addUnit(v, 2, 'Archer', at(10, 5));
    expect(attackAftermath(v, horse, at(10, 5), forecast(), 79, true)!.hp).toBe(100);
    expect(attackAftermath(v, horse, at(10, 5), forecast(), 79, false)!.hp).toBe(79);
  });
});
