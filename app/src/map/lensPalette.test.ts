// Lens markers must stay distinct for every kind of sight (user, 2026-10-11: "All the icons, miniatures, etc need to be
// visually distinct"): the colours a lens uses together are at least 10 CIEDE2000 apart under normal, protan, deutan
// and tritan vision, the same bar the territory borders meet. (The Settle tiers are checked in siteBands.test.ts.)
import { describe, expect, it } from 'vitest';
import { CVD, deltaE00, labOf, type Vision } from '../test/cvd';
import { ATLAS } from './style';

const GROUPS: Record<string, Record<string, string>> = {
  'Resource categories (lens legend and far-zoom marks)': { Bonus: ATLAS.resources.bonus, Strategic: ATLAS.resources.strategic },
  'Movement lens': { easy: ATLAS.lens.tint.easy, rough: ATLAS.lens.tint.rough, blocked: ATLAS.lens.tint.blocked },
};

describe('lens markers are distinct for colour-blind sight', () => {
  for (const [group, colors] of Object.entries(GROUPS)) {
    it(`${group}: every pair at least 10 CIEDE2000 apart`, () => {
      const names = Object.keys(colors);
      for (const vision of Object.keys(CVD) as Vision[]) {
        const labs = names.map((n) => labOf(colors[n], vision));
        for (let i = 0; i < names.length; i++) for (let j = i + 1; j < names.length; j++)
          expect(deltaE00(labs[i], labs[j]), `${vision}: ${names[i]} vs ${names[j]}`).toBeGreaterThanOrEqual(10);
      }
    });
  }
});
