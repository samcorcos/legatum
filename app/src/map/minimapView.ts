// Minimap data and drawing. Everything comes from the viewer's PlayerView and is filtered again
// here, so the minimap can never show more than the viewer knows (pillar 6): unexplored tiles stay
// blank, remembered tiles are dimmed, and foreign units only show on tiles in sight right now.
import { capitalMark } from '../capitalMark';
import type { Hex, PlayerId, PlayerView, Terrain, UnitKind } from '../types';
import { DIRECTIONS, edgeCorners, fromOffset, hexAt, hexCorner, hexToPixel, neighbor, pixelToHex, tileIndex, toOffset } from '../hex';
import { HEX_SIZE, worldBounds } from './render';
import type { MapStyle } from './style';
import type { MapViewport } from './camera';

export interface MiniTile {
  hex: Hex;
  terrain: Terrain;
  /** Known but out of sight: drawn dimmed. */
  dim: boolean;
  owner: PlayerId | null;
}

export interface MiniCity {
  id: number;
  hex: Hex;
  owner: PlayerId;
  name: string;
  /** An original capital, kept for the whole game (not the current capital, which moves after a capture). */
  capital: boolean;
  /** Whose capital it was: the frame wears that empire's colour. */
  founder: PlayerId;
  /** You have scored its +1 VP yourself: the frame is dashed. */
  scored: boolean;
  mine: boolean;
}

export interface MiniUnit {
  hex: Hex;
  owner: PlayerId;
  kind: UnitKind;
  mine: boolean;
  /** Scouts and units on Explore: drawn so the player can find them at a glance. */
  scout: boolean;
  /** Byzantium's Heir: public wherever he stands, so he is always on the minimap, ringed in gilt. */
  heir: boolean;
}

/** Explored tiles only. Unexplored tiles are left out entirely. */
export function minimapTiles(view: PlayerView): MiniTile[] {
  const out: MiniTile[] = [];
  view.tiles.forEach((t, i) => {
    if (!t.explored || !t.terrain) return;
    out.push({ hex: hexAt(i, view.width), terrain: t.terrain, dim: !t.visible, owner: t.owner });
  });
  return out;
}

const tileOf = (view: PlayerView, h: Hex) => {
  const i = tileIndex(h, view.width, view.height);
  return i < 0 ? null : view.tiles[i];
};

/** Cities the viewer knows about (on explored tiles). */
export function minimapCities(view: PlayerView): MiniCity[] {
  return view.cities
    .filter((c) => tileOf(view, c.pos)?.explored)
    .map((c) => {
      const mark = capitalMark(view, c);
      return { id: c.id, hex: c.pos, owner: c.owner, name: c.name, capital: !!mark, founder: mark?.founder ?? c.owner, scored: mark?.scored === 'you', mine: c.owner === view.me };
    });
}

/** The viewer's own units, plus other empires' units standing on tiles in sight this turn, plus every Heir (his position is public). */
export function minimapUnits(view: PlayerView): MiniUnit[] {
  return view.units
    .filter((u) => u.owner === view.me || u.heir_of != null || !!tileOf(view, u.pos)?.visible)
    .map((u) => ({
      hex: u.pos,
      owner: u.owner,
      kind: u.kind,
      mine: u.owner === view.me,
      scout: u.owner === view.me && (u.kind === 'Scout' || u.automation === 'Explore'),
      heir: u.heir_of != null,
    }));
}

/** How the whole map fits the minimap box: world px -> minimap CSS px. */
export interface MinimapLayout {
  scale: number;
  minX: number;
  minY: number;
  /** Minimap size in CSS px (the map's aspect ratio, inside the box). */
  w: number;
  h: number;
  cols: number;
  rows: number;
}

/**
 * Fit the map into a box of at most `maxW` x `maxH` CSS px, keeping its aspect ratio. The world
 * rectangle is the tile area (half a hex of margin), not the camera's padded bounds.
 */
