#!/usr/bin/env node
// Generates the tintable faction seals and glyphs:
//   app/src/assets/heraldry/seal-<faction>.svg and glyph-<faction>.svg
//
//   node app/src/assets/heraldry/src/build-seals.mjs [faction]
//
// Every seal is one color (currentColor) in three strengths: solid relief, a half-tone for
// secondary relief, and a faint field tone with engraved hatching. Engraved detail on the relief
// (outlines, hatched shadows, separations between overlapping parts) is cut out through a mask,
// so the seal stays a single ink that can sit on any background and be tinted per player.
// Mask ids are prefixed per faction so several seals can be inlined in one document.
//
// The charges are the historical set approved on 2026-10-31 (docs/design/heraldry/): one object per
// people, alone in the field. The grand gold crests are rendered from these seals by render-crests.mjs.
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const OUT = fileURLToPath(new URL('..', import.meta.url));

import {
  f, polar, pt, poly, line, circle, ellipse, ring, rect, arc, star4, star4Ridges, leaf, T, xf, tp, mirror, both, C, FIELD, Seal, laurel,
} from './kit.mjs';

// ---------- shared rim: the common seal ----------
function rim(S) {
  S.solid(ring(C, C, 229, 238));
  S.cut(circle(C, C, 233.5), 1.2);
  S.solid(ring(C, C, 221, 224));
  // restrained ornament band: a row of pearls, with house stars at the quarters
  S.tone(ring(C, C, 198, 219));
  S.hatch(ring(C, C, 198, 219), { a: 0, s: 3.6, w: 0.8, op: 0.35 });
  const beads = [];
  for (let k = 0; k < 72; k++) {
    const a = k * 5;
    if ([0, 90, 180, 270].some((q) => Math.abs(((a - q + 540) % 360) - 180) > 180 - 11)) continue;
    const [x, y] = polar(C, C, 208.5, a);
    beads.push(circle(x, y, 3.3));
  }
  S.solid(beads.join(''));
  for (const q of [90, 180, 270]) {
    const [x, y] = polar(C, C, 208.5, q);
    S.solid(star4(x, y, 12, 3, 0));
  }
  S.solid(ring(C, C, 194, 196.5));
  S.solid(ring(C, C, FIELD, 191));
  // the field: faint tone with fine horizontal engraving
  S.tone(circle(C, C, FIELD));
  S.hatch(circle(C, C, FIELD), { a: 0, s: 4.4, w: 0.8, op: 0.28 });
  S.clip('field', circle(C, C, FIELD - 1));
  // the house star crowning the rim
  S.solid(star4(C, 30, 30, 7.5), { halo: 5 });
  S.cut(star4Ridges(C, 30, 30), 1.4);
  S.shade(poly([[C, 30], polar(C, 30, 30, 0), polar(C, 30, 7.5, 45), polar(C, 30, 30, 90), polar(C, 30, 7.5, 135), polar(C, 30, 30, 180)]), { a: 45, s: 3.2, w: 1.1 });
}

// ---------- shared shapes ----------

/** Annular sector between radii r1 < r2 from compass angle a0 clockwise to a1. */
function band(cx, cy, r1, r2, a0, a1) {
  const big = Math.abs(a1 - a0) > 180 ? 1 : 0;
  return `M${pt(polar(cx, cy, r2, a0))}A${f(r2)} ${f(r2)} 0 ${big} 1 ${pt(polar(cx, cy, r2, a1))}L${pt(polar(cx, cy, r1, a1))}A${f(r1)} ${f(r1)} 0 ${big} 0 ${pt(polar(cx, cy, r1, a0))}Z`;
}
/** Mirror top to bottom about the horizontal line y. */
const flipY = (d, y = C) => xf(d, [1, 0, 0, -1, 0, 2 * y]);
const at = (x, y, a = 0, k = 1) => T.c(T.t(x, y), T.r(a), T.s(k));

/** The heraldic flame, drawn at unit scale: base center at 0,0, rising 262 units. */
const FLAME =
  'M0 0C-56 0 -92 -34 -90 -80C-89 -110 -96 -130 -108 -156C-84 -144 -64 -134 -58 -114C-62 -150 -58 -180 -62 -214' +
  'C-40 -192 -30 -176 -26 -150C-26 -196 -10 -230 4 -262C12 -226 28 -196 30 -164C40 -184 52 -200 70 -214' +
  'C62 -176 60 -150 62 -122C76 -138 92 -148 110 -150C96 -128 92 -104 92 -80C94 -34 56 0 0 0Z';

/** A flame with nested engraved contours and a hatched shadow side. */
function flame(S, x, y, k, o = {}) {
  const m = T.c(T.t(x, y), T.s(k));
  const d = xf(FLAME, m);
  S.solid(d, { halo: o.halo ?? 3 });
  const clip = S.clip(`flame${Math.round(x)}${Math.round(y)}`, d);
  S.shade(rect(x + 14 * k, y - 270 * k, 120 * k, 280 * k), { a: 65, s: 3 }, { clip });
  const nest = [0.74, 0.5, 0.28]
    .map((q) => xf(FLAME, T.c(T.t(x, y - 18 * k), T.s(k * q), T.t(0, 0))))
    .join('');
  S.cut(nest, o.vein ?? 1.6);
}

// ---------- charges ----------

