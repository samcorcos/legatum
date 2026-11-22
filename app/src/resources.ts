// Map resources (G11 increment 1): display data exported from crates/core/src/resources.rs by
// crates/ai/tests/ui_rules.rs, plus the words the UI uses for them. The core is authoritative.
import { IMPROVEMENT_INFO, RULES, TECH_NAME, forgesName, itemName } from './data';
import { improvementGain, improvementKindFor, improvementUpgradeYields } from './improvements';
import { empireOfPlayer, PEOPLE_NAME } from './empires';
import { distance, hexAt } from './hex';
import { terrainYields } from './settlement';
import type { BuildingKind, Faction, Hex, PlayerView, Resource, ResourceClass, SupplyView, TechId, Terrain, TileView, UnitKind, Yields } from './types';

export interface ResourceInfo {
  name: string;
  class: ResourceClass;
  terrains: Terrain[];
  bonus: Yields;
  revealedBy: TechId | null;
}

export const RESOURCES: Resource[] = ['Wheat', 'Cattle', 'Stone', 'Copper', 'Iron', 'Niter', 'Coal', 'Oil', 'Silicon'];
/** Core's derived `Resource` order (the enum's declaration order), which keys its BTreeMaps and breaks ties: Coal comes before Niter, Oil and Silicon. */
export const RESOURCE_ORDER: Resource[] = ['Wheat', 'Cattle', 'Stone', 'Copper', 'Iron', 'Coal', 'Niter', 'Oil', 'Silicon'];
/** The strategic resources in era order (core `Resource::STRATEGIC`). */
export const STRATEGIC_RESOURCES = RULES.supply.strategic as unknown as Resource[];
/** Strategic resources begin in the Classical era; Ancient has no supply requirement. */
const ERA_RESOURCES = RULES.supply.eraResources as unknown as (Resource | null)[];
/** The resource an era's new units and key building use; the last era's for any later one. */
export const eraResource = (era0: number): Resource | null => ERA_RESOURCES[Math.max(0, Math.min(era0, ERA_RESOURCES.length - 1))];
export const RESOURCE_INFO = RULES.resources as unknown as Record<Resource, ResourceInfo>;
/** Deposits a technology makes visible on the map. Bonus resources are already visible. */
export const revealedByTech = (tech: TechId): Resource[] => STRATEGIC_RESOURCES.filter((r) => RESOURCE_INFO[r].revealedBy === tech);
/** Supply a known strategic deposit inside your borders gives, and what a Mine on it adds. */
export const SUPPLY_PER_DEPOSIT: number = RULES.supply.perDeposit;
export const SUPPLY_PER_MINE: number = RULES.supply.perMine;
/** Percent of base strength an unsupplied unit loses in every fight. */
export const UNSUPPLIED_STRENGTH_PERCENT: number = RULES.supply.unsuppliedStrengthPercent;
export const STRATEGIC_SITE_VALUE: number = RULES.supply.strategicSiteValue;
const UNIT_NEEDS = RULES.supply.unitNeeds as unknown as Partial<Record<UnitKind, Resource>>;
const BUILDING_NEEDS = RULES.supply.buildingNeeds as unknown as Partial<Record<BuildingKind, Resource>>;

/** Who needs each strategic resource, in the words a missing-resource alert uses ("your gunpowder units and Banks"). */
export const RESOURCE_USERS: Partial<Record<Resource, string>> = {
  Iron: 'your Classical and Medieval units, Forges and Universities',
  Niter: 'your gunpowder units and Banks',
  Coal: 'your Industrial units and Factories',
  Oil: 'your Machine Age, World War and Modern units, aircraft and Research Labs',
  Silicon: 'your Future units',};

/** Who needs a resource, in the viewer's faction's building names. */
export const resourceUsers = (resource: Resource, faction?: Faction | string): string | undefined => RESOURCE_USERS[resource]?.replace(/\bForges\b/, forgesName(faction as Faction | undefined));

