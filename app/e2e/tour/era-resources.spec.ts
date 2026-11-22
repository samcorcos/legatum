// Screen tour, strategic supply from the Classical era on: the era's supply chip and
// the missing-resource alert, the Resources overlay's "You lack it" mark, the tile card's holder line, a Niter-blocked
// upgrade and build card, every strategic resource at far and
// close zoom, and the Guide's era table. Run it alone with `npx playwright test e2e/tour/era-resources`.
// Shots land in test-results/screen-tour/<size>/era-resources/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { improvementCity } from '../../src/test/improvementsFixture';
import { eraTechs } from '../../src/data';
import { fromOffset, ring } from '../../src/hex';
import { idxOf } from '../../src/test/fixtureEngine';
import type { MState } from '../../src/test/fixtureState';
import { spawnUnit } from '../../src/test/fixtureEngine';
import type { Event, Hex, Resource, Terrain, TechId } from '../../src/types';
import { overlayBox } from '../overlayHelpers';
import { SIZES, expectReasonsFit, hexPoint, loadFixture, selectThing, shot, stageHallReport, type GameWindow } from './helpers';

test.describe.configure({ mode: 'parallel' });

const known = (eras: number[], extra: TechId[] = []): TechId[] => [...new Set([...eras.flatMap((e) => eraTechs(e)), ...extra])];

function put(s: MState, h: Hex, terrain: Terrain, resource: Resource) {
  const i = idxOf(s, h);
  s.terrain[i] = terrain; s.resource![i] = resource;
  s.players[0].explored[i] = 1;
  (s.knownOwner[String(s.players[0].team)] ??= new Array(s.terrain.length).fill(-1))[i] = s.owner[i];
}

/**
 * Renaissance (every Ancient to Medieval technology, plus Gunpowder): Iron supplied at home, no Niter. Two
 * Pikemen could upgrade to Musketmen (Niter), so Niter is missing. One Niter deposit lies in Switzerland's land next to
 * its capital, one unclaimed four tiles from ours.
 */
function niterScene() {
  const s = improvementCity({ techs: known([0, 1, 2], ['Gunpowder']) });
  const c = s.cities[0];
  c.buildings.push('Workshop', 'Market');
  // One Pikeman, so the levy has room and the Musketman card is on offer (blocked by Niter, not by the levy).
  s.units = s.units.filter((u) => u.owner !== 0 || u.kind === 'Scout');
  spawnUnit(s, 0, 'Trooper', ring(c.pos, 1)[1], false);
  const foreign = fromOffset(20, 8), free = ring(c.pos, 4)[1];
  put(s, foreign, 'Plains', 'Niter');
  put(s, free, 'Grassland', 'Niter');
  return { s, city: c.id, foreign, free };
}

