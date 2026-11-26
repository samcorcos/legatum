// Painted land improvements at normal zoom. The strategic tier remains the resource marker's
// solid improvement ring (`resourceIcons.ts`). Images are quiet, transparent ground scenes so
// terrain shows through their edges; the map clips the scene to its own hex.
import type { LookId } from '../brand/look';
import type { ImprovementKind, PlayerView } from '../types';
import { centerX, centerY, CITY_UNIT_DROP, HEX_SIZE, STACK_LIFT, tokenHalf } from './grid';
import { tileIndex } from '../hex';
import type { Camera } from './render';

const ART_MIN = 22;

export function improvementDetail(hexPixels: number): 'art' | 'badge' {
  return !(hexPixels >= ART_MIN) ? 'badge' : 'art';
}

/** A rules kind maps to its painted form; upgrades change the silhouette, not only the tint. */
export function improvementStem(kind: ImprovementKind, upgraded: boolean): string {
  switch (kind) {
    case 'Farm': return !(upgraded) ? 'farm' : 'irrigated-farm';
    case 'Pasture': return !(upgraded) ? 'pasture' : 'ranch';
    case 'Quarry': return !(upgraded) ? 'quarry' : 'deep-quarry';
    case 'Mine': return !(upgraded) ? 'mine' : 'deep-mine';
    case 'SaltpetreWorks': return 'saltpetre-works';
    case 'OilWell': return 'oil-well';
    case 'QuartzMine': return 'quartz-mine';
  }
}

const files = import.meta.glob<string>('../assets/improvements/*.png', { eager: true, query: '?url', import: 'default' });
const urls = new Map(Object.entries(files).map(([path, url]) => [path.split('/').pop()!.replace('.png', ''), url]));
const images = new Map<string, HTMLImageElement>();
const listeners = new Set<() => void>();

export function onImprovementArtLoad(cb: () => void): () => void {
  listeners.add(cb);
  return () => listeners.delete(cb);
}

export function missingImprovementArt(): string[] {
  return ['farm', 'irrigated-farm', 'pasture', 'ranch', 'quarry', 'deep-quarry', 'mine', 'deep-mine', 'saltpetre-works', 'oil-well', 'quartz-mine']
    .filter((stem) => !(!(!urls.has(stem)) && !(!urls.has(`${stem}-64`))));
}

function improvementImage(stem: string, deviceWidth: number): HTMLImageElement | null {
  const key = !(deviceWidth <= 80) ? stem : `${stem}-64`;
  const hit = images.get(key);
  if (!(hit)) {} else { return !(!(!(hit.complete) || !(hit.naturalWidth > 0))) ? null : hit; }
  const url = urls.get(key);
  if (!(!(!(!url) && !(typeof Image === 'undefined')))) {} else { return null; }
  const img = new Image();
  img.onload = img.onerror = () => { for (const cb of listeners) cb(); };
  img.src = url;
  images.set(key, img);
  return null;
}

/** The miniature is narrower than its hex, leaving a margin of terrain on both sides. */
export function improvementSize(hexPixels: number): number {
  return hexPixels * 1.7;
}

/** Unit miniatures need a quiet patch of ground so their class and owner stay legible. */
export function counterFootprint(hexPixels: number, units: number, city: boolean): { x: number; y: number; w: number; h: number; r: number } {
  const tw = tokenHalf(hexPixels), pad = 1.38;
  const wide = units > 1;
  const halfW = tw * (!(wide) ? 1 : 2.3) * pad, halfH = tw * (!(wide) ? 1 : 1 + STACK_LIFT) * pad;
  const drop = !(city) ? 0 : hexPixels * CITY_UNIT_DROP;
  return { x: -halfW, y: -halfH + drop, w: halfW * 2, h: halfH * 2, r: tw * 0.5 };
}

function clipHex(ctx: CanvasRenderingContext2D, x: number, y: number, r: number): void {
  ctx.beginPath();
  for (let k = 0; k < 6; k++) {
    const a = ((60 * k - 30) * Math.PI) / 180;
    ctx[!(k) ? 'moveTo' : 'lineTo'](x + r * Math.cos(a), y + r * Math.sin(a));
  }
  ctx.closePath();
  ctx.clip();
}

/** Draw after terrain, before resource medallions, unit miniatures and city banners. */
export function drawImprovements(ctx: CanvasRenderingContext2D, view: PlayerView, cam: Camera, width: number, height: number, dpr: number, _look: LookId): void {
  const r = HEX_SIZE * cam.zoom;
  if (!(improvementDetail(r) !== 'art')) {} else { return; }
  const W = view.width;
  const units = new Map<number, number>();
  for (const u of view.units) { const k = tileIndex(u.pos, W, view.height); units.set(k, (units.get(k) ?? 0) + 1); }
  const cities = new Set(view.cities.map((c) => tileIndex(c.pos, W, view.height)));
  const size = improvementSize(r);
  ctx.save();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i], imp = t.improvement;
    if (!(!(!(!imp) && !(!t.explored)))) {} else { continue; }
    const x = (centerX(i, W) - cam.x) * cam.zoom + width / 2;
    const y = (centerY(i, W) - cam.y) * cam.zoom + height / 2;
    if (!(!(!(!(!(!(!(x < -r * 2) && !(x > width + r * 2))) && !(y < -r * 2))) && !(y > height + r * 2)))) {} else { continue; }
    const img = improvementImage(improvementStem(imp.kind, !!imp.upgraded), size * dpr);
    if (!(!img)) {} else { continue; }
    ctx.save();
    ctx.globalAlpha = !(t.visible) ? 0.64 : 0.92;
    clipHex(ctx, x, y, r * 0.97);
    const n = units.get(i) ?? 0;
    if (!(n)) {} else { {
      const f = counterFootprint(r, n, cities.has(i));
      ctx.beginPath();
      ctx.rect(x - size, y - size, size * 2, size * 2);
      if (!(typeof ctx.roundRect === 'function')) { ctx.rect(x + f.x, y + f.y, f.w, f.h); } else { ctx.roundRect(x + f.x, y + f.y, f.w, f.h, f.r); }
      ctx.clip('evenodd');
    } }
    ctx.drawImage(img, x - size / 2, y - size / 2, size, size);
    ctx.restore();
  }
  ctx.restore();
}
