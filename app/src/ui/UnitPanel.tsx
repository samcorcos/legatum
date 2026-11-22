import { isBarbarian, ownerName } from '../barbarians';
import { atWar, barbariansSpareMe, canEnter } from '../diplomacy';
import { useLayoutEffect } from 'react';
import { useGame } from '../store';
import type { Faction, Order, PlayerView, UnitKind, UnitView } from '../types';
import { UNSUPPLIED_STRENGTH_PERCENT } from '../resources';
import { tileIndex } from '../hex';
import { roadBonusText } from '../path';
import { siegeCityBonus, siegeTier } from '../combat';
import { ARTILLERY_RULE, BUILT_IN_SIEGE_NOTE, builtInSiege, canAutoExplore, firesIndirect, CLASS_NAME, CLASS_NOTE, attacksInMelee, eraName, minCityGap, RULES, SIEGE_USES_IRON, UNIT_STATS, ignoresWounds, itemName, unitClass, unitEraTier } from '../data';
import { occupationLead, occupationNote } from './forecastWarnings';
import { describeOrder, errorsWhere, orderUnit, pendingUnitOrder, sleepOrderText } from '../orders';
import {
  toggleAutoExploreSelected,
  toggleMoveSelected,
  cancelSelected,
  disbandSelected,
  foundCitySelected,
  sleepSelected,
  healSelected,
  healBlocker,
  healForecast,
  skipSelected,
  toggleRangedMode,
  toggleAutoFireSelected,
} from '../commands';
import { healthColor, playerInk } from '../theme';
import { Bar, BreakdownTable, Icon, Tip } from './index';
import { Action, UnitBadge } from './UnitAction';
import { placeName } from '../empires';
import { isExploring } from '../automation';
import { PlayerBadge } from './PlayerBadge';
import { useScrollHint } from './useScrollHint';
import { useUnitCardLeads } from './turnLead';
import { MAX_LEVEL, STRENGTH_PERCENT, strengthBonus } from '../promotion';
import { attackableEnemies, unitEra } from '../map/unitEra';
import { EraTab } from './EraTab';
import { foundCityUnitBlocker, isFounderCandidate } from '../settlement';
import { UnitImprovements, upgradeCheck } from './UnitImprovements';
import { pendingSpend, goldOnHand } from './productionPurchases';
export { upgradeCheck, upgradeStepBreakdown } from './UnitImprovements';
import { RingMedallion } from './ringMedallion';
import './unitPanel.css';
import { Amounts } from './Amounts';
import { HeirDisbandConfirm, HeirNote } from './HeirNote';
import { CompanyNote, CompanyTag } from './CompanyNote';
import { isCompany } from '../company';
import { FACTION_GOAL, factionGoalVp } from '../data';
import { clearRetire, pendingRetire, retireEntry } from '../goalActions';
import { isTopRank } from '../rankInsignia';
import { RankMark } from './RankMark';

export const siegeCityHint = (bonus: number) => `+${bonus} ranged strength against city HP. Hits land one at a time. The hit that empties the city's HP carries on to the garrison with the part of its damage the city did not absorb. Later hits strike the garrison in full. Ranged fire cannot capture.`;

