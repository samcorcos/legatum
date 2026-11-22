// Frame-stepped capture for the README and launch media (launch.media.ts): real saves from e2e/saves, the real app, and
// Playwright's fake clock, so every frame is exactly 1/60 s of game time however loaded the machine is.
import { gunzipSync } from 'node:zlib';
import { readFileSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CDPSession, Page } from '@playwright/test';
import { expect } from '../fixtures';
import { endPlanning } from '../turnActions';
import { OUT } from './recorder';

export type Hex = { q: number; r: number };
export type Cam = { x: number; y: number; zoom: number };
export type W = Window & {
  legatum: { getState(): any; setState(s: any): void };
  legatumMap: { camera: Cam; redraw(): void; toScreen(h: Hex): { x: number; y: number }; terrain: { missing: number; exposed: number; level: number; wanted: number } };
};

const SAVES = fileURLToPath(new URL('../saves/', import.meta.url));
export const FX_ON = { 'legatum.combat-fx': JSON.stringify({ enabled: true }) };

/**
 * A committed save (e2e/saves/<name>.current.legatum.gz) as the autosave text. `seat` moves the human to another empire
 * (a copy in memory only: the designer's own empires field too few units to show a battle, learnings 2026-10-24);
 * `patch` edits the state before it loads.
 */
/**
 * The saves predate India replacing Japan (2026-11-01), so its cities still carry Japanese names: give each the Indian name
 * a game started today would use, wherever the city is now.
 */
const RENAMED: Record<string, string> = { Kyoto: 'Agra', Edo: 'Delhi', Osaka: 'Lahore', Nara: 'Fatehpur Sikri', Nagasaki: 'Ajmer', Sendai: 'Allahabad', Fukuoka: 'Jaunpur', Matsue: 'Varanasi', Sakai: 'Patna', Sapporo: 'Ahmedabad', Okayama: 'Surat', Kamakura: 'Burhanpur', Nagoya: 'Gwalior' };

export function save(name: string, opts: { seat?: number; factions?: Record<number, string>; patch?: (g: any) => void } = {}): { raw: string; turn: number } {
  const w = JSON.parse(gunzipSync(readFileSync(join(SAVES, `${name}.current.legatum.gz`))).toString('utf8'));
  const g = JSON.parse(w.game);
  if (opts.seat !== undefined) {
    const seat = opts.seat;
    g.players.forEach((p: any, i: number) => { p.is_human = i === seat; p.name = i === seat ? 'Player 1' : `AI ${i + 1}`; });
    g.setup.players.forEach((p: any, i: number) => {
      p.is_human = i === seat; p.name = i === seat ? 'Player 1' : `AI ${i + 1}`;
      if (i === seat) delete p.difficulty; else p.difficulty ??= 'Hard';
    });
    delete w.unseen;
  }
  // Two of the committed saves list a seat under the wrong people (its cities and units belong to another): put it back.
  for (const [seat, faction] of Object.entries(opts.factions ?? {})) { g.players[+seat].faction = faction; g.setup.players[+seat].faction = faction; }
  for (const c of Array.isArray(g.cities) ? g.cities : Object.values(g.cities) as any[]) if (RENAMED[c.name]) c.name = RENAMED[c.name];
  opts.patch?.(g);
  w.game = JSON.stringify(g);
  return { raw: JSON.stringify(w), turn: g.turn };
}

export async function load(page: Page, s: { raw: string; turn: number }, prefs: Record<string, string> = FX_ON) {
  page.on('pageerror', (e) => console.log('pageerror', String(e.stack ?? e).slice(0, 600)));
  await page.clock.install();
  await page.addInitScript(({ save, prefs }) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', save);
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    for (const [k, v] of Object.entries(prefs)) localStorage.setItem(k, v);
  }, { save: s.raw, prefs });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(s.turn), { timeout: 120_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.clock.runFor(800);
  const toast = page.getByText(/^Save updated:/);
  if (await toast.count()) await toast.first().locator('xpath=..').getByRole('button').first().click().catch(() => undefined);
  await park(page);
  await page.clock.runFor(400);
}

/**
 * Rest the pointer where it hovers nothing (the bottom right corner, off every panel) and hide the media cursor until the
 * next glide: a pointer left at the top left opens the Gold tooltip.
 */