function switzerland(S) {
  // The alpine cowbell on its broad stitched strap.
  const sc = [256, 178];
  const strap = band(sc[0], sc[1], 52, 92, -160, 160);
  S.solid(strap, { halo: 3 });
  S.cut(arc(sc[0], sc[1], 59, -158, 158) + arc(sc[0], sc[1], 85, -159, 159), 1.3);
  // studs along the strap, and the buckle
  const studs = [];
  for (let a = -10; a <= 144; a += 22) studs.push(star4(...polar(sc[0], sc[1], 72, a), 9, 2.6, a));
  for (const a of [-98, -120, -142]) studs.push(star4(...polar(sc[0], sc[1], 72, a), 9, 2.6, a));
  S.cutFill(studs.join(''));
  S.shade(band(sc[0], sc[1], 52, 64, -160, 160), { a: 60, s: 3.4, w: 1 });
  const bm = at(...polar(sc[0], sc[1], 72, -56), -56);
  S.solid(xf(rect(-27, -19, 54, 38), bm), { halo: 3 });
  S.cutFill(xf(rect(-19, -11, 16, 22) + rect(3, -11, 16, 22), bm));
  S.cut(xf('M-23 -15H23V15H-23Z', bm), 1);
  // the hanger and the bell
  S.solid(rect(228, 256, 56, 22), { halo: 2.4 });
  S.cut('M228 267H284', 1.2);
  const bell = 'M210 276Q256 264 302 276C308 330 332 366 368 388V404H144V388C180 366 204 330 210 276Z';
  S.solid(bell, { halo: 4 });
  S.clip('bell', bell);
  S.cut('M207 300Q256 311 305 300M204 310Q256 321 308 310M152 386H360M149 393H363', 1.4);
  S.shade('M276 270H380V410H318C306 368 286 320 276 270Z', { a: 62, s: 3.2 }, { clip: 'bell' });
  S.cut('M222 318C222 344 204 368 178 384', 1.5, { clip: 'bell' });
  S.cutFill(star4(256, 350, 31, 8));
  S.solid(star4(256, 350, 22, 5.5), { halo: 1 });
  // the clapper showing under the mouth
  S.solid(rect(252, 404, 8, 12) + circle(256, 423, 12), { halo: 2.5, clip: 'field' });
  S.cut(arc(256, 423, 7, 100, 250), 1.2);
}

function india(S) {
  // The lotus of the Mughal court: eight petals and eight sepals around the seed head.
  const c = [256, 258];
  const petal = 'M0 -72C-24 -44 -33 -8 0 30C33 -8 24 -44 0 -72Z';
  for (let k = 0; k < 8; k++) {
    const m = at(...polar(c[0], c[1], 94, k * 45 + 22.5), k * 45 + 22.5, 0.74);
    S.half(xf(petal, m), { halo: 2.4, clip: 'field', op: 0.62 });
    S.cut(xf('M0 -58V8', m), 1.3, { clip: 'field' });
  }
  for (let k = 0; k < 8; k++) {
    const m = at(...polar(c[0], c[1], 90, k * 45), k * 45);
    S.solid(xf(petal, m), { halo: 3.2, clip: 'field' });
    S.shade(xf('M0 -72C24 -44 33 -8 0 30Z', m), { a: k * 45 + 62, s: 3.2, w: 1.1 }, { clip: 'field' });
    S.cut(xf('M0 -60V14M-13 -30Q-7 -6 0 6M13 -30Q7 -6 0 6', m), 1.3, { clip: 'field' });
  }
  S.solid(circle(c[0], c[1], 50), { halo: 4 });
  S.cut(circle(c[0], c[1], 43), 1.4);
  const seeds = [];
  for (let k = 0; k < 12; k++) seeds.push(circle(...polar(c[0], c[1], 33, k * 30 + 15), 4));
  S.cutFill(seeds.join('') + circle(c[0], c[1], 22));
  S.solid(star4(c[0], c[1], 19, 5), { halo: 1 });
}

function byzantium(S) {
  // The processional cross: flared arms, pearls at the corners, pendilia, a knop and tang for the staff.
  const c = [256, 214];
  const arm = (len, w0, w1) => `M${-w0 / 2} 0Q${-w0 / 2 - 1} ${-len * 0.5} ${-w1 / 2} ${-len}L${w1 / 2} ${-len}Q${w0 / 2 + 1} ${-len * 0.5} ${w0 / 2} 0Z`;
  const ARMS = [[0, 104], [90, 88], [180, 132], [270, 88]];
  // pendilia: chains with drops hanging from the cross arm
  for (const x of [184, 206, 306, 328]) {
    const len = Math.abs(x - 256) > 60 ? 64 : 46;
    const y0 = 240;
    const links = [];
    for (let y = y0 + 12; y < y0 + len; y += 11) links.push(circle(x, y, 3.4));
    S.solid(links.join('') + leaf([x, y0 + len - 4], 180, 30, 9).d, { halo: 2 });
  }
  let cross = '';
  let inner = '';
  let pearls = '';
  let gems = '';
  for (const [a, len] of ARMS) {
    const m = at(c[0], c[1], a);
    cross += xf(arm(len, 38, 72), m);
    inner += xf(`M-10 -32Q-11 ${-len * 0.55} -25 ${-len + 9}H25Q11 ${-len * 0.55} 10 -32`, m);
    pearls += xf(circle(-38, -len - 2, 10) + circle(38, -len - 2, 10), m);
    gems += xf(circle(0, -len + 24, 7) + (len > 100 ? circle(0, -len + 54, 5) : ''), m);
  }
  S.solid(pearls, { halo: 2.4 });
  S.solid(cross, { halo: 3.4 });
  S.clip('cross', cross);
  S.shade(rect(256, 100, 40, 260), { a: 60, s: 3.2, w: 1.1 }, { clip: 'cross' });
  S.shade(rect(160, 214, 200, 40), { a: 60, s: 3.2, w: 1.1 }, { clip: 'cross' });
  S.cut(inner, 1.4);
  S.cutFill(gems);
  // the medallion at the crossing
  S.cutFill(circle(c[0], c[1], 38));
  S.solid(circle(c[0], c[1], 33), { halo: 1 });
  S.cut(circle(c[0], c[1], 27), 1.3);
  S.cutFill(star4(c[0], c[1], 21, 5.5));
  // knop and tang
  S.solid(ellipse(256, 360, 23, 13), { halo: 3 });
  S.cut('M235 360H277', 1.3);
  S.solid(rect(245, 373, 22, 7) + poly([[248, 380], [264, 380], [259, 438], [253, 438]]), { halo: 2.4, clip: 'field' });
  S.shade(poly([[256, 380], [264, 380], [259, 438], [256, 438]]), { a: 90, s: 2.6, w: 1 });
}

