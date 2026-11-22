import { describe, expect, it } from 'vitest';
import { FACTIONS, TECHS, UNIT_NAME, itemName, techHasBenefit, unitStatsFor } from '../data';
import { iconUrl, resourceArtUrl } from '../icons';
import { eraResource } from '../resources';
import type { Faction, ResearchView, TechUnlock, UnitKind } from '../types';
import { techCardUnlocks } from './techCardUnlocks';
import { parseUpgradeEffect } from './unlockCardModel';
import RULES from '../rules.generated.json';

const research = { unlocks: [] } as unknown as ResearchView;
const icons = (tech: Parameters<typeof techCardUnlocks>[1], faction: Faction = 'India') => techCardUnlocks(research, tech, faction).map((u) => u.icon);

describe('tech card effect icons', () => {
  it('draw the building, unit or yield an effect names', () => {
    expect(icons('Irrigation')).toEqual(['yield-food']);
    expect(icons('CodeOfLaws')).toEqual(['effect-insight']);
    // The Phalangite is an item on the view's unlock list; the upgrade and Siege lines are effects.
    expect(icons('Mathematics')).toEqual(['ui-siege', 'unit-soldier']);
    expect(icons('Feudalism')).toEqual(['unit-phalangite']);
    expect(icons('MilitaryEngineering')).toEqual(['yield-gold']);
    expect(icons('PrintingPress')).toEqual(['building-laboratory', 'building-market']);
    expect(icons('FusionPower')).toEqual(['yield-production', 'building-inferencecore']);
    expect(icons('StealthTechnology')).toContain('resource-silicon');
    expect(icons('IronWorking')).toContain('resource-iron');
  });

  it('every effect icon exists in the icon set', () => {
    for (const faction of FACTIONS) for (const tech of TECHS) for (const icon of icons(tech, faction)) {
      const art = icon.startsWith('resource-') ? resourceArtUrl(icon.slice('resource-'.length)) : iconUrl(icon);
      expect(art, `${tech}: ${icon}`).not.toBeNull();
    }
  });

  it('names every strategic resource reveal with its medallion', () => {
    for (const [tech, resource] of [['IronWorking', 'Iron'], ['Gunpowder', 'Niter'], ['Industrialization', 'Coal'], ['Combustion', 'Oil'], ['AutonomousSystems', eraResource(8)!]] as const) {
      expect(techCardUnlocks(research, tech, 'India')).toContainEqual(expect.objectContaining({
        label: `Reveals ${resource} on the map`, icon: `resource-${resource.toLowerCase()}`,
      }));
    }
  });

  it('places each resource reveal before bonuses that depend on its deposit', () => {
    for (const [tech, resource] of [['Combustion', 'Oil'], ['IronWorking', 'Iron'], ['Gunpowder', 'Niter'], ['Industrialization', 'Coal'], ['AutonomousSystems', 'Silicon']] as const) {
      const labels = techCardUnlocks(research, tech, 'India').map((u) => u.label);
      const reveal = labels.findIndex((label) => label.startsWith(`Reveals ${resource} on the map`));
      expect(reveal, tech).toBeGreaterThanOrEqual(0);
      for (const [index, label] of labels.entries()) if (new RegExp(`\\b${resource} (?:Wells?|Mines?|deposits?)\\b`).test(label) && index !== reveal)
        expect(reveal, `${tech}: ${label}`).toBeLessThan(index);
    }
    const combustion = techCardUnlocks(research, 'Combustion', 'India').map((u) => u.label);
    expect(combustion.indexOf('Reveals Oil on the map')).toBeLessThan(combustion.findIndex((label) => label.includes('Oil Wells')));
  });

  it('say which unit line upgrades, with that unit as the icon', () => {
    expect(techCardUnlocks(research, 'Chivalry', 'India').map((u) => u.label)).toEqual(['Cataphracts can upgrade to Knights']);
    expect(icons('Chivalry')).toEqual(['unit-cataphract']);
    expect(techCardUnlocks(research, 'Rifling', 'India').map((u) => u.label)).toEqual(['Musketmen can upgrade to Riflemen']);
    // The Mongols buy Siege from the Medieval era, so their cards carry the Siege lines too.
    expect(techCardUnlocks(research, 'Metallurgy', 'Mongols').map((u) => u.label)).toEqual(['Siege units now ignore Star Forts', 'Knights can upgrade to Cuirassiers']);
    expect(techCardUnlocks(research, 'Metallurgy', 'India').map((u) => u.label)).toEqual(['Siege units now ignore Star Forts', 'Knights can upgrade to Cuirassiers']);
    expect(techCardUnlocks(research, 'Machinery', 'India').map((u) => u.label)).toEqual(['Siege units now ignore Castles', '+1 Production for each Workshop, Forge and Factory', 'Composite Archers can upgrade to Crossbowmen']);
    expect(icons('Machinery')).toEqual(['ui-siege', 'building-workshop', 'unit-compositearcher']);
  });
});