export function minimapLayout(view: Pick<PlayerView, 'width' | 'height'>, maxW: number, maxH: number): MinimapLayout {
  const b = worldBounds(view as PlayerView);
  const ww = b.maxX - b.minX, wh = b.maxY - b.minY;
  const scale = Math.min(maxW / ww, maxH / wh);
  return { scale, minX: b.minX, minY: b.minY, w: Math.round(ww * scale), h: Math.round(wh * scale), cols: view.width, rows: view.height };
}

export function worldToMini(l: MinimapLayout, x: number, y: number) {
  return { x: (x - l.minX) * l.scale, y: (y - l.minY) * l.scale };
}

export function miniToWorld(l: MinimapLayout, mx: number, my: number) {
  return { x: mx / l.scale + l.minX, y: my / l.scale + l.minY };
}

/** The map hex under a minimap point, clamped onto the map (a click on the margin picks the edge tile). */
export function miniToHex(l: MinimapLayout, mx: number, my: number): Hex {
  const w = miniToWorld(l, Math.max(0, Math.min(l.w, mx)), Math.max(0, Math.min(l.h, my)));
  const [c, r] = toOffset(pixelToHex(w.x, w.y, HEX_SIZE));
  return fromOffset(Math.max(0, Math.min(l.cols - 1, c)), Math.max(0, Math.min(l.rows - 1, r)));
}

/** The main map's visible area on the minimap, in minimap CSS px (may extend past its edges). */
export function viewportRect(l: MinimapLayout, v: MapViewport) {
  const a = worldToMini(l, v.x - v.w / 2 / v.zoom, v.y - v.h / 2 / v.zoom);
  const b = worldToMini(l, v.x + v.w / 2 / v.zoom, v.y + v.h / 2 / v.zoom);
  return { x: a.x, y: a.y, w: b.x - a.x, h: b.y - a.y };
}

// ---- drawing ----

function mix(a: string, b: string, k: number): string {
  const pa = parseInt(a.slice(1), 16), pb = parseInt(b.slice(1), 16);
  const ch = (p: number, s: number) => (p >> s) & 255;
  const c = (s: number) => Math.round(ch(pa, s) * (1 - k) + ch(pb, s) * k);
  return `#${((c(16) << 16) | (c(8) << 8) | c(0)).toString(16).padStart(6, '0')}`;
}

function hexPath(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number) {
  ctx.beginPath();
  for (let i = 0; i < 6; i++) {
    const [x, y] = hexCorner(cx, cy, r, i);
    if (i) ctx.lineTo(x, y); else ctx.moveTo(x, y);
  }
  ctx.closePath();
}

/**
 * Terrain, fog and territory: everything that only changes when the view does. Drawn once into
 * an offscreen canvas (device px) and blitted under the dots and viewport frame.
 */
