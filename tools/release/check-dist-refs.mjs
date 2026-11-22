#!/usr/bin/env node
// Checks that every file the built web app points to is in the build: art, fonts, scripts, styles and the engine.
// Usage: node tools/release/check-dist-refs.mjs [app/dist]
//
// It reads every .js, .css and .html file under the folder and resolves, relative to that file:
//   - JS:   new URL(`name`, import.meta.url), import("./x.js") / from "./x.js", and "assets/x" (Vite's preload list)
//   - CSS:  url(name)
//   - HTML: src="..." and href="..." (local paths only)
// A reference that names no file exits 1 with the list. Run by scripts/build-web.sh after the web build (so the
// WebP rename in app/vite-plugin-webp.ts is covered) and by CI after the app build.
import { readdirSync, readFileSync, existsSync, statSync } from 'node:fs';
import { join, dirname, resolve, relative, extname } from 'node:path';
import { refsIn, resolveRef } from './check-dist-refs-lib.mjs';

const root = resolve(process.argv[2] ?? 'app/dist');
if (!existsSync(root) || !statSync(root).isDirectory()) {
  console.error(`check-dist-refs: no build folder at ${root}`);
  process.exit(2);
}

function* walk(dir) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) yield* walk(p);
    else yield p;
  }
}

const isLocal = (ref) => ref && !/^(?:[a-z][a-z0-9+.-]*:|\/\/|#|%)/i.test(ref);

let checked = 0;
const missing = [];
for (const file of walk(root)) {
  const ext = extname(file);
  if (!['.js', '.mjs', '.css', '.html'].includes(ext)) continue;
  const text = readFileSync(file, 'utf8');
  for (const raw of refsIn(text, ext)) {
    if (!isLocal(raw)) continue;
    if (!raw.split(/[?#]/)[0]) continue;
    // Relative to the file that names it; "assets/..." and "/..." are from the site root.
    const dir = relative(root, dirname(file));
    const target = join(root, resolveRef(raw, dir ? dir + '/' : ''));
    checked++;
    // A folder (the site root "/") is served by its index.html.
    const served = existsSync(target) && statSync(target).isDirectory() ? join(target, 'index.html') : target;
    if (!existsSync(served)) missing.push(`${relative(root, file)} -> ${raw}`);
  }
}

if (missing.length) {
  console.error(`check-dist-refs: ${missing.length} of ${checked} references in ${root} name no file:`);
  for (const m of [...new Set(missing)].slice(0, 50)) console.error(`  ${m}`);
  if (missing.length > 50) console.error(`  ... and ${missing.length - 50} more`);
  process.exit(1);
}
if (checked === 0) {
  console.error(`check-dist-refs: found no references in ${root}; the patterns no longer match the build`);
  process.exit(1);
}
console.log(`check-dist-refs: all ${checked} references in ${relative(process.cwd(), root) || '.'} resolve`);
