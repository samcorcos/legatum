import type { BuildingKind, Hex, PlayerView } from '../types';
import { hexAt, tileIndex } from '../hex';

export interface ConstructionSite {
  city: number;
  owner: number;
  tile: number;
  hex: Hex;
  building: BuildingKind;
  era: number;
  k: number;
  stage: number;
  seed: number;
  walls: boolean;
}

export const constructionStage = (k: number): number => k >= 1 ? 4 : k >= .75 ? 3 : k >= .4 ? 2 : k >= .1 ? 1 : 0;
const clamp = (k: number) => Math.max(0, Math.min(1, Number.isFinite(k) ? k : 0));
const memo = new WeakMap<PlayerView, ConstructionSite[]>();

/** A chosen wonder site and Walls under construction are public; own sites use their private production detail. */
export function sitesOf(view: PlayerView): ConstructionSite[] {
  const cached = memo.get(view);
  if (cached) return cached;
  const out: ConstructionSite[] = [];
  for (const c of view.cities) {
    const item = c.detail?.producing;
    const ownK = c.detail?.production_cost ? clamp(c.detail.production_stored / c.detail.production_cost) : 0;
    const publicWork = c.construction;
    const wonder = item?.kind === 'Wonder' ? { building: item.building, tile: item.tile, k: ownK } : publicWork
      ? { building: publicWork.building, tile: publicWork.tile, k: clamp(publicWork.progress / 1000) } : null;
    if (wonder) {
      const tile = tileIndex(wonder.tile, view.width, view.height);
      if (tile >= 0) out.push({ city: c.id, owner: c.owner, tile, hex: hexAt(tile, view.width), building: wonder.building,
        era: Math.max(0, (view.players.find(p => p.id === c.owner)?.era ?? 1) - 1), k: wonder.k,
        stage: constructionStage(wonder.k), seed: c.id * 29 + tile, walls: false });
    } else if (c.walls_building || c.owner === view.me && item?.kind === 'Building' && item.building === 'Walls') {
      const tile = tileIndex(c.pos, view.width, view.height);
      if (tile >= 0) out.push({ city: c.id, owner: c.owner, tile, hex: c.pos, building: 'Walls',
        era: Math.max(0, (view.players.find(p => p.id === c.owner)?.era ?? 1) - 1), k: ownK,
        stage: constructionStage(ownK), seed: c.id * 29 + tile, walls: true });
    }
  }
  memo.set(view, out);
  return out;
}
