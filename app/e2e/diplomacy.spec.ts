import { revealSidebar } from './sidebarAssertions';
import type { Page } from '@playwright/test';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { diplomacyCoreSave, diplomacyState } from '../src/test/diplomacyFixture';
import { coreFixture } from '../src/test/coreFixture';
import { callToArmsCoreSave, leaderAllianceOfferCoreSave } from '../src/test/allianceFixture';

/** Continue a saved game at `turn` (first-game hints off). */
async function loadSave(page: Page, game: string, turn: number) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
}

/** Turn 16 in the Standard save: the Swiss Alliance offer awaits our reply. */
function allianceOfferSave(gold = 1000) {
  const state = diplomacyState(16, { met: true });
  state.players[0].gold = gold;
  state.diplomacy!.next_proposal_id = 1;
  state.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 16, respond_by: 18, relation_version: 0 }];
  return coreFixture(state);
}

test('Diplomacy: an incoming Peace offer opens the inbox and resolves next turn', async ({ page }) => {
  const state = diplomacyState(3);
  Object.assign(state.diplomacy!.relations[0], {
    state: 'War', war_id: 1, effective_turn: 1, version: 1, declarers: [0],
  });
  state.diplomacy!.next_war_id = 1;
  state.diplomacy!.next_proposal_id = 2;
  state.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Peace',
    created_turn: 3, respond_by: 5, relation_version: 1, consents: [1] }];
  await loadSave(page, coreFixture(state), 3);
  await revealSidebar(page, 'standings');
  const inbox = page.getByRole('button', { name: /^Diplomacy inbox/ });
  await expect(page.locator('.vp-row .vhc-rel.war')).toBeVisible();
  await expect(inbox).toHaveText('Diplomacy inbox (1)');
  await expect(page.getByRole('button', { name: /^Respond to offer/ })).toBeVisible();
  await inbox.click();
  const accept = page.getByRole('button', { name: /^Accept Peace with Switzerland/ });
  await expect(accept).toHaveClass(/primary/);
  await expect(page.getByRole('button', { name: /^Decline Peace with Switzerland/ })).not.toHaveClass(/primary/);
  await accept.click();
  // G17: answering the only offer closes the inbox and confirms it over the map.
  await expect(page.getByRole('complementary', { name: 'Diplomacy inbox' })).toHaveCount(0);
  await expect(page.locator('.decision-notice')).toContainText('Accepted Peace with Switzerland');
  await expect(page.getByRole('button', { name: /^Respond to offer/ })).toHaveCount(0);
  await expect(inbox).toHaveText('Diplomacy inbox');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('4');
  await expect(inbox).toHaveText('Diplomacy inbox');
  await expect(page.locator('.vp-row .vhc-rel.peace')).toBeVisible();
});

for (const [width, height] of [[1000, 600], [1440, 900]] as const) {
  test(`Diplomacy: an empire you haven't met can't be offered an Alliance; the row says why once (${width}×${height})`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await loadSave(page, diplomacyCoreSave(1, { strangers: true }), 1);
    await revealSidebar(page, 'standings');
    await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
    const panel = page.getByRole('complementary', { name: 'Diplomacy inbox' });
    const offer = panel.getByRole('button', { name: 'Offer Alliance', exact: true });
    await expect(offer).toBeDisabled();
    // Disabled looks disabled: a dashed outline and a not-allowed cursor.
    await expect(offer).toHaveCSS('border-top-style', 'dashed');
    await expect(offer).toHaveCSS('cursor', 'not-allowed');
    const reason = panel.locator('.diplomacy-blocked-reason');
    await expect(reason).toHaveCount(1);
    await expect(reason).toContainText("You haven't met Switzerland yet");
    await reason.scrollIntoViewIfNeeded();
    // Declaring war needs no contact, so it stays available.
    await expect(panel.getByRole('button', { name: 'Declare War', exact: true })).toBeEnabled();
    if (process.env.LEGATUM_SHOTS) await page.screenshot({ path: `${process.env.LEGATUM_SHOTS}/unmet-offer-${width}x${height}.png` });
  });
}