describe('faction-accurate tech cards', () => {
  it('folds upgrade paths into unlocked units and keeps unmatched upgrades standalone', () => {
    const unit = (kind: UnitKind): TechUnlock => ({
      tech: 'Gunpowder', item: { kind: 'Unit', unit: kind }, name: UNIT_NAME[kind], cost: 210,
      buy_cost: 420, yields: { food: 0, production: 0, gold: 0, science: 0 }, city_strength: 0,
      unit_strength: 40, upgrade_cost: 100,
    });
    const unlocked = { unlocks: [unit('AssaultTrooper'), unit('Arquebusier')] } as ResearchView;
    const rows = techCardUnlocks(unlocked, 'Gunpowder', 'India');
    expect(rows.filter((u) => u.kind !== 'Effect').map((u) => [u.label, u.upgradeFrom])).toEqual([
      ['Musketman', 'Pikeman'], ['Arquebusier', 'Crossbowman'],
    ]);
    // Niter's reveal is the only effect: both upgrade lines folded into their units.
    expect(rows.filter((u) => u.kind === 'Effect').map((u) => u.label)).toEqual(['Reveals Niter on the map']);
    const withoutTarget = { unlocks: [unit('AssaultTrooper')] } as ResearchView;
    expect(techCardUnlocks(withoutTarget, 'Gunpowder', 'India').map((u) => u.label)).toEqual([
      'Musketman', 'Reveals Niter on the map', 'Crossbowmen can upgrade to Arquebusiers',
    ]);
  });
  it('show Byzantium every effect, city Science included, like every other faction', () => {
    for (const tech of ['StealthTechnology', 'PrintingPress'] as const) {
      expect(techCardUnlocks(research, tech, 'Byzantium').map((u) => u.label)).toEqual(techCardUnlocks(research, tech, 'India').map((u) => u.label));
      expect(techCardUnlocks(research, tech, 'Byzantium')).not.toEqual([]);
    }
    expect(techCardUnlocks(research, 'PrintingPress', 'Athens').map((u) => u.label)).toEqual(['+1 Science for each Library, University and Research Lab', '+1 Gold for each Market and Bank']);
  });

  it('every card a faction researches gives it something; a tech with nothing for it is skipped', () => {
    expect(techHasBenefit('Byzantium', 'PrintingPress')).toBe(true);
    // Since 2026-10-19 the Mongols raise Walls from the Medieval era, so Masonry helps them too.
    expect(techHasBenefit('Mongols', 'Masonry')).toBe(true);
    // The Nile: Egypt has no Production, so Bronze Working's Workshop does nothing for it: Egypt skips it.
    const noBenefit = new Set<string>(['Egypt BronzeWorking']);
    // Egypt's cards drop the lines that only add Production.
    expect(techCardUnlocks(research, 'Masonry', 'Egypt')).toEqual([]);
    // ...and its building and wonder tiles drop their Production yield (the Pyramids' +1 Production).
    const pyramids = { unlocks: [{ tech: 'Masonry', item: { kind: 'Building', building: 'Pyramids' }, name: 'Pyramids', cost: 240, buy_cost: null,
      yields: { food: 0, production: 1, gold: 0, science: 0 }, city_strength: 0, unit_strength: 0, upgrade_cost: null }] } as unknown as ResearchView;
    expect(techCardUnlocks(pyramids, 'Masonry', 'Egypt')[0].yields).toEqual([]);
    expect(techCardUnlocks(pyramids, 'Masonry', 'India')[0].yields).toEqual([['production', 1]]);
    expect(techCardUnlocks(research, 'Machinery', 'Egypt').map((u) => u.label)).not.toContain('+1 Production for each Workshop, Forge and Factory');
    for (const faction of FACTIONS) for (const tech of TECHS) expect(techHasBenefit(faction, tech), `${faction} ${tech}`).toBe(!noBenefit.has(`${faction} ${tech}`));
  });
});

