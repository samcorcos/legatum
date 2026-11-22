import type { CityView, Hex, PlayerView, Terrain, Wonder, Yields } from './types';
import { hexEq, tileIndex } from './hex';

export const WONDERS: Wonder[] = [
  'MountFuji', 'VictoriaFalls', 'GrandCanyon', 'GiantsCauseway',
  'Uluru', 'HaLongBay', 'IguazuFalls', 'GreatBarrierReef',
];

export const WONDER_INFO: Record<Wonder, { name: string; slug: string; terrain: Terrain[]; yield: keyof Yields }> = {
  MountFuji: { name: 'Mount Fuji', slug: 'mount-fuji', terrain: ['Mountains'], yield: 'science' },
  VictoriaFalls: { name: 'Victoria Falls', slug: 'victoria-falls', terrain: ['Hills'], yield: 'food' },
  GrandCanyon: { name: 'Grand Canyon', slug: 'grand-canyon', terrain: ['Desert'], yield: 'science' },
  GiantsCauseway: { name: "Giant's Causeway", slug: 'giants-causeway', terrain: ['Hills'], yield: 'production' },
  Uluru: { name: 'Uluru', slug: 'uluru', terrain: ['Desert'], yield: 'gold' },
  HaLongBay: { name: 'Ha Long Bay', slug: 'ha-long-bay', terrain: ['Hills'], yield: 'food' },
  IguazuFalls: { name: 'Iguazu Falls', slug: 'iguazu-falls', terrain: ['Forest'], yield: 'food' },
  GreatBarrierReef: { name: 'Great Barrier Reef', slug: 'great-barrier-reef', terrain: ['Coast'], yield: 'science' },
};

export function wonderBonus(wonder: Wonder, base: Yields): Yields {
  const out = { food: 0, production: 0, gold: 0, science: 0 };
  const key = WONDER_INFO[wonder].yield;
  out[key] = Math.max(0, Math.floor((base[key] + 3) / 4));
  return out;
}

export function wonderReward(wonder: Wonder): { gold: number; science: number } {
  return ['MountFuji', 'GrandCanyon', 'HaLongBay', 'GreatBarrierReef'].includes(wonder)
    ? { gold: 0, science: 50 } : { gold: 80, science: 0 };
}

/** Public assets keep this URL usable by the app and Playwright's Node fixtures. */
export function wonderArt(wonder: Wonder, style: 'a' | 'b'): string {
  const { slug } = WONDER_INFO[wonder];
  return `/wonders/${slug}-${style}-${style === 'a' ? '512' : '128'}.png`;
}

export const WONDER_PERCENT = 25;
const YIELD_WORD: Record<keyof Yields, string> = { food: 'Food', production: 'Production', gold: 'Gold', science: 'Science' };

/** "Science": the one yield a wonder boosts. */
export const wonderYieldWord = (wonder: Wonder): string => YIELD_WORD[WONDER_INFO[wonder].yield];

/** The map badge's text: the bonus in yield icons ("+25%" then the yield's icon). */
export const wonderBadgeText = (): string => `+${WONDER_PERCENT}%`;

/** Plain words: what owning the tile does, in two parts so a screen can set the amount in bold with its icon. */
export const wonderBonusParts = (wonder: Wonder): { lead: string; amount: string } =>
  ({ lead: 'The city that owns this tile gets ', amount: `+${WONDER_PERCENT}% ${wonderYieldWord(wonder)}` });
export const wonderBonusSentence = (wonder: Wonder): string => { const p = wonderBonusParts(wonder); return `${p.lead}${p.amount}.`; };

/** "50 Science" or "80 Gold": the one-time discovery reward. */
export function wonderRewardText(wonder: Wonder): string {
  const r = wonderReward(wonder);
  return r.gold ? `${r.gold} Gold` : `${r.science} Science`;
}

/** The discovery line: the reward while it is still open, a plain note once it has been paid. */
export function wonderDiscoveryLine(wonder: Wonder, found: boolean): string {
  return found
    ? 'Discovery reward: already paid to the first empire to see it.'
    : `Discovery reward: ${wonderRewardText(wonder)}, paid once to the first empire to see it.`;
}

/** The label the core puts on a city's yield breakdown for a wonder ("Grand Canyon (+25%)"), or null. */
export function wonderOfLabel(label: string): Wonder | null {
  const hit = WONDERS.find((w) => label === `${WONDER_INFO[w].name} (+${WONDER_PERCENT}%)`);
  return hit ?? null;
}

/** A breakdown row's label, "Grand Canyon +25%". */
export const wonderLineLabel = (wonder: Wonder): string => `${WONDER_INFO[wonder].name} +${WONDER_PERCENT}%`;

export interface OwnedWonder { wonder: Wonder; hex: Hex; /** What the city gains per turn, from its own breakdown; null when its detail is not visible. */ amount: number | null }

/** The wonders inside a city's territory, with what each adds to it per turn (from the city's own breakdown). */
export function cityWonders(view: Pick<PlayerView, 'tiles' | 'width' | 'height'>, city: Pick<CityView, 'detail'>): OwnedWonder[] {
  const detail = city.detail;
  if (!detail) return [];
  const out: OwnedWonder[] = [];
  for (const hex of detail.territory) {
    const wonder = view.tiles[tileIndex(hex, view.width, view.height)]?.wonder;
    if (!wonder) continue;
    const row = detail.yields.sources.find((s) => wonderOfLabel(s.label) === wonder);
    out.push({ wonder, hex, amount: row ? row.yields[WONDER_INFO[wonder].yield] : null });
  }
  return out;
}

/** The city whose territory holds a wonder tile, among the cities the viewer can see into (their own). */
export function holdingCity(view: Pick<PlayerView, 'cities'>, hex: Hex): CityView | null {
  return view.cities.find((c) => c.detail?.territory.some((h) => hexEq(h, hex))) ?? null;
}

/** The wonder art's alpha: strong in the default view, quieter under every lens (the lens reads first), and a little quieter with the Yields overlay on so its icons read over the painting. */
export function wonderArtAlpha(visible: boolean, lens: string | undefined, yieldsOn = false): number {
  const base = visible ? 0.95 : 0.68;
  return lens && lens !== 'default' ? base * 0.55 : yieldsOn ? base * 0.8 : base;
}
