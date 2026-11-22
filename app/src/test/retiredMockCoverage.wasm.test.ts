import { expect, it } from 'vitest';
import { createState, makeStandardSetup, viewFromFixture } from './fixtureEngine';
import { researchFixture } from './researchFixture';

it('WASM map generation pins resource and wonder placement for a seed', () => {
  const setup = makeStandardSetup('531204', 'India', 'Byzantium', 'Aiko', 'Rival');
  setup.width = 24;
  setup.height = 16;
  setup.players = setup.players.slice(0, 2);
  const placements = () => {
    const state = createState(setup);
    return {
      resources: state.resource?.flatMap((resource, index) => resource ? [[index, resource]] : []) ?? [],
      wonders: state.wonder?.flatMap((wonder, index) => wonder ? [[index, wonder]] : []) ?? [],
    };
  };
  const first = placements();
  expect(placements()).toEqual(first);
  expect(first).toMatchInlineSnapshot(`
    {
      "resources": [
        [
          78,
          "Stone",
        ],
        [
          84,
          "Coal",
        ],
        [
          85,
          "Silicon",
        ],
        [
          101,
          "Wheat",
        ],
        [
          105,
          "Iron",
        ],
        [
          109,
          "Oil",
        ],
        [
          111,
          "Coal",
        ],
        [
          114,
          "Stone",
        ],
        [
          126,
          "Stone",
        ],
        [
          128,
          "Stone",
        ],
        [
          130,
          "Copper",
        ],
        [
          133,
          "Niter",
        ],
        [
          136,
          "Stone",
        ],
        [
          139,
          "Cattle",
        ],
        [
          158,
          "Silicon",
        ],
        [
          165,
          "Stone",
        ],
        [
          181,
          "Oil",
        ],
        [
          197,
          "Wheat",
        ],
        [
          203,
          "Wheat",
        ],
        [
          205,
          "Niter",
        ],
        [
          207,
          "Wheat",
        ],
        [
          230,
          "Wheat",
        ],
        [
          234,
          "Copper",
        ],
        [
          237,
          "Wheat",
        ],
        [
          276,
          "Niter",
        ],
        [
          277,
          "Iron",
        ],
        [
          285,
          "Cattle",
        ],
        [
          299,
          "Coal",
        ],
        [
          300,
          "Oil",
        ],
        [
          301,
          "Silicon",
        ],
        [
          302,
          "Copper",
        ],
        [
          306,
          "Iron",
        ],
      ],
      "wonders": [
        [
          81,
          "GiantsCauseway",
        ],
        [
          270,
          "GrandCanyon",
        ],
      ],
    }
  `);
});

it('WASM derives Growth focus only while a city needs population two for a Settler', () => {
  const state = researchFixture('India');
  const city = state.cities.find((c) => c.owner === 0)!;
  city.producing = { kind: 'Unit', unit: 'Settler' };
  city.population = 1;
  const focus = () => viewFromFixture(state, 0).cities.find((c) => c.id === city.id)!.detail!.yields.focus;
  expect(focus()).toBe('Growth');
  city.population = 2;
  expect(focus()).toBe('Balanced');
});
