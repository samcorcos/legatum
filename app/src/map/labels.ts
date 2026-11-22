// Map lettering: names for the seas, ranges and regions a player has explored. Names are
// deterministic per game (salted by the players) and per map region, so a sea keeps its name
// as more of it is revealed. The map is cut into large cells; a cell that is mostly explored
// water gets a sea name, a cluster of mountains a range name, and open land a region name.
import type { PlayerView, Terrain } from '../types';
import { centerX, centerY, neighborIndex } from './grid';
import { hashString, rnd } from './noise';

export interface MapLabel {
  text: string;
  kind: 'sea' | 'land' | 'range';
  x: number;
  y: number;
  /** Radians; ranges follow their spine. */
  angle: number;
}

const SEAS = [
  'The Sundering Sea',
  'The Glass Deep',
  'Sea of Quiet Stars',
  'The Hollow Gulf',
  'Bay of Embers',
  'The Tidewrack',
  'Sea of Heirs',
  'The Drowned India',
  'Gulf of Vesper',
  'The Silent Mere',
  'The Long Water',
  'The Cinder Strait',
  'Sea of Lanterns',
  'The Mirror Deep',
  'The Brine Crown',
  'Gulf of Ash',
  'The Starfall Sea',
  'The Grey Abyss',
];
const RANGES = [
  'Spine of Tesk',
  'The Iron Teeth',
  'The Shattered Crowns',
  'The Hollow Peaks',
  'The Warden Range',
  'Spires of Cadence',
  'The Broken Stair',
  'The Anvil Heights',
  'The Mourning Ridge',
  'The Sentinel Crags',
];
const REGIONS: Record<string, string[]> = {
  Forest: ['The Greenwold', 'Thornwood', 'The Old Canopy', 'Mossreach', 'The Deepwood', 'Vesperwood'],
  Desert: ['The Ashen Waste', 'The Glasslands', 'The Sunscar', 'The Amber Barrens', 'Dust of Kings'],
  Tundra: ['The Pale Reach', 'Frostmarch', 'The White Silence', 'Rimeholt'],
  Hills: ['The Tumbled Downs', 'The Cairnlands', 'The Rolling Wolds', 'The Barrowfold'],
  Open: ['Verdance', 'The Gilded Steppe', 'Harrowfield', 'The Meadowmarch', 'Lanternvale', 'The Wide Accord', 'Emberfield', 'The Throne Lands'],
};

/** A per-game salt from things every seat sees identically. */
function gameSalt(view: PlayerView): number {
  return hashString(`${view.width}x${view.height}|${view.players.map((p) => `${p.name}/${p.faction}`).join('|')}`);
}

