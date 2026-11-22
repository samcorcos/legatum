// Build the art-direction mockups: node tools/design/mockups/build/build.mjs [a b c ...]
// Each variant-<x>.mjs exports pages() -> { 'file.html': html }.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const out = path.resolve(here, '..');
const which = process.argv.slice(2);
const targets = which.length ? which : ['a', 'b', 'c'];
for (const t of targets) {
  const mod = await import(`./variant-${t}.mjs`);
  for (const [name, html] of Object.entries(mod.pages())) {
    fs.writeFileSync(path.join(out, name), html);
    console.log(`wrote ${name} (${(html.length / 1024).toFixed(0)} KB)`);
  }
}
