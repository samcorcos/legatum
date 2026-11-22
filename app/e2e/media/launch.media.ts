// The README and launch media, recorded from the running app at 60 fps on a stepped clock (stepped.ts).
// Every clip starts from a real late-game save in e2e/saves or from a new game; nothing is a sparse fixture.
//   docs/media/src/refresh.sh            records everything and rebuilds docs/media
//   MEDIA_ONLY=battle,world npx playwright test -c playwright.media.config.ts launch.media.ts --trace off
// Output: <MEDIA_OUT>/<clip>.mp4 (1920x1080, 60 fps) and <MEDIA_OUT>/stills/*.png.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { test } from '../fixtures';
import { FACTION_INFO, eraTechs } from '../../src/data';
import { OUT, installCursor, wanted } from './recorder';
import { Film, busiest, cinema, frameAll, load, pagePoint, park, playbackMs, push, resolveTurn, save, settle, snap, tick, war, worldOf, type Cam, type Hex, type W } from './stepped';

// 1280x720 CSS px at 1.5x is a crisp 1920x1080 frame, with interface text large enough to read on a phone.
test.use({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1.5 });
test.beforeEach(async ({ page }) => { page.setDefaultTimeout(90_000); await installCursor(page); mkdirSync(join(OUT, 'stills'), { recursive: true }); });

const LATE = 'p892-race-panel-t203';   // turn 203, eight empires, the Swiss seat (8 cities) with Lucerne under attack
const MID = 'sidebar-mid-t117';        // the same game at turn 117
const AIR = 'p895-athens-bomber-1916'; // turn 172, Athens with a bomber wing
/** That save lists the Celtic seat as India and the Indian seat as Byzantium (their cities and units say otherwise). */
const AIR_FACTIONS = { 2: 'Celts', 7: 'India' };
/** The nine finished peoples (Egypt and Rome are early designs and are not featured). */
const PEOPLES = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens', 'Aztecs'] as const;

const clip = (name: string, fn: (page: Page) => Promise<void>) => test(name, async ({ page }) => {
  test.skip(!wanted(name));
  test.setTimeout(3_600_000);
  await fn(page);
});
/** Let `ms` of game time pass in small steps (so CSS animations advance), without filming. */
async function run(page: Page, ms: number) { for (let t = 0; t < ms; t += 50) await tick(page, 50); }
/** Step time until `cond` holds. */
async function until(page: Page, cond: () => Promise<boolean>, what: string) {
  for (let i = 0; i < 6000; i++) { if (await cond()) return; await page.clock.runFor(50); await page.waitForTimeout(20); }
  throw new Error(`timed out waiting for ${what}`);
}
const centre = async (_page: Page, loc: ReturnType<Page['locator']>) => { const b = await loc.boundingBox(); if (!b) throw new Error('no box'); return { x: b.x + b.width / 2, y: b.y + b.height / 2 }; };
const press = async (page: Page, film: Film) => { await page.mouse.down(); await film.frames(5); await page.mouse.up(); };
const state = <T>(page: Page, fn: (s: any, v: any) => T) => page.evaluate(`(() => { const s = window.legatum.getState(); return (${fn.toString()})(s, s.views[s.seat]); })()`) as Promise<T>;

/** A front at war, from the seat of the empire with the biggest army in contact: every unit attacks, filmed without the HUD. */
for (const [name, file, seat, zoom] of [['battle', LATE, 7, 1.7], ['battle-b', AIR, 4, 1.7]] as const) clip(name, async (page) => {
  const s = save(file, { seat, factions: file === AIR ? AIR_FACTIONS : {} });
  await load(page, s);
  await cinema(page);
  const w = await war(page);
  const cam = await frameAll(page, busiest([...w.at, ...w.targets], 3), zoom);
  await settle(page, { ...cam, zoom: cam.zoom * 0.94 });
  await resolveTurn(page, s.turn, w.orders);
  const ms = await playbackMs(page);
  const n = Math.min(540, Math.ceil(ms / 1000 * 60) + 45);
  console.log(name, 'orders', w.orders.length, 'playback', ms, 'ms', n, 'frames');
  const film = await Film.start(page, name);
  await film.frames(n, push({ ...cam, zoom: cam.zoom * 0.94 }, { ...cam, zoom: cam.zoom * 1.12 }, n));
  await film.stop();
});

