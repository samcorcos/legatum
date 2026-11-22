#!/usr/bin/env node
// Writes THIRD_PARTY_LICENSES.txt: the licence notices a distributed build of Legatum must carry.
//
//   node tools/release/third-party-licenses.mjs [--out FILE]
//
// Covers what the browser build ships: the crates compiled into the WebAssembly engine, the production
// npm packages and the two bundled fonts.
// --out  default app/public/THIRD_PARTY_LICENSES.txt (git-ignored). Vite copies app/public into app/dist,
//        so the file is served at /THIRD_PARTY_LICENSES.txt and the in-game Credits can read it.
//
// Sources: `cargo metadata` (the locked dependency graph, normal and build dependencies, no dev
// dependencies) with the licence files each crate ships in the cargo registry, and app/package-lock.json
// with the licence files in app/node_modules. A crate that ships no licence file gets the standard text
// of its SPDX licence from tools/release/license-texts/. The script fails, naming the crate, when it can
// find neither: add the standard text there (from https://spdx.org/licenses/) and run it again.
// Needs cargo (with network access to fetch crate sources once) and `npm ci` in app/.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const TEXTS = path.join(ROOT, 'tools/release/license-texts');
const LICENCE_FILE = /^(licen[sc]e|copying|notice|unlicense|copyright)/i;

function args() {
  const a = process.argv.slice(2);
  const opt = { out: path.join(ROOT, 'app/public/THIRD_PARTY_LICENSES.txt') };
  for (let i = 0; i < a.length; i++) {
    if (a[i] === '--out') opt.out = path.resolve(a[++i]);
    else throw new Error(`unknown argument ${a[i]}`);
  }
  return opt;
}

/** Third-party crates reachable from `rootName` through normal and build dependencies on `target`. */
function crates(manifest, rootName, target) {
  const json = execFileSync('cargo', ['metadata', '--format-version', '1', '--locked', '--manifest-path', manifest, '--filter-platform', target],
    { cwd: ROOT, maxBuffer: 256 * 1024 * 1024, stdio: ['ignore', 'pipe', 'inherit'] });
  const meta = JSON.parse(json);
  const packages = new Map(meta.packages.map((p) => [p.id, p]));
  const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
  const root = meta.packages.find((p) => p.name === rootName);
  if (!root) throw new Error(`${rootName} is not in ${manifest}`);
  const seen = new Set();
  const todo = [root.id];
  while (todo.length) {
    const id = todo.pop();
    if (seen.has(id)) continue;
    seen.add(id);
    for (const dep of nodes.get(id).deps) if (dep.dep_kinds.some((k) => k.kind !== 'dev')) todo.push(dep.pkg);
  }
  // `source` is null for the project's own crates.
  return [...seen].map((id) => packages.get(id)).filter((p) => p.source);
}