// The eagle is drawn once on the 24 grid and used by both the glyph and (enlarged) the seal.
const EAGLE = {
  wing: 'M10.5 12.6L9.9 9.6L7.6 7.6L4.8 7L2 5.4L3.6 8.2L0.8 9.4L4 10.6L1.4 13.2L4.8 12.8L3.4 16.2L6.4 14.2L6.2 17.6L8.4 14.9L10.4 14.3Z',
  head: 'M12 9.6L10.7 5C11 3.3 10 1.7 8.4 1.7C7.5 1.7 6.8 2.2 6.5 2.9L4.6 3.6Q3.5 4.3 4 5.9L5.2 4.9L6.5 4.8L5.9 6C6.8 6.3 7.6 6.1 8.2 5.6L10.4 10.8Z',
  body: 'M12 9C14 9 14.4 11.6 14.1 13.8C13.8 15.7 12.9 16.9 12 17.6C11.1 16.9 10.2 15.7 9.9 13.8C9.6 11.6 10 9 12 9Z',
  leg: 'M10 15.2L11.2 16.4L9 18.4L9.4 20.4L8.1 19.2L6.2 19.8L7.6 18Z',
  tail: 'M10.7 17L13.3 17L14.5 20.6L12.9 20L12 23.2L11.1 20L9.5 20.6Z',
};

function germany(S) {
  // The double-headed eagle of the old Empire, displayed; no crown, shield or regalia.
  const M = T.c(T.t(C - 12 * 14.6, C - 12.3 * 14.6), T.s(14.6));
  const g = (d) => xf(d, M);
  const L = (pts) => g('M' + pts.map(([x, y]) => `${x} ${y}`).join('L'));
  // wings, with the feathers parted and ribbed
  S.solid(both(g(EAGLE.wing)), { halo: 3.4, clip: 'field' });
  S.clip('wings', both(g(EAGLE.wing)));
  S.shade(both(g('M10.5 12.6L8 11L4.6 9.4L0.8 9.4L1.4 13.2L3.4 16.2L6.2 17.6L8.4 14.9L10.4 14.3Z')), { a: 60, s: 3.2, w: 1.1 }, { clip: 'wings' });
  const parts = [[[3.6, 8.2], [7.9, 8.7]], [[4, 10.6], [8.6, 9.8]], [[4.8, 12.8], [9.1, 10.8]], [[6.4, 14.2], [9.6, 11.7]], [[8.4, 14.9], [10.1, 12.6]]];
  S.cut(both(parts.map(L).join('')), 1.8, { clip: 'wings' });
  const ribs = [[[2.7, 6], [6.6, 7.7]], [[1.7, 9.3], [6.6, 8.9]], [[2.2, 12.8], [7, 10.3]], [[3.8, 15.5], [7.7, 11.7]], [[6.2, 16.6], [8.7, 12.7]], [[8.8, 14.4], [9.9, 13]]];
  S.cut(both(ribs.map(L).join('')), 1, { clip: 'wings' });
  // coverts along the wing bone
  let coverts = '';
  for (const [x, y] of [[9.2, 9.6], [8.3, 8.7], [7.2, 8], [6, 7.6], [4.9, 7.4]]) coverts += g(`M${x - 0.55} ${y + 0.5}Q${x - 0.5} ${y + 1.5} ${x + 0.55} ${y + 1.1}`);
  S.cut(both(coverts), 1.3, { clip: 'wings' });
  // tail and legs
  S.solid(g(EAGLE.tail), { halo: 2.6, clip: 'field' });
  S.cut(L([[12, 17.6], [12, 22.4]]) + L([[11.2, 17.4], [10.4, 20]]) + L([[12.8, 17.4], [13.6, 20]]), 1.2);
  S.solid(both(g(EAGLE.leg)), { halo: 2.6 });
  S.cut(both(L([[10.4, 16], [8.6, 18.4]])), 1.1);
  // necks and heads, then the breast over them
  S.solid(both(g(EAGLE.head)), { halo: 3 });
  S.cutFill(both(circle(...tp(M, 8.2, 3.5), 4.6)));
  S.cut(both(L([[6.4, 3.9], [4.7, 4.5]]) + L([[9.6, 6.2], [11, 9.6]])), 1.2);
  S.shade(both(g('M10.6 5L12 9.6L10.4 10.8L9.4 8Z')), { a: 30, s: 3, w: 1 });
  const body = g(EAGLE.body);
  S.solid(body, { halo: 3.4 });
  S.clip('body', body);
  let scales = '';
  for (let r = 0; r < 8; r++) for (let x = 212 + (r % 2) * 9; x < 304; x += 18) scales += `M${x} ${218 + r * 15}q9 12 18 0`;
  S.cut(scales, 1.2, { clip: 'body' });
  S.shade(rect(268, 200, 40, 150), { a: 60, s: 3.2, w: 1.1 }, { clip: 'body' });
  // the house star between the heads
  S.solid(star4(256, 132, 20, 5), { halo: 2.5 });
  S.cut(star4Ridges(256, 132, 20), 1.1);
}

