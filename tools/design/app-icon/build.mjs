// Builds every Legatum icon from one vector source: the Atlas Seal (the wordmark's star-crowned A in
// an engraved roundel, brick-red star, on parchment).
//
//   node tools/design/app-icon/build.mjs          (from the repo root; needs app/node_modules)
//
// Writes:
//   tools/design/app-icon/seal.svg            flat vector master of the mark
//   app/public/favicon.svg                    favicon, drawn for 16-32 px (also served by the dashboard)
//   app/public/favicon-32.png, favicon-16.png PNG fallbacks
//   app/public/apple-touch-icon.png           180 px full-bleed
//   app/src-tauri/icons/*                     via `npx tauri icon`, then icon.icns and 32x32.png are
//                                             rebuilt so 16-32 px use the bold small-size drawing
// The parchment texture (parchment.jpg) was generated with Codex image generation; the mark itself
// is pure vector.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../../..');
const APP = path.join(ROOT, 'app');
const { chromium } = createRequire(path.join(APP, 'package.json'))('playwright');

const f = (n) => +n.toFixed(2);
const C = { paper: '#f2ead8', ink: '#2b2419', brick: '#9a3322' };

// The house star (STAR_PATH in app/src/brand/ornaments.tsx), parametric: tip radius R, waist ratio k.
function star(cx, cy, R, k = 0.27) {
  const w = (R * k) / Math.SQRT2;
  return `M${f(cx)} ${f(cy - R)}L${f(cx + w)} ${f(cy - w)}L${f(cx + R)} ${f(cy)}L${f(cx + w)} ${f(cy + w)}L${f(cx)} ${f(cy + R)}L${f(cx - w)} ${f(cy + w)}L${f(cx - R)} ${f(cy)}L${f(cx - w)} ${f(cy - w)}Z`;
}

// The wordmark's A as filled geometry: legs cut flat on the baseline, apex beveled the way the
// wordmark's stroke (miter limit 2) bevels it.
function aLegs({ cx, top, base, half, w }) {
  const H = base - top;
  const len = Math.hypot(half, H);
  const nx = H / len;
  const ny = half / len;
  const lo = { x: cx - nx * (w / 2), y: top - ny * (w / 2), dx: -half };
  const li = { x: cx + nx * (w / 2), y: top + ny * (w / 2), dx: -half };
  const ro = { x: cx + nx * (w / 2), y: top - ny * (w / 2), dx: half };
  const ri = { x: cx - nx * (w / 2), y: top + ny * (w / 2), dx: half };
  const atY = (l, y) => l.x + ((y - l.y) / H) * l.dx;
  const innerY = li.y + ((cx - li.x) / -half) * H;
  return `M${f(atY(lo, base))} ${f(base)}L${f(lo.x)} ${f(lo.y)}L${f(ro.x)} ${f(ro.y)}L${f(atY(ro, base))} ${f(base)}L${f(atY(ri, base))} ${f(base)}L${cx} ${f(innerY)}L${f(atY(li, base))} ${f(base)}Z`;
}

// Alternating degree scale, as on the border of an engraved map.
function scaleRing(r0, r1, n) {
  let d = '';
  const p = (r, a) => `${f(50 + r * Math.cos(a))} ${f(50 + r * Math.sin(a))}`;
  for (let i = 0; i < n; i += 2) {
    const a0 = (i / n) * Math.PI * 2 - Math.PI / 2;
    const a1 = ((i + 1) / n) * Math.PI * 2 - Math.PI / 2;
    d += `M${p(r0, a0)}L${p(r1, a0)}A${r1} ${r1} 0 0 1 ${p(r1, a1)}L${p(r0, a1)}A${r0} ${r0} 0 0 0 ${p(r0, a0)}Z`;
  }
  return d;
}

const A = { cx: 50, top: 24.5, base: 71.5, half: 19.2, w: 6.2 };
const STAR = [50, 57.8, 8.6];

