// The painted unlock miniatures (`assets/unlocks`, art direction A, user 2026-10-11): every building,
// faction replacement, wonder, tile improvement and technology effect as a hand-painted tabletop
// miniature, the same family as the city and unit art. Units reuse their own miniatures
// (`unitArt.ts`). One lookup for every surface that shows an unlock: research cards and their
// hovercards, the production list, the city panel, the era-up takeover and the Guide. All seven world
// wonders (Pyramids to ITER Fusion Reactor) are painted, with the same three sizes as every other piece.
import type { BuildingKind, Faction, ImprovementKind, ProductionItem, TechId, UnitKind } from './types';
import { KESHIG_KIND, UNIT_NAME, pluralUnit } from './data';
import { unitMiniatureUrl, unitThumbUrl } from './unitArt';

export type UnlockArtSize = 64 | 128 | 256;

const files = import.meta.glob<string>('./assets/unlocks/*.png', { eager: true, query: '?url', import: 'default' });
const bySlug = new Map<string, Partial<Record<UnlockArtSize, string>>>();
for (const [path, url] of Object.entries(files)) {
  const m = /\/([a-z0-9-]+)-(64|128|256)\.png$/.exec(path);
  if (!m) continue;
  const entry = bySlug.get(m[1]) ?? {};
  entry[Number(m[2]) as UnlockArtSize] = url;
  bySlug.set(m[1], entry);
}

/** The file for `slug` at `size`, else the nearest larger, else the nearest smaller; null if unpainted. */
export function slugUrl(slug: string, size: UnlockArtSize): string | null {
  const entry = bySlug.get(slug);
  if (!entry) return null;
  const order: UnlockArtSize[] = size === 64 ? [64, 128, 256] : size === 128 ? [128, 256, 64] : [256, 128, 64];
  for (const s of order) if (entry[s]) return entry[s]!;
  return null;
}

/** Faction buildings painted as their own piece (core `data::building_name_for`). */
const REPLACEMENTS: Partial<Record<Faction, Partial<Record<BuildingKind, true>>>> = {
  Switzerland: { Walls: true },
  Germany: { Forge: true },
  Athens: { Library: true },
  India: { Market: true },
  Egypt: { Granary: true },
  Rome: { Walls: true },
};

export function buildingArtSlug(building: BuildingKind, faction?: Faction): string {
  const base = `building-${building.toLowerCase()}`;
  return faction && REPLACEMENTS[faction]?.[building] ? `${base}-${faction.toLowerCase()}` : base;
}

const UPGRADED: Record<ImprovementKind, string> = { Farm: 'irrigated-farm', Pasture: 'ranch', Quarry: 'deep-quarry', Mine: 'deep-mine', SaltpetreWorks: 'saltpetre-works', OilWell: 'oil-well', QuartzMine: 'quartz-mine' };
const LATER = new Set<ImprovementKind>(['SaltpetreWorks', 'OilWell', 'QuartzMine']);
export function improvementArtSlug(improvement: ImprovementKind, upgraded = false): string {
  return `improvement-${upgraded || LATER.has(improvement) ? UPGRADED[improvement] : improvement.toLowerCase()}`;
}

function unitUrl(unit: UnitKind, faction: Faction | undefined, size: UnlockArtSize): string | null {
  return size === 64 ? unitThumbUrl(unit, faction) : unitMiniatureUrl(unit, faction);
}

/** A production item's painting, or null if not yet painted. A project's is `project-<name>` (Gold, Science, Egypt's Grow). */
export function itemArtUrl(item: ProductionItem, faction: Faction | undefined, size: UnlockArtSize, upgraded = false): string | null {
  switch (item.kind) {
    case 'Unit': return unitUrl(item.unit, faction, size);
    case 'Building':
    case 'Wonder': return slugUrl(buildingArtSlug(item.building, faction), size) ?? slugUrl(buildingArtSlug(item.building), size);
    case 'Improvement': return slugUrl(improvementArtSlug(item.improvement, upgraded), size);
    case 'Project': return slugUrl(`project-${item.project.toLowerCase()}`, size);
    default: return null;
  }
}

