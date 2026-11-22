#!/usr/bin/env node
// Keeps earlier builds of the game playable on the website, so a game always finishes on the version it started on.
//
// Each deploy publishes its build twice: at the site root (the newest version, where new games start) and at
// v/<id>/ (a copy of its index.html with <base href="../../">, so its scripts, art and public files load from the
// root). Every build still listed in versions.json keeps its v/<id>/ page. Its content-named files stay in assets/
// through carry-assets.mjs, which keeps them for the same number of days. The game reads versions.json
// (app/src/platform/versions.ts) to open an older save in the build that made it, and to send a player whose
// tab is behind to the newest version between games.
//
// Usage: node tools/release/keep-builds.mjs <dist> --id <id> --rules <n> --protocol <n> --label <text>
//          [--commit <sha>] [--from <site>] [--keep-days N] [--legacy <deployment-url>=<id>:<rules>]...
//   --id         this build's id (scripts/build-web.sh: the commit, with a suffix for uncommitted changes)
//   --from       the live site: its versions.json lists the builds to keep, and their pages are copied from it
//   --keep-days  how long a build stays after a newer one replaced it (default 30, carry-assets.mjs's default)
//   --legacy     adds a build from before versions.json existed, from its Cloudflare Pages deployment address;
//                pass the same address to carry-assets.mjs --crawl so its files are kept too
// Writes <dist>/v/<id>/index.html for every kept build and <dist>/versions.json. Files the root had in an earlier
// build (terrain, wonders, audio) that this one dropped are copied back from the site, so older builds still find them.
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';

const DAY = 86_400_000;
const args = process.argv.slice(2);
const dist = resolve(args.shift() ?? '');
const opt = { keepDays: 30, legacy: [] };
while (args.length) {
  const a = args.shift();
  const v = args.shift();
  if (v === undefined) { console.error(`keep-builds: ${a} needs a value`); process.exit(2); }
  if (a === '--id') opt.id = v;
  else if (a === '--rules') opt.rules = Number(v);
  else if (a === '--protocol') opt.protocol = Number(v);
  else if (a === '--label') opt.label = v;
  else if (a === '--commit') opt.commit = v;
  else if (a === '--from') opt.from = v;
  else if (a === '--keep-days') opt.keepDays = Number(v);
  else if (a === '--legacy') opt.legacy.push(v);
  else { console.error(`keep-builds: unknown argument ${a}`); process.exit(2); }
}
const ID = /^[a-z0-9][a-z0-9-]{0,39}$/;
if (!existsSync(join(dist, 'index.html'))) { console.error(`keep-builds: ${dist} is not a built site`); process.exit(2); }
if (!ID.test(opt.id ?? '')) { console.error('keep-builds: --id must be lower-case letters, digits and hyphens'); process.exit(2); }
if (!Number.isInteger(opt.rules) || !Number.isInteger(opt.protocol)) { console.error('keep-builds: --rules and --protocol must be whole numbers'); process.exit(2); }
if (!Number.isFinite(opt.keepDays) || opt.keepDays < 1) { console.error('keep-builds: --keep-days must be 1 or more'); process.exit(2); }

const now = new Date().toISOString();
const cutoff = Date.now() - opt.keepDays * DAY;
const site = (s) => s.replace(/\/+$/, '') + '/';
/** Root files that are not one build's own: never copied back from an older build. */
const SITE_FILES = new Set(['index.html', '_headers', '_redirects', '404.html', 'versions.json', 'asset-history.json']);

/** The page of a build served from v/<id>/: everything relative resolves from the site root. */
function pinnedPage(html) {
  if (/<base\s/i.test(html)) return html;
  const at = html.search(/<head[^>]*>/i);
  if (at < 0) throw new Error('index.html has no <head>');
  const end = html.indexOf('>', at) + 1;
  return `${html.slice(0, end)}\n    <base href="../../" />${html.slice(end)}`;
}

async function get(url) {
  const res = await fetch(url, { redirect: 'follow', cache: 'no-store' });
  return res.ok ? res : null;
}

function writePage(id, html) {
  const file = join(dist, 'v', id, 'index.html');
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, pinnedPage(html));
}

function* walk(dir) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) yield* walk(p);
    else yield p;
  }
}

/** The root's own files: everything but the content-named assets, the kept builds' pages and the site files. */
function publicFiles() {
  const out = [];
  for (const f of walk(dist)) {
    const rel = relative(dist, f).split('\\').join('/');
    if (rel.startsWith('assets/') || rel.startsWith('v/') || SITE_FILES.has(rel)) continue;
    out.push(rel);
  }
  return out.sort();
}