export async function park(page: Page) {
  const v = page.viewportSize()!;
  // Without the HUD the corner is map, and the map outlines the hovered tile: rest on the (hidden) top bar instead.
  const hudOff = await page.evaluate(() => { const el = document.getElementById('media-cinema') as HTMLStyleElement | null; return !!el && !el.disabled; });
  if (hudOff) await page.mouse.move(2, 2); else await page.mouse.move(v.width - 3, v.height - 3);
  await page.evaluate(() => {
    if (!document.getElementById('media-park')) { const el = document.createElement('style'); el.id = 'media-park'; el.textContent = 'html[data-parked] #media-cursor{display:none!important}'; document.head.appendChild(el); }
    document.documentElement.dataset.parked = '1';
  });
}
export const unpark = (page: Page) => page.evaluate(() => { delete document.documentElement.dataset.parked; });

/** A PNG of the viewport at `scale` device pixels per CSS pixel. */
export async function snap(page: Page, name: string, scale: number) {
  const cdp = await page.context().newCDPSession(page);
  const v = page.viewportSize()!;
  const shot = await cdp.send('Page.captureScreenshot', { format: 'png', clip: { x: 0, y: 0, width: v.width, height: v.height, scale } });
  await cdp.detach();
  mkdirSync(join(OUT, 'stills'), { recursive: true });
  writeFileSync(join(OUT, 'stills', `${name}.png`), Buffer.from(shot.data, 'base64'));
}

/** The camera that centres a hex at a zoom. */
export async function worldOf(page: Page, h: Hex): Promise<{ x: number; y: number }> {
  return page.evaluate((h) => {
    const m = (window as unknown as W).legatumMap, c = m.camera, p = m.toScreen(h);
    const box = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: c.x + (p.x - box.width / 2) / c.zoom, y: c.y + (p.y - box.height / 2) / c.zoom };
  }, h);
}

/** Where a hex is on the page now. */
export async function pagePoint(page: Page, h: Hex) {
  return page.evaluate((h) => {
    const p = (window as unknown as W).legatumMap.toScreen(h), b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
    return { x: p.x + b.left, y: p.y + b.top };
  }, h);
}