const UNIT_BY_PLURAL: [RegExp, UnitKind][] = [
  // Longest name first, so "Trench Infantry" is found before the "Infantry" inside it.
  ...(Object.entries(UNIT_NAME) as [UnitKind, string][])
    .sort(([, a], [, b]) => b.length - a.length)
    .map(([kind, name]): [RegExp, UnitKind] => [new RegExp(`\\b(${pluralUnit(name)}|${name})\\b`), kind]),
  // The Mongols' upgrade line names their piece ("Horsemen can upgrade to Keshigs", `factionEffectLabel`).
  [/\bKeshigs?\b/, KESHIG_KIND],
];

/** The painting for one research effect line (core `data::tech_card_effects`). A unit upgrade line
 * ("Spearmen can upgrade to Phalangites") shows the unit it upgrades to; an improvement upgrade
 * line its upgraded improvement; every other effect its own painting. */
export function effectArtSlugOrUnit(tech: TechId, label: string): { slug: string } | { unit: UnitKind } | null {
  if (tech === 'RecursiveAi' && /reveal.*map/i.test(label)) return { slug: 'effect-recursiveai-map' };
  if (tech === 'StealthTechnology') {
    if (/^Jet Fighters can upgrade/.test(label)) return { unit: 'StealthFighter' };
    if (/^Jet Bombers can upgrade/.test(label)) return { unit: 'StealthBomber' };
    return { slug: 'effect-quantumcomputing' };
  }
  // Air Power's Biplane line names no unit (the card unlocks both): it shows the Fighter, the default line.
  if (tech === 'AirPower' && /^Each Biplane can upgrade/.test(label)) return { unit: 'Fighter' };
  const upgradeTo = /can upgrade to (.+)$/.exec(label);
  if (upgradeTo) {
    const hit = UNIT_BY_PLURAL.find(([re]) => re.test(upgradeTo[1]));
    return hit ? { unit: hit[1] } : null;
  }
  if (/^Farms and Pastures upgrade/.test(label)) return { slug: improvementArtSlug('Farm', true) };
  if (/^Mines upgrade/.test(label)) return { slug: improvementArtSlug('Mine', true) };
  if (/^Quarries upgrade/.test(label)) return { slug: improvementArtSlug('Quarry', true) };
  // One painting per rule: a row names its own building or improvement, so two rows of a card never share an image.
  if (/\bGranary\b/.test(label)) return { slug: buildingArtSlug('Granary') };
  if (/\bWorkshop\b/.test(label)) return { slug: buildingArtSlug('Workshop') };
  if (/\bLibrary\b/.test(label)) return { slug: buildingArtSlug('Library') };
  if (/\bMarket\b/.test(label)) return { slug: buildingArtSlug('Market') };
  if (/^Roadworks link/.test(label)) return { slug: buildingArtSlug('Roadworks') };
  if (/^Mines \+1/.test(label)) return { slug: improvementArtSlug('Mine', true) };
  if (/^Factories\b/.test(label)) return { slug: buildingArtSlug('Factory') };
  if (tech === 'CropRotation') return { slug: buildingArtSlug('Granary') };
  if (tech === 'Combustion') return { slug: improvementArtSlug('OilWell') };
  if (tech === 'MilitaryScience') return { slug: 'effect-militaryengineering' };
  if (tech === 'Philosophy') return { slug: buildingArtSlug('GreatLibrary') };
  if (tech === 'ReplaceableParts') return { slug: buildingArtSlug('Factory') };
  if (tech === 'CodeOfLaws') return { slug: 'effect-codeoflaws-claim' };
  return { slug: `effect-${tech.toLowerCase()}` };
}

export function effectArtUrl(tech: TechId, label: string, faction: Faction | undefined, size: UnlockArtSize): string | null {
  const art = effectArtSlugOrUnit(tech, label);
  if (!art) return null;
  return 'unit' in art ? unitUrl(art.unit, faction, size) : slugUrl(art.slug, size);
}

/** The painting of one research-card unlock (`techCardUnlocks`) or unlock-card tile: its item's, or its effect line's. */
export function unlockEntryArt(item: ProductionItem | null | undefined, label: string, tech: TechId, faction: Faction | undefined, size: UnlockArtSize): string | null {
  return item ? itemArtUrl(item, faction, size) : effectArtUrl(tech, label, faction, size);
}
