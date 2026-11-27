// Screen tour, the city upkeep forecast: the added Gold a turn for a 7th city, on the Settler build card,
// the Found City action and a city attack forecast. Run alone with `npx playwright test e2e/tour/city-upkeep`.
// Shots land in test-results/screen-tour/<size>/city-upkeep/.
import { test, expect } from '../fixtures';
import { fromOffset } from '../../src/hex';
import { coreFixture } from '../../src/test/coreFixture';
import { FixtureClient, createState, makeStandardSetup, refreshVision, spawnUnit } from '../../src/test/fixtureEngine';
import { SIZES, aim, centred, expectWhole, loadFixture, parkMouse, posOf, selectThing, shot } from './helpers';

/** Six cities of your own and one rival city at War, with a Settler on an open site and a Spearman beside the rival. */
function sixCities() {
  const setup = makeStandardSetup('99124', 'India', 'Byzantium', 'Explorer', 'Rival');
  setup.width = 30; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: (['India', 'Byzantium'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = 'Plains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 500; }
  const sites: [number, number, number][] = [[0, 3, 3], [0, 8, 3], [0, 13, 3], [0, 3, 8], [0, 8, 8], [0, 13, 8], [1, 22, 5]];
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  for (const c of f.cities) c.population = 3;
  for (const r of f.diplomacy!.relations) r.state = 'War';
  const capital = f.cities.find((c) => c.owner === 0 && c.isCapital) ?? f.cities.find((c) => c.owner === 0)!;
  capital.producing = null;
  const rival = f.cities.find((c) => c.owner === 1)!;
  const settler = spawnUnit(f, 0, 'Settler', fromOffset(19, 11), false);
  const spear = spawnUnit(f, 0, 'Spearman', fromOffset(21, 5), false);
  refreshVision(f);
  return { fixture: f, capital: capital.id, rival: rival.id, settler: settler.id, spear: spear.id };
}

for (const size of SIZES) test.describe(`city upkeep ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the 7th city quotes its upkeep in all three places', async ({ page }) => {
    const setup = sixCities();
    await loadFixture(page, coreFixture(setup.fixture), 2);

    await selectThing(page, 'city', setup.capital);
    const card = page.getByTestId('settler-city-upkeep').first();
    await expect(card).toContainText(/City upkeep: −7\s*Gold a turn \(7th city\)/);
    await card.scrollIntoViewIfNeeded();
    await parkMouse(page);
    await shot(page, size, 'settler-card');
    await card.hover();
    await page.locator('.production-choice', { has: card }).locator('.tip-trigger, button').first().hover();
    await page.waitForTimeout(500);
    await shot(page, size, 'settler-card-hover');

    await selectThing(page, 'unit', setup.settler);
    await expect(page.getByTestId('found-city-upkeep')).toContainText(/City upkeep: −7\s*Gold a turn \(7th city\)/);
    await parkMouse(page);
    await shot(page, size, 'found-city');

    const forecast = page.locator('.forecast');
    await selectThing(page, 'unit', setup.spear);
    await centred(page, setup.spear);
    await aim(page, await posOf(page, 'city', setup.rival), forecast);
    await expectWhole(forecast);
    await shot(page, size, 'capture-forecast');
  });
});
