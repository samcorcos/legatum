// The research screen's whole-card shortcuts (Civ style): click a card to research it now, Shift+click to queue it,
// Enter / Shift+Enter from the keyboard. Selecting never shifts anything under the pointer: the queue strip keeps its
// height (empty slots stand in for the techs not yet queued) and the state tags are pinned out of the layout flow.
// The era rewrite (250cbf77) once silently dropped these shortcuts; this spec is their regression test.
// Shots go to test-results/research-click/<size>/ for the UI reviewer.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Locator, Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { dismissInterstitials } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const out = fileURLToPath(new URL('../test-results/research-click/', import.meta.url));
const HINT = 'Shift+click a technology to queue it next.';

async function load(page: Page) {
  const f = researchFixture('India');
  f.players[0].research = null as never;
  f.players[0].research_queue = [] as never;
  const game = coreFixture(f);
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await dismissInterstitials(page);
  await page.locator('.topbar .research-chip').click();
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await page.waitForTimeout(300);
  return page.locator('dialog.tech-modal');
}

const queueOrder = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  const o = s.orders[s.seat].findLast((x) => x.type === 'SetResearchQueue');
  return o && o.type === 'SetResearchQueue' ? o.techs : [];
});

type Box = { x: number; y: number; width: number; height: number };
const boxes = async (locators: Locator[]) => Promise.all(locators.map(async (l) => (await l.boundingBox()) as Box));
const same = (a: Box, b: Box, what: string) => {
  expect(a && b, what).toBeTruthy();
  for (const k of ['x', 'y', 'width', 'height'] as const) expect(Math.abs(a[k] - b[k]), `${what}: ${k}`).toBeLessThan(0.75);
};

async function shot(page: Page, size: { width: number; height: number }, name: string) {
  await page.mouse.move(size.width - 2, size.height - 2);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(700); // the pick animation has settled
  await mkdir(`${out}${size.width}x${size.height}/`, { recursive: true });
  await page.screenshot({ path: `${out}${size.width}x${size.height}/${name}.png`, animations: 'disabled' });
}

