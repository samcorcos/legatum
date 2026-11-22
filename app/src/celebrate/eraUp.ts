import { TECH_NAME, itemIcon, itemKey } from '../data';
import { eraName, eraNumeral, researchCatalog } from '../ui/eraResearch';
import type { Faction, PlayerView, ProductionItem, TechId } from '../types';

/** One thing the era puts in your hands: a unit, building, improvement or project its technologies unlock (this empire's own version). */
/** `item` gives the painted miniature (`unlockArt.ts`), looked up by the component. */
export interface EraBring { key: string; name: string; icon: string; item: ProductionItem; faction: Faction; kind: 'Unit' | 'Building' | 'Wonder' | 'Improvement' | 'Project' }

/** The board changes with the era: your cities are painted in the new era's style (`map/cityArt.ts`). */
interface EraCities {
  /** Size tier shown: your largest city's (`hamlet` .. `metropolis`, as `map/cityArt.ts` names them). */
  tier: 'hamlet' | 'town' | 'city' | 'metropolis';
  /** The era before (null for the first era of the game) and the era entered: 0-based, with names. */
  from: { era: number; name: string } | null;
  to: { era: number; name: string };
}

/** City size tiers by population (the art's own bands: 1-3, 4-7, 8-12, 13+; `map/cityArt.ts` `cityTier` is the same rule). */
const tierOf = (population: number): EraCities['tier'] => population >= 13 ? 'metropolis' : population >= 8 ? 'city' : population >= 4 ? 'town' : 'hamlet';

/** What the viewer's own era-up card shows: the era, whether they were first, and what is now researchable. */
export interface EraUpCard {
  era: number;
  eraName: string;
  numeral: string;
  first: boolean;
  /** The era's technologies, as the strict gate now offers them (known ones are listed too, marked). */
  techs: { tech: TechId; name: string; cost: number; known: boolean }[];
  /** Units, buildings and improvements the era's technologies open for this empire. */
  brings: EraBring[];
  cities: EraCities;
  /** Research is already set (the queue holds a tech of this era): the primary is Continue, not Choose research. */
  researchSet: boolean;
  /** What Return does: open the research screen when nothing is queued, else close the card. */
  primary: 'research' | 'continue';
}

const BRING_ORDER: Record<EraBring['kind'], number> = { Unit: 0, Building: 1, Wonder: 1, Improvement: 2, Project: 3 };

/** The card for the viewer entering `era`, from the view after the turn. */
/** `era` is 0-based (the gate's numbering); the objectives come from the view (the core numbers eras from 1). */
export function eraUpCardFor(view: PlayerView, era: number, first: boolean): EraUpCard {
  const r = view.research;
  const known = new Set(r.known);
  const me = view.players.find((p) => p.id === view.me);
  const faction = me?.faction ?? 'India';
  const eraTechs = researchCatalog(r).filter((t) => t.era === era);
  const techs = eraTechs.map((t) => ({
    tech: t.tech, name: TECH_NAME[t.tech], cost: t.cost.total, known: known.has(t.tech),
  }));
  const inEra = new Set(eraTechs.map((t) => t.tech));
  const seen = new Set<string>();
  const brings: EraBring[] = [];
  for (const u of r.unlocks) {
    const key = itemKey(u.item);
    if (!inEra.has(u.tech) || seen.has(key)) continue;
    seen.add(key);
    brings.push({ key, name: u.name, icon: itemIcon(u.item, faction), item: u.item, faction, kind: u.item.kind });
  }
  brings.sort((a, b) => BRING_ORDER[a.kind] - BRING_ORDER[b.kind]);
  const biggest = Math.max(0, ...view.cities.filter((c) => c.owner === view.me).map((c) => c.population));
  const tier = tierOf(biggest || 4);
  const researchSet = (r.queue?.length ?? 0) > 0;
  return {
    era, eraName: eraName(era), numeral: eraNumeral(era), first, techs, brings,
    cities: { tier, from: era > 0 ? { era: era - 1, name: eraName(era - 1) } : null, to: { era, name: eraName(era) } },
    researchSet, primary: researchSet ? 'continue' : 'research',
  };
}
