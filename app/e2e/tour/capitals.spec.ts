// Original capitals are marked for the whole game, even after capture (user, 2026-11-05): the star follows the
// scoring rule (`original_capital`), in the founder's ink, hollow once you yourself have scored it, and no
// replacement capital wears one. The banner ends in a star seal. Run alone: `npx playwright test e2e/tour/capitals`.
// Fixture: `src/test/capitalsFixture.ts`.
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { capitalsFixture } from '../../src/test/capitalsFixture';
import { SIZES, hexPoint, loadFixture, posOf, selectThing, shot, type GameWindow } from './helpers';

type MapWindow = GameWindow & { legatumMap: { camera: { zoom: number }; redraw(): void } };

/** Frame the map at a zoom, centred on a city, with nothing selected. */
async function frame(page: Page, cityId: number, zoom: number) {
  const pos = await posOf(page, 'city', cityId);
  await page.evaluate(({ pos, zoom }) => {
    const w = window as unknown as MapWindow;
    w.legatum.getState().select(null);
    w.legatum.getState().set({ techOpen: false });
    w.legatumMap.camera.zoom = zoom;
    w.legatumMap.redraw();
    w.legatum.getState().jumpTo(pos);
  }, { pos, zoom });
}

/** Hover the lower part of a city's tile (its banner covers the centre) until the tile card shows. */
async function hoverTile(page: Page, cityId: number) {
  const pos = await posOf(page, 'city', cityId);
  const p = await hexPoint(page, pos);
  // Half a hex radius below the centre is still inside the same hex.
  const radius = await page.evaluate(() => 36 * (window as unknown as MapWindow).legatumMap.camera.zoom);
  const at = { x: p.x, y: Math.round(p.y + radius * 0.5) };
  await expect.poll(async () => {
    await page.mouse.move(at.x + 2, at.y + 2);
    await page.mouse.move(at.x, at.y, { steps: 3 });
    return page.locator('.tilecard').isVisible();
  }, { intervals: [300, 600, 1000, 2000] }).toBe(true);
  return page.locator('.tilecard');
}

/** The named banner sits above the hex; far-zoom seals sit at its centre. */
async function hoverBanner(page: Page, cityId: number, far = false) {
  const p = await hexPoint(page, await posOf(page, 'city', cityId));
  const zoom = await page.evaluate(() => (window as unknown as MapWindow).legatumMap.camera.zoom);
  await page.mouse.move(p.x, p.y - (far ? 36 * zoom * 0.1 : 36 * zoom * 0.62 + 11), { steps: 3 });
  return page.getByTestId('banner-tooltip');
}

