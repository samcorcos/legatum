// Colour-vision-deficiency check shared by palette tests: Machado et al. (2009) matrices at full
// severity, then CIEDE2000 on the simulated colours (the method seatColors.test.ts introduced).
export const CVD = {
  normal: [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
  protan: [[0.152286, 1.052583, -0.204868], [0.114503, 0.786281, 0.099216], [-0.003882, -0.048116, 1.051998]],
  deutan: [[0.367322, 0.860646, -0.227968], [0.280085, 0.672501, 0.047413], [-0.01182, 0.04294, 0.968881]],
  tritan: [[1.255528, -0.076749, -0.178779], [-0.078411, 0.930809, 0.147602], [0.004733, 0.691367, 0.3039]],
} as const;
export type Vision = keyof typeof CVD;

export const rgb = (hex: string): number[] => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
const lin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

/** `color` seen by `vision`, as CIELAB. A colour may be a hex string or [r, g, b] in 0-255. */
export function labOf(color: string | number[], vision: Vision = 'normal'): [number, number, number] {
  const l = (typeof color === 'string' ? rgb(color) : color).map((v) => lin(v / 255));
  const [r, g, b] = CVD[vision].map((row) => Math.min(1, Math.max(0, row[0] * l[0] + row[1] * l[1] + row[2] * l[2])));
  const f = (t: number) => (t > 216 / 24389 ? Math.cbrt(t) : ((24389 / 27) * t + 16) / 116);
  const fx = f((0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047), fy = f(0.2126 * r + 0.7152 * g + 0.0722 * b), fz = f((0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883);
  return [116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)];
}

export function deltaE00([L1, a1, b1]: number[], [L2, a2, b2]: number[]): number {
  const rad = Math.PI / 180, deg = 180 / Math.PI;
  const C1 = Math.hypot(a1, b1), C2 = Math.hypot(a2, b2), Cb = (C1 + C2) / 2;
  const G = 0.5 * (1 - Math.sqrt(Cb ** 7 / (Cb ** 7 + 25 ** 7)));
  const ap1 = (1 + G) * a1, ap2 = (1 + G) * a2, Cp1 = Math.hypot(ap1, b1), Cp2 = Math.hypot(ap2, b2);
  const hp = (b: number, a: number) => { if (!b && !a) return 0; const h = Math.atan2(b, a) * deg; return h < 0 ? h + 360 : h; };
  const h1 = hp(b1, ap1), h2 = hp(b2, ap2);
  const dh = Cp1 * Cp2 === 0 ? 0 : Math.abs(h2 - h1) <= 180 ? h2 - h1 : h2 - h1 > 180 ? h2 - h1 - 360 : h2 - h1 + 360;
  const dL = L2 - L1, dC = Cp2 - Cp1, dH = 2 * Math.sqrt(Cp1 * Cp2) * Math.sin((dh * rad) / 2);
  const Lb = (L1 + L2) / 2, Cpb = (Cp1 + Cp2) / 2;
  const hb = Cp1 * Cp2 === 0 ? h1 + h2 : Math.abs(h1 - h2) <= 180 ? (h1 + h2) / 2 : h1 + h2 < 360 ? (h1 + h2 + 360) / 2 : (h1 + h2 - 360) / 2;
  const T = 1 - 0.17 * Math.cos((hb - 30) * rad) + 0.24 * Math.cos(2 * hb * rad) + 0.32 * Math.cos((3 * hb + 6) * rad) - 0.2 * Math.cos((4 * hb - 63) * rad);
  const dTh = 30 * Math.exp(-(((hb - 275) / 25) ** 2));
  const Rc = 2 * Math.sqrt(Cpb ** 7 / (Cpb ** 7 + 25 ** 7));
  const Sl = 1 + (0.015 * (Lb - 50) ** 2) / Math.sqrt(20 + (Lb - 50) ** 2), Sc = 1 + 0.045 * Cpb, Sh = 1 + 0.015 * Cpb * T;
  const Rt = -Math.sin(2 * dTh * rad) * Rc;
  return Math.sqrt((dL / Sl) ** 2 + (dC / Sc) ** 2 + (dH / Sh) ** 2 + Rt * (dC / Sc) * (dH / Sh));
}

/** `color` washed over `ground` at `alpha`, as [r, g, b]. */
export const wash = (ground: string, color: string, alpha: number): number[] => rgb(ground).map((g, i) => g * (1 - alpha) + rgb(color)[i] * alpha);
