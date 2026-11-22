// Screen tour, diplomacy wording after the calls-only Alliance rules: an Alliance offer with its folded explanations
// open, and an empire row of an ally. Text is also written to test-results/diplomacy-text/<size>/ for the copy review.
// Run alone with `npx playwright test e2e/tour/diplomacy-text`. Shots land in test-results/screen-tour/<size>/diplomacy-text/.
import { mkdir, writeFile } from 'node:fs/promises';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { diplomacyState } from '../../src/test/diplomacyFixture';
import { createState, makeStandardSetup, meetAll } from '../../src/test/fixtureEngine';
import { revealSidebar } from '../sidebarAssertions';
import { SIZES, loadFixture, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('an Alliance offer with its explanations open', async ({ page }) => {
    const state = diplomacyState(10);
    state.diplomacy!.next_proposal_id = 1;
    state.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 9, respond_by: 12, relation_version: 0 }];
    await loadFixture(page, coreFixture(state), 10);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const card = page.locator('.offer-card');
    await expect(card).toBeVisible();
    for (const label of [/How this offer works/i, /Rules details/i]) {
      const summary = card.locator('summary', { hasText: label }).first();
      if (await summary.count()) await summary.click();
    }
    await page.mouse.move(5, size.height / 2);
    const dir = `test-results/diplomacy-text/${size.name}`;
    await mkdir(dir, { recursive: true });
    await writeFile(`${dir}/offer.txt`, await card.innerText());
    await shot(page, size, '01-alliance-offer-expanded');
  });

  test('an empire row of an ally', async ({ page }) => {
    const state = diplomacyState(20, { met: true });
    const rel = state.diplomacy!.relations.find((r) => (r.a === 0 && r.b === 1) || (r.a === 1 && r.b === 0))!;
    Object.assign(rel, { state: 'Alliance', effective_turn: 12, guarantee_until: 15, version: 1, security: { id: 1, started_turn: 12, protected_until: 22 } });
    await loadFixture(page, coreFixture(state), 20);
    await revealSidebar(page, 'standings');
    await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
    const inbox = page.getByRole('complementary', { name: 'Diplomacy inbox' });
    await expect(inbox).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    const dir = `test-results/diplomacy-text/${size.name}`;
    await mkdir(dir, { recursive: true });
    await writeFile(`${dir}/ally-row.txt`, await inbox.innerText());
    await shot(page, size, '02-ally-empire-row');
  });

  test('an incoming Peace offer names the allied war that continues', async ({ page }) => {
    const setup = makeStandardSetup('700040', 'Mongols', 'Byzantium', 'Mongol envoy', 'Byzantine envoy');
    setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: team === 0 }));
    setup.players.push({ ...setup.players[1], name: 'Spartan envoy', faction: 'Sparta', team: 2 });
    const state = createState(setup);
    state.turn = 86;
    meetAll(state);
    const relation = (a: number, b: number) => state.diplomacy!.relations.find((r) => r.a === a && r.b === b)!;
    Object.assign(relation(1, 2), { state: 'Alliance', version: 1, effective_turn: 55,
      security: { id: 1, started_turn: 55, protected_until: 65 } });
    for (const [a, b] of [[0, 1], [0, 2]]) Object.assign(relation(a, b), {
      state: 'War', version: 2, war_id: b, declarers: [b === 1 ? 1 : 0], effective_turn: b === 1 ? 74 : 85,
    });
    state.diplomacy!.next_treaty_id = 1;
    state.diplomacy!.next_war_id = 2;
    state.diplomacy!.next_proposal_id = 17;
    state.diplomacy!.proposals = [{ id: 17, kind: 'Peace', from: 1, to: 0, proposer: 1,
      created_turn: 86, respond_by: 88, relation_version: relation(0, 1).version, consents: [1] }];
    await loadFixture(page, coreFixture(state), 86);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const card = page.locator('.offer-card');
    await expect(card).toBeVisible();
    await expect(card.locator('.offer-scope')).toContainText('Ends your war with Byzantium.');
    await expect(card.locator('.offer-scope')).toContainText('Your war with Sparta continues.');
    await expect(card.getByRole('button', { name: /Accept Peace with Byzantium/ })).toBeVisible();
    await expect(card.getByRole('button', { name: /Decline Peace with Byzantium/ })).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03-peace-offer-two-empires');
  });
});
