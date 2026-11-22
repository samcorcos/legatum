// Shared scene for the three art-direction mockups: a deterministic 48x32 hex world,
// the game content shown in every direction, the icon sprite, and a painterly map renderer
// parameterized by a per-direction style. Output is static SVG/HTML (no runtime JS).
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
export const ICON_DIR = path.resolve(here, '../../../../app/src/assets/icons');

export const VW = 1440;
export const VH = 900;
export const W = 48;
export const H = 32;
export const R = 27; // hex radius in px (pointy-top)
const S3 = Math.sqrt(3);
const HW = R * S3; // hex width
const OFF = { x: 700 - HW * 22.5, y: 470 - R * 1.5 * 15 }; // Cantor (22,15) lands at (700,470)

// ---------- geometry (odd-r offset, pointy-top; mirrors app/src/hex.ts) ----------
export const center = (c, r) => ({ x: HW * (c + 0.5 * (r & 1)) + OFF.x, y: R * 1.5 * r + OFF.y });
// Edge i runs from corner i to corner i+1; corner i sits at angle 60*i - 30 degrees.
// Neighbor across edge i: E, SE, SW, W, NW, NE.
const DIRS = [
  [[1, 0], [0, 1], [-1, 1], [-1, 0], [-1, -1], [0, -1]],
  [[1, 0], [1, 1], [0, 1], [-1, 0], [0, -1], [1, -1]],
];
export const nb = (c, r, i) => {
  const d = DIRS[r & 1][i];
  return [c + d[0], r + d[1]];
};
export function corners(x, y, rad = R) {
  const out = [];
  for (let i = 0; i < 6; i++) {
    const a = ((60 * i - 30) * Math.PI) / 180;
    out.push([x + rad * Math.cos(a), y + rad * Math.sin(a)]);
  }
  return out;
}
export const f1 = (n) => Math.round(n * 10) / 10;
export const poly = (x, y, rad = R) => corners(x, y, rad).map(([a, b]) => `${f1(a)},${f1(b)}`).join(' ');
export const hexPath = (x, y, rad = R) =>
  'M' + corners(x, y, rad).map(([a, b]) => `${f1(a)} ${f1(b)}`).join('L') + 'Z';
const cube = (c, r) => {
  const q = c - (r - (r & 1)) / 2;
  return [q, r, -q - r];
};
export function dist(a, b) {
  const A = cube(a[0], a[1]);
  const B = cube(b[0], b[1]);
  return Math.max(Math.abs(A[0] - B[0]), Math.abs(A[1] - B[1]), Math.abs(A[2] - B[2]));
}
const inb = (c, r) => c >= 0 && r >= 0 && c < W && r < H;

// ---------- deterministic noise ----------
function hash(i, j, s) {
  let h = Math.imul(i, 374761393) + Math.imul(j, 668265263) + Math.imul(s, 982451653);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}
