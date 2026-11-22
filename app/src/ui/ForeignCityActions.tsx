import { useGame } from '../store';
import type { CityView, Order, PlayerView, UnitId } from '../types';
import { isBarbarian } from '../barbarians';
import { relationWith } from '../diplomacy';
import { empireTitle } from '../empires';
import { attacksInMelee, itemName } from '../data';
import { distance } from '../hex';
import { moveSelected, promptWar, rangedSelected, warDeclaration } from '../commands';
import { feel } from '../feel';
import { cityFireStatus } from '../format';

/** Where you stand with a rival city's empire and whether the selected unit can attack it; null for other cities. */
export function foreignCityChoice(view: PlayerView, city: CityView, attackerId: UnitId | null, orders: Order[]) {
  const owner = view.players.find((p) => p.id === city.owner);
  if (!owner || owner.team === view.team || isBarbarian(city.owner) || view.finished) return null;
  const team = owner.team;
  const relation = relationWith(view, team);
  const state = view.diplomacy?.legacy ? 'War' : relation?.state ?? 'Peace';
  const atWar = state === 'War';
  const war = warDeclaration(view, team);
  const declared = !atWar && 'action' in war && orders.some((o) => o.type === 'DeclareWar' && o.against === team);
  const unit = attackerId === null ? undefined : view.units.find((u) => u.id === attackerId && u.owner === view.me && u.military);
  // A Keshig shoots or charges; the card's attack is the charge (it can take the city), the shot a second button.
  const bow = !!unit && unit.ranged > 0 && attacksInMelee(unit.kind);
  const ranged = !!unit && unit.ranged > 0 && !bow;
  const tiles = unit ? distance(unit.pos, city.pos) : 0;
  const reach = ranged ? unit!.range : Math.max(1, unit?.max_moves ?? 0);
  const inRange = !!unit && tiles <= reach;
  const canShoot = bow && tiles <= unit!.range && (atWar || declared || 'action' in war);
  return { team, relation, state, atWar, war, declared, unit, ranged, bow, canShoot, tiles, reach, inRange,
    /** An attack at hand: the card's primary action, so the turn's own call to action steps back. */
    canAttack: inRange && (atWar || declared || 'action' in war) };
}

/**
 * A foreign city's card actions (playtest 6): where you stand with its empire, and what you can do
 * about it from here: attack it with the unit you had selected, declare War (the same confirmation
 * as an attack), or open diplomacy with that empire. Only an attack at hand is a primary.
 */
export function ForeignCityActions({ city, view, compact = false }: { city: CityView; view: PlayerView; compact?: boolean }) {
  const attackerId = useGame((s) => s.attacker);
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const fire = city.visible ? cityFireStatus(view, city) : null;
  const choice = foreignCityChoice(view, city, attackerId, orders);
  if (!choice) return null;
  const { team, relation, state, atWar, war, declared, unit, ranged, bow, canShoot, tiles, reach, inRange, canAttack } = choice;
  const name = empireTitle(view, team);
  const starts = 'action' in war && war.action.effective_turn === view.turn ? 'this turn' : 'next turn';
  const unitName = unit ? itemName({ kind: 'Unit', unit: unit.kind }, view.players.find((p) => p.id === view.me)?.faction) : '';
  const attack = () => {
    if (!unit) return;
    const s = useGame.getState();
    s.select({ kind: 'unit', id: unit.id });
    feel.select();
    // The same pipeline as ordering it on the map: at Peace it asks to declare War first.
    if (ranged) rangedSelected(city.pos);
    else moveSelected(city.pos);
  };
  const diplomacy = () => useGame.getState().set({ diplomacyOpen: true, diplomacyTeam: team, reportOpen: false, techOpen: false });
  const standing = atWar ? `At War with ${name}${relation && relation.effective_turn ? ` since turn ${relation.effective_turn}` : ''}.`
    : declared ? `War declared on ${name} this turn; it starts ${starts}.`
    : state === 'Peace' ? `At Peace with ${name}: no attacks or entering their land.`
    : `Allied with ${name}: declaring War on them is a betrayal.`;
  const attackLabel = `${bow ? 'Charge' : 'Attack'} with ${unitName}${atWar || declared ? '' : '…'}`;
  const shootLabel = `Shoot with ${unitName}${atWar || declared ? '' : '…'}`;
  const shoot = () => {
    if (!unit) return;
    useGame.getState().select({ kind: 'unit', id: unit.id });
    feel.select();
    rangedSelected(city.pos);
  };
  const badge = <span className={`empire-relation ${(declared ? 'war' : state).toLowerCase()}`}>{declared ? 'War declared' : state}</span>;
  // The phone sheet's peek: one line of facts and one row of buttons; More ↑ shows the full card.
  if (compact) return <div className="foreign-city compact" data-testid="foreign-city-actions">
    <p className="foreign-city-relation">{badge} {name}{city.visible ? <> · Pop {city.population} · Strength {city.strength}{fire && <> · <span data-testid="city-fire">{fire.fires ? 'Fires: has Walls' : fire.text.replace('Cannot fire: ', '')}</span></>}</> : ' · out of sight'}</p>
    {city.greece_captive_tech && <p className="foreign-city-prize">Capture it: +1 free technology</p>}
    <div className="foreign-city-buttons">
      {canAttack && <button className="btn primary" onClick={attack}>{attackLabel}</button>}
      {canShoot && <button className="btn" onClick={shoot}>{shootLabel}</button>}
      {!canAttack && !atWar && !declared && 'action' in war && <button className="btn" onClick={() => promptWar(team, null, city.pos)}>Declare War…</button>}
      <button className="btn" aria-label={`Diplomacy with ${name}`} onClick={diplomacy}>Diplomacy</button>
    </div>
  </div>;
  return <div className="foreign-city" data-testid="foreign-city-actions">
    <p className="foreign-city-relation">{badge} {standing}</p>
    {city.greece_captive_tech && <p className="foreign-city-prize">Capture it: +1 free technology</p>}
    {unit && !inRange && !compact && <p className="muted foreign-city-why">Your {unitName} is {tiles} tiles away ({ranged ? `range ${reach}` : `${reach} moves`}): move it closer to attack.</p>}
    <div className="foreign-city-buttons">
      {canAttack && <button className="btn primary" onClick={attack}>{attackLabel}</button>}
      {canShoot && <button className="btn" onClick={shoot}>{shootLabel}</button>}
      {/* Pillar 9: the turn's own call to action stays the primary unless an attack is at hand. */}
      <button className="btn" onClick={diplomacy}>Diplomacy with {name}</button>
      {!atWar && !declared && 'action' in war && <button className="btn" onClick={() => promptWar(team, null, city.pos)}>Declare War…</button>}
    </div>
    {!atWar && !declared && !('action' in war) && !compact && <p className="muted">Not now: Declare War. {war.blocked}</p>}
  </div>;
}