// Glory rays from the star, masked away around the letter, the star and the rule.
function rays(id, op) {
  let d = '';
  for (let i = 0; i < 96; i++) {
    const a = (i / 96) * Math.PI * 2;
    d += `M${f(STAR[0] + 12 * Math.cos(a))} ${f(STAR[1] + 12 * Math.sin(a))}L${f(STAR[0] + 60 * Math.cos(a))} ${f(STAR[1] + 60 * Math.sin(a))}`;
  }
  return `<mask id="${id}" maskUnits="userSpaceOnUse" x="0" y="0" width="100" height="100">
    <circle cx="50" cy="50" r="39.2" fill="#fff"/>
    <path d="${aLegs(A)}" stroke="#000" stroke-width="2.4" stroke-linejoin="round"/>
    <path d="${star(...STAR)}" stroke="#000" stroke-width="2.4"/>
    <path d="M29 80H71" stroke="#000" stroke-width="4"/>
  </mask>
  <path mask="url(#${id})" d="${d}" stroke="${C.ink}" stroke-width=".2" opacity="${op}"/>`;
}

// The seal in a 100-unit box: double rule, degree scale, the A, its star, the wordmark's rule.
function seal() {
  return `<circle cx="50" cy="50" r="46.4" fill="none" stroke="${C.ink}" stroke-width="2.4"/>
  <circle cx="50" cy="50" r="43.8" fill="none" stroke="${C.ink}" stroke-width=".55"/>
  <path d="${scaleRing(41.9, 43.8, 96)}" fill="${C.ink}"/>
  <circle cx="50" cy="50" r="41.9" fill="none" stroke="${C.ink}" stroke-width=".55"/>
  <circle cx="50" cy="50" r="39.6" fill="none" stroke="${C.ink}" stroke-width=".35" opacity=".55"/>
  <path d="${aLegs(A)}" fill="${C.ink}"/>
  <path d="${star(...STAR)}" fill="${C.brick}"/>
  <path d="M31.5 80H45M55 80H68.5" stroke="${C.ink}" stroke-width=".8" opacity=".6"/>
  <path d="${star(50, 80, 2.7)}" fill="${C.ink}"/>`;
}

const sealSvg = () => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><circle cx="50" cy="50" r="47.6" fill="${C.paper}"/>${rays('r', 0.28)}${seal()}</svg>\n`;

// Favicon: redrawn on a 32 grid so it reads at 16 px. A self-contained parchment disc, so it holds
// on light and dark tab strips alike.
const faviconSvg = () => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="15.1" fill="${C.paper}" stroke="${C.ink}" stroke-width="1.8"/><path d="${aLegs({ cx: 16, top: 6.6, base: 25, half: 8, w: 3.3 })}" fill="${C.ink}"/><path d="${star(16, 19.9, 4.3, 0.3)}" fill="${C.brick}"/></svg>\n`;

/* ---- App icon: 1024 canvas on the macOS grid (824 body at 100, continuous corners, shadow) ---- */
function squircle(x, y, s, e = 268, n = 4.2) {
  // Quarter-superellipse corners: zero curvature where they meet the straight sides.
  const pts = [];
  const corner = (cx, cy, a0) => {
    for (let i = 0; i <= 48; i++) {
      const t = a0 + (i / 48) * (Math.PI / 2);
      const c = Math.cos(t);
      const sn = Math.sin(t);
      pts.push(`${f(cx + Math.sign(c) * Math.abs(c) ** (2 / n) * e)} ${f(cy + Math.sign(sn) * Math.abs(sn) ** (2 / n) * e)}`);
    }
  };
  corner(x + s - e, y + s - e, 0);
  corner(x + e, y + s - e, Math.PI / 2);
  corner(x + e, y + e, Math.PI);
  corner(x + s - e, y + e, (3 * Math.PI) / 2);
  return `M${pts.join('L')}Z`;
}
const SQ = squircle(100, 100, 824);
const PARCHMENT = `data:image/jpeg;base64,${fs.readFileSync(path.join(HERE, 'parchment.jpg')).toString('base64')}`;

