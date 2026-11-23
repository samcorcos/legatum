// UI review of claude/celts-barb-peace and claude/lobby-six (merge-ui-1103): barbarians beside a Celtic city raise no alarm,
// call to action, wounded warning or danger mark, while a non-Celtic city still alarms; the Celtic Auto-fire tooltip and
// How to Play's Celts paragraph; the Single Player lobby's default of 6 empires and its size switches.
// Run alone: `npx playwright test e2e/tour/celts-barb-peace`.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { BARBARIAN, newBarbarians, refreshVision, spawnUnit } from '../../src/test/fixtureEngine';
import { neighbors, tileIndex, hexEq } from '../../src/hex';
import type { Faction } from '../../src/types';
import { SIZES, aim, closeReportIfOpen, dismissCelebrations, hexPoint, loadFixture, openReport, posOf, primaries, selectThing, shot, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

/** The siege fixture with its three attackers turned barbarian; `faction` is you. `rival` adds one Byzantine Spearman at War beside the city. */
function barbarianGate(faction: Faction, { rival = false } = {}) {
  const setup = siegeFixture({ fresh: true });
  const f = setup.fixture;
  f.players[0].faction = faction;
  f.setup.players[0].faction = faction;
  f.barbarians = f.barbarians ?? newBarbarians();
  for (const id of setup.attackers) f.units.find((u) => u.id === id)!.owner = BARBARIAN;
  const own = f.cities.find((c) => c.id === setup.own)!;
  own.buildings = ['Walls'];
  const free = (hs: ReturnType<typeof neighbors>) => hs.filter((h) => f.terrain[tileIndex(h, f.width, f.height)] === 'Plains'
    && !f.units.some((u) => hexEq(u.pos, h)) && !f.cities.some((c) => hexEq(c.pos, h)));
  // A ranged unit in the city (Auto-fire is offered), and a wounded Spearman outside beside a barbarian.
  const archer = spawnUnit(f, 0, 'Archer', own.pos, false);
  const barbPos = f.units.find((u) => u.id === setup.attackers[0])!.pos;
  const woundedAt = free(neighbors(barbPos)).find((h) => !hexEq(h, own.pos))!;
  const wounded = spawnUnit(f, 0, 'Spearman', woundedAt, false);
  wounded.hp = 30;
  let rivalId: number | undefined;
  if (rival) rivalId = spawnUnit(f, 1, 'Spearman', free(neighbors(own.pos))[0], false).id;
  refreshVision(f);
  return { ...setup, archer: archer.id, wounded: wounded.id, rival: rivalId };
}

for (const size of SIZES) test.describe(`celts barbarian peace ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  for (const [faction, label] of [['Celts', 'celts'], ['India', 'india-control']] as const) {
    test(`${label}: barbarians beside the city`, async ({ page }) => {
      test.setTimeout(150_000);
      const setup = barbarianGate(faction);
      await loadFixture(page, coreFixture(setup.fixture), 2);
      const at = await posOf(page, 'city', setup.own);
      await page.evaluate((hex) => { const s = (window as unknown as GameWindow).legatum.getState(); s.set({ techOpen: false }); s.select(null); s.jumpTo(hex); }, at);
      await hexPoint(page, at);
      await page.mouse.move(5, size.height / 2);
      const siege = page.locator(`.seatbar .siege-card[data-city="${setup.own}"]`);
      if (faction === 'Celts') await expect.soft(siege, 'no siege card for the Celts').toHaveCount(0);
      else await expect.soft(siege).toContainText('under attack');
      const cta = page.locator('.seatbar .cta .cta-label');
      if (faction === 'Celts') await expect.soft(cta, 'no siege call to action for the Celts').not.toHaveText('City under attack');
      else await expect.soft(cta).toHaveText('City under attack');
      expect.soft(await primaries(page), 'one primary action').toBe(1);
      if (faction === 'Celts') await shot(page, size, 'cb-celts-own-banner-above-barbarians');
      await shot(page, size, `cb-${label}-01-this-turn`);
      await page.evaluate(() => { const m = (window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap; m.camera.zoom = 0.5; m.redraw(); });
      await hexPoint(page, at);
      await shot(page, size, `cb-${label}-02-far-zoom`);
      await page.evaluate(() => { const m = (window as unknown as { legatumMap: { camera: { zoom: number }; redraw(): void } }).legatumMap; m.camera.zoom = 1.25; m.redraw(); });
      // The Threat lens: barbarians draw no danger for the Celts.
      await page.getByRole('radio', { name: 'Threat', exact: true }).first().click();
      if (faction === 'Celts') await expect(page.locator('.lens-empty')).toHaveText("Nothing in sight can attack you next turn. Barbarians don't attack you.");
      await page.mouse.move(5, size.height / 2);
      await page.waitForTimeout(400);
      await shot(page, size, `cb-${label}-02a-threat-lens`);
      await page.getByRole('radio', { name: 'Default', exact: true }).first().click();
      // Hover a barbarian: does its card say it never attacks the Celts?
      const barb = await posOf(page, 'unit', setup.attackers[2]);
      await aim(page, barb, page.locator('.tilecard'));
      if (faction === 'Celts') {
        const neutral = await page.locator('.tilecard-unit .icon').first().evaluate((el) => (el as HTMLElement).style.backgroundColor.toLowerCase());
        expect(neutral).toBe('currentcolor');
      }
      await shot(page, size, `cb-${label}-02b-barbarian-card`);
      await page.mouse.move(5, size.height / 2);
      // The wounded Spearman beside a barbarian: no "wounded next to enemies" warning for the Celts.
      await selectThing(page, 'unit', setup.wounded);
      await page.waitForTimeout(500);
      if (faction === 'Celts') await expect(page.getByTestId('strength-key')).toContainText("barbarians don't attack you");
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `cb-${label}-03-wounded-unit`);
      // The ranged unit in the city: hover Auto-fire for its tooltip.
      await selectThing(page, 'unit', setup.archer);
      await page.waitForTimeout(500);
      const auto = page.getByRole('button', { name: /Auto-fire from city/ });
      await auto.scrollIntoViewIfNeeded();
      await auto.hover();
      await page.waitForTimeout(900);
      if (faction === 'Celts') await expect.soft(page.locator('[role=tooltip]:visible')).toContainText('never shoots barbarians');
      await shot(page, size, `cb-${label}-04-autofire-tooltip`);
      await page.mouse.move(5, size.height / 2);
      // Plot a move with the wounded Spearman past the barbarians: the route preview's danger line.
      await selectThing(page, 'unit', setup.wounded);
      const target = await page.evaluate(({ id }) => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
        const u = v.units.find((x) => x.id === id)!;
        return { q: u.pos.q + 2, r: u.pos.r };
      }, { id: setup.wounded });
      const p = await hexPoint(page, target);
      await page.mouse.move(p.x, p.y);
      await page.waitForTimeout(700);
      const caught = page.getByText('Enemies can catch this unit leaving.');
      if (faction === 'Celts') await expect.soft(caught, 'barbarians are no route danger to the Celts').toHaveCount(0);
      else await expect.soft(caught).toBeVisible();
      await shot(page, size, `cb-${label}-05-route-hover`);
      await page.mouse.move(5, size.height / 2);
      await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));
      // End the turn: the barbarians strike India but never the Celts; the report says which.
      await endPlanning(page);
      await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 60_000 });
      await dismissCelebrations(page);
      await page.waitForTimeout(3500);
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `cb-${label}-06-next-turn`);
      const report = await openReport(page).catch(() => null);
      if (report) {
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, `cb-${label}-07-report`);
        await report.locator('.report-group[data-category="combat"]').first().scrollIntoViewIfNeeded().catch(() => undefined);
        await page.mouse.move(5, size.height / 2);
        await shot(page, size, `cb-${label}-08-report-combat`);
        await closeReportIfOpen(page);
      }
    });
  }

  test('celts: barbarians and one rival Soldier beside the city', async ({ page }) => {
    test.setTimeout(90_000);
    const setup = barbarianGate('Celts', { rival: true });
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const at = await posOf(page, 'city', setup.own);
    await page.evaluate((hex) => { const s = (window as unknown as GameWindow).legatum.getState(); s.set({ techOpen: false }); s.select(null); s.jumpTo(hex); }, at);
    await hexPoint(page, at);
    await page.mouse.move(5, size.height / 2);
    await expect.soft(page.locator(`.seatbar .siege-card[data-city="${setup.own}"]`)).toContainText('under attack');
    await shot(page, size, 'cb-celts-mixed-01-this-turn');
  });

  test('How to Play: the Celts paragraph', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /Simultaneous combat/ }).click();
    const h = guide.locator('.guide-page').getByRole('heading', { name: /^The Celts/ }).first();
    await h.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'cb-guide-celts');
    await guide.getByRole('button', { name: /Begin your campaign/ }).first().click().catch(() => undefined);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'cb-guide-start');
  });

  test('lobby: default 6, size switches, add and remove', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-00-title');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await expect(page.getByRole('heading', { name: 'Single Player', exact: true })).toBeVisible();
    const rows = page.locator('.seat-row:not(.head)');
    await expect.soft(rows).toHaveCount(6);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-01-default');
    await page.locator('.add-seat').scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-02-default-bottom');
    const mapSize = page.getByRole('radiogroup', { name: 'Map size', exact: true });
    await mapSize.getByRole('radio', { name: /^Small/ }).click();
    await expect.soft(rows).toHaveCount(4);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-03-small');
    await mapSize.getByRole('radio', { name: /^Standard/ }).click();
    await expect.soft(rows).toHaveCount(6);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-04-small-to-standard');
    await mapSize.getByRole('radio', { name: /^Large/ }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-05-large');
    await mapSize.getByRole('radio', { name: /^Standard/ }).click();
    await expect(rows).toHaveCount(6);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-06-large-to-standard');
    // Remove seats down to 4 on Standard: the "maps suit" note appears.
    while (await rows.count() > 4) await page.getByRole('button', { name: `Remove seat ${await rows.count()}`, exact: true }).click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-07-standard-four');
    await page.locator('.lobby-go').scrollIntoViewIfNeeded();
    await shot(page, size, 'lb-08-standard-four-dock');
    await page.locator('.add-seat').click();
    await page.locator('.add-seat').scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'lb-09-added-back');
  });


});
