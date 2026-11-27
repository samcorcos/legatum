import { expect, it } from 'vitest';
import { captureUpkeepForecast, cityUpkeepForecast } from './cityUpkeepForecast';
import type { PlayerView } from '../types';

const view = (cities: number, added: number) => ({
  me: 0, added_city_upkeep: added,
  cities: Array.from({ length: cities }, (_, id) => ({ id, owner: 0 })),
}) as PlayerView;

it('quotes the seventh and eighth city from the core view', () => {
  expect(cityUpkeepForecast(view(6, 7))).toBe('City upkeep: −7 Gold a turn (7th city)');
  expect(cityUpkeepForecast(view(7, 9))).toBe('City upkeep: −9 Gold a turn (8th city)');
});

it('stays hidden while the next city is free', () => {
  expect(cityUpkeepForecast(view(5, 0))).toBeNull();
  expect(cityUpkeepForecast(view(8, 0))).toBeNull();
});

it('says the capture cost applies only if the city is taken', () => {
  expect(captureUpkeepForecast(view(6, 7))).toBe('If you take it: −7 Gold a turn city upkeep (7th city)');
  expect(captureUpkeepForecast(view(5, 0))).toBeNull();
});