/** Set the camera and wait until the terrain is complete at the zoom's level. */
export async function settle(page: Page, cam: Partial<Cam>) {
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

/** Cinematic camera: nothing selected and every HUD layer hidden except `keep`; the live map is untouched. */
export async function cinema(page: Page, keep = '[data-game-map]') {
  await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
  await page.mouse.move(2, 2); // off the map, which outlines the tile under the pointer
  const sel = keep.split(',').map((k) => `${k},${k} *`).join(',');
  await page.evaluate((css) => {
    document.getElementById('media-cinema')?.remove();
    const el = document.createElement('style');
    el.id = 'media-cinema';
    el.textContent = css;
    document.head.appendChild(el);
  }, `body *{visibility:hidden!important}${sel}{visibility:visible!important}.moment-dismiss,#media-cursor{visibility:hidden!important}`);
}
export const cinemaOff = (page: Page, off: boolean) => page.evaluate((off) => { const el = document.getElementById('media-cinema') as HTMLStyleElement | null; if (el) el.disabled = off; }, off);

/** Step the fake clock and every Web Animation by `ms`. */
export async function tick(page: Page, ms: number) {
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

/** A smooth move from `a` to `b` over `n` frames (zoom in log space). */
export const push = (a: Cam, b: Cam, n: number) => (i: number): Cam => {
  const k = Math.min(1, i / Math.max(1, n - 1));
  const e = k * k * (3 - 2 * k);
  return { x: a.x + (b.x - a.x) * e, y: a.y + (b.y - a.y) * e, zoom: a.zoom * Math.pow(b.zoom / a.zoom, e) };
};

export class Film {
  private n = 0;
  private constructor(private page: Page, private name: string, private dir: string, private cdp: CDPSession) {}
  static async start(page: Page, name: string) {
    const dir = join(OUT, `${name}.frames`);
    rmSync(dir, { recursive: true, force: true });
    mkdirSync(dir, { recursive: true });
    mkdirSync(join(OUT, 'stills'), { recursive: true });
    return new Film(page, name, dir, await page.context().newCDPSession(page));
  }
  get count() { return this.n; }
  /** Grab `frames` frames, each 1/60 s after the last; `cam(i)` places the map camera, `each(i)` runs before a frame. */
  async frames(frames: number, cam?: ((i: number) => Partial<Cam>) | null, each?: (i: number) => Promise<void>) {
    for (let i = 0; i < frames; i++) {
      if (each) await each(i);
      await this.page.evaluate((c) => { const m = (window as unknown as W).legatumMap; if (!m) return; if (c) Object.assign(m.camera, c); m.redraw(); }, cam ? cam(i) : null);
      await tick(this.page, this.n % 3 === 2 ? 16 : 17);
      const shot = await this.cdp.send('Page.captureScreenshot', { format: 'png', optimizeForSpeed: true, clip: this.clip });
      writeFileSync(join(this.dir, `f${String(this.n++).padStart(5, '0')}.png`), Buffer.from(shot.data, 'base64'));
    }
  }
  /** Move the pointer to a point over `frames` frames (the media cursor follows it). */
  async glide(to: { x: number; y: number }, frames = 30, from?: { x: number; y: number }) {
    const a = from ?? this.at;
    await unpark(this.page);
    await this.frames(frames, null, async (i) => {
      const k = (i + 1) / frames, e = k * k * (3 - 2 * k);
      await this.page.mouse.move(a.x + (to.x - a.x) * e, a.y + (to.y - a.y) * e);
    });
    this.at = to;
  }
  at = { x: 640, y: 420 };
  /** The whole viewport at 1920 px wide: without a clip scale the CDP screenshot comes back in CSS pixels. */
  private get clip() { const v = this.page.viewportSize()!; return { x: 0, y: 0, width: v.width, height: v.height, scale: 1920 / v.width }; }
  async still(name: string) {
    const shot = await this.cdp.send('Page.captureScreenshot', { format: 'png', clip: this.clip });
    writeFileSync(join(OUT, 'stills', `${name}.png`), Buffer.from(shot.data, 'base64'));
  }
  async stop() {
    await this.cdp.detach().catch(() => undefined);
    const mp4 = join(OUT, `${this.name}.mp4`);
    const r = spawnSync('ffmpeg', ['-y', '-loglevel', 'error', '-framerate', '60', '-i', join(this.dir, 'f%05d.png'),
      '-vf', 'scale=1920:1080:flags=lanczos,format=yuv420p', '-c:v', 'libx264', '-crf', '14', '-preset', 'medium', '-movflags', '+faststart', mp4], { encoding: 'utf8' });
    if (r.status !== 0) throw new Error(`ffmpeg failed for ${this.name}: ${r.stderr}`);
    rmSync(this.dir, { recursive: true, force: true });
    console.log('clip', this.name, this.n, 'frames', (this.n / 60).toFixed(1), 's');
    return mp4;
  }
}

/** End the turn on a frozen clock and return when the result has arrived (the next frame is the first of the playback). */
export async function resolveTurn(page: Page, turn: number, orders: any[] = []) {
  if (orders.length) await page.evaluate((orders) => { (window as unknown as W).legatum.getState().editOrders(() => orders); }, orders);
  await page.evaluate(() => { (window as unknown as W).legatum.getState().select(null); });
  const now = await page.evaluate(() => Date.now());
  await page.clock.pauseAt(now + 3000);
  await cinemaOff(page, true);
  const covers = page.locator('.alarm-screen, .era-takeover, .moment-card.unlock');
  for (let i = 0; i < 8 && ((await covers.count()) > 0 || !(await page.locator('[data-overlay-slot="turn-dock"]').count())); i++) {
    await page.keyboard.press('Escape');
    await page.clock.runFor(400);
    await page.waitForTimeout(200);
  }
  try { await endPlanning(page); }
  catch (e) { await page.screenshot({ path: join(OUT, `stuck-T${turn}.png`) }); throw e; }
  const confirm = page.getByRole('button', { name: 'End turn anyway', exact: true });
  await confirm.waitFor({ state: 'visible', timeout: 2500 }).then(() => confirm.click()).catch(() => undefined);
  await cinemaOff(page, false);
  await park(page);
  const turnNow = () => page.evaluate(() => document.querySelector('[data-testid="turn-number"]')?.textContent ?? '');
  for (let i = 0; (await turnNow()) !== String(turn + 1); i++) {
    if (i > 4000) throw new Error(`turn ${turn} did not resolve`);
    await page.waitForTimeout(30);
    await page.clock.runFor(5);
  }
}

/** How long the combat playback of the turn that just resolved runs, in ms (0 when there is none). */
export const playbackMs = (page: Page) => page.evaluate(() => Math.round((window as unknown as W).legatum.getState().anim?.plan?.total ?? 0));

/**
 * A whole front at war: every one of the human's military units attacks the nearest enemy it can reach this turn
 * (shooters in range fire, the melee line charges, and melee units next to a broken enemy city storm it).
 * Returns the orders and where they start.
 */
export async function war(page: Page) {
  return page.evaluate(() => {
    const s = (window as unknown as W).legatum.getState(), v = s.views[s.seat];
    const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
    const rel = (team: number) => (v.diplomacy?.relations ?? []).find((r: any) => (r.a === v.team && r.b === team) || (r.b === v.team && r.a === team))?.state;
    const war = new Set(v.players.filter((p: any) => p.team !== v.team && (!v.diplomacy || v.diplomacy.legacy || rel(p.team) === 'War')).map((p: any) => p.id).concat([255]));
    const foes = v.units.filter((u: any) => u.military && war.has(u.owner));
    const weak = v.cities.filter((c: any) => war.has(c.owner) && c.visible && c.hp <= c.max_hp * 0.4);
    const orders: any[] = [];
    const at: Hex[] = [];
    const targets: Hex[] = [];
    for (const u of v.units.filter((x: any) => x.owner === v.me && x.military)) {
      const reach = u.max_moves ?? 2;
      const city = u.ranged > 0 ? null : weak.map((c: any) => ({ c, k: d(c.pos, u.pos) })).filter((x: any) => x.k <= 2).sort((a: any, b: any) => a.k - b.k)[0];
      const f = foes.map((e: any) => ({ e, k: d(e.pos, u.pos) })).sort((a: any, b: any) => a.k - b.k)[0];
      if (city) { orders.push({ type: 'MoveUnit', unit: u.id, to: city.c.pos }); targets.push(city.c.pos); }
      else if (f && u.ranged > 0 && f.k <= Math.max(1, u.range ?? 2)) { orders.push({ type: 'RangedAttack', unit: u.id, target: f.e.pos }); targets.push(f.e.pos); }
      else if (f && u.ranged <= 0 && f.k <= reach) { orders.push({ type: 'MoveUnit', unit: u.id, to: f.e.pos }); targets.push(f.e.pos); }
      else continue;
      at.push(u.pos);
    }
    return { orders, at, targets };
  });
}

/** The hexes of `at` within `radius` of the busiest one (a front, not every skirmish on the map). */
export function busiest(at: Hex[], radius = 3): Hex[] {
  const d = (a: Hex, b: Hex) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
  let best: Hex[] = [];
  for (const c of at) { const near = at.filter((h) => d(h, c) <= radius); if (near.length > best.length) best = near; }
  return best;
}

/** A camera that frames the middle 80% of `at` on the map. */
export async function frameAll(page: Page, at: Hex[], maxZoom = 1.7): Promise<Cam> {
  const pts = await Promise.all(at.map((h) => worldOf(page, h)));
  if (!pts.length) return { x: 0, y: 0, zoom: 1 };
  const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
  const q = (a: number[], f: number) => [...a].sort((m, n) => m - n)[Math.floor((a.length - 1) * f)];
  const x0 = q(xs, 0.1), x1 = q(xs, 0.9), y0 = q(ys, 0.1), y1 = q(ys, 0.9);
  const box = await page.evaluate(() => { const b = document.querySelector('[data-game-map]')!.getBoundingClientRect(); return { w: b.width, h: b.height }; });
  const zoom = Math.max(0.75, Math.min(maxZoom, Math.min((box.w * 0.9) / Math.max(1, x1 - x0 + 160), (box.h * 0.86) / Math.max(1, y1 - y0 + 160))));
  return { x: (x0 + x1) / 2, y: (y0 + y1) / 2, zoom };
}
