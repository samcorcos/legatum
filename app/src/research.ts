import { BUILDING_STATS, hasBuilding } from './data';
import type { Breakdown, PlayerView, TechUnlock } from './types';

/** Research completes only at resolution, at most once per empire per turn. */
export function researchTurns(cost: number, stored: number, science: number): number | null {
  const remaining = Math.max(0, cost - stored);
  if (remaining === 0) return 1;
  return science > 0 ? Math.max(1, Math.ceil(remaining / science)) : null;
}

/** Conditional construction estimate: start after research, from zero stored Production. */
export function firstBuildForecast(view: PlayerView, unlock: TechUnlock, researchTime: number | null) {
  if (researchTime === null) return null;
  if (unlock.cities) {
    const best = [...unlock.cities].filter((c) => !c.blocker && c.build_turns !== null).sort((a,b) => a.build_turns! - b.build_turns! || a.city - b.city)[0];
    const city = best && view.cities.find((c) => c.id === best.city);
    return best && city ? { city: city.name, buildTurns: best.build_turns!, totalTurns: researchTime + best.build_turns! } : null;
  }
  const item = unlock.item;
  const candidates = view.cities.filter((c) => c.owner === view.me && c.detail && c.detail.resistance === 0
    && c.detail.yields.total.production > 0
    && !(item.kind === 'Building' && (hasBuilding(c.detail.buildings, item.building) || c.population < (BUILDING_STATS[item.building].minPopulation ?? 0) || (BUILDING_STATS[item.building].predecessor && !c.detail.buildings.includes(BUILDING_STATS[item.building].predecessor!)))))
    .map((city) => ({ city, turns: Math.max(1, Math.ceil(unlock.cost / city.detail!.yields.total.production)) }))
    .sort((a, b) => a.turns - b.turns || a.city.id - b.city.id);
  const best = candidates[0];
  return best ? { city: best.city.name, buildTurns: best.turns, totalTurns: researchTime + best.turns } : null;
}

/**
 * Explain zero income after losing a capital or during resistance.
 * Null once Science is flowing.
 */
export function firstScienceForecast(view: PlayerView): string | null {
  if (view.economy.science_per_turn.total > 0) return null;
  const rule = 'Your capital makes +2 Science a turn from the day it is founded. Every city adds 1 Science for every 2 population, and Libraries and their upgrades add more.';
  const mine = view.cities.filter((c) => c.owner === view.me && c.detail);
  if (!mine.length) return `No Science income. ${rule}`;
  const recovery = mine.some((c) => c.detail!.resistance > 0)
    ? ' Cities in resistance produce nothing until resistance ends.' : '';
  const capital = mine.some((c) => c.is_capital)
    ? '' : ' You have no capital: the next city you found becomes your capital.';
  const growing = mine
    .filter((c) => c.detail!.resistance === 0 && c.population < 2 && c.detail!.turns_to_grow !== null)
    .sort((a, b) => a.detail!.turns_to_grow! - b.detail!.turns_to_grow!)[0];
  if (!growing) return `No Science income. ${rule}${capital}${recovery}`;
  const n = growing.detail!.turns_to_grow!;
  return `No Science income. ${rule}${capital}${recovery} ${growing.name} is forecast to reach population 2 in about ${n} turn${n === 1 ? '' : 's'}, adding 1 Science per turn afterward.`;
}

/** Tech diffusion, made visible: "−20%: 2 empires know it", read from the core's research cost
 * breakdown (economy.rs `tech_cost`: −10% per other living empire that knows it, at most −50%).
 * `short` is just the percentage for a tech card. Null when no other empire knows it. */
export function diffusionText(cost: Breakdown | undefined, short = false): string | null {
  if (!cost) return null;
  const base = cost.parts.find(([label]) => label === 'Listed cost')?.[1] ?? 0;
  const part = cost.parts.find(([label]) => label.startsWith('Tech diffusion ('));
  const empires = Number(part?.[0].match(/\((\d+) empires? knows? it\)/)?.[1] ?? 0);
  if (!part || !empires || base <= 0) return null;
  const pct = `−${Math.round(-part[1] * 100 / base)}%`;
  return short ? pct : `${pct}: ${empires} empire${empires === 1 ? ' knows' : 's know'} it`;
}
