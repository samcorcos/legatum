import { expect, it } from 'vitest';
import { BUILDING_KINDS, FACTIONS, IMPROVEMENT_KINDS, UNIT_KINDS, itemDescription, itemSummary, moreText } from './data';
import type { ProductionItem } from './types';

/** Keep the complete list explicit: additions to any production kind need a short data line. */
const items: ProductionItem[] = [
  ...UNIT_KINDS.map((unit): ProductionItem => ({ kind: 'Unit', unit })),
  ...BUILDING_KINDS.map((building): ProductionItem => ({ kind: 'Building', building })),
  ...(['GreatLibrary', 'RoyalArsenal', 'GrandBazaar', 'YamRelayStation'] as const).map((building): ProductionItem => ({ kind: 'Wonder', building, tile: { q: 0, r: 0 } })),
  ...IMPROVEMENT_KINDS.map((improvement): ProductionItem => ({ kind: 'Improvement', improvement, tile: { q: 0, r: 0 } })),
  ...(['Gold', 'Science'] as const).map((project): ProductionItem => ({ kind: 'Project', project })),
];

it('every production item and faction variant has one nonempty summary of at most 80 characters', () => {
  for (const item of items) for (const faction of FACTIONS) {
    const summary = itemSummary(item, faction);
    expect(summary?.trim(), `${faction} ${JSON.stringify(item)} has no summary`).toBeTruthy();
    expect(summary.length, `${faction} ${JSON.stringify(item)}: ${summary}`).toBeLessThanOrEqual(80);
    expect(summary, `${faction} ${JSON.stringify(item)} contains a line break`).not.toMatch(/[\r\n]/);
  }
});

it('More never repeats the summary: the expanded text adds something', () => {
  const norm = (s: string) => s.toLowerCase().replace(/[.\s]+$/, '').trim();
  const repeats = new Set<string>();
  for (const item of items) for (const faction of FACTIONS) {
    const summary = itemSummary(item, faction);
    const more = moreText(itemDescription(item, faction), summary);
    const lead = more.match(/^(.*?\.)(\s|$)/)?.[1] ?? more;
    if (norm(more) === norm(summary) || norm(lead) === norm(summary)) repeats.add(`${faction} ${JSON.stringify(item)}: ${summary}`);
  }
  expect([...repeats]).toEqual([]);
});

it('moreText drops only a first sentence that restates the card line', () => {
  expect(moreText('+2 Food. Upgrades to the Mill with Feudalism.', '+2 Food')).toBe('Upgrades to the Mill with Feudalism.');
  expect(moreText('+2 Food. Upgrades to the Mill.', '+3 Food')).toBe('+2 Food. Upgrades to the Mill.');
  expect(moreText('+1.5 Gold', '+1.5 Gold')).toBe('+1.5 Gold');
});
