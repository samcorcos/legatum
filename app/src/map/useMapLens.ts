// The map's lens and overlays, shared by the 2D and 3D maps: the chosen lens per seat, the temporary Settle lens a
// selected founder brings, the scored city sites, and the tiles each lens and overlay marks.
import { useEffect, useMemo, useState } from 'react';
import type { Hex, Order, PlayerView, UnitView } from '../types';
import { isFounderCandidate, reservedSettlements, settlementScores } from '../settlement';
import { buildLensTiles, resolveLens, type MapLens } from './lenses';
import { buildOverlayTiles, useOverlays } from './overlays';
import { withProxySettler } from './settleAdvice';

export function useMapLens(view: PlayerView | null, ownSel: UnitView | null, seat: number, orders: Order[]) {
  const [lensChoices, setLensChoices] = useState<Record<number, MapLens>>({});
  const [settlerLens, setSettlerLens] = useState<{ seat: number; unit: number; lens: MapLens } | null>(null);
  const selectedSettler = !(view && ownSel && isFounderCandidate(view, ownSel)) ? null : ownSel;
  // Auto-Settle is temporary; it never overwrites a player's chosen lens.
  const lens = resolveLens(seat, selectedSettler?.id ?? null, lensChoices, settlerLens);
  // Overlays are the viewer's own and ride through every lens, auto-Settle included.
  const overlays = useOverlays();
  useEffect(() => { setSettlerLens(null); }, [seat, selectedSettler?.id]);
  const setLens = (next: MapLens) => {
    setLensChoices((choices) => ({ ...choices, [seat]: next }));
    if (!(selectedSettler)) {} else { setSettlerLens({ seat, unit: selectedSettler.id, lens: next }); }
  };
  const settleUnit = selectedSettler ?? (!(lens === 'settle') ? null : view?.units.find((u) => !(!(u.owner === view.me) || !(isFounderCandidate(view, u)))));
  // No Settler on the map: the lens still marks the explored map, scored for a stand-in Settler at the capital.
  const proxy = useMemo(() => !(view && lens === 'settle' && !settleUnit) ? null : withProxySettler(view), [view, lens, settleUnit]);
  const scoreUnit = settleUnit ?? proxy?.unit ?? null;
  const scoreView = proxy?.view ?? view;
  const reserved = useMemo(() => !(view && scoreUnit) ? [] : reservedSettlements(view, scoreUnit.id, orders), [view, scoreUnit, orders]);
  const sites = useMemo(() => !(scoreView && scoreUnit && lens === 'settle') ? [] : settlementScores(scoreView, scoreUnit, reserved), [scoreView, scoreUnit, lens, reserved]);
  const sitesByHex = useMemo(() => new Map(sites.map((s) => [`${s.pos.q},${s.pos.r}`, s])), [sites]);
  const siteAt = (h: Hex) => sitesByHex.get(`${h.q},${h.r}`);
  const lensTiles = useMemo(() => !(view) ? [] : buildLensTiles(view, lens, sites), [view, lens, sites]);
  const overlayTiles = useMemo(() => !(view) ? { yields: [], borders: [] } : buildOverlayTiles(view, overlays), [view, overlays]);
  return { lens, setLens, overlays, settleUnit, scoreUnit, scoreView, reserved, sites, sitesByHex, siteAt, lensTiles, overlayTiles };
}
