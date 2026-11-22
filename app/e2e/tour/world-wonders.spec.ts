import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { contextFromViews, formatEvents } from '../../src/format';
import { empireOfPlayer } from '../../src/empires';
import type { Event } from '../../src/types';
import { SIZES, loadFixture, primaries, selectThing, shot, type GameWindow } from './helpers';

for (const size of SIZES) test.describe(`world wonder race ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the Pyramids card names known builders and the lost-race report names the refund', async ({ page }) => {
    const fixture = improvementCity({ techs: ['Masonry'] });
    await loadFixture(page, coreFixture(fixture), 12);
    const city = fixture.cities[0];
    const view = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const v = s.views![s.seat];
      const rival = v.players.find((p) => p.id !== v.me && v.diplomacy.met?.includes(p.team))!;
      const views = [...s.views!];
      views[s.seat] = { ...v, great_works: [
        ...(v.great_works ?? []).filter((w) => w.player !== rival.id),
        { player: rival.id, completed: null, building: 'Pyramids', city: null, progress: 140, cost: 240, production_per_turn: 20, turns: null, blocker: null },
      ] };
      s.set({ views });
      return views[s.seat];
    });
    const rival = view.players.find((p) => p.id !== view.me && view.diplomacy.met?.includes(p.team))!;
    await selectThing(page, 'city', city.id);
    const card = page.locator('.cp-great-works .wonder-site', { hasText: 'Pyramids' }).first();
    // The rival's public pace: (240 − 140) / 20 = 5 turns.
    await expect(card).toContainText(`also building: ${empireOfPlayer(view, rival.id)} (about 5 turns)`);
    await expect(card).toContainText('one in the world');
    await card.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'pyramids-race-card');

    const event: Event = { type: 'WonderRaceLost', city: city.id, owner: view.me, winner: rival.id, building: 'Pyramids', production: 140, gold: 70, active: true };
    const report = { turn: view.turn, hash: '', events: [[event]], moments: [[]], alarms: [[]], lines: [formatEvents([event], contextFromViews(view, view))] };
    await page.evaluate((value) => (window as unknown as GameWindow).legatum.getState().set({ report: value, reportOpen: true, alarmSeen: [] }), report);
    const opened = page.getByRole('complementary', { name: 'Turn report' });
    await expect(opened).toContainText(`${empireOfPlayer(view, rival.id)} completed the Pyramids first: ${city.name} stopped building it.`);
    await expect(opened).toContainText(`You got 70 Gold back, half the 140 Production ${city.name} had put into it.`);
    await shot(page, size, 'pyramids-race-report');
    // The same event as a held moment card (the lost-race card with the next-step line).
    await page.evaluate(({ events }) => {
      const st = (window as unknown as GameWindow).legatum.getState();
      const v = st.views![st.seat];
      st.set({ report: { turn: v.turn, hash: 'wrl', events: [events], lines: [[]] }, reportOpen: false });
    }, { events: [event] });
    await expect(page.locator('.moment-card', { hasText: 'Wonder race lost' })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    // The card owns Return: the turn dock steps back and drops its key chip (one primary on screen).
    expect(await primaries(page), 'one primary with a moment card up').toBe(1);
    await expect(page.locator('.seatbar .cta .kbd', { hasText: /^Return$/ })).toHaveCount(0);
    await shot(page, size, 'wonder-race-lost-moment-card');
  });

  test('a city that lost the race is a choose-production decision that names the race (Thun, 8.9.26 T92)', async ({ page }) => {
    const fixture = improvementCity({ techs: ['Masonry'] });
    await loadFixture(page, coreFixture(fixture), 12);
    const city = fixture.cities[0];
    const rivalName = await page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const v = s.views![s.seat];
      const rival = v.players.find((p) => p.id !== v.me && v.diplomacy.met?.includes(p.team))!;
      const views = [...s.views!];
      // Units with nothing to do would outrank the decision on the CTA; this shot is about the city.
      views[s.seat] = { ...v, units: v.units.filter((u) => u.owner !== v.me), cities: v.cities.map((c) => c.id !== id || !c.detail ? c : { ...c, detail: { ...c.detail,
        producing: null, production_stored: 0, idle_production_waste: 0,
        production_queue: [{ kind: 'Building', building: 'Walls' }],
        wonder_lost: { turn: v.turn - 1, winner: rival.id, building: 'Pyramids', production: 140, gold: 70 } } }) };
      s.set({ views });
      return { id: rival.id, view: views[s.seat] };
    }, city.id);
    const rival = empireOfPlayer(rivalName.view, rivalName.id);
    const cta = page.locator('.cta.cta-production');
    await expect(cta).toContainText('Choose production');
    await expect(cta).toContainText(`${city.name} · Pyramids lost to ${rival}`);
    await selectThing(page, 'city', city.id);
    await expect(page.getByTestId('wonder-lost-tag')).toHaveText(`lost to ${rival}`);
    const warning = page.locator('.production-warning');
    await expect(warning).toContainText(`${rival} completed the Pyramids first: ${city.name} stopped building it.`);
    await expect(warning).toContainText(`Choose what ${city.name} builds next.`);
    await expect(warning).not.toContainText("can't start");
    const queued = page.getByTestId('wonder-lost-queue');
    await expect(queued).toContainText('Walls · waits for your choice');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'wonder-race-lost-decision');
    // One click keeps the queue he had planned: the decision is answered.
    await queued.getByRole('button', { name: 'Resume Walls' }).click();
    await expect(page.getByTestId('wonder-lost-tag')).toHaveCount(0);
    await expect(cta).toHaveCount(0);
  });

  test('a city building a wonder a met rival is also building warns with both paces', async ({ page }) => {
    const fixture = improvementCity({ techs: ['Masonry'] });
    await loadFixture(page, coreFixture(fixture), 12);
    const city = fixture.cities[0];
    const rival = await page.evaluate((id) => {
      const s = (window as unknown as GameWindow).legatum.getState();
      const v = s.views![s.seat];
      const rival = v.players.find((p) => p.id !== v.me && v.diplomacy.met?.includes(p.team))!;
      const views = [...s.views!];
      views[s.seat] = { ...v, units: v.units.filter((u) => u.owner !== v.me),
        great_works: [...(v.great_works ?? []).filter((w) => w.player !== rival.id),
          { player: rival.id, completed: null, building: 'Pyramids', city: null, progress: 200, cost: 240, production_per_turn: 20, turns: null, blocker: null }],
        cities: v.cities.map((c) => c.id !== id || !c.detail ? c : { ...c, detail: { ...c.detail,
          producing: { kind: 'Wonder', building: 'Pyramids', tile: { q: c.pos.q + 1, r: c.pos.r } }, production_stall: null } }) };
      s.set({ views });
      return { id: rival.id, view: views[s.seat] };
    }, city.id);
    await selectThing(page, 'city', city.id);
    await expect(page.locator('.production-warning')).toContainText(`${empireOfPlayer(rival.view, rival.id)} is also building the Pyramids and should finish in about 2 turns`);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'pyramids-race-warning');
  });
});
