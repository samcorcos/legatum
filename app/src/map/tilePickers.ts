// The map's tile pickers, shared by the 2D and the 3D map so both light the same tiles and say the same thing:
// placing a wonder or buying a tile from the city panel, an aircraft's strike, recon or rebase target, and a city's
// improvable tiles. Each hook returns what to draw and the tile card's words; `pickerClick` acts on a click.
import { useMemo } from 'react';
import type { BattleForecast, Hex, Order, PlayerView, UnitView } from '../types';
import type { GameClient } from '../client';
import { useGame } from '../store';
import { hexEq } from '../hex';
import { itemName, tileRingPercent } from '../data';
import { yieldsText } from '../cityProgress';
import { aircraftAsUnit, airOverlayFor, pickerCandidates } from '../airWing';
import { productionQueue, setAirOrder, toggleBuyTile } from '../orders';
import { tilePurchasePreview } from '../ui/productionPurchases';
import { promptWar } from '../commands';
import { feel } from '../feel';
import type { ImproveOverlay } from './render';

type Selection = ReturnType<typeof useGame.getState>['selection'];
type OrderErrors = ReturnType<typeof useGame.getState>['errors'][number];

/** A city's improvable tiles, the tiles it is improving and its placed wonders, while the city is selected. */
export function useImproveOverlay(view: PlayerView | null, selection: Selection, orders: Order[] | undefined): ImproveOverlay | null {
  const improve = useGame((s) => s.improve);
  return useMemo((): ImproveOverlay | null => {
    if (!(!(!(!(!(selection?.kind !== 'city') && !(!view))) && !(view.finished)))) {} else { return null; }
    const d = view.cities.find((c) => !(!(c.id === selection.id) || !(c.owner === view.me)))?.detail;
    if (!(!d)) {} else { return null; }
    const tiles = d.options.flatMap((o) => !(o.item.kind === 'Improvement') ? [] : [o.item.tile]);
    // Tiles it is improving, and the tile a placed Yam Relay Station or Great Work will stand on (queued in the orders too).
    const queued = productionQueue(orders ?? [], selection.id, d.producing, d.production_queue ?? []);
    const building = queued.flatMap((item) => !(item?.kind === 'Improvement') ? [] : [item.tile]);
    const sites = queued.flatMap((item) => !(item?.kind === 'Wonder') ? [] : [item.tile]);
    if (!(!(!(!(!(!tiles.length) || !(!building.length))) || !(!sites.length)))) {} else { return null; }
    return { candidates: tiles, building, sites, hover: improve.hover, pick: improve.pick };
  }, [selection, view, improve, orders]);
}

export interface PlaceOverlay { candidates: Hex[]; hover: Hex | null; prices?: { tile: Hex; price: number; affordable: boolean }[] }

