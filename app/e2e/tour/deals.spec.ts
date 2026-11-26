// Deals (rules 61, `GameSetup::deals`): the diplomacy panel with an empire at Peace (the Gold field on
// its offers and Offer Open Borders), an incoming Peace offer that asks for Gold, Give Gold to a met
// empire, Open Borders in force, and the turn report lines for a paid deal. Polish from live play: an empire
// not met stays hidden, a waiting offer says whose answer it waits for, a turned-down offer stays on its row,
// an offer button says which queued order it replaces, and the treaty-signing moment card.
// Run alone: `npx playwright test e2e/tour/deals`. Shots land in test-results/screen-tour/<size>/deals/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { diplomacyState } from '../../src/test/diplomacyFixture';
import { contextFromViews, formatEvents } from '../../src/format';
import type { Event, PeaceProposal } from '../../src/types';
import { SIZES, loadFixture, shot, showMoment, type GameWindow } from './helpers';
import { momentsForTurn } from '../../src/celebrate/moments';

test.describe.configure({ mode: 'parallel' });

const TURN = 12;

/** Two met empires at Peace with deals on; `edit` stages the case. */
function dealsSave(edit: (s: ReturnType<typeof diplomacyState>) => void = () => {}, strangers = false) {
  const s = diplomacyState(TURN, !(strangers) ? { met: true } : { strangers: true });
  s.setup.deals = true;
  s.players[0].gold = 120;
  s.players[1].gold = 300;
  edit(s);
  return coreFixture(s);
}

async function openDiplomacy(page: import('@playwright/test').Page) {
  await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ diplomacyOpen: true, diplomacyTeam: 1 }));
  const panel = page.getByRole('complementary', { name: 'Diplomacy', exact: true });
  await expect(panel).toBeVisible();
  return panel;
}

