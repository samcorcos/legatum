// One set of states for both sides of the design review. Render staging never resolves a turn.
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { expectSidebarGlance, expectSidebarReachable, expectSlimRail } from '../sidebarAssertions';
import { test, expect } from '../fixtures';
import { SIZES, shot, type GameWindow } from './helpers';
import type { ReportLine } from '../../src/format';
import { withV2 } from '../../src/test/vpV2Fixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { coreFixture } from '../../src/test/coreFixture';
import { endPlanning } from '../turnActions';

// Headless Chromium normally hides scrollbars; review captures must show the real scroll affordance.
test.use({ launchOptions: { ignoreDefaultArgs: ['--hide-scrollbars'] } });

const phase = process.env.SIDEBAR_PHASE ?? 'after';

for (const size of SIZES) test(`delegated city cannot choose after Ready ${size.name}`, async ({ page }) => {
  const { fixture, own } = wallsFixture();
  const city = fixture.cities.find((c) => c.id === own)!;
  city.producing = null;
  city.productionQueue = [];
  city.prod = 0;
  for (const player of fixture.players.slice(1)) { player.isHuman = false; fixture.setup.players[player.id].is_human = false; }
  for (const relation of fixture.diplomacy!.relations) relation.state = 'Peace';
  await page.setViewportSize(size);
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Explorer'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(fixture), turn: fixture.turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(fixture.turn));
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), own);
  await page.getByRole('combobox', { name: /Build automatically in/ }).selectOption('Science');
  await page.evaluate((id) => {
    const store = (window as unknown as GameWindow).legatum;
    const s = store.getState();
    const view = s.views![s.seat];
    store.setState({ views: [{ ...view, cities: view.cities.map((c) => c.id === id ? { ...c, detail: { ...c.detail!, options: c.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Soldier') } } : c) }] });
  }, own);
  await endPlanning(page);
  await expect(page.locator('.seatbar .cta.todo')).toContainText('Choose production');
  await expect(page.locator('.seatbar .cta.todo')).toContainText("Nothing here fits its automation focus");
  await page.mouse.move(size.width - 1, 1);
  await shot(page, size, 'delegated-no-choice-ready');
});

for (const size of SIZES) for (const state of ['early', 'mid', 'holding', 'rival'] as const) {
  test(`sidebar ${state} ${size.name}`, async ({ page }) => {
    await page.setViewportSize(size);
    const file = state === 'early' ? 'p892-alliance-loop-t20' : state === 'mid' ? 'sidebar-mid-t117' : 'p892-race-panel-t203';
    const save = gunzipSync(readFileSync(new URL(`../saves/${file}.current.legatum.gz`, import.meta.url))).toString();
    await page.addInitScript((save) => {
      localStorage.setItem('legatum.autosave.v1', save);
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      localStorage.setItem('legatum.era-hud-open.v1', '1');
    }, save);
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByTestId('turn-number')).toHaveText(state === 'early' ? '20' : state === 'mid' ? '117' : '203', { timeout: 90000 });
    if (state === 'holding' || state === 'rival') {
      const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat]; });
      const v = withV2(base, { era: 7, scored: [2, 3, 4, 5, 6] });
      v.turn = 199; v.finished = false; v.winner = null;
      v.research = { ...v.research, era: 6, era_name: 'Future', era_known: 1, current: null, queue: [], schedule: [], turns_left: null };
      const me = v.players.find(p => p.id === v.me)!;
      me.faction = 'Sparta';
      v.faction_goal = { value: 0, target: 3, label: '0 of 3 veterans retired', done: false, live: false };
      v.actions = { ...v.actions, retire: { retired: 0, target: 3, level: 4, shrines: [], veterans: v.units.filter(u => u.owner === v.me).slice(0, 2).map(u => ({ unit: u.id, level: 4, blocker: 'Needs a city with an Agoge' })) } };
      const holder = v.standings.find(s => state === 'holding' ? s.team === v.team : s.team !== v.team)!;
      const owner = holder.members[0];
      v.great_works = (v.great_works ?? []).map(w => w.player === owner ? { ...w, completed: 'RelayUplink' } : w);
      v.eras!.forEach(e => { e.entered_by = [...new Set([...e.entered_by, owner])]; e.scored_by = e.era < 7 ? [...e.scored_by.filter(x => x.player !== owner), { player: owner, slot: 0, turn: 190 }] : []; });
      v.players.forEach(p => {
        p.vp_sources = v.eras!.filter(e => e.scored_by.some(x => x.player === p.id)).map(e => [{ kind: 'EraObjective', era: e.era }, 1]);
        if (p.id === owner) {
          p.era = 7;
          p.vp_sources.push([{ kind: 'GreatWork', of: owner }, 2]);
        }
        p.vp = p.vp_sources.reduce((n, [, points]) => n + points, 0);
      });
      v.standings.forEach(s => { s.victory_hold = s === holder ? { turns_held: state === 'holding' ? 1 : 7, turns_needed: 10 } : null; s.vp = v.players.filter(p => s.members.includes(p.id)).reduce((n, p) => n + p.vp, 0); });
      const lines: ReportLine[] = Array.from({ length: 15 }, (_, i) => ({ category: 'cities', text: i === 0 ? 'A city was attacked' : 'Production completed', details: [], at: null, players: [v.me], weight: 1, tone: i === 0 ? 'bad' : 'good' }));
      const report = { turn: 198, hash: '', events: [[]], lines: [lines] };
      await page.evaluate(({ v, report }) => { const store = (window as unknown as GameWindow).legatum; const s = store.getState(); store.setState({ views: s.views!.map((x, i) => i === s.seat ? v : x), selection: null, reportOpen: false, report }); }, { v, report });
    }
    await page.evaluate(() => { (window as unknown as GameWindow).legatum.getState().set({ error: null, selection: null }); });
    await page.evaluate(() => document.fonts.ready);
    await page.mouse.move(size.width - 1, 1);
    await page.waitForTimeout(700);
    await page.screenshot({ path: `test-results/sidebar/${phase}/${state}-${size.name}.png`, animations: 'disabled' });
    if (phase === 'before') return;
    await expectSidebarGlance(page);
    await expectSidebarReachable(page);
    await page.locator('.sidebar-scroll').evaluate(el => el.focus({ preventScroll: true }));
    await page.waitForTimeout(250);
    await page.mouse.move(size.width - 1, 1);
    await page.screenshot({ path: `test-results/sidebar/expanded/${state}-${size.name}.png`, animations: 'disabled' });
    if (size.width <= 1100 && size.height <= 700) {
      // The commonest short-window state: a unit selected, so its panel narrows the sidebar to the slim rail.
      await page.evaluate(() => {
        const store = (window as unknown as GameWindow).legatum, s = store.getState(), v = s.views![s.seat];
        s.select({ kind: 'unit', id: v.units.find((u) => u.owner === v.me)!.id }, true);
      });
      await expect(page.locator('.game:has(.unit-panel)')).toBeVisible();
      await expectSlimRail(page);
      await page.mouse.move(size.width - 1, 1);
      await page.screenshot({ path: `test-results/sidebar/rail/${state}-${size.name}.png`, animations: 'disabled' });
    }
  });
}