/** The bomber wing strikes: every aircraft of the human seat hits the nearest enemy in range. */
clip('air', async (page) => {
  const s = save(AIR, { factions: AIR_FACTIONS });
  await load(page, s);
  await cinema(page);
  const { orders, first } = await page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    const war = new Set(v.players.filter((p: any) => p.team !== v.team).map((p: any) => p.id));
    const targets: Hex[] = [...v.units.filter((u: any) => u.military && war.has(u.owner)).map((u: any) => u.pos), ...v.cities.filter((c: any) => war.has(c.owner) && c.visible).map((c: any) => c.pos)];
    const orders: any[] = [];
    for (const a of v.aircraft ?? []) {
      const base = v.cities.find((c: any) => c.id === a.city)?.pos;
      if (!base) continue;
      const best = targets.map((t) => ({ t, k: d(t, base) })).filter((x) => x.k <= (a.kind === 'Bomber' ? 8 : 5)).sort((x, y) => x.k - y.k)[0];
      if (best) orders.push({ type: 'AirStrike', plane: a.id, target: best.t });
    }
    return { orders, first: orders[0]?.target ?? null };
  });
  if (!first) throw new Error('air: no aircraft can strike');
  const w = await worldOf(page, first);
  const a = { x: w.x - 40, y: w.y + 10, zoom: 1.5 }, b = { x: w.x - 10, y: w.y, zoom: 1.8 };
  await settle(page, a);
  await resolveTurn(page, s.turn, orders);
  const n = Math.min(480, Math.ceil((await playbackMs(page)) / 1000 * 60) + 45);
  const film = await Film.start(page, 'air');
  await film.frames(n, push(a, b, n));
  await film.stop();
});

/** Turn 203 from a city under attack out to the whole world: eight empires on one map. No HUD. */
clip('world', async (page) => {
  await load(page, save(LATE));
  await cinema(page);
  const near = await worldOf(page, { q: 12, r: 19 });
  const all = await state(page, (_s, v) => v.cities.map((c: any) => c.pos)) as Hex[];
  const far = await frameAll(page, all, 0.6);
  const a = { ...near, zoom: 1.55 }, b = { x: far.x, y: far.y, zoom: Math.max(0.3, far.zoom * 0.62) };
  await settle(page, a);
  const film = await Film.start(page, 'world');
  await film.frames(40, push(a, { ...a, zoom: 1.5 }, 40));
  await film.frames(300, push({ ...a, zoom: 1.5 }, b, 300));
  await film.frames(50, push(b, { ...b, zoom: b.zoom * 0.97 }, 50));
  await film.stop();
});

/** The same map with the interface on: out to the world and back to the capital (the README's late-game loop). */
clip('late-map', async (page) => {
  await load(page, save(LATE));
  const home = await worldOf(page, { q: 9, r: 23 });
  const a = { ...home, zoom: 1.2 }, b = { x: home.x + 300, y: home.y - 250, zoom: 0.36 };
  await settle(page, a);
  const film = await Film.start(page, 'late-map');
  await film.frames(30, () => a);
  await film.frames(200, push(a, b, 200));
  await film.frames(50, () => b);
  await film.still('map-late-wide');
  await film.frames(200, push(b, a, 200));
  await film.frames(30, () => a);
  await film.still('map-late');
  await film.stop();
});

/** One empire at turn 117 and at turn 203 under the same camera: the cut dissolves between them. */
for (const [name, file] of [['growth-a', MID], ['growth-b', LATE]] as const) clip(name, async (page) => {
  await load(page, save(file));
  await cinema(page);
  const home = await worldOf(page, { q: 8, r: 23 });
  const a = { x: home.x + 60, y: home.y - 40, zoom: 0.72 }, b = { ...a, zoom: 0.8 };
  await settle(page, a);
  const film = await Film.start(page, name);
  await film.frames(150, push(a, b, 150));
  await film.stop();
});

