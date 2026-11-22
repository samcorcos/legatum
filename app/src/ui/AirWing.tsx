import { useLayoutEffect } from 'react';
import { useGame } from '../store';
import { AIR_ORDERS, clearAirOrder, describeOrder, errorsWhere, setAirOrder } from '../orders';
import { RULES, UNIT_NAME, itemName, siegeBonusOf, unitStatsFor } from '../data';
import { distance, hexEq } from '../hex';
import { bearing } from '../empires';
import { ownerName } from '../barbarians';
import { healthColor, playerInk } from '../theme';
import { airHeadline, airState, isFighter, noTargetReason, rebaseOptions, reconBlocker, spotsIn, strikeRange, guardRange, type Spot } from '../airWing';
import { promptWar } from '../commands';
import type { AirPick, BattleForecast, CityView, Hex, Order, PlayerView, UnitKind } from '../types';

/** What an aircraft upgrade gives, in a few words, for the Improve row. */
function upgradeEffect(to: UnitKind): string {
  switch (to) {
    case 'Fighter': return `Guards ${guardRange('Fighter')} tiles; strikes ${strikeRange('Fighter')} tiles away`;
    case 'JetFighter': return `Guards ${guardRange('JetFighter')} tiles; strikes ${strikeRange('JetFighter')} tiles away`;
    case 'Bomber': return `Strikes ${strikeRange('Bomber')} tiles away, +${siegeBonusOf('Bomber')} against city HP`;
    case 'JetBomber': return `Strikes ${strikeRange('JetBomber')} tiles away, +${siegeBonusOf('JetBomber')} against city HP`;
    case 'StealthFighter': return 'Guards against Stealth Bombers';
    case 'StealthBomber': return 'Only a Stealth Fighter can stop it';
    default: return `Strikes ${strikeRange(to)} tiles away`;
  }
}
import { Amounts } from './Amounts';
import { Bar, Icon, Tip } from './index';
import { Action, UnitBadge } from './UnitAction';
import { PromotionLine } from './UnitPanel';
import { ImproveRow } from './UnitImprovements';
import { RingMedallion } from './ringMedallion';
import { useScrollHint } from './useScrollHint';
import { Star } from '../brand/ornaments';
import { PlayerBadge } from './PlayerBadge';
import { UNSUPPLIED_STRENGTH_PERCENT, needText, unitNeed } from '../resources';
import { pendingSpend, goldOnHand } from './productionPurchases';

/** `forecast` is null when core cannot forecast it yet: a War declared this turn (`spot.surprise`), or Peace (`spot.peace`: declare War first). */
type Target = { at: Hex; name: string; spot: Spot; peace: boolean; forecast: BattleForecast | null };

/** Everything this aircraft could hit now, best first, with the forecast (core `forecast_air_strike`), then the Peace targets that need a War declaration first. */
function strikeTargets(hostile: Spot[], peaceful: Spot[], forecast: (at: Hex) => BattleForecast | null): Target[] {
  const seen: Target[] = [];
  for (const s of hostile) {
    if (seen.some((t) => hexEq(t.at, s.at) && t.name === s.name)) continue;
    const f = forecast(s.at);
    if (f || s.surprise) seen.push({ at: s.at, name: s.name, spot: s, peace: false, forecast: f });
  }
  const damage = (t: Target) => t.forecast?.expected_damage_to_defender ?? -1;
  seen.sort((a, b) => damage(b) - damage(a));
  for (const s of peaceful) if (!seen.some((t) => hexEq(t.at, s.at) && t.name === s.name)) seen.push({ at: s.at, name: s.name, spot: s, peace: true, forecast: null });
  return seen;
}

/** Opening a picker scrolls it into view, so a long dock card never hides the list it just opened (8.9.25 UI review: the rebase list was clipped at 1000x600). */
const revealPicker = (el: HTMLElement | null) => el?.scrollIntoView?.({ block: 'nearest' });

