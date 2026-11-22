// Deterministic noise for the painterly map: integer hashes, periodic value-noise tiles (sampled
// bilinearly, so the per-pixel cost is a few multiplies), and small texture canvases.

function hash3(i: number, j: number, s: number): number {
  let h = Math.imul(i, 374761393) + Math.imul(j, 668265263) + Math.imul(s, 982451653);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

/** Stable 0..1 per (a, b, salt). */
export const rnd = (a: number, b: number, s = 0) => hash3(a * 7 + 13, b * 11 + 5, s + 101);

export function hashString(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return h >>> 0;
}

/** A square periodic noise tile: `size` texels, octave lattice cells `cells` (each must divide size). */
export interface NoiseTile {
  size: number;
  data: Float32Array;
}

function periodicFbm(size: number, cells: number[], seed: number): NoiseTile {
  const data = new Float32Array(size * size);
  let norm = 0;
  let amp = 1;
  for (let o = 0; o < cells.length; o++) {
    const cell = cells[o];
    const n = size / cell; // lattice points per side (wraps)
    for (let y = 0; y < size; y++) {
      const fy = y / cell;
      const yi = Math.floor(fy);
      const yf = fy - yi;
      const v = yf * yf * (3 - 2 * yf);
      for (let x = 0; x < size; x++) {
        const fx = x / cell;
        const xi = Math.floor(fx);
        const xf = fx - xi;
        const u = xf * xf * (3 - 2 * xf);
        const s = seed + o * 17;
        const a = hash3(xi % n, yi % n, s);
        const b = hash3((xi + 1) % n, yi % n, s);
        const c = hash3(xi % n, (yi + 1) % n, s);
        const d = hash3((xi + 1) % n, (yi + 1) % n, s);
        data[y * size + x] += amp * (a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v);
      }
    }
    norm += amp;
    amp *= 0.5;
  }
  for (let i = 0; i < data.length; i++) data[i] /= norm;
  return { size, data };
}

/** Bilinear sample of a periodic tile at texel coordinates (any real numbers). */
export function sampleTile(t: NoiseTile, x: number, y: number): number {
  const s = t.size;
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const fx = x - xi;
  const fy = y - yi;
  const x0 = ((xi % s) + s) % s;
  const y0 = ((yi % s) + s) % s;
  const x1 = x0 + 1 === s ? 0 : x0 + 1;
  const y1 = y0 + 1 === s ? 0 : y0 + 1;
  const d = t.data;
  const a = d[y0 * s + x0];
  const b = d[y0 * s + x1];
  const c = d[y1 * s + x0];
  const e = d[y1 * s + x1];
  return a + (b - a) * fx + (c - a) * fy + (a - b - c + e) * fx * fy;
}

let dispX: NoiseTile | null = null;
let dispY: NoiseTile | null = null;
/** Coastline displacement noise (two channels), 256 texels, lattice cells 16/8/4. */
export function displacementTiles(): [NoiseTile, NoiseTile] {
  if (!dispX || !dispY) {
    dispX = periodicFbm(256, [16, 8, 4], 11);
    dispY = periodicFbm(256, [16, 8, 4], 29);
  }
  return [dispX, dispY];
}

/** Stretch an fbm tile's values to span roughly 0..1 around 0.5. */
function stretch(t: NoiseTile) {
  let lo = Infinity;
  let hi = -Infinity;
  for (const v of t.data) {
    lo = Math.min(lo, v);
    hi = Math.max(hi, v);
  }
  const k = 1 / Math.max(1e-6, hi - lo);
  for (let i = 0; i < t.data.length; i++) t.data[i] = (t.data[i] - lo) * k;
}

function canvas(w: number, h: number): HTMLCanvasElement | OffscreenCanvas {
  if (typeof OffscreenCanvas !== 'undefined') return new OffscreenCanvas(w, h);
  const c = document.createElement('canvas');
  c.width = w;
  c.height = h;
  return c;
}
export type AnyCanvas = HTMLCanvasElement | OffscreenCanvas;
export type Ctx2D = CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D;
export function makeCanvas(w: number, h: number): AnyCanvas {
  return canvas(Math.max(1, Math.ceil(w)), Math.max(1, Math.ceil(h)));
}
export function ctx2d(c: AnyCanvas): Ctx2D {
  return c.getContext('2d') as Ctx2D;
}

let mottle: AnyCanvas | null = null;
/** Low-frequency grayscale mottle (the paper/water "watercolor" variation). 256 texels, periodic. */
export function mottleCanvas(): AnyCanvas {
  if (mottle) return mottle;
  const t = periodicFbm(256, [64, 32, 16, 8], 9);
  stretch(t);
  const c = makeCanvas(256, 256);
  const g = ctx2d(c);
  const img = g.createImageData(256, 256);
  for (let i = 0; i < t.data.length; i++) {
    const v = Math.round(t.data[i] * 255);
    img.data[4 * i] = v;
    img.data[4 * i + 1] = v;
    img.data[4 * i + 2] = v;
    img.data[4 * i + 3] = 255;
  }
  g.putImageData(img, 0, 0);
  mottle = c;
  return c;
}

let grain: AnyCanvas | null = null;
/** Fine black-alpha grain, 256 px periodic, in device pixels. */
export function grainCanvas(): AnyCanvas {
  if (grain) return grain;
  const n = 256;
  const raw = new Float32Array(n * n);
  for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) raw[y * n + x] = hash3(x, y, 4);
  const c = makeCanvas(n, n);
  const g = ctx2d(c);
  const img = g.createImageData(n, n);
  for (let y = 0; y < n; y++)
    for (let x = 0; x < n; x++) {
      // A touch of smoothing (like turbulence at 0.9) so it reads as paper tooth, not TV static.
      const v =
        0.4 * raw[y * n + x] +
        0.15 * (raw[y * n + ((x + 1) % n)] + raw[((y + 1) % n) * n + x] + raw[y * n + ((x + n - 1) % n)] + raw[((y + n - 1) % n) * n + x]);
      const a = Math.max(0, Math.min(1, (v - 0.38) * 2.2));
      img.data[4 * (y * n + x) + 3] = Math.round(a * 255);
    }
  g.putImageData(img, 0, 0);
  grain = c;
  return c;
}

