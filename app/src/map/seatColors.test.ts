import { describe, expect, it, vi } from 'vitest';
import { FACTION_PALETTE, setSeatFactions } from '../theme';
import type { Faction } from '../types';
import { ATLAS, mapBorderColor, mapPlayerColor, mapPlayerTint } from './style';
import { FAR_HEX_PX, HEX_SIZE, isFar, sealRadius } from './grid';
import { drawLabel } from './sprites';

const rgb = (hex: string) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
const hue = ([r, g, b]: number[]) => {
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min;
  if (!d) return 0;
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return (h * 60 + 360) % 360;
};
const chroma = ([r, g, b]: number[]) => Math.max(r, g, b) - Math.min(r, g, b);
const dist = (a: number[], b: number[]) => Math.hypot(...a.map((v, i) => v - b[i]));

const FACTIONS = Object.keys(FACTION_PALETTE) as Faction[];
// Seat i plays FACTIONS[i]: each faction's colour is looked up through the seat it sits in.
setSeatFactions(FACTIONS);

describe('faction colours on the map are tints', () => {
  const seats = FACTIONS.map((_, i) => mapPlayerColor(i));
  it('keeps every faction hue and settles the chroma below the UI colour', () => {
    seats.forEach((c, i) => {
      expect(c).toMatch(/^#[0-9a-f]{6}$/);
      const ui = rgb(FACTION_PALETTE[FACTIONS[i]].color), map = rgb(c);
      expect(chroma(map)).toBeLessThan(chroma(ui));
      // Neutral steel stays neutral; the rest keep their hue within a few degrees.
      if (chroma(ui) > 40) expect(Math.min(Math.abs(hue(ui) - hue(map)), 360 - Math.abs(hue(ui) - hue(map)))).toBeLessThan(14);
    });
  });
  it('keeps every faction distinguishable, as a rim on parchment and as a wash', () => {
    for (let i = 0; i < seats.length; i++) for (let j = i + 1; j < seats.length; j++) {
      expect(dist(rgb(seats[i]), rgb(seats[j])), `${FACTIONS[i]} vs ${FACTIONS[j]}`).toBeGreaterThanOrEqual(38);
      expect(dist(rgb(mapPlayerTint(i)), rgb(mapPlayerTint(j))), `tint ${FACTIONS[i]} vs ${FACTIONS[j]}`).toBeGreaterThanOrEqual(24);
    }
  });
  it('stays visible against the parchment plate', () => {
    for (const c of seats) expect(dist(rgb(c), rgb('#f7f1e3'))).toBeGreaterThanOrEqual(110);
  });
});

describe('the zoomed-out tier', () => {
  it('starts below the banner-with-meters zoom and keeps seals a legible size', () => {
    expect(isFar(HEX_SIZE * 0.9)).toBe(false);
    expect(isFar(HEX_SIZE * 0.45)).toBe(true);
    expect(isFar(FAR_HEX_PX)).toBe(false);
    for (const z of [0.28, 0.45, 0.6]) expect(sealRadius(HEX_SIZE * z)).toBeGreaterThanOrEqual(7);
  });
  it('measures a label without drawing it', () => {
    const draw = vi.fn();
    const g = { save: draw, restore: draw, translate: draw, rotate: draw, drawImage: draw } as unknown as CanvasRenderingContext2D;
    const canvas = { width: 0, height: 0, getContext: () => ({ scale: vi.fn(), measureText: () => ({ width: 6 }), fillText: vi.fn(), set font(_v: string) {}, set fillStyle(_v: string) {} }) };
    vi.stubGlobal('document', { createElement: () => canvas, fonts: undefined });
    const rect = drawLabel(g, 'atlas', 'sea', 'The Drowned Coast', '#000', 100, 100, 0, 1, 1, false);
    expect(draw).not.toHaveBeenCalled();
    expect(rect.w).toBeGreaterThan(40);
    vi.unstubAllGlobals();
  });
});

// Colour-vision-deficiency check: Machado et al. (2009) matrices at full severity, CIEDE2000 on the result.
const CVD = {
  normal: [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
  protan: [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]],
  deutan: [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.01182, 0.04294, 0.968881]],
  tritan: [[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.3039]],
};
const lin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
function labOf(hexColor: string, m: number[][]): [number, number, number] {
  const l = rgb(hexColor).map((v) => lin(v / 255));
  const [r, g, b] = m.map((row) => Math.min(1, Math.max(0, row[0] * l[0] + row[1] * l[1] + row[2] * l[2])));
  const f = (t: number) => (t > 216 / 24389 ? Math.cbrt(t) : ((24389 / 27) * t + 16) / 116);
  const fx = f((0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047), fy = f(0.2126 * r + 0.7152 * g + 0.0722 * b), fz = f((0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883);
  return [116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)];
}
function deltaE00([L1, a1, b1]: number[], [L2, a2, b2]: number[]): number {
  const rad = Math.PI / 180, deg = 180 / Math.PI;
  const C1 = Math.hypot(a1, b1), C2 = Math.hypot(a2, b2), Cb = (C1 + C2) / 2;
  const G = 0.5 * (1 - Math.sqrt(Cb ** 7 / (Cb ** 7 + 25 ** 7)));
  const ap1 = (1 + G) * a1, ap2 = (1 + G) * a2, Cp1 = Math.hypot(ap1, b1), Cp2 = Math.hypot(ap2, b2);
  const hp = (b: number, a: number) => { if (!b && !a) return 0; const h = Math.atan2(b, a) * deg; return h < 0 ? h + 360 : h; };
  const h1 = hp(b1, ap1), h2 = hp(b2, ap2);
  const dh = Cp1 * Cp2 === 0 ? 0 : Math.abs(h2 - h1) <= 180 ? h2 - h1 : h2 - h1 > 180 ? h2 - h1 - 360 : h2 - h1 + 360;
  const dL = L2 - L1, dC = Cp2 - Cp1, dH = 2 * Math.sqrt(Cp1 * Cp2) * Math.sin((dh * rad) / 2);
  const Lb = (L1 + L2) / 2, Cpb = (Cp1 + Cp2) / 2;
  const hb = Cp1 * Cp2 === 0 ? h1 + h2 : Math.abs(h1 - h2) <= 180 ? (h1 + h2) / 2 : h1 + h2 < 360 ? (h1 + h2 + 360) / 2 : (h1 + h2 - 360) / 2;
  const T = 1 - 0.17 * Math.cos((hb - 30) * rad) + 0.24 * Math.cos(2 * hb * rad) + 0.32 * Math.cos((3 * hb + 6) * rad) - 0.2 * Math.cos((4 * hb - 63) * rad);
  const dTh = 30 * Math.exp(-(((hb - 275) / 25) ** 2));
  const Rc = 2 * Math.sqrt(Cpb ** 7 / (Cpb ** 7 + 25 ** 7));
  const Sl = 1 + (0.015 * (Lb - 50) ** 2) / Math.sqrt(20 + (Lb - 50) ** 2), Sc = 1 + 0.045 * Cpb, Sh = 1 + 0.015 * Cpb * T;
  const Rt = -Math.sin(2 * dTh * rad) * Rc;
  return Math.sqrt((dL / Sl) ** 2 + (dC / Sc) ** 2 + (dH / Sh) ** 2 + Rt * (dC / Sc) * (dH / Sh));
}
const hueDiff = (a: string, b: string) => { const d = Math.abs(hue(rgb(a)) - hue(rgb(b))); return Math.min(d, 360 - d); };

describe('border colours (the bold looks)', () => {
  const seats = FACTIONS.map((_, i) => mapBorderColor(i));
  it('keeps each faction in its own hue family', () => {
    seats.forEach((c, i) => {
      const ui = FACTION_PALETTE[FACTIONS[i]].color;
      expect(c).toBe(ATLAS.seat.border[FACTIONS[i]]);
      // Steel stays neutral; every other faction is within 25 degrees of its UI colour.
      if (chroma(rgb(ui)) > 40) expect(hueDiff(ui, c), FACTIONS[i]).toBeLessThan(25);
      else expect(chroma(rgb(c))).toBeLessThan(20);
    });
  });
  it('keeps every pair of factions apart for normal, protan, deutan and tritan sight', () => {
    for (const [name, m] of Object.entries(CVD)) {
      const labs = seats.map((c) => labOf(c, m));
      for (let i = 0; i < seats.length; i++) for (let j = i + 1; j < seats.length; j++)
        expect(deltaE00(labs[i], labs[j]), `${name}: ${FACTIONS[i]} vs ${FACTIONS[j]}`).toBeGreaterThanOrEqual(10);
    }
  });
  it('stays visible against the parchment and the ink edge', () => {
    const labs = seats.map((c) => labOf(c, CVD.normal));
    for (const l of labs) {
      expect(deltaE00(l, labOf(ATLAS.seat.paper, CVD.normal))).toBeGreaterThanOrEqual(22);
      expect(deltaE00(l, labOf(ATLAS.tone.ink, CVD.normal))).toBeGreaterThanOrEqual(20);
    }
  });
});