for (const size of SIZES) test.describe(`research card shortcuts ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('click researches, Shift+click queues twice; the cards and the queue strip never move; queued cards show a position, not the button', async ({ page }) => {
    const d = await load(page);
    const card = (tech: string) => d.locator(`.era-card[data-tech="${tech}"]`);
    const all = d.locator('.era-card');
    const cards = await all.all();
    expect(cards.length).toBe(5);
    const strip = d.locator('.research-queue');
    // Aim at the card's top band (its name row): always in view, so the click never scrolls the panel.
    const at = { x: 60, y: 24 };
    await expect(d.getByTestId('queue-hint')).toContainText(HINT);
    await shot(page, size, '01-nothing-queued');

    const before = await boxes([...cards, strip]);
    // 1. A plain click on the card (not on a button) researches it now.
    await card('Masonry').click({ position: at });
    await expect(card('Masonry')).toHaveAttribute('data-state', 'researching');
    await expect(card('Masonry').getByTestId('era-researching')).toContainText(/^Researching · \d+ turns?$/);
    await expect(card('Masonry').getByTestId('research-now')).toHaveCount(0);
    await expect(card('Masonry').getByTestId('queue-add')).toHaveCount(0);
    expect(await queueOrder(page)).toEqual(['Masonry']);
    const afterFirst = await boxes([...cards, strip]);
    afterFirst.forEach((b, i) => same(b, before[i], `box ${i} after the click`));

    // 2. Shift+click two more: appended in order, badges pinned, no "+ Queue" left on them.
    await card('Writing').click({ position: at, modifiers: ['Shift'] });
    await card('Currency').click({ position: at, modifiers: ['Shift'] });
    expect(await queueOrder(page)).toEqual(['Masonry', 'Writing', 'Currency']);
    await expect(card('Writing').getByTestId('research-order-badge')).toHaveText('Queued #2');
    await expect(card('Currency').getByTestId('research-order-badge')).toHaveText('Queued #3');
    await expect(card('Masonry').getByTestId('research-order-badge')).toHaveCount(0);
    for (const tech of ['Writing', 'Currency']) {
      await expect(card(tech).getByTestId('queue-add'), `${tech} still offers + Queue`).toHaveCount(0);
      await expect(card(tech).getByRole('button', { name: `Remove ${tech === 'Writing' ? 'Writing' : 'Currency'} from the queue` })).toBeVisible();
      await expect(card(tech).getByTestId('research-now')).toHaveCount(1); // the one action left on a queued card
    }
    // The tag is out of the flow: it overhangs the card's top edge, and no card text sits under it.
    const tag = (await card('Writing').getByTestId('research-order-badge').boundingBox())!;
    const writing = (await card('Writing').boundingBox())!;
    expect(tag.y).toBeLessThan(writing.y);
    const head = (await card('Writing').locator('.era-card-head').boundingBox())!;
    expect(tag.y + tag.height, 'the tag stays clear of the card head').toBeLessThanOrEqual(head.y + 0.75);
    // The card's `when` line stays one line in every state (a wrapped one grew the whole row once).
    for (const tech of ['Masonry', 'Writing', 'Currency']) expect((await card(tech).getByTestId('era-when').boundingBox())!.height, `${tech} when line`).toBeLessThan(18);
    const afterAll = await boxes([...cards, strip]);
    afterAll.forEach((b, i) => same(b, before[i], `box ${i} after three picks`));

    // The strip lists them in order, with no empty slot for a queued tech, and the hint is gone for good.
    await expect(d.locator('.research-queue li')).toHaveCount(3);
    expect(await d.locator('.research-queue li').evaluateAll((els) => els.map((e) => (e as HTMLElement).dataset.tech))).toEqual(['Masonry', 'Writing', 'Currency']);
    await expect(d.getByTestId('queue-hint')).toHaveCount(0);
    expect(await page.evaluate(() => localStorage.getItem('legatum.research-queue-hint.v1'))).toBe('1');
    await shot(page, size, '02-researching-and-queued');

    // 3. Click a queued card: it becomes the current research, the others follow; still nothing moves.
    await card('Currency').click({ position: at });
    expect(await queueOrder(page)).toEqual(['Currency', 'Masonry', 'Writing']);
    await expect(card('Currency')).toHaveAttribute('data-state', 'researching');
    await expect(card('Masonry').getByTestId('research-order-badge')).toHaveText('Queued #2');
    (await boxes([...cards, strip])).forEach((b, i) => same(b, before[i], `box ${i} after Research now by click`));

    // 4. The corner × removes a queued tech; the card is back to Research now + Queue.
    await card('Writing').getByRole('button', { name: 'Remove Writing from the queue' }).click();
    expect(await queueOrder(page)).toEqual(['Currency', 'Masonry']);
    await expect(card('Writing').getByTestId('queue-add')).toBeVisible();
    (await boxes([...cards, strip])).forEach((b, i) => same(b, before[i], `box ${i} after removing`));
    await shot(page, size, '03-after-edits');
  });

  test('buttons keep working, the shortcuts are in their tooltips, and Enter / Shift+Enter work from a focused card', async ({ page }) => {
    const d = await load(page);
    const card = (tech: string) => d.locator(`.era-card[data-tech="${tech}"]`);
    await expect(card('Writing').getByTestId('research-now')).toHaveAttribute('title', /click anywhere on the card/i);
    await expect(card('Writing').getByTestId('queue-add')).toHaveAttribute('title', /Shift\+click the card/);
    await card('Writing').getByTestId('research-now').click();
    await card('Masonry').getByTestId('queue-add').click();
    expect(await queueOrder(page)).toEqual(['Writing', 'Masonry']);
    // Keyboard: focus a card; Enter researches it, Shift+Enter queues it.
    await card('Currency').focus();
    await page.keyboard.press('Shift+Enter');
    expect(await queueOrder(page)).toEqual(['Writing', 'Masonry', 'Currency']);
    await card('Currency').focus();
    await page.keyboard.press('Enter');
    expect(await queueOrder(page)).toEqual(['Currency', 'Writing', 'Masonry']);
    // Researched cards are not targets.
    await expect(d.locator('.era-card.known').first()).not.toHaveAttribute('tabindex', /.*/);
    await expect(d).toBeVisible(); // Enter on a card is the card's, it does not close the screen
  });

  test('the pick animation is short, takes no layout and leaves no chip behind', async ({ page }) => {
    const d = await load(page);
    // Watch the transition before clicking: under a busy browser the 520 ms effect can
    // finish before Playwright's next assertion, even though it did render.
    await d.evaluate((dialog) => {
      const observed = () => {
        if (dialog.querySelector('.era-card.picked')) dialog.setAttribute('data-picked-observed', 'yes');
        if (dialog.querySelector('.fly-chip')) dialog.setAttribute('data-fly-observed', 'yes');
      };
      const observer = new MutationObserver(observed);
      observer.observe(dialog, { subtree: true, childList: true, attributes: true, attributeFilter: ['class'] });
    });
    await d.locator('.era-card[data-tech="Writing"]').click({ position: { x: 60, y: 24 } });
    await expect(d).toHaveAttribute('data-picked-observed', 'yes');
    await expect(d).toHaveAttribute('data-fly-observed', 'yes');
    await expect(d.locator('.fly-chip')).toHaveCount(0, { timeout: 1500 });
    await expect(d.locator('.era-card.picked')).toHaveCount(0, { timeout: 1500 });
  });

  test('reduced motion: the card and queue still change at once, with no flying chip', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const d = await load(page);
    await d.locator('.era-card[data-tech="Writing"]').click({ position: { x: 60, y: 24 } });
    await expect(d.locator('.era-card[data-tech="Writing"]')).toHaveAttribute('data-state', 'researching');
    await expect(d.locator('.fly-chip')).toHaveCount(0);
  });
});
