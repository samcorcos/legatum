import { BUILDING_KINDS, UNIT_KINDS, UNIT_STATS, availableToPlayer, itemName, unitStatsFor } from '../data';
import type { GrowthProgress } from '../cityProgress';
import type { Faction, ProductionItem, ProductionOption, UnavailableOption, UnitKind } from '../types';

/**
 * Fold only real upgrade paths with no loss of combat or movement capability. A line leads with the
 * most advanced unit the city can start now: a newer one short of its strategic resource (Iron) is
 * the line's `better`, shown under it with the reason, as the AI builds (`best_buildable_in_line`).
 */
export function unitLines(options: ProductionOption[], faction: Faction) {
  const units = options.filter((o) => o.item.kind === 'Unit');
  const startable = units.filter((o) => !o.resource_blocker);
  const parent = (o: ProductionOption): ProductionOption => {
    if (o.item.kind !== 'Unit') return o;
    const stats = unitStatsFor(faction, o.item.unit);
    let kind: UnitKind | null = UNIT_STATS[o.item.unit].upgradesTo;
    let best = o;
    while (kind) {
      const candidate = units.find((u) => u.item.kind === 'Unit' && u.item.unit === kind);
      const next = unitStatsFor(faction, kind);
      if (candidate && next.moves >= stats.moves && next.strength >= stats.strength && next.ranged >= stats.ranged && next.range >= stats.range &&
        (next.strength > stats.strength || next.ranged > stats.ranged || next.moves > stats.moves || next.range > stats.range)) best = candidate;
      kind = UNIT_STATS[kind].upgradesTo;
    }
    return best;
  };
  // Oldest first: the further down the upgrade chain from the main unit, the earlier it is listed.
  const hopsTo = (o: ProductionOption, main: ProductionOption): number => {
    let hops = 0;
    for (let kind: UnitKind | null = o.item.kind === 'Unit' ? o.item.unit : null; kind && !(main.item.kind === 'Unit' && kind === main.item.unit); kind = UNIT_STATS[kind].upgradesTo) hops++;
    return hops;
  };
  // A unit with a startable successor that did not fold it is better at something: its card says what ("3 moves; the Arquebusier has 2").
  const keeps = (o: ProductionOption): string | undefined => {
    if (o.item.kind !== 'Unit') return undefined;
    let later: UnitKind | null = null;
    for (let kind: UnitKind | null = UNIT_STATS[o.item.unit].upgradesTo; kind; kind = UNIT_STATS[kind].upgradesTo) {
      const k = kind;
      if (startable.some((u) => u.item.kind === 'Unit' && u.item.unit === k)) later = k;
    }
    if (!later) return undefined;
    const mine = unitStatsFor(faction, o.item.unit), theirs = unitStatsFor(faction, later);
    const name = itemName({ kind: 'Unit', unit: later }, faction);
    const parts = ([['moves', 'moves'], ['strength', 'strength'], ['ranged', 'ranged strength'], ['range', 'range']] as const)
      .filter(([key]) => mine[key] > theirs[key]).map(([key, label]) => `${mine[key]} ${label}; the ${name} has ${theirs[key]}`);
    return parts.length ? parts.join(' · ') : undefined;
  };
  return units.filter((o) => parent(o) === o).map((top) => {
    const line = units.filter((o) => o !== top && parent(o) === top).sort((a, b) => hopsTo(b, top) - hopsTo(a, top));
    // Short of the resource for the newest unit: lead with the newest one that can start now.
    const lead = top.resource_blocker ? [...line].reverse().find((o) => !o.resource_blocker) : undefined;
    if (!lead) return { main: top, keeps: keeps(top), better: undefined, older: line };
    return { main: lead, keeps: keeps(lead), better: top, older: line.filter((o) => o !== lead) };
  });
}

/** Picker order, as core `all_items`: units, then buildings, then projects. */
function itemOrder(item: ProductionItem): number {
  if (item.kind === 'Unit') return UNIT_KINDS.indexOf(item.unit);
  if (item.kind === 'Building') return 100 + BUILDING_KINDS.indexOf(item.building);
  return 200;
}

export type PickerRow<T> = { row: T; unavailable?: undefined } | { row?: undefined; unavailable: UnavailableOption };

