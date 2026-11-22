import { test, expect } from '../fixtures';
import { allianceCoreSave } from '../../src/test/allianceFixture';
import { warPromptCoreSave, PURSUIT } from '../../src/test/warPromptFixture';
import { distance, neighbors } from '../../src/hex';
import { SIZES, hexPoint, loadFixture, selectThing, shot } from './helpers';

for (const size of SIZES) test.describe(`capture and Peace clarity ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('Peace names the allied war that continues', async ({ page }) => {
    const save = JSON.parse(allianceCoreSave());
    save.players[1].name = 'Byzantium'; save.players[1].faction = 'Byzantium';
    save.players[2].name = 'Sparta'; save.players[2].faction = 'Sparta';
    const relation = save.diplomacy.relations.find((r: { a: number; b: number }) => r.a === 0 && r.b === 1);
    save.diplomacy.proposals = [{ id: 7, kind: 'Peace', from: 1, to: 0, proposer: 1,
      created_turn: 50, respond_by: 52, relation_version: relation.version, consents: [1] }];
    save.diplomacy.next_proposal_id = 8;
    await loadFixture(page, JSON.stringify(save), 50);
    await page.getByRole('button', { name: /Respond to offer/ }).first().click();
    const card = page.locator('.offer-card');
    await expect(card.locator('.offer-scope')).toContainText('Ends your war with Byzantium.');
    await expect(card.locator('.offer-scope')).toContainText('Your war with Sparta continues.');
    await expect(card.getByRole('button', { name: /^Accept Peace with Byzantium/ })).toBeVisible();
    await expect(card.getByRole('button', { name: /^Decline Peace with Byzantium/ })).toBeVisible();
    const layout = await card.evaluate((el) => {
      const buttons = [...el.querySelectorAll<HTMLButtonElement>('.offer-actions button')];
      const rows = [...el.querySelectorAll<HTMLElement>('.offer-for-you li')];
      return { buttons: buttons.map((b) => ({ height: b.getBoundingClientRect().height, line: parseFloat(getComputedStyle(b).lineHeight) })),
        lastRowBottom: rows.at(-1)?.getBoundingClientRect().bottom ?? 0, viewport: innerHeight };
    });
    expect(layout.buttons).toHaveLength(2);
    for (const button of layout.buttons) expect(button.height).toBeLessThan(button.line * 2 + 20);
    expect(layout.lastRowBottom).toBeLessThanOrEqual(layout.viewport);
    await shot(page, size, 'peace-card');
  });

  test('a two-step civilian capture names the adjacent barbarian', async ({ page }) => {
    const save = JSON.parse(warPromptCoreSave(true));
    const mover = Object.values(save.units).find((u: any) => u.owner === 0 && u.kind === 'Soldier') as any;
    const civilian = Object.values(save.units).find((u: any) => u.owner === 1 && u.kind === 'Scout') as any;
    const rival = Object.values(save.units).find((u: any) => u.owner === 1 && u.kind === 'Soldier') as any;
    civilian.kind = 'Settler'; civilian.pos = PURSUIT.scout;
    rival.owner = 255;
    rival.pos = neighbors(civilian.pos).find((h) => distance(h, mover.pos) === 2)!;
    await loadFixture(page, JSON.stringify(save), 1);
    await selectThing(page, 'unit', mover.id);
    await expect.poll(async () => {
      const at = await hexPoint(page, civilian.pos);
      await page.mouse.move(at.x, at.y);
      return page.getByTestId('route-warning').isVisible();
    }).toBe(true);
    await expect(page.getByTestId('route-warning')).toContainText(/Reaches the Settler in 2 steps: barbarian .+ is already next to it and may capture it first/);
    await shot(page, size, 'capture-forecast');
  });
});