export function drawMinimapBase(ctx: CanvasRenderingContext2D, view: PlayerView, l: MinimapLayout, style: MapStyle, dpr: number, color: (p: PlayerId) => string) {
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  // Unexplored: the look's blank-map color, a shade deeper so the minimap reads apart from its panel.
  ctx.fillStyle = mix(style.unexplored, '#6b5a3a', 0.12);
  ctx.fillRect(0, 0, l.w, l.h);
  const r = HEX_SIZE * l.scale;
  const tiles = minimapTiles(view);
  const dimTint = style.fog.tint;
  const fill = new Map<string, string>();
  const tint = (terrain: Terrain, dim: boolean) => {
    const key = `${terrain}${dim ? 'd' : ''}`;
    let c = fill.get(key);
    // Atlas's pastel palette is deepened a little so small hexes still read against the paper.
    const own = mix(style.pal[terrain], '#4a3920', 0.14);
    if (!c) fill.set(key, (c = dim ? mix(own, dimTint, 0.34) : own));
    return c;
  };
  // Slightly oversized hexes hide hairline seams between neighbours.
  for (const t of tiles) {
    const p = worldToMini(l, hexToPixel(t.hex, HEX_SIZE).x, hexToPixel(t.hex, HEX_SIZE).y);
    ctx.fillStyle = tint(t.terrain, t.dim);
    hexPath(ctx, p.x, p.y, r + 0.35);
    ctx.fill();
  }
  // Territory: a wash in the owner's color, then a solid edge where the owner changes.
  for (const t of tiles) {
    if (t.owner === null) continue;
    const p = worldToMini(l, hexToPixel(t.hex, HEX_SIZE).x, hexToPixel(t.hex, HEX_SIZE).y);
    ctx.globalAlpha = t.dim ? 0.3 : 0.42;
    ctx.fillStyle = color(t.owner);
    hexPath(ctx, p.x, p.y, r + 0.35);
    ctx.fill();
  }
  ctx.globalAlpha = 1;
  ctx.lineCap = 'round';
  ctx.lineWidth = Math.max(1.1, r * 0.42);
  for (const t of tiles) {
    if (t.owner === null) continue;
    const c = hexToPixel(t.hex, HEX_SIZE);
    const p = worldToMini(l, c.x, c.y);
    ctx.strokeStyle = color(t.owner);
    ctx.beginPath();
    for (let d = 0; d < DIRECTIONS.length; d++) {
      const n = tileOf(view, neighbor(t.hex, d));
      if (n?.explored && n.owner === t.owner) continue;
      const [a, b] = edgeCorners(d);
      const [ax, ay] = hexCorner(p.x, p.y, r * 0.86, a);
      const [bx, by] = hexCorner(p.x, p.y, r * 0.86, b);
      ctx.moveTo(ax, ay);
      ctx.lineTo(bx, by);
    }
    ctx.stroke();
  }
}

export interface MinimapInk {
  /** Outline under every dot and the frame, for contrast on any terrain. */
  dark: string;
  light: string;
}

