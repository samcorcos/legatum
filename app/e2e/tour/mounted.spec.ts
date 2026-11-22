// Screen tour, the mounted rebalance (2026-10-31): the forecast lines "Melee line against mounted (+40%)", "Mounted against ranged (+40%)" and
// "Mounted against a city (-33%)", the mounted unit cards, the production picker, the Guide's unit-class section and the Chivalry unlock.
// Run alone with `npx playwright test e2e/tour/mounted`. Shots land in test-results/screen-tour/<size>/mounted/; each forecast also saves its text.
import { mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import type { TechId } from '../../src/types';
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { spawnUnit, refreshVision } from '../../src/test/fixtureEngine';
import { fromOffset, ring, distance } from '../../src/hex';
import { TECHS, TECH_ERA } from '../../src/data';
import { momentsForTurn } from '../../src/celebrate/moments';
import { contextFromViews, formatEvents } from '../../src/format';
import { SIZES, aim, centred, expectWhole, loadFixture, posOf, selectThing, shot, type GameWindow, type Size } from './helpers';

test.describe.configure({ mode: 'parallel' });

const outRoot = fileURLToPath(new URL('../../test-results/screen-tour/', import.meta.url));
async function saveText(size: Size, name: string, text: string) {
  const dir = `${outRoot}${size.name}/mounted/`;
  await mkdir(dir, { recursive: true });
  await writeFile(`${dir}${name}.txt`, text);
}

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('mounted forecasts: a Horseman against a Spearman, an Archer, an open city and a walled city; a Spearman against a Horseman', async ({ page }) => {
    test.setTimeout(240_000);
    const setup = wallsFixture();
    const f = setup.fixture;
    const open = f.cities.find((c) => c.id === setup.open)!;
    const walled = f.cities.find((c) => c.id === setup.walled)!;
    const spear = spawnUnit(f, 1, 'Spearman', fromOffset(6, 8), false);
    spear.fortify = 6;
    const archer = spawnUnit(f, 1, 'Archer', fromOffset(6, 3), false);
    archer.fortify = 6;
    const horseVsSpear = spawnUnit(f, 0, 'Horseman', ring(spear.pos, 1).find((h) => !f.units.some((u) => u.pos.q === h.q && u.pos.r === h.r))!, false);
    const horseVsArcher = spawnUnit(f, 0, 'Horseman', ring(archer.pos, 1).find((h) => !f.units.some((u) => u.pos.q === h.q && u.pos.r === h.r))!, false);
    const horseVsOpen = spawnUnit(f, 0, 'Horseman', ring(open.pos, 1).find((h) => !f.units.some((u) => u.pos.q === h.q && u.pos.r === h.r))!, false);
    const horseVsWalled = spawnUnit(f, 0, 'Horseman', ring(walled.pos, 1).find((h) => !f.units.some((u) => u.pos.q === h.q && u.pos.r === h.r))!, false);
    const rivalHorse = spawnUnit(f, 2, 'Horseman', fromOffset(20, 8), false);
    const myPike = spawnUnit(f, 0, 'Spearman', ring(rivalHorse.pos, 1)[0], false);
    refreshVision(f);
    expect(distance(horseVsSpear.pos, spear.pos)).toBe(1);
    expect(distance(myPike.pos, rivalHorse.pos)).toBe(1);
    await loadFixture(page, coreFixture(f), 2);
    const forecast = page.locator('.forecast');

    const cases: [number, 'unit' | 'city', number, string, RegExp[]][] = [
      [horseVsSpear.id, 'unit', spear.id, '01-horseman-vs-spearman', [/Melee line against mounted \(\+40%\)/]],
      [horseVsArcher.id, 'unit', archer.id, '02-horseman-vs-archer', [/Mounted against ranged \(\+40%\)/]],
      [horseVsOpen.id, 'city', setup.open, '03-horseman-vs-open-city', [/Mounted against a city \(-33%\)/]],
      [horseVsWalled.id, 'city', setup.walled, '04-horseman-vs-walled-city', [/Mounted against a city \(-33%\)/]],
      [myPike.id, 'unit', rivalHorse.id, '05-spearman-vs-horseman', [/Melee line against mounted \(\+40%\)/]],
    ];
    for (const [attacker, kind, target, name, wanted] of cases) {
      await page.mouse.move(5, size.height / 2);
      await selectThing(page, 'unit', attacker);
      await centred(page, attacker);
      await aim(page, await posOf(page, kind, target), forecast);
      const text = await forecast.innerText();
      await saveText(size, name, text);
      // The breakdown is reachable: open every closed details block so the shot shows the numbers.
      await forecast.evaluate((el) => el.querySelectorAll('details:not([open])').forEach((d) => { (d as HTMLDetailsElement).open = true; }));
      const opened = await forecast.innerText();
      await saveText(size, `${name}-open`, opened);
      for (const w of wanted) expect(opened, `${name}: ${w}`).toMatch(w);
      await expectWhole(forecast);
      await shot(page, size, name);
    }
  });

  test('a ranged strike on a mounted target has no mounted bonus; the same mounted unit charging a ranged unit keeps it (2026-11-10)', async ({ page }) => {
    test.setTimeout(240_000);
    const f = wallsFixture().fixture;
    const rivalHorse = spawnUnit(f, 1, 'Horseman', fromOffset(4, 9), false);
    const myArcher = spawnUnit(f, 0, 'Archer', fromOffset(2, 9), false);
    const rivalArcher = spawnUnit(f, 1, 'Archer', fromOffset(9, 9), false);
    const myHorse = spawnUnit(f, 0, 'Horseman', fromOffset(10, 9), false);
    const rivalCav = spawnUnit(f, 2, 'Cavalry', fromOffset(21, 2), false);
    rivalCav.fortify = 6;
    const myCannon = spawnUnit(f, 0, 'FieldCannon', fromOffset(19, 2), false);
    refreshVision(f);
    expect(distance(myArcher.pos, rivalHorse.pos)).toBe(2);
    expect(distance(myCannon.pos, rivalCav.pos)).toBe(2);
    expect(distance(myHorse.pos, rivalArcher.pos)).toBe(1);
    await loadFixture(page, coreFixture(f), 2);
    const forecast = page.locator('.forecast');
    const cases: [number, number, boolean, string, boolean][] = [
      [myArcher.id, rivalHorse.id, true, '30-archer-shoots-horseman', false],
      [myCannon.id, rivalCav.id, true, '31-cannon-shoots-fortified-cavalry', false],
      [myHorse.id, rivalArcher.id, false, '32-horseman-charges-archer', true],
    ];
    for (const [attacker, target, ranged, name, bonus] of cases) {
      await page.mouse.move(5, size.height / 2);
      await selectThing(page, 'unit', attacker);
      await centred(page, attacker);
      // The strength chip over the target, before the forecast card opens.
      await shot(page, size, `${name}-chip`);
      if (ranged) await page.evaluate(() => (window as unknown as GameWindow).legatum.getState().setMode('ranged'));
      await aim(page, await posOf(page, 'unit', target), forecast);
      await forecast.evaluate((el) => el.querySelectorAll('details:not([open])').forEach((d) => { (d as HTMLDetailsElement).open = true; }));
      const opened = await forecast.innerText();
      await saveText(size, `${name}-open`, opened);
      if (bonus) expect(opened, name).toMatch(/Mounted against ranged \(\+40%\)/);
      else expect(opened, name).not.toMatch(/Mounted against ranged/);
      await expectWhole(forecast);
      await shot(page, size, name);
    }
  });

  test('mounted unit panel: the Class stat and its tooltip, for a Horseman and a Knight', async ({ page }) => {
    const f = researchFixture('India');
    f.players[0].techs = TECHS.filter((t) => TECH_ERA[t] <= 1 || t === 'Chivalry' || t === 'Feudalism');
    const city = f.cities.find((c) => c.owner === 0 && c.isCapital)!;
    const horse = spawnUnit(f, 0, 'Horseman', ring(city.pos, 1)[1], false);
    const knight = spawnUnit(f, 0, 'Knight', ring(city.pos, 1)[3], false);
    refreshVision(f);
    await loadFixture(page, coreFixture(f), 2);
    for (const [unit, name] of [[horse.id, '10-unit-panel-horseman'], [knight.id, '11-unit-panel-knight']] as const) {
      await selectThing(page, 'unit', unit);
      await expect(page.locator('.unit-panel')).toBeVisible();
      await page.mouse.move(20, 25);
      await shot(page, size, name);
      await page.locator('.unit-panel .up-class').hover();
      await expect(page.locator('.tip-bubble:visible').first()).toBeVisible();
      await saveText(size, `${name}-class-tip`, await page.locator('.tip-bubble:visible').first().innerText());
      await shot(page, size, `${name}-class-tip`);
      await page.mouse.move(20, 25);
    }
  });

  test('mounted production picker: the Horseman, Cataphract and Knight build cards in a city', async ({ browser }) => {
    test.setTimeout(240_000);
    const era0 = TECHS.filter((t) => TECH_ERA[t] === 0);
    const sets: [string, TechId[], string, string][] = [
      ['Horseman', era0, 'Horseman', '20-picker-horseman'],
      ['Cataphract', [...era0, 'IronWorking' as TechId], 'Cataphract', '21-picker-cataphract'],
      ['Knight', [...TECHS.filter((t) => TECH_ERA[t] <= 1), 'Feudalism' as TechId, 'Chivalry' as TechId], 'Knight', '22-picker-knight'],
    ];
    const texts: string[] = [];
    for (const [label, techs, unit, name] of sets) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const f = researchFixture('India');
        f.players[0].techs = techs;
        f.players[0].gold = 600;
        const city = f.cities.find((c) => c.owner === 0 && c.isCapital)!;
        city.producing = null as never;
        await loadFixture(p, coreFixture(f), 2);
        await selectThing(p, 'city', city.id);
        const panel = p.locator('.city-panel');
        await expect(panel).toBeVisible();
        const card = panel.locator(`.production-choice[data-item="u:${unit}"]`).first();
        await card.scrollIntoViewIfNeeded();
        await expect(card).toBeVisible();
        texts.push(`${label}: ${await card.innerText()}`);
        // Open any folded older mounted line ("Also: Horseman, Cataphract") so the shot shows the family.
        await panel.evaluate((el) => el.querySelectorAll('details.unit-line').forEach((d) => { (d as HTMLDetailsElement).open = true; }));
        await card.scrollIntoViewIfNeeded();
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, name);
        await card.hover();
        await p.waitForTimeout(700);
        const tip = p.locator('.tip-bubble:visible').first();
        if (await tip.count()) { texts.push(`${label} hover: ${await tip.innerText()}`); await shot(p, size, `${name}-hover`); }
      } finally { await context.close(); }
    }
    await saveText(size, 'picker-cards', texts.join('\n---\n'));
  });

  test('Chivalry: the unlock card for India\'s War Elephant (its Knight) and the research card hovercard', async ({ page }) => {
    test.setTimeout(120_000);
    const f = researchFixture('India');
    f.players[0].techs = TECHS.filter((t) => TECH_ERA[t] <= 1 && t !== 'Chivalry');
    f.players[0].research = null as never;
    f.players[0].research_queue = [] as never;
    await loadFixture(page, coreFixture(f), 2);
    // The unlock card the turn Chivalry finishes.
    const v = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
    const events = [{ type: 'TechResearched' as const, player: v.me, tech: 'Chivalry' as TechId }];
    const report = { turn: v.turn - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
    await page.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
    const card = page.locator('.moment-card.unlock');
    await expect(card).toBeVisible();
    await saveText(size, '30-unlock-chivalry', await card.innerText());
    await shot(page, size, '30-unlock-card-chivalry');
    await page.keyboard.press('Escape');
    // The research card and the Knight row's hovercard.
    await page.locator('.topbar .research-chip').click();
    const research = page.locator('dialog.tech-modal');
    await expect(research).toBeVisible();
    const tech = research.locator('.era-card[data-tech="Chivalry"]');
    await tech.scrollIntoViewIfNeeded();
    await saveText(size, '31-research-card-chivalry', await tech.innerText());
    await page.mouse.move(5, 5);
    await shot(page, size, '31-research-card-chivalry');
    // India's War Elephant takes the Knight's place on Chivalry, so the fixture's row is the War Elephant.
    const row = tech.locator('.era-unlock.unit', { hasText: 'War Elephant' });
    await row.hover();
    const hover = page.getByTestId('unlock-card');
    await expect(hover).toBeVisible();
    await saveText(size, '32-knight-hovercard', await hover.innerText());
    await shot(page, size, '32-knight-hovercard');
  });

  test('Guide: the unit-class section (The matchup is a number)', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await expect(guide).toBeVisible();
    await guide.getByRole('navigation', { name: 'Guide chapters' }).getByRole('button').filter({ hasText: 'Simultaneous combat' }).click();
    const p = guide.locator('p', { hasText: 'The matchup is a number' });
    await p.scrollIntoViewIfNeeded();
    await expect(p).toContainText('Mounted gets the same bonus charging ranged units, but none when they shoot at it');
    await saveText(size, '40-guide-class-section', await p.evaluate((el) => {
      let text = '';
      // The heading above the paragraph, if any, and the paragraphs around it.
      const around = [el.previousElementSibling, el, el.nextElementSibling, el.nextElementSibling?.nextElementSibling].filter(Boolean) as Element[];
      for (const e of around) text += (e as HTMLElement).innerText + '\n\n';
      return text;
    }));
    await page.evaluate((txt) => {
      const el = [...document.querySelectorAll('dialog p')].find((x) => x.textContent?.includes(txt as string));
      el?.scrollIntoView({ block: 'center' });
    }, 'The matchup is a number');
    await shot(page, size, '40-guide-class-section');
  });
});
