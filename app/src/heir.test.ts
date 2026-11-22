import { expect, it } from 'vitest';
import { heirHoverLine, heirTag } from './heir';
import { minimapUnits } from './map/minimapView';
import type { PlayerView, UnitView } from './types';

const view = {
  me: 0, team: 0, width: 4, height: 4,
  players: [
    { id: 0, team: 0, faction: 'Sparta' }, { id: 1, team: 1, faction: 'Byzantium' }, { id: 2, team: 2, faction: 'India' },
  ],
  tiles: Array.from({ length: 16 }, () => ({ visible: false, explored: false })),
} as unknown as PlayerView;
const heir = (owner: number, heir_of = 1): UnitView => ({ id: 9, kind: 'HeirToTheThrone', owner, heir_of, pos: { q: 1, r: 1 } } as unknown as UnitView);

it('names whose Heir he is and who holds him, from the viewer\'s side', () => {
  expect(heirTag(view, heir(1))).toBe('Heir to the Throne');
  expect(heirTag(view, heir(2))).toBe('Held by India');
  expect(heirTag(view, heir(0))).toBe('You hold him');
  expect(heirTag(view, heir(255))).toBe('Held by barbarians');
  expect(heirTag({ ...view, me: 1, team: 1 } as PlayerView, heir(1))).toBe('Your Heir');
  expect(heirTag(view, { ...heir(1), heir_of: undefined } as UnitView)).toBeNull();
});

it('says what capturing him scores in the tile card line', () => {
  expect(heirHoverLine(view, heir(1), 2)).toBe('Be at War, then capture him to score 2 VP');
  expect(heirHoverLine(view, heir(2), 2)).toBe('Held by India: Be at War, then capture him to score 2 VP');
  expect(heirHoverLine(view, heir(0), 2)).toContain('You hold him');
  expect(heirHoverLine(view, heir(255), 2)).toContain('Held by barbarians');
});

it('the minimap shows every Heir, in fog too, and no other foreign unit out of sight', () => {
  const spearman = { id: 3, kind: 'Spearman', owner: 2, pos: { q: 2, r: 2 } } as unknown as UnitView;
  const units = minimapUnits({ ...view, units: [heir(1), spearman] } as PlayerView);
  expect(units.map((u) => [u.owner, u.heir])).toEqual([[1, true]]);
});
