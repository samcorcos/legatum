// The research screen under the strict era gate (era overhaul A1): only the current era's techs are
// cards, later eras are locked rows, every card shows its cost breakdown, and the queue
// forecasts each tech's arrival turn in the player's order. Also the top-bar era chip and the era-up
// notices. Shots go to test-results/era-research (look at them: they are the UI review).
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { auditLayout } from './layoutAudit';
import { coreFixture } from '../src/test/coreFixture';
import { researchFixture } from '../src/test/researchFixture';
import { TECHS, TECH_ERA } from '../src/data';
import type { useGame } from '../src/store';

type GameWindow = Window & { legatum: typeof useGame };
const SIZES = [{ width: 1000, height: 600 }, { width: 1440, height: 900 }];
const out = fileURLToPath(new URL('../test-results/era-research/', import.meta.url));

async function load(page: Page, game: string, turn = 2) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

/** Ada knows two Ancient techs; both rivals know Currency and Writing, so those two carry a tech diffusion discount. */
function ancientState(research: { current: string | null; queue: string[] }) {
  const f = researchFixture('India');
  f.players[0].research = research.current as never;
  f.players[0].research_queue = research.queue as never;
  return coreFixture(f);
}

async function shot(page: Page, size: { width: number; height: number }, name: string) {
  await page.mouse.move(size.width - 2, size.height - 2);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(400);
  await mkdir(`${out}${size.width}x${size.height}/`, { recursive: true });
  await page.screenshot({ path: `${out}${size.width}x${size.height}/${name}.png`, animations: 'disabled' });
}

const dialog = (page: Page) => page.locator('dialog.tech-modal');
const open = async (page: Page) => { await page.locator('.topbar .research-chip').click(); await expect(dialog(page)).toBeVisible(); await page.waitForTimeout(300); };