const defs = `
  <linearGradient id="sheen" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".16"/><stop offset=".45" stop-color="#fff" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".14"/></linearGradient>
  <radialGradient id="vig" cx=".5" cy=".42" r=".72"><stop offset=".6" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".16"/></radialGradient>
  <filter id="grain" x="0" y="0" width="100%" height="100%">
    <feTurbulence type="fractalNoise" baseFrequency=".85" numOctaves="3" seed="7"/>
    <feColorMatrix type="matrix" values="0 0 0 0 .35  0 0 0 0 .27  0 0 0 0 .16  0 0 0 .1 0"/>
  </filter>
  <filter id="mottle" x="0" y="0" width="100%" height="100%">
    <feTurbulence type="fractalNoise" baseFrequency=".005" numOctaves="4" seed="10"/>
    <feColorMatrix type="matrix" values="0 0 0 0 .55  0 0 0 0 .42  0 0 0 0 .22  0 0 0 .5 -.17"/>
  </filter>
  <filter id="press" x="-5%" y="-5%" width="110%" height="110%">
    <feOffset in="SourceAlpha" dy="1.3" result="lo"/>
    <feFlood flood-color="#fffaf0" flood-opacity=".65"/><feComposite in2="lo" operator="in" result="light"/>
    <feOffset in="SourceAlpha" dy="-1" result="hi"/>
    <feFlood flood-color="#3a2a14" flood-opacity=".2"/><feComposite in2="hi" operator="in" result="dark"/>
    <feMerge><feMergeNode in="light"/><feMergeNode in="dark"/><feMergeNode in="SourceGraphic"/></feMerge>
  </filter>
  <filter id="raise" x="-10%" y="-10%" width="120%" height="125%"><feDropShadow dx="0" dy="7" stdDeviation="7" flood-color="#4a3718" flood-opacity=".28"/></filter>
  <linearGradient id="discLight" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fffdf6" stop-opacity=".55"/><stop offset=".5" stop-color="#fffdf6" stop-opacity="0"/><stop offset="1" stop-color="#6b5530" stop-opacity=".12"/></linearGradient>
  <linearGradient id="discRim" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fffaf0" stop-opacity=".95"/><stop offset=".5" stop-color="#fffaf0" stop-opacity="0"/><stop offset="1" stop-color="#5a4526" stop-opacity=".35"/></linearGradient>
  <clipPath id="disc"><circle cx="512" cy="512" r="347.49"/></clipPath>`;

const paper = `<rect x="100" y="100" width="824" height="824" fill="${C.paper}"/>
  <image href="${PARCHMENT}" x="100" y="100" width="824" height="824" preserveAspectRatio="xMidYMid slice"/>
  <rect x="100" y="100" width="824" height="824" filter="url(#mottle)" opacity=".45"/>
  <rect x="100" y="100" width="824" height="824" filter="url(#grain)"/>`;
const light = `<rect x="100" y="100" width="824" height="824" fill="url(#vig)"/><rect x="100" y="100" width="824" height="824" fill="url(#sheen)"/>`;

// Full detail (64 px and up): the seal blind-embossed as a raised paper disc, the ink pressed in.
function detailed() {
  const s = 7.15;
  const t = f(512 - 50 * s);
  return `<g filter="url(#raise)"><circle cx="512" cy="512" r="347.49" fill="${C.paper}"/></g>
  <g clip-path="url(#disc)"><image href="${PARCHMENT}" x="100" y="100" width="824" height="824" transform="rotate(90 512 512)"/><rect x="100" y="100" width="824" height="824" filter="url(#grain)"/></g>
  <circle cx="512" cy="512" r="347.49" fill="url(#discLight)"/>
  <circle cx="512" cy="512" r="345.49" fill="none" stroke="url(#discRim)" stroke-width="4"/>
  <g transform="translate(${t} ${t}) scale(${s})">${rays('appr', 0.3)}<g filter="url(#press)">${seal()}</g></g>`;
}
// Small drawing (16-32 px): one heavy ring and a bold A.
function simple() {
  return `<g transform="translate(155 155) scale(7.14)">
    <circle cx="50" cy="50" r="45" fill="none" stroke="${C.ink}" stroke-width="6.5"/>
    <path d="${aLegs({ cx: 50, top: 21, base: 77, half: 24, w: 10.5 })}" fill="${C.ink}"/>
    <path d="${star(50, 61, 12.5, 0.3)}" fill="${C.brick}"/></g>`;
}

