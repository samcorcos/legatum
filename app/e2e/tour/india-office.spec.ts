// Screen tour, India's Revenue Office (rules 65): the faction sheet, the Guide, the Office's build card against
// the Market, and the Gold breakdown's single Revenue Offices offset in an 8-city India game.
// Run alone with `npx playwright test e2e/tour/india-office`. Shots land in test-results/screen-tour/<size>/india-office/.
import { test, expect } from '../fixtures';
import { fromOffset } from '../../src/hex';
import { coreFixture } from '../../src/test/coreFixture';
import { FixtureClient, createState, makeStandardSetup, refreshVision, spawnUnit } from '../../src/test/fixtureEngine';
import { SIZES, loadFixture, parkMouse, selectThing, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

/** Eight Indian cities, three of them with a Revenue Office, and one Byzantine rival far away. */
function eightCities() {
  const setup = makeStandardSetup('99327', 'India', 'Byzantium', 'Explorer', 'Rival');
  setup.width = 30; setup.height = 14;
  setup.players = setup.players.slice(0, 2).map((p, i) => ({ ...p, team: i, is_human: i === 0, faction: (['India', 'Byzantium'] as const)[i] }));
  const s = createState(setup);
  s.units = []; s.cities = []; s.nextUnitId = 0; s.nextCityId = 0;
  s.terrain.fill('Ocean'); s.ruin.fill(0); s.wonder = undefined; s.owner.fill(-1); s.tileCity.fill(-1);
  for (let row = 1; row <= 12; row++) for (let col = 1; col <= 28; col++) s.terrain[row * s.width + col] = 'Plains';
  for (const p of s.players) { p.explored.fill(1); p.gold = 300; }
  const sites: [number, number, number][] = [[0, 3, 3], [0, 7, 3], [0, 11, 3], [0, 15, 3], [0, 3, 8], [0, 7, 8], [0, 11, 8], [0, 15, 8], [1, 25, 5]];
  for (const [owner, col, row] of sites) spawnUnit(s, owner, 'Settler', fromOffset(col, row), false);
  refreshVision(s);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn(Object.fromEntries([0, 1].map((p) => [p, s.units.filter((u) => u.owner === p).map((u) => ({ type: 'FoundCity' as const, unit: u.id }))])));
  const f = client.debugState();
  for (const c of f.cities) c.population = 4;
  f.players[0].techs = [...new Set([...(f.players[0].techs ?? []), 'BronzeWorking', 'Currency'])] as typeof f.players[0]['techs'];
  const mine = f.cities.filter((c) => c.owner === 0);
  const capital = mine.find((c) => c.isCapital) ?? mine[0];
  const others = mine.filter((c) => c !== capital);
  for (const c of [capital, others[0], others[1]]) c.buildings = [...(c.buildings ?? []), 'Market'];
  const noOffice = others[2];
  noOffice.producing = null; noOffice.productionQueue = [];
  refreshVision(f);
  return { fixture: f, noOffice: noOffice.id };
}

for (const size of SIZES) test.describe(`india office ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('faction sheet and Guide', async ({ page }) => {
    test.setTimeout(120_000);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await expect(stage).toBeVisible();
    await stage.getByRole('option', { name: /India/ }).click();
    const sheet = stage.getByTestId('faction-sheet');
    await expect(sheet).toContainText('Revenue Office');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-india-sheet');
    const piece = sheet.getByText('Revenue Office', { exact: false }).last();
    await piece.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-india-sheet-office');
    await sheet.evaluate((el) => { const s = el.closest('[class*=scroll], .fs-sheet, dialog') as HTMLElement | null; (s ?? el).scrollTop = 99999; el.scrollTop = 99999; });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03-india-sheet-bottom');
    await page.keyboard.press('Escape');

    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await expect(guide).toBeVisible();
    for (const [chapter, text, name] of [
      [/Cities & research/, 'An Indian city with a Revenue Office pays no city upkeep', '04-guide-city-upkeep'],
      [/The victory race/, 'The Imperial Diwan (India)', '05-guide-diwan'],
      [/The peoples/, 'Imperial Diwan', '06-guide-peoples-india'],
    ] as const) {
      await guide.getByRole('button', { name: chapter }).click();
      const p = guide.locator('.guide-page p', { hasText: text }).first();
      if (await p.count()) {
        await p.scrollIntoViewIfNeeded();
        await p.evaluate((el) => el.scrollIntoView({ block: 'center' }));
      }
      await page.mouse.move(5, 5);
      await shot(page, size, name);
    }
    const vs = guide.locator('.guide-piece-vs', { hasText: 'Revenue Office' });
    for (const chapter of [/The peoples/, /Cities & research/, /The victory race/]) {
      await guide.getByRole('button', { name: chapter }).click();
      if (await vs.count()) { await vs.first().evaluate((el) => el.scrollIntoView({ block: 'center' })); await page.mouse.move(5, 5); await shot(page, size, '07-guide-piece-vs'); break; }
    }
  });

  test('Office build card and Gold breakdown in an 8-city game', async ({ page }) => {
    const setup = eightCities();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.locator('.gold-stat').click();
    const tip = page.locator('.tip-bubble').filter({ hasText: 'Revenue Offices (' }).first();
    await expect(tip).toBeVisible();
    await page.waitForTimeout(450);
    await shot(page, size, '08-gold-breakdown');
    await page.keyboard.press('Escape');
    await parkMouse(page);

    await selectThing(page, 'city', setup.noOffice);
    const card = page.locator('.city-panel .production-choice[data-item="b:Market"]');
    await card.scrollIntoViewIfNeeded();
    await expect(card).toBeVisible();
    await expect(card.getByTestId('office-city-upkeep')).toHaveText('City upkeep: saves 2 Gold a turn');
    await parkMouse(page);
    await shot(page, size, '09-office-build-card');
    await card.locator('.tip-trigger, .prod-name, button').first().hover();
    await page.waitForTimeout(600);
    await shot(page, size, '10-office-build-card-hover');
    await parkMouse(page);
    await card.getByText('More', { exact: false }).first().click();
    await page.waitForTimeout(300);
    await parkMouse(page);
    await shot(page, size, '11-office-build-card-more');
  });
});
