// Combat animation captures (docs/design/combat-fx/). Real turns from the designer's 8.9.6 Swiss game, rebuilt from its
// tape (states copied from the trailer's v2 saves, never the designer's own files), resolved live in the real app and
// filmed frame by frame on Playwright's fake clock, so every frame is exactly 1/60 s of game time.
//   FX_SAVES: directory with T<turn>.json.gz, orders.json and <name>.legatum.gz copies (default e2e/media/fx-saves)
//   MEDIA_OUT: where clips, stills and probes go; MEDIA_ONLY: comma list of test names.
import { gunzipSync } from 'node:zlib';
import { readFileSync, mkdirSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { OUT, wanted } from './recorder';
import { decodeOrders } from '../../src/orderDecoding';

const SAVES = process.env.FX_SAVES ?? join(process.cwd(), 'e2e/media/fx-saves');
type Hex = { q: number; r: number };
type Cam = { x: number; y: number; zoom: number };
type W = Window & {
  legatum: { getState(): any };
  legatumMap: { camera: Cam; redraw(): void; toScreen(h: Hex): { x: number; y: number }; terrain: { missing: number; exposed: number; level: number; wanted: number }; perf?: any };
};

/** A state rebuilt from the designer's 8.9.6 game, wrapped as a save. */
function spineSave(turn: number) {
  const game = gunzipSync(readFileSync(join(SAVES, `T${turn}.json.gz`))).toString('utf8');
  return JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Player 1'], savedAt: new Date().toISOString(), game });
}
const spineOrders = (turn: number): any[] => JSON.parse(readFileSync(join(SAVES, 'orders.json'), 'utf8'))[turn] ?? [];
function namedSave(name: string): { raw: string; turn: number } {
  const raw = gunzipSync(readFileSync(join(SAVES, `${name}.legatum.gz`))).toString('utf8');
  return { raw, turn: JSON.parse(JSON.parse(raw).game).turn };
}

const pageErrors: string[] = [];
async function loadRaw(page: Page, raw: string, turn: number, clock: boolean, prefs: Record<string, string> = {}) {
  page.on('pageerror', (e) => { pageErrors.push(String(e.stack ?? e)); console.log('pageerror', String(e.stack ?? e).slice(0, 800)); });
  page.on('console', (m) => { if (m.type() === 'error') console.log('console.error', m.text().slice(0, 800)); });
  if (clock) await page.clock.install();
  await page.addInitScript(({ save, prefs }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', save);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    for (const [k, v] of Object.entries(prefs)) localStorage.setItem(k, v);
  }, { save: raw, prefs });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  try { await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 60_000 }); }
  catch (e) {
    await page.screenshot({ path: join(OUT, `load-${turn}-FAILED.png`) });
    writeFileSync(join(OUT, `load-${turn}-FAILED.txt`), await page.evaluate(() => document.body.innerText.slice(0, 3000)));
    throw e;
  }
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
async function settle(page: Page, cam: Cam, clock: boolean) {
  for (let n = 0; n < 400; n++) {
    const ok = await page.evaluate((cam) => {
      const m = (window as unknown as W).legatumMap;
      Object.assign(m.camera, cam);
      m.redraw();
      const t = m.terrain;
      return t.missing === 0 && t.exposed === 0 && t.level === t.wanted;
    }, cam);
    if (clock) await page.clock.runFor(17); else await page.waitForTimeout(17);
    if (ok && n >= 3) return;
  }
}

/** Cinematic camera: nothing selected and every HUD layer hidden (moment cards too, unless `moments`); the live map is untouched. */
async function cinema(page: Page, moments = false) {
  await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
  const keep = moments ? '[data-game-map],.moment-lane' : '[data-game-map]';
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

/** What the viewer's events say happened, compactly (for designing and checking the choreography). */
async function dumpTurn(page: Page) {
  return page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState();
    const ev = (s.report?.events?.[s.seat] ?? []) as any[];
    const keep = new Set(['Battle', 'UnitMoved', 'UnitDestroyed', 'CityCaptured', 'CityRazed', 'AirStrike', 'AttackPursuit', 'MoveBlocked', 'Bounce', 'SettlerCaptured', 'CityReinforced']);
    return ev.map((e, i) => ({ i, ...e })).filter((e) => keep.has(e.type)).map((e) => {
      if (e.type === 'Battle') {
        const { attacker_strength: _a, defender_strength: _d, ...rest } = e;
        return rest;
      }
      return e;
    });
  });
}

