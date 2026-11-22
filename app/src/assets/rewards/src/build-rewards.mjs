#!/usr/bin/env node
// Generates the reward emblems: app/src/assets/rewards/emblem-<kind>.svg
//
//   node app/src/assets/rewards/src/build-rewards.mjs [kind]
//
// Same engraving language as the faction seals (heraldry/src/kit.mjs): one ink in currentColor, solid
// relief, half-tones and hatching, with engraved cuts and hatched shadow knocked out through a mask,
// so an emblem sits on any background and takes any tint. The difference is the frame: an emblem is a
// bare charge with no rim (the card's medallion ring is its frame), so it can never be mistaken for a
// faction seal. Detail is drawn for 96 px and up; the charge, not the hatching, carries it at 56 px.
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { f, polar, poly, line, circle, ellipse, rect, arc, star4, star4Ridges, leaf, T, xf, tp, C, Seal, laurel } from '../../heraldry/src/kit.mjs';

const OUT = fileURLToPath(new URL('..', import.meta.url));

/** A flat-top hex, drawn in plan view squashed to a ground plate: its vertices, centre (cx, cy), radius R. */
const hexPts = (cx, cy, R, k = 0.42) => Array.from({ length: 6 }, (_, i) => [cx + R * Math.cos((i * Math.PI) / 3), cy + R * k * Math.sin((i * Math.PI) / 3)]);

/** The claimed hex as a ground plate with a thick edge: the common footing of founded and growth. */
function plate(S, cy = 384, R = 168, depth = 22) {
  const top = hexPts(C, cy, R);
  const low = top.map(([x, y]) => [x, y + depth]);
  const inner = (k) => top.map(([x, y]) => [C + (x - C) * k, cy + (y - cy) * k]);
  // the front faces: vertices 0 right, 1 lower right, 2 lower left, 3 left
  S.half(poly(low.slice(0, 4).concat([top[3], top[2], top[1], top[0]])), { op: 0.62 });
  S.cut([0, 1, 2, 3].map((i) => line(top[i], low[i])).join(''), 2);
  S.shade(poly([top[0], top[1], low[1], low[0]]), { a: 60, s: 3.4 });
  S.shade(poly([top[1], top[2], low[2], low[1]]), { a: 90, s: 5, w: 1 });
  S.solid(poly(top), { halo: 3 });
  // ground on the plate: faint tone, long horizontal engraving, and a fine inner rule
  S.cutFill(poly(inner(0.965)));
  S.tone(poly(top), { op: 0.22 });
  S.hatch(poly(top), { a: 0, s: 5, w: 1.2, op: 0.5 });
  S.solid(poly(inner(0.965)) + poly(inner(0.935)));
}