function celts(S) {
  // A La Tène torc: a twisted gold neck ring with buffer terminals, worn with the opening forward.
  const c = [256, 240];
  const [r1, r2, gap] = [88, 126, 146];
  const hoop = band(c[0], c[1], r1, r2, -gap, gap);
  S.solid(hoop, { halo: 3.5 });
  S.clip('hoop', hoop);
  let twist = '';
  for (let a = -gap + 6; a < gap - 4; a += 7.5) twist += `M${pt(polar(c[0], c[1], r1 - 2, a - 6))}Q${pt(polar(c[0], c[1], (r1 + r2) / 2, a - 3))} ${pt(polar(c[0], c[1], r2 + 2, a + 6))}`;
  S.cut(twist, 1.8, { clip: 'hoop' });
  S.shade(band(c[0], c[1], r1, r1 + 13, -gap, gap), { a: 20, s: 3, w: 1.1 }, { clip: 'hoop' });
  S.cut(arc(c[0], c[1], r2 - 5, -gap, gap), 1);
  // collars and buffer terminals
  for (const s of [-1, 1]) {
    const a = s * gap;
    const m = at(...polar(c[0], c[1], (r1 + r2) / 2, a), a + (s > 0 ? 90 : -90));
    S.solid(xf(rect(-9, -25, 14, 50), m), { halo: 2.4 });
    S.cut(xf('M-2 -25V25', m), 1.3);
    const t = polar(c[0], c[1], (r1 + r2) / 2 + 2, s * (gap + 13));
    S.solid(circle(t[0], t[1], 30), { halo: 3.4 });
    S.cut(circle(t[0], t[1], 23.5), 1.4);
    S.cutFill(ring(t[0], t[1], 8, 15.5));
    S.shade(`M${f(t[0])} ${f(t[1] - 23)}A23 23 0 0 1 ${f(t[0])} ${f(t[1] + 23)}Z`, { a: 60, s: 3, w: 1.1 });
    const pellets = [0, 120, 240].map((b) => circle(...polar(t[0], t[1], 11.8, b + s * 30), 2.4)).join('');
    S.solid(pellets, { halo: 0.6 });
  }
  S.solid(star4(c[0], c[1], 36, 9), { halo: 2.5 });
  S.cut(star4Ridges(c[0], c[1], 36), 1.2);
}

function mongols(S) {
  // The recurved composite bow of the steppe horsemen, strung, an arrow on the string.
  const M = T.c(T.t(C + 34, C), T.s(0.93), T.t(-C, -C));
  const g = (d) => xf(d, M);
  const limb = 'M214 236C196 184 232 140 288 120C298 108 292 92 280 82';
  const limbs = g(limb + flipY(limb));
  // the string runs nock to nock
  S.stroke(g('M283 88V424'), 3.2, { halo: 1.4 });
  S.stroke(limbs, 21, { halo: 3, cap: 'round' });
  S.cut(limbs, 1.3);
  // sinew bindings on the limbs and the horn ears
  let wraps = '';
  for (const t of [-1, 1]) for (const [x, y, a] of [[207, 206, 78], [212, 184, 62], [240, 148, 34], [262, 134, 26], [291, 106, 104]]) {
    const yy = t < 0 ? y : 2 * C - y;
    const aa = t < 0 ? a : 180 - a;
    wraps += line(polar(x, yy, 13, aa), polar(x, yy, 13, aa + 180));
  }
  S.cut(g(wraps), 1.6);
  // the grip
  const grip = g('M200 224H228Q234 256 228 288H200Q194 256 200 224Z');
  S.solid(grip, { halo: 3 });
  let cord = '';
  for (let y = 234; y < 284; y += 7) cord += `M194 ${y + 3}L234 ${y}`;
  S.cut(g(cord), 1.4, { clip: S.clip('grip', grip) });
  S.shade(g('M214 224H228Q234 256 228 288H214Z'), { a: 90, s: 2.6, w: 1 });
  // the arrow, nocked
  S.solid(g(rect(64, 251.5, 219, 9)), { halo: 2.4 });
  const head = poly([[26, 256], [84, 230], [72, 256], [84, 282]]);
  S.solid(g(head), { halo: 2.6 });
  S.cut(g('M38 256H72'), 1.2);
  S.shade(g(poly([[26, 256], [72, 256], [84, 282]])), { a: 45, s: 2.8, w: 1 });
  const vane = poly([[230, 252], [246, 229], [279, 229], [270, 252]]);
  S.solid(g(vane + flipY(vane)), { halo: 2.2 });
  let barbs = '';
  for (let x = 240; x < 270; x += 8) barbs += `M${x} 252L${x + 13} 232M${x} 260L${x + 13} 280`;
  S.cut(g(barbs), 1.1);
  S.solid(g(rect(281, 250, 8, 12)), { halo: 1.5 });
}