/**
 * A real turn, filmed: issue `orders`, end the turn on a frozen clock, then grab `frames` frames at 1/60 s each from the
 * moment the result arrives (camera from `cam(i)`). Writes <name>.mp4, <name>.events.json and stills at `stills` frames.
 */
async function liveTurn(page: Page, name: string, turn: number, orders: any[], frames: number, cam: (i: number) => Cam, stills: number[] = []) {
  await settle(page, cam(0), true);
  if (orders.length) await page.evaluate((orders) => { (window as unknown as W).legatum.getState().editOrders(() => orders); }, orders);
  await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
  const before = await page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    return { me: v.me, units: v.units.map((u: any) => ({ id: u.id, kind: u.kind, owner: u.owner, pos: u.pos, hp: u.hp })), cities: v.cities.map((c: any) => ({ id: c.id, name: c.name, owner: c.owner, pos: c.pos, hp: c.hp, walls: c.walls })) };
  });
  // Freeze game time before the turn ends, so the resolution starts at a known instant and is filmed frame by frame.
  const now = await page.evaluate(() => Date.now());
  await page.clock.pauseAt(now + 3000);
  const hud = (on: boolean) => page.evaluate((on) => { const el = document.getElementById('media-cinema') as HTMLStyleElement | null; if (el) el.disabled = on; }, on);
  await hud(true);
  // A turn can open on a takeover or an interstitial: close it so the turn dock is there to end the turn.
  for (let i = 0; i < 6 && !(await page.locator('[data-overlay-slot="turn-dock"]').count()); i++) {
    await page.keyboard.press('Escape');
    await page.clock.runFor(400);
    await page.waitForTimeout(200);
  }
  try { await endPlanning(page); }
  catch (e) { await page.screenshot({ path: join(OUT, `${name}-FAILED.png`) }); throw e; }
  const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
  await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
  await hud(false);
  await page.mouse.move(2, 2);
  const turnNow = () => page.evaluate(() => document.querySelector('[data-testid="turn-number"]')?.textContent ?? '');
  for (let i = 0; (await turnNow()) !== String(turn + 1); i++) {
    if (i > 3000) throw new Error(`${name}: turn ${turn} did not resolve`);
    await page.waitForTimeout(30);
    await page.clock.runFor(5);
  }
  // The result arrived: from here every frame is exactly one 60 fps step of game time.
  const events = await dumpTurn(page);
  // How long the playback runs, and what it holds: the pacing budget is read from these.
  const plan = await page.evaluate(() => {
    const p = (window as unknown as W).legatum.getState().anim?.plan;
    if (!p) return null;
    const lastAction = Math.max(p.volley, ...p.clashes.map((c: any) => c.t), ...p.captures.map((c: any) => c.t), ...[...p.moves.values()].map((m: any) => m[m.length - 1].t1));
    return { totalMs: Math.round(p.total), lastActionMs: Math.round(lastAction), volleyMs: Math.round(p.volley), stepMs: Math.round(p.stepMs), shots: p.shots.length, clashes: p.clashes.length, deaths: p.deaths.length, captures: p.captures.length, movers: p.moves.size };
  });
  console.log('plan', name, JSON.stringify(plan));
  writeFileSync(join(OUT, `${name}.events.json`), JSON.stringify({ before, plan, events }, null, 1));
  if (frames < 0) frames = plan ? Math.min(720, Math.ceil(plan.totalMs / 1000 * 60) + 30) : 240;
  if (!frames) return;
  const dir = join(OUT, `${name}.frames`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const cdp = await page.context().newCDPSession(page);
  for (let i = 0; i < frames; i++) {
    await page.evaluate((c) => { Object.assign((window as unknown as W).legatumMap.camera, c); (window as unknown as W).legatumMap.redraw(); }, cam(i));
    await tick(page, i % 3 === 2 ? 16 : 17);
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

const LUGANO = { q: 6, r: 20 };
const live = (title: string, fn: (page: Page) => Promise<void>) => test(title, async ({ page }) => {
  test.skip(!wanted(title));
  test.setTimeout(3_600_000);
  mkdirSync(join(OUT, 'stills'), { recursive: true });
  await fn(page);
});

const FX_ON = { 'legatum.combat-fx': JSON.stringify({ enabled: true }) };
const FX_OFF = { 'legatum.combat-fx': JSON.stringify({ enabled: false }) };
const FRAMES = Number(process.env.FX_FRAMES ?? 540);
/** A slow push in from `a` to `b` over the clip. */
const push = (a: Cam, b: Cam, n: number) => (i: number): Cam => {
  const k = Math.min(1, i / n);
  const e = k * k * (3 - 2 * k);
  return { x: a.x + (b.x - a.x) * e, y: a.y + (b.y - a.y) * e, zoom: a.zoom * Math.pow(b.zoom / a.zoom, e) };
};
const STILLS = [20, 40, 55, 70, 85, 100, 120, 140, 160, 190, 220, 260, 300];

/** Every own military unit near `target` attacks it: shooters fire, the rest assault (what a player storming a city orders). */
async function assault(page: Page, target: Hex, reach = 3) {
  return page.evaluate(({ target, reach }) => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    const mine = v.units.filter((u: any) => u.owner === v.me && u.military && d(u.pos, target) <= reach);
    return mine.flatMap((u: any) => u.ranged > 0
      ? (d(u.pos, target) <= Math.max(1, u.range ?? 2) ? [{ type: 'RangedAttack', unit: u.id, target }] : [])
      : [{ type: 'MoveUnit', unit: u.id, to: target }]);
  }, { target, reach });
}

/** Every own military unit within `radius` of `center` attacks the nearest enemy it can: shooters in range fire, the rest charge. */
async function skirmish(page: Page, center: Hex, radius = 5) {
  return page.evaluate(({ center, radius }) => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    const war = new Set(v.players.filter((p: any) => p.team !== v.team).map((p: any) => p.id));
    const foes = v.units.filter((u: any) => u.military && war.has(u.owner) && d(u.pos, center) <= radius + 2);
    return v.units.filter((u: any) => u.owner === v.me && u.military && d(u.pos, center) <= radius).flatMap((u: any) => {
      const f = foes.map((e: any) => ({ e, k: d(e.pos, u.pos) })).sort((a: any, b: any) => a.k - b.k)[0];
      if (!f) return [];
      if (u.ranged > 0 && f.k <= Math.max(1, u.range ?? 2)) return [{ type: 'RangedAttack', unit: u.id, target: f.e.pos }];
      return f.k <= 2 ? [{ type: 'MoveUnit', unit: u.id, to: f.e.pos }] : [];
    });
  }, { center, radius });
}