/**
 * Inverse of the standard normal CDF (probit), via a lookup table. The blurred land field of a
 * straight coast is Phi(d / sigma), so sigma * probit(field) is the distance to the coast.
 */
const PROBIT_N = 4096;
const probitLut = (() => {
  const lut = new Float32Array(PROBIT_N + 1);
  for (let i = 0; i <= PROBIT_N; i++) {
    const p = Math.min(1 - 1e-5, Math.max(1e-5, i / PROBIT_N));
    lut[i] = acklam(p);
  }
  return lut;
})();
export function probit(p: number): number {
  const f = (p < 0 ? 0 : p > 1 ? 1 : p) * PROBIT_N;
  const i = f | 0;
  if (i >= PROBIT_N) return probitLut[PROBIT_N];
  const t = f - i;
  return probitLut[i] + (probitLut[i + 1] - probitLut[i]) * t;
}
function acklam(p: number): number {
  const a = [-39.69683028665376, 220.9460984245205, -275.9285104469687, 138.357751867269, -30.66479806614716, 2.506628277459239];
  const b = [-54.47609879822406, 161.5858368580409, -155.6989798598866, 66.80131188771972, -13.28068155288572];
  const c = [-0.007784894002430293, -0.3223964580411365, -2.400758277161838, -2.549732539343734, 4.374664141464968, 2.938163982698783];
  const d = [0.007784695709041462, 0.3224671290700398, 2.445134137142996, 3.754408661907416];
  const lo = 0.02425;
  if (p < lo) {
    const q = Math.sqrt(-2 * Math.log(p));
    return (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1);
  }
  if (p > 1 - lo) {
    const q = Math.sqrt(-2 * Math.log(1 - p));
    return -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1);
  }
  const q = p - 0.5;
  const r = q * q;
  return ((((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q) / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1);
}

/** Separable gaussian blur of a scalar field in place (edges clamp). */
export function blurField(f: Float32Array, w: number, h: number, sigma: number) {
  const r = Math.max(1, Math.ceil(sigma * 3));
  const k = new Float32Array(2 * r + 1);
  let sum = 0;
  for (let i = -r; i <= r; i++) {
    k[i + r] = Math.exp(-(i * i) / (2 * sigma * sigma));
    sum += k[i + r];
  }
  for (let i = 0; i < k.length; i++) k[i] /= sum;
  const tmp = new Float32Array(Math.max(w, h));
  for (let y = 0; y < h; y++) {
    const o = y * w;
    for (let x = 0; x < w; x++) {
      let acc = 0;
      for (let i = -r; i <= r; i++) {
        const xx = x + i < 0 ? 0 : x + i >= w ? w - 1 : x + i;
        acc += f[o + xx] * k[i + r];
      }
      tmp[x] = acc;
    }
    f.set(tmp.subarray(0, w), o);
  }
  for (let x = 0; x < w; x++) {
    for (let y = 0; y < h; y++) {
      let acc = 0;
      for (let i = -r; i <= r; i++) {
        const yy = y + i < 0 ? 0 : y + i >= h ? h - 1 : y + i;
        acc += f[yy * w + x] * k[i + r];
      }
      tmp[y] = acc;
    }
    for (let y = 0; y < h; y++) f[y * w + x] = tmp[y];
  }
}

let filterOk: boolean | null = null;
/** Whether canvas `filter = 'blur()'` works (older WebKit ignores it). `?blur=js` forces the fallback. */
export function canvasFilterSupported(): boolean {
  if (filterOk !== null) return filterOk;
  if (globalThis.location?.search.includes('blur=js')) return (filterOk = false);
  try {
    const c = makeCanvas(9, 1);
    const g = ctx2d(c);
    g.filter = 'blur(2px)';
    g.fillStyle = '#000';
    g.fillRect(4, 0, 1, 1);
    const d = g.getImageData(0, 0, 9, 1).data;
    filterOk = d[3 * 4 + 3] > 0 || d[5 * 4 + 3] > 0;
  } catch {
    filterOk = false;
  }
  return filterOk;
}

const tmpCanvases = new Map<string, AnyCanvas>();

/**
 * Blur a canvas in place by `sigma` px. Uses the canvas filter when available, else a
 * three-pass box blur on the pixels (close to a gaussian).
 */
export function blurCanvas(c: AnyCanvas, sigma: number) {
  if (sigma <= 0.3) return;
  const g = ctx2d(c);
  if (canvasFilterSupported()) {
    const key = `${c.width}x${c.height}`;
    let tmp = tmpCanvases.get(key);
    if (!tmp) {
      tmp = makeCanvas(c.width, c.height);
      if (tmpCanvases.size > 6) tmpCanvases.clear();
      tmpCanvases.set(key, tmp);
    }
    const t = ctx2d(tmp);
    t.setTransform(1, 0, 0, 1, 0, 0);
    t.clearRect(0, 0, tmp.width, tmp.height);
    t.filter = `blur(${sigma}px)`;
    t.drawImage(c, 0, 0);
    t.filter = 'none';
    g.save();
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.globalCompositeOperation = 'copy';
    g.drawImage(tmp, 0, 0);
    g.restore();
    return;
  }
  const img = g.getImageData(0, 0, c.width, c.height);
  boxBlurRGBA(img.data, c.width, c.height, sigma);
  g.putImageData(img, 0, 0);
}

function boxBlurRGBA(px: Uint8ClampedArray, w: number, h: number, sigma: number) {
  // Three box passes with widths from the standard sigma-to-boxes formula.
  const n = 3;
  const wIdeal = Math.sqrt((12 * sigma * sigma) / n + 1);
  let wl = Math.floor(wIdeal);
  if (wl % 2 === 0) wl--;
  const m = Math.round((12 * sigma * sigma - n * wl * wl - 4 * n * wl - 3 * n) / (-4 * wl - 4));
  const sizes = Array.from({ length: n }, (_, i) => (i < m ? wl : wl + 2));
  const a = new Float32Array(px.length);
  for (let i = 0; i < px.length; i++) a[i] = px[i];
  const b = new Float32Array(px.length);
  for (const size of sizes) {
    const r = (size - 1) / 2;
    boxH(a, b, w, h, r);
    boxV(b, a, w, h, r);
  }
  for (let i = 0; i < px.length; i++) px[i] = a[i];
}
function boxH(src: Float32Array, dst: Float32Array, w: number, h: number, r: number) {
  const inv = 1 / (2 * r + 1);
  for (let y = 0; y < h; y++)
    for (let ch = 0; ch < 4; ch++) {
      let acc = 0;
      const at = (x: number) => src[4 * (y * w + (x < 0 ? 0 : x >= w ? w - 1 : x)) + ch];
      for (let x = -r; x <= r; x++) acc += at(x);
      for (let x = 0; x < w; x++) {
        dst[4 * (y * w + x) + ch] = acc * inv;
        acc += at(x + r + 1) - at(x - r);
      }
    }
}
function boxV(src: Float32Array, dst: Float32Array, w: number, h: number, r: number) {
  const inv = 1 / (2 * r + 1);
  for (let x = 0; x < w; x++)
    for (let ch = 0; ch < 4; ch++) {
      let acc = 0;
      const at = (y: number) => src[4 * ((y < 0 ? 0 : y >= h ? h - 1 : y) * w + x) + ch];
      for (let y = -r; y <= r; y++) acc += at(y);
      for (let y = 0; y < h; y++) {
        dst[4 * (y * w + x) + ch] = acc * inv;
        acc += at(y + r + 1) - at(y - r);
      }
    }
}
