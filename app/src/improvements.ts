// Tile improvements (playtest 8): a city builds one on an owned, worked resource tile; the
// resource yields its bonus only once improved, and the improvement upgrades with technology by
// itself. Display helpers shared with the UI; the core (`improvements.rs`) is
// authoritative, and `rules.generated.json` carries its numbers.
import { IMPROVEMENT_INFO, RULES, TECH_NAME } from './data';
import { bearing } from './empires';
import { distance } from './hex';
import { RESOURCE_INFO } from './resources';
import type { Hex, ImprovementKind, ProductionItem, Resource, TechId, TileView, Yields } from './types';

type ResourceImprovement = { kind: ImprovementKind; upgradeYields: Yields };
const FOR_RESOURCE = RULES.improvements.forResource as unknown as Record<Resource, ResourceImprovement>;

/** The improvement a resource needs (core `improvements::kind_for`). */
export const improvementKindFor = (r: Resource): ImprovementKind => FOR_RESOURCE[r].kind;

/** The upgrade's extra yield (core `improvements::upgrade_yields`). */
export const improvementUpgradeYields = (r: Resource): Yields => ({ ...FOR_RESOURCE[r].upgradeYields });

/** Whether an owner with this knowledge has the upgraded form (core `improvements::upgraded`). */
export const improvementUpgraded = (kind: ImprovementKind, knows: (t: TechId) => boolean): boolean => knows(IMPROVEMENT_INFO[kind].upgradeTech);

const ZERO: Yields = { food: 0, production: 0, gold: 0, science: 0 };

/** What an improved resource adds to its tile (core `improvements::resource_yields`). */
export function improvedResourceYields(r: Resource, knows: (t: TechId) => boolean): Yields {
  const tech = RESOURCE_INFO[r].revealedBy;
  if (tech && !knows(tech)) return { ...ZERO };
  const bonus = RESOURCE_INFO[r].bonus;
  const up = improvementUpgraded(improvementKindFor(r), knows) ? FOR_RESOURCE[r].upgradeYields : ZERO;
  const rails = ['Copper', 'Iron', 'Coal', 'Silicon'].includes(r) && knows('Railways') ? 1 : 0;
  return { food: bonus.food + up.food, production: bonus.production + up.production + rails, gold: bonus.gold + up.gold, science: bonus.science + up.science };
}

/**
 * What improving a resource tile gains the viewer now: the one number the tile card and the
 * production picker both show. The resource is on screen, so it is revealed; the upgrade's extra
 * yield counts once its technology is known.
 */
export const improvementGain = (r: Resource, known: readonly TechId[]): Yields =>
  improvedResourceYields(r, (t) => known.includes(t) || t === RESOURCE_INFO[r].revealedBy);

/** "2 tiles NE": distance and bearing kept together (non-breaking spaces, so they never wrap apart). */
export function improvementWhere(item: Extract<ProductionItem, { kind: 'Improvement' }>, cityPos: Hex): string {
  const away = distance(cityPos, item.tile);
  return `${away}\u00a0tile${away === 1 ? '' : 's'}\u00a0${bearing(cityPos, item.tile)}`;
}

/** "Quarry, Stone": the improvement and its resource, the head of every wording (the meter, queue, plan, picker and reports). */
export function improvementName(improvement: ImprovementKind, resource: Resource | null | undefined): string {
  const name = IMPROVEMENT_INFO[improvement].name;
  return resource ? `${name}, ${RESOURCE_INFO[resource].name}` : name;
}

/** "Mine, Iron · 2 tiles NE": the name a queue item gives its tile (a report line has no resource: "Mine, 2 tiles NE"). */
export function improvementTitle(item: Extract<ProductionItem, { kind: 'Improvement' }>, cityPos: Hex, resource: Resource | null | undefined): string {
  return `${improvementName(item.improvement, resource)}${resource ? ' ·' : ','} ${improvementWhere(item, cityPos)}`;
}

/** "Quarry, Stone NE": the short form for the production meter's one-line label (the bearing is never cut). */
export function improvementShortTitle(item: Extract<ProductionItem, { kind: 'Improvement' }>, cityPos: Hex, resource: Resource | null | undefined): string {
  return `${improvementName(item.improvement, resource)} ${bearing(cityPos, item.tile)}`;
}

/**
 * Dormant: a resource you own that is known and not yet improved. The one check behind the picker's
 * count, the map's faded medallion and dashed ring, and the tile card (legatum-ui: a status word
 * comes from one shared check). Strategic deposits count too: their supply works as soon as they are inside the borders, but the
 * tile's own yield waits for the Mine.
 */
export const isDormant = (tile: Pick<TileView, 'resource' | 'owner' | 'improvement'>, me: number, faction?: string): boolean =>
  !!tile.resource && tile.owner === me && !tile.improvement && offersImprovement(faction, improvementKindFor(tile.resource));

/** The Nile: Egypt has no Production, so it isn't offered the Quarry (core `view.rs::city_detail`). */
export const offersImprovement = (faction: string | undefined, kind: ImprovementKind): boolean => !(faction === 'Egypt' && kind === 'Quarry');

/** The technology note for an improvement: "Irrigated Farm with Intensive Cultivation". */
export const improvementUpgradeNote = (kind: ImprovementKind): string =>
  `${IMPROVEMENT_INFO[kind].upgradedName} with ${TECH_NAME[IMPROVEMENT_INFO[kind].upgradeTech]}`;
