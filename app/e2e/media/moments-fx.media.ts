// Payoff-moment animation captures (docs/design/moments-fx/): a city founded, a building or wonder finished, a tech
// and an era, a capture, units marching. Turns start from copies of the designer's 8.9.6 Swiss-game saves and resolve
// live in the app. The original orders tape is unavailable for the construction clips, so their sparse orders.json
// reconstructs the needed founding order and the staged Great Library build. Frames use Playwright's fake clock.
//   MOMENT_SAVES: directory with T<turn>.json.gz, orders.json and <name>.legatum.gz copies (default e2e/media/moments-saves)
//   MEDIA_OUT: where clips, stills and probes go; MEDIA_ONLY: comma list of test names.
import { gunzipSync } from 'node:zlib';
import { readFileSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { OUT, wanted } from './recorder';

const SAVES = process.env.MOMENT_SAVES ?? join(process.cwd(), 'e2e/media/moments-saves');
type Hex = { q: number; r: number };
type Cam = { x: number; y: number; zoom: number };
type W = Window & {
  legatum: { getState(): any; setState(p: unknown): void };
  legatumMap: { camera: Cam; redraw(): void; toScreen(h: Hex): { x: number; y: number }; terrain: { missing: number; exposed: number; level: number; wanted: number }; perf?: any };
};

/** A state rebuilt from the designer's 8.9.6 game, wrapped as a save. */
function spineSave(turn: number, mutate?: (game: any) => void) {
  let game = gunzipSync(readFileSync(join(SAVES, `T${turn}.json.gz`))).toString('utf8');
  // A copy can be staged for the shot; the files on disk are never written.
  if (mutate) { const g = JSON.parse(game); mutate(g); game = JSON.stringify(g); }
  return JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Player 1'], savedAt: new Date().toISOString(), game });
}
const ORDERS: Record<string, any[]> = JSON.parse(readFileSync(join(SAVES, 'orders.json'), 'utf8'));
/** Available reconstructed orders for a turn, excluding Hold and Heal in rebuilt states. */
const spineOrders = (turn: number): any[] => (ORDERS[turn] ?? []).filter((o) => o.type !== 'HoldUnit' && o.type !== 'HealUnit');

async function loadRaw(page: Page, raw: string, turn: number, prefs: Record<string, string> = {}, clock = true) {
  page.on('pageerror', (e) => console.log('pageerror', String(e.stack ?? e).slice(0, 800)));
  page.on('console', (m) => { if (m.type() === 'error') console.log('console.error', m.text().slice(0, 800)); });
  page.setDefaultTimeout(45_000);
  if (clock) await page.clock.install();
  await page.addInitScript(({ save, prefs }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', save);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    for (const [k, v] of Object.entries(prefs)) localStorage.setItem(k, v);
  }, { save: raw, prefs });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 120_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  if (clock) await page.clock.runFor(800); else await page.waitForTimeout(800);
  const toast = page.getByText(/^Save updated:/);
  if (await toast.count()) await toast.first().locator('xpath=..').getByRole('button').first().click().catch(() => undefined);
  await page.mouse.move(2, 2);
}

async function worldOf(page: Page, h: Hex): Promise<{ x: number; y: number }> {
  return page.evaluate((h) => {
    const m = (window as unknown as W).legatumMap, c = m.camera, p = m.toScreen(h);
    const box = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: c.x + (p.x - box.width / 2) / c.zoom, y: c.y + (p.y - box.height / 2) / c.zoom };
  }, h);
}

/** Set the camera and wait until the terrain is complete at the zoom's level. */
async function settle(page: Page, cam: Cam) {
  for (let n = 0; n < 400; n++) {
    const ok = await page.evaluate((cam) => {
      const m = (window as unknown as W).legatumMap;
      Object.assign(m.camera, cam);
      m.redraw();
      const t = m.terrain;
      return t.missing === 0 && t.exposed === 0 && t.level === t.wanted;
    }, cam);
    await page.clock.runFor(17);
    if (ok && n >= 3) return;
  }
}