function appSvg(inner, { bleed = false } = {}) {
  if (bleed) {
    // Full-bleed square for apple-touch-icon and iOS: the OS draws the mask and shadow.
    return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="100 100 824 824" width="1024" height="1024"><defs>${defs}</defs>${paper}${inner}${light}</svg>`;
  }
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024"><defs>${defs}
  <clipPath id="sq"><path d="${SQ}"/></clipPath>
  <filter id="shadow" x="-10%" y="-10%" width="120%" height="125%"><feDropShadow dx="0" dy="10" stdDeviation="12" flood-color="#000" flood-opacity=".3"/></filter></defs>
  <g filter="url(#shadow)"><path d="${SQ}" fill="${C.paper}"/></g>
  <g clip-path="url(#sq)">${paper}${inner}${light}</g>
  <path d="${SQ}" fill="none" stroke="rgba(255,255,255,.55)" stroke-width="3" clip-path="url(#sq)" opacity=".7"/>
  <path d="${SQ}" fill="none" stroke="rgba(0,0,0,.28)" stroke-width="1.2"/></svg>`;
}

async function render(browser, svg, out, size) {
  const page = await browser.newPage({ viewport: { width: size, height: size }, deviceScaleFactor: 1 });
  await page.setContent(`<body style="margin:0;background:transparent"><img src="data:image/svg+xml;base64,${Buffer.from(svg).toString('base64')}" width="${size}" height="${size}" style="display:block"></body>`);
  await page.evaluate(() => document.images[0].decode());
  await page.screenshot({ path: out, omitBackground: true, clip: { x: 0, y: 0, width: size, height: size } });
  await page.close();
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'legatum-icon-'));
const icons = path.join(APP, 'src-tauri/icons');
const pub = path.join(APP, 'public');
fs.mkdirSync(pub, { recursive: true });
fs.writeFileSync(path.join(HERE, 'seal.svg'), sealSvg());
fs.writeFileSync(path.join(pub, 'favicon.svg'), faviconSvg());

const browser = await chromium.launch();
try {
  const full = appSvg(detailed());
  const small = appSvg(simple());
  await render(browser, full, path.join(tmp, 'master.png'), 1024);
  await render(browser, faviconSvg(), path.join(pub, 'favicon-32.png'), 32);
  await render(browser, faviconSvg(), path.join(pub, 'favicon-16.png'), 16);
  await render(browser, appSvg(detailed(), { bleed: true }), path.join(pub, 'apple-touch-icon.png'), 180);

  execFileSync('npx', ['tauri', 'icon', path.join(tmp, 'master.png'), '-o', icons], { cwd: APP, stdio: 'inherit' });
  // Mobile and Windows Store sets are regenerated by `tauri icon` when those targets exist; we ship macOS only.
  for (const extra of ['android', 'ios', ...fs.readdirSync(icons).filter((n) => /^(Square|StoreLogo)/.test(n))]) {
    const p = path.join(icons, extra);
    if (fs.existsSync(p)) execFileSync('trash', [p]);
  }

  // icon.icns by hand, so the 16 and 32 px slots get the bold small drawing instead of a blur.
  const set = path.join(tmp, 'icon.iconset');
  fs.mkdirSync(set);
  for (const [name, px] of [['16x16', 16], ['16x16@2x', 32], ['32x32', 32], ['32x32@2x', 64], ['128x128', 128], ['128x128@2x', 256], ['256x256', 256], ['256x256@2x', 512], ['512x512', 512], ['512x512@2x', 1024]]) {
    await render(browser, px <= 32 ? small : full, path.join(set, `icon_${name}.png`), px);
  }
  execFileSync('iconutil', ['-c', 'icns', set, '-o', path.join(icons, 'icon.icns')]);
  await render(browser, small, path.join(icons, '32x32.png'), 32);
} finally {
  await browser.close();
}
console.log('icons written');
