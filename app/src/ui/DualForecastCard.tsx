// The forecast for a unit that can do two things to the same enemy: the Mongol Keshig, a mounted archer, either
// shoots from where it stands or charges onto the target (one order a turn, no shoot-then-move). Both outcomes sit
// on one card, side by side in reading order, each with the key that orders it, so the player picks and nothing is
// chosen silently (legatum-ui: pillar 5, no hidden math; pillar 9, the next action is obvious).
import { isCalmRouteNote } from '../path';
import { Amounts } from './Amounts';
import type { BattleForecast, Hex, PlayerView, UnitKind, UnitView } from '../types';
import { itemName } from '../data';
import { useGame } from '../store';
import { formatBreakdown } from '../format';
import { forecastEraHeadline, outclasses } from '../map/unitEra';
import { AftermathLine, FortifyLine, forecastModel, tradeMark, winAction } from './ForecastCard';
import { cityEntryWarning, forecastWinLabel, garrisonHeadline } from './forecastWarnings';
import { usePlacedCard, type CardAnchor } from './usePlacedCard';
import { Icon } from './bits';
import { RankMark } from './RankMark';
import './dualForecast.css';

/**
 * The class lines in a fight, in words: "Mounted against ranged (+40%): +13 for your Keshig". A shot between a Keshig and a
 * spear line has none (the shot fights as ranged), which is itself worth saying.
 */
export function classLines(f: BattleForecast, mine: string, theirs: string): string[] {
  const lines: string[] = [];
  for (const [label, amount] of f.attacker_strength.parts) if (/ against /.test(label) && amount > 0) lines.push(`${label}: +${amount} for your ${mine}`);
  for (const [label, amount] of f.defender_strength.parts) if (/ against /.test(label) && amount > 0) lines.push(`${label}: +${amount} for the ${theirs}`);
  return lines;
}

type Model = ReturnType<typeof forecastModel>;

function Option({ kind, f, m, view, unit }: { kind: 'shot' | 'charge'; f: BattleForecast; m: Model; view: PlayerView; unit: UnitView }) {
  const shot = kind === 'shot';
  const classes = classLines(f, m.unitLabel, m.defName);
  const defHpAfter = Math.max(0, f.defender_hp - m.dealt);
  return (
    <section className={`dual-option ${shot ? 'shot' : 'charge'}`} data-testid={`forecast-${kind}`} data-verdict={m.verdict} aria-label={shot ? 'Shoot' : 'Charge'}>
      <header className="dual-head">
        <span className="dual-what"><Icon name={shot ? 'unit-archer' : 'unit-horseman'} size={15} /> {shot ? 'Shoot' : 'Charge'}</span>
        <span className="dual-key">{shot ? 'R, then click' : 'Right-click'}</span>
      </header>
      <p className={`dual-outcome ${m.tone}`} data-testid={`forecast-${kind}-headline`}><b>{m.headline.title}.</b> <span className={`forecast-outcome ${m.headline.tradeTone}`}>{tradeMark(m.headline.tradeTone)}{m.headline.detail}</span></p>
      {!shot && m.aftermath?.severe && <AftermathLine m={m} />}
      <p className="dual-line" data-testid={`forecast-${kind}-numbers`}>
        You <RankMark level={unit.level ?? 0} faction={view.players.find((p) => p.id === unit.owner)?.faction} compact /> <b>{f.attacker_strength.total}</b> vs {m.defName} <RankMark level={m.defUnit?.level ?? 0} faction={view.players.find((p) => p.id === m.defOwner)?.faction} compact /> <b>{f.defender_strength.total}</b> strength · your health <b>{f.attacker_hp} → {m.headline.kind === 'loss' ? 'lost' : m.hpAfter}</b> · {m.defName} <b>{f.defender_hp} → {defHpAfter}</b>
      </p>
      {(m.win > 0 || m.loss > 0) && <p className="dual-odds">
        <b className="good">{m.win}%</b> chance to {winAction(forecastWinLabel(f, !!m.city, shot, false), m.defName, m.city?.name ?? null)}
        {' · '}<b className={m.loss > 0 ? 'bad' : ''}>{m.loss}%</b> chance to lose your {m.unitLabel}
      </p>}
      <p className="dual-class" data-testid={`forecast-${kind}-class`}>
        {classes.length ? classes.join('; ') : 'No class bonus for either side.'}
      </p>
      {m.fire && m.city && <p className="dual-bad" role="alert">City fire: {m.city.name} {f.walls && f.walls.covered > 0 ? 'still ' : ''}fires first, about −{m.fire.fire} health to your {m.unitLabel}. Included in your unit’s health loss above.</p>}
      {!shot && m.risk && <p className="dual-bad" role="alert">{m.risk}</p>}
      {m.city && f.garrison && <p className="dual-line">{garrisonHeadline(f, view, m.city.name, shot)}</p>}
      {m.city && <p className={shot ? 'dual-line' : 'dual-line forecast-note'}>{cityEntryWarning(unit, shot)}</p>}
      {!shot && m.aftermath && !m.aftermath.severe && <AftermathLine m={m} />}
      {!shot && m.fortify && <FortifyLine m={m} />}
      <p className="dual-parts forecast-note">You {formatBreakdown(f.attacker_strength)}; them {formatBreakdown(f.defender_strength)}. Damage varies: each hit can be a fifth more or less.</p>
    </section>
  );
}

/** Shoot and charge, together. `shot` and `charge` are the core's two forecasts of the same target. */
export function DualForecastCard({ shot, charge, anchor, view, unit, target, routeWarning }: {
  shot: BattleForecast; charge: BattleForecast; anchor?: CardAnchor; view: PlayerView; unit: UnitView; target: Hex; routeWarning?: string;
}) {
  const orders = useGame((s) => s.orders[s.seat]);
  const placed = usePlacedCard<HTMLDivElement>(anchor);
  const shotM = forecastModel(shot, true, view, unit, target, orders);
  const chargeM = forecastModel(charge, false, view, unit, target, orders);
  const faction = view.players.find((p) => p.id === unit.owner)?.faction;
  const defFaction = view.players.find((p) => p.id === chargeM.defOwner)?.faction;
  const name = itemName({ kind: 'Unit', unit: unit.kind }, faction);
  return (
    <div className="forecast forecast-dual" ref={placed.ref} style={placed.style} data-testid="forecast-dual">
      <div className="forecast-head">
        <span className="forecast-kind">Shoot or charge</span>
        <span className="dual-once">One order a turn</span>
      </div>
      {chargeM.defUnit?.military && outclasses(chargeM.defUnit.kind as UnitKind, defFaction, unit.kind as UnitKind, faction, charge.defender_strength.total, charge.attacker_strength.total) && (
        <div className="forecast-era bad" data-testid="forecast-era">
          {forecastEraHeadline(chargeM.defUnit.kind as UnitKind, defFaction, unit.kind as UnitKind, charge.defender_strength.total, charge.attacker_strength.total)}
        </div>
      )}
      <Option kind="shot" f={shot} m={shotM} view={view} unit={unit} />
      <Option kind="charge" f={charge} m={chargeM} view={view} unit={unit} />
      {routeWarning && <div className={isCalmRouteNote(routeWarning) ? 'forecast-shield route-note' : 'forecast-risk bad'} role={isCalmRouteNote(routeWarning) ? undefined : 'alert'} data-testid="route-warning">{routeWarning}</div>}
      <div className="forecast-foot forecast-note">
        <Amounts text={`A ${name} either shoots or charges each turn, never both. A shot never moves it and cannot take a city.`} />
      </div>
    </div>
  );
}