// ---------- camp: a broken standard over cold embers ----------
function camp(S) {
  // the hearth ring: back stones, ash with a few dead embers, then the front stones
  const at = (a) => [C + 104 * Math.sin((a * Math.PI) / 180), 396 - 24 * Math.cos((a * Math.PI) / 180)];
  const back = [], front = [];
  for (let a = 0; a < 360; a += 30) (Math.cos((a * Math.PI) / 180) > 0.01 ? back : front).push(at(a));
  const stones = (arr) => arr.map(([x, y]) => ellipse(x, y, 20, 13)).join('');
  S.solid(stones(back), { halo: 2.5 });
  S.tone(ellipse(C, 396, 90, 19), { op: 0.35 });
  S.hatch(ellipse(C, 396, 88, 18), { a: 0, s: 4.2, w: 1.3, op: 0.6 });
  S.half(Array.from({ length: 5 }, (_, k) => poly([[C - 56 + k * 28, 392 + (k % 2) * 6], [C - 50 + k * 28, 386 + (k % 2) * 6], [C - 44 + k * 28, 392 + (k % 2) * 6], [C - 50 + k * 28, 398 + (k % 2) * 6]])).join(''), { op: 0.75 });
  S.solid(stones(front), { halo: 3 });
  for (const [x, y] of [...back, ...front]) S.cut(`M${f(x - 11)} ${f(y - 2)}Q${f(x)} ${f(y - 10)} ${f(x + 11)} ${f(y - 2)}`, 1.8);
  for (const [x, y] of front) S.shade(ellipse(x + 5, y + 6, 12, 5), { a: 0, s: 3, w: 1.4 });
  // the standard: a pole snapped short, a crossbar broken on one side
  S.solid(poly([[245, 404], [245, 160], [249, 148], [253, 158], [258, 134], [263, 156], [268, 146], [269, 160], [269, 404]]), { halo: 3 });
  S.cut('M257 176V398', 1.5);
  S.shade(poly([[261, 166], [269, 160], [269, 400], [261, 400]]), { a: 90, s: 2.8 });
  S.solid(poly([[168, 178], [300, 178], [306, 184], [299, 189], [304, 196], [168, 196]]) + circle(168, 187, 10), { halo: 3 });
  S.cut('M180 187H290', 1.3);
  // torn banner on the left arm, strips on the right
  S.solid('M176 196H245V262L236 290L228 266L218 308L208 270L196 296L188 262L176 276Z', { halo: 3 });
  S.shade('M176 196H194V278L176 276Z', { a: 90, s: 3.4 });
  for (const x of [196, 212, 228]) S.cut(`M${x} 202V${250 + (x % 9) * 4}`, 1.3);
  S.cut('M196 218L226 252M226 218L196 252', 3.4);
  S.solid('M269 196H292V250L283 240L279 272L271 244L269 262Z', { halo: 2.5 });
  S.shade('M269 196H278V258L269 262Z', { a: 90, s: 3 });
  // the fallen top of the pole, blade-first, leaning on the stones
  const m = T.c(T.t(338, 340), T.r(-26));
  S.solid(xf(rect(-62, -8, 124, 16) + poly([[62, -8], [92, 0], [62, 8]]) + 'M66 -8Q92 -30 110 -20Q98 -8 106 6Q84 8 66 8Z', m), { halo: 2.8 });
  S.cut(xf('M-56 0H62', m), 1.4);
  S.shade(xf(rect(-62, 0, 124, 8), m), { a: 90, s: 2.8 });
  S.cut(xf('M-66 -8L-58 0L-66 8', m), 1.6);
}

// ---------- ruin: a broken arch above buried stones ----------
function ruin(S) {
  const CX = C, CY = 236, RI = 78, RO = 134;
  // ground and rubble
  S.solid(rect(98, 414, 316, 14), { halo: 0 });
  S.hatch(rect(98, 414, 316, 14), { a: 0, s: 3, w: 1.1, op: 0.6 });
  // the left pier, with its capital, standing whole
  S.solid(rect(122, 236, 56, 178) + rect(114, 228, 72, 12) + rect(118, 408, 64, 8), { halo: 3 });
  S.cut('M114 234H186M122 400H178', 1.4);
  for (const y of [262, 292, 322, 352, 382]) S.cut(`M122 ${y}H178`, 1.2);
  for (const [y, x] of [[262, 150], [292, 136], [322, 158], [352, 140], [382, 152]]) S.cut(`M${x} ${y}v30`, 1.1);
  S.shade(rect(166, 240, 12, 170), { a: 90, s: 2.6 });
  // the right pier, shattered at the shoulder
  S.solid(poly([[334, 414], [334, 340], [340, 330], [347, 338], [356, 322], [364, 336], [374, 328], [390, 344], [390, 414]]) + rect(326, 408, 72, 8), { halo: 3 });
  for (const y of [362, 392]) S.cut(`M334 ${y}H390`, 1.2);
  S.cut('M352 342v20M368 364v28M346 394v16', 1.1);
  S.shade(rect(378, 346, 12, 66), { a: 90, s: 2.6 });
  // the arch: springs from the left pier, runs over the top and breaks before the right pier
  const outer = [], inner = [];
  const A0 = -90, A1 = 36; // compass degrees, clockwise from up
  for (let a = A0; a <= A1; a += 6) { outer.push(polar(CX, CY, RO, a)); inner.push(polar(CX, CY, RI, a)); }
  const brk = [polar(CX, CY, RO, A1 + 3), polar(CX, CY, RO - 14, A1 + 12), polar(CX, CY, RO - 26, A1 + 4), polar(CX, CY, RO - 38, A1 + 14), polar(CX, CY, RO - 46, A1 + 6), polar(CX, CY, RI + 4, A1 + 10)];
  S.solid(poly([...outer, ...brk, ...inner.reverse()]), { halo: 3 });
  for (let a = A0 + 12; a <= A1 - 2; a += 12) S.cut(line(polar(CX, CY, RI + 1, a), polar(CX, CY, RO - 1, a)), 1.8);
  S.cut(arc(CX, CY, (RI + RO) / 2, A0 + 2, A1 - 2), 1.1);
  S.shade(poly([...Array.from({ length: 9 }, (_, i) => polar(CX, CY, RI + 12, A0 + i * 14)), ...Array.from({ length: 9 }, (_, i) => polar(CX, CY, RI, A0 + (8 - i) * 14))]), { a: 45, s: 3 });
  // A plain dressed-stone crown and rubble keep the site historical.
  S.solid(poly([[234, 102], [278, 102], [270, 128], [242, 128]]), { halo: 1.5 });
  S.cut('M241 116H271', 1.2);
  // Buried blocks and a toppled column replace the old relay-star relic.
  S.solid(poly([[206, 382], [216, 356], [244, 352], [254, 382]]) + rect(270, 370, 58, 28) + rect(286, 398, 64, 16), { halo: 2.4 });
  S.cut('M216 368H247M278 380H322M294 406H342', 1.3);
  S.shade(rect(314, 372, 12, 24), { a: 90, s: 2.5 });
  // fallen blocks at the right foot
  S.solid(poly([[300, 414], [306, 396], [330, 392], [338, 414]]) + poly([[104, 414], [108, 402], [128, 400], [132, 414]]), { halo: 2.4 });
  S.cut('M306 402H334M110 408H128', 1.3);
}

