// Records the launch media from the real app (docs/media/PLAN.md). Each clip is a test; MEDIA_ONLY=a,b picks clips.
import { gunzipSync } from 'node:zlib';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { FACTIONS, FACTION_INFO } from '../../src/data';
import { coreFixture } from '../../src/test/coreFixture';
import { researchFixture } from '../../src/test/researchFixture';
import { assaultFixture } from '../../src/test/assaultFixture';
import { makeStandardSetup, createState, meetAll } from '../../src/test/fixtureEngine';
import { refreshVision } from '../../src/test/fixtureEngine';
import { withV2 } from '../../src/test/vpV2Fixture';
import { endPlanning } from '../turnActions';
import { join } from 'node:path';
import { OUT, glide, installCursor, moveCamera, startRecording, wanted } from './recorder';

type W = Window & { legatum: { getState(): any; setState(s: any): void }; legatumMap: { camera: { x: number; y: number; zoom: number }; toScreen(h: { q: number; r: number }): { x: number; y: number } } };


/** Close any save-migration notice so the recording starts clean. */
async function dismissToasts(page: Page) {
  const toast = page.getByText(/^Save updated:/);
  if (await toast.count()) await toast.first().locator('xpath=..').getByRole('button').first().click().catch(() => undefined);
}

async function loadGame(page: Page, game: string, turn: number) {
  await page.addInitScript(({ game, turn }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 90_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await dismissToasts(page);
}

async function loadLate(page: Page) {
  const save = gunzipSync(readFileSync(fileURLToPath(new URL('../saves/p892-race-panel-t203.current.legatum.gz', import.meta.url)))).toString('utf8');
  await page.addInitScript((save) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', save);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, save);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('203', { timeout: 90_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.waitForTimeout(600);
  await dismissToasts(page);
  await page.waitForTimeout(500);
}

/** Centre the camera on a hex of ours (the nth city) at a zoom. */
async function focusOwnCity(page: Page, nth: number, zoom: number, ms: number) {
  const at = await page.evaluate((nth) => {
    const w = window as unknown as W, s = w.legatum.getState(), v = s.views[s.seat];
    const mine = v.cities.filter((c: any) => c.owner === v.me);
    const c = mine[Math.min(nth, mine.length - 1)];
    const p = w.legatumMap.toScreen(c.pos), box = document.querySelector('[data-game-map]')!.getBoundingClientRect(), cam = w.legatumMap.camera;
    return { x: cam.x + (p.x - box.width / 2) / cam.zoom, y: cam.y + (p.y - box.height / 2) / cam.zoom };
  }, nth);
  await moveCamera(page, { ...at, zoom }, ms);
}


/** End the turn, confirming the "decisions pending" dialog if it appears. */
async function endTurn(page: Page, next: number) {
  await endPlanning(page);
  const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
  await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
  await expect(page.getByTestId('turn-number')).toHaveText(String(next), { timeout: 90_000 });
}

/** Turn 16: our ally is at War with a third empire and calls us to arms (the diplomacy fixture, with people's names). */
function callToArmsSave() {
  const setup = makeStandardSetup('9943', 'Switzerland', 'India', 'Switzerland', 'India');
  setup.players = setup.players.slice(0, 4).map((p, id) => ({ ...p, team: id, is_human: id === 0 }));
  const s = createState(setup);
  s.turn = 16;
  meetAll(s);
  ['Ada', 'Theron', 'Gorgo', 'Temur'].forEach((n, i) => { s.players[i].name = n; });
  (['Switzerland', 'Athens', 'Sparta', 'Mongols'] as const).forEach((f, i) => { (s.players[i] as any).faction = f; });
  s.players[0].gold = 300;
  const d = s.diplomacy;
  Object.assign(d.relations.find((r: any) => r.a === 0 && r.b === 1)!, { state: 'Alliance', effective_turn: 12, version: 1, security: { id: 1, started_turn: 12, protected_until: 22 } });
  Object.assign(d.relations.find((r: any) => r.a === 1 && r.b === 2)!, { state: 'War', war_id: 1, effective_turn: 15, version: 1, declarers: [1] });
  d.next_treaty_id = 1; d.next_war_id = 1; d.next_proposal_id = 3;
  d.calls = [{ id: 3, team: 0, ally: 1, target: 2, created_turn: 16, respond_by: 17, alliance_version: 1, war_version: 1 }];
  refreshVision(s);
  return coreFixture(s);
}

test.beforeEach(async ({ page }) => { await installCursor(page); });

test('factions', async ({ page }) => {
  test.skip(!wanted('factions'));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(900);
  const rec = await startRecording(page, 'factions');
  await page.waitForTimeout(700);
  await glide(page, page.getByRole('button', { name: /^Seat 1 faction:/ }), true);
  const stage = page.locator('dialog[data-testid="faction-stage"][open]');
  await expect(stage).toBeVisible();
  await page.waitForTimeout(900);
  for (const f of FACTIONS) {
    await glide(page, page.getByRole('option', { name: FACTION_INFO[f].name }), true, 14);
    await page.waitForTimeout(1150);
  }
  console.log('clip', await rec.stop());
});

test('opening', async ({ page }) => {
  test.skip(!wanted('opening'));
  await page.addInitScript(() => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true }).getByRole('radio', { name: /^Hard Ruthless/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('20261023');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 90_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.waitForTimeout(1200);
  const rec = await startRecording(page, 'opening');
  // Open on the map in motion: the camera eases in on the capital, founded on turn 1 (rules 45).
  await moveCamera(page, { zoom: 1.5 }, 2200);
  await page.waitForTimeout(2800);
  const auto = page.getByRole('button', { name: /^Auto-explore/ });
  if (await auto.count()) await glide(page, auto.first(), true);
  await page.waitForTimeout(700);
  for (let turn = 2; turn <= 5; turn++) {
    await page.mouse.move(700, 40, { steps: 12 });
    await endTurn(page, turn);
    for (let i = 0; i < 2; i++) {
      await page.waitForTimeout(1500);
      if (await page.locator('.moment-card').count()) { await page.waitForTimeout(1500); await page.keyboard.press('Escape'); }
    }
    const closeReport = page.getByRole('button', { name: 'Close report', exact: true });
    if (await closeReport.isVisible()) await closeReport.click();
    if (await auto.count()) await auto.first().click().catch(() => undefined);
  }
  await page.waitForTimeout(1000);
  console.log('clip', await rec.stop());
});

test('late-map', async ({ page }) => {
  test.skip(!wanted('late-map'));
  await loadLate(page);
  const rec = await startRecording(page, 'late-map');
  await page.waitForTimeout(800);
  await moveCamera(page, { zoom: 0.3 }, 3400);
  await page.waitForTimeout(1200);
  await focusOwnCity(page, 0, 1.15, 3600);
  await page.waitForTimeout(1500);
  console.log('clip', await rec.stop());
});

test('late-race', async ({ page }) => {
  test.skip(!wanted('late-race'));
  await loadLate(page);
  const rec = await startRecording(page, 'late-race');
  await page.waitForTimeout(1400);
  const race = page.getByRole('region', { name: 'Victory race', exact: true });
  await glide(page, race.getByRole('button', { name: 'Score ledger', exact: true }), true);
  await page.waitForTimeout(2600);
  await page.keyboard.press('Escape');
  await page.waitForTimeout(600);
  await glide(page, page.getByTestId('cities-button'), true);
  await page.waitForTimeout(2800);
  await page.keyboard.press('Escape');
  await page.waitForTimeout(900);
  console.log('clip', await rec.stop());
});

test('research', async ({ page }) => {
  test.skip(!wanted('research'));
  const f = researchFixture('India');
  f.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency', 'Writing'];
  f.players[0].research = null as never;
  f.players[0].research_queue = [] as never;
  await loadGame(page, coreFixture(f), 2);
  await page.waitForTimeout(700);
  await page.locator('.topbar .research-chip').click();
  const d = page.locator('dialog.tech-modal');
  await expect(d).toBeVisible();
  await page.waitForTimeout(500);
  const rec = await startRecording(page, 'research');
  await page.waitForTimeout(1200);
  const row = d.locator('.era-card[data-tech="Mathematics"] .era-unlock.unit', { hasText: 'Phalangite' });
  await glide(page, row);
  await expect(page.getByTestId('unlock-card')).toBeVisible();
  await page.waitForTimeout(2800);
  await glide(page, d.locator('.era-card[data-tech="Mathematics"]'), true);
  await page.waitForTimeout(1600);
  console.log('clip', await rec.stop());
});

/** A hex of the live map in page coordinates (after the camera settles). */
async function mapPoint(page: Page, hex: { q: number; r: number }) {
  let last = { x: NaN, y: NaN };
  await expect.poll(async () => {
    const p = await page.evaluate((hex) => {
      const p = (window as unknown as W).legatumMap.toScreen(hex), b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
      return { x: p.x + b.left, y: p.y + b.top };
    }, hex);
    const still = Math.abs(p.x - last.x) < 0.5 && Math.abs(p.y - last.y) < 0.5;
    last = p;
    return still;
  }).toBe(true);
  return last;
}

test('war', async ({ page }) => {
  test.skip(!wanted('war'));
  const fx = assaultFixture();
  fx.fixture.players.forEach((p: any, i: number) => { p.isHuman = i === 0; });
  // The save check matches each player against the setup, so the setup's seats change too.
  fx.fixture.setup.players.forEach((p: any, i: number) => { p.is_human = i === 0; });
  // A city on its last legs: two Spearmen can take it this turn, so the clip ends on the capture.
  fx.fixture.cities.find((c: any) => c.id === fx.open.city)!.hp = 6;
  (fx.fixture.units.find((u: any) => u.id === fx.garrison) as any).hp = 12;
  await loadGame(page, coreFixture(fx.fixture), 2);
  const c = fx.open;
  await page.evaluate((id) => { const s = (window as unknown as W).legatum.getState(); s.select({ kind: 'unit', id }, true); }, c.first);
  await page.waitForTimeout(1200);
  const rec = await startRecording(page, 'war');
  await page.waitForTimeout(900);
  const at = await mapPoint(page, c.pos);
  await glide(page, { x: at.x - 120, y: at.y + 70 });
  await glide(page, at, false, 30);
  await expect(page.getByTestId('forecast-assault')).toBeVisible({ timeout: 15_000 });
  await page.waitForTimeout(3200);
  // Order the first Soldier in, bring the second, and end the turn.
  await page.evaluate(({ unit, to }) => { (window as unknown as W).legatum.getState().editOrders((o: any[]) => [...o, { type: 'MoveUnit', unit, to }]); }, { unit: c.first, to: c.pos });
  await page.waitForTimeout(1200);
  await page.evaluate((id) => { (window as unknown as W).legatum.getState().select({ kind: 'unit', id }, true); }, c.second);
  await page.waitForTimeout(500);
  await page.evaluate(({ unit, to }) => { (window as unknown as W).legatum.getState().editOrders((o: any[]) => [...o, { type: 'MoveUnit', unit, to }]); }, { unit: c.second, to: c.pos });
  await page.waitForTimeout(1500);
  await page.mouse.move(40, 420, { steps: 20 });
  try { await endTurn(page, 3); } catch (e) {
    await page.screenshot({ path: join(OUT, 'war-debug.png') });
    console.log('WAR DEBUG', await page.evaluate(() => document.body.innerText.slice(0, 1500)));
    throw e;
  }
  await page.waitForTimeout(4200);
  console.log('clip', await rec.stop());
});

test('diplomacy', async ({ page }) => {
  test.skip(!wanted('diplomacy'));
  await loadGame(page, callToArmsSave(), 16);
  await page.waitForTimeout(1000);
  const rec = await startRecording(page, 'diplomacy');
  await page.waitForTimeout(1300);
  await glide(page, page.getByRole('button', { name: /Answer call to arms/ }).first(), true);
  const card = page.locator('.call-card');
  await expect(card).toHaveCount(1);
  await page.waitForTimeout(3600);
  await glide(page, card.getByRole('button', { name: /^Join the war on/ }));
  await page.waitForTimeout(1800);
  console.log('clip', await rec.stop());
});

test('era', async ({ page }) => {
  test.skip(!wanted('era'));
  const f = researchFixture('India');
  f.players[0].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency'];
  f.players[0].research = 'Writing';
  f.players[0].research_queue = ['Writing'];
  f.players[0].science = 5000;
  await loadGame(page, coreFixture(f), 2);
  await page.waitForTimeout(900);
  const rec = await startRecording(page, 'era');
  await page.waitForTimeout(900);
  await endPlanning(page);
  await expect(page.locator('.moment-card.unlock')).toBeVisible({ timeout: 60_000 });
  await page.waitForTimeout(1700);
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('era-takeover')).toBeVisible();
  // The takeover on a staged view: every open Age's card.
  const base = await page.evaluate(() => { const s = (window as unknown as W).legatum.getState(); return structuredClone(s.views[s.seat]); });
  const view = withV2(base, { era: 2 });
  await page.evaluate((v) => {
    const store = (window as unknown as W).legatum, s = store.getState();
    store.setState({ views: s.views.map((x: any, i: number) => (i === s.seat ? v : x)) });
    s.set({ report: { turn: v.turn, hash: 'era-up', lines: [[], []], events: [[{ type: 'EraEntered', player: v.me, era: 2, first: false }], []] }, reportOpen: false });
  }, view);
  await page.waitForTimeout(6500);
  console.log('clip', await rec.stop());
});

test('turns', async ({ page }) => {
  test.skip(!wanted('turns'));
  await page.addInitScript(() => localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] })));
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Small/ }).click();
  await page.getByRole('radiogroup', { name: 'AI level (all AI empires)', exact: true }).getByRole('radio', { name: /^Hard Ruthless/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('20261023');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('1', { timeout: 90_000 });
  // The capital is founded on turn 1 (rules 45): no Found City step.
  await page.waitForTimeout(2300);
  const rec = await startRecording(page, 'turns');
  for (let turn = 2; turn <= 13; turn++) {
    for (let i = 0; i < 6; i++) { if (await page.locator('.moment-card, [data-testid="era-takeover"]').count()) await page.keyboard.press('Escape'); }
    const closeReport = page.getByRole('button', { name: 'Close report', exact: true });
    if (await closeReport.isVisible()) await closeReport.click();
    const auto = page.getByRole('button', { name: /^Auto-explore/ });
    if (await auto.count()) await auto.first().click().catch(() => undefined);
    await page.waitForTimeout(500);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 90_000 });
    await page.waitForTimeout(900);
  }
  console.log('clip', await rec.stop());
});
