// Scenarios end to end on the WASM core: open Scenarios from the title, start Sparta's Nicaea
// Strike (its window shortened to one turn by a test override), give a turn of orders, end the
// turn, reach the result, and check the run file the browser downloaded. A copy of the run goes
// to app/test-results/scenarios/ for tools/scenarios/compare.py.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { useGame } from '../src/store';
import { test, expect } from './fixtures';
import { acknowledgeWarDeclaredByYou, endPlanning } from './turnActions';

type GameWindow = Window & { legatum: typeof useGame };
const out = fileURLToPath(new URL('../test-results/scenarios/', import.meta.url));

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (!location.protocol.startsWith('http')) return;
    (globalThis as { legatumScenarioOverrides?: unknown }).legatumScenarioOverrides = { 'war-sparta-nicaea': { objective: { turns: 1 } } };
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  });
});

test('play a scenario turn to its result, leave feedback and save the run', async ({ page }) => {
  test.setTimeout(90_000);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
  // A card carries the question, the length, your best and the Hard AI's result from the same start.
  const card = page.getByRole('option', { name: /Nicaea Strike/ });
  await expect(card).toContainText('Not played yet');
  await expect(card).toContainText(/\?/);
  await expect(card).toContainText(/Hard AI: did it in \d+ turns?, \d+ of \d+ tries/);
  await expect(card).toContainText(/About \d+ min/);
  // The siege scenario ships as a playable fixture.
  await expect(page.getByRole('option', { name: /The Walls of Athens/ })).toBeVisible();
  await expect(page.getByTestId('lab-soon-siege-walls')).toHaveCount(0);
  // At 1000x600 only two rows of cards fit: a pill counts the ones below, and goes once the list is scrolled to its end.
  await page.setViewportSize({ width: 1000, height: 600 });
  const more = page.getByTestId('lab-more');
  await expect(more).toContainText(/^\d+ more below/);
  // The pill sits below the list, never over a card's last line (8.9.14 review).
  const gap = await page.evaluate(() => document.querySelector('[data-testid="lab-more"]')!.getBoundingClientRect().top - document.querySelector('.scn-scroll')!.getBoundingClientRect().bottom);
  expect(gap).toBeGreaterThanOrEqual(0);
  await page.locator('.scn-scroll').evaluate((el) => { el.scrollTop = el.scrollHeight; });
  await expect(more).toHaveCount(0);
  await page.locator('.scn-scroll').evaluate((el) => { el.scrollTop = 0; });
  await page.setViewportSize({ width: 1440, height: 1100 });
  await card.click();
  await expect(page.getByRole('heading', { name: "Sparta's Nicaea Strike", exact: true }).first()).toBeVisible();
  await expect(page.locator('.scn-briefing')).toContainText('Nicaea');
  // The briefing names the target and where it is (the user lost a run not knowing).
  await expect(page.getByTestId('scenario-briefing-target')).toContainText(/Take Nicaea: \d+ tiles? /);
  await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();

  // In play: the objective and the turns left are always on screen.
  const bar = page.getByTestId('scenario-bar');
  await expect(bar).toContainText('Take Nicaea');
  await expect(page.getByTestId('scenario-turns')).toHaveText('1 turn left');
  // ...and where the objective is, one click from the map centring on it.
  await expect(page.getByTestId('scenario-target')).toContainText('Take Nicaea');
  await page.getByTestId('scenario-show-target').click();
  await expect.poll(() => page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.focus?.hex ?? null;
  })).toEqual(await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.scenario!.status.targets!.find((t) => t.kind === 'Take')!.pos;
  }));
  await expect(page.locator('.seatbar .cta').first()).toBeVisible();

  // A turn of orders: declare war on Byzantium and march the army on Nicaea.
  await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    const view = s.views![s.seat];
    const target = view.cities.find((c) => c.name === 'Nicaea')!;
    const owner = view.players.find((p) => p.id === target.owner)!;
    const force = view.units.filter((u) => u.owner === view.me && u.kind !== 'Settler' && u.kind !== 'Scout'
      && Math.max(Math.abs(u.pos.q - target.pos.q), Math.abs(u.pos.r - target.pos.r)) <= 4);
    s.editOrders((orders) => [
      ...orders,
      { type: 'DeclareWar', against: owner.team, breach: false },
      ...force.map((u) => ({ type: 'MoveUnit' as const, unit: u.id, to: target.pos })),
    ]);
  });
  const firstDownload = page.waitForEvent('download');
  await endPlanning(page);

  const result = page.getByTestId('scenario-result');
  await expect(result).toBeVisible();
  await expect(result.getByRole('heading', { name: /^Objective (met|failed)$/ })).toBeVisible();
  await expect(page.getByTestId('scenario-score')).toContainText(/\d/);
  // You against the Hard AI from the same start, in words.
  await expect(page.getByTestId('scenario-vs')).toContainText(/You (took|did not meet)/);
  await expect(page.getByTestId('scenario-vs')).toContainText(/The Hard AI (took|met|failed)/);
  const saved = page.getByTestId('scenario-saved');
  await expect(saved).toContainText(/^Your run was downloaded: war-sparta-nicaea-[\dT-]+Z\.json$/);

  // The run file: start state, every order the human gave, the outcome and the full history.
  const finished = await firstDownload;
  expect(finished.suggestedFilename()).toMatch(/^war-sparta-nicaea-[\dT-]+Z\.json$/);
  const run = JSON.parse(await readFile(await finished.path(), 'utf8'));
  expect(run.format).toBe('legatum-scenario-run');
  expect(run.scenario).toMatchObject({ id: 'war-sparta-nicaea', turns: 1 });
  expect(run.scenario.question).toMatch(/\?$/);
  expect(run.initial_state.encoding).toBe('gzip-base64');
  expect(run.turns).toHaveLength(1);
  expect(run.turns[0].ms).toBeGreaterThanOrEqual(0);
  expect(run.turns[0].orders.some((o: { type: string }) => o.type === 'DeclareWar')).toBe(true);
  expect(run.turns[0].orders.filter((o: { type: string }) => o.type === 'MoveUnit').length).toBeGreaterThan(0);
  expect(run.score).toBe(run.outcome.score);
  expect(run.archive.tape.turns).toHaveLength(1);
  expect(run.feedback ?? null).toBeNull();

  // The feedback card: Fun, Fair, how it felt and a note. Nothing is saved until something is answered.
  await result.getByTestId('scenario-feel').click();
  const feedback = page.getByTestId('scenario-feedback');
  await expect(feedback).toBeVisible();
  await expect(page.getByTestId('scenario-save-feedback')).toBeDisabled();
  await feedback.getByRole('radiogroup', { name: 'Fun' }).getByRole('radio', { name: '4' }).click();
  await feedback.getByRole('radiogroup', { name: 'Fair' }).getByRole('radio', { name: '5' }).click();
  await feedback.getByRole('radio', { name: 'Too easy' }).click();
  await feedback.locator('textarea').fill('Four Hoplites at the gate felt like cheating.');
  const secondDownload = page.waitForEvent('download');
  await page.getByTestId('scenario-save-feedback').click();
  await expect(result.getByRole('heading', { name: 'Thank you' })).toBeVisible();
  // The answers download as the same run file again, now with the feedback.
  const again = await secondDownload;
  expect(again.suggestedFilename()).toBe(finished.suggestedFilename());
  const withFeedback = JSON.parse(await readFile(await again.path(), 'utf8'));
  expect(withFeedback.feedback?.text).toBe('Four Hoplites at the gate felt like cheating.');
  expect(withFeedback.feedback).toMatchObject({ fun: 4, fair: 5, felt: 'too-easy' });
  expect(withFeedback.archive.tape.turns).toHaveLength(1);
  await mkdir(out, { recursive: true });
  await writeFile(`${out}war-sparta-nicaea-e2e.json`, JSON.stringify(withFeedback, null, 1));

  // The scenario is over: no further turns, and the way back is the primary action.
  await result.getByRole('button', { name: 'View the map' }).click();
  await expect(page.getByTestId('scenario-turns')).toHaveText(/^Ended on turn/);
  await expect(page.locator('.seatbar .cta').first()).toContainText('Scenario over');
  await acknowledgeWarDeclaredByYou(page);
  await page.getByRole('button', { name: 'See result' }).first().click();
  await result.getByRole('button', { name: /^Back to scenarios/ }).click();
  await expect(page.getByRole('option', { name: /Nicaea Strike/ })).not.toContainText('Not played yet');
});

test('leaving a scenario part-way saves what was played as an unfinished run', async ({ page }) => {
  await page.addInitScript(() => {
    if (!location.protocol.startsWith('http')) return;
    (globalThis as { legatumScenarioOverrides?: unknown }).legatumScenarioOverrides = { 'war-sparta-nicaea': { objective: { turns: 3 } } };
  });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
  await page.getByRole('option', { name: /Nicaea Strike/ }).click();
  await page.getByRole('button', { name: /^Begin Sparta's Nicaea Strike/ }).click();
  await expect(page.getByTestId('scenario-bar')).toBeVisible();
  await endPlanning(page);
  await expect(page.getByTestId('scenario-turns')).toHaveText('2 turns left');
  const download = page.waitForEvent('download');
  await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().exitToTitle());
  const run = JSON.parse(await readFile(await (await download).path(), 'utf8'));
  expect(run).toMatchObject({ format: 'legatum-scenario-run', abandoned: true, finished_at: null });
  expect(run.turns).toHaveLength(1);
  expect(run.outcome.outcome).toBe('Playing');
});