/** The forecast before an attack: select a unit at the front and point at its target. */
clip('forecast', async (page) => {
  const s = save(LATE, { seat: 7 });
  await load(page, s);
  const w = await war(page);
  const o = w.orders.find((o: any) => o.type === 'MoveUnit') ?? w.orders[0];
  const target: Hex = o.to ?? o.target;
  const from = w.at[w.orders.indexOf(o)];
  const c = await worldOf(page, target);
  await settle(page, { ...c, zoom: 1.5 });
  const film = await Film.start(page, 'forecast');
  await film.frames(20);
  const u = await pagePoint(page, from);
  await film.glide(u, 30, { x: 620, y: 560 });
  await page.evaluate((id) => { (window as unknown as W).legatum.getState().select({ kind: 'unit', id }, true); }, o.unit);
  await film.frames(30);
  const cam = await page.evaluate(() => ({ ...(window as unknown as W).legatumMap.camera }));
  const c2 = await worldOf(page, target);
  await film.frames(40, push(cam, { x: c2.x - 60, y: c2.y, zoom: 1.5 }, 40));
  await film.glide(await pagePoint(page, target), 36, await pagePoint(page, from));
  await film.frames(200);
  await film.still('forecast');
  await film.stop();
});

/** The public race: the panel, then the score ledger with every way to score and who holds it. */
clip('race', async (page) => {
  await load(page, save(LATE));
  const home = await worldOf(page, { q: 9, r: 23 });
  await settle(page, { ...home, zoom: 1.0 });
  const film = await Film.start(page, 'race');
  await film.frames(30);
  const all = page.getByRole('button', { name: /All empires/ }).first();
  if (await all.count()) { await film.glide(await centre(page, all), 30, { x: 760, y: 420 }); await press(page, film); await film.frames(110); await film.still('race-panel'); }
  const link = page.getByRole('button', { name: 'All objectives', exact: true }).first();
  await film.glide(await centre(page, link), 30);
  await press(page, film);
  await film.frames(60);
  await park(page);
  await film.frames(150);
  await film.still('race');
  const sheet = page.locator('dialog[open]').first();
  const box = await sheet.boundingBox();
  if (box) { await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); for (let i = 0; i < 6; i++) { await page.mouse.wheel(0, 60); await film.frames(12); } await film.frames(60); await film.still('race-scrolled'); }
  await film.stop();
});

/** The research screen at turn 203: what each technology unlocks, and a unit card that says what it beats. */
clip('research', async (page) => {
  await load(page, save(LATE));
  const film = await Film.start(page, 'research');
  await film.frames(20);
  await film.glide(await centre(page, page.locator('.topbar .research-chip')), 30, { x: 700, y: 400 });
  await press(page, film);
  await film.frames(70);
  const d = page.locator('dialog.tech-modal');
  const unit = d.locator('.era-unlock.unit').first();
  if (await unit.count()) { await unit.scrollIntoViewIfNeeded(); await film.glide(await centre(page, unit), 36); await film.frames(170); }
  else await film.frames(120);
  await film.still('research');
  await film.stop();
});

/** A technology lands and a new era begins: the last technology of the era is one turn from done on the turn-117 save. */
clip('era', async (page) => {
  const s = save(MID, { patch: (g) => {
    const p = g.players[0];
    let era = 0;
    while (eraTechs(era).length && eraTechs(era).every((t) => p.techs.includes(t))) era++;
    const todo = eraTechs(era).filter((t) => !p.techs.includes(t));
    p.techs.push(...todo.slice(0, -1));
    p.researching = todo[todo.length - 1];
    p.research_queue = [todo[todo.length - 1]];
    p.science_stored = 100_000;
    console.log('era patch: era', era, 'finishing', p.researching);
  } });
  await load(page, s);
  const home = await worldOf(page, { q: 8, r: 23 });
  await settle(page, { ...home, zoom: 1.0 });
  await resolveTurn(page, s.turn);
  const film = await Film.start(page, 'era');
  await film.frames(150);
  await film.still('era-tech');
  for (let i = 0; i < 6 && !(await page.getByTestId('era-takeover').count()); i++) { await page.keyboard.press('Escape'); await film.frames(20); }
  await film.frames(330);
  await film.still('era');
  await film.stop();
});

