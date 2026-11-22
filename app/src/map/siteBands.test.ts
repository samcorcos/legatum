import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { BAND_FLOORS, SITE_BANDS, bandRange, siteBandOf } from './siteBands';
import { MAP_STYLES } from './style';
import { CVD, deltaE00, labOf, rgb, wash, type Vision } from '../test/cvd';

describe('the Settle scale is fixed', () => {
  it('has four rising floors that split five named bands', () => {
    expect(SITE_BANDS).toEqual(['Poor', 'Fair', 'Good', 'Great', 'Exceptional']);
    expect(BAND_FLOORS).toHaveLength(4);
    expect(BAND_FLOORS.every((f, i) => i === 0 || f > BAND_FLOORS[i - 1])).toBe(true);
  });
  it('puts every quality in one band, edges included, and never depends on the other sites', () => {
    const [fair, good, great, exceptional] = BAND_FLOORS;
    expect([-500, 0, fair - 1].map(siteBandOf)).toEqual(['Poor', 'Poor', 'Poor']);
    expect([fair, good - 1].map(siteBandOf)).toEqual(['Fair', 'Fair']);
    expect([good, great - 1].map(siteBandOf)).toEqual(['Good', 'Good']);
    expect([great, exceptional - 1].map(siteBandOf)).toEqual(['Great', 'Great']);
    expect([exceptional, 10_000].map(siteBandOf)).toEqual(['Exceptional', 'Exceptional']);
  });
  it('words the ranges so they tile the number line with no gap', () => {
    const [fair, good, great, exceptional] = BAND_FLOORS;
    expect(SITE_BANDS.map(bandRange)).toEqual([`under ${fair}`, `${fair} to ${good - 1}`, `${good} to ${great - 1}`, `${great} to ${exceptional - 1}`, `${exceptional} or more`]);
  });
});

describe('the Settle palette is one green, colour-blind safe and reads over every land terrain', () => {
  const lens = MAP_STYLES.atlas.lens, colors = SITE_BANDS.map((band) => lens.settle[band]);
  const land = (['Grassland', 'Plains', 'Forest', 'Hills', 'Desert', 'Tundra'] as const).map((t) => MAP_STYLES.atlas.pal[t]);
  const visions = Object.keys(CVD) as Vision[];
  const washed = (ground: string, fade: number) => colors.map((c, i) => wash(ground, c, lens.settleAlpha[SITE_BANDS[i]] * fade));
  // Per band: how far it must sit from the next band, and from the bare ground. Poor is a faint tint by design ("nearly clear").
  // The faded wash (remembered land) is paler, so its bars are lower.
  const bars = [
    { name: 'full wash', fade: 1, adjacent: [4, 7, 9, 9], ground: [1, 5, 12, 22, 30], lightness: 3 },
    { name: 'faded wash', fade: lens.faded / lens.wash, adjacent: [2.5, 4, 6, 6], ground: [0.8, 3, 7, 12, 18], lightness: 2.5 },
  ];
  it.each(bars)('at the $name keeps neighbouring bands and each band apart from the ground, for every vision', ({ fade, adjacent, ground }) => {
    for (const terrain of land) for (const vision of visions) {
      const labs = washed(terrain, fade).map((c) => labOf(c, vision));
      for (let i = 0; i < 4; i++) expect(deltaE00(labs[i], labs[i + 1]), `${terrain} ${vision} ${SITE_BANDS[i]}/${SITE_BANDS[i + 1]}`).toBeGreaterThanOrEqual(adjacent[i]);
      for (const [i, lab] of labs.entries()) expect(deltaE00(lab, labOf(terrain, vision)), `${terrain} ${vision} ${SITE_BANDS[i]} vs ground`).toBeGreaterThanOrEqual(ground[i]);
    }
  });
  it.each(bars)('gets darker in step with quality at the $name, so the order survives any vision and greyscale', ({ fade, lightness }) => {
    for (const terrain of land) for (const vision of visions) {
      const l = washed(terrain, fade).map((c) => labOf(c, vision)[0]);
      for (let i = 0; i < 4; i++) expect(l[i] - l[i + 1], `${terrain} ${vision} ${SITE_BANDS[i]}`).toBeGreaterThan(lightness);
    }
  });
  it('is a single green hue with covering power that rises with quality', () => {
    const hue = (hex: string) => { const [r, g, b] = rgb(hex).map((v) => v / 255), max = Math.max(r, g, b), min = Math.min(r, g, b); return (max === g ? 60 * (2 + (b - r) / (max - min)) : 0); };
    // Emerald (about 150 degrees) is bluer than the terrain greens (olive, 70 to 95), so the lens reads as a layer over grassland and forest.
    for (const c of colors) { expect(hue(c), c).toBeGreaterThan(140); expect(hue(c), c).toBeLessThan(160); }
    const alphas = SITE_BANDS.map((band) => lens.settleAlpha[band]);
    for (let i = 0; i < 4; i++) expect(alphas[i + 1]).toBeGreaterThan(alphas[i]);
    expect(alphas[0]).toBeLessThan(0.3);
  });
  it('is the same palette in the canvas and in the legend swatches', () => {
    const css = readFileSync(fileURLToPath(new URL('../brand/tokens.css', import.meta.url)), 'utf8');
    for (const band of SITE_BANDS) {
      const key = band.toLowerCase();
      expect(css).toContain(`--settle-${key}: ${lens.settle[band]};`);
      expect(css).toContain(`--settle-${key}-a: ${Math.round(lens.settleAlpha[band] * 100)}%;`);
    }
  });
});