for (const size of SIZES) test.describe(`era research ${size.width}x${size.height}`, () => {
  test.use({ viewport: size });

  test('Medieval unit rows show upgrade paths without duplicate art', async ({ page }) => {
    const f = researchFixture('India');
    f.players[0].techs = TECHS.filter((tech) => TECH_ERA[tech] < 2);
    f.players[0].research = null;
    f.players[0].research_queue = [];
    await load(page, coreFixture(f));
    await open(page);
    const d = dialog(page);
    await expect(d.getByTestId('era-badge')).toContainText('Medieval era');
    for (const [tech, from] of [['Feudalism', 'Phalangite'], ['Machinery', 'Composite Archer'], ['Chivalry', 'Cataphract']] as const) {
      const node = d.locator(`.era-card[data-tech="${tech}"]`);
      await expect(node.locator('.era-unlock.unit')).toHaveCount(1);
      await expect(node.locator('.era-unlock.unit')).toContainText(`From ${from}`);
      await expect(node).not.toContainText('can upgrade to');
    }
    await shot(page, size, '08-medieval-upgrades');
  });

  test('the gate: five cards, later eras as locked rows, cost on every card, queue forecast', async ({ page }) => {
    await load(page, ancientState({ current: 'Masonry', queue: ['Masonry', 'Writing'] }));
    await open(page);
    const d = dialog(page);
    // The current era only: five cards, no tree, no 35-card wall.
    await expect(d.locator('.era-card')).toHaveCount(5);
    await expect(d.locator('.research-tree-scroll, .research-lines')).toHaveCount(0);
    await expect(d.getByTestId('era-badge')).toContainText('Ancient era');
    await expect(d.getByTestId('era-progress')).toHaveText('2 of 5 known');
    // Locked later eras: dim rows with their names and tech counts.
    const rows = d.locator('.era-row');
    await expect(rows).toHaveCount(6);
    await expect(rows.nth(0)).toContainText('Classical');
    await expect(rows.nth(0)).toContainText('5 technologies');
    await expect(rows.nth(5)).toContainText('Future');
    await expect(d.locator('.era-row[data-state="locked"]')).toHaveCount(6);

    // A card: cost, the diffusion discount, no boost, the unlocks.
    const writing = d.locator('.era-card[data-tech="Writing"]');
    await expect(writing.getByTestId('era-cost')).toContainText(/\d+/);
    await expect(d.locator('[data-testid^="boost"], .era-boost')).toHaveCount(0);
    await expect(d).not.toContainText(/boost/i);
    const currency = d.locator('.era-card[data-tech="Currency"]');
    const struckText = (await currency.locator('.era-cost-base').textContent())!;
    const struck = Number(struckText);
    const paid = Number((await currency.getByTestId('era-cost').textContent())!.replace(struckText, '').match(/\d+/)![0]);
    expect(struck, 'the struck listed price is above the discounted price').toBeGreaterThan(paid);
    await expect(currency.getByTestId('era-discounts')).toContainText('−20%: known by 2 empires');
    await expect(currency.locator('.era-unlock :is(.icon, .era-unlock-art img, .uc-figure)').first()).toBeVisible();
    // The known techs are marked, the current research ribboned.
    await expect(d.locator('.era-card.known')).toHaveCount(2);
    await expect(d.locator('.era-card.researching .era-now')).toHaveText(/^Researching · \d+ turns?$/);

    // The forecast: every queued tech has its arrival turn, in the order set.
    const arrivals = d.locator('.research-queue [data-testid="queue-arrival"]');
    await expect(arrivals).toHaveCount(2);
    await expect(arrivals.nth(0)).toHaveText(/^Turn \d+ · in \d+ turns?$/);
    const first = Number((await arrivals.nth(0).textContent())!.match(/^Turn (\d+)/)![1]);
    const second = Number((await arrivals.nth(1).textContent())!.match(/^Turn (\d+)/)![1]);
    expect(second).toBeGreaterThan(first);

    // Cards fit: nothing is clipped at the right edge, and at least the cards are whole at scroll 0.
    const fits = await d.evaluate((el) => {
      const box = el.getBoundingClientRect();
      return [...el.querySelectorAll('.era-card')].every((c) => { const b = c.getBoundingClientRect(); return b.left >= box.left && b.right <= box.right; });
    });
    expect(fits, 'cards inside the dialog').toBe(true);
    const issues = (await auditLayout(page, { minTarget: 24 })).filter((i) => /tech-modal|era-|research-/.test(i.where) && i.kind !== 'overlap');
    expect(issues).toEqual([]);
    await shot(page, size, '01-ancient-queue');

    // Locked rows peek at the names; the detail of a card opens under the cards.
    await rows.nth(1).locator('button').click();
    await expect(rows.nth(1).locator('.era-peek li')).toHaveCount(5);
    await writing.getByRole('button', { name: /^Show details/ }).click();
    await expect(d.locator('.research-detail[data-tech="Writing"]')).toBeVisible();
    await shot(page, size, '02-details-and-peek');
  });

  test('reordering the queue moves the forecast; Research now and Remove edit it', async ({ page }) => {
    await load(page, ancientState({ current: 'Masonry', queue: ['Masonry', 'Currency', 'Writing'] }));
    await open(page);
    const d = dialog(page);
    const order = () => d.locator('.research-queue li').evaluateAll((els) => els.map((e) => (e as HTMLElement).dataset.tech));
    const arrivalOf = async (tech: string) => d.locator(`.research-queue li[data-tech="${tech}"] [data-testid="queue-arrival"]`).textContent();
    expect(await order()).toEqual(['Masonry', 'Currency', 'Writing']);
    const writingBefore = await arrivalOf('Writing');
    await d.getByRole('button', { name: 'Move Writing earlier' }).click();
    expect(await order()).toEqual(['Masonry', 'Writing', 'Currency']);
    // Swapping two techs of different prices changes the turn each arrives (or leaves the later one's total).
    expect(await arrivalOf('Writing')).not.toBe(writingBefore);
    await expect(d.getByRole('button', { name: 'Move Masonry earlier' })).toBeDisabled();
    await expect(d.getByRole('button', { name: 'Move Currency later' })).toBeDisabled();
    // Nothing is sent until Ready: the pending mark shows.
    await expect(d.locator('.research-queue .pending-dot')).toBeVisible();
    const queued = () => page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.orders[s.seat].filter((o) => o.type === 'SetResearchQueue'); });
    expect(await queued()).toEqual([{ type: 'SetResearchQueue', techs: ['Masonry', 'Writing', 'Currency'] }]);
    await d.locator('.era-card[data-tech="Currency"]').getByTestId('research-now').click();
    expect(await order()).toEqual(['Currency', 'Masonry', 'Writing']);
    await d.locator('.research-queue li[data-tech="Masonry"]').getByRole('button', { name: 'Remove Masonry from the queue' }).click();
    expect(await order()).toEqual(['Currency', 'Writing']);
    await shot(page, size, '03-reordered');
  });

  test('with nothing queued the next action is obvious: one primary queues the era, cheapest first', async ({ page }) => {
    await load(page, ancientState({ current: null, queue: [] }));
    await expect(page.locator('.research-chip.needs')).toBeVisible();
    await open(page);
    const d = dialog(page);
    await expect(d.locator('.research-queue.idle')).toBeVisible();
    const primary = d.getByTestId('queue-all');
    await expect(primary).toBeVisible();
    await expect(primary).toHaveText(/^Queue all 3, cheapest first/);
    await expect(d.locator('.btn.primary')).toHaveCount(1); // the queue button is the only primary
    await shot(page, size, '04-nothing-queued');
    await primary.click();
    await expect(d.locator('.research-queue li')).toHaveCount(3);
    const costs = await d.locator('.research-queue li').evaluateAll((els) => els.map((e) => (e as HTMLElement).dataset.tech));
    const price = async (t: string) => Number((await d.locator(`.era-card[data-tech="${t}"] [data-testid="era-cost"]`).textContent())!.match(/\d+/g)!.at(-1));
    const prices = [];
    for (const t of costs) prices.push(await price(t as string));
    expect([...prices].sort((a, b) => a - b)).toEqual(prices);
    await expect(d.locator('.research-queue.idle')).toHaveCount(0);
  });

  test('the top bar names the era, fits, and opens the research screen', async ({ page }) => {
    await load(page, ancientState({ current: 'Masonry', queue: ['Masonry', 'Writing'] }));
    const chip = page.getByTestId('era-chip');
    await expect(chip).toBeVisible();
    await expect(chip).toContainText('Ancient');
    const bar = await page.locator('.topbar').boundingBox();
    const box = (await chip.boundingBox())!;
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(bar!.x + bar!.width);
    // The research cell still reads whole beside it.
    const research = await page.locator('.research-chip').boundingBox();
    expect(research!.x).toBeGreaterThanOrEqual(box.x + box.width - 1);
    await shot(page, size, '05-topbar');
    await chip.click();
    await expect(dialog(page)).toBeVisible();
  });

  test('entering an era: your own card names the new techs and opens research; a rival gets a banner', async ({ page }) => {
    // Ada knows four Ancient techs and finishes the fifth this turn.
    const f = researchFixture('India');
    f.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency'];
    f.players[0].research = 'Writing';
    f.players[0].research_queue = ['Writing'];
    f.players[0].science = 5000;
    await load(page, coreFixture(f));
    await endPlanning(page);
    const unlock = page.locator('.moment-card.unlock');
    await expect(unlock).toBeVisible();
    await page.keyboard.press('Escape');
    // The era-up is the full-screen takeover (e2e/era-takeover.spec.ts has its own checks and shots).
    const era = page.getByTestId('era-takeover');
    await expect(era).toBeVisible();
    await expect(era).toContainText('The Classical Era');
    await expect(era).toContainText('No research is set');
    await page.waitForTimeout(3200);
    await shot(page, size, '06-era-up-card');
    await page.keyboard.press('Enter');
    await expect(era).toHaveCount(0);
    await expect(dialog(page)).toBeVisible();
    await expect(dialog(page).getByTestId('era-badge')).toContainText('Classical era');
    await expect(dialog(page).locator('.era-card')).toHaveCount(5);
    await shot(page, size, '07-classical-research');
    await page.keyboard.press('Escape');

    // A rival's entry: a banner over the map, and a highlighted line in the report.
    await page.evaluate(() => {
      const s = (window as unknown as GameWindow).legatum.getState();
      s.set({ report: { turn: s.views![s.seat].turn, hash: 'era', lines: [[], []], events: [[{ type: 'EraEntered', player: 1, era: 3, first: true }], []] }, reportOpen: false });
    });
    const notice = page.getByTestId('era-notice');
    await expect(notice).toBeVisible();
    await expect(notice).toContainText('entered the Medieval era first');
    await shot(page, size, '08-rival-era-notice');
  });
});