/** A new game: the capital trains a Scout, every unit explores, and the map opens up over the first turns. */
clip('opening', async (page) => {
  await page.clock.install();
  await page.addInitScript((prefs) => {
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    for (const [k, v] of Object.entries(prefs)) localStorage.setItem(k, v as string);
  }, { 'legatum.combat-fx': JSON.stringify({ enabled: true }) });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await page.getByLabel('Map seed', { exact: true }).fill('20261121');
  await page.getByRole('button', { name: /^Start game/ }).click();
  await until(page, async () => (await page.getByTestId('turn-number').textContent().catch(() => '')) === '1' && !(await page.locator('.loading-screen').count()), 'turn 1');
  await run(page, 1500);
  // The orders a player gives on turn 1: train a Scout, research the first technology, send every unit exploring.
  const given = await page.evaluate((tech) => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const orders: any[] = [];
    for (const u of v.units.filter((u: any) => u.owner === v.me && (u.kind === 'Scout' || u.military))) orders.push({ type: 'SetAutomation', unit: u.id, automation: 'Explore' });
    const city = v.cities.find((c: any) => c.owner === v.me);
    if (city) orders.push({ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Scout' } });
    orders.push({ type: 'SetResearch', tech });
    s.editOrders(() => orders);
    s.select(null);
    return orders.length;
  }, eraTechs(0)[0]);
  console.log('opening orders', given);
  await park(page);
  await run(page, 300);
  const home = await page.evaluate(() => ({ ...(window as unknown as W).legatumMap.camera }));
  const cam = { ...home, zoom: home.zoom * 1.25 }, wide = { ...home, zoom: home.zoom * 0.6 };
  await settle(page, cam);
  const TOTAL = 60 + 9 * 48;
  const ease = push(cam, wide, TOTAL);
  const film = await Film.start(page, 'opening');
  let at = 0;
  const roll = async (n: number) => { const from = at; await film.frames(n, (i) => ease(from + i)); at += n; };
  await roll(60);
  await film.still('opening');
  for (let turn = 1; turn <= 9; turn++) {
    // A unit whose route was blocked stops exploring: send it out again, as a player would.
    if (turn > 1) await page.evaluate(() => {
      const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
      const idle = v.units.filter((u: any) => u.owner === v.me && (u.kind === 'Scout' || u.military) && !u.automation);
      s.editOrders((orders: any[]) => [...orders.filter((o) => !(o.type === 'SetAutomation' && idle.some((u: any) => u.id === o.unit))), ...idle.map((u: any) => ({ type: 'SetAutomation', unit: u.id, automation: 'Explore' }))]);
    });
    await resolveTurn(page, turn);
    await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
    await roll(48);
    for (let i = 0; i < 4 && (await page.locator('.moment-card, [data-testid="era-takeover"], .alarm-screen').count()); i++) { await page.keyboard.press('Escape'); await film.frames(4, () => ease(at)); }
    const close = page.getByRole('button', { name: 'Close report', exact: true });
    if (await close.count() && await close.isVisible()) { await close.click(); await park(page); }
  }
  await film.frames(40, () => wide);
  await film.still('opening-end');
  await film.stop();
});

/** The people picker: the nine finished peoples, each with its strengths and the price that pays for them. */
clip('factions', async (page) => {
  await page.clock.install();
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Single Player/ }).click();
  await run(page, 600);
  await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
  await run(page, 900);
  const film = await Film.start(page, 'factions');
  await film.frames(20);
  for (const f of PEOPLES) {
    const opt = page.getByRole('option', { name: new RegExp(`^${FACTION_INFO[f].name}`) });
    await film.glide(await centre(page, opt), 8);
    await press(page, film);
    await film.frames(52);
    await film.still(`people-${f.toLowerCase()}`);
  }
  const pane = await page.evaluate(() => { const g = document.querySelector('dialog[open] .fs-gallery')!.getBoundingClientRect(), b = document.querySelector('dialog[open] .fs-body')!.getBoundingClientRect(); return { x: g.right, y: b.top, w: b.right - g.right, h: b.height }; });
  writeFileSync(join(OUT, 'factions.pane.json'), JSON.stringify(pane));
  await film.stop();
});