describe('a tech card states a unit once', () => {
  const unitsOf = (tech: string) => Object.entries(RULES.unitStats as unknown as Record<string, { requires: string | null }>).filter(([, u]) => u.requires === tech).map(([kind]) => kind as UnitKind);
  const unlockOf = (tech: Parameters<typeof techCardUnlocks>[1], unit: UnitKind, faction: Faction): TechUnlock => {
    const stats = unitStatsFor(faction, unit);
    return {
      tech, item: { kind: 'Unit', unit }, name: itemName({ kind: 'Unit', unit }, faction), cost: stats.cost, buy_cost: null,
      yields: { food: 0, production: 0, gold: 0, science: 0 }, city_strength: 0, unit_strength: stats.strength, unit_ranged: stats.ranged, upgrade_cost: null,
    };
  };
  const cardOf = (tech: Parameters<typeof techCardUnlocks>[1], faction: Faction) =>
    techCardUnlocks({ unlocks: unitsOf(tech).map((unit) => unlockOf(tech, unit, faction)) } as ResearchView, tech, faction);

  it('puts the Stealth aircraft reach, guard, intercept, Silicon and upgrade path on their rows', () => {
    const card = cardOf('StealthTechnology', 'India');
    const ranged = (unit: UnitKind) => `${unitStatsFor('India', unit).ranged} ranged`;
    expect(card.map((u) => u.label)).toEqual(['Stealth Bomber', 'Stealth Fighter', 'Hangars hold 3 aircraft']);
    const row = (name: string) => card.find((u) => u.label === name)!;
    expect([row('Stealth Fighter').strength, ...row('Stealth Fighter').notes]).toEqual([ranged('StealthFighter'), 'strikes 7 tiles, guards 5', 'uses Silicon']);
    expect([row('Stealth Bomber').strength, ...row('Stealth Bomber').notes]).toEqual([ranged('StealthBomber'), 'strikes 10 tiles; only Stealth Fighters stop it', 'uses Silicon']);
    expect([row('Stealth Fighter').upgradeFrom, row('Stealth Bomber').upgradeFrom]).toEqual(['Jet Fighter', 'Jet Bomber']);
    expect(card.some((u) => /can upgrade/.test(u.label))).toBe(false);
  });

  it('keeps a rule with no unit on the card as a rule', () => {
    expect(techCardUnlocks(research, 'StealthTechnology', 'India').map((u) => u.label)).toEqual([
      'Stealth aircraft use Silicon instead of Oil', 'Hangars hold 3 aircraft',
      'Jet Fighters can upgrade to Stealth Fighters: strike up to 7 tiles away; guard 5 tiles around their city',
      'Jet Bombers can upgrade to Stealth Bombers: strike up to 10 tiles away; only Stealth Fighters intercept them',
    ]);
  });

  it('no rule row on any card names a unit that the same card unlocks, unless it is an upgrade', () => {
    for (const faction of FACTIONS) for (const tech of TECHS) {
      const card = cardOf(tech, faction);
      const names = card.filter((u) => u.kind === 'Unit').map((u) => u.label);
      for (const rule of card.filter((u) => u.kind === 'Effect' && !u.upgrades.length && !parseUpgradeEffect(u.label))) {
        for (const name of names) expect(rule.label, `${faction} ${tech}: "${rule.label}" repeats ${name}`).not.toMatch(new RegExp(`\\b${name}`));
      }
    }
  });
});