function shuffled<T>(list: T[], salt: number): T[] {
  const a = [...list];
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(rnd(i, salt & 0xffff, salt >>> 16) * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

const isWater = (t: Terrain | null) => t === 'Ocean' || t === 'Coast';

export function computeLabels(view: PlayerView): MapLabel[] {
  const { width: W, height: H } = view;
  const salt = gameSalt(view);
  const cw = W >= 56 ? 12 : 12;
  const chh = 9;
  const nx = Math.ceil(W / cw);
  const ny = Math.ceil(H / chh);
  const seas = shuffled(SEAS, salt);
  const ranges = shuffled(RANGES, salt ^ 0x5bd1);
  const regions: Record<string, string[]> = {};
  for (const [k, v] of Object.entries(REGIONS)) regions[k] = shuffled(v, salt ^ hashString(k));
  const cityTiles = new Set(view.cities.map((c) => `${c.pos.q + (c.pos.r - (c.pos.r & 1)) / 2},${c.pos.r}`));
  const out: MapLabel[] = [];
  const near = (x: number, y: number, kind: MapLabel['kind'], d: number) =>
    out.some((l) => l.kind === kind && Math.hypot(l.x - x, l.y - y) < d);

  // Ranges: connected runs of explored mountains, lettered along their spine.
  const seen = new Uint8Array(W * H);
  for (let i = 0; i < W * H; i++) {
    if (seen[i] || view.tiles[i].terrain !== 'Mountains' || !view.tiles[i].explored) continue;
    const comp: number[] = [];
    const stack = [i];
    seen[i] = 1;
    while (stack.length) {
      const j = stack.pop()!;
      comp.push(j);
      for (let e = 0; e < 6; e++) {
        const nb = neighborIndex(j, e, W, H);
        if (nb < 0 || seen[nb] || !view.tiles[nb].explored || view.tiles[nb].terrain !== 'Mountains') continue;
        seen[nb] = 1;
        stack.push(nb);
      }
    }
    if (comp.length < 4) continue;
    let cx = 0;
    let cy = 0;
    for (const j of comp) {
      cx += centerX(j, W);
      cy += centerY(j, W);
    }
    cx /= comp.length;
    cy /= comp.length;
    let sxx = 0;
    let syy = 0;
    let sxy = 0;
    for (const j of comp) {
      const dx = centerX(j, W) - cx;
      const dy = centerY(j, W) - cy;
      sxx += dx * dx;
      syy += dy * dy;
      sxy += dx * dy;
    }
    let angle = 0.5 * Math.atan2(2 * sxy, sxx - syy);
    if (angle > Math.PI / 2) angle -= Math.PI;
    if (angle < -Math.PI / 2) angle += Math.PI;
    // Near-vertical ranges read better tilted than stacked.
    if (Math.abs(angle) > 1.2) angle = Math.sign(angle) * 1.2;
    const spread = Math.sqrt((sxx + syy) / comp.length);
    if (spread < 45) continue;
    const col = Math.round(cx / (36 * Math.sqrt(3)));
    const row = Math.round(cy / 54);
    let blocked = false;
    for (let dr = -2; dr <= 2 && !blocked; dr++)
      for (let dc = -2; dc <= 2; dc++) if (cityTiles.has(`${col + dc},${row + dr}`)) blocked = true;
    if (blocked || near(cx, cy, 'range', 360)) continue;
    const cell = Math.floor(row / chh) * nx + Math.floor(col / cw);
    out.push({ text: ranges[cell % ranges.length].toUpperCase(), kind: 'range', x: cx, y: cy, angle });
  }
  for (let gy = 0; gy < ny; gy++)
    for (let gx = 0; gx < nx; gx++) {
      const cell = gy * nx + gx;
      let total = 0;
      const water: number[] = [];
      const land: number[] = [];
      const mountains: number[] = [];
      const counts: Record<string, number> = {};
      for (let r = gy * chh; r < Math.min(H, (gy + 1) * chh); r++)
        for (let c = gx * cw; c < Math.min(W, (gx + 1) * cw); c++) {
          const i = r * W + c;
          total++;
          const t = view.tiles[i];
          if (!t.explored || !t.terrain) continue;
          if (isWater(t.terrain)) water.push(i);
          else {
            land.push(i);
            if (t.terrain === 'Mountains') mountains.push(i);
            const k = t.terrain === 'Grassland' || t.terrain === 'Plains' ? 'Open' : t.terrain;
            counts[k] = (counts[k] ?? 0) + 1;
          }
        }
      const centroid = (idx: number[]) => {
        let x = 0;
        let y = 0;
        for (const i of idx) {
          x += centerX(i, W);
          y += centerY(i, W);
        }
        return { x: x / idx.length, y: y / idx.length };
      };
      // Seas: a cell mostly of explored water. Anchor on the water tile nearest the centroid
      // that has the most water around it, so the name sits on open water.
      if (water.length >= 16 && water.length >= total * 0.45) {
        const c = centroid(water);
        const set = new Set(water);
        let best = water[0];
        let bs = -Infinity;
        for (const i of water) {
          let around = 0;
          const col = i % W;
          const row = (i - col) / W;
          for (let dr = -1; dr <= 1; dr++)
            for (let dc = -2; dc <= 2; dc++) if (set.has((row + dr) * W + col + dc)) around++;
          const score = around * 40 - Math.hypot(centerX(i, W) - c.x, centerY(i, W) - c.y);
          if (score > bs) {
            bs = score;
            best = i;
          }
        }
        const x = centerX(best, W);
        const y = centerY(best, W);
        if (!near(x, y, 'sea', 420)) out.push({ text: seas[cell % seas.length], kind: 'sea', x, y, angle: 0 });
      }
      // Regions: open land with no city in the way.
      if (land.length >= 30 && land.length >= total * 0.5) {
        const dominant = Object.entries(counts).sort((a, b) => b[1] - a[1])[0][0];
        const names = regions[dominant === 'Mountains' ? 'Hills' : dominant] ?? regions.Open;
        const mid = centroid(land);
        // Anchor on the explored land tile nearest the centroid (a ring of land has a hole).
        let anchor = land[0];
        let bd = Infinity;
        for (const i of land) {
          const d = Math.hypot(centerX(i, W) - mid.x, centerY(i, W) - mid.y);
          if (d < bd) {
            bd = d;
            anchor = i;
          }
        }
        const c = { x: centerX(anchor, W), y: centerY(anchor, W) };
        const col = anchor % W;
        const row = Math.floor(anchor / W);
        let blocked = false;
        for (let dr = -2; dr <= 2 && !blocked; dr++)
          for (let dc = -2; dc <= 2; dc++) if (cityTiles.has(`${col + dc},${row + dr}`)) blocked = true;
        const y = c.y + 20;
        if (!blocked && !near(c.x, y, 'land', 480) && !near(c.x, y, 'range', 320))
          out.push({ text: names[cell % names.length].toUpperCase(), kind: 'land', x: c.x, y, angle: 0 });
      }
    }
  return out;
}