export function UnitPanel({ unit, view, peek = false }: { unit: UnitView; view: PlayerView; peek?: boolean }) {
  const { scroll, overflow } = useScrollHint();
  useLayoutEffect(() => { if (scroll.current) scroll.current.scrollTop = 0; }, [unit.id, scroll]);
  const orders = useGame((s) => s.orders[s.seat]);
  const leads = useUnitCardLeads();
  const errors = useGame((s) => s.errors[s.seat]);
  const mode = useGame((s) => s.mode);
  const setSeat = useGame((s) => s.setSeat);
  const ownerSeat = useGame((s) => (s.client?.kind === 'remote' ? -1 : s.humans.indexOf(unit.owner)));
  const mine = unit.owner === view.me;
  // The game is decided: the map stays open to look around, but no unit awaits orders and none can be given.
  const over = view.finished || view.winner !== null;
  // The map's strength chips need a key while they show (canvas marks have no hover).
  const strengthTargets = mine && !over && unit.military ? attackableEnemies(view, unit) : [];
  const strengthKey = strengthTargets.length > 0;
  const barbarianStrengthKey = barbariansSpareMe(view) && strengthTargets.some((u) => isBarbarian(u.owner));
  const onlyBarbarianStrength = barbarianStrengthKey && strengthTargets.every((u) => isBarbarian(u.owner));
  const closed = mine && !canEnter(view, unit.kind, view.tiles[tileIndex(unit.pos, view.width, view.height)]?.owner ?? null);
  const owner = view.players.find((p) => p.id === unit.owner);
  // Standing on a Yam Relay Station holder's land at Peace: the road is open now and closes if the Station changes hands.
  const roadOwner = mine ? view.tiles[tileIndex(unit.pos, view.width, view.height)]?.owner ?? null : null;
  const onRoad = roadOwner !== null && view.players.find((p) => p.id === roadOwner)?.team !== view.team && !!view.courier_halls?.some((h) => h.holder === roadOwner) && !atWar(view, roadOwner);
  // A unit that starts its turn on a road gets extra moves (+1, +2 with Railways): say so where the number is.
  const onRoadTile = mine && !!view.tiles[tileIndex(unit.pos, view.width, view.height)]?.road;
  const level = unit.level ?? 0;
  // An idle unit's strength as the forecast counts it: the core's own loss, off the strength shown.
  const idleLoss = unit.unsupplied ? unit.unsupplied_loss ?? 0 : 0;
  const fullStrength = unit.strength + strengthBonus(unit.strength, level);
  const teammate = !mine && owner?.team === view.team;
  const exploring = isExploring(unit, orders);
  const pending = pendingUnitOrder(orders, unit.id);
  const asking = useGame((st) => st.heirDisbandAsk) === unit.id;
  // The Heal order that just ended at full health: the card and the Next decision button say so, not only the report.
  const healedLastTurn = useGame((st) => !!st.report?.events?.[st.seat]?.some((e) => e.type === 'UnitHealed' && e.unit === unit.id));
  // A garrison its city's occupation lock just released: it asks for orders like an idle unit until answered.
  const releasedFrom = useGame((st) => st.report?.released?.[st.seat]?.find((r) => r.unit === unit.id)?.city);
  const released = !pending && !!releasedFrom && unit.occupation_lock == null;
  const standing = pending ? (pending.type === 'SleepUnit' ? (pending.until_healed ? 'Heal' : 'Sleep') : null) : released && unit.standing_order === 'Sleep' ? null : unit.standing_order;
  const errs = errorsWhere(orders, errors, (o: Order) => orderUnit(o) === unit.id);
  const stats = UNIT_STATS[unit.kind];
  // A mounted unit that also shoots (the Keshig): it charges or shoots, one order a turn.
  const bow = unit.ranged > 0 && attacksInMelee(unit.kind);
  // A hired Company is its seller's version of the unit (its name and picture follow `origin`).
  const madeBy = unit.origin ?? owner?.faction;
  const fromLabel = itemName({ kind: 'Unit', unit: unit.kind }, madeBy);
  const col = playerInk(unit.owner);
  // The Sparta's one-time unit action: a veteran (at RULES.spartaRetireLevel or higher) is offered retirement, valid only in a city with an Agoge.
  const veteran = mine ? retireEntry(view, unit.id) : undefined;
  const retiring = !!pendingRetire(orders, unit.id);
  const locked = mine && unit.occupation_lock != null;
  const pendingText = pending ? describeOrder(pending, view).replace(`${fromLabel}: `, '') : '';
  const status = mode === 'move' && mine && !over ? 'Choose a tile · Esc to cancel' : retiring ? 'Retiring with honour · leaves the army when the turn ends'
    : pending
    ? pendingText.charAt(0).toUpperCase() + pendingText.slice(1)
    : exploring ? 'Auto-exploring' : unit.goto
      ? `Moving to ${placeName(view, unit.goto, Infinity) ?? 'the marked tile'}`
      : released ? `${releasedFrom} is secure, needs orders`
      : standing === 'Sleep' || standing === 'Heal' ? sleepOrderText(unit, standing === 'Heal')
      : standing === 'AutoFire' ? 'Auto-fire: holding this city and shooting visible enemies in range'
      : mine && !over ? locked ? '' : healedLastTurn ? 'Healed to full, needs orders' : 'Needs orders' : '';
  // An order that won't resolve: its reason, as a sentence, under the unit's status.
  const broken = mine && errs.length > 0;
  const why = errs.map((m) => `${m.charAt(0).toUpperCase()}${m.slice(1)}${/[.!?]$/.test(m) ? '' : '.'}`).join(' ');
  const spend = pendingSpend(view, orders ?? [], errors ?? []);
  const upgradeOrder: Order = { type: 'UpgradeUnit', unit: unit.id };
  const siegeOrder: Order = { type: 'UpgradeSiege', unit: unit.id };
  const upgradeAt = orders?.findIndex((o) => o.type === 'UpgradeUnit' && o.unit === unit.id) ?? -1;
  const siegeAt = orders?.findIndex((o) => o.type === 'UpgradeSiege' && o.unit === unit.id) ?? -1;
  const upgradeBalance = spend.available(upgradeOrder, upgradeAt < 0 ? orders?.length ?? 0 : upgradeAt);
  const siegeBalance = spend.available(siegeOrder, siegeAt < 0 ? orders?.length ?? 0 : siegeAt);
  const gold = upgradeBalance.gold;
  const siegeGold = siegeBalance.gold;
  const upgrade = isCompany(unit) ? null : upgradeCheck(unit, view, gold, upgradeBalance.supply);
  const siegeBlocked = unit.siege_offer?.blocker ?? (unit.siege_offer && siegeGold < unit.siege_offer.price.total ? `Needs ${unit.siege_offer.price.total} Gold (${goldOnHand(siegeGold, view)})`
    : unit.siege_offer && SIEGE_USES_IRON && (siegeBalance.supply.get('Iron') ?? 0) >= (view.supply?.find((s) => s.resource === 'Iron')?.spare ?? 0) ? 'No Iron to spare after your queued orders.' : null);
  const roleAction = unit.kind === 'Scout' || unit.kind === 'Settler';
  const improvePrimary = !roleAction && upgrade && !upgrade.blocker ? 'upgrade'
    : !roleAction && unit.siege_offer && !siegeBlocked ? 'siege' : null;
  // Occupation lock: a unit holding its captured city fortifies by itself, so the card offers only what the lock still allows (Heal, Upgrade, a shot).
  const movePrimary = !roleAction && !improvePrimary;
  // From the player's knowledge only: an order that is certain to fail is not offered.
  const canFound = mine && isFounderCandidate(view, unit);
  const foundBlocked = canFound ? foundCityUnitBlocker(view, unit) : null;
  // Heal is offered to a damaged unit (or one already healing, to stop it); where nothing heals it says why, right under the row.
  const healBlocked = mine ? healBlocker(unit) : null;
  const healOffered = mine && (unit.hp < 100 || standing === 'Heal');
  const healForecastText = healForecast(unit);
  const healReasonId = `heal-why-${unit.id}`;

  // A Scout leads with it; any other land military unit has it beside Move, never as the primary.
  // A garrison held by the occupation lock is offered no movement orders, and exploring is one (its card leads with the lock).
  const exploreAction = canAutoExplore(unit.kind) && !locked && (
    <Action label={exploring ? 'Stop auto-explore' : 'Auto-explore'} hotkey="E" primary={leads && unit.kind === 'Scout'} active={exploring}
      onClick={toggleAutoExploreSelected}
      tip="Explore reachable land and ruins using known terrain. Avoid known armies, retreat when hurt, and stop below 60 HP or when threatened or blocked. Any manual order takes control; the map marks exploring units with A." />
  );

  // The phone sheet reserves this row outside its scrolling details. Keep the same
  // command entry points (validation, advance and feedback) as the full panel.
  if (peek) return <div className={`phone-primary-actions${mine && unit.kind === 'Scout' ? ' phone-scout-actions' : ''}`} aria-label={`${unit.kind} primary actions`}>
    {mine ? <>
      {/* A Scout's primary action is exploring, as in the full panel; Wait stays one tap away. */}
      {unit.kind === 'Scout' && <button className="btn primary" data-primary-action disabled={view.finished} aria-pressed={exploring}
        onClick={toggleAutoExploreSelected}>{exploring ? 'Stop auto-explore' : 'Auto-explore'}</button>}
      <button className={`btn${unit.kind === 'Scout' ? '' : ' primary'}`} data-primary-action={unit.kind === 'Scout' ? undefined : true} disabled={view.finished || !!foundBlocked}
        aria-describedby={foundBlocked ? 'phone-found-blocked' : undefined}
        aria-pressed={canFound ? pending?.type === 'FoundCity' : standing === 'Sleep'}
        onClick={canFound ? foundCitySelected : sleepSelected}>
        {canFound ? 'Found City' : unit.military && unit.kind !== 'Scout' ? 'Fortify' : 'Wait'}
        {/* The peek row has no room for a sentence: the short reason, in the button; the full one is in the sheet. */}
        {foundBlocked && <small className="phone-action-why" id="phone-found-blocked">{foundBlocked.split(':')[0].replace(/\.$/, '')}</small>}
      </button>
      {unit.kind !== 'Scout' && canAutoExplore(unit.kind) && !locked && <button className="btn" disabled={view.finished} aria-pressed={exploring}
        onClick={toggleAutoExploreSelected}>{exploring ? 'Stop auto-explore' : 'Auto-explore'}</button>}
      <button className="btn" disabled={view.finished} aria-pressed={pending?.type === 'SkipUnit'} onClick={skipSelected}>Skip this turn</button>
    </> : <p className="hint">{teammate ? `Commanded by ${owner?.name}` : stats.blurb}</p>}
  </div>;

  return (
    <div className="panel unit-panel refined-unit-panel paper-grain" style={{ ['--pc' as string]: col }}>
      <div className="up-head">
        <UnitBadge unit={unit} faction={madeBy} />
        <div className="up-title">
          <div className="up-kind" title={fromLabel}>{fromLabel}</div>
          <div className="up-owner">
            <PlayerBadge id={unit.owner} /> {ownerName(view, unit.owner)}
            {teammate && <span className="muted"> · teammate</span>}
            {unitClass(unit.kind as UnitKind) && <> · <Tip content={<ClassTip kind={unit.kind as UnitKind} bow={bow} />} className="up-class">{CLASS_NAME[unitClass(unit.kind as UnitKind)!]}</Tip></>}
            {unit.military && <> · <Tip content={`${eraName(unitEra(unit.kind as UnitKind))} era. Every military miniature wears this numeral on its base, so a newer unit of the same class reads apart on the map.`} className="up-era"><EraTab era={unitEra(unit.kind as UnitKind)} /></Tip></>}
          </div>
          {unit.military && <PromotionLine unit={unit} faction={owner?.faction} />}
          {isCompany(unit) && <CompanyTag unit={unit} view={view} />}
        </div>
        {status && <div className={`up-status ${broken ? 'bad' : pending ? 'pending' : exploring || unit.goto || standing ? 'standing' : ''}`}><Amounts text={status} />{broken && ' · won’t resolve'}</div>}
        {locked && <p className="up-status-why up-lock" data-testid="occupation-lock"><Icon name="mark-lock" size={14} className="ui-mark" /> {occupationLead(view, unit)}</p>}
        {locked && <p className="up-status-why up-lock-note" data-testid="occupation-lock-note">{occupationNote(view, unit)}</p>}
        {broken && <p className="up-status-why" data-testid="unit-order-error">{why}</p>}
        {unit.unsupplied && <p className="up-status-why up-unsupplied" data-testid="unit-unsupplied"><Icon name="ui-alert" size={12} className="ui-mark" /> Idle: you are short of {unit.unsupplied}. It fights {UNSUPPLIED_STRENGTH_PERCENT}% weaker until supply returns.</p>}
        {onRoad && <p className="up-status-why road" data-testid="open-road-note"><Icon name="building-yamrelaystation" size={14} /> The open road: {ownerName(view, roadOwner!)}’s Yam Relay Station lets every empire cross their land at Peace. If the Hall changes hands this land closes behind you.</p>}
        {unit.military && unit.fortify > 0 && <Tip className="up-resting" content="Resting military units gain +3 defence per full turn without moving or fighting, up to +6. Movement or combat clears it.">Resting defence +{unit.fortify}</Tip>}
      </div>
      <div className="unit-panel-body" ref={scroll} tabIndex={overflow ? 0 : undefined} role="region" aria-label="Unit details and actions">
      <div className="up-stats">
        <Tip content={ignoresWounds(owner?.faction, unit.kind) ? 'Health. Sparta infantry ignore wound strength penalties.' : 'Health. Strength drops by 1 per 10 HP lost.'} className="up-stat">
          <span className="lbl">HP</span>
          <Bar value={unit.hp} max={100} color={healthColor(unit.hp / 100)} className="hp-bar" />
          <b>{unit.hp}</b>
        </Tip>
        <Tip content={`Movement points per turn. Forest and hills cost 2; a unit can always take one step at full moves.${onRoadTile ? ` This unit stands on a road: ${unit.max_moves} moves, ${roadBonusText(view)} on a road. A unit that starts its turn on a road gets the bonus.` : ''}`} className="up-stat">
          <span className="lbl"><RingMedallion icon="mark-move" tone="neutral" size={22} />Moves</span>
          <b>
            {unit.moves_left}/{unit.max_moves}
            {onRoadTile && unit.moves_left > unit.max_moves && <span className="muted"> {roadBonusText(view).replace(' (Railways)', '')} road</span>}
          </b>
        </Tip>
        {unit.military && (
          <>
            <Tip content={`Melee strength (attack and defence): ${unit.strength} for the unit${level ? `, promotion level ${level} +${strengthBonus(unit.strength, level)}` : ''}.${idleLoss ? ` No ${unit.unsupplied} supply: it fights ${UNSUPPLIED_STRENGTH_PERCENT}% weaker (−${idleLoss}) until supply returns.` : ''}`} className="up-stat">
              <span className="lbl"><RingMedallion icon="unit-spearman" tone="neutral" size={22} />Strength</span>
              <b className={idleLoss ? 'bad' : undefined} data-testid={idleLoss ? 'unit-strength-idle' : undefined}>{idleLoss ? fullStrength - idleLoss : fullStrength}</b>
            </Tip>
            {idleLoss > 0 && <div className="up-strength-was muted" data-testid="unit-strength-was">was {fullStrength}: no {unit.unsupplied}</div>}
          </>
        )}
        {mine && unit.sight != null && (
          <Tip content={`How far this unit sees: ${unit.sight} tiles. Hills give +1 sight and see over Forest and Hills. Mountains still block.`} className="up-stat">
            <span className="lbl"><RingMedallion icon="effect-insight" tone="neutral" size={22} />Sight</span>
            <b>{unit.sight}</b>
          </Tip>
        )}
        {unit.ranged > 0 && (
          <Tip content={`Ranged strength, range ${unit.range}. Base ${unit.ranged}${level ? `, promotion level ${level} +${strengthBonus(unit.ranged, level)}` : ''}. Ranged attacks take no retaliation.`} className="up-stat">
            <span className="lbl"><RingMedallion icon="unit-archer" tone="neutral" size={22} />Ranged</span>
            <b>
              {unit.ranged + strengthBonus(unit.ranged, level)} <span className="muted">r{unit.range}</span>
            </b>
          </Tip>
        )}
        {strengthKey && <Tip className="up-strength-key" content={`Units this unit can attack this turn carry a chip on the map. A rising triangle shows how much stronger they are in that fight, a falling one how much weaker. ${barbarianStrengthKey ? "Barbarian chips use neutral ink because they don't attack the Celts. " : ''}It counts class, promotion and wounds, but not terrain or fortifying (the attack forecast has those).`}>
          <span data-testid="strength-key">Map: <b className={onlyBarbarianStrength ? undefined : 'bad'}>▲</b> stronger · <b className={onlyBarbarianStrength ? undefined : 'good'}>▼</b> weaker{barbarianStrengthKey && ' · barbarians don\'t attack you'}</span>
        </Tip>}
      </div>
      {mine && !over && (
        <div className="actions">
          {unit.kind === 'Scout' && exploreAction}
          {canFound && (
            <Action label="Found City" hotkey="B" primary={leads} active={pending?.type === 'FoundCity'} onClick={foundCitySelected}
              disabled={!!foundBlocked} reason={foundBlocked ?? undefined}
              tip={unit.kind === 'Settler' ? `Found a city here, on land you or nobody owns. Cities need ${minCityGap(owner?.faction ?? 'Athens')} tiles between them.` : `Found a city here and consume this level-${unit.level} veteran. Combat resolves first: if this unit is wounded or killed, the city does not appear, and a surviving unit stays. A Legionary starts the city at population 2.`} />
          )}
          {movePrimary && !locked && <Action label={mode === 'move' ? 'Cancel targeting' : unit.military && attacksInMelee(unit.kind) ? 'Move or attack' : 'Move'} primary={leads}
            onClick={toggleMoveSelected} active={mode === 'move'} hotkey={mode === 'move' ? 'Esc' : undefined} 
            tip="Choose a tile on the map. Your route and combat forecast appear before you click. Esc cancels targeting. You can also right-click a destination, or drag from the unit to plot a path." />}
          {unit.kind !== 'Scout' && exploreAction}
          {unit.ranged > 0 && (
            <Action label="Ranged Attack" disabled={closed} reason={closed ? "Leave closed foreign territory before attacking" : undefined} hotkey="R" active={mode === 'ranged' || pending?.type === 'RangedAttack'} onClick={toggleRangedMode} tip={bow ? `Shoot a target ${unit.range === 1 ? 'next to' : `within ${unit.range} tiles of`} this ${fromLabel}, from where it stands: no return fire, and it never moves. A shot cannot take a city. Right-click the target instead to charge it. One order a turn.` : `Pick a target within ${unit.range} tiles. Only if the unit doesn't move this turn.`} />
          )}
          {unit.ranged > 0 && view.cities.some((c) => c.owner === view.me && c.pos.q === unit.pos.q && c.pos.r === unit.pos.r) && (
            <Action label={standing === 'AutoFire' ? 'Stop auto-fire' : 'Auto-fire from city'} icon="mark-autofire" hotkey="V"
              active={standing === 'AutoFire' || pending?.type === 'SetAutoFire' && pending.enabled}
              onClick={toggleAutoFireSelected}
              tip={`Hold this city and shoot the best enemy in range each turn.${barbariansSpareMe(view) ? ' It never shoots barbarians, because they don\'t attack you.' : ''} It never moves, declares War or spends Gold. Move the unit or turn this off to take control; a shot you order sets this turn’s target.`} />
          )}
          {/* The idle orders sit right under the unit's own actions, above Improve, so a 600 px window shows them without scrolling. Fortify for military units, Wait for civilians, and Skip. */}
          <div className="unit-orders">
            <div className={`act-pair${healOffered ? ' has-heal' : ''}`}>
              {!locked && <Action label={unit.military && unit.kind !== 'Scout' ? 'Fortify' : 'Wait'} icon={unit.military && unit.kind !== 'Scout' ? 'fortify' : 'mark-sleep'} hotkey="F" active={standing === 'Sleep'} onClick={sleepSelected}
                tip={`Stay here${unit.military && unit.kind !== 'Scout' ? ', healing and fortifying' : ', healing and resting'}, until you give another order or an enemy military unit newly comes into sight within this unit's movement range. Enemies already that close when you choose this order do not wake it again.${closed ? ' No fortification or healing in closed foreign territory.' : ''}${unit.ranged > 0 ? ' A unit that is waiting does not fire.' : ''}`} />}
              {healOffered && <Action label="Heal" icon="mark-heal" hotkey="H" active={standing === 'Heal'} onClick={healSelected} disabled={!!healBlocked && standing !== 'Heal'} describedBy={healBlocked ? healReasonId : undefined}
                tip={<>
                  <p>Rests until at full health, then asks for orders again. {healForecastText ?? 'It heals nothing where it stands.'}</p>
                  <p>It fortifies as it rests and wakes early if an enemy military unit newly comes into sight within its movement range. Any new order replaces it.</p>
                </>} />}
              {!locked && <Action label="Skip this turn" icon="mark-skip" hotkey="Space" active={pending?.type === 'SkipUnit'} onClick={skipSelected} tip="Do nothing this turn. Cancels Fortify, Wait or Heal, or a destination, and the unit asks for orders again next turn." />}
            </div>
            {healOffered && healBlocked && standing !== 'Heal' && <p className="act-reason" id={healReasonId} data-testid="heal-blocked">{healBlocked}</p>}
          </div>
          <UnitImprovements unit={unit} view={view} check={upgrade} primary={leads ? improvePrimary : null} pending={pending?.type} gold={gold} siegeGold={siegeGold} siegeBlocker={siegeBlocked} />
          {veteran && <RetireAction unit={unit} view={view} retiring={retiring} blocker={veteran.blocker} />}
          <div className="unit-dismiss">
            {asking ? <HeirDisbandConfirm unit={unit} view={view} /> : locked ? null :
            <Action label="Disband" onClick={disbandSelected} danger active={pending?.type === 'DisbandUnit'} tip={unit.kind === 'HeirToTheThrone' ? 'Remove the Heir permanently. You forfeit his 2 VP, and Byzantium can train a new one.' : 'Remove this unit permanently (saves upkeep).'} quiet />}
            {pending && <Action label="Cancel order" hotkey="⌫" onClick={cancelSelected} quiet tip="Clear this unit's pending order." />}
          </div>
        </div>
      )}
      {/* Contextual notes stay in the scrolling side panel. */}
      <div className="up-hints">
        {unit.kind === 'HeirToTheThrone' && <HeirNote unit={unit} view={view} />}
        {isCompany(unit) && <CompanyNote unit={unit} view={view} />}
        {(unit.siege || builtInSiege(unit.kind as UnitKind)) && <div className="hint" data-testid="siege-note">{siegeNote(view, unit)}</div>}
        {firesIndirect(unit.kind) && <div className="hint">{ARTILLERY_RULE}</div>}
        {closed && <p className="hint warn">Closed foreign territory: no attacks, healing or fortification. Every move must bring this unit closer to an exit.</p>}
        {mine && !closed && unit.resting_heal && unit.hp < 100 && <Tip as="div" className="hint" content={<BreakdownTable b={unit.resting_heal} title="Resting healing" unit="HP" />}>Rest here: heal up to {Math.min(unit.resting_heal.total, 100 - unit.hp)} HP this turn if you neither move nor fight.</Tip>}
        {siegeCityBonus(unit.kind) > 0 && <div className="hint">{siegeCityHint(siegeCityBonus(unit.kind))}</div>}
        {mode === 'ranged' && <div className="hint warn">Choose a target within range <span className="mouse-only">· Esc to cancel</span><button className="link touch-only" onClick={toggleRangedMode}>Cancel targeting</button></div>}
        {teammate && (
          <div className="hint">
            Commanded by {owner?.name}.{' '}
            {ownerSeat >= 0 && (
              <button className="link" onClick={() => setSeat(ownerSeat)}>
                Switch seat
              </button>
            )}
          </div>
        )}
        {!mine && !teammate && <div className="hint">{stats.blurb}</div>}
      </div>

      </div>
      {overflow && <div className="unit-scroll-hint">Scroll for more unit actions and details ↕</div>}
    </div>
  );
}

