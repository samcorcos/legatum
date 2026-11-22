// Screen tour, research, eras and tech unlocks. One of the domain slices of the old e2e/screen-tour.spec.ts; run it alone with
// `npx playwright test e2e/tour/research-unlocks`. Shots land in test-results/screen-tour/<size>/research-unlocks/.
import type { Page } from '@playwright/test';
import type { TechId } from '../../src/types';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { eraTechs } from '../../src/data';
import { momentsForTurn } from '../../src/celebrate/moments';
import { contextFromViews, formatEvents } from '../../src/format';
import { withWorldAges } from '../../src/test/vpV2Fixture';
import { SIZES, loadFixture, shot, type GameWindow } from './helpers';

// Each test starts from its own save, so tests can run on separate workers.
test.describe.configure({ mode: 'parallel' });

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });


  test('research panel by faction: Byzantium, Mongol stepping stone, Switzerland, Athens', async ({ page, browser }) => {
    test.setTimeout(120_000);
    const research = page.locator('dialog.tech-modal');
    const openResearch = async (p: Page) => {
      // A crowded top bar (the Cities button shares its room) drops the Technologies button first; the research cell
      // opens the same dialog. Click whichever shows, and fail in 5 s naming it rather than waiting out the test.
      const nav = p.locator('.topbar .tb-nav-tech');
      const opener = (await nav.isVisible()) ? nav : p.locator('.topbar .research-chip');
      await expect(opener).toBeVisible({ timeout: 5000 });
      await opener.click();
      await expect(p.locator('dialog.tech-modal')).toBeVisible();
    };
    /** Inspect a card the way a player does (its Details link opens them; the card itself researches), then bring it into view. */
    const inspectTech = async (p: Page, tech: string) => {
      const card = p.locator(`dialog.tech-modal .era-card[data-tech="${tech}"]`);
      await card.scrollIntoViewIfNeeded();
      await card.locator('.era-details-chevron').click();
      // The panel scrolls the details in itself, keeping the card's name in view on a short window.
      await expect(p.locator(`dialog.tech-modal .research-detail[data-tech="${tech}"]`)).toBeVisible();
      await p.mouse.move(2, 2); // off the card, so its art's hover card stays shut
    };
    const byzantiumResearch = researchFixture('Byzantium');
    await loadFixture(page, coreFixture(byzantiumResearch), 2);
    await openResearch(page);
    await expect(research.locator('.benefit-ledger')).toHaveCount(5);
    await expect(research.locator('.research-scroll > .era-current + .era-objectives-research')).toHaveCount(1);
    await inspectTech(page, 'Writing');
    await expect(research.locator('.era-card[data-tech="Writing"]')).toContainText('Library');
    await expect(research.locator('.era-card[data-tech="Writing"]')).not.toContainText(/salvage/i);
    await shot(page, size, '60-research-byzantium');

    // The Mongols cannot raise Masonry's Walls before the Medieval era: the card says so in place of the build line.
    {
      const context0 = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context0.newPage();
        await loadFixture(p, coreFixture(researchFixture('Mongols')), 2);
        await openResearch(p);
        await inspectTech(p, 'Masonry');
        const masonry = p.locator('dialog.tech-modal .era-card[data-tech="Masonry"]');
        await expect(masonry).toContainText('Walls for the Mongols from the Medieval era');
        await expect(masonry).not.toContainText('city strength');
        await expect(masonry).not.toContainText('No benefit');
        await expect(masonry).toContainText('Quarries +1 Production');
        await shot(p, size, '61-research-mongols-stepping-stone');
      } finally { await context0.close(); }
    }

    for (const [faction, tech, name, file] of [
      ['Switzerland', 'Masonry', 'Alpine Redoubt', '62-research-switzerland'],
      ['Athens', 'Writing', 'Lyceum', '63-research-athens'],
    ] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        await loadFixture(p, coreFixture(researchFixture(faction)), 2);
        await openResearch(p);
        await inspectTech(p, tech);
        await expect(p.locator(`dialog.tech-modal .era-card[data-tech="${tech}"]`)).toContainText(name);
        await expect(p.locator('dialog.tech-modal .era-card.stepping')).toHaveCount(0);
        await shot(p, size, file);
      } finally { await context.close(); }
    }
  });

  test('Science hover: the capital, cities and buildings, for Byzantium and a faction with a renamed Library', async ({ page, browser }) => {
    // A University and Writing so the buildings line has something to add (legatum-ui: "Yield
    // hovers show where the number comes from").
    const byzantium = researchFixture('Byzantium');
    const byzantiumCapital = byzantium.cities.find((c) => c.owner === 0 && c.isCapital)!;
    byzantiumCapital.buildings = ['University'];
    byzantium.players[0].techs = [...byzantium.players[0].techs, 'Writing'];
    await loadFixture(page, coreFixture(byzantium), 2);
    const science = page.locator('.science-stat');
    await science.hover();
    const hover = page.locator('.science-hover');
    await expect(hover).toBeVisible();
    await expect(hover).toContainText('Capital');
    await expect(hover).toContainText('Buildings');
    await expect(hover).not.toContainText(/salvage|camp|knowledge from/i);
    await shot(page, size, '64-science-hover-byzantium');

    const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const normal = researchFixture('Athens');
      const normalCapital = normal.cities.find((c) => c.owner === 0 && c.isCapital)!;
      normalCapital.buildings = ['Library']; // Athens' Lyceum, by its own rename.
      await loadFixture(p, coreFixture(normal), 2);
      await p.locator('.science-stat').hover();
      const normalHover = p.locator('.science-hover');
      await expect(normalHover).toBeVisible();
      await expect(normalHover).toContainText('Capital');
      await expect(normalHover).toContainText(/\d+ cit(y|ies)/);
      await expect(normalHover).toContainText('Buildings');
      await shot(p, size, '65-science-hover-faction');
    } finally { await context.close(); }
  });

  test('the World Age box on the research screen: before any Age, and with Ages open', async ({ page }) => {
    test.setTimeout(120_000);
    await loadFixture(page, coreFixture(researchFixture('India')), 2);
    await page.locator('.topbar .research-chip').click();
    const box = page.locator('dialog.tech-modal').getByTestId('research-objectives');
    await box.scrollIntoViewIfNeeded();
    await expect(box).toHaveClass(/\bno-obj\b/);
    await expect(box).toContainText('No Age has begun');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '90a-research-world-age-none');
    await page.evaluate((view) => {
      const store = (window as unknown as GameWindow).legatum, s = store.getState();
      store.setState({ views: s.views!.map((x, i) => (i === s.seat ? view : x)) });
    }, await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); }).then(withWorldAges));
    await expect(box).not.toHaveClass(/\bno-obj\b/);
    await box.scrollIntoViewIfNeeded();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '90b-research-world-age-open');
  });

  test('eras: the research screen with a forecast, the era-up card and a rival\'s era notice', async ({ page }) => {
    test.setTimeout(120_000);
    // Ancient: two techs known, Currency and Writing discounted by diffusion, Masonry researching with Writing queued behind it.
    const ancient = researchFixture('India');
    ancient.players[0].research = 'Masonry';
    ancient.players[0].research_queue = ['Masonry', 'Writing'];
    await loadFixture(page, coreFixture(ancient), 2);
    await page.locator('.topbar .research-chip').click();
    const research = page.locator('dialog.tech-modal');
    await expect(research.locator('.era-card')).toHaveCount(5);
    await expect(research.getByTestId('era-discounts').filter({ hasText: 'known by' })).toHaveCount(3);
    await expect(research.locator('.research-queue [data-testid="queue-arrival"]')).toHaveCount(2);
    await shot(page, size, '90-research-forecast');
    await research.getByRole('button', { name: 'Close research', exact: true }).click();

    // Entering the Classical era: the unlock card first, then the era card with the five techs and the choice to make.
    const context = await page.context().browser()!.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const finishing = researchFixture('India');
      finishing.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency'];
      finishing.players[0].research = 'Writing';
      finishing.players[0].research_queue = ['Writing'];
      finishing.players[0].science = 5000;
      await loadFixture(p, coreFixture(finishing), 2);
      await endPlanning(p);
      await expect(p.locator('.moment-card.unlock:not(.era-up)')).toBeVisible();
      await p.keyboard.press('Escape');
      // The era-up is the full-screen takeover (e2e/era-takeover.spec.ts has its own checks and shots).
      const era = p.getByTestId('era-takeover');
      await expect(era).toContainText('The Classical Era');
      await expect(era).toContainText('5 to go');
      await p.waitForTimeout(3200);
      await shot(p, size, '91-era-up-card');
      // A rival's era-up is a banner over the map. The first empire into an era is told by the World Age banner
      // instead (EraNotice skips first-in events since rules 50), so the notice shot is a rival following in.
      await p.keyboard.press('Escape');
      await expect(era).toHaveCount(0);
      await p.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState();
        s.set({ report: { turn: s.views![s.seat].turn, hash: 'era', lines: [[], []], events: [[{ type: 'EraEntered', player: 1, era: 3, first: false }], []] }, reportOpen: false });
      });
      const notice = p.getByTestId('era-notice');
      await expect(notice).toContainText(/entered the \w+ era$/);
      await expect(notice).not.toHaveClass(/\bfirst\b/);
      await shot(p, size, '92-era-notice');
    } finally { await context.close(); }
  });

  test('tech unlock card: every own tech, with a building to show or a stepping stone to continue from', async ({ page, browser }) => {
    test.setTimeout(180_000);
    // Stages the report AND sets the same research in the store, so the card and the top bar agree.
    const unlockReport = async (p: Page, tech: TechId, researching = true) => {
      const current = researching ? 'Currency' as TechId : null;
      await p.evaluate((current) => {
        const s = (window as unknown as GameWindow).legatum.getState();
        s.set({ views: s.views!.map((v, i) => i === s.seat ? { ...v, research: { ...v.research, current, turns_left: current ? 12 : null } } : v) });
      }, current);
      const v = await p.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
      const events = [{ type: 'TechResearched' as const, player: v.me, tech }];
      const report = { turn: v.turn - 1, hash: '', events: [events], moments: [momentsForTurn(events, v.me, null, v)], alarms: [[]], lines: [formatEvents(events, contextFromViews(v, v))] };
      await p.evaluate((report) => (window as unknown as GameWindow).legatum.getState().set({ report, alarmSeen: [] }), report);
    };
    await loadFixture(page, coreFixture(researchFixture('India')), 2);
    await unlockReport(page, 'BronzeWorking');
    const card = page.locator('.moment-card.unlock');
    // Playwright's "visible" is not enough: the primary button must sit fully inside the window (8.9.14 review: it was clipped at 1000x600).
    const expectPrimaryInView = async (p: Page) => {
      const edges = await p.locator('.moment-card.unlock .btn.primary').evaluate((el) => {
        const r = el.getBoundingClientRect();
        return { top: r.top, bottom: r.bottom, left: r.left, right: r.right, h: window.innerHeight, w: window.innerWidth };
      });
      expect(edges.top).toBeGreaterThanOrEqual(0);
      expect(edges.bottom).toBeLessThanOrEqual(edges.h);
      expect(edges.left).toBeGreaterThanOrEqual(0);
      expect(edges.right).toBeLessThanOrEqual(edges.w);
    };
    await expect(card).toContainText('Bronze Working');
    await expect(card).toContainText('Build a Workshop in any city: +2 Production');
    await expect(card.locator('.unlock-tile .unlock-kind').first()).toHaveText('Building');
    await expect(card.getByRole('button', { name: /^Show Workshop in / })).toBeVisible();
    await shot(page, size, '80-unlock-card');
    // Two unlocks: a unit and an effect share the card, each with its own line.
    await unlockReport(page, 'Feudalism');
    await expect(card).toContainText('Pikeman');
    await expect(card).toContainText('Mill');
    await expect(card.locator('.unlock-tile.unit')).toHaveCount(1);
    await expect(card.locator('.unlock-cost').filter({ hasText: 'From Phalangite' })).toHaveCount(1);
    await shot(page, size, '80b-unlock-card-two-unlocks');
    // Education has the University, with no world wonder and no resource reveal (Niter moved to Gunpowder, 2026-11-07).
    await unlockReport(page, 'Education');
    await expect(card).not.toContainText('Hagia Sophia');
    await expect(card.locator('.unlock-tile.building').filter({ hasText: 'University' }).locator('.unlock-art')).toBeVisible();
    for (const row of await card.locator('.unlock-tile').all()) {
      const r = (await row.boundingBox())!;
      expect(r.y).toBeGreaterThanOrEqual(0);
      expect(r.y + r.height).toBeLessThanOrEqual(size.height);
    }
    await expect(card).not.toContainText('Reveals');
    await expectPrimaryInView(page);
    await shot(page, size, '80h-unlock-card-education');
    // Each era's resource is revealed in the era it is used: Niter at Gunpowder (Renaissance). A short window (1000x600) uses the compact ledger so the Niter row is on screen (by rectangle, not DOM visibility).
    await unlockReport(page, 'Gunpowder');
    await expect(card).toContainText('Reveals Niter');
    for (const row of await card.locator('.unlock-tile').all()) {
      const r = (await row.boundingBox())!;
      expect(r.y).toBeGreaterThanOrEqual(0);
      expect(r.y + r.height).toBeLessThanOrEqual(size.height);
    }
    await expectPrimaryInView(page);
    await shot(page, size, '80h2-unlock-card-gunpowder');
    // Coal is revealed at Industrialization; the Royal Arsenal moved to Ballistics.
    await unlockReport(page, 'Industrialization');
    await expect(card).toContainText('Reveals Coal');
    await expect(card).not.toContainText('Royal Arsenal');
    await expectPrimaryInView(page);
    await shot(page, size, '80e-unlock-card-longest');
    await unlockReport(page, 'Combustion');
    await expect(card.locator('img[src*="improvement-oil-well"]')).toBeVisible();
    await expect(card).toContainText('Reveals Oil');
    const oilRows = await card.locator('.unlock-tile').allTextContents();
    expect(oilRows.findIndex((text) => text.includes('Reveals Oil'))).toBeLessThan(oilRows.findIndex((text) => text.includes('Oil Wells')));
    await shot(page, size, '80j-unlock-card-combustion-oil-well');
    // The Research Lab moved to Electronics; Electricity now unlocks the Stock Exchange.
    await unlockReport(page, 'Electricity');
    await expect(card).toContainText('Stock Exchange');
    await shot(page, size, '80k-unlock-card-electricity');
    await unlockReport(page, 'AutonomousSystems');
    await expect(card).toContainText('Reveals Silicon');
    await shot(page, size, '80k2-unlock-card-autonomous-systems');
    await unlockReport(page, 'Cannonry');
    await expect(card).toContainText('Royal Arsenal');
    await unlockReport(page, 'Machinery');
    await expect(card).toContainText('Hagia Sophia');
    await expect(card).toContainText('+1 Production for each Workshop, Forge and Factory');
    await unlockReport(page, 'PrintingPress');
    await expect(card).toContainText('+1 Science for each Library, University and Research Lab');
    await expect(card).toContainText('+1 Gold for each Market and Bank');
    await unlockReport(page, 'StealthTechnology');
    await expect(card).toContainText('Stealth Fighter');
    await expect(card).toContainText('Stealth Bomber');
    // The units carry their own reach, guard, intercept and supply; the upgrade path is on each unit's row, with no separate upgrade or stats rule.
    await expect(card).toContainText('Strikes 7 tiles, guards 5 · uses Silicon');
    await expect(card).toContainText('Strikes 10 tiles; only Stealth Fighters stop it · uses Silicon');
    // Stealth aircraft upgrade from the Jet line since jets joined the air tree.
    await expect(card).toContainText('From Jet Fighter');
    await expect(card).toContainText('From Jet Bomber');
    await expect(card).not.toContainText('can upgrade');
    await expect(card).not.toContainText('Stealth aircraft use');
    await expect(card).toContainText('Hangars hold 3 aircraft');
    // No row of the card ends in an ellipsis: every rule wraps whole (8.9.23 review: "Fighters can upgrade to Stealth Fighters: range 7, guard …").
    const clipped = await card.locator('.unlock-name, .unlock-cost, .unlock-sentence').evaluateAll((els) => els.filter((el) => el.scrollWidth > el.clientWidth + 1 || getComputedStyle(el).textOverflow === 'ellipsis').map((el) => el.textContent));
    expect(clipped, 'no unlock row is clipped or ends in an ellipsis').toEqual([]);
    await expectPrimaryInView(page);
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '80i-unlock-card-stealth');
    await unlockReport(page, 'FusionPower');
    await expect(card).toContainText('+5 Production in every city');
    await expect(card).toContainText('Factories and Research Labs need no Coal or Oil');
    await unlockReport(page, 'RecursiveAi');
    await expect(card).toContainText('Reveal the whole map to your team; units still need sight');
    await expect(card).toContainText('+20% Production in every city');
    for (const [faction, tech, text, file] of [
      ['Byzantium', 'Writing', 'Build a Library in any city', '80f-unlock-card-library'],
      ['Switzerland', 'BronzeWorking', 'Build a Workshop in any city: +2 Production', '80c-unlock-card-switzerland'],
      ['Mongols', 'Masonry', 'Walls for the Mongols from the Medieval era', '80d-unlock-card-stepping-stone'],
    ] as const) {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        await loadFixture(p, coreFixture(researchFixture(faction)), 2);
        // The Mongols' Walls card is staged with no research set: Choose research is its primary, and the card shows no Build price.
        await unlockReport(p, tech, faction !== 'Mongols' || tech !== 'Masonry');
        await expect(p.locator('.moment-card.unlock')).toContainText(text);
        if (tech === 'Masonry') {
          await expect(p.locator('.moment-card.unlock .btn.primary')).toContainText('Choose research');
        }
        await shot(p, size, file);
      } finally { await context.close(); }
    }
    // The common real case: the engine finishes the last queued tech, so nothing is set to research next.
    const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
    try {
      const p = await context.newPage();
      const state = researchFixture('India');
      state.players[0].research = 'Currency';
      state.players[0].science = 5000;
      await loadFixture(p, coreFixture(state), 2);
      await endPlanning(p);
      const real = p.locator('.moment-card.unlock');
      await expect(real).toContainText('No research is set.');
      await expect(real.getByRole('button', { name: /^Choose research/ })).toBeVisible();
      await shot(p, size, '80g-unlock-card-choose-research-real');
    } finally { await context.close(); }
  });

  test('paired era unlocks show two legible paintings', async ({ page }) => {
    const state = researchFixture('India');
    state.players[0].techs = [0, 1].flatMap(eraTechs);
    await loadFixture(page, coreFixture(state), 2);
    await page.locator('.topbar .research-chip').click();
    const crop = page.locator('dialog.tech-modal .era-card[data-tech="CropRotation"]');
    await crop.scrollIntoViewIfNeeded();
    const pair = crop.locator('.era-unlock-art.pair');
    await expect(pair.locator('img')).toHaveCount(2);
    const widths = await pair.locator('img').evaluateAll((images) => images.map((image) => image.getBoundingClientRect().width));
    expect(widths.every((width) => width >= 24)).toBe(true);
    await shot(page, size, '80l-crop-rotation-paired-unlock');
  });


  test('Egypt skips Bronze Working: four Ancient cards, counts without it, its unlock and era-up cards', async ({ page, browser }) => {
    test.setTimeout(150_000);
    const research = page.locator('dialog.tech-modal');
    // Turn 1 as Egypt: Bronze Working is known from the start (it gives Egypt nothing) and is not on the screen.
    const egypt = researchFixture('Egypt');
    egypt.players[0].techs = ['BronzeWorking'];
    await loadFixture(page, coreFixture(egypt), 2);
    await shot(page, size, '93-egypt-topbar');
    await page.locator('.topbar .research-chip').click();
    await expect(research).toBeVisible();
    await expect(research.locator('.era-current .era-card')).toHaveCount(4);
    await expect(research.locator('.era-card[data-tech="BronzeWorking"]')).toHaveCount(0);
    await expect(research).not.toContainText('No benefit');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '93-research-egypt-turn1');
    await research.locator('.research-scroll').evaluate((el) => el.scrollTo(0, el.scrollHeight));
    await shot(page, size, '93b-research-egypt-later-eras');
    // Compare: India on turn 1 with nothing known.
    {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const india = researchFixture('India');
        india.players[0].techs = [];
        await loadFixture(p, coreFixture(india), 2);
        await shot(p, size, '94-india-topbar');
        await p.locator('.topbar .research-chip').click();
        await expect(p.locator('dialog.tech-modal .era-current .era-card')).toHaveCount(5);
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, '94-research-india-turn1');
      } finally { await context.close(); }
    }
    // Egypt's Machinery card: the Workshop/Forge/Factory Production line is gone, Hagia Sophia stays.
    {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const mid = researchFixture('Egypt');
        mid.players[0].techs = [0, 1].flatMap(eraTechs);
        await loadFixture(p, coreFixture(mid), 2);
        await p.locator('.topbar .research-chip').click();
        const machinery = p.locator('dialog.tech-modal .era-card[data-tech="Machinery"]');
        await machinery.scrollIntoViewIfNeeded();
        await expect(machinery).not.toContainText('Workshop, Forge and Factory');
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, '95-research-egypt-medieval');
      } finally { await context.close(); }
    }
    // Egypt finishes the Ancient era with four technologies: the unlock card, then the era-up.
    {
      const context = await browser.newContext({ viewport: { width: size.width, height: size.height } });
      try {
        const p = await context.newPage();
        const finishing = researchFixture('Egypt');
        finishing.players[0].techs = ['BronzeWorking', 'Irrigation', 'Masonry', 'Currency'];
        finishing.players[0].research = 'Writing';
        finishing.players[0].research_queue = ['Writing'];
        finishing.players[0].science = 5000;
        await loadFixture(p, coreFixture(finishing), 2);
        await endPlanning(p);
        await expect(p.locator('.moment-card.unlock:not(.era-up)')).toBeVisible();
        await p.mouse.move(5, size.height / 2);
        await shot(p, size, '96-egypt-unlock-writing');
        await p.keyboard.press('Escape');
        const era = p.getByTestId('era-takeover');
        await expect(era).toContainText('The Classical Era');
        await p.waitForTimeout(3200);
        await shot(p, size, '97-egypt-era-up');
      } finally { await context.close(); }
    }
  });

  test('the Guide: Eras section', async ({ page }) => {
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^How to Play/ }).click();
    const guide = page.getByRole('dialog', { name: 'How to Play', exact: true });
    await guide.getByRole('button', { name: /Cities & research/ }).click();
    const h = guide.locator('.guide-page').getByRole('heading', { name: /^Eras: research in order/ });
    await h.scrollIntoViewIfNeeded();
    await h.evaluate((el) => el.scrollIntoView({ block: 'start' }));
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '98-guide-eras');
  });

});