for (const size of SIZES) test.describe(`deals ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('an empire at Peace: Gold with the offer, Offer Open Borders, Give Gold', async ({ page }) => {
    await loadFixture(page, dealsSave(), TURN);
    const panel = await openDiplomacy(page);
    const field = panel.getByTestId('deal-gold');
    await expect(field).toBeVisible();
    await expect(panel.getByRole('button', { name: 'Offer Open Borders', exact: true })).toBeVisible();
    await field.getByRole('button', { name: 'You pay' }).click();
    await field.getByRole('button', { name: '10 Gold less' }).click();
    await expect(field.getByTestId('deal-forecast')).toHaveText('You pay 40 Gold when they accept. You have 120 Gold.');
    await field.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd1-peace-gold-field');
    await panel.getByRole('button', { name: 'Offer Open Borders', exact: true }).evaluate((el) => el.scrollIntoView({ block: 'center' }));
    await shot(page, size, 'd1b-offer-open-borders');

    // More Gold than the treasury holds: the offers are disabled with the reason in place.
    await field.getByRole('spinbutton').fill('200');
    await expect(field.getByTestId('deal-forecast')).toHaveText('You have 120 Gold, so offer at most 120.');
    await field.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd1c-offer-cant-pay');
    await field.getByRole('spinbutton').fill('40');

    await field.getByRole('button', { name: 'You ask' }).click();
    await expect(field.getByTestId('deal-forecast')).toHaveText('They pay you 40 Gold when they accept.');
    await expect(panel.getByRole('button', { name: 'Offer Open Borders', exact: true }).locator('[data-gain-chip]')).toContainText('+40');
    await field.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd1d-you-ask');
    await panel.getByRole('button', { name: 'Offer Open Borders', exact: true }).click();
    await expect(panel.getByRole('button', { name: 'Queued: Offer Open Borders', exact: true })).toHaveAttribute('aria-pressed', 'true');
    await expect(panel.getByTestId('diplomacy-queued')).toHaveText('Queued: Offer Open Borders. You ask 40 Gold. Undo');
    const errors = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return { orders: s.orders[s.seat], errors: s.errors[s.seat] }; });
    expect(errors.orders).toEqual([{ type: 'ProposeTreaty', to: 1, kind: 'OpenBorders', gold: -40 }]);
    expect.soft(errors.errors, 'the offer validates').toEqual([]);
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd2-open-borders-offer-queued');

    const gift = panel.getByTestId('rival-gift');
    await gift.scrollIntoViewIfNeeded();
    await expect(gift.getByTestId('gift-forecast')).toContainText('You keep');
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd3-give-gold');
    await gift.getByTestId('give-gold').click();
    await expect(gift.getByTestId('gift-queued')).toBeVisible();
    const giftErrors = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.errors[s.seat]; });
    expect.soft(giftErrors, 'the gift validates').toEqual([]);
    await shot(page, size, 'd3b-give-gold-queued');
  });

  test('an incoming Peace offer that asks for Gold', async ({ page }) => {
    await loadFixture(page, dealsSave((s) => {
      const d = s.diplomacy!;
      d.relations = d.relations.map((r) => ({ ...r, state: 'War', war_id: 1, declarers: [1], effective_turn: TURN - 4, guarantee_until: 0, version: 1 }));
      d.next_war_id = 2;
      d.next_proposal_id = 2;
      const offer: PeaceProposal = { id: 1, from: 1, to: 0, proposer: 1, kind: 'Peace', created_turn: TURN, respond_by: TURN + 1, relation_version: 1, gold: -40, consents: [1] };
      d.proposals = [offer];
    }), TURN);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const card = page.locator('.offer-card');
    await expect(card).toBeVisible();
    await expect(card.locator('.offer-headline')).toContainText('asks 40 Gold for Peace');
    await expect(card.locator('.offer-for-you')).toContainText('You pay 40 Gold.');
    await expect(card.locator('.offer-for-you')).not.toContainText('You have 120 Gold');
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd4-incoming-peace-asks-gold');
  });

  test('Open Borders in force, and the report lines for a paid deal', async ({ page }) => {
    await loadFixture(page, dealsSave((s) => { (s.diplomacy as { open_borders?: [number, number][] }).open_borders = [[0, 1]]; }), TURN);
    const panel = await openDiplomacy(page);
    await expect(panel.locator('.empire-relation.open-borders')).toHaveText('Open Borders');
    await expect(page.getByTestId('rivals-open-borders').getByRole('img', { name: 'Open Borders emblem' })).toBeVisible();
    await expect(panel.getByRole('button', { name: 'Close borders', exact: true })).toBeVisible();
    await panel.locator('.empire-row.open').evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd5-open-borders-active');

    await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().set({ diplomacyOpen: false }));
    const view = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
    const proposal: PeaceProposal = { id: 3, from: 1, to: 0, proposer: 1, kind: 'OpenBorders', created_turn: TURN - 2, respond_by: TURN, relation_version: 0, gold: 40 };
    const events: Event[] = [
      { type: 'ProposalAccepted', proposal, effective_turn: TURN },
      { type: 'GoldGiven', from: 1, to: 0, amount: 40 },
      { type: 'BordersOpened', a: 0, b: 1, effective_turn: TURN },
      { type: 'BordersClosed', a: 0, b: 1, effective_turn: TURN, reason: 'Switzerland closed the borders' },
    ];
    const report = { turn: TURN, hash: '', events: [events], moments: [[]], alarms: [[]], lines: [formatEvents(events, contextFromViews(view, view))] };
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], reportOpen: true }), report);
    const turnReport = page.getByRole('complementary', { name: 'Turn report' });
    await expect(turnReport).toContainText('40 Gold for Open Borders');
    await expect(turnReport).toContainText('Open Borders with Switzerland begin.');
    await expect(turnReport).toContainText('Switzerland closed the borders.');
    await expect(turnReport.locator('.rline-tone .te.openborders')).toHaveCount(2);
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd6-report-deal-lines');
  });

  test('polish: an empire not met is hidden, as in the rivals table', async ({ page }) => {
    await loadFixture(page, dealsSave(() => {}, true), TURN);
    const panel = await openDiplomacy(page);
    await expect(panel.locator('.empire-pick')).toHaveCount(0);
    await expect(panel.getByTestId('unmet-empires')).toHaveText('?1 empire not met yet. Explore to meet them: offers need contact first.');
    await expect(page.locator('.rv-row.unmet')).toContainText('Not met yet');
    await panel.getByTestId('unmet-empires').scrollIntoViewIfNeeded();
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd7-unmet-hidden');
  });

  test('polish: our offer waits for an answer; a rejected offer stays on the row', async ({ page }) => {
    await loadFixture(page, dealsSave((s) => {
      s.diplomacy!.next_proposal_id = 2;
      s.diplomacy!.proposals = [{ id: 1, from: 0, to: 1, proposer: 0, kind: 'OpenBorders', created_turn: TURN - 1, respond_by: TURN + 1, relation_version: 0 }];
    }), TURN);
    // Last turn the Swiss turned down our Alliance offer (the session's turn reports keep it).
    const rejected: PeaceProposal = { id: 0, from: 0, to: 1, proposer: 0, kind: 'Alliance', created_turn: TURN - 3, respond_by: TURN - 2, relation_version: 0 };
    const history = [{ turn: TURN - 1, hash: '', lines: [[]], events: [[{ type: 'ProposalRejected', proposal: rejected, effective_turn: TURN - 1 } as Event]] }];
    await page.evaluate((reportHistory) => (window as unknown as GameWindow).legatum.getState().set({ reportHistory }), history);
    const panel = await openDiplomacy(page);
    const row = panel.locator('.empire-row.open');
    await expect(row.getByTestId('offer-outcome')).toHaveText(`Rejected your Alliance offer · turn ${TURN - 1}`);
    await expect(row.locator('.diplomacy-unavailable')).toHaveText(`Not now: Offer Alliance, Offer Open Borders. Waiting for Switzerland’s answer to your Open Borders offer, due turn ${TURN + 1}.`);
    await row.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd8-offer-waits-rejected-row');
  });

  test('polish: with an offer queued, the other buttons say they replace it', async ({ page }) => {
    await loadFixture(page, dealsSave(), TURN);
    const panel = await openDiplomacy(page);
    await panel.getByRole('button', { name: 'Offer Open Borders', exact: true }).click();
    const alliance = panel.locator('.diplomacy-action', { has: page.getByRole('button', { name: 'Offer Alliance', exact: true }) });
    await expect(alliance.getByTestId('diplomacy-replaces')).toHaveText('Replaces your Open Borders offer');
    await expect(panel.locator('.diplomacy-hostile').getByTestId('diplomacy-replaces')).toHaveText('Replaces your Open Borders offer');
    await alliance.evaluate((el) => el.scrollIntoView({ block: 'center' }));
    await page.mouse.move(4, size.height / 2);
    await shot(page, size, 'd9-replaces-queued-offer');
  });

  for (const [kind, gold, name] of [['OpenBorders', 40, 'd10-treaty-moment-gold'], ['Peace', -60, 'd10b-treaty-moment-peace-paid'], ['Alliance', 0, 'd10c-treaty-moment-alliance']] as const) {
    test(`polish: the treaty-signing moment (${kind}${!(gold) ? '' : `, ${gold} Gold`})`, async ({ page }) => {
      await loadFixture(page, dealsSave(), TURN);
      const v = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
      const proposal: PeaceProposal = { id: 3, from: 0, to: 1, proposer: 0, kind, created_turn: TURN - 2, respond_by: TURN - 1, relation_version: 0, ...(!(gold) ? {} : { gold: -gold }) };
      const relation = { ...v.diplomacy.relations[0], state: !(kind === 'Alliance') ? 'Peace' as const : 'Alliance' as const, effective_turn: TURN, guarantee_until: TURN + 10 };
      const events: Event[] = [
        { type: 'ProposalAccepted', proposal, effective_turn: TURN },
        ...(!(gold) ? [] : [{ type: 'GoldGiven' as const, from: !(gold > 0) ? 0 : 1, to: !(gold > 0) ? 1 : 0, amount: Math.abs(gold) }]),
        !(kind === 'OpenBorders') ? !(kind === 'Alliance') ? { type: 'PeaceEstablished', relation } : { type: 'TreatyActivated', relation } : { type: 'BordersOpened', a: 0, b: 1, effective_turn: TURN },
      ];
      const report = { turn: TURN - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
      await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [], reportOpen: false }), report);
      const title = `${!(kind === 'OpenBorders') ? kind : 'Open Borders'} with Switzerland`;
      await showMoment(page, title);
      const card = page.locator('.moment-card', { hasText: title });
      await expect(card.getByTestId('moment-treaty').locator('.medallion')).toHaveCount(2);
      if (!(gold)) {} else { await expect(card.getByTestId('moment-treaty-gold')).toHaveText(!(gold > 0) ? '−60 Gold to Switzerland' : '+40 Gold from Switzerland'); }
      await page.mouse.move(4, size.height / 2);
      await page.waitForTimeout(700); // let the emblem's seal settle
      await shot(page, size, name);
    });
  }
});