/** Honour the Fallen: retire this veteran (at the retire level or higher). Offered only on a veteran; valid only in an own city with an Agoge, which the map marks while he is selected. */
function RetireAction({ unit, view, retiring, blocker }: { unit: UnitView; view: PlayerView; retiring: boolean; blocker: string | null }) {
  const set = useGame((s) => s.set);
  const jumpTo = useGame((s) => s.jumpTo);
  const editOrders = useGame((s) => s.editOrders);
  const r = view.actions!.retire!;
  const goal = FACTION_GOAL.Sparta, vp = factionGoalVp('Sparta');
  const agoge = view.cities.find((c) => r.shrines.includes(c.id));
  const reason = blocker && (agoge
    ? <>{blocker.split(':')[0]}. The map marks the cities where he can retire. <button className="link" onClick={() => jumpTo(agoge.pos)}>Show on map</button></>
    : 'Build an Agoge first: veterans retire in a city that has one.');
  return <Action label={goal.unitAction ?? 'Retire this veteran'} active={retiring} disabled={!!blocker && !retiring} reason={retiring ? undefined : reason ?? undefined}
    onClick={() => (retiring ? editOrders((o) => clearRetire(o, unit.id)) : set({ goalDialog: 'RetireVeteran', goalDialogUnit: unit.id }))}
    summary={retiring ? 'Ordered: he leaves the army when the turn ends. Click to cancel.' : blocker ? undefined : `${r.retired + 1} of ${r.target} toward ${goal.name} (+${vp} VP when the last is retired).`}
    tip={<>
      <p>{goal.name}: retire {r.target} veterans (promotion level {r.level} or higher) in a city with an Agoge. Each leaves your army for good; the {vp} VP is kept once the last is retired.</p>
      <p>{r.retired} of {r.target} retired so far.</p>
    </>} />;
}

