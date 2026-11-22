// The painted unit set (`assets/units`, art direction A): a hand-painted miniature on a stone base
// for the normal map zoom and the unit panel, and a class roundel (silhouette plus owner rim) for
// the far zoom. Everything falls back to the engraved glyph token while an image loads.
import type { Faction, UnitKind } from './types';
import { iconLoaded } from './icons';

const miniatureUrls = import.meta.glob<string>('./assets/units/*-256.png', { eager: true, query: '?url', import: 'default' });
const panelUrls = import.meta.glob<string>('./assets/units/*-panel.png', { eager: true, query: '?url', import: 'default' });
const thumbUrls = import.meta.glob<string>('./assets/units/*-64.png', { eager: true, query: '?url', import: 'default' });
const cardUrls = import.meta.glob<string>('./assets/units/*-card.png', { eager: true, query: '?url', import: 'default' });
const roundelSources = import.meta.glob<string>('./assets/units/*-roundel.svg', { eager: true, query: '?raw', import: 'default' });

const airPlanUrls = import.meta.glob<string>('./assets/units/air/*-plan.png', { eager: true, query: '?url', import: 'default' });

const slugOf = (path: string) => path.split('/').pop()!.replace(/-(256|64|card|panel|roundel)\..*$/, '');
const bySlug = <T,>(files: Record<string, T>) => new Map(Object.entries(files).map(([path, v]) => [slugOf(path), v]));
const miniatures = bySlug(miniatureUrls);
const panels = bySlug(panelUrls);
const thumbs = bySlug(thumbUrls);
const cards = bySlug(cardUrls);
const roundels = bySlug(roundelSources);

/** The art of each unit kind. Soldier is the Spearman, Trooper the Pikeman, AssaultTrooper the Musketman (the identifiers keep their pre-overhaul spelling). */
const UNIT_ART: Record<UnitKind, string> = {
  Scout: 'scout', Settler: 'settler', Heir: 'byzantine-heir',
  Phalangite: 'phalangite', CompositeArcher: 'composite-archer', Cataphract: 'cataphract', Arquebusier: 'arquebusier', Cuirassier: 'cuirassier',
  Soldier: 'spearman', Trooper: 'pikeman', AssaultTrooper: 'musketman', Rifleman: 'rifleman', Infantry: 'infantry', HunterKiller: 'hunter-killer',
  Ranger: 'archer', Marksman: 'crossbowman', FieldCannon: 'field-cannon', Artillery: 'artillery', Drone: 'drone-swarm',
  Horseman: 'horseman', Knight: 'knight', Cavalry: 'cavalry', Tank: 'tank', AutonomousArmour: 'autonomous-armour',
  Fighter: 'fighter', Bomber: 'bomber', StealthFighter: 'stealth-fighter', StealthBomber: 'stealth-bomber',
  TrenchInfantry: 'trench-infantry', Howitzer: 'howitzer', Landship: 'landship', Biplane: 'biplane',
  // The Modern era's placeholder art (increment 2 paints its own): copies of the World War unit of each line, except the
  // Main Battle Tank, which is the present-day tank painting that the Tank's WWII repaint replaced.
  MechanisedInfantry: 'mechanised-infantry', MainBattleTank: 'main-battle-tank', RocketArtillery: 'rocket-artillery', JetFighter: 'jet-fighter', JetBomber: 'jet-bomber',
};

/** Faction pieces with their own painted figure: every faction's special unit (a piece without a painting yet shows the plain unit's). */
const FACTION_ART: Partial<Record<Faction, Partial<Record<UnitKind, string>>>> = {
  Sparta: { Soldier: 'spartan-hoplite' },
  Mongols: { Cataphract: 'keshig' },
  Byzantium: { Trooper: 'tagmatic-soldier' },
  Rome: { Phalangite: 'legionary' },
  Aztecs: { Soldier: 'eagle-warrior' },
  Egypt: { Soldier: 'medjay' },
  Celts: { Phalangite: 'gaesatae' },
  Athens: { CompositeArcher: 'peltast' },
  India: { Knight: 'war-elephant' },
  Switzerland: { Trooper: 'reislaufer' },
  Germany: { Tank: 'panzer' },
};

