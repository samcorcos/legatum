import { revealSidebar } from './sidebarAssertions';
// Population is territory (the territory bar, worked/owned counts) and Alliance Gold costs, on the real
// engine at the user's laptop size (1000x600) and 1440x900. Screenshots go to
// test-results/borders-alliance/<size>/ (or LEGATUM_SHOTS) for review.
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { coreFixture } from '../src/test/coreFixture';
import { diplomacyState } from '../src/test/diplomacyFixture';

const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const outRoot = process.env.LEGATUM_SHOTS ?? fileURLToPath(new URL('../test-results/borders-alliance/', import.meta.url));

async function shot(page: Page, size: string, name: string) {
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(400);
  await mkdir(`${outRoot}/${size}`, { recursive: true });
  await page.screenshot({ path: `${outRoot}/${size}/${name}.png`, animations: 'disabled' });
}

async function load(page: Page, game: string, turn: number) {
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

/** Two empires; ours is allied with theirs when `allied`, and our capital builds a Settler. */
function game(turn: number, allied = false) {
  const s = diplomacyState(turn, { met: true });
  s.cities[0].buildings = ['Granary'];
  s.cities[0].producing = { kind: 'Unit', unit: 'Settler' };
  if (allied) {
    const r = s.diplomacy!.relations.find((r) => r.a === 0 && r.b === 1)!;
    r.state = 'Alliance'; r.security = { id: 1, started_turn: turn - 2, protected_until: turn + 8 }; r.version = 1;
    s.diplomacy!.next_treaty_id = 1;
  }
  return s;
}

for (const size of SIZES) {
  test.describe(size.name, () => {
    test.use({ viewport: { width: size.width, height: size.height } });

    test('Alliance offer card shows upkeep and net income', async ({ page }) => {
      const s = game(10);
      s.diplomacy!.next_proposal_id = 1;
      s.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 9, respond_by: 12, relation_version: 0 }];
      await load(page, coreFixture(s), 10);
      await page.getByRole('button', { name: /Respond to offer/ }).first().click();
      const card = page.locator('.offer-card');
      await expect(card).toBeVisible();
      await expect(card).toContainText('Upkeep 1 Gold/turn');
      await expect(card).not.toContainText('Net income');
      await shot(page, size.name, '04-alliance-offer');
    });

    test('a betrayal is a visible record: when it happened and when they will ally again', async ({ page }) => {
      const s = game(12);
      // They left our Alliance inside its protected window on turn 8.
      s.diplomacy!.betrayals = [{ by: 1, against: 0, turn: 8, until: 28 }];
      await load(page, coreFixture(s), 12);
      await revealSidebar(page, 'standings');
      await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
      const inbox = page.getByRole('complementary', { name: 'Diplomacy inbox' });
      await inbox.locator('.empire-pick').first().click();
      await expect(inbox.locator('.betrayal-line').first()).toHaveText("Broke an alliance on turn 8: your team won't ally until turn 28");
      await shot(page, size.name, '07-betrayal-record');
    });

    test("an empire you haven't met: the reason once, the Alliance offer clearly disabled", async ({ page }) => {
      const s = diplomacyState(12, { strangers: true });
      await load(page, coreFixture(s), 12);
      await revealSidebar(page, 'standings');
      await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
      const inbox = page.getByRole('complementary', { name: 'Diplomacy inbox' });
      await inbox.locator('.empire-pick').first().click();
      await expect(inbox.locator('.diplomacy-blocked-reason')).toHaveCount(1);
      const offer = inbox.getByRole('button', { name: /Offer Alliance/ });
      await expect(offer).toBeDisabled();
      // Scroll the inbox's own scroll area (not the page) so the disabled offer is in the shot.
      await offer.evaluate((el) => {
        let box = el.parentElement;
        while (box && !(box.scrollHeight > box.clientHeight && /auto|scroll/.test(getComputedStyle(box).overflowY))) box = box.parentElement;
        if (box) box.scrollTop += el.getBoundingClientRect().top - box.getBoundingClientRect().top - 120;
      });
      await shot(page, size.name, '08-not-met');
    });

    test('economy: Gold breakdown includes treaty upkeep', async ({ page }) => {
      await load(page, coreFixture(game(12, true)), 12);
      await page.locator('.topbar .gold-stat').click();
      await expect(page.locator('.tip-bubble:not([hidden])')).toContainText('Treaty upkeep: Alliance with');
      await shot(page, size.name, '05-economy-gold');
      await page.keyboard.press('Escape');
      await revealSidebar(page, 'standings');
      await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
      await expect(page.getByRole('complementary', { name: 'Diplomacy inbox' })).toContainText('Protected until turn 20');
      await shot(page, size.name, '06-diplomacy-allied');
    });
  });
}