/** A peace offer on the table at turn 203: who offers, and what accepting obliges you to do. */
clip('diplomacy', async (page) => {
  await load(page, save(LATE));
  const film = await Film.start(page, 'diplomacy');
  await film.frames(30);
  const inbox = page.getByRole('button', { name: /Diplomacy inbox/ }).first();
  if (!(await inbox.count())) {
    // The link sits in the opened race panel.
    await film.glide(await centre(page, page.locator('summary', { hasText: 'All empires' }).first()), 30, { x: 700, y: 420 });
    await press(page, film);
    await film.frames(40);
  }
  await inbox.scrollIntoViewIfNeeded();
  await film.frames(10);
  await film.glide(await centre(page, inbox), 30);
  await press(page, film);
  await inbox.click();
  await film.frames(240);
  await film.still('diplomacy');
  await film.stop();
});

/** A city at turn 203: what it makes, what it builds next and how long each choice takes. */
clip('city', async (page) => {
  await load(page, save(LATE));
  const pos = await state(page, (_s, v) => v.cities.find((c: any) => c.owner === v.me && c.name === 'Thun').pos) as Hex;
  const c = await worldOf(page, pos);
  await settle(page, { ...c, zoom: 1.3 });
  const film = await Film.start(page, 'city');
  await film.frames(20);
  await film.glide(await pagePoint(page, pos), 30, { x: 700, y: 520 });
  await press(page, film);
  await film.frames(220);
  await film.still('city');
  await film.stop();
});

/** The four stills of the launch post at 2560x1440: the late map, a people's sheet, a battle forecast and the score ledger. */
test.describe('stills', () => {
  test.use({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 2 });
  clip('stills-map', async (page) => {
    await load(page, save(LATE));
    const home = await worldOf(page, { q: 10, r: 21 });
    await settle(page, { x: home.x + 40, y: home.y - 30, zoom: 0.78 });
    await run(page, 600);
    await snap(page, 'map-late-game', 2);
  });
  clip('stills-forecast', async (page) => {
    await load(page, save(LATE, { seat: 7 }));
    const w = await war(page);
    const o = w.orders.find((o: any) => o.type === 'MoveUnit') ?? w.orders[0];
    const target: Hex = o.to ?? o.target;
    await page.evaluate((id) => { (window as unknown as W).legatum.getState().select({ kind: 'unit', id }, true); }, o.unit);
    await run(page, 700);
    const c = await worldOf(page, target);
    await settle(page, { x: c.x - 60, y: c.y, zoom: 1.5 });
    const p = await pagePoint(page, target);
    await page.mouse.move(p.x - 30, p.y + 20);
    await run(page, 200);
    await page.mouse.move(p.x, p.y, { steps: 6 });
    await run(page, 1500);
    await snap(page, 'battle-forecast', 2);
  });
  clip('stills-race', async (page) => {
    await load(page, save(LATE));
    await page.getByRole('button', { name: 'All objectives', exact: true }).first().click();
    await park(page);
    await run(page, 1500);
    await snap(page, 'victory-race', 2);
  });
  clip('stills-people', async (page) => {
    await page.clock.install();
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Single Player/ }).click();
    await run(page, 600);
    await page.getByRole('button', { name: /^Seat 1 faction:/ }).click();
    await run(page, 900);
    for (const f of ['Mongols', 'Sparta', 'Switzerland'] as const) {
      await page.getByRole('option', { name: new RegExp(`^${FACTION_INFO[f].name}`) }).click();
      await park(page);
      await run(page, 1500);
      await snap(page, `people-sheet-${f.toLowerCase()}`, 2);
    }
    const pane = await page.evaluate(() => { const g = document.querySelector('dialog[open] .fs-gallery')!.getBoundingClientRect(), b = document.querySelector('dialog[open] .fs-body')!.getBoundingClientRect(); return { x: g.right, y: b.top, w: b.right - g.right, h: b.height }; });
    writeFileSync(join(OUT, 'stills', 'people-sheet.pane.json'), JSON.stringify(pane));
  });
});

export type { Cam };