/** Scroll only the Guide's own scrolling pane (scrollIntoView would also scroll the dialog off its header). */
function scrollPaneTo(el: Element) {
  let sc = el.parentElement;
  while (sc && !(sc.scrollHeight > sc.clientHeight && /auto|scroll/.test(getComputedStyle(sc).overflowY))) sc = sc.parentElement;
  if (sc) sc.scrollTop += el.getBoundingClientRect().top - sc.getBoundingClientRect().top - 8;
}

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the unrevealed era resource is a muted research target', async ({ page }) => {
    const s = improvementCity({ techs: known([0, 1, 2, 3]) });
    await loadFixture(page, coreFixture(s), 12);
    const coal = page.locator('.topbar .supply-coal');
    await expect(coal).toContainText('Coal: revealed by Industrialization');
    await expect(coal).toHaveClass(/supply-unrevealed/);
    await expect(coal).not.toHaveClass(/supply-short|supply-missing/);
    expect(await page.locator('.topbar .research-chip b, .topbar .research-chip-plan > span').evaluateAll((els) => els.every((el) => el.scrollWidth <= el.clientWidth + 1))).toBe(true);
    await coal.hover();
    await expect(page.locator('[role="tooltip"]:not([hidden])')).toContainText('Riflemen, Field Cannons, Cavalry and Factories');
    await shot(page, size, 'e19-coal-research-target');
  });

  test('Iron Working reveals Iron; Copper is bonus; Silicon names the Quartz Mine', async ({ page }) => {
    const s = improvementCity({ techs: [] });
    const iron = ring(s.cities[0].pos, 3)[2];
    const copper = ring(s.cities[0].pos, 2)[1];
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate((h) => { const st = (window as unknown as GameWindow).legatum.getState(); st.set({ techOpen: false }); st.select(null); st.jumpTo(h); }, iron);
    await expect(page.locator('.supply-iron')).toHaveCount(0);
    await shot(page, size, 'e20-before-iron-working');

    s.players[0].techs.push('IronWorking');
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate((h) => { const st = (window as unknown as GameWindow).legatum.getState(); st.set({ techOpen: false }); st.select(null); st.jumpTo(h); }, iron);
    await expect(page.locator('.supply-iron')).toBeVisible();
    const at = await hexPoint(page, iron);
    await page.mouse.move(at.x, at.y);
    await expect(page.locator('.tilecard')).toContainText('Iron');
    await shot(page, size, 'e21-iron-working-reveal');

    const events: Event[] = [
      { type: 'TechResearched', player: 0, tech: 'IronWorking' },
      { type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: s.cities[0].id, sources: 1, supply: 2 },
    ];
    await stageHallReport(page, events);
    await expect(page.locator('.moment-card').first()).toBeVisible();
    if (await page.locator('.moment-card.unlock').isVisible()) await page.keyboard.press('Escape');
    const reveal = page.locator('.moment-card', { hasText: 'Iron revealed' });
    await expect(reveal).toBeVisible();
    await expect(reveal.locator('.moment-lore')).toHaveCount(0);
    await shot(page, size, 'e21b-iron-reveal-moment');
    // This short celebration can dismiss itself while WebKit captures the screenshot.
    if (await reveal.isVisible()) {
      await reveal.locator('.moment-dismiss').click({ timeout: 2000 }).catch(async (error) => {
        if (await reveal.isVisible()) throw error;
      });
    }

    await page.evaluate((h) => (window as unknown as GameWindow).legatum.getState().jumpTo(h), copper);
    const cu = await hexPoint(page, copper);
    await page.mouse.move(cu.x, cu.y);
    await expect(page.locator('.tilecard')).toContainText('Bonus resource');
    await shot(page, size, 'e22-copper-bonus');

    const silicon = ring(s.cities[0].pos, 2)[3];
    put(s, silicon, 'Hills', 'Silicon');
    s.players[0].techs.push('AutonomousSystems');
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate((h) => { const st = (window as unknown as GameWindow).legatum.getState(); st.set({ techOpen: false }); st.select(null); st.jumpTo(h); }, silicon);
    const si = await hexPoint(page, silicon);
    await page.mouse.move(si.x, si.y);
    await expect(page.locator('.tilecard')).toContainText('Quartz Mine');
    await shot(page, size, 'e23-silicon-quartz-mine');
    await selectThing(page, 'city', s.cities[0].id);
    const quartz = page.locator('.city-panel .production-choice[data-item^="i:QuartzMine"]');
    await expect(quartz.first().locator('img[src*="improvement-quartz-mine"]')).toBeVisible();
    await quartz.first().scrollIntoViewIfNeeded();
    await shot(page, size, 'e23b-quartz-mine-card');
  });

  test('Niter and Oil with only future upgrades stay neutral at zero supply', async ({ page }) => {
    const s = improvementCity({ techs: known([0, 1, 2, 3], ['Electricity']) });
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate(() => {
      const game = (window as unknown as GameWindow).legatum;
      const state = game.getState(), views = [...state.views!], view = views[state.seat];
      views[state.seat] = {
        ...view,
        research: { ...view.research, era: 4, known: [] },
        units: view.units.slice(0, 2).map((u, i) => ({ ...u, kind: i ? 'Cavalry' as const : 'Trooper' as const, upgrade_need: i ? 'Oil' as const : 'Niter' as const, upgrade_supply_blocker: `No ${i ? 'Oil' : 'Niter'} to spare` })),
        cities: view.cities.map((c) => ({ ...c, detail: c.detail ? { ...c.detail, options: [] } : c.detail })),
        supply: (view.supply ?? []).map((resource) => ['Niter', 'Oil'].includes(resource.resource)
          ? { ...resource, revealed: true, supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, idle: [], held: [], spare: 0 }
          : resource),
      };
      game.setState({ views, techOpen: false });
    });
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e24-neutral-zero-supply');
    for (const resource of ['Niter', 'Oil']) {
      const chip = page.locator(`.supply-${resource.toLowerCase()}`);
      await expect(chip).toBeVisible();
      await expect(chip).not.toHaveClass(/supply-short/);
      await expect(page.getByTestId(`supply-${resource}`)).toHaveText('0/0');
    }
  });

  test('missing Niter: era chip, alert tooltip, Resources overlay, tile cards, blocked upgrade and build', async ({ page }) => {
    test.setTimeout(150_000);
    const { s, city, foreign, free } = niterScene();
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    const chip = page.locator('.supply-stat.supply-niter');
    await expect(chip).toBeVisible();
    // The era chip keeps its name at every window size, and says it is missing.
    await expect(chip.locator('.supply-name')).toBeVisible();
    await expect(chip.locator('.supply-alert')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e01-era-chip-missing-niter');
    await chip.click();
    const tip = page.locator('.supply-tip');
    await expect(tip).toBeVisible();
    await expect(tip.locator('.bd-title').first()).toContainText('Missing Niter');
    await expect(tip).not.toContainText(/0 of 0/);
    await expect(tip).toContainText('Your gunpowder units and Banks need it.');
    await expect(tip).toContainText(/One unclaimed Niter deposit lies .* of /);
    await expect(tip.getByRole('button', { name: 'Show on map' })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e02-era-chip-missing-niter-tip');
    await page.keyboard.press('Escape');

    // The Resources overlay with the "You lack it" mark, framed on the unclaimed Niter.
    await page.evaluate((h) => (window as unknown as GameWindow).legatum.getState().jumpTo(h), free);
    const chooser = page.getByRole('button', { name: /^Lens: / });
    if (await chooser.isVisible()) await chooser.click();
    await overlayBox(page, 'Resources').click();
    const opener = page.getByRole('button', { name: 'Overlay key' });
    if (await opener.isVisible()) await opener.click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e03-resources-overlay-lack-niter');
    await page.locator('.map-lenses .lens-key-row').first().hover();
    await shot(page, size, 'e03b-resources-key-tip');
    await page.mouse.move(5, size.height / 2);
    if (await opener.isVisible() && (await opener.getAttribute('aria-expanded')) === 'true') await opener.click();
    if (await chooser.isVisible() && (await chooser.getAttribute('aria-expanded')) === 'true') await chooser.click();

    // The tile cards: a Niter deposit you could take, and one in a rival's land.
    // The camera looks a few tiles west of the rival's deposit, clear of the map's east-west seam.
    for (const [name, hex, look] of [['e04-unclaimed-niter-tile-card', free, free], ['e05-foreign-niter-tile-card', foreign, fromOffset(16, 8)]] as const) {
      await page.evaluate((h) => { const st = (window as unknown as GameWindow).legatum.getState(); st.select(null); st.jumpTo(h); }, name.startsWith('e05') ? hex : look);
      const at = await hexPoint(page, hex);
      // The map wraps east-west, so toScreen may name the copy one map-width away: try it and its two neighbours.
      const span = await page.evaluate((h) => { const w = window as unknown as GameWindow; return (w.legatumMap.toScreen({ q: h.q + 1, r: h.r }).x - w.legatumMap.toScreen(h).x) * w.legatum.getState().views![0].width; }, hex);
      const card = page.locator('.tilecard').filter({ hasText: /^Niter/ });
      for (const x of [at.x, at.x - span, at.x + span]) {
        // Aim below the centre: a rival city's name banner covers the top of the tile next to it.
        await page.mouse.move(x - 3, at.y + 5);
        await page.mouse.move(x, at.y + 8);
        await page.waitForTimeout(700);
        if (await card.isVisible()) break;
      }
      await expect(card, `${name}: the deposit card must be captured`).toBeVisible();
      if (name.startsWith('e05')) await expect(card).toContainText(/hold(s)? this deposit\. You have no Niter yet: take it or find another\./);
      await shot(page, size, name);
    }

    // A Pikeman whose upgrade to Musketman needs Niter.
    const pike = await page.evaluate(() => { const st = (window as unknown as GameWindow).legatum.getState(), v = st.views![st.seat]; return v.units.find((u) => u.owner === v.me && u.kind === 'Trooper')!.id; });
    await selectThing(page, 'unit', pike);
    await expectReasonsFit(page);
    await page.getByRole('button', { name: 'Open Niter supply' }).click();
    await expect(page.locator('.supply-tip')).toContainText('Missing Niter');
    await page.keyboard.press('Escape');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e06-pikeman-upgrade-needs-niter');

    // The city's build list: the Musketman card blocked for want of Niter.
    await selectThing(page, 'city', city);
    const musket = page.locator('.production-choice').filter({ hasText: 'Uses 1 Niter' }).first();
    await musket.scrollIntoViewIfNeeded();
    // The Build card is aria-disabled; clicking it opens the supply breakdown.
    await musket.getByRole('button', { name: /^Build / }).click({ force: true });
    await expect(page.locator('.supply-tip')).toContainText('Missing Niter');
    await page.keyboard.press('Escape');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e07-city-niter-build-card');
    await page.locator('.cp-foot').scrollIntoViewIfNeeded();
    await shot(page, size, 'e08-city-building-list-era-rule');
  });


  test('Industrial empire: Iron, Niter, Coal and Oil each have a chip and their own breakdown', async ({ page }) => {
    const s = improvementCity({ techs: known([0, 1, 2, 3], ['Industrialization', 'Electricity']) });
    await loadFixture(page, coreFixture(s), 12);
    await page.evaluate(() => {
      const game = (window as unknown as GameWindow).legatum;
      const state = game.getState(), views = [...state.views!], view = views[state.seat];
      views[state.seat] = { ...view, research: { ...view.research, era: 4 }, supply: [
        { resource: 'Iron', revealed_by: 'IronWorking', revealed: true, supplied: { total: 9, parts: [['Iron near Pylos', 3], ['Iron near Sparta', 6]] }, used: { total: 8, parts: [['Forge in Pylos', 1], ['University in Sparta', 1], ['Pikeman ×6', 6]] }, idle: [], held: ['University in Pylos'], spare: 0 },
        { resource: 'Niter', revealed_by: 'Gunpowder', revealed: true, supplied: { total: 3, parts: [['Niter near Pylos', 3]] }, used: { total: 1, parts: [['Bank in Pylos', 1]] }, idle: [], held: [], spare: 2 },
        { resource: 'Coal', revealed_by: 'Industrialization', revealed: true, supplied: { total: 4, parts: [['Coal near Sparta', 4]] }, used: { total: 5, parts: [['Factory in Sparta', 1], ['Industrial Infantry ×4', 4]] }, idle: ['Industrial Infantry'], held: [], spare: -1 },
        { resource: 'Oil', revealed_by: 'Combustion', revealed: true, supplied: { total: 2, parts: [['Oil near Pylos', 2]] }, used: { total: 1, parts: [['Research Lab in Pylos', 1]] }, idle: [], held: [], spare: 1 },
      ] };
      game.setState({ views, techOpen: false });
    });
    const chips = page.locator('.supply-stat');
    await expect(chips).toHaveCount(4);
    await expect(chips.locator('.supply-resource-icon')).toHaveCount(4);
    await expect(page.locator('.supply-coal')).toHaveClass(/supply-era/);
    await expect(page.locator('.supply-iron')).not.toHaveClass(/supply-short/);
    await expect(page.locator('.supply-coal')).toHaveClass(/supply-short/);
    await expect(page.getByTestId('supply-Iron')).toHaveText('0/9');
    await expect(page.getByTestId('supply-Niter')).toHaveText('2/3');
    await expect(page.getByTestId('supply-Coal')).toHaveText('0/4');
    await expect(page.getByTestId('supply-Oil')).toHaveText('1/2');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'e18-industrial-four-resource-chips');
    for (const resource of ['Iron', 'Niter', 'Coal', 'Oil']) {
      await page.locator(`.supply-${resource.toLowerCase()}`).click();
      const tip = page.locator('.supply-tip');
      await expect(tip).toBeVisible();
      await expect(tip).toContainText(`${resource} supplied`);
      if (resource === 'Iron') await expect(tip).toContainText('1 held for the University in Pylos');
      if (resource === 'Coal') await expect(tip).toContainText('Short by 1');
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, `e19-${resource.toLowerCase()}-supply-breakdown`);
      await page.keyboard.press('Escape');
    }
  });

  test('every strategic resource at far and close zoom', async ({ page }) => {
    test.setTimeout(120_000);
    const s = improvementCity({ techs: known([0, 1, 2, 3], ['Industrialization', 'Electricity']) });
    const c = s.cities[0];
    put(s, ring(c.pos, 2)[4], 'Plains', 'Niter');
    put(s, ring(c.pos, 2)[10], 'Plains', 'Coal');
    put(s, ring(c.pos, 3)[6], 'Desert', 'Oil');
    put(s, ring(c.pos, 3)[14], 'Hills', 'Silicon');
    await loadFixture(page, coreFixture(s), 12);
    const zoom = async (z: number) => {
      await page.evaluate(({ z, pos }) => {
        const w = window as unknown as GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };
        const st = w.legatum.getState(); st.set({ techOpen: false }); st.select(null); st.jumpTo(pos);
        w.legatumMap.camera.zoom = z; w.legatumMap.redraw();
      }, { z, pos: c.pos });
      await page.waitForTimeout(600);
      await page.mouse.move(5, size.height / 2);
    };
    await zoom(0.35);
    await shot(page, size, 'e12-strategic-far-zoom');
    await zoom(1.25);
    await shot(page, size, 'e13-strategic-close-zoom');
    await overlayBox(page, 'Resources').click();
    await zoom(0.35);
    await shot(page, size, 'e14-strategic-far-zoom-overlay');
  });

  test('the Guide: strategic supply begins with Iron', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /Cities & research/ }).click();
    await guide.getByRole('heading', { name: 'Resources: strategic supply begins with Iron' }).evaluate(scrollPaneTo);
    await page.mouse.move(5, 5);
    await shot(page, size, 'e15-guide-era-resources');
    await guide.locator('.guide-era-table').evaluate(scrollPaneTo);
    await shot(page, size, 'e16-guide-era-table');
    await guide.getByRole('heading', { name: /Improvements: waking a resource/ }).evaluate(scrollPaneTo);
    await shot(page, size, 'e17-guide-improvements');
  });
});