/** Events only (no frames): what a turn contains. */
for (const t of [190, 199, 200]) live(`probe${t}`, async (page) => {
  await loadRaw(page, spineSave(t), t, true, FX_ON);
  await page.screenshot({ path: join(OUT, `probe${t}-loaded.png`) });
  writeFileSync(join(OUT, `probe${t}-loaded.txt`), await page.evaluate(() => document.body.innerText.slice(0, 4000)));
  await cinema(page);
  const w = await worldOf(page, LUGANO);
  await liveTurn(page, `probe${t}`, t, spineOrders(t), 0, () => ({ x: w.x, y: w.y, zoom: 1.5 }));
});

/** The siege of Lugano, turn 190 into 191 (the designer's orders): his cannon fire on the walls, the Mongol arquebusiers answer, his cavalry rides up. */
for (const [label, prefs] of [['siege190', FX_ON], ['siege190-off', FX_OFF]] as const) live(label, async (page) => {
  await loadRaw(page, spineSave(190), 190, true, prefs);
  await cinema(page);
  const w = await worldOf(page, { q: 5, r: 20 });
  await liveTurn(page, label, 190, spineOrders(190), FRAMES, push({ x: w.x - 10, y: w.y + 25, zoom: 2.0 }, { x: w.x + 15, y: w.y + 20, zoom: 2.35 }, FRAMES), STILLS);
});

