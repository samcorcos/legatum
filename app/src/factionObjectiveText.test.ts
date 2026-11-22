import { expect, it } from 'vitest';
import { factionObjectiveText } from './data';

it('an era objective names the building a faction really builds', () => {
  expect(factionObjectiveText('Add a Forge to 3 more cities', 'Germany')).toBe('Add a Ruhr Foundry to 3 more cities');
  expect(factionObjectiveText('Add a Factory to 3 more cities', 'Germany')).toBe('Add a Factory to 3 more cities');
  expect(factionObjectiveText('Add a Factory to 3 more cities', 'India')).toBe('Add a Factory to 3 more cities');
  expect(factionObjectiveText('Add a Library to 6 more cities', 'Byzantium')).toBe('Add a Library to 6 more cities');
  expect(factionObjectiveText('Add a Library to 6 more cities', 'Athens')).toBe('Add a Lyceum to 6 more cities');
  expect(factionObjectiveText('Have 2 cities with a Library-line building', 'Byzantium')).toBe('Have 2 cities with a Library-line building');
  expect(factionObjectiveText('Add a Library to 6 more cities')).toBe('Add a Library to 6 more cities');
});
