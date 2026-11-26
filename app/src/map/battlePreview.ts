// The battle forecast for the selected unit at a target tile, shared by the 2D and the 3D map so both show the
// same cards: the forecast, a city assault's pooled fight, and a mounted archer's shot beside its charge.
import { useMemo } from 'react';
import type { AlongsideAttack, BattleForecast, CityAssaultForecast, Hex, Order, PlayerView, UnitView } from '../types';
import type { GameClient } from '../client';
import { distance, hexEq } from '../hex';
import { attacksInMelee } from '../data';
import { enemyAt, rangedEnemyAt } from '../commands';
import { shotBlockReason } from '../los';
import { atWar } from '../diplomacy';

export interface BattlePreview {
  /** Why a ranged shot at the target is blocked (line of sight), in ranged mode. */
  blockedShot: string | null;
  forecast: BattleForecast | null;
  assault: CityAssaultForecast | null;
  /** A mounted archer out of ranged mode: why the shot is not on offer beside its charge. */
  shotWhy: string | null;
  shotForecast: BattleForecast | null;
}

export function useBattlePreview(
  client: Pick<GameClient, 'forecast' | 'assault'> | null | undefined,
  view: PlayerView | null,
  ownSel: UnitView | null,
  target: Hex | null | undefined,
  mode: string,
  orders: readonly Order[] | undefined,
  forecastTick: number,
): BattlePreview {
  const blockedShot = mode === 'ranged' && ownSel && view && target
    && distance(ownSel.pos, target) <= ownSel.range
    ? shotBlockReason(view, ownSel.pos, target, ownSel.kind) : null;
  const forecast: BattleForecast | null = useMemo(() => {
    if (!client || !view || !ownSel || !ownSel.military || !target) return null;
    if (!enemyAt(view, target)) return null;
    if (mode === 'ranged' && (distance(ownSel.pos, target) > ownSel.range || blockedShot)) return null;
    try {
      return client.forecast(view.me, ownSel.id, target, mode === 'ranged' ? 'Ranged' : 'Melee');
    } catch {
      return null;
    }
  }, [client, view, ownSel, target, mode, forecastTick, blockedShot]); // eslint-disable-line react-hooks/exhaustive-deps

  // The whole fight at an enemy city: core pools this attack with every other attack of yours queued on the same tile this turn.
  const assault: CityAssaultForecast | null = useMemo(() => {
    if (!client?.assault || !view || !ownSel || !forecast || !target) return null;
    if (!view.cities.some((c) => hexEq(c.pos, target) && atWar(view, c.owner))) return null;
    const alongside: AlongsideAttack[] = (orders ?? []).flatMap((o): AlongsideAttack[] => {
      if (o.type === 'RangedAttack') return o.unit !== ownSel.id && hexEq(o.target, target) ? [{ unit: o.unit, mode: 'Ranged' }] : [];
      if (o.type === 'MoveUnit' || o.type === 'MoveUnitPath') return o.unit !== ownSel.id && hexEq(o.to, target) ? [{ unit: o.unit, mode: 'Melee' }] : [];
      return [];
    });
    try {
      return client.assault(view.me, ownSel.id, target, forecast.ranged ? 'Ranged' : 'Melee', alongside);
    } catch {
      return null;
    }
  }, [client, view, ownSel, forecast, target, orders, forecastTick]); // eslint-disable-line react-hooks/exhaustive-deps

  // A Keshig (a mounted archer) shoots or charges. Out of ranged mode, a target it could do both to shows both
  // forecasts on one card; when the shot is not on offer the card says why, next to the charge.
  const bow = !!ownSel && ownSel.military && ownSel.ranged > 0 && attacksInMelee(ownSel.kind);
  const shotWhy: string | null = (() => {
    if (!bow || mode === 'ranged' || !ownSel || !view || !target || !forecast || forecast.ranged) return null;
    if (distance(ownSel.pos, target) > ownSel.range) {
      return `No shot from here: this unit shoots ${ownSel.range === 1 ? 'only at a target next to it' : `up to ${ownSel.range} tiles`}, and the target is ${distance(ownSel.pos, target)} tiles away. Right-click to charge it.`;
    }
    if (!rangedEnemyAt(view, target)) return 'A shot hits only military units and cities. Right-click to ride onto it.';
    return shotBlockReason(view, ownSel.pos, target, ownSel.kind);
  })();
  const shotForecast: BattleForecast | null = useMemo(() => {
    if (!bow || mode === 'ranged' || shotWhy || !forecast || forecast.ranged || !client || !view || !ownSel || !target) return null;
    try {
      return client.forecast(view.me, ownSel.id, target, 'Ranged');
    } catch {
      return null;
    }
  }, [client, view, ownSel, target, mode, forecastTick, bow, shotWhy, forecast]); // eslint-disable-line react-hooks/exhaustive-deps

  return { blockedShot, forecast, assault, shotWhy, shotForecast };
}