function sparta(S) {
  // A Corinthian hoplite helmet in profile under its horsehair crest.
  const plume =
    'M160 204C150 124 228 92 280 96C352 102 402 168 404 250C405 300 398 334 386 350' +
    'C384 310 378 272 368 240C354 180 314 140 262 140C220 140 190 166 180 206Z';
  S.solid(plume, { halo: 3.4, clip: 'field' });
  S.clip('plume', plume);
  let hair = '';
  for (let a = -74; a <= 112; a += 4.6) hair += line(polar(262, 268, 104, a - 5), polar(262, 268, 190, a + 4));
  S.cut(hair, 1.5, { clip: 'plume' });
  S.shade('M330 96H420V360H376C380 300 372 200 330 150Z', { a: 30, s: 3.4, w: 1.1 }, { clip: 'plume' });
  // the helmet
  const helm =
    'M170 262C166 196 206 150 262 150C318 150 352 196 352 262C352 300 358 328 382 352L376 368' +
    'C344 364 314 352 290 340C278 374 252 402 206 416L212 330L206 290L262 276L188 258L184 300L192 374L176 330Z';
  S.solid(helm, { halo: 4 });
  S.clip('helm', helm);
  S.shade('M300 140H400V380H292C316 320 320 220 300 140Z', { a: 60, s: 3.2, w: 1.1 }, { clip: 'helm' });
  // brow ridge, the raised border round the eye, cheek and nape, and rivets
  S.cut('M178 244C204 222 250 226 284 252M196 250L276 270', 1.6, { clip: 'helm' });
  S.cut('M220 334L216 404C254 388 274 362 284 330C310 344 340 356 372 360M220 334L286 292', 1.4, { clip: 'helm' });
  S.cut('M300 160C330 190 342 226 342 262C342 296 348 322 366 344', 1.2, { clip: 'helm' });
  S.cutFill([[232, 356], [256, 322], [312, 338], [344, 350]].map(([x, y]) => circle(x, y, 3.2)).join(''));
  // the crest holder
  S.stroke('M176 222C184 172 218 145 262 145C316 145 350 190 358 246', 4.5, { halo: 1.2 });
}

function athens(S) {
  // The little owl of the Athenian coinage on an olive branch.
  // the perch
  S.stroke('M132 408C200 398 312 398 380 408', 5, { halo: 2, clip: 'field', cap: 'round' });
  for (const [x, a] of [[150, -62], [176, 118], [338, -118], [362, 62]]) {
    const lf = leaf([x, 404 + (Math.abs(x - 256) > 100 ? 2 : -1)], a, 38, 9);
    S.solid(lf.d, { halo: 1.6, clip: 'field' });
    S.cut(lf.rib, 1, { clip: 'field' });
  }
  S.solid(circle(196, 392, 6.5) + circle(318, 392, 6.5), { halo: 1.4 });
  // tail under the perch
  S.solid(poly([[234, 400], [278, 400], [270, 436], [256, 428], [242, 436]]), { halo: 2, clip: 'field' });
  S.cut('M248 406V428M264 406V428', 1.2, { clip: 'field' });
  // folded wings, then the body
  const wingL = 'M204 244C162 286 166 352 220 400C206 348 208 296 222 250Z';
  S.solid(both(wingL), { halo: 2.6 });
  S.cut(both('M196 270C176 306 184 348 206 378M206 262C192 300 196 344 212 376'), 1.3);
  const body = 'M256 226C318 226 326 296 314 344C304 382 282 400 256 404C230 400 208 382 198 344C186 296 194 226 256 226Z';
  S.solid(body, { halo: 3.4 });
  S.clip('owl', body);
  // the dotted breast of the coin's owl
  const dots = [];
  for (let r = 0; r < 9; r++) for (let x = 204 + (r % 2) * 8; x < 312; x += 16) dots.push(circle(x, 262 + r * 15, 3.6));
  S.cutFill(dots.join(''), { clip: 'owl' });
  S.shade(rect(288, 226, 40, 180), { a: 60, s: 3.2, w: 1.1 }, { clip: 'owl' });
  // talons on the perch
  const claws = [-22, 0, 22].map((dx) => leaf([226 + dx * 0.5, 396], 180 + dx * 0.4, 20, 5).d).join('');
  S.solid(both(claws), { halo: 1.6 });
  // the head, turned full face
  const head = 'M256 118C312 118 344 150 344 190C344 232 306 252 256 252C206 252 168 232 168 190C168 150 200 118 256 118Z';
  S.solid(head, { halo: 3.6 });
  S.cut('M186 150C214 142 244 158 256 186C268 158 298 142 326 150', 1.6);
  S.shade('M300 124C332 140 346 164 344 196C340 226 322 240 300 248Z', { a: 60, s: 3.2, w: 1.1 });
  for (const x of [220, 292]) {
    S.cutFill(circle(x, 188, 33));
    S.solid(circle(x, 188, 26), { halo: 1 });
    S.cutFill(circle(x, 188, 14.5));
    S.solid(circle(x - 4, 184, 4.6), { halo: 0.6 });
  }
  S.solid(poly([[256, 196], [244, 212], [256, 240], [268, 212]]), { halo: 3 });
  S.cut('M256 200V234', 1.1);
  // an olive sprig in the field, as on the coin
  for (const [b, a, L] of [[[146, 222], -18, 46], [[152, 196], 36, 40]]) {
    const lf = leaf(b, a, L, 9.5);
    S.solid(lf.d, { halo: 1.6, clip: 'field' });
    S.cut(lf.rib, 1, { clip: 'field' });
  }
  S.stroke('M150 244C146 220 150 204 156 190', 3, { cap: 'round' });
  S.solid(circle(136, 240, 6), { halo: 1.4 });
}