test('Diplomacy: an outgoing offer never counts in the inbox badge', async ({ page }) => {
  await loadSave(page, diplomacyCoreSave(), 1);
  await revealSidebar(page, 'standings');
  const inbox = page.getByRole('button', { name: /^Diplomacy inbox/ });
  await inbox.click();
  await page.getByRole('button', { name: 'Offer Alliance', exact: true }).click();
  await page.getByRole('button', { name: 'Close diplomacy', exact: true }).click();
  await expect(inbox).toHaveText('Diplomacy inbox');
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  const closeReport = page.getByRole('button', { name: 'Close report', exact: true });
  if (await closeReport.isVisible()) await closeReport.click();
  // Our Alliance offer is pending with the AI; nothing awaits our response.
  await inbox.click();
  await expect(page.getByRole('complementary', { name: 'Diplomacy inbox' })).toContainText('Alliance offered to');
  await page.getByRole('button', { name: 'Close diplomacy', exact: true }).click();
  await expect(inbox).toHaveText('Diplomacy inbox');
  await expect(page.getByRole('button', { name: /^Respond to offer/ })).toHaveCount(0);
});

test('Diplomacy: the inbox and the first-game hint share the story area, one at a time (1280×800)', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('424242');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1');
  const hint = page.getByTestId('first-game-hint');
  await expect(hint).toBeVisible();
  await revealSidebar(page, 'standings');
  await page.getByRole('button', { name: /^Diplomacy inbox/ }).click();
  const inbox = page.getByRole('complementary', { name: 'Diplomacy inbox' });
  await expect(inbox).toBeVisible();
  await expect(hint).toHaveCount(0);
  await inbox.getByRole('button', { name: 'Close diplomacy', exact: true }).click();
  await expect(hint).toBeVisible();
});

test('Diplomacy: a human offers an alliance and sees the AI decision and its reasons', async ({ page }) => {
  await loadSave(page, diplomacyCoreSave(16), 16);
  await revealSidebar(page, 'standings');
  const inbox = page.getByRole('button', { name: /^Diplomacy inbox/ });
  await inbox.click();
  const offer = page.getByRole('button', { name: 'Offer Alliance', exact: true });
  // Upkeep is itemised once, beneath the button, before it is sent.
  await expect(offer.locator('.cost-chip')).toHaveCount(0);
  await expect(page.getByText(/Upkeep \d+ Gold\/turn/).first()).toBeVisible();
  await offer.click();
  await expect(page.getByRole('button', { name: /Queued: Offer Alliance/ })).toBeVisible();
  await page.getByRole('button', { name: 'Close diplomacy', exact: true }).click();
  for (const turn of [17, 18]) {
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText(String(turn));
    const close = page.getByRole('button', { name: 'Close report', exact: true });
    if (await close.isVisible()) await close.click();
  }
  await inbox.click();
  const panel = page.getByRole('complementary', { name: 'Diplomacy inbox' });
  await expect(panel).toContainText(/(accepted|declined) Alliance/);
});

// Playtest 3 (G19): "there should be a way to use keyboard to accept or decline instead of making me
// click each time." D declines, the inbox moves on (here: closes), the notice confirms; Z undoes.
test('Diplomacy: D declines the offer from the keyboard, and Z undoes it (G19)', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await loadSave(page, allianceOfferSave(), 16);
  const respond = page.getByRole('button', { name: /^Respond to offer/ });
  await respond.click();
  const panel = page.getByRole('complementary', { name: 'Diplomacy inbox' });
  const card = panel.locator('.offer-card');
  // The keys are on the buttons.
  await expect(card.getByRole('button', { name: 'Accept Alliance', exact: true }).locator('.kbd')).toHaveText('A');
  await expect(card.getByRole('button', { name: 'Decline', exact: true }).locator('.kbd')).toHaveText('D');
  await page.keyboard.press('d');
  await expect(panel).toHaveCount(0);
  const notice = page.locator('.decision-notice');
  await expect(notice).toContainText('Alliance from Switzerland declined');
  await expect(respond).toHaveCount(0);
  await page.keyboard.press('z');
  await expect(notice).toHaveCount(0);
  await expect(card.getByRole('button', { name: 'Accept Alliance', exact: true })).toHaveClass(/primary/);
  await expect(respond).toBeVisible();
  // Return never accepts a costly offer: it closes the inbox (G15).
  await page.keyboard.press('Enter');
  await expect(panel).toHaveCount(0);
  await expect(respond).toBeVisible();
});