export interface BuildingLineNode {
  entry: { row: ProductionOption; unavailable?: undefined } | { row?: undefined; unavailable: UnavailableOption };
  upgrades: BuildingLineNode[];
}

/** Nest only core-identified, missing in-city building predecessors; preserve picker order. */
export function nestBuildingUpgrades(rows: BuildingLineNode['entry'][]): BuildingLineNode[] {
  const nodes = rows.map((entry) => ({ entry, upgrades: [] as BuildingLineNode[] }));
  const keyOf = (node: BuildingLineNode) => {
    const item = node.entry.row ? node.entry.row.item : node.entry.unavailable!.item;
    return item.kind === 'Building' ? item.building : null;
  };
  const root: BuildingLineNode[] = [];
  for (const node of nodes) {
    const unavailable = node.entry.unavailable;
    const prerequisite = unavailable?.prerequisite_building;
    const parent = prerequisite ? nodes.find((candidate) => keyOf(candidate) === prerequisite) : undefined;
    if (parent && parent !== node) parent.upgrades.push(node);
    else root.push(node);
  }
  return root;
}

/** Faction exclusions use structured eligibility; reason wording can change independently. */
export function isPermanentlyUnavailable(u: UnavailableOption, faction: Faction | undefined): boolean {
  return faction !== undefined && !availableToPlayer(faction, u.item);
}

/** Greyed-out items take their usual place in a section, without reordering the choosable ones. */
export function withUnavailable<T>(rows: T[], itemOf: (row: T) => ProductionItem, unavailable: UnavailableOption[]): PickerRow<T>[] {
  const out: PickerRow<T>[] = rows.map((row) => ({ row }));
  for (const u of [...unavailable].sort((a, b) => itemOrder(a.item) - itemOrder(b.item))) {
    const at = out.findIndex((r) => itemOrder(r.row ? itemOf(r.row) : r.unavailable!.item) > itemOrder(u.item));
    out.splice(at < 0 ? out.length : at, 0, { unavailable: u });
  }
  return out;
}

const turnsText = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

/**
 * Why an item can't be chosen here yet, in plain words with what fixes it. `short` fits the card;
 * `growth` is the forecast for population gates (null otherwise); `full` is the tooltip.
 */
export function unavailableText(u: UnavailableOption, cityName: string, population: number, growth: Pick<GrowthProgress, 'state' | 'turns'> | null, name: string) {
  const pop = /population (\d+) \(currently (\d+)\)/.exec(u.reason);
  if (pop) {
    const required = Number(pop[1]);
    const turns = growth?.state === 'growing' ? growth.turns : null;
    const growthText = turns === null
      ? growth?.state === 'resisting' ? `${cityName} can't grow during resistance`
        : growth?.state === 'starving' ? `${cityName} is starving: work more Food first`
        : `${cityName} is not growing: work more Food or build something else first`
      : required === population + 1 ? `${cityName} grows to ${required} in ${turnsText(turns)}`
        : `${cityName} has ${population}; next growth in ${turnsText(turns)}`;
    const why = u.item.kind === 'Unit' && u.item.unit === 'Settler' ? ' A Settler takes 1 population with it to found the new city.' : '';
    return { short: `Needs ${required} population`, growth: growthText, line: `Needs ${required} population · ${growthText}`, full: `${name} needs ${required} population; ${cityName} has ${population}.${why} ${growthText}.` };
  }
  const tier = /^Requires (.+)$/.exec(u.reason);
  const short = tier ? `Build ${tier[1]} here first` : u.reason;
  return { short, growth: null, line: short, full: `${name} can't be built in ${cityName} now: ${tier ? `build ${/^[AEIOU]/.test(tier[1]) ? 'an' : 'a'} ${tier[1]} here first` : short}.` };
}

/**
 * A card's progress, when there is any: the active build's own progress, an item's banked progress,
 * or unassigned Production (overflow, idle storage) that joins whatever starts next.
 */
export function progressText(stored: number, banked: number, cost: number, active: boolean): string | null {
  if (stored <= 0) return null;
  return `${stored}/${cost} ${active ? 'so far' : banked > 0 ? 'banked' : 'carried over'}`;
}