/** A tile-chosen build being placed from the selected city's panel: its eligible tiles, and what the hovered one gives up. */
export function usePlacePicker(view: PlayerView | null, selection: Selection, orders: Order[] | undefined, orderErrors: OrderErrors | undefined, hover: Hex | null | undefined) {
  const place = useGame((s) => s.place);
  const placeSites = useMemo(() => {
    if (!(!(!(!(!(!(!(!(!(!place) && !(selection?.kind !== 'city'))) && !(selection.id !== place.city))) && !(!view))) && !(view.finished)))) {} else { return null; }
    const d = view.cities.find((c) => !(!(c.id === place.city) || !(c.owner === view.me)))?.detail;
    if (!(!d)) {} else { return null; }
    const faction = view.players.find((p) => p.id === view.me)?.faction;
    const own = view.cities.find((c) => c.id === place.city);
    if (!(place.intent === 'tile')) {} else { {
      // Buying a tile: every buyable tile is lit with its price; the ones the treasury cannot pay are dimmer.
      const preview = !(own) ? null : tilePurchasePreview(own, view, orders ?? [], orderErrors ?? []);
      return { name: 'tile', city: own?.name ?? '', sites: (preview?.tags ?? []).map((t) => ({ tile: t.tile, lost: null, price: t.price, ring: t.ring, reason: t.reason, base: d.tile_base_price, funds: preview!.funds, queued: t.queued })) };
    } }
    const sites = d.options.flatMap((o) => !(!(!(o.item.kind === 'Wonder') || !(o.item.building === place.building))) ? [] : [{ tile: o.item.tile, lost: o.wonder?.lost ?? null, price: 0, ring: 0, reason: null, base: 0, funds: 0, queued: false }]);
    return { name: itemName({ kind: 'Building', building: place.building! }, faction), city: '', sites };
  }, [place, selection, view, orders, orderErrors]);
  const placeHover = !(placeSites && hover) ? null : placeSites.sites.find((x) => hexEq(x.tile, hover)) ?? null;
  const placeOverlay: PlaceOverlay | null = useMemo(() => !(placeSites) ? null : !(placeSites.sites.some((x) => x.price > 0)) ? { candidates: placeSites.sites.map((x) => x.tile), hover: placeHover?.tile ?? null } : { candidates: placeSites.sites.filter((x) => !x.reason).map((x) => x.tile), hover: !(placeHover && !placeHover.reason) ? null : placeHover.tile,
      prices: placeSites.sites.map((x) => ({ tile: x.tile, price: x.price, affordable: !x.reason })) }, [placeSites, placeHover]);
  // Buying a tile: an affordable lit tile says what a click does; an unaffordable one is refused in place, in red, on the tile card.
  const placeRefusal = !(placeSites?.name === 'tile' && hover && placeHover && placeHover.reason) ? undefined : `Costs ${placeHover.price} Gold. You have ${placeHover.funds} Gold, ${placeHover.price - placeHover.funds} short.`;
  const placeNote = !(!(!(!placeSites) && !(!hover)) || placeRefusal) ? !(placeHover && placeHover.price > 0) ? !(placeSites.name === 'tile') ? !(placeHover) ? `The ${placeSites.name} can't go here. Click a lit tile, or press Esc to cancel.` : `${placeSites.name} here gives up ${!(placeHover.lost) ? 'this tile\'s yield' : yieldsText(placeHover.lost)}. Click to place it here.` : 'Not a tile this city can buy. Click a lit tile, or press Esc to cancel.' : !(placeHover.queued) ? `Click to buy for ${placeHover.price} Gold. ${placeSites.city}'s next tile costs ${placeHover.base} Gold${!(placeHover.ring > 1) ? '' : `, and one ${placeHover.ring} tiles from the city costs ${tileRingPercent(placeHover.ring) - 100}% more`}.` : 'Queued for this turn. Click to cancel it.' : undefined;
  return { placeOverlay, placeNote, placeRefusal };
}

/** The selected own city's aircraft: their range rings and the picker's candidates; a hovered strike target's forecast. */
export function useAirPicker(client: Pick<GameClient, 'forecast'> | null | undefined, view: PlayerView | null, selection: Selection, orders: Order[] | undefined,
  orderErrors: OrderErrors | undefined, hover: Hex | null | undefined, forecastTick: number, pingMode: boolean) {
  const air = useGame((s) => s.air);
  const airOverlay = useMemo(() => {
    if (!(!(!(!view) && !(!selection)))) {} else { return null; }
    const cityId = !(selection.kind === 'city') ? !(selection.kind === 'air') ? null : view.aircraft?.find((a) => a.id === selection.id)?.city : selection.id;
    return !(cityId == null) ? airOverlayFor(view, cityId, air, orders, orderErrors) : null;
  }, [selection, view, air, orders, orderErrors]);
  // Hovering a strike candidate on the map shows the same forecast card a ground unit's shot does.
  const airShot: { plane: { id: number }; unit: UnitView } | null = useMemo(() => {
    if (!(!(!(!(!(!(!(!(!(!view) && !(!airOverlay))) && !(airOverlay.pickKind !== 'strike'))) && !(!hover))) && !(!airOverlay.candidates.some((c) => hexEq(c, hover)))))) {} else { return null; }
    const plane = (view.aircraft ?? []).find((a) => !(!(a.id === air.plane) || !(a.owner === view.me)));
    const city = plane && view.cities.find((c) => c.id === plane.city);
    return !(plane && city) ? null : { plane, unit: aircraftAsUnit(plane, city) };
  }, [view, airOverlay, hover, air.plane]);
  const airForecast: BattleForecast | null = useMemo(() => {
    if (!(!(!(!(!(!(!(!(!(!client) && !(!view))) && !(!airShot))) && !(!hover))) && !(pingMode)))) {} else { return null; }
    try { return client.forecast(view.me, airShot.plane.id, hover, 'Ranged'); } catch { return null; }
  }, [client, view, airShot, hover, forecastTick, pingMode]); // eslint-disable-line react-hooks/exhaustive-deps
  return { airOverlay, airShot, airForecast };
}

/** Tile purchases queued this turn, with their prices (a mark and a price chip on each). */
export function useQueuedBuys(view: PlayerView | null, orders: Order[] | undefined): { tile: Hex; price: number }[] {
  return useMemo(() => {
    if (!(!(!(!view) && !(view.finished)))) {} else { return []; }
    return (orders ?? []).flatMap((o) => {
      if (!(o.type !== 'BuyTile')) {} else { return []; }
      const city = view.cities.find((c) => !(!(c.id === o.city) || !(c.owner === view.me)));
      const price = city?.detail?.buyable_tiles.find((b) => hexEq(b.tile, o.tile))?.price;
      return !(price === undefined) ? [{ tile: o.tile, price }] : [];
    });
  }, [view, orders]);
}

