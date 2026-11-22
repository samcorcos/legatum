#!/usr/bin/env node
// Renders the grand crests from the seals: app/src/assets/heraldry/raster/crest-<faction>-512.png
//
//   node app/src/assets/heraldry/src/build-seals.mjs      (first, if a charge changed)
//   node app/src/assets/heraldry/src/render-crests.mjs [faction]
//
// A crest is the faction's seal struck in gold on navy: the seal SVG is lit as a bas-relief by an SVG
// lighting filter (its ink is the height map) over one shared ground, so all crests have the same ring,
// ground, light and margin by construction. No painting and no prompt: change a charge in
// build-seals.mjs and run both scripts.
//
// Needs Playwright's Chromium (app/node_modules, or the folder named by LEGATUM_ART_NODE_MODULES).
import { createRequire } from 'node:module';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = fileURLToPath(new URL('..', import.meta.url));
const APP = join(HERE, '..', '..', '..');
const modules = process.env.LEGATUM_ART_NODE_MODULES;
const { chromium } = createRequire(modules ? join(modules, 'x.js') : join(APP, 'package.json'))('playwright');

const SIZE = 512;
// The roundel that shows a crest crops it to a circle at 104% (styles.css `.crest img`), so the rim and
// its crowning star must stay inside 96% of the half-width.
const SCALE = 0.925;
const GOLD = '#d9ad55';

function crestSvg(seal) {
  const inner = seal.replace(/^<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '');
  const o = (SIZE * (1 - SCALE)) / 2;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${SIZE}" height="${SIZE}" viewBox="0 0 512 512">
<defs>
  <radialGradient id="ground" cx="50%" cy="46%" r="70%">
    <stop offset="0" stop-color="#1b2c50"/><stop offset=".5" stop-color="#111c36"/><stop offset="1" stop-color="#060a15"/>
  </radialGradient>
  <radialGradient id="field" cx="42%" cy="36%" r="75%">
    <stop offset="0" stop-color="#22355e"/><stop offset=".7" stop-color="#101a33"/><stop offset="1" stop-color="#0a1226"/>
  </radialGradient>
  <filter id="relief" x="-4%" y="-4%" width="108%" height="108%" color-interpolation-filters="sRGB">
    <feGaussianBlur in="SourceAlpha" stdDeviation="1.25" result="height"/>
    <feDiffuseLighting in="height" surfaceScale="4.6" diffuseConstant="1.18" lighting-color="#fff0c4" result="lit">
      <feDistantLight azimuth="232" elevation="48"/>
    </feDiffuseLighting>
    <feSpecularLighting in="height" surfaceScale="4.6" specularConstant=".55" specularExponent="18" lighting-color="#fff8e0" result="shine">
      <feDistantLight azimuth="232" elevation="48"/>
    </feSpecularLighting>
    <feComposite in="lit" in2="SourceGraphic" operator="arithmetic" k1="1.02" k2="0" k3=".06" k4="0" result="gold"/>
    <feComposite in="shine" in2="gold" operator="arithmetic" k1="0" k2=".8" k3="1" k4="0" result="struck"/>
    <feComposite in="struck" in2="SourceAlpha" operator="in" result="metal"/>
    <feGaussianBlur in="SourceAlpha" stdDeviation="3.2" result="soft"/>
    <feOffset in="soft" dx="2.5" dy="3.5" result="cast"/>
    <feComponentTransfer in="cast" result="shadow"><feFuncA type="linear" slope=".7"/><feFuncR type="linear" slope="0"/><feFuncG type="linear" slope="0"/><feFuncB type="linear" slope="0"/></feComponentTransfer>
    <feMerge><feMergeNode in="shadow"/><feMergeNode in="metal"/></feMerge>
  </filter>
</defs>
<rect width="512" height="512" fill="url(#ground)"/>
<g transform="translate(${o} ${o}) scale(${SCALE})">
  <circle cx="256" cy="256" r="230" fill="url(#field)"/>
  <g filter="url(#relief)" color="${GOLD}" fill="currentColor" fill-rule="evenodd">
    <circle cx="256" cy="256" r="226.5" fill="none" stroke="currentColor" stroke-width="6"/>${inner}
  </g>
</g>
</svg>`;
}

const only = process.argv[2];
const ids = readdirSync(HERE)
  .filter((n) => /^seal-.*\.svg$/.test(n))
  .map((n) => n.slice(5, -4))
  .filter((id) => !only || id === only);
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: SIZE, height: SIZE }, deviceScaleFactor: 1 });
for (const id of ids) {
  await page.setContent(`<body style="margin:0;background:#060a15">${crestSvg(readFileSync(join(HERE, `seal-${id}.svg`), 'utf8'))}</body>`);
  await page.screenshot({ path: join(HERE, 'raster', `crest-${id}-512.png`), clip: { x: 0, y: 0, width: SIZE, height: SIZE } });
  console.log(`crest-${id}-512.png`);
}
await browser.close();