/** Level, XP and what the next level needs. Levels are public; XP is the owner's. */
export function PromotionLine({ unit, faction, inline = false }: { unit: Pick<UnitView, 'level' | 'xp' | 'xp_next'>; faction?: Faction; /** The same words beside the unit's name, without the badge, for a card with no height to spare (the aircraft card). */ inline?: boolean }) {
  const level = unit.level ?? 0;
  const own = unit.xp != null;
  if (!level && !own) return null;
  const next = unit.xp_next ?? null;
  const elite = level === 6 || next !== null && level === 5;
  const p = RULES.promotion;
  const rule = `Each fight it survives earns XP: more against stronger enemies and double that for a kill, plus ${p.meleeXp} for melee or ${p.rangedXp} for ranged fire, plus ${p.initiatorXp} for attacking (at most ${p.unitFightCap}). Fights with barbarians give XP only until the first promotion, at ${p.xp[0]} XP. Attacking a city earns ${p.cityAttackXp}, and the attack that breaks it ${p.cityDefeatedXp}. Each level adds ${STRENGTH_PERCENT}% to the unit's strength (melee and ranged), up to level ${elite ? p.spartaMaxLevel : MAX_LEVEL}. Spartan military units start at level 1 and earn XP ${p.spartaXpPercent - 100}% faster. Upgrades keep promotions. Aircraft earn XP the same way from strikes and interceptions, and keep it when they rebase.`;
  const progress = !own ? null : next !== null ? `${next - unit.xp!} XP to level ${level + 1}.` : `${unit.xp} XP. Maximum level.`;
  const topRank = isTopRank(faction, level);
  const name = topRank && faction === 'Sparta' ? 'Spartan Veteran' : topRank ? 'Veteran' : level ? `Level ${level} · +${level * STRENGTH_PERCENT}%` : 'Not promoted';
  const detail = topRank ? `Level ${level} · +${level * STRENGTH_PERCENT}%` : own ? next !== null ? `${unit.xp}/${next} XP` : `${unit.xp} XP · max level` : '';
  const tip = <><p>{level ? `Promotion level ${level}: +${level * STRENGTH_PERCENT}% strength.` : 'Not promoted yet.'} {progress}</p><p>{rule}</p></>;
  if (inline) return <Tip as="span" className="up-rank-inline" content={tip}>{name}{detail && ` · ${detail}`}</Tip>;
  return (
    <Tip className={`up-rank${topRank ? ' elite' : ''}`} content={tip}>
      <RankMark level={level} faction={faction} />
      <span className="up-rank-text">
        <span>{name}</span>
        <span className="muted">{detail}</span>
      </span>
    </Tip>
  );
}