export function unitArtSlug(kind: UnitKind, faction?: Faction): string {
  return (faction && FACTION_ART[faction]?.[kind]) || UNIT_ART[kind];
}

/** The 64 px miniature, for small badges (the unit panel's head, production cards). */
export const unitThumbUrl = (kind: UnitKind, faction?: Faction): string | null => thumbs.get(unitArtSlug(kind, faction)) ?? null;

/** The 256 px painted miniature (figure on its stone base), for unlock art on cards. */
export const unitMiniatureUrl = (kind: UnitKind, faction?: Faction): string | null => miniatures.get(unitArtSlug(kind, faction)) ?? null;
/** The build-card thumbnail: 64 px, cropped to the figure's upper body so it fills a 30 px card slot (`tools/art/unit-derivatives.py --card`). */
export const unitCardUrl = (kind: UnitKind, faction?: Faction): string | null => cards.get(unitArtSlug(kind, faction)) ?? thumbs.get(unitArtSlug(kind, faction)) ?? null;

/** The 512 px full-figure portrait for the unit panel and cards, or null. */
export const unitPanelUrl = (kind: UnitKind, faction?: Faction): string | null => panels.get(unitArtSlug(kind, faction)) ?? null;

const images = new Map<string, HTMLImageElement | null>();
function load(key: string, url: string | undefined): HTMLImageElement | null {
  if (images.has(key)) {
    const img = images.get(key)!;
    return img && img.complete && img.naturalWidth > 0 ? img : null;
  }
  if (!url || typeof Image === 'undefined') {
    images.set(key, null);
    return null;
  }
  const img = new Image();
  img.onload = () => iconLoaded();
  img.src = url;
  images.set(key, img);
  return null;
}

/** The painted miniature (256 px) once loaded. */
export function miniatureImage(kind: UnitKind, faction?: Faction): HTMLImageElement | null {
  const slug = unitArtSlug(kind, faction);
  return load(`m|${slug}`, miniatures.get(slug));
}

/**
 * Barbarians' own painted raiders (`barbarian-<slug>`), one per kind the camps field (Spearman, Pikeman, Musketman, Archer,
 * Crossbowman, Scout). Any other kind (a Company that turned barbarian) keeps its ordinary figure on the barbarian base.
 */
export const barbarianArtSlug = (kind: UnitKind): string => miniatures.has(`barbarian-${UNIT_ART[kind]}`) ? `barbarian-${UNIT_ART[kind]}` : UNIT_ART[kind];

/** A barbarian's painted miniature (256 px) once loaded. */
export function barbarianMiniatureImage(kind: UnitKind): HTMLImageElement | null {
  const slug = barbarianArtSlug(kind);
  return load(`m|${slug}`, miniatures.get(slug));
}

/** The far-zoom roundel in the owner's colour: the SVG's `currentColor` is its disc wash and rim. */
export function roundelImage(kind: UnitKind, faction: Faction | undefined, color: string): HTMLImageElement | null {
  const slug = unitArtSlug(kind, faction);
  const key = `r|${slug}|${color}`;
  if (!images.has(key)) {
    const svg = roundels.get(slug);
    if (!svg || typeof URL === 'undefined' || typeof Blob === 'undefined') {
      images.set(key, null);
      return null;
    }
    return load(key, URL.createObjectURL(new Blob([svg.replace(/currentColor/g, color)], { type: 'image/svg+xml' })));
  }
  return load(key, undefined);
}

/**
 * The painted top-down sprite the combat animation flies (`assets/units/air/<name>-plan.png`, nose to the right): an
 * aircraft kind's plan view, or `drone` for a Drone Swarm's. Null while it loads.
 */
export type AirPlanName = 'Biplane' | 'Bomber' | 'JetBomber' | 'Fighter' | 'JetFighter' | 'StealthBomber' | 'StealthFighter' | 'drone';
export function airPlanImage(name: AirPlanName): HTMLImageElement | null {
  const slug = name === 'drone' ? 'drone' : unitArtSlug(name);
  return load(`plan|${slug}`, airPlanUrls[`./assets/units/air/${slug}-plan.png`]);
}

/** Start loading the plan sprites a turn's playback will fly, so the first strike of a session shows its aircraft from its first frame. */
export function warmAirArt(names: AirPlanName[]) {
  for (const n of new Set(names)) airPlanImage(n);
}
