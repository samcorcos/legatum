import { test as baseTest, type Page } from '@playwright/test';
import { newestAutosave, newestGameState } from './autosave';
import { test, expect } from './fixtures';
import { dismissInterstitials, endPlanning } from './turnActions';
import { turnFlowFixture } from '../src/test/turnFlowFixture';
import { coreFixture } from '../src/test/coreFixture';

test.beforeEach(async ({ page }) => {
  const fixture = turnFlowFixture();
  fixture.players[1].isHuman = false;
  fixture.setup.players[1].is_human = false;
  await page.addInitScript((game) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2,
      names: ['Explorer'], savedAt: new Date().toISOString(), game,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, coreFixture(fixture));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.locator('[data-game-map]').focus();
});

test('Return cycles units, production and research while decisions remain', async ({ page }) => {
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await page.keyboard.press('Escape');
  for (let i = 0; i < 12; i++) {
    // Dialog controls keep native Enter; close research to continue the gameplay cycle.
    if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
    await page.locator('[data-game-map]').focus();
    await page.keyboard.press('Enter');
    await expect(page.getByTestId('turn-number')).toHaveText('2');
    await expect(page.locator('dialog.end-turn-confirmation')).toHaveCount(0);
  }
});

test('Shift+Return reviews every pending decision; Return on its default control goes to the first', async ({ page }) => {
  await page.keyboard.press('Shift+Enter');
  const dialog = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('Spearman #0: needs orders');
  await expect(dialog).toContainText('Scout #4: needs orders');
  await expect(dialog).toContainText(/Choose what .+ builds/);
  await expect(dialog).toContainText('Choose research');
  await expect(dialog.getByRole('button', { name: 'Give the Spearman orders' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(dialog).toHaveCount(0);
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Shift+Enter');
  await dialog.getByRole('button', { name: 'End turn anyway', exact: true }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('3');
});

test('Space lasts one turn; Fortify persists and stays out of the cycle', async ({ page }) => {
  await page.keyboard.press('Space');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.keyboard.press('f');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Spearman');
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toBeVisible();
  // Autosave is an output readback: no store injection or debug mutations.
  await expect.poll(async () => (await newestAutosave(page))?.turn).toBe(3);
  const units = (await newestGameState<any>(page)).units;
  expect(units['0'].standing_order).toBeUndefined();
  expect(units['4'].standing_order).toBe('Sleep');
});

test('Return ends the turn when all decisions are handled, and focused controls keep native Space', async ({ page }) => {
  const skip = page.getByRole('button', { name: /^Skip this turn/ });
  await skip.focus();
  await page.keyboard.press('Space');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await skip.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await page.locator('.city-panel .production-choice:not(.unavailable) button.prod:enabled').first().click();
  const research = page.locator('dialog.tech-modal');
  await expect(page.locator('.city-panel')).toHaveCount(0);
  await expect(research).toBeVisible();
  await research.locator('.era-card.available').first().getByTestId('research-now').click();
  await page.keyboard.press('Escape');
  await expect(page.locator('.seatbar .cta.ready')).toBeVisible();
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await expect(page.locator('dialog.end-turn-confirmation')).toHaveCount(0);
});

test('mouse Build advances to research and the city remains reachable from Cities', async ({ page }) => {
  // Cities in the top bar lists every city; a row goes to it.
  const openCity = async () => {
    await page.getByTestId('cities-button').click();
    await page.locator('[data-testid="city-row"] .co-city').first().click();
  };
  await openCity();
  const build = page.getByRole('button', { name: 'Build Spearman', exact: true });
  await build.click();
  await expect(page.locator('.city-panel')).toHaveCount(0);
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await page.keyboard.press('Escape');
  await openCity();
  await expect(build).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('button', { name: 'Queue Spearman', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /^Buy Spearman for/ })).toBeVisible();
});

test('Fortify heals while the unit rests and stays in effect at full HP, out of the cycle', async ({ page }) => {
  await page.keyboard.press('f');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('f');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('3');
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('4');
  // The autosave is written after the turn number changes: wait for turn 4's save, or this reads turn 3's (HP 95).
  await expect.poll(async () => (await newestAutosave(page))?.turn).toBe(4);
  const units = (await newestGameState<any>(page)).units;
  expect(units['0'].hp).toBe(100);
  expect(units['0'].standing_order).toBe('Sleep');
  await page.locator('[data-game-map]').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.unit-panel')).toHaveCount(0);
});


test('the turn button counts the selected last idle unit, then the selected idle city', async ({ page }) => {
  await expect(page.locator('.seatbar .cta')).toContainText('2 units need orders');
  await page.keyboard.press('Space');
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await expect(page.locator('.seatbar .cta')).toContainText('1 unit needs orders');
  await page.locator('.seatbar .cta').click();
  await expect(page.locator('.unit-panel .up-kind')).toHaveText('Scout');
  await page.keyboard.press('Space');
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  await page.locator('.seatbar .cta').click();
  await expect(page.locator('.city-panel')).toBeVisible();
  await expect(page.locator('.seatbar .cta')).toContainText('Choose production');
  const production = page.locator('.city-panel .production-choice:not(.unavailable) button.prod:enabled').first();
  await production.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.city-panel')).toHaveCount(0);
  await expect(page.locator('dialog.tech-modal')).toBeVisible();
  await page.keyboard.press('Escape');
  if (await page.locator('dialog.tech-modal').isVisible()) await page.keyboard.press('Escape');
  await expect(page.locator('[data-game-map]')).toBeFocused();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
});

type QueuedCard = { testId: 'era-takeover' | 'score-check' | 'meeting-takeover'; people?: string; era?: string; label: string };

async function mountInterstitialQueue(page: Page, queue: QueuedCard[]) {
  await page.goto('about:blank');
  await page.setContent('<div id="lane"></div>');
  await page.evaluate((cards) => {
    const lane = document.getElementById('lane')!;
    const dismissed: string[] = [];
    (window as unknown as { __dismissed: string[] }).__dismissed = dismissed;
    let index = 0;
    const render = () => {
      lane.replaceChildren();
      const card = cards[index];
      if (!card) return;
      const el = document.createElement('div');
      el.setAttribute('data-testid', card.testId);
      el.setAttribute('role', 'dialog');
      el.setAttribute('aria-label', card.label);
      if (card.people) el.setAttribute('data-people', card.people);
      if (card.era) el.setAttribute('data-era', card.era);
      el.textContent = card.label;
      if (card.testId === 'score-check') {
        const btn = document.createElement('button');
        btn.setAttribute('data-primary-action', '');
        btn.textContent = 'Continue';
        btn.addEventListener('click', advance);
        el.appendChild(btn);
      }
      lane.appendChild(el);
    };
    const advance = () => {
      const card = cards[index];
      if (!card) return;
      dismissed.push(`${card.testId}:${card.people ?? card.era ?? ''}`);
      index += 1;
      render();
    };
    window.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape') return;
      const card = cards[index];
      if (card && (card.testId === 'era-takeover' || card.testId === 'meeting-takeover')) {
        e.preventDefault();
        advance();
      }
    }, true);
    render();
  }, queue);
}

baseTest('dismissInterstitials drains consecutive meetings and era/score-to-meeting queues', async ({ page }) => {
  await mountInterstitialQueue(page, [
    { testId: 'meeting-takeover', people: 'rome', label: 'You meet Rome' },
    { testId: 'meeting-takeover', people: 'sparta', label: 'You meet Sparta' },
  ]);
  await dismissInterstitials(page);
  expect(await page.evaluate(() => (window as unknown as { __dismissed: string[] }).__dismissed)).toEqual([
    'meeting-takeover:rome',
    'meeting-takeover:sparta',
  ]);

  await mountInterstitialQueue(page, [
    { testId: 'era-takeover', era: '2', label: 'The Classical era begins' },
    { testId: 'score-check', era: '2', label: 'Score check: the Classical Age begins' },
    { testId: 'meeting-takeover', people: 'egypt', label: 'You meet Egypt' },
  ]);
  await dismissInterstitials(page);
  expect(await page.evaluate(() => (window as unknown as { __dismissed: string[] }).__dismissed)).toEqual([
    'era-takeover:2',
    'score-check:2',
    'meeting-takeover:egypt',
  ]);
});
