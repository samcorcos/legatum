import { expect, it } from 'vitest';
import { hexEq, neighbors, tileIndex } from '../hex';
import { siegeCover, siegeTier } from '../combat';
import { wallsLine } from '../ui/forecastWarnings';
import { coreFixture } from './coreFixture';
import { spawnUnit } from './fixtureEngine';
import { settlementScores } from '../settlement';
import { viewFromFixture } from './fixtureEngine';
import { settleLensFixture } from './settleLensFixture';
import { siegeUnitsFixture } from './siegeUnitsFixture';

it('stages three core-ranked settlement sites and two visible wonders', () => {
  const scene = settleLensFixture();
  const view = viewFromFixture(scene.fixture, 0);
  const settler = view.units.find((u) => u.id === scene.settler)!;
  const scores = settlementScores(view, settler, []);
  expect(scores.map((s) => s.rank).filter((rank) => rank !== null).sort()).toEqual([1, 2, 3]);
  expect(scores.find((s) => hexEq(s.pos, scene.rich))?.band).toBe('Exceptional');
  expect(view.tiles.filter((t) => t.wonder).map((t) => t.wonder)).toEqual(expect.arrayContaining(['Uluru', 'MountFuji']));
  expect(view.tiles[tileIndex(scene.rich, view.width, view.height)].explored).toBe(true);
});

it('keeps the staged Siege flags in the core view', () => {
  const scene = siegeUnitsFixture();
  const view = scene.client.view(0);
  expect(view.units.find((u) => u.id === scene.spear)?.siege).toBe(true);
  expect(view.units.find((u) => u.id === scene.archer)?.siege).toBe(true);
  expect(view.units.find((u) => u.id === scene.bare)?.siege).not.toBe(true);
});

it('an Industrial unit covers Walls IV with built-in Siege', () => {
  const scene = siegeUnitsFixture('Metallurgy');
  const f = scene.client.debugState();
  f.cities[1].wallsLevel = 4;
  const rifle = spawnUnit(f, 0, 'Rifleman', neighbors(scene.cityAt).find((h) => !f.units.some((u) => hexEq(u.pos, h)) && f.terrain[tileIndex(h, f.width, f.height)] !== 'Ocean')!, false);
  scene.client.load(coreFixture(f));
  const view = scene.client.view(0);
  const card = view.units.find((u) => u.id === rifle.id)!;
  expect(card.siege).not.toBe(true);
  expect(card.siege_offer ?? null).toBeNull();
  expect(siegeCover('Rifleman', false, view.research.known)).toBe(4);
  const forecast = scene.client.forecast(0, rifle.id, scene.cityAt, 'Melee')!;
  expect(forecast.walls).toEqual({ tier: 4, covered: 4, percent: 100, built_in: true });
  expect(wallsLine(forecast, 'Athens', 'Rifleman', siegeTier(view.research.known))).toEqual({ text: 'Walls IV: built-in Siege covers up to level IV, so full damage to Athens.', tone: 'good' });
  // A bare Spearman beside it is still halved.
  const bare = scene.client.forecast(0, scene.bare, scene.cityAt, 'Melee');
  if (bare?.walls) expect(bare.walls.built_in).toBeUndefined();
});