function aztecs(S) {
  // The New Fire: a bound bundle of reeds (the binding of the years) kindled on the temple steps.
  // A ceremonial charge, not a claim to a historical coat of arms.
  const steps = rect(214, 372, 84, 17) + rect(190, 389, 132, 17) + rect(166, 406, 180, 18);
  S.solid(steps, { halo: 2.6, clip: 'field' });
  S.cut('M214 389H298M190 406H322M240 372V424M272 372V424', 1.3, { clip: 'field' });
  let treads = '';
  for (let y = 378; y < 424; y += 6) treads += `M240 ${y}H272`;
  S.cut(treads, 1, { clip: 'field' });
  S.shade(rect(272, 372, 80, 60), { a: 60, s: 3, w: 1.1 }, { clip: S.clip('steps', steps) });
  // the bundle
  const bundle = poly([[234, 372], [278, 372], [292, 268], [220, 268]]);
  S.solid(bundle, { halo: 3 });
  S.clip('bundle', bundle);
  let reeds = '';
  for (let k = 1; k < 6; k++) reeds += line([234 + k * 7.3, 372], [220 + k * 12, 268]);
  S.cut(reeds, 1.4, { clip: 'bundle' });
  S.shade(poly([[262, 372], [278, 372], [292, 268], [268, 268]]), { a: 60, s: 3, w: 1.1 });
  for (const [y, w] of [[296, 78], [338, 64]]) {
    const tie = rect(256 - w / 2, y, w, 15);
    S.solid(tie, { halo: 2.4 });
    let rope = '';
    for (let x = 256 - w / 2 - 6; x < 256 + w / 2; x += 7) rope += `M${x} ${y + 15}l9 -15`;
    S.cut(rope, 1.3, { clip: S.clip(`tie${y}`, tie) });
  }
  // the fire bowl and the flame
  flame(S, 256, 246, 0.56, { vein: 1.5 });
  S.solid('M204 244H308L294 270H218Z' + rect(198, 234, 116, 11), { halo: 3 });
  S.cut('M212 250H300', 1.2);
  S.shade('M262 245H308L294 270H262Z', { a: 60, s: 3, w: 1.1 });
  for (const [x, y, r] of [[176, 170, 9], [338, 150, 8], [352, 214, 6], [162, 226, 6]]) S.solid(star4(x, y, r * 1.5, r * 0.4), { halo: 2 });
}

/** A wheat stalk from base (x, y) along compass angle a: a stem with paired grains and a terminal ear. */
function wheat(S, x, y, a, L) {
  const tip = polar(x, y, L, a);
  S.stroke(line([x, y], tip), 3.2, { cap: 'round', halo: 1.4 });
  const grains = [];
  for (let k = 0; k < 6; k++) {
    const p = polar(x, y, L * (0.42 + 0.085 * k), a);
    for (const side of [-1, 1]) grains.push(leaf(p, a + side * 34, 25, 6.6).d);
  }
  grains.push(leaf(polar(x, y, L * 0.96, a), a, 26, 6.6).d);
  S.solid(grains.join(''), { halo: 1 });
}

function egypt(S) {
  // The winged sun disk over a temple pylon, a sheaf at its foot: the state builds what the harvest feeds.
  const wings = [];
  const ribs = [];
  [[56, 112], [71, 126], [86, 134], [101, 122], [116, 98]].forEach(([a, L], i) => {
    const lf = leaf([282, 144 + i * 5], a, L, 11, -3);
    wings.push(lf.d);
    ribs.push(lf.rib);
  });
  S.solid(both(wings.join('')), { halo: 1.6 });
  S.cut(both(ribs.join('')), 1.1);
  S.solid(circle(256, 148, 28), { halo: 3 });
  S.cut(circle(256, 148, 19), 1.8);
  S.solid(circle(256, 148, 12), { halo: 2 });
  // the pylon: two towers with sloped sides, a cavetto cornice with a torus roll down each edge, a gateway between
  const tower = poly([[154, 350], [184, 242], [238, 242], [238, 350]]);
  S.solid(tower + mirror(tower), { halo: 2 });
  S.solid('M176 244L176 236Q176 224 158 210L354 210Q336 224 336 236L336 244Z', { halo: 2 });
  S.cut(line([180, 227], [332, 227]), 1.4);
  S.cut(line([178, 241], [334, 241]), 1.4);
  let flutes = '';
  for (let x = 170; x <= 342; x += 9) flutes += `M${x} 213V225`;
  S.cut(flutes, 1.1);
  S.cut(both(line([164, 346], [191, 252])), 1.6);
  S.solid(rect(238, 258, 36, 12), { halo: 1.5 });
  S.shade(poly([[218, 246], [238, 246], [238, 350], [222, 350]]), { a: 82, s: 3.4 });
  S.shade(poly([[314, 246], [332, 246], [352, 350], [316, 350]]), { a: 82, s: 3.4 });
  S.cut(both(line([206, 272], [200, 334])), 3.4);
  S.solid(rect(146, 350, 220, 12), { halo: 1.5 });
  S.cut('M146 356H366', 1.1);
  // the sheaf
  [-60, -30, 0, 30, 60].forEach((a) => wheat(S, 256, 438, a, 76));
  S.solid(rect(228, 408, 56, 9), { halo: 1.5 });
}

