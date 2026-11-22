#!/usr/bin/env node
// Keeps earlier builds' files on the website, so a game left open in a tab keeps its art after a deploy.
//
// Vite names every built file by its content (assets/<name>-<hash>.<ext>), and Cloudflare Pages serves only the files
// of the newest deployment. A tab opened before a deploy still runs the old code, which loads art, fonts and code
// chunks lazily by their old names; without this step those addresses answer 404 and the player sees broken images
// (2026-11-22: the WebP switch renamed every picture, and an open game lost its late-era art and city sprites).
//
// Usage: node tools/release/carry-assets.mjs <dist> [--from <site>] [--crawl <site>]... [--from-dir <dir>]... [--keep-days N]
//   --from <site>    the live site. Its asset-history.json lists every file it serves and the day each was first
//                    last used by a deployed build; each file used within --keep-days (default 30) that this build lacks is downloaded
//                    into <dist>/assets. A site with no history yet is crawled instead (see --crawl).
//   --crawl <site>   also carry every file a deployment's pages point to, found by reading its index.html, scripts
//                    and styles (for example an older deployment's own address, https://<id>.<project>.pages.dev).
//   --from-dir <dir> also copy the images and fonts of another local build (<dir>/assets), for a build that is gone
//                    from the site: content-hashed names make an older commit's build give the same files.
// It then writes <dist>/asset-history.json for the next deploy. Hashed names never collide, so a carried file can
// never replace one of this build's. Files are only added; nothing in the build changes.
import { readdirSync, writeFileSync, existsSync, mkdirSync, copyFileSync } from 'node:fs';
import { join, resolve, extname } from 'node:path';
import { refsIn, resolveRef } from './check-dist-refs-lib.mjs';

const DAY = 86_400_000;
const args = process.argv.slice(2);
const dist = resolve(args.shift() ?? '');
let from = null;
const crawls = [];
const dirs = [];
let keepDays = 30;
while (args.length) {
  const a = args.shift();
  if (a === '--from') from = args.shift();
  else if (a === '--crawl') crawls.push(args.shift());
  else if (a === '--from-dir') dirs.push(resolve(args.shift()));
  else if (a === '--keep-days') keepDays = Number(args.shift());
  else { console.error(`carry-assets: unknown argument ${a}`); process.exit(2); }
}
if (!existsSync(join(dist, 'index.html'))) { console.error(`carry-assets: ${dist} is not a built site`); process.exit(2); }
if (!Number.isFinite(keepDays) || keepDays < 1) { console.error('carry-assets: --keep-days must be 1 or more'); process.exit(2); }

const today = new Date().toISOString().slice(0, 10);
const site = (s) => s.replace(/\/+$/, '') + '/';
const ok = (name) => /^assets\/[A-Za-z0-9_.-]+$/.test(name);

async function get(url) {
  const res = await fetch(url, { redirect: 'follow' });
  // Pages answers an unknown address with 404.html; a hashed file is never HTML.
  if (!res.ok) return null;
  return res;
}

