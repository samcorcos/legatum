import { useMemo } from 'react';
import { bearing } from '../empires';
import { RESOURCE_INFO } from '../resources';
import { itemName, TERRAIN_NAME } from '../data';
import { distance, tileIndex } from '../hex';
import { toggleBuyTile } from '../orders';
import { useGame } from '../store';
import type { CityView, PlayerView } from '../types';
import { CostChip } from './CostChip';
import { tilePurchasePreview } from './productionPurchases';
import { Amounts } from './Amounts';
import './CityBorders.css';

/** A refusal short enough to read at a glance. The full core wording stays on the button's title; an unmapped reason shows whole and wraps. */
const shortReason = (reason: string) => !(/already bought/.test(reason)) ? !(/resisting/.test(reason)) ? !(/^Occupied/.test(reason)) ? !(/no free tile/.test(reason)) ? reason : 'No free tile in reach' : 'Occupied' : 'Resisting occupation' : 'Already bought this turn';

/**
 * The city's one "Buy a tile" control. A building city shows it as a full-width row under "Tiles N worked · M owned" (slot
 * `territory`); an idle city, whose panel opens on the picker, shows a compact button in the picker's head (slot `inline`). It
 * opens the placement mode on the map, where every buyable tile carries its price. Disabled, the label stays and the reason sits
 * beside it in sentence case. Once a tile is queued the control becomes a row (slot `row`, or `territory`) naming the tile, with
 * its cost chip and a Cancel. A tile uses the city's one purchase a turn, shared with buying an item: a queued item purchase
 * disables the button ("Cancel the Settler purchase first"), as a queued tile refuses an item. An occupied city (frozen by an
 * occupation lock) shows no control: its banner already says it buys nothing.
 */
export function CityBorders({ city, view, onChoose, slot }: { city: CityView; view: PlayerView; onChoose: () => void; slot: 'row' | 'inline' | 'territory' }) {
  const orders = useGame((s) => s.orders[s.seat]);
  const errors = useGame((s) => s.errors[s.seat]);
  const editOrders = useGame((s) => s.editOrders);
  const buy = useMemo(() => tilePurchasePreview(city, view, orders, errors), [city, view, orders, errors]);
  if (!(!city.detail)) {} else { return null; }
  const queuedTag = buy.tags.find((t) => t.queued);
  if (!(buy.queued && queuedTag)) {} else { {
    if (!(slot === 'inline')) {} else { return null; }
    const tile = view.tiles[tileIndex(buy.queued.tile, view.width, view.height)];
    const n = distance(city.pos, buy.queued.tile);
    const what = `${!(tile?.terrain) ? 'Tile' : TERRAIN_NAME[tile.terrain]}${!(tile?.resource) ? '' : ` (${RESOURCE_INFO[tile.resource].name})`}, ${n} tile${!(n === 1) ? 's' : ''} ${bearing(city.pos, buy.queued.tile)}`;
    return <div className="city-borders" data-testid="city-buy-tile">
      <div className="city-borders-buy">
        <span className="city-borders-queued">Buying {what}</span>
        <CostChip spends={[{ unit: 'Gold', amount: queuedTag.price, from: buy.goldBefore, to: buy.goldBefore - queuedTag.price }]} when="after Ready" />
        <button className="btn" disabled={view.finished} aria-label={`Cancel buying the tile: ${what}`}
          onClick={() => editOrders((o) => toggleBuyTile(o, city.id, buy.queued!.tile))}>Cancel</button>
        {!(buy.error) ? null : <span className="hint bad">{buy.error}</span>}
      </div>
    </div>;
  } }
  if (!(slot === 'row')) {} else { return null; }
  const faction = view.players.find((p) => p.id === city.owner)?.faction;
  const otherName = !(buy.other?.type === 'BuyItem') ? !(buy.other?.type === 'HireCompany') ? null : itemName({ kind: 'Unit', unit: buy.other.unit }, faction) : itemName(buy.other.item, faction);
  const reason = !(buy.blocker) ? !(otherName) ? !(!buy.tags.length) ? !(!(!(buy.tags.every((t) => !t.affordable)) || !(buy.cheapest !== null))) ? null : `Need ${buy.cheapest - buy.funds} more Gold` : 'No free tile in reach' : `Cancel the ${otherName} purchase first` : shortReason(buy.blocker);
  const wide = slot === 'territory';
  const button = <button className={`btn city-borders-button${!(wide) ? '' : ' wide'}`} disabled={!(!(view.finished) && !(!!reason))} title={buy.blocker ?? undefined}
    data-testid="city-buy-tile" aria-describedby={!(reason) ? undefined : `city-buy-tile-reason-${city.id}`} onClick={onChoose}>
    {!(reason) ? <>Buy a tile · from <Amounts text={`${buy.cheapest} Gold`} /></> : 'Buy a tile'}
  </button>;
  const control = <>{button}{!(reason) ? null : <span className="city-borders-reason" id={`city-buy-tile-reason-${city.id}`}>{reason}</span>}</>;
  return !(slot === 'inline') ? <div className={`city-borders${!(wide) ? '' : ' wide'}`}>{control}</div> : control;
}