/**
 * The shared left unit dock for one selected hangar aircraft: what it is doing now and
 * the things it can do: a strike (pick a target with its forecast, on the list or on the map), a recon of a tile
 * (lit tiles on the map), keeping a Fighter on guard, and a rebase to another city of yours with room. One order per aircraft per turn; a
 * Fighter that strikes has used its turn and does not intercept.
 */
export function AirWing({ city, view, planeId }: { city: CityView; view: PlayerView; planeId: number }) {
  const { scroll, overflow } = useScrollHint();
  useLayoutEffect(() => { if (scroll.current) scroll.current.scrollTop = 0; }, [planeId, scroll]);
  const planes = (view.aircraft ?? []).filter((a) => a.city === city.id && a.owner === view.me && a.id === planeId);
  const orders = useGame((s) => s.orders[s.seat]) ?? [];
  const errors = useGame((s) => s.errors[s.seat]) ?? [];
  const client = useGame((s) => s.client);
  const editOrders = useGame((s) => s.editOrders);
  const holdAir = useGame((s) => s.holdAir);
  const sleepAir = useGame((s) => s.sleepAir);
  const air = useGame((s) => s.air);
  const held = useGame((s) => s.airHeld[view.me]);
  const acceptedOrders = orders.filter((_, index) => !errors.some((error) => error.index === index));
  if (!planes.length) return null;
  const setAir = (plane: number | null, what: AirPick | null, hover: Hex | null = null) => useGame.getState().set({ air: { plane, what, hover } });
  const ring = (at: Hex | null) => { const s = useGame.getState(); s.set({ air: { ...s.air, hover: at } }); };
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const order = (o: Order) => { editOrders((ord) => setAirOrder(ord, o), 'commit'); setAir(null, null); };
  const cancel = (id: number) => editOrders((ord) => clearAirOrder(ord, id));
  // One primary button for the whole wing, on the first aircraft that still wants an answer: Strike when something is in reach
  // (hostile or at Peace), otherwise Hold, so a card with nothing to strike still shows what to do next.
  const asking = (p: typeof planes[number]) => { const s = airState(view, city, p, acceptedOrders, held); return s.kind === 'ready' || ((s.kind === 'guard' || s.kind === 'idle') && !s.held); };
  const firstAsking = planes.find(asking)?.id;
  const plane = planes[0];
  const why = errorsWhere(orders, errors, (o) => AIR_ORDERS.has(o.type) && 'plane' in o && o.plane === plane.id);
  const state = airState(view, city, plane, acceptedOrders, held);
  const fighter = isFighter(plane.kind);
  const range = strikeRange(plane.kind);
  const shown = air.plane === plane.id ? air.what : null;
  const reach = spotsIn(view, city, range, orders);
  const inReach = reach.hostile.length + reach.peaceful.length;
  const heldNow = (state.kind === 'guard' || state.kind === 'idle') && state.held;
  const asleep = (state.kind === 'guard' || state.kind === 'idle') && state.asleep;
  const unanswered = state.kind === 'ready' || ((state.kind === 'guard' || state.kind === 'idle') && !state.held);
  const rebases = rebaseOptions(view, city, orders, errors);
  const canRebase = rebases.some((r) => r.available);
  const reconWhy = reconBlocker(view, city, plane, orders);
  const name = itemName({ kind: 'Unit', unit: plane.kind }, faction);
  const supply = unitNeed(plane.kind);
  const hangar = (view.aircraft ?? []).filter((a) => a.city === city.id).length;
  const hangarCap = view.research.known.includes('StealthTechnology') ? 3 : RULES.airBaseCap;
  const ready = state.kind === 'ready';
  // Held for this turn only (an aircraft put to sleep is waiting, not answered).
  const answered = heldNow && !asleep;
  // What a strike order is aimed at, in words: "the Byzantium Spearman 3 tiles W".
  const struck = (to: Hex) => {
    const spot = [...reach.hostile, ...reach.peaceful].find((s) => hexEq(s.at, to));
    const [what, owner] = spot?.name.split(' · ') ?? [];
    return `${!spot ? 'the target' : spot.city ? spot.name : `the ${owner} ${what}`} ${distance(to, city.pos)} tiles ${bearing(city.pos, to)}`;
  };
  const noTarget = noTargetReason(view, city, plane.kind, orders);
  const hint = state.kind === 'ordered'
    ? (state.order.type === 'AirStrike' ? `Strikes ${struck(state.order.target)}.`
      : state.order.type === 'AirRecon' ? `Scouts the tile ${distance(state.order.target, city.pos)} tiles ${bearing(city.pos, state.order.target)}: you see the ${RULES.reconRadius} tiles around it next turn only, so a strike can be ordered then.${fighter ? ' It still guards this turn.' : ''}`
      : <Amounts text={describeOrder(state.order, view)} />)
    : state.kind === 'ready'
      ? (fighter ? 'Strike with no retaliation (its own strength, no city bonus), or stay on guard to intercept. A Fighter that strikes does not intercept this turn.' : `Strike with no retaliation, +${siegeBonusOf(plane.kind)} against city HP. ${plane.kind === 'StealthBomber' ? 'Only a Stealth Fighter on guard can stop the strike.' : 'A Fighter on guard can intercept the strike.'}`)
      : state.kind === 'guard'
        ? (asleep ? `Waiting on guard: it intercepts enemy air strikes and wakes when an enemy is within ${range} tiles.`
          : heldNow ? `On guard: it intercepts enemy air strikes within ${guardRange(plane.kind)} tiles of this city${plane.kind === 'StealthFighter' ? ', Stealth Bombers included' : ', except a Stealth Bomber’s strike'}.`
            : `${noTarget} Stay on guard to intercept enemy air strikes${canRebase ? ', or rebase it' : ''}.`)
        : (asleep ? `Waiting: it wakes when an enemy is within ${range} tiles.` : heldNow ? 'Holding this turn.' : `${noTarget} Wait for a target, ${canRebase ? 'rebase it, ' : ''}or skip this turn.`);
  const holdLabel = asleep ? 'Wake' : fighter ? 'Stay on guard' : 'Wait for target';
  // Hold (F) and Skip (Space) end in the same order when a target is in reach, so that card shows only the one that names the aircraft's job.
  const showHold = asleep || (unanswered && (!ready || fighter)) || (answered && fighter);
  const showSkip = (unanswered && !(ready && fighter)) || (answered && !fighter);
  // The status line, from the refused order itself when one will not resolve; an aircraft that still needs an answer says so.
  const refused = orders.find((o, i) => AIR_ORDERS.has(o.type) && 'plane' in o && o.plane === plane.id && errors.some((e) => e.index === i));
  const refusedText = refused?.type === 'Rebase' ? `Rebasing to ${view.cities.find((c) => c.id === refused.to)?.name ?? 'another city'}`
    : refused ? describeOrder(refused, view).replace(/^[^:]*: /, '').replace(/^./, (c) => c.toUpperCase()) : '';
  const headline = airHeadline(plane, city, state, view);
  const status = why.length ? `${refusedText} · won’t resolve`
    : state.kind === 'guard' && unanswered ? `Needs orders · on guard over ${guardRange(plane.kind)} tiles around ${city.name}`
    : state.kind === 'idle' && unanswered ? `Needs orders · ${state.peaceful.length ? headline : `no targets within ${range} tiles`}`
    : headline;
  const stats = unitStatsFor(view.players.find((p) => p.id === plane.owner)!.faction, plane.kind);
  const beforeUpgrade = pendingSpend(view, orders, errors).available({ type: 'UpgradeAircraft', plane: plane.id });
  // One row per line it can upgrade along (a Biplane: Fighter or Bomber, the player's choice). The new form's resource is a
  // supply it keeps using, not part of the price: the price is Gold, and a refusal says which need is short.
  const upgrades = state.kind === 'ordered' ? [] : (plane.upgrade_options ?? []).map((option) => {
    const need = unitNeed(option.to);
    const spare = need && need !== supply ? (view.supply?.find((s) => s.resource === need)?.spare ?? 0) - (beforeUpgrade.supply.get(need) ?? 0) : 1;
    const blocker = beforeUpgrade.gold < option.cost ? `Not enough Gold: the upgrade costs ${option.cost} and ${goldOnHand(beforeUpgrade.gold, view)}.`
      : spare <= 0 ? `Needs 1 spare ${need} supply, and you have none.` : null;
    return { ...option, need, blocker, name: UNIT_NAME[option.to] };
  });
  return <section aria-label="Aircraft unit card" data-testid="air-wing" className="panel unit-panel refined-unit-panel paper-grain" style={{ ['--pc' as string]: playerInk(plane.owner) }}
    data-selected-plane={planeId} data-plane={plane.id} data-air-state={state.kind}>
    <div className="up-head">
      <UnitBadge unit={{ kind: plane.kind, owner: plane.owner, military: true }} faction={faction} />
      <div className="up-title">
        <div className="up-kind-row"><div className="up-kind" title={name}>{name}</div><PromotionLine unit={plane} faction={faction} inline /></div>
        <div className="up-owner"><PlayerBadge id={plane.owner} /> {ownerName(view, plane.owner)} · Air · {city.name}</div>
      </div>
      <div className={`up-status ${why.length ? 'bad' : state.kind === 'ordered' ? 'pending' : unanswered ? '' : 'standing'}`}>{status}</div>
      {why.length > 0 && <p className="up-status-why" data-testid="unit-order-error">{why[0]}</p>}
      {plane.unsupplied && <p className="up-status-why up-unsupplied" data-testid="unit-unsupplied"><Icon name="ui-alert" size={12} className="ui-mark" /> No {plane.unsupplied} supply: strikes and interceptions are {UNSUPPLIED_STRENGTH_PERCENT}% weaker.</p>}
    </div>
    <div className="unit-panel-body" ref={scroll} tabIndex={overflow ? 0 : undefined} role="region" aria-label="Unit details and actions">
      <div className="up-stats">
        <Tip content="Health." className="up-stat">
          <span className="lbl">HP</span>
          <Bar value={plane.hp} max={100} color={healthColor(plane.hp / 100)} className="hp-bar" />
          <b>{plane.hp}</b>
        </Tip>
        <Tip content={fighter ? `Strikes up to ${range} tiles from its city, and guards ${guardRange(plane.kind)} tiles around it.` : `Strikes up to ${range} tiles from its city.`} className="up-stat">
          <span className="lbl"><RingMedallion icon="unit-ranger" tone="neutral" size={22} />Range</span>
          <b>{range}</b>
        </Tip>
        <Tip content={`Air strike strength ${stats.ranged}${fighter ? ', with no city bonus' : `, +${siegeBonusOf(plane.kind)} against city HP`}. Strikes take no retaliation.`} className="up-stat">
          <span className="lbl"><RingMedallion icon="unit-soldier" tone="neutral" size={22} />Strength</span>
          <b>{stats.ranged}</b>
        </Tip>
        <Tip content={`Aircraft based in ${city.name}: ${hangar} of ${hangarCap} hangar slots used.`} className="up-stat">
          <span className="lbl"><RingMedallion icon="unit-bomber" tone="neutral" size={22} />Hangar</span>
          <b>{hangar}/{hangarCap}</b>
        </Tip>
        {supply && <Tip content={`Uses 1 ${supply} supply while based.`} className="up-stat up-stat-wide">
          <span className="lbl"><RingMedallion icon={`resource-${supply.toLowerCase()}`} tone="neutral" size={22} />Supply</span>
          <b><Amounts text={needText(supply)} /></b>
        </Tip>}
      </div>
      <div className="actions">
        {state.kind !== 'ordered' && inReach > 0 && <Action label="Strike…" hotkey="S" keyShortcuts="S" primary={unanswered && plane.id === firstAsking && shown !== 'rebase'} expanded={shown === 'strike'} disabled={view.finished}
          onClick={() => (shown === 'strike' ? setAir(null, null) : setAir(plane.id, 'strike'))}
          tip={`Pick a target within ${range} tiles, on the list or on the map. A strike takes no retaliation.${fighter ? ' A Fighter that strikes does not intercept this turn.' : ''}`} />}
        {state.kind !== 'ordered' && <Action label="Recon…" icon="unit-scout" hotkey="A" keyShortcuts="A" expanded={shown === 'recon'} disabled={!!reconWhy}
          reason={reconWhy ?? undefined}
          onClick={() => (shown === 'recon' ? setAir(null, null) : setAir(plane.id, 'recon'))}
          tip={`Scout a tile up to ${range} tiles from ${city.name}: you see every tile within ${RULES.reconRadius} of it (${3 * RULES.reconRadius * (RULES.reconRadius + 1) + 1} tiles) next turn only, terrain ignored, so you can order a strike then. It uses this aircraft's order, so it cannot strike this turn${fighter ? ', but a Fighter still guards' : ''}.`} />}
        {upgrades.length > 0 && <section className="unit-improve" aria-label="Improve unit">
          <h3><Star size={10} /> Improve <span /></h3>
          {upgrades.length > 1 && <p className="act-reason improve-reason">Choose one: Fighter or Bomber. The upgrade cannot be undone.</p>}
          {upgrades.map((u) => <div key={u.via}>
            <ImproveRow label={`Upgrade to ${u.name}`} effect={upgradeEffect(u.to)} icon="ui-trend-up" cost={u.cost} resource={u.need ?? undefined}
              primary={false} active={false} blocked={view.finished || !!u.blocker} reasonId={u.blocker ? `upgrade-why-${plane.id}-${u.via}` : undefined}
              onClick={() => order({ type: 'UpgradeAircraft', plane: plane.id, ...(upgrades.length > 1 ? { to: u.via } : {}) })}
              tip={<>{u.need && <p>A {u.name} uses 1 {u.need} supply while it exists.</p>}<p>Treasury: <Amounts text={`${beforeUpgrade.gold} Gold → ${beforeUpgrade.gold - u.cost} Gold`} />.</p></>} />
            {u.blocker && <p className="act-reason improve-reason" id={`upgrade-why-${plane.id}-${u.via}`}>{u.blocker}</p>}
          </div>)}
        </section>}
        <div className="unit-orders">
          {(showHold || showSkip) && <div className="act-pair">
            {showHold && <Action label={holdLabel} icon={fighter && !asleep ? 'fortify' : 'mark-sleep'} hotkey="F" keyShortcuts="F" disabled={view.finished}
              primary={unanswered && inReach === 0 && plane.id === firstAsking && !shown} active={answered}
              onClick={() => asleep || (unanswered && !ready) ? sleepAir(plane.id) : holdAir(plane.id)}
              tip={asleep ? 'Ask for orders again now.' : fighter ? `Stay on guard: it intercepts enemy air strikes within ${guardRange(plane.kind)} tiles of this city, and wakes when an enemy comes within ${range} tiles.` : `Wait: it stops asking each turn and wakes when an enemy is within ${range} tiles.`} />}
            {showSkip && <Action label="Skip this turn" icon="mark-skip" hotkey="Space" keyShortcuts="Space" disabled={view.finished} active={answered}
              onClick={() => holdAir(plane.id)}
              tip="Do nothing this turn. It asks for orders again next turn." />}
          </div>}
          {state.kind !== 'ordered' && <Action label="Rebase…" icon="mark-move" hotkey="R" keyShortcuts="R" expanded={shown === 'rebase'} disabled={view.finished || !canRebase}
            reason={canRebase ? undefined : rebases.length ? 'Rebase unavailable: every other city hangar is full.' : 'Rebase unavailable: no other city of yours.'}
            onClick={() => (shown === 'rebase' ? setAir(null, null) : setAir(plane.id, 'rebase'))}
            tip="Move it to another city of yours with a free hangar slot." />}
          {state.kind === 'ordered' && <div className="unit-dismiss">
            <Action label="Cancel order" onClick={() => cancel(plane.id)} quiet tip="Clear this aircraft's order." />
          </div>}
        </div>
      </div>
      {shown === 'strike' && client && <ul ref={revealPicker} className="air-targets" aria-label={`${name} targets within ${range} tiles`}>
        {(() => {
          const targets = strikeTargets(reach.hostile, reach.peaceful, (at) => client.forecast(view.me, plane.id, at, 'Ranged'));
          return targets.length === 0
            ? <li className="hint">{noTargetReason(view, city, plane.kind, orders)}</li>
            : targets.map((t) => <li key={`${t.at.q},${t.at.r},${t.name}`}>
              <button className={`air-target${air.hover && hexEq(air.hover, t.at) ? ' lit' : ''}`} onPointerEnter={() => ring(t.at)} onPointerLeave={() => ring(null)} onFocus={() => ring(t.at)} onBlur={() => ring(null)}
                onClick={() => (t.peace ? promptWar(t.spot.team, plane.id, t.at, true, true) : order({ type: 'AirStrike', plane: plane.id, target: t.at }))}>
                <b>{t.name}</b>
                <span className="muted air-where">{distance(t.at, city.pos)} tiles {bearing(city.pos, t.at)}</span>
                <span className="air-forecast">{t.peace ? 'At Peace: Declare War and strike'
                  : !t.forecast ? 'War declared this turn: it resolves after the declaration'
                  : t.forecast.air_defence ? <>Air Defence meets it and the strike does not land: <Amounts text={`about −${t.forecast.air_defence.expected_damage_to_attacker} HP`} /> to your {name}{t.forecast.air_defence.loss_chance > 0 ? ` · ${t.forecast.air_defence.loss_chance}% to lose it` : ''} · its strength {t.forecast.air_defence.interceptor_strength} against your {t.forecast.air_defence.striker_strength}</>
                  : t.forecast.win_chance >= 100 ? 'Destroys it' : <><Amounts text={`about −${t.forecast.expected_damage_to_defender} HP`} />{t.forecast.win_chance > 0 ? ` · ${t.forecast.win_chance}% to destroy it` : ''}{t.forecast.city_hp_percent != null ? ` · ${name}s deal ${t.forecast.city_hp_percent}% damage to city HP` : ''}</>}</span>
              </button>
            </li>);
        })()}
      </ul>}
      {shown === 'rebase' && <ul ref={revealPicker} className="air-targets" aria-label={`${name} rebase cities`}>
        {rebases.map(({ city: destination, count, queued, cap, available }) => <li key={destination.id}>
          <button className="air-target" disabled={!available} onPointerEnter={() => ring(destination.pos)} onPointerLeave={() => ring(null)} onFocus={() => ring(destination.pos)} onBlur={() => ring(null)}
            onClick={() => order({ type: 'Rebase', plane: plane.id, to: destination.id })}>
            <b>{destination.name}</b><span className="muted air-where">{distance(destination.pos, city.pos)} tiles away</span>
            {available
              ? <span className="air-forecast">{cap - count} of {cap} hangar slots free{queued ? ' after your queued rebase' : ''}</span>
              : <span className="hint warn">{destination.name}: hangar full, {count} of {cap}{queued ? ' with your queued rebase' : ' aircraft'}</span>}
          </button>
        </li>)}
      </ul>}
      {shown === 'recon' && <p className="hint" data-testid="recon-hint">Click a lit tile on the map to scout it: the {RULES.reconRadius} tiles around it are visible next turn only. Esc cancels.</p>}
      <div className="up-hints"><div className="hint" hidden={!!shown}>{hint}</div></div>
    </div>
    {overflow && <div className="unit-scroll-hint">Scroll for more unit actions and details ↕</div>}
  </section>;
}
