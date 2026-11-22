import type { BuyableTile, Hex } from '../types';

export interface TilePriceTag extends BuyableTile {
  affordable: boolean;
  /** "Need N more Gold" when the treasury cannot pay; otherwise null. The queued tile is always payable (it is already reserved). */
  reason: string | null;
  queued: boolean;
}

/** Presentation model only: eligibility and prices come from the city's view. `gold` is what is free to spend on this purchase. */
export function tilePriceTags(buyable: BuyableTile[], gold: number, queued: Hex | null): TilePriceTag[] {
  return buyable.map((tile) => {
    const isQueued = !!queued && queued.q === tile.tile.q && queued.r === tile.tile.r;
    const affordable = tile.price <= gold || isQueued;
    return { ...tile, affordable, reason: affordable ? null : `Need ${tile.price - gold} more Gold`, queued: isQueued };
  });
}