// ---------- captured: a gate with the banner raised over it ----------
function captured(S) {
  const merlons = (x0, x1, y) => {
    let d = '';
    for (let x = x0; x + 14 <= x1 + 0.1; x += 28) d += rect(x, y - 18, 14, 19);
    return d;
  };
  // curtain wall and its merlons
  S.solid(rect(186, 282, 140, 130) + merlons(190, 326, 282), { halo: 3 });
  for (let y = 300; y < 412; y += 16) S.cut(`M186 ${y}H326`, 1);
  for (let y = 308, k = 0; y < 412; y += 16, k++) for (let x = 190 + (k % 2) * 12; x < 326; x += 24) S.cut(`M${x} ${y}v8`, 1);
  S.cut('M186 288H326', 1.5);
  // the two towers
  for (const x0 of [96, 326]) {
    S.solid(rect(x0, 212, 90, 200) + rect(x0 - 6, 202, 102, 12) + merlons(x0 - 2, x0 + 94, 202), { halo: 3 });
    S.cut(`M${x0 - 6} 208H${x0 + 96}`, 1.4);
    for (let y = 232; y < 410; y += 16) S.cut(`M${x0} ${y}H${x0 + 90}`, 1);
    for (let y = 240, k = 0; y < 410; y += 16, k++) for (let x = x0 + 12 + (k % 2) * 14; x < x0 + 88; x += 28) S.cut(`M${x} ${y}v8`, 1);
    S.cutFill(rect(x0 + 40, 252, 10, 34));
    S.shade(rect(x0 + (x0 < 200 ? 66 : 4), 214, 20, 198), { a: 90, s: 3.2, w: 1.2 });
  }
  // the gate: round arch, portcullis half raised
  const gate = 'M216 412V350A40 40 0 0 1 296 350V412Z';
  S.cutFill(gate);
  S.stroke('M212 412V350A44 44 0 0 1 300 350V412', 5);
  for (let a = -78; a <= 78; a += 26) S.cut(line(polar(256, 350, 41, a), polar(256, 350, 52, a)), 1.6);
  const bars = Array.from({ length: 5 }, (_, i) => `M${228 + i * 14} 332V${388}`).join('');
  S.stroke(bars + 'M222 346H290M222 366H290M222 386H290', 3.2, { cap: 'butt' });
  S.solid(Array.from({ length: 5 }, (_, i) => poly([[225 + i * 14, 388], [231 + i * 14, 388], [228 + i * 14, 400]])).join(''), { halo: 0 });
  S.half(rect(216, 400, 80, 12), { op: 0.8 });
  // the banner raised on its staff above the gate
  S.solid(rect(253, 78, 6, 206) + circle(256, 76, 7), { halo: 3 });
  S.solid(star4(256, 58, 15, 3.8), { halo: 2.4 });
  const flag = 'M259 90H378L354 118L378 146H259Z';
  S.solid(flag, { halo: 3 });
  S.cut('M263 96H366M263 140H366', 1.1);
  S.shade('M259 90H280V146H259Z', { a: 90, s: 3 });
  S.cutFill(star4(307, 118, 20, 5));
  S.solid(star4(307, 118, 13, 3.3));
}