// Playtest G4: a first-time player must see who is offering, where they are, and the one decision.
for (const size of [{ width: 1440, height: 900 }, { width: 1000, height: 600 }]) test.describe(`alliance decision card ${size.width}×${size.height}`, () => {
  test.use({ viewport: size });
  test(`Diplomacy: an empty treasury can accept an Alliance (${size.width}×${size.height})`, async ({ page }, testInfo) => {
    await loadSave(page, allianceOfferSave(12), 16);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const card = page.locator('.offer-card');
    await expect(card.getByRole('button', { name: 'Accept Alliance', exact: true })).toBeEnabled();
    await expect(card).toContainText('Upkeep 1 Gold/turn');
    await expect(card.getByRole('button', { name: 'Decline', exact: true })).toBeEnabled();
    await page.screenshot({ path: testInfo.outputPath('alliance-no-signing-cost.png') });
  });

  test(`Diplomacy: an alliance offer is one decision card with the empire's identity and its Gold costs (${size.width}×${size.height})`, async ({ page }) => {
    await loadSave(page, allianceOfferSave(), 16);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const panel = page.getByRole('complementary', { name: 'Diplomacy inbox' });
    const card = panel.locator('.offer-card');
    await expect(card).toHaveCount(1);
    // Headline: swatch in the empire's map color (Switzerland = deep teal; colours follow the faction since 2026-11-11), faction and seat.
    const headline = card.locator('.offer-headline');
    await expect(headline).toContainText('Switzerland offers an Alliance');
    await expect(headline.locator('.empire-mark .player-badge')).toHaveCSS('background-color', 'rgb(31, 163, 168)');
    await expect(card.locator('.empire-where')).toContainText('Nearest city Harmonia ~');
    await expect(card.getByRole('button', { name: 'Show on map' })).toBeVisible();
    // G16: the upkeep, as a red cost (no net income line: designer, 2026-11-07).
    const forYou = card.locator('.offer-for-you li');
    await expect(forYou.nth(0)).toContainText('Upkeep 1 Gold/turn');
    await expect(card).not.toContainText('Net income');
    // The proposer is a face with a quote and the treaty's emblem; costs are red-ringed medallions; one primary.
    await expect(card.locator('.offer-portrait img')).toBeVisible();
    await expect(card.locator('.offer-portrait .te.alliance')).toBeVisible();
    await expect(card.locator('.offer-quote')).toContainText('Guard the passes. Keep the accounts.');
    await expect(card.locator('.offer-for-you .for-you-line.cost .rm.bad').first()).toBeVisible();
    await expect(card.locator('.btn.primary')).toHaveCount(1);
    const accept = card.getByRole('button', { name: 'Accept Alliance', exact: true });
    expect(await card.locator('.offer-for-you').first().evaluate((el) => !!(el.compareDocumentPosition(el.closest('.offer-card')!.querySelector('.offer-actions')!) & Node.DOCUMENT_POSITION_FOLLOWING)))
      .toBe(true);
    // The costs are the red lines above, once: the button carries no second chip for them.
    await expect(accept.locator('.cost-chip')).toHaveCount(0);
    await expect(card.getByRole('button', { name: 'Decline', exact: true }).locator('.cost-chip')).toHaveCount(0);
    await expect(card).toContainText('Respond by turn 18');
    // Accept and Decline are on screen without scrolling, and nothing covers them.
    for (const name of ['Accept Alliance', 'Decline']) {
      const button = card.getByRole('button', { name, exact: true });
      await expect(button).toBeVisible();
      await expect.poll(() => button.evaluate((el) => {
        const b = el.getBoundingClientRect(), scroller = el.closest('.report-body')!;
        return scroller.scrollTop === 0 && b.top >= 0 && b.bottom <= innerHeight && b.right <= innerWidth
          && el.contains(document.elementFromPoint(b.left + b.width / 2, b.top + b.height / 2));
      })).toBe(true);
    }
    await expect(accept).toHaveClass(/primary/);
    // Every other rule is reachable but starts collapsed.
    const rules = card.locator('details.rules-details');
    await expect(rules).not.toHaveAttribute('open', /.*/);
    await expect(rules.getByText(/A betrayed team won’t ally/)).toBeHidden();
    await rules.locator('summary').click();
    await expect(rules.getByText(/A betrayed team won’t ally/)).toBeVisible();
    await accept.click();
    // G17: the inbox closes; reopened, the card shows the answer as its state, with Undo.
    await expect(panel).toHaveCount(0);
    await page.evaluate(() => (window as unknown as { legatum: { getState(): { set(p: object): void } } }).legatum.getState().set({ diplomacyOpen: true }));
    await expect(card.locator('.offer-answer')).toContainText('Accepted: Alliance starts turn 17');
    await expect(card.locator('.btn.primary')).toHaveCount(0);
    await expect(card.getByRole('button', { name: 'Decline instead', exact: true })).toBeVisible();
  });

  test(`Diplomacy: an Alliance offer against the leader shows upkeep and an empty treasury can accept it (${size.width}×${size.height})`, async ({ page }, testInfo) => {
    await loadSave(page, leaderAllianceOfferCoreSave(), 16);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const card = page.locator('.offer-card');
    await expect(card).toHaveCount(1);
    const forYou = card.locator('.offer-for-you li');
    await expect(forYou.filter({ hasText: 'Upkeep 1 Gold/turn' })).toHaveCount(1);
    const accept = card.getByRole('button', { name: 'Accept Alliance', exact: true });
    await expect(accept).toBeEnabled();
    await expect(accept).toHaveClass(/primary/);
    await expect(accept.locator('.cost-chip')).toHaveCount(0);
    for (const name of ['Accept Alliance', 'Decline']) {
      const button = card.getByRole('button', { name, exact: true });
      await expect(button).toBeVisible();
      await expect.poll(() => button.evaluate((el) => { const b = el.getBoundingClientRect(); return b.top >= 0 && b.bottom <= innerHeight; })).toBe(true);
    }
    await page.screenshot({ path: testInfo.outputPath('alliance-offer.png') });
    await card.locator('details.rules-details summary').click();
    await expect(card.locator('details.rules-details')).toContainText('The Alliance starts in turn');
  });

  // Playtest 3 (G17): "when i pressed 'decline' it switched to 'queued' but then the window was
  // still open so it felt like it didn't necessarily work?"
  test(`Diplomacy: Decline closes the inbox with a notice, and its Undo restores the offer (${size.width}×${size.height})`, async ({ page }) => {
    await loadSave(page, allianceOfferSave(), 16);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const panel = page.getByRole('complementary', { name: 'Diplomacy inbox' });
    await panel.locator('.offer-card').getByRole('button', { name: 'Decline', exact: true }).click();
    await expect(panel).toHaveCount(0);
    const notice = page.locator('.decision-notice');
    await expect(notice).toContainText('Alliance from Switzerland declined');
    await expect(page.getByRole('button', { name: /Respond to offer/ })).toHaveCount(0);
    await notice.getByRole('button', { name: 'Undo', exact: true }).click();
    await expect(notice).toHaveCount(0);
    // The offer is back: the inbox reopens on it, unanswered, with Accept as the primary again.
    await expect(panel.locator('.offer-card').getByRole('button', { name: 'Accept Alliance', exact: true })).toHaveClass(/primary/);
    await expect(panel.locator('.offer-answered')).toHaveCount(0);
  });
});