/** Plain language for the units and key building newly using this resource in the current era. */
export function resourceUsersThisEra(resource: Resource, era: number, faction?: Faction | string): string {
  const forges = forgesName(faction as Faction | undefined);
  if (resource === 'Iron') return era === 1
    ? `Phalangites, Composite Archers, Cataphracts, ${forges} and Siege`
    : era === 2 ? `Pikemen, Crossbowmen, Knights, Universities, ${forges} and Siege`
      : `${forges}, Universities, Siege and older Iron units`;
  if (resource === 'Niter') return era === 3 ? 'Musketmen, Arquebusiers, Cuirassiers and Banks' : 'Banks and older gunpowder units';
  if (resource === 'Coal') return era === 4 ? 'Riflemen, Field Cannons, Cavalry and Factories' : 'Factories and older Industrial units';
  if (resource === 'Oil') return era === 5 ? 'Trench Infantry, Howitzers, Early Tanks and Biplanes'
    : era === 6 ? 'Infantry, Artillery, Tanks, Fighters, Bombers and Research Labs'
      : era === 7 ? 'Mechanised Infantry, Rocket Artillery, Main Battle Tanks, Jet Fighters, Jet Bombers and Research Labs' : 'Research Labs and older Oil units';
  if (resource === ERA_RESOURCES[ERA_RESOURCES.length - 1]) return 'Hunter-Killers, Combat Drones, Autonomous Armour and Stealth aircraft';
  return 'bonus Gold on a worked Copper tile';
}

/** The building tiers that use supply, lowest era first (core `SUPPLY_TIERS`; also the order of `SupplyUser`). */
export const SUPPLY_TIERS: BuildingKind[] = (['Market', 'MachineShop', 'College', 'Clearinghouse', 'Autoworks', 'InferenceCore'] as BuildingKind[]).filter((b) => b in BUILDING_NEEDS);

export const isStrategic = (r: Resource): boolean => RESOURCE_INFO[r].class === 'Strategic';

/** The supply a building tier uses while it stands (Forge: Iron, Factory: Coal). */
export const buildingNeed = (b: BuildingKind): Resource | null => BUILDING_NEEDS[b] ?? null;

/** The supply a unit uses while it exists (Pikeman: Iron). */
export const unitNeed = (u: UnitKind): Resource | null => UNIT_NEEDS[u] ?? null;

/** The supply an upgrade from `from` adds: the target's need, when `from` lacks it. */
export function upgradeNeed(from: UnitKind, upgradesTo: UnitKind | null | undefined): Resource | null {
  const to = upgradesTo ? unitNeed(upgradesTo) : null;
  return to && unitNeed(from) !== to ? to : null;
}

const YIELD_WORD: [keyof Yields, string][] = [
  ['food', 'Food'],
  ['production', 'Production'],
  ['gold', 'Gold'],
  ['science', 'Science'],
];

/** "+1 Production" */
function bonusText(r: Resource, b: Yields = RESOURCE_INFO[r].bonus): string {
  return YIELD_WORD.filter(([k]) => b[k]).map(([k, w]) => `+${b[k]} ${w}`).join(', ');
}

/** "Uses 1 Iron" */
export const needText = (r: Resource): string => `Uses 1 ${RESOURCE_INFO[r].name}`;

/** "an Iron", "a Coal". */
export const withArticle = (name: string): string => `${/^[AEIOU]/.test(name) ? 'an' : 'a'} ${name}`;

/** The improvement a deposit of this resource takes, named by the data ("Mine", "Oil Well"; core `improvements::kind_for`). */
const worksName = (r: Resource): string => IMPROVEMENT_INFO[improvementKindFor(r)].name;

/** The same with its article: "a Mine", "an Oil Well". */
export const aWorks = (r: Resource): string => withArticle(worksName(r));

/**
 * The fix for a shortage, said the same way everywhere (core `fix_clause`): "Bring another Iron deposit
 * inside your borders, or build a Mine on one for +1 Iron" (an Oil Well for Oil). `any` says "another" (you already have one).
 */
export const supplyPerDeposit = (faction?: string): number => SUPPLY_PER_DEPOSIT * (faction === 'Germany' ? 2 : 1);
export const supplyPerMine = (faction?: string): number => SUPPLY_PER_MINE * (faction === 'Germany' ? 2 : 1);

export function fixClause(r: Resource, any: boolean, faction?: string): string {
  const name = RESOURCE_INFO[r].name;
  return `Bring ${any ? `another ${name} deposit` : `${withArticle(name)} deposit`} inside your borders, or build ${aWorks(r)} on one for +${supplyPerMine(faction)} ${name}.`;
}

/** How much of a supply is at work: everything that wants it, less what is idle. */
export const supplyAtWork = (s: SupplyView): number => s.used.total - (s.idle?.length ?? 0);

/** The totals in words, never "in use" for what is only wanted: "3 of 5 Iron at work", or when short "needs 6, supplies 4". */
export function supplyTotalsText(s: SupplyView): string {
  if (s.supplied.total === 0 && s.used.total === 0) return `no ${RESOURCE_INFO[s.resource].name} supplied`;
  return (s.idle?.length ?? 0) > 0
    ? `needs ${s.used.total}, supplies ${s.supplied.total}`
    : `${supplyAtWork(s)} of ${s.supplied.total} ${RESOURCE_INFO[s.resource].name} at work`;
}

