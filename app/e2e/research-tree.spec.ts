// The tech tree view of Research (TechTree.tsx): it opens by default on your era, a click on a
// technology of your era researches it (Shift+click or its + Queue button queues), a later one queues
// behind what it waits for (marked red first), and the queue reaches the engine across eras. Shots go to test-results/research-tree/<size>/ for the UI reviewer.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { dismissInterstitials, endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import type { useGame } from '../src/store';
import type { TechId } from '../src/types';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const out = fileURLToPath(new URL('../test-results/research-tree/', import.meta.url));
const KNOWN: TechId[] = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency', 'Writing', 'Mathematics', 'Engineering', 'IronWorking', 'CodeOfLaws', 'Philosophy', 'Feudalism', 'Education', 'CropRotation'];

async function load(page: Page, known: TechId[] = KNOWN) {
  const f = researchFixture('Switzerland');
  f.players[0].techs = [...known];
  f.players[0].science = 60;
  f.players[1].techs = [...known, 'Chivalry'];
  f.players[0].research = null as never;
  f.players[0].research_queue = [] as never;
  for (const c of f.cities) c.population = 6;
  const game = coreFixture(f);
  await page.addInitScript(({ game }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    // The default view: no remembered choice.
    localStorage.removeItem('legatum.research-view.v1');
  }, { game });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await dismissInterstitials(page);
  await page.locator('.topbar .research-chip').click();
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
}

const plan = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  const orders = s.orders[s.seat];
  const q = orders.findLast((x) => x.type === 'SetResearchQueue');
  const g = orders.findLast((x) => x.type === 'SetResearchGoal');
  return { queue: q?.type === 'SetResearchQueue' ? q.techs : [], goal: g?.type === 'SetResearchGoal' ? g.tech : undefined };
});

async function shot(page: Page, size: { width: number; height: number }, name: string) {
  await page.mouse.move(size.width - 2, size.height - 2);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(500);
  await mkdir(`${out}${size.width}x${size.height}/`, { recursive: true });
  await page.screenshot({ path: `${out}${size.width}x${size.height}/${name}.png`, animations: 'disabled' });
}