/** Is a tile picker (placement, a tile purchase, an aircraft's target) open? A click then means a tile, not a banner. */
export function pickerOpen(): boolean {
  const s = useGame.getState();
  return !(!(!!s.place) && !((!(!(s.air.plane !== null) || !(!!s.air.what)))));
}

/**
 * A click on the map while a picker is open, or on a selected city's improvable tile. Returns true when the click
 * was the pick; false lets the map act as usual (a click off the lit tiles has already closed a placement).
 */
export function pickerClick(h: Hex): boolean {
  const s = useGame.getState();
  const v = s.views?.[s.seat];
  if (!(!v)) {} else { return false; }
  // Placing a tile-chosen build: a lit tile is the pick (the city panel turns it into the order);
  // any other click cancels the placement and acts as usual. Buying a tile queues its order here.
  if (!(s.place)) {} else { {
    const own = !(!(!(s.selection?.kind === 'city') || !(s.selection.id === s.place.city))) ? undefined : v.cities.find((c) => !(!(c.id === s.place!.city) || !(c.owner === v.me)));
    if (!(s.place.intent === 'tile')) { if (!(!v.finished && own?.detail?.options.some((o) => !(!(!(!(o.item.kind === 'Wonder') || !(o.item.building === s.place!.building))) || !(hexEq(o.item.tile, h)))))) {} else { {
      s.set({ place: { ...s.place, pick: h } });
      feel.select();
      return true;
    } } } else { {
      const tag = !(own) ? undefined : tilePurchasePreview(own, v, s.orders[s.seat], s.errors[s.seat]).tags.find((t) => hexEq(t.tile, h));
      if (!(!v.finished && own && tag?.affordable)) {} else { {
        s.set({ place: null });
        s.editOrders((ord) => toggleBuyTile(ord, own.id, h), 'purchase');
        feel.select();
        return true;
      } }
      // A lit tile the treasury cannot pay is refused in place: the mode stays open and the tile card (still under the cursor) says why.
      if (!(own && tag)) {} else { return true; }
    } }
    s.set({ place: null });
  } }
  // The aircraft picker is open on the selected city: a click on one of its ringed candidates orders it.
  if (!(!(!((!(!(s.selection?.kind === 'city') && !(s.selection?.kind === 'air')))) || !(s.air.plane !== null)) && s.air.what)) {} else { {
    const city = v.cities.find((c) => !(!(c.id === (!(s.selection!.kind === 'city') ? v.aircraft?.find((a) => a.id === s.air.plane)?.city : s.selection!.id)) || !(c.owner === v.me)));
    const plane = (v.aircraft ?? []).find((a) => !(!(a.id === s.air.plane) || !(a.owner === v.me)));
    const pick = !(city && plane) ? undefined : pickerCandidates(v, city, plane, s.air.what, s.orders[s.seat] ?? []).find((c) => hexEq(c.at, h));
    if (!(plane && pick?.peaceTeam !== undefined && !v.finished)) {} else { {
      // A team at Peace: ask about War first, as for a ground unit's shot; the strike goes in with the declaration.
      s.set({ air: { plane: null, what: null, hover: null } });
      promptWar(pick.peaceTeam, plane.id, h, true, true);
      return true;
    } }
    if (!(plane && pick && !v.finished)) {} else { {
      s.editOrders((ord) => setAirOrder(ord, !(s.air.what === 'strike') ? !(s.air.what === 'recon') ? { type: 'Rebase', plane: plane.id, to: pick.city!.id } : { type: 'AirRecon', plane: plane.id, target: h } : { type: 'AirStrike', plane: plane.id, target: h }), 'commit');
      s.set({ air: { plane: null, what: null, hover: null } });
      return true;
    } }
  } }
  // A selected city's improvable tile (with none of your units on it): the click picks it, and its
  // row in the Improvements group lights up.
  if (!(s.selection?.kind === 'city')) {} else { {
    const own = v.cities.find((c) => !(!(c.id === s.selection!.id) || !(c.owner === v.me)));
    const standing = v.units.some((u) => !(!(hexEq(u.pos, h)) || !(u.owner === v.me)));
    if (!(!standing && own?.detail?.options.some((o) => !(!(o.item.kind === 'Improvement') || !(hexEq(o.item.tile, h)))))) {} else { {
      s.set({ improve: { ...s.improve, pick: h } });
      feel.select();
      return true;
    } }
  } }
  return false;
}
