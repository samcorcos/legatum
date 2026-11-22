import { describe, expect, it } from 'vitest';
import { FACTIONS, RULES, techCardEffectsFor } from './data';
import type { BuildingKind, ImprovementKind, TechId } from './types';
import { buildingArtSlug, effectArtSlugOrUnit, effectArtUrl, improvementArtSlug, itemArtUrl, slugUrl } from './unlockArt';

const BUILDINGS = Object.keys(RULES.buildingNames) as BuildingKind[];
const IMPROVEMENTS: ImprovementKind[] = ['Farm', 'Pasture', 'Quarry', 'Mine'];
const TECHS = RULES.techOrder as TechId[];

/** Every painting the full set must have: each building, each faction replacement, each improvement
 * and its upgrade, and every effect line that is not a unit or improvement upgrade. */
function expectedSlugs() {
  const slugs = new Set<string>();
  for (const b of BUILDINGS) {
    slugs.add(buildingArtSlug(b));
    for (const f of FACTIONS) slugs.add(buildingArtSlug(b, f));
  }
  for (const k of IMPROVEMENTS) { slugs.add(improvementArtSlug(k)); slugs.add(improvementArtSlug(k, true)); }
  for (const f of FACTIONS) for (const t of TECHS) for (const label of techCardEffectsFor(f, t)) {
    const art = effectArtSlugOrUnit(t, label);
    if (art && 'slug' in art) slugs.add(art.slug);
  }
  return [...slugs].sort();
}

describe('unlock art lookup', () => {
  it('names faction replacements as their own piece and everything else by kind', () => {
    expect(buildingArtSlug('Walls')).toBe('building-walls');
    expect(buildingArtSlug('Walls', 'Switzerland')).toBe('building-walls-switzerland');
    expect(buildingArtSlug('Walls', 'India')).toBe('building-walls');
    expect(buildingArtSlug('Workshop', 'Germany')).toBe('building-workshop');
    expect(buildingArtSlug('Forge', 'Germany')).toBe('building-forge-germany');
    expect(buildingArtSlug('Library', 'Byzantium')).toBe('building-library');
    expect(improvementArtSlug('Farm', true)).toBe('improvement-irrigated-farm');
    expect(improvementArtSlug('SaltpetreWorks')).toBe('improvement-saltpetre-works');
    expect(improvementArtSlug('OilWell')).toBe('improvement-oil-well');
    expect(improvementArtSlug('QuartzMine')).toBe('improvement-quartz-mine');
  });

  it('maps every effect line to a painting or the unit it upgrades to', () => {
    for (const f of FACTIONS) for (const t of TECHS) for (const label of techCardEffectsFor(f, t)) {
      expect(effectArtSlugOrUnit(t, label), `${f} ${t}: ${label}`).not.toBeNull();
    }
    expect(effectArtSlugOrUnit('Mathematics', 'Spearmen can upgrade to Phalangites')).toEqual({ unit: 'Phalangite' });
    expect(effectArtSlugOrUnit('Gunpowder', 'Pikemen can upgrade to Musketmen')).toEqual({ unit: 'Musketman' });
    expect(effectArtSlugOrUnit('CodeOfLaws', 'Every city claims its best available ring-2 tile')).toEqual({ slug: 'effect-codeoflaws-claim' });
    expect(effectArtSlugOrUnit('IronWorking', 'Mines upgrade: Deep Mine (+1 Production)')).toEqual({ slug: 'improvement-deep-mine' });
    expect(effectArtSlugOrUnit('RecursiveAi', 'Reveal the whole map to your team; units still need sight')).toEqual({ slug: 'effect-recursiveai-map' });
    expect(effectArtSlugOrUnit('RecursiveAi', '+20% Production in every city')).toEqual({ slug: 'effect-recursiveai' });
    expect(effectArtUrl('Irrigation', '+1 Food in every city', 'India', 256)).toMatch(/effect-irrigation-256/);
  });

  it('serves the approved samples at every size and falls back to the generic building', () => {
    for (const slug of ['building-workshop', 'building-walls', 'building-market', 'building-library', 'effect-irrigation']) {
      for (const size of [64, 128, 256] as const) expect(slugUrl(slug, size), `${slug} ${size}`).toMatch(new RegExp(`${slug}-${size}`));
    }
    expect(itemArtUrl({ kind: 'Building', building: 'Market' }, 'India', 128)).toMatch(/building-market-india-128/);
    expect(itemArtUrl({ kind: 'Unit', unit: 'Knight' }, 'India', 256)).toMatch(/war-elephant-256/);
    expect(itemArtUrl({ kind: 'Project', project: 'Gold' } as never, 'Germany', 64)).toBeNull();
  });

  it('lists the full set it expects (about 44 pieces)', () => {
    const slugs = expectedSlugs();
    expect(slugs.length).toBeGreaterThanOrEqual(40);
    expect(slugs).toContain('building-forge-germany');
    expect(slugs).toContain('effect-railways');
  });

  it('has every piece of the full set painted at all three sizes', () => {
    const missing = expectedSlugs().flatMap((slug) => ([64, 128, 256] as const).flatMap((size) => {
      const url = slugUrl(slug, size);
      return url && url.includes(`${slug}-${size}`) ? [] : [`${slug}-${size}`];
    }));
    expect(missing).toEqual([]);
  });
});