/** Cinematic camera: nothing selected and the HUD hidden except `keep` (the map, and the moment cards if asked). */
async function cinema(page: Page, keep = '[data-game-map]') {
  await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
  const sel = keep.split(',').map((k) => `${k},${k} *`).join(',');
  await page.evaluate((css) => {
    const el = document.createElement('style');
    el.id = 'media-cinema';
    el.textContent = css;
    document.head.appendChild(el);
  }, `body *{visibility:hidden!important}${sel}{visibility:visible!important}.moment-dismiss{visibility:hidden!important}`);
}

/** Step the fake clock and every Web Animation by `ms`. */
async function tick(page: Page, ms: number) {
  await page.clock.runFor(ms);
  await page.evaluate((ms) => {
    const w = window as unknown as { __vt?: WeakMap<Animation, number> };
    const vt = (w.__vt ??= new WeakMap());
    for (const a of document.getAnimations()) {
      const t = vt.has(a) ? vt.get(a)! + ms : 0;
      vt.set(a, t);
      a.pause();
      a.currentTime = t;
    }
  }, ms);
}

function encode(dir: string, mp4: string) {
  const r = spawnSync('ffmpeg', ['-y', '-loglevel', 'error', '-framerate', '60', '-i', join(dir, 'f%05d.png'),
    '-vf', 'scale=1920:1080:flags=lanczos,format=yuv420p', '-c:v', 'libx264', '-crf', '16', '-preset', 'medium', '-movflags', '+faststart', mp4], { encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`ffmpeg failed for ${mp4}: ${r.stderr}`);
}

const PAYOFF = new Set(['CityFounded', 'ProductionCompleted', 'TechResearched', 'EraEntered', 'CityCaptured', 'CityRazed', 'TileClaimed',
  'MilestoneClaimed', 'EraObjectiveScored', 'FactionGoalCompleted', 'WarDeclared', 'PeaceEstablished', 'TreatyActivated', 'TreatyEnded', 'WarJoined', 'CityGrew']);

/** The payoff events of the turn that just resolved, as this seat saw them. */
async function payoffs(page: Page) {
  return page.evaluate((keep) => {
    const s = (window as unknown as W).legatum.getState();
    const ev = (s.report?.events?.[s.seat] ?? []) as any[];
    const me = s.views[s.seat].me;
    return { me, events: ev.map((e, i) => ({ i, ...e })).filter((e) => keep.includes(e.type)) };
  }, [...PAYOFF]);
}

/** End the turn on a frozen clock and wait for the result; the clock is paused at the instant it arrives. */
async function resolveTurn(page: Page, turn: number, orders: any[]) {
  if (orders.length) await page.evaluate((orders) => { (window as unknown as W).legatum.getState().editOrders(() => orders); }, orders);
  await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
  const now = await page.evaluate(() => Date.now());
  await page.clock.pauseAt(now + 3000);
  const hud = (on: boolean) => page.evaluate((on) => { const el = document.getElementById('media-cinema') as HTMLStyleElement | null; if (el) el.disabled = on; }, on);
  await hud(true);
  // A turn can open on a takeover, an unlock card or an interstitial: close it so the turn dock is there.
  // An alarm (war declared), a takeover or an unlock card can cover the dock: close whatever is up first.
  const covers = page.locator('.alarm-screen, .era-takeover, .moment-card.unlock, [role="dialog"][aria-modal="true"]');
  for (let i = 0; i < 8 && ((await covers.count()) > 0 || !(await page.locator('[data-overlay-slot="turn-dock"]').count())); i++) {
    await page.keyboard.press('Escape');
    await page.clock.runFor(400);
    await page.waitForTimeout(200);
  }
  try { await endPlanning(page); }
  catch (e) { await page.screenshot({ path: join(OUT, `stuck-T${turn}.png`) }); throw e; }
  const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
  await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
  await hud(false);
  await page.mouse.move(2, 2);
  const turnNow = () => page.evaluate(() => document.querySelector('[data-testid="turn-number"]')?.textContent ?? '');
  for (let i = 0; (await turnNow()) !== String(turn + 1); i++) {
    if (i > 3000) throw new Error(`turn ${turn} did not resolve`);
    await page.waitForTimeout(30);
    await page.clock.runFor(5);
  }
}

/** Film `frames` frames from now (camera from `cam(i)`); the recorded MP4 plays at 60 fps. */
async function film(page: Page, name: string, frames: number, cam: ((i: number) => Cam) | null, stills: number[] = [], beforeFrame?: (i: number) => Promise<void>, frameMs = 17) {
  const dir = join(OUT, `${name}.frames`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const cdp = await page.context().newCDPSession(page);
  await page.evaluate(() => (window as unknown as W).legatumMap.perf?.reset());
  for (let i = 0; i < frames; i++) {
    if (beforeFrame) await beforeFrame(i);
    await page.evaluate((c) => { const m = (window as unknown as W).legatumMap; if (c) Object.assign(m.camera, c); m.redraw(); }, cam ? cam(i) : null);
    await tick(page, beforeFrame ? frameMs : i % 3 === 2 ? 16 : 17);
    const shot = await cdp.send('Page.captureScreenshot', { format: 'png' });
    const png = Buffer.from(shot.data, 'base64');
    writeFileSync(join(dir, `f${String(i).padStart(5, '0')}.png`), png);
    if (stills.includes(i)) writeFileSync(join(OUT, 'stills', `${name}-f${String(i).padStart(3, '0')}.png`), png);
  }
  await cdp.detach();
  const perf = await page.evaluate(() => (window as unknown as W).legatumMap.perf ?? null);
  writeFileSync(join(OUT, `${name}.perf.json`), JSON.stringify(perf, null, 1));
  encode(dir, join(OUT, `${name}.mp4`));
  rmSync(dir, { recursive: true, force: true });
  console.log('clip', name, frames, 'frames');
}

const live = (title: string, fn: (page: Page) => Promise<void>) => test(title, async ({ page }) => {
  test.skip(!wanted(title));
  test.setTimeout(3_600_000);
  mkdirSync(join(OUT, 'stills'), { recursive: true });
  await fn(page);
});

/** Probe: replay his turns `from`..`to` and log every payoff event (no frames), to choose the clips. */
for (const [from, to] of [[61, 64], [150, 157], [107, 112]] as const) live(`probe${from}`, async (page) => {
  await loadRaw(page, spineSave(from), from);
  const log: unknown[] = [];
  for (let t = from; t < to; t++) {
    await resolveTurn(page, t, spineOrders(t));
    log.push({ turn: t, ...(await payoffs(page)) });
    writeFileSync(join(OUT, `probe${from}.json`), JSON.stringify(log, null, 1));
    await page.clock.runFor(4000);
  }
});


/** A slow push in from `a` to `b` over `n` frames. */
const push = (a: Cam, b: Cam, n: number) => (i: number): Cam => {
  const k = Math.min(1, i / n);
  const e = k * k * (3 - 2 * k);
  return { x: a.x + (b.x - a.x) * e, y: a.y + (b.y - a.y) * e, zoom: a.zoom * Math.pow(b.zoom / a.zoom, e) };
};
const STILLS = [10, 40, 70, 90, 110, 130, 150, 170, 190, 220, 260];
const FRAMES = Number(process.env.MOMENT_FRAMES ?? 330);

/**
 * Replay his turns from the save at `from` up to `turn`, then film `turn` resolving with the camera on `focus` (read from
 * the state just before the turn ends: a settler about to found, a city about to finish a building).
 */
/** Evaluate `src` (an arrow function's source) on the store's state in the page. */
const onState = <T>(page: Page, src: string) => page.evaluate((src) => new Function('s', `return (${src})(s)`)((window as unknown as W).legatum.getState()), src) as Promise<T>;

async function filmTurn(page: Page, name: string, from: number, turn: number, focus: string, zoom = 2.2, orders?: string, mutate?: (game: any) => void, keep?: string) {
  await loadRaw(page, spineSave(from, mutate), from);
  for (let t = from; t < turn; t++) {
    await resolveTurn(page, t, spineOrders(t));
    await page.clock.runFor(6000);
  }
  const at = await onState<Hex>(page, focus);
  const w = await worldOf(page, at);
  const cam = push({ x: w.x, y: w.y - 8, zoom }, { x: w.x, y: w.y - 4, zoom: zoom * 1.12 }, FRAMES);
  await settle(page, cam(0));
  await cinema(page, keep);
  const extra = orders ? await onState<any[]>(page, orders) : [];
  await resolveTurn(page, turn, [...spineOrders(turn), ...extra]);
  writeFileSync(join(OUT, `${name}.events.json`), JSON.stringify(await payoffs(page), null, 1));
  await film(page, name, FRAMES, cam, STILLS);
}

/** The settler's hex, for a founding clip. */
const settlerAt = (unit: number) => `(s) => s.views[s.seat].units.find((u) => u.id === ${unit}).pos`;

/** Moment 1: Thun founded on turn 63 of the rebuilt game (unit 94), and a late city on turn 155 (unit 254). */
for (const [label, from, turn, unit] of [['found63', 61, 63, 94], ['found155', 151, 155, 254]] as const) live(label, async (page) => {
  await filmTurn(page, label, from, turn, settlerAt(unit));
});

/** The capital's city hex, by name. */
const cityAt = (name: string) => `(s) => s.views[s.seat].cities.find((c) => c.name === ${JSON.stringify(name)}).pos`;

/**
 * Moment 2: a Great Library. His game built none, so a copy of turn 112 starts a Bellinzona build at zero stored
 * Production on its chosen tile. The later turns are resolved live with the orders available in orders.json.
 */
live('wonder151', async (page) => {
  const from = 112, to = 151;
  await loadRaw(page, spineSave(from, (g) => {
    const c = Object.values(g.cities as Record<string, any>).find((x) => x.name === 'Bellinzona');
    c.producing = { kind: 'Wonder', building: 'RelayUplink', tile: { q: 3, r: 25 } };
    if (!c.buildings.includes('Laboratory')) c.buildings.push('Laboratory');
    c.production_queue = [];
    c.production_stored = 0;
  }), from);
  const at = await onState<Hex>(page, cityAt('Bellinzona'));
  const site = { q: at.q, r: at.r + 1 };
  const w = await worldOf(page, site);
  const cam = { x: w.x, y: w.y - 4, zoom: 2.7 };
  await settle(page, cam);
  await cinema(page);
  const perTurn = Number(process.env.MOMENT_FRAMES ?? 24);
  const progress: { stored: number; cost: number; built: boolean }[] = [];
  await film(page, 'wonder151', (to - from + 1) * perTurn, () => cam, [], async (i) => {
    if (i % perTurn === 0) {
      await resolveTurn(page, from + Math.floor(i / perTurn), spineOrders(from + Math.floor(i / perTurn)));
      progress.push(await onState<{ stored: number; cost: number; built: boolean }>(page, `(s) => { const c = s.views[s.seat].cities.find(c => c.name === 'Bellinzona'); return { stored: c.detail?.production_stored ?? 0, cost: c.detail?.production_cost ?? 600, built: c.great_works.includes('RelayUplink') }; }`));
    }
  }, 100);
  writeFileSync(join(OUT, 'wonder151.progress.json'), JSON.stringify(progress, null, 2));
  for (const threshold of [.1, .4, .75]) expect(progress.some((p) => p.stored / p.cost >= threshold)).toBe(true);
  expect(progress.at(-1)?.built).toBe(true);
});

/** Winterthur's wall circuit, staged at 20/60 Production so its section-by-section step is visible. */
live('walls63', async (page) => {
  const from = 61, turns = 3;
  await loadRaw(page, spineSave(from, (g) => {
    const c = Object.values(g.cities as Record<string, any>).find((x) => x.name === 'Winterthur');
    c.production_stored = 20;
  }), from);
  const at = await onState<Hex>(page, cityAt('Winterthur'));
  const w = await worldOf(page, at);
  const cam = { x: w.x, y: w.y - 4, zoom: 2.5 };
  await settle(page, cam);
  await cinema(page);
  const perTurn = Number(process.env.MOMENT_FRAMES ?? 24);
  await film(page, 'walls63', turns * perTurn, () => cam, [], async (i) => {
    if (i % perTurn === 0) await resolveTurn(page, from + Math.floor(i / perTurn), spineOrders(from + Math.floor(i / perTurn)));
  }, 100);
});

/** Frame timing on the real clock for `ms`: draw cost per frame, the moments' share, and rAF gaps. */
async function measure(page: Page, ms: number) {
  return page.evaluate(async ({ ms }) => {
    const m = (window as unknown as W).legatumMap;
    m.perf.reset();
    const gaps: number[] = [];
    let last = performance.now();
    const t0 = last;
    await new Promise<void>((done) => {
      const step = (t: number) => { gaps.push(t - last); last = t; if (t - t0 < ms) requestAnimationFrame(step); else done(); };
      requestAnimationFrame(step);
    });
    const p = m.perf;
    gaps.sort((a, b) => a - b);
    const q = (f: number) => +gaps[Math.min(gaps.length - 1, Math.floor(gaps.length * f))].toFixed(1);
    const r = (v: number) => +v.toFixed(2);
    return { frames: p.frames, drawMean: r(p.drawMs / Math.max(1, p.frames)), drawMax: r(p.maxMs), momentFrames: p.momentFrames, momentMean: r(p.momentMs / Math.max(1, p.momentFrames)), momentMax: r(p.momentMaxMs), gapP50: q(0.5), gapP95: q(0.95), gapMax: q(1) };
  }, { ms });
}

/**
 * Frame cost on the real clock (no stepping) around turn 63 (Thun founded, Walls at Winterthur, and a war):
 * idle with idle life on, the moments playing, and a stress replay of them copied 20 times across the screen; then the
 * same idle with map animations off. The box draws without a GPU, so compare on with off rather than reading absolutes.
 */
live('perf63', async (page) => {
  const results: Record<string, unknown> = {};
  await loadRaw(page, spineSave(61), 61, {}, false);
  for (const t of [61, 62]) {
    await page.evaluate((o) => (window as unknown as W).legatum.getState().editOrders(() => o), spineOrders(t));
    for (let i = 0; i < 8 && !(await page.locator('[data-overlay-slot="turn-dock"]').count()); i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(300); }
    await endPlanning(page);
    const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
    await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
    await expect(page.getByTestId('turn-number')).toHaveText(String(t + 1), { timeout: 120_000 });
    await page.waitForTimeout(5000);
  }
  const at = await onState<Hex>(page, settlerAt(94));
  const w = await worldOf(page, at);
  await page.evaluate((c) => { Object.assign((window as unknown as W).legatumMap.camera, c); }, { x: w.x + 60, y: w.y - 10, zoom: 1.6 });
  for (let i = 0; i < 8 && !(await page.locator('[data-overlay-slot="turn-dock"]').count()); i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(300); }
  await page.waitForTimeout(1500);
  results.idle = await measure(page, 2000);
  await page.evaluate((o) => (window as unknown as W).legatum.getState().editOrders(() => o), spineOrders(63));
  await endPlanning(page);
  const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
  await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
  await expect(page.getByTestId('turn-number')).toHaveText('64', { timeout: 120_000 });
  results.playback = await measure(page, 4000);
  results.stressCopies = await page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState();
    const plan = s.anim?.moments;
    if (!plan) throw new Error('no moment plan after the turn');
    const v = s.views[s.seat];
    // Every visible city gets the turn's founding, a building and a seal, all at once.
    const cities = v.cities.filter((c: any) => c.visible).slice(0, 20);
    const f0 = plan.founds[0], b0 = plan.builds[0];
    const founds = cities.map((c: any, i: number) => ({ ...f0, city: c.id, at: c.pos, t: 300 + i * 20, seed: i * 7 }));
    const builds = cities.map((c: any, i: number) => ({ ...b0, city: c.id, at: c.pos, tile: -1, walls: false, wonder: i % 4 === 0, t: 600 + i * 20, seed: i * 11 }));
    const seals = cities.map((c: any, i: number) => ({ kind: i % 2 ? 'vp' : 'alliance', at: c.pos, owner: c.owner, vp: 1, t: 900 + i * 15, seed: i * 13 }));
    (window as unknown as W).legatum.setState({ anim: { ...s.anim, start: performance.now(), plan: null, moments: { ...plan, founds, builds, seals, total: 6000 } } });
    return { founds: founds.length, builds: builds.length, seals: seals.length };
  });
  results.stress = await measure(page, 4000);
  writeFileSync(join(OUT, 'perf63.json'), JSON.stringify(results, null, 1));
  console.log('perf63', JSON.stringify(results));
});

