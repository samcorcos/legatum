import { describe, expect, it } from 'vitest';
import { BUILDING_NAME, FACTIONS, TECHS, UNIT_NAME } from '../data';
import type { BuildingKind, Faction, ResearchView, TechUnlock, UnitKind } from '../types';
import { techCardUnlocks } from './techCardUnlocks';
import { parseUpgradeEffect, unitMatchup, unlockHover, upgradesFromUnit } from './unlockCardModel';
import { BUILDING_LORE, BUILDING_WHY, UNIT_LORE } from './unlockLore';

const yields = { food: 0, production: 0, gold: 0, science: 0 };
const unit = (tech: TechUnlock['tech'], kind: UnitKind, extra: Partial<TechUnlock> = {}): TechUnlock => ({
  tech, item: { kind: 'Unit', unit: kind }, name: UNIT_NAME[kind], cost: 55, buy_cost: 160, yields, city_strength: 0, unit_strength: 25, upgrade_cost: 42, upgrade_from: 'Spearman', ...extra,
});
const building = (tech: TechUnlock['tech'], kind: BuildingKind, name: string, extra: Partial<TechUnlock> = {}): TechUnlock => ({
  tech, item: { kind: 'Building', building: kind }, name, cost: 80, buy_cost: 200, yields, city_strength: 0, unit_strength: 0, upgrade_cost: null, ...extra,
});
const research = (unlocks: TechUnlock[]) => ({ unlocks }) as unknown as ResearchView;
const hovers = (unlocks: TechUnlock[], tech: TechUnlock['tech'], faction: Faction = 'India') => techCardUnlocks(research(unlocks), tech, faction).map((e) => ({ e, h: unlockHover(e, faction) }));

describe('unit hovercard', () => {
  const [phalangite] = hovers([unit('Mathematics', 'Phalangite')], 'Mathematics');

  it('sells the unit: name, class, history, matchup', () => {
    const h = phalangite.h;
    expect(h.type).toBe('unit');
    if (h.type !== 'unit') return;
    expect(h.name).toBe('Phalangite');
    expect(h.classLabel).toBe('Melee line');
    expect(h.era).toBe('Classical');
    expect(h.flavour).toMatch(/sarissas/);
    expect(h.matchup?.line).toMatch(/mounted units/);
    expect(h.matchup?.beats).toEqual(['mounted units']);
    expect(h.matchup?.weakTo).toEqual(['ranged units']);
  });

  it('compares with the unit it replaces: strength 20 to 25 is +25%, with the upgrade price', () => {
    const h = phalangite.h;
    if (h.type !== 'unit') throw new Error('not a unit');
    expect(h.replaces).toEqual({ kind: 'Spearman', name: 'Spearman' });
    const strength = h.stats.find((s) => s.id === 'strength')!;
    expect([strength.from, strength.value, strength.delta]).toEqual([20, 25, { text: '+25%', tone: 'good' }]);
    expect(h.stats.find((s) => s.id === 'moves')).toMatchObject({ value: 2, from: 2, delta: null });
    // Cost is not compared (the view's price carries modifiers): it is the Build and Buy line.
    expect(h.stats.find((s) => (s.id as string) === 'cost')).toBeUndefined();
    expect(h.upgradeCost).toBe(42);
    expect(h.buyCost).toBe(160);
  });

  it('the class triangle in plain words, including its numbers', () => {
    expect(unitMatchup('Ranged')?.beats).toEqual(['the melee line']);
    expect(unitMatchup('Mounted')?.beats).toEqual(['ranged units']);
    expect(unitMatchup('Air')).toBeNull();
    expect(unitMatchup(null)).toBeNull();
    expect(unitMatchup('Melee')?.line).toMatch(/\+\d+% strength/);
  });

  it('a faction piece keeps its own name and history', () => {
    const [spearman] = hovers([unit('Feudalism', 'Pikeman', { name: 'Tagmatic Soldier' })], 'Feudalism', 'Byzantium');
    if (spearman.h.type !== 'unit') throw new Error('not a unit');
    expect(spearman.h.name).toBe('Tagmatic Soldier');
    expect(spearman.h.flavour).toMatch(/tagmata/);
  });

  it('a faction piece says how it differs from the standard unit it replaces; an ordinary unit does not', () => {
    const [spearman] = hovers([unit('Feudalism', 'Pikeman', { name: 'Tagmatic Soldier' })], 'Feudalism', 'Byzantium');
    if (spearman.h.type !== 'unit') throw new Error('not a unit');
    expect(spearman.h.piece?.against).toBe('Pikeman');
    expect(spearman.h.piece?.diffs.map((d) => [d.id, d.now, d.was])).toEqual([['cost', 162, 152], ['strength', 36, 32]]);
    const [pikeman] = hovers([unit('Feudalism', 'Pikeman')], 'Feudalism', 'India');
    if (pikeman.h.type !== 'unit') throw new Error('not a unit');
    expect(pikeman.h.piece).toBeNull();
  });

  it('a faction building says so too', () => {
    const [redoubt] = hovers([building('Masonry', 'Walls', 'Alpine Redoubt')], 'Masonry', 'Switzerland');
    if (redoubt.h.type !== 'building') throw new Error('not a building');
    expect(redoubt.h.piece?.diffs.map((d) => [d.id, d.now, d.was])).toEqual([['cost', 120, 100], ['cityStrength', 15, 10]]);
    // Its own line carries its own number, never the standard Walls' +10.
    expect(redoubt.h.rule).toMatch(/^\+15 city strength; the city fires/);
    const [walls] = hovers([building('Masonry', 'Walls', 'Walls')], 'Masonry', 'India');
    if (walls.h.type !== 'building') throw new Error('not a building');
    expect(walls.h.piece).toBeNull();
    expect(walls.h.rule).toMatch(/^\+10 city strength; the city fires/);
  });

  it('an unlock the faction cannot build yet carries the core reason in place of the prices', () => {
    const note = 'Walls for the Mongols from the Medieval era';
    const [walls] = hovers([building('Masonry', 'Walls', 'Walls', { locked_note: note })], 'Masonry', 'Mongols');
    if (walls.h.type !== 'building') throw new Error('not a building');
    expect(walls.e.locked).toBe(note);
    expect(walls.h.locked).toBe(note);
  });
});