// This build.
writePage(opt.id, readFileSync(join(dist, 'index.html'), 'utf8'));
const builds = [{ id: opt.id, label: opt.label ?? '', rules: opt.rules, protocol: opt.protocol, commit: opt.commit ?? '', deployedAt: now, retiredAt: null }];
const ownPublic = publicFiles();
let publicKept = new Set(ownPublic);

// The builds the live site still lists, and the root files they may need.
if (opt.from) {
  const base = site(opt.from);
  const res = await get(base + 'versions.json');
  let prior = null;
  if (res) { try { prior = await res.json(); } catch { prior = null; } }
  if (!prior || !Array.isArray(prior.builds)) {
    console.log(`keep-builds: ${opt.from} has no versions.json yet; this is the first build it lists`);
  } else {
    let kept = 0;
    let dropped = 0;
    for (const b of prior.builds) {
      if (!b || !ID.test(b.id ?? '') || b.id === opt.id || builds.some((x) => x.id === b.id)) continue;
      const retiredAt = b.retiredAt ?? now;
      if (Date.parse(retiredAt) < cutoff) { dropped++; continue; }
      const page = await get(`${base}v/${b.id}/index.html`);
      const html = page && (await page.text());
      if (!html || !/<html/i.test(html) || /There is nothing at this address/.test(html)) { console.warn(`keep-builds: v/${b.id}/ is not on ${opt.from}; dropping it`); continue; }
      writePage(b.id, html);
      builds.push({ id: b.id, label: String(b.label ?? ''), rules: Number(b.rules), protocol: Number(b.protocol), commit: String(b.commit ?? ''), deployedAt: String(b.deployedAt ?? now), retiredAt });
      kept++;
    }
    console.log(`keep-builds: kept ${kept} earlier builds from ${opt.from}${dropped ? `, dropped ${dropped} retired more than ${opt.keepDays} days ago` : ''}`);
    let carried = 0;
    for (const f of Array.isArray(prior.publicFiles) ? prior.publicFiles : []) {
      if (typeof f !== 'string' || !/^[A-Za-z0-9_./-]+$/.test(f) || f.includes('..') || publicKept.has(f) || SITE_FILES.has(f)) continue;
      if (f.startsWith('assets/') || f.startsWith('v/')) continue;
      const got = await get(base + f);
      const type = got?.headers.get('content-type') ?? '';
      if (!got || type.startsWith('text/html')) continue;
      mkdirSync(dirname(join(dist, f)), { recursive: true });
      writeFileSync(join(dist, f), Buffer.from(await got.arrayBuffer()));
      publicKept.add(f);
      carried++;
    }
    if (carried) console.log(`keep-builds: ${carried} root files an earlier build used were copied back from ${opt.from}`);
  }
}

// Builds from before this file existed, by their Pages deployment address.
for (const spec of opt.legacy) {
  const m = /^(https:\/\/[^=\s]+)=([a-z0-9][a-z0-9-]*):(\d+)$/.exec(spec);
  if (!m || !ID.test(m[2])) { console.error(`keep-builds: --legacy wants <https-url>=<id>:<rules>, got ${spec}`); process.exit(2); }
  const [, url, id, rules] = m;
  if (builds.some((b) => b.id === id)) continue;
  const page = await get(site(url) + 'index.html');
  if (!page) { console.error(`keep-builds: no index.html at ${url}`); process.exit(1); }
  writePage(id, await page.text());
  builds.push({ id, label: '', rules: Number(rules), protocol: 1, commit: '', deployedAt: now, retiredAt: now });
  console.log(`keep-builds: added the earlier build ${id} (rules ${rules}) from ${url}`);
}

const [current, ...older] = builds;
older.sort((a, b) => Date.parse(b.deployedAt) - Date.parse(a.deployedAt));
writeFileSync(join(dist, 'versions.json'), JSON.stringify({
  note: 'Written by tools/release/keep-builds.mjs: the builds this site serves, newest first. The first is the root; each is also at v/<id>/.',
  builds: [current, ...older],
  publicFiles: [...publicKept].filter((f) => existsSync(join(dist, f)) && statSync(join(dist, f)).isFile()).sort(),
}, null, 1) + '\n');
console.log(`keep-builds: versions.json lists ${builds.length} build${builds.length === 1 ? '' : 's'} (newest ${opt.id}, rules ${opt.rules})`);
