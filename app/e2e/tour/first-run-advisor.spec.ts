// A new player's advisor at both desktop sizes. Each shot keeps the live recommendation and its ring together.
import { test, expect } from '../fixtures';
import { SIZES, shot, type GameWindow } from './helpers';
import { setMapSeed } from '../mapSeed';

test.describe.configure({ mode: 'parallel' });
for (const size of SIZES) test.describe(`first run with advisor ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });
  test('welcome, build, move, research and end turn', async ({ page }) => {
    // The laptop size runs on the tech tree (the default view); the large size keeps the era cards.
    const onTree = size.name === '1000x600';
    await page.addInitScript((view) => { try { localStorage.setItem('legatum.research-view.v1', view); } catch { /* blocked */ } }, onTree ? 'tree' : 'era');
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await setMapSeed(page, '424242');
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    const stage = page.locator('dialog[data-testid="faction-stage"][open]');
    await stage.getByRole('option', { name: /Athens/ }).click();
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Start game/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 60_000 });
    const card = page.getByTestId('advisor');
    const ring = page.getByTestId('advisor-ring');
    const intersects = (a: { x: number; y: number; width: number; height: number }, b: { x: number; y: number; width: number; height: number }) =>
      a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y;
    let tileRadius = 24;
    const capture = async (name: string) => {
      await expect(card).toBeVisible();
      await page.mouse.move(5, size.height / 2);
      await shot(page, size, name);
      const cta = await page.locator('.seatbar .cta').boundingBox();
      const box = await card.boundingBox();
      if (!box) return;
      if (cta) expect(intersects(box, cta), 'the advisor never covers the turn button').toBe(false);
      // The advisor scrolls a cut-off target into view after a moment: wait until the ring's placement (its inline
      // position, not its pulse) has settled, then measure it.
      const placed = () => ring.evaluate((el) => `${(el as HTMLElement).style.left},${(el as HTMLElement).style.top}`);
      if (await ring.count()) for (let i = 0, last = await placed(); i < 10; i++) {
        await page.waitForTimeout(100);
        const now = await placed();
        if (now === last) break;
        last = now;
      }
      const ringBox = await ring.count() ? await ring.boundingBox() : null;
      if (ringBox) {
        // The ring's glow pulses a few pixels past its box; a 2px touch is not cover.
        const inner = { x: ringBox.x + 2, y: ringBox.y + 2, width: ringBox.width - 4, height: ringBox.height - 4 };
        expect(intersects(box, inner), 'the advisor never covers its ring').toBe(false);
        if (await ring.evaluate((el) => el.classList.contains('map-target'))) tileRadius = ringBox.width / 2;
      }
      const selected = await page.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState(), v = s.views?.[s.seat];
        const entity = s.selection?.kind === 'city' ? v?.cities.find((c) => c.id === s.selection?.id)
          : s.selection?.kind === 'unit' ? v?.units.find((u) => u.id === s.selection?.id) : null;
        if (!entity) return null;
        const p = (window as unknown as GameWindow).legatumMap.toScreen(entity.pos);
        const map = document.querySelector('[data-game-map]')!.getBoundingClientRect();
        return { x: map.left + p.x, y: map.top + p.y };
      });
      if (selected) expect(intersects(box, { x: selected.x - tileRadius, y: selected.y - tileRadius, width: 2 * tileRadius, height: 2 * tileRadius }),
        'the advisor never covers the selected tile').toBe(false);
      for (const target of await page.locator('dialog[open] .btn.primary, dialog[open] .era-card-title').all()) {
        const r = await target.boundingBox();
        if (r) expect(intersects(box, r), 'the advisor never covers a dialog primary or technology name').toBe(false);
      }
    };
    await expect(card).toHaveAttribute('data-advisor-id', 'welcome');
    await expect(card.getByRole('button', { name: /Continue/ })).toContainText('Return');
    await expect(page.locator('.seatbar .cta .kbd')).toHaveCount(0);
    await capture('01-welcome');
    await card.getByRole('button', { name: /^Continue/ }).click();
    await expect(card).toHaveAttribute('data-advisor-id', /^city:/);
    // On some layouts the capital is already selected on turn 1, so the card may open on its second step.
    await expect(ring).toBeVisible();
    const buildStepShown = await card.getByText('Step 2 of 2').isVisible().catch(() => false);
    await capture('02-build-first-ring');
    const cityId = Number((await card.getAttribute('data-advisor-id'))!.split(':')[1]);
    await page.evaluate((id) => { const s = (window as unknown as GameWindow).legatum.getState(); s.select({ kind: 'city', id }, true); }, cityId);
    await expect(page.locator('.city-panel')).toBeVisible();
    await expect(card).toContainText('Step 2 of 2');
    await expect(ring).toBeVisible();
    if (!buildStepShown) await capture('03-build-button-ring');
    const buildInstruction = (await card.locator('.advisor-step strong').textContent())!;
    const buildName = /Build on (.+)\./.exec(buildInstruction)?.[1];
    expect(buildName).toBeTruthy();
    await page.locator('.city-panel').getByRole('button', { name: `Build ${buildName}`, exact: true }).click();

    // Choosing a build moves the game to its next decision; on this seed that opens Research, and the advisor follows.
    const tech = page.locator('dialog.tech-modal[open]');
    if (!(await tech.isVisible())) {
      await expect(card).toHaveAttribute('data-advisor-id', 'research');
      await expect(ring).toBeVisible();
      await capture('04-research-ring');
      await page.keyboard.press('t');
    }
    await expect(tech).toBeVisible();
    await expect(card).toHaveAttribute('data-advisor-id', 'research');
    await expect(card).toContainText('Step 2 of 2');
    await expect(ring).toBeVisible();
    await capture('05-research-card-ring');
    if (onTree) {
      // The ring surrounds the recommended technology's node in the tree; clicking it researches it.
      const name = /^Click (.+)\.$/.exec((await card.locator('.advisor-step strong').textContent()) ?? '')?.[1];
      expect(name, 'the advisor names the technology').toBeTruthy();
      const node = tech.locator('.tree-node', { has: page.locator('.tn-name', { hasText: new RegExp(`^${name}$`) }) });
      const nodeBox = (await node.boundingBox())!, techRing = (await ring.boundingBox())!;
      expect(techRing.x <= nodeBox.x && techRing.x + techRing.width >= nodeBox.x + nodeBox.width
        && techRing.y <= nodeBox.y && techRing.y + techRing.height >= nodeBox.y + nodeBox.height, 'the ring surrounds the tree node').toBe(true);
      await node.click();
    } else await tech.getByRole('button', { name: 'Research now' }).first().click();
    // The queue lesson comes first and rings another technology in the open panel.
    await expect(card).toHaveAttribute('data-advisor-id', 'researchQueue');
    await expect(card.locator('.advisor-copy p')).toHaveText('Queue more technologies to keep researching after the current one.');
    await expect(ring).toBeVisible();
    await card.getByRole('button', { name: 'Got it' }).click();
    // With the lesson acknowledged, the advisor rings Close.
    await expect(card).toHaveAttribute('data-advisor-id', 'closeResearch');
    await expect(ring).toBeVisible();
    const close = (await tech.getByRole('button', { name: 'Close research' }).boundingBox())!;
    const closeRing = (await ring.boundingBox())!;
    expect(closeRing.x <= close.x && closeRing.x + closeRing.width >= close.x + close.width, 'the ring surrounds Close').toBe(true);
    await capture('05b-close-research-ring');
    await page.keyboard.press('Escape');
    await expect(tech).toBeHidden();
    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().select(null));

    // The first Hard unit move teaches the right-click command and its exact destination.
    let move = false;
    for (let i = 0; i < 12; i++) {
      await expect(card).toBeVisible();
      if (/^Move with your /.test((await card.locator('h3').textContent()) ?? '')) { move = true; break; }
      if ((await card.getAttribute('data-advisor-id')) === 'endTurn') break;
      await card.getByRole('button', { name: 'Not now' }).click();
    }
    expect(move, 'Hard should recommend a unit move in this opening').toBe(true);
    const unitId = Number((await card.getAttribute('data-advisor-id'))!.split(':')[1]);
    await expect(ring).toBeVisible();
    await capture('06-unit-ring');
    await page.evaluate((id) => { const s = (window as unknown as GameWindow).legatum.getState(); s.select({ kind: 'unit', id }, true); }, unitId);
    await expect(card).toContainText('Step 2 of 2');
    await expect(ring).toBeVisible();
    await capture('07-move-destination-ring');
    const destination = (await ring.boundingBox())!;
    await page.mouse.click(destination.x + destination.width / 2, destination.y + destination.height / 2, { button: 'right' });

    // Other units can be ordered without more tour shots; then the advisor points at End turn.
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
      s.editOrders((orders) => [...orders, ...v.units.filter((u) => u.owner === v.me && !orders.some((o) => 'unit' in o && o.unit === u.id))
        .map((u) => ({ type: 'SkipUnit' as const, unit: u.id }))]);
      s.select(null);
    });
    await expect(card).toHaveAttribute('data-advisor-id', 'endTurn');
    await expect(ring).toBeVisible();
    await capture('08-end-turn-ring');
  });
});