/** The final capture, turn 200 into 201 (his orders): a cannon shot, and his riflemen walk into Lugano at 0 HP. */
live('capture200', async (page) => {
  await loadRaw(page, spineSave(200), 200, true, FX_ON);
  await cinema(page);
  const w = await worldOf(page, LUGANO);
  // The tape predates SleepUnit; translate its Hold and Heal orders before queuing the full turn.
  const orders = decodeOrders(spineOrders(200));
  await liveTurn(page, 'capture200', 200, orders, FRAMES, push({ x: w.x - 30, y: w.y + 25, zoom: 2.0 }, { x: w.x - 15, y: w.y + 15, zoom: 2.35 }, FRAMES), STILLS);
});

/** The siege at its height (turn 185, Lugano still walled): every Swiss unit around it attacks at once, and the walls answer. */
live('storm185', async (page) => {
  const { raw, turn } = namedSave('p896-mongol-attack-t180-2102');
  await loadRaw(page, raw, turn, true, FX_ON);
  await cinema(page);
  const orders = await assault(page, LUGANO);
  console.log('storm185 orders', JSON.stringify(orders));
  const w = await worldOf(page, LUGANO);
  await liveTurn(page, 'storm185', turn, orders, FRAMES, push({ x: w.x - 40, y: w.y + 25, zoom: 1.9 }, { x: w.x - 20, y: w.y + 15, zoom: 2.3 }, FRAMES), STILLS);
});

/** The modern front from another real game: artillery and tanks at Geneva (8.9.4), the AI's own turn. */
live('modern232', async (page) => {
  if (!existsSync(join(SAVES, 'p894-tanks-lost-t232-1615.legatum.gz'))) test.skip();
  const { raw, turn } = namedSave('p894-tanks-lost-t232-1615');
  await loadRaw(page, raw, turn, true, FX_ON);
  await cinema(page);
  const w = await worldOf(page, { q: 15, r: 20 });
  const orders = await skirmish(page, { q: 15, r: 20 });
  console.log('modern232 orders', JSON.stringify(orders));
  await liveTurn(page, 'modern232', turn, orders, FRAMES, push({ ...w, zoom: 1.8 }, { ...w, zoom: 2.1 }, FRAMES), STILLS);
});

/** Earlier wars from the designer's other games, each the human's turn with every unit near the front attacking. */
for (const [label, save, at] of [
  ['aarau151', 'p892-mongol-assault-t149-1416', { q: 8, r: 18 }],
  ['adrianople126', 'p894-mongol-attack-t126-1540', { q: 15, r: 15 }],
] as const) live(label, async (page) => {
  const { raw, turn } = namedSave(save);
  await loadRaw(page, raw, turn, true, FX_ON);
  await cinema(page);
  const orders = await skirmish(page, at, 5);
  console.log(label, 'orders', JSON.stringify(orders));
  const w = await worldOf(page, at);
  await liveTurn(page, label, turn, orders, FRAMES, push({ x: w.x, y: w.y + 20, zoom: 1.8 }, { x: w.x, y: w.y + 10, zoom: 2.1 }, FRAMES), STILLS);
});

/** Every aircraft in the human's hangars strikes the nearest enemy unit or city in its range; returns the orders and the first target. */
async function airstrike(page: Page) {
  return page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    const war = new Set(v.players.filter((p: any) => p.team !== v.team).map((p: any) => p.id));
    const targets: Hex[] = [...v.units.filter((u: any) => u.military && war.has(u.owner)).map((u: any) => u.pos), ...v.cities.filter((c: any) => war.has(c.owner) && c.visible).map((c: any) => c.pos)];
    const orders: any[] = [];
    for (const a of v.aircraft ?? []) {
      const base = v.cities.find((c: any) => c.id === a.city)?.pos;
      if (!base) continue;
      const reach = a.kind === 'Bomber' ? 8 : 5;
      const best = targets.map((t) => ({ t, k: d(t, base) })).filter((x) => x.k <= reach).sort((x, y) => x.k - y.k)[0];
      if (best) orders.push({ type: 'AirStrike', plane: a.id, target: best.t });
    }
    return { orders, first: orders[0]?.target ?? null };
  });
}

