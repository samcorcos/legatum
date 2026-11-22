#!/usr/bin/env node
// Renders every Legatum icon, medallion and miniature set at the size it is actually drawn, one PNG per asset,
// for tools/art/distinctness.py to compare. Headless Chromium (Playwright from app/node_modules) draws each asset
// onto a canvas the way the game does: single-colour icons in ink, painted art stepped down in halves to the display
// size, roundels in one fixed owner colour. Output: <out>/manifest.json and <out>/<set>@<size>/<id>.png (RGBA).
//
//   node tools/art/render-sets.mjs [--out DIR]
import { createRequire } from 'node:module';
import { readFileSync, readdirSync, mkdirSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { dirname, join, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const app = join(root, 'app');
const out = process.argv.includes('--out') ? process.argv[process.argv.indexOf('--out') + 1] : join(root, 'tools', 'art', '.cache');
const { chromium } = createRequire(join(app, 'package.json'))('playwright');

const INK = '#2b2419'; // tokens.css --ink
const OWNER = '#4a74a8'; // one fixed owner colour: in play every unit on screen at once may be any seat's, the pair must differ in shape
const A = join(app, 'src', 'assets');
const ls = (dir) => (existsSync(dir) ? readdirSync(dir).sort() : []);
// Roundels are class marks: units of one class share a silhouette on purpose (the owner colour tells the empire, the
// shape the class), so identical roundels are one asset, named after all the units that wear it.
const dedupe = (items) => {
  const by = new Map();
  for (const i of items) {
    const key = readFileSync(i.file, 'utf8').replace(/aria-label="[^"]*"/, '');
    by.set(key, [...(by.get(key) ?? []), i]);
  }
  return [...by.values()].map((g) => ({ id: g.map((i) => i.id).join('+'), file: g[0].file }));
};
const pick = (dir, re) => ls(dir).filter((f) => re.test(f)).map((f) => ({ id: f.replace(re, ''), file: join(dir, f) }));

// kind: mono = single-colour silhouettes (compared on shape and tone only); colour = painted or multi-colour (colour counts too).
// crop: how much of the frame to compare (a medallion's brass ring is shared by every member, so only its inside is compared).
const SETS = [
  { name: 'resources', kind: 'colour', crop: 'inner', sizes: [14, 24, 32], items: pick(join(A, 'resources'), /-256\.png$/).map((i) => ({ ...i, id: i.id })) },
  { name: 'units-miniature', kind: 'colour', crop: 'full', sizes: [45], items: pick(join(A, 'units'), /-256\.png$/) },
  { name: 'units-roundel', kind: 'colour', crop: 'full', sizes: [16, 24], owner: true, focus: 'ink', items: dedupe(pick(join(A, 'units'), /-roundel\.svg$/)) },
  { name: 'unit-icons', kind: 'mono', crop: 'full', sizes: [16, 24], items: pick(join(A, 'icons'), /\.svg$/).filter((i) => i.id.startsWith('unit-')) },
  { name: 'building-icons', kind: 'mono', crop: 'full', sizes: [16, 24], items: pick(join(A, 'icons'), /\.svg$/).filter((i) => i.id.startsWith('building-')) },
  { name: 'improvement-icons', kind: 'mono', crop: 'full', sizes: [16, 24], items: pick(join(A, 'icons'), /\.svg$/).filter((i) => /^(improvement-|effect-)/.test(i.id)) },
  { name: 'yield-icons', kind: 'mono', crop: 'full', sizes: [16, 20], items: pick(join(A, 'icons'), /\.svg$/).filter((i) => /^(yield-|vp$)/.test(i.id)) },
  { name: 'ui-marks', kind: 'mono', crop: 'full', sizes: [16, 24], items: pick(join(A, 'icons'), /\.svg$/).filter((i) => /^(ui-|mark-|fortify$|capital$|city$|ruin$)/.test(i.id)) },
  { name: 'title-milestone-icons', kind: 'mono', crop: 'full', sizes: [24, 32], items: pick(join(A, 'icons'), /\.svg$/).filter((i) => /^(title-|milestone-)/.test(i.id)) },
  { name: 'faction-glyphs', kind: 'mono', crop: 'full', sizes: [16, 24], items: pick(join(A, 'heraldry'), /\.svg$/).filter((i) => i.id.startsWith('glyph-')) },
  { name: 'faction-seals', kind: 'mono', crop: 'full', sizes: [40, 56], items: pick(join(A, 'heraldry'), /\.svg$/).filter((i) => i.id.startsWith('seal-')) },
  { name: 'reward-emblems', kind: 'mono', crop: 'full', sizes: [24], items: pick(join(A, 'rewards'), /\.svg$/) },
  { name: 'city-art', kind: 'colour', crop: 'full', sizes: [48], items: pick(join(A, 'cities'), /\.png$/).filter((i) => /^[a-z]+-(hamlet|town|city|metropolis)$/.test(i.id)) },
  { name: 'city-marks', kind: 'colour', crop: 'full', sizes: [24], items: pick(join(A, 'cities', 'marks'), /-64\.png$/) },
  { name: 'map-features', kind: 'colour', crop: 'full', sizes: [32], items: [...pick(join(A, 'map-features'), /-128\.png$/)].filter((i) => !i.id.includes('remembered')) },
  { name: 'natural-wonders', kind: 'colour', crop: 'full', sizes: [32, 48], items: pick(join(app, 'public', 'wonders'), /-b-128\.png$/) },
  { name: 'portraits-people', kind: 'colour', crop: 'full', sizes: [48], items: pick(join(A, 'art'), /-360\.jpg$/).filter((i) => i.id.startsWith('people-')) },
  { name: 'era-art', kind: 'colour', crop: 'full', sizes: [64], items: pick(join(A, 'eras'), /-1280\.jpg$/) },
  { name: 'unlock-art', kind: 'colour', crop: 'full', sizes: [64], items: pick(join(A, 'unlocks'), /-64\.png$/) },
].filter((s) => s.items.length > 1);

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
const browser = await chromium.launch();
const page = await browser.newPage();
await page.setContent('<canvas id=c></canvas>');

const render = async (item, size, recolor) => {
  const raw = readFileSync(item.file);
  const ext = item.file.split('.').pop();
  const mime = ext === 'svg' ? 'image/svg+xml' : ext === 'jpg' ? 'image/jpeg' : 'image/png';
  let src;
  if (ext === 'svg') src = `data:${mime};charset=utf-8,${encodeURIComponent(raw.toString('utf8').replaceAll('currentColor', recolor))}`;
  else src = `data:${mime};base64,${raw.toString('base64')}`;
  return page.evaluate(async ({ src, size, vector }) => {
    const img = new Image();
    img.src = src;
    await img.decode();
    // Fit inside the square (era paintings are 16:9), centred.
    const nw = img.naturalWidth || size, nh = img.naturalHeight || size;
    const k = Math.min(size / nw, size / nh), w = Math.max(1, Math.round(nw * k)), h = Math.max(1, Math.round(nh * k));
    let cur = img, cw = nw, ch = nh;
    if (!vector) {
      // Step down in halves (the game caches a high-quality downscale; one big jump aliases).
      while (cw / 2 >= w * 1.5 && ch / 2 >= h * 1.5) {
        const c = document.createElement('canvas');
        c.width = Math.round(cw / 2); c.height = Math.round(ch / 2);
        const g = c.getContext('2d'); g.imageSmoothingQuality = 'high'; g.drawImage(cur, 0, 0, c.width, c.height);
        cur = c; cw = c.width; ch = c.height;
      }
    }
    const c = document.createElement('canvas');
    c.width = size; c.height = size;
    const g = c.getContext('2d'); g.imageSmoothingQuality = 'high';
    g.drawImage(cur, Math.floor((size - w) / 2), Math.floor((size - h) / 2), w, h);
    return c.toDataURL('image/png');
  }, { src, size, vector: ext === 'svg' });
};

const manifest = { generated: new Date().toISOString(), ink: INK, owner: OWNER, sets: {} };
for (const set of SETS) {
  const recolor = set.owner ? OWNER : INK;
  manifest.sets[set.name] = { kind: set.kind, crop: set.crop, focus: set.focus ?? null, sizes: set.sizes, ids: set.items.map((i) => i.id) };
  for (const size of set.sizes) {
    const dir = join(out, `${set.name}@${size}`);
    mkdirSync(dir, { recursive: true });
    for (const item of set.items) {
      const url = await render(item, size, recolor);
      writeFileSync(join(dir, `${item.id}.png`), Buffer.from(url.split(',')[1], 'base64'));
    }
  }
}
writeFileSync(join(out, 'manifest.json'), JSON.stringify(manifest, null, 1));
await browser.close();
console.log(`rendered ${Object.values(manifest.sets).reduce((n, s) => n + s.ids.length * s.sizes.length, 0)} images in ${Object.keys(manifest.sets).length} sets to ${out}`);
