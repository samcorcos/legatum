// Shared geometry, path transforms and the single-ink engraving canvas (Seal) used by the seal and
// reward-emblem generators. Pure helpers: importing this file writes nothing.
// ---------- geometry ----------
export const f = (n) => {
  const v = Math.round(n * 100) / 100;
  return Object.is(v, -0) ? 0 : v;
};
export const rad = (d) => (d * Math.PI) / 180;
/** Point at compass angle `a` (0 = up, clockwise) and radius r. */
export const polar = (cx, cy, r, a) => [cx + r * Math.sin(rad(a)), cy - r * Math.cos(rad(a))];
export const pt = ([x, y]) => `${f(x)} ${f(y)}`;
export const poly = (pts, close = true) => 'M' + pts.map(pt).join('L') + (close ? 'Z' : '');
export const line = (a, b) => `M${pt(a)}L${pt(b)}`;
export const circle = (cx, cy, r) =>
  `M${f(cx - r)} ${f(cy)}A${f(r)} ${f(r)} 0 1 0 ${f(cx + r)} ${f(cy)}A${f(r)} ${f(r)} 0 1 0 ${f(cx - r)} ${f(cy)}Z`;
export const ellipse = (cx, cy, rx, ry) =>
  `M${f(cx - rx)} ${f(cy)}A${f(rx)} ${f(ry)} 0 1 0 ${f(cx + rx)} ${f(cy)}A${f(rx)} ${f(ry)} 0 1 0 ${f(cx - rx)} ${f(cy)}Z`;
export const ring = (cx, cy, r1, r2) => circle(cx, cy, r2) + circle(cx, cy, r1);
export const rect = (x, y, w, h) => `M${f(x)} ${f(y)}H${f(x + w)}V${f(y + h)}H${f(x)}Z`;
/** Open circular arc between compass angles a0 -> a1 (clockwise when a1 > a0). */
export const arc = (cx, cy, r, a0, a1) =>
  `M${pt(polar(cx, cy, r, a0))}A${f(r)} ${f(r)} 0 ${Math.abs(a1 - a0) > 180 ? 1 : 0} ${a1 > a0 ? 1 : 0} ${pt(polar(cx, cy, r, a1))}`;
/** The shared four-point mark: a four-point star (proportions follow the logo's star). */
export const star4 = (cx, cy, R, r = R * 0.24, rot = 0) =>
  poly(Array.from({ length: 8 }, (_, k) => polar(cx, cy, k % 2 ? r : R, rot + k * 45)));
export const star4Ridges = (cx, cy, R, rot = 0) =>
  [0, 90].map((a) => line(polar(cx, cy, R * 0.82, rot + a), polar(cx, cy, R * 0.82, rot + a + 180))).join('');
/** Almond leaf / feather from base b along compass angle a. */
export function leaf(b, a, L, W, bend = 0) {
  const u = [Math.sin(rad(a)), -Math.cos(rad(a))];
  const n = [-u[1], u[0]];
  const tip = [b[0] + u[0] * L + n[0] * bend, b[1] + u[1] * L + n[1] * bend];
  const m = [b[0] + u[0] * L * 0.45, b[1] + u[1] * L * 0.45];
  const c1 = [m[0] + n[0] * W, m[1] + n[1] * W];
  const c2 = [m[0] - n[0] * W + n[0] * bend, m[1] - n[1] * W + n[1] * bend];
  return {
    d: `M${pt(b)}Q${pt(c1)} ${pt(tip)}Q${pt(c2)} ${pt(b)}Z`,
    rib: `M${pt([b[0] + u[0] * L * 0.12, b[1] + u[1] * L * 0.12])}Q${pt([m[0] + n[0] * bend * 0.3, m[1] + n[1] * bend * 0.3])} ${pt([b[0] + u[0] * L * 0.8 + n[0] * bend * 0.7, b[1] + u[1] * L * 0.8 + n[1] * bend * 0.7])}`,
  };
}