for (const [label, save] of [['air217', 'p897-bomber-rebase-t217-2011'], ['air172', 'p895-athens-bomber-1916']] as const) live(label, async (page) => {
  const { raw, turn } = namedSave(save);
  await loadRaw(page, raw, turn, true, FX_ON);
  await cinema(page);
  const { orders, first } = await airstrike(page);
  console.log(label, 'orders', JSON.stringify(orders));
  if (!first) throw new Error(`${label}: no aircraft can strike`);
  const w = await worldOf(page, first);
  await liveTurn(page, label, turn, orders, FRAMES, push({ x: w.x - 30, y: w.y + 10, zoom: 1.8 }, { x: w.x - 10, y: w.y, zoom: 2.1 }, FRAMES), STILLS);
});

/**
 * A whole front at war: every one of the human's military units attacks the nearest enemy it can reach this turn
 * (shooters in range fire; the melee line charges up to 2 tiles, riders up to 4), and melee units next to a broken
 * enemy city storm it. Returns the orders and a camera that frames them all.
 */
async function war(page: Page) {
  return page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    // At war: barbarians, or another team whose relation with ours is War (diplomacy.ts atWar).
    const rel = (team: number) => (v.diplomacy?.relations ?? []).find((r: any) => (r.a === v.team && r.b === team) || (r.b === v.team && r.a === team))?.state;
    const war = new Set(v.players.filter((p: any) => p.team !== v.team && (!v.diplomacy || v.diplomacy.legacy || rel(p.team) === 'War')).map((p: any) => p.id).concat([255]));
    const foes = v.units.filter((u: any) => u.military && war.has(u.owner));
    const weak = v.cities.filter((c: any) => war.has(c.owner) && c.visible && c.hp <= c.max_hp * 0.4);
    const orders: any[] = [];
    const at: Hex[] = [];
    for (const u of v.units.filter((x: any) => x.owner === v.me && x.military)) {
      const reach = u.max_moves ?? (['Horseman', 'Knight', 'Cavalry', 'Cataphract', 'Cuirassier', 'Tank', 'AutonomousArmour'].includes(u.kind) ? 4 : 2);
      const city = u.ranged > 0 ? null : weak.map((c: any) => ({ c, k: d(c.pos, u.pos) })).filter((x: any) => x.k <= 2).sort((a: any, b: any) => a.k - b.k)[0];
      const f = foes.map((e: any) => ({ e, k: d(e.pos, u.pos) })).sort((a: any, b: any) => a.k - b.k)[0];
      if (city) orders.push({ type: 'MoveUnit', unit: u.id, to: city.c.pos });
      else if (f && u.ranged > 0 && f.k <= Math.max(1, u.range ?? 2)) orders.push({ type: 'RangedAttack', unit: u.id, target: f.e.pos });
      else if (f && u.ranged <= 0 && f.k <= reach) orders.push({ type: 'MoveUnit', unit: u.id, to: f.e.pos });
      else continue;
      at.push(u.pos);
    }
    return { orders, at };
  });
}

/** A camera that frames all of `at` (world points) on the 1440x810 map, a little tighter than the bounding box. */
async function frameAll(page: Page, at: Hex[]): Promise<Cam> {
  const pts = await Promise.all(at.map((h) => worldOf(page, h)));
  if (!pts.length) return { x: 0, y: 0, zoom: 1 };
  const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
  // The densest front wins when the fronts are far apart: frame the middle 80% of the units.
  const q = (a: number[], f: number) => [...a].sort((m, n) => m - n)[Math.floor((a.length - 1) * f)];
  const x0 = q(xs, 0.1), x1 = q(xs, 0.9), y0 = q(ys, 0.1), y1 = q(ys, 0.9);
  const zoom = Math.max(0.75, Math.min(1.7, Math.min(1300 / Math.max(1, x1 - x0 + 160), 700 / Math.max(1, y1 - y0 + 160))));
  return { x: (x0 + x1) / 2, y: (y0 + y1) / 2, zoom };
}