function rome(S) {
  // A legion standard (signum): spearhead, crossbar with pendants, wreath, phalerae and crescent, within laurel.
  laurel(S, 160, 200, 11, 9.5, 30, C + 6);
  // the staff
  S.solid(rect(249.5, 112, 13, 306) + poly([[247, 418], [265, 418], [256, 442]]), { halo: 2.6, clip: 'field' });
  S.shade(rect(256, 112, 6.5, 306), { a: 90, s: 2.6, w: 1 });
  S.solid(poly([[256, 74], [277, 126], [256, 116], [235, 126]]), { halo: 2.4 });
  S.cut('M256 90V112', 1.4);
  S.shade(poly([[256, 74], [277, 126], [256, 116]]), { a: 60, s: 2.8, w: 1 });
  // crossbar and its pendant straps
  S.solid(rect(184, 132, 144, 10) + circle(184, 137, 8) + circle(328, 137, 8), { halo: 2.6 });
  S.cut('M194 137H318', 1.1);
  for (const x of [194, 318]) {
    S.stroke(`M${x} 143V190`, 3.6, { halo: 1.2 });
    const drop = leaf([x, 186], 180, 34, 10);
    S.solid(drop.d, { halo: 1.8 });
    S.cut(drop.rib, 1);
  }
  // the wreath
  const w = [256, 184];
  S.solid(ring(w[0], w[1], 17, 38), { halo: 2.8 });
  let leaves = '';
  for (let a = 0; a < 360; a += 20) leaves += line(polar(w[0], w[1], 19, a + 12), polar(w[0], w[1], 37, a - 6));
  S.cut(leaves, 1.6);
  S.solid(circle(w[0], w[1], 7), { halo: 1.5 });
  // phalerae
  [[254, 28], [312, 24]].forEach(([y, r]) => {
    S.solid(circle(256, y, r), { halo: 2.8 });
    S.cut(circle(256, y, r * 0.72), 1.5);
    S.cutFill(ring(256, y, r * 0.28, r * 0.5));
    S.shade(`M256 ${y - r}A${r} ${r} 0 0 1 256 ${y + r}Z`, { a: 60, s: 2.8, w: 1 });
  });
  // crescent and tassel
  S.solid('M224 342A32 32 0 0 0 288 342A34 46 0 0 1 224 342Z', { halo: 2.4 });
  S.solid(poly([[244, 382], [268, 382], [275, 412], [237, 412]]), { halo: 2.2, clip: 'field' });
  let fringe = '';
  for (let x = 244; x <= 268; x += 6) fringe += `M${x} 388L${x + (x - 256) * 0.3} 412`;
  S.cut(fringe + 'M242 388H270', 1.2);
}

const FACTIONS = {
  switzerland: ['Switzerland', switzerland],
  india: ['India', india],
  byzantium: ['Byzantium', byzantium],
  germany: ['Germany', germany],
  celts: ['Celts', celts],
  mongols: ['Mongols', mongols],
  sparta: ['Sparta', sparta],
  athens: ['Athens', athens],
  aztecs: ['Aztecs', aztecs],
  egypt: ['Egypt', egypt],
  rome: ['Rome', rome],
};

const only = process.argv[2];
for (const [key, [title, draw]] of Object.entries(FACTIONS)) {
  if (only && key !== only) continue;
  const S = new Seal(`seal-${key}`);
  rim(S);
  draw(S);
  const svg = S.svg(title);
  writeFileSync(`${OUT}seal-${key}.svg`, svg);
  console.log(`seal-${key}.svg ${(svg.length / 1024).toFixed(1)} KB`);
}

// ---------- glyphs: the same charge as a solid silhouette that reads at 16-24 px ----------
const fill = (d) => `<path d="${d}"/>`;
const strk = (d, w, cap = 'butt') => `<path d="${d}" fill="none" stroke="currentColor" stroke-width="${w}" stroke-linecap="${cap}"/>`;
const mx12 = (d) => xf(d, T.mx(12));
const my12 = (d) => xf(d, [1, 0, 0, -1, 0, 24]);