/** The SPDX identifiers in an expression such as "(MIT OR Apache-2.0) AND Unicode-3.0". */
function spdxIds(expression) {
  return [...new Set((expression ?? '').replace(/[()]/g, ' ').replace(/\//g, ' OR ').split(/\s+(?:OR|AND|WITH)\s+|\s+/i).filter(Boolean))];
}

function standardText(id) {
  const file = path.join(TEXTS, `${id}.txt`);
  return fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : null;
}

/** [{ label, text }] for one package directory: the licence files it ships, else standard texts. */
function licenceTexts(dir, name, licence, declaredFile, authors) {
  const found = fs.readdirSync(dir).filter((f) => LICENCE_FILE.test(f) && fs.statSync(path.join(dir, f)).isFile()).sort();
  if (declaredFile && !found.includes(declaredFile) && fs.existsSync(path.join(dir, declaredFile))) found.push(declaredFile);
  if (found.length) return found.map((f) => ({ label: f, text: fs.readFileSync(path.join(dir, f), 'utf8') }));
  const ids = spdxIds(licence);
  // A standard text has no copyright line of its own, so the crate's authors are named beside it.
  const by = authors?.length ? `; copyright the ${name.split(' ')[0]} authors: ${authors.join(', ')}` : `; copyright the ${name.split(' ')[0]} authors`;
  const texts = ids.map((id) => ({ label: `standard ${id} text${by}`, text: standardText(id) })).filter((t) => t.text);
  if (!texts.length) throw new Error(`${name} (${licence || 'no licence field'}) ships no licence file and tools/release/license-texts/ has no text for ${ids.join(', ') || 'it'}`);
  return texts;
}

/** Production npm packages from the lockfile (everything not marked dev). */
function npmPackages() {
  const lock = JSON.parse(fs.readFileSync(path.join(ROOT, 'app/package-lock.json'), 'utf8'));
  const out = [];
  for (const [key, entry] of Object.entries(lock.packages)) {
    if (!key || entry.dev || entry.devOptional || entry.optional && !fs.existsSync(path.join(ROOT, 'app', key))) continue;
    const dir = path.join(ROOT, 'app', key);
    if (!fs.existsSync(dir)) throw new Error(`${key} is not installed: run npm ci in app/ first`);
    const pkg = JSON.parse(fs.readFileSync(path.join(dir, 'package.json'), 'utf8'));
    out.push({ name: pkg.name, version: pkg.version, licence: typeof pkg.license === 'string' ? pkg.license : pkg.license?.type ?? '', dir });
  }
  return out;
}

const normalise = (text) => text.replace(/\r\n/g, '\n').replace(/[ \t]+$/gm, '').trim();

function main() {
  const opt = args();
  const components = new Map(); // "kind name version" -> component
  const add = (kind, name, version, licence, dir, file, authors) => components.set(`${kind} ${name} ${version}`, { kind, name, version, licence, dir, file, authors });
  for (const p of crates('Cargo.toml', 'legatum-wasm', 'wasm32-unknown-unknown')) add('crate', p.name, p.version, p.license, path.dirname(p.manifest_path), p.license_file, p.authors);
  for (const p of npmPackages()) add('npm', p.name, p.version, p.licence, p.dir, null);

  const list = [...components.values()].sort((a, b) => a.kind.localeCompare(b.kind) || a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
  // Identical texts (the Apache licence, mostly) are printed once, with everything that uses them.
  const texts = new Map(); // hash -> { text, users: [] }
  const missing = [];
  for (const c of list) {
    let found = [];
    try {
      found = licenceTexts(c.dir, `${c.name} ${c.version}`, c.licence, c.file, c.authors);
    } catch (error) {
      missing.push(error.message);
    }
    for (const t of found) {
      const text = normalise(t.text);
      const hash = createHash('sha256').update(text).digest('hex');
      if (!texts.has(hash)) texts.set(hash, { text, users: [] });
      texts.get(hash).users.push(`${c.name} ${c.version} (${t.label})`);
    }
  }
  if (missing.length) throw new Error(`no licence text for ${missing.length} component(s):\n  ${missing.join('\n  ')}`);

  const rule ='='.repeat(100);
  const lines = [];
  lines.push('Legatum: third-party licence notices', '');
  lines.push(`This web build includes the open-source components listed below, each under its own licence.`);
  lines.push("Legatum's own code is MIT licensed and its original artwork is CC BY 4.0: https://github.com/samcorcos/legatum");
  lines.push('Music and sound effects are CC0 1.0; their credits are in the game under How to Play, Credits.', '');
  const mpl = list.filter((c) => c.kind === 'crate' && spdxIds(c.licence).includes('MPL-2.0') && !/\bOR\b/i.test(c.licence ?? ''));
  if (mpl.length) {
    lines.push('Mozilla Public License 2.0', '');
    lines.push(`These crates are licensed under MPL-2.0 and are used unmodified. Their source code is available from crates.io at exactly these versions: ${mpl.map((c) => `${c.name} ${c.version} (https://crates.io/crates/${c.name}/${c.version})`).join(', ')}.`, '');
  }
  const fonts = list.filter((c) => c.kind === 'npm' && c.name.startsWith('@fontsource/'));
  if (fonts.length) {
    lines.push('Fonts', '');
    lines.push(`The interface bundles the typefaces ${fonts.map((c) => c.name.replace('@fontsource/', '').split('-').map((w) => w[0].toUpperCase() + w.slice(1)).join(' ')).join(' and ')} under the SIL Open Font License 1.1, through the npm packages ${fonts.map((c) => `${c.name} ${c.version}`).join(' and ')}. Their copyright lines and the full licence text are below.`, '');
  }
  lines.push('Components', '');
  for (const kind of ['crate', 'npm']) {
    const of = list.filter((c) => c.kind === kind);
    lines.push(`${kind === 'crate' ? 'Rust crates' : 'npm packages'} (${of.length}):`);
    for (const c of of) lines.push(`  ${c.name} ${c.version}  ${c.licence || 'see licence text'}`);
    lines.push('');
  }
  lines.push('Licence texts', '');
  for (const { text, users } of [...texts.values()].sort((a, b) => b.users.length - a.users.length || a.users[0].localeCompare(b.users[0]))) {
    lines.push(rule, `Used by: ${users.join(', ')}`, rule, '', text, '');
  }
  fs.mkdirSync(path.dirname(opt.out), { recursive: true });
  fs.writeFileSync(opt.out, lines.join('\n'));
  const crateCount = list.filter((c) => c.kind === 'crate').length;
  console.log(`wrote ${path.relative(ROOT, opt.out)} (${crateCount} crates, ${list.length - crateCount} npm packages, ${texts.size} licence texts, ${Math.round(fs.statSync(opt.out).size / 1024)} KB)`);
}

try {
  main();
} catch (error) {
  console.error(`third-party-licenses: ${error.message}`);
  process.exit(1);
}