/** The yield a resource's improvement adds, named: "Production" for Iron. */
export const bonusYieldName = (r: Resource): string => YIELD_WORD.filter(([k]) => RESOURCE_INFO[r].bonus[k]).map(([, w]) => w).join(' and ');

/** "1 building or unit is idle", "3 buildings or units are idle". */
export const idleCountText = (n: number): string => (n === 1 ? '1 building or unit is idle' : `${n} buildings or units are idle`);

/** An idle building or unit named by the core's label ("Forge in Bern", "Pikeman"), with the tile that shows it. */
export interface IdleEntry { label: string; hex: Hex | null }

/**
 * Where each idle entry of a supply is. A building's label ends in its city ("Forge in Bern"); a unit's is its
 * name, so repeated labels take your idle units of that kind from the newest id down. Aircraft route to their
 * hangar city because they have no map tile.
 */
export function idleEntries(view: PlayerView, s: SupplyView): IdleEntry[] {
  const cities = view.cities.filter((c) => c.owner === view.me).sort((a, b) => b.name.length - a.name.length);
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const units = view.units.filter((u) => u.owner === view.me && u.unsupplied === s.resource).sort((a, b) => b.id - a.id);
  const aircraft = (view.aircraft ?? []).filter((a) => a.owner === view.me && a.unsupplied === s.resource).sort((a, b) => b.id - a.id);
  const taken = new Set<number>();
  return (s.idle ?? []).map((label) => {
    const city = cities.find((c) => label.endsWith(` in ${c.name}`));
    if (city) return { label, hex: city.pos };
    const unit = units.find((u) => !taken.has(u.id) && itemName({ kind: 'Unit', unit: u.kind as UnitKind }, faction) === label);
    if (unit) taken.add(unit.id);
    const plane = aircraft.find((a) => !taken.has(a.id) && itemName({ kind: 'Unit', unit: a.kind }, faction) === label);
    if (plane) taken.add(plane.id);
    return { label, hex: unit?.pos ?? cities.find((c) => c.id === plane?.city)?.pos ?? null };
  });
}

/** "Each deposit inside your borders supplies 2 Iron, 3 with a Mine." (3 with an Oil Well for Oil.) */
export const supplyRuleText = (r: Resource, faction?: string): string => `Each ${RESOURCE_INFO[r].name} deposit inside your borders supplies ${supplyPerDeposit(faction)} ${RESOURCE_INFO[r].name}, ${supplyPerDeposit(faction) + supplyPerMine(faction)} with ${aWorks(r)}.`;

/**
 * The tile card's benefits for a resource, one short line each: what it gives the tile's improvement
 * and, for strategic ones, the supply a deposit inside your borders gives.
 */
export function resourceBenefitLines(r: Resource, known: readonly TechId[], faction?: string): string[] {
  const info = RESOURCE_INFO[r];
  const gain = nileYields(faction, improvementGain(r, known));
  const lines = YIELD_WORD.some(([k]) => gain[k]) ? [`${bonusText(r, gain)} with ${aWorks(r)}`] : info.class === 'Strategic' ? [] : ['Nothing to build here: you have no Production'];
  if (info.class === 'Strategic') lines.push(`Supplies ${supplyPerDeposit(faction)} ${info.name} inside your borders, ${supplyPerDeposit(faction) + supplyPerMine(faction)} with ${aWorks(r)}`);
  return lines;
}

/** "Coal: revealed by Machine Tools" for strategic resources the viewer can't see yet. */
export const revealHint = (r: Resource): string => {
  const tech = RESOURCE_INFO[r].revealedBy;
  return `${RESOURCE_INFO[r].name}: revealed by ${tech ? TECH_NAME[tech] : 'no technology'}`;
};

const ZERO: Yields = { food: 0, production: 0, gold: 0, science: 0 };

/** A resource's worked-tile bonus (the view only lists resources the viewer knows). */
const resourceBonus = (r: Resource | null | undefined): Yields => (r ? RESOURCE_INFO[r].bonus : ZERO);

/** An improved resource's bonus, with the upgrade's extra yield when the tile's owner has it. */
function improvedTileBonus(r: Resource, upgraded: boolean): Yields {
  const b = resourceBonus(r), up = upgraded ? improvementUpgradeYields(r) : ZERO;
  return { food: b.food + up.food, production: b.production + up.production, gold: b.gold + up.gold, science: b.science + up.science };
}

/**
 * Faction-adjusted terrain plus the improved resource: what a citizen working this tile earns.
 * The Yields overlay and the tile card both read this, so the glance and the detail agree. For Egypt
 * that is the Nile's double Food and no Production.
 */
