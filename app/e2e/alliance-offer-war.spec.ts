import { revealSidebar } from './sidebarAssertions';
// Playtest 8.9.2, turn 20 (user, 2026-10-17): "Switzerland offered me an alliance. I declined. Then I try to declare
// war and attack... at the end of that turn it asks me to respond to the offer again, then all my units get confused
// and I have to reissue their orders, which triggers the offer again." The app keeps one diplomatic order per empire,
// so the war replaced the decline and the offer came back as the primary; answering it replaced the war, and the
// attacks failed at Peace. A gzipped copy of his save (~/LegatumRuns/bug-saves/p892-alliance-loop-t20.legatum) is
// committed beside this spec and loaded as the autosave.
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';

type GameWindow = Window & { legatum: typeof useGame; legatumMap: { toScreen(hex: Hex): { x: number; y: number } } };
const save = () => gunzipSync(readFileSync(fileURLToPath(new URL('./saves/p892-alliance-loop-t20.current.legatum.gz', import.meta.url)))).toString('utf8');
// The Spearman beside Neuchâtel, which strikes the city's tile this turn.
const SPEARMAN = 3;
const SWISS_SPEARMAN: Hex = { q: 29, r: 22 };
const SWITZERLAND = 6;

async function open(page: Page) {
  await page.addInitScript((save) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', save);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, save());
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('20', { timeout: 90_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
}

const orders = (page: Page) => page.evaluate(() => {
  const s = (window as unknown as GameWindow).legatum.getState();
  return s.orders[s.seat];
});

async function mapPoint(page: Page, hex: Hex) {
  // jumpTo animates the camera; wait until the point settles.
  let last = { x: NaN, y: NaN };
  await expect.poll(async () => {
    const p = await page.evaluate((hex) => {
      const p = (window as unknown as GameWindow).legatumMap.toScreen(hex);
      const b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      return { x: p.x + b.left, y: p.y + b.top };
    }, hex);
    const still = Math.hypot(p.x - last.x, p.y - last.y) < 0.5;
    last = p;
    return still;
  }).toBe(true);
  return last;
}

test('declining Switzerland’s Alliance, then declaring War and attacking, never brings the offer back or drops the attack', async ({ page }) => {
  await open(page);
  const cta = page.locator('.seatbar .cta').first();
  await revealSidebar(page, 'standings');
  const inboxButton = page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ });
  const inbox = page.getByRole('complementary', { name: 'Diplomacy', exact: true });
  await expect(cta).toContainText('Respond to offer');
  await expect(inboxButton).toHaveText('Diplomacy (1)');

  // 1. Decline the offer: it is answered.
  await cta.click();
  await inbox.getByRole('button', { name: 'Decline', exact: true }).click();
  expect(await orders(page)).toEqual([{ type: 'RespondToProposal', proposal: 2, accept: false }]);
  await expect(cta).not.toContainText('Respond to offer');

  // 2. Order a Spearman onto a Swiss Spearman at Peace: Declare War and attack.
  await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    s.set({ diplomacyOpen: false, techOpen: false });
    s.select({ kind: 'unit', id }); s.jumpTo(v.units.find((u) => u.id === id)!.pos);
  }, SPEARMAN);
  const target = await mapPoint(page, SWISS_SPEARMAN);
  await page.mouse.click(target.x, target.y, { button: 'right' });
  await page.getByRole('dialog', { name: /^Declare War on / }).getByRole('button', { name: 'Declare War and attack' }).click();
  const plan = [
    { type: 'DeclareWar', against: SWITZERLAND, breach: false },
    { type: 'MoveUnit', unit: SPEARMAN, to: SWISS_SPEARMAN },
  ];
  await expect.poll(() => orders(page)).toEqual(plan);

  // The war ends the offer: no "Respond to offer", no inbox count, and the card says why.
  await expect(cta).not.toContainText('Respond to offer');
  await expect(inboxButton).toHaveText('Diplomacy');
  await inboxButton.click();
  await expect(inbox).toContainText('Ended by your War on');
  await expect(inbox.getByRole('button', { name: 'Decline', exact: true })).toHaveCount(0);
  // D (decline) in the inbox no longer answers anything, so it can't cancel the war.
  await page.keyboard.press('d');
  expect(await orders(page)).toEqual(plan);
  await page.keyboard.press('Escape');
  await expect(inbox).toHaveCount(0);

  // 3. End the turn: the war starts, the attack goes in, and the offer is gone.
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('21', { timeout: 90_000 });
  const after = await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState(), v = s.views![s.seat];
    const events = s.report?.events[s.seat] ?? [];
    return {
      battle: events.some((e) => e.type === 'Battle' && e.attacker === id),
      failed: events.filter((e) => e.type === 'OrderFailed').map((e) => e.type === 'OrderFailed' && e.reason),
      withdrawn: events.some((e) => e.type === 'ProposalInvalidated' && e.proposal.id === 2 && e.reason === 'War was declared'),
      offers: v.diplomacy.proposals.filter((p) => p.to === v.team).length,
    };
  }, SPEARMAN);
  expect(after).toEqual({ battle: true, failed: [], withdrawn: true, offers: 0 });
  await expect(cta).not.toContainText('Respond to offer');
});
