import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { FACTION_INFO, costInFactionYield, costYield, uniquePieceBare } from '../data';
import { PieceCard } from './premium';

// The Nile: Egypt has no Production, so its prices read in the Food that pays them.
it('shows Egypt’s piece prices in Food and every other faction’s in Production', () => {
  expect(costYield('Egypt')).toBe('Food');
  expect(costYield('Rome')).toBe('Production');
  expect(costInFactionYield('Rome', 50)).toBe(50);
  const granary = renderToStaticMarkup(<PieceCard faction="Egypt" piece={{ kind: 'Building', building: 'Granary' }} name="Royal Granary" factionName="Egypt" replaces="Granary" text="" />);
  expect(granary).not.toMatch(/Production/);
  expect(granary).toMatch(/Food/);
  const medjay = renderToStaticMarkup(<PieceCard faction="Egypt" piece={{ kind: 'Unit', unit: 'Soldier' }} name="Medjay" factionName="Egypt" replaces="Spearman" text="" />);
  expect(medjay).not.toMatch(/Production/);
  expect(uniquePieceBare('Egypt')).toMatch(/: \d+ Food\./);
  expect(FACTION_INFO.Egypt.unique).toMatch(/: \d+ Food,/);
});
