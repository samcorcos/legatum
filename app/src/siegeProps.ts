// The siege gear a Siege unit shows on the map (siege study 2026-10-18; Siege upgrade 2026-10-22): a military unit that bought
// Siege fights a city as if it had no Walls, and wears the era's engine beside it so that reads at a glance. Archers get
// a catapult, trebuchet or bombard by the siege tier; every other unit gets scaling ladders (tier 1) or a siege
// tower (tiers 2 and 3). The tier mirrors core `data::siege_tier`: exact for your own empire (its technologies are in
// the view), and from a rival's era for the rest (a rival's technologies are private).
// The gear also belongs to the unit's own period (user, 2026-10-28: a Rifleman beside a siege tower, a Field Cannon beside a
// bombard): the gear is capped by the unit's own era (see `siegePropFor`); Renaissance units get the bombard or ladders. Industrial and later units have Siege built in (core
// `data::built_in_siege`, designer 2026-10-28) and wear no gear: nothing is drawn for it.
import { iconLoaded } from './icons';
import { siegeTier } from './combat';
import { BUILT_IN_SIEGE_ERA, unitClass, unitEraTier } from './data';
import type { PlayerView, UnitKind, UnitView } from './types';

export type SiegeProp = 'catapult' | 'trebuchet' | 'bombard' | 'ladders' | 'siege-tower';

const propUrls = import.meta.glob<string>('./assets/siege-props/*-128.png', { eager: true, query: '?url', import: 'default' });
const urlOf = (prop: SiegeProp): string | undefined => Object.entries(propUrls).find(([path]) => path.endsWith(`/${prop}-128.png`))?.[1];

/** The siege tier of `unit` in this view: 0 when it has no Siege. */
function coverTier(view: PlayerView, unit: UnitView): number {
  if (!unit.military || !unit.siege) return 0;
  if (unit.owner === view.me) return siegeTier(view.research.known);
  const era = view.players.find((p) => p.id === unit.owner)?.era ?? 1;
  return era >= 4 ? 3 : era === 3 ? 2 : era === 2 ? 1 : 0;
}

/** The first era (0-based) with no period siege gear: Industrial, whose units have Siege built in. */
const GEARLESS_ERA = BUILT_IN_SIEGE_ERA;

/**
 * Which gear to draw beside `unit`, or null. The tier from the technologies is capped by the unit's own era (designer, 2026-10-28:
 * an Ancient Archer with Metallurgy wore a bombard): Ancient and Classical archers get the catapult, Medieval ones up to the
 * trebuchet, Renaissance ones the bombard; every other unit gets ladders, or the siege tower at tier 2 and above through Medieval
 * (Renaissance units keep ladders).
 * The prop is cosmetic: the combat tier still comes from the technologies.
 */
export function siegePropFor(view: PlayerView, unit: UnitView): SiegeProp | null {
  const tier = coverTier(view, unit);
  const era = unitEraTier(unit.kind as UnitKind);
  if (tier === 0 || era >= GEARLESS_ERA) return null;
  if (unitClass(unit.kind as UnitKind) === 'Ranged') {
    if (era <= 1) return 'catapult';
    if (era === 2) return tier === 1 ? 'catapult' : 'trebuchet';
    return 'bombard';
  }
  return era <= 2 && tier >= 2 ? 'siege-tower' : 'ladders';
}

const images = new Map<SiegeProp, HTMLImageElement | null>();
/** The prop's 128 px painting once loaded; null while loading or when the art is missing. */
export function siegePropImage(prop: SiegeProp): HTMLImageElement | null {
  if (images.has(prop)) {
    const img = images.get(prop)!;
    return img && img.complete && img.naturalWidth > 0 ? img : null;
  }
  const url = urlOf(prop);
  if (!url || typeof Image === 'undefined') {
    images.set(prop, null);
    return null;
  }
  const img = new Image();
  img.onload = () => iconLoaded();
  img.src = url;
  images.set(prop, img);
  return null;
}

/** Where each prop's painted part lies in its 128 px frame (left, top, right, bottom): the alpha bounds, so a figure can be fitted to a hex exactly. */
const FRAME_BOX: Record<SiegeProp, [number, number, number, number]> = {
  catapult: [8, 8, 120, 120], bombard: [8, 47, 120, 120], ladders: [31, 8, 98, 120], 'siege-tower': [20, 8, 109, 120], trebuchet: [28, 8, 100, 120],
};

/**
 * The 128 px frame's square on the map for a covered unit at token half-width `w`, standing at `y` in a hex centred on `cx` with half-width
 * `half`. The painted gear stands in front of the stack's feet on the hex's right side and never leaves the hex (user, 2026-10-19: it spilled
 * half a hex onto the next unit): at most 2.2 token half-widths across, 2.0 high, 95% of the hex half-width, and its right edge 92% of the way to the hex edge.
 */
export function siegePropRect(prop: SiegeProp, w: number, cx: number, half: number, y: number): { x: number; y: number; size: number; left: number; right: number } {
  const [l, t, r, b] = FRAME_BOX[prop];
  const k = Math.min(Math.min(w * 2.2, half * 0.95) / (r - l), (w * 2.0) / (b - t));
  const edge = cx + half * 0.92;
  return { x: edge - r * k, y: y + w * 1.0 - b * k, size: 128 * k, left: edge - (r - l) * k, right: edge };
}