/** Every assets/ file a deployment's pages point to, by reading index.html and following scripts and styles. */
async function crawl(base) {
  const seen = new Set();
  const queue = ['index.html'];
  while (queue.length) {
    const path = queue.shift();
    const res = await get(base + path);
    if (!res) { if (path === 'index.html') throw new Error(`no index.html at ${base}`); continue; }
    const text = await res.text();
    const dir = path.includes('/') ? path.slice(0, path.lastIndexOf('/') + 1) : '';
    for (const raw of refsIn(text, extname(path))) {
      if (/^(?:[a-z][a-z0-9+.-]*:|\/\/|#|%)/i.test(raw)) continue;
      const ref = resolveRef(raw, dir);
      if (!ok(ref) || seen.has(ref)) continue;
      seen.add(ref);
      if (/\.(m?js|css)$/.test(ref)) queue.push(ref);
    }
  }
  return seen;
}

async function download(base, names) {
  let carried = 0;
  const failed = [];
  const list = [...names];
  await Promise.all(Array.from({ length: 16 }, async () => {
    while (list.length) {
      const name = list.shift();
      const target = join(dist, name);
      if (existsSync(target)) continue;
      const res = await get(base + name);
      const type = res?.headers.get('content-type') ?? '';
      if (!res || type.startsWith('text/html')) { failed.push(name); continue; }
      writeFileSync(target, Buffer.from(await res.arrayBuffer()));
      carried++;
    }
  }));
  return { carried, failed };
}

mkdirSync(join(dist, 'assets'), { recursive: true });
const ownFiles = readdirSync(join(dist, 'assets'));
/** assets/<file> -> the last day a deployed build used it. */
const history = new Map();

if (from) {
  const base = site(from);
  const res = await get(base + 'asset-history.json');
  let prior = null;
  if (res) {
    try { prior = (await res.json()).files ?? null; } catch { prior = null; }
  }
  if (prior) {
    const cutoff = Date.now() - keepDays * DAY;
    const keep = Object.entries(prior).filter(([n, d]) => ok(n) && Date.parse(d) >= cutoff && !ownFiles.includes(n.slice(7)));
    for (const [n, d] of keep) history.set(n, d);
    const { carried, failed } = await download(base, keep.map(([n]) => n));
    console.log(`carry-assets: ${carried} files carried from ${from} (${keep.length} used in the last ${keepDays} days)`);
    for (const n of failed) history.delete(n);
    if (failed.length) console.warn(`carry-assets: ${failed.length} listed files were not on ${from}, e.g. ${failed[0]}`);
  } else {
    console.log(`carry-assets: ${from} has no asset-history.json yet; carrying what its pages point to`);
    crawls.unshift(from);
  }
}

for (const c of crawls) {
  const base = site(c);
  let names;
  try {
    names = await crawl(base);
  } catch (e) {
    // A site that does not exist yet (a first deploy) has nothing to carry; any other failure stops the deploy.
    if (c === from && /no index\.html/.test(String(e))) { console.log(`carry-assets: nothing at ${c} yet`); continue; }
    throw e;
  }
  const { carried, failed } = await download(base, names);
  for (const n of names) if (!failed.includes(n) && !history.has(n)) history.set(n, today);
  console.log(`carry-assets: ${carried} files carried from ${c} (${names.size} referenced)`);
  if (failed.length) console.warn(`carry-assets: ${failed.length} referenced files were missing on ${c}, e.g. ${failed[0]}`);
}

for (const d of dirs) {
  let carried = 0;
  for (const f of readdirSync(join(d, 'assets'))) {
    if (!/\.(png|webp|jpe?g|svg|gif|woff2?|ttf|otf)$/i.test(f) || existsSync(join(dist, 'assets', f))) continue;
    copyFileSync(join(d, 'assets', f), join(dist, 'assets', f));
    carried++;
    if (!history.has(`assets/${f}`)) history.set(`assets/${f}`, today);
  }
  console.log(`carry-assets: ${carried} images and fonts carried from ${d}`);
}

// The date is the last day a deployed build itself used the file, so a file leaves the site 30 days after the
// last build that needed it, not 30 days after it first shipped. This build's own files are dated today; carried
// files keep their date. Read this build's files before anything is carried in.
for (const f of ownFiles) history.set(`assets/${f}`, today);
const files = Object.fromEntries([...history].filter(([n]) => existsSync(join(dist, n))).sort(([a], [b]) => (a < b ? -1 : 1)));
// Cloudflare Pages takes at most 20,000 files a deployment.
const total = Object.keys(files).length;
if (total > 18_000) { console.error(`carry-assets: ${total} files is near Cloudflare Pages' 20,000 limit; deploy with a smaller --keep-days`); process.exit(1); }
writeFileSync(join(dist, 'asset-history.json'), JSON.stringify({ note: 'Written by tools/release/carry-assets.mjs: every file this site serves under assets/ and the last day a deployed build used it.', files }, null, 1) + '\n');
console.log(`carry-assets: ${Object.keys(files).length} files in asset-history.json`);
