// Screen tour, the end of the game and the 8.9.4 features. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/post-game`. Shots land in test-results/screen-tour/<size>/post-game/.
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { overlayBox } from '../overlayHelpers';
import { coreFixture } from '../../src/test/coreFixture';
import { gameOverCoreSave } from '../../src/test/gameOverFixture';
import { postGameSave } from '../../src/test/postGameFixture';
import { siegeFixture } from '../../src/test/siegeFixture';
import { SIZES, loadFixture, shot, showMoment, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('game over: victory, team victory, a rival\'s victory and defeat', async ({ page, browser }) => {
    test.setTimeout(120_000);
    await loadFixture(page, gameOverCoreSave('victory'), 164);
    await expect(page.locator('dialog.gameover')).toBeVisible();
    await page.getByTestId('game-reveal').click({ position: { x: 6, y: 6 } });
    await expect(page.getByTestId('game-reveal')).toHaveAttribute('data-settled', 'true');
    await expect(page.locator('.go-row b').first()).not.toHaveText(/^0\/10$/);
    await shot(page, size, '26-victory');

    for (const [kind, name] of [['team-victory', '26b-victory-team'], ['rival-victory', '26c-defeat-rival-victory'], ['defeat', '27-defeat']] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        await loadFixture(p, gameOverCoreSave(kind), 164);
        await expect(p.locator('dialog.gameover')).toBeVisible();
        await p.getByTestId('game-reveal').click({ position: { x: 6, y: 6 } });
        await expect(p.getByTestId('game-reveal')).toHaveAttribute('data-settled', 'true');
        await expect(p.locator('.go-row b').first()).not.toHaveText(/^0\/10$/);
        await shot(p, size, name);
      } finally { await context.close(); }
    }
  });

  // The end of a game with its recorded history (real end states of the user's own games): the reveal, then the
  // post-game report's four tabs, for a win and for a game lost to the AI.
  test('end of game: the reveal and the post-game report, won and lost', async ({ browser }) => {
    test.setTimeout(240_000);
    for (const [fixture, prefix] of [['swiss-win', '26d'], ['rival-win', '26j']] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const { game, turn } = postGameSave(fixture);
        await loadFixture(p, game, turn);
        const reveal = p.getByTestId('game-reveal');
        await expect(reveal).toBeVisible();
        // A click on the scene finishes the sequence (the standings land on their final numbers).
        await reveal.click({ position: { x: 6, y: 6 } });
        await expect(reveal).toHaveAttribute('data-settled', 'true');
        await shot(p, size, `${prefix}-end-reveal-${fixture}`);
        await p.locator('.gr-actions .btn.primary').click();
        await expect(p.getByTestId('postgame-report')).toBeVisible();
        // Park the mouse: the primary's spot lands on the chart legend at 1440x900, and a hovered legend row picks
        // out one empire and fades every other line (8.9.4 review: the shot showed Byzantium picked out).
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, `${prefix}a-end-report-race-${fixture}`);
        const tabs = [['growth', 'b-end-report-growth'], ['timeline', 'c-end-report-timeline'], ['stats', 'd-end-report-stats']] as const;
        for (const [tab, name] of tabs) {
          await p.locator(`[role="tab"][data-tab="${tab}"]`).click();
          await p.mouse.move(5, size.height / 2);
          await shot(p, size, `${prefix}${name}-${fixture}`);
        }
        await p.getByRole('button', { name: 'Keep looking at the map' }).click();
        await expect(p.locator('.seatbar .cta')).toContainText('Game report');
        await shot(p, size, `${prefix}e-end-map-after-${fixture}`);
      } finally { await context.close(); }
    }
  });
  test('8.9.4 features: the Cities overview, the map overlays and the air wing ready card', async ({ page }) => {
    test.setTimeout(150_000);
    await page.clock.install();
    const setup = siegeFixture({ fresh: true });
    setup.fixture.units = setup.fixture.units.filter((u) => u.owner === 0);
    const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
    own.producing = { kind: 'Unit', unit: 'Fighter' };
    own.prod = 400;
    // Flight unlocks the Biplane; the Fighter needs Air Power.
    setup.fixture.players[0].techs.push('Flight', 'AirPower');
    await loadFixture(page, coreFixture(setup.fixture), 2);
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ techOpen: false }));
    // The Cities overview opens from the top bar button and closes with Esc.
    await page.getByTestId('cities-button').click();
    await expect(page.locator('.co-table')).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '97a-cities-overview');
    await page.keyboard.press('Escape');
    await expect(page.locator('.co-table')).toHaveCount(0);
    // The three overlays stack over the default lens.
    const chooser = page.getByRole('button', { name: /^Lens: / });
    if (await chooser.isVisible()) await chooser.click();
    for (const name of ['Yields', 'Resources', 'Borders']) {
      await overlayBox(page, name).click();
      await expect(overlayBox(page, name)).toBeChecked();
    }
    if (await chooser.isVisible()) await chooser.click();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '97b-overlays-all-on');
    if (await chooser.isVisible()) await chooser.click();
    for (const name of ['Yields', 'Resources', 'Borders']) {
      await overlayBox(page, name).click();
      await expect(overlayBox(page, name)).not.toBeChecked();
    }
    if (await chooser.isVisible()) await chooser.click();
    // A finished Fighter states its job: the kicker carries the city, the body scrolls and the buttons stay pinned.
    await page.clock.pauseAt(await page.evaluate(() => Date.now()) + 2000);
    await endPlanning(page);
    await page.clock.runFor(100);
    await showMoment(page, 'Fighter ready');
    const card = page.locator('.moment-card', { hasText: 'Fighter ready' });
    await expect(card).toContainText('Aircraft ready · Waypost Seven', { timeout: 25_000 });
    await expect(card.locator('h2')).toHaveText('Fighter ready');
    const area = (await card.boundingBox())!;
    // Read both boxes in one frame: the card is still arriving, so separate
    // boundingBox calls can sample different positions during its animation.
    const { bodyBottom, footerTop } = await card.evaluate((el) => ({
      bodyBottom: el.querySelector('.moment-body')!.getBoundingClientRect().bottom,
      footerTop: el.querySelector('footer')!.getBoundingClientRect().top,
    }));
    expect(bodyBottom, 'moment body stops before the buttons').toBeLessThanOrEqual(footerTop + 0.5);
    for (const name of ['Show on map']) {
      const box = (await card.getByRole('button', { name }).boundingBox())!;
      expect(box.y + box.height, `${name} is inside the card and on screen`).toBeLessThanOrEqual(Math.min(area.y + area.height, size.height) + 0.5);
    }
    await page.evaluate(() => (window as unknown as GameWindow).legatum.setState({ reportOpen: false }));
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '97c-air-wing-ready-card');
    await card.getByRole('button', { name: 'Show on map' }).click(); // the primary also closes the card
    await page.evaluate(() => (window as unknown as GameWindow).legatum.setState({ reportOpen: false }));
    const fighter = await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      return s.views?.[s.seat].aircraft?.find((a) => a.kind === 'Fighter')?.id ?? null;
    });
    expect(fighter).not.toBeNull();
    await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'air', id: id! }, true), fighter);
    await expect(page.getByTestId('air-wing')).toBeVisible();
    await expect(page.getByTestId('air-wing').getByRole('button', { name: /Rebase/ })).toBeDisabled();
    await expect(page.getByTestId('air-wing')).toContainText('Rebase unavailable:');
    const wing = (await page.getByTestId('air-wing').boundingBox())!;
    const actions = (await page.getByTestId('air-wing').locator('.actions').boundingBox())!;
    expect(actions.y + actions.height, 'aircraft actions stay inside the unit card').toBeLessThanOrEqual(wing.y + wing.height + 0.5);
    const head = await page.getByTestId('air-wing').locator('.up-head').evaluate((el) => ({ scroll: el.scrollHeight, visible: el.clientHeight }));
    expect(head.scroll, 'aircraft help is fully visible above its buttons').toBeLessThanOrEqual(head.visible + 1);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '97d-aircraft-unit-card');
  });
});
