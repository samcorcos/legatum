// @vitest-environment happy-dom
// Germany's Ruhr Foundry in the Forge slot: every prose table that names the Forge reads the viewer's own building
// (2026-11-06 UI review), and the supply breakdown shows the doubling instead of a bare number.
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { itemDescription, itemSummary, techCardEffectsFor } from './data';
import { resourceUsers, resourceUsersThisEra } from './resources';
import { missingText, SupplyTip } from './ui/SupplyChips';
import { techCardUnlocks } from './ui/techCardUnlocks';
import type { PlayerView, SupplyView } from './types';

it('the Factory card says it upgrades the Ruhr Foundry, and what stays standing', () => {
  const factory = { kind: 'Building', building: 'Autoworks' } as const;
  expect(itemSummary(factory, 'Germany')).toBe('+5 Production; upgrades your Ruhr Foundry');
  expect(itemSummary(factory, 'India')).toBe('+5 Production; upgrades your Forge');
  expect(itemDescription(factory, 'Germany')).toContain('Upgrades your Ruhr Foundry.');
  expect(itemDescription(factory, 'Germany')).toContain('The city keeps its Ruhr Foundry, and it still counts for Five Ruhr Foundries.');
  expect(itemDescription(factory, 'India')).not.toContain('keeps its');
});

it('the supply users name the viewer\'s Forge-slot building', () => {
  expect(resourceUsers('Iron', 'Germany')).toBe('your Classical and Medieval units, Ruhr Foundries and Universities');
  expect(resourceUsers('Iron', 'India')).toBe('your Classical and Medieval units, Forges and Universities');
  expect(resourceUsersThisEra('Iron', 2, 'Germany')).toBe('Pikemen, Crossbowmen, Knights, Universities, Ruhr Foundries and Siege');
  expect(resourceUsersThisEra('Iron', 3, 'Germany')).toBe('Ruhr Foundries, Universities, Siege and older Iron units');
  expect(resourceUsersThisEra('Iron', 3)).toBe('Forges, Universities, Siege and older Iron units');
  expect(missingText('Iron', 'Germany')).toContain('Ruhr Foundries');
  expect(missingText('Iron')).toContain('Forges');
});

it('the Machinery rule uses the viewer\'s own name', () => {
  expect(techCardEffectsFor('Germany', 'Machinery')).toContain('+1 Production for each Workshop, Ruhr Foundry and Factory');
  expect(techCardEffectsFor('India', 'Machinery')).toContain('+1 Production for each Workshop, Forge and Factory');
});

it('Iron Working tells Germany its deposits supply twice as much', () => {
  const research = { catalog: [], unlocks: [], techs: [], known: [] } as never;
  const germany = techCardUnlocks(research, 'IronWorking', 'Germany').map((u) => u.label);
  expect(germany).toContain('Reveals Iron on the map · your deposits supply twice as much');
  expect(techCardUnlocks(research, 'IronWorking', 'India').map((u) => u.label)).toContain('Reveals Iron on the map');
});

const iron = (): SupplyView => ({
  resource: 'Iron', revealed: true, revealed_by: 'IronWorking',
  supplied: { total: 6, parts: [['Iron near Astra', 6]] },
  deposits: [{ hex: { q: 3, r: 3 }, mined: true, supply: 6 }],
  used: { total: 3, parts: [['Pikeman ×3', 3]] }, idle: [], held: [], spare: 3,
} as unknown as SupplyView);
const view = (faction: string) => ({ me: 0, cities: [], players: [{ id: 0, team: 0, faction, name: 'You' }] }) as unknown as Pick<PlayerView, 'cities' | 'players' | 'me'>;

it('the supply breakdown shows the German doubling as its own row, so no number hides it', () => {
  const html = renderToStaticMarkup(<SupplyTip s={iron()} view={view('Germany')} />).replace(/<[^>]*>/g, '|').replace(/\|+/g, '|');
  expect(html).toContain('with a Mine');
  expect(html).toMatch(/with a Mine[^|]*\|3\|/);
  expect(html).toContain('Germany: deposits supply double|+3|');
  expect(html).toContain('Total|6|');
  const india = renderToStaticMarkup(<SupplyTip s={iron()} view={view('India')} />);
  expect(india).not.toContain('deposits supply double');
});
