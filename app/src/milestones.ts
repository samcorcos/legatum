// Milestone wording shared by the turn report and the celebration lane: what a city's growth
// unlocked and what a claimed border tile yields, from the viewer's post-turn view.
import { TERRAIN_NAME, itemName } from './data';
import { tileIndex } from './hex';
import { tileYields } from './resources';
import type { Faction, Hex, PlayerView, Yields } from './types';

const YIELD_ORDER: [keyof Yields, string][] = [['food', 'Food'], ['production', 'Production'], ['gold', 'Gold'], ['science', 'Science']];

/** "Settler" or "Settler and University": items the city's new population unlocked (core `population_unlocks`). */
export function growthUnlocks(view: PlayerView, city: number, population: number): string | null {
  const c = view.cities.find((x) => x.id === city);
  const faction = view.players.find((p) => p.id === c?.owner)?.faction;
  const names = (c?.detail?.population_unlocks ?? []).filter((u) => u.population === population).map((u) => itemName(u.item, faction));
  if (!names.length) return null;
  return names.length === 1 ? names[0] : `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
}

/** "+2 Food, +1 Production" for a tile as its owner would work it, or null when it yields nothing. */
function yieldText(y: Yields): string | null {
  const parts = YIELD_ORDER.filter(([k]) => y[k] !== 0).map(([k, name]) => `${y[k] > 0 ? '+' : '−'}${Math.abs(y[k])} ${name}`);
  return parts.length ? parts.join(', ') : null;
}

/** The claimed tile's terrain and yields, when the viewer knows the tile. */
export function claimedTile(view: PlayerView, tile: Hex, faction: Faction | undefined): { terrain: string; yields: string | null } | null {
  const known = view.tiles[tileIndex(tile, view.width, view.height)];
  if (!known?.terrain) return null;
  return { terrain: TERRAIN_NAME[known.terrain], yields: faction ? yieldText(tileYields(faction, known)) : null };
}