export function tileYields(faction: Faction, tile: Pick<TileView, 'terrain' | 'resource' | 'improvement' | 'landmark'>): Yields {
  if (tile.landmark) return { food: 0, production: 0, gold: 0, science: 0 };
  if (!tile.terrain) return ZERO;
  // A resource yields its bonus only once its tile is improved (and the upgrade's extra once the owner has it).
  const t = terrainYields(faction, tile.terrain);
  const b = tile.resource && tile.improvement ? improvedTileBonus(tile.resource, !!tile.improvement.upgraded) : ZERO;
  return nileYields(faction, { food: t.food + b.food, production: t.production + b.production, gold: t.gold + b.gold, science: t.science + b.science });
}

/** The Nile (core `economy::nile`): Egypt's Food is doubled and it has no Production. Other factions' yields pass through. */
export function nileYields(faction: Faction | string | undefined, y: Yields): Yields {
  return faction === 'Egypt' ? { ...y, food: Math.floor((y.food * RULES.egyptNileFoodPercent) / 100), production: 0 } : y;
}

/** What the viewer has seen of a resource's deposits that it could still take: held by another empire, or nobody's. */
export interface DepositSurvey {
  /** The other empires holding one, by name, in seat order. */
  empires: string[];
  /** How many deposits those empires hold. */
  held: number;
  unclaimed: number;
  /** The nearest unclaimed deposit to your nearest city (else unit), with the nearest of either kind for "Show on map". */
  nearestUnclaimed: Hex | null;
  nearest: Hex | null;
}

/** Where the deposits of a resource the viewer has seen lie. Your own team's deposits are not counted: they already supply you. */
export function depositHolders(view: Pick<PlayerView, 'tiles' | 'players' | 'me' | 'width'> & Partial<Pick<PlayerView, 'cities' | 'units'>>, resource: Resource): DepositSurvey {
  const team = view.players.find((p) => p.id === view.me)?.team;
  const origins: Hex[] = [...(view.cities ?? []).filter((c) => c.owner === view.me).map((c) => c.pos), ...(view.cities?.some((c) => c.owner === view.me) ? [] : (view.units ?? []).filter((u) => u.owner === view.me).map((u) => u.pos))];
  const away = (h: Hex) => (origins.length ? Math.min(...origins.map((o) => distance(o, h))) : 0);
  const holders = new Set<number>();
  let held = 0, unclaimed = 0;
  let nearest: Hex | null = null, nearestUnclaimed: Hex | null = null;
  view.tiles.forEach((t, i) => {
    if (t.resource !== resource || !t.explored) return;
    const hex = hexAt(i, view.width);
    if (t.owner === null) {
      unclaimed++;
      if (!nearestUnclaimed || away(hex) < away(nearestUnclaimed)) nearestUnclaimed = hex;
    } else if (t.owner !== view.me && view.players.find((p) => p.id === t.owner)?.team !== team) {
      held++; holders.add(t.owner);
    } else return;
    if (!nearest || away(hex) < away(nearest)) nearest = hex;
  });
  return { empires: [...holders].sort((a, b) => a - b).map((id) => empireOfPlayer(view, id)), held, unclaimed, nearestUnclaimed, nearest };
}

const PEOPLES = new Set(Object.values(PEOPLE_NAME));
/** An empire named in a sentence: peoples take "the" (the Mongols, the Celts), countries and cities do not (Switzerland, Athens). */
export const withThe = (name: string): string => (PEOPLES.has(name.split(' · ')[0]) ? `the ${name}` : name);

/** "Switzerland holds", "The Mongols hold", "Byzantium and the Celts hold": who holds something, in a sentence. */
export function empireHolds(names: readonly string[]): string {
  const list = listEmpires(names);
  const plural = names.length > 1 || PEOPLES.has(names[0].split(' · ')[0]);
  return `${list.replace(/^./, (c) => c.toUpperCase())} ${plural ? 'hold' : 'holds'}`;
}

/** "the Mongols", "the Mongols and Switzerland", "the Mongols, the Celts and Sparta": empires in a sentence. */
export function listEmpires(names: readonly string[]): string {
  const named = names.map(withThe);
  return named.length < 2 ? named.join('') : `${named.slice(0, -1).join(', ')} and ${named[named.length - 1]}`;
}

/** The strategic resources you lack: revealed to you and none supplied, or some of what needs it idle for want of it. */
export function lackedResources(supply: readonly SupplyView[] | undefined): Set<Resource> {
  return new Set((supply ?? []).filter((s) => s.revealed && (s.supplied.total === 0 || (s.idle?.length ?? 0) > 0)).map((s) => s.resource));
}
