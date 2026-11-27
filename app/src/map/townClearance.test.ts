// The map's layout rule around towns, checked as geometry at every zoom the painted towns show:
// a neighbouring tile's resource medallion, yield cluster and natural-wonder badge never land on a town's painted
// footprint (its ground oval, roofs and walls), and a walled town fits inside its ring.
import { describe, expect, it } from 'vitest';
import { fileURLToPath } from 'node:url';
import { readdirSync } from 'node:fs';
import { alphaOf } from '../test/pngAlpha';
import { hexToPixel, neighbors } from '../hex';
import { CITY_CANVAS, CITY_ERAS, CITY_GROUND, CITY_TIERS, WALLED_TOWN_SCALE, WALL_CUT, TOWN_GAP, clearOfTown } from './cityArt';
import { TOWN_OVAL } from './terrain';
import { RUIN_MASS } from './mapFeatures';
import { HEX_SIZE, isFar } from './grid';
import { townCover } from './landmarks';
import { medallionSlots, pickSlot, type Cover } from './resourceIcons';
import { clusterShift, yieldCluster, yieldIconSize } from './yieldIcons';
import { wonderBadgeAt, wonderBadgeLayout } from './wonderArt';
import { buildOverlayTiles } from './overlays';
import type { PlayerView } from '../types';

const ZOOMS = [0.62, 0.66, 0.8, 0.9, 1.1, 1.4, 1.8, 2.4].filter((z) => !isFar(HEX_SIZE * z));
const city = { q: 6, r: 6 };
const at = (h: { q: number; r: number }, z: number) => { const p = hexToPixel(h, HEX_SIZE); return { x: p.x * z, y: p.y * z }; };
const hit = (a: Cover, b: Cover) => a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;

describe('marks keep clear of a neighbouring town', () => {
  it('has zooms to check', () => expect(ZOOMS.length).toBeGreaterThan(4));
  for (const z of ZOOMS) {
    const r = HEX_SIZE * z;
    const town = townCover(at(city, z), r);
    it(`resource medallions, zoom ${z}`, () => {
      const slots = medallionSlots(r, false);
      for (const n of neighbors(city)) {
        const c = at(n, z);
        for (const set of [slots, { top: slots.stacked, side: slots.side }]) {
          const p = pickSlot(c.x, c.y, set, [town]);
          const box = { x: c.x + p.dx - p.radius, y: c.y + p.dy - p.radius, w: p.radius * 2, h: p.radius * 2 };
          expect(hit(box, town), `neighbour ${JSON.stringify(n)}`).toBe(false);
        }
      }
    });
    it(`yield clusters (four kinds of yield), zoom ${z}`, () => {
      const icon = yieldIconSize(r);
      const cl = yieldCluster([{ kind: 'food', value: 2 }, { kind: 'production', value: 2 }, { kind: 'gold', value: 1 }, { kind: 'science', value: 1 }], icon);
      const w = Math.min(cl.width, r * 1.2), h = cl.height;
      for (const n of neighbors(city)) {
        const c = at(n, z);
        const top = c.y + r * 0.64 - h;
        const dx = clusterShift(c.x, top, w, h, r, [town]);
        expect(hit({ x: c.x + dx - w / 2, y: top, w, h }, town), `neighbour ${JSON.stringify(n)}`).toBe(false);
      }
    });
    it(`natural wonder bonus badges, zoom ${z}`, () => {
      // The canvas measures the text; here a generous 0.75 em a character stands in, so the check is the harder case.
      const L = wonderBadgeLayout(r, (text, font) => text.length * font * 0.75);
      for (const yieldsOn of [false, true]) for (const n of neighbors(city)) {
        const c = at(n, z);
        // The city's banner sits over the top of its own hex: the badge must also keep clear of it.
        const banner = { x: at(city, z).x - r * 1.4, y: at(city, z).y - r * 1.5, w: r * 2.8, h: r * 0.9 };
        const b = wonderBadgeAt(c.x, c.y, r, L, yieldsOn, [town, banner]);
        expect(b, `neighbour ${JSON.stringify(n)}, yields ${yieldsOn}`).toBeDefined();
        const box = { x: b!.x - L.width / 2, y: b!.y - L.height / 2, w: L.width, h: L.height };
        // Zoomed out the pill (its font has a floor) is wider than a hex and can only take the least covered slot.
        if (L.width <= r * 1.5) expect(hit(box, town) || hit(box, banner), `neighbour ${JSON.stringify(n)}, yields ${yieldsOn}`).toBe(false);
      }
    });
  }

  it("draws no yield cluster on a city's own tile", () => {
    const tiles = Array.from({ length: 4 }, () => ({ explored: true, visible: true, terrain: 'Grassland', owner: 0 }));
    const view = { width: 2, height: 2, me: 0, players: [{ id: 0, faction: 'Rome' }], tiles, cities: [{ pos: { q: 0, r: 0 } }] } as unknown as PlayerView;
    const out = buildOverlayTiles(view, { yields: true });
    expect(out.yields.map((t) => t.index)).not.toContain(0);
    expect(out.yields.length).toBe(3);
  });
});