// An ally's war is a call to arms: Join is the one primary action, Decline sits beside it, and the Alliance stays either way.
for (const size of [{ width: 1440, height: 900 }, { width: 1000, height: 600 }]) test.describe(`call to arms ${size.width}×${size.height}`, () => {
  test.use({ viewport: size });
  test(`Diplomacy: a call to arms offers Join and Decline, and declining keeps the Alliance (${size.width}×${size.height})`, async ({ page }) => {
    await loadSave(page, callToArmsCoreSave(), 16);
    await expect(page.getByRole('button', { name: /Answer call to arms/ }).first()).toBeVisible();
    await page.getByRole('button', { name: /Answer call to arms/ }).first().click();
    const card = page.locator('.call-card');
    await expect(card).toHaveCount(1);
    await expect(card.locator('.offer-headline')).toContainText(/Your ally .+ calls you to war with .+/);
    await expect(card.locator('.btn.primary')).toHaveCount(1);
    const join = card.getByRole('button', { name: /^Join the war on/ });
    const decline = card.getByRole('button', { name: 'Decline', exact: true });
    await expect(join).toHaveClass(/primary/);
    await expect(decline).not.toHaveClass(/primary/);
    for (const button of [join, decline]) {
      await expect(button).toBeVisible();
      await expect.poll(() => button.evaluate((el) => { const b = el.getBoundingClientRect(); return b.top >= 0 && b.bottom <= innerHeight; })).toBe(true);
    }
    await expect(card).toContainText('Declining keeps your Alliance');
    await expect(card).toContainText('Declining costs nothing');
    await decline.click();
    await expect(page.getByRole('complementary', { name: 'Diplomacy inbox' })).toHaveCount(0);
    // The decision is settled: the turn can end without the call blocking it.
    await expect(page.getByRole('button', { name: /Answer call to arms/ })).toHaveCount(0);
  });
});
