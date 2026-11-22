import { expect, it } from 'vitest';
import { autoPickReason } from './automation';
import type { BuildingKind, CityBuildFocus, CityView, PlayerView, ProductionItem, ProductionOption, UnitKind } from './types';

const unit = (u: UnitKind): ProductionItem => ({ kind: 'Unit', unit: u });
const building = (b: BuildingKind): ProductionItem => ({ kind: 'Building', building: b });
const option = (item: ProductionItem) => ({ item, cost: 60 }) as ProductionOption;

function scene(options: ProductionItem[]) {
  const city = { id: 1, owner: 0, name: 'Geneva', pos: { q: 3, r: 3 }, detail: { options: options.map(option) } } as unknown as CityView;
  const view = { me: 0, players: [{ id: 0, faction: 'Switzerland' }],
    units: [], cities: [city] } as unknown as PlayerView;
  return { city, view };
}
const reason = (pick: ProductionItem, focus: CityBuildFocus, s: ReturnType<typeof scene>) => autoPickReason(pick, focus, s.city, s.view);

it('names a focus building as the focus pick', () => {
  const s = scene([building('Laboratory'), unit('Horseman')]);
  expect(reason(building('Laboratory'), 'Science', s)).toBe('Library: Science focus, the best research building available.');
  expect(reason(building('Market'), 'Gold', scene([building('Market')]))).toBe('Market: Gold focus, the best economy building available.');
});

it('explains a civilian fallback without inventing a defence exception', () => {
  const s = scene([building('Granary'), building('Laboratory'), unit('Horseman')]);
  expect(reason(building('Granary'), 'Science', s)).toBe('Granary: no Science building can start now, so this is the next best build.');
  expect(reason(building('Granary'), 'Military', s)).toBe("Granary: Not enough Gold to pay another unit's upkeep, so it builds the next best thing.");
});

it('never falls back to a stock phrase', () => {
  const s = scene([building('Granary')]);
  for (const focus of ['Balanced', 'Science', 'Gold', 'Military'] as const) {
    for (const pick of [building('Granary'), unit('Settler'), unit('Scout'), unit('Soldier')]) expect(reason(pick, focus, s)).not.toMatch(/best available choice|will choose/);
  }
  expect(reason(building('Granary'), 'Science', s)).toBe('No Science building available; a Granary is the next best build.');
});