/** Dots and the viewport frame over the cached base. Cheap: a blit plus a few dozen arcs. */
export function drawMinimapOverlay(
  ctx: CanvasRenderingContext2D, base: CanvasImageSource, view: PlayerView, l: MinimapLayout, dpr: number,
  color: (p: PlayerId) => string, ink: MinimapInk, viewport: MapViewport | null,
  /** Own cities under attack (red) or with an army closing (amber): ringed, so the alarm shows with the camera elsewhere. */
  alarms?: ReadonlyMap<number, 'attack' | 'approach'>,
  /** A teammate's live pings: a keylined ring with four ticks in the pinger's colour, so one made off screen is findable. */
  pings?: readonly { hex: Hex; color: string }[],
) {
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.drawImage(base, 0, 0);
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const at = (h: Hex) => { const p = hexToPixel(h, HEX_SIZE); return worldToMini(l, p.x, p.y); };
  const dot = (x: number, y: number, radius: number, fill: string, stroke: string, width: number) => {
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fillStyle = fill;
    ctx.fill();
    ctx.lineWidth = width;
    ctx.strokeStyle = stroke;
    ctx.stroke();
  };
  const units = minimapUnits(view);
  // Foreign units first, then own units, then cities on top, then your scouts above all.
  for (const u of units) if (!u.mine) { const p = at(u.hex); dot(p.x, p.y, 2.2, color(u.owner), ink.dark, 1); }
  for (const u of units) if (u.mine && !u.scout) { const p = at(u.hex); dot(p.x, p.y, 2.8, color(u.owner), ink.light, 1.3); }
  // The Heir: whoever holds him, wherever he is (fog included), a gilt ring round the holder's dot.
  for (const u of units) if (u.heir) { const p = at(u.hex); dot(p.x, p.y, 5.4, 'rgba(0,0,0,0)', ink.dark, 3.4); dot(p.x, p.y, 5.4, 'rgba(0,0,0,0)', '#c9a227', 1.8); dot(p.x, p.y, 2.6, color(u.owner), ink.dark, 1); }
  for (const c of minimapCities(view)) {
    const p = at(c.hex);
    const size = c.capital ? 4.6 : 3.6;
    // Cities are squares, so they never read as units; capitals carry a ring.
    ctx.fillStyle = color(c.owner);
    ctx.strokeStyle = ink.dark;
    ctx.lineWidth = 1.4;
    ctx.beginPath();
    ctx.rect(p.x - size, p.y - size, size * 2, size * 2);
    ctx.fill();
    ctx.stroke();
    const alarm = alarms?.get(c.id);
    if (alarm) {
      const ring = size + 4.5;
      ctx.beginPath();
      ctx.arc(p.x, p.y, ring, 0, Math.PI * 2);
      ctx.lineWidth = 4.5;
      ctx.strokeStyle = ink.light;
      ctx.stroke();
      ctx.lineWidth = 2.4;
      ctx.strokeStyle = alarm === 'attack' ? '#c1271b' : '#b8651d';
      ctx.setLineDash(alarm === 'attack' ? [] : [3, 2]);
      ctx.stroke();
      ctx.setLineDash([]);
    }
    if (c.capital) {
      // The frame wears the founder's colour over a dark rule; dashed once the viewer's team has scored it.
      const f = size + 2.2;
      ctx.setLineDash([]);
      ctx.strokeStyle = ink.dark;
      ctx.lineWidth = c.scored ? 4 : 3.2;
      ctx.strokeRect(p.x - f, p.y - f, f * 2, f * 2);
      // Scored: heavy dashes over the dark rule, so the broken line reads at 1x.
      if (c.scored) ctx.setLineDash([3.4, 2.4]);
      ctx.strokeStyle = color(c.founder);
      ctx.lineWidth = c.scored ? 2.4 : 1.6;
      ctx.strokeRect(p.x - f, p.y - f, f * 2, f * 2);
      ctx.setLineDash([]);
    }
  }
  for (const u of units) {
    if (!u.scout) continue;
    const p = at(u.hex);
    // A diamond in white with the owner's rim: the one shape to look for.
    ctx.beginPath();
    ctx.moveTo(p.x, p.y - 4.4); ctx.lineTo(p.x + 4.4, p.y); ctx.lineTo(p.x, p.y + 4.4); ctx.lineTo(p.x - 4.4, p.y);
    ctx.closePath();
    ctx.fillStyle = ink.light;
    ctx.fill();
    ctx.lineWidth = 1.8;
    ctx.strokeStyle = color(u.owner);
    ctx.stroke();
    ctx.lineWidth = 0.8;
    ctx.strokeStyle = ink.dark;
    ctx.stroke();
  }
  if (viewport) {
    const v = viewportRect(l, viewport);
    const x = Math.max(1, v.x), y = Math.max(1, v.y);
    const w = Math.min(l.w - 1, v.x + v.w) - x, h = Math.min(l.h - 1, v.y + v.h) - y;
    if (w > 0 && h > 0) {
      ctx.lineWidth = 3;
      ctx.strokeStyle = ink.dark;
      ctx.strokeRect(x, y, w, h);
      ctx.lineWidth = 1.5;
      ctx.strokeStyle = ink.light;
      ctx.strokeRect(x, y, w, h);
    }
  }
  for (const pg of pings ?? []) {
    const p = at(pg.hex);
    const ring = (stroke: string, width: number) => {
      ctx.beginPath();
      ctx.arc(p.x, p.y, 5, 0, Math.PI * 2);
      for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) { ctx.moveTo(p.x + dx * 3.2, p.y + dy * 3.2); ctx.lineTo(p.x + dx * 8, p.y + dy * 8); }
      ctx.lineWidth = width;
      ctx.strokeStyle = stroke;
      ctx.stroke();
    };
    ctx.setLineDash([]);
    ring(ink.dark, 4.4);
    ring(ink.light, 3);
    ring(pg.color, 1.6);
  }
}

/** The known city nearest a minimap point, within `radius` CSS px. */
export function cityNear(view: PlayerView, l: MinimapLayout, mx: number, my: number, radius = 7): MiniCity | null {
  let best: MiniCity | null = null, bestD = radius * radius;
  for (const c of minimapCities(view)) {
    const p = hexToPixel(c.hex, HEX_SIZE);
    const m = worldToMini(l, p.x, p.y);
    const d = (m.x - mx) ** 2 + (m.y - my) ** 2;
    if (d <= bestD) { best = c; bestD = d; }
  }
  return best;
}
