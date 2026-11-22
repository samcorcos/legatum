import { ERA_NAMES, TECH_DIFFUSION_MAX, TECH_DIFFUSION_PER_EMPIRE } from '../data';
import type { Breakdown, PlayerId, ResearchView, TechId, TechOption } from '../types';

// The strict era gate, as the research screen sees it (core `eras`): an empire may research only the unknown
// techs of its current era, so the screen shows that era's cards and only names the later eras.

const ERA_NUMERALS = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX'] as const;
export const eraName = (era: number) => ERA_NAMES[era] ?? `Era ${era + 1}`;
export const eraNumeral = (era: number) => ERA_NUMERALS[era] ?? String(era + 1);

type EraState = 'past' | 'current' | 'locked';
export interface EraRow {
  era: number;
  name: string;
  numeral: string;
  total: number;
  known: number;
  state: EraState;
  /** The era's techs in research order, for the read-only peek at a later era. */
  techs: TechId[];
}

/** The techs this empire's research screens show: every tech but those its faction skips (they give it nothing). */
export function researchCatalog(r: ResearchView): TechOption[] {
  const skipped = new Set(r.skipped ?? []);
  return r.catalog.filter((t) => !skipped.has(t.tech));
}

/** One row per era: Ancient first. The current era is the first that still has an unknown tech. */
export function eraRows(r: ResearchView): EraRow[] {
  const known = new Set(r.known);
  const byEra = new Map<number, TechOption[]>();
  for (const t of researchCatalog(r)) byEra.set(t.era, [...(byEra.get(t.era) ?? []), t]);
  const eras = [...new Set([...byEra.keys(), r.era])].sort((a, b) => a - b);
  return eras.map((era): EraRow => {
    const techs = (byEra.get(era) ?? []).map((t) => t.tech);
    return {
      era, name: eraName(era), numeral: eraNumeral(era),
      total: era === r.era && r.era_total ? r.era_total : techs.length,
      known: techs.filter((t) => known.has(t)).length,
      state: era < r.era ? 'past' : era === r.era ? 'current' : 'locked',
      techs,
    };
  });
}

/** The current era's cards: every tech of the era, known ones included, in research order. */
export function currentEraTechs(r: ResearchView): TechOption[] {
  return researchCatalog(r).filter((t) => t.era === r.era);
}

/** Who entered each era first, as the empire's id: `era_firsts` is public. */
export function eraFirst(r: ResearchView, era: number): PlayerId | null {
  return r.era_firsts.find(([e]) => e === era)?.[1] ?? null;
}

// ---- Queue forecast --------------------------------------------------------------------------

export interface QueueSlot {
  tech: TechId;
  cost: number;
  /** Turns from now until it is known (counting the techs before it); null with no Science income. */
  turns: number | null;
  /** The turn it is known from, `turn + turns`; null with no Science income. */
  arrivalTurn: number | null;
}

/**
 * Where each queued tech arrives, in queue order, at today's Science and prices. Core's `research.schedule`
 * is the same sum (cumulative cost less stored Science over income); this version also keeps arrivals
 * strictly increasing because the engine finishes at most one tech per turn, and it follows an unsent
 * reorder of the queue rather than only the saved one.
 */
export function queueSchedule(r: ResearchView, queue: readonly TechId[], science: number, turn: number): QueueSlot[] {
  const price = new Map(r.catalog.map((t) => [t.tech, t.cost.total]));
  let cumulative = 0;
  let last = 0;
  let stuck = false;
  return queue.map((tech): QueueSlot => {
    const cost = price.get(tech) ?? 0;
    cumulative += cost;
    const remaining = cumulative - r.progress;
    const needed = remaining <= 0 ? 1 : science > 0 ? Math.max(1, Math.ceil(remaining / science)) : null;
    if (needed === null || stuck) { stuck = true; return { tech, cost, turns: null, arrivalTurn: null }; }
    last = Math.max(needed, last + 1);
    return { tech, cost, turns: last, arrivalTurn: turn + last };
  });
}

export const turnsText = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

/** "Turn 24 · in 6 turns", the forecast line of a queued tech. */
export function arrivalText(slot: Pick<QueueSlot, 'turns' | 'arrivalTurn'>, noScience = false): string {
  if (slot.turns === null || slot.arrivalTurn === null) return noScience ? 'No Science yet' : 'Needs Science';
  return `Turn ${slot.arrivalTurn} · in ${turnsText(slot.turns)}`;
}

// ---- Queue edits -----------------------------------------------------------------------------

/** Swap an entry with its neighbour (`by` is -1 to move earlier, +1 later). Out-of-range moves change nothing. */
export function moveInQueue<T>(queue: readonly T[], index: number, by: -1 | 1): T[] {
  const to = index + by;
  if (index < 0 || index >= queue.length || to < 0 || to >= queue.length) return [...queue];
  const next = [...queue];
  [next[index], next[to]] = [next[to], next[index]];
  return next;
}

/** The tech to the front, the rest keeping their order behind it. */
export function researchFirst(queue: readonly TechId[], tech: TechId): TechId[] {
  return [tech, ...queue.filter((t) => t !== tech)];
}

/** Add to the end unless it is already queued. */
export function appendToQueue(queue: readonly TechId[], tech: TechId): TechId[] {
  return queue.includes(tech) ? [...queue] : [...queue, tech];
}

/** The auto-order: every researchable tech, cheapest first (discounted ones lead), ties in the era's order. */
export function cheapestFirst(r: ResearchView): TechId[] {
  return currentEraTechs(r).filter((t) => t.researchable)
    .map((t, i) => ({ t, i }))
    .sort((a, b) => a.t.cost.total - b.t.cost.total || a.i - b.i)
    .map(({ t }) => t.tech);
}

// ---- Cost text -------------------------------------------------------------------------

export interface CostParts {
  total: number;
  /** The listed price before diffusion. */
  base: number;
  diffusion: { percent: number; empires: number } | null;
}

/** Read the cost breakdown the core sends: "Listed cost", "Tech diffusion (N empires know it)". */
export function costParts(cost: Breakdown): CostParts {
  const part = (label: RegExp) => cost.parts.find(([l]) => label.test(l));
  const base = part(/^Listed cost/)?.[1] ?? cost.total;
  const diffusion = part(/^Tech diffusion/);
  const empires = Number(diffusion?.[0].match(/\((\d+) empires? know/)?.[1] ?? 0);
  // The rule's own percent, not the rounded Science it took off: a 16-Science tech at 20% loses 4, which is 25% of it.
  const percent = diffusion && diffusion[1] < 0 ? Math.min(empires * TECH_DIFFUSION_PER_EMPIRE, TECH_DIFFUSION_MAX) : 0;
  return {
    total: cost.total, base,
    diffusion: empires > 0 && percent > 0 ? { percent, empires } : null,
  };
}

/** "−30%: known by 3 empires" for the card's discount line. */
export function diffusionLine(p: CostParts): string | null {
  return p.diffusion ? `−${p.diffusion.percent}%: known by ${p.diffusion.empires} empire${p.diffusion.empires === 1 ? '' : 's'}` : null;
}

/** " (Medieval era)" when `tech` lies beyond the viewer's current era, so a locked build says how far off it is; empty otherwise. */
export function requiresEra(view: { research: Pick<ResearchView, 'era' | 'catalog'> }, tech: TechId): string {
  const era = view.research.catalog.find((t) => t.tech === tech)?.era;
  return era !== undefined && era > view.research.era ? ` (${eraName(era)} era)` : '';
}
