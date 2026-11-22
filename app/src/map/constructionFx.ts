// Persistent construction sites, drawn only from painted art: the stage sprites and the finished landmark.
// Nothing here is a canvas line drawing of an object (no crane, workers, scaffold or string line): legatum-ui, 2026-11-01.
import type { Faction, PlayerView } from '../types';
import { buildingIconName } from '../data';
import { iconLoaded, tintedIcon } from '../icons';
import { hexToPixel } from '../hex';
import { wallStem, CITY_CANVAS, CITY_GROUND } from './cityArt';
import { CITY_DROP, CITY_WORLD } from './terrain';
import { HEX_SIZE, isFar } from './grid';
import { landmarkBox, landmarkImage, type LandmarkBox } from './landmarks';
import { ATLAS } from './style';
import { constructionStage, sitesOf, type ConstructionSite } from './constructionSites';

const urls = import.meta.glob<string>('../assets/construction/*.png', { eager: true, query: '?url', import: 'default' });
const byKey = new Map(Object.entries(urls).map(([path, url]) => [path.replace(/^.*\/construction\//, '').replace(/\.png$/, ''), url]));
const images = new Map<string, HTMLImageElement>();
export function stageImage(stem: string, n: number, px: number): HTMLImageElement | null {
  // A missing size falls back to the next larger file before a smaller one (Walls stages have no -256), so nothing is upscaled.
  const sizes = px > 256 ? ['', '-256', '-128', '-64'] : px > 128 ? ['-256', '', '-128', '-64'] : ['-128', '-256', '', '-64'];
  for (const suffix of sizes) {
    const url = byKey.get(`${stem}-stage${n}${suffix}`);
    if (!url || typeof Image === 'undefined') continue;
    let img = images.get(url);
    // A stage that arrives late redraws the map, so a site never waits for the next unrelated repaint.
    if (!img) { img = new Image(); img.onload = () => iconLoaded(); img.src = url; images.set(url, img); }
    if (img.complete && img.naturalWidth) return img;
  }
  return null;
}

type Box = { x: number; y: number; s: number };
type Pt = { x: number; y: number };
const ink = ATLAS.tone.ink, earth = ATLAS.tone.vellum;
const clamp = (k: number) => Math.max(0, Math.min(1, k));

/** The painted stage for the production reached (stages 1 to 3), then the finished landmark. Under 10% nothing stands yet. */
export function drawSite(ctx: CanvasRenderingContext2D, site: ConstructionSite, box: Box, k = site.k, faction?: Faction) {
  const stage = constructionStage(clamp(k));
  const sprite = stage >= 1 && stage <= 3 ? stageImage(`building-${site.building.toLowerCase()}`, stage, box.s) : null;
  if (sprite) ctx.drawImage(sprite, box.x, box.y, box.s, box.s);
  if (stage === 4) { const finished = landmarkImage(site.building, faction); if (finished) ctx.drawImage(finished, box.x, box.y, box.s, box.s); }
}

/**
 * The era's painted Walls stages (`assets/construction/<era>-walls-stage1..3`: footings, half-height courses, nearly finished),
 * registered to the finished overlay's canvas, so the ring never jumps when the Walls complete. Under 10% nothing stands yet.
 */
export function wallsSiteStem(site: ConstructionSite, faction?: Faction): string | null {
  return wallStem(site.era, faction === 'Switzerland');
}

export function drawWallsSite(ctx: CanvasRenderingContext2D, site: ConstructionSite, center: Pt, zoom: number, dpr: number, k = site.k, faction?: Faction) {
  const stage = constructionStage(clamp(k));
  if (stage < 1 || stage > 3) return;
  const s = CITY_WORLD * zoom, scale = s / CITY_CANVAS;
  const stem = wallsSiteStem(site, faction);
  const img = stem ? stageImage(stem, stage, s * dpr) : null;
  if (!img) return;
  ctx.drawImage(img, center.x - (CITY_GROUND.x0 + CITY_GROUND.x1) / 2 * scale, center.y - CITY_GROUND.middle * scale + CITY_DROP * zoom, s, s);
}

export function drawConstructionSites(ctx: CanvasRenderingContext2D, view: PlayerView, toScreen: (p: Pt) => Pt, hexPx: number, w: number, h: number, dpr: number, overrides?: ReadonlyMap<number, number>, boxes: ReadonlyMap<number, LandmarkBox> = new Map(), extras: readonly ConstructionSite[] = [], hidden: ReadonlySet<number> = new Set()) {
  const far = isFar(hexPx);
  ctx.save(); ctx.setTransform(dpr, 0, 0, dpr, 0, 0); ctx.imageSmoothingEnabled = true;
  for (const site of [...sitesOf(view), ...extras]) {
    if (hidden.has(site.city)) continue;
    const p = toScreen(hexToPixel(site.hex, HEX_SIZE));
    if (p.x < -hexPx * 3 || p.x > w + hexPx * 3 || p.y < -hexPx * 3 || p.y > h + hexPx * 3) continue;
    const k = overrides?.get(site.city) ?? site.k;
    ctx.globalAlpha = view.tiles[site.tile]?.visible ? 1 : .75;
    if (far && !site.walls) {
      const r = Math.max(7, hexPx * .5);
      ctx.fillStyle = earth; ctx.strokeStyle = ATLAS.tone.gold; ctx.lineWidth = 1.5; ctx.setLineDash([3, 2]);
      ctx.beginPath(); ctx.arc(p.x, p.y, r, 0, Math.PI * 2); ctx.fill(); ctx.stroke(); ctx.setLineDash([]);
      const name = buildingIconName(site.building);
      const icon = tintedIcon(name, ink, Math.ceil(r * 1.3 * dpr));
      if (icon) ctx.drawImage(icon, p.x - r * .65, p.y - r * .65, r * 1.3, r * 1.3);
    } else if (site.walls) drawWallsSite(ctx, site, p, hexPx / HEX_SIZE, dpr, k, view.players.find(pl => pl.id === site.owner)?.faction);
    else drawSite(ctx, site, boxes.get(site.tile) ?? landmarkBox(p, hexPx), k, view.players.find(pl => pl.id === site.owner)?.faction);
  }
  ctx.restore();
}
