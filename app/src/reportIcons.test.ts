import { expect, it } from 'vitest';
import { iconUrl } from './icons';
import { emblemUrl } from './celebrate/RewardEmblem';
import { CATEGORY_ICON, eventIcon } from './reportIcons';
import { CATEGORY_ORDER } from './format';
import type { Emblem } from './celebrate/moments';
import type { Event } from './types';

/** An icon name resolves to an art file: an icon, a reward emblem, or the treaty star (drawn, not a file). */
const exists = (name: string) => name.startsWith('treaty-') || (name.startsWith('emblem-') ? !!emblemUrl(name.slice(7) as Emblem) : !!iconUrl(name));

it('every report category has an icon that exists', () => {
  expect(Object.keys(CATEGORY_ICON).sort()).toEqual([...CATEGORY_ORDER].sort());
  for (const [category, icon] of Object.entries(CATEGORY_ICON)) expect(exists(icon), `${category}: ${icon}`).toBe(true);
});

it('event icons come from existing art: a capture wears the captured emblem', () => {
  const events: Event[] = [
    { type: 'CityCaptured', city: 1, from: 1, to: 0 },
    { type: 'CityFounded', city: 1, owner: 0, name: 'A', at: { q: 0, r: 0 } } as unknown as Event,
    { type: 'CityGrew', city: 1, owner: 0, population: 2 } as unknown as Event,
    { type: 'TechResearched', player: 0, tech: 'Writing' },
    { type: 'GreatWorkCompleted', player: 0, of: 0, building: 'RelayUplink', vp: 2, again: false },
    { type: 'ItemBought', city: 1, owner: 0, item: { kind: 'Unit', unit: 'Scout' }, gold: 30 },
    { type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Building', building: 'Walls' } },
  ];
  const icons = events.map((e) => eventIcon(e));
  expect(icons[0]).toBe('emblem-captured');
  expect(icons[6]).toBe('building-walls');
  for (const icon of icons) expect(icon && exists(icon), String(icon)).toBe(true);
  expect(eventIcon({ type: 'FactionGoalLost', player: 0, faction: 'Switzerland', vp: 2 })).toBe('ui-trend-down');
  const sack = { type: 'CityRazed', city: 1, from: 1, by: 0, at: { q: 0, r: 0 } } as const;
  expect(eventIcon(sack, undefined, (p) => p === 0)).toBe('emblem-captured');
  expect(eventIcon(sack, undefined, (p) => p === 1)).toBe('ui-alert');
  expect(eventIcon(sack)).toBe('ui-alert');
});