// ---------- founded: the cornerstone set on its hex ----------
function founded(S) {
  plate(S, 392, 172, 22);
  // the cube, drawn isometrically: top, left and right faces
  const a = 88, b = 50, H = 84, cy = 384;
  const top = [[C - a, cy - H], [C, cy - H - b], [C + a, cy - H], [C, cy - H + b]];
  const L = [[C - a, cy - H], [C, cy - H + b], [C, cy + b], [C - a, cy]];
  const R = [[C, cy - H + b], [C + a, cy - H], [C + a, cy], [C, cy + b]];
  S.cutFill(poly(L) + poly(R) + poly(top));
  S.half(poly(L), { halo: 2, op: 0.78 });
  S.shade(poly(L), { a: 90, s: 5, w: 1 });
  S.solid(poly(R), { halo: 3 });
  S.shade(poly(R), { a: 90, s: 3.4, w: 1.3 });
  S.solid(poly(top), { halo: 3 });
  S.cutFill(poly(top.map(([x, y]) => [C + (x - C) * 0.92, cy - H + (y - (cy - H)) * 0.92])));
  S.tone(poly(top.map(([x, y]) => [C + (x - C) * 0.92, cy - H + (y - (cy - H)) * 0.92])), { op: 0.2 });
  S.hatch(poly(top), { a: 27, s: 5, w: 1, op: 0.5 });
  // the relay star carved into the top face
  const m = T.c(T.t(C, cy - H), [1, 0, 0, b / a, 0, 0], T.r(45));
  const star = xf(star4(0, 0, 66, 15), m);
  S.cutFill(xf(star4(0, 0, 74, 18), m));
  S.solid(star, { halo: 1.5 });
  S.cut(xf(star4Ridges(0, 0, 66), m), 1.5);
  // survey stakes at three corners of the hex
  for (const [x, y] of [[C - 156, 392], [C + 156, 392]]) {
    S.solid(rect(x - 2.5, y - 40, 5, 40) + poly([[x + 2.5, y - 40], [x + 24, y - 34], [x + 2.5, y - 26]]), { halo: 2 });
  }
}

// ---------- growth: a sprout rising from its hex ----------
function growth(S) {
  plate(S, 392, 172, 22);
  // soil mound
  S.solid('M196 392Q256 362 316 392Q256 404 196 392Z', { halo: 2.4 });
  S.hatch('M196 392Q256 362 316 392Q256 404 196 392Z', { a: 20, s: 3.4, w: 1.1, op: 0.55 });
  // stem
  S.solid('M251 384C249 330 253 270 256 218C259 270 263 330 261 384Z', { halo: 2.4 });
  // two great leaves and a pair of young ones
  const pairs = [
    [[256, 330], -62, 150, 44, -10],
    [[256, 330], 62, 150, 44, 10],
    [[256, 268], -34, 104, 31, -6],
    [[256, 268], 34, 104, 31, 6],
  ];
  for (const [b, ang, L, W, bend] of pairs) {
    const lf = leaf(b, ang, L, W, bend);
    S.solid(lf.d, { halo: 3 });
    S.cut(lf.rib, 1.6);
    const half = leaf(b, ang, L, W, bend);
    S.shade(half.d, { a: ang < 0 ? 60 : -60, s: 4.4, w: 1.1 }, {});
    S.cut(lf.rib, 1.6);
  }
  // the bud, and the relay star that crowns it
  S.solid('M256 222C240 200 244 168 256 146C268 168 272 200 256 222Z', { halo: 2.6 });
  S.cut('M256 214V158', 1.4);
  S.solid(star4(256, 118, 26, 6.4), { halo: 3 });
  S.cut(star4Ridges(256, 118, 26), 1.2);
}

// ---------- milestone: laurel and relay star; great work: laurel and monument ----------
function wreath(S) {
  // heavier than the seal laurel: fewer, broader leaves on a thick stem, so it holds as a shape at 68 px
  laurel(S, 152, 198, 9, 11.5, 50, C + 4, { stem: 9, W: 13, rib: 2, berries: false });
  // ribbon tied at the foot of the wreath
  S.solid('M256 418C240 424 226 438 214 456L232 450L238 468L256 440Z' + 'M256 418C272 424 286 438 298 456L280 450L274 468L256 440Z', { halo: 2.4 });
  S.cut('M228 446L248 430M284 446L264 430', 1.2);
  S.solid(ellipse(256, 426, 12, 9), { halo: 2.4 });
}

