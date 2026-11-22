import { describe, expect, it } from 'vitest';
import { BUILDING_NAME, FACTION_GOAL, FACTIONS, GREAT_WORK_ICON, buildingIconName, itemIcon, unitIconName, UNIT_STATS } from './data';
import { iconUrl, resourceArtUrl } from './icons';
import type { BuildingKind, UnitKind } from './types';

const UNITS = Object.keys(UNIT_STATS) as UnitKind[];
const BUILDINGS = Object.keys(BUILDING_NAME) as BuildingKind[];

describe('icon set', () => {
  it('provides prepared painted resource art at UI size', () => {
    const PAINTED = ['Wheat', 'Cattle', 'Stone', 'Copper', 'Iron', 'Coal', 'Niter', 'Oil', 'Silicon'];
    // Every strategic resource has an engraved mark (amounts, small chips, the fallback); bonus resources have none.
    for (const resource of ['Copper', 'Iron', 'Niter', 'Coal', 'Oil', 'Silicon']) expect(iconUrl(`resource-${resource.toLowerCase()}`), resource).not.toBeNull();
    for (const resource of ['Wheat', 'Cattle', 'Stone']) expect(iconUrl(`resource-${resource.toLowerCase()}`), resource).toBeNull();
    for (const resource of PAINTED) {
      expect(resourceArtUrl(resource), resource).toContain(`/assets/resources/${resource.toLowerCase()}-64`);
      // The map sprite path takes the capitalised Resource value too (1010c: the map drew no medallions).
      for (const size of [32, 64, 256] as const) expect(resourceArtUrl(resource, size), `${resource} ${size}`).toContain(`/assets/resources/${resource.toLowerCase()}-${size}`);
    }
    // Older views and saves retain the old resource name until core migrates them.
    for (const size of [32, 64, 256] as const) expect(resourceArtUrl('Silicon', size)).toContain(`/assets/resources/silicon-${size}`);
  });

  it('draws every unit and building, including each faction replacement', () => {
    for (const unit of UNITS) for (const faction of [undefined, ...FACTIONS]) expect(iconUrl(unitIconName(unit, faction)), `${faction ?? 'base'} ${unit}`).not.toBeNull();
    for (const building of BUILDINGS) for (const faction of [undefined, ...FACTIONS]) expect(iconUrl(buildingIconName(building, faction)), `${faction ?? 'base'} ${building}`).not.toBeNull();
  });

  it('gives every unit and building tier its own drawing', () => {
    const urls = [...UNITS.map((u) => unitIconName(u)), ...BUILDINGS.map((b) => buildingIconName(b))].map((n) => iconUrl(n));
    expect(new Set(urls).size).toBe(urls.length);
    for (const faction of FACTIONS) for (const unit of UNITS) {
      if (unitIconName(unit, faction) !== unitIconName(unit)) expect(iconUrl(unitIconName(unit, faction))).not.toBe(iconUrl(unitIconName(unit)));
    }
  });

  it('resolves faction pieces by their own name', () => {
    expect(itemIcon({ kind: 'Unit', unit: 'Cataphract' }, 'Mongols')).toBe('unit-keshig');
    expect(itemIcon({ kind: 'Building', building: 'Walls' }, 'Switzerland')).toBe('building-customsbastion');
    expect(itemIcon({ kind: 'Building', building: 'Walls' }, 'India')).toBe('building-walls');
  });

  it('draws every faction goal (the score check boxes show them)', () => {
    for (const f of FACTIONS) expect(iconUrl(FACTION_GOAL[f].icon), `${f}: ${FACTION_GOAL[f].icon}`).not.toBeNull();
  });

  it('has the icons the UI swaps in for text glyphs and map order badges', () => {
    const names = ['ui-alert', 'ui-siege', 'ui-random', 'ui-check', 'ui-cross', 'ui-trend-up', 'ui-trend-down', 'ui-chevron-up', 'ui-chevron-down', 'mark-move', 'mark-heal', 'mark-skip', 'mark-lock', 'mark-sleep', 'mark-found', 'mark-invalid', 'fortify', 'unit-scout', 'unit-archer', 'title-strongest-army'];
    for (const name of names) expect(iconUrl(name), name).not.toBeNull();
    expect(iconUrl(GREAT_WORK_ICON), GREAT_WORK_ICON).not.toBeNull();
  });
});