/** Complex battles from real late-game saves (the user, 2026-10-24: make sure the animations work, and the pacing). */
for (const [label, save] of [
  // The human seat moved to the empire with the biggest army in contact (copies only; see learnings 2026-10-24):
  // the designers' own empires field 1 to 6 units, too few for a big battle.
  ['battle221', 'swap-veyr221'],
  ['battle234', 'swap-oath234'],
  ['battle172', 'swap-oath172'],
  ['battle185', 'swap-mongol185'],
] as const) live(label, async (page) => {
  const { raw, turn } = namedSave(save);
  await loadRaw(page, raw, turn, true, FX_ON);
  await cinema(page);
  const { orders, at } = await war(page);
  console.log(label, 'orders', orders.length, JSON.stringify(orders).slice(0, 1500));
  const cam = await frameAll(page, at);
  await liveTurn(page, label, turn, orders, -1, () => cam, [30, 60, 90, 120, 150, 180, 210, 240, 300]);
});

/** Frame timing on the real clock for `ms`: draw cost per frame (and the combat effects' share) and rAF gaps. */
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
    return { frames: p.frames, drawMean: r(p.drawMs / Math.max(1, p.frames)), drawMax: r(p.maxMs), fxFrames: p.fxFrames, fxMean: r(p.fxMs / Math.max(1, p.fxFrames)), fxMax: r(p.fxMaxMs), gapP50: q(0.5), gapP95: q(0.95), gapMax: q(1) };
  }, { ms });
}

/**
 * Frame cost on the real clock (no frame stepping): the turn-190 siege with combat animations on (then a stress replay
 * of its effects copied 20 times across the screen) or off. The box draws without a GPU, so absolute numbers are
 * pessimistic; compare on with off. Writes perf-<label>.json.
 */
for (const [label, prefs] of [['perf', FX_ON], ['perf-off', FX_OFF]] as const) test(label, async ({ page }) => {
  test.skip(!wanted(label));
  test.setTimeout(1_800_000);
  mkdirSync(OUT, { recursive: true });
  const results: Record<string, unknown> = {};
  await loadRaw(page, spineSave(190), 190, false, prefs);
  const w = await worldOf(page, LUGANO);
  await page.evaluate((c) => { Object.assign((window as unknown as W).legatumMap.camera, c); }, { x: w.x, y: w.y + 20, zoom: 1.4 });
  await page.waitForTimeout(1500);
  results.idle = await measure(page, 1500);
  await page.evaluate((orders) => { (window as unknown as W).legatum.getState().editOrders(() => orders); }, spineOrders(190));
  await endPlanning(page);
  const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
  await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
  await expect(page.getByTestId('turn-number')).toHaveText('191', { timeout: 120_000 });
  results.playback = await measure(page, 3500);
  if (label === 'perf') {
    const copies = await page.evaluate(() => {
      const s = (window as unknown as W).legatum.getState();
      const plan = s.anim?.plan;
      if (!plan) throw new Error('no combat plan after the turn');
      const shots: any[] = [], clashes: any[] = [], deaths: any[] = [];
      for (let i = 0; i < 20; i++) {
        const dx = ((i % 5) - 2) * 130, dy = (Math.floor(i / 5) - 1.5) * 110;
        const mv = (p: any) => ({ x: p.x + dx, y: p.y + dy });
        for (const sh of plan.shots) shots.push({ ...sh, from: mv(sh.from), to: mv(sh.to), seed: sh.seed + i * 1000, target: null, shooter: null });
        clashes.push({ attacker: null, defender: null, from: mv(plan.shots[0].from), to: mv(plan.shots[0].to), t: plan.volley + 200 + i * 20, lunge: true, mounted: i % 2 === 0, guns: true, atCity: false, hit: true, seed: i * 77 });
        for (const d of plan.deaths) deaths.push({ ...d, t: d.t + i * 15 });
      }
      (window as unknown as { legatum: { setState(p: unknown): void } }).legatum.setState({ anim: { ...s.anim, start: performance.now(), plan: { ...plan, shots, clashes, deaths, ghosts: new Map() } } });
      return { shots: shots.length, clashes: clashes.length, deaths: deaths.length };
    });
    results.stressCopies = copies;
    results.stress = await measure(page, 3500);
  }
  console.log(label, JSON.stringify(results));
  writeFileSync(join(OUT, `${label}.json`), JSON.stringify(results, null, 1));
});