export const rnd = (a, b, s = 0) => hash(a * 7 + 13, b * 11 + 5, s + 101);
function vnoise(x, y, s) {
  const xi = Math.floor(x), yi = Math.floor(y);
  const xf = x - xi, yf = y - yi;
  const u = xf * xf * (3 - 2 * xf), v = yf * yf * (3 - 2 * yf);
  const a = hash(xi, yi, s), b = hash(xi + 1, yi, s), c = hash(xi, yi + 1, s), d = hash(xi + 1, yi + 1, s);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
function fbm(x, y, s, oct = 4) {
  let t = 0, amp = 0.5, f = 1, norm = 0;
  for (let o = 0; o < oct; o++) {
    t += amp * vnoise(x * f, y * f, s + o * 17);
    norm += amp;
    amp *= 0.5;
    f *= 2.03;
  }
  return t / norm;
}

// ---------- world ----------
// A designed world: western ocean with a deep bay reaching toward Cantor, a south-eastern inland sea,
// a mountain spine between our lands and Hollow Crown, tundra to the north. px, py in hex units
// (px = column + half-row offset, py = row).
const segDist = (x, y, ax, ay, bx, by) => {
  const dx = bx - ax, dy = by - ay;
  const t = Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)));
  const ex = ax + t * dx - x, ey = ay + t * dy - y;
  return Math.sqrt(ex * ex + ey * ey);
};
const ell = (x, y, cx, cy, rx, ry) => Math.sqrt(((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2);
function landAt(x, y) {
  const n = (fbm(x / 4.5, y / 4.5, 31, 4) - 0.5) * 2.6;
  const coastX = 14.2 + 2.2 * Math.sin(y * 0.42 + 0.6) + n;
  let s = x - coastX; // > 0 is land
  s = Math.min(s, (ell(x, y, 18.2, 16.4, 3.4, 2.4) - 1) * 3 + n * 0.4); // the bay
  s = Math.min(s, (ell(x, y, 33.5, 23.8, 5.2, 3.4) - 1) * 4 + n * 0.5); // inland sea
  s = Math.min(s, y - 1.6 + n * 0.5, 45 - x + n);
  const isle = ell(x, y, 10.5, 9.5, 1.6, 1.3) < 1 + n * 0.15 || ell(x, y, 11.5, 21, 1.4, 1.8) < 1 + n * 0.12;
  return s > 0 || isle ? 1 : 0;
}
function elevAt(x, y) {
  let e = fbm(x / 3.2, y / 3.2, 7, 4) * 0.55;
  e += 0.55 * Math.max(0, 1 - segDist(x, y, 27.6, 3.5, 31.8, 13.5) / 2.1); // Hollow Crown spine
  e += 0.42 * Math.max(0, 1 - segDist(x, y, 14.8, 22.5, 17.5, 26) / 1.8); // southern range
  e += 0.3 * Math.max(0, 1 - segDist(x, y, 23.5, 5.5, 25.5, 8) / 1.6); // northern hills
  return e;
}
const moistAt = (x, y) => fbm(x / 3.4 + 40, y / 3.4 + 12, 23, 4);

export const tiles = [];
const T = (c, r) => (inb(c, r) ? tiles[r * W + c] : null);
export const tile = T;

for (let r = 0; r < H; r++)
  for (let c = 0; c < W; c++) {
    const px = c + 0.5 * (r & 1), py = r;
    tiles.push({ c, r, px, py, e: elevAt(px, py), m: moistAt(px, py), land: landAt(px, py), t: 'Ocean', owner: -1, vis: 'unexplored', city: null });
  }
{
  const landE = tiles.filter((x) => x.land).map((x) => x.e).sort((a, b) => a - b);
  const pct = (p) => landE[Math.floor(p * (landE.length - 1))];
  const eM = pct(0.93), eH = pct(0.8);
  for (const x of tiles) {
    if (!x.land) continue;
    const m = x.m + (x.px > 27 && x.py > 15 ? -0.12 : 0); // drier shores by the inland sea
    if (x.e > eM) x.t = 'Mountains';
    else if (x.e > eH) x.t = 'Hills';
    else if (x.py < 4.5 || (x.py < 6 && m < 0.5)) x.t = 'Tundra';
    else if (m < 0.36 && x.py > 14) x.t = 'Desert';
    else if (m > 0.57) x.t = 'Forest';
    else if (m > 0.47) x.t = 'Grassland';
    else x.t = 'Plains';
  }
}
export const isWater = (t) => t === 'Ocean' || t === 'Coast';

// ---------- scene content ----------
export const PLAYERS = [
  { id: 0, name: 'Ada', faction: 'Japan', color: '#4aa8ff', human: true },
  { id: 1, name: 'Bo', faction: 'Byzantium', color: '#3fd6a4', human: true },
  { id: 2, name: 'Hollow Crown', faction: 'Switzerland', color: '#e5534b' },
  { id: 3, name: 'Saint Veyra', faction: 'Japan', color: '#f2a63b' },
  { id: 4, name: 'The Meridian', faction: 'Byzantium', color: '#a97ff0' },
  { id: 5, name: 'Ashfall Compact', faction: 'Switzerland', color: '#e86fb8' },
  { id: 6, name: 'Orrery of Tesk', faction: 'Japan', color: '#b9d24a' },
  { id: 7, name: 'Pale Assembly', faction: 'Byzantium', color: '#d8dde3' },
];

export const CITIES = [
  { id: 'cantor', name: 'Cantor', owner: 0, pos: [22, 15], pop: 5, capital: true, prod: 'Settler', turns: 4, radius: 2 },
  { id: 'hymnal', name: 'Hymnal', owner: 0, pos: [18, 11], pop: 3, prod: 'Monument', turns: 2, radius: 1 },
  { id: 'astra', name: 'Astra Prime', owner: 1, pos: [21, 21], pop: 4, capital: true, prod: 'Spearman', turns: 3, radius: 2 },
  { id: 'lastlight', name: 'Last Light', owner: 1, pos: [27, 19], pop: 2, prod: 'Granary', turns: 6, radius: 1 },
  { id: 'brasshold', name: 'Brasshold', owner: 2, pos: [29, 8], pop: 6, capital: true, prod: null, turns: null, radius: 3, lastKnown: true },
];

export const UNITS = [
  { id: 'settler', kind: 'Settler', owner: 0, pos: [25, 14], hp: 100, selected: true },
  { id: 'soldier', kind: 'Spearman', owner: 0, pos: [23, 16], hp: 100, fortified: true },
  { id: 'scout', kind: 'Scout', owner: 0, pos: [17, 7], hp: 100 },
  { id: 'bo-soldier', kind: 'Spearman', owner: 1, pos: [20, 19], hp: 100 },
  { id: 'bo-ranger', kind: 'Archer', owner: 1, pos: [24, 20], hp: 64 },
  { id: 'hc-soldier', kind: 'Spearman', owner: 2, pos: [27, 11], hp: 88 },
  { id: 'sv-scout', kind: 'Scout', owner: 3, pos: [17, 17], hp: 100 },
];
export const RUINS = [[19, 6], [28, 16]];
export const TARGET = [26, 12]; // the settler's planned founding site

// Make the scene legal: cities and land units on land that isn't mountains.
const landify = ([c, r], t = 'Plains') => {
  const x = T(c, r);
  if (isWater(x.t) || x.t === 'Mountains') {
    x.t = t;
    x.land = 1;
  }
};
for (const ci of CITIES) landify(ci.pos, 'Grassland');
for (const u of UNITS) landify(u.pos);
for (const p of RUINS) landify(p);
landify(TARGET, 'Plains');
// Hills next to the target make a good site; a couple of forests around Cantor.
T(22, 15).t = 'Plains';
T(26, 12).t = 'Plains';

// Coast = water next to land.
for (const x of tiles)
  if (x.t === 'Ocean')
    for (let i = 0; i < 6; i++) {
      const n = T(...nb(x.c, x.r, i));
      if (n && !isWater(n.t)) {
        x.t = 'Coast';
        break;
      }
    }

// Territory: nearest city within its radius wins.
for (const x of tiles) {
  let best = null, bd = 99;
  for (const ci of CITIES) {
    const d = dist([x.c, x.r], ci.pos);
    const reach = ci.radius + (rnd(x.c, x.r, 3) < 0.25 && d === ci.radius + 1 ? 1 : 0);
    if (d <= reach && d < bd && !(x.t === 'Ocean')) {
      best = ci;
      bd = d;
    }
  }
  if (best) x.owner = best.owner;
}
for (const ci of CITIES) T(...ci.pos).city = ci;

// Vision: visible around our team's cities (3) and units (2, scout 3); explored in a noisy blob.
const ours = (o) => o === 0 || o === 1;
for (const x of tiles) {
  const p = [x.c, x.r];
  let vis = false;
  for (const ci of CITIES) if (ours(ci.owner) && dist(p, ci.pos) <= (ci.capital ? 4 : 3)) vis = true;
  for (const u of UNITS) if (ours(u.owner) && dist(p, u.pos) <= (u.kind === 'Scout' ? 3 : 2) + (u.kind === 'Settler' ? 1 : 0)) vis = true;
  const cx = HW * 23, cy = R * 1.5 * 14.5;
  const px = HW * (x.c + 0.5 * (x.r & 1)), py = R * 1.5 * x.r;
  const dx = (px - cx) / (HW * 13), dy = (py - cy) / (R * 1.5 * 11);
  const dd = Math.sqrt(dx * dx + dy * dy) + (fbm(x.c / 3, x.r / 3, 51, 2) - 0.5) * 0.55;
  let explored = dd < 1 || dist(p, [29, 8]) <= 4 || dist(p, [17, 7]) <= 4;
  if (vis) x.vis = 'visible';
  else if (explored) x.vis = 'explored';
}
// Brasshold's surroundings are last-known: explored but not currently visible.
for (const x of tiles) if (x.vis === 'visible' && dist([x.c, x.r], [29, 8]) <= 1) x.vis = 'explored';

// Settler reach (2 moves; forest and hills cost 2; water and mountains impassable).
export const REACH = new Set();
{
  const start = UNITS.find((u) => u.selected).pos;
  const best = new Map([[start.join(','), 2]]);
  const q = [[start, 2]];
  while (q.length) {
    const [[c, r], mv] = q.shift();
    for (let i = 0; i < 6; i++) {
      const [nc, nr] = nb(c, r, i);
      const n = T(nc, nr);
      if (!n || isWater(n.t) || n.t === 'Mountains') continue;
      const cost = n.t === 'Forest' || n.t === 'Hills' ? 2 : 1;
      const left = mv === 2 ? Math.max(0, mv - cost) : mv - cost;
      if (left < 0) continue;
      const k = `${nc},${nr}`;
      if ((best.get(k) ?? -1) >= left) continue;
      best.set(k, left);
      REACH.add(k);
      q.push([[nc, nr], left]);
    }
  }
  REACH.delete(start.join(','));
}

// ---------- paths from hex sets ----------
const vkey = ([x, y]) => `${Math.round(x * 4)},${Math.round(y * 4)}`;
/** Boundary of the tile set `inside`, chained into closed/open polylines. */
export function boundary(inside, rad = R) {
  const segs = [];
  for (const x of tiles) {
    if (!inside(x)) continue;
    const { x: cx, y: cy } = center(x.c, x.r);
    if (cx < -80 || cx > VW + 80 || cy < -80 || cy > VH + 80) continue;
    const k = corners(cx, cy, rad);
    for (let i = 0; i < 6; i++) {
      const n = T(...nb(x.c, x.r, i));
      if (n && inside(n)) continue;
      segs.push([k[i], k[(i + 1) % 6]]);
    }
  }
  return chain(segs);
}
/** Edges between two tiles where `test(a, b)` holds (each edge once). */
export function edgesWhere(test) {
  const segs = [];
  for (const x of tiles) {
    const { x: cx, y: cy } = center(x.c, x.r);
    if (cx < -60 || cx > VW + 60 || cy < -60 || cy > VH + 60) continue;
    const k = corners(cx, cy);
    for (let i = 0; i < 3; i++) {
      const n = T(...nb(x.c, x.r, i));
      if (!n || !test(x, n)) continue;
      segs.push([k[i], k[(i + 1) % 6]]);
    }
  }
  return chain(segs);
}
function chain(segs) {
  const byStart = new Map();
  for (const s of segs) {
    const k = vkey(s[0]);
    if (!byStart.has(k)) byStart.set(k, []);
    byStart.get(k).push(s);
  }
  const used = new Set();
  const parts = [];
  for (const s of segs) {
    if (used.has(s)) continue;
    used.add(s);
    const pts = [s[0], s[1]];
    let cur = s[1];
    for (;;) {
      const cands = (byStart.get(vkey(cur)) ?? []).filter((t) => !used.has(t));
      if (!cands.length) break;
      const t = cands[0];
      used.add(t);
      pts.push(t[1]);
      cur = t[1];
    }
    const closed = vkey(pts[0]) === vkey(pts[pts.length - 1]);
    if (closed) pts.pop();
    parts.push('M' + pts.map(([a, b]) => `${f1(a)} ${f1(b)}`).join('L') + (closed ? 'Z' : ''));
  }
  return parts.join('');
}
export const onScreen = (x, pad = 60) => {
  const p = center(x.c, x.r);
  return p.x > -pad && p.x < VW + pad && p.y > -pad && p.y < VH + pad;
};

// ---------- colors ----------
const toRgb = (h) => {
  const n = parseInt(h.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
};
const toHex = (a) => '#' + a.map((v) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0')).join('');
export const mix = (a, b, t) => {
  const A = toRgb(a), B = toRgb(b);
  return toHex(A.map((v, i) => v + (B[i] - v) * t));
};
export const desat = (a, t) => {
  const A = toRgb(a);
  const l = 0.3 * A[0] + 0.59 * A[1] + 0.11 * A[2];
  return toHex(A.map((v) => v + (l - v) * t));
};
export const rgba = (h, a) => {
  const [r, g, b] = toRgb(h);
  return `rgba(${r},${g},${b},${a})`;
};

// ---------- icons ----------
export function iconSprite(names) {
  const syms = names.map((n) => {
    const src = fs.readFileSync(path.join(ICON_DIR, `${n}.svg`), 'utf8');
    const vb = src.match(/viewBox="([^"]+)"/)[1];
    const inner = src.replace(/^[\s\S]*?<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '');
    return `<symbol id="i-${n}" viewBox="${vb}"><g fill="currentColor">${inner}</g></symbol>`;
  });
  return `<svg width="0" height="0" style="position:absolute" aria-hidden="true"><defs>${syms.join('')}</defs></svg>`;
}
export const icon = (n, size = 16, cls = 'ic', style = '') =>
  `<svg class="${cls}" width="${size}" height="${size}" style="${style}" aria-hidden="true"><use href="#i-${n}"/></svg>`;
export const rawIcon = (n) => fs.readFileSync(path.join(ICON_DIR, `${n}.svg`), 'utf8');

export const ALL_ICONS = fs
  .readdirSync(ICON_DIR)
  .filter((f) => f.endsWith('.svg'))
  .map((f) => f.replace(/\.svg$/, ''));

// ---------- game content shown in every direction ----------
export const GAME = {
  turn: 27,
  limit: 150,
  gold: 84,
  goldPt: 9,
  sciencePt: 11,
  research: { name: 'Signal Theory', turns: 4, progress: 22, cost: 40, unlocks: 'Library' },
  vp: [
    { label: 'Your team', members: [0, 1], vp: 6, target: 17, mine: true },
    { label: 'Hollow Crown', members: [2], vp: 5, target: 10 },
    { label: 'Saint Veyra', members: [3], vp: 4, target: 10 },
    { label: 'The Meridian', members: [4], vp: 3, target: 10 },
    { label: 'Ashfall Compact', members: [5], vp: 2, target: 10 },
    { label: 'Orrery of Tesk', members: [6], vp: 2, target: 10 },
    { label: 'Pale Assembly', members: [7], vp: 1, target: 10 },
  ],
  city: {
    name: 'Cantor',
    owner: 'Ada',
    faction: 'Japan',
    pop: 5,
    str: 20,
    hp: 200,
    maxHp: 200,
    yields: { food: 9, production: 7, gold: 4, science: 5, influence: 3 },
    food: 22,
    growAt: 47,
    growTurns: 6,
    producing: 'Settler',
    prodStored: 52,
    prodCost: 80,
    prodTurns: 4,
    buy: 112,
    options: [
      { kind: 'unit', key: 'Scout', icon: 'unit-scout', cost: 30, turns: 5 },
      { kind: 'unit', key: 'Settler', icon: 'unit-settler', cost: 80, turns: 4, active: true },
      { kind: 'unit', key: 'Spearman', icon: 'unit-spearman', cost: 40, turns: 6 },
      { kind: 'unit', key: 'Archer', icon: 'unit-archer', cost: 50, turns: 8 },
      { kind: 'building', key: 'Workshop', icon: 'building-workshop', cost: 60, turns: 9, note: '+2 Production' },
      { kind: 'building', key: 'Market', icon: 'building-market', cost: 60, turns: 9, note: '+3 Gold' },
      { kind: 'building', key: 'Walls', icon: 'building-walls', cost: 60, turns: 9, note: '+10 city strength' },
    ],
    focus: 'Balanced',
    buildings: ['Monument', 'Granary'],
    tiles: 14,
    influence: 12,
    nextTile: 25,
    breakdown: {
      key: 'production',
      title: 'Production',
      rows: [
        ['City center', 2],
        ['Worked Hills (2 tiles)', 4],
        ['Worked Forest', 2],
        ['Balanced focus', 1],
        ['Japan: tuned resonance', -2],
      ],
      total: 7,
      note: 'Worked tiles are picked by focus. Change focus to trade Food for Production.',
    },
  },
  unit: {
    kind: 'Settler',
    owner: 'Ada',
    hp: 100,
    moves: 2,
    maxMoves: 2,
    strength: 0,
    status: 'Awaiting orders',
    blurb: 'Founds a new city. Needs population 2; costs 1 pop.',
  },
  seats: [
    { name: 'Ada', faction: 'Japan', color: '#4aa8ff', orders: 3, ready: false, on: true },
    { name: 'Bo', faction: 'Byzantium', color: '#3fd6a4', orders: 5, ready: true },
  ],
  needing: 2,
};

export const YIELD_ORDER = ['food', 'production', 'gold', 'science', 'influence'];
export const YIELD_LABEL = { food: 'Food', production: 'Production', gold: 'Gold', science: 'Science', influence: 'Influence' };

// ---------- painterly map ----------
// Map rendering options control terrain colors, fog, relief, borders, and patterns.
export function renderMap(style, overlay = '') {
  const S = style;
  const vis = tiles.filter((x) => onScreen(x, 80));
  const land = vis.filter((x) => !isWater(x.t));
  const known = (x) => x.vis !== 'unexplored';
  const shade = (x) => {
    // Hillshade from the elevation gradient, lit from the north-west.
    const d = 0.35;
    const g = elevAt(x.px - d, x.py - d) - elevAt(x.px + d, x.py + d);
    return Math.max(-1, Math.min(1, g * 7));
  };
  const colorOf = (x) => {
    let c = S.pal[x.t];
    const j = (rnd(x.c, x.r, 9) - 0.5) * 0.08;
    c = j > 0 ? mix(c, '#ffffff', j) : mix(c, '#000000', -j);
    return c;
  };

  // Land colour field: land hexes plus coastal water painted with a neighbouring land colour so the blur has no dark fringe.
  let landCol = '';
  for (const x of vis) {
    const { x: cx, y: cy } = center(x.c, x.r);
    if (!isWater(x.t)) landCol += `<path d="${hexPath(cx, cy, R + 0.6)}" fill="${colorOf(x)}"/>`;
    else if (x.t === 'Coast') {
      let n = null;
      for (let i = 0; i < 6 && !n; i++) {
        const t = T(...nb(x.c, x.r, i));
        if (t && !isWater(t.t)) n = t;
      }
      if (n) landCol += `<path d="${hexPath(cx, cy, R + 0.6)}" fill="${colorOf(n)}"/>`;
    }
  }
  const landShapes = land.map((x) => {
    const { x: cx, y: cy } = center(x.c, x.r);
    return `<path d="${hexPath(cx, cy, R + 0.5)}"/>`;
  }).join('');

  // Relief: per-hex light/dark, blurred into a soft hillshade.
  let relief = '';
  for (const x of land) {
    const s = shade(x) + (x.t === 'Mountains' ? 0.25 : x.t === 'Hills' ? 0.12 : 0);
    const { x: cx, y: cy } = center(x.c, x.r);
    if (Math.abs(s) < 0.04) continue;
    const col = s > 0 ? S.light : S.shadow;
    relief += `<path d="${hexPath(cx, cy, R)}" fill="${col}" fill-opacity="${f1(Math.min(1, Math.abs(s)) * 10) / 10}"/>`;
  }

  // Shallow water: coast hexes, blurred into the ocean.
  const shallow = vis
    .filter((x) => x.t === 'Coast')
    .map((x) => {
      const { x: cx, y: cy } = center(x.c, x.r);
      return `<path d="${hexPath(cx, cy, R * 1.05)}"/>`;
    })
    .join('');

  // Glyphs: forests, hills, mountains, desert dunes, tundra, grass tufts, sea waves.
  let glyphs = '';
  const fogOp = (x) => (x.vis === 'explored' ? S.fog.glyph ?? 0.7 : 1);
  for (const x of vis) {
    if (!known(x)) continue;
    const { x: cx, y: cy } = center(x.c, x.r);
    const rr = (k) => rnd(x.c, x.r, k);
    const op = fogOp(x);
    if (x.city) continue;
    if (x.t === 'Forest') {
      const n = 6 + Math.floor(rr(1) * 3);
      const trees = [];
      for (let k = 0; k < n; k++) {
        const a = rr(10 + k) * Math.PI * 2, d = Math.sqrt(rr(20 + k)) * R * 0.62;
        trees.push([cx + Math.cos(a) * d, cy + Math.sin(a) * d * 0.85, 3.6 + rr(30 + k) * 2.2]);
      }
      trees.sort((a, b) => a[1] - b[1]);
      glyphs += `<g opacity="${op}">`;
      for (const [tx, ty, tr] of trees)
        glyphs += `<ellipse cx="${f1(tx + 1.6)}" cy="${f1(ty + 2.2)}" rx="${f1(tr)}" ry="${f1(tr * 0.7)}" fill="${S.tree.shadow}"/>`;
      for (const [tx, ty, tr] of trees)
        glyphs += `<circle cx="${f1(tx)}" cy="${f1(ty)}" r="${f1(tr)}" fill="${S.tree.dark}"/><circle cx="${f1(tx - tr * 0.3)}" cy="${f1(ty - tr * 0.35)}" r="${f1(tr * 0.55)}" fill="${S.tree.light}"/>`;
      glyphs += `</g>`;
    } else if (x.t === 'Mountains') {
      const peaks = rr(2) < 0.5 ? [[0, 2, 1]] : [[-7, 5, 0.72], [5, 1, 1]];
      glyphs += `<g opacity="${op}">`;
      for (const [ox, oy, sc] of peaks) {
        const px = cx + ox, py = cy + oy + 8;
        const w = R * 0.78 * sc, h = R * 0.95 * sc;
        const ax = px + (rr(3) - 0.5) * 4, ay = py - h;
        const mx = px + w * 0.08, my = py;
        const l = [px - w, py], rgt = [px + w, py];
        const ridge = `${f1(ax)},${f1(ay)} ${f1(ax + w * 0.18)},${f1(ay + h * 0.32)} ${f1(mx + w * 0.1)},${f1(py - h * 0.3)} ${f1(mx)},${f1(my)}`;
        glyphs += `<ellipse cx="${f1(px + 3)}" cy="${f1(py + 1)}" rx="${f1(w * 1.02)}" ry="${f1(3.2 * sc)}" fill="${S.peak.base}"/>`;
        glyphs += `<polygon points="${f1(l[0])},${f1(l[1])} ${ridge}" fill="${S.peak.light}"/>`;
        glyphs += `<polygon points="${ridge} ${f1(rgt[0])},${f1(rgt[1])}" fill="${S.peak.dark}"/>`;
        glyphs += `<polygon points="${f1(ax)},${f1(ay)} ${f1(ax - w * 0.3)},${f1(ay + h * 0.32)} ${f1(ax - w * 0.1)},${f1(ay + h * 0.26)} ${f1(ax + w * 0.02)},${f1(ay + h * 0.36)} ${f1(ax + w * 0.18)},${f1(ay + h * 0.32)}" fill="${S.peak.snow}"/>`;
        glyphs += `<polyline points="${f1(l[0])},${f1(l[1])} ${f1(ax)},${f1(ay)} ${f1(ax + w * 0.18)},${f1(ay + h * 0.32)} ${f1(rgt[0])},${f1(rgt[1])}" fill="none" stroke="${S.peak.line}" stroke-width="0.9" stroke-linejoin="round"/>`;
        // hachure on the shadow face
        for (let k = 0; k < 4; k++) {
          const t = 0.3 + k * 0.16;
          const hx = ax + (rgt[0] - ax) * t, hy = ay + (rgt[1] - ay) * t;
          glyphs += `<line x1="${f1(hx)}" y1="${f1(hy)}" x2="${f1(hx - w * 0.14)}" y2="${f1(hy + h * 0.1)}" stroke="${S.peak.line}" stroke-width="0.7" opacity=".55"/>`;
        }
      }
      glyphs += `</g>`;
    } else if (x.t === 'Hills') {
      const hs = [[-8, 6, 0.9], [7, 3, 1.05]];
      if (rr(4) < 0.5) hs.push([-1, -9, 0.75]);
      glyphs += `<g opacity="${op}">`;
      for (const [ox, oy, sc] of hs) {
        const px = cx + ox, py = cy + oy;
        const w = 10 * sc, h = 7 * sc;
        glyphs += `<path d="M${f1(px - w)} ${f1(py)}Q${f1(px - w * 0.2)} ${f1(py - h * 1.5)} ${f1(px + w)} ${f1(py)}Z" fill="${S.hill.fill}"/>`;
        glyphs += `<path d="M${f1(px - w)} ${f1(py)}Q${f1(px - w * 0.2)} ${f1(py - h * 1.5)} ${f1(px + w)} ${f1(py)}" fill="none" stroke="${S.hill.line}" stroke-width="1" stroke-linecap="round"/>`;
        for (let k = 0; k < 3; k++) {
          const hx = px + w * (0.1 + k * 0.28), hy = py - h * (0.62 - k * 0.2);
          glyphs += `<line x1="${f1(hx)}" y1="${f1(hy)}" x2="${f1(hx + 1.2)}" y2="${f1(py - 0.8)}" stroke="${S.hill.line}" stroke-width=".8" opacity=".6"/>`;
        }
      }
      glyphs += `</g>`;
    } else if (x.t === 'Desert') {
      glyphs += `<g opacity="${op * 0.9}" fill="none" stroke="${S.dune}" stroke-width="1" stroke-linecap="round">`;
      for (let k = 0; k < 3; k++) {
        const ox = (rr(40 + k) - 0.5) * R, oy = (k - 1) * 9 + (rr(50 + k) - 0.5) * 4;
        glyphs += `<path d="M${f1(cx + ox - 8)} ${f1(cy + oy)}q4 -4 8 0t8 0"/>`;
      }
      glyphs += `</g>`;
    } else if (x.t === 'Tundra') {
      glyphs += `<g opacity="${op}">`;
      for (let k = 0; k < 3; k++) {
        const ox = (rr(60 + k) - 0.5) * R * 1.1, oy = (rr(70 + k) - 0.5) * R * 0.9;
        glyphs += `<ellipse cx="${f1(cx + ox)}" cy="${f1(cy + oy)}" rx="${f1(4 + rr(80 + k) * 4)}" ry="1.6" fill="${S.snow}"/>`;
      }
      glyphs += `</g>`;
    } else if (x.t === 'Grassland' || x.t === 'Plains') {
      const n = x.t === 'Grassland' ? 4 : 3;
      glyphs += `<g opacity="${op * 0.8}" stroke="${x.t === 'Grassland' ? S.tuft.grass : S.tuft.plains}" stroke-width=".9" stroke-linecap="round" fill="none">`;
      for (let k = 0; k < n; k++) {
        const ox = (rr(90 + k) - 0.5) * R * 1.1, oy = (rr(100 + k) - 0.5) * R;
        const tx = cx + ox, ty = cy + oy;
        glyphs += `<path d="M${f1(tx - 2)} ${f1(ty)}l-1 -3M${f1(tx)} ${f1(ty)}v-4M${f1(tx + 2)} ${f1(ty)}l1 -3"/>`;
      }
      glyphs += `</g>`;
    } else if (x.t === 'Ocean' && rr(5) < 0.22) {
      const ox = (rr(6) - 0.5) * R * 0.8, oy = (rr(7) - 0.5) * R * 0.6;
      glyphs += `<path d="M${f1(cx + ox - 7)} ${f1(cy + oy)}q3.5 -3 7 0t7 0" fill="none" stroke="${S.wave}" stroke-width="1" stroke-linecap="round" opacity="${op}"/>`;
    }
  }

  // City footprints: a cluster of roofs on paved ground.
  let towns = '';
  for (const ci of CITIES) {
    const { x: cx, y: cy } = center(...ci.pos);
    const n = 5 + ci.pop;
    towns += `<g opacity="${T(...ci.pos).vis === 'explored' ? 0.75 : 1}"><path d="${hexPath(cx, cy, R * 0.82)}" fill="${S.town.ground}" opacity=".9"/>`;
    const blocks = [];
    for (let k = 0; k < n; k++) {
      const a = rnd(ci.pos[0], k, 70) * Math.PI * 2, d = Math.sqrt(rnd(ci.pos[1], k, 71)) * R * 0.55;
      const w = 4 + rnd(k, ci.pos[0], 72) * 5, h = 3.5 + rnd(k, ci.pos[1], 73) * 3.5;
      blocks.push([cx + Math.cos(a) * d - w / 2, cy + Math.sin(a) * d * 0.8 - h / 2, w, h]);
    }
    blocks.sort((a, b) => a[1] - b[1]);
    for (const [bx, by, w, h] of blocks)
      towns += `<rect x="${f1(bx + 1.2)}" y="${f1(by + 1.5)}" width="${f1(w)}" height="${f1(h)}" fill="${S.town.shadow}"/><rect x="${f1(bx)}" y="${f1(by)}" width="${f1(w)}" height="${f1(h)}" fill="${S.town.roof}"/><rect x="${f1(bx)}" y="${f1(by)}" width="${f1(w)}" height="${f1(h * 0.45)}" fill="${S.town.roofLight}"/>`;
    if (ci.capital)
      towns += `<rect x="${f1(cx - 2.5)}" y="${f1(cy - 13)}" width="5" height="13" fill="${S.town.shadow}"/><rect x="${f1(cx - 3.5)}" y="${f1(cy - 14)}" width="5" height="13" fill="${S.town.roofLight}"/><path d="M${f1(cx - 3.5)} ${f1(cy - 14)}l2.5 -5l2.5 5z" fill="${S.town.roof}"/>`;
    towns += `</g>`;
  }
  for (const [c, r] of RUINS) {
    const { x: cx, y: cy } = center(c, r);
    towns += `<use href="#i-ruin" x="${f1(cx - 9)}" y="${f1(cy - 9)}" width="18" height="18" style="color:${S.ruin}"/>`;
  }

  // Grid: unique edges between known tiles.
  const gridLand = edgesWhere((a, b) => known(a) && known(b) && !(isWater(a.t) && isWater(b.t)));
  const gridWater = edgesWhere((a, b) => known(a) && known(b) && isWater(a.t) && isWater(b.t));

  // Territory.
  let terr = '';
  const clipDefs = [];
  for (const p of PLAYERS) {
    const mine = vis.filter((x) => x.owner === p.id && known(x));
    if (!mine.length) continue;
    const shapes = mine.map((x) => {
      const { x: cx, y: cy } = center(x.c, x.r);
      return `<path d="${hexPath(cx, cy, R + 0.4)}"/>`;
    }).join('');
    clipDefs.push(`<clipPath id="terr${p.id}">${shapes}</clipPath>`);
    const b = boundary((x) => x.owner === p.id && known(x));
    const col = p.color;
    terr += `<g clip-path="url(#terr${p.id})">`;
    terr += `<g>${shapes.replaceAll('<path', `<path fill="${col}" fill-opacity="${S.border.tint}"`)}</g>`;
    terr += `<path d="${b}" fill="none" stroke="${col}" stroke-width="${S.border.band * 2}" stroke-opacity="${S.border.bandAlpha}" ${S.border.blur ? 'filter="url(#bandBlur)"' : ''}/>`;
    if (S.border.dash)
      terr += `<path d="${b}" fill="none" stroke="${S.border.under ?? col}" stroke-width="${S.border.width * 2 + 2}" stroke-opacity=".5"/>`;
    terr += `<path d="${b}" fill="none" stroke="${col}" stroke-width="${S.border.width * 2}" ${S.border.dash ? `stroke-dasharray="${S.border.dash}"` : ''} stroke-linejoin="round"/>`;
    terr += `</g>`;
  }

  // Fog.
  const explored = vis.filter((x) => x.vis === 'explored');
  const fogShapes = explored.map((x) => {
    const { x: cx, y: cy } = center(x.c, x.r);
    return `<path d="${hexPath(cx, cy, R + 0.5)}"/>`;
  }).join('');
  const unx = vis.filter((x) => x.vis === 'unexplored');
  const unxShapes = unx.map((x) => {
    const { x: cx, y: cy } = center(x.c, x.r);
    return `<path d="${hexPath(cx, cy, R + 1)}"/>`;
  }).join('');
  const visEdge = boundary((x) => x.vis === 'visible');
  const knownEdge = boundary((x) => x.vis !== 'unexplored');

  const defs = `
  <filter id="landShape" filterUnits="userSpaceOnUse" x="0" y="0" width="${VW}" height="${VH}" color-interpolation-filters="sRGB">
    <feGaussianBlur in="SourceAlpha" stdDeviation="6.5" result="b"/>
    <feComponentTransfer in="b" result="t"><feFuncA type="linear" slope="16" intercept="-7.6"/></feComponentTransfer>
    <feTurbulence type="fractalNoise" baseFrequency="0.045" numOctaves="3" seed="11" result="n"/>
    <feDisplacementMap in="t" in2="n" scale="11" xChannelSelector="R" yChannelSelector="G" result="d"/>
    <feFlood flood-color="#fff"/><feComposite in2="d" operator="in"/>
  </filter>
  <filter id="coastLines" filterUnits="userSpaceOnUse" x="0" y="0" width="${VW}" height="${VH}" color-interpolation-filters="sRGB">
    <feGaussianBlur in="SourceAlpha" stdDeviation="6.5" result="b"/>
    <feComponentTransfer in="b" result="t"><feFuncA type="linear" slope="16" intercept="-7.6"/></feComponentTransfer>
    <feTurbulence type="fractalNoise" baseFrequency="0.045" numOctaves="3" seed="11" result="n"/>
    <feDisplacementMap in="t" in2="n" scale="11" xChannelSelector="R" yChannelSelector="G" result="d"/>
    <feMorphology in="d" operator="dilate" radius="${S.coastWidth ?? 1.1}" result="d1"/>
    <feComposite in="d1" in2="d" operator="out" result="ring0"/>
    <feMorphology in="d" operator="dilate" radius="5" result="d5"/>
    <feMorphology in="d" operator="dilate" radius="5.8" result="d6"/>
    <feComposite in="d6" in2="d5" operator="out" result="ring1"/>
    <feMorphology in="d" operator="dilate" radius="10" result="d10"/>
    <feMorphology in="d" operator="dilate" radius="10.7" result="d11"/>
    <feComposite in="d11" in2="d10" operator="out" result="ring2"/>
    <feFlood flood-color="${S.coast}" result="c0"/><feComposite in="c0" in2="ring0" operator="in" result="r0"/>
    <feFlood flood-color="${S.waveLine}" flood-opacity="${S.waveAlpha ?? 0.55}" result="c1"/><feComposite in="c1" in2="ring1" operator="in" result="r1"/>
    <feFlood flood-color="${S.waveLine}" flood-opacity="${(S.waveAlpha ?? 0.55) * 0.5}" result="c2"/><feComposite in="c2" in2="ring2" operator="in" result="r2"/>
    <feMerge><feMergeNode in="r2"/><feMergeNode in="r1"/><feMergeNode in="r0"/></feMerge>
  </filter>
  <filter id="soft" x="-5%" y="-5%" width="110%" height="110%"><feGaussianBlur stdDeviation="${S.blend ?? 4}"/></filter>
  <filter id="reliefBlur" x="-5%" y="-5%" width="110%" height="110%"><feGaussianBlur stdDeviation="9"/></filter>
  <filter id="shallowBlur" x="-5%" y="-5%" width="110%" height="110%"><feGaussianBlur stdDeviation="12"/></filter>
  <filter id="bandBlur" x="-5%" y="-5%" width="110%" height="110%"><feGaussianBlur stdDeviation="${S.border.blur ?? 3}"/></filter>
  <filter id="unxBlur" x="-5%" y="-5%" width="110%" height="110%"><feGaussianBlur stdDeviation="${S.unexploredBlur ?? 2.5}"/></filter>
  <filter id="grain" filterUnits="userSpaceOnUse" x="0" y="0" width="${VW}" height="${VH}">
    <feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="4"/>
    <feColorMatrix values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  1.1 0 0 0 -0.42"/>
  </filter>
  <filter id="mottle" filterUnits="userSpaceOnUse" x="0" y="0" width="${VW}" height="${VH}">
    <feTurbulence type="fractalNoise" baseFrequency="0.011 0.014" numOctaves="4" seed="9"/>
    <feColorMatrix type="saturate" values="0"/>
    <feComponentTransfer><feFuncA type="linear" slope="0" intercept="1"/></feComponentTransfer>
  </filter>
  <mask id="landMask" maskUnits="userSpaceOnUse" x="0" y="0" width="${VW}" height="${VH}"><g filter="url(#landShape)">${landShapes}</g></mask>
  <pattern id="fogHatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="6" height="6" fill="none"/><line x1="0" y1="0" x2="0" y2="6" stroke="${S.fog.hatch}" stroke-width="1"/></pattern>
  ${S.unexploredPattern ?? ''}
  <radialGradient id="vignette" cx="50%" cy="52%" r="72%"><stop offset=".55" stop-color="${S.vignetteColor ?? '#000'}" stop-opacity="0"/><stop offset="1" stop-color="${S.vignetteColor ?? '#000'}" stop-opacity="1"/></radialGradient>
  ${clipDefs.join('')}
  `;

  return `<svg class="map" width="${VW}" height="${VH}" viewBox="0 0 ${VW} ${VH}" xmlns="http://www.w3.org/2000/svg">
  <defs>${defs}</defs>
  <rect width="${VW}" height="${VH}" fill="${S.deep}"/>
  <rect width="${VW}" height="${VH}" filter="url(#mottle)" opacity="${S.seaMottle ?? 0.08}" style="mix-blend-mode:${S.mottleBlend ?? 'overlay'}"/>
  <g filter="url(#shallowBlur)" fill="${S.shallow}" opacity=".9">${shallow}</g>
  <g filter="url(#coastLines)">${landShapes}</g>
  <g mask="url(#landMask)">
    <g filter="url(#soft)">${landCol}</g>
    <g filter="url(#reliefBlur)" opacity="${S.relief}">${relief}</g>
    <rect width="${VW}" height="${VH}" filter="url(#mottle)" opacity="${S.mottle}" style="mix-blend-mode:${S.mottleBlend ?? 'overlay'}"/>
    <rect width="${VW}" height="${VH}" filter="url(#grain)" opacity="${S.grain}"/>
  </g>
  <path d="${gridWater}" fill="none" stroke="${S.gridWater}" stroke-width="1"/>
  <path d="${gridLand}" fill="none" stroke="${S.grid}" stroke-width="1"/>
  <g>${glyphs}</g>
  <g>${towns}</g>
  <g class="fog">
    <g fill="#808080" style="mix-blend-mode:saturation" opacity="${S.fog.desat}">${fogShapes}</g>
    <g fill="${S.fog.tint}" opacity="${S.fog.alpha}" ${S.fog.blend ? `style="mix-blend-mode:${S.fog.blend}"` : ''}>${fogShapes}</g>
  </g>
  <g>${terr}</g>
  <g class="fog2">
    <g fill="url(#fogHatch)" opacity="${S.fog.hatchAlpha}">${fogShapes}</g>
    <path d="${visEdge}" fill="none" stroke="${S.fog.edge}" stroke-width="${S.fog.edgeWidth ?? 1}" ${S.fog.edgeDash ? `stroke-dasharray="${S.fog.edgeDash}"` : ''}/>
  </g>
  <g filter="url(#unxBlur)">
    <g fill="${S.unexplored}">${unxShapes}</g>
    ${S.unexploredPattern ? `<g fill="url(#unx)">${unxShapes}</g>` : ''}
  </g>
  <path d="${knownEdge}" fill="none" stroke="${S.unexploredEdge}" stroke-width="1"/>
  ${S.vignette ? `<rect width="${VW}" height="${VH}" fill="url(#vignette)" opacity="${S.vignette}" pointer-events="none"/>` : ''}
  ${overlay}
</svg>`;
}

// Screen position helpers for overlays.
export const at = (pos) => center(pos[0], pos[1]);
export const reachShapes = () =>
  [...REACH].map((k) => {
    const [c, r] = k.split(',').map(Number);
    const p = center(c, r);
    return hexPath(p.x, p.y, R - 1.5);
  }).join('');
export const reachEdge = () => boundary((x) => REACH.has(`${x.c},${x.r}`) || (x.c === 25 && x.r === 14));
export const ringEdge = (pos, rad) => boundary((x) => dist([x.c, x.r], pos) <= rad);

export function page({ title, fonts, css, body }) {
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>${title}</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?${fonts}&display=swap" rel="stylesheet">
<style>${css}</style></head>
<body>${body}</body></html>`;
}
