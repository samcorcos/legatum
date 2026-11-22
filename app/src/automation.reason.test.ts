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

it('names what each focus picked and why', () => {
  const s = scene([building('Library'), unit('Horseman')]);
  expect(reason(unit('Horseman'), 'Military', s)).toBe('Horseman: the best unit your resources allow.');
  expect(reason(building('Library'), 'Economy', s)).toBe('Library: the most useful build for Geneva.');
  expect(reason({ kind: 'Project', project: 'Food' }, 'Food', s)).toBe('Food project: runs until you change the focus.');
  expect(reason({ kind: 'Project', project: 'Gold' }, 'Economy', s)).toBe('Gold project: nothing useful is left to build.');
});

it('explains a Military fallback without inventing a defence exception', () => {
  const s = scene([building('Granary'), unit('Horseman')]);
  expect(reason(building('Granary'), 'Military', s)).toBe('Granary: no unit is available now, so the city builds this.');
});

it('never falls back to a stock phrase', () => {
  const s = scene([building('Granary')]);
  for (const focus of ['Military', 'Economy', 'Gold', 'Science', 'Food'] as const) {
    for (const pick of [building('Granary'), unit('Spearman'), { kind: 'Project', project: 'Science' } as ProductionItem]) expect(reason(pick, focus, s)).not.toMatch(/best available choice|will choose/);
  }
});