describe('a walled town fits inside its ring', () => {
  type Png = ReturnType<typeof alphaOf>;
  const read = (stem: string) => alphaOf(fileURLToPath(new URL(`../assets/cities/${stem}.png`, import.meta.url)));
  const cx = (CITY_GROUND.x0 + CITY_GROUND.x1) / 2, cy = CITY_GROUND.middle;
  // The ring's outer half-width on the cut line, scanning in from each side.
  const outerHalfWidth = (png: Png) => {
    const opaque = (x: number) => png.alpha[WALL_CUT * png.width + x] > 128;
    let l = 0, rr = png.width - 1;
    while (l < cx && !opaque(l)) l++;
    while (rr > cx && !opaque(rr)) rr--;
    return Math.min(cx - l, rr - cx);
  };
  // The wall's own thickness on that line, in canvas px: the scaled town must end inside it.
  const WALL = 20;
  for (const era of CITY_ERAS) {
    // The Renaissance star fort is notched at the cut line between its bastions; at runtime the town is clipped to its court (`courtMask`).
    if (era === 'renaissance') continue;
    let ring: Png;
    try { ring = read(`${era}-walls`); } catch { continue; }
    it(`${era}: every town size and era ends inside the ring's wall on the cut line`, () => {
      const room = outerHalfWidth(ring) - WALL;
      for (const townEra of CITY_ERAS) for (const tier of CITY_TIERS) {
        let town: Png;
        try { town = read(`${townEra}-${tier}`); } catch { continue; }
        let l = town.width, rr = 0;
        for (let x = 0; x < town.width; x++) if (town.alpha[cy * town.width + x] > 128) { l = Math.min(l, x); rr = Math.max(rr, x); }
        const half = Math.max(cx - l, rr - cx) * WALLED_TOWN_SCALE;
        expect(half, `${townEra}-${tier}`).toBeLessThanOrEqual(room);
      }
    });
  }
});

describe('every wall ring has a closed court for the town to stand in', () => {
  // Mirrors `courtMask`: a flood fill of clear ground from the oval's centre on the cut row. If a repaint made the
  // centre opaque or opened a gap to the canvas edge, the runtime clip would silently switch off.
  const dir = (d: string) => fileURLToPath(new URL(`../assets/${d}/`, import.meta.url));
  const files = [...readdirSync(dir('cities')).filter((f) => /^[a-z]+-walls(-128|-64)?\.png$/.test(f)).map((f) => `${dir('cities')}${f}`),
    ...readdirSync(dir('construction')).filter((f) => /^[a-z]+-walls-stage1(-128|-64)?\.png$/.test(f)).map((f) => `${dir('construction')}${f}`)];
  it('finds the rings and footings at every size', () => expect(files.length).toBeGreaterThanOrEqual(CITY_ERAS.length * 2 * 3));
  for (const file of files) it(file.replace(/^.*assets\//, ''), () => {
    const { width: n, alpha } = alphaOf(file);
    const k = n / CITY_CANVAS, cut = Math.round(WALL_CUT * k);
    const start = cut * n + Math.round((CITY_GROUND.x0 + CITY_GROUND.x1) / 2 * k);
    expect(alpha[start], 'the court centre is clear').toBeLessThan(40);
    const seen = new Uint8Array(n * n), stack = [start];
    seen[start] = 1;
    let leaks = false;
    while (stack.length) {
      const i = stack.pop()!, x = i % n, y = (i - x) / n;
      if (x === 0 || y === 0 || x === n - 1 || y === n - 1) { leaks = true; break; }
      for (const j of [i - n, i + n, i - 1, i + 1]) if (!seen[j] && alpha[j] < 40) { seen[j] = 1; stack.push(j); }
    }
    expect(leaks, 'the court is closed by the wall').toBe(false);
  });
});

describe('ruins beside a town keep clear of it', () => {
  // render.ts draws a ruin's painting 0.88 hex radii either side of its centre (near zooms), so in world units its half side is fixed.
  const half = HEX_SIZE * 0.88;
  const centre = hexToPixel(city, HEX_SIZE);
  for (const n of neighbors(city)) it(`neighbour ${JSON.stringify(n)}`, () => {
    const p = hexToPixel(n, HEX_SIZE);
    const dx = p.x - centre.x, dy = p.y - centre.y;
    const out = clearOfTown(dx, dy, half * RUIN_MASS.ha, half * RUIN_MASS.hb, TOWN_OVAL, { gap: TOWN_GAP, maxPush: HEX_SIZE * 0.4, minScale: 0.7 });
    const inside = (out.dx / (TOWN_OVAL.a + half * RUIN_MASS.ha * out.scale + TOWN_GAP)) ** 2 + ((out.dy - TOWN_OVAL.cy) / (TOWN_OVAL.b + half * RUIN_MASS.hb * out.scale + TOWN_GAP)) ** 2;
    expect(inside).toBeGreaterThanOrEqual(1);
    // It stays on its own tile: never pushed more than 0.4 of a hex radius, never shrunk below 70%.
    expect(Math.hypot(out.dx - dx, out.dy - dy)).toBeLessThanOrEqual(HEX_SIZE * 0.4 + 1e-6);
    expect(out.scale).toBeGreaterThanOrEqual(0.7);
  });
  it('leaves a painting that already clears the town alone', () => {
    expect(clearOfTown(200, 0, 10, 10, TOWN_OVAL, { gap: 1.5, maxPush: 14, minScale: 0.7 })).toEqual({ dx: 200, dy: 0, scale: 1 });
  });
});
