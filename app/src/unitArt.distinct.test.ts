// Two unit kinds must never share one piece of art (UI review of train-1101d: the Hunter-Killer drew as the
// Infantry's grenade thrower, and the Stealth Fighter and Stealth Bomber as the Modern Fighter and Bomber).
// A pair that really is one unit under two names goes in SHARED with its reason; the list starts empty.
import { existsSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { FACTIONS, UNIT_STATS } from './data';
import { unitArtSlug } from './unitArt';
import type { Faction, UnitKind } from './types';

const KINDS = Object.keys(UNIT_STATS) as UnitKind[];
const dir = new URL('./assets/units/', import.meta.url).pathname;

/** Pairs of kinds allowed to share a slug, as "KindA+KindB" (sorted) with the reason. */
const SHARED: Record<string, string> = {};

describe('unit art is one-to-one with unit kinds', () => {
  it('gives every unit kind (and every faction piece) its own art slug', () => {
    const bySlug = new Map<string, Set<UnitKind>>();
    for (const faction of [undefined, ...FACTIONS] as (Faction | undefined)[]) {
      for (const kind of KINDS) {
        const slug = unitArtSlug(kind, faction);
        bySlug.set(slug, (bySlug.get(slug) ?? new Set()).add(kind));
      }
    }
    const clashes: string[] = [];
    for (const [slug, kinds] of bySlug) {
      const names = [...kinds].sort();
      for (let i = 0; i < names.length; i++) for (let j = i + 1; j < names.length; j++) {
        if (!SHARED[`${names[i]}+${names[j]}`]) clashes.push(`${names[i]} and ${names[j]} both use "${slug}"`);
      }
    }
    expect(clashes).toEqual([]);
  });

  it('keeps every whitelisted pair real', () => {
    for (const pair of Object.keys(SHARED)) {
      const [a, b] = pair.split('+') as UnitKind[];
      expect(unitArtSlug(a), pair).toBe(unitArtSlug(b));
    }
  });

  it('ships the whole asset set for every slug in use', () => {
    const slugs = new Set<string>();
    for (const faction of [undefined, ...FACTIONS] as (Faction | undefined)[]) for (const kind of KINDS) slugs.add(unitArtSlug(kind, faction));
    const missing: string[] = [];
    for (const slug of slugs) for (const file of ['256.png', '64.png', '48.png', 'card.png', 'panel.png', 'source.png', 'roundel.svg']) {
      if (!existsSync(`${dir}${slug}-${file}`)) missing.push(`${slug}-${file}`);
    }
    expect(missing).toEqual([]);
  });
});