function milestone(S) {
  wreath(S);
  S.solid(star4(256, 246, 118, 28), { halo: 4 });
  S.cut(star4Ridges(256, 246, 118), 1.8);
  S.shade(poly([[256, 246], polar(256, 246, 118, 0), polar(256, 246, 28, 45), polar(256, 246, 118, 90), polar(256, 246, 28, 135), polar(256, 246, 118, 180)]), { a: 45, s: 3.6, w: 1.4 });
  S.cutFill(circle(256, 246, 18));
  S.solid(circle(256, 246, 11));
  S.hatch(star4(256, 246, 118, 28), { a: 0, s: 5, w: 1, op: 0.3 });
}

function greatwork(S) {
  wreath(S);
  // the monument: stepped plinth, a tapering shaft and its pyramidion, the relay star above
  S.solid(rect(208, 380, 96, 16) + rect(222, 362, 68, 20) + rect(234, 344, 44, 20), { halo: 3 });
  S.cut('M208 388H304M222 372H290M234 354H278', 1.3);
  S.shade(rect(290, 380, 14, 16) + rect(278, 362, 12, 20), { a: 90, s: 3 });
  S.solid('M232 346L242 196H270L280 346Z' + poly([[240, 196], [256, 156], [272, 196]]), { halo: 3 });
  for (const t of [0.25, 0.5, 0.75]) S.cut(`M${f(232 + 10 * t)} ${f(346 - 150 * t)}H${f(280 - 10 * t)}`, 1.1);
  S.cut('M256 200V342', 1.2);
  S.shade('M258 200H270L280 346H258Z', { a: 90, s: 3.2 });
  S.solid(star4(256, 118, 40, 9.6), { halo: 3 });
  S.cut(star4Ridges(256, 118, 40), 1.3);
  for (let a = -50; a <= 50; a += 25) if (a) S.stroke(line(polar(256, 118, 52, a), polar(256, 118, 76, a)), 1.8, { op: 0.6 });
}

// ---------- tech: the relay star within its orbits ----------
function tech(S) {
  for (const rot of [-32, 32]) {
    const m = T.c(T.t(C, C), T.r(rot));
    S.stroke(xf(ellipse(0, 0, 192, 66), m), 6, { halo: 0 });
    S.cut(xf(ellipse(0, 0, 192, 66), m), 1.2);
  }
  for (const [rot, ang] of [[-32, 150], [32, 30]]) {
    const m = T.c(T.t(C, C), T.r(rot));
    const [x, y] = tp(m, 192 * Math.cos((ang * Math.PI) / 180), 66 * Math.sin((ang * Math.PI) / 180));
    S.solid(circle(x, y, 17), { halo: 4 });
    S.cut(circle(x, y, 8), 1.3);
  }
  S.solid(star4(C, C, 160, 36), { halo: 5 });
  S.cut(star4Ridges(C, C, 160), 2);
  S.shade(poly([[C, C], polar(C, C, 160, 0), polar(C, C, 36, 45), polar(C, C, 160, 90), polar(C, C, 36, 135), polar(C, C, 160, 180)]), { a: 45, s: 3.6, w: 1.4 });
  S.cutFill(circle(C, C, 22));
  S.solid(circle(C, C, 14));
  for (let a = 22.5; a < 360; a += 45) S.stroke(line(polar(C, C, 66, a), polar(C, C, 112, a)), 2, { op: 0.6 });
}

const EMBLEMS = {
  camp: ['The raiders’ camp: a broken standard over cold embers', camp, [60, 100, 392]],
  ruin: ['An ancient ruin: a broken arch above buried stones', ruin, [84, 84, 344]],
  captured: ['A city taken: a gate with the banner raised', captured, [66, 38, 380]],
  founded: ['A city founded: the cornerstone set on its hex', founded, [60, 90, 392]],
  growth: ['A city grows: a sprout rising from its hex', growth, [60, 90, 392]],
  milestone: ['A milestone: the relay star in a laurel', milestone, [51, 60, 410]],
  greatwork: ['A great work: a monument in a laurel', greatwork, [38, 34, 436]],
  tech: ['A technology: the relay star within its orbits', tech],
};

const only = process.argv[2];
for (const [kind, [title, draw, box = [0, 0, 512]]] of Object.entries(EMBLEMS)) {
  if (only && kind !== only) continue;
  const S = new Seal(`emblem-${kind}`);
  draw(S);
  writeFileSync(`${OUT}emblem-${kind}.svg`, S.svg(title).replace('viewBox="0 0 512 512"', `viewBox="${box[0]} ${box[1]} ${box[2]} ${box[2]}"`));
}