live('perf-off', async (page) => {
  await loadRaw(page, spineSave(151), 151, { 'legatum.combat-fx': JSON.stringify({ enabled: false, speed: 1 }) }, false);
  await page.waitForTimeout(1500);
  const on = await measure(page, 2000);
  writeFileSync(join(OUT, 'perf-off.json'), JSON.stringify({ idleOff: on }, null, 1));
  console.log('perf-off', JSON.stringify(on));
});

live('perf-idle', async (page) => {
  await loadRaw(page, spineSave(151), 151, {}, false);
  await page.waitForTimeout(1500);
  const on = await measure(page, 2000);
  writeFileSync(join(OUT, 'perf-idle.json'), JSON.stringify({ idleOn: on }, null, 1));
  console.log('perf-idle', JSON.stringify(on));
});

/** Moment 4: the capture of Lugano, turn 200 into 201 (his moves and shots): the banner flips and the land floods Swiss. */
live('capture200', async (page) => {
  await loadRaw(page, spineSave(200), 200);
  const w = await worldOf(page, { q: 6, r: 20 });
  const cam = push({ x: w.x - 10, y: w.y - 5, zoom: 2.3 }, { x: w.x - 5, y: w.y - 10, zoom: 2.6 }, FRAMES);
  await settle(page, cam(0));
  await cinema(page);
  await resolveTurn(page, 200, (ORDERS[200] ?? []).filter((o) => o.type === 'MoveUnit' || o.type === 'RangedAttack'));
  writeFileSync(join(OUT, 'capture200.events.json'), JSON.stringify(await payoffs(page), null, 1));
  await film(page, 'capture200', Math.max(FRAMES, 420), cam, STILLS);
});

/**
 * Moment 3: entering an era. Turn 151 of his game finishes the Renaissance's last technology: the era takeover (ink-wash
 * reveal, parallax, the title and rule drawn in), then, once it is closed, the technology's unlock card (stamp, rays,
 * the painted unlocks washing in) over the map, where his towns are being rebuilt in the new era.
 */
live('eraup151', async (page) => {
  await filmTurn(page, 'eraup151', 151, 151, cityAt('Bellinzona'), 2.2, undefined, undefined, '[data-game-map],.moment-lane');
  // Close the takeover and film what follows: the unlock card and the board.
  await page.keyboard.press('Escape');
  await film(page, 'unlock151', 300, null, [20, 60, 120]);
  // The era takeover queues behind the unlock card: close the card and film it.
  await page.keyboard.press('Escape');
  await film(page, 'era151', 300, null, [10, 40, 80, 120, 200]);
});

/** A plain building on the same turn: Aarau's Granary, built on the edge of the town and then joining it. */
live('granary151', async (page) => {
  await filmTurn(page, 'granary151', 151, 151, cityAt('Aarau'), 2.8);
});
