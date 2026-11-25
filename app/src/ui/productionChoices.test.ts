import { expect, it } from 'vitest';
import { nestBuildingUpgrades, progressText, unavailableText, unitLines, withUnavailable } from './productionChoices';
import type { ProductionItem, ProductionOption, UnavailableOption, UnitKind } from '../types';
const option = (unit: UnitKind) => ({ item: { kind: 'Unit', unit } }) as ProductionOption;
it('folds only available superior successors and keeps every old choice discoverable', () => {
  const choices = ['Scout', 'Spearman', 'Pikeman', 'Musketman', 'Archer', 'Crossbowman', 'Settler'].map((u) => option(u as UnitKind));
  const lines = unitLines(choices, 'Sparta');
  expect(lines.map((l) => l.main.item)).toEqual(['Scout', 'Musketman', 'Crossbowman', 'Settler'].map((u) => option(u as UnitKind).item));
  expect(lines.find((l) => l.main.item.kind === 'Unit' && l.main.item.unit === 'Musketman')!.older.map((o) => o.item)).toEqual([option('Spearman').item, option('Pikeman').item]);
  expect(lines.flatMap((l) => [l.main, ...l.older])).toHaveLength(choices.length);
});
it('lists a unit line\'s older units oldest first, whatever order the core sends them', () => {
  const choices = ['Arquebusier', 'Archer', 'CompositeArcher', 'Crossbowman', 'FieldCannon'].map((u) => option(u as UnitKind));
  const line = unitLines(choices, 'India').find((l) => l.main.item.kind === 'Unit' && l.main.item.unit === 'FieldCannon')!;
  expect(line.older.map((o) => o.item.kind === 'Unit' && o.item.unit)).toEqual(['Archer', 'CompositeArcher', 'Crossbowman', 'Arquebusier']);
});
it('a unit its successor does not fold says what it still does better', () => {
  const lines = unitLines(['CompositeArcher', 'Crossbowman', 'Arquebusier'].map((u) => option(u as UnitKind)), 'Athens');
  expect(lines.map((l) => l.main.item.kind === 'Unit' && l.main.item.unit)).toEqual(['CompositeArcher', 'Arquebusier']);
  expect(lines[0].keeps).toBe('3 moves; the Arquebusier has 2');
  expect(lines[1].keeps).toBeUndefined();
  expect(unitLines([option('CompositeArcher')], 'Athens')[0].keeps).toBeUndefined();
});
it('does not fold a Spearman until a successor is actually available', () => {
  expect(unitLines([option('Spearman'), option('Archer')], 'India')).toHaveLength(2);
});