for (const size of SIZES) test.describe(`original capitals ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('the map marks every state at the default zoom: own, rival-held, captured, scored, teammate-scored, replacement', async ({ page }) => {
    const setup = capitalsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    // The view says exactly what the rules say.
    const flags = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      return v.cities.map((c) => ({ id: c.id, original: !!c.original_capital, founder: c.original_capital_of ?? null, current: c.is_capital, owner: c.owner }));
    });
    const by = (id: number) => flags.find((c) => c.id === id)!;
    expect(by(setup.own)).toMatchObject({ original: true, founder: 0, owner: 0 });
    expect(by(setup.held)).toMatchObject({ original: true, founder: 1, owner: 1 });
    expect(by(setup.captured)).toMatchObject({ original: true, founder: 2, owner: 1, current: false });
    expect(by(setup.replacement)).toMatchObject({ original: false, current: true });
    expect(by(setup.scored)).toMatchObject({ original: true, founder: 3, owner: 0, current: false });
    expect(by(setup.teammateScored)).toMatchObject({ original: true, founder: 5, owner: 4, current: false });
    expect(by(setup.replacementMongols)).toMatchObject({ original: false }); // out of sight, so its current-capital flag is not shown

    // Default zoom 0.9, nothing selected, no tile card open: each state centred in the window.
    const states: [number, string][] = [
      [setup.own, 'c01-own-garrisoned'], [setup.held, 'c02-rival-held-garrisoned'], [setup.captured, 'c03-captured-by-rival'],
      [setup.scored, 'c04-scored-by-you'], [setup.teammateScored, 'c05-scored-by-teammate'], [setup.replacement, 'c06-replacement'],
      [setup.teammateOwn, 'c06a-allied-capital'],
    ];
    for (const [id, name] of states) {
      await frame(page, id, 0.9);
      await page.mouse.move(5, size.height / 2);
      await expect(page.locator('.tilecard')).toHaveCount(0);
      await shot(page, size, name);
    }
    // The banner's seal is never narrower than 16 px across at the smallest banner (zoom 0.75).
    await frame(page, setup.captured, 0.75);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c07-near-zoom-075');
    await frame(page, setup.captured, 0.42);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c08-far-zoom');
  });

  test('the banner tooltip explains population, owner, name and star at both zooms', async ({ page }) => {
    const setup = capitalsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    for (const [id, part, name] of [
      [setup.own, 'India', 'own'], [setup.held, 'Byzantium', 'rival'],
      [setup.teammateOwn, 'Aztecs', 'allied'], [setup.captured, 'Original capital of Switzerland', 'captured'],
    ] as const) {
      await frame(page, id, 0.9);
      const tip = await hoverBanner(page, id);
      await expect(tip).toContainText(`Population 3 · ${part === 'Original capital of Switzerland' ? 'Byzantium' : part}`);
      if (name === 'captured') await expect(tip).toContainText('Original capital of Switzerland · each rival that captures it scores +1 VP, once');
      const tipBox = (await tip.boundingBox())!, sideBox = (await page.locator('.right-sidebar').boundingBox())!;
      expect(tipBox.x + tipBox.width, 'banner tooltip stays clear of the sidebar').toBeLessThanOrEqual(sideBox.x + 1);
      await shot(page, size, `c09-banner-tip-${name}`);
    }
    await frame(page, setup.captured, 0.42);
    await expect(await hoverBanner(page, setup.captured, true)).toContainText('Population 3 · Byzantium');
    await shot(page, size, 'c09-banner-tip-far');
  });

  test('hover text: held, captured, scored, teammate-scored, own and replacement', async ({ page }) => {
    const setup = capitalsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await frame(page, setup.captured, 0.9);

    let card = await hoverTile(page, setup.captured);
    await expect(card.getByTestId('tile-capital')).toHaveText("Switzerland's original capital. Capture it to score +1 VP, kept for good. Each empire scores it once.");
    await shot(page, size, 'c10-tile-captured-capital');

    await frame(page, setup.scored, 0.9);
    card = await hoverTile(page, setup.scored);
    await expect(card.getByTestId('tile-capital')).toHaveText("Germany's original capital. You scored its +1 VP when you took it; taking it again scores nothing.");
    await shot(page, size, 'c11-tile-scored-capital');

    // Your teammate scored it: the star stays filled, and the line says you still can.
    await frame(page, setup.teammateScored, 0.9);
    card = await hoverTile(page, setup.teammateScored);
    await expect(card.getByTestId('tile-capital')).toHaveText("Mongols' original capital. Your teammate scored it. If a rival takes it, you can score it by capturing it yourself.");
    await shot(page, size, 'c12-tile-teammate-scored-capital');

    await frame(page, setup.own, 0.9);
    card = await hoverTile(page, setup.own);
    await expect(card.getByTestId('tile-capital')).toHaveText('Your original capital. Each rival that captures it scores +1 VP.');
    // The full stop after the VP icon is not swallowed: the icon leaves its own width to the text that follows.
    const stop = await card.getByTestId('tile-capital').locator('.amt-icon').evaluate((el) => {
      const r = el.getBoundingClientRect(), after = document.createRange();
      const text = el.parentElement!.nextSibling;
      if (!text) return { gap: 99 };
      after.setStart(text, 0); after.setEnd(text, 1);
      return { gap: after.getBoundingClientRect().left - r.right };
    });
    expect(stop.gap, 'the full stop sits clear of the VP icon').toBeGreaterThanOrEqual(0);
    await shot(page, size, 'c13-tile-own-capital');

    // A replacement capital: the current capital, but no star, and the card says what capturing it is worth.
    await frame(page, setup.replacement, 0.9);
    card = await hoverTile(page, setup.replacement);
    await expect(card.getByTestId('tile-capital')).toHaveCount(0);
    const fallen = await page.evaluate((id) => {
      const st = (window as unknown as GameWindow).legatum.getState(), v = st.views![st.seat];
      return v.cities.find((c) => c.id === id)!.name;
    }, setup.captured);
    await expect(card.getByTestId('tile-replacement')).toHaveText(`Capturing it scores no VP: Switzerland's original capital is ${fallen}.`);
    await shot(page, size, 'c14-tile-replacement-capital');
  });

  test('the city panel head: dots on ordinary kickers, the star only on original capitals, tooltips beside the head', async ({ page }) => {
    const setup = capitalsFixture();
    await loadFixture(page, coreFixture(setup.fixture), 2);
    const panel = page.locator('.city-panel');

    await selectThing(page, 'city', setup.captured);
    await expect(panel.getByTestId('city-capital')).toContainText("Switzerland's original capital.");
    await expect(panel.locator('.cp-kicker .capital-star')).toHaveCount(2);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c20-panel-captured-capital');

    await selectThing(page, 'city', setup.scored);
    await expect(panel.getByTestId('city-capital')).toContainText('You scored its +1 VP when you took it');
    await expect(panel.locator('.cp-kicker .capital-star').first()).toHaveAttribute('aria-label', 'Original capital, already scored');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c21-panel-scored-capital');

    await selectThing(page, 'city', setup.teammateScored);
    await expect(panel.getByTestId('city-capital')).toContainText('Your teammate scored it. If a rival takes it');
    await expect(panel.locator('.cp-kicker .capital-star').first()).toHaveAttribute('aria-label', 'Original capital');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c22-panel-teammate-scored-capital');

    // Your own founding city: the star is in the head and the line is a tooltip beside it, so the build list keeps its room.
    await selectThing(page, 'city', setup.own);
    await expect(panel.getByTestId('city-capital')).toHaveCount(0);
    await expect(panel.locator('.cp-kicker .capital-star')).toHaveCount(2);
    const star = panel.locator('.cp-kicker .capital-star').first();
    await star.hover();
    const tip = page.getByRole('tooltip');
    await expect(tip).toContainText('Your original capital. Each rival that captures it scores +1 VP.');
    const sb = (await star.boundingBox())!, tb = (await tip.boundingBox())!;
    // It opens to the right of the head (past the panel's own edge), level with the star: never down over the Lens box.
    expect(tb.x, 'the tooltip is to the right of the star').toBeGreaterThan(sb.x + sb.width);
    expect(Math.abs(tb.y + tb.height / 2 - (sb.y + sb.height / 2)), 'and level with it').toBeLessThan(60);
    await shot(page, size, 'c23-panel-own-capital-tooltip');
    await page.mouse.move(5, size.height / 2);

    // A replacement capital wears no star: its kicker has plain dots, and a hover says what capturing it is worth.
    await selectThing(page, 'city', setup.replacement);
    await expect(panel.locator('.cp-kicker .capital-star')).toHaveCount(0);
    await expect(panel.locator('.cp-kicker .kicker-dot')).toHaveCount(2);
    await expect(panel.getByTestId('city-capital')).toHaveCount(0);
    await panel.getByTestId('city-replacement').hover();
    await expect(page.getByRole('tooltip')).toContainText('Capturing it scores no VP.');
    await shot(page, size, 'c24-panel-replacement-capital-tooltip');
    await page.mouse.move(5, size.height / 2);

    // An ordinary city has dots too.
    await selectThing(page, 'city', setup.replacementGermany);
    await expect(panel.locator('.cp-kicker .capital-star')).toHaveCount(0);
  });

  test('the Guide explains the star', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await expect(guide).toBeVisible();
    await guide.getByRole('button', { name: /The victory race/ }).click();
    const heading = guide.locator('.guide-page').getByRole('heading', { name: /^Original capitals/ });
    await heading.scrollIntoViewIfNeeded();
    await expect(heading).toContainText('+1');
    await expect(guide.locator('.guide-page')).toContainText('A hollow star means you have already scored it.');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, 'c30-guide-original-capitals');
  });
});