for (const size of SIZES) {
  test(`the tech tree researches and queues across eras at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(120_000);
    await page.setViewportSize(size);
    await load(page);
    const tree = page.getByTestId('tech-tree');
    await expect(tree).toBeVisible();
    await expect(page.getByTestId('research-view-tree')).toHaveAttribute('aria-pressed', 'true');
    // Every technology is on the tree, and your era's column is in view.
    await expect(tree.locator('.tree-node')).toHaveCount(45);
    await expect(tree.locator('.tree-era.current')).toHaveAttribute('data-era', '2');
    await expect(tree.locator('.tree-node[data-tech="Chivalry"]')).toBeInViewport();
    await expect(tree.locator('.tree-node[data-tech="Mathematics"]')).toHaveAttribute('data-state', 'known');
    await expect(tree.locator('.tree-node[data-tech="Gunpowder"]')).toHaveAttribute('data-state', 'locked');
    expect(await tree.locator('.tree-links path').count()).toBeGreaterThan(20);
    // The queue stays on screen at every size, and hovering a technology shows what it gives.
    await expect(page.locator('.research-queue')).toBeVisible();
    await tree.locator('.tree-node[data-tech="Gunpowder"]').hover();
    await expect(page.locator('.tree-peek')).toContainText('Gunpowder');
    await page.mouse.move(2, 2);
    await expect(page.locator('.tree-peek')).toHaveCount(0);
    await shot(page, size, '01-tree');

    await tree.locator('.tree-node[data-tech="Chivalry"]').click();
    await expect(tree.locator('.tree-node[data-tech="Chivalry"]')).toHaveAttribute('data-state', 'researching');
    await tree.locator('.tree-node[data-tech="Machinery"]').click({ modifiers: ['Shift'] });
    await expect(tree.locator('.tree-node[data-tech="Machinery"]')).toHaveAttribute('data-state', 'queued');
    expect((await plan(page)).queue).toEqual(['Chivalry', 'Machinery']);
    // Known technologies do nothing on click.
    await expect(tree.locator('.tree-node[data-tech="Feudalism"]')).toHaveAttribute('aria-disabled', 'true');
    await tree.locator('.tree-node[data-tech="Feudalism"]').click({ force: true });
    expect((await plan(page)).queue).toEqual(['Chivalry', 'Machinery']);

    // Every unknown technology has its own Queue button. A later one whose earlier eras are all queued goes straight in.
    await tree.locator('[data-testid="tree-queue"][data-tech="Gunpowder"]').click();
    expect((await plan(page)).queue).toEqual(['Chivalry', 'Machinery', 'Gunpowder']);
    await expect(tree.locator('[data-testid="tree-queue"][data-tech="Gunpowder"]')).toHaveText(/Remove/);

    // One further on: the tree says the era must be finished first, marks what is missing in red, and queues it all in order.
    await tree.locator('.tree-node[data-tech="Rifling"]').click();
    await expect(page.getByTestId('tree-ask')).toContainText('Finish the Renaissance era first');
    await expect(tree.locator('.tree-node.needed')).toHaveCount(4);
    await shot(page, size, '02-ask');
    await page.getByTestId('tree-ask-queue').click();
    await expect(page.getByTestId('tree-ask')).toHaveCount(0);
    const queued = (await plan(page)).queue;
    expect(queued).toHaveLength(8);
    expect(queued.slice(0, 3)).toEqual(['Chivalry', 'Machinery', 'Gunpowder']);
    expect(queued.at(-1)).toBe('Rifling');
    await shot(page, size, '03-queued-across-eras');

    // Removing a Medieval technology takes out the later ones that waited behind it.
    await tree.locator('[data-testid="tree-queue"][data-tech="Machinery"]').click();
    expect((await plan(page)).queue).toEqual(['Chivalry']);
    // Two eras short: both are named, and Cancel leaves the queue alone.
    await tree.locator('[data-testid="tree-queue"][data-tech="Rifling"]').click();
    await expect(page.getByTestId('tree-ask')).toContainText('Finish the Medieval and Renaissance eras first');
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(tree.locator('.tree-node.needed')).toHaveCount(0);
    expect((await plan(page)).queue).toEqual(['Chivalry']);
    await tree.locator('[data-testid="tree-queue"][data-tech="Rifling"]').click();
    await page.getByTestId('tree-ask-queue').click();
    const plan2 = (await plan(page)).queue;
    expect(plan2).toHaveLength(8);

    // The view switch is remembered.
    await page.getByTestId('research-view-era').click();
    await expect(page.locator('.era-card[data-tech="Chivalry"]')).toBeVisible();
    await expect(tree).toHaveCount(0);
    expect(await page.evaluate(() => localStorage.getItem('legatum.research-view.v1'))).toBe('era');
    await page.getByTestId('research-view-tree').click();
    await expect(page.getByTestId('tech-tree')).toBeVisible();

    // The engine takes the queue across eras and keeps it over the turn.
    await page.keyboard.press('Escape');
    await expect(page.locator('dialog.tech-modal')).toHaveCount(0);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText('3', { timeout: 20_000 });
    await dismissInterstitials(page);
    const engineQueue = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views?.[s.seat]?.research.queue; });
    expect(engineQueue?.at(-1)).toBe('Rifling');
  });
}

// In a late era your column is far from the first screen: the tree must still open on it, every time it opens.
const LATE: TechId[] = ['Irrigation', 'Masonry', 'BronzeWorking', 'Currency', 'Writing', 'Mathematics', 'Engineering', 'IronWorking', 'CodeOfLaws', 'Philosophy',
  'Feudalism', 'Machinery', 'Chivalry', 'CropRotation', 'Education', 'Gunpowder', 'MilitaryEngineering', 'Metallurgy', 'Banking', 'PrintingPress',
  'Rifling', 'Cannonry', 'MilitaryScience', 'Industrialization', 'Railways'];
for (const size of SIZES) {
  test(`the tech tree opens on your era late in the game at ${size.width}x${size.height}`, async ({ page }) => {
    test.setTimeout(120_000);
    await page.setViewportSize(size);
    await load(page, LATE);
    const tree = page.getByTestId('tech-tree');
    const current = await tree.locator('.tree-era.current').getAttribute('data-era');
    expect(Number(current)).toBeGreaterThanOrEqual(5);
    await expect(tree.locator('.tree-node.available').first()).toBeInViewport({ ratio: 0.9 });
    await shot(page, size, '04-late');
    // Closed and opened again (after scrolling away), it opens on your era again.
    await tree.locator('.tree-scroll').evaluate((el) => { el.scrollLeft = 0; });
    await page.keyboard.press('Escape');
    await expect(page.locator('dialog.tech-modal')).toHaveCount(0);
    await page.locator('.topbar .research-chip').click();
    await expect(page.getByTestId('tech-tree').locator('.tree-node.available').first()).toBeInViewport({ ratio: 0.9 });
  });
}