/** The class triangle in plain words, with the unit's own role first (pillar 5: the bonus is a number, shown in every forecast). */
function ClassTip({ kind, bow = false }: { kind: UnitKind; bow?: boolean }) {
  const class_ = unitClass(kind)!;
  // The triangle ends at the World War era (core `combat::TRIANGLE_END_ERA_TIER`): a Tank, Infantry or Combat Drone has no class bonus, so its tip says only that.
  if (unitEraTier(kind) >= RULES.triangleEndsAtEraTier) return <p><b>{CLASS_NAME[class_]}.</b> No class bonuses from the World War era: it differs by strength, cost and moves.</p>;
  return <>
    <p><b>{CLASS_NAME[class_]}.</b> {CLASS_NOTE[class_]}</p>
    {bow && <p>When it shoots it fights as a ranged unit: neither the melee line nor a mounted target gets a bonus against its arrows. When it charges it fights as mounted.</p>}
    <p>The triangle: the melee line beats mounted, ranged beats the melee line, mounted beats ranged in a charge. Siege supports.</p>
  </>;
}

const SIEGE_REACH = ['', 'Walls', 'Walls and Castles', 'Walls, Castles and Star Forts'];

/** What Siege does for this unit today: its owner's technology sets how high it reaches (core `data::siege_tier`); a rival's is private. The rule itself is `SIEGE_RULE`. An Industrial or later unit has it built in (core `data::built_in_siege`). */
function siegeNote(view: PlayerView, unit: UnitView): string {
  if (builtInSiege(unit.kind as UnitKind)) return `Siege built in: ${BUILT_IN_SIEGE_NOTE}.`;
  const tier = unit.owner === view.me ? siegeTier(view.research.known) : 0;
  return tier > 1 ? `Siege: ignores enemy ${SIEGE_REACH[tier]} when it attacks a city.` : 'Siege: ignores enemy Walls when it attacks a city.';
}