describe('building hovercard', () => {
  it('says what it gives in numbers, why you want it, and what it costs', () => {
    const [library] = hovers([building('Writing', 'Library', 'Library', { yields: { ...yields, science: 3 } })], 'Writing');
    const h = library.h;
    if (h.type !== 'building') throw new Error('not a building');
    expect(h.yields).toEqual([['science', 3]]);
    expect(h.why).toMatch(/Science/);
    expect(h.flavour).toMatch(/Alexandria/);
    expect(h.cost).toBe(80);
    expect(h.buyCost).toBe(200);
  });

  it('an upgrade names what it upgrades and what population it needs', () => {
    const [forge] = hovers([building('IronWorking', 'Forge', 'Forge', { yields: { ...yields, production: 3 }, min_population: 4 })], 'IronWorking');
    if (forge.h.type !== 'building') throw new Error('not a building');
    expect(forge.h.upgradesFrom).toBe('Workshop');
    expect(forge.h.minPopulation).toBe(4);
  });
});

describe('effect hovercard', () => {
  it('an upgrade effect names both units, so both miniatures are drawn with an arrow', () => {
    const effect = hovers([], 'Mathematics').find((x) => /can upgrade to/.test(x.e.label))!;
    expect(effect.e.label).toBe('Spearmen can upgrade to Phalangites');
    if (effect.h.type !== 'effect') throw new Error('not an effect');
    expect(effect.h.upgrade).toEqual({ from: { kind: 'Spearman', name: 'Spearman' }, to: { kind: 'Phalangite', name: 'Phalangite' } });
    expect(effect.h.why).toMatch(/upgraded/);
    expect(parseUpgradeEffect('+1 Food in every city')).toBeNull();
  });

  it('a plain effect has an icon and a reason', () => {
    const [irrigation] = hovers([], 'Irrigation');
    if (irrigation.h.type !== 'effect') throw new Error('not an effect');
    expect(irrigation.h.icon).toBe('yield-food');
    expect(irrigation.h.why).toMatch(/[Ee]very city/);
  });
});

describe('every unlock sells itself', () => {
  it('has history for every unit and building and a reason for every building', () => {
    for (const kind of Object.keys(UNIT_NAME) as UnitKind[]) expect(UNIT_LORE[kind].length, kind).toBeGreaterThan(40);
    for (const kind of Object.keys(BUILDING_NAME) as BuildingKind[]) {
      expect(BUILDING_LORE[kind].length, kind).toBeGreaterThan(40);
      expect(BUILDING_WHY[kind].length, kind).toBeGreaterThan(30);
    }
  });

  it('every effect of every tech and faction builds a card with no stray artifacts', () => {
    for (const faction of FACTIONS) for (const tech of TECHS) for (const { e, h } of hovers([], tech, faction)) {
      const text = JSON.stringify(h);
      expect(text, `${faction} ${tech} ${e.label}`).not.toMatch(/undefined|NaN|null,|\[\]\./);
      expect(h.type).toBe('effect');
      if (h.type === 'effect') expect(h.icon, e.label).toBeTruthy();
    }
  });

  it('every unit has a place in its upgrade line', () => {
    expect(upgradesFromUnit('Phalangite')).toBe('Spearman');
    expect(upgradesFromUnit('Spearman')).toBeNull();
    expect(upgradesFromUnit('Knight')).toBe('Cataphract');
  });
});

describe('real unlocks from the engine', () => {
  it('every unlock of every tech, for every faction, builds a whole hovercard', async () => {
    const { FixtureClient } = await import('../test/fixtureEngine');
    for (const faction of FACTIONS) {
      const client = new FixtureClient();
      await client.newGame(await client.standardSetup('42', faction, 'Byzantium', 'Ada', 'Bo'));
      const r = client.view(0).research;
      for (const tech of TECHS) for (const entry of techCardUnlocks(r, tech, faction)) {
        const h = unlockHover(entry, faction);
        const text = JSON.stringify(h);
        expect(text, `${faction} ${tech} ${entry.label}`).not.toMatch(/undefined|NaN/);
        expect(h.type).toBe(entry.unlock?.item.kind === 'Unit' ? 'unit' : entry.unlock?.item.kind === 'Building' ? 'building' : 'effect');
        if (h.type === 'unit') {
          expect(h.name).toBe(entry.label);
          expect(h.stats.length, entry.label).toBeGreaterThan(0);
          expect(h.flavour.length, entry.label).toBeGreaterThan(30);
        }
        if (h.type === 'building') expect(h.name).toBe(entry.label);
      }
    }
  });
});
