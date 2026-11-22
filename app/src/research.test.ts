import { describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { diffusionText, firstBuildForecast, firstScienceForecast, researchTurns } from './research';

it('explains capital Science on turn 1 and after capital loss or resistance', async () => {
  const client = new FixtureClient();
  const setup = await client.standardSetup('17', 'India', 'Byzantium', 'A', 'B');
  setup.players = setup.players.map((p) => ({ ...p, is_human: true }));
  await client.newGame(setup);
  expect(firstScienceForecast(client.view(0))).toBeNull();
  const city = client.debugState().cities.find((c) => c.owner === 0)!;
  city.isCapital = false;
  expect(firstScienceForecast(client.view(0))).toContain('the next city you found becomes your capital');
  expect(firstScienceForecast(client.view(0))).toContain('adding 1 Science per turn afterward');
  city.population = 3;
  city.resistance = 2;
  const explanation = firstScienceForecast(client.view(0));
  expect(explanation).toContain('Cities in resistance produce nothing until resistance ends');
  expect(explanation).not.toContain('needs more Food');
});

describe('research forecasts', () => {
  it('rounds up at current Science', () => {
    expect(researchTurns(35, 10, 6)).toBe(5);
    expect(researchTurns(35, 11, 6)).toBe(4);
    expect(researchTurns(20, 30, 0)).toBe(1);
    expect(researchTurns(20, 0, 0)).toBeNull();
    expect(researchTurns(20, 0, -1)).toBeNull();
  });
  it('separates research and construction, excludes resisting and already built cities', async () => {
    const c = new FixtureClient();
    await c.newGame(await c.standardSetup('17', 'India', 'Byzantium', 'A', 'B'));
    c.endTurn({});
    const v = c.view(0);
    const city = v.cities.find((x) => x.owner === 0)!;
    const unlock = v.research.unlocks.find((x) => x.tech === 'BronzeWorking')!;
    // Exercise the compatibility fallback for views saved before core city forecasts.
    unlock.cities = undefined;
    city.detail!.yields.total.production = 7;
    const buildTurns = Math.ceil(unlock.cost / 7); // the Workshop's cost
    expect(firstBuildForecast(v, unlock, 4)).toEqual({ city: city.name, buildTurns, totalTurns: 4 + buildTurns });
    city.detail!.resistance = 1;
    expect(firstBuildForecast(v, unlock, 4)).toBeNull();
    city.detail!.resistance = 0;
    city.detail!.buildings.push('Workshop');
    expect(firstBuildForecast(v, unlock, 4)).toBeNull();
    expect(firstBuildForecast(v, unlock, null)).toBeNull();
  });
});


describe('tech diffusion text', () => {
  it('states the discount and how many empires know the tech', () => {
    const cost = (empires: number, discount: number) => ({ total: 120 - discount, parts: [['Listed cost', 120], [`Tech diffusion (${empires} empires know it)`, -discount]] as [string, number][] });
    expect(diffusionText(cost(2, 24))).toBe('−20%: 2 empires know it');
    expect(diffusionText(cost(1, 12))).toBe('−10%: 1 empire knows it');
    expect(diffusionText(cost(2, 24), true)).toBe('−20%');
    expect(diffusionText(cost(0, 0))).toBeNull();
    expect(diffusionText(undefined)).toBeNull();
  });
});