const settler: UnavailableOption = { item: { kind: 'Unit', unit: 'Settler' }, cost: 78, reason: 'Settler needs population 2 (currently 1)', production_banked: 0 };
it('puts greyed-out items in their usual place without reordering the choosable ones', () => {
  const rows = (['Scout', 'Spearman', 'Archer'] as UnitKind[]).map((u) => option(u));
  const merged = withUnavailable(rows, (o) => o.item, [settler]);
  expect(merged.map((r) => (r.row ? r.row.item : r.unavailable!.item) as ProductionItem)).toEqual(
    (['Scout', 'Settler', 'Spearman', 'Archer'] as UnitKind[]).map((u) => ({ kind: 'Unit', unit: u })));
  expect(withUnavailable([], (o: ProductionOption) => o.item, [settler])).toEqual([{ unavailable: settler }]);
});
it('nests missing building prerequisites into a full upgrade chain', () => {
  const granary: ProductionItem = { kind: 'Building', building: 'Granary' };
  const hydro: UnavailableOption = { item: { kind: 'Building', building: 'Mill' }, cost: 180, reason: 'Requires Granary', prerequisite_building: 'Granary', production_banked: 0 };
  const machine: UnavailableOption = { item: { kind: 'Building', building: 'Forge' }, cost: 260, reason: 'Requires Workshop', prerequisite_building: 'Workshop', production_banked: 0 };
  const factory: UnavailableOption = { item: { kind: 'Building', building: 'Factory' }, cost: 420, reason: 'Requires Machine Shop', prerequisite_building: 'Forge', production_banked: 0 };
  const rows = nestBuildingUpgrades([
    { row: { item: granary } as ProductionOption },
    { unavailable: hydro },
    { row: { item: { kind: 'Building', building: 'Workshop' } } as ProductionOption },
    { unavailable: machine }, { unavailable: factory },
  ]);
  expect(rows.map((n) => n.entry.row?.item ?? n.entry.unavailable?.item)).toEqual([
    granary, { kind: 'Building', building: 'Workshop' },
  ]);
  expect(rows[0].upgrades[0].entry.unavailable?.item).toEqual(hydro.item);
  expect(rows[1].upgrades[0].upgrades[0].entry.unavailable?.item).toEqual(factory.item);
});
it('leaves upgrade rows open when the prerequisite is already built or the blocker is technology', () => {
  const university: ProductionOption = { item: { kind: 'Building', building: 'University' } } as ProductionOption;
  const workshop: UnavailableOption = { item: { kind: 'Building', building: 'Forge' }, cost: 250, reason: 'Requires Fabrication', production_banked: 0 };
  const roots = nestBuildingUpgrades([{ row: university }, { unavailable: workshop }]);
  expect(roots).toHaveLength(2);
  expect(roots.every((node) => node.upgrades.length === 0)).toBe(true);
  expect(roots[0].entry.row?.item).toEqual(university.item);
});
it('explains a population gate with the growth forecast, or what to do when not growing', () => {
  expect(unavailableText(settler, 'First Signal', 1, { state: 'growing', turns: 4 }, 'Settler').line).toBe('Needs 2 population · First Signal grows to 2 in 4 turns');
  expect(unavailableText(settler, 'First Signal', 1, { state: 'growing', turns: 1 }, 'Settler').growth).toBe('First Signal grows to 2 in 1 turn');
  expect(unavailableText(settler, 'First Signal', 1, { state: 'stagnant', turns: null }, 'Settler').line).toBe('Needs 2 population · First Signal is not growing: work more Food or build something else first');
  expect(unavailableText(settler, 'First Signal', 1, { state: 'growing', turns: 4 }, 'Settler').full).toContain('A Settler takes 1 population with it');
  const college: UnavailableOption = { item: { kind: 'Building', building: 'University' }, cost: 120, reason: 'Requires population 5 (currently 2)', production_banked: 0 };
  expect(unavailableText(college, 'Hymnal', 2, { state: 'growing', turns: 3 }, 'University').line).toBe('Needs 5 population · Hymnal has 2; next growth in 3 turns');
  const hydro: UnavailableOption = { item: { kind: 'Building', building: 'Mill' }, cost: 100, reason: 'Requires Granary', production_banked: 0 };
  expect(unavailableText(hydro, 'Hymnal', 2, null, 'Mill')).toMatchObject({ line: 'Build Granary here first', growth: null });
});
it('labels progress by whose it is', () => {
  expect(progressText(15, 0, 78, true)).toBe('15/78 so far');
  expect(progressText(38, 38, 75, false)).toBe('38/75 banked');
  expect(progressText(0, 0, 75, false)).toBeNull();
});
it('leads a unit line with the newest unit the city can start, the one short of Iron under it', () => {
  const blocked = (unit: UnitKind) => ({ ...option(unit), resource_blocker: 'Needs 1 Iron: you have none yet.', supply_need: 'Iron' }) as ProductionOption;
  const choices = [option('Spearman'), blocked('Phalangite'), blocked('Pikeman'), option('Archer'), blocked('CompositeArcher')];
  const lines = unitLines(choices, 'India');
  const spear = lines.find((l) => l.main.item.kind === 'Unit' && l.main.item.unit === 'Spearman')!;
  expect(spear.better?.item).toEqual(option('Pikeman').item);
  expect(spear.older.map((o) => o.item)).toEqual([option('Phalangite').item]);
  expect(lines.find((l) => l.main.item.kind === 'Unit' && l.main.item.unit === 'Archer')!.better?.item).toEqual(option('CompositeArcher').item);
  expect(lines.flatMap((l) => [l.main, ...(l.better ? [l.better] : []), ...l.older])).toHaveLength(choices.length);
  // With nothing startable in the line, the newest unit still leads, with its reason on the card.
  expect(unitLines([blocked('Phalangite')], 'India')[0]).toMatchObject({ main: { item: option('Phalangite').item }, better: undefined });
  // A supplied line is unchanged.
  expect(unitLines([option('Spearman'), option('Phalangite')], 'India')[0]).toMatchObject({ main: { item: option('Phalangite').item }, better: undefined });
});