const GLYPHS = {
  switzerland: () => [
    // cowbell: strap loop, bell with a band knocked out, clapper
    fill(band(12, 6.6, 2.5, 5.2, -138, 138)),
    fill('M8.6 9.6H15.4C15.8 13.4 17.6 16.4 20.8 18.4V20.2H3.2V18.4C6.4 16.4 8.2 13.4 8.6 9.6ZM7.9 13.2Q12 14.2 16.1 13.2L16.5 14.4Q12 15.5 7.5 14.4Z'),
    fill(circle(12, 22, 1.7)),
  ],
  india: () => [
    // lotus seen from above: eight separate petals around the seed head
    ...Array.from({ length: 8 }, (_, k) => fill(xf('M0 -11C-2.9 -8.6 -3 -5.6 0 -3.6C3 -5.6 2.9 -8.6 0 -11Z', at(12, 12, k * 45)))),
    fill(circle(12, 12, 2.5)),
  ],
  byzantium: () => {
    // processional cross: flared arms, a pearl at each corner, knop and tang
    const arm = (len) => `M-1.5 0L-3 ${-len}H3L1.5 0Z`;
    const pearls = (len) => circle(-3.4, -len - 0.1, 1.05) + circle(3.4, -len - 0.1, 1.05);
    return [
      ...[[0, 6.6], [90, 6.4], [180, 8.6], [270, 6.4]].flatMap(([a, len]) => [fill(xf(arm(len), at(12, 8.6, a))), fill(xf(pearls(len), at(12, 8.6, a)))]),
      fill(circle(12, 8.6, 2.3)),
      fill(ellipse(12, 19.6, 2.5, 1.3) + poly([[11, 20.6], [13, 20.6], [12.4, 23.6], [11.6, 23.6]])),
    ];
  },
  germany: () => [
    // double-headed eagle displayed
    fill(EAGLE.wing), fill(mx12(EAGLE.wing)),
    fill(EAGLE.head), fill(mx12(EAGLE.head)),
    fill(EAGLE.body),
    fill(EAGLE.leg), fill(mx12(EAGLE.leg)),
    fill(EAGLE.tail),
  ],
  celts: () => [
    // torc: open twisted ring with buffer terminals
    fill(band(12, 10.4, 5.6, 9.2, -140, 140)),
    ...[-1, 1].map((s) => fill(circle(...polar(12, 10.4, 7.8, s * 147), 2.8))),
  ],
  mongols: () => {
    const limb = 'M11.4 12C10.8 8.4 13.2 5.6 17.4 4.4C18.2 3.6 18 2.6 17.2 1.8';
    const vane = poly([[13.2, 11.3], [14.3, 9.5], [17, 9.5], [16.6, 11.3]]);
    return [
      strk(limb + my12(limb), 2.3, 'round'),
      strk('M17.5 2.2V21.8', 0.9),
      fill(rect(4.4, 11.3, 13.4, 1.4) + poly([[1.4, 12], [6.2, 9.2], [5.4, 12], [6.2, 14.8]])),
      fill(vane + my12(vane)),
    ];
  },
  sparta: () => [
    // Corinthian helmet in profile: horsehair crest over the dome, eye and cheek slit knocked out
    fill('M4.3 9.9C3.7 4.2 8.6 1 12.8 1C18.2 1 22.6 5.4 22.8 11.4C22.9 14.6 22.4 16.8 21.6 18C21.4 15.4 21 13 20.4 11.6C19.6 6.8 16.6 3.9 12.4 3.9C9.2 3.9 6.6 5.9 5.9 9.5Z'),
    fill('M5.4 13.2C5.1 8.4 8.2 5.1 12.4 5.1C16.6 5.1 19.1 8.4 19.1 13C19.1 15.6 19.6 17.4 21.2 18.8L20.8 20.1C18.6 19.9 16.6 19.1 14.9 18.2C14 20.7 12.1 22.6 8.9 23.4L10 17.3L9.7 15.6L13.6 14.2L7.9 12.4L7.6 15.4L8 20.6L6.3 17.2Z'),
  ],
  athens: () => [
    // owl full face: big ringed eyes, beak, folded body on a perch
    fill('M12 1.8C16 1.8 18.6 4.2 18.6 7.4C18.6 10.6 15.8 12.4 12 12.4C8.2 12.4 5.4 10.6 5.4 7.4C5.4 4.2 8 1.8 12 1.8Z' + circle(9.2, 7.2, 2.5) + circle(14.8, 7.2, 2.5) + circle(9.2, 7.2, 1.05) + circle(14.8, 7.2, 1.05) + poly([[12, 8.6], [11.1, 9.9], [12, 11.8], [12.9, 9.9]])),
    fill('M7.2 13.2C8.6 13.8 10.2 14.1 12 14.1C13.8 14.1 15.4 13.8 16.8 13.2C17.6 15.4 17.4 17.8 16.4 19.4C15.4 20.6 13.8 21.1 12 21.1C10.2 21.1 8.6 20.6 7.6 19.4C6.6 17.8 6.4 15.4 7.2 13.2Z'),
    fill(rect(4, 21.6, 16, 1.5)),
  ],
  aztecs: () => [
    // the New Fire: flame, fire bowl, bound reed bundle, temple steps
    fill(xf(FLAME, at(12, 10.6, 0, 0.037))),
    fill(poly([[7.2, 11.4], [16.8, 11.4], [15.4, 13.6], [8.6, 13.6]])),
    fill(poly([[9.3, 14.3], [14.7, 14.3], [14.4, 16.2], [9.6, 16.2]]) + poly([[9.7, 16.9], [14.3, 16.9], [14, 18.9], [10, 18.9]])),
    fill(rect(6.6, 19.6, 10.8, 1.6) + rect(4, 21.2, 16, 1.8)),
  ],
  egypt: () => [
    fill(circle(12, 4.6, 2.3)),
    fill('M9.6 5.1L2 6.4L2.7 7.5L9.6 6.9ZM14.4 5.1L22 6.4L21.3 7.5L14.4 6.9Z'),
    fill('M3.4 22L4.4 10.6H10.2V22ZM13.8 22V10.6H19.6L20.6 22Z'),
    fill('M4.2 10.9V9.8Q4.2 9 2.6 7.8H21.4Q19.8 9 19.8 9.8V10.9Z'),
    fill(rect(9.8, 13.2, 4.4, 1.6)),
    fill(rect(2.2, 21.8, 19.6, 1.4)),
  ],
  rome: () => [
    // legion standard: spearhead, crossbar with pendants, wreath, one phalera
    fill(poly([[12, 0.3], [13.9, 3.9], [10.1, 3.9]])),
    fill(rect(11.2, 3.6, 1.6, 20.2)),
    fill(rect(4.6, 4.7, 14.8, 1.6) + rect(4.8, 6.3, 1.6, 3.9) + rect(17.6, 6.3, 1.6, 3.9)),
    fill(ring(12, 10.4, 1.7, 3.9)),
    fill(circle(12, 17.4, 3.1)),
  ],
};

for (const [key, draw] of Object.entries(GLYPHS)) {
  if (only && key !== only) continue;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" fill-rule="evenodd">${draw().join('')}</svg>\n`;
  writeFileSync(`${OUT}glyph-${key}.svg`, svg);
}