// ---------- affine transforms of path data (absolute M L H V A C Q Z only) ----------
export const mul = (A, B) => [
  A[0] * B[0] + A[2] * B[1],
  A[1] * B[0] + A[3] * B[1],
  A[0] * B[2] + A[2] * B[3],
  A[1] * B[2] + A[3] * B[3],
  A[0] * B[4] + A[2] * B[5] + A[4],
  A[1] * B[4] + A[3] * B[5] + A[5],
];
export const T = {
  t: (x, y) => [1, 0, 0, 1, x, y],
  r: (deg) => [Math.cos(rad(deg)), Math.sin(rad(deg)), -Math.sin(rad(deg)), Math.cos(rad(deg)), 0, 0],
  s: (k) => [k, 0, 0, k, 0, 0],
  mx: (cx = 256) => [-1, 0, 0, 1, 2 * cx, 0],
  /** compose(A, B, C) applies C first. */
  c: (...ms) => ms.reduce((acc, m) => mul(acc, m)),
};
export function xf(d, m) {
  const det = m[0] * m[3] - m[1] * m[2];
  const s = Math.sqrt(Math.abs(det));
  const rot = (Math.atan2(m[1], m[0]) * 180) / Math.PI;
  const P = (x, y) => [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
  const tk = d.match(/[A-Za-z]|-?\d*\.?\d+(?:e-?\d+)?/g);
  let i = 0;
  let cmd = '';
  let cx = 0;
  let cy = 0;
  let out = '';
  const n = () => parseFloat(tk[i++]);
  while (i < tk.length) {
    if (/[A-Za-z]/.test(tk[i])) cmd = tk[i++];
    switch (cmd) {
      case 'M':
      case 'L': {
        cx = n();
        cy = n();
        out += cmd + pt(P(cx, cy));
        if (cmd === 'M') cmd = 'L';
        break;
      }
      case 'H':
        cx = n();
        out += 'L' + pt(P(cx, cy));
        break;
      case 'V':
        cy = n();
        out += 'L' + pt(P(cx, cy));
        break;
      case 'A': {
        const [rx, ry, r0, la, sw] = [n(), n(), n(), n(), n()];
        cx = n();
        cy = n();
        const r1 = det < 0 ? rot - r0 : rot + r0;
        out += `A${f(rx * s)} ${f(ry * s)} ${f(((r1 % 180) + 180) % 180)} ${la} ${det < 0 ? 1 - sw : sw} ${pt(P(cx, cy))}`;
        break;
      }
      case 'C': {
        const p = [n(), n(), n(), n(), n(), n()];
        cx = p[4];
        cy = p[5];
        out += `C${pt(P(p[0], p[1]))} ${pt(P(p[2], p[3]))} ${pt(P(p[4], p[5]))}`;
        break;
      }
      case 'Q': {
        const p = [n(), n(), n(), n()];
        cx = p[2];
        cy = p[3];
        out += `Q${pt(P(p[0], p[1]))} ${pt(P(p[2], p[3]))}`;
        break;
      }
      case 'Z':
      case 'z':
        out += 'Z';
        cmd = '';
        break;
      default:
        throw new Error(`xf: unsupported command ${cmd} in ${d}`);
    }
  }
  return out;
}
export const tp = (m, x, y) => [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
export const mirror = (d) => xf(d, T.mx());
export const both = (d) => d + mirror(d);

// ---------- the seal canvas ----------
export const C = 256; // center
export const HALO = 0.62; // global weight of the separations between overlapping relief
export const FIELD = 186; // radius of the charge field inside the rim

export class Seal {
  constructor(key) {
    this.key = key;
    this.body = [];
    this.mask = [];
    this.defs = new Map();
  }
  id(n) {
    return `${this.key}-${n}`;
  }
  pattern(kind, a, s, w, op) {
    const name = `${kind}${a}-${s}-${w}-${op}`.replace(/\./g, '_').replace(/-(?=-)/g, 'm');
    const id = this.id(name);
    if (!this.defs.has(id)) {
      const stroke = kind === 'h' ? `stroke="currentColor" stroke-opacity="${op}"` : 'stroke="#000"';
      this.defs.set(
        id,
        `<pattern id="${id}" patternUnits="userSpaceOnUse" width="${s}" height="${s}" patternTransform="rotate(${a})"><path d="M-1 ${s / 2}H${s + 1}" ${stroke} stroke-width="${w}"/></pattern>`,
      );
    }
    return id;
  }
  clip(name, d) {
    this.defs.set(this.id(name), `<clipPath id="${this.id(name)}"><path d="${d}"/></clipPath>`);
    return name;
  }
  at(o) {
    return o.clip ? ` clip-path="url(#${this.id(o.clip)})"` : '';
  }
  halo(d, o, stroke) {
    if (!o.halo) return;
    this.mask.push(`<path d="${d}" fill="none" stroke="#000" stroke-width="${f((stroke || 0) + 2 * o.halo * HALO)}" stroke-linejoin="round"${this.at(o)}/>`);
    this.mask.push(
      stroke
        ? `<path d="${d}" fill="none" stroke="#fff" stroke-width="${stroke}"${this.at(o)}/>`
        : `<path d="${d}" fill="#fff"${this.at(o)}/>`,
    );
  }
  /** Full-strength relief. */
  solid(d, o = {}) {
    this.halo(d, o);
    this.body.push(`<path d="${d}"${this.at(o)}/>`);
    return this;
  }
  /** Half-strength relief. */
  half(d, o = {}) {
    this.halo(d, o);
    this.body.push(`<path d="${d}" fill-opacity="${o.op ?? 0.5}"${this.at(o)}/>`);
    return this;
  }
  /** Faint field tone. */
  tone(d, o = {}) {
    this.body.push(`<path d="${d}" fill-opacity="${o.op ?? 0.16}"${this.at(o)}/>`);
    return this;
  }
  /** Engraved hatching (bright lines) over a region. */
  hatch(d, { a = 45, s = 5, w = 1.1, op = 0.55 } = {}, o = {}) {
    this.body.push(`<path d="${d}" fill="url(#${this.pattern('h', a, s, w, op)})"${this.at(o)}/>`);
    return this;
  }
  /** Stroked relief line. */
  stroke(d, w, o = {}) {
    this.halo(d, o, w);
    const op = o.op !== undefined ? ` stroke-opacity="${o.op}"` : '';
    const cap = o.cap ? ` stroke-linecap="${o.cap}"` : '';
    this.body.push(`<path d="${d}" fill="none" stroke="currentColor" stroke-width="${w}"${op}${cap}${this.at(o)}/>`);
    return this;
  }
  /** Engraved cut: a line removed from everything drawn so far and later un-haloed parts. */
  cut(d, w = 1.8, o = {}) {
    const cap = o.cap ? ` stroke-linecap="${o.cap}"` : '';
    this.mask.push(`<path d="${d}" fill="none" stroke="#000" stroke-width="${w}"${cap}${this.at(o)}/>`);
    return this;
  }
  cutFill(d, o = {}) {
    this.mask.push(`<path d="${d}"${this.at(o)}/>`);
    return this;
  }
  /** Hatched shadow cut into relief. */
  shade(d, { a = 45, s = 4, w = 1.3 } = {}, o = {}) {
    this.mask.push(`<path d="${d}" fill="url(#${this.pattern('c', a, s, w, 1)})"${this.at(o)}/>`);
    return this;
  }
  svg(title) {
    const m = this.id('cut');
    return (
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="currentColor" fill-rule="evenodd" role="img" aria-label="${title}">` +
      `<defs>${[...this.defs.values()].join('')}` +
      `<mask id="${m}" maskUnits="userSpaceOnUse" x="0" y="0" width="512" height="512"><rect width="512" height="512" fill="#fff"/>${this.mask.join('')}</mask></defs>` +
      `<g mask="url(#${m})">${this.body.join('')}</g></svg>\n`
    );
  }
}


/** A laurel branch pair (mirrored) of `n` leaf pairs along a circle of radius R centred (C, cy). */
export function laurel(S, R, a0, n, step, L0, cy = C, { stem: stemW = 3, W = 7.2, rib: ribW = 1.1, berries: withBerries = true } = {}) {
  const branch = [];
  const ribs = [];
  const berries = [];
  for (let k = 0; k < n; k++) {
    const a = a0 + k * step;
    const p = polar(C, cy, R, a);
    const L = L0 - k * 1.1;
    for (const side of [-1, 1]) {
      const lf = leaf(p, a + 90 + side * 36, L, W, side * 2);
      branch.push(lf.d);
      ribs.push(lf.rib);
    }
    if (withBerries && k % 2 && k < n - 1) berries.push(circle(...polar(C, cy, R - 12, a + step / 2), 3.2));
  }
  const stem = arc(C, cy, R, a0 - 4, a0 + (n - 1) * step + 2);
  S.stroke(both(stem), stemW);
  S.solid(both(branch.join('') + berries.join('')), { halo: 1.6 });
  S.cut(both(ribs.join('')), ribW);
}
