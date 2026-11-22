import { expect, it } from 'vitest';
import { readdirSync } from 'node:fs';
import { drawSite, drawWallsSite, wallsSiteStem } from './constructionFx';
import { wallStem } from './cityArt';
import type { ConstructionSite } from './constructionSites';

const recorder = () => {
  const out: string[] = [];
  const ctx = new Proxy({} as CanvasRenderingContext2D, {
    get: (_target, key) => (...args: unknown[]) => { out.push(`${String(key)}:${JSON.stringify(args)}`); },
    set: (_target, key, value) => { out.push(`${String(key)}=${String(value)}`); return true; },
  });
  return { ctx, out };
};
const site = (walls: boolean): ConstructionSite => ({ city: 1, owner: 0, tile: 1, hex: { q: 0, r: 0 }, building: walls ? 'Walls' : 'GreatLibrary', era: 1,
  k: .32, stage: 1, seed: 27, walls });
// A canvas line drawing of an object is a flat glyph beside painted art (legatum-ui, 2026-11-01).
const LINE_CALLS = /^(stroke|strokeRect|lineTo|moveTo|arc|ellipse|setLineDash|fillRect|fillText):/;

it('draws the same site from the same production progress', () => {
  const calls = () => { const r = recorder(); drawSite(r.ctx, site(false), { x: 0, y: 0, s: 100 }); return r.out; };
  expect(calls()).toEqual(calls());
});

it('draws no canvas line art for a wonder site at any stage', () => {
  for (const k of [0, .05, .2, .5, .8, .99, 1]) {
    const r = recorder();
    drawSite(r.ctx, site(false), { x: 0, y: 0, s: 100 }, k);
    expect(r.out.filter(c => LINE_CALLS.test(c)), `k=${k}`).toEqual([]);
  }
});

it('draws no canvas line art for a Walls site', () => {
  for (const k of [0, .3, .7, 1]) {
    const r = recorder();
    drawWallsSite(r.ctx, site(true), { x: 50, y: 50 }, 1, 1, k);
    expect(r.out.filter(c => /^(stroke|strokeRect|lineTo|setLineDash|fillRect):/.test(c)), `k=${k}`).toEqual([]);
  }
});

it('has the painted Walls footings for every era with a finished wall overlay, and nothing later', () => {
  const dir = (d: string) => new URL(`../assets/${d}/`, import.meta.url);
  const eras = readdirSync(dir('cities')).flatMap((f) => /^([a-z]+)-walls\.png$/.exec(f)?.[1] ?? []);
  expect(eras.length).toBeGreaterThan(0);
  const stages = new Set(readdirSync(dir('construction')));
  for (const era of eras) {
    expect(stages.has(`${era}-walls-stage1.png`), `${era} stage 1`).toBe(true);
    // Later stages looked finished on the map (Sam, 2026-11-22): the footings stand for the whole build.
    for (const n of [2, 3]) expect(stages.has(`${era}-walls-stage${n}.png`), `${era} stage ${n}`).toBe(false);
  }
});


it('uses the Renaissance star fort for Swiss Walls at every construction stage', () => {
  for (const era of [0, 1, 2, 3, 4, 5, 6]) {
    expect(wallsSiteStem({ ...site(true), era }, 'Switzerland')).toBe('renaissance-walls');
    expect(wallsSiteStem({ ...site(true), era }, 'Athens')).toBe(wallStem(era, false));
  }
});
